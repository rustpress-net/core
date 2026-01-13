//! Asset Minification and Bundling
//!
//! Minifies CSS, JavaScript, and HTML content. Provides bundling support for assets.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minification errors
#[derive(Debug, Error)]
pub enum MinificationError {
    #[error("Failed to minify {asset_type}: {message}")]
    MinifyFailed { asset_type: String, message: String },

    #[error("Bundle error: {0}")]
    BundleError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Minification configuration
#[derive(Debug, Clone)]
pub struct MinificationConfig {
    /// Enable CSS minification
    pub minify_css: bool,
    /// Enable JavaScript minification
    pub minify_js: bool,
    /// Enable HTML minification
    pub minify_html: bool,
    /// Remove comments
    pub remove_comments: bool,
    /// Collapse whitespace
    pub collapse_whitespace: bool,
    /// Mangle variable names (JS only)
    pub mangle_names: bool,
    /// Generate source maps
    pub source_maps: bool,
    /// Cache minified assets
    pub cache_enabled: bool,
    /// Maximum cache entries
    pub max_cache_entries: usize,
}

impl Default for MinificationConfig {
    fn default() -> Self {
        Self {
            minify_css: true,
            minify_js: true,
            minify_html: true,
            remove_comments: true,
            collapse_whitespace: true,
            mangle_names: false,
            source_maps: true,
            cache_enabled: true,
            max_cache_entries: 1000,
        }
    }
}

/// Asset minifier
pub struct Minifier {
    config: MinificationConfig,
    cache: Arc<RwLock<HashMap<String, MinifiedAsset>>>,
}

/// Minified asset result
#[derive(Debug, Clone)]
pub struct MinifiedAsset {
    /// Minified content
    pub content: String,
    /// Original size in bytes
    pub original_size: usize,
    /// Minified size in bytes
    pub minified_size: usize,
    /// Compression ratio (0.0-1.0)
    pub compression_ratio: f64,
    /// Source map (if generated)
    pub source_map: Option<String>,
    /// Content hash for cache busting
    pub hash: String,
}

impl Minifier {
    pub fn new(config: MinificationConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Minify CSS content
    pub fn minify_css(&self, css: &str) -> Result<MinifiedAsset, MinificationError> {
        if !self.config.minify_css {
            return Ok(self.create_passthrough(css));
        }

        // Check cache
        let cache_key = format!("css:{}", blake3::hash(css.as_bytes()).to_hex());
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.read().get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let original_size = css.len();
        let mut minified = css.to_string();

        // Remove comments
        if self.config.remove_comments {
            minified = remove_css_comments(&minified);
        }

        // Collapse whitespace
        if self.config.collapse_whitespace {
            minified = collapse_css_whitespace(&minified);
        }

        // Additional CSS-specific optimizations
        minified = optimize_css(&minified);

        let minified_size = minified.len();
        let hash = blake3::hash(minified.as_bytes()).to_hex()[..16].to_string();

        let result = MinifiedAsset {
            content: minified,
            original_size,
            minified_size,
            compression_ratio: 1.0 - (minified_size as f64 / original_size.max(1) as f64),
            source_map: None,
            hash,
        };

        // Cache result
        if self.config.cache_enabled {
            self.cache_result(&cache_key, &result);
        }

        Ok(result)
    }

    /// Minify JavaScript content
    pub fn minify_js(&self, js: &str) -> Result<MinifiedAsset, MinificationError> {
        if !self.config.minify_js {
            return Ok(self.create_passthrough(js));
        }

        // Check cache
        let cache_key = format!("js:{}", blake3::hash(js.as_bytes()).to_hex());
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.read().get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let original_size = js.len();
        let mut minified = js.to_string();

        // Remove comments
        if self.config.remove_comments {
            minified = remove_js_comments(&minified);
        }

        // Collapse whitespace
        if self.config.collapse_whitespace {
            minified = collapse_js_whitespace(&minified);
        }

        // JavaScript-specific optimizations
        minified = optimize_js(&minified);

        let minified_size = minified.len();
        let hash = blake3::hash(minified.as_bytes()).to_hex()[..16].to_string();

        let result = MinifiedAsset {
            content: minified,
            original_size,
            minified_size,
            compression_ratio: 1.0 - (minified_size as f64 / original_size.max(1) as f64),
            source_map: None,
            hash,
        };

        // Cache result
        if self.config.cache_enabled {
            self.cache_result(&cache_key, &result);
        }

        Ok(result)
    }

    /// Minify HTML content
    pub fn minify_html(&self, html: &str) -> Result<MinifiedAsset, MinificationError> {
        if !self.config.minify_html {
            return Ok(self.create_passthrough(html));
        }

        // Check cache
        let cache_key = format!("html:{}", blake3::hash(html.as_bytes()).to_hex());
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.read().get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let original_size = html.len();
        let mut minified = html.to_string();

        // Remove HTML comments (but preserve conditional comments)
        if self.config.remove_comments {
            minified = remove_html_comments(&minified);
        }

        // Collapse whitespace
        if self.config.collapse_whitespace {
            minified = collapse_html_whitespace(&minified);
        }

        // HTML-specific optimizations
        minified = optimize_html(&minified);

        let minified_size = minified.len();
        let hash = blake3::hash(minified.as_bytes()).to_hex()[..16].to_string();

        let result = MinifiedAsset {
            content: minified,
            original_size,
            minified_size,
            compression_ratio: 1.0 - (minified_size as f64 / original_size.max(1) as f64),
            source_map: None,
            hash,
        };

        // Cache result
        if self.config.cache_enabled {
            self.cache_result(&cache_key, &result);
        }

        Ok(result)
    }

    fn create_passthrough(&self, content: &str) -> MinifiedAsset {
        let size = content.len();
        MinifiedAsset {
            content: content.to_string(),
            original_size: size,
            minified_size: size,
            compression_ratio: 0.0,
            source_map: None,
            hash: blake3::hash(content.as_bytes()).to_hex()[..16].to_string(),
        }
    }

    fn cache_result(&self, key: &str, result: &MinifiedAsset) {
        let mut cache = self.cache.write();

        // Evict if over capacity
        if cache.len() >= self.config.max_cache_entries {
            // Simple eviction: remove first entry
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }

        cache.insert(key.to_string(), result.clone());
    }

    /// Clear minification cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let total_original: usize = cache.values().map(|a| a.original_size).sum();
        let total_minified: usize = cache.values().map(|a| a.minified_size).sum();

        CacheStats {
            entries: cache.len(),
            total_original_bytes: total_original,
            total_minified_bytes: total_minified,
            total_savings_bytes: total_original.saturating_sub(total_minified),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_original_bytes: usize,
    pub total_minified_bytes: usize,
    pub total_savings_bytes: usize,
}

// CSS minification helpers
fn remove_css_comments(css: &str) -> String {
    let re = regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    re.replace_all(css, "").to_string()
}

fn collapse_css_whitespace(css: &str) -> String {
    let mut result = css.to_string();

    // Replace multiple whitespace with single space
    let re = regex::Regex::new(r"\s+").unwrap();
    result = re.replace_all(&result, " ").to_string();

    // Remove whitespace around certain characters
    for ch in &[':', ';', '{', '}', ',', '>', '+', '~'] {
        result = result.replace(&format!(" {} ", ch), &ch.to_string());
        result = result.replace(&format!(" {}", ch), &ch.to_string());
        result = result.replace(&format!("{} ", ch), &ch.to_string());
    }

    result.trim().to_string()
}

fn optimize_css(css: &str) -> String {
    let mut result = css.to_string();

    // Remove last semicolon before closing brace
    result = result.replace(";}", "}");

    // Shorten color values (e.g., #aabbcc -> #abc)
    // Rust regex doesn't support backreferences, so we find and shorten manually
    let color_re = regex::Regex::new(r"#([0-9a-fA-F]{6})\b").unwrap();
    result = color_re.replace_all(&result, |caps: &regex::Captures| {
        let hex = &caps[1];
        let chars: Vec<char> = hex.chars().collect();
        // Check if it's a shortable color (pairs are identical)
        if chars[0].eq_ignore_ascii_case(&chars[1])
            && chars[2].eq_ignore_ascii_case(&chars[3])
            && chars[4].eq_ignore_ascii_case(&chars[5]) {
            format!("#{}{}{}", chars[0], chars[2], chars[4])
        } else {
            format!("#{}", hex)
        }
    }).to_string();

    // Remove units from zero values
    let zero_re = regex::Regex::new(r"\b0(px|em|rem|%|pt|cm|mm|in|pc|ex|ch|vw|vh|vmin|vmax)\b").unwrap();
    result = zero_re.replace_all(&result, "0").to_string();

    // Shorten common values
    result = result.replace("font-weight:bold", "font-weight:700");
    result = result.replace("font-weight:normal", "font-weight:400");

    result
}

// JavaScript minification helpers
fn remove_js_comments(js: &str) -> String {
    let mut result = String::new();
    let mut chars = js.chars().peekable();
    let mut in_string = false;
    let mut string_char = '"';
    let mut in_single_comment = false;
    let mut in_multi_comment = false;
    let mut prev_char = ' ';

    while let Some(c) = chars.next() {
        if in_single_comment {
            if c == '\n' {
                in_single_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_multi_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_multi_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = c;
            continue;
        }

        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = c;
            result.push(c);
            continue;
        }

        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    in_single_comment = true;
                    chars.next();
                    continue;
                } else if next == '*' {
                    in_multi_comment = true;
                    chars.next();
                    continue;
                }
            }
        }

