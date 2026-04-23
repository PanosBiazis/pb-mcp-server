//! # System & Environment Tools
//!
//! Process listing, disk usage, network interfaces, and environment variable access.

use std::env;
use std::path::PathBuf;
use std::process::Command;

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
        Command::new("ps")
            .args(["aux", "--sort=-%cpu"])
            .output()
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
                out.push_str(&format!("Filesystem info for {}:\n{}\n", path.display(), stdout));
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
            let output = Command::new("dir")
                .arg(&path)
                .output();
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
            out.push_str("Ensure 'ip' (Linux), 'ifconfig' (macOS), or 'ipconfig' (Windows) is available.\n");
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
/// Refuses to return values for keys containing sensitive keywords:
/// SECRET, PASSWORD, TOKEN, KEY, CREDENTIAL, PRIVATE (case-insensitive).
pub fn system_env_get_work(key: String) -> String {
    let mut out = String::from("SYSTEM: ENV GET\n\n");
    out.push_str(&format!("Key: {key}\n"));

    // Check for sensitive key names
    let upper = key.to_uppercase();
    let sensitive_keywords = [
        "SECRET",
        "PASSWORD",
        "TOKEN",
        "KEY",
        "CREDENTIAL",
        "PRIVATE",
        "AUTH",
        "SIGNING",
    ];

    let is_sensitive = sensitive_keywords
        .iter()
        .any(|kw| upper.contains(kw));

    if is_sensitive {
        out.push_str("Value: [REDACTED — key name suggests sensitive content]\n");
        out.push_str("This tool refuses to display values for keys containing:\n");
        out.push_str("SECRET, PASSWORD, TOKEN, KEY, CREDENTIAL, PRIVATE, AUTH, SIGNING\n");
        return out;
    }

    match env::var(&key) {
        Ok(value) => {
            let value_len = value.len();
            out.push_str(&format!("Value: {}\n", value));
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
