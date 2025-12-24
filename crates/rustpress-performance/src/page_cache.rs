//! Page Caching with Cache Tags
//!
//! Full page caching system with tag-based invalidation for efficient cache management.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Page cache errors
#[derive(Debug, Error)]
pub enum PageCacheError {
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache storage error: {0}")]
    Storage(String),
}

/// Cached page entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPage {
    /// The cached HTML content
    pub content: String,
    /// HTTP status code
    pub status_code: u16,
    /// Response headers to restore
    pub headers: HashMap<String, String>,
    /// Content type
    pub content_type: String,
    /// Cache tags for invalidation
    pub tags: HashSet<String>,
    /// When this entry was created
    pub created_at: i64,
    /// Time-to-live in seconds
    pub ttl: u64,
    /// ETag for conditional requests
    pub etag: String,
    /// Last-Modified timestamp
    pub last_modified: i64,
}

/// Cache entry with metadata
struct CacheEntry {
    page: CachedPage,
    created: Instant,
    hits: u64,
}

/// Page cache configuration
#[derive(Debug, Clone)]
pub struct PageCacheConfig {
    /// Maximum number of pages to cache
    pub max_entries: usize,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Enable compression for cached content
    pub compress: bool,
    /// Minimum content size for compression (bytes)
    pub compress_threshold: usize,
    /// Cache query strings
    pub cache_query_strings: bool,
    /// Query parameters to ignore when caching
    pub ignore_query_params: Vec<String>,
    /// Paths to never cache
    pub excluded_paths: Vec<String>,
    /// Cache authenticated requests
    pub cache_authenticated: bool,
}

impl Default for PageCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl: 3600, // 1 hour
            compress: true,
            compress_threshold: 1024, // 1KB
            cache_query_strings: true,
            ignore_query_params: vec![
                "utm_source".to_string(),
                "utm_medium".to_string(),
                "utm_campaign".to_string(),
                "fbclid".to_string(),
                "gclid".to_string(),
            ],
            excluded_paths: vec![
                "/wp-admin".to_string(),
                "/admin".to_string(),
                "/api".to_string(),
            ],
            cache_authenticated: false,
        }
    }
}

/// Cache key generator
pub struct CacheKeyGenerator {
    config: PageCacheConfig,
}

impl CacheKeyGenerator {
    pub fn new(config: PageCacheConfig) -> Self {
        Self { config }
    }

    /// Generate cache key from request
    pub fn generate(&self, method: &str, uri: &str, vary_headers: &HashMap<String, String>) -> String {
        let mut hasher = blake3::Hasher::new();

        // Add method
        hasher.update(method.as_bytes());
        hasher.update(b":");

        // Parse and normalize URI
        let normalized_uri = self.normalize_uri(uri);
        hasher.update(normalized_uri.as_bytes());

        // Add vary headers
        let mut vary_keys: Vec<_> = vary_headers.keys().collect();
        vary_keys.sort();

        for key in vary_keys {
            if let Some(value) = vary_headers.get(key) {
                hasher.update(b":");
                hasher.update(key.as_bytes());
                hasher.update(b"=");
                hasher.update(value.as_bytes());
            }
        }

        hasher.finalize().to_hex().to_string()
    }

    fn normalize_uri(&self, uri: &str) -> String {
        if !self.config.cache_query_strings {
            // Strip query string entirely
            return uri.split('?').next().unwrap_or(uri).to_string();
        }

        // Parse query string and remove ignored params
        if let Some(pos) = uri.find('?') {
            let path = &uri[..pos];
            let query = &uri[pos + 1..];

            let filtered_params: Vec<_> = query
                .split('&')
                .filter(|param| {
                    let key = param.split('=').next().unwrap_or("");
                    !self.config.ignore_query_params.contains(&key.to_string())
                })
                .collect();

            if filtered_params.is_empty() {
                path.to_string()
            } else {
                format!("{}?{}", path, filtered_params.join("&"))
            }
        } else {
            uri.to_string()
        }
    }
}

/// Page cache with tag-based invalidation
pub struct PageCache {
    config: PageCacheConfig,
    /// Main cache storage: key -> entry
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Tag index: tag -> set of cache keys
    tag_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Key generator
    key_generator: CacheKeyGenerator,
    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub invalidations: u64,
    pub evictions: u64,
    pub total_bytes_served: u64,
}

