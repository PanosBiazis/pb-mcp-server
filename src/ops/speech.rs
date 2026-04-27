//! # Speech Recognition & Text-to-Speech Tools
//!
//! Voice recognition (speech-to-text) and text-to-speech capabilities for AI
//! agents using local tools: **Whisper** CLI for transcription/translation,
//! **espeak** / **espeak-ng** for TTS, **sox** / **ffmpeg** for audio
//! processing, and **arecord** (ALSA) for microphone capture.
//!
//! All functions return human-readable `String` values. Errors are prefixed
//! with `"Error:"`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum input audio file size (500 MiB).
const MAX_AUDIO_BYTES: u64 = 500 * 1024 * 1024;

/// Recognised Whisper model names.
const WHISPER_MODELS: &[&str] = &["tiny", "base", "small", "medium", "large"];

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

/// Check whether a CLI program exists on `PATH` by running `<program> --version`.
fn cli_exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

/// Resolve the whisper CLI binary name.
///
/// Checks `whisper` first, then `whisper-ctranslate2`.
fn resolve_whisper_cmd() -> Result<&'static str, String> {
    if cli_exists("whisper") {
        return Ok("whisper");
    }
    if cli_exists("whisper-ctranslate2") {
        return Ok("whisper-ctranslate2");
    }
    Err(
        "Error: whisper CLI not found on PATH (tried 'whisper' and 'whisper-ctranslate2')"
            .to_string(),
    )
}

/// Resolve the espeak CLI binary name.
///
/// Checks `espeak-ng` first, then `espeak`.
fn resolve_espeak_cmd() -> Result<&'static str, String> {
    if cli_exists("espeak-ng") {
        return Ok("espeak-ng");
    }
    if cli_exists("espeak") {
        return Ok("espeak");
    }
    Err("Error: espeak/espeak-ng not found on PATH".to_string())
}

