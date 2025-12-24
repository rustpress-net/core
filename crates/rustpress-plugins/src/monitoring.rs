//! Plugin Performance Monitoring and Error Isolation
//!
//! Tracks plugin performance and isolates errors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, error, warn};

// ============================================================================
// Performance Monitoring (Point 180)
// ============================================================================

/// Performance monitor
pub struct PerformanceMonitor {
    /// Metrics per plugin
    metrics: Arc<RwLock<HashMap<String, PluginMetrics>>>,
    /// Thresholds
    thresholds: PerformanceThresholds,
    /// Alerts
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
}

/// Performance thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Max execution time for a single hook (ms)
    pub max_hook_time_ms: u64,
    /// Max memory per plugin (bytes)
    pub max_memory_bytes: u64,
    /// Max CPU time per request (ms)
    pub max_cpu_time_ms: u64,
    /// Error rate threshold (percentage)
    pub error_rate_threshold: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_hook_time_ms: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50 MB
            max_cpu_time_ms: 500,
            error_rate_threshold: 5.0,
        }
    }
}

/// Plugin metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub plugin_id: String,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Number of executions
    pub execution_count: u64,
    /// Average execution time
    pub avg_execution_time_ms: f64,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Minimum execution time
    pub min_execution_time: Duration,
    /// Memory usage
    pub memory_bytes: u64,
    /// Peak memory usage
    pub peak_memory_bytes: u64,
    /// Error count
    pub error_count: u64,
    /// Last error
    pub last_error: Option<String>,
    /// Last execution
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    /// Hook metrics
    pub hook_metrics: HashMap<String, HookMetrics>,
}

/// Hook-level metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookMetrics {
    pub hook_name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub avg_time_ms: f64,
    pub max_time: Duration,
    pub error_count: u64,
}

/// Performance alert
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceAlert {
    pub plugin_id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    SlowExecution,
    HighMemory,
    HighErrorRate,
    ResourceExhaustion,
}

