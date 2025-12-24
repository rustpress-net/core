//! Cloudflare CDN Integration
//!
//! Full integration with Cloudflare's CDN, DNS, and security services.
//!
//! # Features
//!
//! - Auto-configuration of zone settings
//! - Cache purging (by URL, tag, or all)
//! - Page rules management
//! - DNS management
//! - SSL/TLS configuration
//! - Firewall rules
//! - Analytics and statistics

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{
    CacheRule, CacheSettings, CdnClient, CdnConfiguration, CdnError, CdnStats,
    DnsRecord, PurgeResult, Result, default_cache_rules,
};

/// Cloudflare configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// API Token (recommended) or API Key
    pub api_token: String,

    /// Zone ID
    pub zone_id: String,

    /// Account ID (optional)
    #[serde(default)]
    pub account_id: Option<String>,

    /// API Email (only needed with API Key)
    #[serde(default)]
    pub email: Option<String>,

    /// Use API Token (true) or API Key (false)
    #[serde(default = "default_use_token")]
    pub use_token: bool,

    /// Base API URL
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

fn default_use_token() -> bool {
    true
}

fn default_api_url() -> String {
    "https://api.cloudflare.com/client/v4".to_string()
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            zone_id: String::new(),
            account_id: None,
            email: None,
            use_token: true,
            api_url: default_api_url(),
        }
    }
}

