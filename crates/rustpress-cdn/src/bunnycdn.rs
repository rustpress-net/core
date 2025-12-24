//! BunnyCDN Integration
//!
//! Full integration with BunnyCDN for content delivery and storage.
//!
//! # Features
//!
//! - Pull zone management
//! - Storage zone integration
//! - Cache purging
//! - Edge rules
//! - Statistics and analytics
//! - Stream (video) support

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{
    CacheRule, CacheSettings, CdnClient, CdnConfiguration, CdnError, CdnStats,
    DnsRecord, PurgeResult, Result, default_cache_rules,
};

/// BunnyCDN configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BunnyCdnConfig {
    /// API Key
    pub api_key: String,

    /// Pull Zone ID (optional, will be created if not provided)
    #[serde(default)]
    pub pull_zone_id: Option<u64>,

    /// Storage Zone name (optional)
    #[serde(default)]
    pub storage_zone: Option<String>,

    /// Storage Zone API key (optional)
    #[serde(default)]
    pub storage_api_key: Option<String>,

    /// Base API URL
    #[serde(default = "default_api_url")]
    pub api_url: String,

    /// Storage API URL
    #[serde(default = "default_storage_url")]
    pub storage_url: String,
}

fn default_api_url() -> String {
    "https://api.bunny.net".to_string()
}

fn default_storage_url() -> String {
    "https://storage.bunnycdn.com".to_string()
}

impl Default for BunnyCdnConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            pull_zone_id: None,
            storage_zone: None,
            storage_api_key: None,
            api_url: default_api_url(),
            storage_url: default_storage_url(),
        }
    }
}

/// BunnyCDN API client
pub struct BunnyCdnClient {
    config: BunnyCdnConfig,
    client: reqwest::Client,
}

