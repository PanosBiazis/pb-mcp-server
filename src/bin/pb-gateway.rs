use axum::{
    body::Bytes,
    extract::{ConnectInfo, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fs4::fs_std::FileExt;
use hmac::{Hmac, Mac};
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    fs::OpenOptions,
    io::Write,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;

struct AppState {
    token: Option<String>,
    signing_secret: Option<String>,
    signing_max_skew_ms: u64,
    nonce_seen: Mutex<HashMap<String, u64>>,
    trust_local_network: bool,
    project_root: PathBuf,
}

fn project_root() -> PathBuf {
    std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn require_token(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, &'static str)> {
    let Some(expected) = state.token.as_deref() else {
        return Ok(());
    };
    let Some(got) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    else {
        return Err((StatusCode::UNAUTHORIZED, "missing authorization header"));
    };

    // Accept either "Bearer <token>" or "<token>" to ease simple curl use.
    let got = got
        .strip_prefix("Bearer ")
        .or_else(|| got.strip_prefix("bearer "))
        .unwrap_or(got);

    // Constant-time compare to reduce token oracle leakage.
    if bool::from(got.as_bytes().ct_eq(expected.as_bytes())) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "invalid token"))
    }
}

fn is_local_network_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_loopback() || v4.is_private() {
                return true;
            }
            let o = v4.octets();
            // Tailscale CGNAT (100.64.0.0/10)
            o[0] == 100 && (64..=127).contains(&o[1])
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn canonical_sign_input(
    ts_ms: &str,
    nonce: &str,
    method: &Method,
    uri: &Uri,
    body_sha256_hex: &str,
) -> String {
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    format!(
        "{ts_ms}\n{nonce}\n{}\n{path}\n{query}\n{body_sha256_hex}",
        method.as_str().to_ascii_uppercase()
    )
}

fn require_signed_request(
    state: &AppState,
    headers: &HeaderMap,
    method: &Method,
    uri: &Uri,
    body: &[u8],
) -> Result<(), (StatusCode, &'static str)> {
    let Some(secret) = state.signing_secret.as_deref() else {
        return Ok(());
    };

    let ts = headers
        .get("x-pb-ts")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-ts"))?;
    let nonce = headers
        .get("x-pb-nonce")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-nonce"))?;
    let sig = headers
        .get("x-pb-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-signature"))?;

    let ts_u = ts
        .parse::<u64>()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid x-pb-ts"))?;
    let now = now_ms();
    let delta = now.abs_diff(ts_u);
    if delta > state.signing_max_skew_ms {
        return Err((
            StatusCode::UNAUTHORIZED,
            "signature timestamp out of window",
        ));
    }

    // Replay defense: reject nonce reuse inside skew window.
    {
        let mut map = state
            .nonce_seen
            .lock()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "nonce cache lock failed"))?;
        let min_ts = now.saturating_sub(state.signing_max_skew_ms);
        map.retain(|_, seen_ts| *seen_ts >= min_ts);
        if map.contains_key(nonce) {
            return Err((StatusCode::UNAUTHORIZED, "replayed nonce"));
        }
        map.insert(nonce.to_string(), now);
    }

    let body_hash = Sha256::digest(body);
    let body_hex = hex_lower(&body_hash);
    let input = canonical_sign_input(ts, nonce, method, uri, &body_hex);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid signing key"))?;
    mac.update(input.as_bytes());
    let expected = hex_lower(&mac.finalize().into_bytes());
    if !bool::from(sig.as_bytes().ct_eq(expected.as_bytes())) {
        return Err((StatusCode::FORBIDDEN, "bad request signature"));
    }
    Ok(())
}

fn authorize_request(
    state: &AppState,
    peer_ip: IpAddr,
    headers: &HeaderMap,
    method: &Method,
    uri: &Uri,
    body: &[u8],
) -> Result<(), (StatusCode, &'static str)> {
    if state.trust_local_network && is_local_network_ip(peer_ip) {
        return Ok(());
    }
    require_token(state, headers)?;
    require_signed_request(state, headers, method, uri, body)?;
    Ok(())
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        ok: true,
        service: "pb-gateway",
    })
}

#[derive(Serialize)]
struct StatusResponse {
    ok: bool,
    service: &'static str,
    project_root: String,
    peer: Option<String>,
    bus_exists: bool,
}

