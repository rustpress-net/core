//! # Shortcode Parser and Renderer
//!
//! WordPress-compatible shortcode system for RustPress.
//!
//! Supports:
//! - Self-closing shortcodes: `[gallery ids="1,2,3"]`
//! - Enclosing shortcodes: `[caption]Content[/caption]`
//! - Nested shortcodes (with proper handling)
//! - Named and positional attributes
//! - Custom shortcode handlers

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

/// Shortcode parsing and rendering errors
#[derive(Debug, Error)]
pub enum ShortcodeError {
    #[error("Unknown shortcode: {0}")]
    UnknownShortcode(String),

    #[error("Shortcode parse error: {0}")]
    ParseError(String),

    #[error("Handler error: {0}")]
    HandlerError(String),

    #[error("Nesting error: unclosed shortcode '{0}'")]
    UnclosedShortcode(String),

    #[error("Invalid attribute: {0}")]
    InvalidAttribute(String),

    #[error("Max nesting depth exceeded")]
    MaxNestingExceeded,
}

/// Represents a parsed shortcode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcode {
    /// The shortcode tag name (e.g., "gallery", "caption")
    pub tag: String,

    /// Named attributes (e.g., ids="1,2,3")
    pub attributes: ShortcodeAttributes,

    /// Content between opening and closing tags (None for self-closing)
    pub content: Option<String>,

    /// Whether this is a self-closing shortcode
    pub self_closing: bool,

    /// Original raw shortcode string
    pub raw: String,

    /// Position in the original content
    pub position: usize,
}

/// Shortcode attributes supporting both named and positional values
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShortcodeAttributes {
    /// Named attributes (key=value pairs)
    pub named: HashMap<String, String>,

    /// Positional attributes (values without keys)
    pub positional: Vec<String>,
}

impl ShortcodeAttributes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a named attribute
    pub fn get(&self, key: &str) -> Option<&str> {
        self.named.get(key).map(|s| s.as_str())
    }

    /// Get a named attribute or default
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.named.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// Get a named attribute as integer
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.named.get(key).and_then(|v| v.parse().ok())
    }

    /// Get a named attribute as boolean
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.named.get(key).map(|v| {
            matches!(v.to_lowercase().as_str(), "true" | "yes" | "1" | "on")
        })
    }

    /// Get a named attribute as comma-separated list
    pub fn get_list(&self, key: &str) -> Vec<String> {
        self.named
            .get(key)
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Get positional attribute by index
    pub fn positional(&self, index: usize) -> Option<&str> {
        self.positional.get(index).map(|s| s.as_str())
    }

    /// Check if attribute exists
    pub fn has(&self, key: &str) -> bool {
        self.named.contains_key(key)
    }

    /// Set a named attribute
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.named.insert(key.into(), value.into());
    }

    /// Add a positional attribute
    pub fn add_positional(&mut self, value: impl Into<String>) {
        self.positional.push(value.into());
    }
}

/// Result of shortcode rendering
#[derive(Debug, Clone)]
pub struct ShortcodeOutput {
    /// The rendered HTML output
    pub html: String,

    /// Whether to continue processing nested shortcodes
    pub process_nested: bool,

    /// Additional CSS classes to add
    pub css_classes: Vec<String>,

    /// Additional inline styles
    pub inline_styles: HashMap<String, String>,

    /// Scripts to enqueue
    pub scripts: Vec<String>,

    /// Stylesheets to enqueue
    pub styles: Vec<String>,
}

impl ShortcodeOutput {
    pub fn html(html: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            process_nested: true,
            css_classes: Vec::new(),
            inline_styles: HashMap::new(),
            scripts: Vec::new(),
            styles: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::html("")
    }

    pub fn with_script(mut self, script: impl Into<String>) -> Self {
        self.scripts.push(script.into());
        self
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.styles.push(style.into());
        self
    }

    pub fn no_nested(mut self) -> Self {
        self.process_nested = false;
        self
    }
}

/// Context passed to shortcode handlers
#[derive(Debug, Clone)]
pub struct ShortcodeContext {
    /// Current post ID (if available)
    pub post_id: Option<i64>,

