// Image tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageInfoArgs {
    /// Path to the image file relative to project root.
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageResizeArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Target width in pixels (must be > 0).
    width: u32,
    /// Target height in pixels (must be > 0).
    height: u32,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageConvertArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Target format: png, jpg, jpeg, webp, bmp, ico, tiff.
    target_format: String,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageGrayscaleArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageCropArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// X offset (left edge) of the crop region in pixels.
    x: u32,
    /// Y offset (top edge) of the crop region in pixels.
    y: u32,
    /// Width of the crop region in pixels (must be > 0).
    width: u32,
    /// Height of the crop region in pixels (must be > 0).
    height: u32,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageRotateArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Degrees to rotate clockwise (e.g. 90.0, 180.0, 270.0; arbitrary angles use nearest neighbour).
    degrees: f64,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageFlipArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// If true, flip horizontally (mirror left-right); if false, flip vertically (top-bottom).
    horizontal: bool,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageThumbnailArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Maximum side length in pixels; aspect ratio is preserved.
    size: u32,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageHistogramArgs {
    /// Path to the image file relative to project root.
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageCompareArgs {
    /// Path to the first image file relative to project root.
    path_a: String,
    /// Path to the second image file relative to project root.
    path_b: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageExtractExifArgs {
    /// Path to the image file relative to project root.
    path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImageWatermarkArgs {
    /// Path to the image file relative to project root.
    path: String,
    /// Watermark text to overlay on the image.
    text: String,
    /// Position: topleft, topright, bottomleft, bottomright, or center.
    position: String,
    /// Output path relative to project root (optional, auto-derived if omitted).
    #[serde(default)]
    output: Option<String>,
}
