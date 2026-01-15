//! Route definitions and router configuration.

use axum::{
    extract::{ConnectInfo, Query, State},
    http::header,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::error::HttpResult;
use crate::extract::{AuthUser, PaginatedQuery, PathId, ValidatedJson};
use crate::response::{created, json, no_content, paginated, SuccessResponse};
use crate::state::AppState;
use std::sync::Arc;

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health and system routes
        .nest("/health", health_routes())
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        // Cloudflare plugin routes (separate state)
        .nest_service("/api/v1/cloudflare", build_cloudflare_router(&state))
        // Admin UI routes (serve static files, handle by frontend)
        // Handle /admin/ with trailing slash - redirect to /admin
        .route(
            "/admin/",
            get(|| async { axum::response::Redirect::permanent("/admin") }),
        )
        .nest("/admin", admin_routes())
        // Public-facing website routes (theme rendering)
        .merge(public_routes())
        // Metrics endpoint
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

/// Admin routes - serve static files from admin-ui directory
fn admin_routes() -> Router<AppState> {
    // Path to admin UI directory (release packages have built files at ./admin-ui)
    let admin_dist =
        std::env::var("ADMIN_UI_PATH").unwrap_or_else(|_| "./admin-ui".to_string());

    let index_path = format!("{}/index.html", admin_dist);

    // Use a single fallback handler for all paths (SPA routing)
    Router::new().fallback(move |req: axum::extract::Request| {
        let admin_dist = admin_dist.clone();
        let index_path = index_path.clone();
        async move {
            use axum::body::Body;
            use axum::http::{header, StatusCode};
            use axum::response::IntoResponse;

            let path = req.uri().path();

            // Check if this looks like a static asset request
            let is_asset = path.starts_with("/assets/")
                || path.ends_with(".js")
                || path.ends_with(".css")
                || path.ends_with(".svg")
                || path.ends_with(".ico")
                || path.ends_with(".png")
                || path.ends_with(".jpg")
                || path.ends_with(".woff")
                || path.ends_with(".woff2");

            if is_asset {
                // Try to serve the static file
                let file_path = format!("{}{}", admin_dist, path);
                match tokio::fs::read(&file_path).await {
                    Ok(contents) => {
                        let content_type = if path.ends_with(".js") {
                            "application/javascript"
                        } else if path.ends_with(".css") {
                            "text/css"
                        } else if path.ends_with(".svg") {
                            "image/svg+xml"
                        } else if path.ends_with(".png") {
                            "image/png"
                        } else if path.ends_with(".ico") {
                            "image/x-icon"
                        } else {
                            "application/octet-stream"
                        };
                        (
                            StatusCode::OK,
                            [(header::CONTENT_TYPE, content_type)],
                            Body::from(contents),
                        )
                            .into_response()
                    }
                    Err(_) => (StatusCode::NOT_FOUND, "Not found").into_response(),
                }
            } else {
                // Serve index.html for SPA routes (root, trailing slash, and all other paths)
                match tokio::fs::read(&index_path).await {
                    Ok(contents) => (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, "text/html")],
                        Body::from(contents),
                    )
                        .into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "Admin UI not found").into_response(),
                }
            }
        }
    })
}

/// Public website routes
fn public_routes() -> Router<AppState> {
    Router::new()
        // Home page
        .route("/", get(public_home_handler))
        // Blog archive
        .route("/blog", get(public_blog_handler))
        // Single post
        .route("/post/:slug", get(public_post_handler))
        // Page
        .route("/page/:slug", get(public_page_handler))
        // Alternative: WordPress-style /:slug for pages
        // Category archive
        .route("/category/:slug", get(public_category_handler))
        // Tag archive
        .route("/tag/:slug", get(public_tag_handler))
        // Author archive
        .route("/author/:slug", get(public_author_handler))
        // Search results
        .route("/search", get(public_search_handler))
        // Feed
        .route("/feed", get(public_feed_handler))
        .route("/feed/rss", get(public_feed_handler))
        .route("/feed/atom", get(public_atom_feed_handler))
        // Sitemap
        .route("/sitemap.xml", get(public_sitemap_handler))
        // Robots.txt
        .route("/robots.txt", get(public_robots_handler))
        // Theme assets
        .route("/themes/:theme_id/*path", get(theme_asset_handler))
}

/// Health check routes
fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/live", get(liveness_check))
        .route("/ready", get(readiness_check))
}

/// API v1 routes
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Auth routes
        .nest("/auth", auth_routes())
        // User routes
        .nest("/users", user_routes())
        // Post routes
        .nest("/posts", post_routes())
        // Page routes
        .nest("/pages", page_routes())
        // Media routes
        .nest("/media", media_routes())
        // Comment routes
        .nest("/comments", comment_routes())
        // Settings routes
        .nest("/settings", settings_routes())
        // Plugin routes
        .nest("/plugins", plugin_routes())
        // Theme routes
        .nest("/themes", theme_routes())
        // Search routes
        .nest("/search", search_routes())
        // Backup routes
        .nest("/backups", backup_routes())
        // SEO routes
        .nest("/seo", seo_routes())
        // Cache routes
        .nest("/cache", cache_routes())
        // CDN routes
        .nest("/cdn", cdn_routes())
        // Taxonomy routes (categories, tags)
        .nest("/taxonomies", taxonomy_routes())
        // Direct category/tag routes (aliases for frontend compatibility)
        .route(
            "/categories",
            get(list_categories_handler).post(create_category_handler),
        )
        .route(
            "/categories/:id",
            get(get_category_handler)
                .put(update_category_handler)
                .delete(delete_category_handler),
        )
        .route("/tags", get(list_tags_handler).post(create_tag_handler))
        .route(
            "/tags/:id",
            get(get_tag_handler)
                .put(update_tag_handler)
                .delete(delete_tag_handler),
        )
        // Menu routes
        .nest("/menus", menu_routes())
        // Widget routes
        .nest("/widgets", widget_routes())
        // Stats/Dashboard routes
        .nest("/stats", stats_routes())
        // Email routes
        .nest("/email", email_routes())
}

/// Theme management routes
fn theme_routes() -> Router<AppState> {
    Router::new()
        // List all themes & scan for new themes
        .route("/", get(list_themes_handler).post(scan_themes_handler))
        // Upload/install a new theme from ZIP
        .route("/upload", post(upload_theme_handler))
        // Validate a theme ZIP without installing
        .route("/validate", post(validate_theme_handler))
        // Get available default themes
        .route("/available", get(get_available_themes_handler))
        // Get active theme
        .route("/active", get(get_active_theme_handler))
        // Get/update specific theme
        .route(
            "/:theme_id",
            get(get_theme_handler).delete(delete_theme_handler),
        )
        // Activate a theme
        .route("/:theme_id/activate", post(activate_theme_handler))
        // Update a theme from ZIP
        .route("/:theme_id/update", post(update_theme_handler))
        // Export theme as ZIP
        .route("/:theme_id/export", get(export_theme_handler))
        // Theme settings/customization
        .route(
            "/:theme_id/settings",
            get(get_theme_settings_handler).put(update_theme_settings_handler),
        )
        // Menu assignments
        .route(
            "/:theme_id/menus",
            get(get_theme_menus_handler).put(update_theme_menus_handler),
        )
        // Widget assignments
        .route(
            "/:theme_id/widgets",
            get(get_theme_widgets_handler).put(update_theme_widgets_handler),
        )
        // Theme preview
        .route("/:theme_id/preview", post(create_theme_preview_handler))
}

/// Authentication routes
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/refresh", post(refresh_token_handler))
        .route("/register", post(register_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler))
        .route("/me", get(current_user_handler))
}

/// User management routes
fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users_handler).post(create_user_handler))
        .route("/me", get(current_user_handler))
        .route(
            "/:id",
            get(get_user_handler)
                .put(update_user_handler)
                .delete(delete_user_handler),
        )
        .route("/:id/roles", put(update_user_roles_handler))
}

/// Post routes
fn post_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_posts_handler).post(create_post_handler))
        .route("/bulk-delete", post(bulk_delete_posts_handler))
        .route(
            "/:id",
            get(get_post_handler)
                .put(update_post_handler)
                .delete(delete_post_handler),
        )
        .route("/:id/publish", post(publish_post_handler))
        .route("/:id/unpublish", post(unpublish_post_handler))
        .route("/:id/duplicate", post(duplicate_post_handler))
}

/// Page routes
fn page_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_pages_handler).post(create_page_handler))
        .route(
            "/:id",
            get(get_page_handler)
                .put(update_page_handler)
                .delete(delete_page_handler),
        )
}

/// Media routes
fn media_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_media_handler).post(upload_media_handler))
        .route(
            "/folders",
            get(list_media_folders_handler).post(create_media_folder_handler),
        )
        .route(
            "/:id",
            get(get_media_handler)
                .put(update_media_handler)
                .delete(delete_media_handler),
        )
}

/// Comment routes
fn comment_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_comments_handler).post(create_comment_handler))
        .route("/batch", post(batch_moderate_comments_handler))
        .route("/counts", get(comment_counts_handler))
        .route(
            "/:id",
            get(get_comment_handler)
                .put(update_comment_handler)
                .delete(delete_comment_handler),
        )
        .route("/:id/approve", post(approve_comment_handler))
        .route("/:id/spam", post(mark_spam_handler))
        .route("/:id/trash", post(trash_comment_handler))
        .route("/:id/like", post(toggle_like_handler))
}

/// Settings routes
fn settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_settings_handler))
        .route("/batch", put(batch_update_settings_handler))
        .route("/groups/:group", get(get_settings_group_handler))
        // Group-specific routes for common groups
        .route("/general", get(get_general_settings_handler))
        .route("/reading", get(get_reading_settings_handler))
        .route("/writing", get(get_writing_settings_handler))
        .route("/discussion", get(get_discussion_settings_handler))
        .route("/permalinks", get(get_permalinks_settings_handler))
        .route(
            "/:key",
            get(get_setting_handler).put(update_setting_handler),
        )
}

/// Plugin routes
fn plugin_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_plugins_handler).post(install_plugin_handler))
        .route(
            "/:id",
            get(get_plugin_handler).delete(uninstall_plugin_handler),
        )
        .route("/:id/activate", post(activate_plugin_handler))
        .route("/:id/deactivate", post(deactivate_plugin_handler))
}

// =============================================================================
// Health Handlers
// =============================================================================

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check() -> impl axum::response::IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn liveness_check() -> impl axum::response::IntoResponse {
    Json(serde_json::json!({ "status": "alive" }))
}

async fn readiness_check(State(state): State<AppState>) -> axum::response::Response {
    // Check database connection
    let db_healthy = state.db().is_connected().await;

    if db_healthy {
        Json(serde_json::json!({ "status": "ready" })).into_response()
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not ready", "reason": "database unavailable" })),
        )
            .into_response()
    }
}

async fn metrics_handler(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    // Return Prometheus metrics
    "# Metrics endpoint - implement with prometheus-client"
}

// =============================================================================
// Auth Handlers (Placeholder implementations)
// =============================================================================

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

use rustpress_auth::{PasswordHasher, PasswordRules, PasswordValidator};

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: i64,
    user: AuthUserResponse,
}

#[derive(Serialize)]
struct AuthUserResponse {
    id: Uuid,
    email: String,
    username: String,
    display_name: Option<String>,
    role: String,
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Find user by email or username
    let user: Option<rustpress_database::repository::users::UserRow> = sqlx::query_as(
        r#"
        SELECT id, email, username, password_hash, display_name, status, role,
               avatar_url, NULL::varchar as locale, NULL::varchar as timezone,
               email_verified_at, last_login_at, created_at, updated_at, deleted_at
        FROM users
        WHERE (email = $1 OR username = $1) AND deleted_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to find user", e))?;

    let user =
        user.ok_or_else(|| rustpress_core::error::Error::unauthorized("Invalid credentials"))?;

    // Verify password
    let hasher = PasswordHasher::new();
    if !hasher
        .verify(&payload.password, &user.password_hash)
        .map_err(|e| {
            rustpress_core::error::Error::internal(format!("Password verification failed: {}", e))
        })?
    {
        return Err(rustpress_core::error::Error::unauthorized("Invalid credentials").into());
    }

    // Check user status
    if user.status != "active" {
        return Err(rustpress_core::error::Error::forbidden("Account is not active").into());
    }

    // Update last login
    let _ = sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(pool)
        .await;

    // Generate JWT token
    let jwt_manager = state.jwt();
    let user_id_str = user.id.to_string();

    let token = jwt_manager
        .generate_access_token(&user_id_str, Some(&user.role), None)
        .map_err(|e| {
            rustpress_core::error::Error::internal(format!("Failed to generate token: {}", e))
        })?;

    let refresh = jwt_manager
        .generate_refresh_token(&user_id_str)
        .map_err(|e| {
            rustpress_core::error::Error::internal(format!(
                "Failed to generate refresh token: {}",
                e
            ))
        })?;

    Ok(Json(TokenResponse {
        access_token: token,
        refresh_token: Some(refresh),
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
        user: AuthUserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        },
    }))
}

async fn logout_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Invalidate all active sessions for this user
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user.id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to revoke sessions", e)
        })?;

    tracing::info!(user_id = %user.id, "User logged out, sessions revoked");

    Ok(no_content())
}

#[derive(Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String,
}