/// Validate that a Whisper model name is recognised.
fn validate_whisper_model(model: &str) -> Result<(), String> {
    if WHISPER_MODELS.contains(&model) {
        Ok(())
    } else {
        Err(format!(
            "Error: unknown whisper model '{}'. Supported: {}",
            model,
            WHISPER_MODELS.join(", ")
        ))
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

/// Run a command with `String` arguments and return its stdout, or an error
/// string on failure.
fn run_command_strings(cmd: &str, args: &[String]) -> Result<String, String> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_command(cmd, &str_args)
}

/// Run a command, returning combined stdout + stderr.
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

// ---------------------------------------------------------------------------
// 1. speech_transcribe_file
// ---------------------------------------------------------------------------

/// Transcribe an audio file to text using the Whisper CLI.
///
/// Runs the `whisper` (or `whisper-ctranslate2`) command with the given audio
/// file. An optional `language` hint (e.g. `"en"`, `"fr"`) and `model` name
/// (`"tiny"`, `"base"`, `"small"`, `"medium"`, `"large"`) can be supplied.
///
/// # Arguments
///
/// * `path` - Path to the audio file to transcribe.
/// * `language` - Optional ISO 639-1 language code hint.
/// * `model` - Optional Whisper model name (defaults to `"base"`).
pub fn speech_transcribe_file_work(
    path: PathBuf,
    language: Option<String>,
    model: Option<String>,
) -> String {
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    let mdl = model.unwrap_or_else(|| "base".to_string());
    if let Err(e) = validate_whisper_model(&mdl) {
        return e;
    }
    let p_str = path.to_string_lossy().to_string();
    let mut args: Vec<String> = vec![
        p_str,
        "--model".to_string(),
        mdl.clone(),
        "--output_format".to_string(),
        "txt".to_string(),
    ];
    if let Some(ref lang) = language {
        args.push("--language".to_string());
        args.push(lang.clone());
    }

    let result = run_command_strings(whisper, &args);
    match result {
        Ok(stdout) => {
            let mut out = format!("SPEECH_TRANSCRIBE_FILE path={}\n", path.display());
            out.push_str(&format!("model={mdl}\n"));
            if let Some(ref lang) = language {
                out.push_str(&format!("language={lang}\n"));
            }
            out.push_str(&format!("\n{stdout}"));
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 2. speech_transcribe_microphone
// ---------------------------------------------------------------------------

/// Record from the microphone and transcribe the recording.
///
/// Uses `arecord` (ALSA) to capture audio for `duration_secs` seconds, then
/// passes the resulting WAV to Whisper for transcription.
///
/// # Arguments
///
/// * `duration_secs` - Recording duration in seconds.
/// * `language` - Optional ISO 639-1 language code hint.
pub fn speech_transcribe_microphone_work(duration_secs: u64, language: Option<String>) -> String {
    if duration_secs == 0 {
        return "Error: duration_secs must be > 0".to_string();
    }
    if !cli_exists("arecord") {
        return "Error: arecord (ALSA) not found on PATH".to_string();
    }
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };

    // Create a temporary file for the recording
    let tmp_dir = std::env::temp_dir();
    let recording_path = tmp_dir.join(format!("mcp_mic_recording_{}.wav", std::process::id()));
    let rec_str = recording_path.to_string_lossy().to_string();
    let dur_str = duration_secs.to_string();

    // Record using arecord
    let rec_result = run_command(
        "arecord",
        &[
            "-f", "S16_LE", "-r", "16000", "-c", "1", "-d", &dur_str, &rec_str,
        ],
    );
    if let Err(e) = rec_result {
        return format!("Error: microphone recording failed: {e}");
    }

    // Transcribe the recording
    let mdl = "base".to_string();
    let mut args: Vec<String> = vec![
        rec_str.clone(),
        "--model".to_string(),
        mdl.clone(),
        "--output_format".to_string(),
        "txt".to_string(),
    ];
    if let Some(ref lang) = language {
        args.push("--language".to_string());
        args.push(lang.clone());
    }

    let result = run_command_strings(whisper, &args);
    // Clean up temp file
    let _ = fs::remove_file(&recording_path);

    match result {
        Ok(stdout) => {
            let mut out = format!("SPEECH_TRANSCRIBE_MICROPHONE duration={duration_secs}s\n");
            out.push_str(&format!("model={mdl}\n"));
            if let Some(ref lang) = language {
                out.push_str(&format!("language={lang}\n"));
            }
            out.push_str(&format!("\n{stdout}"));
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 3. speech_transcribe_stream
// ---------------------------------------------------------------------------

/// Transcribe a long audio file in segments for better accuracy.
///
/// Splits the audio into `segment_duration_secs` chunks using `ffmpeg`,
/// transcribes each segment individually with Whisper, then combines the
/// results.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
/// * `segment_duration_secs` - Duration of each segment in seconds.
/// * `language` - Optional ISO 639-1 language code hint.
pub fn speech_transcribe_stream_work(
    path: PathBuf,
    segment_duration_secs: f64,
    language: Option<String>,
) -> String {
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if !cli_exists("ffmpeg") && !cli_exists("ffprobe") {
        return "Error: ffmpeg/ffprobe not found on PATH (needed to split audio)".to_string();
    }
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if segment_duration_secs <= 0.0 {
        return "Error: segment_duration_secs must be positive".to_string();
    }

    let p_str = path.to_string_lossy().to_string();

    // Get total duration via ffprobe
    let dur_output = match run_command(
        "ffprobe",
        &[
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            &p_str,
        ],
    ) {
        Ok(s) => s,
        Err(e) => return format!("Error: cannot determine audio duration: {e}"),
    };
    let total_duration: f64 = match dur_output.trim().parse() {
        Ok(d) => d,
        Err(_) => return format!("Error: cannot parse duration from ffprobe output: {dur_output}"),
    };

    let seg_dur = segment_duration_secs;
    let num_segments = ((total_duration / seg_dur).ceil() as usize).max(1);
    let tmp_dir = std::env::temp_dir().join(format!(
        "mcp_stream_{}_{}",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("audio"),
        std::process::id()
    ));
    if let Err(e) = fs::create_dir_all(&tmp_dir) {
        return format!("Error: cannot create temp directory: {e}");
    }

    let mut transcripts: Vec<String> = Vec::new();
    let mdl = "base".to_string();

    for i in 0..num_segments {
        let start = i as f64 * seg_dur;
        let seg_path = tmp_dir.join(format!("segment_{i:04}.wav"));
        let seg_str = seg_path.to_string_lossy().to_string();
        let start_arg = format!("{start:.3}");
        let dur_arg = format!("{seg_dur:.3}");

        // Extract segment
        let split_result = run_command(
            "ffmpeg",
            &[
                "-y", "-i", &p_str, "-ss", &start_arg, "-t", &dur_arg, "-ar", "16000", "-ac", "1",
                &seg_str,
            ],
        );
        if let Err(e) = split_result {
            // Clean up
            let _ = fs::remove_dir_all(&tmp_dir);
            return format!("Error: ffmpeg segment {i} failed: {e}");
        }

        // Check segment exists and has content
        if !seg_path.is_file() {
            continue;
        }
        let seg_meta = match fs::metadata(&seg_path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if seg_meta.len() < 44 {
            // Smallest valid WAV header
            continue;
        }

        // Transcribe segment
        let mut args: Vec<String> = vec![
            seg_str.clone(),
            "--model".to_string(),
            mdl.clone(),
            "--output_format".to_string(),
            "txt".to_string(),
        ];
        if let Some(ref lang) = language {
            args.push("--language".to_string());
            args.push(lang.clone());
        }

        match run_command_strings(whisper, &args) {
            Ok(txt) => transcripts.push(txt),
            Err(e) => transcripts.push(format!("[segment {i} error: {e}]")),
        }

        // Remove segment file
        let _ = fs::remove_file(&seg_path);
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(&tmp_dir);

    let mut out = format!("SPEECH_TRANSCRIBE_STREAM path={}\n", path.display());
    out.push_str(&format!(
        "total_duration={total_duration:.1}s segments={num_segments} segment_length={seg_dur:.1}s\n"
    ));
    out.push_str(&format!("model={mdl}\n"));
    if let Some(ref lang) = language {
        out.push_str(&format!("language={lang}\n"));
    }
    out.push_str("\n--- Transcript ---\n");
    out.push_str(&transcripts.join("\n"));
    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 4. speech_text_to_speech
// ---------------------------------------------------------------------------

/// Convert text to speech audio using `espeak` or `espeak-ng`.
///
/// Saves the output as a WAV file. If `output` is `None` the file is written
/// to a conventional temp location.
///
/// # Arguments
///
/// * `text` - Text to synthesise.
/// * `voice` - Optional voice name (e.g. `"en-us"`, `"de"`).
/// * `speed` - Optional speed in words-per-minute (default 175).
/// * `output` - Optional output WAV file path.
pub fn speech_text_to_speech_work(
    text: String,
    voice: Option<String>,
    speed: Option<u32>,
    output: Option<PathBuf>,
) -> String {
    let espeak = match resolve_espeak_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if text.is_empty() {
        return "Error: text is empty".to_string();
    }
    let spd = speed.unwrap_or(175);
    if spd == 0 {
        return "Error: speed must be > 0".to_string();
    }

    let out_path = output.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("tts_output_{}.wav", std::process::id()))
    });
    let o_str = out_path.to_string_lossy().to_string();

    let mut args: Vec<String> = vec![
        "-w".to_string(),
        o_str.clone(),
        "-s".to_string(),
        spd.to_string(),
    ];
    if let Some(ref v) = voice {
        args.push("-v".to_string());
        args.push(v.clone());
    }
    args.push(text.clone());

    let result = run_command_strings(espeak, &args);
    match result {
        Ok(_) => {
            let mut out = "SPEECH_TEXT_TO_SPEECH\n".to_string();
            out.push_str(&format!(
                "voice={}\n",
                voice.as_deref().unwrap_or("default")
            ));
            out.push_str(&format!("speed={spd}\n"));
            out.push_str(&format!("output={o_str}\n"));
            out.push_str(&format!("text_length={} chars\n", text.chars().count()));
            if out_path.is_file() {
                if let Ok(meta) = fs::metadata(&out_path) {
                    out.push_str(&format!("file_size={} bytes\n", meta.len()));
                }
            }
            out.push_str("\nTTS audio generated successfully.");
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 5. speech_text_to_speech_ssml
// ---------------------------------------------------------------------------

/// Text-to-speech with SSML markup support (limited).
///
/// espeak / espeak-ng have built-in markup support (pitch, speed, pause
/// markers) which is a subset of SSML. The markup is passed through via the
/// `-m` flag.
///
/// # Arguments
///
/// * `ssml` - SSML-marked-up text to synthesise.
/// * `voice` - Optional voice name.
/// * `output` - Optional output WAV file path.
pub fn speech_text_to_speech_ssml_work(
    ssml: String,
    voice: Option<String>,
    output: Option<PathBuf>,
) -> String {
    let espeak = match resolve_espeak_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if ssml.is_empty() {
        return "Error: ssml text is empty".to_string();
    }

    let out_path = output.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("tts_ssml_output_{}.wav", std::process::id()))
    });
    let o_str = out_path.to_string_lossy().to_string();

    let mut args: Vec<String> = vec!["-w".to_string(), o_str.clone(), "-m".to_string()];
    if let Some(ref v) = voice {
        args.push("-v".to_string());
        args.push(v.clone());
    }
    args.push(ssml.clone());

    let result = run_command_strings(espeak, &args);
    match result {
        Ok(_) => {
            let mut out = "SPEECH_TEXT_TO_SPEECH_SSML\n".to_string();
            out.push_str(&format!(
                "voice={}\n",
                voice.as_deref().unwrap_or("default")
            ));
            out.push_str(&format!("output={o_str}\n"));
            out.push_str(&format!("ssml_length={} chars\n", ssml.chars().count()));
            out.push_str("\nNote: espeak supports a limited SSML subset (pitch, speed, pause).");
            if out_path.is_file() {
                if let Ok(meta) = fs::metadata(&out_path) {
                    out.push_str(&format!("\nfile_size={} bytes", meta.len()));
                }
            }
            out.push_str("\nTTS SSML audio generated successfully.");
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 6. speech_list_voices
// ---------------------------------------------------------------------------

/// List available TTS voices using `espeak --voices`.
///
/// Returns a human-readable table of all installed voice names, languages,
/// and identifiers.
pub fn speech_list_voices_work() -> String {
    let espeak = match resolve_espeak_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };

    let result = run_command(espeak, &["--voices"]);
    match result {
        Ok(stdout) => {
            let mut out = "SPEECH_LIST_VOICES\n".to_string();
            out.push_str(&stdout);
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 7. speech_list_models
// ---------------------------------------------------------------------------

/// List available Whisper models by checking common download directories.
///
/// Searches `~/.cache/whisper/` and `/usr/share/whisper/` for model files.
pub fn speech_list_models_work() -> String {
    let mut out = "SPEECH_LIST_MODELS\n".to_string();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    let search_dirs: Vec<PathBuf> = vec![
        PathBuf::from(format!("{home}/.cache/whisper")),
        PathBuf::from("/usr/share/whisper"),
        PathBuf::from("/usr/local/share/whisper"),
        PathBuf::from(format!("{home}/.local/share/whisper")),
    ];

    let mut found: Vec<String> = Vec::new();

    for dir in &search_dirs {
        if dir.is_dir() {
            out.push_str(&format!("Searching {} ...\n", dir.display()));
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Whisper model files are typically .pt or .bin
                    if name.ends_with(".pt") || name.ends_with(".bin") {
                        for model_name in WHISPER_MODELS {
                            if name.contains(model_name) && !found.contains(&model_name.to_string())
                            {
                                found.push(model_name.to_string());
                                let meta = fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
                                out.push_str(&format!(
                                    "  {model_name}: {} ({})\n",
                                    name,
                                    format_size(meta)
                                ));
                            }
                        }
                    }
                }
            }
        } else {
            out.push_str(&format!("{} (not found)\n", dir.display()));
        }
    }

    out.push_str("\n--- Summary ---\n");
    if found.is_empty() {
        out.push_str("No whisper model files found in common directories.\n");
        out.push_str(
            "Models can be downloaded via: whisper --model <name> (auto-downloads on first use)\n",
        );
        out.push_str(&format!(
            "Available model names: {}\n",
            WHISPER_MODELS.join(", ")
        ));
    } else {
        out.push_str(&format!("Found models: {}\n", found.join(", ")));
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 8. speech_detect_language
// ---------------------------------------------------------------------------

/// Detect the spoken language in an audio file using Whisper's language
/// detection.
///
/// Runs Whisper without a `--language` flag so that it auto-detects the
/// language, or uses `--language ""` to force detection mode depending on the
/// CLI version.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
pub fn speech_detect_language_work(path: PathBuf) -> String {
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let p_str = path.to_string_lossy().to_string();
    let args: Vec<String> = vec![
        p_str,
        "--model".to_string(),
        "base".to_string(),
        "--output_format".to_string(),
        "txt".to_string(),
        "--language".to_string(),
        "".to_string(),
    ];

    let result = run_command_strings(whisper, &args);
    match result {
        Ok(stdout) => {
            let mut out = format!("SPEECH_DETECT_LANGUAGE path={}\n", path.display());
            out.push_str(&format!("\n{stdout}"));
            truncate_response(&out)
        }
        Err(e) => {
            // If --language "" is not supported, try without the flag
            let p_str2 = path.to_string_lossy().to_string();
            let args2: Vec<String> = vec![
                p_str2,
                "--model".to_string(),
                "base".to_string(),
                "--output_format".to_string(),
                "txt".to_string(),
            ];
            match run_command_strings(whisper, &args2) {
                Ok(stdout) => {
                    let mut out = format!("SPEECH_DETECT_LANGUAGE path={}\n", path.display());
                    out.push_str(&format!("\n{stdout}"));
                    truncate_response(&out)
                }
                Err(e2) => format!("Error: language detection failed:\n{e}\n{e2}"),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 9. speech_translate
// ---------------------------------------------------------------------------

/// Transcribe and translate audio to English using Whisper's translation mode.
///
/// Whisper's `--task translate` mode produces an English transcription
/// regardless of the source language.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
/// * `target_language` - Optional target language hint (Whisper translate mode
///   always outputs English; this parameter is for informational purposes).
pub fn speech_translate_work(path: PathBuf, target_language: Option<String>) -> String {
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let p_str = path.to_string_lossy().to_string();
    let mdl = "base".to_string();
    let mut args: Vec<String> = vec![
        p_str,
        "--model".to_string(),
        mdl.clone(),
        "--task".to_string(),
        "translate".to_string(),
        "--output_format".to_string(),
        "txt".to_string(),
    ];
    if let Some(ref lang) = target_language {
        args.push("--language".to_string());
        args.push(lang.clone());
    }

    let result = run_command_strings(whisper, &args);
    match result {
        Ok(stdout) => {
            let mut out = format!("SPEECH_TRANSLATE path={}\n", path.display());
            out.push_str(&format!("model={mdl}\n"));
            out.push_str("task=translate (to English)\n");
            if let Some(ref lang) = target_language {
                out.push_str(&format!("source_language_hint={lang}\n"));
            }
            out.push_str(&format!("\n--- English Translation ---\n{stdout}"));
            truncate_response(&out)
        }
        Err(e) => e,
    }
}

// ---------------------------------------------------------------------------
// 10. speech_diarize
// ---------------------------------------------------------------------------

/// Basic speaker diarization — determine who spoke when.
///
/// Uses a simple approach: detect silence boundaries via `ffmpeg`
/// `silencedetect`, then label segments as alternating speakers.
/// If `pyannote` is available via Python, a more sophisticated approach is
/// attempted.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
/// * `num_speakers` - Optional expected number of speakers (default 2 for
///   simple approach).
pub fn speech_diarize_work(path: PathBuf, num_speakers: Option<usize>) -> String {
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }
    if !cli_exists("ffmpeg") {
        return "Error: ffmpeg not found on PATH (needed for silence detection)".to_string();
    }

    let num_spk = num_speakers.unwrap_or(2);
    if num_spk == 0 {
        return "Error: num_speakers must be > 0".to_string();
    }

    let p_str = path.to_string_lossy().to_string();

    // First, try pyannote via Python for sophisticated diarization
    if cli_exists("python3") {
        let script = r#"
import sys
try:
    from pyannote.audio import Pipeline
    print("pyannote_available=True")
except ImportError:
    print("pyannote_available=False")
"#;
        let py_check = Command::new("python3").arg("-c").arg(script).output();

        if let Ok(output) = py_check {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("pyannote_available=True") {
                // Pyannote is available — attempt real diarization
                let p_str_escaped = p_str.replace('"', r#"\""#);
                let diarize_script = format!(
                    r#"
import sys, json
try:
    from pyannote.audio import Pipeline
    pipeline = Pipeline.from_pretrained("pyannote/speaker-diarization-3.1",
                                         use_auth_token=None)
    diarization = pipeline("{p_str_escaped}", num_speakers={num_spk})
    for turn, _, speaker in diarization.itertracks(yield_label=True):
        print(f"{{speaker}} {{turn.start:.1f}}s-{{turn.end:.1f}}s")
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
                    num_spk = num_spk
                );
                let py_result = Command::new("python3")
                    .arg("-c")
                    .arg(&diarize_script)
                    .output();

                match py_result {
                    Ok(o) if o.status.success() => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        let mut out = format!("SPEECH_DIARIZE path={}\n", path.display());
                        out.push_str("method=pyannote\n");
                        out.push_str(&format!("num_speakers={num_spk}\n\n"));
                        out.push_str(stdout.trim());
                        return truncate_response(&out);
                    }
                    _ => {
                        // Fall through to simple method
                    }
                }
            }
        }
    }

    // Simple approach: use ffmpeg silencedetect to find speech segments,
    // then assign alternating speaker labels.
    let silence_result = run_command_combined(
        "ffmpeg",
        &[
            "-i",
            &p_str,
            "-af",
            "silencedetect=noise=-30dB:d=0.5",
            "-f",
            "null",
            "-",
        ],
    );

    let mut speech_segments: Vec<(f64, f64)> = Vec::new();

    if let Ok(output) = silence_result {
        // Parse silence_start / silence_end to find speech segments
        let mut silence_starts: Vec<f64> = Vec::new();
        let mut silence_ends: Vec<f64> = Vec::new();

        for line in output.lines() {
            if let Some(val) = parse_silence_value(line, "silence_start") {
                silence_starts.push(val);
            }
            if let Some(val) = parse_silence_value(line, "silence_end") {
                silence_ends.push(val);
            }
        }

        // Derive speech segments from silence boundaries
        // Speech = regions between silence_end and silence_start
        let mut current = 0.0;
        let mut si = 0; // silence_starts index
        let mut ei = 0; // silence_ends index

        loop {
            // Next speech segment: from current to next silence_start
            if si < silence_starts.len() && silence_starts[si] > current {
                speech_segments.push((current, silence_starts[si]));
                current = silence_starts[si];
            } else if si >= silence_starts.len() {
                // No more silences — speech from current to end
                speech_segments.push((current, -1.0)); // -1 = until end
                break;
            }

            // Skip over silence: advance current to next silence_end
            if ei < silence_ends.len() {
                current = silence_ends[ei];
                ei += 1;
            }
            si += 1;

            if si >= silence_starts.len() && ei >= silence_ends.len() {
                // Remaining speech from current to end
                speech_segments.push((current, -1.0));
                break;
            }
        }
    }

    // If no segments found, the whole file is one speech segment
    if speech_segments.is_empty() {
        speech_segments.push((0.0, -1.0));
    }

    let mut out = format!("SPEECH_DIARIZE path={}\n", path.display());
    out.push_str("method=silence_based\n");
    out.push_str(&format!("num_speakers={num_spk}\n\n"));

    for (i, (seg_start, seg_end)) in speech_segments.iter().enumerate() {
        let speaker = format!("SPEAKER_{}", (i % num_spk) + 1);
        if *seg_end < 0.0 {
            out.push_str(&format!("{speaker}: {seg_start:.1}s - end\n"));
        } else {
            out.push_str(&format!("{speaker}: {seg_start:.1}s - {seg_end:.1}s\n"));
        }
    }

    out.push_str(
        "\nNote: This is a basic silence-based diarization. Install pyannote-audio for more accurate results.",
    );
    truncate_response(&out)
}

/// Parse a silence detection value from an ffmpeg silencedetect output line.
fn parse_silence_value(line: &str, key: &str) -> Option<f64> {
    let prefix = format!("{key}:");
    for token in line.split_whitespace() {
        if token.starts_with(&prefix) {
            let val_str = token.trim_start_matches(&prefix);
            return val_str.parse().ok();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 11. speech_enhance
// ---------------------------------------------------------------------------

/// Enhance audio quality for better transcription using `sox` noise reduction
/// or `ffmpeg` filters.
///
/// Applies noise reduction to improve the signal-to-noise ratio, making
/// subsequent transcription more accurate.
///
/// # Arguments
///
/// * `path` - Path to the input audio file.
/// * `noise_reduction` - Optional noise reduction amount 0.0–1.0 (default 0.5).
/// * `output` - Optional output path for the enhanced audio.
pub fn speech_enhance_work(
    path: PathBuf,
    noise_reduction: Option<f64>,
    output: Option<PathBuf>,
) -> String {
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let nr = noise_reduction.unwrap_or(0.5);
    if !(0.0..=1.0).contains(&nr) {
        return "Error: noise_reduction must be between 0.0 and 1.0".to_string();
    }

    let out_path = output_path(&path, "_enhanced", Some("wav"), &output);
    let o_str = out_path.to_string_lossy().to_string();
    let p_str = path.to_string_lossy().to_string();

    // Prefer sox for noise reduction (profiling approach)
    if cli_exists("sox") {
        // Create a noise profile from the first 0.5 seconds
        let tmp_dir = std::env::temp_dir();
        let noise_profile = tmp_dir.join(format!("noise_profile_{}.prof", std::process::id()));
        let prof_str = noise_profile.to_string_lossy().to_string();

        // Step 1: create noise profile from first 0.5s (assumed to be noise)
        let profile_result = run_command(
            "sox",
            &[&p_str, "-n", "trim", "0", "0.5", "noiseprof", &prof_str],
        );

        if profile_result.is_ok() && noise_profile.is_file() {
            // Step 2: apply noise reduction
            let nr_amount = format!("{}", nr * 100.0); // sox uses 0-100 range
            let enhance_result =
                run_command("sox", &[&p_str, &o_str, "noisered", &prof_str, &nr_amount]);

            // Clean up noise profile
            let _ = fs::remove_file(&noise_profile);

            if enhance_result.is_ok() {
                let mut out = "SPEECH_ENHANCE\n".to_string();
                out.push_str(&format!("input={}\n", path.display()));
                out.push_str(&format!("output={o_str}\n"));
                out.push_str("method=sox_noisered\n");
                out.push_str(&format!("noise_reduction={nr}\n"));
                if out_path.is_file() {
                    if let Ok(meta) = fs::metadata(&out_path) {
                        out.push_str(&format!("output_size={} bytes\n", meta.len()));
                    }
                }
                out.push_str("\nAudio enhanced successfully.");
                return truncate_response(&out);
            }
            // Clean up if sox failed
            let _ = fs::remove_file(&noise_profile);
        }

        // Fallback: sox high-pass filter to remove low-frequency noise
        let filter_result = run_command(
            "sox",
            &[
                &p_str,
                &o_str,
                "highpass",
                "200",
                "compand",
                "0.05,0.2",
                "6:-54,-54,-54,-54",
                "0,-90",
                "0.1",
            ],
        );

        if filter_result.is_ok() {
            let mut out = "SPEECH_ENHANCE\n".to_string();
            out.push_str(&format!("input={}\n", path.display()));
            out.push_str(&format!("output={o_str}\n"));
            out.push_str("method=sox_highpass_compand\n");
            out.push_str(&format!("noise_reduction={nr}\n"));
            if out_path.is_file() {
                if let Ok(meta) = fs::metadata(&out_path) {
                    out.push_str(&format!("output_size={} bytes\n", meta.len()));
                }
            }
            out.push_str("\nAudio enhanced successfully (high-pass + compand fallback).");
            return truncate_response(&out);
        }
    }

    // Fallback: ffmpeg afftdn filter for noise reduction
    if cli_exists("ffmpeg") {
        let nr_val = (nr * 97.0 + 3.0) as i32; // map 0-1 to 3-100 for fftdn
        let filter = format!("afftdn=nr={nr_val}");
        let args: Vec<String> = vec![
            "-y".to_string(),
            "-i".to_string(),
            p_str,
            "-af".to_string(),
            filter,
            o_str.clone(),
        ];

        let result = run_command_strings("ffmpeg", &args);
        match result {
            Ok(_) => {
                let mut out = "SPEECH_ENHANCE\n".to_string();
                out.push_str(&format!("input={}\n", path.display()));
                out.push_str(&format!("output={o_str}\n"));
                out.push_str("method=ffmpeg_afftdn\n");
                out.push_str(&format!("noise_reduction={nr}\n"));
                if out_path.is_file() {
                    if let Ok(meta) = fs::metadata(&out_path) {
                        out.push_str(&format!("output_size={} bytes\n", meta.len()));
                    }
                }
                out.push_str("\nAudio enhanced successfully.");
                return truncate_response(&out);
            }
            Err(e) => return e,
        }
    }

    "Error: neither sox nor ffmpeg found on PATH for audio enhancement".to_string()
}

// ---------------------------------------------------------------------------
// 12. speech_extract_words
// ---------------------------------------------------------------------------

/// Extract word-level timestamps from transcription using Whisper's word
/// timestamp mode.
///
/// Runs Whisper with the `--word_timestamps True` flag to get per-word timing
/// information.
///
/// # Arguments
///
/// * `path` - Path to the audio file.
/// * `language` - Optional ISO 639-1 language code hint.
pub fn speech_extract_words_work(path: PathBuf, language: Option<String>) -> String {
    let whisper = match resolve_whisper_cmd() {
        Ok(c) => c,
        Err(e) => return e,
    };
    if let Err(e) = validate_audio_path(&path) {
        return e;
    }

    let p_str = path.to_string_lossy().to_string();
    let mdl = "base".to_string();
    let mut args: Vec<String> = vec![
        p_str,
        "--model".to_string(),
        mdl.clone(),
        "--output_format".to_string(),
        "json".to_string(),
        "--word_timestamps".to_string(),
        "True".to_string(),
    ];
    if let Some(ref lang) = language {
        args.push("--language".to_string());
        args.push(lang.clone());
    }

    let result = run_command_strings(whisper, &args);
    match result {
        Ok(stdout) => {
            let mut out = format!("SPEECH_EXTRACT_WORDS path={}\n", path.display());
            out.push_str(&format!("model={mdl}\n"));
            if let Some(ref lang) = language {
                out.push_str(&format!("language={lang}\n"));
            }
            out.push_str("\n--- Word Timestamps ---\n");

            // Try to parse the JSON output for word-level timestamps
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(segments) = val.get("segments").and_then(|v| v.as_array()) {
                    for seg in segments {
                        if let Some(words) = seg.get("words").and_then(|v| v.as_array()) {
                            for word in words {
                                let w_text =
                                    word.get("word").and_then(|v| v.as_str()).unwrap_or("?");
                                let start =
                                    word.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let end = word.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let prob = word
                                    .get("probability")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0);
                                out.push_str(&format!(
                                    "[{start:.2}-{end:.2}] {w_text} (prob={prob:.2})\n"
                                ));
                            }
                        }
                    }
                } else {
                    // No structured word data — fall back to raw output
                    out.push_str(&stdout);
                }
            } else {
                // Not valid JSON — show raw output
                out.push_str(&stdout);
            }

            truncate_response(&out)
        }
        Err(e) => {
            // Some Whisper CLI versions may not support --word_timestamps
            // Try without it and note the limitation
            let p_str2 = path.to_string_lossy().to_string();
            let fallback_args: Vec<String> = vec![
                p_str2,
                "--model".to_string(),
                mdl,
                "--output_format".to_string(),
                "txt".to_string(),
            ];
            match run_command_strings(whisper, &fallback_args) {
                Ok(stdout) => {
                    let mut out = format!("SPEECH_EXTRACT_WORDS path={}\n", path.display());
                    out.push_str(
                        "Note: word-level timestamps not supported by this whisper version.\n",
                    );
                    out.push_str("Falling back to full transcription:\n\n");
                    out.push_str(&stdout);
                    truncate_response(&out)
                }
                Err(e2) => format!("Error: word extraction failed:\n{e}\n{e2}"),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_validate_whisper_model_valid() {
        assert!(validate_whisper_model("tiny").is_ok());
        assert!(validate_whisper_model("base").is_ok());
        assert!(validate_whisper_model("small").is_ok());
        assert!(validate_whisper_model("medium").is_ok());
        assert!(validate_whisper_model("large").is_ok());
    }

    #[test]
    fn test_validate_whisper_model_invalid() {
        assert!(validate_whisper_model("mega").is_err());
        assert!(validate_whisper_model("").is_err());
    }

    #[test]
    fn test_output_path_with_explicit_output() {
        let input = PathBuf::from("/tmp/test.wav");
        let output = Some(PathBuf::from("/tmp/output_enhanced.wav"));
        let result = output_path(&input, "_enhanced", Some("wav"), &output);
        assert_eq!(result, PathBuf::from("/tmp/output_enhanced.wav"));
    }

    #[test]
    fn test_output_path_derived_default() {
        let input = PathBuf::from("/tmp/test.wav");
        let result = output_path(&input, "_enhanced", None, &None);
        assert_eq!(result, PathBuf::from("/tmp/test_enhanced.wav"));
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
    fn test_cli_exists_nonexistent() {
        assert!(!cli_exists("definitely_not_a_real_program_xyzzy_99999"));
    }

    #[test]
    fn test_parse_silence_value_start() {
        let line = "[silencedetect @ 0x...] silence_start:12.345";
        assert_eq!(parse_silence_value(line, "silence_start"), Some(12.345));
    }

    #[test]
    fn test_parse_silence_value_end() {
        let line = "[silencedetect @ 0x...] silence_end:15.678 | silence_duration:3.433";
        assert_eq!(parse_silence_value(line, "silence_end"), Some(15.678));
    }

    #[test]
    fn test_parse_silence_value_no_match() {
        let line = "some other output";
        assert_eq!(parse_silence_value(line, "silence_start"), None);
    }

    #[test]
    fn test_speech_transcribe_microphone_zero_duration() {
        let result = speech_transcribe_microphone_work(0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_text_to_speech_empty_text() {
        let result = speech_text_to_speech_work("".to_string(), None, None, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_text_to_speech_zero_speed() {
        let result = speech_text_to_speech_work("hello".to_string(), None, Some(0), None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_text_to_speech_ssml_empty() {
        let result = speech_text_to_speech_ssml_work("".to_string(), None, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_diarize_zero_speakers() {
        let result = speech_diarize_work(PathBuf::from("/tmp/test.wav"), Some(0));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_enhance_invalid_noise_reduction_negative() {
        let result = speech_enhance_work(PathBuf::from("/tmp/test.wav"), Some(-0.5), None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_enhance_invalid_noise_reduction_over_one() {
        let result = speech_enhance_work(PathBuf::from("/tmp/test.wav"), Some(1.5), None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_transcribe_stream_zero_duration() {
        let result = speech_transcribe_stream_work(PathBuf::from("/tmp/test.wav"), 0.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_speech_transcribe_file_invalid_model() {
        let result = speech_transcribe_file_work(
            PathBuf::from("/tmp/test.wav"),
            None,
            Some("mega".to_string()),
        );
        assert!(result.starts_with("Error:"));
        // If whisper CLI is not on PATH, we get a different error first;
        // otherwise the model validation should mention "unknown whisper model".
        assert!(
            result.to_lowercase().contains("unknown whisper model")
                || result.to_lowercase().contains("whisper"),
            "expected unknown whisper model or whisper CLI error, got: {result}"
        );
    }
}
