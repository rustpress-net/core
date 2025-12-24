//! Plugin Admin Page Registration
//!
//! Allows plugins to add pages to the admin dashboard.

use crate::manifest::{AdminMenuItem, AdminPage, AdminSection, DashboardWidget};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::debug;

/// Admin registry for plugin pages and menus
pub struct AdminRegistry {
    /// Menu items
    menus: Arc<RwLock<Vec<RegisteredMenuItem>>>,
    /// Admin pages
    pages: Arc<RwLock<HashMap<String, RegisteredPage>>>,
    /// Dashboard widgets
    widgets: Arc<RwLock<Vec<RegisteredDashboardWidget>>>,
    /// Settings pages
    settings_pages: Arc<RwLock<HashMap<String, SettingsPageConfig>>>,
}

/// Registered menu item
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredMenuItem {
    pub plugin_id: String,
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub position: i32,
    pub capability: String,
    pub parent: Option<String>,
    pub page_id: String,
    pub badge_count: Option<u32>,
}

/// Registered admin page
#[derive(Debug, Clone)]
pub struct RegisteredPage {
    pub plugin_id: String,
    pub id: String,
    pub title: String,
    pub handler: String,
    pub template: Option<String>,
    pub capability: String,
    pub scripts: Vec<String>,
    pub styles: Vec<String>,
    pub help_tabs: Vec<HelpTab>,
}

/// Help tab for admin pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTab {
    pub id: String,
    pub title: String,
    pub content: String,
}

/// Registered dashboard widget
#[derive(Debug, Clone)]
pub struct RegisteredDashboardWidget {
    pub plugin_id: String,
    pub id: String,
    pub title: String,
    pub handler: String,
    pub position: WidgetPosition,
    pub capability: String,
    pub configurable: bool,
    pub config_handler: Option<String>,
}

/// Widget position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WidgetPosition {
    Normal,
    Side,
    Column3,
    Column4,
}

impl Default for WidgetPosition {
    fn default() -> Self {
        Self::Normal
    }
}

/// Settings page configuration
#[derive(Debug, Clone)]
pub struct SettingsPageConfig {
    pub plugin_id: String,
    pub title: String,
    pub capability: String,
    pub icon: Option<String>,
    pub sections: Vec<SettingsPageSection>,
}

/// Settings page section
#[derive(Debug, Clone)]
pub struct SettingsPageSection {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<String>,
}

