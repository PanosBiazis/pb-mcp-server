//! Append-only JSONL message bus under `<project>/.mint-mcp/agent-bus.jsonl`.
//! Uses `agent-bus.lock` with flock so multiple processes (separate MCP / agent sessions) can coordinate.

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Directory name for the MCP bus storage under the project root
pub const BUS_DIR: &str = ".mint-mcp";
const BUS_FILE: &str = "agent-bus.jsonl";
const LOCK_FILE: &str = "agent-bus.lock";

/// Max stored body length per bus message (truncated on send).
pub const MAX_AGENT_MESSAGE_BODY_CHARS: usize = 24_000;

/// Max distinct DM recipients for `agent_fan_out` (parallel delivery in one lock).
pub const MAX_FAN_OUT_RECIPIENTS: usize = 32;

/// Max agent ids for `agent_multi_inbox` (single read, partitioned in memory).
pub const MAX_MULTI_INBOX_AGENTS: usize = 16;

/// Max ordered posts in `agent_send_pipeline` (serial protocol under one lock).
pub const MAX_PIPELINE_STEPS: usize = 24;
const MAX_BODY_CHARS: usize = MAX_AGENT_MESSAGE_BODY_CHARS;
const MAX_NAME_CHARS: usize = 48;
const MAX_TOPIC_CHARS: usize = 120;
const MAX_JSONL_BYTES: u64 = 8 * 1024 * 1024;
const KEEP_LINES_ON_ROTATE: usize = 5_000;

/// Above this size, `agent_recent` without `since_ts_ms` loads from the end of the file only.
const BUS_TAIL_READ_THRESHOLD: u64 = 512 * 1024;
const BUS_TAIL_INITIAL_CHUNK: usize = 512 * 1024;
const BUS_TAIL_MAX_CHUNK: usize = 4 * 1024 * 1024;

fn read_file_tail_bytes(path: &Path, tail_bytes: usize) -> Result<String, String> {
    let mut f = fs::File::open(path).map_err(|e| e.to_string())?;
    let len = f.metadata().map_err(|e| e.to_string())?.len() as usize;
    if len == 0 {
        return Ok(String::new());
    }
    let take = tail_bytes.min(len);
    let start = len - take;
    f.seek(SeekFrom::Start(start as u64))
        .map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; take];
    f.read_exact(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Parse messages from the tail of a large bus file until at least `min_messages` are found
/// (or the whole file is loaded). Used for fast “recent” views.
fn load_messages_tail(path: &Path, min_messages: usize) -> Result<Vec<AgentMessage>, String> {
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let len = meta.len() as usize;
    if len <= BUS_TAIL_READ_THRESHOLD as usize {
        let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
        return Ok(parse_all_lines(&s));
    }

    let min_messages = min_messages.max(1);
    let mut chunk = BUS_TAIL_INITIAL_CHUNK.min(len);
    loop {
        let mut s = read_file_tail_bytes(path, chunk)?;
        if chunk < len {
            if let Some(pos) = s.find('\n') {
                s = s[pos + 1..].to_string();
            }
        }
        let msgs = parse_all_lines(&s);
        if msgs.len() >= min_messages || chunk >= len || chunk >= BUS_TAIL_MAX_CHUNK {
            if msgs.len() < min_messages && chunk >= BUS_TAIL_MAX_CHUNK && chunk < len {
                let full = fs::read_to_string(path).map_err(|e| e.to_string())?;
                return Ok(parse_all_lines(&full));
            }
            return Ok(msgs);
        }
        chunk = (chunk.saturating_mul(2)).min(len).min(BUS_TAIL_MAX_CHUNK);
    }
}

/// Message structure for agent-to-agent communication via the bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unique message identifier
    pub id: String,
    /// Unix time in milliseconds when the message was created
    pub ts: u64,
    /// Sender agent identifier
    pub from_agent: String,
    /// Optional recipient agent for direct messages
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_agent: Option<String>,
    /// Optional topic for pub/sub messaging
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Message kind (e.g., note, question, suggestion, handoff)
    #[serde(default)]
    pub kind: String,
    /// Message body content
    pub body: String,
}

fn log_path(root: &Path) -> PathBuf {
    root.join(BUS_DIR).join(BUS_FILE)
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(BUS_DIR).join(LOCK_FILE)
}

fn ensure_bus_dir(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join(BUS_DIR)).map_err(|e| e.to_string())
}

/// True if `from_agent` will be accepted for `agent_send` after sanitization.
pub fn from_agent_acceptable(s: &str) -> bool {
    !sanitize_agent_name(s).is_empty()
}

/// Canonical agent id after the same sanitization used when appending to the bus (for comparisons).
pub fn normalized_agent_id(s: &str) -> String {
    sanitize_agent_name(s)
}

