//! # Session Management
//!
//! User session tracking and management.
//!
//! Features:
//! - Session creation and validation
//! - Session token generation
//! - Multiple device sessions
//! - Session expiration
//! - Active sessions listing
//! - Session revocation
//! - Device fingerprinting

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use uuid::Uuid;

/// Session token type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    /// Token ID (stored in database)
    pub id: Uuid,

    /// The actual token value (sent to client)
    pub token: String,

    /// Token hash (stored in database)
    pub token_hash: String,
}

impl SessionToken {
    /// Generate a new session token
    pub fn generate() -> Self {
        let token = generate_secure_token(64);
        let token_hash = hash_token(&token);

        Self {
            id: Uuid::new_v4(),
            token,
            token_hash,
        }
    }

    /// Verify a token against its hash
    pub fn verify(token: &str, hash: &str) -> bool {
        hash_token(token) == hash
    }
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: Uuid,

    /// User ID
    pub user_id: i64,

    /// Token hash
    pub token_hash: String,

    /// User agent string
    pub user_agent: Option<String>,

    /// Parsed device info
    pub device: DeviceInfo,

    /// IP address
    pub ip_address: String,

    /// Location (from IP)
    pub location: Option<String>,

    /// Whether this is the current session
    pub is_current: bool,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,

    /// Session expires at
    pub expires_at: DateTime<Utc>,

    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Device information parsed from user agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device type (desktop, mobile, tablet)
    pub device_type: String,

    /// Browser name
    pub browser: Option<String>,

    /// Browser version
    pub browser_version: Option<String>,

    /// Operating system
    pub os: Option<String>,

    /// OS version
    pub os_version: Option<String>,

    /// Device name/model
    pub device_name: Option<String>,
}

impl DeviceInfo {
    /// Parse device info from user agent
    pub fn from_user_agent(user_agent: &str) -> Self {
        let ua = user_agent.to_lowercase();

        let device_type = if ua.contains("mobile") || ua.contains("android") && !ua.contains("tablet") {
            "mobile"
        } else if ua.contains("tablet") || ua.contains("ipad") {
            "tablet"
        } else {
            "desktop"
        }.to_string();

        let browser = if ua.contains("chrome") && !ua.contains("edg") {
            Some("Chrome".to_string())
        } else if ua.contains("firefox") {
            Some("Firefox".to_string())
        } else if ua.contains("safari") && !ua.contains("chrome") {
            Some("Safari".to_string())
        } else if ua.contains("edg") {
            Some("Edge".to_string())
        } else if ua.contains("opera") || ua.contains("opr") {
            Some("Opera".to_string())
        } else {
            None
        };

        let os = if ua.contains("windows") {
            Some("Windows".to_string())
        } else if ua.contains("mac os") || ua.contains("macos") {
            Some("macOS".to_string())
        } else if ua.contains("linux") && !ua.contains("android") {
            Some("Linux".to_string())
        } else if ua.contains("android") {
            Some("Android".to_string())
        } else if ua.contains("iphone") || ua.contains("ipad") || ua.contains("ios") {
            Some("iOS".to_string())
        } else {
            None
        };

        Self {
            device_type,
            browser,
            browser_version: None,
            os,
            os_version: None,
            device_name: None,
        }
    }

    /// Get a display name for the device
    pub fn display_name(&self) -> String {
        let browser = self.browser.as_deref().unwrap_or("Unknown Browser");
        let os = self.os.as_deref().unwrap_or("Unknown OS");
        format!("{} on {}", browser, os)
    }
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: i64,
        token_hash: &str,
        user_agent: Option<&str>,
        ip: &str,
        duration: Duration,
    ) -> Self {
        let now = Utc::now();
        let device = user_agent
            .map(DeviceInfo::from_user_agent)
            .unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            user_id,
            token_hash: token_hash.to_string(),
            user_agent: user_agent.map(|s| s.to_string()),
            device,
            ip_address: ip.to_string(),
            location: None,
            is_current: false,
            last_activity_at: now,
            expires_at: now + duration,
            created_at: now,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Update last activity
    pub fn touch(&mut self) {
        self.last_activity_at = Utc::now();
    }

    /// Extend session
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
        self.last_activity_at = Utc::now();
    }
}

