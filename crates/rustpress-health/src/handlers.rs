//! HTTP handlers for health check endpoints

use crate::probes::{ProbeResult, ProbeType};
use crate::status::{HealthStatus, KubernetesProbeResponse};
use crate::HealthState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::sync::Arc;
use std::time::Instant;

/// Health check response wrapper
pub struct HealthResponse {
    status: StatusCode,
    body: serde_json::Value,
}

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Main health endpoint handler
/// GET /health
pub async fn health_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let report = state.get_health(false).await;
    let status = match report.status {
        HealthStatus::Healthy | HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    HealthResponse {
        status,
        body: serde_json::to_value(&report).unwrap_or_default(),
    }
}

/// Liveness probe handler
/// GET /health/live
pub async fn liveness_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let start = Instant::now();
    let alive = state.checker.check_liveness().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    if alive {
        let result = ProbeResult::success(ProbeType::Liveness, duration_ms);
        HealthResponse {
            status: StatusCode::OK,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    } else {
        let result = ProbeResult::failure(ProbeType::Liveness, "Application not alive", duration_ms);
        HealthResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    }
}

/// Readiness probe handler
/// GET /health/ready
pub async fn readiness_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let start = Instant::now();
    let ready = state.checker.check_readiness().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    if ready {
        let result = ProbeResult::success(ProbeType::Readiness, duration_ms);
        HealthResponse {
            status: StatusCode::OK,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    } else {
        let result = ProbeResult::failure(ProbeType::Readiness, "Application not ready", duration_ms);
        HealthResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    }
}

/// Startup probe handler
/// GET /health/startup
pub async fn startup_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let start = Instant::now();
    let started = state.checker.check_startup().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    if started {
        let result = ProbeResult::success(ProbeType::Startup, duration_ms);
        HealthResponse {
            status: StatusCode::OK,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    } else {
        let result = ProbeResult::failure(ProbeType::Startup, "Application still starting", duration_ms);
        HealthResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: serde_json::to_value(&result).unwrap_or_default(),
        }
    }
}

/// Kubernetes-style probe response
/// GET /healthz
pub async fn healthz_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let ready = state.checker.check_readiness().await;

    if ready {
        let response = KubernetesProbeResponse::pass();
        HealthResponse {
            status: StatusCode::OK,
            body: serde_json::to_value(&response).unwrap_or_default(),
        }
    } else {
        let response = KubernetesProbeResponse::fail("Service not ready");
        HealthResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            body: serde_json::to_value(&response).unwrap_or_default(),
        }
    }
}

/// Detailed health check with all components
/// GET /health/detailed
pub async fn detailed_health_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    // Force refresh for detailed check
    let report = state.get_health(true).await;
    let status = match report.status {
        HealthStatus::Healthy | HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    HealthResponse {
        status,
        body: serde_json::to_value(&report).unwrap_or_default(),
    }
}

/// Simple status endpoint (for load balancers)
/// GET /status
pub async fn status_handler() -> StatusCode {
    StatusCode::OK
}

/// Version endpoint
/// GET /version
pub async fn version_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "rustpress",
        "version": env!("CARGO_PKG_VERSION"),
        "git_hash": option_env!("GIT_HASH").unwrap_or("unknown"),
        "build_date": option_env!("BUILD_DATE").unwrap_or("unknown"),
    }))
}

/// Database health check
/// GET /health/db
pub async fn database_health_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let report = state.get_health(true).await;

    if let Some(db_health) = report.components.get("database") {
        let status = match db_health.status {
            crate::status::ComponentStatus::Up => StatusCode::OK,
            crate::status::ComponentStatus::Degraded => StatusCode::OK,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };

        HealthResponse {
            status,
            body: serde_json::to_value(db_health).unwrap_or_default(),
        }
    } else {
        HealthResponse {
            status: StatusCode::NOT_FOUND,
            body: serde_json::json!({"error": "Database health check not configured"}),
        }
    }
}

/// Cache health check
/// GET /health/cache
pub async fn cache_health_handler(State(state): State<Arc<HealthState>>) -> HealthResponse {
    let report = state.get_health(true).await;

    if let Some(cache_health) = report.components.get("cache") {
        let status = match cache_health.status {
            crate::status::ComponentStatus::Up => StatusCode::OK,
            crate::status::ComponentStatus::Degraded => StatusCode::OK,
            _ => StatusCode::SERVICE_UNAVAILABLE,
        };

        HealthResponse {
            status,
            body: serde_json::to_value(cache_health).unwrap_or_default(),
        }
    } else {
        HealthResponse {
            status: StatusCode::NOT_FOUND,
            body: serde_json::json!({"error": "Cache health check not configured"}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response() {
        let response = HealthResponse {
            status: StatusCode::OK,
            body: serde_json::json!({"status": "ok"}),
        };

        assert_eq!(response.status, StatusCode::OK);
    }
}
