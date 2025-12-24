//! Request fingerprinting middleware.
//!
//! Creates unique fingerprints of client requests for:
//! - Client behavior tracking
//! - Anomaly detection
//! - Session correlation
//! - Bot detection augmentation

use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

/// Request fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFingerprint {
    /// Hash of header names and order
    pub header_hash: u64,
    /// Hash of Accept-* headers pattern
    pub accept_hash: u64,
    /// User-Agent hash
    pub ua_hash: u64,
    /// TLS info hash (if available)
    pub tls_hash: Option<u64>,
    /// Combined fingerprint hash
    pub combined_hash: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl RequestFingerprint {
    /// Create a fingerprint from a request
    pub fn from_request(request: &Request<Body>) -> Self {
        let headers = request.headers();

        // Hash header names in order
        let header_hash = {
            let mut hasher = DefaultHasher::new();
            for (name, _) in headers.iter() {
                name.as_str().hash(&mut hasher);
            }
            hasher.finish()
        };

        // Hash Accept-* headers
        let accept_hash = {
            let mut hasher = DefaultHasher::new();
            if let Some(accept) = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()) {
                accept.hash(&mut hasher);
            }
            if let Some(lang) = headers.get(header::ACCEPT_LANGUAGE).and_then(|v| v.to_str().ok()) {
                lang.hash(&mut hasher);
            }
            if let Some(enc) = headers.get(header::ACCEPT_ENCODING).and_then(|v| v.to_str().ok()) {
                enc.hash(&mut hasher);
            }
            hasher.finish()
        };

        // Hash User-Agent
        let ua_hash = {
            let mut hasher = DefaultHasher::new();
            if let Some(ua) = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()) {
                ua.hash(&mut hasher);
            }
            hasher.finish()
        };

        // Combine all hashes
        let combined_hash = {
            let mut hasher = DefaultHasher::new();
            header_hash.hash(&mut hasher);
            accept_hash.hash(&mut hasher);
            ua_hash.hash(&mut hasher);
            hasher.finish()
        };

        Self {
            header_hash,
            accept_hash,
            ua_hash,
            tls_hash: None, // Would need TLS info from connection
            combined_hash,
            timestamp: Utc::now(),
        }
    }

    /// Calculate similarity to another fingerprint (0.0 - 1.0)
    pub fn similarity(&self, other: &RequestFingerprint) -> f64 {
        let mut matches = 0.0;
        let mut total = 3.0;

        if self.header_hash == other.header_hash {
            matches += 1.0;
        }
        if self.accept_hash == other.accept_hash {
            matches += 1.0;
        }
        if self.ua_hash == other.ua_hash {
            matches += 1.0;
        }

        if let (Some(a), Some(b)) = (self.tls_hash, other.tls_hash) {
            total += 1.0;
            if a == b {
                matches += 1.0;
            }
        }

        matches / total
    }
}

/// Client profile built from multiple fingerprints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfile {
    /// Client identifier (IP or session)
    pub client_id: String,
    /// Historical fingerprints
    pub fingerprints: Vec<RequestFingerprint>,
    /// First seen timestamp
    pub first_seen: DateTime<Utc>,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Total request count
    pub request_count: u64,
    /// Anomaly score (0.0 - 1.0)
    pub anomaly_score: f32,
    /// Dominant fingerprint hash
    pub dominant_fingerprint: Option<u64>,
    /// Request paths accessed
    pub paths_accessed: Vec<String>,
}

impl ClientProfile {
    pub fn new(client_id: String) -> Self {
        let now = Utc::now();
        Self {
            client_id,
            fingerprints: Vec::new(),
            first_seen: now,
            last_seen: now,
            request_count: 0,
            anomaly_score: 0.0,
            dominant_fingerprint: None,
            paths_accessed: Vec::new(),
        }
    }

