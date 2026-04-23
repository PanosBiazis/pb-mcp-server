//! # Image Processing Tools
//!
//! Comprehensive image manipulation, analysis, and conversion tools for AI agents.
//!
//! When the `ocr-tools` or `screenshot-tools` feature is enabled, the `image` crate
//! is used directly for processing. Otherwise, subprocess calls to ImageMagick
//! (`convert`/`magick`), `ffmpeg`, `file`, and `exiftool` are used as fallbacks.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum input image size (50 MiB).
const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;

/// Supported output formats for conversion.
const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "ico", "tiff"];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Check that the path exists, is a file, and is under the size limit.
/// Returns `Ok(metadata)` or an error string starting with `"Error:"`.
fn validate_image_path(path: &Path) -> Result<fs::Metadata, String> {
    if !path.is_file() {
        return Err(format!("Error: not a file: {}", path.display()));
    }
    let meta =
        fs::metadata(path).map_err(|e| format!("Error: cannot stat {}: {e}", path.display()))?;
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "Error: file too large ({} bytes, max {} MiB)",
            meta.len(),
            MAX_IMAGE_BYTES / (1024 * 1024)
        ));
    }
    Ok(meta)
}

/// Determine the output path: if `output` is `Some`, use it; otherwise derive
/// from `input` by appending a suffix before the extension (or replacing it).
fn output_path(
    input: &Path,
    suffix: &str,
    new_ext: Option<&str>,
    output: &Option<PathBuf>,
) -> PathBuf {
    match output {
        Some(p) => p.clone(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let ext = new_ext
                .unwrap_or_else(|| input.extension().and_then(|e| e.to_str()).unwrap_or("png"));
            input
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{stem}{suffix}.{ext}"))
        }
    }
}

/// Try to find an available ImageMagick binary (`magick` first, then `convert`).
/// Returns the program name or an error string.
fn find_imagemagick() -> Result<&'static str, String> {
    // Try `magick` (ImageMagick 7) first, then `convert` (ImageMagick 6).
    for cmd in &["magick", "convert"] {
        if Command::new(cmd).arg("--version").output().is_ok() {
            return Ok(cmd);
        }
    }
    Err("Error: neither `magick` nor `convert` (ImageMagick) found on PATH".to_string())
}

/// Check whether a given CLI program exists on PATH by running `<program> --version`.
fn cli_exists(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

/// Run a command and return its stdout, or an error string on failure.
fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return Err(format!(
                    "Error: {} exit={:?}\nSTDERR:\n{se}",
                    cmd,
                    o.status.code()
                ));
            }
            Ok(stdout.trim().to_string())
        }
        Err(e) => Err(format!("Error: spawn {cmd}: {e}")),
    }
}

/// Format file size in a human-readable way.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.2} GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KiB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

// ---------------------------------------------------------------------------
// 1. image_info
// ---------------------------------------------------------------------------

