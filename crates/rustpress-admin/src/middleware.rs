//! Admin middleware for authentication and authorization

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::IpAddr;

/// Admin authentication middleware
pub async fn require_admin_auth(request: Request, next: Next) -> Response {
    // Check for admin session/token
    let auth_header = request.headers().get(header::AUTHORIZATION);

    if let Some(_auth) = auth_header {
        // TODO: Validate admin token
        // For now, allow all authenticated requests
        next.run(request).await
    } else {
        // Check for session cookie
        let cookie_header = request.headers().get(header::COOKIE);

        if let Some(_cookie) = cookie_header {
            // TODO: Validate session cookie
            next.run(request).await
        } else {
            // Return unauthorized
            (
                StatusCode::UNAUTHORIZED,
                [("WWW-Authenticate", "Bearer")],
                "Admin authentication required",
            )
                .into_response()
        }
    }
}

/// IP whitelist middleware
pub async fn ip_whitelist(
    allowed_ips: &[String],
    request: Request,
    next: Next,
) -> Response {
    // If no whitelist configured, allow all
    if allowed_ips.is_empty() {
        return next.run(request).await;
    }

    // Get client IP
    let client_ip = get_client_ip(&request);

    if let Some(ip) = client_ip {
        let ip_str = ip.to_string();

        // Check if IP is in whitelist
        if allowed_ips.iter().any(|allowed| {
            // Support exact match or CIDR notation
            allowed == &ip_str || matches_cidr(allowed, &ip)
        }) {
            return next.run(request).await;
        }
    }

    // IP not allowed
    (StatusCode::FORBIDDEN, "Access denied from this IP address").into_response()
}

/// Extract client IP from request
fn get_client_ip(request: &Request) -> Option<IpAddr> {
    // Check X-Forwarded-For header first (for reverse proxies)
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(value) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(ip_str) = value.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(value) = real_ip.to_str() {
            if let Ok(ip) = value.parse() {
                return Some(ip);
            }
        }
    }

    // TODO: Get from connection info
    None
}

/// Check if IP matches CIDR notation
fn matches_cidr(cidr: &str, ip: &IpAddr) -> bool {
    // Simple implementation - for production use a proper CIDR library
    if !cidr.contains('/') {
        return false;
    }

    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    if let (Ok(network), Ok(prefix_len)) = (parts[0].parse::<IpAddr>(), parts[1].parse::<u8>()) {
        match (network, ip) {
            (IpAddr::V4(net), IpAddr::V4(addr)) => {
                let net_bits = u32::from(net);
                let addr_bits = u32::from(*addr);
                let mask = !0u32 << (32 - prefix_len);
                return (net_bits & mask) == (addr_bits & mask);
            }
            (IpAddr::V6(net), IpAddr::V6(addr)) => {
                let net_bits = u128::from(net);
                let addr_bits = u128::from(*addr);
                let mask = !0u128 << (128 - prefix_len);
                return (net_bits & mask) == (addr_bits & mask);
            }
            _ => return false,
        }
    }

    false
}

/// Rate limiting middleware for admin routes
pub async fn admin_rate_limit(request: Request, next: Next) -> Response {
    // TODO: Implement rate limiting
    // For now, just pass through
    next.run(request).await
}

/// Security headers middleware
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com https://unpkg.com https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'"
            .parse()
            .unwrap(),
    );

    // Other security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "SAMEORIGIN".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_cidr_v4() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        assert!(matches_cidr("192.168.1.0/24", &ip));
        assert!(!matches_cidr("192.168.2.0/24", &ip));
        assert!(matches_cidr("192.168.0.0/16", &ip));
    }

    #[test]
    fn test_matches_cidr_invalid() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();
        assert!(!matches_cidr("invalid", &ip));
        assert!(!matches_cidr("192.168.1.0", &ip));
    }
}