impl PageCache {
    pub fn new(config: PageCacheConfig) -> Self {
        let key_gen_config = config.clone();
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            key_generator: CacheKeyGenerator::new(key_gen_config),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Check if path should be cached
    pub fn should_cache(&self, path: &str) -> bool {
        !self.config.excluded_paths.iter().any(|p| path.starts_with(p))
    }

    /// Generate cache key
    pub fn cache_key(&self, method: &str, uri: &str, vary_headers: &HashMap<String, String>) -> String {
        self.key_generator.generate(method, uri, vary_headers)
    }

    /// Get cached page
    pub fn get(&self, key: &str) -> Option<CachedPage> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(key) {
            // Check TTL
            let age = entry.created.elapsed().as_secs();
            if age > entry.page.ttl {
                // Expired
                cache.remove(key);
                self.stats.write().misses += 1;
                return None;
            }

            entry.hits += 1;
            self.stats.write().hits += 1;
            self.stats.write().total_bytes_served += entry.page.content.len() as u64;

            Some(entry.page.clone())
        } else {
            self.stats.write().misses += 1;
            None
        }
    }

    /// Store page in cache
    pub fn store(&self, key: String, page: CachedPage) {
        // Check capacity and evict if needed
        self.evict_if_needed();

        // Index tags
        let tags = page.tags.clone();
        {
            let mut tag_index = self.tag_index.write();
            for tag in &tags {
                tag_index
                    .entry(tag.clone())
                    .or_insert_with(HashSet::new)
                    .insert(key.clone());
            }
        }

        // Store entry
        let entry = CacheEntry {
            page,
            created: Instant::now(),
            hits: 0,
        };

        self.cache.write().insert(key, entry);
        self.stats.write().stores += 1;
    }

    /// Invalidate by cache key
    pub fn invalidate(&self, key: &str) {
        if let Some(entry) = self.cache.write().remove(key) {
            // Remove from tag index
            let mut tag_index = self.tag_index.write();
            for tag in &entry.page.tags {
                if let Some(keys) = tag_index.get_mut(tag) {
                    keys.remove(key);
                }
            }
            self.stats.write().invalidations += 1;
        }
    }

    /// Invalidate by tag
    pub fn invalidate_by_tag(&self, tag: &str) {
        let keys_to_remove: Vec<String> = {
            let tag_index = self.tag_index.read();
            tag_index
                .get(tag)
                .map(|keys| keys.iter().cloned().collect())
                .unwrap_or_default()
        };

        for key in keys_to_remove {
            self.invalidate(&key);
        }

        // Remove tag from index
        self.tag_index.write().remove(tag);
    }

    /// Invalidate by multiple tags (any match)
    pub fn invalidate_by_tags(&self, tags: &[String]) {
        for tag in tags {
            self.invalidate_by_tag(tag);
        }
    }

    /// Invalidate by tag pattern (glob-style)
    pub fn invalidate_by_pattern(&self, pattern: &str) -> usize {
        let regex = glob_to_regex(pattern);
        let mut count = 0;

        let tags_to_invalidate: Vec<String> = {
            let tag_index = self.tag_index.read();
            tag_index
                .keys()
                .filter(|tag| regex.is_match(tag))
                .cloned()
                .collect()
        };

        for tag in tags_to_invalidate {
            let keys_count = self.tag_index.read()
                .get(&tag)
                .map(|k| k.len())
                .unwrap_or(0);
            self.invalidate_by_tag(&tag);
            count += keys_count;
        }

        count
    }

    /// Clear entire cache
    pub fn clear(&self) {
        self.cache.write().clear();
        self.tag_index.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    /// Evict entries if over capacity
    fn evict_if_needed(&self) {
        let mut cache = self.cache.write();

        if cache.len() >= self.config.max_entries {
            // LRU-style eviction: remove entries with lowest hits and oldest
            let mut entries: Vec<_> = cache.iter()
                .map(|(k, v)| (k.clone(), v.hits, v.created))
                .collect();

            // Sort by hits (ascending) then by age (oldest first)
            entries.sort_by(|a, b| {
                a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2))
            });

            // Remove bottom 10%
            let to_remove = (self.config.max_entries / 10).max(1);
            let mut stats = self.stats.write();

            for (key, _, _) in entries.into_iter().take(to_remove) {
                if let Some(entry) = cache.remove(&key) {
                    // Clean up tag index
                    let mut tag_index = self.tag_index.write();
                    for tag in &entry.page.tags {
                        if let Some(keys) = tag_index.get_mut(tag) {
                            keys.remove(&key);
                        }
                    }
                }
                stats.evictions += 1;
            }
        }
    }

    /// Warm cache with precomputed pages
    pub fn warm(&self, entries: Vec<(String, CachedPage)>) {
        for (key, page) in entries {
            self.store(key, page);
        }
    }
}

/// Generate ETag from content
pub fn generate_etag(content: &str) -> String {
    let hash = blake3::hash(content.as_bytes());
    format!("\"{}\"", &hash.to_hex()[..16])
}

