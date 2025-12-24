//! # User Roles System
//!
//! Role management with custom capabilities.
//!
//! Features:
//! - WordPress-compatible default roles
//! - Custom role creation
//! - Capability management
//! - Role hierarchy
//! - Role assignment

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// User role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role identifier (slug)
    pub name: String,

    /// Display name
    pub display_name: String,

    /// Role description
    pub description: String,

    /// Capabilities granted by this role
    pub capabilities: HashSet<String>,

    /// Whether this is a built-in role
    pub is_builtin: bool,

    /// Role level (for hierarchy, higher = more privileges)
    pub level: u32,

    /// Role is enabled
    pub enabled: bool,

    /// When the role was created
    pub created_at: DateTime<Utc>,

    /// When last modified
    pub modified_at: DateTime<Utc>,
}

impl Role {
    pub fn new(name: &str, display_name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            description: String::new(),
            capabilities: HashSet::new(),
            is_builtin: false,
            level: 0,
            enabled: true,
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.level = level;
        self
    }

    pub fn builtin(mut self) -> Self {
        self.is_builtin = true;
        self
    }

    /// Add a capability
    pub fn add_cap(&mut self, capability: &str) {
        self.capabilities.insert(capability.to_string());
        self.modified_at = Utc::now();
    }

    /// Add multiple capabilities
    pub fn add_caps(&mut self, capabilities: &[&str]) {
        for cap in capabilities {
            self.capabilities.insert(cap.to_string());
        }
        self.modified_at = Utc::now();
    }

    /// Remove a capability
    pub fn remove_cap(&mut self, capability: &str) {
        self.capabilities.remove(capability);
        self.modified_at = Utc::now();
    }

    /// Check if role has capability
    pub fn has_cap(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }

    /// Get all capabilities
    pub fn get_caps(&self) -> Vec<&String> {
        self.capabilities.iter().collect()
    }
}

/// Capability definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability identifier
    pub name: String,

    /// Display label
    pub label: String,

    /// Description
    pub description: String,

    /// Category for grouping
    pub category: String,

    /// Whether it's a primitive capability
    pub is_primitive: bool,

    /// Implied capabilities (if this cap is granted, these are too)
    pub implies: Vec<String>,
}

