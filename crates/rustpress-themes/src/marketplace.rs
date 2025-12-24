//! Theme Marketplace
//!
//! Browse, search, and install themes from remote repositories.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// Marketplace errors
#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Marketplace configuration
#[derive(Debug, Clone)]
pub struct MarketplaceConfig {
    /// API base URL
    pub api_url: String,
    /// Cache duration in seconds
    pub cache_duration: u64,
    /// Download directory
    pub download_dir: PathBuf,
    /// Number of results per page
    pub per_page: u32,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.rustpress.org/themes".to_string(),
            cache_duration: 3600, // 1 hour
            download_dir: PathBuf::from("temp/theme-downloads"),
            per_page: 24,
        }
    }
}

/// Theme listing from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeListing {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub version: String,
    pub author: ThemeAuthor,
    pub description: String,
    pub short_description: Option<String>,
    pub homepage: Option<String>,
    pub preview_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub download_url: String,
    pub download_count: u64,
    pub rating: f32,
    pub ratings_count: u32,
    pub last_updated: String,
    pub requires: Option<String>,
    pub requires_php: Option<String>,
    pub tags: Vec<String>,
    pub features: Vec<String>,
    pub is_block_theme: bool,
    pub price: Option<Price>,
}

/// Theme author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAuthor {
    pub name: String,
    pub url: Option<String>,
    pub avatar: Option<String>,
}

/// Theme price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
}

/// Search parameters
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: Option<String>,
    pub tag: Option<String>,
    pub feature: Option<String>,
    pub author: Option<String>,
    pub sort: SortOption,
    pub page: u32,
    pub per_page: u32,
    pub block_themes_only: bool,
    pub free_only: bool,
}

/// Sort options
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOption {
    #[default]
    Popular,
    New,
    Updated,
    Rating,
    Downloads,
    Name,
}

impl SortOption {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Popular => "popular",
            Self::New => "new",
            Self::Updated => "updated",
            Self::Rating => "rating",
            Self::Downloads => "downloads",
            Self::Name => "name",
        }
    }
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub themes: Vec<ThemeListing>,
    pub total: u64,
    pub page: u32,
    pub total_pages: u32,
}

/// Theme marketplace client
pub struct MarketplaceClient {
    config: MarketplaceConfig,
    client: reqwest::Client,
    /// Cache of search results
    search_cache: Arc<RwLock<HashMap<String, CachedResult<SearchResults>>>>,
    /// Cache of theme details
    theme_cache: Arc<RwLock<HashMap<String, CachedResult<ThemeListing>>>>,
}

/// Cached result with expiration
#[derive(Debug, Clone)]
struct CachedResult<T> {
    data: T,
    expires_at: std::time::Instant,
}

impl MarketplaceClient {
    pub fn new(config: MarketplaceConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("RustPress/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            search_cache: Arc::new(RwLock::new(HashMap::new())),
            theme_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Search for themes
    pub async fn search(&self, params: &SearchParams) -> Result<SearchResults, MarketplaceError> {
        let cache_key = format!("{:?}", params);

        // Check cache
        if let Some(cached) = self.get_cached_search(&cache_key) {
            return Ok(cached);
        }

        // Build query
        let mut query_params = vec![
            ("page", params.page.to_string()),
            ("per_page", params.per_page.to_string()),
            ("sort", params.sort.as_str().to_string()),
        ];

        if let Some(ref q) = params.query {
            query_params.push(("search", q.clone()));
        }

        if let Some(ref tag) = params.tag {
            query_params.push(("tag", tag.clone()));
        }

        if let Some(ref feature) = params.feature {
            query_params.push(("feature", feature.clone()));
        }

        if let Some(ref author) = params.author {
            query_params.push(("author", author.clone()));
        }

        if params.block_themes_only {
            query_params.push(("block_theme", "true".to_string()));
        }

        if params.free_only {
            query_params.push(("free", "true".to_string()));
        }

        let url = format!("{}/search", self.config.api_url);

        let response = self.client
            .get(&url)
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let results: SearchResults = response.json().await?;

        // Cache results
        self.cache_search(cache_key, results.clone());

        Ok(results)
    }

    /// Get theme details
    pub async fn get_theme(&self, slug: &str) -> Result<ThemeListing, MarketplaceError> {
        // Check cache
        if let Some(cached) = self.get_cached_theme(slug) {
            return Ok(cached);
        }

        let url = format!("{}/{}", self.config.api_url, slug);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(MarketplaceError::NotFound(slug.to_string()));
        }

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let theme: ThemeListing = response.json().await?;

        // Cache result
        self.cache_theme(slug.to_string(), theme.clone());

        Ok(theme)
    }

    /// Download a theme
    pub async fn download(&self, theme: &ThemeListing) -> Result<PathBuf, MarketplaceError> {
        fs::create_dir_all(&self.config.download_dir).await?;

        let filename = format!("{}-{}.zip", theme.slug, theme.version);
        let download_path = self.config.download_dir.join(&filename);

        // Check if already downloaded
        if download_path.exists() {
            return Ok(download_path);
        }

        // Download file
        let response = self.client
            .get(&theme.download_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::DownloadFailed(
                response.status().to_string(),
            ));
        }

        let bytes = response.bytes().await?;
        fs::write(&download_path, bytes).await?;

        Ok(download_path)
    }

    /// Install a theme from marketplace
    pub async fn install(
        &self,
        theme: &ThemeListing,
        themes_dir: &Path,
    ) -> Result<PathBuf, MarketplaceError> {
        // Download theme
        let zip_path = self.download(theme).await?;

        // Extract to themes directory
        let importer = crate::export::ThemeImporter::new(themes_dir.to_path_buf());
        let theme_id = importer.import_zip(&zip_path).await
            .map_err(|e| MarketplaceError::InstallFailed(e.to_string()))?;

        // Clean up download
        let _ = fs::remove_file(zip_path).await;

        Ok(themes_dir.join(theme_id))
    }

