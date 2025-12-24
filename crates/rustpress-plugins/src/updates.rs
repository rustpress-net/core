//! Plugin Update Checking and Marketplace Integration
//!
//! Handles checking for updates and marketplace operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

// ============================================================================
// Update Checking (Point 177)
// ============================================================================

/// Update checker
pub struct UpdateChecker {
    /// Cached update info
    cache: Arc<RwLock<HashMap<String, UpdateInfo>>>,
    /// Last check timestamp
    last_check: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
    /// Configuration
    config: UpdateConfig,
}

/// Update configuration
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Update server URL
    pub update_server: String,
    /// Check interval (hours)
    pub check_interval_hours: u32,
    /// Enable auto-update checking
    pub auto_check: bool,
    /// Include beta versions
    pub include_beta: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            update_server: "https://api.rustpress.io/plugins".to_string(),
            check_interval_hours: 12,
            auto_check: true,
            include_beta: false,
        }
    }
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub plugin_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub changelog: Option<String>,
    pub release_date: Option<chrono::DateTime<chrono::Utc>>,
    pub requires_rustpress: Option<String>,
    pub tested_up_to: Option<String>,
    pub package_hash: Option<String>,
    pub is_security_update: bool,
}

impl UpdateChecker {
    pub fn new(config: UpdateConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            last_check: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check for updates for all plugins
    pub async fn check_all(
        &self,
        plugins: &[(String, String)], // (plugin_id, current_version)
    ) -> Result<Vec<UpdateInfo>, UpdateError> {
        let mut updates = Vec::new();

        for (plugin_id, current_version) in plugins {
            match self.check_plugin(plugin_id, current_version).await {
                Ok(info) => {
                    if info.update_available {
                        updates.push(info);
                    }
                }
                Err(e) => {
                    warn!("Failed to check updates for {}: {}", plugin_id, e);
                }
            }
        }

        *self.last_check.write() = Some(chrono::Utc::now());
        info!("Found {} plugin updates", updates.len());
        Ok(updates)
    }

    /// Check for update for a single plugin
    pub async fn check_plugin(
        &self,
        plugin_id: &str,
        current_version: &str,
    ) -> Result<UpdateInfo, UpdateError> {
        // In real implementation, would make HTTP request to update server
        // For now, return mock data
        let info = UpdateInfo {
            plugin_id: plugin_id.to_string(),
            current_version: current_version.to_string(),
            latest_version: current_version.to_string(),
            update_available: false,
            download_url: None,
            changelog: None,
            release_date: None,
            requires_rustpress: None,
            tested_up_to: None,
            package_hash: None,
            is_security_update: false,
        };

        self.cache.write().insert(plugin_id.to_string(), info.clone());
        Ok(info)
    }

    /// Get cached update info
    pub fn get_cached(&self, plugin_id: &str) -> Option<UpdateInfo> {
        self.cache.read().get(plugin_id).cloned()
    }

    /// Get all cached updates
    pub fn get_all_cached(&self) -> Vec<UpdateInfo> {
        self.cache.read().values().cloned().collect()
    }

    /// Get available updates
    pub fn get_available_updates(&self) -> Vec<UpdateInfo> {
        self.cache
            .read()
            .values()
            .filter(|u| u.update_available)
            .cloned()
            .collect()
    }

    /// Check if cache is stale
    pub fn is_cache_stale(&self) -> bool {
        let last = self.last_check.read();
        match *last {
            Some(t) => {
                let hours_since = (chrono::Utc::now() - t).num_hours();
                hours_since >= self.config.check_interval_hours as i64
            }
            None => true,
        }
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
        *self.last_check.write() = None;
    }
}

/// Update error
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),
}

// ============================================================================
// Marketplace Integration (Point 178)
// ============================================================================

/// Marketplace client
pub struct MarketplaceClient {
    /// Base URL
    base_url: String,
    /// API key (optional)
    api_key: Option<String>,
    /// Cached search results
    cache: Arc<RwLock<MarketplaceCache>>,
}

/// Marketplace cache
#[derive(Default)]
struct MarketplaceCache {
    featured: Option<Vec<MarketplacePlugin>>,
    popular: Option<Vec<MarketplacePlugin>>,
    recent: Option<Vec<MarketplacePlugin>>,
    search_results: HashMap<String, Vec<MarketplacePlugin>>,
    plugin_details: HashMap<String, MarketplacePlugin>,
}

