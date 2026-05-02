// Speech tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ---------------------------------------------------------------------------
// 1. speech_transcribe_file
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTranscribeFileArgs {
    /// Path to the audio file to transcribe, relative to the project root.
    path: String,
    /// Language hint for the STT engine (e.g. "en", "fr"). Omit for auto-detection.
    #[serde(default)]
    language: Option<String>,
    /// STT model name (e.g. "whisper-base", "whisper-large-v3"). Omit for default.
    #[serde(default)]
    model: Option<String>,
}

// ---------------------------------------------------------------------------
// 2. speech_transcribe_microphone
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTranscribeMicrophoneArgs {
    /// Duration to record from the microphone, in seconds.
    duration_secs: u64,
    /// Language hint for the STT engine (e.g. "en", "fr"). Omit for auto-detection.
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 3. speech_transcribe_stream
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTranscribeStreamArgs {
    /// Path to the audio file to stream-transcribe, relative to the project root.
    path: String,
    /// Duration of each streaming segment in seconds (e.g. 5.0).
    segment_duration_secs: f64,
    /// Language hint for the STT engine (e.g. "en", "fr"). Omit for auto-detection.
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 4. speech_text_to_speech
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTextToSpeechArgs {
    /// Text to convert to speech.
    text: String,
    /// Voice name or identifier (e.g. "en-US-Wavenet-A"). Omit for default.
    #[serde(default)]
    voice: Option<String>,
    /// Speech speed as a percentage (100 = normal, 150 = 1.5x). Omit for default.
    #[serde(default)]
    speed: Option<u32>,
    /// Output audio file path relative to the project root. Omit to auto-generate.
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 5. speech_text_to_speech_ssml
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTextToSpeechSsmlArgs {
    /// SSML markup string to synthesize into speech.
    ssml: String,
    /// Voice name or identifier. Omit for default.
    #[serde(default)]
    voice: Option<String>,
    /// Output audio file path relative to the project root. Omit to auto-generate.
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 6. speech_list_voices  — uses EmptyArgs (defined in mcp_args.rs)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 7. speech_list_models  — uses EmptyArgs (defined in mcp_args.rs)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 8. speech_detect_language
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechDetectLanguageArgs {
    /// Path to the audio file whose spoken language is to be detected, relative to the project root.
    path: String,
}

// ---------------------------------------------------------------------------
// 9. speech_translate
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechTranslateArgs {
    /// Path to the audio file to transcribe and translate, relative to the project root.
    path: String,
    /// Target translation language code (e.g. "en", "de"). Omit for default (usually English).
    #[serde(default)]
    target_language: Option<String>,
}

// ---------------------------------------------------------------------------
// 10. speech_diarize
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechDiarizeArgs {
    /// Path to the audio file to diarize (identify and separate speakers), relative to the project root.
    path: String,
    /// Expected number of speakers. Omit for automatic detection.
    #[serde(default)]
    num_speakers: Option<usize>,
}

// ---------------------------------------------------------------------------
// 11. speech_enhance
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechEnhanceArgs {
    /// Path to the audio file to enhance (noise reduction, clarity), relative to the project root.
    path: String,
    /// Noise reduction strength in the range 0.0 (none) to 1.0 (maximum). Omit for default.
    #[serde(default)]
    noise_reduction: Option<f64>,
    /// Output file path relative to the project root. Omit to auto-derive from the input path.
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 12. speech_extract_words
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SpeechExtractWordsArgs {
    /// Path to the audio file from which to extract word-level timestamps, relative to the project root.
    path: String,
    /// Language hint for the STT engine (e.g. "en", "fr"). Omit for auto-detection.
    #[serde(default)]
    language: Option<String>,
}
