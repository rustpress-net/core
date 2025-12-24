//! Incremental Static Regeneration (ISR)
//!
//! Implements ISR for static page generation with background regeneration.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;

/// ISR errors
#[derive(Debug, Error)]
pub enum IsrError {
    #[error("Page not found: {0}")]
    NotFound(String),

    #[error("Regeneration failed: {0}")]
    RegenerationFailed(String),

    #[error("Render timeout")]
    Timeout,
}

/// ISR configuration
#[derive(Debug, Clone)]
pub struct IsrConfig {
    /// Default revalidation period in seconds
    pub default_revalidate: u64,
    /// Maximum stale age before forced regeneration
    pub max_stale_age: u64,
    /// Concurrent regeneration limit
    pub max_concurrent_regenerations: usize,
    /// Enable on-demand revalidation
    pub on_demand_revalidation: bool,
    /// Revalidation secret for on-demand API
    pub revalidation_secret: Option<String>,
    /// Enable fallback pages
    pub fallback: FallbackMode,
    /// Prerender paths
    pub prerender_paths: Vec<String>,
}

impl Default for IsrConfig {
    fn default() -> Self {
        Self {
            default_revalidate: 60, // 1 minute
            max_stale_age: 3600,    // 1 hour
            max_concurrent_regenerations: 4,
            on_demand_revalidation: true,
            revalidation_secret: None,
            fallback: FallbackMode::Blocking,
            prerender_paths: Vec::new(),
        }
    }
}

/// Fallback mode for pages not yet generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackMode {
    /// Block until page is generated
    Blocking,
    /// Show fallback/loading state, hydrate when ready
    Fallback,
    /// Return 404 for unknown paths
    None,
}

/// Static page entry
#[derive(Debug, Clone)]
pub struct StaticPage {
    /// Page path
    pub path: String,
    /// Rendered HTML
    pub html: String,
    /// Page props/data as JSON
    pub props: Option<String>,
    /// Generation timestamp
    pub generated_at: Instant,
    /// Revalidation period in seconds
    pub revalidate: u64,
    /// Page headers
    pub headers: HashMap<String, String>,
    /// Build ID for cache invalidation
    pub build_id: String,
}

impl StaticPage {
    /// Check if page needs revalidation
    pub fn needs_revalidation(&self) -> bool {
        self.generated_at.elapsed().as_secs() >= self.revalidate
    }

    /// Check if page is too stale
    pub fn is_too_stale(&self, max_stale: u64) -> bool {
        self.generated_at.elapsed().as_secs() >= max_stale
    }

    /// Get page age in seconds
    pub fn age_seconds(&self) -> u64 {
        self.generated_at.elapsed().as_secs()
    }
}

/// ISR page store
pub struct IsrStore {
    config: IsrConfig,
    /// Cached static pages
    pages: Arc<RwLock<HashMap<String, StaticPage>>>,
    /// Pages currently being regenerated
    regenerating: Arc<RwLock<HashMap<String, RegenerationState>>>,
    /// Build ID
    build_id: String,
    /// Revalidation event broadcaster
    revalidation_tx: broadcast::Sender<RevalidationEvent>,
}

#[derive(Debug, Clone)]
struct RegenerationState {
    started_at: Instant,
    in_progress: bool,
}

/// Revalidation event
#[derive(Debug, Clone)]
pub struct RevalidationEvent {
    pub path: String,
    pub success: bool,
    pub timestamp: i64,
}

impl IsrStore {
    pub fn new(config: IsrConfig) -> Self {
        let build_id = generate_build_id();
        let (tx, _) = broadcast::channel(100);

        Self {
            config,
            pages: Arc::new(RwLock::new(HashMap::new())),
            regenerating: Arc::new(RwLock::new(HashMap::new())),
            build_id,
            revalidation_tx: tx,
        }
    }

    /// Get a static page
    pub fn get(&self, path: &str) -> Option<StaticPage> {
        self.pages.read().get(path).cloned()
    }

    /// Store a static page
    pub fn store(&self, page: StaticPage) {
        self.pages.write().insert(page.path.clone(), page);
    }

    /// Check if a path is being regenerated
    pub fn is_regenerating(&self, path: &str) -> bool {
        self.regenerating.read()
            .get(path)
            .map(|s| s.in_progress)
            .unwrap_or(false)
    }

    /// Start regeneration for a path
    pub fn start_regeneration(&self, path: &str) -> bool {
        let mut regenerating = self.regenerating.write();

        // Check concurrent limit
        let active_count = regenerating.values().filter(|s| s.in_progress).count();
        if active_count >= self.config.max_concurrent_regenerations {
            return false;
        }

        // Check if already regenerating
        if let Some(state) = regenerating.get(path) {
            if state.in_progress {
                return false;
            }
        }

        regenerating.insert(path.to_string(), RegenerationState {
            started_at: Instant::now(),
            in_progress: true,
        });

        true
    }

