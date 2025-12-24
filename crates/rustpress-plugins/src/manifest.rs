//! Plugin Manifest Format (TOML-based)
//!
//! Defines the structure for plugin.toml manifest files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manifest - parsed from plugin.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMeta,

    /// Author information
    #[serde(default)]
    pub author: AuthorInfo,

    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: DependencySection,

    /// WordPress compatibility
    #[serde(default)]
    pub wordpress: WordPressCompat,

    /// Plugin permissions/capabilities required
    #[serde(default)]
    pub permissions: Vec<String>,

    /// Plugin hooks
    #[serde(default)]
    pub hooks: HooksSection,

    /// Plugin settings schema
    #[serde(default)]
    pub settings: SettingsSection,

    /// Database migrations
    #[serde(default)]
    pub migrations: MigrationsSection,

    /// Plugin assets
    #[serde(default)]
    pub assets: AssetsSection,

    /// API endpoints
    #[serde(default)]
    pub api: ApiSection,

    /// Admin pages
    #[serde(default)]
    pub admin: AdminSection,

    /// Shortcodes
    #[serde(default)]
    pub shortcodes: Vec<ShortcodeDefinition>,

    /// Blocks (Gutenberg-style)
    #[serde(default)]
    pub blocks: Vec<BlockDefinition>,

    /// Widgets
    #[serde(default)]
    pub widgets: Vec<WidgetDefinition>,

    /// CLI commands
    #[serde(default)]
    pub cli: Vec<CliCommandDefinition>,

    /// Cron jobs
    #[serde(default)]
    pub cron: Vec<CronJobDefinition>,

    /// WebAssembly configuration
    #[serde(default)]
    pub wasm: WasmSection,

    /// Feature flags
    #[serde(default)]
    pub features: HashMap<String, FeatureDefinition>,

    /// Network/multisite configuration
    #[serde(default)]
    pub network: NetworkSection,

    /// Code signing
    #[serde(default)]
    pub signing: SigningSection,
}

/// Core plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Unique plugin identifier (slug)
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Short description
    #[serde(default)]
    pub description: String,

    /// Long description (markdown)
    #[serde(default)]
    pub readme: Option<String>,

    /// Plugin homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// License identifier (SPDX)
    #[serde(default = "default_license")]
    pub license: String,

    /// Plugin icon path
    #[serde(default)]
    pub icon: Option<String>,

    /// Plugin banner path
    #[serde(default)]
    pub banner: Option<String>,

    /// Plugin screenshots
    #[serde(default)]
    pub screenshots: Vec<Screenshot>,

    /// Plugin category
    #[serde(default)]
    pub category: Option<PluginCategory>,

    /// Plugin tags for discoverability
    #[serde(default)]
    pub tags: Vec<String>,

    /// Minimum RustPress version required
    #[serde(default)]
    pub min_rustpress_version: Option<String>,

    /// Maximum RustPress version supported
    #[serde(default)]
    pub max_rustpress_version: Option<String>,

    /// Whether this is a must-use plugin
    #[serde(default)]
    pub must_use: bool,

    /// Plugin type
    #[serde(default)]
    pub plugin_type: PluginType,

    /// Plugin entry point
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_license() -> String {
    "MIT".to_string()
}

fn default_entry() -> String {
    "plugin.wasm".to_string()
}

/// Plugin type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// WebAssembly plugin (sandboxed)
    #[default]
    Wasm,
    /// Native Rust plugin (dynamic library)
    Native,
    /// Script plugin (interpreted)
    Script,
}

/// Plugin category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginCategory {
    Analytics,
    Backup,
    Caching,
    Commerce,
    Communication,
    ContentManagement,
    CustomPostTypes,
    Database,
    Development,
    Editor,
    Email,
    Forms,
    Gallery,
    Integrations,
    Localization,
    Marketing,
    Media,
    Membership,
    Navigation,
    Performance,
    Security,
    Seo,
    Social,
    Themes,
    Utilities,
    Widgets,
    Other,
}

/// Screenshot definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub path: String,
    #[serde(default)]
    pub caption: Option<String>,
}

/// Author information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorInfo {
    /// Author name
    #[serde(default)]
    pub name: String,

    /// Author email
    #[serde(default)]
    pub email: Option<String>,

    /// Author website
    #[serde(default)]
    pub url: Option<String>,

    /// Support URL
    #[serde(default)]
    pub support_url: Option<String>,

    /// Donation URL
    #[serde(default)]
    pub donate_url: Option<String>,
}

