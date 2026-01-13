//! Database migration system.

use rustpress_core::error::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use std::path::Path;

/// Migration entry
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub sql: String,
}

impl Migration {
    pub fn new(version: i64, name: impl Into<String>, sql: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            sql: sql.into(),
        }
    }
}

/// Database migrator
pub struct Migrator {
    migrations: Vec<Migration>,
}

impl Migrator {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Add a migration
    pub fn add(&mut self, migration: Migration) -> &mut Self {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
        self
    }

    /// Load migrations from embedded SQL
    pub fn with_migrations(mut self, migrations: Vec<Migration>) -> Self {
        for m in migrations {
            self.add(m);
        }
        self
    }

    /// Run all pending migrations
    pub async fn run(&self, pool: &PgPool) -> Result<Vec<i64>> {
        // Ensure migrations table exists
        self.ensure_migrations_table(pool).await?;

        // Get applied migrations
        let applied = self.get_applied_migrations(pool).await?;

        let mut newly_applied = Vec::new();

        for migration in &self.migrations {
            if !applied.contains(&migration.version) {
                self.apply_migration(pool, migration).await?;
                newly_applied.push(migration.version);
            }
        }

        if newly_applied.is_empty() {
            tracing::info!("No pending migrations");
        } else {
            tracing::info!(count = newly_applied.len(), "Applied migrations");
        }

        Ok(newly_applied)
    }

    /// Rollback the last migration
    pub async fn rollback(&self, pool: &PgPool) -> Result<Option<i64>> {
        let applied = self.get_applied_migrations(pool).await?;

        if let Some(&last_version) = applied.last() {
            // Find the migration
            let migration = self
                .migrations
                .iter()
                .find(|m| m.version == last_version)
                .ok_or_else(|| Error::Migration {
                    message: format!("Migration {} not found", last_version),
                })?;

            // Remove from applied
            sqlx::query("DELETE FROM _migrations WHERE version = $1")
                .bind(last_version)
                .execute(pool)
                .await
                .map_err(|e| Error::database_with_source("Failed to remove migration record", e))?;

            tracing::info!(version = last_version, name = %migration.name, "Rolled back migration");
            Ok(Some(last_version))
        } else {
            Ok(None)
        }
    }

    /// Get migration status
    pub async fn status(&self, pool: &PgPool) -> Result<Vec<MigrationStatus>> {
        self.ensure_migrations_table(pool).await?;
        let applied = self.get_applied_migrations(pool).await?;

        let statuses = self
            .migrations
            .iter()
            .map(|m| MigrationStatus {
                version: m.version,
                name: m.name.clone(),
                applied: applied.contains(&m.version),
            })
            .collect();

        Ok(statuses)
    }

    async fn ensure_migrations_table(&self, pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| Error::database_with_source("Failed to create migrations table", e))?;

        Ok(())
    }

    async fn get_applied_migrations(&self, pool: &PgPool) -> Result<Vec<i64>> {
        let rows: Vec<(i64,)> = sqlx::query_as("SELECT version FROM _migrations ORDER BY version")
            .fetch_all(pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to get applied migrations", e))?;

        Ok(rows.into_iter().map(|(v,)| v).collect())
    }

    async fn apply_migration(&self, pool: &PgPool, migration: &Migration) -> Result<()> {
        tracing::info!(
            version = migration.version,
            name = %migration.name,
            "Applying migration"
        );

        // Execute the migration SQL
        sqlx::query(&migration.sql)
            .execute(pool)
            .await
            .map_err(|e| {
                Error::database_with_source(
                    format!("Migration {} failed", migration.version),
                    e,
                )
            })?;

        // Record the migration
        sqlx::query("INSERT INTO _migrations (version, name) VALUES ($1, $2)")
            .bind(migration.version)
            .bind(&migration.name)
            .execute(pool)
            .await
            .map_err(|e| Error::database_with_source("Failed to record migration", e))?;

        Ok(())
    }
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration status
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub version: i64,
    pub name: String,
    pub applied: bool,
}

