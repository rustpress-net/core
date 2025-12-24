//! Database Providers for RustPress
//!
//! This module contains integrations for various database providers,
//! including managed cloud databases and self-hosted options.
//!
//! # Supported Providers
//!
//! ## PostgreSQL-based
//! - `postgres` - Self-hosted PostgreSQL
//! - `supabase` - Supabase managed PostgreSQL
//! - `neon` - Neon serverless PostgreSQL
//! - `cockroachdb` - CockroachDB distributed SQL
//!
//! ## MySQL-based
//! - `mysql` - Self-hosted MySQL/MariaDB
//! - `planetscale` - PlanetScale serverless MySQL
//! - `tidb` - TiDB distributed SQL
//!
//! ## Embedded
//! - `sqlite` - SQLite embedded database
//! - `libsql` - libSQL/Turso edge database
//!
//! # Usage
//!
//! ```yaml
//! database:
//!   provider: supabase
//!   project_url: https://xxx.supabase.co
//!   anon_key: your-key
//!   database_password: your-password
//! ```

pub mod planetscale;
pub mod postgres;
pub mod supabase;

// Re-exports
pub use planetscale::{PlanetScaleConfig, PlanetScaleProvider};
pub use postgres::{PostgresConfig as FullPostgresConfig, PostgresProvider, DatabaseStats};
pub use supabase::{SupabaseConfig, SupabaseProvider};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Database error types
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),
}

/// Result type for database operations
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Database provider trait
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;

    /// Get the connection pool (if connected)
    fn pool(&self) -> Option<&sqlx::PgPool>;

    /// Check if connected and healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get provider name
    fn provider_name(&self) -> &str;
}

/// Universal database configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum DatabaseConfig {
    /// PostgreSQL (self-hosted)
    #[serde(rename = "postgres")]
    Postgres(PostgresConfig),

    /// Supabase managed PostgreSQL
    #[serde(rename = "supabase")]
    Supabase(SupabaseConfig),

    /// PlanetScale serverless MySQL
    #[serde(rename = "planetscale")]
    PlanetScale(PlanetScaleConfig),

    /// SQLite embedded
    #[serde(rename = "sqlite")]
    Sqlite(SqliteConfig),

    /// MySQL (self-hosted)
    #[serde(rename = "mysql")]
    Mysql(MysqlConfig),
}

/// PostgreSQL configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    #[serde(default)]
    pub ssl_mode: Option<String>,
    #[serde(default)]
    pub pool_size: Option<u32>,
}

/// SQLite configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
    #[serde(default)]
    pub create_if_missing: bool,
    #[serde(default)]
    pub busy_timeout: Option<u64>,
}

/// MySQL configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    #[serde(default)]
    pub ssl_mode: Option<String>,
    #[serde(default)]
    pub pool_size: Option<u32>,
}

/// Database factory for creating providers
pub struct DatabaseFactory;

impl DatabaseFactory {
    /// Create a database provider from configuration
    pub fn create(config: &DatabaseConfig) -> Result<Box<dyn DatabaseProvider>> {
        match config {
            DatabaseConfig::Postgres(cfg) => {
                Ok(Box::new(PostgresProvider::new(postgres::PostgresConfig {
                    host: cfg.host.clone(),
                    port: cfg.port,
                    username: cfg.username.clone(),
                    password: cfg.password.clone(),
                    database: cfg.database.clone(),
                    ssl_mode: cfg.ssl_mode.clone().unwrap_or_else(|| "prefer".to_string()),
                    pool: postgres::PoolConfig {
                        max_connections: cfg.pool_size.unwrap_or(10),
                        ..Default::default()
                    },
                    ..Default::default()
                })))
            }
            DatabaseConfig::Supabase(cfg) => {
                Ok(Box::new(SupabaseProvider::new(cfg.clone())))
            }
            DatabaseConfig::PlanetScale(cfg) => {
                Ok(Box::new(PlanetScaleProvider::new(cfg.clone())))
            }
            DatabaseConfig::Sqlite(_) => Err(DatabaseError::UnsupportedProvider(
                "SQLite provider not yet implemented".to_string(),
            )),
            DatabaseConfig::Mysql(_) => Err(DatabaseError::UnsupportedProvider(
                "MySQL provider not yet implemented".to_string(),
            )),
        }
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Box<dyn DatabaseProvider>> {
        let provider = std::env::var("DATABASE_PROVIDER")
            .unwrap_or_else(|_| "postgres".to_string());

        match provider.as_str() {
            "postgres" | "postgresql" => {
                // Check if DATABASE_URL is set (common pattern)
                if let Ok(url) = std::env::var("DATABASE_URL") {
                    return Ok(Box::new(PostgresProvider::from_url(&url)?));
                }
                // Otherwise use individual env vars
                Ok(Box::new(PostgresProvider::from_env()?))
            }
            "supabase" => {
                let config = SupabaseConfig {
                    project_url: std::env::var("SUPABASE_URL")
                        .map_err(|_| DatabaseError::Configuration(
                            "SUPABASE_URL not set".to_string()
                        ))?,
                    anon_key: std::env::var("SUPABASE_ANON_KEY")
                        .map_err(|_| DatabaseError::Configuration(
                            "SUPABASE_ANON_KEY not set".to_string()
                        ))?,
                    service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok(),
                    database_password: std::env::var("SUPABASE_DB_PASSWORD")
                        .map_err(|_| DatabaseError::Configuration(
                            "SUPABASE_DB_PASSWORD not set".to_string()
                        ))?,
                    use_pooler: std::env::var("SUPABASE_USE_POOLER")
                        .map(|v| v == "true")
                        .unwrap_or(true),
                    pooler_mode: std::env::var("SUPABASE_POOLER_MODE")
                        .unwrap_or_else(|_| "transaction".to_string()),
                    pool: supabase::PoolConfig::default(),
                };
                Ok(Box::new(SupabaseProvider::new(config)))
            }
            "planetscale" => {
                let config = PlanetScaleConfig {
                    host: std::env::var("PLANETSCALE_HOST")
                        .unwrap_or_else(|_| "aws.connect.psdb.cloud".to_string()),
                    username: std::env::var("PLANETSCALE_USERNAME")
                        .map_err(|_| DatabaseError::Configuration(
                            "PLANETSCALE_USERNAME not set".to_string()
                        ))?,
                    password: std::env::var("PLANETSCALE_PASSWORD")
                        .map_err(|_| DatabaseError::Configuration(
                            "PLANETSCALE_PASSWORD not set".to_string()
                        ))?,
                    database: std::env::var("PLANETSCALE_DATABASE")
                        .map_err(|_| DatabaseError::Configuration(
                            "PLANETSCALE_DATABASE not set".to_string()
                        ))?,
                    use_ssl: true,
                    pool: planetscale::PoolConfig::default(),
                    branch: std::env::var("PLANETSCALE_BRANCH").ok(),
                };
                Ok(Box::new(PlanetScaleProvider::new(config)))
            }
            _ => Err(DatabaseError::UnsupportedProvider(format!(
                "Unknown provider '{}'. Supported: postgres, supabase, planetscale",
                provider
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_parsing() {
        let yaml = r#"
provider: supabase
project_url: https://test.supabase.co
anon_key: test-key
database_password: test-pass
"#;

        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        match config {
            DatabaseConfig::Supabase(cfg) => {
                assert_eq!(cfg.project_url, "https://test.supabase.co");
            }
            _ => panic!("Expected Supabase config"),
        }
    }
}
