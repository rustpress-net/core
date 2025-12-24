//! Plugin Lifecycle Hooks
//!
//! Manages plugin activation, deactivation, upgrade, and uninstallation.

use crate::manifest::PluginManifest;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};

/// Plugin lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginState {
    /// Plugin is discovered but not loaded
    Discovered,
    /// Plugin is loaded but not active
    Inactive,
    /// Plugin is being activated
    Activating,
    /// Plugin is active and running
    Active,
    /// Plugin is being deactivated
    Deactivating,
    /// Plugin is being upgraded
    Upgrading,
    /// Plugin is being uninstalled
    Uninstalling,
    /// Plugin encountered an error
    Error,
    /// Plugin is paused (temporarily disabled)
    Paused,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::Discovered => write!(f, "discovered"),
            PluginState::Inactive => write!(f, "inactive"),
            PluginState::Activating => write!(f, "activating"),
            PluginState::Active => write!(f, "active"),
            PluginState::Deactivating => write!(f, "deactivating"),
            PluginState::Upgrading => write!(f, "upgrading"),
            PluginState::Uninstalling => write!(f, "uninstalling"),
            PluginState::Error => write!(f, "error"),
            PluginState::Paused => write!(f, "paused"),
        }
    }
}

/// Lifecycle event type
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// Plugin discovered in filesystem
    Discovered { plugin_id: String },
    /// Plugin loaded into memory
    Loaded { plugin_id: String },
    /// Plugin activation started
    ActivationStarted { plugin_id: String },
    /// Plugin activated successfully
    Activated { plugin_id: String },
    /// Plugin activation failed
    ActivationFailed { plugin_id: String, error: String },
    /// Plugin deactivation started
    DeactivationStarted { plugin_id: String },
    /// Plugin deactivated successfully
    Deactivated { plugin_id: String },
    /// Plugin deactivation failed
    DeactivationFailed { plugin_id: String, error: String },
    /// Plugin upgrade started
    UpgradeStarted { plugin_id: String, from_version: String, to_version: String },
    /// Plugin upgraded successfully
    Upgraded { plugin_id: String, from_version: String, to_version: String },
    /// Plugin upgrade failed
    UpgradeFailed { plugin_id: String, error: String },
    /// Plugin uninstall started
    UninstallStarted { plugin_id: String },
    /// Plugin uninstalled successfully
    Uninstalled { plugin_id: String },
    /// Plugin uninstall failed
    UninstallFailed { plugin_id: String, error: String },
    /// Plugin paused
    Paused { plugin_id: String },
    /// Plugin resumed
    Resumed { plugin_id: String },
    /// Plugin error occurred
    Error { plugin_id: String, error: String },
}

/// Lifecycle hook trait for plugins
#[async_trait]
pub trait LifecycleHook: Send + Sync {
    /// Called when plugin is activated
    async fn on_activate(&self, context: &ActivationContext) -> Result<(), HookError>;

    /// Called when plugin is deactivated
    async fn on_deactivate(&self, context: &DeactivationContext) -> Result<(), HookError>;

    /// Called when plugin is upgraded
    async fn on_upgrade(&self, context: &UpgradeContext) -> Result<(), HookError> {
        let _ = context;
        Ok(())
    }

    /// Called when plugin is uninstalled
    async fn on_uninstall(&self, context: &UninstallContext) -> Result<(), HookError> {
        let _ = context;
        Ok(())
    }

    /// Called on each request (init hook)
    async fn on_init(&self, context: &InitContext) -> Result<(), HookError> {
        let _ = context;
        Ok(())
    }

    /// Called when plugin is loaded
    async fn on_load(&self, context: &LoadContext) -> Result<(), HookError> {
        let _ = context;
        Ok(())
    }

    /// Called before shutdown
    async fn on_shutdown(&self, context: &ShutdownContext) -> Result<(), HookError> {
        let _ = context;
        Ok(())
    }
}

