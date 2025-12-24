//! Hook system similar to WordPress actions and filters.
//!
//! Provides a powerful event-driven architecture for extending functionality.

use async_trait::async_trait;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Priority levels for hook execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Priority(pub i32);

impl Priority {
    pub const LOWEST: Priority = Priority(-100);
    pub const LOW: Priority = Priority(-50);
    pub const NORMAL: Priority = Priority(0);
    pub const HIGH: Priority = Priority(50);
    pub const HIGHEST: Priority = Priority(100);
}

impl Default for Priority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Type alias for async action handlers
pub type ActionHandler = Arc<
    dyn Fn(Arc<dyn Any + Send + Sync>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// Type alias for async filter handlers
pub type FilterHandler<T> = Arc<
    dyn Fn(T) -> Pin<Box<dyn Future<Output = T> + Send>>
        + Send
        + Sync,
>;

/// A registered action callback
struct ActionCallback {
    handler: ActionHandler,
    priority: Priority,
    plugin_id: Option<String>,
}

/// A registered filter callback
struct FilterCallback<T: Send + 'static> {
    handler: FilterHandler<T>,
    priority: Priority,
    plugin_id: Option<String>,
}

/// Actions are hooks that perform side effects without modifying data
pub struct Action {
    name: String,
    callbacks: Vec<ActionCallback>,
}

impl Action {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            callbacks: Vec::new(),
        }
    }

    /// Add a callback to this action
    pub fn add<F, Fut>(&mut self, handler: F, priority: Priority, plugin_id: Option<String>)
    where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handler: ActionHandler = Arc::new(move |data| Box::pin(handler(data)));

        self.callbacks.push(ActionCallback {
            handler,
            priority,
            plugin_id,
        });

        // Sort by priority (higher priority executes first)
        self.callbacks.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove callbacks from a specific plugin
    pub fn remove_plugin(&mut self, plugin_id: &str) {
        self.callbacks
            .retain(|cb| cb.plugin_id.as_deref() != Some(plugin_id));
    }

    /// Execute all callbacks
    pub async fn execute(&self, data: Arc<dyn Any + Send + Sync>) {
        for callback in &self.callbacks {
            (callback.handler)(data.clone()).await;
        }
    }

    /// Get the number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }
}

/// Filters are hooks that can modify data as it passes through
pub struct Filter<T: Clone + Send + 'static> {
    name: String,
    callbacks: Vec<FilterCallback<T>>,
}

impl<T: Clone + Send + 'static> Filter<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            callbacks: Vec::new(),
        }
    }

    /// Add a callback to this filter
    pub fn add<F, Fut>(&mut self, handler: F, priority: Priority, plugin_id: Option<String>)
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        let handler: FilterHandler<T> = Arc::new(move |data| Box::pin(handler(data)));

        self.callbacks.push(FilterCallback {
            handler,
            priority,
            plugin_id,
        });

        self.callbacks.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove callbacks from a specific plugin
    pub fn remove_plugin(&mut self, plugin_id: &str) {
        self.callbacks
            .retain(|cb| cb.plugin_id.as_deref() != Some(plugin_id));
    }

    /// Apply all filters to the data
    pub async fn apply(&self, mut data: T) -> T {
        for callback in &self.callbacks {
            data = (callback.handler)(data).await;
        }
        data
    }

    /// Get the number of registered callbacks
    pub fn callback_count(&self) -> usize {
        self.callbacks.len()
    }
}

/// The main hook trait for type-safe hooks
#[async_trait]
pub trait Hook: Send + Sync {
    /// The data type this hook operates on
    type Data: Send + Sync;

    /// The name of this hook
    fn name(&self) -> &str;
}

/// Type-erased action storage
struct ActionStorage {
    actions: HashMap<String, Action>,
}

impl ActionStorage {
    fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }
}

/// Registry for all hooks in the system
pub struct HookRegistry {
    actions: RwLock<ActionStorage>,
    // Filters are stored with type erasure using Any
    filters: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(ActionStorage::new()),
            filters: RwLock::new(HashMap::new()),
        }
    }

    // === Action methods ===

    /// Register an action hook
    pub fn add_action<F, Fut>(
        &self,
        name: &str,
        handler: F,
        priority: Priority,
        plugin_id: Option<String>,
    ) where
        F: Fn(Arc<dyn Any + Send + Sync>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut storage = self.actions.write();
        let action = storage
            .actions
            .entry(name.to_string())
            .or_insert_with(|| Action::new(name));
        action.add(handler, priority, plugin_id);
    }

    /// Execute an action hook
    pub async fn do_action(&self, name: &str, data: Arc<dyn Any + Send + Sync>) {
        let action = {
            let storage = self.actions.read();
            storage.actions.get(name).cloned()
        };

        if let Some(action) = action {
            action.execute(data).await;
        }
    }

    /// Remove all action callbacks from a plugin
    pub fn remove_action_plugin(&self, plugin_id: &str) {
        let mut storage = self.actions.write();
        for action in storage.actions.values_mut() {
            action.remove_plugin(plugin_id);
        }
    }

    /// Check if an action has any callbacks
    pub fn has_action(&self, name: &str) -> bool {
        let storage = self.actions.read();
        storage
            .actions
            .get(name)
            .map(|a| a.callback_count() > 0)
            .unwrap_or(false)
    }

    // === Filter methods ===

    /// Register a filter hook
    pub fn add_filter<T, F, Fut>(
        &self,
        name: &str,
        handler: F,
        priority: Priority,
        plugin_id: Option<String>,
    ) where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        let mut filters = self.filters.write();
        let filter = filters
            .entry(name.to_string())
            .or_insert_with(|| Box::new(RwLock::new(Filter::<T>::new(name))));

        if let Some(filter) = filter.downcast_mut::<RwLock<Filter<T>>>() {
            filter.write().add(handler, priority, plugin_id);
        }
    }

    /// Apply a filter hook
    pub async fn apply_filter<T>(&self, name: &str, data: T) -> T
    where
        T: Clone + Send + Sync + 'static,
    {
        let filter = {
            let filters = self.filters.read();
            filters
                .get(name)
                .and_then(|f| f.downcast_ref::<RwLock<Filter<T>>>())
                .map(|f| f.read().callbacks.clone())
        };

        if let Some(callbacks) = filter {
            let mut result = data;
            for callback in callbacks {
                result = (callback.handler)(result).await;
            }
            result
        } else {
            data
        }
    }

    /// Check if a filter has any callbacks
    pub fn has_filter<T: Clone + Send + Sync + 'static>(&self, name: &str) -> bool {
        let filters = self.filters.read();
        filters
            .get(name)
            .and_then(|f| f.downcast_ref::<RwLock<Filter<T>>>())
            .map(|f| f.read().callback_count() > 0)
            .unwrap_or(false)
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Action {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            callbacks: self.callbacks.iter().map(|cb| ActionCallback {
                handler: cb.handler.clone(),
                priority: cb.priority,
                plugin_id: cb.plugin_id.clone(),
            }).collect(),
        }
    }
}

