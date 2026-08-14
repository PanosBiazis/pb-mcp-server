//! # IDE Integration Tools
//!
//! Integration helpers for Zed, VS Code, Cursor, JetBrains / Android Studio,
//! and generic IDE workflows.  Each public `_work` function accepts
//! `serde_json::Value` args and returns `serde_json::Value`, using
//! `serde_json::json!()` for construction and `std::process::Command` for
//! external tooling.  When an IDE CLI is not found the functions return
//! `json!({"error": "IDE not found", "tool_available": false})`.

use serde_json::{json, Value};
use std::process::Command;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the standard "IDE not found" error payload.
fn ide_not_found() -> Value {
    json!({"error": "IDE not found", "tool_available": false})
}

/// Check whether a command exists on `$PATH`.
#[cfg(not(test))]
fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Unit tests must never launch host IDEs or attach to physical Android devices.
// Live integration lanes exercise the normal implementation above.
#[cfg(test)]
fn which(_cmd: &str) -> bool {
    false
}

/// Extract a string field from a JSON value, defaulting to `default`.
fn get_str<'a>(args: &'a Value, key: &str, default: &'a str) -> &'a str {
    args.get(key).and_then(|v| v.as_str()).unwrap_or(default)
}

/// Extract an integer field from a JSON value, defaulting to `default`.
fn get_int(args: &Value, key: &str, default: i64) -> i64 {
    args.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// Run a command and return its exit success status as a JSON object
/// with key `success`.
fn run_and_success(cmd: &str, args: &[&str]) -> Value {
    match Command::new(cmd).args(args).status() {
        Ok(s) => json!({"success": s.success()}),
        Err(_) => ide_not_found(),
    }
}

// ===========================================================================
// Zed Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. zed_open_file
// ---------------------------------------------------------------------------

/// Open a file in the Zed editor, optionally at a specific line.
///
/// Runs `zed <path>:<line>`.
pub fn zed_open_file_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let line = get_int(&args, "line", 0);

    if !which("zed") {
        return ide_not_found();
    }

    let target = if line > 0 {
        format!("{}:{}", path, line)
    } else {
        path.to_string()
    };

    match Command::new("zed").arg(&target).spawn() {
        Ok(_) => json!({"success": true}),
        Err(_) => json!({"success": false, "error": "failed to launch zed"}),
    }
}

// ---------------------------------------------------------------------------
// 2. zed_get_diagnostics
// ---------------------------------------------------------------------------

/// Placeholder for Zed diagnostics retrieval.
///
/// Zed does not yet expose diagnostics via CLI; the recommendation is to use
/// the LSP bridge for live diagnostics.
pub fn zed_get_diagnostics_work(args: Value) -> Value {
    let _path = get_str(&args, "path", "");
    json!({
        "diagnostics": [],
        "note": "Use LSP bridge for live diagnostics"
    })
}

// ---------------------------------------------------------------------------
// 3. zed_search_workspace
// ---------------------------------------------------------------------------

/// Trigger a workspace search via the Zed CLI.
///
/// Runs `zed --search <query>`.
pub fn zed_search_workspace_work(args: Value) -> Value {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return json!({"error": "missing 'query' argument", "tool_available": false}),
    };

    if !which("zed") {
        return ide_not_found();
    }

    match Command::new("zed").args(["--search", query]).spawn() {
        Ok(_) => json!({"results": [], "note": "Workspace search triggered"}),
        Err(_) => json!({"results": [], "note": "Failed to trigger workspace search"}),
    }
}

// ===========================================================================
// VS Code Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 4. vscode_open_diff
// ---------------------------------------------------------------------------

/// Open a diff view in VS Code.
///
/// Runs `code --diff <left> <right>`.
pub fn vscode_open_diff_work(args: Value) -> Value {
    let left = match args.get("left").and_then(|v| v.as_str()) {
        Some(l) => l,
        None => return json!({"error": "missing 'left' argument", "tool_available": false}),
    };
    let right = match args.get("right").and_then(|v| v.as_str()) {
        Some(r) => r,
        None => return json!({"error": "missing 'right' argument", "tool_available": false}),
    };

    if !which("code") {
        return ide_not_found();
    }

    run_and_success("code", &["--diff", left, right])
}

// ---------------------------------------------------------------------------
// 5. vscode_open_file
// ---------------------------------------------------------------------------

/// Open a file in VS Code, optionally jumping to a specific line.
///
/// Runs `code --goto <path>:<line>`.
pub fn vscode_open_file_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let line = get_int(&args, "line", 0);

    if !which("code") {
        return ide_not_found();
    }

    let target = if line > 0 {
        format!("{}:{}", path, line)
    } else {
        path.to_string()
    };

    run_and_success("code", &["--goto", &target])
}

