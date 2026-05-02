// Speech tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ---------------------------------------------------------------------------
// 1. speech_transcribe_file
// ---------------------------------------------------------------------------

#[tool(
    description = "Transcribe an audio file to text using Whisper or another STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe file", read_only_hint = true)
)]
async fn speech_transcribe_file(
    &self,
    Parameters(args): Parameters<SpeechTranscribeFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let model = args.model.clone();
    let out =
        tokio::task::spawn_blocking(move || speech_transcribe_file_work(path, language, model))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. speech_transcribe_microphone
// ---------------------------------------------------------------------------

#[tool(
    description = "Record audio from the microphone for a given duration and transcribe it with an STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe microphone", read_only_hint = false)
)]
async fn speech_transcribe_microphone(
    &self,
    Parameters(args): Parameters<SpeechTranscribeMicrophoneArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let duration_secs = args.duration_secs;
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || {
        speech_transcribe_microphone_work(duration_secs, language)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. speech_transcribe_stream
// ---------------------------------------------------------------------------

#[tool(
    description = "Transcribe an audio file in streaming segments of a fixed duration, returning per-segment text. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe stream", read_only_hint = true)
)]
async fn speech_transcribe_stream(
    &self,
    Parameters(args): Parameters<SpeechTranscribeStreamArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let segment_duration_secs = args.segment_duration_secs;
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || {
        speech_transcribe_stream_work(path, segment_duration_secs, language)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. speech_text_to_speech
// ---------------------------------------------------------------------------

#[tool(
    description = "Convert plain text to speech audio and save to a file using a TTS engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech text to speech", read_only_hint = false)
)]
async fn speech_text_to_speech(
    &self,
    Parameters(args): Parameters<SpeechTextToSpeechArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let text = args.text.clone();
    let voice = args.voice.clone();
    let speed = args.speed;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_text_to_speech_work(text, voice, speed, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. speech_text_to_speech_ssml
// ---------------------------------------------------------------------------

#[tool(
    description = "Convert SSML markup to speech audio and save to a file using a TTS engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech text to speech SSML", read_only_hint = false)
)]
async fn speech_text_to_speech_ssml(
    &self,
    Parameters(args): Parameters<SpeechTextToSpeechSsmlArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let ssml = args.ssml.clone();
    let voice = args.voice.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_text_to_speech_ssml_work(ssml, voice, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. speech_list_voices
// ---------------------------------------------------------------------------

#[tool(
    description = "List available TTS voice identifiers supported by the configured speech engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech list voices", read_only_hint = true)
)]
async fn speech_list_voices(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(speech_list_voices_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 7. speech_list_models
// ---------------------------------------------------------------------------

#[tool(
    description = "List available STT/TTS model names supported by the configured speech engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech list models", read_only_hint = true)
)]
async fn speech_list_models(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(speech_list_models_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 8. speech_detect_language
// ---------------------------------------------------------------------------

#[tool(
    description = "Detect the spoken language in an audio file. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech detect language", read_only_hint = true)
)]
async fn speech_detect_language(
    &self,
    Parameters(args): Parameters<SpeechDetectLanguageArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || speech_detect_language_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 9. speech_translate
// ---------------------------------------------------------------------------

#[tool(
    description = "Transcribe and translate an audio file to text in a target language. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech translate", read_only_hint = true)
)]
async fn speech_translate(
    &self,
    Parameters(args): Parameters<SpeechTranslateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_language = args.target_language.clone();
    let out = tokio::task::spawn_blocking(move || speech_translate_work(path, target_language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 10. speech_diarize
// ---------------------------------------------------------------------------

#[tool(
    description = "Perform speaker diarization on an audio file, identifying and labelling each speaker turn. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech diarize", read_only_hint = true)
)]
async fn speech_diarize(
    &self,
    Parameters(args): Parameters<SpeechDiarizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let num_speakers = args.num_speakers;
    let out = tokio::task::spawn_blocking(move || speech_diarize_work(path, num_speakers))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 11. speech_enhance
// ---------------------------------------------------------------------------

#[tool(
    description = "Enhance an audio file by applying noise reduction and clarity improvements. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech enhance", read_only_hint = false)
)]
async fn speech_enhance(
    &self,
    Parameters(args): Parameters<SpeechEnhanceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let noise_reduction = args.noise_reduction;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_enhance_work(path, noise_reduction, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 12. speech_extract_words
// ---------------------------------------------------------------------------

#[tool(
    description = "Extract word-level timestamps from an audio file using an STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech extract words", read_only_hint = true)
)]
async fn speech_extract_words(
    &self,
    Parameters(args): Parameters<SpeechExtractWordsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || speech_extract_words_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
