//! Cache management service

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::PurgeResponse;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Cache management service
pub struct CacheService {
    client: Option<Arc<CloudflareClient>>,
    db: PgPool,
}

impl CacheService {
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self { client: Some(client), db }
    }

    /// Create without a configured client (for initial setup)
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db }
    }

    /// Get the client or return an error if not configured
    fn get_client(&self) -> CloudflareResult<&CloudflareClient> {
        self.client.as_ref()
            .map(|c| c.as_ref())
            .ok_or_else(|| CloudflareError::ConfigError("Cloudflare not configured. Please connect your account.".to_string()))
    }

    /// Purge entire cache
    pub async fn purge_all(&self) -> CloudflareResult<PurgeResponse> {
        let client = self.get_client()?;
        info!("Purging all cache for zone {}", client.zone_id());
        let result = client.purge_all_cache().await?;
        self.log_purge_event("purge_all", None).await?;
        Ok(result)
    }

    /// Purge cache by URLs
    pub async fn purge_urls(&self, urls: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let client = self.get_client()?;
        info!("Purging {} URLs from cache", urls.len());
        let result = client.purge_cache_by_urls(urls.clone()).await?;
        self.log_purge_event("purge_urls", Some(serde_json::json!({ "urls": urls })))
            .await?;
        Ok(result)
    }

    /// Purge cache by tags
    pub async fn purge_tags(&self, tags: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let client = self.get_client()?;
        info!("Purging cache by tags: {:?}", tags);
        let result = client.purge_cache_by_tags(tags.clone()).await?;
        self.log_purge_event("purge_tags", Some(serde_json::json!({ "tags": tags })))
            .await?;
        Ok(result)
    }

    /// Purge cache by prefix
    pub async fn purge_prefix(&self, prefixes: Vec<String>) -> CloudflareResult<PurgeResponse> {
        let client = self.get_client()?;
        info!("Purging cache by prefixes: {:?}", prefixes);
        let result = client.purge_cache_by_prefix(prefixes.clone()).await?;
        self.log_purge_event("purge_prefix", Some(serde_json::json!({ "prefixes": prefixes })))
            .await?;
        Ok(result)
    }

    /// Auto-purge on content update
    pub async fn auto_purge_post(&self, post_url: &str) -> CloudflareResult<()> {
        // Purge the post URL and related URLs
        let urls_to_purge = vec![
            post_url.to_string(),
            format!("{}/", post_url),
            // Also purge homepage and archive pages
        ];

        self.purge_urls(urls_to_purge).await?;
        Ok(())
    }

    /// Log purge event to database
    async fn log_purge_event(
        &self,
        event_type: &str,
        details: Option<serde_json::Value>,
    ) -> CloudflareResult<()> {
        sqlx::query(
            r#"
            INSERT INTO cloudflare_cache_events (event_type, details, created_at)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(event_type)
        .bind(details)
        .execute(&self.db)
        .await
        .map_err(|e| CloudflareError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get cache statistics from recent analytics
    pub async fn get_cache_stats(&self, hours: i32) -> CloudflareResult<CacheStats> {
        let client = self.get_client()?;
        let since = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
        let analytics = client.get_analytics(
            &since.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            &chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ).await?;

        let totals = analytics.totals.unwrap_or_default();
        let requests = totals.requests.unwrap_or_default();
        let bandwidth = totals.bandwidth.unwrap_or_default();

        Ok(CacheStats {
            total_requests: requests.all,
            cached_requests: requests.cached,
            uncached_requests: requests.uncached,
            cache_hit_ratio: if requests.all > 0 {
                (requests.cached as f64 / requests.all as f64) * 100.0
            } else {
                0.0
            },
            total_bandwidth: bandwidth.all,
            cached_bandwidth: bandwidth.cached,
            uncached_bandwidth: bandwidth.uncached,
            bandwidth_saved_ratio: if bandwidth.all > 0 {
                (bandwidth.cached as f64 / bandwidth.all as f64) * 100.0
            } else {
                0.0
            },
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    pub total_requests: i64,
    pub cached_requests: i64,
    pub uncached_requests: i64,
    pub cache_hit_ratio: f64,
    pub total_bandwidth: i64,
    pub cached_bandwidth: i64,
    pub uncached_bandwidth: i64,
    pub bandwidth_saved_ratio: f64,
}

impl Default for crate::models::AnalyticsTotals {
    fn default() -> Self {
        Self {
            requests: None,
            bandwidth: None,
            threats: None,
            pageviews: None,
            uniques: None,
        }
    }
}

impl Default for crate::models::AnalyticsRequests {
    fn default() -> Self {
        Self {
            all: 0,
            cached: 0,
            uncached: 0,
            content_type: None,
            country: None,
            ssl: None,
            ssl_protocols: None,
            http_status: None,
        }
    }
}

impl Default for crate::models::AnalyticsBandwidth {
    fn default() -> Self {
        Self {
            all: 0,
            cached: 0,
            uncached: 0,
            content_type: None,
            country: None,
            ssl: None,
        }
    }
}
