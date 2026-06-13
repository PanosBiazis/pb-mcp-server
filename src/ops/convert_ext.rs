//! # Extended Conversion Tools
//!
//! Additional conversion utilities for the MCP server: document format
//! conversion (pandoc / LibreOffice), PDF text extraction (pdftotext),
//! archive listing & extraction, binary hex dumps, character-encoding
//! conversion (iconv), and QR code generation (qrencode).
//!
//! Each public work function accepts typed parameters, executes the
//! appropriate CLI tool (or pure-Rust implementation for hex dumps), and
//! returns a `String` response. Responses are truncated to approximately
//! 20 000 characters to stay within MCP transport limits.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum characters in a single work-function response.
const MAX_RESPONSE_CHARS: usize = 20_000;

/// Default maximum characters returned by PDF extraction.
const DEFAULT_PDF_MAX_CHARS: usize = 50_000;

/// Default maximum bytes read by the hex-dump tool.
const DEFAULT_HEXDUMP_LENGTH: u64 = 1024;

/// Default bytes per line in the hex-dump output.
const DEFAULT_HEXDUMP_WIDTH: usize = 16;

/// Default pixel size per QR module.
const DEFAULT_QR_PIXEL_SIZE: u32 = 4;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a string to *max* characters, appending a notice if clipped.
fn truncate_response(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push_str(&format!("\n\n[truncated to {} chars]", max));
    out
}

/// Return the lower-case file extension (without the leading dot), or an
/// empty string if none is present.
fn extension_lower(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Detect whether a path looks like a compound tar extension
/// (`.tar.gz`, `.tar.bz2`, `.tar.xz`).
fn is_tar_compressed(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tbz2")
        || name.ends_with(".tar.xz")
        || name.ends_with(".txz")
}

/// Classify an archive path into one of the supported formats.
///
/// Returns `"tar"`, `"zip"`, or `"7z"`. Returns an error string for
/// unsupported extensions.
fn classify_archive(path: &Path) -> Result<&'static str, String> {
    if is_tar_compressed(path) {
        return Ok("tar");
    }
    match extension_lower(path).as_str() {
        "tar" => Ok("tar"),
        "zip" | "jar" | "apk" => Ok("zip"),
        "7z" => Ok("7z"),
        other => Err(format!(
            "Error: unsupported archive extension .{other} (supported: tar, tar.gz, tgz, tar.bz2, tbz2, tar.xz, txz, zip, jar, apk, 7z)"
        )),
    }
}

/// Run an external command and return its stdout as a String, or an error
/// message on failure.
fn run_command(cmd: &mut Command) -> Result<String, String> {
    let output = cmd
        .output()
        .map_err(|e| format!("spawn {}: {e}", cmd.get_program().to_string_lossy()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".into());
        return Err(format!(
            "exit={code}\nSTDERR: {}",
            truncate_response(stderr.trim(), 4000)
        ));
    }
    // Some tools emit informational messages on stderr even on success.
    let mut combined = stdout.into_owned();
    let stderr_trimmed = stderr.trim();
    if !stderr_trimmed.is_empty() {
        combined.push_str(&format!(
            "\n--- stderr ---\n{}",
            truncate_response(stderr_trimmed, 4000)
        ));
    }
    Ok(combined)
}

/// Check that a file exists and is a regular file; return an error string
/// otherwise.
fn ensure_file(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("Error: not a file: {}", path.display()));
    }
    Ok(())
}

