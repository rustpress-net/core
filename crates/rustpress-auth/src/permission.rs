//! Permission and role management.

use rustpress_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A permission for a specific action on a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Create a wildcard permission for a resource
    pub fn all(resource: impl Into<String>) -> Self {
        Self::new(resource, "*")
    }

    /// Create a super admin permission
    pub fn super_admin() -> Self {
        Self::new("*", "*")
    }

    /// Check if this permission matches another
    pub fn matches(&self, other: &Permission) -> bool {
        let resource_match = self.resource == "*" || self.resource == other.resource;
        let action_match = self.action == "*" || self.action == other.action;
        resource_match && action_match
    }

    /// Check if this permission covers another
    pub fn covers(&self, other: &Permission) -> bool {
        self.matches(other)
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource, self.action)
    }
}

impl std::str::FromStr for Permission {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.as_slice() {
            [resource, action] => Ok(Permission::new(*resource, *action)),
            _ => Err(Error::InvalidInput {
                field: "permission".to_string(),
                message: "Permission must be in format 'resource:action'".to_string(),
            }),
        }
    }
}

/// Predefined permissions
pub mod permissions {
    use super::Permission;

    // Posts
    pub const POSTS_READ: Permission = Permission { resource: String::new(), action: String::new() };

    pub fn posts_read() -> Permission {
        Permission::new("posts", "read")
    }
    pub fn posts_create() -> Permission {
        Permission::new("posts", "create")
    }
    pub fn posts_edit() -> Permission {
        Permission::new("posts", "edit")
    }
    pub fn posts_delete() -> Permission {
        Permission::new("posts", "delete")
    }
    pub fn posts_publish() -> Permission {
        Permission::new("posts", "publish")
    }
    pub fn posts_manage() -> Permission {
        Permission::all("posts")
    }

    // Pages
    pub fn pages_read() -> Permission {
        Permission::new("pages", "read")
    }
    pub fn pages_create() -> Permission {
        Permission::new("pages", "create")
    }
    pub fn pages_edit() -> Permission {
        Permission::new("pages", "edit")
    }
    pub fn pages_delete() -> Permission {
        Permission::new("pages", "delete")
    }
    pub fn pages_manage() -> Permission {
        Permission::all("pages")
    }

    // Users
    pub fn users_read() -> Permission {
        Permission::new("users", "read")
    }
    pub fn users_create() -> Permission {
        Permission::new("users", "create")
    }
    pub fn users_edit() -> Permission {
        Permission::new("users", "edit")
    }
    pub fn users_delete() -> Permission {
        Permission::new("users", "delete")
    }
    pub fn users_manage() -> Permission {
        Permission::all("users")
    }

    // Media
    pub fn media_read() -> Permission {
        Permission::new("media", "read")
    }
    pub fn media_upload() -> Permission {
        Permission::new("media", "upload")
    }
    pub fn media_edit() -> Permission {
        Permission::new("media", "edit")
    }
    pub fn media_delete() -> Permission {
        Permission::new("media", "delete")
    }
    pub fn media_manage() -> Permission {
        Permission::all("media")
    }

    // Comments
    pub fn comments_read() -> Permission {
        Permission::new("comments", "read")
    }
    pub fn comments_create() -> Permission {
        Permission::new("comments", "create")
    }
    pub fn comments_moderate() -> Permission {
        Permission::new("comments", "moderate")
    }
    pub fn comments_delete() -> Permission {
        Permission::new("comments", "delete")
    }

    // Settings
    pub fn settings_read() -> Permission {
        Permission::new("settings", "read")
    }
    pub fn settings_edit() -> Permission {
        Permission::new("settings", "edit")
    }

    // Plugins
    pub fn plugins_read() -> Permission {
        Permission::new("plugins", "read")
    }
    pub fn plugins_manage() -> Permission {
        Permission::all("plugins")
    }

    // Themes
    pub fn themes_read() -> Permission {
        Permission::new("themes", "read")
    }
    pub fn themes_manage() -> Permission {
        Permission::all("themes")
    }
}

/// A role with a set of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: HashSet<Permission>,
    pub inherits_from: Vec<String>,
}

impl Role {
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            description: None,
            permissions: HashSet::new(),
            inherits_from: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.insert(permission);
        self
    }

    pub fn with_permissions(mut self, permissions: impl IntoIterator<Item = Permission>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    pub fn inherits(mut self, role_name: impl Into<String>) -> Self {
        self.inherits_from.push(role_name.into());
        self
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.covers(permission))
    }
}

/// Predefined roles
pub mod roles {
    use super::*;
    use permissions::*;

