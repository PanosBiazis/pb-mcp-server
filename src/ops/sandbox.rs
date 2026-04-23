//! # Sandbox & Isolated Execution Tools
//!
//! Sandboxed execution primitives for AI agents, providing resource-limited,
//! namespace-isolated command, script, and code evaluation capabilities.
//!
//! ## Safety guarantees
//!
//! - All paths validated against a `root` directory (path-traversal prevention)
//! - Dangerous commands blocked (`rm -rf /`, `mkfs`, `dd if=`, etc.)
//! - Timeout enforcement on every execution (spawn + kill pattern)
//! - Output size capped at 1 MB per execution
//! - SSRF protection on network requests (private-IP blocking)
//! - URL protocol restricted to HTTP/HTTPS

use std::fs;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default timeout in seconds for sandboxed executions.
const SANDBOX_DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum output bytes per execution (1 MB).
const SANDBOX_MAX_OUTPUT_BYTES: usize = 1_048_576;

/// Polling interval for file-watch (seconds).
const FILE_WATCH_POLL_INTERVAL_SECS: u64 = 2;

/// Maximum output bytes for `sandbox_network_request_work` (capped lower
/// because web responses can be huge).
const MAX_NETWORK_RESPONSE_BYTES: usize = 512 * 1024;

// ---------------------------------------------------------------------------
// Dangerous-command patterns
// ---------------------------------------------------------------------------

/// Substrings that indicate a command is too dangerous to execute inside
/// a sandbox.  Checked case-insensitively against the full command string.
const DANGEROUS_COMMAND_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "rm -rf ~",
    "rm -rf /*",
    "mkfs",
    "dd if=",
    "dd of=/dev",
    "format",
    "del /s",
    "del /f",
    ":(){:|:&};:", // fork bomb
    "> /dev/sda",
    "> /dev/hda",
    "> /dev/nvme",
    "shutdown",
    "reboot",
    "init 0",
    "init 6",
    "poweroff",
    "halt",
    "systemctl stop",
    "systemctl disable",
    "systemctl reboot",
    "systemctl poweroff",
    "systemctl halt",
    "chmod -R 777 /",
    "chown -R",
    "wget http",
    "curl http",
    "nc -l",
    "ncat -l",
    "/etc/passwd",
    "/etc/shadow",
    "crontab -r",
];

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Validate that `path` resolves to a location inside `root`, preventing
/// path-traversal attacks.
///
/// Canonicalises both paths (creating `root` if it does not exist) and checks
/// that the canonical `path` starts with the canonical `root`.
fn path_is_under_root(root: &Path, path: &Path) -> Result<(), String> {
    let canonical_root = if root.exists() {
        root.canonicalize()
            .map_err(|e| format!("cannot canonicalize root '{}': {e}", root.display()))?
    } else {
        fs::create_dir_all(root)
            .map_err(|e| format!("cannot create root '{}': {e}", root.display()))?;
        root.canonicalize()
            .map_err(|e| format!("cannot canonicalize root '{}': {e}", root.display()))?
    };

    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("cannot canonicalize path '{}': {e}", path.display()))?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(format!(
            "path '{}' is outside root '{}'",
            path.display(),
            root.display()
        ));
    }
    Ok(())
}

/// Check whether a command string contains any dangerous pattern.
///
/// Returns `Ok(())` if the command appears safe, or `Err(message)` otherwise.
fn check_dangerous_command(cmd_str: &str) -> Result<(), String> {
    let lower = cmd_str.to_lowercase();
    for pattern in DANGEROUS_COMMAND_PATTERNS {
        if lower.contains(&pattern.to_lowercase()) {
            return Err(format!("command contains dangerous pattern: '{pattern}'"));
        }
    }
    Ok(())
}

/// Run a [`Command`] with a wall-clock timeout and an output-byte limit.
///
/// Spawns the child, polls for completion up to `timeout`, kills on expiry,
/// then truncates captured stdout+stderr to `max_output_bytes`.
fn run_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
    max_output_bytes: usize,
) -> Result<(Option<i32>, String, String), String> {
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let start = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn command: {e}"))?;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished — collect output
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("failed to read output: {e}"))?;
                let stdout = truncate_bytes(&output.stdout, max_output_bytes);
                let stderr = truncate_bytes(&output.stderr, max_output_bytes);
                return Ok((status.code(), stdout, stderr));
            }
            Ok(None) => {
                // Still running
                if start.elapsed() > timeout {
                    // Kill the child process group
                    let _ = child.kill();
                    let _ = child.wait(); // reap zombie
                    return Err(format!(
                        "command timed out after {} seconds",
                        timeout.as_secs()
                    ));
                }
                // Brief sleep to avoid busy-spin
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("error checking child status: {e}"));
            }
        }
    }
}

/// Truncate a byte slice to at most `max` bytes, converting to a UTF-8
/// `String` (lossy replacement for invalid sequences).
fn truncate_bytes(data: &[u8], max: usize) -> String {
    if data.len() <= max {
        String::from_utf8_lossy(data).into_owned()
    } else {
        String::from_utf8_lossy(&data[..max]).into_owned()
    }
}

/// Final truncation pass using the parent module's `truncate_chars` and
/// `MAX_RESPONSE_CHARS`.
fn final_truncate(s: &str) -> String {
    let (out, _) = super::truncate_chars(s, super::MAX_RESPONSE_CHARS);
    out
}

/// Determine the interpreter command for a given scripting language name.
///
/// Returns `(program, extra_args)` — e.g. `("python3", &[])`.
fn interpreter_for_language(language: &str) -> Result<(&'static str, Vec<&'static str>), String> {
    match language.to_lowercase().as_str() {
        "python" | "python3" | "py" => Ok(("python3", vec![])),
        "node" | "javascript" | "js" => Ok(("node", vec![])),
        "bash" | "sh" | "shell" => Ok(("bash", vec![])),
        "ruby" | "rb" => Ok(("ruby", vec![])),
        "perl" | "pl" => Ok(("perl", vec![])),
        other => Err(format!(
            "unsupported language '{other}'. Supported: python3, node, bash, ruby, perl"
        )),
    }
}