fn sanitize_agent_name(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() {
        return String::new();
    }
    t.chars()
        .take(MAX_NAME_CHARS)
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn sanitize_topic(s: &str) -> String {
    s.trim().chars().take(MAX_TOPIC_CHARS).collect()
}

fn sanitize_kind(s: &str) -> String {
    let k: String = s
        .trim()
        .chars()
        .take(24)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if k.is_empty() {
        "note".to_string()
    } else {
        k
    }
}

fn truncate_body(body: &str) -> String {
    let (s, _) = crate::ops::truncate_chars(body, MAX_BODY_CHARS);
    s
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn new_id() -> String {
    format!("{}-{}", now_ms(), std::process::id())
}

fn parse_all_lines(content: &str) -> Vec<AgentMessage> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<AgentMessage>(l).ok())
        .collect()
}

fn maybe_rotate_log(path: &Path) -> Result<(), String> {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if meta.len() < MAX_JSONL_BYTES {
        return Ok(());
    }
    let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= KEEP_LINES_ON_ROTATE {
        return Ok(());
    }
    let start = lines.len() - KEEP_LINES_ON_ROTATE;
    let new_content = lines[start..].join("\n") + "\n";
    fs::write(path, new_content).map_err(|e| e.to_string())
}

fn with_shared_lock<T>(root: &Path, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ensure_bus_dir(root)?;
    let lp = lock_path(root);
    let lockf = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&lp)
        .map_err(|e| format!("bus lock open: {e}"))?;
    lockf
        .lock_shared()
        .map_err(|e| format!("bus lock shared: {e}"))?;
    let out = f();
    let _ = lockf.unlock();
    out
}

fn with_exclusive_lock<T>(root: &Path, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ensure_bus_dir(root)?;
    let lp = lock_path(root);
    let lockf = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&lp)
        .map_err(|e| format!("bus lock open: {e}"))?;
    lockf
        .lock_exclusive()
        .map_err(|e| format!("bus lock exclusive: {e}"))?;
    let out = f();
    let _ = lockf.unlock();
    out
}

/// Append a message; returns message id.
pub fn agent_send(
    project_root: PathBuf,
    from_agent: String,
    body: String,
    to_agent: Option<String>,
    topic: Option<String>,
    kind: Option<String>,
) -> Result<String, String> {
    let from_agent = sanitize_agent_name(&from_agent);
    if from_agent.is_empty() {
        return Err("from_agent must be a non-empty identifier (letters, digits, _.-)".into());
    }
    let body = truncate_body(&body);
    if body.trim().is_empty() {
        return Err("body must not be empty".into());
    }

    let to_agent = to_agent
        .as_ref()
        .map(|s| sanitize_agent_name(s))
        .filter(|s| !s.is_empty());

    let topic = topic
        .as_ref()
        .map(|s| sanitize_topic(s))
        .filter(|s| !s.is_empty());

    let kind = sanitize_kind(kind.as_deref().unwrap_or("note"));

    let msg = AgentMessage {
        id: new_id(),
        ts: now_ms(),
        from_agent,
        to_agent,
        topic,
        kind,
        body,
    };

    let id = msg.id.clone();
    let line = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
    let root = project_root;

    with_exclusive_lock(&root, || {
        let path = log_path(&root);
        maybe_rotate_log(&path)?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("bus log open: {e}"))?;
        writeln!(&mut f, "{line}").map_err(|e| format!("bus append: {e}"))?;
        Ok(id.clone())
    })
}

/// One identical DM to many recipients under a **single** exclusive lock (parallel fan-out on the bus).
pub fn agent_fan_out(
    project_root: PathBuf,
    from_agent: String,
    body: String,
    to_agents: Vec<String>,
    topic: Option<String>,
    kind: Option<String>,
) -> Result<String, String> {
    let from_agent = sanitize_agent_name(&from_agent);
    if from_agent.is_empty() {
        return Err("from_agent must be a non-empty identifier (letters, digits, _.-)".into());
    }
    let body = truncate_body(&body);
    if body.trim().is_empty() {
        return Err("message body must not be empty".into());
    }

    let topic = topic
        .as_ref()
        .map(|s| sanitize_topic(s))
        .filter(|s| !s.is_empty());

    let kind_base = sanitize_kind(kind.as_deref().unwrap_or("note"));

    let mut seen: HashSet<String> = HashSet::new();
    let mut recipients: Vec<String> = Vec::new();
    for raw in to_agents {
        let t = sanitize_agent_name(&raw);
        if t.is_empty() || t == from_agent {
            continue;
        }
        if seen.insert(t.clone()) {
            recipients.push(t);
        }
    }
    if recipients.is_empty() {
        return Err(
            "to_agents must contain at least one valid recipient distinct from from_agent".into(),
        );
    }
    if recipients.len() > MAX_FAN_OUT_RECIPIENTS {
        return Err(format!(
            "too many recipients (max {MAX_FAN_OUT_RECIPIENTS})"
        ));
    }

    let root = project_root;
    with_exclusive_lock(&root, || {
        let path = log_path(&root);
        maybe_rotate_log(&path)?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("bus log open: {e}"))?;

        let mut lines_out = String::new();
        lines_out.push_str("AGENT_FAN_OUT\n");
        lines_out.push_str(&format!("recipients: {}\n\n", recipients.len()));

        for to in recipients {
            let kind = kind_base.clone();
            let msg = AgentMessage {
                id: new_id(),
                ts: now_ms(),
                from_agent: from_agent.clone(),
                to_agent: Some(to.clone()),
                topic: topic.clone(),
                kind,
                body: body.clone(),
            };
            let id = msg.id.clone();
            let line = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            writeln!(&mut f, "{line}").map_err(|e| format!("bus append: {e}"))?;
            lines_out.push_str(&format!("message_id={id} to_agent={to}\n"));
        }
        Ok(lines_out)
    })
}

