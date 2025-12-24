//! # Audit Logging
//!
//! Comprehensive audit trail for user actions.
//!
//! Features:
//! - Action logging
//! - User activity tracking
//! - Security event logging
//! - Log retention policies
//! - Log search and filtering
//! - Export capabilities

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit event category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditCategory {
    /// Authentication events
    Authentication,
    /// User account events
    UserAccount,
    /// Content management events
    Content,
    /// Settings changes
    Settings,
    /// Security events
    Security,
    /// System events
    System,
    /// API access
    Api,
    /// Plugin events
    Plugin,
    /// Theme events
    Theme,
    /// Media events
    Media,
}

impl AuditCategory {
    pub fn name(&self) -> &'static str {
        match self {
            AuditCategory::Authentication => "Authentication",
            AuditCategory::UserAccount => "User Account",
            AuditCategory::Content => "Content",
            AuditCategory::Settings => "Settings",
            AuditCategory::Security => "Security",
            AuditCategory::System => "System",
            AuditCategory::Api => "API",
            AuditCategory::Plugin => "Plugin",
            AuditCategory::Theme => "Theme",
            AuditCategory::Media => "Media",
        }
    }
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational
    Info,
    /// Notice (normal but significant)
    Notice,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl AuditSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "Info",
            AuditSeverity::Notice => "Notice",
            AuditSeverity::Warning => "Warning",
            AuditSeverity::Error => "Error",
            AuditSeverity::Critical => "Critical",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "#64748b",
            AuditSeverity::Notice => "#3b82f6",
            AuditSeverity::Warning => "#f59e0b",
            AuditSeverity::Error => "#ef4444",
            AuditSeverity::Critical => "#dc2626",
        }
    }
}

/// Common audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    // Authentication
    LoginSuccess,
    LoginFailed,
    Logout,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorVerified,
    TwoFactorFailed,
    SessionCreated,
    SessionRevoked,

    // User Account
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserActivated,
    UserDeactivated,
    UserLocked,
    UserUnlocked,
    RoleChanged,
    ProfileUpdated,
    EmailChanged,
    AvatarUpdated,

    // Content
    PostCreated,
    PostUpdated,
    PostDeleted,
    PostPublished,
    PostUnpublished,
    PageCreated,
    PageUpdated,
    PageDeleted,
    CommentCreated,
    CommentApproved,
    CommentDeleted,
    CommentSpammed,

    // Settings
    SettingChanged,
    OptionUpdated,

    // Security
    SuspiciousActivity,
    BruteForceAttempt,
    UnauthorizedAccess,
    PermissionDenied,

    // System
    SystemStarted,
    SystemStopped,
    CacheCleared,
    DatabaseMigrated,
    BackupCreated,
    BackupRestored,

    // Plugin/Theme
    PluginInstalled,
    PluginActivated,
    PluginDeactivated,
    PluginDeleted,
    PluginUpdated,
    ThemeActivated,
    ThemeInstalled,
    ThemeDeleted,
    ThemeUpdated,

    // Media
    MediaUploaded,
    MediaDeleted,
    MediaUpdated,

    // Custom
    Custom(String),
}

