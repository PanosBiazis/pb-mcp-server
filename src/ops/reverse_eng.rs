//! # Reverse Engineering & Binary Analysis Tools
//!
//! Tools for AI agents to analyze binaries, firmware, and data structures.
//! Subprocess-based tools (`objdump`, `readelf`, `strings`, `file`, `nm`,
//! `hexdump`, `strace`, `binwalk`, etc.) are checked for availability before
//! invocation. Pure-Rust implementations are used for entropy calculation,
//! byte carving, pattern search, struct parsing, and binary diffing.
//!
//! All functions return `String`. Errors are prefixed with `"Error:"`.
//! Responses are truncated via `super::truncate_chars` and
//! `super::MAX_RESPONSE_CHARS`.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum binary file size accepted by any function in this module (100 MiB).
const MAX_BINARY_BYTES: u64 = 100 * 1024 * 1024;

/// Default minimum string length for `strings` extraction.
const DEFAULT_MIN_STRING_LEN: usize = 4;

/// Default block size for entropy analysis.
const DEFAULT_ENTROPY_BLOCK_SIZE: usize = 1024;

/// Maximum number of pattern-search results.
const DEFAULT_MAX_PATTERN_RESULTS: usize = 500;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate that `path` exists, is a file, and is within the size limit.
/// Returns the file metadata or an error string starting with `"Error:"`.
fn validate_path(path: &Path) -> Result<fs::Metadata, String> {
    if !path.exists() {
        return Err(format!("Error: path does not exist: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("Error: not a file: {}", path.display()));
    }
    let meta =
        fs::metadata(path).map_err(|e| format!("Error: cannot stat {}: {e}", path.display()))?;
    if meta.len() > MAX_BINARY_BYTES {
        return Err(format!(
            "Error: file too large ({} bytes, max {} MiB)",
            meta.len(),
            MAX_BINARY_BYTES / (1024 * 1024)
        ));
    }
    Ok(meta)
}

/// Check whether a CLI tool is available on `PATH` by running `<tool> --version`.
fn cli_exists(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a command and return its stdout as a String, or an error on non-zero
/// exit.  Stderr is appended when the command fails.
fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return Err(format!(
                    "Error: {} exit={:?}\nSTDERR:\n{se}",
                    cmd,
                    o.status.code()
                ));
            }
            Ok(stdout.trim().to_string())
        }
        Err(e) => Err(format!("Error: spawn {cmd}: {e}")),
    }
}

/// Run a command and return combined stdout + stderr (some tools write
/// informational output to stderr).
fn run_command_combined(cmd: &str, args: &[&str]) -> Result<String, String> {
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return Err(format!(
                    "Error: {} exit={:?}\nSTDERR:\n{se}",
                    cmd,
                    o.status.code()
                ));
            }
            let mut combined = String::new();
            if !stdout.is_empty() {
                combined.push_str(stdout.trim());
            }
            if !stderr.is_empty() && !stdout.is_empty() {
                combined.push('\n');
            }
            if !stderr.is_empty() {
                combined.push_str(stderr.trim());
            }
            Ok(combined)
        }
        Err(e) => Err(format!("Error: spawn {cmd}: {e}")),
    }
}

/// Run a command with a wall-clock timeout (spawn + kill pattern).
fn run_command_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let start = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("Error: spawn {cmd}: {e}"))?;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("Error: read output: {e}"))?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !status.success() {
                    let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                    return Err(format!(
                        "Error: {cmd} exit={:?}\nSTDERR:\n{se}",
                        status.code()
                    ));
                }
                let mut combined = String::new();
                if !stdout.is_empty() {
                    combined.push_str(stdout.trim());
                }
                if !stderr.is_empty() && !stdout.is_empty() {
                    combined.push('\n');
                }
                if !stderr.is_empty() {
                    combined.push_str(stderr.trim());
                }
                return Ok(combined);
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "Error: {} timed out after {} seconds",
                        cmd,
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("Error: checking child status: {e}"));
            }
        }
    }
}

/// Read the entire file into a byte vector (up to `MAX_BINARY_BYTES`).
fn read_file_bytes(path: &Path) -> Result<Vec<u8>, String> {
    validate_path(path)?;
    let mut f = fs::File::open(path).map_err(|e| format!("Error: open {}: {e}", path.display()))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)
        .map_err(|e| format!("Error: read {}: {e}", path.display()))?;
    Ok(buf)
}

/// Format a byte count as a human-readable size string.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Truncate a response using the parent module's `truncate_chars` and
/// `MAX_RESPONSE_CHARS`.
fn truncate_response(s: &str) -> String {
    let (t, was_truncated) = super::truncate_chars(s, super::MAX_RESPONSE_CHARS);
    if was_truncated {
        format!("{t}\n[...truncated]")
    } else {
        t
    }
}

// ---------------------------------------------------------------------------
// 1. re_binary_info
// ---------------------------------------------------------------------------

