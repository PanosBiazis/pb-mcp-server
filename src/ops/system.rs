//! # System & Environment Tools
//!
//! Process listing, disk usage, network interfaces, and environment variable access.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use serde_json::json;

/// Maximum entries for process list output.
const MAX_PROCESS_ENTRIES: usize = 50;

// ---------------------------------------------------------------------------
// 1. system_process_list
// ---------------------------------------------------------------------------

/// List running processes with PID, CPU%, and memory%.
///
/// Uses `ps aux` on Linux/macOS or `tasklist` on Windows.
/// Returns top processes sorted by CPU usage, capped at max_entries.
pub fn system_process_list_work(max_entries: usize) -> String {
    let max = max_entries.clamp(1, MAX_PROCESS_ENTRIES);

    let output = if cfg!(target_os = "windows") {
        Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
    } else {
        Command::new("ps").args(["aux", "--sort=-%cpu"]).output()
    };

    let mut out = String::from("SYSTEM: PROCESS LIST\n\n");

    match output {
        Ok(proc_output) => {
            let stdout = String::from_utf8_lossy(&proc_output.stdout);
            let lines: Vec<&str> = stdout.lines().take(max).collect();

            if lines.is_empty() {
                out.push_str("No process information available.\n");
            } else {
                out.push_str(&format!("Showing top {} processes:\n\n", lines.len()));
                for line in &lines {
                    out.push_str(&format!("{}\n", line.trim()));
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("Error listing processes: {e}\n"));
            out.push_str("Ensure 'ps' (Linux/macOS) or 'tasklist' (Windows) is available.\n");
        }
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 2. system_disk_usage
// ---------------------------------------------------------------------------

/// Report disk usage for a path or mount point.
///
/// Uses `df -h` for filesystem-level info and `du -sh` for directory-level.
pub fn system_disk_usage_work(path: PathBuf) -> String {
    let mut out = String::from("SYSTEM: DISK USAGE\n\n");

    // Filesystem-level info
    if cfg!(target_os = "windows") {
        let output = Command::new("fsutil")
            .args(["volume", "diskfree"])
            .arg(&path)
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                out.push_str(&format!(
                    "Filesystem info for {}:\n{}\n",
                    path.display(),
                    stdout
                ));
            }
            Err(e) => out.push_str(&format!("Error getting filesystem info: {e}\n")),
        }
    } else {
        let output = Command::new("df")
            .args(["-h", &path.to_string_lossy()])
            .output();
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                out.push_str(&format!("Filesystem:\n{}\n", stdout.trim()));
            }
            Err(e) => out.push_str(&format!("Error running df: {e}\n")),
        }
    }

    out.push('\n');

    // Directory-level info
    if path.is_dir() {
        if cfg!(target_os = "windows") {
            let output = Command::new("dir").arg(&path).output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    out.push_str(&format!("Directory info:\n{}\n", stdout.trim()));
                }
                Err(e) => out.push_str(&format!("Error running dir: {e}\n")),
            }
        } else {
            let output = Command::new("du")
                .args(["-sh", &path.to_string_lossy()])
                .output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    out.push_str(&format!("Directory size:\n{}\n", stdout.trim()));
                }
                Err(e) => out.push_str(&format!("Error running du: {e}\n")),
            }
        }
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 3. system_network_interfaces
// ---------------------------------------------------------------------------

/// List network interfaces with IP addresses.
///
/// Uses `ip addr` on Linux, `ifconfig` on macOS, or `ipconfig` on Windows.
pub fn system_network_interfaces_work() -> String {
    let mut out = String::from("SYSTEM: NETWORK INTERFACES\n\n");

    let output = if cfg!(target_os = "windows") {
        Command::new("ipconfig").output()
    } else if cfg!(target_os = "macos") {
        Command::new("ifconfig").output()
    } else {
        // Try `ip addr` first (Linux), fall back to `ifconfig`
        match Command::new("ip").args(["addr"]).output() {
            Ok(o) if !o.stdout.is_empty() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                out.push_str(&stdout);
                let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
                return out;
            }
            _ => Command::new("ifconfig").output(),
        }
    };

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            out.push_str(&stdout);
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
            out.push_str(
                "Ensure 'ip' (Linux), 'ifconfig' (macOS), or 'ipconfig' (Windows) is available.\n",
            );
        }
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 4. system_env_get
// ---------------------------------------------------------------------------