impl AuditAction {
    pub fn name(&self) -> String {
        match self {
            AuditAction::LoginSuccess => "Login Success".to_string(),
            AuditAction::LoginFailed => "Login Failed".to_string(),
            AuditAction::Logout => "Logout".to_string(),
            AuditAction::PasswordChanged => "Password Changed".to_string(),
            AuditAction::PasswordResetRequested => "Password Reset Requested".to_string(),
            AuditAction::PasswordResetCompleted => "Password Reset Completed".to_string(),
            AuditAction::TwoFactorEnabled => "2FA Enabled".to_string(),
            AuditAction::TwoFactorDisabled => "2FA Disabled".to_string(),
            AuditAction::TwoFactorVerified => "2FA Verified".to_string(),
            AuditAction::TwoFactorFailed => "2FA Failed".to_string(),
            AuditAction::SessionCreated => "Session Created".to_string(),
            AuditAction::SessionRevoked => "Session Revoked".to_string(),
            AuditAction::UserCreated => "User Created".to_string(),
            AuditAction::UserUpdated => "User Updated".to_string(),
            AuditAction::UserDeleted => "User Deleted".to_string(),
            AuditAction::UserActivated => "User Activated".to_string(),
            AuditAction::UserDeactivated => "User Deactivated".to_string(),
            AuditAction::UserLocked => "User Locked".to_string(),
            AuditAction::UserUnlocked => "User Unlocked".to_string(),
            AuditAction::RoleChanged => "Role Changed".to_string(),
            AuditAction::ProfileUpdated => "Profile Updated".to_string(),
            AuditAction::EmailChanged => "Email Changed".to_string(),
            AuditAction::AvatarUpdated => "Avatar Updated".to_string(),
            AuditAction::PostCreated => "Post Created".to_string(),
            AuditAction::PostUpdated => "Post Updated".to_string(),
            AuditAction::PostDeleted => "Post Deleted".to_string(),
            AuditAction::PostPublished => "Post Published".to_string(),
            AuditAction::PostUnpublished => "Post Unpublished".to_string(),
            AuditAction::PageCreated => "Page Created".to_string(),
            AuditAction::PageUpdated => "Page Updated".to_string(),
            AuditAction::PageDeleted => "Page Deleted".to_string(),
            AuditAction::CommentCreated => "Comment Created".to_string(),
            AuditAction::CommentApproved => "Comment Approved".to_string(),
            AuditAction::CommentDeleted => "Comment Deleted".to_string(),
            AuditAction::CommentSpammed => "Comment Marked Spam".to_string(),
            AuditAction::SettingChanged => "Setting Changed".to_string(),
            AuditAction::OptionUpdated => "Option Updated".to_string(),
            AuditAction::SuspiciousActivity => "Suspicious Activity".to_string(),
            AuditAction::BruteForceAttempt => "Brute Force Attempt".to_string(),
            AuditAction::UnauthorizedAccess => "Unauthorized Access".to_string(),
            AuditAction::PermissionDenied => "Permission Denied".to_string(),
            AuditAction::SystemStarted => "System Started".to_string(),
            AuditAction::SystemStopped => "System Stopped".to_string(),
            AuditAction::CacheCleared => "Cache Cleared".to_string(),
            AuditAction::DatabaseMigrated => "Database Migrated".to_string(),
            AuditAction::BackupCreated => "Backup Created".to_string(),
            AuditAction::BackupRestored => "Backup Restored".to_string(),
            AuditAction::PluginInstalled => "Plugin Installed".to_string(),
            AuditAction::PluginActivated => "Plugin Activated".to_string(),
            AuditAction::PluginDeactivated => "Plugin Deactivated".to_string(),
            AuditAction::PluginDeleted => "Plugin Deleted".to_string(),
            AuditAction::PluginUpdated => "Plugin Updated".to_string(),
            AuditAction::ThemeActivated => "Theme Activated".to_string(),
            AuditAction::ThemeInstalled => "Theme Installed".to_string(),
            AuditAction::ThemeDeleted => "Theme Deleted".to_string(),
            AuditAction::ThemeUpdated => "Theme Updated".to_string(),
            AuditAction::MediaUploaded => "Media Uploaded".to_string(),
            AuditAction::MediaDeleted => "Media Deleted".to_string(),
            AuditAction::MediaUpdated => "Media Updated".to_string(),
            AuditAction::Custom(name) => name.clone(),
        }
    }

