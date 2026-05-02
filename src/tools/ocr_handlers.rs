// Handlers for extended OCR tools.
// include!()-d inside `impl MintMcp { … }` in main.rs.
// No mod/impl/use here — all identifiers must be in scope from main.rs.

// ---------------------------------------------------------------------------
// 1. ocr_file
// ---------------------------------------------------------------------------

#[tool(
    description = "OCR an image file (png, jpg, tiff, webp, bmp, …) using Tesseract. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR file", read_only_hint = true)
)]
async fn ocr_file(
    &self,
    Parameters(args): Parameters<OcrFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_file_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. ocr_pdf
// ---------------------------------------------------------------------------

#[tool(
    description = "OCR a PDF document page-by-page using Tesseract, via `pdftoppm` or ImageMagick for page rendering. Supports an optional page range. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR PDF", read_only_hint = true)
)]
async fn ocr_pdf(
    &self,
    Parameters(args): Parameters<OcrPdfArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let page_range = args.page_range.clone();
    let out = tokio::task::spawn_blocking(move || ocr_pdf_work(path, language, page_range))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. ocr_batch
// ---------------------------------------------------------------------------

#[tool(
    description = "OCR multiple image files in batch using Tesseract. Returns per-file results combined into one response. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR batch", read_only_hint = true)
)]
async fn ocr_batch(
    &self,
    Parameters(args): Parameters<OcrBatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let paths: Vec<PathBuf> = args
        .paths
        .iter()
        .map(|p| self.resolve_path(p).unwrap_or_else(|_| PathBuf::from(p)))
        .collect();
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_batch_work(paths, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. ocr_from_clipboard
// ---------------------------------------------------------------------------

#[tool(
    description = "Capture an image from the clipboard and OCR it using Tesseract. Requires `xclip` on Linux. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR from clipboard", read_only_hint = true)
)]
async fn ocr_from_clipboard(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(ocr_from_clipboard_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. ocr_screen_region
// ---------------------------------------------------------------------------

#[tool(
    description = "Capture a screen region with `scrot` or ImageMagick `import` and OCR it immediately with Tesseract. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR screen region", read_only_hint = true)
)]
async fn ocr_screen_region(
    &self,
    Parameters(args): Parameters<OcrScreenRegionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let language = args.language.clone();
    let out =
        tokio::task::spawn_blocking(move || ocr_screen_region_work(x, y, width, height, language))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. ocr_table
// ---------------------------------------------------------------------------

#[tool(
    description = "OCR an image that contains a table and reconstruct column alignment using Tesseract TSV output. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR table", read_only_hint = true)
)]
async fn ocr_table(
    &self,
    Parameters(args): Parameters<OcrTableArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_table_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 7. ocr_with_confidence
// ---------------------------------------------------------------------------

#[tool(
    description = "OCR an image and return per-word confidence scores alongside the recognised text. Uses Tesseract TSV output. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR with confidence", read_only_hint = true)
)]
async fn ocr_with_confidence(
    &self,
    Parameters(args): Parameters<OcrWithConfidenceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_with_confidence_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 8. ocr_preprocess
// ---------------------------------------------------------------------------

#[tool(
    description = "Preprocess an image with ImageMagick before OCR. Operations: grayscale | threshold | despeckle | sharpen | resize2x. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR preprocess image", read_only_hint = true)
)]
async fn ocr_preprocess(
    &self,
    Parameters(args): Parameters<OcrPreprocessArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let operation = args.operation.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || ocr_preprocess_work(path, operation, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 9. ocr_detect_text_regions
// ---------------------------------------------------------------------------

#[tool(
    description = "Detect text bounding-box regions in an image using Tesseract's makebox mode. Returns region coordinates for each detected character/word cluster. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR detect text regions", read_only_hint = true)
)]
async fn ocr_detect_text_regions(
    &self,
    Parameters(args): Parameters<OcrDetectTextRegionsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || ocr_detect_text_regions_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 10. ocr_list_languages
// ---------------------------------------------------------------------------

#[tool(
    description = "List all Tesseract language packs installed on the system. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR list languages", read_only_hint = true)
)]
async fn ocr_list_languages(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(ocr_list_languages_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
