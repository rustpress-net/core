//! # Featured Image System
//!
//! Featured image management with focal point selection.
//!
//! Features:
//! - Featured image attachment
//! - Focal point selection for smart cropping
//! - Responsive image generation
//! - Open Graph and social media images
//! - Fallback image handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Featured image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedImage {
    /// Attachment/media ID
    pub attachment_id: i64,

    /// Original image URL
    pub url: String,

    /// Alt text
    pub alt: String,

    /// Title
    pub title: String,

    /// Caption
    pub caption: String,

    /// Original dimensions
    pub width: u32,
    pub height: u32,

    /// Focal point for smart cropping
    pub focal_point: FocalPoint,

    /// Available sizes
    pub sizes: HashMap<String, ImageSize>,

    /// MIME type
    pub mime_type: String,

    /// File size in bytes
    pub file_size: u64,

    /// Custom meta
    pub meta: HashMap<String, String>,
}

impl FeaturedImage {
    pub fn new(attachment_id: i64, url: &str) -> Self {
        Self {
            attachment_id,
            url: url.to_string(),
            alt: String::new(),
            title: String::new(),
            caption: String::new(),
            width: 0,
            height: 0,
            focal_point: FocalPoint::center(),
            sizes: HashMap::new(),
            mime_type: "image/jpeg".to_string(),
            file_size: 0,
            meta: HashMap::new(),
        }
    }

    /// Get URL for specific size
    pub fn get_size_url(&self, size: &str) -> Option<&str> {
        self.sizes.get(size).map(|s| s.url.as_str())
    }

    /// Get URL for size or fallback to original
    pub fn get_url_or_original(&self, size: &str) -> &str {
        self.get_size_url(size).unwrap_or(&self.url)
    }

    /// Get srcset attribute value
    pub fn srcset(&self) -> String {
        let mut parts: Vec<String> = self.sizes
            .values()
            .map(|size| format!("{} {}w", size.url, size.width))
            .collect();

        parts.push(format!("{} {}w", self.url, self.width));
        parts.join(", ")
    }

    /// Get sizes attribute for responsive images
    pub fn sizes_attr(&self, default_sizes: &str) -> String {
        if default_sizes.is_empty() {
            "(max-width: 768px) 100vw, 768px".to_string()
        } else {
            default_sizes.to_string()
        }
    }

    /// Generate responsive image HTML
    pub fn to_html(&self, class: &str, sizes: &str) -> String {
        let alt = if self.alt.is_empty() { &self.title } else { &self.alt };

        format!(
            r#"<img src="{}" srcset="{}" sizes="{}" alt="{}" width="{}" height="{}" class="{}" loading="lazy">"#,
            self.get_url_or_original("large"),
            self.srcset(),
            self.sizes_attr(sizes),
            alt,
            self.width,
            self.height,
            class
        )
    }

    /// Generate picture element with WebP support
    pub fn to_picture(&self, class: &str, sizes: &str) -> String {
        let alt = if self.alt.is_empty() { &self.title } else { &self.alt };

        let webp_srcset = self.sizes.values()
            .filter_map(|size| {
                size.webp_url.as_ref().map(|webp| format!("{} {}w", webp, size.width))
            })
            .collect::<Vec<_>>()
            .join(", ");

        let mut html = String::from("<picture>");

        if !webp_srcset.is_empty() {
            html.push_str(&format!(
                r#"<source type="image/webp" srcset="{}" sizes="{}">"#,
                webp_srcset, self.sizes_attr(sizes)
            ));
        }

        html.push_str(&format!(
            r#"<source srcset="{}" sizes="{}">"#,
            self.srcset(), self.sizes_attr(sizes)
        ));

        html.push_str(&format!(
            r#"<img src="{}" alt="{}" width="{}" height="{}" class="{}" loading="lazy">"#,
            self.get_url_or_original("large"),
            alt,
            self.width,
            self.height,
            class
        ));

        html.push_str("</picture>");
        html
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f64 / self.height as f64
    }

    /// Check if image is landscape
    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    /// Check if image is portrait
    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }
}

/// Image size variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSize {
    pub name: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub crop: bool,
    pub webp_url: Option<String>,
}

/// Focal point for smart cropping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FocalPoint {
    /// X position as percentage (0.0 to 1.0)
    pub x: f64,

    /// Y position as percentage (0.0 to 1.0)
    pub y: f64,
}

impl Default for FocalPoint {
    fn default() -> Self {
        Self::center()
    }
}