    pub fn category(&self) -> AuditCategory {
        match self {
            AuditAction::LoginSuccess | AuditAction::LoginFailed | AuditAction::Logout |
            AuditAction::PasswordChanged | AuditAction::PasswordResetRequested |
            AuditAction::PasswordResetCompleted | AuditAction::TwoFactorEnabled |
            AuditAction::TwoFactorDisabled | AuditAction::TwoFactorVerified |
            AuditAction::TwoFactorFailed | AuditAction::SessionCreated |
            AuditAction::SessionRevoked => AuditCategory::Authentication,

            AuditAction::UserCreated | AuditAction::UserUpdated | AuditAction::UserDeleted |
            AuditAction::UserActivated | AuditAction::UserDeactivated |
            AuditAction::UserLocked | AuditAction::UserUnlocked |
            AuditAction::RoleChanged | AuditAction::ProfileUpdated |
            AuditAction::EmailChanged | AuditAction::AvatarUpdated => AuditCategory::UserAccount,

            AuditAction::PostCreated | AuditAction::PostUpdated | AuditAction::PostDeleted |
            AuditAction::PostPublished | AuditAction::PostUnpublished |
            AuditAction::PageCreated | AuditAction::PageUpdated | AuditAction::PageDeleted |
            AuditAction::CommentCreated | AuditAction::CommentApproved |
            AuditAction::CommentDeleted | AuditAction::CommentSpammed => AuditCategory::Content,

            AuditAction::SettingChanged | AuditAction::OptionUpdated => AuditCategory::Settings,

            AuditAction::SuspiciousActivity | AuditAction::BruteForceAttempt |
            AuditAction::UnauthorizedAccess | AuditAction::PermissionDenied => AuditCategory::Security,

            AuditAction::SystemStarted | AuditAction::SystemStopped |
            AuditAction::CacheCleared | AuditAction::DatabaseMigrated |
            AuditAction::BackupCreated | AuditAction::BackupRestored => AuditCategory::System,

            AuditAction::PluginInstalled | AuditAction::PluginActivated |
            AuditAction::PluginDeactivated | AuditAction::PluginDeleted |
            AuditAction::PluginUpdated => AuditCategory::Plugin,

            AuditAction::ThemeActivated | AuditAction::ThemeInstalled |
            AuditAction::ThemeDeleted | AuditAction::ThemeUpdated => AuditCategory::Theme,

            AuditAction::MediaUploaded | AuditAction::MediaDeleted |
            AuditAction::MediaUpdated => AuditCategory::Media,

            AuditAction::Custom(_) => AuditCategory::System,
        }
    }

    pub fn severity(&self) -> AuditSeverity {
        match self {
            AuditAction::LoginFailed | AuditAction::TwoFactorFailed => AuditSeverity::Warning,
            AuditAction::SuspiciousActivity | AuditAction::BruteForceAttempt => AuditSeverity::Error,
            AuditAction::UnauthorizedAccess | AuditAction::PermissionDenied => AuditSeverity::Warning,
            AuditAction::UserDeleted | AuditAction::PostDeleted |
            AuditAction::PageDeleted | AuditAction::MediaDeleted => AuditSeverity::Notice,
            _ => AuditSeverity::Info,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub id: Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// User ID (if applicable)
    pub user_id: Option<i64>,

    /// Username at time of action
    pub username: Option<String>,

    /// Action performed
    pub action: AuditAction,

    /// Category
    pub category: AuditCategory,

    /// Severity
    pub severity: AuditSeverity,

    /// Resource type (e.g., "post", "user", "setting")
    pub resource_type: Option<String>,

    /// Resource ID
    pub resource_id: Option<String>,

    /// Resource name/title
    pub resource_name: Option<String>,

    /// Old values (for changes)
    pub old_values: Option<serde_json::Value>,

    /// New values (for changes)
    pub new_values: Option<serde_json::Value>,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Session ID
    pub session_id: Option<Uuid>,

    /// Additional metadata
    pub metadata: serde_json::Value,

    /// Human-readable description
    pub description: String,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(action: AuditAction) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: None,
            username: None,
            action: action.clone(),
            category: action.category(),
            severity: action.severity(),
            resource_type: None,
            resource_id: None,
            resource_name: None,
            old_values: None,
            new_values: None,
            ip_address: None,
            user_agent: None,
            session_id: None,
            metadata: serde_json::json!({}),
            description: action.name(),
        }
    }

