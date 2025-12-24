//! Image Optimization Pipeline
//!
//! Optimizes images for web delivery with format conversion, resizing, and compression.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Image optimization errors
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Failed to load image: {0}")]
    LoadError(String),

    #[error("Failed to encode image: {0}")]
    EncodeError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ProcessingError(String),
}

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
    Gif,
    Svg,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "webp" => Some(Self::WebP),
            "avif" => Some(Self::Avif),
            "gif" => Some(Self::Gif),
            "svg" => Some(Self::Svg),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
            Self::Gif => "gif",
            Self::Svg => "svg",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
            Self::Gif => "image/gif",
            Self::Svg => "image/svg+xml",
        }
    }
}

/// Image optimization configuration
#[derive(Debug, Clone)]
pub struct ImageOptimizerConfig {
    /// Target quality for lossy formats (1-100)
    pub quality: u8,
    /// Enable WebP conversion
    pub enable_webp: bool,
    /// Enable AVIF conversion
    pub enable_avif: bool,
    /// Maximum image width
    pub max_width: u32,
    /// Maximum image height
    pub max_height: u32,
    /// Generate responsive image sizes
    pub responsive_sizes: Vec<u32>,
    /// Strip metadata (EXIF, etc.)
    pub strip_metadata: bool,
    /// Enable progressive JPEG
    pub progressive_jpeg: bool,
    /// PNG compression level (0-9)
    pub png_compression: u8,
    /// Cache optimized images
    pub cache_enabled: bool,
    /// Placeholder blur radius for lazy loading
    pub placeholder_blur: u32,
    /// Generate dominant color placeholder
    pub color_placeholder: bool,
}

impl Default for ImageOptimizerConfig {
    fn default() -> Self {
        Self {
            quality: 80,
            enable_webp: true,
            enable_avif: false, // AVIF encoding is slower
            max_width: 2560,
            max_height: 2560,
            responsive_sizes: vec![320, 640, 768, 1024, 1280, 1920],
            strip_metadata: true,
            progressive_jpeg: true,
            png_compression: 6,
            cache_enabled: true,
            placeholder_blur: 20,
            color_placeholder: true,
        }
    }
}

/// Optimized image result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedImage {
    /// Original filename
    pub original_filename: String,
    /// Original format
    pub original_format: ImageFormat,
    /// Original dimensions
    pub original_width: u32,
    pub original_height: u32,
    /// Original file size
    pub original_size: usize,
    /// Generated variants
    pub variants: Vec<ImageVariant>,
    /// Low-quality placeholder (base64 data URI)
    pub placeholder: Option<String>,
    /// Dominant color (hex)
    pub dominant_color: Option<String>,
    /// Content hash for caching
    pub hash: String,
}

/// Image variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariant {
    /// Output filename
    pub filename: String,
    /// Format
    pub format: ImageFormat,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// File size in bytes
    pub size: usize,
    /// URL path
    pub url: String,
}

/// Image optimizer
pub struct ImageOptimizer {
    config: ImageOptimizerConfig,
    cache: Arc<RwLock<HashMap<String, OptimizedImage>>>,
}

