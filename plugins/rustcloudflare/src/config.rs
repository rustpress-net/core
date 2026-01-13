//! Configuration management for RustCloudflare

use crate::error::{CloudflareError, CloudflareResult};
use serde::{Deserialize, Serialize};

/// Main configuration for Cloudflare integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    // Credentials
    pub api_token: String,
    pub account_id: String,
    pub zone_id: String,
    pub email: Option<String>,

    // CDN Settings
    #[serde(default = "default_true")]
    pub cdn_enabled: bool,
    #[serde(default = "default_auto_minify")]
    pub auto_minify: Vec<String>,
    #[serde(default = "default_true")]
    pub brotli_compression: bool,
    #[serde(default = "default_true")]
    pub early_hints: bool,
    #[serde(default)]
    pub rocket_loader: bool,

    // Cache Settings
    #[serde(default = "default_cache_level")]
    pub cache_level: CacheLevel,
    #[serde(default = "default_browser_cache_ttl")]
    pub browser_cache_ttl: u32,
    #[serde(default = "default_edge_cache_ttl")]
    pub edge_cache_ttl: u32,
    #[serde(default)]
    pub cache_by_device_type: bool,
    #[serde(default = "default_true")]
    pub cache_deception_armor: bool,

    // Security Settings
    #[serde(default = "default_security_level")]
    pub security_level: SecurityLevel,
    #[serde(default = "default_true")]
    pub waf_enabled: bool,
    #[serde(default = "default_true")]
    pub bot_management: bool,
    #[serde(default = "default_true")]
    pub ddos_protection: bool,
    #[serde(default = "default_challenge_passage")]
    pub challenge_passage: u32,
    #[serde(default = "default_true")]
    pub browser_integrity_check: bool,

    // SSL/TLS Settings
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: SslMode,
    #[serde(default = "default_true")]
    pub always_use_https: bool,
    #[serde(default = "default_min_tls")]
    pub min_tls_version: String,
    #[serde(default = "default_true")]
    pub automatic_https_rewrites: bool,
    #[serde(default = "default_true")]
    pub opportunistic_encryption: bool,

    // Performance Settings
    #[serde(default)]
    pub argo_smart_routing: bool,
    #[serde(default)]
    pub argo_tiered_caching: bool,
    #[serde(default = "default_true")]
    pub http2: bool,
    #[serde(default = "default_true")]
    pub http3: bool,
    #[serde(default = "default_true")]
    pub zero_rtt: bool,
    #[serde(default = "default_true")]
    pub websockets: bool,

    // Image Optimization
    #[serde(default = "default_polish")]
    pub polish: PolishMode,
    #[serde(default = "default_true")]
    pub webp: bool,
    #[serde(default = "default_true")]
    pub mirage: bool,
    #[serde(default = "default_true")]
    pub image_resizing: bool,

    // Workers Settings
    #[serde(default = "default_true")]
    pub workers_enabled: bool,
    #[serde(default = "default_true")]
    pub workers_kv_enabled: bool,
    pub workers_kv_namespace: Option<String>,

    // R2 Storage Settings
    #[serde(default)]
    pub r2_enabled: bool,
    pub r2_bucket: Option<String>,
    pub r2_access_key_id: Option<String>,
    pub r2_secret_access_key: Option<String>,
    pub r2_public_url: Option<String>,

    // D1 Database Settings
    #[serde(default)]
    pub d1_enabled: bool,
    pub d1_database_id: Option<String>,

    // Stream Settings
    #[serde(default)]
    pub stream_enabled: bool,

    // Analytics
    #[serde(default = "default_true")]
    pub analytics_enabled: bool,
    pub analytics_token: Option<String>,

    // DNS Settings
    #[serde(default = "default_true")]
    pub dns_management: bool,
    #[serde(default)]
    pub auto_dns_sync: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheLevel {
    Bypass,
    Basic,
    Simplified,
    #[default]
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Off,
    EssentiallyOff,
    Low,
    #[default]
    Medium,
    High,
    UnderAttack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Off,
    Flexible,
    Full,
    #[default]
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PolishMode {
    Off,
    Lossless,
    #[default]
    Lossy,
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_auto_minify() -> Vec<String> {
    vec![
        "javascript".to_string(),
        "css".to_string(),
        "html".to_string(),
    ]
}

fn default_cache_level() -> CacheLevel {
    CacheLevel::Aggressive
}

fn default_browser_cache_ttl() -> u32 {
    14400 // 4 hours
}

fn default_edge_cache_ttl() -> u32 {
    86400 // 24 hours
}

fn default_security_level() -> SecurityLevel {
    SecurityLevel::Medium
}

fn default_challenge_passage() -> u32 {
    1800 // 30 minutes
}

fn default_ssl_mode() -> SslMode {
    SslMode::Strict
}

fn default_min_tls() -> String {
    "1.2".to_string()
}

fn default_polish() -> PolishMode {
    PolishMode::Lossy
}

impl CloudflareConfig {
    /// Create configuration from JSON settings value
    pub fn from_settings(settings: serde_json::Value) -> CloudflareResult<Self> {
        // Validate required fields exist
        let _api_token = settings
            .get("api_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CloudflareError::MissingConfig("api_token".to_string()))?;

        let _account_id = settings
            .get("account_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CloudflareError::MissingConfig("account_id".to_string()))?;

        let _zone_id = settings
            .get("zone_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CloudflareError::MissingConfig("zone_id".to_string()))?;

        // Parse full config with defaults
        let config: CloudflareConfig = serde_json::from_value(settings)
            .map_err(|e| CloudflareError::InvalidConfig(e.to_string()))?;

        Ok(config)
    }

    /// Check if configuration is available (either from env or database)
    pub fn is_configured() -> bool {
        std::env::var("CLOUDFLARE_API_TOKEN").is_ok()
            || std::env::var("CLOUDFLARE_ACCOUNT_ID").is_ok()
    }

    /// Create configuration from environment variables
    pub fn from_env() -> CloudflareResult<Self> {
        Ok(Self {
            api_token: std::env::var("CLOUDFLARE_API_TOKEN")
                .map_err(|_| CloudflareError::MissingConfig("CLOUDFLARE_API_TOKEN".to_string()))?,
            account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID")
                .map_err(|_| CloudflareError::MissingConfig("CLOUDFLARE_ACCOUNT_ID".to_string()))?,
            zone_id: std::env::var("CLOUDFLARE_ZONE_ID")
                .map_err(|_| CloudflareError::MissingConfig("CLOUDFLARE_ZONE_ID".to_string()))?,
            email: std::env::var("CLOUDFLARE_EMAIL").ok(),
            ..Default::default()
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> CloudflareResult<()> {
        if self.api_token.is_empty() {
            return Err(CloudflareError::InvalidConfig(
                "API token cannot be empty".to_string(),
            ));
        }

        if self.account_id.is_empty() {
            return Err(CloudflareError::InvalidConfig(
                "Account ID cannot be empty".to_string(),
            ));
        }

        if self.zone_id.is_empty() {
            return Err(CloudflareError::InvalidConfig(
                "Zone ID cannot be empty".to_string(),
            ));
        }

        // Validate R2 config if enabled
        if self.r2_enabled {
            if self.r2_bucket.is_none() {
                return Err(CloudflareError::InvalidConfig(
                    "R2 bucket name required when R2 is enabled".to_string(),
                ));
            }
        }

        // Validate D1 config if enabled
        if self.d1_enabled {
            if self.d1_database_id.is_none() {
                return Err(CloudflareError::InvalidConfig(
                    "D1 database ID required when D1 is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the Cloudflare API base URL
    pub fn api_base_url(&self) -> &str {
        "https://api.cloudflare.com/client/v4"
    }

    /// Get the R2 endpoint URL
    pub fn r2_endpoint(&self) -> String {
        format!(
            "https://{}.r2.cloudflarestorage.com",
            self.account_id
        )
    }
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            account_id: String::new(),
            zone_id: String::new(),
            email: None,
            cdn_enabled: true,
            auto_minify: default_auto_minify(),
            brotli_compression: true,
            early_hints: true,
            rocket_loader: false,
            cache_level: CacheLevel::default(),
            browser_cache_ttl: default_browser_cache_ttl(),
            edge_cache_ttl: default_edge_cache_ttl(),
            cache_by_device_type: false,
            cache_deception_armor: true,
            security_level: SecurityLevel::default(),
            waf_enabled: true,
            bot_management: true,
            ddos_protection: true,
            challenge_passage: default_challenge_passage(),
            browser_integrity_check: true,
            ssl_mode: SslMode::default(),
            always_use_https: true,
            min_tls_version: default_min_tls(),
            automatic_https_rewrites: true,
            opportunistic_encryption: true,
            argo_smart_routing: false,
            argo_tiered_caching: false,
            http2: true,
            http3: true,
            zero_rtt: true,
            websockets: true,
            polish: PolishMode::default(),
            webp: true,
            mirage: true,
            image_resizing: true,
            workers_enabled: true,
            workers_kv_enabled: true,
            workers_kv_namespace: None,
            r2_enabled: false,
            r2_bucket: None,
            r2_access_key_id: None,
            r2_secret_access_key: None,
            r2_public_url: None,
            d1_enabled: false,
            d1_database_id: None,
            stream_enabled: false,
            analytics_enabled: true,
            analytics_token: None,
            dns_management: true,
            auto_dns_sync: false,
        }
    }
}
