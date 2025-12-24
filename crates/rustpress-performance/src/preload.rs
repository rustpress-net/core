//! Preloading and Prefetching Hints
//!
//! Generates resource hints for browsers to optimize loading performance.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Resource hint type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HintType {
    /// Preload - fetch with high priority
    Preload,
    /// Prefetch - fetch during idle time
    Prefetch,
    /// Preconnect - establish early connection
    Preconnect,
    /// DNS prefetch - resolve DNS early
    DnsPrefetch,
    /// Module preload - preload ES module
    ModulePreload,
}

impl HintType {
    pub fn as_rel(&self) -> &'static str {
        match self {
            Self::Preload => "preload",
            Self::Prefetch => "prefetch",
            Self::Preconnect => "preconnect",
            Self::DnsPrefetch => "dns-prefetch",
            Self::ModulePreload => "modulepreload",
        }
    }
}

/// Resource type for preload 'as' attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceAs {
    Script,
    Style,
    Font,
    Image,
    Document,
    Fetch,
    Audio,
    Video,
    Track,
    Worker,
}

impl ResourceAs {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Style => "style",
            Self::Font => "font",
            Self::Image => "image",
            Self::Document => "document",
            Self::Fetch => "fetch",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Track => "track",
            Self::Worker => "worker",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" => Some(Self::Script),
            "css" => Some(Self::Style),
            "woff" | "woff2" | "ttf" | "otf" | "eot" => Some(Self::Font),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "svg" => Some(Self::Image),
            "html" | "htm" => Some(Self::Document),
            "mp3" | "wav" | "ogg" | "flac" => Some(Self::Audio),
            "mp4" | "webm" | "avi" => Some(Self::Video),
            "vtt" | "srt" => Some(Self::Track),
            _ => None,
        }
    }
}

/// Resource hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHint {
    /// Hint type
    pub hint_type: HintType,
    /// Resource URL
    pub href: String,
    /// Resource type (for preload)
    pub resource_as: Option<ResourceAs>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Crossorigin attribute
    pub crossorigin: Option<CrossOrigin>,
    /// Media query
    pub media: Option<String>,
    /// Integrity hash (SRI)
    pub integrity: Option<String>,
    /// Fetch priority hint
    pub fetch_priority: Option<FetchPriority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossOrigin {
    Anonymous,
    UseCredentials,
}

impl CrossOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::UseCredentials => "use-credentials",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FetchPriority {
    High,
    Low,
    Auto,
}

impl FetchPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Low => "low",
            Self::Auto => "auto",
        }
    }
}

impl ResourceHint {
    /// Create a preload hint
    pub fn preload(href: &str, resource_as: ResourceAs) -> Self {
        let crossorigin = match resource_as {
            ResourceAs::Font => Some(CrossOrigin::Anonymous),
            _ => None,
        };

        Self {
            hint_type: HintType::Preload,
            href: href.to_string(),
            resource_as: Some(resource_as),
            mime_type: None,
            crossorigin,
            media: None,
            integrity: None,
            fetch_priority: None,
        }
    }

    /// Create a prefetch hint
    pub fn prefetch(href: &str) -> Self {
        Self {
            hint_type: HintType::Prefetch,
            href: href.to_string(),
            resource_as: None,
            mime_type: None,
            crossorigin: None,
            media: None,
            integrity: None,
            fetch_priority: None,
        }
    }

    /// Create a preconnect hint
    pub fn preconnect(origin: &str) -> Self {
        Self {
            hint_type: HintType::Preconnect,
            href: origin.to_string(),
            resource_as: None,
            mime_type: None,
            crossorigin: Some(CrossOrigin::Anonymous),
            media: None,
            integrity: None,
            fetch_priority: None,
        }
    }

    /// Create a DNS prefetch hint
    pub fn dns_prefetch(origin: &str) -> Self {
        Self {
            hint_type: HintType::DnsPrefetch,
            href: origin.to_string(),
            resource_as: None,
            mime_type: None,
            crossorigin: None,
            media: None,
            integrity: None,
            fetch_priority: None,
        }
    }

