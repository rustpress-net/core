//! D1 Database API endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::{D1Database, D1QueryResult};
use crate::services::CloudflareServices;

/// List D1 databases response
#[derive(Debug, Serialize)]
pub struct ListDatabasesResponse {
    pub databases: Vec<D1Database>,
}

/// Create database request
#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
}

/// Query request
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub sql: String,
}

/// Batch query request
#[derive(Debug, Deserialize)]
pub struct BatchQueryRequest {
    pub statements: Vec<String>,
}

/// Table schema response
#[derive(Debug, Serialize)]
pub struct TableSchemaResponse {
    pub table_name: String,
    pub columns: Vec<serde_json::Value>,
}

/// List all D1 databases
pub async fn list_databases(
    State(services): State<Arc<CloudflareServices>>,
) -> Result<Json<ListDatabasesResponse>, (StatusCode, String)> {
    let databases = services
        .d1
        .list_databases()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ListDatabasesResponse { databases }))
}

/// Get a database by ID
pub async fn get_database(
    State(services): State<Arc<CloudflareServices>>,
    Path(database_id): Path<String>,
) -> Result<Json<D1Database>, (StatusCode, String)> {
    let database = services
        .d1
        .get_database(&database_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok(Json(database))
}

/// Create a new database
pub async fn create_database(
    State(services): State<Arc<CloudflareServices>>,
    Json(req): Json<CreateDatabaseRequest>,
) -> Result<Json<D1Database>, (StatusCode, String)> {
    let database = services
        .d1
        .create_database(&req.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(database))
}

/// Execute a SQL query
pub async fn execute_query(
    State(services): State<Arc<CloudflareServices>>,
    Path(database_id): Path<String>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<D1QueryResult>, (StatusCode, String)> {
    let result = services
        .d1
        .execute_query(&database_id, &req.sql)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// Execute batch queries
pub async fn execute_batch(
    State(services): State<Arc<CloudflareServices>>,
    Path(database_id): Path<String>,
    Json(req): Json<BatchQueryRequest>,
) -> Result<Json<Vec<D1QueryResult>>, (StatusCode, String)> {
    let results = services
        .d1
        .execute_batch(&database_id, req.statements)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

/// List tables in a database
pub async fn list_tables(
    State(services): State<Arc<CloudflareServices>>,
    Path(database_id): Path<String>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let tables = services
        .d1
        .list_tables(&database_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tables))
}

/// Get table schema
pub async fn get_table_schema(
    State(services): State<Arc<CloudflareServices>>,
    Path((database_id, table_name)): Path<(String, String)>,
) -> Result<Json<TableSchemaResponse>, (StatusCode, String)> {
    let columns = services
        .d1
        .get_table_schema(&database_id, &table_name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TableSchemaResponse {
        table_name,
        columns,
    }))
}
