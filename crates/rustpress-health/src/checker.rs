//! Health checker implementation

use crate::status::{ComponentHealth, HealthReport, ServiceHealth};
use crate::system::SystemHealth;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Health checker that performs health checks on various components
pub struct HealthChecker {
    /// Database connection pool
    database: Option<sqlx::PgPool>,

    /// Redis connection
    redis: Option<redis::aio::ConnectionManager>,

    /// External service URLs to check
    external_services: Vec<ExternalService>,

    /// Custom health checks
    custom_checks: Vec<Box<dyn HealthCheck + Send + Sync>>,

    /// Service start time
    started_at: chrono::DateTime<Utc>,

    /// Last successful database check
    last_db_success: Arc<RwLock<Option<chrono::DateTime<Utc>>>>,

    /// Last successful redis check
    last_redis_success: Arc<RwLock<Option<chrono::DateTime<Utc>>>>,

    /// System health checker
    system_health: SystemHealth,

    /// Check timeout
    timeout: Duration,
}

/// External service configuration
#[derive(Clone, Debug)]
pub struct ExternalService {
    /// Service name
    pub name: String,
    /// Health check URL
    pub url: String,
    /// Expected status code
    pub expected_status: u16,
    /// Is critical for overall health
    pub critical: bool,
    /// Timeout for this service
    pub timeout: Duration,
}

/// Trait for custom health checks
#[async_trait::async_trait]
pub trait HealthCheck {
    /// Name of the health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> ComponentHealth;

    /// Is this check critical?
    fn is_critical(&self) -> bool {
        false
    }
}

impl HealthChecker {
    /// Create a new health checker builder
    pub fn builder() -> HealthCheckerBuilder {
        HealthCheckerBuilder::new()
    }

    /// Perform all health checks
    pub async fn check_all(&self) -> HealthReport {
        let start = Instant::now();
        let mut report = HealthReport::new();

        // Set service info
        report.service = ServiceHealth {
            name: "rustpress".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("RUSTPRESS_ENV").unwrap_or_else(|_| "development".to_string()),
            instance_id: std::env::var("HOSTNAME").ok(),
            uptime_seconds: Utc::now()
                .signed_duration_since(self.started_at)
                .num_seconds() as u64,
            started_at: self.started_at,
        };

        // Check database
        if let Some(ref pool) = self.database {
            let health = self.check_database(pool).await;
            report.add_component("database", health);
        }

        // Check Redis
        if let Some(ref redis) = self.redis {
            let health = self.check_redis(redis).await;
            report.add_component("cache", health);
        }

        // Check external services
        for service in &self.external_services {
            let health = self.check_external_service(service).await;
            report.add_component(&service.name, health);
        }

        // Run custom checks
        for check in &self.custom_checks {
            let health = check.check().await;
            report.add_component(check.name(), health);
        }

        // Add system metrics
        report.system = Some(self.system_health.get_metrics());

        // Calculate overall status
        report.calculate_status();

        // Set check duration
        report.check_duration_ms = start.elapsed().as_millis() as u64;
        report.timestamp = Utc::now();

        report
    }

