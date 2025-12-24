//! Plugin Database Migrations Support
//!
//! Handles database migrations for plugins.

use crate::manifest::MigrationsSection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

/// Migration manager for plugins
pub struct MigrationManager {
    /// Applied migrations per plugin
    applied: HashMap<String, Vec<AppliedMigration>>,
    /// Migration files per plugin
    migrations: HashMap<String, Vec<Migration>>,
}

/// Applied migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedMigration {
    pub plugin_id: String,
    pub version: String,
    pub checksum: String,
    pub applied_at: DateTime<Utc>,
    pub execution_time_ms: u64,
}

/// Migration definition
#[derive(Debug, Clone)]
pub struct Migration {
    pub plugin_id: String,
    pub version: String,
    pub description: Option<String>,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub checksum: String,
}

/// Migration result
#[derive(Debug)]
pub struct MigrationResult {
    pub plugin_id: String,
    pub applied: Vec<String>,
    pub failed: Vec<MigrationFailure>,
    pub skipped: Vec<String>,
}

/// Migration failure
#[derive(Debug)]
pub struct MigrationFailure {
    pub version: String,
    pub error: String,
}

/// Migration direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    Up,
    Down,
}

impl MigrationManager {
    pub fn new() -> Self {
        Self {
            applied: HashMap::new(),
            migrations: HashMap::new(),
        }
    }

