//! HTTP middleware implementations.

use axum::{
    body::Body,
    extract::State,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::{Duration, Instant};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn, Span};
use uuid::Uuid;

use crate::state::AppState;

/// Request ID middleware - adds unique ID to each request
pub async fn request_id(mut request: Request<Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    // Store in extensions for later use
    request.extensions_mut().insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap_or_else(|_| "unknown".parse().unwrap()),
    );

    response
}

/// Request ID wrapper
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Request logging middleware
pub async fn request_logging(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Create span for this request
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );

    let response = {
        let _guard = span.enter();
        next.run(request).await
    };

    let duration = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed with error"
        );
    } else {
        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }

    response
}

/// Request timeout middleware
pub async fn request_timeout(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let timeout = Duration::from_secs(30);

    match tokio::time::timeout(timeout, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            warn!("Request timed out after {:?}", timeout);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Rate limiting state for a client
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub reset_at: Instant,
}

/// Simple in-memory rate limiter (for production, use Redis)
pub async fn rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Get client identifier (IP address)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let rate_limit = &state.config.rate_limit;

    // Check if rate limiting is enabled
    if !rate_limit.enabled {
        return next.run(request).await;
    }

    // Check if path is exempt from rate limiting
    let path = request.uri().path();
    for exempt_path in &rate_limit.exempt_paths {
        if path.starts_with(exempt_path) {
            return next.run(request).await;
        }
    }

    // Simple rate limit check using cache
    let cache_key = format!("rate_limit:{}", client_ip);
    let current_count: u32 = state
        .cache
        .get(&cache_key)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);

    if current_count >= rate_limit.requests_per_window {
        let mut response = Response::new(Body::from("Too many requests"));
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        response.headers_mut().insert(
            "retry-after",
            "60".parse().unwrap(),
        );
        return response;
    }

    // Increment counter
    let _ = state
        .cache
        .set(&cache_key, &(current_count + 1), Some(Duration::from_secs(60)))
        .await;

    let mut response = next.run(request).await;

    // Add rate limit headers
    response.headers_mut().insert(
        "x-ratelimit-limit",
        rate_limit.requests_per_window.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "x-ratelimit-remaining",
        (rate_limit.requests_per_window.saturating_sub(current_count).saturating_sub(1))
            .to_string()
            .parse()
            .unwrap(),
    );

    response
}

/// CORS middleware configuration
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            "x-request-id".parse().unwrap(),
        ])
        .expose_headers([
            "x-request-id".parse().unwrap(),
            "x-ratelimit-limit".parse().unwrap(),
            "x-ratelimit-remaining".parse().unwrap(),
        ])
        .max_age(Duration::from_secs(3600))
}

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// Enable HSTS (HTTP Strict Transport Security)
    pub enable_hsts: bool,
    /// HSTS max-age in seconds (default: 1 year)
    pub hsts_max_age: u64,
    /// Include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Enable HSTS preload
    pub hsts_preload: bool,
    /// Content Security Policy
    pub csp: String,
    /// X-Frame-Options value
    pub frame_options: String,
    /// Referrer-Policy value
    pub referrer_policy: String,
    /// Permissions-Policy value
    pub permissions_policy: String,
    /// Enable Cross-Origin policies
    pub enable_cross_origin_policies: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: false,
            csp: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'self'; base-uri 'self'; form-action 'self'".to_string(),
            frame_options: "SAMEORIGIN".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()".to_string(),
            enable_cross_origin_policies: true,
        }
    }
}

/// Security headers middleware with enhanced protection
pub async fn security_headers(request: Request<Body>, next: Next) -> Response {
    security_headers_with_config(request, next, &SecurityHeadersConfig::default()).await
}

