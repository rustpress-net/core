//! Functions System - Event-driven serverless functions for RustPress
//!
//! This module provides a comprehensive event-driven function system similar to
//! AWS Lambda or WordPress hooks, allowing users to create custom automation
//! triggered by various events in RustPress.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// EVENT DEFINITIONS (100+ Events)
// ============================================================================

/// All available events that can trigger functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // -------------------------------------------------------------------------
    // POST EVENTS (1-15)
    // -------------------------------------------------------------------------
    PostCreated,
    PostUpdated,
    PostDeleted,
    PostPublished,
    PostUnpublished,
    PostScheduled,
    PostTrashed,
    PostRestored,
    PostViewed,
    PostSlugChanged,
    PostCategoryChanged,
    PostTagsChanged,
    PostFeaturedImageSet,
    PostFeaturedImageRemoved,
    PostStatusChanged,

    // -------------------------------------------------------------------------
    // PAGE EVENTS (16-28)
    // -------------------------------------------------------------------------
    PageCreated,
    PageUpdated,
    PageDeleted,
    PagePublished,
    PageUnpublished,
    PageViewed,
    PageTemplateChanged,
    PageParentChanged,
    PageOrderChanged,
    PageSlugChanged,
    PageTrashed,
    PageRestored,
    PageStatusChanged,

    // -------------------------------------------------------------------------
    // USER EVENTS (29-48)
    // -------------------------------------------------------------------------
    UserRegistered,
    UserLoggedIn,
    UserLoggedOut,
    UserUpdated,
    UserDeleted,
    UserRoleChanged,
    UserPasswordChanged,
    UserPasswordReset,
    UserEmailVerified,
    UserProfileUpdated,
    UserAvatarChanged,
    UserActivated,
    UserDeactivated,
    UserLocked,
    UserUnlocked,
    UserSessionCreated,
    UserSessionDestroyed,
    UserPreferencesUpdated,
    User2FAEnabled,
    User2FADisabled,

    // -------------------------------------------------------------------------
    // COMMENT EVENTS (49-62)
    // -------------------------------------------------------------------------
    CommentCreated,
    CommentUpdated,
    CommentDeleted,
    CommentApproved,
    CommentUnapproved,
    CommentSpamMarked,
    CommentSpamUnmarked,
    CommentReplied,
    CommentEdited,
    CommentTrashed,
    CommentRestored,
    CommentReported,
    CommentLiked,
    CommentPinned,

    // -------------------------------------------------------------------------
    // MEDIA EVENTS (63-78)
    // -------------------------------------------------------------------------
    MediaUploaded,
    MediaDeleted,
    MediaUpdated,
    MediaViewed,
    MediaDownloaded,
    MediaThumbnailGenerated,
    MediaOptimized,
    MediaMoved,
    MediaCopied,
    MediaRenamed,
    MediaMetadataUpdated,
    MediaAltTextChanged,
    MediaCaptionChanged,
    MediaFolderCreated,
    MediaFolderDeleted,
    MediaBulkDeleted,

    // -------------------------------------------------------------------------
    // THEME EVENTS (79-92)
    // -------------------------------------------------------------------------
    ThemeInstalled,
    ThemeActivated,
    ThemeDeactivated,
    ThemeDeleted,
    ThemeUpdated,
    ThemeCustomized,
    ThemeExported,
    ThemeImported,
    ThemeOptionsUpdated,
    ThemeMenuAssigned,
    ThemeSidebarAssigned,
    ThemeWidgetAdded,
    ThemeWidgetRemoved,
    ThemeColorChanged,

    // -------------------------------------------------------------------------
    // PLUGIN EVENTS (93-106)
    // -------------------------------------------------------------------------
    PluginInstalled,
    PluginActivated,
    PluginDeactivated,
    PluginDeleted,
    PluginUpdated,
    PluginSettingsChanged,
    PluginHookRegistered,
    PluginHookUnregistered,
    PluginErrorOccurred,
    PluginLicenseActivated,
    PluginLicenseDeactivated,
    PluginDependencyMissing,
    PluginConflictDetected,
    PluginDataExported,

    // -------------------------------------------------------------------------
    // MENU EVENTS (107-116)
    // -------------------------------------------------------------------------
    MenuCreated,
    MenuUpdated,
    MenuDeleted,
    MenuItemAdded,
    MenuItemRemoved,
    MenuItemMoved,
    MenuAssigned,
    MenuUnassigned,
    MenuExported,
    MenuImported,

    // -------------------------------------------------------------------------
    // TAXONOMY EVENTS (117-128)
    // -------------------------------------------------------------------------
    CategoryCreated,
    CategoryUpdated,
    CategoryDeleted,
    CategoryMerged,
    TagCreated,
    TagUpdated,
    TagDeleted,
    TagMerged,
    TaxonomyTermCreated,
    TaxonomyTermUpdated,
    TaxonomyTermDeleted,
    TaxonomyReordered,

    // -------------------------------------------------------------------------
    // SETTINGS EVENTS (129-142)
    // -------------------------------------------------------------------------
    SettingsUpdated,
    SiteNameChanged,
    SiteDescriptionChanged,
    SiteUrlChanged,
    PermalinkStructureChanged,
    TimezoneChanged,
    DateFormatChanged,
    EmailSettingsChanged,
    ReadingSettingsChanged,
    WritingSettingsChanged,
    DiscussionSettingsChanged,
    PrivacySettingsChanged,
    SecuritySettingsChanged,
    PerformanceSettingsChanged,

    // -------------------------------------------------------------------------
    // BACKUP EVENTS (143-152)
    // -------------------------------------------------------------------------
    BackupCreated,
    BackupRestored,
    BackupDeleted,
    BackupScheduled,
    BackupFailed,
    BackupCompleted,
    BackupDownloaded,
    BackupUploaded,
    BackupVerified,
    BackupExpired,

    // -------------------------------------------------------------------------
    // SEO EVENTS (153-162)
    // -------------------------------------------------------------------------
    SeoMetaUpdated,
    SitemapGenerated,
    SitemapUpdated,
    RobotsFileUpdated,
    SchemaMarkupUpdated,
    CanonicalUrlSet,
    RedirectCreated,
    RedirectDeleted,
    SeoScoreCalculated,
    BrokenLinkDetected,

    // -------------------------------------------------------------------------
    // ANALYTICS EVENTS (163-172)
    // -------------------------------------------------------------------------
    PageViewRecorded,
    UniqueVisitorRecorded,
    BounceRecorded,
    ConversionRecorded,
    GoalCompleted,
    EventTracked,
    SearchPerformed,
    ExternalLinkClicked,
    FileDownloadTracked,
    VideoPlayTracked,

    // -------------------------------------------------------------------------
    // EMAIL EVENTS (173-182)
    // -------------------------------------------------------------------------
    EmailSent,
    EmailFailed,
    EmailBounced,
    EmailOpened,
    EmailClicked,
    EmailUnsubscribed,
    NewsletterSubscribed,
    NewsletterUnsubscribed,
    ContactFormSubmitted,
    EmailTemplateUpdated,

    // -------------------------------------------------------------------------
    // SECURITY EVENTS (183-196)
    // -------------------------------------------------------------------------
    LoginAttemptFailed,
    BruteForceDetected,
    IpBlocked,
    IpUnblocked,
    SuspiciousActivityDetected,
    MalwareDetected,
    FileIntegrityViolation,
    PermissionDenied,
    UnauthorizedAccess,
    CsrfTokenInvalid,
    SqlInjectionAttempt,
    XssAttempt,
    SecurityScanCompleted,
    VulnerabilityFound,

    // -------------------------------------------------------------------------
    // SCHEDULER EVENTS (197-220)
    // -------------------------------------------------------------------------
    SchedulerMinutely,
    SchedulerEvery5Minutes,
    SchedulerEvery10Minutes,
    SchedulerEvery15Minutes,
    SchedulerEvery30Minutes,
    SchedulerHourly,
    SchedulerEvery2Hours,
    SchedulerEvery4Hours,
    SchedulerEvery6Hours,
    SchedulerEvery12Hours,
    SchedulerDaily,
    SchedulerDailyMorning,
    SchedulerDailyNoon,
    SchedulerDailyEvening,
    SchedulerDailyMidnight,
    SchedulerWeekly,
    SchedulerWeeklyMonday,
    SchedulerWeeklyFriday,
    SchedulerBiweekly,
    SchedulerMonthly,
    SchedulerMonthlyFirst,
    SchedulerMonthlyLast,
    SchedulerQuarterly,
    SchedulerYearly,

    // -------------------------------------------------------------------------
    // SYSTEM EVENTS (221-240)
    // -------------------------------------------------------------------------
    SystemStartup,
    SystemShutdown,
    SystemError,
    SystemWarning,
    CacheCleared,
    CacheWarmed,
    DatabaseOptimized,
    DatabaseBackedUp,
    LogsRotated,
    TempFilesCleared,
    UpdateAvailable,
    UpdateInstalled,
    MaintenanceModeEnabled,
    MaintenanceModeDisabled,
    CronJobExecuted,
    QueueJobProcessed,
    QueueJobFailed,
    RateLimitExceeded,
    ApiKeyCreated,
    ApiKeyRevoked,

    // -------------------------------------------------------------------------
    // WEBHOOK EVENTS (241-250)
    // -------------------------------------------------------------------------
    WebhookReceived,
    WebhookSent,
    WebhookFailed,
    WebhookRetried,
    WebhookVerified,
    ExternalApiCalled,
    ExternalApiFailed,
    IntegrationConnected,
    IntegrationDisconnected,
    SyncCompleted,

    // -------------------------------------------------------------------------
    // E-COMMERCE EVENTS (251-270)
    // -------------------------------------------------------------------------
    OrderCreated,
    OrderUpdated,
    OrderCompleted,
    OrderCancelled,
    OrderRefunded,
    PaymentReceived,
    PaymentFailed,
    CartCreated,
    CartUpdated,
    CartAbandoned,
    ProductCreated,
    ProductUpdated,
    ProductDeleted,
    ProductOutOfStock,
    ProductBackInStock,
    CouponCreated,
    CouponUsed,
    CouponExpired,
    ShippingCalculated,
    InvoiceGenerated,

    // -------------------------------------------------------------------------
    // FORM EVENTS (271-280)
    // -------------------------------------------------------------------------
    FormSubmitted,
    FormValidationFailed,
    FormFieldUpdated,
    FormCreated,
    FormDeleted,
    FormEntryCreated,
    FormEntryDeleted,
    FormEntryExported,
    FormSpamDetected,
    FormFileUploaded,

    // -------------------------------------------------------------------------
    // CUSTOM EVENTS (281-300)
    // -------------------------------------------------------------------------
    CustomEvent1,
    CustomEvent2,
    CustomEvent3,
    CustomEvent4,
    CustomEvent5,
    CustomEvent6,
    CustomEvent7,
    CustomEvent8,
    CustomEvent9,
    CustomEvent10,
    CustomEventTriggered,
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    AutomationTriggered,
    NotificationSent,
    ReminderTriggered,
    AlertTriggered,
    ThresholdReached,
    MilestoneAchieved,
}

