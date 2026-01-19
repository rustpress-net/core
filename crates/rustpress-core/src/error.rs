//! Unified error types for RustPress using thiserror.
//!
//! Provides consistent error handling across all components.

use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// The unified error type for RustPress operations
#[derive(Error, Debug)]
pub enum Error {
    // Database errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Duplicate entity: {entity_type} already exists")]
    Duplicate { entity_type: String, field: String },

    // Authentication errors
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Authorization failed: insufficient permissions for {action}")]
    Authorization { action: String, required: String },

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {reason}")]
    InvalidToken { reason: String },

    // Validation errors
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },

    // Plugin errors
    #[error("Plugin error: {plugin_id} - {message}")]
    Plugin { plugin_id: String, message: String },

    #[error("Plugin not found: {plugin_id}")]
    PluginNotFound { plugin_id: String },

    #[error("Plugin dependency error: {plugin_id} requires {dependency}")]
    PluginDependency {
        plugin_id: String,
        dependency: String,
    },

    // Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    // Storage errors
    #[error("Storage error: {message}")]
    Storage {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    // Cache errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    // Job/Queue errors
    #[error("Job error: {job_id} - {message}")]
    Job { job_id: String, message: String },

    #[error("Job timeout: {job_id} exceeded {timeout_secs}s")]
    JobTimeout { job_id: String, timeout_secs: u64 },

    // Rate limiting
    #[error("Rate limit exceeded: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    // Multi-tenancy errors
    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: String },

    #[error("Tenant suspended: {tenant_id}")]
    TenantSuspended { tenant_id: String },

    // Hook errors
    #[error("Hook error: {hook_name} - {message}")]
    Hook { hook_name: String, message: String },

    // Network errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    // Serialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    // Internal errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        request_id: Option<Uuid>,
    },

    // Migration errors
    #[error("Migration error: {message}")]
    Migration { message: String },

    // Shutdown errors
    #[error("Shutdown in progress")]
    ShutdownInProgress,

    // Generic wrapped error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Validation errors container
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
        write!(f, "{}", messages.join("; "))
    }
}

impl std::error::Error for ValidationErrors {}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            code: None,
        });
    }

    pub fn add_with_code(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            code: Some(code.into()),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(Error::Validation(self))
        }
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

/// A single validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Error context for enhanced debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub request_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub operation: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            request_id: None,
            tenant_id: None,
            user_id: None,
            operation: operation.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_request_id(mut self, id: Uuid) -> Self {
        self.request_id = Some(id);
        self
    }

    pub fn with_tenant_id(mut self, id: impl Into<String>) -> Self {
        self.tenant_id = Some(id.into());
        self
    }

    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorExt {
    fn with_context(self, ctx: ErrorContext) -> Error;
}