#[derive(Serialize)]
struct ProjectInfoResponse {
    ok: bool,
    service: &'static str,
    project_root: String,
    project_name: String,
    git_repo: bool,
    git_branch: Option<String>,
    top_level_entries: Vec<String>,
}

#[derive(Serialize)]
struct SecurityStatusResponse {
    ok: bool,
    service: &'static str,
    token_protection_enabled: bool,
    signed_requests_enabled: bool,
    signed_requests_max_skew_ms: u64,
    project_root_confined: bool,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct ReviewPacketAck {
    ok: bool,
    service: &'static str,
    stored: bool,
    bytes_appended: usize,
}

#[derive(Serialize)]
struct SearchResponse {
    ok: bool,
    service: &'static str,
    search_root: String,
    kind: &'static str,
    pattern: String,
    matches: usize,
    results: Vec<String>,
}

#[derive(Serialize)]
struct ListFilesResponse {
    ok: bool,
    service: &'static str,
    root: String,
    path: String,
    entries: Vec<String>,
}

#[derive(Serialize)]
struct ReadFileResponse {
    ok: bool,
    service: &'static str,
    path: String,
    text: String,
    truncated: bool,
}

#[derive(Serialize)]
struct GitStatusResponse {
    ok: bool,
    service: &'static str,
    repo: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Deserialize)]
