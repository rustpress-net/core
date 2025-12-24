//! Plugin REST API Endpoint Registration
//!
//! Allows plugins to register custom API endpoints.

use crate::manifest::{ApiEndpoint, ApiSection, HttpMethod, RateLimit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info};

/// API registry for plugin endpoints
pub struct ApiRegistry {
    /// Registered routes
    routes: Arc<RwLock<HashMap<String, Vec<RegisteredRoute>>>>,
    /// Namespaces
    namespaces: Arc<RwLock<HashMap<String, NamespaceInfo>>>,
    /// Rate limiters
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiterState>>>,
}

/// Registered API route
#[derive(Debug, Clone)]
pub struct RegisteredRoute {
    pub plugin_id: String,
    pub namespace: String,
    pub path: String,
    pub full_path: String,
    pub method: HttpMethod,
    pub handler: String,
    pub permission: Option<String>,
    pub rate_limit: Option<RateLimit>,
    pub description: Option<String>,
    pub parameters: Vec<RouteParameter>,
    pub responses: Vec<RouteResponse>,
}

/// Route parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteParameter {
    pub name: String,
    pub location: ParameterLocation,
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<String>,
    pub validation: Option<ParameterValidation>,
}

/// Parameter location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
    Body,
    Header,
}

/// Parameter validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<String>>,
}

/// Route response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResponse {
    pub status: u16,
    pub description: String,
    pub content_type: String,
    pub schema: Option<serde_json::Value>,
}

/// Namespace info
#[derive(Debug, Clone)]
pub struct NamespaceInfo {
    pub plugin_id: String,
    pub namespace: String,
    pub version: String,
    pub description: Option<String>,
}

/// Rate limiter state
#[derive(Debug)]
struct RateLimiterState {
    requests: u32,
    window_start: std::time::Instant,
    limit: RateLimit,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            namespaces: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register API from plugin manifest
    pub fn register_from_manifest(
        &self,
        plugin_id: &str,
        section: &ApiSection,
    ) {
        let namespace = section.namespace.clone().unwrap_or_else(|| plugin_id.to_string());
        let version = &section.version;

        // Register namespace
        self.namespaces.write().insert(
            format!("{}/{}", namespace, version),
            NamespaceInfo {
                plugin_id: plugin_id.to_string(),
                namespace: namespace.clone(),
                version: version.clone(),
                description: None,
            },
        );

        // Register endpoints
        for endpoint in &section.endpoints {
            self.register_route(plugin_id, &namespace, version, endpoint);
        }

        debug!(
            "Registered {} API endpoints for plugin: {}",
            section.endpoints.len(),
            plugin_id
        );
    }

    /// Register a single route
    fn register_route(
        &self,
        plugin_id: &str,
        namespace: &str,
        version: &str,
        endpoint: &ApiEndpoint,
    ) {
        let full_path = format!(
            "/wp-json/{}/{}/{}",
            namespace,
            version,
            endpoint.path.trim_start_matches('/')
        );

        let route = RegisteredRoute {
            plugin_id: plugin_id.to_string(),
            namespace: namespace.to_string(),
            path: endpoint.path.clone(),
            full_path: full_path.clone(),
            method: endpoint.method,
            handler: endpoint.handler.clone(),
            permission: endpoint.permission.clone(),
            rate_limit: endpoint.rate_limit.clone(),
            description: endpoint.description.clone(),
            parameters: Vec::new(),
            responses: Vec::new(),
        };

        let mut routes = self.routes.write();
        routes
            .entry(plugin_id.to_string())
            .or_insert_with(Vec::new)
            .push(route);
    }

    /// Register a route programmatically
    pub fn register(
        &self,
        plugin_id: &str,
        namespace: &str,
        path: &str,
        method: HttpMethod,
        handler: &str,
    ) -> RouteBuilder {
        RouteBuilder {
            registry: self,
            plugin_id: plugin_id.to_string(),
            namespace: namespace.to_string(),
            path: path.to_string(),
            method,
            handler: handler.to_string(),
            permission: None,
            rate_limit: None,
            description: None,
            parameters: Vec::new(),
            responses: Vec::new(),
        }
    }

    /// Get all routes for a plugin
    pub fn get_routes(&self, plugin_id: &str) -> Vec<RegisteredRoute> {
        self.routes
            .read()
            .get(plugin_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all routes
    pub fn get_all_routes(&self) -> Vec<RegisteredRoute> {
        self.routes
            .read()
            .values()
            .flatten()
            .cloned()
            .collect()
    }

    /// Find route by path and method
    pub fn find_route(&self, path: &str, method: HttpMethod) -> Option<RegisteredRoute> {
        let routes = self.routes.read();
        for plugin_routes in routes.values() {
            for route in plugin_routes {
                if route.full_path == path && route.method == method {
                    return Some(route.clone());
                }
            }
        }
        None
    }

    /// Match route with path parameters
    pub fn match_route(&self, path: &str, method: HttpMethod) -> Option<(RegisteredRoute, HashMap<String, String>)> {
        let routes = self.routes.read();
        for plugin_routes in routes.values() {
            for route in plugin_routes {
                if route.method == method {
                    if let Some(params) = self.match_path(&route.full_path, path) {
                        return Some((route.clone(), params));
                    }
                }
            }
        }
        None
    }

    /// Match a route pattern against a path
    fn match_path(&self, pattern: &str, path: &str) -> Option<HashMap<String, String>> {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                let param_name = pattern_part.trim_start_matches(':');
                params.insert(param_name.to_string(), path_part.to_string());
            } else if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                let param_name = &pattern_part[1..pattern_part.len() - 1];
                params.insert(param_name.to_string(), path_part.to_string());
            } else if pattern_part != path_part {
                return None;
            }
        }

