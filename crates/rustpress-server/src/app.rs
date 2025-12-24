//! Main application struct and server setup.

use axum::{middleware as axum_middleware, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::error::HttpError;
use crate::metrics::Metrics;
use crate::middleware::{
    api_version, body_limit, compression_layer, cors_layer, rate_limit,
    request_id, request_logging, security_headers, tenant_identification,
};
use crate::routes::create_router;
use crate::security::{
    bot_detection::{bot_detection, BotDetectionConfig, BotDetectionMiddleware},
    content_security::{content_security, ContentSecurityConfig, ContentSecurityMiddleware},
    fingerprint::{fingerprint, FingerprintConfig, FingerprintMiddleware},
    request_validation::{request_validation, SecurityConfig, SecurityMiddleware},
    security_audit::{security_audit, SecurityAuditConfig, SecurityAuditLogger},
};
use crate::shutdown::{
    graceful_shutdown, listen_for_shutdown_signals, ShutdownController, ShutdownExecutor,
    ShutdownPhase,
};
use crate::state::AppState;

/// Main application struct
pub struct App {
    state: AppState,
    metrics: Arc<Metrics>,
    shutdown_controller: ShutdownController,
    // Security middleware
    security_middleware: SecurityMiddleware,
    content_security: ContentSecurityMiddleware,
    bot_detection: BotDetectionMiddleware,
    fingerprint: FingerprintMiddleware,
    audit_logger: SecurityAuditLogger,
}

impl App {
    /// Create a new application instance
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            metrics: Arc::new(Metrics::new()),
            shutdown_controller: ShutdownController::with_default_timeout(),
            // Initialize security middleware with default configs
            security_middleware: SecurityMiddleware::new(SecurityConfig::default()),
            content_security: ContentSecurityMiddleware::new(ContentSecurityConfig::default()),
            bot_detection: BotDetectionMiddleware::new(BotDetectionConfig::default()),
            fingerprint: FingerprintMiddleware::new(FingerprintConfig::default()),
            audit_logger: SecurityAuditLogger::new(SecurityAuditConfig::default()),
        }
    }

    /// Create with custom shutdown timeout
    pub fn with_shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_controller = ShutdownController::new(timeout);
        self
    }

    /// Get the application state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get the metrics
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Get the shutdown controller
    pub fn shutdown_controller(&self) -> &ShutdownController {
        &self.shutdown_controller
    }

    /// Build the router with all middleware
    pub fn build_router(&self) -> Router {
        let router = create_router(self.state.clone());

        // Apply middleware stack (order matters - last added is first executed)
        // Execution order: Compression -> Tracing -> Request ID -> Security Audit ->
        // Fingerprint -> Bot Detection -> Logging -> Security Headers ->
        // Request Validation -> Content Security -> CORS -> Body Limit ->
        // API Version -> Rate Limit -> Tenant ID -> Route Handler
        router
            .layer(
                ServiceBuilder::new()
                    // Compression
                    .layer(compression_layer())
                    // Tracing
                    .layer(TraceLayer::new_for_http())
            )
            // Request ID (first, so all subsequent middleware can use it)
            .layer(axum_middleware::from_fn(request_id))
            // Security audit logging (captures all security events)
            .layer(axum_middleware::from_fn_with_state(
                self.audit_logger.clone(),
                security_audit,
            ))
            // Request fingerprinting (tracks client behavior)
            .layer(axum_middleware::from_fn_with_state(
                self.fingerprint.clone(),
                fingerprint,
            ))
            // Bot detection (blocks automated attacks)
            .layer(axum_middleware::from_fn_with_state(
                self.bot_detection.clone(),
                bot_detection,
            ))
            // Request logging
            .layer(axum_middleware::from_fn(request_logging))
            // Security headers (enhanced with COEP, COOP, CORP, Permissions-Policy)
            .layer(axum_middleware::from_fn(security_headers))
            // Request validation (SQL injection, XSS, path traversal protection)
            .layer(axum_middleware::from_fn_with_state(
                self.security_middleware.clone(),
                request_validation,
            ))
            // Content security (JSON depth, content-type validation)
            .layer(axum_middleware::from_fn_with_state(
                self.content_security.clone(),
                content_security,
            ))
            // CORS
            .layer(cors_layer())
            // Body size limit
            .layer(axum_middleware::from_fn(body_limit))
            // API versioning
            .layer(axum_middleware::from_fn(api_version))
            // Rate limiting
            .layer(axum_middleware::from_fn_with_state(
                self.state.clone(),
                rate_limit,
            ))
            // Tenant identification
            .layer(axum_middleware::from_fn_with_state(
                self.state.clone(),
                tenant_identification,
            ))
    }

    /// Run the HTTP server
    pub async fn run(self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let router = self.build_router();

        info!("Starting RustPress server on {}", addr);

        // Create TCP listener
        let listener = TcpListener::bind(addr).await?;
        info!("Server listening on {}", addr);

        // Spawn shutdown signal listener
        let shutdown_controller = self.shutdown_controller.clone();
        tokio::spawn(listen_for_shutdown_signals(shutdown_controller.clone()));

        // Create shutdown executor for ordered cleanup
        let mut shutdown_executor = ShutdownExecutor::new(shutdown_controller.clone());

        // Register shutdown handlers
        let state_clone = self.state.clone();
        shutdown_executor.register(ShutdownPhase::FlushCaches, move || {
            let _state = state_clone.clone();
            async move {
                info!("Flushing caches...");
                // Cache flush logic would go here
            }
        });

        let state_clone = self.state.clone();
        shutdown_executor.register(ShutdownPhase::CloseDatabase, move || {
            let _state = state_clone.clone();
            async move {
                info!("Closing database connections...");
                // Database close logic would go here
            }
        });

        // Run server with graceful shutdown
        axum::serve(listener, router)
            .with_graceful_shutdown(graceful_shutdown(shutdown_controller))
            .await?;

        // Execute ordered shutdown
        shutdown_executor.execute().await;

        info!("Server shutdown complete");
        Ok(())
    }

    /// Run the server on the configured address
    pub async fn run_from_config(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::new(
            self.state.config.server.host.parse()?,
            self.state.config.server.port,
        );
        self.run(addr).await
    }
}

/// Server configuration builder
pub struct ServerBuilder {
    state: Option<AppState>,
    host: String,
    port: u16,
    shutdown_timeout: Duration,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            state: None,
            host: "127.0.0.1".to_string(),
            port: 3000,
            shutdown_timeout: Duration::from_secs(30),
        }
    }

    pub fn state(mut self, state: AppState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Build and run the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.ok_or("AppState is required")?;
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse()?;

        let app = App::new(state).with_shutdown_timeout(self.shutdown_timeout);
        app.run(addr).await
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick start function for development
pub async fn serve(state: AppState, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = addr.parse()?;
    let app = App::new(state);
    app.run(addr).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder() {
        let builder = ServerBuilder::new()
            .host("0.0.0.0")
            .port(8080)
            .shutdown_timeout(Duration::from_secs(60));

        assert_eq!(builder.host, "0.0.0.0");
        assert_eq!(builder.port, 8080);
        assert_eq!(builder.shutdown_timeout, Duration::from_secs(60));
    }
}
