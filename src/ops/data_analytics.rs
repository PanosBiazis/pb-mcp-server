//! # Data Analytics Tools
//!
//! CSV analysis, JSON flattening, JSONL diffing, regex extraction,
//! time-series anomaly detection, and Parquet reading.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool_available(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 1. data_csv_analyze
// ---------------------------------------------------------------------------

/// Analyze a CSV file: detect headers, count rows, infer column types, find nulls.
///
/// Reads up to `max_rows` data rows. Uses a hand-rolled CSV parser that respects
/// quoted fields (including embedded commas and escaped quotes).
///
/// Returns JSON:
/// ```json
/// {
///   "path": "...", "row_count": N, "col_count": N,
///   "columns": [{"name","type","null_count","sample_values":[…5]}],
///   "preview": [{row-as-object}, …3]
/// }
/// ```
pub fn data_csv_analyze_work(path: PathBuf, max_rows: usize) -> String {
    let path_str = path.to_string_lossy().to_string();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return json!({"error": format!("failed to read {path_str}: {e}")}).to_string()
        }
    };

    let mut lines = content.lines();

    // First line = header
    let header_line = match lines.next() {
        Some(l) => l,
        None => return json!({"error": "empty file", "path": path_str}).to_string(),
    };

    let headers: Vec<String> = parse_csv_line(header_line)
        .into_iter()
        .map(|h| h.trim().trim_matches('"').to_string())
        .collect();
    let col_count = headers.len();

    let mut null_counts = vec![0usize; col_count];
    let mut numeric_counts = vec![0usize; col_count];
    let mut sample_values: Vec<Vec<String>> = vec![Vec::new(); col_count];
    let mut preview_rows: Vec<Vec<String>> = Vec::new();
    let mut row_count = 0usize;

    for line in lines.take(max_rows.max(1).min(100_000)) {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_line(line);
        row_count += 1;

        for (i, raw_cell) in cols.iter().enumerate() {
            if i >= col_count {
                break;
            }
            let cell = raw_cell.trim().trim_matches('"');
            if cell.is_empty()
                || cell.eq_ignore_ascii_case("null")
                || cell.eq_ignore_ascii_case("n/a")
                || cell.eq_ignore_ascii_case("na")
                || cell == "\\N"
            {
                null_counts[i] += 1;
            } else {
                if cell.parse::<f64>().is_ok() {
                    numeric_counts[i] += 1;
                }
                if sample_values[i].len() < 5 {
                    sample_values[i].push(cell.to_string());
                }
            }
        }

        if preview_rows.len() < 3 {
            preview_rows.push(cols.into_iter().map(|c| c.trim().to_string()).collect());
        }
    }

    let columns: Vec<Value> = headers
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let non_null = row_count.saturating_sub(null_counts[i]);
            let col_type = if non_null == 0 {
                "unknown"
            } else if non_null > 0
                && numeric_counts[i] as f64 / non_null as f64 >= 0.90
            {
                "numeric"
            } else {
                "string"
            };
            json!({
                "name":          name,
                "type":          col_type,
                "null_count":    null_counts[i],
                "sample_values": &sample_values[i],
            })
        })
        .collect();

    // First 3 rows as objects
    let preview: Vec<Value> = preview_rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, h) in headers.iter().enumerate() {
                obj.insert(h.clone(), json!(row.get(i).cloned().unwrap_or_default()));
            }
            Value::Object(obj)
        })
        .collect();

    json!({
        "path":      path_str,
        "row_count": row_count,
        "col_count": col_count,
        "columns":   columns,
        "preview":   preview,
    })
    .to_string()
}

/// Parse one CSV line, respecting double-quoted fields (RFC 4180).
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next(); // escaped ""
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields
}

// ---------------------------------------------------------------------------
// 2. data_json_flatten
// ---------------------------------------------------------------------------