// ---------------------------------------------------------------------------
// 6. vscode_list_extensions
// ---------------------------------------------------------------------------

/// List installed VS Code extensions.
///
/// Runs `code --list-extensions --show-names` and parses the output into an
/// array of `{id, name}` objects.
pub fn vscode_list_extensions_work(args: Value) -> Value {
    let _ = args; // no arguments needed

    if !which("code") {
        return json!({
            "extensions": [],
            "error": "VS Code CLI not found",
            "tool_available": false
        });
    }

    match Command::new("code")
        .args(["--list-extensions", "--show-names"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let extensions: Vec<Value> = stdout
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() {
                        return None;
                    }
                    // Format: <extension.id>\t<Display Name>
                    let mut parts = line.splitn(2, '\t');
                    let id = parts.next().unwrap_or("").to_string();
                    let name = parts.next().unwrap_or(id.as_str()).to_string();
                    Some(json!({"id": id, "name": name}))
                })
                .collect();
            json!({"extensions": extensions})
        }
        Err(_) => json!({"extensions": [], "error": "failed to list extensions"}),
    }
}

// ---------------------------------------------------------------------------
// 7. vscode_install_extension
// ---------------------------------------------------------------------------

/// Install a VS Code extension by its identifier.
///
/// Runs `code --install-extension <extension_id>`.
pub fn vscode_install_extension_work(args: Value) -> Value {
    let extension_id = match args.get("extension_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => {
            return json!({
                "error": "missing 'extension_id' argument",
                "tool_available": false
            })
        }
    };

    if !which("code") {
        return ide_not_found();
    }

    run_and_success("code", &["--install-extension", extension_id])
}

// ===========================================================================
// Cursor Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 8. cursor_open_file
// ---------------------------------------------------------------------------

/// Open a file in the Cursor editor, optionally at a specific line.
///
/// Runs `cursor <path>:<line>`.
pub fn cursor_open_file_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let line = get_int(&args, "line", 0);

    if !which("cursor") {
        return ide_not_found();
    }

    let target = if line > 0 {
        format!("{}:{}", path, line)
    } else {
        path.to_string()
    };

    match Command::new("cursor").arg(&target).spawn() {
        Ok(_) => json!({"success": true}),
        Err(_) => json!({"success": false, "error": "failed to launch cursor"}),
    }
}

// ===========================================================================
// JetBrains / Android Studio Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 9. jetbrains_open_file
// ---------------------------------------------------------------------------

/// Open a file in a JetBrains IDE, optionally at a specific line.
///
/// Valid `ide` values: `"idea"`, `"android-studio"`, `"clion"`, `"pycharm"`,
/// `"rustrover"`.
///
/// Runs `<ide> <path>:<line>`.
pub fn jetbrains_open_file_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let ide = match args.get("ide").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return json!({"error": "missing 'ide' argument", "tool_available": false}),
    };
    let line = get_int(&args, "line", 0);

    let allowed = ["idea", "android-studio", "clion", "pycharm", "rustrover"];
    if !allowed.contains(&ide) {
        return json!({
            "error": format!("unsupported IDE '{}'; choose from {:?}", ide, allowed),
            "tool_available": false,
        });
    }

    if !which(ide) {
        return ide_not_found();
    }

    let target = if line > 0 {
        format!("{}:{}", path, line)
    } else {
        path.to_string()
    };

    match Command::new(ide).arg(&target).spawn() {
        Ok(_) => json!({"success": true}),
        Err(_) => json!({"success": false, "error": format!("failed to launch {}", ide)}),
    }
}

// ---------------------------------------------------------------------------
// 10. jetbrains_adb_shell
// ---------------------------------------------------------------------------

/// Execute an ADB shell command, optionally targeting a specific device.
///
/// Runs `adb [-s <device>] shell <command>`.
pub fn jetbrains_adb_shell_work(args: Value) -> Value {
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return json!({"error": "missing 'command' argument", "tool_available": false}),
    };
    let device = args.get("device").and_then(|v| v.as_str());

    if !which("adb") {
        return ide_not_found();
    }

    let mut cmd = Command::new("adb");
    if let Some(d) = device {
        cmd.args(["-s", d]);
    }
    cmd.args(["shell", command]);

    match cmd.output() {
        Ok(output) => json!({
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            "exit_code": output.status.code().unwrap_or(-1),
        }),
        Err(_) => json!({
            "stdout": "",
            "stderr": "failed to execute adb",
            "exit_code": -1,
        }),
    }
}

// ---------------------------------------------------------------------------
// 11. jetbrains_device_list
// ---------------------------------------------------------------------------

