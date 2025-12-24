//! Auth Audit Logging (Point 73)
//!
//! Comprehensive authentication and authorization audit logging
//! for security monitoring and compliance.

use chrono::{DateTime, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use uuid::Uuid;

/// Auth event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthEventType {
    // Authentication events
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    Logout,
    LogoutAll,

    // Session events
    SessionCreated,
    SessionExtended,
    SessionExpired,
    SessionRevoked,

    // Token events
    TokenIssued,
    TokenRefreshed,
    TokenRevoked,
    TokenExpired,

    // Password events
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    PasswordResetFailed,

    // 2FA events
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorSuccess,
    TwoFactorFailure,
    RecoveryCodeUsed,

    // Account events
    AccountCreated,
    AccountLocked,
    AccountUnlocked,
    AccountDeleted,
    AccountSuspended,
    EmailVerified,
    EmailChanged,

    // Authorization events
    PermissionGranted,
    PermissionRevoked,
    RoleAssigned,
    RoleRemoved,
    AccessDenied,

    // Security events
    SuspiciousActivity,
    BruteForceDetected,
    IpBlocked,
    ApiKeyCreated,
    ApiKeyRevoked,

    // Impersonation events
    ImpersonationStarted,
    ImpersonationEnded,
}

impl AuthEventType {
    pub fn category(&self) -> EventCategory {
        match self {
            Self::LoginAttempt | Self::LoginSuccess | Self::LoginFailure
            | Self::Logout | Self::LogoutAll => EventCategory::Authentication,

            Self::SessionCreated | Self::SessionExtended | Self::SessionExpired
            | Self::SessionRevoked => EventCategory::Session,

            Self::TokenIssued | Self::TokenRefreshed | Self::TokenRevoked
            | Self::TokenExpired => EventCategory::Token,

            Self::PasswordChanged | Self::PasswordResetRequested
            | Self::PasswordResetCompleted | Self::PasswordResetFailed => EventCategory::Password,

            Self::TwoFactorEnabled | Self::TwoFactorDisabled | Self::TwoFactorSuccess
            | Self::TwoFactorFailure | Self::RecoveryCodeUsed => EventCategory::TwoFactor,

            Self::AccountCreated | Self::AccountLocked | Self::AccountUnlocked
            | Self::AccountDeleted | Self::AccountSuspended | Self::EmailVerified
            | Self::EmailChanged => EventCategory::Account,

            Self::PermissionGranted | Self::PermissionRevoked | Self::RoleAssigned
            | Self::RoleRemoved | Self::AccessDenied => EventCategory::Authorization,

            Self::SuspiciousActivity | Self::BruteForceDetected | Self::IpBlocked
            | Self::ApiKeyCreated | Self::ApiKeyRevoked => EventCategory::Security,

            Self::ImpersonationStarted | Self::ImpersonationEnded => EventCategory::Impersonation,
        }
    }

    pub fn severity(&self) -> EventSeverity {
        match self {
            Self::LoginFailure | Self::TwoFactorFailure | Self::PasswordResetFailed
            | Self::AccessDenied => EventSeverity::Warning,

            Self::SuspiciousActivity | Self::BruteForceDetected | Self::AccountLocked
            | Self::IpBlocked | Self::AccountSuspended => EventSeverity::High,

            Self::AccountDeleted | Self::ImpersonationStarted => EventSeverity::Critical,

            _ => EventSeverity::Info,
        }
    }
}

/// Event category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    Authentication,
    Session,
    Token,
    Password,
    TwoFactor,
    Account,
    Authorization,
    Security,
    Impersonation,
}

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    High,
    Critical,
}

/// Outcome of an auth event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOutcome {
    Success,
    Failure,
    Denied,
    Blocked,
}

