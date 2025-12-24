//! Responsive Image System
//!
//! Generates responsive image srcsets and handles image optimization.

use image::{DynamicImage, GenericImageView, ImageFormat};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// Image processing errors
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Image not found: {0}")]
    NotFound(String),
}

/// Image size configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageSize {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub crop: bool,
}

/// Default WordPress-like image sizes
pub fn default_image_sizes() -> Vec<ImageSize> {
    vec![
        ImageSize { name: "thumbnail".to_string(), width: 150, height: 150, crop: true },
        ImageSize { name: "medium".to_string(), width: 300, height: 300, crop: false },
        ImageSize { name: "medium_large".to_string(), width: 768, height: 0, crop: false },
        ImageSize { name: "large".to_string(), width: 1024, height: 1024, crop: false },
        ImageSize { name: "1536x1536".to_string(), width: 1536, height: 1536, crop: false },
        ImageSize { name: "2048x2048".to_string(), width: 2048, height: 2048, crop: false },
    ]
}

/// Responsive image generator
pub struct ResponsiveImageGenerator {
    /// Registered image sizes
    sizes: Arc<RwLock<Vec<ImageSize>>>,
    /// Output directory
    output_dir: PathBuf,
    /// Quality settings
    quality: ImageQuality,
    /// Generated images cache
    cache: Arc<RwLock<HashMap<String, GeneratedImageSet>>>,
}

/// Image quality settings
#[derive(Debug, Clone)]
pub struct ImageQuality {
    pub jpeg_quality: u8,
    pub png_compression: u8,
    pub webp_quality: u8,
    pub avif_quality: u8,
}

impl Default for ImageQuality {
    fn default() -> Self {
        Self {
            jpeg_quality: 82,
            png_compression: 6,
            webp_quality: 80,
            avif_quality: 75,
        }
    }
}

/// Set of generated images for different sizes
#[derive(Debug, Clone, serde::Serialize)]
pub struct GeneratedImageSet {
    pub original: ImageInfo,
    pub sizes: HashMap<String, ImageInfo>,
    pub srcset: String,
    pub sizes_attr: String,
}

/// Information about a single generated image
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
    pub format: String,
}

