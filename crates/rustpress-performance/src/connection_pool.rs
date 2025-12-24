//! Database Connection Pooling Tuning
//!
//! Configures and optimizes database connection pools for high performance.

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Maximum number of connections allowed
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle connection timeout in seconds
    pub idle_timeout: u64,
    /// Maximum connection lifetime in seconds
    pub max_lifetime: u64,
    /// Time to wait for a connection from pool
    pub acquire_timeout: u64,
    /// Validate connections before use
    pub test_before_acquire: bool,
    /// Connection validation query
    pub validation_query: Option<String>,
    /// Enable statement caching
    pub statement_cache_size: usize,
    /// Enable connection recycling
    pub recycle_connections: bool,
    /// Health check interval in seconds
    pub health_check_interval: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            connect_timeout: 10,
            idle_timeout: 300,
            max_lifetime: 3600,
            acquire_timeout: 30,
            test_before_acquire: true,
            validation_query: Some("SELECT 1".to_string()),
            statement_cache_size: 100,
            recycle_connections: true,
            health_check_interval: 30,
        }
    }
}

impl PoolConfig {
    /// Create configuration optimized for read-heavy workloads
    pub fn read_optimized() -> Self {
        Self {
            min_connections: 10,
            max_connections: 50,
            idle_timeout: 600,
            statement_cache_size: 200,
            ..Default::default()
        }
    }

    /// Create configuration optimized for write-heavy workloads
    pub fn write_optimized() -> Self {
        Self {
            min_connections: 5,
            max_connections: 25,
            idle_timeout: 180,
            test_before_acquire: true,
            statement_cache_size: 50,
            ..Default::default()
        }
    }

    /// Create configuration for low-memory environments
    pub fn low_memory() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            idle_timeout: 120,
            statement_cache_size: 25,
            ..Default::default()
        }
    }

    /// Create configuration for high-traffic environments
    pub fn high_traffic() -> Self {
        Self {
            min_connections: 20,
            max_connections: 100,
            connect_timeout: 5,
            idle_timeout: 900,
            acquire_timeout: 15,
            statement_cache_size: 500,
            ..Default::default()
        }
    }

    /// Calculate optimal pool size based on system resources
    pub fn auto_tune(available_connections: u32, cpu_cores: u32) -> Self {
        // PostgreSQL formula: connections = (core_count * 2) + effective_spindle_count
        // For SSDs, effective_spindle_count = 1
        let optimal_connections = (cpu_cores * 2) + 1;

        // Don't exceed available connections (usually 100 for PostgreSQL)
        let max_connections = optimal_connections.min(available_connections / 2);
        let min_connections = (max_connections / 4).max(2);

        Self {
            min_connections,
            max_connections,
            ..Default::default()
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolStats {
    /// Current number of active connections
    pub active_connections: u32,
    /// Current number of idle connections
    pub idle_connections: u32,
    /// Total connections created
    pub connections_created: u64,
    /// Total connections closed
    pub connections_closed: u64,
    /// Total connection acquisitions
    pub acquisitions: u64,
    /// Failed acquisition attempts
    pub acquisition_failures: u64,
    /// Total wait time for acquisitions (ms)
    pub total_wait_time_ms: u64,
    /// Average acquisition time (ms)
    pub avg_acquisition_time_ms: f64,
    /// Peak active connections
    pub peak_connections: u32,
    /// Health check failures
    pub health_check_failures: u64,
}

/// Connection pool monitor
pub struct PoolMonitor {
    config: PoolConfig,
    stats: Arc<RwLock<PoolStats>>,
    /// Threshold for alerting
    utilization_threshold: f64,
}

impl PoolMonitor {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(PoolStats::default())),
            utilization_threshold: 0.8,
        }
    }

    /// Update pool statistics
    pub fn update_stats(&self, stats: PoolStats) {
        *self.stats.write() = stats;
    }

    /// Get current statistics
    pub fn get_stats(&self) -> PoolStats {
        self.stats.read().clone()
    }

    /// Check pool health
    pub fn check_health(&self) -> PoolHealth {
        let stats = self.stats.read();
        let total = stats.active_connections + stats.idle_connections;
        let utilization = stats.active_connections as f64 / self.config.max_connections as f64;

        let status = if utilization >= 0.95 {
            HealthStatus::Critical
        } else if utilization >= self.utilization_threshold {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        let recommendations = self.generate_recommendations(&stats, utilization);

        PoolHealth {
            status,
            utilization,
            total_connections: total,
            max_connections: self.config.max_connections,
            average_wait_ms: stats.avg_acquisition_time_ms,
            recommendations,
        }
    }

    fn generate_recommendations(&self, stats: &PoolStats, utilization: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if utilization >= 0.8 {
            recommendations.push(format!(
                "Pool utilization is high ({:.1}%). Consider increasing max_connections from {} to {}",
                utilization * 100.0,
                self.config.max_connections,
                (self.config.max_connections as f64 * 1.5) as u32
            ));
        }

        if stats.avg_acquisition_time_ms > 100.0 {
            recommendations.push(format!(
                "Average connection wait time is high ({:.1}ms). Consider increasing pool size",
                stats.avg_acquisition_time_ms
            ));
        }

        if stats.acquisition_failures > 0 {
            recommendations.push(format!(
                "There have been {} acquisition failures. Review pool configuration and database limits",
                stats.acquisition_failures
            ));
        }

        if stats.idle_connections as f64 / self.config.max_connections as f64 > 0.5 {
            recommendations.push(format!(
                "Many idle connections ({}). Consider reducing min_connections",
                stats.idle_connections
            ));
        }

        if stats.health_check_failures > 0 {
            recommendations.push("Health check failures detected. Review database connectivity".to_string());
        }

        recommendations
    }

    /// Record an acquisition event
    pub fn record_acquisition(&self, wait_time_ms: u64, success: bool) {
        let mut stats = self.stats.write();
        stats.acquisitions += 1;

        if success {
            stats.total_wait_time_ms += wait_time_ms;
            stats.avg_acquisition_time_ms = stats.total_wait_time_ms as f64 / stats.acquisitions as f64;
        } else {
            stats.acquisition_failures += 1;
        }
    }

    /// Record connection creation
    pub fn record_connection_created(&self) {
        let mut stats = self.stats.write();
        stats.connections_created += 1;
        stats.active_connections += 1;

        if stats.active_connections > stats.peak_connections {
            stats.peak_connections = stats.active_connections;
        }
    }

    /// Record connection closed
    pub fn record_connection_closed(&self) {
        let mut stats = self.stats.write();
        stats.connections_closed += 1;
        if stats.active_connections > 0 {
            stats.active_connections -= 1;
        }
    }
}

