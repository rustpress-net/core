//! Theme Service & Installer Manager
//!
//! Service layer that combines the file-based ThemeManager with database persistence.
//! This ensures themes are properly registered in the database while maintaining
//! file system theme resources (templates, assets).
//!
//! Features:
//! - Theme installation from ZIP files
//! - Theme activation/deactivation
//! - Theme settings management
//! - Menu/widget assignments per theme
//! - Theme preview sessions

use chrono::{DateTime, Duration, Utc};
use rustpress_core::error::{Error, Result};
use rustpress_database::repository::themes::{ThemeRepository, ThemeRow};
use rustpress_themes::manager::{ThemeManager, RegisteredTheme};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use zip::ZipArchive;

/// Theme scan result
#[derive(Debug, Serialize)]
pub struct ThemeScanResult {
    pub scanned: usize,
    pub registered: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}

/// Theme preview result
#[derive(Debug, Serialize)]
pub struct ThemePreviewResult {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Theme installation result
#[derive(Debug, Serialize)]
pub struct ThemeInstallResult {
    pub success: bool,
    pub theme_id: String,
    pub theme_name: String,
    pub message: String,
    pub warnings: Vec<String>,
}

/// Theme validation result
#[derive(Debug, Serialize)]
pub struct ThemeValidationResult {
    pub valid: bool,
    pub theme_id: Option<String>,
    pub theme_name: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Theme info combining file system and database data
#[derive(Debug, Clone, Serialize)]
pub struct ThemeInfo {
    pub id: Uuid,
    pub theme_id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub author_url: Option<String>,
    pub license: Option<String>,
    pub is_active: bool,
    pub is_installed: bool,
    pub parent_theme_id: Option<String>,
    pub screenshot_url: Option<String>,
    pub tags: Vec<String>,
    pub supports: serde_json::Value,
    pub menu_locations: serde_json::Value,
    pub widget_areas: serde_json::Value,
    pub template_count: i32,
    pub activated_at: Option<DateTime<Utc>>,
    pub installed_at: Option<DateTime<Utc>>,
}

impl From<ThemeRow> for ThemeInfo {
    fn from(row: ThemeRow) -> Self {
        Self {
            id: row.id,
            theme_id: row.theme_id,
            name: row.name,
            description: row.description,
            version: row.version,
            author: row.author,
            author_url: row.author_url,
            license: row.license,
            is_active: row.is_active,
            is_installed: row.is_installed,
            parent_theme_id: row.parent_theme_id,
            screenshot_url: row.screenshot_url,
            tags: row.tags.unwrap_or_default(),
            supports: row.supports,
            menu_locations: row.menu_locations,
            widget_areas: row.widget_areas,
            template_count: row.template_count.unwrap_or(0),
            activated_at: row.activated_at,
            installed_at: row.installed_at,
        }
    }
}

/// Theme settings response
#[derive(Debug, Serialize)]
pub struct ThemeSettingsInfo {
    pub theme_id: String,
    pub customizer_schema: serde_json::Value,
    pub settings: serde_json::Value,
}

/// Theme service combining file system and database operations
pub struct ThemeService {
    /// File-based theme manager
    file_manager: Arc<ThemeManager>,
    /// Database pool
    pool: PgPool,
    /// Current site ID (for multi-site)
    site_id: Option<Uuid>,
    /// Themes directory path
    themes_dir: PathBuf,
}

impl ThemeService {
    /// Create a new theme service
    pub fn new(pool: PgPool, themes_dir: PathBuf, site_id: Option<Uuid>) -> Self {
        let file_manager = Arc::new(ThemeManager::new(themes_dir.clone()));

        Self {
            file_manager,
            pool,
            site_id,
            themes_dir,
        }
    }

    /// Get the themes directory path
    pub fn themes_dir(&self) -> &Path {
        &self.themes_dir
    }