        result.push(c);
        prev_char = c;
    }

    result
}

fn collapse_js_whitespace(js: &str) -> String {
    let mut result = String::new();
    let mut chars = js.chars().peekable();
    let mut in_string = false;
    let mut string_char = '"';
    let mut last_was_space = false;
    let mut prev_char = ' ';

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            if c == string_char && prev_char != '\\' {
                in_string = false;
            }
            prev_char = c;
            last_was_space = false;
            continue;
        }

        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = c;
            result.push(c);
            last_was_space = false;
            continue;
        }

        if c.is_whitespace() {
            if !last_was_space {
                // Keep one space if between alphanumeric characters
                let last_char = result.chars().last().unwrap_or(' ');
                if last_char.is_alphanumeric() || last_char == '_' || last_char == '$' {
                    if let Some(&next) = chars.peek() {
                        if next.is_alphanumeric() || next == '_' || next == '$' {
                            result.push(' ');
                        }
                    }
                }
                last_was_space = true;
            }
            continue;
        }

        result.push(c);
        prev_char = c;
        last_was_space = false;
    }

    result
}

fn optimize_js(js: &str) -> String {
    let mut result = js.to_string();

    // Remove unnecessary semicolons
    result = result.replace(";;", ";");

    // Optimize boolean literals
    result = result.replace("true===", "!0===");
    result = result.replace("===true", "===!0");
    result = result.replace("false===", "!1===");
    result = result.replace("===false", "===!1");

    result
}