/// Context for activation hook
#[derive(Debug, Clone)]
pub struct ActivationContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Is network activation (multisite)
    pub network_wide: bool,
    /// Site ID (for multisite)
    pub site_id: Option<i64>,
    /// Is fresh install (vs reactivation)
    pub fresh_install: bool,
}

/// Context for deactivation hook
#[derive(Debug, Clone)]
pub struct DeactivationContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Is network deactivation
    pub network_wide: bool,
    /// Site ID
    pub site_id: Option<i64>,
    /// Reason for deactivation
    pub reason: DeactivationReason,
}

/// Reason for deactivation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeactivationReason {
    /// User manually deactivated
    UserRequested,
    /// Deactivated due to dependency issue
    DependencyIssue,
    /// Deactivated due to conflict
    Conflict,
    /// Deactivated due to error
    Error,
    /// Deactivated for upgrade
    Upgrade,
    /// Deactivated for uninstall
    Uninstall,
    /// System maintenance
    Maintenance,
}

/// Context for upgrade hook
#[derive(Debug, Clone)]
pub struct UpgradeContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Previous version
    pub from_version: String,
    /// New version
    pub to_version: String,
    /// Old manifest
    pub old_manifest: PluginManifest,
    /// New manifest
    pub new_manifest: PluginManifest,
    /// Is network upgrade
    pub network_wide: bool,
}

/// Context for uninstall hook
#[derive(Debug, Clone)]
pub struct UninstallContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Delete all data
    pub delete_data: bool,
    /// Is network uninstall
    pub network_wide: bool,
}

/// Context for init hook (runs on every request)
#[derive(Debug, Clone)]
pub struct InitContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Current request path (if any)
    pub request_path: Option<String>,
    /// Is admin request
    pub is_admin: bool,
    /// Is REST API request
    pub is_api: bool,
    /// Is cron request
    pub is_cron: bool,
}

/// Context for load hook
#[derive(Debug, Clone)]
pub struct LoadContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin manifest
    pub manifest: PluginManifest,
}

/// Context for shutdown hook
#[derive(Debug, Clone)]
pub struct ShutdownContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Reason for shutdown
    pub reason: ShutdownReason,
}

/// Reason for shutdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    /// Normal shutdown
    Normal,
    /// Timeout
    Timeout,
    /// Error
    Error,
    /// System restart
    Restart,
}

/// Hook error
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Hook execution failed: {0}")]
    Execution(String),

    #[error("Hook timeout")]
    Timeout,

    #[error("Hook cancelled")]
    Cancelled,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    #[error("Conflict with plugin: {0}")]
    Conflict(String),

    #[error("Rollback required: {0}")]
    RollbackRequired(String),
}

/// Lifecycle manager
pub struct LifecycleManager {
    /// Plugin states
    states: Arc<RwLock<HashMap<String, PluginState>>>,
    /// Registered hooks
    hooks: Arc<RwLock<HashMap<String, Arc<dyn LifecycleHook>>>>,
    /// Event listeners
    listeners: Arc<RwLock<Vec<Box<dyn Fn(LifecycleEvent) + Send + Sync>>>>,
    /// Plugin metadata (activation timestamps, etc.)
    metadata: Arc<RwLock<HashMap<String, PluginMetadata>>>,
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub plugin_id: String,
    pub version: String,
    pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deactivated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub activation_count: u64,
    pub error_count: u64,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            hooks: Arc::new(RwLock::new(HashMap::new())),
            listeners: Arc::new(RwLock::new(Vec::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a lifecycle hook for a plugin
    pub fn register_hook(&self, plugin_id: &str, hook: Arc<dyn LifecycleHook>) {
        self.hooks.write().insert(plugin_id.to_string(), hook);
    }

    /// Unregister a lifecycle hook
    pub fn unregister_hook(&self, plugin_id: &str) {
        self.hooks.write().remove(plugin_id);
    }

    /// Add an event listener
    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(LifecycleEvent) + Send + Sync + 'static,
    {
        self.listeners.write().push(Box::new(listener));
    }

    /// Get plugin state
    pub fn get_state(&self, plugin_id: &str) -> Option<PluginState> {
        self.states.read().get(plugin_id).copied()
    }

    /// Set plugin state
    fn set_state(&self, plugin_id: &str, state: PluginState) {
        self.states.write().insert(plugin_id.to_string(), state);
    }

    /// Emit a lifecycle event
    fn emit_event(&self, event: LifecycleEvent) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener(event.clone());
        }
    }

