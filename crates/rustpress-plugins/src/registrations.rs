//! Plugin Registration Types
//!
//! Handles registration for shortcodes, blocks, widgets, CLI commands,
//! cron jobs, and capabilities.

use crate::manifest::{
    BlockDefinition, CliCommandDefinition, CronJobDefinition, CronSchedule,
    ShortcodeDefinition, WidgetDefinition,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::debug;

// ============================================================================
// Shortcode Registration (Point 171)
// ============================================================================

/// Shortcode registry
pub struct ShortcodeRegistry {
    shortcodes: Arc<RwLock<HashMap<String, RegisteredShortcode>>>,
}

/// Registered shortcode
#[derive(Debug, Clone)]
pub struct RegisteredShortcode {
    pub plugin_id: String,
    pub tag: String,
    pub handler: String,
    pub description: Option<String>,
    pub attributes: Vec<ShortcodeAttribute>,
    pub supports_content: bool,
}

/// Shortcode attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcodeAttribute {
    pub name: String,
    pub attr_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub description: Option<String>,
}

impl ShortcodeRegistry {
    pub fn new() -> Self {
        Self {
            shortcodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register shortcode from manifest
    pub fn register_from_manifest(&self, plugin_id: &str, shortcode: &ShortcodeDefinition) {
        let registered = RegisteredShortcode {
            plugin_id: plugin_id.to_string(),
            tag: shortcode.tag.clone(),
            handler: shortcode.handler.clone(),
            description: shortcode.description.clone(),
            attributes: shortcode
                .attributes
                .iter()
                .map(|a| ShortcodeAttribute {
                    name: a.name.clone(),
                    attr_type: a.attr_type.clone(),
                    required: a.required,
                    default: a.default.clone(),
                    description: a.description.clone(),
                })
                .collect(),
            supports_content: shortcode.supports_content,
        };

        self.shortcodes.write().insert(shortcode.tag.clone(), registered);
    }

    /// Register shortcode programmatically
    pub fn register(&self, plugin_id: &str, tag: &str, handler: &str) {
        let shortcode = RegisteredShortcode {
            plugin_id: plugin_id.to_string(),
            tag: tag.to_string(),
            handler: handler.to_string(),
            description: None,
            attributes: Vec::new(),
            supports_content: false,
        };
        self.shortcodes.write().insert(tag.to_string(), shortcode);
    }

    /// Get shortcode by tag
    pub fn get(&self, tag: &str) -> Option<RegisteredShortcode> {
        self.shortcodes.read().get(tag).cloned()
    }

    /// Get all shortcodes
    pub fn get_all(&self) -> Vec<RegisteredShortcode> {
        self.shortcodes.read().values().cloned().collect()
    }

    /// Check if shortcode exists
    pub fn exists(&self, tag: &str) -> bool {
        self.shortcodes.read().contains_key(tag)
    }

    /// Unregister shortcode
    pub fn unregister(&self, tag: &str) {
        self.shortcodes.write().remove(tag);
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.shortcodes.write().retain(|_, s| s.plugin_id != plugin_id);
    }

    /// Parse shortcode from content
    pub fn parse(&self, content: &str) -> Vec<ParsedShortcode> {
        let mut results = Vec::new();
        let re = regex::Regex::new(r"\[(\w+)([^\]]*)\](?:(.*?)\[/\1\])?").unwrap();

        for cap in re.captures_iter(content) {
            let tag = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if self.exists(tag) {
                results.push(ParsedShortcode {
                    tag: tag.to_string(),
                    attributes: Self::parse_attributes(cap.get(2).map(|m| m.as_str()).unwrap_or("")),
                    content: cap.get(3).map(|m| m.as_str().to_string()),
                    full_match: cap.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
                });
            }
        }

        results
    }

    fn parse_attributes(attr_str: &str) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        let re = regex::Regex::new(r#"(\w+)=["']([^"']*)["']|(\w+)=(\S+)"#).unwrap();

        for cap in re.captures_iter(attr_str) {
            if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                attrs.insert(key.as_str().to_string(), value.as_str().to_string());
            } else if let (Some(key), Some(value)) = (cap.get(3), cap.get(4)) {
                attrs.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }

        attrs
    }
}

impl Default for ShortcodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed shortcode
#[derive(Debug, Clone)]
pub struct ParsedShortcode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub content: Option<String>,
    pub full_match: String,
}

// ============================================================================
// Block Registration (Point 172)
// ============================================================================

/// Block registry (Gutenberg-style)
pub struct BlockRegistry {
    blocks: Arc<RwLock<HashMap<String, RegisteredBlock>>>,
}

/// Registered block
#[derive(Debug, Clone)]
pub struct RegisteredBlock {
    pub plugin_id: String,
    pub name: String,
    pub title: String,
    pub category: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub attributes: HashMap<String, BlockAttribute>,
    pub render_handler: String,
    pub editor_script: Option<String>,
    pub editor_style: Option<String>,
    pub style: Option<String>,
    pub supports: BlockSupports,
}

/// Block attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAttribute {
    pub attr_type: String,
    pub default: Option<serde_json::Value>,
}