impl FocalPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }

    pub fn center() -> Self {
        Self { x: 0.5, y: 0.5 }
    }

    pub fn top() -> Self {
        Self { x: 0.5, y: 0.0 }
    }

    pub fn bottom() -> Self {
        Self { x: 0.5, y: 1.0 }
    }

    pub fn left() -> Self {
        Self { x: 0.0, y: 0.5 }
    }

    pub fn right() -> Self {
        Self { x: 1.0, y: 0.5 }
    }

    /// Calculate crop position for target dimensions
    pub fn calculate_crop(&self, source_w: u32, source_h: u32, target_w: u32, target_h: u32) -> CropPosition {
        let source_ratio = source_w as f64 / source_h as f64;
        let target_ratio = target_w as f64 / target_h as f64;

        let (scaled_w, scaled_h) = if source_ratio > target_ratio {
            // Source is wider, crop horizontally
            let scaled_h = source_h;
            let scaled_w = (source_h as f64 * target_ratio) as u32;
            (scaled_w, scaled_h)
        } else {
            // Source is taller, crop vertically
            let scaled_w = source_w;
            let scaled_h = (source_w as f64 / target_ratio) as u32;
            (scaled_w, scaled_h)
        };

        // Calculate offset based on focal point
        let max_offset_x = source_w.saturating_sub(scaled_w);
        let max_offset_y = source_h.saturating_sub(scaled_h);

        let offset_x = (max_offset_x as f64 * self.x) as u32;
        let offset_y = (max_offset_y as f64 * self.y) as u32;

        CropPosition {
            x: offset_x,
            y: offset_y,
            width: scaled_w,
            height: scaled_h,
        }
    }

    /// Get CSS background-position value
    pub fn to_css(&self) -> String {
        format!("{}% {}%", (self.x * 100.0).round(), (self.y * 100.0).round())
    }

    /// Get CSS object-position value
    pub fn to_object_position(&self) -> String {
        self.to_css()
    }
}

/// Calculated crop position
#[derive(Debug, Clone, Copy)]
pub struct CropPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Social media image specifications
#[derive(Debug, Clone)]
pub struct SocialImageSpec {
    pub platform: SocialPlatform,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub aspect_ratio: f64,
}

/// Social media platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialPlatform {
    OpenGraph,
    TwitterCard,
    Facebook,
    LinkedIn,
    Pinterest,
}

impl SocialPlatform {
    /// Get recommended image specifications
    pub fn get_spec(&self) -> SocialImageSpec {
        match self {
            Self::OpenGraph | Self::Facebook => SocialImageSpec {
                platform: *self,
                width: 1200,
                height: 630,
                min_width: 600,
                min_height: 315,
                aspect_ratio: 1.91,
            },
            Self::TwitterCard => SocialImageSpec {
                platform: *self,
                width: 1200,
                height: 600,
                min_width: 600,
                min_height: 300,
                aspect_ratio: 2.0,
            },
            Self::LinkedIn => SocialImageSpec {
                platform: *self,
                width: 1200,
                height: 627,
                min_width: 400,
                min_height: 400,
                aspect_ratio: 1.91,
            },
            Self::Pinterest => SocialImageSpec {
                platform: *self,
                width: 1000,
                height: 1500,
                min_width: 600,
                min_height: 900,
                aspect_ratio: 0.67,
            },
        }
    }

    /// Get size name for this platform
    pub fn size_name(&self) -> &str {
        match self {
            Self::OpenGraph => "og_image",
            Self::TwitterCard => "twitter_card",
            Self::Facebook => "facebook_share",
            Self::LinkedIn => "linkedin_share",
            Self::Pinterest => "pinterest_pin",
        }
    }
}

/// Featured image manager
pub struct FeaturedImageManager {
    /// Default featured image
    default_image: Option<FeaturedImage>,

    /// Fallback images by post type
    fallbacks: HashMap<String, FeaturedImage>,

    /// Required sizes to generate
    required_sizes: Vec<ImageSizeDefinition>,
}

/// Image size definition
#[derive(Debug, Clone)]
pub struct ImageSizeDefinition {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub crop: bool,
}

