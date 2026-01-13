//! Object Caching with Redis Integration
//!
//! Multi-tier object caching with local memory cache and Redis backend.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use parking_lot::RwLock;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Object cache errors
#[derive(Debug, Error)]
pub enum ObjectCacheError {
    #[error("Cache miss for key: {0}")]
    CacheMiss(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Connection pool error: {0}")]
    Pool(String),

    #[error("Invalid cache group: {0}")]
    InvalidGroup(String),
}

/// Cache backend trait
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get value from cache
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectCacheError>;

    /// Set value in cache
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), ObjectCacheError>;

    /// Delete value from cache
    async fn delete(&self, key: &str) -> Result<bool, ObjectCacheError>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, ObjectCacheError>;

    /// Delete by pattern
    async fn delete_pattern(&self, pattern: &str) -> Result<u64, ObjectCacheError>;

    /// Flush all keys
    async fn flush(&self) -> Result<(), ObjectCacheError>;

    /// Increment value
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, ObjectCacheError>;

    /// Get multiple values
    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>, ObjectCacheError>;

    /// Set multiple values
    async fn mset(&self, pairs: &[(String, Vec<u8>)], ttl: Option<Duration>) -> Result<(), ObjectCacheError>;
}

/// Redis cache backend
pub struct RedisBackend {
    client: redis::Client,
    prefix: String,
}

impl RedisBackend {
    pub fn new(redis_url: &str, prefix: &str) -> Result<Self, ObjectCacheError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, ObjectCacheError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(ObjectCacheError::from)
    }
}

#[async_trait]
impl CacheBackend for RedisBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let result: Option<Vec<u8>> = conn.get(self.prefixed_key(key)).await?;
        Ok(result)
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<(), ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let prefixed = self.prefixed_key(key);

        if let Some(ttl) = ttl {
            conn.set_ex(&prefixed, value, ttl.as_secs()).await?;
        } else {
            conn.set(&prefixed, value).await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let deleted: i64 = conn.del(self.prefixed_key(key)).await?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool, ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(self.prefixed_key(key)).await?;
        Ok(exists)
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<u64, ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let prefixed_pattern = self.prefixed_key(pattern);

        // Use SCAN to find matching keys
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&prefixed_pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: i64 = conn.del(keys).await?;
        Ok(deleted as u64)
    }

    async fn flush(&self) -> Result<(), ObjectCacheError> {
        self.delete_pattern("*").await?;
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64) -> Result<i64, ObjectCacheError> {
        let mut conn = self.get_connection().await?;
        let result: i64 = conn.incr(self.prefixed_key(key), delta).await?;
        Ok(result)
    }

    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>, ObjectCacheError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.get_connection().await?;
        let prefixed_keys: Vec<String> = keys.iter().map(|k| self.prefixed_key(k)).collect();
        let results: Vec<Option<Vec<u8>>> = conn.mget(prefixed_keys).await?;
        Ok(results)
    }

    async fn mset(&self, pairs: &[(String, Vec<u8>)], ttl: Option<Duration>) -> Result<(), ObjectCacheError> {
        if pairs.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;

        // Use pipeline for efficiency
        let mut pipe = redis::pipe();

        for (key, value) in pairs {
            let prefixed = self.prefixed_key(key);
            if let Some(ttl) = ttl {
                pipe.set_ex(&prefixed, value.as_slice(), ttl.as_secs());
            } else {
                pipe.set(&prefixed, value.as_slice());
            }
        }

        pipe.query_async(&mut conn).await?;
        Ok(())
    }
}

/// In-memory cache backend using Moka
pub struct MemoryBackend {
    cache: MokaCache<String, Vec<u8>>,
}

impl MemoryBackend {
    pub fn new(max_capacity: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(3600))
            .build();

        Self { cache }
    }
}