/// Pool health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealth {
    pub status: HealthStatus,
    pub utilization: f64,
    pub total_connections: u32,
    pub max_connections: u32,
    pub average_wait_ms: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

/// Connection pool advisor for auto-tuning
pub struct PoolAdvisor {
    historical_stats: Vec<PoolStats>,
    max_history: usize,
}

impl PoolAdvisor {
    pub fn new(max_history: usize) -> Self {
        Self {
            historical_stats: Vec::new(),
            max_history,
        }
    }

    /// Record stats for analysis
    pub fn record(&mut self, stats: PoolStats) {
        self.historical_stats.push(stats);

        if self.historical_stats.len() > self.max_history {
            self.historical_stats.remove(0);
        }
    }

    /// Analyze historical stats and suggest configuration
    pub fn suggest_config(&self, current: &PoolConfig) -> PoolConfig {
        if self.historical_stats.is_empty() {
            return current.clone();
        }

        let avg_active: f64 = self.historical_stats.iter()
            .map(|s| s.active_connections as f64)
            .sum::<f64>() / self.historical_stats.len() as f64;

        let peak_active = self.historical_stats.iter()
            .map(|s| s.peak_connections)
            .max()
            .unwrap_or(0);

        let avg_idle: f64 = self.historical_stats.iter()
            .map(|s| s.idle_connections as f64)
            .sum::<f64>() / self.historical_stats.len() as f64;

        let avg_wait: f64 = self.historical_stats.iter()
            .map(|s| s.avg_acquisition_time_ms)
            .sum::<f64>() / self.historical_stats.len() as f64;

        let mut suggested = current.clone();

        // Adjust max connections based on peak usage
        if peak_active as f64 >= current.max_connections as f64 * 0.9 {
            // Near capacity, increase
            suggested.max_connections = (current.max_connections as f64 * 1.25) as u32;
        } else if peak_active as f64 <= current.max_connections as f64 * 0.5 {
            // Over-provisioned, decrease
            suggested.max_connections = ((peak_active as f64 * 1.5) as u32).max(current.min_connections);
        }

        // Adjust min connections based on average active
        suggested.min_connections = (avg_active * 0.8) as u32;
        suggested.min_connections = suggested.min_connections.max(2).min(suggested.max_connections / 2);

        // Adjust idle timeout based on connection patterns
        if avg_idle > avg_active * 2.0 {
            // Too many idle connections, reduce timeout
            suggested.idle_timeout = (current.idle_timeout as f64 * 0.75) as u64;
        }

        // Adjust acquire timeout based on wait times
        if avg_wait > current.acquire_timeout as f64 * 1000.0 * 0.5 {
            suggested.acquire_timeout = (current.acquire_timeout as f64 * 1.5) as u64;
        }

        suggested
    }
}

