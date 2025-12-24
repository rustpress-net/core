//! ID types for entities in RustPress.
//!
//! Uses UUID v7 for time-ordered identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use uuid::Uuid;

/// A type-safe ID wrapper that provides compile-time safety for entity references.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<T> {
    inner: Uuid,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    /// Create a new random ID using UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self {
            inner: Uuid::now_v7(),
            _marker: PhantomData,
        }
    }

    /// Create an ID from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            inner: uuid,
            _marker: PhantomData,
        }
    }

    /// Get the inner UUID
    pub fn into_uuid(self) -> Uuid {
        self.inner
    }

    /// Get a reference to the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.inner
    }

    /// Check if this is a nil (zero) UUID
    pub fn is_nil(&self) -> bool {
        self.inner.is_nil()
    }

    /// Get timestamp from UUID v7 (returns None if not v7)
    pub fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        let (secs, nanos) = self.inner.get_timestamp()?.to_unix();
        chrono::DateTime::from_timestamp(secs as i64, nanos)
    }

    /// Create a nil ID
    pub fn nil() -> Self {
        Self {
            inner: Uuid::nil(),
            _marker: PhantomData,
        }
    }

    /// Cast to a different entity type (use with caution)
    pub fn cast<U>(self) -> Id<U> {
        Id {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({})", self.inner)
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T> FromStr for Id<T> {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: Uuid::parse_str(s)?,
            _marker: PhantomData,
        })
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uuid = Uuid::deserialize(deserializer)?;
        Ok(Self {
            inner: uuid,
            _marker: PhantomData,
        })
    }
}

impl<T> From<Uuid> for Id<T> {
    fn from(uuid: Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

impl<T> From<Id<T>> for Uuid {
    fn from(id: Id<T>) -> Self {
        id.inner
    }
}

impl<T> AsRef<Uuid> for Id<T> {
    fn as_ref(&self) -> &Uuid {
        &self.inner
    }
}

/// Marker traits for entity types
pub mod entities {
    /// User entity marker
    #[derive(Clone, Copy)]
    pub struct User;
    /// Post entity marker
    #[derive(Clone, Copy)]
    pub struct Post;
    /// Page entity marker
    #[derive(Clone, Copy)]
    pub struct Page;
    /// Comment entity marker
    #[derive(Clone, Copy)]
    pub struct Comment;
    /// Media entity marker
    #[derive(Clone, Copy)]
    pub struct Media;
    /// Taxonomy entity marker
    #[derive(Clone, Copy)]
    pub struct Taxonomy;
    /// Term entity marker
    #[derive(Clone, Copy)]
    pub struct Term;
    /// Menu entity marker
    #[derive(Clone, Copy)]
    pub struct Menu;
    /// Option entity marker
    #[derive(Clone, Copy)]
    pub struct Option;
    /// Plugin entity marker
    #[derive(Clone, Copy)]
    pub struct Plugin;
    /// Theme entity marker
    #[derive(Clone, Copy)]
    pub struct Theme;
    /// Tenant entity marker
    #[derive(Clone, Copy)]
    pub struct Tenant;
    /// Role entity marker
    #[derive(Clone, Copy)]
    pub struct Role;
    /// Permission entity marker
    #[derive(Clone, Copy)]
    pub struct Permission;
    /// Session entity marker
    #[derive(Clone, Copy)]
    pub struct Session;
    /// Job entity marker
    #[derive(Clone, Copy)]
    pub struct Job;
    /// Webhook entity marker
    #[derive(Clone, Copy)]
    pub struct Webhook;
}

// Type aliases for common entity IDs
pub type UserId = Id<entities::User>;
pub type PostId = Id<entities::Post>;
pub type PageId = Id<entities::Page>;
pub type CommentId = Id<entities::Comment>;
pub type MediaId = Id<entities::Media>;
pub type TaxonomyId = Id<entities::Taxonomy>;
pub type TermId = Id<entities::Term>;
pub type MenuId = Id<entities::Menu>;
pub type OptionId = Id<entities::Option>;
pub type PluginId = Id<entities::Plugin>;
pub type ThemeId = Id<entities::Theme>;
pub type TenantId = Id<entities::Tenant>;
pub type RoleId = Id<entities::Role>;
pub type PermissionId = Id<entities::Permission>;
pub type SessionId = Id<entities::Session>;
pub type JobId = Id<entities::Job>;
pub type WebhookId = Id<entities::Webhook>;

/// A generic entity ID without type safety (for dynamic scenarios)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for EntityId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl<T> From<Id<T>> for EntityId {
    fn from(id: Id<T>) -> Self {
        Self(id.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id1: UserId = Id::new();
        let id2: UserId = Id::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_type_safety() {
        let user_id: UserId = Id::new();
        let _post_id: PostId = Id::new();

        // This should work - same types
        let _: UserId = user_id;

        // These would not compile - different types
        // let _: PostId = user_id;
    }

    #[test]
    fn test_id_parsing() {
        let id: UserId = Id::new();
        let s = id.to_string();
        let parsed: UserId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_serialization() {
        let id: UserId = Id::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_uuid_v7_ordering() {
        let id1: UserId = Id::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2: UserId = Id::new();

        // UUID v7 should be time-ordered
        assert!(id1.into_uuid() < id2.into_uuid());
    }
}