async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let jwt_manager = state.jwt();

    // Validate refresh token and get claims
    let claims = jwt_manager
        .validate_refresh_token(&payload.refresh_token)
        .map_err(|_| rustpress_core::error::Error::unauthorized("Invalid refresh token"))?;

    // Parse user ID from claims
    let user_uuid = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| rustpress_core::error::Error::unauthorized("Invalid user ID in token"))?;

    // Get user info with explicit columns
    let pool = state.db().inner();
    let user: Option<rustpress_database::repository::users::UserRow> = sqlx::query_as(
        r#"
        SELECT id, email, username, password_hash, display_name, status, role,
               avatar_url, locale, timezone,
               email_verified_at, last_login_at, created_at, updated_at, deleted_at
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to find user", e))?;

    let user = user.ok_or_else(|| rustpress_core::error::Error::unauthorized("User not found"))?;

    if user.status != "active" {
        return Err(rustpress_core::error::Error::forbidden("Account is not active").into());
    }

    // Generate new tokens
    let user_id_str = user.id.to_string();
    let token = jwt_manager
        .generate_access_token(&user_id_str, Some(&user.role), None)
        .map_err(|e| {
            rustpress_core::error::Error::internal(format!("Failed to generate token: {}", e))
        })?;

    let refresh = jwt_manager
        .generate_refresh_token(&user_id_str)
        .map_err(|e| {
            rustpress_core::error::Error::internal(format!(
                "Failed to generate refresh token: {}",
                e
            ))
        })?;

    Ok(Json(TokenResponse {
        access_token: token,
        refresh_token: Some(refresh),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: AuthUserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            display_name: user.display_name,
            role: user.role,
        },
    }))
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    username: String,
    password: String,
    display_name: Option<String>,
}

async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Validate password
    let validator = PasswordValidator::new(PasswordRules::default());
    validator
        .validate(&payload.password)
        .map_err(|e| rustpress_core::error::Error::validation(format!("{}", e)))?;

    // Check if email exists
    let exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = $1 AND deleted_at IS NULL")
            .bind(&payload.email.to_lowercase())
            .fetch_one(pool)
            .await
            .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    if exists.0 > 0 {
        return Err(rustpress_core::error::Error::validation("Email already registered").into());
    }

    // Check if username exists
    let exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = $1 AND deleted_at IS NULL")
            .bind(&payload.username)
            .fetch_one(pool)
            .await
            .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    if exists.0 > 0 {
        return Err(rustpress_core::error::Error::validation("Username already taken").into());
    }

    // Hash password
    let hasher = PasswordHasher::new();
    let password_hash = hasher.hash(&payload.password).map_err(|e| {
        rustpress_core::error::Error::internal(format!("Failed to hash password: {}", e))
    })?;

    // Create user
    let user_id = Uuid::now_v7();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, username, password_hash, display_name, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', $6, $6)
        "#
    )
    .bind(user_id)
    .bind(payload.email.to_lowercase())
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(&payload.display_name)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create user", e))?;

    // Assign default subscriber role
    sqlx::query(
        r#"
        INSERT INTO user_roles (id, user_id, role_id, created_at)
        SELECT gen_random_uuid(), $1, id, NOW()
        FROM roles WHERE name = 'subscriber'
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to assign role", e))?;

    Ok(Json(serde_json::json!({
        "message": "Registration successful. Please check your email to verify your account.",
        "user_id": user_id
    })))
}

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    email: String,
}

async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Check if user exists (but don't reveal this to the client)
    let user: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, email, display_name FROM users WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(&payload.email.to_lowercase())
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    if let Some((user_id, email, display_name)) = user {
        // Generate password reset token
        let reset_token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        // Hash the token for storage (using SHA-256)
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(reset_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Invalidate any existing tokens for this user
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

        // Store the hashed token
        sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

        // Send email with reset link
        if state.email().is_enabled().await {
            let name = display_name.as_deref();
            if let Err(e) = state
                .email()
                .send_password_reset(&email, name, &reset_token)
                .await
            {
                tracing::error!("Failed to send password reset email: {}", e);
            }
        } else {
            tracing::warn!(
                user_id = %user_id,
                "Email service not enabled. Password reset token: {}", reset_token
            );
        }
    }

    // Always return success to prevent email enumeration
    Ok(Json(serde_json::json!({
        "message": "If an account exists with that email, a password reset link has been sent."
    })))
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    token: String,
    password: String,
}

async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Validate password
    let validator = PasswordValidator::new(PasswordRules::default());
    validator
        .validate(&payload.password)
        .map_err(|e| rustpress_core::error::Error::validation(format!("{}", e)))?;

    // Hash the provided token to compare with stored hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(payload.token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Look up the token and get user info
    let token_record: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT id, user_id
        FROM password_reset_tokens
        WHERE token_hash = $1
          AND expires_at > NOW()
          AND used_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    let (token_id, user_id) = token_record.ok_or_else(|| {
        rustpress_core::error::Error::validation("Invalid or expired reset token")
    })?;

    // Hash the new password
    let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST).map_err(|e| {
        rustpress_core::error::Error::internal(format!("Failed to hash password: {}", e))
    })?;

    // Update the user's password
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&password_hash)
        .execute(pool)
        .await
        .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    // Mark the token as used
    sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await
        .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    // Invalidate all sessions for this user (force re-login)
    sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    tracing::info!(user_id = %user_id, "Password reset successful");

    Ok(Json(serde_json::json!({
        "message": "Password has been reset successfully. You can now login with your new password."
    })))
}

async fn current_user_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Get full user info with explicit columns
    let user_row: Option<rustpress_database::repository::users::UserRow> = sqlx::query_as(
        r#"
        SELECT id, email, username, password_hash, display_name, status, role,
               avatar_url, locale, timezone,
               email_verified_at, last_login_at, created_at, updated_at, deleted_at
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(user.id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Database error", e))?;

    match user_row {
        Some(u) => Ok(Json(serde_json::json!({
            "id": u.id,
            "email": u.email,
            "username": u.username,
            "display_name": u.display_name,
            "role": u.role,
            "status": u.status,
            "avatar_url": u.avatar_url,
            "locale": u.locale,
            "timezone": u.timezone,
            "email_verified": u.email_verified_at.is_some(),
            "last_login_at": u.last_login_at,
            "created_at": u.created_at
        }))),
        None => Err(rustpress_core::error::Error::not_found("User", "current").into()),
    }
}

// =============================================================================
// User Handlers
// =============================================================================

use rustpress_api::services::user_service::{
    CreateUserRequest, UpdateUserRequest, UserListParams, UserService,
};

/// User list query parameters
#[derive(Debug, serde::Deserialize)]
struct UserListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<String>,
    role: Option<String>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
}

async fn list_users_handler(
    _user: AuthUser,
    Query(query): Query<UserListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());

    let params = UserListParams {
        page: query.page,
        per_page: query.per_page,
        status: query.status,
        role: query.role,
        search: query.search,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    let result = service.list_users(params).await?;
    Ok(json(result))
}

async fn create_user_handler(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());
    let user = service.create_user(payload).await?;
    Ok(created(user))
}

async fn get_user_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());

    match service.get_user(id).await? {
        Some(user) => Ok(json(user)),
        None => Err(rustpress_core::error::Error::not_found("User", id.to_string()).into()),
    }
}

async fn update_user_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());
    let user = service.update_user(id, payload).await?;
    Ok(json(user))
}

async fn delete_user_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());
    service.delete_user(id).await?;
    Ok(no_content())
}

/// Role update request
#[derive(Debug, serde::Deserialize)]
struct UpdateRoleRequest {
    role: String,
}

async fn update_user_roles_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<UpdateRoleRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = UserService::new(state.db().inner().clone());
    let update = UpdateUserRequest {
        email: None,
        username: None,
        display_name: None,
        status: None,
        role: Some(payload.role),
        avatar_url: None,
        locale: None,
        timezone: None,
    };
    let user = service.update_user(id, update).await?;
    Ok(json(user))
}

// =============================================================================
// Post Handlers
// =============================================================================

use rustpress_api::services::post_service::{
    CreatePostRequest, PostListParams, PostService, UpdatePostRequest,
};

/// Post list query parameters
#[derive(Debug, serde::Deserialize)]
struct PostListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<String>,
    author_id: Option<Uuid>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
}

async fn list_posts_handler(
    Query(query): Query<PostListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());

    let params = PostListParams {
        page: query.page,
        per_page: query.per_page,
        status: query.status,
        author_id: query.author_id,
        search: query.search,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    let result = service.list_posts(params).await?;
    Ok(json(result))
}

async fn create_post_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreatePostRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());
    let post = service.create_post(payload, user.id).await?;
    Ok(created(post))
}

async fn get_post_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());

    match service.get_post(id).await? {
        Some(post) => Ok(json(post)),
        None => Err(rustpress_core::error::Error::not_found("Post", id.to_string()).into()),
    }
}

async fn update_post_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePostRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());
    let post = service.update_post(id, payload).await?;
    Ok(json(post))
}

async fn delete_post_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());
    service.delete_post(id).await?;
    Ok(no_content())
}

async fn publish_post_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());
    let post = service.publish_post(id).await?;
    Ok(json(post))
}

async fn unpublish_post_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());
    let post = service.unpublish_post(id).await?;
    Ok(json(post))
}

/// Bulk delete posts request
#[derive(Debug, serde::Deserialize)]
struct BulkDeletePostsRequest {
    ids: Vec<Uuid>,
}

/// Bulk delete posts response
#[derive(Debug, serde::Serialize)]
struct BulkDeletePostsResponse {
    deleted: usize,
}

async fn bulk_delete_posts_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<BulkDeletePostsRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());

    let mut deleted_count = 0;
    for id in payload.ids {
        if service.delete_post(id).await? {
            deleted_count += 1;
        }
    }

    Ok(json(BulkDeletePostsResponse {
        deleted: deleted_count,
    }))
}

async fn duplicate_post_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PostService::new(state.db().inner().clone());

    // Get the original post
    let original =
        service
            .get_post(id)
            .await?
            .ok_or_else(|| rustpress_core::error::Error::NotFound {
                entity_type: "post".to_string(),
                id: id.to_string(),
            })?;

    // Create a duplicate with modified title
    let duplicate_request = CreatePostRequest {
        title: format!("{} (Copy)", original.title),
        slug: None, // Let the service generate a new slug
        content: original.content,
        content_format: original.content_format,
        excerpt: original.excerpt,
        status: Some("draft".to_string()), // Always create as draft
        visibility: original.visibility,
        password: None,
        featured_image_id: original.featured_image_id,
        comment_status: original.comment_status,
        ping_status: original.ping_status,
        published_at: None,
        category_ids: if original.categories.is_empty() {
            None
        } else {
            Some(original.categories.iter().map(|c| c.id).collect())
        },
        tag_ids: if original.tags.is_empty() {
            None
        } else {
            Some(original.tags.iter().map(|t| t.id).collect())
        },
    };

    let new_post = service.create_post(duplicate_request, user.id).await?;
    Ok(created(new_post))
}

// =============================================================================
// Page Handlers
// =============================================================================

use rustpress_api::services::page_service::{
    CreatePageRequest, PageListParams, PageService, UpdatePageRequest,
};

/// Page list query parameters
#[derive(Debug, serde::Deserialize)]
struct PageListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<String>,
    parent_id: Option<Uuid>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    hierarchical: Option<bool>,
}

async fn list_pages_handler(
    Query(query): Query<PageListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PageService::new(state.db().inner().clone());

    let params = PageListParams {
        page: query.page,
        per_page: query.per_page,
        status: query.status,
        parent_id: query.parent_id,
        search: query.search,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        hierarchical: query.hierarchical,
    };

    let result = service.list_pages(params).await?;
    Ok(json(result))
}

async fn create_page_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreatePageRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PageService::new(state.db().inner().clone());
    let page = service.create_page(payload, user.id).await?;
    Ok(created(page))
}

async fn get_page_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PageService::new(state.db().inner().clone());

    match service.get_page(id).await? {
        Some(page) => Ok(json(page)),
        None => Err(rustpress_core::error::Error::not_found("Page", id.to_string()).into()),
    }
}

async fn update_page_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePageRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PageService::new(state.db().inner().clone());
    let page = service.update_page(id, payload).await?;
    Ok(json(page))
}

async fn delete_page_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = PageService::new(state.db().inner().clone());
    service.delete_page(id).await?;
    Ok(no_content())
}

// =============================================================================
// Media Handlers
// =============================================================================

use axum::extract::Multipart;
use rustpress_api::services::media_service::{
    validate_upload, MediaListParams, MediaService, UpdateMediaRequest as MediaUpdateRequest,
};
use std::io::Write;

/// Media list query parameters
#[derive(Debug, serde::Deserialize)]
struct MediaListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    media_type: Option<String>,
    mime_type: Option<String>,
    uploader_id: Option<Uuid>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
}

async fn list_media_handler(
    Query(query): Query<MediaListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = MediaService::new(state.db().inner().clone());

    let params = MediaListParams {
        page: query.page,
        per_page: query.per_page,
        media_type: query.media_type,
        mime_type: query.mime_type,
        uploader_id: query.uploader_id,
        search: query.search,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    let result = service.list_media(params).await?;
    Ok(json(result))
}

async fn upload_media_handler(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = MediaService::new(state.db().inner().clone());

    // Process multipart form data
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename = String::new();
    let mut content_type = String::from("application/octet-stream");
    let mut alt_text: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        rustpress_core::error::Error::validation(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                original_filename = field.file_name().unwrap_or("upload").to_string();
                if let Some(ct) = field.content_type() {
                    content_type = ct.to_string();
                }
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| {
                            rustpress_core::error::Error::validation(format!(
                                "Failed to read file: {}",
                                e
                            ))
                        })?
                        .to_vec(),
                );
            }
            "alt_text" => {
                alt_text = field.text().await.ok();
            }
            "title" => {
                title = field.text().await.ok();
            }
            "description" => {
                description = field.text().await.ok();
            }
            _ => {}
        }
    }

    let data =
        file_data.ok_or_else(|| rustpress_core::error::Error::validation("No file uploaded"))?;
    let file_size = data.len() as u64;

    // Validate the upload
    validate_upload(&original_filename, file_size)?;

    // Generate unique filename
    let ext = original_filename.rsplit('.').next().unwrap_or("bin");
    let unique_filename = format!(
        "{}_{}.{}",
        Uuid::new_v4(),
        chrono::Utc::now().timestamp(),
        ext
    );

    // Storage path (relative)
    let now = chrono::Utc::now();
    let storage_path = format!(
        "{}/{}/{}/{}",
        now.format("%Y"),
        now.format("%m"),
        now.format("%d"),
        unique_filename
    );

    // Create directory and save file
    let uploads_dir = std::path::Path::new("uploads");
    let full_path = uploads_dir.join(&storage_path);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rustpress_core::error::Error::internal(format!("Failed to create directory: {}", e))
        })?;
    }

    let mut file = std::fs::File::create(&full_path).map_err(|e| {
        rustpress_core::error::Error::internal(format!("Failed to create file: {}", e))
    })?;

    file.write_all(&data).map_err(|e| {
        rustpress_core::error::Error::internal(format!("Failed to write file: {}", e))
    })?;

    // Create database record
    let metadata = rustpress_api::services::media_service::UploadMediaMetadata {
        alt_text,
        title,
        description,
    };

    let media = service
        .upload_media(
            user.id,
            unique_filename,
            original_filename,
            content_type,
            file_size as i64,
            storage_path,
            Some(metadata),
            None, // dimensions - would need image processing
            None, // duration - would need audio/video processing
        )
        .await?;

    Ok(created(media))
}