/// Cloudflare API client
pub struct CloudflareClient {
    config: CloudflareConfig,
    client: reqwest::Client,
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(config: CloudflareConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Build authorization headers
    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if self.config.use_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.config.api_token))
                    .unwrap(),
            );
        } else {
            headers.insert(
                "X-Auth-Email",
                HeaderValue::from_str(self.config.email.as_deref().unwrap_or(""))
                    .unwrap(),
            );
            headers.insert(
                "X-Auth-Key",
                HeaderValue::from_str(&self.config.api_token).unwrap(),
            );
        }

        headers
    }

    /// Make API request
    async fn api_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.api_url, endpoint);

        let mut request = self.client.request(method, &url).headers(self.auth_headers());

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CdnError::Network(e.to_string()))?;

        let _status = response.status();
        let body: CloudflareResponse<T> = response
            .json()
            .await
            .map_err(|e| CdnError::Api(e.to_string()))?;

        if !body.success {
            let errors: Vec<String> = body
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect();
            return Err(CdnError::Api(errors.join(", ")));
        }

        body.result.ok_or_else(|| CdnError::Api("Empty response".to_string()))
    }

    /// Get zone details
    pub async fn get_zone(&self) -> Result<ZoneDetails> {
        self.api_request(
            reqwest::Method::GET,
            &format!("/zones/{}", self.config.zone_id),
            None,
        )
        .await
    }

    /// Update zone settings
    pub async fn update_zone_setting(
        &self,
        setting: &str,
        value: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.api_request(
            reqwest::Method::PATCH,
            &format!("/zones/{}/settings/{}", self.config.zone_id, setting),
            Some(serde_json::json!({ "value": value })),
        )
        .await
    }

    /// Configure SSL/TLS
    pub async fn configure_ssl(&self, mode: &str) -> Result<()> {
        // Set SSL mode (off, flexible, full, strict)
        self.update_zone_setting("ssl", serde_json::json!(mode)).await?;

        // Enable Always Use HTTPS
        self.update_zone_setting("always_use_https", serde_json::json!("on")).await?;

        // Enable Automatic HTTPS Rewrites
        self.update_zone_setting("automatic_https_rewrites", serde_json::json!("on")).await?;

        // Enable TLS 1.3
        self.update_zone_setting("min_tls_version", serde_json::json!("1.2")).await?;

        info!("SSL/TLS configured with mode: {}", mode);
        Ok(())
    }

    /// Configure caching settings
    pub async fn configure_caching(&self, settings: &CacheSettings) -> Result<()> {
        // Set cache level
        let cache_level = match settings.cache_level.as_str() {
            "aggressive" => "aggressive",
            "basic" => "basic",
            _ => "standard",
        };
        self.update_zone_setting("cache_level", serde_json::json!(cache_level)).await?;

        // Set browser cache TTL
        self.update_zone_setting(
            "browser_cache_ttl",
            serde_json::json!(settings.browser_ttl),
        )
        .await?;

        // Configure minification
        self.update_zone_setting(
            "minify",
            serde_json::json!({
                "html": if settings.minify.html { "on" } else { "off" },
                "css": if settings.minify.css { "on" } else { "off" },
                "js": if settings.minify.js { "on" } else { "off" },
            }),
        )
        .await?;

        // Enable Brotli compression
        self.update_zone_setting("brotli", serde_json::json!("on")).await?;

        info!("Caching settings configured");
        Ok(())
    }

    /// Configure security settings
    pub async fn configure_security(&self) -> Result<()> {
        // Security level
        self.update_zone_setting("security_level", serde_json::json!("medium")).await?;

        // Enable WAF
        self.update_zone_setting("waf", serde_json::json!("on")).await?;

        // Enable Bot Fight Mode
        self.update_zone_setting("bot_fight_mode", serde_json::json!("on")).await?;

        // Enable Email Obfuscation
        self.update_zone_setting("email_obfuscation", serde_json::json!("on")).await?;

        // Enable Hotlink Protection
        self.update_zone_setting("hotlink_protection", serde_json::json!("on")).await?;

        info!("Security settings configured");
        Ok(())
    }

    /// Create page rules
    pub async fn create_page_rule(&self, rule: &CacheRule) -> Result<String> {
        let mut actions = vec![];

        if rule.ttl == 0 {
            actions.push(serde_json::json!({
                "id": "cache_level",
                "value": "bypass"
            }));
        } else {
            actions.push(serde_json::json!({
                "id": "cache_level",
                "value": rule.cache_level.as_deref().unwrap_or("standard")
            }));

            if let Some(edge_ttl) = rule.edge_ttl {
                actions.push(serde_json::json!({
                    "id": "edge_cache_ttl",
                    "value": edge_ttl
                }));
            }

            if let Some(browser_ttl) = rule.browser_ttl {
                actions.push(serde_json::json!({
                    "id": "browser_cache_ttl",
                    "value": browser_ttl
                }));
            }
        }

        let response: PageRule = self
            .api_request(
                reqwest::Method::POST,
                &format!("/zones/{}/pagerules", self.config.zone_id),
                Some(serde_json::json!({
                    "targets": [{
                        "target": "url",
                        "constraint": {
                            "operator": "matches",
                            "value": format!("*{}*", rule.pattern)
                        }
                    }],
                    "actions": actions,
                    "priority": rule.priority,
                    "status": "active"
                })),
            )
            .await?;

        Ok(response.id)
    }

    /// Get DNS records
    pub async fn get_dns_records(&self) -> Result<Vec<DnsRecordCf>> {
        self.api_request(
            reqwest::Method::GET,
            &format!("/zones/{}/dns_records", self.config.zone_id),
            None,
        )
        .await
    }

    /// Create DNS record
    pub async fn create_dns_record(&self, record: &DnsRecord) -> Result<DnsRecordCf> {
        self.api_request(
            reqwest::Method::POST,
            &format!("/zones/{}/dns_records", self.config.zone_id),
            Some(serde_json::json!({
                "type": record.record_type,
                "name": record.name,
                "content": record.content,
                "ttl": record.ttl,
                "proxied": record.proxied
            })),
        )
        .await
    }

    /// Get analytics
    pub async fn get_analytics(&self, since: &str, until: &str) -> Result<Analytics> {
        self.api_request(
            reqwest::Method::GET,
            &format!(
                "/zones/{}/analytics/dashboard?since={}&until={}",
                self.config.zone_id, since, until
            ),
            None,
        )
        .await
    }
}

#[async_trait]
impl CdnClient for CloudflareClient {
    fn provider_name(&self) -> &str {
        "cloudflare"
    }

