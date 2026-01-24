//! Publishing Workflow
//!
//! Status management, scheduling, and publishing workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Publishing settings and workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostPublishing {
    /// Current status
    pub status: PublishStatus,

    /// Scheduled publish date (for scheduled posts)
    pub scheduled_at: Option<DateTime<Utc>>,

    /// Actual publish date
    pub published_at: Option<DateTime<Utc>>,

    /// Expiration date (auto-unpublish)
    pub expires_at: Option<DateTime<Utc>>,

    /// Post visibility
    pub visibility: PostVisibility,

    /// Review workflow status
    pub review: Option<ReviewStatus>,

    /// Timezone for display
    pub timezone: Option<String>,

    /// Publishing history
    #[serde(default)]
    pub history: Vec<PublishEvent>,
}

impl Default for PostPublishing {
    fn default() -> Self {
        Self {
            status: PublishStatus::Draft,
            scheduled_at: None,
            published_at: None,
            expires_at: None,
            visibility: PostVisibility::Public,
            review: None,
            timezone: None,
            history: Vec::new(),
        }
    }
}

impl PostPublishing {
    /// Publish immediately
    pub fn publish(&mut self) {
        let now = Utc::now();
        self.status = PublishStatus::Published;
        self.published_at = Some(now);
        self.scheduled_at = None;
        self.history.push(PublishEvent {
            event_type: PublishEventType::Published,
            timestamp: now,
            user_id: None,
            note: None,
        });
    }

    /// Schedule for later
    pub fn schedule(&mut self, date: DateTime<Utc>) {
        self.status = PublishStatus::Scheduled;
        self.scheduled_at = Some(date);
        self.history.push(PublishEvent {
            event_type: PublishEventType::Scheduled,
            timestamp: Utc::now(),
            user_id: None,
            note: Some(format!("Scheduled for {}", date)),
        });
    }

    /// Revert to draft
    pub fn unpublish(&mut self) {
        self.status = PublishStatus::Draft;
        self.history.push(PublishEvent {
            event_type: PublishEventType::Unpublished,
            timestamp: Utc::now(),
            user_id: None,
            note: None,
        });
    }

    /// Move to trash
    pub fn trash(&mut self) {
        self.status = PublishStatus::Trash;
        self.history.push(PublishEvent {
            event_type: PublishEventType::Trashed,
            timestamp: Utc::now(),
            user_id: None,
            note: None,
        });
    }

    /// Restore from trash
    pub fn restore(&mut self) {
        self.status = PublishStatus::Draft;
        self.history.push(PublishEvent {
            event_type: PublishEventType::Restored,
            timestamp: Utc::now(),
            user_id: None,
            note: None,
        });
    }

    /// Submit for review
    pub fn submit_for_review(&mut self) {
        self.status = PublishStatus::Pending;
        self.review = Some(ReviewStatus::new());
        self.history.push(PublishEvent {
            event_type: PublishEventType::SubmittedForReview,
            timestamp: Utc::now(),
            user_id: None,
            note: None,
        });
    }

    /// Check if should auto-publish (for scheduled posts)
    pub fn should_publish(&self) -> bool {
        if self.status != PublishStatus::Scheduled {
            return false;
        }
        if let Some(scheduled) = self.scheduled_at {
            Utc::now() >= scheduled
        } else {
            false
        }
    }

    /// Check if should auto-expire
    pub fn should_expire(&self) -> bool {
        if self.status != PublishStatus::Published {
            return false;
        }
        if let Some(expires) = self.expires_at {
            Utc::now() >= expires
        } else {
            false
        }
    }

    /// Get effective publish date for display
    pub fn get_display_date(&self) -> Option<DateTime<Utc>> {
        self.published_at.or(self.scheduled_at)
    }

    /// Check if post is viewable
    pub fn is_viewable(&self) -> bool {
        matches!(
            self.status,
            PublishStatus::Published | PublishStatus::Private
        )
    }
}

/// Publish status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PublishStatus {
    /// Work in progress
    Draft,
    /// Auto-saved draft
    AutoDraft,
    /// Awaiting review
    Pending,
    /// Private (only visible to admins/editors)
    Private,
    /// Publicly published
    Published,
    /// Scheduled for future
    Scheduled,
    /// In trash
    Trash,
}

impl Default for PublishStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl PublishStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::AutoDraft => "Auto Draft",
            Self::Pending => "Pending Review",
            Self::Private => "Private",
            Self::Published => "Published",
            Self::Scheduled => "Scheduled",
            Self::Trash => "Trash",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Draft => "#6b7280",       // gray
            Self::AutoDraft => "#9ca3af",   // lighter gray
            Self::Pending => "#f59e0b",     // amber
            Self::Private => "#8b5cf6",     // purple
            Self::Published => "#10b981",   // green
            Self::Scheduled => "#3b82f6",   // blue
            Self::Trash => "#ef4444",       // red
        }
    }
}

/// Post visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostVisibility {
    /// Visible to everyone
    Public,
    /// Requires password
    PasswordProtected,
    /// Only visible to logged-in users
    Private,
    /// Only visible to specific roles
    RoleRestricted,
}

impl Default for PostVisibility {
    fn default() -> Self {
        Self::Public
    }
}

/// Review workflow status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatus {
    /// Current state
    pub state: ReviewState,
    /// Assigned reviewer
    pub reviewer_id: Option<i64>,
    /// Review notes
    pub notes: Option<String>,
    /// Requested changes
    #[serde(default)]
    pub requested_changes: Vec<String>,
    /// Last reviewed at
    pub reviewed_at: Option<DateTime<Utc>>,
}

impl ReviewStatus {
    pub fn new() -> Self {
        Self {
            state: ReviewState::Pending,
            reviewer_id: None,
            notes: None,
            requested_changes: Vec::new(),
            reviewed_at: None,
        }
    }
}

impl Default for ReviewStatus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewState {
    /// Awaiting review
    Pending,
    /// Under review
    InReview,
    /// Changes requested
    ChangesRequested,
    /// Approved
    Approved,
    /// Rejected
    Rejected,
}

/// Publishing history event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishEvent {
    pub event_type: PublishEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishEventType {
    Created,
    Published,
    Unpublished,
    Scheduled,
    Rescheduled,
    SubmittedForReview,
    ReviewApproved,
    ReviewRejected,
    ChangesRequested,
    Trashed,
    Restored,
    VisibilityChanged,
    Expired,
}