        Some(params)
    }

    /// Check rate limit
    pub fn check_rate_limit(&self, route_key: &str, rate_limit: &RateLimit) -> bool {
        let mut limiters = self.rate_limiters.write();
        let now = std::time::Instant::now();

        let state = limiters.entry(route_key.to_string()).or_insert_with(|| {
            RateLimiterState {
                requests: 0,
                window_start: now,
                limit: rate_limit.clone(),
            }
        });

        // Check if window has expired
        let window_duration = std::time::Duration::from_secs(state.limit.window_seconds as u64);
        if now.duration_since(state.window_start) > window_duration {
            state.requests = 0;
            state.window_start = now;
        }

        if state.requests >= state.limit.requests {
            return false;
        }

        state.requests += 1;
        true
    }

    /// Unregister all routes for a plugin
    pub fn unregister(&self, plugin_id: &str) {
        self.routes.write().remove(plugin_id);

        // Remove namespace if no other plugin uses it
        let mut namespaces = self.namespaces.write();
        namespaces.retain(|_, info| info.plugin_id != plugin_id);
    }

    /// Get OpenAPI specification
    pub fn get_openapi_spec(&self) -> serde_json::Value {
        let mut paths: HashMap<String, serde_json::Value> = HashMap::new();

        for route in self.get_all_routes() {
            let method_name = match route.method {
                HttpMethod::Get => "get",
                HttpMethod::Post => "post",
                HttpMethod::Put => "put",
                HttpMethod::Patch => "patch",
                HttpMethod::Delete => "delete",
                HttpMethod::Options => "options",
                HttpMethod::Head => "head",
            };

            let operation = serde_json::json!({
                "summary": route.description,
                "operationId": route.handler,
                "tags": [route.namespace],
                "parameters": route.parameters.iter().map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "in": format!("{:?}", p.location).to_lowercase(),
                        "required": p.required,
                        "description": p.description,
                        "schema": { "type": p.param_type }
                    })
                }).collect::<Vec<_>>(),
                "responses": {
                    "200": {
                        "description": "Success"
                    }
                }
            });

            paths
                .entry(route.full_path.clone())
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .unwrap()
                .insert(method_name.to_string(), operation);
        }

        serde_json::json!({
            "openapi": "3.0.0",
            "info": {
                "title": "RustPress Plugin API",
                "version": "1.0.0"
            },
            "paths": paths
        })
    }
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Route builder for fluent API registration
pub struct RouteBuilder<'a> {
    registry: &'a ApiRegistry,
    plugin_id: String,
    namespace: String,
    path: String,
    method: HttpMethod,
    handler: String,
    permission: Option<String>,
    rate_limit: Option<RateLimit>,
    description: Option<String>,
    parameters: Vec<RouteParameter>,
    responses: Vec<RouteResponse>,
}

impl<'a> RouteBuilder<'a> {
    pub fn permission(mut self, permission: &str) -> Self {
        self.permission = Some(permission.to_string());
        self
    }

    pub fn rate_limit(mut self, requests: u32, window_seconds: u32) -> Self {
        self.rate_limit = Some(RateLimit {
            requests,
            window_seconds,
        });
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn param(mut self, name: &str, param_type: &str, location: ParameterLocation) -> Self {
        self.parameters.push(RouteParameter {
            name: name.to_string(),
            location,
            param_type: param_type.to_string(),
            required: false,
            description: None,
            default: None,
            validation: None,
        });
        self
    }

    pub fn required_param(mut self, name: &str, param_type: &str, location: ParameterLocation) -> Self {
        self.parameters.push(RouteParameter {
            name: name.to_string(),
            location,
            param_type: param_type.to_string(),
            required: true,
            description: None,
            default: None,
            validation: None,
        });
        self
    }

    pub fn response(mut self, status: u16, description: &str) -> Self {
        self.responses.push(RouteResponse {
            status,
            description: description.to_string(),
            content_type: "application/json".to_string(),
            schema: None,
        });
        self
    }

    pub fn build(self) {
        let full_path = format!(
            "/wp-json/{}/v1/{}",
            self.namespace,
            self.path.trim_start_matches('/')
        );

        let route = RegisteredRoute {
            plugin_id: self.plugin_id.clone(),
            namespace: self.namespace,
            path: self.path,
            full_path,
            method: self.method,
            handler: self.handler,
            permission: self.permission,
            rate_limit: self.rate_limit,
            description: self.description,
            parameters: self.parameters,
            responses: self.responses,
        };

        let mut routes = self.registry.routes.write();
        routes
            .entry(self.plugin_id)
            .or_insert_with(Vec::new)
            .push(route);
    }
}

/// API response builder
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub meta: Option<ResponseMeta>,
}

/// API error
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize)]
pub struct ResponseMeta {
    pub total: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub links: Option<HashMap<String, String>>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            }),
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_registry() {
        let registry = ApiRegistry::new();

        registry
            .register("test-plugin", "test", "/items", HttpMethod::Get, "get_items")
            .description("Get all items")
            .permission("read")
            .build();

        let routes = registry.get_routes("test-plugin");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, HttpMethod::Get);
    }

    #[test]
    fn test_path_matching() {
        let registry = ApiRegistry::new();

        let params = registry.match_path("/api/users/:id", "/api/users/123");
        assert!(params.is_some());
        let params = params.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_rate_limiting() {
        let registry = ApiRegistry::new();
        let limit = RateLimit {
            requests: 2,
            window_seconds: 60,
        };

        assert!(registry.check_rate_limit("test-route", &limit));
        assert!(registry.check_rate_limit("test-route", &limit));
        assert!(!registry.check_rate_limit("test-route", &limit)); // Should be blocked
    }
}