#[async_trait]
impl CacheBackend for MemoryBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ObjectCacheError> {
        Ok(self.cache.get(key).await)
    }

    async fn set(&self, key: &str, value: &[u8], _ttl: Option<Duration>) -> Result<(), ObjectCacheError> {
        self.cache.insert(key.to_string(), value.to_vec()).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, ObjectCacheError> {
        self.cache.remove(key).await;
        Ok(true)
    }

    async fn exists(&self, key: &str) -> Result<bool, ObjectCacheError> {
        Ok(self.cache.contains_key(key))
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<u64, ObjectCacheError> {
        let regex = glob_to_regex(pattern);
        let mut count = 0u64;

        // Moka doesn't support iteration, so we can't implement pattern deletion efficiently
        // In production, consider using a different data structure for pattern matching
        tracing::warn!("Pattern deletion not efficiently supported in memory backend");

        Ok(count)
    }

    async fn flush(&self) -> Result<(), ObjectCacheError> {
        self.cache.invalidate_all();
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64) -> Result<i64, ObjectCacheError> {
        let current = self.cache.get(key).await
            .and_then(|v| {
                if v.len() == 8 {
                    Some(i64::from_le_bytes(v.try_into().ok()?))
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let new_value = current + delta;
        self.cache.insert(key.to_string(), new_value.to_le_bytes().to_vec()).await;
        Ok(new_value)
    }

    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>, ObjectCacheError> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.cache.get(key).await);
        }
        Ok(results)
    }

    async fn mset(&self, pairs: &[(String, Vec<u8>)], _ttl: Option<Duration>) -> Result<(), ObjectCacheError> {
        for (key, value) in pairs {
            self.cache.insert(key.clone(), value.clone()).await;
        }
        Ok(())
    }
}

/// Multi-tier object cache
pub struct ObjectCache {
    /// Local (L1) cache
    local: MokaCache<String, Vec<u8>>,
    /// Remote (L2) backend
    remote: Option<Arc<dyn CacheBackend>>,
    /// Cache groups with their TTLs
    groups: Arc<RwLock<HashMap<String, GroupConfig>>>,
    /// Statistics
    stats: Arc<RwLock<ObjectCacheStats>>,
    /// Configuration
    config: ObjectCacheConfig,
}

/// Cache group configuration
#[derive(Debug, Clone)]
pub struct GroupConfig {
    pub ttl: Duration,
    pub local_ttl: Duration,
    pub persist_to_remote: bool,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(3600),
            local_ttl: Duration::from_secs(300),
            persist_to_remote: true,
        }
    }
}

/// Object cache configuration
#[derive(Debug, Clone)]
pub struct ObjectCacheConfig {
    /// Local cache max capacity
    pub local_max_capacity: u64,
    /// Default TTL
    pub default_ttl: Duration,
    /// Local cache TTL (shorter than remote)
    pub local_ttl: Duration,
    /// Enable write-through to remote
    pub write_through: bool,
    /// Redis URL (if using Redis backend)
    pub redis_url: Option<String>,
    /// Cache key prefix
    pub key_prefix: String,
}