impl AdminRegistry {
    pub fn new() -> Self {
        Self {
            menus: Arc::new(RwLock::new(Vec::new())),
            pages: Arc::new(RwLock::new(HashMap::new())),
            widgets: Arc::new(RwLock::new(Vec::new())),
            settings_pages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register admin from plugin manifest
    pub fn register_from_manifest(&self, plugin_id: &str, section: &AdminSection) {
        // Register menu items
        for menu_item in &section.menu {
            self.register_menu_item(plugin_id, menu_item);
        }

        // Register pages
        for page in &section.pages {
            self.register_page(plugin_id, page);
        }

        // Register dashboard widgets
        for widget in &section.dashboard_widgets {
            self.register_widget(plugin_id, widget);
        }

        // Register settings page
        if let Some(settings) = &section.settings_page {
            self.register_settings_page(plugin_id, settings);
        }

        debug!("Registered admin components for plugin: {}", plugin_id);
    }

    /// Register a menu item
    fn register_menu_item(&self, plugin_id: &str, item: &AdminMenuItem) {
        let menu = RegisteredMenuItem {
            plugin_id: plugin_id.to_string(),
            id: item.id.clone(),
            label: item.label.clone(),
            icon: item.icon.clone(),
            position: item.position.unwrap_or(100),
            capability: item.capability.clone().unwrap_or_else(|| "manage_options".to_string()),
            parent: item.parent.clone(),
            page_id: item.page.clone(),
            badge_count: None,
        };

        self.menus.write().push(menu);
    }

    /// Register a page
    fn register_page(&self, plugin_id: &str, page: &AdminPage) {
        let registered = RegisteredPage {
            plugin_id: plugin_id.to_string(),
            id: page.id.clone(),
            title: page.title.clone(),
            handler: page.handler.clone(),
            template: page.template.clone(),
            capability: page.capability.clone().unwrap_or_else(|| "manage_options".to_string()),
            scripts: Vec::new(),
            styles: Vec::new(),
            help_tabs: Vec::new(),
        };

        self.pages.write().insert(page.id.clone(), registered);
    }

    /// Register a dashboard widget
    fn register_widget(&self, plugin_id: &str, widget: &DashboardWidget) {
        let position = match widget.position {
            crate::manifest::WidgetPosition::Normal => WidgetPosition::Normal,
            crate::manifest::WidgetPosition::Side => WidgetPosition::Side,
            crate::manifest::WidgetPosition::Column3 => WidgetPosition::Column3,
            crate::manifest::WidgetPosition::Column4 => WidgetPosition::Column4,
        };

        let registered = RegisteredDashboardWidget {
            plugin_id: plugin_id.to_string(),
            id: widget.id.clone(),
            title: widget.title.clone(),
            handler: widget.handler.clone(),
            position,
            capability: "read".to_string(),
            configurable: false,
            config_handler: None,
        };

        self.widgets.write().push(registered);
    }

    /// Register a settings page
    fn register_settings_page(&self, plugin_id: &str, settings: &crate::manifest::SettingsPage) {
        let config = SettingsPageConfig {
            plugin_id: plugin_id.to_string(),
            title: settings.title.clone(),
            capability: settings.capability.clone().unwrap_or_else(|| "manage_options".to_string()),
            icon: settings.icon.clone(),
            sections: Vec::new(),
        };

        self.settings_pages.write().insert(plugin_id.to_string(), config);
    }

    /// Add a top-level menu
    pub fn add_menu_page(
        &self,
        plugin_id: &str,
        page_title: &str,
        menu_title: &str,
        capability: &str,
        menu_slug: &str,
        handler: &str,
    ) -> MenuBuilder {
        MenuBuilder {
            registry: self,
            plugin_id: plugin_id.to_string(),
            page_title: page_title.to_string(),
            menu_title: menu_title.to_string(),
            capability: capability.to_string(),
            menu_slug: menu_slug.to_string(),
            handler: handler.to_string(),
            icon: None,
            position: None,
            parent: None,
        }
    }

    /// Add a submenu
    pub fn add_submenu_page(
        &self,
        plugin_id: &str,
        parent_slug: &str,
        page_title: &str,
        menu_title: &str,
        capability: &str,
        menu_slug: &str,
        handler: &str,
    ) -> MenuBuilder {
        MenuBuilder {
            registry: self,
            plugin_id: plugin_id.to_string(),
            page_title: page_title.to_string(),
            menu_title: menu_title.to_string(),
            capability: capability.to_string(),
            menu_slug: menu_slug.to_string(),
            handler: handler.to_string(),
            icon: None,
            position: None,
            parent: Some(parent_slug.to_string()),
        }
    }

    /// Get menu tree
    pub fn get_menu_tree(&self, user_capabilities: &[String]) -> Vec<MenuNode> {
        let menus = self.menus.read();
        let mut nodes: HashMap<String, MenuNode> = HashMap::new();
        let mut top_level: Vec<String> = Vec::new();

        // First pass: create all nodes
        for menu in menus.iter() {
            if !user_capabilities.contains(&menu.capability) {
                continue;
            }

            let node = MenuNode {
                id: menu.id.clone(),
                label: menu.label.clone(),
                icon: menu.icon.clone(),
                position: menu.position,
                page_id: menu.page_id.clone(),
                badge_count: menu.badge_count,
                children: Vec::new(),
            };

            if menu.parent.is_none() {
                top_level.push(menu.id.clone());
            }

            nodes.insert(menu.id.clone(), node);
        }

        // Second pass: build tree
        for menu in menus.iter() {
            if let Some(parent_id) = &menu.parent {
                // Clone the child first
                let child = nodes.get(&menu.id).cloned();
                if let (Some(parent), Some(child)) = (nodes.get_mut(parent_id), child) {
                    parent.children.push(child);
                }
            }
        }

        // Sort and return top-level items
        let mut result: Vec<MenuNode> = top_level
            .iter()
            .filter_map(|id| nodes.remove(id))
            .collect();

        result.sort_by_key(|n| n.position);

        for node in &mut result {
            node.children.sort_by_key(|c| c.position);
        }

        result
    }

    /// Get page by ID
    pub fn get_page(&self, page_id: &str) -> Option<RegisteredPage> {
        self.pages.read().get(page_id).cloned()
    }

    /// Get all dashboard widgets
    pub fn get_dashboard_widgets(&self, user_capabilities: &[String]) -> Vec<RegisteredDashboardWidget> {
        self.widgets
            .read()
            .iter()
            .filter(|w| user_capabilities.contains(&w.capability))
            .cloned()
            .collect()
    }

    /// Get settings page for plugin
    pub fn get_settings_page(&self, plugin_id: &str) -> Option<SettingsPageConfig> {
        self.settings_pages.read().get(plugin_id).cloned()
    }

    /// Update menu badge count
    pub fn update_badge(&self, menu_id: &str, count: u32) {
        let mut menus = self.menus.write();
        if let Some(menu) = menus.iter_mut().find(|m| m.id == menu_id) {
            menu.badge_count = if count > 0 { Some(count) } else { None };
        }
    }

    /// Add help tab to page
    pub fn add_help_tab(&self, page_id: &str, tab: HelpTab) {
        let mut pages = self.pages.write();
        if let Some(page) = pages.get_mut(page_id) {
            page.help_tabs.push(tab);
        }
    }

    /// Unregister all for a plugin
    pub fn unregister(&self, plugin_id: &str) {
        self.menus.write().retain(|m| m.plugin_id != plugin_id);
        self.pages.write().retain(|_, p| p.plugin_id != plugin_id);
        self.widgets.write().retain(|w| w.plugin_id != plugin_id);
        self.settings_pages.write().remove(plugin_id);
    }
}

impl Default for AdminRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Menu tree node
#[derive(Debug, Clone, Serialize)]
pub struct MenuNode {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub position: i32,
    pub page_id: String,
    pub badge_count: Option<u32>,
    pub children: Vec<MenuNode>,
}

/// Menu builder
pub struct MenuBuilder<'a> {
    registry: &'a AdminRegistry,
    plugin_id: String,
    page_title: String,
    menu_title: String,
    capability: String,
    menu_slug: String,
    handler: String,
    icon: Option<String>,
    position: Option<i32>,
    parent: Option<String>,
}

impl<'a> MenuBuilder<'a> {
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }

    pub fn build(self) {
        let menu = RegisteredMenuItem {
            plugin_id: self.plugin_id.clone(),
            id: self.menu_slug.clone(),
            label: self.menu_title,
            icon: self.icon,
            position: self.position.unwrap_or(100),
            capability: self.capability.clone(),
            parent: self.parent,
            page_id: self.menu_slug.clone(),
            badge_count: None,
        };

        self.registry.menus.write().push(menu);

        let page = RegisteredPage {
            plugin_id: self.plugin_id,
            id: self.menu_slug,
            title: self.page_title,
            handler: self.handler,
            template: None,
            capability: self.capability,
            scripts: Vec::new(),
            styles: Vec::new(),
            help_tabs: Vec::new(),
        };

        self.registry.pages.write().insert(page.id.clone(), page);
    }
}

/// Admin notice
#[derive(Debug, Clone, Serialize)]
pub struct AdminNotice {
    pub id: String,
    pub plugin_id: String,
    pub message: String,
    pub notice_type: NoticeType,
    pub dismissible: bool,
}

/// Notice type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NoticeType {
    Success,
    Warning,
    Error,
    Info,
}

/// Admin notices manager
pub struct NoticeManager {
    notices: Arc<RwLock<Vec<AdminNotice>>>,
}

impl NoticeManager {
    pub fn new() -> Self {
        Self {
            notices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add(&self, notice: AdminNotice) {
        self.notices.write().push(notice);
    }

    pub fn add_success(&self, plugin_id: &str, message: &str) {
        self.add(AdminNotice {
            id: uuid::Uuid::new_v4().to_string(),
            plugin_id: plugin_id.to_string(),
            message: message.to_string(),
            notice_type: NoticeType::Success,
            dismissible: true,
        });
    }

    pub fn add_error(&self, plugin_id: &str, message: &str) {
        self.add(AdminNotice {
            id: uuid::Uuid::new_v4().to_string(),
            plugin_id: plugin_id.to_string(),
            message: message.to_string(),
            notice_type: NoticeType::Error,
            dismissible: true,
        });
    }

    pub fn get_all(&self) -> Vec<AdminNotice> {
        self.notices.read().clone()
    }

    pub fn dismiss(&self, notice_id: &str) {
        self.notices.write().retain(|n| n.id != notice_id);
    }

    pub fn clear_plugin(&self, plugin_id: &str) {
        self.notices.write().retain(|n| n.plugin_id != plugin_id);
    }
}

impl Default for NoticeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_registry() {
        let registry = AdminRegistry::new();

        registry
            .add_menu_page(
                "test-plugin",
                "Test Page",
                "Test",
                "manage_options",
                "test-page",
                "render_test_page",
            )
            .icon("dashicons-admin-generic")
            .position(30)
            .build();

        let menus = registry.menus.read();
        assert_eq!(menus.len(), 1);
        assert_eq!(menus[0].label, "Test");
    }

    #[test]
    fn test_menu_tree() {
        let registry = AdminRegistry::new();

        registry
            .add_menu_page("plugin", "Parent", "Parent", "read", "parent", "h")
            .build();

        registry
            .add_submenu_page("plugin", "parent", "Child", "Child", "read", "child", "h")
            .build();

        let tree = registry.get_menu_tree(&["read".to_string()]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
    }
}
