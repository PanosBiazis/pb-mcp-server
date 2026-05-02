// Audio tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

#[tool(
    description = "Get audio file metadata: duration, format, bitrate, sample rate, channels. Requires ffprobe on PATH. Blocked on public profile.",
    annotations(title = "Audio info", read_only_hint = true)
)]
async fn audio_info(
    &self,
    Parameters(args): Parameters<AudioInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || audio_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an audio file to a different format (mp3, wav, ogg, flac, aac, m4a, opus, wma). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio convert", read_only_hint = false)
)]
async fn audio_convert(
    &self,
    Parameters(args): Parameters<AudioConvertArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_format = args.target_format.clone();
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_convert_work(path, target_format, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract a time segment from an audio file by start time and duration. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio extract segment", read_only_hint = false)
)]
async fn audio_extract_segment(
    &self,
    Parameters(args): Parameters<AudioExtractSegmentArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let start_secs = args.start_secs;
    let duration_secs = args.duration_secs;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || {
        audio_extract_segment_work(path, start_secs, duration_secs, output)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Trim an audio file to a start/end time range. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio trim", read_only_hint = false)
)]
async fn audio_trim(
    &self,
    Parameters(args): Parameters<AudioTrimArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let start_secs = args.start_secs;
    let end_secs = args.end_secs;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || audio_trim_work(path, start_secs, end_secs, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Concatenate multiple audio files into one output file in order. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio merge", read_only_hint = false)
)]
async fn audio_merge(
    &self,
    Parameters(args): Parameters<AudioMergeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let paths: Vec<PathBuf> = args
        .paths
        .iter()
        .map(|p| self.resolve_path(p).unwrap_or_else(|_| PathBuf::from(p)))
        .collect();
    let output = match self.resolve_path(&args.output) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || audio_merge_work(paths, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Mix (overlay) two audio files together with optional per-track volume control. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio mix", read_only_hint = false)
)]
async fn audio_mix(
    &self,
    Parameters(args): Parameters<AudioMixArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = match self.resolve_path(&args.output) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let volume_a = args.volume_a;
    let volume_b = args.volume_b;
    let out = tokio::task::spawn_blocking(move || {
        audio_mix_work(path_a, path_b, output, volume_a, volume_b)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Adjust the volume of an audio file by a multiplier (1.0 = original, 2.0 = double). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio volume", read_only_hint = false)
)]
async fn audio_volume(
    &self,
    Parameters(args): Parameters<AudioVolumeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let volume = args.volume;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_volume_work(path, volume, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Change the playback speed of an audio file (1.0 = original, 2.0 = double speed). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio speed", read_only_hint = false)
)]
async fn audio_speed(
    &self,
    Parameters(args): Parameters<AudioSpeedArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let speed = args.speed;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_speed_work(path, speed, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Reverse an audio file so it plays backwards. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio reverse", read_only_hint = false)
)]
async fn audio_reverse(
    &self,
    Parameters(args): Parameters<AudioReverseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_reverse_work(path, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Normalize audio loudness to a target dB level (e.g. -14.0 for streaming, -23.0 for broadcast). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio normalize", read_only_hint = false)
)]
async fn audio_normalize(
    &self,
    Parameters(args): Parameters<AudioNormalizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_db = args.target_db;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_normalize_work(path, target_db, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Split an audio file into equal-length segments of a given duration. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio split", read_only_hint = false)
)]
async fn audio_split(
    &self,
    Parameters(args): Parameters<AudioSplitArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let segment_duration_secs = args.segment_duration_secs;
    let output_dir = args.output_dir.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || {
        audio_split_work(path, segment_duration_secs, output_dir)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract the audio track from a video file. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio extract from video", read_only_hint = false)
)]
async fn audio_extract_from_video(
    &self,
    Parameters(args): Parameters<AudioExtractFromVideoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let video_path = match self.resolve_path(&args.video_path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || audio_extract_from_video_work(video_path, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Generate a waveform amplitude profile for an audio file, returning sample values as text. Requires ffprobe on PATH. Blocked on public profile.",
    annotations(title = "Audio waveform", read_only_hint = true)
)]
async fn audio_waveform(
    &self,
    Parameters(args): Parameters<AudioWaveformArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let samples = args.samples;
    let out = tokio::task::spawn_blocking(move || audio_waveform_work(path, samples))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Detect silent intervals in an audio file, returning timestamps and durations. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio silence detect", read_only_hint = true)
)]
async fn audio_silence_detect(
    &self,
    Parameters(args): Parameters<AudioSilenceDetectArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let min_duration = args.min_duration;
    let noise_db = args.noise_db;
    let out =
        tokio::task::spawn_blocking(move || audio_silence_detect_work(path, min_duration, noise_db))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
