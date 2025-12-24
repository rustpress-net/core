//! Plugin Asset Management
//!
//! Handles CSS, JavaScript, and static file assets for plugins.

use crate::manifest::{AssetFile, AssetLocation, AssetsSection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

/// Asset manager for plugins
pub struct AssetManager {
    /// Registered assets per plugin
    assets: Arc<RwLock<HashMap<String, Vec<RegisteredAsset>>>>,
    /// Asset dependencies
    dependencies: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Asset base path
    base_path: PathBuf,
    /// Asset URL prefix
    url_prefix: String,
}

/// Registered asset
#[derive(Debug, Clone)]
pub struct RegisteredAsset {
    pub plugin_id: String,
    pub handle: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub url: String,
    pub version: Option<String>,
    pub dependencies: Vec<String>,
    pub location: AssetLocation,
    pub admin_only: bool,
    pub frontend_only: bool,
    pub condition: Option<AssetCondition>,
    pub attributes: HashMap<String, String>,
    pub inline_content: Option<String>,
    pub is_enqueued: bool,
}

/// Asset type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Css,
    JavaScript,
    Image,
    Font,
    Other,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "css" => AssetType::Css,
            "js" | "mjs" => AssetType::JavaScript,
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => AssetType::Image,
            "woff" | "woff2" | "ttf" | "eot" | "otf" => AssetType::Font,
            _ => AssetType::Other,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            AssetType::Css => "text/css",
            AssetType::JavaScript => "application/javascript",
            AssetType::Image => "image/*",
            AssetType::Font => "font/*",
            AssetType::Other => "application/octet-stream",
        }
    }
}

/// Asset condition for conditional loading
#[derive(Debug, Clone)]
pub struct AssetCondition {
    pub condition_type: ConditionType,
    pub value: String,
}

/// Condition type
#[derive(Debug, Clone)]
pub enum ConditionType {
    /// Load on specific page types
    PageType(Vec<String>),
    /// Load on specific post types
    PostType(Vec<String>),
    /// Load on specific URLs/paths
    UrlPattern(String),
    /// Load based on custom callback
    Custom(String),
    /// Load in specific areas
    Area(AssetArea),
}

/// Asset area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetArea {
    Frontend,
    Admin,
    Login,
    All,
}

/// Asset bundle for combining multiple assets
#[derive(Debug, Clone)]
pub struct AssetBundle {
    pub name: String,
    pub assets: Vec<String>,
    pub output_path: PathBuf,
    pub minified: bool,
}

impl AssetManager {
    pub fn new(base_path: PathBuf, url_prefix: &str) -> Self {
        Self {
            assets: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            base_path,
            url_prefix: url_prefix.to_string(),
        }
    }

    /// Register assets from plugin manifest
    pub fn register_from_manifest(
        &self,
        plugin_id: &str,
        plugin_path: &Path,
        section: &AssetsSection,
    ) {
        // Register CSS assets
        for css in &section.css {
            self.register_asset(plugin_id, plugin_path, css, AssetType::Css);
        }

        // Register JS assets
        for js in &section.js {
            self.register_asset(plugin_id, plugin_path, js, AssetType::JavaScript);
        }

        debug!("Registered assets for plugin: {}", plugin_id);
    }

    /// Register a single asset
    fn register_asset(
        &self,
        plugin_id: &str,
        plugin_path: &Path,
        asset: &AssetFile,
        asset_type: AssetType,
    ) {
        let full_path = plugin_path.join(&asset.path);
        let handle = asset.handle.clone().unwrap_or_else(|| {
            format!(
                "{}-{}",
                plugin_id,
                full_path.file_stem().unwrap_or_default().to_string_lossy()
            )
        });

        let url = format!(
            "{}/plugins/{}/{}",
            self.url_prefix,
            plugin_id,
            asset.path
        );

        let condition = asset.condition.as_ref().map(|c| AssetCondition {
            condition_type: ConditionType::Custom(c.clone()),
            value: c.clone(),
        });

        let registered = RegisteredAsset {
            plugin_id: plugin_id.to_string(),
            handle: handle.clone(),
            asset_type,
            path: full_path,
            url,
            version: None,
            dependencies: asset.dependencies.clone(),
            location: asset.location,
            admin_only: asset.admin_only,
            frontend_only: asset.frontend_only,
            condition,
            attributes: HashMap::new(),
            inline_content: None,
            is_enqueued: false,
        };

        let mut assets = self.assets.write();
        assets
            .entry(plugin_id.to_string())
            .or_insert_with(Vec::new)
            .push(registered);

        // Store dependencies
        let mut deps = self.dependencies.write();
        deps.insert(handle.clone(), asset.dependencies.clone());
    }

