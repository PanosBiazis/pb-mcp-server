//! # OCR (Optical Character Recognition) Tools
//!
//! Comprehensive OCR tools for AI agents to extract text from images and documents.
//!
//! All OCR operations rely on **Tesseract** as the primary backend, with supporting
//! tools: **pdftoppm** (Poppler) or **convert/magick** (ImageMagick) for PDF rendering,
//! **xclip** for clipboard access, and **scrot** or **import** (ImageMagick) for screen
//! capture. Functions return human-readable strings; errors are prefixed with `"Error:"`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum input image file size for OCR (50 MiB).
const MAX_OCR_FILE_BYTES: u64 = 50 * 1024 * 1024;

/// Supported image extensions for Tesseract input.
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "tif", "tiff", "webp", "bmp", "pnm", "gif", "pbm", "pgm",
];

/// Default Tesseract language.
const DEFAULT_LANG: &str = "eng";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate that `path` exists, is a file, and is under the size limit.
/// Returns the file metadata or an error string starting with `"Error:"`.
fn validate_file_path(path: &Path) -> Result<fs::Metadata, String> {
    if !path.is_file() {
        return Err(format!("Error: not a file: {}", path.display()));
    }
    let meta =
        fs::metadata(path).map_err(|e| format!("Error: cannot stat {}: {e}", path.display()))?;
    if meta.len() > MAX_OCR_FILE_BYTES {
        return Err(format!(
            "Error: file too large ({} bytes, max {} MiB)",
            meta.len(),
            MAX_OCR_FILE_BYTES / (1024 * 1024)
        ));
    }
    Ok(meta)
}

/// Validate that `path` exists, is a file (allowing directories for some ops),
/// and is under the size limit.
fn validate_image_path(path: &Path) -> Result<fs::Metadata, String> {
    let meta = validate_file_path(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !SUPPORTED_IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!(
            "Error: unsupported image extension .{ext} (allowed: {})",
            SUPPORTED_IMAGE_EXTENSIONS.join(",")
        ));
    }
    Ok(meta)
}

/// Sanitize and default the Tesseract language parameter.
/// Returns a validated language string or an error.
fn sanitize_language(lang: &Option<String>) -> Result<String, String> {
    let lang = match lang {
        Some(l) if !l.trim().is_empty() => l.trim().to_string(),
        _ => DEFAULT_LANG.to_string(),
    };
    if !lang
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
    {
        return Err("Error: lang must be ASCII alnum with optional _ + -".to_string());
    }
    Ok(lang)
}

/// Check whether a CLI program exists on `PATH` by running `<program> --version`.
fn cli_exists(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

/// Require a CLI program to exist; return an error string if not found.
fn require_cli(program: &str) -> Result<(), String> {
    if cli_exists(program) {
        Ok(())
    } else {
        Err(format!("Error: `{program}` not found on PATH"))
    }
}

/// Try to find an available ImageMagick binary (`magick` first, then `convert`).
/// Returns the program name or an error string.
fn find_imagemagick() -> Result<&'static str, String> {
    for cmd in &["magick", "convert"] {
        if cli_exists(cmd) {
            return Ok(cmd);
        }
    }
    Err("Error: neither `magick` nor `convert` (ImageMagick) found on PATH".to_string())
}

/// Try to find an available PDF-to-image converter (`pdftoppm` first, then ImageMagick).
/// Returns the program name or an error string.
fn find_pdf_renderer() -> Result<String, String> {
    if cli_exists("pdftoppm") {
        return Ok("pdftoppm".to_string());
    }
    if cli_exists("magick") || cli_exists("convert") {
        return Ok("imagemagick".to_string());
    }
    Err(
        "Error: neither `pdftoppm` (Poppler) nor ImageMagick found on PATH for PDF rendering"
            .to_string(),
    )
}

/// Run a command and return its stdout, or an error string on failure.
fn _run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
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

/// Run Tesseract on a single image path, returning the extracted text.
/// Assumes the caller has already validated the path and language.
fn run_tesseract(path: &Path, lang: &str, extra_args: &[&str]) -> Result<String, String> {
    let mut args = vec![path.to_string_lossy().to_string(), "stdout".to_string()];
    args.push("-l".to_string());
    args.push(lang.to_string());
    for a in extra_args {
        args.push(a.to_string());
    }
    match Command::new("tesseract").args(&args).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return Err(format!(
                    "Error: tesseract exit={:?}\npath={}\nSTDERR:\n{se}",
                    o.status.code(),
                    path.display()
                ));
            }
            let mut result = stdout.trim().to_string();
            if !stderr.trim().is_empty() {
                let (w, _) = super::truncate_chars(stderr.trim(), 1200);
                result.push_str(&format!("\n\n--- tesseract stderr (info) ---\n{w}\n"));
            }
            Ok(result)
        }
        Err(e) => Err(format!("Error: spawn tesseract: {e}")),
    }
}

// ---------------------------------------------------------------------------
// 1. ocr_file
// ---------------------------------------------------------------------------