/// Create initial migrations for RustPress
pub fn create_initial_migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            1,
            "create_users_table",
            r#"
            CREATE TABLE users (
                id UUID PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                username VARCHAR(100) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                display_name VARCHAR(255),
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                role VARCHAR(50) NOT NULL DEFAULT 'subscriber',
                avatar_url TEXT,
                locale VARCHAR(10) DEFAULT 'en',
                timezone VARCHAR(50) DEFAULT 'UTC',
                email_verified_at TIMESTAMPTZ,
                last_login_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ
            );

            CREATE INDEX idx_users_email ON users(email);
            CREATE INDEX idx_users_username ON users(username);
            CREATE INDEX idx_users_status ON users(status);
            "#,
        ),
        Migration::new(
            2,
            "create_posts_table",
            r#"
            CREATE TABLE posts (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                author_id UUID NOT NULL REFERENCES users(id),
                title VARCHAR(500) NOT NULL,
                slug VARCHAR(500) NOT NULL,
                excerpt TEXT,
                content TEXT,
                content_format VARCHAR(50) DEFAULT 'html',
                status VARCHAR(50) NOT NULL DEFAULT 'draft',
                visibility VARCHAR(50) DEFAULT 'public',
                password VARCHAR(255),
                featured_image_id UUID,
                comment_status VARCHAR(50) DEFAULT 'open',
                ping_status VARCHAR(50) DEFAULT 'open',
                published_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ,

                CONSTRAINT unique_slug_per_tenant UNIQUE (tenant_id, slug)
            );

            CREATE INDEX idx_posts_author ON posts(author_id);
            CREATE INDEX idx_posts_status ON posts(status);
            CREATE INDEX idx_posts_slug ON posts(slug);
            CREATE INDEX idx_posts_published ON posts(published_at);
            CREATE INDEX idx_posts_tenant ON posts(tenant_id);
            "#,
        ),
        Migration::new(
            3,
            "create_pages_table",
            r#"
            CREATE TABLE pages (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                author_id UUID NOT NULL REFERENCES users(id),
                parent_id UUID REFERENCES pages(id),
                title VARCHAR(500) NOT NULL,
                slug VARCHAR(500) NOT NULL,
                content TEXT,
                content_format VARCHAR(50) DEFAULT 'html',
                status VARCHAR(50) NOT NULL DEFAULT 'draft',
                visibility VARCHAR(50) DEFAULT 'public',
                password VARCHAR(255),
                template VARCHAR(255),
                menu_order INT DEFAULT 0,
                featured_image_id UUID,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ,

                CONSTRAINT unique_page_slug_per_tenant UNIQUE (tenant_id, slug)
            );

            CREATE INDEX idx_pages_author ON pages(author_id);
            CREATE INDEX idx_pages_parent ON pages(parent_id);
            CREATE INDEX idx_pages_slug ON pages(slug);
            "#,
        ),
        Migration::new(
            4,
            "create_comments_table",
            r#"
            CREATE TABLE comments (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
                parent_id UUID REFERENCES comments(id),
                author_id UUID REFERENCES users(id),
                author_name VARCHAR(255),
                author_email VARCHAR(255),
                author_url TEXT,
                author_ip VARCHAR(45),
                content TEXT NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                user_agent TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_comments_post ON comments(post_id);
            CREATE INDEX idx_comments_parent ON comments(parent_id);
            CREATE INDEX idx_comments_status ON comments(status);
            "#,
        ),
        Migration::new(
            5,
            "create_media_table",
            r#"
            CREATE TABLE media (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                uploader_id UUID REFERENCES users(id),
                filename VARCHAR(255) NOT NULL,
                original_filename VARCHAR(255) NOT NULL,
                mime_type VARCHAR(100) NOT NULL,
                file_size BIGINT NOT NULL,
                storage_path TEXT NOT NULL,
                storage_backend VARCHAR(50) DEFAULT 'local',
                alt_text TEXT,
                title VARCHAR(500),
                description TEXT,
                width INT,
                height INT,
                duration INT,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ
            );

            CREATE INDEX idx_media_uploader ON media(uploader_id);
            CREATE INDEX idx_media_mime ON media(mime_type);
            CREATE INDEX idx_media_tenant ON media(tenant_id);
            "#,
        ),
        Migration::new(
            6,
            "create_taxonomies_table",
            r#"
            CREATE TABLE taxonomies (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                name VARCHAR(100) NOT NULL,
                slug VARCHAR(100) NOT NULL,
                description TEXT,
                hierarchical BOOLEAN DEFAULT FALSE,

                CONSTRAINT unique_taxonomy_slug_per_tenant UNIQUE (tenant_id, slug)
            );

            CREATE TABLE terms (
                id UUID PRIMARY KEY,
                taxonomy_id UUID NOT NULL REFERENCES taxonomies(id) ON DELETE CASCADE,
                parent_id UUID REFERENCES terms(id),
                name VARCHAR(255) NOT NULL,
                slug VARCHAR(255) NOT NULL,
                description TEXT,
                term_order INT DEFAULT 0,
                count INT DEFAULT 0,

                CONSTRAINT unique_term_slug_per_taxonomy UNIQUE (taxonomy_id, slug)
            );

            CREATE TABLE term_relationships (
                object_id UUID NOT NULL,
                object_type VARCHAR(50) NOT NULL,
                term_id UUID NOT NULL REFERENCES terms(id) ON DELETE CASCADE,
                term_order INT DEFAULT 0,
                PRIMARY KEY (object_id, term_id)
            );

            CREATE INDEX idx_terms_taxonomy ON terms(taxonomy_id);
            CREATE INDEX idx_term_rel_object ON term_relationships(object_id, object_type);
            "#,
        ),
        Migration::new(
            7,
            "create_options_table",
            r#"
            CREATE TABLE options (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                name VARCHAR(255) NOT NULL,
                value JSONB,
                autoload BOOLEAN DEFAULT TRUE,

                CONSTRAINT unique_option_per_tenant UNIQUE (tenant_id, name)
            );

            CREATE INDEX idx_options_autoload ON options(autoload) WHERE autoload = TRUE;
            "#,
        ),
        Migration::new(
            8,
            "create_sessions_table",
            r#"
            CREATE TABLE sessions (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash VARCHAR(255) NOT NULL UNIQUE,
                ip_address VARCHAR(45),
                user_agent TEXT,
                last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_sessions_user ON sessions(user_id);
            CREATE INDEX idx_sessions_expires ON sessions(expires_at);
            "#,
        ),
        Migration::new(
            9,
            "create_tenants_table",
            r#"
            CREATE TABLE tenants (
                id UUID PRIMARY KEY,
                slug VARCHAR(100) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                domain VARCHAR(255) UNIQUE,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                plan VARCHAR(50) DEFAULT 'free',
                owner_id UUID REFERENCES users(id),
                settings JSONB DEFAULT '{}',
                quotas JSONB DEFAULT '{}',
                trial_ends_at TIMESTAMPTZ,
                subscription_ends_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_tenants_slug ON tenants(slug);
            CREATE INDEX idx_tenants_domain ON tenants(domain);
            CREATE INDEX idx_tenants_owner ON tenants(owner_id);
            "#,
        ),
        Migration::new(
            10,
            "create_jobs_table",
            r#"
            CREATE TABLE jobs (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                queue VARCHAR(100) NOT NULL DEFAULT 'default',
                job_type VARCHAR(255) NOT NULL,
                payload JSONB NOT NULL,
                status VARCHAR(50) NOT NULL DEFAULT 'pending',
                priority INT DEFAULT 0,
                attempts INT DEFAULT 0,
                max_attempts INT DEFAULT 3,
                last_error TEXT,
                available_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                reserved_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_jobs_queue_status ON jobs(queue, status);
            CREATE INDEX idx_jobs_available ON jobs(available_at) WHERE status = 'pending';
            "#,
        ),
        Migration::new(
            11,
            "create_webhooks_table",
            r#"
            CREATE TABLE webhooks (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                name VARCHAR(255) NOT NULL,
                url TEXT NOT NULL,
                secret VARCHAR(255),
                events TEXT[] NOT NULL,
                is_active BOOLEAN DEFAULT TRUE,
                last_triggered_at TIMESTAMPTZ,
                failure_count INT DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE webhook_deliveries (
                id UUID PRIMARY KEY,
                webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
                event_type VARCHAR(100) NOT NULL,
                payload JSONB NOT NULL,
                response_status INT,
                response_body TEXT,
                duration_ms INT,
                success BOOLEAN,
                error_message TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_webhooks_tenant ON webhooks(tenant_id);
            CREATE INDEX idx_webhook_deliveries_webhook ON webhook_deliveries(webhook_id);
            "#,
        ),
        Migration::new(
            12,
            "create_audit_log_table",
            r#"
            CREATE TABLE audit_logs (
                id UUID PRIMARY KEY,
                tenant_id UUID,
                user_id UUID,
                action VARCHAR(100) NOT NULL,
                entity_type VARCHAR(100) NOT NULL,
                entity_id UUID,
                old_values JSONB,
                new_values JSONB,
                ip_address VARCHAR(45),
                user_agent TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX idx_audit_user ON audit_logs(user_id);
            CREATE INDEX idx_audit_entity ON audit_logs(entity_type, entity_id);
            CREATE INDEX idx_audit_created ON audit_logs(created_at);
            "#,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_ordering() {
        let migrations = create_initial_migrations();
        for i in 1..migrations.len() {
            assert!(migrations[i].version > migrations[i - 1].version);
        }
    }

    #[test]
    fn test_migrator_add() {
        let mut migrator = Migrator::new();
        migrator.add(Migration::new(2, "second", "SELECT 2"));
        migrator.add(Migration::new(1, "first", "SELECT 1"));

        assert_eq!(migrator.migrations[0].version, 1);
        assert_eq!(migrator.migrations[1].version, 2);
    }
}