/// Check that a directory exists (creating it if possible); return an error
/// string on failure.
fn ensure_dir(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(path)
        .map_err(|e| format!("Error: cannot create directory {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// 1. convert_document_work
// ---------------------------------------------------------------------------

/// Convert DOCX / ODT / RTF documents to Markdown or plain text.
///
/// Tries `pandoc` first; falls back to `libreoffice --headless --convert-to`.
/// When neither tool is available the response contains installation
/// instructions.
pub fn convert_document_work(input_path: PathBuf, output_format: Option<String>) -> String {
    if let Err(e) = ensure_file(&input_path) {
        return e;
    }

    let fmt = output_format
        .as_deref()
        .unwrap_or("markdown")
        .trim()
        .to_ascii_lowercase();

    if !matches!(fmt.as_str(), "markdown" | "plain" | "plaintext") {
        return format!("Error: unsupported output_format {fmt:?} (use \"markdown\" or \"plain\")");
    }

    let pandoc_target = if fmt == "plain" || fmt == "plaintext" {
        "plain"
    } else {
        "markdown"
    };

    let ext = extension_lower(&input_path);
    let pandoc_from = ext.as_str();

    // --- Attempt 1: pandoc ---------------------------------------------------
    let pandoc_result = Command::new("pandoc")
        .args(["-f", pandoc_from, "-t", pandoc_target])
        .arg(&input_path)
        .output();

    match pandoc_result {
        Ok(out) => {
            if out.status.success() {
                let body = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let mut resp = format!(
                    "CONVERT_DOCUMENT tool=pandoc from={pandoc_from} to={pandoc_target} path={}\n\n{}",
                    input_path.display(),
                    body.trim()
                );
                if !stderr.trim().is_empty() {
                    resp.push_str(&format!(
                        "\n\n--- pandoc stderr ---\n{}",
                        truncate_response(stderr.trim(), 2000)
                    ));
                }
                return truncate_response(&resp, MAX_RESPONSE_CHARS);
            }
            // pandoc ran but failed — try the fallback below.
        }
        Err(_) => {
            // pandoc not installed — try the fallback below.
        }
    }

    // --- Attempt 2: libreoffice --headless -----------------------------------
    let libre_target = if pandoc_target == "plain" {
        "txt"
    } else {
        // LibreOffice cannot produce markdown directly; fall back to txt.
        "txt"
    };

    let libre_result = Command::new("libreoffice")
        .args(["--headless", "--convert-to", libre_target])
        .arg(&input_path)
        .output();

    if let Ok(out) = libre_result {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            // LibreOffice writes the converted file next to the input.
            // Deduce the output path from the input path + new extension.
            let mut out_path = input_path.clone();
            out_path.set_extension(libre_target);

            let body = if out_path.is_file() {
                match fs::read_to_string(&out_path) {
                    Ok(t) => t,
                    Err(e) => format!(
                        "<could not read converted file {}: {e}>",
                        out_path.display()
                    ),
                }
            } else {
                // If we cannot locate the output file, show whatever
                // libreoffice printed on stdout.
                stdout.clone().into_owned()
            };

            let mut resp = format!(
                "CONVERT_DOCUMENT tool=libreoffice to={libre_target} path={}\n\n{}",
                input_path.display(),
                body.trim()
            );
            if !stderr.trim().is_empty() {
                resp.push_str(&format!(
                    "\n\n--- libreoffice stderr ---\n{}",
                    truncate_response(stderr.trim(), 2000)
                ));
            }
            return truncate_response(&resp, MAX_RESPONSE_CHARS);
        }
        let code = out
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".into());
        let stderr = String::from_utf8_lossy(&out.stderr);
        return format!(
            "CONVERT_DOCUMENT failed: pandoc unavailable, libreoffice exit={code}\nSTDERR: {}",
            truncate_response(stderr.trim(), 4000)
        );
    }

    // --- Neither tool available ---------------------------------------------
    "Error: neither pandoc nor libreoffice found on PATH.\n\
         Install one of:\n  \
         apt install pandoc                    # Debian / Ubuntu\n  \
         brew install pandoc                   # macOS\n  \
         apt install libreoffice               # Debian / Ubuntu\n  \
         brew install --cask libreoffice       # macOS"
        .to_string()
}

// ---------------------------------------------------------------------------
// 2. convert_pdf_extract_work
// ---------------------------------------------------------------------------

/// Extract text from a PDF using the `pdftotext` CLI.
///
/// * `pages` — optional page range like `"1-5"` (passed as `-f 1 -l 5`).
/// * `max_chars` — truncation cap (default 50 000).
pub fn convert_pdf_extract_work(
    pdf_path: PathBuf,
    pages: Option<String>,
    max_chars: Option<usize>,
) -> String {
    if let Err(e) = ensure_file(&pdf_path) {
        return e;
    }

    let max = max_chars
        .unwrap_or(DEFAULT_PDF_MAX_CHARS)
        .clamp(1024, 500_000);

    let mut cmd = Command::new("pdftotext");
    cmd.arg("-layout");

    // Parse optional page range "start-end".
    if let Some(ref p) = pages {
        let p = p.trim();
        if !p.is_empty() {
            match parse_page_range(p) {
                Ok((first, last)) => {
                    cmd.args(["-f", &first.to_string(), "-l", &last.to_string()]);
                }
                Err(e) => return format!("Error: invalid page range: {e}"),
            }
        }
    }

    cmd.arg(&pdf_path);
    cmd.arg("-"); // write to stdout

    match cmd.output() {
        Ok(out) => {
            if !out.status.success() {
                let code = out
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into());
                let stderr = String::from_utf8_lossy(&out.stderr);
                return format!(
                    "CONVERT_PDF_EXTRACT failed: pdftotext exit={code}\nSTDERR: {}",
                    truncate_response(stderr.trim(), 4000)
                );
            }
            let body = String::from_utf8_lossy(&out.stdout);
            let header = format!(
                "CONVERT_PDF_EXTRACT path={} pages={}\n\n",
                pdf_path.display(),
                pages.as_deref().unwrap_or("all")
            );
            let full = format!("{}{}", header, body.trim());
            truncate_response(&full, max)
        }
        Err(e) => {
            format!(
                "Error: pdftotext not found on PATH: {e}\n\
                 Install with:\n  \
                 apt install poppler-utils     # Debian / Ubuntu\n  \
                 brew install poppler          # macOS"
            )
        }
    }
}

/// Parse a page range string like `"1-5"` into `(first, last)`.
fn parse_page_range(s: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("expected \"start-end\", got {s:?}"));
    }
    let first: u32 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("invalid start page {:?}", parts[0].trim()))?;
    let last: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("invalid end page {:?}", parts[1].trim()))?;
    if first == 0 || last == 0 {
        return Err("page numbers must be >= 1".into());
    }
    if first > last {
        return Err(format!("start page {first} > end page {last}"));
    }
    Ok((first, last))
}

