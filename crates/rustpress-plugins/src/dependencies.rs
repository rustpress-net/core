//! Plugin Dependency Resolution
//!
//! Resolves plugin dependencies using a graph-based approach.

use crate::manifest::{DependencySpec, PluginManifest};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};

/// Dependency resolver for plugins
pub struct DependencyResolver {
    /// Available plugins with their manifests
    available: HashMap<String, PluginManifest>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            available: HashMap::new(),
        }
    }

    /// Add available plugins
    pub fn add_available(&mut self, manifests: impl IntoIterator<Item = PluginManifest>) {
        for manifest in manifests {
            self.available.insert(manifest.plugin.id.clone(), manifest);
        }
    }

    /// Resolve dependencies for a set of plugins to activate
    pub fn resolve(
        &self,
        plugins_to_activate: &[String],
    ) -> Result<ResolutionResult, ResolutionError> {
        let mut graph: DiGraph<String, DependencyEdge> = DiGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Add nodes for all plugins we want to activate
        for plugin_id in plugins_to_activate {
            if !node_map.contains_key(plugin_id) {
                let idx = graph.add_node(plugin_id.clone());
                node_map.insert(plugin_id.clone(), idx);
            }
        }

        // Process dependencies recursively
        let mut to_process: Vec<String> = plugins_to_activate.to_vec();
        let mut processed: HashSet<String> = HashSet::new();

        while let Some(plugin_id) = to_process.pop() {
            if processed.contains(&plugin_id) {
                continue;
            }
            processed.insert(plugin_id.clone());

            let manifest = match self.available.get(&plugin_id) {
                Some(m) => m,
                None => {
                    if plugins_to_activate.contains(&plugin_id) {
                        errors.push(ResolutionError::PluginNotFound(plugin_id.clone()));
                    }
                    continue;
                }
            };

            // Ensure node exists
            let plugin_node = *node_map.entry(plugin_id.clone()).or_insert_with(|| {
                graph.add_node(plugin_id.clone())
            });

            // Process dependencies
            for (dep_id, dep_spec) in &manifest.dependencies.plugins {
                let (version_req, optional) = match dep_spec {
                    DependencySpec::Version(v) => (v.clone(), false),
                    DependencySpec::Detailed(d) => (d.version.clone(), d.optional),
                };

                // Check if dependency is available
                let dep_manifest = self.available.get(dep_id);

                match dep_manifest {
                    Some(dep) => {
                        // Verify version compatibility
                        let req = match VersionReq::parse(&version_req) {
                            Ok(r) => r,
                            Err(_) => {
                                errors.push(ResolutionError::InvalidVersionReq {
                                    plugin: plugin_id.clone(),
                                    dependency: dep_id.clone(),
                                    version: version_req.clone(),
                                });
                                continue;
                            }
                        };

                        let version = match Version::parse(&dep.plugin.version) {
                            Ok(v) => v,
                            Err(_) => {
                                errors.push(ResolutionError::InvalidVersion {
                                    plugin: dep_id.clone(),
                                    version: dep.plugin.version.clone(),
                                });
                                continue;
                            }
                        };

                        if !req.matches(&version) {
                            if optional {
                                warnings.push(format!(
                                    "Optional dependency {} for {} requires {} but {} is available",
                                    dep_id, plugin_id, version_req, dep.plugin.version
                                ));
                            } else {
                                errors.push(ResolutionError::VersionMismatch {
                                    plugin: plugin_id.clone(),
                                    dependency: dep_id.clone(),
                                    required: version_req,
                                    available: dep.plugin.version.clone(),
                                });
                            }
                            continue;
                        }

                        // Add dependency node and edge
                        let dep_node = *node_map.entry(dep_id.clone()).or_insert_with(|| {
                            graph.add_node(dep_id.clone())
                        });

                        graph.add_edge(plugin_node, dep_node, DependencyEdge {
                            version_req: version_req.clone(),
                            optional,
                        });

                        // Queue dependency for processing
                        if !processed.contains(dep_id) {
                            to_process.push(dep_id.clone());
                        }
                    }
                    None => {
                        if optional {
                            warnings.push(format!(
                                "Optional dependency {} for {} is not available",
                                dep_id, plugin_id
                            ));
                        } else {
                            errors.push(ResolutionError::DependencyNotFound {
                                plugin: plugin_id.clone(),
                                dependency: dep_id.clone(),
                            });
                        }
                    }
                }
            }

            // Check for conflicts
            for conflict_id in &manifest.dependencies.conflicts {
                if self.available.contains_key(conflict_id) && plugins_to_activate.contains(conflict_id) {
                    errors.push(ResolutionError::Conflict {
                        plugin: plugin_id.clone(),
                        conflicts_with: conflict_id.clone(),
                    });
                }
            }
        }

        // Check for cycles using topological sort
        let order = match toposort(&graph, None) {
            Ok(order) => {
                // Reverse the order since we want dependencies first
                order.into_iter()
                    .rev()
                    .filter_map(|idx| graph.node_weight(idx).cloned())
                    .collect()
            }
            Err(cycle) => {
                // Find the cycle
                let cycle_node = graph.node_weight(cycle.node_id()).cloned().unwrap_or_default();
                errors.push(ResolutionError::CyclicDependency(cycle_node));
                Vec::new()
            }
        };

        if !errors.is_empty() {
            return Err(errors.into_iter().next().unwrap());
        }

        Ok(ResolutionResult {
            load_order: order,
            warnings,
            resolved_versions: self.collect_resolved_versions(&node_map),
        })
    }

    /// Collect resolved versions for all plugins
    fn collect_resolved_versions(
        &self,
        node_map: &HashMap<String, NodeIndex>,
    ) -> HashMap<String, String> {
        let mut versions = HashMap::new();
        for plugin_id in node_map.keys() {
            if let Some(manifest) = self.available.get(plugin_id) {
                versions.insert(plugin_id.clone(), manifest.plugin.version.clone());
            }
        }
        versions
    }

    /// Check if a plugin can be safely deactivated
    pub fn can_deactivate(
        &self,
        plugin_id: &str,
        active_plugins: &[String],
    ) -> Result<(), Vec<String>> {
        let mut dependents = Vec::new();

        for active_id in active_plugins {
            if active_id == plugin_id {
                continue;
            }

            if let Some(manifest) = self.available.get(active_id) {
                for (dep_id, spec) in &manifest.dependencies.plugins {
                    if dep_id == plugin_id {
                        let optional = match spec {
                            DependencySpec::Version(_) => false,
                            DependencySpec::Detailed(d) => d.optional,
                        };

                        if !optional {
                            dependents.push(active_id.clone());
                        }
                    }
                }
            }
        }

        if dependents.is_empty() {
            Ok(())
        } else {
            Err(dependents)
        }
    }

    /// Get all dependencies for a plugin (recursive)
    pub fn get_all_dependencies(&self, plugin_id: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        let mut to_process = vec![plugin_id.to_string()];

        while let Some(current) = to_process.pop() {
            if let Some(manifest) = self.available.get(&current) {
                for (dep_id, _) in &manifest.dependencies.plugins {
                    if deps.insert(dep_id.clone()) {
                        to_process.push(dep_id.clone());
                    }
                }
            }
        }

        deps
    }

    /// Get plugins that depend on a specific plugin
    pub fn get_dependents(&self, plugin_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        for (id, manifest) in &self.available {
            if manifest.dependencies.plugins.contains_key(plugin_id) {
                dependents.push(id.clone());
            }
        }

        dependents
    }

    /// Validate all available plugins have satisfiable dependencies
    pub fn validate_all(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (plugin_id, manifest) in &self.available {
            for (dep_id, spec) in &manifest.dependencies.plugins {
                let (version_req, optional) = match spec {
                    DependencySpec::Version(v) => (v.clone(), false),
                    DependencySpec::Detailed(d) => (d.version.clone(), d.optional),
                };

                match self.available.get(dep_id) {
                    Some(dep_manifest) => {
                        if let (Ok(req), Ok(version)) = (
                            VersionReq::parse(&version_req),
                            Version::parse(&dep_manifest.plugin.version),
                        ) {
                            if !req.matches(&version) {
                                issues.push(ValidationIssue {
                                    plugin: plugin_id.clone(),
                                    issue_type: IssueType::VersionMismatch {
                                        dependency: dep_id.clone(),
                                        required: version_req,
                                        available: dep_manifest.plugin.version.clone(),
                                    },
                                    severity: if optional {
                                        IssueSeverity::Warning
                                    } else {
                                        IssueSeverity::Error
                                    },
                                });
                            }
                        }
                    }
                    None => {
                        issues.push(ValidationIssue {
                            plugin: plugin_id.clone(),
                            issue_type: IssueType::MissingDependency(dep_id.clone()),
                            severity: if optional {
                                IssueSeverity::Warning
                            } else {
                                IssueSeverity::Error
                            },
                        });
                    }
                }
            }
        }

        issues
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge data in dependency graph
#[derive(Debug, Clone)]
struct DependencyEdge {
    version_req: String,
    optional: bool,
}

/// Resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Plugins in dependency order (dependencies first)
    pub load_order: Vec<String>,

    /// Warnings during resolution
    pub warnings: Vec<String>,

    /// Resolved versions for each plugin
    pub resolved_versions: HashMap<String, String>,
}