    /// Get featured themes
    pub async fn get_featured(&self) -> Result<Vec<ThemeListing>, MarketplaceError> {
        let url = format!("{}/featured", self.config.api_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let themes: Vec<ThemeListing> = response.json().await?;
        Ok(themes)
    }

    /// Get popular themes
    pub async fn get_popular(&self, limit: u32) -> Result<Vec<ThemeListing>, MarketplaceError> {
        self.search(&SearchParams {
            sort: SortOption::Popular,
            per_page: limit,
            ..Default::default()
        })
        .await
        .map(|r| r.themes)
    }

    /// Get new themes
    pub async fn get_new(&self, limit: u32) -> Result<Vec<ThemeListing>, MarketplaceError> {
        self.search(&SearchParams {
            sort: SortOption::New,
            per_page: limit,
            ..Default::default()
        })
        .await
        .map(|r| r.themes)
    }

    /// Get available tags
    pub async fn get_tags(&self) -> Result<Vec<TagInfo>, MarketplaceError> {
        let url = format!("{}/tags", self.config.api_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let tags: Vec<TagInfo> = response.json().await?;
        Ok(tags)
    }

    /// Get available features
    pub async fn get_features(&self) -> Result<Vec<FeatureInfo>, MarketplaceError> {
        let url = format!("{}/features", self.config.api_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let features: Vec<FeatureInfo> = response.json().await?;
        Ok(features)
    }

    /// Check for theme updates
    pub async fn check_updates(
        &self,
        installed: &[(String, String)], // (slug, version)
    ) -> Result<Vec<UpdateInfo>, MarketplaceError> {
        let url = format!("{}/updates", self.config.api_url);

        let body: HashMap<String, String> = installed.iter().cloned().collect();

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(MarketplaceError::InvalidResponse(
                response.status().to_string(),
            ));
        }

        let updates: Vec<UpdateInfo> = response.json().await?;
        Ok(updates)
    }

    fn get_cached_search(&self, key: &str) -> Option<SearchResults> {
        let cache = self.search_cache.read();
        cache.get(key).and_then(|cached| {
            if cached.expires_at > std::time::Instant::now() {
                Some(cached.data.clone())
            } else {
                None
            }
        })
    }

    fn cache_search(&self, key: String, data: SearchResults) {
        let expires_at = std::time::Instant::now() +
            std::time::Duration::from_secs(self.config.cache_duration);

        self.search_cache.write().insert(key, CachedResult {
            data,
            expires_at,
        });
    }

    fn get_cached_theme(&self, slug: &str) -> Option<ThemeListing> {
        let cache = self.theme_cache.read();
        cache.get(slug).and_then(|cached| {
            if cached.expires_at > std::time::Instant::now() {
                Some(cached.data.clone())
            } else {
                None
            }
        })
    }

    fn cache_theme(&self, slug: String, data: ThemeListing) {
        let expires_at = std::time::Instant::now() +
            std::time::Duration::from_secs(self.config.cache_duration);

        self.theme_cache.write().insert(slug, CachedResult {
            data,
            expires_at,
        });
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.search_cache.write().clear();
        self.theme_cache.write().clear();
    }
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub slug: String,
    pub name: String,
    pub count: u64,
}

/// Feature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub count: u64,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub slug: String,
    pub current_version: String,
    pub new_version: String,
    pub download_url: String,
    pub changelog: Option<String>,
}

/// Common theme tags
pub const COMMON_TAGS: &[(&str, &str)] = &[
    ("blog", "Blog"),
    ("e-commerce", "E-Commerce"),
    ("portfolio", "Portfolio"),
    ("photography", "Photography"),
    ("business", "Business"),
    ("news", "News & Magazine"),
    ("education", "Education"),
    ("entertainment", "Entertainment"),
    ("food-drink", "Food & Drink"),
    ("holiday", "Holiday"),
];

/// Common theme features
pub const COMMON_FEATURES: &[(&str, &str)] = &[
    ("block-editor", "Block Editor"),
    ("full-site-editing", "Full Site Editing"),
    ("custom-colors", "Custom Colors"),
    ("custom-logo", "Custom Logo"),
    ("custom-menu", "Custom Menu"),
    ("featured-images", "Featured Images"),
    ("footer-widgets", "Footer Widgets"),
    ("full-width-template", "Full Width Template"),
    ("one-column", "One Column"),
    ("two-columns", "Two Columns"),
    ("three-columns", "Three Columns"),
    ("left-sidebar", "Left Sidebar"),
    ("right-sidebar", "Right Sidebar"),
    ("responsive-layout", "Responsive Layout"),
    ("rtl-support", "RTL Support"),
    ("translation-ready", "Translation Ready"),
    ("accessibility-ready", "Accessibility Ready"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_default() {
        let params = SearchParams::default();
        assert!(params.query.is_none());
        assert_eq!(params.page, 0);
    }

    #[test]
    fn test_sort_option_as_str() {
        assert_eq!(SortOption::Popular.as_str(), "popular");
        assert_eq!(SortOption::New.as_str(), "new");
        assert_eq!(SortOption::Rating.as_str(), "rating");
    }

    #[test]
    fn test_marketplace_config_default() {
        let config = MarketplaceConfig::default();
        assert!(config.api_url.contains("rustpress"));
        assert_eq!(config.per_page, 24);
    }
}
