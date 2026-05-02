// Argument structs for extended screenshot tools (screenshot_handlers.rs).
// include!()-d at module scope in main.rs — no use statements here.

// ---------------------------------------------------------------------------
// 1. screenshot_region
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotRegionArgs {
    #[schemars(description = "X coordinate of the top-left corner of the region.")]
    x: u32,
    #[schemars(description = "Y coordinate of the top-left corner of the region.")]
    y: u32,
    #[schemars(description = "Width of the region in pixels.")]
    width: u32,
    #[schemars(description = "Height of the region in pixels.")]
    height: u32,
    #[schemars(
        description = "Optional output file path (defaults to a timestamped .png in .mint-mcp/screenshots/)."
    )]
    #[serde(default)]
    output: Option<String>,
    #[schemars(description = "Optional X11 display string (e.g. ':0'). Defaults to ':0'.")]
    #[serde(default)]
    display: Option<String>,
}

// ---------------------------------------------------------------------------
// 2. screenshot_window
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotWindowArgs {
    #[schemars(
        description = "Optional substring to match against window titles. Captures the active window if omitted."
    )]
    #[serde(default)]
    window_name: Option<String>,
    #[schemars(
        description = "Optional output file path (defaults to a timestamped .png in .mint-mcp/screenshots/)."
    )]
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 3. screenshot_ocr
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotOcrArgs {
    #[schemars(
        description = "Optional path to save the captured screenshot before running OCR. If omitted the screenshot is discarded after OCR."
    )]
    #[serde(default)]
    output: Option<String>,
    #[schemars(
        description = "Tesseract language code (e.g. 'eng', 'deu'). Defaults to 'eng'."
    )]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 4. screenshot_record
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotRecordArgs {
    #[schemars(description = "Recording duration in seconds (capped at 600 s).")]
    duration_secs: u64,
    #[schemars(
        description = "Optional output file path (defaults to a timestamped .mp4 in .mint-mcp/screenshots/)."
    )]
    #[serde(default)]
    output: Option<String>,
    #[schemars(description = "Frames per second for the recording (default 15, clamped 1–120).")]
    #[serde(default)]
    fps: Option<u32>,
}

// ---------------------------------------------------------------------------
// 5. screenshot_annotate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotAnnotateArgs {
    #[schemars(description = "Path to the source screenshot image under the project root.")]
    path: String,
    #[schemars(
        description = "JSON array of annotation objects: [{\"text\": \"label\", \"x\": 10, \"y\": 20, \"w\": 100, \"h\": 30}]"
    )]
    annotations_json: String,
    #[schemars(
        description = "Optional output file path (defaults to the source path with an _annotated suffix)."
    )]
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 6. screenshot_compare
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotCompareArgs {
    #[schemars(description = "Path to the first screenshot image under the project root.")]
    path_a: String,
    #[schemars(description = "Path to the second screenshot image under the project root.")]
    path_b: String,
    #[schemars(
        description = "MAE threshold 0.0–1.0 (default 0.01). Images with MAE at or below this are considered identical."
    )]
    #[serde(default)]
    threshold: Option<f64>,
}

// ---------------------------------------------------------------------------
// 7. screenshot_monitor_info  — uses EmptyArgs (defined in mcp_args.rs)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 8. screenshot_color_pick
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotColorPickArgs {
    #[schemars(description = "X coordinate of the pixel to sample.")]
    x: u32,
    #[schemars(description = "Y coordinate of the pixel to sample.")]
    y: u32,
    #[schemars(description = "Optional X11 display string (e.g. ':0'). Defaults to ':0'.")]
    #[serde(default)]
    display: Option<String>,
}

// ---------------------------------------------------------------------------
// 9. screenshot_timelapse
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotTimelapseArgs {
    #[schemars(description = "Interval between screenshots in seconds.")]
    interval_secs: u64,
    #[schemars(description = "Total number of screenshots to take.")]
    count: u32,
    #[schemars(
        description = "Optional output directory for timelapse frame images (defaults to .mint-mcp/screenshots/timelapse/)."
    )]
    #[serde(default)]
    output_dir: Option<String>,
}