impl<T: Clone + Send + 'static> Clone for FilterCallback<T> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            priority: self.priority,
            plugin_id: self.plugin_id.clone(),
        }
    }
}

/// Predefined WordPress-like hooks
pub mod hooks {
    /// Called when a post is created
    pub const POST_CREATED: &str = "post_created";
    /// Called before a post is saved
    pub const PRE_POST_SAVE: &str = "pre_post_save";
    /// Called after a post is saved
    pub const POST_SAVED: &str = "post_saved";
    /// Called before a post is deleted
    pub const PRE_POST_DELETE: &str = "pre_post_delete";
    /// Called after a post is deleted
    pub const POST_DELETED: &str = "post_deleted";

    /// Called when a user is created
    pub const USER_CREATED: &str = "user_created";
    /// Called when a user logs in
    pub const USER_LOGIN: &str = "user_login";
    /// Called when a user logs out
    pub const USER_LOGOUT: &str = "user_logout";

    /// Called when a comment is created
    pub const COMMENT_CREATED: &str = "comment_created";
    /// Called before a comment is approved
    pub const PRE_COMMENT_APPROVE: &str = "pre_comment_approve";

    /// Called when a plugin is activated
    pub const PLUGIN_ACTIVATED: &str = "plugin_activated";
    /// Called when a plugin is deactivated
    pub const PLUGIN_DEACTIVATED: &str = "plugin_deactivated";

    /// Called at the start of each request
    pub const REQUEST_START: &str = "request_start";
    /// Called at the end of each request
    pub const REQUEST_END: &str = "request_end";

    /// Filter: Modify post content before saving
    pub const FILTER_POST_CONTENT: &str = "filter_post_content";
    /// Filter: Modify post title before saving
    pub const FILTER_POST_TITLE: &str = "filter_post_title";
    /// Filter: Modify rendered content
    pub const FILTER_THE_CONTENT: &str = "filter_the_content";
    /// Filter: Modify user capabilities
    pub const FILTER_USER_CAPS: &str = "filter_user_caps";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[tokio::test]
    async fn test_action_execution() {
        let mut action = Action::new("test_action");
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = counter.clone();

        action.add(
            move |_| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            },
            Priority::NORMAL,
            None,
        );

        action.execute(Arc::new(())).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_action_priority() {
        let mut action = Action::new("priority_test");
        let order = Arc::new(RwLock::new(Vec::new()));

        let order1 = order.clone();
        action.add(
            move |_| {
                let o = order1.clone();
                async move {
                    o.write().push(1);
                }
            },
            Priority::LOW,
            None,
        );

        let order2 = order.clone();
        action.add(
            move |_| {
                let o = order2.clone();
                async move {
                    o.write().push(2);
                }
            },
            Priority::HIGH,
            None,
        );

        action.execute(Arc::new(())).await;
        assert_eq!(*order.read(), vec![2, 1]); // High priority first
    }

    #[tokio::test]
    async fn test_filter_apply() {
        let mut filter: Filter<String> = Filter::new("test_filter");

        filter.add(
            |s| async move { format!("{}_modified", s) },
            Priority::NORMAL,
            None,
        );

        let result = filter.apply("test".to_string()).await;
        assert_eq!(result, "test_modified");
    }

    #[tokio::test]
    async fn test_filter_chain() {
        let mut filter: Filter<i32> = Filter::new("chain_test");

        filter.add(|n| async move { n + 1 }, Priority::HIGH, None);
        filter.add(|n| async move { n * 2 }, Priority::NORMAL, None);

        // High priority runs first: (1 + 1) * 2 = 4
        let result = filter.apply(1).await;
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_hook_registry() {
        let registry = HookRegistry::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = counter.clone();

        registry.add_action(
            "test",
            move |_| {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            },
            Priority::NORMAL,
            None,
        );

        assert!(registry.has_action("test"));
        registry.do_action("test", Arc::new(())).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