impl Capability {
    pub fn new(name: &str, label: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            label: label.to_string(),
            description: String::new(),
            category: category.to_string(),
            is_primitive: true,
            implies: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn implies(mut self, caps: &[&str]) -> Self {
        self.implies = caps.iter().map(|s| s.to_string()).collect();
        self.is_primitive = false;
        self
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: i64,
    pub role: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<i64>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl UserRole {
    pub fn new(user_id: i64, role: &str) -> Self {
        Self {
            user_id,
            role: role.to_string(),
            assigned_at: Utc::now(),
            assigned_by: None,
            expires_at: None,
        }
    }

    pub fn with_expiry(mut self, expires: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }

    pub fn assigned_by(mut self, user_id: i64) -> Self {
        self.assigned_by = Some(user_id);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }
}

/// Role manager
pub struct RoleManager {
    roles: HashMap<String, Role>,
    capabilities: HashMap<String, Capability>,
    user_roles: HashMap<i64, Vec<UserRole>>,
    default_role: String,
}

impl Default for RoleManager {
    fn default() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            capabilities: HashMap::new(),
            user_roles: HashMap::new(),
            default_role: "subscriber".to_string(),
        };

        manager.register_default_capabilities();
        manager.register_default_roles();

        manager
    }
}

impl RoleManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register WordPress-compatible default capabilities
    fn register_default_capabilities(&mut self) {
        // Reading
        self.register_capability(Capability::new("read", "Read", "reading")
            .with_description("Allows access to the dashboard"));

        // Posts
        self.register_capability(Capability::new("edit_posts", "Edit Posts", "posts")
            .with_description("Allows editing own posts"));
        self.register_capability(Capability::new("edit_others_posts", "Edit Others' Posts", "posts")
            .with_description("Allows editing other users' posts"));
        self.register_capability(Capability::new("edit_published_posts", "Edit Published Posts", "posts")
            .with_description("Allows editing already published posts"));
        self.register_capability(Capability::new("publish_posts", "Publish Posts", "posts")
            .with_description("Allows publishing posts"));
        self.register_capability(Capability::new("delete_posts", "Delete Posts", "posts")
            .with_description("Allows deleting own posts"));
        self.register_capability(Capability::new("delete_others_posts", "Delete Others' Posts", "posts")
            .with_description("Allows deleting other users' posts"));
        self.register_capability(Capability::new("delete_published_posts", "Delete Published Posts", "posts")
            .with_description("Allows deleting published posts"));
        self.register_capability(Capability::new("delete_private_posts", "Delete Private Posts", "posts")
            .with_description("Allows deleting private posts"));
        self.register_capability(Capability::new("edit_private_posts", "Edit Private Posts", "posts")
            .with_description("Allows editing private posts"));
        self.register_capability(Capability::new("read_private_posts", "Read Private Posts", "posts")
            .with_description("Allows reading private posts"));

        // Pages
        self.register_capability(Capability::new("edit_pages", "Edit Pages", "pages")
            .with_description("Allows editing own pages"));
        self.register_capability(Capability::new("edit_others_pages", "Edit Others' Pages", "pages")
            .with_description("Allows editing other users' pages"));
        self.register_capability(Capability::new("edit_published_pages", "Edit Published Pages", "pages")
            .with_description("Allows editing published pages"));
        self.register_capability(Capability::new("publish_pages", "Publish Pages", "pages")
            .with_description("Allows publishing pages"));
        self.register_capability(Capability::new("delete_pages", "Delete Pages", "pages")
            .with_description("Allows deleting own pages"));
        self.register_capability(Capability::new("delete_others_pages", "Delete Others' Pages", "pages")
            .with_description("Allows deleting other users' pages"));
        self.register_capability(Capability::new("delete_published_pages", "Delete Published Pages", "pages")
            .with_description("Allows deleting published pages"));
        self.register_capability(Capability::new("delete_private_pages", "Delete Private Pages", "pages")
            .with_description("Allows deleting private pages"));
        self.register_capability(Capability::new("edit_private_pages", "Edit Private Pages", "pages")
            .with_description("Allows editing private pages"));
        self.register_capability(Capability::new("read_private_pages", "Read Private Pages", "pages")
            .with_description("Allows reading private pages"));

        // Media/Files
        self.register_capability(Capability::new("upload_files", "Upload Files", "media")
            .with_description("Allows uploading files"));
        self.register_capability(Capability::new("edit_files", "Edit Files", "media")
            .with_description("Allows editing uploaded files"));
        self.register_capability(Capability::new("delete_files", "Delete Files", "media")
            .with_description("Allows deleting uploaded files"));

        // Comments
        self.register_capability(Capability::new("moderate_comments", "Moderate Comments", "comments")
            .with_description("Allows moderating comments"));
        self.register_capability(Capability::new("edit_comment", "Edit Comments", "comments")
            .with_description("Allows editing comments"));

        // Categories & Tags
        self.register_capability(Capability::new("manage_categories", "Manage Categories", "taxonomy")
            .with_description("Allows managing categories"));
        self.register_capability(Capability::new("edit_categories", "Edit Categories", "taxonomy")
            .with_description("Allows editing categories"));
        self.register_capability(Capability::new("delete_categories", "Delete Categories", "taxonomy")
            .with_description("Allows deleting categories"));
        self.register_capability(Capability::new("assign_categories", "Assign Categories", "taxonomy")
            .with_description("Allows assigning categories to posts"));
        self.register_capability(Capability::new("manage_tags", "Manage Tags", "taxonomy")
            .with_description("Allows managing tags"));
        self.register_capability(Capability::new("edit_tags", "Edit Tags", "taxonomy")
            .with_description("Allows editing tags"));
        self.register_capability(Capability::new("delete_tags", "Delete Tags", "taxonomy")
            .with_description("Allows deleting tags"));
        self.register_capability(Capability::new("assign_tags", "Assign Tags", "taxonomy")
            .with_description("Allows assigning tags to posts"));

        // Users
        self.register_capability(Capability::new("list_users", "List Users", "users")
            .with_description("Allows viewing user list"));
        self.register_capability(Capability::new("create_users", "Create Users", "users")
            .with_description("Allows creating new users"));
        self.register_capability(Capability::new("edit_users", "Edit Users", "users")
            .with_description("Allows editing user profiles"));
        self.register_capability(Capability::new("delete_users", "Delete Users", "users")
            .with_description("Allows deleting users"));
        self.register_capability(Capability::new("promote_users", "Promote Users", "users")
            .with_description("Allows changing user roles"));

        // Themes
        self.register_capability(Capability::new("switch_themes", "Switch Themes", "themes")
            .with_description("Allows switching themes"));
        self.register_capability(Capability::new("edit_themes", "Edit Themes", "themes")
            .with_description("Allows editing theme files"));
        self.register_capability(Capability::new("edit_theme_options", "Edit Theme Options", "themes")
            .with_description("Allows editing theme options"));
        self.register_capability(Capability::new("install_themes", "Install Themes", "themes")
            .with_description("Allows installing themes"));
        self.register_capability(Capability::new("update_themes", "Update Themes", "themes")
            .with_description("Allows updating themes"));
        self.register_capability(Capability::new("delete_themes", "Delete Themes", "themes")
            .with_description("Allows deleting themes"));

        // Plugins
        self.register_capability(Capability::new("activate_plugins", "Activate Plugins", "plugins")
            .with_description("Allows activating plugins"));
        self.register_capability(Capability::new("edit_plugins", "Edit Plugins", "plugins")
            .with_description("Allows editing plugin files"));
        self.register_capability(Capability::new("install_plugins", "Install Plugins", "plugins")
            .with_description("Allows installing plugins"));
        self.register_capability(Capability::new("update_plugins", "Update Plugins", "plugins")
            .with_description("Allows updating plugins"));
        self.register_capability(Capability::new("delete_plugins", "Delete Plugins", "plugins")
            .with_description("Allows deleting plugins"));

        // Settings
        self.register_capability(Capability::new("manage_options", "Manage Options", "settings")
            .with_description("Allows managing site settings"));
        self.register_capability(Capability::new("export", "Export", "settings")
            .with_description("Allows exporting site content"));
        self.register_capability(Capability::new("import", "Import", "settings")
            .with_description("Allows importing content"));

        // Super admin capabilities
        self.register_capability(Capability::new("manage_network", "Manage Network", "multisite")
            .with_description("Super admin capability for multisite"));
        self.register_capability(Capability::new("manage_sites", "Manage Sites", "multisite")
            .with_description("Allows managing sites in multisite"));
        self.register_capability(Capability::new("manage_network_users", "Manage Network Users", "multisite")
            .with_description("Allows managing network users"));
        self.register_capability(Capability::new("manage_network_plugins", "Manage Network Plugins", "multisite")
            .with_description("Allows managing network plugins"));
        self.register_capability(Capability::new("manage_network_themes", "Manage Network Themes", "multisite")
            .with_description("Allows managing network themes"));
        self.register_capability(Capability::new("manage_network_options", "Manage Network Options", "multisite")
            .with_description("Allows managing network options"));

        // Misc
        self.register_capability(Capability::new("unfiltered_html", "Unfiltered HTML", "misc")
            .with_description("Allows posting unfiltered HTML"));
        self.register_capability(Capability::new("unfiltered_upload", "Unfiltered Upload", "misc")
            .with_description("Allows uploading any file type"));
        self.register_capability(Capability::new("edit_dashboard", "Edit Dashboard", "misc")
            .with_description("Allows editing dashboard widgets"));
        self.register_capability(Capability::new("update_core", "Update Core", "misc")
            .with_description("Allows updating WordPress core"));
    }

