//! # Log Analysis, History, and Chart Visualization Tools
//!
//! Provides `log_*_work`, `history_*_work`, and `chart_*_work` functions
//! for log parsing, audit-history querying, and SVG/DOT chart generation.
//!
//! All functions take `args: serde_json::Value` and return `serde_json::Value`.

use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
use std::path::Path;

const MAX_HISTOGRAM_BINS: usize = 10_000;
const MAX_LOG_RESULT_LIMIT: usize = 10_000;
const MAX_HISTORY_COUNT: usize = 1_000;
const MAX_TIMESERIES_WINDOW: usize = 10_000;

// ---------------------------------------------------------------------------
// Arg helpers
// ---------------------------------------------------------------------------

fn args_str(args: &Value, key: &str, default: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn args_u64(args: &Value, key: &str, default: u64) -> u64 {
    args.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

fn args_usize_clamped(args: &Value, key: &str, default: usize, min: usize, max: usize) -> usize {
    let raw = args
        .get(key)
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(default);
    raw.clamp(min, max)
}

fn args_i64(args: &Value, key: &str, default: i64) -> i64 {
    args.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

fn args_bool(args: &Value, key: &str, default: bool) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

fn args_f64(args: &Value, key: &str, default: f64) -> f64 {
    args.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

fn read_file_lines(path: &str) -> Result<Vec<String>, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("File not found: {path}"));
    }
    let file = fs::File::open(p).map_err(|e| format!("Error opening {path}: {e}"))?;
    let reader = std::io::BufReader::new(file);
    let mut lines = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Error reading {path}: {e}"))?;
        lines.push(line);
    }
    Ok(lines)
}

/// Detect log level from a line of text.
fn detect_level(line: &str) -> &'static str {
    let upper = line.to_uppercase();
    if upper.contains("ERROR") || upper.contains("FATAL") || upper.contains("CRIT") {
        "ERROR"
    } else if upper.contains("WARN") || upper.contains("WARNING") {
        "WARN"
    } else if upper.contains("INFO") || upper.contains("NOTICE") {
        "INFO"
    } else if upper.contains("DEBUG") || upper.contains("TRACE") {
        "DEBUG"
    } else {
        "UNKNOWN"
    }
}

/// Normalize variable parts in an error line for clustering.
fn normalize_for_cluster(line: &str) -> String {
    // Replace hex patterns like 0x1a2b3c
    let s = replace_regex_all(line, r"0x[0-9a-fA-F]+", "*");
    // Replace IPv4-like patterns (e.g. 10.0.0.1) before general digit replacement
    // so that connection errors to different IPs cluster together
    let s = replace_regex_all(&s, r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", "*");
    // Replace numbers (sequences of digits)
    let s = replace_regex_all(&s, r"\b\d{2,}\b", "*");
    // Replace file paths (heuristic: /something/... or C:\...)
    replace_regex_all(
        &s,
        r#"(?:/[a-zA-Z0-9_.\-]+){2,}|(?:[A-Za-z]:\\[a-zA-Z0-9_.\-\\]+)"#,
        "*",
    )
}

fn replace_regex_all(text: &str, pattern: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace_all(text, replacement).to_string())
        .unwrap_or_else(|_| text.to_string())
}

// ---------------------------------------------------------------------------
// 1. log_tail_work
// ---------------------------------------------------------------------------

/// Read last N lines of a log file, optionally filtering by log level.
pub fn log_tail_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let n = args_usize_clamped(&args, "n", 100, 1, MAX_LOG_RESULT_LIMIT);
    let filter_level = args_str(&args, "filter_level", "");

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let total_lines = lines.len();
    let filter_upper = filter_level.to_uppercase();
    let filter = if ["DEBUG", "INFO", "WARN", "ERROR"].contains(&filter_upper.as_str()) {
        Some(filter_upper)
    } else {
        None
    };

    let mut result = Vec::new();
    for (i, line) in lines.iter().enumerate().rev() {
        if result.len() >= n {
            break;
        }
        let level = detect_level(line);
        if let Some(ref f) = filter {
            if level != f.as_str() && level != "UNKNOWN" {
                continue;
            }
            // If filter is set, skip lines that don't match at all
            if level == "UNKNOWN" {
                continue;
            }
        }
        result.push(json!({
            "line_number": i + 1,
            "content": line,
            "level": level,
        }));
    }
    result.reverse();

    json!({
        "lines": result,
        "total_lines": total_lines,
    })
}

// ---------------------------------------------------------------------------
// 2. log_search_work
// ---------------------------------------------------------------------------

/// Grep a log file with a regex pattern.
pub fn log_search_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let pattern = args_str(&args, "pattern", "");
    let max_results = args_usize_clamped(&args, "max_results", 50, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }
    if pattern.is_empty() {
        return json!({"error": "pattern is required"});
    }

    let re = match Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => return json!({"error": format!("Invalid regex: {e}")}),
    };

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut matches = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if matches.len() >= max_results {
            break;
        }
        if let Some(m) = re.find(line) {
            matches.push(json!({
                "line_number": i + 1,
                "content": line,
                "matched_text": m.as_str(),
            }));
        }
    }

    json!({
        "matches": matches,
        "total_matches": matches.len(),
    })
}

// ---------------------------------------------------------------------------
// 3. log_parse_json_work
// ---------------------------------------------------------------------------

/// Parse JSON Lines log file with optional field-value filters.
pub fn log_parse_json_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let limit = args_usize_clamped(&args, "limit", 100, 1, MAX_LOG_RESULT_LIMIT);
    let filter = args.get("filter").and_then(|v| v.as_object()).cloned();

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut entries = Vec::new();
    for line in &lines {
        if entries.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Apply filters if provided
        if let Some(ref filt) = filter {
            let entry_obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };
            let mut pass = true;
            for (key, expected) in filt {
                if let Some(actual) = entry_obj.get(key) {
                    if actual != expected {
                        pass = false;
                        break;
                    }
                } else {
                    pass = false;
                    break;
                }
            }
            if !pass {
                continue;
            }
        }

        entries.push(entry);
    }

    json!({
        "entries": entries,
        "total": entries.len(),
    })
}

// ---------------------------------------------------------------------------
// 4. log_parse_nginx_work
// ---------------------------------------------------------------------------

/// Parse Nginx access log into structured JSON.
pub fn log_parse_nginx_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let limit = args_usize_clamped(&args, "limit", 100, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let re = match Regex::new(r#"^(\S+) \S+ \S+ \[([^\]]+)\] "(\S+) (\S+) \S+" (\d+) (\d+)"#) {
        Ok(r) => r,
        Err(e) => return json!({"error": format!("Regex error: {e}")}),
    };

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut entries = Vec::new();
    for line in &lines {
        if entries.len() >= limit {
            break;
        }
        if let Some(caps) = re.captures(line) {
            entries.push(json!({
                "ip": caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                "timestamp": caps.get(2).map(|m| m.as_str()).unwrap_or(""),
                "method": caps.get(3).map(|m| m.as_str()).unwrap_or(""),
                "path": caps.get(4).map(|m| m.as_str()).unwrap_or(""),
                "status": caps.get(5).map(|m| m.as_str()).unwrap_or(""),
                "size": caps.get(6).map(|m| m.as_str()).unwrap_or(""),
            }));
        }
    }

    json!({ "entries": entries })
}

// ---------------------------------------------------------------------------
// 5. log_parse_syslog_work
// ---------------------------------------------------------------------------

/// Parse syslog format into structured JSON.
pub fn log_parse_syslog_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let limit = args_usize_clamped(&args, "limit", 100, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    // Standard syslog: "Mmm DD HH:MM:SS hostname facility: message"
    let re = match Regex::new(
        r"^(\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\S+?)(?:\[\d+\])?(?::|\s)(.*)",
    ) {
        Ok(r) => r,
        Err(e) => return json!({"error": format!("Regex error: {e}")}),
    };

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut entries = Vec::new();
    for line in &lines {
        if entries.len() >= limit {
            break;
        }
        if let Some(caps) = re.captures(line) {
            entries.push(json!({
                "timestamp": caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                "host": caps.get(2).map(|m| m.as_str()).unwrap_or(""),
                "facility": caps.get(3).map(|m| m.as_str()).unwrap_or(""),
                "message": caps.get(4).map(|m| m.as_str().trim()).unwrap_or(""),
            }));
        }
    }

    json!({ "entries": entries })
}

// ---------------------------------------------------------------------------
// 6. log_level_summary_work
// ---------------------------------------------------------------------------