// HTML minification helpers
fn remove_html_comments(html: &str) -> String {
    // Remove regular HTML comments but preserve conditional comments
    // Rust regex doesn't support look-ahead, so we match all comments and filter
    let re = regex::Regex::new(r"<!--[\s\S]*?-->").unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let comment = &caps[0];
        // Preserve IE conditional comments
        if comment.starts_with("<!--[if") || comment.starts_with("<!--[endif") {
            comment.to_string()
        } else {
            String::new()
        }
    }).to_string()
}

fn collapse_html_whitespace(html: &str) -> String {
    let mut result = html.to_string();

    // Collapse whitespace outside of pre, script, style, and textarea tags
    let preserve_tags = ["pre", "script", "style", "textarea", "code"];

    // Simple whitespace collapse (not preserving special tags in this basic version)
    let re = regex::Regex::new(r">\s+<").unwrap();
    result = re.replace_all(&result, "><").to_string();

    // Collapse multiple spaces to one
    let re = regex::Regex::new(r" {2,}").unwrap();
    result = re.replace_all(&result, " ").to_string();

    result
}

fn optimize_html(html: &str) -> String {
    let mut result = html.to_string();

    // Remove optional closing tags (simplified)
    // In a production system, this would be more sophisticated

    // Remove boolean attribute values
    result = result.replace("=\"true\"", "");
    result = result.replace("='true'", "");

    // Simplify common boolean attributes
    result = result.replace(" disabled=\"disabled\"", " disabled");
    result = result.replace(" checked=\"checked\"", " checked");
    result = result.replace(" selected=\"selected\"", " selected");
    result = result.replace(" readonly=\"readonly\"", " readonly");
    result = result.replace(" required=\"required\"", " required");

    result
}

