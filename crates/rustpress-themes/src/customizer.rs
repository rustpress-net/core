//! Theme Customizer with Live Preview
//!
//! Provides real-time theme customization with live preview capabilities.

use crate::manifest::{ThemeManifest, ColorDefinition, FontFamily, LayoutSection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Customizer error types
#[derive(Debug, Error)]
pub enum CustomizerError {
    #[error("Section not found: {0}")]
    SectionNotFound(String),

    #[error("Control not found: {0}")]
    ControlNotFound(String),

    #[error("Invalid value for control {control}: {message}")]
    InvalidValue { control: String, message: String },

    #[error("Preview session not found: {0}")]
    SessionNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Theme customizer manager
pub struct ThemeCustomizer {
    /// Registered sections
    sections: Arc<RwLock<HashMap<String, CustomizerSection>>>,
    /// Registered controls
    controls: Arc<RwLock<HashMap<String, CustomizerControl>>>,
    /// Active preview sessions
    sessions: Arc<RwLock<HashMap<String, PreviewSession>>>,
    /// Change broadcaster for live preview
    change_tx: broadcast::Sender<CustomizerChange>,
    /// Default values
    defaults: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

/// Customizer section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizerSection {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub panel: Option<String>,
    pub capability: String,
    pub theme_supports: Option<String>,
    pub active_callback: Option<String>,
}

/// Customizer control types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlType {
    Text {
        placeholder: Option<String>,
        input_attrs: HashMap<String, String>,
    },
    Textarea {
        rows: Option<u32>,
    },
    Checkbox,
    Radio {
        choices: Vec<ControlChoice>,
    },
    Select {
        choices: Vec<ControlChoice>,
    },
    Dropdown {
        choices: Vec<ControlChoice>,
        allow_empty: bool,
    },
    Color {
        show_opacity: bool,
    },
    Image {
        extensions: Vec<String>,
    },
    Media {
        mime_types: Vec<String>,
    },
    Upload {
        extensions: Vec<String>,
        max_size: Option<u64>,
    },
    Range {
        min: f64,
        max: f64,
        step: f64,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    Url {
        placeholder: Option<String>,
    },
    Email,
    Date,
    DateTime,
    Code {
        language: String,
    },
    Font {
        google_fonts: bool,
        system_fonts: bool,
        custom_fonts: bool,
    },
    Spacing {
        linked: bool,
        units: Vec<String>,
    },
    Background {
        show_color: bool,
        show_image: bool,
        show_position: bool,
        show_size: bool,
        show_repeat: bool,
        show_attachment: bool,
    },
    Gradient {
        types: Vec<String>,
    },
    Border {
        show_width: bool,
        show_style: bool,
        show_color: bool,
        show_radius: bool,
    },
}

/// Choice for select/radio controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlChoice {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

/// Customizer control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizerControl {
    pub id: String,
    pub section: String,
    pub label: String,
    pub description: Option<String>,
    pub control_type: ControlType,
    pub priority: i32,
    pub capability: String,
    pub default: Option<serde_json::Value>,
    pub transport: Transport,
    pub sanitize_callback: Option<String>,
    pub validate_callback: Option<String>,
    pub active_callback: Option<String>,
    /// CSS selector for live preview
    pub css_selector: Option<String>,
    /// CSS property for live preview
    pub css_property: Option<String>,
}

/// How changes are transported to the preview
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// Full page refresh
    Refresh,
    /// Partial refresh via AJAX
    PostMessage,
}

impl Default for Transport {
    fn default() -> Self {
        Self::PostMessage
    }
}

/// Preview session state
#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub id: String,
    pub theme_id: String,
    pub user_id: String,
    pub changeset: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: SessionStatus,
}

/// Preview session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Draft,
    Scheduled,
    Published,
    Trashed,
}

/// Change notification for live preview
#[derive(Debug, Clone, Serialize)]
pub struct CustomizerChange {
    pub session_id: String,
    pub control_id: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub css_rules: Vec<CssRule>,
}

/// CSS rule for live preview
#[derive(Debug, Clone, Serialize)]
pub struct CssRule {
    pub selector: String,
    pub property: String,
    pub value: String,
}