/// Count occurrences of each log level in a file.
pub fn log_level_summary_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut counts: HashMap<&str, usize> = HashMap::new();
    counts.insert("DEBUG", 0);
    counts.insert("INFO", 0);
    counts.insert("WARN", 0);
    counts.insert("ERROR", 0);

    let mut total = 0usize;
    for line in &lines {
        let level = detect_level(line);
        if level != "UNKNOWN" {
            *counts.entry(level).or_insert(0) += 1;
            total += 1;
        }
    }

    json!({
        "DEBUG": counts.get("DEBUG").unwrap_or(&0),
        "INFO": counts.get("INFO").unwrap_or(&0),
        "WARN": counts.get("WARN").unwrap_or(&0),
        "ERROR": counts.get("ERROR").unwrap_or(&0),
        "total": total,
    })
}

// ---------------------------------------------------------------------------
// 7. log_error_cluster_work
// ---------------------------------------------------------------------------

/// Group repeated errors by normalized pattern.
pub fn log_error_cluster_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let max_clusters = args_usize_clamped(&args, "max_clusters", 10, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    // Collect error lines
    let mut clusters: HashMap<String, (usize, String, usize)> = HashMap::new();
    // pattern -> (count, example_line, first_line_number)

    for (i, line) in lines.iter().enumerate() {
        let level = detect_level(line);
        if level != "ERROR" {
            continue;
        }
        let pattern = normalize_for_cluster(line);
        let entry = clusters.entry(pattern).or_insert((0, line.clone(), i + 1));
        entry.0 += 1;
    }

    let mut cluster_list: Vec<(String, usize, String, usize)> = clusters
        .into_iter()
        .map(|(pattern, (count, example, first))| (pattern, count, example, first))
        .collect();
    cluster_list.sort_by(|a, b| b.1.cmp(&a.1));
    cluster_list.truncate(max_clusters);

    let clusters_json: Vec<Value> = cluster_list
        .into_iter()
        .map(|(pattern, count, example_line, first_line_number)| {
            json!({
                "pattern": pattern,
                "count": count,
                "example_line": example_line,
                "first_line_number": first_line_number,
            })
        })
        .collect();

    json!({ "clusters": clusters_json })
}

// ---------------------------------------------------------------------------
// 8. log_time_range_filter_work
// ---------------------------------------------------------------------------

/// Extract log lines within a timestamp range.
pub fn log_time_range_filter_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let since = args_str(&args, "since", "");
    let until = args_str(&args, "until", "");
    let limit = args_usize_clamped(&args, "limit", 1000, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    // Regex to find ISO-ish timestamps in lines: e.g. 2024-01-15T10:30:00 or 2024-01-15 10:30:00
    let ts_re = Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}").ok();

    let mut entries = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if entries.len() >= limit {
            break;
        }
        let ts = match ts_re.as_ref().and_then(|re| re.find(line)) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };

        // Simple string comparison works for ISO-format timestamps
        if !since.is_empty() && ts.as_str() < since.as_str() {
            continue;
        }
        if !until.is_empty() && ts.as_str() > until.as_str() {
            continue;
        }

        entries.push(json!({
            "line_number": i + 1,
            "content": line,
            "timestamp": ts,
        }));
    }

    json!({
        "entries": entries,
        "count": entries.len(),
    })
}

// ---------------------------------------------------------------------------
// 9. log_export_work
// ---------------------------------------------------------------------------

/// Export filtered log entries to JSON, CSV, or Markdown table.
pub fn log_export_work(args: Value) -> Value {
    let entries = match args.get("entries").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "entries array is required"}),
    };
    let format = args_str(&args, "format", "json");
    let output_path = args_str(&args, "output_path", "");

    if output_path.is_empty() {
        return json!({"error": "output_path is required"});
    }

    let entry_count = entries.len();
    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string()),
        "csv" => {
            let mut wtr = String::new();
            // Collect all unique keys preserving order
            let mut keys: Vec<String> = Vec::new();
            let mut key_set: std::collections::HashSet<String> = std::collections::HashSet::new();
            for entry in &entries {
                if let Some(obj) = entry.as_object() {
                    for k in obj.keys() {
                        if key_set.insert(k.clone()) {
                            keys.push(k.clone());
                        }
                    }
                }
            }
            // Header
            wtr.push_str(&keys.join(","));
            wtr.push('\n');
            // Rows
            for entry in &entries {
                if let Some(obj) = entry.as_object() {
                    let vals: Vec<String> = keys
                        .iter()
                        .map(|k| {
                            let v = obj.get(k).map(|v| v.to_string()).unwrap_or_default();
                            // Escape commas and quotes for CSV
                            if v.contains(',') || v.contains('"') || v.contains('\n') {
                                format!("\"{}\"", v.replace('"', "\"\""))
                            } else {
                                v
                            }
                        })
                        .collect();
                    wtr.push_str(&vals.join(","));
                    wtr.push('\n');
                }
            }
            wtr
        }
        "markdown" => {
            let mut md = String::new();
            let mut keys: Vec<String> = Vec::new();
            let mut key_set: std::collections::HashSet<String> = std::collections::HashSet::new();
            for entry in &entries {
                if let Some(obj) = entry.as_object() {
                    for k in obj.keys() {
                        if key_set.insert(k.clone()) {
                            keys.push(k.clone());
                        }
                    }
                }
            }
            // Header row
            md.push_str("| ");
            md.push_str(&keys.join(" | "));
            md.push_str(" |\n");
            // Separator
            md.push_str("| ");
            md.push_str(&vec!["---"; keys.len()].join(" | "));
            md.push_str(" |\n");
            // Data rows
            for entry in &entries {
                md.push_str("| ");
                if let Some(obj) = entry.as_object() {
                    let vals: Vec<String> = keys
                        .iter()
                        .map(|k| {
                            obj.get(k)
                                .map(|v| v.to_string())
                                .unwrap_or_default()
                                .replace('|', "\\|")
                                .replace('\n', " ")
                        })
                        .collect();
                    md.push_str(&vals.join(" | "));
                }
                md.push_str(" |\n");
            }
            md
        }
        _ => return json!({"error": "format must be json, csv, or markdown"}),
    };

    if let Err(e) = fs::write(&output_path, content.as_bytes()) {
        return json!({"error": format!("Error writing {}: {e}", output_path)});
    }

    json!({
        "path": output_path,
        "entry_count": entry_count,
        "format": format,
    })
}

// ---------------------------------------------------------------------------
// 10. log_audit_query_work
// ---------------------------------------------------------------------------