    /// Get the theme repository with site context
    fn repo(&self) -> ThemeRepository {
        let repo = ThemeRepository::new(self.pool.clone());
        if let Some(site_id) = self.site_id {
            repo.with_site(site_id)
        } else {
            repo
        }
    }

    /// Scan themes directory and sync with database
    pub async fn scan_themes(&self) -> Result<ThemeScanResult> {
        let mut result = ThemeScanResult {
            scanned: 0,
            registered: 0,
            updated: 0,
            errors: vec![],
        };

        // Scan file system for themes
        let scanned_ids = self.file_manager.scan_themes().await
            .map_err(|e| Error::internal(format!("Failed to scan themes: {}", e)))?;

        result.scanned = scanned_ids.len();

        // Sync each theme to database
        for theme_id in scanned_ids {
            match self.sync_theme_to_db(&theme_id).await {
                Ok(is_new) => {
                    if is_new {
                        result.registered += 1;
                    } else {
                        result.updated += 1;
                    }
                }
                Err(e) => {
                    result.errors.push(format!("{}: {}", theme_id, e));
                }
            }
        }

        Ok(result)
    }

    /// Sync a single theme from file system to database
    async fn sync_theme_to_db(&self, theme_id: &str) -> Result<bool> {
        let registered = self.file_manager.get_theme(theme_id)
            .ok_or_else(|| Error::not_found("Theme", theme_id))?;

        let existing = self.repo().find_by_theme_id(theme_id).await?;
        let is_new = existing.is_none();

        // Read theme.json for additional metadata
        let theme_json_path = self.themes_dir.join(theme_id).join("theme.json");
        let extra_meta: Option<serde_json::Value> = if theme_json_path.exists() {
            let content = tokio::fs::read_to_string(&theme_json_path).await.ok();
            content.and_then(|c| serde_json::from_str(&c).ok())
        } else {
            None
        };

        // Count templates
        let templates_dir = self.themes_dir.join(theme_id).join("templates");
        let template_count = if templates_dir.exists() {
            count_templates(&templates_dir).await.unwrap_or(0)
        } else {
            0
        };

        // Build theme row
        let theme_row = ThemeRow {
            id: existing.as_ref().map(|e| e.id).unwrap_or_else(Uuid::now_v7),
            site_id: self.site_id,
            theme_id: theme_id.to_string(),
            name: registered.manifest.theme.name.clone(),
            description: Some(registered.manifest.theme.description.clone()),
            version: Some(registered.manifest.theme.version.clone()),
            author: Some(registered.manifest.theme.author.clone()),
            author_url: registered.manifest.theme.author_url.clone(),
            license: Some(registered.manifest.theme.license.clone()),
            is_active: existing.as_ref().map(|e| e.is_active).unwrap_or(false),
            is_installed: true,
            parent_theme_id: registered.parent_id.clone(),
            screenshot_url: registered.screenshot.as_ref().map(|p| {
                format!("/themes/{}/screenshot.png", theme_id)
            }),
            homepage_url: None,
            tags: extra_meta.as_ref()
                .and_then(|m| m.get("tags"))
                .and_then(|t| serde_json::from_value(t.clone()).ok()),
            supports: extra_meta.as_ref()
                .and_then(|m| m.get("supports"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({
                    "post_thumbnails": true,
                    "custom_logo": true,
                    "menus": true,
                    "widgets": true
                })),
            menu_locations: extra_meta.as_ref()
                .and_then(|m| m.get("menu_locations"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({
                    "primary": "Primary Navigation",
                    "footer": "Footer Navigation"
                })),
            widget_areas: extra_meta.as_ref()
                .and_then(|m| m.get("widget_areas"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({
                    "sidebar": "Main Sidebar",
                    "footer-1": "Footer Column 1",
                    "footer-2": "Footer Column 2"
                })),
            customizer_schema: extra_meta.as_ref()
                .and_then(|m| m.get("customizer"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            settings: existing.as_ref()
                .map(|e| e.settings.clone())
                .unwrap_or_else(|| serde_json::json!({})),
            template_count: Some(template_count),
            activated_at: existing.as_ref().and_then(|e| e.activated_at),
            installed_at: existing.as_ref().and_then(|e| e.installed_at),
            created_at: existing.as_ref().map(|e| e.created_at).unwrap_or_else(Utc::now),
            updated_at: Utc::now(),
        };

        self.repo().install(&theme_row).await?;

        Ok(is_new)
    }

    /// List all themes from database
    pub async fn list_themes(&self) -> Result<Vec<ThemeInfo>> {
        let rows = self.repo().list().await?;
        Ok(rows.into_iter().map(ThemeInfo::from).collect())
    }

    /// Get a specific theme
    pub async fn get_theme(&self, theme_id: &str) -> Result<Option<ThemeInfo>> {
        let row = self.repo().find_by_theme_id(theme_id).await?;
        Ok(row.map(ThemeInfo::from))
    }

    /// Get the active theme
    pub async fn get_active_theme(&self) -> Result<Option<ThemeInfo>> {
        let row = self.repo().get_active().await?;
        Ok(row.map(ThemeInfo::from))
    }

    /// Get active theme ID
    pub async fn get_active_theme_id(&self) -> Result<Option<String>> {
        let row = self.repo().get_active().await?;
        Ok(row.map(|r| r.theme_id))
    }

    /// Activate a theme
    pub async fn activate_theme(&self, theme_id: &str) -> Result<ThemeInfo> {
        // Verify theme exists in file system
        if self.file_manager.get_theme(theme_id).is_none() {
            return Err(Error::not_found("Theme", theme_id));
        }

        // Activate in file system
        self.file_manager.activate(theme_id).await
            .map_err(|e| Error::internal(format!("Failed to activate theme: {}", e)))?;

        // Activate in database (trigger will deactivate others)
        let row = self.repo().activate(theme_id).await?;

        Ok(ThemeInfo::from(row))
    }

    /// Delete a theme
    pub async fn delete_theme(&self, theme_id: &str) -> Result<()> {
        // Delete from file system
        self.file_manager.delete_theme(theme_id).await
            .map_err(|e| Error::internal(format!("Failed to delete theme: {}", e)))?;

        // Delete from database
        self.repo().delete(theme_id).await?;

        Ok(())
    }

    /// Get theme settings
    pub async fn get_theme_settings(&self, theme_id: &str) -> Result<ThemeSettingsInfo> {
        let row = self.repo().find_by_theme_id(theme_id).await?
            .ok_or_else(|| Error::not_found("Theme", theme_id))?;

        Ok(ThemeSettingsInfo {
            theme_id: row.theme_id,
            customizer_schema: row.customizer_schema,
            settings: row.settings,
        })
    }

    /// Update theme settings
    pub async fn update_theme_settings(&self, theme_id: &str, settings: serde_json::Value) -> Result<serde_json::Value> {
        let row = self.repo().update_settings(theme_id, settings).await?;
        Ok(row.settings)
    }

    /// Get menu assignments for a theme
    pub async fn get_menu_assignments(&self, theme_id: &str) -> Result<serde_json::Value> {
        let assignments = self.repo().get_menu_assignments(theme_id).await?;

        let result: Vec<serde_json::Value> = assignments.into_iter().map(|a| {
            serde_json::json!({
                "location_slug": a.location_slug,
                "menu_id": a.menu_id
            })
        }).collect();

        Ok(serde_json::json!(result))
    }

    /// Update menu assignments
    pub async fn update_menu_assignments(&self, theme_id: &str, assignments: serde_json::Value) -> Result<()> {
        if let Some(arr) = assignments.as_array() {
            for item in arr {
                if let (Some(location), Some(menu_id)) = (
                    item.get("location_slug").and_then(|v| v.as_str()),
                    item.get("menu_id").and_then(|v| v.as_str())
                ) {
                    let menu_uuid = Uuid::parse_str(menu_id)
                        .map_err(|_| Error::validation("Invalid menu_id UUID"))?;
                    self.repo().assign_menu(theme_id, location, menu_uuid).await?;
                }
            }
        }
        Ok(())
    }

    /// Get widget assignments for a theme
    pub async fn get_widget_assignments(&self, theme_id: &str) -> Result<serde_json::Value> {
        let assignments = self.repo().get_widget_assignments(theme_id, None).await?;

        let result: Vec<serde_json::Value> = assignments.into_iter().map(|a| {
            serde_json::json!({
                "area_slug": a.area_slug,
                "widget_id": a.widget_id,
                "position": a.position,
                "is_active": a.is_active
            })
        }).collect();

        Ok(serde_json::json!(result))
    }

    /// Update widget assignments
    pub async fn update_widget_assignments(&self, theme_id: &str, assignments: serde_json::Value) -> Result<()> {
        if let Some(arr) = assignments.as_array() {
            for item in arr {
                if let (Some(area), Some(widget_id), Some(position)) = (
                    item.get("area_slug").and_then(|v| v.as_str()),
                    item.get("widget_id").and_then(|v| v.as_str()),
                    item.get("position").and_then(|v| v.as_i64())
                ) {
                    let widget_uuid = Uuid::parse_str(widget_id)
                        .map_err(|_| Error::validation("Invalid widget_id UUID"))?;
                    self.repo().assign_widget(theme_id, area, widget_uuid, position as i32).await?;
                }
            }
        }
        Ok(())
    }

    /// Create a preview session
    pub async fn create_preview(&self, user_id: Uuid, theme_id: &str, _settings: serde_json::Value) -> Result<ThemePreviewResult> {
        // Generate token
        let token = format!("preview_{}", Uuid::new_v4());
        let expires_at = Utc::now() + Duration::minutes(30);

        // Store in database
        self.repo().create_preview(user_id, theme_id, &token, expires_at).await?;

        Ok(ThemePreviewResult { token, expires_at })
    }

    /// Get the file-based theme manager for template rendering
    pub fn file_manager(&self) -> &Arc<ThemeManager> {
        &self.file_manager
    }

    // ============================================================================
    // Theme Installer Manager
    // ============================================================================

    /// Install a theme from a ZIP file
    pub async fn install_from_zip(&self, zip_data: &[u8], activate_after: bool) -> Result<ThemeInstallResult> {
        let mut warnings = Vec::new();

        // Validate the ZIP first
        let validation = self.validate_zip(zip_data)?;
        if !validation.valid {
            return Err(Error::validation(format!(
                "Invalid theme package: {}",
                validation.errors.join(", ")
            )));
        }

        warnings.extend(validation.warnings);

        let theme_id = validation.theme_id.unwrap();
        let theme_name = validation.theme_name.unwrap_or_else(|| theme_id.clone());

        // Check if theme already exists
        let existing = self.repo().find_by_theme_id(&theme_id).await?;
        if existing.is_some() {
            return Err(Error::validation(format!(
                "Theme '{}' is already installed. Please uninstall it first or use update.",
                theme_id
            )));
        }

        // Extract to themes directory
        let theme_path = self.themes_dir.join(&theme_id);
        self.extract_zip(zip_data, &theme_path)?;

        // Sync to database
        self.sync_theme_to_db(&theme_id).await?;

        // Activate if requested
        if activate_after {
            self.activate_theme(&theme_id).await?;
        }

        Ok(ThemeInstallResult {
            success: true,
            theme_id: theme_id.clone(),
            theme_name,
            message: format!(
                "Theme installed successfully{}",
                if activate_after { " and activated" } else { "" }
            ),
            warnings,
        })
    }

    /// Update an existing theme from a ZIP file
    pub async fn update_from_zip(&self, zip_data: &[u8]) -> Result<ThemeInstallResult> {
        let mut warnings = Vec::new();

        // Validate the ZIP
        let validation = self.validate_zip(zip_data)?;
        if !validation.valid {
            return Err(Error::validation(format!(
                "Invalid theme package: {}",
                validation.errors.join(", ")
            )));
        }

        warnings.extend(validation.warnings);

        let theme_id = validation.theme_id.unwrap();
        let theme_name = validation.theme_name.unwrap_or_else(|| theme_id.clone());

        // Check if theme exists
        let existing = self.repo().find_by_theme_id(&theme_id).await?;
        if existing.is_none() {
            return Err(Error::not_found("Theme", &theme_id));
        }

        let was_active = existing.as_ref().map(|e| e.is_active).unwrap_or(false);

        // Backup current settings
        let current_settings = existing.map(|e| e.settings).unwrap_or_default();

        // Remove old files and extract new
        let theme_path = self.themes_dir.join(&theme_id);
        if theme_path.exists() {
            tokio::fs::remove_dir_all(&theme_path).await
                .map_err(|e| Error::internal(format!("Failed to remove old theme: {}", e)))?;
        }

        self.extract_zip(zip_data, &theme_path)?;

        // Sync to database (preserving settings)
        self.sync_theme_to_db(&theme_id).await?;

        // Restore settings
        self.repo().update_settings(&theme_id, current_settings).await?;

        // Re-activate if it was active
        if was_active {
            self.activate_theme(&theme_id).await?;
        }

        Ok(ThemeInstallResult {
            success: true,
            theme_id: theme_id.clone(),
            theme_name,
            message: "Theme updated successfully".to_string(),
            warnings,
        })
    }

    /// Validate a theme ZIP file without installing
    pub fn validate_zip(&self, zip_data: &[u8]) -> Result<ThemeValidationResult> {
        use std::io::Cursor;

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut theme_id = None;
        let mut theme_name = None;

        // Try to open as ZIP
        let cursor = Cursor::new(zip_data);
        let mut archive = match ZipArchive::new(cursor) {
            Ok(a) => a,
            Err(e) => {
                return Ok(ThemeValidationResult {
                    valid: false,
                    theme_id: None,
                    theme_name: None,
                    errors: vec![format!("Invalid ZIP file: {}", e)],
                    warnings: vec![],
                });
            }
        };

        // Find theme.json or theme.toml
        let mut has_manifest = false;
        let mut root_folder: Option<String> = None;

        for i in 0..archive.len() {
            // First pass: get name and detect root folder
            let (name, is_manifest) = {
                let file = archive.by_index(i).map_err(|e| Error::internal(e.to_string()))?;
                let name = file.name().to_string();
                let is_manifest = name.ends_with("theme.json") || name.ends_with("theme.toml");

                // Detect root folder (themes are often zipped with a root folder)
                if root_folder.is_none() && name.contains('/') {
                    let parts: Vec<&str> = name.split('/').collect();
                    if parts.len() > 1 && !parts[0].is_empty() {
                        root_folder = Some(parts[0].to_string());
                    }
                }

                (name, is_manifest)
            };

            // Second pass: read manifest content if needed
            if is_manifest {
                has_manifest = true;

                let mut file = archive.by_index(i).map_err(|e| Error::internal(e.to_string()))?;
                let mut content = String::new();
                file.read_to_string(&mut content).ok();

                if name.ends_with("theme.json") {
                    if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                        theme_name = manifest.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        // Theme ID is typically the folder name
                        if let Some(ref folder) = root_folder {
                            theme_id = Some(folder.clone());
                        }
                    }
                } else if name.ends_with("theme.toml") {
                    if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                        theme_name = manifest.get("theme")
                            .and_then(|t| t.get("name"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        theme_id = manifest.get("theme")
                            .and_then(|t| t.get("id"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
            }
        }

        // Use root folder as theme_id if not found in manifest
        if theme_id.is_none() {
            if let Some(ref folder) = root_folder {
                theme_id = Some(folder.clone());
            }
        }

        // Validation checks
        if !has_manifest {
            errors.push("Missing theme.json or theme.toml manifest file".to_string());
        }

        if theme_id.is_none() {
            errors.push("Could not determine theme ID from package".to_string());
        }

        // Check for required directories
        let mut has_templates = false;
        let mut has_assets = false;

        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| Error::internal(e.to_string()))?;
            let name = file.name();

            if name.contains("templates/") {
                has_templates = true;
            }
            if name.contains("assets/") {
                has_assets = true;
            }
        }

        if !has_templates {
            warnings.push("Theme does not contain a templates/ directory".to_string());
        }
        if !has_assets {
            warnings.push("Theme does not contain an assets/ directory".to_string());
        }

        // Validate theme_id format
        if let Some(ref id) = theme_id {
            if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                errors.push(format!(
                    "Invalid theme ID '{}': must contain only letters, numbers, hyphens, and underscores",
                    id
                ));
            }
        }

        Ok(ThemeValidationResult {
            valid: errors.is_empty(),
            theme_id,
            theme_name,
            errors,
            warnings,
        })
    }

    /// Extract a ZIP file to a directory
    fn extract_zip(&self, zip_data: &[u8], dest: &Path) -> Result<()> {
        use std::io::Cursor;

        let cursor = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| Error::internal(format!("Failed to open ZIP: {}", e)))?;

        // Detect if there's a root folder wrapper
        let root_folder = self.detect_zip_root_folder(&mut archive);

        // Create destination directory
        std::fs::create_dir_all(dest)
            .map_err(|e| Error::internal(format!("Failed to create theme directory: {}", e)))?;

        // Extract files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| Error::internal(format!("Failed to read ZIP entry: {}", e)))?;

            let mut outpath = dest.to_path_buf();

            // Handle path stripping if there's a root folder
            let name = file.name();
            let relative_path = if let Some(ref root) = root_folder {
                if name.starts_with(root) {
                    name.strip_prefix(&format!("{}/", root)).unwrap_or(name)
                } else {
                    name
                }
            } else {
                name
            };

            // Skip empty paths
            if relative_path.is_empty() {
                continue;
            }

            outpath.push(relative_path);

            // Security: prevent path traversal
            if !outpath.starts_with(dest) {
                return Err(Error::validation("ZIP contains invalid path (possible path traversal)"));
            }

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)
                    .map_err(|e| Error::internal(format!("Failed to create directory: {}", e)))?;
            } else {
                // Create parent directories
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| Error::internal(format!("Failed to create parent directory: {}", e)))?;
                }

                // Extract file
                let mut outfile = std::fs::File::create(&outpath)
                    .map_err(|e| Error::internal(format!("Failed to create file: {}", e)))?;

                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| Error::internal(format!("Failed to write file: {}", e)))?;
            }

            // Set permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
                }
            }
        }