    /// Register WordPress-compatible default roles
    fn register_default_roles(&mut self) {
        // Administrator
        let mut admin = Role::new("administrator", "Administrator")
            .with_description("Full site access")
            .with_level(10)
            .builtin();

        // Add all capabilities to admin
        let all_caps: Vec<String> = self.capabilities.keys().cloned().collect();
        for cap in all_caps {
            admin.add_cap(&cap);
        }
        self.register_role(admin);

        // Editor
        let mut editor = Role::new("editor", "Editor")
            .with_description("Can publish and manage posts including others' posts")
            .with_level(7)
            .builtin();
        editor.add_caps(&[
            "read",
            "edit_posts", "edit_others_posts", "edit_published_posts",
            "publish_posts", "delete_posts", "delete_others_posts",
            "delete_published_posts", "delete_private_posts",
            "edit_private_posts", "read_private_posts",
            "edit_pages", "edit_others_pages", "edit_published_pages",
            "publish_pages", "delete_pages", "delete_others_pages",
            "delete_published_pages", "delete_private_pages",
            "edit_private_pages", "read_private_pages",
            "upload_files", "moderate_comments",
            "manage_categories", "manage_tags", "edit_categories",
            "delete_categories", "edit_tags", "delete_tags",
            "assign_categories", "assign_tags",
            "unfiltered_html",
        ]);
        self.register_role(editor);

        // Author
        let mut author = Role::new("author", "Author")
            .with_description("Can publish and manage their own posts")
            .with_level(5)
            .builtin();
        author.add_caps(&[
            "read",
            "edit_posts", "edit_published_posts", "publish_posts",
            "delete_posts", "delete_published_posts",
            "upload_files",
            "assign_categories", "assign_tags",
        ]);
        self.register_role(author);

        // Contributor
        let mut contributor = Role::new("contributor", "Contributor")
            .with_description("Can write and manage their own posts but cannot publish")
            .with_level(3)
            .builtin();
        contributor.add_caps(&[
            "read",
            "edit_posts", "delete_posts",
            "assign_categories", "assign_tags",
        ]);
        self.register_role(contributor);

        // Subscriber
        let mut subscriber = Role::new("subscriber", "Subscriber")
            .with_description("Can only read content and manage their profile")
            .with_level(1)
            .builtin();
        subscriber.add_caps(&["read"]);
        self.register_role(subscriber);
    }