/// OCR a single image file using Tesseract.
///
/// `language` defaults to `"eng"`. Uses the `tesseract` CLI subprocess.
/// Returns the extracted text, or an `"Error:"` prefixed string on failure.
pub fn ocr_file_work(path: PathBuf, language: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = validate_image_path(&path) {
        return e;
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    match run_tesseract(&path, &lang, &[]) {
        Ok(text) => {
            let (body, clipped) = super::truncate_chars(&text, super::MAX_RESPONSE_CHARS);
            let mut s = format!("OCR_FILE path={} lang={lang}\n\n{body}\n", path.display());
            if clipped {
                s.push_str(&format!(
                    "\n[truncated to {} chars]\n",
                    super::MAX_RESPONSE_CHARS
                ));
            }
            s
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 2. ocr_pdf
// ---------------------------------------------------------------------------

/// OCR a PDF document.
///
/// Converts pages to images via `pdftoppm` (Poppler) or `convert` (ImageMagick),
/// then runs Tesseract on each page image. Supports a page range like `"1-5"`.
/// `language` defaults to `"eng"`.
pub fn ocr_pdf_work(path: PathBuf, language: Option<String>, page_range: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if !path.is_file() {
        return format!("Error: not a file: {}", path.display());
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext != "pdf" {
        return format!("Error: expected .pdf file, got .{ext}");
    }
    if let Ok(m) = fs::metadata(&path) {
        if m.len() > MAX_OCR_FILE_BYTES {
            return format!(
                "Error: file too large for OCR (max {} MiB)",
                MAX_OCR_FILE_BYTES / (1024 * 1024)
            );
        }
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    let renderer = match find_pdf_renderer() {
        Ok(r) => r,
        Err(e) => return e,
    };

    // Create a temp directory for page images.
    let tmp_dir = match std::env::temp_dir().join("pb-mcp-ocr-pdf") {
        td => {
            if let Err(e) = fs::create_dir_all(&td) {
                return format!("Error: cannot create temp dir {}: {e}", td.display());
            }
            td
        }
    };

    // Parse page range.
    let (first_page, last_page) = parse_page_range(page_range.as_deref());

    // Convert PDF pages to images.
    let image_paths = match renderer.as_str() {
        "pdftoppm" => render_pdf_pdftoppm(&path, &tmp_dir, first_page, last_page),
        "imagemagick" => render_pdf_imagemagick(&path, &tmp_dir, first_page, last_page),
        _ => return format!("Error: unknown renderer {renderer}"),
    };

    let image_paths = match image_paths {
        Ok(p) => p,
        Err(e) => return e,
    };

    if image_paths.is_empty() {
        return "Error: no page images generated from PDF".to_string();
    }

    let mut results = Vec::new();
    results.push(format!(
        "OCR_PDF path={} lang={lang} pages={}\n",
        path.display(),
        image_paths.len()
    ));

    for (i, img_path) in image_paths.iter().enumerate() {
        let page_num = match first_page {
            Some(fp) => fp + i as u32,
            None => i as u32 + 1,
        };
        match run_tesseract(img_path, &lang, &[]) {
            Ok(text) => {
                let (body, clipped) = super::truncate_chars(
                    &text,
                    super::MAX_RESPONSE_CHARS / image_paths.len().max(1),
                );
                results.push(format!("--- Page {page_num} ---\n{body}"));
                if clipped {
                    results.push(format!("[page {page_num} truncated]"));
                }
            }
            Err(e) => results.push(format!("--- Page {page_num} ---\n{e}")),
        }
        // Clean up temp image.
        let _ = fs::remove_file(img_path);
    }

    let full = results.join("\n\n");
    let (body, clipped) = super::truncate_chars(&full, super::MAX_RESPONSE_CHARS);
    let mut s = body;
    if clipped {
        s.push_str(&format!(
            "\n\n[truncated to {} chars]\n",
            super::MAX_RESPONSE_CHARS
        ));
    }
    s
}

/// Parse a page range string like "1-5" or "3" into (first_page, last_page).
/// Pages are 1-indexed. Returns (None, None) if no range specified.
fn parse_page_range(range: Option<&str>) -> (Option<u32>, Option<u32>) {
    let range = match range {
        Some(r) if !r.trim().is_empty() => r.trim(),
        _ => return (None, None),
    };
    if let Some((a, b)) = range.split_once('-') {
        let first = a.trim().parse::<u32>().ok();
        let last = b.trim().parse::<u32>().ok();
        (first, last)
    } else {
        let n = range.parse::<u32>().ok();
        (n, n)
    }
}

/// Render PDF pages to PNG images using `pdftoppm`.
fn render_pdf_pdftoppm(
    pdf_path: &Path,
    tmp_dir: &Path,
    first_page: Option<u32>,
    last_page: Option<u32>,
) -> Result<Vec<PathBuf>, String> {
    let prefix = tmp_dir.join("page");
    let mut args = vec![
        "-png".to_string(),
        "-r".to_string(),
        "300".to_string(), // 300 DPI for good OCR quality
    ];
    if let Some(fp) = first_page {
        args.push("-f".to_string());
        args.push(fp.to_string());
    }
    if let Some(lp) = last_page {
        args.push("-l".to_string());
        args.push(lp.to_string());
    }
    args.push(pdf_path.to_string_lossy().to_string());
    args.push(prefix.to_string_lossy().to_string());

    let output = match Command::new("pdftoppm").args(&args).output() {
        Ok(o) => o,
        Err(e) => return Err(format!("Error: spawn pdftoppm: {e}")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let (se, _) = super::truncate_chars(stderr.trim(), 4000);
        return Err(format!(
            "Error: pdftoppm exit={:?}\nSTDERR:\n{se}",
            output.status.code()
        ));
    }

    // pdftoppm outputs files as prefix-1.png, prefix-2.png, etc.
    let mut images = Vec::new();
    if let Ok(entries) = fs::read_dir(tmp_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("page-") && name.ends_with(".png") {
                    images.push(p);
                }
            }
        }
    }
    images.sort();
    Ok(images)
}

/// Render PDF pages to PNG images using ImageMagick `convert`/`magick`.
fn render_pdf_imagemagick(
    pdf_path: &Path,
    tmp_dir: &Path,
    first_page: Option<u32>,
    last_page: Option<u32>,
) -> Result<Vec<PathBuf>, String> {
    let im = find_imagemagick()?;
    let out_pattern = tmp_dir.join("page.png").to_string_lossy().to_string();

    // Build the input path with page range for ImageMagick.
    let input = match (first_page, last_page) {
        (Some(f), Some(l)) => format!("{}[{}-{}]", pdf_path.to_string_lossy(), f - 1, l - 1),
        (Some(f), None) => format!("{}[{}-]", pdf_path.to_string_lossy(), f - 1),
        (None, Some(l)) => format!("{}[0-{}]", pdf_path.to_string_lossy(), l - 1),
        (None, None) => pdf_path.to_string_lossy().to_string(),
    };

    let args = vec![
        "-density".to_string(),
        "300".to_string(),
        input,
        out_pattern,
    ];

    let output = match Command::new(im).args(&args).output() {
        Ok(o) => o,
        Err(e) => return Err(format!("Error: spawn {im}: {e}")),
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let (se, _) = super::truncate_chars(stderr.trim(), 4000);
        return Err(format!(
            "Error: {im} exit={:?}\nSTDERR:\n{se}",
            output.status.code()
        ));
    }

    // ImageMagick outputs files as page-0.png, page-1.png, etc.
    let mut images = Vec::new();
    if let Ok(entries) = fs::read_dir(tmp_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("page-") && name.ends_with(".png") {
                    images.push(p);
                }
            }
        }
    }
    images.sort();
    Ok(images)
}

// ---------------------------------------------------------------------------
// 3. ocr_batch
// ---------------------------------------------------------------------------

/// OCR multiple image files in batch, returning results per file.
///
/// Processes each image with Tesseract. `language` defaults to `"eng"`.
/// Files that fail are reported individually without aborting the batch.
pub fn ocr_batch_work(paths: Vec<PathBuf>, language: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if paths.is_empty() {
        return "Error: no paths provided".to_string();
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    let per_file_limit = super::MAX_RESPONSE_CHARS / paths.len().max(1);
    let mut results = Vec::new();
    results.push(format!("OCR_BATCH count={} lang={lang}\n", paths.len()));

    for path in &paths {
        if let Err(e) = validate_image_path(path) {
            results.push(format!("--- {} ---\n{e}\n", path.display()));
            continue;
        }
        match run_tesseract(path, &lang, &[]) {
            Ok(text) => {
                let (body, clipped) = super::truncate_chars(&text, per_file_limit);
                results.push(format!("--- {} ---\n{body}", path.display()));
                if clipped {
                    results.push(format!("[truncated for {}]", path.display()));
                }
            }
            Err(e) => results.push(format!("--- {} ---\n{e}", path.display())),
        }
        results.push(String::new()); // blank line separator
    }

    let full = results.join("\n");
    let (body, clipped) = super::truncate_chars(&full, super::MAX_RESPONSE_CHARS);
    let mut s = body;
    if clipped {
        s.push_str(&format!(
            "\n\n[truncated to {} chars]\n",
            super::MAX_RESPONSE_CHARS
        ));
    }
    s
}

// ---------------------------------------------------------------------------
// 4. ocr_from_clipboard
// ---------------------------------------------------------------------------

/// Capture a screenshot from the clipboard and OCR it.
///
/// Uses `xclip` on Linux to get clipboard image data (requires `xclip -selection clipboard -t image/png -o`).
/// Saves the clipboard image to a temporary file, then runs Tesseract on it.
pub fn ocr_from_clipboard_work() -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = require_cli("xclip") {
        return e;
    }

    let tmp_path = std::env::temp_dir().join("pb-mcp-ocr-clipboard.png");

    // Use xclip to extract clipboard image as PNG.
    let output = match Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "image/png", "-o"])
        .output()
    {
        Ok(o) => o,
        Err(e) => return format!("Error: spawn xclip: {e}"),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return format!(
            "Error: xclip exit={:?}\nSTDERR:\n{}",
            output.status.code(),
            stderr.trim()
        );
    }

    if output.stdout.is_empty() {
        return "Error: clipboard does not contain image data (xclip returned empty output)"
            .to_string();
    }

    // Write clipboard image data to temp file.
    if let Err(e) = fs::write(&tmp_path, &output.stdout) {
        return format!(
            "Error: cannot write clipboard image to {}: {e}",
            tmp_path.display()
        );
    }

    // Validate the temp image.
    if let Ok(m) = fs::metadata(&tmp_path) {
        if m.len() > MAX_OCR_FILE_BYTES {
            let _ = fs::remove_file(&tmp_path);
            return format!(
                "Error: clipboard image too large ({} bytes, max {} MiB)",
                m.len(),
                MAX_OCR_FILE_BYTES / (1024 * 1024)
            );
        }
    }

    let result = match run_tesseract(&tmp_path, DEFAULT_LANG, &[]) {
        Ok(text) => {
            let (body, clipped) = super::truncate_chars(&text, super::MAX_RESPONSE_CHARS);
            let mut s = format!("OCR_CLIPBOARD lang={DEFAULT_LANG}\n\n{body}\n");
            if clipped {
                s.push_str(&format!(
                    "\n[truncated to {} chars]\n",
                    super::MAX_RESPONSE_CHARS
                ));
            }
            s
        }
        Err(e) => e,
    };

    // Clean up temp file.
    let _ = fs::remove_file(&tmp_path);
    result
}

// ---------------------------------------------------------------------------
// 5. ocr_screen_region
// ---------------------------------------------------------------------------

/// Capture a screen region and OCR it.
///
/// Uses `scrot` or `import` (ImageMagick) to capture the specified screen region,
/// then runs Tesseract on the captured image.
/// `language` defaults to `"eng"`.
pub fn ocr_screen_region_work(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    language: Option<String>,
) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if width == 0 || height == 0 {
        return "Error: width and height must be > 0".to_string();
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    let tmp_path = std::env::temp_dir().join("pb-mcp-ocr-screen.png");
    let geometry = format!("{width}x{height}+{x}+{y}");

    // Try scrot first, then import (ImageMagick).
    let capture_ok = if cli_exists("scrot") {
        match Command::new("scrot")
            .args(["-a", &format!("{x},{y},{width},{height}"), "-z"])
            .arg(&tmp_path)
            .output()
        {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    } else {
        false
    };

    let capture_ok = if !capture_ok {
        if let Ok(_im) = find_imagemagick() {
            // Use `import` from ImageMagick for screen region capture.
            // Note: `import -crop WxH+X+Y -window root` captures from the root window.
            match Command::new("import")
                .args(["-window", "root", "-crop", &geometry])
                .arg(&tmp_path)
                .output()
            {
                Ok(o) => o.status.success(),
                Err(_) => {
                    return "Error: neither `scrot` nor `import` (ImageMagick) could capture screen region".to_string();
                }
            }
        } else if !cli_exists("scrot") {
            return "Error: neither `scrot` nor `import` (ImageMagick) found on PATH for screen capture".to_string();
        } else {
            false
        }
    } else {
        true
    };

    if !capture_ok {
        // scrot ran but failed, and we already tried import above — both failed.
        return "Error: screen region capture failed (both scrot and import failed)".to_string();
    }

    if !tmp_path.is_file() {
        return "Error: screen capture produced no output file".to_string();
    }

    let result = match run_tesseract(&tmp_path, &lang, &[]) {
        Ok(text) => {
            let (body, clipped) = super::truncate_chars(&text, super::MAX_RESPONSE_CHARS);
            let mut s = format!(
                "OCR_SCREEN_REGION x={x} y={y} width={width} height={height} lang={lang}\n\n{body}\n"
            );
            if clipped {
                s.push_str(&format!(
                    "\n[truncated to {} chars]\n",
                    super::MAX_RESPONSE_CHARS
                ));
            }
            s
        }
        Err(e) => e,
    };

    // Clean up temp file.
    let _ = fs::remove_file(&tmp_path);
    result
}

// ---------------------------------------------------------------------------
// 6. ocr_table
// ---------------------------------------------------------------------------

/// OCR a table image and attempt to preserve tabular structure.
///
/// Uses Tesseract's TSV output mode (`--oem 3 --psm 6` with TSV) which produces
/// tab-separated values that better preserve the table layout.
/// `language` defaults to `"eng"`.
pub fn ocr_table_work(path: PathBuf, language: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = validate_image_path(&path) {
        return e;
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    // Use Tesseract with TSV output for tabular data.
    // --oem 3 = default OCR Engine Mode, --psm 6 = assume a single uniform block of text.
    let tmp_tsv = std::env::temp_dir().join("pb-mcp-ocr-table-tsv");
    let args = vec![
        path.to_string_lossy().to_string(),
        tmp_tsv.to_string_lossy().to_string(),
        "-l".to_string(),
        lang.clone(),
        "--oem".to_string(),
        "3".to_string(),
        "--psm".to_string(),
        "6".to_string(),
        "tsv".to_string(),
    ];

    let output = match Command::new("tesseract").args(&args).output() {
        Ok(o) => o,
        Err(e) => {
            return format!("Error: spawn tesseract: {e}");
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let (se, _) = super::truncate_chars(stderr.trim(), 4000);
        return format!(
            "Error: tesseract exit={:?}\npath={}\nSTDERR:\n{se}",
            output.status.code(),
            path.display()
        );
    }

    // Tesseract writes to tmp_tsv.tsv
    let tsv_path = {
        let mut p = tmp_tsv.as_os_str().to_string_lossy().to_string();
        p.push_str(".tsv");
        PathBuf::from(&p)
    };

    let tsv_content = match fs::read_to_string(&tsv_path) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&tsv_path);
            return format!("Error: cannot read TSV output: {e}");
        }
    };

    // Clean up temp TSV file.
    let _ = fs::remove_file(&tsv_path);

    // Parse TSV to reconstruct a readable table.
    let table_text = parse_tesseract_tsv_to_table(&tsv_content);

    let (body, clipped) = super::truncate_chars(&table_text, super::MAX_RESPONSE_CHARS);
    let mut s = format!("OCR_TABLE path={} lang={lang}\n\n{body}\n", path.display());
    if clipped {
        s.push_str(&format!(
            "\n[truncated to {} chars]\n",
            super::MAX_RESPONSE_CHARS
        ));
    }
    if !stderr.trim().is_empty() {
        let (w, _) = super::truncate_chars(stderr.trim(), 1200);
        s.push_str(&format!("\n--- tesseract stderr (info) ---\n{w}\n"));
    }
    s
}

/// Parse Tesseract's TSV output into a readable table representation.
///
/// Tesseract TSV format has columns:
/// level, page_num, block_num, par_num, line_num, word_num, left, top, width, height, conf, text
///
/// We group words by line and reconstruct lines with spacing based on position.
fn parse_tesseract_tsv_to_table(tsv: &str) -> String {
    let mut lines: Vec<Vec<(i32, String)>> = Vec::new(); // (left_pos, word)
    let mut current_line = 0u32;

    for row in tsv.lines().skip(1) {
        // skip header
        let cols: Vec<&str> = row.split('\t').collect();
        if cols.len() < 12 {
            continue;
        }
        let _level: u32 = cols[0].parse().unwrap_or(0);
        let line_num: u32 = cols[4].parse().unwrap_or(0);
        let left: i32 = cols[6].parse().unwrap_or(0);
        let conf: i32 = cols[10].parse().unwrap_or(0);
        let text = cols[11].to_string();

        // Skip entries with very low confidence (likely noise)
        if conf < 30 {
            continue;
        }
        // Skip empty text
        if text.trim().is_empty() {
            continue;
        }

        if line_num != current_line || lines.is_empty() {
            current_line = line_num;
            lines.push(Vec::new());
        }

        if let Some(last) = lines.last_mut() {
            last.push((left, text));
        }
    }

    let mut result = Vec::new();
    for line_words in &lines {
        let line_text = reconstruct_line(line_words);
        result.push(line_text);
    }
    result.join("\n")
}

/// Reconstruct a line of text from positioned words, approximating column alignment.
fn reconstruct_line(words: &[(i32, String)]) -> String {
    if words.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    let mut prev_right: Option<i32> = None;

    for (left, word) in words {
        if let Some(pr) = prev_right {
            let gap = left - pr;
            // If gap is large, treat as a column separator (tab or multiple spaces).
            if gap > 50 {
                result.push('\t');
            } else if gap > 15 {
                result.push_str("  ");
            } else {
                result.push(' ');
            }
        }
        result.push_str(word);
        // Approximate word width: ~8 pixels per character.
        prev_right = Some(*left + (word.len() as i32 * 8));
    }
    result
}

// ---------------------------------------------------------------------------
// 7. ocr_with_confidence
// ---------------------------------------------------------------------------

/// OCR with confidence scores for each word/line.
///
/// Uses Tesseract's TSV output mode and parses confidence values from the output.
/// `language` defaults to `"eng"`.
pub fn ocr_with_confidence_work(path: PathBuf, language: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = validate_image_path(&path) {
        return e;
    }
    let lang = match sanitize_language(&language) {
        Ok(l) => l,
        Err(e) => return e,
    };

    let tmp_tsv = std::env::temp_dir().join("pb-mcp-ocr-conf-tsv");
    let args = vec![
        path.to_string_lossy().to_string(),
        tmp_tsv.to_string_lossy().to_string(),
        "-l".to_string(),
        lang.clone(),
        "tsv".to_string(),
    ];

    let output = match Command::new("tesseract").args(&args).output() {
        Ok(o) => o,
        Err(e) => {
            return format!("Error: spawn tesseract: {e}");
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let (se, _) = super::truncate_chars(stderr.trim(), 4000);
        return format!(
            "Error: tesseract exit={:?}\npath={}\nSTDERR:\n{se}",
            output.status.code(),
            path.display()
        );
    }

    let tsv_path = {
        let mut p = tmp_tsv.as_os_str().to_string_lossy().to_string();
        p.push_str(".tsv");
        PathBuf::from(&p)
    };

    let tsv_content = match fs::read_to_string(&tsv_path) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&tsv_path);
            return format!("Error: cannot read TSV output: {e}");
        }
    };

    let _ = fs::remove_file(&tsv_path);

    let confidence_text = parse_tesseract_tsv_with_confidence(&tsv_content);

    let (body, clipped) = super::truncate_chars(&confidence_text, super::MAX_RESPONSE_CHARS);
    let mut s = format!(
        "OCR_WITH_CONFIDENCE path={} lang={lang}\n\n{body}\n",
        path.display()
    );
    if clipped {
        s.push_str(&format!(
            "\n[truncated to {} chars]\n",
            super::MAX_RESPONSE_CHARS
        ));
    }
    if !stderr.trim().is_empty() {
        let (w, _) = super::truncate_chars(stderr.trim(), 1200);
        s.push_str(&format!("\n--- tesseract stderr (info) ---\n{w}\n"));
    }
    s
}