/// Analyze a binary file: architecture, format (ELF/PE/Mach-O), linked
/// libraries (ldd), symbol count, and sections.
///
/// Uses `file`, `readelf -h` / `objdump -f`, and `ldd`.
pub fn re_binary_info_work(path: PathBuf) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    let p = path.display();
    let mut out = format!("RE_BINARY_INFO: {}\n\n", p);

    // --- file(1) ---
    if cli_exists("file") {
        match run_command("file", &[&path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("File type:\n{s}\n\n")),
            Err(e) => out.push_str(&format!("file: {e}\n\n")),
        }
    } else {
        out.push_str("file: not found on PATH\n\n");
    }

    // --- ELF header (readelf preferred, objdump fallback) ---
    if cli_exists("readelf") {
        match run_command_combined("readelf", &["-h", &path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("ELF header (readelf -h):\n{s}\n\n")),
            Err(e) => out.push_str(&format!("readelf -h: {e}\n\n")),
        }
    } else if cli_exists("objdump") {
        match run_command_combined("objdump", &["-f", &path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("File header (objdump -f):\n{s}\n\n")),
            Err(e) => out.push_str(&format!("objdump -f: {e}\n\n")),
        }
    } else {
        out.push_str("ELF/header info: readelf/objdump not found\n\n");
    }

    // --- Shared library dependencies (ldd) ---
    if cfg!(target_os = "linux") && cli_exists("ldd") {
        match run_command("ldd", &[&path.to_string_lossy()]) {
            Ok(s) => {
                let count = s.lines().filter(|l| !l.trim().is_empty()).count();
                out.push_str(&format!(
                    "Shared libraries (ldd) — {count} entries:\n{s}\n\n"
                ));
            }
            Err(e) => out.push_str(&format!("ldd: {e}\n\n")),
        }
    } else if cfg!(target_os = "macos") && cli_exists("otool") {
        match run_command("otool", &["-L", &path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("Shared libraries (otool -L):\n{s}\n\n")),
            Err(e) => out.push_str(&format!("otool -L: {e}\n\n")),
        }
    } else {
        out.push_str("Shared libraries: ldd/otool not available\n\n");
    }

    // --- Symbol count ---
    if cli_exists("nm") {
        match run_command("nm", &[&path.to_string_lossy()]) {
            Ok(s) => {
                let count = s.lines().filter(|l| !l.trim().is_empty()).count();
                out.push_str(&format!("Symbols (nm): {count}\n\n"));
            }
            Err(_) => {
                // nm may fail on stripped binaries — fall back to objdump -t
                if cli_exists("objdump") {
                    match run_command("objdump", &["-t", &path.to_string_lossy()]) {
                        Ok(s) => {
                            let count = s
                                .lines()
                                .filter(|l| !l.trim().is_empty() && !l.starts_with('.'))
                                .count()
                                .saturating_sub(2); // header lines
                            out.push_str(&format!("Symbols (objdump -t): ~{count}\n\n"));
                        }
                        Err(e) => out.push_str(&format!("objdump -t: {e}\n\n")),
                    }
                } else {
                    out.push_str("Symbols: nm/objdump not found\n\n");
                }
            }
        }
    } else if cli_exists("objdump") {
        match run_command("objdump", &["-t", &path.to_string_lossy()]) {
            Ok(s) => {
                let count = s
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.starts_with('.'))
                    .count()
                    .saturating_sub(2);
                out.push_str(&format!("Symbols (objdump -t): ~{count}\n\n"));
            }
            Err(e) => out.push_str(&format!("objdump -t: {e}\n\n")),
        }
    } else {
        out.push_str("Symbols: nm/objdump not found\n\n");
    }

    // --- Section headers ---
    if cli_exists("readelf") {
        match run_command("readelf", &["-S", &path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("Sections (readelf -S):\n{s}\n")),
            Err(e) => out.push_str(&format!("readelf -S: {e}\n")),
        }
    } else if cli_exists("objdump") {
        match run_command("objdump", &["-h", &path.to_string_lossy()]) {
            Ok(s) => out.push_str(&format!("Sections (objdump -h):\n{s}\n")),
            Err(e) => out.push_str(&format!("objdump -h: {e}\n")),
        }
    } else {
        out.push_str("Sections: readelf/objdump not found\n");
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 2. re_strings
// ---------------------------------------------------------------------------

/// Extract printable strings from a binary using the `strings` command.
///
/// * `min_length` — minimum string length (default 4).
/// * `encoding` — character encoding: `ascii`, `utf-8`, `utf-16le`,
///   `utf-16be` (default `ascii`).
pub fn re_strings_work(
    path: PathBuf,
    min_length: Option<usize>,
    encoding: Option<String>,
) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    if !cli_exists("strings") {
        return "Error: 'strings' not found on PATH".to_string();
    }

    let min_len = min_length.unwrap_or(DEFAULT_MIN_STRING_LEN);
    let enc = encoding.unwrap_or_else(|| "ascii".to_string());

    // Build encoding flag for GNU strings
    let encoding_flag = match enc.to_lowercase().as_str() {
        "ascii" | "utf-8" => "", // default — no flag needed
        "utf-16le" => "-el",
        "utf-16be" => "-eb",
        other => {
            return format!("Error: unsupported encoding '{other}'. Supported: ascii, utf-8, utf-16le, utf-16be");
        }
    };

    let path_str = path.to_string_lossy().to_string();
    let mut args: Vec<String> = Vec::new();

    // Minimum length flag
    args.push(format!("-n{min_len}"));

    // Encoding flag
    if !encoding_flag.is_empty() {
        args.push(encoding_flag.to_string());
    }

    args.push(path_str);

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let header = format!(
        "RE_STRINGS: {} (min_len={}, encoding={})\n\n",
        path.display(),
        min_len,
        enc
    );

    match run_command("strings", &args_ref) {
        Ok(s) => truncate_response(&format!("{header}{s}")),
        Err(e) => format!("{header}{e}"),
    }
}

// ---------------------------------------------------------------------------
// 3. re_disassemble
// ---------------------------------------------------------------------------

/// Disassemble a binary using `objdump -d`.
///
/// * `section` — section to disassemble (e.g. `.text`).
/// * `architecture` — architecture override for objdump (e.g. `i386`,
///   `arm`, `mips`).
/// * `start_addr` — hex address to start from (e.g. `"0x400000"`).
/// * `count` — limit on the number of disassembly lines returned.
pub fn re_disassemble_work(
    path: PathBuf,
    section: Option<String>,
    architecture: Option<String>,
    start_addr: Option<String>,
    count: Option<usize>,
) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    if !cli_exists("objdump") {
        return "Error: 'objdump' not found on PATH".to_string();
    }

    let path_str = path.to_string_lossy().to_string();
    let mut args: Vec<String> = Vec::new();

    args.push("-d".to_string()); // disassemble

    // Architecture override
    if let Some(arch) = &architecture {
        args.push(format!("-m{arch}"));
    }

    // Section filter
    if let Some(sec) = &section {
        args.push(format!("--section={sec}"));
    }

    // Start address
    if let Some(addr) = &start_addr {
        args.push(format!("--start-address={addr}"));
    }

    args.push(path_str);

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let header = format!(
        "RE_DISASSEMBLE: {} (section={}, arch={}, start={}, count={})\n\n",
        path.display(),
        section.as_deref().unwrap_or("all"),
        architecture.as_deref().unwrap_or("default"),
        start_addr.as_deref().unwrap_or("start"),
        count.map_or("all".to_string(), |c| c.to_string())
    );

    match run_command_combined("objdump", &args_ref) {
        Ok(s) => {
            let limited = if let Some(n) = count {
                s.lines().take(n).collect::<Vec<_>>().join("\n")
            } else {
                s
            };
            truncate_response(&format!("{header}{limited}"))
        }
        Err(e) => format!("{header}{e}"),
    }
}

// ---------------------------------------------------------------------------
// 4. re_symbols
// ---------------------------------------------------------------------------

/// List symbols from a binary using `nm` or `objdump -t`.
///
/// * `symbol_type` — filter: `all`, `functions` (text symbols), `variables`
///   (data/bss symbols), `imports` (undefined), `exports` (defined, non-local).
pub fn re_symbols_work(path: PathBuf, symbol_type: Option<String>) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }

    let stype = symbol_type.unwrap_or_else(|| "all".to_string());
    let path_str = path.to_string_lossy().to_string();
    let header = format!("RE_SYMBOLS: {} (type={})\n\n", path.display(), stype);

    // Try nm first
    if cli_exists("nm") {
        // nm flags: -C (demangle), --no-sort (preserve order)
        let nm_output = run_command_combined("nm", &["-C", &path_str]);
        match nm_output {
            Ok(s) => {
                let filtered = filter_symbols(&s, &stype);
                return truncate_response(&format!("{header}{filtered}"));
            }
            Err(_) => {
                // nm may fail on non-ELF or stripped binaries
            }
        }
    }

    // Fallback: objdump -t
    if cli_exists("objdump") {
        match run_command_combined("objdump", &["-C", "-t", &path_str]) {
            Ok(s) => {
                let filtered = filter_symbols(&s, &stype);
                return truncate_response(&format!("{header}{filtered}"));
            }
            Err(e) => return format!("{header}{e}"),
        }
    }

    format!("{header}Error: nm and objdump not found on PATH")
}