    /// Current user ID (if available)
    pub user_id: Option<i64>,

    /// Whether we're in admin context
    pub is_admin: bool,

    /// Whether we're rendering for email
    pub is_email: bool,

    /// Whether we're rendering for RSS
    pub is_rss: bool,

    /// Current nesting depth
    pub depth: usize,

    /// Custom context data
    pub data: HashMap<String, String>,
}

impl Default for ShortcodeContext {
    fn default() -> Self {
        Self {
            post_id: None,
            user_id: None,
            is_admin: false,
            is_email: false,
            is_rss: false,
            depth: 0,
            data: HashMap::new(),
        }
    }
}

impl ShortcodeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_post(mut self, post_id: i64) -> Self {
        self.post_id = Some(post_id);
        self
    }

    pub fn with_user(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn admin(mut self) -> Self {
        self.is_admin = true;
        self
    }

    pub fn nested(&self) -> Self {
        let mut ctx = self.clone();
        ctx.depth += 1;
        ctx
    }
}

/// Trait for shortcode handlers
pub trait ShortcodeHandler: Send + Sync {
    /// Render the shortcode to HTML
    fn render(
        &self,
        shortcode: &Shortcode,
        context: &ShortcodeContext,
    ) -> Result<ShortcodeOutput, ShortcodeError>;

    /// Get the shortcode tag name
    fn tag(&self) -> &str;

    /// Get shortcode description
    fn description(&self) -> &str {
        ""
    }

    /// Whether this shortcode supports enclosing content
    fn supports_enclosing(&self) -> bool {
        true
    }

    /// Whether this shortcode can be nested
    fn supports_nesting(&self) -> bool {
        false
    }
}

/// Function-based shortcode handler
pub struct FnShortcodeHandler {
    tag: String,
    description: String,
    handler: Box<dyn Fn(&Shortcode, &ShortcodeContext) -> Result<ShortcodeOutput, ShortcodeError> + Send + Sync>,
    supports_enclosing: bool,
    supports_nesting: bool,
}

