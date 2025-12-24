//! Admin dashboard routes

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

use crate::dbmanager::dbmanager_router;
use crate::handlers::*;

/// Create the admin router with database pool for dbmanager
pub fn admin_router(pool: PgPool) -> Router {
    let state = Arc::new(AdminState::new());

    Router::new()
        // Dashboard
        .route("/", get(dashboard_index))
        .route("/dashboard", get(dashboard_index))

        // System
        .route("/system/status", get(system_status))

        // Database
        .route("/database/status", get(database_status))
        .route("/database/optimize", post(optimize_database))
        .route("/database/migrations", get(migration_status))
        .route("/database/migrations/run", post(run_migrations))

        // Cache
        .route("/cache/status", get(cache_status))
        .route("/cache/purge", post(purge_cache))
        .route("/cache/purge/keys", post(purge_cache_keys))

        // CDN
        .route("/cdn/status", get(cdn_status))
        .route("/cdn/purge", post(cdn_purge_all))
        .route("/cdn/purge/urls", post(cdn_purge_urls))

        // Backups
        .route("/backups", get(list_backups))
        .route("/backups", post(create_backup))
        .route("/backups/:id/restore", post(restore_backup))
        .route("/backups/:id", delete(delete_backup))

        // Settings
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))

        // Logs
        .route("/logs", get(get_logs))
        .route("/logs/clear", post(clear_logs))

        // Activity
        .route("/activity", get(get_activity))

        .with_state(state)
        // Database Manager API - pass the pool
        .nest("/dbmanager/v1", dbmanager_router(pool.clone()))
}

/// Admin API router with prefix
pub fn admin_api_router(prefix: &str, pool: PgPool) -> Router {
    Router::new().nest(prefix, admin_router(pool))
}
