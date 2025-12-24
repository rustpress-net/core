//! PostgreSQL Database Provider
//!
//! Self-hosted PostgreSQL database integration.
//! Supports connection pooling, SSL, and all standard PostgreSQL features.
//!
//! # Configuration
//!
//! ```yaml
//! database:
//!   provider: postgres
//!   host: localhost
//!   port: 5432
//!   username: postgres
//!   password: your-password
//!   database: rustpress
//!   ssl_mode: prefer  # disable, prefer, require
//!   pool_size: 10
//! ```
//!
//! Or via environment variables:
//! ```bash
//! DATABASE_PROVIDER=postgres
//! POSTGRES_HOST=localhost
//! POSTGRES_PORT=5432
//! POSTGRES_USER=postgres
//! POSTGRES_PASSWORD=password
//! POSTGRES_DB=rustpress
//! POSTGRES_SSL_MODE=prefer
//! POSTGRES_POOL_SIZE=10
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::{DatabaseError, DatabaseProvider, Result};

/// PostgreSQL configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database host
    #[serde(default = "default_host")]
    pub host: String,

    /// Database port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database username
    #[serde(default = "default_username")]
    pub username: String,

    /// Database password
    pub password: String,

    /// Database name
    #[serde(default = "default_database")]
    pub database: String,

    /// SSL mode: disable, prefer, require
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,

    /// Connection pool settings
    #[serde(default)]
    pub pool: PoolConfig,

    /// Application name for connection identification
    #[serde(default = "default_application_name")]
    pub application_name: String,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,

    /// Statement timeout in seconds (0 = no timeout)
    #[serde(default)]
    pub statement_timeout: u64,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    5432
}

fn default_username() -> String {
    "postgres".to_string()
}

fn default_database() -> String {
    "rustpress".to_string()
}

fn default_ssl_mode() -> String {
    "prefer".to_string()
}

fn default_application_name() -> String {
    "rustpress".to_string()
}

fn default_connect_timeout() -> u64 {
    30
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            username: default_username(),
            password: String::new(),
            database: default_database(),
            ssl_mode: default_ssl_mode(),
            pool: PoolConfig::default(),
            application_name: default_application_name(),
            connect_timeout: default_connect_timeout(),
            statement_timeout: 0,
        }
    }
}

impl PostgresConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("POSTGRES_HOST").unwrap_or_else(|_| default_host()),
            port: std::env::var("POSTGRES_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(default_port()),
            username: std::env::var("POSTGRES_USER").unwrap_or_else(|_| default_username()),
            password: std::env::var("POSTGRES_PASSWORD").map_err(|_| {
                DatabaseError::Configuration("POSTGRES_PASSWORD not set".to_string())
            })?,
            database: std::env::var("POSTGRES_DB").unwrap_or_else(|_| default_database()),
            ssl_mode: std::env::var("POSTGRES_SSL_MODE").unwrap_or_else(|_| default_ssl_mode()),
            pool: PoolConfig {
                max_connections: std::env::var("POSTGRES_POOL_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
                ..Default::default()
            },
            application_name: std::env::var("POSTGRES_APP_NAME")
                .unwrap_or_else(|_| default_application_name()),
            connect_timeout: std::env::var("POSTGRES_CONNECT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default_connect_timeout()),
            statement_timeout: std::env::var("POSTGRES_STATEMENT_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
    }

    /// Create configuration from a database URL
    pub fn from_url(url: &str) -> Result<Self> {
        // Parse URL like: postgres://user:password@host:port/database?sslmode=prefer
        let parsed = url::Url::parse(url)
            .map_err(|e| DatabaseError::Configuration(format!("Invalid database URL: {}", e)))?;

        let host = parsed.host_str().unwrap_or("localhost").to_string();
        let port = parsed.port().unwrap_or(5432);
        let username = parsed.username().to_string();
        let password = parsed
            .password()
            .map(|p| urlencoding::decode(p).unwrap_or_default().to_string())
            .unwrap_or_default();
        let database = parsed.path().trim_start_matches('/').to_string();

        let ssl_mode = parsed
            .query_pairs()
            .find(|(k, _)| k == "sslmode")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(default_ssl_mode);

        Ok(Self {
            host,
            port,
            username,
            password,
            database,
            ssl_mode,
            ..Default::default()
        })
    }

    /// Build connection URL
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.username,
            urlencoding::encode(&self.password),
            self.host,
            self.port,
            self.database,
            self.ssl_mode
        )
    }
}