/// Ordered multi-post from one sender under a **single** exclusive lock (serial bus protocol).
/// A single step in a pipeline send: (message, to_agent, topic, kind).
pub type PipelineStep = (String, Option<String>, Option<String>, Option<String>);

#[allow(clippy::too_many_arguments)]
pub fn agent_send_pipeline(
    project_root: PathBuf,
    from_agent: String,
    steps: Vec<PipelineStep>,
) -> Result<String, String> {
    let from_agent = sanitize_agent_name(&from_agent);
    if from_agent.is_empty() {
        return Err("from_agent must be a non-empty identifier (letters, digits, _.-)".into());
    }
    if steps.is_empty() {
        return Err("steps must not be empty".into());
    }
    if steps.len() > MAX_PIPELINE_STEPS {
        return Err(format!("too many steps (max {MAX_PIPELINE_STEPS})"));
    }

    let root = project_root;
    with_exclusive_lock(&root, || {
        let path = log_path(&root);
        maybe_rotate_log(&path)?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("bus log open: {e}"))?;

        let mut out = String::new();
        out.push_str("AGENT_SEND_PIPELINE\n");
        out.push_str(&format!("steps_written: {}\n\n", steps.len()));

        for (i, (body_raw, to_raw, topic_raw, kind_raw)) in steps.into_iter().enumerate() {
            let body = truncate_body(&body_raw);
            if body.trim().is_empty() {
                return Err(format!("step {}: message must not be empty", i + 1));
            }
            let to_agent = to_raw
                .as_ref()
                .map(|s| sanitize_agent_name(s))
                .filter(|s| !s.is_empty());
            let topic = topic_raw
                .as_ref()
                .map(|s| sanitize_topic(s))
                .filter(|s| !s.is_empty());
            let kind = sanitize_kind(kind_raw.as_deref().unwrap_or("note"));

            let msg = AgentMessage {
                id: new_id(),
                ts: now_ms(),
                from_agent: from_agent.clone(),
                to_agent,
                topic,
                kind,
                body,
            };
            let id = msg.id.clone();
            let line = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
            writeln!(&mut f, "{line}").map_err(|e| format!("bus append: {e}"))?;
            out.push_str(&format!("step {} message_id={id}\n", i + 1));
        }
        Ok(out)
    })
}

/// Many inboxes from **one** bus read (parallel fan-in for the caller).
pub fn agent_multi_inbox(
    project_root: PathBuf,
    for_agents: Vec<String>,
    limit: usize,
    skip_own_broadcasts: bool,
    since_ts_ms: Option<u64>,
) -> Result<String, String> {
    let limit = limit.clamp(1, 200);
    let mut agents: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for a in for_agents {
        let x = sanitize_agent_name(&a);
        if x.is_empty() {
            continue;
        }
        if seen.insert(x.clone()) {
            agents.push(x);
        }
    }
    if agents.is_empty() {
        return Err("for_agents must contain at least one valid id".into());
    }
    if agents.len() > MAX_MULTI_INBOX_AGENTS {
        return Err(format!(
            "too many for_agents (max {MAX_MULTI_INBOX_AGENTS})"
        ));
    }

    let path = log_path(&project_root);
    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_MULTI_INBOX\nNo bus file yet. Expected: {}\n",
                path.display()
            ));
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);

        let mut out = String::new();
        out.push_str("AGENT_MULTI_INBOX\n");
        out.push_str("mode: single_bus_read parallel sections\n\n");

        for agent in agents {
            out.push_str(&format!("===== inbox for_agent={agent} =====\n"));
            let picked =
                collect_inbox_messages(&all, &agent, limit, skip_own_broadcasts, since_ts_ms);
            if picked.is_empty() {
                out.push_str("No messages for this agent (with current filters).\n\n");
            } else {
                let block: String = picked
                    .iter()
                    .map(|m| format_message(m))
                    .collect::<Vec<_>>()
                    .join("\n");
                out.push_str(&block);
                out.push_str("\n\n");
            }
        }

        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