/// Read a single environment variable by name.
///
/// Refuses to return values for keys or values that look sensitive.
pub fn system_env_get_work(key: String) -> String {
    let mut out = String::from("SYSTEM: ENV GET\n\n");
    out.push_str(&format!("Key: {key}\n"));

    if is_sensitive_env_key(&key) {
        out.push_str("Value: [REDACTED — key name suggests sensitive content]\n");
        out.push_str("This tool refuses to display values for keys containing:\n");
        out.push_str(
            "SECRET, PASSWORD/PASSWD/PASS, TOKEN, KEY, CREDENTIAL, PRIVATE, AUTH, SIGNING, WEBHOOK, COOKIE, SESSION, DATABASE/DB URL, CONNECTION, DSN, BEARER, OAUTH, JWT, PAT, PWD, CERT\n",
        );
        return out;
    }

    match env::var(&key) {
        Ok(value) => {
            let value_len = value.len();
            if looks_sensitive_env_value(&value) {
                out.push_str("Value: [REDACTED — value looks like a secret]\n");
            } else {
                out.push_str(&format!("Value: {}\n", value));
            }
            out.push_str(&format!("Length: {value_len} characters\n"));
        }
        Err(env::VarError::NotPresent) => {
            out.push_str("Value: (not set)\n");
        }
        Err(env::VarError::NotUnicode(_)) => {
            out.push_str("Value: (set but not valid Unicode)\n");
        }
    }

    out
}

fn env_key_tokens_upper(key: &str) -> Vec<String> {
    key.split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_uppercase())
        .collect()
}

fn is_sensitive_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    let sensitive_substrings = [
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "TOKEN",
        "CREDENTIAL",
        "PRIVATE",
        "AUTH",
        "SIGNING",
        "WEBHOOK",
        "COOKIE",
        "SESSION",
        "DATABASE_URL",
        "DB_URL",
        "CONNECTION_STRING",
        "CONN_STRING",
        "DSN",
        "BEARER",
        "OAUTH",
        "JWT",
        "APIKEY",
    ];
    if sensitive_substrings
        .iter()
        .any(|needle| upper.contains(needle))
    {
        return true;
    }

    let tokens = env_key_tokens_upper(key);
    let sensitive_tokens = ["KEY", "PAT", "PWD", "PASS", "CERT"];
    tokens
        .iter()
        .any(|token| sensitive_tokens.contains(&token.as_str()))
}

