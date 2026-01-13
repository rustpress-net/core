//! Lazy Loading for Admin Components
//!
//! Implements lazy loading patterns for admin UI components to improve initial load time.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Component metadata for lazy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyComponent {
    /// Unique component identifier
    pub id: String,
    /// Component display name
    pub name: String,
    /// JavaScript module path
    pub module_path: String,
    /// CSS file paths
    pub styles: Vec<String>,
    /// Component dependencies (other component IDs)
    pub dependencies: Vec<String>,
    /// Route patterns where this component is needed
    pub routes: Vec<String>,
    /// Priority for preloading (higher = more important)
    pub priority: u8,
    /// Estimated size in bytes
    pub size_bytes: usize,
    /// Whether to preload on idle
    pub preload_on_idle: bool,
}

/// Chunk configuration for code splitting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Chunk identifier
    pub id: String,
    /// Components in this chunk
    pub components: Vec<String>,
    /// Output filename pattern
    pub filename: String,
    /// Hash for cache busting
    pub hash: String,
    /// Whether this is a critical chunk
    pub critical: bool,
}

/// Lazy loading registry
pub struct LazyLoadingRegistry {
    /// Registered components
    components: Arc<RwLock<HashMap<String, LazyComponent>>>,
    /// Chunk configurations
    chunks: Arc<RwLock<HashMap<String, ChunkConfig>>>,
    /// Route to component mapping
    route_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Loaded components tracking
    loaded: Arc<RwLock<HashSet<String>>>,
}

impl LazyLoadingRegistry {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            route_map: Arc::new(RwLock::new(HashMap::new())),
            loaded: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a lazy component
    pub fn register(&self, component: LazyComponent) {
        // Update route map
        {
            let mut route_map = self.route_map.write();
            for route in &component.routes {
                route_map
                    .entry(route.clone())
                    .or_insert_with(Vec::new)
                    .push(component.id.clone());
            }
        }

        // Store component
        self.components.write().insert(component.id.clone(), component);
    }

    /// Register a chunk
    pub fn register_chunk(&self, chunk: ChunkConfig) {
        self.chunks.write().insert(chunk.id.clone(), chunk);
    }

    /// Get components needed for a route
    pub fn get_components_for_route(&self, route: &str) -> Vec<LazyComponent> {
        let route_map = self.route_map.read();
        let components = self.components.read();

        // Find matching routes
        let mut component_ids: HashSet<String> = HashSet::new();

        for (pattern, ids) in route_map.iter() {
            if route_matches(route, pattern) {
                for id in ids {
                    component_ids.insert(id.clone());
                }
            }
        }

        // Resolve dependencies
        let all_ids = self.resolve_dependencies(&component_ids);

        // Get component details
        all_ids
            .iter()
            .filter_map(|id| components.get(id).cloned())
            .collect()
    }

    /// Resolve component dependencies recursively
    fn resolve_dependencies(&self, ids: &HashSet<String>) -> HashSet<String> {
        let components = self.components.read();
        let mut resolved: HashSet<String> = ids.clone();
        let mut to_process: Vec<String> = ids.iter().cloned().collect();

        while let Some(id) = to_process.pop() {
            if let Some(component) = components.get(&id) {
                for dep in &component.dependencies {
                    if resolved.insert(dep.clone()) {
                        to_process.push(dep.clone());
                    }
                }
            }
        }

        resolved
    }

    /// Get preload hints for a route
    pub fn get_preload_hints(&self, current_route: &str) -> Vec<PreloadHint> {
        let components = self.components.read();
        let loaded = self.loaded.read();
        let route_map = self.route_map.read();

        let mut hints = Vec::new();

        // Get likely next routes based on current route
        let likely_routes = predict_next_routes(current_route);

        for next_route in likely_routes {
            if let Some(component_ids) = route_map.get(&next_route) {
                for id in component_ids {
                    if loaded.contains(id) {
                        continue;
                    }

                    if let Some(component) = components.get(id) {
                        hints.push(PreloadHint {
                            resource: component.module_path.clone(),
                            resource_type: ResourceType::Script,
                            priority: component.priority,
                            crossorigin: Some("anonymous".to_string()),
                        });

                        for style in &component.styles {
                            hints.push(PreloadHint {
                                resource: style.clone(),
                                resource_type: ResourceType::Style,
                                priority: component.priority,
                                crossorigin: None,
                            });
                        }
                    }
                }
            }
        }

        // Sort by priority
        hints.sort_by(|a, b| b.priority.cmp(&a.priority));
        hints
    }

