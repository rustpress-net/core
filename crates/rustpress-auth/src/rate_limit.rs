//! Rate Limiting (Point 68)
//!
//! Per-user/IP rate limiting for security.

use chrono::{DateTime, Duration, Utc};
use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
    /// Whether to use sliding window
    pub sliding_window: bool,
    /// Burst allowance (extra requests)
    pub burst_size: u32,
    /// Key prefix for identification
    pub key_prefix: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            sliding_window: true,
            burst_size: 10,
            key_prefix: "rate_limit".to_string(),
        }
    }
}

impl RateLimitConfig {
    /// Strict rate limit for auth endpoints
    pub fn auth_strict() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 60,
            sliding_window: true,
            burst_size: 0,
            key_prefix: "rate_limit:auth".to_string(),
        }
    }

    /// Moderate rate limit for API endpoints
    pub fn api_moderate() -> Self {
        Self {
            max_requests: 60,
            window_seconds: 60,
            sliding_window: true,
            burst_size: 10,
            key_prefix: "rate_limit:api".to_string(),
        }
    }

    /// Relaxed rate limit for public endpoints
    pub fn public_relaxed() -> Self {
        Self {
            max_requests: 300,
            window_seconds: 60,
            sliding_window: true,
            burst_size: 50,
            key_prefix: "rate_limit:public".to_string(),
        }
    }
}

/// Rate limit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining: u32,
    /// Total limit
    pub limit: u32,
    /// When the window resets
    pub reset_at: DateTime<Utc>,
    /// Retry after (seconds) if not allowed
    pub retry_after: Option<u64>,
}

impl RateLimitResult {
    pub fn allowed(remaining: u32, limit: u32, reset_at: DateTime<Utc>) -> Self {
        Self {
            allowed: true,
            remaining,
            limit,
            reset_at,
            retry_after: None,
        }
    }

    pub fn denied(limit: u32, reset_at: DateTime<Utc>, retry_after: u64) -> Self {
        Self {
            allowed: false,
            remaining: 0,
            limit,
            reset_at,
            retry_after: Some(retry_after),
        }
    }
}

/// Rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: DateTime<Utc>,
    requests: Vec<DateTime<Utc>>, // For sliding window
}

/// Rate limiter storage trait
#[async_trait::async_trait]
pub trait RateLimitStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<(u32, DateTime<Utc>)>>;
    async fn increment(&self, key: &str, window_seconds: u64) -> Result<(u32, DateTime<Utc>)>;
    async fn get_sliding(&self, key: &str, window_seconds: u64) -> Result<Vec<DateTime<Utc>>>;
    async fn add_request(&self, key: &str, window_seconds: u64) -> Result<Vec<DateTime<Utc>>>;
}

/// Rate limiter
pub struct RateLimiter<S: RateLimitStore> {
    store: S,
    config: RateLimitConfig,
}

impl<S: RateLimitStore> RateLimiter<S> {
    pub fn new(store: S, config: RateLimitConfig) -> Self {
        Self { store, config }
    }

    /// Check and record a request
    pub async fn check(&self, identifier: &str) -> Result<RateLimitResult> {
        let key = format!("{}:{}", self.config.key_prefix, identifier);
        let total_limit = self.config.max_requests + self.config.burst_size;

        if self.config.sliding_window {
            self.check_sliding(&key, total_limit).await
        } else {
            self.check_fixed(&key, total_limit).await
        }
    }

    /// Fixed window rate limiting
    async fn check_fixed(&self, key: &str, limit: u32) -> Result<RateLimitResult> {
        let (count, window_start) = self.store
            .increment(key, self.config.window_seconds)
            .await?;

        let reset_at = window_start + Duration::seconds(self.config.window_seconds as i64);

        if count > limit {
            let retry_after = (reset_at - Utc::now()).num_seconds().max(0) as u64;
            return Ok(RateLimitResult::denied(limit, reset_at, retry_after));
        }

        let remaining = limit.saturating_sub(count);
        Ok(RateLimitResult::allowed(remaining, limit, reset_at))
    }

    /// Sliding window rate limiting
    async fn check_sliding(&self, key: &str, limit: u32) -> Result<RateLimitResult> {
        let requests = self.store
            .add_request(key, self.config.window_seconds)
            .await?;

        let count = requests.len() as u32;
        let reset_at = Utc::now() + Duration::seconds(self.config.window_seconds as i64);

        if count > limit {
            // Find oldest request in window to calculate retry_after
            let oldest = requests.first().cloned().unwrap_or(Utc::now());
            let retry_after = ((oldest + Duration::seconds(self.config.window_seconds as i64)) - Utc::now())
                .num_seconds()
                .max(0) as u64;
            return Ok(RateLimitResult::denied(limit, reset_at, retry_after));
        }

        let remaining = limit.saturating_sub(count);
        Ok(RateLimitResult::allowed(remaining, limit, reset_at))
    }

