//! Append-only JSONL message bus under `<project>/.mint-mcp/agent-bus.jsonl`.
//! Uses `agent-bus.lock` with flock so multiple processes (separate MCP / agent sessions) can coordinate.

use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const BUS_DIR: &str = ".mint-mcp";
const BUS_FILE: &str = "agent-bus.jsonl";
const LOCK_FILE: &str = "agent-bus.lock";

const MAX_BODY_CHARS: usize = 24_000;
const MAX_NAME_CHARS: usize = 48;
const MAX_TOPIC_CHARS: usize = 120;
const MAX_JSONL_BYTES: u64 = 8 * 1024 * 1024;
const KEEP_LINES_ON_ROTATE: usize = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    /// Unix time in milliseconds.
    pub ts: u64,
    pub from_agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// e.g. note, question, suggestion, handoff
    #[serde(default)]
    pub kind: String,
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
        .read(true)
        .write(true)
        .open(&lp)
        .map_err(|e| format!("bus lock open: {e}"))?;
    lockf.lock_shared().map_err(|e| format!("bus lock shared: {e}"))?;
    let out = f();
    let _ = lockf.unlock();
    out
}

fn with_exclusive_lock<T>(root: &Path, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ensure_bus_dir(root)?;
    let lp = lock_path(root);
    let lockf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lp)
        .map_err(|e| format!("bus lock open: {e}"))?;
    lockf.lock_exclusive().map_err(|e| format!("bus lock exclusive: {e}"))?;
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
        "[{}] {} -> {}{} kind={}\n{}\n---",
        m.id, m.from_agent, to, topic, m.kind, m.body
    )
}

/// Inbox: direct messages to `for_agent`, plus broadcasts (optionally excluding own).
pub fn agent_inbox(
    project_root: PathBuf,
    for_agent: String,
    limit: usize,
    skip_own_broadcasts: bool,
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
        let mut picked: Vec<&AgentMessage> = Vec::new();
        for m in all.iter().rev() {
            if picked.len() >= limit {
                break;
            }
            let include = match &m.to_agent {
                None => {
                    if skip_own_broadcasts && m.from_agent == for_agent {
                        false
                    } else {
                        true
                    }
                }
                Some(to) => to == &for_agent,
            };
            if include {
                picked.push(m);
            }
        }
        picked.reverse();
        if picked.is_empty() {
            return Ok("No messages for this agent (try `agent_recent` or a different for_agent).".into());
        }
        let block: String = picked.iter().map(|m| format_message(m)).collect::<Vec<_>>().join("\n");
        Ok(block)
    })
}

/// All messages in a topic thread, oldest first among the tail window.
pub fn agent_thread(project_root: PathBuf, topic: String, limit: usize) -> Result<String, String> {
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
            .collect();
        if picked.len() > limit {
            let start = picked.len() - limit;
            picked = picked[start..].to_vec();
        }
        if picked.is_empty() {
            return Ok(format!("No messages with topic \"{topic}\"."));
        }
        let block: String = picked.iter().map(|m| format_message(m)).collect::<Vec<_>>().join("\n");
        Ok(block)
    })
}

/// Recent messages for shared awareness (newest at bottom among returned slice).
pub fn agent_recent(project_root: PathBuf, limit: usize) -> Result<String, String> {
    let limit = limit.clamp(1, 150);
    let path = log_path(&project_root);

    with_shared_lock(&project_root, || {
        if !path.exists() {
            return Ok("No bus file yet.".into());
        }
        let s = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let all = parse_all_lines(&s);
        let start = all.len().saturating_sub(limit);
        let slice = &all[start..];
        if slice.is_empty() {
            return Ok("Log is empty.".into());
        }
        let block: String = slice.iter().map(|m| format_message(m)).collect::<Vec<_>>().join("\n");
        Ok(block)
    })
}

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
        let n = s.lines().filter(|l| !l.trim().is_empty()).count();
        Ok(format!(
            "bus_path: {}\nexists: true\napprox_messages: {}\nsize_bytes: {}\nlock: {}\n\
             hint: use `agent_send` / `agent_inbox` / `agent_thread` / `agent_recent` for multi-agent coordination; each agent should use a stable `from_agent` name.",
            rel.display(),
            n,
            meta.len(),
            Path::new(BUS_DIR).join(LOCK_FILE).display()
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

fn format_message_search(
    m: &AgentMessage,
    show_bodies: bool,
    max_body_chars: usize,
) -> String {
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
    let topic = topic
        .map(|s| sanitize_topic(&s))
        .filter(|s| !s.is_empty());

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
            out.push_str(&format_message_search(&m, show_bodies, max_body_chars));
            out.push('\n');
        }

        let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
        Ok(out)
    })
}