/// Security headers middleware with custom configuration
pub async fn security_headers_with_config(
    request: Request<Body>,
    next: Next,
    config: &SecurityHeadersConfig,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // === Core Security Headers ===

    // Prevent MIME type sniffing
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());

    // Enable XSS protection (legacy, but still useful for older browsers)
    headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());

    // Prevent clickjacking
    headers.insert(
        "x-frame-options",
        config.frame_options.parse().unwrap_or_else(|_| "SAMEORIGIN".parse().unwrap()),
    );

    // Referrer policy
    headers.insert(
        "referrer-policy",
        config.referrer_policy.parse().unwrap_or_else(|_| "strict-origin-when-cross-origin".parse().unwrap()),
    );

    // === HSTS (HTTP Strict Transport Security) ===

    if config.enable_hsts {
        let mut hsts_value = format!("max-age={}", config.hsts_max_age);
        if config.hsts_include_subdomains {
            hsts_value.push_str("; includeSubDomains");
        }
        if config.hsts_preload {
            hsts_value.push_str("; preload");
        }
        headers.insert(
            "strict-transport-security",
            hsts_value.parse().unwrap(),
        );
    }

    // === Content Security Policy ===

    if !config.csp.is_empty() {
        headers.insert(
            "content-security-policy",
            config.csp.parse().unwrap_or_else(|_| "default-src 'self'".parse().unwrap()),
        );
    }

    // === Permissions Policy (formerly Feature-Policy) ===

    if !config.permissions_policy.is_empty() {
        headers.insert(
            "permissions-policy",
            config.permissions_policy.parse().unwrap(),
        );
    }

    // === Cross-Origin Policies ===

    if config.enable_cross_origin_policies {
        // Cross-Origin-Embedder-Policy (COEP)
        // Requires resources to explicitly grant permission
        headers.insert(
            "cross-origin-embedder-policy",
            "require-corp".parse().unwrap(),
        );

        // Cross-Origin-Opener-Policy (COOP)
        // Isolates browsing context
        headers.insert(
            "cross-origin-opener-policy",
            "same-origin".parse().unwrap(),
        );

        // Cross-Origin-Resource-Policy (CORP)
        // Protects resources from being loaded by other origins
        headers.insert(
            "cross-origin-resource-policy",
            "same-origin".parse().unwrap(),
        );
    }

    // === Additional Security Headers ===

    // Prevent DNS prefetching
    headers.insert("x-dns-prefetch-control", "off".parse().unwrap());

    // Download options for IE
    headers.insert("x-download-options", "noopen".parse().unwrap());

    // Permitted cross-domain policies for Flash/PDF
    headers.insert("x-permitted-cross-domain-policies", "none".parse().unwrap());

    response
}

/// Compression middleware (via tower-http)
pub fn compression_layer() -> tower_http::compression::CompressionLayer {
    tower_http::compression::CompressionLayer::new()
}

/// Request body size limit middleware
pub async fn body_limit(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    const MAX_BODY_SIZE: u64 = 10 * 1024 * 1024; // 10MB

    if let Some(content_length) = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
    {
        if content_length > MAX_BODY_SIZE {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
    }

    Ok(next.run(request).await)
}

/// API versioning middleware
pub async fn api_version(request: Request<Body>, next: Next) -> Response {
    let version = request
        .headers()
        .get("api-version")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1")
        .to_string();

    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "api-version",
        version.parse().unwrap_or_else(|_| "1".parse().unwrap()),
    );

    response
}

/// Tenant identification middleware for multi-tenancy
pub async fn tenant_identification(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Skip if multi-tenancy is disabled
    if !state.config.multitenancy.enabled {
        return next.run(request).await;
    }

    // Try to identify tenant from subdomain or header
    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            // Try to extract from Host header (subdomain)
            request
                .headers()
                .get(header::HOST)
                .and_then(|v| v.to_str().ok())
                .and_then(|host| {
                    let parts: Vec<&str> = host.split('.').collect();
                    if parts.len() > 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                })
        });

    if let Some(tenant_id) = tenant_id {
        request.extensions_mut().insert(TenantId(tenant_id));
    }

    next.run(request).await
}

/// Tenant ID wrapper
#[derive(Clone, Debug)]
pub struct TenantId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_wrapper() {
        let id = RequestId("test-123".to_string());
        assert_eq!(id.0, "test-123");
    }

    #[test]
    fn test_tenant_id_wrapper() {
        let id = TenantId("tenant-456".to_string());
        assert_eq!(id.0, "tenant-456");
    }
}
