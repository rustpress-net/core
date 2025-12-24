//! CDN Integration Support
//!
//! Provides integration with Content Delivery Networks for asset distribution.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// CDN errors
#[derive(Debug, Error)]
pub enum CdnError {
    #[error("CDN configuration error: {0}")]
    ConfigError(String),

    #[error("CDN request failed: {0}")]
    RequestFailed(String),

    #[error("CDN purge failed: {0}")]
    PurgeFailed(String),

    #[error("Unsupported CDN provider: {0}")]
    UnsupportedProvider(String),
}

/// Supported CDN providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CdnProvider {
    /// Cloudflare CDN
    Cloudflare,
    /// AWS CloudFront
    CloudFront,
    /// Fastly CDN
    Fastly,
    /// Bunny CDN
    BunnyCdn,
    /// KeyCDN
    KeyCdn,
    /// Custom/Self-hosted CDN
    Custom,
}

/// CDN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    /// CDN provider
    pub provider: CdnProvider,
    /// CDN base URL
    pub base_url: String,
    /// CDN zone/distribution ID
    pub zone_id: Option<String>,
    /// API key or token
    pub api_key: Option<String>,
    /// API secret (for providers that require it)
    pub api_secret: Option<String>,
    /// Whether CDN is enabled
    pub enabled: bool,
    /// Asset types to serve via CDN
    pub asset_types: Vec<AssetType>,
    /// Custom headers to add to CDN requests
    pub custom_headers: HashMap<String, String>,
    /// Enable CDN for authenticated users
    pub serve_authenticated: bool,
    /// Cache TTL for different asset types (seconds)
    pub ttl_config: HashMap<AssetType, u64>,
}

/// Asset type for CDN routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Image,
    JavaScript,
    Css,
    Font,
    Video,
    Audio,
    Document,
    Other,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" | "svg" | "ico" => Self::Image,
            "js" | "mjs" => Self::JavaScript,
            "css" => Self::Css,
            "woff" | "woff2" | "ttf" | "otf" | "eot" => Self::Font,
            "mp4" | "webm" | "ogg" | "avi" | "mov" => Self::Video,
            "mp3" | "wav" | "flac" | "aac" => Self::Audio,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" => Self::Document,
            _ => Self::Other,
        }
    }

    pub fn from_content_type(content_type: &str) -> Self {
        if content_type.starts_with("image/") {
            Self::Image
        } else if content_type.contains("javascript") {
            Self::JavaScript
        } else if content_type.contains("css") {
            Self::Css
        } else if content_type.starts_with("font/") || content_type.contains("font") {
            Self::Font
        } else if content_type.starts_with("video/") {
            Self::Video
        } else if content_type.starts_with("audio/") {
            Self::Audio
        } else if content_type.contains("pdf") || content_type.contains("document") {
            Self::Document
        } else {
            Self::Other
        }
    }
}

impl Default for CdnConfig {
    fn default() -> Self {
        let mut ttl_config = HashMap::new();
        ttl_config.insert(AssetType::Image, 31536000);      // 1 year
        ttl_config.insert(AssetType::JavaScript, 31536000); // 1 year
        ttl_config.insert(AssetType::Css, 31536000);        // 1 year
        ttl_config.insert(AssetType::Font, 31536000);       // 1 year
        ttl_config.insert(AssetType::Video, 604800);        // 1 week
        ttl_config.insert(AssetType::Audio, 604800);        // 1 week
        ttl_config.insert(AssetType::Document, 86400);      // 1 day
        ttl_config.insert(AssetType::Other, 3600);          // 1 hour

        Self {
            provider: CdnProvider::Custom,
            base_url: String::new(),
            zone_id: None,
            api_key: None,
            api_secret: None,
            enabled: false,
            asset_types: vec![
                AssetType::Image,
                AssetType::JavaScript,
                AssetType::Css,
                AssetType::Font,
            ],
            custom_headers: HashMap::new(),
            serve_authenticated: false,
            ttl_config,
        }
    }
}

/// CDN URL rewriter
pub struct CdnRewriter {
    config: CdnConfig,
    /// Pattern cache for URL matching
    patterns: Vec<(String, AssetType)>,
}