/// Parse Tesseract TSV output to produce confidence-annotated text.
///
/// Groups words by line and shows per-word confidence scores.
fn parse_tesseract_tsv_with_confidence(tsv: &str) -> String {
    let mut lines: Vec<Vec<(i32, String, i32)>> = Vec::new(); // (left, word, confidence)
    let mut current_line = 0u32;

    for row in tsv.lines().skip(1) {
        let cols: Vec<&str> = row.split('\t').collect();
        if cols.len() < 12 {
            continue;
        }
        let line_num: u32 = cols[4].parse().unwrap_or(0);
        let left: i32 = cols[6].parse().unwrap_or(0);
        let conf: i32 = cols[10].parse().unwrap_or(0);
        let text = cols[11].to_string();

        if text.trim().is_empty() {
            continue;
        }

        if line_num != current_line || lines.is_empty() {
            current_line = line_num;
            lines.push(Vec::new());
        }

        if let Some(last) = lines.last_mut() {
            last.push((left, text, conf));
        }
    }

    let mut result = Vec::new();
    for line_words in &lines {
        let text_line: Vec<String> = line_words.iter().map(|(_, w, _)| w.clone()).collect();
        let conf_line: Vec<String> = line_words
            .iter()
            .map(|(_, _, c)| {
                if *c < 50 {
                    format!("LOW({c})")
                } else if *c < 75 {
                    format!("MED({c})")
                } else {
                    format!("{c}")
                }
            })
            .collect();
        result.push(text_line.join(" "));
        result.push(format!("  confidence: {}", conf_line.join(" | ")));
    }
    result.join("\n")
}

