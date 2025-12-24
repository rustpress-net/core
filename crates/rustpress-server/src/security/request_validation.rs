//! Request validation and sanitization middleware.
//!
//! Detects and blocks malicious request patterns including:
//! - SQL injection attempts
//! - XSS (Cross-Site Scripting) attacks
//! - Path traversal attacks
//! - Command injection attempts

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Types of security threats that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatType {
    SqlInjection,
    Xss,
    PathTraversal,
    CommandInjection,
    InvalidEncoding,
    SuspiciousPattern,
}

impl std::fmt::Display for ThreatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatType::SqlInjection => write!(f, "SQL Injection"),
            ThreatType::Xss => write!(f, "XSS"),
            ThreatType::PathTraversal => write!(f, "Path Traversal"),
            ThreatType::CommandInjection => write!(f, "Command Injection"),
            ThreatType::InvalidEncoding => write!(f, "Invalid Encoding"),
            ThreatType::SuspiciousPattern => write!(f, "Suspicious Pattern"),
        }
    }
}

/// Result of security validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_safe: bool,
    pub threat_type: Option<ThreatType>,
    pub matched_pattern: Option<String>,
    pub location: Option<String>,
}

impl ValidationResult {
    pub fn safe() -> Self {
        Self {
            is_safe: true,
            threat_type: None,
            matched_pattern: None,
            location: None,
        }
    }

    pub fn threat(threat_type: ThreatType, pattern: &str, location: &str) -> Self {
        Self {
            is_safe: false,
            threat_type: Some(threat_type),
            matched_pattern: Some(pattern.to_string()),
            location: Some(location.to_string()),
        }
    }
}

/// Security middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable SQL injection detection
    pub enable_sql_injection_detection: bool,
    /// Enable XSS detection
    pub enable_xss_detection: bool,
    /// Enable path traversal detection
    pub enable_path_traversal_detection: bool,
    /// Enable command injection detection
    pub enable_command_injection_detection: bool,
    /// Custom blocked patterns (regex)
    pub custom_blocked_patterns: Vec<String>,
    /// Paths to exclude from validation
    pub excluded_paths: Vec<String>,
    /// Whether to block or just log threats
    pub blocking_mode: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_sql_injection_detection: true,
            enable_xss_detection: true,
            enable_path_traversal_detection: true,
            enable_command_injection_detection: true,
            custom_blocked_patterns: Vec::new(),
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
            ],
            blocking_mode: true,
        }
    }
}

/// Security middleware for request validation
#[derive(Clone)]
pub struct SecurityMiddleware {
    config: Arc<SecurityConfig>,
    custom_patterns: Vec<Regex>,
}

impl SecurityMiddleware {
    pub fn new(config: SecurityConfig) -> Self {
        let custom_patterns = config
            .custom_blocked_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            config: Arc::new(config),
            custom_patterns,
        }
    }

    /// Validate a string for security threats
    pub fn validate(&self, input: &str, location: &str) -> ValidationResult {
        // Check for SQL injection
        if self.config.enable_sql_injection_detection {
            if let Some(pattern) = detect_sql_injection(input) {
                return ValidationResult::threat(ThreatType::SqlInjection, &pattern, location);
            }
        }

        // Check for XSS
        if self.config.enable_xss_detection {
            if let Some(pattern) = detect_xss(input) {
                return ValidationResult::threat(ThreatType::Xss, &pattern, location);
            }
        }

        // Check for path traversal
        if self.config.enable_path_traversal_detection {
            if let Some(pattern) = detect_path_traversal(input) {
                return ValidationResult::threat(ThreatType::PathTraversal, &pattern, location);
            }
        }

        // Check for command injection
        if self.config.enable_command_injection_detection {
            if let Some(pattern) = detect_command_injection(input) {
                return ValidationResult::threat(ThreatType::CommandInjection, &pattern, location);
            }
        }

        // Check custom patterns
        for pattern in &self.custom_patterns {
            if pattern.is_match(input) {
                return ValidationResult::threat(
                    ThreatType::SuspiciousPattern,
                    pattern.as_str(),
                    location,
                );
            }
        }

        ValidationResult::safe()
    }

    /// Check if path is excluded from validation
    pub fn is_excluded(&self, path: &str) -> bool {
        self.config.excluded_paths.iter().any(|p| path.starts_with(p))
    }
}