impl ResponsiveImageGenerator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            sizes: Arc::new(RwLock::new(default_image_sizes())),
            output_dir,
            quality: ImageQuality::default(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_quality(mut self, quality: ImageQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Register additional image size
    pub fn add_size(&self, size: ImageSize) {
        self.sizes.write().push(size);
    }

    /// Remove an image size
    pub fn remove_size(&self, name: &str) {
        self.sizes.write().retain(|s| s.name != name);
    }

    /// Generate all sizes for an image
    pub async fn generate(&self, source: &Path, base_url: &str) -> Result<GeneratedImageSet, ImageError> {
        let cache_key = source.to_string_lossy().to_string();

        // Check cache
        if let Some(cached) = self.cache.read().get(&cache_key) {
            return Ok(cached.clone());
        }

        // Load source image
        let img = self.load_image(source).await?;
        let (orig_width, orig_height) = img.dimensions();

        // Create output directory
        let output_subdir = self.output_dir.join(
            source.file_stem().unwrap().to_string_lossy().to_string()
        );
        fs::create_dir_all(&output_subdir).await?;

        // Save original (optimized)
        let original_info = self.save_optimized(&img, &output_subdir, "original", base_url).await?;

        // Generate each size
        let mut sizes_map = HashMap::new();
        let mut srcset_parts = Vec::new();

        // Add original to srcset
        srcset_parts.push(format!("{} {}w", original_info.url, orig_width));

        let sizes = self.sizes.read().clone();
        for size in &sizes {
            // Skip if original is smaller
            if size.width > 0 && orig_width <= size.width && (size.height == 0 || orig_height <= size.height) {
                continue;
            }

            let resized = self.resize_image(&img, size);
            if let Some(resized) = resized {
                let (w, _h) = resized.dimensions();
                let info = self.save_optimized(&resized, &output_subdir, &size.name, base_url).await?;

                srcset_parts.push(format!("{} {}w", info.url, w));
                sizes_map.insert(size.name.clone(), info);
            }
        }

        // Sort srcset by width
        srcset_parts.sort_by_key(|s| {
            s.split_whitespace()
                .last()
                .and_then(|w| w.trim_end_matches('w').parse::<u32>().ok())
                .unwrap_or(0)
        });

        // Generate sizes attribute
        let sizes_attr = self.generate_sizes_attr(&sizes);

        let result = GeneratedImageSet {
            original: original_info,
            sizes: sizes_map,
            srcset: srcset_parts.join(", "),
            sizes_attr,
        };

        // Cache result
        self.cache.write().insert(cache_key, result.clone());

        Ok(result)
    }

    async fn load_image(&self, path: &Path) -> Result<DynamicImage, ImageError> {
        let data = fs::read(path).await?;
        image::load_from_memory(&data)
            .map_err(|e| ImageError::ImageProcessing(e.to_string()))
    }

    fn resize_image(&self, img: &DynamicImage, size: &ImageSize) -> Option<DynamicImage> {
        let (orig_width, orig_height) = img.dimensions();

        if size.crop {
            // Crop to exact dimensions
            Some(img.resize_to_fill(
                size.width,
                size.height,
                image::imageops::FilterType::Lanczos3,
            ))
        } else {
            // Calculate dimensions maintaining aspect ratio
            let (new_width, new_height) = if size.height == 0 {
                // Only width specified
                let ratio = size.width as f32 / orig_width as f32;
                (size.width, (orig_height as f32 * ratio) as u32)
            } else if size.width == 0 {
                // Only height specified
                let ratio = size.height as f32 / orig_height as f32;
                ((orig_width as f32 * ratio) as u32, size.height)
            } else {
                // Both specified, fit within
                let width_ratio = size.width as f32 / orig_width as f32;
                let height_ratio = size.height as f32 / orig_height as f32;
                let ratio = width_ratio.min(height_ratio);
                (
                    (orig_width as f32 * ratio) as u32,
                    (orig_height as f32 * ratio) as u32,
                )
            };

            if new_width >= orig_width && new_height >= orig_height {
                return None; // Don't upscale
            }

            Some(img.resize(
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            ))
        }
    }

    async fn save_optimized(
        &self,
        img: &DynamicImage,
        output_dir: &Path,
        name: &str,
        base_url: &str,
    ) -> Result<ImageInfo, ImageError> {
        let (width, height) = img.dimensions();

        // Save as WebP (modern format, good compression)
        let webp_path = output_dir.join(format!("{}.webp", name));
        let webp_data = self.encode_webp(img)?;
        fs::write(&webp_path, &webp_data).await?;

        let url = format!(
            "{}/{}/{}",
            base_url,
            output_dir.file_name().unwrap().to_string_lossy(),
            webp_path.file_name().unwrap().to_string_lossy()
        );

        Ok(ImageInfo {
            path: webp_path,
            url,
            width,
            height,
            file_size: webp_data.len() as u64,
            format: "webp".to_string(),
        })
    }

    fn encode_webp(&self, img: &DynamicImage) -> Result<Vec<u8>, ImageError> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::WebP)
            .map_err(|e| ImageError::ImageProcessing(e.to_string()))?;
        Ok(buffer.into_inner())
    }

    fn generate_sizes_attr(&self, sizes: &[ImageSize]) -> String {
        // Generate responsive sizes attribute
        let mut parts = Vec::new();

        // Sort by width descending
        let mut sorted: Vec<_> = sizes.iter().filter(|s| s.width > 0).collect();
        sorted.sort_by(|a, b| b.width.cmp(&a.width));

        for (i, size) in sorted.iter().enumerate() {
            if i == sorted.len() - 1 {
                parts.push(format!("{}px", size.width));
            } else {
                parts.push(format!("(max-width: {}px) {}px", size.width + 100, size.width));
            }
        }

        parts.join(", ")
    }

    /// Generate srcset HTML attribute
    pub fn generate_srcset_attr(&self, image_set: &GeneratedImageSet) -> String {
        format!(r#"srcset="{}""#, image_set.srcset)
    }

    /// Generate complete img tag
    pub fn generate_img_tag(
        &self,
        image_set: &GeneratedImageSet,
        alt: &str,
        class: Option<&str>,
        lazy: bool,
    ) -> String {
        let mut attrs = vec![
            format!(r#"src="{}""#, image_set.original.url),
            format!(r#"srcset="{}""#, image_set.srcset),
            format!(r#"sizes="{}""#, image_set.sizes_attr),
            format!(r#"width="{}""#, image_set.original.width),
            format!(r#"height="{}""#, image_set.original.height),
            format!(r#"alt="{}""#, html_escape(alt)),
        ];

        if let Some(class) = class {
            attrs.push(format!(r#"class="{}""#, class));
        }

        if lazy {
            attrs.push(r#"loading="lazy""#.to_string());
            attrs.push(r#"decoding="async""#.to_string());
        }

        format!("<img {}>", attrs.join(" "))
    }

    /// Generate picture element with format fallbacks
    pub fn generate_picture_tag(
        &self,
        image_set: &GeneratedImageSet,
        alt: &str,
        class: Option<&str>,
        lazy: bool,
    ) -> String {
        let img_tag = self.generate_img_tag(image_set, alt, class, lazy);

        format!(
            r#"<picture>
  <source type="image/webp" srcset="{}">
  {}
</picture>"#,
            image_set.srcset,
            img_tag
        )
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

/// HTML escape helper
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Art direction for responsive images
#[derive(Debug, Clone)]
pub struct ArtDirection {
    pub breakpoints: Vec<ArtDirectionBreakpoint>,
}

/// Art direction breakpoint
#[derive(Debug, Clone)]
pub struct ArtDirectionBreakpoint {
    pub media_query: String,
    pub source: PathBuf,
    pub crop: Option<CropSettings>,
}

/// Crop settings for art direction
#[derive(Debug, Clone)]
pub struct CropSettings {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ArtDirection {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
        }
    }

    pub fn add_breakpoint(mut self, breakpoint: ArtDirectionBreakpoint) -> Self {
        self.breakpoints.push(breakpoint);
        self
    }

    /// Generate picture element with art direction
    pub async fn generate_picture(
        &self,
        generator: &ResponsiveImageGenerator,
        default_source: &Path,
        alt: &str,
        base_url: &str,
    ) -> Result<String, ImageError> {
        let mut sources = Vec::new();

        // Generate sources for each breakpoint
        for bp in &self.breakpoints {
            let image_set = generator.generate(&bp.source, base_url).await?;
            sources.push(format!(
                r#"<source media="{}" srcset="{}">"#,
                bp.media_query,
                image_set.srcset
            ));
        }

        // Generate default image
        let default_set = generator.generate(default_source, base_url).await?;

        let img_tag = format!(
            r#"<img src="{}" srcset="{}" sizes="{}" alt="{}" loading="lazy">"#,
            default_set.original.url,
            default_set.srcset,
            default_set.sizes_attr,
            html_escape(alt)
        );

        Ok(format!(
            "<picture>\n  {}\n  {}\n</picture>",
            sources.join("\n  "),
            img_tag
        ))
    }
}

impl Default for ArtDirection {
    fn default() -> Self {
        Self::new()
    }
}

/// Image focal point for smart cropping
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FocalPoint {
    pub x: f32, // 0.0 to 1.0
    pub y: f32, // 0.0 to 1.0
}

impl Default for FocalPoint {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 } // Center
    }
}

/// Smart crop using focal point
pub fn smart_crop(img: &DynamicImage, target_width: u32, target_height: u32, focal: FocalPoint) -> DynamicImage {
    let (orig_width, orig_height) = img.dimensions();

    // Calculate crop dimensions
    let target_ratio = target_width as f32 / target_height as f32;
    let orig_ratio = orig_width as f32 / orig_height as f32;

    let (crop_width, crop_height) = if orig_ratio > target_ratio {
        // Original is wider, crop sides
        let new_width = (orig_height as f32 * target_ratio) as u32;
        (new_width, orig_height)
    } else {
        // Original is taller, crop top/bottom
        let new_height = (orig_width as f32 / target_ratio) as u32;
        (orig_width, new_height)
    };

    // Calculate crop position based on focal point
    let max_x = orig_width.saturating_sub(crop_width);
    let max_y = orig_height.saturating_sub(crop_height);

    let crop_x = (max_x as f32 * focal.x) as u32;
    let crop_y = (max_y as f32 * focal.y) as u32;

    // Crop and resize
    let cropped = img.crop_imm(crop_x, crop_y, crop_width, crop_height);
    cropped.resize_exact(target_width, target_height, image::imageops::FilterType::Lanczos3)
}

/// Placeholder image generator
pub struct PlaceholderGenerator;

impl PlaceholderGenerator {
    /// Generate a low-quality image placeholder (LQIP)
    pub fn generate_lqip(img: &DynamicImage, size: u32) -> Result<String, ImageError> {
        let resized = img.resize(size, size, image::imageops::FilterType::Nearest);

        let mut buffer = std::io::Cursor::new(Vec::new());
        resized.write_to(&mut buffer, ImageFormat::Jpeg)
            .map_err(|e| ImageError::ImageProcessing(e.to_string()))?;

        let base64 = base64_encode(&buffer.into_inner());
        Ok(format!("data:image/jpeg;base64,{}", base64))
    }

    /// Generate a dominant color placeholder
    pub fn generate_dominant_color(img: &DynamicImage) -> String {
        let tiny = img.resize(1, 1, image::imageops::FilterType::Nearest);
        let pixel = tiny.get_pixel(0, 0);

        format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
    }

    /// Generate an SVG blur placeholder
    pub fn generate_svg_blur(img: &DynamicImage, width: u32, height: u32) -> Result<String, ImageError> {
        let tiny = img.resize(20, 20, image::imageops::FilterType::Gaussian);

        let mut buffer = std::io::Cursor::new(Vec::new());
        tiny.write_to(&mut buffer, ImageFormat::Jpeg)
            .map_err(|e| ImageError::ImageProcessing(e.to_string()))?;

        let base64 = base64_encode(&buffer.into_inner());

        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <filter id="blur" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
    <feGaussianBlur stdDeviation="20 20" edgeMode="duplicate"/>
    <feComponentTransfer>
      <feFuncA type="discrete" tableValues="1 1"/>
    </feComponentTransfer>
  </filter>
  <image filter="url(#blur)" x="0" y="0" width="100%" height="100%" href="data:image/jpeg;base64,{}"/>
</svg>"#,
            width, height, base64
        );

        Ok(format!(
            "data:image/svg+xml;base64,{}",
            base64_encode(svg.as_bytes())
        ))
    }
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sizes() {
        let sizes = default_image_sizes();
        assert!(sizes.iter().any(|s| s.name == "thumbnail"));
        assert!(sizes.iter().any(|s| s.name == "medium"));
        assert!(sizes.iter().any(|s| s.name == "large"));
    }

    #[test]
    fn test_focal_point_default() {
        let focal = FocalPoint::default();
        assert!((focal.x - 0.5).abs() < f32::EPSILON);
        assert!((focal.y - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape(r#"He said "hi""#), "He said &quot;hi&quot;");
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode(b"Hello");
        assert_eq!(encoded, "SGVsbG8=");
    }
}
