//! # HTML Sanitization
//!
//! Comprehensive HTML sanitization using ammonia with WordPress-compatible presets.
//!
//! Features:
//! - Multiple sanitization levels (strict, standard, relaxed, raw)
//! - WordPress-compatible allowed tags and attributes
//! - URL scheme validation
//! - Custom attribute sanitizers
//! - KSES-style capability-based filtering

use ammonia::{Builder, Url, UrlRelative};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Sanitization errors
#[derive(Debug, Error)]
pub enum SanitizeError {
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("URL validation failed: {0}")]
    UrlError(String),
}

/// Sanitization level presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanitizationLevel {
    /// Strip all HTML tags
    Strict,

    /// Allow basic formatting (b, i, a, p, br, etc.)
    Basic,

    /// Standard WordPress post content allowlist
    Standard,

    /// Extended allowlist including tables, iframes (for embeds)
    Relaxed,

    /// Allow most HTML, strip only dangerous elements
    Permissive,

    /// No sanitization (dangerous, use only for trusted admin content)
    Raw,
}

impl Default for SanitizationLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Sanitization configuration
#[derive(Debug, Clone)]
pub struct SanitizeConfig {
    /// Base sanitization level
    pub level: SanitizationLevel,

    /// Additional allowed tags
    pub additional_tags: HashSet<String>,

    /// Additional allowed attributes (tag -> attributes)
    pub additional_attributes: HashMap<String, HashSet<String>>,

    /// Allowed URL schemes
    pub url_schemes: HashSet<String>,

    /// Strip comments
    pub strip_comments: bool,

    /// Clean document tags (html, head, body)
    pub clean_document_tags: bool,

    /// Linkify URLs in text
    pub linkify_urls: bool,

    /// Max link length (0 for unlimited)
    pub max_link_length: usize,

    /// Add rel="noopener" to external links
    pub add_noopener: bool,

    /// Add rel="noreferrer" to external links
    pub add_noreferrer: bool,

    /// Add target="_blank" to external links
    pub link_target_blank: bool,

    /// Base URL for relative links
    pub base_url: Option<String>,

    /// Domain allowlist for URLs
    pub allowed_domains: HashSet<String>,

    /// Domain blocklist for URLs
    pub blocked_domains: HashSet<String>,

    /// CSS property allowlist (for style attributes)
    pub allowed_css_properties: HashSet<String>,

    /// Custom tag handlers
    pub tag_filters: HashMap<String, TagFilter>,
}

/// Custom tag filter behavior
#[derive(Debug, Clone)]
pub enum TagFilter {
    /// Allow tag with all attributes
    Allow,

    /// Allow tag with specific attributes only
    AllowWithAttributes(HashSet<String>),

    /// Replace tag with another
    Replace(String),

    /// Remove tag but keep content
    StripTag,

    /// Remove tag and content
    Remove,

    /// Custom transform function name
    Custom(String),
}

impl Default for SanitizeConfig {
    fn default() -> Self {
        Self {
            level: SanitizationLevel::Standard,
            additional_tags: HashSet::new(),
            additional_attributes: HashMap::new(),
            url_schemes: default_url_schemes(),
            strip_comments: true,
            clean_document_tags: true,
            linkify_urls: true,
            max_link_length: 0,
            add_noopener: true,
            add_noreferrer: true,
            link_target_blank: true,
            base_url: None,
            allowed_domains: HashSet::new(),
            blocked_domains: HashSet::new(),
            allowed_css_properties: default_css_properties(),
            tag_filters: HashMap::new(),
        }
    }
}

/// Default allowed URL schemes
fn default_url_schemes() -> HashSet<String> {
    [
        "http", "https", "mailto", "tel", "ftp", "data",
    ].iter().map(|s| s.to_string()).collect()
}