// ---------------------------------------------------------------------------
// 8. ocr_preprocess
// ---------------------------------------------------------------------------

/// Preprocess an image for better OCR results.
///
/// Operations: `"grayscale"`, `"threshold"`, `"despeckle"`, `"sharpen"`, `"resize2x"`.
/// Uses ImageMagick `convert`/`magick`.
/// If `output` is `None`, derives the output path by appending a suffix to the input filename.
pub fn ocr_preprocess_work(path: PathBuf, operation: String, output: Option<PathBuf>) -> String {
    if let Err(e) = validate_file_path(&path) {
        return e;
    }
    let im = match find_imagemagick() {
        Ok(m) => m,
        Err(e) => return e,
    };

    let op = operation.trim().to_ascii_lowercase();
    let (suffix, args) = match op.as_str() {
        "grayscale" => (
            "_gray",
            vec!["-colorspace", "Gray"],
        ),
        "threshold" => (
            "_thresh",
            vec!["-colorspace", "Gray", "-threshold", "50%"],
        ),
        "despeckle" => (
            "_despeckle",
            vec!["-despeckle"],
        ),
        "sharpen" => (
            "_sharpen",
            vec!["-sharpen", "0x1"],
        ),
        "resize2x" => (
            "_2x",
            vec!["-resize", "200%"],
        ),
        _ => {
            return format!(
                "Error: unknown operation \"{op}\". Allowed: grayscale, threshold, despeckle, sharpen, resize2x"
            )
        }
    };

    let out_path = match &output {
        Some(p) => p.clone(),
        None => {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{stem}{suffix}.{ext}"))
        }
    };

    let mut cmd_args = vec![path.to_string_lossy().to_string()];
    for a in &args {
        cmd_args.push(a.to_string());
    }
    cmd_args.push(out_path.to_string_lossy().to_string());

    match Command::new(im).args(&cmd_args).output() {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return format!("Error: {im} exit={:?}\nSTDERR:\n{se}", o.status.code());
            }
            format!(
                "OCR_PREPROCESS ok\ninput={}\noperation={op}\noutput={}\n",
                path.display(),
                out_path.display()
            )
        }
        Err(e) => format!("Error: spawn {im}: {e}"),
    }
}

