//! Error types for the RustCloudflare plugin

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Main error type for Cloudflare operations
#[derive(Error, Debug)]
pub enum CloudflareError {
    #[error("API authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Invalid API token")]
    InvalidToken,

    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Zone not found: {0}")]
    ZoneNotFound(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("API error: {message} (code: {code})")]
    ApiError { code: i32, message: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Worker error: {0}")]
    WorkerError(String),

    #[error("R2 storage error: {0}")]
    R2Error(String),

    #[error("D1 database error: {0}")]
    D1Error(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("KV error: {0}")]
    KvError(String),

    #[error("DNS error: {0}")]
    DnsError(String),

    #[error("SSL/TLS error: {0}")]
    SslError(String),

    #[error("WAF error: {0}")]
    WafError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Plugin not configured - Cloudflare credentials required")]
    NotConfigured,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl From<CloudflareError> for rustpress_core::Error {
    fn from(err: CloudflareError) -> Self {
        rustpress_core::Error::internal(err.to_string())
    }
}

impl CloudflareError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthenticationError(_) | Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::PermissionDenied(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) | Self::ZoneNotFound(_) => StatusCode::NOT_FOUND,
            Self::ValidationError(_) | Self::InvalidConfig(_) | Self::MissingConfig(_) | Self::ConfigError(_) => {
                StatusCode::BAD_REQUEST
            }
            Self::NotConfigured => StatusCode::SERVICE_UNAVAILABLE,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::ServiceUnavailable(_) | Self::Timeout(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ApiError { code, .. } => {
                // Map Cloudflare error codes to HTTP status
                match code {
                    1000..=1099 => StatusCode::BAD_REQUEST,
                    6000..=6999 => StatusCode::UNAUTHORIZED,
                    7000..=7999 => StatusCode::FORBIDDEN,
                    8000..=8999 => StatusCode::NOT_FOUND,
                    9000..=9999 => StatusCode::TOO_MANY_REQUESTS,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::AuthenticationError(_) => "AUTH_ERROR",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::MissingConfig(_) => "MISSING_CONFIG",
            Self::InvalidConfig(_) => "INVALID_CONFIG",
            Self::ZoneNotFound(_) => "ZONE_NOT_FOUND",
            Self::NotFound(_) => "NOT_FOUND",
            Self::RateLimitExceeded => "RATE_LIMITED",
            Self::ApiError { .. } => "API_ERROR",
            Self::NetworkError(_) => "NETWORK_ERROR",
            Self::Timeout(_) => "TIMEOUT",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::PermissionDenied(_) => "PERMISSION_DENIED",
            Self::Conflict(_) => "CONFLICT",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::WorkerError(_) => "WORKER_ERROR",
            Self::R2Error(_) => "R2_ERROR",
            Self::D1Error(_) => "D1_ERROR",
            Self::StreamError(_) => "STREAM_ERROR",
            Self::KvError(_) => "KV_ERROR",
            Self::DnsError(_) => "DNS_ERROR",
            Self::SslError(_) => "SSL_ERROR",
            Self::WafError(_) => "WAF_ERROR",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::ConfigError(_) => "CONFIG_ERROR",
            Self::NotConfigured => "NOT_CONFIGURED",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Reqwest(_) => "HTTP_CLIENT_ERROR",
            Self::SerdeJson(_) => "SERIALIZATION_ERROR",
            Self::Sqlx(_) => "DATABASE_ERROR",
            Self::Anyhow(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for CloudflareError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({
            "success": false,
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
            }
        }));

        (status, body).into_response()
    }
}

/// Result type alias for Cloudflare operations
pub type CloudflareResult<T> = Result<T, CloudflareError>;