/// List connected Android devices via ADB.
///
/// Runs `adb devices -l` and parses the output into structured records.
pub fn jetbrains_device_list_work(args: Value) -> Value {
    let _ = args;

    if !which("adb") {
        return ide_not_found();
    }

    match Command::new("adb").args(["devices", "-l"]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut devices: Vec<Value> = Vec::new();

            for line in stdout.lines().skip(1) {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                let serial = parts.first().unwrap_or(&"").to_string();
                let rest = parts.get(1).unwrap_or(&"");

                // State is the first token after the serial
                let state = rest
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();

                // Try to extract model from "model:<value>" token
                let model = rest
                    .split_whitespace()
                    .find(|t| t.starts_with("model:"))
                    .map(|t| t.strip_prefix("model:").unwrap_or(t).to_string())
                    .unwrap_or_default();

                devices.push(json!({
                    "serial": serial,
                    "state": state,
                    "model": model,
                }));
            }

            json!({"devices": devices})
        }
        Err(_) => json!({"devices": [], "error": "failed to run adb"}),
    }
}

// ---------------------------------------------------------------------------
// 12. jetbrains_logcat_tail
// ---------------------------------------------------------------------------

/// Read the last *n* lines of Android Logcat, optionally filtered and
/// targeting a specific device.
///
/// Runs `adb [-s <device>] logcat -d -t <n>`.
pub fn jetbrains_logcat_tail_work(args: Value) -> Value {
    let device = args.get("device").and_then(|v| v.as_str());
    let n = get_int(&args, "n", 100).clamp(1, 10_000);
    let _filter = args.get("filter").and_then(|v| v.as_str());

    if !which("adb") {
        return ide_not_found();
    }

    let mut cmd = Command::new("adb");
    if let Some(d) = device {
        cmd.args(["-s", d]);
    }
    cmd.args(["logcat", "-d", "-t", &n.to_string()]);

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
            let count = lines.len();
            json!({"lines": lines, "count": count})
        }
        Err(_) => json!({"lines": [], "count": 0, "error": "failed to run adb logcat"}),
    }
}

// ===========================================================================
// Generic IDE Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 13. ide_detect
// ---------------------------------------------------------------------------

/// Detect running IDEs by scanning `/proc` on Linux.
///
/// Looks for processes whose command lines match known IDE names.
pub fn ide_detect_work(args: Value) -> Value {
    let _ = args;

    let known_ides = [
        "zed",
        "code",
        "cursor",
        "idea",
        "android-studio",
        "clion",
        "pycharm",
        "rustrover",
        "webstorm",
        "goland",
        "vim",
        "nvim",
        "emacs",
        "sublime_text",
    ];

    let mut running: Vec<Value> = Vec::new();

    if cfg!(target_os = "linux") {
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let pid: u32 = match name.to_str().and_then(|s| s.parse::<u32>().ok()) {
                    Some(p) => p,
                    None => continue,
                };

                let cmdline_path = format!("/proc/{}/cmdline", pid);
                if let Ok(data) = std::fs::read(&cmdline_path) {
                    // cmdline is null-separated; join with space for searching
                    let cmdline = data
                        .split(|&b| b == 0)
                        .filter_map(|b| std::str::from_utf8(b).ok())
                        .collect::<Vec<_>>()
                        .join(" ");

                    for ide in &known_ides {
                        if cmdline.contains(ide) {
                            running.push(json!({"name": ide, "pid": pid}));
                            break; // one match per process is enough
                        }
                    }
                }
            }
        }
    }

    json!({"running_ides": running})
}

// ---------------------------------------------------------------------------
// 14. ide_format_document
// ---------------------------------------------------------------------------

/// Format a document using the specified formatter.
///
/// Valid `formatter` values: `"rustfmt"`, `"prettier"`, `"black"`,
/// `"clang-format"`.
///
/// Runs the formatter on the given `path`.
pub fn ide_format_document_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let formatter = match args.get("formatter").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return json!({"error": "missing 'formatter' argument", "tool_available": false}),
    };

    let allowed = ["rustfmt", "prettier", "black", "clang-format"];
    if !allowed.contains(&formatter) {
        return json!({
            "error": format!("unsupported formatter '{}'; choose from {:?}", formatter, allowed),
            "tool_available": false,
        });
    }

    if !which(formatter) {
        return json!({"error": "IDE not found", "tool_available": false});
    }

    let success = match formatter {
        "rustfmt" => Command::new("rustfmt").arg(path).status(),
        "prettier" => Command::new("prettier").args(["--write", path]).status(),
        "black" => Command::new("black").arg(path).status(),
        "clang-format" => Command::new("clang-format").args(["-i", path]).status(),
        _ => return json!({"formatted": false, "formatter_used": formatter}),
    };

    match success {
        Ok(s) => json!({"formatted": s.success(), "formatter_used": formatter}),
        Err(_) => json!({"formatted": false, "formatter_used": formatter}),
    }
}