// ---------------------------------------------------------------------------
// 3. convert_archive_list_work
// ---------------------------------------------------------------------------

/// List the contents of an archive (tar, tar.gz, tar.bz2, tar.xz, zip, 7z).
///
/// The archive format is auto-detected from the file extension.
pub fn convert_archive_list_work(archive_path: PathBuf) -> String {
    if let Err(e) = ensure_file(&archive_path) {
        return e;
    }

    let kind = match classify_archive(&archive_path) {
        Ok(k) => k,
        Err(e) => return e,
    };

    let result = match kind {
        "tar" => list_tar(&archive_path),
        "zip" => list_zip(&archive_path),
        "7z" => list_7z(&archive_path),
        _ => unreachable!(),
    };

    match result {
        Ok(body) => {
            let header = format!(
                "CONVERT_ARCHIVE_LIST format={kind} path={}\n\n",
                archive_path.display()
            );
            truncate_response(&format!("{}{}", header, body.trim()), MAX_RESPONSE_CHARS)
        }
        Err(e) => format!(
            "CONVERT_ARCHIVE_LIST format={kind} path={}\nError: {e}",
            archive_path.display()
        ),
    }
}

fn list_tar(path: &Path) -> Result<String, String> {
    run_command(Command::new("tar").args(["-tf"]).arg(path))
}

fn list_zip(path: &Path) -> Result<String, String> {
    // Prefer `zipinfo` for richer output; fall back to `unzip -l`.
    if Command::new("zipinfo").arg("--help").output().is_ok() {
        run_command(Command::new("zipinfo").arg("-1").arg(path))
    } else {
        run_command(Command::new("unzip").args(["-l"]).arg(path))
    }
}

fn list_7z(path: &Path) -> Result<String, String> {
    run_command(Command::new("7z").args(["l", "-ba"]).arg(path))
}

// ---------------------------------------------------------------------------
// 4. convert_archive_extract_work
// ---------------------------------------------------------------------------

