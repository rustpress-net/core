//! Content security middleware.
//!
//! Provides protection against payload-based attacks including:
//! - JSON depth limiting (DoS prevention)
//! - Content-Type validation
//! - File upload validation
//! - Payload structure validation

use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Content security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSecurityConfig {
    /// Maximum JSON nesting depth (default: 32)
    pub max_json_depth: usize,
    /// Maximum number of JSON keys (default: 1000)
    pub max_json_keys: usize,
    /// Maximum JSON string length (default: 1MB)
    pub max_json_string_length: usize,
    /// Enforce Content-Type header matching
    pub enforce_content_type: bool,
    /// Allowed Content-Types for requests with bodies
    pub allowed_content_types: Vec<String>,
    /// Maximum request body size per route pattern
    pub route_body_limits: Vec<(String, usize)>,
    /// Default maximum body size (10MB)
    pub default_max_body_size: usize,
    /// Allowed file upload MIME types
    pub allowed_upload_types: Vec<String>,
    /// Enable magic byte validation for uploads
    pub validate_magic_bytes: bool,
}

impl Default for ContentSecurityConfig {
    fn default() -> Self {
        Self {
            max_json_depth: 32,
            max_json_keys: 1000,
            max_json_string_length: 1024 * 1024, // 1MB
            enforce_content_type: true,
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
                "multipart/form-data".to_string(),
                "text/plain".to_string(),
            ],
            route_body_limits: vec![
                ("/api/media/upload".to_string(), 100 * 1024 * 1024), // 100MB for uploads
                ("/api/".to_string(), 10 * 1024 * 1024),             // 10MB for API
            ],
            default_max_body_size: 10 * 1024 * 1024, // 10MB
            allowed_upload_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/svg+xml".to_string(),
                "application/pdf".to_string(),
                "video/mp4".to_string(),
                "video/webm".to_string(),
                "audio/mpeg".to_string(),
                "audio/wav".to_string(),
            ],
            validate_magic_bytes: true,
        }
    }
}

/// Content security middleware
#[derive(Clone)]
pub struct ContentSecurityMiddleware {
    config: Arc<ContentSecurityConfig>,
}

impl ContentSecurityMiddleware {
    pub fn new(config: ContentSecurityConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Get the body size limit for a given path
    pub fn get_body_limit(&self, path: &str) -> usize {
        for (pattern, limit) in &self.config.route_body_limits {
            if path.starts_with(pattern) {
                return *limit;
            }
        }
        self.config.default_max_body_size
    }

    /// Validate Content-Type header
    pub fn validate_content_type(&self, content_type: Option<&str>) -> bool {
        if !self.config.enforce_content_type {
            return true;
        }

        match content_type {
            Some(ct) => {
                let ct_lower = ct.to_lowercase();
                self.config
                    .allowed_content_types
                    .iter()
                    .any(|allowed| ct_lower.starts_with(allowed))
            }
            None => true, // No Content-Type for GET/HEAD/OPTIONS is fine
        }
    }

    /// Validate JSON depth
    pub fn validate_json_depth(&self, json: &serde_json::Value) -> Result<(), ContentSecurityError> {
        fn check_depth(value: &serde_json::Value, current: usize, max: usize) -> Result<usize, usize> {
            if current > max {
                return Err(current);
            }

            match value {
                serde_json::Value::Array(arr) => {
                    let mut max_depth = current;
                    for item in arr {
                        max_depth = max_depth.max(check_depth(item, current + 1, max)?);
                    }
                    Ok(max_depth)
                }
                serde_json::Value::Object(obj) => {
                    let mut max_depth = current;
                    for (_, v) in obj {
                        max_depth = max_depth.max(check_depth(v, current + 1, max)?);
                    }
                    Ok(max_depth)
                }
                _ => Ok(current),
            }
        }

        check_depth(json, 0, self.config.max_json_depth)
            .map(|_| ())
            .map_err(|depth| ContentSecurityError::JsonDepthExceeded {
                max: self.config.max_json_depth,
                actual: depth,
            })
    }

    /// Validate JSON key count
    pub fn validate_json_keys(&self, json: &serde_json::Value) -> Result<(), ContentSecurityError> {
        fn count_keys(value: &serde_json::Value) -> usize {
            match value {
                serde_json::Value::Array(arr) => arr.iter().map(count_keys).sum(),
                serde_json::Value::Object(obj) => {
                    obj.len() + obj.values().map(count_keys).sum::<usize>()
                }
                _ => 0,
            }
        }

        let count = count_keys(json);
        if count > self.config.max_json_keys {
            Err(ContentSecurityError::JsonKeyCountExceeded {
                max: self.config.max_json_keys,
                actual: count,
            })
        } else {
            Ok(())
        }
    }

    /// Validate JSON string lengths
    pub fn validate_json_strings(&self, json: &serde_json::Value) -> Result<(), ContentSecurityError> {
        fn check_strings(
            value: &serde_json::Value,
            max_len: usize,
        ) -> Result<(), (usize, String)> {
            match value {
                serde_json::Value::String(s) => {
                    if s.len() > max_len {
                        Err((s.len(), s.chars().take(50).collect()))
                    } else {
                        Ok(())
                    }
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        check_strings(item, max_len)?;
                    }
                    Ok(())
                }
                serde_json::Value::Object(obj) => {
                    for (_, v) in obj {
                        check_strings(v, max_len)?;
                    }
                    Ok(())
                }
                _ => Ok(()),
            }
        }

        check_strings(json, self.config.max_json_string_length).map_err(|(len, preview)| {
            ContentSecurityError::JsonStringTooLong {
                max: self.config.max_json_string_length,
                actual: len,
                preview,
            }
        })
    }

