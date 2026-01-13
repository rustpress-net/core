//! High-level cache API.

use crate::backend::CacheBackend;
use crate::key::CacheKey;
use rustpress_core::error::{Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Key prefix for namespacing
    pub prefix: Option<String>,
    /// Enable cache metrics
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600),
            prefix: None,
            enable_metrics: true,
        }
    }
}

/// High-level cache interface
pub struct Cache {
    backend: Arc<dyn CacheBackend>,
    config: CacheConfig,
}

impl Cache {
    /// Create a new cache with the given backend
    pub fn new(backend: Arc<dyn CacheBackend>) -> Self {
        Self {
            backend,
            config: CacheConfig::default(),
        }
    }

    /// Create a cache with custom configuration
    pub fn with_config(backend: Arc<dyn CacheBackend>, config: CacheConfig) -> Self {
        Self { backend, config }
    }

    /// Get the full key with prefix
    fn full_key(&self, key: &CacheKey) -> CacheKey {
        match &self.config.prefix {
            Some(prefix) => key.prefix(prefix),
            None => key.clone(),
        }
    }

    /// Get a value from the cache
    pub async fn get<T: DeserializeOwned>(&self, key: impl Into<CacheKey>) -> Result<Option<T>> {
        let key = self.full_key(&key.into());

        match self.backend.get(&key).await? {
            Some(bytes) => {
                let value: T = serde_json::from_slice(&bytes).map_err(|e| Error::Cache {
                    message: format!("Deserialization failed: {}", e),
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get a value or compute it if not present
    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: impl Into<CacheKey>,
        ttl: Option<Duration>,
        f: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let key = key.into();

        if let Some(value) = self.get::<T>(key.clone()).await? {
            return Ok(value);
        }

        let value = f().await?;
        self.set(key, &value, ttl).await?;
        Ok(value)
    }

    /// Set a value in the cache
    pub async fn set<T: Serialize>(
        &self,
        key: impl Into<CacheKey>,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let key = self.full_key(&key.into());
        let bytes = serde_json::to_vec(value).map_err(|e| Error::Cache {
            message: format!("Serialization failed: {}", e),
        })?;

        let ttl = ttl.or(Some(self.config.default_ttl));
        self.backend.set(&key, bytes, ttl).await
    }

    /// Set a value with the default TTL
    pub async fn set_default<T: Serialize>(
        &self,
        key: impl Into<CacheKey>,
        value: &T,
    ) -> Result<()> {
        self.set(key, value, Some(self.config.default_ttl)).await
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: impl Into<CacheKey>) -> Result<bool> {
        let key = self.full_key(&key.into());
        self.backend.delete(&key).await
    }

    /// Check if a key exists
    pub async fn exists(&self, key: impl Into<CacheKey>) -> Result<bool> {
        let key = self.full_key(&key.into());
        self.backend.exists(&key).await
    }

    /// Delete keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        let pattern = match &self.config.prefix {
            Some(prefix) => format!("{}:{}", prefix, pattern),
            None => pattern.to_string(),
        };
        self.backend.delete_pattern(&pattern).await
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        self.backend.clear().await
    }

    /// Get remaining TTL for a key
    pub async fn ttl(&self, key: impl Into<CacheKey>) -> Result<Option<Duration>> {
        let key = self.full_key(&key.into());
        self.backend.ttl(&key).await
    }

    /// Increment a counter
    pub async fn increment(&self, key: impl Into<CacheKey>, delta: i64) -> Result<i64> {
        let key = self.full_key(&key.into());
        self.backend.increment(&key, delta).await
    }

    /// Decrement a counter
    pub async fn decrement(&self, key: impl Into<CacheKey>, delta: i64) -> Result<i64> {
        let key = self.full_key(&key.into());
        self.backend.decrement(&key, delta).await
    }

    /// Remember a value (get or compute and store)
    pub async fn remember<T, F, Fut>(&self, key: impl Into<CacheKey>, ttl: Duration, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.get_or_set(key, Some(ttl), f).await
    }

    /// Remember forever (no TTL)
    pub async fn remember_forever<T, F, Fut>(&self, key: impl Into<CacheKey>, f: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        self.get_or_set(key, None, f).await
    }

    /// Get multiple values
    pub async fn get_many<T: DeserializeOwned>(
        &self,
        keys: &[CacheKey],
    ) -> Result<Vec<Option<T>>> {
        let full_keys: Vec<CacheKey> = keys.iter().map(|k| self.full_key(k)).collect();
        let bytes_list = self.backend.get_many(&full_keys).await?;

        bytes_list
            .into_iter()
            .map(|opt_bytes| {
                opt_bytes
                    .map(|bytes| {
                        serde_json::from_slice(&bytes).map_err(|e| Error::Cache {
                            message: format!("Deserialization failed: {}", e),
                        })
                    })
                    .transpose()
            })
            .collect()
    }

    /// Set multiple values
    pub async fn set_many<T: Serialize>(
        &self,
        entries: &[(CacheKey, T)],
        ttl: Option<Duration>,
    ) -> Result<()> {
        let serialized: Result<Vec<(CacheKey, Vec<u8>)>> = entries
            .iter()
            .map(|(key, value)| {
                let bytes = serde_json::to_vec(value).map_err(|e| Error::Cache {
                    message: format!("Serialization failed: {}", e),
                })?;
                Ok((self.full_key(key), bytes))
            })
            .collect();

        let ttl = ttl.or(Some(self.config.default_ttl));
        self.backend.set_many(&serialized?, ttl).await
    }

    /// Forget (delete) a cached value
    pub async fn forget(&self, key: impl Into<CacheKey>) -> Result<bool> {
        self.delete(key).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        self.backend.health_check().await
    }

    /// Tag-based cache invalidation helper
    pub async fn invalidate_tags(&self, tags: &[&str]) -> Result<u64> {
        let mut total = 0;
        for tag in tags {
            let pattern = format!("tag:{}:*", tag);
            total += self.delete_pattern(&pattern).await?;
        }
        Ok(total)
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.backend.stats().await
    }

    /// Clear cache entries by prefix
    pub async fn clear_by_prefix(&self, prefix: &str) -> Result<u64> {
        let pattern = format!("{}*", prefix);
        self.delete_pattern(&pattern).await
    }

    /// Clear cache entries by tag
    pub async fn clear_by_tag(&self, tag: &str) -> Result<u64> {
        let pattern = format!("tag:{}:*", tag);
        self.delete_pattern(&pattern).await
    }

    /// List cache keys matching an optional prefix
    pub async fn list_keys(&self, prefix: Option<&str>) -> Vec<String> {
        self.backend.list_keys(prefix).await
    }
}

// Re-export CacheStats from crate root
pub use crate::CacheStats;

/// Cache tags for organizing cached data
pub struct CacheTags<'a> {
    cache: &'a Cache,
    tags: Vec<String>,
}

impl<'a> CacheTags<'a> {
    pub fn new(cache: &'a Cache, tags: Vec<String>) -> Self {
        Self { cache, tags }
    }