    /// Activate a plugin
    pub async fn activate(&self, context: ActivationContext) -> Result<(), LifecycleError> {
        let plugin_id = context.plugin_id.clone();

        // Check current state
        let current_state = self.get_state(&plugin_id);
        if current_state == Some(PluginState::Active) {
            return Ok(());
        }

        if current_state == Some(PluginState::Activating) {
            return Err(LifecycleError::AlreadyInProgress(plugin_id));
        }

        info!("Activating plugin: {}", plugin_id);
        self.set_state(&plugin_id, PluginState::Activating);
        self.emit_event(LifecycleEvent::ActivationStarted {
            plugin_id: plugin_id.clone(),
        });

        // Call hook if registered
        if let Some(hook) = self.hooks.read().get(&plugin_id).cloned() {
            match hook.on_activate(&context).await {
                Ok(()) => {
                    self.set_state(&plugin_id, PluginState::Active);
                    self.update_metadata(&plugin_id, |meta| {
                        meta.activated_at = Some(chrono::Utc::now());
                        meta.activation_count += 1;
                    });
                    self.emit_event(LifecycleEvent::Activated {
                        plugin_id: plugin_id.clone(),
                    });
                    info!("Plugin activated: {}", plugin_id);
                    Ok(())
                }
                Err(e) => {
                    let error = e.to_string();
                    self.set_state(&plugin_id, PluginState::Error);
                    self.update_metadata(&plugin_id, |meta| {
                        meta.last_error = Some(error.clone());
                        meta.error_count += 1;
                    });
                    self.emit_event(LifecycleEvent::ActivationFailed {
                        plugin_id: plugin_id.clone(),
                        error: error.clone(),
                    });
                    error!("Plugin activation failed: {} - {}", plugin_id, error);
                    Err(LifecycleError::HookFailed(plugin_id, error))
                }
            }
        } else {
            // No hook registered, just mark as active
            self.set_state(&plugin_id, PluginState::Active);
            self.emit_event(LifecycleEvent::Activated {
                plugin_id: plugin_id.clone(),
            });
            Ok(())
        }
    }

    /// Deactivate a plugin
    pub async fn deactivate(&self, context: DeactivationContext) -> Result<(), LifecycleError> {
        let plugin_id = context.plugin_id.clone();

        let current_state = self.get_state(&plugin_id);
        if current_state == Some(PluginState::Inactive) {
            return Ok(());
        }

        if current_state == Some(PluginState::Deactivating) {
            return Err(LifecycleError::AlreadyInProgress(plugin_id));
        }

        info!("Deactivating plugin: {}", plugin_id);
        self.set_state(&plugin_id, PluginState::Deactivating);
        self.emit_event(LifecycleEvent::DeactivationStarted {
            plugin_id: plugin_id.clone(),
        });

        if let Some(hook) = self.hooks.read().get(&plugin_id).cloned() {
            match hook.on_deactivate(&context).await {
                Ok(()) => {
                    self.set_state(&plugin_id, PluginState::Inactive);
                    self.update_metadata(&plugin_id, |meta| {
                        meta.deactivated_at = Some(chrono::Utc::now());
                    });
                    self.emit_event(LifecycleEvent::Deactivated {
                        plugin_id: plugin_id.clone(),
                    });
                    info!("Plugin deactivated: {}", plugin_id);
                    Ok(())
                }
                Err(e) => {
                    let error = e.to_string();
                    // Still mark as inactive even if hook fails
                    self.set_state(&plugin_id, PluginState::Inactive);
                    self.emit_event(LifecycleEvent::DeactivationFailed {
                        plugin_id: plugin_id.clone(),
                        error: error.clone(),
                    });
                    warn!("Plugin deactivation hook failed: {} - {}", plugin_id, error);
                    Ok(())
                }
            }
        } else {
            self.set_state(&plugin_id, PluginState::Inactive);
            self.emit_event(LifecycleEvent::Deactivated {
                plugin_id: plugin_id.clone(),
            });
            Ok(())
        }
    }