/// Asset bundler
pub struct Bundler {
    config: BundlerConfig,
    minifier: Arc<Minifier>,
}

/// Bundler configuration
#[derive(Debug, Clone)]
pub struct BundlerConfig {
    /// Output directory for bundles
    pub output_dir: PathBuf,
    /// Enable minification
    pub minify: bool,
    /// Generate source maps
    pub source_maps: bool,
    /// Bundle name pattern
    pub filename_pattern: String,
}

impl Default for BundlerConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist"),
            minify: true,
            source_maps: true,
            filename_pattern: "[name].[hash].js".to_string(),
        }
    }
}

/// Bundle entry
#[derive(Debug, Clone)]
pub struct BundleEntry {
    /// Entry name
    pub name: String,
    /// Source file paths
    pub sources: Vec<PathBuf>,
    /// Bundle type
    pub bundle_type: BundleType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BundleType {
    JavaScript,
    Css,
}

/// Bundle result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResult {
    /// Bundle name
    pub name: String,
    /// Output filename
    pub filename: String,
    /// Content hash
    pub hash: String,
    /// Original total size
    pub original_size: usize,
    /// Final bundle size
    pub bundle_size: usize,
    /// Source files included
    pub sources: Vec<String>,
}

impl Bundler {
    pub fn new(config: BundlerConfig, minifier: Arc<Minifier>) -> Self {
        Self { config, minifier }
    }

    /// Bundle JavaScript files
    pub fn bundle_js(&self, entry: &BundleEntry, contents: &[String]) -> Result<BundleResult, MinificationError> {
        if entry.bundle_type != BundleType::JavaScript {
            return Err(MinificationError::InvalidInput(
                "Expected JavaScript bundle type".to_string(),
            ));
        }

        // Concatenate all sources
        let mut combined = String::new();
        let mut original_size = 0;

        for content in contents {
            original_size += content.len();
            combined.push_str(content);
            combined.push('\n');
        }

        // Wrap in IIFE to avoid scope pollution
        let wrapped = format!("(function(){{{}}})()", combined);

        // Minify if enabled
        let final_content = if self.config.minify {
            let minified = self.minifier.minify_js(&wrapped)?;
            minified.content
        } else {
            wrapped
        };

        let hash = blake3::hash(final_content.as_bytes()).to_hex()[..8].to_string();
        let filename = self.config.filename_pattern
            .replace("[name]", &entry.name)
            .replace("[hash]", &hash);

        Ok(BundleResult {
            name: entry.name.clone(),
            filename,
            hash,
            original_size,
            bundle_size: final_content.len(),
            sources: entry.sources.iter().map(|p| p.display().to_string()).collect(),
        })
    }

