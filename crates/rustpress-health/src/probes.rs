//! Kubernetes probe types and configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Probe type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeType {
    /// Liveness probe - is the container alive?
    Liveness,
    /// Readiness probe - is the container ready to accept traffic?
    Readiness,
    /// Startup probe - has the container finished starting?
    Startup,
}

/// Probe result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe type
    pub probe_type: ProbeType,
    /// Success status
    pub success: bool,
    /// Message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Check duration in milliseconds
    pub duration_ms: u64,
}

impl ProbeResult {
    /// Create a successful probe result
    pub fn success(probe_type: ProbeType, duration_ms: u64) -> Self {
        Self {
            probe_type,
            success: true,
            message: None,
            duration_ms,
        }
    }

    /// Create a failed probe result
    pub fn failure(probe_type: ProbeType, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            probe_type,
            success: false,
            message: Some(message.into()),
            duration_ms,
        }
    }
}

/// Probe configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProbeConfig {
    /// Initial delay before starting probes
    pub initial_delay: Duration,
    /// Period between probes
    pub period: Duration,
    /// Timeout for each probe
    pub timeout: Duration,
    /// Number of consecutive successes before considered healthy
    pub success_threshold: u32,
    /// Number of consecutive failures before considered unhealthy
    pub failure_threshold: u32,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(5),
            period: Duration::from_secs(10),
            timeout: Duration::from_secs(3),
            success_threshold: 1,
            failure_threshold: 3,
        }
    }
}

impl ProbeConfig {
    /// Create liveness probe configuration
    pub fn liveness() -> Self {
        Self {
            initial_delay: Duration::from_secs(10),
            period: Duration::from_secs(10),
            timeout: Duration::from_secs(3),
            success_threshold: 1,
            failure_threshold: 3,
        }
    }

    /// Create readiness probe configuration
    pub fn readiness() -> Self {
        Self {
            initial_delay: Duration::from_secs(5),
            period: Duration::from_secs(5),
            timeout: Duration::from_secs(3),
            success_threshold: 1,
            failure_threshold: 3,
        }
    }

    /// Create startup probe configuration
    pub fn startup() -> Self {
        Self {
            initial_delay: Duration::from_secs(0),
            period: Duration::from_secs(5),
            timeout: Duration::from_secs(3),
            success_threshold: 1,
            failure_threshold: 30, // Allow up to 150 seconds for startup
        }
    }
}

/// Kubernetes probe YAML generator
#[allow(dead_code)]
pub struct ProbeYamlGenerator;

#[allow(dead_code)]
impl ProbeYamlGenerator {
    /// Generate liveness probe YAML
    pub fn liveness_probe(path: &str, port: u16, config: &ProbeConfig) -> String {
        format!(
            r#"livenessProbe:
  httpGet:
    path: {}
    port: {}
  initialDelaySeconds: {}
  periodSeconds: {}
  timeoutSeconds: {}
  successThreshold: {}
  failureThreshold: {}"#,
            path,
            port,
            config.initial_delay.as_secs(),
            config.period.as_secs(),
            config.timeout.as_secs(),
            config.success_threshold,
            config.failure_threshold
        )
    }

    /// Generate readiness probe YAML
    pub fn readiness_probe(path: &str, port: u16, config: &ProbeConfig) -> String {
        format!(
            r#"readinessProbe:
  httpGet:
    path: {}
    port: {}
  initialDelaySeconds: {}
  periodSeconds: {}
  timeoutSeconds: {}
  successThreshold: {}
  failureThreshold: {}"#,
            path,
            port,
            config.initial_delay.as_secs(),
            config.period.as_secs(),
            config.timeout.as_secs(),
            config.success_threshold,
            config.failure_threshold
        )
    }

    /// Generate startup probe YAML
    pub fn startup_probe(path: &str, port: u16, config: &ProbeConfig) -> String {
        format!(
            r#"startupProbe:
  httpGet:
    path: {}
    port: {}
  initialDelaySeconds: {}
  periodSeconds: {}
  timeoutSeconds: {}
  successThreshold: {}
  failureThreshold: {}"#,
            path,
            port,
            config.initial_delay.as_secs(),
            config.period.as_secs(),
            config.timeout.as_secs(),
            config.success_threshold,
            config.failure_threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_result() {
        let success = ProbeResult::success(ProbeType::Liveness, 10);
        assert!(success.success);
        assert!(success.message.is_none());

        let failure = ProbeResult::failure(ProbeType::Readiness, "timeout", 5000);
        assert!(!failure.success);
        assert_eq!(failure.message, Some("timeout".to_string()));
    }

    #[test]
    fn test_probe_config() {
        let liveness = ProbeConfig::liveness();
        assert_eq!(liveness.initial_delay, Duration::from_secs(10));

        let readiness = ProbeConfig::readiness();
        assert_eq!(readiness.initial_delay, Duration::from_secs(5));

        let startup = ProbeConfig::startup();
        assert_eq!(startup.failure_threshold, 30);
    }

    #[test]
    fn test_yaml_generation() {
        let config = ProbeConfig::liveness();
        let yaml = ProbeYamlGenerator::liveness_probe("/health/live", 8080, &config);

        assert!(yaml.contains("livenessProbe:"));
        assert!(yaml.contains("/health/live"));
        assert!(yaml.contains("8080"));
    }
}
