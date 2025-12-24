//! # Content Access Control
//!
//! Password protection, staging/preview, and membership restrictions.
//!
//! Features:
//! - Password-protected posts
//! - Content staging and preview
//! - Membership/paywall restrictions
//! - Access tokens and preview links

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Password Protection
// ============================================================================

/// Password-protected content manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordProtection {
    /// Post ID
    pub post_id: i64,

    /// Password hash (not plain text)
    pub password_hash: String,

    /// Custom message to show
    pub message: String,

    /// Expiration of password (None = permanent)
    pub expires_at: Option<DateTime<Utc>>,

    /// When protection was set
    pub created_at: DateTime<Utc>,
}

impl PasswordProtection {
    pub fn new(post_id: i64, password_hash: &str) -> Self {
        Self {
            post_id,
            password_hash: password_hash.to_string(),
            message: "This content is password protected. Please enter the password to view.".to_string(),
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_expiration(mut self, expires: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }
}

/// Password protection session (for remembering access)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordSession {
    pub session_id: String,
    pub post_id: i64,
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl PasswordSession {
    pub fn new(post_id: i64, duration_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            post_id,
            granted_at: now,
            expires_at: now + Duration::hours(duration_hours),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

// ============================================================================
// Content Staging/Preview
// ============================================================================

/// Staged content for preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedContent {
    /// Unique staging ID
    pub id: Uuid,

    /// Original post ID
    pub post_id: i64,

    /// Staged title
    pub title: String,

    /// Staged content
    pub content: String,

    /// Staged excerpt
    pub excerpt: String,

    /// Staged meta
    pub meta: HashMap<String, serde_json::Value>,

    /// Who created this staged version
    pub author_id: i64,

    /// When it was staged
    pub created_at: DateTime<Utc>,

    /// Last modified
    pub modified_at: DateTime<Utc>,

    /// Notes about this version
    pub notes: String,

    /// Staging status
    pub status: StagingStatus,
}

/// Staging status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagingStatus {
    Draft,
    PendingReview,
    Approved,
    Published,
    Rejected,
}

impl StagedContent {
    pub fn new(post_id: i64, author_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            post_id,
            title: String::new(),
            content: String::new(),
            excerpt: String::new(),
            meta: HashMap::new(),
            author_id,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            notes: String::new(),
            status: StagingStatus::Draft,
        }
    }
}

/// Preview token for sharing unpublished content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewToken {
    /// Token string
    pub token: String,

    /// Post ID this previews
    pub post_id: i64,

    /// Optional staged content ID
    pub staged_id: Option<Uuid>,

    /// Who created the token
    pub created_by: i64,

    /// When token was created
    pub created_at: DateTime<Utc>,

    /// When token expires
    pub expires_at: DateTime<Utc>,

    /// Max number of uses (None = unlimited)
    pub max_uses: Option<u32>,

    /// Current use count
    pub use_count: u32,

    /// Token active status
    pub active: bool,
}

impl PreviewToken {
    pub fn new(post_id: i64, created_by: i64, duration_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            token: Uuid::new_v4().to_string(),
            post_id,
            staged_id: None,
            created_by,
            created_at: now,
            expires_at: now + Duration::hours(duration_hours),
            max_uses: None,
            use_count: 0,
            active: true,
        }
    }

    pub fn for_staged(mut self, staged_id: Uuid) -> Self {
        self.staged_id = Some(staged_id);
        self
    }

    pub fn with_max_uses(mut self, max: u32) -> Self {
        self.max_uses = Some(max);
        self
    }

    pub fn is_valid(&self) -> bool {
        if !self.active {
            return false;
        }

        if Utc::now() > self.expires_at {
            return false;
        }

        if let Some(max) = self.max_uses {
            if self.use_count >= max {
                return false;
            }
        }

        true
    }

    pub fn use_token(&mut self) -> bool {
        if !self.is_valid() {
            return false;
        }

        self.use_count += 1;
        true
    }

    /// Generate preview URL
    pub fn preview_url(&self, base_url: &str) -> String {
        format!("{}?preview=true&token={}", base_url, self.token)
    }
}