impl ThemeCustomizer {
    pub fn new() -> Self {
        let (change_tx, _) = broadcast::channel(100);

        Self {
            sections: Arc::new(RwLock::new(HashMap::new())),
            controls: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            change_tx,
            defaults: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register sections from theme manifest
    pub fn register_from_manifest(&self, manifest: &ThemeManifest) {
        // Register color section
        self.register_color_section(manifest);

        // Register typography section
        self.register_typography_section(manifest);

        // Register layout section
        self.register_layout_section(manifest);

        // Register custom settings sections
        for section in &manifest.settings.sections {
            self.register_settings_section(&manifest.theme.id, section, &manifest.settings.settings);
        }
    }

    fn register_color_section(&self, manifest: &ThemeManifest) {
        let section = CustomizerSection {
            id: "colors".to_string(),
            title: "Colors".to_string(),
            description: Some("Customize theme colors".to_string()),
            priority: 40,
            panel: None,
            capability: "edit_theme_options".to_string(),
            theme_supports: None,
            active_callback: None,
        };

        self.add_section(section);

        // Add color controls from palette
        for (i, color) in manifest.colors.palette.iter().enumerate() {
            let control = CustomizerControl {
                id: format!("color_{}", color.slug),
                section: "colors".to_string(),
                label: color.name.clone(),
                description: None,
                control_type: ControlType::Color { show_opacity: true },
                priority: i as i32 * 10,
                capability: "edit_theme_options".to_string(),
                default: Some(serde_json::Value::String(color.color.clone())),
                transport: Transport::PostMessage,
                sanitize_callback: Some("sanitize_hex_color".to_string()),
                validate_callback: None,
                active_callback: None,
                css_selector: Some(":root".to_string()),
                css_property: Some(format!("--wp--preset--color--{}", color.slug)),
            };

            self.add_control(control);
        }
    }

    fn register_typography_section(&self, manifest: &ThemeManifest) {
        let section = CustomizerSection {
            id: "typography".to_string(),
            title: "Typography".to_string(),
            description: Some("Customize fonts and text styles".to_string()),
            priority: 50,
            panel: None,
            capability: "edit_theme_options".to_string(),
            theme_supports: None,
            active_callback: None,
        };

        self.add_section(section);

        // Add font family controls
        for (i, font) in manifest.typography.font_families.iter().enumerate() {
            let control = CustomizerControl {
                id: format!("font_{}", font.slug),
                section: "typography".to_string(),
                label: font.name.clone(),
                description: None,
                control_type: ControlType::Font {
                    google_fonts: true,
                    system_fonts: true,
                    custom_fonts: true,
                },
                priority: i as i32 * 10,
                capability: "edit_theme_options".to_string(),
                default: Some(serde_json::Value::String(font.font_family.clone())),
                transport: Transport::PostMessage,
                sanitize_callback: None,
                validate_callback: None,
                active_callback: None,
                css_selector: Some(":root".to_string()),
                css_property: Some(format!("--wp--preset--font-family--{}", font.slug)),
            };

            self.add_control(control);
        }

        // Add font size controls
        for (i, size) in manifest.typography.font_sizes.iter().enumerate() {
            let control = CustomizerControl {
                id: format!("font_size_{}", size.slug),
                section: "typography".to_string(),
                label: size.name.clone(),
                description: None,
                control_type: ControlType::Text {
                    placeholder: Some("e.g., 16px, 1rem".to_string()),
                    input_attrs: HashMap::new(),
                },
                priority: 100 + i as i32 * 10,
                capability: "edit_theme_options".to_string(),
                default: Some(serde_json::Value::String(size.size.clone())),
                transport: Transport::PostMessage,
                sanitize_callback: None,
                validate_callback: None,
                active_callback: None,
                css_selector: Some(":root".to_string()),
                css_property: Some(format!("--wp--preset--font-size--{}", size.slug)),
            };

            self.add_control(control);
        }
    }

    fn register_layout_section(&self, manifest: &ThemeManifest) {
        let section = CustomizerSection {
            id: "layout".to_string(),
            title: "Layout".to_string(),
            description: Some("Customize layout settings".to_string()),
            priority: 60,
            panel: None,
            capability: "edit_theme_options".to_string(),
            theme_supports: None,
            active_callback: None,
        };

        self.add_section(section);

        // Content width control
        self.add_control(CustomizerControl {
            id: "content_width".to_string(),
            section: "layout".to_string(),
            label: "Content Width".to_string(),
            description: Some("Maximum width for content area".to_string()),
            control_type: ControlType::Text {
                placeholder: Some("e.g., 1200px".to_string()),
                input_attrs: HashMap::new(),
            },
            priority: 10,
            capability: "edit_theme_options".to_string(),
            default: manifest.layout.content_width.clone().map(|s| serde_json::Value::String(s)),
            transport: Transport::PostMessage,
            sanitize_callback: None,
            validate_callback: None,
            active_callback: None,
            css_selector: Some(":root".to_string()),
            css_property: Some("--wp--style--global--content-size".to_string()),
        });

        // Wide width control
        self.add_control(CustomizerControl {
            id: "wide_width".to_string(),
            section: "layout".to_string(),
            label: "Wide Width".to_string(),
            description: Some("Maximum width for wide-aligned content".to_string()),
            control_type: ControlType::Text {
                placeholder: Some("e.g., 1400px".to_string()),
                input_attrs: HashMap::new(),
            },
            priority: 20,
            capability: "edit_theme_options".to_string(),
            default: manifest.layout.wide_width.clone().map(|s| serde_json::Value::String(s)),
            transport: Transport::PostMessage,
            sanitize_callback: None,
            validate_callback: None,
            active_callback: None,
            css_selector: Some(":root".to_string()),
            css_property: Some("--wp--style--global--wide-size".to_string()),
        });
    }

    fn register_settings_section(
        &self,
        theme_id: &str,
        section: &crate::manifest::SettingSection,
        all_settings: &[crate::manifest::SettingDefinition],
    ) {
        let customizer_section = CustomizerSection {
            id: format!("{}_{}", theme_id, section.id),
            title: section.title.clone(),
            description: section.description.clone(),
            priority: 100,
            panel: None,
            capability: "edit_theme_options".to_string(),
            theme_supports: None,
            active_callback: None,
        };

        self.add_section(customizer_section);

        // Find settings that belong to this section
        let section_settings: Vec<_> = all_settings.iter()
            .filter(|s| s.section.as_ref() == Some(&section.id))
            .collect();

        // Register controls for each setting
        for (i, setting) in section_settings.iter().enumerate() {
            let control_type = match &setting.setting_type {
                crate::manifest::SettingType::Text => ControlType::Text {
                    placeholder: None,
                    input_attrs: HashMap::new(),
                },
                crate::manifest::SettingType::Textarea => ControlType::Textarea { rows: Some(5) },
                crate::manifest::SettingType::Checkbox => ControlType::Checkbox,
                crate::manifest::SettingType::Color => ControlType::Color { show_opacity: true },
                crate::manifest::SettingType::Image => ControlType::Image {
                    extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string(), "gif".to_string()],
                },
                crate::manifest::SettingType::Select | crate::manifest::SettingType::Dropdown => {
                    ControlType::Select {
                        choices: setting.choices.as_ref().map(|choices| {
                            choices.iter().map(|o| ControlChoice {
                                value: o.value.clone(),
                                label: o.label.clone(),
                                description: None,
                            }).collect()
                        }).unwrap_or_default(),
                    }
                },
                crate::manifest::SettingType::Range | crate::manifest::SettingType::Number => {
                    ControlType::Range {
                        min: 0.0,
                        max: 100.0,
                        step: 1.0,
                    }
                },
                _ => ControlType::Text {
                    placeholder: None,
                    input_attrs: HashMap::new(),
                },
            };

            let control = CustomizerControl {
                id: format!("{}_{}_{}", theme_id, section.id, setting.id),
                section: format!("{}_{}", theme_id, section.id),
                label: setting.label.clone(),
                description: setting.description.clone(),
                control_type,
                priority: i as i32 * 10,
                capability: "edit_theme_options".to_string(),
                default: setting.default.clone(),
                transport: Transport::PostMessage,
                sanitize_callback: None,
                validate_callback: None,
                active_callback: None,
                css_selector: None,
                css_property: None,
            };

            self.add_control(control);
        }
    }

    /// Add a section
    pub fn add_section(&self, section: CustomizerSection) {
        self.sections.write().insert(section.id.clone(), section);
    }

    /// Add a control
    pub fn add_control(&self, control: CustomizerControl) {
        if let Some(default) = &control.default {
            self.defaults.write().insert(control.id.clone(), default.clone());
        }
        self.controls.write().insert(control.id.clone(), control);
    }

    /// Get all sections
    pub fn get_sections(&self) -> Vec<CustomizerSection> {
        let mut sections: Vec<_> = self.sections.read().values().cloned().collect();
        sections.sort_by_key(|s| s.priority);
        sections
    }

    /// Get controls for a section
    pub fn get_controls_for_section(&self, section_id: &str) -> Vec<CustomizerControl> {
        let mut controls: Vec<_> = self.controls
            .read()
            .values()
            .filter(|c| c.section == section_id)
            .cloned()
            .collect();
        controls.sort_by_key(|c| c.priority);
        controls
    }

    /// Start a preview session
    pub fn start_session(&self, theme_id: &str, user_id: &str) -> String {
        let session_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let session = PreviewSession {
            id: session_id.clone(),
            theme_id: theme_id.to_string(),
            user_id: user_id.to_string(),
            changeset: HashMap::new(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            status: SessionStatus::Draft,
        };

        self.sessions.write().insert(session_id.clone(), session);
        session_id
    }

    /// Update a value in the preview session
    pub fn update_value(
        &self,
        session_id: &str,
        control_id: &str,
        value: serde_json::Value,
    ) -> Result<(), CustomizerError> {
        let mut sessions = self.sessions.write();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| CustomizerError::SessionNotFound(session_id.to_string()))?;

        let control = self.controls
            .read()
            .get(control_id)
            .cloned()
            .ok_or_else(|| CustomizerError::ControlNotFound(control_id.to_string()))?;

        // Validate value
        self.validate_value(&control, &value)?;

        let old_value = session.changeset.get(control_id).cloned();
        session.changeset.insert(control_id.to_string(), value.clone());

        // Generate CSS rules for live preview
        let css_rules = self.generate_css_rules(&control, &value);

        // Broadcast change for live preview
        let change = CustomizerChange {
            session_id: session_id.to_string(),
            control_id: control_id.to_string(),
            old_value,
            new_value: value,
            css_rules,
        };

        let _ = self.change_tx.send(change);

        Ok(())
    }

    fn validate_value(&self, control: &CustomizerControl, value: &serde_json::Value) -> Result<(), CustomizerError> {
        match &control.control_type {
            ControlType::Range { min, max, .. } => {
                if let Some(num) = value.as_f64() {
                    if num < *min || num > *max {
                        return Err(CustomizerError::InvalidValue {
                            control: control.id.clone(),
                            message: format!("Value must be between {} and {}", min, max),
                        });
                    }
                }
            }
            ControlType::Number { min, max, .. } => {
                if let Some(num) = value.as_f64() {
                    if let Some(min_val) = min {
                        if num < *min_val {
                            return Err(CustomizerError::InvalidValue {
                                control: control.id.clone(),
                                message: format!("Value must be at least {}", min_val),
                            });
                        }
                    }
                    if let Some(max_val) = max {
                        if num > *max_val {
                            return Err(CustomizerError::InvalidValue {
                                control: control.id.clone(),
                                message: format!("Value must be at most {}", max_val),
                            });
                        }
                    }
                }
            }
            ControlType::Color { .. } => {
                if let Some(color) = value.as_str() {
                    if !is_valid_color(color) {
                        return Err(CustomizerError::InvalidValue {
                            control: control.id.clone(),
                            message: "Invalid color format".to_string(),
                        });
                    }
                }
            }
            ControlType::Email => {
                if let Some(email) = value.as_str() {
                    if !email.contains('@') {
                        return Err(CustomizerError::InvalidValue {
                            control: control.id.clone(),
                            message: "Invalid email address".to_string(),
                        });
                    }
                }
            }
            ControlType::Url { .. } => {
                if let Some(url) = value.as_str() {
                    if url::Url::parse(url).is_err() {
                        return Err(CustomizerError::InvalidValue {
                            control: control.id.clone(),
                            message: "Invalid URL".to_string(),
                        });
                    }
                }
            }
            ControlType::Select { choices } | ControlType::Radio { choices } => {
                if let Some(selected) = value.as_str() {
                    if !choices.iter().any(|c| c.value == selected) {
                        return Err(CustomizerError::InvalidValue {
                            control: control.id.clone(),
                            message: "Invalid selection".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn generate_css_rules(&self, control: &CustomizerControl, value: &serde_json::Value) -> Vec<CssRule> {
        let mut rules = Vec::new();

        if let (Some(selector), Some(property)) = (&control.css_selector, &control.css_property) {
            let css_value = match &control.control_type {
                ControlType::Color { .. } => value.as_str().map(|s| s.to_string()),
                ControlType::Range { .. } | ControlType::Number { .. } => {
                    value.as_f64().map(|n| format!("{}px", n))
                }
                _ => value.as_str().map(|s| s.to_string()),
            };

            if let Some(css_value) = css_value {
                rules.push(CssRule {
                    selector: selector.clone(),
                    property: property.clone(),
                    value: css_value,
                });
            }
        }

        rules
    }

    /// Get the current changeset for a session
    pub fn get_changeset(&self, session_id: &str) -> Result<HashMap<String, serde_json::Value>, CustomizerError> {
        self.sessions
            .read()
            .get(session_id)
            .map(|s| s.changeset.clone())
            .ok_or_else(|| CustomizerError::SessionNotFound(session_id.to_string()))
    }

    /// Publish the changeset
    pub async fn publish(&self, session_id: &str) -> Result<(), CustomizerError> {
        let mut sessions = self.sessions.write();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| CustomizerError::SessionNotFound(session_id.to_string()))?;

        session.status = SessionStatus::Published;

        // The actual saving would be handled by the theme settings system
        Ok(())
    }

    /// Schedule the changeset for future publication
    pub fn schedule(
        &self,
        session_id: &str,
        publish_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), CustomizerError> {
        let mut sessions = self.sessions.write();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| CustomizerError::SessionNotFound(session_id.to_string()))?;

        session.status = SessionStatus::Scheduled;
        session.expires_at = publish_at;

        Ok(())
    }

    /// Subscribe to changes for live preview
    pub fn subscribe(&self) -> broadcast::Receiver<CustomizerChange> {
        self.change_tx.subscribe()
    }

    /// Generate preview CSS for a session
    pub fn generate_preview_css(&self, session_id: &str) -> Result<String, CustomizerError> {
        let sessions = self.sessions.read();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| CustomizerError::SessionNotFound(session_id.to_string()))?;

        let controls = self.controls.read();
        let mut css_parts: Vec<String> = Vec::new();
        let mut root_vars: Vec<String> = Vec::new();

        for (control_id, value) in &session.changeset {
            if let Some(control) = controls.get(control_id) {
                let rules = self.generate_css_rules(control, value);

                for rule in rules {
                    if rule.selector == ":root" {
                        root_vars.push(format!("  {}: {};", rule.property, rule.value));
                    } else {
                        css_parts.push(format!("{} {{ {}: {}; }}", rule.selector, rule.property, rule.value));
                    }
                }
            }
        }

        let mut css = String::new();

        if !root_vars.is_empty() {
            css.push_str(":root {\n");
            css.push_str(&root_vars.join("\n"));
            css.push_str("\n}\n\n");
        }

        css.push_str(&css_parts.join("\n"));

        Ok(css)
    }

    /// Discard session
    pub fn discard_session(&self, session_id: &str) {
        self.sessions.write().remove(session_id);
    }

    /// Clean expired sessions
    pub fn clean_expired_sessions(&self) {
        let now = chrono::Utc::now();
        self.sessions.write().retain(|_, s| {
            s.status == SessionStatus::Scheduled || s.expires_at > now
        });
    }
}

impl Default for ThemeCustomizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a color string is valid
fn is_valid_color(color: &str) -> bool {
    // Check hex colors
    if color.starts_with('#') {
        let hex = &color[1..];
        return (hex.len() == 3 || hex.len() == 6 || hex.len() == 8)
            && hex.chars().all(|c| c.is_ascii_hexdigit());
    }

    // Check rgb/rgba
    if color.starts_with("rgb") {
        return color.contains('(') && color.contains(')');
    }

    // Check hsl/hsla
    if color.starts_with("hsl") {
        return color.contains('(') && color.contains(')');
    }

    // Check named colors (basic set)
    let named_colors = [
        "black", "white", "red", "green", "blue", "yellow", "cyan", "magenta",
        "gray", "grey", "silver", "maroon", "olive", "navy", "purple", "teal",
        "aqua", "fuchsia", "lime", "orange", "pink", "transparent", "inherit",
        "currentColor",
    ];

    named_colors.contains(&color.to_lowercase().as_str())
}

/// Customizer panel for grouping sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizerPanel {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub capability: String,
    pub theme_supports: Option<String>,
}

/// Live preview JavaScript generator
pub struct PreviewScriptGenerator {
    controls: Vec<CustomizerControl>,
}

impl PreviewScriptGenerator {
    pub fn new(controls: Vec<CustomizerControl>) -> Self {
        Self { controls }
    }

    /// Generate JavaScript for live preview
    pub fn generate(&self) -> String {
        let mut scripts = Vec::new();

        scripts.push("(function($) {".to_string());
        scripts.push("  'use strict';".to_string());
        scripts.push("".to_string());

        for control in &self.controls {
            if control.transport == Transport::PostMessage {
                if let Some(script) = self.generate_control_script(control) {
                    scripts.push(script);
                }
            }
        }

        scripts.push("})(jQuery);".to_string());

        scripts.join("\n")
    }

    fn generate_control_script(&self, control: &CustomizerControl) -> Option<String> {
        let selector = control.css_selector.as_ref()?;
        let property = control.css_property.as_ref()?;

        Some(format!(
            r#"
  wp.customize('{}', function(value) {{
    value.bind(function(newval) {{
      if ('{}' === ':root') {{
        document.documentElement.style.setProperty('{}', newval);
      }} else {{
        $('{}').css('{}', newval);
      }}
    }});
  }});
"#,
            control.id,
            selector,
            property,
            selector,
            property.replace("--", "").replace("-", "")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_validation() {
        assert!(is_valid_color("#fff"));
        assert!(is_valid_color("#ffffff"));
        assert!(is_valid_color("#ffffffaa"));
        assert!(is_valid_color("rgb(255, 255, 255)"));
        assert!(is_valid_color("rgba(255, 255, 255, 0.5)"));
        assert!(is_valid_color("hsl(0, 100%, 50%)"));
        assert!(is_valid_color("red"));
        assert!(is_valid_color("transparent"));
        assert!(!is_valid_color("notacolor"));
        assert!(!is_valid_color("#gg"));
    }

    #[test]
    fn test_customizer_session() {
        let customizer = ThemeCustomizer::new();

        let session_id = customizer.start_session("test-theme", "user-1");
        assert!(!session_id.is_empty());

        let changeset = customizer.get_changeset(&session_id).unwrap();
        assert!(changeset.is_empty());
    }

    #[test]
    fn test_add_section_and_control() {
        let customizer = ThemeCustomizer::new();

        customizer.add_section(CustomizerSection {
            id: "test".to_string(),
            title: "Test Section".to_string(),
            description: None,
            priority: 10,
            panel: None,
            capability: "edit_theme_options".to_string(),
            theme_supports: None,
            active_callback: None,
        });

        customizer.add_control(CustomizerControl {
            id: "test_control".to_string(),
            section: "test".to_string(),
            label: "Test Control".to_string(),
            description: None,
            control_type: ControlType::Text {
                placeholder: None,
                input_attrs: HashMap::new(),
            },
            priority: 10,
            capability: "edit_theme_options".to_string(),
            default: None,
            transport: Transport::PostMessage,
            sanitize_callback: None,
            validate_callback: None,
            active_callback: None,
            css_selector: None,
            css_property: None,
        });

        let sections = customizer.get_sections();
        assert_eq!(sections.len(), 1);

        let controls = customizer.get_controls_for_section("test");
        assert_eq!(controls.len(), 1);
    }
}