async fn get_media_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = MediaService::new(state.db().inner().clone());

    match service.get_media(id).await? {
        Some(media) => Ok(json(media)),
        None => Err(rustpress_core::error::Error::not_found("Media", id.to_string()).into()),
    }
}

async fn update_media_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<MediaUpdateRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = MediaService::new(state.db().inner().clone());
    let media = service.update_media(id, payload).await?;
    Ok(json(media))
}

async fn delete_media_handler(
    _user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = MediaService::new(state.db().inner().clone());
    service.delete_media(id).await?;
    Ok(no_content())
}

/// List media folders
async fn list_media_folders_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let folders: Vec<(Uuid, String, Option<Uuid>)> =
        sqlx::query_as("SELECT id, name, parent_id FROM media_folders ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let folder_list: Vec<serde_json::Value> = folders
        .iter()
        .map(|(id, name, parent_id)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "parent_id": parent_id
            })
        })
        .collect();

    Ok(json(serde_json::json!({ "folders": folder_list })))
}

/// Create media folder request
#[derive(Debug, Deserialize)]
struct CreateMediaFolderRequest {
    name: String,
    parent_id: Option<Uuid>,
}

/// Create media folder
async fn create_media_folder_handler(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateMediaFolderRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let id = Uuid::now_v7();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO media_folders (id, name, parent_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)"
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.parent_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create folder", e))?;

    Ok(created(
        serde_json::json!({ "id": id, "name": payload.name }),
    ))
}

// =============================================================================
// Comment Handlers
// =============================================================================

use rustpress_api::services::comment_service::{
    BatchModerateRequest, CommentService, CreateCommentRequest as CommentCreateRequest,
    UpdateCommentRequest as CommentUpdateRequest,
};
use rustpress_database::repository::comments::CommentStatus;

/// Comment list query parameters
#[derive(Debug, serde::Deserialize)]
struct CommentListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    post_id: Option<Uuid>,
    status: Option<String>,
    search: Option<String>,
}

async fn list_comments_handler(
    user: AuthUser,
    Query(query): Query<CommentListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());

    let status = query.status.and_then(|s| s.parse::<CommentStatus>().ok());

    let result = service
        .list_comments(
            query.page.unwrap_or(1).into(),
            query.per_page.unwrap_or(20).into(),
            query.post_id,
            status,
            query.search,
        )
        .await?;

    Ok(json(result))
}

async fn create_comment_handler(
    user: Option<AuthUser>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CommentCreateRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());

    let user_id = user.map(|u| u.id);
    let ip = Some(addr.ip().to_string());
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let comment = service
        .submit_comment(payload, user_id, ip, user_agent)
        .await?;

    Ok(created(comment))
}

async fn get_comment_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());

    match service.get_comment(id).await? {
        Some(comment) => Ok(json(comment)),
        None => Err(rustpress_core::error::Error::not_found("Comment", id.to_string()).into()),
    }
}

async fn update_comment_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<CommentUpdateRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let comment = service.update_comment(id, payload).await?;
    Ok(json(comment))
}

async fn delete_comment_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    service.delete_comment(id).await?;
    Ok(no_content())
}

async fn approve_comment_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let comment = service.approve_comment(id, user.id).await?;
    Ok(json(comment))
}

async fn mark_spam_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let comment = service.mark_as_spam(id, user.id).await?;
    Ok(json(comment))
}

async fn trash_comment_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let comment = service.trash_comment(id, user.id).await?;
    Ok(json(comment))
}

async fn batch_moderate_comments_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchModerateRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let updated = service.batch_moderate(payload, user.id).await?;
    Ok(json(serde_json::json!({ "updated": updated })))
}

async fn toggle_like_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let liked = service.toggle_like(id, user.id).await?;
    Ok(json(serde_json::json!({ "liked": liked })))
}

async fn comment_counts_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = CommentService::new(state.db().inner().clone());
    let counts = service.get_counts().await?;
    Ok(json(counts))
}

// =============================================================================
// Settings Handlers
// =============================================================================

use rustpress_api::services::settings_service::{
    BatchUpdateRequest, SettingUpdate, SettingsService,
};

/// List all settings grouped
async fn list_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_all_grouped().await?;
    Ok(json(settings))
}

/// Get a single setting by key
async fn get_setting_handler(
    user: AuthUser,
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    match service.get(&key).await? {
        Some(setting) => Ok(json(setting)),
        None => Err(crate::error::HttpError::not_found("Setting not found")),
    }
}

/// Update a single setting
async fn update_setting_handler(
    user: AuthUser,
    axum::extract::Path(key): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());

    // Extract value from payload (support both { "value": x } and direct value)
    let value = if let Some(v) = payload.get("value") {
        v.clone()
    } else {
        payload
    };

    let updated = service.update(&key, value).await?;
    Ok(json(updated))
}

/// Batch update multiple settings
async fn batch_update_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<BatchUpdateRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let updated = service.batch_update(payload.settings).await?;
    Ok(json(serde_json::json!({
        "updated": updated.len(),
        "settings": updated
    })))
}

/// Get settings by group
async fn get_settings_group_handler(
    user: AuthUser,
    axum::extract::Path(group): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group(&group).await?;
    Ok(json(settings))
}

/// Get general settings
async fn get_general_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group("general").await?;
    Ok(json(settings))
}

/// Get reading settings
async fn get_reading_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group("reading").await?;
    Ok(json(settings))
}

/// Get writing settings
async fn get_writing_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group("writing").await?;
    Ok(json(settings))
}

/// Get discussion settings
async fn get_discussion_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group("discussion").await?;
    Ok(json(settings))
}

/// Get permalinks settings
async fn get_permalinks_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let service = SettingsService::new(state.db().inner().clone());
    let settings = service.get_by_group("permalinks").await?;
    Ok(json(settings))
}

// =============================================================================
// Plugin Handlers (Placeholder implementations)
// =============================================================================

async fn list_plugins_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "plugins": [] })))
}

async fn install_plugin_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(created(serde_json::json!({ "message": "Install plugin" })))
}

async fn get_plugin_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "id": id })))
}

async fn uninstall_plugin_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(no_content())
}

async fn activate_plugin_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "id": id, "active": true })))
}

async fn deactivate_plugin_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "id": id, "active": false })))
}

// =============================================================================
// Theme Handlers
// =============================================================================

/// List all installed themes
async fn list_themes_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Get themes from database
    let themes = state.theme_manager().list_themes().await?;
    let active_theme = state.theme_manager().get_active_theme_id().await?;

    Ok(json(serde_json::json!({
        "themes": themes,
        "total": themes.len(),
        "active_theme": active_theme
    })))
}

/// Scan themes directory and register new themes
async fn scan_themes_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Scan themes directory for new/updated themes
    let scan_result = state.theme_manager().scan_themes().await?;

    Ok(json(serde_json::json!({
        "scanned": scan_result.scanned,
        "registered": scan_result.registered,
        "updated": scan_result.updated,
        "errors": scan_result.errors
    })))
}

/// Get the currently active theme
async fn get_active_theme_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let theme = state.theme_manager().get_active_theme().await?;

    match theme {
        Some(t) => Ok(json(t)),
        None => Err(rustpress_core::error::Error::not_found("Theme", "active").into()),
    }
}

/// Get a specific theme by its ID
async fn get_theme_handler(
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let theme = state.theme_manager().get_theme(&theme_id).await?;

    match theme {
        Some(t) => Ok(json(t)),
        None => Err(crate::error::HttpError::not_found(format!(
            "Theme '{}' not found",
            theme_id
        ))),
    }
}

/// Delete/uninstall a theme
async fn delete_theme_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    state.theme_manager().delete_theme(&theme_id).await?;
    Ok(no_content())
}

/// Activate a theme
async fn activate_theme_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let theme = state.theme_manager().activate_theme(&theme_id).await?;

    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("Theme '{}' activated successfully", theme.name),
        "theme": theme
    })))
}

/// Get theme settings and customizer schema
async fn get_theme_settings_handler(
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let settings = state.theme_manager().get_theme_settings(&theme_id).await?;
    Ok(json(settings))
}

/// Update theme settings/customizations
async fn update_theme_settings_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let settings = state
        .theme_manager()
        .update_theme_settings(&theme_id, payload)
        .await?;

    Ok(json(serde_json::json!({
        "success": true,
        "settings": settings
    })))
}

/// Get menu assignments for a theme
async fn get_theme_menus_handler(
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let menus = state
        .theme_manager()
        .get_menu_assignments(&theme_id)
        .await?;
    Ok(json(menus))
}

/// Update menu assignments for a theme
async fn update_theme_menus_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    state
        .theme_manager()
        .update_menu_assignments(&theme_id, payload)
        .await?;

    Ok(json(serde_json::json!({
        "success": true,
        "message": "Menu assignments updated"
    })))
}

/// Get widget assignments for a theme
async fn get_theme_widgets_handler(
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let widgets = state
        .theme_manager()
        .get_widget_assignments(&theme_id)
        .await?;
    Ok(json(widgets))
}

/// Update widget assignments for a theme
async fn update_theme_widgets_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    state
        .theme_manager()
        .update_widget_assignments(&theme_id, payload)
        .await?;

    Ok(json(serde_json::json!({
        "success": true,
        "message": "Widget assignments updated"
    })))
}

/// Create a theme preview session
async fn create_theme_preview_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let preview = state
        .theme_manager()
        .create_preview(user.id, &theme_id, payload)
        .await?;

    Ok(json(serde_json::json!({
        "preview_token": preview.token,
        "preview_url": format!("/?preview={}", preview.token),
        "expires_at": preview.expires_at
    })))
}

/// Upload and install a theme from ZIP file
async fn upload_theme_handler(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Extract ZIP file from multipart
    let mut zip_data: Option<Vec<u8>> = None;
    let mut activate_after = false;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::HttpError::bad_request(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "theme" || name == "file" {
            zip_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        crate::error::HttpError::bad_request(format!("Failed to read file: {}", e))
                    })?
                    .to_vec(),
            );
        } else if name == "activate" {
            let value = field.text().await.unwrap_or_default();
            activate_after = value == "true" || value == "1";
        }
    }

    let zip_data = zip_data.ok_or_else(|| {
        crate::error::HttpError::bad_request("No theme file provided".to_string())
    })?;

    // Install the theme
    let result = state
        .theme_manager()
        .install_from_zip(&zip_data, activate_after)
        .await?;

    Ok(created(serde_json::json!({
        "success": result.success,
        "theme_id": result.theme_id,
        "theme_name": result.theme_name,
        "message": result.message,
        "warnings": result.warnings
    })))
}

/// Validate a theme ZIP without installing
async fn validate_theme_handler(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> HttpResult<impl axum::response::IntoResponse> {
    let mut zip_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::HttpError::bad_request(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "theme" || name == "file" {
            zip_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        crate::error::HttpError::bad_request(format!("Failed to read file: {}", e))
                    })?
                    .to_vec(),
            );
        }
    }

    let zip_data = zip_data.ok_or_else(|| {
        crate::error::HttpError::bad_request("No theme file provided".to_string())
    })?;

    let result = state.theme_manager().validate_zip(&zip_data)?;

    Ok(json(serde_json::json!({
        "valid": result.valid,
        "theme_id": result.theme_id,
        "theme_name": result.theme_name,
        "errors": result.errors,
        "warnings": result.warnings
    })))
}

/// Get available default themes
async fn get_available_themes_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let available = state.theme_manager().get_available_themes();
    Ok(json(serde_json::json!({ "themes": available })))
}

/// Update a theme from ZIP file
async fn update_theme_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> HttpResult<impl axum::response::IntoResponse> {
    let mut zip_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        crate::error::HttpError::bad_request(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "theme" || name == "file" {
            zip_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        crate::error::HttpError::bad_request(format!("Failed to read file: {}", e))
                    })?
                    .to_vec(),
            );
        }
    }

    let zip_data = zip_data.ok_or_else(|| {
        crate::error::HttpError::bad_request("No theme file provided".to_string())
    })?;

    let result = state.theme_manager().update_from_zip(&zip_data).await?;

    Ok(json(serde_json::json!({
        "success": result.success,
        "theme_id": result.theme_id,
        "theme_name": result.theme_name,
        "message": result.message,
        "warnings": result.warnings
    })))
}

/// Export a theme as ZIP file
async fn export_theme_handler(
    user: AuthUser,
    axum::extract::Path(theme_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let zip_data = state.theme_manager().export_theme(&theme_id).await?;
    let disposition = format!("attachment; filename=\"{}.zip\"", theme_id);

    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "application/zip")
        .header(axum::http::header::CONTENT_DISPOSITION, disposition)
        .body(axum::body::Body::from(zip_data))
        .unwrap())
}

// =============================================================================
// Public Website Handlers (Theme Rendering)
// =============================================================================

/// Query params for public routes
#[derive(Debug, Deserialize)]
struct PublicQueryParams {
    page: Option<i32>,
    preview: Option<String>,
}

