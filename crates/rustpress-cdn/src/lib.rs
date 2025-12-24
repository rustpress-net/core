//! RustPress CDN Integration
//!
//! This module provides CDN integration and auto-configuration for RustPress.
//! Supports Cloudflare, BunnyCDN, and other major CDN providers.
//!
//! # Features
//!
//! - Auto-configuration of CDN settings
//! - Cache purging and invalidation
//! - Asset optimization
//! - SSL/TLS management
//! - DNS configuration
//!
//! # Example
//!
//! ```rust,ignore
//! use rustpress_cdn::{CdnManager, CloudflareConfig};
//!
//! let config = CloudflareConfig {
//!     api_token: "your-api-token".to_string(),
//!     zone_id: "your-zone-id".to_string(),
//!     ..Default::default()
//! };
//!
//! let cdn = CdnManager::cloudflare(config);
//! cdn.auto_configure().await?;
//! ```

pub mod cloudflare;
pub mod bunnycdn;
pub mod manager;

pub use cloudflare::{CloudflareClient, CloudflareConfig};
pub use bunnycdn::{BunnyCdnClient, BunnyCdnConfig};
pub use manager::{CdnManager, CdnProvider};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CDN error types
#[derive(Debug, thiserror::Error)]
pub enum CdnError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// Result type for CDN operations
pub type Result<T> = std::result::Result<T, CdnError>;

/// CDN provider trait
#[async_trait]
pub trait CdnClient: Send + Sync {
    /// Get provider name
    fn provider_name(&self) -> &str;

    /// Auto-configure CDN for RustPress
    async fn auto_configure(&self, domain: &str) -> Result<CdnConfiguration>;

    /// Purge cache for specific URLs
    async fn purge_urls(&self, urls: &[String]) -> Result<PurgeResult>;

    /// Purge entire cache
    async fn purge_all(&self) -> Result<PurgeResult>;

    /// Purge by cache tags
    async fn purge_tags(&self, tags: &[String]) -> Result<PurgeResult>;

    /// Get cache statistics
    async fn get_stats(&self) -> Result<CdnStats>;

    /// Configure page rules
    async fn configure_rules(&self, rules: Vec<CacheRule>) -> Result<()>;

    /// Check if CDN is properly configured
    async fn health_check(&self) -> Result<bool>;
}

/// CDN configuration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfiguration {
    /// CDN provider name
    pub provider: String,

    /// CDN domain/endpoint
    pub cdn_domain: String,

    /// Origin domain
    pub origin_domain: String,

    /// SSL/TLS status
    pub ssl_enabled: bool,

    /// Cache settings applied
    pub cache_settings: CacheSettings,

    /// DNS records configured
    pub dns_records: Vec<DnsRecord>,

    /// Additional settings
    pub extra_settings: HashMap<String, serde_json::Value>,
}

/// Cache settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Default TTL in seconds
    pub default_ttl: u64,

    /// Browser cache TTL
    pub browser_ttl: u64,

    /// Cache level (aggressive, standard, basic)
    pub cache_level: String,

    /// Minify settings
    pub minify: MinifySettings,

    /// Cache static assets
    pub cache_static: bool,

    /// Cache API responses
    pub cache_api: bool,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            default_ttl: 86400,      // 1 day
            browser_ttl: 14400,       // 4 hours
            cache_level: "standard".to_string(),
            minify: MinifySettings::default(),
            cache_static: true,
            cache_api: false,
        }
    }
}

/// Minification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinifySettings {
    pub html: bool,
    pub css: bool,
    pub js: bool,
}

impl Default for MinifySettings {
    fn default() -> Self {
        Self {
            html: true,
            css: true,
            js: true,
        }
    }
}

/// DNS record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

/// Cache rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheRule {
    /// Rule name
    pub name: String,

    /// URL pattern (glob or regex)
    pub pattern: String,

    /// Cache TTL in seconds (0 = bypass)
    pub ttl: u64,

    /// Cache level override
    pub cache_level: Option<String>,

    /// Edge TTL override
    pub edge_ttl: Option<u64>,

    /// Browser TTL override
    pub browser_ttl: Option<u64>,

    /// Priority (lower = higher priority)
    pub priority: i32,
}

/// Purge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeResult {
    pub success: bool,
    pub purged_count: u64,
    pub message: String,
}

/// CDN statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnStats {
    /// Total requests
    pub total_requests: u64,

    /// Cached requests
    pub cached_requests: u64,

    /// Cache hit ratio (0-100)
    pub cache_hit_ratio: f64,

    /// Bandwidth saved in bytes
    pub bandwidth_saved: u64,

    /// Total bandwidth in bytes
    pub total_bandwidth: u64,

    /// Threats blocked
    pub threats_blocked: u64,

    /// Time period (start)
    pub period_start: chrono::DateTime<chrono::Utc>,

    /// Time period (end)
    pub period_end: chrono::DateTime<chrono::Utc>,
}

/// Default cache rules for RustPress
pub fn default_cache_rules() -> Vec<CacheRule> {
    vec![
        // Static assets - long cache
        CacheRule {
            name: "Static Assets".to_string(),
            pattern: "*.{css,js,woff,woff2,ttf,eot,ico,svg}".to_string(),
            ttl: 31536000, // 1 year
            cache_level: Some("aggressive".to_string()),
            edge_ttl: Some(2592000), // 30 days
            browser_ttl: Some(604800), // 7 days
            priority: 1,
        },
        // Images - long cache
        CacheRule {
            name: "Images".to_string(),
            pattern: "*.{jpg,jpeg,png,gif,webp,avif}".to_string(),
            ttl: 2592000, // 30 days
            cache_level: Some("aggressive".to_string()),
            edge_ttl: Some(604800), // 7 days
            browser_ttl: Some(86400), // 1 day
            priority: 2,
        },
        // Media uploads
        CacheRule {
            name: "Media Uploads".to_string(),
            pattern: "/uploads/*".to_string(),
            ttl: 604800, // 7 days
            cache_level: Some("standard".to_string()),
            edge_ttl: Some(86400), // 1 day
            browser_ttl: Some(3600), // 1 hour
            priority: 3,
        },
        // API responses - short cache
        CacheRule {
            name: "API Cache".to_string(),
            pattern: "/api/v1/public/*".to_string(),
            ttl: 300, // 5 minutes
            cache_level: Some("standard".to_string()),
            edge_ttl: Some(60),
            browser_ttl: Some(0),
            priority: 4,
        },
        // Admin - no cache
        CacheRule {
            name: "Admin No Cache".to_string(),
            pattern: "/admin/*".to_string(),
            ttl: 0,
            cache_level: Some("bypass".to_string()),
            edge_ttl: None,
            browser_ttl: None,
            priority: 0,
        },
        // Auth endpoints - no cache
        CacheRule {
            name: "Auth No Cache".to_string(),
            pattern: "/api/v1/auth/*".to_string(),
            ttl: 0,
            cache_level: Some("bypass".to_string()),
            edge_ttl: None,
            browser_ttl: None,
            priority: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cache_settings() {
        let settings = CacheSettings::default();
        assert_eq!(settings.default_ttl, 86400);
        assert!(settings.cache_static);
    }

    #[test]
    fn test_default_rules() {
        let rules = default_cache_rules();
        assert!(!rules.is_empty());

        // Check admin bypass rule exists
        let admin_rule = rules.iter().find(|r| r.pattern.contains("admin"));
        assert!(admin_rule.is_some());
        assert_eq!(admin_rule.unwrap().ttl, 0);
    }
}