    /// Complete regeneration for a path
    pub fn complete_regeneration(&self, path: &str, page: Option<StaticPage>) {
        self.regenerating.write().remove(path);

        if let Some(page) = page {
            self.pages.write().insert(path.to_string(), page);

            // Broadcast revalidation event
            let _ = self.revalidation_tx.send(RevalidationEvent {
                path: path.to_string(),
                success: true,
                timestamp: chrono::Utc::now().timestamp(),
            });
        } else {
            let _ = self.revalidation_tx.send(RevalidationEvent {
                path: path.to_string(),
                success: false,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
    }

    /// Subscribe to revalidation events
    pub fn subscribe(&self) -> broadcast::Receiver<RevalidationEvent> {
        self.revalidation_tx.subscribe()
    }

    /// Invalidate a specific path
    pub fn invalidate(&self, path: &str) {
        self.pages.write().remove(path);
    }

    /// Invalidate paths matching a pattern
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        let re = regex::Regex::new(pattern).unwrap_or_else(|_| {
            regex::Regex::new(&regex::escape(pattern)).unwrap()
        });

        let mut pages = self.pages.write();
        let to_remove: Vec<String> = pages
            .keys()
            .filter(|path| re.is_match(path))
            .cloned()
            .collect();

        let count = to_remove.len();
        for path in to_remove {
            pages.remove(&path);
        }

        count
    }

    /// Invalidate all pages
    pub fn invalidate_all(&self) {
        self.pages.write().clear();
    }

    /// Get store statistics
    pub fn stats(&self) -> IsrStats {
        let pages = self.pages.read();
        let regenerating = self.regenerating.read();

        let stale_count = pages.values()
            .filter(|p| p.needs_revalidation())
            .count();

        IsrStats {
            total_pages: pages.len(),
            stale_pages: stale_count,
            regenerating_pages: regenerating.values().filter(|s| s.in_progress).count(),
        }
    }

    /// Get the current build ID
    pub fn build_id(&self) -> &str {
        &self.build_id
    }
}

/// ISR statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsrStats {
    pub total_pages: usize,
    pub stale_pages: usize,
    pub regenerating_pages: usize,
}

/// ISR handler for processing requests
pub struct IsrHandler {
    store: Arc<IsrStore>,
    config: IsrConfig,
}

impl IsrHandler {
    pub fn new(config: IsrConfig) -> Self {
        let store = Arc::new(IsrStore::new(config.clone()));
        Self { store, config }
    }

    /// Get static page for a path
    pub async fn get_page<F, Fut>(
        &self,
        path: &str,
        generate_fn: F,
    ) -> Result<(StaticPage, bool), IsrError>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<StaticPage, IsrError>>,
    {
        // Check for existing page
        if let Some(page) = self.store.get(path) {
            let is_stale = page.needs_revalidation();

            if is_stale && !page.is_too_stale(self.config.max_stale_age) {
                // Trigger background regeneration
                self.trigger_regeneration(path, generate_fn);
            }

            // Return existing page (stale-while-revalidate)
            return Ok((page, is_stale));
        }

        // Page not found, generate it
        match self.config.fallback {
            FallbackMode::Blocking => {
                // Generate and wait
                let page = generate_fn(path.to_string()).await?;
                self.store.store(page.clone());
                Ok((page, false))
            }
            FallbackMode::Fallback => {
                // Return fallback, generate in background
                self.trigger_regeneration(path, generate_fn);
                Err(IsrError::NotFound(path.to_string()))
            }
            FallbackMode::None => {
                Err(IsrError::NotFound(path.to_string()))
            }
        }
    }

    fn trigger_regeneration<F, Fut>(&self, path: &str, generate_fn: F)
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<StaticPage, IsrError>>,
    {
        if !self.store.start_regeneration(path) {
            return; // Already regenerating or at limit
        }

        let store = self.store.clone();
        let path = path.to_string();

        tokio::spawn(async move {
            let result = generate_fn(path.clone()).await;

            match result {
                Ok(page) => {
                    store.complete_regeneration(&path, Some(page));
                }
                Err(e) => {
                    tracing::error!("ISR regeneration failed for {}: {}", path, e);
                    store.complete_regeneration(&path, None);
                }
            }
        });
    }

    /// Handle on-demand revalidation request
    pub async fn revalidate<F, Fut>(
        &self,
        path: &str,
        secret: Option<&str>,
        generate_fn: F,
    ) -> Result<(), IsrError>
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = Result<StaticPage, IsrError>>,
    {
        // Verify secret if configured
        if let Some(expected_secret) = &self.config.revalidation_secret {
            match secret {
                Some(s) if s == expected_secret => {}
                _ => return Err(IsrError::RegenerationFailed("Invalid secret".to_string())),
            }
        }

        // Force regeneration
        self.store.invalidate(path);

        if self.store.start_regeneration(path) {
            let page = generate_fn(path.to_string()).await?;
            self.store.complete_regeneration(path, Some(page));
        }

        Ok(())
    }

