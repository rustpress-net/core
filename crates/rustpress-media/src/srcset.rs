//! Responsive image srcset generation
//!
//! Provides functionality for generating responsive image variants
//! and srcset attributes for optimal image loading.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{MediaError, MediaResult, MediaItem};
use crate::image_optimizer::ImageOptimizer;

/// Srcset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcsetConfig {
    /// Breakpoint widths to generate
    pub widths: Vec<u32>,

    /// Quality for each size (default 85)
    pub quality: u8,

    /// Generate WebP versions
    pub generate_webp: bool,

    /// Generate AVIF versions
    pub generate_avif: bool,

    /// Maximum width
    pub max_width: u32,

    /// Maintain aspect ratio
    pub maintain_aspect_ratio: bool,
}

impl Default for SrcsetConfig {
    fn default() -> Self {
        Self {
            widths: vec![320, 640, 768, 1024, 1280, 1536, 1920],
            quality: 85,
            generate_webp: true,
            generate_avif: false,
            max_width: 2560,
            maintain_aspect_ratio: true,
        }
    }
}

/// Srcset generator service
pub struct SrcsetGenerator {
    config: SrcsetConfig,
    optimizer: ImageOptimizer,
}

impl SrcsetGenerator {
    /// Create new generator with config
    pub fn new(config: SrcsetConfig) -> Self {
        Self {
            config,
            optimizer: ImageOptimizer::default_config(),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(SrcsetConfig::default())
    }

    /// Generate srcset variants from image data
    pub fn generate_variants(&self, data: &[u8], original_width: u32) -> MediaResult<Vec<ImageVariant>> {
        let mut variants = Vec::new();

        // Get applicable widths (smaller than original)
        let widths: Vec<u32> = self.config.widths
            .iter()
            .filter(|&&w| w <= original_width && w <= self.config.max_width)
            .copied()
            .collect();

        // Always include the original if within max_width
        let widths = if original_width <= self.config.max_width && !widths.contains(&original_width) {
            let mut w = widths;
            w.push(original_width);
            w.sort();
            w
        } else {
            widths
        };

        for width in widths {
            // Generate JPEG variant
            let jpeg_data = self.optimizer.resize(data, width, u32::MAX)?;
            variants.push(ImageVariant {
                width,
                height: self.calculate_height(data, width)?,
                format: ImageVariantFormat::Jpeg,
                data: jpeg_data,
                size_descriptor: format!("{}w", width),
            });

            // Generate WebP variant
            if self.config.generate_webp {
                let resized = self.optimizer.resize(data, width, u32::MAX)?;
                let webp_data = self.optimizer.to_webp(&resized)?;
                variants.push(ImageVariant {
                    width,
                    height: self.calculate_height(data, width)?,
                    format: ImageVariantFormat::WebP,
                    data: webp_data,
                    size_descriptor: format!("{}w", width),
                });
            }
        }

        Ok(variants)
    }

    /// Generate srcset attribute string
    pub fn generate_srcset(&self, base_url: &str, filename: &str, format: ImageVariantFormat) -> String {
        let ext = match format {
            ImageVariantFormat::Jpeg => "jpg",
            ImageVariantFormat::Png => "png",
            ImageVariantFormat::WebP => "webp",
            ImageVariantFormat::Avif => "avif",
        };

        let name = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename);

        self.config.widths
            .iter()
            .map(|w| format!("{}/{}-{}.{} {}w", base_url, name, w, ext, w))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate sizes attribute for common layouts
    pub fn generate_sizes(&self, layout: SrcsetLayout) -> String {
        match layout {
            SrcsetLayout::FullWidth => {
                "100vw".to_string()
            }
            SrcsetLayout::Container => {
                "(max-width: 640px) 100vw, (max-width: 1024px) 80vw, 1200px".to_string()
            }
            SrcsetLayout::HalfWidth => {
                "(max-width: 640px) 100vw, 50vw".to_string()
            }
            SrcsetLayout::ThirdWidth => {
                "(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw".to_string()
            }
            SrcsetLayout::Thumbnail => {
                "150px".to_string()
            }
            SrcsetLayout::Custom(sizes) => sizes,
        }
    }

    /// Generate complete picture element HTML
    pub fn generate_picture_html(
        &self,
        media: &MediaItem,
        base_url: &str,
        layout: SrcsetLayout,
        lazy: bool,
    ) -> String {
        let filename = &media.filename;
        let alt = &media.alt_text;
        let sizes = self.generate_sizes(layout);

        let mut html = String::from("<picture>");

        // WebP source
        if self.config.generate_webp {
            let webp_srcset = self.generate_srcset(base_url, filename, ImageVariantFormat::WebP);
            html.push_str(&format!(
                "<source type=\"image/webp\" srcset=\"{}\" sizes=\"{}\">",
                webp_srcset, sizes
            ));
        }

        // AVIF source
        if self.config.generate_avif {
            let avif_srcset = self.generate_srcset(base_url, filename, ImageVariantFormat::Avif);
            html.push_str(&format!(
                "<source type=\"image/avif\" srcset=\"{}\" sizes=\"{}\">",
                avif_srcset, sizes
            ));
        }

        // Fallback img with JPEG srcset
        let jpeg_srcset = self.generate_srcset(base_url, filename, ImageVariantFormat::Jpeg);
        let loading = if lazy { " loading=\"lazy\"" } else { "" };
        let decoding = if lazy { " decoding=\"async\"" } else { "" };

        html.push_str(&format!(
            "<img src=\"{}\" srcset=\"{}\" sizes=\"{}\" alt=\"{}\"{}{}",
            media.url, jpeg_srcset, sizes, escape_html(alt), loading, decoding
        ));

        if let (Some(w), Some(h)) = (media.width, media.height) {
            html.push_str(&format!(" width=\"{}\" height=\"{}\"", w, h));
        }

        html.push_str(">");
        html.push_str("</picture>");

        html
    }

    /// Calculate height for given width maintaining aspect ratio
    fn calculate_height(&self, data: &[u8], target_width: u32) -> MediaResult<u32> {
        let (orig_width, orig_height) = ImageOptimizer::dimensions(data)?;
        let ratio = orig_height as f64 / orig_width as f64;
        Ok((target_width as f64 * ratio) as u32)
    }
}

/// Image variant
#[derive(Debug, Clone)]
pub struct ImageVariant {
    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Format
    pub format: ImageVariantFormat,

