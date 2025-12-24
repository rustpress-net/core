//! Security audit logging middleware.
//!
//! Provides centralized logging of security-related events including:
//! - Blocked requests
//! - Authentication failures
//! - Authorization denials
//! - Rate limit violations
//! - Suspicious pattern detections

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityEvent {
    /// Request was blocked due to security policy
    BlockedRequest {
        reason: String,
        pattern: Option<String>,
    },
    /// Authentication attempt failed
    AuthenticationFailure {
        username: Option<String>,
        method: String,
        reason: String,
    },
    /// Authorization was denied
    AuthorizationDenied {
        user_id: Option<String>,
        resource: String,
        action: String,
    },
    /// Rate limit was exceeded
    RateLimitExceeded {
        limit: u32,
        window_seconds: u64,
    },
    /// Suspicious pattern was detected
    SuspiciousPattern {
        pattern_type: String,
        details: String,
    },
    /// Bot was detected
    BotDetected {
        score: u32,
        signals: Vec<String>,
    },
    /// Brute force attempt detected
    BruteForceAttempt {
        target: String,
        attempt_count: u32,
    },
    /// Session anomaly detected
    SessionAnomaly {
        session_id: String,
        anomaly_type: String,
    },
    /// IP was blocked
    IpBlocked {
        reason: String,
    },
    /// CSRF validation failed
    CsrfFailure {
        reason: String,
    },
}

impl SecurityEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            SecurityEvent::BlockedRequest { .. } => "blocked_request",
            SecurityEvent::AuthenticationFailure { .. } => "auth_failure",
            SecurityEvent::AuthorizationDenied { .. } => "authz_denied",
            SecurityEvent::RateLimitExceeded { .. } => "rate_limit",
            SecurityEvent::SuspiciousPattern { .. } => "suspicious_pattern",
            SecurityEvent::BotDetected { .. } => "bot_detected",
            SecurityEvent::BruteForceAttempt { .. } => "brute_force",
            SecurityEvent::SessionAnomaly { .. } => "session_anomaly",
            SecurityEvent::IpBlocked { .. } => "ip_blocked",
            SecurityEvent::CsrfFailure { .. } => "csrf_failure",
        }
    }

    pub fn severity(&self) -> SecuritySeverity {
        match self {
            SecurityEvent::BlockedRequest { .. } => SecuritySeverity::Medium,
            SecurityEvent::AuthenticationFailure { .. } => SecuritySeverity::Medium,
            SecurityEvent::AuthorizationDenied { .. } => SecuritySeverity::Low,
            SecurityEvent::RateLimitExceeded { .. } => SecuritySeverity::Low,
            SecurityEvent::SuspiciousPattern { .. } => SecuritySeverity::High,
            SecurityEvent::BotDetected { .. } => SecuritySeverity::Medium,
            SecurityEvent::BruteForceAttempt { .. } => SecuritySeverity::High,
            SecurityEvent::SessionAnomaly { .. } => SecuritySeverity::High,
            SecurityEvent::IpBlocked { .. } => SecuritySeverity::Medium,
            SecurityEvent::CsrfFailure { .. } => SecuritySeverity::Medium,
        }
    }
}

/// Severity levels for security events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A recorded security event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEventRecord {
    /// Unique event ID
    pub id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The security event
    pub event: SecurityEvent,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Request path
    pub path: Option<String>,
    /// Request method
    pub method: Option<String>,
    /// Additional context
    pub context: Option<serde_json::Value>,
}

impl SecurityEventRecord {
    pub fn new(event: SecurityEvent) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            event,
            request_id: None,
            client_ip: None,
            user_agent: None,
            path: None,
            method: None,
            context: None,
        }
    }

    pub fn with_request_info(
        mut self,
        request_id: Option<String>,
        client_ip: Option<String>,
        user_agent: Option<String>,
        path: Option<String>,
        method: Option<String>,
    ) -> Self {
        self.request_id = request_id;
        self.client_ip = client_ip;
        self.user_agent = user_agent;
        self.path = path;
        self.method = method;
        self
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// Configuration for the security audit logger
#[derive(Debug, Clone)]
pub struct SecurityAuditConfig {
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Whether to log to tracing as well
    pub log_to_tracing: bool,
    /// Minimum severity to log
    pub min_severity: SecuritySeverity,
    /// Whether to include request body in logs (careful with sensitive data)
    pub include_request_body: bool,
}

impl Default for SecurityAuditConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            log_to_tracing: true,
            min_severity: SecuritySeverity::Low,
            include_request_body: false,
        }
    }
}

/// Security audit logger
#[derive(Clone)]
pub struct SecurityAuditLogger {
    events: Arc<RwLock<VecDeque<SecurityEventRecord>>>,
    config: Arc<SecurityAuditConfig>,
}

