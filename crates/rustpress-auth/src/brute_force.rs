//! Brute Force Protection (Points 69, 77)
//!
//! Protection against brute force attacks with exponential backoff
//! and account lockout after failed attempts.

use chrono::{DateTime, Duration, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Login attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub id: Uuid,
    pub identifier: String, // username, email, or IP
    pub identifier_type: IdentifierType,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

/// Identifier type for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierType {
    Username,
    Email,
    IpAddress,
    UserId,
}

/// Lockout status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutStatus {
    pub is_locked: bool,
    pub failed_attempts: u32,
    pub max_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub remaining_attempts: u32,
    pub next_lockout_duration: Duration,
}

impl LockoutStatus {
    pub fn unlocked(failed_attempts: u32, max_attempts: u32) -> Self {
        Self {
            is_locked: false,
            failed_attempts,
            max_attempts,
            locked_until: None,
            remaining_attempts: max_attempts.saturating_sub(failed_attempts),
            next_lockout_duration: Duration::zero(),
        }
    }

    pub fn locked(failed_attempts: u32, max_attempts: u32, until: DateTime<Utc>, next_duration: Duration) -> Self {
        Self {
            is_locked: true,
            failed_attempts,
            max_attempts,
            locked_until: Some(until),
            remaining_attempts: 0,
            next_lockout_duration: next_duration,
        }
    }

    pub fn seconds_until_unlock(&self) -> Option<i64> {
        self.locked_until.map(|until| {
            (until - Utc::now()).num_seconds().max(0)
        })
    }
}

/// Brute force protection configuration
#[derive(Debug, Clone)]
pub struct BruteForceConfig {
    /// Maximum failed attempts before lockout
    pub max_attempts: u32,
    /// Time window for counting attempts (seconds)
    pub window_seconds: u64,
    /// Initial lockout duration (seconds)
    pub initial_lockout_seconds: u64,
    /// Maximum lockout duration (seconds)
    pub max_lockout_seconds: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Clear attempts after successful login
    pub clear_on_success: bool,
    /// Track by IP in addition to username
    pub track_ip: bool,
    /// IP-specific settings
    pub ip_max_attempts: u32,
    /// Notify on lockout
    pub notify_on_lockout: bool,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_seconds: 900, // 15 minutes
            initial_lockout_seconds: 60, // 1 minute
            max_lockout_seconds: 3600, // 1 hour
            backoff_multiplier: 2.0,
            clear_on_success: true,
            track_ip: true,
            ip_max_attempts: 20,
            notify_on_lockout: true,
        }
    }
}

/// Brute force storage trait
#[async_trait::async_trait]
pub trait BruteForceStore: Send + Sync {
    /// Record a login attempt
    async fn record_attempt(&self, attempt: &LoginAttempt) -> Result<()>;

    /// Get failed attempts count in window
    async fn get_failed_count(&self, identifier: &str, window_start: DateTime<Utc>) -> Result<u32>;

    /// Get lockout info
    async fn get_lockout(&self, identifier: &str) -> Result<Option<(DateTime<Utc>, u32)>>;

    /// Set lockout
    async fn set_lockout(&self, identifier: &str, until: DateTime<Utc>, attempt_count: u32) -> Result<()>;

    /// Clear lockout
    async fn clear_lockout(&self, identifier: &str) -> Result<()>;

    /// Clear failed attempts
    async fn clear_attempts(&self, identifier: &str) -> Result<()>;

    /// Get recent attempts for an identifier
    async fn get_recent_attempts(&self, identifier: &str, limit: usize) -> Result<Vec<LoginAttempt>>;
}

/// Brute force protection manager
pub struct BruteForceProtection<S: BruteForceStore> {
    store: S,
    config: BruteForceConfig,
}

impl<S: BruteForceStore> BruteForceProtection<S> {
    pub fn new(store: S, config: BruteForceConfig) -> Self {
        Self { store, config }
    }