/// One read: roster-style counts, hot topics, and recent tail — fewer round-trips for orchestrators.
pub fn agent_coordination_snapshot(
    project_root: PathBuf,
    topics_limit: usize,
    recent_limit: usize,
) -> Result<String, String> {
    let topics_limit = topics_limit.clamp(1, 100);
    let recent_limit = recent_limit.clamp(1, 80);
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_COORDINATION_SNAPSHOT\nbus_path: {}\nexists: false\n",
                rel.display()
            ));
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        if all.is_empty() {
            return Ok(format!(
                "AGENT_COORDINATION_SNAPSHOT\nbus_path: {}\nmessages_total: 0\n",
                rel.display()
            ));
        }

        let mut by_from: HashMap<String, usize> = HashMap::new();
        let mut topic_agg: HashMap<String, (usize, u64)> = HashMap::new();
        for m in &all {
            *by_from.entry(m.from_agent.clone()).or_insert(0) += 1;
            if let Some(t) = m.topic.as_deref().filter(|x| !x.is_empty()) {
                let e = topic_agg.entry(t.to_string()).or_insert((0, 0));
                e.0 += 1;
                e.1 = e.1.max(m.ts);
            }
        }

        let mut topics: Vec<(String, usize, u64)> = topic_agg
            .into_iter()
            .map(|(t, (c, ts))| (t, c, ts))
            .collect();
        topics.sort_by(|a, b| b.2.cmp(&a.2));

        let mut senders: Vec<(String, usize)> = by_from.into_iter().collect();
        senders.sort_by(|a, b| b.1.cmp(&a.1));

        let start = all.len().saturating_sub(recent_limit);
        let recent_slice = &all[start..];

        let mut out = String::new();
        out.push_str("AGENT_COORDINATION_SNAPSHOT\n");
        out.push_str(&format!(
            "bus_path: {}\nmessages_total: {}\n",
            rel.display(),
            all.len()
        ));
        out.push_str("\n--- top senders (posts) ---\n");
        for (name, n) in senders.iter().take(12) {
            out.push_str(&format!("  {name}: {n}\n"));
        }
        out.push_str("\n--- hot topics (by last activity) ---\n");
        for (t, c, ts) in topics.iter().take(topics_limit) {
            out.push_str(&format!("  {t} | msgs={c} | last_ts_ms={ts}\n"));
        }
        out.push_str(&format!(
            "\n--- recent tail (last {}) ---\n",
            recent_slice.len()
        ));
        for m in recent_slice.iter() {
            out.push_str(&format_message_compact(m, 400));
            out.push('\n');
        }
        out.push_str("\nUse agent_thread / agent_multi_inbox / agent_fan_out for detail.\n");
        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

fn collect_inbox_messages<'a>(
    all: &'a [AgentMessage],
    for_agent: &str,
    limit: usize,
    skip_own_broadcasts: bool,
    since_ts_ms: Option<u64>,
) -> Vec<&'a AgentMessage> {
    let mut picked: Vec<&AgentMessage> = Vec::new();
    for m in all.iter().rev() {
        if picked.len() >= limit {
            break;
        }
        if let Some(t0) = since_ts_ms {
            if m.ts <= t0 {
                continue;
            }
        }
        let include = match &m.to_agent {
            None => !(skip_own_broadcasts && m.from_agent == for_agent),
            Some(to) => to == for_agent,
        };
        if include {
            picked.push(m);
        }
    }
    picked.reverse();
    picked
}

fn format_message(m: &AgentMessage) -> String {
    let to = m
        .to_agent
        .as_deref()
        .map(|t| format!("@{t}"))
        .unwrap_or_else(|| "(broadcast)".to_string());
    let topic = m
        .topic
        .as_deref()
        .map(|t| format!(" topic={t}"))
        .unwrap_or_default();
    format!(
        "[{}] ts_ms={} {} -> {}{} kind={}\n{}\n---",
        m.id, m.ts, m.from_agent, to, topic, m.kind, m.body
    )
}

/// Inbox: direct messages to `for_agent`, plus broadcasts (optionally excluding own).
/// If `since_ts_ms` is set, only messages with `ts` strictly greater than that value are included (incremental polling).
pub fn agent_inbox(
    project_root: PathBuf,
    for_agent: String,
    limit: usize,
    skip_own_broadcasts: bool,
    since_ts_ms: Option<u64>,
) -> Result<String, String> {
    let for_agent = sanitize_agent_name(&for_agent);
    if for_agent.is_empty() {
        return Err("for_agent must be non-empty".into());
    }
    let limit = limit.clamp(1, 200);
    let path = log_path(&project_root);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "No bus file yet. Agents should call `agent_send` first. Expected: {}",
                path.display()
            ));
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        let picked =
            collect_inbox_messages(&all, &for_agent, limit, skip_own_broadcasts, since_ts_ms);
        if picked.is_empty() {
            return Ok(
                "No messages for this agent (try `agent_recent` or a different for_agent).".into(),
            );
        }
        let block: String = picked
            .iter()
            .map(|m| format_message(m))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(block)
    })
}

/// All messages in a topic thread, oldest first among the tail window.
/// If `since_ts_ms` is set, only messages with `ts` >= that value are kept (then windowed by `limit`).
pub fn agent_thread(
    project_root: PathBuf,
    topic: String,
    limit: usize,
    since_ts_ms: Option<u64>,
) -> Result<String, String> {
    let topic = sanitize_topic(&topic);
    if topic.is_empty() {
        return Err("topic must be non-empty".into());
    }
    let limit = limit.clamp(1, 200);
    let path = log_path(&project_root);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok("No bus file yet.".into());
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        let mut picked: Vec<&AgentMessage> = all
            .iter()
            .filter(|m| m.topic.as_deref() == Some(topic.as_str()))
            .filter(|m| {
                if let Some(t0) = since_ts_ms {
                    m.ts >= t0
                } else {
                    true
                }
            })
            .collect();
        if picked.len() > limit {
            let start = picked.len() - limit;
            picked = picked[start..].to_vec();
        }
        if picked.is_empty() {
            return Ok(format!("No messages with topic \"{topic}\"."));
        }
        let block: String = picked
            .iter()
            .map(|m| format_message(m))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(block)
    })
}

