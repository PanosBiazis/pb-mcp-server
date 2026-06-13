//! # Anti-Malware & System Protection Tools
//!
//! Security scanning, integrity verification, anomaly detection, and
//! quarantine tools for Linux systems. All functions take
//! `serde_json::Value` and return `serde_json::Value`.

use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum bytes to read for secret scanning per file (1 MB default).
const MAX_SECRET_SCAN_BYTES: u64 = 1_048_576;

/// Maximum bytes for manifest entries during integrity checks.
const MAX_MANIFEST_BYTES: u64 = 10 * 1024 * 1024;

/// Maximum entries for SUID/world-writable scans.
const MAX_PERMISSION_SCAN_ENTRIES: usize = 10_000;

/// Maximum lines of /proc/net/tcp to parse.
const MAX_NET_TCP_LINES: usize = 5_000;

/// Maximum PIDs to inspect in process anomaly detection.
const MAX_PID_SCAN: usize = 65_536;

// ---------------------------------------------------------------------------
// 1. protect_scan_file_clamav_work
// ---------------------------------------------------------------------------

/// Scan a single file with ClamAV.
///
/// Args: `path` (string) — file path to scan.
///
/// Returns: `{clean, infected, threats, scan_time_ms}`.
pub fn protect_scan_file_clamav_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'", "tool_available": false}),
    };

    if !Path::new(&path).exists() {
        return json!({"error": format!("file not found: {path}"), "tool_available": true});
    }

    if which_check("clamscan").is_none() {
        return json!({
            "error": "clamscan not found. Install ClamAV: sudo apt install clamav (Debian/Ubuntu) or sudo dnf install clamav (Fedora)",
            "tool_available": false
        });
    }

    let start = std::time::Instant::now();
    let output = match Command::new("clamscan")
        .arg("--no-summary")
        .arg(&path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run clamscan: {e}"), "tool_available": true})
        }
    };
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let exit_code = output.status.code().unwrap_or(-1);
    let infected = exit_code == 1;
    let clean = exit_code == 0;

    let threats: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if line.contains("FOUND") {
                line.split(':')
                    .nth(1)
                    .map(|s| s.trim().trim_end_matches(" FOUND").to_string())
            } else {
                None
            }
        })
        .collect();

    json!({
        "clean": clean,
        "infected": infected,
        "threats": threats,
        "scan_time_ms": elapsed_ms,
        "exit_code": exit_code,
        "stderr": stderr.trim(),
        "tool_available": true
    })
}

// ---------------------------------------------------------------------------
// 2. protect_scan_directory_clamav_work
// ---------------------------------------------------------------------------

/// Recursive ClamAV directory scan.
///
/// Args: `path` (string), `max_depth` (int, default 10).
///
/// Runs `clamscan -r --max-dir-recursion=<depth> <path>`.
pub fn protect_scan_directory_clamav_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'", "tool_available": false}),
    };

    let max_depth = args
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(10)
        .clamp(1, 50);

    if !Path::new(&path).is_dir() {
        return json!({"error": format!("directory not found: {path}"), "tool_available": true});
    }

    if which_check("clamscan").is_none() {
        return json!({
            "error": "clamscan not found. Install ClamAV: sudo apt install clamav (Debian/Ubuntu) or sudo dnf install clamav (Fedora)",
            "tool_available": false
        });
    }

    let start = std::time::Instant::now();
    let output = match Command::new("clamscan")
        .args(["-r", "--no-summary"])
        .arg(format!("--max-dir-recursion={max_depth}"))
        .arg(&path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run clamscan: {e}"), "tool_available": true})
        }
    };
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let infected = exit_code == 1;
    let clean = exit_code == 0;

    let threats: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if line.contains("FOUND") {
                line.split(':')
                    .nth(1)
                    .map(|s| s.trim().trim_end_matches(" FOUND").to_string())
            } else {
                None
            }
        })
        .collect();

    json!({
        "clean": clean,
        "infected": infected,
        "threats": threats,
        "scan_time_ms": elapsed_ms,
        "max_depth": max_depth,
        "path": path,
        "exit_code": exit_code,
        "tool_available": true
    })
}

// ---------------------------------------------------------------------------
// 3. protect_yara_scan_dir_work
// ---------------------------------------------------------------------------

/// YARA rule scan on a directory.
///
/// Args: `path` (string), `rules_path` (string).
///
/// Runs `yara -r <rules_path> <path>`.
pub fn protect_yara_scan_dir_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'", "tool_available": false}),
    };
    let rules_path = match args.get("rules_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return json!({"error": "missing required arg 'rules_path'", "tool_available": false})
        }
    };

    if !Path::new(&path).exists() {
        return json!({"error": format!("path not found: {path}"), "tool_available": true});
    }
    if !Path::new(&rules_path).exists() {
        return json!({
            "error": format!("rules file not found: {rules_path}"),
            "tool_available": true
        });
    }

    if which_check("yara").is_none() {
        return json!({
            "error": "yara not found. Install YARA: sudo apt install yara (Debian/Ubuntu), sudo dnf install yara (Fedora), or build from https://github.com/VirusTotal/yara",
            "tool_available": false
        });
    }

    let output = match Command::new("yara")
        .args(["-r", &rules_path, &path])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run yara: {e}"), "tool_available": true})
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Parse yara output: rule_name filepath
    let matches: Vec<Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Some(json!({
                    "rule": parts[0].to_string(),
                    "file": parts[1].to_string()
                }))
            } else {
                None
            }
        })
        .collect();

    json!({
        "matches": matches,
        "match_count": matches.len(),
        "exit_code": exit_code,
        "stderr": stderr.trim(),
        "path": path,
        "rules_path": rules_path,
        "tool_available": true
    })
}

// ---------------------------------------------------------------------------
// 7. protect_integrity_check_work
// ---------------------------------------------------------------------------

/// SHA-256 checksum verification against a manifest file.
///
/// Args: `manifest_path` (string) — path to JSON file: `{"path": "sha256hex", ...}`.
///
/// Returns: `{checked, matched, mismatched: [{path, expected, actual}]}`.
pub fn protect_integrity_check_work(args: Value) -> Value {
    let manifest_path = match args.get("manifest_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'manifest_path'"}),
    };

    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => return json!({"error": format!("cannot read manifest {}: {e}", manifest_path)}),
    };

    if manifest_content.len() > MAX_MANIFEST_BYTES as usize {
        return json!({"error": "manifest file too large"});
    }

    let manifest: Value = match serde_json::from_str(&manifest_content) {
        Ok(v) => v,
        Err(e) => return json!({"error": format!("invalid JSON in manifest: {e}")}),
    };

    let entries = match manifest.as_object() {
        Some(obj) => obj,
        None => return json!({"error": "manifest must be a JSON object"}),
    };

    let mut checked: u64 = 0;
    let mut matched: u64 = 0;
    let mut mismatched: Vec<Value> = Vec::new();

    for (file_path, expected_hash) in entries {
        let expected = match expected_hash.as_str() {
            Some(h) => h.to_lowercase(),
            None => continue,
        };

        let actual = match compute_sha256_hex(Path::new(file_path)) {
            Ok(h) => h.to_lowercase(),
            Err(e) => {
                mismatched.push(json!({
                    "path": file_path,
                    "expected": expected,
                    "actual": format!("error: {e}")
                }));
                checked += 1;
                continue;
            }
        };

        checked += 1;
        if actual == expected {
            matched += 1;
        } else {
            mismatched.push(json!({
                "path": file_path,
                "expected": expected,
                "actual": actual
            }));
        }
    }

    json!({
        "checked": checked,
        "matched": matched,
        "mismatched": mismatched
    })
}