/// Dependency section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencySection {
    /// Required plugins with version constraints
    #[serde(default)]
    pub plugins: HashMap<String, DependencySpec>,

    /// PHP extensions (for WordPress compat)
    #[serde(default)]
    pub php_extensions: Vec<String>,

    /// System requirements
    #[serde(default)]
    pub system: SystemRequirements,

    /// Conflicting plugins
    #[serde(default)]
    pub conflicts: Vec<String>,

    /// Plugins that this replaces
    #[serde(default)]
    pub replaces: Vec<String>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version constraint
    Version(String),
    /// Detailed dependency
    Detailed(DetailedDependency),
}

/// Detailed dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    pub version: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

/// System requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Minimum memory in MB
    #[serde(default)]
    pub min_memory: Option<u64>,

    /// Required disk space in MB
    #[serde(default)]
    pub disk_space: Option<u64>,

    /// Required features
    #[serde(default)]
    pub features: Vec<String>,
}

/// WordPress compatibility layer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WordPressCompat {
    /// Whether to enable WordPress compatibility mode
    #[serde(default)]
    pub enabled: bool,

    /// Tested WordPress version
    #[serde(default)]
    pub tested_up_to: Option<String>,

    /// WordPress hooks to expose
    #[serde(default)]
    pub hooks: Vec<String>,

    /// WordPress filters to expose
    #[serde(default)]
    pub filters: Vec<String>,
}

/// Hooks section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksSection {
    /// Activation hook
    #[serde(default)]
    pub activate: Option<String>,

    /// Deactivation hook
    #[serde(default)]
    pub deactivate: Option<String>,

    /// Uninstall hook
    #[serde(default)]
    pub uninstall: Option<String>,

    /// Upgrade hook
    #[serde(default)]
    pub upgrade: Option<String>,

    /// Init hook (runs on every request)
    #[serde(default)]
    pub init: Option<String>,

    /// Custom action hooks to register
    #[serde(default)]
    pub actions: Vec<ActionHook>,

    /// Custom filter hooks to register
    #[serde(default)]
    pub filters: Vec<FilterHook>,
}

/// Action hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHook {
    pub hook: String,
    pub callback: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 {
    10
}

/// Filter hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterHook {
    pub hook: String,
    pub callback: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

/// Settings section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsSection {
    /// Settings schema
    #[serde(default)]
    pub schema: HashMap<String, SettingDefinition>,

    /// Settings groups
    #[serde(default)]
    pub groups: Vec<SettingsGroup>,

    /// Default values
    #[serde(default)]
    pub defaults: HashMap<String, serde_json::Value>,
}

/// Setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    pub setting_type: SettingType,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub validation: Option<ValidationRule>,
}

/// Setting type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingType {
    String,
    Text,
    Number,
    Integer,
    Boolean,
    Select,
    MultiSelect,
    Radio,
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
    Code,
}

/// Settings group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsGroup {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    pub settings: Vec<String>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    #[serde(default)]
    pub min: Option<serde_json::Value>,
    #[serde(default)]
    pub max: Option<serde_json::Value>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<SelectOption>>,
}

/// Select option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Migrations section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationsSection {
    /// Migrations directory
    #[serde(default = "default_migrations_dir")]
    pub directory: String,

    /// Migration files
    #[serde(default)]
    pub files: Vec<MigrationFile>,

    /// Auto-run migrations on activate
    #[serde(default = "default_true")]
    pub auto_run: bool,
}

fn default_migrations_dir() -> String {
    "migrations".to_string()
}

fn default_true() -> bool {
    true
}

/// Migration file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFile {
    pub version: String,
    pub file: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Assets section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetsSection {
    /// CSS files
    #[serde(default)]
    pub css: Vec<AssetFile>,

    /// JavaScript files
    #[serde(default)]
    pub js: Vec<AssetFile>,

    /// Static files directory
    #[serde(default)]
    pub static_dir: Option<String>,

    /// Build configuration
    #[serde(default)]
    pub build: Option<AssetBuild>,
}

/// Asset file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFile {
    pub path: String,
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub location: AssetLocation,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub admin_only: bool,
    #[serde(default)]
    pub frontend_only: bool,
}

/// Asset location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetLocation {
    #[default]
    Header,
    Footer,
}

/// Asset build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBuild {
    pub command: String,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub watch_patterns: Vec<String>,
}

/// API section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiSection {
    /// API namespace
    #[serde(default)]
    pub namespace: Option<String>,

    /// API version
    #[serde(default = "default_api_version")]
    pub version: String,

    /// API endpoints
    #[serde(default)]
    pub endpoints: Vec<ApiEndpoint>,
}

fn default_api_version() -> String {
    "v1".to_string()
}

/// API endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub handler: String,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub rate_limit: Option<RateLimit>,
    #[serde(default)]
    pub description: Option<String>,
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests: u32,
    pub window_seconds: u32,
}

