//! # RustPress User Management
//!
//! Comprehensive user management system for RustPress CMS.
//!
//! This crate provides WordPress-compatible user management functionality:
//!
//! ## Registration & Authentication
//! - [`registration`] - Customizable user registration with field validation
//! - [`password`] - Password policies, validation, and hashing
//! - [`session`] - Session management and tracking
//!
//! ## Security
//! - [`two_factor`] - Two-factor authentication (TOTP, recovery codes)
//! - [`oauth`] - OAuth/SSO integration (Google, GitHub, etc.)
//! - [`audit`] - Audit logging for security events
//!
//! ## User Profiles
//! - [`profile`] - User profile editing and management
//! - [`avatar`] - Avatar support with Gravatar fallback
//!
//! ## Roles & Permissions
//! - [`roles`] - Role management with WordPress-compatible capabilities
//!
//! ## User Administration
//! - [`listing`] - User listing with search, filters, and bulk actions
//! - [`import_export`] - User import/export functionality
//!
//! ## Activity & Engagement
//! - [`activity`] - User activity tracking and login history
//! - [`notifications`] - User notification preferences
//! - [`dashboard`] - Personalized user dashboard with widgets
//!
//! ## Social Features
//! - [`social`] - Following, messaging, reputation, and badges
//!
//! ## Content Management
//! - [`ownership`] - Content ownership transfer and multi-author support
//!
//! ## Groups & Privacy
//! - [`groups`] - User groups, invitations, approval workflow, directory, privacy
//!
//! ## Compliance
//! - [`gdpr`] - GDPR compliance: data export and account deletion

// Registration & Authentication
pub mod registration;
pub mod password;
pub mod session;

// Security
pub mod two_factor;
pub mod oauth;
pub mod audit;

// User Profiles
pub mod avatar;
pub mod profile;

// Roles & Permissions
pub mod roles;

// User Administration
pub mod import_export;
pub mod listing;

// Activity & Engagement
pub mod activity;
pub mod dashboard;
pub mod notifications;

// Social Features
pub mod social;

// Content Management
pub mod ownership;

// Groups & Privacy
pub mod groups;

// Compliance
pub mod gdpr;

// Re-export commonly used types
pub use activity::{
    Activity, ActivityCategory, ActivityManager, ActivityQuery, ActivityType,
    LoginHistory, LoginRecord, SuspiciousActivity,
};

pub use avatar::{
    Avatar, AvatarManager, AvatarSettings, AvatarType, GravatarDefault, GravatarRating,
};

pub use dashboard::{
    DashboardLayout, DashboardManager, DashboardNotification, DashboardStats, DashboardWidget,
    NotificationLevel, UserDashboardStats, WidgetType,
};

pub use gdpr::{
    Consent, ConsentManager, ConsentType, DataExportRequest, DeletionManager, DeletionRequest,
    DeletionStatus, PersonalDataExport, RetentionPolicy,
};

pub use groups::{
    ApprovalManager, ApprovalStatus, GroupManager, GroupRole, Invitation, InvitationManager,
    InvitationStatus, PendingUser, PrivacyManager, PrivacySettings, UserGroup,
};

pub use import_export::{
    ExportConfig, ExportFormat, ExportableUser, ImportConfig, ImportResult, ImportableUser,
    UserExporter, UserImporter,
};

pub use listing::{
    BulkAction, BulkActionRequest, BulkActionResult, BulkExecutor, ListColumn, UserListItem,
    UserListQuery, UserListResult, UserOrderBy, UserSearch, UserStatus,
};

pub use notifications::{
    NotificationChannel, NotificationFrequency, NotificationPreference,
    NotificationPreferencesManager, NotificationType, UserNotificationPreferences,
};

pub use ownership::{
    AuthorRole, BulkTransfer, Contribution, ContributionType, MultiAuthorManager,
    OwnershipTransfer, PostAuthor, TransferStatus,
};

