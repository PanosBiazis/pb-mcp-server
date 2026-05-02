// Image tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

#[tool(
    description = "Read image metadata: dimensions, format, file size, and color depth. Uses the image crate when available, falls back to ImageMagick/file commands. Blocked on public profile.",
    annotations(title = "Image info", read_only_hint = true)
)]
async fn image_info(
    &self,
    Parameters(args): Parameters<ImageInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Resize an image to exact width x height pixels. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image resize", read_only_hint = false)
)]
async fn image_resize(
    &self,
    Parameters(args): Parameters<ImageResizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_resize_work(path, width, height, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an image to a different format (png, jpg, jpeg, webp, bmp, ico, tiff). Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image convert", read_only_hint = false)
)]
async fn image_convert(
    &self,
    Parameters(args): Parameters<ImageConvertArgs>,
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
    let out =
        tokio::task::spawn_blocking(move || image_convert_work(path, target_format, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an image to grayscale. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image grayscale", read_only_hint = false)
)]
async fn image_grayscale(
    &self,
    Parameters(args): Parameters<ImageGrayscaleArgs>,
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
    let out = tokio::task::spawn_blocking(move || image_grayscale_work(path, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Crop a rectangular region from an image at offset (x, y) with the given width and height. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image crop", read_only_hint = false)
)]
async fn image_crop(
    &self,
    Parameters(args): Parameters<ImageCropArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || image_crop_work(path, x, y, width, height, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Rotate an image by the specified number of degrees clockwise. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image rotate", read_only_hint = false)
)]
async fn image_rotate(
    &self,
    Parameters(args): Parameters<ImageRotateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let degrees = args.degrees;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_rotate_work(path, degrees, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Flip an image horizontally (mirror) or vertically. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image flip", read_only_hint = false)
)]
async fn image_flip(
    &self,
    Parameters(args): Parameters<ImageFlipArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let horizontal = args.horizontal;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_flip_work(path, horizontal, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Create a thumbnail with a maximum side length in pixels while preserving the aspect ratio. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image thumbnail", read_only_hint = false)
)]
async fn image_thumbnail(
    &self,
    Parameters(args): Parameters<ImageThumbnailArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let size = args.size;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_thumbnail_work(path, size, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compute a colour histogram for an image, returning per-channel bucket counts as text. Blocked on public profile.",
    annotations(title = "Image histogram", read_only_hint = true)
)]
async fn image_histogram(
    &self,
    Parameters(args): Parameters<ImageHistogramArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_histogram_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compare two images and report a similarity score with per-channel difference metrics. Blocked on public profile.",
    annotations(title = "Image compare", read_only_hint = true)
)]
async fn image_compare(
    &self,
    Parameters(args): Parameters<ImageCompareArgs>,
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
    let out = tokio::task::spawn_blocking(move || image_compare_work(path_a, path_b))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract EXIF / XMP metadata from an image file. Uses exiftool when available, falls back to basic format detection. Blocked on public profile.",
    annotations(title = "Image extract EXIF", read_only_hint = true)
)]
async fn image_extract_exif(
    &self,
    Parameters(args): Parameters<ImageExtractExifArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_extract_exif_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Overlay text as a watermark on an image at a given position (topleft, topright, bottomleft, bottomright, center). Uses ImageMagick. Blocked on public profile.",
    annotations(title = "Image watermark", read_only_hint = false)
)]
async fn image_watermark(
    &self,
    Parameters(args): Parameters<ImageWatermarkArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let text = args.text.clone();
    let position = args.position.clone();
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || image_watermark_work(path, text, position, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
