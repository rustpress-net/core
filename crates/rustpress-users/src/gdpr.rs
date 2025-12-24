//! # GDPR Compliance & Account Deletion
//!
//! User data export and account deletion for GDPR compliance.
//!
//! Features:
//! - Personal data export
//! - Account deletion workflow
//! - Data retention policies
//! - Consent management
//! - Audit logging

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Data Export
// ============================================================================

/// Personal data export request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub id: Uuid,
    pub user_id: i64,
    pub requested_at: DateTime<Utc>,
    pub status: ExportStatus,
    pub download_token: Option<String>,
    pub download_expires: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
    pub completed_at: Option<DateTime<Utc>>,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub ip_address: String,
}

/// Export status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportStatus {
    Pending,
    Processing,
    Ready,
    Downloaded,
    Expired,
    Failed,
}

impl DataExportRequest {
    pub fn new(user_id: i64, ip_address: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            requested_at: Utc::now(),
            status: ExportStatus::Pending,
            download_token: None,
            download_expires: None,
            file_path: None,
            file_size: None,
            completed_at: None,
            downloaded_at: None,
            ip_address: ip_address.to_string(),
        }
    }

    pub fn mark_ready(&mut self, file_path: &str, file_size: u64) {
        self.status = ExportStatus::Ready;
        self.file_path = Some(file_path.to_string());
        self.file_size = Some(file_size);
        self.completed_at = Some(Utc::now());
        self.download_token = Some(Uuid::new_v4().to_string());
        self.download_expires = Some(Utc::now() + Duration::days(3));
    }

    pub fn mark_downloaded(&mut self) {
        self.status = ExportStatus::Downloaded;
        self.downloaded_at = Some(Utc::now());
    }

    pub fn is_download_valid(&self) -> bool {
        self.status == ExportStatus::Ready &&
        self.download_expires.map(|e| Utc::now() < e).unwrap_or(false)
    }

    pub fn download_url(&self, base_url: &str) -> Option<String> {
        self.download_token.as_ref().map(|token| {
            format!("{}/user/export/download?token={}", base_url, token)
        })
    }
}