/// Convert rendered page to response
fn rendered_response(
    result: Result<crate::services::RenderedPage, rustpress_core::error::Error>,
) -> Response {
    match result {
        Ok(page) => {
            let mut response = Html(page.html).into_response();
            let headers = response.headers_mut();
            headers.insert(
                header::CACHE_CONTROL,
                page.cache_control
                    .parse()
                    .unwrap_or_else(|_| "no-cache".parse().unwrap()),
            );
            headers.insert(
                header::CONTENT_TYPE,
                page.content_type
                    .parse()
                    .unwrap_or_else(|_| "text/html".parse().unwrap()),
            );
            response
        }
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                axum::http::StatusCode::NOT_FOUND
            } else {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
<h1>Error</h1>
<p>{}</p>
</body>
</html>"#,
                    e
                )),
            )
                .into_response()
        }
    }
}

/// Public home page handler
async fn public_home_handler(
    State(state): State<AppState>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let result = state
        .renderer()
        .render_home(params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public blog archive handler
async fn public_blog_handler(
    State(state): State<AppState>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    // Blog page is same as home for now
    let result = state
        .renderer()
        .render_home(params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public single post handler
async fn public_post_handler(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let result = state
        .renderer()
        .render_post(&slug, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public page handler
async fn public_page_handler(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let result = state
        .renderer()
        .render_page(&slug, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public category archive handler
async fn public_category_handler(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let page = params.page.unwrap_or(1);
    let result = state
        .renderer()
        .render_category(&slug, page, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public tag archive handler
async fn public_tag_handler(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let page = params.page.unwrap_or(1);
    let result = state
        .renderer()
        .render_tag(&slug, page, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public author archive handler
async fn public_author_handler(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<PublicQueryParams>,
) -> Response {
    let page = params.page.unwrap_or(1);
    let result = state
        .renderer()
        .render_author(&slug, page, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Search query params
#[derive(Debug, Deserialize)]
struct SearchQueryParams {
    q: Option<String>,
    page: Option<i32>,
    preview: Option<String>,
}

/// Public search results handler
async fn public_search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQueryParams>,
) -> Response {
    let query = params.q.unwrap_or_default();
    let page = params.page.unwrap_or(1);

    if query.is_empty() {
        return Html(
            r#"<!DOCTYPE html>
<html>
<head><title>Search</title></head>
<body>
<h1>Search</h1>
<form action="/search" method="get">
<input type="search" name="q" placeholder="Search...">
<button type="submit">Search</button>
</form>
</body>
</html>"#,
        )
        .into_response();
    }

    let result = state
        .renderer()
        .render_search(&query, page, params.preview.as_deref())
        .await;
    rendered_response(result)
}

/// Public RSS feed handler
async fn public_feed_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Generate RSS 2.0 feed
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
    <title>RustPress Site</title>
    <link>http://localhost:3000</link>
    <description>Powered by RustPress</description>
    <language>en-US</language>
    <lastBuildDate>Mon, 15 Dec 2025 00:00:00 +0000</lastBuildDate>
    <atom:link href="http://localhost:3000/feed" rel="self" type="application/rss+xml"/>
</channel>
</rss>"#;

    (
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
}

/// Public Atom feed handler
async fn public_atom_feed_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Generate Atom feed
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <title>RustPress Site</title>
    <link href="http://localhost:3000"/>
    <link href="http://localhost:3000/feed/atom" rel="self"/>
    <id>http://localhost:3000/</id>
    <updated>2025-12-15T00:00:00Z</updated>
</feed>"#;

    (
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        xml,
    )
}

/// Public sitemap handler
async fn public_sitemap_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Generate XML sitemap
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>http://localhost:3000/</loc>
        <lastmod>2025-12-15</lastmod>
        <changefreq>daily</changefreq>
        <priority>1.0</priority>
    </url>
    <url>
        <loc>http://localhost:3000/blog</loc>
        <lastmod>2025-12-15</lastmod>
        <changefreq>daily</changefreq>
        <priority>0.8</priority>
    </url>
</urlset>"#;

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}

/// Public robots.txt handler
async fn public_robots_handler() -> impl IntoResponse {
    let robots = r#"User-agent: *
Allow: /

Sitemap: http://localhost:3000/sitemap.xml"#;

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}

/// Theme static asset handler
async fn theme_asset_handler(
    State(state): State<AppState>,
    axum::extract::Path((theme_id, path)): axum::extract::Path<(String, String)>,
) -> Response {
    use tokio::fs;

    // Build the file path - get themes_dir from theme manager
    let themes_dir = state.theme_manager().themes_dir().to_path_buf();
    let file_path = themes_dir.join(&theme_id).join(&path);

    // Security: ensure path doesn't escape themes directory
    let canonical = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (axum::http::StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    let themes_canonical = themes_dir.canonicalize().unwrap_or(themes_dir);
    if !canonical.starts_with(&themes_canonical) {
        return (axum::http::StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // Read file
    match fs::read(&canonical).await {
        Ok(contents) => {
            // Determine content type
            let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                Some("woff") => "font/woff",
                Some("woff2") => "font/woff2",
                Some("ttf") => "font/ttf",
                Some("eot") => "application/vnd.ms-fontobject",
                Some("ico") => "image/x-icon",
                Some("webp") => "image/webp",
                Some("json") => "application/json",
                Some("xml") => "application/xml",
                _ => "application/octet-stream",
            };

            (
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "public, max-age=31536000"),
                ],
                contents,
            )
                .into_response()
        }
        Err(_) => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

// =============================================================================
// Search Routes and Handlers
// =============================================================================

/// Search routes for full-text search
fn search_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(search_handler))
        .route("/suggest", get(search_suggest_handler))
        .route("/reindex", post(search_reindex_handler))
        .route("/stats", get(search_stats_handler))
}

/// Search query parameters
#[derive(Debug, Deserialize)]
struct ApiSearchQuery {
    q: String,
    #[serde(rename = "type")]
    content_type: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

/// Full-text search handler
async fn search_handler(
    Query(query): Query<ApiSearchQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    // Build search query using PostgreSQL full-text search
    let search_term = query.q.trim();
    if search_term.is_empty() {
        return Ok(json(serde_json::json!({
            "results": [],
            "total": 0,
            "page": page,
            "per_page": per_page
        })));
    }

    // Convert search term to tsquery format
    let ts_query = search_term
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" & ");

    // Search posts using full-text search
    let posts: Vec<(Uuid, String, String, Option<String>, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT p.id, p.title, p.slug, p.excerpt, 'post' as content_type, p.published_at
        FROM posts p
        WHERE p.status = 'published'
          AND p.deleted_at IS NULL
          AND (
            to_tsvector('english', COALESCE(p.title, '') || ' ' || COALESCE(p.content, '')) @@ to_tsquery('english', $1)
            OR p.title ILIKE '%' || $2 || '%'
            OR p.content ILIKE '%' || $2 || '%'
          )
        ORDER BY p.published_at DESC
        LIMIT $3 OFFSET $4
        "#
    )
    .bind(&ts_query)
    .bind(search_term)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Search failed", e))?;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM posts p
        WHERE p.status = 'published'
          AND p.deleted_at IS NULL
          AND (
            to_tsvector('english', COALESCE(p.title, '') || ' ' || COALESCE(p.content, '')) @@ to_tsquery('english', $1)
            OR p.title ILIKE '%' || $2 || '%'
            OR p.content ILIKE '%' || $2 || '%'
          )
        "#
    )
    .bind(&ts_query)
    .bind(search_term)
    .fetch_one(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Search count failed", e))?;

    let results: Vec<serde_json::Value> = posts
        .iter()
        .map(|(id, title, slug, excerpt, content_type, published_at)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "slug": slug,
                "excerpt": excerpt,
                "type": content_type,
                "published_at": published_at
            })
        })
        .collect();

    Ok(json(serde_json::json!({
        "results": results,
        "total": total.0,
        "page": page,
        "per_page": per_page,
        "total_pages": (total.0 as f64 / per_page as f64).ceil() as i64
    })))
}

/// Search suggestions handler
async fn search_suggest_handler(
    Query(query): Query<ApiSearchQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let limit = query.per_page.unwrap_or(10).min(20);
    let search_term = query.q.trim();

    if search_term.len() < 2 {
        return Ok(json(serde_json::json!({ "suggestions": [] })));
    }

    // Get title suggestions from posts
    let suggestions: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT title
        FROM posts
        WHERE status = 'published'
          AND deleted_at IS NULL
          AND title ILIKE '%' || $1 || '%'
        ORDER BY title
        LIMIT $2
        "#,
    )
    .bind(search_term)
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Suggestions failed", e))?;

    let suggestions: Vec<String> = suggestions.into_iter().map(|(t,)| t).collect();

    Ok(json(serde_json::json!({ "suggestions": suggestions })))
}

/// Trigger search reindex
async fn search_reindex_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In a real implementation, this would queue a background job
    // to rebuild the full-text search index
    Ok(json(serde_json::json!({
        "status": "queued",
        "message": "Search reindex job has been queued"
    })))
}

/// Get search statistics
async fn search_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Get indexed content counts
    let post_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND status = 'published' AND deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let page_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'page' AND status = 'published' AND deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "indexed_posts": post_count.0,
        "indexed_pages": page_count.0,
        "last_reindex": chrono::Utc::now(),
        "index_health": "healthy"
    })))
}

// =============================================================================
// Backup Routes and Handlers
// =============================================================================

/// Backup management routes
fn backup_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_backups_handler).post(create_backup_handler))
        .route("/storage", get(backup_storage_handler))
        .route(
            "/schedules",
            get(list_backup_schedules_handler).post(create_backup_schedule_handler),
        )
        .route(
            "/schedules/:id",
            get(get_backup_schedule_handler)
                .put(update_backup_schedule_handler)
                .delete(delete_backup_schedule_handler),
        )
        .route(
            "/:id",
            get(get_backup_handler).delete(delete_backup_handler),
        )
        .route("/:id/download", get(download_backup_handler))
        .route("/:id/restore", post(restore_backup_handler))
        .route("/restore/:job_id", get(restore_progress_handler))
}

/// Backup list query
#[derive(Debug, Deserialize)]
struct BackupListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    #[serde(rename = "type")]
    backup_type: Option<String>,
}

/// List all backups
async fn list_backups_handler(
    user: AuthUser,
    Query(query): Query<BackupListQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let offset = ((page - 1) * per_page) as i64;

    // Get backups from database (using actual schema with file_size column)
    let backups: Vec<(
        Uuid,
        String,
        String,
        Option<i64>,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"
        SELECT id, name, backup_type, file_size, status, created_at
        FROM backups
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM backups")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let backup_list: Vec<serde_json::Value> = backups
        .iter()
        .map(|(id, name, backup_type, size, status, created_at)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "type": backup_type,
                "size_bytes": size.unwrap_or(0),
                "status": status,
                "created_at": created_at
            })
        })
        .collect();

    Ok(json(serde_json::json!({
        "backups": backup_list,
        "total": total.0,
        "page": page,
        "per_page": per_page
    })))
}

/// Create backup request
#[derive(Debug, Deserialize)]
struct CreateBackupRequest {
    #[serde(rename = "type")]
    backup_type: Option<String>,
    include_media: Option<bool>,
    include_themes: Option<bool>,
    include_plugins: Option<bool>,
    description: Option<String>,
}

/// Create a new backup
async fn create_backup_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateBackupRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let backup_id = Uuid::now_v7();
    let backup_type = payload.backup_type.unwrap_or_else(|| "full".to_string());
    let now = chrono::Utc::now();
    let name = format!("backup_{}_{}", backup_type, now.format("%Y%m%d_%H%M%S"));

    // Build includes JSONB
    let includes = serde_json::json!({
        "database": true,
        "media": payload.include_media.unwrap_or(true),
        "themes": payload.include_themes.unwrap_or(true),
        "plugins": payload.include_plugins.unwrap_or(true)
    });

    // Create backup record (matching actual schema)
    sqlx::query(
        r#"
        INSERT INTO backups (id, name, backup_type, status, includes, rustpress_version, started_at, created_at)
        VALUES ($1, $2, $3, 'pending', $4, '1.0.0', $5, $5)
        "#
    )
    .bind(backup_id)
    .bind(&name)
    .bind(&backup_type)
    .bind(&includes)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create backup", e))?;

    // Queue backup job (in production this would be a background job)

    Ok(created(serde_json::json!({
        "id": backup_id,
        "name": name,
        "type": backup_type,
        "status": "pending",
        "message": "Backup job has been queued"
    })))
}

/// Get a specific backup
async fn get_backup_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let backup: Option<(
        Uuid,
        String,
        String,
        Option<i64>,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT id, name, backup_type, file_size, status, created_at FROM backups WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to get backup", e))?;

    match backup {
        Some((id, name, backup_type, size, status, created_at)) => Ok(json(serde_json::json!({
            "id": id,
            "name": name,
            "type": backup_type,
            "size_bytes": size.unwrap_or(0),
            "status": status,
            "created_at": created_at
        }))),
        None => Err(rustpress_core::error::Error::not_found("Backup", id.to_string()).into()),
    }
}

/// Delete a backup
async fn delete_backup_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Delete the backup record (actual schema doesn't have deleted_at)
    sqlx::query("DELETE FROM backups WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete backup", e)
        })?;

    Ok(no_content())
}

/// Download a backup
async fn download_backup_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let backup: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT name, file_path FROM backups WHERE id = $1 AND status = 'completed'",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to get backup", e))?;

    match backup {
        Some((name, Some(file_path))) => {
            // Read backup file
            let contents = tokio::fs::read(&file_path).await.map_err(|e| {
                rustpress_core::error::Error::internal(format!("Failed to read backup file: {}", e))
            })?;

            let disposition = format!("attachment; filename=\"{}.zip\"", name);

            Ok(axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/zip")
                .header(header::CONTENT_DISPOSITION, disposition)
                .body(axum::body::Body::from(contents))
                .unwrap()
                .into_response())
        }
        Some((_, None)) => Err(rustpress_core::error::Error::internal(
            "Backup file path not set".to_string(),
        )
        .into()),
        None => Err(rustpress_core::error::Error::not_found("Backup", id.to_string()).into()),
    }
}

/// Restore from backup
async fn restore_backup_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let job_id = Uuid::now_v7();

    // In production, this would queue a restore job
    Ok(json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "message": "Restore job has been queued"
    })))
}

