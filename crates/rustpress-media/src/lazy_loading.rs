//! Lazy loading support for images
//!
//! Provides functionality for generating lazy-loaded image markup
//! with placeholder images and intersection observer support.

use serde::{Deserialize, Serialize};

/// Lazy loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadConfig {
    /// Use native loading="lazy" attribute
    pub use_native_lazy: bool,

    /// Use Intersection Observer for custom lazy loading
    pub use_intersection_observer: bool,

    /// Threshold for Intersection Observer (0.0 - 1.0)
    pub threshold: f64,

    /// Root margin for preloading (e.g., "100px")
    pub root_margin: String,

    /// Placeholder type
    pub placeholder_type: PlaceholderType,

    /// LQIP (Low Quality Image Placeholder) quality
    pub lqip_quality: u8,

    /// LQIP blur amount
    pub lqip_blur: f32,

    /// Fade-in duration in milliseconds
    pub fade_duration: u32,

    /// CSS class for lazy images
    pub lazy_class: String,

    /// CSS class when image is loaded
    pub loaded_class: String,
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self {
            use_native_lazy: true,
            use_intersection_observer: true,
            threshold: 0.1,
            root_margin: "200px".to_string(),
            placeholder_type: PlaceholderType::BlurHash,
            lqip_quality: 10,
            lqip_blur: 20.0,
            fade_duration: 300,
            lazy_class: "lazy".to_string(),
            loaded_class: "loaded".to_string(),
        }
    }
}

/// Placeholder types for lazy loading
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlaceholderType {
    /// No placeholder (blank)
    None,

    /// Solid color placeholder
    Color,

    /// Low Quality Image Placeholder
    Lqip,

    /// BlurHash encoded placeholder
    BlurHash,

    /// Dominant color from image
    DominantColor,

    /// SVG placeholder with aspect ratio
    SvgAspectRatio,

    /// Skeleton/shimmer effect
    Skeleton,
}

/// Lazy loading service
pub struct LazyLoadService {
    config: LazyLoadConfig,
}

impl LazyLoadService {
    /// Create new service with config
    pub fn new(config: LazyLoadConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(LazyLoadConfig::default())
    }

    /// Generate lazy image HTML
    pub fn generate_img_tag(&self, image: &LazyImage) -> String {
        let mut attrs = Vec::new();

        // Data attributes for lazy loading
        attrs.push(format!("data-src=\"{}\"", image.src));

        if let Some(srcset) = &image.srcset {
            attrs.push(format!("data-srcset=\"{}\"", srcset));
        }

        if let Some(sizes) = &image.sizes {
            attrs.push(format!("sizes=\"{}\"", sizes));
        }

        // Placeholder
        let placeholder = self.generate_placeholder(image);
        attrs.push(format!("src=\"{}\"", placeholder));

        // Native lazy loading
        if self.config.use_native_lazy {
            attrs.push("loading=\"lazy\"".to_string());
        }

        // Dimensions
        if let Some(width) = image.width {
            attrs.push(format!("width=\"{}\"", width));
        }
        if let Some(height) = image.height {
            attrs.push(format!("height=\"{}\"", height));
        }

        // Accessibility
        attrs.push(format!("alt=\"{}\"", escape_html(&image.alt)));

        // CSS class
        attrs.push(format!("class=\"{} {}\"", self.config.lazy_class, image.class.as_deref().unwrap_or("")));

        // Decoding
        attrs.push("decoding=\"async\"".to_string());

        format!("<img {}>", attrs.join(" "))
    }

    /// Generate picture element with lazy loading
    pub fn generate_picture_tag(&self, image: &LazyImage, sources: &[ImageSource]) -> String {
        let mut html = String::from("<picture>");

        // Add source elements
        for source in sources {
            html.push_str(&format!(
                "<source data-srcset=\"{}\" type=\"{}\"",
                source.srcset, source.mime_type
            ));

            if let Some(sizes) = &source.sizes {
                html.push_str(&format!(" sizes=\"{}\"", sizes));
            }

            if let Some(media) = &source.media {
                html.push_str(&format!(" media=\"{}\"", media));
            }

            html.push_str(">");
        }

        // Add img element
        html.push_str(&self.generate_img_tag(image));

        html.push_str("</picture>");
        html
    }