/// Read image metadata (dimensions, format, file size, color depth).
///
/// Uses the `image` crate when the `ocr-tools` feature is enabled.
/// Falls back to `file` and `identify` (ImageMagick) subprocess calls otherwise.
pub fn image_info_work(path: PathBuf) -> String {
    let p = &path;
    let meta = match validate_image_path(p) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let mut out = format!("IMAGE_INFO path={}\n", p.display());
    out.push_str(&format!("file_size={}\n", format_size(meta.len())));

    #[cfg(feature = "ocr-tools")]
    {
        match image::io::Reader::open(p) {
            Ok(reader) => match reader.into_dimensions() {
                Ok((w, h)) => {
                    out.push_str(&format!("width={w}\nheight={h}\n"));
                }
                Err(e) => {
                    out.push_str(&format!("dimensions=unknown ({e})\n"));
                }
            },
            Err(e) => return format!("Error: cannot open image with `image` crate: {e}"),
        }
        // Try to detect format via extension/content
        match image::io::Reader::open(p) {
            Ok(reader) => {
                let fmt = reader.format();
                out.push_str(&format!(
                    "format={}\n",
                    fmt.map(|f| f.extensions_str().join(","))
                        .unwrap_or_else(|| "unknown".to_string())
                ));
            }
            Err(_) => out.push_str("format=unknown\n"),
        }
        // Color depth estimate: read a pixel to get color type
        match image::open(p) {
            Ok(img) => {
                let color_type = match &img {
                    image::DynamicImage::ImageLuma8(_) => "Luma8 (8-bit grayscale)",
                    image::DynamicImage::ImageLumaA8(_) => "LumaA8 (8-bit grayscale+alpha)",
                    image::DynamicImage::ImageRgb8(_) => "Rgb8 (8-bit RGB, 24-bit color)",
                    image::DynamicImage::ImageRgba8(_) => "Rgba8 (8-bit RGBA, 32-bit color)",
                    image::DynamicImage::ImageLuma16(_) => "Luma16 (16-bit grayscale)",
                    image::DynamicImage::ImageLumaA16(_) => "LumaA16 (16-bit grayscale+alpha)",
                    image::DynamicImage::ImageRgb16(_) => "Rgb16 (16-bit RGB, 48-bit color)",
                    image::DynamicImage::ImageRgba16(_) => "Rgba16 (16-bit RGBA, 64-bit color)",
                    image::DynamicImage::ImageRgb32F(_) => "Rgb32F (32-bit float RGB)",
                    image::DynamicImage::ImageRgba32F(_) => "Rgba32F (32-bit float RGBA)",
                    _ => "unknown",
                };
                out.push_str(&format!("color_type={color_type}\n"));
            }
            Err(_) => out.push_str("color_type=unknown\n"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        // Use `file` command for format/encoding info
        if let Ok(info) = run_command("file", &["-b", &p.to_string_lossy()]) {
            out.push_str(&format!("file_type={info}\n"));
        }
        // Use `identify` for dimensions and depth
        if cli_exists("identify") {
            if let Ok(id) = run_command(
                "identify",
                &["-format", "%wx%h %m %depth", &p.to_string_lossy()],
            ) {
                // identify format: "WIDTHxHEIGHT FORMAT DEPTH"
                let parts: Vec<&str> = id.split_whitespace().collect();
                if parts.len() >= 3 {
                    let dims = parts[0];
                    let fmt = parts[1];
                    let depth = parts[2];
                    if let Some((w, h)) = dims.split_once('x') {
                        out.push_str(&format!("width={w}\nheight={h}\n"));
                    }
                    out.push_str(&format!("format={fmt}\ncolor_depth={depth}\n"));
                } else {
                    out.push_str(&format!("identify_raw={id}\n"));
                }
            }
        } else {
            out.push_str("color_depth=unknown (identify not available)\n");
        }
    }

    let (truncated, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    truncated
}

// ---------------------------------------------------------------------------
// 2. image_resize
// ---------------------------------------------------------------------------

/// Resize an image to exact `width` × `height` pixels.
///
/// Uses the `image` crate when `ocr-tools` is enabled, otherwise shells out to
/// ImageMagick `convert`/`magick`.
pub fn image_resize_work(
    path: PathBuf,
    width: u32,
    height: u32,
    output: Option<PathBuf>,
) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    if width == 0 || height == 0 {
        return "Error: width and height must be > 0".to_string();
    }
    let out_path = output_path(p, "_resized", None, &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let resized =
                    img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
                match resized.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_RESIZE ok\ninput={}\noutput={}\nnew_size={width}x{height}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save resized image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        let resize_spec = format!("{width}x{height}");
        match run_command(
            im,
            &[
                &p.to_string_lossy(),
                "-resize",
                &resize_spec,
                &out_path.to_string_lossy(),
            ],
        ) {
            Ok(_) => format!(
                "IMAGE_RESIZE ok\ninput={}\noutput={}\nnew_size={width}x{height}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 3. image_convert
// ---------------------------------------------------------------------------

/// Convert an image to a different format (png, jpg, webp, bmp, ico, tiff).
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_convert_work(path: PathBuf, target_format: String, output: Option<PathBuf>) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    let fmt = target_format.trim().to_ascii_lowercase();
    if !SUPPORTED_FORMATS.contains(&fmt.as_str()) {
        return format!(
            "Error: unsupported target format '{fmt}' (allowed: {})",
            SUPPORTED_FORMATS.join(", ")
        );
    }
    let out_path = output_path(p, "_converted", Some(&fmt), &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let save_result = match fmt.as_str() {
                    "png" => img.save_with_format(&out_path, image::ImageFormat::Png),
                    "jpg" | "jpeg" => img.save_with_format(&out_path, image::ImageFormat::Jpeg),
                    "webp" => img.save_with_format(&out_path, image::ImageFormat::WebP),
                    "bmp" => img.save_with_format(&out_path, image::ImageFormat::Bmp),
                    "ico" => img.save_with_format(&out_path, image::ImageFormat::Ico),
                    "tiff" => img.save_with_format(&out_path, image::ImageFormat::Tiff),
                    other => return format!("Error: unsupported format '{other}'"),
                };
                match save_result {
                    Ok(()) => format!(
                        "IMAGE_CONVERT ok\ninput={}\noutput={}\ntarget_format={fmt}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save converted image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        match run_command(im, &[&p.to_string_lossy(), &out_path.to_string_lossy()]) {
            Ok(_) => format!(
                "IMAGE_CONVERT ok\ninput={}\noutput={}\ntarget_format={fmt}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 4. image_grayscale
// ---------------------------------------------------------------------------

/// Convert an image to grayscale.
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_grayscale_work(path: PathBuf, output: Option<PathBuf>) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    let out_path = output_path(p, "_gray", None, &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let gray = img.grayscale();
                match gray.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_GRAYSCALE ok\ninput={}\noutput={}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save grayscale image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        match run_command(
            im,
            &[
                &p.to_string_lossy(),
                "-colorspace",
                "Gray",
                &out_path.to_string_lossy(),
            ],
        ) {
            Ok(_) => format!(
                "IMAGE_GRAYSCALE ok\ninput={}\noutput={}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 5. image_crop
// ---------------------------------------------------------------------------

/// Crop a rectangular region from an image starting at `(x, y)` with the given
/// `width` and `height`.
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_crop_work(
    path: PathBuf,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    output: Option<PathBuf>,
) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    if width == 0 || height == 0 {
        return "Error: crop width and height must be > 0".to_string();
    }
    let out_path = output_path(p, "_cropped", None, &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let iw = img.width();
                let ih = img.height();
                if x >= iw || y >= ih {
                    return format!(
                        "Error: crop origin ({x},{y}) is outside image bounds ({iw}x{ih})"
                    );
                }
                let crop_w = width.min(iw - x);
                let crop_h = height.min(ih - y);
                let cropped = img.crop_imm(x, y, crop_w, crop_h);
                match cropped.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_CROP ok\ninput={}\noutput={}\nregion=x={x},y={y},w={crop_w},h={crop_h}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save cropped image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        // ImageMagick crop geometry: WxH+X+Y
        let geometry = format!("{width}x{height}+{x}+{y}");
        match run_command(
            im,
            &[
                &p.to_string_lossy(),
                "-crop",
                &geometry,
                "+repage",
                &out_path.to_string_lossy(),
            ],
        ) {
            Ok(_) => format!(
                "IMAGE_CROP ok\ninput={}\noutput={}\nregion=x={x},y={y},w={width},h={height}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 6. image_rotate
// ---------------------------------------------------------------------------

/// Rotate an image by the specified number of degrees (clockwise).
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_rotate_work(path: PathBuf, degrees: f64, output: Option<PathBuf>) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    let out_path = output_path(p, "_rotated", None, &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                // The `image` crate only supports 90° increments natively.
                // For arbitrary angles we approximate: snap to nearest 90° or
                // use repeated 90° rotations + note about limitation.
                let rounded = (degrees.round() as i32) % 360;
                let rotated = match rounded {
                    90 | -270 => img.rotate90(),
                    180 | -180 => img.rotate180(),
                    270 | -90 => img.rotate270(),
                    0 => img,
                    other => {
                        // Arbitrary angle: use filter and note limitation
                        let rad = (other as f64).to_radians();
                        // Use the image crate's rotate about center
                        let (w, h) = (img.width(), img.height());
                        let bg = image::Rgba([0, 0, 0, 0]);
                        image::imageops::rotate_about_center(
                            &img,
                            rad,
                            image::imageops::FilterType::Lanczos3,
                            bg,
                        )
                    }
                };
                match rotated.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_ROTATE ok\ninput={}\noutput={}\ndegrees={degrees}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save rotated image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        match run_command(
            im,
            &[
                &p.to_string_lossy(),
                "-rotate",
                &degrees.to_string(),
                &out_path.to_string_lossy(),
            ],
        ) {
            Ok(_) => format!(
                "IMAGE_ROTATE ok\ninput={}\noutput={}\ndegrees={degrees}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 7. image_flip
// ---------------------------------------------------------------------------

/// Flip an image horizontally (`horizontal=true`) or vertically (`horizontal=false`).
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_flip_work(path: PathBuf, horizontal: bool, output: Option<PathBuf>) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    let suffix = if horizontal { "_hflip" } else { "_vflip" };
    let out_path = output_path(p, suffix, None, &output);
    let direction = if horizontal { "horizontal" } else { "vertical" };

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let flipped = if horizontal {
                    image::imageops::flip_horizontal(&img)
                } else {
                    image::imageops::flip_vertical(&img)
                };
                match flipped.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_FLIP ok\ninput={}\noutput={}\ndirection={direction}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save flipped image: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        let flag = if horizontal { "-flop" } else { "-flip" };
        match run_command(
            im,
            &[&p.to_string_lossy(), flag, &out_path.to_string_lossy()],
        ) {
            Ok(_) => format!(
                "IMAGE_FLIP ok\ninput={}\noutput={}\ndirection={direction}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 8. image_thumbnail
// ---------------------------------------------------------------------------

/// Create a thumbnail that fits within a square of `size` × `size` pixels,
/// preserving aspect ratio.
///
/// Uses the `image` crate when available, otherwise ImageMagick subprocess.
pub fn image_thumbnail_work(path: PathBuf, size: u32, output: Option<PathBuf>) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    if size == 0 {
        return "Error: thumbnail size must be > 0".to_string();
    }
    let out_path = output_path(p, "_thumb", None, &output);

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let thumb = img.thumbnail(size, size);
                match thumb.save(&out_path) {
                    Ok(()) => format!(
                        "IMAGE_THUMBNAIL ok\ninput={}\noutput={}\nmax_size={size}x{size}\n",
                        p.display(),
                        out_path.display()
                    ),
                    Err(e) => format!("Error: cannot save thumbnail: {e}"),
                }
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let im = match find_imagemagick() {
            Ok(v) => v,
            Err(e) => return e,
        };
        let geometry = format!("{size}x{size}");
        match run_command(
            im,
            &[
                &p.to_string_lossy(),
                "-thumbnail",
                &geometry,
                &out_path.to_string_lossy(),
            ],
        ) {
            Ok(_) => format!(
                "IMAGE_THUMBNAIL ok\ninput={}\noutput={}\nmax_size={size}x{size}\n",
                p.display(),
                out_path.display()
            ),
            Err(e) => e,
        }
    }
}

// ---------------------------------------------------------------------------
// 9. image_histogram
// ---------------------------------------------------------------------------

/// Compute and return a text histogram of color distribution (R/G/B channel stats).
///
/// Uses the `image` crate when available. Otherwise uses ImageMagick `convert`
/// with `- histogram` or `identify -verbose`.
pub fn image_histogram_work(path: PathBuf) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }

    #[cfg(feature = "ocr-tools")]
    {
        match image::open(p) {
            Ok(img) => {
                let rgb = img.to_rgb8();
                let (w, h) = (rgb.width() as u64, rgb.height() as u64);
                let pixel_count = w * h;
                let mut r_sum: u64 = 0;
                let mut g_sum: u64 = 0;
                let mut b_sum: u64 = 0;
                let mut r_hist = [0u64; 256];
                let mut g_hist = [0u64; 256];
                let mut b_hist = [0u64; 256];

                for pixel in rgb.pixels() {
                    let [r, g, b] = pixel.0;
                    r_sum += r as u64;
                    g_sum += g as u64;
                    b_sum += b as u64;
                    r_hist[r as usize] += 1;
                    g_hist[g as usize] += 1;
                    b_hist[b as usize] += 1;
                }

                let r_mean = r_sum as f64 / pixel_count as f64;
                let g_mean = g_sum as f64 / pixel_count as f64;
                let b_mean = b_sum as f64 / pixel_count as f64;

                let mut out = format!("IMAGE_HISTOGRAM path={}\n", p.display());
                out.push_str(&format!("pixels={pixel_count}\n"));
                out.push_str(&format!(
                    "channel_means: R={r_mean:.1} G={g_mean:.1} B={b_mean:.1}\n\n"
                ));

                // Bin histogram into 16 buckets for readability
                out.push_str("R histogram (16 bins):\n");
                out.push_str(&format_binned_histogram(&r_hist, 16));
                out.push_str("\nG histogram (16 bins):\n");
                out.push_str(&format_binned_histogram(&g_hist, 16));
                out.push_str("\nB histogram (16 bins):\n");
                out.push_str(&format_binned_histogram(&b_hist, 16));

                let (truncated, clipped) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
                let mut result = truncated;
                if clipped {
                    result.push_str(&format!(
                        "\n[truncated to {} chars]\n",
                        super::MAX_RESPONSE_CHARS
                    ));
                }
                result
            }
            Err(e) => format!("Error: cannot open image: {e}"),
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        // Use identify -verbose to get histogram data
        if !cli_exists("identify") {
            return "Error: `identify` (ImageMagick) not found on PATH".to_string();
        }
        match run_command("identify", &["-verbose", &p.to_string_lossy()]) {
            Ok(verbose) => {
                let mut out = format!("IMAGE_HISTOGRAM path={}\n", p.display());
                // Extract histogram-related lines from identify -verbose
                let mut in_histogram = false;
                for line in verbose.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("Histogram:") {
                        in_histogram = true;
                        out.push_str("Histogram (from identify):\n");
                        continue;
                    }
                    if in_histogram {
                        if trimmed.starts_with("Rendering intent")
                            || trimmed.starts_with("Gamma")
                            || trimmed.is_empty()
                        {
                            in_histogram = false;
                        } else {
                            out.push_str(&format!("{trimmed}\n"));
                        }
                    }
                    // Also grab channel statistics
                    if trimmed.starts_with("Channel")
                        || trimmed.contains("mean:")
                        || trimmed.contains("standard")
                    {
                        out.push_str(&format!("{trimmed}\n"));
                    }
                }
                let (truncated, clipped) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
                let mut result = truncated;
                if clipped {
                    result.push_str(&format!(
                        "\n[truncated to {} chars]\n",
                        super::MAX_RESPONSE_CHARS
                    ));
                }
                result
            }
            Err(e) => e,
        }
    }
}

/// Format a 256-bin histogram into `num_bins` bins with ASCII bar visualization.
fn format_binned_histogram(hist: &[u64; 256], num_bins: usize) -> String {
    let max_val = hist.iter().copied().max().unwrap_or(1);
    let mut out = String::new();
    for i in 0..num_bins {
        let start = i * 256 / num_bins;
        let end = (i + 1) * 256 / num_bins - 1;
        let count: u64 = hist[start..=end].iter().sum();
        let bar_len = if max_val > 0 {
            ((count as f64 / max_val as f64) * 40.0) as usize
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        out.push_str(&format!(" {}-{}: {:>8} {}\n", start, end, count, bar));
    }
    out
}

// ---------------------------------------------------------------------------
// 10. image_compare
// ---------------------------------------------------------------------------

/// Compare two images: report dimensions, format, file size, and a basic pixel
/// difference estimate.
///
/// When the `image` crate is available, computes per-channel mean absolute
/// difference. Otherwise, uses `identify` for metadata and `compare` (ImageMagick)
/// for a difference metric.
pub fn image_compare_work(path_a: PathBuf, path_b: PathBuf) -> String {
    let meta_a = match validate_image_path(&path_a) {
        Ok(m) => m,
        Err(e) => return e,
    };
    let meta_b = match validate_image_path(&path_b) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let mut out = format!(
        "IMAGE_COMPARE\npath_a={}\npath_b={}\n",
        path_a.display(),
        path_b.display()
    );
    out.push_str(&format!(
        "size_a={}\nsize_b={}\n",
        format_size(meta_a.len()),
        format_size(meta_b.len())
    ));

    #[cfg(feature = "ocr-tools")]
    {
        let img_a = match image::open(&path_a) {
            Ok(i) => i,
            Err(e) => return format!("Error: cannot open image A: {e}"),
        };
        let img_b = match image::open(&path_b) {
            Ok(i) => i,
            Err(e) => return format!("Error: cannot open image B: {e}"),
        };

        out.push_str(&format!(
            "dimensions_a={}x{}\ndimensions_b={}x{}\n",
            img_a.width(),
            img_a.height(),
            img_b.width(),
            img_b.height()
        ));

        let format_a = path_a
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");
        let format_b = path_b
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");
        out.push_str(&format!("format_a={format_a}\nformat_b={format_b}\n"));

        if img_a.width() != img_b.width() || img_a.height() != img_b.height() {
            out.push_str("pixel_diff=different dimensions, cannot compute pixel difference\n");
        } else {
            let rgb_a = img_a.to_rgb8();
            let rgb_b = img_b.to_rgb8();
            let n = rgb_a.pixels().count() as f64;
            let mut total_diff = 0.0f64;
            let mut diff_r = 0.0f64;
            let mut diff_g = 0.0f64;
            let mut diff_b = 0.0f64;
            let mut identical_pixels = 0u64;

            for (pa, pb) in rgb_a.pixels().zip(rgb_b.pixels()) {
                let dr = (pa.0[0] as f64 - pb.0[0] as f64).abs();
                let dg = (pa.0[1] as f64 - pb.0[1] as f64).abs();
                let db = (pa.0[2] as f64 - pb.0[2] as f64).abs();
                diff_r += dr;
                diff_g += dg;
                diff_b += db;
                total_diff += dr + dg + db;
                if dr == 0.0 && dg == 0.0 && db == 0.0 {
                    identical_pixels += 1;
                }
            }

            let mean_diff = total_diff / (n * 3.0);
            let identical_pct = (identical_pixels as f64 / n) * 100.0;
            out.push_str(&format!("mean_pixel_diff={mean_diff:.2}\n"));
            out.push_str(&format!(
                "channel_mean_diff: R={:.2} G={:.2} B={:.2}\n",
                diff_r / n,
                diff_g / n,
                diff_b / n
            ));
            out.push_str(&format!("identical_pixels={identical_pct:.1}%\n"));
        }
    }

    #[cfg(not(feature = "ocr-tools"))]
    {
        let format_a = path_a
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");
        let format_b = path_b
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");
        out.push_str(&format!("format_a={format_a}\nformat_b={format_b}\n"));

        // Get dimensions via identify
        if cli_exists("identify") {
            if let Ok(info_a) =
                run_command("identify", &["-format", "%wx%h", &path_a.to_string_lossy()])
            {
                out.push_str(&format!("dimensions_a={info_a}\n"));
            }
            if let Ok(info_b) =
                run_command("identify", &["-format", "%wx%h", &path_b.to_string_lossy()])
            {
                out.push_str(&format!("dimensions_b={info_b}\n"));
            }
        }

        // Use ImageMagick `compare` for a difference metric
        if cli_exists("compare") {
            match Command::new("compare")
                .args([
                    "-metric",
                    "MAE",
                    &path_a.to_string_lossy(),
                    &path_b.to_string_lossy(),
                    "/dev/null",
                ])
                .output()
            {
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    // ImageMagick compare writes the metric to stderr
                    let metric = stderr.trim();
                    if !metric.is_empty() {
                        out.push_str(&format!("compare_metric_MAE={metric}\n"));
                    }
                }
                Err(_) => out.push_str("compare_metric=unavailable\n"),
            }
        }
    }

    let (truncated, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    truncated
}

// ---------------------------------------------------------------------------
// 11. image_extract_exif
// ---------------------------------------------------------------------------

/// Extract EXIF metadata from an image using `exiftool` or ImageMagick `identify`.
///
/// This function always uses subprocess tools since the `image` crate does not
/// natively parse EXIF. Falls back from `exiftool` to `identify -verbose`.
pub fn image_extract_exif_work(path: PathBuf) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }

    // Prefer exiftool — comprehensive EXIF output
    if cli_exists("exiftool") {
        match run_command("exiftool", &[&p.to_string_lossy()]) {
            Ok(output) => {
                let mut out = format!("IMAGE_EXIF path={}\n\n", p.display());
                out.push_str(&output);
                let (truncated, clipped) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
                let mut result = truncated;
                if clipped {
                    result.push_str(&format!(
                        "\n[truncated to {} chars]\n",
                        super::MAX_RESPONSE_CHARS
                    ));
                }
                return result;
            }
            Err(e) => return e,
        }
    }

    // Fallback: identify -verbose includes some EXIF
    if cli_exists("identify") {
        match run_command("identify", &["-verbose", &p.to_string_lossy()]) {
            Ok(verbose) => {
                let mut out = format!(
                    "IMAGE_EXIF path={}\n(fallback: identify -verbose)\n\n",
                    p.display()
                );
                // Extract EXIF-related lines
                let mut in_exif = false;
                for line in verbose.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("exif:") || trimmed.starts_with("EXIF") {
                        in_exif = true;
                    }
                    if in_exif {
                        if trimmed.is_empty() {
                            in_exif = false;
                        } else {
                            out.push_str(&format!("{trimmed}\n"));
                        }
                    }
                    // Also capture orientation, date, make, model lines
                    for keyword in &[
                        "Make:",
                        "Model:",
                        "DateTime:",
                        "Orientation:",
                        "Software:",
                        "Copyright:",
                    ] {
                        if trimmed.starts_with(keyword) {
                            out.push_str(&format!("{trimmed}\n"));
                        }
                    }
                }
                if out.lines().count() <= 3 {
                    out.push_str("No EXIF data found.\n");
                }
                let (truncated, clipped) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
                let mut result = truncated;
                if clipped {
                    result.push_str(&format!(
                        "\n[truncated to {} chars]\n",
                        super::MAX_RESPONSE_CHARS
                    ));
                }
                return result;
            }
            Err(e) => return e,
        }
    }

    format!("Error: no EXIF tool available. Install `exiftool` or ImageMagick (`identify`).")
}

// ---------------------------------------------------------------------------
// 12. image_watermark
// ---------------------------------------------------------------------------

/// Add a text watermark to an image.
///
/// Uses ImageMagick `convert`/`magick` for text rendering (the `image` crate
/// does not have font rendering). Supports positions: `top-left`, `top-right`,
/// `center`, `bottom-left`, `bottom-right`.
pub fn image_watermark_work(
    path: PathBuf,
    text: String,
    position: String,
    output: Option<PathBuf>,
) -> String {
    let p = &path;
    if let Err(e) = validate_image_path(p) {
        return e;
    }
    if text.trim().is_empty() {
        return "Error: watermark text must not be empty".to_string();
    }

    let im = match find_imagemagick() {
        Ok(v) => v,
        Err(e) => return e,
    };

    let out_path = output_path(p, "_watermarked", None, &output);

    let pos = position.trim().to_ascii_lowercase();
    let gravity = match pos.as_str() {
        "top-left" | "tl" => "NorthWest",
        "top-right" | "tr" => "NorthEast",
        "center" | "c" => "Center",
        "bottom-left" | "bl" => "SouthWest",
        "bottom-right" | "br" => "SouthEast",
        "top" | "north" => "North",
        "bottom" | "south" => "South",
        "left" | "west" => "West",
        "right" | "east" => "East",
        _ => "SouthEast",
    };

    // Build the -annotate argument with text styling
    // Semi-transparent white text with a subtle shadow for visibility
    let annotate_arg = format!("0x0+10+10 {text}");

    match Command::new(im)
        .args([
            &p.to_string_lossy(),
            "-gravity",
            gravity,
            "-fill",
            "rgba(255,255,255,0.5)",
            "-stroke",
            "rgba(0,0,0,0.3)",
            "-strokewidth",
            "2",
            "-pointsize",
            "36",
            "-annotate",
            &annotate_arg,
            &out_path.to_string_lossy(),
        ])
        .output()
    {
        Ok(o) => {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let (se, _) = super::truncate_chars(stderr.trim(), 4000);
                return format!(
                    "Error: ImageMagick watermark failed (exit={:?})\nSTDERR:\n{se}\n",
                    o.status.code()
                );
            }
            format!(
                "IMAGE_WATERMARK ok\ninput={}\noutput={}\nposition={pos}\ntext=\"{text}\"\n",
                p.display(),
                out_path.display()
            )
        }
        Err(e) => format!("Error: spawn {im}: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a small temporary PNG file using the image crate (if available)
    /// or build a valid PNG from scratch using flate2 for zlib compression.
    fn create_test_png(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        #[cfg(feature = "ocr-tools")]
        {
            let img = image::DynamicImage::ImageRgb8(image::ImageBuffer::from_pixel(
                64,
                64,
                image::Rgb([128, 64, 200]),
            ));
            img.save(&path).expect("save test png");
        }
        #[cfg(not(feature = "ocr-tools"))]
        {
            let png_bytes = build_minimal_png();
            fs::write(&path, &png_bytes).expect("write test png");
        }
        path
    }

    /// Build a minimal 2×2 RGB PNG from scratch with correct CRCs and zlib data.
    fn build_minimal_png() -> Vec<u8> {
        use std::io::Write as _;

        // --- PNG helpers ---
        fn u32_be(v: u32) -> [u8; 4] {
            v.to_be_bytes()
        }

        /// Compute CRC32 (PNG polynomial 0xEDB88320) over the concatenation of type + data.
        fn png_crc32(chunk_type: &[u8; 4], data: &[u8]) -> u32 {
            // Standard CRC-32 used in PNG (polynomial 0xEDB88320, init 0xFFFFFFFF, final XOR)
            let mut crc: u32 = 0xFFFF_FFFF;
            let table: [u32; 256] = {
                let mut t = [0u32; 256];
                for i in 0..256u32 {
                    let mut c = i;
                    for _ in 0..8 {
                        if c & 1 == 1 {
                            c = 0xEDB88320 ^ (c >> 1);
                        } else {
                            c >>= 1;
                        }
                    }
                    t[i as usize] = c;
                }
                t
            };
            fn update(crc: u32, table: &[u32; 256], buf: &[u8]) -> u32 {
                let mut c = crc;
                for &b in buf {
                    c = table[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
                }
                c
            }
            crc = update(crc, &table, chunk_type);
            crc = update(crc, &table, data);
            crc ^ 0xFFFF_FFFF
        }

        fn make_chunk(chunk_type: [u8; 4], data: &[u8]) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(&u32_be(data.len() as u32));
            out.extend_from_slice(&chunk_type);
            out.extend_from_slice(data);
            let crc = png_crc32(&chunk_type, data);
            out.extend_from_slice(&u32_be(crc));
            out
        }

        // --- IHDR: 2×2, 8-bit RGB ---
        let mut ihdr_data = Vec::new();
        ihdr_data.extend_from_slice(&u32_be(2)); // width
        ihdr_data.extend_from_slice(&u32_be(2)); // height
        ihdr_data.push(8); // bit depth
        ihdr_data.push(2); // color type: RGB
        ihdr_data.push(0); // compression
        ihdr_data.push(0); // filter
        ihdr_data.push(0); // interlace
        let ihdr = make_chunk(*b"IHDR", &ihdr_data);

        // --- IDAT: zlib-compressed scanlines for 2×2 RGB ---
        // Each scanline: filter byte (0=None) + 2 pixels × 3 bytes = 7 bytes per line, 2 lines
        let raw_data: &[u8] = &[
            0x00, // filter: None
            0xFF, 0x00, 0x00, // pixel (0,0): red
            0x00, 0xFF, 0x00, // pixel (1,0): green
            0x00, // filter: None
            0x00, 0x00, 0xFF, // pixel (0,1): blue
            0xFF, 0xFF, 0xFF, // pixel (1,1): white
        ];
        let mut zlib_buf = Vec::new();
        {
            let mut encoder =
                flate2::write::ZlibEncoder::new(&mut zlib_buf, flate2::Compression::default());
            encoder.write_all(raw_data).expect("zlib compress");
            encoder.finish().expect("zlib finish");
        }
        let idat = make_chunk(*b"IDAT", &zlib_buf);

        // --- IEND ---
        let iend = make_chunk(*b"IEND", &[]);

        // --- Assemble PNG ---
        let mut png = Vec::new();
        png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // signature
        png.extend_from_slice(&ihdr);
        png.extend_from_slice(&idat);
        png.extend_from_slice(&iend);
        png
    }

    #[test]
    fn test_validate_image_path_missing() {
        let result = validate_image_path(Path::new("/no/such/file/here.png"));
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Error:"));
    }

    #[test]
    fn test_validate_image_path_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = validate_image_path(dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a file"));
    }

    #[test]
    fn test_output_path_with_explicit_output() {
        let out = output_path(
            Path::new("/tmp/test.png"),
            "_x",
            None,
            &Some(PathBuf::from("/tmp/custom.jpg")),
        );
        assert_eq!(out, PathBuf::from("/tmp/custom.jpg"));
    }

    #[test]
    fn test_output_path_derived() {
        let out = output_path(Path::new("/tmp/test.png"), "_gray", None, &None);
        assert_eq!(out, PathBuf::from("/tmp/test_gray.png"));
    }

    #[test]
    fn test_output_path_derived_with_new_ext() {
        let out = output_path(Path::new("/tmp/test.png"), "_conv", Some("jpg"), &None);
        assert_eq!(out, PathBuf::from("/tmp/test_conv.jpg"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert!(format_size(2048).contains("KiB"));
        assert!(format_size(5 * 1024 * 1024).contains("MiB"));
    }

    #[test]
    fn test_image_info_nonexistent() {
        let result = image_info_work(PathBuf::from("/nonexistent/image.png"));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_info_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "info_test.png");
        let result = image_info_work(path);
        assert!(result.contains("IMAGE_INFO"));
        assert!(result.contains("file_size="));
    }

    #[test]
    fn test_image_resize_nonexistent() {
        let result = image_resize_work(PathBuf::from("/no/image.png"), 100, 100, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_resize_zero_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "resize_test.png");
        let result = image_resize_work(path, 0, 100, None);
        assert!(result.contains("Error:"));
    }

    #[test]
    fn test_image_resize_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "resize_valid.png");
        let out = dir.path().join("resized.png");
        let result = image_resize_work(path, 32, 32, Some(out.clone()));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_RESIZE ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            // If ImageMagick is available, should succeed; otherwise skip
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_RESIZE ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_convert_unsupported_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "conv_test.png");
        let result = image_convert_work(path, "xyz".to_string(), None);
        assert!(result.contains("Error:"));
        assert!(result.contains("unsupported"));
    }

    #[test]
    fn test_image_convert_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "conv_valid.png");
        let out = dir.path().join("converted.jpg");
        let result = image_convert_work(path, "jpg".to_string(), Some(out.clone()));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_CONVERT ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_CONVERT ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_grayscale_nonexistent() {
        let result = image_grayscale_work(PathBuf::from("/no/img.png"), None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_grayscale_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "gray_valid.png");
        let out = dir.path().join("gray.png");
        let result = image_grayscale_work(path, Some(out));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_GRAYSCALE ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_GRAYSCALE ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_crop_zero_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "crop_test.png");
        let result = image_crop_work(path, 0, 0, 0, 0, None);
        assert!(result.contains("Error:"));
    }

    #[test]
    fn test_image_crop_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "crop_valid.png");
        let out = dir.path().join("cropped.png");
        let result = image_crop_work(path, 0, 0, 16, 16, Some(out));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_CROP ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_CROP ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_rotate_nonexistent() {
        let result = image_rotate_work(PathBuf::from("/no/img.png"), 90.0, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_rotate_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "rotate_valid.png");
        let out = dir.path().join("rotated.png");
        let result = image_rotate_work(path, 90.0, Some(out));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_ROTATE ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_ROTATE ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_flip_nonexistent() {
        let result = image_flip_work(PathBuf::from("/no/img.png"), true, None);
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_flip_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "flip_valid.png");
        let out = dir.path().join("flipped.png");
        let result = image_flip_work(path, true, Some(out));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_FLIP ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_FLIP ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_thumbnail_zero_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "thumb_test.png");
        let result = image_thumbnail_work(path, 0, None);
        assert!(result.contains("Error:"));
    }

    #[test]
    fn test_image_thumbnail_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "thumb_valid.png");
        let out = dir.path().join("thumb.png");
        let result = image_thumbnail_work(path, 32, Some(out));
        #[cfg(feature = "ocr-tools")]
        assert!(result.contains("IMAGE_THUMBNAIL ok"), "result: {result}");
        #[cfg(not(feature = "ocr-tools"))]
        {
            if result.contains("Error:") && result.contains("ImageMagick") {
                eprintln!("Skipping: ImageMagick not available");
            } else {
                assert!(result.contains("IMAGE_THUMBNAIL ok"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_histogram_nonexistent() {
        let result = image_histogram_work(PathBuf::from("/no/img.png"));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_histogram_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "hist_valid.png");
        let result = image_histogram_work(path);
        #[cfg(feature = "ocr-tools")]
        {
            assert!(result.contains("IMAGE_HISTOGRAM"), "result: {result}");
            assert!(result.contains("pixels="));
        }
        #[cfg(not(feature = "ocr-tools"))]
        {
            // May fail if identify is not available
            if !result.starts_with("Error:") {
                assert!(result.contains("IMAGE_HISTOGRAM"), "result: {result}");
            }
        }
    }

    #[test]
    fn test_image_compare_nonexistent() {
        let result = image_compare_work(PathBuf::from("/no/a.png"), PathBuf::from("/no/b.png"));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_compare_same_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "same.png");
        let result = image_compare_work(path.clone(), path);
        assert!(result.contains("IMAGE_COMPARE"), "result: {result}");
    }

    #[test]
    fn test_image_extract_exif_nonexistent() {
        let result = image_extract_exif_work(PathBuf::from("/no/img.png"));
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_image_watermark_empty_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_png(dir.path(), "wm_test.png");
        let result = image_watermark_work(path, "".to_string(), "center".to_string(), None);
        assert!(result.contains("Error:"));
    }

    #[test]
    fn test_image_watermark_nonexistent() {
        let result = image_watermark_work(
            PathBuf::from("/no/img.png"),
            "test".to_string(),
            "center".to_string(),
            None,
        );
        assert!(result.starts_with("Error:"));
    }

    #[test]
    fn test_format_binned_histogram() {
        let mut hist = [0u64; 256];
        hist[0] = 100;
        hist[128] = 50;
        hist[255] = 25;
        let result = format_binned_histogram(&hist, 16);
        assert!(result.contains("0-15:"));
        assert!(result.contains("100"));
    }

    #[test]
    fn test_supported_formats() {
        assert!(SUPPORTED_FORMATS.contains(&"png"));
        assert!(SUPPORTED_FORMATS.contains(&"jpg"));
        assert!(SUPPORTED_FORMATS.contains(&"webp"));
        assert!(SUPPORTED_FORMATS.contains(&"bmp"));
        assert!(SUPPORTED_FORMATS.contains(&"ico"));
        assert!(SUPPORTED_FORMATS.contains(&"tiff"));
    }

    #[test]
    fn test_max_image_bytes() {
        assert_eq!(MAX_IMAGE_BYTES, 50 * 1024 * 1024);
    }

    #[test]
    fn test_image_crop_out_of_bounds() {
        #[cfg(feature = "ocr-tools")]
        {
            let dir = tempfile::tempdir().unwrap();
            let path = create_test_png(dir.path(), "crop_oob.png");
            // x=100 is out of bounds for a 64x64 image
            let result = image_crop_work(path, 100, 100, 10, 10, None);
            assert!(result.contains("Error:"));
            assert!(result.contains("outside image bounds"));
        }
    }
}
