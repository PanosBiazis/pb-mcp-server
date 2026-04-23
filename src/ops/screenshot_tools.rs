//! # Screenshot Capture and Analysis Tools
//!
//! Comprehensive screenshot tools for AI agents to capture, annotate, compare,
//! and extract information from the screen.
//!
//! All screenshot operations rely on Linux CLI tools: **scrot**, **gnome-screenshot**,
//! **import** (ImageMagick), **xdotool**, **xrandr**, **ffmpeg**, and **tesseract**.
//! Functions return human-readable strings; errors are prefixed with `"Error:"`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default filename prefix for screenshots.
const DEFAULT_PREFIX: &str = "screenshot";

/// Default filename prefix for screen recordings.
const RECORDING_PREFIX: &str = "recording";

/// Default filename prefix for timelapse captures.
const TIMELAPSE_PREFIX: &str = "timelapse";

/// Default frames per second for screen recording.
const DEFAULT_FPS: u32 = 15;

/// Default comparison threshold (0.0 = identical, 1.0 = completely different).
const DEFAULT_COMPARE_THRESHOLD: f64 = 0.01;

/// Default OCR language.
const DEFAULT_OCR_LANG: &str = "eng";

/// Maximum recording duration in seconds (safety limit).
const MAX_RECORD_DURATION_SECS: u64 = 600;

/// Maximum timelapse count.
const MAX_TIMELAPSE_COUNT: u32 = 360;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Try to find an available screenshot capture tool.
/// Returns the tool name or an error string.
fn find_screenshot_tool() -> Result<&'static str, String> {
    for tool in &["scrot", "gnome-screenshot", "import"] {
        if cli_exists(tool) {
            return Ok(tool);
        }
    }
    Err(
        "Error: no screenshot tool found on PATH (need scrot, gnome-screenshot, or import)"
            .to_string(),
    )
}

/// Generate a timestamped filename for screenshots.
fn timestamped_filename(prefix: &str, ext: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}_{:.0}.{}", prefix, now.as_secs(), ext)
}

/// Sanitize a user-provided output path, ensuring parent directory exists
/// and the path ends with a valid image extension.
/// Returns the resolved `PathBuf` or an error string.
fn resolve_output_path(
    output: &Option<PathBuf>,
    prefix: &str,
    ext: &str,
) -> Result<PathBuf, String> {
    let path = match output {
        Some(p) => {
            let p = if p.is_relative() {
                std::env::current_dir().unwrap_or_default().join(p)
            } else {
                p.clone()
            };
            // Ensure the extension matches
            match p.extension().and_then(|e| e.to_str()) {
                Some(e) if e.eq_ignore_ascii_case(ext) => p,
                _ => p.with_extension(ext),
            }
        }
        None => std::env::current_dir()
            .unwrap_or_default()
            .join(timestamped_filename(prefix, ext)),
    };

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Error: cannot create output directory {}: {e}",
                    parent.display()
                )
            })?;
        }
    }

    Ok(path)
}

/// Run a command and return its stdout, or an error string on failure.
fn run_command(cmd: &str, args: &[String]) -> Result<String, String> {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match Command::new(cmd).args(&arg_refs).output() {
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

/// Build the DISPLAY environment variable string.
/// If `display` is `Some`, use it; otherwise default to `:0`.
fn display_env(display: &Option<String>) -> String {
    display.as_deref().unwrap_or(":0").to_string()
}

/// Capture a full screenshot using the first available tool.
/// Returns the output path on success, or an error string.
fn capture_full_screenshot(output_path: &Path, display: &Option<String>) -> Result<String, String> {
    let disp = display_env(display);

    // Try scrot first
    if cli_exists("scrot") {
        match Command::new("scrot")
            .env("DISPLAY", &disp)
            .arg("-z")
            .arg(output_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && output_path.is_file() {
                    return Ok("scrot".to_string());
                }
            }
            Err(_) => {}
        }
    }

    // Try gnome-screenshot
    if cli_exists("gnome-screenshot") {
        match Command::new("gnome-screenshot")
            .env("DISPLAY", &disp)
            .args(["-f"])
            .arg(output_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && output_path.is_file() {
                    return Ok("gnome-screenshot".to_string());
                }
            }
            Err(_) => {}
        }
    }

    // Try import (ImageMagick)
    if cli_exists("import") {
        match Command::new("import")
            .env("DISPLAY", &disp)
            .args(["-window", "root"])
            .arg(output_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && output_path.is_file() {
                    return Ok("import".to_string());
                }
            }
            Err(_) => {}
        }
    }

    Err(
        "Error: no screenshot tool could capture the screen (tried scrot, gnome-screenshot, import)"
            .to_string(),
    )
}

