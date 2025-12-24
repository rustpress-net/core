//! RustPress Server Entry Point
//!
//! This is the main entry point for the RustPress CMS server.
//! It initializes all components and starts the HTTP server.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rustpress_auth::{JwtConfig, JwtManager, PermissionChecker};
use rustpress_cache::{Cache, CacheConfig, MemoryBackend};
use rustpress_core::config::AppConfig;
use rustpress_core::hook::HookRegistry;
use rustpress_core::plugin::PluginManager;
use rustpress_database::{DatabasePool, PoolConfig};
use rustpress_events::EventBus;
use rustpress_jobs::JobQueue;
use rustpress_storage::{LocalBackend, Storage, StorageConfig};

use rustpress_server::state::AppState;
use rustpress_server::App;

/// Environment variable names
mod env_vars {
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const SERVER_HOST: &str = "RUSTPRESS_HOST";
    pub const SERVER_PORT: &str = "RUSTPRESS_PORT";
    pub const JWT_SECRET: &str = "JWT_SECRET";
    pub const STORAGE_PATH: &str = "STORAGE_PATH";
    pub const THEMES_PATH: &str = "THEMES_PATH";
    pub const CACHE_MAX_CAPACITY: &str = "CACHE_MAX_CAPACITY";
    pub const LOG_LEVEL: &str = "RUST_LOG";
}

/// Initialize the tracing/logging subsystem
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpress=info,tower_http=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load configuration from environment variables with defaults
fn load_config() -> AppConfig {
    let mut config = AppConfig::default();

    // Server configuration
    if let Ok(host) = env::var(env_vars::SERVER_HOST) {
        config.server.host = host;
    }
    if let Ok(port) = env::var(env_vars::SERVER_PORT) {
        if let Ok(port) = port.parse() {
            config.server.port = port;
        }
    }

    // Database configuration
    if let Ok(url) = env::var(env_vars::DATABASE_URL) {
        config.database.url = url;
    }

    // Auth configuration
    if let Ok(secret) = env::var(env_vars::JWT_SECRET) {
        config.auth.jwt_secret = secret;
    } else {
        warn!("JWT_SECRET not set, using default (not recommended for production)");
    }

    // Storage configuration
    if let Ok(path) = env::var(env_vars::STORAGE_PATH) {
        config.storage.local_path = PathBuf::from(path);
    }

    config
}

/// Initialize the database connection pool
async fn init_database(config: &AppConfig) -> Result<DatabasePool, Box<dyn std::error::Error>> {
    info!("Connecting to database...");

    let pool_config = PoolConfig::from(config.database.clone());
    let pool = DatabasePool::new(pool_config).await?;

    // Verify connection
    pool.health_check().await?;
    info!("Database connection established");

    Ok(pool)
}

/// Initialize the cache subsystem
fn init_cache(config: &AppConfig) -> Cache {
    let max_capacity = env::var(env_vars::CACHE_MAX_CAPACITY)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000);

    let backend = Arc::new(MemoryBackend::with_ttl(
        max_capacity,
        Duration::from_secs(config.cache.default_ttl_secs),
    ));

    let cache_config = CacheConfig {
        default_ttl: Duration::from_secs(config.cache.default_ttl_secs),
        prefix: Some("rustpress".to_string()),
        enable_metrics: config.cache.enable_metrics,
    };

    info!(max_capacity = max_capacity, "Cache initialized");
    Cache::with_config(backend, cache_config)
}

/// Initialize the event bus
fn init_event_bus() -> EventBus {
    info!("Event bus initialized");
    EventBus::new()
}

/// Initialize the job queue
fn init_job_queue(pool: &DatabasePool) -> JobQueue {
    info!("Job queue initialized");
    JobQueue::new(pool.inner().clone())
}

/// Initialize the storage subsystem
fn init_storage(config: &AppConfig) -> Storage {
    let backend = Arc::new(
        LocalBackend::new(&config.storage.local_path)
            .with_base_url("/uploads"),
    );

    let storage_config = StorageConfig {
        max_upload_size: config.storage.max_upload_size as u64,
        allowed_types: config.storage.allowed_types.clone(),
        ..Default::default()
    };

    info!(path = ?config.storage.local_path, "Storage initialized");
    Storage::with_config(backend, storage_config)
}

