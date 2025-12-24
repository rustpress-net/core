//! Bot and automation detection middleware.
//!
//! Detects automated traffic and potential bots using:
//! - User-Agent analysis
//! - Request timing patterns
//! - Header anomaly detection
//! - Honeypot field detection

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Bot detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDetectionConfig {
    /// Enable User-Agent checking
    pub enable_user_agent_check: bool,
    /// Enable timing analysis
    pub enable_timing_analysis: bool,
    /// Enable header anomaly detection
    pub enable_header_check: bool,
    /// Threshold score to consider as bot (0-100)
    pub bot_threshold: u32,
    /// Block detected bots
    pub block_bots: bool,
    /// Known good bot patterns (e.g., Googlebot)
    pub allowed_bots: Vec<String>,
    /// Suspicious patterns to flag
    pub suspicious_patterns: Vec<String>,
    /// Minimum request interval in milliseconds
    pub min_request_interval_ms: u64,
    /// Maximum requests per minute from same IP
    pub max_requests_per_minute: u32,
}

impl Default for BotDetectionConfig {
    fn default() -> Self {
        Self {
            enable_user_agent_check: true,
            enable_timing_analysis: true,
            enable_header_check: true,
            bot_threshold: 70,
            block_bots: false, // Log only by default
            allowed_bots: vec![
                "Googlebot".to_string(),
                "Bingbot".to_string(),
                "DuckDuckBot".to_string(),
                "Slurp".to_string(), // Yahoo
                "facebookexternalhit".to_string(),
                "Twitterbot".to_string(),
                "LinkedInBot".to_string(),
            ],
            suspicious_patterns: vec![
                "curl".to_string(),
                "wget".to_string(),
                "python-requests".to_string(),
                "Go-http-client".to_string(),
                "Java".to_string(),
                "libwww".to_string(),
                "lwp-trivial".to_string(),
                "Scrapy".to_string(),
            ],
            min_request_interval_ms: 50,
            max_requests_per_minute: 120,
        }
    }
}

/// Bot score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotScore {
    /// Overall bot score (0-100, higher = more likely bot)
    pub score: u32,
    /// Whether the request is classified as a bot
    pub is_bot: bool,
    /// Whether it's an allowed bot (e.g., search engines)
    pub is_allowed_bot: bool,
    /// Reasons for the score
    pub signals: Vec<BotSignal>,
}

impl BotScore {
    pub fn new() -> Self {
        Self {
            score: 0,
            is_bot: false,
            is_allowed_bot: false,
            signals: Vec::new(),
        }
    }

    pub fn add_signal(&mut self, signal: BotSignal) {
        self.score = self.score.saturating_add(signal.score_impact());
        self.signals.push(signal);
    }

    pub fn finalize(&mut self, threshold: u32) {
        self.is_bot = self.score >= threshold;
    }
}

impl Default for BotScore {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual bot detection signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotSignal {
    /// No User-Agent header
    MissingUserAgent,
    /// Empty User-Agent
    EmptyUserAgent,
    /// Suspicious User-Agent pattern
    SuspiciousUserAgent(String),
    /// Known bot User-Agent
    KnownBot(String),
    /// Allowed bot (search engine, etc.)
    AllowedBot(String),
    /// Missing Accept header
    MissingAccept,
    /// Missing Accept-Language header
    MissingAcceptLanguage,
    /// Missing Accept-Encoding header
    MissingAcceptEncoding,
    /// Requests too fast (timing)
    RequestTooFast { interval_ms: u64 },
    /// Too many requests per minute
    TooManyRequests { count: u32 },
    /// Header order anomaly
    HeaderOrderAnomaly,
    /// Honeypot field triggered
    HoneypotTriggered(String),
    /// Connection pattern anomaly
    ConnectionAnomaly(String),
}

impl BotSignal {
    pub fn score_impact(&self) -> u32 {
        match self {
            BotSignal::MissingUserAgent => 40,
            BotSignal::EmptyUserAgent => 35,
            BotSignal::SuspiciousUserAgent(_) => 25,
            BotSignal::KnownBot(_) => 60,
            BotSignal::AllowedBot(_) => 0, // No penalty for allowed bots
            BotSignal::MissingAccept => 15,
            BotSignal::MissingAcceptLanguage => 10,
            BotSignal::MissingAcceptEncoding => 10,
            BotSignal::RequestTooFast { .. } => 30,
            BotSignal::TooManyRequests { .. } => 35,
            BotSignal::HeaderOrderAnomaly => 20,
            BotSignal::HoneypotTriggered(_) => 80,
            BotSignal::ConnectionAnomaly(_) => 25,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BotSignal::MissingUserAgent => "missing_user_agent",
            BotSignal::EmptyUserAgent => "empty_user_agent",
            BotSignal::SuspiciousUserAgent(_) => "suspicious_user_agent",
            BotSignal::KnownBot(_) => "known_bot",
            BotSignal::AllowedBot(_) => "allowed_bot",
            BotSignal::MissingAccept => "missing_accept",
            BotSignal::MissingAcceptLanguage => "missing_accept_language",
            BotSignal::MissingAcceptEncoding => "missing_accept_encoding",
            BotSignal::RequestTooFast { .. } => "request_too_fast",
            BotSignal::TooManyRequests { .. } => "too_many_requests",
            BotSignal::HeaderOrderAnomaly => "header_order_anomaly",
            BotSignal::HoneypotTriggered(_) => "honeypot_triggered",
            BotSignal::ConnectionAnomaly(_) => "connection_anomaly",
        }
    }
}

/// Request timing tracker
struct RequestTracker {
    last_request: HashMap<String, Instant>,
    request_counts: HashMap<String, Vec<Instant>>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            last_request: HashMap::new(),
            request_counts: HashMap::new(),
        }
    }

    fn record_request(&mut self, ip: &str) -> (Option<u64>, u32) {
        let now = Instant::now();

        // Calculate interval since last request
        let interval = self
            .last_request
            .insert(ip.to_string(), now)
            .map(|last| now.duration_since(last).as_millis() as u64);

        // Count requests in last minute
        let counts = self.request_counts.entry(ip.to_string()).or_default();

        // Remove old entries (older than 1 minute)
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        counts.retain(|t| *t > one_minute_ago);
        counts.push(now);

        (interval, counts.len() as u32)
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        let five_minutes_ago = now - std::time::Duration::from_secs(300);

        // Clean up old entries
        self.last_request.retain(|_, v| *v > five_minutes_ago);
        self.request_counts.retain(|_, v| {
            if let Some(last) = v.last() {
                *last > five_minutes_ago
            } else {
                false
            }
        });
    }
}

