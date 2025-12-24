//! Cache key generation and management.

use std::fmt;

/// A cache key with namespace support
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    namespace: Option<String>,
    key: String,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            namespace: None,
            key: key.into(),
        }
    }

    /// Create a cache key with namespace
    pub fn with_namespace(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            key: key.into(),
        }
    }

    /// Create a tenant-scoped cache key
    pub fn tenant(tenant_id: impl fmt::Display, key: impl Into<String>) -> Self {
        Self::with_namespace(format!("tenant:{}", tenant_id), key)
    }

    /// Create a user-scoped cache key
    pub fn user(user_id: impl fmt::Display, key: impl Into<String>) -> Self {
        Self::with_namespace(format!("user:{}", user_id), key)
    }

    /// Create a session-scoped cache key
    pub fn session(session_id: impl fmt::Display, key: impl Into<String>) -> Self {
        Self::with_namespace(format!("session:{}", session_id), key)
    }

    /// Get the full cache key string
    pub fn as_str(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}:{}", ns, self.key),
            None => self.key.clone(),
        }
    }

    /// Add a prefix to the key
    pub fn prefix(&self, prefix: impl Into<String>) -> Self {
        Self {
            namespace: self.namespace.clone(),
            key: format!("{}:{}", prefix.into(), self.key),
        }
    }

    /// Add a suffix to the key
    pub fn suffix(&self, suffix: impl Into<String>) -> Self {
        Self {
            namespace: self.namespace.clone(),
            key: format!("{}:{}", self.key, suffix.into()),
        }
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for CacheKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for CacheKey {
    fn from(s: &String) -> Self {
        Self::new(s.clone())
    }
}

/// Builder for complex cache keys
pub struct CacheKeyBuilder {
    parts: Vec<String>,
    namespace: Option<String>,
}

impl CacheKeyBuilder {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            namespace: None,
        }
    }

    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    pub fn part(mut self, part: impl Into<String>) -> Self {
        self.parts.push(part.into());
        self
    }

    pub fn entity(self, entity_type: impl Into<String>, id: impl fmt::Display) -> Self {
        self.part(entity_type).part(id.to_string())
    }

    pub fn build(self) -> CacheKey {
        let key = self.parts.join(":");
        match self.namespace {
            Some(ns) => CacheKey::with_namespace(ns, key),
            None => CacheKey::new(key),
        }
    }
}

impl Default for CacheKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined cache key patterns
pub mod keys {
    use super::*;

    /// Post cache key
    pub fn post(id: impl fmt::Display) -> CacheKey {
        CacheKey::new(format!("post:{}", id))
    }

    /// Post by slug cache key
    pub fn post_by_slug(slug: &str) -> CacheKey {
        CacheKey::new(format!("post:slug:{}", slug))
    }

    /// Page cache key
    pub fn page(id: impl fmt::Display) -> CacheKey {
        CacheKey::new(format!("page:{}", id))
    }

    /// User cache key
    pub fn user(id: impl fmt::Display) -> CacheKey {
        CacheKey::new(format!("user:{}", id))
    }

    /// User by email cache key
    pub fn user_by_email(email: &str) -> CacheKey {
        CacheKey::new(format!("user:email:{}", email))
    }

    /// Options cache key
    pub fn options() -> CacheKey {
        CacheKey::new("options")
    }

    /// Single option cache key
    pub fn option(name: &str) -> CacheKey {
        CacheKey::new(format!("option:{}", name))
    }

    /// Menu cache key
    pub fn menu(id: impl fmt::Display) -> CacheKey {
        CacheKey::new(format!("menu:{}", id))
    }

    /// Taxonomy terms cache key
    pub fn taxonomy_terms(taxonomy: &str) -> CacheKey {
        CacheKey::new(format!("taxonomy:{}:terms", taxonomy))
    }

    /// Rate limit cache key
    pub fn rate_limit(identifier: &str) -> CacheKey {
        CacheKey::new(format!("rate_limit:{}", identifier))
    }

    /// Session cache key
    pub fn session(token_hash: &str) -> CacheKey {
        CacheKey::new(format!("session:{}", token_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let key = CacheKey::new("test");
        assert_eq!(key.as_str(), "test");
    }

    #[test]
    fn test_cache_key_with_namespace() {
        let key = CacheKey::with_namespace("ns", "test");
        assert_eq!(key.as_str(), "ns:test");
    }

    #[test]
    fn test_cache_key_tenant() {
        let key = CacheKey::tenant("123", "posts");
        assert_eq!(key.as_str(), "tenant:123:posts");
    }

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new()
            .namespace("app")
            .entity("post", "123")
            .part("comments")
            .build();
        assert_eq!(key.as_str(), "app:post:123:comments");
    }

    #[test]
    fn test_predefined_keys() {
        assert_eq!(keys::post("123").as_str(), "post:123");
        assert_eq!(keys::user_by_email("test@example.com").as_str(), "user:email:test@example.com");
    }
}