    /// Create a module preload hint
    pub fn module_preload(href: &str) -> Self {
        Self {
            hint_type: HintType::ModulePreload,
            href: href.to_string(),
            resource_as: None,
            mime_type: None,
            crossorigin: Some(CrossOrigin::Anonymous),
            media: None,
            integrity: None,
            fetch_priority: None,
        }
    }

    /// Add crossorigin attribute
    pub fn with_crossorigin(mut self, crossorigin: CrossOrigin) -> Self {
        self.crossorigin = Some(crossorigin);
        self
    }

    /// Add media query
    pub fn with_media(mut self, media: &str) -> Self {
        self.media = Some(media.to_string());
        self
    }

    /// Add integrity hash
    pub fn with_integrity(mut self, integrity: &str) -> Self {
        self.integrity = Some(integrity.to_string());
        self
    }

    /// Add fetch priority
    pub fn with_fetch_priority(mut self, priority: FetchPriority) -> Self {
        self.fetch_priority = Some(priority);
        self
    }

    /// Generate HTML link element
    pub fn to_html(&self) -> String {
        let mut attrs = vec![
            format!("rel=\"{}\"", self.hint_type.as_rel()),
            format!("href=\"{}\"", self.href),
        ];

        if let Some(ref resource_as) = self.resource_as {
            attrs.push(format!("as=\"{}\"", resource_as.as_str()));
        }

        if let Some(ref mime_type) = self.mime_type {
            attrs.push(format!("type=\"{}\"", mime_type));
        }

        if let Some(ref crossorigin) = self.crossorigin {
            attrs.push(format!("crossorigin=\"{}\"", crossorigin.as_str()));
        }

        if let Some(ref media) = self.media {
            attrs.push(format!("media=\"{}\"", media));
        }

        if let Some(ref integrity) = self.integrity {
            attrs.push(format!("integrity=\"{}\"", integrity));
        }

        if let Some(ref priority) = self.fetch_priority {
            attrs.push(format!("fetchpriority=\"{}\"", priority.as_str()));
        }

        format!("<link {}>", attrs.join(" "))
    }

    /// Generate Link header value
    pub fn to_link_header(&self) -> String {
        let mut parts = vec![
            format!("<{}>", self.href),
            format!("rel={}", self.hint_type.as_rel()),
        ];

        if let Some(ref resource_as) = self.resource_as {
            parts.push(format!("as={}", resource_as.as_str()));
        }

        if let Some(ref crossorigin) = self.crossorigin {
            parts.push(format!("crossorigin={}", crossorigin.as_str()));
        }

        parts.join("; ")
    }
}

/// Resource hints manager
pub struct ResourceHintsManager {
    /// Critical resources to always preload
    critical_resources: Vec<ResourceHint>,
    /// Origins to preconnect
    preconnect_origins: HashSet<String>,
    /// Page-specific hints
    page_hints: HashMap<String, Vec<ResourceHint>>,
}

impl ResourceHintsManager {
    pub fn new() -> Self {
        Self {
            critical_resources: Vec::new(),
            preconnect_origins: HashSet::new(),
            page_hints: HashMap::new(),
        }
    }

    /// Add a critical resource that should always be preloaded
    pub fn add_critical_resource(&mut self, hint: ResourceHint) {
        self.critical_resources.push(hint);
    }

    /// Add an origin to preconnect to
    pub fn add_preconnect_origin(&mut self, origin: &str) {
        self.preconnect_origins.insert(origin.to_string());
    }

    /// Add hints for a specific page
    pub fn add_page_hints(&mut self, path: &str, hints: Vec<ResourceHint>) {
        self.page_hints
            .entry(path.to_string())
            .or_insert_with(Vec::new)
            .extend(hints);
    }