    /// Upgrade a plugin
    pub async fn upgrade(&self, context: UpgradeContext) -> Result<(), LifecycleError> {
        let plugin_id = context.plugin_id.clone();
        let from_version = context.from_version.clone();
        let to_version = context.to_version.clone();

        info!(
            "Upgrading plugin {} from {} to {}",
            plugin_id, from_version, to_version
        );

        self.set_state(&plugin_id, PluginState::Upgrading);
        self.emit_event(LifecycleEvent::UpgradeStarted {
            plugin_id: plugin_id.clone(),
            from_version: from_version.clone(),
            to_version: to_version.clone(),
        });

        if let Some(hook) = self.hooks.read().get(&plugin_id).cloned() {
            match hook.on_upgrade(&context).await {
                Ok(()) => {
                    self.set_state(&plugin_id, PluginState::Active);
                    self.update_metadata(&plugin_id, |meta| {
                        meta.version = to_version.clone();
                    });
                    self.emit_event(LifecycleEvent::Upgraded {
                        plugin_id: plugin_id.clone(),
                        from_version,
                        to_version,
                    });
                    info!("Plugin upgraded: {}", plugin_id);
                    Ok(())
                }
                Err(e) => {
                    let error = e.to_string();
                    self.set_state(&plugin_id, PluginState::Error);
                    self.emit_event(LifecycleEvent::UpgradeFailed {
                        plugin_id: plugin_id.clone(),
                        error: error.clone(),
                    });
                    error!("Plugin upgrade failed: {} - {}", plugin_id, error);
                    Err(LifecycleError::HookFailed(plugin_id, error))
                }
            }
        } else {
            self.set_state(&plugin_id, PluginState::Active);
            self.emit_event(LifecycleEvent::Upgraded {
                plugin_id: plugin_id.clone(),
                from_version,
                to_version,
            });
            Ok(())
        }
    }

    /// Uninstall a plugin
    pub async fn uninstall(&self, context: UninstallContext) -> Result<(), LifecycleError> {
        let plugin_id = context.plugin_id.clone();

        info!("Uninstalling plugin: {}", plugin_id);

        // First deactivate if active
        if self.get_state(&plugin_id) == Some(PluginState::Active) {
            let deactivate_context = DeactivationContext {
                plugin_id: plugin_id.clone(),
                manifest: context.manifest.clone(),
                network_wide: context.network_wide,
                site_id: None,
                reason: DeactivationReason::Uninstall,
            };
            self.deactivate(deactivate_context).await?;
        }

        self.set_state(&plugin_id, PluginState::Uninstalling);
        self.emit_event(LifecycleEvent::UninstallStarted {
            plugin_id: plugin_id.clone(),
        });

        if let Some(hook) = self.hooks.read().get(&plugin_id).cloned() {
            match hook.on_uninstall(&context).await {
                Ok(()) => {
                    // Remove all traces
                    self.states.write().remove(&plugin_id);
                    self.hooks.write().remove(&plugin_id);
                    self.metadata.write().remove(&plugin_id);
                    self.emit_event(LifecycleEvent::Uninstalled {
                        plugin_id: plugin_id.clone(),
                    });
                    info!("Plugin uninstalled: {}", plugin_id);
                    Ok(())
                }
                Err(e) => {
                    let error = e.to_string();
                    self.set_state(&plugin_id, PluginState::Error);
                    self.emit_event(LifecycleEvent::UninstallFailed {
                        plugin_id: plugin_id.clone(),
                        error: error.clone(),
                    });
                    error!("Plugin uninstall failed: {} - {}", plugin_id, error);
                    Err(LifecycleError::HookFailed(plugin_id, error))
                }
            }
        } else {
            self.states.write().remove(&plugin_id);
            self.metadata.write().remove(&plugin_id);
            self.emit_event(LifecycleEvent::Uninstalled {
                plugin_id: plugin_id.clone(),
            });
            Ok(())
        }
    }