// ---------------------------------------------------------------------------
// 9. ocr_detect_text_regions
// ---------------------------------------------------------------------------

/// Detect text regions in an image (bounding boxes) using Tesseract's makebox mode.
///
/// Returns bounding box coordinates for each detected text region.
pub fn ocr_detect_text_regions_work(path: PathBuf) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = validate_image_path(&path) {
        return e;
    }

    let tmp_box = std::env::temp_dir().join("pb-mcp-ocr-box");
    let args = vec![
        path.to_string_lossy().to_string(),
        tmp_box.to_string_lossy().to_string(),
        "makebox".to_string(),
    ];

    let output = match Command::new("tesseract").args(&args).output() {
        Ok(o) => o,
        Err(e) => {
            return format!("Error: spawn tesseract: {e}");
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let (se, _) = super::truncate_chars(stderr.trim(), 4000);
        return format!(
            "Error: tesseract exit={:?}\npath={}\nSTDERR:\n{se}",
            output.status.code(),
            path.display()
        );
    }

    // Tesseract writes to tmp_box.box
    let box_path = {
        let mut p = tmp_box.as_os_str().to_string_lossy().to_string();
        p.push_str(".box");
        PathBuf::from(&p)
    };

    let box_content = match fs::read_to_string(&box_path) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::remove_file(&box_path);
            return format!("Error: cannot read box output: {e}");
        }
    };

    let _ = fs::remove_file(&box_path);

    // Parse the box file format: <symbol> <left> <bottom> <right> <top> <page>
    let regions = parse_box_file(&box_content);

    let (body, clipped) = super::truncate_chars(&regions, super::MAX_RESPONSE_CHARS);
    let mut s = format!(
        "OCR_DETECT_TEXT_REGIONS path={}\n\n{body}\n",
        path.display()
    );
    if clipped {
        s.push_str(&format!(
            "\n[truncated to {} chars]\n",
            super::MAX_RESPONSE_CHARS
        ));
    }
    if !stderr.trim().is_empty() {
        let (w, _) = super::truncate_chars(stderr.trim(), 1200);
        s.push_str(&format!("\n--- tesseract stderr (info) ---\n{w}\n"));
    }
    s
}

