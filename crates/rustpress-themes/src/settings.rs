//! Theme Settings API
//!
//! Persistent storage and retrieval of theme settings with validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::fs;

/// Settings error types
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Setting not found: {0}")]
    NotFound(String),

    #[error("Invalid setting value: {0}")]
    InvalidValue(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Theme settings manager
pub struct ThemeSettings {
    /// Theme ID
    theme_id: String,
    /// Settings storage path
    storage_path: PathBuf,
    /// Current settings
    settings: Arc<RwLock<HashMap<String, SettingValue>>>,
    /// Settings schema
    schema: Arc<RwLock<HashMap<String, SettingSchema>>>,
    /// Mods (theme modifications)
    mods: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Change listeners
    listeners: Arc<RwLock<Vec<Box<dyn Fn(&str, &SettingValue) + Send + Sync>>>>,
}

/// Individual setting value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingValue {
    pub value: serde_json::Value,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub modified_by: Option<String>,
    pub autoload: bool,
}

/// Setting schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingSchema {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub setting_type: SettingType,
    pub default: serde_json::Value,
    pub sanitize: Option<SanitizeType>,
    pub validate: Option<ValidationRule>,
    pub capability: String,
    pub show_in_rest: bool,
    pub transport: SettingTransport,
}

/// Setting types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    Color,
    Image,
    Url,
    Html,
}

/// Sanitization types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SanitizeType {
    Text,
    TextMultiline,
    Html,
    HtmlAllowed { tags: Vec<String> },
    Url,
    Email,
    FileName,
    HexColor,
    RgbaColor,
    Integer,
    Float,
    Boolean,
    Css,
    JavaScript,
    Custom { callback: String },
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Min(f64),
    Max(f64),
    Range { min: f64, max: f64 },
    Pattern(String),
    Enum { values: Vec<serde_json::Value> },
    Custom { callback: String },
    All(Vec<ValidationRule>),
    Any(Vec<ValidationRule>),
}

/// How setting changes are transported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingTransport {
    Refresh,
    PostMessage,
}

impl Default for SettingTransport {
    fn default() -> Self {
        Self::Refresh
    }
}

impl ThemeSettings {
    pub fn new(theme_id: &str, storage_path: PathBuf) -> Self {
        Self {
            theme_id: theme_id.to_string(),
            storage_path,
            settings: Arc::new(RwLock::new(HashMap::new())),
            schema: Arc::new(RwLock::new(HashMap::new())),
            mods: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Load settings from storage
    pub async fn load(&self) -> Result<(), SettingsError> {
        let settings_file = self.storage_path.join(format!("{}_settings.json", self.theme_id));

        if settings_file.exists() {
            let content = fs::read_to_string(&settings_file).await?;
            let loaded: HashMap<String, SettingValue> = serde_json::from_str(&content)?;
            *self.settings.write() = loaded;
        }

        let mods_file = self.storage_path.join(format!("{}_mods.json", self.theme_id));

        if mods_file.exists() {
            let content = fs::read_to_string(&mods_file).await?;
            let loaded: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
            *self.mods.write() = loaded;
        }

        Ok(())
    }

    /// Save settings to storage
    pub async fn save(&self) -> Result<(), SettingsError> {
        fs::create_dir_all(&self.storage_path).await?;

        let settings_file = self.storage_path.join(format!("{}_settings.json", self.theme_id));
        let content = serde_json::to_string_pretty(&*self.settings.read())?;
        fs::write(&settings_file, content).await?;

        let mods_file = self.storage_path.join(format!("{}_mods.json", self.theme_id));
        let content = serde_json::to_string_pretty(&*self.mods.read())?;
        fs::write(&mods_file, content).await?;

        Ok(())
    }

    /// Register a setting schema
    pub fn register(&self, schema: SettingSchema) {
        let id = schema.id.clone();
        self.schema.write().insert(id.clone(), schema.clone());

        // Set default if not already set
        if !self.settings.read().contains_key(&id) {
            let value = SettingValue {
                value: schema.default,
                modified_at: chrono::Utc::now(),
                modified_by: None,
                autoload: true,
            };
            self.settings.write().insert(id, value);
        }
    }

    /// Get a setting value
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.settings
            .read()
            .get(key)
            .map(|v| v.value.clone())
    }

    /// Get a setting value with default
    pub fn get_or_default(&self, key: &str, default: serde_json::Value) -> serde_json::Value {
        self.get(key).unwrap_or(default)
    }

    /// Get typed setting value
    pub fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.get(key)
            .and_then(|v| serde_json::from_value(v).ok())
    }

