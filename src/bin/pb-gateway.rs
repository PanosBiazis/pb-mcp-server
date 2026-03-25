use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, net::SocketAddr, path::PathBuf, sync::Arc};
use subtle::ConstantTimeEq;

#[derive(Clone)]
struct AppState {
    token: Option<String>,
    project_root: PathBuf,
}

fn project_root() -> PathBuf {
    std::env::var("MINT_PROJECT_ROOT")
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
    if got.as_bytes().ct_eq(expected.as_bytes()).into() {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "invalid token"))
    }
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
struct ReviewPacketAck {
    ok: bool,
    service: &'static str,
    stored: bool,
    bytes_appended: usize,
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

async fn status(
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err((code, msg)) = require_token(&state, &headers) {
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

async fn review_packet(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(packet): Json<ReviewPacket>,
) -> axum::response::Response {
    if let Err((code, msg)) = require_token(&state, &headers) {
        return (code, msg).into_response();
    }

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
    headers: HeaderMap,
    axum::extract::Query(q): axum::extract::Query<ReviewPacketsQuery>,
) -> axum::response::Response {
    if let Err((code, msg)) = require_token(&state, &headers) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = std::env::var("PB_GATEWAY_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let token = std::env::var("PB_GATEWAY_TOKEN").ok().filter(|s| !s.trim().is_empty());

    let state = Arc::new(AppState {
        token,
        project_root: project_root(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/status", get(status))
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