    /// Check database health
    async fn check_database(&self, pool: &sqlx::PgPool) -> ComponentHealth {
        let start = Instant::now();

        let result = tokio::time::timeout(self.timeout, async {
            sqlx::query("SELECT 1")
                .execute(pool)
                .await
        })
        .await;

        let response_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(_)) => {
                // Update last success
                *self.last_db_success.write().await = Some(Utc::now());

                // Get connection pool stats
                let pool_size = pool.size();
                let idle = pool.num_idle();

                let mut health = ComponentHealth::healthy()
                    .with_type("postgresql")
                    .with_response_time(response_time)
                    .with_detail("pool_size", pool_size)
                    .with_detail("idle_connections", idle);

                // Check for degraded state
                if response_time > 1000 {
                    health = ComponentHealth::degraded("Slow response time")
                        .with_type("postgresql")
                        .with_response_time(response_time);
                }

                health
            }
            Ok(Err(e)) => {
                warn!("Database health check failed: {}", e);
                ComponentHealth::unhealthy(format!("Query failed: {}", e))
                    .with_type("postgresql")
                    .with_response_time(response_time)
            }
            Err(_) => {
                warn!("Database health check timed out");
                ComponentHealth::unhealthy("Connection timeout")
                    .with_type("postgresql")
            }
        }
    }

    /// Check Redis health
    async fn check_redis(&self, redis: &redis::aio::ConnectionManager) -> ComponentHealth {
        let start = Instant::now();
        let mut conn = redis.clone();

        let result = tokio::time::timeout(self.timeout, async {
            redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
        })
        .await;

        let response_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(response)) if response == "PONG" => {
                // Update last success
                *self.last_redis_success.write().await = Some(Utc::now());

                // Get Redis info
                let info_result: Result<String, _> = redis::cmd("INFO")
                    .arg("server")
                    .query_async(&mut conn)
                    .await;

                let mut health = ComponentHealth::healthy()
                    .with_type("redis")
                    .with_response_time(response_time);

                if let Ok(info) = info_result {
                    // Parse version from info
                    for line in info.lines() {
                        if line.starts_with("redis_version:") {
                            if let Some(version) = line.split(':').nth(1) {
                                health = health.with_detail("version", version.trim());
                            }
                        }
                    }
                }

                health
            }
            Ok(Ok(_)) => {
                ComponentHealth::degraded("Unexpected PING response")
                    .with_type("redis")
                    .with_response_time(response_time)
            }
            Ok(Err(e)) => {
                warn!("Redis health check failed: {}", e);
                ComponentHealth::unhealthy(format!("Command failed: {}", e))
                    .with_type("redis")
                    .with_response_time(response_time)
            }
            Err(_) => {
                warn!("Redis health check timed out");
                ComponentHealth::unhealthy("Connection timeout")
                    .with_type("redis")
            }
        }
    }

    /// Check external service health
    async fn check_external_service(&self, service: &ExternalService) -> ComponentHealth {
        let start = Instant::now();

        let client = reqwest::Client::builder()
            .timeout(service.timeout)
            .build()
            .unwrap_or_default();

        let result = client.get(&service.url).send().await;
        let response_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status = response.status().as_u16();

                if status == service.expected_status {
                    ComponentHealth::healthy()
                        .with_type("external")
                        .with_response_time(response_time)
                        .with_detail("url", &service.url)
                        .with_detail("status_code", status)
                } else {
                    ComponentHealth::unhealthy(format!(
                        "Unexpected status code: {} (expected {})",
                        status, service.expected_status
                    ))
                    .with_type("external")
                    .with_response_time(response_time)
                }
            }
            Err(e) => {
                warn!("External service check failed for {}: {}", service.name, e);
                ComponentHealth::unhealthy(format!("Request failed: {}", e))
                    .with_type("external")
            }
        }
    }

    /// Quick liveness check (is the application alive?)
    pub async fn check_liveness(&self) -> bool {
        // Application is alive if we can respond
        true
    }

    /// Readiness check (is the application ready to accept traffic?)
    pub async fn check_readiness(&self) -> bool {
        // Check database connection
        if let Some(ref pool) = self.database {
            let result = tokio::time::timeout(Duration::from_secs(5), async {
                sqlx::query("SELECT 1").execute(pool).await
            })
            .await;

            if result.is_err() || result.unwrap().is_err() {
                debug!("Readiness check failed: database unavailable");
                return false;
            }
        }

        // Check Redis connection
        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            let result = tokio::time::timeout(Duration::from_secs(3), async {
                redis::cmd("PING")
                    .query_async::<_, String>(&mut conn)
                    .await
            })
            .await;

            if result.is_err() || result.unwrap().is_err() {
                debug!("Readiness check failed: cache unavailable");
                return false;
            }
        }

        true
    }

    /// Startup check (has the application finished starting?)
    pub async fn check_startup(&self) -> bool {
        // Check if we've been running for at least a few seconds
        let uptime = Utc::now()
            .signed_duration_since(self.started_at)
            .num_seconds();

        if uptime < 2 {
            return false;
        }

        // Verify critical services
        self.check_readiness().await
    }
}

/// Builder for HealthChecker
pub struct HealthCheckerBuilder {
    database: Option<sqlx::PgPool>,
    redis: Option<redis::aio::ConnectionManager>,
    external_services: Vec<ExternalService>,
    custom_checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
    timeout: Duration,
}

impl HealthCheckerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            database: None,
            redis: None,
            external_services: Vec::new(),
            custom_checks: Vec::new(),
            timeout: Duration::from_secs(10),
        }
    }

    /// Set database connection pool
    pub fn with_database(mut self, pool: sqlx::PgPool) -> Self {
        self.database = Some(pool);
        self
    }

    /// Set Redis connection
    pub fn with_redis(mut self, conn: redis::aio::ConnectionManager) -> Self {
        self.redis = Some(conn);
        self
    }

    /// Add external service to check
    pub fn with_external_service(mut self, service: ExternalService) -> Self {
        self.external_services.push(service);
        self
    }

    /// Add custom health check
    pub fn with_custom_check<C: HealthCheck + Send + Sync + 'static>(mut self, check: C) -> Self {
        self.custom_checks.push(Box::new(check));
        self
    }

    /// Set check timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the health checker
    pub fn build(self) -> HealthChecker {
        HealthChecker {
            database: self.database,
            redis: self.redis,
            external_services: self.external_services,
            custom_checks: self.custom_checks,
            started_at: Utc::now(),
            last_db_success: Arc::new(RwLock::new(None)),
            last_redis_success: Arc::new(RwLock::new(None)),
            system_health: SystemHealth::new(),
            timeout: self.timeout,
        }
    }
}

impl Default for HealthCheckerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Implement async_trait for HealthCheck
// This is a workaround since we can't use async_trait in the trait definition directly
// in this example. In a real implementation, add async-trait to dependencies.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let checker = HealthChecker::builder()
            .with_timeout(Duration::from_secs(5))
            .build();

        assert!(checker.database.is_none());
        assert!(checker.redis.is_none());
    }

    #[test]
    fn test_external_service() {
        let service = ExternalService {
            name: "test".to_string(),
            url: "http://localhost/health".to_string(),
            expected_status: 200,
            critical: false,
            timeout: Duration::from_secs(5),
        };

        assert_eq!(service.name, "test");
        assert!(!service.critical);
    }
}