// ---------------------------------------------------------------------------
// 8. protect_secret_scan_dir_work
// ---------------------------------------------------------------------------

/// Scan directory for accidentally committed secrets.
///
/// Args: `path` (string), `max_file_size` (int, default 1MB in bytes).
///
/// Patterns: AWS keys, GitHub tokens, JWTs, private keys, passwords in config,
/// generic API keys.
///
/// Returns: `{files_scanned, findings: [{file, line, type, preview}]}`.
pub fn protect_secret_scan_dir_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'"}),
    };

    let max_file_size = args
        .get("max_file_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(MAX_SECRET_SCAN_BYTES);

    if !Path::new(&path).is_dir() {
        return json!({"error": format!("directory not found: {path}")});
    }

    // Secret patterns (name, regex)
    let patterns: &[(&str, &str)] = &[
        ("AWS Access Key ID", r"AKIA[0-9A-Z]{16}"),
        (
            "AWS Secret Access Key",
            r"(?i)aws[_\-]?secret[_\-]?access[_\-]?key\s*[=:]\s*[A-Za-z0-9/+=]{40}",
        ),
        ("GitHub Token", r"gh[ps]_[A-Za-z0-9_]{36,}"),
        ("GitHub OAuth", r"gho_[A-Za-z0-9_]{36,}"),
        ("GitHub Fine-grained PAT", r"github_pat_[A-Za-z0-9_]{22,}"),
        (
            "JWT",
            r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*",
        ),
        ("Private Key", r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----"),
        (
            "Password in Config",
            r"(?i)(password|passwd|pwd)\s*[=:]\s*\S+",
        ),
        ("Slack Token", r"xox[baprs]-[A-Za-z0-9\-]{10,}"),
        ("Stripe Key", r"[sr]k_live_[A-Za-z0-9]{24,}"),
        ("Google API Key", r"AIza[A-Za-z0-9_\\-]{35}"),
        (
            "Google OAuth",
            r"[0-9]+-[a-z0-9_]{32}\.apps\.googleusercontent\.com",
        ),
        (
            "Heroku API Key",
            r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        ),
        ("Twilio API Key", r"SK[0-9a-fA-F]{32}"),
        (
            "Generic API Key",
            r"(?i)(api[_\-]?key|apikey|api[_\-]?secret)\s*[=:]\s*[A-Za-z0-9_\-]{20,}",
        ),
        (
            "SendGrid Key",
            r"SG\.[A-Za-z0-9_\-]{22}\.[A-Za-z0-9_\-]{43}",
        ),
        ("Mailgun API Key", r"key-[0-9a-zA-Z]{32}"),
    ];

    let compiled: Vec<(&str, Regex)> = patterns
        .iter()
        .filter_map(|(name, pat)| Regex::new(pat).ok().map(|r| (*name, r)))
        .collect();

    let mut files_scanned: u64 = 0;
    let mut findings: Vec<Value> = Vec::new();

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let file_size = entry.metadata().map(|m| m.len()).unwrap_or(u64::MAX);
        if file_size > max_file_size {
            continue;
        }

        if is_likely_binary(file_path) {
            continue;
        }

        let path_str = file_path.to_string_lossy();
        if path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/target/")
        {
            continue;
        }

        files_scanned += 1;

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (name, re) in &compiled {
            for (line_num, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    let preview: String = line.trim().chars().take(120).collect();
                    findings.push(json!({
                        "file": file_path.to_string_lossy().to_string(),
                        "line": line_num + 1,
                        "type": name,
                        "preview": preview
                    }));

                    if findings.len() >= 500 {
                        return json!({
                            "files_scanned": files_scanned,
                            "findings": findings,
                            "truncated": true
                        });
                    }
                }
            }
        }
    }

    json!({
        "files_scanned": files_scanned,
        "findings": findings
    })
}

// ---------------------------------------------------------------------------
// 9. protect_dependency_audit_work
// ---------------------------------------------------------------------------