    /// Generate placeholder based on config
    fn generate_placeholder(&self, image: &LazyImage) -> String {
        match self.config.placeholder_type {
            PlaceholderType::None => {
                // Transparent 1x1 pixel
                "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7".to_string()
            }
            PlaceholderType::Color => {
                let color = image.placeholder_color.as_deref().unwrap_or("#f0f0f0");
                self.generate_color_placeholder(color, image.width, image.height)
            }
            PlaceholderType::Lqip => {
                image.lqip_data.clone().unwrap_or_else(|| {
                    self.generate_color_placeholder("#f0f0f0", image.width, image.height)
                })
            }
            PlaceholderType::BlurHash => {
                image.blurhash.as_ref()
                    .map(|_| {
                        // BlurHash would be decoded client-side or server-side
                        // For now, return a placeholder
                        self.generate_color_placeholder("#f0f0f0", image.width, image.height)
                    })
                    .unwrap_or_else(|| {
                        self.generate_color_placeholder("#f0f0f0", image.width, image.height)
                    })
            }
            PlaceholderType::DominantColor => {
                let color = image.dominant_color.as_deref().unwrap_or("#f0f0f0");
                self.generate_color_placeholder(color, image.width, image.height)
            }
            PlaceholderType::SvgAspectRatio => {
                self.generate_svg_placeholder(image.width, image.height)
            }
            PlaceholderType::Skeleton => {
                // Return a data URL with skeleton animation
                self.generate_skeleton_placeholder(image.width, image.height)
            }
        }
    }

    /// Generate SVG placeholder with aspect ratio
    fn generate_svg_placeholder(&self, width: Option<u32>, height: Option<u32>) -> String {
        let w = width.unwrap_or(100);
        let h = height.unwrap_or(100);

        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\"><rect fill=\"#f0f0f0\" width=\"100%\" height=\"100%\"/></svg>"
        );

        format!("data:image/svg+xml;base64,{}", base64_encode(&svg))
    }

    /// Generate color placeholder
    fn generate_color_placeholder(&self, color: &str, width: Option<u32>, height: Option<u32>) -> String {
        let w = width.unwrap_or(100);
        let h = height.unwrap_or(100);

        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\"><rect fill=\"{color}\" width=\"100%\" height=\"100%\"/></svg>"
        );

        format!("data:image/svg+xml;base64,{}", base64_encode(&svg))
    }

    /// Generate skeleton placeholder with shimmer effect
    fn generate_skeleton_placeholder(&self, width: Option<u32>, height: Option<u32>) -> String {
        let w = width.unwrap_or(100);
        let h = height.unwrap_or(100);

        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\">\
                <defs>\
                    <linearGradient id=\"shimmer\" x1=\"0%\" y1=\"0%\" x2=\"100%\" y2=\"0%\">\
                        <stop offset=\"0%\" style=\"stop-color:#f0f0f0\">\
                            <animate attributeName=\"offset\" values=\"-2;1\" dur=\"1.5s\" repeatCount=\"indefinite\"/>\
                        </stop>\
                        <stop offset=\"50%\" style=\"stop-color:#e0e0e0\">\
                            <animate attributeName=\"offset\" values=\"-1;2\" dur=\"1.5s\" repeatCount=\"indefinite\"/>\
                        </stop>\
                        <stop offset=\"100%\" style=\"stop-color:#f0f0f0\">\
                            <animate attributeName=\"offset\" values=\"0;3\" dur=\"1.5s\" repeatCount=\"indefinite\"/>\
                        </stop>\
                    </linearGradient>\
                </defs>\
                <rect fill=\"url(#shimmer)\" width=\"100%\" height=\"100%\"/>\
            </svg>"
        );

        format!("data:image/svg+xml;base64,{}", base64_encode(&svg))
    }

    /// Generate JavaScript for custom lazy loading
    pub fn generate_lazy_load_script(&self) -> String {
        format!(
            r#"
(function() {{
    const config = {{
        threshold: {},
        rootMargin: '{}',
        lazyClass: '{}',
        loadedClass: '{}',
        fadeDuration: {}
    }};

    function loadImage(img) {{
        const src = img.dataset.src;
        const srcset = img.dataset.srcset;

        if (srcset) {{
            img.srcset = srcset;
        }}
        if (src) {{
            img.src = src;
        }}

        img.onload = function() {{
            img.classList.add(config.loadedClass);
            img.style.transition = `opacity ${{config.fadeDuration}}ms`;
        }};
    }}

    if ('IntersectionObserver' in window) {{
        const observer = new IntersectionObserver((entries) => {{
            entries.forEach(entry => {{
                if (entry.isIntersecting) {{
                    loadImage(entry.target);
                    observer.unobserve(entry.target);
                }}
            }});
        }}, {{
            threshold: config.threshold,
            rootMargin: config.rootMargin
        }});

        document.querySelectorAll('.' + config.lazyClass).forEach(img => {{
            observer.observe(img);
        }});
    }} else {{
        // Fallback for browsers without IntersectionObserver
        document.querySelectorAll('.' + config.lazyClass).forEach(loadImage);
    }}
}})();
"#,
            self.config.threshold,
            self.config.root_margin,
            self.config.lazy_class,
            self.config.loaded_class,
            self.config.fade_duration
        )
    }

    /// Generate CSS for lazy loading
    pub fn generate_lazy_load_css(&self) -> String {
        format!(
            r#"
.{lazy_class} {{
    opacity: 0;
    transition: opacity {fade}ms ease-in-out;
}}

.{lazy_class}.{loaded_class} {{
    opacity: 1;
}}

/* Skeleton animation */
@keyframes shimmer {{
    0% {{
        background-position: -200% 0;
    }}
    100% {{
        background-position: 200% 0;
    }}
}}

.{lazy_class}[data-placeholder="skeleton"] {{
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite;
}}
"#,
            lazy_class = self.config.lazy_class,
            loaded_class = self.config.loaded_class,
            fade = self.config.fade_duration
        )
    }
}