impl Default for ObjectCacheConfig {
    fn default() -> Self {
        Self {
            local_max_capacity: 10000,
            default_ttl: Duration::from_secs(3600),
            local_ttl: Duration::from_secs(300),
            write_through: true,
            redis_url: None,
            key_prefix: "rustpress".to_string(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct ObjectCacheStats {
    pub local_hits: u64,
    pub remote_hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub deletes: u64,
}

impl ObjectCache {
    pub async fn new(config: ObjectCacheConfig) -> Result<Self, ObjectCacheError> {
        let local = MokaCache::builder()
            .max_capacity(config.local_max_capacity)
            .time_to_live(config.local_ttl)
            .build();

        let remote: Option<Arc<dyn CacheBackend>> = if let Some(ref redis_url) = config.redis_url {
            Some(Arc::new(RedisBackend::new(redis_url, &config.key_prefix)?))
        } else {
            None
        };

        let mut groups = HashMap::new();
        // Register default groups
        groups.insert("default".to_string(), GroupConfig::default());
        groups.insert(
            "options".to_string(),
            GroupConfig {
                ttl: Duration::from_secs(0), // Non-expiring
                local_ttl: Duration::from_secs(3600),
                persist_to_remote: true,
            },
        );
        groups.insert(
            "transients".to_string(),
            GroupConfig {
                ttl: Duration::from_secs(3600),
                local_ttl: Duration::from_secs(300),
                persist_to_remote: true,
            },
        );
        groups.insert(
            "posts".to_string(),
            GroupConfig {
                ttl: Duration::from_secs(86400), // 24 hours
                local_ttl: Duration::from_secs(600),
                persist_to_remote: true,
            },
        );

        Ok(Self {
            local,
            remote,
            groups: Arc::new(RwLock::new(groups)),
            stats: Arc::new(RwLock::new(ObjectCacheStats::default())),
            config,
        })
    }

    /// Create with memory-only backend
    pub fn memory_only(config: ObjectCacheConfig) -> Self {
        let local = MokaCache::builder()
            .max_capacity(config.local_max_capacity)
            .time_to_live(config.local_ttl)
            .build();

        Self {
            local,
            remote: None,
            groups: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ObjectCacheStats::default())),
            config,
        }
    }

    /// Register a cache group
    pub fn register_group(&self, name: &str, config: GroupConfig) {
        self.groups.write().insert(name.to_string(), config);
    }

    /// Build full cache key
    fn build_key(&self, group: &str, key: &str) -> String {
        format!("{}:{}:{}", self.config.key_prefix, group, key)
    }

    /// Get value from cache
    pub async fn get<T: DeserializeOwned>(&self, group: &str, key: &str) -> Result<Option<T>, ObjectCacheError> {
        let full_key = self.build_key(group, key);

        // Try local cache first
        if let Some(data) = self.local.get(&full_key).await {
            self.stats.write().local_hits += 1;
            return serde_json::from_slice(&data)
                .map(Some)
                .map_err(|e| ObjectCacheError::Serialization(e.to_string()));
        }

        // Try remote cache
        if let Some(ref remote) = self.remote {
            if let Some(data) = remote.get(&full_key).await? {
                self.stats.write().remote_hits += 1;

                // Populate local cache
                self.local.insert(full_key.clone(), data.clone()).await;

                return serde_json::from_slice(&data)
                    .map(Some)
                    .map_err(|e| ObjectCacheError::Serialization(e.to_string()));
            }
        }

        self.stats.write().misses += 1;
        Ok(None)
    }

    /// Set value in cache
    pub async fn set<T: Serialize>(&self, group: &str, key: &str, value: &T) -> Result<(), ObjectCacheError> {
        let full_key = self.build_key(group, key);
        let data = serde_json::to_vec(value)
            .map_err(|e| ObjectCacheError::Serialization(e.to_string()))?;

        let group_config = self.groups.read()
            .get(group)
            .cloned()
            .unwrap_or_default();

        // Store in local cache
        self.local.insert(full_key.clone(), data.clone()).await;

        // Store in remote cache if configured
        if self.config.write_through && group_config.persist_to_remote {
            if let Some(ref remote) = self.remote {
                let ttl = if group_config.ttl.as_secs() > 0 {
                    Some(group_config.ttl)
                } else {
                    None
                };
                remote.set(&full_key, &data, ttl).await?;
            }
        }

        self.stats.write().stores += 1;
        Ok(())
    }

    /// Set value with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        group: &str,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), ObjectCacheError> {
        let full_key = self.build_key(group, key);
        let data = serde_json::to_vec(value)
            .map_err(|e| ObjectCacheError::Serialization(e.to_string()))?;

        // Store in local cache
        self.local.insert(full_key.clone(), data.clone()).await;

        // Store in remote cache
        if let Some(ref remote) = self.remote {
            remote.set(&full_key, &data, Some(ttl)).await?;
        }

        self.stats.write().stores += 1;
        Ok(())
    }

    /// Delete from cache
    pub async fn delete(&self, group: &str, key: &str) -> Result<bool, ObjectCacheError> {
        let full_key = self.build_key(group, key);

        // Remove from local
        self.local.remove(&full_key).await;

        // Remove from remote
        let deleted = if let Some(ref remote) = self.remote {
            remote.delete(&full_key).await?
        } else {
            true
        };

        self.stats.write().deletes += 1;
        Ok(deleted)
    }

    /// Delete by group
    pub async fn delete_group(&self, group: &str) -> Result<u64, ObjectCacheError> {
        let pattern = format!("{}:{}:*", self.config.key_prefix, group);

        // Clear local cache (invalidate all as we can't pattern match efficiently)
        self.local.invalidate_all();

        // Clear remote
        if let Some(ref remote) = self.remote {
            return remote.delete_pattern(&pattern).await;
        }

        Ok(0)
    }

    /// Increment a counter
    pub async fn incr(&self, group: &str, key: &str, delta: i64) -> Result<i64, ObjectCacheError> {
        let full_key = self.build_key(group, key);

        if let Some(ref remote) = self.remote {
            let result = remote.incr(&full_key, delta).await?;

            // Update local cache
            self.local.insert(full_key, result.to_le_bytes().to_vec()).await;

            Ok(result)
        } else {
            // Local only increment
            let current = self.local.get(&full_key).await
                .and_then(|v| {
                    if v.len() == 8 {
                        Some(i64::from_le_bytes(v.try_into().ok()?))
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            let new_value = current + delta;
            self.local.insert(full_key, new_value.to_le_bytes().to_vec()).await;
            Ok(new_value)
        }
    }

    /// Get or compute value
    pub async fn get_or_set<T, F, Fut>(
        &self,
        group: &str,
        key: &str,
        f: F,
    ) -> Result<T, ObjectCacheError>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, ObjectCacheError>>,
    {
        if let Some(value) = self.get(group, key).await? {
            return Ok(value);
        }

        let value = f().await?;
        self.set(group, key, &value).await?;
        Ok(value)
    }

    /// Get multiple values
    pub async fn mget<T: DeserializeOwned>(
        &self,
        group: &str,
        keys: &[String],
    ) -> Result<Vec<Option<T>>, ObjectCacheError> {
        let full_keys: Vec<String> = keys.iter()
            .map(|k| self.build_key(group, k))
            .collect();

        // Try local first
        let mut results = Vec::with_capacity(keys.len());
        let mut missing_indices = Vec::new();
        let mut missing_keys = Vec::new();

        for (i, full_key) in full_keys.iter().enumerate() {
            if let Some(data) = self.local.get(full_key).await {
                results.push(serde_json::from_slice(&data).ok());
            } else {
                results.push(None);
                missing_indices.push(i);
                missing_keys.push(full_key.clone());
            }
        }

        // Fetch missing from remote
        if !missing_keys.is_empty() {
            if let Some(ref remote) = self.remote {
                let remote_results = remote.mget(&missing_keys).await?;

                for (idx, data) in missing_indices.iter().zip(remote_results.into_iter()) {
                    if let Some(data) = data {
                        // Update local cache
                        self.local.insert(full_keys[*idx].clone(), data.clone()).await;

                        results[*idx] = serde_json::from_slice(&data).ok();
                    }
                }
            }
        }

        Ok(results)
    }

    /// Flush all caches
    pub async fn flush(&self) -> Result<(), ObjectCacheError> {
        self.local.invalidate_all();

        if let Some(ref remote) = self.remote {
            remote.flush().await?;
        }

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> ObjectCacheStats {
        self.stats.read().clone()
    }

    /// Check if remote backend is available
    pub fn has_remote(&self) -> bool {
        self.remote.is_some()
    }

    /// Clear local cache (synchronous)
    pub fn clear_local(&self) {
        self.local.invalidate_all();
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

/// Cache lock for distributed locking
pub struct CacheLock {
    cache: Arc<ObjectCache>,
    group: String,
    key: String,
    token: String,
}

impl CacheLock {
    pub async fn acquire(
        cache: Arc<ObjectCache>,
        group: &str,
        key: &str,
        ttl: Duration,
    ) -> Option<Self> {
        let token = uuid::Uuid::new_v4().to_string();
        let lock_key = format!("_lock:{}", key);

        // Try to set lock (would need atomic SET NX in production)
        let existing: Option<String> = cache.get(group, &lock_key).await.ok().flatten();

        if existing.is_none() {
            if cache.set_with_ttl(group, &lock_key, &token, ttl).await.is_ok() {
                return Some(Self {
                    cache,
                    group: group.to_string(),
                    key: lock_key,
                    token,
                });
            }
        }

        None
    }

    pub async fn release(self) -> Result<(), ObjectCacheError> {
        // Verify we still own the lock
        let current: Option<String> = self.cache.get(&self.group, &self.key).await?;

        if current.as_ref() == Some(&self.token) {
            self.cache.delete(&self.group, &self.key).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() {
        let config = ObjectCacheConfig::default();
        let cache = ObjectCache::memory_only(config);

        cache.set("test", "key1", &"value1".to_string()).await.unwrap();

        let result: Option<String> = cache.get("test", "key1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let config = ObjectCacheConfig::default();
        let cache = ObjectCache::memory_only(config);

        let result: Option<String> = cache.get("test", "nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_increment() {
        let config = ObjectCacheConfig::default();
        let cache = ObjectCache::memory_only(config);

        let result1 = cache.incr("counters", "visits", 1).await.unwrap();
        assert_eq!(result1, 1);

        let result2 = cache.incr("counters", "visits", 5).await.unwrap();
        assert_eq!(result2, 6);
    }
}