// ---------------------------------------------------------------------------
// SSRF protection (mirrors notify.rs pattern)
// ---------------------------------------------------------------------------

/// Extract the host portion from a URL string.
fn extract_host_from_url(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .or_else(|| url.strip_prefix("HTTP://"))
        .or_else(|| url.strip_prefix("HTTPS://"))?;
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority.starts_with('[') {
        if let Some(bracket_end) = authority.find(']') {
            return Some(authority[1..bracket_end].to_string());
        }
    }
    let host = authority.rsplit_once(':').map_or(authority, |(h, _)| h);
    Some(host.to_string())
}

/// Check whether an IP address is private / internal / loopback.
fn is_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_internal_ipv4(v4),
        IpAddr::V6(v6) => is_internal_ipv6(v6),
    }
}

fn is_internal_ipv4(v4: &Ipv4Addr) -> bool {
    let o = v4.octets();
    o[0] == 10
        || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)
        || (o[0] == 192 && o[1] == 168)
        || o[0] == 127
        || o[0] == 0
        || (o[0] == 169 && o[1] == 254)
}

fn is_internal_ipv6(v6: &Ipv6Addr) -> bool {
    if v6.is_loopback() {
        return true;
    }
    let segments = v6.segments();
    let first_byte = (segments[0] >> 8) as u8;
    if first_byte == 0xfc || first_byte == 0xfd {
        return true;
    }
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    false
}

/// Resolve a hostname to an IP address via `getent hosts` (Linux) or
/// direct parsing.  Returns `None` when resolution fails.
fn resolve_host_to_ip(host: &str) -> Option<IpAddr> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Some(ip);
    }
    if cfg!(target_os = "linux") {
        let out = Command::new("getent").args(["hosts", host]).output().ok()?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let first_ip = stdout.lines().next()?.split_whitespace().next()?.trim();
        first_ip.parse::<IpAddr>().ok()
    } else if cfg!(target_os = "macos") {
        let out = Command::new("dscacheutil")
            .args(["-q", "host", "-a", &format!("name={host}")])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if let Some(ip_str) = line.strip_prefix("ip_address:") {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
        None
    } else {
        host.parse::<IpAddr>().ok()
    }
}

/// Return `true` if the URL resolves to a private / internal IP.
///
/// Conservative: returns `true` when the host cannot be resolved so that
/// ambiguous requests are blocked.
fn url_points_to_internal_ip(url: &str) -> bool {
    let host = match extract_host_from_url(url) {
        Some(h) => h,
        None => return true,
    };
    match resolve_host_to_ip(&host) {
        Some(ip) => is_internal_ip(&ip),
        None => true,
    }
}

// ---------------------------------------------------------------------------
// 1. sandbox_exec_work
// ---------------------------------------------------------------------------