/// Session settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Session duration (seconds)
    pub session_duration_seconds: i64,

    /// Remember me duration (seconds)
    pub remember_me_duration_seconds: i64,

    /// Idle timeout (seconds, 0 = disabled)
    pub idle_timeout_seconds: i64,

    /// Maximum concurrent sessions (0 = unlimited)
    pub max_sessions: usize,

    /// Invalidate other sessions on password change
    pub invalidate_on_password_change: bool,

    /// Extend session on activity
    pub extend_on_activity: bool,

    /// Cookie name
    pub cookie_name: String,

    /// Cookie secure flag
    pub cookie_secure: bool,

    /// Cookie HTTP only
    pub cookie_http_only: bool,

    /// Cookie same site
    pub cookie_same_site: String,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            session_duration_seconds: 24 * 60 * 60, // 24 hours
            remember_me_duration_seconds: 30 * 24 * 60 * 60, // 30 days
            idle_timeout_seconds: 60 * 60, // 1 hour
            max_sessions: 5,
            invalidate_on_password_change: true,
            extend_on_activity: true,
            cookie_name: "rustpress_session".to_string(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: "Lax".to_string(),
        }
    }
}

/// Session manager
pub struct SessionManager {
    /// Sessions by ID
    sessions: HashMap<Uuid, Session>,

    /// Sessions by user ID
    user_sessions: HashMap<i64, Vec<Uuid>>,

    /// Sessions by token hash
    sessions_by_token: HashMap<String, Uuid>,

    /// Settings
    settings: SessionSettings,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            sessions_by_token: HashMap::new(),
            settings: SessionSettings::default(),
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_settings(mut self, settings: SessionSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        user_id: i64,
        user_agent: Option<&str>,
        ip: &str,
        remember_me: bool,
    ) -> Result<(Session, String), String> {
        // Check max sessions
        if self.settings.max_sessions > 0 {
            let current_sessions = self.get_user_sessions(user_id);
            if current_sessions.len() >= self.settings.max_sessions {
                // Remove oldest session
                if let Some(oldest) = current_sessions.iter()
                    .min_by_key(|s| s.created_at)
                {
                    let oldest_id = oldest.id;
                    self.revoke_session(oldest_id)?;
                }
            }
        }

        // Generate token
        let token = SessionToken::generate();

        // Calculate duration
        let duration = if remember_me {
            Duration::seconds(self.settings.remember_me_duration_seconds)
        } else {
            Duration::seconds(self.settings.session_duration_seconds)
        };

        // Create session
        let session = Session::new(user_id, &token.token_hash, user_agent, ip, duration);
        let session_id = session.id;

        // Store session
        self.sessions.insert(session_id, session.clone());
        self.user_sessions.entry(user_id)
            .or_insert_with(Vec::new)
            .push(session_id);
        self.sessions_by_token.insert(token.token_hash.clone(), session_id);

        Ok((session, token.token))
    }

