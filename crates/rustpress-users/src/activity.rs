//! # User Activity Tracking
//!
//! Track and log user activities for auditing and analytics.
//!
//! Features:
//! - Activity logging
//! - Activity types and categories
//! - Activity streams
//! - User session tracking
//! - Login history

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Activity type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityCategory {
    Authentication,
    Content,
    Comment,
    User,
    Media,
    Settings,
    Plugin,
    Theme,
    System,
    Custom,
}

impl ActivityCategory {
    pub fn label(&self) -> &str {
        match self {
            Self::Authentication => "Authentication",
            Self::Content => "Content",
            Self::Comment => "Comments",
            Self::User => "Users",
            Self::Media => "Media",
            Self::Settings => "Settings",
            Self::Plugin => "Plugins",
            Self::Theme => "Themes",
            Self::System => "System",
            Self::Custom => "Custom",
        }
    }
}

/// Specific activity types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    // Authentication
    Login,
    Logout,
    LoginFailed,
    PasswordReset,
    PasswordChanged,
    TwoFactorEnabled,
    TwoFactorDisabled,
    SessionExpired,

    // Content
    PostCreated,
    PostUpdated,
    PostDeleted,
    PostPublished,
    PostTrashed,
    PostRestored,
    PageCreated,
    PageUpdated,
    PageDeleted,

    // Comments
    CommentCreated,
    CommentApproved,
    CommentSpammed,
    CommentTrashed,
    CommentDeleted,

    // Users
    UserCreated,
    UserUpdated,
    UserDeleted,
    ProfileUpdated,
    RoleChanged,
    UserActivated,
    UserDeactivated,

    // Media
    MediaUploaded,
    MediaUpdated,
    MediaDeleted,

    // Settings
    OptionUpdated,
    SettingsSaved,

    // Plugins
    PluginActivated,
    PluginDeactivated,
    PluginInstalled,
    PluginDeleted,
    PluginUpdated,

    // Themes
    ThemeSwitched,
    ThemeCustomized,
    ThemeInstalled,
    ThemeDeleted,

    // System
    CoreUpdated,
    ExportCreated,
    ImportCompleted,

    // Custom
    Custom(String),
}