impl PerformanceMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            thresholds,
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start timing an execution
    pub fn start_timing(&self, plugin_id: &str) -> ExecutionTimer {
        ExecutionTimer {
            plugin_id: plugin_id.to_string(),
            hook_name: None,
            start: Instant::now(),
            monitor: self,
        }
    }

    /// Start timing a hook
    pub fn start_hook_timing(&self, plugin_id: &str, hook_name: &str) -> ExecutionTimer {
        ExecutionTimer {
            plugin_id: plugin_id.to_string(),
            hook_name: Some(hook_name.to_string()),
            start: Instant::now(),
            monitor: self,
        }
    }

    /// Record execution
    fn record_execution(&self, plugin_id: &str, hook_name: Option<&str>, duration: Duration, error: Option<&str>) {
        let mut metrics = self.metrics.write();
        let plugin_metrics = metrics.entry(plugin_id.to_string()).or_insert_with(|| {
            PluginMetrics {
                plugin_id: plugin_id.to_string(),
                min_execution_time: Duration::MAX,
                ..Default::default()
            }
        });

        plugin_metrics.execution_count += 1;
        plugin_metrics.total_execution_time += duration;
        plugin_metrics.avg_execution_time_ms =
            plugin_metrics.total_execution_time.as_millis() as f64 / plugin_metrics.execution_count as f64;

        if duration > plugin_metrics.max_execution_time {
            plugin_metrics.max_execution_time = duration;
        }
        if duration < plugin_metrics.min_execution_time {
            plugin_metrics.min_execution_time = duration;
        }

        plugin_metrics.last_execution = Some(chrono::Utc::now());

        if let Some(err) = error {
            plugin_metrics.error_count += 1;
            plugin_metrics.last_error = Some(err.to_string());
        }

        // Update hook metrics
        if let Some(hook) = hook_name {
            let hook_metrics = plugin_metrics
                .hook_metrics
                .entry(hook.to_string())
                .or_insert_with(|| HookMetrics {
                    hook_name: hook.to_string(),
                    ..Default::default()
                });

            hook_metrics.call_count += 1;
            hook_metrics.total_time += duration;
            hook_metrics.avg_time_ms = hook_metrics.total_time.as_millis() as f64 / hook_metrics.call_count as f64;

            if duration > hook_metrics.max_time {
                hook_metrics.max_time = duration;
            }

            if error.is_some() {
                hook_metrics.error_count += 1;
            }
        }

        // Check thresholds
        drop(metrics);
        self.check_thresholds(plugin_id, duration);
    }

    /// Check performance thresholds
    fn check_thresholds(&self, plugin_id: &str, duration: Duration) {
        if duration.as_millis() as u64 > self.thresholds.max_hook_time_ms {
            self.add_alert(PerformanceAlert {
                plugin_id: plugin_id.to_string(),
                alert_type: AlertType::SlowExecution,
                message: format!("Plugin execution exceeded {}ms threshold", self.thresholds.max_hook_time_ms),
                value: duration.as_millis() as f64,
                threshold: self.thresholds.max_hook_time_ms as f64,
                timestamp: chrono::Utc::now(),
            });
        }

        // Check error rate
        if let Some(metrics) = self.get_metrics(plugin_id) {
            let error_rate = if metrics.execution_count > 0 {
                (metrics.error_count as f64 / metrics.execution_count as f64) * 100.0
            } else {
                0.0
            };

            if error_rate > self.thresholds.error_rate_threshold as f64 {
                self.add_alert(PerformanceAlert {
                    plugin_id: plugin_id.to_string(),
                    alert_type: AlertType::HighErrorRate,
                    message: format!("Plugin error rate exceeded {}%", self.thresholds.error_rate_threshold),
                    value: error_rate,
                    threshold: self.thresholds.error_rate_threshold as f64,
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }

    /// Add an alert
    fn add_alert(&self, alert: PerformanceAlert) {
        warn!(
            "Performance alert for {}: {} (value: {}, threshold: {})",
            alert.plugin_id, alert.message, alert.value, alert.threshold
        );
        self.alerts.write().push(alert);
    }

    /// Get metrics for a plugin
    pub fn get_metrics(&self, plugin_id: &str) -> Option<PluginMetrics> {
        self.metrics.read().get(plugin_id).cloned()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<PluginMetrics> {
        self.metrics.read().values().cloned().collect()
    }

    /// Get alerts
    pub fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.read().clone()
    }

    /// Get alerts for plugin
    pub fn get_plugin_alerts(&self, plugin_id: &str) -> Vec<PerformanceAlert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| a.plugin_id == plugin_id)
            .cloned()
            .collect()
    }

    /// Clear alerts
    pub fn clear_alerts(&self) {
        self.alerts.write().clear();
    }

    /// Clear metrics for plugin
    pub fn clear_plugin(&self, plugin_id: &str) {
        self.metrics.write().remove(plugin_id);
        self.alerts.write().retain(|a| a.plugin_id != plugin_id);
    }

    /// Record memory usage
    pub fn record_memory(&self, plugin_id: &str, bytes: u64) {
        let mut metrics = self.metrics.write();
        if let Some(plugin_metrics) = metrics.get_mut(plugin_id) {
            plugin_metrics.memory_bytes = bytes;
            if bytes > plugin_metrics.peak_memory_bytes {
                plugin_metrics.peak_memory_bytes = bytes;
            }
        }

        if bytes > self.thresholds.max_memory_bytes {
            drop(metrics);
            self.add_alert(PerformanceAlert {
                plugin_id: plugin_id.to_string(),
                alert_type: AlertType::HighMemory,
                message: format!("Plugin memory exceeded {} bytes", self.thresholds.max_memory_bytes),
                value: bytes as f64,
                threshold: self.thresholds.max_memory_bytes as f64,
                timestamp: chrono::Utc::now(),
            });
        }
    }

    /// Get slow plugins
    pub fn get_slow_plugins(&self) -> Vec<PluginMetrics> {
        self.metrics
            .read()
            .values()
            .filter(|m| m.avg_execution_time_ms > self.thresholds.max_hook_time_ms as f64)
            .cloned()
            .collect()
    }
}

/// Execution timer
pub struct ExecutionTimer<'a> {
    plugin_id: String,
    hook_name: Option<String>,
    start: Instant,
    monitor: &'a PerformanceMonitor,
}