/// Query the pb-mcp-server audit log.
pub fn log_audit_query_work(args: Value) -> Value {
    let agent_id = args_str(&args, "agent_id", "");
    let tool_name = args_str(&args, "tool_name", "");
    let since_ms = if args.get("since_ms").is_some() {
        Some(args_u64(&args, "since_ms", 0))
    } else {
        None
    };
    let limit = args_usize_clamped(&args, "limit", 100, 1, MAX_LOG_RESULT_LIMIT);

    // Try to find the project root from env or common paths
    let project_root = std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .unwrap_or_else(|_| ".".to_string());

    let audit_path = format!("{}/.mint-mcp/audit/audit-log.jsonl", project_root);
    let lines = match read_file_lines(&audit_path) {
        Ok(l) => l,
        Err(e) => {
            return json!({"error": format!("Failed to read audit log: {e}"), "audit_path": audit_path})
        }
    };

    let mut entries_json: Vec<Value> = Vec::new();
    for line in &lines {
        if entries_json.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Apply filters
        if !agent_id.is_empty()
            && entry.get("agent_id").and_then(|v| v.as_str()).unwrap_or("") != agent_id
        {
            continue;
        }
        if !tool_name.is_empty()
            && entry
                .get("tool_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                != tool_name
        {
            continue;
        }
        if let Some(since) = since_ms {
            if entry
                .get("timestamp_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                < since
            {
                continue;
            }
        }

        entries_json.push(entry);
    }

    json!({
        "entries": entries_json,
        "total": entries_json.len(),
    })
}

// ---------------------------------------------------------------------------
// 11. history_tool_calls_work
// ---------------------------------------------------------------------------

/// Get recent tool calls from the audit log.
pub fn history_tool_calls_work(args: Value) -> Value {
    let agent_id = args_str(&args, "agent_id", "");
    let limit = args_usize_clamped(&args, "limit", 50, 1, MAX_LOG_RESULT_LIMIT);

    let project_root = std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .unwrap_or_else(|_| ".".to_string());

    let audit_path = format!("{}/.mint-mcp/audit/audit-log.jsonl", project_root);
    let lines = match read_file_lines(&audit_path) {
        Ok(l) => l,
        Err(e) => {
            return json!({"error": format!("Failed to read audit log: {e}"), "audit_path": audit_path})
        }
    };

    let mut calls: Vec<Value> = Vec::new();
    for line in &lines {
        if calls.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if !agent_id.is_empty()
            && entry.get("agent_id").and_then(|v| v.as_str()).unwrap_or("") != agent_id
        {
            continue;
        }

        calls.push(json!({
            "timestamp": entry.get("timestamp_iso").and_then(|v| v.as_str()).unwrap_or(""),
            "agent_id": entry.get("agent_id").and_then(|v| v.as_str()).unwrap_or(""),
            "tool_name": entry.get("tool_name").and_then(|v| v.as_str()).unwrap_or(""),
            "status": entry.get("result_status").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }

    json!({
        "calls": calls
    })
}

// ---------------------------------------------------------------------------
// 12. history_bus_replay_work
// ---------------------------------------------------------------------------

/// Replay bus messages from JSONL file.
pub fn history_bus_replay_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let since = args_str(&args, "since", "");
    let agent = args_str(&args, "agent", "");
    let limit = args_usize_clamped(&args, "limit", 100, 1, MAX_LOG_RESULT_LIMIT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let lines = match read_file_lines(&path) {
        Ok(l) => l,
        Err(e) => return json!({"error": e}),
    };

    let mut messages = Vec::new();
    for line in &lines {
        if messages.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Filter by timestamp (if `since` is given)
        if !since.is_empty() {
            if let Some(ts) = entry.get("timestamp").and_then(|v| v.as_str()) {
                if ts < since.as_str() {
                    continue;
                }
            }
        }

        // Filter by agent (check both `from` and `to`)
        if !agent.is_empty() {
            let from = entry.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = entry.get("to").and_then(|v| v.as_str()).unwrap_or("");
            if from != agent && to != agent {
                continue;
            }
        }

        messages.push(json!({
            "from": entry.get("from").and_then(|v| v.as_str()).unwrap_or(""),
            "to": entry.get("to").and_then(|v| v.as_str()).unwrap_or(""),
            "content": entry.get("content").unwrap_or(&json!("")),
            "msg_type": entry.get("msg_type").and_then(|v| v.as_str()).unwrap_or(""),
            "timestamp": entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }

    json!({
        "messages": messages,
        "total": messages.len(),
    })
}

// ---------------------------------------------------------------------------
// 13. history_diff_timeline_work
// ---------------------------------------------------------------------------

/// Show how a file changed over N commits.
pub fn history_diff_timeline_work(args: Value) -> Value {
    let path = args_str(&args, "path", "");
    let count = args_usize_clamped(&args, "count", 5, 1, MAX_HISTORY_COUNT);

    if path.is_empty() {
        return json!({"error": "path is required"});
    }

    let project_root = std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .unwrap_or_else(|_| ".".to_string());

    // git log --oneline -n <count> -- <path>
    let log_output = match std::process::Command::new("git")
        .args([
            "log",
            "--oneline",
            "--no-color",
            "-n",
            &count.to_string(),
            "--",
            &path,
        ])
        .current_dir(&project_root)
        .output()
    {
        Ok(o) => o,
        Err(e) => return json!({"error": format!("git log failed: {e}")}),
    };

    let log_stdout = String::from_utf8_lossy(&log_output.stdout);
    let mut commits = Vec::new();

    for line in log_stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Parse "sha message" format
        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }
        let sha = parts[0].to_string();
        let message = parts[1].to_string();

        // git show --format="%ai" <sha> -- <path> to get the date
        let date_output = std::process::Command::new("git")
            .args(["show", "--no-color", "--format=%ai", "-s", &sha])
            .current_dir(&project_root)
            .output();

        let date = date_output
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // git show <sha>:<path> to get content length
        let show_output = std::process::Command::new("git")
            .args(["show", "--no-color", &format!("{}:{}", sha, path)])
            .current_dir(&project_root)
            .output();

        let content_length = show_output.ok().map(|o| o.stdout.len()).unwrap_or(0);

        commits.push(json!({
            "sha": sha,
            "message": message,
            "date": date,
            "content_length": content_length,
        }));
    }

    json!({ "commits": commits })
}

// ---------------------------------------------------------------------------
// 14. history_search_past_calls_work
// ---------------------------------------------------------------------------

/// Full-text search across audit log entries.
pub fn history_search_past_calls_work(args: Value) -> Value {
    let query = args_str(&args, "query", "");
    let limit = args_usize_clamped(&args, "limit", 20, 1, MAX_LOG_RESULT_LIMIT);

    if query.is_empty() {
        return json!({"error": "query is required"});
    }

    let project_root = std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .unwrap_or_else(|_| ".".to_string());

    let audit_path = format!("{}/.mint-mcp/audit/audit-log.jsonl", project_root);
    let lines = match read_file_lines(&audit_path) {
        Ok(l) => l,
        Err(e) => {
            return json!({"error": format!("Failed to read audit log: {e}"), "audit_path": audit_path})
        }
    };

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    for line in &lines {
        if results.len() >= limit {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Search in tool_name, agent_id, result_status, and metadata
        let agent = entry.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
        let tool = entry
            .get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let status = entry
            .get("result_status")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let meta_str = entry
            .get("metadata")
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_default();

        let searchable = format!("{} {} {} {}", agent, tool, status, meta_str);

        if searchable.to_lowercase().contains(&query_lower) {
            // Build match context snippet
            let match_context = if tool.to_lowercase().contains(&query_lower) {
                format!("tool_name: {}", tool)
            } else if agent.to_lowercase().contains(&query_lower) {
                format!("agent_id: {}", agent)
            } else {
                format!("metadata: {}", meta_str)
            };

            results.push(json!({
                "timestamp": entry.get("timestamp_iso").and_then(|v| v.as_str()).unwrap_or(""),
                "agent_id": agent,
                "tool_name": tool,
                "match_context": match_context,
            }));
        }
    }

    json!({ "results": results })
}

// ---------------------------------------------------------------------------
// SVG helpers
// ---------------------------------------------------------------------------

const SVG_W: u32 = 800;
const SVG_H: u32 = 400;
const MARGIN: u32 = 60;

fn svg_result(svg: String) -> Value {
    svg_result_with_dims(svg, SVG_W, SVG_H)
}

fn svg_result_with_dims(svg: String, w: u32, h: u32) -> Value {
    let mut result = serde_json::Map::new();
    result.insert("svg".to_string(), Value::String(svg));
    result.insert("width".to_string(), Value::Number(w.into()));
    result.insert("height".to_string(), Value::Number(h.into()));
    Value::Object(result)
}

fn svg_header(w: u32, h: u32, _title: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">"#
    )
}

fn svg_title(w: u32, title: &str) -> String {
    format!(
        r#"<text x="{}" y="30" text-anchor="middle" font-family="sans-serif" font-size="16" font-weight="bold">{}</text>"#,
        w / 2,
        html_escape(title)
    )
}

fn svg_axes(
    plot_x: u32,
    plot_y: u32,
    plot_w: u32,
    plot_h: u32,
    x_label: &str,
    y_label: &str,
) -> String {
    let mut s = String::new();
    let x_end = plot_x + plot_w;
    let y_end = plot_y + plot_h;
    // X axis
    s.push_str(&format!(
        "<line x1=\"{plot_x}\" y1=\"{y_end}\" x2=\"{x_end}\" y2=\"{y_end}\" stroke=\"#333\" stroke-width=\"1\"/>"
    ));
    // Y axis
    s.push_str(&format!(
        "<line x1=\"{plot_x}\" y1=\"{plot_y}\" x2=\"{plot_x}\" y2=\"{y_end}\" stroke=\"#333\" stroke-width=\"1\"/>"
    ));
    // X label
    let x_lbl_x = plot_x + plot_w / 2;
    let x_lbl_y = plot_y + plot_h + 45;
    s.push_str(&format!(
        "<text x=\"{x_lbl_x}\" y=\"{x_lbl_y}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"12\">{}</text>",
        html_escape(x_label)
    ));
    // Y label
    let y_lbl_pos = plot_y + plot_h / 2;
    s.push_str(&format!(
        "<text x=\"15\" y=\"{y_lbl_pos}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"12\" transform=\"rotate(-90,15,{y_lbl_pos})\">{}</text>",
        html_escape(y_label)
    ));
    s
}

fn svg_tick_labels_x(
    plot_x: u32,
    plot_y: u32,
    plot_w: u32,
    plot_h: u32,
    ticks: &[String],
) -> String {
    let mut s = String::new();
    if ticks.is_empty() {
        return s;
    }
    let step = if ticks.len() > 1 {
        plot_w as f64 / (ticks.len() - 1) as f64
    } else {
        0.0
    };
    for (i, label) in ticks.iter().enumerate() {
        let x = plot_x as f64 + i as f64 * step;
        let tick_y = plot_y + plot_h + 18;
        s.push_str(&format!(
            "<text x=\"{x:.0}\" y=\"{tick_y}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"10\">{}</text>",
            html_escape(label)
        ));
    }
    s
}

fn svg_tick_labels_y(
    plot_x: u32,
    plot_y: u32,
    plot_h: u32,
    min_val: f64,
    max_val: f64,
    num_ticks: usize,
) -> String {
    let mut s = String::new();
    for i in 0..=num_ticks {
        let frac = i as f64 / num_ticks as f64;
        let val = max_val - frac * (max_val - min_val);
        let y = plot_y as f64 + frac * plot_h as f64;
        let tx = plot_x - 8;
        let ty = y + 4.0;
        s.push_str(&format!(
            "<text x=\"{tx}\" y=\"{ty:.0}\" text-anchor=\"end\" font-family=\"sans-serif\" font-size=\"10\">{val:.1}</text>"
        ));
        // Grid line
        let grid_x2 = plot_x + (SVG_W - MARGIN);
        s.push_str(&format!(
            "<line x1=\"{plot_x}\" y1=\"{y:.0}\" x2=\"{grid_x2}\" y2=\"{y:.0}\" stroke=\"#eee\" stroke-width=\"0.5\"/>"
        ));
    }
    s
}

/// Simple HTML entity escaping for SVG text content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// 15. chart_line_work
// ---------------------------------------------------------------------------

/// Generate SVG line chart data.
pub fn chart_line_work(args: Value) -> Value {
    let series = match args.get("series").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "series array is required"}),
    };
    let title = args_str(&args, "title", "Line Chart");
    let x_label = args_str(&args, "x_label", "X");
    let y_label = args_str(&args, "y_label", "Y");

    let plot_x = MARGIN + 20;
    let plot_y = 45u32;
    let plot_w = SVG_W - MARGIN * 2 - 20;
    let plot_h = SVG_H - plot_y - 55;

    // Collect all x and y to determine ranges
    let mut all_x: Vec<f64> = Vec::new();
    let mut all_y: Vec<f64> = Vec::new();
    for s in &series {
        if let Some(vals) = s.get("values").and_then(|v| v.as_array()) {
            for pt in vals {
                if let Some(x) = pt.get("x").and_then(|v| v.as_f64()) {
                    all_x.push(x);
                }
                if let Some(y) = pt.get("y").and_then(|v| v.as_f64()) {
                    all_y.push(y);
                }
            }
        }
    }

    let x_min = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < 1e-10 {
        1.0
    } else {
        x_max - x_min
    };
    let y_range = if (y_max - y_min).abs() < 1e-10 {
        1.0
    } else {
        y_max - y_min
    };

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));
    svg.push_str(&svg_axes(
        plot_x, plot_y, plot_w, plot_h, &x_label, &y_label,
    ));
    svg.push_str(&svg_tick_labels_y(plot_x, plot_y, plot_h, y_min, y_max, 5));

    // X tick labels (use first series)
    if let Some(first) = series.first() {
        if let Some(vals) = first.get("values").and_then(|v| v.as_array()) {
            let ticks: Vec<String> = vals
                .iter()
                .filter_map(|pt| {
                    pt.get("x")
                        .and_then(|v| v.as_f64())
                        .map(|f| format!("{f:.0}"))
                })
                .collect();
            svg.push_str(&svg_tick_labels_x(plot_x, plot_y, plot_w, plot_h, &ticks));
        }
    }

    // Draw series
    let colors = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6",
    ];
    for (si, s) in series.iter().enumerate() {
        let label = s.get("label").and_then(|v| v.as_str()).unwrap_or("");
        let color = colors[si % colors.len()];
        if let Some(vals) = s.get("values").and_then(|v| v.as_array()) {
            let mut points_str = String::new();
            for pt in vals {
                let x = pt.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = pt.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let px = plot_x as f64 + ((x - x_min) / x_range) * plot_w as f64;
                let py = (plot_y + plot_h) as f64 - ((y - y_min) / y_range) * plot_h as f64;
                points_str.push_str(&format!("{px:.1},{py:.1} "));
            }
            let pts = points_str.trim();
            svg.push_str(&format!(
                "<polyline points=\"{pts}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\"/>"
            ));
            // Data points
            for pt in vals {
                let x = pt.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = pt.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let px = plot_x as f64 + ((x - x_min) / x_range) * plot_w as f64;
                let py = (plot_y + plot_h) as f64 - ((y - y_min) / y_range) * plot_h as f64;
                svg.push_str(&format!(
                    "<circle cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"3\" fill=\"{color}\"/>"
                ));
            }
            // Legend entry
            let lgd_x = plot_x + 10;
            let lgd_y = plot_y + 10 + (si as u32) * 18;
            svg.push_str(&format!(
                "<rect x=\"{lgd_x}\" y=\"{lgd_y}\" width=\"12\" height=\"12\" fill=\"{color}\"/>"
            ));
            let lgd_tx = plot_x + 26;
            let lgd_ty = plot_y + 20 + (si as u32) * 18;
            svg.push_str(&format!(
                "<text x=\"{lgd_tx}\" y=\"{lgd_ty}\" font-family=\"sans-serif\" font-size=\"11\">{}</text>",
                html_escape(label)
            ));
        }
    }

    svg.push_str("</svg>");

    svg_result(svg)
}

