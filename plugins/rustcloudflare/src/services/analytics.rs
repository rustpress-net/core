//! Analytics service

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::Analytics;
use sqlx::PgPool;
use std::sync::Arc;
use chrono::{Duration, Utc};

pub struct AnalyticsService {
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
}

impl AnalyticsService {
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

    pub async fn get_dashboard(&self, hours: i32) -> CloudflareResult<Analytics> {
        let client = self.get_client()?;
        let since = Utc::now() - Duration::hours(hours as i64);
        let until = Utc::now();

        client.get_analytics(
            &since.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            &until.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ).await
    }

    pub async fn get_traffic_summary(&self) -> CloudflareResult<TrafficSummary> {
        let analytics = self.get_dashboard(24).await?;
        let totals = analytics.totals.unwrap_or_default();
        let requests = totals.requests.unwrap_or_default();
        let bandwidth = totals.bandwidth.unwrap_or_default();
        let threats = totals.threats.unwrap_or_default();

        Ok(TrafficSummary {
            total_requests: requests.all,
            cached_requests: requests.cached,
            uncached_requests: requests.uncached,
            total_bandwidth_bytes: bandwidth.all,
            cached_bandwidth_bytes: bandwidth.cached,
            threats_blocked: threats.all,
            cache_hit_rate: if requests.all > 0 {
                (requests.cached as f64 / requests.all as f64) * 100.0
            } else {
                0.0
            },
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrafficSummary {
    pub total_requests: i64,
    pub cached_requests: i64,
    pub uncached_requests: i64,
    pub total_bandwidth_bytes: i64,
    pub cached_bandwidth_bytes: i64,
    pub threats_blocked: i64,
    pub cache_hit_rate: f64,
}

impl Default for crate::models::AnalyticsThreats {
    fn default() -> Self {
        Self {
            all: 0,
            country: None,
            threat_type: None,
        }
    }
}