impl<'a> ExecutionTimer<'a> {
    pub fn finish(self) {
        let duration = self.start.elapsed();
        self.monitor.record_execution(&self.plugin_id, self.hook_name.as_deref(), duration, None);
    }

    pub fn finish_with_error(self, error: &str) {
        let duration = self.start.elapsed();
        self.monitor.record_execution(&self.plugin_id, self.hook_name.as_deref(), duration, Some(error));
    }
}

// ============================================================================
// Error Isolation (Point 181)
// ============================================================================

/// Error isolator
pub struct ErrorIsolator {
    /// Error history per plugin
    errors: Arc<RwLock<HashMap<String, Vec<PluginError>>>>,
    /// Disabled plugins (due to errors)
    disabled: Arc<RwLock<HashMap<String, DisabledReason>>>,
    /// Configuration
    config: IsolationConfig,
}

/// Isolation configuration
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// Max errors before disabling
    pub max_errors: u32,
    /// Error window (minutes)
    pub error_window_minutes: u32,
    /// Auto-disable on critical errors
    pub auto_disable: bool,
    /// Retry after (minutes)
    pub retry_after_minutes: u32,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            max_errors: 5,
            error_window_minutes: 5,
            auto_disable: true,
            retry_after_minutes: 30,
        }
    }
}

/// Plugin error
#[derive(Debug, Clone, Serialize)]
pub struct PluginError {
    pub plugin_id: String,
    pub error_type: ErrorType,
    pub message: String,
    pub stack_trace: Option<String>,
    pub context: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ErrorSeverity,
}

/// Error type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    Runtime,
    Timeout,
    MemoryExhausted,
    Database,
    Network,
    Permission,
    Configuration,
    Unknown,
}

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Disabled reason
#[derive(Debug, Clone, Serialize)]
pub struct DisabledReason {
    pub reason: String,
    pub disabled_at: chrono::DateTime<chrono::Utc>,
    pub retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count: u32,
}