// ---------------------------------------------------------------------------
// 16. chart_bar_work
// ---------------------------------------------------------------------------

/// Generate SVG bar chart.
pub fn chart_bar_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "values array is required"}),
    };
    let title = args_str(&args, "title", "Bar Chart");

    if values.is_empty() {
        return json!({"svg": "", "error": "no values"});
    }

    let plot_x = MARGIN + 20;
    let plot_y = 45u32;
    let plot_w = SVG_W - MARGIN * 2 - 20;
    let plot_h = SVG_H - plot_y - 55;

    let max_val = values
        .iter()
        .filter_map(|v| v.get("value").and_then(|n| n.as_f64()))
        .fold(0.0_f64, f64::max)
        .max(1.0);

    let n = values.len().max(1);
    let bar_w = (plot_w as f64 / n as f64 * 0.7) as u32;
    let gap = (plot_w as f64 / n as f64 * 0.3) as u32;

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));
    svg.push_str(&svg_axes(plot_x, plot_y, plot_w, plot_h, "", ""));
    svg.push_str(&svg_tick_labels_y(plot_x, plot_y, plot_h, 0.0, max_val, 5));

    let colors = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6", "#dd4477", "#66aa00",
    ];

    for (i, v) in values.iter().enumerate() {
        let label = v.get("label").and_then(|s| s.as_str()).unwrap_or("");
        let val = v.get("value").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let bar_h = ((val / max_val) * plot_h as f64) as u32;
        let x = plot_x + (i as u32) * (bar_w + gap) + gap / 2;
        let y = plot_y + plot_h - bar_h;
        let color = colors[i % colors.len()];

        svg.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{bar_w}" height="{bar_h}" fill="{color}" rx="2"/>"#
        ));
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" text-anchor="middle" font-family="sans-serif" font-size="10" transform="rotate(-45,{},{})">{}</text>"#,
            x + bar_w / 2,
            plot_y + plot_h + 18,
            x + bar_w / 2,
            plot_y + plot_h + 18,
            html_escape(label)
        ));
        // Value on top
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" text-anchor="middle" font-family="sans-serif" font-size="9">{:.1}</text>"#,
            x + bar_w / 2,
            y - 4,
            val
        ));
    }

    svg.push_str("</svg>");
    svg_result(svg)
}

// ---------------------------------------------------------------------------
// 17. chart_pie_work
// ---------------------------------------------------------------------------

