//! Image optimization functionality
//!
//! Provides image optimization including:
//! - Compression and quality adjustment
//! - WebP conversion
//! - AVIF conversion
//! - Automatic format selection

use image::{DynamicImage, ImageFormat, GenericImageView};
use std::io::Cursor;
use std::path::Path;

use crate::{MediaError, MediaResult};

/// Image optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// JPEG quality (1-100)
    pub jpeg_quality: u8,

    /// PNG compression level (0-9)
    pub png_compression: u8,

    /// WebP quality (1-100)
    pub webp_quality: u8,

    /// Maximum dimension (width or height)
    pub max_dimension: u32,

    /// Strip metadata (EXIF, etc.)
    pub strip_metadata: bool,

    /// Enable progressive JPEG
    pub progressive_jpeg: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            jpeg_quality: 85,
            png_compression: 6,
            webp_quality: 80,
            max_dimension: 4096,
            strip_metadata: true,
            progressive_jpeg: true,
        }
    }
}

/// Image optimizer
pub struct ImageOptimizer {
    config: OptimizationConfig,
}

impl ImageOptimizer {
    /// Create new optimizer with config
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }

    /// Create optimizer with default config
    pub fn default_config() -> Self {
        Self::new(OptimizationConfig::default())
    }

    /// Optimize image from bytes
    pub fn optimize(&self, data: &[u8], format: ImageFormat) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;

        // Resize if too large
        let img = self.resize_if_needed(img);

        // Encode with optimization
        self.encode_optimized(&img, format)
    }

    /// Optimize image file
    pub fn optimize_file(&self, input_path: &Path, output_path: &Path) -> MediaResult<()> {
        let img = image::open(input_path)?;
        let img = self.resize_if_needed(img);

        let format = ImageFormat::from_path(output_path)
            .map_err(|_| MediaError::UnsupportedFormat("Unknown format".to_string()))?;

        let optimized = self.encode_optimized(&img, format)?;
        std::fs::write(output_path, optimized)?;

        Ok(())
    }

    /// Convert image to WebP
    pub fn to_webp(&self, data: &[u8]) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;
        let img = self.resize_if_needed(img);

        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::WebP)?;

        Ok(buffer.into_inner())
    }

    /// Convert image to WebP from file
    pub fn to_webp_file(&self, input_path: &Path, output_path: &Path) -> MediaResult<()> {
        let img = image::open(input_path)?;
        let img = self.resize_if_needed(img);

        img.save_with_format(output_path, ImageFormat::WebP)?;

        Ok(())
    }

    /// Convert image to AVIF (placeholder - requires additional dependencies)
    pub fn to_avif(&self, _data: &[u8]) -> MediaResult<Vec<u8>> {
        // AVIF encoding requires the `avif-encoder` or similar crate
        // This is a placeholder that returns an error
        Err(MediaError::UnsupportedFormat(
            "AVIF encoding not yet implemented. Add avif-encoder crate.".to_string()
        ))
    }

    /// Generate thumbnail
    pub fn generate_thumbnail(&self, data: &[u8], width: u32, height: u32) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;

        // Use thumbnail_exact for exact dimensions, or thumbnail for maintaining aspect ratio
        let thumb = img.thumbnail(width, height);

        let mut buffer = Cursor::new(Vec::new());
        thumb.write_to(&mut buffer, ImageFormat::Jpeg)?;

        Ok(buffer.into_inner())
    }

    /// Generate thumbnail with exact dimensions (crop to fit)
    pub fn generate_thumbnail_exact(&self, data: &[u8], width: u32, height: u32) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;

        // Calculate crop region to maintain aspect ratio
        let (orig_w, orig_h) = img.dimensions();
        let target_ratio = width as f64 / height as f64;
        let orig_ratio = orig_w as f64 / orig_h as f64;

        let (crop_w, crop_h) = if orig_ratio > target_ratio {
            // Image is wider, crop width
            ((orig_h as f64 * target_ratio) as u32, orig_h)
        } else {
            // Image is taller, crop height
            (orig_w, (orig_w as f64 / target_ratio) as u32)
        };

        let crop_x = (orig_w - crop_w) / 2;
        let crop_y = (orig_h - crop_h) / 2;

        let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
        let thumb = cropped.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        let mut buffer = Cursor::new(Vec::new());
        thumb.write_to(&mut buffer, ImageFormat::Jpeg)?;

        Ok(buffer.into_inner())
    }

    /// Resize image maintaining aspect ratio
    pub fn resize(&self, data: &[u8], max_width: u32, max_height: u32) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;
        let resized = img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);

        let format = image::guess_format(data)?;
        self.encode_optimized(&resized, format)
    }

    /// Resize image to exact dimensions
    pub fn resize_exact(&self, data: &[u8], width: u32, height: u32) -> MediaResult<Vec<u8>> {
        let img = image::load_from_memory(data)?;
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        let format = image::guess_format(data)?;
        self.encode_optimized(&resized, format)
    }

    /// Get image dimensions
    pub fn dimensions(data: &[u8]) -> MediaResult<(u32, u32)> {
        let img = image::load_from_memory(data)?;
        Ok(img.dimensions())
    }

    /// Detect image format
    pub fn detect_format(data: &[u8]) -> MediaResult<ImageFormat> {
        Ok(image::guess_format(data)?)
    }

    fn resize_if_needed(&self, img: DynamicImage) -> DynamicImage {
        let (w, h) = img.dimensions();
        let max = self.config.max_dimension;

        if w > max || h > max {
            img.resize(max, max, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    }

    fn encode_optimized(&self, img: &DynamicImage, format: ImageFormat) -> MediaResult<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());

        match format {
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut buffer,
                    self.config.jpeg_quality,
                );
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::Png => {
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut buffer,
                    image::codecs::png::CompressionType::Default,
                    image::codecs::png::FilterType::Adaptive,
                );
                img.write_with_encoder(encoder)?;
            }
            ImageFormat::WebP => {
                img.write_to(&mut buffer, ImageFormat::WebP)?;
            }
            ImageFormat::Gif => {
                img.write_to(&mut buffer, ImageFormat::Gif)?;
            }
            _ => {
                img.write_to(&mut buffer, format)?;
            }
        }

        Ok(buffer.into_inner())
    }
}