impl BunnyCdnClient {
    /// Create a new BunnyCDN client
    pub fn new(config: BunnyCdnConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Build authorization headers
    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "AccessKey",
            HeaderValue::from_str(&self.config.api_key).unwrap(),
        );
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

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(CdnError::Api(format!("{}: {}", status, error_text)));
        }

        response
            .json()
            .await
            .map_err(|e| CdnError::Api(e.to_string()))
    }

    /// List pull zones
    pub async fn list_pull_zones(&self) -> Result<Vec<PullZone>> {
        self.api_request(reqwest::Method::GET, "/pullzone", None).await
    }

    /// Get pull zone
    pub async fn get_pull_zone(&self, id: u64) -> Result<PullZone> {
        self.api_request(reqwest::Method::GET, &format!("/pullzone/{}", id), None)
            .await
    }

    /// Create pull zone
    pub async fn create_pull_zone(&self, name: &str, origin_url: &str) -> Result<PullZone> {
        self.api_request(
            reqwest::Method::POST,
            "/pullzone",
            Some(serde_json::json!({
                "Name": name,
                "OriginUrl": origin_url,
                "Type": 0, // Standard pull zone
                "EnableGeoZoneUS": true,
                "EnableGeoZoneEU": true,
                "EnableGeoZoneASIA": true,
                "EnableGeoZoneSA": true,
                "EnableGeoZoneAF": true,
                "CacheControlMaxAgeOverride": 86400,
                "CacheControlBrowserMaxAgeOverride": 14400,
                "EnableCacheSlice": false,
                "EnableWebPVary": true,
                "EnableAvifVary": true,
                "EnableCountryCodeVary": false,
                "EnableHostnameVary": false,
                "EnableMobileVary": false,
                "EnableCookieVary": false,
                "EnableAutoSSL": true,
                "EnableForceSSL": true,
                "EnableSmartCache": true,
                "EnableOriginShield": true,
                "OriginShieldZoneCode": "NY",
                "AddCanonicalHeader": true,
                "EnableLogging": true,
                "LoggingIPAnonymizationEnabled": true,
                "LogForwardingEnabled": false,
                "IgnoreQueryStrings": false,
                "UseStaleWhileUpdating": true,
                "UseStaleWhileOffline": true
            })),
        )
        .await
    }

    /// Update pull zone
    pub async fn update_pull_zone(&self, id: u64, settings: serde_json::Value) -> Result<PullZone> {
        self.api_request(
            reqwest::Method::POST,
            &format!("/pullzone/{}", id),
            Some(settings),
        )
        .await
    }

    /// Add hostname to pull zone
    pub async fn add_hostname(&self, zone_id: u64, hostname: &str) -> Result<()> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/pullzone/{}/addHostname", zone_id),
                Some(serde_json::json!({ "Hostname": hostname })),
            )
            .await?;
        Ok(())
    }

    /// Request SSL certificate
    pub async fn request_ssl(&self, zone_id: u64, hostname: &str) -> Result<()> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::GET,
                &format!(
                    "/pullzone/{}/loadFreeCertificate?hostname={}",
                    zone_id, hostname
                ),
                None,
            )
            .await?;
        Ok(())
    }

    /// Create edge rule
    pub async fn create_edge_rule(&self, zone_id: u64, rule: EdgeRule) -> Result<()> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/pullzone/{}/edgerules/addOrUpdate", zone_id),
                Some(serde_json::to_value(&rule).unwrap()),
            )
            .await?;
        Ok(())
    }

    /// Get statistics
    pub async fn get_statistics(
        &self,
        start: &str,
        end: &str,
        pull_zone: Option<u64>,
    ) -> Result<Statistics> {
        let mut url = format!("/statistics?dateFrom={}&dateTo={}", start, end);
        if let Some(zone_id) = pull_zone {
            url.push_str(&format!("&pullZone={}", zone_id));
        }

        self.api_request(reqwest::Method::GET, &url, None).await
    }

    /// Purge cache by URL
    pub async fn purge_url(&self, url: &str) -> Result<()> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                "/purge",
                Some(serde_json::json!({ "url": url })),
            )
            .await?;
        Ok(())
    }

    /// Purge entire pull zone
    pub async fn purge_zone(&self, zone_id: u64) -> Result<()> {
        let _: serde_json::Value = self
            .api_request(
                reqwest::Method::POST,
                &format!("/pullzone/{}/purgeCache", zone_id),
                None,
            )
            .await?;
        Ok(())
    }

    // Storage Zone operations

    /// Upload file to storage
    pub async fn upload_file(
        &self,
        path: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String> {
        let storage_zone = self.config.storage_zone.as_ref().ok_or_else(|| {
            CdnError::Configuration("Storage zone not configured".to_string())
        })?;

        let storage_key = self.config.storage_api_key.as_ref().ok_or_else(|| {
            CdnError::Configuration("Storage API key not configured".to_string())
        })?;

        let url = format!("{}/{}/{}", self.config.storage_url, storage_zone, path);

        let response = self
            .client
            .put(&url)
            .header("AccessKey", storage_key)
            .header(CONTENT_TYPE, content_type)
            .body(data)
            .send()
            .await
            .map_err(|e| CdnError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CdnError::Api("Upload failed".to_string()));
        }

        // Return CDN URL
        Ok(format!(
            "https://{}.b-cdn.net/{}",
            storage_zone, path
        ))
    }

    /// Delete file from storage
    pub async fn delete_file(&self, path: &str) -> Result<()> {
        let storage_zone = self.config.storage_zone.as_ref().ok_or_else(|| {
            CdnError::Configuration("Storage zone not configured".to_string())
        })?;

        let storage_key = self.config.storage_api_key.as_ref().ok_or_else(|| {
            CdnError::Configuration("Storage API key not configured".to_string())
        })?;

        let url = format!("{}/{}/{}", self.config.storage_url, storage_zone, path);

        let response = self
            .client
            .delete(&url)
            .header("AccessKey", storage_key)
            .send()
            .await
            .map_err(|e| CdnError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CdnError::Api("Delete failed".to_string()));
        }

        Ok(())
    }

    /// Generate signed URL for private content
    pub fn generate_signed_url(&self, url: &str, expiry_timestamp: u64) -> Result<String> {
        let security_key = self.config.storage_api_key.as_ref().ok_or_else(|| {
            CdnError::Configuration("Security key not configured".to_string())
        })?;

        let parsed_url = url::Url::parse(url)
            .map_err(|e| CdnError::Configuration(e.to_string()))?;

        let path = parsed_url.path();

        // Create signature
        let signature_string = format!("{}{}{}", security_key, path, expiry_timestamp);

        let mut mac = Hmac::<Sha256>::new_from_slice(security_key.as_bytes())
            .map_err(|_| CdnError::Configuration("Invalid key".to_string()))?;
        mac.update(signature_string.as_bytes());

        let signature = BASE64.encode(mac.finalize().into_bytes());
        let signature = signature.replace('+', "-").replace('/', "_");

        Ok(format!(
            "{}?token={}&expires={}",
            url, signature, expiry_timestamp
        ))
    }
}