// SQL Injection patterns
static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Union-based injection
        Regex::new(r"(?i)\bunion\s+(all\s+)?select\b").unwrap(),
        // Boolean-based injection
        Regex::new(r"(?i)\bor\s+1\s*=\s*1\b").unwrap(),
        Regex::new(r"(?i)\band\s+1\s*=\s*1\b").unwrap(),
        Regex::new(r"(?i)\bor\s+'[^']*'\s*=\s*'[^']*'").unwrap(),
        // Comment injection
        Regex::new(r"--\s*$").unwrap(),
        Regex::new(r"/\*.*\*/").unwrap(),
        // Stacked queries
        Regex::new(r";\s*(drop|delete|truncate|update|insert|alter)\b").unwrap(),
        // Time-based injection
        Regex::new(r"(?i)\bwaitfor\s+delay\b").unwrap(),
        Regex::new(r"(?i)\bsleep\s*\(").unwrap(),
        Regex::new(r"(?i)\bbenchmark\s*\(").unwrap(),
        // Error-based injection
        Regex::new(r"(?i)\bextractvalue\s*\(").unwrap(),
        Regex::new(r"(?i)\bupdatexml\s*\(").unwrap(),
        // Common SQL keywords in suspicious contexts
        Regex::new(r"(?i)'\s*(or|and)\s+").unwrap(),
        Regex::new(r"(?i)\bdrop\s+table\b").unwrap(),
        Regex::new(r"(?i)\bexec(\s+|\()").unwrap(),
        Regex::new(r"(?i)\bxp_cmdshell\b").unwrap(),
        // Hex encoding bypass
        Regex::new(r"0x[0-9a-fA-F]+").unwrap(),
    ]
});

fn detect_sql_injection(input: &str) -> Option<String> {
    let decoded = url_decode(input);
    for pattern in SQL_INJECTION_PATTERNS.iter() {
        if pattern.is_match(&decoded) {
            return Some(pattern.as_str().to_string());
        }
    }
    None
}

// XSS patterns
static XSS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Script tags
        Regex::new(r"(?i)<script[\s>]").unwrap(),
        Regex::new(r"(?i)</script>").unwrap(),
        // Event handlers
        Regex::new(r"(?i)\bon\w+\s*=").unwrap(),
        // JavaScript protocol
        Regex::new(r"(?i)javascript\s*:").unwrap(),
        Regex::new(r"(?i)vbscript\s*:").unwrap(),
        Regex::new(r"(?i)data\s*:\s*text/html").unwrap(),
        // SVG/XML attacks
        Regex::new(r"(?i)<svg[\s>]").unwrap(),
        Regex::new(r"(?i)<iframe[\s>]").unwrap(),
        Regex::new(r"(?i)<object[\s>]").unwrap(),
        Regex::new(r"(?i)<embed[\s>]").unwrap(),
        // Expression injection
        Regex::new(r"(?i)expression\s*\(").unwrap(),
        // Encoded attacks
        Regex::new(r"&#x?[0-9a-fA-F]+;").unwrap(),
        // Base64 encoded scripts
        Regex::new(r"(?i)base64\s*,").unwrap(),
    ]
});

fn detect_xss(input: &str) -> Option<String> {
    let decoded = url_decode(input);
    // Also decode HTML entities
    let html_decoded = decode_html_entities(&decoded);

    for pattern in XSS_PATTERNS.iter() {
        if pattern.is_match(&html_decoded) {
            return Some(pattern.as_str().to_string());
        }
    }
    None
}

// Path traversal patterns
static PATH_TRAVERSAL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Basic traversal
        Regex::new(r"\.\.[\\/]").unwrap(),
        // URL encoded
        Regex::new(r"%2e%2e[%2f%5c]").unwrap(),
        Regex::new(r"%252e%252e[%252f%255c]").unwrap(),
        // Double encoding
        Regex::new(r"\.\.%2f").unwrap(),
        Regex::new(r"\.\.%5c").unwrap(),
        // Unicode encoding
        Regex::new(r"%c0%ae").unwrap(),
        Regex::new(r"%c1%1c").unwrap(),
        // Null byte injection
        Regex::new(r"%00").unwrap(),
        // Absolute paths (Unix)
        Regex::new(r"^/etc/").unwrap(),
        Regex::new(r"^/proc/").unwrap(),
        Regex::new(r"^/var/").unwrap(),
        // Windows paths
        Regex::new(r"(?i)^[a-z]:[\\/]").unwrap(),
    ]
});

fn detect_path_traversal(input: &str) -> Option<String> {
    let decoded = url_decode(input);
    for pattern in PATH_TRAVERSAL_PATTERNS.iter() {
        if pattern.is_match(&decoded) {
            return Some(pattern.as_str().to_string());
        }
    }
    None
}

// Command injection patterns
static COMMAND_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Shell metacharacters
        Regex::new(r"[|;&]").unwrap(),
        Regex::new(r"\$\(").unwrap(),
        Regex::new(r"`[^`]+`").unwrap(),
        // Common commands
        Regex::new(r"(?i)\b(cat|ls|dir|type|wget|curl|nc|netcat)\s+").unwrap(),
        Regex::new(r"(?i)\b(rm|del|rmdir)\s+(-rf?\s+)?").unwrap(),
        Regex::new(r"(?i)\b(chmod|chown)\s+").unwrap(),
        // Pipes and redirects
        Regex::new(r">\s*/").unwrap(),
        Regex::new(r"<\s*/").unwrap(),
        // Shell expansion
        Regex::new(r"\$\{[^}]+\}").unwrap(),
    ]
});