/// Auth audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAuditEvent {
    pub id: Uuid,
    pub event_type: AuthEventType,
    pub category: EventCategory,
    pub severity: EventSeverity,
    pub outcome: EventOutcome,

    /// User who performed the action (if authenticated)
    pub user_id: Option<Uuid>,
    /// Target user (for actions on other users)
    pub target_user_id: Option<Uuid>,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Tenant ID for multi-tenant systems
    pub tenant_id: Option<Uuid>,

    /// Request information
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub request_path: Option<String>,
    pub request_method: Option<String>,

    /// Event details
    pub description: String,
    pub details: HashMap<String, serde_json::Value>,
    pub failure_reason: Option<String>,

    /// Timestamps
    pub occurred_at: DateTime<Utc>,

    /// Geolocation (if available)
    pub geo_country: Option<String>,
    pub geo_city: Option<String>,
}

impl AuthAuditEvent {
    pub fn new(event_type: AuthEventType, outcome: EventOutcome, ip_address: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_type,
            category: event_type.category(),
            severity: event_type.severity(),
            outcome,
            user_id: None,
            target_user_id: None,
            session_id: None,
            tenant_id: None,
            ip_address: ip_address.into(),
            user_agent: None,
            request_id: None,
            request_path: None,
            request_method: None,
            description: String::new(),
            details: HashMap::new(),
            failure_reason: None,
            occurred_at: Utc::now(),
            geo_country: None,
            geo_city: None,
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_target_user(mut self, user_id: Uuid) -> Self {
        self.target_user_id = Some(user_id);
        self
    }

    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_request(mut self, request_id: impl Into<String>, path: impl Into<String>, method: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self.request_path = Some(path.into());
        self.request_method = Some(method.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.details.insert(key.into(), v);
        }
        self
    }

    pub fn with_failure_reason(mut self, reason: impl Into<String>) -> Self {
        self.failure_reason = Some(reason.into());
        self
    }

    pub fn with_geo(mut self, country: Option<String>, city: Option<String>) -> Self {
        self.geo_country = country;
        self.geo_city = city;
        self
    }
}

/// Builder for common auth events
pub struct AuthEventBuilder;

impl AuthEventBuilder {
    pub fn login_success(user_id: Uuid, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::LoginSuccess, EventOutcome::Success, ip)
            .with_user(user_id)
            .with_description("User logged in successfully")
    }

    pub fn login_failure(username: &str, ip: impl Into<String>, reason: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::LoginFailure, EventOutcome::Failure, ip)
            .with_description(format!("Login failed for '{}'", username))
            .with_failure_reason(reason)
            .with_detail("username", username)
    }

    pub fn logout(user_id: Uuid, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::Logout, EventOutcome::Success, ip)
            .with_user(user_id)
            .with_description("User logged out")
    }

    pub fn password_changed(user_id: Uuid, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::PasswordChanged, EventOutcome::Success, ip)
            .with_user(user_id)
            .with_description("Password changed successfully")
    }

    pub fn password_reset_requested(email: &str, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::PasswordResetRequested, EventOutcome::Success, ip)
            .with_description(format!("Password reset requested for {}", email))
            .with_detail("email", email)
    }

    pub fn two_factor_enabled(user_id: Uuid, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::TwoFactorEnabled, EventOutcome::Success, ip)
            .with_user(user_id)
            .with_description("Two-factor authentication enabled")
    }

    pub fn access_denied(user_id: Option<Uuid>, resource: &str, ip: impl Into<String>) -> AuthAuditEvent {
        let mut event = AuthAuditEvent::new(AuthEventType::AccessDenied, EventOutcome::Denied, ip)
            .with_description(format!("Access denied to resource: {}", resource))
            .with_detail("resource", resource);

        if let Some(uid) = user_id {
            event = event.with_user(uid);
        }
        event
    }

    pub fn brute_force_detected(identifier: &str, ip: impl Into<String>, attempt_count: u32) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::BruteForceDetected, EventOutcome::Blocked, ip)
            .with_description(format!("Brute force attack detected for '{}'", identifier))
            .with_detail("identifier", identifier)
            .with_detail("attempt_count", attempt_count)
    }

    pub fn impersonation_started(admin_id: Uuid, target_id: Uuid, ip: impl Into<String>) -> AuthAuditEvent {
        AuthAuditEvent::new(AuthEventType::ImpersonationStarted, EventOutcome::Success, ip)
            .with_user(admin_id)
            .with_target_user(target_id)
            .with_description(format!("Admin started impersonating user"))
    }
}