/// Bot detection middleware
#[derive(Clone)]
pub struct BotDetectionMiddleware {
    config: Arc<BotDetectionConfig>,
    tracker: Arc<RwLock<RequestTracker>>,
    allowed_bot_patterns: Vec<Regex>,
    suspicious_patterns: Vec<Regex>,
}

impl BotDetectionMiddleware {
    pub fn new(config: BotDetectionConfig) -> Self {
        let allowed_bot_patterns = config
            .allowed_bots
            .iter()
            .filter_map(|p| Regex::new(&format!("(?i){}", regex::escape(p))).ok())
            .collect();

        let suspicious_patterns = config
            .suspicious_patterns
            .iter()
            .filter_map(|p| Regex::new(&format!("(?i){}", regex::escape(p))).ok())
            .collect();

        Self {
            config: Arc::new(config),
            tracker: Arc::new(RwLock::new(RequestTracker::new())),
            allowed_bot_patterns,
            suspicious_patterns,
        }
    }

    /// Analyze a request for bot indicators
    pub fn analyze(&self, request: &Request<Body>, client_ip: &str) -> BotScore {
        let mut score = BotScore::new();

        // User-Agent analysis
        if self.config.enable_user_agent_check {
            self.analyze_user_agent(request, &mut score);
        }

        // Header analysis
        if self.config.enable_header_check {
            self.analyze_headers(request, &mut score);
        }

        // Timing analysis
        if self.config.enable_timing_analysis {
            self.analyze_timing(client_ip, &mut score);
        }

        // Finalize score
        if score.is_allowed_bot {
            score.score = 0; // Reset score for allowed bots
            score.is_bot = true; // Still a bot, just allowed
        } else {
            score.finalize(self.config.bot_threshold);
        }

        score
    }

    fn analyze_user_agent(&self, request: &Request<Body>, score: &mut BotScore) {
        let user_agent = request
            .headers()
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok());

        match user_agent {
            None => {
                score.add_signal(BotSignal::MissingUserAgent);
            }
            Some(ua) if ua.is_empty() => {
                score.add_signal(BotSignal::EmptyUserAgent);
            }
            Some(ua) => {
                // Check for allowed bots first
                for pattern in &self.allowed_bot_patterns {
                    if pattern.is_match(ua) {
                        score.add_signal(BotSignal::AllowedBot(ua.to_string()));
                        score.is_allowed_bot = true;
                        return;
                    }
                }

                // Check for suspicious patterns
                for pattern in &self.suspicious_patterns {
                    if pattern.is_match(ua) {
                        score.add_signal(BotSignal::SuspiciousUserAgent(ua.to_string()));
                        break;
                    }
                }

                // Check for known bot indicators
                if is_known_bot_ua(ua) {
                    score.add_signal(BotSignal::KnownBot(ua.to_string()));
                }
            }
        }
    }

    fn analyze_headers(&self, request: &Request<Body>, score: &mut BotScore) {
        let headers = request.headers();

        // Check for missing browser headers
        if !headers.contains_key(header::ACCEPT) {
            score.add_signal(BotSignal::MissingAccept);
        }

        if !headers.contains_key(header::ACCEPT_LANGUAGE) {
            score.add_signal(BotSignal::MissingAcceptLanguage);
        }

        if !headers.contains_key(header::ACCEPT_ENCODING) {
            score.add_signal(BotSignal::MissingAcceptEncoding);
        }
    }

    fn analyze_timing(&self, client_ip: &str, score: &mut BotScore) {
        let (interval, count) = self.tracker.write().record_request(client_ip);

        // Check request interval
        if let Some(interval_ms) = interval {
            if interval_ms < self.config.min_request_interval_ms {
                score.add_signal(BotSignal::RequestTooFast { interval_ms });
            }
        }

        // Check request rate
        if count > self.config.max_requests_per_minute {
            score.add_signal(BotSignal::TooManyRequests { count });
        }
    }

    /// Periodic cleanup of old tracking data
    pub fn cleanup(&self) {
        self.tracker.write().cleanup();
    }
}