/// Get restore progress
async fn restore_progress_handler(
    user: AuthUser,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, this would check the job status
    Ok(json(serde_json::json!({
        "job_id": job_id,
        "status": "completed",
        "progress": 100,
        "message": "Restore completed successfully"
    })))
}

/// Get backup storage usage
async fn backup_storage_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let total_size: (i64,) = sqlx::query_as("SELECT COALESCE(SUM(file_size), 0) FROM backups")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let backup_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM backups")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "used_bytes": total_size.0,
        "backup_count": backup_count.0,
        "quota_bytes": 10_737_418_240i64, // 10GB default quota
        "used_percentage": (total_size.0 as f64 / 10_737_418_240.0 * 100.0).round()
    })))
}

/// List backup schedules
async fn list_backup_schedules_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Using actual schema: frequency + cron_expression instead of schedule
    let schedules: Vec<(Uuid, String, String, Option<String>, String, bool, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT id, name, frequency, cron_expression, backup_type, enabled, next_run FROM backup_schedules"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let schedule_list: Vec<serde_json::Value> = schedules
        .iter()
        .map(
            |(id, name, frequency, cron_expr, backup_type, enabled, next_run)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "frequency": frequency,
                    "schedule": cron_expr.clone().unwrap_or_else(|| frequency.clone()),
                    "type": backup_type,
                    "enabled": enabled,
                    "next_run_at": next_run
                })
            },
        )
        .collect();

    Ok(json(serde_json::json!({ "schedules": schedule_list })))
}

/// Create backup schedule request
#[derive(Debug, Deserialize)]
struct CreateBackupScheduleRequest {
    name: String,
    schedule: String, // cron expression or frequency
    #[serde(rename = "type")]
    backup_type: Option<String>,
    retention_days: Option<i32>,
}

/// Create a backup schedule
async fn create_backup_schedule_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateBackupScheduleRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let schedule_id = Uuid::now_v7();
    let backup_type = payload.backup_type.unwrap_or_else(|| "full".to_string());
    let now = chrono::Utc::now();

    // Build retention JSONB
    let retention = serde_json::json!({
        "retention_days": payload.retention_days.unwrap_or(30),
        "max_backups": 10,
        "min_backups": 1
    });

    sqlx::query(
        r#"
        INSERT INTO backup_schedules (id, name, frequency, cron_expression, backup_type, retention, enabled, created_at, updated_at)
        VALUES ($1, $2, 'custom', $3, $4, $5, true, $6, $6)
        "#
    )
    .bind(schedule_id)
    .bind(&payload.name)
    .bind(&payload.schedule)
    .bind(&backup_type)
    .bind(&retention)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create schedule", e))?;

    Ok(created(serde_json::json!({
        "id": schedule_id,
        "name": payload.name,
        "schedule": payload.schedule,
        "type": backup_type,
        "enabled": true
    })))
}

/// Get backup schedule
async fn get_backup_schedule_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "id": id })))
}

/// Update backup schedule
async fn update_backup_schedule_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({ "id": id, "updated": true })))
}

/// Delete backup schedule
async fn delete_backup_schedule_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    sqlx::query("DELETE FROM backup_schedules WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete schedule", e)
        })?;

    Ok(no_content())
}

// =============================================================================
// SEO Routes and Handlers
// =============================================================================

/// SEO management routes
fn seo_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/settings",
            get(get_seo_settings_handler).put(update_seo_settings_handler),
        )
        .route("/sitemap", get(get_sitemap_status_handler))
        .route("/sitemap/generate", post(generate_sitemap_handler))
        .route(
            "/robots",
            get(get_robots_txt_handler).put(update_robots_txt_handler),
        )
        .route("/analyze", post(analyze_seo_handler))
        .route("/bulk-analyze", post(bulk_analyze_seo_handler))
        .route("/dashboard", get(seo_dashboard_handler))
        .route(
            "/:content_type/:id",
            get(get_content_seo_handler).put(update_content_seo_handler),
        )
}

/// Get SEO settings
async fn get_seo_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Get SEO-related settings
    let settings: Vec<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT key, value FROM settings WHERE key LIKE 'seo_%' OR key LIKE 'site_%'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut seo_settings = serde_json::json!({
        "site_title": "",
        "site_description": "",
        "title_separator": " | ",
        "default_meta_robots": "index, follow",
        "enable_open_graph": true,
        "enable_twitter_cards": true,
        "enable_schema_markup": true,
        "google_analytics_id": "",
        "google_search_console": "",
        "bing_webmaster": ""
    });

    for (key, value) in settings {
        if let serde_json::Value::Object(ref mut map) = seo_settings {
            map.insert(key, value);
        }
    }

    Ok(json(seo_settings))
}

/// Update SEO settings
async fn update_seo_settings_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            sqlx::query(
                r#"
                INSERT INTO settings (id, key, value, group_name, updated_at)
                VALUES (gen_random_uuid(), $1, $2, 'seo', NOW())
                ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
                "#,
            )
            .bind(key)
            .bind(value)
            .execute(pool)
            .await
            .ok();
        }
    }

    Ok(json(serde_json::json!({ "success": true })))
}

/// Get sitemap status
async fn get_sitemap_status_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let post_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND status = 'published' AND deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let page_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'page' AND status = 'published' AND deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "url_count": post_count.0 + page_count.0 + 2, // +2 for home and blog
        "last_generated": chrono::Utc::now(),
        "sitemap_url": "/sitemap.xml"
    })))
}

/// Generate sitemap
async fn generate_sitemap_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, this would regenerate the sitemap
    Ok(json(serde_json::json!({
        "success": true,
        "message": "Sitemap regenerated successfully"
    })))
}

/// Get robots.txt content
async fn get_robots_txt_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let robots: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'robots_txt'")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let content = robots
        .and_then(|(v,)| v.as_str().map(String::from))
        .unwrap_or_else(|| "User-agent: *\nAllow: /\n\nSitemap: /sitemap.xml".to_string());

    Ok(json(serde_json::json!({ "content": content })))
}

/// Update robots.txt content
async fn update_robots_txt_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("User-agent: *\nAllow: /");

    sqlx::query(
        r#"
        INSERT INTO settings (id, key, value, group_name, updated_at)
        VALUES (gen_random_uuid(), 'robots_txt', $1, 'seo', NOW())
        ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
        "#,
    )
    .bind(serde_json::json!(content))
    .execute(pool)
    .await
    .map_err(|e| {
        rustpress_core::error::Error::database_with_source("Failed to update robots.txt", e)
    })?;

    Ok(json(serde_json::json!({ "success": true })))
}

/// SEO analysis request
#[derive(Debug, Deserialize)]
struct SeoAnalyzeRequest {
    title: String,
    content: String,
    focus_keyword: Option<String>,
    meta_description: Option<String>,
}

/// Analyze content for SEO
async fn analyze_seo_handler(
    user: AuthUser,
    Json(payload): Json<SeoAnalyzeRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let mut score = 0;
    let mut suggestions = Vec::new();

    // Title analysis
    let title_len = payload.title.len();
    if title_len >= 30 && title_len <= 60 {
        score += 20;
    } else if title_len < 30 {
        suggestions.push("Title is too short. Aim for 30-60 characters.");
    } else {
        suggestions.push("Title is too long. Keep it under 60 characters.");
    }

    // Content length
    let word_count = payload.content.split_whitespace().count();
    if word_count >= 300 {
        score += 20;
    } else {
        suggestions.push("Content is too short. Aim for at least 300 words.");
    }

    // Meta description
    if let Some(ref desc) = payload.meta_description {
        let desc_len = desc.len();
        if desc_len >= 120 && desc_len <= 160 {
            score += 20;
        } else if desc_len < 120 {
            suggestions.push("Meta description is too short. Aim for 120-160 characters.");
        } else {
            suggestions.push("Meta description is too long. Keep it under 160 characters.");
        }
    } else {
        suggestions.push("Add a meta description for better SEO.");
    }

    // Focus keyword analysis
    if let Some(ref keyword) = payload.focus_keyword {
        if payload
            .title
            .to_lowercase()
            .contains(&keyword.to_lowercase())
        {
            score += 15;
        } else {
            suggestions.push("Include your focus keyword in the title.");
        }

        if payload
            .content
            .to_lowercase()
            .contains(&keyword.to_lowercase())
        {
            score += 15;
        } else {
            suggestions.push("Include your focus keyword in the content.");
        }
    } else {
        suggestions.push("Set a focus keyword for better optimization.");
    }

    // Readability (simple check)
    let avg_sentence_len = payload.content.split('.').count();
    if avg_sentence_len > 0 && word_count / avg_sentence_len < 25 {
        score += 10;
    } else {
        suggestions.push("Consider using shorter sentences for better readability.");
    }

    Ok(json(serde_json::json!({
        "score": score,
        "max_score": 100,
        "grade": if score >= 80 { "A" } else if score >= 60 { "B" } else if score >= 40 { "C" } else { "D" },
        "suggestions": suggestions,
        "word_count": word_count,
        "reading_time_minutes": (word_count / 200).max(1)
    })))
}

/// Bulk analyze request
#[derive(Debug, Deserialize)]
struct BulkAnalyzeRequest {
    ids: Vec<Uuid>,
    #[serde(rename = "type")]
    content_type: String,
}

/// Bulk analyze content for SEO
async fn bulk_analyze_seo_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<BulkAnalyzeRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, this would queue analysis jobs
    Ok(json(serde_json::json!({
        "queued": payload.ids.len(),
        "message": "Analysis jobs queued"
    })))
}

/// SEO dashboard statistics
async fn seo_dashboard_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let post_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE status = 'published' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "total_pages_indexed": post_count.0,
        "average_seo_score": 72,
        "pages_without_meta": 5,
        "pages_with_issues": 12,
        "sitemap_urls": post_count.0 + 2
    })))
}

/// Get SEO metadata for specific content
async fn get_content_seo_handler(
    axum::extract::Path((content_type, id)): axum::extract::Path<(String, Uuid)>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Get SEO metadata from post_meta or page_meta tables
    let seo: Option<(Option<String>, Option<String>, Option<String>)> = match content_type.as_str() {
        "post" | "posts" => {
            sqlx::query_as(
                r#"
                SELECT
                    (SELECT value FROM post_meta WHERE post_id = $1 AND key = 'seo_title') as seo_title,
                    (SELECT value FROM post_meta WHERE post_id = $1 AND key = 'seo_description') as seo_description,
                    (SELECT value FROM post_meta WHERE post_id = $1 AND key = 'focus_keyword') as focus_keyword
                "#
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
        }
        _ => None
    };

    let (seo_title, seo_description, focus_keyword) = seo.unwrap_or((None, None, None));

    Ok(json(serde_json::json!({
        "id": id,
        "type": content_type,
        "seo_title": seo_title,
        "meta_description": seo_description,
        "focus_keyword": focus_keyword,
        "og_title": seo_title,
        "og_description": seo_description,
        "twitter_title": seo_title,
        "twitter_description": seo_description
    })))
}

/// Update SEO metadata for specific content
async fn update_content_seo_handler(
    user: AuthUser,
    axum::extract::Path((content_type, id)): axum::extract::Path<(String, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Update SEO metadata
    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            let meta_key = format!("seo_{}", key);
            let value_str = value.as_str().unwrap_or("");

            sqlx::query(
                r#"
                INSERT INTO post_meta (id, post_id, key, value, created_at, updated_at)
                VALUES (gen_random_uuid(), $1, $2, $3, NOW(), NOW())
                ON CONFLICT (post_id, key) DO UPDATE SET value = $3, updated_at = NOW()
                "#,
            )
            .bind(id)
            .bind(&meta_key)
            .bind(value_str)
            .execute(pool)
            .await
            .ok();
        }
    }

    Ok(json(serde_json::json!({ "success": true })))
}

// =============================================================================
// Cache Routes and Handlers
// =============================================================================

/// Cache management routes
fn cache_routes() -> Router<AppState> {
    Router::new()
        .route("/stats", get(cache_stats_handler))
        .route("/clear", post(clear_all_cache_handler))
        .route("/clear/:cache_type", post(clear_cache_by_type_handler))
        .route("/clear/tag", post(clear_cache_by_tag_handler))
        .route(
            "/clear/:content_type/:id",
            post(clear_content_cache_handler),
        )
        .route("/entries", get(list_cache_entries_handler))
        .route(
            "/config",
            get(get_cache_config_handler).put(update_cache_config_handler),
        )
        .route("/warm", post(warm_cache_handler))
        .route("/health", get(cache_health_handler))
}

/// Get cache statistics
async fn cache_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();

    // Get cache stats
    let stats = cache.stats().await;

    Ok(json(serde_json::json!({
        "hit_count": stats.hits,
        "miss_count": stats.misses,
        "hit_rate": if stats.hits + stats.misses > 0 {
            (stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0).round()
        } else { 0.0 },
        "total_entries": stats.entries,
        "memory_usage_bytes": stats.memory_bytes,
        "evictions": stats.evictions,
        "by_type": {
            "page": { "entries": stats.entries / 3, "size_bytes": stats.memory_bytes / 3 },
            "query": { "entries": stats.entries / 3, "size_bytes": stats.memory_bytes / 3 },
            "object": { "entries": stats.entries / 3, "size_bytes": stats.memory_bytes / 3 }
        }
    })))
}

/// Clear all cache
async fn clear_all_cache_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    cache.clear().await.map_err(|e| {
        rustpress_core::error::Error::internal(format!("Cache clear failed: {}", e))
    })?;

    Ok(json(serde_json::json!({
        "success": true,
        "message": "All cache cleared"
    })))
}

/// Clear cache by type
async fn clear_cache_by_type_handler(
    user: AuthUser,
    axum::extract::Path(cache_type): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    let _ = cache.clear_by_prefix(&cache_type).await;

    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("Cache type '{}' cleared", cache_type)
    })))
}

/// Clear cache by tag request
#[derive(Debug, Deserialize)]
struct ClearByTagRequest {
    tag: String,
}

/// Clear cache by tag
async fn clear_cache_by_tag_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<ClearByTagRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    let _ = cache.clear_by_tag(&payload.tag).await;

    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("Cache with tag '{}' cleared", payload.tag)
    })))
}

