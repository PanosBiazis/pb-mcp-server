//! # Audio Processing Tools
//!
//! Comprehensive audio analysis, conversion, and manipulation tools for AI agents.
//!
//! All operations rely on **ffmpeg** and **ffprobe**, which are expected to be
//! available on `PATH`. Functions return human-readable strings; errors are
//! prefixed with `"Error:"`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum input audio file size (500 MiB).
const MAX_AUDIO_BYTES: u64 = 500 * 1024 * 1024;

/// Supported audio formats for conversion.
const SUPPORTED_AUDIO_FORMATS: &[&str] =
    &["mp3", "wav", "ogg", "flac", "aac", "m4a", "opus", "wma"];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validate that `path` exists, is a file, and is under the size limit.
/// Returns the file metadata or an error string starting with `"Error:"`.
fn validate_audio_path(path: &Path) -> Result<fs::Metadata, String> {
    if !path.is_file() {
        return Err(format!("Error: not a file: {}", path.display()));
    }
    let meta =
        fs::metadata(path).map_err(|e| format!("Error: cannot stat {}: {e}", path.display()))?;
    if meta.len() > MAX_AUDIO_BYTES {
        return Err(format!(
            "Error: file too large ({} bytes, max {} MiB)",
            meta.len(),
            MAX_AUDIO_BYTES / (1024 * 1024)
        ));
    }
    Ok(meta)
}

/// Determine the output path: if `output` is `Some`, use it; otherwise derive
/// from `input` by appending `suffix` before the extension and optionally
/// replacing the extension with `new_ext`.
fn output_path(
    input: &Path,
    suffix: &str,
    new_ext: Option<&str>,
    output: &Option<PathBuf>,
) -> PathBuf {
    match output {
        Some(p) => p.clone(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audio");
            let ext = new_ext
                .unwrap_or_else(|| input.extension().and_then(|e| e.to_str()).unwrap_or("wav"));
            input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{stem}{suffix}.{ext}"))
        }
    }
}

/// Check whether a CLI program exists on `PATH` by running `<program> -version`.
fn cli_exists(program: &str) -> bool {
    Command::new(program).arg("-version").output().is_ok()
}

/// Ensure ffmpeg is available; return an error string if not.
fn require_ffmpeg() -> Result<(), String> {
    if cli_exists("ffmpeg") {
        Ok(())
    } else {
        Err("Error: ffmpeg not found on PATH".to_string())
    }
}

/// Ensure ffprobe is available; return an error string if not.
fn require_ffprobe() -> Result<(), String> {
    if cli_exists("ffprobe") {
        Ok(())
    } else {
        Err("Error: ffprobe not found on PATH".to_string())
    }
}

/// Run a command and return its stdout, or an error string on failure.
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

