//! RustPress Health Check Module
//!
//! This module provides health check endpoints for Kubernetes probes
//! and monitoring systems.
//!
//! # Features
//!
//! - Liveness probe: Is the application running?
//! - Readiness probe: Is the application ready to accept traffic?
//! - Startup probe: Has the application finished starting?
//! - Deep health checks: Database, cache, external services
//!
//! # Example
//!
//! ```rust,ignore
//! use rustpress_health::{HealthChecker, HealthRouter};
//!
//! let health_checker = HealthChecker::new()
//!     .with_database(pool.clone())
//!     .with_redis(redis_client.clone())
//!     .build();
//!
//! let app = Router::new()
//!     .merge(HealthRouter::new(health_checker));
//! ```

mod checker;
mod handlers;
mod probes;
mod router;
mod status;
mod system;

pub use checker::{HealthChecker, HealthCheckerBuilder};
pub use handlers::*;
pub use probes::{ProbeConfig, ProbeResult, ProbeType};
pub use router::HealthRouter;
pub use status::{
    ComponentHealth, ComponentStatus, HealthReport, HealthStatus, ServiceHealth,
};
pub use system::SystemHealth;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Health check state shared across handlers
pub struct HealthState {
    /// Health checker instance
    pub checker: Arc<HealthChecker>,

    /// Cached health status
    pub cached_status: Arc<RwLock<Option<CachedHealth>>>,

    /// Configuration
    pub config: HealthConfig,
}

/// Cached health check result
#[derive(Clone)]
pub struct CachedHealth {
    /// Health report
    pub report: HealthReport,

    /// Cache timestamp
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Health check configuration
#[derive(Clone, Debug)]
pub struct HealthConfig {
    /// Cache duration in seconds
    pub cache_duration: u64,

    /// Include detailed info in responses
    pub detailed: bool,

    /// Include system metrics
    pub include_system: bool,

    /// Timeout for health checks
    pub timeout_seconds: u64,

    /// Enable Kubernetes-specific endpoints
    pub kubernetes_mode: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            cache_duration: 5,
            detailed: true,
            include_system: true,
            timeout_seconds: 10,
            kubernetes_mode: true,
        }
    }
}

impl HealthState {
    /// Create new health state
    pub fn new(checker: HealthChecker, config: HealthConfig) -> Self {
        Self {
            checker: Arc::new(checker),
            cached_status: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Get health status, using cache if available
    pub async fn get_health(&self, force_refresh: bool) -> HealthReport {
        // Check cache
        if !force_refresh {
            let cache = self.cached_status.read().await;
            if let Some(cached) = cache.as_ref() {
                let age = chrono::Utc::now()
                    .signed_duration_since(cached.cached_at)
                    .num_seconds();

                if age < self.config.cache_duration as i64 {
                    return cached.report.clone();
                }
            }
        }

        // Perform health check
        let report = self.checker.check_all().await;

        // Update cache
        let mut cache = self.cached_status.write().await;
        *cache = Some(CachedHealth {
            report: report.clone(),
            cached_at: chrono::Utc::now(),
        });

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.cache_duration, 5);
        assert!(config.detailed);
        assert!(config.kubernetes_mode);
    }
}