/// Recent messages for shared awareness (newest at bottom among returned slice).
/// With `since_ts_ms`, only messages with `ts` strictly greater than that value are considered, then the last `limit` of those.
pub fn agent_recent(
    project_root: PathBuf,
    limit: usize,
    since_ts_ms: Option<u64>,
) -> Result<String, String> {
    let limit = limit.clamp(1, 150);
    let path = log_path(&project_root);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok("No bus file yet.".into());
        }
        let all = if since_ts_ms.is_none() {
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            if meta.len() > BUS_TAIL_READ_THRESHOLD {
                load_messages_tail(&path, limit)?
            } else {
                let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                parse_all_lines(&s)
            }
        } else {
            let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            parse_all_lines(&s)
        };
        let filtered: Vec<&AgentMessage> = all
            .iter()
            .filter(|m| since_ts_ms.map(|t0| m.ts > t0).unwrap_or(true))
            .collect();
        let start = filtered.len().saturating_sub(limit);
        let slice = &filtered[start..];
        if slice.is_empty() {
            return Ok(match since_ts_ms {
                Some(_) => "No messages newer than since_ts_ms.".into(),
                None => "Log is empty.".into(),
            });
        }
        let block: String = slice
            .iter()
            .map(|m| format_message(m))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(block)
    })
}

/// Get the current status of the agent bus for a project
///
/// Returns information about the bus file including:
/// - Whether it exists
/// - File size
/// - Number of messages
/// - Last modified time
pub fn agent_bus_status(project_root: PathBuf) -> Result<String, String> {
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);
    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "bus_path: {}\nexists: false\nhint: add `.mint-mcp/` to .gitignore if you do not want the bus in version control",
                rel.display()
            ));
        }
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        let n = all.len();
        let mut uniq_from = std::collections::HashSet::new();
        let mut uniq_topic = std::collections::HashSet::new();
        let mut uniq_dm_to = std::collections::HashSet::new();
        let mut first_ts: Option<u64> = None;
        let mut last_ts: Option<u64> = None;
        for m in &all {
            uniq_from.insert(m.from_agent.clone());
            if let Some(ref t) = m.topic {
                if !t.is_empty() {
                    uniq_topic.insert(t.clone());
                }
            }
            if let Some(ref t) = m.to_agent {
                uniq_dm_to.insert(t.clone());
            }
            first_ts.get_or_insert(m.ts);
            last_ts = Some(m.ts);
        }
        Ok(format!(
            "bus_path: {}\nexists: true\napprox_messages: {}\nsize_bytes: {}\nlock: {}\n\
             first_ts_ms: {}\nlast_ts_ms: {}\nunique_from_agents: {}\nunique_topics: {}\nunique_dm_recipients: {}\n\
             hint: use `agent_send` / `agent_inbox` / `agent_thread` / `agent_recent` / `agent_list_topics` / `agent_roster` for coordination; each agent should use a stable `from_agent` name.",
            rel.display(),
            n,
            meta.len(),
            Path::new(BUS_DIR).join(LOCK_FILE).display(),
            first_ts.map(|t| t.to_string()).unwrap_or_else(|| "n/a".into()),
            last_ts.map(|t| t.to_string()).unwrap_or_else(|| "n/a".into()),
            uniq_from.len(),
            uniq_topic.len(),
            uniq_dm_to.len(),
        ))
    })
}

fn format_message_compact(m: &AgentMessage, max_body_chars: usize) -> String {
    let to = m
        .to_agent
        .as_deref()
        .map(|t| format!("@{t}"))
        .unwrap_or_else(|| "(broadcast)".to_string());
    let topic = m
        .topic
        .as_deref()
        .map(|t| format!(" topic={t}"))
        .unwrap_or_default();

    let (body, _) = crate::ops::truncate_chars(&m.body, max_body_chars);
    format!(
        "[{}] {} -> {}{} kind={}\n{}\n---",
        m.id, m.from_agent, to, topic, m.kind, body
    )
}

