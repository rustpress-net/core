//! Database connection pool management.

use rustpress_core::error::{Error, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres};
use std::sync::Arc;
use std::time::Duration;

/// Configuration for database pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub url: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/rustpress".to_string(),
            min_connections: 2,
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

impl From<rustpress_core::config::DatabaseConfig> for PoolConfig {
    fn from(config: rustpress_core::config::DatabaseConfig) -> Self {
        // Extract timeouts first to avoid borrow after move
        let connect_timeout = config.connect_timeout();
        let idle_timeout = config.idle_timeout();
        let max_lifetime = config.max_lifetime();

        Self {
            url: config.url,
            min_connections: config.pool_min,
            max_connections: config.pool_max,
            connect_timeout,
            idle_timeout,
            max_lifetime,
        }
    }
}

/// Database pool wrapper
#[derive(Clone)]
pub struct DatabasePool {
    pool: PgPool,
    config: Arc<PoolConfig>,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(config: PoolConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.url)
            .await
            .map_err(|e| Error::database_with_source("Failed to create database pool", e))?;

        tracing::info!(
            min = config.min_connections,
            max = config.max_connections,
            "Database pool created"
        );

        Ok(Self {
            pool,
            config: Arc::new(config),
        })
    }

    /// Get a reference to the underlying pool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max: self.config.max_connections,
        }
    }

    /// Acquire a connection from the pool
    pub async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<Postgres>> {
        self.pool
            .acquire()
            .await
            .map_err(|e| Error::database_with_source("Failed to acquire connection", e))
    }

    /// Check if the database is healthy
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::database_with_source("Health check failed", e))?;
        Ok(())
    }

    /// Check if the database is connected (returns bool)
    pub async fn is_connected(&self) -> bool {
        self.health_check().await.is_ok()
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database pool closed");
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, Postgres>> {
        self.pool
            .begin()
            .await
            .map_err(|e| Error::database_with_source("Failed to begin transaction", e))
    }
}

impl std::ops::Deref for DatabasePool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub max: u32,
}

impl PoolStats {
    pub fn utilization(&self) -> f64 {
        if self.max == 0 {
            0.0
        } else {
            (self.size - self.idle as u32) as f64 / self.max as f64
        }
    }
}

/// Database executor trait for abstraction
pub trait DatabaseExecutor: Send + Sync {
    fn pool(&self) -> &PgPool;
}

impl DatabaseExecutor for DatabasePool {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            size: 10,
            idle: 5,
            max: 20,
        };
        assert_eq!(stats.utilization(), 0.25);
    }
}