        Ok(())
    }

    /// Detect if ZIP has a single root folder
    fn detect_zip_root_folder<R: Read + std::io::Seek>(&self, archive: &mut ZipArchive<R>) -> Option<String> {
        if archive.is_empty() {
            return None;
        }

        // Get first entry info and drop borrow immediately
        let first_component = {
            let first_entry = archive.by_index(0).ok()?;
            let first_name = first_entry.name();

            // Check if it's a directory entry at root level
            if first_name.ends_with('/') && !first_name.contains('/') {
                return None; // No nested root, it's just a file at root
            }

            // Get first path component
            first_name.split('/').next()?.to_string()
        };

        // Verify all entries start with this component
        let archive_len = archive.len();
        for i in 0..archive_len {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if !name.starts_with(&first_component) {
                    return None;
                }
            }
        }

        Some(first_component)
    }

    /// Export a theme as a ZIP file
    pub async fn export_theme(&self, theme_id: &str) -> Result<Vec<u8>> {
        use std::io::Cursor;
        use zip::write::SimpleFileOptions;

        let theme_path = self.themes_dir.join(theme_id);
        if !theme_path.exists() {
            return Err(Error::not_found("Theme", theme_id));
        }

        let mut zip_buffer = Cursor::new(Vec::new());

        {
            let mut zip = zip::ZipWriter::new(&mut zip_buffer);
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            // Add all files from theme directory
            self.add_dir_to_zip(&mut zip, &theme_path, theme_id, options)?;

            zip.finish().map_err(|e| Error::internal(format!("Failed to finalize ZIP: {}", e)))?;
        }

        Ok(zip_buffer.into_inner())
    }

    /// Recursively add directory contents to ZIP
    fn add_dir_to_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut zip::ZipWriter<W>,
        dir: &Path,
        prefix: &str,
        options: zip::write::SimpleFileOptions,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| Error::internal(e.to_string()))?;
            let path = entry.path();
            let name = format!("{}/{}", prefix, entry.file_name().to_string_lossy());

            if path.is_dir() {
                zip.add_directory(&name, options)
                    .map_err(|e| Error::internal(format!("Failed to add directory to ZIP: {}", e)))?;
                self.add_dir_to_zip(zip, &path, &name, options)?;
            } else {
                zip.start_file(&name, options)
                    .map_err(|e| Error::internal(format!("Failed to start file in ZIP: {}", e)))?;

                let mut file = std::fs::File::open(&path)
                    .map_err(|e| Error::internal(format!("Failed to open file: {}", e)))?;

                std::io::copy(&mut file, zip)
                    .map_err(|e| Error::internal(format!("Failed to write to ZIP: {}", e)))?;
            }
        }

        Ok(())
    }

    /// Get list of available default themes that can be installed
    pub fn get_available_themes(&self) -> Vec<DefaultThemeInfo> {
        vec![
            DefaultThemeInfo {
                id: "starter-starter".to_string(),
                name: "Starter".to_string(),
                description: "A minimal, clean starter theme for blogs".to_string(),
                version: "1.0.0".to_string(),
                screenshot_url: "/themes/starter-starter/screenshot.png".to_string(),
                category: "Blog".to_string(),
            },
            DefaultThemeInfo {
                id: "business-elite".to_string(),
                name: "Business Elite".to_string(),
                description: "Professional corporate theme for businesses".to_string(),
                version: "1.0.0".to_string(),
                screenshot_url: "/themes/business-elite/screenshot.png".to_string(),
                category: "Business".to_string(),
            },
            DefaultThemeInfo {
                id: "portfolio-creative".to_string(),
                name: "Portfolio Creative".to_string(),
                description: "Creative portfolio theme with animations".to_string(),
                version: "1.0.0".to_string(),
                screenshot_url: "/themes/portfolio-creative/screenshot.png".to_string(),
                category: "Portfolio".to_string(),
            },
            DefaultThemeInfo {
                id: "developer-developer".to_string(),
                name: "Developer".to_string(),
                description: "Tech-focused magazine theme for developers".to_string(),
                version: "1.0.0".to_string(),
                screenshot_url: "/themes/developer-developer/screenshot.png".to_string(),
                category: "Magazine".to_string(),
            },
        ]
    }
}

/// Info about a default/bundled theme
#[derive(Debug, Serialize)]
pub struct DefaultThemeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub screenshot_url: String,
    pub category: String,
}

/// Count template files in a directory
async fn count_templates(dir: &std::path::Path) -> Result<i32> {
    let mut count = 0;
    let mut entries = tokio::fs::read_dir(dir).await
        .map_err(|e| Error::internal(format!("Failed to read directory: {}", e)))?;

    while let Some(entry) = entries.next_entry().await
        .map_err(|e| Error::internal(format!("Failed to read entry: {}", e)))? {

        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "html" || ext == "tera" {
                    count += 1;
                }
            }
        } else if path.is_dir() {
            count += Box::pin(count_templates(&path)).await?;
        }
    }

    Ok(count)
}