impl CdnRewriter {
    pub fn new(config: CdnConfig) -> Self {
        let patterns = vec![
            ("/wp-content/uploads/".to_string(), AssetType::Image),
            ("/wp-content/themes/".to_string(), AssetType::Css),
            ("/wp-content/plugins/".to_string(), AssetType::JavaScript),
            ("/wp-includes/".to_string(), AssetType::JavaScript),
            ("/static/".to_string(), AssetType::Other),
            ("/assets/".to_string(), AssetType::Other),
        ];

        Self { config, patterns }
    }

    /// Check if a URL should be served via CDN
    pub fn should_use_cdn(&self, url: &str, is_authenticated: bool) -> bool {
        if !self.config.enabled {
            return false;
        }

        if is_authenticated && !self.config.serve_authenticated {
            return false;
        }

        // Determine asset type from URL
        let asset_type = self.get_asset_type(url);
        self.config.asset_types.contains(&asset_type)
    }

    /// Get asset type from URL
    fn get_asset_type(&self, url: &str) -> AssetType {
        // Extract extension from URL
        let path = url.split('?').next().unwrap_or(url);
        let ext = path.rsplit('.').next().unwrap_or("");
        AssetType::from_extension(ext)
    }

    /// Rewrite URL to CDN URL
    pub fn rewrite_url(&self, url: &str, is_authenticated: bool) -> String {
        if !self.should_use_cdn(url, is_authenticated) {
            return url.to_string();
        }

        // Handle absolute URLs
        if url.starts_with("http://") || url.starts_with("https://") {
            // Extract path from absolute URL
            if let Some(pos) = url.find("://") {
                if let Some(path_start) = url[pos + 3..].find('/') {
                    let path = &url[pos + 3 + path_start..];
                    return format!("{}{}", self.config.base_url.trim_end_matches('/'), path);
                }
            }
        }

        // Handle relative URLs
        format!("{}{}", self.config.base_url.trim_end_matches('/'), url)
    }