    /// Register a capability
    pub fn register_capability(&mut self, capability: Capability) {
        self.capabilities.insert(capability.name.clone(), capability);
    }

    /// Register a role
    pub fn register_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Get a role by name
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Get a mutable role
    pub fn get_role_mut(&mut self, name: &str) -> Option<&mut Role> {
        self.roles.get_mut(name)
    }

    /// Get all roles
    pub fn get_roles(&self) -> Vec<&Role> {
        let mut roles: Vec<&Role> = self.roles.values().collect();
        roles.sort_by(|a, b| b.level.cmp(&a.level));
        roles
    }

    /// Get all editable (non-builtin) roles
    pub fn get_editable_roles(&self) -> Vec<&Role> {
        self.roles.values()
            .filter(|r| !r.is_builtin)
            .collect()
    }

    /// Create a new custom role
    pub fn create_role(&mut self, name: &str, display_name: &str, capabilities: &[&str]) -> Result<&Role, String> {
        if self.roles.contains_key(name) {
            return Err("Role already exists".to_string());
        }

        let mut role = Role::new(name, display_name);
        role.add_caps(capabilities);
        self.roles.insert(name.to_string(), role);

        Ok(self.roles.get(name).unwrap())
    }

    /// Delete a role (non-builtin only)
    pub fn delete_role(&mut self, name: &str) -> Result<(), String> {
        if let Some(role) = self.roles.get(name) {
            if role.is_builtin {
                return Err("Cannot delete built-in role".to_string());
            }
        } else {
            return Err("Role not found".to_string());
        }

        self.roles.remove(name);
        Ok(())
    }