fn looks_sensitive_env_value(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("bearer ") || lower.starts_with("basic ") {
        return true;
    }
    if trimmed.contains("://") && trimmed.contains('@') {
        return true;
    }
    if trimmed.len() >= 32
        && !trimmed.contains(char::is_whitespace)
        && !trimmed.starts_with('/')
        && trimmed.chars().any(|ch| ch.is_ascii_alphabetic())
        && trimmed.chars().any(|ch| ch.is_ascii_digit())
    {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Helper shared by the new sys_* functions below
// ---------------------------------------------------------------------------

fn tool_available_sys(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 5. sys_process_kill
// ---------------------------------------------------------------------------

/// Kill process(es) by PID or name.
///
/// `kill -<signal> <pid>` or `pkill -<signal> <name>`.
/// Populates `targets_found` by querying `pgrep -a` before killing.
/// Returns JSON: `{killed, targets_found: [{pid, name}], signal}`
pub fn sys_process_kill_work(pid: Option<u32>, name: Option<String>, signal: String) -> String {
    let sig = signal.trim_start_matches("SIG").to_string();
    if sig.is_empty() {
        return json!({"error": "signal is required (e.g. 9, TERM, KILL)"}).to_string();
    }
    if pid.is_none() && name.is_none() {
        return json!({"error": "either pid or name must be provided"}).to_string();
    }

    let mut targets_found: Vec<serde_json::Value> = Vec::new();

    if let Some(p) = pid {
        let proc_name = Command::new("ps")
            .args(["-p", &p.to_string(), "-o", "comm="])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        targets_found.push(json!({"pid": p, "name": proc_name}));
        let killed = Command::new("kill")
            .args([format!("-{sig}"), p.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        return json!({
            "killed":        killed,
            "targets_found": targets_found,
            "signal":        sig,
        })
        .to_string();
    }

    if let Some(ref n) = name {
        if n.is_empty() {
            return json!({"error": "name must not be empty"}).to_string();
        }
        if tool_available_sys("pgrep") {
            if let Ok(o) = Command::new("pgrep").args(["-a", n]).output() {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                for line in stdout.lines() {
                    let mut parts = line.splitn(2, ' ');
                    let p = parts.next().unwrap_or("").to_string();
                    let pname = parts.next().unwrap_or("").to_string();
                    targets_found.push(json!({"pid": p, "name": pname}));
                }
            }
        }
        let killed = Command::new("pkill")
            .args([format!("-{sig}"), n.clone()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        return json!({
            "killed":        killed,
            "targets_found": targets_found,
            "signal":        sig,
            "name":          n,
        })
        .to_string();
    }

    json!({"error": "unreachable"}).to_string()
}

// ---------------------------------------------------------------------------
// 6. sys_service_control
// ---------------------------------------------------------------------------

/// Control a systemd service (start/stop/restart/status/enable/disable/reload).
/// Returns JSON: `{service, action, success, output, active_state}`
pub fn sys_service_control_work(service: String, action: String) -> String {
    const VALID: &[&str] = &[
        "start",
        "stop",
        "restart",
        "status",
        "enable",
        "disable",
        "reload",
        "is-active",
        "is-enabled",
    ];
    if !VALID.contains(&action.as_str()) {
        return json!({
            "error": format!(
                "invalid action '{}'; valid: {}",
                action,
                VALID.join(", ")
            ),
        })
        .to_string();
    }
    if service.is_empty() {
        return json!({"error": "service name is required"}).to_string();
    }
    if !tool_available_sys("systemctl") {
        return json!({
            "error":          "systemctl not found; not a systemd-based system",
            "tool_available": false,
        })
        .to_string();
    }

    match Command::new("systemctl").args([&action, &service]).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let output = if stdout.is_empty() { stderr } else { stdout };
            let success = o.status.success();
            let active_state = if matches!(action.as_str(), "status" | "is-active") {
                Command::new("systemctl")
                    .args(["show", &service, "--property=ActiveState"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .and_then(|s| {
                        s.lines()
                            .find(|l| l.starts_with("ActiveState="))
                            .map(|l| l.trim_start_matches("ActiveState=").to_string())
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };
            json!({
                "service":      service,
                "action":       action,
                "success":      success,
                "output":       output,
                "active_state": active_state,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":   format!("systemctl failed: {e}"),
            "service": service,
            "action":  action,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 7. sys_cron_list
// ---------------------------------------------------------------------------

/// List cron jobs: current user's crontab + system cron directories.
/// Returns JSON: `{user_crontab: [entries], system_cron_files: [paths], total_jobs}`
pub fn sys_cron_list_work() -> String {
    let user_crontab: Vec<String> = Command::new("crontab")
        .args(["-l"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    let system_dirs = [
        "/etc/cron.d",
        "/etc/cron.daily",
        "/etc/cron.hourly",
        "/etc/cron.weekly",
        "/etc/cron.monthly",
    ];
    let mut system_cron_files: Vec<String> = Vec::new();
    if std::path::Path::new("/etc/crontab").exists() {
        system_cron_files.push("/etc/crontab".to_string());
    }
    for dir in &system_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                system_cron_files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    let total_jobs = user_crontab.len() + system_cron_files.len();
    json!({
        "user_crontab":      user_crontab,
        "system_cron_files": system_cron_files,
        "total_jobs":        total_jobs,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 8. sys_cron_add
// ---------------------------------------------------------------------------

/// Add an entry to the current user's crontab.
///
/// Validates that `schedule` has exactly 5 or 6 whitespace-separated fields.
/// Appends via `(crontab -l 2>/dev/null; echo "<entry>") | crontab -`.
/// Returns JSON: `{added, entry}`
pub fn sys_cron_add_work(schedule: String, command: String) -> String {
    if command.is_empty() {
        return json!({"error": "command is required"}).to_string();
    }
    let fields: Vec<&str> = schedule.split_whitespace().collect();
    if fields.len() != 5 && fields.len() != 6 {
        return json!({
            "error": format!(
                "schedule must have 5 or 6 fields (got {}); example: '0 12 * * *'",
                fields.len()
            ),
            "added": false,
        })
        .to_string();
    }

    let entry = format!("{schedule} {command}");
    // Escape single quotes in the entry for use inside '...'
    let safe_entry = entry.replace('\'', "'\"'\"'");
    let sh_cmd = format!("(crontab -l 2>/dev/null; echo '{safe_entry}') | crontab -");

    let added = Command::new("sh")
        .args(["-c", &sh_cmd])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    json!({
        "added": added,
        "entry": entry,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 9. sys_journal_tail
// ---------------------------------------------------------------------------

/// Tail journald log entries for a systemd unit.
///
/// Runs `journalctl -u <unit> -n <lines> --no-pager -o json`.
/// Returns JSON: `{unit, lines_returned, entries: [{timestamp, priority, message}]}`
pub fn sys_journal_tail_work(unit: String, lines: usize) -> String {
    if unit.is_empty() {
        return json!({"error": "unit name is required"}).to_string();
    }
    if !tool_available_sys("journalctl") {
        return json!({
            "error":          "journalctl not found; not a systemd-based system",
            "tool_available": false,
        })
        .to_string();
    }

    let n = lines.clamp(1, 5000).to_string();
    match Command::new("journalctl")
        .args(["-u", &unit, "-n", &n, "--no-pager", "-o", "json"])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let mut entries: Vec<serde_json::Value> = Vec::new();
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    let ts_micros = obj
                        .get("__REALTIME_TIMESTAMP")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    let timestamp = if ts_micros > 0 {
                        format!("{}", ts_micros / 1_000_000)
                    } else {
                        String::new()
                    };
                    let priority = obj
                        .get("PRIORITY")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let message = obj
                        .get("MESSAGE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    entries.push(json!({
                        "timestamp": timestamp,
                        "priority":  priority,
                        "message":   message,
                    }));
                }
            }
            let lines_returned = entries.len();
            json!({
                "unit":           unit,
                "lines_returned": lines_returned,
                "entries":        entries,
            })
            .to_string()
        }
        Err(e) => json!({
            "error": format!("journalctl failed: {e}"),
            "unit":  unit,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 10. sys_docker_ps
// ---------------------------------------------------------------------------

/// List Docker containers.
///
/// Runs `docker ps [--all] --format json` (newer Docker) or
/// `--format '{{json .}}'` (older Docker).
/// Returns JSON: `{containers: [{id, name, image, status, ports, created}]}`
pub fn sys_docker_ps_work(all: bool) -> String {
    if !tool_available_sys("docker") {
        return json!({
            "error":          "docker not found",
            "tool_available": false,
        })
        .to_string();
    }

    let run = |fmt: &str| -> Option<std::process::Output> {
        let mut args = vec!["ps", "--format", fmt];
        if all {
            args.insert(1, "--all");
        }
        Command::new("docker").args(&args).output().ok()
    };

    let output = run("json").or_else(|| run("{{json .}}"));

    match output {
        Some(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let mut containers: Vec<serde_json::Value> = Vec::new();
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                    containers.push(json!({
                        "id":      obj.get("ID").or(obj.get("Id")),
                        "name":    obj.get("Names").or(obj.get("Name")),
                        "image":   obj.get("Image"),
                        "status":  obj.get("Status"),
                        "state":   obj.get("State"),
                        "ports":   obj.get("Ports"),
                        "created": obj.get("CreatedAt").or(obj.get("Created")),
                    }));
                }
            }
            json!({
                "containers":     containers,
                "show_all":       all,
                "tool_available": true,
            })
            .to_string()
        }
        None => json!({
            "error":          "docker ps failed",
            "tool_available": true,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 11. sys_docker_logs
// ---------------------------------------------------------------------------

/// Fetch recent log output from a Docker container.
///
/// Runs `docker logs --tail <lines> <container>`.
/// Returns JSON: `{container, lines, output}`
pub fn sys_docker_logs_work(container: String, lines: usize) -> String {
    if container.is_empty() {
        return json!({"error": "container name/id is required"}).to_string();
    }
    if !tool_available_sys("docker") {
        return json!({
            "error":          "docker not found",
            "tool_available": false,
        })
        .to_string();
    }
    let n = lines.clamp(1, 10_000).to_string();
    match Command::new("docker")
        .args(["logs", "--tail", &n, &container])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let output = if stdout.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            json!({
                "container":      container,
                "lines":          lines,
                "output":         output,
                "tool_available": true,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":     format!("docker logs failed: {e}"),
            "container": container,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 12. sys_docker_exec
// ---------------------------------------------------------------------------

/// Execute a shell command inside a running Docker container.
///
/// Runs `docker exec <container> sh -c '<command>'`.
/// Returns JSON: `{container, command, exit_code, stdout, stderr}`
pub fn sys_docker_exec_work(container: String, command: String) -> String {
    if container.is_empty() {
        return json!({"error": "container name/id is required"}).to_string();
    }
    if command.is_empty() {
        return json!({"error": "command is required"}).to_string();
    }
    if !tool_available_sys("docker") {
        return json!({
            "error":          "docker not found",
            "tool_available": false,
        })
        .to_string();
    }
    match Command::new("docker")
        .args(["exec", &container, "sh", "-c", &command])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let exit_code = o.status.code().unwrap_or(-1);
            json!({
                "container": container,
                "command":   command,
                "exit_code": exit_code,
                "stdout":    stdout,
                "stderr":    stderr,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":     format!("docker exec failed: {e}"),
            "container": container,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 13. sys_docker_build
// ---------------------------------------------------------------------------

/// Build a Docker image and return timing + last 20 lines of output.
///
/// Runs `docker build -t <tag> <path>`.
/// Returns JSON: `{tag, path, success, output_tail: [last 20 lines], build_time_ms}`
pub fn sys_docker_build_work(path: String, tag: String) -> String {
    if path.is_empty() {
        return json!({"error": "build context path is required"}).to_string();
    }
    if tag.is_empty() {
        return json!({"error": "image tag is required"}).to_string();
    }
    if !tool_available_sys("docker") {
        return json!({
            "error":          "docker not found",
            "tool_available": false,
        })
        .to_string();
    }
    let start = Instant::now();
    match Command::new("docker")
        .args(["build", "-t", &tag, &path])
        .output()
    {
        Ok(o) => {
            let build_time_ms = start.elapsed().as_millis() as u64;
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let combined = if stdout.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            let success = o.status.success();
            let output_tail: Vec<String> = combined
                .lines()
                .rev()
                .take(20)
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            json!({
                "tag":           tag,
                "path":          path,
                "success":       success,
                "output_tail":   output_tail,
                "build_time_ms": build_time_ms,
            })
            .to_string()
        }
        Err(e) => json!({
            "error": format!("docker build failed: {e}"),
            "tag":   tag,
            "path":  path,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 14. sys_k8s_pods
// ---------------------------------------------------------------------------

/// List Kubernetes pods in a namespace.
///
/// Runs `kubectl get pods -n <namespace> -o json`.
/// Returns JSON: `{namespace, pod_count, pods: [{name, status, ready, restarts, node, start_time}]}`
pub fn sys_k8s_pods_work(namespace: String) -> String {
    if !tool_available_sys("kubectl") {
        return json!({
            "error":          "kubectl not found",
            "tool_available": false,
        })
        .to_string();
    }
    let ns = if namespace.is_empty() {
        "default".to_string()
    } else {
        namespace
    };
    match Command::new("kubectl")
        .args(["get", "pods", "-n", &ns, "-o", "json"])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if !o.status.success() {
                return json!({"error": stderr, "namespace": ns}).to_string();
            }
            let parsed = serde_json::from_str::<serde_json::Value>(&stdout).unwrap_or_default();
            let empty: Vec<serde_json::Value> = vec![];
            let items = parsed["items"].as_array().unwrap_or(&empty);
            let pods: Vec<serde_json::Value> = items
                .iter()
                .map(|item| {
                    let name = item["metadata"]["name"].as_str().unwrap_or("").to_string();
                    let phase = item["status"]["phase"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    let node = item["spec"]["nodeName"].as_str().unwrap_or("").to_string();
                    let start_time = item["status"]["startTime"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    let empty_cs: Vec<serde_json::Value> = vec![];
                    let cs = item["status"]["containerStatuses"]
                        .as_array()
                        .unwrap_or(&empty_cs);
                    let ready = cs.iter().all(|c| c["ready"].as_bool().unwrap_or(false));
                    let restarts: u64 = cs
                        .iter()
                        .map(|c| c["restartCount"].as_u64().unwrap_or(0))
                        .sum();
                    json!({
                        "name":       name,
                        "status":     phase,
                        "ready":      ready,
                        "restarts":   restarts,
                        "node":       node,
                        "start_time": start_time,
                    })
                })
                .collect();
            json!({
                "namespace":      ns,
                "pod_count":      pods.len(),
                "pods":           pods,
                "tool_available": true,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":     format!("kubectl failed: {e}"),
            "namespace": ns,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 15. sys_k8s_logs
// ---------------------------------------------------------------------------

/// Fetch recent log lines from a Kubernetes pod.
///
/// Runs `kubectl logs <pod> -n <namespace> --tail=<lines> [-c <container>]`.
/// Returns JSON: `{pod, namespace, container, lines, output}`
pub fn sys_k8s_logs_work(
    pod: String,
    namespace: String,
    lines: usize,
    container: Option<String>,
) -> String {
    if pod.is_empty() {
        return json!({"error": "pod name is required"}).to_string();
    }
    if !tool_available_sys("kubectl") {
        return json!({
            "error":          "kubectl not found",
            "tool_available": false,
        })
        .to_string();
    }
    let ns = if namespace.is_empty() {
        "default".to_string()
    } else {
        namespace
    };
    let tail_flag = format!("--tail={}", lines.clamp(1, 50_000));
    let mut cmd = Command::new("kubectl");
    cmd.args(["logs", &pod, "-n", &ns, &tail_flag]);
    if let Some(ref c) = container {
        if !c.is_empty() {
            cmd.args(["-c", c]);
        }
    }
    match cmd.output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let output = if stdout.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            json!({
                "pod":       pod,
                "namespace": ns,
                "container": container,
                "lines":     lines,
                "output":    output,
            })
            .to_string()
        }
        Err(e) => json!({
            "error": format!("kubectl logs failed: {e}"),
            "pod":   pod,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 16. sys_k8s_apply
// ---------------------------------------------------------------------------

/// Apply a Kubernetes manifest file.
///
/// Runs `kubectl apply -f <manifest_path>` and parses resource names from output.
/// Returns JSON: `{success, output, resources_applied: [names], manifest_path}`
pub fn sys_k8s_apply_work(manifest_path: String) -> String {
    if manifest_path.is_empty() {
        return json!({"error": "manifest_path is required"}).to_string();
    }
    if !tool_available_sys("kubectl") {
        return json!({
            "error":          "kubectl not found",
            "tool_available": false,
        })
        .to_string();
    }
    match Command::new("kubectl")
        .args(["apply", "-f", &manifest_path])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let success = o.status.success();
            let output = if stdout.trim().is_empty() {
                stderr
            } else {
                stdout
            };
            // Parse lines like "deployment.apps/myapp created"
            let resources_applied: Vec<String> = output
                .lines()
                .filter_map(|l| {
                    let word = l.split_whitespace().next()?;
                    if word.contains('/') {
                        Some(word.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            json!({
                "success":           success,
                "output":            output,
                "resources_applied": resources_applied,
                "manifest_path":     manifest_path,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":         format!("kubectl apply failed: {e}"),
            "manifest_path": manifest_path,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 17. sys_hardware_info
// ---------------------------------------------------------------------------

/// Aggregate hardware information from multiple system commands.
///
/// Runs `lscpu`, `free -h`, `lsblk [-J]`, `lspci`, `lsusb`.
/// Returns JSON: `{cpu: {...}, memory: {...}, disks: [...], pci_devices: [...], usb_devices: [...]}`
pub fn sys_hardware_info_work() -> String {
    // -- CPU via lscpu --------------------------------------------------------
    let cpu_info = if tool_available_sys("lscpu") {
        match Command::new("lscpu").output() {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout).to_string();
                let mut model = String::new();
                let mut logical_cpus = 0usize;
                let mut threads_per_core = 1usize;
                let mut sockets = 1usize;
                let mut arch = String::new();
                for line in text.lines() {
                    if let Some(pos) = line.find(':') {
                        let key = line[..pos].trim().to_lowercase();
                        let val = line[pos + 1..].trim().to_string();
                        match key.as_str() {
                            "model name" => model = val,
                            "cpu(s)" => logical_cpus = val.parse().unwrap_or(0),
                            "thread(s) per core" => threads_per_core = val.parse().unwrap_or(1),
                            "socket(s)" => sockets = val.parse().unwrap_or(1),
                            "architecture" => arch = val,
                            _ => {}
                        }
                    }
                }
                json!({
                    "model":        model,
                    "logical_cpus": logical_cpus,
                    "threads":      logical_cpus * threads_per_core,
                    "sockets":      sockets,
                    "architecture": arch,
                })
            }
            Err(e) => json!({"error": format!("lscpu failed: {e}")}),
        }
    } else {
        json!({"error": "lscpu not found"})
    };

    // -- Memory via free -h ---------------------------------------------------
    let mem_info = Command::new("free")
        .args(["-h"])
        .output()
        .map(|o| {
            let text = String::from_utf8_lossy(&o.stdout).to_string();
            let mut total = String::new();
            let mut available = String::new();
            for line in text.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    total = parts.get(1).copied().unwrap_or("").to_string();
                    available = parts.get(6).copied().unwrap_or("").to_string();
                    break;
                }
            }
            json!({"total": total, "available": available})
        })
        .unwrap_or_else(|e| json!({"error": format!("free -h failed: {e}")}));

    // -- Disks via lsblk ------------------------------------------------------
    let disk_info: Vec<serde_json::Value> = {
        // Prefer JSON output
        let jout = Command::new("lsblk")
            .args(["-o", "NAME,SIZE,TYPE,MOUNTPOINT", "-J"])
            .output();
        if let Ok(o) = jout {
            if o.status.success() {
                let text = String::from_utf8_lossy(&o.stdout).to_string();
                serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|p| p["blockdevices"].as_array().cloned())
                    .map(|devs| {
                        devs.iter()
                            .map(|d| {
                                json!({
                                    "name":       d["name"],
                                    "size":       d["size"],
                                    "type":       d["type"],
                                    "mountpoint": d["mountpoint"],
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                vec![]
            }
        } else {
            // Fallback: plain text
            Command::new("lsblk")
                .args(["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])
                .output()
                .map(|o| {
                    let text = String::from_utf8_lossy(&o.stdout).to_string();
                    text.lines()
                        .skip(1)
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| {
                            let p: Vec<&str> = l.split_whitespace().collect();
                            json!({
                                "name":       p.first().copied().unwrap_or(""),
                                "size":       p.get(1).copied().unwrap_or(""),
                                "type":       p.get(2).copied().unwrap_or(""),
                                "mountpoint": p.get(3).copied().unwrap_or(""),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
    };

    // -- PCI devices via lspci -----------------------------------------------
    let pci_devices: Vec<serde_json::Value> = if tool_available_sys("lspci") {
        Command::new("lspci")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .take(30)
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| json!({"description": l.trim()}))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    // -- USB devices via lsusb -----------------------------------------------
    let usb_devices: Vec<serde_json::Value> = if tool_available_sys("lsusb") {
        Command::new("lsusb")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| json!({"description": l.trim()}))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    json!({
        "cpu":         cpu_info,
        "memory":      mem_info,
        "disks":       disk_info,
        "pci_devices": pci_devices,
        "usb_devices": usb_devices,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_env_get_basic() {
        // HOME should be set on most systems
        let result = system_env_get_work("HOME".to_string());
        assert!(result.contains("SYSTEM: ENV GET"));
        assert!(result.contains("HOME"));
    }

    #[test]
    fn test_system_env_get_sensitive() {
        let result = system_env_get_work("MY_SECRET_PASSWORD".to_string());
        assert!(result.contains("REDACTED"));
    }

    #[test]
    fn test_system_env_get_redacts_common_secret_names() {
        for key in [
            "OPENAI_API_KEY",
            "GH_PAT",
            "DATABASE_URL",
            "SLACK_WEBHOOK_URL",
            "AWS_SESSION_TOKEN",
            "SSH_AUTH_SOCK",
            "CLIENT_CERT",
            "OAUTH_REFRESH_TOKEN",
        ] {
            let result = system_env_get_work(key.to_string());
            assert!(result.contains("REDACTED"), "{key} should be redacted");
        }
    }

    #[test]
    fn test_system_env_get_avoids_key_substring_false_positive() {
        assert!(!is_sensitive_env_key("MONKEY_MODE"));
    }

    #[test]
    fn test_system_env_value_secret_heuristic() {
        assert!(looks_sensitive_env_value(
            "postgres://user:pass@localhost:5432/db"
        ));
        assert!(looks_sensitive_env_value(
            "Bearer abcdefghijklmnopqrstuvwxyz1234567890"
        ));
        assert!(!looks_sensitive_env_value("/home/panosboy/Documents"));
    }

    #[test]
    fn test_system_env_get_not_set() {
        let result = system_env_get_work("PB_MCP_SERVER_NONEXISTENT_VAR_XYZ".to_string());
        assert!(result.contains("not set"));
    }

    #[test]
    fn test_system_process_list() {
        let result = system_process_list_work(10);
        assert!(result.contains("SYSTEM: PROCESS LIST"));
    }

    #[test]
    fn test_system_disk_usage() {
        let result = system_disk_usage_work(PathBuf::from("/tmp"));
        assert!(result.contains("SYSTEM: DISK USAGE"));
    }

    #[test]
    fn test_system_network_interfaces() {
        let result = system_network_interfaces_work();
        assert!(result.contains("SYSTEM: NETWORK INTERFACES"));
    }
}
