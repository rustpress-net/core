//! # RustPress Cache
//!
//! Caching layer supporting Redis and in-memory backends.

pub mod backend;
pub mod cache;
pub mod key;

pub use backend::{CacheBackend, MemoryBackend};
pub use cache::{Cache, CacheConfig};
pub use key::CacheKey;

#[cfg(feature = "redis")]
pub use backend::RedisBackend;

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of entries in cache
    pub entries: u64,
    /// Approximate memory usage in bytes
    pub memory_bytes: u64,
    /// Number of evictions
    pub evictions: u64,
}