impl Error {
    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Error::Database {
            message: message.into(),
            source: None,
        }
    }

    /// Create a database error with source
    pub fn database_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::Database {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Error::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Error::Internal {
            message: message.into(),
            request_id: None,
        }
    }

    /// Create an internal error with request ID
    pub fn internal_with_request_id(message: impl Into<String>, request_id: Uuid) -> Self {
        Error::Internal {
            message: message.into(),
            request_id: Some(request_id),
        }
    }

    /// Create a validation error from a single message
    pub fn validation(message: impl Into<String>) -> Self {
        let msg = message.into();
        let mut errors = ValidationErrors::new();
        errors.add("validation", &msg);
        Error::Validation(errors)
    }

    /// Create an invalid input error
    pub fn invalid_input(field: impl Into<String>, message: impl Into<String>) -> Self {
        Error::InvalidInput {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization(action: impl Into<String>, required: impl Into<String>) -> Self {
        Error::Authorization {
            action: action.into(),
            required: required.into(),
        }
    }

    /// Create an unauthorized error (401)
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Error::Authentication {
            message: message.into(),
        }
    }

    /// Create a forbidden error (403)
    pub fn forbidden(action: impl Into<String>) -> Self {
        Error::Authorization {
            action: action.into(),
            required: "appropriate permissions".into(),
        }
    }

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Error::Storage {
            message: message.into(),
            source: None,
        }
    }

    /// Create a storage error with source
    pub fn storage_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::Storage {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Error::Serialization {
            message: message.into(),
        }
    }

    /// Create a serialization error with source
    pub fn serialization_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::Serialization {
            message: format!("{}: {}", message.into(), source),
        }
    }

    /// Create a deserialization error
    pub fn deserialization(message: impl Into<String>) -> Self {
        Error::Serialization {
            message: message.into(),
        }
    }

    /// Create a deserialization error with source
    pub fn deserialization_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Error::Serialization {
            message: format!("{}: {}", message.into(), source),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Database { .. }
                | Error::Network { .. }
                | Error::ServiceUnavailable { .. }
                | Error::RateLimited { .. }
                | Error::Cache { .. }
        )
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Error::NotFound { .. } | Error::FileNotFound { .. } => 404,
            Error::Authentication { .. } | Error::TokenExpired | Error::InvalidToken { .. } => 401,
            Error::Authorization { .. } => 403,
            Error::Validation(_) | Error::InvalidInput { .. } => 400,
            Error::Duplicate { .. } => 409,
            Error::RateLimited { .. } => 429,
            Error::ServiceUnavailable { .. } | Error::ShutdownInProgress => 503,
            Error::TenantNotFound { .. } | Error::TenantSuspended { .. } => 403,
            _ => 500,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Database { .. } => "DATABASE_ERROR",
            Error::NotFound { .. } => "NOT_FOUND",
            Error::Duplicate { .. } => "DUPLICATE",
            Error::Authentication { .. } => "AUTH_FAILED",
            Error::Authorization { .. } => "FORBIDDEN",
            Error::TokenExpired => "TOKEN_EXPIRED",
            Error::InvalidToken { .. } => "INVALID_TOKEN",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::InvalidInput { .. } => "INVALID_INPUT",
            Error::Plugin { .. } => "PLUGIN_ERROR",
            Error::PluginNotFound { .. } => "PLUGIN_NOT_FOUND",
            Error::PluginDependency { .. } => "PLUGIN_DEPENDENCY",
            Error::Configuration { .. } => "CONFIG_ERROR",
            Error::Storage { .. } => "STORAGE_ERROR",
            Error::FileNotFound { .. } => "FILE_NOT_FOUND",
            Error::Cache { .. } => "CACHE_ERROR",
            Error::Job { .. } => "JOB_ERROR",
            Error::JobTimeout { .. } => "JOB_TIMEOUT",
            Error::RateLimited { .. } => "RATE_LIMITED",
            Error::TenantNotFound { .. } => "TENANT_NOT_FOUND",
            Error::TenantSuspended { .. } => "TENANT_SUSPENDED",
            Error::Hook { .. } => "HOOK_ERROR",
            Error::Network { .. } => "NETWORK_ERROR",
            Error::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Error::Serialization { .. } => "SERIALIZATION_ERROR",
            Error::Internal { .. } => "INTERNAL_ERROR",
            Error::Migration { .. } => "MIGRATION_ERROR",
            Error::ShutdownInProgress => "SHUTDOWN",
            Error::Other(_) => "UNKNOWN_ERROR",
        }
    }
}

/// Result type alias for RustPress operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(Error::not_found("Post", "123").status_code(), 404);
        assert_eq!(
            Error::Authentication {
                message: "test".into()
            }
            .status_code(),
            401
        );
        assert_eq!(
            Error::RateLimited {
                retry_after_secs: 60
            }
            .status_code(),
            429
        );
    }

    #[test]
    fn test_validation_errors() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());

        errors.add("email", "Invalid email format");
        errors.add_with_code("password", "Too short", "PASSWORD_TOO_SHORT");

        assert!(!errors.is_empty());
        assert_eq!(errors.errors.len(), 2);
    }

    #[test]
    fn test_error_retryable() {
        assert!(Error::database("connection failed").is_retryable());
        assert!(Error::RateLimited {
            retry_after_secs: 60
        }
        .is_retryable());
        assert!(!Error::not_found("Post", "123").is_retryable());
    }
}