/// Compact/summarized view of the agent bus for prompt budgeting.
///
/// Returns aggregate counts and (optionally) a short tail of recent messages.
pub fn agent_bus_compact(
    project_root: PathBuf,
    limit: usize,
    show_bodies: bool,
    max_body_chars: usize,
) -> Result<String, String> {
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);

    let limit = limit.clamp(1, 500);
    let max_body_chars = max_body_chars.clamp(20, 5000);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_BUS_COMPACT\nbus_path: {}\nexists: false\nhint: call `agent_send` first.\n",
                rel.display()
            ));
        }

        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        if all.is_empty() {
            return Ok(format!(
                "AGENT_BUS_COMPACT\nbus_path: {}\nmessages_total: 0\n",
                rel.display()
            ));
        }

        let slice_start = all.len().saturating_sub(limit);
        let slice = &all[slice_start..];

        let mut by_from: HashMap<String, usize> = HashMap::new();
        let mut by_kind: HashMap<String, usize> = HashMap::new();
        for m in slice.iter() {
            *by_from.entry(m.from_agent.clone()).or_insert(0) += 1;
            *by_kind.entry(m.kind.clone()).or_insert(0) += 1;
        }

        let mut top_from: Vec<(String, usize)> = by_from.into_iter().collect();
        top_from.sort_by(|a, b| b.1.cmp(&a.1));
        let mut top_kind: Vec<(String, usize)> = by_kind.into_iter().collect();
        top_kind.sort_by(|a, b| b.1.cmp(&a.1));

        let mut out = String::new();
        out.push_str("AGENT_BUS_COMPACT\n");
        out.push_str(&format!(
            "bus_path: {}\nmessages_total: {}\nrecent_window: last {} messages\n\n",
            rel.display(),
            all.len(),
            slice.len()
        ));

        out.push_str("by_from_agent_top:\n");
        for (name, count) in top_from.into_iter().take(10) {
            out.push_str(&format!("  {name}: {count}\n"));
        }

        out.push_str("\nby_kind_top:\n");
        for (name, count) in top_kind.into_iter().take(10) {
            out.push_str(&format!("  {name}: {count}\n"));
        }

        if show_bodies {
            out.push_str("\nrecent_messages (compact):\n");
            for m in slice.iter() {
                out.push_str(&format_message_compact(m, max_body_chars));
                out.push('\n');
            }
        } else {
            out.push_str("\nrecent_messages: (body elided; set show_bodies=true)\n");
        }

        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

fn format_message_search(m: &AgentMessage, show_bodies: bool, max_body_chars: usize) -> String {
    let to = m
        .to_agent
        .as_deref()
        .map(|t| format!("@{t}"))
        .unwrap_or_else(|| "(broadcast)".to_string());
    let topic = m
        .topic
        .as_deref()
        .map(|t| format!(" topic={t}"))
        .unwrap_or_default();

    if show_bodies {
        let (body, _) = crate::ops::truncate_chars(&m.body, max_body_chars);
        format!(
            "[{}] {} -> {}{} kind={}\n{}\n---",
            m.id, m.from_agent, to, topic, m.kind, body
        )
    } else {
        format!(
            "[{}] {} -> {}{} kind={}\n(body elided)\n---",
            m.id, m.from_agent, to, topic, m.kind
        )
    }
}

/// Search the durable bus for messages whose body contains `needle`, with optional filters.
#[allow(clippy::too_many_arguments)]
pub fn agent_bus_search(
    project_root: PathBuf,
    needle: String,
    case_sensitive: bool,
    limit: usize,
    show_bodies: bool,
    max_body_chars: usize,
    from_agent: Option<String>,
    to_agent: Option<String>,
    kind: Option<String>,
    topic: Option<String>,
) -> Result<String, String> {
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);

    let needle = needle.trim().to_string();
    if needle.is_empty() {
        return Err("needle must not be empty".into());
    }

    let limit = limit.clamp(1, 200);
    let max_body_chars = max_body_chars.clamp(20, 5000);

    let needle_cmp = if case_sensitive {
        needle.clone()
    } else {
        needle.to_lowercase()
    };

    let from_agent = from_agent
        .map(|s| sanitize_agent_name(&s))
        .filter(|s| !s.is_empty());
    let to_agent = to_agent
        .map(|s| sanitize_agent_name(&s))
        .filter(|s| !s.is_empty());
    let kind = kind.and_then(|s| {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(sanitize_kind(t))
        }
    });
    let topic = topic.map(|s| sanitize_topic(&s)).filter(|s| !s.is_empty());

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_BUS_SEARCH\nbus_path: {}\nexists: false\nhint: call `agent_send` first.\n",
                rel.display()
            ));
        }

        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);

        if all.is_empty() {
            return Ok(format!(
                "AGENT_BUS_SEARCH\nbus_path: {}\nmessages_total: 0\n",
                rel.display()
            ));
        }

        let mut picked: Vec<&AgentMessage> = Vec::new();
        for m in all.iter().rev() {
            if picked.len() >= limit {
                break;
            }

            if let Some(ref fa) = from_agent {
                if &m.from_agent != fa {
                    continue;
                }
            }
            if let Some(ref ta) = to_agent {
                if m.to_agent.as_deref() != Some(ta.as_str()) {
                    continue;
                }
            }
            if let Some(ref k) = kind {
                if &m.kind != k {
                    continue;
                }
            }
            if let Some(ref t) = topic {
                if m.topic.as_deref() != Some(t.as_str()) {
                    continue;
                }
            }

            let hay = if case_sensitive {
                m.body.clone()
            } else {
                m.body.to_lowercase()
            };
            if !hay.contains(&needle_cmp) {
                continue;
            }

            picked.push(m);
        }

        picked.reverse();

        if picked.is_empty() {
            return Ok(format!(
                "AGENT_BUS_SEARCH\nbus_path: {}\nneedle: {}\nmatch_count: 0\n",
                rel.display(),
                needle
            ));
        }

        let mut out = String::new();
        out.push_str("AGENT_BUS_SEARCH\n");
        out.push_str(&format!(
            "bus_path: {}\nneedle: {}\nmatch_count: {}\nfilters: from_agent={:?} to_agent={:?} kind={:?} topic={:?}\n\n",
            rel.display(),
            needle,
            picked.len(),
            from_agent,
            to_agent,
            kind,
            topic
        ));

        for m in picked {
            out.push_str(&format_message_search(m, show_bodies, max_body_chars));
            out.push('\n');
        }

        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