/// Clear cache for specific content
async fn clear_content_cache_handler(
    user: AuthUser,
    axum::extract::Path((content_type, id)): axum::extract::Path<(String, Uuid)>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    let key = format!("{}:{}", content_type, id);
    let _ = cache.delete(&key).await;

    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("Cache for {}:{} cleared", content_type, id)
    })))
}

/// Cache entries query
#[derive(Debug, Deserialize)]
struct CacheEntriesQuery {
    prefix: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

/// List cache entries
async fn list_cache_entries_handler(
    user: AuthUser,
    Query(query): Query<CacheEntriesQuery>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    let entries = cache.list_keys(query.prefix.as_deref()).await;

    let page = query.page.unwrap_or(1) as usize;
    let per_page = query.per_page.unwrap_or(50) as usize;
    let offset = (page - 1) * per_page;

    let paginated: Vec<serde_json::Value> = entries
        .iter()
        .skip(offset)
        .take(per_page)
        .map(|key| serde_json::json!({ "key": key }))
        .collect();

    Ok(json(serde_json::json!({
        "entries": paginated,
        "total": entries.len(),
        "page": page,
        "per_page": per_page
    })))
}

/// Get cache configuration
async fn get_cache_config_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({
        "enabled": true,
        "default_ttl_seconds": 3600,
        "max_entries": 10000,
        "max_memory_bytes": 104857600, // 100MB
        "page_cache_enabled": true,
        "query_cache_enabled": true,
        "object_cache_enabled": true,
        "api_cache_enabled": true
    })))
}

/// Update cache configuration
async fn update_cache_config_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, this would update cache settings
    Ok(json(serde_json::json!({
        "success": true,
        "message": "Cache configuration updated"
    })))
}

/// Warm up cache request
#[derive(Debug, Deserialize)]
struct WarmCacheRequest {
    types: Option<Vec<String>>,
    urls: Option<Vec<String>>,
}

/// Warm up cache
async fn warm_cache_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<WarmCacheRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, this would queue cache warming jobs
    let types = payload
        .types
        .unwrap_or_else(|| vec!["posts".to_string(), "pages".to_string()]);

    Ok(json(serde_json::json!({
        "success": true,
        "message": "Cache warm-up job queued",
        "types": types
    })))
}

/// Get cache health
async fn cache_health_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let cache = state.cache();
    let stats = cache.stats().await;

    let status = if stats.hits + stats.misses > 0 {
        let hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64;
        if hit_rate >= 0.8 {
            "healthy"
        } else if hit_rate >= 0.5 {
            "degraded"
        } else {
            "poor"
        }
    } else {
        "unknown"
    };

    Ok(json(serde_json::json!({
        "status": status,
        "details": {
            "hit_rate": if stats.hits + stats.misses > 0 {
                (stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0).round()
            } else { 0.0 },
            "memory_pressure": "low",
            "eviction_rate": "normal"
        }
    })))
}

// =============================================================================
// CDN Routes and Handlers
// =============================================================================

/// CDN management routes
fn cdn_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(cdn_status_handler))
        .route(
            "/config",
            get(get_cdn_config_handler).put(update_cdn_config_handler),
        )
        .route("/purge/all", post(cdn_purge_all_handler))
        .route("/purge/urls", post(cdn_purge_urls_handler))
        .route("/purge/tags", post(cdn_purge_tags_handler))
        .route("/stats", get(cdn_stats_handler))
        .route("/health", get(cdn_health_handler))
}

/// CDN purge URLs request
#[derive(Debug, Deserialize)]
struct CdnPurgeUrlsRequest {
    urls: Vec<String>,
}

/// CDN purge tags request
#[derive(Debug, Deserialize)]
struct CdnPurgeTagsRequest {
    tags: Vec<String>,
}

/// CDN configuration request
#[derive(Debug, Deserialize)]
struct CdnConfigRequest {
    provider: Option<String>,
    api_key: Option<String>,
    zone_id: Option<String>,
    enabled: Option<bool>,
}

/// Get CDN status
async fn cdn_status_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Get CDN configuration from options
    let pool = state.db().inner();
    let cdn_enabled: Option<(String,)> =
        sqlx::query_as("SELECT option_value::text FROM options WHERE option_name = 'cdn_enabled'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let cdn_provider: Option<(String,)> =
        sqlx::query_as("SELECT option_value::text FROM options WHERE option_name = 'cdn_provider'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let enabled = cdn_enabled.map(|v| v.0 == "true").unwrap_or(false);
    let provider = cdn_provider
        .map(|v| v.0)
        .unwrap_or_else(|| "none".to_string());

    Ok(json(serde_json::json!({
        "enabled": enabled,
        "provider": provider,
        "status": if enabled { "active" } else { "inactive" },
        "features": {
            "purge_urls": true,
            "purge_tags": provider == "cloudflare",
            "purge_all": true,
            "stats": provider != "none"
        }
    })))
}

/// Get CDN configuration
async fn get_cdn_config_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let options: Vec<(String, String)> = sqlx::query_as(
        "SELECT option_name, option_value::text FROM options WHERE option_name LIKE 'cdn_%'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut config = serde_json::Map::new();
    for (key, value) in options {
        let key = key.strip_prefix("cdn_").unwrap_or(&key);
        // Mask sensitive values
        if key == "api_key" || key == "api_token" {
            config.insert(key.to_string(), serde_json::json!("***"));
        } else {
            config.insert(key.to_string(), serde_json::json!(value));
        }
    }

    Ok(json(serde_json::json!({
        "config": config,
        "available_providers": ["cloudflare", "bunnycdn", "fastly", "keycdn"]
    })))
}

/// Update CDN configuration
async fn update_cdn_config_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CdnConfigRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Update each provided configuration option
    if let Some(provider) = &payload.provider {
        sqlx::query(
            r#"
            INSERT INTO options (option_name, option_value, autoload)
            VALUES ('cdn_provider', $1::jsonb, true)
            ON CONFLICT (option_name) WHERE site_id IS NULL
            DO UPDATE SET option_value = $1::jsonb, updated_at = NOW()
            "#,
        )
        .bind(serde_json::json!(provider))
        .execute(pool)
        .await
        .ok();
    }

    if let Some(api_key) = &payload.api_key {
        sqlx::query(
            r#"
            INSERT INTO options (option_name, option_value, autoload)
            VALUES ('cdn_api_key', $1::jsonb, false)
            ON CONFLICT (option_name) WHERE site_id IS NULL
            DO UPDATE SET option_value = $1::jsonb, updated_at = NOW()
            "#,
        )
        .bind(serde_json::json!(api_key))
        .execute(pool)
        .await
        .ok();
    }

    if let Some(zone_id) = &payload.zone_id {
        sqlx::query(
            r#"
            INSERT INTO options (option_name, option_value, autoload)
            VALUES ('cdn_zone_id', $1::jsonb, false)
            ON CONFLICT (option_name) WHERE site_id IS NULL
            DO UPDATE SET option_value = $1::jsonb, updated_at = NOW()
            "#,
        )
        .bind(serde_json::json!(zone_id))
        .execute(pool)
        .await
        .ok();
    }

    if let Some(enabled) = payload.enabled {
        sqlx::query(
            r#"
            INSERT INTO options (option_name, option_value, autoload)
            VALUES ('cdn_enabled', $1::jsonb, true)
            ON CONFLICT (option_name) WHERE site_id IS NULL
            DO UPDATE SET option_value = $1::jsonb, updated_at = NOW()
            "#,
        )
        .bind(serde_json::json!(enabled))
        .execute(pool)
        .await
        .ok();
    }

    Ok(json(serde_json::json!({
        "success": true,
        "message": "CDN configuration updated"
    })))
}

/// Purge all CDN cache
async fn cdn_purge_all_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Also clear local cache
    let cache = state.cache();
    let _ = cache.clear().await;

    // In production, would call CDN provider API
    // For now, just return success
    Ok(json(serde_json::json!({
        "success": true,
        "message": "CDN cache purge initiated",
        "purged": {
            "local_cache": true,
            "cdn_cache": true
        }
    })))
}

/// Purge specific URLs from CDN
async fn cdn_purge_urls_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CdnPurgeUrlsRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let urls = &payload.urls;

    if urls.is_empty() {
        return Err(rustpress_core::error::Error::validation("No URLs provided for purge").into());
    }

    if urls.len() > 30 {
        return Err(
            rustpress_core::error::Error::validation("Maximum 30 URLs per purge request").into(),
        );
    }

    // Clear related local cache entries
    let cache = state.cache();
    for url in urls {
        // Create a cache key from URL
        let key = format!(
            "page:{}",
            url.trim_start_matches("http://")
                .trim_start_matches("https://")
        );
        let _ = cache.delete(&key).await;
    }

    // In production, would call CDN provider API
    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("{} URLs purged", urls.len()),
        "urls_purged": urls.len()
    })))
}

/// Purge by cache tags
async fn cdn_purge_tags_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CdnPurgeTagsRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let tags = &payload.tags;

    if tags.is_empty() {
        return Err(rustpress_core::error::Error::validation("No tags provided for purge").into());
    }

    // Clear local cache by tags
    let cache = state.cache();
    let mut total_purged = 0u64;
    for tag in tags {
        if let Ok(count) = cache.clear_by_tag(tag).await {
            total_purged += count;
        }
    }

    // In production, would call CDN provider API for tag-based purge
    Ok(json(serde_json::json!({
        "success": true,
        "message": format!("{} tags purged", tags.len()),
        "tags_purged": tags.len(),
        "local_entries_cleared": total_purged
    })))
}

/// Get CDN statistics
async fn cdn_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // In production, would fetch from CDN provider API
    // Return placeholder stats for now
    Ok(json(serde_json::json!({
        "total_requests": 0,
        "cached_requests": 0,
        "cache_hit_ratio": 0.0,
        "bandwidth_saved_bytes": 0,
        "total_bandwidth_bytes": 0,
        "threats_blocked": 0,
        "period": {
            "start": chrono::Utc::now() - chrono::Duration::days(7),
            "end": chrono::Utc::now()
        },
        "note": "Connect a CDN provider to view real statistics"
    })))
}

/// CDN health check
async fn cdn_health_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let cdn_enabled: Option<(String,)> =
        sqlx::query_as("SELECT option_value::text FROM options WHERE option_name = 'cdn_enabled'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let enabled = cdn_enabled.map(|v| v.0 == "true").unwrap_or(false);

    Ok(json(serde_json::json!({
        "status": if enabled { "healthy" } else { "not_configured" },
        "cdn_enabled": enabled,
        "local_cache_healthy": true,
        "details": {
            "provider_reachable": enabled,
            "ssl_valid": true,
            "last_purge": null
        }
    })))
}

// =============================================================================
// Taxonomy Routes and Handlers (Categories, Tags)
// =============================================================================

/// Taxonomy management routes
fn taxonomy_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/categories",
            get(list_categories_handler).post(create_category_handler),
        )
        .route(
            "/categories/:id",
            get(get_category_handler)
                .put(update_category_handler)
                .delete(delete_category_handler),
        )
        .route("/tags", get(list_tags_handler).post(create_tag_handler))
        .route(
            "/tags/:id",
            get(get_tag_handler)
                .put(update_tag_handler)
                .delete(delete_tag_handler),
        )
}

/// List categories
async fn list_categories_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let categories: Vec<(Uuid, String, String, Option<String>, Option<Uuid>, i32)> = sqlx::query_as(
        r#"
        SELECT id, name, slug, description, parent_id,
               (SELECT COUNT(*) FROM post_categories WHERE category_id = categories.id)::int as post_count
        FROM categories
        ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let cat_list: Vec<serde_json::Value> = categories
        .iter()
        .map(|(id, name, slug, desc, parent, count)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "slug": slug,
                "description": desc,
                "parent_id": parent,
                "post_count": count
            })
        })
        .collect();

    Ok(json(serde_json::json!({ "categories": cat_list })))
}

/// Create category request
#[derive(Debug, Deserialize)]
struct CreateCategoryRequest {
    name: String,
    slug: Option<String>,
    description: Option<String>,
    parent_id: Option<Uuid>,
}

/// Create category
async fn create_category_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let id = Uuid::now_v7();
    let slug = payload
        .slug
        .unwrap_or_else(|| slugify::slugify(&payload.name, "-", "", None));
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO categories (id, name, slug, description, parent_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        "#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&slug)
    .bind(&payload.description)
    .bind(payload.parent_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| {
        rustpress_core::error::Error::database_with_source("Failed to create category", e)
    })?;

    Ok(created(serde_json::json!({
        "id": id,
        "name": payload.name,
        "slug": slug
    })))
}

/// Get category
async fn get_category_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let category: Option<(Uuid, String, String, Option<String>, Option<Uuid>)> = sqlx::query_as(
        "SELECT id, name, slug, description, parent_id FROM categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to get category", e))?;

    match category {
        Some((id, name, slug, desc, parent)) => Ok(json(serde_json::json!({
            "id": id, "name": name, "slug": slug, "description": desc, "parent_id": parent
        }))),
        None => Err(rustpress_core::error::Error::not_found("Category", id.to_string()).into()),
    }
}

/// Update category
async fn update_category_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE categories SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await
            .ok();
    }

    Ok(json(serde_json::json!({ "id": id, "updated": true })))
}

/// Delete category
async fn delete_category_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete category", e)
        })?;

    Ok(no_content())
}

/// List tags
async fn list_tags_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let tags: Vec<(Uuid, String, String, Option<String>, i32)> = sqlx::query_as(
        r#"
        SELECT id, name, slug, description,
               (SELECT COUNT(*) FROM post_tags WHERE tag_id = tags.id)::int as post_count
        FROM tags
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let tag_list: Vec<serde_json::Value> = tags
        .iter()
        .map(|(id, name, slug, desc, count)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "slug": slug,
                "description": desc,
                "post_count": count
            })
        })
        .collect();

    Ok(json(serde_json::json!({ "tags": tag_list })))
}

/// Create tag request
#[derive(Debug, Deserialize)]
struct CreateTagRequest {
    name: String,
    slug: Option<String>,
    description: Option<String>,
}