/// Extract an archive into `output_dir`.
///
/// If `members` is `Some(...)`, only those entries are extracted (tar / zip).
/// Returns a listing of the extracted files.
pub fn convert_archive_extract_work(
    archive_path: PathBuf,
    output_dir: PathBuf,
    members: Option<Vec<String>>,
) -> String {
    if let Err(e) = ensure_file(&archive_path) {
        return e;
    }
    if let Err(e) = ensure_dir(&output_dir) {
        return e;
    }

    let kind = match classify_archive(&archive_path) {
        Ok(k) => k,
        Err(e) => return e,
    };

    let result = match kind {
        "tar" => extract_tar(&archive_path, &output_dir, members.as_deref()),
        "zip" => extract_zip(&archive_path, &output_dir, members.as_deref()),
        "7z" => extract_7z(&archive_path, &output_dir, members.as_deref()),
        _ => unreachable!(),
    };

    match result {
        Ok(body) => {
            // Walk the output directory so we can report what was extracted.
            let extracted = list_extracted_files(&output_dir);
            let header = format!(
                "CONVERT_ARCHIVE_EXTRACT format={kind} path={} output_dir={}\n",
                archive_path.display(),
                output_dir.display()
            );
            let mut resp = header;
            if !body.trim().is_empty() {
                resp.push_str(&format!("\n--- command output ---\n{}\n", body.trim()));
            }
            resp.push_str(&format!("\n--- extracted files ---\n{}", extracted));
            truncate_response(&resp, MAX_RESPONSE_CHARS)
        }
        Err(e) => format!(
            "CONVERT_ARCHIVE_EXTRACT format={kind} path={}\nError: {e}",
            archive_path.display()
        ),
    }
}

fn extract_tar(path: &Path, dir: &Path, members: Option<&[String]>) -> Result<String, String> {
    let mut cmd = Command::new("tar");
    cmd.args(["-xf"]).arg(path).args(["-C"]).arg(dir);
    if let Some(m) = members {
        for member in m {
            cmd.arg(member);
        }
    }
    run_command(&mut cmd)
}

fn extract_zip(path: &Path, dir: &Path, members: Option<&[String]>) -> Result<String, String> {
    let mut cmd = Command::new("unzip");
    cmd.arg("-o").arg(path).args(["-d"]).arg(dir);
    if let Some(m) = members {
        for member in m {
            cmd.arg(member);
        }
    }
    run_command(&mut cmd)
}

fn extract_7z(path: &Path, dir: &Path, members: Option<&[String]>) -> Result<String, String> {
    let mut cmd = Command::new("7z");
    cmd.args(["x", "-y"])
        .arg(format!("-o{}", dir.display()))
        .arg(path);
    if let Some(m) = members {
        for member in m {
            cmd.arg(member);
        }
    }
    run_command(&mut cmd)
}

/// Recursively list files under `dir`, one per line (limited to 500 entries).
fn list_extracted_files(dir: &Path) -> String {
    let mut entries = Vec::new();
    let _ = walk_dir(dir, &mut entries, 500);
    if entries.is_empty() {
        return "<no files found>".into();
    }
    entries.join("\n")
}