    /// Add a fingerprint and update profile
    pub fn add_fingerprint(&mut self, fingerprint: RequestFingerprint, path: String) {
        self.last_seen = Utc::now();
        self.request_count += 1;

        // Track paths (limit to last 100)
        self.paths_accessed.push(path);
        if self.paths_accessed.len() > 100 {
            self.paths_accessed.remove(0);
        }

        // Calculate anomaly score based on fingerprint consistency
        if !self.fingerprints.is_empty() {
            let last_fingerprint = self.fingerprints.last().unwrap();
            let similarity = fingerprint.similarity(last_fingerprint);

            // Low similarity = potential anomaly
            if similarity < 0.5 {
                self.anomaly_score = (self.anomaly_score + 0.3).min(1.0);
            } else {
                self.anomaly_score = (self.anomaly_score - 0.1).max(0.0);
            }
        }

        // Keep limited fingerprint history
        if self.fingerprints.len() >= 10 {
            self.fingerprints.remove(0);
        }
        self.fingerprints.push(fingerprint.clone());

        // Update dominant fingerprint
        self.update_dominant_fingerprint();
    }

    fn update_dominant_fingerprint(&mut self) {
        let mut counts: HashMap<u64, usize> = HashMap::new();
        for fp in &self.fingerprints {
            *counts.entry(fp.combined_hash).or_insert(0) += 1;
        }

        self.dominant_fingerprint = counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hash, _)| hash);
    }

    /// Check if the profile shows suspicious behavior
    pub fn is_suspicious(&self) -> bool {
        self.anomaly_score > 0.7
    }

    /// Get the profile age in seconds
    pub fn age_seconds(&self) -> i64 {
        (self.last_seen - self.first_seen).num_seconds()
    }

    /// Get requests per minute rate
    pub fn requests_per_minute(&self) -> f64 {
        let age_minutes = self.age_seconds() as f64 / 60.0;
        if age_minutes > 0.0 {
            self.request_count as f64 / age_minutes
        } else {
            self.request_count as f64
        }
    }
}

/// Fingerprint middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    /// Maximum number of client profiles to track
    pub max_profiles: usize,
    /// Profile expiry in seconds
    pub profile_expiry_seconds: u64,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Anomaly threshold to flag
    pub anomaly_threshold: f32,
    /// Track request paths
    pub track_paths: bool,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            max_profiles: 10000,
            profile_expiry_seconds: 3600, // 1 hour
            enable_anomaly_detection: true,
            anomaly_threshold: 0.7,
            track_paths: true,
        }
    }
}