impl ActivityType {
    pub fn category(&self) -> ActivityCategory {
        match self {
            Self::Login | Self::Logout | Self::LoginFailed |
            Self::PasswordReset | Self::PasswordChanged |
            Self::TwoFactorEnabled | Self::TwoFactorDisabled |
            Self::SessionExpired => ActivityCategory::Authentication,

            Self::PostCreated | Self::PostUpdated | Self::PostDeleted |
            Self::PostPublished | Self::PostTrashed | Self::PostRestored |
            Self::PageCreated | Self::PageUpdated | Self::PageDeleted => ActivityCategory::Content,

            Self::CommentCreated | Self::CommentApproved |
            Self::CommentSpammed | Self::CommentTrashed |
            Self::CommentDeleted => ActivityCategory::Comment,

            Self::UserCreated | Self::UserUpdated | Self::UserDeleted |
            Self::ProfileUpdated | Self::RoleChanged |
            Self::UserActivated | Self::UserDeactivated => ActivityCategory::User,

            Self::MediaUploaded | Self::MediaUpdated |
            Self::MediaDeleted => ActivityCategory::Media,

            Self::OptionUpdated | Self::SettingsSaved => ActivityCategory::Settings,

            Self::PluginActivated | Self::PluginDeactivated |
            Self::PluginInstalled | Self::PluginDeleted |
            Self::PluginUpdated => ActivityCategory::Plugin,

            Self::ThemeSwitched | Self::ThemeCustomized |
            Self::ThemeInstalled | Self::ThemeDeleted => ActivityCategory::Theme,

            Self::CoreUpdated | Self::ExportCreated |
            Self::ImportCompleted => ActivityCategory::System,

            Self::Custom(_) => ActivityCategory::Custom,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Login => "Logged in".to_string(),
            Self::Logout => "Logged out".to_string(),
            Self::LoginFailed => "Failed login attempt".to_string(),
            Self::PasswordReset => "Password reset requested".to_string(),
            Self::PasswordChanged => "Password changed".to_string(),
            Self::TwoFactorEnabled => "Two-factor authentication enabled".to_string(),
            Self::TwoFactorDisabled => "Two-factor authentication disabled".to_string(),
            Self::SessionExpired => "Session expired".to_string(),
            Self::PostCreated => "Created post".to_string(),
            Self::PostUpdated => "Updated post".to_string(),
            Self::PostDeleted => "Deleted post".to_string(),
            Self::PostPublished => "Published post".to_string(),
            Self::PostTrashed => "Trashed post".to_string(),
            Self::PostRestored => "Restored post".to_string(),
            Self::PageCreated => "Created page".to_string(),
            Self::PageUpdated => "Updated page".to_string(),
            Self::PageDeleted => "Deleted page".to_string(),
            Self::CommentCreated => "Created comment".to_string(),
            Self::CommentApproved => "Approved comment".to_string(),
            Self::CommentSpammed => "Marked comment as spam".to_string(),
            Self::CommentTrashed => "Trashed comment".to_string(),
            Self::CommentDeleted => "Deleted comment".to_string(),
            Self::UserCreated => "Created user".to_string(),
            Self::UserUpdated => "Updated user".to_string(),
            Self::UserDeleted => "Deleted user".to_string(),
            Self::ProfileUpdated => "Updated profile".to_string(),
            Self::RoleChanged => "Changed role".to_string(),
            Self::UserActivated => "Activated user".to_string(),
            Self::UserDeactivated => "Deactivated user".to_string(),
            Self::MediaUploaded => "Uploaded media".to_string(),
            Self::MediaUpdated => "Updated media".to_string(),
            Self::MediaDeleted => "Deleted media".to_string(),
            Self::OptionUpdated => "Updated option".to_string(),
            Self::SettingsSaved => "Saved settings".to_string(),
            Self::PluginActivated => "Activated plugin".to_string(),
            Self::PluginDeactivated => "Deactivated plugin".to_string(),
            Self::PluginInstalled => "Installed plugin".to_string(),
            Self::PluginDeleted => "Deleted plugin".to_string(),
            Self::PluginUpdated => "Updated plugin".to_string(),
            Self::ThemeSwitched => "Switched theme".to_string(),
            Self::ThemeCustomized => "Customized theme".to_string(),
            Self::ThemeInstalled => "Installed theme".to_string(),
            Self::ThemeDeleted => "Deleted theme".to_string(),
            Self::CoreUpdated => "Updated core".to_string(),
            Self::ExportCreated => "Created export".to_string(),
            Self::ImportCompleted => "Completed import".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }

    pub fn icon(&self) -> &str {
        match self.category() {
            ActivityCategory::Authentication => "key",
            ActivityCategory::Content => "file-text",
            ActivityCategory::Comment => "message-square",
            ActivityCategory::User => "user",
            ActivityCategory::Media => "image",
            ActivityCategory::Settings => "settings",
            ActivityCategory::Plugin => "puzzle",
            ActivityCategory::Theme => "palette",
            ActivityCategory::System => "server",
            ActivityCategory::Custom => "activity",
        }
    }
}

/// Activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Unique ID
    pub id: Uuid,

    /// Activity type
    pub activity_type: ActivityType,

    /// User who performed the action
    pub user_id: Option<i64>,

    /// Username for display
    pub username: Option<String>,

    /// Object type (post, user, comment, etc.)
    pub object_type: Option<String>,

    /// Object ID
    pub object_id: Option<i64>,

    /// Object name for display
    pub object_name: Option<String>,

    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// When the activity occurred
    pub created_at: DateTime<Utc>,

    /// Site ID (for multisite)
    pub site_id: Option<i64>,
}

impl Activity {
    pub fn new(activity_type: ActivityType) -> Self {
        Self {
            id: Uuid::new_v4(),
            activity_type,
            user_id: None,
            username: None,
            object_type: None,
            object_id: None,
            object_name: None,
            context: HashMap::new(),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
            site_id: None,
        }
    }

    pub fn by_user(mut self, user_id: i64, username: &str) -> Self {
        self.user_id = Some(user_id);
        self.username = Some(username.to_string());
        self
    }