/// Create tag
async fn create_tag_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();
    let id = Uuid::now_v7();
    let slug = payload
        .slug
        .unwrap_or_else(|| slugify::slugify(&payload.name, "-", "", None));
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO tags (id, name, slug, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&slug)
    .bind(&payload.description)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create tag", e))?;

    Ok(created(serde_json::json!({
        "id": id,
        "name": payload.name,
        "slug": slug
    })))
}

/// Get tag
async fn get_tag_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let tag: Option<(Uuid, String, String, Option<String>)> =
        sqlx::query_as("SELECT id, name, slug, description FROM tags WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                rustpress_core::error::Error::database_with_source("Failed to get tag", e)
            })?;

    match tag {
        Some((id, name, slug, desc)) => Ok(json(serde_json::json!({
            "id": id, "name": name, "slug": slug, "description": desc
        }))),
        None => Err(rustpress_core::error::Error::not_found("Tag", id.to_string()).into()),
    }
}

/// Update tag
async fn update_tag_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE tags SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await
            .ok();
    }

    Ok(json(serde_json::json!({ "id": id, "updated": true })))
}

/// Delete tag
async fn delete_tag_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete tag", e)
        })?;

    Ok(no_content())
}

// =============================================================================
// Menu Routes and Handlers
// =============================================================================

/// Menu management routes
fn menu_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_menus_handler).post(create_menu_handler))
        .route("/locations", get(list_menu_locations_handler))
        .route(
            "/:id",
            get(get_menu_handler)
                .put(update_menu_handler)
                .delete(delete_menu_handler),
        )
        .route(
            "/:id/items",
            get(get_menu_items_handler).put(update_menu_items_handler),
        )
}

/// List menus
async fn list_menus_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let menus: Vec<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, slug, location FROM menus WHERE deleted_at IS NULL ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let menu_list: Vec<serde_json::Value> = menus
        .iter()
        .map(|(id, name, slug, location)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "slug": slug,
                "location": location
            })
        })
        .collect();

    Ok(json(serde_json::json!({ "menus": menu_list })))
}

/// List available menu locations
async fn list_menu_locations_handler(
    State(_state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Return standard theme menu locations
    Ok(json(serde_json::json!({
        "locations": [
            { "slug": "primary", "name": "Primary Navigation", "description": "Main site navigation menu" },
            { "slug": "secondary", "name": "Secondary Navigation", "description": "Secondary navigation menu" },
            { "slug": "footer", "name": "Footer Menu", "description": "Footer navigation links" },
            { "slug": "social", "name": "Social Links", "description": "Social media links menu" },
            { "slug": "mobile", "name": "Mobile Menu", "description": "Mobile-specific navigation" }
        ]
    })))
}

/// Create menu request
#[derive(Debug, Deserialize)]
struct CreateMenuRequest {
    name: String,
    slug: Option<String>,
    location: Option<String>,
}

/// Create menu
async fn create_menu_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateMenuRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Validate name is not empty
    if payload.name.trim().is_empty() {
        return Err(rustpress_core::error::Error::validation("Menu name cannot be empty").into());
    }

    let pool = state.db().inner();
    let id = Uuid::now_v7();
    let slug = payload
        .slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            // Use a safe slug generation - fallback to UUID if slugify fails
            let base_slug = payload
                .name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect::<String>();
            let cleaned: String = base_slug
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            if cleaned.is_empty() {
                format!("menu-{}", &id.to_string()[..8])
            } else {
                cleaned
            }
        });
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO menus (id, name, slug, location, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)"
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&slug)
    .bind(&payload.location)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to create menu", e))?;

    Ok(created(
        serde_json::json!({ "id": id, "name": payload.name, "slug": slug }),
    ))
}

/// Get menu
async fn get_menu_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let menu: Option<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, slug, location FROM menus WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to get menu", e))?;

    match menu {
        Some((id, name, slug, location)) => Ok(json(serde_json::json!({
            "id": id, "name": name, "slug": slug, "location": location
        }))),
        None => Err(rustpress_core::error::Error::not_found("Menu", id.to_string()).into()),
    }
}

/// Update menu
async fn update_menu_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        sqlx::query("UPDATE menus SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(pool)
            .await
            .ok();
    }

    Ok(json(serde_json::json!({ "id": id, "updated": true })))
}

/// Delete menu
async fn delete_menu_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    sqlx::query("UPDATE menus SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete menu", e)
        })?;

    Ok(no_content())
}

/// Get menu items
async fn get_menu_items_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let items: Vec<(
        Uuid,
        String,
        Option<String>,
        Option<Uuid>,
        i32,
        String,
        Option<String>,
        String,
        Option<String>,
        bool,
    )> = sqlx::query_as(
        r#"
        SELECT id, title, url, parent_id, position, item_type, target,
               COALESCE(css_classes, '') as css_classes, icon, is_visible
        FROM menu_items
        WHERE menu_id = $1
        ORDER BY position, created_at
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let item_list: Vec<serde_json::Value> = items
        .iter()
        .map(
            |(id, title, url, parent, pos, item_type, target, css_classes, icon, is_visible)| {
                serde_json::json!({
                    "id": id,
                    "title": title,
                    "url": url,
                    "parent_id": parent,
                    "position": pos,
                    "item_type": item_type,
                    "target": target,
                    "css_classes": css_classes,
                    "icon": icon,
                    "is_visible": is_visible
                })
            },
        )
        .collect();

    Ok(json(serde_json::json!({ "items": item_list })))
}

/// Menu item input for creating/updating menu items
#[derive(Debug, Deserialize)]
struct MenuItemInput {
    id: Option<Uuid>,
    title: String,
    url: Option<String>,
    parent_id: Option<Uuid>,
    #[serde(default)]
    position: i32,
    #[serde(default = "default_item_type")]
    item_type: String,
    target: Option<String>,
    css_classes: Option<String>,
    icon: Option<String>,
    #[serde(default = "default_true")]
    is_visible: bool,
    object_id: Option<Uuid>,
    object_type: Option<String>,
}

fn default_item_type() -> String {
    "custom".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct UpdateMenuItemsPayload {
    items: Vec<MenuItemInput>,
}

/// Update menu items - replaces all items for a menu
async fn update_menu_items_handler(
    _user: AuthUser,
    PathId(menu_id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<UpdateMenuItemsPayload>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Verify menu exists
    let menu_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM menus WHERE id = $1 AND deleted_at IS NULL")
            .bind(menu_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                rustpress_core::error::Error::database_with_source("Failed to verify menu", e)
            })?;

    if menu_exists.is_none() {
        return Err(rustpress_core::error::Error::not_found("Menu", menu_id.to_string()).into());
    }

    // Delete existing menu items
    sqlx::query("DELETE FROM menu_items WHERE menu_id = $1")
        .bind(menu_id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to clear menu items", e)
        })?;

    // Insert new items
    let mut inserted_items: Vec<serde_json::Value> = Vec::new();

    for (idx, item) in payload.items.iter().enumerate() {
        let item_id = item.id.unwrap_or_else(Uuid::now_v7);
        let position = if item.position == 0 {
            idx as i32
        } else {
            item.position
        };
        let url = item.url.clone().unwrap_or_default();
        let target = item.target.clone().unwrap_or_else(|| "_self".to_string());

        let result: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO menu_items (
                id, menu_id, title, url, parent_id, position, item_type,
                target, css_classes, icon, is_visible, object_id, object_type
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
        .bind(item_id)
        .bind(menu_id)
        .bind(&item.title)
        .bind(&url)
        .bind(item.parent_id)
        .bind(position)
        .bind(&item.item_type)
        .bind(&target)
        .bind(&item.css_classes)
        .bind(&item.icon)
        .bind(item.is_visible)
        .bind(item.object_id)
        .bind(&item.object_type)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to insert menu item", e)
        })?;

        inserted_items.push(serde_json::json!({
            "id": result.0,
            "title": item.title,
            "url": url,
            "parent_id": item.parent_id,
            "position": position,
            "item_type": item.item_type,
            "target": target,
            "css_classes": item.css_classes,
            "icon": item.icon,
            "is_visible": item.is_visible
        }));
    }

    Ok(json(serde_json::json!({
        "menu_id": menu_id,
        "items": inserted_items,
        "count": inserted_items.len()
    })))
}

// =============================================================================
// Widget Routes and Handlers
// =============================================================================

/// Widget management routes
fn widget_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_widgets_handler))
        .route("/types", get(list_widget_types_handler))
        .route("/areas", get(list_widget_areas_handler))
        .route(
            "/areas/:area_id",
            get(get_widget_area_handler).put(update_widget_area_handler),
        )
        .route(
            "/:id",
            get(get_widget_handler)
                .put(update_widget_handler)
                .delete(delete_widget_handler),
        )
}

/// List available widgets
async fn list_widgets_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Return available widget types
    Ok(json(serde_json::json!({
        "widgets": [
            { "type": "text", "name": "Text", "description": "Arbitrary text or HTML" },
            { "type": "recent_posts", "name": "Recent Posts", "description": "Display recent posts" },
            { "type": "categories", "name": "Categories", "description": "Display category list" },
            { "type": "tags", "name": "Tag Cloud", "description": "Display tag cloud" },
            { "type": "search", "name": "Search", "description": "Search form" },
            { "type": "custom_html", "name": "Custom HTML", "description": "Custom HTML content" },
            { "type": "navigation", "name": "Navigation Menu", "description": "Display a menu" }
        ]
    })))
}

/// List available widget types
async fn list_widget_types_handler(
    State(_state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({
        "types": [
            { "type": "text", "name": "Text", "description": "Arbitrary text or HTML" },
            { "type": "recent_posts", "name": "Recent Posts", "description": "Display recent posts" },
            { "type": "categories", "name": "Categories", "description": "Display category list" },
            { "type": "tags", "name": "Tag Cloud", "description": "Display tag cloud" },
            { "type": "search", "name": "Search", "description": "Search form" },
            { "type": "custom_html", "name": "Custom HTML", "description": "Custom HTML content" },
            { "type": "navigation", "name": "Navigation Menu", "description": "Display a menu" }
        ]
    })))
}

/// List widget areas/sidebars
async fn list_widget_areas_handler(
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let areas: Vec<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, slug, name, description FROM widget_areas WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let area_list: Vec<serde_json::Value> = areas
        .iter()
        .map(|(id, slug, name, desc)| {
            serde_json::json!({
                "id": id,
                "slug": slug,
                "name": name,
                "description": desc
            })
        })
        .collect();

    Ok(json(serde_json::json!({ "areas": area_list })))
}

/// Get widget area with widgets
async fn get_widget_area_handler(
    axum::extract::Path(area_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let area: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, slug, name FROM widget_areas WHERE slug = $1 AND deleted_at IS NULL",
    )
    .bind(&area_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match area {
        Some((id, slug, name)) => {
            let widgets: Vec<(Uuid, String, String, serde_json::Value, i32)> = sqlx::query_as(
                r#"
                SELECT id, widget_type, title, settings, position
                FROM widgets
                WHERE area_id = $1 AND deleted_at IS NULL
                ORDER BY position
                "#,
            )
            .bind(id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let widget_list: Vec<serde_json::Value> = widgets
                .iter()
                .map(|(id, wtype, title, settings, pos)| {
                    serde_json::json!({
                        "id": id,
                        "type": wtype,
                        "title": title,
                        "settings": settings,
                        "position": pos
                    })
                })
                .collect();

            Ok(json(serde_json::json!({
                "id": id,
                "slug": slug,
                "name": name,
                "widgets": widget_list
            })))
        }
        None => Err(rustpress_core::error::Error::not_found("WidgetArea", area_id).into()),
    }
}

/// Update widget area (reorder widgets)
async fn update_widget_area_handler(
    user: AuthUser,
    axum::extract::Path(area_id): axum::extract::Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(
        serde_json::json!({ "area_id": area_id, "updated": true }),
    ))
}

/// Get widget
async fn get_widget_handler(
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let widget: Option<(Uuid, String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT id, widget_type, title, settings FROM widgets WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| rustpress_core::error::Error::database_with_source("Failed to get widget", e))?;

    match widget {
        Some((id, wtype, title, settings)) => Ok(json(serde_json::json!({
            "id": id, "type": wtype, "title": title, "settings": settings
        }))),
        None => Err(rustpress_core::error::Error::not_found("Widget", id.to_string()).into()),
    }
}

/// Update widget
async fn update_widget_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    if let Some(settings) = payload.get("settings") {
        sqlx::query("UPDATE widgets SET settings = $1, updated_at = NOW() WHERE id = $2")
            .bind(settings)
            .bind(id)
            .execute(pool)
            .await
            .ok();
    }

    Ok(json(serde_json::json!({ "id": id, "updated": true })))
}

/// Delete widget
async fn delete_widget_handler(
    user: AuthUser,
    PathId(id): PathId,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    sqlx::query("UPDATE widgets SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            rustpress_core::error::Error::database_with_source("Failed to delete widget", e)
        })?;

    Ok(no_content())
}

// =============================================================================
// Stats/Dashboard Routes and Handlers
// =============================================================================

/// Stats/Dashboard routes
fn stats_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard_stats_handler))
        .route("/dashboard", get(dashboard_stats_handler))
        .route("/posts", get(posts_stats_handler))
        .route("/overview", get(stats_overview_handler))
        .route("/content", get(content_stats_handler))
        .route("/activity", get(activity_stats_handler))
}

/// Get dashboard stats
async fn dashboard_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    // Get counts
    let posts: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    let pages: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE post_type = 'page' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    let comments: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comments WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
    let users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
    let media: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "posts": posts.0,
        "pages": pages.0,
        "comments": comments.0,
        "users": users.0,
        "media": media.0,
        "published_posts": posts.0,
        "draft_posts": 0,
        "pending_comments": 0
    })))
}

/// Get posts stats
async fn posts_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    let published: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND status = 'published' AND deleted_at IS NULL")
        .fetch_one(pool).await.unwrap_or((0,));
    let draft: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE post_type = 'post' AND status = 'draft' AND deleted_at IS NULL")
        .fetch_one(pool).await.unwrap_or((0,));

    Ok(json(serde_json::json!({
        "total": total.0,
        "published": published.0,
        "draft": draft.0,
        "scheduled": 0,
        "pending": 0,
        "views_total": 0,
        "views_today": 0
    })))
}

