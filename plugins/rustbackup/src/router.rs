//! RustBackup Router
//!
//! Axum routes for backup API endpoints.

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::{ApiResponse, Pagination, PaginatedResponse};
use crate::handlers::backups::{BackupHandlers, CreateBackupRequest, RestoreBackupRequest};
use crate::handlers::schedules::ScheduleHandlers;
use crate::handlers::storage::StorageHandlers;
use crate::handlers::logs::LogHandlers;
use crate::models::backup::{Backup, BackupFilters, BackupStats};
use crate::models::schedule::{Schedule, ScheduleRequest, ScheduleFilters};
use crate::models::storage::{StorageProvider, StorageRequest, StorageFilters, StorageTestResult};
use crate::models::log::{BackupLog, LogFilters, LogStats};
use crate::services::restore::{RestoreResult, RestorePreview};
use crate::plugin::RustBackupPlugin;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub plugin: Arc<RustBackupPlugin>,
}

/// Create the backup API router
pub fn create_router(plugin: Arc<RustBackupPlugin>) -> Router {
    let state = AppState { plugin };

    Router::new()
        // Backup routes
        .route("/backups", get(list_backups).post(create_backup))
        .route("/backups/stats", get(backup_stats))
        .route("/backups/quick", post(quick_backup))
        .route("/backups/database", post(backup_database))
        .route("/backups/files", post(backup_files))
        .route("/backups/:id", get(get_backup).delete(delete_backup))
        .route("/backups/:id/restore", post(restore_backup))
        .route("/backups/:id/preview", get(preview_restore))
        .route("/backups/:id/verify", post(verify_backup))
        .route("/backups/:id/upload", post(upload_backup))
        .route("/backups/:id/download", get(download_backup))
        // Schedule routes
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route("/schedules/:id", get(get_schedule).put(update_schedule).delete(delete_schedule))
        .route("/schedules/:id/enable", post(enable_schedule))
        .route("/schedules/:id/disable", post(disable_schedule))
        .route("/schedules/:id/run", post(run_schedule_now))
        // Storage routes
        .route("/storage", get(list_storage).post(create_storage))
        .route("/storage/:id", get(get_storage).put(update_storage).delete(delete_storage))
        .route("/storage/:id/test", post(test_storage))
        .route("/storage/:id/default", post(set_default_storage))
        .route("/storage/:id/files", get(list_storage_files))
        .route("/storage/:id/usage", get(storage_usage))
        // Log routes
        .route("/logs", get(list_logs))
        .route("/logs/stats", get(log_stats))
        .route("/logs/:id", get(get_log))
        .route("/logs/backup/:backup_id", get(backup_logs))
        .route("/logs/cleanup", post(cleanup_logs))
        .with_state(state)
}

// ============================================================================
// Backup Handlers
// ============================================================================

/// List backups with filtering and pagination
async fn list_backups(
    State(state): State<AppState>,
    Query(filters): Query<BackupFilters>,
    Query(pagination): Query<Pagination>,
) -> Json<ApiResponse<PaginatedResponse<Backup>>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.list(filters, pagination).await)
}

/// Get backup by ID
async fn get_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.get(id).await)
}

/// Create a new backup
async fn create_backup(
    State(state): State<AppState>,
    Json(request): Json<CreateBackupRequest>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.create(request.to_backup_request(), Some("api")).await)
}

/// Quick full backup with defaults
async fn quick_backup(
    State(state): State<AppState>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.quick_backup(Some("api")).await)
}

/// Database-only backup
async fn backup_database(
    State(state): State<AppState>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.backup_database(Some("api")).await)
}

/// Files-only backup
async fn backup_files(
    State(state): State<AppState>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.backup_files(Some("api")).await)
}

/// Delete backup
async fn delete_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.delete(id).await)
}

/// Restore from backup
async fn restore_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<RestoreBackupRequest>,
) -> Json<ApiResponse<RestoreResult>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.restore(id, request.to_restore_options()).await)
}

/// Preview restore
async fn preview_restore(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PreviewParams>,
) -> Json<ApiResponse<RestorePreview>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.preview_restore(id, params.password.as_deref()).await)
}

#[derive(Debug, Deserialize)]
struct PreviewParams {
    password: Option<String>,
}

/// Verify backup integrity
async fn verify_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<crate::handlers::backups::VerifyResult>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.verify(id).await)
}

/// Upload backup to remote storage
async fn upload_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(params): Json<UploadParams>,
) -> Json<ApiResponse<String>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.upload_to_remote(id, params.storage_id).await)
}

#[derive(Debug, Deserialize)]
struct UploadParams {
    storage_id: Uuid,
}

/// Download backup from remote storage
async fn download_backup(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<String>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.download_from_remote(id).await)
}

/// Get backup statistics
async fn backup_stats(
    State(state): State<AppState>,
) -> Json<ApiResponse<BackupStats>> {
    let handlers = state.plugin.backup_handlers();
    Json(handlers.stats().await)
}