// ---------------------------------------------------------------------------
// 15. ide_run_test
// ---------------------------------------------------------------------------

/// Run tests for the specified file / language.
///
/// For Rust files, runs `cargo test`. Returns `{success, output}`.
pub fn ide_run_test_work(args: Value) -> Value {
    let path = get_str(&args, "path", "");
    let _line = get_int(&args, "line", 0);
    let language = match args.get("language").and_then(|v| v.as_str()) {
        Some(l) => l,
        None => return json!({"error": "missing 'language' argument", "tool_available": false}),
    };

    let (cmd_name, cmd_args): (&str, Vec<&str>) = match language.to_lowercase().as_str() {
        "rust" => ("cargo", vec!["test"]),
        "python" | "py" => ("pytest", vec![path]),
        "javascript" | "typescript" | "js" | "ts" => ("npx", vec!["jest", path]),
        _ => {
            return json!({
                "success": false,
                "output": format!("unsupported language: {}", language),
            })
        }
    };

    if !which(cmd_name) {
        return json!({
            "success": false,
            "output": format!("{} not found on PATH", cmd_name),
        });
    }

    match Command::new(cmd_name).args(&cmd_args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = format!("{}\n{}", stdout, stderr);
            json!({
                "success": output.status.success(),
                "output": combined,
            })
        }
        Err(e) => json!({
            "success": false,
            "output": format!("failed to execute {}: {}", cmd_name, e),
        }),
    }
}

// ---------------------------------------------------------------------------
// 16. ide_goto_definition
// ---------------------------------------------------------------------------

/// Find a symbol's definition via `grep -rn`.
///
/// Searches for the symbol under `path` and returns matching locations.
pub fn ide_goto_definition_work(args: Value) -> Value {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "missing 'symbol' argument", "tool_available": false}),
    };
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };

    if !which("grep") {
        return json!({"definitions": [], "error": "grep not found"});
    }

    match Command::new("grep").args(["-rn", symbol, path]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let definitions: Vec<Value> = stdout
                .lines()
                .take(50)
                .filter_map(|line| {
                    // Format: <file>:<line>:<context>
                    let mut parts = line.splitn(3, ':');
                    let file = parts.next().unwrap_or("").to_string();
                    let line_num = parts.next().unwrap_or("0");
                    let context = parts.next().unwrap_or("").trim().to_string();
                    if file.is_empty() {
                        None
                    } else {
                        Some(json!({
                            "file": file,
                            "line": line_num.parse::<u64>().unwrap_or(0),
                            "context": context,
                        }))
                    }
                })
                .collect();
            json!({"definitions": definitions})
        }
        Err(_) => json!({"definitions": [], "error": "failed to execute grep"}),
    }
}

// ---------------------------------------------------------------------------
// 17. ide_find_references
// ---------------------------------------------------------------------------

/// Find references to a symbol via `grep -rn`.
///
/// Searches for the symbol under `path` and returns matching locations plus
/// a count.
pub fn ide_find_references_work(args: Value) -> Value {
    let symbol = match args.get("symbol").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "missing 'symbol' argument", "tool_available": false}),
    };
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };

    if !which("grep") {
        return json!({"references": [], "count": 0, "error": "grep not found"});
    }

    match Command::new("grep").args(["-rn", symbol, path]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let references: Vec<Value> = stdout
                .lines()
                .take(50)
                .filter_map(|line| {
                    let mut parts = line.splitn(3, ':');
                    let file = parts.next().unwrap_or("").to_string();
                    let line_num = parts.next().unwrap_or("0");
                    let context = parts.next().unwrap_or("").trim().to_string();
                    if file.is_empty() {
                        None
                    } else {
                        Some(json!({
                            "file": file,
                            "line": line_num.parse::<u64>().unwrap_or(0),
                            "context": context,
                        }))
                    }
                })
                .collect();
            let count = references.len();
            json!({"references": references, "count": count})
        }
        Err(_) => json!({"references": [], "count": 0, "error": "failed to execute grep"}),
    }
}

// ---------------------------------------------------------------------------
// 18. zed_context_attach
// ---------------------------------------------------------------------------