    /// Get all hints for a page
    pub fn get_hints_for_page(&self, path: &str) -> Vec<ResourceHint> {
        let mut hints = Vec::new();

        // Add preconnect hints
        for origin in &self.preconnect_origins {
            hints.push(ResourceHint::preconnect(origin));
            hints.push(ResourceHint::dns_prefetch(origin));
        }

        // Add critical resources
        hints.extend(self.critical_resources.iter().cloned());

        // Add page-specific hints
        if let Some(page_hints) = self.page_hints.get(path) {
            hints.extend(page_hints.iter().cloned());
        }

        hints
    }

    /// Generate all hints as HTML
    pub fn generate_html(&self, path: &str) -> String {
        self.get_hints_for_page(path)
            .iter()
            .map(|h| h.to_html())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate Link headers for early hints
    pub fn generate_link_headers(&self, path: &str) -> Vec<String> {
        self.get_hints_for_page(path)
            .iter()
            .map(|h| h.to_link_header())
            .collect()
    }
}

impl Default for ResourceHintsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatic resource hint analyzer
pub struct ResourceHintAnalyzer {
    /// Detected critical resources
    critical_css: Vec<String>,
    critical_js: Vec<String>,
    critical_fonts: Vec<String>,
    /// External origins used
    external_origins: HashSet<String>,
}

impl ResourceHintAnalyzer {
    pub fn new() -> Self {
        Self {
            critical_css: Vec::new(),
            critical_js: Vec::new(),
            critical_fonts: Vec::new(),
            external_origins: HashSet::new(),
        }
    }