/// Collect replies in a topic after `since_ts_ms`, excluding messages from `exclude_from_agent`.
///
/// Intended for an "ask_agents" style orchestrator: one agent posts a question with a topic, then
/// polls for other agents' responses on the same topic. Returns the **most recent** `limit`
/// matching messages (not the oldest chunk of the bus).
pub fn agent_collect_topic_replies(
    project_root: PathBuf,
    topic: String,
    since_ts_ms: u64,
    exclude_from_agent: String,
    limit: usize,
) -> Result<Vec<AgentMessage>, String> {
    let topic = sanitize_topic(&topic);
    if topic.is_empty() {
        return Err("topic must be non-empty".into());
    }
    let exclude_from_agent = sanitize_agent_name(&exclude_from_agent);
    if exclude_from_agent.is_empty() {
        return Err("exclude_from_agent must be non-empty".into());
    }
    let limit = limit.clamp(1, 200);

    let path = log_path(&project_root);
    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);

        let mut out: Vec<AgentMessage> = Vec::new();
        for m in all.into_iter() {
            if m.ts < since_ts_ms {
                continue;
            }
            if m.from_agent == exclude_from_agent {
                continue;
            }
            if m.topic.as_deref() != Some(topic.as_str()) {
                continue;
            }
            out.push(m);
        }
        if out.len() > limit {
            out = out[out.len() - limit..].to_vec();
        }
        Ok(out)
    })
}

/// Topics that appear on the bus, with activity stats (newest-first).
pub fn agent_list_topics(project_root: PathBuf, limit: usize) -> Result<String, String> {
    let limit = limit.clamp(1, 200);
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_LIST_TOPICS\nbus_path: {}\nexists: false\nhint: call `agent_send` with a topic first.\n",
                rel.display()
            ));
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        if all.is_empty() {
            return Ok(format!(
                "AGENT_LIST_TOPICS\nbus_path: {}\nmessages_total: 0\n",
                rel.display()
            ));
        }

        #[derive(Default)]
        struct Agg {
            count: usize,
            last_ts: u64,
            first_ts: u64,
        }
        let mut m: HashMap<String, Agg> = HashMap::new();
        for msg in &all {
            let Some(t) = msg.topic.as_deref().filter(|x| !x.is_empty()) else {
                continue;
            };
            let e = m.entry(t.to_string()).or_default();
            e.count += 1;
            e.last_ts = e.last_ts.max(msg.ts);
            if e.count == 1 {
                e.first_ts = msg.ts;
            } else {
                e.first_ts = e.first_ts.min(msg.ts);
            }
        }

        if m.is_empty() {
            return Ok(format!(
                "AGENT_LIST_TOPICS\nbus_path: {}\nmessages_total: {}\ntopics: 0\nhint: no messages carry a `topic` field yet.\n",
                rel.display(),
                all.len()
            ));
        }

        let mut rows: Vec<(String, Agg)> = m.into_iter().collect();
        rows.sort_by(|a, b| b.1.last_ts.cmp(&a.1.last_ts));

        let mut out = String::new();
        out.push_str("AGENT_LIST_TOPICS\n");
        out.push_str(&format!(
            "bus_path: {}\nmessages_total: {}\ntopics_listed: {}\n\n",
            rel.display(),
            all.len(),
            rows.len().min(limit)
        ));
        out.push_str("topic | msgs | first_ts_ms | last_ts_ms\n");
        for (topic, agg) in rows.into_iter().take(limit) {
            out.push_str(&format!(
                "{topic} | {} | {} | {}\n",
                agg.count, agg.first_ts, agg.last_ts
            ));
        }
        out.push_str("\nUse `agent_thread(topic)` for full thread text; reuse exact topic strings when replying.\n");
        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