    /// Set a setting value
    pub fn set(&self, key: &str, value: serde_json::Value, user_id: Option<&str>) -> Result<(), SettingsError> {
        // Validate against schema if exists
        if let Some(schema) = self.schema.read().get(key) {
            self.validate_value(&schema.setting_type, &value)?;

            if let Some(rule) = &schema.validate {
                self.apply_validation(rule, &value)?;
            }

            // Sanitize value
            let sanitized = if let Some(sanitize) = &schema.sanitize {
                self.sanitize_value(sanitize, value)?
            } else {
                value
            };

            let setting = SettingValue {
                value: sanitized.clone(),
                modified_at: chrono::Utc::now(),
                modified_by: user_id.map(|s| s.to_string()),
                autoload: true,
            };

            self.settings.write().insert(key.to_string(), setting.clone());

            // Notify listeners
            for listener in self.listeners.read().iter() {
                listener(key, &setting);
            }

            Ok(())
        } else {
            // No schema, just store
            let setting = SettingValue {
                value,
                modified_at: chrono::Utc::now(),
                modified_by: user_id.map(|s| s.to_string()),
                autoload: true,
            };

            self.settings.write().insert(key.to_string(), setting);
            Ok(())
        }
    }

    /// Remove a setting
    pub fn remove(&self, key: &str) {
        self.settings.write().remove(key);
    }

    /// Get theme mod
    pub fn get_mod(&self, name: &str) -> Option<serde_json::Value> {
        self.mods.read().get(name).cloned()
    }

    /// Get theme mod with default
    pub fn get_mod_or_default(&self, name: &str, default: serde_json::Value) -> serde_json::Value {
        self.get_mod(name).unwrap_or(default)
    }

    /// Set theme mod
    pub fn set_mod(&self, name: &str, value: serde_json::Value) {
        self.mods.write().insert(name.to_string(), value);
    }

    /// Remove theme mod
    pub fn remove_mod(&self, name: &str) {
        self.mods.write().remove(name);
    }

    /// Get all theme mods
    pub fn get_all_mods(&self) -> HashMap<String, serde_json::Value> {
        self.mods.read().clone()
    }