/// Block supports
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockSupports {
    pub align: bool,
    pub anchor: bool,
    pub custom_class_name: bool,
    pub color: bool,
    pub typography: bool,
    pub spacing: bool,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register block from manifest
    pub fn register_from_manifest(&self, plugin_id: &str, block: &BlockDefinition) {
        let registered = RegisteredBlock {
            plugin_id: plugin_id.to_string(),
            name: format!("{}/{}", plugin_id, block.name),
            title: block.title.clone(),
            category: block.category.clone(),
            icon: block.icon.clone(),
            description: block.description.clone(),
            keywords: block.keywords.clone(),
            attributes: block
                .attributes
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        BlockAttribute {
                            attr_type: v.attr_type.clone(),
                            default: v.default.clone(),
                        },
                    )
                })
                .collect(),
            render_handler: block.render.clone(),
            editor_script: block.editor_script.clone(),
            editor_style: block.editor_style.clone(),
            style: block.style.clone(),
            supports: BlockSupports {
                align: block.supports.align,
                anchor: block.supports.anchor,
                custom_class_name: block.supports.custom_class_name,
                color: block.supports.color,
                typography: block.supports.typography,
                spacing: block.supports.spacing,
            },
        };

        self.blocks.write().insert(registered.name.clone(), registered);
    }

    /// Get block by name
    pub fn get(&self, name: &str) -> Option<RegisteredBlock> {
        self.blocks.read().get(name).cloned()
    }

    /// Get all blocks
    pub fn get_all(&self) -> Vec<RegisteredBlock> {
        self.blocks.read().values().cloned().collect()
    }

    /// Get blocks by category
    pub fn get_by_category(&self, category: &str) -> Vec<RegisteredBlock> {
        self.blocks
            .read()
            .values()
            .filter(|b| b.category == category)
            .cloned()
            .collect()
    }

    /// Unregister block
    pub fn unregister(&self, name: &str) {
        self.blocks.write().remove(name);
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.blocks.write().retain(|_, b| b.plugin_id != plugin_id);
    }

    /// Get block categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .blocks
            .read()
            .values()
            .map(|b| b.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Widget Registration (Point 173)
// ============================================================================

/// Widget registry
pub struct WidgetRegistry {
    widgets: Arc<RwLock<HashMap<String, RegisteredWidget>>>,
}

/// Registered widget
#[derive(Debug, Clone)]
pub struct RegisteredWidget {
    pub plugin_id: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub render_handler: String,
    pub form_handler: Option<String>,
    pub settings: HashMap<String, WidgetSetting>,
}

/// Widget setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSetting {
    pub setting_type: String,
    pub label: String,
    pub default: Option<serde_json::Value>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            widgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register widget from manifest
    pub fn register_from_manifest(&self, plugin_id: &str, widget: &WidgetDefinition) {
        let registered = RegisteredWidget {
            plugin_id: plugin_id.to_string(),
            id: widget.id.clone(),
            name: widget.name.clone(),
            description: widget.description.clone(),
            render_handler: widget.render.clone(),
            form_handler: widget.form.clone(),
            settings: widget
                .settings
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        WidgetSetting {
                            setting_type: format!("{:?}", v.setting_type),
                            label: v.label.clone(),
                            default: v.default.clone(),
                        },
                    )
                })
                .collect(),
        };

        self.widgets.write().insert(widget.id.clone(), registered);
    }

    /// Get widget by ID
    pub fn get(&self, id: &str) -> Option<RegisteredWidget> {
        self.widgets.read().get(id).cloned()
    }

    /// Get all widgets
    pub fn get_all(&self) -> Vec<RegisteredWidget> {
        self.widgets.read().values().cloned().collect()
    }

    /// Unregister widget
    pub fn unregister(&self, id: &str) {
        self.widgets.write().remove(id);
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.widgets.write().retain(|_, w| w.plugin_id != plugin_id);
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CLI Command Registration (Point 174)
// ============================================================================

/// CLI command registry
pub struct CliRegistry {
    commands: Arc<RwLock<HashMap<String, RegisteredCliCommand>>>,
}

/// Registered CLI command
#[derive(Debug, Clone)]
pub struct RegisteredCliCommand {
    pub plugin_id: String,
    pub name: String,
    pub handler: String,
    pub description: Option<String>,
    pub arguments: Vec<CliArgument>,
    pub options: Vec<CliOption>,
    pub subcommands: Vec<CliSubcommand>,
}

/// CLI argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// CLI option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOption {
    pub name: String,
    pub short: Option<char>,
    pub description: Option<String>,
    pub takes_value: bool,
    pub default: Option<String>,
}

