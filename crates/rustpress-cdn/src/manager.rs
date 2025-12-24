//! CDN Manager - Unified interface for all CDN providers
//!
//! Provides a simple, unified API for managing CDN configuration
//! across different providers.

use crate::{
    bunnycdn::{BunnyCdnClient, BunnyCdnConfig},
    cloudflare::{CloudflareClient, CloudflareConfig},
    CacheRule, CdnClient, CdnConfiguration, CdnStats, PurgeResult, Result, CdnError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Supported CDN providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CdnProvider {
    /// Cloudflare
    Cloudflare,
    /// BunnyCDN
    BunnyCdn,
    /// AWS CloudFront
    CloudFront,
    /// Fastly
    Fastly,
    /// None (bypass CDN)
    None,
}

impl Default for CdnProvider {
    fn default() -> Self {
        Self::None
    }
}

/// Universal CDN configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum CdnConfig {
    #[serde(rename = "cloudflare")]
    Cloudflare(CloudflareConfig),

    #[serde(rename = "bunnycdn")]
    BunnyCdn(BunnyCdnConfig),

    #[serde(rename = "none")]
    None,
}

impl Default for CdnConfig {
    fn default() -> Self {
        Self::None
    }
}

/// CDN Manager for unified CDN operations
pub struct CdnManager {
    /// CDN client
    client: Option<Arc<dyn CdnClient>>,

    /// Provider type
    provider: CdnProvider,

    /// Domain being served
    domain: String,

    /// Current configuration
    configuration: Option<CdnConfiguration>,
}

impl CdnManager {
    /// Create a new CDN manager with no provider
    pub fn new() -> Self {
        Self {
            client: None,
            provider: CdnProvider::None,
            domain: String::new(),
            configuration: None,
        }
    }

    /// Create CDN manager with Cloudflare
    pub fn cloudflare(config: CloudflareConfig) -> Self {
        Self {
            client: Some(Arc::new(CloudflareClient::new(config))),
            provider: CdnProvider::Cloudflare,
            domain: String::new(),
            configuration: None,
        }
    }

    /// Create CDN manager with BunnyCDN
    pub fn bunnycdn(config: BunnyCdnConfig) -> Self {
        Self {
            client: Some(Arc::new(BunnyCdnClient::new(config))),
            provider: CdnProvider::BunnyCdn,
            domain: String::new(),
            configuration: None,
        }
    }

    /// Create CDN manager from configuration
    pub fn from_config(config: CdnConfig) -> Self {
        match config {
            CdnConfig::Cloudflare(cfg) => Self::cloudflare(cfg),
            CdnConfig::BunnyCdn(cfg) => Self::bunnycdn(cfg),
            CdnConfig::None => Self::new(),
        }
    }

    /// Create CDN manager from environment variables
    pub fn from_env() -> Result<Self> {
        let provider = std::env::var("CDN_PROVIDER")
            .unwrap_or_else(|_| "none".to_string())
            .to_lowercase();

        match provider.as_str() {
            "cloudflare" => {
                let config = CloudflareConfig {
                    api_token: std::env::var("CLOUDFLARE_API_TOKEN")
                        .map_err(|_| CdnError::Configuration(
                            "CLOUDFLARE_API_TOKEN not set".to_string()
                        ))?,
                    zone_id: std::env::var("CLOUDFLARE_ZONE_ID")
                        .map_err(|_| CdnError::Configuration(
                            "CLOUDFLARE_ZONE_ID not set".to_string()
                        ))?,
                    ..Default::default()
                };
                Ok(Self::cloudflare(config))
            }
            "bunnycdn" | "bunny" => {
                let config = BunnyCdnConfig {
                    api_key: std::env::var("BUNNYCDN_API_KEY")
                        .map_err(|_| CdnError::Configuration(
                            "BUNNYCDN_API_KEY not set".to_string()
                        ))?,
                    pull_zone_id: std::env::var("BUNNYCDN_PULL_ZONE_ID")
                        .ok()
                        .and_then(|s| s.parse().ok()),
                    storage_zone: std::env::var("BUNNYCDN_STORAGE_ZONE").ok(),
                    storage_api_key: std::env::var("BUNNYCDN_STORAGE_API_KEY").ok(),
                    ..Default::default()
                };
                Ok(Self::bunnycdn(config))
            }
            "none" | "" => Ok(Self::new()),
            _ => Err(CdnError::Configuration(format!(
                "Unknown CDN provider: {}",
                provider
            ))),
        }
    }

    /// Set the domain
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    /// Get provider name
    pub fn provider_name(&self) -> &str {
        match self.provider {
            CdnProvider::Cloudflare => "cloudflare",
            CdnProvider::BunnyCdn => "bunnycdn",
            CdnProvider::CloudFront => "cloudfront",
            CdnProvider::Fastly => "fastly",
            CdnProvider::None => "none",
        }
    }