/// Parse Tesseract makebox output into a readable format with bounding boxes.
///
/// Box file format: `<char> <left> <bottom> <right> <top> <page>`
/// We group consecutive characters into text regions (lines/blocks).
fn parse_box_file(content: &str) -> String {
    let mut regions: Vec<(String, i32, i32, i32, i32)> = Vec::new(); // (text, left, bottom, right, top)
    let mut current_text = String::new();
    let mut region_left: i32 = i32::MAX;
    let mut region_bottom: i32 = i32::MAX;
    let mut region_right: i32 = i32::MIN;
    let mut region_top: i32 = i32::MIN;
    let mut prev_line: Option<i32> = None;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }
        let ch = parts[0];
        let left: i32 = parts[1].parse().unwrap_or(0);
        let bottom: i32 = parts[2].parse().unwrap_or(0);
        let right: i32 = parts[3].parse().unwrap_or(0);
        let top: i32 = parts[4].parse().unwrap_or(0);

        // Detect line breaks via large vertical gap.
        let line_avg = (top + bottom) / 2;
        if let Some(pl) = prev_line {
            if (line_avg - pl).unsigned_abs() > 20 && !current_text.is_empty() {
                // New line — flush current region.
                regions.push((
                    current_text.trim().to_string(),
                    region_left,
                    region_bottom,
                    region_right,
                    region_top,
                ));
                current_text = String::new();
                region_left = i32::MAX;
                region_bottom = i32::MAX;
                region_right = i32::MIN;
                region_top = i32::MIN;
            }
        }

        // Append character to current region.
        if ch == " " {
            current_text.push(' ');
        } else {
            current_text.push_str(ch);
        }
        region_left = region_left.min(left);
        region_bottom = region_bottom.min(bottom);
        region_right = region_right.max(right);
        region_top = region_top.max(top);
        prev_line = Some(line_avg);
    }

    // Flush last region.
    if !current_text.trim().is_empty() {
        regions.push((
            current_text.trim().to_string(),
            region_left,
            region_bottom,
            region_right,
            region_top,
        ));
    }

    if regions.is_empty() {
        return "No text regions detected".to_string();
    }

    let mut result = Vec::new();
    result.push(format!("Found {} text region(s):\n", regions.len()));
    for (i, (text, left, bottom, right, top)) in regions.iter().enumerate() {
        let width = right - left;
        let height = top - bottom;
        let (text_preview, _) = super::truncate_chars(text, 60);
        result.push(format!(
            "  [{i}] bbox=({left}, {bottom}, {right}, {top}) size={width}x{height} text=\"{text_preview}\""
        ));
    }
    result.join("\n")
}

