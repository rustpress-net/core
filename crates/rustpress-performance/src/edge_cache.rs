//! Edge Caching Configuration
//!
//! Configuration for edge/CDN caching with proper cache directives.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Edge caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCacheConfig {
    /// Default cache TTL for static assets (seconds)
    pub static_ttl: u64,
    /// Default cache TTL for dynamic pages (seconds)
    pub dynamic_ttl: u64,
    /// Default cache TTL for API responses (seconds)
    pub api_ttl: u64,
    /// Stale-while-revalidate window (seconds)
    pub stale_while_revalidate: u64,
    /// Stale-if-error window (seconds)
    pub stale_if_error: u64,
    /// Enable surrogate keys (Fastly/Varnish)
    pub surrogate_keys: bool,
    /// Enable vary headers
    pub vary_headers: Vec<String>,
    /// Cache bypass cookie names
    pub bypass_cookies: Vec<String>,
    /// Routes to never cache
    pub no_cache_paths: Vec<String>,
    /// Custom cache rules
    pub rules: Vec<EdgeCacheRule>,
}

impl Default for EdgeCacheConfig {
    fn default() -> Self {
        Self {
            static_ttl: 31536000,  // 1 year
            dynamic_ttl: 3600,     // 1 hour
            api_ttl: 60,           // 1 minute
            stale_while_revalidate: 86400, // 1 day
            stale_if_error: 604800,        // 1 week
            surrogate_keys: true,
            vary_headers: vec![
                "Accept-Encoding".to_string(),
                "Accept".to_string(),
            ],
            bypass_cookies: vec![
                "wordpress_logged_in".to_string(),
                "session".to_string(),
            ],
            no_cache_paths: vec![
                "/wp-admin".to_string(),
                "/admin".to_string(),
                "/api/auth".to_string(),
                "/wp-login.php".to_string(),
            ],
            rules: Vec::new(),
        }
    }
}

/// Custom cache rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCacheRule {
    /// Rule name
    pub name: String,
    /// Path pattern (glob or regex)
    pub path_pattern: String,
    /// HTTP methods this rule applies to
    pub methods: Vec<String>,
    /// Cache behavior
    pub behavior: CacheBehavior,
    /// Custom TTL (overrides default)
    pub ttl: Option<u64>,
    /// Custom surrogate keys
    pub surrogate_keys: Vec<String>,
    /// Custom vary headers
    pub vary: Vec<String>,
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

/// Cache behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheBehavior {
    /// Cache at edge
    Cache,
    /// Bypass cache
    Bypass,
    /// Cache but revalidate on every request
    Revalidate,
    /// Cache private (only browser, not edge)
    Private,
}

/// Edge cache headers generator
pub struct EdgeCacheHeaders {
    config: EdgeCacheConfig,
}

impl EdgeCacheHeaders {
    pub fn new(config: EdgeCacheConfig) -> Self {
        Self { config }
    }

    /// Generate cache headers for a request
    pub fn generate_headers(
        &self,
        path: &str,
        content_type: &str,
        is_authenticated: bool,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // Check for no-cache paths
        if self.should_bypass_cache(path, is_authenticated) {
            headers.insert(
                "Cache-Control".to_string(),
                "no-store, no-cache, must-revalidate".to_string(),
            );
            headers.insert("Pragma".to_string(), "no-cache".to_string());
            return headers;
        }

        // Find matching rule
        if let Some(rule) = self.find_matching_rule(path) {
            return self.apply_rule(&rule, content_type);
        }

        // Apply default caching based on content type
        self.apply_default_caching(path, content_type)
    }

    fn should_bypass_cache(&self, path: &str, is_authenticated: bool) -> bool {
        // Always bypass for authenticated users
        if is_authenticated {
            return true;
        }

        // Check no-cache paths
        self.config.no_cache_paths.iter().any(|p| path.starts_with(p))
    }

    fn find_matching_rule(&self, path: &str) -> Option<EdgeCacheRule> {
        let mut rules: Vec<_> = self.config.rules.iter().collect();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in rules {
            if glob_matches(path, &rule.path_pattern) {
                return Some(rule.clone());
            }
        }

        None
    }

    fn apply_rule(&self, rule: &EdgeCacheRule, content_type: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        let cache_control = match rule.behavior {
            CacheBehavior::Cache => {
                let ttl = rule.ttl.unwrap_or(self.config.dynamic_ttl);
                format!(
                    "public, max-age={}, s-maxage={}, stale-while-revalidate={}, stale-if-error={}",
                    ttl, ttl, self.config.stale_while_revalidate, self.config.stale_if_error
                )
            }
            CacheBehavior::Bypass => {
                "no-store, no-cache, must-revalidate".to_string()
            }
            CacheBehavior::Revalidate => {
                "no-cache, must-revalidate".to_string()
            }
            CacheBehavior::Private => {
                let ttl = rule.ttl.unwrap_or(3600);
                format!("private, max-age={}", ttl)
            }
        };

        headers.insert("Cache-Control".to_string(), cache_control);

        // Add vary headers
        let vary = if rule.vary.is_empty() {
            self.config.vary_headers.join(", ")
        } else {
            rule.vary.join(", ")
        };
        if !vary.is_empty() {
            headers.insert("Vary".to_string(), vary);
        }

        // Add surrogate keys
        if self.config.surrogate_keys && !rule.surrogate_keys.is_empty() {
            headers.insert(
                "Surrogate-Key".to_string(),
                rule.surrogate_keys.join(" "),
            );
        }

        headers
    }