struct ExecRequest {
    command: String,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

#[derive(Serialize)]
struct ExecResponse {
    ok: bool,
    service: &'static str,
    command: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteAgentMessage {
    id: String,
    ts: u64,
    from_agent: String,
    #[serde(default)]
    to_agent: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    kind: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct AgentSendRequest {
    from_agent: String,
    message: String,
    #[serde(default)]
    to_agent: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentRecentQuery {
    #[serde(default = "default_agent_recent_limit")]
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct AgentThreadQuery {
    topic: String,
    #[serde(default = "default_agent_recent_limit")]
    limit: usize,
}

fn default_agent_recent_limit() -> usize {
    60
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReviewPacket {
    schema_version: u32,
    project: String,
    task_id: String,
    created_at_ms: u64,
    agent: ReviewPacketAgent,
    summary: String,
    #[serde(default)]
    design_choices: Vec<String>,
    #[serde(default)]
    changed_files: Vec<String>,
    #[serde(default)]
    test_results: Option<String>,
    #[serde(default)]
    risks: Vec<String>,
    #[serde(default)]
    open_questions: Vec<String>,
    #[serde(default)]
    links: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReviewPacketAgent {
    id: String,
    #[serde(default)]
    kind: Option<String>,
}

fn review_log_path(root: &PathBuf) -> PathBuf {
    root.join(".mint-mcp").join("review-packets.jsonl")
}

fn review_lock_path(root: &PathBuf) -> PathBuf {
    root.join(".mint-mcp").join("review-packets.lock")
}

fn ensure_bus_dir(root: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(root.join(".mint-mcp"))
}

fn append_review_packet_jsonl(root: &PathBuf, json_line: &str) -> Result<usize, String> {
    ensure_bus_dir(root).map_err(|e| format!("create .mint-mcp dir: {e}"))?;

    let lockp = review_lock_path(root);
    let lockf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lockp)
        .map_err(|e| format!("open lock: {e}"))?;
    lockf
        .lock_exclusive()
        .map_err(|e| format!("lock exclusive: {e}"))?;

    let path = review_log_path(root);
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open log: {e}"))?;
    // Append one line + newline (JSONL).
    f.write_all(json_line.as_bytes())
        .and_then(|_| f.write_all(b"\n"))
        .map_err(|e| format!("append: {e}"))?;

    let _ = lockf.unlock();
    Ok(json_line.len() + 1)
}

fn read_review_packets_tail(root: &PathBuf, limit: usize) -> Result<Vec<String>, String> {
    let limit = limit.clamp(1, 200);
    ensure_bus_dir(root).map_err(|e| format!("create .mint-mcp dir: {e}"))?;
    let lockp = review_lock_path(root);
    let lockf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lockp)
        .map_err(|e| format!("open lock: {e}"))?;
    lockf
        .lock_shared()
        .map_err(|e| format!("lock shared: {e}"))?;

    let path = review_log_path(root);
    let s = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = lockf.unlock();
            return Ok(Vec::new());
        }
        Err(e) => {
            let _ = lockf.unlock();
            return Err(format!("read log: {e}"));
        }
    };
    let mut lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() > limit {
        lines = lines[lines.len() - limit..].to_vec();
    }
    let out = lines.into_iter().map(|l| l.to_string()).collect();
    let _ = lockf.unlock();
    Ok(out)
}

fn remote_bus_path(root: &PathBuf) -> PathBuf {
    root.join(".mint-mcp").join("remote-agent-bus.jsonl")
}

fn remote_bus_lock_path(root: &PathBuf) -> PathBuf {
    root.join(".mint-mcp").join("remote-agent-bus.lock")
}

fn sanitize_agent_name(s: &str) -> String {
    s.trim()
        .chars()
        .take(64)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn append_remote_agent_message(root: &PathBuf, msg: &RemoteAgentMessage) -> Result<(), String> {
    ensure_bus_dir(root).map_err(|e| format!("create .mint-mcp dir: {e}"))?;
    let lockf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(remote_bus_lock_path(root))
        .map_err(|e| format!("open lock: {e}"))?;
    lockf
        .lock_exclusive()
        .map_err(|e| format!("lock exclusive: {e}"))?;
    let line = serde_json::to_string(msg).map_err(|e| format!("serialize message: {e}"))?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(remote_bus_path(root))
        .map_err(|e| format!("open bus: {e}"))?;
    f.write_all(line.as_bytes())
        .and_then(|_| f.write_all(b"\n"))
        .map_err(|e| format!("append bus: {e}"))?;
    let _ = lockf.unlock();
    Ok(())
}

fn read_remote_agent_messages(root: &PathBuf) -> Result<Vec<RemoteAgentMessage>, String> {
    ensure_bus_dir(root).map_err(|e| format!("create .mint-mcp dir: {e}"))?;
    let lockf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(remote_bus_lock_path(root))
        .map_err(|e| format!("open lock: {e}"))?;
    lockf
        .lock_shared()
        .map_err(|e| format!("lock shared: {e}"))?;
    let s = match fs::read_to_string(remote_bus_path(root)) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = lockf.unlock();
            return Ok(Vec::new());
        }
        Err(e) => {
            let _ = lockf.unlock();
            return Err(format!("read bus: {e}"));
        }
    };
    let out = s
        .lines()
        .filter_map(|l| serde_json::from_str::<RemoteAgentMessage>(l).ok())
        .collect::<Vec<_>>();
    let _ = lockf.unlock();
    Ok(out)
}

fn resolve_under_root(base: &PathBuf, path: &str) -> Result<PathBuf, String> {
    let root = base.canonicalize().unwrap_or_else(|_| base.clone());
    if path.trim().is_empty() {
        return Ok(root);
    }
    let p = std::path::Path::new(path);
    let candidate = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    let resolved = candidate
        .canonicalize()
        .map_err(|e| format!("invalid path: {e}"))?;
    if resolved.starts_with(&root) {
        Ok(resolved)
    } else {
        Err("path escapes project root".to_string())
    }
}

fn read_text_limited(path: &std::path::Path, max_bytes: u64) -> Option<String> {
    let f = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    use std::io::Read;
    let mut take = f.take(max_bytes);
    take.read_to_end(&mut buf).ok()?;
    if buf.iter().any(|&b| b == 0) {
        return None;
    }
    String::from_utf8(buf).ok()
}

fn search_work(
    search_root: PathBuf,
    pattern: &str,
    literal: bool,
    case_sensitive: bool,
    max_matches: usize,
) -> Result<(usize, Vec<String>), String> {
    let max_matches = max_matches.clamp(1, 300);
    let re = if literal {
        RegexBuilder::new(&regex::escape(pattern))
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|e| format!("invalid literal pattern: {e}"))?
    } else {
        RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|e| format!("invalid regex pattern: {e}"))?
    };

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut count = 0usize;
    let mut out: Vec<String> = Vec::new();
    for entry in walk.build().flatten() {
        if count >= max_matches {
            break;
        }
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(text) = read_text_limited(p, 256 * 1024) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if count >= max_matches {
                break;
            }
            if re.is_match(line) {
                let rel = p
                    .strip_prefix(&search_root)
                    .map(|x| x.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_else(|_| p.display().to_string());
                out.push(format!("{rel}:{}:{line}", i + 1));
                count += 1;
            }
        }
    }
    Ok((count, out))
}