    /// Bundle CSS files
    pub fn bundle_css(&self, entry: &BundleEntry, contents: &[String]) -> Result<BundleResult, MinificationError> {
        if entry.bundle_type != BundleType::Css {
            return Err(MinificationError::InvalidInput(
                "Expected CSS bundle type".to_string(),
            ));
        }

        // Concatenate all sources
        let mut combined = String::new();
        let mut original_size = 0;

        for content in contents {
            original_size += content.len();
            combined.push_str(content);
            combined.push('\n');
        }

        // Minify if enabled
        let final_content = if self.config.minify {
            let minified = self.minifier.minify_css(&combined)?;
            minified.content
        } else {
            combined
        };

        let hash = blake3::hash(final_content.as_bytes()).to_hex()[..8].to_string();
        let filename = self.config.filename_pattern
            .replace("[name]", &entry.name)
            .replace("[hash]", &hash)
            .replace(".js", ".css");

        Ok(BundleResult {
            name: entry.name.clone(),
            filename,
            hash,
            original_size,
            bundle_size: final_content.len(),
            sources: entry.sources.iter().map(|p| p.display().to_string()).collect(),
        })
    }
}

/// Inline critical CSS extractor
pub struct CriticalCssExtractor {
    /// Viewport width for above-the-fold calculation
    pub viewport_width: u32,
    /// Viewport height for above-the-fold calculation
    pub viewport_height: u32,
}

impl CriticalCssExtractor {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            viewport_width,
            viewport_height,
        }
    }

    /// Extract critical CSS rules (simplified heuristic version)
    pub fn extract(&self, css: &str, html: &str) -> String {
        // This is a simplified version. A full implementation would:
        // 1. Parse the CSS into an AST
        // 2. Parse the HTML and determine above-the-fold elements
        // 3. Match CSS selectors against those elements
        // 4. Return only the matching rules

        // For now, extract rules that likely affect above-the-fold content
        let critical_selectors = [
            "html", "body", "header", "nav", "main", "h1", "h2", ".hero",
            ".banner", ".header", ".navigation", ".nav", ".menu",
        ];

        let mut critical_css = String::new();
        let rule_re = regex::Regex::new(r"([^{}]+)\{([^{}]*)\}").unwrap();

        for cap in rule_re.captures_iter(css) {
            let selector = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let rules = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Check if selector matches any critical patterns
            let is_critical = critical_selectors.iter().any(|cs| {
                selector.contains(cs)
            });

            if is_critical {
                critical_css.push_str(selector);
                critical_css.push('{');
                critical_css.push_str(rules);
                critical_css.push('}');
            }
        }

        critical_css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_minification() {
        let minifier = Minifier::new(MinificationConfig::default());

        let css = r#"
            /* This is a comment */
            body {
                margin: 0px;
                padding: 0px;
                color: #ffffff;
            }
        "#;

        let result = minifier.minify_css(css).unwrap();
        assert!(result.minified_size < result.original_size);
        assert!(!result.content.contains("/*"));
        assert!(result.content.contains("margin:0"));
        assert!(result.content.contains("#fff"));
    }

    #[test]
    fn test_js_minification() {
        let minifier = Minifier::new(MinificationConfig::default());

        let js = r#"
            // Single line comment
            /* Multi line
               comment */
            function hello(name) {
                console.log("Hello, " + name);
            }
        "#;

        let result = minifier.minify_js(js).unwrap();
        assert!(result.minified_size < result.original_size);
        assert!(!result.content.contains("//"));
        assert!(!result.content.contains("/*"));
    }

    #[test]
    fn test_html_minification() {
        let minifier = Minifier::new(MinificationConfig::default());

        let html = r#"
            <!DOCTYPE html>
            <html>
                <!-- Comment -->
                <head>
                    <title>Test</title>
                </head>
                <body>
                    <div disabled="disabled">
                        Content
                    </div>
                </body>
            </html>
        "#;

        let result = minifier.minify_html(html).unwrap();
        assert!(result.minified_size < result.original_size);
        assert!(!result.content.contains("<!-- Comment -->"));
    }
}