fn detect_command_injection(input: &str) -> Option<String> {
    let decoded = url_decode(input);
    for pattern in COMMAND_INJECTION_PATTERNS.iter() {
        if pattern.is_match(&decoded) {
            return Some(pattern.as_str().to_string());
        }
    }
    None
}

/// URL decode helper
fn url_decode(input: &str) -> String {
    percent_decode(input.as_bytes())
}

fn percent_decode(input: &[u8]) -> String {
    let mut result = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            if let (Some(h), Some(l)) = (
                hex_to_byte(input[i + 1]),
                hex_to_byte(input[i + 2]),
            ) {
                result.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        result.push(input[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

fn hex_to_byte(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Decode HTML entities
fn decode_html_entities(input: &str) -> String {
    let mut result = input.to_string();

    // Decode common HTML entities
    let entities = [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
        ("&quot;", "\""),
        ("&apos;", "'"),
        ("&#60;", "<"),
        ("&#62;", ">"),
        ("&#x3c;", "<"),
        ("&#x3e;", ">"),
        ("&#x3C;", "<"),
        ("&#x3E;", ">"),
    ];

    for (entity, replacement) in entities {
        result = result.replace(entity, replacement);
    }

    result
}

/// Security validation middleware function
pub async fn request_validation(
    State(security): State<SecurityMiddleware>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Skip excluded paths
    if security.is_excluded(&path) {
        return next.run(request).await;
    }

    // Validate URI path
    let path_result = security.validate(&path, "uri_path");
    if !path_result.is_safe {
        return create_blocked_response(&path_result);
    }

    // Validate query string
    if let Some(query) = request.uri().query() {
        let query_result = security.validate(query, "query_string");
        if !query_result.is_safe {
            return create_blocked_response(&query_result);
        }
    }

    // Validate headers that could contain user input
    for (name, value) in request.headers() {
        let header_name = name.as_str();
        // Only validate certain headers that could be exploited
        if matches!(
            header_name,
            "referer" | "user-agent" | "x-forwarded-for" | "x-custom-header"
        ) {
            if let Ok(value_str) = value.to_str() {
                let header_result = security.validate(value_str, header_name);
                if !header_result.is_safe && security.config.blocking_mode {
                    tracing::warn!(
                        threat_type = ?header_result.threat_type,
                        pattern = ?header_result.matched_pattern,
                        location = header_name,
                        "Security threat detected in header"
                    );
                    // Don't block on headers, just log
                }
            }
        }
    }

    next.run(request).await
}

fn create_blocked_response(result: &ValidationResult) -> Response {
    tracing::warn!(
        threat_type = ?result.threat_type,
        pattern = ?result.matched_pattern,
        location = ?result.location,
        "Request blocked due to security threat"
    );

    (
        StatusCode::BAD_REQUEST,
        "Request blocked due to security policy",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        assert!(detect_sql_injection("1 OR 1=1").is_some());
        assert!(detect_sql_injection("'; DROP TABLE users--").is_some());
        assert!(detect_sql_injection("UNION SELECT * FROM users").is_some());
        assert!(detect_sql_injection("normal text").is_none());
    }

    #[test]
    fn test_xss_detection() {
        assert!(detect_xss("<script>alert('xss')</script>").is_some());
        assert!(detect_xss("onclick=alert(1)").is_some());
        assert!(detect_xss("javascript:alert(1)").is_some());
        assert!(detect_xss("normal text").is_none());
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(detect_path_traversal("../../../etc/passwd").is_some());
        assert!(detect_path_traversal("%2e%2e%2f").is_some());
        assert!(detect_path_traversal("normal/path/file.txt").is_none());
    }

    #[test]
    fn test_command_injection_detection() {
        assert!(detect_command_injection("; rm -rf /").is_some());
        assert!(detect_command_injection("$(whoami)").is_some());
        assert!(detect_command_injection("normal input").is_none());
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("%2e%2e%2f"), "../");
        assert_eq!(url_decode("hello%20world"), "hello world");
    }

    #[test]
    fn test_security_middleware_validation() {
        let config = SecurityConfig::default();
        let middleware = SecurityMiddleware::new(config);

        let result = middleware.validate("normal text", "test");
        assert!(result.is_safe);

        let result = middleware.validate("'; DROP TABLE--", "test");
        assert!(!result.is_safe);
        assert_eq!(result.threat_type, Some(ThreatType::SqlInjection));
    }
}
