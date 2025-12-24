//! # User Dashboard
//!
//! Personalized user dashboard with widgets and stats.
//!
//! Features:
//! - Dashboard widgets
//! - Personalized content
//! - Quick stats
//! - Recent activity
//! - Widget customization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dashboard widget types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WidgetType {
    // Content widgets
    QuickDraft,
    RecentPosts,
    RecentComments,
    RecentActivity,

    // Stats widgets
    AtAGlance,
    SiteStats,
    UserStats,
    ContentStats,

    // External widgets
    NewsFeed,
    Welcome,

    // Social widgets
    Followers,
    Following,
    Notifications,

    // Admin widgets
    PendingReview,
    ScheduledPosts,
    DraftPosts,

    // Custom widget
    Custom(String),
}

impl WidgetType {
    pub fn label(&self) -> &str {
        match self {
            Self::QuickDraft => "Quick Draft",
            Self::RecentPosts => "Recent Posts",
            Self::RecentComments => "Recent Comments",
            Self::RecentActivity => "Recent Activity",
            Self::AtAGlance => "At a Glance",
            Self::SiteStats => "Site Stats",
            Self::UserStats => "Your Stats",
            Self::ContentStats => "Content Stats",
            Self::NewsFeed => "News Feed",
            Self::Welcome => "Welcome",
            Self::Followers => "Followers",
            Self::Following => "Following",
            Self::Notifications => "Notifications",
            Self::PendingReview => "Pending Review",
            Self::ScheduledPosts => "Scheduled Posts",
            Self::DraftPosts => "Your Drafts",
            Self::Custom(name) => name,
        }
    }

    pub fn default_column(&self) -> DashboardColumn {
        match self {
            Self::AtAGlance | Self::QuickDraft | Self::RecentActivity => DashboardColumn::Normal,
            Self::SiteStats | Self::UserStats | Self::ContentStats => DashboardColumn::Side,
            _ => DashboardColumn::Normal,
        }
    }
}

/// Dashboard column positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardColumn {
    Normal,
    Side,
    FullWidth,
}

/// Dashboard widget instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub column: DashboardColumn,
    pub order: i32,
    pub visible: bool,
    pub minimized: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

impl DashboardWidget {
    pub fn new(widget_type: WidgetType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: widget_type.label().to_string(),
            column: widget_type.default_column(),
            widget_type,
            order: 0,
            visible: true,
            minimized: false,
            settings: HashMap::new(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn in_column(mut self, column: DashboardColumn) -> Self {
        self.column = column;
        self
    }

    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn setting(mut self, key: &str, value: serde_json::Value) -> Self {
        self.settings.insert(key.to_string(), value);
        self
    }
}

/// User dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub user_id: i64,
    pub widgets: Vec<DashboardWidget>,
    pub columns: u8,
    pub updated_at: DateTime<Utc>,
}

impl DashboardLayout {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            widgets: Vec::new(),
            columns: 2,
            updated_at: Utc::now(),
        }
    }

    /// Create default admin dashboard
    pub fn default_admin(user_id: i64) -> Self {
        let mut layout = Self::new(user_id);

        layout.add_widget(DashboardWidget::new(WidgetType::AtAGlance).order(1));
        layout.add_widget(DashboardWidget::new(WidgetType::RecentActivity).order(2));
        layout.add_widget(DashboardWidget::new(WidgetType::QuickDraft).order(3));
        layout.add_widget(DashboardWidget::new(WidgetType::PendingReview).order(4));
        layout.add_widget(DashboardWidget::new(WidgetType::RecentComments).in_column(DashboardColumn::Side).order(1));
        layout.add_widget(DashboardWidget::new(WidgetType::ScheduledPosts).in_column(DashboardColumn::Side).order(2));

        layout
    }

    /// Create default user dashboard
    pub fn default_user(user_id: i64) -> Self {
        let mut layout = Self::new(user_id);

        layout.add_widget(DashboardWidget::new(WidgetType::Welcome).order(1));
        layout.add_widget(DashboardWidget::new(WidgetType::UserStats).order(2));
        layout.add_widget(DashboardWidget::new(WidgetType::RecentPosts).order(3));
        layout.add_widget(DashboardWidget::new(WidgetType::Notifications).in_column(DashboardColumn::Side).order(1));
        layout.add_widget(DashboardWidget::new(WidgetType::DraftPosts).in_column(DashboardColumn::Side).order(2));

        layout
    }

    pub fn add_widget(&mut self, widget: DashboardWidget) {
        self.widgets.push(widget);
        self.updated_at = Utc::now();
    }

    pub fn remove_widget(&mut self, widget_id: &str) {
        self.widgets.retain(|w| w.id != widget_id);
        self.updated_at = Utc::now();
    }

    pub fn get_widget(&self, widget_id: &str) -> Option<&DashboardWidget> {
        self.widgets.iter().find(|w| w.id == widget_id)
    }

    pub fn get_widget_mut(&mut self, widget_id: &str) -> Option<&mut DashboardWidget> {
        self.widgets.iter_mut().find(|w| w.id == widget_id)
    }

    pub fn toggle_visibility(&mut self, widget_id: &str) {
        if let Some(widget) = self.get_widget_mut(widget_id) {
            widget.visible = !widget.visible;
            self.updated_at = Utc::now();
        }
    }

    pub fn toggle_minimized(&mut self, widget_id: &str) {
        if let Some(widget) = self.get_widget_mut(widget_id) {
            widget.minimized = !widget.minimized;
            self.updated_at = Utc::now();
        }
    }

    pub fn reorder(&mut self, widget_id: &str, new_order: i32, new_column: Option<DashboardColumn>) {
        if let Some(widget) = self.get_widget_mut(widget_id) {
            widget.order = new_order;
            if let Some(col) = new_column {
                widget.column = col;
            }
            self.updated_at = Utc::now();
        }
    }

    pub fn get_column_widgets(&self, column: DashboardColumn) -> Vec<&DashboardWidget> {
        let mut widgets: Vec<_> = self.widgets.iter()
            .filter(|w| w.column == column && w.visible)
            .collect();
        widgets.sort_by_key(|w| w.order);
        widgets
    }
}