    /// Builder methods
    pub fn user(mut self, user_id: i64, username: Option<&str>) -> Self {
        self.user_id = Some(user_id);
        self.username = username.map(|s| s.to_string());
        self
    }

    pub fn resource(mut self, resource_type: &str, resource_id: &str, name: Option<&str>) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = Some(resource_id.to_string());
        self.resource_name = name.map(|s| s.to_string());
        self
    }

    pub fn changes(mut self, old: serde_json::Value, new: serde_json::Value) -> Self {
        self.old_values = Some(old);
        self.new_values = Some(new);
        self
    }

    pub fn request_info(mut self, ip: &str, user_agent: Option<&str>) -> Self {
        self.ip_address = Some(ip.to_string());
        self.user_agent = user_agent.map(|s| s.to_string());
        self
    }

    pub fn session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Audit log query
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by user ID
    pub user_id: Option<i64>,

    /// Filter by category
    pub category: Option<AuditCategory>,

    /// Filter by severity (minimum)
    pub min_severity: Option<AuditSeverity>,

    /// Filter by resource type
    pub resource_type: Option<String>,

    /// Filter by resource ID
    pub resource_id: Option<String>,

    /// Filter by IP address
    pub ip_address: Option<String>,

    /// Search in description
    pub search: Option<String>,

    /// Start date
    pub start_date: Option<DateTime<Utc>>,

    /// End date
    pub end_date: Option<DateTime<Utc>>,

    /// Limit
    pub limit: Option<usize>,

    /// Offset
    pub offset: Option<usize>,

    /// Order by (timestamp_asc, timestamp_desc)
    pub order_by: Option<String>,
}

impl AuditQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_user(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn category(mut self, category: AuditCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn min_severity(mut self, severity: AuditSeverity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn last_days(mut self, days: i64) -> Self {
        self.start_date = Some(Utc::now() - Duration::days(days));
        self.end_date = Some(Utc::now());
        self
    }

    pub fn paginate(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// Audit log retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Days to retain info logs
    pub info_days: u32,

    /// Days to retain notice logs
    pub notice_days: u32,

    /// Days to retain warning logs
    pub warning_days: u32,

    /// Days to retain error logs
    pub error_days: u32,

    /// Days to retain critical logs
    pub critical_days: u32,

    /// Days to retain security-related logs
    pub security_days: u32,

    /// Maximum total entries (0 = unlimited)
    pub max_entries: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            info_days: 30,
            notice_days: 90,
            warning_days: 180,
            error_days: 365,
            critical_days: 730,
            security_days: 365,
            max_entries: 100000,
        }
    }
}

/// Audit manager
pub struct AuditManager {
    /// Audit entries
    entries: Vec<AuditEntry>,

    /// Retention policy
    retention_policy: RetentionPolicy,

    /// Statistics
    stats: AuditStats,
}

/// Audit statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: usize,
    pub entries_by_category: HashMap<String, usize>,
    pub entries_by_severity: HashMap<String, usize>,
    pub entries_today: usize,
    pub entries_this_week: usize,
    pub entries_this_month: usize,
}

impl Default for AuditManager {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            retention_policy: RetentionPolicy::default(),
            stats: AuditStats::default(),
        }
    }
}

