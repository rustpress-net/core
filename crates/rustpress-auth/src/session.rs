//! Session management.

use chrono::{DateTime, Duration, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: Uuid, token_hash: String, expires_in: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id,
            token_hash,
            ip_address: None,
            user_agent: None,
            data: HashMap::new(),
            last_active_at: now,
            expires_at: now + expires_in,
            created_at: now,
        }
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
        self.touch();
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<T: Serialize>(&mut self, key: impl Into<String>, value: T) {
        if let Ok(v) = serde_json::to_value(value) {
            self.data.insert(key.into(), v);
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn remaining_time(&self) -> Duration {
        let remaining = self.expires_at.signed_duration_since(Utc::now());
        if remaining < Duration::zero() {
            Duration::zero()
        } else {
            remaining
        }
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session lifetime
    pub lifetime: Duration,
    /// Extend session on activity
    pub extend_on_activity: bool,
    /// Minimum time between extensions
    pub extension_threshold: Duration,
    /// Maximum sessions per user
    pub max_sessions_per_user: Option<usize>,
    /// Cookie name
    pub cookie_name: String,
    /// Secure cookie
    pub secure: bool,
    /// HTTP only
    pub http_only: bool,
    /// Same site policy
    pub same_site: SameSite,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            lifetime: Duration::hours(24),
            extend_on_activity: true,
            extension_threshold: Duration::minutes(5),
            max_sessions_per_user: Some(5),
            cookie_name: "rustpress_session".to_string(),
            secure: true,
            http_only: true,
            same_site: SameSite::Lax,
        }
    }
}

/// Same site policy for cookies
#[derive(Debug, Clone, Copy)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

/// Session manager trait
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session
    async fn create(&self, session: Session) -> Result<Session>;

    /// Get session by token hash
    async fn get_by_token(&self, token_hash: &str) -> Result<Option<Session>>;

    /// Get session by ID
    async fn get(&self, id: Uuid) -> Result<Option<Session>>;

    /// Update session
    async fn update(&self, session: &Session) -> Result<()>;

    /// Delete session
    async fn delete(&self, id: Uuid) -> Result<()>;

    /// Delete session by token
    async fn delete_by_token(&self, token_hash: &str) -> Result<()>;

    /// Delete all sessions for a user
    async fn delete_user_sessions(&self, user_id: Uuid) -> Result<u64>;

    /// Get all sessions for a user
    async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>>;

    /// Delete expired sessions
    async fn cleanup_expired(&self) -> Result<u64>;
}

/// Session manager
pub struct SessionManager<S: SessionStore> {
    store: S,
    config: SessionConfig,
}

impl<S: SessionStore> SessionManager<S> {
    pub fn new(store: S, config: SessionConfig) -> Self {
        Self { store, config }
    }

    /// Create a new session for a user
    pub async fn create_session(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(Session, String)> {
        // Generate token
        let token = generate_session_token();
        let token_hash = hash_token(&token);

        let mut session = Session::new(user_id, token_hash, self.config.lifetime);

        if let Some(ip) = ip_address {
            session = session.with_ip(ip);
        }
        if let Some(ua) = user_agent {
            session = session.with_user_agent(ua);
        }

        // Check max sessions
        if let Some(max) = self.config.max_sessions_per_user {
            let existing = self.store.get_user_sessions(user_id).await?;
            if existing.len() >= max {
                // Delete oldest session
                if let Some(oldest) = existing.into_iter().min_by_key(|s| s.created_at) {
                    self.store.delete(oldest.id).await?;
                }
            }
        }

        let session = self.store.create(session).await?;
        Ok((session, token))
    }

    /// Validate and get session
    pub async fn validate(&self, token: &str) -> Result<Session> {
        let token_hash = hash_token(token);
        let session = self
            .store
            .get_by_token(&token_hash)
            .await?
            .ok_or_else(|| Error::Authentication {
                message: "Invalid session".to_string(),
            })?;

        if session.is_expired() {
            self.store.delete(session.id).await?;
            return Err(Error::TokenExpired);
        }

        // Extend session if needed
        if self.config.extend_on_activity {
            let since_last_active = Utc::now().signed_duration_since(session.last_active_at);
            if since_last_active > self.config.extension_threshold {
                let mut updated = session.clone();
                updated.extend(self.config.lifetime);
                self.store.update(&updated).await?;
                return Ok(updated);
            }
        }

        Ok(session)
    }

    /// Invalidate session
    pub async fn invalidate(&self, token: &str) -> Result<()> {
        let token_hash = hash_token(token);
        self.store.delete_by_token(&token_hash).await
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&self, user_id: Uuid) -> Result<u64> {
        self.store.delete_user_sessions(user_id).await
    }

    /// Get all sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>> {
        self.store.get_user_sessions(user_id).await
    }

    /// Cleanup expired sessions
    pub async fn cleanup(&self) -> Result<u64> {
        self.store.cleanup_expired().await
    }

    /// Get config
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// Generate a random session token
fn generate_session_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    base64_url_encode(&bytes)
}

/// Hash a session token
fn hash_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Base64 URL-safe encoding
fn base64_url_encode(bytes: &[u8]) -> String {
    use std::io::Write;
    let mut output = String::new();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    for chunk in bytes.chunks(3) {
        let n = match chunk.len() {
            1 => (chunk[0] as u32) << 16,
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
            _ => continue,
        };

        output.push(alphabet[((n >> 18) & 0x3F) as usize] as char);
        output.push(alphabet[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            output.push(alphabet[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(alphabet[(n & 0x3F) as usize] as char);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            Uuid::now_v7(),
            "token_hash".to_string(),
            Duration::hours(1),
        );

        assert!(!session.is_expired());
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_data() {
        let mut session = Session::new(
            Uuid::now_v7(),
            "token_hash".to_string(),
            Duration::hours(1),
        );

        session.set("key", "value");
        assert_eq!(session.get::<String>("key"), Some("value".to_string()));

        session.set("count", 42);
        assert_eq!(session.get::<i32>("count"), Some(42));

        session.remove("key");
        assert_eq!(session.get::<String>("key"), None);
    }

    #[test]
    fn test_session_expiry() {
        let session = Session::new(
            Uuid::now_v7(),
            "token_hash".to_string(),
            Duration::seconds(-1), // Already expired
        );

        assert!(session.is_expired());
        assert!(!session.is_valid());
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        assert!(!token1.is_empty());
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
        assert_ne!(hash_token("different"), hash1);
    }
}