/// CLI subcommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSubcommand {
    pub name: String,
    pub handler: String,
    pub description: Option<String>,
}

impl CliRegistry {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register CLI command from manifest
    pub fn register_from_manifest(&self, plugin_id: &str, command: &CliCommandDefinition) {
        let registered = RegisteredCliCommand {
            plugin_id: plugin_id.to_string(),
            name: command.name.clone(),
            handler: command.handler.clone(),
            description: command.description.clone(),
            arguments: command
                .arguments
                .iter()
                .map(|a| CliArgument {
                    name: a.name.clone(),
                    description: a.description.clone(),
                    required: a.required,
                })
                .collect(),
            options: command
                .options
                .iter()
                .map(|o| CliOption {
                    name: o.name.clone(),
                    short: o.short,
                    description: o.description.clone(),
                    takes_value: o.takes_value,
                    default: o.default.clone(),
                })
                .collect(),
            subcommands: command
                .subcommands
                .iter()
                .map(|s| CliSubcommand {
                    name: s.name.clone(),
                    handler: s.handler.clone(),
                    description: s.description.clone(),
                })
                .collect(),
        };

        self.commands.write().insert(command.name.clone(), registered);
    }

    /// Get command by name
    pub fn get(&self, name: &str) -> Option<RegisteredCliCommand> {
        self.commands.read().get(name).cloned()
    }

    /// Get all commands
    pub fn get_all(&self) -> Vec<RegisteredCliCommand> {
        self.commands.read().values().cloned().collect()
    }

    /// Unregister command
    pub fn unregister(&self, name: &str) {
        self.commands.write().remove(name);
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.commands.write().retain(|_, c| c.plugin_id != plugin_id);
    }
}

impl Default for CliRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Cron Job Registration (Point 175)
// ============================================================================

/// Cron registry
pub struct CronRegistry {
    jobs: Arc<RwLock<HashMap<String, RegisteredCronJob>>>,
}

/// Registered cron job
#[derive(Debug, Clone)]
pub struct RegisteredCronJob {
    pub plugin_id: String,
    pub name: String,
    pub handler: String,
    pub schedule: CronScheduleType,
    pub description: Option<String>,
    pub enabled: bool,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// Cron schedule type
#[derive(Debug, Clone)]
pub enum CronScheduleType {
    /// Cron expression (e.g., "0 * * * *")
    Cron(String),
    /// Predefined interval
    Interval(CronIntervalType),
}

/// Predefined cron interval
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronIntervalType {
    Hourly,
    TwiceDaily,
    Daily,
    Weekly,
}

impl CronIntervalType {
    pub fn to_seconds(&self) -> u64 {
        match self {
            CronIntervalType::Hourly => 3600,
            CronIntervalType::TwiceDaily => 43200,
            CronIntervalType::Daily => 86400,
            CronIntervalType::Weekly => 604800,
        }
    }
}

impl CronRegistry {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register cron job from manifest
    pub fn register_from_manifest(&self, plugin_id: &str, job: &CronJobDefinition) {
        let schedule = match &job.schedule {
            CronSchedule::Cron(expr) => CronScheduleType::Cron(expr.clone()),
            CronSchedule::Interval(interval) => {
                use crate::manifest::CronInterval;
                CronScheduleType::Interval(match interval {
                    CronInterval::Hourly => CronIntervalType::Hourly,
                    CronInterval::TwiceDaily => CronIntervalType::TwiceDaily,
                    CronInterval::Daily => CronIntervalType::Daily,
                    CronInterval::Weekly => CronIntervalType::Weekly,
                })
            }
        };

        let registered = RegisteredCronJob {
            plugin_id: plugin_id.to_string(),
            name: job.name.clone(),
            handler: job.handler.clone(),
            schedule,
            description: job.description.clone(),
            enabled: job.enabled,
            last_run: None,
            next_run: None,
        };

        self.jobs.write().insert(job.name.clone(), registered);
    }

    /// Get job by name
    pub fn get(&self, name: &str) -> Option<RegisteredCronJob> {
        self.jobs.read().get(name).cloned()
    }

    /// Get all jobs
    pub fn get_all(&self) -> Vec<RegisteredCronJob> {
        self.jobs.read().values().cloned().collect()
    }

    /// Get due jobs
    pub fn get_due_jobs(&self) -> Vec<RegisteredCronJob> {
        let now = chrono::Utc::now();
        self.jobs
            .read()
            .values()
            .filter(|j| j.enabled && j.next_run.map(|t| t <= now).unwrap_or(true))
            .cloned()
            .collect()
    }

    /// Update last run time
    pub fn update_last_run(&self, name: &str) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(name) {
            job.last_run = Some(chrono::Utc::now());
            job.next_run = Some(self.calculate_next_run(&job.schedule));
        }
    }

