//! Setup Wizard Module
//!
//! Provides a web-based setup wizard for initial RustPress configuration.
//! This module handles database connection setup, schema installation,
//! and initial configuration when RustPress is first started.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::{
    extract::{Json, State},
    response::Html,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Setup wizard state
#[derive(Clone)]
pub struct SetupState {
    pub config_path: PathBuf,
    pub setup_complete: Arc<RwLock<bool>>,
}

impl SetupState {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            setup_complete: Arc::new(RwLock::new(false)),
        }
    }
}

/// Database configuration from setup wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn to_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// Request to test database connection
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// Response from connection test
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

/// Request to install schema
#[derive(Debug, Deserialize)]
pub struct InstallSchemaRequest {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub admin_email: String,
    pub admin_username: String,
    pub admin_password: String,
    pub site_title: String,
}

/// Response from schema installation
#[derive(Debug, Serialize)]
pub struct InstallSchemaResponse {
    pub success: bool,
    pub message: String,
}

/// RustPress configuration file structure (nested for compatibility)
#[derive(Debug, Serialize, Deserialize)]
pub struct RustPressConfig {
    pub database: DatabaseSection,
    pub server: ServerSection,
    pub auth: AuthSection,
    pub setup_complete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSection {
    pub database_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSection {
    pub jwt_secret: String,
}

/// Check if setup is needed
pub fn needs_setup(config_path: &PathBuf) -> bool {
    if !config_path.exists() {
        return true;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => match toml::from_str::<RustPressConfig>(&content) {
            Ok(config) => !config.setup_complete,
            Err(_) => true,
        },
        Err(_) => true,
    }
}

/// Get database URL from config if available
pub fn get_database_url(config_path: &PathBuf) -> Option<String> {
    if !config_path.exists() {
        return None;
    }

    fs::read_to_string(config_path)
        .ok()
        .and_then(|content| toml::from_str::<RustPressConfig>(&content).ok())
        .filter(|config| config.setup_complete)
        .map(|config| config.database.database_url)
}

/// Create the setup wizard router
pub fn create_setup_router(state: SetupState) -> Router {
    Router::new()
        .route("/", get(setup_page))
        .route("/setup", get(setup_page))
        .route("/api/setup/test-connection", post(test_connection))
        .route("/api/setup/install", post(install_schema))
        .route("/api/setup/status", get(setup_status))
        .with_state(state)
}

/// Serve the setup page HTML
async fn setup_page() -> Html<&'static str> {
    Html(SETUP_PAGE_HTML)
}

/// Check setup status
async fn setup_status(State(state): State<SetupState>) -> Json<serde_json::Value> {
    let complete = *state.setup_complete.read().await;
    Json(serde_json::json!({
        "setup_complete": complete
    }))
}

/// Test database connection
async fn test_connection(Json(req): Json<TestConnectionRequest>) -> Json<TestConnectionResponse> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        req.username, req.password, req.host, req.port, req.database
    );

    info!(
        "Testing database connection to {}:{}/{}",
        req.host, req.port, req.database
    );

    match PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            // Try a simple query to verify connection
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => {
                    info!("Database connection successful");
                    Json(TestConnectionResponse {
                        success: true,
                        message: "Connection successful!".to_string(),
                        details: Some(format!(
                            "Connected to PostgreSQL at {}:{}",
                            req.host, req.port
                        )),
                    })
                }
                Err(e) => {
                    error!("Database query failed: {}", e);
                    Json(TestConnectionResponse {
                        success: false,
                        message: "Connection failed".to_string(),
                        details: Some(format!("Query error: {}", e)),
                    })
                }
            }
        }
        Err(e) => {
            error!("Database connection failed: {}", e);
            Json(TestConnectionResponse {
                success: false,
                message: "Connection failed".to_string(),
                details: Some(format!("Error: {}", e)),
            })
        }
    }
}