    fn validate_value(&self, setting_type: &SettingType, value: &serde_json::Value) -> Result<(), SettingsError> {
        match setting_type {
            SettingType::String | SettingType::Html | SettingType::Url | SettingType::Color | SettingType::Image => {
                if !value.is_string() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected string".to_string()));
                }
            }
            SettingType::Integer => {
                if !value.is_i64() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected integer".to_string()));
                }
            }
            SettingType::Number => {
                if !value.is_number() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected number".to_string()));
                }
            }
            SettingType::Boolean => {
                if !value.is_boolean() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected boolean".to_string()));
                }
            }
            SettingType::Array => {
                if !value.is_array() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected array".to_string()));
                }
            }
            SettingType::Object => {
                if !value.is_object() && !value.is_null() {
                    return Err(SettingsError::InvalidValue("Expected object".to_string()));
                }
            }
        }

        Ok(())
    }

    fn apply_validation(&self, rule: &ValidationRule, value: &serde_json::Value) -> Result<(), SettingsError> {
        match rule {
            ValidationRule::Required => {
                if value.is_null() {
                    return Err(SettingsError::SchemaValidation("Value is required".to_string()));
                }
            }
            ValidationRule::MinLength(min) => {
                if let Some(s) = value.as_str() {
                    if s.len() < *min {
                        return Err(SettingsError::SchemaValidation(
                            format!("Minimum length is {}", min),
                        ));
                    }
                }
            }
            ValidationRule::MaxLength(max) => {
                if let Some(s) = value.as_str() {
                    if s.len() > *max {
                        return Err(SettingsError::SchemaValidation(
                            format!("Maximum length is {}", max),
                        ));
                    }
                }
            }
            ValidationRule::Min(min) => {
                if let Some(n) = value.as_f64() {
                    if n < *min {
                        return Err(SettingsError::SchemaValidation(
                            format!("Minimum value is {}", min),
                        ));
                    }
                }
            }
            ValidationRule::Max(max) => {
                if let Some(n) = value.as_f64() {
                    if n > *max {
                        return Err(SettingsError::SchemaValidation(
                            format!("Maximum value is {}", max),
                        ));
                    }
                }
            }
            ValidationRule::Range { min, max } => {
                if let Some(n) = value.as_f64() {
                    if n < *min || n > *max {
                        return Err(SettingsError::SchemaValidation(
                            format!("Value must be between {} and {}", min, max),
                        ));
                    }
                }
            }
            ValidationRule::Pattern(pattern) => {
                if let Some(s) = value.as_str() {
                    let re = regex::Regex::new(pattern)
                        .map_err(|e| SettingsError::SchemaValidation(e.to_string()))?;
                    if !re.is_match(s) {
                        return Err(SettingsError::SchemaValidation(
                            format!("Value does not match pattern: {}", pattern),
                        ));
                    }
                }
            }
            ValidationRule::Enum { values } => {
                if !values.contains(value) {
                    return Err(SettingsError::SchemaValidation(
                        "Value not in allowed set".to_string(),
                    ));
                }
            }
            ValidationRule::All(rules) => {
                for rule in rules {
                    self.apply_validation(rule, value)?;
                }
            }
            ValidationRule::Any(rules) => {
                let mut passed = false;
                for rule in rules {
                    if self.apply_validation(rule, value).is_ok() {
                        passed = true;
                        break;
                    }
                }
                if !passed {
                    return Err(SettingsError::SchemaValidation(
                        "No validation rule passed".to_string(),
                    ));
                }
            }
            ValidationRule::Custom { .. } => {
                // Custom validation would be handled by external callback
            }
        }

        Ok(())
    }

    fn sanitize_value(&self, sanitize: &SanitizeType, value: serde_json::Value) -> Result<serde_json::Value, SettingsError> {
        match sanitize {
            SanitizeType::Text => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_text(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::TextMultiline => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_text_multiline(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Html => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_html(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::HtmlAllowed { tags } => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_html_allowed(s, tags)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Url => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_url(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Email => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_email(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::HexColor => {
                if let Some(s) = value.as_str() {
                    Ok(serde_json::Value::String(sanitize_hex_color(s)))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Integer => {
                if let Some(n) = value.as_f64() {
                    Ok(serde_json::Value::Number(
                        serde_json::Number::from(n as i64),
                    ))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Float => {
                if let Some(n) = value.as_f64() {
                    Ok(serde_json::json!(n))
                } else {
                    Ok(value)
                }
            }
            SanitizeType::Boolean => {
                Ok(serde_json::Value::Bool(value.as_bool().unwrap_or(false)))
            }
            _ => Ok(value),
        }
    }

    /// Add change listener
    pub fn on_change<F>(&self, listener: F)
    where
        F: Fn(&str, &SettingValue) + Send + Sync + 'static,
    {
        self.listeners.write().push(Box::new(listener));
    }

    /// Get all settings for autoload
    pub fn get_autoload(&self) -> HashMap<String, serde_json::Value> {
        self.settings
            .read()
            .iter()
            .filter(|(_, v)| v.autoload)
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Export settings as JSON
    pub fn export(&self) -> serde_json::Value {
        serde_json::json!({
            "theme_id": self.theme_id,
            "settings": *self.settings.read(),
            "mods": *self.mods.read(),
        })
    }

    /// Import settings from JSON
    pub fn import(&self, data: serde_json::Value) -> Result<(), SettingsError> {
        if let Some(settings) = data.get("settings").and_then(|v| v.as_object()) {
            for (key, value) in settings {
                if let Ok(setting) = serde_json::from_value::<SettingValue>(value.clone()) {
                    self.settings.write().insert(key.clone(), setting);
                }
            }
        }

        if let Some(mods) = data.get("mods").and_then(|v| v.as_object()) {
            for (key, value) in mods {
                self.mods.write().insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    /// Reset to defaults
    pub fn reset(&self) {
        let schema = self.schema.read();
        let mut settings = self.settings.write();

        for (id, s) in schema.iter() {
            settings.insert(
                id.clone(),
                SettingValue {
                    value: s.default.clone(),
                    modified_at: chrono::Utc::now(),
                    modified_by: None,
                    autoload: true,
                },
            );
        }

        self.mods.write().clear();
    }
}

/// Global settings registry
pub struct GlobalSettingsRegistry {
    themes: Arc<RwLock<HashMap<String, Arc<ThemeSettings>>>>,
    active_theme: Arc<RwLock<Option<String>>>,
}

impl GlobalSettingsRegistry {
    pub fn new() -> Self {
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
            active_theme: Arc::new(RwLock::new(None)),
        }
    }

    /// Register theme settings
    pub fn register(&self, theme_id: &str, settings: Arc<ThemeSettings>) {
        self.themes.write().insert(theme_id.to_string(), settings);
    }

    /// Set active theme
    pub fn set_active(&self, theme_id: &str) {
        *self.active_theme.write() = Some(theme_id.to_string());
    }

    /// Get active theme settings
    pub fn get_active(&self) -> Option<Arc<ThemeSettings>> {
        let active = self.active_theme.read();
        active.as_ref().and_then(|id| {
            self.themes.read().get(id).cloned()
        })
    }

    /// Get theme settings by ID
    pub fn get(&self, theme_id: &str) -> Option<Arc<ThemeSettings>> {
        self.themes.read().get(theme_id).cloned()
    }

    /// Unregister theme
    pub fn unregister(&self, theme_id: &str) {
        self.themes.write().remove(theme_id);
    }
}

impl Default for GlobalSettingsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Sanitization functions

fn sanitize_text(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect::<String>()
        .trim()
        .to_string()
}

fn sanitize_text_multiline(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\t' || *c == '\n' || *c == '\r')
        .collect::<String>()
        .trim()
        .to_string()
}

fn sanitize_html(s: &str) -> String {
    // Basic HTML escaping
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn sanitize_html_allowed(s: &str, allowed_tags: &[String]) -> String {
    // In a real implementation, use an HTML parser
    // This is a simplified version
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut current_tag = String::new();

    for c in s.chars() {
        if c == '<' {
            in_tag = true;
            tag_name.clear();
            current_tag.clear();
            current_tag.push(c);
        } else if c == '>' && in_tag {
            current_tag.push(c);
            in_tag = false;

            // Check if tag is allowed
            let normalized_tag = tag_name.to_lowercase();
            let tag_without_slash = normalized_tag.trim_start_matches('/');

            if allowed_tags.iter().any(|t| t.to_lowercase() == tag_without_slash) {
                result.push_str(&current_tag);
            }
        } else if in_tag {
            current_tag.push(c);
            if c.is_alphabetic() || c == '/' {
                tag_name.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn sanitize_url(s: &str) -> String {
    // Basic URL sanitization
    let s = s.trim();

    // Check for dangerous protocols
    let lower = s.to_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
    {
        return String::new();
    }

    s.to_string()
}

fn sanitize_email(s: &str) -> String {
    let s = s.trim().to_lowercase();

    // Basic email validation
    if s.contains('@') && s.contains('.') {
        s
    } else {
        String::new()
    }
}

fn sanitize_hex_color(s: &str) -> String {
    let s = s.trim();

    // Handle colors with or without #
    let hex = if s.starts_with('#') { &s[1..] } else { s };

    // Validate hex characters
    if hex.chars().all(|c| c.is_ascii_hexdigit()) {
        match hex.len() {
            3 | 6 | 8 => format!("#{}", hex.to_lowercase()),
            _ => String::new(),
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_theme_settings() {
        let dir = tempdir().unwrap();
        let settings = ThemeSettings::new("test-theme", dir.path().to_path_buf());

        settings.set("key1", serde_json::json!("value1"), None).unwrap();
        assert_eq!(settings.get("key1"), Some(serde_json::json!("value1")));
    }

    #[test]
    fn test_sanitize_hex_color() {
        assert_eq!(sanitize_hex_color("#fff"), "#fff");
        assert_eq!(sanitize_hex_color("#FFFFFF"), "#ffffff");
        assert_eq!(sanitize_hex_color("abc"), "#abc");
        assert_eq!(sanitize_hex_color("invalid"), "");
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(sanitize_url("https://example.com"), "https://example.com");
        assert_eq!(sanitize_url("javascript:alert(1)"), "");
        assert_eq!(sanitize_url("data:text/html,<script>"), "");
    }

    #[test]
    fn test_sanitize_html() {
        assert_eq!(sanitize_html("<script>"), "&lt;script&gt;");
        assert_eq!(sanitize_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_validation_rules() {
        let settings = ThemeSettings::new("test", PathBuf::from("/tmp"));

        // Register with validation
        settings.register(SettingSchema {
            id: "age".to_string(),
            label: "Age".to_string(),
            description: None,
            setting_type: SettingType::Integer,
            default: serde_json::json!(18),
            sanitize: None,
            validate: Some(ValidationRule::Range { min: 0.0, max: 150.0 }),
            capability: "edit_theme_options".to_string(),
            show_in_rest: true,
            transport: SettingTransport::PostMessage,
        });

        // Valid value
        assert!(settings.set("age", serde_json::json!(25), None).is_ok());

        // Invalid value
        assert!(settings.set("age", serde_json::json!(200), None).is_err());
    }
}
