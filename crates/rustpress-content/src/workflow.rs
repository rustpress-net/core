//! # Content Workflow System
//!
//! Draft/publish/schedule workflow states and content locking.
//!
//! Features:
//! - Post status management
//! - Scheduled publishing
//! - Content locking for concurrent editing
//! - Workflow transitions
//! - Publishing permissions

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Workflow errors
#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("Invalid status transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Content is locked by user {0}")]
    ContentLocked(i64),

    #[error("Lock expired")]
    LockExpired,

    #[error("Not authorized for this action")]
    NotAuthorized,

    #[error("Invalid schedule date: {0}")]
    InvalidScheduleDate(String),

    #[error("Workflow error: {0}")]
    Other(String),
}

/// Post status (workflow state)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostStatus {
    /// Work in progress, not visible
    Draft,

    /// Submitted for review
    Pending,

    /// Published and visible
    Publish,

    /// Scheduled for future publication
    Future,

    /// Private (visible only to admins/author)
    Private,

    /// In trash (soft deleted)
    Trash,

    /// Auto-draft (system created)
    AutoDraft,

    /// Inherited from parent
    Inherit,

    /// Custom status
    Custom(u32),
}

impl PostStatus {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Publish => "publish",
            Self::Future => "future",
            Self::Private => "private",
            Self::Trash => "trash",
            Self::AutoDraft => "auto-draft",
            Self::Inherit => "inherit",
            Self::Custom(_) => "custom",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => Self::Draft,
            "pending" => Self::Pending,
            "publish" | "published" => Self::Publish,
            "future" | "scheduled" => Self::Future,
            "private" => Self::Private,
            "trash" | "trashed" => Self::Trash,
            "auto-draft" | "autodraft" => Self::AutoDraft,
            "inherit" => Self::Inherit,
            _ => Self::Draft,
        }
    }

    /// Check if content is publicly visible
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Publish)
    }

    /// Check if content is viewable (to those with access)
    pub fn is_viewable(&self) -> bool {
        matches!(self, Self::Publish | Self::Private | Self::Future)
    }

    /// Check if content is editable
    pub fn is_editable(&self) -> bool {
        !matches!(self, Self::Trash)
    }

    /// Get user-friendly label
    pub fn label(&self) -> &str {
        match self {
            Self::Draft => "Draft",
            Self::Pending => "Pending Review",
            Self::Publish => "Published",
            Self::Future => "Scheduled",
            Self::Private => "Private",
            Self::Trash => "Trashed",
            Self::AutoDraft => "Auto-Draft",
            Self::Inherit => "Inherited",
            Self::Custom(_) => "Custom",
        }
    }

    /// Get color for status badge
    pub fn color(&self) -> &str {
        match self {
            Self::Draft => "#999",
            Self::Pending => "#f0ad4e",
            Self::Publish => "#5cb85c",
            Self::Future => "#5bc0de",
            Self::Private => "#d9534f",
            Self::Trash => "#777",
            Self::AutoDraft => "#ccc",
            Self::Inherit => "#aaa",
            Self::Custom(_) => "#666",
        }
    }
}

impl Default for PostStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Workflow manager for status transitions
pub struct WorkflowManager {
    /// Allowed transitions (from -> to)
    transitions: HashMap<PostStatus, Vec<PostStatus>>,

    /// Custom statuses
    custom_statuses: HashMap<u32, CustomStatus>,

    /// Default status for new posts
    default_status: PostStatus,
}