/// Attach a file to Zed AI session context.
///
/// Returns instructions for Zed to include the file content in its AI context.
/// When Zed is available, attempts to open the file; otherwise returns
/// copy-paste instructions.
pub fn zed_context_attach_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let context_role = get_str(&args, "role", "reference");

    if which("zed") {
        // Best-effort: open the file in Zed so it's in the active buffer
        let _ = Command::new("zed").arg(path).spawn();
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => {
            let max_chars = 8000;
            if c.chars().count() > max_chars {
                let truncated: String = c.chars().take(max_chars).collect();
                json!({
                    "attached": true,
                    "path": path,
                    "role": context_role,
                    "content_preview": truncated,
                    "truncated": true,
                    "instructions": format!("In Zed AI Assistant panel, use @{} to reference this file. The file has been opened in your editor.", path)
                })
            } else {
                json!({
                    "attached": true,
                    "path": path,
                    "role": context_role,
                    "content": c,
                    "truncated": false,
                    "instructions": format!("In Zed AI Assistant panel, use @{} to reference this file.", path)
                })
            }
        }
        Err(e) => json!({
            "attached": false,
            "path": path,
            "role": context_role,
            "error": format!("Could not read file: {}", e),
            "instructions": format!("Manually open {} in Zed and use @{} in the AI Assistant panel.", path, path)
        }),
    };

    content
}

// ---------------------------------------------------------------------------
// 19. cursor_composer_send
// ---------------------------------------------------------------------------

/// Send content to Cursor Composer panel.
///
/// Returns instructions for Cursor to process the content in its AI Composer.
/// When Cursor is available, attempts to open the file; otherwise returns
/// copy-paste instructions.
pub fn cursor_composer_send_work(args: Value) -> Value {
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return json!({"error": "missing 'content' argument", "tool_available": false}),
    };
    let path = get_str(&args, "path", "");
    let instruction = get_str(
        &args,
        "instruction",
        "Review and improve the following code",
    );

    if which("cursor") {
        // Open the file in Cursor so it's in the active editor
        if !path.is_empty() {
            let _ = Command::new("cursor").arg(path).spawn();
        }
    }

    json!({
        "sent": true,
        "content_preview": content.chars().take(2000).collect::<String>(),
        "instruction": instruction,
        "path": if path.is_empty() { Value::Null } else { json!(path) },
        "instructions": format!(
            "In Cursor Composer (Ctrl+I / Cmd+I), paste the following instruction:\n\n{}\n\nThen paste or reference the code content.",
            instruction
        ),
        "tool_available": which("cursor")
    })
}

// ---------------------------------------------------------------------------
// 20. vscode_get_diagnostics
// ---------------------------------------------------------------------------

/// Read VS Code diagnostics via CLI or file.
///
/// Attempts to read VS Code's saved diagnostics data, or uses `code` CLI
/// to list problems. Returns structured diagnostic records.
pub fn vscode_get_diagnostics_work(args: Value) -> Value {
    let path = get_str(&args, "path", "");
    let _severity = get_str(&args, "severity", "all"); // "all", "error", "warning"

    if !which("code") {
        return json!({
            "diagnostics": [],
            "tool_available": false,
            "error": "VS Code CLI ('code') not found on PATH",
            "fallback": "Open VS Code Problems panel (Ctrl+Shift+M) for live diagnostics"
        });
    }

    // Try to use `code` CLI to get diagnostics output
    // VS Code doesn't have a direct CLI for diagnostics, so we provide guidance
    // and attempt to read from a stored location if available
    let mut diagnostics: Vec<Value> = Vec::new();

    // Check for a VS Code workspace diagnostics cache (best-effort)
    if !path.is_empty() {
        let abs_path = std::path::Path::new(path);
        if abs_path.exists() {
            // Try running code --status which shows some workspace info
            if let Ok(output) = Command::new("code").args(["--status"]).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse any relevant error/warning lines
                for line in stdout.lines().take(100) {
                    if line.contains("error") || line.contains("warning") || line.contains("Error")
                    {
                        diagnostics.push(json!({
                            "message": line.trim().to_string(),
                            "source": "vscode-status"
                        }));
                    }
                }
            }
        }
    }

    json!({
        "diagnostics": diagnostics,
        "path": path,
        "tool_available": true,
        "note": "VS Code does not expose diagnostics via CLI directly. For live diagnostics, use the Problems panel (Ctrl+Shift+M) or the LSP bridge."
    })
}

// ---------------------------------------------------------------------------
// 21. jetbrains_lsp_code_action
// ---------------------------------------------------------------------------

