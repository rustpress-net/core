//! Discovery Service - Auto-discovery for plugins, themes, and apps
//!
//! Supports both local directory scanning and remote URL fetching for
//! discovering and managing RustPress components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Source type for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiscoverySource {
    /// Local filesystem path
    Local { path: PathBuf },
    /// Remote URL (marketplace, GitHub, etc.)
    Remote {
        url: String,
        #[serde(default)]
        auth_token: Option<String>,
    },
    /// Git repository
    Git {
        url: String,
        #[serde(default = "default_branch")]
        branch: String,
        #[serde(default)]
        auth_token: Option<String>,
    },
}

fn default_branch() -> String {
    "main".to_string()
}

/// Component type being discovered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Plugin,
    Theme,
    App,
    Function,
}

impl ComponentType {
    pub fn manifest_filename(&self) -> &'static str {
        match self {
            ComponentType::Plugin => "plugin.toml",
            ComponentType::Theme => "theme.toml",
            ComponentType::App => "app.toml",
            ComponentType::Function => "function.toml",
        }
    }

    pub fn alt_manifest_filename(&self) -> Option<&'static str> {
        match self {
            ComponentType::Theme => Some("style.css"), // WordPress-style theme detection
            _ => None,
        }
    }
}

/// Generic component manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub author_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub min_rustpress_version: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    /// Source information (where this was discovered from)
    #[serde(skip)]
    pub source: Option<DiscoverySource>,
    /// Local path if installed
    #[serde(skip)]
    pub local_path: Option<PathBuf>,
    /// Additional component-specific data
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Result of a discovery scan
#[derive(Debug, Default)]
pub struct DiscoveryResult {
    /// Successfully discovered components
    pub discovered: Vec<ComponentManifest>,
    /// Errors encountered during scanning
    pub errors: Vec<String>,
    /// Paths/URLs that were skipped
    pub skipped: Vec<String>,
}

impl DiscoveryResult {
    pub fn merge(&mut self, other: DiscoveryResult) {
        self.discovered.extend(other.discovered);
        self.errors.extend(other.errors);
        self.skipped.extend(other.skipped);
    }
}

/// Discovery service for finding components
pub struct DiscoveryService {
    /// Configured sources for discovery
    sources: Vec<DiscoverySource>,
    /// HTTP client for remote fetching
    #[cfg(feature = "remote-discovery")]
    client: Option<reqwest::Client>,
}

impl DiscoveryService {
    /// Create a new discovery service with local path
    pub fn new(local_path: impl Into<PathBuf>) -> Self {
        Self {
            sources: vec![DiscoverySource::Local {
                path: local_path.into(),
            }],
            #[cfg(feature = "remote-discovery")]
            client: None,
        }
    }

    /// Create a discovery service with multiple sources
    pub fn with_sources(sources: Vec<DiscoverySource>) -> Self {
        Self {
            sources,
            #[cfg(feature = "remote-discovery")]
            client: None,
        }
    }

    /// Add a local source
    pub fn add_local_source(&mut self, path: impl Into<PathBuf>) {
        self.sources.push(DiscoverySource::Local { path: path.into() });
    }

    /// Add a remote source
    pub fn add_remote_source(&mut self, url: impl Into<String>, auth_token: Option<String>) {
        self.sources.push(DiscoverySource::Remote {
            url: url.into(),
            auth_token,
        });
    }

    /// Add a git source
    pub fn add_git_source(
        &mut self,
        url: impl Into<String>,
        branch: Option<String>,
        auth_token: Option<String>,
    ) {
        self.sources.push(DiscoverySource::Git {
            url: url.into(),
            branch: branch.unwrap_or_else(default_branch),
            auth_token,
        });
    }