async fn status(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }

    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let bus = root.join(".mint-mcp").join("agent-bus.jsonl");

    Json(StatusResponse {
        ok: true,
        service: "pb-gateway",
        project_root: root.display().to_string(),
        peer: Some(peer.to_string()),
        bus_exists: bus.exists(),
    })
    .into_response()
}

async fn project_info(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }

    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());

    let project_name = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut entries: Vec<String> = match std::fs::read_dir(&root) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    entries.truncate(80);

    let git_repo = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);

    let git_branch = if git_repo {
        Command::new("git")
            .current_dir(&root)
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    };

    Json(ProjectInfoResponse {
        ok: true,
        service: "pb-gateway",
        project_root: root.display().to_string(),
        project_name,
        git_repo,
        git_branch,
        top_level_entries: entries,
    })
    .into_response()
}

async fn security_status(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }

    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let confined = root.exists() && root.is_dir();
    let mut notes = Vec::new();
    if state.token.is_none() {
        notes.push("PB_GATEWAY_TOKEN is not set (token auth disabled).".to_string());
    }
    if state.signing_secret.is_none() {
        notes.push("PB_GATEWAY_SIGNING_SECRET is not set (request signing disabled).".to_string());
    }
    if !confined {
        notes.push("project_root does not exist or is not a directory.".to_string());
    }

    Json(SecurityStatusResponse {
        ok: true,
        service: "pb-gateway",
        token_protection_enabled: state.token.is_some(),
        signed_requests_enabled: state.signing_secret.is_some(),
        signed_requests_max_skew_ms: state.signing_max_skew_ms,
        project_root_confined: confined,
        notes,
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    pattern: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    literal: bool,
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[serde(default = "default_search_limit")]
    max_matches: usize,
}

fn default_true() -> bool {
    true
}

fn default_search_limit() -> usize {
    120
}

async fn search_code(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<SearchQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    if q.pattern.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "pattern must not be empty").into_response();
    }

    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let search_root = match resolve_under_root(&root, &q.path) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let (n, results) = match search_work(
        search_root.clone(),
        &q.pattern,
        q.literal,
        q.case_sensitive,
        q.max_matches,
    ) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    Json(SearchResponse {
        ok: true,
        service: "pb-gateway",
        search_root: search_root.display().to_string(),
        kind: "code",
        pattern: q.pattern,
        matches: n,
        results,
    })
    .into_response()
}

async fn search_docs(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<SearchQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    if q.pattern.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "pattern must not be empty").into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    // docs-only scope
    let docs_root = root.join("docs");
    let base = if docs_root.exists() {
        docs_root
    } else {
        root.clone()
    };
    let search_root = match resolve_under_root(&base, &q.path) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let (n, results) = match search_work(
        search_root.clone(),
        &q.pattern,
        q.literal,
        q.case_sensitive,
        q.max_matches,
    ) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };

    Json(SearchResponse {
        ok: true,
        service: "pb-gateway",
        search_root: search_root.display().to_string(),
        kind: "docs",
        pattern: q.pattern,
        matches: n,
        results,
    })
    .into_response()
}