    pub fn on_object(mut self, object_type: &str, object_id: i64, object_name: &str) -> Self {
        self.object_type = Some(object_type.to_string());
        self.object_id = Some(object_id);
        self.object_name = Some(object_name.to_string());
        self
    }

    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }

    pub fn from_request(mut self, ip: &str, user_agent: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self.user_agent = Some(user_agent.to_string());
        self
    }

    /// Generate human-readable message
    pub fn message(&self) -> String {
        let actor = self.username.as_deref().unwrap_or("Someone");
        let action = self.activity_type.label();

        if let Some(ref name) = self.object_name {
            format!("{} {} \"{}\"", actor, action.to_lowercase(), name)
        } else {
            format!("{} {}", actor, action.to_lowercase())
        }
    }
}

/// Activity query parameters
#[derive(Debug, Clone, Default)]
pub struct ActivityQuery {
    pub user_id: Option<i64>,
    pub object_type: Option<String>,
    pub object_id: Option<i64>,
    pub activity_type: Option<ActivityType>,
    pub category: Option<ActivityCategory>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub limit: usize,
    pub offset: usize,
}

impl ActivityQuery {
    pub fn new() -> Self {
        Self {
            limit: 50,
            ..Default::default()
        }
    }

    pub fn for_user(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn for_object(mut self, object_type: &str, object_id: i64) -> Self {
        self.object_type = Some(object_type.to_string());
        self.object_id = Some(object_id);
        self
    }

    pub fn of_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = Some(activity_type);
        self
    }

    pub fn in_category(mut self, category: ActivityCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn since(mut self, date: DateTime<Utc>) -> Self {
        self.after = Some(date);
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Activity manager
pub struct ActivityManager {
    activities: Vec<Activity>,
    retention_days: u32,
    enabled_categories: Vec<ActivityCategory>,
}

impl Default for ActivityManager {
    fn default() -> Self {
        Self {
            activities: Vec::new(),
            retention_days: 90,
            enabled_categories: vec![
                ActivityCategory::Authentication,
                ActivityCategory::Content,
                ActivityCategory::User,
                ActivityCategory::Settings,
                ActivityCategory::System,
            ],
        }
    }
}

impl ActivityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_retention(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }

    pub fn enable_category(&mut self, category: ActivityCategory) {
        if !self.enabled_categories.contains(&category) {
            self.enabled_categories.push(category);
        }
    }

    pub fn disable_category(&mut self, category: ActivityCategory) {
        self.enabled_categories.retain(|c| c != &category);
    }

    /// Log an activity
    pub fn log(&mut self, activity: Activity) {
        let category = activity.activity_type.category();
        if self.enabled_categories.contains(&category) {
            self.activities.push(activity);
        }
    }

    /// Query activities
    pub fn query(&self, query: &ActivityQuery) -> Vec<&Activity> {
        self.activities.iter()
            .filter(|a| {
                if let Some(uid) = query.user_id {
                    if a.user_id != Some(uid) {
                        return false;
                    }
                }
                if let Some(ref ot) = query.object_type {
                    if a.object_type.as_ref() != Some(ot) {
                        return false;
                    }
                }
                if let Some(oid) = query.object_id {
                    if a.object_id != Some(oid) {
                        return false;
                    }
                }
                if let Some(ref at) = query.activity_type {
                    if &a.activity_type != at {
                        return false;
                    }
                }
                if let Some(cat) = query.category {
                    if a.activity_type.category() != cat {
                        return false;
                    }
                }
                if let Some(after) = query.after {
                    if a.created_at < after {
                        return false;
                    }
                }
                if let Some(before) = query.before {
                    if a.created_at > before {
                        return false;
                    }
                }
                true
            })
            .skip(query.offset)
            .take(query.limit)
            .collect()
    }

    /// Get recent activities
    pub fn recent(&self, limit: usize) -> Vec<&Activity> {
        self.activities.iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Get activities for user
    pub fn for_user(&self, user_id: i64, limit: usize) -> Vec<&Activity> {
        self.activities.iter()
            .rev()
            .filter(|a| a.user_id == Some(user_id))
            .take(limit)
            .collect()
    }

    /// Get activity count by category
    pub fn count_by_category(&self) -> HashMap<ActivityCategory, usize> {
        let mut counts = HashMap::new();
        for activity in &self.activities {
            let category = activity.activity_type.category();
            *counts.entry(category).or_insert(0) += 1;
        }
        counts
    }

    /// Cleanup old activities
    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        self.activities.retain(|a| a.created_at > cutoff);
    }
}

// ============================================================================
// Login History
// ============================================================================

/// Login history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRecord {
    pub id: Uuid,
    pub user_id: i64,
    pub ip_address: String,
    pub user_agent: String,
    pub location: Option<GeoLocation>,
    pub device_info: DeviceInfo,
    pub login_at: DateTime<Utc>,
    pub logout_at: Option<DateTime<Utc>>,
    pub session_id: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
}

/// Geolocation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub browser: Option<String>,
    pub browser_version: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

impl DeviceInfo {
    pub fn from_user_agent(user_agent: &str) -> Self {
        let ua = user_agent.to_lowercase();

        let device_type = if ua.contains("mobile") || ua.contains("android") && !ua.contains("tablet") {
            DeviceType::Mobile
        } else if ua.contains("tablet") || ua.contains("ipad") {
            DeviceType::Tablet
        } else if ua.contains("windows") || ua.contains("macintosh") || ua.contains("linux") {
            DeviceType::Desktop
        } else {
            DeviceType::Unknown
        };

        // Simple browser detection
        let (browser, browser_version) = if ua.contains("firefox") {
            (Some("Firefox".to_string()), None)
        } else if ua.contains("chrome") && !ua.contains("edg") {
            (Some("Chrome".to_string()), None)
        } else if ua.contains("safari") && !ua.contains("chrome") {
            (Some("Safari".to_string()), None)
        } else if ua.contains("edg") {
            (Some("Edge".to_string()), None)
        } else {
            (None, None)
        };

        // Simple OS detection
        let (os, os_version) = if ua.contains("windows") {
            (Some("Windows".to_string()), None)
        } else if ua.contains("macintosh") || ua.contains("mac os") {
            (Some("macOS".to_string()), None)
        } else if ua.contains("linux") {
            (Some("Linux".to_string()), None)
        } else if ua.contains("android") {
            (Some("Android".to_string()), None)
        } else if ua.contains("iphone") || ua.contains("ipad") {
            (Some("iOS".to_string()), None)
        } else {
            (None, None)
        };

        Self {
            device_type,
            browser,
            browser_version,
            os,
            os_version,
        }
    }
}

impl LoginRecord {
    pub fn new(user_id: i64, ip: &str, user_agent: &str, success: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            ip_address: ip.to_string(),
            user_agent: user_agent.to_string(),
            location: None,
            device_info: DeviceInfo::from_user_agent(user_agent),
            login_at: Utc::now(),
            logout_at: None,
            session_id: None,
            success,
            failure_reason: None,
        }
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_failure_reason(mut self, reason: &str) -> Self {
        self.failure_reason = Some(reason.to_string());
        self
    }