    /// Check if identifier is currently locked out
    pub async fn check_lockout(&self, identifier: &str) -> Result<LockoutStatus> {
        // Check for active lockout
        if let Some((locked_until, attempt_count)) = self.store.get_lockout(identifier).await? {
            if Utc::now() < locked_until {
                let next_duration = self.calculate_next_lockout_duration(attempt_count);
                return Ok(LockoutStatus::locked(
                    attempt_count,
                    self.config.max_attempts,
                    locked_until,
                    next_duration,
                ));
            } else {
                // Lockout expired, clear it
                self.store.clear_lockout(identifier).await?;
            }
        }

        // Count recent failed attempts
        let window_start = Utc::now() - Duration::seconds(self.config.window_seconds as i64);
        let failed_count = self.store.get_failed_count(identifier, window_start).await?;

        Ok(LockoutStatus::unlocked(failed_count, self.config.max_attempts))
    }

    /// Record a failed login attempt
    pub async fn record_failure(
        &self,
        identifier: &str,
        identifier_type: IdentifierType,
        ip_address: &str,
        user_agent: Option<&str>,
        reason: Option<&str>,
    ) -> Result<LockoutStatus> {
        let attempt = LoginAttempt {
            id: Uuid::now_v7(),
            identifier: identifier.to_string(),
            identifier_type,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.map(String::from),
            success: false,
            failure_reason: reason.map(String::from),
            attempted_at: Utc::now(),
        };

        self.store.record_attempt(&attempt).await?;

        // Also track IP if enabled
        if self.config.track_ip && identifier != ip_address {
            let ip_attempt = LoginAttempt {
                id: Uuid::now_v7(),
                identifier: ip_address.to_string(),
                identifier_type: IdentifierType::IpAddress,
                ip_address: ip_address.to_string(),
                user_agent: user_agent.map(String::from),
                success: false,
                failure_reason: reason.map(String::from),
                attempted_at: Utc::now(),
            };
            self.store.record_attempt(&ip_attempt).await?;
        }

        // Check if we should lock out
        let window_start = Utc::now() - Duration::seconds(self.config.window_seconds as i64);
        let failed_count = self.store.get_failed_count(identifier, window_start).await?;

        if failed_count >= self.config.max_attempts {
            let lockout_duration = self.calculate_lockout_duration(failed_count);
            let locked_until = Utc::now() + lockout_duration;

            self.store.set_lockout(identifier, locked_until, failed_count).await?;

            let next_duration = self.calculate_next_lockout_duration(failed_count);
            return Ok(LockoutStatus::locked(
                failed_count,
                self.config.max_attempts,
                locked_until,
                next_duration,
            ));
        }

        Ok(LockoutStatus::unlocked(failed_count, self.config.max_attempts))
    }

    /// Record a successful login
    pub async fn record_success(
        &self,
        identifier: &str,
        identifier_type: IdentifierType,
        ip_address: &str,
        user_agent: Option<&str>,
    ) -> Result<()> {
        let attempt = LoginAttempt {
            id: Uuid::now_v7(),
            identifier: identifier.to_string(),
            identifier_type,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.map(String::from),
            success: true,
            failure_reason: None,
            attempted_at: Utc::now(),
        };

        self.store.record_attempt(&attempt).await?;

        if self.config.clear_on_success {
            self.store.clear_lockout(identifier).await?;
            self.store.clear_attempts(identifier).await?;
        }

        Ok(())
    }

    /// Calculate lockout duration with exponential backoff
    fn calculate_lockout_duration(&self, attempt_count: u32) -> Duration {
        let excess = attempt_count.saturating_sub(self.config.max_attempts);
        let multiplier = self.config.backoff_multiplier.powi(excess as i32);
        let seconds = (self.config.initial_lockout_seconds as f64 * multiplier) as u64;
        let capped = seconds.min(self.config.max_lockout_seconds);
        Duration::seconds(capped as i64)
    }

    /// Calculate next lockout duration (for warning)
    fn calculate_next_lockout_duration(&self, current_attempts: u32) -> Duration {
        self.calculate_lockout_duration(current_attempts + 1)
    }

    /// Manually unlock an identifier
    pub async fn unlock(&self, identifier: &str) -> Result<()> {
        self.store.clear_lockout(identifier).await?;
        self.store.clear_attempts(identifier).await?;
        Ok(())
    }

    /// Get recent login attempts
    pub async fn get_recent_attempts(&self, identifier: &str, limit: usize) -> Result<Vec<LoginAttempt>> {
        self.store.get_recent_attempts(identifier, limit).await
    }