impl ErrorIsolator {
    pub fn new(config: IsolationConfig) -> Self {
        Self {
            errors: Arc::new(RwLock::new(HashMap::new())),
            disabled: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Record an error
    pub fn record_error(&self, error: PluginError) {
        let plugin_id = error.plugin_id.clone();
        let severity = error.severity;

        error!(
            plugin = %plugin_id,
            error_type = ?error.error_type,
            "Plugin error: {}",
            error.message
        );

        // Add to history
        {
            let mut errors = self.errors.write();
            let plugin_errors = errors.entry(plugin_id.clone()).or_insert_with(Vec::new);
            plugin_errors.push(error);

            // Clean old errors
            let window = chrono::Duration::minutes(self.config.error_window_minutes as i64);
            let cutoff = chrono::Utc::now() - window;
            plugin_errors.retain(|e| e.timestamp > cutoff);
        }

        // Check if should disable
        if self.config.auto_disable {
            self.check_should_disable(&plugin_id, severity);
        }
    }

    /// Check if plugin should be disabled
    fn check_should_disable(&self, plugin_id: &str, severity: ErrorSeverity) {
        // Critical errors disable immediately
        if severity == ErrorSeverity::Critical {
            self.disable_plugin(plugin_id, "Critical error occurred");
            return;
        }

        // Check error count in window
        let error_count = self.get_recent_error_count(plugin_id);
        if error_count >= self.config.max_errors {
            self.disable_plugin(plugin_id, &format!(
                "Too many errors ({} in {} minutes)",
                error_count,
                self.config.error_window_minutes
            ));
        }
    }

    /// Get recent error count
    fn get_recent_error_count(&self, plugin_id: &str) -> u32 {
        let errors = self.errors.read();
        errors.get(plugin_id).map(|e| e.len() as u32).unwrap_or(0)
    }

    /// Disable a plugin
    pub fn disable_plugin(&self, plugin_id: &str, reason: &str) {
        let retry_at = if self.config.retry_after_minutes > 0 {
            Some(chrono::Utc::now() + chrono::Duration::minutes(self.config.retry_after_minutes as i64))
        } else {
            None
        };

        let disabled_reason = DisabledReason {
            reason: reason.to_string(),
            disabled_at: chrono::Utc::now(),
            retry_at,
            error_count: self.get_recent_error_count(plugin_id),
        };

        warn!(plugin = %plugin_id, "Disabling plugin: {}", reason);
        self.disabled.write().insert(plugin_id.to_string(), disabled_reason);
    }

    /// Enable a plugin
    pub fn enable_plugin(&self, plugin_id: &str) {
        self.disabled.write().remove(plugin_id);
        self.errors.write().remove(plugin_id);
    }

    /// Check if plugin is disabled
    pub fn is_disabled(&self, plugin_id: &str) -> bool {
        let disabled = self.disabled.read();
        if let Some(reason) = disabled.get(plugin_id) {
            // Check if retry time has passed
            if let Some(retry_at) = reason.retry_at {
                if chrono::Utc::now() >= retry_at {
                    return false; // Can retry
                }
            }
            return true;
        }
        false
    }

    /// Get disabled reason
    pub fn get_disabled_reason(&self, plugin_id: &str) -> Option<DisabledReason> {
        self.disabled.read().get(plugin_id).cloned()
    }

    /// Get all disabled plugins
    pub fn get_disabled_plugins(&self) -> Vec<(String, DisabledReason)> {
        self.disabled
            .read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get errors for plugin
    pub fn get_errors(&self, plugin_id: &str) -> Vec<PluginError> {
        self.errors.read().get(plugin_id).cloned().unwrap_or_default()
    }

    /// Clear errors for plugin
    pub fn clear_errors(&self, plugin_id: &str) {
        self.errors.write().remove(plugin_id);
    }

    /// Execute with error isolation
    pub async fn execute_isolated<F, T, E>(
        &self,
        plugin_id: &str,
        f: F,
    ) -> Result<T, IsolationError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if disabled
        if self.is_disabled(plugin_id) {
            return Err(IsolationError::PluginDisabled(plugin_id.to_string()));
        }

        // Execute
        match f.await {
            Ok(result) => Ok(result),
            Err(e) => {
                self.record_error(PluginError {
                    plugin_id: plugin_id.to_string(),
                    error_type: ErrorType::Runtime,
                    message: e.to_string(),
                    stack_trace: None,
                    context: None,
                    timestamp: chrono::Utc::now(),
                    severity: ErrorSeverity::Medium,
                });
                Err(IsolationError::ExecutionFailed(e.to_string()))
            }
        }
    }
}

/// Isolation error
#[derive(Debug, thiserror::Error)]
pub enum IsolationError {
    #[error("Plugin is disabled: {0}")]
    PluginDisabled(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new(PerformanceThresholds::default());

        let timer = monitor.start_timing("test-plugin");
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.finish();

        let metrics = monitor.get_metrics("test-plugin").unwrap();
        assert_eq!(metrics.execution_count, 1);
        assert!(metrics.avg_execution_time_ms >= 10.0);
    }

    #[test]
    fn test_error_isolator() {
        let isolator = ErrorIsolator::new(IsolationConfig {
            max_errors: 2,
            error_window_minutes: 5,
            auto_disable: true,
            retry_after_minutes: 0,
        });

        // Record errors
        for i in 0..3 {
            isolator.record_error(PluginError {
                plugin_id: "test-plugin".to_string(),
                error_type: ErrorType::Runtime,
                message: format!("Error {}", i),
                stack_trace: None,
                context: None,
                timestamp: chrono::Utc::now(),
                severity: ErrorSeverity::Medium,
            });
        }

        // Should be disabled after 2 errors
        assert!(isolator.is_disabled("test-plugin"));
    }

    #[test]
    fn test_critical_error_disables() {
        let isolator = ErrorIsolator::new(IsolationConfig::default());

        isolator.record_error(PluginError {
            plugin_id: "test-plugin".to_string(),
            error_type: ErrorType::Runtime,
            message: "Critical failure".to_string(),
            stack_trace: None,
            context: None,
            timestamp: chrono::Utc::now(),
            severity: ErrorSeverity::Critical,
        });

        assert!(isolator.is_disabled("test-plugin"));
    }
}