/// Install the database schema
async fn install_schema(
    State(state): State<SetupState>,
    Json(req): Json<InstallSchemaRequest>,
) -> Json<InstallSchemaResponse> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        req.username, req.password, req.host, req.port, req.database
    );

    info!("Installing RustPress schema...");

    // Connect to database
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Json(InstallSchemaResponse {
                success: false,
                message: format!("Database connection failed: {}", e),
            });
        }
    };

    // Hash the admin password
    let password_hash = match hash_password(&req.admin_password) {
        Ok(hash) => hash,
        Err(e) => {
            return Json(InstallSchemaResponse {
                success: false,
                message: format!("Failed to hash password: {}", e),
            });
        }
    };

    // Run schema installation
    if let Err(e) = run_schema_installation(&pool, &req, &password_hash).await {
        error!("Schema installation failed: {}", e);
        return Json(InstallSchemaResponse {
            success: false,
            message: format!("Schema installation failed: {}", e),
        });
    }

    // Generate JWT secret
    let jwt_secret = generate_jwt_secret();

    // Save configuration
    let config = RustPressConfig {
        database: DatabaseSection { database_url },
        server: ServerSection {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        auth: AuthSection { jwt_secret },
        setup_complete: true,
    };

    if let Err(e) = save_config(&state.config_path, &config) {
        return Json(InstallSchemaResponse {
            success: false,
            message: format!("Failed to save configuration: {}", e),
        });
    }

    // Mark setup as complete
    *state.setup_complete.write().await = true;

    info!("RustPress setup completed successfully!");
    Json(InstallSchemaResponse {
        success: true,
        message:
            "RustPress has been installed successfully! The server will restart automatically."
                .to_string(),
    })
}

/// Hash password using argon2
fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Generate a random JWT secret
fn generate_jwt_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    bytes
        .iter()
        .map(|b| {
            let idx = (*b as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

/// Save configuration to file
fn save_config(path: &PathBuf, config: &RustPressConfig) -> Result<(), std::io::Error> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// Run the schema installation SQL
async fn run_schema_installation(
    pool: &sqlx::PgPool,
    req: &InstallSchemaRequest,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    // Execute consolidated schema using raw_sql which supports multiple statements
    sqlx::raw_sql(SCHEMA_SQL).execute(pool).await?;

    // Insert admin user
    sqlx::query(
        r#"
        INSERT INTO users (email, username, password_hash, display_name, role, status, email_verified_at)
        VALUES ($1, $2, $3, 'Administrator', 'administrator', 'active', NOW())
        ON CONFLICT (email) DO UPDATE SET
            username = EXCLUDED.username,
            password_hash = EXCLUDED.password_hash,
            updated_at = NOW()
        "#,
    )
    .bind(&req.admin_email)
    .bind(&req.admin_username)
    .bind(password_hash)
    .execute(pool)
    .await?;

    // Insert default theme (using new UUID-based schema)
    sqlx::query(
        r#"
        INSERT INTO themes (theme_id, name, version, description, author, is_active, is_installed, activated_at, installed_at)
        VALUES ('rustpress-default', 'RustPress Default', '1.0.0', 'Default RustPress theme', 'RustPress Team', true, true, NOW(), NOW())
        ON CONFLICT (site_id, theme_id) DO UPDATE SET
            is_active = true,
            activated_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .execute(pool)
    .await?;

    // Update settings with site title
    sqlx::query(
        r#"
        UPDATE settings SET value = $1 WHERE key = 'site_title'
        "#,
    )
    .bind(&req.site_title)
    .execute(pool)
    .await?;

    Ok(())
}

/// Run the setup wizard server
pub async fn run_setup_wizard(
    addr: SocketAddr,
    config_path: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    let state = SetupState::new(config_path);
    let setup_complete = state.setup_complete.clone();

    let router = create_setup_router(state);

    info!("Starting RustPress Setup Wizard on http://{}", addr);
    info!(
        "Open your browser and navigate to http://{} to complete setup",
        addr
    );

    let listener = TcpListener::bind(addr).await?;

    // Create a shutdown signal that triggers when setup is complete
    let setup_complete_clone = setup_complete.clone();
    let shutdown_signal = async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if *setup_complete_clone.read().await {
                break;
            }
        }
    };

    // Run the server with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    // Check if setup was completed
    let completed = *setup_complete.read().await;
    if completed {
        info!("Setup complete, shutting down setup wizard...");
    }

    Ok(completed)
}

/// Consolidated SQL schema from migration files (00001, 00023, 00024)
/// This schema matches the official RustPress migration files exactly
const SCHEMA_SQL: &str = r#"
-- ============================================
-- Migration 00001: Initial Schema
-- ============================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    avatar_url VARCHAR(500),
    role VARCHAR(50) NOT NULL DEFAULT 'subscriber',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE,
    email_verified_at TIMESTAMP WITH TIME ZONE,
    meta JSONB DEFAULT '{}'::jsonb
);

-- Posts table
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL UNIQUE,
    content TEXT,
    excerpt TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    post_type VARCHAR(50) NOT NULL DEFAULT 'post',
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    featured_image_id UUID,
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    meta JSONB DEFAULT '{}'::jsonb
);