/// Audit log query filters
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub user_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub event_types: Option<Vec<AuthEventType>>,
    pub categories: Option<Vec<EventCategory>>,
    pub severities: Option<Vec<EventSeverity>>,
    pub outcomes: Option<Vec<EventOutcome>>,
    pub ip_address: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub tenant_id: Option<Uuid>,
}

/// Audit log storage trait
#[async_trait::async_trait]
pub trait AuditLogStore: Send + Sync {
    /// Log an event
    async fn log(&self, event: &AuthAuditEvent) -> Result<()>;

    /// Query events
    async fn query(&self, filter: &AuditLogFilter, limit: usize, offset: usize) -> Result<Vec<AuthAuditEvent>>;

    /// Count events matching filter
    async fn count(&self, filter: &AuditLogFilter) -> Result<u64>;

    /// Get event by ID
    async fn get(&self, id: Uuid) -> Result<Option<AuthAuditEvent>>;

    /// Get events for a user
    async fn get_user_events(&self, user_id: Uuid, limit: usize) -> Result<Vec<AuthAuditEvent>>;

    /// Cleanup old events
    async fn cleanup(&self, older_than: DateTime<Utc>) -> Result<u64>;
}

/// Auth audit logger
pub struct AuditLogger<S: AuditLogStore> {
    store: S,
    enabled: bool,
    min_severity: EventSeverity,
}

impl<S: AuditLogStore> AuditLogger<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            enabled: true,
            min_severity: EventSeverity::Info,
        }
    }

    pub fn with_min_severity(mut self, severity: EventSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Log an event
    pub async fn log(&self, event: AuthAuditEvent) -> Result<()> {
        if !self.enabled || event.severity < self.min_severity {
            return Ok(());
        }

        self.store.log(&event).await
    }

    /// Query events
    pub async fn query(&self, filter: &AuditLogFilter, limit: usize, offset: usize) -> Result<Vec<AuthAuditEvent>> {
        self.store.query(filter, limit, offset).await
    }

    /// Get user's recent auth activity
    pub async fn get_user_activity(&self, user_id: Uuid, limit: usize) -> Result<Vec<AuthAuditEvent>> {
        self.store.get_user_events(user_id, limit).await
    }

    /// Check for suspicious patterns
    pub async fn detect_suspicious_activity(&self, user_id: Uuid) -> Result<Vec<SecurityAlert>> {
        let events = self.store.get_user_events(user_id, 100).await?;
        let mut alerts = Vec::new();

        // Check for multiple failed logins
        let recent_failures: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == AuthEventType::LoginFailure)
            .take(10)
            .collect();

        if recent_failures.len() >= 5 {
            alerts.push(SecurityAlert {
                alert_type: AlertType::MultipleFailedLogins,
                severity: EventSeverity::Warning,
                description: format!("{} failed login attempts detected", recent_failures.len()),
                user_id: Some(user_id),
                ip_addresses: recent_failures.iter().map(|e| e.ip_address.clone()).collect(),
                detected_at: Utc::now(),
            });
        }

        // Check for logins from multiple IPs
        let recent_logins: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == AuthEventType::LoginSuccess)
            .take(20)
            .collect();

        let unique_ips: std::collections::HashSet<_> = recent_logins.iter().map(|e| &e.ip_address).collect();
        if unique_ips.len() > 5 {
            alerts.push(SecurityAlert {
                alert_type: AlertType::MultipleLocations,
                severity: EventSeverity::Warning,
                description: format!("Logins from {} different IP addresses", unique_ips.len()),
                user_id: Some(user_id),
                ip_addresses: unique_ips.into_iter().cloned().collect(),
                detected_at: Utc::now(),
            });
        }

        Ok(alerts)
    }

    /// Cleanup old audit logs
    pub async fn cleanup(&self, older_than: DateTime<Utc>) -> Result<u64> {
        self.store.cleanup(older_than).await
    }
}