/// Initialize the JWT manager
fn init_jwt(config: &AppConfig) -> JwtManager {
    let jwt_config = JwtConfig {
        secret: config.auth.jwt_secret.clone(),
        issuer: config.auth.jwt_issuer.clone(),
        access_expiry_secs: config.auth.jwt_access_expiry_secs as i64,
        refresh_expiry_secs: config.auth.jwt_refresh_expiry_secs as i64,
    };

    info!("JWT manager initialized");
    JwtManager::new(jwt_config)
}

/// Build the application state with all initialized components
fn build_app_state(
    config: AppConfig,
    database: DatabasePool,
    cache: Cache,
    event_bus: EventBus,
    job_queue: JobQueue,
    storage: Storage,
    jwt: JwtManager,
) -> Result<AppState, &'static str> {
    let themes_dir = env::var(env_vars::THEMES_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./themes"));

    AppState::builder()
        .config(config)
        .database(database)
        .cache(cache)
        .event_bus(event_bus)
        .job_queue(job_queue)
        .storage(storage)
        .jwt(jwt)
        .permissions(PermissionChecker::default())
        .hooks(HookRegistry::new())
        .plugins(PluginManager::new())
        .themes_dir(themes_dir)
        .build()
}

/// Ensure required directories exist
async fn ensure_directories(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create storage directory
    tokio::fs::create_dir_all(&config.storage.local_path).await?;
    info!(path = ?config.storage.local_path, "Storage directory ready");

    // Create themes directory
    let themes_dir = env::var(env_vars::THEMES_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./themes"));
    tokio::fs::create_dir_all(&themes_dir).await?;
    info!(path = ?themes_dir, "Themes directory ready");

    Ok(())
}

/// Print startup banner
fn print_banner() {
    println!(r#"
  ____           _   ____
 |  _ \ _   _ __| |_|  _ \ _ __ ___  ___ ___
 | |_) | | | / _` __| |_) | '__/ _ \/ __/ __|
 |  _ <| |_| \__ \ |_|  __/| | |  __/\__ \__ \
 |_| \_\\__,_|___/\__|_|   |_|  \___||___/___/

    "#);
    println!("  RustPress CMS - A Modern WordPress Alternative in Rust");
    println!("  Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing first
    init_tracing();

    // Print startup banner
    print_banner();

    info!("Starting RustPress CMS Server");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config();
    info!(host = %config.server.host, port = config.server.port, "Configuration loaded");

    // Ensure required directories exist
    ensure_directories(&config).await?;

    // Initialize components
    let database = match init_database(&config).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            error!("Make sure PostgreSQL is running and DATABASE_URL is set correctly");
            error!("Example: DATABASE_URL=postgres://user:pass@localhost/rustpress");
            return Err(e);
        }
    };

    let cache = init_cache(&config);
    let event_bus = init_event_bus();
    let job_queue = init_job_queue(&database);
    let storage = init_storage(&config);
    let jwt = init_jwt(&config);

    // Build application state
    let state = build_app_state(
        config.clone(),
        database,
        cache,
        event_bus,
        job_queue,
        storage,
        jwt,
    )?;

    // Auto-scan themes on startup
    info!("Scanning themes directory...");
    match state.theme_manager().scan_themes().await {
        Ok(result) => {
            info!(
                scanned = result.scanned,
                registered = result.registered,
                updated = result.updated,
                "Themes scan complete"
            );
            if !result.errors.is_empty() {
                for error in &result.errors {
                    warn!("Theme scan error: {}", error);
                }
            }
        }
        Err(e) => {
            warn!("Failed to scan themes: {}", e);
        }
    }

    // Create server address
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    // Print startup info
    info!("=================================================");
    info!("RustPress server starting on http://{}", addr);
    info!("API endpoint: http://{}/api/v1", addr);
    info!("Admin panel: http://{}/admin", addr);
    info!("Health check: http://{}/health", addr);
    info!("Metrics: http://{}/metrics", addr);
    info!("=================================================");

    // Create and run the application
    let app = App::new(state)
        .with_shutdown_timeout(Duration::from_secs(config.server.shutdown_timeout_secs));

    // Run the server
    if let Err(e) = app.run(addr).await {
        error!("Server error: {}", e);
        return Err(e);
    }

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_defaults() {
        let config = load_config();
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
    }

    #[test]
    fn test_jwt_config() {
        let config = load_config();
        let jwt = init_jwt(&config);
        assert!(jwt.config().access_expiry_secs > 0);
    }
}