    /// Mark a component as loaded
    pub fn mark_loaded(&self, component_id: &str) {
        self.loaded.write().insert(component_id.to_string());
    }

    /// Check if a component is loaded
    pub fn is_loaded(&self, component_id: &str) -> bool {
        self.loaded.read().contains(component_id)
    }

    /// Get loading status
    pub fn get_loading_status(&self) -> LoadingStatus {
        let components = self.components.read();
        let loaded = self.loaded.read();

        let total = components.len();
        let loaded_count = loaded.len();
        let total_size: usize = components.values().map(|c| c.size_bytes).sum();
        let loaded_size: usize = components
            .values()
            .filter(|c| loaded.contains(&c.id))
            .map(|c| c.size_bytes)
            .sum();

        LoadingStatus {
            total_components: total,
            loaded_components: loaded_count,
            total_size_bytes: total_size,
            loaded_size_bytes: loaded_size,
        }
    }

    /// Generate dynamic import code for a component
    pub fn generate_import_code(&self, component_id: &str) -> Option<String> {
        let components = self.components.read();
        let component = components.get(component_id)?;

        Some(format!(
            r#"const {{ default: {} }} = await import('{}');"#,
            to_pascal_case(&component.name),
            component.module_path
        ))
    }

    /// Generate loader manifest for client
    pub fn generate_manifest(&self) -> LazyLoadManifest {
        let components = self.components.read();
        let chunks = self.chunks.read();
        let route_map = self.route_map.read();

        LazyLoadManifest {
            components: components.values().cloned().collect(),
            chunks: chunks.values().cloned().collect(),
            route_map: route_map.clone(),
        }
    }
}

impl Default for LazyLoadingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Preload hint for browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreloadHint {
    pub resource: String,
    pub resource_type: ResourceType,
    pub priority: u8,
    pub crossorigin: Option<String>,
}

impl PreloadHint {
    /// Generate link header value
    pub fn to_link_header(&self) -> String {
        let rel = match self.resource_type {
            ResourceType::Script => "modulepreload",
            ResourceType::Style => "preload",
            ResourceType::Font => "preload",
            ResourceType::Image => "preload",
        };

        let as_type = match self.resource_type {
            ResourceType::Script => "script",
            ResourceType::Style => "style",
            ResourceType::Font => "font",
            ResourceType::Image => "image",
        };

        let mut header = format!("<{}>; rel=\"{}\"; as=\"{}\"", self.resource, rel, as_type);

        if let Some(ref crossorigin) = self.crossorigin {
            header.push_str(&format!("; crossorigin=\"{}\"", crossorigin));
        }

        header
    }

    /// Generate HTML link element
    pub fn to_html(&self) -> String {
        let rel = match self.resource_type {
            ResourceType::Script => "modulepreload",
            ResourceType::Style => "preload",
            ResourceType::Font => "preload",
            ResourceType::Image => "preload",
        };

        let as_type = match self.resource_type {
            ResourceType::Script => "script",
            ResourceType::Style => "style",
            ResourceType::Font => "font",
            ResourceType::Image => "image",
        };

        let crossorigin = self.crossorigin
            .as_ref()
            .map(|c| format!(" crossorigin=\"{}\"", c))
            .unwrap_or_default();

        format!(
            "<link rel=\"{}\" href=\"{}\" as=\"{}\"{}>",
            rel, self.resource, as_type, crossorigin
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Script,
    Style,
    Font,
    Image,
}

/// Loading status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadingStatus {
    pub total_components: usize,
    pub loaded_components: usize,
    pub total_size_bytes: usize,
    pub loaded_size_bytes: usize,
}

/// Lazy load manifest for client-side loader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadManifest {
    pub components: Vec<LazyComponent>,
    pub chunks: Vec<ChunkConfig>,
    pub route_map: HashMap<String, Vec<String>>,
}

/// Intersection Observer configuration for lazy loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionObserverConfig {
    /// Root margin for triggering load
    pub root_margin: String,
    /// Visibility threshold (0.0-1.0)
    pub threshold: f64,
}

impl Default for IntersectionObserverConfig {
    fn default() -> Self {
        Self {
            root_margin: "100px".to_string(),
            threshold: 0.1,
        }
    }
}