/// Execute an LSP code action via JetBrains gateway.
///
/// Uses the JetBrains IDE's REST API or CLI to request a code action
/// (quick fix, refactoring, etc.) at a specific position in a file.
/// Returns instructions for the action.
pub fn jetbrains_lsp_code_action_work(args: Value) -> Value {
    let path = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "missing 'path' argument", "tool_available": false}),
    };
    let ide = match args.get("ide").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return json!({"error": "missing 'ide' argument", "tool_available": false}),
    };
    let line = get_int(&args, "line", 1);
    let column = get_int(&args, "column", 1);
    let action_kind = get_str(&args, "action_kind", "quickfix"); // "quickfix", "refactor", "source"

    let allowed = [
        "idea",
        "android-studio",
        "clion",
        "pycharm",
        "rustrover",
        "webstorm",
        "goland",
    ];
    if !allowed.contains(&ide) {
        return json!({
            "error": format!("unsupported IDE '{}'; choose from {:?}", ide, allowed),
            "tool_available": false,
        });
    }

    if !which(ide) {
        return ide_not_found();
    }

    // JetBrains IDEs expose a built-in development server on port 63342 (idea)
    // or 6942 (some others). We try the REST API for code actions.
    let port = get_int(&args, "port", 63342);
    let url = format!(
        "http://localhost:{}/api/codeAction?file={}&line={}&column={}&kind={}",
        port,
        urlencoding(path),
        line,
        column,
        action_kind
    );

    // Attempt the REST call via curl
    let result = if which("curl") {
        match Command::new("curl")
            .args(["-s", "-f", "--max-time", "5", &url])
            .output()
        {
            Ok(output) => {
                let body = String::from_utf8_lossy(&output.stdout).to_string();
                if body.is_empty() {
                    json!({
                        "actions": [],
                        "status": "no_response",
                        "ide": ide,
                        "port": port,
                    })
                } else {
                    json!({
                        "actions": body,
                        "status": "ok",
                        "ide": ide,
                        "port": port,
                    })
                }
            }
            Err(e) => json!({
                "actions": [],
                "status": "error",
                "error": format!("curl failed: {}", e),
                "ide": ide,
            }),
        }
    } else {
        json!({
            "actions": [],
            "status": "curl_unavailable",
            "ide": ide,
        })
    };

    json!({
        "path": path,
        "line": line,
        "column": column,
        "action_kind": action_kind,
        "ide": ide,
        "result": result,
        "instructions": format!(
            "In {}, open {} at line {} and press Alt+Enter to see available code actions.",
            ide, path, line
        ),
        "tool_available": true,
    })
}

/// Minimal URL-encoding for path segments.
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '/' => "%2F".to_string(),
            ':' => "%3A".to_string(),
            '#' => "%23".to_string(),
            '?' => "%3F".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 22. jetbrains_run_configuration
// ---------------------------------------------------------------------------