    /// Analyze HTML to detect resources for preloading
    pub fn analyze_html(&mut self, html: &str) {
        // Extract stylesheet links
        let css_re = regex::Regex::new(r#"<link[^>]+href=["']([^"']+\.css[^"']*)["'][^>]*>"#).unwrap();
        for cap in css_re.captures_iter(html) {
            if let Some(href) = cap.get(1) {
                self.critical_css.push(href.as_str().to_string());
                self.extract_origin(href.as_str());
            }
        }

        // Extract script sources
        let js_re = regex::Regex::new(r#"<script[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        for cap in js_re.captures_iter(html) {
            if let Some(src) = cap.get(1) {
                self.critical_js.push(src.as_str().to_string());
                self.extract_origin(src.as_str());
            }
        }

        // Extract font preloads
        let font_re = regex::Regex::new(r#"url\(["']?([^"'()]+\.(woff2?|ttf|otf|eot))["']?\)"#).unwrap();
        for cap in font_re.captures_iter(html) {
            if let Some(url) = cap.get(1) {
                self.critical_fonts.push(url.as_str().to_string());
                self.extract_origin(url.as_str());
            }
        }
    }

    fn extract_origin(&mut self, url: &str) {
        if url.starts_with("http://") || url.starts_with("https://") {
            if let Some(pos) = url.find("://") {
                if let Some(end) = url[pos + 3..].find('/') {
                    let origin = &url[..pos + 3 + end];
                    self.external_origins.insert(origin.to_string());
                }
            }
        }
    }

    /// Generate hints based on analysis
    pub fn generate_hints(&self) -> Vec<ResourceHint> {
        let mut hints = Vec::new();

        // Preconnect to external origins
        for origin in &self.external_origins {
            hints.push(ResourceHint::preconnect(origin));
        }

        // Preload critical CSS
        for css in &self.critical_css {
            hints.push(
                ResourceHint::preload(css, ResourceAs::Style)
                    .with_fetch_priority(FetchPriority::High)
            );
        }

        // Preload critical fonts
        for font in &self.critical_fonts {
            hints.push(
                ResourceHint::preload(font, ResourceAs::Font)
                    .with_crossorigin(CrossOrigin::Anonymous)
            );
        }

        // Module preload for ES modules
        for js in &self.critical_js {
            if js.contains("type=\"module\"") || js.ends_with(".mjs") {
                hints.push(ResourceHint::module_preload(js));
            } else {
                hints.push(ResourceHint::preload(js, ResourceAs::Script));
            }
        }

        hints
    }
}

impl Default for ResourceHintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Early Hints (103 status code) generator
pub struct EarlyHintsGenerator {
    hints_manager: ResourceHintsManager,
}

impl EarlyHintsGenerator {
    pub fn new(hints_manager: ResourceHintsManager) -> Self {
        Self { hints_manager }
    }

    /// Generate Early Hints response headers
    pub fn generate_early_hints(&self, path: &str) -> Vec<(String, String)> {
        let link_values = self.hints_manager.generate_link_headers(path);

        if link_values.is_empty() {
            return Vec::new();
        }

        vec![("Link".to_string(), link_values.join(", "))]
    }
}

/// Navigation preload predictor for likely next pages
pub struct NavigationPredictor {
    /// Navigation graph: current_path -> (next_path, probability)
    nav_graph: HashMap<String, Vec<(String, f32)>>,
    /// Minimum probability threshold for prefetching
    min_probability: f32,
}

impl NavigationPredictor {
    pub fn new(min_probability: f32) -> Self {
        Self {
            nav_graph: HashMap::new(),
            min_probability,
        }
    }

    /// Record a navigation event
    pub fn record_navigation(&mut self, from: &str, to: &str) {
        let edges = self.nav_graph.entry(from.to_string()).or_insert_with(Vec::new);

        if let Some(edge) = edges.iter_mut().find(|(path, _)| path == to) {
            // Increase probability for existing edge
            edge.1 = (edge.1 + 0.1).min(1.0);
        } else {
            // Add new edge
            edges.push((to.to_string(), 0.1));
        }

        // Normalize probabilities
        let total: f32 = edges.iter().map(|(_, p)| *p).sum();
        for (_, prob) in edges.iter_mut() {
            *prob /= total;
        }
    }

    /// Get prefetch hints for likely next navigations
    pub fn get_prefetch_hints(&self, current_path: &str) -> Vec<ResourceHint> {
        let mut hints = Vec::new();

        if let Some(edges) = self.nav_graph.get(current_path) {
            for (path, prob) in edges {
                if *prob >= self.min_probability {
                    hints.push(ResourceHint::prefetch(path));
                }
            }
        }

        hints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_hint_generation() {
        let hint = ResourceHint::preload("/styles/main.css", ResourceAs::Style);
        let html = hint.to_html();

        assert!(html.contains("rel=\"preload\""));
        assert!(html.contains("as=\"style\""));
        assert!(html.contains("href=\"/styles/main.css\""));
    }

    #[test]
    fn test_preconnect_hint() {
        let hint = ResourceHint::preconnect("https://cdn.example.com");
        let html = hint.to_html();

        assert!(html.contains("rel=\"preconnect\""));
        assert!(html.contains("href=\"https://cdn.example.com\""));
    }

    #[test]
    fn test_link_header_generation() {
        let hint = ResourceHint::preload("/app.js", ResourceAs::Script);
        let header = hint.to_link_header();

        assert!(header.contains("</app.js>"));
        assert!(header.contains("rel=preload"));
        assert!(header.contains("as=script"));
    }

    #[test]
    fn test_resource_hints_manager() {
        let mut manager = ResourceHintsManager::new();
        manager.add_preconnect_origin("https://cdn.example.com");
        manager.add_critical_resource(ResourceHint::preload("/main.css", ResourceAs::Style));

        let hints = manager.get_hints_for_page("/");
        assert!(hints.len() >= 2);
    }

    #[test]
    fn test_html_analysis() {
        let mut analyzer = ResourceHintAnalyzer::new();
        let html = r#"
            <link href="https://cdn.example.com/style.css" rel="stylesheet">
            <script src="/app.js"></script>
        "#;

        analyzer.analyze_html(html);
        let hints = analyzer.generate_hints();

        assert!(!hints.is_empty());
    }
}