    /// Register a CSS asset programmatically
    pub fn register_style(
        &self,
        plugin_id: &str,
        handle: &str,
        path: &str,
        dependencies: Vec<String>,
    ) {
        self.register_asset_internal(
            plugin_id,
            handle,
            path,
            AssetType::Css,
            dependencies,
            AssetLocation::Header,
            false,
            false,
        );
    }

    /// Register a JavaScript asset programmatically
    pub fn register_script(
        &self,
        plugin_id: &str,
        handle: &str,
        path: &str,
        dependencies: Vec<String>,
        in_footer: bool,
    ) {
        let location = if in_footer {
            AssetLocation::Footer
        } else {
            AssetLocation::Header
        };
        self.register_asset_internal(
            plugin_id,
            handle,
            path,
            AssetType::JavaScript,
            dependencies,
            location,
            false,
            false,
        );
    }

    fn register_asset_internal(
        &self,
        plugin_id: &str,
        handle: &str,
        path: &str,
        asset_type: AssetType,
        dependencies: Vec<String>,
        location: AssetLocation,
        admin_only: bool,
        frontend_only: bool,
    ) {
        let full_path = self.base_path.join("plugins").join(plugin_id).join(path);
        let url = format!("{}/plugins/{}/{}", self.url_prefix, plugin_id, path);

        let registered = RegisteredAsset {
            plugin_id: plugin_id.to_string(),
            handle: handle.to_string(),
            asset_type,
            path: full_path,
            url,
            version: None,
            dependencies: dependencies.clone(),
            location,
            admin_only,
            frontend_only,
            condition: None,
            attributes: HashMap::new(),
            inline_content: None,
            is_enqueued: false,
        };

        let mut assets = self.assets.write();
        assets
            .entry(plugin_id.to_string())
            .or_insert_with(Vec::new)
            .push(registered);

        let mut deps = self.dependencies.write();
        deps.insert(handle.to_string(), dependencies);
    }

    /// Enqueue an asset for output
    pub fn enqueue(&self, handle: &str) {
        let mut assets = self.assets.write();
        for plugin_assets in assets.values_mut() {
            for asset in plugin_assets.iter_mut() {
                if asset.handle == handle {
                    asset.is_enqueued = true;
                    return;
                }
            }
        }
        warn!("Asset not found: {}", handle);
    }

    /// Dequeue an asset
    pub fn dequeue(&self, handle: &str) {
        let mut assets = self.assets.write();
        for plugin_assets in assets.values_mut() {
            for asset in plugin_assets.iter_mut() {
                if asset.handle == handle {
                    asset.is_enqueued = false;
                    return;
                }
            }
        }
    }

    /// Get all enqueued assets for a location
    pub fn get_enqueued(&self, location: AssetLocation, is_admin: bool) -> Vec<RegisteredAsset> {
        let assets = self.assets.read();
        let mut result: Vec<RegisteredAsset> = Vec::new();

        for plugin_assets in assets.values() {
            for asset in plugin_assets {
                if !asset.is_enqueued {
                    continue;
                }

                if asset.location != location {
                    continue;
                }

                // Check admin/frontend restrictions
                if asset.admin_only && !is_admin {
                    continue;
                }
                if asset.frontend_only && is_admin {
                    continue;
                }

                result.push(asset.clone());
            }
        }

        // Sort by dependencies
        self.sort_by_dependencies(&mut result);
        result
    }