/// Default allowed CSS properties
fn default_css_properties() -> HashSet<String> {
    [
        // Text
        "color", "font", "font-family", "font-size", "font-style", "font-weight",
        "text-align", "text-decoration", "text-indent", "text-transform",
        "line-height", "letter-spacing", "word-spacing",
        // Box model
        "margin", "margin-top", "margin-right", "margin-bottom", "margin-left",
        "padding", "padding-top", "padding-right", "padding-bottom", "padding-left",
        "border", "border-width", "border-style", "border-color",
        "border-top", "border-right", "border-bottom", "border-left",
        "border-radius",
        // Size
        "width", "height", "max-width", "max-height", "min-width", "min-height",
        // Layout
        "display", "float", "clear", "vertical-align",
        // Background
        "background", "background-color", "background-image", "background-position",
        "background-repeat", "background-size",
        // Other
        "list-style", "list-style-type", "opacity", "visibility",
    ].iter().map(|s| s.to_string()).collect()
}

/// Get basic allowed tags
fn basic_tags() -> HashSet<&'static str> {
    [
        "a", "abbr", "acronym", "b", "bdo", "big", "blockquote", "br", "cite",
        "code", "dfn", "em", "i", "kbd", "mark", "p", "pre", "q", "s", "samp",
        "small", "span", "strike", "strong", "sub", "sup", "time", "tt", "u",
        "var", "wbr",
    ].into_iter().collect()
}

/// Get standard WordPress allowed tags
fn standard_tags() -> HashSet<&'static str> {
    let mut tags = basic_tags();
    tags.extend([
        // Lists
        "ul", "ol", "li", "dl", "dt", "dd",
        // Headings
        "h1", "h2", "h3", "h4", "h5", "h6",
        // Structure
        "div", "article", "section", "header", "footer", "nav", "aside",
        "figure", "figcaption", "main",
        // Media
        "img", "audio", "video", "source", "track", "picture",
        // Tables
        "table", "thead", "tbody", "tfoot", "tr", "th", "td", "caption", "colgroup", "col",
        // Forms (limited)
        "label", "legend", "fieldset",
        // Other
        "hr", "address", "details", "summary", "data", "meter", "progress",
        "ruby", "rt", "rp",
    ].into_iter());
    tags
}

/// Get relaxed allowed tags
fn relaxed_tags() -> HashSet<&'static str> {
    let mut tags = standard_tags();
    tags.extend([
        "iframe", "embed", "object", "param",
        "form", "input", "button", "select", "option", "optgroup", "textarea",
        "output", "datalist",
        "canvas", "svg", "math",
        "style", "template",
    ].into_iter());
    tags
}