-- Pages table (similar to posts but for static pages)
CREATE TABLE IF NOT EXISTS pages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL UNIQUE,
    content TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    parent_id UUID REFERENCES pages(id) ON DELETE SET NULL,
    template VARCHAR(100),
    menu_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    meta JSONB DEFAULT '{}'::jsonb
);

-- Categories table
CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES categories(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Post categories junction table
CREATE TABLE IF NOT EXISTS post_categories (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, category_id)
);

-- Post tags junction table
CREATE TABLE IF NOT EXISTS post_tags (
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

-- Media table
CREATE TABLE IF NOT EXISTS media (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filename VARCHAR(500) NOT NULL,
    original_filename VARCHAR(500) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    width INTEGER,
    height INTEGER,
    alt_text VARCHAR(500),
    caption TEXT,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    meta JSONB DEFAULT '{}'::jsonb
);

-- Comments table
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    author_id UUID REFERENCES users(id) ON DELETE SET NULL,
    author_name VARCHAR(255),
    author_email VARCHAR(255),
    author_url VARCHAR(500),
    content TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(255) NOT NULL UNIQUE,
    value TEXT,
    type VARCHAR(50) NOT NULL DEFAULT 'string',
    group_name VARCHAR(100) NOT NULL DEFAULT 'general',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Themes table (matches ThemeRow struct in repository.rs)
CREATE TABLE IF NOT EXISTS themes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID,
    theme_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version VARCHAR(50),
    author VARCHAR(255),
    author_url VARCHAR(500),
    license VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    is_installed BOOLEAN NOT NULL DEFAULT TRUE,
    parent_theme_id VARCHAR(100),
    screenshot_url VARCHAR(500),
    homepage_url VARCHAR(500),
    tags TEXT[],
    supports JSONB NOT NULL DEFAULT '{}'::jsonb,
    menu_locations JSONB NOT NULL DEFAULT '{}'::jsonb,
    widget_areas JSONB NOT NULL DEFAULT '[]'::jsonb,
    customizer_schema JSONB NOT NULL DEFAULT '{}'::jsonb,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    template_count INTEGER DEFAULT 0,
    activated_at TIMESTAMP WITH TIME ZONE,
    installed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(site_id, theme_id)
);