/// Batch optimization results
#[derive(Debug)]
pub struct BatchOptimizationResult {
    /// Original file size
    pub original_size: u64,

    /// Optimized file size
    pub optimized_size: u64,

    /// Savings percentage
    pub savings_percent: f64,

    /// Output path
    pub output_path: String,
}

/// Image analysis results
#[derive(Debug, Clone)]
pub struct ImageAnalysis {
    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Format
    pub format: String,

    /// Has alpha channel
    pub has_alpha: bool,

    /// Is animated (GIF)
    pub is_animated: bool,

    /// Color type
    pub color_type: String,

    /// Estimated complexity (affects compression)
    pub complexity: f64,
}

impl ImageAnalysis {
    /// Analyze image data
    pub fn analyze(data: &[u8]) -> MediaResult<Self> {
        let img = image::load_from_memory(data)?;
        let format = image::guess_format(data)?;

        let (width, height) = img.dimensions();
        let color = img.color();

        let has_alpha = matches!(
            color,
            image::ColorType::La8 | image::ColorType::La16 |
            image::ColorType::Rgba8 | image::ColorType::Rgba16
        );

        // Simple complexity estimation based on unique colors in a sample
        let complexity = Self::estimate_complexity(&img);

        Ok(Self {
            width,
            height,
            format: format!("{:?}", format),
            has_alpha,
            is_animated: matches!(format, ImageFormat::Gif),
            color_type: format!("{:?}", color),
            complexity,
        })
    }

    fn estimate_complexity(img: &DynamicImage) -> f64 {
        // Sample the image and count unique colors
        let thumb = img.thumbnail(100, 100);
        let rgb = thumb.to_rgb8();

        let mut colors = std::collections::HashSet::new();
        for pixel in rgb.pixels() {
            // Quantize colors to reduce noise
            let r = pixel[0] / 16;
            let g = pixel[1] / 16;
            let b = pixel[2] / 16;
            colors.insert((r, g, b));
        }

        // More unique colors = more complex image
        let total_pixels = (thumb.width() * thumb.height()) as f64;
        colors.len() as f64 / total_pixels
    }

    /// Recommend optimal format based on analysis
    pub fn recommend_format(&self) -> &str {
        if self.is_animated {
            "gif" // Or "webp" if animation support needed
        } else if self.has_alpha {
            if self.complexity < 0.1 {
                "png" // Simple images with transparency
            } else {
                "webp" // Complex images with transparency
            }
        } else if self.complexity < 0.1 {
            "png" // Simple graphics, logos
        } else {
            "jpeg" // Photos and complex images
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_default() {
        let config = OptimizationConfig::default();
        assert_eq!(config.jpeg_quality, 85);
        assert_eq!(config.webp_quality, 80);
    }
}