/// Marketplace plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub author_url: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub rating: f32,
    pub rating_count: u32,
    pub active_installs: u64,
    pub download_url: String,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub requires_rustpress: Option<String>,
    pub tested_up_to: Option<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub sections: PluginSections,
    pub is_verified: bool,
    pub is_premium: bool,
    pub price: Option<f64>,
}

/// Plugin detail sections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSections {
    pub description: Option<String>,
    pub installation: Option<String>,
    pub faq: Option<String>,
    pub changelog: Option<String>,
    pub screenshots: Vec<Screenshot>,
    pub reviews: Vec<Review>,
}

/// Screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub url: String,
    pub caption: Option<String>,
}

/// Review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub author: String,
    pub rating: u8,
    pub content: String,
    pub date: chrono::DateTime<chrono::Utc>,
}

/// Search query
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub author: Option<String>,
    pub sort_by: SortBy,
    pub page: u32,
    pub per_page: u32,
}

/// Sort options
#[derive(Debug, Clone, Copy, Default)]
pub enum SortBy {
    #[default]
    Relevance,
    Rating,
    Popularity,
    Downloads,
    Updated,
    Name,
}

impl MarketplaceClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: None,
            cache: Arc::new(RwLock::new(MarketplaceCache::default())),
        }
    }

    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Search plugins
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResult, MarketplaceError> {
        // In real implementation, would make HTTP request
        Ok(SearchResult {
            plugins: Vec::new(),
            total: 0,
            page: query.page,
            per_page: query.per_page,
            total_pages: 0,
        })
    }

    /// Get featured plugins
    pub async fn get_featured(&self) -> Result<Vec<MarketplacePlugin>, MarketplaceError> {
        if let Some(cached) = &self.cache.read().featured {
            return Ok(cached.clone());
        }

        // Would make HTTP request
        let plugins = Vec::new();
        self.cache.write().featured = Some(plugins.clone());
        Ok(plugins)
    }

    /// Get popular plugins
    pub async fn get_popular(&self) -> Result<Vec<MarketplacePlugin>, MarketplaceError> {
        if let Some(cached) = &self.cache.read().popular {
            return Ok(cached.clone());
        }

        let plugins = Vec::new();
        self.cache.write().popular = Some(plugins.clone());
        Ok(plugins)
    }

    /// Get plugin details
    pub async fn get_plugin(&self, slug: &str) -> Result<MarketplacePlugin, MarketplaceError> {
        if let Some(cached) = self.cache.read().plugin_details.get(slug) {
            return Ok(cached.clone());
        }

        // Would make HTTP request
        Err(MarketplaceError::NotFound(slug.to_string()))
    }

    /// Download plugin
    pub async fn download(&self, slug: &str) -> Result<Vec<u8>, MarketplaceError> {
        let plugin = self.get_plugin(slug).await?;

        // Would download from URL
        let _ = plugin.download_url;
        Ok(Vec::new())
    }

    /// Install plugin from marketplace
    pub async fn install(&self, slug: &str, install_path: &std::path::Path) -> Result<(), MarketplaceError> {
        let data = self.download(slug).await?;

        // Would extract and install
        let _ = (data, install_path);
        Ok(())
    }

    /// Submit a review
    pub async fn submit_review(&self, slug: &str, rating: u8, content: &str) -> Result<(), MarketplaceError> {
        if self.api_key.is_none() {
            return Err(MarketplaceError::AuthRequired);
        }

        // Would submit review
        let _ = (slug, rating, content);
        Ok(())
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        *self.cache.write() = MarketplaceCache::default();
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub plugins: Vec<MarketplacePlugin>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Marketplace error
#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Authentication required")]
    AuthRequired,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),
}

// ============================================================================
// Conflict Detection (Point 179)
// ============================================================================

/// Conflict detector
pub struct ConflictDetector {
    /// Known conflicts
    known_conflicts: Vec<KnownConflict>,
    /// Detected conflicts
    detected: Arc<RwLock<Vec<DetectedConflict>>>,
}

/// Known conflict definition
#[derive(Debug, Clone)]
pub struct KnownConflict {
    pub plugin_a: String,
    pub plugin_b: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub resolution: Option<String>,
}

/// Conflict type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// Plugins cannot be active at the same time
    Incompatible,
    /// Plugins have overlapping functionality
    Overlap,
    /// Resource conflict (hooks, routes, etc.)
    Resource,
    /// Version conflict
    Version,
}

