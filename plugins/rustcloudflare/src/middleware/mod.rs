//! Middleware for RustCloudflare

use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};

/// Add Cloudflare-specific headers to responses
pub async fn cloudflare_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let response = next.run(request).await;
    // Add any Cloudflare-specific headers here
    response
}

/// Cache control middleware
pub async fn cache_control(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let response = next.run(request).await;
    // Add cache control headers based on route
    response
}