/// Who has posted and who receives DMs — helps agents discover collaborators without reading the whole log.
pub fn agent_roster(project_root: PathBuf) -> Result<String, String> {
    let path = log_path(&project_root);
    let rel = Path::new(BUS_DIR).join(BUS_FILE);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok(format!(
                "AGENT_ROSTER\nbus_path: {}\nexists: false\n",
                rel.display()
            ));
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        if all.is_empty() {
            return Ok(format!(
                "AGENT_ROSTER\nbus_path: {}\nmessages_total: 0\n",
                rel.display()
            ));
        }

        #[derive(Default)]
        struct SendAgg {
            posts: usize,
            last_ts: u64,
            kinds: HashMap<String, usize>,
        }
        let mut senders: HashMap<String, SendAgg> = HashMap::new();
        let mut dm_to: HashMap<String, (usize, u64)> = HashMap::new();

        for msg in &all {
            let se = senders.entry(msg.from_agent.clone()).or_default();
            se.posts += 1;
            se.last_ts = se.last_ts.max(msg.ts);
            *se.kinds.entry(msg.kind.clone()).or_insert(0) += 1;
            if let Some(to) = msg.to_agent.as_deref() {
                let e = dm_to.entry(to.to_string()).or_insert((0, 0));
                e.0 += 1;
                e.1 = e.1.max(msg.ts);
            }
        }

        let mut send_rows: Vec<_> = senders.into_iter().collect();
        send_rows.sort_by(|a, b| b.1.last_ts.cmp(&a.1.last_ts));

        let mut out = String::new();
        out.push_str("AGENT_ROSTER\n");
        out.push_str(&format!(
            "bus_path: {}\nmessages_total: {}\nunique_senders: {}\nunique_dm_recipients: {}\n\n",
            rel.display(),
            all.len(),
            send_rows.len(),
            dm_to.len()
        ));

        out.push_str("from_agent | posts | last_ts_ms | kind breakdown (top)\n");
        for (name, agg) in send_rows.iter().take(40) {
            let mut k: Vec<_> = agg.kinds.iter().collect();
            k.sort_by(|a, b| b.1.cmp(a.1));
            let kinds: String = k
                .into_iter()
                .take(4)
                .map(|(kind, n)| format!("{kind}:{n}"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "{name} | {} | {} | {}\n",
                agg.posts, agg.last_ts, kinds
            ));
        }
        if send_rows.len() > 40 {
            out.push_str(&format!("... and {} more senders\n", send_rows.len() - 40));
        }

        if !dm_to.is_empty() {
            let mut dm_rows: Vec<_> = dm_to.into_iter().collect();
            dm_rows.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
            out.push_str("\nDM recipients (by recent inbound DM):\n");
            for (to, (n, ts)) in dm_rows.into_iter().take(30) {
                out.push_str(&format!("  @{to} | dms: {n} | last_ts_ms: {ts}\n"));
            }
        }

        out.push_str("\nPick a stable `from_agent` id per session; use `agent_send` + shared `topic` for threads.\n");
        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}

/// Fetch one message by exact `id` (as returned by `agent_send` or shown in bus listings).
pub fn agent_get_message(project_root: PathBuf, message_id: String) -> Result<String, String> {
    let id = message_id.trim();
    if id.is_empty() {
        return Err("message_id must not be empty".into());
    }
    if id.len() > 120 {
        return Err("message_id too long".into());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "message_id must contain only ASCII letters, digits, hyphen, underscore".into(),
        );
    }

    let path = log_path(&project_root);
    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Err("No bus file yet.".into());
        }
        let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
        let len = meta.len() as usize;

        if len > BUS_TAIL_READ_THRESHOLD as usize {
            let mut chunk = BUS_TAIL_INITIAL_CHUNK.min(len);
            loop {
                let mut s = read_file_tail_bytes(&path, chunk)?;
                if chunk < len {
                    if let Some(pos) = s.find('\n') {
                        s = s[pos + 1..].to_string();
                    }
                }
                for line in s.lines().rev() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(m) = serde_json::from_str::<AgentMessage>(line) {
                        if m.id == id {
                            return Ok(format!("AGENT_GET_MESSAGE\n{}", format_message(&m)));
                        }
                    }
                }
                if chunk >= len || chunk >= BUS_TAIL_MAX_CHUNK {
                    break;
                }
                chunk = (chunk.saturating_mul(2)).min(len).min(BUS_TAIL_MAX_CHUNK);
            }
            let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            for m in parse_all_lines(&s).iter().rev() {
                if m.id == id {
                    return Ok(format!("AGENT_GET_MESSAGE\n{}", format_message(m)));
                }
            }
        } else {
            let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            for m in parse_all_lines(&s).iter().rev() {
                if m.id == id {
                    return Ok(format!("AGENT_GET_MESSAGE\n{}", format_message(m)));
                }
            }
        }
        Err(format!("No message with id \"{id}\"."))
    })
}

/// Non-empty inbox-style payload for wait loops (same rules as `agent_inbox`).
pub fn agent_inbox_non_empty(
    project_root: PathBuf,
    for_agent: String,
    limit: usize,
    skip_own_broadcasts: bool,
    since_ts_ms: Option<u64>,
) -> Result<bool, String> {
    let s = agent_inbox(
        project_root,
        for_agent,
        limit,
        skip_own_broadcasts,
        since_ts_ms,
    )?;
    Ok(!s.starts_with("No messages for this agent"))
}

/// True when there is at least one topic message from another agent after `since_ts_ms`.
pub fn agent_topic_has_reply_from_others(
    project_root: PathBuf,
    topic: String,
    since_ts_ms: u64,
    exclude_from_agent: String,
) -> Result<bool, String> {
    let msgs =
        agent_collect_topic_replies(project_root, topic, since_ts_ms, exclude_from_agent, 1)?;
    Ok(!msgs.is_empty())
}