/// Flatten a nested JSON file into a single-level object using a key separator.
///
/// Arrays become `key.0`, `key.1`, etc.
///
/// Returns JSON:
/// ```json
/// { "path": "...", "separator": ".", "key_count": N,
///   "flattened": {"a.b.c": "val", ...},
///   "csv_preview": "a.b.c,...\nval,..." }
/// ```
pub fn data_json_flatten_work(path: PathBuf, separator: String) -> String {
    let path_str = path.to_string_lossy().to_string();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return json!({"error": format!("failed to read {path_str}: {e}")}).to_string()
        }
    };

    let value: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return json!({"error": format!("invalid JSON: {e}"), "path": path_str}).to_string()
        }
    };

    let mut flat = serde_json::Map::new();
    flatten_value(&value, String::new(), &separator, &mut flat);

    // Build a compact CSV preview (headers + first row of values)
    let keys: Vec<String> = flat.keys().cloned().collect();
    let header = keys.join(",");
    let values_row = keys
        .iter()
        .map(|k| {
            flat[k]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| flat[k].to_string())
        })
        .collect::<Vec<_>>()
        .join(",");
    let csv_preview = format!("{header}\n{values_row}");

    let key_count = flat.len();
    json!({
        "path":        path_str,
        "separator":   separator,
        "key_count":   key_count,
        "flattened":   Value::Object(flat),
        "csv_preview": csv_preview,
    })
    .to_string()
}

