//! HTTP Caching Headers Management
//!
//! Comprehensive HTTP cache control with ETags, Cache-Control, and conditional requests.

use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};
use axum::http::{header, HeaderMap, HeaderValue};

/// Cache control directives
#[derive(Debug, Clone, Default)]
pub struct CacheControl {
    /// Response can be stored by any cache
    pub public: bool,
    /// Response is for a single user
    pub private: bool,
    /// Response must not be cached
    pub no_cache: bool,
    /// Response must not be stored
    pub no_store: bool,
    /// Response must not be transformed
    pub no_transform: bool,
    /// Must revalidate after becoming stale
    pub must_revalidate: bool,
    /// Like must-revalidate but for shared caches
    pub proxy_revalidate: bool,
    /// Max age in seconds
    pub max_age: Option<u64>,
    /// Max age for shared caches
    pub s_maxage: Option<u64>,
    /// Stale while revalidate
    pub stale_while_revalidate: Option<u64>,
    /// Stale if error
    pub stale_if_error: Option<u64>,
    /// Immutable (content will never change)
    pub immutable: bool,
}

impl CacheControl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Public cacheable response
    pub fn public() -> Self {
        Self {
            public: true,
            ..Default::default()
        }
    }

    /// Private (user-specific) cacheable response
    pub fn private() -> Self {
        Self {
            private: true,
            ..Default::default()
        }
    }

    /// No caching
    pub fn no_cache() -> Self {
        Self {
            no_cache: true,
            no_store: true,
            must_revalidate: true,
            ..Default::default()
        }
    }

    /// Immutable static asset
    pub fn immutable_asset() -> Self {
        Self {
            public: true,
            max_age: Some(31536000), // 1 year
            immutable: true,
            ..Default::default()
        }
    }

    /// Short-lived API response
    pub fn api(max_age: u64) -> Self {
        Self {
            private: true,
            max_age: Some(max_age),
            must_revalidate: true,
            ..Default::default()
        }
    }

    /// Set max age
    pub fn with_max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set s-maxage for CDNs
    pub fn with_s_maxage(mut self, seconds: u64) -> Self {
        self.s_maxage = Some(seconds);
        self
    }

    /// Enable stale-while-revalidate
    pub fn with_stale_while_revalidate(mut self, seconds: u64) -> Self {
        self.stale_while_revalidate = Some(seconds);
        self
    }

    /// Enable stale-if-error
    pub fn with_stale_if_error(mut self, seconds: u64) -> Self {
        self.stale_if_error = Some(seconds);
        self
    }

    /// Convert to header value
    pub fn to_header_value(&self) -> String {
        let mut parts = Vec::new();

        if self.public {
            parts.push("public".to_string());
        }
        if self.private {
            parts.push("private".to_string());
        }
        if self.no_cache {
            parts.push("no-cache".to_string());
        }
        if self.no_store {
            parts.push("no-store".to_string());
        }
        if self.no_transform {
            parts.push("no-transform".to_string());
        }
        if self.must_revalidate {
            parts.push("must-revalidate".to_string());
        }
        if self.proxy_revalidate {
            parts.push("proxy-revalidate".to_string());
        }
        if let Some(max_age) = self.max_age {
            parts.push(format!("max-age={}", max_age));
        }
        if let Some(s_maxage) = self.s_maxage {
            parts.push(format!("s-maxage={}", s_maxage));
        }
        if let Some(swr) = self.stale_while_revalidate {
            parts.push(format!("stale-while-revalidate={}", swr));
        }
        if let Some(sie) = self.stale_if_error {
            parts.push(format!("stale-if-error={}", sie));
        }
        if self.immutable {
            parts.push("immutable".to_string());
        }

        parts.join(", ")
    }
}

/// ETag type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ETag {
    /// Strong validator
    Strong(String),
    /// Weak validator
    Weak(String),
}

impl ETag {
    /// Create strong ETag from content
    pub fn from_content(content: &[u8]) -> Self {
        let hash = blake3::hash(content);
        Self::Strong(format!("\"{}\"", &hash.to_hex()[..16]))
    }