/// Detected conflict
#[derive(Debug, Clone)]
pub struct DetectedConflict {
    pub plugin_a: String,
    pub plugin_b: String,
    pub conflict_type: ConflictType,
    pub resource: Option<String>,
    pub description: String,
    pub severity: ConflictSeverity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Conflict severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            known_conflicts: Vec::new(),
            detected: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add known conflict
    pub fn add_known_conflict(&mut self, conflict: KnownConflict) {
        self.known_conflicts.push(conflict);
    }

    /// Check for conflicts between plugins
    pub fn check(&self, active_plugins: &[String]) -> Vec<DetectedConflict> {
        let mut conflicts = Vec::new();

        // Check known conflicts
        for conflict in &self.known_conflicts {
            if active_plugins.contains(&conflict.plugin_a) && active_plugins.contains(&conflict.plugin_b) {
                conflicts.push(DetectedConflict {
                    plugin_a: conflict.plugin_a.clone(),
                    plugin_b: conflict.plugin_b.clone(),
                    conflict_type: conflict.conflict_type,
                    resource: None,
                    description: conflict.description.clone(),
                    severity: ConflictSeverity::High,
                    detected_at: chrono::Utc::now(),
                });
            }
        }

        *self.detected.write() = conflicts.clone();
        conflicts
    }

    /// Check for hook conflicts
    pub fn check_hooks(&self, hooks: &HashMap<String, Vec<String>>) -> Vec<DetectedConflict> {
        let mut conflicts = Vec::new();

        for (hook, plugins) in hooks {
            if plugins.len() > 1 {
                // Multiple plugins on same hook - potential conflict
                for i in 0..plugins.len() {
                    for j in (i + 1)..plugins.len() {
                        conflicts.push(DetectedConflict {
                            plugin_a: plugins[i].clone(),
                            plugin_b: plugins[j].clone(),
                            conflict_type: ConflictType::Resource,
                            resource: Some(format!("hook:{}", hook)),
                            description: format!("Both plugins hook into '{}'", hook),
                            severity: ConflictSeverity::Low,
                            detected_at: chrono::Utc::now(),
                        });
                    }
                }
            }
        }

        conflicts
    }

    /// Check for route conflicts
    pub fn check_routes(&self, routes: &[(String, String)]) -> Vec<DetectedConflict> {
        let mut conflicts = Vec::new();
        let mut route_map: HashMap<String, Vec<String>> = HashMap::new();

        for (plugin_id, route) in routes {
            route_map
                .entry(route.clone())
                .or_insert_with(Vec::new)
                .push(plugin_id.clone());
        }

        for (route, plugins) in route_map {
            if plugins.len() > 1 {
                conflicts.push(DetectedConflict {
                    plugin_a: plugins[0].clone(),
                    plugin_b: plugins[1].clone(),
                    conflict_type: ConflictType::Resource,
                    resource: Some(format!("route:{}", route)),
                    description: format!("Route conflict on '{}'", route),
                    severity: ConflictSeverity::High,
                    detected_at: chrono::Utc::now(),
                });
            }
        }

        conflicts
    }

    /// Get all detected conflicts
    pub fn get_detected(&self) -> Vec<DetectedConflict> {
        self.detected.read().clone()
    }

    /// Clear detected conflicts
    pub fn clear(&self) {
        self.detected.write().clear();
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_checker() {
        let checker = UpdateChecker::new(UpdateConfig::default());
        assert!(checker.is_cache_stale());
    }

    #[test]
    fn test_conflict_detection() {
        let mut detector = ConflictDetector::new();
        detector.add_known_conflict(KnownConflict {
            plugin_a: "plugin-a".to_string(),
            plugin_b: "plugin-b".to_string(),
            conflict_type: ConflictType::Incompatible,
            description: "These plugins conflict".to_string(),
            resolution: None,
        });

        let conflicts = detector.check(&["plugin-a".to_string(), "plugin-b".to_string()]);
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_route_conflict() {
        let detector = ConflictDetector::new();
        let routes = vec![
            ("plugin-a".to_string(), "/api/users".to_string()),
            ("plugin-b".to_string(), "/api/users".to_string()),
        ];

        let conflicts = detector.check_routes(&routes);
        assert_eq!(conflicts.len(), 1);
    }
}