/// Standard attributes for each tag
fn standard_attributes() -> HashMap<&'static str, HashSet<&'static str>> {
    let mut attrs = HashMap::new();

    // Global attributes
    let global: HashSet<&str> = [
        "id", "class", "style", "title", "dir", "lang", "hidden",
        "data-*", "aria-*", "role", "tabindex",
    ].into_iter().collect();

    // Links
    attrs.insert("a", {
        let mut set = global.clone();
        set.extend(["href", "target", "rel", "name", "download", "hreflang", "type"]);
        set
    });

    // Images
    attrs.insert("img", {
        let mut set = global.clone();
        set.extend([
            "src", "alt", "width", "height", "loading", "decoding",
            "srcset", "sizes", "crossorigin", "usemap", "ismap",
        ]);
        set
    });

    // Video
    attrs.insert("video", {
        let mut set = global.clone();
        set.extend([
            "src", "poster", "width", "height", "controls", "autoplay",
            "loop", "muted", "preload", "playsinline",
        ]);
        set
    });

    // Audio
    attrs.insert("audio", {
        let mut set = global.clone();
        set.extend(["src", "controls", "autoplay", "loop", "muted", "preload"]);
        set
    });

    // Source
    attrs.insert("source", {
        let mut set = global.clone();
        set.extend(["src", "type", "media", "srcset", "sizes"]);
        set
    });

    // Iframe (for embeds)
    attrs.insert("iframe", {
        let mut set = global.clone();
        set.extend([
            "src", "srcdoc", "name", "sandbox", "allow", "allowfullscreen",
            "width", "height", "loading", "referrerpolicy",
        ]);
        set
    });

    // Tables
    for tag in ["table", "th", "td"] {
        attrs.insert(tag, {
            let mut set = global.clone();
            set.extend(["align", "valign", "width", "height", "colspan", "rowspan", "scope"]);
            set
        });
    }

    // Blockquote
    attrs.insert("blockquote", {
        let mut set = global.clone();
        set.extend(["cite"]);
        set
    });

    // Lists
    attrs.insert("ol", {
        let mut set = global.clone();
        set.extend(["start", "reversed", "type"]);
        set
    });

    // Time
    attrs.insert("time", {
        let mut set = global.clone();
        set.extend(["datetime"]);
        set
    });

    // Data
    attrs.insert("data", {
        let mut set = global.clone();
        set.extend(["value"]);
        set
    });

    // Progress/Meter
    attrs.insert("progress", {
        let mut set = global.clone();
        set.extend(["value", "max"]);
        set
    });
    attrs.insert("meter", {
        let mut set = global.clone();
        set.extend(["value", "min", "max", "low", "high", "optimum"]);
        set
    });

    // Default global for other tags
    for tag in standard_tags() {
        attrs.entry(tag).or_insert_with(|| global.clone());
    }

    attrs
}

/// HTML Sanitizer
pub struct Sanitizer {
    config: SanitizeConfig,
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new(SanitizeConfig::default())
    }
}

impl Sanitizer {
    pub fn new(config: SanitizeConfig) -> Self {
        Self { config }
    }

    /// Create sanitizer with specific level
    pub fn with_level(level: SanitizationLevel) -> Self {
        Self::new(SanitizeConfig {
            level,
            ..Default::default()
        })
    }

    /// Sanitize HTML content
    pub fn sanitize(&self, html: &str) -> String {
        match self.config.level {
            SanitizationLevel::Raw => html.to_string(),
            SanitizationLevel::Strict => self.strip_all_html(html),
            _ => self.sanitize_with_ammonia(html),
        }
    }

    /// Strip all HTML tags
    fn strip_all_html(&self, html: &str) -> String {
        let re = Regex::new(r"<[^>]+>").unwrap();
        let stripped = re.replace_all(html, "");

        // Decode basic HTML entities
        stripped
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    }

    /// Sanitize using ammonia
    fn sanitize_with_ammonia(&self, html: &str) -> String {
        let mut builder = Builder::default();

        // Get allowed tags based on level
        let tags: HashSet<&str> = match self.config.level {
            SanitizationLevel::Basic => basic_tags(),
            SanitizationLevel::Standard => standard_tags(),
            SanitizationLevel::Relaxed | SanitizationLevel::Permissive => relaxed_tags(),
            _ => HashSet::new(),
        };

        // Add additional custom tags
        let all_tags: HashSet<&str> = tags
            .into_iter()
            .chain(self.config.additional_tags.iter().map(|s| s.as_str()))
            .collect();

        builder.tags(all_tags.clone());

        // Get attributes for tags
        let mut attrs = standard_attributes();

        // Add additional custom attributes
        for (tag, custom_attrs) in &self.config.additional_attributes {
            let entry = attrs.entry(tag.as_str()).or_insert_with(HashSet::new);
            entry.extend(custom_attrs.iter().map(|s| s.as_str()));
        }

        // Convert to ammonia format - filter to only allowed tags
        // Also remove 'rel' from <a> tag attributes since ammonia doesn't allow
        // setting 'rel' in tag_attributes when using link_rel()
        let filtered_attrs: HashMap<&str, HashSet<&str>> = attrs
            .iter()
            .filter(|(tag, _)| all_tags.contains(*tag))
            .map(|(tag, attr_set)| {
                if *tag == "a" {
                    let mut filtered_set = attr_set.clone();
                    filtered_set.remove("rel");
                    (*tag, filtered_set)
                } else {
                    (*tag, attr_set.clone())
                }
            })
            .collect();
        builder.tag_attributes(filtered_attrs);

        // URL schemes
        let schemes: HashSet<&str> = self.config.url_schemes.iter().map(|s| s.as_str()).collect();
        builder.url_schemes(schemes);

        // Link rel attributes
        let mut link_rel = String::new();
        if self.config.add_noopener {
            link_rel.push_str("noopener ");
        }
        if self.config.add_noreferrer {
            link_rel.push_str("noreferrer ");
        }
        if !link_rel.is_empty() {
            builder.link_rel(Some(link_rel.trim()));
        }

        // URL handling
        if let Some(ref base) = self.config.base_url {
            if let Ok(url) = Url::parse(base) {
                builder.url_relative(UrlRelative::RewriteWithBase(url));
            }
        }

        // Strip comments
        builder.strip_comments(self.config.strip_comments);

        // Clean result
        let result = builder.clean(html).to_string();

        // Post-process for additional cleaning
        self.post_process(&result)
    }