    async fn auto_configure(&self, domain: &str) -> Result<CdnConfiguration> {
        info!("Auto-configuring Cloudflare for domain: {}", domain);

        // Get zone details
        let zone = self.get_zone().await?;

        // Configure SSL
        self.configure_ssl("full").await?;

        // Configure caching
        let cache_settings = CacheSettings::default();
        self.configure_caching(&cache_settings).await?;

        // Configure security
        self.configure_security().await?;

        // Create default page rules
        let rules = default_cache_rules();
        for rule in &rules {
            match self.create_page_rule(rule).await {
                Ok(_) => debug!("Created page rule: {}", rule.name),
                Err(e) => warn!("Failed to create page rule {}: {}", rule.name, e),
            }
        }

        // Get DNS records
        let dns_records: Vec<DnsRecord> = self
            .get_dns_records()
            .await?
            .into_iter()
            .map(|r| DnsRecord {
                record_type: r.record_type,
                name: r.name,
                content: r.content,
                ttl: r.ttl,
                proxied: r.proxied,
            })
            .collect();

        Ok(CdnConfiguration {
            provider: "cloudflare".to_string(),
            cdn_domain: zone.name.clone(),
            origin_domain: domain.to_string(),
            ssl_enabled: true,
            cache_settings,
            dns_records,
            extra_settings: HashMap::new(),
        })
    }

    async fn purge_urls(&self, urls: &[String]) -> Result<PurgeResult> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/zones/{}/purge_cache", self.config.zone_id),
                Some(serde_json::json!({ "files": urls })),
            )
            .await?;

        Ok(PurgeResult {
            success: true,
            purged_count: urls.len() as u64,
            message: format!("Purged {} URLs", urls.len()),
        })
    }

    async fn purge_all(&self) -> Result<PurgeResult> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/zones/{}/purge_cache", self.config.zone_id),
                Some(serde_json::json!({ "purge_everything": true })),
            )
            .await?;

        Ok(PurgeResult {
            success: true,
            purged_count: 0,
            message: "Purged entire cache".to_string(),
        })
    }

    async fn purge_tags(&self, tags: &[String]) -> Result<PurgeResult> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/zones/{}/purge_cache", self.config.zone_id),
                Some(serde_json::json!({ "tags": tags })),
            )
            .await?;

        Ok(PurgeResult {
            success: true,
            purged_count: 0,
            message: format!("Purged cache for tags: {:?}", tags),
        })
    }

    async fn get_stats(&self) -> Result<CdnStats> {
        let analytics = self.get_analytics("-1440", "0").await?;

        let totals = &analytics.totals;
        let total_requests = totals.requests.all;
        let cached_requests = totals.requests.cached;

        Ok(CdnStats {
            total_requests,
            cached_requests,
            cache_hit_ratio: if total_requests > 0 {
                (cached_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
            bandwidth_saved: totals.bandwidth.cached,
            total_bandwidth: totals.bandwidth.all,
            threats_blocked: totals.threats.all,
            period_start: chrono::Utc::now() - chrono::Duration::hours(24),
            period_end: chrono::Utc::now(),
        })
    }

    async fn configure_rules(&self, rules: Vec<CacheRule>) -> Result<()> {
        for rule in rules {
            self.create_page_rule(&rule).await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        match self.get_zone().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

// Cloudflare API response types

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct ZoneDetails {
    pub id: String,
    pub name: String,
    pub status: String,
    pub name_servers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecordCf {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Deserialize)]
pub struct PageRule {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Analytics {
    pub totals: AnalyticsTotals,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsTotals {
    pub requests: RequestStats,
    pub bandwidth: BandwidthStats,
    pub threats: ThreatStats,
}

#[derive(Debug, Deserialize)]
pub struct RequestStats {
    pub all: u64,
    pub cached: u64,
    pub uncached: u64,
}

#[derive(Debug, Deserialize)]
pub struct BandwidthStats {
    pub all: u64,
    pub cached: u64,
    pub uncached: u64,
}

#[derive(Debug, Deserialize)]
pub struct ThreatStats {
    pub all: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_config() {
        let config = CloudflareConfig {
            api_token: "test-token".to_string(),
            zone_id: "test-zone".to_string(),
            ..Default::default()
        };

        let client = CloudflareClient::new(config);
        assert_eq!(client.provider_name(), "cloudflare");
    }
}