// ============================================================================
// Schedule Handlers
// ============================================================================

/// List schedules
async fn list_schedules(
    State(state): State<AppState>,
    Query(filters): Query<ScheduleFilters>,
    Query(pagination): Query<Pagination>,
) -> Json<ApiResponse<PaginatedResponse<Schedule>>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.list(filters, pagination).await)
}

/// Get schedule by ID
async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Schedule>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.get(id).await)
}

/// Create a new schedule
async fn create_schedule(
    State(state): State<AppState>,
    Json(request): Json<crate::handlers::schedules::CreateScheduleRequest>,
) -> Json<ApiResponse<Schedule>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.create(request).await)
}

/// Update a schedule
async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<crate::handlers::schedules::UpdateScheduleRequest>,
) -> Json<ApiResponse<Schedule>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.update(id, request).await)
}

/// Delete a schedule
async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.delete(id).await)
}

/// Enable a schedule
async fn enable_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Schedule>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.enable(id).await)
}

/// Disable a schedule
async fn disable_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Schedule>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.disable(id).await)
}

/// Run schedule immediately
async fn run_schedule_now(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Backup>> {
    let handlers = state.plugin.schedule_handlers();
    Json(handlers.run_now(id).await)
}

// ============================================================================
// Storage Handlers
// ============================================================================

/// List storage providers
async fn list_storage(
    State(state): State<AppState>,
    Query(filters): Query<StorageFilters>,
    Query(pagination): Query<Pagination>,
) -> Json<ApiResponse<PaginatedResponse<StorageProvider>>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.list(filters, pagination).await)
}

/// Get storage provider by ID
async fn get_storage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<StorageProvider>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.get(id).await)
}

/// Create a new storage provider
async fn create_storage(
    State(state): State<AppState>,
    Json(request): Json<crate::handlers::storage::CreateStorageRequest>,
) -> Json<ApiResponse<StorageProvider>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.create(request).await)
}

/// Update a storage provider
async fn update_storage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<crate::handlers::storage::UpdateStorageRequest>,
) -> Json<ApiResponse<StorageProvider>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.update(id, request).await)
}

/// Delete a storage provider
async fn delete_storage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.delete(id).await)
}

/// Test storage connection
async fn test_storage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<StorageTestResult>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.test(id).await)
}

/// Set default storage provider
async fn set_default_storage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<StorageProvider>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.set_default(id).await)
}

/// List files in storage
async fn list_storage_files(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListFilesParams>,
) -> Json<ApiResponse<Vec<crate::services::storage::StorageFile>>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.list_files(id, params.prefix.as_deref()).await)
}

#[derive(Debug, Deserialize)]
struct ListFilesParams {
    prefix: Option<String>,
}

/// Get storage usage
async fn storage_usage(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<crate::services::storage::StorageUsage>> {
    let handlers = state.plugin.storage_handlers();
    Json(handlers.get_usage(id).await)
}

// ============================================================================
// Log Handlers
// ============================================================================

/// List logs with filters
async fn list_logs(
    State(state): State<AppState>,
    Query(filters): Query<LogFilters>,
    Query(pagination): Query<Pagination>,
) -> Json<ApiResponse<PaginatedResponse<BackupLog>>> {
    let handlers = state.plugin.log_handlers.clone();
    Json(handlers.list(filters, pagination).await)
}

/// Get log by ID
async fn get_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<BackupLog>> {
    let handlers = state.plugin.log_handlers.clone();
    Json(handlers.get(id).await)
}

/// Get logs for a backup
async fn backup_logs(
    State(state): State<AppState>,
    Path(backup_id): Path<Uuid>,
) -> Json<ApiResponse<Vec<BackupLog>>> {
    let handlers = state.plugin.log_handlers.clone();
    Json(handlers.get_by_backup(backup_id).await)
}

/// Get log statistics
async fn log_stats(
    State(state): State<AppState>,
) -> Json<ApiResponse<LogStats>> {
    let handlers = state.plugin.log_handlers.clone();
    Json(handlers.stats().await)
}

/// Cleanup old logs
async fn cleanup_logs(
    State(state): State<AppState>,
    Json(params): Json<CleanupParams>,
) -> Json<ApiResponse<i64>> {
    let handlers = state.plugin.log_handlers.clone();
    Json(handlers.cleanup(params.days_to_keep.unwrap_or(30)).await)
}

#[derive(Debug, Deserialize)]
struct CleanupParams {
    days_to_keep: Option<i32>,
}

// ============================================================================
// Health Check
// ============================================================================

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub plugin_id: String,
}

/// Create health check router
pub fn health_router(plugin: Arc<RustBackupPlugin>) -> Router {
    Router::new()
        .route("/health", get(|| async move {
            Json(HealthCheck {
                status: "healthy".to_string(),
                version: crate::VERSION.to_string(),
                plugin_id: "rustbackup".to_string(),
            })
        }))
}
