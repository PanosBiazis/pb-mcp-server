// Argument structs for extended OCR tools (ocr_handlers.rs).
// include!()-d at module scope in main.rs — no use statements here.

// ---------------------------------------------------------------------------
// 1. ocr_file
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrFileArgs {
    #[schemars(
        description = "Path to the image file to OCR (png, jpg, tiff, webp, bmp, …) under the project root."
    )]
    path: String,
    #[schemars(
        description = "Tesseract language code (e.g. 'eng', 'deu', 'fra'). Defaults to 'eng'."
    )]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 2. ocr_pdf
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrPdfArgs {
    #[schemars(description = "Path to the PDF file under the project root.")]
    path: String,
    #[schemars(description = "Tesseract language code (e.g. 'eng', 'deu'). Defaults to 'eng'.")]
    #[serde(default)]
    language: Option<String>,
    #[schemars(
        description = "Optional page range (e.g. '1-5', '3', '2-'). Omit to process all pages."
    )]
    #[serde(default)]
    page_range: Option<String>,
}

// ---------------------------------------------------------------------------
// 3. ocr_batch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrBatchArgs {
    #[schemars(description = "List of image file paths under the project root to OCR in batch.")]
    paths: Vec<String>,
    #[schemars(
        description = "Tesseract language code applied to all files (e.g. 'eng'). Defaults to 'eng'."
    )]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 4. ocr_from_clipboard  — uses EmptyArgs (defined in mcp_args.rs)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 5. ocr_screen_region
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrScreenRegionArgs {
    #[schemars(description = "X coordinate of the top-left corner of the screen region.")]
    x: u32,
    #[schemars(description = "Y coordinate of the top-left corner of the screen region.")]
    y: u32,
    #[schemars(description = "Width of the region in pixels.")]
    width: u32,
    #[schemars(description = "Height of the region in pixels.")]
    height: u32,
    #[schemars(description = "Tesseract language code (e.g. 'eng', 'deu'). Defaults to 'eng'.")]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 6. ocr_table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrTableArgs {
    #[schemars(
        description = "Path to the image containing a table to OCR, under the project root."
    )]
    path: String,
    #[schemars(description = "Tesseract language code (e.g. 'eng', 'deu'). Defaults to 'eng'.")]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 7. ocr_with_confidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrWithConfidenceArgs {
    #[schemars(
        description = "Path to the image file to OCR with per-word confidence scores, under the project root."
    )]
    path: String,
    #[schemars(description = "Tesseract language code (e.g. 'eng', 'deu'). Defaults to 'eng'.")]
    #[serde(default)]
    language: Option<String>,
}

// ---------------------------------------------------------------------------
// 8. ocr_preprocess
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrPreprocessArgs {
    #[schemars(description = "Path to the image file to preprocess, under the project root.")]
    path: String,
    #[schemars(
        description = "Preprocessing operation: grayscale | threshold | despeckle | sharpen | resize2x."
    )]
    operation: String,
    #[schemars(
        description = "Optional output file path. Defaults to the source path with an operation-specific suffix."
    )]
    #[serde(default)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// 9. ocr_detect_text_regions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrDetectTextRegionsArgs {
    #[schemars(
        description = "Path to the image file to detect text regions in, under the project root."
    )]
    path: String,
}

// ---------------------------------------------------------------------------
// 10. ocr_list_languages  — uses EmptyArgs (defined in mcp_args.rs)
// ---------------------------------------------------------------------------