    /// Clone a role as a new custom role
    pub fn clone_role(&mut self, source: &str, new_name: &str, new_display_name: &str) -> Result<&Role, String> {
        let source_role = self.roles.get(source)
            .ok_or_else(|| "Source role not found".to_string())?
            .clone();

        if self.roles.contains_key(new_name) {
            return Err("Role with that name already exists".to_string());
        }

        let mut new_role = Role::new(new_name, new_display_name);
        new_role.capabilities = source_role.capabilities;
        new_role.level = source_role.level;

        self.roles.insert(new_name.to_string(), new_role);
        Ok(self.roles.get(new_name).unwrap())
    }

    /// Get a capability
    pub fn get_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.get(name)
    }

    /// Get all capabilities
    pub fn get_capabilities(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    /// Get capabilities by category
    pub fn get_capabilities_by_category(&self, category: &str) -> Vec<&Capability> {
        self.capabilities.values()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Get capability categories
    pub fn get_capability_categories(&self) -> Vec<&str> {
        let mut categories: HashSet<&str> = self.capabilities.values()
            .map(|c| c.category.as_str())
            .collect();
        let mut result: Vec<&str> = categories.drain().collect();
        result.sort();
        result
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_id: i64, role: &str, assigned_by: Option<i64>) -> Result<(), String> {
        if !self.roles.contains_key(role) {
            return Err("Role not found".to_string());
        }

        let mut user_role = UserRole::new(user_id, role);
        if let Some(by) = assigned_by {
            user_role = user_role.assigned_by(by);
        }

        self.user_roles.entry(user_id)
            .or_insert_with(Vec::new)
            .push(user_role);

        Ok(())
    }

    /// Remove role from user
    pub fn remove_role(&mut self, user_id: i64, role: &str) {
        if let Some(roles) = self.user_roles.get_mut(&user_id) {
            roles.retain(|r| r.role != role);
        }
    }

    /// Set user's roles (replace all)
    pub fn set_user_roles(&mut self, user_id: i64, roles: &[&str], assigned_by: Option<i64>) -> Result<(), String> {
        // Validate all roles exist
        for role in roles {
            if !self.roles.contains_key(*role) {
                return Err(format!("Role '{}' not found", role));
            }
        }

        let user_roles: Vec<UserRole> = roles.iter()
            .map(|r| {
                let mut ur = UserRole::new(user_id, r);
                if let Some(by) = assigned_by {
                    ur = ur.assigned_by(by);
                }
                ur
            })
            .collect();

        self.user_roles.insert(user_id, user_roles);
        Ok(())
    }

    /// Get user's roles
    pub fn get_user_roles(&self, user_id: i64) -> Vec<&UserRole> {
        self.user_roles.get(&user_id)
            .map(|roles| roles.iter().filter(|r| !r.is_expired()).collect())
            .unwrap_or_default()
    }

    /// Get user's primary role (highest level)
    pub fn get_user_primary_role(&self, user_id: i64) -> Option<&Role> {
        self.get_user_roles(user_id)
            .iter()
            .filter_map(|ur| self.roles.get(&ur.role))
            .max_by_key(|r| r.level)
    }

    /// Check if user has capability
    pub fn user_can(&self, user_id: i64, capability: &str) -> bool {
        // Check implied capabilities
        let effective_caps = self.get_effective_capabilities(capability);

        self.get_user_roles(user_id)
            .iter()
            .filter_map(|ur| self.roles.get(&ur.role))
            .any(|role| {
                role.capabilities.iter()
                    .any(|c| effective_caps.contains(c))
            })
    }

    /// Get effective capabilities (including implied)
    fn get_effective_capabilities(&self, capability: &str) -> HashSet<String> {
        let mut caps = HashSet::new();
        caps.insert(capability.to_string());

        // Add capabilities that imply this one
        for cap in self.capabilities.values() {
            if cap.implies.contains(&capability.to_string()) {
                caps.insert(cap.name.clone());
            }
        }

        caps
    }

    /// Get all capabilities for a user
    pub fn get_user_capabilities(&self, user_id: i64) -> HashSet<String> {
        let mut caps = HashSet::new();

        for ur in self.get_user_roles(user_id) {
            if let Some(role) = self.roles.get(&ur.role) {
                for cap in &role.capabilities {
                    caps.insert(cap.clone());
                }
            }
        }

        caps
    }

    /// Get default role
    pub fn get_default_role(&self) -> &str {
        &self.default_role
    }

    /// Set default role
    pub fn set_default_role(&mut self, role: &str) -> Result<(), String> {
        if !self.roles.contains_key(role) {
            return Err("Role not found".to_string());
        }
        self.default_role = role.to_string();
        Ok(())
    }

    /// Get users by role
    pub fn get_users_with_role(&self, role: &str) -> Vec<i64> {
        self.user_roles.iter()
            .filter(|(_, roles)| roles.iter().any(|r| r.role == role && !r.is_expired()))
            .map(|(user_id, _)| *user_id)
            .collect()
    }

    /// Count users by role
    pub fn count_users_by_role(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = self.roles.keys()
            .map(|r| (r.clone(), 0))
            .collect();

        for roles in self.user_roles.values() {
            for ur in roles {
                if !ur.is_expired() {
                    if let Some(count) = counts.get_mut(&ur.role) {
                        *count += 1;
                    }
                }
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let mut role = Role::new("custom", "Custom Role");
        role.add_cap("edit_posts");
        role.add_cap("publish_posts");

        assert!(role.has_cap("edit_posts"));
        assert!(role.has_cap("publish_posts"));
        assert!(!role.has_cap("delete_posts"));
    }

    #[test]
    fn test_default_roles() {
        let manager = RoleManager::new();

        assert!(manager.get_role("administrator").is_some());
        assert!(manager.get_role("editor").is_some());
        assert!(manager.get_role("author").is_some());
        assert!(manager.get_role("contributor").is_some());
        assert!(manager.get_role("subscriber").is_some());
    }

    #[test]
    fn test_role_hierarchy() {
        let manager = RoleManager::new();
        let roles = manager.get_roles();

        // Admin should be first (highest level)
        assert_eq!(roles[0].name, "administrator");
    }

    #[test]
    fn test_user_capabilities() {
        let mut manager = RoleManager::new();
        manager.assign_role(1, "editor", None).unwrap();

        assert!(manager.user_can(1, "edit_posts"));
        assert!(manager.user_can(1, "edit_others_posts"));
        assert!(!manager.user_can(1, "manage_options"));
    }

    #[test]
    fn test_custom_role() {
        let mut manager = RoleManager::new();
        manager.create_role("moderator", "Moderator", &["read", "moderate_comments", "edit_posts"]).unwrap();

        let role = manager.get_role("moderator").unwrap();
        assert!(role.has_cap("moderate_comments"));
        assert!(!role.is_builtin);
    }

    #[test]
    fn test_clone_role() {
        let mut manager = RoleManager::new();
        manager.clone_role("editor", "senior_editor", "Senior Editor").unwrap();

        let cloned = manager.get_role("senior_editor").unwrap();
        let original = manager.get_role("editor").unwrap();

        assert_eq!(cloned.capabilities, original.capabilities);
        assert!(!cloned.is_builtin);
    }

    #[test]
    fn test_cannot_delete_builtin() {
        let mut manager = RoleManager::new();
        let result = manager.delete_role("administrator");
        assert!(result.is_err());
    }
}