// ============================================================================
// Membership/Paywall Restrictions
// ============================================================================

/// Access restriction level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Public - anyone can view
    Public,

    /// Registered - must be logged in
    Registered,

    /// Subscriber level
    Subscriber,

    /// Member level (paid)
    Member,

    /// Premium member
    Premium,

    /// Administrator only
    Admin,

    /// Custom level
    Custom(u32),
}

impl AccessLevel {
    pub fn weight(&self) -> u32 {
        match self {
            Self::Public => 0,
            Self::Registered => 10,
            Self::Subscriber => 20,
            Self::Member => 30,
            Self::Premium => 40,
            Self::Admin => 100,
            Self::Custom(w) => *w,
        }
    }

    /// Check if user level can access this content level
    pub fn can_access(&self, user_level: AccessLevel) -> bool {
        user_level.weight() >= self.weight()
    }
}

/// Content access rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Rule ID
    pub id: Uuid,

    /// Rule name
    pub name: String,

    /// Required access level
    pub level: AccessLevel,

    /// Required membership plan IDs (if any)
    pub membership_plans: Vec<i64>,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Redirect URL when access denied
    pub redirect_url: Option<String>,

    /// Message to show when access denied
    pub denied_message: String,

    /// Allow teaser content
    pub show_teaser: bool,

    /// Teaser length (in characters)
    pub teaser_length: usize,

    /// Allow excerpt when restricted
    pub show_excerpt: bool,

    /// Active status
    pub active: bool,
}

impl AccessRule {
    pub fn new(name: &str, level: AccessLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            level,
            membership_plans: Vec::new(),
            capabilities: Vec::new(),
            redirect_url: None,
            denied_message: "You don't have permission to view this content.".to_string(),
            show_teaser: false,
            teaser_length: 150,
            show_excerpt: true,
            active: true,
        }
    }

    pub fn with_plans(mut self, plans: Vec<i64>) -> Self {
        self.membership_plans = plans;
        self
    }

    pub fn with_teaser(mut self, length: usize) -> Self {
        self.show_teaser = true;
        self.teaser_length = length;
        self
    }

    pub fn with_redirect(mut self, url: &str) -> Self {
        self.redirect_url = Some(url.to_string());
        self
    }
}

/// Content restriction assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRestriction {
    /// Content ID (post, page, etc.)
    pub content_id: i64,

    /// Content type
    pub content_type: String,

    /// Applied rule ID
    pub rule_id: Uuid,

    /// Restriction scope
    pub scope: RestrictionScope,

    /// Custom override message
    pub custom_message: Option<String>,
}

/// Scope of restriction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestrictionScope {
    /// Entire content
    Full,

    /// Only specific parts (marked in content)
    Partial,

    /// Time-based (content delayed)
    Delayed,

    /// After certain date
    AfterDate,
}

/// Access manager
pub struct AccessManager {
    rules: HashMap<Uuid, AccessRule>,
    restrictions: HashMap<i64, Vec<ContentRestriction>>,
    password_protection: HashMap<i64, PasswordProtection>,
    preview_tokens: HashMap<String, PreviewToken>,
    staged_content: HashMap<Uuid, StagedContent>,
}

