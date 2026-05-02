// Handlers for extended screenshot tools.
// include!()-d inside `impl MintMcp { … }` in main.rs.
// No mod/impl/use here — all identifiers must be in scope from main.rs.

// ---------------------------------------------------------------------------
// 1. screenshot_region
// ---------------------------------------------------------------------------

#[tool(
    description = "Capture a specific screen region to a file using `scrot` or ImageMagick `import`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot region capture", read_only_hint = false)
)]
async fn screenshot_region(
    &self,
    Parameters(args): Parameters<ScreenshotRegionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let display = args.display.clone();
    let out = tokio::task::spawn_blocking(move || {
        screenshot_region_work(x, y, width, height, output, display)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. screenshot_window
// ---------------------------------------------------------------------------

#[tool(
    description = "Capture a specific window by name using `xdotool` + `import`/`scrot`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot window capture", read_only_hint = false)
)]
async fn screenshot_window(
    &self,
    Parameters(args): Parameters<ScreenshotWindowArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let window_name = args.window_name.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || screenshot_window_work(window_name, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. screenshot_ocr
// ---------------------------------------------------------------------------

#[tool(
    description = "Take a full-screen screenshot and OCR it with Tesseract in a single step. Optionally saves the screenshot. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot + OCR", read_only_hint = true)
)]
async fn screenshot_ocr(
    &self,
    Parameters(args): Parameters<ScreenshotOcrArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || screenshot_ocr_work(output, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. screenshot_record
// ---------------------------------------------------------------------------

#[tool(
    description = "Record a screen video (screencast) using `ffmpeg` with `x11grab`. Duration is capped at 600 s. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot screen record", read_only_hint = false)
)]
async fn screenshot_record(
    &self,
    Parameters(args): Parameters<ScreenshotRecordArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let duration_secs = args.duration_secs;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let fps = args.fps;
    let out =
        tokio::task::spawn_blocking(move || screenshot_record_work(duration_secs, output, fps))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. screenshot_annotate
// ---------------------------------------------------------------------------

#[tool(
    description = "Add labelled rectangle annotations to a screenshot using ImageMagick. Each annotation specifies text, x, y, width, and height. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot annotate", read_only_hint = false)
)]
async fn screenshot_annotate(
    &self,
    Parameters(args): Parameters<ScreenshotAnnotateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let annotations: Vec<(String, u32, u32, u32, u32)> =
        serde_json::from_str::<Vec<serde_json::Value>>(&args.annotations_json)
            .map(|v| {
                v.iter()
                    .map(|a| {
                        (
                            a["text"].as_str().unwrap_or("").to_string(),
                            a["x"].as_u64().unwrap_or(0) as u32,
                            a["y"].as_u64().unwrap_or(0) as u32,
                            a["w"].as_u64().unwrap_or(100) as u32,
                            a["h"].as_u64().unwrap_or(30) as u32,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || screenshot_annotate_work(path, annotations, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. screenshot_compare
// ---------------------------------------------------------------------------

#[tool(
    description = "Compare two screenshots using ImageMagick's MAE metric and return a match/different verdict. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot compare", read_only_hint = true)
)]
async fn screenshot_compare(
    &self,
    Parameters(args): Parameters<ScreenshotCompareArgs>,
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
    let threshold = args.threshold;
    let out =
        tokio::task::spawn_blocking(move || screenshot_compare_work(path_a, path_b, threshold))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 7. screenshot_monitor_info
// ---------------------------------------------------------------------------

#[tool(
    description = "List available monitors and display resolutions using `xrandr`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot monitor info", read_only_hint = true)
)]
async fn screenshot_monitor_info(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(screenshot_monitor_info_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 8. screenshot_color_pick
// ---------------------------------------------------------------------------

#[tool(
    description = "Sample the color of a single pixel at (x, y) on screen using ImageMagick. Returns hex color and RGB values. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot color pick", read_only_hint = true)
)]
async fn screenshot_color_pick(
    &self,
    Parameters(args): Parameters<ScreenshotColorPickArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let display = args.display.clone();
    let out = tokio::task::spawn_blocking(move || screenshot_color_pick_work(x, y, display))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 9. screenshot_timelapse
// ---------------------------------------------------------------------------

#[tool(
    description = "Capture a series of screenshots at a fixed interval to create a timelapse sequence. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot timelapse", read_only_hint = false)
)]
async fn screenshot_timelapse(
    &self,
    Parameters(args): Parameters<ScreenshotTimelapseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let interval_secs = args.interval_secs;
    let count = args.count;
    let output_dir = args.output_dir.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || {
        screenshot_timelapse_work(interval_secs, count, output_dir)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
