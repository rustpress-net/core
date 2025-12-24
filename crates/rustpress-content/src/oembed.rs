//! # oEmbed Content Embedding
//!
//! Embedded content handling with oEmbed protocol support.
//!
//! Features:
//! - oEmbed discovery and fetching
//! - Provider registry
//! - Response caching
//! - Fallback handling
//! - Security filtering

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// oEmbed response types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OembedType {
    Photo,
    Video,
    Link,
    Rich,
}

impl OembedType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "photo" => Some(Self::Photo),
            "video" => Some(Self::Video),
            "link" => Some(Self::Link),
            "rich" => Some(Self::Rich),
            _ => None,
        }
    }
}

/// oEmbed response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OembedResponse {
    /// Response type
    pub oembed_type: OembedType,

    /// oEmbed version
    pub version: String,

    /// Resource title
    pub title: Option<String>,

    /// Author name
    pub author_name: Option<String>,

    /// Author URL
    pub author_url: Option<String>,

    /// Provider name
    pub provider_name: Option<String>,

    /// Provider URL
    pub provider_url: Option<String>,

    /// Cache time in seconds
    pub cache_age: Option<u64>,

    /// Thumbnail URL
    pub thumbnail_url: Option<String>,

    /// Thumbnail width
    pub thumbnail_width: Option<u32>,

    /// Thumbnail height
    pub thumbnail_height: Option<u32>,

    // Photo type specific
    /// Photo URL
    pub url: Option<String>,

    /// Photo width
    pub width: Option<u32>,

    /// Photo height
    pub height: Option<u32>,

    // Video/Rich type specific
    /// HTML embed code
    pub html: Option<String>,
}

impl OembedResponse {
    /// Get responsive embed HTML
    pub fn get_embed_html(&self, max_width: Option<u32>) -> String {
        if let Some(ref html) = self.html {
            let mut embed = html.clone();

            // Make iframe responsive if present
            if embed.contains("<iframe") {
                embed = make_iframe_responsive(&embed, max_width);
            }

            // Wrap in container
            format!(
                r#"<div class="wp-embed-wrapper" data-provider="{}">{}</div>"#,
                self.provider_name.as_deref().unwrap_or("unknown"),
                embed
            )
        } else if let Some(ref url) = self.url {
            // Photo type
            let width = self.width.unwrap_or(640);
            let height = self.height.unwrap_or(480);
            let alt = self.title.as_deref().unwrap_or("");

            format!(
                r#"<figure class="wp-embed-photo"><img src="{}" width="{}" height="{}" alt="{}" loading="lazy"></figure>"#,
                url, width, height, alt
            )
        } else {
            // Link fallback
            let title = self.title.as_deref().unwrap_or("Embedded content");
            if let Some(ref provider) = self.provider_name {
                format!(
                    r#"<div class="wp-embed-link"><a href="#">{}</a> - {}</div>"#,
                    title, provider
                )
            } else {
                format!(r#"<div class="wp-embed-link"><a href="#">{}</a></div>"#, title)
            }
        }
    }
}

/// Make iframe responsive with aspect ratio
fn make_iframe_responsive(html: &str, max_width: Option<u32>) -> String {
    let width_re = Regex::new(r#"width="(\d+)""#).unwrap();
    let height_re = Regex::new(r#"height="(\d+)""#).unwrap();

    let width: f64 = width_re.captures(html)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(560.0);

    let height: f64 = height_re.captures(html)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(315.0);

    let aspect_ratio = height / width * 100.0;

    let max_w = max_width.map(|w| format!("max-width: {}px;", w)).unwrap_or_default();

    format!(
        r#"<div class="wp-embed-responsive" style="position: relative; padding-bottom: {:.2}%; height: 0; overflow: hidden; {}">
<iframe style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;" {} allowfullscreen></iframe>
</div>"#,
        aspect_ratio,
        max_w,
        html.trim_start_matches("<iframe")
            .trim_end_matches("</iframe>")
            .trim_end_matches('>')
    )
}

/// oEmbed provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OembedProvider {
    /// Provider name
    pub name: String,

    /// URL patterns this provider handles
    pub url_patterns: Vec<String>,

    /// oEmbed endpoint URL
    pub endpoint: String,

    /// Whether to use discovery
    pub discovery: bool,

    /// Default parameters
    pub params: HashMap<String, String>,
}

impl OembedProvider {
    pub fn new(name: &str, patterns: Vec<&str>, endpoint: &str) -> Self {
        Self {
            name: name.to_string(),
            url_patterns: patterns.iter().map(|s| s.to_string()).collect(),
            endpoint: endpoint.to_string(),
            discovery: false,
            params: HashMap::new(),
        }
    }

    /// Check if URL matches this provider
    pub fn matches(&self, url: &str) -> bool {
        self.url_patterns.iter().any(|pattern| {
            let regex_pattern = pattern
                .replace(".", r"\.")
                .replace("*", ".*");
            Regex::new(&regex_pattern).map(|re| re.is_match(url)).unwrap_or(false)
        })
    }

    /// Build oEmbed request URL
    pub fn build_request_url(&self, content_url: &str, max_width: Option<u32>, max_height: Option<u32>) -> String {
        let mut url = if self.endpoint.contains('?') {
            format!("{}&url={}", self.endpoint, urlencoding::encode(content_url))
        } else {
            format!("{}?url={}", self.endpoint, urlencoding::encode(content_url))
        };

        url.push_str("&format=json");

        if let Some(w) = max_width {
            url.push_str(&format!("&maxwidth={}", w));
        }
        if let Some(h) = max_height {
            url.push_str(&format!("&maxheight={}", h));
        }

        for (key, value) in &self.params {
            url.push_str(&format!("&{}={}", key, urlencoding::encode(value)));
        }

        url
    }
}

/// oEmbed provider registry
pub struct OembedRegistry {
    providers: Vec<OembedProvider>,
    cache: HashMap<String, CachedEmbed>,
    default_cache_time: u64,
    max_width: Option<u32>,
    max_height: Option<u32>,
}

/// Cached embed data
#[derive(Debug, Clone)]
struct CachedEmbed {
    response: OembedResponse,
    cached_at: std::time::Instant,
    cache_duration: u64,
}

impl CachedEmbed {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed().as_secs() > self.cache_duration
    }
}

impl Default for OembedRegistry {
    fn default() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
            cache: HashMap::new(),
            default_cache_time: 86400, // 24 hours
            max_width: Some(800),
            max_height: Some(600),
        };

