//! Error handling for HTTP responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rustpress_core::error::Error as CoreError;
use serde::Serialize;
use std::collections::HashMap;

/// API error response format
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error code for client handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, String>>,
    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// HTTP error wrapper for Axum
pub struct HttpError {
    pub status: StatusCode,
    pub body: ApiError,
}

impl HttpError {
    pub fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            body: ApiError::new(code, message),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "BAD_REQUEST", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "FORBIDDEN", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "CONFLICT", message)
    }

    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_ERROR", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", message)
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.body.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.body.request_id = Some(request_id.into());
        self
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<CoreError> for HttpError {
    fn from(err: CoreError) -> Self {
        match &err {
            CoreError::NotFound { entity_type, id } => {
                HttpError::not_found(format!("{} with id '{}' not found", entity_type, id))
            }
            CoreError::Duplicate { entity_type, field } => {
                HttpError::conflict(format!("{} with {} already exists", entity_type, field))
            }
            CoreError::Authentication { message } => HttpError::unauthorized(message.clone()),
            CoreError::Authorization { action, required } => {
                HttpError::forbidden(format!("Permission '{}' required for action '{}'", required, action))
            }
            CoreError::Validation(validation_errors) => {
                let details: HashMap<String, String> = validation_errors.errors
                    .iter()
                    .map(|e| (e.field.clone(), e.message.clone()))
                    .collect();
                HttpError::unprocessable_entity("Validation failed").with_details(details)
            }
            CoreError::InvalidInput { field, message } => {
                let mut details = HashMap::new();
                details.insert(field.clone(), message.clone());
                HttpError::bad_request("Invalid input").with_details(details)
            }
            CoreError::Database { message, .. } => {
                tracing::error!("Database error: {}", message);
                HttpError::internal_error("A database error occurred")
            }
            CoreError::Internal { message, request_id } => {
                tracing::error!("Internal error: {}", message);
                let mut error = HttpError::internal_error("An internal error occurred");
                if let Some(rid) = request_id {
                    error = error.with_request_id(rid.to_string());
                }
                error
            }
            CoreError::ServiceUnavailable { service } => {
                tracing::error!("Service unavailable: {}", service);
                HttpError::service_unavailable(format!("Service '{}' is unavailable", service))
            }
            CoreError::Network { message, .. } => {
                tracing::error!("Network error: {}", message);
                HttpError::service_unavailable("Network error occurred")
            }
            CoreError::RateLimited { retry_after_secs } => {
                let mut error = HttpError::new(
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    "Too many requests",
                );
                let mut details = HashMap::new();
                details.insert("retry_after".to_string(), retry_after_secs.to_string());
                error = error.with_details(details);
                error
            }
            CoreError::TokenExpired => HttpError::unauthorized("Token has expired"),
            CoreError::InvalidToken { reason } => {
                HttpError::unauthorized(format!("Invalid token: {}", reason))
            }
            CoreError::Plugin { plugin_id, message } => {
                tracing::error!("Plugin error ({}): {}", plugin_id, message);
                HttpError::internal_error(format!("Plugin '{}' encountered an error", plugin_id))
            }
            CoreError::PluginNotFound { plugin_id } => {
                HttpError::not_found(format!("Plugin '{}' not found", plugin_id))
            }
            CoreError::PluginDependency { plugin_id, dependency } => {
                HttpError::bad_request(format!("Plugin '{}' requires '{}'", plugin_id, dependency))
            }
            CoreError::Configuration { message } => {
                tracing::error!("Configuration error: {}", message);
                HttpError::internal_error("Configuration error")
            }
            CoreError::Storage { message, .. } => {
                tracing::error!("Storage error: {}", message);
                HttpError::internal_error("A storage error occurred")
            }
            CoreError::FileNotFound { path } => {
                HttpError::not_found(format!("File not found: {}", path))
            }
            CoreError::Cache { message } => {
                tracing::error!("Cache error: {}", message);
                HttpError::internal_error("A cache error occurred")
            }
            CoreError::Job { job_id, message } => {
                tracing::error!("Job error ({}): {}", job_id, message);
                HttpError::internal_error("A job error occurred")
            }
            CoreError::JobTimeout { job_id, timeout_secs } => {
                tracing::error!("Job timeout ({}): {}s", job_id, timeout_secs);
                HttpError::internal_error("Job timed out")
            }
            CoreError::TenantNotFound { tenant_id } => {
                HttpError::not_found(format!("Tenant '{}' not found", tenant_id))
            }
            CoreError::TenantSuspended { tenant_id } => {
                HttpError::forbidden(format!("Tenant '{}' is suspended", tenant_id))
            }
            CoreError::Hook { hook_name, message } => {
                tracing::error!("Hook error ({}): {}", hook_name, message);
                HttpError::internal_error("A hook error occurred")
            }
            CoreError::Serialization { message } => {
                tracing::error!("Serialization error: {}", message);
                HttpError::internal_error("Serialization error")
            }
            CoreError::Migration { message } => {
                tracing::error!("Migration error: {}", message);
                HttpError::internal_error("Migration error")
            }
            CoreError::ShutdownInProgress => {
                HttpError::service_unavailable("Service is shutting down")
            }
            CoreError::Other(e) => {
                tracing::error!("Unexpected error: {}", e);
                HttpError::internal_error("An unexpected error occurred")
            }
        }
    }
}

/// Result type for HTTP handlers
pub type HttpResult<T> = Result<T, HttpError>;

/// Extension trait for converting Results to HttpResults
pub trait IntoHttpResult<T> {
    fn into_http(self) -> HttpResult<T>;
}

impl<T> IntoHttpResult<T> for Result<T, CoreError> {
    fn into_http(self) -> HttpResult<T> {
        self.map_err(HttpError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
    }

    #[test]
    fn test_http_error_from_core() {
        let core_error = CoreError::NotFound {
            entity_type: "Post".to_string(),
            id: "123".to_string(),
        };
        let http_error: HttpError = core_error.into();
        assert_eq!(http_error.status, StatusCode::NOT_FOUND);
    }
}