/// Filter symbol lines by type category.
fn filter_symbols(symbols: &str, stype: &str) -> String {
    let lines: Vec<&str> = match stype.to_lowercase().as_str() {
        "functions" => symbols
            .lines()
            .filter(|l| {
                // nm: type char t/T indicates text section
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    return false;
                }
                // Look for nm-style symbol type in the type column
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let sym_type = parts[1];
                    sym_type == "T" || sym_type == "t" || sym_type == "W" || sym_type == "w"
                } else {
                    trimmed.contains("FUNC") || trimmed.contains(".text")
                }
            })
            .collect(),
        "variables" => symbols
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    return false;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let sym_type = parts[1];
                    sym_type == "D"
                        || sym_type == "d"
                        || sym_type == "B"
                        || sym_type == "b"
                        || sym_type == "R"
                        || sym_type == "r"
                        || sym_type == "G"
                        || sym_type == "g"
                        || sym_type == "S"
                        || sym_type == "s"
                } else {
                    trimmed.contains("OBJECT")
                        || trimmed.contains(".data")
                        || trimmed.contains(".bss")
                }
            })
            .collect(),
        "imports" => symbols
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    return false;
                }
                // nm: 'U' means undefined (import)
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let sym_type = parts[1];
                    sym_type == "U" || sym_type == "u" || sym_type == "v" || sym_type == "w"
                } else {
                    trimmed.contains("*UND*")
                }
            })
            .collect(),
        "exports" => symbols
            .lines()
            .filter(|l| {
                let trimmed = l.trim();
                if trimmed.is_empty() {
                    return false;
                }
                // Defined, global (uppercase type letter)
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let sym_type = parts[1];
                    let c = sym_type.chars().next().unwrap_or('\0');
                    c.is_ascii_uppercase() && c != 'U' && c != 'V' && c != 'W'
                } else {
                    !trimmed.contains("*UND*") && !trimmed.contains("LOCAL")
                }
            })
            .collect(),
        _ => symbols.lines().collect(), // "all"
    };

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// 5. re_sections
// ---------------------------------------------------------------------------

/// List ELF/binary sections using `readelf -S` or `objdump -h`.
pub fn re_sections_work(path: PathBuf) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    let path_str = path.to_string_lossy().to_string();
    let header = format!("RE_SECTIONS: {}\n\n", path.display());

    if cli_exists("readelf") {
        match run_command("readelf", &["-S", &path_str]) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(e) => return format!("{header}{e}"),
        }
    }

    if cli_exists("objdump") {
        match run_command("objdump", &["-h", &path_str]) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(e) => return format!("{header}{e}"),
        }
    }

    format!("{header}Error: readelf and objdump not found on PATH")
}

// ---------------------------------------------------------------------------
// 6. re_dependencies
// ---------------------------------------------------------------------------

/// List shared library dependencies using `ldd` (Linux) or `otool -L` (macOS).
pub fn re_dependencies_work(path: PathBuf) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    let path_str = path.to_string_lossy().to_string();
    let header = format!("RE_DEPENDENCIES: {}\n\n", path.display());

    if cfg!(target_os = "linux") && cli_exists("ldd") {
        match run_command("ldd", &[&path_str]) {
            Ok(s) => {
                let count = s.lines().filter(|l| !l.trim().is_empty()).count();
                return truncate_response(&format!("{header}Shared libraries ({count}):\n{s}"));
            }
            Err(e) => return format!("{header}{e}"),
        }
    }

    if cfg!(target_os = "macos") && cli_exists("otool") {
        match run_command("otool", &["-L", &path_str]) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(e) => return format!("{header}{e}"),
        }
    }

    format!("{header}Error: ldd (Linux) or otool (macOS) not found on PATH")
}

// ---------------------------------------------------------------------------
// 7. re_hexdump
// ---------------------------------------------------------------------------

/// Hex dump of a binary region with configurable offset, length, and byte
/// width per line.
///
/// Uses `xxd` or `hexdump` if available; falls back to pure Rust.
///
/// * `offset` — byte offset to start from (default 0).
/// * `length` — number of bytes to dump (default 256).
/// * `width` — bytes per line (default 16).
pub fn re_hexdump_work(
    path: PathBuf,
    offset: Option<u64>,
    length: Option<usize>,
    width: Option<usize>,
) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }

    let off = offset.unwrap_or(0);
    let len = length.unwrap_or(256);
    let w = width.unwrap_or(16).clamp(1, 32);

    let header = format!(
        "RE_HEXDUMP: {} (offset=0x{:X}, length={}, width={})\n\n",
        path.display(),
        off,
        len,
        w
    );

    // Try xxd first
    if cli_exists("xxd") {
        let args = [
            "-s",
            &off.to_string(),
            "-l",
            &len.to_string(),
            "-c",
            &w.to_string(),
            &path.to_string_lossy(),
        ];
        match run_command("xxd", &args) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(_) => { /* fall through */ }
        }
    }

    // Try hexdump
    if cli_exists("hexdump") {
        let args = [
            "-C",
            "-s",
            &off.to_string(),
            "-n",
            &len.to_string(),
            &path.to_string_lossy(),
        ];
        match run_command("hexdump", &args) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(_) => { /* fall through */ }
        }
    }

    // Pure Rust fallback
    match hexdump_pure(&path, off, len, w) {
        Ok(s) => truncate_response(&format!("{header}{s}")),
        Err(e) => format!("{header}{e}"),
    }
}

/// Pure-Rust hex dump implementation.
fn hexdump_pure(path: &Path, offset: u64, length: usize, width: usize) -> Result<String, String> {
    let data = read_file_bytes(path)?;
    let start = offset as usize;
    if start >= data.len() {
        return Ok("(offset past end of file)".to_string());
    }
    let end = (start + length).min(data.len());
    let slice = &data[start..end];

    let mut out = String::new();
    for (i, chunk) in slice.chunks(width).enumerate() {
        let addr = start + i * width;
        // Address
        out.push_str(&format!("{:08x}: ", addr));
        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            out.push_str(&format!("{:02x}", byte));
            if j < width - 1 {
                out.push(' ');
            }
            // Extra space at the midpoint of the width
            if width == 16 && j == 7 {
                out.push(' ');
            }
        }
        // Padding if last chunk is short
        if chunk.len() < width {
            for _ in 0..(width - chunk.len()) {
                out.push_str("   ");
            }
            if width == 16 && chunk.len() < 8 {
                out.push(' ');
            }
        }
        out.push_str("  ");
        // ASCII column
        for byte in chunk.iter() {
            let c = if *byte >= 0x20 && *byte <= 0x7e {
                *byte as char
            } else {
                '.'
            };
            out.push(c);
        }
        out.push('\n');
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 8. re_entropy
// ---------------------------------------------------------------------------

/// Calculate Shannon entropy of a file (overall and per-block) to identify
/// encrypted/compressed regions.  Pure Rust implementation.
///
/// * `block_size` — size of each block for per-block analysis
///   (default 1024 bytes).
pub fn re_entropy_work(path: PathBuf, block_size: Option<usize>) -> String {
    let data = match read_file_bytes(&path) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let bs = block_size.unwrap_or(DEFAULT_ENTROPY_BLOCK_SIZE).max(1);
    let header = format!(
        "RE_ENTROPY: {} (size={}, block_size={})\n\n",
        path.display(),
        format_size(data.len() as u64),
        bs
    );

    let overall = shannon_entropy(&data);
    let mut out = format!(
        "{header}Overall entropy: {:.4} bits/byte (max 8.0)\n",
        overall
    );

    // Interpretation hint
    if overall > 7.5 {
        out.push_str("  → Likely encrypted or high-compression data\n");
    } else if overall > 6.0 {
        out.push_str("  → Possibly compressed data\n");
    } else if overall < 1.0 {
        out.push_str("  → Mostly uniform / sparse data\n");
    } else {
        out.push_str("  → Mixed or normal data\n");
    }
    out.push('\n');

    // Per-block analysis
    out.push_str("Per-block entropy:\n");
    out.push_str("Offset        Size    Entropy  Hint\n");
    out.push_str("------------  ------  -------  --------------------\n");

    for (i, chunk) in data.chunks(bs).enumerate() {
        let offset = i * bs;
        let ent = shannon_entropy(chunk);
        let hint = if ent > 7.5 {
            "encrypted/compressed"
        } else if ent > 6.0 {
            "compressed"
        } else if ent < 1.0 {
            "uniform/sparse"
        } else {
            "normal"
        };
        out.push_str(&format!(
            "0x{:08X}  {:>6}  {:>7.4}  {}\n",
            offset,
            format_size(chunk.len() as u64),
            ent,
            hint
        ));
    }

    truncate_response(&out)
}

/// Compute Shannon entropy of a byte slice in bits per byte (0.0–8.0).
fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let len = data.len() as f64;
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let mut entropy = 0.0_f64;
    for &count in &freq {
        if count == 0 {
            continue;
        }
        let p = count as f64 / len;
        entropy -= p * p.log2();
    }
    entropy
}