    pub fn with_location(mut self, location: GeoLocation) -> Self {
        self.location = Some(location);
        self
    }
}

/// Login history manager
pub struct LoginHistory {
    records: HashMap<i64, Vec<LoginRecord>>,
    max_records_per_user: usize,
}

impl Default for LoginHistory {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            max_records_per_user: 100,
        }
    }
}

impl LoginHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a login
    pub fn record_login(&mut self, record: LoginRecord) {
        let records = self.records.entry(record.user_id).or_insert_with(Vec::new);
        records.push(record);

        // Trim old records
        if records.len() > self.max_records_per_user {
            let excess = records.len() - self.max_records_per_user;
            records.drain(0..excess);
        }
    }

    /// Record logout
    pub fn record_logout(&mut self, user_id: i64, session_id: Option<&str>) {
        if let Some(records) = self.records.get_mut(&user_id) {
            for record in records.iter_mut().rev() {
                if record.logout_at.is_none() {
                    if let Some(sid) = session_id {
                        if record.session_id.as_deref() == Some(sid) {
                            record.logout_at = Some(Utc::now());
                            break;
                        }
                    } else {
                        record.logout_at = Some(Utc::now());
                        break;
                    }
                }
            }
        }
    }

    /// Get login history for user
    pub fn get_history(&self, user_id: i64) -> Vec<&LoginRecord> {
        self.records.get(&user_id)
            .map(|records| records.iter().rev().collect())
            .unwrap_or_default()
    }

    /// Get recent logins
    pub fn get_recent(&self, user_id: i64, limit: usize) -> Vec<&LoginRecord> {
        self.records.get(&user_id)
            .map(|records| records.iter().rev().take(limit).collect())
            .unwrap_or_default()
    }

    /// Get failed login attempts
    pub fn get_failed_attempts(&self, user_id: i64, since: DateTime<Utc>) -> Vec<&LoginRecord> {
        self.records.get(&user_id)
            .map(|records| {
                records.iter()
                    .filter(|r| !r.success && r.login_at > since)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check for suspicious activity
    pub fn check_suspicious(&self, user_id: i64, current_ip: &str) -> Vec<SuspiciousActivity> {
        let mut alerts = Vec::new();

        if let Some(records) = self.records.get(&user_id) {
            let recent: Vec<_> = records.iter()
                .rev()
                .filter(|r| r.success)
                .take(10)
                .collect();

            // Check for login from new IP
            let known_ips: Vec<_> = recent.iter()
                .map(|r| r.ip_address.as_str())
                .collect();

            if !known_ips.contains(&current_ip) && !recent.is_empty() {
                alerts.push(SuspiciousActivity::NewIpAddress {
                    ip: current_ip.to_string(),
                });
            }

            // Check for login from new device type
            if let Some(last) = recent.first() {
                let current_device = DeviceInfo::from_user_agent("");
                if last.device_info.device_type != current_device.device_type {
                    alerts.push(SuspiciousActivity::NewDeviceType);
                }
            }

            // Check for multiple failed attempts
            let hour_ago = Utc::now() - Duration::hours(1);
            let failed_count = records.iter()
                .filter(|r| !r.success && r.login_at > hour_ago)
                .count();

            if failed_count >= 5 {
                alerts.push(SuspiciousActivity::MultipleFailedAttempts {
                    count: failed_count,
                });
            }
        }

        alerts
    }
}

/// Suspicious activity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspiciousActivity {
    NewIpAddress { ip: String },
    NewDeviceType,
    NewLocation { location: String },
    MultipleFailedAttempts { count: usize },
    RapidLocationChange,
}

impl SuspiciousActivity {
    pub fn message(&self) -> String {
        match self {
            Self::NewIpAddress { ip } => {
                format!("Login from new IP address: {}", ip)
            }
            Self::NewDeviceType => {
                "Login from a new device type".to_string()
            }
            Self::NewLocation { location } => {
                format!("Login from new location: {}", location)
            }
            Self::MultipleFailedAttempts { count } => {
                format!("{} failed login attempts in the last hour", count)
            }
            Self::RapidLocationChange => {
                "Login from geographically distant location in short time".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_creation() {
        let activity = Activity::new(ActivityType::PostCreated)
            .by_user(1, "admin")
            .on_object("post", 123, "Hello World");

        assert_eq!(activity.user_id, Some(1));
        assert_eq!(activity.object_id, Some(123));
    }

    #[test]
    fn test_activity_message() {
        let activity = Activity::new(ActivityType::PostPublished)
            .by_user(1, "admin")
            .on_object("post", 1, "My First Post");

        assert_eq!(activity.message(), "admin published post \"My First Post\"");
    }

    #[test]
    fn test_activity_manager() {
        let mut manager = ActivityManager::new();

        manager.log(Activity::new(ActivityType::Login).by_user(1, "admin"));
        manager.log(Activity::new(ActivityType::PostCreated).by_user(1, "admin"));

        let recent = manager.recent(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_activity_query() {
        let mut manager = ActivityManager::new();

        manager.log(Activity::new(ActivityType::Login).by_user(1, "admin"));
        manager.log(Activity::new(ActivityType::Login).by_user(2, "editor"));
        manager.log(Activity::new(ActivityType::PostCreated).by_user(1, "admin"));

        let query = ActivityQuery::new()
            .for_user(1)
            .in_category(ActivityCategory::Authentication);

        let results = manager.query(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_login_record() {
        let record = LoginRecord::new(1, "192.168.1.1", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0", true);

        assert_eq!(record.device_info.device_type, DeviceType::Desktop);
        assert_eq!(record.device_info.browser, Some("Chrome".to_string()));
        assert_eq!(record.device_info.os, Some("Windows".to_string()));
    }

    #[test]
    fn test_login_history() {
        let mut history = LoginHistory::new();

        history.record_login(LoginRecord::new(1, "192.168.1.1", "Test Agent", true));
        history.record_login(LoginRecord::new(1, "192.168.1.2", "Test Agent", false));

        let records = history.get_history(1);
        assert_eq!(records.len(), 2);

        let failed = history.get_failed_attempts(1, Utc::now() - Duration::hours(1));
        assert_eq!(failed.len(), 1);
    }
}