    /// Administrator with full access
    pub fn administrator() -> Role {
        Role::new("administrator", "Administrator")
            .with_description("Full access to all features")
            .with_permission(Permission::super_admin())
    }

    /// Editor can manage all content
    pub fn editor() -> Role {
        Role::new("editor", "Editor")
            .with_description("Can manage all posts and pages")
            .with_permissions([
                posts_manage(),
                pages_manage(),
                media_manage(),
                comments_moderate(),
                users_read(),
            ])
    }

    /// Author can create and manage own content
    pub fn author() -> Role {
        Role::new("author", "Author")
            .with_description("Can create and edit own posts")
            .with_permissions([
                posts_create(),
                posts_edit(),
                posts_publish(),
                media_upload(),
                media_edit(),
                comments_read(),
            ])
    }

    /// Contributor can create content but not publish
    pub fn contributor() -> Role {
        Role::new("contributor", "Contributor")
            .with_description("Can create posts but not publish")
            .with_permissions([posts_create(), posts_edit(), media_upload()])
    }

    /// Subscriber can read and comment
    pub fn subscriber() -> Role {
        Role::new("subscriber", "Subscriber")
            .with_description("Can read content and post comments")
            .with_permissions([posts_read(), pages_read(), comments_read(), comments_create()])
    }
}

/// Permission checker with role hierarchy support
pub struct PermissionChecker {
    roles: HashMap<String, Role>,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Create with default WordPress-like roles
    pub fn with_default_roles() -> Self {
        let mut checker = Self::new();
        checker.register_role(roles::administrator());
        checker.register_role(roles::editor());
        checker.register_role(roles::author());
        checker.register_role(roles::contributor());
        checker.register_role(roles::subscriber());
        checker
    }

    /// Register a role
    pub fn register_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Get a role by name
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Check if a role has a specific permission
    pub fn role_has_permission(&self, role_name: &str, permission: &Permission) -> bool {
        self.get_all_permissions(role_name)
            .iter()
            .any(|p| p.covers(permission))
    }

    /// Get all permissions for a role (including inherited)
    pub fn get_all_permissions(&self, role_name: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_permissions(role_name, &mut permissions, &mut visited);
        permissions
    }

    fn collect_permissions(
        &self,
        role_name: &str,
        permissions: &mut HashSet<Permission>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(role_name) {
            return; // Prevent cycles
        }
        visited.insert(role_name.to_string());

        if let Some(role) = self.roles.get(role_name) {
            permissions.extend(role.permissions.clone());

            for parent in &role.inherits_from {
                self.collect_permissions(parent, permissions, visited);
            }
        }
    }

    /// Check if user with given roles can perform action
    pub fn can(
        &self,
        user_roles: &[String],
        resource: &str,
        action: &str,
    ) -> bool {
        let permission = Permission::new(resource, action);
        user_roles
            .iter()
            .any(|role| self.role_has_permission(role, &permission))
    }

    /// List all available roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::with_default_roles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        let admin = Permission::super_admin();
        let posts_read = Permission::new("posts", "read");

        assert!(admin.covers(&posts_read));
        assert!(!posts_read.covers(&admin));

        let posts_all = Permission::all("posts");
        assert!(posts_all.covers(&posts_read));
    }

    #[test]
    fn test_permission_parsing() {
        let p: Permission = "posts:read".parse().unwrap();
        assert_eq!(p.resource, "posts");
        assert_eq!(p.action, "read");
    }

    #[test]
    fn test_role_permissions() {
        let admin = roles::administrator();
        assert!(admin.has_permission(&Permission::new("posts", "delete")));

        let subscriber = roles::subscriber();
        assert!(subscriber.has_permission(&permissions::posts_read()));
        assert!(!subscriber.has_permission(&permissions::posts_delete()));
    }

    #[test]
    fn test_permission_checker() {
        let checker = PermissionChecker::with_default_roles();

        assert!(checker.can(&["administrator".to_string()], "posts", "delete"));
        assert!(checker.can(&["editor".to_string()], "posts", "edit"));
        assert!(!checker.can(&["subscriber".to_string()], "posts", "delete"));
    }

    #[test]
    fn test_role_inheritance() {
        let mut checker = PermissionChecker::new();

        let base = Role::new("base", "Base")
            .with_permission(Permission::new("resource", "read"));

        let extended = Role::new("extended", "Extended")
            .inherits("base")
            .with_permission(Permission::new("resource", "write"));

        checker.register_role(base);
        checker.register_role(extended);

        let permissions = checker.get_all_permissions("extended");
        assert!(permissions.contains(&Permission::new("resource", "read")));
        assert!(permissions.contains(&Permission::new("resource", "write")));
    }
}