// ---------------------------------------------------------------------------
// 9. re_carve
// ---------------------------------------------------------------------------

/// Carve/extract a region of bytes from a binary at a given offset.
/// Pure Rust implementation.
///
/// * `offset` — byte offset to start carving from.
/// * `length` — number of bytes to carve.
/// * `output` — optional output file path; if omitted, a hex preview is
///   returned.
pub fn re_carve_work(path: PathBuf, offset: u64, length: usize, output: Option<PathBuf>) -> String {
    let data = match read_file_bytes(&path) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let start = offset as usize;
    if start >= data.len() {
        return format!(
            "Error: offset 0x{:X} past end of file (size={})",
            offset,
            data.len()
        );
    }
    let end = (start + length).min(data.len());
    let carved = &data[start..end];
    let actual_len = carved.len();

    let header = format!(
        "RE_CARVE: {} (offset=0x{:X}, requested={}, actual={})\n",
        path.display(),
        offset,
        length,
        actual_len
    );

    if let Some(out_path) = output {
        match fs::write(&out_path, carved) {
            Ok(()) => {
                format!(
                    "{header}Carved {} bytes to {}\n",
                    actual_len,
                    out_path.display()
                )
            }
            Err(e) => format!("{header}Error: failed to write {}: {e}", out_path.display()),
        }
    } else {
        // Hex preview (first 512 bytes)
        let preview_len = actual_len.min(512);
        let preview = &carved[..preview_len];
        let mut hex = String::new();
        for (i, chunk) in preview.chunks(16).enumerate() {
            let addr = start + i * 16;
            hex.push_str(&format!("{:08x}: ", addr));
            for (j, byte) in chunk.iter().enumerate() {
                hex.push_str(&format!("{:02x}", byte));
                if j < 15 {
                    hex.push(' ');
                }
                if j == 7 {
                    hex.push(' ');
                }
            }
            // Padding
            if chunk.len() < 16 {
                for _ in 0..(16 - chunk.len()) {
                    hex.push_str("   ");
                }
                if chunk.len() < 8 {
                    hex.push(' ');
                }
            }
            hex.push_str("  ");
            for byte in chunk.iter() {
                let c = if *byte >= 0x20 && *byte <= 0x7e {
                    *byte as char
                } else {
                    '.'
                };
                hex.push(c);
            }
            hex.push('\n');
        }
        if actual_len > preview_len {
            hex.push_str(&format!(
                "... {} more bytes not shown\n",
                actual_len - preview_len
            ));
        }
        format!("{header}\n{hex}")
    }
}

// ---------------------------------------------------------------------------
// 10. re_pattern_search
// ---------------------------------------------------------------------------