-- Menus table
CREATE TABLE IF NOT EXISTS menus (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    location VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Menu items table
CREATE TABLE IF NOT EXISTS menu_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    menu_id UUID NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES menu_items(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    url VARCHAR(500),
    target VARCHAR(50) DEFAULT '_self',
    object_type VARCHAR(50),
    object_id UUID,
    menu_order INTEGER NOT NULL DEFAULT 0,
    css_classes VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Widgets table
CREATE TABLE IF NOT EXISTS widgets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sidebar VARCHAR(100) NOT NULL,
    widget_type VARCHAR(100) NOT NULL,
    title VARCHAR(255),
    content TEXT,
    settings JSONB DEFAULT '{}'::jsonb,
    widget_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_posts_status ON posts(status);
CREATE INDEX IF NOT EXISTS idx_posts_author ON posts(author_id);
CREATE INDEX IF NOT EXISTS idx_posts_published_at ON posts(published_at);
CREATE INDEX IF NOT EXISTS idx_posts_slug ON posts(slug);
CREATE INDEX IF NOT EXISTS idx_pages_slug ON pages(slug);
CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_status ON comments(status);
CREATE INDEX IF NOT EXISTS idx_media_uploaded_by ON media(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);
CREATE INDEX IF NOT EXISTS idx_menu_items_menu ON menu_items(menu_id);

-- Insert default settings
INSERT INTO settings (key, value, type, group_name) VALUES
    ('site_title', 'RustPress', 'string', 'general'),
    ('site_tagline', 'A Modern CMS Built with Rust', 'string', 'general'),
    ('site_url', 'http://localhost:8080', 'string', 'general'),
    ('admin_email', 'admin@rustpress.local', 'string', 'general'),
    ('posts_per_page', '10', 'integer', 'reading'),
    ('date_format', 'F j, Y', 'string', 'general'),
    ('time_format', 'g:i a', 'string', 'general'),
    ('timezone', 'UTC', 'string', 'general'),
    ('comments_enabled', 'true', 'boolean', 'discussion'),
    ('comment_moderation', 'true', 'boolean', 'discussion')
ON CONFLICT (key) DO NOTHING;

-- Insert default category
INSERT INTO categories (name, slug, description) VALUES
    ('Uncategorized', 'uncategorized', 'Default category for posts')
ON CONFLICT (slug) DO NOTHING;

-- ============================================
-- Migration 00023: Password Reset Tokens
-- ============================================

-- Password reset tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for fast token lookups
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON password_reset_tokens(token_hash);

-- Index for finding unexpired tokens by user
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_expires ON password_reset_tokens(user_id, expires_at);

-- Email verification tokens table
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for fast token lookups
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_hash ON email_verification_tokens(token_hash);

-- Index for finding unexpired tokens by user
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_user ON email_verification_tokens(user_id, expires_at);

-- ============================================
-- Migration 00024: Add Missing Schema
-- ============================================

-- Add missing columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS locale VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS timezone VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create sessions table for user session management
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    user_agent TEXT,
    ip_address VARCHAR(45),
    last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);

-- Create options table (distinct from settings, used by OptionsRepository)
CREATE TABLE IF NOT EXISTS options (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    site_id UUID,
    option_name VARCHAR(255) NOT NULL,
    option_value JSONB,
    option_group VARCHAR(100) NOT NULL DEFAULT 'general',
    autoload BOOLEAN NOT NULL DEFAULT TRUE,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    value_type VARCHAR(50),
    validation JSONB,
    display_name VARCHAR(255),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(site_id, option_name)
);

CREATE INDEX IF NOT EXISTS idx_options_site_id ON options(site_id);
CREATE INDEX IF NOT EXISTS idx_options_option_name ON options(option_name);
CREATE INDEX IF NOT EXISTS idx_options_option_group ON options(option_group);
CREATE INDEX IF NOT EXISTS idx_options_autoload ON options(autoload);

-- Migrate existing settings to options table
INSERT INTO options (option_name, option_value, option_group, autoload, is_system)
SELECT
    key as option_name,
    CASE
        WHEN type = 'boolean' THEN to_jsonb(value = 'true')
        WHEN type = 'integer' THEN to_jsonb(value::integer)
        ELSE to_jsonb(value)
    END as option_value,
    group_name as option_group,
    TRUE as autoload,
    TRUE as is_system
FROM settings
ON CONFLICT (site_id, option_name) DO NOTHING;

-- Add missing columns to media table if needed
ALTER TABLE media ADD COLUMN IF NOT EXISTS folder_id UUID;
ALTER TABLE media ADD COLUMN IF NOT EXISTS title VARCHAR(500);
ALTER TABLE media ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE media ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create media_folders table
CREATE TABLE IF NOT EXISTS media_folders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    parent_id UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_media_folders_parent_id ON media_folders(parent_id);

-- Add missing columns to comments table
ALTER TABLE comments ADD COLUMN IF NOT EXISTS likes_count INTEGER DEFAULT 0;
ALTER TABLE comments ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP WITH TIME ZONE;

-- Create comment_likes table
CREATE TABLE IF NOT EXISTS comment_likes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    comment_id UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ip_address VARCHAR(45),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_comment_likes_comment_id ON comment_likes(comment_id);
"#;

/// The setup page HTML
const SETUP_PAGE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RustPress Setup Wizard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(-45deg, #1a1a2e, #16213e, #0f3460, #1a1a2e);
            background-size: 400% 400%;
            animation: gradientShift 15s ease infinite;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
            position: relative;
            overflow: hidden;
        }

        @keyframes gradientShift {
            0% { background-position: 0% 50%; }
            50% { background-position: 100% 50%; }
            100% { background-position: 0% 50%; }
        }

        .container {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 16px;
            box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
            max-width: 600px;
            width: 100%;
            padding: 40px;
            position: relative;
            z-index: 1;
            backdrop-filter: blur(10px);
        }

        .logo {
            text-align: center;
            margin-bottom: 30px;
        }

        .logo h1 {
            font-size: 2.5rem;
            background: linear-gradient(135deg, #6366f1, #06b6d4);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .logo p {
            color: #64748b;
            margin-top: 8px;
        }

        /* Main progress bar */
        .main-progress {
            width: 100%;
            height: 6px;
            background: #e5e7eb;
            border-radius: 3px;
            margin-bottom: 30px;
            overflow: hidden;
            display: none;
        }

        .main-progress.show {
            display: block;
        }

        .main-progress-bar {
            height: 100%;
            background: linear-gradient(90deg, #6366f1, #06b6d4);
            border-radius: 3px;
            transition: width 0.5s ease;
            position: relative;
            width: 0%;
        }

        .main-progress-bar::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(
                90deg,
                transparent,
                rgba(255, 255, 255, 0.4),
                transparent
            );
            animation: shimmer 2s infinite;
        }

        @keyframes shimmer {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(100%); }
        }

        .progress-message {
            text-align: center;
            color: #64748b;
            font-size: 0.9rem;
            margin-bottom: 20px;
            min-height: 20px;
            display: none;
        }

        .progress-message.show {
            display: block;
        }

        .step {
            display: none;
        }

        .step.active {
            display: block;
        }

        .step-header {
            margin-bottom: 24px;
        }

        .step-header h2 {
            color: #1e293b;
            font-size: 1.5rem;
            margin-bottom: 8px;
        }

        .step-header p {
            color: #64748b;
        }

        .form-group {
            margin-bottom: 20px;
        }

        .form-group label {
            display: block;
            color: #374151;
            font-weight: 500;
            margin-bottom: 6px;
        }

        .form-group input {
            width: 100%;
            padding: 12px 16px;
            border: 2px solid #e5e7eb;
            border-radius: 8px;
            font-size: 1rem;
            transition: border-color 0.2s, box-shadow 0.2s;
        }

        .form-group input:focus {
            outline: none;
            border-color: #6366f1;
            box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
        }

        .form-row {
            display: grid;
            grid-template-columns: 2fr 1fr;
            gap: 16px;
        }

        .btn {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            padding: 12px 24px;
            border: none;
            border-radius: 8px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }

        .btn-primary {
            background: linear-gradient(135deg, #6366f1, #4f46e5);
            color: white;
        }

        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 20px -10px rgba(99, 102, 241, 0.5);
        }

        .btn-secondary {
            background: #f1f5f9;
            color: #475569;
        }

        .btn-secondary:hover {
            background: #e2e8f0;
        }

        .btn:disabled {
            opacity: 0.6;
            cursor: not-allowed;
            transform: none;
        }

        .btn-group {
            display: flex;
            gap: 12px;
            margin-top: 24px;
        }

        .alert {
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 20px;
            display: none;
        }

        .alert.show {
            display: block;
        }

        .alert-success {
            background: #dcfce7;
            color: #166534;
            border: 1px solid #86efac;
        }

        .alert-error {
            background: #fee2e2;
            color: #991b1b;
            border: 1px solid #fca5a5;
        }

        .progress-steps {
            display: flex;
            justify-content: center;
            margin-bottom: 30px;
        }

        .progress-step {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background: #e5e7eb;
            color: #9ca3af;
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 600;
            position: relative;
            transition: all 0.3s ease;
        }

        .progress-step.active {
            background: linear-gradient(135deg, #6366f1, #4f46e5);
            color: white;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0%, 100% { box-shadow: 0 0 0 0 rgba(99, 102, 241, 0.4); }
            50% { box-shadow: 0 0 0 10px rgba(99, 102, 241, 0); }
        }

        .progress-step.done {
            background: #10b981;
            color: white;
        }

        .progress-step:not(:last-child)::after {
            content: '';
            position: absolute;
            left: 40px;
            top: 50%;
            width: 60px;
            height: 3px;
            background: #e5e7eb;
            transform: translateY(-50%);
            transition: background 0.3s ease;
        }

        .progress-step.done:not(:last-child)::after {
            background: #10b981;
        }

        .loading {
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 3px solid rgba(255,255,255,0.3);
            border-radius: 50%;
            border-top-color: white;
            animation: spin 1s linear infinite;
            margin-right: 8px;
        }

        @keyframes spin {
            to { transform: rotate(360deg); }
        }

        .success-icon {
            width: 80px;
            height: 80px;
            background: linear-gradient(135deg, #10b981, #059669);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 24px;
            animation: successPop 0.5s ease-out;
        }

        @keyframes successPop {
            0% { transform: scale(0); }
            50% { transform: scale(1.2); }
            100% { transform: scale(1); }
        }

        /* Error page styles */
        .error-icon {
            width: 100px;
            height: 100px;
            background: linear-gradient(135deg, #ef4444, #dc2626);
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 0 auto 24px;
            animation: errorPulse 2s ease-in-out infinite, errorShake 0.5s ease-in-out;
        }

        .error-icon svg {
            width: 50px;
            height: 50px;
            color: white;
        }

        @keyframes errorPulse {
            0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
            50% { box-shadow: 0 0 0 20px rgba(239, 68, 68, 0); }
        }

        @keyframes errorShake {
            0%, 100% { transform: translateX(0); }
            10%, 30%, 50%, 70%, 90% { transform: translateX(-5px); }
            20%, 40%, 60%, 80% { transform: translateX(5px); }
        }

        @keyframes slideIn {
            from { opacity: 0; transform: translateY(-20px); }
            to { opacity: 1; transform: translateY(0); }
        }

        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }

        .error-container {
            animation: slideIn 0.5s ease-out;
        }

        .error-title {
            color: #dc2626;
            font-size: 1.75rem;
            font-weight: 700;
            margin-bottom: 16px;
        }

        .error-message {
            background: linear-gradient(135deg, #fef2f2, #fee2e2);
            border: 2px solid #fecaca;
            border-radius: 12px;
            padding: 20px;
            margin: 20px 0;
            color: #991b1b;
            font-family: Monaco, Menlo, monospace;
            font-size: 0.9rem;
            line-height: 1.6;
            animation: fadeIn 0.5s ease-out 0.3s both;
            max-height: 200px;
            overflow-y: auto;
        }

        .error-hint {
            color: #64748b;
            font-size: 0.95rem;
            margin-top: 16px;
            animation: fadeIn 0.5s ease-out 0.5s both;
        }

        .error-hint strong {
            color: #475569;
        }

        .btn-danger {
            background: linear-gradient(135deg, #ef4444, #dc2626);
            color: white;
        }

        .btn-danger:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 20px -10px rgba(239, 68, 68, 0.5);
        }

        .success-icon svg {
            width: 40px;
            height: 40px;
            color: white;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">
            <h1>RustPress</h1>
            <p>Setup Wizard</p>
        </div>

        <!-- Main progress bar -->
        <div class="main-progress" id="main-progress">
            <div class="main-progress-bar" id="main-progress-bar"></div>
        </div>
        <div class="progress-message" id="progress-message"></div>

        <div class="progress-steps">
            <div class="progress-step active" id="step1-indicator">1</div>
            <div class="progress-step" id="step2-indicator">2</div>
            <div class="progress-step" id="step3-indicator">3</div>
        </div>

        <!-- Step 1: Database Connection -->
        <div class="step active" id="step1">
            <div class="step-header">
                <h2>Database Connection</h2>
                <p>Enter your PostgreSQL database credentials</p>
            </div>

            <div id="db-alert" class="alert"></div>

            <div class="form-row">
                <div class="form-group">
                    <label for="db-host">Host</label>
                    <input type="text" id="db-host" value="localhost" placeholder="localhost">
                </div>
                <div class="form-group">
                    <label for="db-port">Port</label>
                    <input type="number" id="db-port" value="5432" placeholder="5432">
                </div>
            </div>

            <div class="form-group">
                <label for="db-username">Username</label>
                <input type="text" id="db-username" placeholder="postgres">
            </div>

            <div class="form-group">
                <label for="db-password">Password</label>
                <input type="password" id="db-password" placeholder="Enter password">
            </div>

            <div class="form-group">
                <label for="db-name">Database Name</label>
                <input type="text" id="db-name" value="rustpress" placeholder="rustpress">
            </div>

            <div class="btn-group">
                <button class="btn btn-secondary" onclick="testConnection()">Test Connection</button>
                <button class="btn btn-primary" onclick="goToStep(2)" id="next-step1" disabled>Continue</button>
            </div>
        </div>

        <!-- Step 2: Admin Account -->
        <div class="step" id="step2">
            <div class="step-header">
                <h2>Admin Account</h2>
                <p>Create your administrator account</p>
            </div>

            <div class="form-group">
                <label for="admin-email">Email Address</label>
                <input type="email" id="admin-email" placeholder="admin@example.com">
            </div>

            <div class="form-group">
                <label for="admin-username">Username</label>
                <input type="text" id="admin-username" placeholder="admin">
            </div>

            <div class="form-group">
                <label for="admin-password">Password</label>
                <input type="password" id="admin-password" placeholder="Choose a strong password">
            </div>

            <div class="form-group">
                <label for="site-title">Site Title</label>
                <input type="text" id="site-title" value="My RustPress Site" placeholder="My RustPress Site">
            </div>

            <div class="btn-group">
                <button class="btn btn-secondary" onclick="goToStep(1)">Back</button>
                <button class="btn btn-primary" onclick="installRustPress()">Install RustPress</button>
            </div>
        </div>

        <!-- Step 3: Complete -->
        <div class="step" id="step3">
            <div class="step-header" style="text-align: center;">
                <div class="success-icon">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                </div>
                <h2>Installation Complete!</h2>
                <p style="margin-top: 16px;">RustPress has been successfully installed.</p>
                <p style="margin-top: 8px; color: #6366f1;">The server will restart automatically...</p>
            </div>

            <div class="btn-group" style="justify-content: center; margin-top: 32px;">
                <button class="btn btn-primary" onclick="window.location.reload()">Go to Dashboard</button>
            </div>
        </div>

        <!-- Step 4: Error -->
        <div class="step" id="step4">
            <div class="error-container" style="text-align: center;">
                <div class="error-icon">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </div>
                <h2 class="error-title">Installation Failed</h2>
                <div class="error-message" id="error-message"></div>
                <p class="error-hint"><strong>Tip:</strong> Make sure the database is empty before installation, or check your credentials.</p>
            </div>

            <div class="btn-group" style="justify-content: center; margin-top: 32px;">
                <button class="btn btn-secondary" onclick="goToStep(1)">Back to Setup</button>
                <button class="btn btn-danger" onclick="window.location.reload()">Try Again</button>
            </div>
        </div>
    </div>

    <script>
        let connectionVerified = false;

        // Progress bar functions
        function showProgress(percent, message) {
            const progressContainer = document.getElementById('main-progress');
            const progressBar = document.getElementById('main-progress-bar');
            const progressMessage = document.getElementById('progress-message');

            progressContainer.classList.add('show');
            progressMessage.classList.add('show');
            progressBar.style.width = percent + '%';
            progressMessage.textContent = message;
        }

        function hideProgress() {
            document.getElementById('main-progress').classList.remove('show');
            document.getElementById('progress-message').classList.remove('show');
        }

        function showAlert(elementId, message, type) {
            const alert = document.getElementById(elementId);
            alert.textContent = message;
            alert.className = 'alert show alert-' + type;
        }

        function hideAlert(elementId) {
            document.getElementById(elementId).className = 'alert';
        }

        function showError(message) {
            hideProgress();
            document.getElementById('error-message').textContent = message;
            document.querySelectorAll('.step').forEach(s => s.classList.remove('active'));
            document.getElementById('step4').classList.add('active');
            document.querySelectorAll('.progress-step').forEach(s => {
                s.classList.remove('active', 'done');
            });
        }

        function goToStep(step) {
            document.querySelectorAll('.step').forEach(s => s.classList.remove('active'));
            document.getElementById('step' + step).classList.add('active');

            document.querySelectorAll('.progress-step').forEach((s, i) => {
                s.classList.remove('active', 'done');
                if (i + 1 < step) s.classList.add('done');
                if (i + 1 === step) s.classList.add('active');
            });
        }

        async function testConnection() {
            const btn = event.target;
            btn.disabled = true;
            btn.innerHTML = '<span class="loading"></span>Testing...';
            hideAlert('db-alert');

            const data = {
                host: document.getElementById('db-host').value,
                port: parseInt(document.getElementById('db-port').value),
                username: document.getElementById('db-username').value,
                password: document.getElementById('db-password').value,
                database: document.getElementById('db-name').value
            };

            try {
                const response = await fetch('/api/setup/test-connection', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                const result = await response.json();

                if (result.success) {
                    showAlert('db-alert', result.message + (result.details ? ' - ' + result.details : ''), 'success');
                    document.getElementById('next-step1').disabled = false;
                    connectionVerified = true;
                } else {
                    showAlert('db-alert', result.message + (result.details ? ' - ' + result.details : ''), 'error');
                }
            } catch (error) {
                showAlert('db-alert', 'Connection test failed: ' + error.message, 'error');
            }

            btn.disabled = false;
            btn.textContent = 'Test Connection';
        }

        async function installRustPress() {
            const btn = event.target;
            btn.disabled = true;
            btn.innerHTML = '<span class="loading"></span>Installing...';

            // Show progress bar
            showProgress(10, 'Connecting to database...');

            const data = {
                host: document.getElementById('db-host').value,
                port: parseInt(document.getElementById('db-port').value),
                username: document.getElementById('db-username').value,
                password: document.getElementById('db-password').value,
                database: document.getElementById('db-name').value,
                admin_email: document.getElementById('admin-email').value,
                admin_username: document.getElementById('admin-username').value,
                admin_password: document.getElementById('admin-password').value,
                site_title: document.getElementById('site-title').value
            };

            try {
                // Simulate progress steps
                showProgress(20, 'Creating database tables...');
                await new Promise(r => setTimeout(r, 300));

                showProgress(40, 'Running migrations...');
                await new Promise(r => setTimeout(r, 200));

                const response = await fetch('/api/setup/install', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                showProgress(70, 'Creating admin account...');
                await new Promise(r => setTimeout(r, 200));

                const result = await response.json();

                if (result.success) {
                    showProgress(90, 'Saving configuration...');
                    await new Promise(r => setTimeout(r, 300));

                    showProgress(100, 'Installation complete!');
                    await new Promise(r => setTimeout(r, 500));

                    hideProgress();
                    goToStep(3);
                    // Reload after a short delay
                    setTimeout(() => window.location.reload(), 3000);
                } else {
                    showError(result.message);
                    btn.disabled = false;
                    btn.textContent = 'Install RustPress';
                }
            } catch (error) {
                showError(error.message);
                btn.disabled = false;
                btn.textContent = 'Install RustPress';
            }
        }
    </script>
</body>
</html>
"#;