/// Resolution error
#[derive(Debug, Clone, thiserror::Error)]
pub enum ResolutionError {
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Dependency not found: {plugin} requires {dependency}")]
    DependencyNotFound { plugin: String, dependency: String },

    #[error("Version mismatch: {plugin} requires {dependency} {required}, but {available} is available")]
    VersionMismatch {
        plugin: String,
        dependency: String,
        required: String,
        available: String,
    },

    #[error("Invalid version requirement for {plugin} -> {dependency}: {version}")]
    InvalidVersionReq {
        plugin: String,
        dependency: String,
        version: String,
    },

    #[error("Invalid version for plugin {plugin}: {version}")]
    InvalidVersion { plugin: String, version: String },

    #[error("Cyclic dependency detected involving: {0}")]
    CyclicDependency(String),

    #[error("Plugin {plugin} conflicts with {conflicts_with}")]
    Conflict { plugin: String, conflicts_with: String },
}

/// Validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub plugin: String,
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
}

/// Issue type
#[derive(Debug, Clone)]
pub enum IssueType {
    MissingDependency(String),
    VersionMismatch {
        dependency: String,
        required: String,
        available: String,
    },
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Warning,
    Error,
}

/// Lock file for reproducible installations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockFile {
    /// Lock file version
    pub version: u32,

    /// Locked plugins with exact versions
    pub plugins: HashMap<String, LockedPlugin>,

    /// Checksum of the entire lock
    pub checksum: String,

    /// When the lock was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Locked plugin entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockedPlugin {
    pub version: String,
    pub checksum: String,
    pub source: String,
    pub dependencies: Vec<String>,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: 1,
            plugins: HashMap::new(),
            checksum: String::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a lock file from resolution result
    pub fn from_resolution(
        result: &ResolutionResult,
        manifests: &HashMap<String, PluginManifest>,
        checksums: &HashMap<String, String>,
    ) -> Self {
        let mut lock = Self::new();

        for plugin_id in &result.load_order {
            if let Some(manifest) = manifests.get(plugin_id) {
                let deps: Vec<String> = manifest
                    .dependencies
                    .plugins
                    .keys()
                    .cloned()
                    .collect();

                lock.plugins.insert(
                    plugin_id.clone(),
                    LockedPlugin {
                        version: manifest.plugin.version.clone(),
                        checksum: checksums.get(plugin_id).cloned().unwrap_or_default(),
                        source: "local".to_string(),
                        dependencies: deps,
                    },
                );
            }
        }

        lock.update_checksum();
        lock
    }

    /// Update the overall checksum
    fn update_checksum(&mut self) {
        let mut hasher = blake3::Hasher::new();

        let mut plugins: Vec<_> = self.plugins.iter().collect();
        plugins.sort_by_key(|(k, _)| *k);

        for (id, locked) in plugins {
            hasher.update(id.as_bytes());
            hasher.update(locked.version.as_bytes());
            hasher.update(locked.checksum.as_bytes());
        }

        self.checksum = hasher.finalize().to_hex().to_string();
    }

    /// Save to file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Load from file
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Check if lock file matches current installation
    pub fn verify(&self, current_versions: &HashMap<String, String>) -> Vec<LockMismatch> {
        let mut mismatches = Vec::new();

        for (plugin_id, locked) in &self.plugins {
            match current_versions.get(plugin_id) {
                Some(current) => {
                    if current != &locked.version {
                        mismatches.push(LockMismatch {
                            plugin: plugin_id.clone(),
                            mismatch_type: MismatchType::VersionChanged {
                                locked: locked.version.clone(),
                                current: current.clone(),
                            },
                        });
                    }
                }
                None => {
                    mismatches.push(LockMismatch {
                        plugin: plugin_id.clone(),
                        mismatch_type: MismatchType::Missing,
                    });
                }
            }
        }

        // Check for extra plugins
        for plugin_id in current_versions.keys() {
            if !self.plugins.contains_key(plugin_id) {
                mismatches.push(LockMismatch {
                    plugin: plugin_id.clone(),
                    mismatch_type: MismatchType::Unlocked,
                });
            }
        }

        mismatches
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Lock file mismatch
#[derive(Debug, Clone)]
pub struct LockMismatch {
    pub plugin: String,
    pub mismatch_type: MismatchType,
}

/// Mismatch type
#[derive(Debug, Clone)]
pub enum MismatchType {
    Missing,
    Unlocked,
    VersionChanged { locked: String, current: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{PluginMeta, DependencySection};

    fn create_test_manifest(id: &str, version: &str, deps: HashMap<String, DependencySpec>) -> PluginManifest {
        PluginManifest {
            plugin: PluginMeta {
                id: id.to_string(),
                name: id.to_string(),
                version: version.to_string(),
                description: String::new(),
                readme: None,
                homepage: None,
                repository: None,
                license: "MIT".to_string(),
                icon: None,
                banner: None,
                screenshots: Vec::new(),
                category: None,
                tags: Vec::new(),
                min_rustpress_version: None,
                max_rustpress_version: None,
                must_use: false,
                plugin_type: crate::manifest::PluginType::Wasm,
                entry: "plugin.wasm".to_string(),
            },
            author: Default::default(),
            dependencies: DependencySection {
                plugins: deps,
                ..Default::default()
            },
            wordpress: Default::default(),
            permissions: Vec::new(),
            hooks: Default::default(),
            settings: Default::default(),
            migrations: Default::default(),
            assets: Default::default(),
            api: Default::default(),
            admin: Default::default(),
            shortcodes: Vec::new(),
            blocks: Vec::new(),
            widgets: Vec::new(),
            cli: Vec::new(),
            cron: Vec::new(),
            wasm: Default::default(),
            features: HashMap::new(),
            network: Default::default(),
            signing: Default::default(),
        }
    }

    #[test]
    fn test_simple_resolution() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = create_test_manifest("plugin-a", "1.0.0", HashMap::new());
        let mut deps_b = HashMap::new();
        deps_b.insert("plugin-a".to_string(), DependencySpec::Version("^1.0".to_string()));
        let plugin_b = create_test_manifest("plugin-b", "1.0.0", deps_b);

        resolver.add_available(vec![plugin_a, plugin_b]);

        let result = resolver.resolve(&["plugin-b".to_string()]).unwrap();
        assert_eq!(result.load_order, vec!["plugin-a", "plugin-b"]);
    }

    #[test]
    fn test_cyclic_detection() {
        let mut resolver = DependencyResolver::new();

        let mut deps_a = HashMap::new();
        deps_a.insert("plugin-b".to_string(), DependencySpec::Version("^1.0".to_string()));
        let plugin_a = create_test_manifest("plugin-a", "1.0.0", deps_a);

        let mut deps_b = HashMap::new();
        deps_b.insert("plugin-a".to_string(), DependencySpec::Version("^1.0".to_string()));
        let plugin_b = create_test_manifest("plugin-b", "1.0.0", deps_b);

        resolver.add_available(vec![plugin_a, plugin_b]);

        let result = resolver.resolve(&["plugin-a".to_string()]);
        assert!(matches!(result, Err(ResolutionError::CyclicDependency(_))));
    }

    #[test]
    fn test_missing_dependency() {
        let mut resolver = DependencyResolver::new();

        let mut deps = HashMap::new();
        deps.insert("missing-plugin".to_string(), DependencySpec::Version("^1.0".to_string()));
        let plugin = create_test_manifest("plugin-a", "1.0.0", deps);

        resolver.add_available(vec![plugin]);

        let result = resolver.resolve(&["plugin-a".to_string()]);
        assert!(matches!(result, Err(ResolutionError::DependencyNotFound { .. })));
    }
}