/// Search for a byte pattern in a binary file. Supports hex patterns
/// (e.g. `"DEADBEEF"`, `"DE AD BE EF"`) and ASCII text patterns.
/// Pure Rust implementation.
///
/// * `pattern` — hex string (uppercase hex digits, optional spaces) or a
///   plain text pattern prefixed with `text:` (e.g. `"text:password"`).
/// * `max_results` — maximum matches to return (default 500).
pub fn re_pattern_search_work(
    path: PathBuf,
    pattern: String,
    max_results: Option<usize>,
) -> String {
    let data = match read_file_bytes(&path) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let max = max_results.unwrap_or(DEFAULT_MAX_PATTERN_RESULTS);
    let header = format!(
        "RE_PATTERN_SEARCH: {} (pattern=\"{}\", max_results={})\n\n",
        path.display(),
        pattern,
        max
    );

    // Parse pattern
    let needle = match parse_pattern(&pattern) {
        Ok(n) => n,
        Err(e) => return format!("{header}{e}"),
    };

    if needle.is_empty() {
        return format!("{header}Error: empty pattern");
    }

    // Search
    let mut matches_: Vec<usize> = Vec::new();
    let max_search = data.len().saturating_sub(needle.len() - 1);
    for i in 0..max_search {
        if data[i..].starts_with(&needle) {
            matches_.push(i);
            if matches_.len() >= max {
                break;
            }
        }
    }

    if matches_.is_empty() {
        return format!("{header}No matches found.\n");
    }

    let mut out = format!("{header}Found {} match(es):\n\n", matches_.len());
    for &offset in &matches_ {
        let end = (offset + 32).min(data.len());
        let context = &data[offset..end];
        let hex: Vec<String> = context.iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = context
            .iter()
            .map(|&b| {
                if (0x20..=0x7e).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        out.push_str(&format!(
            "  0x{:08X}: {}  |{}|\n",
            offset,
            hex.join(" "),
            ascii
        ));
    }

    truncate_response(&out)
}

/// Parse a pattern string into a byte vector.
///
/// - Hex patterns: `"DEADBEEF"`, `"DE AD BE EF"`, `"0xDEADBEEF"`
/// - Text patterns: `"text:hello"` or `"txt:hello"`
fn parse_pattern(pattern: &str) -> Result<Vec<u8>, String> {
    let trimmed = pattern.trim();

    // Text pattern
    if let Some(text) = trimmed
        .strip_prefix("text:")
        .or_else(|| trimmed.strip_prefix("txt:"))
    {
        return Ok(text.as_bytes().to_vec());
    }

    // Hex pattern — strip optional "0x" prefix and spaces
    let hex_str = trimmed
        .strip_prefix("0x")
        .unwrap_or(trimmed)
        .replace(' ', "");

    if hex_str.is_empty() {
        return Err("Error: empty hex pattern".to_string());
    }

    if !hex_str.len().is_multiple_of(2) {
        return Err(format!(
            "Error: hex pattern has odd length ({} chars): '{}'",
            hex_str.len(),
            hex_str
        ));
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte_str = &hex_str[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(b) => bytes.push(b),
            Err(_) => {
                return Err(format!("Error: invalid hex byte '{}' in pattern", byte_str));
            }
        }
    }
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// 11. re_struct_parse
// ---------------------------------------------------------------------------

/// Parse a binary structure at a given offset using a format string.
/// Pure Rust implementation using `std::io::Read`.
///
/// Format string is a comma-separated list of type specs:
/// - `u8`, `u16le`, `u16be`, `u32le`, `u32be`, `u64le`, `u64be`
/// - `i8`, `i16le`, `i16be`, `i32le`, `i32be`, `i64le`, `i64be`
/// - `f32le`, `f32be`, `f64le`, `f64be`
/// - `bytes:N` — N raw bytes (displayed as hex)
///
/// Example: `"u32le,u16be,f32le,bytes:4"`
pub fn re_struct_parse_work(path: PathBuf, offset: u64, format: String) -> String {
    let data = match read_file_bytes(&path) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let header = format!(
        "RE_STRUCT_PARSE: {} (offset=0x{:X}, format=\"{}\")\n\n",
        path.display(),
        offset,
        format
    );

    // Parse format string into field specs
    let fields = match parse_format_string(&format) {
        Ok(f) => f,
        Err(e) => return format!("{header}{e}"),
    };

    let start = offset as usize;
    if start >= data.len() {
        return format!(
            "{header}Error: offset 0x{:X} past end of file (size={})",
            offset,
            data.len()
        );
    }

    let remaining = &data[start..];
    let mut cursor = std::io::Cursor::new(remaining);
    let mut out = String::new();
    let mut field_offset: u64 = offset;
    out.push_str("Field Offset Type Value\n");
    out.push_str("----------- ---------- -------- ----------------\n");

    'fields: for field in &fields {
        let label = &field.label;
        let type_name = &field.type_name;
        let size = field.size;

        if cursor.position() as usize + size > remaining.len() {
            out.push_str(&format!(
                "{:<11} 0x{:08X} {:<8} (not enough data: need {} bytes, {} remaining)\n",
                label,
                field_offset,
                type_name,
                size,
                remaining.len() - cursor.position() as usize
            ));
            break 'fields;
        }

        let value = read_field_value(&mut cursor, field);
        out.push_str(&format!(
            "{:<11} 0x{:08X}  {:<8}  {}\n",
            label, field_offset, type_name, value
        ));
        field_offset += size as u64;
    }

    truncate_response(&format!("{header}{out}"))
}

/// A single field specification parsed from the format string.
struct FieldSpec {
    label: String,
    type_name: String,
    size: usize,
    variant: FieldVariant,
}

enum FieldVariant {
    U8,
    I8,
    U16Le,
    U16Be,
    I16Le,
    I16Be,
    U32Le,
    U32Be,
    I32Le,
    I32Be,
    U64Le,
    U64Be,
    I64Le,
    I64Be,
    F32Le,
    F32Be,
    F64Le,
    F64Be,
    Bytes(usize),
}

/// Parse a comma-separated format string into field specifications.
fn parse_format_string(format: &str) -> Result<Vec<FieldSpec>, String> {
    let mut fields = Vec::new();
    for (idx, token) in format.split(',').enumerate() {
        let tok = token.trim().to_lowercase();
        if tok.is_empty() {
            continue;
        }
        let (label, type_name, size, variant) = match tok.as_str() {
            "u8" => (
                format!("field{}", idx),
                "u8".to_string(),
                1,
                FieldVariant::U8,
            ),
            "i8" => (
                format!("field{}", idx),
                "i8".to_string(),
                1,
                FieldVariant::I8,
            ),
            "u16le" => (
                format!("field{}", idx),
                "u16le".to_string(),
                2,
                FieldVariant::U16Le,
            ),
            "u16be" => (
                format!("field{}", idx),
                "u16be".to_string(),
                2,
                FieldVariant::U16Be,
            ),
            "i16le" => (
                format!("field{}", idx),
                "i16le".to_string(),
                2,
                FieldVariant::I16Le,
            ),
            "i16be" => (
                format!("field{}", idx),
                "i16be".to_string(),
                2,
                FieldVariant::I16Be,
            ),
            "u32le" => (
                format!("field{}", idx),
                "u32le".to_string(),
                4,
                FieldVariant::U32Le,
            ),
            "u32be" => (
                format!("field{}", idx),
                "u32be".to_string(),
                4,
                FieldVariant::U32Be,
            ),
            "i32le" => (
                format!("field{}", idx),
                "i32le".to_string(),
                4,
                FieldVariant::I32Le,
            ),
            "i32be" => (
                format!("field{}", idx),
                "i32be".to_string(),
                4,
                FieldVariant::I32Be,
            ),
            "u64le" => (
                format!("field{}", idx),
                "u64le".to_string(),
                8,
                FieldVariant::U64Le,
            ),
            "u64be" => (
                format!("field{}", idx),
                "u64be".to_string(),
                8,
                FieldVariant::U64Be,
            ),
            "i64le" => (
                format!("field{}", idx),
                "i64le".to_string(),
                8,
                FieldVariant::I64Le,
            ),
            "i64be" => (
                format!("field{}", idx),
                "i64be".to_string(),
                8,
                FieldVariant::I64Be,
            ),
            "f32le" => (
                format!("field{}", idx),
                "f32le".to_string(),
                4,
                FieldVariant::F32Le,
            ),
            "f32be" => (
                format!("field{}", idx),
                "f32be".to_string(),
                4,
                FieldVariant::F32Be,
            ),
            "f64le" => (
                format!("field{}", idx),
                "f64le".to_string(),
                8,
                FieldVariant::F64Le,
            ),
            "f64be" => (
                format!("field{}", idx),
                "f64be".to_string(),
                8,
                FieldVariant::F64Be,
            ),
            s if s.starts_with("bytes:") => {
                let n: usize = s[6..]
                    .parse()
                    .map_err(|_| format!("Error: invalid byte count in '{}'", s))?;
                (
                    format!("field{}", idx),
                    format!("bytes:{}", n),
                    n,
                    FieldVariant::Bytes(n),
                )
            }
            other => {
                return Err(format!(
                    "Error: unknown type '{}' in format string. \
                     Supported: u8, i8, u16le, u16be, i16le, i16be, \
                     u32le, u32be, i32le, i32be, u64le, u64be, i64le, i64be, \
                     f32le, f32be, f64le, f64be, bytes:N",
                    other
                ));
            }
        };
        fields.push(FieldSpec {
            label,
            type_name,
            size,
            variant,
        });
    }
    if fields.is_empty() {
        return Err("Error: empty format string".to_string());
    }
    Ok(fields)
}

/// Read a single field value from a cursor and format it as a string.
fn read_field_value(cursor: &mut std::io::Cursor<&[u8]>, field: &FieldSpec) -> String {
    let mut read_bytes = |n: usize| -> Vec<u8> {
        let mut buf = vec![0u8; n];
        let _ = cursor.read_exact(&mut buf);
        buf
    };

    match &field.variant {
        FieldVariant::U8 => {
            let buf = read_bytes(1);
            format!("{} (0x{:02X})", buf[0], buf[0])
        }
        FieldVariant::I8 => {
            let buf = read_bytes(1);
            format!("{} (0x{:02X})", buf[0] as i8, buf[0])
        }
        FieldVariant::U16Le => {
            let buf = read_bytes(2);
            let v = u16::from_le_bytes([buf[0], buf[1]]);
            format!("{} (0x{:04X})", v, v)
        }
        FieldVariant::U16Be => {
            let buf = read_bytes(2);
            let v = u16::from_be_bytes([buf[0], buf[1]]);
            format!("{} (0x{:04X})", v, v)
        }
        FieldVariant::I16Le => {
            let buf = read_bytes(2);
            let v = i16::from_le_bytes([buf[0], buf[1]]);
            format!("{} (0x{:04X})", v, v as u16)
        }
        FieldVariant::I16Be => {
            let buf = read_bytes(2);
            let v = i16::from_be_bytes([buf[0], buf[1]]);
            format!("{} (0x{:04X})", v, v as u16)
        }
        FieldVariant::U32Le => {
            let buf = read_bytes(4);
            let v = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{} (0x{:08X})", v, v)
        }
        FieldVariant::U32Be => {
            let buf = read_bytes(4);
            let v = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{} (0x{:08X})", v, v)
        }
        FieldVariant::I32Le => {
            let buf = read_bytes(4);
            let v = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{} (0x{:08X})", v, v as u32)
        }
        FieldVariant::I32Be => {
            let buf = read_bytes(4);
            let v = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{} (0x{:08X})", v, v as u32)
        }
        FieldVariant::U64Le => {
            let buf = read_bytes(8);
            let v = u64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{} (0x{:016X})", v, v)
        }
        FieldVariant::U64Be => {
            let buf = read_bytes(8);
            let v = u64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{} (0x{:016X})", v, v)
        }
        FieldVariant::I64Le => {
            let buf = read_bytes(8);
            let v = i64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{} (0x{:016X})", v, v as u64)
        }
        FieldVariant::I64Be => {
            let buf = read_bytes(8);
            let v = i64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{} (0x{:016X})", v, v as u64)
        }
        FieldVariant::F32Le => {
            let buf = read_bytes(4);
            let v = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{v}")
        }
        FieldVariant::F32Be => {
            let buf = read_bytes(4);
            let v = f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            format!("{v}")
        }
        FieldVariant::F64Le => {
            let buf = read_bytes(8);
            let v = f64::from_le_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{v}")
        }
        FieldVariant::F64Be => {
            let buf = read_bytes(8);
            let v = f64::from_be_bytes([
                buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
            ]);
            format!("{v}")
        }
        FieldVariant::Bytes(n) => {
            let buf = read_bytes(*n);
            let hex: Vec<String> = buf.iter().map(|b| format!("{:02x}", b)).collect();
            let ascii: String = buf
                .iter()
                .map(|&b| {
                    if (0x20..=0x7e).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            format!("{} |{}|", hex.join(" "), ascii)
        }
    }
}

// ---------------------------------------------------------------------------
// 12. re_diff_binary
// ---------------------------------------------------------------------------

/// Compare two binary files (size, hash, byte-level differences summary).
/// Pure Rust implementation.
pub fn re_diff_binary_work(path_a: PathBuf, path_b: PathBuf) -> String {
    let data_a = match read_file_bytes(&path_a) {
        Ok(d) => d,
        Err(e) => return e,
    };
    let data_b = match read_file_bytes(&path_b) {
        Ok(d) => d,
        Err(e) => return e,
    };

    let header = format!(
        "RE_DIFF_BINARY:\n  A: {}\n  B: {}\n\n",
        path_a.display(),
        path_b.display()
    );

    let mut out = header;

    // Size comparison
    out.push_str(&format!(
        "Size A: {} bytes ({})\n",
        data_a.len(),
        format_size(data_a.len() as u64)
    ));
    out.push_str(&format!(
        "Size B: {} bytes ({})\n",
        data_b.len(),
        format_size(data_b.len() as u64)
    ));
    if data_a.len() != data_b.len() {
        let diff = (data_b.len() as i64 - data_a.len() as i64).abs();
        out.push_str(&format!("Size difference: {} bytes\n", diff));
    } else {
        out.push_str("Sizes are identical.\n");
    }

    // SHA-256 hash comparison
    let hash_a = sha256_hex(&data_a);
    let hash_b = sha256_hex(&data_b);
    out.push_str(&format!("\nSHA-256 A: {hash_a}\n"));
    out.push_str(&format!("SHA-256 B: {hash_b}\n"));

    if hash_a == hash_b {
        out.push_str("\nFiles are identical (same SHA-256 hash).\n");
        return truncate_response(&out);
    }
    out.push('\n');

    // Byte-level diff summary
    let min_len = data_a.len().min(data_b.len());
    let mut diff_count: usize = 0;
    let mut first_diff_offset: Option<usize> = None;
    let mut last_diff_offset: Option<usize> = None;

    for i in 0..min_len {
        if data_a[i] != data_b[i] {
            diff_count += 1;
            if first_diff_offset.is_none() {
                first_diff_offset = Some(i);
            }
            last_diff_offset = Some(i);
        }
    }

    // Extra bytes beyond the common length count as differences too
    let extra_diff = data_a.len().abs_diff(data_b.len());
    diff_count += extra_diff;

    out.push_str(&format!("Differing bytes: {}\n", diff_count));
    if let (Some(first), Some(last)) = (first_diff_offset, last_diff_offset) {
        out.push_str(&format!("First difference at offset: 0x{:08X}\n", first));
        out.push_str(&format!("Last difference at offset:  0x{:08X}\n", last));
    }

    // Show first N differences in detail
    if diff_count > 0 {
        out.push_str("\nFirst differences:\n");
        out.push_str("Offset      A        B\n");
        out.push_str("----------  ------   ------\n");
        let mut shown = 0;
        for i in 0..min_len {
            if data_a[i] != data_b[i] {
                out.push_str(&format!(
                    "0x{:08X}  {:02x}       {:02x}\n",
                    i, data_a[i], data_b[i]
                ));
                shown += 1;
                if shown >= 64 {
                    out.push_str(&format!(
                        "... {} more differing bytes not shown\n",
                        diff_count - shown - extra_diff
                    ));
                    break;
                }
            }
        }
        if extra_diff > 0 {
            out.push_str(&format!(
                "... + {} trailing bytes differ due to size mismatch\n",
                extra_diff
            ));
        }
    }

    truncate_response(&out)
}

/// Compute SHA-256 hash of a byte slice and return it as a hex string.
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode_bytes(result.as_slice())
}

/// Encode a byte slice as a lowercase hex string.
fn hex_encode_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------------
// 13. re_strace
// ---------------------------------------------------------------------------

/// Run a binary under `strace` to capture system calls.  Timeout enforced.
///
/// * `args` — arguments to pass to the binary.
/// * `timeout_secs` — maximum execution time (default 30 seconds).
pub fn re_strace_work(path: PathBuf, args: Vec<String>, timeout_secs: Option<u64>) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }
    if !cfg!(target_os = "linux") {
        return "Error: strace is only available on Linux".to_string();
    }
    if !cli_exists("strace") {
        return "Error: 'strace' not found on PATH".to_string();
    }

    let timeout = Duration::from_secs(timeout_secs.unwrap_or(30).clamp(1, 120));
    let path_str = path.to_string_lossy().to_string();

    let mut strace_args = vec![
        "-f".to_string(), // follow forks
        "-e".to_string(),
        "trace=file,process,network,signal,desc".to_string(), // key syscall groups
        "-s".to_string(),
        "128".to_string(), // string size
        path_str,
    ];
    strace_args.extend(args);

    let args_ref: Vec<&str> = strace_args.iter().map(|s| s.as_str()).collect();

    let header = format!(
        "RE_STRACE: {} (timeout={}s)\n\n",
        path.display(),
        timeout.as_secs()
    );

    match run_command_timeout("strace", &args_ref, timeout) {
        Ok(s) => truncate_response(&format!("{header}{s}")),
        Err(e) => format!("{header}{e}"),
    }
}

// ---------------------------------------------------------------------------
// 14. re_binwalk_scan
// ---------------------------------------------------------------------------

/// Scan for embedded files and firmware signatures using `binwalk` if
/// available, otherwise a basic magic-bytes check in pure Rust.
pub fn re_binwalk_scan_work(path: PathBuf) -> String {
    if let Err(e) = validate_path(&path) {
        return e;
    }

    let header = format!("RE_BINWALK_SCAN: {}\n\n", path.display());

    if cli_exists("binwalk") {
        match run_command("binwalk", &[&path.to_string_lossy()]) {
            Ok(s) => return truncate_response(&format!("{header}{s}")),
            Err(e) => return format!("{header}{e}"),
        }
    }

    // Pure Rust fallback: basic magic bytes check
    let data = match read_file_bytes(&path) {
        Ok(d) => d,
        Err(e) => return format!("{header}{e}"),
    };

    let mut out = format!("{header}binwalk not found — performing basic magic-bytes scan.\n\n");

    let mut results = Vec::new();
    let mut offset: usize = 0;
    while offset < data.len().saturating_sub(2) {
        if let Some(sig) = check_magic(&data[offset..]) {
            results.push((offset, sig));
        }
        offset += 1;
    }

    if results.is_empty() {
        out.push_str("No known signatures found.\n");
    } else {
        out.push_str("Decimal  Hex       Description\n");
        out.push_str("-------  --------  -----------\n");
        for (off, desc) in &results {
            out.push_str(&format!("{off:>7}  0x{off:06X}  {desc}\n"));
        }
        out.push_str(&format!("\n{} signature(s) found.\n", results.len()));
    }

    truncate_response(&out)
}

/// Check for known magic bytes at the start of a data slice.
/// Returns a description string if a signature is recognized.
fn check_magic(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }

    // Common magic signatures (order matters — longer signatures first)
    if data.len() >= 8 && data.starts_with(&[0x7f, 0x45, 0x4c, 0x46]) {
        let cls = if data.len() > 4 && data[4] == 1 {
            "32-bit"
        } else if data.len() > 4 && data[4] == 2 {
            "64-bit"
        } else {
            "unknown-class"
        };
        let endian = if data.len() > 5 && data[5] == 1 {
            "LE"
        } else if data.len() > 5 && data[5] == 2 {
            "BE"
        } else {
            "unknown-endian"
        };
        return Some(format!("ELF executable ({cls}, {endian})"));
    }

    if data.len() >= 2 && data.starts_with(&[0x4d, 0x5a]) {
        return Some("PE executable (MZ header)".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0xce, 0xfa, 0xed, 0xfe]) {
        return Some("Mach-O executable (32-bit)".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0xcf, 0xfa, 0xed, 0xfe]) {
        return Some("Mach-O executable (64-bit)".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0xca, 0xfe, 0xba, 0xbe]) {
        return Some("Mach-O universal / Java class file".to_string());
    }

    // Compressed archives
    if data.len() >= 2 && data.starts_with(&[0x1f, 0x8b]) {
        return Some("gzip compressed data".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0x50, 0x4b, 0x03, 0x04]) {
        return Some("ZIP archive (PK header)".to_string());
    }

    if data.len() >= 6 && data.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return Some("7-zip archive".to_string());
    }

    if data.len() >= 6 && data.starts_with(b"\xfd\x37\x7a\x58\x5a\x00") {
        return Some("XZ compressed data".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0x42, 0x5a, 0x68, 0x39]) {
        return Some("bzip2 compressed data".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        return Some("Zstandard compressed data".to_string());
    }

    // Tar archives
    if data.len() >= 262 {
        // POSIX tar magic at offset 257
        if data[257..262].starts_with(b"ustar") {
            return Some("tar archive (POSIX)".to_string());
        }
    }

    // RAR
    if data.len() >= 7 && data.starts_with(&[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00]) {
        return Some("RAR archive (v4)".to_string());
    }

    if data.len() >= 7 && data.starts_with(&[0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01]) {
        return Some("RAR archive (v5)".to_string());
    }

    // Images
    if data.len() >= 3 && data.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("JPEG image".to_string());
    }

    if data.len() >= 8 && data.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Some("PNG image".to_string());
    }

    if data.len() >= 4 && data.starts_with(b"GIF8") {
        return Some("GIF image".to_string());
    }

    if data.len() >= 2 && data.starts_with(b"BM") {
        return Some("BMP image".to_string());
    }

    // Audio/Video
    if data.len() >= 4 && data.starts_with(b"RIFF") {
        return Some("RIFF container (WAV/AVI/WebP)".to_string());
    }

    if data.len() >= 3 && data.starts_with(b"ID3") {
        return Some("MP3 audio (ID3 tag)".to_string());
    }

    if data.len() >= 4 && data.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return Some("Matroska/WebM video".to_string());
    }

    // PDF
    if data.len() >= 5 && data.starts_with(b"%PDF-") {
        return Some("PDF document".to_string());
    }

    // OpenSSL / PEM-like
    if data.len() >= 5 && data.starts_with(&[0x30, 0x82, 0x00, 0x00, 0x02])
        || (data.len() >= 3 && data[0] == 0x30 && (data[1] == 0x82 || data[1] == 0x81))
    {
        return Some("DER-encoded ASN.1 / certificate".to_string());
    }

    // Android DEX
    if data.len() >= 8 && data.starts_with(b"dex\n035\0") {
        return Some("Android DEX executable".to_string());
    }

    // Linux kernel / boot image
    if data.len() >= 4 && data.starts_with(&[0x27, 0x05, 0x19, 0x56]) {
        return Some("Linux kernel image (arm)".to_string());
    }

    // U-Boot
    if data.len() >= 4 && data.starts_with(&[0x27, 0x05, 0x19, 0x71]) {
        return Some("U-Boot legacy image".to_string());
    }

    // CPIO
    if data.len() >= 6
        && (data.starts_with(b"070707")
            || data.starts_with(b"070701")
            || data.starts_with(b"070702"))
    {
        return Some("cpio archive".to_string());
    }

    // SquashFS
    if data.len() >= 4
        && (data.starts_with(&[0x73, 0x71, 0x73, 0x68])
            || data.starts_with(&[0x68, 0x73, 0x71, 0x73]))
    {
        return Some("SquashFS filesystem".to_string());
    }

    // ISO 9660
    if data.len() >= 6 && data.starts_with(&[0x43, 0x44, 0x30, 0x30, 0x31, 0x01]) {
        return Some("ISO 9660 filesystem".to_string());
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper to create a temp file with given contents.
    fn temp_file(name: &str, data: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join("pb_mcp_reverse_eng_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(data).unwrap();
        path
    }

    fn cleanup(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    // ---- validate_path ----

    #[test]
    fn validate_path_rejects_missing() {
        let r = validate_path(Path::new("/nonexistent/file/xyz123"));
        assert!(r.is_err());
        assert!(r.unwrap_err().starts_with("Error:"));
    }

    #[test]
    fn validate_path_rejects_directory() {
        let r = validate_path(Path::new("/tmp"));
        assert!(r.is_err());
    }

    #[test]
    fn validate_path_accepts_file() {
        let p = temp_file("exists.bin", b"hello");
        let r = validate_path(&p);
        assert!(r.is_ok());
        cleanup(&p);
    }

    // ---- Shannon entropy ----

    #[test]
    fn entropy_zero_for_empty() {
        assert!((shannon_entropy(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn entropy_low_for_uniform() {
        // All zeros → zero entropy
        let data = vec![0u8; 1024];
        assert!(shannon_entropy(&data) < 0.01);
    }

    #[test]
    fn entropy_high_for_random() {
        // Every byte value equally → close to 8.0
        let mut data = Vec::new();
        for b in 0u8..=255 {
            for _ in 0..8 {
                data.push(b);
            }
        }
        let ent = shannon_entropy(&data);
        assert!(ent > 7.9, "entropy was {ent}");
    }

    // ---- parse_pattern ----

    #[test]
    fn parse_pattern_hex() {
        let result = parse_pattern("DEADBEEF").unwrap();
        assert_eq!(result, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_pattern_hex_spaced() {
        let result = parse_pattern("DE AD BE EF").unwrap();
        assert_eq!(result, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_pattern_hex_0x_prefix() {
        let result = parse_pattern("0xDEADBEEF").unwrap();
        assert_eq!(result, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_pattern_text() {
        let result = parse_pattern("text:hello").unwrap();
        assert_eq!(result, b"hello".to_vec());
    }

    #[test]
    fn parse_pattern_txt() {
        let result = parse_pattern("txt:hello").unwrap();
        assert_eq!(result, b"hello".to_vec());
    }

    #[test]
    fn parse_pattern_odd_hex_fails() {
        assert!(parse_pattern("ABC").is_err());
    }

    #[test]
    fn parse_pattern_invalid_hex_fails() {
        assert!(parse_pattern("ZZ").is_err());
    }

    // ---- re_pattern_search_work ----

    #[test]
    fn pattern_search_finds_match() {
        let data = vec![0x00, 0x01, 0xDE, 0xAD, 0xBE, 0xEF, 0x02, 0x03];
        let p = temp_file("search.bin", &data);
        let result = re_pattern_search_work(p.clone(), "DEADBEEF".to_string(), None);
        assert!(result.contains("0x00000002"));
        assert!(result.contains("1 match"));
        cleanup(&p);
    }

    #[test]
    fn pattern_search_text_mode() {
        let data = b"hello world this is a test".to_vec();
        let p = temp_file("search_text.bin", &data);
        let result = re_pattern_search_work(p.clone(), "text:world".to_string(), None);
        assert!(result.contains("0x00000006"));
        cleanup(&p);
    }

    #[test]
    fn pattern_search_no_match() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        let p = temp_file("search_nomatch.bin", &data);
        let result = re_pattern_search_work(p.clone(), "FF".to_string(), None);
        assert!(result.contains("No matches"));
        cleanup(&p);
    }

    // ---- re_struct_parse_work ----

    #[test]
    fn struct_parse_basic() {
        // u32le = 0x04030201 = 67305985, u16be = 0x0506 = 1286
        let data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let p = temp_file("struct.bin", &data);
        let result = re_struct_parse_work(p.clone(), 0, "u32le,u16be".to_string());
        assert!(result.contains("67305985"));
        assert!(result.contains("1286"));
        cleanup(&p);
    }

    #[test]
    fn struct_parse_bytes_field() {
        let data: Vec<u8> = vec![0x41, 0x42, 0x43, 0x44];
        let p = temp_file("struct_bytes.bin", &data);
        let result = re_struct_parse_work(p.clone(), 0, "bytes:4".to_string());
        assert!(result.contains("41 42 43 44"));
        assert!(result.contains("ABCD"));
        cleanup(&p);
    }

    #[test]
    fn struct_parse_bad_type() {
        assert!(parse_format_string("badtype").is_err());
    }

    // ---- re_carve_work ----

    #[test]
    fn carve_basic() {
        let data: Vec<u8> = (0..64).collect();
        let p = temp_file("carve.bin", &data);
        let result = re_carve_work(p.clone(), 4, 8, None);
        assert!(result.contains("actual=8"));
        assert!(result.contains("04 05 06 07"));
        cleanup(&p);
    }

    #[test]
    fn carve_to_file() {
        let data: Vec<u8> = (0..64).collect();
        let p = temp_file("carve_src.bin", &data);
        let out = std::env::temp_dir()
            .join("pb_mcp_reverse_eng_test")
            .join("carved_out.bin");
        let result = re_carve_work(p.clone(), 4, 8, Some(out.clone()));
        assert!(result.contains("Carved 8 bytes"));
        let carved = std::fs::read(&out).unwrap();
        assert_eq!(carved, vec![4, 5, 6, 7, 8, 9, 10, 11]);
        cleanup(&p);
        let _ = std::fs::remove_file(&out);
    }

    // ---- re_diff_binary_work ----

    #[test]
    fn diff_identical() {
        let data = b"same content".to_vec();
        let pa = temp_file("diff_a.bin", &data);
        let pb = temp_file("diff_b.bin", &data);
        let result = re_diff_binary_work(pa.clone(), pb.clone());
        assert!(result.contains("identical"));
        cleanup(&pa);
        cleanup(&pb);
    }

    #[test]
    fn diff_different() {
        let da = b"hello world".to_vec();
        let db = b"hello WARLD".to_vec();
        let pa = temp_file("diff2_a.bin", &da);
        let pb = temp_file("diff2_b.bin", &db);
        let result = re_diff_binary_work(pa.clone(), pb.clone());
        assert!(result.contains("Differing bytes:"));
        cleanup(&pa);
        cleanup(&pb);
    }

    // ---- hexdump_pure ----

    #[test]
    fn hexdump_pure_basic() {
        let data: Vec<u8> = (0..32).collect();
        let p = temp_file("hexdump.bin", &data);
        let result = hexdump_pure(&p, 0, 32, 16).unwrap();
        assert!(result.contains("00000000"));
        assert!(result.contains("00 01 02 03"));
        cleanup(&p);
    }

    // ---- re_entropy_work ----

    #[test]
    fn entropy_work_uniform() {
        let data = vec![0xAA_u8; 256];
        let p = temp_file("entropy_uniform.bin", &data);
        let result = re_entropy_work(p.clone(), None);
        assert!(result.contains("Overall entropy"));
        assert!(result.contains("uniform/sparse"));
        cleanup(&p);
    }

    // ---- check_magic ----

    #[test]
    fn magic_elf() {
        let data = [0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00];
        let sig = check_magic(&data).unwrap();
        assert!(sig.contains("ELF"));
        assert!(sig.contains("64-bit"));
    }

    #[test]
    fn magic_pe() {
        let data = [0x4d, 0x5a, 0x90, 0x00];
        let sig = check_magic(&data).unwrap();
        assert!(sig.contains("PE"));
    }

    #[test]
    fn magic_gzip() {
        let data = [0x1f, 0x8b, 0x08, 0x00];
        let sig = check_magic(&data).unwrap();
        assert!(sig.contains("gzip"));
    }

    #[test]
    fn magic_none() {
        let data = [0x00, 0x00];
        assert!(check_magic(&data).is_none());
    }

    // ---- filter_symbols ----

    #[test]
    fn filter_symbols_functions() {
        let input =
            "0000000000001000 T main\n0000000000002000 t helper\n0000000000003000 D data_sym\n";
        let result = filter_symbols(input, "functions");
        assert!(result.contains("main"));
        assert!(result.contains("helper"));
        assert!(!result.contains("data_sym"));
    }

    #[test]
    fn filter_symbols_imports() {
        let input = "0000000000000000 U printf\n0000000000001000 T main\n";
        let result = filter_symbols(input, "imports");
        assert!(result.contains("printf"));
        assert!(!result.contains("main"));
    }

    // ---- format_size ----

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn format_size_kib() {
        assert_eq!(format_size(2048), "2.00 KiB");
    }

    #[test]
    fn format_size_mib() {
        assert_eq!(format_size(5 * 1024 * 1024), "5.00 MiB");
    }
}