    /// Sort assets by their dependencies
    fn sort_by_dependencies(&self, assets: &mut Vec<RegisteredAsset>) {
        let deps = self.dependencies.read();

        assets.sort_by(|a, b| {
            let a_deps = deps.get(&a.handle).cloned().unwrap_or_default();
            let b_deps = deps.get(&b.handle).cloned().unwrap_or_default();

            // If A depends on B, A should come after B
            if a_deps.contains(&b.handle) {
                std::cmp::Ordering::Greater
            } else if b_deps.contains(&a.handle) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }

    /// Generate HTML for header assets
    pub fn render_header(&self, is_admin: bool) -> String {
        let mut html = String::new();

        // CSS assets
        for asset in self.get_enqueued(AssetLocation::Header, is_admin) {
            if asset.asset_type == AssetType::Css {
                html.push_str(&self.render_css(&asset));
            } else if asset.asset_type == AssetType::JavaScript {
                html.push_str(&self.render_js(&asset));
            }
        }

        html
    }

    /// Generate HTML for footer assets
    pub fn render_footer(&self, is_admin: bool) -> String {
        let mut html = String::new();

        for asset in self.get_enqueued(AssetLocation::Footer, is_admin) {
            if asset.asset_type == AssetType::JavaScript {
                html.push_str(&self.render_js(&asset));
            }
        }

        html
    }

    /// Render CSS link tag
    fn render_css(&self, asset: &RegisteredAsset) -> String {
        let version = asset.version.as_deref().unwrap_or("1.0.0");
        let url = format!("{}?v={}", asset.url, version);

        let mut attrs = String::new();
        for (key, value) in &asset.attributes {
            attrs.push_str(&format!(" {}=\"{}\"", key, value));
        }

        format!(
            r#"<link rel="stylesheet" id="{}-css" href="{}"{}>{}"#,
            asset.handle,
            url,
            attrs,
            "\n"
        )
    }

    /// Render JavaScript script tag
    fn render_js(&self, asset: &RegisteredAsset) -> String {
        let version = asset.version.as_deref().unwrap_or("1.0.0");
        let url = format!("{}?v={}", asset.url, version);

        let mut attrs = String::new();
        for (key, value) in &asset.attributes {
            attrs.push_str(&format!(" {}=\"{}\"", key, value));
        }

        if let Some(inline) = &asset.inline_content {
            format!(
                r#"<script id="{}-js" src="{}"{}></script><script id="{}-js-extra">{}</script>{}"#,
                asset.handle, url, attrs, asset.handle, inline, "\n"
            )
        } else {
            format!(
                r#"<script id="{}-js" src="{}"{}></script>{}"#,
                asset.handle, url, attrs, "\n"
            )
        }
    }

    /// Add inline content to an asset
    pub fn add_inline(&self, handle: &str, content: &str, position: InlinePosition) {
        let mut assets = self.assets.write();
        for plugin_assets in assets.values_mut() {
            for asset in plugin_assets.iter_mut() {
                if asset.handle == handle {
                    let existing = asset.inline_content.take().unwrap_or_default();
                    asset.inline_content = Some(match position {
                        InlinePosition::Before => format!("{}\n{}", content, existing),
                        InlinePosition::After => format!("{}\n{}", existing, content),
                    });
                    return;
                }
            }
        }
    }

    /// Localize a script (add data for JavaScript)
    pub fn localize(&self, handle: &str, object_name: &str, data: serde_json::Value) {
        let script = format!(
            "var {} = {};",
            object_name,
            serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())
        );
        self.add_inline(handle, &script, InlinePosition::Before);
    }

    /// Get asset by handle
    pub fn get(&self, handle: &str) -> Option<RegisteredAsset> {
        let assets = self.assets.read();
        for plugin_assets in assets.values() {
            for asset in plugin_assets {
                if asset.handle == handle {
                    return Some(asset.clone());
                }
            }
        }
        None
    }

    /// Get all assets for a plugin
    pub fn get_plugin_assets(&self, plugin_id: &str) -> Vec<RegisteredAsset> {
        self.assets
            .read()
            .get(plugin_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Remove all assets for a plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        let handles: Vec<String> = self
            .assets
            .read()
            .get(plugin_id)
            .map(|a| a.iter().map(|asset| asset.handle.clone()).collect())
            .unwrap_or_default();

        self.assets.write().remove(plugin_id);

        let mut deps = self.dependencies.write();
        for handle in handles {
            deps.remove(&handle);
        }
    }

    /// Get asset file contents
    pub fn get_contents(&self, handle: &str) -> Option<Vec<u8>> {
        let asset = self.get(handle)?;
        std::fs::read(&asset.path).ok()
    }

    /// Calculate asset hash for cache busting
    pub fn calculate_hash(&self, handle: &str) -> Option<String> {
        let contents = self.get_contents(handle)?;
        Some(blake3::hash(&contents).to_hex()[..8].to_string())
    }
}

/// Inline content position
#[derive(Debug, Clone, Copy)]
pub enum InlinePosition {
    Before,
    After,
}

/// Asset bundler for combining assets
pub struct AssetBundler {
    bundles: HashMap<String, AssetBundle>,
}

impl AssetBundler {
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }

    /// Create a new bundle
    pub fn create_bundle(&mut self, name: &str, assets: Vec<String>, output_path: PathBuf) {
        self.bundles.insert(
            name.to_string(),
            AssetBundle {
                name: name.to_string(),
                assets,
                output_path,
                minified: false,
            },
        );
    }

    /// Build a bundle
    pub fn build(&self, name: &str, asset_manager: &AssetManager) -> Result<PathBuf, BundleError> {
        let bundle = self
            .bundles
            .get(name)
            .ok_or_else(|| BundleError::NotFound(name.to_string()))?;

        let mut combined = String::new();

        for handle in &bundle.assets {
            if let Some(contents) = asset_manager.get_contents(handle) {
                let content_str = String::from_utf8_lossy(&contents);
                combined.push_str(&content_str);
                combined.push('\n');
            } else {
                warn!("Asset not found for bundling: {}", handle);
            }
        }

        // Write to output
        std::fs::write(&bundle.output_path, combined)
            .map_err(|e| BundleError::WriteError(e.to_string()))?;

        info!("Built bundle: {} -> {:?}", name, bundle.output_path);
        Ok(bundle.output_path.clone())
    }
}

impl Default for AssetBundler {
    fn default() -> Self {
        Self::new()
    }
}

/// Bundle error
#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("Bundle not found: {0}")]
    NotFound(String),

    #[error("Write error: {0}")]
    WriteError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("css"), AssetType::Css);
        assert_eq!(AssetType::from_extension("js"), AssetType::JavaScript);
        assert_eq!(AssetType::from_extension("png"), AssetType::Image);
        assert_eq!(AssetType::from_extension("woff2"), AssetType::Font);
        assert_eq!(AssetType::from_extension("unknown"), AssetType::Other);
    }

    #[test]
    fn test_asset_manager() {
        let manager = AssetManager::new(PathBuf::from("/tmp"), "/assets");

        manager.register_style("test-plugin", "test-css", "style.css", vec![]);
        manager.register_script("test-plugin", "test-js", "script.js", vec![], true);

        let assets = manager.get_plugin_assets("test-plugin");
        assert_eq!(assets.len(), 2);
    }

    #[test]
    fn test_asset_enqueue() {
        let manager = AssetManager::new(PathBuf::from("/tmp"), "/assets");

        manager.register_style("test-plugin", "test-css", "style.css", vec![]);
        manager.enqueue("test-css");

        let header = manager.get_enqueued(AssetLocation::Header, false);
        assert_eq!(header.len(), 1);
        assert!(header[0].is_enqueued);
    }
}