impl Default for FeaturedImageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FeaturedImageManager {
    pub fn new() -> Self {
        let required_sizes = vec![
            ImageSizeDefinition {
                name: "thumbnail".to_string(),
                width: 150,
                height: 150,
                crop: true,
            },
            ImageSizeDefinition {
                name: "medium".to_string(),
                width: 300,
                height: 300,
                crop: false,
            },
            ImageSizeDefinition {
                name: "large".to_string(),
                width: 1024,
                height: 1024,
                crop: false,
            },
            ImageSizeDefinition {
                name: "og_image".to_string(),
                width: 1200,
                height: 630,
                crop: true,
            },
            ImageSizeDefinition {
                name: "twitter_card".to_string(),
                width: 1200,
                height: 600,
                crop: true,
            },
        ];

        Self {
            default_image: None,
            fallbacks: HashMap::new(),
            required_sizes,
        }
    }

    /// Set default featured image
    pub fn set_default(&mut self, image: FeaturedImage) {
        self.default_image = Some(image);
    }

    /// Set fallback for post type
    pub fn set_fallback(&mut self, post_type: &str, image: FeaturedImage) {
        self.fallbacks.insert(post_type.to_string(), image);
    }

    /// Get featured image or fallback
    pub fn get_or_fallback(&self, image: Option<&FeaturedImage>, post_type: &str) -> Option<&FeaturedImage> {
        image
            .or_else(|| self.fallbacks.get(post_type))
            .or(self.default_image.as_ref())
    }

    /// Get required sizes
    pub fn get_required_sizes(&self) -> &[ImageSizeDefinition] {
        &self.required_sizes
    }

    /// Add required size
    pub fn add_size(&mut self, definition: ImageSizeDefinition) {
        self.required_sizes.push(definition);
    }

    /// Generate Open Graph meta tags
    pub fn generate_og_tags(&self, image: &FeaturedImage, page_url: &str) -> String {
        let og_url = image.get_url_or_original("og_image");

        format!(
            r#"<meta property="og:image" content="{}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:image:alt" content="{}">
<meta property="og:url" content="{}">"#,
            og_url,
            image.alt,
            page_url
        )
    }

    /// Generate Twitter Card meta tags
    pub fn generate_twitter_tags(&self, image: &FeaturedImage) -> String {
        let twitter_url = image.get_url_or_original("twitter_card");

        format!(
            r#"<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:image" content="{}">
<meta name="twitter:image:alt" content="{}">"#,
            twitter_url,
            image.alt
        )
    }

    /// Generate all social meta tags
    pub fn generate_social_tags(&self, image: &FeaturedImage, page_url: &str) -> String {
        let mut tags = self.generate_og_tags(image, page_url);
        tags.push('\n');
        tags.push_str(&self.generate_twitter_tags(image));
        tags
    }
}

/// Generate responsive featured image HTML
pub fn featured_image_html(image: &FeaturedImage, class: &str) -> String {
    image.to_html(class, "(max-width: 768px) 100vw, 768px")
}

/// Generate featured image with caption
pub fn featured_image_with_caption(image: &FeaturedImage, class: &str) -> String {
    let mut html = String::from("<figure class=\"featured-image\">");
    html.push_str(&image.to_html(class, "(max-width: 768px) 100vw, 768px"));

    if !image.caption.is_empty() {
        html.push_str(&format!("<figcaption>{}</figcaption>", image.caption));
    }

    html.push_str("</figure>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focal_point_crop() {
        let fp = FocalPoint::center();
        let crop = fp.calculate_crop(1000, 500, 500, 500);

        // Should center the crop
        assert_eq!(crop.width, 500);
        assert_eq!(crop.height, 500);
        assert_eq!(crop.x, 250); // Centered
    }

    #[test]
    fn test_focal_point_css() {
        let fp = FocalPoint::new(0.25, 0.75);
        let css = fp.to_css();

        assert!(css.contains("25%"));
        assert!(css.contains("75%"));
    }

    #[test]
    fn test_featured_image_srcset() {
        let mut image = FeaturedImage::new(1, "http://example.com/image.jpg");
        image.width = 1200;
        image.height = 800;
        image.sizes.insert("medium".to_string(), ImageSize {
            name: "medium".to_string(),
            url: "http://example.com/image-300x200.jpg".to_string(),
            width: 300,
            height: 200,
            crop: false,
            webp_url: None,
        });

        let srcset = image.srcset();
        assert!(srcset.contains("300w"));
        assert!(srcset.contains("1200w"));
    }

    #[test]
    fn test_social_platform_specs() {
        let og = SocialPlatform::OpenGraph.get_spec();
        assert_eq!(og.width, 1200);
        assert_eq!(og.height, 630);

        let twitter = SocialPlatform::TwitterCard.get_spec();
        assert_eq!(twitter.width, 1200);
        assert_eq!(twitter.height, 600);
    }
}