/// Generate JavaScript loader code
pub fn generate_loader_script(config: &IntersectionObserverConfig) -> String {
    format!(
        r#"
class LazyLoader {{
    constructor(manifest) {{
        this.manifest = manifest;
        this.loaded = new Set();
        this.loading = new Map();
        this.observer = null;
        this.init();
    }}

    init() {{
        // Set up intersection observer for viewport-based loading
        this.observer = new IntersectionObserver((entries) => {{
            entries.forEach(entry => {{
                if (entry.isIntersecting) {{
                    const componentId = entry.target.dataset.lazyComponent;
                    if (componentId && !this.loaded.has(componentId)) {{
                        this.load(componentId);
                    }}
                }}
            }});
        }}, {{
            rootMargin: '{}',
            threshold: {}
        }});

        // Observe lazy component placeholders
        document.querySelectorAll('[data-lazy-component]').forEach(el => {{
            this.observer.observe(el);
        }});

        // Preload on idle
        if ('requestIdleCallback' in window) {{
            requestIdleCallback(() => this.preloadIdle());
        }}
    }}

    async load(componentId) {{
        if (this.loaded.has(componentId) || this.loading.has(componentId)) {{
            return this.loading.get(componentId);
        }}

        const component = this.manifest.components.find(c => c.id === componentId);
        if (!component) return null;

        // Load dependencies first
        for (const depId of component.dependencies) {{
            await this.load(depId);
        }}

        // Load styles
        const stylePromises = component.styles.map(href => {{
            return new Promise((resolve, reject) => {{
                const link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = href;
                link.onload = resolve;
                link.onerror = reject;
                document.head.appendChild(link);
            }});
        }});

        // Load module
        const loadPromise = Promise.all([
            import(component.module_path),
            ...stylePromises
        ]).then(([module]) => {{
            this.loaded.add(componentId);
            this.loading.delete(componentId);
            return module;
        }});

        this.loading.set(componentId, loadPromise);
        return loadPromise;
    }}

    preloadIdle() {{
        const toPreload = this.manifest.components
            .filter(c => c.preload_on_idle && !this.loaded.has(c.id))
            .sort((a, b) => b.priority - a.priority);

        toPreload.forEach(component => {{
            const link = document.createElement('link');
            link.rel = 'modulepreload';
            link.href = component.module_path;
            document.head.appendChild(link);
        }});
    }}

    preloadRoute(route) {{
        const componentIds = this.manifest.route_map[route] || [];
        componentIds.forEach(id => this.load(id));
    }}
}}

window.lazyLoader = new LazyLoader(window.__LAZY_MANIFEST__);
"#,
        config.root_margin, config.threshold
    )
}

/// Check if a route matches a pattern
fn route_matches(route: &str, pattern: &str) -> bool {
    if pattern == route {
        return true;
    }

    // Handle wildcard patterns
    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        return route.starts_with(prefix);
    }

    // Handle parameter patterns like /post/:id
    let route_parts: Vec<&str> = route.split('/').collect();
    let pattern_parts: Vec<&str> = pattern.split('/').collect();

    if route_parts.len() != pattern_parts.len() {
        return false;
    }

    route_parts.iter().zip(pattern_parts.iter()).all(|(r, p)| {
        p.starts_with(':') || *r == *p
    })
}