    fn calculate_next_run(&self, schedule: &CronScheduleType) -> chrono::DateTime<chrono::Utc> {
        let now = chrono::Utc::now();
        match schedule {
            CronScheduleType::Interval(interval) => {
                now + chrono::Duration::seconds(interval.to_seconds() as i64)
            }
            CronScheduleType::Cron(_expr) => {
                // In real implementation, would parse cron expression
                now + chrono::Duration::hours(1)
            }
        }
    }

    /// Enable/disable job
    pub fn set_enabled(&self, name: &str, enabled: bool) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(name) {
            job.enabled = enabled;
        }
    }

    /// Unregister job
    pub fn unregister(&self, name: &str) {
        self.jobs.write().remove(name);
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.jobs.write().retain(|_, j| j.plugin_id != plugin_id);
    }
}

impl Default for CronRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Capability Registration (Point 176)
// ============================================================================

/// Capability registry
pub struct CapabilityRegistry {
    capabilities: Arc<RwLock<HashMap<String, RegisteredCapability>>>,
    role_capabilities: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

/// Registered capability
#[derive(Debug, Clone)]
pub struct RegisteredCapability {
    pub plugin_id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_roles: Vec<String>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(RwLock::new(HashMap::new())),
            role_capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register capability
    pub fn register(&self, plugin_id: &str, name: &str, description: Option<&str>, default_roles: Vec<&str>) {
        let cap = RegisteredCapability {
            plugin_id: plugin_id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            default_roles: default_roles.iter().map(|s| s.to_string()).collect(),
        };

        self.capabilities.write().insert(name.to_string(), cap.clone());

        // Add to default roles
        let mut roles = self.role_capabilities.write();
        for role in &cap.default_roles {
            roles
                .entry(role.clone())
                .or_insert_with(Vec::new)
                .push(name.to_string());
        }
    }

    /// Check if capability exists
    pub fn exists(&self, name: &str) -> bool {
        self.capabilities.read().contains_key(name)
    }

    /// Get capability
    pub fn get(&self, name: &str) -> Option<RegisteredCapability> {
        self.capabilities.read().get(name).cloned()
    }

    /// Get all capabilities
    pub fn get_all(&self) -> Vec<RegisteredCapability> {
        self.capabilities.read().values().cloned().collect()
    }

    /// Get capabilities for role
    pub fn get_for_role(&self, role: &str) -> Vec<String> {
        self.role_capabilities
            .read()
            .get(role)
            .cloned()
            .unwrap_or_default()
    }

    /// Grant capability to role
    pub fn grant_to_role(&self, capability: &str, role: &str) {
        let mut roles = self.role_capabilities.write();
        let caps = roles.entry(role.to_string()).or_insert_with(Vec::new);
        if !caps.contains(&capability.to_string()) {
            caps.push(capability.to_string());
        }
    }

    /// Revoke capability from role
    pub fn revoke_from_role(&self, capability: &str, role: &str) {
        let mut roles = self.role_capabilities.write();
        if let Some(caps) = roles.get_mut(role) {
            caps.retain(|c| c != capability);
        }
    }

    /// Unregister capability
    pub fn unregister(&self, name: &str) {
        self.capabilities.write().remove(name);

        // Remove from all roles
        let mut roles = self.role_capabilities.write();
        for caps in roles.values_mut() {
            caps.retain(|c| c != name);
        }
    }

    /// Unregister all for plugin
    pub fn unregister_plugin(&self, plugin_id: &str) {
        let caps_to_remove: Vec<String> = self
            .capabilities
            .read()
            .iter()
            .filter(|(_, c)| c.plugin_id == plugin_id)
            .map(|(k, _)| k.clone())
            .collect();

        for cap in caps_to_remove {
            self.unregister(&cap);
        }
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcode_parsing() {
        let registry = ShortcodeRegistry::new();
        registry.register("test-plugin", "gallery", "render_gallery");

        let content = r#"[gallery id="123" columns="3"]Some content[/gallery]"#;
        let parsed = registry.parse(content);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].tag, "gallery");
        assert_eq!(parsed[0].attributes.get("id"), Some(&"123".to_string()));
        assert_eq!(parsed[0].content, Some("Some content".to_string()));
    }

    #[test]
    fn test_capability_registry() {
        let registry = CapabilityRegistry::new();
        registry.register("test-plugin", "manage_test", Some("Manage test"), vec!["administrator"]);

        assert!(registry.exists("manage_test"));
        assert!(registry.get_for_role("administrator").contains(&"manage_test".to_string()));
    }

    #[test]
    fn test_cron_interval() {
        assert_eq!(CronIntervalType::Hourly.to_seconds(), 3600);
        assert_eq!(CronIntervalType::Daily.to_seconds(), 86400);
    }
}