impl EventType {
    pub fn category(&self) -> &'static str {
        match self {
            Self::PostCreated | Self::PostUpdated | Self::PostDeleted |
            Self::PostPublished | Self::PostUnpublished | Self::PostScheduled |
            Self::PostTrashed | Self::PostRestored | Self::PostViewed |
            Self::PostSlugChanged | Self::PostCategoryChanged | Self::PostTagsChanged |
            Self::PostFeaturedImageSet | Self::PostFeaturedImageRemoved | Self::PostStatusChanged => "Posts",

            Self::PageCreated | Self::PageUpdated | Self::PageDeleted |
            Self::PagePublished | Self::PageUnpublished | Self::PageViewed |
            Self::PageTemplateChanged | Self::PageParentChanged | Self::PageOrderChanged |
            Self::PageSlugChanged | Self::PageTrashed | Self::PageRestored | Self::PageStatusChanged => "Pages",

            Self::UserRegistered | Self::UserLoggedIn | Self::UserLoggedOut |
            Self::UserUpdated | Self::UserDeleted | Self::UserRoleChanged |
            Self::UserPasswordChanged | Self::UserPasswordReset | Self::UserEmailVerified |
            Self::UserProfileUpdated | Self::UserAvatarChanged | Self::UserActivated |
            Self::UserDeactivated | Self::UserLocked | Self::UserUnlocked |
            Self::UserSessionCreated | Self::UserSessionDestroyed | Self::UserPreferencesUpdated |
            Self::User2FAEnabled | Self::User2FADisabled => "Users",

            Self::CommentCreated | Self::CommentUpdated | Self::CommentDeleted |
            Self::CommentApproved | Self::CommentUnapproved | Self::CommentSpamMarked |
            Self::CommentSpamUnmarked | Self::CommentReplied | Self::CommentEdited |
            Self::CommentTrashed | Self::CommentRestored | Self::CommentReported |
            Self::CommentLiked | Self::CommentPinned => "Comments",

            Self::MediaUploaded | Self::MediaDeleted | Self::MediaUpdated |
            Self::MediaViewed | Self::MediaDownloaded | Self::MediaThumbnailGenerated |
            Self::MediaOptimized | Self::MediaMoved | Self::MediaCopied |
            Self::MediaRenamed | Self::MediaMetadataUpdated | Self::MediaAltTextChanged |
            Self::MediaCaptionChanged | Self::MediaFolderCreated | Self::MediaFolderDeleted |
            Self::MediaBulkDeleted => "Media",

            Self::ThemeInstalled | Self::ThemeActivated | Self::ThemeDeactivated |
            Self::ThemeDeleted | Self::ThemeUpdated | Self::ThemeCustomized |
            Self::ThemeExported | Self::ThemeImported | Self::ThemeOptionsUpdated |
            Self::ThemeMenuAssigned | Self::ThemeSidebarAssigned | Self::ThemeWidgetAdded |
            Self::ThemeWidgetRemoved | Self::ThemeColorChanged => "Themes",

            Self::PluginInstalled | Self::PluginActivated | Self::PluginDeactivated |
            Self::PluginDeleted | Self::PluginUpdated | Self::PluginSettingsChanged |
            Self::PluginHookRegistered | Self::PluginHookUnregistered | Self::PluginErrorOccurred |
            Self::PluginLicenseActivated | Self::PluginLicenseDeactivated | Self::PluginDependencyMissing |
            Self::PluginConflictDetected | Self::PluginDataExported => "Plugins",

            Self::MenuCreated | Self::MenuUpdated | Self::MenuDeleted |
            Self::MenuItemAdded | Self::MenuItemRemoved | Self::MenuItemMoved |
            Self::MenuAssigned | Self::MenuUnassigned | Self::MenuExported |
            Self::MenuImported => "Menus",

            Self::CategoryCreated | Self::CategoryUpdated | Self::CategoryDeleted |
            Self::CategoryMerged | Self::TagCreated | Self::TagUpdated |
            Self::TagDeleted | Self::TagMerged | Self::TaxonomyTermCreated |
            Self::TaxonomyTermUpdated | Self::TaxonomyTermDeleted | Self::TaxonomyReordered => "Taxonomies",

            Self::SettingsUpdated | Self::SiteNameChanged | Self::SiteDescriptionChanged |
            Self::SiteUrlChanged | Self::PermalinkStructureChanged | Self::TimezoneChanged |
            Self::DateFormatChanged | Self::EmailSettingsChanged | Self::ReadingSettingsChanged |
            Self::WritingSettingsChanged | Self::DiscussionSettingsChanged | Self::PrivacySettingsChanged |
            Self::SecuritySettingsChanged | Self::PerformanceSettingsChanged => "Settings",

            Self::BackupCreated | Self::BackupRestored | Self::BackupDeleted |
            Self::BackupScheduled | Self::BackupFailed | Self::BackupCompleted |
            Self::BackupDownloaded | Self::BackupUploaded | Self::BackupVerified |
            Self::BackupExpired => "Backup",

            Self::SeoMetaUpdated | Self::SitemapGenerated | Self::SitemapUpdated |
            Self::RobotsFileUpdated | Self::SchemaMarkupUpdated | Self::CanonicalUrlSet |
            Self::RedirectCreated | Self::RedirectDeleted | Self::SeoScoreCalculated |
            Self::BrokenLinkDetected => "SEO",

            Self::PageViewRecorded | Self::UniqueVisitorRecorded | Self::BounceRecorded |
            Self::ConversionRecorded | Self::GoalCompleted | Self::EventTracked |
            Self::SearchPerformed | Self::ExternalLinkClicked | Self::FileDownloadTracked |
            Self::VideoPlayTracked => "Analytics",

            Self::EmailSent | Self::EmailFailed | Self::EmailBounced |
            Self::EmailOpened | Self::EmailClicked | Self::EmailUnsubscribed |
            Self::NewsletterSubscribed | Self::NewsletterUnsubscribed | Self::ContactFormSubmitted |
            Self::EmailTemplateUpdated => "Email",

            Self::LoginAttemptFailed | Self::BruteForceDetected | Self::IpBlocked |
            Self::IpUnblocked | Self::SuspiciousActivityDetected | Self::MalwareDetected |
            Self::FileIntegrityViolation | Self::PermissionDenied | Self::UnauthorizedAccess |
            Self::CsrfTokenInvalid | Self::SqlInjectionAttempt | Self::XssAttempt |
            Self::SecurityScanCompleted | Self::VulnerabilityFound => "Security",

            Self::SchedulerMinutely | Self::SchedulerEvery5Minutes | Self::SchedulerEvery10Minutes |
            Self::SchedulerEvery15Minutes | Self::SchedulerEvery30Minutes | Self::SchedulerHourly |
            Self::SchedulerEvery2Hours | Self::SchedulerEvery4Hours | Self::SchedulerEvery6Hours |
            Self::SchedulerEvery12Hours | Self::SchedulerDaily | Self::SchedulerDailyMorning |
            Self::SchedulerDailyNoon | Self::SchedulerDailyEvening | Self::SchedulerDailyMidnight |
            Self::SchedulerWeekly | Self::SchedulerWeeklyMonday | Self::SchedulerWeeklyFriday |
            Self::SchedulerBiweekly | Self::SchedulerMonthly | Self::SchedulerMonthlyFirst |
            Self::SchedulerMonthlyLast | Self::SchedulerQuarterly | Self::SchedulerYearly => "Scheduler",

            Self::SystemStartup | Self::SystemShutdown | Self::SystemError |
            Self::SystemWarning | Self::CacheCleared | Self::CacheWarmed |
            Self::DatabaseOptimized | Self::DatabaseBackedUp | Self::LogsRotated |
            Self::TempFilesCleared | Self::UpdateAvailable | Self::UpdateInstalled |
            Self::MaintenanceModeEnabled | Self::MaintenanceModeDisabled | Self::CronJobExecuted |
            Self::QueueJobProcessed | Self::QueueJobFailed | Self::RateLimitExceeded |
            Self::ApiKeyCreated | Self::ApiKeyRevoked => "System",

            Self::WebhookReceived | Self::WebhookSent | Self::WebhookFailed |
            Self::WebhookRetried | Self::WebhookVerified | Self::ExternalApiCalled |
            Self::ExternalApiFailed | Self::IntegrationConnected | Self::IntegrationDisconnected |
            Self::SyncCompleted => "Webhooks",

            Self::OrderCreated | Self::OrderUpdated | Self::OrderCompleted |
            Self::OrderCancelled | Self::OrderRefunded | Self::PaymentReceived |
            Self::PaymentFailed | Self::CartCreated | Self::CartUpdated |
            Self::CartAbandoned | Self::ProductCreated | Self::ProductUpdated |
            Self::ProductDeleted | Self::ProductOutOfStock | Self::ProductBackInStock |
            Self::CouponCreated | Self::CouponUsed | Self::CouponExpired |
            Self::ShippingCalculated | Self::InvoiceGenerated => "E-Commerce",

            Self::FormSubmitted | Self::FormValidationFailed | Self::FormFieldUpdated |
            Self::FormCreated | Self::FormDeleted | Self::FormEntryCreated |
            Self::FormEntryDeleted | Self::FormEntryExported | Self::FormSpamDetected |
            Self::FormFileUploaded => "Forms",

            _ => "Custom",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::PostCreated => "Triggered when a new post is created",
            Self::PostUpdated => "Triggered when an existing post is updated",
            Self::PostDeleted => "Triggered when a post is permanently deleted",
            Self::PostPublished => "Triggered when a post is published",
            Self::PostUnpublished => "Triggered when a post is unpublished",
            Self::PostScheduled => "Triggered when a post is scheduled for future publication",
            Self::PostTrashed => "Triggered when a post is moved to trash",
            Self::PostRestored => "Triggered when a post is restored from trash",
            Self::PostViewed => "Triggered when a post is viewed by a visitor",
            Self::PostSlugChanged => "Triggered when a post's URL slug is changed",
            Self::PostCategoryChanged => "Triggered when a post's categories are modified",
            Self::PostTagsChanged => "Triggered when a post's tags are modified",
            Self::PostFeaturedImageSet => "Triggered when a featured image is set for a post",
            Self::PostFeaturedImageRemoved => "Triggered when a featured image is removed from a post",
            Self::PostStatusChanged => "Triggered when a post's status changes",

            Self::UserLoggedIn => "Triggered when a user logs in",
            Self::UserLoggedOut => "Triggered when a user logs out",
            Self::UserRegistered => "Triggered when a new user registers",
            Self::UserUpdated => "Triggered when user information is updated",
            Self::UserDeleted => "Triggered when a user is deleted",
            Self::UserRoleChanged => "Triggered when a user's role is changed",
            Self::UserPasswordChanged => "Triggered when a user changes their password",
            Self::UserPasswordReset => "Triggered when a password reset is requested",

            Self::MediaUploaded => "Triggered when a media file is uploaded",
            Self::MediaDeleted => "Triggered when a media file is deleted",
            Self::MediaUpdated => "Triggered when media metadata is updated",

            Self::CommentCreated => "Triggered when a new comment is posted",
            Self::CommentApproved => "Triggered when a comment is approved",
            Self::CommentDeleted => "Triggered when a comment is deleted",
            Self::CommentSpamMarked => "Triggered when a comment is marked as spam",

            Self::ThemeActivated => "Triggered when a theme is activated",
            Self::ThemeDeactivated => "Triggered when a theme is deactivated",
            Self::ThemeInstalled => "Triggered when a theme is installed",

            Self::PluginActivated => "Triggered when a plugin is activated",
            Self::PluginDeactivated => "Triggered when a plugin is deactivated",
            Self::PluginInstalled => "Triggered when a plugin is installed",

            Self::SchedulerMinutely => "Triggered every minute",
            Self::SchedulerEvery5Minutes => "Triggered every 5 minutes",
            Self::SchedulerEvery15Minutes => "Triggered every 15 minutes",
            Self::SchedulerHourly => "Triggered every hour",
            Self::SchedulerDaily => "Triggered once daily",
            Self::SchedulerWeekly => "Triggered once weekly",
            Self::SchedulerMonthly => "Triggered once monthly",

            Self::BackupCreated => "Triggered when a backup is created",
            Self::BackupCompleted => "Triggered when a backup completes successfully",
            Self::BackupFailed => "Triggered when a backup fails",

            Self::EmailSent => "Triggered when an email is sent successfully",
            Self::EmailFailed => "Triggered when an email fails to send",

            Self::LoginAttemptFailed => "Triggered when a login attempt fails",
            Self::BruteForceDetected => "Triggered when brute force attack is detected",
            Self::IpBlocked => "Triggered when an IP address is blocked",

            Self::SystemStartup => "Triggered when the system starts up",
            Self::SystemShutdown => "Triggered when the system shuts down",
            Self::SystemError => "Triggered when a system error occurs",
            Self::CacheCleared => "Triggered when the cache is cleared",

            Self::WebhookReceived => "Triggered when a webhook is received",
            Self::WebhookSent => "Triggered when a webhook is sent",

            Self::OrderCreated => "Triggered when a new order is created",
            Self::OrderCompleted => "Triggered when an order is completed",
            Self::PaymentReceived => "Triggered when a payment is received",

            Self::FormSubmitted => "Triggered when a form is submitted",

            _ => "Custom event trigger",
        }
    }

    pub fn all_events() -> Vec<EventInfo> {
        vec![
            // Posts
            EventInfo::new(Self::PostCreated, "post_created"),
            EventInfo::new(Self::PostUpdated, "post_updated"),
            EventInfo::new(Self::PostDeleted, "post_deleted"),
            EventInfo::new(Self::PostPublished, "post_published"),
            EventInfo::new(Self::PostUnpublished, "post_unpublished"),
            EventInfo::new(Self::PostScheduled, "post_scheduled"),
            EventInfo::new(Self::PostTrashed, "post_trashed"),
            EventInfo::new(Self::PostRestored, "post_restored"),
            EventInfo::new(Self::PostViewed, "post_viewed"),
            EventInfo::new(Self::PostSlugChanged, "post_slug_changed"),
            EventInfo::new(Self::PostCategoryChanged, "post_category_changed"),
            EventInfo::new(Self::PostTagsChanged, "post_tags_changed"),
            EventInfo::new(Self::PostFeaturedImageSet, "post_featured_image_set"),
            EventInfo::new(Self::PostFeaturedImageRemoved, "post_featured_image_removed"),
            EventInfo::new(Self::PostStatusChanged, "post_status_changed"),
            // Pages
            EventInfo::new(Self::PageCreated, "page_created"),
            EventInfo::new(Self::PageUpdated, "page_updated"),
            EventInfo::new(Self::PageDeleted, "page_deleted"),
            EventInfo::new(Self::PagePublished, "page_published"),
            EventInfo::new(Self::PageUnpublished, "page_unpublished"),
            EventInfo::new(Self::PageViewed, "page_viewed"),
            EventInfo::new(Self::PageTemplateChanged, "page_template_changed"),
            EventInfo::new(Self::PageParentChanged, "page_parent_changed"),
            EventInfo::new(Self::PageOrderChanged, "page_order_changed"),
            EventInfo::new(Self::PageSlugChanged, "page_slug_changed"),
            EventInfo::new(Self::PageTrashed, "page_trashed"),
            EventInfo::new(Self::PageRestored, "page_restored"),
            EventInfo::new(Self::PageStatusChanged, "page_status_changed"),
            // Users
            EventInfo::new(Self::UserRegistered, "user_registered"),
            EventInfo::new(Self::UserLoggedIn, "user_logged_in"),
            EventInfo::new(Self::UserLoggedOut, "user_logged_out"),
            EventInfo::new(Self::UserUpdated, "user_updated"),
            EventInfo::new(Self::UserDeleted, "user_deleted"),
            EventInfo::new(Self::UserRoleChanged, "user_role_changed"),
            EventInfo::new(Self::UserPasswordChanged, "user_password_changed"),
            EventInfo::new(Self::UserPasswordReset, "user_password_reset"),
            EventInfo::new(Self::UserEmailVerified, "user_email_verified"),
            EventInfo::new(Self::UserProfileUpdated, "user_profile_updated"),
            EventInfo::new(Self::UserAvatarChanged, "user_avatar_changed"),
            EventInfo::new(Self::UserActivated, "user_activated"),
            EventInfo::new(Self::UserDeactivated, "user_deactivated"),
            EventInfo::new(Self::UserLocked, "user_locked"),
            EventInfo::new(Self::UserUnlocked, "user_unlocked"),
            EventInfo::new(Self::UserSessionCreated, "user_session_created"),
            EventInfo::new(Self::UserSessionDestroyed, "user_session_destroyed"),
            EventInfo::new(Self::UserPreferencesUpdated, "user_preferences_updated"),
            EventInfo::new(Self::User2FAEnabled, "user_2fa_enabled"),
            EventInfo::new(Self::User2FADisabled, "user_2fa_disabled"),
            // Comments
            EventInfo::new(Self::CommentCreated, "comment_created"),
            EventInfo::new(Self::CommentUpdated, "comment_updated"),
            EventInfo::new(Self::CommentDeleted, "comment_deleted"),
            EventInfo::new(Self::CommentApproved, "comment_approved"),
            EventInfo::new(Self::CommentUnapproved, "comment_unapproved"),
            EventInfo::new(Self::CommentSpamMarked, "comment_spam_marked"),
            EventInfo::new(Self::CommentSpamUnmarked, "comment_spam_unmarked"),
            EventInfo::new(Self::CommentReplied, "comment_replied"),
            EventInfo::new(Self::CommentEdited, "comment_edited"),
            EventInfo::new(Self::CommentTrashed, "comment_trashed"),
            EventInfo::new(Self::CommentRestored, "comment_restored"),
            EventInfo::new(Self::CommentReported, "comment_reported"),
            EventInfo::new(Self::CommentLiked, "comment_liked"),
            EventInfo::new(Self::CommentPinned, "comment_pinned"),
            // Media
            EventInfo::new(Self::MediaUploaded, "media_uploaded"),
            EventInfo::new(Self::MediaDeleted, "media_deleted"),
            EventInfo::new(Self::MediaUpdated, "media_updated"),
            EventInfo::new(Self::MediaViewed, "media_viewed"),
            EventInfo::new(Self::MediaDownloaded, "media_downloaded"),
            EventInfo::new(Self::MediaThumbnailGenerated, "media_thumbnail_generated"),
            EventInfo::new(Self::MediaOptimized, "media_optimized"),
            EventInfo::new(Self::MediaMoved, "media_moved"),
            EventInfo::new(Self::MediaCopied, "media_copied"),
            EventInfo::new(Self::MediaRenamed, "media_renamed"),
            EventInfo::new(Self::MediaMetadataUpdated, "media_metadata_updated"),
            EventInfo::new(Self::MediaAltTextChanged, "media_alt_text_changed"),
            EventInfo::new(Self::MediaCaptionChanged, "media_caption_changed"),
            EventInfo::new(Self::MediaFolderCreated, "media_folder_created"),
            EventInfo::new(Self::MediaFolderDeleted, "media_folder_deleted"),
            EventInfo::new(Self::MediaBulkDeleted, "media_bulk_deleted"),
            // Themes
            EventInfo::new(Self::ThemeInstalled, "theme_installed"),
            EventInfo::new(Self::ThemeActivated, "theme_activated"),
            EventInfo::new(Self::ThemeDeactivated, "theme_deactivated"),
            EventInfo::new(Self::ThemeDeleted, "theme_deleted"),
            EventInfo::new(Self::ThemeUpdated, "theme_updated"),
            EventInfo::new(Self::ThemeCustomized, "theme_customized"),
            EventInfo::new(Self::ThemeExported, "theme_exported"),
            EventInfo::new(Self::ThemeImported, "theme_imported"),
            EventInfo::new(Self::ThemeOptionsUpdated, "theme_options_updated"),
            EventInfo::new(Self::ThemeMenuAssigned, "theme_menu_assigned"),
            EventInfo::new(Self::ThemeSidebarAssigned, "theme_sidebar_assigned"),
            EventInfo::new(Self::ThemeWidgetAdded, "theme_widget_added"),
            EventInfo::new(Self::ThemeWidgetRemoved, "theme_widget_removed"),
            EventInfo::new(Self::ThemeColorChanged, "theme_color_changed"),
            // Plugins
            EventInfo::new(Self::PluginInstalled, "plugin_installed"),
            EventInfo::new(Self::PluginActivated, "plugin_activated"),
            EventInfo::new(Self::PluginDeactivated, "plugin_deactivated"),
            EventInfo::new(Self::PluginDeleted, "plugin_deleted"),
            EventInfo::new(Self::PluginUpdated, "plugin_updated"),
            EventInfo::new(Self::PluginSettingsChanged, "plugin_settings_changed"),
            EventInfo::new(Self::PluginHookRegistered, "plugin_hook_registered"),
            EventInfo::new(Self::PluginHookUnregistered, "plugin_hook_unregistered"),
            EventInfo::new(Self::PluginErrorOccurred, "plugin_error_occurred"),
            EventInfo::new(Self::PluginLicenseActivated, "plugin_license_activated"),
            EventInfo::new(Self::PluginLicenseDeactivated, "plugin_license_deactivated"),
            EventInfo::new(Self::PluginDependencyMissing, "plugin_dependency_missing"),
            EventInfo::new(Self::PluginConflictDetected, "plugin_conflict_detected"),
            EventInfo::new(Self::PluginDataExported, "plugin_data_exported"),
            // Menus
            EventInfo::new(Self::MenuCreated, "menu_created"),
            EventInfo::new(Self::MenuUpdated, "menu_updated"),
            EventInfo::new(Self::MenuDeleted, "menu_deleted"),
            EventInfo::new(Self::MenuItemAdded, "menu_item_added"),
            EventInfo::new(Self::MenuItemRemoved, "menu_item_removed"),
            EventInfo::new(Self::MenuItemMoved, "menu_item_moved"),
            EventInfo::new(Self::MenuAssigned, "menu_assigned"),
            EventInfo::new(Self::MenuUnassigned, "menu_unassigned"),
            EventInfo::new(Self::MenuExported, "menu_exported"),
            EventInfo::new(Self::MenuImported, "menu_imported"),
            // Taxonomies
            EventInfo::new(Self::CategoryCreated, "category_created"),
            EventInfo::new(Self::CategoryUpdated, "category_updated"),
            EventInfo::new(Self::CategoryDeleted, "category_deleted"),
            EventInfo::new(Self::CategoryMerged, "category_merged"),
            EventInfo::new(Self::TagCreated, "tag_created"),
            EventInfo::new(Self::TagUpdated, "tag_updated"),
            EventInfo::new(Self::TagDeleted, "tag_deleted"),
            EventInfo::new(Self::TagMerged, "tag_merged"),
            EventInfo::new(Self::TaxonomyTermCreated, "taxonomy_term_created"),
            EventInfo::new(Self::TaxonomyTermUpdated, "taxonomy_term_updated"),
            EventInfo::new(Self::TaxonomyTermDeleted, "taxonomy_term_deleted"),
            EventInfo::new(Self::TaxonomyReordered, "taxonomy_reordered"),
            // Settings
            EventInfo::new(Self::SettingsUpdated, "settings_updated"),
            EventInfo::new(Self::SiteNameChanged, "site_name_changed"),
            EventInfo::new(Self::SiteDescriptionChanged, "site_description_changed"),
            EventInfo::new(Self::SiteUrlChanged, "site_url_changed"),
            EventInfo::new(Self::PermalinkStructureChanged, "permalink_structure_changed"),
            EventInfo::new(Self::TimezoneChanged, "timezone_changed"),
            EventInfo::new(Self::DateFormatChanged, "date_format_changed"),
            EventInfo::new(Self::EmailSettingsChanged, "email_settings_changed"),
            EventInfo::new(Self::ReadingSettingsChanged, "reading_settings_changed"),
            EventInfo::new(Self::WritingSettingsChanged, "writing_settings_changed"),
            EventInfo::new(Self::DiscussionSettingsChanged, "discussion_settings_changed"),
            EventInfo::new(Self::PrivacySettingsChanged, "privacy_settings_changed"),
            EventInfo::new(Self::SecuritySettingsChanged, "security_settings_changed"),
            EventInfo::new(Self::PerformanceSettingsChanged, "performance_settings_changed"),
            // Backup
            EventInfo::new(Self::BackupCreated, "backup_created"),
            EventInfo::new(Self::BackupRestored, "backup_restored"),
            EventInfo::new(Self::BackupDeleted, "backup_deleted"),
            EventInfo::new(Self::BackupScheduled, "backup_scheduled"),
            EventInfo::new(Self::BackupFailed, "backup_failed"),
            EventInfo::new(Self::BackupCompleted, "backup_completed"),
            EventInfo::new(Self::BackupDownloaded, "backup_downloaded"),
            EventInfo::new(Self::BackupUploaded, "backup_uploaded"),
            EventInfo::new(Self::BackupVerified, "backup_verified"),
            EventInfo::new(Self::BackupExpired, "backup_expired"),
            // SEO
            EventInfo::new(Self::SeoMetaUpdated, "seo_meta_updated"),
            EventInfo::new(Self::SitemapGenerated, "sitemap_generated"),
            EventInfo::new(Self::SitemapUpdated, "sitemap_updated"),
            EventInfo::new(Self::RobotsFileUpdated, "robots_file_updated"),
            EventInfo::new(Self::SchemaMarkupUpdated, "schema_markup_updated"),
            EventInfo::new(Self::CanonicalUrlSet, "canonical_url_set"),
            EventInfo::new(Self::RedirectCreated, "redirect_created"),
            EventInfo::new(Self::RedirectDeleted, "redirect_deleted"),
            EventInfo::new(Self::SeoScoreCalculated, "seo_score_calculated"),
            EventInfo::new(Self::BrokenLinkDetected, "broken_link_detected"),
            // Analytics
            EventInfo::new(Self::PageViewRecorded, "page_view_recorded"),
            EventInfo::new(Self::UniqueVisitorRecorded, "unique_visitor_recorded"),
            EventInfo::new(Self::BounceRecorded, "bounce_recorded"),
            EventInfo::new(Self::ConversionRecorded, "conversion_recorded"),
            EventInfo::new(Self::GoalCompleted, "goal_completed"),
            EventInfo::new(Self::EventTracked, "event_tracked"),
            EventInfo::new(Self::SearchPerformed, "search_performed"),
            EventInfo::new(Self::ExternalLinkClicked, "external_link_clicked"),
            EventInfo::new(Self::FileDownloadTracked, "file_download_tracked"),
            EventInfo::new(Self::VideoPlayTracked, "video_play_tracked"),
            // Email
            EventInfo::new(Self::EmailSent, "email_sent"),
            EventInfo::new(Self::EmailFailed, "email_failed"),
            EventInfo::new(Self::EmailBounced, "email_bounced"),
            EventInfo::new(Self::EmailOpened, "email_opened"),
            EventInfo::new(Self::EmailClicked, "email_clicked"),
            EventInfo::new(Self::EmailUnsubscribed, "email_unsubscribed"),
            EventInfo::new(Self::NewsletterSubscribed, "newsletter_subscribed"),
            EventInfo::new(Self::NewsletterUnsubscribed, "newsletter_unsubscribed"),
            EventInfo::new(Self::ContactFormSubmitted, "contact_form_submitted"),
            EventInfo::new(Self::EmailTemplateUpdated, "email_template_updated"),
            // Security
            EventInfo::new(Self::LoginAttemptFailed, "login_attempt_failed"),
            EventInfo::new(Self::BruteForceDetected, "brute_force_detected"),
            EventInfo::new(Self::IpBlocked, "ip_blocked"),
            EventInfo::new(Self::IpUnblocked, "ip_unblocked"),
            EventInfo::new(Self::SuspiciousActivityDetected, "suspicious_activity_detected"),
            EventInfo::new(Self::MalwareDetected, "malware_detected"),
            EventInfo::new(Self::FileIntegrityViolation, "file_integrity_violation"),
            EventInfo::new(Self::PermissionDenied, "permission_denied"),
            EventInfo::new(Self::UnauthorizedAccess, "unauthorized_access"),
            EventInfo::new(Self::CsrfTokenInvalid, "csrf_token_invalid"),
            EventInfo::new(Self::SqlInjectionAttempt, "sql_injection_attempt"),
            EventInfo::new(Self::XssAttempt, "xss_attempt"),
            EventInfo::new(Self::SecurityScanCompleted, "security_scan_completed"),
            EventInfo::new(Self::VulnerabilityFound, "vulnerability_found"),
            // Scheduler
            EventInfo::new(Self::SchedulerMinutely, "scheduler_minutely"),
            EventInfo::new(Self::SchedulerEvery5Minutes, "scheduler_every_5_minutes"),
            EventInfo::new(Self::SchedulerEvery10Minutes, "scheduler_every_10_minutes"),
            EventInfo::new(Self::SchedulerEvery15Minutes, "scheduler_every_15_minutes"),
            EventInfo::new(Self::SchedulerEvery30Minutes, "scheduler_every_30_minutes"),
            EventInfo::new(Self::SchedulerHourly, "scheduler_hourly"),
            EventInfo::new(Self::SchedulerEvery2Hours, "scheduler_every_2_hours"),
            EventInfo::new(Self::SchedulerEvery4Hours, "scheduler_every_4_hours"),
            EventInfo::new(Self::SchedulerEvery6Hours, "scheduler_every_6_hours"),
            EventInfo::new(Self::SchedulerEvery12Hours, "scheduler_every_12_hours"),
            EventInfo::new(Self::SchedulerDaily, "scheduler_daily"),
            EventInfo::new(Self::SchedulerDailyMorning, "scheduler_daily_morning"),
            EventInfo::new(Self::SchedulerDailyNoon, "scheduler_daily_noon"),
            EventInfo::new(Self::SchedulerDailyEvening, "scheduler_daily_evening"),
            EventInfo::new(Self::SchedulerDailyMidnight, "scheduler_daily_midnight"),
            EventInfo::new(Self::SchedulerWeekly, "scheduler_weekly"),
            EventInfo::new(Self::SchedulerWeeklyMonday, "scheduler_weekly_monday"),
            EventInfo::new(Self::SchedulerWeeklyFriday, "scheduler_weekly_friday"),
            EventInfo::new(Self::SchedulerBiweekly, "scheduler_biweekly"),
            EventInfo::new(Self::SchedulerMonthly, "scheduler_monthly"),
            EventInfo::new(Self::SchedulerMonthlyFirst, "scheduler_monthly_first"),
            EventInfo::new(Self::SchedulerMonthlyLast, "scheduler_monthly_last"),
            EventInfo::new(Self::SchedulerQuarterly, "scheduler_quarterly"),
            EventInfo::new(Self::SchedulerYearly, "scheduler_yearly"),
            // System
            EventInfo::new(Self::SystemStartup, "system_startup"),
            EventInfo::new(Self::SystemShutdown, "system_shutdown"),
            EventInfo::new(Self::SystemError, "system_error"),
            EventInfo::new(Self::SystemWarning, "system_warning"),
            EventInfo::new(Self::CacheCleared, "cache_cleared"),
            EventInfo::new(Self::CacheWarmed, "cache_warmed"),
            EventInfo::new(Self::DatabaseOptimized, "database_optimized"),
            EventInfo::new(Self::DatabaseBackedUp, "database_backed_up"),
            EventInfo::new(Self::LogsRotated, "logs_rotated"),
            EventInfo::new(Self::TempFilesCleared, "temp_files_cleared"),
            EventInfo::new(Self::UpdateAvailable, "update_available"),
            EventInfo::new(Self::UpdateInstalled, "update_installed"),
            EventInfo::new(Self::MaintenanceModeEnabled, "maintenance_mode_enabled"),
            EventInfo::new(Self::MaintenanceModeDisabled, "maintenance_mode_disabled"),
            EventInfo::new(Self::CronJobExecuted, "cron_job_executed"),
            EventInfo::new(Self::QueueJobProcessed, "queue_job_processed"),
            EventInfo::new(Self::QueueJobFailed, "queue_job_failed"),
            EventInfo::new(Self::RateLimitExceeded, "rate_limit_exceeded"),
            EventInfo::new(Self::ApiKeyCreated, "api_key_created"),
            EventInfo::new(Self::ApiKeyRevoked, "api_key_revoked"),
            // Webhooks
            EventInfo::new(Self::WebhookReceived, "webhook_received"),
            EventInfo::new(Self::WebhookSent, "webhook_sent"),
            EventInfo::new(Self::WebhookFailed, "webhook_failed"),
            EventInfo::new(Self::WebhookRetried, "webhook_retried"),
            EventInfo::new(Self::WebhookVerified, "webhook_verified"),
            EventInfo::new(Self::ExternalApiCalled, "external_api_called"),
            EventInfo::new(Self::ExternalApiFailed, "external_api_failed"),
            EventInfo::new(Self::IntegrationConnected, "integration_connected"),
            EventInfo::new(Self::IntegrationDisconnected, "integration_disconnected"),
            EventInfo::new(Self::SyncCompleted, "sync_completed"),
            // E-Commerce
            EventInfo::new(Self::OrderCreated, "order_created"),
            EventInfo::new(Self::OrderUpdated, "order_updated"),
            EventInfo::new(Self::OrderCompleted, "order_completed"),
            EventInfo::new(Self::OrderCancelled, "order_cancelled"),
            EventInfo::new(Self::OrderRefunded, "order_refunded"),
            EventInfo::new(Self::PaymentReceived, "payment_received"),
            EventInfo::new(Self::PaymentFailed, "payment_failed"),
            EventInfo::new(Self::CartCreated, "cart_created"),
            EventInfo::new(Self::CartUpdated, "cart_updated"),
            EventInfo::new(Self::CartAbandoned, "cart_abandoned"),
            EventInfo::new(Self::ProductCreated, "product_created"),
            EventInfo::new(Self::ProductUpdated, "product_updated"),
            EventInfo::new(Self::ProductDeleted, "product_deleted"),
            EventInfo::new(Self::ProductOutOfStock, "product_out_of_stock"),
            EventInfo::new(Self::ProductBackInStock, "product_back_in_stock"),
            EventInfo::new(Self::CouponCreated, "coupon_created"),
            EventInfo::new(Self::CouponUsed, "coupon_used"),
            EventInfo::new(Self::CouponExpired, "coupon_expired"),
            EventInfo::new(Self::ShippingCalculated, "shipping_calculated"),
            EventInfo::new(Self::InvoiceGenerated, "invoice_generated"),
            // Forms
            EventInfo::new(Self::FormSubmitted, "form_submitted"),
            EventInfo::new(Self::FormValidationFailed, "form_validation_failed"),
            EventInfo::new(Self::FormFieldUpdated, "form_field_updated"),
            EventInfo::new(Self::FormCreated, "form_created"),
            EventInfo::new(Self::FormDeleted, "form_deleted"),
            EventInfo::new(Self::FormEntryCreated, "form_entry_created"),
            EventInfo::new(Self::FormEntryDeleted, "form_entry_deleted"),
            EventInfo::new(Self::FormEntryExported, "form_entry_exported"),
            EventInfo::new(Self::FormSpamDetected, "form_spam_detected"),
            EventInfo::new(Self::FormFileUploaded, "form_file_uploaded"),
            // Custom
            EventInfo::new(Self::CustomEvent1, "custom_event_1"),
            EventInfo::new(Self::CustomEvent2, "custom_event_2"),
            EventInfo::new(Self::CustomEvent3, "custom_event_3"),
            EventInfo::new(Self::CustomEvent4, "custom_event_4"),
            EventInfo::new(Self::CustomEvent5, "custom_event_5"),
            EventInfo::new(Self::CustomEvent6, "custom_event_6"),
            EventInfo::new(Self::CustomEvent7, "custom_event_7"),
            EventInfo::new(Self::CustomEvent8, "custom_event_8"),
            EventInfo::new(Self::CustomEvent9, "custom_event_9"),
            EventInfo::new(Self::CustomEvent10, "custom_event_10"),
            EventInfo::new(Self::CustomEventTriggered, "custom_event_triggered"),
            EventInfo::new(Self::WorkflowStarted, "workflow_started"),
            EventInfo::new(Self::WorkflowCompleted, "workflow_completed"),
            EventInfo::new(Self::WorkflowFailed, "workflow_failed"),
            EventInfo::new(Self::AutomationTriggered, "automation_triggered"),
            EventInfo::new(Self::NotificationSent, "notification_sent"),
            EventInfo::new(Self::ReminderTriggered, "reminder_triggered"),
            EventInfo::new(Self::AlertTriggered, "alert_triggered"),
            EventInfo::new(Self::ThresholdReached, "threshold_reached"),
            EventInfo::new(Self::MilestoneAchieved, "milestone_achieved"),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub event_type: EventType,
    pub slug: String,
    pub category: String,
    pub description: String,
}

impl EventInfo {
    pub fn new(event_type: EventType, slug: &str) -> Self {
        Self {
            category: event_type.category().to_string(),
            description: event_type.description().to_string(),
            event_type,
            slug: slug.to_string(),
        }
    }
}

// ============================================================================
// FUNCTION TYPES
// ============================================================================

/// Function execution language/runtime
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionRuntime {
    JavaScript,
    TypeScript,
    Rust,
    Lua,
    Sql,
    HttpWebhook,
    Email,
    Slack,
    Discord,
}

/// Function status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionStatus {
    Active,
    Inactive,
    Draft,
    Error,
}