/// Trigger a JetBrains run configuration.
///
/// Uses the JetBrains IDE CLI or REST API to execute a saved run/debug
/// configuration. Returns the result of the attempt.
pub fn jetbrains_run_configuration_work(args: Value) -> Value {
    let ide = match args.get("ide").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return json!({"error": "missing 'ide' argument", "tool_available": false}),
    };
    let config_name = match args.get("config_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return json!({"error": "missing 'config_name' argument", "tool_available": false}),
    };
    let debug = args.get("debug").and_then(|v| v.as_bool()).unwrap_or(false);

    let allowed = [
        "idea",
        "android-studio",
        "clion",
        "pycharm",
        "rustrover",
        "webstorm",
        "goland",
    ];
    if !allowed.contains(&ide) {
        return json!({
            "error": format!("unsupported IDE '{}'; choose from {:?}", ide, allowed),
            "tool_available": false,
        });
    }

    if !which(ide) {
        return ide_not_found();
    }

    // JetBrains IDEs support running configurations via CLI:
    // <ide> run <config_name>  or  <ide> debug <config_name>
    // Some versions also support: <ide> . --run <config_name>
    let subcmd = if debug { "debug" } else { "run" };

    let result = match Command::new(ide).args([subcmd, config_name]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            json!({
                "success": output.status.success(),
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code().unwrap_or(-1),
            })
        }
        Err(e) => json!({
            "success": false,
            "error": format!("failed to execute {} {}: {}", ide, subcmd, e),
        }),
    };

    json!({
        "ide": ide,
        "config_name": config_name,
        "debug": debug,
        "mode": subcmd,
        "result": result,
        "tool_available": true,
        "instructions": format!(
            "Alternatively, in {}, use Run > Run '{}' (or Shift+F10) to run this configuration manually.",
            ide, config_name
        ),
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    // -- Zed ----------------------------------------------------------------

    #[test]
    fn test_zed_open_file_missing_path() {
        let result = zed_open_file_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_zed_get_diagnostics_returns_placeholder() {
        let result = zed_get_diagnostics_work(json!({"path": "/tmp/test.rs"}));
        assert!(result["diagnostics"].is_array());
        assert_eq!(result["note"], "Use LSP bridge for live diagnostics");
    }

    #[test]
    fn test_zed_search_workspace_missing_query() {
        let result = zed_search_workspace_work(json!({}));
        assert!(result.get("error").is_some());
    }

    // -- VS Code ------------------------------------------------------------

    #[test]
    fn test_vscode_open_diff_missing_args() {
        let result = vscode_open_diff_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_vscode_open_file_missing_path() {
        let result = vscode_open_file_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_vscode_list_extensions_no_args() {
        // Just ensures it doesn't panic; may or may not find `code`
        let result = vscode_list_extensions_work(json!({}));
        assert!(result["extensions"].is_array());
    }

    #[test]
    fn test_vscode_install_extension_missing_id() {
        let result = vscode_install_extension_work(json!({}));
        assert!(result.get("error").is_some());
    }

    // -- Cursor -------------------------------------------------------------

    #[test]
    fn test_cursor_open_file_missing_path() {
        let result = cursor_open_file_work(json!({}));
        assert!(result.get("error").is_some());
    }

    // -- JetBrains ----------------------------------------------------------

    #[test]
    fn test_jetbrains_open_file_missing_path() {
        let result = jetbrains_open_file_work(json!({"ide": "idea"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_open_file_unsupported_ide() {
        let result = jetbrains_open_file_work(json!({"path": "/tmp/a.rs", "ide": "vim"}));
        assert!(result.get("error").is_some());
        assert!(!result["tool_available"].as_bool().unwrap_or(true));
    }

    #[test]
    fn test_jetbrains_adb_shell_missing_command() {
        let result = jetbrains_adb_shell_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_device_list_returns_array() {
        let result = jetbrains_device_list_work(json!({}));
        assert!(result["devices"].is_array() || result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_logcat_tail_defaults() {
        let result = jetbrains_logcat_tail_work(json!({}));
        assert!(result["lines"].is_array() || result.get("error").is_some());
    }

    // -- Generic IDE --------------------------------------------------------

    #[test]
    fn test_ide_detect_returns_array() {
        let result = ide_detect_work(json!({}));
        assert!(result["running_ides"].is_array());
    }

    #[test]
    fn test_ide_format_document_missing_path() {
        let result = ide_format_document_work(json!({"formatter": "rustfmt"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ide_format_document_unsupported_formatter() {
        let result = ide_format_document_work(json!({"path": "/tmp/a.rs", "formatter": "goofmt"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ide_run_test_missing_language() {
        let result = ide_run_test_work(json!({"path": "/tmp/a.rs"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ide_run_test_unsupported_language() {
        let result = ide_run_test_work(json!({"path": "/tmp/a.rs", "language": "cobol"}));
        assert!(!result["success"].as_bool().unwrap_or(true));
    }

    #[test]
    fn test_ide_goto_definition_missing_symbol() {
        let result = ide_goto_definition_work(json!({"path": "/tmp"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ide_find_references_missing_symbol() {
        let result = ide_find_references_work(json!({"path": "/tmp"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ide_find_references_missing_path() {
        let result = ide_find_references_work(json!({"symbol": "main"}));
        assert!(result.get("error").is_some());
    }

    // -- Helper tests -------------------------------------------------------

    #[test]
    fn test_ide_not_found_helper() {
        let v = ide_not_found();
        assert_eq!(v["error"], "IDE not found");
        assert!(!v["tool_available"].as_bool().unwrap());
    }

    #[test]
    fn test_get_str_default() {
        let v = json!({});
        assert_eq!(get_str(&v, "missing", "fallback"), "fallback");
    }

    #[test]
    fn test_get_int_default() {
        let v = json!({});
        assert_eq!(get_int(&v, "missing", 42), 42);
    }

    #[test]
    fn test_zed_open_file_with_line() {
        // We cannot guarantee `zed` exists, so we just ensure the function
        // handles the line arg gracefully without panic.
        let result = zed_open_file_work(json!({"path": "/tmp/test.rs", "line": 10}));
        // Either success or ide_not_found – both are valid outcomes
        assert!(result.get("success").is_some() || result.get("error").is_some());
    }

    #[test]
    fn test_vscode_open_diff_with_args() {
        let result = vscode_open_diff_work(json!({"left": "/tmp/a.rs", "right": "/tmp/b.rs"}));
        assert!(result.get("success").is_some() || result.get("error").is_some());
    }

    #[test]
    fn test_vscode_open_file_with_line() {
        let result = vscode_open_file_work(json!({"path": "/tmp/a.rs", "line": 5}));
        assert!(result.get("success").is_some() || result.get("error").is_some());
    }

    #[test]
    fn test_cursor_open_file_with_line() {
        let result = cursor_open_file_work(json!({"path": "/tmp/a.rs", "line": 3}));
        assert!(result.get("success").is_some() || result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_open_file_with_line() {
        let result =
            jetbrains_open_file_work(json!({"path": "/tmp/a.rs", "ide": "idea", "line": 7}));
        assert!(result.get("success").is_some() || result.get("error").is_some());
    }

    #[test]
    fn test_ide_format_document_with_rustfmt() {
        let result = ide_format_document_work(json!({"path": "/tmp/a.rs", "formatter": "rustfmt"}));
        // rustfmt may or may not be installed
        assert!(result.get("formatted").is_some() || result.get("error").is_some());
    }

    // -- New IDE tools (18-22) ---------------------------------------------

    #[test]
    fn test_zed_context_attach_missing_path() {
        let result = zed_context_attach_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_zed_context_attach_with_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("test_ctx.rs");
        fs::write(&file_path, "fn main() {}\n").unwrap();
        let result = zed_context_attach_work(json!({
            "path": file_path.to_str().unwrap(),
            "role": "reference"
        }));
        assert!(result["attached"].as_bool().unwrap_or(false));
        assert_eq!(result["role"], "reference");
        assert!(result["instructions"].as_str().unwrap_or("").contains("@"));
    }

    #[test]
    fn test_zed_context_attach_nonexistent_file() {
        let result = zed_context_attach_work(json!({
            "path": "/nonexistent/file.rs",
            "role": "reference"
        }));
        assert!(!result["attached"].as_bool().unwrap_or(true));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_cursor_composer_send_missing_content() {
        let result = cursor_composer_send_work(json!({}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_cursor_composer_send_with_content() {
        let result = cursor_composer_send_work(json!({
            "content": "fn hello() { println!(\"hi\"); }",
            "instruction": "Add error handling"
        }));
        assert!(result["sent"].as_bool().unwrap_or(false));
        assert_eq!(result["instruction"], "Add error handling");
        assert!(result["instructions"]
            .as_str()
            .unwrap_or("")
            .contains("Composer"));
    }

    #[test]
    fn test_vscode_get_diagnostics_returns_structure() {
        let result = vscode_get_diagnostics_work(json!({"path": "/tmp/test.rs"}));
        assert!(result["diagnostics"].is_array());
    }

    #[test]
    fn test_vscode_get_diagnostics_missing_ide() {
        // This test just ensures no panic; code may or may not be on PATH
        let result = vscode_get_diagnostics_work(json!({}));
        assert!(result["diagnostics"].is_array());
    }

    #[test]
    fn test_jetbrains_lsp_code_action_missing_path() {
        let result = jetbrains_lsp_code_action_work(json!({"ide": "idea"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_lsp_code_action_missing_ide() {
        let result = jetbrains_lsp_code_action_work(json!({"path": "/tmp/a.rs"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_lsp_code_action_unsupported_ide() {
        let result = jetbrains_lsp_code_action_work(json!({
            "path": "/tmp/a.rs",
            "ide": "vim",
            "line": 1,
            "column": 1
        }));
        assert!(result.get("error").is_some());
        assert!(!result["tool_available"].as_bool().unwrap_or(true));
    }

    #[test]
    fn test_jetbrains_lsp_code_action_with_valid_args() {
        let result = jetbrains_lsp_code_action_work(json!({
            "path": "/tmp/a.rs",
            "ide": "idea",
            "line": 5,
            "column": 10,
            "action_kind": "quickfix"
        }));
        // IDEA may or may not be installed; if not, we get a tool_available=false response
        if result.get("tool_available") == Some(&json!(false)) {
            assert!(result.get("error").is_some());
        } else {
            assert_eq!(result["path"], "/tmp/a.rs");
            assert_eq!(result["line"], 5);
            assert_eq!(result["column"], 10);
            assert_eq!(result["action_kind"], "quickfix");
        }
    }

    #[test]
    fn test_jetbrains_run_configuration_missing_ide() {
        let result = jetbrains_run_configuration_work(json!({"config_name": "Run Tests"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_run_configuration_missing_config() {
        let result = jetbrains_run_configuration_work(json!({"ide": "idea"}));
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_jetbrains_run_configuration_unsupported_ide() {
        let result = jetbrains_run_configuration_work(json!({
            "ide": "nano",
            "config_name": "Run All"
        }));
        assert!(result.get("error").is_some());
        assert!(!result["tool_available"].as_bool().unwrap_or(true));
    }

    #[test]
    fn test_jetbrains_run_configuration_debug_mode() {
        let result = jetbrains_run_configuration_work(json!({
            "ide": "idea",
            "config_name": "Run App",
            "debug": true
        }));
        // IDEA may or may not be installed; if not, we get a tool_available=false response
        if result.get("tool_available") == Some(&json!(false)) {
            assert!(result.get("error").is_some());
        } else {
            assert_eq!(result["mode"], "debug");
            assert_eq!(result["config_name"], "Run App");
        }
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("/tmp/a.rs"), "%2Ftmp%2Fa.rs");
        assert_eq!(urlencoding("simple"), "simple");
    }
}
