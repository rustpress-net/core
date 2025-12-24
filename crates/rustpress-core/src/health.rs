//! Health check system for monitoring service status.

use crate::service::{HealthStatus, ServiceHealth};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: OverallStatus,
    pub version: String,
    pub uptime_secs: u64,
    pub checks: HashMap<String, HealthCheckResult>,
}

/// Overall system status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverallStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl From<HealthStatus> for OverallStatus {
    fn from(status: HealthStatus) -> Self {
        match status {
            HealthStatus::Healthy => Self::Healthy,
            HealthStatus::Degraded => Self::Degraded,
            HealthStatus::Unhealthy => Self::Unhealthy,
        }
    }
}

/// Result of a single health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: OverallStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<ServiceHealth> for HealthCheckResult {
    fn from(health: ServiceHealth) -> Self {
        Self {
            status: health.status.into(),
            message: health.message,
            latency_ms: health.latency_ms,
            details: None,
        }
    }
}

/// A health check that can be registered with the system
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of this health check
    fn name(&self) -> &str;

    /// Execute the health check
    async fn check(&self) -> HealthCheckResult;

    /// Whether this check is critical (affects overall status)
    fn is_critical(&self) -> bool {
        true
    }

    /// Timeout for this health check
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// Database health check
pub struct DatabaseHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    name: String,
    check_fn: F,
}

impl<F> DatabaseHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
        }
    }
}

#[async_trait]
impl<F> HealthCheck for DatabaseHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        match (self.check_fn)().await {
            Ok(()) => HealthCheckResult {
                status: OverallStatus::Healthy,
                message: None,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                details: None,
            },
            Err(e) => HealthCheckResult {
                status: OverallStatus::Unhealthy,
                message: Some(e),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                details: None,
            },
        }
    }
}

/// Redis health check
pub struct RedisHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    check_fn: F,
}

impl<F> RedisHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    pub fn new(check_fn: F) -> Self {
        Self { check_fn }
    }
}

#[async_trait]
impl<F> HealthCheck for RedisHealthCheck<F>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
{
    fn name(&self) -> &str {
        "redis"
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        match (self.check_fn)().await {
            Ok(()) => HealthCheckResult {
                status: OverallStatus::Healthy,
                message: None,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                details: None,
            },
            Err(e) => HealthCheckResult {
                status: OverallStatus::Unhealthy,
                message: Some(e),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                details: None,
            },
        }
    }

    fn is_critical(&self) -> bool {
        false // Redis is often optional
    }
}

/// Health check registry
pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheck>>,
    start_time: Instant,
    version: String,
}

impl HealthChecker {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            checks: Vec::new(),
            start_time: Instant::now(),
            version: version.into(),
        }
    }

    /// Register a health check
    pub fn register(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Run all health checks
    pub async fn check_all(&self) -> HealthResponse {
        let mut results = HashMap::new();
        let mut overall_status = OverallStatus::Healthy;

        for check in &self.checks {
            let timeout = check.timeout();
            let result = tokio::time::timeout(timeout, check.check()).await;

            let check_result = match result {
                Ok(r) => r,
                Err(_) => HealthCheckResult {
                    status: OverallStatus::Unhealthy,
                    message: Some("Health check timed out".to_string()),
                    latency_ms: Some(timeout.as_millis() as u64),
                    details: None,
                },
            };

            // Update overall status based on critical checks
            if check.is_critical() {
                match (&overall_status, &check_result.status) {
                    (OverallStatus::Healthy, OverallStatus::Degraded) => {
                        overall_status = OverallStatus::Degraded;
                    }
                    (_, OverallStatus::Unhealthy) => {
                        overall_status = OverallStatus::Unhealthy;
                    }
                    (OverallStatus::Degraded, OverallStatus::Healthy) => {
                        // Keep degraded
                    }
                    _ => {}
                }
            }

            results.insert(check.name().to_string(), check_result);
        }

        HealthResponse {
            status: overall_status,
            version: self.version.clone(),
            uptime_secs: self.start_time.elapsed().as_secs(),
            checks: results,
        }
    }

    /// Run a quick liveness check (just returns if server is running)
    pub fn liveness(&self) -> LivenessResponse {
        LivenessResponse {
            status: "ok".to_string(),
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }

    /// Run readiness check (checks if server can handle requests)
    pub async fn readiness(&self) -> ReadinessResponse {
        let health = self.check_all().await;
        ReadinessResponse {
            ready: health.status != OverallStatus::Unhealthy,
            status: health.status,
        }
    }
}

/// Simple liveness response
#[derive(Debug, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub status: String,
    pub uptime_secs: u64,
}

/// Readiness response
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub status: OverallStatus,
}

/// Memory health check
pub struct MemoryHealthCheck {
    max_memory_percent: f64,
}

impl MemoryHealthCheck {
    pub fn new(max_memory_percent: f64) -> Self {
        Self { max_memory_percent }
    }
}

#[async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        "memory"
    }

    async fn check(&self) -> HealthCheckResult {
        // This is a simplified check - in production you'd use sys-info crate
        // For now, always return healthy
        HealthCheckResult {
            status: OverallStatus::Healthy,
            message: None,
            latency_ms: Some(0),
            details: Some(serde_json::json!({
                "threshold_percent": self.max_memory_percent,
            })),
        }
    }

    fn is_critical(&self) -> bool {
        false
    }
}

/// Disk health check
pub struct DiskHealthCheck {
    path: String,
    min_free_bytes: u64,
}

impl DiskHealthCheck {
    pub fn new(path: impl Into<String>, min_free_bytes: u64) -> Self {
        Self {
            path: path.into(),
            min_free_bytes,
        }
    }
}

#[async_trait]
impl HealthCheck for DiskHealthCheck {
    fn name(&self) -> &str {
        "disk"
    }

    async fn check(&self) -> HealthCheckResult {
        // Simplified - in production you'd check actual disk space
        HealthCheckResult {
            status: OverallStatus::Healthy,
            message: None,
            latency_ms: Some(0),
            details: Some(serde_json::json!({
                "path": self.path,
                "min_free_bytes": self.min_free_bytes,
            })),
        }
    }

    fn is_critical(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker() {
        let mut checker = HealthChecker::new("1.0.0");

        // Add a simple always-healthy check
        struct AlwaysHealthy;

        #[async_trait]
        impl HealthCheck for AlwaysHealthy {
            fn name(&self) -> &str {
                "always_healthy"
            }

            async fn check(&self) -> HealthCheckResult {
                HealthCheckResult {
                    status: OverallStatus::Healthy,
                    message: None,
                    latency_ms: Some(1),
                    details: None,
                }
            }
        }

        checker.register(Arc::new(AlwaysHealthy));

        let response = checker.check_all().await;
        assert_eq!(response.status, OverallStatus::Healthy);
        assert!(response.checks.contains_key("always_healthy"));
    }

    #[test]
    fn test_liveness() {
        let checker = HealthChecker::new("1.0.0");
        let response = checker.liveness();
        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn test_readiness() {
        let checker = HealthChecker::new("1.0.0");
        let response = checker.readiness().await;
        assert!(response.ready);
    }
}