/// Hook timing - when to execute the function relative to the event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HookTiming {
    /// Execute before the event action (can modify/cancel the action)
    Before,
    /// Execute after the event action (for notifications, logging, etc.)
    #[default]
    After,
    /// Execute both before and after
    Both,
}

impl HookTiming {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
            Self::Both => "both",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "before" => Self::Before,
            "both" => Self::Both,
            _ => Self::After,
        }
    }
}

// ============================================================================
// DATABASE MODELS
// ============================================================================

/// Stored function definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Function {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub runtime: String,
    pub code: String,
    pub trigger_events: serde_json::Value, // Vec<String>
    pub status: String,
    pub hook_timing: String, // "before", "after", or "both"
    pub priority: i32, // Lower = runs first (default 10)
    pub timeout_ms: i32,
    pub max_retries: i32,
    pub retry_delay_ms: i32,
    pub config: serde_json::Value, // HashMap<String, Value>
    pub environment_vars: serde_json::Value, // HashMap<String, String>
    pub is_template: bool,
    pub template_category: Option<String>,
    pub execution_count: i64,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Function execution log
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FunctionExecution {
    pub id: Uuid,
    pub function_id: Uuid,
    pub trigger_event: String,
    pub event_data: serde_json::Value,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: i32,
}

/// Scheduled function run
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledRun {
    pub id: Uuid,
    pub function_id: Uuid,
    pub scheduled_for: DateTime<Utc>,
    pub cron_expression: Option<String>,
    pub status: String,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// API MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFunctionRequest {
    pub name: String,
    pub description: Option<String>,
    pub runtime: String,
    pub code: String,
    pub trigger_events: Vec<String>,
    pub hook_timing: Option<String>, // "before", "after", "both" (default: "after")
    pub priority: Option<i32>, // Lower = runs first (default: 10)
    pub timeout_ms: Option<i32>,
    pub max_retries: Option<i32>,
    pub config: Option<HashMap<String, serde_json::Value>>,
    pub environment_vars: Option<HashMap<String, String>>,
    pub is_template: Option<bool>,
    pub template_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFunctionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub runtime: Option<String>,
    pub code: Option<String>,
    pub trigger_events: Option<Vec<String>>,
    pub hook_timing: Option<String>,
    pub priority: Option<i32>,
    pub status: Option<String>,
    pub timeout_ms: Option<i32>,
    pub max_retries: Option<i32>,
    pub config: Option<HashMap<String, serde_json::Value>>,
    pub environment_vars: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteFunctionRequest {
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
    pub runtime: Option<String>,
    pub is_template: Option<bool>,
    pub category: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub function_id: Option<Uuid>,
    pub status: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

// ============================================================================
// ROUTER
// ============================================================================

#[derive(Clone)]
pub struct FunctionsState {
    pub pool: PgPool,
}

pub fn functions_router(pool: PgPool) -> Router {
    let state = FunctionsState { pool };

    Router::new()
        // Events
        .route("/events", get(get_all_events))
        .route("/events/categories", get(get_event_categories))
        // Functions CRUD
        .route("/", get(list_functions))
        .route("/", post(create_function))
        .route("/:id", get(get_function))
        .route("/:id", put(update_function))
        .route("/:id", delete(delete_function))
        .route("/:id/duplicate", post(duplicate_function))
        .route("/:id/toggle", post(toggle_function_status))
        // Execution
        .route("/:id/execute", post(execute_function))
        .route("/:id/test", post(test_function))
        .route("/trigger/:event", post(trigger_event))
        // Executions log
        .route("/executions", get(list_executions))
        .route("/executions/:id", get(get_execution))
        .route("/executions/:id/retry", post(retry_execution))
        // Templates
        .route("/templates", get(get_function_templates))
        .route("/templates/:slug", get(get_template_by_slug))
        .route("/templates/:slug/use", post(use_template))
        // Stats
        .route("/stats", get(get_function_stats))
        .route("/stats/executions", get(get_execution_stats))
        // Scheduler
        .route("/scheduled", get(list_scheduled_runs))
        .route("/scheduled", post(create_scheduled_run))
        .route("/scheduled/:id", delete(delete_scheduled_run))
        .with_state(state)
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Get all available events
async fn get_all_events() -> impl IntoResponse {
    let events = EventType::all_events();
    Json(serde_json::json!({
        "success": true,
        "data": events,
        "total": events.len()
    }))
}

/// Get event categories
async fn get_event_categories() -> impl IntoResponse {
    let events = EventType::all_events();
    let mut categories: HashMap<String, Vec<EventInfo>> = HashMap::new();

    for event in events {
        categories
            .entry(event.category.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    Json(serde_json::json!({
        "success": true,
        "data": categories
    }))
}

/// List all functions
async fn list_functions(
    State(state): State<FunctionsState>,
    Query(query): Query<FunctionListQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut sql = String::from("SELECT * FROM functions WHERE 1=1");
    let mut count_sql = String::from("SELECT COUNT(*) FROM functions WHERE 1=1");

    if let Some(status) = &query.status {
        sql.push_str(&format!(" AND status = '{}'", status));
        count_sql.push_str(&format!(" AND status = '{}'", status));
    }

    if let Some(runtime) = &query.runtime {
        sql.push_str(&format!(" AND runtime = '{}'", runtime));
        count_sql.push_str(&format!(" AND runtime = '{}'", runtime));
    }

    if let Some(is_template) = query.is_template {
        sql.push_str(&format!(" AND is_template = {}", is_template));
        count_sql.push_str(&format!(" AND is_template = {}", is_template));
    }

    if let Some(category) = &query.category {
        sql.push_str(&format!(" AND template_category = '{}'", category));
        count_sql.push_str(&format!(" AND template_category = '{}'", category));
    }

    if let Some(search) = &query.search {
        let search_cond = format!(" AND (name ILIKE '%{}%' OR description ILIKE '%{}%')", search, search);
        sql.push_str(&search_cond);
        count_sql.push_str(&search_cond);
    }

    sql.push_str(" ORDER BY created_at DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", per_page, offset));

    let functions: Vec<Function> = match sqlx::query_as(&sql)
        .fetch_all(&state.pool)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Database error: {}", e)
                }))
            ).into_response();
        }
    };

    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    Json(serde_json::json!({
        "success": true,
        "data": functions,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total.0,
            "total_pages": (total.0 as f64 / per_page as f64).ceil() as i64
        }
    })).into_response()
}