/// Security alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub alert_type: AlertType,
    pub severity: EventSeverity,
    pub description: String,
    pub user_id: Option<Uuid>,
    pub ip_addresses: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// Alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    MultipleFailedLogins,
    MultipleLocations,
    UnusualTime,
    NewDevice,
    PasswordSpray,
    AccountTakeover,
}

/// In-memory audit log store
pub struct InMemoryAuditLogStore {
    events: RwLock<Vec<AuthAuditEvent>>,
    max_events: usize,
}

impl InMemoryAuditLogStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            max_events,
        }
    }
}

impl Default for InMemoryAuditLogStore {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[async_trait::async_trait]
impl AuditLogStore for InMemoryAuditLogStore {
    async fn log(&self, event: &AuthAuditEvent) -> Result<()> {
        let mut events = self.events.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        events.insert(0, event.clone());

        // Trim to max size
        if events.len() > self.max_events {
            events.truncate(self.max_events);
        }

        Ok(())
    }

    async fn query(&self, filter: &AuditLogFilter, limit: usize, offset: usize) -> Result<Vec<AuthAuditEvent>> {
        let events = self.events.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let filtered: Vec<_> = events
            .iter()
            .filter(|e| {
                if let Some(user_id) = filter.user_id {
                    if e.user_id != Some(user_id) {
                        return false;
                    }
                }
                if let Some(ref types) = filter.event_types {
                    if !types.contains(&e.event_type) {
                        return false;
                    }
                }
                if let Some(from) = filter.from_date {
                    if e.occurred_at < from {
                        return false;
                    }
                }
                if let Some(to) = filter.to_date {
                    if e.occurred_at > to {
                        return false;
                    }
                }
                true
            })
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn count(&self, filter: &AuditLogFilter) -> Result<u64> {
        let events = self.query(filter, usize::MAX, 0).await?;
        Ok(events.len() as u64)
    }

    async fn get(&self, id: Uuid) -> Result<Option<AuthAuditEvent>> {
        let events = self.events.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(events.iter().find(|e| e.id == id).cloned())
    }

    async fn get_user_events(&self, user_id: Uuid, limit: usize) -> Result<Vec<AuthAuditEvent>> {
        self.query(&AuditLogFilter { user_id: Some(user_id), ..Default::default() }, limit, 0).await
    }

    async fn cleanup(&self, older_than: DateTime<Utc>) -> Result<u64> {
        let mut events = self.events.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        let before = events.len();
        events.retain(|e| e.occurred_at >= older_than);
        Ok((before - events.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_event() {
        let store = InMemoryAuditLogStore::default();
        let logger = AuditLogger::new(store);

        let event = AuthEventBuilder::login_success(Uuid::now_v7(), "192.168.1.1");
        logger.log(event).await.unwrap();

        let events = logger.query(&AuditLogFilter::default(), 10, 0).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuthEventType::LoginSuccess);
    }

    #[tokio::test]
    async fn test_query_by_user() {
        let store = InMemoryAuditLogStore::default();
        let logger = AuditLogger::new(store);

        let user1 = Uuid::now_v7();
        let user2 = Uuid::now_v7();

        logger.log(AuthEventBuilder::login_success(user1, "1.1.1.1")).await.unwrap();
        logger.log(AuthEventBuilder::login_success(user2, "2.2.2.2")).await.unwrap();
        logger.log(AuthEventBuilder::logout(user1, "1.1.1.1")).await.unwrap();

        let events = logger.get_user_activity(user1, 10).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_event_severity() {
        assert_eq!(AuthEventType::LoginSuccess.severity(), EventSeverity::Info);
        assert_eq!(AuthEventType::LoginFailure.severity(), EventSeverity::Warning);
        assert_eq!(AuthEventType::BruteForceDetected.severity(), EventSeverity::High);
        assert_eq!(AuthEventType::AccountDeleted.severity(), EventSeverity::Critical);
    }

    #[test]
    fn test_event_builder() {
        let event = AuthEventBuilder::brute_force_detected("admin", "10.0.0.1", 10);

        assert_eq!(event.event_type, AuthEventType::BruteForceDetected);
        assert_eq!(event.outcome, EventOutcome::Blocked);
        assert!(event.details.contains_key("attempt_count"));
    }
}