    /// Validate file upload MIME type
    pub fn validate_upload_type(&self, content_type: &str) -> bool {
        self.config
            .allowed_upload_types
            .iter()
            .any(|allowed| content_type.starts_with(allowed))
    }

    /// Validate file magic bytes
    pub fn validate_magic_bytes(&self, bytes: &[u8], claimed_type: &str) -> bool {
        if !self.config.validate_magic_bytes {
            return true;
        }

        // Check magic bytes for common file types
        match claimed_type {
            "image/jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
            "image/png" => bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
            "image/webp" => {
                bytes.len() >= 12
                    && bytes.starts_with(b"RIFF")
                    && &bytes[8..12] == b"WEBP"
            }
            "application/pdf" => bytes.starts_with(b"%PDF"),
            "video/mp4" => {
                // MP4 has "ftyp" at offset 4
                bytes.len() >= 8 && &bytes[4..8] == b"ftyp"
            }
            "video/webm" => bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]),
            "audio/mpeg" => {
                // MP3 starts with ID3 or sync bits
                bytes.starts_with(b"ID3") || (bytes.len() >= 2 && bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0)
            }
            "audio/wav" => bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WAVE",
            "image/svg+xml" => {
                // SVG is XML, check for opening tag
                let s = String::from_utf8_lossy(bytes);
                s.contains("<svg") || s.contains("<?xml")
            }
            _ => true, // Unknown types pass through
        }
    }
}

/// Content security errors
#[derive(Debug, Clone)]
pub enum ContentSecurityError {
    JsonDepthExceeded { max: usize, actual: usize },
    JsonKeyCountExceeded { max: usize, actual: usize },
    JsonStringTooLong { max: usize, actual: usize, preview: String },
    InvalidContentType { received: String },
    BodyTooLarge { max: usize, actual: usize },
    InvalidUploadType { received: String },
    MagicByteMismatch { claimed: String },
}

impl std::fmt::Display for ContentSecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentSecurityError::JsonDepthExceeded { max, actual } => {
                write!(f, "JSON depth {} exceeds maximum {}", actual, max)
            }
            ContentSecurityError::JsonKeyCountExceeded { max, actual } => {
                write!(f, "JSON key count {} exceeds maximum {}", actual, max)
            }
            ContentSecurityError::JsonStringTooLong { max, actual, preview } => {
                write!(
                    f,
                    "JSON string length {} exceeds maximum {} (preview: {}...)",
                    actual, max, preview
                )
            }
            ContentSecurityError::InvalidContentType { received } => {
                write!(f, "Invalid Content-Type: {}", received)
            }
            ContentSecurityError::BodyTooLarge { max, actual } => {
                write!(f, "Body size {} exceeds maximum {}", actual, max)
            }
            ContentSecurityError::InvalidUploadType { received } => {
                write!(f, "Invalid upload type: {}", received)
            }
            ContentSecurityError::MagicByteMismatch { claimed } => {
                write!(f, "File magic bytes don't match claimed type: {}", claimed)
            }
        }
    }
}