/// Create a new function
async fn create_function(
    State(state): State<FunctionsState>,
    Json(req): Json<CreateFunctionRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let slug = req.name.to_lowercase().replace(' ', "-");
    let now = Utc::now();

    let hook_timing = req.hook_timing.clone().unwrap_or_else(|| "after".to_string());
    let priority = req.priority.unwrap_or(10);

    let result = sqlx::query(
        r#"INSERT INTO functions (
            id, name, slug, description, runtime, code, trigger_events, status,
            hook_timing, priority, timeout_ms, max_retries, retry_delay_ms, config, environment_vars,
            is_template, template_category, execution_count, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, $9, $10, $11, 1000, $12, $13, $14, $15, 0, $16, $16)"#
    )
    .bind(id)
    .bind(&req.name)
    .bind(&slug)
    .bind(&req.description)
    .bind(&req.runtime)
    .bind(&req.code)
    .bind(serde_json::to_value(&req.trigger_events).unwrap_or(serde_json::json!([])))
    .bind(&hook_timing)
    .bind(priority)
    .bind(req.timeout_ms.unwrap_or(30000))
    .bind(req.max_retries.unwrap_or(3))
    .bind(serde_json::to_value(&req.config).unwrap_or(serde_json::json!({})))
    .bind(serde_json::to_value(&req.environment_vars).unwrap_or(serde_json::json!({})))
    .bind(req.is_template.unwrap_or(false))
    .bind(&req.template_category)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": id,
                    "name": req.name,
                    "slug": slug
                }
            })).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to create function: {}", e)
                }))
            ).into_response()
        }
    }
}

/// Get a single function
async fn get_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result: Result<Function, _> = sqlx::query_as("SELECT * FROM functions WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(function) => Json(serde_json::json!({
            "success": true,
            "data": function
        })).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Function not found"
            }))
        ).into_response()
    }
}

/// Update a function
async fn update_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFunctionRequest>,
) -> impl IntoResponse {
    let mut updates = vec!["updated_at = NOW()".to_string()];

    if let Some(name) = &req.name {
        updates.push(format!("name = '{}'", name));
        updates.push(format!("slug = '{}'", name.to_lowercase().replace(' ', "-")));
    }
    if let Some(desc) = &req.description {
        updates.push(format!("description = '{}'", desc));
    }
    if let Some(runtime) = &req.runtime {
        updates.push(format!("runtime = '{}'", runtime));
    }
    if let Some(code) = &req.code {
        updates.push(format!("code = '{}'", code.replace('\'', "''")));
    }
    if let Some(events) = &req.trigger_events {
        updates.push(format!("trigger_events = '{}'", serde_json::to_string(events).unwrap_or_default()));
    }
    if let Some(hook_timing) = &req.hook_timing {
        updates.push(format!("hook_timing = '{}'", hook_timing));
    }
    if let Some(priority) = req.priority {
        updates.push(format!("priority = {}", priority));
    }
    if let Some(status) = &req.status {
        updates.push(format!("status = '{}'", status));
    }
    if let Some(timeout) = req.timeout_ms {
        updates.push(format!("timeout_ms = {}", timeout));
    }
    if let Some(retries) = req.max_retries {
        updates.push(format!("max_retries = {}", retries));
    }

    let sql = format!("UPDATE functions SET {} WHERE id = $1", updates.join(", "));

    match sqlx::query(&sql).bind(id).execute(&state.pool).await {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Function updated successfully"
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update function: {}", e)
            }))
        ).into_response()
    }
}

/// Delete a function
async fn delete_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM functions WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Function deleted successfully"
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete function: {}", e)
            }))
        ).into_response()
    }
}

/// Duplicate a function
async fn duplicate_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let original: Result<Function, _> = sqlx::query_as("SELECT * FROM functions WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match original {
        Ok(func) => {
            let new_id = Uuid::new_v4();
            let new_name = format!("{} (Copy)", func.name);
            let new_slug = format!("{}-copy-{}", func.slug, &new_id.to_string()[..8]);
            let now = Utc::now();

            let result = sqlx::query(
                r#"INSERT INTO functions (
                    id, name, slug, description, runtime, code, trigger_events, status,
                    timeout_ms, max_retries, retry_delay_ms, config, environment_vars,
                    is_template, template_category, execution_count, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'draft', $8, $9, $10, $11, $12, false, NULL, 0, $13, $13)"#
            )
            .bind(new_id)
            .bind(&new_name)
            .bind(&new_slug)
            .bind(&func.description)
            .bind(&func.runtime)
            .bind(&func.code)
            .bind(&func.trigger_events)
            .bind(func.timeout_ms)
            .bind(func.max_retries)
            .bind(func.retry_delay_ms)
            .bind(&func.config)
            .bind(&func.environment_vars)
            .bind(now)
            .execute(&state.pool)
            .await;

            match result {
                Ok(_) => Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "id": new_id,
                        "name": new_name,
                        "slug": new_slug
                    }
                })).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to duplicate function: {}", e)
                    }))
                ).into_response()
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Function not found"
            }))
        ).into_response()
    }
}

/// Toggle function status (active/inactive)
async fn toggle_function_status(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query(
        r#"UPDATE functions SET
            status = CASE WHEN status = 'active' THEN 'inactive' ELSE 'active' END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING status"#
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(row) => {
            let new_status: String = sqlx::Row::get(&row, "status");
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "status": new_status
                }
            })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to toggle status: {}", e)
            }))
        ).into_response()
    }
}

/// Execute a function manually
async fn execute_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecuteFunctionRequest>,
) -> impl IntoResponse {
    let function: Result<Function, _> = sqlx::query_as("SELECT * FROM functions WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match function {
        Ok(func) => {
            let execution_id = Uuid::new_v4();
            let started_at = Utc::now();

            // Log the execution start
            let _ = sqlx::query(
                r#"INSERT INTO function_executions (
                    id, function_id, trigger_event, event_data, status, started_at, retry_count
                ) VALUES ($1, $2, $3, $4, 'running', $5, 0)"#
            )
            .bind(execution_id)
            .bind(id)
            .bind(&req.event_type)
            .bind(&req.event_data)
            .bind(started_at)
            .execute(&state.pool)
            .await;

            // Execute the function (simplified - in production would use a proper runtime)
            let result = execute_function_code(&func, &req.event_data).await;
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds();

            match result {
                Ok(output) => {
                    // Update execution log with success
                    let _ = sqlx::query(
                        r#"UPDATE function_executions SET
                            status = 'success', completed_at = $1, duration_ms = $2, output = $3
                        WHERE id = $4"#
                    )
                    .bind(completed_at)
                    .bind(duration_ms)
                    .bind(&output)
                    .bind(execution_id)
                    .execute(&state.pool)
                    .await;

                    // Update function stats
                    let _ = sqlx::query(
                        r#"UPDATE functions SET
                            execution_count = execution_count + 1,
                            last_executed_at = $1,
                            last_error = NULL
                        WHERE id = $2"#
                    )
                    .bind(completed_at)
                    .bind(id)
                    .execute(&state.pool)
                    .await;

                    Json(serde_json::json!({
                        "success": true,
                        "data": {
                            "execution_id": execution_id,
                            "status": "success",
                            "duration_ms": duration_ms,
                            "output": output
                        }
                    })).into_response()
                }
                Err(error) => {
                    // Update execution log with error
                    let _ = sqlx::query(
                        r#"UPDATE function_executions SET
                            status = 'error', completed_at = $1, duration_ms = $2, error = $3
                        WHERE id = $4"#
                    )
                    .bind(completed_at)
                    .bind(duration_ms)
                    .bind(&error)
                    .bind(execution_id)
                    .execute(&state.pool)
                    .await;

                    // Update function stats with error
                    let _ = sqlx::query(
                        r#"UPDATE functions SET
                            execution_count = execution_count + 1,
                            last_executed_at = $1,
                            last_error = $2
                        WHERE id = $3"#
                    )
                    .bind(completed_at)
                    .bind(&error)
                    .bind(id)
                    .execute(&state.pool)
                    .await;

                    Json(serde_json::json!({
                        "success": false,
                        "data": {
                            "execution_id": execution_id,
                            "status": "error",
                            "duration_ms": duration_ms,
                            "error": error
                        }
                    })).into_response()
                }
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Function not found"
            }))
        ).into_response()
    }
}

/// Test a function without saving execution log
async fn test_function(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ExecuteFunctionRequest>,
) -> impl IntoResponse {
    let function: Result<Function, _> = sqlx::query_as("SELECT * FROM functions WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match function {
        Ok(func) => {
            let started_at = Utc::now();
            let result = execute_function_code(&func, &req.event_data).await;
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds();

            match result {
                Ok(output) => Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "status": "success",
                        "duration_ms": duration_ms,
                        "output": output
                    }
                })).into_response(),
                Err(error) => Json(serde_json::json!({
                    "success": false,
                    "data": {
                        "status": "error",
                        "duration_ms": duration_ms,
                        "error": error
                    }
                })).into_response()
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Function not found"
            }))
        ).into_response()
    }
}

/// Trigger an event (executes all functions listening to this event)
async fn trigger_event(
    State(state): State<FunctionsState>,
    Path(event): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Find all active functions listening to this event
    let functions: Vec<Function> = sqlx::query_as(
        r#"SELECT * FROM functions
        WHERE status = 'active'
        AND trigger_events @> $1::jsonb"#
    )
    .bind(serde_json::json!([event]))
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut results = Vec::new();

    for func in functions {
        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        // Log execution start
        let _ = sqlx::query(
            r#"INSERT INTO function_executions (
                id, function_id, trigger_event, event_data, status, started_at, retry_count
            ) VALUES ($1, $2, $3, $4, 'running', $5, 0)"#
        )
        .bind(execution_id)
        .bind(func.id)
        .bind(&event)
        .bind(&data)
        .bind(started_at)
        .execute(&state.pool)
        .await;

        let result = execute_function_code(&func, &data).await;
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds();

        let status = if result.is_ok() { "success" } else { "error" };

        // Update execution log
        let _ = sqlx::query(
            r#"UPDATE function_executions SET
                status = $1, completed_at = $2, duration_ms = $3,
                output = $4, error = $5
            WHERE id = $6"#
        )
        .bind(status)
        .bind(completed_at)
        .bind(duration_ms)
        .bind(result.as_ref().ok())
        .bind(result.as_ref().err())
        .bind(execution_id)
        .execute(&state.pool)
        .await;

        results.push(serde_json::json!({
            "function_id": func.id,
            "function_name": func.name,
            "execution_id": execution_id,
            "status": status,
            "duration_ms": duration_ms
        }));
    }

    Json(serde_json::json!({
        "success": true,
        "data": {
            "event": event,
            "functions_triggered": results.len(),
            "results": results
        }
    }))
}