    /// Create weak ETag
    pub fn weak(value: &str) -> Self {
        Self::Weak(format!("W/\"{}\"", value))
    }

    /// Parse ETag from header value
    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();

        if trimmed.starts_with("W/") {
            let inner = trimmed[2..].trim_matches('"');
            Some(Self::Weak(inner.to_string()))
        } else {
            let inner = trimmed.trim_matches('"');
            Some(Self::Strong(inner.to_string()))
        }
    }

    /// Check if ETags match (considering weak comparison)
    pub fn matches(&self, other: &ETag, weak_comparison: bool) -> bool {
        if weak_comparison {
            // Weak comparison: ignore W/ prefix
            self.inner_value() == other.inner_value()
        } else {
            // Strong comparison: must both be strong and equal
            match (self, other) {
                (ETag::Strong(a), ETag::Strong(b)) => a == b,
                _ => false,
            }
        }
    }

    fn inner_value(&self) -> &str {
        match self {
            ETag::Strong(v) | ETag::Weak(v) => v,
        }
    }

    /// Convert to header value
    pub fn to_header_value(&self) -> String {
        match self {
            ETag::Strong(v) => format!("\"{}\"", v),
            ETag::Weak(v) => format!("W/\"{}\"", v),
        }
    }
}

/// HTTP cache headers builder
pub struct HttpCacheHeaders {
    headers: HeaderMap,
}

impl HttpCacheHeaders {
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
        }
    }

    /// Set Cache-Control header
    pub fn cache_control(mut self, control: CacheControl) -> Self {
        if let Ok(value) = HeaderValue::from_str(&control.to_header_value()) {
            self.headers.insert(header::CACHE_CONTROL, value);
        }
        self
    }

    /// Set ETag header
    pub fn etag(mut self, etag: ETag) -> Self {
        if let Ok(value) = HeaderValue::from_str(&etag.to_header_value()) {
            self.headers.insert(header::ETAG, value);
        }
        self
    }

    /// Set Last-Modified header
    pub fn last_modified(mut self, timestamp: i64) -> Self {
        let datetime = httpdate::HttpDate::from(
            UNIX_EPOCH + Duration::from_secs(timestamp as u64)
        );
        if let Ok(value) = HeaderValue::from_str(&datetime.to_string()) {
            self.headers.insert(header::LAST_MODIFIED, value);
        }
        self
    }

    /// Set Expires header
    pub fn expires(mut self, timestamp: i64) -> Self {
        let datetime = httpdate::HttpDate::from(
            UNIX_EPOCH + Duration::from_secs(timestamp as u64)
        );
        if let Ok(value) = HeaderValue::from_str(&datetime.to_string()) {
            self.headers.insert(header::EXPIRES, value);
        }
        self
    }

    /// Set Vary header
    pub fn vary(mut self, headers: &[&str]) -> Self {
        let value = headers.join(", ");
        if let Ok(value) = HeaderValue::from_str(&value) {
            self.headers.insert(header::VARY, value);
        }
        self
    }

    /// Set Age header
    pub fn age(mut self, seconds: u64) -> Self {
        if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
            self.headers.insert(header::AGE, value);
        }
        self
    }

    /// Set Surrogate-Control for CDNs
    pub fn surrogate_control(mut self, value: &str) -> Self {
        if let Ok(header_value) = HeaderValue::from_str(value) {
            self.headers.insert(
                header::HeaderName::from_static("surrogate-control"),
                header_value,
            );
        }
        self
    }

    /// Set CDN-Cache-Control
    pub fn cdn_cache_control(mut self, control: CacheControl) -> Self {
        if let Ok(value) = HeaderValue::from_str(&control.to_header_value()) {
            self.headers.insert(
                header::HeaderName::from_static("cdn-cache-control"),
                value,
            );
        }
        self
    }

    /// Build headers
    pub fn build(self) -> HeaderMap {
        self.headers
    }
}

impl Default for HttpCacheHeaders {
    fn default() -> Self {
        Self::new()
    }
}