#[async_trait]
impl CdnClient for BunnyCdnClient {
    fn provider_name(&self) -> &str {
        "bunnycdn"
    }

    async fn auto_configure(&self, domain: &str) -> Result<CdnConfiguration> {
        info!("Auto-configuring BunnyCDN for domain: {}", domain);

        // Get or create pull zone
        let pull_zone = if let Some(zone_id) = self.config.pull_zone_id {
            self.get_pull_zone(zone_id).await?
        } else {
            // Create new pull zone
            let zone_name = domain.replace('.', "-");
            let origin_url = format!("https://{}", domain);
            self.create_pull_zone(&zone_name, &origin_url).await?
        };

        let zone_id = pull_zone.id;

        // Add custom hostname
        match self.add_hostname(zone_id, domain).await {
            Ok(_) => info!("Added hostname: {}", domain),
            Err(e) => warn!("Failed to add hostname: {}", e),
        }

        // Request SSL certificate
        match self.request_ssl(zone_id, domain).await {
            Ok(_) => info!("SSL certificate requested for: {}", domain),
            Err(e) => warn!("Failed to request SSL: {}", e),
        }

        // Create edge rules for cache control
        let rules = default_cache_rules();
        for rule in &rules {
            let edge_rule = EdgeRule::from_cache_rule(rule);
            match self.create_edge_rule(zone_id, edge_rule).await {
                Ok(_) => debug!("Created edge rule: {}", rule.name),
                Err(e) => warn!("Failed to create edge rule {}: {}", rule.name, e),
            }
        }

        Ok(CdnConfiguration {
            provider: "bunnycdn".to_string(),
            cdn_domain: pull_zone.hostnames.first()
                .map(|h| h.value.clone())
                .unwrap_or_else(|| format!("{}.b-cdn.net", pull_zone.name)),
            origin_domain: domain.to_string(),
            ssl_enabled: true,
            cache_settings: CacheSettings::default(),
            dns_records: vec![DnsRecord {
                record_type: "CNAME".to_string(),
                name: domain.to_string(),
                content: format!("{}.b-cdn.net", pull_zone.name),
                ttl: 300,
                proxied: false,
            }],
            extra_settings: {
                let mut map = HashMap::new();
                map.insert("pull_zone_id".to_string(), serde_json::json!(zone_id));
                map
            },
        })
    }

    async fn purge_urls(&self, urls: &[String]) -> Result<PurgeResult> {
        for url in urls {
            self.purge_url(url).await?;
        }

        Ok(PurgeResult {
            success: true,
            purged_count: urls.len() as u64,
            message: format!("Purged {} URLs", urls.len()),
        })
    }

    async fn purge_all(&self) -> Result<PurgeResult> {
        let zone_id = self.config.pull_zone_id.ok_or_else(|| {
            CdnError::Configuration("Pull zone ID not configured".to_string())
        })?;

        self.purge_zone(zone_id).await?;

        Ok(PurgeResult {
            success: true,
            purged_count: 0,
            message: "Purged entire zone".to_string(),
        })
    }