    /// Discover components of a specific type
    pub async fn discover(&self, component_type: ComponentType) -> DiscoveryResult {
        let mut result = DiscoveryResult::default();

        for source in &self.sources {
            let source_result = match source {
                DiscoverySource::Local { path } => {
                    self.scan_local_directory(path, component_type).await
                }
                DiscoverySource::Remote { url, auth_token } => {
                    self.fetch_remote_manifest(url, auth_token.as_deref(), component_type)
                        .await
                }
                DiscoverySource::Git {
                    url,
                    branch,
                    auth_token,
                } => {
                    self.discover_from_git(url, branch, auth_token.as_deref(), component_type)
                        .await
                }
            };
            result.merge(source_result);
        }

        info!(
            component_type = ?component_type,
            discovered = result.discovered.len(),
            errors = result.errors.len(),
            skipped = result.skipped.len(),
            "Discovery complete"
        );

        result
    }

    /// Scan a local directory for components
    async fn scan_local_directory(
        &self,
        path: &Path,
        component_type: ComponentType,
    ) -> DiscoveryResult {
        let mut result = DiscoveryResult::default();

        if !path.exists() {
            warn!(path = ?path, "Directory does not exist");
            return result;
        }

        info!(path = ?path, component_type = ?component_type, "Scanning local directory");

        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to read directory {}: {}", path.display(), e));
                return result;
            }
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();

            if !entry_path.is_dir() {
                continue;
            }

            // Look for manifest file
            let manifest_path = entry_path.join(component_type.manifest_filename());
            let alt_manifest_path = component_type
                .alt_manifest_filename()
                .map(|f| entry_path.join(f));

            let manifest_to_load = if manifest_path.exists() {
                Some(manifest_path)
            } else if let Some(alt_path) = alt_manifest_path {
                if alt_path.exists() {
                    Some(alt_path)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(manifest_file) = manifest_to_load {
                match self.load_local_manifest(&manifest_file, component_type) {
                    Ok(mut manifest) => {
                        manifest.source = Some(DiscoverySource::Local {
                            path: path.to_path_buf(),
                        });
                        manifest.local_path = Some(entry_path);
                        info!(
                            id = %manifest.id,
                            name = %manifest.name,
                            version = %manifest.version,
                            "Discovered component"
                        );
                        result.discovered.push(manifest);
                    }
                    Err(e) => {
                        result.errors.push(format!(
                            "Failed to load {}: {}",
                            manifest_file.display(),
                            e
                        ));
                    }
                }
            } else {
                debug!(path = ?entry_path, "No manifest found, skipping");
                result.skipped.push(entry_path.display().to_string());
            }
        }

        result
    }