/// Image data for lazy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyImage {
    /// Image source URL
    pub src: String,

    /// Srcset attribute
    pub srcset: Option<String>,

    /// Sizes attribute
    pub sizes: Option<String>,

    /// Alt text
    pub alt: String,

    /// Width
    pub width: Option<u32>,

    /// Height
    pub height: Option<u32>,

    /// CSS class
    pub class: Option<String>,

    /// LQIP data URL
    pub lqip_data: Option<String>,

    /// BlurHash string
    pub blurhash: Option<String>,

    /// Dominant color (hex)
    pub dominant_color: Option<String>,

    /// Placeholder color (hex)
    pub placeholder_color: Option<String>,
}

/// Source element for picture tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    /// Srcset attribute
    pub srcset: String,

    /// MIME type
    pub mime_type: String,

    /// Sizes attribute
    pub sizes: Option<String>,

    /// Media query
    pub media: Option<String>,
}

/// Simple HTML escaping
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Simple base64 encoding
fn base64_encode(s: &str) -> String {
    use std::io::Write;
    let mut encoder = base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    encoder.write_all(s.as_bytes()).unwrap();
    encoder.into_inner()
}

// Use a simple implementation without the base64 crate
mod base64 {
    pub mod engine {
        pub mod general_purpose {
            pub struct Standard;
            pub const STANDARD: Standard = Standard;
        }
    }
    pub mod write {
        use super::engine::general_purpose::Standard;

        pub struct EncoderStringWriter {
            data: Vec<u8>,
        }

        impl EncoderStringWriter {
            pub fn new(_: &Standard) -> Self {
                Self { data: Vec::new() }
            }

            pub fn into_inner(self) -> String {
                // Simple base64 encoding
                const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

                let mut result = String::new();
                let mut i = 0;

                while i < self.data.len() {
                    let b0 = self.data[i];
                    let b1 = if i + 1 < self.data.len() { self.data[i + 1] } else { 0 };
                    let b2 = if i + 2 < self.data.len() { self.data[i + 2] } else { 0 };

                    result.push(ALPHABET[(b0 >> 2) as usize] as char);
                    result.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

                    if i + 1 < self.data.len() {
                        result.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
                    } else {
                        result.push('=');
                    }

                    if i + 2 < self.data.len() {
                        result.push(ALPHABET[(b2 & 0x3f) as usize] as char);
                    } else {
                        result.push('=');
                    }

                    i += 3;
                }

                result
            }
        }

        impl std::io::Write for EncoderStringWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.data.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_load_img_tag() {
        let service = LazyLoadService::default_config();
        let image = LazyImage {
            src: "/images/test.jpg".to_string(),
            srcset: Some("/images/test-300.jpg 300w, /images/test-600.jpg 600w".to_string()),
            sizes: Some("(max-width: 600px) 300px, 600px".to_string()),
            alt: "Test image".to_string(),
            width: Some(600),
            height: Some(400),
            class: None,
            lqip_data: None,
            blurhash: None,
            dominant_color: None,
            placeholder_color: None,
        };

        let html = service.generate_img_tag(&image);
        assert!(html.contains("data-src=\"/images/test.jpg\""));
        assert!(html.contains("loading=\"lazy\""));
        assert!(html.contains("alt=\"Test image\""));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
    }
}
