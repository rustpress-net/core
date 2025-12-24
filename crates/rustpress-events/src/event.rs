//! Event types and domain events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use uuid::Uuid;

/// Event type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventType(String);

impl EventType {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Base event trait
pub trait Event: Send + Sync + 'static {
    /// Get the event type
    fn event_type(&self) -> EventType;

    /// Get the event ID
    fn event_id(&self) -> Uuid;

    /// Get the timestamp when the event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Get the aggregate ID (if applicable)
    fn aggregate_id(&self) -> Option<Uuid> {
        None
    }

    /// Get the tenant ID (if applicable)
    fn tenant_id(&self) -> Option<Uuid> {
        None
    }

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Serialize to JSON
    fn to_json(&self) -> serde_json::Value;
}

/// Domain event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub event_type: String,
    pub aggregate_id: Option<Uuid>,
    pub aggregate_type: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent {
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_type: event_type.into(),
            aggregate_id: None,
            aggregate_type: None,
            tenant_id: None,
            payload,
            metadata: EventMetadata::default(),
            occurred_at: Utc::now(),
        }
    }

    pub fn with_aggregate(mut self, id: Uuid, aggregate_type: impl Into<String>) -> Self {
        self.aggregate_id = Some(id);
        self.aggregate_type = Some(aggregate_type.into());
        self
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.data.insert(key.into(), value);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.metadata.causation_id = Some(causation_id);
        self
    }
}

impl Event for DomainEvent {
    fn event_type(&self) -> EventType {
        EventType::new(&self.event_type)
    }

    fn event_id(&self) -> Uuid {
        self.id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn aggregate_id(&self) -> Option<Uuid> {
        self.aggregate_id
    }

    fn tenant_id(&self) -> Option<Uuid> {
        self.tenant_id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

/// Event metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub request_id: Option<Uuid>,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Predefined event types
pub mod events {
    use super::*;

    // User events
    pub const USER_CREATED: &str = "user.created";
    pub const USER_UPDATED: &str = "user.updated";
    pub const USER_DELETED: &str = "user.deleted";
    pub const USER_LOGGED_IN: &str = "user.logged_in";
    pub const USER_LOGGED_OUT: &str = "user.logged_out";
    pub const USER_PASSWORD_CHANGED: &str = "user.password_changed";
    pub const USER_EMAIL_VERIFIED: &str = "user.email_verified";

    // Post events
    pub const POST_CREATED: &str = "post.created";
    pub const POST_UPDATED: &str = "post.updated";
    pub const POST_DELETED: &str = "post.deleted";
    pub const POST_PUBLISHED: &str = "post.published";
    pub const POST_UNPUBLISHED: &str = "post.unpublished";
    pub const POST_TRASHED: &str = "post.trashed";
    pub const POST_RESTORED: &str = "post.restored";

    // Page events
    pub const PAGE_CREATED: &str = "page.created";
    pub const PAGE_UPDATED: &str = "page.updated";
    pub const PAGE_DELETED: &str = "page.deleted";
    pub const PAGE_PUBLISHED: &str = "page.published";

    // Comment events
    pub const COMMENT_CREATED: &str = "comment.created";
    pub const COMMENT_UPDATED: &str = "comment.updated";
    pub const COMMENT_DELETED: &str = "comment.deleted";
    pub const COMMENT_APPROVED: &str = "comment.approved";
    pub const COMMENT_MARKED_SPAM: &str = "comment.marked_spam";

    // Media events
    pub const MEDIA_UPLOADED: &str = "media.uploaded";
    pub const MEDIA_UPDATED: &str = "media.updated";
    pub const MEDIA_DELETED: &str = "media.deleted";

    // Plugin events
    pub const PLUGIN_ACTIVATED: &str = "plugin.activated";
    pub const PLUGIN_DEACTIVATED: &str = "plugin.deactivated";
    pub const PLUGIN_INSTALLED: &str = "plugin.installed";
    pub const PLUGIN_UNINSTALLED: &str = "plugin.uninstalled";

    // Theme events
    pub const THEME_ACTIVATED: &str = "theme.activated";
    pub const THEME_DEACTIVATED: &str = "theme.deactivated";

    // Tenant events
    pub const TENANT_CREATED: &str = "tenant.created";
    pub const TENANT_UPDATED: &str = "tenant.updated";
    pub const TENANT_SUSPENDED: &str = "tenant.suspended";
    pub const TENANT_ACTIVATED: &str = "tenant.activated";

    // System events
    pub const SYSTEM_STARTUP: &str = "system.startup";
    pub const SYSTEM_SHUTDOWN: &str = "system.shutdown";
    pub const CACHE_CLEARED: &str = "cache.cleared";
    pub const SETTINGS_UPDATED: &str = "settings.updated";

    /// Create a user created event
    pub fn user_created(user_id: Uuid, email: &str, username: &str) -> DomainEvent {
        DomainEvent::new(
            USER_CREATED,
            serde_json::json!({
                "user_id": user_id,
                "email": email,
                "username": username,
            }),
        )
        .with_aggregate(user_id, "user")
    }

    /// Create a post created event
    pub fn post_created(post_id: Uuid, author_id: Uuid, title: &str, slug: &str) -> DomainEvent {
        DomainEvent::new(
            POST_CREATED,
            serde_json::json!({
                "post_id": post_id,
                "author_id": author_id,
                "title": title,
                "slug": slug,
            }),
        )
        .with_aggregate(post_id, "post")
    }

    /// Create a post published event
    pub fn post_published(post_id: Uuid, author_id: Uuid, title: &str) -> DomainEvent {
        DomainEvent::new(
            POST_PUBLISHED,
            serde_json::json!({
                "post_id": post_id,
                "author_id": author_id,
                "title": title,
            }),
        )
        .with_aggregate(post_id, "post")
    }

    /// Create a comment created event
    pub fn comment_created(
        comment_id: Uuid,
        post_id: Uuid,
        author_name: &str,
    ) -> DomainEvent {
        DomainEvent::new(
            COMMENT_CREATED,
            serde_json::json!({
                "comment_id": comment_id,
                "post_id": post_id,
                "author_name": author_name,
            }),
        )
        .with_aggregate(comment_id, "comment")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event() {
        let event = DomainEvent::new("test.event", serde_json::json!({"key": "value"}))
            .with_aggregate(Uuid::now_v7(), "test")
            .with_metadata("custom", serde_json::json!("data"));

        assert_eq!(event.event_type, "test.event");
        assert!(event.aggregate_id.is_some());
        assert!(event.metadata.data.contains_key("custom"));
    }

    #[test]
    fn test_event_type() {
        let et = EventType::new("user.created");
        assert_eq!(et.as_str(), "user.created");
        assert_eq!(et.to_string(), "user.created");
    }

    #[test]
    fn test_predefined_events() {
        let event = events::user_created(
            Uuid::now_v7(),
            "test@example.com",
            "testuser",
        );
        assert_eq!(event.event_type, events::USER_CREATED);
    }
}