    /// Pause a plugin (temporarily disable)
    pub fn pause(&self, plugin_id: &str) -> Result<(), LifecycleError> {
        let current_state = self.get_state(plugin_id);
        if current_state != Some(PluginState::Active) {
            return Err(LifecycleError::InvalidState(
                plugin_id.to_string(),
                current_state,
            ));
        }

        self.set_state(plugin_id, PluginState::Paused);
        self.emit_event(LifecycleEvent::Paused {
            plugin_id: plugin_id.to_string(),
        });
        info!("Plugin paused: {}", plugin_id);
        Ok(())
    }

    /// Resume a paused plugin
    pub fn resume(&self, plugin_id: &str) -> Result<(), LifecycleError> {
        let current_state = self.get_state(plugin_id);
        if current_state != Some(PluginState::Paused) {
            return Err(LifecycleError::InvalidState(
                plugin_id.to_string(),
                current_state,
            ));
        }

        self.set_state(plugin_id, PluginState::Active);
        self.emit_event(LifecycleEvent::Resumed {
            plugin_id: plugin_id.to_string(),
        });
        info!("Plugin resumed: {}", plugin_id);
        Ok(())
    }

    /// Run init hooks for all active plugins
    pub async fn run_init_hooks(&self, context: &InitContext) {
        let hooks = self.hooks.read().clone();
        let states = self.states.read().clone();

        for (plugin_id, hook) in hooks {
            if states.get(&plugin_id) == Some(&PluginState::Active) {
                let mut ctx = context.clone();
                ctx.plugin_id = plugin_id.clone();
                if let Err(e) = hook.on_init(&ctx).await {
                    warn!("Plugin init hook failed for {}: {}", plugin_id, e);
                }
            }
        }
    }

