//! RustBuilder API Module
//!
//! Provides the API routes for the visual page builder.

use axum::{Router, routing::get, extract::State, Json};
use sqlx::PgPool;
use serde::Serialize;

/// API state for RustBuilder
#[derive(Clone)]
pub struct BuilderState {
    pub pool: PgPool,
}

/// Response type for API info
#[derive(Serialize)]
pub struct BuilderInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Get builder info handler
async fn get_info() -> Json<BuilderInfo> {
    Json(BuilderInfo {
        name: super::PLUGIN_NAME,
        version: super::PLUGIN_VERSION,
    })
}

/// Create the RustBuilder router
pub fn create_router(pool: PgPool) -> Router {
    let state = BuilderState { pool };

    Router::new()
        .route("/info", get(get_info))
        .with_state(state)
}
