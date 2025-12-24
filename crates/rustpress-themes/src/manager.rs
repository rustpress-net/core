//! Theme Manager and Switching
//!
//! Central theme management, activation, and switching.

use crate::child_theme::ThemeInheritance;
use crate::manifest::ThemeManifest;
use crate::settings::ThemeSettings;
use crate::templates::{TemplateEngine, TemplateError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;
use tokio::sync::broadcast;

/// Theme manager errors
#[derive(Debug, Error)]
pub enum ThemeManagerError {
    #[error("Theme not found: {0}")]
    NotFound(String),

    #[error("Theme already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid theme: {0}")]
    Invalid(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    Manifest(String),

    #[error("Template error: {0}")]
    Template(#[from] TemplateError),

    #[error("Theme activation failed: {0}")]
    ActivationFailed(String),

    #[error("Theme deactivation failed: {0}")]
    DeactivationFailed(String),
}

/// Theme status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeStatus {
    Active,
    Inactive,
    Broken,
    UpdateAvailable,
}

/// Registered theme information
#[derive(Debug, Clone)]
pub struct RegisteredTheme {
    pub id: String,
    pub manifest: ThemeManifest,
    pub path: PathBuf,
    pub status: ThemeStatus,
    pub parent_id: Option<String>,
    pub screenshot: Option<PathBuf>,
}

/// Theme switch event
#[derive(Debug, Clone)]
pub struct ThemeSwitchEvent {
    pub old_theme: Option<String>,
    pub new_theme: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Central theme manager
pub struct ThemeManager {
    /// Themes directory
    themes_dir: PathBuf,
    /// Registered themes
    themes: Arc<RwLock<HashMap<String, RegisteredTheme>>>,
    /// Currently active theme
    active_theme: Arc<RwLock<Option<String>>>,
    /// Theme inheritance handler
    inheritance: Arc<ThemeInheritance>,
    /// Theme settings by ID
    settings: Arc<RwLock<HashMap<String, Arc<ThemeSettings>>>>,
    /// Template engines by theme ID
    engines: Arc<RwLock<HashMap<String, Arc<TemplateEngine>>>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<ThemeEvent>,
}

/// Theme events
#[derive(Debug, Clone)]
pub enum ThemeEvent {
    Registered(String),
    Unregistered(String),
    Activated(String),
    Deactivated(String),
    Updated(String),
    SettingsChanged { theme_id: String, key: String },
}

impl ThemeManager {
    pub fn new(themes_dir: PathBuf) -> Self {
        let (event_tx, _) = broadcast::channel(100);

        Self {
            themes_dir: themes_dir.clone(),
            themes: Arc::new(RwLock::new(HashMap::new())),
            active_theme: Arc::new(RwLock::new(None)),
            inheritance: Arc::new(ThemeInheritance::new(themes_dir)),
            settings: Arc::new(RwLock::new(HashMap::new())),
            engines: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Scan and register all themes in the themes directory
    pub async fn scan_themes(&self) -> Result<Vec<String>, ThemeManagerError> {
        let mut registered = Vec::new();

        if !self.themes_dir.exists() {
            fs::create_dir_all(&self.themes_dir).await?;
            return Ok(registered);
        }

        let mut entries = fs::read_dir(&self.themes_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let theme_id = entry.file_name().to_string_lossy().to_string();
                // Check for theme.toml (preferred) or theme.json (fallback)
                let manifest_path_toml = entry.path().join("theme.toml");
                let manifest_path_json = entry.path().join("theme.json");

                if manifest_path_toml.exists() || manifest_path_json.exists() {
                    match self.register_theme(&theme_id).await {
                        Ok(_) => registered.push(theme_id),
                        Err(e) => {
                            tracing::warn!("Failed to register theme {}: {}", theme_id, e);
                        }
                    }
                }
            }
        }

        Ok(registered)
    }

    /// Register a single theme
    pub async fn register_theme(&self, theme_id: &str) -> Result<(), ThemeManagerError> {
        let theme_path = self.themes_dir.join(theme_id);
        let manifest_path_toml = theme_path.join("theme.toml");
        let manifest_path_json = theme_path.join("theme.json");

        // Determine which manifest format to use
        let (manifest_path, is_json) = if manifest_path_toml.exists() {
            (manifest_path_toml, false)
        } else if manifest_path_json.exists() {
            (manifest_path_json, true)
        } else {
            return Err(ThemeManagerError::NotFound(format!(
                "Manifest not found for theme {} (tried theme.toml and theme.json)",
                theme_id
            )));
        };

        // Parse manifest based on format
        let manifest_content = fs::read_to_string(&manifest_path).await?;
        let manifest: ThemeManifest = if is_json {
            ThemeManifest::from_json(&manifest_content, theme_id)
                .map_err(|e| ThemeManagerError::Manifest(format!("JSON parse error: {}", e)))?
        } else {
            toml::from_str(&manifest_content)
                .map_err(|e| ThemeManagerError::Manifest(e.to_string()))?
        };

        // Check for screenshot
        let screenshot = self.find_screenshot(&theme_path).await;

        // Determine parent
        let parent_id = manifest.parent.as_ref().map(|p| p.id.clone());

        // Validate theme
        self.validate_theme(&theme_path, &manifest).await?;

        let theme = RegisteredTheme {
            id: theme_id.to_string(),
            manifest,
            path: theme_path,
            status: ThemeStatus::Inactive,
            parent_id,
            screenshot,
        };

        self.themes.write().insert(theme_id.to_string(), theme);

        // Broadcast event
        let _ = self.event_tx.send(ThemeEvent::Registered(theme_id.to_string()));

        Ok(())
    }

    async fn find_screenshot(&self, theme_path: &Path) -> Option<PathBuf> {
        let extensions = ["png", "jpg", "jpeg", "webp"];

        for ext in &extensions {
            let path = theme_path.join(format!("screenshot.{}", ext));
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    async fn validate_theme(&self, _path: &Path, manifest: &ThemeManifest) -> Result<(), ThemeManagerError> {
        // Validate required fields
        if manifest.theme.id.is_empty() {
            return Err(ThemeManagerError::Invalid("Missing theme ID".to_string()));
        }

        if manifest.theme.name.is_empty() {
            return Err(ThemeManagerError::Invalid("Missing theme name".to_string()));
        }

        // Validate parent if specified
        if let Some(parent) = &manifest.parent {
            let parent_path = self.themes_dir.join(&parent.id);
            if !parent_path.exists() {
                return Err(ThemeManagerError::Invalid(format!(
                    "Parent theme not found: {}",
                    parent.id
                )));
            }
        }

        Ok(())
    }

    /// Unregister a theme
    pub fn unregister_theme(&self, theme_id: &str) -> Result<(), ThemeManagerError> {
        // Check if active
        if self.active_theme.read().as_ref() == Some(&theme_id.to_string()) {
            return Err(ThemeManagerError::DeactivationFailed(
                "Cannot unregister active theme".to_string(),
            ));
        }

        self.themes.write().remove(theme_id);
        self.settings.write().remove(theme_id);
        self.engines.write().remove(theme_id);

        let _ = self.event_tx.send(ThemeEvent::Unregistered(theme_id.to_string()));

        Ok(())
    }

    /// Activate a theme
    pub async fn activate(&self, theme_id: &str) -> Result<(), ThemeManagerError> {
        // Get theme
        let theme = self.themes.read().get(theme_id).cloned()
            .ok_or_else(|| ThemeManagerError::NotFound(theme_id.to_string()))?;

        // Deactivate current theme if any
        let old_theme = self.active_theme.read().clone();
        if let Some(ref old_id) = old_theme {
            self.deactivate_internal(old_id)?;
        }

        // Load parent chain via inheritance
        self.inheritance.load_theme(theme_id).await
            .map_err(|e| ThemeManagerError::ActivationFailed(e.to_string()))?;

        // Initialize theme settings
        let settings_path = self.themes_dir.join(".settings");
        let theme_settings = Arc::new(ThemeSettings::new(theme_id, settings_path));
        theme_settings.load().await.map_err(|e| {
            ThemeManagerError::ActivationFailed(format!("Failed to load settings: {}", e))
        })?;

        self.settings.write().insert(theme_id.to_string(), theme_settings);

        // Initialize template engine
        let extension = theme.manifest.templates.extension.as_str();
        let engine = Arc::new(TemplateEngine::new(theme.path.clone(), extension)?);
        self.engines.write().insert(theme_id.to_string(), engine);

        // Update status
        {
            let mut themes = self.themes.write();
            if let Some(t) = themes.get_mut(theme_id) {
                t.status = ThemeStatus::Active;
            }
        }

        // Set as active
        *self.active_theme.write() = Some(theme_id.to_string());

        // Broadcast event
        let _ = self.event_tx.send(ThemeEvent::Activated(theme_id.to_string()));

        tracing::info!("Activated theme: {}", theme_id);

        Ok(())
    }

    fn deactivate_internal(&self, theme_id: &str) -> Result<(), ThemeManagerError> {
        // Update status
        {
            let mut themes = self.themes.write();
            if let Some(theme) = themes.get_mut(theme_id) {
                theme.status = ThemeStatus::Inactive;
            }
        }

        // Broadcast event
        let _ = self.event_tx.send(ThemeEvent::Deactivated(theme_id.to_string()));

        Ok(())
    }

    /// Switch to a different theme
    pub async fn switch_theme(&self, new_theme_id: &str) -> Result<ThemeSwitchEvent, ThemeManagerError> {
        let old_theme = self.active_theme.read().clone();

        // Activate new theme
        self.activate(new_theme_id).await?;

        let event = ThemeSwitchEvent {
            old_theme,
            new_theme: new_theme_id.to_string(),
            timestamp: chrono::Utc::now(),
        };

        Ok(event)
    }

    /// Get the active theme
    pub fn get_active(&self) -> Option<RegisteredTheme> {
        let active_id = self.active_theme.read().clone()?;
        self.themes.read().get(&active_id).cloned()
    }

    /// Get the active theme ID
    pub fn get_active_id(&self) -> Option<String> {
        self.active_theme.read().clone()
    }

    /// Get a theme by ID
    pub fn get_theme(&self, theme_id: &str) -> Option<RegisteredTheme> {
        self.themes.read().get(theme_id).cloned()
    }

    /// Get all registered themes
    pub fn get_all_themes(&self) -> Vec<RegisteredTheme> {
        self.themes.read().values().cloned().collect()
    }

    /// Get theme settings
    pub fn get_settings(&self, theme_id: &str) -> Option<Arc<ThemeSettings>> {
        self.settings.read().get(theme_id).cloned()
    }

    /// Get active theme settings
    pub fn get_active_settings(&self) -> Option<Arc<ThemeSettings>> {
        let active_id = self.active_theme.read().clone()?;
        self.settings.read().get(&active_id).cloned()
    }

    /// Get template engine for a theme
    pub fn get_engine(&self, theme_id: &str) -> Option<Arc<TemplateEngine>> {
        self.engines.read().get(theme_id).cloned()
    }

    /// Get active theme engine
    pub fn get_active_engine(&self) -> Option<Arc<TemplateEngine>> {
        let active_id = self.active_theme.read().clone()?;
        self.engines.read().get(&active_id).cloned()
    }

    /// Subscribe to theme events
    pub fn subscribe(&self) -> broadcast::Receiver<ThemeEvent> {
        self.event_tx.subscribe()
    }

    /// Get themes directory
    pub fn themes_dir(&self) -> &Path {
        &self.themes_dir
    }

    /// Get theme inheritance handler
    pub fn inheritance(&self) -> &ThemeInheritance {
        &self.inheritance
    }

    /// Delete a theme
    pub async fn delete_theme(&self, theme_id: &str) -> Result<(), ThemeManagerError> {
        // Check not active
        if self.active_theme.read().as_ref() == Some(&theme_id.to_string()) {
            return Err(ThemeManagerError::DeactivationFailed(
                "Cannot delete active theme".to_string(),
            ));
        }

        // Check no children depend on it
        let has_children = self.themes.read().values()
            .any(|t| t.parent_id.as_ref() == Some(&theme_id.to_string()));

        if has_children {
            return Err(ThemeManagerError::DeactivationFailed(
                "Cannot delete theme with child themes".to_string(),
            ));
        }

        // Unregister
        self.unregister_theme(theme_id)?;

        // Delete files
        let theme_path = self.themes_dir.join(theme_id);
        if theme_path.exists() {
            fs::remove_dir_all(theme_path).await?;
        }

        Ok(())
    }

    /// Install theme from path (copy to themes directory)
    pub async fn install_from_path(&self, source: &Path) -> Result<String, ThemeManagerError> {
        // Read manifest to get theme ID
        let manifest_path = source.join("theme.toml");
        if !manifest_path.exists() {
            return Err(ThemeManagerError::Invalid("No theme.toml found".to_string()));
        }

        let manifest_content = fs::read_to_string(&manifest_path).await?;
        let manifest: ThemeManifest = toml::from_str(&manifest_content)
            .map_err(|e| ThemeManagerError::Manifest(e.to_string()))?;

        let theme_id = manifest.theme.id.clone();

        // Check if already exists
        let dest = self.themes_dir.join(&theme_id);
        if dest.exists() {
            return Err(ThemeManagerError::AlreadyExists(theme_id));
        }

        // Copy directory
        copy_dir_recursive(source, &dest).await?;

        // Register
        self.register_theme(&theme_id).await?;

        Ok(theme_id)
    }
}

/// Recursively copy a directory
async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst).await?;

    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry.file_type().await?.is_dir() {
            Box::pin(copy_dir_recursive(&src_path, &dst_path)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await?;
        }
    }

    Ok(())
}

/// Theme preview manager for live previewing themes
pub struct ThemePreview {
    manager: Arc<ThemeManager>,
    preview_sessions: Arc<RwLock<HashMap<String, PreviewSession>>>,
}

/// Preview session
#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub id: String,
    pub theme_id: String,
    pub user_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl ThemePreview {
    pub fn new(manager: Arc<ThemeManager>) -> Self {
        Self {
            manager,
            preview_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a preview session
    pub fn start_preview(&self, theme_id: &str, user_id: &str) -> Result<String, ThemeManagerError> {
        // Verify theme exists
        if !self.manager.themes.read().contains_key(theme_id) {
            return Err(ThemeManagerError::NotFound(theme_id.to_string()));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let session = PreviewSession {
            id: session_id.clone(),
            theme_id: theme_id.to_string(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
        };

        self.preview_sessions.write().insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get preview session
    pub fn get_preview(&self, session_id: &str) -> Option<PreviewSession> {
        let sessions = self.preview_sessions.read();
        let session = sessions.get(session_id)?;

        // Check expiration
        if session.expires_at < chrono::Utc::now() {
            return None;
        }

        Some(session.clone())
    }

    /// End preview session
    pub fn end_preview(&self, session_id: &str) {
        self.preview_sessions.write().remove(session_id);
    }

    /// Clean expired sessions
    pub fn clean_expired(&self) {
        let now = chrono::Utc::now();
        self.preview_sessions.write().retain(|_, s| s.expires_at > now);
    }

    /// Get theme for request (considers preview)
    pub fn get_effective_theme(&self, session_id: Option<&str>) -> Option<RegisteredTheme> {
        if let Some(sid) = session_id {
            if let Some(session) = self.get_preview(sid) {
                return self.manager.get_theme(&session.theme_id);
            }
        }

        self.manager.get_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_theme_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = ThemeManager::new(dir.path().to_path_buf());

        assert!(manager.get_active().is_none());
        assert!(manager.get_all_themes().is_empty());
    }

    #[tokio::test]
    async fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let manager = ThemeManager::new(dir.path().to_path_buf());

        let themes = manager.scan_themes().await.unwrap();
        assert!(themes.is_empty());
    }
}