    fn apply_default_caching(&self, path: &str, content_type: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        let (ttl, is_immutable) = if is_static_asset(content_type) || has_hash_in_path(path) {
            (self.config.static_ttl, true)
        } else if content_type.contains("json") || path.starts_with("/api") {
            (self.config.api_ttl, false)
        } else {
            (self.config.dynamic_ttl, false)
        };

        let mut cache_control = format!(
            "public, max-age={}, s-maxage={}",
            ttl, ttl
        );

        if is_immutable {
            cache_control.push_str(", immutable");
        } else {
            cache_control.push_str(&format!(
                ", stale-while-revalidate={}, stale-if-error={}",
                self.config.stale_while_revalidate, self.config.stale_if_error
            ));
        }

        headers.insert("Cache-Control".to_string(), cache_control);

        // Add vary headers
        let vary = self.config.vary_headers.join(", ");
        if !vary.is_empty() {
            headers.insert("Vary".to_string(), vary);
        }

        headers
    }

    /// Generate Cloudflare-specific headers
    pub fn cloudflare_headers(&self, ttl: u64, cache_tag: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "Cache-Control".to_string(),
            format!("public, max-age={}", ttl),
        );

        headers.insert("CDN-Cache-Control".to_string(), format!("max-age={}", ttl));

        if !cache_tag.is_empty() {
            headers.insert("Cache-Tag".to_string(), cache_tag.to_string());
        }

        headers
    }

    /// Generate Fastly-specific headers
    pub fn fastly_headers(&self, ttl: u64, surrogate_keys: &[String]) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "Surrogate-Control".to_string(),
            format!("max-age={}, stale-while-revalidate={}, stale-if-error={}",
                ttl, self.config.stale_while_revalidate, self.config.stale_if_error
            ),
        );

        if !surrogate_keys.is_empty() {
            headers.insert(
                "Surrogate-Key".to_string(),
                surrogate_keys.join(" "),
            );
        }

        headers
    }

    /// Generate Varnish-specific headers
    pub fn varnish_headers(&self, ttl: u64, tags: &[String]) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert(
            "Cache-Control".to_string(),
            format!("public, max-age={}", ttl),
        );

        if !tags.is_empty() {
            headers.insert("X-Cache-Tags".to_string(), tags.join(","));
        }

        headers.insert("X-Varnish-TTL".to_string(), ttl.to_string());

        headers
    }
}

/// Check if path matches a glob pattern
fn glob_matches(path: &str, pattern: &str) -> bool {
    // Use placeholder to avoid replacing the * in .* when replacing single *
    let regex_pattern = pattern
        .replace(".", r"\.")
        .replace("**", "\x00DOUBLESTAR\x00")
        .replace("*", "[^/]*")
        .replace("\x00DOUBLESTAR\x00", ".*");

    let regex = format!("^{}$", regex_pattern);
    regex::Regex::new(&regex)
        .map(|re| re.is_match(path))
        .unwrap_or(false)
}

/// Check if content type indicates a static asset
fn is_static_asset(content_type: &str) -> bool {
    content_type.starts_with("image/")
        || content_type.starts_with("font/")
        || content_type.starts_with("audio/")
        || content_type.starts_with("video/")
        || content_type.contains("javascript")
        || content_type.contains("css")
        || content_type.contains("woff")
}

/// Check if path contains a content hash (for cache busting)
fn has_hash_in_path(path: &str) -> bool {
    // Match patterns like file.abc123.js or file-abc123.css
    let hash_pattern = regex::Regex::new(r"[.-][a-f0-9]{8,}[.-]").unwrap();
    hash_pattern.is_match(path)
}

/// Edge cache key generator
pub struct EdgeCacheKeyGenerator {
    /// Query parameters to include in cache key
    include_params: Vec<String>,
    /// Query parameters to exclude from cache key
    exclude_params: Vec<String>,
    /// Headers to include in cache key
    include_headers: Vec<String>,
}

impl EdgeCacheKeyGenerator {
    pub fn new() -> Self {
        Self {
            include_params: Vec::new(),
            exclude_params: vec![
                "utm_source".to_string(),
                "utm_medium".to_string(),
                "utm_campaign".to_string(),
                "utm_term".to_string(),
                "utm_content".to_string(),
                "fbclid".to_string(),
                "gclid".to_string(),
                "_ga".to_string(),
            ],
            include_headers: vec!["Accept".to_string()],
        }
    }

