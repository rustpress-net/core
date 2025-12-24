//! Health status types and responses

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall health status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All components healthy
    Healthy,
    /// Some components degraded but functional
    Degraded,
    /// Critical components unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Convert to HTTP status code
    pub fn to_http_status(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still operational
            HealthStatus::Unhealthy => 503,
        }
    }

    /// Check if status indicates the service is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Healthy
    }
}

/// Component health status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentStatus {
    /// Component is healthy
    Up,
    /// Component is degraded
    Degraded,
    /// Component is down
    Down,
    /// Component status unknown
    Unknown,
}

impl ComponentStatus {
    /// Convert to overall health status
    pub fn to_health_status(&self) -> HealthStatus {
        match self {
            ComponentStatus::Up => HealthStatus::Healthy,
            ComponentStatus::Degraded => HealthStatus::Degraded,
            ComponentStatus::Down | ComponentStatus::Unknown => HealthStatus::Unhealthy,
        }
    }
}

/// Health check report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall status
    pub status: HealthStatus,

    /// Timestamp of check
    pub timestamp: DateTime<Utc>,

    /// Service information
    pub service: ServiceHealth,

    /// Component health details
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentHealth>,

    /// System metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemMetrics>,

    /// Check duration in milliseconds
    pub check_duration_ms: u64,
}

impl Default for HealthReport {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            service: ServiceHealth::default(),
            components: HashMap::new(),
            system: None,
            check_duration_ms: 0,
        }
    }
}

impl HealthReport {
    /// Create a new health report
    pub fn new() -> Self {
        Self::default()
    }

    /// Add component health
    pub fn add_component(&mut self, name: impl Into<String>, health: ComponentHealth) {
        self.components.insert(name.into(), health);
    }

    /// Calculate overall status from components
    pub fn calculate_status(&mut self) {
        let mut has_critical_down = false;
        let mut has_degraded = false;

        for (name, component) in &self.components {
            match component.status {
                ComponentStatus::Down => {
                    // Check if critical component
                    if is_critical_component(name) {
                        has_critical_down = true;
                    } else {
                        has_degraded = true;
                    }
                }
                ComponentStatus::Degraded => {
                    has_degraded = true;
                }
                ComponentStatus::Unknown => {
                    if is_critical_component(name) {
                        has_degraded = true;
                    }
                }
                ComponentStatus::Up => {}
            }
        }

        self.status = if has_critical_down {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
    }
}

/// Check if a component is critical
fn is_critical_component(name: &str) -> bool {
    matches!(name.to_lowercase().as_str(), "database" | "postgres" | "postgresql")
}

/// Service health information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Environment
    pub environment: String,

    /// Instance ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Start time
    pub started_at: DateTime<Utc>,
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            name: "rustpress".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("RUSTPRESS_ENV").unwrap_or_else(|_| "development".to_string()),
            instance_id: std::env::var("HOSTNAME").ok(),
            uptime_seconds: 0,
            started_at: Utc::now(),
        }
    }
}

/// Individual component health
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: ComponentStatus,

    /// Component type/name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_type: Option<String>,

    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,

    /// Error message if unhealthy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Last successful check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success: Option<DateTime<Utc>>,
}

impl ComponentHealth {
    /// Create healthy component
    pub fn healthy() -> Self {
        Self {
            status: ComponentStatus::Up,
            component_type: None,
            response_time_ms: None,
            details: None,
            error: None,
            last_success: Some(Utc::now()),
        }
    }

    /// Create unhealthy component
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            status: ComponentStatus::Down,
            component_type: None,
            response_time_ms: None,
            details: None,
            error: Some(error.into()),
            last_success: None,
        }
    }

    /// Create degraded component
    pub fn degraded(reason: impl Into<String>) -> Self {
        Self {
            status: ComponentStatus::Degraded,
            component_type: None,
            response_time_ms: None,
            details: None,
            error: Some(reason.into()),
            last_success: Some(Utc::now()),
        }
    }

    /// Set component type
    pub fn with_type(mut self, component_type: impl Into<String>) -> Self {
        self.component_type = Some(component_type.into());
        self
    }

    /// Set response time
    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = Some(ms);
        self
    }

    /// Add detail
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let details = self.details.get_or_insert_with(HashMap::new);
        if let Ok(v) = serde_json::to_value(value) {
            details.insert(key.into(), v);
        }
        self
    }
}

/// System metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,

    /// Memory usage percentage (0-100)
    pub memory_usage: f32,

    /// Total memory in bytes
    pub memory_total: u64,

    /// Used memory in bytes
    pub memory_used: u64,

    /// Disk usage percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_usage: Option<f32>,

    /// Load average (1, 5, 15 minutes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_average: Option<[f64; 3]>,

    /// Number of active connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_connections: Option<u64>,

    /// Number of open file descriptors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_files: Option<u64>,
}

/// Kubernetes-specific probe response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KubernetesProbeResponse {
    /// Probe status (pass/fail)
    pub status: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl KubernetesProbeResponse {
    /// Create passing probe response
    pub fn pass() -> Self {
        Self {
            status: "pass".to_string(),
            timestamp: Utc::now(),
            message: None,
        }
    }

    /// Create failing probe response
    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            status: "fail".to_string(),
            timestamp: Utc::now(),
            message: Some(message.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_http_codes() {
        assert_eq!(HealthStatus::Healthy.to_http_status(), 200);
        assert_eq!(HealthStatus::Degraded.to_http_status(), 200);
        assert_eq!(HealthStatus::Unhealthy.to_http_status(), 503);
    }

    #[test]
    fn test_component_health_builder() {
        let health = ComponentHealth::healthy()
            .with_type("database")
            .with_response_time(15)
            .with_detail("connections", 42);

        assert_eq!(health.status, ComponentStatus::Up);
        assert_eq!(health.component_type, Some("database".to_string()));
        assert_eq!(health.response_time_ms, Some(15));
    }

    #[test]
    fn test_calculate_status() {
        let mut report = HealthReport::new();

        // All healthy
        report.add_component("database", ComponentHealth::healthy());
        report.add_component("cache", ComponentHealth::healthy());
        report.calculate_status();
        assert_eq!(report.status, HealthStatus::Healthy);

        // Non-critical degraded
        report.add_component("cache", ComponentHealth::degraded("slow"));
        report.calculate_status();
        assert_eq!(report.status, HealthStatus::Degraded);

        // Critical down
        report.add_component("database", ComponentHealth::unhealthy("connection failed"));
        report.calculate_status();
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }
}