/// Read replica pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaPoolConfig {
    /// Primary database pool config
    pub primary: PoolConfig,
    /// Replica pool config (typically larger)
    pub replica: PoolConfig,
    /// Read routing strategy
    pub routing: ReadRoutingStrategy,
    /// Maximum replication lag tolerance (ms)
    pub max_replication_lag_ms: u64,
}

/// Read routing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadRoutingStrategy {
    /// Round-robin across replicas
    RoundRobin,
    /// Route to least-loaded replica
    LeastConnections,
    /// Random selection
    Random,
    /// Sticky sessions
    Sticky,
    /// Prefer primary for consistency
    PreferPrimary,
}

impl Default for ReplicaPoolConfig {
    fn default() -> Self {
        Self {
            primary: PoolConfig::write_optimized(),
            replica: PoolConfig::read_optimized(),
            routing: ReadRoutingStrategy::LeastConnections,
            max_replication_lag_ms: 1000,
        }
    }
}

/// Database connection string builder
pub struct ConnectionStringBuilder {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: Option<String>,
    options: HashMap<String, String>,
}

use std::collections::HashMap as HashMap_;
type HashMap<K, V> = HashMap_<K, V>;

impl ConnectionStringBuilder {
    pub fn new(host: &str, database: &str, user: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 5432,
            database: database.to_string(),
            user: user.to_string(),
            password: None,
            options: HashMap::new(),
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn option(mut self, key: &str, value: &str) -> Self {
        self.options.insert(key.to_string(), value.to_string());
        self
    }

    /// Add SSL mode
    pub fn ssl_mode(self, mode: &str) -> Self {
        self.option("sslmode", mode)
    }

    /// Add application name
    pub fn application_name(self, name: &str) -> Self {
        self.option("application_name", name)
    }

    /// Add statement timeout
    pub fn statement_timeout(self, timeout_ms: u64) -> Self {
        self.option("options", &format!("-c statement_timeout={}", timeout_ms))
    }

    /// Build PostgreSQL connection string
    pub fn build_postgres(&self) -> String {
        let mut url = format!(
            "postgres://{}",
            self.user
        );

        if let Some(ref password) = self.password {
            url.push(':');
            url.push_str(&urlencoding::encode(password));
        }

        url.push('@');
        url.push_str(&self.host);
        url.push(':');
        url.push_str(&self.port.to_string());
        url.push('/');
        url.push_str(&self.database);

        if !self.options.is_empty() {
            url.push('?');
            let opts: Vec<String> = self.options.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            url.push_str(&opts.join("&"));
        }

        url
    }

    /// Build MySQL connection string
    pub fn build_mysql(&self) -> String {
        let mut url = format!(
            "mysql://{}",
            self.user
        );

        if let Some(ref password) = self.password {
            url.push(':');
            url.push_str(&urlencoding::encode(password));
        }

        url.push('@');
        url.push_str(&self.host);
        url.push(':');
        url.push_str(&self.port.to_string());
        url.push('/');
        url.push_str(&self.database);

        url
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert!(config.max_connections > config.min_connections);
        assert!(config.idle_timeout > 0);
    }

    #[test]
    fn test_auto_tune() {
        let config = PoolConfig::auto_tune(100, 4);
        assert!(config.max_connections <= 50);
        assert!(config.min_connections >= 2);
    }

    #[test]
    fn test_pool_health() {
        let config = PoolConfig::default();
        let monitor = PoolMonitor::new(config.clone());

        let stats = PoolStats {
            active_connections: (config.max_connections as f64 * 0.9) as u32,
            idle_connections: 2,
            ..Default::default()
        };

        monitor.update_stats(stats);
        let health = monitor.check_health();

        assert_eq!(health.status, HealthStatus::Warning);
    }

    #[test]
    fn test_connection_string_builder() {
        let conn = ConnectionStringBuilder::new("localhost", "rustpress", "admin")
            .port(5432)
            .password("secret")
            .ssl_mode("require")
            .build_postgres();

        assert!(conn.contains("localhost:5432"));
        assert!(conn.contains("rustpress"));
        assert!(conn.contains("sslmode=require"));
    }
}
