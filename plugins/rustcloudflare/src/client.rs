//! Cloudflare API Client
//!
//! HTTP client for communicating with the Cloudflare API

use crate::config::CloudflareConfig;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::*;
use reqwest::{header, Client, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Default max retry attempts
const MAX_RETRIES: u32 = 3;
/// Base delay for exponential backoff (milliseconds)
const BASE_DELAY_MS: u64 = 1000;

/// Base URL for Cloudflare API
const API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API client
#[derive(Debug, Clone)]
pub struct CloudflareClient {
    client: Client,
    #[allow(dead_code)]
    api_token: String,
    account_id: String,
    zone_id: String,
}

impl CloudflareClient {
    /// Create a new Cloudflare API client
    pub fn new(config: &CloudflareConfig) -> CloudflareResult<Self> {
        config.validate()?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.api_token))
                .map_err(|_| CloudflareError::InvalidToken)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| CloudflareError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            api_token: config.api_token.clone(),
            account_id: config.account_id.clone(),
            zone_id: config.zone_id.clone(),
        })
    }

    /// Get the zone ID
    pub fn zone_id(&self) -> &str {
        &self.zone_id
    }

    /// Get the account ID
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    /// Verify the connection to Cloudflare
    pub async fn verify_connection(&self) -> CloudflareResult<()> {
        let url = format!("{}/user/tokens/verify", API_BASE_URL);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            debug!("Cloudflare connection verified");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Cloudflare connection failed: {} - {}", status, body);
            Err(CloudflareError::AuthenticationError(
                "Token verification failed".to_string(),
            ))
        }
    }

    /// Check if an error is retryable
    fn is_retryable_error(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
                | StatusCode::BAD_GATEWAY
        )
    }

    /// Calculate backoff delay with exponential increase and jitter
    fn calculate_backoff(attempt: u32) -> Duration {
        let base_delay = BASE_DELAY_MS * 2u64.pow(attempt);
        let jitter = rand::random::<u64>() % 500;
        Duration::from_millis(base_delay + jitter)
    }

    /// Make a GET request with retry logic
    async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> CloudflareResult<ApiResponse<T>> {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        for attempt in 0..MAX_RETRIES {
            debug!("GET {} (attempt {})", url, attempt + 1);

            match self.client.get(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if Self::is_retryable_error(status) && attempt < MAX_RETRIES - 1 {
                        let delay = Self::calculate_backoff(attempt);
                        warn!("Retryable error {} for GET {}, retrying in {:?}", status, url, delay);
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) if attempt < MAX_RETRIES - 1 && e.is_timeout() => {
                    let delay = Self::calculate_backoff(attempt);
                    warn!("Timeout for GET {}, retrying in {:?}", url, delay);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(CloudflareError::NetworkError("Max retries exceeded".to_string()))
    }

    /// Make a POST request with retry logic
    async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> CloudflareResult<ApiResponse<T>> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        let body_json = serde_json::to_value(body).map_err(|e| CloudflareError::Internal(e.to_string()))?;

        for attempt in 0..MAX_RETRIES {
            debug!("POST {} (attempt {})", url, attempt + 1);

            match self.client.post(&url).json(&body_json).send().await {
                Ok(response) => {
                    let status = response.status();
                    if Self::is_retryable_error(status) && attempt < MAX_RETRIES - 1 {
                        let delay = Self::calculate_backoff(attempt);
                        warn!("Retryable error {} for POST {}, retrying in {:?}", status, url, delay);
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) if attempt < MAX_RETRIES - 1 && e.is_timeout() => {
                    let delay = Self::calculate_backoff(attempt);
                    warn!("Timeout for POST {}, retrying in {:?}", url, delay);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(CloudflareError::NetworkError("Max retries exceeded".to_string()))
    }

    /// Make a PUT request with retry logic
    async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> CloudflareResult<ApiResponse<T>> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        let body_json = serde_json::to_value(body).map_err(|e| CloudflareError::Internal(e.to_string()))?;

        for attempt in 0..MAX_RETRIES {
            debug!("PUT {} (attempt {})", url, attempt + 1);

            match self.client.put(&url).json(&body_json).send().await {
                Ok(response) => {
                    let status = response.status();
                    if Self::is_retryable_error(status) && attempt < MAX_RETRIES - 1 {
                        let delay = Self::calculate_backoff(attempt);
                        warn!("Retryable error {} for PUT {}, retrying in {:?}", status, url, delay);
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) if attempt < MAX_RETRIES - 1 && e.is_timeout() => {
                    let delay = Self::calculate_backoff(attempt);
                    warn!("Timeout for PUT {}, retrying in {:?}", url, delay);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(CloudflareError::NetworkError("Max retries exceeded".to_string()))
    }

    /// Make a PATCH request with retry logic
    async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> CloudflareResult<ApiResponse<T>> {
        let url = format!("{}{}", API_BASE_URL, endpoint);
        let body_json = serde_json::to_value(body).map_err(|e| CloudflareError::Internal(e.to_string()))?;

        for attempt in 0..MAX_RETRIES {
            debug!("PATCH {} (attempt {})", url, attempt + 1);

            match self.client.patch(&url).json(&body_json).send().await {
                Ok(response) => {
                    let status = response.status();
                    if Self::is_retryable_error(status) && attempt < MAX_RETRIES - 1 {
                        let delay = Self::calculate_backoff(attempt);
                        warn!("Retryable error {} for PATCH {}, retrying in {:?}", status, url, delay);
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) if attempt < MAX_RETRIES - 1 && e.is_timeout() => {
                    let delay = Self::calculate_backoff(attempt);
                    warn!("Timeout for PATCH {}, retrying in {:?}", url, delay);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(CloudflareError::NetworkError("Max retries exceeded".to_string()))
    }

    /// Make a DELETE request with retry logic
    async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> CloudflareResult<ApiResponse<T>> {
        let url = format!("{}{}", API_BASE_URL, endpoint);

        for attempt in 0..MAX_RETRIES {
            debug!("DELETE {} (attempt {})", url, attempt + 1);

            match self.client.delete(&url).send().await {
                Ok(response) => {
                    let status = response.status();
                    if Self::is_retryable_error(status) && attempt < MAX_RETRIES - 1 {
                        let delay = Self::calculate_backoff(attempt);
                        warn!("Retryable error {} for DELETE {}, retrying in {:?}", status, url, delay);
                        sleep(delay).await;
                        continue;
                    }
                    return self.handle_response(response).await;
                }
                Err(e) if attempt < MAX_RETRIES - 1 && e.is_timeout() => {
                    let delay = Self::calculate_backoff(attempt);
                    warn!("Timeout for DELETE {}, retrying in {:?}", url, delay);
                    sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(CloudflareError::NetworkError("Max retries exceeded".to_string()))
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> CloudflareResult<ApiResponse<T>> {
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(CloudflareError::RateLimitExceeded);
        }

        let body = response.text().await?;

        let api_response: ApiResponse<T> = serde_json::from_str(&body).map_err(|e| {
            error!("Failed to parse response: {} - Body: {}", e, body);
            CloudflareError::Internal(format!("Failed to parse response: {}", e))
        })?;

        if !api_response.success {
            if let Some(errors) = &api_response.errors {
                if let Some(first_error) = errors.first() {
                    return Err(CloudflareError::ApiError {
                        code: first_error.code,
                        message: first_error.message.clone(),
                    });
                }
            }
            return Err(CloudflareError::ApiError {
                code: 0,
                message: "Unknown API error".to_string(),
            });
        }

        Ok(api_response)
    }

    // =========================================================================
    // Zone Operations
    // =========================================================================

    /// Get zone details
    pub async fn get_zone(&self) -> CloudflareResult<Zone> {
        let response: ApiResponse<Zone> = self
            .get(&format!("/zones/{}", self.zone_id))
            .await?;
        response.result.ok_or(CloudflareError::ZoneNotFound(self.zone_id.clone()))
    }

    /// Get zone settings
    pub async fn get_zone_settings(&self) -> CloudflareResult<Vec<ZoneSetting>> {
        let response: ApiResponse<Vec<ZoneSetting>> = self
            .get(&format!("/zones/{}/settings", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Update zone setting
    pub async fn update_zone_setting(
        &self,
        setting_id: &str,
        value: serde_json::Value,
    ) -> CloudflareResult<ZoneSetting> {
        let body = serde_json::json!({ "value": value });
        let response: ApiResponse<ZoneSetting> = self
            .patch(
                &format!("/zones/{}/settings/{}", self.zone_id, setting_id),
                &body,
            )
            .await?;
        response.result.ok_or(CloudflareError::NotFound(setting_id.to_string()))
    }

    /// Toggle development mode
    pub async fn toggle_development_mode(&self, enabled: bool) -> CloudflareResult<ZoneSetting> {
        self.update_zone_setting(
            "development_mode",
            serde_json::json!(if enabled { "on" } else { "off" }),
        )
        .await
    }

    // =========================================================================
    // Cache Operations
    // =========================================================================

    /// Purge all cache
    pub async fn purge_all_cache(&self) -> CloudflareResult<PurgeResponse> {
        let body = serde_json::json!({ "purge_everything": true });
        let response: ApiResponse<PurgeResponse> = self
            .post(&format!("/zones/{}/purge_cache", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::CacheError("Purge failed".to_string()))
    }

    /// Purge cache by URLs
    pub async fn purge_cache_by_urls(&self, urls: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let body = serde_json::json!({ "files": urls });
        let response: ApiResponse<PurgeResponse> = self
            .post(&format!("/zones/{}/purge_cache", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::CacheError("Purge failed".to_string()))
    }

    /// Purge cache by tags
    pub async fn purge_cache_by_tags(&self, tags: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let body = serde_json::json!({ "tags": tags });
        let response: ApiResponse<PurgeResponse> = self
            .post(&format!("/zones/{}/purge_cache", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::CacheError("Purge failed".to_string()))
    }

    /// Purge cache by prefix
    pub async fn purge_cache_by_prefix(&self, prefixes: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let body = serde_json::json!({ "prefixes": prefixes });
        let response: ApiResponse<PurgeResponse> = self
            .post(&format!("/zones/{}/purge_cache", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::CacheError("Purge failed".to_string()))
    }

    // =========================================================================
    // DNS Operations
    // =========================================================================

    /// List DNS records
    pub async fn list_dns_records(
        &self,
        params: Option<DnsListParams>,
    ) -> CloudflareResult<Vec<DnsRecord>> {
        let mut endpoint = format!("/zones/{}/dns_records", self.zone_id);

        if let Some(p) = params {
            let query = serde_urlencoded::to_string(&p).unwrap_or_default();
            if !query.is_empty() {
                endpoint = format!("{}?{}", endpoint, query);
            }
        }

        let response: ApiResponse<Vec<DnsRecord>> = self.get(&endpoint).await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create DNS record
    pub async fn create_dns_record(&self, record: CreateDnsRecord) -> CloudflareResult<DnsRecord> {
        let response: ApiResponse<DnsRecord> = self
            .post(&format!("/zones/{}/dns_records", self.zone_id), &record)
            .await?;
        response.result.ok_or(CloudflareError::DnsError("Create failed".to_string()))
    }

    /// Update DNS record
    pub async fn update_dns_record(
        &self,
        id: &str,
        record: UpdateDnsRecord,
    ) -> CloudflareResult<DnsRecord> {
        let response: ApiResponse<DnsRecord> = self
            .put(&format!("/zones/{}/dns_records/{}", self.zone_id, id), &record)
            .await?;
        response.result.ok_or(CloudflareError::DnsError("Update failed".to_string()))
    }

    /// Delete DNS record
    pub async fn delete_dns_record(&self, id: &str) -> CloudflareResult<DeleteResponse> {
        let response: ApiResponse<DeleteResponse> = self
            .delete(&format!("/zones/{}/dns_records/{}", self.zone_id, id))
            .await?;
        response.result.ok_or(CloudflareError::DnsError("Delete failed".to_string()))
    }

    // =========================================================================
    // SSL/TLS Operations
    // =========================================================================

    /// Get SSL/TLS settings
    pub async fn get_ssl_settings(&self) -> CloudflareResult<SslSettings> {
        let response: ApiResponse<SslSettings> = self
            .get(&format!("/zones/{}/settings/ssl", self.zone_id))
            .await?;
        response.result.ok_or(CloudflareError::SslError("Failed to get SSL settings".to_string()))
    }

    /// Update SSL mode
    pub async fn update_ssl_mode(&self, mode: &str) -> CloudflareResult<SslSettings> {
        let body = serde_json::json!({ "value": mode });
        let response: ApiResponse<SslSettings> = self
            .patch(&format!("/zones/{}/settings/ssl", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::SslError("Failed to update SSL".to_string()))
    }

    /// List SSL certificates
    pub async fn list_certificates(&self) -> CloudflareResult<Vec<Certificate>> {
        let response: ApiResponse<Vec<Certificate>> = self
            .get(&format!("/zones/{}/ssl/certificate_packs", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    // =========================================================================
    // Security / WAF Operations
    // =========================================================================

    /// Get security level
    pub async fn get_security_level(&self) -> CloudflareResult<ZoneSetting> {
        let response: ApiResponse<ZoneSetting> = self
            .get(&format!("/zones/{}/settings/security_level", self.zone_id))
            .await?;
        response.result.ok_or(CloudflareError::NotFound("security_level".to_string()))
    }

    /// Set security level
    pub async fn set_security_level(&self, level: &str) -> CloudflareResult<ZoneSetting> {
        self.update_zone_setting("security_level", serde_json::json!(level))
            .await
    }

    /// Toggle Under Attack mode
    pub async fn toggle_under_attack_mode(&self, enabled: bool) -> CloudflareResult<ZoneSetting> {
        let level = if enabled { "under_attack" } else { "medium" };
        self.set_security_level(level).await
    }

    /// List WAF rules
    pub async fn list_waf_rules(&self) -> CloudflareResult<Vec<WafRule>> {
        let response: ApiResponse<Vec<WafRule>> = self
            .get(&format!("/zones/{}/firewall/waf/packages", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// List firewall rules
    pub async fn list_firewall_rules(&self) -> CloudflareResult<Vec<FirewallRule>> {
        let response: ApiResponse<Vec<FirewallRule>> = self
            .get(&format!("/zones/{}/firewall/rules", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create firewall rule
    pub async fn create_firewall_rule(
        &self,
        rule: CreateFirewallRule,
    ) -> CloudflareResult<FirewallRule> {
        let response: ApiResponse<Vec<FirewallRule>> = self
            .post(&format!("/zones/{}/firewall/rules", self.zone_id), &[rule])
            .await?;
        response
            .result
            .and_then(|v| v.into_iter().next())
            .ok_or(CloudflareError::WafError("Create failed".to_string()))
    }

    /// List IP access rules
    pub async fn list_ip_access_rules(&self) -> CloudflareResult<Vec<IpAccessRule>> {
        let response: ApiResponse<Vec<IpAccessRule>> = self
            .get(&format!("/zones/{}/firewall/access_rules/rules", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create IP access rule
    pub async fn create_ip_access_rule(
        &self,
        rule: CreateIpAccessRule,
    ) -> CloudflareResult<IpAccessRule> {
        let response: ApiResponse<IpAccessRule> = self
            .post(
                &format!("/zones/{}/firewall/access_rules/rules", self.zone_id),
                &rule,
            )
            .await?;
        response.result.ok_or(CloudflareError::WafError("Create failed".to_string()))
    }

    // =========================================================================
    // Page Rules Operations
    // =========================================================================

    /// List page rules
    pub async fn list_page_rules(&self) -> CloudflareResult<Vec<PageRule>> {
        let response: ApiResponse<Vec<PageRule>> = self
            .get(&format!("/zones/{}/pagerules", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create page rule
    pub async fn create_page_rule(&self, rule: CreatePageRule) -> CloudflareResult<PageRule> {
        let response: ApiResponse<PageRule> = self
            .post(&format!("/zones/{}/pagerules", self.zone_id), &rule)
            .await?;
        response.result.ok_or(CloudflareError::NotFound("Page rule".to_string()))
    }

    /// Update page rule
    pub async fn update_page_rule(
        &self,
        id: &str,
        rule: UpdatePageRule,
    ) -> CloudflareResult<PageRule> {
        let response: ApiResponse<PageRule> = self
            .put(&format!("/zones/{}/pagerules/{}", self.zone_id, id), &rule)
            .await?;
        response.result.ok_or(CloudflareError::NotFound(id.to_string()))
    }

    /// Delete page rule
    pub async fn delete_page_rule(&self, id: &str) -> CloudflareResult<DeleteResponse> {
        let response: ApiResponse<DeleteResponse> = self
            .delete(&format!("/zones/{}/pagerules/{}", self.zone_id, id))
            .await?;
        response.result.ok_or(CloudflareError::NotFound(id.to_string()))
    }

    // =========================================================================
    // Workers Operations
    // =========================================================================

    /// List Workers
    pub async fn list_workers(&self) -> CloudflareResult<Vec<Worker>> {
        let response: ApiResponse<Vec<Worker>> = self
            .get(&format!("/accounts/{}/workers/scripts", self.account_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Get Worker script
    pub async fn get_worker(&self, name: &str) -> CloudflareResult<Worker> {
        let response: ApiResponse<Worker> = self
            .get(&format!(
                "/accounts/{}/workers/scripts/{}",
                self.account_id, name
            ))
            .await?;
        response.result.ok_or(CloudflareError::WorkerError(format!(
            "Worker '{}' not found",
            name
        )))
    }

    /// Deploy Worker script
    pub async fn deploy_worker(&self, name: &str, script: &str) -> CloudflareResult<Worker> {
        // Workers API requires multipart form data for script upload
        let url = format!(
            "{}/accounts/{}/workers/scripts/{}",
            API_BASE_URL, self.account_id, name
        );

        let form = reqwest::multipart::Form::new()
            .text("script", script.to_string());

        let response = self
            .client
            .put(&url)
            .multipart(form)
            .send()
            .await?;

        let api_response: ApiResponse<Worker> = self.handle_response(response).await?;
        api_response.result.ok_or(CloudflareError::WorkerError("Deploy failed".to_string()))
    }

    /// Delete Worker
    pub async fn delete_worker(&self, name: &str) -> CloudflareResult<DeleteResponse> {
        let response: ApiResponse<DeleteResponse> = self
            .delete(&format!(
                "/accounts/{}/workers/scripts/{}",
                self.account_id, name
            ))
            .await?;
        response.result.ok_or(CloudflareError::WorkerError("Delete failed".to_string()))
    }

    /// List Worker routes
    pub async fn list_worker_routes(&self) -> CloudflareResult<Vec<WorkerRoute>> {
        let response: ApiResponse<Vec<WorkerRoute>> = self
            .get(&format!("/zones/{}/workers/routes", self.zone_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create Worker route
    pub async fn create_worker_route(
        &self,
        pattern: &str,
        script: &str,
    ) -> CloudflareResult<WorkerRoute> {
        let body = serde_json::json!({
            "pattern": pattern,
            "script": script
        });
        let response: ApiResponse<WorkerRoute> = self
            .post(&format!("/zones/{}/workers/routes", self.zone_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::WorkerError("Create route failed".to_string()))
    }

    // =========================================================================
    // Workers KV Operations
    // =========================================================================

    /// List KV namespaces
    pub async fn list_kv_namespaces(&self) -> CloudflareResult<Vec<KvNamespace>> {
        let response: ApiResponse<Vec<KvNamespace>> = self
            .get(&format!(
                "/accounts/{}/storage/kv/namespaces",
                self.account_id
            ))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create KV namespace
    pub async fn create_kv_namespace(&self, title: &str) -> CloudflareResult<KvNamespace> {
        let body = serde_json::json!({ "title": title });
        let response: ApiResponse<KvNamespace> = self
            .post(
                &format!("/accounts/{}/storage/kv/namespaces", self.account_id),
                &body,
            )
            .await?;
        response.result.ok_or(CloudflareError::KvError("Create failed".to_string()))
    }

    /// List KV keys
    pub async fn list_kv_keys(&self, namespace_id: &str) -> CloudflareResult<Vec<KvKey>> {
        let response: ApiResponse<Vec<KvKey>> = self
            .get(&format!(
                "/accounts/{}/storage/kv/namespaces/{}/keys",
                self.account_id, namespace_id
            ))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Get KV value
    pub async fn get_kv_value(&self, namespace_id: &str, key: &str) -> CloudflareResult<String> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            API_BASE_URL, self.account_id, namespace_id, key
        );

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(CloudflareError::KvError(format!("Key '{}' not found", key)))
        }
    }

    /// Set KV value
    pub async fn set_kv_value(
        &self,
        namespace_id: &str,
        key: &str,
        value: &str,
    ) -> CloudflareResult<()> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            API_BASE_URL, self.account_id, namespace_id, key
        );

        let response = self.client.put(&url).body(value.to_string()).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CloudflareError::KvError("Failed to set value".to_string()))
        }
    }

    /// Delete KV value
    pub async fn delete_kv_value(&self, namespace_id: &str, key: &str) -> CloudflareResult<()> {
        let url = format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values/{}",
            API_BASE_URL, self.account_id, namespace_id, key
        );

        let response = self.client.delete(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CloudflareError::KvError("Failed to delete value".to_string()))
        }
    }

    // =========================================================================
    // Analytics Operations
    // =========================================================================

    /// Get zone analytics
    pub async fn get_analytics(&self, since: &str, until: &str) -> CloudflareResult<Analytics> {
        let response: ApiResponse<Analytics> = self
            .get(&format!(
                "/zones/{}/analytics/dashboard?since={}&until={}",
                self.zone_id, since, until
            ))
            .await?;
        response.result.ok_or(CloudflareError::NotFound("Analytics".to_string()))
    }

    // =========================================================================
    // D1 Database Operations
    // =========================================================================

    /// List D1 databases
    pub async fn list_d1_databases(&self) -> CloudflareResult<Vec<D1Database>> {
        let response: ApiResponse<Vec<D1Database>> = self
            .get(&format!("/accounts/{}/d1/database", self.account_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create D1 database
    pub async fn create_d1_database(&self, name: &str) -> CloudflareResult<D1Database> {
        let body = serde_json::json!({ "name": name });
        let response: ApiResponse<D1Database> = self
            .post(&format!("/accounts/{}/d1/database", self.account_id), &body)
            .await?;
        response.result.ok_or(CloudflareError::D1Error("Create failed".to_string()))
    }

    /// Query D1 database
    pub async fn query_d1(
        &self,
        database_id: &str,
        sql: &str,
    ) -> CloudflareResult<D1QueryResult> {
        let body = serde_json::json!({ "sql": sql });
        let response: ApiResponse<Vec<D1QueryResult>> = self
            .post(
                &format!(
                    "/accounts/{}/d1/database/{}/query",
                    self.account_id, database_id
                ),
                &body,
            )
            .await?;
        response
            .result
            .and_then(|v| v.into_iter().next())
            .ok_or(CloudflareError::D1Error("Query failed".to_string()))
    }

    // =========================================================================
    // Stream Operations
    // =========================================================================

    /// List Stream videos
    pub async fn list_stream_videos(&self) -> CloudflareResult<Vec<StreamVideo>> {
        let response: ApiResponse<Vec<StreamVideo>> = self
            .get(&format!("/accounts/{}/stream", self.account_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Get Stream video
    pub async fn get_stream_video(&self, video_id: &str) -> CloudflareResult<StreamVideo> {
        let response: ApiResponse<StreamVideo> = self
            .get(&format!("/accounts/{}/stream/{}", self.account_id, video_id))
            .await?;
        response.result.ok_or(CloudflareError::StreamError(format!(
            "Video '{}' not found",
            video_id
        )))
    }

    /// Delete Stream video
    pub async fn delete_stream_video(&self, video_id: &str) -> CloudflareResult<()> {
        let _: ApiResponse<serde_json::Value> = self
            .delete(&format!("/accounts/{}/stream/{}", self.account_id, video_id))
            .await?;
        Ok(())
    }

    /// List live inputs
    pub async fn list_live_inputs(&self) -> CloudflareResult<Vec<LiveInput>> {
        let response: ApiResponse<Vec<LiveInput>> = self
            .get(&format!("/accounts/{}/stream/live_inputs", self.account_id))
            .await?;
        Ok(response.result.unwrap_or_default())
    }

    /// Create live input
    pub async fn create_live_input(&self, input: CreateLiveInput) -> CloudflareResult<LiveInput> {
        let response: ApiResponse<LiveInput> = self
            .post(
                &format!("/accounts/{}/stream/live_inputs", self.account_id),
                &input,
            )
            .await?;
        response.result.ok_or(CloudflareError::StreamError("Create failed".to_string()))
    }
}