impl FnShortcodeHandler {
    pub fn new<F>(tag: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&Shortcode, &ShortcodeContext) -> Result<ShortcodeOutput, ShortcodeError> + Send + Sync + 'static,
    {
        Self {
            tag: tag.into(),
            description: String::new(),
            handler: Box::new(handler),
            supports_enclosing: true,
            supports_nesting: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn enclosing(mut self, supports: bool) -> Self {
        self.supports_enclosing = supports;
        self
    }

    pub fn nesting(mut self, supports: bool) -> Self {
        self.supports_nesting = supports;
        self
    }
}

impl ShortcodeHandler for FnShortcodeHandler {
    fn render(
        &self,
        shortcode: &Shortcode,
        context: &ShortcodeContext,
    ) -> Result<ShortcodeOutput, ShortcodeError> {
        (self.handler)(shortcode, context)
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn supports_enclosing(&self) -> bool {
        self.supports_enclosing
    }

    fn supports_nesting(&self) -> bool {
        self.supports_nesting
    }
}

/// Shortcode parser
pub struct ShortcodeParser {
    /// Maximum nesting depth
    max_depth: usize,
}

impl Default for ShortcodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcodeParser {
    pub fn new() -> Self {
        Self { max_depth: 10 }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Parse shortcodes from content
    pub fn parse(&self, content: &str) -> Result<Vec<Shortcode>, ShortcodeError> {
        let mut shortcodes = Vec::new();
        self.parse_recursive(content, 0, &mut shortcodes, 0)?;
        Ok(shortcodes)
    }

    fn parse_recursive(
        &self,
        content: &str,
        offset: usize,
        shortcodes: &mut Vec<Shortcode>,
        depth: usize,
    ) -> Result<(), ShortcodeError> {
        if depth > self.max_depth {
            return Err(ShortcodeError::MaxNestingExceeded);
        }

        // Regex for shortcode opening tag
        let opening_re = Regex::new(r"\[([a-zA-Z_][a-zA-Z0-9_-]*)(\s[^\]]*)?(/)?]").unwrap();

        let mut pos = 0;
        while pos < content.len() {
            if let Some(caps) = opening_re.captures(&content[pos..]) {
                let full_match = caps.get(0).unwrap();
                let tag = caps.get(1).unwrap().as_str().to_string();
                let attrs_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let self_closing = caps.get(3).is_some();

                let start_pos = pos + full_match.start();
                let attributes = self.parse_attributes(attrs_str)?;

                if self_closing {
                    // Self-closing shortcode
                    shortcodes.push(Shortcode {
                        tag,
                        attributes,
                        content: None,
                        self_closing: true,
                        raw: full_match.as_str().to_string(),
                        position: offset + start_pos,
                    });
                    pos += full_match.end();
                } else {
                    // Look for closing tag
                    let closing_pattern = format!(r"\[/{}\]", regex::escape(&tag));
                    let closing_re = Regex::new(&closing_pattern).unwrap();

                    let after_opening = pos + full_match.end();
                    if let Some(closing_match) = closing_re.find(&content[after_opening..]) {
                        let inner_content = &content[after_opening..after_opening + closing_match.start()];
                        let full_end = after_opening + closing_match.end();
                        let full_raw = &content[start_pos..full_end];

                        shortcodes.push(Shortcode {
                            tag: tag.clone(),
                            attributes,
                            content: Some(inner_content.to_string()),
                            self_closing: false,
                            raw: full_raw.to_string(),
                            position: offset + start_pos,
                        });

                        // Parse nested shortcodes in content
                        self.parse_recursive(
                            inner_content,
                            offset + after_opening,
                            shortcodes,
                            depth + 1,
                        )?;

                        pos = full_end;
                    } else {
                        // No closing tag found, treat as self-closing
                        shortcodes.push(Shortcode {
                            tag,
                            attributes,
                            content: None,
                            self_closing: true,
                            raw: full_match.as_str().to_string(),
                            position: offset + start_pos,
                        });
                        pos += full_match.end();
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Parse shortcode attributes
    fn parse_attributes(&self, attr_str: &str) -> Result<ShortcodeAttributes, ShortcodeError> {
        let mut attrs = ShortcodeAttributes::new();
        let trimmed = attr_str.trim();

        if trimmed.is_empty() {
            return Ok(attrs);
        }

        // Regex for named attributes: key="value" or key='value' or key=value
        let named_re = Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s\]]+))"#).unwrap();

        // Regex for positional attributes (quoted strings without key=)
        let positional_re = Regex::new(r#"(?:^|\s)(?:"([^"]*)"|'([^']*)')"#).unwrap();

        // First extract named attributes
        let mut named_positions: Vec<(usize, usize)> = Vec::new();
        for caps in named_re.captures_iter(trimmed) {
            let key = caps.get(1).unwrap().as_str().to_string();
            let value = caps.get(2)
                .or_else(|| caps.get(3))
                .or_else(|| caps.get(4))
                .map(|m| m.as_str())
                .unwrap_or("");

            attrs.named.insert(key, value.to_string());
            named_positions.push((caps.get(0).unwrap().start(), caps.get(0).unwrap().end()));
        }

        // Then look for positional attributes that weren't part of named ones
        for caps in positional_re.captures_iter(trimmed) {
            let full_match = caps.get(0).unwrap();
            let start = full_match.start();

            // Skip if this was part of a named attribute
            let is_named = named_positions.iter().any(|(s, e)| start >= *s && start < *e);
            if !is_named {
                let value = caps.get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                attrs.positional.push(value.to_string());
            }
        }

        Ok(attrs)
    }

    /// Find all shortcode tags in content (without full parsing)
    pub fn find_tags(&self, content: &str) -> Vec<String> {
        let re = Regex::new(r"\[([a-zA-Z_][a-zA-Z0-9_-]*)[\s\]/]").unwrap();
        let mut tags: Vec<String> = re
            .captures_iter(content)
            .map(|caps| caps.get(1).unwrap().as_str().to_string())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Check if content contains shortcodes
    pub fn has_shortcodes(&self, content: &str) -> bool {
        let re = Regex::new(r"\[[a-zA-Z_][a-zA-Z0-9_-]*[\s\]/]").unwrap();
        re.is_match(content)
    }
}

/// Shortcode registry and processor
pub struct ShortcodeRegistry {
    handlers: RwLock<HashMap<String, Box<dyn ShortcodeHandler>>>,
    parser: ShortcodeParser,
}

impl Default for ShortcodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcodeRegistry {
    pub fn new() -> Self {
        let registry = Self {
            handlers: RwLock::new(HashMap::new()),
            parser: ShortcodeParser::new(),
        };
        registry.register_builtins();
        registry
    }

    /// Register a shortcode handler
    pub fn register<H: ShortcodeHandler + 'static>(&self, handler: H) {
        let tag = handler.tag().to_string();
        self.handlers.write().unwrap().insert(tag, Box::new(handler));
    }

    /// Register a function-based shortcode
    pub fn register_fn<F>(&self, tag: impl Into<String>, handler: F)
    where
        F: Fn(&Shortcode, &ShortcodeContext) -> Result<ShortcodeOutput, ShortcodeError> + Send + Sync + 'static,
    {
        let tag_str = tag.into();
        self.register(FnShortcodeHandler::new(tag_str, handler));
    }

    /// Unregister a shortcode
    pub fn unregister(&self, tag: &str) {
        self.handlers.write().unwrap().remove(tag);
    }

    /// Check if a shortcode is registered
    pub fn is_registered(&self, tag: &str) -> bool {
        self.handlers.read().unwrap().contains_key(tag)
    }

    /// Get all registered shortcode tags
    pub fn tags(&self) -> Vec<String> {
        self.handlers.read().unwrap().keys().cloned().collect()
    }

    /// Process shortcodes in content
    pub fn process(&self, content: &str, context: &ShortcodeContext) -> Result<String, ShortcodeError> {
        if !self.parser.has_shortcodes(content) {
            return Ok(content.to_string());
        }

        self.process_recursive(content, context, 0)
    }

    fn process_recursive(
        &self,
        content: &str,
        context: &ShortcodeContext,
        depth: usize,
    ) -> Result<String, ShortcodeError> {
        if depth > 10 {
            return Err(ShortcodeError::MaxNestingExceeded);
        }

        let shortcodes = self.parser.parse(content)?;
        if shortcodes.is_empty() {
            return Ok(content.to_string());
        }

        let mut result = content.to_string();
        let handlers = self.handlers.read().unwrap();

        // Process shortcodes from last to first to maintain positions
        let mut sorted: Vec<_> = shortcodes.iter().collect();
        sorted.sort_by(|a, b| b.position.cmp(&a.position));

        for shortcode in sorted {
            if let Some(handler) = handlers.get(&shortcode.tag) {
                let output = handler.render(shortcode, context)?;

                // Process nested shortcodes in content if supported
                let final_html = if output.process_nested && handler.supports_nesting() {
                    self.process_recursive(&output.html, &context.nested(), depth + 1)?
                } else {
                    output.html
                };

                // Replace the shortcode with rendered output
                let raw_escaped = regex::escape(&shortcode.raw);
                let re = Regex::new(&raw_escaped).unwrap();
                result = re.replace(&result, final_html.as_str()).to_string();
            }
        }

        Ok(result)
    }

    /// Strip all shortcodes from content (leaving just text)
    pub fn strip(&self, content: &str) -> String {
        let re = Regex::new(r"\[/?[a-zA-Z_][a-zA-Z0-9_-]*[^\]]*\]").unwrap();
        re.replace_all(content, "").to_string()
    }

    /// Register built-in shortcodes
    fn register_builtins(&self) {
        // [caption] - Image caption
        self.register_fn("caption", |sc, _ctx| {
            let id = sc.attributes.get_or("id", "");
            let align = sc.attributes.get_or("align", "alignnone");
            let width = sc.attributes.get_int("width").unwrap_or(0);
            let caption = sc.attributes.get("caption").unwrap_or("");

            let content = sc.content.as_deref().unwrap_or("");

            let style = if width > 0 {
                format!(" style=\"width: {}px\"", width + 10)
            } else {
                String::new()
            };

            let id_attr = if !id.is_empty() {
                format!(" id=\"{}\"", id)
            } else {
                String::new()
            };

            let html = format!(
                "<figure{}class=\"wp-caption {}\"{}>{}<figcaption class=\"wp-caption-text\">{}</figcaption></figure>",
                id_attr,
                align,
                style,
                content,
                if caption.is_empty() { content } else { caption }
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [gallery] - Image gallery
        self.register_fn("gallery", |sc, _ctx| {
            let ids = sc.attributes.get_list("ids");
            let columns = sc.attributes.get_int("columns").unwrap_or(3) as usize;
            let size = sc.attributes.get_or("size", "thumbnail");
            let link = sc.attributes.get_or("link", "attachment");

            let mut html = format!(
                "<div class=\"gallery gallery-columns-{}\" data-size=\"{}\" data-link=\"{}\">",
                columns, size, link
            );

            for (i, id) in ids.iter().enumerate() {
                let column_class = if (i + 1) % columns == 0 { " gallery-item-last" } else { "" };
                html.push_str(&format!(
                    "<figure class=\"gallery-item{}\"><div class=\"gallery-icon\"><a href=\"#\" data-attachment-id=\"{}\"><img src=\"/media/{}/{}\" alt=\"\"/></a></div></figure>",
                    column_class, id, size, id
                ));
            }

            html.push_str("</div>");
            Ok(ShortcodeOutput::html(html).with_style("/css/gallery.css"))
        });

        // [audio] - Audio player
        self.register_fn("audio", |sc, _ctx| {
            let src = sc.attributes.get("src").or_else(|| sc.attributes.positional(0)).unwrap_or("");
            let autoplay = sc.attributes.get_bool("autoplay").unwrap_or(false);
            let loop_attr = sc.attributes.get_bool("loop").unwrap_or(false);
            let preload = sc.attributes.get_or("preload", "none");

            let autoplay_attr = if autoplay { " autoplay" } else { "" };
            let loop_attr = if loop_attr { " loop" } else { "" };

            let html = format!(
                "<audio controls{}{} preload=\"{}\"><source src=\"{}\" type=\"audio/mpeg\">Your browser does not support the audio element.</audio>",
                autoplay_attr, loop_attr, preload, src
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [video] - Video player
        self.register_fn("video", |sc, _ctx| {
            let src = sc.attributes.get("src").or_else(|| sc.attributes.positional(0)).unwrap_or("");
            let poster = sc.attributes.get("poster").unwrap_or("");
            let width = sc.attributes.get_int("width");
            let height = sc.attributes.get_int("height");
            let autoplay = sc.attributes.get_bool("autoplay").unwrap_or(false);
            let loop_attr = sc.attributes.get_bool("loop").unwrap_or(false);
            let muted = sc.attributes.get_bool("muted").unwrap_or(false);
            let preload = sc.attributes.get_or("preload", "metadata");

            let mut attrs = String::new();
            if let Some(w) = width {
                attrs.push_str(&format!(" width=\"{}\"", w));
            }
            if let Some(h) = height {
                attrs.push_str(&format!(" height=\"{}\"", h));
            }
            if !poster.is_empty() {
                attrs.push_str(&format!(" poster=\"{}\"", poster));
            }
            if autoplay {
                attrs.push_str(" autoplay");
            }
            if loop_attr {
                attrs.push_str(" loop");
            }
            if muted {
                attrs.push_str(" muted");
            }

            let html = format!(
                "<video controls{} preload=\"{}\"><source src=\"{}\" type=\"video/mp4\">Your browser does not support the video element.</video>",
                attrs, preload, src
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [embed] - oEmbed wrapper
        self.register_fn("embed", |sc, _ctx| {
            let url = sc.content.as_deref()
                .or_else(|| sc.attributes.get("url"))
                .or_else(|| sc.attributes.positional(0))
                .unwrap_or("");
            let width = sc.attributes.get_int("width");
            let height = sc.attributes.get_int("height");

            let mut style = String::new();
            if let Some(w) = width {
                style.push_str(&format!("max-width: {}px; ", w));
            }
            if let Some(h) = height {
                style.push_str(&format!("max-height: {}px; ", h));
            }

            let style_attr = if !style.is_empty() {
                format!(" style=\"{}\"", style)
            } else {
                String::new()
            };

            // Placeholder - actual oEmbed resolution happens in oembed module
            let html = format!(
                "<div class=\"wp-embed-wrapper\"{} data-url=\"{}\"><a href=\"{}\">{}</a></div>",
                style_attr, url, url, url
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [code] - Code block
        self.register_fn("code", |sc, _ctx| {
            let language = sc.attributes.get("lang")
                .or_else(|| sc.attributes.get("language"))
                .or_else(|| sc.attributes.positional(0))
                .unwrap_or("plaintext");
            let content = sc.content.as_deref().unwrap_or("");

            // Escape HTML in code content
            let escaped = content
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");

            let html = format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                language, escaped
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [quote] - Blockquote
        self.register_fn("quote", |sc, _ctx| {
            let author = sc.attributes.get("author").unwrap_or("");
            let source = sc.attributes.get("source").unwrap_or("");
            let content = sc.content.as_deref().unwrap_or("");

            let mut html = format!("<blockquote class=\"wp-quote\"><p>{}</p>", content);

            if !author.is_empty() || !source.is_empty() {
                html.push_str("<footer>");
                if !author.is_empty() {
                    html.push_str(&format!("<cite>{}</cite>", author));
                }
                if !source.is_empty() {
                    if !author.is_empty() {
                        html.push_str(", ");
                    }
                    html.push_str(&format!("<span class=\"source\">{}</span>", source));
                }
                html.push_str("</footer>");
            }

            html.push_str("</blockquote>");
            Ok(ShortcodeOutput::html(html))
        });

        // [button] - Button link
        self.register_fn("button", |sc, _ctx| {
            let url = sc.attributes.get("url").or_else(|| sc.attributes.get("href")).unwrap_or("#");
            let text = sc.content.as_deref()
                .or_else(|| sc.attributes.get("text"))
                .unwrap_or("Click here");
            let target = sc.attributes.get("target").unwrap_or("_self");
            let style = sc.attributes.get("style").unwrap_or("default");
            let size = sc.attributes.get("size").unwrap_or("medium");

            let html = format!(
                "<a href=\"{}\" target=\"{}\" class=\"wp-button wp-button-{} wp-button-size-{}\">{}</a>",
                url, target, style, size, text
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [columns] and [column] - Column layout
        self.register_fn("columns", |sc, _ctx| {
            let cols = sc.attributes.get_int("cols").unwrap_or(2);
            let gap = sc.attributes.get_or("gap", "20px");
            let content = sc.content.as_deref().unwrap_or("");

            let html = format!(
                "<div class=\"wp-columns wp-columns-{}\" style=\"gap: {}\">{}</div>",
                cols, gap, content
            );

            Ok(ShortcodeOutput::html(html))
        });

        self.register_fn("column", |sc, _ctx| {
            let span = sc.attributes.get_int("span").unwrap_or(1);
            let content = sc.content.as_deref().unwrap_or("");

            let html = format!(
                "<div class=\"wp-column\" style=\"grid-column: span {}\">{}</div>",
                span, content
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [accordion] and [accordion_item] - Accordion component
        self.register_fn("accordion", |sc, _ctx| {
            let content = sc.content.as_deref().unwrap_or("");
            let html = format!("<div class=\"wp-accordion\">{}</div>", content);
            Ok(ShortcodeOutput::html(html).with_script("/js/accordion.js"))
        });

        self.register_fn("accordion_item", |sc, _ctx| {
            let title = sc.attributes.get("title").unwrap_or("Accordion Item");
            let open = sc.attributes.get_bool("open").unwrap_or(false);
            let content = sc.content.as_deref().unwrap_or("");

            let open_class = if open { " open" } else { "" };
            let html = format!(
                "<div class=\"wp-accordion-item{}\"><div class=\"wp-accordion-header\">{}</div><div class=\"wp-accordion-content\">{}</div></div>",
                open_class, title, content
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [tabs] and [tab] - Tabbed content
        self.register_fn("tabs", |sc, _ctx| {
            let content = sc.content.as_deref().unwrap_or("");
            let html = format!("<div class=\"wp-tabs\">{}</div>", content);
            Ok(ShortcodeOutput::html(html).with_script("/js/tabs.js"))
        });

        self.register_fn("tab", |sc, _ctx| {
            let title = sc.attributes.get("title").unwrap_or("Tab");
            let id = sc.attributes.get("id").unwrap_or("");
            let content = sc.content.as_deref().unwrap_or("");

            let id_attr = if !id.is_empty() {
                format!(" id=\"{}\"", id)
            } else {
                String::new()
            };

            let html = format!(
                "<div class=\"wp-tab\"{} data-title=\"{}\">{}</div>",
                id_attr, title, content
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [notice] - Notice/alert box
        self.register_fn("notice", |sc, _ctx| {
            let notice_type = sc.attributes.get("type")
                .or_else(|| sc.attributes.positional(0))
                .unwrap_or("info");
            let title = sc.attributes.get("title").unwrap_or("");
            let dismissible = sc.attributes.get_bool("dismissible").unwrap_or(false);
            let content = sc.content.as_deref().unwrap_or("");

            let dismiss_class = if dismissible { " dismissible" } else { "" };
            let dismiss_btn = if dismissible {
                "<button class=\"wp-notice-dismiss\" aria-label=\"Dismiss\">&times;</button>"
            } else {
                ""
            };

            let title_html = if !title.is_empty() {
                format!("<div class=\"wp-notice-title\">{}</div>", title)
            } else {
                String::new()
            };

            let html = format!(
                "<div class=\"wp-notice wp-notice-{}{}\">{}{}<div class=\"wp-notice-content\">{}</div></div>",
                notice_type, dismiss_class, dismiss_btn, title_html, content
            );

            Ok(ShortcodeOutput::html(html))
        });

        // [spoiler] - Collapsible spoiler content
        self.register_fn("spoiler", |sc, _ctx| {
            let title = sc.attributes.get("title").unwrap_or("Spoiler");
            let content = sc.content.as_deref().unwrap_or("");

            let html = format!(
                "<details class=\"wp-spoiler\"><summary>{}</summary><div class=\"wp-spoiler-content\">{}</div></details>",
                title, content
            );

            Ok(ShortcodeOutput::html(html))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_self_closing() {
        let parser = ShortcodeParser::new();
        let shortcodes = parser.parse("[gallery ids=\"1,2,3\"/]").unwrap();

        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].tag, "gallery");
        assert!(shortcodes[0].self_closing);
        assert_eq!(shortcodes[0].attributes.get("ids"), Some("1,2,3"));
    }

    #[test]
    fn test_parse_enclosing() {
        let parser = ShortcodeParser::new();
        let shortcodes = parser.parse("[caption]Image caption[/caption]").unwrap();

        assert_eq!(shortcodes.len(), 1);
        assert_eq!(shortcodes[0].tag, "caption");
        assert!(!shortcodes[0].self_closing);
        assert_eq!(shortcodes[0].content, Some("Image caption".to_string()));
    }

    #[test]
    fn test_parse_attributes() {
        let parser = ShortcodeParser::new();
        let attrs = parser.parse_attributes(r#"id="test" width='300' height=200"#).unwrap();

        assert_eq!(attrs.get("id"), Some("test"));
        assert_eq!(attrs.get("width"), Some("300"));
        assert_eq!(attrs.get("height"), Some("200"));
    }

    #[test]
    fn test_registry_process() {
        let registry = ShortcodeRegistry::new();
        let ctx = ShortcodeContext::new();

        let result = registry.process("[code lang=\"rust\"]fn main() {}[/code]", &ctx).unwrap();
        assert!(result.contains("<pre><code"));
        assert!(result.contains("language-rust"));
    }

    #[test]
    fn test_strip_shortcodes() {
        let registry = ShortcodeRegistry::new();
        let result = registry.strip("Hello [gallery ids=\"1,2,3\"/] World");
        assert_eq!(result, "Hello  World");
    }
}
