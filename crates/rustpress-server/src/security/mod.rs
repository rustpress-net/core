//! Security middleware modules for enhanced protection.
//!
//! This module provides multiple layers of security middleware:
//! - Request validation and sanitization
//! - Content security controls
//! - Bot detection
//! - Security audit logging
//! - Request fingerprinting

pub mod bot_detection;
pub mod content_security;
pub mod fingerprint;
pub mod request_validation;
pub mod security_audit;

// Re-export commonly used types
pub use bot_detection::{BotDetectionConfig, BotDetectionMiddleware, BotScore, BotSignal};
pub use content_security::{ContentSecurityConfig, ContentSecurityMiddleware, ContentSecurityError};
pub use fingerprint::{ClientProfile, FingerprintConfig, FingerprintMiddleware, RequestFingerprint};
pub use request_validation::{SecurityConfig, SecurityMiddleware, ThreatType, ValidationResult};
pub use security_audit::{SecurityAuditConfig, SecurityAuditLogger, SecurityEvent, SecurityEventRecord, SecuritySeverity};