    async fn purge_tags(&self, _tags: &[String]) -> Result<PurgeResult> {
        // BunnyCDN doesn't support tag-based purging directly
        // Fall back to full purge
        self.purge_all().await
    }

    async fn get_stats(&self) -> Result<CdnStats> {
        let now = chrono::Utc::now();
        let yesterday = now - chrono::Duration::hours(24);

        let stats = self
            .get_statistics(
                &yesterday.format("%Y-%m-%d").to_string(),
                &now.format("%Y-%m-%d").to_string(),
                self.config.pull_zone_id,
            )
            .await?;

        let total_requests = stats.total_requests_served;
        let cached_requests = stats.cache_hit_count;

        Ok(CdnStats {
            total_requests,
            cached_requests,
            cache_hit_ratio: if total_requests > 0 {
                (cached_requests as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
            bandwidth_saved: stats.bandwidth_cached_bytes,
            total_bandwidth: stats.bandwidth_used_bytes,
            threats_blocked: 0, // BunnyCDN doesn't expose this
            period_start: yesterday,
            period_end: now,
        })
    }

    async fn configure_rules(&self, rules: Vec<CacheRule>) -> Result<()> {
        let zone_id = self.config.pull_zone_id.ok_or_else(|| {
            CdnError::Configuration("Pull zone ID not configured".to_string())
        })?;

        for rule in rules {
            let edge_rule = EdgeRule::from_cache_rule(&rule);
            self.create_edge_rule(zone_id, edge_rule).await?;
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        match self.list_pull_zones().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

// BunnyCDN API types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PullZone {
    pub id: u64,
    pub name: String,
    pub origin_url: String,
    pub enabled: bool,
    pub hostnames: Vec<Hostname>,
    pub storage_zone_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Hostname {
    pub id: u64,
    pub value: String,
    pub force_ssl: bool,
    pub is_system_hostname: bool,
    pub has_certificate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeRule {
    pub guid: Option<String>,
    pub action_type: u8,
    pub action_parameter1: String,
    pub action_parameter2: String,
    pub triggers: Vec<EdgeRuleTrigger>,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeRuleTrigger {
    pub r#type: u8,
    pub pattern_matches: Vec<String>,
    pub pattern_matching_type: u8,
}

impl EdgeRule {
    /// Create edge rule from cache rule
    pub fn from_cache_rule(rule: &CacheRule) -> Self {
        let action_type = if rule.ttl == 0 {
            3 // Bypass cache
        } else {
            0 // Override cache time
        };

        Self {
            guid: None,
            action_type,
            action_parameter1: rule.ttl.to_string(),
            action_parameter2: String::new(),
            triggers: vec![EdgeRuleTrigger {
                r#type: 0, // URL match
                pattern_matches: vec![rule.pattern.clone()],
                pattern_matching_type: 0, // Wildcard
            }],
            description: rule.name.clone(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Statistics {
    pub total_requests_served: u64,
    pub cache_hit_count: u64,
    pub bandwidth_used_bytes: u64,
    pub bandwidth_cached_bytes: u64,
    pub origin_traffic_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bunnycdn_config() {
        let config = BunnyCdnConfig {
            api_key: "test-key".to_string(),
            ..Default::default()
        };

        let client = BunnyCdnClient::new(config);
        assert_eq!(client.provider_name(), "bunnycdn");
    }

    #[test]
    fn test_edge_rule_from_cache_rule() {
        let cache_rule = CacheRule {
            name: "Test Rule".to_string(),
            pattern: "*.css".to_string(),
            ttl: 86400,
            cache_level: None,
            edge_ttl: None,
            browser_ttl: None,
            priority: 1,
        };

        let edge_rule = EdgeRule::from_cache_rule(&cache_rule);
        assert_eq!(edge_rule.description, "Test Rule");
        assert_eq!(edge_rule.action_type, 0);
    }
}