/// Conditional request handler
pub struct ConditionalRequest {
    /// If-None-Match ETags
    if_none_match: Vec<ETag>,
    /// If-Modified-Since timestamp
    if_modified_since: Option<i64>,
    /// If-Match ETags
    if_match: Vec<ETag>,
    /// If-Unmodified-Since timestamp
    if_unmodified_since: Option<i64>,
}

impl ConditionalRequest {
    /// Parse conditional headers from request
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let if_none_match = headers
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .map(|v| {
                v.split(',')
                    .filter_map(|s| ETag::parse(s.trim()))
                    .collect()
            })
            .unwrap_or_default();

        let if_modified_since = headers
            .get(header::IF_MODIFIED_SINCE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| httpdate::parse_http_date(s).ok())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

        let if_match = headers
            .get(header::IF_MATCH)
            .and_then(|v| v.to_str().ok())
            .map(|v| {
                v.split(',')
                    .filter_map(|s| ETag::parse(s.trim()))
                    .collect()
            })
            .unwrap_or_default();

        let if_unmodified_since = headers
            .get(header::IF_UNMODIFIED_SINCE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| httpdate::parse_http_date(s).ok())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

        Self {
            if_none_match,
            if_modified_since,
            if_match,
            if_unmodified_since,
        }
    }

    /// Check if response should be 304 Not Modified
    pub fn is_not_modified(&self, etag: Option<&ETag>, last_modified: Option<i64>) -> bool {
        // Check If-None-Match
        if !self.if_none_match.is_empty() {
            if let Some(current_etag) = etag {
                return self.if_none_match.iter().any(|e| e.matches(current_etag, true));
            }
        }

        // Check If-Modified-Since
        if let (Some(since), Some(modified)) = (self.if_modified_since, last_modified) {
            return modified <= since;
        }

        false
    }

    /// Check if precondition failed (for conditional updates)
    pub fn precondition_failed(&self, etag: Option<&ETag>, last_modified: Option<i64>) -> bool {
        // Check If-Match
        if !self.if_match.is_empty() {
            if let Some(current_etag) = etag {
                if !self.if_match.iter().any(|e| e.matches(current_etag, false)) {
                    return true;
                }
            } else {
                return true;
            }
        }

        // Check If-Unmodified-Since
        if let (Some(since), Some(modified)) = (self.if_unmodified_since, last_modified) {
            if modified > since {
                return true;
            }
        }

        false
    }
}

/// Cache profile for different content types
#[derive(Debug, Clone)]
pub struct CacheProfile {
    pub name: String,
    pub cache_control: CacheControl,
    pub vary: Vec<String>,
    pub enable_etag: bool,
    pub enable_last_modified: bool,
}

impl CacheProfile {
    /// Profile for static assets
    pub fn static_assets() -> Self {
        Self {
            name: "static_assets".to_string(),
            cache_control: CacheControl {
                public: true,
                max_age: Some(31536000), // 1 year
                immutable: true,
                ..Default::default()
            },
            vary: vec![],
            enable_etag: true,
            enable_last_modified: true,
        }
    }

    /// Profile for HTML pages
    pub fn html_pages() -> Self {
        Self {
            name: "html_pages".to_string(),
            cache_control: CacheControl {
                public: true,
                max_age: Some(300), // 5 minutes
                s_maxage: Some(3600), // 1 hour for CDN
                stale_while_revalidate: Some(86400), // 1 day
                ..Default::default()
            },
            vary: vec!["Accept-Encoding".to_string(), "Cookie".to_string()],
            enable_etag: true,
            enable_last_modified: true,
        }
    }

    /// Profile for API responses
    pub fn api_responses() -> Self {
        Self {
            name: "api_responses".to_string(),
            cache_control: CacheControl {
                private: true,
                max_age: Some(60), // 1 minute
                must_revalidate: true,
                ..Default::default()
            },
            vary: vec!["Accept".to_string(), "Authorization".to_string()],
            enable_etag: true,
            enable_last_modified: false,
        }
    }

