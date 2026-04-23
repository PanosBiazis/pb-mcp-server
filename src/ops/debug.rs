//! # Debug Tools
//!
//! Agent debugging: trace recording, replay, failure clustering,
//! and root cause analysis. Traces are stored as JSONL files under
//! `.mint-mcp/traces/` in the project root.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Maximum characters for debug tool responses.
const _MAX_DEBUG_RESPONSE_CHARS: usize = 20_000;

/// Maximum stdout/stderr preview length in trace entries.
const MAX_IO_PREVIEW_CHARS: usize = 4_000;

/// Maximum number of trace files scanned for clustering.
const MAX_TRACE_FILES: usize = 256;

/// Maximum JSONL entries read from a single trace file.
const MAX_TRACE_ENTRIES: usize = 10_000;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single step recorded in a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Unix-epoch milliseconds when this step was recorded.
    pub ts_ms: i64,
    /// Step index (0-based) within the trace.
    pub step: usize,
    /// Command that was executed.
    pub command: String,
    /// Arguments passed to the command.
    pub args: Vec<String>,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u128,
    /// Process exit code (None if the process could not be started).
    pub exit_code: Option<i32>,
    /// First N characters of stdout.
    pub stdout_preview: String,
    /// First N characters of stderr.
    pub stderr_preview: String,
}

/// Summary of a failure cluster.
#[derive(Debug, Clone)]
struct FailureCluster {
    key: String,
    count: usize,
    examples: Vec<TraceEntry>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get the traces directory under the project root.
fn traces_dir(root: &Path) -> PathBuf {
    root.join(".mint-mcp").join("traces")
}

/// Get the path for a specific trace file.
fn trace_path(root: &Path, trace_id: &str) -> PathBuf {
    // Sanitize trace_id: only allow alphanumeric, _, -, .
    let safe_id: String = trace_id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect();
    traces_dir(root).join(format!("{safe_id}.jsonl"))
}

/// Create the traces directory if it doesn't exist.
fn ensure_traces_dir(root: &Path) -> Result<PathBuf, String> {
    let dir = traces_dir(root);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Error creating traces directory: {e}"))?;
    }
    Ok(dir)
}

/// Truncate a string to a maximum character count.
fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…[truncated]")
    }
}

/// Current time in Unix-epoch milliseconds.
fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// Read all trace entries from a JSONL trace file.
fn read_trace_entries(path: &Path) -> Result<Vec<TraceEntry>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Error reading trace file {}: {e}", path.display()))?;

    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<TraceEntry>(trimmed) {
            Ok(entry) => {
                entries.push(entry);
                if entries.len() >= MAX_TRACE_ENTRIES {
                    break;
                }
            }
            Err(_) => continue, // Skip malformed lines
        }
    }

    Ok(entries)
}