impl ImageOptimizer {
    pub fn new(config: ImageOptimizerConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Optimize an image from bytes
    pub fn optimize(&self, data: &[u8], filename: &str) -> Result<OptimizedImage, ImageError> {
        // Calculate content hash for caching
        let hash = blake3::hash(data).to_hex()[..16].to_string();

        // Check cache
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.read().get(&hash) {
                return Ok(cached.clone());
            }
        }

        // Determine format from extension
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let format = ImageFormat::from_extension(ext)
            .ok_or_else(|| ImageError::UnsupportedFormat(ext.to_string()))?;

        // Handle SVG separately (no pixel-based processing)
        if format == ImageFormat::Svg {
            return self.optimize_svg(data, filename, &hash);
        }

        // Load image using the image crate
        let img = image::load_from_memory(data)
            .map_err(|e| ImageError::LoadError(e.to_string()))?;

        let original_width = img.width();
        let original_height = img.height();

        // Generate variants
        let mut variants = Vec::new();

        // Generate responsive sizes
        for &target_width in &self.config.responsive_sizes {
            if target_width > original_width {
                continue;
            }

            // Calculate proportional height
            let ratio = target_width as f64 / original_width as f64;
            let target_height = (original_height as f64 * ratio) as u32;

            // Resize image
            let resized = img.resize(
                target_width,
                target_height,
                image::imageops::FilterType::Lanczos3,
            );

            // Generate original format variant
            let original_variant = self.encode_variant(
                &resized,
                filename,
                format,
                target_width,
                target_height,
                &hash,
            )?;
            variants.push(original_variant);

            // Generate WebP variant
            if self.config.enable_webp && format != ImageFormat::WebP {
                let webp_variant = self.encode_variant(
                    &resized,
                    filename,
                    ImageFormat::WebP,
                    target_width,
                    target_height,
                    &hash,
                )?;
                variants.push(webp_variant);
            }

            // Generate AVIF variant
            if self.config.enable_avif && format != ImageFormat::Avif {
                let avif_variant = self.encode_variant(
                    &resized,
                    filename,
                    ImageFormat::Avif,
                    target_width,
                    target_height,
                    &hash,
                )?;
                variants.push(avif_variant);
            }
        }

        // Generate placeholder
        let placeholder = self.generate_placeholder(&img)?;

        // Extract dominant color
        let dominant_color = if self.config.color_placeholder {
            Some(self.extract_dominant_color(&img))
        } else {
            None
        };

        let result = OptimizedImage {
            original_filename: filename.to_string(),
            original_format: format,
            original_width,
            original_height,
            original_size: data.len(),
            variants,
            placeholder: Some(placeholder),
            dominant_color,
            hash: hash.clone(),
        };

        // Cache result
        if self.config.cache_enabled {
            self.cache.write().insert(hash, result.clone());
        }

        Ok(result)
    }

    fn optimize_svg(&self, data: &[u8], filename: &str, hash: &str) -> Result<OptimizedImage, ImageError> {
        // For SVG, just minify and return
        let svg_str = String::from_utf8_lossy(data);
        let minified = minify_svg(&svg_str);

        let variant = ImageVariant {
            filename: filename.to_string(),
            format: ImageFormat::Svg,
            width: 0, // SVG dimensions are in the file
            height: 0,
            size: minified.len(),
            url: format!("/uploads/{}", filename),
        };

        Ok(OptimizedImage {
            original_filename: filename.to_string(),
            original_format: ImageFormat::Svg,
            original_width: 0,
            original_height: 0,
            original_size: data.len(),
            variants: vec![variant],
            placeholder: None,
            dominant_color: None,
            hash: hash.to_string(),
        })
    }