impl Default for WorkflowManager {
    fn default() -> Self {
        let mut manager = Self {
            transitions: HashMap::new(),
            custom_statuses: HashMap::new(),
            default_status: PostStatus::Draft,
        };

        // Set up default transitions
        manager.init_default_transitions();
        manager
    }
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize default status transitions
    fn init_default_transitions(&mut self) {
        // Draft can go to: Pending, Publish, Future, Private, Trash
        self.transitions.insert(PostStatus::Draft, vec![
            PostStatus::Pending,
            PostStatus::Publish,
            PostStatus::Future,
            PostStatus::Private,
            PostStatus::Trash,
        ]);

        // Auto-draft can go to: Draft, Trash
        self.transitions.insert(PostStatus::AutoDraft, vec![
            PostStatus::Draft,
            PostStatus::Trash,
        ]);

        // Pending can go to: Draft, Publish, Future, Private, Trash
        self.transitions.insert(PostStatus::Pending, vec![
            PostStatus::Draft,
            PostStatus::Publish,
            PostStatus::Future,
            PostStatus::Private,
            PostStatus::Trash,
        ]);

        // Publish can go to: Draft, Pending, Private, Trash
        self.transitions.insert(PostStatus::Publish, vec![
            PostStatus::Draft,
            PostStatus::Pending,
            PostStatus::Private,
            PostStatus::Trash,
        ]);

        // Future can go to: Draft, Pending, Publish, Private, Trash
        self.transitions.insert(PostStatus::Future, vec![
            PostStatus::Draft,
            PostStatus::Pending,
            PostStatus::Publish,
            PostStatus::Private,
            PostStatus::Trash,
        ]);

        // Private can go to: Draft, Publish, Trash
        self.transitions.insert(PostStatus::Private, vec![
            PostStatus::Draft,
            PostStatus::Publish,
            PostStatus::Trash,
        ]);

        // Trash can go to: Draft (restore)
        self.transitions.insert(PostStatus::Trash, vec![
            PostStatus::Draft,
        ]);
    }

    /// Check if transition is allowed
    pub fn can_transition(&self, from: PostStatus, to: PostStatus) -> bool {
        self.transitions
            .get(&from)
            .map(|allowed| allowed.contains(&to))
            .unwrap_or(false)
    }

    /// Perform status transition
    pub fn transition(
        &self,
        current: PostStatus,
        new: PostStatus,
        context: &TransitionContext,
    ) -> Result<TransitionResult, WorkflowError> {
        if !self.can_transition(current, new) {
            return Err(WorkflowError::InvalidTransition {
                from: current.as_str().to_string(),
                to: new.as_str().to_string(),
            });
        }

        // Check permissions
        if !self.has_permission(current, new, context) {
            return Err(WorkflowError::NotAuthorized);
        }

        // Validate scheduled date if transitioning to Future
        if new == PostStatus::Future {
            if let Some(date) = context.scheduled_date {
                if date <= Utc::now() {
                    return Err(WorkflowError::InvalidScheduleDate(
                        "Schedule date must be in the future".to_string()
                    ));
                }
            } else {
                return Err(WorkflowError::InvalidScheduleDate(
                    "Schedule date is required for future posts".to_string()
                ));
            }
        }

        Ok(TransitionResult {
            from: current,
            to: new,
            transitioned_at: Utc::now(),
            transitioned_by: context.user_id,
            scheduled_date: context.scheduled_date,
            notes: context.notes.clone(),
        })
    }

    /// Check if user has permission for transition
    fn has_permission(&self, from: PostStatus, to: PostStatus, context: &TransitionContext) -> bool {
        // Contributors can only create drafts and submit for pending
        if context.role == UserRole::Contributor {
            return matches!(
                (from, to),
                (PostStatus::Draft, PostStatus::Pending) |
                (PostStatus::AutoDraft, PostStatus::Draft)
            );
        }

        // Authors can publish their own posts
        if context.role == UserRole::Author {
            if context.is_owner {
                return true;
            }
            return matches!(to, PostStatus::Draft | PostStatus::Pending);
        }

        // Editors and admins can do anything
        matches!(context.role, UserRole::Editor | UserRole::Administrator)
    }

    /// Register a custom status
    pub fn register_custom_status(&mut self, id: u32, status: CustomStatus) {
        self.custom_statuses.insert(id, status);
    }

    /// Get allowed transitions from a status
    pub fn get_allowed_transitions(&self, from: PostStatus) -> Vec<PostStatus> {
        self.transitions.get(&from).cloned().unwrap_or_default()
    }

    /// Add a custom transition
    pub fn add_transition(&mut self, from: PostStatus, to: PostStatus) {
        self.transitions
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);
    }
}

/// Context for status transition
#[derive(Debug, Clone)]
pub struct TransitionContext {
    /// User performing the transition
    pub user_id: i64,