    /// Post-process sanitized HTML
    fn post_process(&self, html: &str) -> String {
        let mut result = html.to_string();

        // Clean document tags if needed
        if self.config.clean_document_tags {
            result = result
                .replace("<html>", "")
                .replace("</html>", "")
                .replace("<head>", "")
                .replace("</head>", "")
                .replace("<body>", "")
                .replace("</body>", "");

            // Remove DOCTYPE
            let doctype_re = Regex::new(r"<!DOCTYPE[^>]*>").unwrap();
            result = doctype_re.replace_all(&result, "").to_string();
        }

        // Sanitize style attributes if present
        if self.config.level != SanitizationLevel::Raw {
            result = self.sanitize_styles(&result);
        }

        // Add target="_blank" to external links if configured
        if self.config.link_target_blank {
            result = self.process_external_links(&result);
        }

        // Linkify URLs if configured
        if self.config.linkify_urls {
            result = self.linkify(&result);
        }

        result.trim().to_string()
    }

    /// Sanitize inline CSS styles
    fn sanitize_styles(&self, html: &str) -> String {
        let style_re = Regex::new(r#"style="([^"]*)""#).unwrap();

        style_re.replace_all(html, |caps: &regex::Captures| {
            let styles = &caps[1];
            let sanitized = self.sanitize_css(styles);
            if sanitized.is_empty() {
                String::new()
            } else {
                format!("style=\"{}\"", sanitized)
            }
        }).to_string()
    }

    /// Sanitize CSS properties
    fn sanitize_css(&self, css: &str) -> String {
        css.split(';')
            .filter_map(|rule| {
                let parts: Vec<&str> = rule.split(':').collect();
                if parts.len() != 2 {
                    return None;
                }

                let property = parts[0].trim().to_lowercase();
                let value = parts[1].trim();

                // Check if property is allowed
                if !self.config.allowed_css_properties.contains(&property) {
                    return None;
                }

                // Check for dangerous values
                let value_lower = value.to_lowercase();
                if value_lower.contains("javascript:")
                    || value_lower.contains("expression(")
                    || value_lower.contains("url(")
                {
                    return None;
                }

                Some(format!("{}: {}", property, value))
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Process external links
    fn process_external_links(&self, html: &str) -> String {
        let link_re = Regex::new(r#"<a([^>]*href=["'])(https?://[^"']+)(["'][^>]*)>"#).unwrap();

        link_re.replace_all(html, |caps: &regex::Captures| {
            let pre = &caps[1];
            let url = &caps[2];
            let post = &caps[3];

            // Check if already has target
            let full_tag = format!("<a{}{}{}>", pre, url, post);
            if full_tag.contains("target=") {
                return full_tag;
            }

            format!("<a{}{}{} target=\"_blank\">", pre, url, post)
        }).to_string()
    }

    /// Convert plain URLs to links
    fn linkify(&self, text: &str) -> String {
        // Don't linkify inside existing tags
        // Use a simpler pattern and check preceding character manually
        // since Rust regex doesn't support look-behind
        let url_re = Regex::new(r#"https?://[^\s<>"']+"#).unwrap();

        let mut result = String::new();
        let mut last_end = 0;

        for mat in url_re.find_iter(text) {
            let url = mat.as_str();
            let start = mat.start();

            // Check if preceded by =, ", ', or > (inside an HTML attribute or tag)
            let skip = if start > 0 {
                let prev_char = text.as_bytes()[start - 1];
                matches!(prev_char, b'=' | b'"' | b'\'' | b'>')
            } else {
                false
            };

            // Append text before this match
            result.push_str(&text[last_end..start]);

            if skip {
                // Keep the URL as-is
                result.push_str(url);
            } else {
                // Check max length
                if self.config.max_link_length > 0 && url.len() > self.config.max_link_length {
                    result.push_str(url);
                } else {
                    // Check domain blocks
                    let mut blocked = false;
                    if !self.config.blocked_domains.is_empty() {
                        if let Ok(parsed) = Url::parse(url) {
                            if let Some(host) = parsed.host_str() {
                                if self.config.blocked_domains.contains(host) {
                                    blocked = true;
                                }
                            }
                        }
                    }

                    if blocked {
                        result.push_str(url);
                    } else {
                        result.push_str(&format!("<a href=\"{}\">{}</a>", url, url));
                    }
                }
            }

            last_end = mat.end();
        }

        // Append remaining text
        result.push_str(&text[last_end..]);
        result
    }

    /// Sanitize for specific context
    pub fn sanitize_for_context(&self, html: &str, context: ContentContext) -> String {
        let config = match context {
            ContentContext::PostContent => SanitizeConfig {
                level: SanitizationLevel::Standard,
                ..Default::default()
            },
            ContentContext::Comment => SanitizeConfig {
                level: SanitizationLevel::Basic,
                link_target_blank: true,
                ..Default::default()
            },
            ContentContext::Excerpt => SanitizeConfig {
                level: SanitizationLevel::Strict,
                ..Default::default()
            },
            ContentContext::Widget => SanitizeConfig {
                level: SanitizationLevel::Relaxed,
                ..Default::default()
            },
            ContentContext::AdminContent => SanitizeConfig {
                level: SanitizationLevel::Permissive,
                ..Default::default()
            },
            ContentContext::Email => SanitizeConfig {
                level: SanitizationLevel::Basic,
                linkify_urls: false,
                ..Default::default()
            },
        };

        Sanitizer::new(config).sanitize(html)
    }
}

/// Content context for sanitization
#[derive(Debug, Clone, Copy)]
pub enum ContentContext {
    /// Regular post/page content
    PostContent,

    /// User comments
    Comment,

    /// Post excerpts
    Excerpt,

    /// Widget content
    Widget,

    /// Admin-only content (trusted)
    AdminContent,

    /// Email content
    Email,
}

/// WordPress KSES-style sanitization based on user capabilities
pub struct KsesSanitizer {
    capability_tags: HashMap<String, HashSet<String>>,
}

impl Default for KsesSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl KsesSanitizer {
    pub fn new() -> Self {
        let mut capability_tags = HashMap::new();

        // Subscriber level
        capability_tags.insert("subscriber".to_string(), {
            let tags: HashSet<String> = ["a", "b", "i", "em", "strong", "br"]
                .iter().map(|s| s.to_string()).collect();
            tags
        });

        // Contributor level
        capability_tags.insert("contributor".to_string(), {
            let tags: HashSet<String> = basic_tags()
                .iter().map(|s| s.to_string()).collect();
            tags
        });

        // Author level
        capability_tags.insert("author".to_string(), {
            let mut tags: HashSet<String> = standard_tags()
                .iter().map(|s| s.to_string()).collect();
            tags.remove("iframe");
            tags
        });

        // Editor level
        capability_tags.insert("editor".to_string(), {
            standard_tags().iter().map(|s| s.to_string()).collect()
        });

        // Administrator level
        capability_tags.insert("administrator".to_string(), {
            relaxed_tags().iter().map(|s| s.to_string()).collect()
        });

        Self { capability_tags }
    }

    /// Sanitize based on user role
    pub fn sanitize(&self, html: &str, role: &str) -> String {
        let allowed_tags = self.capability_tags
            .get(role)
            .cloned()
            .unwrap_or_else(|| self.capability_tags.get("subscriber").cloned().unwrap());

        let config = SanitizeConfig {
            additional_tags: allowed_tags,
            ..Default::default()
        };

        Sanitizer::new(config).sanitize(html)
    }

    /// Check if user can use specific tag
    pub fn can_use_tag(&self, role: &str, tag: &str) -> bool {
        self.capability_tags
            .get(role)
            .map(|tags| tags.contains(tag))
            .unwrap_or(false)
    }

    /// Add custom tags for a role
    pub fn add_tags_for_role(&mut self, role: &str, tags: Vec<&str>) {
        let entry = self.capability_tags.entry(role.to_string()).or_insert_with(HashSet::new);
        entry.extend(tags.iter().map(|s| s.to_string()));
    }
}

/// Quick sanitization functions
pub fn sanitize(html: &str) -> String {
    Sanitizer::default().sanitize(html)
}

pub fn sanitize_strict(html: &str) -> String {
    Sanitizer::with_level(SanitizationLevel::Strict).sanitize(html)
}

pub fn sanitize_basic(html: &str) -> String {
    Sanitizer::with_level(SanitizationLevel::Basic).sanitize(html)
}

pub fn sanitize_relaxed(html: &str) -> String {
    Sanitizer::with_level(SanitizationLevel::Relaxed).sanitize(html)
}

pub fn strip_tags(html: &str) -> String {
    sanitize_strict(html)
}

/// Escape HTML entities
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Unescape HTML entities
pub fn unescape_html(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Escape for use in HTML attributes
pub fn escape_attr(text: &str) -> String {
    escape_html(text)
        .replace('\n', "&#10;")
        .replace('\r', "&#13;")
        .replace('\t', "&#9;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_all_html() {
        let html = "<p>Hello <strong>World</strong>!</p>";
        let result = sanitize_strict(html);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_basic_sanitization() {
        let html = "<p>Hello <script>alert('xss')</script><strong>World</strong>!</p>";
        let result = sanitize_basic(html);
        assert!(result.contains("<strong>"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_standard_sanitization() {
        let html = "<div><table><tr><td>Cell</td></tr></table></div>";
        let result = sanitize(html);
        assert!(result.contains("<table>"));
        assert!(result.contains("<td>"));
    }

    #[test]
    fn test_xss_prevention() {
        let html = r#"<a href="javascript:alert('xss')">Click</a>"#;
        let result = sanitize(html);
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn test_style_sanitization() {
        let html = r#"<p style="color: red; background: url(javascript:alert())">Test</p>"#;
        let sanitizer = Sanitizer::default();
        let result = sanitizer.sanitize(html);
        assert!(result.contains("color: red"));
        assert!(!result.contains("javascript"));
    }

    #[test]
    fn test_escape_html() {
        let text = "<script>alert('xss')</script>";
        let escaped = escape_html(text);
        assert_eq!(escaped, "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;");
    }

    #[test]
    fn test_kses_roles() {
        let kses = KsesSanitizer::new();

        assert!(kses.can_use_tag("administrator", "iframe"));
        assert!(!kses.can_use_tag("subscriber", "iframe"));
        assert!(kses.can_use_tag("contributor", "strong"));
    }
}