fn walk_dir(dir: &Path, out: &mut Vec<String>, limit: usize) -> std::io::Result<()> {
    if out.len() >= limit {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, out, limit)?;
        } else if out.len() < limit {
            out.push(path.display().to_string());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. convert_binary_hexdump_work
// ---------------------------------------------------------------------------

/// Produce a hex dump of a binary file (pure Rust — no external CLI).
///
/// * `offset` — start byte (default 0).
/// * `length` — maximum bytes to read (default 1024).
/// * `width` — bytes per line (default 16).
///
/// Output format per line:
/// ```text
/// 00000000 | 7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 | .ELF............
/// ```
pub fn convert_binary_hexdump_work(
    binary_path: PathBuf,
    offset: Option<u64>,
    length: Option<u64>,
    width: Option<usize>,
) -> String {
    if let Err(e) = ensure_file(&binary_path) {
        return e;
    }

    let offset = offset.unwrap_or(0);
    let length = length.unwrap_or(DEFAULT_HEXDUMP_LENGTH).min(64 * 1024); // hard cap 64 KiB
    let width = width.unwrap_or(DEFAULT_HEXDUMP_WIDTH).clamp(1, 32);

    let mut file = match fs::File::open(&binary_path) {
        Ok(f) => f,
        Err(e) => return format!("Error: cannot open {}: {e}", binary_path.display()),
    };

    // Seek to the requested offset.
    if offset > 0 {
        use std::io::SeekFrom;
        if let Err(e) = std::io::Seek::seek(&mut file, SeekFrom::Start(offset)) {
            return format!("Error: seek to offset {offset}: {e}");
        }
    }

    let mut buf = vec![0u8; length as usize];
    let bytes_read = match file.read(&mut buf) {
        Ok(n) => n,
        Err(e) => return format!("Error: read {}: {e}", binary_path.display()),
    };
    buf.truncate(bytes_read);

    if buf.is_empty() {
        return format!(
            "CONVERT_BINARY_HEXDUMP path={} offset={offset} bytes_read=0\n\n<empty / past end of file>",
            binary_path.display()
        );
    }

    let mut lines = Vec::new();
    for chunk_start in (0..buf.len()).step_by(width) {
        let chunk = &buf[chunk_start..std::cmp::min(chunk_start + width, buf.len())];
        let addr = offset + chunk_start as u64;

        // Address
        let mut line = format!("{addr:08x} | ");

        // Hex bytes
        for (i, &b) in chunk.iter().enumerate() {
            if i == width / 2 && width > 1 {
                line.push(' '); // extra gap at midpoint
            }
            line.push_str(&format!("{b:02x} "));
        }

        // Padding so the ASCII column aligns even for short last rows.
        let hex_width = width * 3 + if width > 1 { 1 } else { 0 };
        let hex_written = chunk.len() * 3
            + if chunk.len() > width / 2 && width > 1 {
                1
            } else {
                0
            };
        if hex_written < hex_width {
            for _ in 0..hex_width - hex_written {
                line.push(' ');
            }
        }

        line.push_str("| ");

        // ASCII representation
        for &b in chunk.iter() {
            let ch = if (0x20..=0x7e).contains(&b) {
                b as char
            } else {
                '.'
            };
            line.push(ch);
        }

        lines.push(line);
    }

    let header = format!(
        "CONVERT_BINARY_HEXDUMP path={} offset={offset} bytes_read={bytes_read} width={width}\n\n",
        binary_path.display()
    );

    let body = lines.join("\n");
    truncate_response(&format!("{}{}", header, body), MAX_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 6. convert_encoding_work
// ---------------------------------------------------------------------------

/// Convert a text file from one character encoding to another using `iconv`.
///
/// * `from_encoding` — source encoding (e.g. `"ISO-8859-1"`, `"Windows-1252"`).
/// * `to_encoding` — target encoding (e.g. `"UTF-8"`).
///
/// Returns the converted text (truncated to the response limit).
pub fn convert_encoding_work(
    input_path: PathBuf,
    from_encoding: String,
    to_encoding: String,
) -> String {
    if let Err(e) = ensure_file(&input_path) {
        return e;
    }

    let from = from_encoding.trim();
    let to = to_encoding.trim();

    if from.is_empty() || to.is_empty() {
        return "Error: from_encoding and to_encoding must not be empty".to_string();
    }

    // Basic sanity: reject characters that could enable shell injection
    // (iconv names are alphanumeric with hyphens/underscores/dots only).
    for enc in [&from, &to] {
        if !enc
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return format!(
                "Error: encoding name {enc:?} contains disallowed characters (allowed: alnum, '-', '_', '.')"
            );
        }
    }

    match Command::new("iconv")
        .args(["-f", from, "-t", to])
        .arg(&input_path)
        .output()
    {
        Ok(out) => {
            if !out.status.success() {
                let code = out
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into());
                let stderr = String::from_utf8_lossy(&out.stderr);
                return format!(
                    "CONVERT_ENCODING failed: iconv exit={code}\nSTDERR: {}",
                    truncate_response(stderr.trim(), 4000)
                );
            }
            // The converted bytes might not be valid UTF-8 if the user chose a
            // non-UTF-8 target. In that case, show a hex preview.
            let body = match String::from_utf8(out.stdout) {
                Ok(s) => s,
                Err(err) => {
                    let bytes = err.into_bytes();
                    let preview = bytes
                        .iter()
                        .take(256)
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    return format!(
                        "CONVERT_ENCODING path={} from={from} to={to}\n\
                         Note: output is not valid UTF-8 ({} bytes).\n\
                         Hex preview (first 256 bytes):\n{preview}",
                        input_path.display(),
                        bytes.len()
                    );
                }
            };
            let header = format!(
                "CONVERT_ENCODING path={} from={from} to={to}\n\n",
                input_path.display()
            );
            truncate_response(&format!("{}{}", header, body.trim()), MAX_RESPONSE_CHARS)
        }
        Err(e) => {
            format!(
                "Error: iconv not found on PATH: {e}\n\
                 Install with:\n  \
                 apt install libc-bin          # Debian / Ubuntu (usually pre-installed)\n  \
                 brew install libiconv         # macOS (usually pre-installed)"
            )
        }
    }
}

// ---------------------------------------------------------------------------
// 7. convert_qr_generate_work
// ---------------------------------------------------------------------------

/// Generate a QR code using the `qrencode` CLI.
///
/// * If `output_path` is provided, a PNG image is written there.
/// * If `output_path` is `None`, the QR code is returned as terminal
///   ANSI/UTF-8 art.
/// * `size` — pixel size per QR module (default 4, only affects PNG output).
pub fn convert_qr_generate_work(
    text: String,
    output_path: Option<PathBuf>,
    size: Option<u32>,
) -> String {
    if text.is_empty() {
        return "Error: text must not be empty".to_string();
    }

    let pixel_size = size.unwrap_or(DEFAULT_QR_PIXEL_SIZE).clamp(1, 16);

    if let Some(ref path) = output_path {
        // PNG output
        let parent = path.parent();
        if let Some(dir) = parent {
            if !dir.as_os_str().is_empty() {
                if let Err(e) = ensure_dir(dir) {
                    return e;
                }
            }
        }

        match Command::new("qrencode")
            .args(["-o"])
            .arg(path)
            .args(["-s", &pixel_size.to_string()])
            .arg(&text)
            .output()
        {
            Ok(out) => {
                if !out.status.success() {
                    let code = out
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".into());
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return format!(
                        "CONVERT_QR_GENERATE failed: qrencode exit={code}\nSTDERR: {}",
                        truncate_response(stderr.trim(), 4000)
                    );
                }
                let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                format!(
                    "CONVERT_QR_GENERATE mode=png output={} size={pixel_size} text_len={} file_bytes={file_size}\n\
                     QR code written successfully.",
                    path.display(),
                    text.len()
                )
            }
            Err(e) => {
                format!(
                    "Error: qrencode not found on PATH: {e}\n\
                     Install with:\n  \
                     apt install qrencode          # Debian / Ubuntu\n  \
                     brew install qrencode         # macOS"
                )
            }
        }
    } else {
        // Terminal / ANSI UTF-8 output
        match Command::new("qrencode")
            .args(["-t", "ANSIUTF8"])
            .arg(&text)
            .output()
        {
            Ok(out) => {
                if !out.status.success() {
                    let code = out
                        .status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "signal".into());
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return format!(
                        "CONVERT_QR_GENERATE failed: qrencode exit={code}\nSTDERR: {}",
                        truncate_response(stderr.trim(), 4000)
                    );
                }
                let body = String::from_utf8_lossy(&out.stdout);
                let header = format!(
                    "CONVERT_QR_GENERATE mode=terminal text_len={}\n\n",
                    text.len()
                );
                truncate_response(&format!("{}{}", header, body.trim()), MAX_RESPONSE_CHARS)
            }
            Err(e) => {
                format!(
                    "Error: qrencode not found on PATH: {e}\n\
                     Install with:\n  \
                     apt install qrencode          # Debian / Ubuntu\n  \
                     brew install qrencode         # macOS"
                )
            }
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ---- helpers ----

    /// Create a temporary file with the given content and extension.
    fn temp_file(name: &str, content: &[u8]) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::with_prefix(name).expect("temp file");
        f.write_all(content).expect("write temp");
        f.flush().expect("flush");
        f
    }

    // ---- truncate_response ----

    #[test]
    fn truncate_response_does_not_truncate_short_strings() {
        let s = "hello".to_string();
        let out = truncate_response(&s, 100);
        assert_eq!(out, "hello");
        assert!(!out.contains("[truncated"));
    }

    #[test]
    fn truncate_response_truncates_long_strings() {
        let s: String = "x".repeat(300);
        let out = truncate_response(&s, 100);
        assert!(out.contains("[truncated to 100 chars]"));
        assert!(out.chars().count() < 300);
    }

    // ---- extension_lower ----

    #[test]
    fn extension_lower_basic() {
        assert_eq!(extension_lower(Path::new("foo.TAR.GZ")), "gz");
        assert_eq!(extension_lower(Path::new("archive.zip")), "zip");
        assert_eq!(extension_lower(Path::new("noext")), "");
    }

    // ---- is_tar_compressed ----

    #[test]
    fn is_tar_compressed_detects_compound_extensions() {
        assert!(is_tar_compressed(Path::new("a.tar.gz")));
        assert!(is_tar_compressed(Path::new("a.TGZ")));
        assert!(is_tar_compressed(Path::new("a.tar.bz2")));
        assert!(is_tar_compressed(Path::new("a.TBZ2")));
        assert!(is_tar_compressed(Path::new("a.tar.xz")));
        assert!(is_tar_compressed(Path::new("a.TXZ")));
        assert!(!is_tar_compressed(Path::new("a.zip")));
        assert!(!is_tar_compressed(Path::new("a.gz")));
    }

    // ---- classify_archive ----

    #[test]
    fn classify_archive_tar() {
        assert_eq!(classify_archive(Path::new("x.tar")), Ok("tar"));
        assert_eq!(classify_archive(Path::new("x.tar.gz")), Ok("tar"));
        assert_eq!(classify_archive(Path::new("x.tgz")), Ok("tar"));
    }

    #[test]
    fn classify_archive_zip() {
        assert_eq!(classify_archive(Path::new("x.zip")), Ok("zip"));
        assert_eq!(classify_archive(Path::new("x.jar")), Ok("zip"));
    }

    #[test]
    fn classify_archive_7z() {
        assert_eq!(classify_archive(Path::new("x.7z")), Ok("7z"));
    }

    #[test]
    fn classify_archive_unknown() {
        assert!(classify_archive(Path::new("x.rar")).is_err());
    }

    // ---- parse_page_range ----

    #[test]
    fn parse_page_range_valid() {
        assert_eq!(parse_page_range("1-5"), Ok((1, 5)));
        assert_eq!(parse_page_range("10-20"), Ok((10, 20)));
    }

    #[test]
    fn parse_page_range_invalid() {
        assert!(parse_page_range("5").is_err());
        assert!(parse_page_range("0-5").is_err());
        assert!(parse_page_range("5-3").is_err());
        assert!(parse_page_range("a-b").is_err());
    }

    // ---- convert_document_work: missing file ----

    #[test]
    fn convert_document_missing_file() {
        let result = convert_document_work(PathBuf::from("/nonexistent/file.docx"), None);
        assert!(result.starts_with("Error: not a file"));
    }

    // ---- convert_pdf_extract_work: missing file ----

    #[test]
    fn convert_pdf_extract_missing_file() {
        let result = convert_pdf_extract_work(PathBuf::from("/nonexistent/file.pdf"), None, None);
        assert!(result.starts_with("Error: not a file"));
    }

    #[test]
    fn convert_pdf_extract_bad_page_range() {
        let f = temp_file("test", b"dummy pdf content");
        let result = convert_pdf_extract_work(f.path().to_path_buf(), Some("bad".into()), None);
        assert!(result.to_lowercase().contains("invalid page range"));
    }

    // ---- convert_archive_list_work: missing file ----

    #[test]
    fn convert_archive_list_missing_file() {
        let result = convert_archive_list_work(PathBuf::from("/nonexistent/file.zip"));
        assert!(result.starts_with("Error: not a file"));
    }

    #[test]
    fn convert_archive_list_unsupported_ext() {
        let f = temp_file("test.rar", b"");
        let result = convert_archive_list_work(f.path().to_path_buf());
        assert!(result.contains("unsupported archive extension"));
    }

    // ---- convert_archive_extract_work: missing file ----

    #[test]
    fn convert_archive_extract_missing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let result = convert_archive_extract_work(
            PathBuf::from("/nonexistent/file.zip"),
            tmp.path().to_path_buf(),
            None,
        );
        assert!(result.starts_with("Error: not a file"));
    }

    // ---- convert_binary_hexdump_work ----

    #[test]
    fn hexdump_basic() {
        let f = temp_file("hex", b"Hello, world!");
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), None, None, None);
        assert!(result.contains("CONVERT_BINARY_HEXDUMP"));
        assert!(result.contains("48 65 6c 6c 6f")); // "Hello"
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn hexdump_with_offset() {
        let f = temp_file("hex", b"ABCDEFGHIJ");
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), Some(3), None, None);
        assert!(result.contains("offset=3"));
        assert!(result.contains("44 45 46")); // "DEF"
        assert!(!result.contains("41 42 43")); // "ABC" should not appear at start
    }

    #[test]
    fn hexdump_with_length() {
        let f = temp_file("hex", b"ABCDEFGHIJ");
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), None, Some(3), None);
        assert!(result.contains("bytes_read=3"));
    }

    #[test]
    fn hexdump_non_printable() {
        let data = vec![0x00, 0x01, 0x7f, 0x80, 0xff, 0x41];
        let f = temp_file("hex", &data);
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), None, None, None);
        // Non-printable bytes render as '.'; 0x7f (DEL) is also non-printable,
        // so all 5 non-printable bytes become dots: ".....A"
        assert!(result.contains(".....A"));
    }

    #[test]
    fn hexdump_missing_file() {
        let result =
            convert_binary_hexdump_work(PathBuf::from("/nonexistent/file.bin"), None, None, None);
        assert!(result.starts_with("Error: not a file"));
    }

    #[test]
    fn hexdump_custom_width() {
        let data = vec![0u8; 32];
        let f = temp_file("hex", &data);
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), None, None, Some(8));
        assert!(result.contains("width=8"));
        // 32 bytes / 8 per line = 4 lines
        let lines: Vec<&str> = result
            .lines()
            .skip_while(|l| !l.contains("00000000"))
            .collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn hexdump_empty_file() {
        let f = temp_file("empty", b"");
        let result = convert_binary_hexdump_work(f.path().to_path_buf(), None, None, None);
        assert!(result.contains("bytes_read=0"));
    }

    // ---- convert_encoding_work: missing file ----

    #[test]
    fn convert_encoding_missing_file() {
        let result = convert_encoding_work(
            PathBuf::from("/nonexistent/file.txt"),
            "ISO-8859-1".into(),
            "UTF-8".into(),
        );
        assert!(result.starts_with("Error: not a file"));
    }

    #[test]
    fn convert_encoding_empty_from() {
        let f = temp_file("enc", b"hello");
        let result = convert_encoding_work(f.path().to_path_buf(), "".into(), "UTF-8".into());
        assert!(result.contains("must not be empty"));
    }

    #[test]
    fn convert_encoding_invalid_name() {
        let f = temp_file("enc", b"hello");
        let result =
            convert_encoding_work(f.path().to_path_buf(), "bad;name".into(), "UTF-8".into());
        assert!(result.contains("disallowed characters"));
    }

    // ---- convert_qr_generate_work ----

    #[test]
    fn convert_qr_empty_text() {
        let result = convert_qr_generate_work("".to_string(), None, None);
        assert!(result.starts_with("Error:"));
    }

    // ---- walk_dir / list_extracted_files ----

    #[test]
    fn list_extracted_files_walks_recursively() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).expect("mkdir");
        fs::write(tmp.path().join("a.txt"), b"a").expect("write");
        fs::write(sub.join("b.txt"), b"b").expect("write");

        let listing = list_extracted_files(tmp.path());
        assert!(listing.contains("a.txt"));
        assert!(listing.contains("b.txt"));
    }

    // ---- ensure_file / ensure_dir ----

    #[test]
    fn ensure_file_rejects_missing() {
        assert!(ensure_file(Path::new("/no/such/file")).is_err());
    }

    #[test]
    fn ensure_dir_creates_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let new_dir = tmp.path().join("new_sub");
        assert!(ensure_dir(&new_dir).is_ok());
        assert!(new_dir.is_dir());
    }
}