/// Exported personal data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalDataExport {
    pub export_date: DateTime<Utc>,
    pub user_id: i64,
    pub account_info: AccountInfo,
    pub profile_data: ProfileData,
    pub content: ContentData,
    pub comments: Vec<CommentData>,
    pub activity: Vec<ActivityData>,
    pub settings: SettingsData,
    pub media: Vec<MediaData>,
    pub messages: Vec<MessageData>,
    pub meta: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub username: String,
    pub email: String,
    pub registered_date: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub roles: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: String,
    pub bio: String,
    pub website: String,
    pub social_links: HashMap<String, String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentData {
    pub posts: Vec<PostExport>,
    pub pages: Vec<PostExport>,
    pub drafts: Vec<PostExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostExport {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentData {
    pub id: i64,
    pub post_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    pub activity_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsData {
    pub notification_preferences: HashMap<String, bool>,
    pub privacy_settings: HashMap<String, String>,
    pub ui_preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaData {
    pub id: i64,
    pub filename: String,
    pub url: String,
    pub uploaded_at: DateTime<Utc>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub id: String,
    pub with_user: String,
    pub content: String,
    pub sent_at: DateTime<Utc>,
    pub direction: String,
}

impl PersonalDataExport {
    pub fn new(user_id: i64) -> Self {
        Self {
            export_date: Utc::now(),
            user_id,
            account_info: AccountInfo {
                username: String::new(),
                email: String::new(),
                registered_date: Utc::now(),
                last_login: None,
                roles: Vec::new(),
                status: String::new(),
            },
            profile_data: ProfileData {
                display_name: String::new(),
                first_name: String::new(),
                last_name: String::new(),
                nickname: String::new(),
                bio: String::new(),
                website: String::new(),
                social_links: HashMap::new(),
                avatar_url: None,
            },
            content: ContentData {
                posts: Vec::new(),
                pages: Vec::new(),
                drafts: Vec::new(),
            },
            comments: Vec::new(),
            activity: Vec::new(),
            settings: SettingsData {
                notification_preferences: HashMap::new(),
                privacy_settings: HashMap::new(),
                ui_preferences: HashMap::new(),
            },
            media: Vec::new(),
            messages: Vec::new(),
            meta: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

// ============================================================================
// Account Deletion
// ============================================================================

/// Account deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub id: Uuid,
    pub user_id: i64,
    pub reason: Option<String>,
    pub feedback: Option<String>,
    pub status: DeletionStatus,
    pub requested_at: DateTime<Utc>,
    pub confirmation_token: String,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub scheduled_deletion: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub ip_address: String,
    pub content_action: ContentDeletionAction,
    pub reassign_to: Option<i64>,
}

/// Deletion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionStatus {
    Pending,
    Confirmed,
    Scheduled,
    Processing,
    Completed,
    Cancelled,
}

/// What to do with user's content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentDeletionAction {
    Delete,
    Anonymize,
    Reassign,
}

impl DeletionRequest {
    pub fn new(user_id: i64, ip_address: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            reason: None,
            feedback: None,
            status: DeletionStatus::Pending,
            requested_at: Utc::now(),
            confirmation_token: Uuid::new_v4().to_string(),
            confirmed_at: None,
            scheduled_deletion: None,
            completed_at: None,
            ip_address: ip_address.to_string(),
            content_action: ContentDeletionAction::Anonymize,
            reassign_to: None,
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    pub fn with_feedback(mut self, feedback: &str) -> Self {
        self.feedback = Some(feedback.to_string());
        self
    }

    pub fn confirm(&mut self, grace_period_days: i64) {
        self.status = DeletionStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
        self.scheduled_deletion = Some(Utc::now() + Duration::days(grace_period_days));
        self.status = DeletionStatus::Scheduled;
    }

    pub fn cancel(&mut self) {
        self.status = DeletionStatus::Cancelled;
    }

    pub fn complete(&mut self) {
        self.status = DeletionStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn is_ready_for_deletion(&self) -> bool {
        self.status == DeletionStatus::Scheduled &&
        self.scheduled_deletion.map(|d| Utc::now() >= d).unwrap_or(false)
    }

    pub fn confirmation_url(&self, base_url: &str) -> String {
        format!("{}/account/delete/confirm?token={}", base_url, self.confirmation_token)
    }
}

/// Deletion checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionChecklistItem {
    pub id: String,
    pub label: String,
    pub data_type: String,
    pub count: u32,
    pub action: ContentDeletionAction,
    pub completed: bool,
}

/// Account deletion manager
pub struct DeletionManager {
    requests: HashMap<Uuid, DeletionRequest>,
    grace_period_days: i64,
    allow_immediate_deletion: bool,
}

impl Default for DeletionManager {
    fn default() -> Self {
        Self {
            requests: HashMap::new(),
            grace_period_days: 14,
            allow_immediate_deletion: false,
        }
    }
}

impl DeletionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grace_period(mut self, days: i64) -> Self {
        self.grace_period_days = days;
        self
    }

    /// Request account deletion
    pub fn request(&mut self, request: DeletionRequest) -> &DeletionRequest {
        let id = request.id;
        self.requests.insert(id, request);
        self.requests.get(&id).unwrap()
    }

    /// Confirm deletion
    pub fn confirm(&mut self, token: &str) -> Result<&DeletionRequest, String> {
        let request = self.requests.values_mut()
            .find(|r| r.confirmation_token == token && r.status == DeletionStatus::Pending)
            .ok_or_else(|| "Invalid or expired confirmation token".to_string())?;

        request.confirm(self.grace_period_days);

        let id = request.id;
        Ok(self.requests.get(&id).unwrap())
    }

    /// Cancel deletion
    pub fn cancel(&mut self, request_id: Uuid) -> Result<(), String> {
        let request = self.requests.get_mut(&request_id)
            .ok_or_else(|| "Request not found".to_string())?;

        if matches!(request.status, DeletionStatus::Completed | DeletionStatus::Processing) {
            return Err("Cannot cancel deletion at this stage".to_string());
        }

        request.cancel();
        Ok(())
    }

    /// Get pending request for user
    pub fn get_pending(&self, user_id: i64) -> Option<&DeletionRequest> {
        self.requests.values()
            .find(|r| r.user_id == user_id && !matches!(r.status, DeletionStatus::Completed | DeletionStatus::Cancelled))
    }

    /// Get requests ready for deletion
    pub fn get_ready_for_deletion(&self) -> Vec<&DeletionRequest> {
        self.requests.values()
            .filter(|r| r.is_ready_for_deletion())
            .collect()
    }

    /// Process deletion
    pub fn process(&mut self, request_id: Uuid) -> Result<&DeletionRequest, String> {
        let request = self.requests.get_mut(&request_id)
            .ok_or_else(|| "Request not found".to_string())?;

        if !request.is_ready_for_deletion() {
            return Err("Request is not ready for deletion".to_string());
        }

        request.status = DeletionStatus::Processing;

        let id = request.id;
        Ok(self.requests.get(&id).unwrap())
    }

    /// Complete deletion
    pub fn complete(&mut self, request_id: Uuid) {
        if let Some(request) = self.requests.get_mut(&request_id) {
            request.complete();
        }
    }

    /// Generate deletion checklist
    pub fn generate_checklist(&self, user_id: i64) -> Vec<DeletionChecklistItem> {
        vec![
            DeletionChecklistItem {
                id: "account".to_string(),
                label: "User account".to_string(),
                data_type: "account".to_string(),
                count: 1,
                action: ContentDeletionAction::Delete,
                completed: false,
            },
            DeletionChecklistItem {
                id: "profile".to_string(),
                label: "Profile information".to_string(),
                data_type: "profile".to_string(),
                count: 1,
                action: ContentDeletionAction::Delete,
                completed: false,
            },
            DeletionChecklistItem {
                id: "posts".to_string(),
                label: "Posts and pages".to_string(),
                data_type: "content".to_string(),
                count: 0, // Should be populated with actual count
                action: ContentDeletionAction::Anonymize,
                completed: false,
            },
            DeletionChecklistItem {
                id: "comments".to_string(),
                label: "Comments".to_string(),
                data_type: "comments".to_string(),
                count: 0,
                action: ContentDeletionAction::Anonymize,
                completed: false,
            },
            DeletionChecklistItem {
                id: "media".to_string(),
                label: "Uploaded media".to_string(),
                data_type: "media".to_string(),
                count: 0,
                action: ContentDeletionAction::Delete,
                completed: false,
            },
            DeletionChecklistItem {
                id: "messages".to_string(),
                label: "Private messages".to_string(),
                data_type: "messages".to_string(),
                count: 0,
                action: ContentDeletionAction::Delete,
                completed: false,
            },
            DeletionChecklistItem {
                id: "activity".to_string(),
                label: "Activity logs".to_string(),
                data_type: "activity".to_string(),
                count: 0,
                action: ContentDeletionAction::Delete,
                completed: false,
            },
        ]
    }
}

// ============================================================================
// Consent Management
// ============================================================================

/// User consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consent {
    pub id: Uuid,
    pub user_id: i64,
    pub consent_type: ConsentType,
    pub granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub ip_address: String,
    pub version: String,
}

/// Consent types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsentType {
    PrivacyPolicy,
    TermsOfService,
    Marketing,
    Analytics,
    ThirdPartySharing,
    Custom(String),
}

impl ConsentType {
    pub fn label(&self) -> &str {
        match self {
            Self::PrivacyPolicy => "Privacy Policy",
            Self::TermsOfService => "Terms of Service",
            Self::Marketing => "Marketing Communications",
            Self::Analytics => "Analytics & Tracking",
            Self::ThirdPartySharing => "Third-Party Data Sharing",
            Self::Custom(s) => s,
        }
    }

    pub fn required(&self) -> bool {
        matches!(self, Self::PrivacyPolicy | Self::TermsOfService)
    }
}

impl Consent {
    pub fn grant(user_id: i64, consent_type: ConsentType, ip: &str, version: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            consent_type,
            granted: true,
            granted_at: Some(Utc::now()),
            revoked_at: None,
            ip_address: ip.to_string(),
            version: version.to_string(),
        }
    }

    pub fn revoke(&mut self) {
        self.granted = false;
        self.revoked_at = Some(Utc::now());
    }
}

/// Consent manager
pub struct ConsentManager {
    consents: HashMap<i64, Vec<Consent>>,
    policy_versions: HashMap<ConsentType, String>,
}

impl Default for ConsentManager {
    fn default() -> Self {
        let mut versions = HashMap::new();
        versions.insert(ConsentType::PrivacyPolicy, "1.0".to_string());
        versions.insert(ConsentType::TermsOfService, "1.0".to_string());

        Self {
            consents: HashMap::new(),
            policy_versions: versions,
        }
    }
}

impl ConsentManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record consent
    pub fn record(&mut self, consent: Consent) {
        self.consents.entry(consent.user_id)
            .or_insert_with(Vec::new)
            .push(consent);
    }

    /// Check if user has valid consent
    pub fn has_consent(&self, user_id: i64, consent_type: &ConsentType) -> bool {
        let current_version = self.policy_versions.get(consent_type);

        self.consents.get(&user_id)
            .map(|consents| {
                consents.iter()
                    .filter(|c| &c.consent_type == consent_type && c.granted)
                    .any(|c| current_version.map(|v| &c.version == v).unwrap_or(true))
            })
            .unwrap_or(false)
    }

    /// Get all consents for user
    pub fn get_user_consents(&self, user_id: i64) -> Vec<&Consent> {
        self.consents.get(&user_id)
            .map(|c| c.iter().collect())
            .unwrap_or_default()
    }

    /// Revoke consent
    pub fn revoke(&mut self, user_id: i64, consent_type: &ConsentType) {
        if let Some(consents) = self.consents.get_mut(&user_id) {
            for consent in consents.iter_mut() {
                if &consent.consent_type == consent_type && consent.granted {
                    consent.revoke();
                }
            }
        }
    }

    /// Update policy version (requires re-consent)
    pub fn update_policy_version(&mut self, consent_type: ConsentType, version: &str) {
        self.policy_versions.insert(consent_type, version.to_string());
    }

    /// Get users needing re-consent
    pub fn get_users_needing_reconsent(&self, consent_type: &ConsentType) -> Vec<i64> {
        let current_version = match self.policy_versions.get(consent_type) {
            Some(v) => v,
            None => return Vec::new(),
        };

        self.consents.iter()
            .filter(|(_, consents)| {
                let latest = consents.iter()
                    .filter(|c| &c.consent_type == consent_type && c.granted)
                    .max_by_key(|c| c.granted_at);

                latest.map(|c| &c.version != current_version).unwrap_or(true)
            })
            .map(|(user_id, _)| *user_id)
            .collect()
    }
}