/// Connection pool configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Maximum connections in pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Connection acquire timeout in seconds
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout: u64,

    /// Idle connection timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// Maximum connection lifetime in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
}

fn default_min_connections() -> u32 {
    1
}

fn default_max_connections() -> u32 {
    10
}

fn default_acquire_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_max_lifetime() -> u64 {
    1800
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: default_min_connections(),
            max_connections: default_max_connections(),
            acquire_timeout: default_acquire_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
        }
    }
}

/// PostgreSQL database provider
pub struct PostgresProvider {
    config: PostgresConfig,
    pool: Option<PgPool>,
}

impl PostgresProvider {
    /// Create a new PostgreSQL provider
    pub fn new(config: PostgresConfig) -> Self {
        Self { config, pool: None }
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self::new(PostgresConfig::from_env()?))
    }

    /// Create provider from a database URL
    pub fn from_url(url: &str) -> Result<Self> {
        Ok(Self::new(PostgresConfig::from_url(url)?))
    }

    /// Get the connection pool (if connected)
    pub fn get_pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    /// Parse SSL mode string to PgSslMode
    fn parse_ssl_mode(mode: &str) -> PgSslMode {
        match mode.to_lowercase().as_str() {
            "disable" | "disabled" | "off" | "false" => PgSslMode::Disable,
            "require" | "required" | "on" | "true" => PgSslMode::Require,
            "verify-ca" => PgSslMode::VerifyCa,
            "verify-full" => PgSslMode::VerifyFull,
            _ => PgSslMode::Prefer, // Default to prefer
        }
    }

    /// Build connection options
    fn build_connect_options(&self) -> PgConnectOptions {
        let ssl_mode = Self::parse_ssl_mode(&self.config.ssl_mode);

        let mut options = PgConnectOptions::new()
            .host(&self.config.host)
            .port(self.config.port)
            .username(&self.config.username)
            .password(&self.config.password)
            .database(&self.config.database)
            .ssl_mode(ssl_mode)
            .application_name(&self.config.application_name);

        // Set statement timeout if specified
        if self.config.statement_timeout > 0 {
            options = options.options([
                ("statement_timeout", format!("{}s", self.config.statement_timeout)),
            ]);
        }

        options
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::Connection("Not connected to database".to_string())
        })?;

        // Get connection pool stats
        let pool_size = pool.size();
        let idle_connections = pool.num_idle();

        // Get database size
        let db_size: Option<i64> = sqlx::query_scalar(
            "SELECT pg_database_size(current_database())"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Get connection count
        let active_connections: Option<i64> = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database()"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| DatabaseError::Query(e.to_string()))?;

        // Get PostgreSQL version
        let version: Option<String> = sqlx::query_scalar("SELECT version()")
            .fetch_optional(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(DatabaseStats {
            pool_size,
            idle_connections,
            database_size_bytes: db_size.unwrap_or(0),
            active_connections: active_connections.unwrap_or(0) as u32,
            version: version.unwrap_or_else(|| "Unknown".to_string()),
        })
    }

    /// Run a raw SQL query (for admin operations)
    pub async fn execute_raw(&self, sql: &str) -> Result<u64> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::Connection("Not connected to database".to_string())
        })?;

        let result = sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Vacuum the database (cleanup dead tuples)
    pub async fn vacuum(&self, full: bool) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::Connection("Not connected to database".to_string())
        })?;

        let sql = if full { "VACUUM FULL" } else { "VACUUM" };

        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        info!("Database vacuum completed (full={})", full);
        Ok(())
    }

    /// Analyze the database (update statistics)
    pub async fn analyze(&self) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::Connection("Not connected to database".to_string())
        })?;

        sqlx::query("ANALYZE")
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        info!("Database analyze completed");
        Ok(())
    }

    /// Reindex the database
    pub async fn reindex(&self) -> Result<()> {
        let pool = self.pool.as_ref().ok_or_else(|| {
            DatabaseError::Connection("Not connected to database".to_string())
        })?;

        let db_name = &self.config.database;
        sqlx::query(&format!("REINDEX DATABASE \"{}\"", db_name))
            .execute(pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;

        info!("Database reindex completed");
        Ok(())
    }
}