    /// User's role
    pub role: UserRole,

    /// Whether user owns the content
    pub is_owner: bool,

    /// Scheduled publication date (for Future status)
    pub scheduled_date: Option<DateTime<Utc>>,

    /// Optional notes about the transition
    pub notes: Option<String>,
}

/// Result of a status transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    pub from: PostStatus,
    pub to: PostStatus,
    pub transitioned_at: DateTime<Utc>,
    pub transitioned_by: i64,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// User role for permission checks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Subscriber,
    Contributor,
    Author,
    Editor,
    Administrator,
}

/// Custom status definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatus {
    pub id: u32,
    pub name: String,
    pub label: String,
    pub description: String,
    pub color: String,
    pub is_public: bool,
    pub allowed_transitions: Vec<PostStatus>,
}

// ============================================================================
// Content Locking System
// ============================================================================

/// Content lock for concurrent editing protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLock {
    /// Unique lock ID
    pub id: Uuid,

    /// ID of the locked content
    pub content_id: i64,

    /// Content type (post, page, etc.)
    pub content_type: String,

    /// User who holds the lock
    pub user_id: i64,

    /// User's display name
    pub user_name: String,

    /// When the lock was acquired
    pub locked_at: DateTime<Utc>,

    /// When the lock expires
    pub expires_at: DateTime<Utc>,

    /// Lock token for verification
    pub token: String,

    /// Whether this is a heartbeat-based lock
    pub requires_heartbeat: bool,

    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,
}

impl ContentLock {
    pub fn new(content_id: i64, content_type: &str, user_id: i64, user_name: &str, duration_minutes: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content_id,
            content_type: content_type.to_string(),
            user_id,
            user_name: user_name.to_string(),
            locked_at: now,
            expires_at: now + Duration::minutes(duration_minutes),
            token: Uuid::new_v4().to_string(),
            requires_heartbeat: true,
            last_heartbeat: now,
        }
    }

    /// Check if lock is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if heartbeat is stale
    pub fn is_heartbeat_stale(&self, max_stale_seconds: i64) -> bool {
        if !self.requires_heartbeat {
            return false;
        }
        let stale_threshold = self.last_heartbeat + Duration::seconds(max_stale_seconds);
        Utc::now() > stale_threshold
    }

    /// Extend the lock
    pub fn extend(&mut self, additional_minutes: i64) {
        self.expires_at = Utc::now() + Duration::minutes(additional_minutes);
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        // Also extend expiration slightly on heartbeat
        let min_expiration = Utc::now() + Duration::minutes(5);
        if self.expires_at < min_expiration {
            self.expires_at = min_expiration;
        }
    }

    /// Verify lock token
    pub fn verify_token(&self, token: &str) -> bool {
        self.token == token
    }
}

/// Content locking manager
pub struct LockManager {
    /// Active locks (content_id -> lock)
    locks: HashMap<i64, ContentLock>,

    /// Default lock duration in minutes
    default_duration: i64,

    /// Maximum lock duration in minutes
    max_duration: i64,

    /// Heartbeat timeout in seconds
    heartbeat_timeout: i64,

    /// Whether to allow lock takeover
    allow_takeover: bool,
}

impl Default for LockManager {
    fn default() -> Self {
        Self {
            locks: HashMap::new(),
            default_duration: 30,
            max_duration: 120,
            heartbeat_timeout: 60,
            allow_takeover: true,
        }
    }
}

