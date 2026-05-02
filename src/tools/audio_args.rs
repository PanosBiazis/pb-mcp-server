// Audio tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioInfoArgs {
    /// Path to the audio file relative to project root.
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioConvertArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Target format: mp3, wav, ogg, flac, aac, m4a, opus, wma.
    target_format: String,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioExtractSegmentArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Start time in seconds (must be >= 0).
    start_secs: f64,
    /// Duration of the segment to extract in seconds (must be > 0).
    duration_secs: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioTrimArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Start time in seconds (must be >= 0).
    start_secs: f64,
    /// End time in seconds (must be > start_secs).
    end_secs: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioMergeArgs {
    /// Ordered list of audio file paths to merge (relative to project root). Minimum 2.
    paths: Vec<String>,
    /// Output path for the merged file (relative to project root).
    output: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioMixArgs {
    /// Path to the first audio file relative to project root.
    path_a: String,
    /// Path to the second audio file relative to project root.
    path_b: String,
    /// Output path for the mixed file (relative to project root).
    output: String,
    /// Volume multiplier for the first track (default 1.0 = original).
    #[serde(default)]
    volume_a: Option<f64>,
    /// Volume multiplier for the second track (default 1.0 = original).
    #[serde(default)]
    volume_b: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioVolumeArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Volume multiplier: 1.0 = original, 0.5 = half, 2.0 = double.
    volume: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioSpeedArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Speed multiplier: 1.0 = original, 2.0 = double speed, 0.5 = half speed.
    speed: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioReverseArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioNormalizeArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Target loudness in dB (e.g. -14.0 for streaming, -23.0 for broadcast).
    target_db: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioSplitArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Duration of each output segment in seconds (must be > 0).
    segment_duration_secs: f64,
    /// Output directory relative to project root (optional, defaults to same directory as input).
    #[serde(default)]
    output_dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioExtractFromVideoArgs {
    /// Path to the video file relative to project root.
    video_path: String,
    /// Output path for the extracted audio (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioWaveformArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Number of amplitude samples to return in the waveform (optional).
    #[serde(default)]
    samples: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AudioSilenceDetectArgs {
    /// Path to the audio file relative to project root.
    path: String,
    /// Minimum silence duration in seconds to detect (optional, default ~2.0 s).
    #[serde(default)]
    min_duration: Option<f64>,
    /// Noise threshold in dB below which audio is considered silent (optional, default -30 dB).
    #[serde(default)]
    noise_db: Option<f64>,
}