    /// Profile for authenticated pages
    pub fn authenticated() -> Self {
        Self {
            name: "authenticated".to_string(),
            cache_control: CacheControl {
                private: true,
                no_cache: true,
                must_revalidate: true,
                ..Default::default()
            },
            vary: vec!["Cookie".to_string()],
            enable_etag: true,
            enable_last_modified: true,
        }
    }

    /// Profile for no caching
    pub fn no_cache() -> Self {
        Self {
            name: "no_cache".to_string(),
            cache_control: CacheControl::no_cache(),
            vary: vec![],
            enable_etag: false,
            enable_last_modified: false,
        }
    }
}

/// Cache profile registry
pub struct CacheProfileRegistry {
    profiles: HashMap<String, CacheProfile>,
    path_patterns: Vec<(regex::Regex, String)>,
}

impl CacheProfileRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            profiles: HashMap::new(),
            path_patterns: Vec::new(),
        };

        // Register default profiles
        registry.register(CacheProfile::static_assets());
        registry.register(CacheProfile::html_pages());
        registry.register(CacheProfile::api_responses());
        registry.register(CacheProfile::authenticated());
        registry.register(CacheProfile::no_cache());

        // Default path patterns
        registry.add_path_pattern(r"^/static/.*", "static_assets");
        registry.add_path_pattern(r"^/assets/.*", "static_assets");
        registry.add_path_pattern(r"\.(css|js|woff2?|ttf|eot|ico|png|jpg|jpeg|gif|svg|webp)$", "static_assets");
        registry.add_path_pattern(r"^/api/.*", "api_responses");
        registry.add_path_pattern(r"^/admin/.*", "authenticated");
        registry.add_path_pattern(r"^/wp-admin/.*", "authenticated");

        registry
    }

    /// Register a cache profile
    pub fn register(&mut self, profile: CacheProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Add path pattern mapping
    pub fn add_path_pattern(&mut self, pattern: &str, profile_name: &str) {
        if let Ok(regex) = regex::Regex::new(pattern) {
            self.path_patterns.push((regex, profile_name.to_string()));
        }
    }

    /// Get profile for path
    pub fn get_for_path(&self, path: &str) -> Option<&CacheProfile> {
        for (pattern, profile_name) in &self.path_patterns {
            if pattern.is_match(path) {
                return self.profiles.get(profile_name);
            }
        }

        // Default to html_pages
        self.profiles.get("html_pages")
    }

    /// Get profile by name
    pub fn get(&self, name: &str) -> Option<&CacheProfile> {
        self.profiles.get(name)
    }
}

impl Default for CacheProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_header() {
        let cc = CacheControl {
            public: true,
            max_age: Some(3600),
            s_maxage: Some(86400),
            stale_while_revalidate: Some(3600),
            ..Default::default()
        };

        let header = cc.to_header_value();
        assert!(header.contains("public"));
        assert!(header.contains("max-age=3600"));
        assert!(header.contains("s-maxage=86400"));
        assert!(header.contains("stale-while-revalidate=3600"));
    }

    #[test]
    fn test_etag_parsing() {
        let strong = ETag::parse("\"abc123\"").unwrap();
        assert!(matches!(strong, ETag::Strong(_)));

        let weak = ETag::parse("W/\"abc123\"").unwrap();
        assert!(matches!(weak, ETag::Weak(_)));
    }

    #[test]
    fn test_etag_matching() {
        let e1 = ETag::Strong("abc".to_string());
        let e2 = ETag::Strong("abc".to_string());
        let e3 = ETag::Weak("abc".to_string());

        // Strong comparison
        assert!(e1.matches(&e2, false));
        assert!(!e1.matches(&e3, false));

        // Weak comparison
        assert!(e1.matches(&e3, true));
    }

    #[test]
    fn test_conditional_not_modified() {
        let etag = ETag::Strong("test123".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("\"test123\""),
        );

        let conditional = ConditionalRequest::from_headers(&headers);
        assert!(conditional.is_not_modified(Some(&etag), None));
    }
}