    fn encode_variant(
        &self,
        img: &image::DynamicImage,
        original_filename: &str,
        format: ImageFormat,
        width: u32,
        height: u32,
        hash: &str,
    ) -> Result<ImageVariant, ImageError> {
        let mut buffer = Cursor::new(Vec::new());

        let filename = generate_variant_filename(original_filename, width, format, hash);

        match format {
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buffer,
                    self.config.quality,
                );
                img.write_with_encoder(encoder)
                    .map_err(|e| ImageError::EncodeError(e.to_string()))?;
            }
            ImageFormat::Png => {
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut buffer,
                    image::codecs::png::CompressionType::Default,
                    image::codecs::png::FilterType::Adaptive,
                );
                img.write_with_encoder(encoder)
                    .map_err(|e| ImageError::EncodeError(e.to_string()))?;
            }
            ImageFormat::WebP => {
                // Use WebP encoder
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut buffer);
                img.write_with_encoder(encoder)
                    .map_err(|e| ImageError::EncodeError(e.to_string()))?;
            }
            ImageFormat::Gif => {
                let encoder = image::codecs::gif::GifEncoder::new(&mut buffer);
                img.write_with_encoder(encoder)
                    .map_err(|e| ImageError::EncodeError(e.to_string()))?;
            }
            _ => {
                return Err(ImageError::UnsupportedFormat(format!("{:?}", format)));
            }
        }

        let data = buffer.into_inner();

        Ok(ImageVariant {
            filename: filename.clone(),
            format,
            width,
            height,
            size: data.len(),
            url: format!("/uploads/optimized/{}", filename),
        })
    }

    fn generate_placeholder(&self, img: &image::DynamicImage) -> Result<String, ImageError> {
        // Create a tiny blurred version
        let tiny = img.resize(
            32,
            32,
            image::imageops::FilterType::Gaussian,
        );

        // Blur
        let blurred = tiny.blur(self.config.placeholder_blur as f32);

        // Encode as base64 JPEG
        let mut buffer = Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 20);
        blurred.write_with_encoder(encoder)
            .map_err(|e| ImageError::EncodeError(e.to_string()))?;

        let base64_data = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            buffer.into_inner(),
        );

        Ok(format!("data:image/jpeg;base64,{}", base64_data))
    }

    fn extract_dominant_color(&self, img: &image::DynamicImage) -> String {
        // Resize to 1x1 to get average color
        let tiny = img.resize_exact(1, 1, image::imageops::FilterType::Gaussian);
        let rgba = tiny.to_rgba8();
        let pixel = rgba.get_pixel(0, 0);

        format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
    }

    /// Get optimization statistics
    pub fn stats(&self) -> OptimizerStats {
        let cache = self.cache.read();

        let mut total_original = 0;
        let mut total_optimized = 0;
        let mut variant_count = 0;

        for img in cache.values() {
            total_original += img.original_size;
            for variant in &img.variants {
                total_optimized += variant.size;
                variant_count += 1;
            }
        }

        OptimizerStats {
            cached_images: cache.len(),
            total_variants: variant_count,
            total_original_bytes: total_original,
            total_optimized_bytes: total_optimized,
            savings_bytes: total_original.saturating_sub(total_optimized),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

/// Optimizer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerStats {
    pub cached_images: usize,
    pub total_variants: usize,
    pub total_original_bytes: usize,
    pub total_optimized_bytes: usize,
    pub savings_bytes: usize,
}

/// Generate variant filename
fn generate_variant_filename(original: &str, width: u32, format: ImageFormat, hash: &str) -> String {
    let stem = Path::new(original)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    format!("{}-{}w-{}.{}", stem, width, &hash[..8], format.extension())
}

/// Minify SVG content
fn minify_svg(svg: &str) -> String {
    let mut result = svg.to_string();

    // Remove comments
    let comment_re = regex::Regex::new(r"<!--[\s\S]*?-->").unwrap();
    result = comment_re.replace_all(&result, "").to_string();

    // Remove unnecessary whitespace
    let ws_re = regex::Regex::new(r">\s+<").unwrap();
    result = ws_re.replace_all(&result, "><").to_string();

    // Remove empty attributes
    let empty_attr_re = regex::Regex::new(r#"\s+\w+=""\s*"#).unwrap();
    result = empty_attr_re.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

/// Responsive image HTML generator
pub struct ResponsiveImageGenerator {
    /// Base URL for images
    pub base_url: String,
    /// Default sizes attribute
    pub default_sizes: String,
}

impl ResponsiveImageGenerator {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            default_sizes: "(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw".to_string(),
        }
    }

    /// Generate srcset attribute value
    pub fn generate_srcset(&self, image: &OptimizedImage, format: Option<ImageFormat>) -> String {
        let target_format = format.unwrap_or(image.original_format);

        image.variants
            .iter()
            .filter(|v| v.format == target_format)
            .map(|v| format!("{}{} {}w", self.base_url, v.url, v.width))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate picture element HTML
    pub fn generate_picture(&self, image: &OptimizedImage, alt: &str, class: Option<&str>) -> String {
        let class_attr = class.map(|c| format!(" class=\"{}\"", c)).unwrap_or_default();

        let mut html = String::from("<picture>\n");

        // Add WebP sources
        let webp_srcset = self.generate_srcset(image, Some(ImageFormat::WebP));
        if !webp_srcset.is_empty() {
            html.push_str(&format!(
                "  <source type=\"image/webp\" srcset=\"{}\" sizes=\"{}\">\n",
                webp_srcset, self.default_sizes
            ));
        }

        // Add AVIF sources
        let avif_srcset = self.generate_srcset(image, Some(ImageFormat::Avif));
        if !avif_srcset.is_empty() {
            html.push_str(&format!(
                "  <source type=\"image/avif\" srcset=\"{}\" sizes=\"{}\">\n",
                avif_srcset, self.default_sizes
            ));
        }

        // Add original format as fallback
        let original_srcset = self.generate_srcset(image, Some(image.original_format));

        // Find the default src (largest variant)
        let default_src = image.variants
            .iter()
            .filter(|v| v.format == image.original_format)
            .max_by_key(|v| v.width)
            .map(|v| format!("{}{}", self.base_url, v.url))
            .unwrap_or_default();

        // Generate style for aspect ratio
        let aspect_ratio = if image.original_height > 0 {
            format!(
                " style=\"aspect-ratio: {} / {};\"",
                image.original_width, image.original_height
            )
        } else {
            String::new()
        };

        // Add placeholder background
        let placeholder_style = image.dominant_color
            .as_ref()
            .map(|c| format!(" background-color: {};", c))
            .unwrap_or_default();

        html.push_str(&format!(
            "  <img src=\"{}\" srcset=\"{}\" sizes=\"{}\" alt=\"{}\" loading=\"lazy\" decoding=\"async\"{}{}{}>\n",
            default_src,
            original_srcset,
            self.default_sizes,
            alt,
            class_attr,
            aspect_ratio,
            placeholder_style
        ));

        html.push_str("</picture>");
        html
    }

    /// Generate img element with srcset
    pub fn generate_img(&self, image: &OptimizedImage, alt: &str) -> String {
        let srcset = self.generate_srcset(image, None);

        let default_src = image.variants
            .first()
            .map(|v| format!("{}{}", self.base_url, v.url))
            .unwrap_or_default();

        format!(
            "<img src=\"{}\" srcset=\"{}\" sizes=\"{}\" alt=\"{}\" loading=\"lazy\" decoding=\"async\" width=\"{}\" height=\"{}\">",
            default_src,
            srcset,
            self.default_sizes,
            alt,
            image.original_width,
            image.original_height
        )
    }
}

/// Image processing queue for background optimization
pub struct ImageProcessingQueue {
    optimizer: Arc<ImageOptimizer>,
    queue: Arc<RwLock<Vec<ImageJob>>>,
}

#[derive(Debug, Clone)]
pub struct ImageJob {
    pub id: String,
    pub data: Vec<u8>,
    pub filename: String,
    pub priority: u8,
    pub status: JobStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl ImageProcessingQueue {
    pub fn new(optimizer: Arc<ImageOptimizer>) -> Self {
        Self {
            optimizer,
            queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a job to the queue
    pub fn enqueue(&self, data: Vec<u8>, filename: String, priority: u8) -> String {
        let id = blake3::hash(&data).to_hex()[..16].to_string();

        let job = ImageJob {
            id: id.clone(),
            data,
            filename,
            priority,
            status: JobStatus::Pending,
        };

        let mut queue = self.queue.write();
        queue.push(job);
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        id
    }

    /// Get job status
    pub fn get_status(&self, id: &str) -> Option<JobStatus> {
        self.queue.read()
            .iter()
            .find(|j| j.id == id)
            .map(|j| j.status.clone())
    }

    /// Process the next job in the queue
    pub fn process_next(&self) -> Option<Result<OptimizedImage, ImageError>> {
        let job = {
            let mut queue = self.queue.write();
            let idx = queue.iter().position(|j| j.status == JobStatus::Pending)?;
            queue[idx].status = JobStatus::Processing;
            queue[idx].clone()
        };

        let result = self.optimizer.optimize(&job.data, &job.filename);

        // Update job status
        let mut queue = self.queue.write();
        if let Some(j) = queue.iter_mut().find(|j| j.id == job.id) {
            j.status = match &result {
                Ok(_) => JobStatus::Completed,
                Err(e) => JobStatus::Failed(e.to_string()),
            };
        }

        Some(result)
    }

    /// Get queue length
    pub fn queue_length(&self) -> usize {
        self.queue.read().len()
    }

    /// Get pending count
    pub fn pending_count(&self) -> usize {
        self.queue.read().iter().filter(|j| j.status == JobStatus::Pending).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("webp"), Some(ImageFormat::WebP));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_variant_filename_generation() {
        let filename = generate_variant_filename("photo.jpg", 800, ImageFormat::WebP, "abc123def456");
        assert!(filename.contains("photo"));
        assert!(filename.contains("800w"));
        assert!(filename.ends_with(".webp"));
    }

    #[test]
    fn test_svg_minification() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg">
                <!-- Comment -->
                <rect width="" height="100" />
            </svg>
        "#;
        let minified = minify_svg(svg);
        assert!(!minified.contains("<!--"));
        assert!(minified.len() < svg.len());
    }
}