async fn review_packet(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, &body) {
        return (code, msg).into_response();
    }

    let packet: ReviewPacket = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "packet must be valid JSON").into_response(),
    };

    if packet.schema_version != 1 {
        return (StatusCode::BAD_REQUEST, "schema_version must be 1").into_response();
    }
    if packet.project.trim().is_empty()
        || packet.task_id.trim().is_empty()
        || packet.summary.trim().is_empty()
        || packet.agent.id.trim().is_empty()
    {
        return (StatusCode::BAD_REQUEST, "missing required fields").into_response();
    }

    // Size guard (not a usage limit; just payload sanity).
    let max_bytes: usize = std::env::var("PB_REVIEW_PACKET_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256 * 1024);
    let json_line = match serde_json::to_string(&packet) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "packet must be valid JSON").into_response(),
    };
    if json_line.len() > max_bytes {
        return (StatusCode::PAYLOAD_TOO_LARGE, "review packet too large").into_response();
    }

    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    match append_review_packet_jsonl(&root, &json_line) {
        Ok(n) => Json(ReviewPacketAck {
            ok: true,
            service: "pb-gateway",
            stored: true,
            bytes_appended: n,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to store review packet: {e}"),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct ReviewPacketsQuery {
    #[serde(default = "default_review_list_limit")]
    limit: usize,
}

fn default_review_list_limit() -> usize {
    30
}

#[derive(Serialize)]
struct ReviewPacketsListResponse {
    ok: bool,
    service: &'static str,
    count: usize,
    packets_jsonl: Vec<String>,
}

async fn review_packets(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<ReviewPacketsQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    match read_review_packets_tail(&root, q.limit) {
        Ok(lines) => Json(ReviewPacketsListResponse {
            ok: true,
            service: "pb-gateway",
            count: lines.len(),
            packets_jsonl: lines,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to read review packets: {e}"),
        )
            .into_response(),
    }
}

async fn exec_local(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(req): Json<ExecRequest>,
) -> axum::response::Response {
    if !(state.trust_local_network && is_local_network_ip(peer.ip())) {
        return (
            StatusCode::FORBIDDEN,
            "exec is only allowed for local trusted network",
        )
            .into_response();
    }
    if req.command.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "command is required").into_response();
    }
    let timeout_ms = req.timeout_ms.unwrap_or(20_000).clamp(200, 120_000);

    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-lc")
        .arg(req.command.clone())
        .current_dir(state.project_root.clone());

    let result =
        tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), cmd.output()).await;
    match result {
        Ok(Ok(o)) => Json(ExecResponse {
            ok: true,
            service: "pb-gateway",
            command: req.command,
            exit_code: o.status.code(),
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
            timed_out: false,
        })
        .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("exec failed: {e}"),
        )
            .into_response(),
        Err(_) => Json(ExecResponse {
            ok: false,
            service: "pb-gateway",
            command: req.command,
            exit_code: None,
            stdout: String::new(),
            stderr: "timed out".to_string(),
            timed_out: true,
        })
        .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct PathQuery {
    #[serde(default)]
    path: String,
}

#[derive(Debug, Deserialize)]
struct ReadFileQuery {
    path: String,
    #[serde(default)]
    start_line: Option<usize>,
    #[serde(default)]
    end_line: Option<usize>,
}

fn read_file_slice(
    path: &std::path::Path,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<(String, bool), String> {
    let max_bytes: u64 = 512 * 1024;
    let max_chars: usize = 20_000;
    let bytes = fs::read(path).map_err(|e| format!("read file: {e}"))?;
    let clipped_disk = (bytes.len() as u64) > max_bytes;
    let text = if clipped_disk {
        String::from_utf8_lossy(&bytes[..max_bytes as usize]).to_string()
    } else {
        String::from_utf8_lossy(&bytes).to_string()
    };
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let n = lines.len();
    let mut out = if start_line.is_some() || end_line.is_some() {
        let s = start_line.unwrap_or(1).max(1);
        let e = end_line.unwrap_or(n).max(1).min(n);
        if s > n {
            String::new()
        } else {
            lines[s - 1..e].join("")
        }
    } else {
        text
    };
    let clipped_chars = out.chars().count() > max_chars;
    if clipped_chars {
        out = out.chars().take(max_chars).collect();
    }
    Ok((out, clipped_disk || clipped_chars))
}

async fn list_files(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<PathQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let p = match resolve_under_root(&root, &q.path) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let mut entries: Vec<String> = match fs::read_dir(&p) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect(),
        Err(e) => return (StatusCode::BAD_REQUEST, format!("read dir: {e}")).into_response(),
    };
    entries.sort();
    entries.truncate(300);
    Json(ListFilesResponse {
        ok: true,
        service: "pb-gateway",
        root: root.display().to_string(),
        path: p.display().to_string(),
        entries,
    })
    .into_response()
}

async fn read_file(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<ReadFileQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    if q.path.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "path is required").into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let p = match resolve_under_root(&root, &q.path) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    let (text, truncated) = match read_file_slice(&p, q.start_line, q.end_line) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    Json(ReadFileResponse {
        ok: true,
        service: "pb-gateway",
        path: p.display().to_string(),
        text,
        truncated,
    })
    .into_response()
}

