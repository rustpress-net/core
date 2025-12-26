//! Cache backend implementations.

use crate::key::CacheKey;
use crate::CacheStats;
use async_trait::async_trait;
use rustpress_core::error::{Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Cache backend trait
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>>;

    /// Set a value in the cache with optional TTL
    async fn set(&self, key: &CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &CacheKey) -> Result<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &CacheKey) -> Result<bool>;

    /// Delete multiple keys by pattern
    async fn delete_pattern(&self, pattern: &str) -> Result<u64>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;

    /// Get remaining TTL for a key
    async fn ttl(&self, key: &CacheKey) -> Result<Option<Duration>>;

    /// Increment a numeric value
    async fn increment(&self, key: &CacheKey, delta: i64) -> Result<i64>;

    /// Decrement a numeric value
    async fn decrement(&self, key: &CacheKey, delta: i64) -> Result<i64>;

    /// Get multiple values
    async fn get_many(&self, keys: &[CacheKey]) -> Result<Vec<Option<Vec<u8>>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    /// Set multiple values
    async fn set_many(
        &self,
        entries: &[(CacheKey, Vec<u8>)],
        ttl: Option<Duration>,
    ) -> Result<()> {
        for (key, value) in entries {
            self.set(key, value.clone(), ttl).await?;
        }
        Ok(())
    }

    /// Health check
    async fn health_check(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats {
        CacheStats::default()
    }

    /// List keys matching an optional prefix
    async fn list_keys(&self, _prefix: Option<&str>) -> Vec<String> {
        Vec::new()
    }
}

/// In-memory cache backend using moka
#[cfg(feature = "memory")]
pub struct MemoryBackend {
    cache: moka::future::Cache<String, Vec<u8>>,
}

#[cfg(feature = "memory")]
impl MemoryBackend {
    pub fn new(max_capacity: u64) -> Self {
        Self {
            cache: moka::future::Cache::builder()
                .max_capacity(max_capacity)
                .build(),
        }
    }

    pub fn with_ttl(max_capacity: u64, default_ttl: Duration) -> Self {
        Self {
            cache: moka::future::Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(default_ttl)
                .build(),
        }
    }
}

#[cfg(feature = "memory")]
#[async_trait]
impl CacheBackend for MemoryBackend {
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        Ok(self.cache.get(&key.as_str()).await)
    }

    async fn set(&self, key: &CacheKey, value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        // For moka, we'd need to use a different approach for per-key TTL
        // For now, just insert with the cache's default TTL
        self.cache.insert(key.as_str(), value).await;
        Ok(())
    }

    async fn delete(&self, key: &CacheKey) -> Result<bool> {
        let existed = self.cache.contains_key(&key.as_str());
        self.cache.remove(&key.as_str()).await;
        Ok(existed)
    }

    async fn exists(&self, key: &CacheKey) -> Result<bool> {
        Ok(self.cache.contains_key(&key.as_str()))
    }

    async fn delete_pattern(&self, _pattern: &str) -> Result<u64> {
        // Moka doesn't support pattern-based deletion easily
        // In production, you'd implement this differently
        Ok(0)
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        Ok(())
    }

    async fn ttl(&self, _key: &CacheKey) -> Result<Option<Duration>> {
        // Moka doesn't expose TTL easily
        Ok(None)
    }

    async fn increment(&self, key: &CacheKey, delta: i64) -> Result<i64> {
        let current = self
            .get(key)
            .await?
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let new_value = current + delta;
        self.set(key, new_value.to_string().into_bytes(), None)
            .await?;
        Ok(new_value)
    }

    async fn decrement(&self, key: &CacheKey, delta: i64) -> Result<i64> {
        self.increment(key, -delta).await
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

/// Redis cache backend
#[cfg(feature = "redis")]
pub struct RedisBackend {
    pool: deadpool_redis::Pool,
}

#[cfg(feature = "redis")]
impl RedisBackend {
    pub async fn new(url: &str) -> Result<Self> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| Error::Cache {
                message: format!("Failed to create Redis pool: {}", e),
            })?;

        Ok(Self { pool })
    }

    async fn get_connection(
        &self,
    ) -> Result<deadpool_redis::Connection> {
        self.pool.get().await.map_err(|e| Error::Cache {
            message: format!("Failed to get Redis connection: {}", e),
        })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl CacheBackend for RedisBackend {
    async fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let result: Option<Vec<u8>> = conn.get(key.as_str()).await.map_err(|e| Error::Cache {
            message: format!("Redis GET failed: {}", e),
        })?;
        Ok(result)
    }

    async fn set(&self, key: &CacheKey, value: Vec<u8>, ttl: Option<Duration>) -> Result<()> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;

        if let Some(ttl) = ttl {
            conn.set_ex(key.as_str(), value, ttl.as_secs() as u64)
                .await
                .map_err(|e| Error::Cache {
                    message: format!("Redis SETEX failed: {}", e),
                })?;
        } else {
            conn.set(key.as_str(), value)
                .await
                .map_err(|e| Error::Cache {
                    message: format!("Redis SET failed: {}", e),
                })?;
        }
        Ok(())
    }

    async fn delete(&self, key: &CacheKey) -> Result<bool> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let deleted: i64 = conn.del(key.as_str()).await.map_err(|e| Error::Cache {
            message: format!("Redis DEL failed: {}", e),
        })?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &CacheKey) -> Result<bool> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(key.as_str()).await.map_err(|e| Error::Cache {
            message: format!("Redis EXISTS failed: {}", e),
        })?;
        Ok(exists)
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<u64> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = conn.keys(pattern).await.map_err(|e| Error::Cache {
            message: format!("Redis KEYS failed: {}", e),
        })?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: i64 = conn.del(&keys).await.map_err(|e| Error::Cache {
            message: format!("Redis DEL failed: {}", e),
        })?;
        Ok(deleted as u64)
    }

    async fn clear(&self) -> Result<()> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        redis::cmd("FLUSHDB")
            .query_async(&mut *conn)
            .await
            .map_err(|e| Error::Cache {
                message: format!("Redis FLUSHDB failed: {}", e),
            })?;
        Ok(())
    }

    async fn ttl(&self, key: &CacheKey) -> Result<Option<Duration>> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let ttl: i64 = conn.ttl(key.as_str()).await.map_err(|e| Error::Cache {
            message: format!("Redis TTL failed: {}", e),
        })?;

        if ttl < 0 {
            Ok(None)
        } else {
            Ok(Some(Duration::from_secs(ttl as u64)))
        }
    }

    async fn increment(&self, key: &CacheKey, delta: i64) -> Result<i64> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let value: i64 = conn
            .incr(key.as_str(), delta)
            .await
            .map_err(|e| Error::Cache {
                message: format!("Redis INCRBY failed: {}", e),
            })?;
        Ok(value)
    }

    async fn decrement(&self, key: &CacheKey, delta: i64) -> Result<i64> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let value: i64 = conn
            .decr(key.as_str(), delta)
            .await
            .map_err(|e| Error::Cache {
                message: format!("Redis DECRBY failed: {}", e),
            })?;
        Ok(value)
    }

    async fn health_check(&self) -> Result<()> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;
        let _: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| Error::Cache {
                message: format!("Redis PING failed: {}", e),
            })?;
        Ok(())
    }
}