impl SecurityAuditLogger {
    pub fn new(config: SecurityAuditConfig) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_events))),
            config: Arc::new(config),
        }
    }

    /// Log a security event
    pub fn log(&self, record: SecurityEventRecord) {
        let severity = record.event.severity();

        // Check minimum severity
        if severity < self.config.min_severity {
            return;
        }

        // Log to tracing if enabled
        if self.config.log_to_tracing {
            match severity {
                SecuritySeverity::Low => {
                    tracing::info!(
                        event_type = record.event.event_type(),
                        severity = %severity,
                        request_id = ?record.request_id,
                        client_ip = ?record.client_ip,
                        path = ?record.path,
                        "Security event: {:?}",
                        record.event
                    );
                }
                SecuritySeverity::Medium => {
                    tracing::warn!(
                        event_type = record.event.event_type(),
                        severity = %severity,
                        request_id = ?record.request_id,
                        client_ip = ?record.client_ip,
                        path = ?record.path,
                        "Security event: {:?}",
                        record.event
                    );
                }
                SecuritySeverity::High | SecuritySeverity::Critical => {
                    tracing::error!(
                        event_type = record.event.event_type(),
                        severity = %severity,
                        request_id = ?record.request_id,
                        client_ip = ?record.client_ip,
                        path = ?record.path,
                        "Security event: {:?}",
                        record.event
                    );
                }
            }
        }

        // Store in memory
        let mut events = self.events.write();
        if events.len() >= self.config.max_events {
            events.pop_front();
        }
        events.push_back(record);
    }

    /// Log a security event with request context
    pub fn log_with_request(&self, event: SecurityEvent, request: &Request<Body>) {
        let request_id = request
            .extensions()
            .get::<crate::middleware::RequestId>()
            .map(|r| r.0.clone());

        let client_ip = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string());

        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let path = Some(request.uri().path().to_string());
        let method = Some(request.method().to_string());

        let record = SecurityEventRecord::new(event)
            .with_request_info(request_id, client_ip, user_agent, path, method);

        self.log(record);
    }

    /// Get recent events
    pub fn get_events(&self, limit: usize) -> Vec<SecurityEventRecord> {
        let events = self.events.read();
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &str, limit: usize) -> Vec<SecurityEventRecord> {
        let events = self.events.read();
        events
            .iter()
            .rev()
            .filter(|e| e.event.event_type() == event_type)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(
        &self,
        min_severity: SecuritySeverity,
        limit: usize,
    ) -> Vec<SecurityEventRecord> {
        let events = self.events.read();
        events
            .iter()
            .rev()
            .filter(|e| e.event.severity() >= min_severity)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get events by IP
    pub fn get_events_by_ip(&self, ip: &str, limit: usize) -> Vec<SecurityEventRecord> {
        let events = self.events.read();
        events
            .iter()
            .rev()
            .filter(|e| e.client_ip.as_deref() == Some(ip))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get event count by type in the last N seconds
    pub fn count_events_since(
        &self,
        event_type: &str,
        seconds: i64,
    ) -> usize {
        let cutoff = Utc::now() - chrono::Duration::seconds(seconds);
        let events = self.events.read();
        events
            .iter()
            .filter(|e| e.timestamp > cutoff && e.event.event_type() == event_type)
            .count()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
    }

    /// Export events as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let events = self.events.read();
        serde_json::to_string_pretty(&*events)
    }
}

/// Security audit middleware
pub async fn security_audit(
    State(logger): State<SecurityAuditLogger>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();

    let request_id = request
        .extensions()
        .get::<crate::middleware::RequestId>()
        .map(|r| r.0.clone());

    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    // Log security-relevant status codes
    if status == StatusCode::UNAUTHORIZED {
        logger.log(
            SecurityEventRecord::new(SecurityEvent::AuthenticationFailure {
                username: None,
                method: "unknown".to_string(),
                reason: "Unauthorized response".to_string(),
            })
            .with_request_info(
                request_id.clone(),
                client_ip.clone(),
                None,
                Some(path.clone()),
                Some(method.clone()),
            ),
        );
    } else if status == StatusCode::FORBIDDEN {
        logger.log(
            SecurityEventRecord::new(SecurityEvent::AuthorizationDenied {
                user_id: None,
                resource: path.clone(),
                action: method.clone(),
            })
            .with_request_info(
                request_id.clone(),
                client_ip.clone(),
                None,
                Some(path.clone()),
                Some(method.clone()),
            ),
        );
    } else if status == StatusCode::TOO_MANY_REQUESTS {
        logger.log(
            SecurityEventRecord::new(SecurityEvent::RateLimitExceeded {
                limit: 0,
                window_seconds: 0,
            })
            .with_request_info(
                request_id,
                client_ip,
                None,
                Some(path),
                Some(method),
            ),
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_event_type() {
        let event = SecurityEvent::BlockedRequest {
            reason: "test".to_string(),
            pattern: None,
        };
        assert_eq!(event.event_type(), "blocked_request");
    }

    #[test]
    fn test_security_event_severity() {
        let event = SecurityEvent::BruteForceAttempt {
            target: "user".to_string(),
            attempt_count: 5,
        };
        assert_eq!(event.severity(), SecuritySeverity::High);
    }

    #[test]
    fn test_security_audit_logger() {
        let config = SecurityAuditConfig::default();
        let logger = SecurityAuditLogger::new(config);

        let record = SecurityEventRecord::new(SecurityEvent::BlockedRequest {
            reason: "test".to_string(),
            pattern: Some("pattern".to_string()),
        });

        logger.log(record);

        let events = logger.get_events(10);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_event_limit() {
        let config = SecurityAuditConfig {
            max_events: 5,
            ..Default::default()
        };
        let logger = SecurityAuditLogger::new(config);

        for i in 0..10 {
            let record = SecurityEventRecord::new(SecurityEvent::BlockedRequest {
                reason: format!("test {}", i),
                pattern: None,
            });
            logger.log(record);
        }

        let events = logger.get_events(100);
        assert_eq!(events.len(), 5);
    }
}