// ============================================================================
// Data Retention
// ============================================================================

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub retention_days: i64,
    pub action: RetentionAction,
    pub enabled: bool,
}

/// Retention action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionAction {
    Delete,
    Anonymize,
    Archive,
}

impl RetentionPolicy {
    pub fn new(id: &str, name: &str, data_type: &str, retention_days: i64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            data_type: data_type.to_string(),
            retention_days,
            action: RetentionAction::Delete,
            enabled: true,
        }
    }
}

/// Default retention policies
pub fn default_retention_policies() -> Vec<RetentionPolicy> {
    vec![
        RetentionPolicy::new("activity_logs", "Activity Logs", "activity", 365),
        RetentionPolicy::new("login_history", "Login History", "login", 90),
        RetentionPolicy::new("deleted_users", "Deleted User Data", "deleted_user", 30),
        RetentionPolicy::new("export_files", "Data Export Files", "export", 7),
        RetentionPolicy::new("sessions", "User Sessions", "session", 30),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_export_request() {
        let mut request = DataExportRequest::new(1, "127.0.0.1");
        assert_eq!(request.status, ExportStatus::Pending);

        request.mark_ready("/path/to/file.zip", 1024);
        assert_eq!(request.status, ExportStatus::Ready);
        assert!(request.is_download_valid());
    }

    #[test]
    fn test_deletion_request() {
        let mut request = DeletionRequest::new(1, "127.0.0.1");
        assert_eq!(request.status, DeletionStatus::Pending);

        request.confirm(14);
        assert_eq!(request.status, DeletionStatus::Scheduled);
    }

    #[test]
    fn test_deletion_manager() {
        let mut manager = DeletionManager::new();

        let request = DeletionRequest::new(1, "127.0.0.1");
        let token = request.confirmation_token.clone();
        manager.request(request);

        manager.confirm(&token).unwrap();

        let pending = manager.get_pending(1);
        assert!(pending.is_some());
    }

    #[test]
    fn test_consent() {
        let mut manager = ConsentManager::new();

        let consent = Consent::grant(1, ConsentType::PrivacyPolicy, "127.0.0.1", "1.0");
        manager.record(consent);

        assert!(manager.has_consent(1, &ConsentType::PrivacyPolicy));
        assert!(!manager.has_consent(1, &ConsentType::Marketing));
    }

    #[test]
    fn test_consent_revocation() {
        let mut manager = ConsentManager::new();

        let consent = Consent::grant(1, ConsentType::Marketing, "127.0.0.1", "1.0");
        manager.record(consent);

        manager.revoke(1, &ConsentType::Marketing);
        assert!(!manager.has_consent(1, &ConsentType::Marketing));
    }
}
