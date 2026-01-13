//! # User Notification Preferences
//!
//! User notification settings and preferences management.
//!
//! Features:
//! - Email notification preferences
//! - In-app notification settings
//! - Notification channels
//! - Frequency controls
//! - Digest settings

use chrono::{DateTime, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    InApp,
    Push,
    Sms,
    Webhook,
}

impl NotificationChannel {
    pub fn label(&self) -> &str {
        match self {
            Self::Email => "Email",
            Self::InApp => "In-App",
            Self::Push => "Push Notifications",
            Self::Sms => "SMS",
            Self::Webhook => "Webhook",
        }
    }

    pub fn default_enabled() -> Vec<Self> {
        vec![Self::Email, Self::InApp]
    }
}

/// Notification type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationType {
    // Content notifications
    NewComment,
    CommentReply,
    CommentApproved,
    CommentMention,

    // Post notifications
    PostPublished,
    PostPending,
    PostUpdated,

    // User notifications
    NewFollower,
    ProfileMention,
    DirectMessage,

    // System notifications
    PasswordChanged,
    LoginFromNewDevice,
    AccountActivity,
    SecurityAlert,

    // Admin notifications
    NewUserRegistration,
    ContentFlagged,
    PluginUpdate,
    CoreUpdate,

    // Marketing/Newsletter
    Newsletter,
    ProductUpdates,
    Tips,
}

impl NotificationType {
    pub fn label(&self) -> &str {
        match self {
            Self::NewComment => "New comments on your posts",
            Self::CommentReply => "Replies to your comments",
            Self::CommentApproved => "When your comment is approved",
            Self::CommentMention => "When someone mentions you in a comment",
            Self::PostPublished => "When a post you're following is published",
            Self::PostPending => "Posts pending review",
            Self::PostUpdated => "Updates to posts you're following",
            Self::NewFollower => "When someone follows you",
            Self::ProfileMention => "When someone mentions you",
            Self::DirectMessage => "Direct messages",
            Self::PasswordChanged => "Password changes",
            Self::LoginFromNewDevice => "Login from new device",
            Self::AccountActivity => "Account activity",
            Self::SecurityAlert => "Security alerts",
            Self::NewUserRegistration => "New user registrations",
            Self::ContentFlagged => "Flagged content",
            Self::PluginUpdate => "Plugin updates available",
            Self::CoreUpdate => "Core updates available",
            Self::Newsletter => "Newsletter",
            Self::ProductUpdates => "Product updates",
            Self::Tips => "Tips and suggestions",
        }
    }

    pub fn category(&self) -> &str {
        match self {
            Self::NewComment | Self::CommentReply | Self::CommentApproved |
            Self::CommentMention => "Comments",
            Self::PostPublished | Self::PostPending | Self::PostUpdated => "Posts",
            Self::NewFollower | Self::ProfileMention | Self::DirectMessage => "Social",
            Self::PasswordChanged | Self::LoginFromNewDevice |
            Self::AccountActivity | Self::SecurityAlert => "Security",
            Self::NewUserRegistration | Self::ContentFlagged |
            Self::PluginUpdate | Self::CoreUpdate => "Admin",
            Self::Newsletter | Self::ProductUpdates | Self::Tips => "Marketing",
        }
    }

    pub fn default_enabled(&self) -> bool {
        // Security notifications are always on by default
        matches!(self,
            Self::PasswordChanged |
            Self::LoginFromNewDevice |
            Self::SecurityAlert |
            Self::DirectMessage |
            Self::CommentReply
        )
    }

    pub fn can_disable(&self) -> bool {
        // Some security notifications cannot be disabled
        !matches!(self, Self::PasswordChanged | Self::SecurityAlert)
    }
}

/// Notification frequency options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationFrequency {
    Instant,
    Hourly,
    Daily,
    Weekly,
    Never,
}

impl NotificationFrequency {
    pub fn label(&self) -> &str {
        match self {
            Self::Instant => "Immediately",
            Self::Hourly => "Hourly digest",
            Self::Daily => "Daily digest",
            Self::Weekly => "Weekly digest",
            Self::Never => "Never",
        }
    }
}

/// Individual notification preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub notification_type: NotificationType,
    pub channels: HashMap<NotificationChannel, bool>,
    pub frequency: NotificationFrequency,
}