/// Generate SVG pie/donut chart.
pub fn chart_pie_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "values array is required"}),
    };
    let title = args_str(&args, "title", "Pie Chart");
    let donut = args_bool(&args, "donut", false);

    if values.is_empty() {
        return json!({"svg": "", "error": "no values"});
    }

    let total: f64 = values
        .iter()
        .filter_map(|v| v.get("value").and_then(|n| n.as_f64()))
        .sum();
    if total <= 0.0 {
        return json!({"svg": "", "error": "total must be positive"});
    }

    let cx = SVG_W as f64 / 2.0;
    let cy = SVG_H as f64 / 2.0 + 10.0;
    let radius = (SVG_H as f64 / 2.0 - 50.0).min(150.0);

    let colors = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6", "#dd4477", "#66aa00",
        "#b82e2e", "#316395",
    ];

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));

    let mut current_angle = -std::f64::consts::FRAC_PI_2; // start at top

    for (i, v) in values.iter().enumerate() {
        let label = v.get("label").and_then(|s| s.as_str()).unwrap_or("");
        let val = v.get("value").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let slice_angle = (val / total) * 2.0 * std::f64::consts::PI;
        let color = colors[i % colors.len()];

        let x1 = cx + radius * current_angle.cos();
        let y1 = cy + radius * current_angle.sin();
        let next_angle = current_angle + slice_angle;
        let x2 = cx + radius * next_angle.cos();
        let y2 = cy + radius * next_angle.sin();

        let large_arc = if slice_angle > std::f64::consts::PI {
            1
        } else {
            0
        };

        if donut {
            let inner_r = radius * 0.55;
            let ix1 = cx + inner_r * current_angle.cos();
            let iy1 = cy + inner_r * current_angle.sin();
            let ix2 = cx + inner_r * next_angle.cos();
            let iy2 = cy + inner_r * next_angle.sin();
            svg.push_str(&format!(
                r#"<path d="M {ix1:.1} {iy1:.1} L {x1:.1} {y1:.1} A {radius:.1} {radius:.1} 0 {large_arc} 1 {x2:.1} {y2:.1} L {ix2:.1} {iy2:.1} A {inner_r:.1} {inner_r:.1} 0 {large_arc} 0 {ix1:.1} {iy1:.1} Z" fill="{color}"/>"#
            ));
        } else {
            svg.push_str(&format!(
                r#"<path d="M {cx:.1} {cy:.1} L {x1:.1} {y1:.1} A {radius:.1} {radius:.1} 0 {large_arc} 1 {x2:.1} {y2:.1} Z" fill="{color}"/>"#
            ));
        }

        // Label
        let mid_angle = current_angle + slice_angle / 2.0;
        let label_r = if donut {
            (radius + radius * 0.55) / 2.0
        } else {
            radius * 0.65
        };
        let lx = cx + label_r * mid_angle.cos();
        let ly = cy + label_r * mid_angle.sin();

        if slice_angle > 0.15 {
            svg.push_str(&format!(
                r#"<text x="{lx:.1}" y="{ly:.1}" text-anchor="middle" dominant-baseline="central" font-family="sans-serif" font-size="10" fill="white" font-weight="bold">{:.0}%</text>"#,
                (val / total) * 100.0
            ));
        }

        // Legend
        let legend_y = SVG_H as f64 - 18.0;
        let legend_x = 20.0 + (i as f64 * 100.0).min((SVG_W - 100) as f64);
        svg.push_str(&format!(
            r#"<rect x="{legend_x:.0}" y="{legend_y:.0}" width="10" height="10" fill="{color}"/>"#
        ));
        svg.push_str(&format!(
            r#"<text x="{:.0}" y="{:.0}" font-family="sans-serif" font-size="9">{}</text>"#,
            legend_x + 14.0,
            legend_y + 9.0,
            html_escape(label)
        ));

        current_angle = next_angle;
    }

    svg.push_str("</svg>");
    svg_result(svg)
}

// ---------------------------------------------------------------------------
// 18. chart_histogram_work
// ---------------------------------------------------------------------------

/// Generate SVG histogram.
pub fn chart_histogram_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "values array is required"}),
    };
    let bins = args_usize_clamped(&args, "bins", 10, 1, MAX_HISTOGRAM_BINS);
    let title = args_str(&args, "title", "Histogram");

    let nums: Vec<f64> = values.iter().filter_map(|v| v.as_f64()).collect();
    if nums.is_empty() {
        return json!({"error": "no numeric values", "svg": "", "bin_edges": [], "bin_counts": []});
    }

    let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 1e-10 {
        1.0
    } else {
        max - min
    };
    let bin_width = range / bins as f64;

    // Compute bin edges and counts
    let mut bin_edges: Vec<f64> = Vec::with_capacity(bins + 1);
    let mut bin_counts: Vec<usize> = vec![0; bins];
    for i in 0..=bins {
        bin_edges.push(min + i as f64 * bin_width);
    }
    for &v in &nums {
        let idx = (((v - min) / bin_width) as usize).min(bins - 1);
        bin_counts[idx] += 1;
    }

    let max_count = bin_counts.iter().cloned().max().unwrap_or(1).max(1);

    let plot_x = MARGIN + 20;
    let plot_y = 45u32;
    let plot_w = SVG_W - MARGIN * 2 - 20;
    let plot_h = SVG_H - plot_y - 55;

    let bw = plot_w as f64 / bins as f64;

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));
    svg.push_str(&svg_axes(plot_x, plot_y, plot_w, plot_h, "Value", "Count"));
    svg.push_str(&svg_tick_labels_y(
        plot_x,
        plot_y,
        plot_h,
        0.0,
        max_count as f64,
        5,
    ));

    for (i, &count) in bin_counts.iter().enumerate() {
        let bar_h = (count as f64 / max_count as f64 * plot_h as f64) as u32;
        let x = plot_x + (i as f64 * bw) as u32;
        let y = plot_y + plot_h - bar_h;

        let bar_w_val = (bw as u32).saturating_sub(1).max(1);
        svg.push_str(&format!(
            "<rect x=\"{x}\" y=\"{y}\" width=\"{bar_w_val}\" height=\"{bar_h}\" fill=\"#3366cc\" rx=\"1\"/>"
        ));
    }

    svg.push_str("</svg>");
    let mut result = svg_result(svg);
    if let Value::Object(ref mut m) = result {
        m.insert(
            "bin_edges".to_string(),
            serde_json::to_value(&bin_edges).unwrap_or(Value::Null),
        );
        m.insert(
            "bin_counts".to_string(),
            serde_json::to_value(&bin_counts).unwrap_or(Value::Null),
        );
    }
    result
}

// ---------------------------------------------------------------------------
// 19. chart_scatter_work
// ---------------------------------------------------------------------------

/// Generate SVG scatter plot.
pub fn chart_scatter_work(args: Value) -> Value {
    let points = match args.get("points").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "points array is required"}),
    };
    let title = args_str(&args, "title", "Scatter Plot");

    let plot_x = MARGIN + 20;
    let plot_y = 45u32;
    let plot_w = SVG_W - MARGIN * 2 - 20;
    let plot_h = SVG_H - plot_y - 55;

    let xs: Vec<f64> = points
        .iter()
        .filter_map(|p| p.get("x").and_then(|v| v.as_f64()))
        .collect();
    let ys: Vec<f64> = points
        .iter()
        .filter_map(|p| p.get("y").and_then(|v| v.as_f64()))
        .collect();

    if xs.is_empty() {
        return json!({"error": "no points", "svg": ""});
    }

    let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let x_range = if (x_max - x_min).abs() < 1e-10 {
        1.0
    } else {
        x_max - x_min
    };
    let y_range = if (y_max - y_min).abs() < 1e-10 {
        1.0
    } else {
        y_max - y_min
    };

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));
    svg.push_str(&svg_axes(plot_x, plot_y, plot_w, plot_h, "X", "Y"));
    svg.push_str(&svg_tick_labels_y(plot_x, plot_y, plot_h, y_min, y_max, 5));

    for pt in &points {
        let x = pt.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let y = pt.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let px = plot_x as f64 + ((x - x_min) / x_range) * plot_w as f64;
        let py = (plot_y + plot_h) as f64 - ((y - y_min) / y_range) * plot_h as f64;
        svg.push_str(&format!(
            "<circle cx=\"{px:.1}\" cy=\"{py:.1}\" r=\"4\" fill=\"#3366cc\" opacity=\"0.7\"/>"
        ));
    }

    svg.push_str("</svg>");
    svg_result(svg)
}

// ---------------------------------------------------------------------------
// 20. chart_timeseries_work
// ---------------------------------------------------------------------------