impl AuditManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_retention_policy(mut self, policy: RetentionPolicy) -> Self {
        self.retention_policy = policy;
        self
    }

    /// Log an audit entry
    pub fn log(&mut self, entry: AuditEntry) {
        // Update stats
        self.stats.total_entries += 1;
        *self.stats.entries_by_category
            .entry(entry.category.name().to_string())
            .or_insert(0) += 1;
        *self.stats.entries_by_severity
            .entry(entry.severity.name().to_string())
            .or_insert(0) += 1;

        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);

        if entry.timestamp >= today_start {
            self.stats.entries_today += 1;
        }

        if entry.timestamp >= now - Duration::days(7) {
            self.stats.entries_this_week += 1;
        }

        if entry.timestamp >= now - Duration::days(30) {
            self.stats.entries_this_month += 1;
        }

        // Add entry
        self.entries.push(entry);

        // Apply retention if needed
        if self.retention_policy.max_entries > 0 && self.entries.len() > self.retention_policy.max_entries {
            self.apply_retention();
        }
    }

    /// Log a quick action
    pub fn log_action(&mut self, action: AuditAction, user_id: Option<i64>, description: &str) {
        let mut entry = AuditEntry::new(action);
        if let Some(id) = user_id {
            entry = entry.user(id, None);
        }
        entry = entry.description(description);
        self.log(entry);
    }

    /// Query audit logs
    pub fn query(&self, query: &AuditQuery) -> Vec<&AuditEntry> {
        let mut results: Vec<&AuditEntry> = self.entries.iter()
            .filter(|entry| {
                // Filter by user
                if let Some(user_id) = query.user_id {
                    if entry.user_id != Some(user_id) {
                        return false;
                    }
                }

                // Filter by category
                if let Some(category) = query.category {
                    if entry.category != category {
                        return false;
                    }
                }

                // Filter by minimum severity
                if let Some(min_severity) = query.min_severity {
                    if entry.severity < min_severity {
                        return false;
                    }
                }

                // Filter by resource type
                if let Some(ref resource_type) = query.resource_type {
                    if entry.resource_type.as_ref() != Some(resource_type) {
                        return false;
                    }
                }

                // Filter by IP address
                if let Some(ref ip) = query.ip_address {
                    if entry.ip_address.as_ref() != Some(ip) {
                        return false;
                    }
                }

                // Filter by date range
                if let Some(start) = query.start_date {
                    if entry.timestamp < start {
                        return false;
                    }
                }

                if let Some(end) = query.end_date {
                    if entry.timestamp > end {
                        return false;
                    }
                }

                // Search
                if let Some(ref search) = query.search {
                    let search_lower = search.to_lowercase();
                    if !entry.description.to_lowercase().contains(&search_lower) {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort
        match query.order_by.as_deref() {
            Some("timestamp_asc") => results.sort_by_key(|e| e.timestamp),
            _ => results.sort_by_key(|e| std::cmp::Reverse(e.timestamp)),
        }

        // Pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);

        results.into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }

    /// Get entry by ID
    pub fn get(&self, id: Uuid) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get recent entries
    pub fn recent(&self, limit: usize) -> Vec<&AuditEntry> {
        let mut entries: Vec<&AuditEntry> = self.entries.iter().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
        entries.into_iter().take(limit).collect()
    }

    /// Get entries for a specific user
    pub fn for_user(&self, user_id: i64, limit: usize) -> Vec<&AuditEntry> {
        self.query(&AuditQuery::new().for_user(user_id).paginate(limit, 0))
    }

    /// Get security events
    pub fn security_events(&self, days: i64) -> Vec<&AuditEntry> {
        self.query(&AuditQuery::new()
            .category(AuditCategory::Security)
            .last_days(days)
            .paginate(1000, 0))
    }

    /// Get statistics
    pub fn stats(&self) -> &AuditStats {
        &self.stats
    }

    /// Apply retention policy
    pub fn apply_retention(&mut self) {
        let now = Utc::now();

        self.entries.retain(|entry| {
            let days = match entry.category {
                AuditCategory::Security => self.retention_policy.security_days,
                _ => match entry.severity {
                    AuditSeverity::Info => self.retention_policy.info_days,
                    AuditSeverity::Notice => self.retention_policy.notice_days,
                    AuditSeverity::Warning => self.retention_policy.warning_days,
                    AuditSeverity::Error => self.retention_policy.error_days,
                    AuditSeverity::Critical => self.retention_policy.critical_days,
                },
            };

            let cutoff = now - Duration::days(days as i64);
            entry.timestamp > cutoff
        });

        // Trim to max entries
        if self.retention_policy.max_entries > 0 && self.entries.len() > self.retention_policy.max_entries {
            self.entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
            self.entries.truncate(self.retention_policy.max_entries);
        }

        // Recalculate stats
        self.recalculate_stats();
    }

    /// Recalculate statistics
    fn recalculate_stats(&mut self) {
        let now = Utc::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);

        self.stats = AuditStats::default();
        self.stats.total_entries = self.entries.len();

        for entry in &self.entries {
            *self.stats.entries_by_category
                .entry(entry.category.name().to_string())
                .or_insert(0) += 1;
            *self.stats.entries_by_severity
                .entry(entry.severity.name().to_string())
                .or_insert(0) += 1;

            if entry.timestamp >= today_start {
                self.stats.entries_today += 1;
            }

            if entry.timestamp >= now - Duration::days(7) {
                self.stats.entries_this_week += 1;
            }

            if entry.timestamp >= now - Duration::days(30) {
                self.stats.entries_this_month += 1;
            }
        }
    }

    /// Export logs as JSON
    pub fn export_json(&self, query: &AuditQuery) -> String {
        let entries = self.query(query);
        serde_json::to_string_pretty(&entries).unwrap_or_default()
    }

    /// Export logs as CSV
    pub fn export_csv(&self, query: &AuditQuery) -> String {
        let entries = self.query(query);
        let mut csv = String::from("id,timestamp,user_id,username,action,category,severity,resource_type,resource_id,ip_address,description\n");

        for entry in entries {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                entry.id,
                entry.timestamp.to_rfc3339(),
                entry.user_id.map(|id| id.to_string()).unwrap_or_default(),
                entry.username.as_deref().unwrap_or(""),
                entry.action.name(),
                entry.category.name(),
                entry.severity.name(),
                entry.resource_type.as_deref().unwrap_or(""),
                entry.resource_id.as_deref().unwrap_or(""),
                entry.ip_address.as_deref().unwrap_or(""),
                entry.description.replace(",", ";").replace("\n", " "),
            ));
        }

        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry() {
        let entry = AuditEntry::new(AuditAction::LoginSuccess)
            .user(1, Some("admin"))
            .request_info("192.168.1.1", Some("Mozilla/5.0"));

        assert_eq!(entry.category, AuditCategory::Authentication);
        assert_eq!(entry.severity, AuditSeverity::Info);
        assert_eq!(entry.user_id, Some(1));
    }

    #[test]
    fn test_audit_manager() {
        let mut manager = AuditManager::new();

        manager.log(AuditEntry::new(AuditAction::LoginSuccess).user(1, None));
        manager.log(AuditEntry::new(AuditAction::PostCreated).user(1, None));

        assert_eq!(manager.stats().total_entries, 2);
    }

    #[test]
    fn test_audit_query() {
        let mut manager = AuditManager::new();

        manager.log(AuditEntry::new(AuditAction::LoginSuccess).user(1, None));
        manager.log(AuditEntry::new(AuditAction::LoginSuccess).user(2, None));
        manager.log(AuditEntry::new(AuditAction::PostCreated).user(1, None));

        let results = manager.query(&AuditQuery::new().for_user(1));
        assert_eq!(results.len(), 2);

        let results = manager.query(&AuditQuery::new().category(AuditCategory::Authentication));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_audit_export() {
        let mut manager = AuditManager::new();
        manager.log(AuditEntry::new(AuditAction::LoginSuccess).user(1, Some("admin")));

        let json = manager.export_json(&AuditQuery::new());
        assert!(json.contains("LoginSuccess"));

        let csv = manager.export_csv(&AuditQuery::new());
        assert!(csv.contains("admin"));
    }
}
