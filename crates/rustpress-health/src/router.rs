//! Health check router configuration

use crate::handlers::*;
use crate::{HealthChecker, HealthConfig, HealthState};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

/// Health check router builder
pub struct HealthRouter;

#[allow(clippy::new_ret_no_self)]
impl HealthRouter {
    /// Create a new health router with default configuration
    pub fn new(checker: HealthChecker) -> Router {
        Self::with_config(checker, HealthConfig::default())
    }

    /// Create a new health router with custom configuration
    pub fn with_config(checker: HealthChecker, config: HealthConfig) -> Router {
        let state = Arc::new(HealthState::new(checker, config.clone()));

        let mut router = Router::new()
            // Main health endpoint
            .route("/health", get(health_handler))
            // Liveness probe
            .route("/health/live", get(liveness_handler))
            // Readiness probe
            .route("/health/ready", get(readiness_handler))
            // Startup probe
            .route("/health/startup", get(startup_handler))
            // Detailed health check
            .route("/health/detailed", get(detailed_health_handler))
            // Database health
            .route("/health/db", get(database_health_handler))
            // Cache health
            .route("/health/cache", get(cache_health_handler))
            // Simple status
            .route("/status", get(status_handler))
            // Version info
            .route("/version", get(version_handler));

        // Add Kubernetes-specific endpoints
        if config.kubernetes_mode {
            router = router
                .route("/healthz", get(healthz_handler))
                .route("/readyz", get(readiness_handler))
                .route("/livez", get(liveness_handler));
        }

        router.with_state(state)
    }

    /// Create minimal router for basic health checks
    pub fn minimal(checker: HealthChecker) -> Router {
        let config = HealthConfig {
            detailed: false,
            include_system: false,
            ..Default::default()
        };

        let state = Arc::new(HealthState::new(checker, config));

        Router::new()
            .route("/health", get(health_handler))
            .route("/health/live", get(liveness_handler))
            .route("/health/ready", get(readiness_handler))
            .with_state(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthCheckerBuilder;

    #[test]
    fn test_router_creation() {
        let checker = HealthCheckerBuilder::new().build();
        let _router = HealthRouter::new(checker);
    }

    #[test]
    fn test_minimal_router() {
        let checker = HealthCheckerBuilder::new().build();
        let _router = HealthRouter::minimal(checker);
    }
}