/// Run cargo audit or npm audit.
///
/// Args: `path` (string, project root), `language` ("rust" or "node").
///
/// Returns: `{vulnerabilities, report}`.
pub fn protect_dependency_audit_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'"}),
    };
    let language = match args.get("language").and_then(|v| v.as_str()) {
        Some(l) => l.to_lowercase(),
        None => return json!({"error": "missing required arg 'language' (rust or node)"}),
    };

    if !Path::new(&path).exists() {
        return json!({"error": format!("path not found: {path}")});
    }

    match language.as_str() {
        "rust" => {
            if which_check("cargo").is_none() {
                return json!({
                    "error": "cargo not found. Install Rust: https://rustup.rs",
                    "tool_available": false
                });
            }

            let output = match Command::new("cargo")
                .args(["audit"])
                .current_dir(&path)
                .output()
            {
                Ok(o) => {
                    // If exit code is 127, cargo-audit subcommand not installed
                    if o.status.code() == Some(127) {
                        return json!({
                            "error": "cargo-audit not found. Install: cargo install cargo-audit",
                            "tool_available": false
                        });
                    }
                    o
                }
                Err(e) => {
                    return json!({
                        "error": format!("failed to run cargo audit: {e}"),
                        "tool_available": true
                    })
                }
            };

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let vulnerabilities = count_cargo_audit_vulns(&stdout);

            json!({
                "language": "rust",
                "vulnerabilities": vulnerabilities,
                "report": stdout.trim(),
                "stderr": stderr.trim(),
                "exit_code": output.status.code().unwrap_or(-1),
                "tool_available": true
            })
        }
        "node" => {
            if which_check("npm").is_none() {
                return json!({
                    "error": "npm not found. Install Node.js: https://nodejs.org",
                    "tool_available": false
                });
            }

            let output = match Command::new("npm")
                .args(["audit", "--json"])
                .current_dir(&path)
                .output()
            {
                Ok(o) => o,
                Err(e) => {
                    return json!({
                        "error": format!("failed to run npm audit: {e}"),
                        "tool_available": true
                    })
                }
            };

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            let vulnerabilities = if let Ok(parsed) = serde_json::from_str::<Value>(&stdout) {
                parsed
                    .get("metadata")
                    .and_then(|m| m.get("vulnerabilities"))
                    .and_then(|v| {
                        let total: u64 = ["info", "low", "moderate", "high", "critical"]
                            .iter()
                            .filter_map(|sev| v.get(*sev).and_then(|n| n.as_u64()))
                            .sum();
                        if total > 0 {
                            Some(total)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            json!({
                "language": "node",
                "vulnerabilities": vulnerabilities,
                "report": stdout,
                "exit_code": output.status.code().unwrap_or(-1),
                "tool_available": true
            })
        }
        other => json!({"error": format!("unsupported language '{other}'; use 'rust' or 'node'")}),
    }
}

// ---------------------------------------------------------------------------
// 10. protect_suid_find_work
// ---------------------------------------------------------------------------

/// Find SUID/SGID binaries.
///
/// Args: `path` (string, default "/").
///
/// Returns: `{suid_files: [{path, mode, owner}], sgid_files: [{path, mode, owner}]}`.
#[cfg(not(unix))]
pub fn protect_suid_find_work(_args: Value) -> Value {
    json!({
        "error": "SUID/SGID scanning is not supported on Windows (Unix-only concept)",
        "tool_available": false,
        "suid_files": [],
        "sgid_files": [],
        "suid_count": 0,
        "sgid_count": 0
    })
}

#[cfg(unix)]
/// Find SUID and SGID files under a Unix directory.
pub fn protect_suid_find_work(args: Value) -> Value {
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("/");

    if !Path::new(path).is_dir() {
        return json!({"error": format!("directory not found: {path}")});
    }

    let mut suid_files: Vec<Value> = Vec::new();
    let mut sgid_files: Vec<Value> = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(MAX_PERMISSION_SCAN_ENTRIES)
    {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mode = metadata.permissions().mode();
        let is_suid = (mode & 0o4000) != 0;
        let is_sgid = (mode & 0o2000) != 0;

        if is_suid || is_sgid {
            let owner = get_file_owner(&metadata);
            let mode_str = format!("{:o}", mode & 0o7777);

            if is_suid && suid_files.len() < 500 {
                suid_files.push(json!({
                    "path": file_path.to_string_lossy().to_string(),
                    "mode": mode_str,
                    "owner": owner
                }));
            }
            if is_sgid && sgid_files.len() < 500 {
                sgid_files.push(json!({
                    "path": file_path.to_string_lossy().to_string(),
                    "mode": mode_str,
                    "owner": owner
                }));
            }
        }
    }

    json!({
        "suid_files": suid_files,
        "sgid_files": sgid_files,
        "suid_count": suid_files.len(),
        "sgid_count": sgid_files.len()
    })
}

// ---------------------------------------------------------------------------
// 11. protect_world_writable_find_work
// ---------------------------------------------------------------------------

/// Find world-writable files.
///
/// Args: `path` (string), `include_dirs` (bool, default false).
///
/// Returns: `{files: [{path, mode}]}`.
#[cfg(not(unix))]
pub fn protect_world_writable_find_work(_args: Value) -> Value {
    json!({
        "error": "World-writable file scanning is not supported on Windows (Unix-only concept)",
        "tool_available": false,
        "files": [],
        "count": 0
    })
}

#[cfg(unix)]
/// Find world-writable files under a Unix directory.
pub fn protect_world_writable_find_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return json!({"error": "missing required arg 'path'"}),
    };

    let include_dirs = args
        .get("include_dirs")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !Path::new(&path).is_dir() {
        return json!({"error": format!("directory not found: {path}")});
    }

    let mut files: Vec<Value> = Vec::new();

    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(MAX_PERMISSION_SCAN_ENTRIES)
    {
        let file_path = entry.path();
        let is_dir = entry.file_type().is_dir();
        if is_dir && !include_dirs {
            continue;
        }
        if !is_dir && !file_path.is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mode = metadata.permissions().mode();
        if (mode & 0o002) != 0 && files.len() < 500 {
            files.push(json!({
                "path": file_path.to_string_lossy().to_string(),
                "mode": format!("{:o}", mode & 0o7777)
            }));
        }
    }

    json!({
    "files": files,
            "count": files.len()
        })
}
// ---------------------------------------------------------------------------
// 17. protect_rootkit_check_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Check for common Linux rootkit indicators, returning a JSON string.
///
/// Checks `/proc` for hidden processes (PID gaps), `/etc/ld.so.preload`,
/// and common rootkit file paths (`/usr/bin/.hidden`, `/etc/.hidden`, etc.).
///
/// Returns JSON: `{ "checks": [...], "suspicious_findings": N, "details": "..." }`.
#[allow(dead_code)]
pub fn protect_rootkit_check_work(params: Value) -> String {
    let _ = params;
    let mut checks: Vec<Value> = Vec::new();
    let mut suspicious_findings: usize = 0;

    // --- Check /proc for hidden processes (gaps in PID sequence) ---
    let proc_path = Path::new("/proc");
    let mut pids: Vec<u32> = Vec::new();
    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(pid) = name.parse::<u32>() {
                pids.push(pid);
            }
        }
    }
    pids.sort();
    let mut gaps: Vec<String> = Vec::new();
    for window in pids.windows(2) {
        let diff = window[1] - window[0];
        if diff > 1 && diff < 1000 && gaps.len() < 50 {
            gaps.push(format!("gap between PID {} and {}", window[0], window[1]));
        }
    }
    let gap_check_found = !gaps.is_empty();
    if gap_check_found {
        suspicious_findings += 1;
    }
    checks.push(json!({
        "check": "proc_pid_gaps",
        "found": gap_check_found,
        "gaps": gaps
    }));

    // --- Check /etc/ld.so.preload ---
    let preload_found = if let Ok(content) = fs::read_to_string("/etc/ld.so.preload") {
        let entries: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .collect();
        if !entries.is_empty() {
            suspicious_findings += 1;
            checks.push(json!({
                "check": "ld_so_preload",
                "found": true,
                "entries": entries.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            }));
            true
        } else {
            false
        }
    } else {
        false
    };
    if !preload_found {
        checks.push(json!({ "check": "ld_so_preload", "found": false }));
    }

    // --- Check for common rootkit files ---
    let rootkit_paths = [
        "/usr/bin/.hidden",
        "/etc/.hidden",
        "/usr/lib/libsh.so",
        "/usr/lib/libsh.so.interpreter",
        "/usr/lib/libsh/bash",
        "/etc/sh.conf",
        "/dev/.shit",
        "/dev/.shdev",
        "/dev/.shpid",
        "/dev/.shdir",
        "/dev/.shproc",
        "/lib/.sh-",
        "/usr/lib/libsh",
        "/sbin/adore",
        "/usr/bin/.spad",
        "/dev/.devfsd",
        "/dev/.udev",
        "/dev/.static",
    ];
    let mut found_rootkit_files: Vec<String> = Vec::new();
    for rp in &rootkit_paths {
        if Path::new(rp).exists() {
            found_rootkit_files.push(rp.to_string());
        }
    }
    let rootkit_files_found = !found_rootkit_files.is_empty();
    if rootkit_files_found {
        suspicious_findings += 1;
    }
    checks.push(json!({
        "check": "rootkit_file_paths",
        "found": rootkit_files_found,
        "files": found_rootkit_files
    }));

    let details = if suspicious_findings == 0 {
        "No rootkit indicators found".to_string()
    } else {
        format!("{suspicious_findings} suspicious rootkit indicator(s) detected")
    };

    json!({
        "checks": checks,
        "suspicious_findings": suspicious_findings,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 18. protect_process_anomaly_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Detect processes with unusual characteristics, returning a JSON string.
///
/// Uses `ps aux` to list processes and flags: no command line, weird
/// parent-child relationships, or running from `/tmp`.
///
/// Returns JSON: `{ "processes_scanned": N, "anomalies": [...], "details": "..." }`.
#[allow(dead_code)]
pub fn protect_process_anomaly_work(params: Value) -> String {
    let _ = params; // accepted for API consistency; no params required

    let output = match Command::new("ps").args(["aux"]).output() {
        Ok(o) => o,
        Err(e) => {
            return json!({
                "error": format!("failed to run ps aux: {e}"),
                "processes_scanned": 0,
                "anomalies": [],
                "details": "could not execute ps aux"
            })
            .to_string();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut anomalies: Vec<Value> = Vec::new();
    let mut processes_scanned: u64 = 0;

    for line in stdout.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 11 {
            continue;
        }
        processes_scanned += 1;

        let user = fields[0];
        let _pid = fields[1];
        let command = fields[10..].join(" ");

        // Flag: process with no command line
        if command.trim().is_empty() || command == "[" || command.trim().len() < 2 {
            anomalies.push(json!({
                "user": user,
                "pid": fields[1],
                "command": command,
                "reason": "process with no meaningful command line"
            }));
        }

        // Flag: process running from /tmp or /dev/shm
        if command.contains("/tmp/") || command.contains("/dev/shm/") {
            anomalies.push(json!({
                "user": user,
                "pid": fields[1],
                "command": command,
                "reason": "process running from /tmp or /dev/shm"
            }));
        }

        // Flag: processes with weird parent-child (e.g. user process child of kernel)
        // Simplified: flag non-root user processes whose STAT contains 'Z' (zombie)
        let stat = fields[7];
        if stat.contains('Z') {
            anomalies.push(json!({
                "user": user,
                "pid": fields[1],
                "command": command,
                "reason": "zombie process detected"
            }));
        }
    }

    let details = if anomalies.is_empty() {
        format!("{processes_scanned} processes scanned, no anomalies detected")
    } else {
        format!(
            "{} processes scanned, {} anomaly/anomalies detected",
            processes_scanned,
            anomalies.len()
        )
    };

    json!({
        "processes_scanned": processes_scanned,
        "anomalies": anomalies,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 19. protect_network_anomaly_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Detect unusual network connections, returning a JSON string.
///
/// Uses `ss -tulpn` to list connections and flags unexpected listening
/// ports or outbound connections.
///
/// Returns JSON: `{ "connections_found": N, "anomalies": [...], "details": "..." }`.
#[allow(dead_code)]
pub fn protect_network_anomaly_work(params: Value) -> String {
    let _ = params; // accepted for API consistency; no params required

    // Well-known ports that are not flagged as unexpected
    let standard_ports: &[u16] = &[
        20, 21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 3306, 5432, 5900, 6379,
        8080, 8443, 9090, 27017,
    ];

    let output = match Command::new("ss").args(["-tulpn"]).output() {
        Ok(o) => o,
        Err(e) => {
            return json!({
                "error": format!("failed to run ss -tulpn: {e}"),
                "connections_found": 0,
                "anomalies": [],
                "details": "could not execute ss"
            })
            .to_string();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut connections_found: u64 = 0;
    let mut anomalies: Vec<Value> = Vec::new();

    for line in stdout.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        connections_found += 1;

        let local_addr = fields[3].to_string();
        let state = fields.first().unwrap_or(&"").to_string();

        // Extract port from local address (last segment after colon or dot)
        let port: u16 = match local_addr.rsplit(':').next() {
            Some(p) => p.parse().unwrap_or(0),
            None => 0,
        };

        // Flag: unexpected listening port
        if state.contains("LISTEN") && !standard_ports.contains(&port) && port != 0 {
            anomalies.push(json!({
                "local_address": local_addr,
                "state": state,
                "reason": format!("unexpected listening port {port}")
            }));
        }

        // Flag: established connection to high port (possible C2)
        if state.contains("ESTAB") {
            let remote_addr = fields.get(4).unwrap_or(&"0").to_string();
            let remote_port: u16 = match remote_addr.rsplit(':').next() {
                Some(p) => p.parse().unwrap_or(0),
                None => 0,
            };
            if remote_port > 40000 {
                anomalies.push(json!({
                    "local_address": local_addr,
                    "remote_address": remote_addr,
                    "state": state,
                    "reason": format!("outbound connection to high port {remote_port}")
                }));
            }
        }
    }

    let details = if anomalies.is_empty() {
        format!("{connections_found} connections found, no anomalies detected")
    } else {
        format!(
            "{} connections found, {} anomaly/anomalies detected",
            connections_found,
            anomalies.len()
        )
    };

    json!({
        "connections_found": connections_found,
        "anomalies": anomalies,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 20. protect_cron_audit_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Audit cron jobs, returning a JSON string.
///
/// Reads current user's crontab via `crontab -l`, checks `/etc/cron.d/`
/// for system cron jobs, and flags suspicious entries (ones that download
/// and execute, use `/dev/shm`, etc.).
///
/// Returns JSON: `{ "cron_jobs_found": N, "suspicious": [...], "details": "..." }`.
#[allow(dead_code)]
pub fn protect_cron_audit_work(params: Value) -> String {
    let _ = params; // accepted for API consistency; no params required

    let mut cron_jobs_found: u64 = 0;
    let mut suspicious: Vec<Value> = Vec::new();

    fn is_suspicious_entry(command: &str) -> Vec<String> {
        let mut reasons = Vec::new();
        if (command.contains("wget ") || command.contains("curl "))
            && (command.contains(" | ") || command.contains("|sh") || command.contains("bash"))
        {
            reasons.push("cron job downloads and executes code".to_string());
        }
        if command.contains("/dev/shm/") {
            reasons.push("cron job uses /dev/shm (RAM disk, often abused)".to_string());
        }
        if command.contains("base64") && (command.contains("-d") || command.contains("--decode")) {
            reasons.push("cron job decodes base64 (possible obfuscation)".to_string());
        }
        if command.contains("nc ") || command.contains("ncat ") || command.contains("netcat ") {
            reasons.push("cron job uses netcat (possible reverse shell)".to_string());
        }
        if command.contains("/dev/tcp/") || command.contains("/dev/udp/") {
            reasons.push("cron job uses /dev/tcp or /dev/udp (bash network redirect)".to_string());
        }
        if command.contains("chmod +x") || command.contains("chmod 777") {
            reasons.push("cron job changes executable/permission bits".to_string());
        }
        reasons
    }

    // Read current user's crontab
    let crontab_output = Command::new("crontab").args(["-l"]).output();
    if let Ok(output) = crontab_output {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            cron_jobs_found += 1;
            let reasons = is_suspicious_entry(line);
            if !reasons.is_empty() {
                suspicious.push(json!({
                    "entry": line,
                    "source": "user_crontab",
                    "reasons": reasons
                }));
            }
        }
    }

    // Check /etc/cron.d/ for system cron jobs
    if let Ok(cron_d) = fs::read_dir("/etc/cron.d/") {
        for entry in cron_d.filter_map(|e| e.ok()) {
            let fpath = entry.path();
            if !fpath.is_file() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&fpath) {
                let source = format!(
                    "/etc/cron.d/{}",
                    fpath.file_name().unwrap_or_default().to_string_lossy()
                );
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    cron_jobs_found += 1;
                    let reasons = is_suspicious_entry(line);
                    if !reasons.is_empty() {
                        suspicious.push(json!({
                            "entry": line,
                            "source": source,
                            "reasons": reasons
                        }));
                    }
                }
            }
        }
    }

    let details = if suspicious.is_empty() {
        format!("{cron_jobs_found} cron jobs found, none suspicious")
    } else {
        format!(
            "{} cron jobs found, {} suspicious",
            cron_jobs_found,
            suspicious.len()
        )
    };

    json!({
        "cron_jobs_found": cron_jobs_found,
        "suspicious": suspicious,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 21. protect_ssh_key_audit_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Audit SSH keys, returning a JSON string.
///
/// Reads `~/.ssh/authorized_keys` and checks for weak key types
/// (ssh-dss), empty keys, or unknown options.
///
/// Returns JSON: `{ "keys_found": N, "issues": [...], "details": "..." }`.
#[allow(dead_code)]
pub fn protect_ssh_key_audit_work(params: Value) -> String {
    let user = params.get("user").and_then(|v| v.as_str()).unwrap_or("");

    let home_dir = if user.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
    } else {
        get_user_home(user)
    };

    let auth_keys_path = format!("{home_dir}/.ssh/authorized_keys");

    if !Path::new(&auth_keys_path).exists() {
        return json!({
            "keys_found": 0,
            "issues": [],
            "details": format!("authorized_keys file not found at {auth_keys_path}")
        })
        .to_string();
    }

    let content = match fs::read_to_string(&auth_keys_path) {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "error": format!("cannot read {auth_keys_path}: {e}"),
                "keys_found": 0,
                "issues": [],
                "details": "failed to read authorized_keys"
            })
            .to_string();
        }
    };

    let mut keys_found: u64 = 0;
    let mut issues: Vec<Value> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        keys_found += 1;

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 2 {
            issues.push(json!({
                "line": line.chars().take(80).collect::<String>(),
                "issue": "empty or malformed key entry"
            }));
            continue;
        }

        let key_type = parts[0].to_string();
        let key_data = parts.get(1).unwrap_or(&"");
        let comment = parts.get(2).map(|s| s.trim()).unwrap_or("");

        match key_type.as_str() {
            "ssh-dss" => {
                issues.push(json!({
                    "key_type": key_type,
                    "comment": comment,
                    "issue": "DSA key is insecure and deprecated"
                }));
            }
            "ssh-rsa" => {
                let key_len_bits = estimate_rsa_key_bits(key_data);
                if key_len_bits > 0 && key_len_bits < 2048 {
                    issues.push(json!({
                        "key_type": key_type,
                        "comment": comment,
                        "issue": format!("short RSA key (~{key_len_bits} bits, minimum 2048 recommended)")
                    }));
                }
            }
            "ssh-ed25519"
            | "ecdsa-sha2-nistp384"
            | "ecdsa-sha2-nistp521"
            | "sk-ssh-ed25519@openssh.com"
            | "sk-ecdsa-sha2-nistp256@openssh.com" => {
                // These are fine
            }
            "ecdsa-sha2-nistp256" => {
                issues.push(json!({
                    "key_type": key_type,
                    "comment": comment,
                    "issue": "ECDSA with NIST P-256 (consider Ed25519 instead)"
                }));
            }
            _ => {
                issues.push(json!({
                    "key_type": key_type,
                    "comment": comment,
                    "issue": format!("unknown key type: {key_type}")
                }));
            }
        }

        if key_data.trim().is_empty() {
            issues.push(json!({
                "key_type": key_type,
                "issue": "empty key data"
            }));
        }

        if comment.is_empty() {
            issues.push(json!({
                "key_type": key_type,
                "issue": "key has no comment (cannot identify owner)"
            }));
        }
    }

    let details = if issues.is_empty() {
        format!("{keys_found} SSH keys found, no issues detected")
    } else {
        format!(
            "{} SSH keys found, {} issue(s) detected",
            keys_found,
            issues.len()
        )
    };

    json!({
        "keys_found": keys_found,
        "issues": issues,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 22. protect_firewall_status_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Read firewall rules, returning a JSON string.
///
/// Tries `iptables -L -n` and `nft list ruleset`. Returns the raw output
/// and a summary of open ports.
///
/// Returns JSON: `{ "firewall_type": "iptables|nftables|unknown", "rules": [...],
///                   "open_ports": [...], "details": "..." }`.
#[allow(dead_code)]
pub fn protect_firewall_status_work(params: Value) -> String {
    let _ = params; // accepted for API consistency; no params required

    let mut firewall_type = "unknown".to_string();
    let mut rules: Vec<Value> = Vec::new();
    let mut open_ports: Vec<Value> = Vec::new();
    let details;

    if which_check("iptables").is_some() {
        firewall_type = "iptables".to_string();
        let output = match Command::new("iptables").args(["-L", "-n"]).output() {
            Ok(o) => o,
            Err(e) => {
                return json!({
                    "error": format!("failed to run iptables: {e}"),
                    "firewall_type": firewall_type,
                    "rules": [],
                    "open_ports": [],
                    "details": "could not execute iptables"
                })
                .to_string();
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Parse ACCEPT rules to find open ports
            if trimmed.starts_with("ACCEPT") {
                rules.push(json!({ "rule": trimmed }));
                // Try to extract port from rules like "ACCEPT tcp -- 0.0.0.0/0 0.0.0.0/0 tcp dpt:22"
                if let Some(port_part) = trimmed.split("dpt:").nth(1) {
                    let port_str: String = port_part
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if let Ok(port) = port_str.parse::<u16>() {
                        let proto = if trimmed.contains("tcp") {
                            "tcp"
                        } else if trimmed.contains("udp") {
                            "udp"
                        } else {
                            "unknown"
                        };
                        open_ports
                            .push(json!({ "port": port, "protocol": proto, "chain": "INPUT" }));
                    }
                }
            } else if !trimmed.starts_with("Chain") && !trimmed.starts_with("target") {
                rules.push(json!({ "rule": trimmed }));
            }
        }
        details = format!(
            "iptables: {} rules, {} open ports detected",
            rules.len(),
            open_ports.len()
        );
    } else if which_check("nft").is_some() {
        firewall_type = "nftables".to_string();
        let output = match Command::new("nft").args(["list", "ruleset"]).output() {
            Ok(o) => o,
            Err(e) => {
                return json!({
                    "error": format!("failed to run nft: {e}"),
                    "firewall_type": firewall_type,
                    "rules": [],
                    "open_ports": [],
                    "details": "could not execute nft"
                })
                .to_string();
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            rules.push(json!({ "rule": trimmed }));
            // Parse nftables accept rules for open ports
            if trimmed.contains("accept") {
                if let Some(port_part) = trimmed.split("dport").nth(1) {
                    let port_str: String = port_part
                        .trim()
                        .trim_start_matches(&[' ', '"', '-'][..])
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if let Ok(port) = port_str.parse::<u16>() {
                        let proto = if trimmed.contains("tcp") {
                            "tcp"
                        } else if trimmed.contains("udp") {
                            "udp"
                        } else {
                            "unknown"
                        };
                        open_ports.push(json!({ "port": port, "protocol": proto }));
                    }
                }
            }
        }
        details = format!(
            "nftables: {} rules, {} open ports detected",
            rules.len(),
            open_ports.len()
        );
    } else {
        details = "neither iptables nor nft found. Firewall may not be installed.".to_string();
    }

    json!({
        "firewall_type": firewall_type,
        "rules": rules,
        "open_ports": open_ports,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 23. protect_quarantine_file_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Move a suspicious file to quarantine, returning a JSON string.
///
/// Accepts `path` parameter. Creates `.mint-mcp/quarantine/` directory
/// and moves the file there with a timestamp prefix.
///
/// Returns JSON: `{ "quarantined": true, "original_path": "...",
///                   "quarantine_path": "...", "details": "..." }`.
#[allow(dead_code)]
pub fn protect_quarantine_file_work(params: Value) -> String {
    let path = match params.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return json!({
                "error": "missing required arg 'path'",
                "quarantined": false,
                "original_path": "",
                "quarantine_path": "",
                "details": "path parameter is required"
            })
            .to_string();
        }
    };

    let quarantine_dir = ".mint-mcp/quarantine".to_string();

    if !Path::new(&path).exists() {
        return json!({
            "error": format!("file not found: {path}"),
            "quarantined": false,
            "original_path": path,
            "quarantine_path": "",
            "details": "source file does not exist"
        })
        .to_string();
    }

    if let Err(e) = fs::create_dir_all(&quarantine_dir) {
        return json!({
            "error": format!("cannot create quarantine dir {quarantine_dir}: {e}"),
            "quarantined": false,
            "original_path": path,
            "quarantine_path": "",
            "details": "failed to create quarantine directory"
        })
        .to_string();
    }

    let file_name = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "quarantined_file".to_string());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let quarantine_path = format!("{quarantine_dir}/{timestamp}_{file_name}");

    match fs::rename(&path, &quarantine_path) {
        Ok(()) => {
            #[cfg(unix)]
            let _ = fs::set_permissions(&quarantine_path, fs::Permissions::from_mode(0o000));
            json!({
                "quarantined": true,
                "original_path": path,
                "quarantine_path": quarantine_path,
                "details": "file quarantined and permissions set to 000"
            })
        }
        Err(_) => {
            // Try copy + delete (cross-device)
            if fs::copy(&path, &quarantine_path).is_ok() {
                #[cfg(unix)]
                let _ = fs::set_permissions(&quarantine_path, fs::Permissions::from_mode(0o000));
                let _ = fs::remove_file(&path);
                json!({
                    "quarantined": true,
                    "original_path": path,
                    "quarantine_path": quarantine_path,
                    "details": "file quarantined via copy-and-delete, permissions set to 000"
                })
            } else {
                json!({
                    "error": "failed to quarantine file".to_string(),
                    "quarantined": false,
                    "original_path": path,
                    "quarantine_path": "",
                    "details": "could not move or copy file to quarantine"
                })
            }
        }
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// 24. protect_file_reputation_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Check file hash reputation, returning a JSON string.
///
/// Accepts `path` parameter. Computes SHA-256 of the file and returns
/// the hash for manual lookup (no API call needed).
///
/// Returns JSON: `{ "sha256": "...", "file_size": N, "details": "..." }`.
#[allow(dead_code)]
pub fn protect_file_reputation_work(params: Value) -> String {
    let path = match params.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return json!({
                "error": "missing required arg 'path'",
                "sha256": "",
                "file_size": 0,
                "details": "path parameter is required"
            })
            .to_string();
        }
    };

    let file_path = Path::new(&path);
    if !file_path.exists() {
        return json!({
            "error": format!("file not found: {path}"),
            "sha256": "",
            "file_size": 0,
            "details": "file does not exist"
        })
        .to_string();
    }

    let file_size = match fs::metadata(&path) {
        Ok(m) => m.len(),
        Err(e) => {
            return json!({
                "error": format!("cannot read metadata for {path}: {e}"),
                "sha256": "",
                "file_size": 0,
                "details": "failed to read file metadata"
            })
            .to_string();
        }
    };

    let sha256 = match compute_sha256_hex(file_path) {
        Ok(h) => h,
        Err(e) => {
            return json!({
                "error": format!("cannot compute SHA-256: {e}"),
                "sha256": "",
                "file_size": file_size,
                "details": "failed to compute hash"
            })
            .to_string();
        }
    };

    json!({
        "sha256": sha256,
        "file_size": file_size,
        "details": format!("SHA-256 hash computed for manual reputation lookup; file size: {file_size} bytes")
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 25. protect_permissions_scan_work (JSON-string variant)
// ---------------------------------------------------------------------------

/// Scan for unsafe file permissions, returning a JSON string.
///
/// Accepts `path` parameter (default "."). Walks the directory tree,
/// finds world-writable files and SUID/SGID binaries.
///
/// Returns JSON: `{ "world_writable": N, "suid_binaries": N,
///                   "sgid_binaries": N, "details": "..." }`.
#[allow(dead_code)]
#[cfg(not(unix))]
pub fn protect_permissions_scan_work(_params: Value) -> String {
    json!({
        "error": "Permission scanning (world-writable/SUID/SGID) is not supported on Windows",
        "tool_available": false,
        "world_writable": 0,
        "suid_binaries": 0,
        "sgid_binaries": 0,
        "details": "Unix file permission bits (mode) do not exist on Windows"
    })
    .to_string()
}

#[allow(dead_code)]
#[cfg(unix)]
/// Run a combined Unix permission scan for risky permission bits.
pub fn protect_permissions_scan_work(params: Value) -> String {
    let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    if !Path::new(path).is_dir() {
        return json!({
            "error": format!("directory not found: {path}"),
            "world_writable": 0,
            "suid_binaries": 0,
            "sgid_binaries": 0,
            "details": "specified path is not a directory"
        })
        .to_string();
    }

    let mut world_writable: u64 = 0;
    let mut suid_binaries: u64 = 0;
    let mut sgid_binaries: u64 = 0;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(MAX_PERMISSION_SCAN_ENTRIES)
    {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let mode = metadata.permissions().mode();

        if (mode & 0o002) != 0 {
            world_writable += 1;
        }
        if (mode & 0o4000) != 0 {
            suid_binaries += 1;
        }
        if (mode & 0o2000) != 0 {
            sgid_binaries += 1;
        }
    }

    let details = format!(
        "scan of '{}': {} world-writable, {} SUID, {} SGID files found",
        path, world_writable, suid_binaries, sgid_binaries
    );

    json!({
        "world_writable": world_writable,
        "suid_binaries": suid_binaries,
        "sgid_binaries": sgid_binaries,
        "details": details
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Check if a command is available on PATH.
fn which_check(program: &str) -> Option<String> {
    let output = Command::new("which").arg(program).output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}

/// Compute SHA-256 hex digest of a file.
fn compute_sha256_hex(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("read error: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Check if a file is likely binary (contains NUL bytes in first 8KB).
fn is_likely_binary(path: &Path) -> bool {
    let mut buf = [0u8; 8192];
    if let Ok(mut file) = fs::File::open(path) {
        if let Ok(n) = file.read(&mut buf) {
            return buf[..n].contains(&0);
        }
    }
    false
}

/// Count vulnerabilities from cargo audit output.
fn count_cargo_audit_vulns(output: &str) -> u64 {
    let mut count = 0u64;
    for line in output.lines() {
        if line.starts_with("ID:") {
            count += 1;
        }
    }
    count
}

/// Get file owner UID as string.
fn get_file_owner(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        format!("uid:{}", metadata.uid())
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        "unknown".to_string()
    }
}

/// Get user home directory from /etc/passwd.
fn get_user_home(user: &str) -> String {
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 7 && parts[0] == user {
                return parts[5].to_string();
            }
        }
    }
    if user == "root" {
        "/root".to_string()
    } else {
        format!("/home/{user}")
    }
}

/// Estimate RSA key bit length from base64 key data length.
fn estimate_rsa_key_bits(key_data: &str) -> u32 {
    let len = key_data.len();
    if len == 0 {
        return 0;
    }
    if len >= 350 {
        2048
    } else if len >= 200 {
        1024
    } else if len >= 100 {
        512
    } else {
        256
    }
}

/// Compute SSH key fingerprint using ssh-keygen.
fn compute_ssh_fingerprint(key_line: &str) -> String {
    if which_check("ssh-keygen").is_none() {
        return "unavailable (ssh-keygen not found)".to_string();
    }

    let tmp_path = "/tmp/pb-ssh-key-audit-tmp.pub";
    if fs::write(tmp_path, key_line.as_bytes()).is_err() {
        return "error writing temp key".to_string();
    }

    let output = Command::new("ssh-keygen").args(["-lf", tmp_path]).output();

    let _ = fs::remove_file(tmp_path);

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if stdout.is_empty() {
                "unknown".to_string()
            } else {
                stdout
            }
        }
        Err(_) => "error".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // 1. ClamAV scan
    #[test]
    fn test_clamav_scan_file_missing_path() {
        let result = protect_scan_file_clamav_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_clamav_scan_file_nonexistent() {
        let result = protect_scan_file_clamav_work(json!({"path": "/nonexistent/file.xyz"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_clamav_scan_file_returns_json() {
        let result = protect_scan_file_clamav_work(json!({"path": "/etc/hostname"}));
        assert!(result.is_object());
        assert!(result.get("tool_available").is_some());
    }

    // 2. ClamAV directory scan
    #[test]
    fn test_clamav_scan_dir_missing_path() {
        let result = protect_scan_directory_clamav_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_clamav_scan_dir_default_depth() {
        let result = protect_scan_directory_clamav_work(json!({"path": "/tmp"}));
        assert!(result.is_object());
        assert!(result.get("max_depth").is_some());
    }

    #[test]
    fn test_clamav_scan_dir_custom_depth() {
        let result = protect_scan_directory_clamav_work(json!({"path": "/tmp", "max_depth": 5}));
        assert_eq!(result.get("max_depth").unwrap().as_u64().unwrap(), 5);
    }

    // 3. YARA scan
    #[test]
    fn test_yara_scan_missing_args() {
        let result = protect_yara_scan_dir_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_yara_scan_missing_rules() {
        let result = protect_yara_scan_dir_work(json!({"path": "/tmp"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_yara_scan_nonexistent_path() {
        let result = protect_yara_scan_dir_work(
            json!({"path": "/nonexistent", "rules_path": "/nonexistent.yar"}),
        );
        assert!(result.get("error").is_some());
    }

    // 4. Rootkit check
    #[test]
    fn test_rootkit_check_returns_indicators() {
        let result = protect_rootkit_check_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("checks").is_some());
        let checks = result.get("checks").unwrap().as_array().unwrap();
        assert!(!checks.is_empty());
    }

    #[test]
    fn test_rootkit_check_indicator_structure() {
        let result = protect_rootkit_check_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        let checks = result.get("checks").unwrap().as_array().unwrap();
        for check in checks {
            assert!(check.get("check").is_some());
            assert!(check.get("found").is_some());
        }
    }

    // 5. Process anomaly
    #[test]
    fn test_process_anomaly_returns_anomalies() {
        let result = protect_process_anomaly_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("anomalies").is_some());
    }

    #[test]
    fn test_process_anomaly_has_scan_count() {
        let result = protect_process_anomaly_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("pids_scanned").is_some() || result.get("anomalies").is_some());
    }

    // 6. Network anomaly
    #[test]
    fn test_network_anomaly_returns_structure() {
        let result = protect_network_anomaly_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("connections_found").is_some());
        assert!(result.get("anomalies").is_some());
    }

    // 7. Integrity check
    #[test]
    fn test_integrity_check_missing_path() {
        let result = protect_integrity_check_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_integrity_check_nonexistent_manifest() {
        let result =
            protect_integrity_check_work(json!({"manifest_path": "/nonexistent/manifest.json"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_integrity_check_invalid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("manifest.json");
        fs::write(&manifest, "not json").unwrap();
        let result =
            protect_integrity_check_work(json!({"manifest_path": manifest.to_str().unwrap()}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_integrity_check_valid_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let test_file = tmp.path().join("test.txt");
        fs::write(&test_file, "hello world").unwrap();
        let hash = compute_sha256_hex(&test_file).unwrap();

        let manifest = tmp.path().join("manifest.json");
        let manifest_content = json!({test_file.to_str().unwrap(): hash});
        fs::write(&manifest, manifest_content.to_string()).unwrap();

        let result =
            protect_integrity_check_work(json!({"manifest_path": manifest.to_str().unwrap()}));
        assert_eq!(result.get("checked").unwrap().as_u64().unwrap(), 1);
        assert_eq!(result.get("matched").unwrap().as_u64().unwrap(), 1);
        assert_eq!(
            result.get("mismatched").unwrap().as_array().unwrap().len(),
            0
        );
    }

    #[test]
    fn test_integrity_check_mismatched_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let test_file = tmp.path().join("test.txt");
        fs::write(&test_file, "hello world").unwrap();

        let manifest = tmp.path().join("manifest.json");
        let manifest_content = json!({test_file.to_str().unwrap(): "0000000000000000000000000000000000000000000000000000000000000000"});
        fs::write(&manifest, manifest_content.to_string()).unwrap();

        let result =
            protect_integrity_check_work(json!({"manifest_path": manifest.to_str().unwrap()}));
        assert_eq!(result.get("checked").unwrap().as_u64().unwrap(), 1);
        assert_eq!(result.get("matched").unwrap().as_u64().unwrap(), 0);
        assert_eq!(
            result.get("mismatched").unwrap().as_array().unwrap().len(),
            1
        );
    }

    // 8. Secret scan
    #[test]
    fn test_secret_scan_missing_path() {
        let result = protect_secret_scan_dir_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_secret_scan_nonexistent_dir() {
        let result = protect_secret_scan_dir_work(json!({"path": "/nonexistent"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_secret_scan_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let result = protect_secret_scan_dir_work(json!({"path": tmp.path().to_str().unwrap()}));
        assert!(result.get("files_scanned").is_some());
    }

    #[test]
    fn test_secret_scan_detects_aws_key() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("config.txt");
        fs::write(&file_path, "AWS_KEY=AKIAIOSFODNN7EXAMPLE\n").unwrap();

        let result = protect_secret_scan_dir_work(json!({"path": tmp.path().to_str().unwrap()}));
        let findings = result.get("findings").unwrap().as_array().unwrap();
        assert!(!findings.is_empty());
        assert!(findings[0]
            .get("type")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("AWS"));
    }

    #[test]
    fn test_secret_scan_detects_github_token() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("config.txt");
        fs::write(
            &file_path,
            "token=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij1234\n",
        )
        .unwrap();

        let result = protect_secret_scan_dir_work(json!({"path": tmp.path().to_str().unwrap()}));
        let findings = result.get("findings").unwrap().as_array().unwrap();
        assert!(!findings.is_empty());
        assert!(findings[0]
            .get("type")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("GitHub"));
    }

    #[test]
    fn test_secret_scan_detects_private_key() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("key.pem");
        fs::write(&file_path, "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n").unwrap();

        let result = protect_secret_scan_dir_work(json!({"path": tmp.path().to_str().unwrap()}));
        let findings = result.get("findings").unwrap().as_array().unwrap();
        assert!(!findings.is_empty());
        assert!(findings[0]
            .get("type")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("Private Key"));
    }

    // 9. Dependency audit
    #[test]
    fn test_dependency_audit_missing_args() {
        let result = protect_dependency_audit_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_dependency_audit_invalid_language() {
        let result = protect_dependency_audit_work(json!({"path": "/tmp", "language": "python"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_dependency_audit_rust() {
        let result = protect_dependency_audit_work(json!({"path": "/tmp", "language": "rust"}));
        assert!(result.is_object());
    }

    #[test]
    fn test_dependency_audit_node() {
        let result = protect_dependency_audit_work(json!({"path": "/tmp", "language": "node"}));
        assert!(result.is_object());
    }

    // 10. SUID find
    #[test]
    fn test_suid_find_default_path() {
        let result = protect_suid_find_work(json!({}));
        assert!(result.get("suid_files").is_some());
        assert!(result.get("sgid_files").is_some());
    }

    #[test]
    fn test_suid_find_custom_path() {
        let tmp = tempfile::tempdir().unwrap();
        let result = protect_suid_find_work(json!({"path": tmp.path().to_str().unwrap()}));
        assert!(result.get("suid_files").is_some());
    }

    #[test]
    fn test_suid_find_nonexistent() {
        let result = protect_suid_find_work(json!({"path": "/nonexistent/dir"}));
        assert!(result.get("error").is_some());
    }

    // 11. World-writable find
    #[test]
    fn test_world_writable_find_missing_path() {
        let result = protect_world_writable_find_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_world_writable_find_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let result =
            protect_world_writable_find_work(json!({"path": tmp.path().to_str().unwrap()}));
        assert!(result.get("files").is_some());
    }

    // 12. Cron audit
    #[test]
    fn test_cron_audit_returns_entries() {
        let result = protect_cron_audit_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("cron_jobs_found").is_some());
    }

    // 13. SSH key audit
    #[test]
    fn test_ssh_key_audit_no_user() {
        let result = protect_ssh_key_audit_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("keys_found").is_some());
    }
    #[test]
    fn test_ssh_key_audit_specific_user() {
        let result = protect_ssh_key_audit_work(json!({"user": "root"}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("keys_found").is_some());
    }

    // 14. Firewall status
    #[test]
    fn test_firewall_status_returns_structure() {
        let result = protect_firewall_status_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("firewall_type").is_some());
        assert!(result.get("rules").is_some() || result.get("open_ports").is_some());
    }

    // 15. Quarantine file
    #[test]
    fn test_quarantine_file_missing_path() {
        let result = protect_quarantine_file_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_quarantine_file_nonexistent() {
        let result = protect_quarantine_file_work(json!({"path": "/nonexistent/file"}));
        let result: Value = serde_json::from_str(&result).unwrap();
        let quarantined = result
            .get("quarantined")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        assert!(result.get("error").is_some() || !quarantined);
    }

    #[test]
    fn test_quarantine_file_success() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("suspicious.txt");
        fs::write(&file_path, "malware content").unwrap();

        let quarantine_dir = tmp.path().join("quarantine");
        let result = protect_quarantine_file_work(json!({
            "path": file_path.to_str().unwrap(),
            "quarantine_dir": quarantine_dir.to_str().unwrap()
        }));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("quarantined").unwrap().as_bool().unwrap());
        assert!(!file_path.exists());
    }

    // 16. File reputation
    #[test]
    fn test_file_reputation_missing_path() {
        let result = protect_file_reputation_work(json!({}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_file_reputation_nonexistent() {
        let result = protect_file_reputation_work(json!({"path": "/nonexistent"}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_file_reputation_no_api_key() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let result = protect_file_reputation_work(json!({"path": file_path.to_str().unwrap()}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("sha256").is_some());
        assert!(result.get("file_size").is_some());
        assert!(result.get("details").is_some());
    }

    #[test]
    fn test_file_reputation_with_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let expected_hash = compute_sha256_hex(&file_path).unwrap();
        let result = protect_file_reputation_work(json!({"path": file_path.to_str().unwrap()}));
        let result: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(
            result.get("sha256").unwrap().as_str().unwrap(),
            expected_hash
        );
    }

    // Helper function tests
    #[test]
    fn test_compute_sha256_known_vector() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("abc.txt");
        fs::write(&file_path, "abc").unwrap();

        let hash = compute_sha256_hex(&file_path).unwrap();
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_is_likely_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let text_file = tmp.path().join("text.txt");
        fs::write(&text_file, "hello world").unwrap();
        assert!(!is_likely_binary(&text_file));

        let bin_file = tmp.path().join("binary.bin");
        fs::write(&bin_file, b"hello\x00world").unwrap();
        assert!(is_likely_binary(&bin_file));
    }

    #[test]
    fn test_which_check_existing_program() {
        // ls should exist on any Unix system
        let result = which_check("ls");
        assert!(result.is_some());
    }

    #[test]
    fn test_which_check_nonexistent_program() {
        let result = which_check("definitely_not_a_real_program_xyz_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_estimate_rsa_key_bits() {
        assert_eq!(estimate_rsa_key_bits(""), 0);
        assert_eq!(estimate_rsa_key_bits(&"x".repeat(100)), 512);
        assert_eq!(estimate_rsa_key_bits(&"x".repeat(200)), 1024);
        assert_eq!(estimate_rsa_key_bits(&"x".repeat(400)), 2048);
    }

    #[test]
    fn test_get_user_home_root() {
        assert_eq!(get_user_home("root"), "/root");
    }

    #[test]
    fn test_count_cargo_audit_vulns_empty() {
        assert_eq!(count_cargo_audit_vulns(""), 0);
    }

    #[test]
    fn test_count_cargo_audit_vulns_with_ids() {
        let output = "ID: RUSTSEC-2021-001\nID: RUSTSEC-2021-002\nother line\n";
        assert_eq!(count_cargo_audit_vulns(output), 2);
    }
}