/// List execution logs
async fn list_executions(
    State(state): State<FunctionsState>,
    Query(query): Query<ExecutionListQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut sql = String::from(
        "SELECT e.*, f.name as function_name FROM function_executions e
        LEFT JOIN functions f ON e.function_id = f.id WHERE 1=1"
    );

    if let Some(function_id) = query.function_id {
        sql.push_str(&format!(" AND e.function_id = '{}'", function_id));
    }
    if let Some(status) = &query.status {
        sql.push_str(&format!(" AND e.status = '{}'", status));
    }

    sql.push_str(" ORDER BY e.started_at DESC");
    sql.push_str(&format!(" LIMIT {} OFFSET {}", per_page, offset));

    let executions: Vec<serde_json::Value> = sqlx::query(&sql)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": sqlx::Row::get::<Uuid, _>(row, "id"),
                "function_id": sqlx::Row::get::<Uuid, _>(row, "function_id"),
                "function_name": sqlx::Row::get::<Option<String>, _>(row, "function_name"),
                "trigger_event": sqlx::Row::get::<String, _>(row, "trigger_event"),
                "status": sqlx::Row::get::<String, _>(row, "status"),
                "started_at": sqlx::Row::get::<DateTime<Utc>, _>(row, "started_at"),
                "completed_at": sqlx::Row::get::<Option<DateTime<Utc>>, _>(row, "completed_at"),
                "duration_ms": sqlx::Row::get::<Option<i64>, _>(row, "duration_ms"),
                "retry_count": sqlx::Row::get::<i32, _>(row, "retry_count")
            })
        })
        .collect();

    Json(serde_json::json!({
        "success": true,
        "data": executions,
        "pagination": {
            "page": page,
            "per_page": per_page
        }
    }))
}

/// Get a single execution
async fn get_execution(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result: Result<FunctionExecution, _> = sqlx::query_as(
        "SELECT * FROM function_executions WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(execution) => Json(serde_json::json!({
            "success": true,
            "data": execution
        })).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Execution not found"
            }))
        ).into_response()
    }
}

/// Retry a failed execution
async fn retry_execution(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let execution: Result<FunctionExecution, _> = sqlx::query_as(
        "SELECT * FROM function_executions WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await;

    match execution {
        Ok(exec) => {
            let function: Result<Function, _> = sqlx::query_as(
                "SELECT * FROM functions WHERE id = $1"
            )
            .bind(exec.function_id)
            .fetch_one(&state.pool)
            .await;

            match function {
                Ok(func) => {
                    let new_execution_id = Uuid::new_v4();
                    let started_at = Utc::now();

                    let _ = sqlx::query(
                        r#"INSERT INTO function_executions (
                            id, function_id, trigger_event, event_data, status, started_at, retry_count
                        ) VALUES ($1, $2, $3, $4, 'running', $5, $6)"#
                    )
                    .bind(new_execution_id)
                    .bind(func.id)
                    .bind(&exec.trigger_event)
                    .bind(&exec.event_data)
                    .bind(started_at)
                    .bind(exec.retry_count + 1)
                    .execute(&state.pool)
                    .await;

                    let result = execute_function_code(&func, &exec.event_data).await;
                    let completed_at = Utc::now();
                    let duration_ms = (completed_at - started_at).num_milliseconds();

                    let status = if result.is_ok() { "success" } else { "error" };

                    let _ = sqlx::query(
                        r#"UPDATE function_executions SET
                            status = $1, completed_at = $2, duration_ms = $3,
                            output = $4, error = $5
                        WHERE id = $6"#
                    )
                    .bind(status)
                    .bind(completed_at)
                    .bind(duration_ms)
                    .bind(result.as_ref().ok())
                    .bind(result.as_ref().err())
                    .bind(new_execution_id)
                    .execute(&state.pool)
                    .await;

                    Json(serde_json::json!({
                        "success": true,
                        "data": {
                            "execution_id": new_execution_id,
                            "status": status,
                            "duration_ms": duration_ms
                        }
                    })).into_response()
                }
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "Function not found"
                    }))
                ).into_response()
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Execution not found"
            }))
        ).into_response()
    }
}

/// Get function templates
async fn get_function_templates(
    State(state): State<FunctionsState>,
) -> impl IntoResponse {
    let templates: Vec<Function> = sqlx::query_as(
        "SELECT * FROM functions WHERE is_template = true ORDER BY template_category, name"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Group by category
    let mut categories: HashMap<String, Vec<&Function>> = HashMap::new();
    for template in &templates {
        let category = template.template_category.clone().unwrap_or_else(|| "General".to_string());
        categories.entry(category).or_insert_with(Vec::new).push(template);
    }

    Json(serde_json::json!({
        "success": true,
        "data": templates,
        "categories": categories.keys().collect::<Vec<_>>(),
        "total": templates.len()
    }))
}

/// Get a template by slug
async fn get_template_by_slug(
    State(state): State<FunctionsState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let template: Result<Function, _> = sqlx::query_as(
        "SELECT * FROM functions WHERE slug = $1 AND is_template = true"
    )
    .bind(&slug)
    .fetch_one(&state.pool)
    .await;

    match template {
        Ok(t) => Json(serde_json::json!({
            "success": true,
            "data": t
        })).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Template not found"
            }))
        ).into_response()
    }
}

/// Create a function from a template
async fn use_template(
    State(state): State<FunctionsState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let template: Result<Function, _> = sqlx::query_as(
        "SELECT * FROM functions WHERE slug = $1 AND is_template = true"
    )
    .bind(&slug)
    .fetch_one(&state.pool)
    .await;

    match template {
        Ok(t) => {
            let new_id = Uuid::new_v4();
            let new_name = format!("{} (from template)", t.name);
            let new_slug = format!("{}-{}", t.slug, &new_id.to_string()[..8]);
            let now = Utc::now();

            let result = sqlx::query(
                r#"INSERT INTO functions (
                    id, name, slug, description, runtime, code, trigger_events, status,
                    timeout_ms, max_retries, retry_delay_ms, config, environment_vars,
                    is_template, template_category, execution_count, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'draft', $8, $9, $10, $11, $12, false, NULL, 0, $13, $13)"#
            )
            .bind(new_id)
            .bind(&new_name)
            .bind(&new_slug)
            .bind(&t.description)
            .bind(&t.runtime)
            .bind(&t.code)
            .bind(&t.trigger_events)
            .bind(t.timeout_ms)
            .bind(t.max_retries)
            .bind(t.retry_delay_ms)
            .bind(&t.config)
            .bind(&t.environment_vars)
            .bind(now)
            .execute(&state.pool)
            .await;

            match result {
                Ok(_) => Json(serde_json::json!({
                    "success": true,
                    "data": {
                        "id": new_id,
                        "name": new_name,
                        "slug": new_slug
                    }
                })).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to create from template: {}", e)
                    }))
                ).into_response()
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Template not found"
            }))
        ).into_response()
    }
}

/// Get function statistics
async fn get_function_stats(
    State(state): State<FunctionsState>,
) -> impl IntoResponse {
    let total_functions: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM functions WHERE is_template = false")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let active_functions: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM functions WHERE status = 'active' AND is_template = false"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    let total_templates: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM functions WHERE is_template = true")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let total_executions: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM function_executions")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let successful_executions: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM function_executions WHERE status = 'success'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    let failed_executions: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM function_executions WHERE status = 'error'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0,));

    Json(serde_json::json!({
        "success": true,
        "data": {
            "total_functions": total_functions.0,
            "active_functions": active_functions.0,
            "inactive_functions": total_functions.0 - active_functions.0,
            "total_templates": total_templates.0,
            "total_executions": total_executions.0,
            "successful_executions": successful_executions.0,
            "failed_executions": failed_executions.0,
            "success_rate": if total_executions.0 > 0 {
                (successful_executions.0 as f64 / total_executions.0 as f64) * 100.0
            } else {
                0.0
            }
        }
    }))
}

/// Get execution statistics
async fn get_execution_stats(
    State(state): State<FunctionsState>,
) -> impl IntoResponse {
    // Executions by hour (last 24 hours)
    let hourly_stats: Vec<(DateTime<Utc>, i64, i64)> = sqlx::query_as(
        r#"SELECT
            date_trunc('hour', started_at) as hour,
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'success') as successful
        FROM function_executions
        WHERE started_at > NOW() - INTERVAL '24 hours'
        GROUP BY hour
        ORDER BY hour"#
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Top functions by execution count
    let top_functions: Vec<(Uuid, String, i64)> = sqlx::query_as(
        r#"SELECT f.id, f.name, f.execution_count
        FROM functions f
        WHERE is_template = false
        ORDER BY execution_count DESC
        LIMIT 10"#
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    // Average execution time
    let avg_duration: (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(duration_ms) FROM function_executions WHERE status = 'success'"
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or((None,));

    Json(serde_json::json!({
        "success": true,
        "data": {
            "hourly_stats": hourly_stats.iter().map(|(hour, total, successful)| {
                serde_json::json!({
                    "hour": hour,
                    "total": total,
                    "successful": successful
                })
            }).collect::<Vec<_>>(),
            "top_functions": top_functions.iter().map(|(id, name, count)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "execution_count": count
                })
            }).collect::<Vec<_>>(),
            "average_duration_ms": avg_duration.0.unwrap_or(0.0)
        }
    }))
}

/// List scheduled runs
async fn list_scheduled_runs(
    State(state): State<FunctionsState>,
) -> impl IntoResponse {
    let runs: Vec<ScheduledRun> = sqlx::query_as(
        "SELECT * FROM scheduled_runs ORDER BY scheduled_for"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Json(serde_json::json!({
        "success": true,
        "data": runs
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduledRunRequest {
    pub function_id: Uuid,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub cron_expression: Option<String>,
}

/// Create a scheduled run
async fn create_scheduled_run(
    State(state): State<FunctionsState>,
    Json(req): Json<CreateScheduledRunRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let scheduled_for = req.scheduled_for.unwrap_or_else(|| now + Duration::hours(1));

    let result = sqlx::query(
        r#"INSERT INTO scheduled_runs (
            id, function_id, scheduled_for, cron_expression, status, created_at
        ) VALUES ($1, $2, $3, $4, 'pending', $5)"#
    )
    .bind(id)
    .bind(req.function_id)
    .bind(scheduled_for)
    .bind(&req.cron_expression)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "data": {
                "id": id,
                "scheduled_for": scheduled_for
            }
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create scheduled run: {}", e)
            }))
        ).into_response()
    }
}

/// Delete a scheduled run
async fn delete_scheduled_run(
    State(state): State<FunctionsState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM scheduled_runs WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
    {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Scheduled run deleted"
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete: {}", e)
            }))
        ).into_response()
    }
}

// ============================================================================
// FUNCTION EXECUTION ENGINE (Simplified)
// ============================================================================

async fn execute_function_code(
    function: &Function,
    event_data: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match function.runtime.as_str() {
        "javascript" | "typescript" => {
            // In production, this would use a proper JS runtime like deno or quickjs
            Ok(serde_json::json!({
                "message": "JavaScript execution simulated",
                "input": event_data,
                "function": function.name
            }))
        }
        "rust" => {
            // Rust functions are compiled plugins or WASM modules
            // In production, this would load and execute a compiled Rust plugin
            // For now, we parse the code as a simple DSL or execute predefined functions
            execute_rust_function(function, event_data).await
        }
        "lua" => {
            // Lua scripting execution
            Ok(serde_json::json!({
                "message": "Lua execution simulated",
                "input": event_data,
                "function": function.name
            }))
        }
        "sql" => {
            // SQL execution would run against the database
            Ok(serde_json::json!({
                "message": "SQL execution simulated",
                "query": function.code
            }))
        }
        "http_webhook" => {
            // Would make HTTP request to configured URL
            Ok(serde_json::json!({
                "message": "Webhook execution simulated",
                "payload": event_data
            }))
        }
        "email" => {
            // Would send email
            Ok(serde_json::json!({
                "message": "Email execution simulated"
            }))
        }
        "slack" => {
            // Would send Slack message
            Ok(serde_json::json!({
                "message": "Slack notification simulated"
            }))
        }
        "discord" => {
            // Would send Discord message
            Ok(serde_json::json!({
                "message": "Discord notification simulated"
            }))
        }
        _ => Err(format!("Unknown runtime: {}", function.runtime)),
    }
}

/// Execute a Rust function
/// Rust functions can be:
/// 1. Predefined built-in functions (referenced by name)
/// 2. Simple expression DSL for data transformation
/// 3. WASM modules (future)
async fn execute_rust_function(
    function: &Function,
    event_data: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    // Parse the code to determine what type of Rust function this is
    let code = function.code.trim();

    // Check for built-in function references
    if code.starts_with("builtin::") {
        let builtin_name = &code[9..];
        return execute_builtin_function(builtin_name, event_data).await;
    }

    // Check for simple transformations
    if code.starts_with("transform::") {
        let transform = &code[11..];
        return execute_transform(transform, event_data);
    }

    // Check for validation rules
    if code.starts_with("validate::") {
        let validation = &code[10..];
        return execute_validation(validation, event_data);
    }

    // Default: treat as a simple expression or return success
    Ok(serde_json::json!({
        "message": "Rust function executed",
        "function": function.name,
        "input": event_data,
        "output": null
    }))
}

