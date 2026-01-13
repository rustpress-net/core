//! RustPress Performance Optimization and Caching System
//!
//! This crate provides comprehensive performance optimization features for RustPress CMS,
//! including multi-tier caching, image optimization, CDN integration, and more.
//!
//! # Features
//!
//! - **Page Caching** - Full page caching with tag-based invalidation
//! - **Object Caching** - Multi-tier caching with Redis and in-memory backends
//! - **Query Caching** - Database query result caching with automatic invalidation
//! - **HTTP Caching** - Cache-Control headers, ETags, and conditional requests
//! - **Static Files** - Static file serving with cache busting and compression
//! - **Query Logging** - Database query optimization logging and analysis
//! - **Lazy Loading** - Lazy loading for admin components
//! - **Minification** - CSS, JavaScript, and HTML minification
//! - **Image Optimization** - Image resizing, format conversion, and compression
//! - **CDN Integration** - URL rewriting and cache purging for CDNs
//! - **Preloading** - Resource hints for preloading and prefetching
//! - **Service Worker** - Offline support and caching strategies
//! - **SSR Optimization** - Server-side rendering with streaming and hydration
//! - **ISR** - Incremental static regeneration
//! - **Edge Caching** - Edge/CDN cache configuration
//! - **Connection Pooling** - Database connection pool tuning
//! - **Profiling** - Performance profiling endpoints
//! - **Load Balancing** - Session handling for load-balanced environments
//!
//! # Example
//!
//! ```rust,ignore
//! use rustpress_performance::{
//!     page_cache::{PageCache, PageCacheConfig, CachedPage, CacheTagBuilder},
//!     object_cache::{ObjectCache, ObjectCacheConfig},
//!     http_cache::{CacheControl, ETag, HttpCacheHeaders},
//! };
//!
//! // Create page cache
//! let page_cache = PageCache::new(PageCacheConfig::default());
//!
//! // Cache a page with tags
//! let page = CachedPage {
//!     content: "<html>...</html>".to_string(),
//!     status_code: 200,
//!     headers: HashMap::new(),
//!     content_type: "text/html".to_string(),
//!     tags: CacheTagBuilder::new().post(1).post_type("post").build(),
//!     created_at: chrono::Utc::now().timestamp(),
//!     ttl: 3600,
//!     etag: "\"abc123\"".to_string(),
//!     last_modified: chrono::Utc::now().timestamp(),
//! };
//!
//! page_cache.store("page_key".to_string(), page);
//!
//! // Invalidate by tag
//! page_cache.invalidate_by_tag("post:1");
//! ```

pub mod page_cache;
pub mod object_cache;
pub mod query_cache;
pub mod http_cache;
pub mod static_files;
pub mod query_logging;
pub mod lazy_loading;
pub mod minification;
pub mod image_optimization;
pub mod cdn;
pub mod preload;
pub mod service_worker;
pub mod ssr;
pub mod isr;
pub mod edge_cache;
pub mod connection_pool;
pub mod profiling;
pub mod load_balancing;

// Re-export commonly used types
pub use page_cache::{PageCache, PageCacheConfig, CachedPage, CacheTagBuilder};
pub use object_cache::{ObjectCache, ObjectCacheConfig, CacheBackend};
pub use query_cache::{QueryCache, QueryCacheConfig};
pub use http_cache::{CacheControl, ETag, HttpCacheHeaders, CacheProfile};
pub use static_files::{StaticFileServer, AssetManifest};
pub use query_logging::{QueryLogger, QueryLoggerConfig, QueryLogEntry};
pub use lazy_loading::{LazyLoadingRegistry, LazyComponent};
pub use minification::{Minifier, MinificationConfig, MinifiedAsset};
pub use image_optimization::{ImageOptimizer, ImageOptimizerConfig, OptimizedImage};
pub use cdn::{CdnManager, CdnConfig, CdnRewriter, CdnPurger};
pub use preload::{ResourceHint, ResourceHintsManager, HintType};
pub use service_worker::{ServiceWorkerGenerator, ServiceWorkerConfig};
pub use ssr::{SsrRenderer, SsrConfig, StreamingRenderer};
pub use isr::{IsrHandler, IsrConfig, IsrStore, StaticPage};
pub use edge_cache::{EdgeCacheConfig, EdgeCacheHeaders, EdgeCacheRule};
pub use connection_pool::{PoolConfig, PoolMonitor, PoolStats, PoolHealth};
pub use profiling::{Profiler, ProfilingConfig, RequestTrace, PerformanceMetrics};
pub use load_balancing::{LoadBalancer, LoadBalancingStrategy, Session, SessionConfig};