/// Predict likely next routes based on navigation patterns
fn predict_next_routes(current_route: &str) -> Vec<String> {
    // Common navigation patterns in admin interfaces
    match current_route {
        "/admin" | "/admin/" => vec![
            "/admin/posts".to_string(),
            "/admin/pages".to_string(),
            "/admin/comments".to_string(),
        ],
        "/admin/posts" => vec![
            "/admin/posts/new".to_string(),
            "/admin/posts/:id".to_string(),
        ],
        "/admin/pages" => vec![
            "/admin/pages/new".to_string(),
            "/admin/pages/:id".to_string(),
        ],
        "/admin/settings" => vec![
            "/admin/settings/general".to_string(),
            "/admin/settings/writing".to_string(),
            "/admin/settings/reading".to_string(),
        ],
        _ => vec![],
    }
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Admin component registry with common WordPress admin components
pub fn create_admin_component_registry() -> LazyLoadingRegistry {
    let registry = LazyLoadingRegistry::new();

    // Core admin components
    registry.register(LazyComponent {
        id: "dashboard".to_string(),
        name: "Dashboard".to_string(),
        module_path: "/admin/js/components/dashboard.js".to_string(),
        styles: vec!["/admin/css/dashboard.css".to_string()],
        dependencies: vec!["charts".to_string()],
        routes: vec!["/admin".to_string(), "/admin/".to_string()],
        priority: 10,
        size_bytes: 50000,
        preload_on_idle: true,
    });

    registry.register(LazyComponent {
        id: "posts-list".to_string(),
        name: "Posts List".to_string(),
        module_path: "/admin/js/components/posts-list.js".to_string(),
        styles: vec!["/admin/css/posts.css".to_string()],
        dependencies: vec![],
        routes: vec!["/admin/posts".to_string()],
        priority: 8,
        size_bytes: 40000,
        preload_on_idle: true,
    });

    registry.register(LazyComponent {
        id: "post-editor".to_string(),
        name: "Post Editor".to_string(),
        module_path: "/admin/js/components/post-editor.js".to_string(),
        styles: vec![
            "/admin/css/editor.css".to_string(),
            "/admin/css/blocks.css".to_string(),
        ],
        dependencies: vec!["rich-text".to_string(), "media-library".to_string()],
        routes: vec![
            "/admin/posts/new".to_string(),
            "/admin/posts/:id".to_string(),
        ],
        priority: 9,
        size_bytes: 150000,
        preload_on_idle: true,
    });

    registry.register(LazyComponent {
        id: "media-library".to_string(),
        name: "Media Library".to_string(),
        module_path: "/admin/js/components/media-library.js".to_string(),
        styles: vec!["/admin/css/media.css".to_string()],
        dependencies: vec!["uploader".to_string()],
        routes: vec!["/admin/media".to_string()],
        priority: 7,
        size_bytes: 80000,
        preload_on_idle: false,
    });

    registry.register(LazyComponent {
        id: "settings".to_string(),
        name: "Settings".to_string(),
        module_path: "/admin/js/components/settings.js".to_string(),
        styles: vec!["/admin/css/settings.css".to_string()],
        dependencies: vec![],
        routes: vec!["/admin/settings/*".to_string()],
        priority: 5,
        size_bytes: 30000,
        preload_on_idle: false,
    });

    registry.register(LazyComponent {
        id: "charts".to_string(),
        name: "Charts".to_string(),
        module_path: "/admin/js/components/charts.js".to_string(),
        styles: vec![],
        dependencies: vec![],
        routes: vec![],
        priority: 3,
        size_bytes: 100000,
        preload_on_idle: false,
    });

    registry.register(LazyComponent {
        id: "rich-text".to_string(),
        name: "Rich Text Editor".to_string(),
        module_path: "/admin/js/components/rich-text.js".to_string(),
        styles: vec!["/admin/css/rich-text.css".to_string()],
        dependencies: vec![],
        routes: vec![],
        priority: 8,
        size_bytes: 120000,
        preload_on_idle: false,
    });

    registry.register(LazyComponent {
        id: "uploader".to_string(),
        name: "File Uploader".to_string(),
        module_path: "/admin/js/components/uploader.js".to_string(),
        styles: vec!["/admin/css/uploader.css".to_string()],
        dependencies: vec![],
        routes: vec![],
        priority: 6,
        size_bytes: 40000,
        preload_on_idle: false,
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_matching() {
        assert!(route_matches("/admin/posts", "/admin/posts"));
        assert!(route_matches("/admin/posts/123", "/admin/posts/:id"));
        assert!(route_matches("/admin/settings/general", "/admin/settings/*"));
        assert!(!route_matches("/admin/posts", "/admin/pages"));
    }

    #[test]
    fn test_dependency_resolution() {
        let registry = LazyLoadingRegistry::new();

        registry.register(LazyComponent {
            id: "a".to_string(),
            name: "A".to_string(),
            module_path: "/a.js".to_string(),
            styles: vec![],
            dependencies: vec!["b".to_string()],
            routes: vec!["/test".to_string()],
            priority: 1,
            size_bytes: 1000,
            preload_on_idle: false,
        });

        registry.register(LazyComponent {
            id: "b".to_string(),
            name: "B".to_string(),
            module_path: "/b.js".to_string(),
            styles: vec![],
            dependencies: vec!["c".to_string()],
            routes: vec![],
            priority: 1,
            size_bytes: 1000,
            preload_on_idle: false,
        });

        registry.register(LazyComponent {
            id: "c".to_string(),
            name: "C".to_string(),
            module_path: "/c.js".to_string(),
            styles: vec![],
            dependencies: vec![],
            routes: vec![],
            priority: 1,
            size_bytes: 1000,
            preload_on_idle: false,
        });

        let components = registry.get_components_for_route("/test");
        assert_eq!(components.len(), 3);
    }

    #[test]
    fn test_preload_hints() {
        let registry = create_admin_component_registry();
        let hints = registry.get_preload_hints("/admin");

        assert!(!hints.is_empty());
    }
}