impl Default for AccessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessManager {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            restrictions: HashMap::new(),
            password_protection: HashMap::new(),
            preview_tokens: HashMap::new(),
            staged_content: HashMap::new(),
        }
    }

    /// Register an access rule
    pub fn register_rule(&mut self, rule: AccessRule) {
        self.rules.insert(rule.id, rule);
    }

    /// Get a rule by ID
    pub fn get_rule(&self, id: &Uuid) -> Option<&AccessRule> {
        self.rules.get(id)
    }

    /// Apply restriction to content
    pub fn restrict_content(&mut self, restriction: ContentRestriction) {
        self.restrictions
            .entry(restriction.content_id)
            .or_insert_with(Vec::new)
            .push(restriction);
    }

    /// Check if user can access content
    pub fn can_access(
        &self,
        content_id: i64,
        user_level: AccessLevel,
        user_plans: &[i64],
    ) -> AccessCheckResult {
        // Check restrictions
        if let Some(restrictions) = self.restrictions.get(&content_id) {
            for restriction in restrictions {
                if let Some(rule) = self.rules.get(&restriction.rule_id) {
                    if !rule.active {
                        continue;
                    }

                    // Check access level
                    if !rule.level.can_access(user_level) {
                        return AccessCheckResult::Denied {
                            reason: rule.denied_message.clone(),
                            redirect: rule.redirect_url.clone(),
                            show_teaser: rule.show_teaser,
                            teaser_length: rule.teaser_length,
                        };
                    }

                    // Check membership plans
                    if !rule.membership_plans.is_empty() {
                        let has_plan = rule.membership_plans.iter()
                            .any(|plan| user_plans.contains(plan));

                        if !has_plan {
                            return AccessCheckResult::Denied {
                                reason: rule.denied_message.clone(),
                                redirect: rule.redirect_url.clone(),
                                show_teaser: rule.show_teaser,
                                teaser_length: rule.teaser_length,
                            };
                        }
                    }
                }
            }
        }

        // Check password protection
        if self.password_protection.contains_key(&content_id) {
            return AccessCheckResult::PasswordRequired;
        }

        AccessCheckResult::Allowed
    }

    /// Set password protection
    pub fn set_password(&mut self, protection: PasswordProtection) {
        self.password_protection.insert(protection.post_id, protection);
    }

    /// Remove password protection
    pub fn remove_password(&mut self, post_id: i64) {
        self.password_protection.remove(&post_id);
    }

    /// Create preview token
    pub fn create_preview_token(&mut self, post_id: i64, created_by: i64, hours: i64) -> PreviewToken {
        let token = PreviewToken::new(post_id, created_by, hours);
        self.preview_tokens.insert(token.token.clone(), token.clone());
        token
    }

    /// Validate preview token
    pub fn validate_preview_token(&mut self, token: &str) -> Option<&PreviewToken> {
        if let Some(preview) = self.preview_tokens.get_mut(token) {
            if preview.use_token() {
                return self.preview_tokens.get(token);
            }
        }
        None
    }

    /// Create staged content
    pub fn create_staged(&mut self, staged: StagedContent) {
        self.staged_content.insert(staged.id, staged);
    }

    /// Get staged content
    pub fn get_staged(&self, id: &Uuid) -> Option<&StagedContent> {
        self.staged_content.get(id)
    }

    /// Publish staged content (returns the staged content for application)
    pub fn publish_staged(&mut self, id: &Uuid) -> Option<StagedContent> {
        self.staged_content.remove(id)
    }
}

/// Result of access check
#[derive(Debug, Clone)]
pub enum AccessCheckResult {
    Allowed,
    PasswordRequired,
    Denied {
        reason: String,
        redirect: Option<String>,
        show_teaser: bool,
        teaser_length: usize,
    },
}

impl AccessCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_levels() {
        assert!(AccessLevel::Member.can_access(AccessLevel::Premium));
        assert!(!AccessLevel::Premium.can_access(AccessLevel::Member));
        assert!(AccessLevel::Public.can_access(AccessLevel::Registered));
    }

    #[test]
    fn test_preview_token() {
        let mut token = PreviewToken::new(1, 1, 24);
        assert!(token.is_valid());
        assert!(token.use_token());
        assert_eq!(token.use_count, 1);
    }

    #[test]
    fn test_max_uses() {
        let mut token = PreviewToken::new(1, 1, 24).with_max_uses(2);
        assert!(token.use_token());
        assert!(token.use_token());
        assert!(!token.use_token()); // Max uses reached
    }

    #[test]
    fn test_access_manager() {
        let mut manager = AccessManager::new();

        let rule = AccessRule::new("Members Only", AccessLevel::Member);
        let rule_id = rule.id;
        manager.register_rule(rule);

        manager.restrict_content(ContentRestriction {
            content_id: 1,
            content_type: "post".to_string(),
            rule_id,
            scope: RestrictionScope::Full,
            custom_message: None,
        });

        let result = manager.can_access(1, AccessLevel::Public, &[]);
        assert!(!result.is_allowed());

        let result = manager.can_access(1, AccessLevel::Premium, &[]);
        assert!(result.is_allowed());
    }
}