    /// Check multiple identifiers (username + IP)
    pub async fn check_multiple(&self, username: &str, ip: &str) -> Result<LockoutStatus> {
        let user_status = self.check_lockout(username).await?;
        if user_status.is_locked {
            return Ok(user_status);
        }

        if self.config.track_ip {
            let ip_status = self.check_lockout(ip).await?;
            if ip_status.is_locked {
                return Ok(ip_status);
            }
        }

        Ok(user_status)
    }
}

/// In-memory brute force store
pub struct InMemoryBruteForceStore {
    attempts: RwLock<Vec<LoginAttempt>>,
    lockouts: RwLock<HashMap<String, (DateTime<Utc>, u32)>>,
}

impl InMemoryBruteForceStore {
    pub fn new() -> Self {
        Self {
            attempts: RwLock::new(Vec::new()),
            lockouts: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryBruteForceStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BruteForceStore for InMemoryBruteForceStore {
    async fn record_attempt(&self, attempt: &LoginAttempt) -> Result<()> {
        let mut attempts = self.attempts.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        attempts.push(attempt.clone());
        Ok(())
    }

    async fn get_failed_count(&self, identifier: &str, window_start: DateTime<Utc>) -> Result<u32> {
        let attempts = self.attempts.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let count = attempts
            .iter()
            .filter(|a| {
                a.identifier == identifier
                    && !a.success
                    && a.attempted_at >= window_start
            })
            .count() as u32;

        Ok(count)
    }

    async fn get_lockout(&self, identifier: &str) -> Result<Option<(DateTime<Utc>, u32)>> {
        let lockouts = self.lockouts.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        Ok(lockouts.get(identifier).cloned())
    }

    async fn set_lockout(&self, identifier: &str, until: DateTime<Utc>, attempt_count: u32) -> Result<()> {
        let mut lockouts = self.lockouts.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        lockouts.insert(identifier.to_string(), (until, attempt_count));
        Ok(())
    }

    async fn clear_lockout(&self, identifier: &str) -> Result<()> {
        let mut lockouts = self.lockouts.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        lockouts.remove(identifier);
        Ok(())
    }

    async fn clear_attempts(&self, identifier: &str) -> Result<()> {
        let mut attempts = self.attempts.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;
        attempts.retain(|a| a.identifier != identifier);
        Ok(())
    }

    async fn get_recent_attempts(&self, identifier: &str, limit: usize) -> Result<Vec<LoginAttempt>> {
        let attempts = self.attempts.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let mut filtered: Vec<_> = attempts
            .iter()
            .filter(|a| a.identifier == identifier)
            .cloned()
            .collect();

        filtered.sort_by(|a, b| b.attempted_at.cmp(&a.attempted_at));
        filtered.truncate(limit);

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allows_initial_attempts() {
        let store = InMemoryBruteForceStore::new();
        let protection = BruteForceProtection::new(store, BruteForceConfig::default());

        let status = protection.check_lockout("user@example.com").await.unwrap();
        assert!(!status.is_locked);
        assert_eq!(status.remaining_attempts, 5);
    }

    #[tokio::test]
    async fn test_locks_out_after_max_attempts() {
        let store = InMemoryBruteForceStore::new();
        let config = BruteForceConfig {
            max_attempts: 3,
            ..Default::default()
        };
        let protection = BruteForceProtection::new(store, config);

        // Record failures
        for _ in 0..3 {
            protection
                .record_failure("user@example.com", IdentifierType::Email, "1.2.3.4", None, None)
                .await
                .unwrap();
        }

        let status = protection.check_lockout("user@example.com").await.unwrap();
        assert!(status.is_locked);
        assert!(status.locked_until.is_some());
    }

    #[tokio::test]
    async fn test_clears_on_success() {
        let store = InMemoryBruteForceStore::new();
        let config = BruteForceConfig {
            max_attempts: 5,
            clear_on_success: true,
            ..Default::default()
        };
        let protection = BruteForceProtection::new(store, config);

        // Record some failures
        for _ in 0..3 {
            protection
                .record_failure("user@example.com", IdentifierType::Email, "1.2.3.4", None, None)
                .await
                .unwrap();
        }

        // Success should clear
        protection
            .record_success("user@example.com", IdentifierType::Email, "1.2.3.4", None)
            .await
            .unwrap();

        let status = protection.check_lockout("user@example.com").await.unwrap();
        assert!(!status.is_locked);
        assert_eq!(status.failed_attempts, 0);
    }
}