/// Cache tag builder for common patterns
pub struct CacheTagBuilder {
    tags: HashSet<String>,
}

impl CacheTagBuilder {
    pub fn new() -> Self {
        Self {
            tags: HashSet::new(),
        }
    }

    /// Add a post tag
    pub fn post(mut self, post_id: i64) -> Self {
        self.tags.insert(format!("post:{}", post_id));
        self
    }

    /// Add a post type tag
    pub fn post_type(mut self, post_type: &str) -> Self {
        self.tags.insert(format!("post_type:{}", post_type));
        self
    }

    /// Add a term tag
    pub fn term(mut self, taxonomy: &str, term_id: i64) -> Self {
        self.tags.insert(format!("term:{}:{}", taxonomy, term_id));
        self
    }

    /// Add a user tag
    pub fn user(mut self, user_id: i64) -> Self {
        self.tags.insert(format!("user:{}", user_id));
        self
    }

    /// Add a template tag
    pub fn template(mut self, template: &str) -> Self {
        self.tags.insert(format!("template:{}", template));
        self
    }

    /// Add a menu tag
    pub fn menu(mut self, menu_id: i64) -> Self {
        self.tags.insert(format!("menu:{}", menu_id));
        self
    }

    /// Add a widget area tag
    pub fn widget_area(mut self, area: &str) -> Self {
        self.tags.insert(format!("widget_area:{}", area));
        self
    }

    /// Add a custom tag
    pub fn custom(mut self, tag: &str) -> Self {
        self.tags.insert(tag.to_string());
        self
    }

    /// Add global tag (invalidated on any content change)
    pub fn global(mut self) -> Self {
        self.tags.insert("global".to_string());
        self
    }

    /// Build the tag set
    pub fn build(self) -> HashSet<String> {
        self.tags
    }
}

impl Default for CacheTagBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert glob pattern to regex
fn glob_to_regex(pattern: &str) -> regex::Regex {
    let escaped = regex::escape(pattern);
    let regex_pattern = escaped
        .replace(r"\*", ".*")
        .replace(r"\?", ".");
    regex::Regex::new(&format!("^{}$", regex_pattern)).unwrap()
}

/// Cache warming configuration
#[derive(Debug, Clone)]
pub struct WarmingConfig {
    /// URLs to warm
    pub urls: Vec<String>,
    /// Concurrent warm requests
    pub concurrency: usize,
    /// Delay between requests (ms)
    pub delay_ms: u64,
}

impl Default for WarmingConfig {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            concurrency: 4,
            delay_ms: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let config = PageCacheConfig::default();
        let generator = CacheKeyGenerator::new(config);

        let key1 = generator.generate("GET", "/page", &HashMap::new());
        let key2 = generator.generate("GET", "/page", &HashMap::new());
        assert_eq!(key1, key2);

        let key3 = generator.generate("GET", "/other", &HashMap::new());
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_store_and_get() {
        let cache = PageCache::new(PageCacheConfig::default());

        let page = CachedPage {
            content: "<html>Test</html>".to_string(),
            status_code: 200,
            headers: HashMap::new(),
            content_type: "text/html".to_string(),
            tags: CacheTagBuilder::new().post(1).build(),
            created_at: chrono::Utc::now().timestamp(),
            ttl: 3600,
            etag: generate_etag("<html>Test</html>"),
            last_modified: chrono::Utc::now().timestamp(),
        };

        cache.store("test_key".to_string(), page.clone());

        let retrieved = cache.get("test_key");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, page.content);
    }

    #[test]
    fn test_tag_invalidation() {
        let cache = PageCache::new(PageCacheConfig::default());

        let page1 = CachedPage {
            content: "Page 1".to_string(),
            status_code: 200,
            headers: HashMap::new(),
            content_type: "text/html".to_string(),
            tags: CacheTagBuilder::new().post(1).build(),
            created_at: chrono::Utc::now().timestamp(),
            ttl: 3600,
            etag: generate_etag("Page 1"),
            last_modified: chrono::Utc::now().timestamp(),
        };

        let page2 = CachedPage {
            content: "Page 2".to_string(),
            status_code: 200,
            headers: HashMap::new(),
            content_type: "text/html".to_string(),
            tags: CacheTagBuilder::new().post(1).post(2).build(),
            created_at: chrono::Utc::now().timestamp(),
            ttl: 3600,
            etag: generate_etag("Page 2"),
            last_modified: chrono::Utc::now().timestamp(),
        };

        cache.store("key1".to_string(), page1);
        cache.store("key2".to_string(), page2);

        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_some());

        // Invalidate by post:1 tag
        cache.invalidate_by_tag("post:1");

        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_none());
    }
}