/// Time-series chart with moving average.
pub fn chart_timeseries_work(args: Value) -> Value {
    let values = match args.get("values").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "values array is required"}),
    };
    let window = args_usize_clamped(&args, "window", 5, 1, MAX_TIMESERIES_WINDOW);
    let title = args_str(&args, "title", "Time Series");

    if values.is_empty() {
        return json!({"error": "no values", "svg": ""});
    }

    let plot_x = MARGIN + 20;
    let plot_y = 45u32;
    let plot_w = SVG_W - MARGIN * 2 - 20;
    let plot_h = SVG_H - plot_y - 55;

    let timestamps: Vec<String> = values
        .iter()
        .filter_map(|v| {
            v.get("timestamp")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    let nums: Vec<f64> = values
        .iter()
        .filter_map(|v| v.get("value").and_then(|n| n.as_f64()))
        .collect();

    if nums.is_empty() {
        return json!({"error": "no numeric values", "svg": ""});
    }

    let y_min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = if (y_max - y_min).abs() < 1e-10 {
        1.0
    } else {
        y_max - y_min
    };
    let n = nums.len();

    // Compute moving average
    let ma: Vec<f64> = {
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let start = (i + 1).saturating_sub(window);
            let slice = &nums[start..=i];
            result.push(slice.iter().sum::<f64>() / slice.len() as f64);
        }
        result
    };

    let x_range = if n > 1 { (n - 1) as f64 } else { 1.0 };

    let mut svg = svg_header(SVG_W, SVG_H, &title);
    svg.push_str(&svg_title(SVG_W, &title));
    svg.push_str(&svg_axes(plot_x, plot_y, plot_w, plot_h, "Time", "Value"));
    svg.push_str(&svg_tick_labels_y(plot_x, plot_y, plot_h, y_min, y_max, 5));

    // X ticks: show a few timestamps
    let tick_step = (n / 5).max(1);
    let x_ticks: Vec<String> = timestamps
        .iter()
        .enumerate()
        .filter(|(i, _)| i % tick_step == 0 || *i == n - 1)
        .map(|(_, t)| {
            // Shorten timestamp for display
            if t.len() > 16 {
                t[..16].to_string()
            } else {
                t.clone()
            }
        })
        .collect();
    svg.push_str(&svg_tick_labels_x(plot_x, plot_y, plot_w, plot_h, &x_ticks));

    // Raw data polyline
    let mut raw_points = String::new();
    for (i, &v) in nums.iter().enumerate() {
        let px = plot_x as f64 + (i as f64 / x_range) * plot_w as f64;
        let py = (plot_y + plot_h) as f64 - ((v - y_min) / y_range) * plot_h as f64;
        raw_points.push_str(&format!("{px:.1},{py:.1} "));
    }
    let raw_pts = raw_points.trim();
    svg.push_str(&format!(
        "<polyline points=\"{raw_pts}\" fill=\"none\" stroke=\"#3366cc\" stroke-width=\"1.5\" opacity=\"0.5\"/>"
    ));

    // Moving average polyline
    let mut ma_points = String::new();
    for (i, &v) in ma.iter().enumerate() {
        let px = plot_x as f64 + (i as f64 / x_range) * plot_w as f64;
        let py = (plot_y + plot_h) as f64 - ((v - y_min) / y_range) * plot_h as f64;
        ma_points.push_str(&format!("{px:.1},{py:.1} "));
    }
    let ma_pts = ma_points.trim();
    svg.push_str(&format!(
        "<polyline points=\"{ma_pts}\" fill=\"none\" stroke=\"#dc3912\" stroke-width=\"2\"/>"
    ));

    // Legend
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#3366cc" stroke-width="2" opacity="0.5"/>"##,
        plot_x + 10, plot_y + 10, plot_x + 30, plot_y + 10
    ));
    svg.push_str(&format!(
        r##"<text x="{}" y="{}" font-family="sans-serif" font-size="11">Raw</text>"##,
        plot_x + 34,
        plot_y + 14
    ));
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#dc3912" stroke-width="2"/>"##,
        plot_x + 10,
        plot_y + 26,
        plot_x + 30,
        plot_y + 26
    ));
    svg.push_str(&format!(
        r##"<text x="{}" y="{}" font-family="sans-serif" font-size="11">MA({window})</text>"##,
        plot_x + 34,
        plot_y + 30
    ));

    svg.push_str("</svg>");
    svg_result(svg)
}

// ---------------------------------------------------------------------------
// 21. chart_dependency_graph_work
// ---------------------------------------------------------------------------

/// Render module/file dependency graph as DOT format.
pub fn chart_dependency_graph_work(args: Value) -> Value {
    let nodes = match args.get("nodes").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "nodes array is required"}),
    };
    let edges = match args.get("edges").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "edges array is required"}),
    };
    let title = args_str(&args, "title", "Dependency Graph");

    let mut dot = format!("digraph \"{}\" {{\n", html_escape(&title));
    dot.push_str("    rankdir=LR;\n");
    dot.push_str("    node [shape=box, fontname=\"sans-serif\", fontsize=10];\n");
    dot.push_str("    edge [fontname=\"sans-serif\", fontsize=9];\n\n");

    // Nodes
    for n in &nodes {
        if let Some(name) = n.as_str() {
            dot.push_str(&format!("    \"{}\";\n", html_escape(name)));
        }
    }

    dot.push('\n');

    // Edges
    for e in &edges {
        let from = e.get("from").and_then(|v| v.as_str()).unwrap_or("");
        let to = e.get("to").and_then(|v| v.as_str()).unwrap_or("");
        let label = e.get("label").and_then(|v| v.as_str()).unwrap_or("");
        if !from.is_empty() && !to.is_empty() {
            if label.is_empty() {
                dot.push_str(&format!(
                    "    \"{}\" -> \"{}\";\n",
                    html_escape(from),
                    html_escape(to)
                ));
            } else {
                dot.push_str(&format!(
                    "    \"{}\" -> \"{}\" [label=\"{}\"];\n",
                    html_escape(from),
                    html_escape(to),
                    html_escape(label)
                ));
            }
        }
    }

    dot.push_str("}\n");

    json!({ "dot": dot })
}

// ---------------------------------------------------------------------------
// 22. chart_gantt_work
// ---------------------------------------------------------------------------