    /// Check without recording (dry run)
    pub async fn peek(&self, identifier: &str) -> Result<RateLimitResult> {
        let key = format!("{}:{}", self.config.key_prefix, identifier);
        let total_limit = self.config.max_requests + self.config.burst_size;

        let (count, window_start) = self.store
            .get(&key)
            .await?
            .unwrap_or((0, Utc::now()));

        let reset_at = window_start + Duration::seconds(self.config.window_seconds as i64);
        let remaining = total_limit.saturating_sub(count);

        Ok(RateLimitResult::allowed(remaining, total_limit, reset_at))
    }

    /// Get rate limit headers for HTTP response
    pub fn get_headers(&self, result: &RateLimitResult) -> Vec<(String, String)> {
        let mut headers = vec![
            ("X-RateLimit-Limit".to_string(), result.limit.to_string()),
            ("X-RateLimit-Remaining".to_string(), result.remaining.to_string()),
            ("X-RateLimit-Reset".to_string(), result.reset_at.timestamp().to_string()),
        ];

        if let Some(retry_after) = result.retry_after {
            headers.push(("Retry-After".to_string(), retry_after.to_string()));
        }

        headers
    }
}

/// In-memory rate limit store
pub struct InMemoryRateLimitStore {
    entries: RwLock<HashMap<String, RateLimitEntry>>,
}

impl InMemoryRateLimitStore {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRateLimitStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RateLimitStore for InMemoryRateLimitStore {
    async fn get(&self, key: &str) -> Result<Option<(u32, DateTime<Utc>)>> {
        let entries = self.entries.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        Ok(entries.get(key).map(|e| (e.count, e.window_start)))
    }

    async fn increment(&self, key: &str, window_seconds: u64) -> Result<(u32, DateTime<Utc>)> {
        let mut entries = self.entries.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let window_duration = Duration::seconds(window_seconds as i64);

        let entry = entries.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: now,
            requests: Vec::new(),
        });

        // Check if window has expired
        if now > entry.window_start + window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;
        Ok((entry.count, entry.window_start))
    }

    async fn get_sliding(&self, key: &str, window_seconds: u64) -> Result<Vec<DateTime<Utc>>> {
        let entries = self.entries.read().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let window_start = now - Duration::seconds(window_seconds as i64);

        Ok(entries
            .get(key)
            .map(|e| {
                e.requests
                    .iter()
                    .filter(|&&t| t >= window_start)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn add_request(&self, key: &str, window_seconds: u64) -> Result<Vec<DateTime<Utc>>> {
        let mut entries = self.entries.write().map_err(|_| Error::Internal {
            message: "Lock poisoned".to_string(),
            request_id: None,
        })?;

        let now = Utc::now();
        let window_start = now - Duration::seconds(window_seconds as i64);

        let entry = entries.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: now,
            requests: Vec::new(),
        });

        // Remove old requests outside window
        entry.requests.retain(|&t| t >= window_start);

        // Add new request
        entry.requests.push(now);
        entry.count = entry.requests.len() as u32;

        Ok(entry.requests.clone())
    }
}

/// Multi-tier rate limiter for different limits based on context
pub struct TieredRateLimiter<S: RateLimitStore> {
    limiters: HashMap<String, RateLimiter<S>>,
}

impl<S: RateLimitStore + Clone> TieredRateLimiter<S> {
    pub fn new() -> Self {
        Self {
            limiters: HashMap::new(),
        }
    }

    pub fn add_tier(&mut self, name: &str, store: S, config: RateLimitConfig) {
        self.limiters.insert(name.to_string(), RateLimiter::new(store, config));
    }

    pub async fn check(&self, tier: &str, identifier: &str) -> Result<RateLimitResult> {
        let limiter = self.limiters.get(tier).ok_or_else(|| Error::Internal {
            message: format!("Unknown rate limit tier: {}", tier),
            request_id: None,
        })?;

        limiter.check(identifier).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_allows_requests() {
        let store = InMemoryRateLimitStore::new();
        let config = RateLimitConfig {
            max_requests: 5,
            window_seconds: 60,
            sliding_window: false,
            burst_size: 0,
            key_prefix: "test".to_string(),
        };
        let limiter = RateLimiter::new(store, config);

        for i in 0..5 {
            let result = limiter.check("user_1").await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i + 1);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_denies_excess() {
        let store = InMemoryRateLimitStore::new();
        let config = RateLimitConfig {
            max_requests: 3,
            window_seconds: 60,
            sliding_window: false,
            burst_size: 0,
            key_prefix: "test".to_string(),
        };
        let limiter = RateLimiter::new(store, config);

        // Make 3 allowed requests
        for _ in 0..3 {
            let result = limiter.check("user_1").await.unwrap();
            assert!(result.allowed);
        }

        // 4th request should be denied
        let result = limiter.check("user_1").await.unwrap();
        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_rate_limit_different_users() {
        let store = InMemoryRateLimitStore::new();
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
            sliding_window: false,
            burst_size: 0,
            key_prefix: "test".to_string(),
        };
        let limiter = RateLimiter::new(store, config);

        // User 1 makes 2 requests
        limiter.check("user_1").await.unwrap();
        limiter.check("user_1").await.unwrap();
        let result = limiter.check("user_1").await.unwrap();
        assert!(!result.allowed);

        // User 2 should still be allowed
        let result = limiter.check("user_2").await.unwrap();
        assert!(result.allowed);
    }
}