/// Quick stats for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub posts_count: u32,
    pub pages_count: u32,
    pub comments_count: u32,
    pub pending_comments: u32,
    pub users_count: u32,
    pub categories_count: u32,
    pub tags_count: u32,
    pub media_count: u32,
    pub draft_posts: u32,
    pub scheduled_posts: u32,
}

impl Default for DashboardStats {
    fn default() -> Self {
        Self {
            posts_count: 0,
            pages_count: 0,
            comments_count: 0,
            pending_comments: 0,
            users_count: 0,
            categories_count: 0,
            tags_count: 0,
            media_count: 0,
            draft_posts: 0,
            scheduled_posts: 0,
        }
    }
}

/// User-specific dashboard stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboardStats {
    pub user_id: i64,
    pub posts_published: u32,
    pub posts_drafted: u32,
    pub comments_received: u32,
    pub comments_made: u32,
    pub followers_count: u32,
    pub following_count: u32,
    pub views_this_month: u64,
    pub views_total: u64,
    pub reputation_score: i32,
    pub badges_count: u32,
}

impl UserDashboardStats {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            posts_published: 0,
            posts_drafted: 0,
            comments_received: 0,
            comments_made: 0,
            followers_count: 0,
            following_count: 0,
            views_this_month: 0,
            views_total: 0,
            reputation_score: 0,
            badges_count: 0,
        }
    }
}

/// Dashboard recent item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardItem {
    pub id: i64,
    pub title: String,
    pub item_type: String,
    pub status: String,
    pub author_id: i64,
    pub author_name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub url: Option<String>,
}

/// Quick draft data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickDraft {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

impl QuickDraft {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            tags: Vec::new(),
        }
    }
}

impl Default for QuickDraft {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard notification item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationLevel,
    pub action_url: Option<String>,
    pub action_label: Option<String>,
    pub dismissible: bool,
    pub created_at: DateTime<Utc>,
    pub read: bool,
}

/// Notification level/type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    pub fn color(&self) -> &str {
        match self {
            Self::Info => "#0073aa",
            Self::Success => "#46b450",
            Self::Warning => "#ffb900",
            Self::Error => "#dc3232",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Success => "check-circle",
            Self::Warning => "alert-triangle",
            Self::Error => "alert-circle",
        }
    }
}

impl DashboardNotification {
    pub fn info(title: &str, message: &str) -> Self {
        Self::new(title, message, NotificationLevel::Info)
    }

    pub fn success(title: &str, message: &str) -> Self {
        Self::new(title, message, NotificationLevel::Success)
    }

    pub fn warning(title: &str, message: &str) -> Self {
        Self::new(title, message, NotificationLevel::Warning)
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self::new(title, message, NotificationLevel::Error)
    }

    fn new(title: &str, message: &str, notification_type: NotificationLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            action_url: None,
            action_label: None,
            dismissible: true,
            created_at: Utc::now(),
            read: false,
        }
    }

    pub fn with_action(mut self, url: &str, label: &str) -> Self {
        self.action_url = Some(url.to_string());
        self.action_label = Some(label.to_string());
        self
    }

    pub fn not_dismissible(mut self) -> Self {
        self.dismissible = false;
        self
    }
}

/// Dashboard manager
pub struct DashboardManager {
    layouts: HashMap<i64, DashboardLayout>,
    available_widgets: Vec<WidgetType>,
    global_notifications: Vec<DashboardNotification>,
    user_notifications: HashMap<i64, Vec<DashboardNotification>>,
}