    /// Check if CDN is configured
    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    /// Auto-configure CDN for the domain
    pub async fn auto_configure(&mut self) -> Result<CdnConfiguration> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        if self.domain.is_empty() {
            return Err(CdnError::Configuration("Domain not set".to_string()));
        }

        info!(
            "Auto-configuring {} for domain: {}",
            self.provider_name(),
            self.domain
        );

        let config = client.auto_configure(&self.domain).await?;
        self.configuration = Some(config.clone());

        info!("CDN auto-configuration complete");
        Ok(config)
    }

    /// Purge specific URLs from cache
    pub async fn purge_urls(&self, urls: &[String]) -> Result<PurgeResult> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        client.purge_urls(urls).await
    }

    /// Purge entire cache
    pub async fn purge_all(&self) -> Result<PurgeResult> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        info!("Purging entire CDN cache");
        client.purge_all().await
    }

    /// Purge cache by tags
    pub async fn purge_tags(&self, tags: &[String]) -> Result<PurgeResult> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        client.purge_tags(tags).await
    }

    /// Purge cache for a post
    pub async fn purge_post(&self, post_id: u64, post_slug: &str) -> Result<PurgeResult> {
        let base_url = if self.domain.starts_with("http") {
            self.domain.clone()
        } else {
            format!("https://{}", self.domain)
        };

        let urls = vec![
            format!("{}/", base_url),                          // Home page
            format!("{}/posts/{}", base_url, post_slug),       // Post page
            format!("{}/api/v1/posts/{}", base_url, post_id),  // API endpoint
            format!("{}/sitemap.xml", base_url),               // Sitemap
            format!("{}/feed.xml", base_url),                  // RSS feed
        ];

        self.purge_urls(&urls).await
    }

    /// Get CDN statistics
    pub async fn get_stats(&self) -> Result<CdnStats> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        client.get_stats().await
    }

    /// Configure cache rules
    pub async fn configure_rules(&self, rules: Vec<CacheRule>) -> Result<()> {
        let client = self.client.as_ref().ok_or_else(|| {
            CdnError::Configuration("No CDN provider configured".to_string())
        })?;

        client.configure_rules(rules).await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        if let Some(client) = &self.client {
            client.health_check().await
        } else {
            Ok(true) // No CDN is always "healthy"
        }
    }

    /// Get current configuration
    pub fn get_configuration(&self) -> Option<&CdnConfiguration> {
        self.configuration.as_ref()
    }

    /// Get CDN URL for an asset
    pub fn get_cdn_url(&self, path: &str) -> String {
        if let Some(config) = &self.configuration {
            format!("https://{}/{}", config.cdn_domain, path.trim_start_matches('/'))
        } else if !self.domain.is_empty() {
            format!("https://{}/{}", self.domain, path.trim_start_matches('/'))
        } else {
            path.to_string()
        }
    }
}

impl Default for CdnManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CDN middleware helper for cache headers
pub struct CdnHeaders;

impl CdnHeaders {
    /// Generate cache headers for static assets
    pub fn static_asset() -> Vec<(&'static str, String)> {
        vec![
            ("Cache-Control", "public, max-age=31536000, immutable".to_string()),
            ("CDN-Cache-Control", "max-age=31536000".to_string()),
        ]
    }

    /// Generate cache headers for dynamic content
    pub fn dynamic_content(max_age: u64) -> Vec<(&'static str, String)> {
        vec![
            ("Cache-Control", format!("public, max-age={}, s-maxage={}", max_age / 2, max_age)),
            ("CDN-Cache-Control", format!("max-age={}", max_age)),
            ("Vary", "Accept-Encoding".to_string()),
        ]
    }

    /// Generate no-cache headers
    pub fn no_cache() -> Vec<(&'static str, String)> {
        vec![
            ("Cache-Control", "private, no-cache, no-store, must-revalidate".to_string()),
            ("Pragma", "no-cache".to_string()),
            ("Expires", "0".to_string()),
        ]
    }

    /// Generate cache tag header (for Cloudflare Enterprise)
    pub fn cache_tag(tags: &[&str]) -> (&'static str, String) {
        ("Cache-Tag", tags.join(","))
    }

    /// Generate surrogate key header (for Fastly)
    pub fn surrogate_key(keys: &[&str]) -> (&'static str, String) {
        ("Surrogate-Key", keys.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdn_manager_default() {
        let manager = CdnManager::new();
        assert!(!manager.is_configured());
        assert_eq!(manager.provider_name(), "none");
    }

    #[test]
    fn test_cdn_headers() {
        let headers = CdnHeaders::static_asset();
        assert!(!headers.is_empty());

        let no_cache = CdnHeaders::no_cache();
        assert!(no_cache.iter().any(|(k, v)| *k == "Cache-Control" && v.contains("no-cache")));
    }

    #[test]
    fn test_get_cdn_url() {
        let manager = CdnManager::new().with_domain("example.com");
        let url = manager.get_cdn_url("/images/logo.png");
        assert_eq!(url, "https://example.com/images/logo.png");
    }
}
