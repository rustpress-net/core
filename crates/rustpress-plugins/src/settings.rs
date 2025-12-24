//! Plugin Settings API
//!
//! Provides typed settings management for plugins.

use crate::manifest::{SettingDefinition, SettingType, SettingsSection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, warn};

/// Plugin settings manager
pub struct SettingsManager {
    /// Settings storage per plugin
    storage: Arc<RwLock<HashMap<String, PluginSettings>>>,
    /// Settings schemas per plugin
    schemas: Arc<RwLock<HashMap<String, SettingsSchema>>>,
    /// Change listeners
    listeners: Arc<RwLock<Vec<Box<dyn Fn(&str, &str, &SettingValue) + Send + Sync>>>>,
}

/// Plugin settings container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSettings {
    pub plugin_id: String,
    pub values: HashMap<String, SettingValue>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Setting value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<SettingValue>),
    Object(HashMap<String, SettingValue>),
    Null,
}

impl SettingValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SettingValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SettingValue::Integer(i) => Some(*i),
            SettingValue::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            SettingValue::Number(n) => Some(*n),
            SettingValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, SettingValue::Null)
    }
}

impl From<serde_json::Value> for SettingValue {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => SettingValue::Null,
            serde_json::Value::Bool(b) => SettingValue::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    SettingValue::Integer(i)
                } else {
                    SettingValue::Number(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => SettingValue::String(s),
            serde_json::Value::Array(arr) => {
                SettingValue::Array(arr.into_iter().map(Into::into).collect())
            }
            serde_json::Value::Object(obj) => {
                SettingValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

impl From<SettingValue> for serde_json::Value {
    fn from(v: SettingValue) -> Self {
        match v {
            SettingValue::Null => serde_json::Value::Null,
            SettingValue::Boolean(b) => serde_json::Value::Bool(b),
            SettingValue::Integer(i) => serde_json::Value::Number(i.into()),
            SettingValue::Number(n) => {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            SettingValue::String(s) => serde_json::Value::String(s),
            SettingValue::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Into::into).collect())
            }
            SettingValue::Object(obj) => {
                serde_json::Value::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

/// Settings schema
#[derive(Debug, Clone, Default)]
pub struct SettingsSchema {
    pub plugin_id: String,
    pub fields: HashMap<String, SchemaField>,
    pub groups: Vec<SettingsGroup>,
}

/// Schema field definition
#[derive(Debug, Clone)]
pub struct SchemaField {
    pub key: String,
    pub field_type: FieldType,
    pub label: String,
    pub description: Option<String>,
    pub default: Option<SettingValue>,
    pub required: bool,
    pub validation: Option<FieldValidation>,
}

/// Field type
#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Text,
    Number,
    Integer,
    Boolean,
    Select { options: Vec<SelectOption> },
    MultiSelect { options: Vec<SelectOption> },
    Radio { options: Vec<SelectOption> },
    Checkbox,
    Color,
    Date,
    DateTime,
    File,
    Image,
    Url,
    Email,
    Password,
    Json,
    Code { language: String },
}

impl From<&SettingType> for FieldType {
    fn from(st: &SettingType) -> Self {
        match st {
            SettingType::String => FieldType::String,
            SettingType::Text => FieldType::Text,
            SettingType::Number => FieldType::Number,
            SettingType::Integer => FieldType::Integer,
            SettingType::Boolean => FieldType::Boolean,
            SettingType::Select => FieldType::Select { options: Vec::new() },
            SettingType::MultiSelect => FieldType::MultiSelect { options: Vec::new() },
            SettingType::Radio => FieldType::Radio { options: Vec::new() },
            SettingType::Checkbox => FieldType::Checkbox,
            SettingType::Color => FieldType::Color,
            SettingType::Date => FieldType::Date,
            SettingType::DateTime => FieldType::DateTime,
            SettingType::File => FieldType::File,
            SettingType::Image => FieldType::Image,
            SettingType::Url => FieldType::Url,
            SettingType::Email => FieldType::Email,
            SettingType::Password => FieldType::Password,
            SettingType::Json => FieldType::Json,
            SettingType::Code => FieldType::Code { language: "text".to_string() },
        }
    }
}

/// Select option
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Field validation
#[derive(Debug, Clone)]
pub struct FieldValidation {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom: Option<String>,
}

/// Settings group
#[derive(Debug, Clone)]
pub struct SettingsGroup {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub fields: Vec<String>,
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            schemas: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register settings schema for a plugin
    pub fn register_schema(&self, plugin_id: &str, section: &SettingsSection) {
        let mut schema = SettingsSchema {
            plugin_id: plugin_id.to_string(),
            fields: HashMap::new(),
            groups: Vec::new(),
        };

        // Convert manifest settings to schema
        for (key, def) in &section.schema {
            let field = SchemaField {
                key: key.clone(),
                field_type: FieldType::from(&def.setting_type),
                label: def.label.clone(),
                description: def.description.clone(),
                default: def.default.clone().map(Into::into),
                required: def.required,
                validation: def.validation.as_ref().map(|v| FieldValidation {
                    min: v.min.as_ref().and_then(|v| v.as_f64()),
                    max: v.max.as_ref().and_then(|v| v.as_f64()),
                    min_length: None,
                    max_length: None,
                    pattern: v.pattern.clone(),
                    custom: None,
                }),
            };
            schema.fields.insert(key.clone(), field);
        }

        // Convert groups
        for group in &section.groups {
            schema.groups.push(SettingsGroup {
                id: group.id.clone(),
                label: group.label.clone(),
                description: group.description.clone(),
                fields: group.settings.clone(),
            });
        }

        self.schemas.write().insert(plugin_id.to_string(), schema);

        // Initialize storage with defaults
        let mut settings = PluginSettings {
            plugin_id: plugin_id.to_string(),
            values: HashMap::new(),
            updated_at: None,
        };

        for (key, value) in &section.defaults {
            settings.values.insert(key.clone(), value.clone().into());
        }

        self.storage.write().insert(plugin_id.to_string(), settings);
        debug!("Registered settings schema for plugin: {}", plugin_id);
    }

    /// Unregister settings for a plugin
    pub fn unregister(&self, plugin_id: &str) {
        self.schemas.write().remove(plugin_id);
        self.storage.write().remove(plugin_id);
    }

    /// Get a setting value
    pub fn get(&self, plugin_id: &str, key: &str) -> Option<SettingValue> {
        let storage = self.storage.read();
        storage.get(plugin_id)?.values.get(key).cloned()
    }

    /// Get a setting with default fallback
    pub fn get_or_default(&self, plugin_id: &str, key: &str) -> SettingValue {
        // First check stored value
        if let Some(value) = self.get(plugin_id, key) {
            return value;
        }

        // Fall back to schema default
        if let Some(schema) = self.schemas.read().get(plugin_id) {
            if let Some(field) = schema.fields.get(key) {
                if let Some(default) = &field.default {
                    return default.clone();
                }
            }
        }

        SettingValue::Null
    }

    /// Set a setting value
    pub fn set(&self, plugin_id: &str, key: &str, value: SettingValue) -> Result<(), SettingsError> {
        // Validate against schema
        if let Some(schema) = self.schemas.read().get(plugin_id) {
            if let Some(field) = schema.fields.get(key) {
                self.validate_value(&value, field)?;
            }
        }

        // Update storage
        {
            let mut storage = self.storage.write();
            let settings = storage.entry(plugin_id.to_string()).or_insert_with(|| {
                PluginSettings {
                    plugin_id: plugin_id.to_string(),
                    values: HashMap::new(),
                    updated_at: None,
                }
            });
            settings.values.insert(key.to_string(), value.clone());
            settings.updated_at = Some(chrono::Utc::now());
        }

        // Notify listeners
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener(plugin_id, key, &value);
        }

        Ok(())
    }

    /// Set multiple settings at once
    pub fn set_many(
        &self,
        plugin_id: &str,
        values: HashMap<String, SettingValue>,
    ) -> Result<(), SettingsError> {
        for (key, value) in values {
            self.set(plugin_id, &key, value)?;
        }
        Ok(())
    }

    /// Delete a setting (reset to default)
    pub fn delete(&self, plugin_id: &str, key: &str) {
        let mut storage = self.storage.write();
        if let Some(settings) = storage.get_mut(plugin_id) {
            settings.values.remove(key);
            settings.updated_at = Some(chrono::Utc::now());
        }
    }

    /// Get all settings for a plugin
    pub fn get_all(&self, plugin_id: &str) -> Option<PluginSettings> {
        self.storage.read().get(plugin_id).cloned()
    }

    /// Get settings schema for a plugin
    pub fn get_schema(&self, plugin_id: &str) -> Option<SettingsSchema> {
        self.schemas.read().get(plugin_id).cloned()
    }

    /// Add a change listener
    pub fn on_change<F>(&self, listener: F)
    where
        F: Fn(&str, &str, &SettingValue) + Send + Sync + 'static,
    {
        self.listeners.write().push(Box::new(listener));
    }

    /// Validate a value against a field
    fn validate_value(&self, value: &SettingValue, field: &SchemaField) -> Result<(), SettingsError> {
        // Check required
        if field.required && value.is_null() {
            return Err(SettingsError::Required(field.key.clone()));
        }

        // Type validation
        match (&field.field_type, value) {
            (FieldType::String | FieldType::Text | FieldType::Email | FieldType::Url | FieldType::Password, SettingValue::String(_)) => {}
            (FieldType::Number, SettingValue::Number(_) | SettingValue::Integer(_)) => {}
            (FieldType::Integer, SettingValue::Integer(_)) => {}
            (FieldType::Boolean | FieldType::Checkbox, SettingValue::Boolean(_)) => {}
            (_, SettingValue::Null) => {} // Null is allowed for non-required fields
            _ => {
                return Err(SettingsError::TypeMismatch {
                    key: field.key.clone(),
                    expected: format!("{:?}", field.field_type),
                });
            }
        }

        // Range validation
        if let Some(validation) = &field.validation {
            if let Some(num) = value.as_f64() {
                if let Some(min) = validation.min {
                    if num < min {
                        return Err(SettingsError::RangeError {
                            key: field.key.clone(),
                            min: validation.min,
                            max: validation.max,
                        });
                    }
                }
                if let Some(max) = validation.max {
                    if num > max {
                        return Err(SettingsError::RangeError {
                            key: field.key.clone(),
                            min: validation.min,
                            max: validation.max,
                        });
                    }
                }
            }

            // Pattern validation
            if let (Some(pattern), Some(s)) = (&validation.pattern, value.as_str()) {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if !re.is_match(s) {
                        return Err(SettingsError::PatternMismatch {
                            key: field.key.clone(),
                            pattern: pattern.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Export settings to JSON
    pub fn export(&self, plugin_id: &str) -> Option<serde_json::Value> {
        let settings = self.storage.read().get(plugin_id)?.clone();
        let values: HashMap<String, serde_json::Value> = settings
            .values
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        Some(serde_json::json!({
            "plugin_id": settings.plugin_id,
            "values": values,
            "updated_at": settings.updated_at
        }))
    }

    /// Import settings from JSON
    pub fn import(&self, plugin_id: &str, data: serde_json::Value) -> Result<(), SettingsError> {
        let values = data
            .get("values")
            .and_then(|v| v.as_object())
            .ok_or(SettingsError::ImportError("Invalid settings format".to_string()))?;

        for (key, value) in values {
            self.set(plugin_id, key, value.clone().into())?;
        }

        Ok(())
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings error
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Setting '{0}' is required")]
    Required(String),

    #[error("Type mismatch for '{key}': expected {expected}")]
    TypeMismatch { key: String, expected: String },

    #[error("Value for '{key}' out of range (min: {min:?}, max: {max:?})")]
    RangeError {
        key: String,
        min: Option<f64>,
        max: Option<f64>,
    },

    #[error("Value for '{key}' doesn't match pattern: {pattern}")]
    PatternMismatch { key: String, pattern: String },

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Import error: {0}")]
    ImportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_value_conversions() {
        let json = serde_json::json!({
            "name": "test",
            "count": 42,
            "enabled": true
        });

        let value: SettingValue = json.into();
        if let SettingValue::Object(obj) = value {
            assert!(matches!(obj.get("name"), Some(SettingValue::String(s)) if s == "test"));
            assert!(matches!(obj.get("count"), Some(SettingValue::Integer(42))));
            assert!(matches!(obj.get("enabled"), Some(SettingValue::Boolean(true))));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_settings_manager() {
        let manager = SettingsManager::new();

        manager.set("test-plugin", "api_key", SettingValue::String("secret".to_string())).unwrap();

        let value = manager.get("test-plugin", "api_key");
        assert!(matches!(value, Some(SettingValue::String(s)) if s == "secret"));

        manager.delete("test-plugin", "api_key");
        assert!(manager.get("test-plugin", "api_key").is_none());
    }
}