    /// Load a manifest from a local file
    fn load_local_manifest(
        &self,
        path: &Path,
        component_type: ComponentType,
    ) -> Result<ComponentManifest, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Handle different manifest formats
        if path.extension().map_or(false, |ext| ext == "css") {
            // Parse WordPress-style theme header from CSS
            self.parse_css_manifest(&content, component_type)
        } else if path.extension().map_or(false, |ext| ext == "toml") {
            toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))
        } else if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
        } else {
            // Try TOML first, then JSON
            toml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content).map_err(|e| e.to_string()))
                .map_err(|e| format!("Failed to parse manifest: {}", e))
        }
    }

    /// Parse a WordPress-style CSS theme header
    fn parse_css_manifest(
        &self,
        content: &str,
        _component_type: ComponentType,
    ) -> Result<ComponentManifest, String> {
        let mut manifest = ComponentManifest {
            id: String::new(),
            name: String::new(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            author_url: None,
            license: None,
            homepage: None,
            repository: None,
            min_rustpress_version: None,
            tags: Vec::new(),
            category: None,
            source: None,
            local_path: None,
            extra: HashMap::new(),
        };

        // Parse CSS header comments
        // Format: /* Theme Name: My Theme */ or multiline:
        // /*
        // Theme Name: My Theme
        // */
        let mut in_comment = false;
        for line in content.lines() {
            let line = line.trim();

            // Track whether we're inside a comment block
            if line.contains("/*") {
                in_comment = true;
            }

            // Process if we're in a comment or this is a comment line
            if in_comment || line.starts_with("/*") || line.starts_with("*") {
                let line = line
                    .trim_start_matches("/*")
                    .trim_start_matches("*")
                    .trim_end_matches("*/")
                    .trim();

                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_lowercase();
                    let value = value.trim().to_string();

                    match key.as_str() {
                        "theme name" => manifest.name = value.clone(),
                        "theme id" | "text domain" => manifest.id = value,
                        "version" => manifest.version = value,
                        "description" => manifest.description = Some(value),
                        "author" => manifest.author = Some(value),
                        "author uri" | "author url" => manifest.author_url = Some(value),
                        "license" => manifest.license = Some(value),
                        "theme uri" | "theme url" => manifest.homepage = Some(value),
                        "tags" => {
                            manifest.tags = value.split(',').map(|s| s.trim().to_string()).collect()
                        }
                        _ => {}
                    }
                }
            }

            // End comment block tracking
            if line.contains("*/") {
                in_comment = false;
            }
        }

        // Generate ID from name if not provided
        if manifest.id.is_empty() && !manifest.name.is_empty() {
            manifest.id = manifest
                .name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect();
        }

        if manifest.id.is_empty() {
            return Err("No theme ID or name found in CSS header".to_string());
        }

        Ok(manifest)
    }

    /// Fetch manifest from a remote URL
    async fn fetch_remote_manifest(
        &self,
        url: &str,
        auth_token: Option<&str>,
        component_type: ComponentType,
    ) -> DiscoveryResult {
        let mut result = DiscoveryResult::default();

        info!(url = %url, component_type = ?component_type, "Fetching from remote URL");

        // Build request
        let client = reqwest::Client::new();
        let mut request = client.get(url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        match request.send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    result.errors.push(format!(
                        "Failed to fetch {}: HTTP {}",
                        url,
                        response.status()
                    ));
                    return result;
                }

                match response.text().await {
                    Ok(content) => {
                        // Try to parse as a list of manifests or a single manifest
                        if let Ok(manifests) = serde_json::from_str::<Vec<ComponentManifest>>(&content) {
                            for mut manifest in manifests {
                                manifest.source = Some(DiscoverySource::Remote {
                                    url: url.to_string(),
                                    auth_token: auth_token.map(|s| s.to_string()),
                                });
                                result.discovered.push(manifest);
                            }
                        } else if let Ok(mut manifest) =
                            serde_json::from_str::<ComponentManifest>(&content)
                        {
                            manifest.source = Some(DiscoverySource::Remote {
                                url: url.to_string(),
                                auth_token: auth_token.map(|s| s.to_string()),
                            });
                            result.discovered.push(manifest);
                        } else {
                            result.errors.push(format!(
                                "Failed to parse response from {}: invalid format",
                                url
                            ));
                        }
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("Failed to read response from {}: {}", url, e));
                    }
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to connect to {}: {}", url, e));
            }
        }

        result
    }

    /// Discover components from a git repository
    async fn discover_from_git(
        &self,
        url: &str,
        branch: &str,
        auth_token: Option<&str>,
        component_type: ComponentType,
    ) -> DiscoveryResult {
        let mut result = DiscoveryResult::default();

        info!(url = %url, branch = %branch, component_type = ?component_type, "Discovering from Git repository");

        // For GitHub/GitLab, we can use their APIs to fetch manifests directly
        if url.contains("github.com") {
            return self
                .discover_from_github(url, branch, auth_token, component_type)
                .await;
        }

        // For other git providers, we'd need to clone - skip for now
        result
            .errors
            .push(format!("Git discovery only supported for GitHub: {}", url));
        result
    }

    /// Discover components from GitHub
    async fn discover_from_github(
        &self,
        url: &str,
        branch: &str,
        auth_token: Option<&str>,
        component_type: ComponentType,
    ) -> DiscoveryResult {
        let mut result = DiscoveryResult::default();

        // Parse GitHub URL to extract owner/repo
        let parts: Vec<&str> = url
            .trim_end_matches(".git")
            .split('/')
            .collect();

        if parts.len() < 2 {
            result
                .errors
                .push(format!("Invalid GitHub URL format: {}", url));
            return result;
        }

        let repo = parts[parts.len() - 1];
        let owner = parts[parts.len() - 2];

        // Use GitHub raw content URL
        let manifest_filename = component_type.manifest_filename();
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, branch, manifest_filename
        );

        let client = reqwest::Client::new();
        let mut request = client.get(&raw_url);

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("token {}", token));
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(content) = response.text().await {
                        match toml::from_str::<ComponentManifest>(&content) {
                            Ok(mut manifest) => {
                                manifest.source = Some(DiscoverySource::Git {
                                    url: url.to_string(),
                                    branch: branch.to_string(),
                                    auth_token: auth_token.map(|s| s.to_string()),
                                });
                                manifest.repository = Some(url.to_string());
                                result.discovered.push(manifest);
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to parse manifest from {}: {}",
                                    raw_url, e
                                ));
                            }
                        }
                    }
                } else {
                    result.errors.push(format!(
                        "Manifest not found at {}: HTTP {}",
                        raw_url,
                        response.status()
                    ));
                }
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to fetch from GitHub {}: {}", raw_url, e));
            }
        }

        result
    }
}