    /// Get store reference
    pub fn store(&self) -> &Arc<IsrStore> {
        &self.store
    }
}

/// Generate unique build ID
fn generate_build_id() -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let random: u32 = rand::random();
    format!("{:x}{:x}", timestamp, random)
}

/// Path matcher for ISR routes
pub struct PathMatcher {
    patterns: Vec<PathPattern>,
}

#[derive(Debug, Clone)]
struct PathPattern {
    pattern: String,
    regex: regex::Regex,
    params: Vec<String>,
    revalidate: u64,
}

impl PathMatcher {
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    /// Add a path pattern with revalidation period
    pub fn add_pattern(&mut self, pattern: &str, revalidate: u64) {
        let (regex_str, params) = pattern_to_regex(pattern);
        let regex = regex::Regex::new(&regex_str).unwrap();

        self.patterns.push(PathPattern {
            pattern: pattern.to_string(),
            regex,
            params,
            revalidate,
        });
    }

    /// Match a path and extract parameters
    pub fn match_path(&self, path: &str) -> Option<(u64, HashMap<String, String>)> {
        for pattern in &self.patterns {
            if let Some(caps) = pattern.regex.captures(path) {
                let mut params = HashMap::new();
                for (i, param_name) in pattern.params.iter().enumerate() {
                    if let Some(m) = caps.get(i + 1) {
                        params.insert(param_name.clone(), m.as_str().to_string());
                    }
                }
                return Some((pattern.revalidate, params));
            }
        }
        None
    }
}

impl Default for PathMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert path pattern to regex
fn pattern_to_regex(pattern: &str) -> (String, Vec<String>) {
    let mut regex = String::from("^");
    let mut params = Vec::new();

    for segment in pattern.split('/') {
        if segment.is_empty() {
            continue;
        }

        regex.push('/');

        if segment.starts_with('[') && segment.ends_with(']') {
            // Dynamic segment
            let param = &segment[1..segment.len()-1];

            if param.starts_with("...") {
                // Catch-all segment
                let name = &param[3..];
                params.push(name.to_string());
                regex.push_str("(.+)");
            } else {
                // Regular dynamic segment
                params.push(param.to_string());
                regex.push_str("([^/]+)");
            }
        } else {
            // Static segment
            regex.push_str(&regex::escape(segment));
        }
    }

    regex.push('$');
    (regex, params)
}

/// Get static paths for pre-rendering
pub struct StaticPathsProvider {
    paths: Vec<StaticPath>,
}

#[derive(Debug, Clone)]
pub struct StaticPath {
    pub path: String,
    pub params: HashMap<String, String>,
}

impl StaticPathsProvider {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    /// Add a static path
    pub fn add(&mut self, path: String, params: HashMap<String, String>) {
        self.paths.push(StaticPath { path, params });
    }

    /// Get all static paths
    pub fn get_paths(&self) -> &[StaticPath] {
        &self.paths
    }

    /// Generate paths from a list of values
    pub fn from_values(pattern: &str, param_name: &str, values: &[String]) -> Self {
        let mut provider = Self::new();

        for value in values {
            let path = pattern.replace(&format!("[{}]", param_name), value);
            let mut params = HashMap::new();
            params.insert(param_name.to_string(), value.clone());
            provider.add(path, params);
        }

        provider
    }
}

impl Default for StaticPathsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_to_regex() {
        let (regex, params) = pattern_to_regex("/posts/[id]");
        assert_eq!(params, vec!["id"]);

        let re = regex::Regex::new(&regex).unwrap();
        assert!(re.is_match("/posts/123"));
        assert!(!re.is_match("/posts/123/comments"));
    }

    #[test]
    fn test_catch_all_pattern() {
        let (regex, params) = pattern_to_regex("/docs/[...slug]");
        assert_eq!(params, vec!["slug"]);

        let re = regex::Regex::new(&regex).unwrap();
        assert!(re.is_match("/docs/getting-started"));
        assert!(re.is_match("/docs/api/reference/methods"));
    }

    #[test]
    fn test_path_matcher() {
        let mut matcher = PathMatcher::new();
        matcher.add_pattern("/posts/[id]", 60);
        matcher.add_pattern("/users/[username]/posts", 120);

        let result = matcher.match_path("/posts/123");
        assert!(result.is_some());
        let (revalidate, params) = result.unwrap();
        assert_eq!(revalidate, 60);
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_static_page_staleness() {
        let page = StaticPage {
            path: "/test".to_string(),
            html: "<div>Test</div>".to_string(),
            props: None,
            generated_at: Instant::now() - Duration::from_secs(120),
            revalidate: 60,
            headers: HashMap::new(),
            build_id: "test".to_string(),
        };

        assert!(page.needs_revalidation());
        assert!(!page.is_too_stale(300));
    }

    #[test]
    fn test_static_paths_provider() {
        let provider = StaticPathsProvider::from_values(
            "/posts/[id]",
            "id",
            &["1".to_string(), "2".to_string(), "3".to_string()],
        );

        let paths = provider.get_paths();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].path, "/posts/1");
        assert_eq!(paths[1].path, "/posts/2");
    }
}