pub use profile::{
    EmailChangeRequest, PasswordChangeRequest, ProfileChangeHistory, ProfileField,
    ProfileFieldType, ProfileManager, ProfileSection, ProfileUpdate, ProfileVisibility,
    SocialLinks, UserProfile,
};

pub use registration::{
    FieldValidation, FieldVisibility, RegistrationField, RegistrationFieldType,
    RegistrationForm, RegistrationManager, RegistrationSubmission, RegistrationValidator,
    SubmissionStatus, ValidationError,
};

pub use roles::{
    Capability, Role, RoleManager, UserRole,
};

pub use social::{
    AuthorArchiveSettings, AuthorMeta, AuthorPage, AuthorStats, Badge, BadgeManager, BadgeTier,
    EarnedBadge, Follow, FollowManager, InboxManager, Message, MessageThread, ReputationAction,
    ReputationLevel, Subscription, SubscriptionType, UserReputation,
};

// New security modules
pub use password::{
    PasswordPolicy, PasswordValidator, PasswordValidationResult, PasswordStrength,
    PasswordError, PasswordHistoryManager, PasswordExpiration,
    hash_password, verify_password, generate_password,
};

pub use session::{
    Session, SessionToken, SessionManager, SessionSettings, DeviceInfo,
};

pub use two_factor::{
    TwoFactorManager, TwoFactorStatus, TwoFactorMethod, TotpSetup,
    TrustedDevice, VerificationResult,
};

pub use oauth::{
    OAuthManager, OAuthProvider, OAuthProviderConfig, OAuthState, OAuthTokens,
    OAuthUserProfile, OAuthSettings, LinkedAccount,
};

pub use audit::{
    AuditManager, AuditEntry, AuditAction, AuditCategory, AuditSeverity,
    AuditQuery, AuditStats, RetentionPolicy as AuditRetentionPolicy,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::activity::{Activity, ActivityManager, LoginHistory};
    pub use crate::avatar::{Avatar, AvatarManager};
    pub use crate::dashboard::{DashboardLayout, DashboardManager};
    pub use crate::gdpr::{ConsentManager, DeletionManager};
    pub use crate::groups::{ApprovalManager, GroupManager, InvitationManager, PrivacyManager};
    pub use crate::import_export::{UserExporter, UserImporter};
    pub use crate::listing::{BulkExecutor, UserListQuery};
    pub use crate::notifications::NotificationPreferencesManager;
    pub use crate::ownership::MultiAuthorManager;
    pub use crate::profile::{ProfileManager, UserProfile};
    pub use crate::registration::{RegistrationForm, RegistrationManager};
    pub use crate::roles::RoleManager;
    pub use crate::social::{BadgeManager, FollowManager, InboxManager};

    // Security modules
    pub use crate::password::{PasswordPolicy, PasswordValidator};
    pub use crate::session::{SessionManager, Session};
    pub use crate::two_factor::{TwoFactorManager, TwoFactorStatus};
    pub use crate::oauth::{OAuthManager, OAuthProvider};
    pub use crate::audit::{AuditManager, AuditEntry, AuditAction};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Test that all modules can be imported
        let _ = RegistrationForm::default();
        let _ = UserProfile::new(1, "test", "test@example.com");
        let _ = Avatar::new(1, "test@example.com");
        let _ = RoleManager::new();
        let _ = UserListQuery::new();
        let _ = ActivityManager::new();
        let _ = NotificationPreferencesManager::new();
        let _ = DashboardManager::new();
        let _ = FollowManager::new();
        let _ = BadgeManager::new();
        let _ = MultiAuthorManager::new();
        let _ = GroupManager::new();
        let _ = InvitationManager::new();
        let _ = ApprovalManager::new();
        let _ = PrivacyManager::new();
        let _ = DeletionManager::new();
        let _ = ConsentManager::new();
    }

    #[test]
    fn test_prelude() {
        use crate::prelude::*;

        // Verify prelude exports work
        let _ = RegistrationManager::new();
        let _ = ProfileManager::new();
        let _ = RoleManager::new();
    }
}