/// Recursively flatten a JSON value into a dot-separated flat map.
fn flatten_value(
    value: &Value,
    prefix: String,
    sep: &str,
    out: &mut serde_json::Map<String, Value>,
) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let new_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}{sep}{k}")
                };
                flatten_value(v, new_key, sep, out);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_key = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}{sep}{i}")
                };
                flatten_value(v, new_key, sep, out);
            }
        }
        _ => {
            out.insert(prefix, value.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// 3. data_diff_jsonl
// ---------------------------------------------------------------------------

/// Diff two JSONL files by key field or by line position.
///
/// If `key_field` is given, records are matched by that field's value across files.
/// Otherwise records are matched positionally.
///
/// Returns JSON:
/// ```json
/// { "added_count": N, "removed_count": N, "modified_count": N,
///   "added": [...], "removed": [...],
///   "modified": [{"key"|"position", "before": {...}, "after": {...}}] }
/// ```
pub fn data_diff_jsonl_work(
    path_a: PathBuf,
    path_b: PathBuf,
    key_field: Option<String>,
) -> String {
    let path_a_str = path_a.to_string_lossy().to_string();
    let path_b_str = path_b.to_string_lossy().to_string();

    let read_jsonl = |p: &PathBuf, label: &str| -> Result<Vec<Value>, String> {
        std::fs::read_to_string(p)
            .map_err(|e| format!("failed to read {label}: {e}"))
            .map(|c| {
                c.lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect()
            })
    };

    let records_a = match read_jsonl(&path_a, &path_a_str) {
        Ok(r) => r,
        Err(e) => return json!({"error": e}).to_string(),
    };
    let records_b = match read_jsonl(&path_b, &path_b_str) {
        Ok(r) => r,
        Err(e) => return json!({"error": e}).to_string(),
    };

    let mut added: Vec<Value> = Vec::new();
    let mut removed: Vec<Value> = Vec::new();
    let mut modified: Vec<Value> = Vec::new();

    if let Some(ref key) = key_field {
        // Key-based diff using HashMap<String, Value>
        let make_map = |records: &[Value]| -> HashMap<String, Value> {
            records
                .iter()
                .filter_map(|r| {
                    r.get(key).map(|k| {
                        let key_str = k
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| k.to_string());
                        (key_str, r.clone())
                    })
                })
                .collect()
        };

        let map_a = make_map(&records_a);
        let map_b = make_map(&records_b);

        for (k, va) in &map_a {
            match map_b.get(k) {
                Some(vb) if va != vb => {
                    modified.push(json!({"key": k, "before": va, "after": vb}))
                }
                Some(_) => {} // unchanged
                None => removed.push(va.clone()),
            }
        }
        for (k, vb) in &map_b {
            if !map_a.contains_key(k) {
                added.push(vb.clone());
            }
        }
    } else {
        // Positional diff
        let max_len = records_a.len().max(records_b.len());
        for i in 0..max_len {
            match (records_a.get(i), records_b.get(i)) {
                (Some(a), Some(b)) => {
                    if a != b {
                        modified.push(json!({"position": i, "before": a, "after": b}));
                    }
                }
                (Some(a), None) => removed.push(a.clone()),
                (None, Some(b)) => added.push(b.clone()),
                (None, None) => {}
            }
        }
    }

    let added_count = added.len();
    let removed_count = removed.len();
    let modified_count = modified.len();

    json!({
        "path_a":         path_a_str,
        "path_b":         path_b_str,
        "key_field":      key_field,
        "added_count":    added_count,
        "removed_count":  removed_count,
        "modified_count": modified_count,
        "added":          added,
        "removed":        removed,
        "modified":       modified,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 4. data_regex_extract
// ---------------------------------------------------------------------------

/// Extract regex matches (with optional named capture groups) from a file.
///
/// Supports named groups: `(?P<ip>\d+\.\d+\.\d+\.\d+)`.
/// Unnamed groups are keyed by index ("0", "1", …).
///
/// Returns JSON:
/// ```json
/// { "pattern": "...", "matches": [{"line": N, "text": "...", "groups": {...}}],
///   "total_matches": N, "truncated": false }
/// ```
pub fn data_regex_extract_work(path: PathBuf, pattern: String, max_matches: usize) -> String {
    let path_str = path.to_string_lossy().to_string();

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return json!({"error": format!("failed to read {path_str}: {e}")}).to_string()
        }
    };

    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => {
            return json!({"error": format!("invalid regex: {e}"), "pattern": pattern}).to_string()
        }
    };

    let capture_names: Vec<Option<&str>> = re.capture_names().collect();
    let mut matches_vec: Vec<Value> = Vec::new();
    let mut total_matches = 0usize;
    let mut truncated = false;

    for (line_num, line) in content.lines().enumerate() {
        for caps in re.captures_iter(line) {
            total_matches += 1;
            if matches_vec.len() >= max_matches {
                truncated = true;
                continue;
            }
            let mut groups = serde_json::Map::new();
            for (i, name_opt) in capture_names.iter().enumerate() {
                if let Some(cap_match) = caps.get(i) {
                    let key = match name_opt {
                        Some(n) => n.to_string(),
                        None => i.to_string(),
                    };
                    groups.insert(key, json!(cap_match.as_str()));
                }
            }
            matches_vec.push(json!({
                "line":   line_num + 1,
                "text":   line,
                "groups": Value::Object(groups),
            }));
        }
    }

    json!({
        "path":          path_str,
        "pattern":       pattern,
        "matches":       matches_vec,
        "total_matches": total_matches,
        "truncated":     truncated,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 5. data_timeseries_detect
// ---------------------------------------------------------------------------

/// Detect anomalies in a time series using a rolling z-score.
///
/// Flags any index `i` where `|values[i] - rolling_mean(window)| > threshold * rolling_std`.
///
/// Returns JSON:
/// ```json
/// { "total_points": N, "window": W, "threshold": T, "anomaly_count": K,
///   "anomalies": [{"index": N, "value": F, "zscore": F, "rolling_mean": F, "rolling_std": F}],
///   "stats": {"mean": F, "std": F, "min": F, "max": F} }
/// ```
pub fn data_timeseries_detect_work(values: Vec<f64>, window: usize, threshold: f64) -> String {
    if values.is_empty() {
        return json!({"error": "empty values array"}).to_string();
    }

    let total_points = values.len();
    let window = window.clamp(2, total_points);

    // Global statistics
    let mean: f64 = values.iter().sum::<f64>() / total_points as f64;
    let variance: f64 =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / total_points as f64;
    let std_dev = variance.sqrt();
    let min = values
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max = values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    // Rolling window anomaly detection
    let mut anomalies: Vec<Value> = Vec::new();
    for i in 0..total_points {
        let start = i.saturating_sub(window - 1);
        let slice = &values[start..=i];

        let w_mean: f64 = slice.iter().sum::<f64>() / slice.len() as f64;
        let w_var: f64 =
            slice.iter().map(|v| (v - w_mean).powi(2)).sum::<f64>() / slice.len() as f64;
        let w_std = w_var.sqrt();

        if w_std > 1e-12 {
            let zscore = (values[i] - w_mean) / w_std;
            if zscore.abs() > threshold {
                anomalies.push(json!({
                    "index":        i,
                    "value":        values[i],
                    "zscore":       (zscore * 1e6).round() / 1e6,
                    "rolling_mean": (w_mean * 1e6).round() / 1e6,
                    "rolling_std":  (w_std  * 1e6).round() / 1e6,
                }));
            }
        }
    }

    let anomaly_count = anomalies.len();
    json!({
        "total_points":  total_points,
        "window":        window,
        "threshold":     threshold,
        "anomaly_count": anomaly_count,
        "anomalies":     anomalies,
        "stats": {
            "mean": (mean    * 1e6).round() / 1e6,
            "std":  (std_dev * 1e6).round() / 1e6,
            "min":  min,
            "max":  max,
        },
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 6. data_parquet_read
// ---------------------------------------------------------------------------

/// Read a Parquet file using Python/pandas or DuckDB (whichever is available).
///
/// Returns JSON with schema and sample rows, or an error if neither tool is present.
pub fn data_parquet_read_work(path: PathBuf, limit: usize) -> String {
    let path_str = path.to_string_lossy().to_string();

    if !path.exists() {
        return json!({"error": format!("file not found: {path_str}")}).to_string();
    }

    // ── Python 3 + pandas ────────────────────────────────────────────────────
    if tool_available("python3") {
        let script = format!(
            r#"import sys, json
try:
    import pandas as pd
    df = pd.read_parquet(r'{path}')
    schema = {{col: str(dtype) for col, dtype in df.dtypes.items()}}
    rows = df.head({limit}).to_dict(orient='records')
    print(json.dumps({{"schema": schema, "rows": rows, "total_rows": len(df), "method": "pandas"}}))
except ImportError:
    print(json.dumps({{"error": "pandas not installed; run: pip install pandas pyarrow"}}))
except Exception as e:
    print(json.dumps({{"error": str(e)}}))
"#,
            path = path_str,
            limit = limit,
        );

        if let Ok(o) = Command::new("python3").args(["-c", &script]).output() {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !stdout.is_empty() {
                if let Ok(parsed) = serde_json::from_str::<Value>(&stdout) {
                    if parsed.get("error").is_none() {
                        return json!({
                            "path":  path_str,
                            "limit": limit,
                            "tool":  "python3+pandas",
                            "data":  parsed,
                        })
                        .to_string();
                    }
                    // pandas-specific error → fall through to DuckDB
                }
            }
        }
    }

    // ── DuckDB ───────────────────────────────────────────────────────────────
    if tool_available("duckdb") {
        let query = format!("SELECT * FROM '{}' LIMIT {}", path_str, limit);
        match Command::new("duckdb").args(["-json", "-c", &query]).output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                if o.status.success() && !stdout.is_empty() {
                    let rows =
                        serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| json!(stdout));
                    return json!({
                        "path":   path_str,
                        "limit":  limit,
                        "tool":   "duckdb",
                        "rows":   rows,
                        "stderr": stderr,
                    })
                    .to_string();
                }
                return json!({
                    "error": format!("duckdb failed: {stderr}"),
                    "path":  path_str,
                })
                .to_string();
            }
            Err(e) => {
                return json!({
                    "error": format!("duckdb execution failed: {e}"),
                    "path":  path_str,
                })
                .to_string();
            }
        }
    }

    json!({
        "error": "neither python3+pandas nor duckdb available; \
                  install one: pip install pandas pyarrow  OR  https://duckdb.org",
        "path":           path_str,
        "tool_available": false,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ── CSV ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_csv_analyze_basic() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "name,age,score").unwrap();
        writeln!(f, "Alice,30,9.5").unwrap();
        writeln!(f, "Bob,25,8.0").unwrap();
        writeln!(f, ",40,").unwrap(); // nulls

        let result = data_csv_analyze_work(f.path().to_path_buf(), 1000);
        let parsed: Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(parsed["row_count"], 3);
        assert_eq!(parsed["col_count"], 3);
        let cols = parsed["columns"].as_array().unwrap();
        assert_eq!(cols[0]["name"], "name");
        assert_eq!(cols[0]["type"], "string");
        assert_eq!(cols[1]["type"], "numeric");
    }

    #[test]
    fn test_csv_analyze_missing_file() {
        let result = data_csv_analyze_work(
            PathBuf::from("/tmp/pb_no_such_file_xyz.csv"),
            100,
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("error").is_some());
    }

    #[test]
    fn test_parse_csv_line_quoted() {
        let fields = parse_csv_line(r#"hello,"world,comma","quoted ""escape"""#);
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[1], "world,comma");
        assert_eq!(fields[2], r#"quoted "escape""#);
    }

    // ── JSON flatten ─────────────────────────────────────────────────────────

    #[test]
    fn test_json_flatten_basic() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, r#"{{"a":{{"b":1}},"c":[10,20]}}"#).unwrap();

        let result =
            data_json_flatten_work(f.path().to_path_buf(), ".".to_string());
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let flat = &parsed["flattened"];
        assert_eq!(flat["a.b"], 1);
        assert_eq!(flat["c.0"], 10);
        assert_eq!(flat["c.1"], 20);
    }

    // ── JSONL diff ───────────────────────────────────────────────────────────

    #[test]
    fn test_diff_jsonl_positional() {
        let mut fa = tempfile::NamedTempFile::new().unwrap();
        let mut fb = tempfile::NamedTempFile::new().unwrap();
        writeln!(fa, r#"{{"id":1,"v":"a"}}"#).unwrap();
        writeln!(fa, r#"{{"id":2,"v":"b"}}"#).unwrap();
        writeln!(fb, r#"{{"id":1,"v":"a"}}"#).unwrap();
        writeln!(fb, r#"{{"id":2,"v":"changed"}}"#).unwrap();
        writeln!(fb, r#"{{"id":3,"v":"new"}}"#).unwrap();

        let result = data_diff_jsonl_work(
            fa.path().to_path_buf(),
            fb.path().to_path_buf(),
            None,
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["modified_count"], 1);
        assert_eq!(parsed["added_count"], 1);
        assert_eq!(parsed["removed_count"], 0);
    }

    #[test]
    fn test_diff_jsonl_by_key() {
        let mut fa = tempfile::NamedTempFile::new().unwrap();
        let mut fb = tempfile::NamedTempFile::new().unwrap();
        writeln!(fa, r#"{{"id":"x","v":1}}"#).unwrap();
        writeln!(fb, r#"{{"id":"x","v":2}}"#).unwrap();
        writeln!(fb, r#"{{"id":"y","v":3}}"#).unwrap();

        let result = data_diff_jsonl_work(
            fa.path().to_path_buf(),
            fb.path().to_path_buf(),
            Some("id".to_string()),
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["modified_count"], 1);
        assert_eq!(parsed["added_count"], 1);
    }

    // ── Regex extract ────────────────────────────────────────────────────────

    #[test]
    fn test_regex_extract_named_groups() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "connect from 192.168.1.1 port 22").unwrap();
        writeln!(f, "connect from 10.0.0.5 port 80").unwrap();

        let result = data_regex_extract_work(
            f.path().to_path_buf(),
            r"(?P<ip>\d+\.\d+\.\d+\.\d+)".to_string(),
            100,
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total_matches"], 2);
        let m0 = &parsed["matches"][0];
        assert_eq!(m0["groups"]["ip"], "192.168.1.1");
    }

    #[test]
    fn test_regex_extract_bad_pattern() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "anything").unwrap();

        let result = data_regex_extract_work(
            f.path().to_path_buf(),
            "(?P<unclosed".to_string(),
            10,
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("error").is_some());
    }

    // ── Time-series anomaly ──────────────────────────────────────────────────

    #[test]
    fn test_timeseries_spike_detected() {
        let mut values: Vec<f64> = (0..50).map(|_| 1.0_f64).collect();
        values[25] = 100.0; // spike

        let result = data_timeseries_detect_work(values, 10, 3.0);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["anomaly_count"].as_u64().unwrap_or(0) >= 1);
        let anomalies = parsed["anomalies"].as_array().unwrap();
        let has_spike = anomalies.iter().any(|a| a["index"] == 25);
        assert!(has_spike, "spike at index 25 should be detected");
    }

    #[test]
    fn test_timeseries_empty() {
        let result = data_timeseries_detect_work(vec![], 5, 2.0);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("error").is_some());
    }

    // ── Parquet read ─────────────────────────────────────────────────────────

    #[test]
    fn test_parquet_read_missing_file() {
        let result = data_parquet_read_work(
            PathBuf::from("/tmp/pb_no_such_file_xyz.parquet"),
            10,
        );
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.get("error").is_some());
    }
}