/// Null cache backend (no-op)
pub struct NullBackend;

#[async_trait]
impl CacheBackend for NullBackend {
    async fn get(&self, _key: &CacheKey) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set(&self, _key: &CacheKey, _value: Vec<u8>, _ttl: Option<Duration>) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _key: &CacheKey) -> Result<bool> {
        Ok(false)
    }

    async fn exists(&self, _key: &CacheKey) -> Result<bool> {
        Ok(false)
    }

    async fn delete_pattern(&self, _pattern: &str) -> Result<u64> {
        Ok(0)
    }

    async fn clear(&self) -> Result<()> {
        Ok(())
    }

    async fn ttl(&self, _key: &CacheKey) -> Result<Option<Duration>> {
        Ok(None)
    }

    async fn increment(&self, _key: &CacheKey, _delta: i64) -> Result<i64> {
        Ok(0)
    }

    async fn decrement(&self, _key: &CacheKey, _delta: i64) -> Result<i64> {
        Ok(0)
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn test_memory_backend() {
        let backend = MemoryBackend::new(1000);

        // Test set and get
        let key = CacheKey::new("test");
        backend
            .set(&key, b"hello".to_vec(), None)
            .await
            .unwrap();
        let value = backend.get(&key).await.unwrap();
        assert_eq!(value, Some(b"hello".to_vec()));

        // Test exists
        assert!(backend.exists(&key).await.unwrap());

        // Test delete
        assert!(backend.delete(&key).await.unwrap());
        assert!(!backend.exists(&key).await.unwrap());
    }

    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn test_memory_increment() {
        let backend = MemoryBackend::new(1000);
        let key = CacheKey::new("counter");

        let val = backend.increment(&key, 5).await.unwrap();
        assert_eq!(val, 5);

        let val = backend.increment(&key, 3).await.unwrap();
        assert_eq!(val, 8);

        let val = backend.decrement(&key, 2).await.unwrap();
        assert_eq!(val, 6);
    }

    #[tokio::test]
    async fn test_null_backend() {
        let backend = NullBackend;
        let key = CacheKey::new("test");

        backend.set(&key, b"value".to_vec(), None).await.unwrap();
        assert_eq!(backend.get(&key).await.unwrap(), None);
        assert!(!backend.exists(&key).await.unwrap());
    }
}