/// Admin section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminSection {
    /// Admin menu items
    #[serde(default)]
    pub menu: Vec<AdminMenuItem>,

    /// Admin pages
    #[serde(default)]
    pub pages: Vec<AdminPage>,

    /// Dashboard widgets
    #[serde(default)]
    pub dashboard_widgets: Vec<DashboardWidget>,

    /// Settings page
    #[serde(default)]
    pub settings_page: Option<SettingsPage>,
}

/// Admin menu item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMenuItem {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    pub page: String,
}

/// Admin page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPage {
    pub id: String,
    pub title: String,
    pub handler: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub title: String,
    pub handler: String,
    #[serde(default)]
    pub position: WidgetPosition,
}

/// Widget position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WidgetPosition {
    #[default]
    Normal,
    Side,
    Column3,
    Column4,
}

/// Settings page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPage {
    pub title: String,
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

/// Shortcode definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcodeDefinition {
    pub tag: String,
    pub handler: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: Vec<ShortcodeAttribute>,
    #[serde(default)]
    pub supports_content: bool,
}

/// Shortcode attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcodeAttribute {
    pub name: String,
    #[serde(default)]
    pub attr_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Block definition (Gutenberg-style)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDefinition {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, BlockAttribute>,
    pub render: String,
    #[serde(default)]
    pub editor_script: Option<String>,
    #[serde(default)]
    pub editor_style: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub supports: BlockSupports,
}

/// Block attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockAttribute {
    pub attr_type: String,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

/// Block supports
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockSupports {
    #[serde(default)]
    pub align: bool,
    #[serde(default)]
    pub anchor: bool,
    #[serde(default)]
    pub custom_class_name: bool,
    #[serde(default)]
    pub color: bool,
    #[serde(default)]
    pub typography: bool,
    #[serde(default)]
    pub spacing: bool,
}

/// Widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub render: String,
    #[serde(default)]
    pub form: Option<String>,
    #[serde(default)]
    pub settings: HashMap<String, SettingDefinition>,
}

/// CLI command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandDefinition {
    pub name: String,
    pub handler: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub arguments: Vec<CliArgument>,
    #[serde(default)]
    pub options: Vec<CliOption>,
    #[serde(default)]
    pub subcommands: Vec<CliSubcommand>,
}

/// CLI argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliArgument {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

/// CLI option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOption {
    pub name: String,
    #[serde(default)]
    pub short: Option<char>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub takes_value: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// CLI subcommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSubcommand {
    pub name: String,
    pub handler: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Cron job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobDefinition {
    pub name: String,
    pub handler: String,
    pub schedule: CronSchedule,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

/// Cron schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CronSchedule {
    /// Cron expression (e.g., "0 * * * *")
    Cron(String),
    /// Predefined interval
    Interval(CronInterval),
}

/// Predefined cron interval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CronInterval {
    Hourly,
    TwiceDaily,
    Daily,
    Weekly,
}

/// WebAssembly configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasmSection {
    /// Memory limit in MB
    #[serde(default = "default_wasm_memory")]
    pub memory_limit: u64,

    /// CPU time limit in milliseconds
    #[serde(default = "default_wasm_timeout")]
    pub timeout_ms: u64,

    /// Allowed imports
    #[serde(default)]
    pub imports: Vec<String>,

    /// WASI capabilities
    #[serde(default)]
    pub wasi: WasiConfig,

    /// Fuel limit (instruction count)
    #[serde(default)]
    pub fuel_limit: Option<u64>,
}

fn default_wasm_memory() -> u64 {
    64
}

fn default_wasm_timeout() -> u64 {
    5000
}

/// WASI configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasiConfig {
    /// Allow file system access
    #[serde(default)]
    pub fs_read: Vec<String>,
    #[serde(default)]
    pub fs_write: Vec<String>,

    /// Allow network access
    #[serde(default)]
    pub network: bool,

    /// Environment variables to pass
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Inherit environment
    #[serde(default)]
    pub inherit_env: bool,
}

/// Feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub description: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rollout_percentage: Option<u8>,
    #[serde(default)]
    pub conditions: Vec<FeatureCondition>,
}

/// Feature condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCondition {
    pub condition_type: ConditionType,
    pub value: serde_json::Value,
}

/// Condition type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    UserRole,
    UserId,
    UserMeta,
    PostType,
    Environment,
    Custom,
}

/// Network/multisite section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkSection {
    /// Network-wide activation support
    #[serde(default)]
    pub network_wide: bool,

    /// Per-site activation
    #[serde(default)]
    pub per_site: bool,

    /// Network admin menu
    #[serde(default)]
    pub network_menu: bool,

    /// Shared tables across sites
    #[serde(default)]
    pub shared_tables: Vec<String>,
}

/// Code signing section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SigningSection {
    /// Whether signing is required
    #[serde(default)]
    pub required: bool,

    /// Signature algorithm
    #[serde(default)]
    pub algorithm: Option<String>,

    /// Public key for verification
    #[serde(default)]
    pub public_key: Option<String>,

    /// Signature (base64 encoded)
    #[serde(default)]
    pub signature: Option<String>,

    /// Files included in signature
    #[serde(default)]
    pub signed_files: Vec<String>,
}