/// Execute a built-in Rust function
async fn execute_builtin_function(
    name: &str,
    event_data: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match name {
        // Content functions
        "generate_slug" => {
            let title = event_data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let slug = title.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            Ok(serde_json::json!({
                "slug": slug,
                "modified_data": { "slug": slug }
            }))
        }
        "calculate_reading_time" => {
            let content = event_data.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let word_count = content.split_whitespace().count();
            let reading_time = (word_count as f64 / 200.0).ceil() as u32;
            Ok(serde_json::json!({
                "word_count": word_count,
                "reading_time_minutes": reading_time,
                "reading_time_text": format!("{} min read", reading_time)
            }))
        }
        "generate_excerpt" => {
            let content = event_data.get("content").and_then(|v| v.as_str()).unwrap_or("");
            // Strip HTML tags (simple regex-free approach)
            let plain_text: String = content.chars()
                .fold((String::new(), false), |(mut acc, in_tag), c| {
                    if c == '<' { (acc, true) }
                    else if c == '>' { (acc, false) }
                    else if !in_tag { acc.push(c); (acc, false) }
                    else { (acc, true) }
                }).0;
            let excerpt: String = plain_text.chars().take(160).collect();
            let excerpt = if plain_text.len() > 160 {
                format!("{}...", excerpt.trim())
            } else {
                excerpt.trim().to_string()
            };
            Ok(serde_json::json!({
                "excerpt": excerpt,
                "modified_data": { "excerpt": excerpt }
            }))
        }
        // Validation functions
        "validate_not_empty" => {
            let title = event_data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            if title.trim().is_empty() {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": "Title cannot be empty"
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        "validate_slug_format" => {
            let slug = event_data.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let is_valid = slug.chars().all(|c| c.is_alphanumeric() || c == '-');
            if !is_valid {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": "Slug can only contain letters, numbers, and hyphens"
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        // Security functions
        "sanitize_html" => {
            let content = event_data.get("content").and_then(|v| v.as_str()).unwrap_or("");
            // Simple XSS prevention - remove script tags
            let sanitized = content
                .replace("<script", "&lt;script")
                .replace("</script>", "&lt;/script&gt;")
                .replace("javascript:", "")
                .replace("onerror=", "")
                .replace("onclick=", "");
            Ok(serde_json::json!({
                "sanitized": sanitized,
                "modified_data": { "content": sanitized }
            }))
        }
        "check_spam" => {
            let content = event_data.get("content").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let spam_keywords = ["viagra", "casino", "lottery", "winner", "click here", "free money"];
            let is_spam = spam_keywords.iter().any(|kw| content.contains(kw));
            if is_spam {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": "Content flagged as potential spam"
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        // Logging functions
        "log_event" => {
            // In production, this would write to a log file or logging service
            Ok(serde_json::json!({
                "logged": true,
                "event_data": event_data,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }
        _ => Err(format!("Unknown built-in function: {}", name)),
    }
}

/// Execute a simple data transformation
fn execute_transform(transform: &str, event_data: &serde_json::Value) -> Result<serde_json::Value, String> {
    match transform {
        "uppercase_title" => {
            let title = event_data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            Ok(serde_json::json!({
                "modified_data": { "title": title.to_uppercase() }
            }))
        }
        "lowercase_slug" => {
            let slug = event_data.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            Ok(serde_json::json!({
                "modified_data": { "slug": slug.to_lowercase() }
            }))
        }
        "trim_content" => {
            let content = event_data.get("content").and_then(|v| v.as_str()).unwrap_or("");
            Ok(serde_json::json!({
                "modified_data": { "content": content.trim() }
            }))
        }
        _ => Err(format!("Unknown transform: {}", transform)),
    }
}

/// Execute a validation rule
fn execute_validation(validation: &str, event_data: &serde_json::Value) -> Result<serde_json::Value, String> {
    let parts: Vec<&str> = validation.split(':').collect();
    if parts.len() < 2 {
        return Err("Invalid validation format. Use: field:rule".to_string());
    }

    let field = parts[0];
    let rule = parts[1];
    let value = event_data.get(field).and_then(|v| v.as_str()).unwrap_or("");

    match rule {
        "required" => {
            if value.trim().is_empty() {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": format!("{} is required", field)
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        "min_length" if parts.len() >= 3 => {
            let min_len: usize = parts[2].parse().unwrap_or(0);
            if value.len() < min_len {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": format!("{} must be at least {} characters", field, min_len)
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        "max_length" if parts.len() >= 3 => {
            let max_len: usize = parts[2].parse().unwrap_or(usize::MAX);
            if value.len() > max_len {
                Ok(serde_json::json!({
                    "proceed": false,
                    "reason": format!("{} must be at most {} characters", field, max_len)
                }))
            } else {
                Ok(serde_json::json!({ "proceed": true }))
            }
        }
        _ => Err(format!("Unknown validation rule: {}", rule)),
    }
}

// ============================================================================
// EVENT DISPATCHER - Integration Point for RustPress
// ============================================================================

/// Result of executing "before" hooks - can modify data or cancel the action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeforeHookResult {
    /// Whether to proceed with the action (false = cancel)
    pub proceed: bool,
    /// Modified event data (if any modifications were made)
    pub modified_data: Option<serde_json::Value>,
    /// Reason for cancellation (if proceed is false)
    pub cancel_reason: Option<String>,
    /// Results from each function execution
    pub executions: Vec<FunctionExecutionResult>,
}

/// Result of executing "after" hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfterHookResult {
    /// Results from each function execution
    pub executions: Vec<FunctionExecutionResult>,
}

/// Individual function execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExecutionResult {
    pub function_id: Uuid,
    pub function_name: String,
    pub status: String,
    pub duration_ms: i64,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Event Dispatcher - Call this from anywhere in RustPress to trigger functions
#[derive(Clone)]
pub struct EventDispatcher {
    pool: PgPool,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Dispatch an event and execute all matching functions
    ///
    /// This is the main entry point for triggering functions from anywhere in RustPress.
    ///
    /// # Arguments
    /// * `event_type` - The event type (e.g., "post_created", "user_registered")
    /// * `event_data` - The event data as JSON
    ///
    /// # Returns
    /// * `(BeforeHookResult, AfterHookResult)` - Results from before and after hooks
    ///
    /// # Example
    /// ```rust,ignore
    /// let dispatcher = EventDispatcher::new(pool.clone());
    ///
    /// // Before creating a post, run "before" hooks
    /// let (before_result, _) = dispatcher.dispatch_before("post_created", &post_data).await;
    ///
    /// if !before_result.proceed {
    ///     // A hook cancelled the action
    ///     return Err(before_result.cancel_reason.unwrap_or("Action cancelled by hook".into()));
    /// }
    ///
    /// // Use modified data if provided
    /// let final_data = before_result.modified_data.unwrap_or(post_data);
    ///
    /// // Create the post...
    /// create_post(final_data).await?;
    ///
    /// // After creating the post, run "after" hooks
    /// let after_result = dispatcher.dispatch_after("post_created", &final_data).await;
    /// ```
    pub async fn dispatch(&self, event_type: &str, event_data: &serde_json::Value) -> (BeforeHookResult, AfterHookResult) {
        let before_result = self.dispatch_before(event_type, event_data).await;
        let after_result = self.dispatch_after(event_type, event_data).await;
        (before_result, after_result)
    }

    /// Execute only "before" hooks for an event
    ///
    /// Call this BEFORE performing the action. Hooks can:
    /// - Modify the event data
    /// - Cancel the action by returning proceed=false
    pub async fn dispatch_before(&self, event_type: &str, event_data: &serde_json::Value) -> BeforeHookResult {
        let functions = self.get_functions_for_event(event_type, "before").await;

        let mut result = BeforeHookResult {
            proceed: true,
            modified_data: None,
            cancel_reason: None,
            executions: Vec::new(),
        };

        let mut current_data = event_data.clone();

        for func in functions {
            let execution_result = self.execute_and_log(&func, event_type, &current_data).await;

            // Check if this function wants to cancel the action
            if let Some(output) = &execution_result.output {
                if let Some(proceed) = output.get("proceed").and_then(|v| v.as_bool()) {
                    if !proceed {
                        result.proceed = false;
                        result.cancel_reason = output.get("reason")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| Some(format!("Cancelled by function: {}", func.name)));
                    }
                }

                // Check if function modified the data
                if let Some(modified) = output.get("modified_data") {
                    current_data = modified.clone();
                    result.modified_data = Some(modified.clone());
                }
            }

            result.executions.push(execution_result);

            // Stop if action was cancelled
            if !result.proceed {
                break;
            }
        }

        result
    }

    /// Execute only "after" hooks for an event
    ///
    /// Call this AFTER performing the action. These hooks are typically used for:
    /// - Sending notifications
    /// - Logging/auditing
    /// - Triggering external integrations
    pub async fn dispatch_after(&self, event_type: &str, event_data: &serde_json::Value) -> AfterHookResult {
        let functions = self.get_functions_for_event(event_type, "after").await;

        let mut result = AfterHookResult {
            executions: Vec::new(),
        };

        for func in functions {
            let execution_result = self.execute_and_log(&func, event_type, event_data).await;
            result.executions.push(execution_result);
        }

        result
    }

    /// Get all active functions that listen to an event with specific timing
    async fn get_functions_for_event(&self, event_type: &str, timing: &str) -> Vec<Function> {
        // Get functions that match the timing (before, after) or "both"
        let query = match timing {
            "before" => {
                r#"SELECT * FROM functions
                   WHERE status = 'active'
                   AND is_template = false
                   AND trigger_events @> $1::jsonb
                   AND (hook_timing = 'before' OR hook_timing = 'both')
                   ORDER BY priority ASC, created_at ASC"#
            }
            "after" => {
                r#"SELECT * FROM functions
                   WHERE status = 'active'
                   AND is_template = false
                   AND trigger_events @> $1::jsonb
                   AND (hook_timing = 'after' OR hook_timing = 'both')
                   ORDER BY priority ASC, created_at ASC"#
            }
            _ => return Vec::new(),
        };

        sqlx::query_as(query)
            .bind(serde_json::json!([event_type]))
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default()
    }

    /// Execute a function and log the execution
    async fn execute_and_log(&self, func: &Function, event_type: &str, event_data: &serde_json::Value) -> FunctionExecutionResult {
        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        // Log execution start
        let _ = sqlx::query(
            r#"INSERT INTO function_executions (
                id, function_id, trigger_event, event_data, status, started_at, retry_count
            ) VALUES ($1, $2, $3, $4, 'running', $5, 0)"#
        )
        .bind(execution_id)
        .bind(func.id)
        .bind(event_type)
        .bind(event_data)
        .bind(started_at)
        .execute(&self.pool)
        .await;

        // Execute the function
        let result = execute_function_code(func, event_data).await;
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds();

        let (status, output, error) = match &result {
            Ok(o) => ("success", Some(o.clone()), None),
            Err(e) => ("error", None, Some(e.clone())),
        };

        // Update execution log
        let _ = sqlx::query(
            r#"UPDATE function_executions SET
                status = $1, completed_at = $2, duration_ms = $3,
                output = $4, error = $5
            WHERE id = $6"#
        )
        .bind(status)
        .bind(completed_at)
        .bind(duration_ms)
        .bind(&output)
        .bind(&error)
        .bind(execution_id)
        .execute(&self.pool)
        .await;

        // Update function stats
        let _ = sqlx::query(
            r#"UPDATE functions SET
                execution_count = execution_count + 1,
                last_executed_at = $1,
                last_error = $2
            WHERE id = $3"#
        )
        .bind(completed_at)
        .bind(&error)
        .bind(func.id)
        .execute(&self.pool)
        .await;

        FunctionExecutionResult {
            function_id: func.id,
            function_name: func.name.clone(),
            status: status.to_string(),
            duration_ms,
            output,
            error,
        }
    }
}

/// Convenience function to create an event dispatcher
pub fn create_dispatcher(pool: PgPool) -> EventDispatcher {
    EventDispatcher::new(pool)
}

// ============================================================================
// DATABASE INITIALIZATION
// ============================================================================

pub async fn init_functions_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Functions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS functions (
            id UUID PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            slug VARCHAR(255) NOT NULL UNIQUE,
            description TEXT,
            runtime VARCHAR(50) NOT NULL,
            code TEXT NOT NULL,
            trigger_events JSONB NOT NULL DEFAULT '[]',
            status VARCHAR(20) NOT NULL DEFAULT 'draft',
            hook_timing VARCHAR(20) NOT NULL DEFAULT 'after',
            priority INTEGER NOT NULL DEFAULT 10,
            timeout_ms INTEGER NOT NULL DEFAULT 30000,
            max_retries INTEGER NOT NULL DEFAULT 3,
            retry_delay_ms INTEGER NOT NULL DEFAULT 1000,
            config JSONB NOT NULL DEFAULT '{}',
            environment_vars JSONB NOT NULL DEFAULT '{}',
            is_template BOOLEAN NOT NULL DEFAULT false,
            template_category VARCHAR(100),
            execution_count BIGINT NOT NULL DEFAULT 0,
            last_executed_at TIMESTAMPTZ,
            last_error TEXT,
            created_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Add hook_timing column if it doesn't exist (for migration)
    let _ = sqlx::query("ALTER TABLE functions ADD COLUMN IF NOT EXISTS hook_timing VARCHAR(20) NOT NULL DEFAULT 'after'")
        .execute(pool).await;
    let _ = sqlx::query("ALTER TABLE functions ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 10")
        .execute(pool).await;

    // Function executions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS function_executions (
            id UUID PRIMARY KEY,
            function_id UUID NOT NULL REFERENCES functions(id) ON DELETE CASCADE,
            trigger_event VARCHAR(100) NOT NULL,
            event_data JSONB NOT NULL DEFAULT '{}',
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            started_at TIMESTAMPTZ NOT NULL,
            completed_at TIMESTAMPTZ,
            duration_ms BIGINT,
            output JSONB,
            error TEXT,
            retry_count INTEGER NOT NULL DEFAULT 0
        )
    "#).execute(pool).await?;

    // Scheduled runs table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS scheduled_runs (
            id UUID PRIMARY KEY,
            function_id UUID NOT NULL REFERENCES functions(id) ON DELETE CASCADE,
            scheduled_for TIMESTAMPTZ NOT NULL,
            cron_expression VARCHAR(100),
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            last_run_at TIMESTAMPTZ,
            next_run_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_functions_status ON functions(status)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_functions_runtime ON functions(runtime)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_functions_is_template ON functions(is_template)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_functions_trigger_events ON functions USING GIN(trigger_events)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_function_executions_function_id ON function_executions(function_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_function_executions_status ON function_executions(status)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_function_executions_started_at ON function_executions(started_at)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_runs_function_id ON scheduled_runs(function_id)").execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_scheduled_runs_scheduled_for ON scheduled_runs(scheduled_for)").execute(pool).await?;

    // Insert default templates (200 ready-to-use functions)
    insert_default_templates(pool).await?;

    Ok(())
}

async fn insert_default_templates(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Check if templates already exist
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM functions WHERE is_template = true")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        return Ok(()); // Templates already inserted
    }

    let templates = get_default_templates();

    for template in templates {
        let id = Uuid::new_v4();
        let _ = sqlx::query(
            r#"INSERT INTO functions (
                id, name, slug, description, runtime, code, trigger_events, status,
                timeout_ms, max_retries, retry_delay_ms, config, environment_vars,
                is_template, template_category, execution_count, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, 3, 1000, '{}', '{}', true, $9, 0, NOW(), NOW())"#
        )
        .bind(id)
        .bind(&template.name)
        .bind(&template.slug)
        .bind(&template.description)
        .bind(&template.runtime)
        .bind(&template.code)
        .bind(serde_json::to_value(&template.trigger_events).unwrap_or(serde_json::json!([])))
        .bind(template.timeout_ms)
        .bind(&template.category)
        .execute(pool)
        .await;
    }

    Ok(())
}

struct TemplateData {
    name: String,
    slug: String,
    description: String,
    runtime: String,
    code: String,
    trigger_events: Vec<String>,
    timeout_ms: i32,
    category: String,
}

fn get_default_templates() -> Vec<TemplateData> {
    vec![
        // =====================================================================
        // CONTENT MANAGEMENT (1-30)
        // =====================================================================
        TemplateData {
            name: "Send Email on New Post".to_string(),
            slug: "send-email-on-new-post".to_string(),
            description: "Send notification email when a new post is published".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{admin_email}}", "subject": "New Post: {{post.title}}", "body": "A new post has been published: {{post.title}}\n\nView it at: {{post.url}}"}"#.to_string(),
            trigger_events: vec!["post_published".to_string()],
            timeout_ms: 10000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Slack Notification on Post".to_string(),
            slug: "slack-notify-new-post".to_string(),
            description: "Send Slack message when a new post is published".to_string(),
            runtime: "slack".to_string(),
            code: r##"{"channel": "#content", "text": "New post published: *{{post.title}}*\nAuthor: {{post.author}}\nLink: {{post.url}}"}"##.to_string(),
            trigger_events: vec!["post_published".to_string()],
            timeout_ms: 10000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Auto-Tweet New Posts".to_string(),
            slug: "auto-tweet-new-posts".to_string(),
            description: "Automatically tweet when a new post is published".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://api.twitter.com/2/tweets", "method": "POST", "body": {"text": "New post: {{post.title}} - {{post.url}} #blog"}}"#.to_string(),
            trigger_events: vec!["post_published".to_string()],
            timeout_ms: 15000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Generate Reading Time".to_string(),
            slug: "generate-reading-time".to_string(),
            description: "Calculate and store estimated reading time for posts".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const wordCount = event.post.content.split(/\s+/).length;
const readingTime = Math.ceil(wordCount / 200);
return { reading_time: readingTime + " min read", word_count: wordCount };
"#.to_string(),
            trigger_events: vec!["post_created".to_string(), "post_updated".to_string()],
            timeout_ms: 5000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "SEO Title Generator".to_string(),
            slug: "seo-title-generator".to_string(),
            description: "Generate SEO-optimized title suggestions".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const title = event.post.title;
const suggestions = [
    title + " | " + site.name,
    "Learn " + title + " - Complete Guide",
    title + " - Everything You Need to Know",
    "The Ultimate Guide to " + title
];
return { suggestions };
"#.to_string(),
            trigger_events: vec!["post_created".to_string()],
            timeout_ms: 5000,
            category: "SEO".to_string(),
        },
        TemplateData {
            name: "Auto-Generate Excerpt".to_string(),
            slug: "auto-generate-excerpt".to_string(),
            description: "Automatically generate post excerpt from content".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const content = event.post.content.replace(/<[^>]*>/g, '');
const excerpt = content.substring(0, 160).trim() + "...";
return { excerpt };
"#.to_string(),
            trigger_events: vec!["post_created".to_string()],
            timeout_ms: 5000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Notify Editor on Draft".to_string(),
            slug: "notify-editor-on-draft".to_string(),
            description: "Send notification to editor when draft is ready for review".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "editor@site.com", "subject": "Draft Ready for Review: {{post.title}}", "body": "A new draft is ready for your review.\n\nTitle: {{post.title}}\nAuthor: {{post.author}}\n\nPlease review at your earliest convenience."}"#.to_string(),
            trigger_events: vec!["post_status_changed".to_string()],
            timeout_ms: 10000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Archive Old Posts".to_string(),
            slug: "archive-old-posts".to_string(),
            description: "Automatically archive posts older than 2 years".to_string(),
            runtime: "sql".to_string(),
            code: r#"UPDATE posts SET status = 'archived' WHERE published_at < NOW() - INTERVAL '2 years' AND status = 'published'"#.to_string(),
            trigger_events: vec!["scheduler_monthly".to_string()],
            timeout_ms: 60000,
            category: "Content".to_string(),
        },
        TemplateData {
            name: "Ping Search Engines".to_string(),
            slug: "ping-search-engines".to_string(),
            description: "Notify search engines when new content is published".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://www.google.com/ping?sitemap={{site.url}}/sitemap.xml", "method": "GET"}"#.to_string(),
            trigger_events: vec!["post_published".to_string(), "page_published".to_string()],
            timeout_ms: 10000,
            category: "SEO".to_string(),
        },
        TemplateData {
            name: "Generate Social Images".to_string(),
            slug: "generate-social-images".to_string(),
            description: "Auto-generate Open Graph images for posts".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://og-image-service.com/generate", "method": "POST", "body": {"title": "{{post.title}}", "author": "{{post.author}}", "date": "{{post.date}}"}}"#.to_string(),
            trigger_events: vec!["post_published".to_string()],
            timeout_ms: 30000,
            category: "Content".to_string(),
        },

        // =====================================================================
        // USER MANAGEMENT (31-60)
        // =====================================================================
        TemplateData {
            name: "Welcome Email".to_string(),
            slug: "welcome-email".to_string(),
            description: "Send welcome email to new users".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Welcome to {{site.name}}!", "body": "Hi {{user.name}},\n\nWelcome to {{site.name}}! We're excited to have you.\n\nBest regards,\nThe {{site.name}} Team"}"#.to_string(),
            trigger_events: vec!["user_registered".to_string()],
            timeout_ms: 10000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Login Alert".to_string(),
            slug: "login-alert".to_string(),
            description: "Send security alert on new login".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "New Login to Your Account", "body": "A new login was detected:\n\nTime: {{event.timestamp}}\nIP: {{event.ip}}\nBrowser: {{event.user_agent}}\n\nIf this wasn't you, please change your password immediately."}"#.to_string(),
            trigger_events: vec!["user_logged_in".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Failed Login Counter".to_string(),
            slug: "failed-login-counter".to_string(),
            description: "Track failed login attempts per IP".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO login_attempts (ip, attempt_time) VALUES ('{{event.ip}}', NOW())"#.to_string(),
            trigger_events: vec!["login_attempt_failed".to_string()],
            timeout_ms: 5000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Block IP After Failed Logins".to_string(),
            slug: "block-ip-failed-logins".to_string(),
            description: "Automatically block IP after 5 failed login attempts".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const failedAttempts = await db.query(
    "SELECT COUNT(*) FROM login_attempts WHERE ip = $1 AND attempt_time > NOW() - INTERVAL '1 hour'",
    [event.ip]
);
if (failedAttempts > 5) {
    await db.query("INSERT INTO blocked_ips (ip, reason, blocked_at) VALUES ($1, 'Too many failed logins', NOW())", [event.ip]);
    return { blocked: true, ip: event.ip };
}
return { blocked: false };
"#.to_string(),
            trigger_events: vec!["login_attempt_failed".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "User Activity Logger".to_string(),
            slug: "user-activity-logger".to_string(),
            description: "Log all user activities to database".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO user_activity_log (user_id, action, details, ip, created_at) VALUES ('{{user.id}}', '{{event.type}}', '{{event.details}}', '{{event.ip}}', NOW())"#.to_string(),
            trigger_events: vec!["user_logged_in".to_string(), "user_logged_out".to_string(), "user_updated".to_string()],
            timeout_ms: 5000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Admin Alert on New User".to_string(),
            slug: "admin-alert-new-user".to_string(),
            description: "Notify admins when new users register".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "New User Registration", "body": "A new user has registered:\n\nName: {{user.name}}\nEmail: {{user.email}}\nRegistered: {{event.timestamp}}"}"#.to_string(),
            trigger_events: vec!["user_registered".to_string()],
            timeout_ms: 10000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Deactivation Notice".to_string(),
            slug: "deactivation-notice".to_string(),
            description: "Send email when account is deactivated".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Your Account Has Been Deactivated", "body": "Hi {{user.name}},\n\nYour account has been deactivated. If you believe this is an error, please contact support."}"#.to_string(),
            trigger_events: vec!["user_deactivated".to_string()],
            timeout_ms: 10000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Password Change Confirmation".to_string(),
            slug: "password-change-confirmation".to_string(),
            description: "Confirm password change via email".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Password Changed Successfully", "body": "Hi {{user.name}},\n\nYour password was changed on {{event.timestamp}}.\n\nIf you didn't make this change, please contact support immediately."}"#.to_string(),
            trigger_events: vec!["user_password_changed".to_string()],
            timeout_ms: 10000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Role Change Notification".to_string(),
            slug: "role-change-notification".to_string(),
            description: "Notify user when their role changes".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Your Role Has Been Updated", "body": "Hi {{user.name}},\n\nYour role has been changed from {{event.old_role}} to {{event.new_role}}.\n\nThis change was made by an administrator."}"#.to_string(),
            trigger_events: vec!["user_role_changed".to_string()],
            timeout_ms: 10000,
            category: "Users".to_string(),
        },
        TemplateData {
            name: "Inactive User Reminder".to_string(),
            slug: "inactive-user-reminder".to_string(),
            description: "Send reminder to users inactive for 30 days".to_string(),
            runtime: "sql".to_string(),
            code: r#"SELECT id, email, name FROM users WHERE last_login < NOW() - INTERVAL '30 days' AND status = 'active'"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 30000,
            category: "Users".to_string(),
        },

        // =====================================================================
        // COMMENTS & MODERATION (61-80)
        // =====================================================================
        TemplateData {
            name: "Notify Author on Comment".to_string(),
            slug: "notify-author-on-comment".to_string(),
            description: "Email post author when someone comments".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{post.author.email}}", "subject": "New Comment on: {{post.title}}", "body": "{{comment.author}} commented on your post:\n\n{{comment.content}}\n\nView: {{post.url}}#comment-{{comment.id}}"}"#.to_string(),
            trigger_events: vec!["comment_created".to_string()],
            timeout_ms: 10000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Comment Approval Notification".to_string(),
            slug: "comment-approval-notification".to_string(),
            description: "Notify commenter when their comment is approved".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{comment.author.email}}", "subject": "Your Comment Has Been Approved", "body": "Your comment on \"{{post.title}}\" has been approved and is now visible.\n\nView: {{post.url}}#comment-{{comment.id}}"}"#.to_string(),
            trigger_events: vec!["comment_approved".to_string()],
            timeout_ms: 10000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Spam Detection".to_string(),
            slug: "spam-detection".to_string(),
            description: "Check comments for spam patterns".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const spamPatterns = [/viagra/i, /casino/i, /loan/i, /https?:\/\/[^\s]+/g];
const content = event.comment.content;
const isSpam = spamPatterns.some(p => p.test(content));
if (isSpam) {
    return { spam: true, action: 'mark_as_spam' };
}
return { spam: false };
"#.to_string(),
            trigger_events: vec!["comment_created".to_string()],
            timeout_ms: 5000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Comment Reply Notification".to_string(),
            slug: "comment-reply-notification".to_string(),
            description: "Notify users when someone replies to their comment".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{parent_comment.author.email}}", "subject": "Someone Replied to Your Comment", "body": "{{comment.author}} replied to your comment:\n\n{{comment.content}}\n\nView: {{post.url}}#comment-{{comment.id}}"}"#.to_string(),
            trigger_events: vec!["comment_replied".to_string()],
            timeout_ms: 10000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Profanity Filter".to_string(),
            slug: "profanity-filter".to_string(),
            description: "Check comments for profanity and flag for review".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const badWords = ['badword1', 'badword2'];
const content = event.comment.content.toLowerCase();
const hasProfanity = badWords.some(w => content.includes(w));
return { flagged: hasProfanity, reason: hasProfanity ? 'profanity' : null };
"#.to_string(),
            trigger_events: vec!["comment_created".to_string()],
            timeout_ms: 5000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Daily Comment Digest".to_string(),
            slug: "daily-comment-digest".to_string(),
            description: "Send daily digest of comments to admin".to_string(),
            runtime: "sql".to_string(),
            code: r#"SELECT c.*, p.title as post_title FROM comments c JOIN posts p ON c.post_id = p.id WHERE c.created_at > NOW() - INTERVAL '24 hours' ORDER BY c.created_at DESC"#.to_string(),
            trigger_events: vec!["scheduler_daily_morning".to_string()],
            timeout_ms: 30000,
            category: "Comments".to_string(),
        },
        TemplateData {
            name: "Auto-Approve Trusted Users".to_string(),
            slug: "auto-approve-trusted".to_string(),
            description: "Auto-approve comments from users with 5+ approved comments".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const approvedCount = await db.query(
    "SELECT COUNT(*) FROM comments WHERE author_id = $1 AND status = 'approved'",
    [event.comment.author_id]
);
if (approvedCount >= 5) {
    return { auto_approve: true };
}
return { auto_approve: false };
"#.to_string(),
            trigger_events: vec!["comment_created".to_string()],
            timeout_ms: 5000,
            category: "Comments".to_string(),
        },

        // =====================================================================
        // MEDIA MANAGEMENT (81-100)
        // =====================================================================
        TemplateData {
            name: "Image Optimization".to_string(),
            slug: "image-optimization".to_string(),
            description: "Automatically optimize uploaded images".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://image-optimizer.com/api/optimize", "method": "POST", "body": {"image_url": "{{media.url}}", "quality": 85, "format": "webp"}}"#.to_string(),
            trigger_events: vec!["media_uploaded".to_string()],
            timeout_ms: 60000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Generate Thumbnails".to_string(),
            slug: "generate-thumbnails".to_string(),
            description: "Create thumbnail versions of uploaded images".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://image-service.com/resize", "method": "POST", "body": {"image": "{{media.url}}", "sizes": [150, 300, 600, 1200]}}"#.to_string(),
            trigger_events: vec!["media_uploaded".to_string()],
            timeout_ms: 60000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Alt Text Reminder".to_string(),
            slug: "alt-text-reminder".to_string(),
            description: "Remind users to add alt text to images".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Don't Forget Alt Text!", "body": "You uploaded an image without alt text. Adding descriptive alt text improves accessibility and SEO.\n\nImage: {{media.filename}}"}"#.to_string(),
            trigger_events: vec!["media_uploaded".to_string()],
            timeout_ms: 10000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Large File Alert".to_string(),
            slug: "large-file-alert".to_string(),
            description: "Alert admin when large files are uploaded".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const maxSize = 10 * 1024 * 1024; // 10MB
if (event.media.size > maxSize) {
    return { alert: true, size_mb: (event.media.size / 1024 / 1024).toFixed(2) };
}
return { alert: false };
"#.to_string(),
            trigger_events: vec!["media_uploaded".to_string()],
            timeout_ms: 5000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Cleanup Orphaned Media".to_string(),
            slug: "cleanup-orphaned-media".to_string(),
            description: "Delete media files not attached to any content".to_string(),
            runtime: "sql".to_string(),
            code: r#"DELETE FROM media WHERE id NOT IN (SELECT DISTINCT media_id FROM post_media) AND created_at < NOW() - INTERVAL '30 days'"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 120000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Video Transcoding".to_string(),
            slug: "video-transcoding".to_string(),
            description: "Automatically transcode uploaded videos".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://video-service.com/transcode", "method": "POST", "body": {"video_url": "{{media.url}}", "formats": ["mp4", "webm"], "qualities": ["720p", "1080p"]}}"#.to_string(),
            trigger_events: vec!["media_uploaded".to_string()],
            timeout_ms: 300000,
            category: "Media".to_string(),
        },
        TemplateData {
            name: "Media Usage Stats".to_string(),
            slug: "media-usage-stats".to_string(),
            description: "Track media file usage and generate reports".to_string(),
            runtime: "sql".to_string(),
            code: r#"SELECT m.id, m.filename, m.size, COUNT(pm.post_id) as usage_count FROM media m LEFT JOIN post_media pm ON m.id = pm.media_id GROUP BY m.id"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 60000,
            category: "Media".to_string(),
        },

        // =====================================================================
        // BACKUP & MAINTENANCE (101-120)
        // =====================================================================
        TemplateData {
            name: "Daily Database Backup".to_string(),
            slug: "daily-database-backup".to_string(),
            description: "Create daily database backup".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{site.backup_endpoint}}", "method": "POST", "body": {"type": "database", "format": "sql"}}"#.to_string(),
            trigger_events: vec!["scheduler_daily_midnight".to_string()],
            timeout_ms: 300000,
            category: "Backup".to_string(),
        },
        TemplateData {
            name: "Weekly Full Backup".to_string(),
            slug: "weekly-full-backup".to_string(),
            description: "Create weekly full site backup".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{site.backup_endpoint}}", "method": "POST", "body": {"type": "full", "include": ["database", "media", "themes", "plugins"]}}"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 600000,
            category: "Backup".to_string(),
        },
        TemplateData {
            name: "Backup Success Notification".to_string(),
            slug: "backup-success-notification".to_string(),
            description: "Notify admin when backup completes".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "Backup Completed Successfully", "body": "Backup completed:\n\nType: {{backup.type}}\nSize: {{backup.size}}\nDuration: {{backup.duration}}\nLocation: {{backup.url}}"}"#.to_string(),
            trigger_events: vec!["backup_completed".to_string()],
            timeout_ms: 10000,
            category: "Backup".to_string(),
        },
        TemplateData {
            name: "Backup Failure Alert".to_string(),
            slug: "backup-failure-alert".to_string(),
            description: "Alert admin when backup fails".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "URGENT: Backup Failed", "body": "A backup has failed:\n\nType: {{backup.type}}\nError: {{backup.error}}\nTime: {{event.timestamp}}\n\nPlease investigate immediately."}"#.to_string(),
            trigger_events: vec!["backup_failed".to_string()],
            timeout_ms: 10000,
            category: "Backup".to_string(),
        },
        TemplateData {
            name: "Clear Expired Sessions".to_string(),
            slug: "clear-expired-sessions".to_string(),
            description: "Remove expired user sessions".to_string(),
            runtime: "sql".to_string(),
            code: r#"DELETE FROM sessions WHERE expires_at < NOW()"#.to_string(),
            trigger_events: vec!["scheduler_hourly".to_string()],
            timeout_ms: 30000,
            category: "Maintenance".to_string(),
        },
        TemplateData {
            name: "Optimize Database Tables".to_string(),
            slug: "optimize-database-tables".to_string(),
            description: "Run VACUUM on database tables".to_string(),
            runtime: "sql".to_string(),
            code: r#"VACUUM ANALYZE"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 300000,
            category: "Maintenance".to_string(),
        },
        TemplateData {
            name: "Clear Old Logs".to_string(),
            slug: "clear-old-logs".to_string(),
            description: "Remove logs older than 90 days".to_string(),
            runtime: "sql".to_string(),
            code: r#"DELETE FROM activity_logs WHERE created_at < NOW() - INTERVAL '90 days'"#.to_string(),
            trigger_events: vec!["scheduler_weekly".to_string()],
            timeout_ms: 60000,
            category: "Maintenance".to_string(),
        },
        TemplateData {
            name: "Cleanup Temp Files".to_string(),
            slug: "cleanup-temp-files".to_string(),
            description: "Delete temporary files older than 24 hours".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{site.cleanup_endpoint}}", "method": "POST", "body": {"type": "temp_files", "older_than_hours": 24}}"#.to_string(),
            trigger_events: vec!["scheduler_daily".to_string()],
            timeout_ms: 60000,
            category: "Maintenance".to_string(),
        },
        TemplateData {
            name: "Cache Warmup".to_string(),
            slug: "cache-warmup".to_string(),
            description: "Warm cache after clearing".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{site.url}}/api/cache/warm", "method": "POST", "body": {"pages": ["home", "popular", "recent"]}}"#.to_string(),
            trigger_events: vec!["cache_cleared".to_string()],
            timeout_ms: 120000,
            category: "Performance".to_string(),
        },
        TemplateData {
            name: "Update Statistics".to_string(),
            slug: "update-statistics".to_string(),
            description: "Update aggregate statistics tables".to_string(),
            runtime: "sql".to_string(),
            code: r#"REFRESH MATERIALIZED VIEW posts_statistics; REFRESH MATERIALIZED VIEW user_statistics;"#.to_string(),
            trigger_events: vec!["scheduler_hourly".to_string()],
            timeout_ms: 60000,
            category: "Maintenance".to_string(),
        },

        // =====================================================================
        // ANALYTICS & REPORTING (121-140)
        // =====================================================================
        TemplateData {
            name: "Daily Traffic Report".to_string(),
            slug: "daily-traffic-report".to_string(),
            description: "Generate and email daily traffic report".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "Daily Traffic Report - {{date}}", "body": "Page Views: {{stats.page_views}}\nUnique Visitors: {{stats.unique_visitors}}\nBounce Rate: {{stats.bounce_rate}}%\nTop Pages:\n{{stats.top_pages}}"}"#.to_string(),
            trigger_events: vec!["scheduler_daily_morning".to_string()],
            timeout_ms: 30000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Weekly Performance Summary".to_string(),
            slug: "weekly-performance-summary".to_string(),
            description: "Send weekly performance summary".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "Weekly Performance Summary", "body": "This week's highlights:\n\nTotal Views: {{stats.total_views}}\nNew Users: {{stats.new_users}}\nNew Posts: {{stats.new_posts}}\nComments: {{stats.comments}}"}"#.to_string(),
            trigger_events: vec!["scheduler_weekly_monday".to_string()],
            timeout_ms: 30000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Track Page View".to_string(),
            slug: "track-page-view".to_string(),
            description: "Log page view to analytics table".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO page_views (page_url, user_id, session_id, ip, user_agent, referrer, created_at) VALUES ('{{event.url}}', '{{event.user_id}}', '{{event.session_id}}', '{{event.ip}}', '{{event.user_agent}}', '{{event.referrer}}', NOW())"#.to_string(),
            trigger_events: vec!["page_view_recorded".to_string()],
            timeout_ms: 5000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Conversion Tracking".to_string(),
            slug: "conversion-tracking".to_string(),
            description: "Track and log conversions".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO conversions (goal_id, user_id, session_id, value, created_at) VALUES ('{{event.goal_id}}', '{{event.user_id}}', '{{event.session_id}}', '{{event.value}}', NOW())"#.to_string(),
            trigger_events: vec!["conversion_recorded".to_string()],
            timeout_ms: 5000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Monthly Report Generator".to_string(),
            slug: "monthly-report-generator".to_string(),
            description: "Generate comprehensive monthly report".to_string(),
            runtime: "sql".to_string(),
            code: r#"SELECT date_trunc('day', created_at) as day, COUNT(*) as views FROM page_views WHERE created_at > NOW() - INTERVAL '30 days' GROUP BY day ORDER BY day"#.to_string(),
            trigger_events: vec!["scheduler_monthly_first".to_string()],
            timeout_ms: 120000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Traffic Spike Alert".to_string(),
            slug: "traffic-spike-alert".to_string(),
            description: "Alert when traffic exceeds threshold".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const currentHourViews = await db.query("SELECT COUNT(*) FROM page_views WHERE created_at > NOW() - INTERVAL '1 hour'");