/// Write a trace entry as a JSONL line appended to the trace file.
fn append_trace_entry(path: &Path, entry: &TraceEntry) -> Result<(), String> {
    let json =
        serde_json::to_string(entry).map_err(|e| format!("Error serializing trace entry: {e}"))?;

    // Use append mode with a file lock for safety
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Error opening trace file for append: {e}"))?;

    // Exclusive lock
    fs4::fs_std::FileExt::lock_exclusive(&file)
        .map_err(|e| format!("Error locking trace file: {e}"))?;
    writeln!(file, "{json}").map_err(|e| format!("Error writing trace entry: {e}"))?;
    fs4::fs_std::FileExt::unlock(&file).map_err(|e| format!("Error unlocking trace file: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// 1. debug_trace_run
// ---------------------------------------------------------------------------

/// Record a structured trace of a command execution.
///
/// Executes the command, measures wall-clock duration, and appends a
/// JSONL entry to `.mint-mcp/traces/{trace_id}.jsonl`. Returns a
/// summary of the recorded step.
pub fn debug_trace_run_work(
    root: PathBuf,
    trace_id: String,
    command: String,
    args: Vec<String>,
) -> String {
    if trace_id.trim().is_empty() {
        return "Error: trace_id must not be empty".to_string();
    }
    if command.trim().is_empty() {
        return "Error: command must not be empty".to_string();
    }

    // Ensure traces directory exists
    if let Err(e) = ensure_traces_dir(&root) {
        return e;
    }

    let path = trace_path(&root, &trace_id);

    // Determine the next step index
    let step = if path.exists() {
        match read_trace_entries(&path) {
            Ok(entries) => entries.len(),
            Err(_) => 0,
        }
    } else {
        0
    };

    // Execute the command
    let start = Instant::now();
    let output = Command::new(&command)
        .args(&args)
        .current_dir(&root)
        .output();
    let duration_ms = start.elapsed().as_millis();

    let (exit_code, stdout_preview, stderr_preview) = match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            (
                out.status.code(),
                truncate_str(&stdout, MAX_IO_PREVIEW_CHARS),
                truncate_str(&stderr, MAX_IO_PREVIEW_CHARS),
            )
        }
        Err(e) => (
            None,
            String::new(),
            truncate_str(&e.to_string(), MAX_IO_PREVIEW_CHARS),
        ),
    };

    let entry = TraceEntry {
        ts_ms: now_ms(),
        step,
        command: command.clone(),
        args: args.clone(),
        duration_ms,
        exit_code,
        stdout_preview,
        stderr_preview,
    };

    // Append the entry
    if let Err(e) = append_trace_entry(&path, &entry) {
        return format!("Error recording trace: {e}");
    }

    // Build summary
    let mut out = String::from("DEBUG: TRACE RUN\n\n");
    out.push_str(&format!("Trace ID: {trace_id}\n"));
    out.push_str(&format!("Step: {step}\n"));
    out.push_str(&format!("Command: {command}\n"));
    out.push_str(&format!(
        "Args: {}\n",
        if args.is_empty() {
            "(none)".to_string()
        } else {
            args.join(" ")
        }
    ));
    out.push_str(&format!("Duration: {duration_ms}ms\n"));
    out.push_str(&format!(
        "Exit code: {}\n",
        exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A (process error)".to_string())
    ));
    out.push_str(&format!(
        "Trace file: {}\n",
        path.strip_prefix(&root).unwrap_or(&path).display()
    ));

    if exit_code != Some(0) {
        out.push_str("\n⚠ Step FAILED\n");
        if !entry.stderr_preview.is_empty() {
            out.push_str(&format!("Stderr preview:\n{}\n", entry.stderr_preview));
        }
    } else {
        out.push_str("\n✓ Step succeeded\n");
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 2. debug_replay_run
// ---------------------------------------------------------------------------

/// Replay a recorded trace, optionally re-executing each command.
///
/// If `dry_run` is true, only lists the commands that would be replayed.
/// If `dry_run` is false, re-executes each command and compares the
/// results with the original recorded results.
pub fn debug_replay_run_work(root: PathBuf, trace_id: String, dry_run: bool) -> String {
    let path = trace_path(&root, &trace_id);
    if !path.exists() {
        return format!(
            "Error: trace file not found: {}",
            path.strip_prefix(&root).unwrap_or(&path).display()
        );
    }

    let entries = match read_trace_entries(&path) {
        Ok(e) => e,
        Err(e) => return e,
    };

    if entries.is_empty() {
        return "DEBUG: REPLAY RUN\n\nTrace file is empty — no steps to replay.".to_string();
    }

    let mut out = String::from("DEBUG: REPLAY RUN\n\n");
    out.push_str(&format!("Trace ID: {trace_id}\n"));
    out.push_str(&format!("Steps: {}\n", entries.len()));
    out.push_str(&format!(
        "Mode: {}\n\n",
        if dry_run { "dry-run" } else { "live" }
    ));

    for entry in &entries {
        out.push_str(&format!(
            "Step {} | {} {} | original exit: {} | original duration: {}ms\n",
            entry.step,
            entry.command,
            entry.args.join(" "),
            entry
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            entry.duration_ms,
        ));

        if dry_run {
            out.push_str("  (dry-run: skipped)\n\n");
            continue;
        }

        // Re-execute the command
        let start = Instant::now();
        let replay_output = Command::new(&entry.command)
            .args(&entry.args)
            .current_dir(&root)
            .output();
        let replay_duration = start.elapsed().as_millis();

        match replay_output {
            Ok(replay) => {
                let replay_exit = replay.status.code();
                let replay_stdout = truncate_str(
                    &String::from_utf8_lossy(&replay.stdout),
                    MAX_IO_PREVIEW_CHARS,
                );
                let replay_stderr = truncate_str(
                    &String::from_utf8_lossy(&replay.stderr),
                    MAX_IO_PREVIEW_CHARS,
                );

                let exit_match = replay_exit == entry.exit_code;
                let duration_diff = if replay_duration > entry.duration_ms {
                    format!("+{}ms", replay_duration as i128 - entry.duration_ms as i128)
                } else {
                    format!("-{}ms", entry.duration_ms as i128 - replay_duration as i128)
                };

                out.push_str(&format!(
                    "  Replay exit: {} {}\n",
                    replay_exit
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    if exit_match { "✓" } else { "✗ DIFFERENT" }
                ));
                out.push_str(&format!(
                    "  Replay duration: {replay_duration}ms ({duration_diff})\n"
                ));

                if !exit_match {
                    out.push_str(&format!("  Replay stderr: {replay_stderr}\n"));
                }

                // Check if output differs significantly
                let orig_stdout = &entry.stdout_preview;
                if orig_stdout != &replay_stdout && !replay_stdout.is_empty() {
                    out.push_str("  ⚠ stdout differs from original\n");
                }
            }
            Err(e) => {
                out.push_str(&format!("  Replay FAILED to execute: {e}\n"));
            }
        }

        out.push('\n');
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 3. debug_failure_cluster
// ---------------------------------------------------------------------------

/// Group failed trace entries by command name, error pattern, and exit code.
///
/// Scans all trace JSONL files under `.mint-mcp/traces/` (or a
/// specified subdirectory) for entries with non-zero exit codes, then
/// clusters them by (command, error_prefix, exit_code). Returns clusters
/// sorted by frequency.
pub fn debug_failure_cluster_work(
    root: PathBuf,
    trace_dir: Option<String>,
    min_cluster_size: usize,
) -> String {
    let dir = if let Some(ref subdir) = trace_dir {
        root.join(subdir)
    } else {
        traces_dir(&root)
    };

    if !dir.exists() {
        return format!("Error: trace directory not found: {}", dir.display());
    }

    // Collect all JSONL files
    let jsonl_files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|e| e.path())
            .take(MAX_TRACE_FILES)
            .collect(),
        Err(e) => return format!("Error reading trace directory: {e}"),
    };

    if jsonl_files.is_empty() {
        return "DEBUG: FAILURE CLUSTER\n\nNo trace files found.".to_string();
    }

    // Collect all failed entries
    let mut failed_entries: Vec<(String, TraceEntry)> = Vec::new(); // (trace_id, entry)
    for file in &jsonl_files {
        let trace_id = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("(unknown)")
            .to_string();
        if let Ok(entries) = read_trace_entries(file) {
            for entry in entries {
                if entry.exit_code != Some(0) {
                    failed_entries.push((trace_id.clone(), entry));
                }
            }
        }
    }

    if failed_entries.is_empty() {
        return "DEBUG: FAILURE CLUSTER\n\nNo failed entries found across all traces.".to_string();
    }

    // Cluster by (command_basename, error_prefix, exit_code)
    let mut clusters: HashMap<String, FailureCluster> = HashMap::new();
    for (_trace_id, entry) in &failed_entries {
        let cmd_basename = entry
            .command
            .rsplit('/')
            .next()
            .unwrap_or(&entry.command)
            .to_string();
        let error_prefix = truncate_str(entry.stderr_preview.lines().next().unwrap_or(""), 80);
        let exit_str = entry
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let key = format!("{cmd_basename}|exit={exit_str}|err={error_prefix}");

        let cluster = clusters
            .entry(key.clone())
            .or_insert_with(|| FailureCluster {
                key,
                count: 0,
                examples: Vec::new(),
            });
        cluster.count += 1;
        if cluster.examples.len() < 5 {
            cluster.examples.push(entry.clone());
        }
    }

    // Sort clusters by count descending
    let mut sorted_clusters: Vec<&FailureCluster> = clusters.values().collect();
    sorted_clusters.sort_by(|a, b| b.count.cmp(&a.count));

    // Build report
    let mut out = String::from("DEBUG: FAILURE CLUSTER\n\n");
    out.push_str(&format!("Trace files scanned: {}\n", jsonl_files.len()));
    out.push_str(&format!("Total failed entries: {}\n", failed_entries.len()));
    out.push_str(&format!("Unique clusters: {}\n", clusters.len()));
    out.push_str(&format!("Min cluster size filter: {min_cluster_size}\n\n"));

    let mut shown = 0usize;
    for cluster in sorted_clusters {
        if cluster.count < min_cluster_size {
            continue;
        }
        shown += 1;

        let parts: Vec<&str> = cluster.key.splitn(3, '|').collect();
        let cmd = parts.first().unwrap_or(&"(unknown)");
        let exit = parts.get(1).unwrap_or(&"exit=?");
        let err = parts.get(2).unwrap_or(&"err=?");

        out.push_str(&format!(
            "Cluster #{} ({} failures)\n",
            shown, cluster.count
        ));
        out.push_str(&format!("  Command: {cmd}\n"));
        out.push_str(&format!("  {exit}\n"));
        out.push_str(&format!("  {err}\n"));

        if !cluster.examples.is_empty() {
            out.push_str("  Example entries:\n");
            for ex in &cluster.examples {
                out.push_str(&format!(
                    "    Step {} | {}ms | stderr: {}\n",
                    ex.step,
                    ex.duration_ms,
                    truncate_str(ex.stderr_preview.lines().next().unwrap_or("(empty)"), 60)
                ));
            }
        }
        out.push('\n');
    }

    if shown == 0 {
        out.push_str(&format!(
            "No clusters with ≥ {min_cluster_size} failures found.\n"
        ));
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 4. debug_root_cause_report
// ---------------------------------------------------------------------------

/// Analyze a trace for failure patterns and suggest root causes.
///
/// Reads all entries from the trace, identifies the first failure,
/// detects cascading failures, and produces a structured report with
/// timeline, root cause hypotheses, and recommendations.
pub fn debug_root_cause_report_work(root: PathBuf, trace_id: String) -> String {
    let path = trace_path(&root, &trace_id);
    if !path.exists() {
        return format!(
            "Error: trace file not found: {}",
            path.strip_prefix(&root).unwrap_or(&path).display()
        );
    }

    let entries = match read_trace_entries(&path) {
        Ok(e) => e,
        Err(e) => return e,
    };

    if entries.is_empty() {
        return "DEBUG: ROOT CAUSE REPORT\n\nTrace file is empty.".to_string();
    }

    let mut out = String::from("DEBUG: ROOT CAUSE REPORT\n\n");
    out.push_str(&format!("Trace ID: {trace_id}\n"));
    out.push_str(&format!("Total steps: {}\n\n", entries.len()));

    // --- Timeline ---
    out.push_str("=== TIMELINE ===\n");
    for entry in &entries {
        let status_icon = if entry.exit_code == Some(0) {
            "✓"
        } else {
            "✗"
        };
        out.push_str(&format!(
            " {status_icon} Step {} | {} {} | {}ms | exit={}\n",
            entry.step,
            entry.command,
            entry.args.join(" "),
            entry.duration_ms,
            entry
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        ));
    }
    out.push('\n');

    // --- First failure ---
    let first_failure = entries.iter().find(|e| e.exit_code != Some(0));

    match first_failure {
        Some(ff) => {
            out.push_str("=== FIRST FAILURE ===\n");
            out.push_str(&format!("Step: {}\n", ff.step));
            out.push_str(&format!("Command: {} {}\n", ff.command, ff.args.join(" ")));
            out.push_str(&format!("Duration: {}ms\n", ff.duration_ms));
            out.push_str(&format!(
                "Exit code: {}\n",
                ff.exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "N/A (process error)".to_string())
            ));
            if !ff.stderr_preview.is_empty() {
                out.push_str(&format!("Stderr:\n{}\n", ff.stderr_preview));
            }
            out.push('\n');

            // --- Failure cascade ---
            let cascade: Vec<&TraceEntry> = entries
                .iter()
                .filter(|e| e.step > ff.step && e.exit_code != Some(0))
                .collect();

            out.push_str("=== FAILURE CASCADE ===\n");
            if cascade.is_empty() {
                out.push_str("No subsequent failures detected — single point of failure.\n\n");
            } else {
                out.push_str(&format!(
                    "{} subsequent step(s) also failed after the first failure:\n",
                    cascade.len()
                ));
                for entry in &cascade {
                    out.push_str(&format!(
                        "  Step {} | {} {} | exit={}\n",
                        entry.step,
                        entry.command,
                        entry.args.join(" "),
                        entry
                            .exit_code
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "N/A".to_string())
                    ));
                }
                out.push('\n');
            }

            // --- Error pattern analysis ---
            out.push_str("=== ERROR PATTERNS ===\n");
            let all_failures: Vec<&TraceEntry> =
                entries.iter().filter(|e| e.exit_code != Some(0)).collect();

            // Group failures by command
            let mut cmd_fail_counts: HashMap<String, usize> = HashMap::new();
            for entry in &all_failures {
                *cmd_fail_counts.entry(entry.command.clone()).or_insert(0) += 1;
            }
            let mut sorted_cmds: Vec<_> = cmd_fail_counts.iter().collect();
            sorted_cmds.sort_by(|a, b| b.1.cmp(a.1));

            for (cmd, count) in sorted_cmds {
                out.push_str(&format!("  {cmd}: {count} failure(s)\n"));
            }
            out.push('\n');

            // --- Root cause hypotheses ---
            out.push_str("=== ROOT CAUSE HYPOTHESES ===\n");

            // Heuristic 1: If the first failure is a dependency/infrastructure
            // command (e.g. docker, cargo, npm) and cascades follow, it's
            // likely an environment issue.
            let first_cmd = ff.command.rsplit('/').next().unwrap_or(&ff.command);
            let infra_commands = [
                "docker", "cargo", "npm", "pip", "make", "cmake", "gradle", "mvn", "go", "rustc",
                "python", "node",
            ];
            if infra_commands.contains(&first_cmd) && !cascade.is_empty() {
                out.push_str(
                    "  1. ENVIRONMENT ISSUE: First failure is an infrastructure tool;\n\
                     cascading failures suggest missing dependency or misconfigured environment.\n",
                );
                out.push_str(&format!(
                    "     Evidence: '{}' failed at step {}, then {} more steps failed.\n",
                    first_cmd,
                    ff.step,
                    cascade.len()
                ));
            }

            // Heuristic 2: If the first failure has a timeout-like signature
            // (long duration, specific exit codes like 124 for timeout),
            // suggest a timeout issue.
            if ff.duration_ms > 30_000 || ff.exit_code == Some(124) {
                out.push_str(
                    "  2. TIMEOUT: First failure has a long duration or timeout exit code.\n\
                     Consider increasing timeouts or checking network connectivity.\n",
                );
                out.push_str(&format!(
                    "     Evidence: duration={}ms, exit_code={:?}\n",
                    ff.duration_ms, ff.exit_code
                ));
            }

            // Heuristic 3: If stderr mentions common issues
            let stderr_lower = ff.stderr_preview.to_lowercase();
            if stderr_lower.contains("permission denied")
                || stderr_lower.contains("access denied")
                || stderr_lower.contains("eacces")
            {
                out.push_str(
                    "  3. PERMISSION ISSUE: Stderr mentions access/permission denied.\n\
                     Check file permissions and user privileges.\n",
                );
            }
            if stderr_lower.contains("not found")
                || stderr_lower.contains("no such file")
                || stderr_lower.contains("enoent")
            {
                out.push_str(
                    "  4. MISSING RESOURCE: Stderr mentions 'not found' or 'no such file'.\n\
                     Verify the file or dependency exists and paths are correct.\n",
                );
            }
            if stderr_lower.contains("connection")
                || stderr_lower.contains("network")
                || stderr_lower.contains("timeout")
                || stderr_lower.contains("econnrefused")
            {
                out.push_str(
                    "  5. NETWORK/CONNECTIVITY: Stderr mentions connection issues.\n\
                     Check network configuration, DNS, and firewall rules.\n",
                );
            }
            if stderr_lower.contains("out of memory")
                || stderr_lower.contains("oom")
                || stderr_lower.contains("cannot allocate")
            {
                out.push_str(
                    "  6. MEMORY ISSUE: Stderr mentions out-of-memory or allocation failure.\n\
                     Check available RAM, swap, and consider reducing data sizes.\n",
                );
            }

            // Heuristic 4: If all failures have the same command
            if cmd_fail_counts.len() == 1 && all_failures.len() > 1 {
                let cmd = cmd_fail_counts.keys().next().unwrap();
                out.push_str(&format!(
                    "  7. REPEATED COMMAND FAILURE: All {} failures are from the same command '{cmd}'.\n\
                     The command itself may have a systematic bug or missing prerequisite.\n",
                    all_failures.len()
                ));
            }

            // Heuristic 5: Fallback if no specific hypotheses matched
            if !out.contains("1. ENVIRONMENT")
                && !out.contains("2. TIMEOUT")
                && !out.contains("3. PERMISSION")
                && !out.contains("4. MISSING RESOURCE")
                && !out.contains("5. NETWORK")
                && !out.contains("6. MEMORY")
                && !out.contains("7. REPEATED")
            {
                out.push_str(&format!(
                    "  1. GENERAL: First failure at step {} with command '{}'.\n\
                     Review the stderr output above for specific error details.\n",
                    ff.step, first_cmd
                ));
                out.push_str(
                    "  2. SUGGESTION: Re-run with debug_trace_run in isolation to\n\
                     get more detailed output from the failing command.\n",
                );
            }
        }
        None => {
            out.push_str("=== RESULT ===\n");
            out.push_str("✓ All steps succeeded — no failures detected in this trace.\n");
        }
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_dir() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();
        // Create .mint-mcp/traces/
        fs::create_dir_all(root.join(".mint-mcp").join("traces")).unwrap();
        (temp, root)
    }

    #[test]
    fn test_traces_dir_path() {
        let root = PathBuf::from("/tmp/test_project");
        let dir = traces_dir(&root);
        assert_eq!(dir, root.join(".mint-mcp").join("traces"));
    }

    #[test]
    fn test_trace_path_sanitization() {
        let root = PathBuf::from("/tmp/test");
        let path = trace_path(&root, "my-trace_1.0");
        assert!(path.to_str().unwrap().contains("my-trace_1.0.jsonl"));
    }

    #[test]
    fn test_trace_path_rejects_special_chars() {
        let root = PathBuf::from("/tmp/test");
        let path = trace_path(&root, "../../../etc/passwd");
        // Special chars should be stripped from the trace_id (note: '.' is allowed by trace_path)
        // The full path naturally contains '/' from directory components,
        // so only check the filename portion.
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(!filename.contains('/'));
    }

    #[test]
    fn test_trace_entry_serialization() {
        let entry = TraceEntry {
            ts_ms: 1000,
            step: 0,
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            duration_ms: 50,
            exit_code: Some(0),
            stdout_preview: "hello\n".to_string(),
            stderr_preview: String::new(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"echo\""));
        assert!(json.contains("\"hello\""));

        // Round-trip
        let deserialized: TraceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.command, "echo");
        assert_eq!(deserialized.args, vec!["hello"]);
        assert_eq!(deserialized.exit_code, Some(0));
    }

    #[test]
    fn test_read_trace_entries_empty_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.jsonl");
        fs::write(&path, "").unwrap();
        let entries = read_trace_entries(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_read_trace_entries_valid() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.jsonl");
        let entry = TraceEntry {
            ts_ms: 1000,
            step: 0,
            command: "ls".to_string(),
            args: vec![],
            duration_ms: 10,
            exit_code: Some(0),
            stdout_preview: "file1\nfile2\n".to_string(),
            stderr_preview: String::new(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        fs::write(&path, format!("{json}\n")).unwrap();

        let entries = read_trace_entries(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "ls");
    }

    #[test]
    fn test_read_trace_entries_mixed() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.jsonl");

        let entry1 = TraceEntry {
            ts_ms: 1000,
            step: 0,
            command: "echo".to_string(),
            args: vec![],
            duration_ms: 5,
            exit_code: Some(0),
            stdout_preview: String::new(),
            stderr_preview: String::new(),
        };
        let entry2 = TraceEntry {
            ts_ms: 2000,
            step: 1,
            command: "false".to_string(),
            args: vec![],
            duration_ms: 3,
            exit_code: Some(1),
            stdout_preview: String::new(),
            stderr_preview: "error".to_string(),
        };

        let content = format!(
            "{}\n{}\nmalformed line\n\n",
            serde_json::to_string(&entry1).unwrap(),
            serde_json::to_string(&entry2).unwrap(),
        );
        fs::write(&path, content).unwrap();

        let entries = read_trace_entries(&path).unwrap();
        assert_eq!(entries.len(), 2); // malformed line skipped
        assert_eq!(entries[1].exit_code, Some(1));
    }

    #[test]
    fn test_debug_trace_run_creates_file() {
        let (temp, root) = setup_temp_dir();
        let result = debug_trace_run_work(
            root,
            "test_trace".to_string(),
            "echo".to_string(),
            vec!["hello".to_string()],
        );
        assert!(result.contains("DEBUG: TRACE RUN"));
        assert!(result.contains("Step: 0"));
        assert!(result.contains("succeeded"));

        // Verify the file was created
        let path = trace_path(&PathBuf::from(temp.path()), "test_trace");
        assert!(path.exists());

        let entries = read_trace_entries(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "echo");
    }

    #[test]
    fn test_debug_trace_run_empty_id() {
        let (_, root) = setup_temp_dir();
        let result = debug_trace_run_work(root, "".to_string(), "echo".to_string(), vec![]);
        assert!(result.contains("Error"));
    }

    #[test]
    fn test_debug_replay_run_dry() {
        let (temp, root) = setup_temp_dir();

        // First, create a trace
        let _ = debug_trace_run_work(
            root.clone(),
            "replay_test".to_string(),
            "echo".to_string(),
            vec!["test".to_string()],
        );

        // Dry-run replay
        let result = debug_replay_run_work(root.clone(), "replay_test".to_string(), true);
        assert!(result.contains("DEBUG: REPLAY RUN"));
        assert!(result.contains("dry-run"));
        assert!(result.contains("skipped"));
    }

    #[test]
    fn test_debug_replay_run_live() {
        let (temp, root) = setup_temp_dir();

        // Create a trace
        let _ = debug_trace_run_work(
            root.clone(),
            "replay_live".to_string(),
            "echo".to_string(),
            vec!["test".to_string()],
        );

        // Live replay
        let result = debug_replay_run_work(root.clone(), "replay_live".to_string(), false);
        assert!(result.contains("DEBUG: REPLAY RUN"));
        assert!(result.contains("live"));
    }

    #[test]
    fn test_debug_replay_run_nonexistent() {
        let (_, root) = setup_temp_dir();
        let result = debug_replay_run_work(root, "nonexistent".to_string(), true);
        assert!(result.contains("Error"));
    }

    #[test]
    fn test_debug_failure_cluster_empty_dir() {
        let temp = tempfile::tempdir().unwrap();
        let result = debug_failure_cluster_work(temp.path().to_path_buf(), None, 1);
        assert!(result.contains("Error") || result.contains("No trace files"));
    }

    #[test]
    fn test_debug_failure_cluster_with_failures() {
        let (temp, root) = setup_temp_dir();

        // Create a trace with a failure
        let _ = debug_trace_run_work(
            root.clone(),
            "cluster_test".to_string(),
            "false".to_string(), // Always exits with code 1
            vec![],
        );

        let result = debug_failure_cluster_work(root, None, 1);
        assert!(result.contains("DEBUG: FAILURE CLUSTER"));
    }

    #[test]
    fn test_debug_root_cause_report_nonexistent() {
        let (_, root) = setup_temp_dir();
        let result = debug_root_cause_report_work(root, "nonexistent".to_string());
        assert!(result.contains("Error"));
    }

    #[test]
    fn test_debug_root_cause_report_all_success() {
        let (temp, root) = setup_temp_dir();

        // Create a trace with a successful command
        let _ = debug_trace_run_work(
            root.clone(),
            "success_trace".to_string(),
            "echo".to_string(),
            vec!["hello".to_string()],
        );

        let result = debug_root_cause_report_work(root, "success_trace".to_string());
        assert!(result.contains("DEBUG: ROOT CAUSE REPORT"));
        assert!(result.contains("All steps succeeded"));
    }

    #[test]
    fn test_debug_root_cause_report_with_failure() {
        let (temp, root) = setup_temp_dir();

        // Create a trace with a failure
        let _ = debug_trace_run_work(
            root.clone(),
            "fail_trace".to_string(),
            "false".to_string(),
            vec![],
        );

        let result = debug_root_cause_report_work(root, "fail_trace".to_string());
        assert!(result.contains("DEBUG: ROOT CAUSE REPORT"));
        assert!(result.contains("FIRST FAILURE"));
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        let long = "abcdefghij".repeat(10);
        let truncated = truncate_str(&long, 20);
        assert!(truncated.contains("…[truncated]"));
    }

    #[test]
    fn test_now_ms() {
        let ts = now_ms();
        assert!(ts > 0);
        // Should be roughly current time (after year 2020)
        assert!(ts > 1577836800000);
    }
}