/// Performance optimization prelude - import commonly used types
pub mod prelude {
    pub use crate::page_cache::{PageCache, PageCacheConfig, CachedPage, CacheTagBuilder};
    pub use crate::object_cache::{ObjectCache, ObjectCacheConfig};
    pub use crate::query_cache::{QueryCache, QueryCacheConfig};
    pub use crate::http_cache::{CacheControl, ETag, HttpCacheHeaders};
    pub use crate::static_files::{StaticFileServer, AssetManifest};
    pub use crate::cdn::{CdnManager, CdnConfig};
    pub use crate::profiling::{Profiler, ProfilingConfig};
}

/// Create a default performance configuration
pub fn default_performance_config() -> PerformanceConfig {
    PerformanceConfig::default()
}

/// Comprehensive performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub page_cache: PageCacheConfig,
    pub object_cache: ObjectCacheConfig,
    pub query_cache: QueryCacheConfig,
    pub static_files: static_files::StaticFileConfig,
    pub query_logging: QueryLoggerConfig,
    pub minification: MinificationConfig,
    pub image_optimization: ImageOptimizerConfig,
    pub cdn: CdnConfig,
    pub service_worker: ServiceWorkerConfig,
    pub ssr: SsrConfig,
    pub isr: IsrConfig,
    pub edge_cache: EdgeCacheConfig,
    pub connection_pool: PoolConfig,
    pub profiling: ProfilingConfig,
    pub session: SessionConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            page_cache: PageCacheConfig::default(),
            object_cache: ObjectCacheConfig::default(),
            query_cache: QueryCacheConfig::default(),
            static_files: static_files::StaticFileConfig::default(),
            query_logging: QueryLoggerConfig::default(),
            minification: MinificationConfig::default(),
            image_optimization: ImageOptimizerConfig::default(),
            cdn: CdnConfig::default(),
            service_worker: ServiceWorkerConfig::default(),
            ssr: SsrConfig::default(),
            isr: IsrConfig::default(),
            edge_cache: EdgeCacheConfig::default(),
            connection_pool: PoolConfig::default(),
            profiling: ProfilingConfig::default(),
            session: SessionConfig::default(),
        }
    }
}

/// Performance optimization manager
pub struct PerformanceManager {
    pub page_cache: PageCache,
    pub object_cache: ObjectCache,
    pub query_cache: QueryCache,
    pub static_files: StaticFileServer,
    pub query_logger: QueryLogger,
    pub minifier: std::sync::Arc<Minifier>,
    pub image_optimizer: std::sync::Arc<ImageOptimizer>,
    pub cdn: CdnManager,
    pub profiler: std::sync::Arc<Profiler>,
}

impl PerformanceManager {
    /// Create a new performance manager with default configuration
    pub fn new(config: PerformanceConfig) -> Self {
        let minifier = std::sync::Arc::new(Minifier::new(config.minification));

        Self {
            page_cache: PageCache::new(config.page_cache),
            object_cache: ObjectCache::memory_only(config.object_cache),
            query_cache: QueryCache::new(config.query_cache),
            static_files: StaticFileServer::new(config.static_files),
            query_logger: QueryLogger::new(config.query_logging),
            minifier: minifier.clone(),
            image_optimizer: std::sync::Arc::new(ImageOptimizer::new(config.image_optimization)),
            cdn: CdnManager::new(config.cdn),
            profiler: std::sync::Arc::new(Profiler::new(config.profiling)),
        }
    }

    /// Clear all caches
    pub fn clear_all_caches(&self) {
        self.page_cache.clear();
        self.object_cache.clear_local();
        self.query_cache.clear();
    }

    /// Get combined statistics
    pub fn get_stats(&self) -> CombinedStats {
        CombinedStats {
            page_cache: self.page_cache.stats(),
            object_cache: self.object_cache.stats(),
            query_cache: self.query_cache.stats(),
            profiling: self.profiler.get_metrics(),
        }
    }
}

/// Combined statistics from all subsystems
#[derive(Debug, Clone)]
pub struct CombinedStats {
    pub page_cache: page_cache::CacheStats,
    pub object_cache: object_cache::ObjectCacheStats,
    pub query_cache: query_cache::QueryCacheStats,
    pub profiling: PerformanceMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_performance_config();
        assert!(config.page_cache.max_entries > 0);
        assert!(config.object_cache.local_max_capacity > 0);
    }

    #[test]
    fn test_performance_manager() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);

        let stats = manager.get_stats();
        assert_eq!(stats.page_cache.hits, 0);
    }
}