/// Generate Gantt chart as SVG.
pub fn chart_gantt_work(args: Value) -> Value {
    let tasks = match args.get("tasks").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => return json!({"error": "tasks array is required"}),
    };
    let title = args_str(&args, "title", "Gantt Chart");

    if tasks.is_empty() {
        return json!({"error": "no tasks", "svg": ""});
    }

    let plot_x = MARGIN + 80; // Extra margin for task names
    let plot_y = 45u32;
    let plot_w = SVG_W - plot_x - MARGIN;
    let row_h = 30u32;
    let plot_h = (tasks.len() as u32 * row_h).max(100);

    let h = plot_y + plot_h + 30;

    // Parse timestamps to find time range
    let parse_ts = |s: &str| -> f64 {
        // Try simple ISO-ish parsing: just find a numeric timestamp or use string sorting
        s.replace(['-', ':', 'T', 'Z'], "")
            .parse::<f64>()
            .unwrap_or(0.0)
    };

    let mut min_time = f64::INFINITY;
    let mut max_time = f64::NEG_INFINITY;
    for t in &tasks {
        let start = t.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
        let end = t.get("end_time").and_then(|v| v.as_str()).unwrap_or("");
        let s = parse_ts(start);
        let e = parse_ts(end);
        if s < min_time {
            min_time = s;
        }
        if e > max_time {
            max_time = e;
        }
    }
    let time_range = if (max_time - min_time).abs() < 1e-10 {
        1.0
    } else {
        max_time - min_time
    };

    let colors = [
        "#3366cc", "#dc3912", "#ff9900", "#109618", "#990099", "#0099c6", "#dd4477", "#66aa00",
        "#b82e2e", "#316395",
    ];

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {SVG_W} {h}\" width=\"{SVG_W}\" height=\"{h}\">"
    );
    let title_x = SVG_W / 2;
    svg.push_str(&format!(
        "<text x=\"{title_x}\" y=\"28\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"16\" font-weight=\"bold\">{}</text>",
        html_escape(&title)
    ));

    for (i, task) in tasks.iter().enumerate() {
        let name = task.get("name").and_then(|v| v.as_str()).unwrap_or("task");
        let start_str = task
            .get("start_time")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let end_str = task.get("end_time").and_then(|v| v.as_str()).unwrap_or("");
        let color = task
            .get("color")
            .and_then(|v| v.as_str())
            .unwrap_or(colors[i % colors.len()]);

        let s = parse_ts(start_str);
        let e = parse_ts(end_str);

        let x = plot_x as f64 + ((s - min_time) / time_range) * plot_w as f64;
        let w = ((e - s) / time_range) * plot_w as f64;
        let y = plot_y + i as u32 * row_h;

        // Task name
        let name_x = plot_x - 10;
        let name_y = y + row_h / 2 + 4;
        svg.push_str(&format!(
            "<text x=\"{name_x}\" y=\"{name_y}\" text-anchor=\"end\" font-family=\"sans-serif\" font-size=\"11\">{}</text>",
            html_escape(name)
        ));
        // Bar
        let bar_h = row_h - 6;
        svg.push_str(&format!(
            "<rect x=\"{x:.0}\" y=\"{y}\" width=\"{w:.0}\" height=\"{bar_h}\" fill=\"{color}\" rx=\"3\"/>"
        ));

        // Separator line
        let sep_y = y + row_h;
        let sep_x2 = plot_x + plot_w;
        svg.push_str(&format!(
            "<line x1=\"{plot_x}\" y1=\"{sep_y}\" x2=\"{sep_x2}\" y2=\"{sep_y}\" stroke=\"#eee\" stroke-width=\"0.5\"/>"
        ));
    }

    svg.push_str("</svg>");

    svg_result_with_dims(svg, SVG_W, h)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_count_args_are_clamped() {
        assert_eq!(
            args_usize_clamped(
                &json!({"limit": u64::MAX}),
                "limit",
                100,
                1,
                MAX_LOG_RESULT_LIMIT
            ),
            MAX_LOG_RESULT_LIMIT
        );
        assert_eq!(
            args_usize_clamped(&json!({"limit": 0}), "limit", 100, 1, MAX_LOG_RESULT_LIMIT),
            1
        );
        assert_eq!(
            args_usize_clamped(&json!({}), "limit", 100, 1, MAX_LOG_RESULT_LIMIT),
            100
        );
    }

    // --- log_tail_work ---

    #[test]
    fn test_log_tail_missing_path() {
        let result = log_tail_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_log_tail_empty_file() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.log");
        std::fs::write(&path, "").unwrap();
        let result = log_tail_work(json!({"path": path.to_str().unwrap()}));
        assert_eq!(result["total_lines"], 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_tail_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_tail");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(&path, "INFO line1\nERROR line2\nWARN line3\n").unwrap();
        let result = log_tail_work(json!({"path": path.to_str().unwrap(), "n": 10}));
        assert_eq!(result["total_lines"], 3);
        let lines = result["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["level"], "INFO");
        assert_eq!(lines[1]["level"], "ERROR");
        assert_eq!(lines[2]["level"], "WARN");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_tail_filter_level() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_filter");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(&path, "INFO line1\nERROR line2\nWARN line3\nDEBUG line4\n").unwrap();
        let result =
            log_tail_work(json!({"path": path.to_str().unwrap(), "filter_level": "ERROR"}));
        let lines = result["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["level"], "ERROR");
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_search_work ---

    #[test]
    fn test_log_search_missing_pattern() {
        let result = log_search_work(json!({"path": "/dev/null"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_log_search_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_search");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(&path, "hello world\nfoo bar\nhello again\n").unwrap();
        let result = log_search_work(json!({"path": path.to_str().unwrap(), "pattern": "hello"}));
        assert_eq!(result["total_matches"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_search_huge_max_results_is_clamped() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_search_clamp");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(&path, "hello world\nhello again\n").unwrap();
        let result = log_search_work(json!({
            "path": path.to_str().unwrap(),
            "pattern": "hello",
            "max_results": u64::MAX
        }));
        assert_eq!(result["total_matches"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_parse_json_work ---

    #[test]
    fn test_log_parse_json_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_jsonl");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jsonl");
        std::fs::write(&path, r#"{"level":"INFO","msg":"a"}"#).unwrap();
        std::fs::write(&path, r#"{"level":"ERROR","msg":"b"}"#).unwrap();
        // Actually write properly
        let content = r#"{"level":"INFO","msg":"a"}
{"level":"ERROR","msg":"b"}
"#;
        std::fs::write(&path, content).unwrap();
        let result = log_parse_json_work(json!({"path": path.to_str().unwrap()}));
        assert_eq!(result["total"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_parse_json_with_filter() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_jsonl_filt");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jsonl");
        let content = r#"{"level":"INFO","msg":"a"}
{"level":"ERROR","msg":"b"}
"#;
        std::fs::write(&path, content).unwrap();
        let result = log_parse_json_work(json!({
            "path": path.to_str().unwrap(),
            "filter": {"level": "ERROR"}
        }));
        assert_eq!(result["total"], 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_parse_nginx_work ---

    #[test]
    fn test_log_parse_nginx_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_nginx");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("access.log");
        std::fs::write(
            &path,
            r#"192.168.1.1 - - [15/Jan/2024:10:30:00 +0000] "GET /index.html HTTP/1.1" 200 1234
10.0.0.1 - - [15/Jan/2024:10:31:00 +0000] "POST /api HTTP/1.1" 404 56
"#,
        )
        .unwrap();
        let result = log_parse_nginx_work(json!({"path": path.to_str().unwrap()}));
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["ip"], "192.168.1.1");
        assert_eq!(entries[0]["method"], "GET");
        assert_eq!(entries[0]["status"], "200");
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_parse_syslog_work ---

    #[test]
    fn test_log_parse_syslog_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_syslog");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("syslog");
        std::fs::write(
            &path,
            "Jan 15 10:30:00 myhost sshd[1234]: Accepted connection\n",
        )
        .unwrap();
        let result = log_parse_syslog_work(json!({"path": path.to_str().unwrap()}));
        let entries = result["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["host"], "myhost");
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_level_summary_work ---

    #[test]
    fn test_log_level_summary_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_level");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(&path, "INFO a\nINFO b\nERROR c\nWARN d\nDEBUG e\n").unwrap();
        let result = log_level_summary_work(json!({"path": path.to_str().unwrap()}));
        assert_eq!(result["INFO"], 2);
        assert_eq!(result["ERROR"], 1);
        assert_eq!(result["WARN"], 1);
        assert_eq!(result["DEBUG"], 1);
        assert_eq!(result["total"], 5);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_level_summary_missing_path() {
        let result = log_level_summary_work(json!({}));
        assert!(result.get("error").is_some());
    }

    // --- log_error_cluster_work ---

    #[test]
    fn test_log_error_cluster_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_cluster");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(
            &path,
            "ERROR Connection failed to 10.0.0.1:5432\n\
             INFO OK\n\
             ERROR Connection failed to 10.0.0.2:5432\n\
             ERROR Timeout after 30000ms\n\
             ERROR Timeout after 15000ms\n",
        )
        .unwrap();
        let result = log_error_cluster_work(json!({"path": path.to_str().unwrap()}));
        let clusters = result["clusters"].as_array().unwrap();
        assert!(!clusters.is_empty());
        // The Connection errors should cluster together
        let conn_cluster = clusters
            .iter()
            .find(|c| c["pattern"].as_str().unwrap_or("").contains("Connection"));
        assert!(conn_cluster.is_some());
        assert_eq!(conn_cluster.unwrap()["count"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_time_range_filter_work ---

    #[test]
    fn test_log_time_range_filter_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_timerange");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.log");
        std::fs::write(
            &path,
            "2024-01-15T09:00:00 early event\n\
             2024-01-15T10:30:00 mid event\n\
             2024-01-15T12:00:00 late event\n",
        )
        .unwrap();
        let result = log_time_range_filter_work(json!({
            "path": path.to_str().unwrap(),
            "since": "2024-01-15T10:00",
            "until": "2024-01-15T11:00",
        }));
        assert_eq!(result["count"], 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_export_work ---

    #[test]
    fn test_log_export_missing_entries() {
        let result = log_export_work(json!({"format": "json", "output_path": "/tmp/x"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_log_export_json() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_export");
        std::fs::create_dir_all(&dir).unwrap();
        let out_path = dir.join("export.json");
        let result = log_export_work(json!({
            "entries": [{"a": 1}, {"a": 2}],
            "format": "json",
            "output_path": out_path.to_str().unwrap(),
        }));
        assert_eq!(result["format"], "json");
        assert_eq!(result["entry_count"], 2);
        assert!(out_path.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_export_csv() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_export_csv");
        std::fs::create_dir_all(&dir).unwrap();
        let out_path = dir.join("export.csv");
        let result = log_export_work(json!({
            "entries": [{"name": "a", "val": 1}, {"name": "b", "val": 2}],
            "format": "csv",
            "output_path": out_path.to_str().unwrap(),
        }));
        assert_eq!(result["format"], "csv");
        assert!(out_path.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_log_export_markdown() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_export_md");
        std::fs::create_dir_all(&dir).unwrap();
        let out_path = dir.join("export.md");
        let result = log_export_work(json!({
            "entries": [{"name": "a", "val": 1}],
            "format": "markdown",
            "output_path": out_path.to_str().unwrap(),
        }));
        assert_eq!(result["format"], "markdown");
        let content = std::fs::read_to_string(&out_path).unwrap();
        assert!(content.contains("| name |"));
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- log_audit_query_work ---

    #[test]
    fn test_log_audit_query_returns_json() {
        // This may fail if no audit log exists, but should return a JSON value
        let result = log_audit_query_work(json!({"limit": 5}));
        // Should either have entries or an error key, but not panic
        assert!(result.is_object());
    }

    // --- history_tool_calls_work ---

    #[test]
    fn test_history_tool_calls_returns_json() {
        let result = history_tool_calls_work(json!({"limit": 5}));
        assert!(result.is_object());
    }

    // --- history_bus_replay_work ---

    #[test]
    fn test_history_bus_replay_missing_path() {
        let result = history_bus_replay_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_history_bus_replay_basic() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_bus");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("agent-bus.jsonl");
        let content = r#"{"from":"agent1","to":"agent2","content":"hello","msg_type":"dm","timestamp":"2024-01-15T10:00:00"}
{"from":"agent2","to":"agent1","content":"world","msg_type":"dm","timestamp":"2024-01-15T11:00:00"}
"#;
        std::fs::write(&path, content).unwrap();
        let result = history_bus_replay_work(json!({"path": path.to_str().unwrap()}));
        assert_eq!(result["total"], 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_history_bus_replay_filter_agent() {
        let dir = std::env::temp_dir().join("pb_mcp_logs_charts_test_bus2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("agent-bus.jsonl");
        let content = r#"{"from":"agent1","to":"agent2","content":"hello","msg_type":"dm","timestamp":"2024-01-15T10:00:00"}
{"from":"agent3","to":"agent4","content":"other","msg_type":"dm","timestamp":"2024-01-15T11:00:00"}
"#;
        std::fs::write(&path, content).unwrap();
        let result =
            history_bus_replay_work(json!({"path": path.to_str().unwrap(), "agent": "agent1"}));
        assert_eq!(result["total"], 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    // --- history_search_past_calls_work ---

    #[test]
    fn test_history_search_missing_query() {
        let result = history_search_past_calls_work(json!({}));
        assert!(result.get("error").is_some());
    }

    // --- chart_line_work ---

    #[test]
    fn test_chart_line_missing_series() {
        let result = chart_line_work(json!({"title": "test"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_chart_line_basic() {
        let result = chart_line_work(json!({
            "series": [{"label": "A", "values": [{"x": 0, "y": 1}, {"x": 1, "y": 3}, {"x": 2, "y": 2}]}],
            "title": "Test",
            "x_label": "X",
            "y_label": "Y",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
        assert_eq!(result["width"], SVG_W);
        assert_eq!(result["height"], SVG_H);
    }

    // --- chart_bar_work ---

    #[test]
    fn test_chart_bar_missing_values() {
        let result = chart_bar_work(json!({"title": "test"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_chart_bar_basic() {
        let result = chart_bar_work(json!({
            "values": [{"label": "A", "value": 10}, {"label": "B", "value": 20}],
            "title": "Bars",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    // --- chart_pie_work ---

    #[test]
    fn test_chart_pie_basic() {
        let result = chart_pie_work(json!({
            "values": [{"label": "A", "value": 30}, {"label": "B", "value": 70}],
            "title": "Pie",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    #[test]
    fn test_chart_pie_donut() {
        let result = chart_pie_work(json!({
            "values": [{"label": "A", "value": 30}, {"label": "B", "value": 70}],
            "title": "Donut",
            "donut": true,
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    // --- chart_histogram_work ---

    #[test]
    fn test_chart_histogram_missing_values() {
        let result = chart_histogram_work(json!({"title": "test"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_chart_histogram_basic() {
        let result = chart_histogram_work(json!({
            "values": [1, 2, 2, 3, 3, 3, 4, 4, 5],
            "bins": 4,
            "title": "Hist",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
        assert_eq!(result["bin_edges"].as_array().unwrap().len(), 5); // bins + 1 edges
        assert_eq!(result["bin_counts"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn test_chart_histogram_clamps_bins() {
        let result = chart_histogram_work(json!({
            "values": [1, 2, 3],
            "bins": 0,
            "title": "Hist",
        }));
        assert_eq!(result["bin_counts"].as_array().unwrap().len(), 1);

        let result = chart_histogram_work(json!({
            "values": [1, 2, 3],
            "bins": 99999999999u64,
            "title": "Hist",
        }));
        assert_eq!(
            result["bin_counts"].as_array().unwrap().len(),
            MAX_HISTOGRAM_BINS
        );
    }

    // --- chart_scatter_work ---

    #[test]
    fn test_chart_scatter_basic() {
        let result = chart_scatter_work(json!({
            "points": [{"x": 0, "y": 0}, {"x": 1, "y": 1}, {"x": 2, "y": 0.5}],
            "title": "Scatter",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    // --- chart_timeseries_work ---

    #[test]
    fn test_chart_timeseries_missing_values() {
        let result = chart_timeseries_work(json!({"title": "test"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_chart_timeseries_basic() {
        let result = chart_timeseries_work(json!({
            "values": [
                {"timestamp": "2024-01-01", "value": 10},
                {"timestamp": "2024-01-02", "value": 20},
                {"timestamp": "2024-01-03", "value": 15},
            ],
            "window": 2,
            "title": "TS",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    // --- chart_dependency_graph_work ---

    #[test]
    fn test_chart_dependency_graph_basic() {
        let result = chart_dependency_graph_work(json!({
            "nodes": ["A", "B", "C"],
            "edges": [{"from": "A", "to": "B", "label": "uses"}, {"from": "B", "to": "C"}],
            "title": "Deps",
        }));
        let dot = result["dot"].as_str().unwrap();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("A"));
        assert!(dot.contains("->"));
    }

    #[test]
    fn test_chart_dependency_graph_missing_nodes() {
        let result = chart_dependency_graph_work(json!({"edges": []}));
        assert!(result.get("error").is_some());
    }

    // --- chart_gantt_work ---

    #[test]
    fn test_chart_gantt_basic() {
        let result = chart_gantt_work(json!({
            "tasks": [
                {"name": "Task A", "start_time": "20240101T090000", "end_time": "20240101T120000", "color": "#3366cc"},
                {"name": "Task B", "start_time": "20240101T110000", "end_time": "20240101T150000"},
            ],
            "title": "Gantt",
        }));
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    #[test]
    fn test_chart_gantt_missing_tasks() {
        let result = chart_gantt_work(json!({"title": "test"}));
        assert!(result.get("error").is_some());
    }

    // --- normalize_for_cluster ---

    #[test]
    fn test_normalize_for_cluster() {
        // Both IPs should normalize to the same pattern after IPv4 replacement
        let a = normalize_for_cluster("Error connecting to 10.0.0.1:5432");
        let b = normalize_for_cluster("Error connecting to 10.0.0.2:5432");
        assert_eq!(a, b, "Different IPs should normalize to the same pattern");
        // After normalization IPv4 addresses and port numbers are replaced with *
        assert!(a.contains('*'), "Normalized pattern should contain *");
        assert!(!a.contains("5432"), "Port number should be replaced");
    }

    // --- detect_level ---

    #[test]
    fn test_detect_level() {
        assert_eq!(detect_level("2024-01-01 ERROR something"), "ERROR");
        assert_eq!(detect_level("2024-01-01 WARN something"), "WARN");
        assert_eq!(detect_level("2024-01-01 INFO something"), "INFO");
        assert_eq!(detect_level("2024-01-01 DEBUG something"), "DEBUG");
        assert_eq!(detect_level("just a line"), "UNKNOWN");
    }

    // --- html_escape ---

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a<b>c&d"), "a&lt;b&gt;c&amp;d");
        assert_eq!(html_escape("normal"), "normal");
    }
}