    /// Set a value with tags
    pub async fn set<T: Serialize>(
        &self,
        key: impl Into<CacheKey>,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let key = key.into();

        // Store the value
        self.cache.set(key.clone(), value, ttl).await?;

        // Store tag references
        for tag in &self.tags {
            let tag_key = CacheKey::new(format!("tag:{}:{}", tag, key.as_str()));
            self.cache.set(tag_key, &"1", ttl).await?;
        }

        Ok(())
    }

    /// Flush all values with these tags
    pub async fn flush(&self) -> Result<u64> {
        self.cache
            .invalidate_tags(&self.tags.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MemoryBackend;

    fn create_test_cache() -> Cache {
        let backend = Arc::new(MemoryBackend::new(1000));
        Cache::new(backend)
    }

    #[tokio::test]
    async fn test_get_set() {
        let cache = create_test_cache();

        cache
            .set("test_key", &"hello world", None)
            .await
            .unwrap();
        let value: String = cache.get("test_key").await.unwrap().unwrap();
        assert_eq!(value, "hello world");
    }

    #[tokio::test]
    async fn test_get_or_set() {
        let cache = create_test_cache();

        // First call computes the value
        let value: String = cache
            .get_or_set("computed", None, || async { Ok("computed_value".to_string()) })
            .await
            .unwrap();
        assert_eq!(value, "computed_value");

        // Second call returns cached value
        let value: String = cache
            .get_or_set("computed", None, || async {
                Ok("should_not_be_called".to_string())
            })
            .await
            .unwrap();
        assert_eq!(value, "computed_value");
    }

    #[tokio::test]
    async fn test_exists_delete() {
        let cache = create_test_cache();

        cache.set("key", &"value", None).await.unwrap();
        assert!(cache.exists("key").await.unwrap());

        cache.delete("key").await.unwrap();
        assert!(!cache.exists("key").await.unwrap());
    }

    #[tokio::test]
    async fn test_increment_decrement() {
        let cache = create_test_cache();

        let val = cache.increment("counter", 5).await.unwrap();
        assert_eq!(val, 5);

        let val = cache.increment("counter", 3).await.unwrap();
        assert_eq!(val, 8);

        let val = cache.decrement("counter", 2).await.unwrap();
        assert_eq!(val, 6);
    }

    #[tokio::test]
    async fn test_complex_types() {
        let cache = create_test_cache();

        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        cache.set("struct", &data, None).await.unwrap();
        let retrieved: TestStruct = cache.get("struct").await.unwrap().unwrap();
        assert_eq!(retrieved, data);
    }
}