#[async_trait]
impl DatabaseProvider for PostgresProvider {
    async fn connect(&mut self) -> Result<()> {
        info!(
            "Connecting to PostgreSQL at {}:{}...",
            self.config.host, self.config.port
        );

        let options = self.build_connect_options();

        let pool = PgPoolOptions::new()
            .min_connections(self.config.pool.min_connections)
            .max_connections(self.config.pool.max_connections)
            .acquire_timeout(Duration::from_secs(self.config.pool.acquire_timeout))
            .idle_timeout(Duration::from_secs(self.config.pool.idle_timeout))
            .max_lifetime(Duration::from_secs(self.config.pool.max_lifetime))
            .connect_with(options)
            .await
            .map_err(|e| DatabaseError::Connection(format!("Failed to connect: {}", e)))?;

        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DatabaseError::Connection(format!("Connection test failed: {}", e)))?;

        self.pool = Some(pool);

        info!(
            "Connected to PostgreSQL database '{}' successfully",
            self.config.database
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            info!("Closing PostgreSQL connection pool...");
            pool.close().await;
            info!("PostgreSQL connection pool closed");
        }
        Ok(())
    }

    fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }

    async fn health_check(&self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => {
                    debug!("PostgreSQL health check passed");
                    Ok(true)
                }
                Err(e) => {
                    warn!("PostgreSQL health check failed: {}", e);
                    Ok(false)
                }
            }
        } else {
            debug!("PostgreSQL health check: not connected");
            Ok(false)
        }
    }

    fn provider_name(&self) -> &str {
        "postgres"
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStats {
    /// Number of connections in the pool
    pub pool_size: u32,
    /// Number of idle connections
    pub idle_connections: u32,
    /// Database size in bytes
    pub database_size_bytes: i64,
    /// Active connections to the database
    pub active_connections: u32,
    /// PostgreSQL version
    pub version: String,
}

impl DatabaseStats {
    /// Get human-readable database size
    pub fn database_size_human(&self) -> String {
        let bytes = self.database_size_bytes as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.2} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.2} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else {
            format!("{} bytes", self.database_size_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PostgresConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.username, "postgres");
        assert_eq!(config.database, "rustpress");
        assert_eq!(config.ssl_mode, "prefer");
    }

    #[test]
    fn test_config_from_url() {
        let url = "postgres://myuser:mypass@myhost:5433/mydb?sslmode=require";
        let config = PostgresConfig::from_url(url).unwrap();

        assert_eq!(config.host, "myhost");
        assert_eq!(config.port, 5433);
        assert_eq!(config.username, "myuser");
        assert_eq!(config.password, "mypass");
        assert_eq!(config.database, "mydb");
        assert_eq!(config.ssl_mode, "require");
    }

    #[test]
    fn test_config_from_url_with_special_chars() {
        let url = "postgres://user:p%40ss%3Aword@host:5432/db";
        let config = PostgresConfig::from_url(url).unwrap();

        assert_eq!(config.password, "p@ss:word");
    }

    #[test]
    fn test_connection_url() {
        let config = PostgresConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            database: "testdb".to_string(),
            ssl_mode: "require".to_string(),
            ..Default::default()
        };

        let url = config.connection_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("localhost:5432"));
        assert!(url.contains("testdb"));
        assert!(url.contains("sslmode=require"));
    }

    #[test]
    fn test_ssl_mode_parsing() {
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("disable"),
            PgSslMode::Disable
        ));
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("require"),
            PgSslMode::Require
        ));
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("prefer"),
            PgSslMode::Prefer
        ));
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("verify-ca"),
            PgSslMode::VerifyCa
        ));
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("verify-full"),
            PgSslMode::VerifyFull
        ));
        assert!(matches!(
            PostgresProvider::parse_ssl_mode("unknown"),
            PgSslMode::Prefer
        ));
    }

    #[test]
    fn test_provider_name() {
        let config = PostgresConfig::default();
        let provider = PostgresProvider::new(config);
        assert_eq!(provider.provider_name(), "postgres");
    }

    #[test]
    fn test_database_stats_human_size() {
        let stats = DatabaseStats {
            pool_size: 10,
            idle_connections: 5,
            database_size_bytes: 1_500_000_000,
            active_connections: 3,
            version: "PostgreSQL 15.0".to_string(),
        };

        assert!(stats.database_size_human().contains("GB"));

        let small_stats = DatabaseStats {
            database_size_bytes: 500_000,
            ..stats.clone()
        };
        assert!(small_stats.database_size_human().contains("KB"));
    }
}
