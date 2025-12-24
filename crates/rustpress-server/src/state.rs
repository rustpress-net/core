//! Application state management.

use rustpress_auth::{JwtManager, PermissionChecker};
use rustpress_cache::Cache;
use rustpress_core::config::AppConfig;
use rustpress_core::hook::HookRegistry;
use rustpress_core::plugin::PluginManager;
use rustpress_database::{DatabasePool, pool::DatabaseExecutor};
use rustpress_events::EventBus;
use rustpress_jobs::JobQueue;
use rustpress_storage::Storage;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::{ThemeService, RenderService, EmailService, EmailConfig};

/// Application state shared across all requests
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<AppConfig>,
    /// Database connection pool
    pub database: Arc<DatabasePool>,
    /// Cache instance
    pub cache: Arc<Cache>,
    /// Event bus for domain events
    pub event_bus: Arc<EventBus>,
    /// Job queue for background tasks
    pub job_queue: Arc<JobQueue>,
    /// File storage
    pub storage: Arc<Storage>,
    /// JWT manager for token operations
    pub jwt: Arc<JwtManager>,
    /// Permission checker for authorization
    pub permissions: Arc<PermissionChecker>,
    /// Hook registry for WordPress-like hooks
    pub hooks: Arc<RwLock<HookRegistry>>,
    /// Plugin manager
    pub plugins: Arc<RwLock<PluginManager>>,
    /// Theme service for theme management
    pub theme_service: Arc<ThemeService>,
    /// Render service for public-facing pages
    pub render_service: Arc<RenderService>,
    /// Email service for transactional emails
    pub email_service: Arc<EmailService>,
}

impl AppState {
    /// Create a new application state builder
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    /// Get the application configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get the database pool
    pub fn db(&self) -> &DatabasePool {
        &self.database
    }

    /// Get the cache
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Get the event bus
    pub fn events(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get the job queue
    pub fn jobs(&self) -> &JobQueue {
        &self.job_queue
    }

    /// Get the storage
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get the JWT manager
    pub fn jwt(&self) -> &JwtManager {
        &self.jwt
    }

    /// Get the permission checker
    pub fn permissions(&self) -> &PermissionChecker {
        &self.permissions
    }

    /// Get the theme service/manager
    pub fn theme_manager(&self) -> &ThemeService {
        &self.theme_service
    }

    /// Get the render service
    pub fn renderer(&self) -> &RenderService {
        &self.render_service
    }

    /// Get the email service
    pub fn email(&self) -> &EmailService {
        &self.email_service
    }
}

/// Builder for AppState
pub struct AppStateBuilder {
    config: Option<AppConfig>,
    database: Option<DatabasePool>,
    cache: Option<Cache>,
    event_bus: Option<EventBus>,
    job_queue: Option<JobQueue>,
    storage: Option<Storage>,
    jwt: Option<JwtManager>,
    permissions: Option<PermissionChecker>,
    hooks: Option<HookRegistry>,
    plugins: Option<PluginManager>,
    themes_dir: Option<PathBuf>,
    email_config: Option<EmailConfig>,
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            database: None,
            cache: None,
            event_bus: None,
            job_queue: None,
            storage: None,
            jwt: None,
            permissions: None,
            hooks: None,
            plugins: None,
            themes_dir: None,
            email_config: None,
        }
    }

    pub fn config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn database(mut self, database: DatabasePool) -> Self {
        self.database = Some(database);
        self
    }

    pub fn cache(mut self, cache: Cache) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn job_queue(mut self, job_queue: JobQueue) -> Self {
        self.job_queue = Some(job_queue);
        self
    }

    pub fn storage(mut self, storage: Storage) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn jwt(mut self, jwt: JwtManager) -> Self {
        self.jwt = Some(jwt);
        self
    }

    pub fn permissions(mut self, permissions: PermissionChecker) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn hooks(mut self, hooks: HookRegistry) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn plugins(mut self, plugins: PluginManager) -> Self {
        self.plugins = Some(plugins);
        self
    }

    pub fn themes_dir(mut self, themes_dir: PathBuf) -> Self {
        self.themes_dir = Some(themes_dir);
        self
    }

    pub fn email_config(mut self, email_config: EmailConfig) -> Self {
        self.email_config = Some(email_config);
        self
    }

    /// Build the AppState
    pub fn build(self) -> Result<AppState, &'static str> {
        let database = self.database.ok_or("database is required")?;
        let themes_dir = self.themes_dir.unwrap_or_else(|| PathBuf::from("./themes"));

        // Create theme service
        let theme_service = Arc::new(ThemeService::new(
            database.pool().clone(),
            themes_dir.clone(),
            None, // site_id for multi-site support
        ));

        // Create render service
        let render_service = Arc::new(RenderService::new(
            database.pool().clone(),
            theme_service.clone(),
            themes_dir,
        ));

        // Create email service
        let email_service = Arc::new(EmailService::new());
        // Email configuration will be applied at runtime via configure()

        Ok(AppState {
            config: Arc::new(self.config.ok_or("config is required")?),
            database: Arc::new(database),
            cache: Arc::new(self.cache.ok_or("cache is required")?),
            event_bus: Arc::new(self.event_bus.ok_or("event_bus is required")?),
            job_queue: Arc::new(self.job_queue.ok_or("job_queue is required")?),
            storage: Arc::new(self.storage.ok_or("storage is required")?),
            jwt: Arc::new(self.jwt.ok_or("jwt is required")?),
            permissions: Arc::new(self.permissions.unwrap_or_else(PermissionChecker::default)),
            hooks: Arc::new(RwLock::new(self.hooks.unwrap_or_else(HookRegistry::new))),
            plugins: Arc::new(RwLock::new(self.plugins.unwrap_or_else(PluginManager::new))),
            theme_service,
            render_service,
            email_service,
        })
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