    /// Validate a session token
    pub fn validate_token(&mut self, token: &str) -> Option<&Session> {
        let token_hash = hash_token(token);
        let session_id = self.sessions_by_token.get(&token_hash)?;
        let session = self.sessions.get_mut(session_id)?;

        if session.is_expired() {
            return None;
        }

        // Check idle timeout
        if self.settings.idle_timeout_seconds > 0 {
            let idle_duration = Duration::seconds(self.settings.idle_timeout_seconds);
            if Utc::now() - session.last_activity_at > idle_duration {
                return None;
            }
        }

        // Update activity
        session.touch();

        // Extend session if configured
        if self.settings.extend_on_activity {
            let duration = Duration::seconds(self.settings.session_duration_seconds);
            session.extend(duration);
        }

        Some(session)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: Uuid) -> Option<&Session> {
        self.sessions.get(&session_id)
    }

    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: i64) -> Vec<&Session> {
        self.user_sessions.get(&user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.sessions.get(id))
                    .filter(|s| s.is_valid())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Revoke a specific session
    pub fn revoke_session(&mut self, session_id: Uuid) -> Result<(), String> {
        let session = self.sessions.remove(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Remove from user sessions
        if let Some(user_sessions) = self.user_sessions.get_mut(&session.user_id) {
            user_sessions.retain(|id| *id != session_id);
        }

        // Remove from token index
        self.sessions_by_token.remove(&session.token_hash);

        Ok(())
    }

    /// Revoke all sessions for a user
    pub fn revoke_all_user_sessions(&mut self, user_id: i64) -> usize {
        let session_ids: Vec<Uuid> = self.user_sessions.remove(&user_id)
            .unwrap_or_default();

        let count = session_ids.len();

        for session_id in session_ids {
            if let Some(session) = self.sessions.remove(&session_id) {
                self.sessions_by_token.remove(&session.token_hash);
            }
        }

        count
    }

    /// Revoke all sessions except current
    pub fn revoke_other_sessions(&mut self, user_id: i64, current_session_id: Uuid) -> usize {
        let session_ids: Vec<Uuid> = self.user_sessions.get(&user_id)
            .map(|ids| ids.iter().filter(|id| **id != current_session_id).cloned().collect())
            .unwrap_or_default();

        let count = session_ids.len();

        for session_id in session_ids {
            let _ = self.revoke_session(session_id);
        }

        count
    }

    /// Mark a session as current
    pub fn mark_current(&mut self, session_id: Uuid, user_id: i64) {
        // First, unmark all other sessions for this user
        if let Some(session_ids) = self.user_sessions.get(&user_id) {
            for id in session_ids {
                if let Some(session) = self.sessions.get_mut(id) {
                    session.is_current = *id == session_id;
                }
            }
        }
    }

    /// Cleanup expired sessions
    pub fn cleanup_expired(&mut self) -> usize {
        let expired: Vec<Uuid> = self.sessions.iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| *id)
            .collect();

        let count = expired.len();

        for session_id in expired {
            let _ = self.revoke_session(session_id);
        }

        count
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_valid()).count()
    }

    /// Get settings
    pub fn settings(&self) -> &SessionSettings {
        &self.settings
    }
}

/// Generate a cryptographically secure random token
fn generate_secure_token(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

/// Hash a token using SHA-256
fn hash_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    data_encoding::HEXLOWER.encode(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_token() {
        let token = SessionToken::generate();
        assert!(!token.token.is_empty());
        assert!(!token.token_hash.is_empty());
        assert!(SessionToken::verify(&token.token, &token.token_hash));
    }

    #[test]
    fn test_device_info() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/91.0";
        let device = DeviceInfo::from_user_agent(ua);

        assert_eq!(device.device_type, "desktop");
        assert_eq!(device.browser, Some("Chrome".to_string()));
        assert_eq!(device.os, Some("Windows".to_string()));
    }

    #[test]
    fn test_session_creation() {
        let mut manager = SessionManager::new();
        let (session, token) = manager.create_session(
            1,
            Some("Mozilla/5.0"),
            "127.0.0.1",
            false,
        ).unwrap();

        assert_eq!(session.user_id, 1);
        assert!(!token.is_empty());
        assert!(session.is_valid());
    }

    #[test]
    fn test_session_validation() {
        let mut manager = SessionManager::new();
        let (_, token) = manager.create_session(1, None, "127.0.0.1", false).unwrap();

        let session = manager.validate_token(&token);
        assert!(session.is_some());

        let invalid = manager.validate_token("invalid_token");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_session_revocation() {
        let mut manager = SessionManager::new();
        let (session, _) = manager.create_session(1, None, "127.0.0.1", false).unwrap();

        assert!(manager.revoke_session(session.id).is_ok());
        assert!(manager.get_session(session.id).is_none());
    }

    #[test]
    fn test_max_sessions() {
        let settings = SessionSettings {
            max_sessions: 2,
            ..Default::default()
        };

        let mut manager = SessionManager::new().with_settings(settings);

        manager.create_session(1, None, "127.0.0.1", false).unwrap();
        manager.create_session(1, None, "127.0.0.1", false).unwrap();
        manager.create_session(1, None, "127.0.0.1", false).unwrap();

        // Should only have 2 sessions
        assert_eq!(manager.get_user_sessions(1).len(), 2);
    }
}
