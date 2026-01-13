//! Cache Service
//!
//! Caching service for analytics data to reduce API calls and improve performance.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, info, warn};

/// Database pool type alias
type DbPool = Arc<dyn std::any::Any + Send + Sync>;

/// Cache entry with expiration
#[derive(Debug)]
struct CacheEntry {
    data: String,
    expires_at: Instant,
}

/// Cache Service for storing analytics data
pub struct CacheService {
    /// In-memory cache
    memory_cache: RwLock<std::collections::HashMap<String, CacheEntry>>,
    /// Database pool for persistent cache (reserved for future use)
    #[allow(dead_code)]
    db: DbPool,
    /// Cache duration in minutes
    cache_duration: u32,
    /// Maximum memory cache size
    max_memory_entries: usize,
}

impl CacheService {
    /// Create a new cache service
    pub fn new(db: DbPool, cache_duration_minutes: u32) -> Self {
        Self {
            memory_cache: RwLock::new(std::collections::HashMap::new()),
            db,
            cache_duration: cache_duration_minutes,
            max_memory_entries: 1000,
        }
    }

    /// Get a cached value
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // Check memory cache first
        {
            let cache = self.memory_cache.read();
            if let Some(entry) = cache.get(key) {
                if entry.expires_at > Instant::now() {
                    debug!("Cache hit (memory) for key: {}", key);
                    return serde_json::from_str(&entry.data).ok();
                }
            }
        }

        // Check database cache
        if let Some(data) = self.get_from_db(key).await {
            debug!("Cache hit (database) for key: {}", key);

            // Promote to memory cache
            self.set_memory(key, &data);

            return serde_json::from_str(&data).ok();
        }

        debug!("Cache miss for key: {}", key);
        None
    }

    /// Set a cached value
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) {
        let data = match serde_json::to_string(value) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to serialize cache value: {}", e);
                return;
            }
        };

        // Set in memory cache
        self.set_memory(key, &data);

        // Set in database cache
        self.set_in_db(key, &data).await;

        debug!("Cached value for key: {}", key);
    }

    /// Set value in memory cache
    fn set_memory(&self, key: &str, data: &str) {
        let mut cache = self.memory_cache.write();

        // Evict old entries if cache is full
        if cache.len() >= self.max_memory_entries {
            self.evict_expired(&mut cache);

            // If still full, remove oldest entries
            if cache.len() >= self.max_memory_entries {
                let to_remove: Vec<String> = cache
                    .iter()
                    .take(self.max_memory_entries / 4)
                    .map(|(k, _)| k.clone())
                    .collect();

                for key in to_remove {
                    cache.remove(&key);
                }
            }
        }

        cache.insert(
            key.to_string(),
            CacheEntry {
                data: data.to_string(),
                expires_at: Instant::now() + Duration::from_secs(self.cache_duration as u64 * 60),
            },
        );
    }

    /// Evict expired entries from memory cache
    fn evict_expired(&self, cache: &mut std::collections::HashMap<String, CacheEntry>) {
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }

    /// Get from database cache
    async fn get_from_db(&self, _key: &str) -> Option<String> {
        // In a real implementation, this would query the database
        // For now, return None
        None
    }

    /// Set in database cache
    async fn set_in_db(&self, _key: &str, _data: &str) {
        // In a real implementation, this would insert into the database
        // Example SQL:
        // INSERT INTO rustanalytics_cache (cache_key, data, expires_at)
        // VALUES ($1, $2, $3)
        // ON CONFLICT (cache_key) DO UPDATE SET data = $2, expires_at = $3
    }

    /// Delete a cached value
    pub async fn delete(&self, key: &str) {
        // Remove from memory cache
        {
            let mut cache = self.memory_cache.write();
            cache.remove(key);
        }

        // Remove from database cache
        self.delete_from_db(key).await;

        debug!("Deleted cache key: {}", key);
    }

    /// Delete from database cache
    async fn delete_from_db(&self, _key: &str) {
        // DELETE FROM rustanalytics_cache WHERE cache_key = $1
    }

    /// Delete cached values matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) {
        // Remove from memory cache
        {
            let mut cache = self.memory_cache.write();
            cache.retain(|k, _| !k.starts_with(pattern));
        }

        // Remove from database cache
        self.delete_pattern_from_db(pattern).await;

        debug!("Deleted cache keys matching pattern: {}", pattern);
    }

    /// Delete pattern from database cache
    async fn delete_pattern_from_db(&self, _pattern: &str) {
        // DELETE FROM rustanalytics_cache WHERE cache_key LIKE $1 || '%'
    }

    /// Clear all cache
    pub async fn clear(&self) {
        // Clear memory cache
        {
            let mut cache = self.memory_cache.write();
            cache.clear();
        }

        // Clear database cache
        self.clear_db().await;

        info!("Cleared all analytics cache");
    }

    /// Clear database cache
    async fn clear_db(&self) {
        // TRUNCATE TABLE rustanalytics_cache
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.memory_cache.read();
        let now = Instant::now();

        let mut total_entries = 0;
        let mut expired_entries = 0;
        let mut total_size = 0;

        for entry in cache.values() {
            total_entries += 1;
            total_size += entry.data.len();
            if entry.expires_at <= now {
                expired_entries += 1;
            }
        }

        CacheStats {
            memory_entries: total_entries,
            memory_size_bytes: total_size,
            expired_entries,
            max_entries: self.max_memory_entries,
            cache_duration_minutes: self.cache_duration,
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup(&self) {
        // Cleanup memory cache
        {
            let mut cache = self.memory_cache.write();
            self.evict_expired(&mut cache);
        }

        // Cleanup database cache
        self.cleanup_db().await;

        info!("Cache cleanup completed");
    }

    /// Cleanup expired entries from database
    async fn cleanup_db(&self) {
        // DELETE FROM rustanalytics_cache WHERE expires_at < NOW()
    }

    /// Warm up cache with common queries
    pub async fn warm_up(&self) {
        info!("Warming up analytics cache...");

        // In a real implementation, this would pre-fetch common data
        // like today's overview, realtime data, etc.
    }
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub memory_size_bytes: usize,
    pub expired_entries: usize,
    pub max_entries: usize,
    pub cache_duration_minutes: u32,
}

impl std::fmt::Debug for CacheService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheService")
            .field("cache_duration", &self.cache_duration)
            .field("max_memory_entries", &self.max_memory_entries)
            .finish()
    }
}