impl PluginManifest {
    /// Parse a manifest from TOML string
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Parse a manifest from a file
    pub fn from_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::Io(e.to_string()))?;
        Self::from_toml(&content)
            .map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate plugin ID
        if self.plugin.id.is_empty() {
            errors.push(ValidationError::new("plugin.id", "Plugin ID is required"));
        } else if !is_valid_slug(&self.plugin.id) {
            errors.push(ValidationError::new(
                "plugin.id",
                "Plugin ID must be lowercase alphanumeric with hyphens",
            ));
        }

        // Validate name
        if self.plugin.name.is_empty() {
            errors.push(ValidationError::new("plugin.name", "Plugin name is required"));
        }

        // Validate version
        if semver::Version::parse(&self.plugin.version).is_err() {
            errors.push(ValidationError::new(
                "plugin.version",
                "Invalid semver version",
            ));
        }

        // Validate dependencies
        for (dep_id, spec) in &self.dependencies.plugins {
            if !is_valid_slug(dep_id) {
                errors.push(ValidationError::new(
                    &format!("dependencies.plugins.{}", dep_id),
                    "Invalid dependency ID",
                ));
            }

            let version_str = match spec {
                DependencySpec::Version(v) => v,
                DependencySpec::Detailed(d) => &d.version,
            };

            if semver::VersionReq::parse(version_str).is_err() {
                errors.push(ValidationError::new(
                    &format!("dependencies.plugins.{}", dep_id),
                    "Invalid version requirement",
                ));
            }
        }

        // Validate API endpoints
        for (i, endpoint) in self.api.endpoints.iter().enumerate() {
            if endpoint.path.is_empty() {
                errors.push(ValidationError::new(
                    &format!("api.endpoints[{}].path", i),
                    "Endpoint path is required",
                ));
            }
            if endpoint.handler.is_empty() {
                errors.push(ValidationError::new(
                    &format!("api.endpoints[{}].handler", i),
                    "Endpoint handler is required",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the full API namespace
    pub fn api_namespace(&self) -> String {
        self.api.namespace.clone().unwrap_or_else(|| self.plugin.id.clone())
    }

    /// Check if the plugin has any database migrations
    pub fn has_migrations(&self) -> bool {
        !self.migrations.files.is_empty()
    }

    /// Check if the plugin requires network activation
    pub fn is_network_only(&self) -> bool {
        self.network.network_wide && !self.network.per_site
    }
}

fn is_valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// Manifest parsing/validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

/// Manifest error
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation errors")]
    Validation(Vec<ValidationError>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[plugin]
id = "my-plugin"
name = "My Plugin"
version = "1.0.0"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.plugin.id, "my-plugin");
        assert_eq!(manifest.plugin.name, "My Plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
    }

    #[test]
    fn test_parse_full_manifest() {
        let toml = r#"
[plugin]
id = "advanced-seo"
name = "Advanced SEO"
version = "2.0.0"
description = "Complete SEO solution"
license = "GPL-3.0"
category = "seo"
tags = ["seo", "meta", "sitemap"]

[author]
name = "Plugin Author"
email = "author@example.com"
url = "https://example.com"

[dependencies.plugins]
"some-dependency" = "^1.0"

[settings.schema.enable_sitemap]
setting_type = "boolean"
label = "Enable Sitemap"
default = true

[[api.endpoints]]
path = "/analyze"
method = "POST"
handler = "analyze_content"
permission = "edit_posts"

[[shortcodes]]
tag = "seo-score"
handler = "render_seo_score"
description = "Display SEO score"

[[cron]]
name = "sitemap_update"
handler = "update_sitemap"
schedule = "daily"
"#;

        let manifest = PluginManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.plugin.id, "advanced-seo");
        assert_eq!(manifest.dependencies.plugins.len(), 1);
        assert_eq!(manifest.api.endpoints.len(), 1);
        assert_eq!(manifest.shortcodes.len(), 1);
        assert_eq!(manifest.cron.len(), 1);
    }

    #[test]
    fn test_validate_manifest() {
        let mut manifest = PluginManifest::from_toml(
            r#"
[plugin]
id = "valid-plugin"
name = "Valid Plugin"
version = "1.0.0"
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_ok());

        manifest.plugin.id = "".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_slug_validation() {
        assert!(is_valid_slug("my-plugin"));
        assert!(is_valid_slug("plugin123"));
        assert!(!is_valid_slug("My-Plugin"));
        assert!(!is_valid_slug("-plugin"));
        assert!(!is_valid_slug("plugin-"));
        assert!(!is_valid_slug(""));
    }
}