impl Default for DashboardManager {
    fn default() -> Self {
        Self {
            layouts: HashMap::new(),
            available_widgets: vec![
                WidgetType::AtAGlance,
                WidgetType::QuickDraft,
                WidgetType::RecentPosts,
                WidgetType::RecentComments,
                WidgetType::RecentActivity,
                WidgetType::SiteStats,
                WidgetType::UserStats,
                WidgetType::NewsFeed,
                WidgetType::Notifications,
                WidgetType::PendingReview,
                WidgetType::ScheduledPosts,
                WidgetType::DraftPosts,
                WidgetType::Followers,
                WidgetType::Following,
            ],
            global_notifications: Vec::new(),
            user_notifications: HashMap::new(),
        }
    }
}

impl DashboardManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create layout for user
    pub fn get_layout(&mut self, user_id: i64, is_admin: bool) -> &DashboardLayout {
        self.layouts.entry(user_id).or_insert_with(|| {
            if is_admin {
                DashboardLayout::default_admin(user_id)
            } else {
                DashboardLayout::default_user(user_id)
            }
        })
    }

    /// Save layout
    pub fn save_layout(&mut self, layout: DashboardLayout) {
        self.layouts.insert(layout.user_id, layout);
    }

    /// Get available widgets
    pub fn get_available_widgets(&self) -> &[WidgetType] {
        &self.available_widgets
    }

    /// Register custom widget
    pub fn register_widget(&mut self, widget_type: WidgetType) {
        if !self.available_widgets.contains(&widget_type) {
            self.available_widgets.push(widget_type);
        }
    }

    /// Add global notification
    pub fn add_global_notification(&mut self, notification: DashboardNotification) {
        self.global_notifications.push(notification);
    }

    /// Add user notification
    pub fn add_user_notification(&mut self, user_id: i64, notification: DashboardNotification) {
        self.user_notifications.entry(user_id)
            .or_insert_with(Vec::new)
            .push(notification);
    }

    /// Get notifications for user
    pub fn get_notifications(&self, user_id: i64) -> Vec<&DashboardNotification> {
        let mut notifications: Vec<_> = self.global_notifications.iter().collect();

        if let Some(user_notifs) = self.user_notifications.get(&user_id) {
            notifications.extend(user_notifs.iter());
        }

        notifications.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        notifications
    }

    /// Dismiss notification
    pub fn dismiss_notification(&mut self, user_id: i64, notification_id: &str) {
        // Mark as read in user notifications
        if let Some(notifications) = self.user_notifications.get_mut(&user_id) {
            if let Some(notif) = notifications.iter_mut().find(|n| n.id == notification_id) {
                if notif.dismissible {
                    notif.read = true;
                }
            }
        }
    }

    /// Clear read notifications
    pub fn clear_read_notifications(&mut self, user_id: i64) {
        if let Some(notifications) = self.user_notifications.get_mut(&user_id) {
            notifications.retain(|n| !n.read);
        }
    }

    /// Get unread count
    pub fn get_unread_count(&self, user_id: i64) -> usize {
        let global_unread = self.global_notifications.iter()
            .filter(|n| !n.read)
            .count();

        let user_unread = self.user_notifications.get(&user_id)
            .map(|notifications| notifications.iter().filter(|n| !n.read).count())
            .unwrap_or(0);

        global_unread + user_unread
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_layout() {
        let layout = DashboardLayout::default_admin(1);
        assert!(!layout.widgets.is_empty());
    }

    #[test]
    fn test_widget_columns() {
        let layout = DashboardLayout::default_admin(1);
        let normal_widgets = layout.get_column_widgets(DashboardColumn::Normal);
        let side_widgets = layout.get_column_widgets(DashboardColumn::Side);

        assert!(!normal_widgets.is_empty());
        assert!(!side_widgets.is_empty());
    }

    #[test]
    fn test_widget_visibility() {
        let mut layout = DashboardLayout::default_admin(1);
        let widget_id = layout.widgets[0].id.clone();

        layout.toggle_visibility(&widget_id);
        assert!(!layout.get_widget(&widget_id).unwrap().visible);
    }

    #[test]
    fn test_notifications() {
        let mut manager = DashboardManager::new();

        manager.add_global_notification(
            DashboardNotification::info("Test", "Test message")
        );

        manager.add_user_notification(1,
            DashboardNotification::warning("User", "User message")
        );

        let notifications = manager.get_notifications(1);
        assert_eq!(notifications.len(), 2);
    }

    #[test]
    fn test_unread_count() {
        let mut manager = DashboardManager::new();

        manager.add_user_notification(1, DashboardNotification::info("A", "B"));
        manager.add_user_notification(1, DashboardNotification::info("C", "D"));

        assert_eq!(manager.get_unread_count(1), 2);
    }
}