    /// Load migrations for a plugin from manifest
    pub fn load_from_manifest(
        &mut self,
        plugin_id: &str,
        plugin_path: &Path,
        section: &MigrationsSection,
    ) -> Result<usize, MigrationError> {
        let migrations_dir = plugin_path.join(&section.directory);

        if !migrations_dir.exists() {
            debug!("No migrations directory for plugin: {}", plugin_id);
            return Ok(0);
        }

        let mut migrations = Vec::new();

        // Load from manifest-defined files
        for file_def in &section.files {
            let file_path = migrations_dir.join(&file_def.file);
            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path)
                    .map_err(|e| MigrationError::Io(e.to_string()))?;

                let checksum = blake3::hash(content.as_bytes()).to_hex().to_string();

                migrations.push(Migration {
                    plugin_id: plugin_id.to_string(),
                    version: file_def.version.clone(),
                    description: file_def.description.clone(),
                    up_sql: content,
                    down_sql: None,
                    checksum,
                });
            }
        }

        // If no manifest-defined files, scan directory
        if migrations.is_empty() {
            migrations = self.scan_migrations_directory(&migrations_dir, plugin_id)?;
        }

        // Sort by version
        migrations.sort_by(|a, b| {
            semver::Version::parse(&a.version)
                .unwrap_or_else(|_| semver::Version::new(0, 0, 0))
                .cmp(
                    &semver::Version::parse(&b.version)
                        .unwrap_or_else(|_| semver::Version::new(0, 0, 0)),
                )
        });

        let count = migrations.len();
        self.migrations.insert(plugin_id.to_string(), migrations);

        info!("Loaded {} migrations for plugin: {}", count, plugin_id);
        Ok(count)
    }

    /// Scan a directory for migration files
    fn scan_migrations_directory(
        &self,
        dir: &Path,
        plugin_id: &str,
    ) -> Result<Vec<Migration>, MigrationError> {
        let mut migrations = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| MigrationError::Io(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| MigrationError::Io(e.to_string()))?;
            let path = entry.path();

            if path.extension().map(|e| e == "sql").unwrap_or(false) {
                if let Some(migration) = self.parse_migration_file(&path, plugin_id)? {
                    migrations.push(migration);
                }
            }
        }

        Ok(migrations)
    }

    /// Parse a migration file
    fn parse_migration_file(
        &self,
        path: &Path,
        plugin_id: &str,
    ) -> Result<Option<Migration>, MigrationError> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| MigrationError::InvalidFile(path.to_path_buf()))?;

        // Expected format: V{version}__{description}.sql
        // e.g., V001__create_users_table.sql
        let parts: Vec<&str> = filename.splitn(2, "__").collect();
        if parts.len() < 2 {
            warn!("Invalid migration filename format: {}", filename);
            return Ok(None);
        }

        let version = parts[0].trim_start_matches('V').trim_start_matches('v');
        let description = parts[1].replace('_', " ");

        let content = std::fs::read_to_string(path)
            .map_err(|e| MigrationError::Io(e.to_string()))?;

        // Check for up/down sections
        let (up_sql, down_sql) = if content.contains("-- migrate:up") {
            self.parse_up_down_sections(&content)
        } else {
            (content, None)
        };

        let checksum = blake3::hash(up_sql.as_bytes()).to_hex().to_string();

        Ok(Some(Migration {
            plugin_id: plugin_id.to_string(),
            version: version.to_string(),
            description: Some(description),
            up_sql,
            down_sql,
            checksum,
        }))
    }

    /// Parse up/down sections from migration content
    fn parse_up_down_sections(&self, content: &str) -> (String, Option<String>) {
        let mut up_sql = String::new();
        let mut down_sql = String::new();
        let mut current_section = None;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("-- migrate:up") {
                current_section = Some("up");
            } else if trimmed.starts_with("-- migrate:down") {
                current_section = Some("down");
            } else {
                match current_section {
                    Some("up") => {
                        up_sql.push_str(line);
                        up_sql.push('\n');
                    }
                    Some("down") => {
                        down_sql.push_str(line);
                        down_sql.push('\n');
                    }
                    _ => {}
                }
            }
        }

        let down = if down_sql.is_empty() {
            None
        } else {
            Some(down_sql)
        };

        (up_sql, down)
    }

    /// Get pending migrations for a plugin
    pub fn get_pending(&self, plugin_id: &str) -> Vec<&Migration> {
        let applied = self.applied.get(plugin_id);
        let migrations = match self.migrations.get(plugin_id) {
            Some(m) => m,
            None => return Vec::new(),
        };

        migrations
            .iter()
            .filter(|m| {
                !applied
                    .map(|a| a.iter().any(|am| am.version == m.version))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get applied migrations for a plugin
    pub fn get_applied(&self, plugin_id: &str) -> Vec<&AppliedMigration> {
        self.applied
            .get(plugin_id)
            .map(|a| a.iter().collect())
            .unwrap_or_default()
    }

    /// Run pending migrations for a plugin
    pub async fn run_pending(
        &mut self,
        plugin_id: &str,
        executor: &dyn MigrationExecutor,
    ) -> MigrationResult {
        let pending: Vec<Migration> = self.get_pending(plugin_id).into_iter().cloned().collect();

        let mut result = MigrationResult {
            plugin_id: plugin_id.to_string(),
            applied: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        };

        for migration in pending {
            let start = std::time::Instant::now();

            match executor.execute(&migration.up_sql).await {
                Ok(()) => {
                    let elapsed = start.elapsed();
                    let applied = AppliedMigration {
                        plugin_id: plugin_id.to_string(),
                        version: migration.version.clone(),
                        checksum: migration.checksum.clone(),
                        applied_at: Utc::now(),
                        execution_time_ms: elapsed.as_millis() as u64,
                    };

                    // Record applied migration
                    self.applied
                        .entry(plugin_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(applied);

                    result.applied.push(migration.version.clone());
                    info!(
                        "Applied migration {} for plugin {} ({}ms)",
                        migration.version,
                        plugin_id,
                        elapsed.as_millis()
                    );
                }
                Err(e) => {
                    error!(
                        "Migration {} failed for plugin {}: {}",
                        migration.version, plugin_id, e
                    );
                    result.failed.push(MigrationFailure {
                        version: migration.version.clone(),
                        error: e,
                    });
                    // Stop on first failure
                    break;
                }
            }
        }

        result
    }

    /// Rollback last migration
    pub async fn rollback(
        &mut self,
        plugin_id: &str,
        executor: &dyn MigrationExecutor,
    ) -> Result<Option<String>, MigrationError> {
        let applied = self.applied.get(plugin_id);
        let last_applied = match applied.and_then(|a| a.last()) {
            Some(a) => a.clone(),
            None => return Ok(None),
        };

        // Find the migration
        let migration = self
            .migrations
            .get(plugin_id)
            .and_then(|m| m.iter().find(|m| m.version == last_applied.version))
            .ok_or_else(|| MigrationError::NotFound(last_applied.version.clone()))?;

        let down_sql = migration
            .down_sql
            .as_ref()
            .ok_or_else(|| MigrationError::NoRollback(last_applied.version.clone()))?;

        executor.execute(down_sql).await.map_err(|e| MigrationError::ExecutionFailed(e))?;

        // Remove from applied
        if let Some(applied) = self.applied.get_mut(plugin_id) {
            applied.retain(|a| a.version != last_applied.version);
        }

        info!(
            "Rolled back migration {} for plugin {}",
            last_applied.version, plugin_id
        );

        Ok(Some(last_applied.version))
    }

    /// Validate migration checksums
    pub fn validate_checksums(&self, plugin_id: &str) -> Vec<ChecksumMismatch> {
        let mut mismatches = Vec::new();

        let applied = match self.applied.get(plugin_id) {
            Some(a) => a,
            None => return mismatches,
        };

        let migrations = match self.migrations.get(plugin_id) {
            Some(m) => m,
            None => return mismatches,
        };

        for applied_migration in applied {
            if let Some(migration) = migrations.iter().find(|m| m.version == applied_migration.version) {
                if migration.checksum != applied_migration.checksum {
                    mismatches.push(ChecksumMismatch {
                        version: applied_migration.version.clone(),
                        expected: migration.checksum.clone(),
                        actual: applied_migration.checksum.clone(),
                    });
                }
            }
        }

        mismatches
    }

    /// Get migration status for a plugin
    pub fn get_status(&self, plugin_id: &str) -> MigrationStatus {
        let total = self
            .migrations
            .get(plugin_id)
            .map(|m| m.len())
            .unwrap_or(0);
        let applied = self.applied.get(plugin_id).map(|a| a.len()).unwrap_or(0);
        let pending = total.saturating_sub(applied);
        let last_applied = self
            .applied
            .get(plugin_id)
            .and_then(|a| a.last())
            .map(|a| a.version.clone());

        MigrationStatus {
            plugin_id: plugin_id.to_string(),
            total,
            applied,
            pending,
            last_applied,
        }
    }

    /// Record an applied migration (for loading from database)
    pub fn record_applied(&mut self, migration: AppliedMigration) {
        self.applied
            .entry(migration.plugin_id.clone())
            .or_insert_with(Vec::new)
            .push(migration);
    }

    /// Clear all migrations for a plugin
    pub fn clear(&mut self, plugin_id: &str) {
        self.migrations.remove(plugin_id);
        self.applied.remove(plugin_id);
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration executor trait
#[async_trait::async_trait]
pub trait MigrationExecutor: Send + Sync {
    /// Execute a migration SQL
    async fn execute(&self, sql: &str) -> Result<(), String>;

    /// Begin a transaction
    async fn begin(&self) -> Result<(), String>;

    /// Commit a transaction
    async fn commit(&self) -> Result<(), String>;

    /// Rollback a transaction
    async fn rollback(&self) -> Result<(), String>;
}

/// Checksum mismatch
#[derive(Debug)]
pub struct ChecksumMismatch {
    pub version: String,
    pub expected: String,
    pub actual: String,
}

/// Migration status
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub plugin_id: String,
    pub total: usize,
    pub applied: usize,
    pub pending: usize,
    pub last_applied: Option<String>,
}

/// Migration error
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Invalid migration file: {0}")]
    InvalidFile(PathBuf),

    #[error("Migration not found: {0}")]
    NotFound(String),

    #[error("No rollback available for migration: {0}")]
    NoRollback(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Checksum mismatch for migration: {0}")]
    ChecksumMismatch(String),
}

/// Migration builder for programmatic migration creation
pub struct MigrationBuilder {
    plugin_id: String,
    version: Option<String>,
    description: Option<String>,
    up_statements: Vec<String>,
    down_statements: Vec<String>,
}

impl MigrationBuilder {
    pub fn new(plugin_id: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            version: None,
            description: None,
            up_statements: Vec::new(),
            down_statements: Vec::new(),
        }
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn create_table(mut self, name: &str, columns: &str) -> Self {
        self.up_statements.push(format!(
            "CREATE TABLE IF NOT EXISTS {} ({});",
            name, columns
        ));
        self.down_statements.insert(0, format!("DROP TABLE IF EXISTS {};", name));
        self
    }

    pub fn add_column(mut self, table: &str, column: &str, definition: &str) -> Self {
        self.up_statements
            .push(format!("ALTER TABLE {} ADD COLUMN {} {};", table, column, definition));
        self.down_statements
            .insert(0, format!("ALTER TABLE {} DROP COLUMN {};", table, column));
        self
    }

    pub fn create_index(mut self, name: &str, table: &str, columns: &str) -> Self {
        self.up_statements.push(format!(
            "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
            name, table, columns
        ));
        self.down_statements.insert(0, format!("DROP INDEX IF EXISTS {};", name));
        self
    }

    pub fn raw_up(mut self, sql: &str) -> Self {
        self.up_statements.push(sql.to_string());
        self
    }

    pub fn raw_down(mut self, sql: &str) -> Self {
        self.down_statements.insert(0, sql.to_string());
        self
    }

    pub fn build(self) -> Migration {
        let up_sql = self.up_statements.join("\n");
        let down_sql = if self.down_statements.is_empty() {
            None
        } else {
            Some(self.down_statements.join("\n"))
        };

        let checksum = blake3::hash(up_sql.as_bytes()).to_hex().to_string();

        Migration {
            plugin_id: self.plugin_id,
            version: self.version.unwrap_or_else(|| "0.0.0".to_string()),
            description: self.description,
            up_sql,
            down_sql,
            checksum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_builder() {
        let migration = MigrationBuilder::new("test-plugin")
            .version("1.0.0")
            .description("Create users table")
            .create_table("users", "id INTEGER PRIMARY KEY, name TEXT NOT NULL")
            .create_index("idx_users_name", "users", "name")
            .build();

        assert_eq!(migration.version, "1.0.0");
        assert!(migration.up_sql.contains("CREATE TABLE"));
        assert!(migration.up_sql.contains("CREATE INDEX"));
        assert!(migration.down_sql.is_some());
        assert!(migration.down_sql.unwrap().contains("DROP TABLE"));
    }

    #[test]
    fn test_migration_manager() {
        let manager = MigrationManager::new();
        let status = manager.get_status("nonexistent");

        assert_eq!(status.total, 0);
        assert_eq!(status.applied, 0);
        assert_eq!(status.pending, 0);
    }
}
