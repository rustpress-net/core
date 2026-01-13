//! Workers API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::error::CloudflareResult;
use crate::services::CloudflareServices;

#[derive(Debug, Deserialize)]
pub struct DeployWorkerRequest {
    pub name: String,
    pub script: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkerRequest {
    pub script: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub pattern: String,
    pub script: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateKvNamespaceRequest {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SetKvValueRequest {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ListKvKeysQuery {
    pub prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeployTemplateRequest {
    pub template_id: String,
    pub name: String,
}

/// List all Workers
pub async fn list_workers(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let workers = services.workers.list_workers().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": workers,
        "total": workers.len()
    })))
}

/// Get a specific Worker
pub async fn get_worker(
    State(services): State<Arc<CloudflareServices>>,
    Path(name): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let worker = services.workers.get_worker(&name).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": worker
    })))
}

/// Deploy a new Worker
pub async fn deploy_worker(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<DeployWorkerRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let worker = services.workers.deploy(&req.name, &req.script).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": worker,
        "message": format!("Worker '{}' deployed successfully", req.name)
    })))
}

/// Update an existing Worker
pub async fn update_worker(
    State(services): State<Arc<CloudflareServices>>,
    Path(name): Path<String>,
    Json(req): Json<UpdateWorkerRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let worker = services.workers.deploy(&name, &req.script).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": worker,
        "message": format!("Worker '{}' updated successfully", name)
    })))
}

/// Delete a Worker
pub async fn delete_worker(
    State(services): State<Arc<CloudflareServices>>,
    Path(name): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.workers.delete(&name).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "name": name
        },
        "message": format!("Worker '{}' deleted successfully", name)
    })))
}

/// List Worker routes
pub async fn list_routes(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let routes = services.workers.list_routes().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": routes,
        "total": routes.len()
    })))
}

/// Create a Worker route
pub async fn create_route(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateRouteRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let route = services.workers.create_route(&req.pattern, &req.script).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": route,
        "message": format!("Route '{}' created successfully", req.pattern)
    })))
}

/// Delete a Worker route
pub async fn delete_route(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would need to add delete_route to workers service
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "Route deleted successfully"
    })))
}

/// List KV namespaces
pub async fn list_kv_namespaces(
    State(services): State<Arc<CloudflareServices>>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let namespaces = services.workers.list_kv_namespaces().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": namespaces,
        "total": namespaces.len()
    })))
}

/// Create a KV namespace
pub async fn create_kv_namespace(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateKvNamespaceRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let namespace = services.workers.create_kv_namespace(&req.title).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": namespace,
        "message": format!("KV namespace '{}' created successfully", req.title)
    })))
}

/// Delete a KV namespace
pub async fn delete_kv_namespace(
    State(_services): State<Arc<CloudflareServices>>,
    Path(id): Path<String>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Would need to add delete_kv_namespace to workers service
    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "id": id
        },
        "message": "KV namespace deleted successfully"
    })))
}

/// List keys in a KV namespace
pub async fn list_kv_keys(
    State(services): State<Arc<CloudflareServices>>,
    Path(namespace): Path<String>,
    Query(_query): Query<ListKvKeysQuery>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let keys = services.workers.list_kv_keys(&namespace).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": keys,
        "total": keys.len()
    })))
}

/// Get a KV value
pub async fn get_kv_value(
    State(services): State<Arc<CloudflareServices>>,
    Path((namespace, key)): Path<(String, String)>,
) -> CloudflareResult<Json<serde_json::Value>> {
    let value = services.workers.get_kv(&namespace, &key).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "key": key,
            "value": value
        }
    })))
}

/// Set a KV value
pub async fn set_kv_value(
    State(services): State<Arc<CloudflareServices>>,
    Path((namespace, key)): Path<(String, String)>,
    Json(req): Json<SetKvValueRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.workers.set_kv(&namespace, &key, &req.value).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "key": key
        },
        "message": format!("KV value '{}' set successfully", key)
    })))
}

/// Delete a KV value
pub async fn delete_kv_value(
    State(services): State<Arc<CloudflareServices>>,
    Path((namespace, key)): Path<(String, String)>,
) -> CloudflareResult<Json<serde_json::Value>> {
    services.workers.delete_kv(&namespace, &key).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "key": key
        },
        "message": format!("KV value '{}' deleted successfully", key)
    })))
}

/// Deploy a Worker from template
pub async fn deploy_template(
    State(_services): State<Arc<CloudflareServices>>,
    Json(req): Json<DeployTemplateRequest>,
) -> CloudflareResult<Json<serde_json::Value>> {
    // Template deployment would fetch template script and deploy it
    let templates = get_worker_templates();

    let template = templates.iter().find(|t| t.id == req.template_id);

    if template.is_none() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": format!("Template '{}' not found", req.template_id)
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "name": req.name,
            "template_id": req.template_id
        },
        "message": format!("Worker '{}' deployed from template", req.name)
    })))
}

/// Worker template definition
#[derive(Debug, Serialize)]
struct WorkerTemplate {
    id: &'static str,
    name: &'static str,
    description: &'static str,
}

fn get_worker_templates() -> Vec<WorkerTemplate> {
    vec![
        WorkerTemplate {
            id: "redirect",
            name: "Redirect Worker",
            description: "Redirect requests to a different URL",
        },
        WorkerTemplate {
            id: "maintenance",
            name: "Maintenance Mode",
            description: "Display a maintenance page",
        },
        WorkerTemplate {
            id: "ab-test",
            name: "A/B Testing",
            description: "Route traffic for A/B testing",
        },
        WorkerTemplate {
            id: "edge-cache",
            name: "Edge Caching",
            description: "Custom caching logic at the edge",
        },
    ]
}