const avgHourlyViews = await db.query("SELECT AVG(views) FROM hourly_stats WHERE hour > NOW() - INTERVAL '7 days'");
if (currentHourViews > avgHourlyViews * 3) {
    return { spike: true, current: currentHourViews, average: avgHourlyViews };
}
return { spike: false };
"#.to_string(),
            trigger_events: vec!["scheduler_hourly".to_string()],
            timeout_ms: 30000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Search Analytics".to_string(),
            slug: "search-analytics".to_string(),
            description: "Track internal search queries".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO search_queries (query, results_count, user_id, created_at) VALUES ('{{event.query}}', {{event.results_count}}, '{{event.user_id}}', NOW())"#.to_string(),
            trigger_events: vec!["search_performed".to_string()],
            timeout_ms: 5000,
            category: "Analytics".to_string(),
        },
        TemplateData {
            name: "Zero Results Alert".to_string(),
            slug: "zero-results-alert".to_string(),
            description: "Log searches with no results".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
if (event.results_count === 0) {
    await db.query("INSERT INTO zero_result_searches (query, created_at) VALUES ($1, NOW())", [event.query]);
    return { logged: true };
}
return { logged: false };
"#.to_string(),
            trigger_events: vec!["search_performed".to_string()],
            timeout_ms: 5000,
            category: "Analytics".to_string(),
        },

        // =====================================================================
        // SECURITY (141-160)
        // =====================================================================
        TemplateData {
            name: "Security Scan Alert".to_string(),
            slug: "security-scan-alert".to_string(),
            description: "Notify admin of security scan results".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "Security Scan Report", "body": "Security scan completed:\n\nFiles Scanned: {{scan.files_scanned}}\nIssues Found: {{scan.issues_count}}\nSeverity: {{scan.max_severity}}\n\nPlease review the full report in the admin dashboard."}"#.to_string(),
            trigger_events: vec!["security_scan_completed".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Vulnerability Alert".to_string(),
            slug: "vulnerability-alert".to_string(),
            description: "Immediate alert on vulnerability detection".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "CRITICAL: Vulnerability Detected", "body": "A vulnerability has been detected:\n\nType: {{vulnerability.type}}\nSeverity: {{vulnerability.severity}}\nLocation: {{vulnerability.location}}\nDescription: {{vulnerability.description}}\n\nPlease address this immediately."}"#.to_string(),
            trigger_events: vec!["vulnerability_found".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Brute Force Alert".to_string(),
            slug: "brute-force-alert".to_string(),
            description: "Alert on brute force attack detection".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "admin@site.com", "subject": "Brute Force Attack Detected", "body": "A brute force attack has been detected:\n\nIP Address: {{event.ip}}\nAttempts: {{event.attempts}}\nTarget: {{event.target}}\n\nThe IP has been automatically blocked."}"#.to_string(),
            trigger_events: vec!["brute_force_detected".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "File Integrity Monitor".to_string(),
            slug: "file-integrity-monitor".to_string(),
            description: "Check core files for unauthorized changes".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{site.security_endpoint}}/integrity-check", "method": "POST", "body": {"directories": ["core", "plugins", "themes"]}}"#.to_string(),
            trigger_events: vec!["scheduler_daily".to_string()],
            timeout_ms: 120000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "SQL Injection Logger".to_string(),
            slug: "sql-injection-logger".to_string(),
            description: "Log SQL injection attempts".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO security_events (event_type, ip, payload, user_agent, created_at) VALUES ('sql_injection', '{{event.ip}}', '{{event.payload}}', '{{event.user_agent}}', NOW())"#.to_string(),
            trigger_events: vec!["sql_injection_attempt".to_string()],
            timeout_ms: 5000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "XSS Attack Logger".to_string(),
            slug: "xss-attack-logger".to_string(),
            description: "Log XSS attack attempts".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO security_events (event_type, ip, payload, user_agent, created_at) VALUES ('xss', '{{event.ip}}', '{{event.payload}}', '{{event.user_agent}}', NOW())"#.to_string(),
            trigger_events: vec!["xss_attempt".to_string()],
            timeout_ms: 5000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Unblock IP After Time".to_string(),
            slug: "unblock-ip-after-time".to_string(),
            description: "Automatically unblock IPs after 24 hours".to_string(),
            runtime: "sql".to_string(),
            code: r#"DELETE FROM blocked_ips WHERE blocked_at < NOW() - INTERVAL '24 hours' AND permanent = false"#.to_string(),
            trigger_events: vec!["scheduler_hourly".to_string()],
            timeout_ms: 30000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Admin Action Audit".to_string(),
            slug: "admin-action-audit".to_string(),
            description: "Log all admin actions for audit".to_string(),
            runtime: "sql".to_string(),
            code: r#"INSERT INTO admin_audit_log (user_id, action, details, ip, created_at) VALUES ('{{user.id}}', '{{event.action}}', '{{event.details}}', '{{event.ip}}', NOW())"#.to_string(),
            trigger_events: vec!["settings_updated".to_string(), "user_role_changed".to_string(), "plugin_activated".to_string(), "theme_activated".to_string()],
            timeout_ms: 5000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "SSL Certificate Check".to_string(),
            slug: "ssl-certificate-check".to_string(),
            description: "Check SSL certificate expiration".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "https://ssl-check-service.com/check", "method": "POST", "body": {"domain": "{{site.domain}}"}}"#.to_string(),
            trigger_events: vec!["scheduler_daily".to_string()],
            timeout_ms: 30000,
            category: "Security".to_string(),
        },
        TemplateData {
            name: "Two-Factor Enabled Notification".to_string(),
            slug: "two-factor-enabled".to_string(),
            description: "Confirm when 2FA is enabled".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{user.email}}", "subject": "Two-Factor Authentication Enabled", "body": "Hi {{user.name}},\n\nTwo-factor authentication has been enabled on your account. Your account is now more secure.\n\nIf you didn't enable this, please contact support immediately."}"#.to_string(),
            trigger_events: vec!["user_2fa_enabled".to_string()],
            timeout_ms: 10000,
            category: "Security".to_string(),
        },

        // =====================================================================
        // NOTIFICATIONS & INTEGRATIONS (161-180)
        // =====================================================================
        TemplateData {
            name: "Discord New Post".to_string(),
            slug: "discord-new-post".to_string(),
            description: "Post to Discord when new content is published".to_string(),
            runtime: "discord".to_string(),
            code: r#"{"webhook_url": "{{config.discord_webhook}}", "embeds": [{"title": "{{post.title}}", "description": "{{post.excerpt}}", "url": "{{post.url}}", "color": 5814783}]}"#.to_string(),
            trigger_events: vec!["post_published".to_string()],
            timeout_ms: 10000,
            category: "Integrations".to_string(),
        },
        TemplateData {
            name: "Slack Error Alert".to_string(),
            slug: "slack-error-alert".to_string(),
            description: "Send Slack alert on system errors".to_string(),
            runtime: "slack".to_string(),
            code: r##"{"channel": "#alerts", "attachments": [{"color": "danger", "title": "System Error", "text": "{{error.message}}", "fields": [{"title": "File", "value": "{{error.file}}"}, {"title": "Line", "value": "{{error.line}}"}]}]}"##.to_string(),
            trigger_events: vec!["system_error".to_string()],
            timeout_ms: 10000,
            category: "Integrations".to_string(),
        },
        TemplateData {
            name: "Webhook Retry Handler".to_string(),
            slug: "webhook-retry-handler".to_string(),
            description: "Retry failed webhooks with exponential backoff".to_string(),
            runtime: "javascript".to_string(),
            code: r#"
const maxRetries = 5;
const baseDelay = 1000;
if (event.retry_count < maxRetries) {
    const delay = baseDelay * Math.pow(2, event.retry_count);
    return { retry: true, delay_ms: delay };
}
return { retry: false, give_up: true };
"#.to_string(),
            trigger_events: vec!["webhook_failed".to_string()],
            timeout_ms: 5000,
            category: "Integrations".to_string(),
        },
        TemplateData {
            name: "Zapier Trigger".to_string(),
            slug: "zapier-trigger".to_string(),
            description: "Send data to Zapier webhook".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{config.zapier_webhook}}", "method": "POST", "body": {"event": "{{event.type}}", "data": "{{event.data}}", "timestamp": "{{event.timestamp}}"}}"#.to_string(),
            trigger_events: vec!["post_published".to_string(), "user_registered".to_string(), "order_completed".to_string()],
            timeout_ms: 15000,
            category: "Integrations".to_string(),
        },
        TemplateData {
            name: "Microsoft Teams Alert".to_string(),
            slug: "teams-alert".to_string(),
            description: "Send alert to Microsoft Teams".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{config.teams_webhook}}", "method": "POST", "body": {"@type": "MessageCard", "summary": "{{event.title}}", "sections": [{"activityTitle": "{{event.title}}", "text": "{{event.message}}"}]}}"#.to_string(),
            trigger_events: vec!["system_warning".to_string(), "backup_failed".to_string()],
            timeout_ms: 10000,
            category: "Integrations".to_string(),
        },
        TemplateData {
            name: "Push Notification".to_string(),
            slug: "push-notification".to_string(),
            description: "Send push notification to mobile app".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{config.push_service}}/send", "method": "POST", "body": {"title": "{{notification.title}}", "body": "{{notification.body}}", "data": {"url": "{{notification.url}}"}}}"#.to_string(),
            trigger_events: vec!["notification_sent".to_string()],
            timeout_ms: 10000,
            category: "Notifications".to_string(),
        },
        TemplateData {
            name: "SMS Alert".to_string(),
            slug: "sms-alert".to_string(),
            description: "Send SMS for critical alerts".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{config.sms_api}}/send", "method": "POST", "body": {"to": "{{config.admin_phone}}", "message": "{{alert.message}}"}}"#.to_string(),
            trigger_events: vec!["brute_force_detected".to_string(), "vulnerability_found".to_string()],
            timeout_ms: 15000,
            category: "Notifications".to_string(),
        },

        // =====================================================================
        // E-COMMERCE (181-200)
        // =====================================================================
        TemplateData {
            name: "Order Confirmation Email".to_string(),
            slug: "order-confirmation-email".to_string(),
            description: "Send order confirmation to customer".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{order.customer.email}}", "subject": "Order Confirmed - #{{order.id}}", "body": "Thank you for your order!\n\nOrder #: {{order.id}}\nTotal: {{order.total}}\nItems: {{order.items_summary}}\n\nWe'll notify you when it ships."}"#.to_string(),
            trigger_events: vec!["order_created".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Low Stock Alert".to_string(),
            slug: "low-stock-alert".to_string(),
            description: "Alert when product stock is low".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "inventory@site.com", "subject": "Low Stock Alert: {{product.name}}", "body": "Product {{product.name}} is running low on stock.\n\nCurrent Stock: {{product.stock}}\nThreshold: {{product.low_stock_threshold}}\n\nPlease reorder soon."}"#.to_string(),
            trigger_events: vec!["product_out_of_stock".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Abandoned Cart Email".to_string(),
            slug: "abandoned-cart-email".to_string(),
            description: "Send reminder for abandoned carts".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{cart.customer.email}}", "subject": "You Left Something Behind!", "body": "Hi {{cart.customer.name}},\n\nYou have items waiting in your cart:\n\n{{cart.items_summary}}\n\nComplete your purchase: {{cart.checkout_url}}"}"#.to_string(),
            trigger_events: vec!["cart_abandoned".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Payment Failed Notification".to_string(),
            slug: "payment-failed-notification".to_string(),
            description: "Notify customer of failed payment".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{order.customer.email}}", "subject": "Payment Failed - Order #{{order.id}}", "body": "Hi {{order.customer.name}},\n\nYour payment for order #{{order.id}} failed.\n\nReason: {{payment.failure_reason}}\n\nPlease try again: {{order.payment_url}}"}"#.to_string(),
            trigger_events: vec!["payment_failed".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Refund Processed".to_string(),
            slug: "refund-processed".to_string(),
            description: "Notify customer when refund is processed".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{order.customer.email}}", "subject": "Refund Processed - Order #{{order.id}}", "body": "Hi {{order.customer.name}},\n\nYour refund of {{refund.amount}} has been processed.\n\nIt may take 5-10 business days to appear in your account."}"#.to_string(),
            trigger_events: vec!["order_refunded".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Back in Stock Alert".to_string(),
            slug: "back-in-stock-alert".to_string(),
            description: "Notify customers when product is back in stock".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{subscriber.email}}", "subject": "{{product.name}} is Back in Stock!", "body": "Great news!\n\n{{product.name}} is back in stock.\n\nGet it before it sells out again: {{product.url}}"}"#.to_string(),
            trigger_events: vec!["product_back_in_stock".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Daily Sales Report".to_string(),
            slug: "daily-sales-report".to_string(),
            description: "Generate daily sales summary".to_string(),
            runtime: "sql".to_string(),
            code: r#"SELECT COUNT(*) as orders, SUM(total) as revenue, AVG(total) as avg_order FROM orders WHERE created_at > NOW() - INTERVAL '24 hours'"#.to_string(),
            trigger_events: vec!["scheduler_daily_evening".to_string()],
            timeout_ms: 30000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Invoice Generator".to_string(),
            slug: "invoice-generator".to_string(),
            description: "Generate PDF invoice for orders".to_string(),
            runtime: "http_webhook".to_string(),
            code: r#"{"url": "{{config.invoice_service}}/generate", "method": "POST", "body": {"order_id": "{{order.id}}", "template": "default", "format": "pdf"}}"#.to_string(),
            trigger_events: vec!["order_completed".to_string()],
            timeout_ms: 30000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Coupon Usage Tracker".to_string(),
            slug: "coupon-usage-tracker".to_string(),
            description: "Track coupon usage and update limits".to_string(),
            runtime: "sql".to_string(),
            code: r#"UPDATE coupons SET usage_count = usage_count + 1, last_used_at = NOW() WHERE code = '{{coupon.code}}'"#.to_string(),
            trigger_events: vec!["coupon_used".to_string()],
            timeout_ms: 5000,
            category: "E-Commerce".to_string(),
        },
        TemplateData {
            name: "Shipping Notification".to_string(),
            slug: "shipping-notification".to_string(),
            description: "Notify customer when order ships".to_string(),
            runtime: "email".to_string(),
            code: r#"{"to": "{{order.customer.email}}", "subject": "Your Order Has Shipped! - #{{order.id}}", "body": "Hi {{order.customer.name}},\n\nGreat news! Your order is on its way.\n\nTracking Number: {{shipment.tracking_number}}\nCarrier: {{shipment.carrier}}\nEstimated Delivery: {{shipment.estimated_delivery}}\n\nTrack your package: {{shipment.tracking_url}}"}"#.to_string(),
            trigger_events: vec!["order_updated".to_string()],
            timeout_ms: 10000,
            category: "E-Commerce".to_string(),
        },
    ]
}