/// Fingerprint middleware
#[derive(Clone)]
pub struct FingerprintMiddleware {
    config: Arc<FingerprintConfig>,
    profiles: Arc<RwLock<HashMap<String, ClientProfile>>>,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl FingerprintMiddleware {
    pub fn new(config: FingerprintConfig) -> Self {
        Self {
            config: Arc::new(config),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Process a request and update client profile
    pub fn process(&self, request: &Request<Body>, client_id: &str) -> (RequestFingerprint, Option<ClientProfile>) {
        let fingerprint = RequestFingerprint::from_request(request);
        let path = request.uri().path().to_string();

        let mut profiles = self.profiles.write();

        // Periodic cleanup
        self.maybe_cleanup(&mut profiles);

        // Get or create profile
        let profile = profiles
            .entry(client_id.to_string())
            .or_insert_with(|| ClientProfile::new(client_id.to_string()));

        // Update profile
        profile.add_fingerprint(fingerprint.clone(), path);

        let profile_clone = if self.config.enable_anomaly_detection {
            Some(profile.clone())
        } else {
            None
        };

        (fingerprint, profile_clone)
    }

    /// Get a client profile
    pub fn get_profile(&self, client_id: &str) -> Option<ClientProfile> {
        self.profiles.read().get(client_id).cloned()
    }

    /// Get all profiles with anomaly score above threshold
    pub fn get_suspicious_profiles(&self) -> Vec<ClientProfile> {
        self.profiles
            .read()
            .values()
            .filter(|p| p.anomaly_score >= self.config.anomaly_threshold)
            .cloned()
            .collect()
    }

    /// Get profile statistics
    pub fn get_stats(&self) -> FingerprintStats {
        let profiles = self.profiles.read();
        let total = profiles.len();
        let suspicious = profiles
            .values()
            .filter(|p| p.anomaly_score >= self.config.anomaly_threshold)
            .count();
        let total_requests: u64 = profiles.values().map(|p| p.request_count).sum();

        FingerprintStats {
            total_profiles: total,
            suspicious_profiles: suspicious,
            total_requests,
        }
    }

    fn maybe_cleanup(&self, profiles: &mut HashMap<String, ClientProfile>) {
        let mut last_cleanup = self.last_cleanup.write();
        let now = Instant::now();

        // Cleanup every minute
        if now.duration_since(*last_cleanup).as_secs() < 60 {
            return;
        }
        *last_cleanup = now;

        let cutoff = Utc::now() - chrono::Duration::seconds(self.config.profile_expiry_seconds as i64);

        // Remove expired profiles
        profiles.retain(|_, p| p.last_seen > cutoff);

        // If still over limit, remove oldest profiles
        if profiles.len() > self.config.max_profiles {
            let mut entries: Vec<_> = profiles.iter().map(|(k, v)| (k.clone(), v.last_seen)).collect();
            entries.sort_by_key(|(_, ts)| *ts);

            let to_remove = profiles.len() - self.config.max_profiles;
            for (key, _) in entries.into_iter().take(to_remove) {
                profiles.remove(&key);
            }
        }
    }

    /// Clear all profiles
    pub fn clear(&self) {
        self.profiles.write().clear();
    }
}

/// Fingerprint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintStats {
    pub total_profiles: usize,
    pub suspicious_profiles: usize,
    pub total_requests: u64,
}

/// Fingerprint middleware function
pub async fn fingerprint(
    State(middleware): State<FingerprintMiddleware>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let (fingerprint, profile) = middleware.process(&request, &client_ip);

    // Log suspicious profiles
    if let Some(ref p) = profile {
        if p.is_suspicious() {
            tracing::warn!(
                client_ip = %client_ip,
                anomaly_score = p.anomaly_score,
                request_count = p.request_count,
                "Suspicious client profile detected"
            );
        }
    }

    // Store fingerprint in extensions
    let mut request = request;
    request.extensions_mut().insert(fingerprint);
    if let Some(p) = profile {
        request.extensions_mut().insert(p);
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn create_request(headers: Vec<(&str, &str)>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn test_fingerprint_creation() {
        let request = create_request(vec![
            ("user-agent", "Mozilla/5.0"),
            ("accept", "text/html"),
            ("accept-language", "en-US"),
        ]);

        let fingerprint = RequestFingerprint::from_request(&request);
        assert!(fingerprint.combined_hash > 0);
    }

    #[test]
    fn test_fingerprint_similarity() {
        let request1 = create_request(vec![
            ("user-agent", "Mozilla/5.0"),
            ("accept", "text/html"),
        ]);

        let request2 = create_request(vec![
            ("user-agent", "Mozilla/5.0"),
            ("accept", "text/html"),
        ]);

        let fp1 = RequestFingerprint::from_request(&request1);
        let fp2 = RequestFingerprint::from_request(&request2);

        assert_eq!(fp1.similarity(&fp2), 1.0);
    }

    #[test]
    fn test_client_profile() {
        let mut profile = ClientProfile::new("test-client".to_string());

        let request = create_request(vec![
            ("user-agent", "Mozilla/5.0"),
            ("accept", "text/html"),
        ]);

        let fingerprint = RequestFingerprint::from_request(&request);
        profile.add_fingerprint(fingerprint, "/test".to_string());

        assert_eq!(profile.request_count, 1);
        assert!(!profile.is_suspicious());
    }

    #[test]
    fn test_fingerprint_middleware() {
        let config = FingerprintConfig::default();
        let middleware = FingerprintMiddleware::new(config);

        let request = create_request(vec![
            ("user-agent", "Mozilla/5.0"),
            ("accept", "text/html"),
        ]);

        let (fingerprint, profile) = middleware.process(&request, "192.168.1.1");

        assert!(fingerprint.combined_hash > 0);
        assert!(profile.is_some());

        let stats = middleware.get_stats();
        assert_eq!(stats.total_profiles, 1);
        assert_eq!(stats.total_requests, 1);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut profile = ClientProfile::new("test".to_string());

        // Add consistent fingerprints
        for _ in 0..5 {
            let request = create_request(vec![
                ("user-agent", "Mozilla/5.0"),
                ("accept", "text/html"),
            ]);
            let fp = RequestFingerprint::from_request(&request);
            profile.add_fingerprint(fp, "/test".to_string());
        }

        assert!(profile.anomaly_score < 0.5);

        // Add different fingerprint
        let different_request = create_request(vec![
            ("user-agent", "curl/7.0"),
        ]);
        let different_fp = RequestFingerprint::from_request(&different_request);
        profile.add_fingerprint(different_fp, "/test".to_string());

        // Anomaly score should increase
        assert!(profile.anomaly_score > 0.0);
    }
}