    /// Set parameters to include in cache key
    pub fn with_include_params(mut self, params: Vec<String>) -> Self {
        self.include_params = params;
        self
    }

    /// Set headers to include in cache key
    pub fn with_include_headers(mut self, headers: Vec<String>) -> Self {
        self.include_headers = headers;
        self
    }

    /// Generate cache key for a request
    pub fn generate(&self, method: &str, uri: &str, headers: &HashMap<String, String>) -> String {
        let mut hasher = blake3::Hasher::new();

        // Add method
        hasher.update(method.as_bytes());
        hasher.update(b"|");

        // Parse and normalize URI
        let normalized_uri = self.normalize_uri(uri);
        hasher.update(normalized_uri.as_bytes());

        // Add relevant headers
        for header_name in &self.include_headers {
            if let Some(value) = headers.get(header_name) {
                hasher.update(b"|");
                hasher.update(header_name.as_bytes());
                hasher.update(b"=");
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().to_hex().to_string()
    }

    fn normalize_uri(&self, uri: &str) -> String {
        let (path, query) = match uri.find('?') {
            Some(pos) => (&uri[..pos], Some(&uri[pos + 1..])),
            None => (uri, None),
        };

        let query = match query {
            Some(q) => {
                let mut params: Vec<(&str, &str)> = q
                    .split('&')
                    .filter_map(|p| {
                        let mut parts = p.splitn(2, '=');
                        let key = parts.next()?;
                        let value = parts.next().unwrap_or("");
                        Some((key, value))
                    })
                    .filter(|(key, _)| {
                        if !self.include_params.is_empty() {
                            self.include_params.contains(&key.to_string())
                        } else {
                            !self.exclude_params.contains(&key.to_string())
                        }
                    })
                    .collect();

                params.sort_by(|a, b| a.0.cmp(b.0));

                if params.is_empty() {
                    None
                } else {
                    Some(
                        params
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join("&"),
                    )
                }
            }
            None => None,
        };

        match query {
            Some(q) => format!("{}?{}", path, q),
            None => path.to_string(),
        }
    }
}

impl Default for EdgeCacheKeyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Surrogate key generator for cache invalidation
pub struct SurrogateKeyGenerator;

impl SurrogateKeyGenerator {
    /// Generate surrogate key for a post
    pub fn post(post_id: i64) -> String {
        format!("post-{}", post_id)
    }

    /// Generate surrogate key for a post type
    pub fn post_type(post_type: &str) -> String {
        format!("type-{}", post_type)
    }

    /// Generate surrogate key for a term
    pub fn term(taxonomy: &str, term_id: i64) -> String {
        format!("term-{}-{}", taxonomy, term_id)
    }

    /// Generate surrogate key for a user
    pub fn user(user_id: i64) -> String {
        format!("user-{}", user_id)
    }

    /// Generate surrogate key for a page
    pub fn page(path: &str) -> String {
        let normalized = path.replace('/', "_").trim_matches('_').to_string();
        format!("page-{}", normalized)
    }

    /// Generate global surrogate key
    pub fn global() -> String {
        "global".to_string()
    }

    /// Generate surrogate key for homepage
    pub fn homepage() -> String {
        "homepage".to_string()
    }

    /// Generate surrogate key for feed
    pub fn feed() -> String {
        "feed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(glob_matches("/posts/123", "/posts/*"));
        assert!(glob_matches("/api/users/1/posts", "/api/**"));
        assert!(!glob_matches("/admin/posts", "/posts/*"));
    }

    #[test]
    fn test_hash_detection() {
        assert!(has_hash_in_path("/assets/main.abc12345.js"));
        assert!(has_hash_in_path("/css/style-def67890.css"));
        assert!(!has_hash_in_path("/posts/123"));
    }

    #[test]
    fn test_cache_key_generation() {
        let generator = EdgeCacheKeyGenerator::new();

        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "text/html".to_string());

        let key1 = generator.generate("GET", "/page?a=1&utm_source=test", &headers);
        let key2 = generator.generate("GET", "/page?a=1", &headers);

        // UTM params should be excluded
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_surrogate_keys() {
        assert_eq!(SurrogateKeyGenerator::post(123), "post-123");
        assert_eq!(SurrogateKeyGenerator::post_type("page"), "type-page");
        assert_eq!(SurrogateKeyGenerator::term("category", 5), "term-category-5");
    }

    #[test]
    fn test_edge_headers() {
        let config = EdgeCacheConfig::default();
        let generator = EdgeCacheHeaders::new(config);

        let headers = generator.generate_headers("/assets/main.js", "application/javascript", false);
        assert!(headers.get("Cache-Control").unwrap().contains("max-age="));
    }
}