        registry.register_default_providers();
        registry
    }
}

impl OembedRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register default providers
    fn register_default_providers(&mut self) {
        // YouTube
        self.register(OembedProvider::new(
            "YouTube",
            vec![
                "https://www.youtube.com/watch?*",
                "https://youtu.be/*",
                "https://www.youtube.com/embed/*",
            ],
            "https://www.youtube.com/oembed",
        ));

        // Vimeo
        self.register(OembedProvider::new(
            "Vimeo",
            vec![
                "https://vimeo.com/*",
                "https://player.vimeo.com/video/*",
            ],
            "https://vimeo.com/api/oembed.json",
        ));

        // Twitter/X
        self.register(OembedProvider::new(
            "Twitter",
            vec![
                "https://twitter.com/*/status/*",
                "https://x.com/*/status/*",
            ],
            "https://publish.twitter.com/oembed",
        ));

        // Instagram
        self.register(OembedProvider::new(
            "Instagram",
            vec![
                "https://www.instagram.com/p/*",
                "https://www.instagram.com/reel/*",
            ],
            "https://graph.facebook.com/v10.0/instagram_oembed",
        ));

        // Spotify
        self.register(OembedProvider::new(
            "Spotify",
            vec![
                "https://open.spotify.com/*",
            ],
            "https://open.spotify.com/oembed",
        ));

        // SoundCloud
        self.register(OembedProvider::new(
            "SoundCloud",
            vec![
                "https://soundcloud.com/*",
            ],
            "https://soundcloud.com/oembed",
        ));

        // TikTok
        self.register(OembedProvider::new(
            "TikTok",
            vec![
                "https://www.tiktok.com/*/video/*",
            ],
            "https://www.tiktok.com/oembed",
        ));

        // CodePen
        self.register(OembedProvider::new(
            "CodePen",
            vec![
                "https://codepen.io/*/pen/*",
            ],
            "https://codepen.io/api/oembed",
        ));

        // Flickr
        self.register(OembedProvider::new(
            "Flickr",
            vec![
                "https://www.flickr.com/photos/*",
                "https://flic.kr/p/*",
            ],
            "https://www.flickr.com/services/oembed/",
        ));

        // SlideShare
        self.register(OembedProvider::new(
            "SlideShare",
            vec![
                "https://www.slideshare.net/*/*",
            ],
            "https://www.slideshare.net/api/oembed/2",
        ));
    }

    /// Register a provider
    pub fn register(&mut self, provider: OembedProvider) {
        self.providers.push(provider);
    }

    /// Find provider for URL
    pub fn find_provider(&self, url: &str) -> Option<&OembedProvider> {
        self.providers.iter().find(|p| p.matches(url))
    }

    /// Check if URL is supported
    pub fn is_supported(&self, url: &str) -> bool {
        self.find_provider(url).is_some()
    }

    /// Get cached response
    pub fn get_cached(&self, url: &str) -> Option<&OembedResponse> {
        self.cache.get(url).filter(|c| !c.is_expired()).map(|c| &c.response)
    }

    /// Cache a response
    pub fn cache_response(&mut self, url: &str, response: OembedResponse) {
        let cache_duration = response.cache_age.unwrap_or(self.default_cache_time);
        self.cache.insert(url.to_string(), CachedEmbed {
            response,
            cached_at: std::time::Instant::now(),
            cache_duration,
        });
    }

    /// Get embed request URL for provider
    pub fn get_request_url(&self, url: &str) -> Option<String> {
        self.find_provider(url).map(|p| {
            p.build_request_url(url, self.max_width, self.max_height)
        })
    }

    /// Clear expired cache entries
    pub fn clear_expired(&mut self) {
        self.cache.retain(|_, v| !v.is_expired());
    }

    /// Clear all cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Set default dimensions
    pub fn set_dimensions(&mut self, max_width: Option<u32>, max_height: Option<u32>) {
        self.max_width = max_width;
        self.max_height = max_height;
    }
}