impl LockManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_duration(mut self, minutes: i64) -> Self {
        self.default_duration = minutes;
        self
    }

    pub fn with_heartbeat_timeout(mut self, seconds: i64) -> Self {
        self.heartbeat_timeout = seconds;
        self
    }

    /// Acquire a lock on content
    pub fn acquire(
        &mut self,
        content_id: i64,
        content_type: &str,
        user_id: i64,
        user_name: &str,
    ) -> Result<ContentLock, WorkflowError> {
        // Check for existing lock
        if let Some(existing) = self.locks.get(&content_id) {
            if !existing.is_expired() && !existing.is_heartbeat_stale(self.heartbeat_timeout) {
                if existing.user_id != user_id {
                    return Err(WorkflowError::ContentLocked(existing.user_id));
                }
                // Same user, extend the lock
                let mut lock = existing.clone();
                lock.extend(self.default_duration);
                self.locks.insert(content_id, lock.clone());
                return Ok(lock);
            }
        }

        // Create new lock
        let lock = ContentLock::new(content_id, content_type, user_id, user_name, self.default_duration);
        self.locks.insert(content_id, lock.clone());
        Ok(lock)
    }

    /// Release a lock
    pub fn release(&mut self, content_id: i64, user_id: i64, token: &str) -> Result<(), WorkflowError> {
        if let Some(lock) = self.locks.get(&content_id) {
            if lock.user_id != user_id {
                return Err(WorkflowError::NotAuthorized);
            }
            if !lock.verify_token(token) {
                return Err(WorkflowError::NotAuthorized);
            }
        }

        self.locks.remove(&content_id);
        Ok(())
    }

    /// Update heartbeat for a lock
    pub fn heartbeat(&mut self, content_id: i64, user_id: i64, token: &str) -> Result<(), WorkflowError> {
        if let Some(lock) = self.locks.get_mut(&content_id) {
            if lock.user_id != user_id {
                return Err(WorkflowError::NotAuthorized);
            }
            if !lock.verify_token(token) {
                return Err(WorkflowError::NotAuthorized);
            }
            lock.heartbeat();
            Ok(())
        } else {
            Err(WorkflowError::LockExpired)
        }
    }

    /// Check if content is locked
    pub fn is_locked(&self, content_id: i64) -> Option<&ContentLock> {
        self.locks.get(&content_id).filter(|lock| {
            !lock.is_expired() && !lock.is_heartbeat_stale(self.heartbeat_timeout)
        })
    }

    /// Check if user can edit content
    pub fn can_edit(&self, content_id: i64, user_id: i64) -> bool {
        match self.is_locked(content_id) {
            Some(lock) => lock.user_id == user_id,
            None => true,
        }
    }

    /// Forcefully take over a lock (admin only)
    pub fn takeover(
        &mut self,
        content_id: i64,
        content_type: &str,
        new_user_id: i64,
        new_user_name: &str,
    ) -> Result<ContentLock, WorkflowError> {
        if !self.allow_takeover {
            return Err(WorkflowError::NotAuthorized);
        }

        self.locks.remove(&content_id);
        self.acquire(content_id, content_type, new_user_id, new_user_name)
    }

    /// Clean up expired locks
    pub fn cleanup_expired(&mut self) {
        self.locks.retain(|_, lock| {
            !lock.is_expired() && !lock.is_heartbeat_stale(self.heartbeat_timeout)
        });
    }

    /// Get all active locks
    pub fn get_all_locks(&self) -> Vec<&ContentLock> {
        self.locks.values()
            .filter(|lock| !lock.is_expired() && !lock.is_heartbeat_stale(self.heartbeat_timeout))
            .collect()
    }

    /// Get locks by user
    pub fn get_user_locks(&self, user_id: i64) -> Vec<&ContentLock> {
        self.locks.values()
            .filter(|lock| lock.user_id == user_id && !lock.is_expired())
            .collect()
    }
}

/// Lock check result for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockStatus {
    pub is_locked: bool,
    pub locked_by: Option<i64>,
    pub locked_by_name: Option<String>,
    pub locked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub can_edit: bool,
    pub can_takeover: bool,
}

impl LockStatus {
    pub fn from_lock(lock: Option<&ContentLock>, current_user_id: i64, is_admin: bool) -> Self {
        match lock {
            Some(lock) => Self {
                is_locked: true,
                locked_by: Some(lock.user_id),
                locked_by_name: Some(lock.user_name.clone()),
                locked_at: Some(lock.locked_at),
                expires_at: Some(lock.expires_at),
                can_edit: lock.user_id == current_user_id,
                can_takeover: is_admin && lock.user_id != current_user_id,
            },
            None => Self {
                is_locked: false,
                locked_by: None,
                locked_by_name: None,
                locked_at: None,
                expires_at: None,
                can_edit: true,
                can_takeover: false,
            },
        }
    }
}