/// Configuration for discovery sources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryConfig {
    /// Plugin discovery sources
    #[serde(default)]
    pub plugins: Vec<DiscoverySource>,
    /// Theme discovery sources
    #[serde(default)]
    pub themes: Vec<DiscoverySource>,
    /// App discovery sources
    #[serde(default)]
    pub apps: Vec<DiscoverySource>,
    /// Function discovery sources
    #[serde(default)]
    pub functions: Vec<DiscoverySource>,
}

impl DiscoveryConfig {
    /// Create a default config with local paths
    pub fn with_local_paths(base_path: &Path) -> Self {
        Self {
            plugins: vec![DiscoverySource::Local {
                path: base_path.join("plugins"),
            }],
            themes: vec![DiscoverySource::Local {
                path: base_path.join("themes"),
            }],
            apps: vec![DiscoverySource::Local {
                path: base_path.join("apps"),
            }],
            functions: vec![DiscoverySource::Local {
                path: base_path.join("functions"),
            }],
        }
    }

    /// Add a remote marketplace for all component types
    pub fn add_marketplace(&mut self, url: &str, auth_token: Option<String>) {
        let source = DiscoverySource::Remote {
            url: url.to_string(),
            auth_token,
        };
        self.plugins.push(source.clone());
        self.themes.push(source.clone());
        self.apps.push(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css_manifest() {
        let css = r#"/*
Theme Name: My Awesome Theme
Theme ID: my-awesome-theme
Version: 2.0.0
Description: A beautiful responsive theme
Author: John Doe
Author URI: https://johndoe.com
Tags: responsive, modern, dark
*/

body { background: #000; }
"#;

        let service = DiscoveryService::new("/tmp");
        let manifest = service
            .parse_css_manifest(css, ComponentType::Theme)
            .unwrap();

        assert_eq!(manifest.id, "my-awesome-theme");
        assert_eq!(manifest.name, "My Awesome Theme");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.author, Some("John Doe".to_string()));
        assert_eq!(manifest.tags, vec!["responsive", "modern", "dark"]);
    }

    #[test]
    fn test_component_manifest_filenames() {
        assert_eq!(ComponentType::Plugin.manifest_filename(), "plugin.toml");
        assert_eq!(ComponentType::Theme.manifest_filename(), "theme.toml");
        assert_eq!(ComponentType::App.manifest_filename(), "app.toml");
    }
}