    /// Rewrite all URLs in HTML content
    pub fn rewrite_html(&self, html: &str, is_authenticated: bool) -> String {
        if !self.config.enabled {
            return html.to_string();
        }

        let mut result = html.to_string();

        // Rewrite src attributes
        let src_re = regex::Regex::new(r#"src=["']([^"']+)["']"#).unwrap();
        result = src_re.replace_all(&result, |caps: &regex::Captures| {
            let url = &caps[1];
            let new_url = self.rewrite_url(url, is_authenticated);
            format!("src=\"{}\"", new_url)
        }).to_string();

        // Rewrite href attributes for stylesheets
        let href_re = regex::Regex::new(r#"href=["']([^"']+\.css[^"']*)["']"#).unwrap();
        result = href_re.replace_all(&result, |caps: &regex::Captures| {
            let url = &caps[1];
            let new_url = self.rewrite_url(url, is_authenticated);
            format!("href=\"{}\"", new_url)
        }).to_string();

        // Rewrite srcset attributes
        let srcset_re = regex::Regex::new(r#"srcset=["']([^"']+)["']"#).unwrap();
        result = srcset_re.replace_all(&result, |caps: &regex::Captures| {
            let srcset = &caps[1];
            let new_srcset: String = srcset
                .split(',')
                .map(|part| {
                    let trimmed = part.trim();
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.is_empty() {
                        return trimmed.to_string();
                    }
                    let url = self.rewrite_url(parts[0], is_authenticated);
                    if parts.len() > 1 {
                        format!("{} {}", url, parts[1..].join(" "))
                    } else {
                        url
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("srcset=\"{}\"", new_srcset)
        }).to_string();

        result
    }

    /// Get TTL for an asset type
    pub fn get_ttl(&self, asset_type: AssetType) -> u64 {
        *self.config.ttl_config.get(&asset_type).unwrap_or(&3600)
    }

    /// Generate Cache-Control header value for an asset
    pub fn get_cache_control(&self, asset_type: AssetType) -> String {
        let ttl = self.get_ttl(asset_type);
        format!("public, max-age={}, immutable", ttl)
    }
}

/// CDN cache purger
pub struct CdnPurger {
    config: CdnConfig,
    /// Pending purge requests
    pending_purges: Arc<RwLock<Vec<PurgeRequest>>>,
}

#[derive(Debug, Clone)]
pub struct PurgeRequest {
    /// URLs or paths to purge
    pub targets: Vec<String>,
    /// Purge type
    pub purge_type: PurgeType,
    /// Request timestamp
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PurgeType {
    /// Purge specific URLs
    Urls,
    /// Purge by cache tag
    Tags,
    /// Purge by path prefix
    Prefix,
    /// Purge entire cache
    Everything,
}

impl CdnPurger {
    pub fn new(config: CdnConfig) -> Self {
        Self {
            config,
            pending_purges: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Purge specific URLs from CDN cache
    pub async fn purge_urls(&self, urls: Vec<String>) -> Result<(), CdnError> {
        if !self.config.enabled {
            return Ok(());
        }

        let request = PurgeRequest {
            targets: urls,
            purge_type: PurgeType::Urls,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.execute_purge(request).await
    }

    /// Purge by cache tags
    pub async fn purge_tags(&self, tags: Vec<String>) -> Result<(), CdnError> {
        if !self.config.enabled {
            return Ok(());
        }

        let request = PurgeRequest {
            targets: tags,
            purge_type: PurgeType::Tags,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.execute_purge(request).await
    }

    /// Purge by path prefix
    pub async fn purge_prefix(&self, prefix: &str) -> Result<(), CdnError> {
        if !self.config.enabled {
            return Ok(());
        }

        let request = PurgeRequest {
            targets: vec![prefix.to_string()],
            purge_type: PurgeType::Prefix,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.execute_purge(request).await
    }

    /// Purge entire CDN cache
    pub async fn purge_all(&self) -> Result<(), CdnError> {
        if !self.config.enabled {
            return Ok(());
        }

        let request = PurgeRequest {
            targets: vec![],
            purge_type: PurgeType::Everything,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.execute_purge(request).await
    }

    async fn execute_purge(&self, request: PurgeRequest) -> Result<(), CdnError> {
        match self.config.provider {
            CdnProvider::Cloudflare => self.purge_cloudflare(&request).await,
            CdnProvider::CloudFront => self.purge_cloudfront(&request).await,
            CdnProvider::Fastly => self.purge_fastly(&request).await,
            CdnProvider::BunnyCdn => self.purge_bunny(&request).await,
            CdnProvider::KeyCdn => self.purge_keycdn(&request).await,
            CdnProvider::Custom => self.purge_custom(&request).await,
        }
    }

    async fn purge_cloudflare(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        let zone_id = self.config.zone_id.as_ref()
            .ok_or_else(|| CdnError::ConfigError("Cloudflare zone_id is required".to_string()))?;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| CdnError::ConfigError("Cloudflare api_key is required".to_string()))?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            zone_id
        );

        let body = match request.purge_type {
            PurgeType::Everything => serde_json::json!({"purge_everything": true}),
            PurgeType::Urls => serde_json::json!({"files": request.targets}),
            PurgeType::Tags => serde_json::json!({"tags": request.targets}),
            PurgeType::Prefix => serde_json::json!({"prefixes": request.targets}),
        };

        // In production, this would use reqwest or similar
        // For now, we log the purge request
        tracing::info!(
            "Cloudflare purge request: URL={}, body={:?}",
            url,
            body
        );

        Ok(())
    }

    async fn purge_cloudfront(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        let distribution_id = self.config.zone_id.as_ref()
            .ok_or_else(|| CdnError::ConfigError("CloudFront distribution_id is required".to_string()))?;

        let paths: Vec<String> = match request.purge_type {
            PurgeType::Everything => vec!["/*".to_string()],
            PurgeType::Urls | PurgeType::Prefix => request.targets.clone(),
            PurgeType::Tags => {
                return Err(CdnError::ConfigError(
                    "CloudFront does not support tag-based purging".to_string()
                ));
            }
        };

        tracing::info!(
            "CloudFront invalidation request: distribution={}, paths={:?}",
            distribution_id,
            paths
        );

        Ok(())
    }

    async fn purge_fastly(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        let service_id = self.config.zone_id.as_ref()
            .ok_or_else(|| CdnError::ConfigError("Fastly service_id is required".to_string()))?;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| CdnError::ConfigError("Fastly api_key is required".to_string()))?;

        match request.purge_type {
            PurgeType::Everything => {
                tracing::info!("Fastly purge all: service={}", service_id);
            }
            PurgeType::Tags => {
                for tag in &request.targets {
                    tracing::info!("Fastly purge tag: service={}, tag={}", service_id, tag);
                }
            }
            PurgeType::Urls => {
                for url in &request.targets {
                    tracing::info!("Fastly purge URL: {}", url);
                }
            }
            PurgeType::Prefix => {
                tracing::info!("Fastly purge prefix: {:?}", request.targets);
            }
        }

        Ok(())
    }

    async fn purge_bunny(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| CdnError::ConfigError("BunnyCDN api_key is required".to_string()))?;

        tracing::info!("BunnyCDN purge request: {:?}", request.targets);
        Ok(())
    }

    async fn purge_keycdn(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        let zone_id = self.config.zone_id.as_ref()
            .ok_or_else(|| CdnError::ConfigError("KeyCDN zone_id is required".to_string()))?;

        tracing::info!("KeyCDN purge request: zone={}, targets={:?}", zone_id, request.targets);
        Ok(())
    }

    async fn purge_custom(&self, request: &PurgeRequest) -> Result<(), CdnError> {
        tracing::info!("Custom CDN purge request: {:?}", request.targets);
        Ok(())
    }

    /// Queue a purge request for batch processing
    pub fn queue_purge(&self, request: PurgeRequest) {
        self.pending_purges.write().push(request);
    }

    /// Process all pending purge requests
    pub async fn process_pending(&self) -> Result<usize, CdnError> {
        let requests: Vec<PurgeRequest> = {
            let mut pending = self.pending_purges.write();
            std::mem::take(&mut *pending)
        };

        let count = requests.len();

        for request in requests {
            self.execute_purge(request).await?;
        }

        Ok(count)
    }
}

/// CDN health checker
pub struct CdnHealthChecker {
    config: CdnConfig,
    last_check: Arc<RwLock<Option<CdnHealthStatus>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnHealthStatus {
    pub healthy: bool,
    pub latency_ms: u64,
    pub timestamp: i64,
    pub error: Option<String>,
}

impl CdnHealthChecker {
    pub fn new(config: CdnConfig) -> Self {
        Self {
            config,
            last_check: Arc::new(RwLock::new(None)),
        }
    }

    /// Check CDN health
    pub async fn check(&self) -> CdnHealthStatus {
        let start = std::time::Instant::now();

        // In production, this would make actual HTTP requests to the CDN
        // For now, we return a simulated healthy status
        let status = CdnHealthStatus {
            healthy: self.config.enabled,
            latency_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now().timestamp(),
            error: None,
        };

        *self.last_check.write() = Some(status.clone());
        status
    }

    /// Get last health check result
    pub fn last_status(&self) -> Option<CdnHealthStatus> {
        self.last_check.read().clone()
    }
}

/// CDN integration manager
pub struct CdnManager {
    pub rewriter: CdnRewriter,
    pub purger: CdnPurger,
    pub health_checker: CdnHealthChecker,
}

impl CdnManager {
    pub fn new(config: CdnConfig) -> Self {
        Self {
            rewriter: CdnRewriter::new(config.clone()),
            purger: CdnPurger::new(config.clone()),
            health_checker: CdnHealthChecker::new(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_detection() {
        assert_eq!(AssetType::from_extension("jpg"), AssetType::Image);
        assert_eq!(AssetType::from_extension("js"), AssetType::JavaScript);
        assert_eq!(AssetType::from_extension("css"), AssetType::Css);
        assert_eq!(AssetType::from_extension("woff2"), AssetType::Font);
        assert_eq!(AssetType::from_extension("mp4"), AssetType::Video);
    }

    #[test]
    fn test_url_rewriting() {
        let config = CdnConfig {
            enabled: true,
            base_url: "https://cdn.example.com".to_string(),
            asset_types: vec![AssetType::Image, AssetType::Css, AssetType::JavaScript],
            ..Default::default()
        };

        let rewriter = CdnRewriter::new(config);

        let url = "/wp-content/uploads/image.jpg";
        let rewritten = rewriter.rewrite_url(url, false);
        assert!(rewritten.starts_with("https://cdn.example.com"));
        assert!(rewritten.ends_with("/wp-content/uploads/image.jpg"));
    }

    #[test]
    fn test_html_rewriting() {
        let config = CdnConfig {
            enabled: true,
            base_url: "https://cdn.example.com".to_string(),
            asset_types: vec![AssetType::Image, AssetType::Css, AssetType::JavaScript],
            ..Default::default()
        };

        let rewriter = CdnRewriter::new(config);

        let html = r#"<img src="/images/test.jpg"><link href="/style.css">"#;
        let rewritten = rewriter.rewrite_html(html, false);

        assert!(rewritten.contains("https://cdn.example.com/images/test.jpg"));
        assert!(rewritten.contains("https://cdn.example.com/style.css"));
    }
}