async fn git_status(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<PathQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    let p = match resolve_under_root(&root, &q.path) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, e).into_response(),
    };
    match Command::new("git")
        .current_dir(&p)
        .args(["status", "--short", "--branch"])
        .output()
    {
        Ok(o) => Json(GitStatusResponse {
            ok: true,
            service: "pb-gateway",
            repo: p.display().to_string(),
            exit_code: o.status.code(),
            stdout: String::from_utf8_lossy(&o.stdout).to_string(),
            stderr: String::from_utf8_lossy(&o.stderr).to_string(),
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("git status failed: {e}"),
        )
            .into_response(),
    }
}

async fn agent_send(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Json(req): Json<AgentSendRequest>,
) -> axum::response::Response {
    let empty = b"";
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, empty) {
        return (code, msg).into_response();
    }
    let from = sanitize_agent_name(&req.from_agent);
    if from.is_empty() || req.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "from_agent and message are required",
        )
            .into_response();
    }
    let msg = RemoteAgentMessage {
        id: format!("{}-{}", now_ms(), std::process::id()),
        ts: now_ms(),
        from_agent: from,
        to_agent: req
            .to_agent
            .map(|s| sanitize_agent_name(&s))
            .filter(|s| !s.is_empty()),
        topic: req
            .topic
            .map(|s| s.trim().chars().take(120).collect())
            .filter(|s: &String| !s.is_empty()),
        kind: req
            .kind
            .unwrap_or_else(|| "note".to_string())
            .chars()
            .take(24)
            .collect(),
        body: req.message.chars().take(24_000).collect(),
    };
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    match append_remote_agent_message(&root, &msg) {
        Ok(_) => {
            Json(serde_json::json!({"ok": true, "service":"pb-gateway", "message_id": msg.id}))
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("agent_send failed: {e}"),
        )
            .into_response(),
    }
}

async fn agent_recent(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<AgentRecentQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    match read_remote_agent_messages(&root) {
        Ok(all) => {
            let limit = q.limit.clamp(1, 200);
            let start = all.len().saturating_sub(limit);
            let slice = &all[start..];
            Json(serde_json::json!({"ok": true, "service":"pb-gateway", "count": slice.len(), "messages": slice})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("agent_recent failed: {e}"),
        )
            .into_response(),
    }
}

async fn agent_thread(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<AgentThreadQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = authorize_request(&state, peer.ip(), &headers, &method, &uri, b"") {
        return (code, msg).into_response();
    }
    if q.topic.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "topic is required").into_response();
    }
    let topic = q.topic.trim().to_string();
    let root = state
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| state.project_root.clone());
    match read_remote_agent_messages(&root) {
        Ok(all) => {
            let mut matched: Vec<RemoteAgentMessage> = all
                .into_iter()
                .filter(|m| m.topic.as_deref() == Some(topic.as_str()))
                .collect();
            let limit = q.limit.clamp(1, 200);
            if matched.len() > limit {
                matched = matched[matched.len() - limit..].to_vec();
            }
            Json(serde_json::json!({"ok": true, "service":"pb-gateway", "topic": topic, "count": matched.len(), "messages": matched})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("agent_thread failed: {e}"),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = std::env::var("PB_GATEWAY_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let token = std::env::var("PB_GATEWAY_TOKEN")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let signing_secret = std::env::var("PB_GATEWAY_SIGNING_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let signing_max_skew_ms: u64 = std::env::var("PB_GATEWAY_SIGNING_MAX_SKEW_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120_000);
    let trust_local_network = std::env::var("PB_GATEWAY_TRUST_LOCAL_NETWORK")
        .ok()
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let state = Arc::new(AppState {
        token,
        signing_secret,
        signing_max_skew_ms,
        nonce_seen: Mutex::new(HashMap::new()),
        trust_local_network,
        project_root: project_root(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/security-status", get(security_status))
        .route("/project-info", get(project_info))
        .route("/list-files", get(list_files))
        .route("/read-file", get(read_file))
        .route("/git-status", get(git_status))
        .route("/search-code", get(search_code))
        .route("/search-docs", get(search_docs))
        .route("/agent-send", post(agent_send))
        .route("/agent-recent", get(agent_recent))
        .route("/agent-thread", get(agent_thread))
        .route("/exec", post(exec_local))
        .route("/review-packet", post(review_packet))
        .route("/review-packets", get(review_packets))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