/// Get basic image info (dimensions, file size) for a captured screenshot.
fn image_info_string(path: &Path) -> String {
    let mut info = format!("path={}", path.display());

    if let Ok(meta) = fs::metadata(path) {
        info.push_str(&format!(" size={}bytes", meta.len()));
    }

    // Try to get dimensions via identify (ImageMagick)
    if find_imagemagick().is_ok() {
        let ident_cmd = if cli_exists("identify") {
            "identify"
        } else {
            "magick"
        };

        let args: Vec<String> = if ident_cmd == "identify" {
            vec![
                "-format".to_string(),
                "%wx%h".to_string(),
                path.to_string_lossy().to_string(),
            ]
        } else {
            vec![
                "identify".to_string(),
                "-format".to_string(),
                "%wx%h".to_string(),
                path.to_string_lossy().to_string(),
            ]
        };

        if let Ok(dims) = run_command(ident_cmd, &args) {
            info.push_str(&format!(" dimensions={dims}"));
        }
    }

    info
}

// ---------------------------------------------------------------------------
// 1. screenshot_capture_work
// ---------------------------------------------------------------------------

/// Capture a full screenshot of the entire display.
///
/// Uses `scrot`, `gnome-screenshot`, or `import` (ImageMagick) on Linux,
/// in that order of preference. Returns the path to the captured file and
/// basic image info, or an `"Error:"` prefixed string on failure.
///
/// # Arguments
/// * `output` — Optional output file path (defaults to a timestamped PNG in the current directory).
/// * `display` — Optional X11 display string (defaults to `:0`).
pub fn screenshot_capture_work(output: Option<PathBuf>, display: Option<String>) -> String {
    if let Err(e) = find_screenshot_tool() {
        return e;
    }

    let out_path = match resolve_output_path(&output, DEFAULT_PREFIX, "png") {
        Ok(p) => p,
        Err(e) => return e,
    };

    match capture_full_screenshot(&out_path, &display) {
        Ok(backend) => {
            let info = image_info_string(&out_path);
            let result = format!("SCREENSHOT_CAPTURE backend={backend}\n{info}\n");
            let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
            if clipped {
                return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
            }
            r
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 2. screenshot_region_work
// ---------------------------------------------------------------------------

/// Capture a specific screen region.
///
/// Uses `scrot --select` with geometry or `import -crop` from ImageMagick.
/// Returns the path to the captured file and basic info, or an `"Error:"`
/// prefixed string on failure.
///
/// # Arguments
/// * `x`, `y` — Top-left corner of the region.
/// * `width`, `height` — Size of the region in pixels.
/// * `output` — Optional output file path.
/// * `display` — Optional X11 display string (defaults to `:0`).
pub fn screenshot_region_work(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    output: Option<PathBuf>,
    display: Option<String>,
) -> String {
    if width == 0 || height == 0 {
        return "Error: width and height must be > 0".to_string();
    }

    let out_path = match resolve_output_path(&output, DEFAULT_PREFIX, "png") {
        Ok(p) => p,
        Err(e) => return e,
    };

    let disp = display_env(&display);
    let geometry = format!("{width}x{height}+{x}+{y}");
    let mut backend = "none";

    // Try scrot with -a (area) flag
    if cli_exists("scrot") {
        let area = format!("{x},{y},{width},{height}");
        match Command::new("scrot")
            .env("DISPLAY", &disp)
            .args(["-a", &area, "-z"])
            .arg(&out_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && out_path.is_file() {
                    backend = "scrot";
                }
            }
            Err(_) => {}
        }
    }

    // Try import (ImageMagick) with -crop
    if backend == "none" && cli_exists("import") {
        match Command::new("import")
            .env("DISPLAY", &disp)
            .args(["-window", "root", "-crop", &geometry])
            .arg(&out_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && out_path.is_file() {
                    backend = "import";
                }
            }
            Err(_) => {}
        }
    }

    if backend == "none" {
        // Clean up any empty file that might have been created
        let _ = fs::remove_file(&out_path);
        return "Error: screen region capture failed (tried scrot and import)".to_string();
    }

    let info = image_info_string(&out_path);
    let result = format!(
        "SCREENSHOT_REGION backend={backend} x={x} y={y} width={width} height={height}\n{info}\n"
    );
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// 3. screenshot_window_work
// ---------------------------------------------------------------------------

/// Capture a specific window by name.
///
/// Uses `xdotool` to find the window ID, then `import` or `scrot` to capture it.
/// Returns the path to the captured file, or an `"Error:"` prefixed string on
/// failure.
///
/// # Arguments
/// * `window_name` — Optional substring to match against window titles. If `None`, captures the active window.
/// * `output` — Optional output file path.
pub fn screenshot_window_work(window_name: Option<String>, output: Option<PathBuf>) -> String {
    if let Err(e) = require_cli("xdotool") {
        return format!("Error: xdotool required for window capture: {e}");
    }

    let out_path = match resolve_output_path(&output, DEFAULT_PREFIX, "png") {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Find the window ID
    let window_id = match &window_name {
        Some(name) => {
            // Search for window by name
            match run_command(
                "xdotool",
                &["search".to_string(), "--name".to_string(), name.clone()],
            ) {
                Ok(output) => {
                    let first_line = output.lines().next().unwrap_or("").trim();
                    if first_line.is_empty() {
                        return format!("Error: no window found matching name: {name}");
                    }
                    // xdotool may return multiple IDs; use the first one
                    let id = first_line.split_whitespace().next().unwrap_or("");
                    if id.is_empty() {
                        return format!("Error: no window ID found matching name: {name}");
                    }
                    id.to_string()
                }
                Err(e) => return format!("Error: xdotool search failed: {e}"),
            }
        }
        None => {
            // Get the currently active window
            match run_command("xdotool", &["getactivewindow".to_string()]) {
                Ok(output) => {
                    let id = output.trim();
                    if id.is_empty() {
                        return "Error: no active window found".to_string();
                    }
                    id.to_string()
                }
                Err(e) => return format!("Error: xdotool getactivewindow failed: {e}"),
            }
        }
    };

    let mut backend = "none";

    // Try import (ImageMagick) with -window <wid>
    if cli_exists("import") {
        match Command::new("import")
            .args(["-window", &window_id])
            .arg(&out_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && out_path.is_file() {
                    backend = "import";
                }
            }
            Err(_) => {}
        }
    }

    // Try scrot with -u (focused window) and window ID via xdotool
    if backend == "none" && cli_exists("scrot") {
        // scrot doesn't directly support window IDs, so we activate the window first
        let _ = run_command(
            "xdotool",
            &["windowactivate".to_string(), window_id.clone()],
        );
        std::thread::sleep(std::time::Duration::from_millis(200));

        match Command::new("scrot")
            .args(["-u", "-z"])
            .arg(&out_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && out_path.is_file() {
                    backend = "scrot";
                }
            }
            Err(_) => {}
        }
    }

    if backend == "none" {
        let _ = fs::remove_file(&out_path);
        return "Error: window capture failed (tried import and scrot)".to_string();
    }

    let info = image_info_string(&out_path);
    let win_desc = window_name.as_deref().unwrap_or("<active>");
    let result = format!(
        "SCREENSHOT_WINDOW backend={backend} window_id={window_id} window_name={win_desc}\n{info}\n"
    );
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// 4. screenshot_ocr_work
// ---------------------------------------------------------------------------

/// Capture a screenshot and immediately OCR it.
///
/// Combines a full screenshot capture with Tesseract OCR in one step.
/// Returns the extracted text, or an `"Error:"` prefixed string on failure.
///
/// # Arguments
/// * `output` — Optional path to save the intermediate screenshot image.
/// * `language` — Optional Tesseract language (defaults to `"eng"`).
pub fn screenshot_ocr_work(output: Option<PathBuf>, language: Option<String>) -> String {
    if let Err(e) = require_cli("tesseract") {
        return e;
    }
    if let Err(e) = find_screenshot_tool() {
        return e;
    }

    // Sanitize language
    let lang = match language {
        Some(l) if !l.trim().is_empty() => {
            let l = l.trim().to_string();
            if !l
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
            {
                return "Error: lang must be ASCII alnum with optional _ + -".to_string();
            }
            l
        }
        _ => DEFAULT_OCR_LANG.to_string(),
    };

    // Capture screenshot to a temp path
    let tmp_path = std::env::temp_dir().join(format!(
        "pb-mcp-screenshot-ocr-{:.0}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));

    let capture_result = capture_full_screenshot(&tmp_path, &None);
    if !tmp_path.is_file() {
        return format!("Error: screenshot capture failed for OCR: {capture_result:?}");
    }

    // Optionally copy the screenshot to the user-specified output path
    let saved_path_info = if let Some(ref out) = output {
        match fs::copy(&tmp_path, out) {
            Ok(_) => format!("screenshot_saved={}", out.display()),
            Err(e) => format!("screenshot_save_error={e}"),
        }
    } else {
        "screenshot_not_saved".to_string()
    };

    // Run Tesseract
    let tesseract_result = match Command::new("tesseract")
        .args([&tmp_path.to_string_lossy(), "stdout", "-l", &lang])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                Err(format!(
                    "Error: tesseract exit={:?}\nSTDERR:\n{se}",
                    o.status.code()
                ))
            } else {
                let mut text = stdout.trim().to_string();
                if !stderr.trim().is_empty() {
                    let (w, _) = super::truncate_chars(stderr.trim(), 1200);
                    text.push_str(&format!("\n\n--- tesseract stderr (info) ---\n{w}\n"));
                }
                Ok(text)
            }
        }
        Err(e) => Err(format!("Error: spawn tesseract: {e}")),
    };

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

    match tesseract_result {
        Ok(text) => {
            let (body, clipped) = super::truncate_chars(&text, super::MAX_RESPONSE_CHARS);
            let mut result = format!("SCREENSHOT_OCR lang={lang} {saved_path_info}\n\n{body}\n");
            if clipped {
                result.push_str(&format!(
                    "\n[truncated to {} chars]\n",
                    super::MAX_RESPONSE_CHARS
                ));
            }
            result
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 5. screenshot_record_work
// ---------------------------------------------------------------------------

/// Record a screen video (screencast).
///
/// Uses `ffmpeg` with the `x11grab` input to record the display.
/// Returns the path to the recorded file, or an `"Error:"` prefixed string
/// on failure.
///
/// # Arguments
/// * `duration_secs` — Recording duration in seconds (capped at 600s).
/// * `output` — Optional output file path (defaults to a timestamped `.mp4`).
/// * `fps` — Optional frames per second (defaults to 15).
pub fn screenshot_record_work(
    duration_secs: u64,
    output: Option<PathBuf>,
    fps: Option<u32>,
) -> String {
    if let Err(e) = require_cli("ffmpeg") {
        return e;
    }

    let duration = duration_secs.min(MAX_RECORD_DURATION_SECS);
    if duration == 0 {
        return "Error: duration_secs must be > 0".to_string();
    }

    let fps = fps.unwrap_or(DEFAULT_FPS).max(1).min(120);
    let out_path = match resolve_output_path(&output, RECORDING_PREFIX, "mp4") {
        Ok(p) => p,
        Err(e) => return e,
    };

    let disp = ":0";

    match Command::new("ffmpeg")
        .args([
            "-f",
            "x11grab",
            "-framerate",
            &fps.to_string(),
            "-i",
            &format!("{disp}+0,0"),
            "-t",
            &duration.to_string(),
            "-y", // overwrite output
            "-video_size",
            "1920x1080",
        ])
        .arg(&out_path)
        .output()
    {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                // Clean up potentially incomplete file
                let _ = fs::remove_file(&out_path);
                return format!("Error: ffmpeg exit={:?}\n{se}", o.status.code());
            }
        }
        Err(e) => {
            let _ = fs::remove_file(&out_path);
            return format!("Error: spawn ffmpeg: {e}");
        }
    }

    if !out_path.is_file() {
        return "Error: ffmpeg produced no output file".to_string();
    }

    let file_size = fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);

    let result = format!(
        "SCREENSHOT_RECORD duration={duration}s fps={fps}\npath={}\nsize={file_size}bytes\n",
        out_path.display()
    );
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// 6. screenshot_annotate_work
// ---------------------------------------------------------------------------

/// Add annotations (rectangles with text labels) to a screenshot.
///
/// Each annotation is a tuple of `(text, x, y, width, height)`, which draws
/// a rectangle at the specified position with an optional text label.
/// Uses ImageMagick `convert -draw` (or `magick`).
///
/// # Arguments
/// * `path` — Path to the source screenshot image.
/// * `annotations` — Vector of `(text, x, y, width, height)` annotation tuples.
/// * `output` — Optional output file path (defaults to source path with `_annotated` suffix).
pub fn screenshot_annotate_work(
    path: PathBuf,
    annotations: Vec<(String, u32, u32, u32, u32)>,
    output: Option<PathBuf>,
) -> String {
    if annotations.is_empty() {
        return "Error: annotations list is empty".to_string();
    }

    if !path.is_file() {
        return format!("Error: source image not found: {}", path.display());
    }

    let im = match find_imagemagick() {
        Ok(cmd) => cmd,
        Err(e) => return e,
    };

    // Determine output path
    let out_path = match &output {
        Some(p) => p.clone(),
        None => {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("screenshot");
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
            path.with_file_name(format!("{stem}_annotated.{ext}"))
        }
    };

    // Build the -draw arguments for each annotation
    let mut draw_args: Vec<String> = Vec::new();
    for (i, (text, x, y, w, h)) in annotations.iter().enumerate() {
        if *w == 0 || *h == 0 {
            return format!("Error: annotation {i} has zero width or height");
        }
        // Draw a red rectangle
        draw_args.push(format!(
            "fill none stroke red stroke-width 2 rectangle {x},{y} {},{}",
            x + w,
            y + h
        ));
        // Add text label if non-empty
        if !text.is_empty() {
            // Escape any special characters in text for ImageMagick
            let safe_text = text.replace('\\', "\\\\").replace('"', "\\\"");
            draw_args.push(format!(
                "fill red font-size 16 text {},{} '\"{safe_text}\"'",
                x + 4,
                y + 16
            ));
        }
    }

    // Construct the command
    // We need to build: convert <input> [ -draw "..." ... ] <output>
    let mut cmd = Command::new(im);
    cmd.arg(&path);

    for draw in &draw_args {
        cmd.args(["-draw", draw]);
    }

    cmd.arg(&out_path);

    match cmd.output() {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return format!(
                    "Error: ImageMagick annotate exit={:?}\n{se}",
                    o.status.code()
                );
            }
        }
        Err(e) => return format!("Error: spawn {im}: {e}"),
    }

    if !out_path.is_file() {
        return "Error: annotation produced no output file".to_string();
    }

    let info = image_info_string(&out_path);
    let result = format!(
        "SCREENSHOT_ANNOTATE annotations={}\nsource={}\n{info}\n",
        annotations.len(),
        path.display()
    );
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// 7. screenshot_compare_work
// ---------------------------------------------------------------------------

/// Compare two screenshots and report visual differences.
///
/// Uses ImageMagick `compare` to compute the mean absolute error (MAE) between
/// two images. Returns whether the images differ beyond a threshold.
///
/// # Arguments
/// * `path_a` — Path to the first screenshot.
/// * `path_b` — Path to the second screenshot.
/// * `threshold` — Optional MAE threshold (0.0–1.0, defaults to 0.01). Values below this are considered identical.
pub fn screenshot_compare_work(path_a: PathBuf, path_b: PathBuf, threshold: Option<f64>) -> String {
    if !path_a.is_file() {
        return format!("Error: first image not found: {}", path_a.display());
    }
    if !path_b.is_file() {
        return format!("Error: second image not found: {}", path_b.display());
    }

    let threshold = threshold
        .unwrap_or(DEFAULT_COMPARE_THRESHOLD)
        .clamp(0.0, 1.0);
    let im = match find_imagemagick() {
        Ok(cmd) => cmd,
        Err(e) => return e,
    };

    // Use ImageMagick `compare` to get the MAE metric.
    // `compare -metric MAE image_a image_b /dev/null 2>&1`
    // On success, the metric value is printed to stderr.
    let null_path = if cfg!(unix) { "/dev/null" } else { "NUL" };

    let output = match Command::new(im)
        .args(["compare", "-metric", "MAE"])
        .arg(&path_a)
        .arg(&path_b)
        .arg(null_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => return format!("Error: spawn {im}: {e}"),
    };

    // ImageMagick compare prints metric to stderr, exit code 0 = similar, 1 = different
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse the MAE value from stderr (format: "MAE: 0.001234 (1.234e-03)")
    let mae_value = parse_mae_from_output(&stderr)
        .or_else(|| parse_mae_from_output(&stdout))
        .unwrap_or(-1.0);

    let verdict = if mae_value < 0.0 {
        "unknown"
    } else if mae_value <= threshold {
        "match"
    } else {
        "different"
    };

    // Also try to get a diff image for visual inspection
    let diff_path = path_a.with_extension("diff.png");
    let diff_result = Command::new(im)
        .args(["compare", "-metric", "AE"])
        .arg(&path_a)
        .arg(&path_b)
        .arg(&diff_path)
        .output();

    let diff_info = match diff_result {
        Ok(o) if diff_path.is_file() => {
            let ae_stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let ae_value = ae_stderr
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(-1.0);
            format!("diff_image={} ae={ae_value:.0}", diff_path.display())
        }
        _ => "diff_image=unavailable".to_string(),
    };

    // Clean up diff image
    let _ = fs::remove_file(&diff_path);

    let result = format!(
        "SCREENSHOT_COMPARE\nimage_a={}\nimage_b={}\nmae={mae_value:.6}\nthreshold={threshold}\nverdict={verdict}\n{diff_info}\n",
        path_a.display(),
        path_b.display()
    );
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

/// Parse the MAE (Mean Absolute Error) value from ImageMagick compare output.
/// The output format is typically: "MAE: 0.00123 (1.23e-03)" or just a number.
fn parse_mae_from_output(s: &str) -> Option<f64> {
    // Try "MAE: 0.123 (1.23e-01)" format
    if let Some(rest) = s.strip_prefix("MAE:") {
        let first_token = rest.trim().split_whitespace().next()?;
        return first_token.trim_end_matches(',').parse::<f64>().ok();
    }
    // Try just a plain number
    let first_token = s.split_whitespace().next()?;
    first_token.trim_end_matches(',').parse::<f64>().ok()
}

// ---------------------------------------------------------------------------
// 8. screenshot_monitor_info_work
// ---------------------------------------------------------------------------

/// List available monitors/displays with their resolutions and positions.
///
/// Uses `xrandr` on Linux to query connected display information.
/// Returns a human-readable summary, or an `"Error:"` prefixed string on failure.
pub fn screenshot_monitor_info_work() -> String {
    if let Err(e) = require_cli("xrandr") {
        return e;
    }

    let raw = match run_command("xrandr", &["--query".to_string()]) {
        Ok(o) => o,
        Err(e) => return e,
    };

    // Parse xrandr output for connected monitors
    let mut monitors: Vec<String> = Vec::new();
    for line in raw.lines() {
        // Connected lines look like: "DP-1 connected primary 1920x1080+0+0 ..."
        if line.contains(" connected") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"?");

            // Find the resolution+position field (format: WxH+X+Y)
            let mut res_pos = "?";
            let mut is_primary = false;
            for part in &parts {
                if *part == "primary" {
                    is_primary = true;
                }
                // Match patterns like 1920x1080+0+0
                if part.contains('x') && part.contains('+') {
                    res_pos = part;
                    break;
                }
            }

            // Parse resolution and position
            let (resolution, position) = parse_res_pos(res_pos);

            monitors.push(format!(
                "{}{} {} pos=({})",
                name,
                if is_primary { " [primary]" } else { "" },
                resolution,
                position
            ));
        }
    }

    if monitors.is_empty() {
        return "SCREENSHOT_MONITOR_INFO\nNo connected monitors found\n".to_string();
    }

    let mut result = format!("SCREENSHOT_MONITOR_INFO count={}\n", monitors.len());
    for m in &monitors {
        result.push_str(m);
        result.push('\n');
    }
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

/// Parse an xrandr resolution+position string like "1920x1080+0+0".
/// Returns (resolution, position) e.g. ("1920x1080", "0,0").
fn parse_res_pos(s: &str) -> (&str, String) {
    // Split on the first '+' after the 'x' pattern
    if let Some(plus_pos) = s.find('+') {
        let res = &s[..plus_pos];
        let pos_part = &s[plus_pos + 1..];
        let pos = pos_part.replace('+', ",");
        (res, pos)
    } else {
        (s, "unknown".to_string())
    }
}

// ---------------------------------------------------------------------------
// 9. screenshot_color_pick_work
// ---------------------------------------------------------------------------

/// Pick a color at specific screen coordinates.
///
/// Takes a tiny 1x1 screenshot at the given coordinates and reads the pixel
/// color value. Returns the color in hex and RGB format.
///
/// # Arguments
/// * `x`, `y` — Screen coordinates to sample.
/// * `display` — Optional X11 display string (defaults to `:0`).
pub fn screenshot_color_pick_work(x: u32, y: u32, display: Option<String>) -> String {
    let im = match find_imagemagick() {
        Ok(cmd) => cmd,
        Err(e) => return e,
    };

    let disp = display_env(&display);

    // Capture a 1x1 region at (x, y) using import
    let tmp_path = std::env::temp_dir().join(format!(
        "pb-mcp-colorpick-{:.0}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));

    // Try import (ImageMagick) first — most reliable for 1x1 crop
    let captured = if cli_exists("import") {
        let geometry = format!("1x1+{x}+{y}");
        match Command::new("import")
            .env("DISPLAY", &disp)
            .args(["-window", "root", "-crop", &geometry])
            .arg(&tmp_path)
            .output()
        {
            Ok(o) => o.status.success() && tmp_path.is_file(),
            Err(_) => false,
        }
    } else {
        false
    };

    // Fallback: scrot + crop
    let captured = if !captured && cli_exists("scrot") {
        let full_path = std::env::temp_dir().join(format!(
            "pb-mcp-colorpick-full-{:.0}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ));

        match Command::new("scrot")
            .env("DISPLAY", &disp)
            .arg("-z")
            .arg(&full_path)
            .output()
        {
            Ok(o) => {
                if o.status.success() && full_path.is_file() {
                    // Crop using ImageMagick
                    let geometry = format!("1x1+{x}+{y}");
                    let crop_ok = match Command::new(im)
                        .args([&full_path.to_string_lossy(), "-crop", &geometry, "+repage"])
                        .arg(&tmp_path)
                        .output()
                    {
                        Ok(c) => c.status.success() && tmp_path.is_file(),
                        Err(_) => false,
                    };
                    let _ = fs::remove_file(&full_path);
                    crop_ok
                } else {
                    let _ = fs::remove_file(&full_path);
                    false
                }
            }
            Err(_) => {
                let _ = fs::remove_file(&full_path);
                false
            }
        }
    } else {
        captured
    };

    if !captured || !tmp_path.is_file() {
        let _ = fs::remove_file(&tmp_path);
        return "Error: could not capture pixel at specified coordinates".to_string();
    }

    // Read the pixel color using ImageMagick identify
    let ident_cmd = if cli_exists("identify") {
        "identify"
    } else {
        "magick"
    };

    let pixel_format = "%[pixel:p{0,0}]";
    let hex_format = "#%02x%02x%02x";

    let args: Vec<String> = if ident_cmd == "identify" {
        vec![
            "-format".to_string(),
            pixel_format.to_string(),
            tmp_path.to_string_lossy().to_string(),
        ]
    } else {
        vec![
            "identify".to_string(),
            "-format".to_string(),
            pixel_format.to_string(),
            tmp_path.to_string_lossy().to_string(),
        ]
    };

    let color_result = run_command(ident_cmd, &args);

    let hex_args: Vec<String> = if ident_cmd == "identify" {
        vec![
            "-format".to_string(),
            hex_format.to_string(),
            tmp_path.to_string_lossy().to_string(),
        ]
    } else {
        vec![
            "identify".to_string(),
            "-format".to_string(),
            hex_format.to_string(),
            tmp_path.to_string_lossy().to_string(),
        ]
    };

    let hex_result = run_command(ident_cmd, &hex_args);

    // Clean up temp file
    let _ = fs::remove_file(&tmp_path);

    let color_str = color_result.unwrap_or_else(|e| format!("unknown ({e})"));
    let hex_str = hex_result.unwrap_or_else(|_| "unknown".to_string());

    let result = format!("SCREENSHOT_COLOR_PICK x={x} y={y}\ncolor={color_str}\nhex={hex_str}\n");
    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// 10. screenshot_timelapse_work
// ---------------------------------------------------------------------------

/// Capture screenshots at regular intervals for a timelapse.
///
/// Captures `count` screenshots, one every `interval_secs` seconds.
/// Returns a list of captured file paths, or an `"Error:"` prefixed string
/// on failure.
///
/// # Arguments
/// * `interval_secs` — Seconds between each capture.
/// * `count` — Number of screenshots to capture (capped at 360).
/// * `output_dir` — Optional directory for output files (defaults to current directory + `timelapse/` subdirectory).
pub fn screenshot_timelapse_work(
    interval_secs: u64,
    count: u32,
    output_dir: Option<PathBuf>,
) -> String {
    if let Err(e) = find_screenshot_tool() {
        return e;
    }

    if interval_secs == 0 {
        return "Error: interval_secs must be > 0".to_string();
    }

    let count = count.min(MAX_TIMELAPSE_COUNT);
    if count == 0 {
        return "Error: count must be > 0".to_string();
    }

    // Determine output directory
    let out_dir = match &output_dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()
            .unwrap_or_default()
            .join(TIMELAPSE_PREFIX),
    };

    if let Err(e) = fs::create_dir_all(&out_dir) {
        return format!("Error: cannot create timelapse directory: {e}");
    }

    let mut captured_paths: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for i in 0..count {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(interval_secs));
        }

        let filename = format!("{TIMELAPSE_PREFIX}_{:04}.png", i + 1);
        let file_path = out_dir.join(&filename);

        match capture_full_screenshot(&file_path, &None) {
            Ok(_) => {
                if file_path.is_file() {
                    captured_paths.push(file_path.display().to_string());
                } else {
                    errors.push(format!("frame {i}: file not created"));
                }
            }
            Err(e) => {
                errors.push(format!("frame {i}: {e}"));
            }
        }
    }

    let mut result = format!(
        "SCREENSHOT_TIMELAPSE interval={interval_secs}s count={count} captured={}\n",
        captured_paths.len()
    );

    if !errors.is_empty() {
        result.push_str("errors:\n");
        for e in &errors {
            result.push_str(&format!(" {e}\n"));
        }
    }

    result.push_str("files:\n");
    for p in &captured_paths {
        result.push_str(&format!(" {p}\n"));
    }

    let (r, clipped) = super::truncate_chars(&result, super::MAX_RESPONSE_CHARS);
    if clipped {
        return format!("{r}\n[truncated to {} chars]\n", super::MAX_RESPONSE_CHARS);
    }
    r
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamped_filename_format() {
        let name = timestamped_filename("screenshot", "png");
        assert!(name.starts_with("screenshot_"));
        assert!(name.ends_with(".png"));
    }

    #[test]
    fn test_resolve_output_path_default() {
        let path = resolve_output_path(&None, "test", "png").unwrap();
        assert!(path.to_string_lossy().ends_with(".png"));
        assert!(path.to_string_lossy().contains("test_"));
    }

    #[test]
    fn test_resolve_output_path_custom() {
        let custom = PathBuf::from("/tmp/myshot.png");
        let path = resolve_output_path(&Some(custom.clone()), "test", "png").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/myshot.png"));
    }

    #[test]
    fn test_resolve_output_path_adds_extension() {
        let custom = PathBuf::from("/tmp/myshot");
        let path = resolve_output_path(&Some(custom), "test", "png").unwrap();
        assert!(path.to_string_lossy().ends_with(".png"));
    }

    #[test]
    fn test_display_env_default() {
        assert_eq!(display_env(&None), ":0");
    }

    #[test]
    fn test_display_env_custom() {
        assert_eq!(display_env(&Some(":1".to_string())), ":1");
    }

    #[test]
    fn test_parse_res_pos_normal() {
        let (res, pos) = parse_res_pos("1920x1080+0+0");
        assert_eq!(res, "1920x1080");
        assert_eq!(pos, "0,0");
    }

    #[test]
    fn test_parse_res_pos_with_offset() {
        let (res, pos) = parse_res_pos("2560x1440+1920+0");
        assert_eq!(res, "2560x1440");
        assert_eq!(pos, "1920,0");
    }

    #[test]
    fn test_parse_res_pos_no_plus() {
        let (res, pos) = parse_res_pos("1920x1080");
        assert_eq!(res, "1920x1080");
        assert_eq!(pos, "unknown");
    }

    #[test]
    fn test_parse_mae_from_output_with_prefix() {
        let val = parse_mae_from_output("MAE: 0.005432 (5.432e-03)");
        assert!(val.is_some());
        assert!((val.unwrap() - 0.005432).abs() < 1e-6);
    }

    #[test]
    fn test_parse_mae_from_output_plain_number() {
        let val = parse_mae_from_output("0.001234");
        assert!(val.is_some());
        assert!((val.unwrap() - 0.001234).abs() < 1e-9);
    }

    #[test]
    fn test_parse_mae_from_output_empty() {
        assert!(parse_mae_from_output("").is_none());
    }

    #[test]
    fn test_screenshot_capture_no_tool() {
        // This test just checks the function returns an error-like string
        // when no tool is available (which may happen in CI)
        let result = screenshot_capture_work(None, None);
        // It should either succeed (if a tool exists) or start with "Error:"
        assert!(result.starts_with("SCREENSHOT_CAPTURE") || result.starts_with("Error:"));
    }

    #[test]
    fn test_screenshot_region_zero_dimensions() {
        let result = screenshot_region_work(0, 0, 0, 100, None, None);
        assert!(result.starts_with("Error:"));
        assert!(result.contains("width and height must be > 0"));

        let result = screenshot_region_work(0, 0, 100, 0, None, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_screenshot_annotate_empty() {
        let result = screenshot_annotate_work(PathBuf::from("/tmp/test.png"), vec![], None);
        assert!(result.starts_with("Error:"));
        assert!(result.contains("annotations list is empty"));
    }

    #[test]
    fn test_screenshot_annotate_missing_file() {
        let result = screenshot_annotate_work(
            PathBuf::from("/tmp/nonexistent_test_file_12345.png"),
            vec![("test".to_string(), 10, 10, 100, 50)],
            None,
        );
        assert!(result.starts_with("Error:"));
        assert!(result.contains("source image not found"));
    }

    #[test]
    fn test_screenshot_compare_missing_file() {
        let result = screenshot_compare_work(
            PathBuf::from("/tmp/nonexistent_a_12345.png"),
            PathBuf::from("/tmp/nonexistent_b_12345.png"),
            None,
        );
        assert!(result.starts_with("Error:"));
        assert!(result.contains("first image not found"));
    }

    #[test]
    fn test_screenshot_compare_one_missing() {
        // Create a temp file for path_a
        let tmp = std::env::temp_dir().join("pb-mcp-test-compare-a.png");
        let _ = fs::write(&tmp, b"fake");
        let result = screenshot_compare_work(
            tmp.clone(),
            PathBuf::from("/tmp/nonexistent_b_12345.png"),
            None,
        );
        assert!(result.starts_with("Error:"));
        assert!(result.contains("second image not found"));
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_screenshot_record_zero_duration() {
        let result = screenshot_record_work(0, None, None);
        assert!(result.starts_with("Error:"));
        // If ffmpeg is not on PATH, we get a different error first;
        // otherwise we should see the duration validation message.
        assert!(
            result.contains("duration_secs must be > 0") || result.contains("ffmpeg"),
            "expected duration or ffmpeg error, got: {result}"
        );
    }

    #[test]
    fn test_screenshot_timelapse_zero_interval() {
        let result = screenshot_timelapse_work(0, 5, None);
        assert!(result.starts_with("Error:"));
        assert!(result.contains("interval_secs must be > 0"));
    }

    #[test]
    fn test_screenshot_timelapse_zero_count() {
        let result = screenshot_timelapse_work(1, 0, None);
        assert!(result.starts_with("Error:"));
        assert!(result.contains("count must be > 0"));
    }

    #[test]
    fn test_screenshot_monitor_info_runs() {
        // Just verify it doesn't panic — may return error in CI without xrandr
        let result = screenshot_monitor_info_work();
        assert!(result.starts_with("SCREENSHOT_MONITOR_INFO") || result.starts_with("Error:"));
    }
}
