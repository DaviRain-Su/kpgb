use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Image optimization configuration
#[derive(Debug, Clone)]
pub struct ImageOptimizationConfig {
    /// Maximum width for images (resize if larger)
    pub max_width: u32,
    /// Maximum height for images (resize if larger)
    pub max_height: u32,
    /// JPEG quality (0-100)
    pub jpeg_quality: u8,
    /// PNG compression level (0-10, where 0 is fastest and 10 is best compression)
    pub png_compression: u8,
    /// Enable WebP conversion
    pub enable_webp: bool,
    /// WebP quality (0-100)
    pub webp_quality: u8,
}

impl Default for ImageOptimizationConfig {
    fn default() -> Self {
        Self {
            max_width: 1920,
            max_height: 1080,
            jpeg_quality: 85,
            png_compression: 6,
            enable_webp: false, // Disabled by default as it requires additional setup
            webp_quality: 85,
        }
    }
}

/// Optimize all images in a directory recursively
pub async fn optimize_images_in_directory(
    dir: &Path,
    config: &ImageOptimizationConfig,
) -> Result<OptimizationStats> {
    let mut stats = OptimizationStats::default();
    optimize_directory_recursive(dir, config, &mut stats)?;
    Ok(stats)
}

fn optimize_directory_recursive(
    dir: &Path,
    config: &ImageOptimizationConfig,
    stats: &mut OptimizationStats,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            optimize_directory_recursive(&path, config, stats)?;
        } else if is_image_file(&path) {
            match optimize_image(&path, config) {
                Ok(saved) => {
                    stats.images_processed += 1;
                    stats.bytes_saved += saved;
                    info!(
                        "Optimized image: {} (saved {} bytes)",
                        path.display(),
                        saved
                    );
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("Failed to optimize image {}: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

/// Check if a file is an image based on extension
fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
        ),
        None => false,
    }
}

/// Optimize a single image file
pub fn optimize_image(path: &Path, config: &ImageOptimizationConfig) -> Result<i64> {
    let original_size = fs::metadata(path)?.len() as i64;

    // Load the image
    let img = image::open(path)?;

    // Resize if necessary
    let img = resize_if_needed(img, config.max_width, config.max_height);

    // Determine output format based on input
    let format = determine_format(path)?;

    // Create a temporary file
    let temp_path = path.with_extension("tmp");

    // Save with optimization
    match format {
        ImageFormat::Jpeg => {
            save_jpeg(&img, &temp_path, config.jpeg_quality)?;
        }
        ImageFormat::Png => {
            save_png(&img, &temp_path, config.png_compression)?;
        }
        _ => {
            // For other formats, save as-is
            img.save(&temp_path)?;
        }
    }

    // Check if optimization actually reduced size
    let new_size = fs::metadata(&temp_path)?.len() as i64;
    let saved = original_size - new_size;

    if saved > 0 {
        // Replace original with optimized version
        fs::rename(&temp_path, path)?;
        Ok(saved)
    } else {
        // Keep original if optimization didn't help
        fs::remove_file(&temp_path)?;
        Ok(0)
    }
}

/// Resize image if it exceeds maximum dimensions
fn resize_if_needed(img: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    if width <= max_width && height <= max_height {
        return img;
    }

    // Calculate new dimensions maintaining aspect ratio
    let width_ratio = width as f32 / max_width as f32;
    let height_ratio = height as f32 / max_height as f32;
    let ratio = width_ratio.max(height_ratio);

    let new_width = (width as f32 / ratio) as u32;
    let new_height = (height as f32 / ratio) as u32;

    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// Determine image format from file extension
fn determine_format(path: &Path) -> Result<ImageFormat> {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        Some(ext) => match ext.as_str() {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "gif" => Ok(ImageFormat::Gif),
            "webp" => Ok(ImageFormat::WebP),
            "bmp" => Ok(ImageFormat::Bmp),
            _ => Err(anyhow::anyhow!("Unsupported image format: {}", ext)),
        },
        None => Err(anyhow::anyhow!("No file extension found")),
    }
}

/// Save image as JPEG with specified quality
fn save_jpeg(img: &DynamicImage, path: &Path, quality: u8) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Convert to RGB8 if necessary (JPEG doesn't support alpha)
    let rgb_img = img.to_rgb8();

    // Use image crate's JPEG encoder with quality setting
    image::codecs::jpeg::JpegEncoder::new_with_quality(writer, quality).encode_image(&rgb_img)?;

    Ok(())
}

/// Save image as PNG with specified compression level
fn save_png(img: &DynamicImage, path: &Path, _compression: u8) -> Result<()> {
    // Note: The image crate doesn't expose PNG compression level directly
    // For now, we'll use the default compression
    img.save(path)?;
    Ok(())
}

/// Statistics from image optimization
#[derive(Debug, Default)]
pub struct OptimizationStats {
    pub images_processed: usize,
    pub bytes_saved: i64,
    pub errors: usize,
}

impl OptimizationStats {
    pub fn summary(&self) -> String {
        format!(
            "Processed {} images, saved {} bytes ({:.2} MB), {} errors",
            self.images_processed,
            self.bytes_saved,
            self.bytes_saved as f64 / 1_048_576.0,
            self.errors
        )
    }
}

/// Process images referenced in markdown content
/// Returns the markdown with updated image paths
pub async fn process_markdown_images(
    markdown: &str,
    content_dir: &Path,
    output_dir: &Path,
    config: &ImageOptimizationConfig,
) -> Result<(String, OptimizationStats)> {
    let mut stats = OptimizationStats::default();
    let updated_markdown = markdown.to_string();

    // Find all image references in markdown
    let image_regex = regex::Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)")?;

    for cap in image_regex.captures_iter(markdown) {
        let alt_text = &cap[1];
        let image_path = &cap[2];

        // Skip external URLs
        if image_path.starts_with("http://") || image_path.starts_with("https://") {
            continue;
        }

        // Resolve image path relative to content directory
        let source_path = content_dir.join(image_path);

        if source_path.exists() && is_image_file(&source_path) {
            // Create output path maintaining directory structure
            let relative_path = Path::new(image_path);
            let output_path = output_dir.join(relative_path);

            // Create output directory if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy and optimize the image
            fs::copy(&source_path, &output_path)?;

            match optimize_image(&output_path, config) {
                Ok(saved) => {
                    stats.images_processed += 1;
                    stats.bytes_saved += saved;
                    info!(
                        "Optimized image: {} (saved {} bytes)",
                        output_path.display(),
                        saved
                    );
                }
                Err(e) => {
                    stats.errors += 1;
                    warn!("Failed to optimize image {}: {}", output_path.display(), e);
                }
            }
        }
    }

    Ok((updated_markdown, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.JPEG")));
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.gif")));
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
    }

    #[test]
    fn test_determine_format() {
        assert!(matches!(
            determine_format(Path::new("test.jpg")).unwrap(),
            ImageFormat::Jpeg
        ));
        assert!(matches!(
            determine_format(Path::new("test.png")).unwrap(),
            ImageFormat::Png
        ));
        assert!(determine_format(Path::new("test.unknown")).is_err());
    }
}