    /// Image data
    pub data: Vec<u8>,

    /// Size descriptor (e.g., "640w" or "2x")
    pub size_descriptor: String,
}

/// Image variant formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageVariantFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
}

impl ImageVariantFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Avif => "avif",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::Avif => "image/avif",
        }
    }
}

/// Common srcset layouts
#[derive(Debug, Clone)]
pub enum SrcsetLayout {
    /// Full viewport width
    FullWidth,

    /// Container width (with max-width)
    Container,

    /// Half viewport width
    HalfWidth,

    /// Third viewport width
    ThirdWidth,

    /// Thumbnail size
    Thumbnail,

    /// Custom sizes attribute
    Custom(String),
}

/// Art direction breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtDirectionBreakpoint {
    /// Media query
    pub media: String,

    /// Image URL for this breakpoint
    pub src: String,

    /// Srcset for this breakpoint
    pub srcset: Option<String>,
}

/// Generate art-directed picture element
pub fn generate_art_directed_picture(
    breakpoints: &[ArtDirectionBreakpoint],
    fallback: &MediaItem,
    alt: &str,
    lazy: bool,
) -> String {
    let mut html = String::from("<picture>");

    // Add source elements for each breakpoint
    for bp in breakpoints {
        html.push_str(&format!("<source media=\"{}\"", bp.media));

        if let Some(srcset) = &bp.srcset {
            html.push_str(&format!(" srcset=\"{}\"", srcset));
        } else {
            html.push_str(&format!(" srcset=\"{}\"", bp.src));
        }

        html.push_str(">");
    }

    // Fallback img
    let loading = if lazy { " loading=\"lazy\"" } else { "" };
    let decoding = if lazy { " decoding=\"async\"" } else { "" };

    html.push_str(&format!(
        "<img src=\"{}\" alt=\"{}\"{}{}",
        fallback.url, escape_html(alt), loading, decoding
    ));

    if let (Some(w), Some(h)) = (fallback.width, fallback.height) {
        html.push_str(&format!(" width=\"{}\" height=\"{}\"", w, h));
    }

    html.push_str(">");
    html.push_str("</picture>");

    html
}

/// Simple HTML escaping
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srcset_config_default() {
        let config = SrcsetConfig::default();
        assert!(config.widths.contains(&320));
        assert!(config.widths.contains(&1920));
        assert!(config.generate_webp);
    }

    #[test]
    fn test_generate_sizes() {
        let generator = SrcsetGenerator::default_config();

        assert_eq!(generator.generate_sizes(SrcsetLayout::FullWidth), "100vw");
        assert_eq!(generator.generate_sizes(SrcsetLayout::Thumbnail), "150px");
    }

    #[test]
    fn test_variant_format() {
        assert_eq!(ImageVariantFormat::WebP.extension(), "webp");
        assert_eq!(ImageVariantFormat::WebP.mime_type(), "image/webp");
    }
}