impl IntoResponse for ContentSecurityError {
    fn into_response(self) -> Response {
        let status = match &self {
            ContentSecurityError::BodyTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        };

        (status, self.to_string()).into_response()
    }
}

/// Content security middleware function
pub async fn content_security(
    State(security): State<ContentSecurityMiddleware>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Skip validation for safe methods
    if matches!(method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(request).await;
    }

    // Validate Content-Type
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    if !security.validate_content_type(content_type) {
        tracing::warn!(
            content_type = ?content_type,
            path = %path,
            "Invalid Content-Type"
        );
        return ContentSecurityError::InvalidContentType {
            received: content_type.unwrap_or("none").to_string(),
        }
        .into_response();
    }

    // Check body size limit from Content-Length header
    let body_limit = security.get_body_limit(&path);
    if let Some(content_length) = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        if content_length > body_limit {
            tracing::warn!(
                content_length = content_length,
                limit = body_limit,
                path = %path,
                "Request body too large"
            );
            return ContentSecurityError::BodyTooLarge {
                max: body_limit,
                actual: content_length,
            }
            .into_response();
        }
    }

    next.run(request).await
}

/// Validate JSON body middleware (use after body extraction)
pub fn validate_json_body(
    security: &ContentSecurityMiddleware,
    json: &serde_json::Value,
) -> Result<(), ContentSecurityError> {
    security.validate_json_depth(json)?;
    security.validate_json_keys(json)?;
    security.validate_json_strings(json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_depth_validation() {
        let config = ContentSecurityConfig {
            max_json_depth: 3,
            ..Default::default()
        };
        let middleware = ContentSecurityMiddleware::new(config);

        // Valid depth
        let shallow = json!({"a": {"b": "c"}});
        assert!(middleware.validate_json_depth(&shallow).is_ok());

        // Exceeds depth
        let deep = json!({"a": {"b": {"c": {"d": "e"}}}});
        assert!(middleware.validate_json_depth(&deep).is_err());
    }

    #[test]
    fn test_json_key_count_validation() {
        let config = ContentSecurityConfig {
            max_json_keys: 5,
            ..Default::default()
        };
        let middleware = ContentSecurityMiddleware::new(config);

        // Valid key count
        let few_keys = json!({"a": 1, "b": 2, "c": 3});
        assert!(middleware.validate_json_keys(&few_keys).is_ok());

        // Exceeds key count
        let many_keys = json!({"a": 1, "b": 2, "c": 3, "d": 4, "e": 5, "f": 6});
        assert!(middleware.validate_json_keys(&many_keys).is_err());
    }

    #[test]
    fn test_magic_bytes_validation() {
        let config = ContentSecurityConfig::default();
        let middleware = ContentSecurityMiddleware::new(config);

        // Valid JPEG
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert!(middleware.validate_magic_bytes(&jpeg, "image/jpeg"));

        // Valid PNG
        let png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(middleware.validate_magic_bytes(&png, "image/png"));

        // Invalid - claiming JPEG but is PNG
        assert!(!middleware.validate_magic_bytes(&png, "image/jpeg"));
    }

    #[test]
    fn test_content_type_validation() {
        let config = ContentSecurityConfig::default();
        let middleware = ContentSecurityMiddleware::new(config);

        assert!(middleware.validate_content_type(Some("application/json")));
        assert!(middleware.validate_content_type(Some("application/json; charset=utf-8")));
        assert!(middleware.validate_content_type(Some("multipart/form-data; boundary=----")));
        assert!(!middleware.validate_content_type(Some("text/html")));
    }

    #[test]
    fn test_body_limit() {
        let config = ContentSecurityConfig {
            route_body_limits: vec![
                ("/api/upload".to_string(), 100 * 1024 * 1024),
                ("/api/".to_string(), 1024 * 1024),
            ],
            default_max_body_size: 512 * 1024,
            ..Default::default()
        };
        let middleware = ContentSecurityMiddleware::new(config);

        assert_eq!(middleware.get_body_limit("/api/upload/image"), 100 * 1024 * 1024);
        assert_eq!(middleware.get_body_limit("/api/users"), 1024 * 1024);
        assert_eq!(middleware.get_body_limit("/other"), 512 * 1024);
    }
}