/// Process embed shortcodes in content
pub fn process_embeds(content: &str, registry: &OembedRegistry) -> String {
    // Match [embed]URL[/embed] or plain URLs on their own line
    let embed_re = Regex::new(r"\[embed\]([^\[]+)\[/embed\]").unwrap();
    let url_re = Regex::new(r"(?m)^(https?://[^\s<>\[\]]+)$").unwrap();

    let mut result = content.to_string();

    // Process [embed] shortcodes
    for caps in embed_re.captures_iter(content) {
        let url = caps[1].trim();
        if registry.is_supported(url) {
            let placeholder = format!(
                r#"<div class="wp-embed-placeholder" data-url="{}">Loading embed...</div>"#,
                url
            );
            result = result.replace(&caps[0], &placeholder);
        }
    }

    // Process plain URLs
    for caps in url_re.captures_iter(content) {
        let url = &caps[1];
        if registry.is_supported(url) {
            let placeholder = format!(
                r#"<div class="wp-embed-placeholder" data-url="{}">Loading embed...</div>"#,
                url
            );
            result = result.replace(&caps[0], &placeholder);
        }
    }

    result
}

/// Generate simple embed fallback when oEmbed fails
pub fn generate_fallback(url: &str) -> String {
    format!(
        r#"<div class="wp-embed-fallback"><a href="{}" target="_blank" rel="noopener">{}</a></div>"#,
        url, url
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_matching() {
        let provider = OembedProvider::new(
            "YouTube",
            vec!["https://www.youtube.com/watch?*"],
            "https://www.youtube.com/oembed",
        );

        assert!(provider.matches("https://www.youtube.com/watch?v=abc123"));
        assert!(!provider.matches("https://vimeo.com/123456"));
    }

    #[test]
    fn test_registry() {
        let registry = OembedRegistry::new();

        assert!(registry.is_supported("https://www.youtube.com/watch?v=test"));
        assert!(registry.is_supported("https://vimeo.com/123456"));
        assert!(!registry.is_supported("https://example.com/video"));
    }

    #[test]
    fn test_request_url_building() {
        let provider = OembedProvider::new(
            "Test",
            vec!["https://test.com/*"],
            "https://test.com/oembed",
        );

        let url = provider.build_request_url("https://test.com/video/123", Some(800), None);
        assert!(url.contains("url="));
        assert!(url.contains("maxwidth=800"));
        assert!(url.contains("format=json"));
    }

    #[test]
    fn test_process_embeds() {
        let registry = OembedRegistry::new();
        let content = "[embed]https://www.youtube.com/watch?v=test[/embed]";

        let processed = process_embeds(content, &registry);
        assert!(processed.contains("wp-embed-placeholder"));
        assert!(processed.contains("data-url"));
    }
}