impl NotificationPreference {
    pub fn new(notification_type: NotificationType) -> Self {
        let mut channels = HashMap::new();
        channels.insert(NotificationChannel::Email, notification_type.default_enabled());
        channels.insert(NotificationChannel::InApp, true);

        Self {
            notification_type,
            channels,
            frequency: NotificationFrequency::Instant,
        }
    }

    pub fn is_enabled(&self, channel: NotificationChannel) -> bool {
        *self.channels.get(&channel).unwrap_or(&false)
    }

    pub fn set_channel(&mut self, channel: NotificationChannel, enabled: bool) {
        if !enabled && !self.notification_type.can_disable() {
            return; // Cannot disable mandatory notifications
        }
        self.channels.insert(channel, enabled);
    }
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    pub user_id: i64,
    pub preferences: HashMap<NotificationType, NotificationPreference>,
    pub email_verified: bool,
    pub email_frequency: NotificationFrequency,
    pub digest_day: Option<Weekday>,
    pub digest_hour: u8,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: u8,
    pub quiet_hours_end: u8,
    pub timezone: String,
    pub unsubscribe_all: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserNotificationPreferences {
    pub fn new(user_id: i64) -> Self {
        let mut preferences = HashMap::new();

        // Initialize with all notification types
        for notification_type in [
            NotificationType::NewComment,
            NotificationType::CommentReply,
            NotificationType::CommentApproved,
            NotificationType::CommentMention,
            NotificationType::PostPublished,
            NotificationType::PostPending,
            NotificationType::PostUpdated,
            NotificationType::NewFollower,
            NotificationType::ProfileMention,
            NotificationType::DirectMessage,
            NotificationType::PasswordChanged,
            NotificationType::LoginFromNewDevice,
            NotificationType::AccountActivity,
            NotificationType::SecurityAlert,
            NotificationType::Newsletter,
            NotificationType::ProductUpdates,
            NotificationType::Tips,
        ] {
            preferences.insert(notification_type, NotificationPreference::new(notification_type));
        }

        Self {
            user_id,
            preferences,
            email_verified: false,
            email_frequency: NotificationFrequency::Instant,
            digest_day: Some(Weekday::Mon),
            digest_hour: 9,
            quiet_hours_enabled: false,
            quiet_hours_start: 22,
            quiet_hours_end: 8,
            timezone: "UTC".to_string(),
            unsubscribe_all: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Check if notification should be sent
    pub fn should_notify(&self, notification_type: NotificationType, channel: NotificationChannel) -> bool {
        let is_security_notification = matches!(
            notification_type,
            NotificationType::PasswordChanged | NotificationType::SecurityAlert
        );

        if self.unsubscribe_all && channel == NotificationChannel::Email {
            // Security notifications bypass unsubscribe
            if !is_security_notification {
                return false;
            }
        }

        if channel == NotificationChannel::Email && !self.email_verified {
            // Security notifications also bypass email verification
            if !is_security_notification {
                return false;
            }
        }

        self.preferences.get(&notification_type)
            .map(|p| p.is_enabled(channel))
            .unwrap_or(false)
    }

    /// Get notification frequency
    pub fn get_frequency(&self, notification_type: NotificationType) -> NotificationFrequency {
        self.preferences.get(&notification_type)
            .map(|p| p.frequency)
            .unwrap_or(NotificationFrequency::Never)
    }

    /// Update preference
    pub fn update_preference(&mut self, notification_type: NotificationType, channel: NotificationChannel, enabled: bool) {
        if let Some(pref) = self.preferences.get_mut(&notification_type) {
            pref.set_channel(channel, enabled);
            self.updated_at = Utc::now();
        }
    }

    /// Set frequency for notification type
    pub fn set_frequency(&mut self, notification_type: NotificationType, frequency: NotificationFrequency) {
        if let Some(pref) = self.preferences.get_mut(&notification_type) {
            pref.frequency = frequency;
            self.updated_at = Utc::now();
        }
    }

    /// Bulk update channel settings
    pub fn set_channel_all(&mut self, channel: NotificationChannel, enabled: bool) {
        for pref in self.preferences.values_mut() {
            if enabled || pref.notification_type.can_disable() {
                pref.channels.insert(channel, enabled);
            }
        }
        self.updated_at = Utc::now();
    }

    /// Get preferences by category
    pub fn get_by_category(&self, category: &str) -> Vec<&NotificationPreference> {
        self.preferences.values()
            .filter(|p| p.notification_type.category() == category)
            .collect()
    }

    /// Check if in quiet hours
    pub fn is_quiet_hours(&self) -> bool {
        if !self.quiet_hours_enabled {
            return false;
        }

        let now = Utc::now();
        let hour = now.hour() as u8;

        if self.quiet_hours_start < self.quiet_hours_end {
            hour >= self.quiet_hours_start && hour < self.quiet_hours_end
        } else {
            // Quiet hours span midnight
            hour >= self.quiet_hours_start || hour < self.quiet_hours_end
        }
    }
}

/// Notification preferences manager
pub struct NotificationPreferencesManager {
    preferences: HashMap<i64, UserNotificationPreferences>,
}

impl Default for NotificationPreferencesManager {
    fn default() -> Self {
        Self {
            preferences: HashMap::new(),
        }
    }
}

impl NotificationPreferencesManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create preferences for user
    pub fn get_or_create(&mut self, user_id: i64) -> &mut UserNotificationPreferences {
        self.preferences.entry(user_id)
            .or_insert_with(|| UserNotificationPreferences::new(user_id))
    }

    /// Get preferences for user
    pub fn get(&self, user_id: i64) -> Option<&UserNotificationPreferences> {
        self.preferences.get(&user_id)
    }

    /// Save preferences
    pub fn save(&mut self, prefs: UserNotificationPreferences) {
        self.preferences.insert(prefs.user_id, prefs);
    }

    /// Delete preferences
    pub fn delete(&mut self, user_id: i64) {
        self.preferences.remove(&user_id);
    }

    /// Get users who should receive notification
    pub fn get_recipients(&self, notification_type: NotificationType, channel: NotificationChannel) -> Vec<i64> {
        self.preferences.iter()
            .filter(|(_, prefs)| prefs.should_notify(notification_type, channel))
            .map(|(user_id, _)| *user_id)
            .collect()
    }

    /// Get users needing digest
    pub fn get_digest_recipients(&self, frequency: NotificationFrequency) -> Vec<i64> {
        self.preferences.iter()
            .filter(|(_, prefs)| prefs.email_frequency == frequency)
            .map(|(user_id, _)| *user_id)
            .collect()
    }
}

/// Email unsubscribe token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeToken {
    pub token: String,
    pub user_id: i64,
    pub notification_type: Option<NotificationType>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl UnsubscribeToken {
    pub fn new(user_id: i64, notification_type: Option<NotificationType>) -> Self {
        let now = Utc::now();
        Self {
            token: uuid::Uuid::new_v4().to_string(),
            user_id,
            notification_type,
            created_at: now,
            expires_at: now + chrono::Duration::days(30),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn url(&self, base_url: &str) -> String {
        format!("{}/unsubscribe?token={}", base_url, self.token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_preferences() {
        let prefs = UserNotificationPreferences::new(1);

        // Security notifications should be on by default
        assert!(prefs.should_notify(NotificationType::SecurityAlert, NotificationChannel::Email));
    }

    #[test]
    fn test_cannot_disable_security() {
        let mut prefs = UserNotificationPreferences::new(1);
        prefs.update_preference(NotificationType::SecurityAlert, NotificationChannel::Email, false);

        // Should still be enabled
        assert!(prefs.should_notify(NotificationType::SecurityAlert, NotificationChannel::Email));
    }

    #[test]
    fn test_unsubscribe_all() {
        let mut prefs = UserNotificationPreferences::new(1);
        prefs.email_verified = true;
        prefs.unsubscribe_all = true;

        // Regular notifications should be blocked
        assert!(!prefs.should_notify(NotificationType::Newsletter, NotificationChannel::Email));

        // Security notifications should still work
        assert!(prefs.should_notify(NotificationType::SecurityAlert, NotificationChannel::Email));
    }

    #[test]
    fn test_quiet_hours() {
        let mut prefs = UserNotificationPreferences::new(1);
        prefs.quiet_hours_enabled = true;
        prefs.quiet_hours_start = 0;
        prefs.quiet_hours_end = 24;

        // Should always be in quiet hours
        assert!(prefs.is_quiet_hours());
    }

    #[test]
    fn test_preferences_manager() {
        let mut manager = NotificationPreferencesManager::new();
        let prefs = manager.get_or_create(1);
        prefs.email_verified = true;

        assert!(manager.get(1).is_some());
    }
}