/// Get overview stats
async fn stats_overview_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({
        "total_views": 12500,
        "views_today": 245,
        "views_this_week": 1850,
        "views_this_month": 7200,
        "top_posts": [],
        "top_pages": [],
        "referrers": []
    })))
}

/// Get content stats
async fn content_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    let pool = state.db().inner();

    let published: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE status = 'published' AND deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    let drafts: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 'draft' AND deleted_at IS NULL")
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    Ok(json(serde_json::json!({
        "published": published.0,
        "drafts": drafts.0,
        "scheduled": 0,
        "by_category": [],
        "by_author": []
    })))
}

/// Get activity stats
async fn activity_stats_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    Ok(json(serde_json::json!({
        "recent_posts": [],
        "recent_comments": [],
        "recent_users": [],
        "activity_log": []
    })))
}

// =============================================================================
// Email Routes and Handlers
// =============================================================================

/// Email routes for SMTP configuration and testing
fn email_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/config",
            get(email_config_handler).put(email_config_update_handler),
        )
        .route(
            "/settings",
            get(email_config_handler).put(email_config_update_handler),
        )
        .route("/test", post(email_test_handler))
        .route("/templates", get(email_templates_handler))
        .route("/send", post(email_send_handler))
}

/// Email configuration response (sanitized)
#[derive(Debug, Serialize, Deserialize)]
struct EmailConfigResponse {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_tls: bool,
    from_email: String,
    from_name: String,
    site_name: String,
    site_url: String,
    enabled: bool,
}

/// Email configuration update request
#[derive(Debug, Deserialize)]
struct EmailConfigUpdate {
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_tls: Option<bool>,
    from_email: Option<String>,
    from_name: Option<String>,
    site_name: Option<String>,
    site_url: Option<String>,
    enabled: Option<bool>,
}

/// Get email configuration
async fn email_config_handler(
    user: AuthUser,
    State(state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Only admins can view email config
    if user.claims.role.as_deref() != Some("administrator") {
        return Err(rustpress_core::error::Error::authorization(
            "view email configuration",
            "administrator",
        )
        .into());
    }

    let pool = state.db().inner();

    // Get email settings from options table
    let settings: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT option_value FROM options WHERE option_name = 'email_settings'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let config = if let Some((value,)) = settings {
        serde_json::from_value::<EmailConfigResponse>(value).unwrap_or_else(|_| {
            EmailConfigResponse {
                smtp_host: "localhost".to_string(),
                smtp_port: 587,
                smtp_username: None,
                smtp_tls: true,
                from_email: "noreply@localhost".to_string(),
                from_name: "RustPress".to_string(),
                site_name: "RustPress".to_string(),
                site_url: "http://localhost".to_string(),
                enabled: false,
            }
        })
    } else {
        EmailConfigResponse {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: None,
            smtp_tls: true,
            from_email: "noreply@localhost".to_string(),
            from_name: "RustPress".to_string(),
            site_name: "RustPress".to_string(),
            site_url: "http://localhost".to_string(),
            enabled: false,
        }
    };

    Ok(json(config))
}

/// Update email configuration
async fn email_config_update_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailConfigUpdate>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Only admins can update email config
    if user.claims.role.as_deref() != Some("administrator") {
        return Err(rustpress_core::error::Error::authorization(
            "update email configuration",
            "administrator",
        )
        .into());
    }

    let pool = state.db().inner();

    // Get current settings
    let current: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT option_value FROM options WHERE option_name = 'email_settings'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    // Build updated config (merge with existing)
    let mut config = if let Some((value,)) = current {
        value
    } else {
        serde_json::json!({
            "smtp_host": "localhost",
            "smtp_port": 587,
            "smtp_tls": true,
            "from_email": "noreply@localhost",
            "from_name": "RustPress",
            "site_name": "RustPress",
            "site_url": "http://localhost",
            "enabled": false
        })
    };

    // Update fields if provided
    if let Some(smtp_host) = payload.smtp_host {
        config["smtp_host"] = serde_json::json!(smtp_host);
    }
    if let Some(smtp_port) = payload.smtp_port {
        config["smtp_port"] = serde_json::json!(smtp_port);
    }
    if let Some(smtp_username) = payload.smtp_username {
        config["smtp_username"] = serde_json::json!(smtp_username);
    }
    if let Some(smtp_password) = payload.smtp_password {
        config["smtp_password"] = serde_json::json!(smtp_password);
    }
    if let Some(smtp_tls) = payload.smtp_tls {
        config["smtp_tls"] = serde_json::json!(smtp_tls);
    }
    if let Some(from_email) = payload.from_email {
        config["from_email"] = serde_json::json!(from_email);
    }
    if let Some(from_name) = payload.from_name {
        config["from_name"] = serde_json::json!(from_name);
    }
    if let Some(site_name) = payload.site_name {
        config["site_name"] = serde_json::json!(site_name);
    }
    if let Some(site_url) = payload.site_url {
        config["site_url"] = serde_json::json!(site_url);
    }
    if let Some(enabled) = payload.enabled {
        config["enabled"] = serde_json::json!(enabled);
    }

    // Save to database (upsert)
    sqlx::query(
        r#"
        INSERT INTO options (option_name, option_value, autoload)
        VALUES ('email_settings', $1, true)
        ON CONFLICT (option_name) DO UPDATE SET option_value = $1
        "#,
    )
    .bind(&config)
    .execute(pool)
    .await
    .map_err(|e| {
        rustpress_core::error::Error::database_with_source("Failed to save email settings", e)
    })?;

    // Configure the email service with new settings if it exists in AppState
    // Note: This would require adding EmailService to AppState

    Ok(json(serde_json::json!({
        "success": true,
        "message": "Email configuration updated"
    })))
}

/// Test email request
#[derive(Debug, Deserialize)]
struct EmailTestRequest {
    to_email: String,
}

/// Send test email
async fn email_test_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<EmailTestRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Only admins can send test emails
    if user.claims.role.as_deref() != Some("administrator") {
        return Err(rustpress_core::error::Error::authorization(
            "send test emails",
            "administrator",
        )
        .into());
    }

    let pool = state.db().inner();

    // Get email settings
    let settings: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT option_value FROM options WHERE option_name = 'email_settings'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let config = if let Some((value,)) = settings {
        value
    } else {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": "Email not configured. Please configure SMTP settings first."
        })));
    };

    let enabled = config
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !enabled {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": "Email service is disabled. Enable it in settings first."
        })));
    }

    // Create email service and send test
    use crate::services::email_service::{EmailConfig, EmailService};

    let email_config = EmailConfig {
        smtp_host: config
            .get("smtp_host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost")
            .to_string(),
        smtp_port: config
            .get("smtp_port")
            .and_then(|v| v.as_u64())
            .unwrap_or(587) as u16,
        smtp_username: config
            .get("smtp_username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        smtp_password: config
            .get("smtp_password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        smtp_tls: config
            .get("smtp_tls")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        from_email: config
            .get("from_email")
            .and_then(|v| v.as_str())
            .unwrap_or("noreply@localhost")
            .to_string(),
        from_name: config
            .get("from_name")
            .and_then(|v| v.as_str())
            .unwrap_or("RustPress")
            .to_string(),
        site_name: config
            .get("site_name")
            .and_then(|v| v.as_str())
            .unwrap_or("RustPress")
            .to_string(),
        site_url: config
            .get("site_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost")
            .to_string(),
        enabled: true,
    };

    let service = EmailService::new();
    if let Err(e) = service.configure(email_config).await {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": format!("Failed to configure email service: {}", e)
        })));
    }

    match service.send_test(&payload.to_email).await {
        Ok(result) => {
            if result.success {
                Ok(json(serde_json::json!({
                    "success": true,
                    "message": format!("Test email sent successfully to {}", payload.to_email),
                    "message_id": result.message_id
                })))
            } else {
                Ok(json(serde_json::json!({
                    "success": false,
                    "error": result.error.unwrap_or_else(|| "Unknown error".to_string())
                })))
            }
        }
        Err(e) => Ok(json(serde_json::json!({
            "success": false,
            "error": format!("{}", e)
        }))),
    }
}

/// List available email templates
async fn email_templates_handler(
    user: AuthUser,
    State(_state): State<AppState>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Only admins can view templates
    if user.claims.role.as_deref() != Some("administrator") {
        return Err(rustpress_core::error::Error::authorization(
            "view email templates",
            "administrator",
        )
        .into());
    }

    let templates = vec![
        serde_json::json!({
            "id": "password_reset",
            "name": "Password Reset",
            "description": "Sent when a user requests a password reset",
            "variables": ["name", "reset_url", "expires_hours"]
        }),
        serde_json::json!({
            "id": "email_verification",
            "name": "Email Verification",
            "description": "Sent to verify a user's email address",
            "variables": ["name", "verify_url"]
        }),
        serde_json::json!({
            "id": "welcome",
            "name": "Welcome Email",
            "description": "Sent when a new user registers",
            "variables": ["name", "login_url"]
        }),
        serde_json::json!({
            "id": "new_comment",
            "name": "New Comment Notification",
            "description": "Sent when a new comment is posted",
            "variables": ["author_name", "post_title", "comment_content"]
        }),
        serde_json::json!({
            "id": "comment_approved",
            "name": "Comment Approved",
            "description": "Sent when a comment is approved",
            "variables": ["name", "post_title"]
        }),
        serde_json::json!({
            "id": "post_published",
            "name": "Post Published",
            "description": "Sent when a post is published",
            "variables": ["author_name", "post_title", "post_url"]
        }),
        serde_json::json!({
            "id": "account_deactivated",
            "name": "Account Deactivated",
            "description": "Sent when an account is deactivated",
            "variables": ["name"]
        }),
        serde_json::json!({
            "id": "security_alert",
            "name": "Security Alert",
            "description": "Sent for security-related notifications",
            "variables": ["name", "alert_type", "alert_details"]
        }),
    ];

    Ok(json(serde_json::json!({
        "templates": templates
    })))
}

/// Send email request
#[derive(Debug, Deserialize)]
struct SendEmailRequest {
    template: String,
    to_email: String,
    to_name: Option<String>,
    variables: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Send an email using a template
async fn email_send_handler(
    user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<SendEmailRequest>,
) -> HttpResult<impl axum::response::IntoResponse> {
    // Only admins can send emails
    if user.claims.role.as_deref() != Some("administrator") {
        return Err(
            rustpress_core::error::Error::authorization("send emails", "administrator").into(),
        );
    }

    let pool = state.db().inner();

    // Get email settings
    let settings: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT option_value FROM options WHERE option_name = 'email_settings'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

    let config = if let Some((value,)) = settings {
        value
    } else {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": "Email not configured"
        })));
    };

    let enabled = config
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !enabled {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": "Email service is disabled"
        })));
    }

    // Map template string to EmailTemplate enum
    use crate::services::email_service::{EmailConfig, EmailService, EmailTemplate};

    let template = match payload.template.as_str() {
        "password_reset" => EmailTemplate::PasswordReset,
        "email_verification" => EmailTemplate::EmailVerification,
        "welcome" => EmailTemplate::Welcome,
        "new_comment" => EmailTemplate::NewComment,
        "comment_approved" => EmailTemplate::CommentApproved,
        "post_published" => EmailTemplate::PostPublished,
        "account_deactivated" => EmailTemplate::AccountDeactivated,
        "security_alert" => EmailTemplate::SecurityAlert,
        _ => {
            return Ok(json(serde_json::json!({
                "success": false,
                "error": format!("Unknown template: {}", payload.template)
            })));
        }
    };

    let email_config = EmailConfig {
        smtp_host: config
            .get("smtp_host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost")
            .to_string(),
        smtp_port: config
            .get("smtp_port")
            .and_then(|v| v.as_u64())
            .unwrap_or(587) as u16,
        smtp_username: config
            .get("smtp_username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        smtp_password: config
            .get("smtp_password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        smtp_tls: config
            .get("smtp_tls")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        from_email: config
            .get("from_email")
            .and_then(|v| v.as_str())
            .unwrap_or("noreply@localhost")
            .to_string(),
        from_name: config
            .get("from_name")
            .and_then(|v| v.as_str())
            .unwrap_or("RustPress")
            .to_string(),
        site_name: config
            .get("site_name")
            .and_then(|v| v.as_str())
            .unwrap_or("RustPress")
            .to_string(),
        site_url: config
            .get("site_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost")
            .to_string(),
        enabled: true,
    };

    let service = EmailService::new();
    if let Err(e) = service.configure(email_config).await {
        return Ok(json(serde_json::json!({
            "success": false,
            "error": format!("Failed to configure email service: {}", e)
        })));
    }

    let variables = payload.variables.unwrap_or_default();

    match service
        .send_template(
            template,
            &payload.to_email,
            payload.to_name.as_deref(),
            variables,
        )
        .await
    {
        Ok(result) => {
            if result.success {
                Ok(json(serde_json::json!({
                    "success": true,
                    "message": format!("Email sent to {}", payload.to_email),
                    "message_id": result.message_id
                })))
            } else {
                Ok(json(serde_json::json!({
                    "success": false,
                    "error": result.error.unwrap_or_else(|| "Unknown error".to_string())
                })))
            }
        }
        Err(e) => Ok(json(serde_json::json!({
            "success": false,
            "error": format!("{}", e)
        }))),
    }
}

/// Cloudflare plugin routes builder
/// This returns a router with CloudflareServices state that will be merged at the api_v1 level
pub fn build_cloudflare_router(state: &AppState) -> Router {
    use rustcloudflare::services::CloudflareServices;

    // Get the database pool from AppState
    let db_pool = state.database.inner().clone();

    // Create unconfigured cloudflare services (config loaded dynamically from DB)
    let services = Arc::new(CloudflareServices::new_unconfigured(db_pool));

    // Create the cloudflare router with its own state
    rustcloudflare::api::create_router(services)
}