/// Check if User-Agent looks like a known bot
fn is_known_bot_ua(ua: &str) -> bool {
    static BOT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
        vec![
            Regex::new(r"(?i)bot\b").unwrap(),
            Regex::new(r"(?i)spider\b").unwrap(),
            Regex::new(r"(?i)crawler\b").unwrap(),
            Regex::new(r"(?i)scraper\b").unwrap(),
            Regex::new(r"(?i)headless").unwrap(),
            Regex::new(r"(?i)phantomjs").unwrap(),
            Regex::new(r"(?i)selenium").unwrap(),
            Regex::new(r"(?i)puppeteer").unwrap(),
            Regex::new(r"(?i)playwright").unwrap(),
        ]
    });

    BOT_PATTERNS.iter().any(|p| p.is_match(ua))
}

/// Bot detection middleware function
pub async fn bot_detection(
    State(detector): State<BotDetectionMiddleware>,
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

    let path = request.uri().path().to_string();
    let score = detector.analyze(&request, &client_ip);

    if score.is_bot && !score.is_allowed_bot {
        tracing::warn!(
            client_ip = %client_ip,
            path = %path,
            score = score.score,
            signals = ?score.signals.iter().map(|s| s.name()).collect::<Vec<_>>(),
            "Bot detected"
        );

        if detector.config.block_bots {
            return (StatusCode::FORBIDDEN, "Access denied").into_response();
        }
    }

    // Store score in extensions for downstream handlers
    let mut request = request;
    request.extensions_mut().insert(score);

    next.run(request).await
}

/// Check honeypot fields in form data
pub fn check_honeypot(form_data: &HashMap<String, String>, honeypot_fields: &[&str]) -> Option<BotSignal> {
    for field in honeypot_fields {
        if let Some(value) = form_data.get(*field) {
            if !value.is_empty() {
                return Some(BotSignal::HoneypotTriggered(field.to_string()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn create_request_with_headers(headers: Vec<(&str, &str)>) -> Request<Body> {
        let mut builder = Request::builder().uri("/test");
        for (name, value) in headers {
            builder = builder.header(name, value);
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn test_bot_score_signals() {
        let mut score = BotScore::new();
        score.add_signal(BotSignal::MissingUserAgent);
        assert_eq!(score.score, 40);

        score.add_signal(BotSignal::MissingAccept);
        assert_eq!(score.score, 55);
    }

    #[test]
    fn test_bot_detection_missing_ua() {
        let config = BotDetectionConfig::default();
        let detector = BotDetectionMiddleware::new(config);

        let request = create_request_with_headers(vec![]);
        let score = detector.analyze(&request, "127.0.0.1");

        assert!(score.score >= 40);
        assert!(score.signals.iter().any(|s| matches!(s, BotSignal::MissingUserAgent)));
    }

    #[test]
    fn test_allowed_bot_detection() {
        let config = BotDetectionConfig::default();
        let detector = BotDetectionMiddleware::new(config);

        let request = create_request_with_headers(vec![
            ("user-agent", "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"),
            ("accept", "*/*"),
        ]);
        let score = detector.analyze(&request, "127.0.0.1");

        assert!(score.is_allowed_bot);
        assert_eq!(score.score, 0);
    }

    #[test]
    fn test_suspicious_ua_detection() {
        let config = BotDetectionConfig::default();
        let detector = BotDetectionMiddleware::new(config);

        let request = create_request_with_headers(vec![
            ("user-agent", "python-requests/2.28.0"),
            ("accept", "*/*"),
        ]);
        let score = detector.analyze(&request, "127.0.0.1");

        assert!(score.signals.iter().any(|s| matches!(s, BotSignal::SuspiciousUserAgent(_))));
    }

    #[test]
    fn test_honeypot_detection() {
        let mut form_data = HashMap::new();
        form_data.insert("email".to_string(), "user@example.com".to_string());
        form_data.insert("website".to_string(), "".to_string()); // Honeypot empty = good
        form_data.insert("fax_number".to_string(), "123456".to_string()); // Honeypot filled = bad

        let honeypots = &["website", "fax_number"];
        let result = check_honeypot(&form_data, honeypots);

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), BotSignal::HoneypotTriggered(_)));
    }
}