    /// Get all active plugins
    pub fn get_active_plugins(&self) -> Vec<String> {
        self.states
            .read()
            .iter()
            .filter(|(_, state)| **state == PluginState::Active)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all plugins in a specific state
    pub fn get_plugins_by_state(&self, state: PluginState) -> Vec<String> {
        self.states
            .read()
            .iter()
            .filter(|(_, s)| **s == state)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get plugin metadata
    pub fn get_metadata(&self, plugin_id: &str) -> Option<PluginMetadata> {
        self.metadata.read().get(plugin_id).cloned()
    }

    /// Update plugin metadata
    fn update_metadata<F>(&self, plugin_id: &str, updater: F)
    where
        F: FnOnce(&mut PluginMetadata),
    {
        let mut metadata = self.metadata.write();
        let meta = metadata.entry(plugin_id.to_string()).or_insert_with(|| PluginMetadata {
            plugin_id: plugin_id.to_string(),
            version: String::new(),
            activated_at: None,
            deactivated_at: None,
            last_error: None,
            activation_count: 0,
            error_count: 0,
        });
        updater(meta);
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifecycle error
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("Operation already in progress for plugin: {0}")]
    AlreadyInProgress(String),

    #[error("Invalid state for plugin {0}: {:?}", .1)]
    InvalidState(String, Option<PluginState>),

    #[error("Hook failed for plugin {0}: {1}")]
    HookFailed(String, String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Dependency error: {0}")]
    Dependency(String),

    #[error("Conflict with plugin: {0}")]
    Conflict(String),
}

/// Action hook registration
#[derive(Debug, Clone)]
pub struct ActionHookRegistration {
    pub hook_name: String,
    pub plugin_id: String,
    pub callback: String,
    pub priority: i32,
}

/// Filter hook registration
#[derive(Debug, Clone)]
pub struct FilterHookRegistration {
    pub hook_name: String,
    pub plugin_id: String,
    pub callback: String,
    pub priority: i32,
}

/// Hook registry for WordPress-compatible action/filter system
pub struct HookRegistry {
    actions: RwLock<HashMap<String, Vec<ActionHookRegistration>>>,
    filters: RwLock<HashMap<String, Vec<FilterHookRegistration>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
            filters: RwLock::new(HashMap::new()),
        }
    }

    /// Register an action hook
    pub fn add_action(&self, hook_name: &str, registration: ActionHookRegistration) {
        let mut actions = self.actions.write();
        let hooks = actions.entry(hook_name.to_string()).or_insert_with(Vec::new);
        hooks.push(registration);
        hooks.sort_by_key(|h| h.priority);
    }

    /// Register a filter hook
    pub fn add_filter(&self, hook_name: &str, registration: FilterHookRegistration) {
        let mut filters = self.filters.write();
        let hooks = filters.entry(hook_name.to_string()).or_insert_with(Vec::new);
        hooks.push(registration);
        hooks.sort_by_key(|h| h.priority);
    }

    /// Remove all hooks for a plugin
    pub fn remove_plugin_hooks(&self, plugin_id: &str) {
        {
            let mut actions = self.actions.write();
            for hooks in actions.values_mut() {
                hooks.retain(|h| h.plugin_id != plugin_id);
            }
        }
        {
            let mut filters = self.filters.write();
            for hooks in filters.values_mut() {
                hooks.retain(|h| h.plugin_id != plugin_id);
            }
        }
    }

    /// Get action hooks for a hook name
    pub fn get_actions(&self, hook_name: &str) -> Vec<ActionHookRegistration> {
        self.actions.read().get(hook_name).cloned().unwrap_or_default()
    }

    /// Get filter hooks for a hook name
    pub fn get_filters(&self, hook_name: &str) -> Vec<FilterHookRegistration> {
        self.filters.read().get(hook_name).cloned().unwrap_or_default()
    }

    /// Check if hook has any registered callbacks
    pub fn has_action(&self, hook_name: &str) -> bool {
        self.actions.read().get(hook_name).map(|h| !h.is_empty()).unwrap_or(false)
    }

    /// Check if filter has any registered callbacks
    pub fn has_filter(&self, hook_name: &str) -> bool {
        self.filters.read().get(hook_name).map(|h| !h.is_empty()).unwrap_or(false)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_state_display() {
        assert_eq!(PluginState::Active.to_string(), "active");
        assert_eq!(PluginState::Inactive.to_string(), "inactive");
    }

    #[test]
    fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new();
        assert!(manager.get_active_plugins().is_empty());
    }

    #[test]
    fn test_hook_registry() {
        let registry = HookRegistry::new();

        registry.add_action(
            "init",
            ActionHookRegistration {
                hook_name: "init".to_string(),
                plugin_id: "test-plugin".to_string(),
                callback: "on_init".to_string(),
                priority: 10,
            },
        );

        assert!(registry.has_action("init"));
        assert!(!registry.has_action("nonexistent"));

        let actions = registry.get_actions("init");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].plugin_id, "test-plugin");

        registry.remove_plugin_hooks("test-plugin");
        assert!(!registry.has_action("init"));
    }

    #[test]
    fn test_hook_priority_sorting() {
        let registry = HookRegistry::new();

        registry.add_filter(
            "content",
            FilterHookRegistration {
                hook_name: "content".to_string(),
                plugin_id: "plugin-a".to_string(),
                callback: "filter_a".to_string(),
                priority: 20,
            },
        );

        registry.add_filter(
            "content",
            FilterHookRegistration {
                hook_name: "content".to_string(),
                plugin_id: "plugin-b".to_string(),
                callback: "filter_b".to_string(),
                priority: 5,
            },
        );

        let filters = registry.get_filters("content");
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].plugin_id, "plugin-b"); // Lower priority first
        assert_eq!(filters[1].plugin_id, "plugin-a");
    }
}