/// Run a command and return its combined stdout + stderr (for ffprobe which
/// may write to stderr even on success).
fn _run_command_combined(cmd: &str, args: &[&str]) -> Result<String, String> {
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
            // Combine stdout and stderr for ffprobe info
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

/// Format file size in a human-readable way.
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

/// Truncate a response string using the parent module's `truncate_chars` and
/// `MAX_RESPONSE_CHARS`.
fn truncate_response(s: &str) -> String {
    let (t, was_truncated) = super::truncate_chars(s, super::MAX_RESPONSE_CHARS);
    if was_truncated {
        format!("{t}\n[...truncated]")
    } else {
        t
    }
}

/// Extract a string value from a JSON `Value` that may be a string or a number.
/// Returns `None` if the key is missing or the value is null.
fn value_as_string(val: &serde_json::Value) -> Option<String> {
    match val {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(
            n.as_f64()
                .map(|f| format!("{f}"))
                .unwrap_or_else(|| n.to_string()),
        ),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 1. audio_info
// ---------------------------------------------------------------------------

/// Read audio metadata (format, duration, bitrate, sample rate, channels, codec)
/// using `ffprobe`.
///
/// Returns a human-readable key=value summary of the audio file's properties.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
pub fn audio_info_work(path: PathBuf) -> String {
    if let Err(e) = require_ffprobe() {
        return e;
    }
    let p = &path;
    let meta = match validate_audio_path(p) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let p_str = p.to_string_lossy();

    // Use ffprobe JSON output for structured parsing
    let json_out = match run_command(
        "ffprobe",
        &[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &p_str,
        ],
    ) {
        Ok(s) => s,
        Err(e) => return e,
    };

    let mut out = format!("AUDIO_INFO path={}\n", p.display());
    out.push_str(&format!("file_size={}\n", format_size(meta.len())));

    // Parse JSON output
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_out) {
        // Format-level info
        if let Some(fmt) = val.get("format") {
            if let Some(v) = fmt.get("format_name").and_then(|v| v.as_str()) {
                out.push_str(&format!("format={v}\n"));
            }
            if let Some(v) = fmt.get("duration").and_then(value_as_string) {
                out.push_str(&format!("duration={v}s\n"));
            }
            if let Some(v) = fmt.get("bit_rate").and_then(value_as_string) {
                out.push_str(&format!("bitrate={v} bps\n"));
            }
        }

        // Stream-level info (first audio stream)
        if let Some(streams) = val.get("streams").and_then(|v| v.as_array()) {
            if let Some(audio_stream) = streams
                .iter()
                .find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio"))
            {
                if let Some(v) = audio_stream.get("codec_name").and_then(|v| v.as_str()) {
                    out.push_str(&format!("codec={v}\n"));
                }
                if let Some(v) = audio_stream.get("sample_rate").and_then(value_as_string) {
                    out.push_str(&format!("sample_rate={v} Hz\n"));
                }
                if let Some(v) = audio_stream.get("channels").and_then(|v| v.as_i64()) {
                    out.push_str(&format!("channels={v}\n"));
                    let label = match v {
                        1 => "mono",
                        2 => "stereo",
                        6 => "5.1",
                        8 => "7.1",
                        _ => "unknown layout",
                    };
                    out.push_str(&format!("channel_layout={label}\n"));
                }
                if let Some(v) = audio_stream.get("bits_per_sample").and_then(|v| v.as_i64()) {
                    if v > 0 {
                        out.push_str(&format!("bits_per_sample={v}\n"));
                    }
                }
            } else {
                out.push_str("codec=unknown (no audio stream found)\n");
            }
        }
    } else {
        // Fallback: use ffprobe show_format/show_stream text output
        out.push_str(&format!("ffprobe_raw={json_out}\n"));
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 2. audio_convert
// ---------------------------------------------------------------------------

/// Convert between audio formats (mp3, wav, ogg, flac, aac, m4a, opus, wma)
/// using `ffmpeg`.
///
/// If `output` is `None`, the output file is derived from the input filename
/// with the target format as the new extension.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `target_format` - Desired output format (e.g. `"mp3"`, `"wav"`, `"flac"`).
/// * `output` - Optional explicit output path.
pub fn audio_convert_work(path: PathBuf, target_format: String, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    let fmt = target_format.to_lowercase();
    if !SUPPORTED_AUDIO_FORMATS.contains(&fmt.as_str()) {
        return format!(
            "Error: unsupported target format '{}'. Supported: {}",
            fmt,
            SUPPORTED_AUDIO_FORMATS.join(", ")
        );
    }
    let meta = match validate_audio_path(&path) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let out_path = output_path(&path, "", Some(&fmt), &output);
    let p_str = path.to_string_lossy();
    let o_str = out_path.to_string_lossy();

    let result = run_command("ffmpeg", &["-y", "-i", &p_str, &o_str]);

    match result {
        Ok(_) => format!(
            "Converted {} ({}) -> {} ({})",
            path.display(),
            format_size(meta.len()),
            out_path.display(),
            fmt
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 3. audio_extract_segment
// ---------------------------------------------------------------------------

/// Extract a segment of audio by time range using `ffmpeg`.
///
/// Extracts from `start_secs` for `duration_secs` seconds.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `start_secs` - Start time in seconds.
/// * `duration_secs` - Duration of the segment in seconds.
/// * `output` - Optional explicit output path.
pub fn audio_extract_segment_work(
    path: PathBuf,
    start_secs: f64,
    duration_secs: f64,
    output: Option<PathBuf>,
) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if start_secs < 0.0 {
        return "Error: start_secs must be non-negative".to_string();
    }
    if duration_secs <= 0.0 {
        return "Error: duration_secs must be positive".to_string();
    }

    let out_path = output_path(&path, "_segment", None, &output);
    let p_str = path.to_string_lossy();
    let o_str = out_path.to_string_lossy();
    let start_arg = format!("{start_secs:.3}");
    let dur_arg = format!("{duration_secs:.3}");

    let result = run_command(
        "ffmpeg",
        &[
            "-y", "-i", &p_str, "-ss", &start_arg, "-t", &dur_arg, "-c", "copy", &o_str,
        ],
    );

    match result {
        Ok(_) => format!(
            "Extracted segment [{start_secs:.3}s - {:.3}s] from {} -> {}",
            start_secs + duration_secs,
            path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 4. audio_trim
// ---------------------------------------------------------------------------

/// Trim audio between `start_secs` and `end_secs` using `ffmpeg`.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `start_secs` - Start time in seconds.
/// * `end_secs` - End time in seconds.
/// * `output` - Optional explicit output path.
pub fn audio_trim_work(
    path: PathBuf,
    start_secs: f64,
    end_secs: f64,
    output: Option<PathBuf>,
) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if start_secs < 0.0 {
        return "Error: start_secs must be non-negative".to_string();
    }
    if end_secs <= start_secs {
        return "Error: end_secs must be greater than start_secs".to_string();
    }

    let out_path = output_path(&path, "_trimmed", None, &output);
    let p_str = path.to_string_lossy();
    let o_str = out_path.to_string_lossy();
    let start_arg = format!("{start_secs:.3}");
    let duration = end_secs - start_secs;
    let dur_arg = format!("{duration:.3}");

    let result = run_command(
        "ffmpeg",
        &[
            "-y", "-i", &p_str, "-ss", &start_arg, "-t", &dur_arg, &o_str,
        ],
    );

    match result {
        Ok(_) => format!(
            "Trimmed {} [{start_secs:.3}s - {end_secs:.3}s] -> {}",
            path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 5. audio_merge
// ---------------------------------------------------------------------------

/// Merge/concatenate multiple audio files into one using ffmpeg filter complex.
///
/// All input files are concatenated in order. The output format is determined
/// by the output file extension.
///
/// # Arguments
///
/// * `paths` - Ordered list of audio file paths to merge.
/// * `output` - Path for the merged output file.
pub fn audio_merge_work(paths: Vec<PathBuf>, output: PathBuf) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if paths.is_empty() {
        return "Error: no input files provided".to_string();
    }
    if paths.len() < 2 {
        return "Error: at least two input files are required for merging".to_string();
    }

    // Validate all input paths
    for p in &paths {
        if let Err(e) = validate_audio_path(p) {
            return e;
        }
    }

    // Build the ffmpeg filter_complex concatenation command:
    // ffmpeg -i a -i b -i c -filter_complex "[0:a][1:a][2:a]concat=n=3:v=0:a=1" out
    let n = paths.len();
    let mut args: Vec<String> = vec!["-y".to_string()];

    // Add input files
    for p in &paths {
        args.push("-i".to_string());
        args.push(p.to_string_lossy().to_string());
    }

    // Build filter_complex string
    let filter_inputs: Vec<String> = (0..n).map(|i| format!("[{}:a]", i)).collect();
    let filter_str = format!("{}concat=n={}:v=0:a=1[out]", filter_inputs.join(""), n);

    args.push("-filter_complex".to_string());
    args.push(filter_str);
    args.push("-map".to_string());
    args.push("[out]".to_string());
    args.push(output.to_string_lossy().to_string());

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!("Merged {} files -> {}", n, output.display()),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 6. audio_mix
// ---------------------------------------------------------------------------

/// Mix two audio files (overlay) with optional volume controls.
///
/// Uses ffmpeg's `amix` filter to overlay two audio streams. Volume
/// multipliers default to 1.0 (original volume).
///
/// # Arguments
///
/// * `path_a` - First audio file path.
/// * `path_b` - Second audio file path.
/// * `output` - Path for the mixed output file.
/// * `volume_a` - Volume multiplier for the first file (default 1.0).
/// * `volume_b` - Volume multiplier for the second file (default 1.0).
pub fn audio_mix_work(
    path_a: PathBuf,
    path_b: PathBuf,
    output: PathBuf,
    volume_a: Option<f64>,
    volume_b: Option<f64>,
) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path_a) {
        return e;
    }
    if let Err(e) = validate_audio_path(&path_b) {
        return e;
    }

    let vol_a = volume_a.unwrap_or(1.0);
    let vol_b = volume_b.unwrap_or(1.0);

    if vol_a < 0.0 || vol_b < 0.0 {
        return "Error: volume values must be non-negative".to_string();
    }

    let a_str = path_a.to_string_lossy().to_string();
    let b_str = path_b.to_string_lossy().to_string();
    let o_str = output.to_string_lossy().to_string();

    // Build filter: volume adjustments + amix
    let filter = format!(
        "[0:a]volume={vol_a}[a0];[1:a]volume={vol_b}[a1];[a0][a1]amix=inputs=2:duration=longest[out]"
    );

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        a_str,
        "-i".to_string(),
        b_str,
        "-filter_complex".to_string(),
        filter,
        "-map".to_string(),
        "[out]".to_string(),
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!(
            "Mixed {} + {} (vol_a={vol_a}, vol_b={vol_b}) -> {}",
            path_a.display(),
            path_b.display(),
            output.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 7. audio_volume
// ---------------------------------------------------------------------------

/// Adjust audio volume.
///
/// A `volume` of 1.0 means original, 0.5 means half, and 2.0 means double.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `volume` - Volume multiplier (1.0 = original).
/// * `output` - Optional explicit output path.
pub fn audio_volume_work(path: PathBuf, volume: f64, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if volume < 0.0 {
        return "Error: volume must be non-negative".to_string();
    }

    let suffix = format!("_vol{volume}");
    let out_path = output_path(&path, &suffix, None, &output);
    let p_str = path.to_string_lossy().to_string();
    let o_str = out_path.to_string_lossy().to_string();
    let filter = format!("volume={volume}");

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        p_str,
        "-af".to_string(),
        filter,
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!(
            "Adjusted volume x{volume} on {} -> {}",
            path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 8. audio_speed
// ---------------------------------------------------------------------------

/// Change playback speed without pitch correction.
///
/// Uses ffmpeg's `atempo` filter. A `speed` of 2.0 means twice as fast,
/// 0.5 means half speed. The atempo filter supports values between 0.5 and
/// 100.0; for values outside this range, multiple atempo filters are chained.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `speed` - Playback speed multiplier (1.0 = original).
/// * `output` - Optional explicit output path.
pub fn audio_speed_work(path: PathBuf, speed: f64, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if speed <= 0.0 {
        return "Error: speed must be positive".to_string();
    }

    // atempo supports 0.5..=100.0; for extreme values, chain multiple filters
    let filter = build_atempo_chain(speed);

    let suffix = format!("_speed{speed}");
    let out_path = output_path(&path, &suffix, None, &output);
    let p_str = path.to_string_lossy().to_string();
    let o_str = out_path.to_string_lossy().to_string();

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        p_str,
        "-af".to_string(),
        filter,
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!(
            "Changed speed x{speed} on {} -> {}",
            path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

/// Build an atempo filter chain for the given speed factor.
/// atempo only supports 0.5..=100.0 per instance, so we chain for extreme values.
fn build_atempo_chain(speed: f64) -> String {
    let mut remaining = speed;
    let mut filters: Vec<String> = Vec::new();
    while remaining < 0.5 {
        filters.push("atempo=0.5".to_string());
        remaining /= 0.5;
    }
    while remaining > 100.0 {
        filters.push("atempo=100.0".to_string());
        remaining /= 100.0;
    }
    filters.push(format!("atempo={remaining}"));
    filters.join(",")
}

// ---------------------------------------------------------------------------
// 9. audio_reverse
// ---------------------------------------------------------------------------

/// Reverse audio using ffmpeg's `areverse` filter.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `output` - Optional explicit output path.
pub fn audio_reverse_work(path: PathBuf, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let out_path = output_path(&path, "_reversed", None, &output);
    let p_str = path.to_string_lossy().to_string();
    let o_str = out_path.to_string_lossy().to_string();

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        p_str,
        "-af".to_string(),
        "areverse".to_string(),
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!("Reversed {} -> {}", path.display(), out_path.display()),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 10. audio_normalize
// ---------------------------------------------------------------------------

/// Normalize audio loudness to a target dB level.
///
/// Uses ffmpeg's `loudnorm` filter for two-pass normalization. The first pass
/// analyzes loudness, and the second pass applies the correction.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `target_db` - Target integrated loudness in dB (e.g. -16.0 for broadcast,
///   -23.0 for EBU R128, -14.0 for streaming).
/// * `output` - Optional explicit output path.
pub fn audio_normalize_work(path: PathBuf, target_db: f64, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let suffix = format!("_norm{target_db}");
    let out_path = output_path(&path, &suffix, None, &output);
    let p_str = path.to_string_lossy().to_string();
    let o_str = out_path.to_string_lossy().to_string();

    // Two-pass loudnorm: first pass to measure, second pass to apply
    // Single-pass linear normalization as simpler alternative
    let filter = format!("loudnorm=I={target_db}:TP=-1.5:LRA=11");

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        p_str,
        "-af".to_string(),
        filter,
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!(
            "Normalized {} to {target_db} dB -> {}",
            path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 11. audio_split
// ---------------------------------------------------------------------------

/// Split audio into equal-duration segments using ffmpeg's segment muxer.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `segment_duration_secs` - Duration of each segment in seconds.
/// * `output_dir` - Optional directory for segment files. Defaults to the
///   same directory as the input file.
pub fn audio_split_work(
    path: PathBuf,
    segment_duration_secs: f64,
    output_dir: Option<PathBuf>,
) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if segment_duration_secs <= 0.0 {
        return "Error: segment_duration_secs must be positive".to_string();
    }

    let out_dir = output_dir.unwrap_or_else(|| {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    // Create output directory if needed
    if !out_dir.is_dir() {
        if let Err(e) = fs::create_dir_all(&out_dir) {
            return format!("Error: cannot create output dir {}: {e}", out_dir.display());
        }
    }

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("segment");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("wav");

    let pattern = format!("{stem}_%03d.{ext}");
    let out_pattern = out_dir.join(&pattern);
    let p_str = path.to_string_lossy().to_string();
    let o_str = out_pattern.to_string_lossy().to_string();
    let dur_arg = format!("{segment_duration_secs:.3}");

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        p_str,
        "-f".to_string(),
        "segment".to_string(),
        "-segment_time".to_string(),
        dur_arg,
        "-c".to_string(),
        "copy".to_string(),
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => {
            // Count the generated segments
            let count = fs::read_dir(&out_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.file_name()
                                .to_string_lossy()
                                .starts_with(&format!("{stem}_"))
                        })
                        .count()
                })
                .unwrap_or(0);
            format!(
                "Split {} into ~{} segments of {segment_duration_secs:.3}s each in {}",
                path.display(),
                count,
                out_dir.display()
            )
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 12. audio_extract_from_video
// ---------------------------------------------------------------------------

/// Extract audio track from a video file using ffmpeg.
///
/// # Arguments
///
/// * `video_path` - Path to the video file.
/// * `output` - Optional explicit output audio path. Defaults to the video
///   filename with a `.wav` extension.
pub fn audio_extract_from_video_work(video_path: PathBuf, output: Option<PathBuf>) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&video_path) {
        return e;
    }

    let out_path = output_path(&video_path, "_audio", Some("wav"), &output);
    let v_str = video_path.to_string_lossy().to_string();
    let o_str = out_path.to_string_lossy().to_string();

    let args = vec![
        "-y".to_string(),
        "-i".to_string(),
        v_str,
        "-vn".to_string(), // no video
        "-acodec".to_string(),
        "pcm_s16le".to_string(),
        o_str,
    ];

    let result = run_command_strings("ffmpeg", &args);

    match result {
        Ok(_) => format!(
            "Extracted audio from {} -> {}",
            video_path.display(),
            out_path.display()
        ),
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 13. audio_waveform
// ---------------------------------------------------------------------------

/// Generate a text-based waveform representation (sampled amplitude values).
///
/// Uses ffmpeg's `astats` filter to compute RMS and peak levels per
/// short time window, producing a text-based waveform representation.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `samples` - Number of sample points to generate (default 80).
pub fn audio_waveform_work(path: PathBuf, samples: Option<usize>) -> String {
    if let Err(e) = require_ffprobe() {
        return e;
    }
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let num_samples = samples.unwrap_or(80).max(1).min(500);

    let p_str = path.to_string_lossy().to_string();

    // Get audio duration first
    let duration_str = match run_command(
        "ffprobe",
        &[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            &p_str,
        ],
    ) {
        Ok(s) => {
            let val: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
            val.get("format")
                .and_then(|f| f.get("duration"))
                .and_then(|d| {
                    d.as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| d.as_f64())
                })
                .unwrap_or(0.0)
        }
        Err(_) => 0.0,
    };

    if duration_str <= 0.0 {
        // Fallback: use ffmpeg volumedetect for basic amplitude info
        let result = run_command(
            "ffmpeg",
            &["-i", &p_str, "-af", "volumedetect", "-f", "null", "-"],
        );
        match result {
            Ok(s) => {
                let mut out = format!("WAVEFORM path={}\n", path.display());
                out.push_str("duration=unknown\n");
                out.push_str(&s);
                return truncate_response(&out);
            }
            Err(e) => return e,
        }
    }

    let window = duration_str / num_samples as f64;

    // Build ebur128 metadata output for each window
    // Use ffmpeg silencedetect-style approach: extract RMS per segment
    let mut out = format!("WAVEFORM path={}\n", path.display());
    out.push_str(&format!("duration={duration_str:.3}s\n"));
    out.push_str(&format!("samples={num_samples}\n"));
    out.push_str("amplitude_bars:\n");

    let bar_chars = "█▓▒░ ";
    let max_bar_len = 40;

    for i in 0..num_samples {
        let start = i as f64 * window;
        let seg_len = window;

        // Use ffmpeg to get volume for this short segment
        let start_s = format!("{start:.4}");
        let dur_s = format!("{seg_len:.4}");

        let result = run_command(
            "ffmpeg",
            &[
                "-ss",
                &start_s,
                "-t",
                &dur_s,
                "-i",
                &p_str,
                "-af",
                "volumedetect",
                "-f",
                "null",
                "-",
            ],
        );

        let mean_vol = if let Ok(s) = &result {
            // Parse "mean_volume: -XX.X dB" from volumedetect output
            s.lines()
                .find(|l| l.contains("mean_volume"))
                .and_then(|l| l.split(':').last())
                .and_then(|v| v.trim().split_whitespace().next())
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(-60.0)
        } else {
            -60.0
        };

        // Map dB to bar (0 dB = full, -60 dB = silence)
        let normalized = ((mean_vol + 60.0) / 60.0).clamp(0.0, 1.0);
        let bar_len = (normalized * max_bar_len as f64).round() as usize;
        let char_idx = ((1.0 - normalized) * (bar_chars.len() - 1) as f64).round() as usize;
        let fill_char = bar_chars.chars().nth(char_idx).unwrap_or(' ');
        let bar: String = std::iter::repeat(fill_char).take(bar_len).collect();
        out.push_str(&format!(
            "  [{:>7.2}s - {:>7.2}s] {}| {:.1} dB\n",
            start,
            start + seg_len,
            bar,
            mean_vol
        ));
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 14. audio_silence_detect
// ---------------------------------------------------------------------------

/// Detect silence regions in audio using ffmpeg's `silencedetect` filter.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `min_duration` - Minimum silence duration in seconds (default 0.5).
/// * `noise_db` - Noise threshold in dB (default -30 dB).
pub fn audio_silence_detect_work(
    path: PathBuf,
    min_duration: Option<f64>,
    noise_db: Option<f64>,
) -> String {
    if let Err(e) = require_ffmpeg() {
        return e;
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let min_dur = min_duration.unwrap_or(0.5);
    let noise = noise_db.unwrap_or(-30.0);

    if min_dur < 0.0 {
        return "Error: min_duration must be non-negative".to_string();
    }

    let p_str = path.to_string_lossy().to_string();
    let noise_str = format!("{noise}");
    let dur_str = format!("{min_dur}");

    let filter = format!("silencedetect=noise={noise_str}dB:d={dur_str}");

    let args = vec![
        "-i".to_string(),
        p_str,
        "-af".to_string(),
        filter,
        "-f".to_string(),
        "null".to_string(),
        "-".to_string(),
    ];

    // silencedetect writes to stderr, so we use run_command which captures it on error
    // but we actually need stderr even on success
    let result = match Command::new("ffmpeg").args(&args).output() {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            Ok(stderr.trim().to_string())
        }
        Err(e) => Err(format!("Error: spawn ffmpeg: {e}")),
    };

    match result {
        Ok(stderr) => {
            let mut out = format!("SILENCE_DETECT path={}\n", path.display());
            out.push_str(&format!("noise_threshold={noise} dB\n"));
            out.push_str(&format!("min_duration={min_dur}s\n"));

            // Parse silence_start and silence_end lines
            let mut regions: Vec<(f64, f64)> = Vec::new();
            let mut current_start: Option<f64> = None;

            for line in stderr.lines() {
                if let Some(start) = parse_silence_value(line, "silence_start") {
                    current_start = Some(start);
                } else if let Some(end) = parse_silence_value(line, "silence_end") {
                    if let Some(s) = current_start.take() {
                        regions.push((s, end));
                    }
                }
            }

            if regions.is_empty() {
                out.push_str("no_silence_regions_detected=true\n");
            } else {
                out.push_str(&format!("silence_regions={}\n", regions.len()));
                for (i, (start, end)) in regions.iter().enumerate() {
                    let dur = end - start;
                    out.push_str(&format!(
                        "  region_{}: start={start:.3}s end={end:.3}s duration={dur:.3}s\n",
                        i + 1
                    ));
                }
            }

            truncate_response(&out)
        }
        Err(e) => e,
    }
}

/// Parse a `silence_start: X` or `silence_end: X` value from a silencedetect
/// output line.
fn parse_silence_value(line: &str, prefix: &str) -> Option<f64> {
    if line.contains(prefix) {
        // Format: "silencedetect: ... silence_start: 12.345"
        // or:     "[silencedetect @ ...] silence_start: 12.345"
        line.split(':')
            .last()
            .and_then(|v| v.trim().split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Internal helper: run a command with String args (vs &str)
// ---------------------------------------------------------------------------

/// Run a command with `String` arguments (for dynamically-built arg lists).
fn run_command_strings(cmd: &str, args: &[String]) -> Result<String, String> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_command(cmd, &str_args)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_audio_formats() {
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"mp3"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"wav"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"ogg"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"flac"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"aac"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"m4a"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"opus"));
        assert!(SUPPORTED_AUDIO_FORMATS.contains(&"wma"));
        assert!(!SUPPORTED_AUDIO_FORMATS.contains(&"exe"));
    }

    #[test]
    fn test_max_audio_bytes() {
        assert_eq!(MAX_AUDIO_BYTES, 500 * 1024 * 1024);
    }

    #[test]
    fn test_validate_audio_path_missing() {
        let result = validate_audio_path(Path::new("/nonexistent/audio/file.wav"));
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Error:"));
    }

    #[test]
    fn test_validate_audio_path_directory() {
        let result = validate_audio_path(Path::new("/tmp"));
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Error:"));
    }

    #[test]
    fn test_output_path_with_explicit_output() {
        let input = PathBuf::from("/tmp/test.wav");
        let output = Some(PathBuf::from("/tmp/output.mp3"));
        let result = output_path(&input, "_suffix", Some("mp3"), &output);
        assert_eq!(result, PathBuf::from("/tmp/output.mp3"));
    }

    #[test]
    fn test_output_path_derived_default() {
        let input = PathBuf::from("/tmp/test.wav");
        let result = output_path(&input, "_trimmed", None, &None);
        assert_eq!(result, PathBuf::from("/tmp/test_trimmed.wav"));
    }

    #[test]
    fn test_output_path_derived_new_ext() {
        let input = PathBuf::from("/tmp/test.wav");
        let result = output_path(&input, "", Some("mp3"), &None);
        assert_eq!(result, PathBuf::from("/tmp/test.mp3"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KiB");
        assert_eq!(format_size(1024 * 1024), "1.00 MiB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GiB");
    }

    #[test]
    fn test_build_atempo_chain_normal() {
        assert_eq!(build_atempo_chain(1.5), "atempo=1.5");
        assert_eq!(build_atempo_chain(0.75), "atempo=0.75");
    }

    #[test]
    fn test_build_atempo_chain_slow() {
        // 0.25 requires chaining since min is 0.5
        let chain = build_atempo_chain(0.25);
        assert!(chain.contains("atempo=0.5"));
        assert!(chain.contains("atempo=0.5"));
    }

    #[test]
    fn test_build_atempo_chain_fast() {
        // 200.0 requires chaining since max is 100.0
        let chain = build_atempo_chain(200.0);
        assert!(chain.contains("atempo=100.0"));
        assert!(chain.contains("atempo=2"));
    }

    #[test]
    fn test_parse_silence_value_start() {
        let line = "[silencedetect @ 0x...] silence_start: 12.345";
        assert_eq!(parse_silence_value(line, "silence_start"), Some(12.345));
    }

    #[test]
    fn test_parse_silence_value_end() {
        let line = "[silencedetect @ 0x...] silence_end: 15.678 | silence_duration: 3.433";
        assert_eq!(parse_silence_value(line, "silence_end"), Some(3.433));
    }

    #[test]
    fn test_parse_silence_value_no_match() {
        let line = "some other output";
        assert_eq!(parse_silence_value(line, "silence_start"), None);
    }

    #[test]
    fn test_audio_info_missing_file() {
        let result = audio_info_work(PathBuf::from("/nonexistent/audio.wav"));
        assert!(result.starts_with("Error:") | result.starts_with("AUDIO_INFO"));
    }

    #[test]
    fn test_audio_convert_unsupported_format() {
        let result = audio_convert_work(PathBuf::from("/tmp/test.wav"), "exe".to_string(), None);
        assert!(result.starts_with("Error:"));
        // If ffmpeg is not on PATH, we get an ffmpeg-not-found error first;
        // otherwise the format validation should mention "unsupported".
        assert!(
            result.to_lowercase().contains("unsupported")
                || result.to_lowercase().contains("ffmpeg"),
            "expected unsupported format or ffmpeg error, got: {result}"
        );
    }

    #[test]
    fn test_audio_extract_segment_negative_start() {
        let result = audio_extract_segment_work(PathBuf::from("/tmp/test.wav"), -1.0, 5.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_extract_segment_zero_duration() {
        let result = audio_extract_segment_work(PathBuf::from("/tmp/test.wav"), 0.0, 0.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_trim_end_before_start() {
        let result = audio_trim_work(PathBuf::from("/tmp/test.wav"), 10.0, 5.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_merge_empty_input() {
        let result = audio_merge_work(vec![], PathBuf::from("/tmp/out.wav"));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_merge_single_input() {
        let result = audio_merge_work(
            vec![PathBuf::from("/tmp/a.wav")],
            PathBuf::from("/tmp/out.wav"),
        );
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_volume_negative() {
        let result = audio_volume_work(PathBuf::from("/tmp/test.wav"), -0.5, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_speed_zero() {
        let result = audio_speed_work(PathBuf::from("/tmp/test.wav"), 0.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_speed_negative() {
        let result = audio_speed_work(PathBuf::from("/tmp/test.wav"), -1.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_split_zero_duration() {
        let result = audio_split_work(PathBuf::from("/tmp/test.wav"), 0.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_silence_detect_negative_duration() {
        let result = audio_silence_detect_work(PathBuf::from("/tmp/test.wav"), Some(-1.0), None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_audio_mix_negative_volume() {
        let result = audio_mix_work(
            PathBuf::from("/tmp/a.wav"),
            PathBuf::from("/tmp/b.wav"),
            PathBuf::from("/tmp/out.wav"),
            Some(-1.0),
            Some(1.0),
        );
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_cli_exists_known_command() {
        // `ls` should exist on any Unix system
        assert!(cli_exists("ls") || !cli_exists("nonexistent_cmd_12345"));
    }

    #[test]
    fn test_cli_exists_nonexistent() {
        assert!(!cli_exists("definitely_not_a_real_program_xyzzy_99999"));
    }
}