// ---------------------------------------------------------------------------
// 10. ocr_list_languages
// ---------------------------------------------------------------------------

/// List available Tesseract language packs using `tesseract --list-langs`.
pub fn ocr_list_languages_work() -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }

    match Command::new("tesseract").args(["--list-langs"]).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return format!(
                    "Error: tesseract --list-langs exit={:?}\nSTDERR:\n{se}",
                    o.status.code()
                );
            }
            let langs: Vec<&str> = stdout
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();
            let mut s = format!(
                "OCR_LIST_LANGUAGES count={}\n\n",
                langs.len().saturating_sub(1) // first line is usually a header
            );
            for lang in &langs {
                s.push_str(&format!("  {lang}\n"));
            }
            let (body, _) = super::truncate_chars(&s, super::MAX_RESPONSE_CHARS);
            body
        }
        Err(e) => format!("Error: spawn tesseract: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_language_default() {
        assert_eq!(sanitize_language(&None).unwrap(), "eng");
        assert_eq!(sanitize_language(&Some(String::new())).unwrap(), "eng");
        assert_eq!(sanitize_language(&Some("  ".to_string())).unwrap(), "eng");
    }

    #[test]
    fn test_sanitize_language_valid() {
        assert_eq!(sanitize_language(&Some("eng".to_string())).unwrap(), "eng");
        assert_eq!(sanitize_language(&Some("deu".to_string())).unwrap(), "deu");
        assert_eq!(
            sanitize_language(&Some("eng+deu".to_string())).unwrap(),
            "eng+deu"
        );
        assert_eq!(
            sanitize_language(&Some("chi_sim".to_string())).unwrap(),
            "chi_sim"
        );
    }

    #[test]
    fn test_sanitize_language_invalid() {
        assert!(sanitize_language(&Some("eng deu".to_string())).is_err());
        assert!(sanitize_language(&Some("eng;rm".to_string())).is_err());
        assert!(sanitize_language(&Some("../hack".to_string())).is_err());
    }

    #[test]
    fn test_parse_page_range_none() {
        let (f, l) = parse_page_range(None);
        assert!(f.is_none());
        assert!(l.is_none());
    }

    #[test]
    fn test_parse_page_range_single() {
        let (f, l) = parse_page_range(Some("5"));
        assert_eq!(f, Some(5));
        assert_eq!(l, Some(5));
    }

    #[test]
    fn test_parse_page_range_range() {
        let (f, l) = parse_page_range(Some("1-5"));
        assert_eq!(f, Some(1));
        assert_eq!(l, Some(5));
    }

    #[test]
    fn test_parse_page_range_open_end() {
        let (f, l) = parse_page_range(Some("3-"));
        assert_eq!(f, Some(3));
        assert_eq!(l, None);
    }

    #[test]
    fn test_validate_file_path_nonexistent() {
        let result = validate_file_path(Path::new("/nonexistent/path/to/file.png"));
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Error:"));
    }

    #[test]
    fn test_validate_image_path_bad_ext() {
        // Create a temp file with wrong extension.
        let tmp = std::env::temp_dir().join("pb-mcp-test-bad-ext.xyz");
        std::fs::write(&tmp, b"fake").unwrap();
        let result = validate_image_path(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported image extension"));
    }

    #[test]
    fn test_reconstruct_line_empty() {
        assert_eq!(reconstruct_line(&[]), "");
    }

    #[test]
    fn test_reconstruct_line_words() {
        let words = vec![(10, "Hello".to_string()), (60, "World".to_string())];
        let line = reconstruct_line(&words);
        assert!(line.contains("Hello"));
        assert!(line.contains("World"));
    }

    #[test]
    fn test_reconstruct_line_column_gap() {
        // Gap of 100 pixels should produce a tab separator.
        let words = vec![(10, "Col1".to_string()), (200, "Col2".to_string())];
        let line = reconstruct_line(&words);
        assert!(line.contains('\t'));
    }

    #[test]
    fn test_parse_tesseract_tsv_empty() {
        let result = parse_tesseract_tsv_to_table("level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_tesseract_tsv_with_data() {
        let tsv = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n5\t1\t1\t1\t1\t1\t10\t20\t30\t10\t95\tHello\n5\t1\t1\t1\t1\t2\t50\t20\t30\t10\t90\tWorld\n";
        let result = parse_tesseract_tsv_to_table(tsv);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_parse_tesseract_tsv_with_confidence() {
        let tsv = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n5\t1\t1\t1\t1\t1\t10\t20\t30\t10\t95\tHello\n5\t1\t1\t1\t1\t2\t50\t20\t30\t10\t40\tWorld\n";
        let result = parse_tesseract_tsv_with_confidence(tsv);
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
        assert!(result.contains("95"));
        assert!(result.contains("LOW(40)"));
    }

    #[test]
    fn test_parse_box_file_empty() {
        let result = parse_box_file("");
        assert_eq!(result, "No text regions detected");
    }

    #[test]
    fn test_parse_box_file_data() {
        let box_data = "H 10 100 20 110 0\ne 22 100 30 110 0\nl 32 100 38 110 0\nl 40 100 46 110 0\no 48 100 58 110 0\n  60 100 68 110 0\nW 70 100 82 110 0\no 84 100 94 110 0\nr 96 100 104 110 0\nl 106 100 112 110 0\nd 114 100 124 110 0\n";
        let result = parse_box_file(box_data);
        assert!(result.contains("text="));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_ocr_file_missing_tesseract() {
        // If tesseract is not on PATH, should return an error.
        // This test is environment-dependent, so just check it returns a string.
        let _ = ocr_file_work(PathBuf::from("/nonexistent.png"), None);
    }

    #[test]
    fn test_ocr_list_languages_runs() {
        // Just check it doesn't panic.
        let _ = ocr_list_languages_work();
    }

    #[test]
    fn test_ocr_preprocess_invalid_operation() {
        let tmp = std::env::temp_dir().join("pb-mcp-test-preprocess.png");
        std::fs::write(&tmp, b"fake").unwrap();
        let result = ocr_preprocess_work(tmp.clone(), "invalid_op".to_string(), None);
        let _ = std::fs::remove_file(&tmp);
        assert!(result.contains("Error:"));
        assert!(result.contains("unknown operation"));
    }

    #[test]
    fn test_ocr_batch_empty() {
        let result = ocr_batch_work(vec![], None);
        assert!(result.contains("Error:"));
    }
}