// ============================================================================
// Scheduled Publishing
// ============================================================================

/// Scheduled post for future publication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledPost {
    pub post_id: i64,
    pub scheduled_date: DateTime<Utc>,
    pub scheduled_by: i64,
    pub timezone: String,
    pub status_after_publish: PostStatus,
}

/// Scheduler for handling scheduled publications
pub struct WorkflowScheduler {
    /// Scheduled posts
    scheduled: Vec<ScheduledPost>,
}

impl Default for WorkflowScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowScheduler {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
        }
    }

    /// Schedule a post for future publication
    pub fn schedule(
        &mut self,
        post_id: i64,
        date: DateTime<Utc>,
        user_id: i64,
        timezone: &str,
    ) -> Result<ScheduledPost, WorkflowError> {
        if date <= Utc::now() {
            return Err(WorkflowError::InvalidScheduleDate(
                "Date must be in the future".to_string()
            ));
        }

        let scheduled = ScheduledPost {
            post_id,
            scheduled_date: date,
            scheduled_by: user_id,
            timezone: timezone.to_string(),
            status_after_publish: PostStatus::Publish,
        };

        // Remove any existing schedule for this post
        self.scheduled.retain(|s| s.post_id != post_id);
        self.scheduled.push(scheduled.clone());

        Ok(scheduled)
    }

    /// Unschedule a post
    pub fn unschedule(&mut self, post_id: i64) {
        self.scheduled.retain(|s| s.post_id != post_id);
    }

    /// Get posts due for publication
    pub fn get_due_posts(&self) -> Vec<&ScheduledPost> {
        let now = Utc::now();
        self.scheduled.iter()
            .filter(|s| s.scheduled_date <= now)
            .collect()
    }

    /// Process due posts (returns post IDs to publish)
    pub fn process_due(&mut self) -> Vec<i64> {
        let now = Utc::now();
        let due: Vec<i64> = self.scheduled.iter()
            .filter(|s| s.scheduled_date <= now)
            .map(|s| s.post_id)
            .collect();

        self.scheduled.retain(|s| s.scheduled_date > now);
        due
    }

    /// Get scheduled post info
    pub fn get_schedule(&self, post_id: i64) -> Option<&ScheduledPost> {
        self.scheduled.iter().find(|s| s.post_id == post_id)
    }

    /// Get all scheduled posts
    pub fn get_all_scheduled(&self) -> &[ScheduledPost] {
        &self.scheduled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_status() {
        assert!(PostStatus::Publish.is_public());
        assert!(!PostStatus::Draft.is_public());
        assert!(PostStatus::Draft.is_editable());
        assert!(!PostStatus::Trash.is_editable());
    }

    #[test]
    fn test_workflow_transitions() {
        let manager = WorkflowManager::new();

        assert!(manager.can_transition(PostStatus::Draft, PostStatus::Publish));
        assert!(manager.can_transition(PostStatus::Draft, PostStatus::Pending));
        assert!(!manager.can_transition(PostStatus::AutoDraft, PostStatus::Publish));
    }

    #[test]
    fn test_content_lock() {
        let mut manager = LockManager::new();

        let lock = manager.acquire(1, "post", 100, "Test User").unwrap();
        assert!(!lock.is_expired());

        // Same user can extend
        let lock2 = manager.acquire(1, "post", 100, "Test User").unwrap();
        assert_eq!(lock2.user_id, 100);

        // Different user should fail
        let result = manager.acquire(1, "post", 200, "Other User");
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_expiration() {
        let mut lock = ContentLock::new(1, "post", 100, "Test", 0);
        // With 0 minutes, should be expired immediately
        assert!(lock.is_expired());
    }

    #[test]
    fn test_scheduler() {
        let mut scheduler = WorkflowScheduler::new();

        let future = Utc::now() + Duration::hours(1);
        let result = scheduler.schedule(1, future, 100, "UTC");
        assert!(result.is_ok());

        let due = scheduler.get_due_posts();
        assert!(due.is_empty()); // Not due yet
    }
}