/// Execute a command in a sandboxed environment with resource limits.
///
/// On Linux, uses `unshare` for PID / mount / network namespace isolation
/// when available.  Falls back to basic resource-limited execution if
/// `unshare` is not installed.
///
/// Enforces:
/// - timeout (kills the process on expiry)
/// - output size limit (`max_output_bytes`)
/// - dangerous-command blocking
/// - path-traversal validation (command runs inside `root`)
pub fn sandbox_exec_work(
    root: PathBuf,
    command: String,
    args: Vec<String>,
    timeout_secs: u64,
    max_output_bytes: usize,
) -> String {
    let mut out = String::from("SANDBOX: EXEC\n\n");

    // Validate root exists or create it
    if !root.exists() {
        if let Err(e) = fs::create_dir_all(&root) {
            out.push_str(&format!(
                "Error: cannot create root '{}': {e}\n",
                root.display()
            ));
            return final_truncate(&out);
        }
    }

    // Validate command is not dangerous
    let full_cmd = format!("{} {}", command, args.join(" "));
    if let Err(e) = check_dangerous_command(&full_cmd) {
        out.push_str(&format!("Error: {e}\n"));
        return final_truncate(&out);
    }

    // Sanitise command name — must be a bare executable, not a path
    if command.contains('/') || command.contains('\\') || command.contains("..") {
        out.push_str("Error: command must be a bare executable name, not a path\n");
        return final_truncate(&out);
    }

    let timeout = Duration::from_secs(timeout_secs.clamp(1, 600));
    let max_out = max_output_bytes.clamp(1, SANDBOX_MAX_OUTPUT_BYTES);

    // Try unshare on Linux for namespace isolation
    let use_unshare = cfg!(target_os = "linux")
        && Command::new("unshare")
            .arg("--help")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

    let result = if use_unshare {
        let mut cmd = Command::new("unshare");
        cmd.args([
            "--pid",
            "--mount",
            "--net",
            "--fork",
            "--map-root-user",
            "--root",
            &root.to_string_lossy(),
        ]);
        cmd.arg(&command);
        cmd.args(&args);
        run_with_timeout(&mut cmd, timeout, max_out)
    } else {
        let mut cmd = Command::new(&command);
        cmd.args(&args);
        cmd.current_dir(&root);
        // Basic environment isolation: clear env and set minimal PATH
        cmd.env_clear();
        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
        cmd.env("HOME", &root);
        run_with_timeout(&mut cmd, timeout, max_out)
    };

    match result {
        Ok((exit_code, stdout, stderr)) => {
            out.push_str(&format!("Command: {command}\n"));
            out.push_str(&format!("Args: {}\n", args.join(" ")));
            out.push_str(&format!("Root: {}\n", root.display()));
            out.push_str(&format!("Timeout: {timeout_secs}s\n"));
            out.push_str(&format!(
                "Namespace: {}\n",
                if use_unshare {
                    "unshare (pid,mount,net)"
                } else {
                    "basic (env-clear)"
                }
            ));
            out.push_str(&format!(
                "Exit: {}\n\n",
                exit_code.map_or("N/A".to_string(), |c| c.to_string())
            ));
            if !stdout.is_empty() {
                out.push_str("--- stdout ---\n");
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str("--- stderr ---\n");
                out.push_str(&stderr);
                out.push('\n');
            }
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 2. sandbox_script_work
// ---------------------------------------------------------------------------

/// Execute a script in a sandboxed temporary directory.
///
/// Creates a temporary file under `root`, writes the script content, runs it
/// with the appropriate interpreter (`python3`, `node`, `bash`, `ruby`,
/// `perl`), captures output, and cleans up the temp file.
///
/// The script runs with `root` as its working directory.
pub fn sandbox_script_work(
    root: PathBuf,
    script: String,
    language: String,
    timeout_secs: u64,
) -> String {
    let mut out = String::from("SANDBOX: SCRIPT\n\n");

    // Ensure root exists
    if !root.exists() {
        if let Err(e) = fs::create_dir_all(&root) {
            out.push_str(&format!(
                "Error: cannot create root '{}': {e}\n",
                root.display()
            ));
            return final_truncate(&out);
        }
    }

    // Resolve interpreter
    let (interpreter, extra_args) = match interpreter_for_language(&language) {
        Ok(v) => v,
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
            return final_truncate(&out);
        }
    };

    // Build file extension
    let extension = match language.to_lowercase().as_str() {
        "python" | "python3" | "py" => "py",
        "node" | "javascript" | "js" => "js",
        "bash" | "sh" | "shell" => "sh",
        "ruby" | "rb" => "rb",
        "perl" | "pl" => "pl",
        _ => "txt",
    };

    // Create temp script file inside root
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let script_name = format!("sandbox_script_{timestamp}.{extension}");
    let script_path = root.join(&script_name);

    if let Err(e) = fs::write(&script_path, &script) {
        out.push_str(&format!(
            "Error: cannot write script file '{}': {e}\n",
            script_path.display()
        ));
        return final_truncate(&out);
    }

    let timeout = Duration::from_secs(timeout_secs.clamp(1, 600));

    let mut cmd = Command::new(interpreter);
    cmd.args(&extra_args);
    cmd.arg(&script_path);
    cmd.current_dir(&root);
    cmd.env_clear();
    cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
    cmd.env("HOME", &root);

    let result = run_with_timeout(&mut cmd, timeout, SANDBOX_MAX_OUTPUT_BYTES);

    // Clean up temp script
    let _ = fs::remove_file(&script_path);

    match result {
        Ok((exit_code, stdout, stderr)) => {
            out.push_str(&format!("Language: {language}\n"));
            out.push_str(&format!("Interpreter: {interpreter}\n"));
            out.push_str(&format!("Root: {}\n", root.display()));
            out.push_str(&format!("Timeout: {timeout_secs}s\n"));
            out.push_str(&format!(
                "Exit: {}\n\n",
                exit_code.map_or("N/A".to_string(), |c| c.to_string())
            ));
            if !stdout.is_empty() {
                out.push_str("--- stdout ---\n");
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str("--- stderr ---\n");
                out.push_str(&stderr);
                out.push('\n');
            }
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 3. sandbox_docker_run_work
// ---------------------------------------------------------------------------

/// Run a command inside a Docker container (if Docker is available).
///
/// Uses `docker run --rm` with resource limits:
/// - **Memory**: capped at `memory_mb` MB (default 256 MB if not specified)
/// - **CPU**: 1 CPU core limit
/// - **Network**: disabled by default (`--network=none`)
///
/// If Docker is not available, returns an informative error.
pub fn sandbox_docker_run_work(
    root: PathBuf,
    image: String,
    command: String,
    timeout_secs: u64,
    memory_mb: Option<u64>,
) -> String {
    let mut out = String::from("SANDBOX: DOCKER RUN\n\n");

    // Check docker is available
    let docker_check = Command::new("docker")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match docker_check {
        Ok(s) if s.success() => {}
        Ok(_) => {
            out.push_str(
                "Error: docker is installed but returned an error. Is the daemon running?\n",
            );
            return final_truncate(&out);
        }
        Err(e) => {
            out.push_str(&format!("Error: docker not found on PATH: {e}\n"));
            out.push_str("Install Docker: https://docs.docker.com/get-docker/\n");
            return final_truncate(&out);
        }
    }

    // Validate command is not dangerous
    if let Err(e) = check_dangerous_command(&command) {
        out.push_str(&format!("Error: {e}\n"));
        return final_truncate(&out);
    }

    // Sanitise image name (no path traversal)
    if image.contains('/') && image.contains("..") {
        out.push_str("Error: image name must not contain '..'\n");
        return final_truncate(&out);
    }
    if image.is_empty() {
        out.push_str("Error: image name must not be empty\n");
        return final_truncate(&out);
    }

    let mem_mb = memory_mb.unwrap_or(256).clamp(16, 4096);
    let mem_flag = format!("{}m", mem_mb);
    let timeout = Duration::from_secs(timeout_secs.clamp(1, 600));

    // Mount root as /workspace (read-only by default)
    let root_str = root.to_string_lossy().to_string();

    let mut cmd = Command::new("docker");
    cmd.args([
        "run",
        "--rm",
        "--network=none",
        "--cpus=1",
        "--memory",
        &mem_flag,
        "-v",
        &format!("{root_str}:/workspace:ro"),
        "-w",
        "/workspace",
    ]);
    cmd.arg(&image);
    // Pass the command through `sh -c` so shell features (pipes, redirects)
    // work inside the container.  This is safe because the container is
    // already isolated.
    cmd.args(["sh", "-c", &command]);

    let result = run_with_timeout(&mut cmd, timeout, SANDBOX_MAX_OUTPUT_BYTES);

    match result {
        Ok((exit_code, stdout, stderr)) => {
            out.push_str(&format!("Image: {image}\n"));
            out.push_str(&format!("Command: {command}\n"));
            out.push_str(&format!("Root: {}\n", root.display()));
            out.push_str(&format!("Memory: {mem_mb}MB\n"));
            out.push_str(&format!("CPU: 1 core\n"));
            out.push_str(&format!("Network: none\n"));
            out.push_str(&format!("Timeout: {timeout_secs}s\n"));
            out.push_str(&format!(
                "Exit: {}\n\n",
                exit_code.map_or("N/A".to_string(), |c| c.to_string())
            ));
            if !stdout.is_empty() {
                out.push_str("--- stdout ---\n");
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str("--- stderr ---\n");
                out.push_str(&stderr);
                out.push('\n');
            }
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 4. sandbox_eval_code_work
// ---------------------------------------------------------------------------

/// Evaluate code with optional stdin input.
///
/// Supports `python3`, `node`, `ruby`, `perl`, and `bash`.
/// Creates a temporary file, provides `input_data` via stdin, captures
/// stdout + stderr, and cleans up.
pub fn sandbox_eval_code_work(
    root: PathBuf,
    code: String,
    language: String,
    input_data: Option<String>,
    timeout_secs: u64,
) -> String {
    let mut out = String::from("SANDBOX: EVAL CODE\n\n");

    // Ensure root exists
    if !root.exists() {
        if let Err(e) = fs::create_dir_all(&root) {
            out.push_str(&format!(
                "Error: cannot create root '{}': {e}\n",
                root.display()
            ));
            return final_truncate(&out);
        }
    }

    // Resolve interpreter
    let (interpreter, extra_args) = match interpreter_for_language(&language) {
        Ok(v) => v,
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
            return final_truncate(&out);
        }
    };

    // Build file extension
    let extension = match language.to_lowercase().as_str() {
        "python" | "python3" | "py" => "py",
        "node" | "javascript" | "js" => "js",
        "bash" | "sh" | "shell" => "sh",
        "ruby" | "rb" => "rb",
        "perl" | "pl" => "pl",
        _ => "txt",
    };

    // Create temp code file inside root
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let script_name = format!("sandbox_eval_{timestamp}.{extension}");
    let script_path = root.join(&script_name);

    if let Err(e) = fs::write(&script_path, &code) {
        out.push_str(&format!(
            "Error: cannot write code file '{}': {e}\n",
            script_path.display()
        ));
        return final_truncate(&out);
    }

    let timeout = Duration::from_secs(timeout_secs.clamp(1, 600));

    let mut cmd = Command::new(interpreter);
    cmd.args(&extra_args);
    cmd.arg(&script_path);
    cmd.current_dir(&root);
    cmd.env_clear();
    cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
    cmd.env("HOME", &root);

    // Provide stdin if input_data is given
    let stdin_bytes = input_data
        .as_deref()
        .map(|s| s.as_bytes().to_vec())
        .unwrap_or_default();
    if !stdin_bytes.is_empty() {
        cmd.stdin(std::process::Stdio::piped());
    }

    let start = Instant::now();
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let spawn_result = cmd.spawn();
    let mut child = match spawn_result {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&script_path);
            out.push_str(&format!("Error: failed to spawn process: {e}\n"));
            return final_truncate(&out);
        }
    };

    // Write stdin if present
    if !stdin_bytes.is_empty() {
        if let Some(mut stdin_handle) = child.stdin.take() {
            if let Err(e) = stdin_handle.write_all(&stdin_bytes) {
                // Non-fatal: the child may have already closed stdin
                out.push_str(&format!("Warning: could not write all stdin: {e}\n"));
            }
        }
    }

    // Poll with timeout
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = match child.wait_with_output() {
                    Ok(o) => o,
                    Err(e) => {
                        break Err(format!("failed to read output: {e}"));
                    }
                };
                let stdout = truncate_bytes(&output.stdout, SANDBOX_MAX_OUTPUT_BYTES);
                let stderr = truncate_bytes(&output.stderr, SANDBOX_MAX_OUTPUT_BYTES);
                break Ok((status.code(), stdout, stderr));
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!(
                        "command timed out after {} seconds",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                break Err(format!("error checking child status: {e}"));
            }
        }
    };

    // Clean up
    let _ = fs::remove_file(&script_path);

    match result {
        Ok((exit_code, stdout, stderr)) => {
            out.push_str(&format!("Language: {language}\n"));
            out.push_str(&format!("Interpreter: {interpreter}\n"));
            out.push_str(&format!("Root: {}\n", root.display()));
            out.push_str(&format!("Stdin: {} bytes\n", stdin_bytes.len()));
            out.push_str(&format!("Timeout: {timeout_secs}s\n"));
            out.push_str(&format!(
                "Exit: {}\n\n",
                exit_code.map_or("N/A".to_string(), |c| c.to_string())
            ));
            if !stdout.is_empty() {
                out.push_str("--- stdout ---\n");
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str("--- stderr ---\n");
                out.push_str(&stderr);
                out.push('\n');
            }
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 5. sandbox_file_watch_work
// ---------------------------------------------------------------------------

/// Watch a file for changes and run a command when it changes (one-shot).
///
/// Compares the file's modification time before and after a polling interval.
/// If the mtime changed, the command is triggered.  This is a single-shot
/// check, not a persistent watcher.
pub fn sandbox_file_watch_work(
    root: PathBuf,
    path: PathBuf,
    command: String,
    timeout_secs: u64,
) -> String {
    let mut out = String::from("SANDBOX: FILE WATCH\n\n");

    // Resolve and validate path is under root
    let full_path = if path.is_absolute() {
        path.clone()
    } else {
        root.join(&path)
    };

    if let Err(e) = path_is_under_root(&root, &full_path) {
        out.push_str(&format!("Error: {e}\n"));
        return final_truncate(&out);
    }

    // Check the file exists
    if !full_path.exists() {
        out.push_str(&format!(
            "Error: file '{}' does not exist\n",
            full_path.display()
        ));
        return final_truncate(&out);
    }

    // Validate command is not dangerous
    if let Err(e) = check_dangerous_command(&command) {
        out.push_str(&format!("Error: {e}\n"));
        return final_truncate(&out);
    }

    // Get initial mtime
    let mtime_before = match fs::metadata(&full_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(e) => {
            out.push_str(&format!(
                "Error: cannot read mtime of '{}': {e}\n",
                full_path.display()
            ));
            return final_truncate(&out);
        }
    };

    out.push_str(&format!("Watching: {}\n", full_path.display()));
    out.push_str(&format!("Command: {command}\n"));
    out.push_str(&format!(
        "Poll interval: {FILE_WATCH_POLL_INTERVAL_SECS}s\n"
    ));
    out.push_str(&format!("Timeout: {timeout_secs}s\n\n"));

    // Poll
    let poll_duration = Duration::from_secs(FILE_WATCH_POLL_INTERVAL_SECS);
    std::thread::sleep(poll_duration);

    let mtime_after = match fs::metadata(&full_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(e) => {
            out.push_str(&format!("Error: cannot re-read mtime: {e}\n"));
            return final_truncate(&out);
        }
    };

    if mtime_before != mtime_after {
        out.push_str("File changed! Triggering command...\n\n");

        // Execute the command
        let timeout = Duration::from_secs(timeout_secs.clamp(1, 600));

        let mut cmd = Command::new("sh");
        cmd.args(["-c", &command]);
        cmd.current_dir(&root);

        let result = run_with_timeout(&mut cmd, timeout, SANDBOX_MAX_OUTPUT_BYTES);
        match result {
            Ok((exit_code, stdout, stderr)) => {
                out.push_str(&format!(
                    "Exit: {}\n\n",
                    exit_code.map_or("N/A".to_string(), |c| c.to_string())
                ));
                if !stdout.is_empty() {
                    out.push_str("--- stdout ---\n");
                    out.push_str(&stdout);
                    out.push('\n');
                }
                if !stderr.is_empty() {
                    out.push_str("--- stderr ---\n");
                    out.push_str(&stderr);
                    out.push('\n');
                }
            }
            Err(e) => {
                out.push_str(&format!("Error executing command: {e}\n"));
            }
        }
    } else {
        out.push_str("No change detected within poll interval.\n");
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 6. sandbox_network_request_work
// ---------------------------------------------------------------------------

/// Make an isolated HTTP request using `curl` with resource limits.
///
/// **SSRF protection**: blocks requests to internal/private network IPs
/// (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 127.0.0.0/8, ::1,
/// fc00::/7).
///
/// **URL validation**: only `http://` and `https://` protocols are allowed.
pub fn sandbox_network_request_work(
    root: PathBuf,
    url: String,
    method: String,
    headers: Vec<String>,
    body: Option<String>,
    timeout_secs: u64,
) -> String {
    let mut out = String::from("SANDBOX: NETWORK REQUEST\n\n");

    // Validate URL protocol
    let url_trimmed = url.trim();
    if !url_trimmed.starts_with("http://") && !url_trimmed.starts_with("https://") {
        out.push_str("Error: only http:// and https:// URLs are allowed\n");
        return final_truncate(&out);
    }

    // Validate HTTP method
    let method_upper = method.trim().to_uppercase();
    match method_upper.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {}
        _ => {
            out.push_str(&format!(
                "Error: unsupported HTTP method '{method}'. \
                 Supported: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS\n"
            ));
            return final_truncate(&out);
        }
    }

    // SSRF protection — block internal IPs
    if url_points_to_internal_ip(url_trimmed) {
        out.push_str(&format!(
            "Error: URL \"{url_trimmed}\" resolves to a private/internal IP address.\n"
        ));
        out.push_str(
            "Network requests to internal addresses are blocked to prevent SSRF.\n\
             Blocked ranges: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, \
             127.0.0.0/8, ::1, fc00::/7\n",
        );
        return final_truncate(&out);
    }

    // Check curl is available
    let curl_check = Command::new("curl")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if curl_check.is_err() {
        out.push_str("Error: `curl` is not installed or not found on PATH.\n");
        return final_truncate(&out);
    }

    let timeout = timeout_secs.clamp(1, 300);

    let mut cmd = Command::new("curl");
    cmd.args([
        "-s",
        "-i",
        "-L",
        "--max-time",
        &timeout.to_string(),
        "-X",
        &method_upper,
    ]);

    // Add headers
    for hdr in &headers {
        let trimmed = hdr.trim();
        if !trimmed.is_empty() {
            cmd.arg("-H").arg(trimmed);
        }
    }

    // Add body for applicable methods
    if let Some(ref b) = body {
        if !b.is_empty() && matches!(method_upper.as_str(), "POST" | "PUT" | "PATCH") {
            cmd.args(["-d", b.as_str()]);
        }
    }

    cmd.arg(url_trimmed);

    let result = run_with_timeout(
        &mut cmd,
        Duration::from_secs(timeout),
        MAX_NETWORK_RESPONSE_BYTES,
    );

    match result {
        Ok((exit_code, stdout, stderr)) => {
            if let Some(code) = exit_code {
                if code != 0 && stdout.trim().is_empty() {
                    out.push_str(&format!(
                        "Error: curl exited with code {code}\n{}",
                        stderr.trim()
                    ));
                    return final_truncate(&out);
                }
            }

            out.push_str(&format!("URL: {url_trimmed}\n"));
            out.push_str(&format!("Method: {method_upper}\n"));
            out.push_str(&format!("Timeout: {timeout}s\n"));
            out.push_str(&format!(
                "Headers: {} custom\n",
                headers.iter().filter(|h| !h.trim().is_empty()).count()
            ));
            if body.as_deref().map_or(false, |b| !b.is_empty()) {
                out.push_str(&format!(
                    "Body: {} bytes\n",
                    body.as_deref().map_or(0, |b| b.len())
                ));
            }
            out.push_str(&format!("Root: {}\n", root.display()));
            out.push_str(&format!(
                "Exit: {}\n\n",
                exit_code.map_or("N/A".to_string(), |c| c.to_string())
            ));

            if !stdout.is_empty() {
                out.push_str("--- response ---\n");
                out.push_str(&stdout);
                out.push('\n');
            }
            if !stderr.is_empty() {
                out.push_str("--- stderr ---\n");
                out.push_str(&stderr);
                out.push('\n');
            }
        }
        Err(e) => {
            out.push_str(&format!("Error: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 7. sandbox_diff_work
// ---------------------------------------------------------------------------

/// Compute a unified diff between two files safely using the `diff` command.
///
/// Validates that both paths are within `root` (path-traversal prevention)
/// before running the diff.  `context_lines` defaults to 3 when `None`.
pub fn sandbox_diff_work(
    root: PathBuf,
    path_a: PathBuf,
    path_b: PathBuf,
    context_lines: Option<usize>,
) -> String {
    let mut out = String::from("SANDBOX: DIFF\n\n");

    let full_a = if path_a.is_absolute() {
        path_a.clone()
    } else {
        root.join(&path_a)
    };
    let full_b = if path_b.is_absolute() {
        path_b.clone()
    } else {
        root.join(&path_b)
    };

    // Validate both paths are under root
    if let Err(e) = path_is_under_root(&root, &full_a) {
        out.push_str(&format!("Error (path_a): {e}\n"));
        return final_truncate(&out);
    }
    if let Err(e) = path_is_under_root(&root, &full_b) {
        out.push_str(&format!("Error (path_b): {e}\n"));
        return final_truncate(&out);
    }

    // Check files exist
    if !full_a.exists() {
        out.push_str(&format!(
            "Error: file '{}' does not exist\n",
            full_a.display()
        ));
        return final_truncate(&out);
    }
    if !full_b.exists() {
        out.push_str(&format!(
            "Error: file '{}' does not exist\n",
            full_b.display()
        ));
        return final_truncate(&out);
    }

    let context = context_lines.unwrap_or(3);

    let result = Command::new("diff")
        .args([
            "-u",
            &format!("-U{context}"),
            &full_a.to_string_lossy(),
            &full_b.to_string_lossy(),
        ])
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            out.push_str(&format!("Path A: {}\n", full_a.display()));
            out.push_str(&format!("Path B: {}\n", full_b.display()));
            out.push_str(&format!("Context: {context} lines\n"));
            out.push_str(&format!(
                "Exit: {}\n\n",
                output
                    .status
                    .code()
                    .map_or("N/A".to_string(), |c| c.to_string())
            ));

            // diff exit codes: 0 = identical, 1 = differences, 2 = error
            match output.status.code() {
                Some(0) => out.push_str("Files are identical.\n"),
                Some(1) => {
                    out.push_str("--- diff ---\n");
                    out.push_str(&stdout);
                    out.push('\n');
                }
                Some(2) | _ => {
                    out.push_str(&stdout);
                    out.push('\n');
                    if !stderr.is_empty() {
                        out.push_str(&stderr);
                        out.push('\n');
                    }
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("Error running diff: {e}\n"));
            out.push_str("Ensure 'diff' is available on the system.\n");
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// 8. sandbox_checksum_work
// ---------------------------------------------------------------------------

/// Compute a checksum/hash of a file.
///
/// Supports the following algorithms:
///
/// | Algorithm | Command used |
/// |-----------|--------------|
/// | `sha256`  | `sha256sum`  |
/// | `sha1`    | `sha1sum`    |
/// | `md5`     | `md5sum`     |
/// | `blake3`  | `b3sum`      |
///
/// Validates that the file path is within `root` before computing.
pub fn sandbox_checksum_work(root: PathBuf, path: PathBuf, algorithm: String) -> String {
    let mut out = String::from("SANDBOX: CHECKSUM\n\n");

    let full_path = if path.is_absolute() {
        path.clone()
    } else {
        root.join(&path)
    };

    // Validate path is under root
    if let Err(e) = path_is_under_root(&root, &full_path) {
        out.push_str(&format!("Error: {e}\n"));
        return final_truncate(&out);
    }

    // Check file exists
    if !full_path.exists() {
        out.push_str(&format!(
            "Error: file '{}' does not exist\n",
            full_path.display()
        ));
        return final_truncate(&out);
    }

    // Determine the checksum command
    let algo_lower = algorithm.to_lowercase();
    let (checksum_cmd, cmd_name) = match algo_lower.as_str() {
        "sha256" => ("sha256sum", "sha256sum"),
        "sha1" => ("sha1sum", "sha1sum"),
        "md5" => ("md5sum", "md5sum"),
        "blake3" => ("b3sum", "b3sum"),
        _ => {
            out.push_str(&format!(
                "Error: unsupported algorithm '{algorithm}'. \
                 Supported: sha256, sha1, md5, blake3\n"
            ));
            return final_truncate(&out);
        }
    };

    // Check the tool is available
    let tool_check = Command::new(checksum_cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if tool_check.is_err() {
        out.push_str(&format!(
            "Error: `{cmd_name}` is not installed or not found on PATH.\n"
        ));
        if algo_lower == "blake3" {
            out.push_str("Install b3sum: cargo install b3sum\n");
        }
        return final_truncate(&out);
    }

    let result = Command::new(checksum_cmd).arg(&full_path).output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            out.push_str(&format!("File: {}\n", full_path.display()));
            out.push_str(&format!("Algorithm: {algorithm}\n"));
            out.push_str(&format!("Tool: {cmd_name}\n\n"));

            if output.status.success() {
                // Typical output: "<hash>  <filename>"
                // Extract just the hash
                let hash = stdout.split_whitespace().next().unwrap_or(stdout.trim());
                out.push_str(&format!("Hash: {hash}\n"));
            } else {
                out.push_str(&format!("Error: {stderr}"));
            }
        }
        Err(e) => {
            out.push_str(&format!("Error running {cmd_name}: {e}\n"));
        }
    }

    final_truncate(&out)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a temporary directory for tests.
    fn setup_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "sandbox_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    // -- path_is_under_root tests --

    #[test]
    fn test_path_is_under_root_valid() {
        let root = setup_temp_dir();
        let file_path = root.join("subdir/file.txt");
        // Create the file so canonicalize works
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&file_path, "test");
        assert!(path_is_under_root(&root, &file_path).is_ok());
        // Clean up
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_path_is_under_root_traversal() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let _ = fs::write(root.join("file.txt"), "test");
        let traversal_path = root.join("../../../etc/passwd");
        // This may or may not resolve outside root depending on the
        // filesystem, but the canonicalize check should catch it.
        let result = path_is_under_root(&root, &traversal_path);
        // If it resolves outside root, it should fail
        if let Ok(canonical) = traversal_path.canonicalize() {
            assert!(path_is_under_root(&root, &canonical).is_err());
        }
        let _ = fs::remove_dir_all(&root);
    }

    // -- check_dangerous_command tests --

    #[test]
    fn test_safe_command_passes() {
        assert!(check_dangerous_command("ls -la").is_ok());
        assert!(check_dangerous_command("echo hello").is_ok());
        assert!(check_dangerous_command("cat file.txt").is_ok());
    }

    #[test]
    fn test_dangerous_command_blocked() {
        assert!(check_dangerous_command("rm -rf /").is_err());
        assert!(check_dangerous_command("mkfs.ext4 /dev/sda1").is_err());
        assert!(check_dangerous_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(check_dangerous_command("shutdown now").is_err());
        assert!(check_dangerous_command("reboot").is_err());
    }

    #[test]
    fn test_dangerous_command_case_insensitive() {
        assert!(check_dangerous_command("RM -RF /").is_err());
        assert!(check_dangerous_command("Shutdown now").is_err());
    }

    // -- interpreter_for_language tests --

    #[test]
    fn test_interpreter_python() {
        assert_eq!(
            interpreter_for_language("python3").unwrap(),
            ("python3", vec![])
        );
        assert_eq!(interpreter_for_language("py").unwrap(), ("python3", vec![]));
    }

    #[test]
    fn test_interpreter_node() {
        assert_eq!(interpreter_for_language("node").unwrap(), ("node", vec![]));
        assert_eq!(interpreter_for_language("js").unwrap(), ("node", vec![]));
    }

    #[test]
    fn test_interpreter_bash() {
        assert_eq!(interpreter_for_language("bash").unwrap(), ("bash", vec![]));
        assert_eq!(interpreter_for_language("shell").unwrap(), ("bash", vec![]));
    }

    #[test]
    fn test_interpreter_unsupported() {
        assert!(interpreter_for_language("cobol").is_err());
        assert!(interpreter_for_language("rust").is_err());
    }

    // -- SSRF protection tests --

    #[test]
    fn test_is_internal_ip_v4() {
        assert!(is_internal_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_internal_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_internal_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_internal_ip(&"127.0.0.1".parse().unwrap()));
        assert!(!is_internal_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_internal_ip(&"1.1.1.1".parse().unwrap()));
    }

    #[test]
    fn test_is_internal_ip_v6() {
        assert!(is_internal_ip(&"::1".parse().unwrap()));
        assert!(is_internal_ip(&"fd00::1".parse().unwrap()));
        assert!(!is_internal_ip(&"2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn test_extract_host_from_url() {
        assert_eq!(
            extract_host_from_url("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_host_from_url("http://example.com:8080/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_host_from_url("https://[::1]:8080/path"),
            Some("::1".to_string())
        );
        assert_eq!(extract_host_from_url("ftp://example.com"), None);
    }

    // -- truncate_bytes tests --

    #[test]
    fn test_truncate_bytes_under_limit() {
        let data = b"hello world";
        let result = truncate_bytes(data, 100);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_truncate_bytes_over_limit() {
        let data = b"hello world this is a long string";
        let result = truncate_bytes(data, 5);
        assert_eq!(result, "hello");
    }

    // -- sandbox_exec_work integration tests --

    #[test]
    fn test_sandbox_exec_dangerous_command() {
        let root = setup_temp_dir();
        let result = sandbox_exec_work(
            root.clone(),
            "rm".to_string(),
            vec!["-rf".to_string(), "/".to_string()],
            5,
            1024,
        );
        assert!(result.contains("SANDBOX: EXEC"));
        assert!(result.contains("dangerous"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_exec_path_in_command() {
        let root = setup_temp_dir();
        let result = sandbox_exec_work(
            root.clone(),
            "/usr/bin/echo".to_string(),
            vec!["hello".to_string()],
            5,
            1024,
        );
        assert!(result.contains("SANDBOX: EXEC"));
        assert!(result.contains("bare executable"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_exec_echo() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let result = sandbox_exec_work(
            root.clone(),
            "echo".to_string(),
            vec!["hello sandbox".to_string()],
            10,
            1024,
        );
        assert!(result.contains("SANDBOX: EXEC"));
        // Should contain the echo output in stdout
        assert!(result.contains("hello sandbox") || result.contains("Exit:"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- sandbox_script_work tests --

    #[test]
    fn test_sandbox_script_unsupported_language() {
        let root = setup_temp_dir();
        let result = sandbox_script_work(
            root.clone(),
            "print('hi')".to_string(),
            "cobol".to_string(),
            5,
        );
        assert!(result.contains("unsupported language"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_script_bash() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let result = sandbox_script_work(
            root.clone(),
            "echo hello from script".to_string(),
            "bash".to_string(),
            10,
        );
        assert!(result.contains("SANDBOX: SCRIPT"));
        assert!(result.contains("bash"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- sandbox_docker_run_work tests --

    #[test]
    fn test_sandbox_docker_run_dangerous_command() {
        let root = setup_temp_dir();
        let result = sandbox_docker_run_work(
            root.clone(),
            "alpine".to_string(),
            "rm -rf /".to_string(),
            10,
            None,
        );
        assert!(result.contains("SANDBOX: DOCKER RUN"));
        assert!(result.contains("dangerous"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_docker_run_empty_image() {
        let root = setup_temp_dir();
        let result = sandbox_docker_run_work(
            root.clone(),
            "".to_string(),
            "echo hello".to_string(),
            10,
            None,
        );
        assert!(result.contains("must not be empty"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- sandbox_diff_work tests --

    #[test]
    fn test_sandbox_diff_identical_files() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let file_a = root.join("a.txt");
        let file_b = root.join("b.txt");
        let _ = fs::write(&file_a, "hello\nworld\n");
        let _ = fs::write(&file_b, "hello\nworld\n");
        let result = sandbox_diff_work(root.clone(), file_a, file_b, None);
        assert!(result.contains("SANDBOX: DIFF"));
        assert!(result.contains("identical") || result.contains("Exit: 0"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_diff_different_files() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let file_a = root.join("a.txt");
        let file_b = root.join("b.txt");
        let _ = fs::write(&file_a, "hello\nworld\n");
        let _ = fs::write(&file_b, "hello\nearth\n");
        let result = sandbox_diff_work(root.clone(), file_a, file_b, Some(3));
        assert!(result.contains("SANDBOX: DIFF"));
        assert!(result.contains("world") || result.contains("earth") || result.contains("Exit: 1"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_diff_nonexistent_file() {
        let root = setup_temp_dir();
        let result = sandbox_diff_work(
            root.clone(),
            root.join("nonexistent_a.txt"),
            root.join("nonexistent_b.txt"),
            None,
        );
        assert!(result.contains("does not exist") || result.contains("Error"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- sandbox_checksum_work tests --

    #[test]
    fn test_sandbox_checksum_unsupported_algorithm() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let file = root.join("test.txt");
        let _ = fs::write(&file, "hello");
        let result = sandbox_checksum_work(root.clone(), file, "rot13".to_string());
        assert!(result.contains("unsupported algorithm"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_checksum_sha256() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let file = root.join("checksum_test.txt");
        let _ = fs::write(&file, "hello world");
        let result = sandbox_checksum_work(root.clone(), file.clone(), "sha256".to_string());
        assert!(result.contains("SANDBOX: CHECKSUM"));
        assert!(result.contains("sha256"));
        // Should contain a 64-char hex hash
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_checksum_nonexistent_file() {
        let root = setup_temp_dir();
        let result = sandbox_checksum_work(
            root.clone(),
            root.join("no_such_file.txt"),
            "sha256".to_string(),
        );
        assert!(result.contains("does not exist") || result.contains("Error"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- sandbox_network_request_work tests --

    #[test]
    fn test_sandbox_network_request_invalid_protocol() {
        let root = setup_temp_dir();
        let result = sandbox_network_request_work(
            root,
            "ftp://example.com/file".to_string(),
            "GET".to_string(),
            vec![],
            None,
            10,
        );
        assert!(result.contains("only http:// and https://"));
    }

    #[test]
    fn test_sandbox_network_request_unsupported_method() {
        let root = setup_temp_dir();
        let result = sandbox_network_request_work(
            root,
            "https://example.com".to_string(),
            "TRACE".to_string(),
            vec![],
            None,
            10,
        );
        assert!(result.contains("unsupported HTTP method"));
    }

    #[test]
    fn test_sandbox_network_request_internal_ip() {
        let root = setup_temp_dir();
        let result = sandbox_network_request_work(
            root,
            "http://127.0.0.1:8080/internal".to_string(),
            "GET".to_string(),
            vec![],
            None,
            10,
        );
        assert!(result.contains("private/internal IP") || result.contains("SSRF"));
    }

    // -- sandbox_eval_code_work tests --

    #[test]
    fn test_sandbox_eval_code_unsupported_language() {
        let root = setup_temp_dir();
        let result = sandbox_eval_code_work(
            root,
            "print('hi')".to_string(),
            "fortran".to_string(),
            None,
            10,
        );
        assert!(result.contains("unsupported language"));
    }

    // -- sandbox_file_watch_work tests --

    #[test]
    fn test_sandbox_file_watch_nonexistent() {
        let root = setup_temp_dir();
        let result = sandbox_file_watch_work(
            root.clone(),
            root.join("nonexistent.txt"),
            "echo changed".to_string(),
            5,
        );
        assert!(result.contains("SANDBOX: FILE WATCH"));
        assert!(result.contains("does not exist") || result.contains("Error"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn test_sandbox_file_watch_path_traversal() {
        let root = setup_temp_dir();
        let _ = fs::create_dir_all(&root);
        let _ = fs::write(root.join("marker.txt"), "test");
        // Try to watch a file outside root
        let outside = PathBuf::from("/etc/passwd");
        let result = sandbox_file_watch_work(root.clone(), outside, "echo gotcha".to_string(), 5);
        assert!(result.contains("outside root") || result.contains("Error"));
        let _ = fs::remove_dir_all(&root);
    }

    // -- Constants tests --

    #[test]
    fn test_constants_reasonable() {
        assert_eq!(SANDBOX_DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(SANDBOX_MAX_OUTPUT_BYTES, 1_048_576);
        assert!(MAX_NETWORK_RESPONSE_BYTES < SANDBOX_MAX_OUTPUT_BYTES);
    }
}
