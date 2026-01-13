//! Request and application context for RustPress.
//!
//! Provides context objects that carry state through the request lifecycle.

use crate::id::{TenantId, UserId};
use crate::tenant::Tenant;
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Request context carries information through the entire request lifecycle.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request identifier for tracing
    pub request_id: Uuid,
    /// Start time of the request
    pub start_time: std::time::Instant,
    /// Authenticated user ID (if any)
    pub user_id: Option<UserId>,
    /// Current tenant (if multi-tenancy enabled)
    pub tenant_id: Option<TenantId>,
    /// API version being requested
    pub api_version: Option<String>,
    /// Request path
    pub path: String,
    /// Request method
    pub method: String,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Additional metadata
    metadata: Arc<RwLock<HashMap<String, String>>>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(path: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            request_id: Uuid::now_v7(),
            start_time: std::time::Instant::now(),
            user_id: None,
            tenant_id: None,
            api_version: None,
            path: path.into(),
            method: method.into(),
            client_ip: None,
            user_agent: None,
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a context for testing
    pub fn test() -> Self {
        Self::new("/test", "GET")
    }

    /// Set the authenticated user
    pub fn with_user(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the tenant
    pub fn with_tenant(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Set the API version
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Set client IP
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Add metadata
    pub fn set_metadata(&self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.write().insert(key.into(), value.into());
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.read().get(key).cloned()
    }

    /// Get elapsed time since request start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Get a tracing span for this request
    pub fn span(&self) -> tracing::Span {
        tracing::info_span!(
            "request",
            request_id = %self.request_id,
            method = %self.method,
            path = %self.path,
            user_id = ?self.user_id.map(|u| u.to_string()),
            tenant_id = ?self.tenant_id.map(|t| t.to_string()),
        )
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new("/", "GET")
    }
}

/// Type-erased storage for application state
pub struct TypeMap {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast().ok().map(|b| *b))
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast().ok().map(|b| *b))
    }
}

impl Default for TypeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Application context shared across all requests.
/// Contains shared resources like database pools, caches, etc.
pub struct AppContext {
    /// Type-erased state storage
    state: RwLock<TypeMap>,
    /// Configuration
    config: Arc<crate::config::AppConfig>,
    /// Current tenant (for single-tenant mode or default tenant)
    current_tenant: Option<Arc<Tenant>>,
    /// Shutdown signal
    shutdown: Arc<tokio::sync::watch::Sender<bool>>,
}

impl AppContext {
    /// Create a new application context
    pub fn new(config: crate::config::AppConfig) -> Self {
        let (shutdown_tx, _) = tokio::sync::watch::channel(false);
        Self {
            state: RwLock::new(TypeMap::new()),
            config: Arc::new(config),
            current_tenant: None,
            shutdown: Arc::new(shutdown_tx),
        }
    }

    /// Get the application configuration
    pub fn config(&self) -> &crate::config::AppConfig {
        &self.config
    }

    /// Register a state object
    pub fn register<T: Send + Sync + 'static>(&self, value: T) {
        self.state.write().insert(value);
    }

    /// Get a reference to a state object
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<impl std::ops::Deref<Target = T> + '_> {
        let guard = self.state.read();
        if guard.contains::<T>() {
            Some(parking_lot::RwLockReadGuard::map(guard, |state| {
                state.get::<T>().unwrap()
            }))
        } else {
            None
        }
    }

    /// Set the current tenant
    pub fn set_tenant(&mut self, tenant: Tenant) {
        self.current_tenant = Some(Arc::new(tenant));
    }

    /// Get the current tenant
    pub fn tenant(&self) -> Option<&Arc<Tenant>> {
        self.current_tenant.as_ref()
    }

    /// Get a shutdown receiver
    pub fn shutdown_receiver(&self) -> tokio::sync::watch::Receiver<bool> {
        self.shutdown.subscribe()
    }

    /// Trigger shutdown
    pub fn shutdown(&self) {
        // Use send_replace instead of send to ensure the value is updated
        // even if there are no active receivers
        self.shutdown.send_replace(true);
    }

    /// Check if shutdown has been triggered
    pub fn is_shutting_down(&self) -> bool {
        *self.shutdown.borrow()
    }
}

/// Builder for creating AppContext with dependencies
pub struct AppContextBuilder {
    config: crate::config::AppConfig,
    tenant: Option<Tenant>,
}

impl AppContextBuilder {
    pub fn new(config: crate::config::AppConfig) -> Self {
        Self {
            config,
            tenant: None,
        }
    }

    pub fn with_tenant(mut self, tenant: Tenant) -> Self {
        self.tenant = Some(tenant);
        self
    }

    pub fn build(self) -> AppContext {
        let mut ctx = AppContext::new(self.config);
        if let Some(tenant) = self.tenant {
            ctx.set_tenant(tenant);
        }
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new("/api/posts", "GET")
            .with_client_ip("127.0.0.1")
            .with_api_version("v1");

        assert_eq!(ctx.path, "/api/posts");
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.client_ip, Some("127.0.0.1".to_string()));
        assert_eq!(ctx.api_version, Some("v1".to_string()));
        assert!(!ctx.is_authenticated());
    }

    #[test]
    fn test_request_context_metadata() {
        let ctx = RequestContext::test();
        ctx.set_metadata("key", "value");
        assert_eq!(ctx.get_metadata("key"), Some("value".to_string()));
        assert_eq!(ctx.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_type_map() {
        let mut map = TypeMap::new();

        #[derive(Debug, PartialEq)]
        struct MyState {
            value: i32,
        }

        map.insert(MyState { value: 42 });
        assert!(map.contains::<MyState>());
        assert_eq!(map.get::<MyState>().unwrap().value, 42);

        let removed = map.remove::<MyState>();
        assert_eq!(removed.unwrap().value, 42);
        assert!(!map.contains::<MyState>());
    }

    #[test]
    fn test_app_context() {
        let config = crate::config::AppConfig::default();
        let ctx = AppContext::new(config);

        assert_eq!(ctx.config().server.port, 8080);
        assert!(!ctx.is_shutting_down());

        ctx.shutdown();
        assert!(ctx.is_shutting_down());
    }
}
