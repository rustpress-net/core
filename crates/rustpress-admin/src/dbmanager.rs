//! Database Manager Module - phpMyAdmin-like functionality for RustPress
//!
//! Provides comprehensive database management capabilities including:
//! - Database status and statistics
//! - Table browsing and management
//! - SQL query execution
//! - Data import/export
//! - Query history and saved queries

use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, Column, TypeInfo};
use std::collections::HashMap;
use std::sync::Arc;

use crate::handlers::ApiResponse;

// ============================================================================
// Types
// ============================================================================

/// Database connection status
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    #[serde(rename = "type")]
    pub db_type: String,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub ssl: bool,
    pub uptime: String,
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

/// Database statistics
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStats {
    pub total_tables: u32,
    pub total_rows: u64,
    pub database_size: String,
    pub index_size: String,
    pub cache_hit_ratio: f64,
    pub avg_query_time: f64,
    pub queries_per_second: f64,
    pub slow_queries: u32,
}

/// Table information
#[derive(Debug, Clone, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
    pub rows: u64,
    pub size: String,
    pub index_size: String,
    pub has_relations: bool,
    pub last_modified: String,
    #[serde(rename = "type")]
    pub table_type: String,
}

/// Column information
#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub references: Option<ForeignKeyReference>,
}

/// Foreign key reference
#[derive(Debug, Clone, Serialize)]
pub struct ForeignKeyReference {
    pub table: String,
    pub column: String,
}

/// Index information
#[derive(Debug, Clone, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    #[serde(rename = "type")]
    pub index_type: String,
}

/// Foreign key information
#[derive(Debug, Clone, Serialize)]
pub struct ForeignKeyInfo {
    pub name: String,
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: String,
    pub on_delete: String,
    pub on_update: String,
}

/// Query execution result
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: u64,
    pub execution_time: f64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_rows: Option<u64>,
}

/// Saved query
#[derive(Debug, Clone, Serialize)]
pub struct SavedQuery {
    pub id: String,
    pub name: String,
    pub query: String,
    pub created_at: String,
}

/// Query history item
#[derive(Debug, Clone, Serialize)]
pub struct QueryHistoryItem {
    pub id: String,
    pub query: String,
    pub execution_time: f64,
    pub status: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<u64>,
}

/// Query explain plan
#[derive(Debug, Clone, Serialize)]
pub struct QueryExplain {
    pub plan: String,
    pub execution_time: f64,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// List tables query params
#[derive(Debug, Deserialize)]
pub struct ListTablesQuery {
    #[serde(rename = "showSystem")]
    pub show_system: Option<bool>,
}

/// Table data query params
#[derive(Debug, Deserialize)]
pub struct TableDataQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    #[serde(rename = "sortColumn")]
    pub sort_column: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<String>,
    /// Column to filter on
    #[serde(rename = "filterColumn")]
    pub filter_column: Option<String>,
    /// Value to filter by
    #[serde(rename = "filterValue")]
    pub filter_value: Option<String>,
    /// Search across all text columns
    pub search: Option<String>,
}

/// Execute query request
#[derive(Debug, Deserialize)]
pub struct ExecuteQueryRequest {
    pub query: String,
    /// Confirmation flag required for destructive operations (DROP, TRUNCATE, DELETE)
    #[serde(default)]
    pub confirm: bool,
}

/// Blocked SQL commands that cannot be executed through the UI
pub const BLOCKED_COMMANDS: &[&str] = &[
    "DROP DATABASE",
    "DROP SCHEMA",
    "CREATE DATABASE",
    "ALTER SYSTEM",
    "COPY",  // Prevent file system access
    "pg_read_file",
    "pg_write_file",
    "lo_import",
    "lo_export",
];

/// Destructive commands that require confirmation
pub const DESTRUCTIVE_COMMANDS: &[&str] = &[
    "DROP TABLE",
    "DROP INDEX",
    "DROP VIEW",
    "DROP FUNCTION",
    "DROP TRIGGER",
    "DROP SEQUENCE",
    "TRUNCATE",
    "DELETE FROM",
    "DELETE",
];

/// Check if a query contains blocked commands
fn is_blocked_query(query: &str) -> Option<&'static str> {
    let query_upper = query.to_uppercase();
    BLOCKED_COMMANDS.iter().find(|&blocked| query_upper.contains(blocked)).copied()
}

/// Check if a query is destructive and requires confirmation
fn is_destructive_query(query: &str) -> bool {
    let query_upper = query.to_uppercase();
    DESTRUCTIVE_COMMANDS.iter().any(|cmd| query_upper.contains(cmd))
}

/// Save query request
#[derive(Debug, Deserialize)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query: String,
}

/// Query history params
#[derive(Debug, Deserialize)]
pub struct QueryHistoryParams {
    pub limit: Option<u32>,
}

/// Create table request
#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    pub name: String,
    pub columns: Vec<CreateColumnRequest>,
}

/// Column definition for create table
#[derive(Debug, Deserialize)]
pub struct CreateColumnRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

/// Alter table request
#[derive(Debug, Deserialize)]
pub struct AlterTableRequest {
    pub add_columns: Option<Vec<CreateColumnRequest>>,
    pub drop_columns: Option<Vec<String>>,
    pub modify_columns: Option<Vec<CreateColumnRequest>>,
    pub add_indexes: Option<Vec<CreateIndexRequest>>,
    pub drop_indexes: Option<Vec<String>>,
}

/// Create index request
#[derive(Debug, Deserialize)]
pub struct CreateIndexRequest {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

/// Insert row request
#[derive(Debug, Deserialize)]
pub struct InsertRowRequest {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

/// Update row request
#[derive(Debug, Deserialize)]
pub struct UpdateRowRequest {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

/// Bulk delete request
#[derive(Debug, Deserialize)]
pub struct BulkDeleteRequest {
    pub ids: Vec<String>,
}

/// Export tables request
#[derive(Debug, Deserialize)]
pub struct ExportTablesRequest {
    pub tables: Vec<String>,
    pub format: String,
    #[serde(rename = "includeStructure")]
    pub include_structure: Option<bool>,
    #[serde(rename = "includeData")]
    pub include_data: Option<bool>,
}

/// Export query request
#[derive(Debug, Deserialize)]
pub struct ExportQueryRequest {
    pub query: String,
    pub format: String,
}

/// Tables response
#[derive(Debug, Serialize)]
pub struct TablesResponse {
    pub tables: Vec<TableInfo>,
}

/// Table detail response
#[derive(Debug, Serialize)]
pub struct TableDetailResponse {
    pub table: TableInfo,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

/// Columns response
#[derive(Debug, Serialize)]
pub struct ColumnsResponse {
    pub columns: Vec<ColumnInfo>,
}

/// Indexes response
#[derive(Debug, Serialize)]
pub struct IndexesResponse {
    pub indexes: Vec<IndexInfo>,
}

/// Foreign keys response
#[derive(Debug, Serialize)]
pub struct ForeignKeysResponse {
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

/// Table data response
#[derive(Debug, Serialize)]
pub struct TableDataResponse {
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    /// Primary key column(s) for this table
    pub primary_key_columns: Vec<String>,
}

/// Saved queries response
#[derive(Debug, Serialize)]
pub struct SavedQueriesResponse {
    pub queries: Vec<SavedQuery>,
}

/// Query history response
#[derive(Debug, Serialize)]
pub struct QueryHistoryResponse {
    pub history: Vec<QueryHistoryItem>,
}

/// Import SQL result
#[derive(Debug, Serialize)]
pub struct ImportSqlResult {
    pub success: bool,
    pub total_statements: u32,
    pub successful_statements: u32,
    pub failed_statements: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

/// Import CSV result
#[derive(Debug, Serialize)]
pub struct ImportCsvResult {
    pub success: bool,
    pub rows_imported: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub event_type: String,
    pub event_data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    pub created_at: String,
}

/// Audit log query params
#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
}

/// Audit log response
#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limit configuration
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,  // 100 requests per minute
            window_seconds: 60,
        }
    }
}

/// Rate limiter using sliding window algorithm
pub struct RateLimiter {
    config: RateLimitConfig,
    requests: std::sync::RwLock<HashMap<String, Vec<std::time::Instant>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            requests: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Check if request should be rate limited
    /// Returns Ok(remaining) if allowed, Err(retry_after_secs) if limited
    pub fn check(&self, key: &str) -> Result<u32, u64> {
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(self.config.window_seconds);

        let mut requests = self.requests.write().unwrap();
        let entry = requests.entry(key.to_string()).or_default();

        // Remove expired entries
        entry.retain(|t| now.duration_since(*t) < window);

        if entry.len() >= self.config.max_requests as usize {
            // Calculate when the oldest request will expire
            if let Some(oldest) = entry.first() {
                let retry_after = window.saturating_sub(now.duration_since(*oldest));
                return Err(retry_after.as_secs());
            }
            return Err(self.config.window_seconds);
        }

        // Add this request
        entry.push(now);
        let remaining = self.config.max_requests.saturating_sub(entry.len() as u32);
        Ok(remaining)
    }

    /// Cleanup old entries periodically
    pub fn cleanup(&self) {
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(self.config.window_seconds);

        let mut requests = self.requests.write().unwrap();
        requests.retain(|_, v| {
            v.retain(|t| now.duration_since(*t) < window);
            !v.is_empty()
        });
    }
}

// ============================================================================
// Database Manager State
// ============================================================================

/// Database Manager state
pub struct DbManagerState {
    /// Database connection pool
    pub pool: PgPool,
    /// In-memory cache for query history (also persisted to DB)
    pub query_history_cache: std::sync::RwLock<Vec<QueryHistoryItem>>,
    /// Rate limiter for query execution
    pub rate_limiter: RateLimiter,
}

impl DbManagerState {
    /// Create new DbManagerState and initialize database tables
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            query_history_cache: std::sync::RwLock::new(Vec::new()),
            rate_limiter: RateLimiter::new(RateLimitConfig::default()),
        }
    }

    /// Create with custom rate limit config
    pub fn with_rate_limit(pool: PgPool, rate_config: RateLimitConfig) -> Self {
        Self {
            pool,
            query_history_cache: std::sync::RwLock::new(Vec::new()),
            rate_limiter: RateLimiter::new(rate_config),
        }
    }

    /// Initialize database tables for persistence (call this on startup)
    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // Create saved queries table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS dbmanager_saved_queries (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name VARCHAR(255) NOT NULL,
                query TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create query history table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS dbmanager_query_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                query TEXT NOT NULL,
                execution_time_ms DOUBLE PRECISION NOT NULL,
                status VARCHAR(50) NOT NULL,
                row_count BIGINT,
                error_message TEXT,
                executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create index for faster history retrieval
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_dbmanager_query_history_executed_at
            ON dbmanager_query_history(executed_at DESC)
        "#)
        .execute(&self.pool)
        .await?;

        // Create audit log table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS dbmanager_audit_log (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                event_type VARCHAR(100) NOT NULL,
                event_data JSONB NOT NULL,
                user_id VARCHAR(255),
                ip_address VARCHAR(45),
                user_agent TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await?;

        // Create indexes for audit log
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_dbmanager_audit_log_created_at
            ON dbmanager_audit_log(created_at DESC)
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_dbmanager_audit_log_event_type
            ON dbmanager_audit_log(event_type)
        "#)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_dbmanager_audit_log_user_id
            ON dbmanager_audit_log(user_id)
        "#)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ============================================================================
// CSV Parser
// ============================================================================

/// Represents a parsed CSV row
#[derive(Debug)]
struct CsvRow {
    values: Vec<String>,
}

/// CSV Parser that handles quoted values, escaping, and multiline fields
struct CsvParser {
    delimiter: char,
    quote_char: char,
}

impl CsvParser {
    fn new() -> Self {
        Self {
            delimiter: ',',
            quote_char: '"',
        }
    }

    #[allow(dead_code)]
    fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Parse CSV content into headers and rows
    fn parse(&self, content: &str) -> Result<(Vec<String>, Vec<CsvRow>), String> {
        let mut rows: Vec<CsvRow> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            if in_quotes {
                if c == self.quote_char {
                    // Check for escaped quote (doubled quote)
                    if chars.peek() == Some(&self.quote_char) {
                        chars.next(); // consume the second quote
                        current_field.push(self.quote_char);
                    } else {
                        // End of quoted field
                        in_quotes = false;
                    }
                } else {
                    current_field.push(c);
                }
            } else {
                match c {
                    c if c == self.quote_char => {
                        in_quotes = true;
                    }
                    c if c == self.delimiter => {
                        current_row.push(current_field.trim().to_string());
                        current_field = String::new();
                    }
                    '\r' => {
                        // Handle CRLF - skip \r
                        continue;
                    }
                    '\n' => {
                        // End of row
                        current_row.push(current_field.trim().to_string());
                        if !current_row.iter().all(|s| s.is_empty()) {
                            rows.push(CsvRow { values: current_row });
                        }
                        current_row = Vec::new();
                        current_field = String::new();
                    }
                    _ => {
                        current_field.push(c);
                    }
                }
            }
        }

        // Handle last field and row if not empty
        if !current_field.is_empty() || !current_row.is_empty() {
            current_row.push(current_field.trim().to_string());
            if !current_row.iter().all(|s| s.is_empty()) {
                rows.push(CsvRow { values: current_row });
            }
        }

        if rows.is_empty() {
            return Err("CSV file is empty or contains no valid data".to_string());
        }

        // First row is headers
        let headers = rows.remove(0).values;

        // Validate that all rows have the same number of columns as headers
        for (i, row) in rows.iter().enumerate() {
            if row.values.len() != headers.len() {
                return Err(format!(
                    "Row {} has {} columns but header has {} columns",
                    i + 2, // +2 because row 1 is header and array is 0-indexed
                    row.values.len(),
                    headers.len()
                ));
            }
        }

        Ok((headers, rows))
    }

    /// Parse a single value, handling NULL and empty strings
    fn parse_value(&self, value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

// ============================================================================
// Hook System
// ============================================================================

/// Events that can trigger hooks
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DbManagerEvent {
    /// Query is about to be executed
    BeforeQuery {
        query: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Query has been executed
    AfterQuery {
        query: String,
        success: bool,
        execution_time_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        row_count: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Table is about to be modified (INSERT, UPDATE, DELETE)
    BeforeTableModify {
        table_name: String,
        operation: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Table has been modified
    AfterTableModify {
        table_name: String,
        operation: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        affected_rows: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Data import started
    BeforeImport {
        import_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        table_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Data import completed
    AfterImport {
        import_type: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        table_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rows_imported: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Data export started
    BeforeExport {
        export_format: String,
        tables: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Data export completed
    AfterExport {
        export_format: String,
        tables: Vec<String>,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
    /// Table structure changed (CREATE, ALTER, DROP)
    SchemaChange {
        operation: String,
        table_name: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
    },
}

/// Hook handler function type
pub type HookHandler = Arc<dyn Fn(DbManagerEvent) + Send + Sync>;

/// Hook registry for managing event handlers
pub struct HookRegistry {
    handlers: std::sync::RwLock<Vec<HookHandler>>,
}

impl HookRegistry {
    /// Create a new empty hook registry
    pub fn new() -> Self {
        Self {
            handlers: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Register a new hook handler
    pub fn register(&self, handler: HookHandler) {
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.push(handler);
        }
    }

    /// Trigger an event to all registered handlers
    pub fn trigger(&self, event: DbManagerEvent) {
        if let Ok(handlers) = self.handlers.read() {
            for handler in handlers.iter() {
                handler(event.clone());
            }
        }
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.read().map(|h| h.len()).unwrap_or(0)
    }

    /// Clear all handlers (useful for testing)
    pub fn clear(&self) {
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.clear();
        }
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global hook registry instance
static HOOK_REGISTRY: std::sync::OnceLock<HookRegistry> = std::sync::OnceLock::new();

/// Get the global hook registry
pub fn hooks() -> &'static HookRegistry {
    HOOK_REGISTRY.get_or_init(HookRegistry::new)
}

/// Register a hook handler for dbmanager events
///
/// # Example
/// ```rust
/// use rustpress_admin::dbmanager::{register_hook, DbManagerEvent};
///
/// register_hook(|event| {
///     match event {
///         DbManagerEvent::AfterQuery { query, success, .. } => {
///             println!("Query executed: {} (success: {})", query, success);
///         }
///         _ => {}
///     }
/// });
/// ```
pub fn register_hook<F>(handler: F)
where
    F: Fn(DbManagerEvent) + Send + Sync + 'static,
{
    hooks().register(Arc::new(handler));
}

/// Trigger a hook event
pub fn trigger_hook(event: DbManagerEvent) {
    hooks().trigger(event);
}

// ============================================================================
// Handlers - Status & Info
// ============================================================================

/// Get database connection status
pub async fn get_status(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    // Get PostgreSQL version and connection info
    let version_result: Result<(String,), _> = sqlx::query_as("SELECT version()")
        .fetch_one(&state.pool)
        .await;

    let version = version_result
        .map(|(v,)| v)
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get connection pool stats
    let pool_size = state.pool.options().get_max_connections();
    let pool_state = state.pool.size();
    let idle = state.pool.num_idle();

    // Get database name and current user
    let db_info: Result<(String, String), _> = sqlx::query_as(
        "SELECT current_database(), current_user"
    )
    .fetch_one(&state.pool)
    .await;

    let (database, user) = db_info.unwrap_or(("unknown".to_string(), "unknown".to_string()));

    // Get host and port from the actual connection
    // inet_server_addr() returns the server IP, inet_server_port() returns the port
    // For Docker/local connections these might be NULL, so we fall back to checking pg_settings
    let conn_info: Result<(Option<String>, Option<i32>), _> = sqlx::query_as(
        r#"
        SELECT
            COALESCE(
                host(inet_server_addr())::text,
                (SELECT setting FROM pg_settings WHERE name = 'listen_addresses'),
                'localhost'
            ) as host,
            COALESCE(
                inet_server_port(),
                (SELECT setting::int FROM pg_settings WHERE name = 'port'),
                5432
            ) as port
        "#
    )
    .fetch_one(&state.pool)
    .await;

    let (host, port) = match conn_info {
        Ok((h, p)) => (
            h.unwrap_or_else(|| "localhost".to_string()),
            p.unwrap_or(5432) as u16
        ),
        Err(_) => ("localhost".to_string(), 5432u16)
    };

    // Get uptime
    let uptime_result: Result<(String,), _> = sqlx::query_as(
        "SELECT to_char(NOW() - pg_postmaster_start_time(), 'DD \"days\" HH24:MI:SS')"
    )
    .fetch_one(&state.pool)
    .await;

    let uptime = uptime_result.map(|(u,)| u).unwrap_or_else(|_| "Unknown".to_string());

    // Check SSL
    let ssl_result: Result<(bool,), _> = sqlx::query_as(
        "SELECT ssl FROM pg_stat_ssl WHERE pid = pg_backend_pid()"
    )
    .fetch_optional(&state.pool)
    .await
    .map(|opt| opt.unwrap_or((false,)));

    let ssl = ssl_result.unwrap_or((false,)).0;

    let status = DatabaseStatus {
        connected: true,
        db_type: "PostgreSQL".to_string(),
        version,
        host,
        port,
        database,
        user,
        ssl,
        uptime,
        pool_size,
        active_connections: pool_state - idle as u32,
        idle_connections: idle as u32,
    };

    Json(ApiResponse {
        success: true,
        data: Some(status),
        error: None,
    })
}

/// Get database statistics
pub async fn get_stats(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    // Get table count
    let table_count: Result<(i64,), _> = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_one(&state.pool)
    .await;

    let total_tables = table_count.map(|(c,)| c as u32).unwrap_or(0);

    // Get total row estimate (faster than COUNT(*) on all tables)
    let row_estimate: Result<(i64,), _> = sqlx::query_as(
        "SELECT COALESCE(SUM(n_live_tup), 0) FROM pg_stat_user_tables"
    )
    .fetch_one(&state.pool)
    .await;

    let total_rows = row_estimate.map(|(r,)| r as u64).unwrap_or(0);

    // Get database size
    let db_size: Result<(String,), _> = sqlx::query_as(
        "SELECT pg_size_pretty(pg_database_size(current_database()))"
    )
    .fetch_one(&state.pool)
    .await;

    let database_size = db_size.map(|(s,)| s).unwrap_or_else(|_| "Unknown".to_string());

    // Get index size
    let idx_size: Result<(String,), _> = sqlx::query_as(
        "SELECT pg_size_pretty(SUM(pg_indexes_size(relid))) FROM pg_stat_user_tables"
    )
    .fetch_one(&state.pool)
    .await;

    let index_size = idx_size.map(|(s,)| s).unwrap_or_else(|_| "Unknown".to_string());

    // Get cache hit ratio
    let cache_ratio: Result<(Option<f64>,), _> = sqlx::query_as(
        r#"SELECT
            CASE WHEN (blks_hit + blks_read) > 0
            THEN 100.0 * blks_hit / (blks_hit + blks_read)
            ELSE 100.0 END
        FROM pg_stat_database
        WHERE datname = current_database()"#
    )
    .fetch_one(&state.pool)
    .await;

    let cache_hit_ratio = cache_ratio.map(|(r,)| r.unwrap_or(0.0)).unwrap_or(0.0);

    // Get average query time from pg_stat_statements if available
    let avg_time: Result<(Option<f64>,), _> = sqlx::query_as(
        "SELECT AVG(mean_exec_time) FROM pg_stat_statements WHERE dbid = (SELECT oid FROM pg_database WHERE datname = current_database())"
    )
    .fetch_one(&state.pool)
    .await;

    let avg_query_time = avg_time.map(|(t,)| t.unwrap_or(0.0)).unwrap_or(0.0);

    // Get queries per second (from pg_stat_database)
    let qps: Result<(Option<i64>, Option<i64>, Option<f64>), _> = sqlx::query_as(
        r#"SELECT
            xact_commit + xact_rollback,
            EXTRACT(EPOCH FROM (NOW() - stats_reset))::bigint,
            CASE WHEN EXTRACT(EPOCH FROM (NOW() - stats_reset)) > 0
            THEN (xact_commit + xact_rollback)::float / EXTRACT(EPOCH FROM (NOW() - stats_reset))
            ELSE 0 END
        FROM pg_stat_database
        WHERE datname = current_database()"#
    )
    .fetch_one(&state.pool)
    .await;

    let queries_per_second = qps.map(|(_, _, q)| q.unwrap_or(0.0)).unwrap_or(0.0);

    // Get slow queries count (queries > 1 second from pg_stat_statements if available)
    let slow_count: Result<(i64,), _> = sqlx::query_as(
        "SELECT COALESCE(COUNT(*), 0) FROM pg_stat_statements WHERE mean_exec_time > 1000 AND dbid = (SELECT oid FROM pg_database WHERE datname = current_database())"
    )
    .fetch_one(&state.pool)
    .await;

    let slow_queries = slow_count.map(|(c,)| c as u32).unwrap_or(0);

    let stats = DatabaseStats {
        total_tables,
        total_rows,
        database_size,
        index_size,
        cache_hit_ratio,
        avg_query_time,
        queries_per_second,
        slow_queries,
    };

    Json(ApiResponse {
        success: true,
        data: Some(stats),
        error: None,
    })
}

// ============================================================================
// Handlers - Tables
// ============================================================================

/// List all tables
pub async fn list_tables(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<ListTablesQuery>,
) -> impl IntoResponse {
    let show_system = params.show_system.unwrap_or(false);

    // Query for tables with statistics
    let schema_filter = if show_system {
        "1=1"
    } else {
        "t.table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')"
    };

    let query = format!(r#"
        SELECT
            t.table_name,
            t.table_schema,
            t.table_type,
            COALESCE(s.n_live_tup, 0) as row_count,
            pg_size_pretty(COALESCE(pg_total_relation_size(quote_ident(t.table_schema) || '.' || quote_ident(t.table_name)), 0)) as total_size,
            pg_size_pretty(COALESCE(pg_indexes_size(quote_ident(t.table_schema) || '.' || quote_ident(t.table_name)), 0)) as index_size,
            EXISTS (
                SELECT 1 FROM information_schema.table_constraints tc
                WHERE tc.table_name = t.table_name
                AND tc.table_schema = t.table_schema
                AND tc.constraint_type = 'FOREIGN KEY'
            ) OR EXISTS (
                SELECT 1 FROM information_schema.constraint_column_usage ccu
                WHERE ccu.table_name = t.table_name
                AND ccu.table_schema = t.table_schema
            ) as has_relations,
            COALESCE(s.last_autovacuum, s.last_vacuum, s.last_autoanalyze, s.last_analyze, NULL)::text as last_modified
        FROM information_schema.tables t
        LEFT JOIN pg_stat_user_tables s ON s.relname = t.table_name AND s.schemaname = t.table_schema
        WHERE {schema_filter}
        ORDER BY t.table_schema, t.table_name
    "#);

    let rows = sqlx::query(&query)
        .fetch_all(&state.pool)
        .await;

    match rows {
        Ok(rows) => {
            let tables: Vec<TableInfo> = rows.iter().map(|row| {
                let table_type_raw: String = row.get("table_type");
                let table_type = match table_type_raw.as_str() {
                    "BASE TABLE" => "table",
                    "VIEW" => "view",
                    _ => "table",
                }.to_string();

                TableInfo {
                    name: row.get("table_name"),
                    schema: row.get("table_schema"),
                    rows: row.get::<i64, _>("row_count") as u64,
                    size: row.get("total_size"),
                    index_size: row.get("index_size"),
                    has_relations: row.get("has_relations"),
                    last_modified: row.get::<Option<String>, _>("last_modified").unwrap_or_default(),
                    table_type,
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(TablesResponse { tables }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to list tables: {}", e)),
            })
        }
    }
}

/// Get table details
pub async fn get_table(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    // Get table info
    let table_query = r#"
        SELECT
            t.table_name,
            t.table_schema,
            t.table_type,
            COALESCE(s.n_live_tup, 0) as row_count,
            pg_size_pretty(COALESCE(pg_total_relation_size(quote_ident(t.table_schema) || '.' || quote_ident(t.table_name)), 0)) as total_size,
            pg_size_pretty(COALESCE(pg_indexes_size(quote_ident(t.table_schema) || '.' || quote_ident(t.table_name)), 0)) as index_size,
            COALESCE(s.last_autovacuum, s.last_vacuum, s.last_autoanalyze, s.last_analyze, NULL)::text as last_modified
        FROM information_schema.tables t
        LEFT JOIN pg_stat_user_tables s ON s.relname = t.table_name AND s.schemaname = t.table_schema
        WHERE t.table_name = $1 AND t.table_schema = 'public'
    "#;

    let table_row = sqlx::query(table_query)
        .bind(&table_name)
        .fetch_optional(&state.pool)
        .await;

    let table = match table_row {
        Ok(Some(row)) => {
            let table_type_raw: String = row.get("table_type");
            TableInfo {
                name: row.get("table_name"),
                schema: row.get("table_schema"),
                rows: row.get::<i64, _>("row_count") as u64,
                size: row.get("total_size"),
                index_size: row.get("index_size"),
                has_relations: false, // Will check below
                last_modified: row.get::<Option<String>, _>("last_modified").unwrap_or_default(),
                table_type: if table_type_raw == "BASE TABLE" { "table".to_string() } else { "view".to_string() },
            }
        }
        Ok(None) => {
            return Json(ApiResponse::<TableDetailResponse> {
                success: false,
                data: None,
                error: Some(format!("Table '{}' not found", table_name)),
            });
        }
        Err(e) => {
            return Json(ApiResponse::<TableDetailResponse> {
                success: false,
                data: None,
                error: Some(format!("Failed to get table: {}", e)),
            });
        }
    };

    // Get columns
    let columns_query = r#"
        SELECT
            c.column_name,
            c.data_type,
            c.udt_name,
            c.character_maximum_length,
            c.numeric_precision,
            c.numeric_scale,
            c.is_nullable,
            c.column_default,
            COALESCE(pk.is_primary_key, false) as is_primary_key,
            COALESCE(fk.is_foreign_key, false) as is_foreign_key,
            fk.foreign_table_name,
            fk.foreign_column_name
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT kcu.column_name, true as is_primary_key
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'
        ) pk ON pk.column_name = c.column_name
        LEFT JOIN (
            SELECT
                kcu.column_name,
                true as is_foreign_key,
                ccu.table_name as foreign_table_name,
                ccu.column_name as foreign_column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
            WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
        ) fk ON fk.column_name = c.column_name
        WHERE c.table_name = $1 AND c.table_schema = 'public'
        ORDER BY c.ordinal_position
    "#;

    let column_rows = sqlx::query(columns_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    let columns: Vec<ColumnInfo> = match column_rows {
        Ok(rows) => rows.iter().map(|row| {
            let data_type: String = row.get("data_type");
            let udt_name: String = row.get("udt_name");
            let char_len: Option<i32> = row.get("character_maximum_length");
            let num_prec: Option<i32> = row.get("numeric_precision");

            let full_type = if let Some(len) = char_len {
                format!("{}({})", data_type, len)
            } else if let Some(prec) = num_prec {
                let scale: Option<i32> = row.get("numeric_scale");
                if let Some(s) = scale {
                    format!("{}({},{})", data_type, prec, s)
                } else {
                    format!("{}({})", data_type, prec)
                }
            } else if data_type == "ARRAY" {
                format!("{}[]", udt_name.trim_start_matches('_'))
            } else if data_type == "USER-DEFINED" {
                udt_name.clone()
            } else {
                data_type.clone()
            };

            let is_fk: bool = row.get("is_foreign_key");
            let references = if is_fk {
                Some(ForeignKeyReference {
                    table: row.get::<Option<String>, _>("foreign_table_name").unwrap_or_default(),
                    column: row.get::<Option<String>, _>("foreign_column_name").unwrap_or_default(),
                })
            } else {
                None
            };

            ColumnInfo {
                name: row.get("column_name"),
                data_type: full_type,
                nullable: row.get::<String, _>("is_nullable") == "YES",
                default_value: row.get("column_default"),
                is_primary_key: row.get("is_primary_key"),
                is_foreign_key: is_fk,
                references,
            }
        }).collect(),
        Err(_) => vec![],
    };

    // Get indexes
    let indexes_query = r#"
        SELECT
            i.relname as index_name,
            array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as columns,
            ix.indisunique as is_unique,
            am.amname as index_type
        FROM pg_class t
        JOIN pg_index ix ON t.oid = ix.indrelid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_am am ON i.relam = am.oid
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
        WHERE t.relname = $1 AND t.relkind = 'r'
        GROUP BY i.relname, ix.indisunique, am.amname
        ORDER BY i.relname
    "#;

    let index_rows = sqlx::query(indexes_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    let indexes: Vec<IndexInfo> = match index_rows {
        Ok(rows) => rows.iter().map(|row| {
            IndexInfo {
                name: row.get("index_name"),
                columns: row.get::<Vec<String>, _>("columns"),
                unique: row.get("is_unique"),
                index_type: row.get("index_type"),
            }
        }).collect(),
        Err(_) => vec![],
    };

    // Get foreign keys
    let fk_query = r#"
        SELECT
            tc.constraint_name,
            kcu.column_name,
            ccu.table_name as foreign_table_name,
            ccu.column_name as foreign_column_name,
            rc.delete_rule,
            rc.update_rule
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
        JOIN information_schema.referential_constraints rc ON tc.constraint_name = rc.constraint_name
        WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
        ORDER BY tc.constraint_name
    "#;

    let fk_rows = sqlx::query(fk_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    let foreign_keys: Vec<ForeignKeyInfo> = match fk_rows {
        Ok(rows) => rows.iter().map(|row| {
            ForeignKeyInfo {
                name: row.get("constraint_name"),
                column: row.get("column_name"),
                referenced_table: row.get("foreign_table_name"),
                referenced_column: row.get("foreign_column_name"),
                on_delete: row.get("delete_rule"),
                on_update: row.get("update_rule"),
            }
        }).collect(),
        Err(_) => vec![],
    };

    // Update has_relations
    let mut table_with_relations = table;
    table_with_relations.has_relations = !foreign_keys.is_empty();

    Json(ApiResponse {
        success: true,
        data: Some(TableDetailResponse {
            table: table_with_relations,
            columns,
            indexes,
            foreign_keys,
        }),
        error: None,
    })
}

/// Get table columns
pub async fn get_table_columns(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let columns_query = r#"
        SELECT
            c.column_name,
            c.data_type,
            c.udt_name,
            c.character_maximum_length,
            c.numeric_precision,
            c.numeric_scale,
            c.is_nullable,
            c.column_default,
            COALESCE(pk.is_primary_key, false) as is_primary_key,
            COALESCE(fk.is_foreign_key, false) as is_foreign_key,
            fk.foreign_table_name,
            fk.foreign_column_name
        FROM information_schema.columns c
        LEFT JOIN (
            SELECT kcu.column_name, true as is_primary_key
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'
        ) pk ON pk.column_name = c.column_name
        LEFT JOIN (
            SELECT
                kcu.column_name,
                true as is_foreign_key,
                ccu.table_name as foreign_table_name,
                ccu.column_name as foreign_column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
            WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
        ) fk ON fk.column_name = c.column_name
        WHERE c.table_name = $1 AND c.table_schema = 'public'
        ORDER BY c.ordinal_position
    "#;

    let rows = sqlx::query(columns_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    match rows {
        Ok(rows) => {
            let columns: Vec<ColumnInfo> = rows.iter().map(|row| {
                let data_type: String = row.get("data_type");
                let udt_name: String = row.get("udt_name");
                let char_len: Option<i32> = row.get("character_maximum_length");
                let num_prec: Option<i32> = row.get("numeric_precision");

                let full_type = if let Some(len) = char_len {
                    format!("{}({})", data_type, len)
                } else if let Some(prec) = num_prec {
                    let scale: Option<i32> = row.get("numeric_scale");
                    if let Some(s) = scale {
                        format!("{}({},{})", data_type, prec, s)
                    } else {
                        format!("{}({})", data_type, prec)
                    }
                } else if data_type == "ARRAY" {
                    format!("{}[]", udt_name.trim_start_matches('_'))
                } else if data_type == "USER-DEFINED" {
                    udt_name.clone()
                } else {
                    data_type.clone()
                };

                let is_fk: bool = row.get("is_foreign_key");
                let references = if is_fk {
                    Some(ForeignKeyReference {
                        table: row.get::<Option<String>, _>("foreign_table_name").unwrap_or_default(),
                        column: row.get::<Option<String>, _>("foreign_column_name").unwrap_or_default(),
                    })
                } else {
                    None
                };

                ColumnInfo {
                    name: row.get("column_name"),
                    data_type: full_type,
                    nullable: row.get::<String, _>("is_nullable") == "YES",
                    default_value: row.get("column_default"),
                    is_primary_key: row.get("is_primary_key"),
                    is_foreign_key: is_fk,
                    references,
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(ColumnsResponse { columns }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to get columns: {}", e)),
            })
        }
    }
}

/// Get table indexes
pub async fn get_table_indexes(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let indexes_query = r#"
        SELECT
            i.relname as index_name,
            array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as columns,
            ix.indisunique as is_unique,
            am.amname as index_type
        FROM pg_class t
        JOIN pg_index ix ON t.oid = ix.indrelid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_am am ON i.relam = am.oid
        JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
        WHERE t.relname = $1 AND t.relkind = 'r'
        GROUP BY i.relname, ix.indisunique, am.amname
        ORDER BY i.relname
    "#;

    let rows = sqlx::query(indexes_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    match rows {
        Ok(rows) => {
            let indexes: Vec<IndexInfo> = rows.iter().map(|row| {
                IndexInfo {
                    name: row.get("index_name"),
                    columns: row.get::<Vec<String>, _>("columns"),
                    unique: row.get("is_unique"),
                    index_type: row.get("index_type"),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(IndexesResponse { indexes }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to get indexes: {}", e)),
            })
        }
    }
}

/// Get table foreign keys
pub async fn get_table_foreign_keys(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let fk_query = r#"
        SELECT
            tc.constraint_name,
            kcu.column_name,
            ccu.table_name as foreign_table_name,
            ccu.column_name as foreign_column_name,
            rc.delete_rule,
            rc.update_rule
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
        JOIN information_schema.referential_constraints rc ON tc.constraint_name = rc.constraint_name
        WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
        ORDER BY tc.constraint_name
    "#;

    let rows = sqlx::query(fk_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await;

    match rows {
        Ok(rows) => {
            let foreign_keys: Vec<ForeignKeyInfo> = rows.iter().map(|row| {
                ForeignKeyInfo {
                    name: row.get("constraint_name"),
                    column: row.get("column_name"),
                    referenced_table: row.get("foreign_table_name"),
                    referenced_column: row.get("foreign_column_name"),
                    on_delete: row.get("delete_rule"),
                    on_update: row.get("update_rule"),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(ForeignKeysResponse { foreign_keys }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to get foreign keys: {}", e)),
            })
        }
    }
}

/// Create a new table
pub async fn create_table(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<CreateTableRequest>,
) -> impl IntoResponse {
    // Validate table name (prevent SQL injection)
    if !is_valid_identifier(&request.name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    // Build CREATE TABLE statement
    let mut columns_sql = Vec::new();
    let mut primary_keys = Vec::new();

    for col in &request.columns {
        if !is_valid_identifier(&col.name) {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Invalid column name: {}", col.name)),
            });
        }

        let mut col_def = format!("\"{}\" {}", col.name, col.data_type);
        if !col.nullable {
            col_def.push_str(" NOT NULL");
        }
        if let Some(default) = &col.default_value {
            col_def.push_str(&format!(" DEFAULT {}", default));
        }
        columns_sql.push(col_def);

        if col.is_primary_key {
            primary_keys.push(format!("\"{}\"", col.name));
        }
    }

    if !primary_keys.is_empty() {
        columns_sql.push(format!("PRIMARY KEY ({})", primary_keys.join(", ")));
    }

    let sql = format!(
        "CREATE TABLE \"{}\" ({})",
        request.name,
        columns_sql.join(", ")
    );

    match sqlx::query(&sql).execute(&state.pool).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some(format!("Table '{}' created successfully", request.name)),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to create table: {}", e)),
        }),
    }
}

/// Alter table structure
pub async fn alter_table(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Json(request): Json<AlterTableRequest>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    let mut statements = Vec::new();

    // Add columns
    if let Some(cols) = &request.add_columns {
        for col in cols {
            if !is_valid_identifier(&col.name) {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid column name: {}", col.name)),
                });
            }
            let mut col_def = format!("ADD COLUMN \"{}\" {}", col.name, col.data_type);
            if !col.nullable {
                col_def.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }
            statements.push(col_def);
        }
    }

    // Drop columns
    if let Some(cols) = &request.drop_columns {
        for col in cols {
            if !is_valid_identifier(col) {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid column name: {}", col)),
                });
            }
            statements.push(format!("DROP COLUMN \"{}\"", col));
        }
    }

    // Modify columns
    if let Some(cols) = &request.modify_columns {
        for col in cols {
            if !is_valid_identifier(&col.name) {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid column name: {}", col.name)),
                });
            }
            statements.push(format!("ALTER COLUMN \"{}\" TYPE {}", col.name, col.data_type));
            if col.nullable {
                statements.push(format!("ALTER COLUMN \"{}\" DROP NOT NULL", col.name));
            } else {
                statements.push(format!("ALTER COLUMN \"{}\" SET NOT NULL", col.name));
            }
            if let Some(default) = &col.default_value {
                statements.push(format!("ALTER COLUMN \"{}\" SET DEFAULT {}", col.name, default));
            }
        }
    }

    // Add indexes
    if let Some(indexes) = &request.add_indexes {
        for idx in indexes {
            if !is_valid_identifier(&idx.name) {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid index name: {}", idx.name)),
                });
            }
            let unique = if idx.unique { "UNIQUE " } else { "" };
            let cols: Vec<String> = idx.columns.iter().map(|c| format!("\"{}\"", c)).collect();
            let create_idx = format!(
                "CREATE {}INDEX \"{}\" ON \"{}\" ({})",
                unique, idx.name, table_name, cols.join(", ")
            );
            // Execute index creation separately (not part of ALTER TABLE)
            if let Err(e) = sqlx::query(&create_idx).execute(&state.pool).await {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to create index: {}", e)),
                });
            }
        }
    }

    // Drop indexes
    if let Some(indexes) = &request.drop_indexes {
        for idx in indexes {
            if !is_valid_identifier(idx) {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid index name: {}", idx)),
                });
            }
            let drop_idx = format!("DROP INDEX IF EXISTS \"{}\"", idx);
            if let Err(e) = sqlx::query(&drop_idx).execute(&state.pool).await {
                return Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to drop index: {}", e)),
                });
            }
        }
    }

    if !statements.is_empty() {
        let sql = format!("ALTER TABLE \"{}\" {}", table_name, statements.join(", "));
        if let Err(e) = sqlx::query(&sql).execute(&state.pool).await {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to alter table: {}", e)),
            });
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(format!("Table '{}' altered successfully", table_name)),
        error: None,
    })
}

/// Query params for destructive operations requiring confirmation
#[derive(Debug, Deserialize)]
pub struct ConfirmQuery {
    /// Must be true to proceed with destructive operation
    #[serde(default)]
    pub confirm: bool,
}

/// Drop a table (requires confirmation)
pub async fn drop_table(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Query(params): Query<ConfirmQuery>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    // Require explicit confirmation for destructive operation
    if !params.confirm {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!(
                "Dropping table '{}' is a destructive operation. Add ?confirm=true to the request to proceed. \
                This will permanently delete the table and all its data.",
                table_name
            )),
        });
    }

    let sql = format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table_name);
    match sqlx::query(&sql).execute(&state.pool).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some(format!("Table '{}' dropped successfully", table_name)),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to drop table: {}", e)),
        }),
    }
}

/// Truncate a table (requires confirmation)
pub async fn truncate_table(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Query(params): Query<ConfirmQuery>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    // Require explicit confirmation for destructive operation
    if !params.confirm {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!(
                "Truncating table '{}' is a destructive operation. Add ?confirm=true to the request to proceed. \
                This will permanently delete all data in the table.",
                table_name
            )),
        });
    }

    let sql = format!("TRUNCATE TABLE \"{}\" RESTART IDENTITY CASCADE", table_name);
    match sqlx::query(&sql).execute(&state.pool).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some(format!("Table '{}' truncated successfully", table_name)),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to truncate table: {}", e)),
        }),
    }
}

/// Validate identifier (table name, column name, etc.) to prevent SQL injection
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 63 {
        return false;
    }
    // PostgreSQL identifier rules: start with letter or underscore, followed by letters, digits, underscores
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Get the primary key column(s) for a table
async fn get_primary_key_columns(pool: &sqlx::PgPool, table_name: &str) -> Result<Vec<String>, sqlx::Error> {
    let query = r#"
        SELECT kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        WHERE tc.table_name = $1
            AND tc.table_schema = 'public'
            AND tc.constraint_type = 'PRIMARY KEY'
        ORDER BY kcu.ordinal_position
    "#;

    let rows: Vec<(String,)> = sqlx::query_as(query)
        .bind(table_name)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|(col,)| col).collect())
}

/// Get the first primary key column or fallback to "id"
async fn get_primary_key_column(pool: &sqlx::PgPool, table_name: &str) -> String {
    match get_primary_key_columns(pool, table_name).await {
        Ok(cols) if !cols.is_empty() => cols[0].clone(),
        _ => "id".to_string(),
    }
}

// ============================================================================
// Handlers - Data Operations
// ============================================================================

/// Get table data with pagination
pub async fn get_table_data(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Query(params): Query<TableDataQuery>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000
    let offset = (page - 1) * limit;

    // Build WHERE clause with proper parameterization
    let mut where_clauses: Vec<String> = Vec::new();
    let mut param_values: Vec<String> = Vec::new();
    let mut param_idx = 1u32;

    // Filter by specific column
    if let (Some(filter_col), Some(filter_val)) = (&params.filter_column, &params.filter_value) {
        if !filter_val.is_empty() && is_valid_identifier(filter_col) {
            where_clauses.push(format!("CAST(\"{}\" AS TEXT) ILIKE ${}", filter_col, param_idx));
            param_values.push(format!("%{}%", filter_val));
            param_idx += 1;
        }
    }

    // Search across all text columns
    if let Some(search) = &params.search {
        if !search.is_empty() {
            // Get text columns for the table
            let columns_query = r#"
                SELECT column_name, data_type
                FROM information_schema.columns
                WHERE table_name = $1 AND table_schema = 'public'
                AND data_type IN ('character varying', 'text', 'char', 'varchar', 'uuid')
            "#;

            let text_columns: Vec<String> = match sqlx::query_as::<_, (String, String)>(columns_query)
                .bind(&table_name)
                .fetch_all(&state.pool)
                .await
            {
                Ok(cols) => cols.into_iter().map(|(name, _)| name).collect(),
                Err(_) => vec!["id".to_string()], // Fallback to id
            };

            if !text_columns.is_empty() {
                let search_conditions: Vec<String> = text_columns
                    .iter()
                    .filter(|col| is_valid_identifier(col))
                    .map(|col| format!("CAST(\"{}\" AS TEXT) ILIKE ${}", col, param_idx))
                    .collect();

                if !search_conditions.is_empty() {
                    where_clauses.push(format!("({})", search_conditions.join(" OR ")));
                    param_values.push(format!("%{}%", search));
                    #[allow(unused_assignments)]
                    { param_idx += 1; }
                }
            }
        }
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    // Get total count with filter
    let count_sql = format!("SELECT COUNT(*) as count FROM \"{}\"{}", table_name, where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &param_values {
        count_query = count_query.bind(value);
    }

    let total: i64 = match count_query.fetch_one(&state.pool).await {
        Ok(count) => count,
        Err(e) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to count rows: {}", e)),
            });
        }
    };

    // Build data query with sorting
    let mut sql = format!("SELECT * FROM \"{}\"{}", table_name, where_clause);

    if let Some(sort_col) = &params.sort_column {
        if is_valid_identifier(sort_col) {
            let order = params.sort_order.as_deref().unwrap_or("ASC");
            let order = if order.to_uppercase() == "DESC" { "DESC" } else { "ASC" };
            sql.push_str(&format!(" ORDER BY \"{}\" {}", sort_col, order));
        }
    }

    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let mut data_query = sqlx::query(&sql);
    for value in &param_values {
        data_query = data_query.bind(value);
    }

    let rows = data_query.fetch_all(&state.pool).await;

    // Get primary key columns for this table
    let pk_columns = get_primary_key_columns(&state.pool, &table_name)
        .await
        .unwrap_or_else(|_| vec!["id".to_string()]);

    match rows {
        Ok(rows) => {
            let data_rows: Vec<HashMap<String, serde_json::Value>> = rows.iter().map(|row| {
                let mut map = HashMap::new();
                for col in row.columns() {
                    let name = col.name();
                    let value = convert_pg_value_to_json(row, col);
                    map.insert(name.to_string(), value);
                }
                map
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(TableDataResponse {
                    rows: data_rows,
                    total: total as u64,
                    page,
                    page_size: limit,
                    primary_key_columns: pk_columns,
                }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch data: {}", e)),
            })
        }
    }
}

/// Convert PostgreSQL row value to JSON
fn convert_pg_value_to_json(row: &sqlx::postgres::PgRow, col: &sqlx::postgres::PgColumn) -> serde_json::Value {
    let type_name = col.type_info().name();

    // Try to get value based on type
    match type_name {
        "BOOL" => row.try_get::<bool, _>(col.name())
            .map(serde_json::Value::Bool)
            .unwrap_or(serde_json::Value::Null),
        "INT2" => row.try_get::<i16, _>(col.name())
            .map(|v| serde_json::Value::Number(v.into()))
            .unwrap_or(serde_json::Value::Null),
        "INT4" => row.try_get::<i32, _>(col.name())
            .map(|v| serde_json::Value::Number(v.into()))
            .unwrap_or(serde_json::Value::Null),
        "INT8" => row.try_get::<i64, _>(col.name())
            .map(|v| serde_json::Value::Number(v.into()))
            .unwrap_or(serde_json::Value::Null),
        "FLOAT4" => row.try_get::<f32, _>(col.name())
            .map(|v| serde_json::Number::from_f64(v as f64).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null),
        "FLOAT8" => row.try_get::<f64, _>(col.name())
            .map(|v| serde_json::Number::from_f64(v).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null),
        "UUID" => row.try_get::<uuid::Uuid, _>(col.name())
            .map(|v| serde_json::Value::String(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
        "TIMESTAMPTZ" | "TIMESTAMP" => row.try_get::<chrono::DateTime<chrono::Utc>, _>(col.name())
            .map(|v| serde_json::Value::String(v.to_rfc3339()))
            .unwrap_or(serde_json::Value::Null),
        "DATE" => row.try_get::<chrono::NaiveDate, _>(col.name())
            .map(|v| serde_json::Value::String(v.to_string()))
            .unwrap_or(serde_json::Value::Null),
        "JSON" | "JSONB" => row.try_get::<serde_json::Value, _>(col.name())
            .unwrap_or(serde_json::Value::Null),
        _ => row.try_get::<String, _>(col.name())
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null),
    }
}

/// Insert a new row
pub async fn insert_row(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Json(request): Json<InsertRowRequest>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    if request.data.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("No data provided".to_string()),
        });
    }

    let columns: Vec<&String> = request.data.keys().collect();
    let mut placeholders = Vec::new();
    let mut values: Vec<String> = Vec::new();

    for (i, (_key, value)) in request.data.iter().enumerate() {
        placeholders.push(format!("${}", i + 1));
        values.push(json_value_to_sql_string(value));
    }

    let columns_str: Vec<String> = columns.iter().map(|c| format!("\"{}\"", c)).collect();
    let sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
        table_name,
        columns_str.join(", "),
        placeholders.join(", ")
    );

    // Build query with bound parameters
    let mut query = sqlx::query(&sql);
    for value in &request.data.values().collect::<Vec<_>>() {
        query = bind_json_value(query, value);
    }

    match query.fetch_one(&state.pool).await {
        Ok(row) => {
            let mut result = HashMap::new();
            for col in row.columns() {
                let value = convert_pg_value_to_json(&row, col);
                result.insert(col.name().to_string(), value);
            }
            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "row": result,
                    "message": format!("Row inserted into '{}'", table_name)
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to insert row: {}", e)),
        }),
    }
}

/// Bind JSON value to query
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: &serde_json::Value,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match value {
        serde_json::Value::Null => query.bind(None::<String>),
        serde_json::Value::Bool(b) => query.bind(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        serde_json::Value::String(s) => query.bind(s.clone()),
        _ => query.bind(value.to_string()),
    }
}

/// Convert JSON value to SQL string representation
fn json_value_to_sql_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        _ => value.to_string(),
    }
}

/// Convert a PostgreSQL row to a JSON HashMap
fn row_to_json(row: &sqlx::postgres::PgRow) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    for col in row.columns() {
        let value = convert_pg_value_to_json(row, col);
        result.insert(col.name().to_string(), value);
    }
    result
}

/// Get column value as a displayable string
fn get_column_value_as_string(row: &sqlx::postgres::PgRow, column: &str) -> String {
    row.try_get::<String, _>(column)
        .unwrap_or_else(|_| {
            // Try other types
            row.try_get::<i64, _>(column)
                .map(|v| v.to_string())
                .or_else(|_| row.try_get::<f64, _>(column).map(|v| v.to_string()))
                .or_else(|_| row.try_get::<bool, _>(column).map(|v| v.to_string()))
                .or_else(|_| row.try_get::<uuid::Uuid, _>(column).map(|v| v.to_string()))
                .unwrap_or_else(|_| String::new())
        })
}

/// Update a row
pub async fn update_row(
    State(state): State<Arc<DbManagerState>>,
    Path((table_name, row_id)): Path<(String, String)>,
    Json(request): Json<UpdateRowRequest>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    if request.data.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("No data provided".to_string()),
        });
    }

    // Get dynamic primary key column
    let pk_column = get_primary_key_column(&state.pool, &table_name).await;

    let mut set_clauses = Vec::new();
    let mut param_idx = 1;

    for key in request.data.keys() {
        if !is_valid_identifier(key) {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Invalid column name: {}", key)),
            });
        }
        set_clauses.push(format!("\"{}\" = ${}", key, param_idx));
        param_idx += 1;
    }

    let sql = format!(
        "UPDATE \"{}\" SET {} WHERE \"{}\" = ${} RETURNING *",
        table_name,
        set_clauses.join(", "),
        pk_column,
        param_idx
    );

    let mut query = sqlx::query(&sql);
    for value in request.data.values() {
        query = bind_json_value(query, value);
    }
    query = query.bind(&row_id);

    match query.fetch_one(&state.pool).await {
        Ok(row) => {
            let mut result = HashMap::new();
            for col in row.columns() {
                let value = convert_pg_value_to_json(&row, col);
                result.insert(col.name().to_string(), value);
            }
            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "row": result,
                    "message": format!("Row '{}' in '{}' updated successfully", row_id, table_name)
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to update row: {}", e)),
        }),
    }
}

/// Delete a row
pub async fn delete_row(
    State(state): State<Arc<DbManagerState>>,
    Path((table_name, row_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    // Get dynamic primary key column
    let pk_column = get_primary_key_column(&state.pool, &table_name).await;

    let sql = format!("DELETE FROM \"{}\" WHERE \"{}\" = $1", table_name, pk_column);
    match sqlx::query(&sql)
        .bind(&row_id)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Json(ApiResponse {
                    success: true,
                    data: Some(format!("Row '{}' deleted from '{}'", row_id, table_name)),
                    error: None,
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Row '{}' not found in '{}'", row_id, table_name)),
                })
            }
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to delete row: {}", e)),
        }),
    }
}

/// Bulk delete rows
pub async fn bulk_delete_rows(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
    Json(request): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    if request.ids.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("No IDs provided".to_string()),
        });
    }

    // Get dynamic primary key column
    let pk_column = get_primary_key_column(&state.pool, &table_name).await;

    // Build placeholders for IN clause
    let placeholders: Vec<String> = (1..=request.ids.len()).map(|i| format!("${}", i)).collect();
    let sql = format!(
        "DELETE FROM \"{}\" WHERE \"{}\" IN ({})",
        table_name,
        pk_column,
        placeholders.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for id in &request.ids {
        query = query.bind(id);
    }

    match query.execute(&state.pool).await {
        Ok(result) => Json(ApiResponse {
            success: true,
            data: Some(format!("{} rows deleted from '{}'", result.rows_affected(), table_name)),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to delete rows: {}", e)),
        }),
    }
}

// ============================================================================
// Handlers - Query Execution
// ============================================================================

/// Execute a SQL query
pub async fn execute_query(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExecuteQueryRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Rate limiting check (using "global" key for now, can be per-user with auth)
    match state.rate_limiter.check("query_execution") {
        Ok(_remaining) => {}
        Err(retry_after) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!(
                    "Rate limit exceeded. Please try again in {} seconds. Max {} queries per minute allowed.",
                    retry_after, 100
                )),
            });
        }
    }

    // Basic query validation
    let query = request.query.trim();
    if query.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Query cannot be empty".to_string()),
        });
    }

    // Check for blocked commands
    if let Some(blocked_cmd) = is_blocked_query(query) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!(
                "Blocked command: '{}' is not allowed for security reasons. This command could compromise the database system.",
                blocked_cmd
            )),
        });
    }

    // Check for destructive commands that require confirmation
    if is_destructive_query(query) && !request.confirm {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(
                "This query contains a destructive operation (DROP, TRUNCATE, or DELETE). \
                Please set confirm=true in the request to proceed.".to_string()
            ),
        });
    }

    // Detect query type
    let query_upper = query.to_uppercase();
    let is_select = query_upper.starts_with("SELECT") ||
                    query_upper.starts_with("WITH") ||
                    query_upper.starts_with("SHOW") ||
                    query_upper.starts_with("EXPLAIN");

    // Trigger BeforeQuery hook
    trigger_hook(DbManagerEvent::BeforeQuery {
        query: query.to_string(),
        user_id: None, // TODO: Extract from auth context when available
    });

    // Query timeout (30 seconds default)
    let timeout_duration = std::time::Duration::from_secs(30);

    let result = if is_select {
        // Execute SELECT query with timeout
        let query_future = sqlx::query(query).fetch_all(&state.pool);
        match tokio::time::timeout(timeout_duration, query_future).await {
            Ok(Ok(rows)) => {
                let mut columns = Vec::new();
                let mut data_rows = Vec::new();

                if !rows.is_empty() {
                    // Get column names from first row
                    columns = rows[0].columns().iter().map(|c| c.name().to_string()).collect();

                    // Convert rows to JSON
                    for row in &rows {
                        let mut map = HashMap::new();
                        for col in row.columns() {
                            let value = convert_pg_value_to_json(row, col);
                            map.insert(col.name().to_string(), value);
                        }
                        data_rows.push(map);
                    }
                }

                QueryResult {
                    columns,
                    rows: data_rows.clone(),
                    row_count: data_rows.len() as u64,
                    execution_time: start.elapsed().as_secs_f64() * 1000.0,
                    status: "success".to_string(),
                    error: None,
                    affected_rows: None,
                }
            }
            Ok(Err(e)) => QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
                status: "error".to_string(),
                error: Some(e.to_string()),
                affected_rows: None,
            },
            Err(_) => QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
                status: "error".to_string(),
                error: Some(format!("Query timed out after {} seconds. Consider optimizing your query or adding appropriate indexes.", timeout_duration.as_secs())),
                affected_rows: None,
            },
        }
    } else {
        // Execute non-SELECT query with timeout (INSERT, UPDATE, DELETE, CREATE, etc.)
        let query_future = sqlx::query(query).execute(&state.pool);
        match tokio::time::timeout(timeout_duration, query_future).await {
            Ok(Ok(result)) => QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
                status: "success".to_string(),
                error: None,
                affected_rows: Some(result.rows_affected()),
            },
            Ok(Err(e)) => QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
                status: "error".to_string(),
                error: Some(e.to_string()),
                affected_rows: None,
            },
            Err(_) => QueryResult {
                columns: vec![],
                rows: vec![],
                row_count: 0,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
                status: "error".to_string(),
                error: Some(format!("Query timed out after {} seconds. Consider optimizing your query.", timeout_duration.as_secs())),
                affected_rows: None,
            },
        }
    };

    // Add to history (persist to database)
    let _ = sqlx::query(r#"
        INSERT INTO dbmanager_query_history (query, execution_time_ms, status, row_count, error_message, executed_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
    "#)
    .bind(query.chars().take(10000).collect::<String>()) // Store up to 10k chars
    .bind(result.execution_time)
    .bind(&result.status)
    .bind(result.row_count as i64)
    .bind(&result.error)
    .execute(&state.pool)
    .await;

    // Clean up old history entries (keep last 1000)
    let _ = sqlx::query(r#"
        DELETE FROM dbmanager_query_history
        WHERE id NOT IN (
            SELECT id FROM dbmanager_query_history
            ORDER BY executed_at DESC
            LIMIT 1000
        )
    "#)
    .execute(&state.pool)
    .await;

    let success = result.status == "success";

    // Trigger AfterQuery hook
    trigger_hook(DbManagerEvent::AfterQuery {
        query: query.to_string(),
        success,
        execution_time_ms: result.execution_time,
        row_count: if success { Some(result.row_count) } else { None },
        error: result.error.clone(),
        user_id: None,
    });

    Json(ApiResponse {
        success,
        data: Some(result),
        error: None,
    })
}

/// Explain a query
pub async fn explain_query(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExecuteQueryRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();

    let query = request.query.trim();
    if query.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Query cannot be empty".to_string()),
        });
    }

    // Build EXPLAIN ANALYZE query
    let explain_query = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT) {}", query);

    match sqlx::query(&explain_query).fetch_all(&state.pool).await {
        Ok(rows) => {
            let plan: String = rows.iter()
                .filter_map(|row| row.try_get::<String, _>(0).ok())
                .collect::<Vec<_>>()
                .join("\n");

            let explain = QueryExplain {
                plan,
                execution_time: start.elapsed().as_secs_f64() * 1000.0,
            };

            Json(ApiResponse {
                success: true,
                data: Some(explain),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to explain query: {}", e)),
        }),
    }
}

/// Get query history from database
#[allow(clippy::type_complexity)]
pub async fn get_query_history(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<QueryHistoryParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100) as i64;

    let result: Result<Vec<(uuid::Uuid, String, f64, String, Option<i64>, chrono::DateTime<chrono::Utc>)>, _> =
        sqlx::query_as(r#"
            SELECT id, query, execution_time_ms, status, row_count, executed_at
            FROM dbmanager_query_history
            ORDER BY executed_at DESC
            LIMIT $1
        "#)
        .bind(limit)
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(rows) => {
            let history: Vec<QueryHistoryItem> = rows.into_iter().map(|(id, query, execution_time, status, row_count, timestamp)| {
                QueryHistoryItem {
                    id: id.to_string(),
                    query,
                    execution_time,
                    status,
                    timestamp: timestamp.to_rfc3339(),
                    row_count: row_count.map(|c| c as u64),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(QueryHistoryResponse { history }),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to fetch query history: {}", e)),
        })
    }
}

/// Save a query to database
pub async fn save_query(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<SaveQueryRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    let result = sqlx::query(r#"
        INSERT INTO dbmanager_saved_queries (id, name, query, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
    "#)
    .bind(id)
    .bind(&request.name)
    .bind(&request.query)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let saved_query = SavedQuery {
                id: id.to_string(),
                name: request.name,
                query: request.query,
                created_at: now.to_rfc3339(),
            };
            Json(ApiResponse {
                success: true,
                data: Some(saved_query),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to save query: {}", e)),
        })
    }
}

/// Get saved queries from database
#[allow(clippy::type_complexity)]
pub async fn get_saved_queries(State(state): State<Arc<DbManagerState>>) -> impl IntoResponse {
    let result: Result<Vec<(uuid::Uuid, String, String, chrono::DateTime<chrono::Utc>)>, _> =
        sqlx::query_as(r#"
            SELECT id, name, query, created_at
            FROM dbmanager_saved_queries
            ORDER BY created_at DESC
        "#)
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(rows) => {
            let queries: Vec<SavedQuery> = rows.into_iter().map(|(id, name, query, created_at)| {
                SavedQuery {
                    id: id.to_string(),
                    name,
                    query,
                    created_at: created_at.to_rfc3339(),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(SavedQueriesResponse { queries }),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to fetch saved queries: {}", e)),
        })
    }
}

/// Delete a saved query from database
pub async fn delete_saved_query(
    State(state): State<Arc<DbManagerState>>,
    Path(query_id): Path<String>,
) -> impl IntoResponse {
    let id = match uuid::Uuid::parse_str(&query_id) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some("Invalid query ID format".to_string()),
        })
    };

    let result = sqlx::query("DELETE FROM dbmanager_saved_queries WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Json(ApiResponse::<()> {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(format!("Failed to delete query: {}", e)),
        })
    }
}

// ============================================================================
// Handlers - Import/Export
// ============================================================================

/// Export tables
pub async fn export_tables(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExportTablesRequest>,
) -> impl IntoResponse {
    let mut content = String::new();
    let include_structure = request.include_structure.unwrap_or(true);
    let include_data = request.include_data.unwrap_or(true);

    match request.format.as_str() {
        "sql" => {
            for table in &request.tables {
                if !is_valid_identifier(table) {
                    continue;
                }

                if include_structure {
                    // Get table structure
                    let cols_query = r#"
                        SELECT column_name, data_type, is_nullable, column_default
                        FROM information_schema.columns
                        WHERE table_name = $1 AND table_schema = 'public'
                        ORDER BY ordinal_position
                    "#;

                    if let Ok(cols) = sqlx::query(cols_query)
                        .bind(table)
                        .fetch_all(&state.pool)
                        .await
                    {
                        content.push_str(&format!("-- Table structure for {}\n", table));
                        content.push_str(&format!("DROP TABLE IF EXISTS \"{}\" CASCADE;\n", table));
                        content.push_str(&format!("CREATE TABLE \"{}\" (\n", table));

                        let col_defs: Vec<String> = cols.iter().map(|row| {
                            let name: String = row.get("column_name");
                            let dtype: String = row.get("data_type");
                            let nullable: String = row.get("is_nullable");
                            let default: Option<String> = row.get("column_default");

                            let mut def = format!("    \"{}\" {}", name, dtype);
                            if nullable == "NO" {
                                def.push_str(" NOT NULL");
                            }
                            if let Some(d) = default {
                                def.push_str(&format!(" DEFAULT {}", d));
                            }
                            def
                        }).collect();

                        content.push_str(&col_defs.join(",\n"));
                        content.push_str("\n);\n\n");
                    }
                }

                if include_data {
                    // Get table data
                    let data_query = format!("SELECT * FROM \"{}\"", table);
                    if let Ok(rows) = sqlx::query(&data_query).fetch_all(&state.pool).await {
                        if !rows.is_empty() {
                            content.push_str(&format!("-- Data for {}\n", table));

                            let columns: Vec<String> = rows[0].columns().iter()
                                .map(|c| format!("\"{}\"", c.name()))
                                .collect();

                            for row in &rows {
                                let values: Vec<String> = row.columns().iter().map(|col| {
                                    let val = convert_pg_value_to_json(row, col);
                                    match val {
                                        serde_json::Value::Null => "NULL".to_string(),
                                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                                        serde_json::Value::Bool(b) => b.to_string(),
                                        serde_json::Value::Number(n) => n.to_string(),
                                        _ => format!("'{}'", val.to_string().replace('\'', "''")),
                                    }
                                }).collect();

                                content.push_str(&format!(
                                    "INSERT INTO \"{}\" ({}) VALUES ({});\n",
                                    table,
                                    columns.join(", "),
                                    values.join(", ")
                                ));
                            }
                            content.push('\n');
                        }
                    }
                }
            }
        }
        "json" => {
            let mut all_data = serde_json::Map::new();
            for table in &request.tables {
                if !is_valid_identifier(table) {
                    continue;
                }
                let query = format!("SELECT * FROM \"{}\"", table);
                if let Ok(rows) = sqlx::query(&query).fetch_all(&state.pool).await {
                    let table_data: Vec<serde_json::Value> = rows.iter().map(|row| {
                        let mut obj = serde_json::Map::new();
                        for col in row.columns() {
                            let val = convert_pg_value_to_json(row, col);
                            obj.insert(col.name().to_string(), val);
                        }
                        serde_json::Value::Object(obj)
                    }).collect();
                    all_data.insert(table.clone(), serde_json::Value::Array(table_data));
                }
            }
            content = serde_json::to_string_pretty(&serde_json::Value::Object(all_data))
                .unwrap_or_default();
        }
        "csv" => {
            // CSV export only supports single table
            if let Some(table) = request.tables.first() {
                if is_valid_identifier(table) {
                    let query = format!("SELECT * FROM \"{}\"", table);
                    if let Ok(rows) = sqlx::query(&query).fetch_all(&state.pool).await {
                        if !rows.is_empty() {
                            // Header
                            let headers: Vec<String> = rows[0].columns().iter()
                                .map(|c| c.name().to_string())
                                .collect();
                            content.push_str(&headers.join(","));
                            content.push('\n');

                            // Data rows
                            for row in &rows {
                                let values: Vec<String> = row.columns().iter().map(|col| {
                                    let val = convert_pg_value_to_json(row, col);
                                    match val {
                                        serde_json::Value::Null => "".to_string(),
                                        serde_json::Value::String(s) => {
                                            if s.contains(',') || s.contains('"') || s.contains('\n') {
                                                format!("\"{}\"", s.replace('"', "\"\""))
                                            } else {
                                                s
                                            }
                                        }
                                        _ => val.to_string(),
                                    }
                                }).collect();
                                content.push_str(&values.join(","));
                                content.push('\n');
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"export.{}\"", request.format),
        )
        .body(Body::from(content))
        .unwrap()
}

/// Export query result
pub async fn export_query_result(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExportQueryRequest>,
) -> impl IntoResponse {
    let query = request.query.trim();
    let rows = match sqlx::query(query).fetch_all(&state.pool).await {
        Ok(r) => r,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(format!("Query error: {}", e)))
                .unwrap();
        }
    };

    let content = match request.format.as_str() {
        "csv" => {
            let mut output = String::new();
            if !rows.is_empty() {
                let headers: Vec<String> = rows[0].columns().iter()
                    .map(|c| c.name().to_string())
                    .collect();
                output.push_str(&headers.join(","));
                output.push('\n');

                for row in &rows {
                    let values: Vec<String> = row.columns().iter().map(|col| {
                        let val = convert_pg_value_to_json(row, col);
                        match val {
                            serde_json::Value::Null => "".to_string(),
                            serde_json::Value::String(s) => {
                                if s.contains(',') || s.contains('"') || s.contains('\n') {
                                    format!("\"{}\"", s.replace('"', "\"\""))
                                } else {
                                    s
                                }
                            }
                            _ => val.to_string(),
                        }
                    }).collect();
                    output.push_str(&values.join(","));
                    output.push('\n');
                }
            }
            output
        }
        "json" => {
            let data: Vec<serde_json::Value> = rows.iter().map(|row| {
                let mut obj = serde_json::Map::new();
                for col in row.columns() {
                    let val = convert_pg_value_to_json(row, col);
                    obj.insert(col.name().to_string(), val);
                }
                serde_json::Value::Object(obj)
            }).collect();
            serde_json::to_string_pretty(&data).unwrap_or_default()
        }
        "sql" => {
            let mut output = String::new();
            if !rows.is_empty() {
                let columns: Vec<String> = rows[0].columns().iter()
                    .map(|c| format!("\"{}\"", c.name()))
                    .collect();

                for row in &rows {
                    let values: Vec<String> = row.columns().iter().map(|col| {
                        let val = convert_pg_value_to_json(row, col);
                        match val {
                            serde_json::Value::Null => "NULL".to_string(),
                            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => format!("'{}'", val.to_string().replace('\'', "''")),
                        }
                    }).collect();

                    output.push_str(&format!(
                        "INSERT INTO table_name ({}) VALUES ({});\n",
                        columns.join(", "),
                        values.join(", ")
                    ));
                }
            }
            output
        }
        _ => "".to_string(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"query_result.{}\"", request.format),
        )
        .body(Body::from(content))
        .unwrap()
}

/// Import SQL file
pub async fn import_sql(
    State(state): State<Arc<DbManagerState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut result = ImportSqlResult {
        success: true,
        total_statements: 0,
        successful_statements: 0,
        failed_statements: 0,
        errors: None,
    };
    let mut errors = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            match field.bytes().await {
                Ok(data) => {
                    let content = String::from_utf8_lossy(&data);
                    let statements: Vec<&str> = content
                        .split(';')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    result.total_statements = statements.len() as u32;

                    for (i, statement) in statements.iter().enumerate() {
                        match sqlx::query(statement).execute(&state.pool).await {
                            Ok(_) => result.successful_statements += 1,
                            Err(e) => {
                                result.failed_statements += 1;
                                errors.push(format!("Statement {}: {}", i + 1, e));
                            }
                        }
                    }
                }
                Err(_) => {
                    result.success = false;
                    errors.push("Failed to read file content".to_string());
                }
            }
        }
    }

    if !errors.is_empty() {
        result.errors = Some(errors);
    }
    if result.failed_statements > 0 {
        result.success = false;
    }

    Json(ApiResponse {
        success: result.success,
        data: Some(result),
        error: None,
    })
}

/// Import CSV file
pub async fn import_csv(
    State(state): State<Arc<DbManagerState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut table_name = String::new();
    let mut csv_content = Vec::new();
    let mut result = ImportCsvResult {
        success: true,
        rows_imported: 0,
        errors: None,
    };
    let mut errors = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("file") => {
                match field.bytes().await {
                    Ok(data) => {
                        csv_content = data.to_vec();
                    }
                    Err(_) => {
                        result.success = false;
                        errors.push("Failed to read CSV file".to_string());
                    }
                }
            }
            Some("table") => {
                if let Ok(text) = field.text().await {
                    table_name = text;
                }
            }
            _ => {}
        }
    }

    if table_name.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: Some(ImportCsvResult {
                success: false,
                rows_imported: 0,
                errors: Some(vec!["Table name is required".to_string()]),
            }),
            error: None,
        });
    }

    if !is_valid_identifier(&table_name) {
        return Json(ApiResponse {
            success: false,
            data: Some(ImportCsvResult {
                success: false,
                rows_imported: 0,
                errors: Some(vec!["Invalid table name".to_string()]),
            }),
            error: None,
        });
    }

    if !csv_content.is_empty() && result.success {
        let content = String::from_utf8_lossy(&csv_content);
        let parser = CsvParser::new();

        match parser.parse(&content) {
            Ok((headers, rows)) => {
                // Validate headers are valid identifiers
                for header in &headers {
                    if !is_valid_identifier(header) {
                        errors.push(format!("Invalid column name: {}", header));
                    }
                }

                if errors.is_empty() {
                    let columns: Vec<String> = headers.iter().map(|h| format!("\"{}\"", h)).collect();
                    let placeholders: Vec<String> = (1..=headers.len()).map(|i| format!("${}", i)).collect();

                    let sql = format!(
                        "INSERT INTO \"{}\" ({}) VALUES ({})",
                        table_name,
                        columns.join(", "),
                        placeholders.join(", ")
                    );

                    for (i, row) in rows.iter().enumerate() {
                        let mut query = sqlx::query(&sql);

                        for value in &row.values {
                            // Handle NULL values and empty strings
                            match parser.parse_value(value) {
                                Some(v) => query = query.bind(v),
                                None => query = query.bind(None::<String>),
                            }
                        }

                        match query.execute(&state.pool).await {
                            Ok(_) => result.rows_imported += 1,
                            Err(e) => {
                                errors.push(format!("Row {}: {}", i + 2, e)); // +2 for 1-indexed and header row
                            }
                        }
                    }
                }
            }
            Err(parse_error) => {
                errors.push(format!("CSV parsing error: {}", parse_error));
            }
        }
    }

    if !errors.is_empty() {
        result.errors = Some(errors);
        result.success = false;
    }

    Json(ApiResponse {
        success: result.success,
        data: Some(result),
        error: None,
    })
}

// ============================================================================
// Router
// ============================================================================
// Handlers - Audit Log
// ============================================================================

/// Helper function to write an audit log entry
pub async fn write_audit_log(
    pool: &PgPool,
    event_type: &str,
    event_data: serde_json::Value,
    user_id: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"
        INSERT INTO dbmanager_audit_log (event_type, event_data, user_id, ip_address, user_agent, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
    "#)
    .bind(event_type)
    .bind(event_data)
    .bind(user_id)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get audit log entries
pub async fn get_audit_log(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(500);
    let offset = params.offset.unwrap_or(0);

    // Build query with optional filters
    let mut conditions = vec!["1=1".to_string()];
    let mut bind_idx = 1;

    if params.event_type.is_some() {
        conditions.push(format!("event_type = ${}", bind_idx));
        bind_idx += 1;
    }
    if params.user_id.is_some() {
        conditions.push(format!("user_id = ${}", bind_idx));
        bind_idx += 1;
    }
    if params.start_date.is_some() {
        conditions.push(format!("created_at >= ${}::timestamptz", bind_idx));
        bind_idx += 1;
    }
    if params.end_date.is_some() {
        conditions.push(format!("created_at <= ${}::timestamptz", bind_idx));
        #[allow(unused_assignments)]
        { bind_idx += 1; }
    }

    let where_clause = conditions.join(" AND ");

    // Get total count
    let count_query = format!(
        "SELECT COUNT(*) as count FROM dbmanager_audit_log WHERE {}",
        where_clause
    );

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
    if let Some(ref event_type) = params.event_type {
        count_q = count_q.bind(event_type);
    }
    if let Some(ref user_id) = params.user_id {
        count_q = count_q.bind(user_id);
    }
    if let Some(ref start_date) = params.start_date {
        count_q = count_q.bind(start_date);
    }
    if let Some(ref end_date) = params.end_date {
        count_q = count_q.bind(end_date);
    }

    let total = count_q.fetch_one(&state.pool).await.unwrap_or(0) as u64;

    // Get entries
    let data_query = format!(
        r#"
        SELECT id, event_type, event_data, user_id, ip_address, user_agent, created_at
        FROM dbmanager_audit_log
        WHERE {}
        ORDER BY created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    let mut data_q = sqlx::query(&data_query);
    if let Some(ref event_type) = params.event_type {
        data_q = data_q.bind(event_type);
    }
    if let Some(ref user_id) = params.user_id {
        data_q = data_q.bind(user_id);
    }
    if let Some(ref start_date) = params.start_date {
        data_q = data_q.bind(start_date);
    }
    if let Some(ref end_date) = params.end_date {
        data_q = data_q.bind(end_date);
    }

    match data_q.fetch_all(&state.pool).await {
        Ok(rows) => {
            let entries: Vec<AuditLogEntry> = rows.iter().map(|row| {
                AuditLogEntry {
                    id: row.get::<uuid::Uuid, _>("id").to_string(),
                    event_type: row.get("event_type"),
                    event_data: row.get("event_data"),
                    user_id: row.get("user_id"),
                    ip_address: row.get("ip_address"),
                    user_agent: row.get("user_agent"),
                    created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(AuditLogResponse {
                    entries,
                    total,
                    limit,
                    offset,
                }),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to fetch audit log: {}", e)),
            })
        }
    }
}

/// Clear old audit log entries (keeps last N days)
pub async fn clear_audit_log(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let days = params.get("days")
        .and_then(|d| d.parse::<i32>().ok())
        .unwrap_or(30);

    match sqlx::query(r#"
        DELETE FROM dbmanager_audit_log
        WHERE created_at < NOW() - INTERVAL '1 day' * $1
    "#)
    .bind(days)
    .execute(&state.pool)
    .await
    {
        Ok(result) => {
            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "deleted": result.rows_affected(),
                    "message": format!("Deleted entries older than {} days", days)
                })),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to clear audit log: {}", e)),
            })
        }
    }
}

// ============================================================================
// ENHANCEMENT 1: Query Snippets Library
// ============================================================================

/// Query snippet category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySnippet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub query: String,
    pub parameters: Vec<SnippetParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetParameter {
    pub name: String,
    pub param_type: String,
    pub default_value: Option<String>,
    pub description: String,
}

/// Get all query snippets organized by category
pub async fn get_query_snippets() -> impl IntoResponse {
    let snippets = vec![
        // SELECT snippets
        QuerySnippet {
            id: "select_all".to_string(),
            name: "Select All".to_string(),
            description: "Select all columns from a table".to_string(),
            category: "SELECT".to_string(),
            query: "SELECT * FROM :table_name LIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to query".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("100".to_string()), description: "Row limit".to_string() },
            ],
        },
        QuerySnippet {
            id: "select_columns".to_string(),
            name: "Select Specific Columns".to_string(),
            description: "Select specific columns from a table".to_string(),
            category: "SELECT".to_string(),
            query: "SELECT :columns FROM :table_name WHERE :condition LIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "columns".to_string(), param_type: "text".to_string(), default_value: Some("*".to_string()), description: "Columns to select".to_string() },
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to query".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: Some("1=1".to_string()), description: "WHERE condition".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("100".to_string()), description: "Row limit".to_string() },
            ],
        },
        QuerySnippet {
            id: "select_count".to_string(),
            name: "Count Rows".to_string(),
            description: "Count rows in a table".to_string(),
            category: "SELECT".to_string(),
            query: "SELECT COUNT(*) as total FROM :table_name WHERE :condition;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to count".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: Some("1=1".to_string()), description: "WHERE condition".to_string() },
            ],
        },
        QuerySnippet {
            id: "select_distinct".to_string(),
            name: "Select Distinct".to_string(),
            description: "Select distinct values from a column".to_string(),
            category: "SELECT".to_string(),
            query: "SELECT DISTINCT :column FROM :table_name ORDER BY :column;".to_string(),
            parameters: vec![
                SnippetParameter { name: "column".to_string(), param_type: "column".to_string(), default_value: None, description: "Column to get distinct values".to_string() },
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to query".to_string() },
            ],
        },
        // JOIN snippets
        QuerySnippet {
            id: "inner_join".to_string(),
            name: "Inner Join".to_string(),
            description: "Join two tables on matching columns".to_string(),
            category: "JOIN".to_string(),
            query: "SELECT t1.*, t2.*\nFROM :table1 t1\nINNER JOIN :table2 t2 ON t1.:column1 = t2.:column2\nLIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table1".to_string(), param_type: "table".to_string(), default_value: None, description: "First table".to_string() },
                SnippetParameter { name: "table2".to_string(), param_type: "table".to_string(), default_value: None, description: "Second table".to_string() },
                SnippetParameter { name: "column1".to_string(), param_type: "column".to_string(), default_value: None, description: "Join column from first table".to_string() },
                SnippetParameter { name: "column2".to_string(), param_type: "column".to_string(), default_value: None, description: "Join column from second table".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("100".to_string()), description: "Row limit".to_string() },
            ],
        },
        QuerySnippet {
            id: "left_join".to_string(),
            name: "Left Join".to_string(),
            description: "Left outer join two tables".to_string(),
            category: "JOIN".to_string(),
            query: "SELECT t1.*, t2.*\nFROM :table1 t1\nLEFT JOIN :table2 t2 ON t1.:column1 = t2.:column2\nLIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table1".to_string(), param_type: "table".to_string(), default_value: None, description: "First table".to_string() },
                SnippetParameter { name: "table2".to_string(), param_type: "table".to_string(), default_value: None, description: "Second table".to_string() },
                SnippetParameter { name: "column1".to_string(), param_type: "column".to_string(), default_value: None, description: "Join column from first table".to_string() },
                SnippetParameter { name: "column2".to_string(), param_type: "column".to_string(), default_value: None, description: "Join column from second table".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("100".to_string()), description: "Row limit".to_string() },
            ],
        },
        // AGGREGATE snippets
        QuerySnippet {
            id: "group_by_count".to_string(),
            name: "Group By Count".to_string(),
            description: "Count rows grouped by a column".to_string(),
            category: "AGGREGATE".to_string(),
            query: "SELECT :group_column, COUNT(*) as count\nFROM :table_name\nGROUP BY :group_column\nORDER BY count DESC\nLIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to query".to_string() },
                SnippetParameter { name: "group_column".to_string(), param_type: "column".to_string(), default_value: None, description: "Column to group by".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("100".to_string()), description: "Row limit".to_string() },
            ],
        },
        QuerySnippet {
            id: "sum_avg".to_string(),
            name: "Sum and Average".to_string(),
            description: "Calculate sum and average of a numeric column".to_string(),
            category: "AGGREGATE".to_string(),
            query: "SELECT \n  SUM(:numeric_column) as total,\n  AVG(:numeric_column) as average,\n  MIN(:numeric_column) as minimum,\n  MAX(:numeric_column) as maximum\nFROM :table_name\nWHERE :condition;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to query".to_string() },
                SnippetParameter { name: "numeric_column".to_string(), param_type: "column".to_string(), default_value: None, description: "Numeric column to aggregate".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: Some("1=1".to_string()), description: "WHERE condition".to_string() },
            ],
        },
        // INSERT snippets
        QuerySnippet {
            id: "insert_single".to_string(),
            name: "Insert Single Row".to_string(),
            description: "Insert a single row into a table".to_string(),
            category: "INSERT".to_string(),
            query: "INSERT INTO :table_name (:columns)\nVALUES (:values)\nRETURNING *;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to insert into".to_string() },
                SnippetParameter { name: "columns".to_string(), param_type: "text".to_string(), default_value: None, description: "Column names".to_string() },
                SnippetParameter { name: "values".to_string(), param_type: "text".to_string(), default_value: None, description: "Values to insert".to_string() },
            ],
        },
        QuerySnippet {
            id: "insert_select".to_string(),
            name: "Insert From Select".to_string(),
            description: "Insert rows from a SELECT query".to_string(),
            category: "INSERT".to_string(),
            query: "INSERT INTO :target_table (:columns)\nSELECT :select_columns\nFROM :source_table\nWHERE :condition;".to_string(),
            parameters: vec![
                SnippetParameter { name: "target_table".to_string(), param_type: "table".to_string(), default_value: None, description: "Target table".to_string() },
                SnippetParameter { name: "columns".to_string(), param_type: "text".to_string(), default_value: None, description: "Target columns".to_string() },
                SnippetParameter { name: "source_table".to_string(), param_type: "table".to_string(), default_value: None, description: "Source table".to_string() },
                SnippetParameter { name: "select_columns".to_string(), param_type: "text".to_string(), default_value: None, description: "Columns to select".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: Some("1=1".to_string()), description: "WHERE condition".to_string() },
            ],
        },
        // UPDATE snippets
        QuerySnippet {
            id: "update_single".to_string(),
            name: "Update Rows".to_string(),
            description: "Update rows matching a condition".to_string(),
            category: "UPDATE".to_string(),
            query: "UPDATE :table_name\nSET :column = :value\nWHERE :condition\nRETURNING *;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to update".to_string() },
                SnippetParameter { name: "column".to_string(), param_type: "column".to_string(), default_value: None, description: "Column to update".to_string() },
                SnippetParameter { name: "value".to_string(), param_type: "text".to_string(), default_value: None, description: "New value".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: None, description: "WHERE condition".to_string() },
            ],
        },
        // DELETE snippets
        QuerySnippet {
            id: "delete_rows".to_string(),
            name: "Delete Rows".to_string(),
            description: "Delete rows matching a condition".to_string(),
            category: "DELETE".to_string(),
            query: "DELETE FROM :table_name\nWHERE :condition\nRETURNING *;".to_string(),
            parameters: vec![
                SnippetParameter { name: "table_name".to_string(), param_type: "table".to_string(), default_value: None, description: "Table to delete from".to_string() },
                SnippetParameter { name: "condition".to_string(), param_type: "text".to_string(), default_value: None, description: "WHERE condition".to_string() },
            ],
        },
        // ANALYSIS snippets
        QuerySnippet {
            id: "table_size".to_string(),
            name: "Table Sizes".to_string(),
            description: "Get sizes of all tables".to_string(),
            category: "ANALYSIS".to_string(),
            query: "SELECT \n  schemaname,\n  tablename,\n  pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) as total_size,\n  pg_size_pretty(pg_relation_size(schemaname || '.' || tablename)) as table_size,\n  pg_size_pretty(pg_indexes_size(schemaname || '.' || tablename)) as index_size\nFROM pg_tables\nWHERE schemaname = :schema\nORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC;".to_string(),
            parameters: vec![
                SnippetParameter { name: "schema".to_string(), param_type: "text".to_string(), default_value: Some("'public'".to_string()), description: "Schema name".to_string() },
            ],
        },
        QuerySnippet {
            id: "index_usage".to_string(),
            name: "Index Usage Stats".to_string(),
            description: "Check index usage statistics".to_string(),
            category: "ANALYSIS".to_string(),
            query: "SELECT \n  schemaname,\n  tablename,\n  indexname,\n  idx_scan as scans,\n  idx_tup_read as tuples_read,\n  idx_tup_fetch as tuples_fetched\nFROM pg_stat_user_indexes\nWHERE schemaname = :schema\nORDER BY idx_scan DESC;".to_string(),
            parameters: vec![
                SnippetParameter { name: "schema".to_string(), param_type: "text".to_string(), default_value: Some("'public'".to_string()), description: "Schema name".to_string() },
            ],
        },
        QuerySnippet {
            id: "slow_queries".to_string(),
            name: "Slow Queries".to_string(),
            description: "Find slow running queries".to_string(),
            category: "ANALYSIS".to_string(),
            query: "SELECT \n  query,\n  calls,\n  total_exec_time / 1000 as total_seconds,\n  mean_exec_time / 1000 as avg_seconds,\n  rows\nFROM pg_stat_statements\nWHERE mean_exec_time > :threshold_ms\nORDER BY mean_exec_time DESC\nLIMIT :limit;".to_string(),
            parameters: vec![
                SnippetParameter { name: "threshold_ms".to_string(), param_type: "number".to_string(), default_value: Some("1000".to_string()), description: "Threshold in milliseconds".to_string() },
                SnippetParameter { name: "limit".to_string(), param_type: "number".to_string(), default_value: Some("20".to_string()), description: "Number of queries".to_string() },
            ],
        },
        QuerySnippet {
            id: "active_connections".to_string(),
            name: "Active Connections".to_string(),
            description: "View active database connections".to_string(),
            category: "ANALYSIS".to_string(),
            query: "SELECT \n  pid,\n  usename,\n  application_name,\n  client_addr,\n  state,\n  query,\n  query_start,\n  NOW() - query_start as duration\nFROM pg_stat_activity\nWHERE state != 'idle'\nORDER BY query_start;".to_string(),
            parameters: vec![],
        },
        QuerySnippet {
            id: "locks".to_string(),
            name: "View Locks".to_string(),
            description: "Check current database locks".to_string(),
            category: "ANALYSIS".to_string(),
            query: "SELECT \n  l.locktype,\n  l.relation::regclass,\n  l.mode,\n  l.granted,\n  a.usename,\n  a.query,\n  a.query_start\nFROM pg_locks l\nJOIN pg_stat_activity a ON l.pid = a.pid\nWHERE l.relation IS NOT NULL\nORDER BY l.relation;".to_string(),
            parameters: vec![],
        },
    ];

    Json(ApiResponse {
        success: true,
        data: Some(snippets),
        error: None,
    })
}

/// Request to save a query snippet
#[derive(Debug, Deserialize)]
pub struct SaveSnippetRequest {
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub category: Option<String>,
    pub parameters: Option<Vec<SnippetParameter>>,
    pub tags: Option<Vec<String>>,
}

/// Save a new query snippet
pub async fn save_query_snippet(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<SaveSnippetRequest>,
) -> impl IntoResponse {
    let params_json = serde_json::to_value(request.parameters.unwrap_or_default()).unwrap_or(serde_json::json!([]));
    let tags = request.tags.unwrap_or_default();

    match sqlx::query(r#"
        INSERT INTO dbmanager_query_snippets (name, description, query, category, parameters, tags)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, description, query, category, parameters, tags, usage_count, created_at
    "#)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.query)
    .bind(&request.category)
    .bind(&params_json)
    .bind(&tags)
    .fetch_one(&state.pool)
    .await
    {
        Ok(row) => {
            let snippet = serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id").to_string(),
                "name": row.get::<String, _>("name"),
                "description": row.get::<Option<String>, _>("description"),
                "query": row.get::<String, _>("query"),
                "category": row.get::<Option<String>, _>("category"),
                "parameters": row.get::<serde_json::Value, _>("parameters"),
                "tags": row.get::<Vec<String>, _>("tags"),
                "usageCount": row.get::<i32, _>("usage_count"),
                "createdAt": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339()
            });
            Json(ApiResponse {
                success: true,
                data: Some(snippet),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to save snippet: {}", e)),
        }),
    }
}

/// Request to update a query snippet
#[derive(Debug, Deserialize)]
pub struct UpdateSnippetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub category: Option<String>,
    pub parameters: Option<Vec<SnippetParameter>>,
    pub tags: Option<Vec<String>>,
}

/// Update an existing query snippet
pub async fn update_query_snippet(
    State(state): State<Arc<DbManagerState>>,
    Path(snippet_id): Path<String>,
    Json(request): Json<UpdateSnippetRequest>,
) -> impl IntoResponse {
    let snippet_uuid = match uuid::Uuid::parse_str(&snippet_id) {
        Ok(id) => id,
        Err(_) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid snippet ID".to_string()),
            });
        }
    };

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut param_idx = 1;

    if request.name.is_some() {
        updates.push(format!("name = ${}", param_idx));
        param_idx += 1;
    }
    if request.description.is_some() {
        updates.push(format!("description = ${}", param_idx));
        param_idx += 1;
    }
    if request.query.is_some() {
        updates.push(format!("query = ${}", param_idx));
        param_idx += 1;
    }
    if request.category.is_some() {
        updates.push(format!("category = ${}", param_idx));
        param_idx += 1;
    }
    if request.parameters.is_some() {
        updates.push(format!("parameters = ${}", param_idx));
        param_idx += 1;
    }
    if request.tags.is_some() {
        updates.push(format!("tags = ${}", param_idx));
        param_idx += 1;
    }

    if updates.is_empty() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("No fields to update".to_string()),
        });
    }

    updates.push("updated_at = NOW()".to_string());
    let query = format!(
        "UPDATE dbmanager_query_snippets SET {} WHERE id = ${} RETURNING id",
        updates.join(", "),
        param_idx
    );

    let mut q = sqlx::query(&query);

    if let Some(ref name) = request.name {
        q = q.bind(name);
    }
    if let Some(ref desc) = request.description {
        q = q.bind(desc);
    }
    if let Some(ref query_str) = request.query {
        q = q.bind(query_str);
    }
    if let Some(ref cat) = request.category {
        q = q.bind(cat);
    }
    if let Some(ref params) = request.parameters {
        q = q.bind(serde_json::to_value(params).unwrap_or(serde_json::json!([])));
    }
    if let Some(ref tags) = request.tags {
        q = q.bind(tags);
    }
    q = q.bind(snippet_uuid);

    match q.fetch_one(&state.pool).await {
        Ok(_) => Json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({ "message": "Snippet updated" })),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to update snippet: {}", e)),
        }),
    }
}

/// Delete a query snippet
pub async fn delete_query_snippet(
    State(state): State<Arc<DbManagerState>>,
    Path(snippet_id): Path<String>,
) -> impl IntoResponse {
    let snippet_uuid = match uuid::Uuid::parse_str(&snippet_id) {
        Ok(id) => id,
        Err(_) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Invalid snippet ID".to_string()),
            });
        }
    };

    match sqlx::query("DELETE FROM dbmanager_query_snippets WHERE id = $1")
        .bind(snippet_uuid)
        .execute(&state.pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Json(ApiResponse {
                    success: true,
                    data: Some(serde_json::json!({ "message": "Snippet deleted" })),
                    error: None,
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Snippet not found".to_string()),
                })
            }
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to delete snippet: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 2: Query Parameterization
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ParameterizedQueryRequest {
    pub query: String,
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
}

/// Execute a parameterized query with named parameters
pub async fn execute_parameterized_query(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ParameterizedQueryRequest>,
) -> impl IntoResponse {
    // Replace named parameters with positional parameters
    let mut processed_query = request.query.clone();
    let mut param_values: Vec<String> = Vec::new();
    let mut param_idx = 1;

    // Sort parameters by name length (longest first) to avoid partial replacements
    let mut sorted_params: Vec<_> = request.parameters.iter().collect();
    sorted_params.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (name, value) in sorted_params {
        let placeholder = format!(":{}", name);
        if processed_query.contains(&placeholder) {
            processed_query = processed_query.replace(&placeholder, &format!("${}", param_idx));
            param_values.push(json_value_to_sql(value));
            param_idx += 1;
        }
    }

    // Check for blocked commands
    if let Some(blocked) = is_blocked_query(&processed_query) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Blocked command detected: {}", blocked)),
        });
    }

    let start = std::time::Instant::now();

    // Build and execute the query with bindings
    let mut query = sqlx::query(&processed_query);
    for value in &param_values {
        query = query.bind(value);
    }

    match query.fetch_all(&state.pool).await {
        Ok(rows) => {
            let execution_time = start.elapsed().as_secs_f64() * 1000.0;
            let row_count = rows.len();

            // Convert rows to JSON
            let data: Vec<HashMap<String, serde_json::Value>> = rows
                .iter()
                .map(row_to_json)
                .collect();

            // Get columns
            let columns: Vec<String> = if !rows.is_empty() {
                rows[0].columns().iter().map(|c| c.name().to_string()).collect()
            } else {
                vec![]
            };

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "columns": columns,
                    "rows": data,
                    "rowCount": row_count,
                    "executionTime": execution_time,
                    "processedQuery": processed_query,
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Query execution failed: {}", e)),
        }),
    }
}

fn json_value_to_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "NULL".to_string(),
        _ => value.to_string(),
    }
}

// ============================================================================
// ENHANCEMENT 3: Multi-Statement Execution
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MultiStatementRequest {
    pub statements: String,
    #[serde(default)]
    pub stop_on_error: bool,
}

#[derive(Debug, Serialize)]
pub struct StatementResult {
    pub statement: String,
    pub success: bool,
    pub row_count: Option<u64>,
    pub execution_time: f64,
    pub error: Option<String>,
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
}

/// Execute multiple SQL statements
pub async fn execute_multi_statement(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<MultiStatementRequest>,
) -> impl IntoResponse {
    let statements: Vec<&str> = request.statements
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut results: Vec<StatementResult> = Vec::new();

    for statement in statements {
        // Check for blocked commands
        if let Some(blocked) = is_blocked_query(statement) {
            results.push(StatementResult {
                statement: statement.to_string(),
                success: false,
                row_count: None,
                execution_time: 0.0,
                error: Some(format!("Blocked command: {}", blocked)),
                columns: vec![],
                rows: vec![],
            });
            if request.stop_on_error {
                break;
            }
            continue;
        }

        let start = std::time::Instant::now();
        let query_upper = statement.to_uppercase();

        // Execute as query or command based on statement type
        if query_upper.starts_with("SELECT") || query_upper.starts_with("WITH") || query_upper.contains("RETURNING") {
            match sqlx::query(statement).fetch_all(&state.pool).await {
                Ok(rows) => {
                    let execution_time = start.elapsed().as_secs_f64() * 1000.0;
                    let columns: Vec<String> = if !rows.is_empty() {
                        rows[0].columns().iter().map(|c| c.name().to_string()).collect()
                    } else {
                        vec![]
                    };
                    let data: Vec<HashMap<String, serde_json::Value>> = rows.iter().map(row_to_json).collect();

                    results.push(StatementResult {
                        statement: statement.to_string(),
                        success: true,
                        row_count: Some(data.len() as u64),
                        execution_time,
                        error: None,
                        columns,
                        rows: data,
                    });
                }
                Err(e) => {
                    results.push(StatementResult {
                        statement: statement.to_string(),
                        success: false,
                        row_count: None,
                        execution_time: start.elapsed().as_secs_f64() * 1000.0,
                        error: Some(e.to_string()),
                        columns: vec![],
                        rows: vec![],
                    });
                    if request.stop_on_error {
                        break;
                    }
                }
            }
        } else {
            match sqlx::query(statement).execute(&state.pool).await {
                Ok(result) => {
                    results.push(StatementResult {
                        statement: statement.to_string(),
                        success: true,
                        row_count: Some(result.rows_affected()),
                        execution_time: start.elapsed().as_secs_f64() * 1000.0,
                        error: None,
                        columns: vec![],
                        rows: vec![],
                    });
                }
                Err(e) => {
                    results.push(StatementResult {
                        statement: statement.to_string(),
                        success: false,
                        row_count: None,
                        execution_time: start.elapsed().as_secs_f64() * 1000.0,
                        error: Some(e.to_string()),
                        columns: vec![],
                        rows: vec![],
                    });
                    if request.stop_on_error {
                        break;
                    }
                }
            }
        }
    }

    let total_success = results.iter().filter(|r| r.success).count();
    let total_failed = results.iter().filter(|r| !r.success).count();

    Json(ApiResponse {
        success: total_failed == 0,
        data: Some(serde_json::json!({
            "results": results,
            "summary": {
                "total": results.len(),
                "success": total_success,
                "failed": total_failed,
            }
        })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 4: Query Timeout Configuration
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct QueryWithTimeoutRequest {
    pub query: String,
    pub timeout_ms: u32,
}

/// Execute query with timeout
pub async fn execute_query_with_timeout(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<QueryWithTimeoutRequest>,
) -> impl IntoResponse {
    let timeout = std::time::Duration::from_millis(request.timeout_ms as u64);

    // Set statement timeout
    let set_timeout = format!("SET statement_timeout = {}", request.timeout_ms);
    if let Err(e) = sqlx::query(&set_timeout).execute(&state.pool).await {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to set timeout: {}", e)),
        });
    }

    let start = std::time::Instant::now();
    let result = tokio::time::timeout(
        timeout,
        sqlx::query(&request.query).fetch_all(&state.pool)
    ).await;

    // Reset timeout
    let _ = sqlx::query("SET statement_timeout = 0").execute(&state.pool).await;

    match result {
        Ok(Ok(rows)) => {
            let execution_time = start.elapsed().as_secs_f64() * 1000.0;
            let columns: Vec<String> = if !rows.is_empty() {
                rows[0].columns().iter().map(|c| c.name().to_string()).collect()
            } else {
                vec![]
            };
            let data: Vec<HashMap<String, serde_json::Value>> = rows.iter().map(row_to_json).collect();

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "columns": columns,
                    "rows": data,
                    "rowCount": data.len(),
                    "executionTime": execution_time,
                })),
                error: None,
            })
        }
        Ok(Err(e)) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Query failed: {}", e)),
        }),
        Err(_) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Query timed out after {} ms", request.timeout_ms)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 5: Query Cost Estimation
// ============================================================================

#[derive(Debug, Serialize)]
pub struct QueryCostEstimate {
    pub startup_cost: f64,
    pub total_cost: f64,
    pub estimated_rows: u64,
    pub estimated_width: u32,
    pub plan_type: String,
    pub plan_details: serde_json::Value,
    pub warnings: Vec<String>,
}

/// Estimate query cost without executing
pub async fn estimate_query_cost(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExecuteQueryRequest>,
) -> impl IntoResponse {
    let explain_query = format!("EXPLAIN (FORMAT JSON) {}", request.query);

    match sqlx::query_scalar::<_, String>(&explain_query)
        .fetch_one(&state.pool)
        .await
    {
        Ok(plan_json) => {
            if let Ok(plan) = serde_json::from_str::<serde_json::Value>(&plan_json) {
                let plan_node = &plan[0]["Plan"];

                let startup_cost = plan_node["Startup Cost"].as_f64().unwrap_or(0.0);
                let total_cost = plan_node["Total Cost"].as_f64().unwrap_or(0.0);
                let estimated_rows = plan_node["Plan Rows"].as_u64().unwrap_or(0);
                let estimated_width = plan_node["Plan Width"].as_u64().unwrap_or(0) as u32;
                let plan_type = plan_node["Node Type"].as_str().unwrap_or("Unknown").to_string();

                // Generate warnings based on plan analysis
                let mut warnings = Vec::new();

                if total_cost > 10000.0 {
                    warnings.push("High cost query - consider optimization".to_string());
                }
                if estimated_rows > 100000 {
                    warnings.push("Large result set expected - consider adding LIMIT".to_string());
                }
                if plan_type == "Seq Scan" && estimated_rows > 1000 {
                    warnings.push("Sequential scan detected - consider adding an index".to_string());
                }

                // Check for nested loops with high row counts
                fn check_nested_loops(node: &serde_json::Value, warnings: &mut Vec<String>) {
                    if let Some(node_type) = node["Node Type"].as_str() {
                        if node_type == "Nested Loop" {
                            if let Some(rows) = node["Plan Rows"].as_u64() {
                                if rows > 10000 {
                                    warnings.push("Nested loop with high row count - may be slow".to_string());
                                }
                            }
                        }
                    }
                    if let Some(plans) = node["Plans"].as_array() {
                        for plan in plans {
                            check_nested_loops(plan, warnings);
                        }
                    }
                }
                check_nested_loops(plan_node, &mut warnings);

                Json(ApiResponse {
                    success: true,
                    data: Some(serde_json::json!(QueryCostEstimate {
                        startup_cost,
                        total_cost,
                        estimated_rows,
                        estimated_width,
                        plan_type,
                        plan_details: plan,
                        warnings,
                    })),
                    error: None,
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Failed to parse explain output".to_string()),
                })
            }
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to estimate cost: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 12: Excel Export
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ExcelExportRequest {
    pub query: String,
    pub sheet_name: Option<String>,
    pub include_headers: Option<bool>,
}

/// Export query results to Excel format (simple XML spreadsheet)
pub async fn export_to_excel(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExcelExportRequest>,
) -> impl IntoResponse {
    match sqlx::query(&request.query).fetch_all(&state.pool).await {
        Ok(rows) => {
            let sheet_name = request.sheet_name.unwrap_or_else(|| "Query Results".to_string());
            let include_headers = request.include_headers.unwrap_or(true);

            // Get columns
            let columns: Vec<String> = if !rows.is_empty() {
                rows[0].columns().iter().map(|c| c.name().to_string()).collect()
            } else {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
                    .header(header::CONTENT_DISPOSITION, "attachment; filename=\"export.xlsx\"")
                    .body(Body::empty())
                    .unwrap();
            };

            // Build simple XML spreadsheet format
            let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<?mso-application progid="Excel.Sheet"?>
<Workbook xmlns="urn:schemas-microsoft-com:office:spreadsheet"
 xmlns:ss="urn:schemas-microsoft-com:office:spreadsheet">
<Worksheet ss:Name=""#);
            xml.push_str(&sheet_name);
            xml.push_str(r#""><Table>"#);

            // Add header row
            if include_headers {
                xml.push_str("<Row>");
                for col in &columns {
                    xml.push_str(&format!(r#"<Cell><Data ss:Type="String">{}</Data></Cell>"#,
                        escape_xml(col)));
                }
                xml.push_str("</Row>");
            }

            // Add data rows
            for row in &rows {
                xml.push_str("<Row>");
                for col in &columns {
                    let value = get_column_value_as_string(row, col);
                    let data_type = if value.parse::<f64>().is_ok() { "Number" } else { "String" };
                    xml.push_str(&format!(r#"<Cell><Data ss:Type="{}">{}</Data></Cell>"#,
                        data_type, escape_xml(&value)));
                }
                xml.push_str("</Row>");
            }

            xml.push_str("</Table></Worksheet></Workbook>");

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/vnd.ms-excel")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"export.xls\"")
                .body(Body::from(xml))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Query failed: {}", e)))
            .unwrap()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ============================================================================
// ENHANCEMENT 13: SQL INSERT Export
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct InsertExportRequest {
    pub query: String,
    pub target_table: String,
    pub include_columns: Option<bool>,
}

/// Export query results as INSERT statements
pub async fn export_as_insert(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<InsertExportRequest>,
) -> impl IntoResponse {
    match sqlx::query(&request.query).fetch_all(&state.pool).await {
        Ok(rows) => {
            if rows.is_empty() {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("-- No data to export"))
                    .unwrap();
            }

            let columns: Vec<String> = rows[0].columns().iter()
                .map(|c| c.name().to_string())
                .collect();

            let include_columns = request.include_columns.unwrap_or(true);
            let mut output = String::new();

            output.push_str(&format!("-- Generated INSERT statements for table: {}\n", request.target_table));
            output.push_str(&format!("-- Total rows: {}\n\n", rows.len()));

            for row in &rows {
                if include_columns {
                    output.push_str(&format!("INSERT INTO \"{}\" (", request.target_table));
                    output.push_str(&columns.iter()
                        .map(|c| format!("\"{}\"", c))
                        .collect::<Vec<_>>()
                        .join(", "));
                    output.push_str(") VALUES (");
                } else {
                    output.push_str(&format!("INSERT INTO \"{}\" VALUES (", request.target_table));
                }

                let values: Vec<String> = columns.iter()
                    .map(|col| format_value_for_insert(row, col))
                    .collect();

                output.push_str(&values.join(", "));
                output.push_str(");\n");
            }

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"insert_statements.sql\"")
                .body(Body::from(output))
                .unwrap()
        }
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Query failed: {}", e)))
            .unwrap()
    }
}

fn format_value_for_insert(row: &sqlx::postgres::PgRow, column: &str) -> String {
    let value = get_column_value_as_string(row, column);
    if value == "NULL" {
        return "NULL".to_string();
    }
    // Try to detect if it's a number
    if value.parse::<f64>().is_ok() {
        value
    } else if value == "true" || value == "false" {
        value.to_uppercase()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

// ============================================================================
// ENHANCEMENT 16: Table Cloning
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CloneTableRequest {
    pub source_table: String,
    pub target_table: String,
    pub include_data: bool,
    pub include_indexes: bool,
    pub include_constraints: bool,
}

/// Clone a table with optional data and indexes
pub async fn clone_table(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<CloneTableRequest>,
) -> impl IntoResponse {
    // Validate table names
    if !is_valid_identifier(&request.source_table) || !is_valid_identifier(&request.target_table) {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Invalid table name".to_string()),
        });
    }

    // Check if source table exists
    let check_query = "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = $1)";
    let exists: bool = sqlx::query_scalar(check_query)
        .bind(&request.source_table)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);

    if !exists {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Source table does not exist".to_string()),
        });
    }

    // Create table structure
    let create_query = if request.include_data {
        format!(
            "CREATE TABLE \"{}\" AS SELECT * FROM \"{}\"",
            request.target_table, request.source_table
        )
    } else {
        format!(
            "CREATE TABLE \"{}\" AS SELECT * FROM \"{}\" WHERE 1=0",
            request.target_table, request.source_table
        )
    };

    if let Err(e) = sqlx::query(&create_query).execute(&state.pool).await {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to create table: {}", e)),
        });
    }

    let mut messages = vec![format!("Table '{}' created", request.target_table)];

    // Clone indexes if requested
    if request.include_indexes {
        let index_query = r#"
            SELECT indexdef
            FROM pg_indexes
            WHERE tablename = $1 AND indexname NOT LIKE '%_pkey'
        "#;

        if let Ok(indexes) = sqlx::query_scalar::<_, String>(index_query)
            .bind(&request.source_table)
            .fetch_all(&state.pool)
            .await
        {
            for indexdef in indexes {
                let new_indexdef = indexdef.replace(
                    &format!("ON \"{}\"", request.source_table),
                    &format!("ON \"{}\"", request.target_table)
                ).replace(
                    &request.source_table,
                    &format!("{}_clone", request.target_table)
                );

                if let Err(e) = sqlx::query(&new_indexdef).execute(&state.pool).await {
                    messages.push(format!("Warning: Failed to create index: {}", e));
                }
            }
            messages.push("Indexes cloned".to_string());
        }
    }

    // Add primary key constraint if source has one and constraints are requested
    if request.include_constraints {
        let pk_query = r#"
            SELECT a.attname
            FROM pg_index i
            JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
            WHERE i.indrelid = $1::regclass AND i.indisprimary
        "#;

        if let Ok(pk_columns) = sqlx::query_scalar::<_, String>(pk_query)
            .bind(&request.source_table)
            .fetch_all(&state.pool)
            .await
        {
            if !pk_columns.is_empty() {
                let pk_sql = format!(
                    "ALTER TABLE \"{}\" ADD PRIMARY KEY ({})",
                    request.target_table,
                    pk_columns.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", ")
                );

                if let Err(e) = sqlx::query(&pk_sql).execute(&state.pool).await {
                    messages.push(format!("Warning: Failed to add primary key: {}", e));
                } else {
                    messages.push("Primary key constraint added".to_string());
                }
            }
        }
    }

    // Get row count
    let count_query = format!("SELECT COUNT(*) FROM \"{}\"", request.target_table);
    let row_count: i64 = sqlx::query_scalar(&count_query)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "table": request.target_table,
            "rowCount": row_count,
            "messages": messages,
        })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 17: Table Partitioning Wizard
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PartitionInfo {
    pub partition_name: String,
    pub parent_table: String,
    pub partition_expression: String,
    pub partition_bounds: String,
    pub row_count: i64,
    pub size: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePartitionRequest {
    pub table_name: String,
    pub partition_type: String, // RANGE, LIST, HASH
    pub partition_column: String,
    pub partitions: Vec<PartitionDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct PartitionDefinition {
    pub name: String,
    pub values: Option<Vec<String>>, // For LIST
    pub from_value: Option<String>,  // For RANGE
    pub to_value: Option<String>,    // For RANGE
    pub modulus: Option<u32>,        // For HASH
    pub remainder: Option<u32>,      // For HASH
}

/// Get partition information for a table
pub async fn get_table_partitions(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let query = r#"
        SELECT
            c.relname as partition_name,
            parent.relname as parent_table,
            pg_get_partkeydef(parent.oid) as partition_expression,
            pg_get_expr(c.relpartbound, c.oid) as partition_bounds,
            (SELECT COUNT(*) FROM ONLY c.relname::regclass) as row_count,
            pg_size_pretty(pg_relation_size(c.oid)) as size
        FROM pg_class c
        JOIN pg_inherits i ON c.oid = i.inhrelid
        JOIN pg_class parent ON i.inhparent = parent.oid
        WHERE parent.relname = $1
        ORDER BY c.relname
    "#;

    match sqlx::query(query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => {
            let partitions: Vec<PartitionInfo> = rows.iter().map(|row| {
                PartitionInfo {
                    partition_name: row.get("partition_name"),
                    parent_table: row.get("parent_table"),
                    partition_expression: row.get::<Option<String>, _>("partition_expression").unwrap_or_default(),
                    partition_bounds: row.get::<Option<String>, _>("partition_bounds").unwrap_or_default(),
                    row_count: row.get::<Option<i64>, _>("row_count").unwrap_or(0),
                    size: row.get::<Option<String>, _>("size").unwrap_or_else(|| "0 bytes".to_string()),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({ "partitions": partitions })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get partitions: {}", e)),
        }),
    }
}

/// Create partitions for a table
pub async fn create_partitions(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<CreatePartitionRequest>,
) -> impl IntoResponse {
    let mut results = Vec::new();

    for partition in &request.partitions {
        let partition_sql = match request.partition_type.to_uppercase().as_str() {
            "RANGE" => {
                if let (Some(from), Some(to)) = (&partition.from_value, &partition.to_value) {
                    format!(
                        "CREATE TABLE \"{}\" PARTITION OF \"{}\" FOR VALUES FROM ({}) TO ({})",
                        partition.name, request.table_name, from, to
                    )
                } else {
                    results.push(format!("Skipped {}: missing FROM/TO values", partition.name));
                    continue;
                }
            }
            "LIST" => {
                if let Some(values) = &partition.values {
                    format!(
                        "CREATE TABLE \"{}\" PARTITION OF \"{}\" FOR VALUES IN ({})",
                        partition.name, request.table_name, values.join(", ")
                    )
                } else {
                    results.push(format!("Skipped {}: missing values", partition.name));
                    continue;
                }
            }
            "HASH" => {
                if let (Some(modulus), Some(remainder)) = (partition.modulus, partition.remainder) {
                    format!(
                        "CREATE TABLE \"{}\" PARTITION OF \"{}\" FOR VALUES WITH (MODULUS {}, REMAINDER {})",
                        partition.name, request.table_name, modulus, remainder
                    )
                } else {
                    results.push(format!("Skipped {}: missing modulus/remainder", partition.name));
                    continue;
                }
            }
            _ => {
                results.push(format!("Unknown partition type: {}", request.partition_type));
                continue;
            }
        };

        match sqlx::query(&partition_sql).execute(&state.pool).await {
            Ok(_) => results.push(format!("Created partition: {}", partition.name)),
            Err(e) => results.push(format!("Failed to create {}: {}", partition.name, e)),
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "results": results })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 18: Table Statistics
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TableStatistics {
    pub table_name: String,
    pub total_rows: i64,
    pub live_rows: i64,
    pub dead_rows: i64,
    pub table_size: String,
    pub index_size: String,
    pub total_size: String,
    pub bloat_ratio: f64,
    pub last_vacuum: Option<String>,
    pub last_autovacuum: Option<String>,
    pub last_analyze: Option<String>,
    pub last_autoanalyze: Option<String>,
    pub seq_scan: i64,
    pub seq_tup_read: i64,
    pub idx_scan: i64,
    pub idx_tup_fetch: i64,
    pub n_tup_ins: i64,
    pub n_tup_upd: i64,
    pub n_tup_del: i64,
}

/// Get detailed table statistics
pub async fn get_table_statistics(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let query = r#"
        SELECT
            s.relname as table_name,
            s.n_live_tup as live_rows,
            s.n_dead_tup as dead_rows,
            pg_size_pretty(pg_relation_size(s.relid)) as table_size,
            pg_size_pretty(pg_indexes_size(s.relid)) as index_size,
            pg_size_pretty(pg_total_relation_size(s.relid)) as total_size,
            CASE WHEN s.n_live_tup > 0
                THEN ROUND((s.n_dead_tup::numeric / s.n_live_tup::numeric) * 100, 2)
                ELSE 0
            END as bloat_ratio,
            s.last_vacuum::text,
            s.last_autovacuum::text,
            s.last_analyze::text,
            s.last_autoanalyze::text,
            s.seq_scan,
            s.seq_tup_read,
            s.idx_scan,
            s.idx_tup_fetch,
            s.n_tup_ins,
            s.n_tup_upd,
            s.n_tup_del
        FROM pg_stat_user_tables s
        WHERE s.relname = $1
    "#;

    match sqlx::query(query)
        .bind(&table_name)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(row)) => {
            // Get total row count
            let count_query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
            let total_rows: i64 = sqlx::query_scalar(&count_query)
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);

            let stats = TableStatistics {
                table_name: row.get("table_name"),
                total_rows,
                live_rows: row.get("live_rows"),
                dead_rows: row.get("dead_rows"),
                table_size: row.get("table_size"),
                index_size: row.get("index_size"),
                total_size: row.get("total_size"),
                bloat_ratio: row.try_get::<f64, _>("bloat_ratio").unwrap_or(0.0),
                last_vacuum: row.get("last_vacuum"),
                last_autovacuum: row.get("last_autovacuum"),
                last_analyze: row.get("last_analyze"),
                last_autoanalyze: row.get("last_autoanalyze"),
                seq_scan: row.get("seq_scan"),
                seq_tup_read: row.get("seq_tup_read"),
                idx_scan: row.get::<Option<i64>, _>("idx_scan").unwrap_or(0),
                idx_tup_fetch: row.get::<Option<i64>, _>("idx_tup_fetch").unwrap_or(0),
                n_tup_ins: row.get("n_tup_ins"),
                n_tup_upd: row.get("n_tup_upd"),
                n_tup_del: row.get("n_tup_del"),
            };

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::to_value(stats).unwrap()),
                error: None,
            })
        }
        Ok(None) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Table not found".to_string()),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get statistics: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 19: Index Recommendations
// ============================================================================

#[derive(Debug, Serialize)]
pub struct IndexRecommendation {
    pub table_name: String,
    pub column_name: String,
    pub recommendation: String,
    pub reason: String,
    pub estimated_improvement: String,
    pub create_statement: String,
}

/// Get index recommendations for a table
pub async fn get_index_recommendations(
    State(state): State<Arc<DbManagerState>>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let mut recommendations = Vec::new();

    // Check for columns frequently used in WHERE clauses but without indexes
    let seq_scan_query = r#"
        SELECT
            s.relname as table_name,
            s.seq_scan,
            s.seq_tup_read,
            s.idx_scan,
            pg_relation_size(s.relid) as size_bytes
        FROM pg_stat_user_tables s
        WHERE s.relname = $1
    "#;

    if let Ok(Some(row)) = sqlx::query(seq_scan_query)
        .bind(&table_name)
        .fetch_optional(&state.pool)
        .await
    {
        let seq_scan: i64 = row.get("seq_scan");
        let idx_scan: Option<i64> = row.get("idx_scan");
        let size_bytes: i64 = row.get("size_bytes");

        // If many sequential scans and table is large, suggest indexes
        if seq_scan > 1000 && idx_scan.unwrap_or(0) < seq_scan / 2 && size_bytes > 10_000_000 {
            recommendations.push(IndexRecommendation {
                table_name: table_name.clone(),
                column_name: String::new(),
                recommendation: "General".to_string(),
                reason: format!(
                    "High sequential scan count ({}) vs index scans ({}) on large table",
                    seq_scan, idx_scan.unwrap_or(0)
                ),
                estimated_improvement: "Significant".to_string(),
                create_statement: "-- Analyze query patterns to determine which columns to index".to_string(),
            });
        }
    }

    // Check for foreign key columns without indexes
    let fk_query = r#"
        SELECT
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage AS ccu
            ON ccu.constraint_name = tc.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = $1
    "#;

    if let Ok(fk_rows) = sqlx::query(fk_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await
    {
        for fk_row in fk_rows {
            let column: String = fk_row.get("column_name");
            let foreign_table: String = fk_row.get("foreign_table_name");

            // Check if this FK column has an index
            let index_check = r#"
                SELECT COUNT(*) FROM pg_indexes
                WHERE tablename = $1 AND indexdef LIKE '%' || $2 || '%'
            "#;

            let has_index: i64 = sqlx::query_scalar(index_check)
                .bind(&table_name)
                .bind(&column)
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);

            if has_index == 0 {
                recommendations.push(IndexRecommendation {
                    table_name: table_name.clone(),
                    column_name: column.clone(),
                    recommendation: "Foreign Key Index".to_string(),
                    reason: format!("Foreign key to {} lacks index - will slow JOINs", foreign_table),
                    estimated_improvement: "High".to_string(),
                    create_statement: format!(
                        "CREATE INDEX idx_{}_{} ON \"{}\"(\"{}\");",
                        table_name, column, table_name, column
                    ),
                });
            }
        }
    }

    // Check for columns with high cardinality that might benefit from indexing
    let columns_query = r#"
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name = $1
        AND data_type IN ('integer', 'bigint', 'uuid', 'character varying', 'text', 'timestamp with time zone', 'date')
    "#;

    if let Ok(columns) = sqlx::query(columns_query)
        .bind(&table_name)
        .fetch_all(&state.pool)
        .await
    {
        for col_row in columns {
            let column: String = col_row.get("column_name");
            let data_type: String = col_row.get("data_type");

            // Check distinct ratio
            let distinct_query = format!(
                "SELECT COUNT(DISTINCT \"{}\")::float / GREATEST(COUNT(*)::float, 1) as ratio FROM \"{}\"",
                column, table_name
            );

            if let Ok(ratio) = sqlx::query_scalar::<_, f64>(&distinct_query)
                .fetch_one(&state.pool)
                .await
            {
                // High cardinality columns (>50% unique) might benefit from B-tree index
                if ratio > 0.5 {
                    // Check if already indexed
                    let index_check = r#"
                        SELECT COUNT(*) FROM pg_indexes
                        WHERE tablename = $1 AND indexdef LIKE '%' || $2 || '%'
                    "#;

                    let has_index: i64 = sqlx::query_scalar(index_check)
                        .bind(&table_name)
                        .bind(&column)
                        .fetch_one(&state.pool)
                        .await
                        .unwrap_or(1);

                    if has_index == 0 && !column.contains("id") && column != "created_at" && column != "updated_at" {
                        recommendations.push(IndexRecommendation {
                            table_name: table_name.clone(),
                            column_name: column.clone(),
                            recommendation: "High Cardinality Index".to_string(),
                            reason: format!(
                                "Column '{}' ({}) has {}% unique values - good candidate for B-tree index",
                                column, data_type, (ratio * 100.0) as i32
                            ),
                            estimated_improvement: "Medium".to_string(),
                            create_statement: format!(
                                "CREATE INDEX idx_{}_{} ON \"{}\"(\"{}\");",
                                table_name, column, table_name, column
                            ),
                        });
                    }
                }
            }
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({ "recommendations": recommendations })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 20: Foreign Key Visualization
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TableRelationship {
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
    pub constraint_name: String,
    pub on_delete: String,
    pub on_update: String,
}

#[derive(Debug, Serialize)]
pub struct RelationshipGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub source_column: String,
    pub target_column: String,
    pub label: String,
}

/// Get foreign key relationships for visualization
pub async fn get_foreign_key_graph(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let schema = params.get("schema").map(|s| s.as_str()).unwrap_or("public");

    // Get all foreign key relationships
    let fk_query = r#"
        SELECT
            tc.table_name as source_table,
            kcu.column_name as source_column,
            ccu.table_name as target_table,
            ccu.column_name as target_column,
            tc.constraint_name,
            rc.delete_rule as on_delete,
            rc.update_rule as on_update
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
        JOIN information_schema.constraint_column_usage ccu
            ON ccu.constraint_name = tc.constraint_name
        JOIN information_schema.referential_constraints rc
            ON rc.constraint_name = tc.constraint_name
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = $1
        ORDER BY tc.table_name, kcu.column_name
    "#;

    match sqlx::query(fk_query)
        .bind(schema)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => {
            let mut relationships: Vec<TableRelationship> = Vec::new();
            let mut table_set: std::collections::HashSet<String> = std::collections::HashSet::new();

            for row in &rows {
                let source: String = row.get("source_table");
                let target: String = row.get("target_table");
                table_set.insert(source.clone());
                table_set.insert(target.clone());

                relationships.push(TableRelationship {
                    source_table: source,
                    source_column: row.get("source_column"),
                    target_table: target,
                    target_column: row.get("target_column"),
                    constraint_name: row.get("constraint_name"),
                    on_delete: row.get("on_delete"),
                    on_update: row.get("on_update"),
                });
            }

            // Build graph nodes with columns
            let mut nodes: Vec<GraphNode> = Vec::new();
            for table in &table_set {
                let cols_query = r#"
                    SELECT column_name
                    FROM information_schema.columns
                    WHERE table_name = $1 AND table_schema = $2
                    ORDER BY ordinal_position
                "#;

                let columns: Vec<String> = sqlx::query_scalar(cols_query)
                    .bind(table)
                    .bind(schema)
                    .fetch_all(&state.pool)
                    .await
                    .unwrap_or_default();

                nodes.push(GraphNode {
                    id: table.clone(),
                    label: table.clone(),
                    columns,
                });
            }

            // Build edges
            let edges: Vec<GraphEdge> = relationships.iter().map(|r| {
                GraphEdge {
                    source: r.source_table.clone(),
                    target: r.target_table.clone(),
                    source_column: r.source_column.clone(),
                    target_column: r.target_column.clone(),
                    label: r.constraint_name.clone(),
                }
            }).collect();

            let graph = RelationshipGraph { nodes, edges };

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "relationships": relationships,
                    "graph": graph,
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get relationships: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 21: Schema Compare
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SchemaCompareResult {
    pub tables_only_in_first: Vec<String>,
    pub tables_only_in_second: Vec<String>,
    pub tables_with_differences: Vec<TableDifference>,
}

#[derive(Debug, Serialize)]
pub struct TableDifference {
    pub table_name: String,
    pub column_differences: Vec<ColumnDifference>,
    pub index_differences: Vec<String>,
    pub constraint_differences: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ColumnDifference {
    pub column_name: String,
    pub difference_type: String, // "only_in_first", "only_in_second", "type_mismatch", "nullable_mismatch"
    pub first_value: Option<String>,
    pub second_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SchemaCompareRequest {
    pub first_schema: String,
    pub second_schema: String,
}

/// Compare two database schemas
pub async fn compare_schemas(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<SchemaCompareRequest>,
) -> impl IntoResponse {
    // Get tables in first schema
    let tables_query = "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE'";

    let first_tables: Vec<String> = sqlx::query_scalar(tables_query)
        .bind(&request.first_schema)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let second_tables: Vec<String> = sqlx::query_scalar(tables_query)
        .bind(&request.second_schema)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let first_set: std::collections::HashSet<_> = first_tables.iter().collect();
    let second_set: std::collections::HashSet<_> = second_tables.iter().collect();

    let tables_only_in_first: Vec<String> = first_set.difference(&second_set)
        .map(|s| (*s).clone())
        .collect();

    let tables_only_in_second: Vec<String> = second_set.difference(&first_set)
        .map(|s| (*s).clone())
        .collect();

    // Compare common tables
    let common_tables: Vec<_> = first_set.intersection(&second_set).collect();
    let mut tables_with_differences = Vec::new();

    for table in common_tables {
        let mut column_differences = Vec::new();
        let mut index_differences = Vec::new();
        let constraint_differences = Vec::<String>::new();

        // Get columns from both schemas
        let cols_query = r#"
            SELECT column_name, data_type, is_nullable, column_default
            FROM information_schema.columns
            WHERE table_schema = $1 AND table_name = $2
        "#;

        let first_cols: HashMap<String, (String, String, Option<String>)> = sqlx::query(cols_query)
            .bind(&request.first_schema)
            .bind(*table)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                let name: String = r.get("column_name");
                let dtype: String = r.get("data_type");
                let nullable: String = r.get("is_nullable");
                let default: Option<String> = r.get("column_default");
                (name, (dtype, nullable, default))
            })
            .collect();

        let second_cols: HashMap<String, (String, String, Option<String>)> = sqlx::query(cols_query)
            .bind(&request.second_schema)
            .bind(*table)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| {
                let name: String = r.get("column_name");
                let dtype: String = r.get("data_type");
                let nullable: String = r.get("is_nullable");
                let default: Option<String> = r.get("column_default");
                (name, (dtype, nullable, default))
            })
            .collect();

        // Find columns only in first
        for (col, (dtype, nullable, _)) in &first_cols {
            if !second_cols.contains_key(col) {
                column_differences.push(ColumnDifference {
                    column_name: col.clone(),
                    difference_type: "only_in_first".to_string(),
                    first_value: Some(format!("{} ({})", dtype, nullable)),
                    second_value: None,
                });
            }
        }

        // Find columns only in second or with differences
        for (col, (dtype2, nullable2, _)) in &second_cols {
            if let Some((dtype1, nullable1, _)) = first_cols.get(col) {
                if dtype1 != dtype2 {
                    column_differences.push(ColumnDifference {
                        column_name: col.clone(),
                        difference_type: "type_mismatch".to_string(),
                        first_value: Some(dtype1.clone()),
                        second_value: Some(dtype2.clone()),
                    });
                }
                if nullable1 != nullable2 {
                    column_differences.push(ColumnDifference {
                        column_name: col.clone(),
                        difference_type: "nullable_mismatch".to_string(),
                        first_value: Some(nullable1.clone()),
                        second_value: Some(nullable2.clone()),
                    });
                }
            } else {
                column_differences.push(ColumnDifference {
                    column_name: col.clone(),
                    difference_type: "only_in_second".to_string(),
                    first_value: None,
                    second_value: Some(format!("{} ({})", dtype2, nullable2)),
                });
            }
        }

        // Compare indexes
        let idx_query = "SELECT indexname FROM pg_indexes WHERE schemaname = $1 AND tablename = $2";

        let first_indexes: std::collections::HashSet<String> = sqlx::query_scalar(idx_query)
            .bind(&request.first_schema)
            .bind(*table)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();

        let second_indexes: std::collections::HashSet<String> = sqlx::query_scalar(idx_query)
            .bind(&request.second_schema)
            .bind(*table)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect();

        for idx in first_indexes.difference(&second_indexes) {
            index_differences.push(format!("Index '{}' only in {}", idx, request.first_schema));
        }
        for idx in second_indexes.difference(&first_indexes) {
            index_differences.push(format!("Index '{}' only in {}", idx, request.second_schema));
        }

        if !column_differences.is_empty() || !index_differences.is_empty() || !constraint_differences.is_empty() {
            tables_with_differences.push(TableDifference {
                table_name: (*table).clone(),
                column_differences,
                index_differences,
                constraint_differences,
            });
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!(SchemaCompareResult {
            tables_only_in_first,
            tables_only_in_second,
            tables_with_differences,
        })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 22: Schema Versioning
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SchemaVersion {
    pub id: String,
    pub version: i32,
    pub description: String,
    pub changes: serde_json::Value,
    pub created_at: String,
    pub created_by: Option<String>,
}

/// Get schema version history
pub async fn get_schema_versions(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    let query = r#"
        SELECT id, version, description, changes, created_at, created_by
        FROM dbmanager_schema_versions
        ORDER BY version DESC
        LIMIT 100
    "#;

    match sqlx::query(query).fetch_all(&state.pool).await {
        Ok(rows) => {
            let versions: Vec<SchemaVersion> = rows.iter().map(|row| {
                SchemaVersion {
                    id: row.get::<uuid::Uuid, _>("id").to_string(),
                    version: row.get("version"),
                    description: row.get("description"),
                    changes: row.get("changes"),
                    created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
                    created_by: row.get("created_by"),
                }
            }).collect();

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({ "versions": versions })),
                error: None,
            })
        }
        Err(e) => {
            // Table might not exist yet
            if e.to_string().contains("does not exist") {
                Json(ApiResponse {
                    success: true,
                    data: Some(serde_json::json!({ "versions": Vec::<SchemaVersion>::new() })),
                    error: None,
                })
            } else {
                Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to get versions: {}", e)),
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSchemaVersionRequest {
    pub description: String,
}

/// Create a new schema version snapshot
pub async fn create_schema_version(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<CreateSchemaVersionRequest>,
) -> impl IntoResponse {
    // Get current schema state
    let tables_query = r#"
        SELECT
            t.table_name,
            json_agg(json_build_object(
                'column_name', c.column_name,
                'data_type', c.data_type,
                'is_nullable', c.is_nullable,
                'column_default', c.column_default
            ) ORDER BY c.ordinal_position) as columns
        FROM information_schema.tables t
        JOIN information_schema.columns c ON t.table_name = c.table_name AND t.table_schema = c.table_schema
        WHERE t.table_schema = 'public' AND t.table_type = 'BASE TABLE'
        GROUP BY t.table_name
    "#;

    let schema_state = match sqlx::query(tables_query).fetch_all(&state.pool).await {
        Ok(rows) => {
            let mut state_map = serde_json::Map::new();
            for row in rows {
                let table: String = row.get("table_name");
                let columns: serde_json::Value = row.get("columns");
                state_map.insert(table, columns);
            }
            serde_json::Value::Object(state_map)
        }
        Err(e) => return Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to capture schema: {}", e)),
        }),
    };

    // Get next version number
    let version: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM dbmanager_schema_versions")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(1);

    // Insert version
    let insert_query = r#"
        INSERT INTO dbmanager_schema_versions (version, description, changes, created_at)
        VALUES ($1, $2, $3, NOW())
        RETURNING id
    "#;

    match sqlx::query_scalar::<_, uuid::Uuid>(insert_query)
        .bind(version)
        .bind(&request.description)
        .bind(&schema_state)
        .fetch_one(&state.pool)
        .await
    {
        Ok(id) => Json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "id": id.to_string(),
                "version": version,
                "description": request.description,
            })),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to create version: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 23: ER Diagram Generator
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ERDiagram {
    pub mermaid: String,
    pub dot: String,
    pub tables: Vec<ERTable>,
}

#[derive(Debug, Serialize)]
pub struct ERTable {
    pub name: String,
    pub columns: Vec<ERColumn>,
    pub primary_key: Vec<String>,
    pub foreign_keys: Vec<ERForeignKey>,
}

#[derive(Debug, Serialize)]
pub struct ERColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

#[derive(Debug, Serialize)]
pub struct ERForeignKey {
    pub column: String,
    pub references_table: String,
    pub references_column: String,
}

/// Generate ER diagram data
pub async fn generate_er_diagram(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let schema = params.get("schema").map(|s| s.as_str()).unwrap_or("public");
    let tables_filter: Option<Vec<&str>> = params.get("tables").map(|t| t.split(',').collect());

    // Get all tables
    let mut tables_query = String::from(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE'"
    );
    if tables_filter.is_some() {
        tables_query.push_str(" AND table_name = ANY($2)");
    }

    let table_names: Vec<String> = if let Some(filter) = &tables_filter {
        sqlx::query_scalar(&tables_query)
            .bind(schema)
            .bind(filter)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
    } else {
        sqlx::query_scalar(&tables_query)
            .bind(schema)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
    };

    let mut er_tables = Vec::new();
    let mut mermaid = String::from("erDiagram\n");
    let mut dot = String::from("digraph ERD {\n  graph [rankdir=LR];\n  node [shape=record];\n\n");

    for table_name in &table_names {
        // Get columns
        let cols_query = r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_pk,
                CASE WHEN fk.column_name IS NOT NULL THEN true ELSE false END as is_fk
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
                WHERE tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'
            ) pk ON c.column_name = pk.column_name
            LEFT JOIN (
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
                WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
            ) fk ON c.column_name = fk.column_name
            WHERE c.table_name = $1 AND c.table_schema = $2
            ORDER BY c.ordinal_position
        "#;

        let columns: Vec<ERColumn> = sqlx::query(cols_query)
            .bind(table_name)
            .bind(schema)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .iter()
            .map(|row| ERColumn {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get::<String, _>("is_nullable") == "YES",
                is_primary_key: row.get("is_pk"),
                is_foreign_key: row.get("is_fk"),
            })
            .collect();

        // Get foreign keys
        let fk_query = r#"
            SELECT
                kcu.column_name,
                ccu.table_name as ref_table,
                ccu.column_name as ref_column
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON ccu.constraint_name = tc.constraint_name
            WHERE tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = $2
        "#;

        let foreign_keys: Vec<ERForeignKey> = sqlx::query(fk_query)
            .bind(table_name)
            .bind(schema)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default()
            .iter()
            .map(|row| ERForeignKey {
                column: row.get("column_name"),
                references_table: row.get("ref_table"),
                references_column: row.get("ref_column"),
            })
            .collect();

        // Get primary key columns
        let pk_cols: Vec<String> = columns.iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.clone())
            .collect();

        // Build Mermaid diagram
        mermaid.push_str(&format!("  {} {{\n", table_name));
        for col in &columns {
            let pk_marker = if col.is_primary_key { " PK" } else { "" };
            let fk_marker = if col.is_foreign_key { " FK" } else { "" };
            mermaid.push_str(&format!("    {} {}{}{}\n",
                col.data_type.replace(' ', "_"), col.name, pk_marker, fk_marker));
        }
        mermaid.push_str("  }\n");

        // Add relationships to Mermaid
        for fk in &foreign_keys {
            mermaid.push_str(&format!("  {} ||--o{{ {} : \"{}\"\n",
                fk.references_table, table_name, fk.column));
        }

        // Build DOT diagram
        let col_labels: String = columns.iter()
            .map(|c| {
                let markers = format!("{}{}",
                    if c.is_primary_key { "🔑" } else { "" },
                    if c.is_foreign_key { "🔗" } else { "" }
                );
                format!("{} {} {}", c.name, c.data_type, markers)
            })
            .collect::<Vec<_>>()
            .join("\\l");

        dot.push_str(&format!("  {} [label=\"{{{}|{}\\l}}\"];\n",
            table_name, table_name, col_labels));

        er_tables.push(ERTable {
            name: table_name.clone(),
            columns,
            primary_key: pk_cols,
            foreign_keys,
        });
    }

    // Add DOT edges
    for table in &er_tables {
        for fk in &table.foreign_keys {
            dot.push_str(&format!("  {} -> {} [label=\"{}\"];\n",
                table.name, fk.references_table, fk.column));
        }
    }

    dot.push_str("}\n");

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!(ERDiagram {
            mermaid,
            dot,
            tables: er_tables,
        })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 24: Schema Search
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SchemaSearchResult {
    pub object_type: String, // table, column, index, constraint, function
    pub object_name: String,
    pub parent_name: Option<String>, // table name for columns
    pub schema_name: String,
    pub details: String,
}

#[derive(Debug, Deserialize)]
pub struct SchemaSearchRequest {
    pub query: String,
    pub search_tables: Option<bool>,
    pub search_columns: Option<bool>,
    pub search_indexes: Option<bool>,
    pub search_constraints: Option<bool>,
    pub search_functions: Option<bool>,
}

/// Search across entire schema
pub async fn search_schema(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<SchemaSearchRequest>,
) -> impl IntoResponse {
    let search_pattern = format!("%{}%", request.query.to_lowercase());
    let mut results = Vec::new();

    // Search tables
    if request.search_tables.unwrap_or(true) {
        let query = r#"
            SELECT table_schema, table_name
            FROM information_schema.tables
            WHERE table_type = 'BASE TABLE' AND LOWER(table_name) LIKE $1
            ORDER BY table_name
            LIMIT 50
        "#;

        if let Ok(rows) = sqlx::query(query).bind(&search_pattern).fetch_all(&state.pool).await {
            for row in rows {
                results.push(SchemaSearchResult {
                    object_type: "table".to_string(),
                    object_name: row.get("table_name"),
                    parent_name: None,
                    schema_name: row.get("table_schema"),
                    details: "Table".to_string(),
                });
            }
        }
    }

    // Search columns
    if request.search_columns.unwrap_or(true) {
        let query = r#"
            SELECT table_schema, table_name, column_name, data_type
            FROM information_schema.columns
            WHERE LOWER(column_name) LIKE $1
            ORDER BY table_name, column_name
            LIMIT 100
        "#;

        if let Ok(rows) = sqlx::query(query).bind(&search_pattern).fetch_all(&state.pool).await {
            for row in rows {
                let data_type: String = row.get("data_type");
                results.push(SchemaSearchResult {
                    object_type: "column".to_string(),
                    object_name: row.get("column_name"),
                    parent_name: Some(row.get("table_name")),
                    schema_name: row.get("table_schema"),
                    details: format!("Type: {}", data_type),
                });
            }
        }
    }

    // Search indexes
    if request.search_indexes.unwrap_or(true) {
        let query = r#"
            SELECT schemaname, tablename, indexname, indexdef
            FROM pg_indexes
            WHERE LOWER(indexname) LIKE $1
            ORDER BY indexname
            LIMIT 50
        "#;

        if let Ok(rows) = sqlx::query(query).bind(&search_pattern).fetch_all(&state.pool).await {
            for row in rows {
                results.push(SchemaSearchResult {
                    object_type: "index".to_string(),
                    object_name: row.get("indexname"),
                    parent_name: Some(row.get("tablename")),
                    schema_name: row.get("schemaname"),
                    details: row.get("indexdef"),
                });
            }
        }
    }

    // Search constraints
    if request.search_constraints.unwrap_or(true) {
        let query = r#"
            SELECT constraint_schema, table_name, constraint_name, constraint_type
            FROM information_schema.table_constraints
            WHERE LOWER(constraint_name) LIKE $1
            ORDER BY constraint_name
            LIMIT 50
        "#;

        if let Ok(rows) = sqlx::query(query).bind(&search_pattern).fetch_all(&state.pool).await {
            for row in rows {
                let constraint_type: String = row.get("constraint_type");
                results.push(SchemaSearchResult {
                    object_type: "constraint".to_string(),
                    object_name: row.get("constraint_name"),
                    parent_name: Some(row.get("table_name")),
                    schema_name: row.get("constraint_schema"),
                    details: format!("Type: {}", constraint_type),
                });
            }
        }
    }

    // Search functions
    if request.search_functions.unwrap_or(true) {
        let query = r#"
            SELECT n.nspname as schema_name, p.proname as function_name,
                   pg_get_function_arguments(p.oid) as arguments
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE LOWER(p.proname) LIKE $1 AND n.nspname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY p.proname
            LIMIT 50
        "#;

        if let Ok(rows) = sqlx::query(query).bind(&search_pattern).fetch_all(&state.pool).await {
            for row in rows {
                let args: String = row.get("arguments");
                results.push(SchemaSearchResult {
                    object_type: "function".to_string(),
                    object_name: row.get("function_name"),
                    parent_name: None,
                    schema_name: row.get("schema_name"),
                    details: format!("Arguments: {}", args),
                });
            }
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!({
            "results": results,
            "total": results.len(),
        })),
        error: None,
    })
}

// ============================================================================
// ENHANCEMENT 25: DDL Generator
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DDLGeneratorRequest {
    pub object_type: String, // table, index, constraint, function, view
    pub object_name: String,
    pub schema_name: Option<String>,
    pub include_indexes: Option<bool>,
    pub include_constraints: Option<bool>,
}

/// Generate DDL statements for database objects
pub async fn generate_ddl(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<DDLGeneratorRequest>,
) -> impl IntoResponse {
    let schema = request.schema_name.as_deref().unwrap_or("public");

    let ddl = match request.object_type.to_lowercase().as_str() {
        "table" => generate_table_ddl(&state.pool, schema, &request.object_name,
            request.include_indexes.unwrap_or(true),
            request.include_constraints.unwrap_or(true)).await,
        "index" => generate_index_ddl(&state.pool, schema, &request.object_name).await,
        "view" => generate_view_ddl(&state.pool, schema, &request.object_name).await,
        "function" => generate_function_ddl(&state.pool, schema, &request.object_name).await,
        _ => Err(format!("Unsupported object type: {}", request.object_type)),
    };

    match ddl {
        Ok(ddl_text) => Json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "ddl": ddl_text,
                "object_type": request.object_type,
                "object_name": request.object_name,
            })),
            error: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e),
        }),
    }
}

async fn generate_table_ddl(pool: &PgPool, schema: &str, table: &str, include_indexes: bool, include_constraints: bool) -> Result<String, String> {
    let mut ddl = String::new();

    // Get columns
    let cols_query = r#"
        SELECT column_name, data_type, character_maximum_length, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_schema = $1 AND table_name = $2
        ORDER BY ordinal_position
    "#;

    let columns = sqlx::query(cols_query)
        .bind(schema)
        .bind(table)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if columns.is_empty() {
        return Err("Table not found".to_string());
    }

    ddl.push_str(&format!("CREATE TABLE \"{}\".\"{}\" (\n", schema, table));

    let col_defs: Vec<String> = columns.iter().map(|row| {
        let name: String = row.get("column_name");
        let dtype: String = row.get("data_type");
        let max_len: Option<i32> = row.get("character_maximum_length");
        let nullable: String = row.get("is_nullable");
        let default: Option<String> = row.get("column_default");

        let type_with_len = if let Some(len) = max_len {
            format!("{}({})", dtype, len)
        } else {
            dtype
        };

        let nullable_str = if nullable == "NO" { " NOT NULL" } else { "" };
        let default_str = default.map(|d| format!(" DEFAULT {}", d)).unwrap_or_default();

        format!("  \"{}\" {}{}{}", name, type_with_len, nullable_str, default_str)
    }).collect();

    ddl.push_str(&col_defs.join(",\n"));

    // Add primary key
    let pk_query = r#"
        SELECT kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
        WHERE tc.table_schema = $1 AND tc.table_name = $2 AND tc.constraint_type = 'PRIMARY KEY'
    "#;

    let pk_cols: Vec<String> = sqlx::query_scalar(pk_query)
        .bind(schema)
        .bind(table)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    if !pk_cols.is_empty() {
        ddl.push_str(",\n  PRIMARY KEY (");
        ddl.push_str(&pk_cols.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "));
        ddl.push(')');
    }

    // Add foreign keys if requested
    if include_constraints {
        let fk_query = r#"
            SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name as ref_table,
                ccu.column_name as ref_column,
                rc.delete_rule,
                rc.update_rule
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON ccu.constraint_name = tc.constraint_name
            JOIN information_schema.referential_constraints rc ON rc.constraint_name = tc.constraint_name
            WHERE tc.table_schema = $1 AND tc.table_name = $2 AND tc.constraint_type = 'FOREIGN KEY'
        "#;

        if let Ok(fks) = sqlx::query(fk_query).bind(schema).bind(table).fetch_all(pool).await {
            for fk in fks {
                let col: String = fk.get("column_name");
                let ref_table: String = fk.get("ref_table");
                let ref_col: String = fk.get("ref_column");
                let on_delete: String = fk.get("delete_rule");
                let on_update: String = fk.get("update_rule");

                ddl.push_str(&format!(
                    ",\n  FOREIGN KEY (\"{}\") REFERENCES \"{}\"(\"{}\") ON DELETE {} ON UPDATE {}",
                    col, ref_table, ref_col, on_delete, on_update
                ));
            }
        }
    }

    ddl.push_str("\n);\n");

    // Add indexes if requested
    if include_indexes {
        let idx_query = "SELECT indexdef FROM pg_indexes WHERE schemaname = $1 AND tablename = $2 AND indexname NOT LIKE '%_pkey'";

        if let Ok(indexes) = sqlx::query_scalar::<_, String>(idx_query)
            .bind(schema)
            .bind(table)
            .fetch_all(pool)
            .await
        {
            for indexdef in indexes {
                ddl.push_str(&format!("\n{};\n", indexdef));
            }
        }
    }

    Ok(ddl)
}

async fn generate_index_ddl(pool: &PgPool, schema: &str, index_name: &str) -> Result<String, String> {
    let query = "SELECT indexdef FROM pg_indexes WHERE schemaname = $1 AND indexname = $2";

    sqlx::query_scalar::<_, String>(query)
        .bind(schema)
        .bind(index_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .map(|def| format!("{};\n", def))
        .ok_or_else(|| "Index not found".to_string())
}

async fn generate_view_ddl(pool: &PgPool, schema: &str, view_name: &str) -> Result<String, String> {
    let query = "SELECT definition FROM pg_views WHERE schemaname = $1 AND viewname = $2";

    sqlx::query_scalar::<_, String>(query)
        .bind(schema)
        .bind(view_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .map(|def| format!("CREATE OR REPLACE VIEW \"{}\".\"{}\" AS\n{};\n", schema, view_name, def))
        .ok_or_else(|| "View not found".to_string())
}

async fn generate_function_ddl(pool: &PgPool, schema: &str, func_name: &str) -> Result<String, String> {
    let query = r#"
        SELECT pg_get_functiondef(p.oid) as definition
        FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        WHERE n.nspname = $1 AND p.proname = $2
        LIMIT 1
    "#;

    sqlx::query_scalar::<_, String>(query)
        .bind(schema)
        .bind(func_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .map(|def| format!("{};\n", def))
        .ok_or_else(|| "Function not found".to_string())
}

// ============================================================================
// ENHANCEMENT 26: Connection Pooling Stats
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ConnectionPoolStats {
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub waiting_requests: u32,
    pub max_connections: i32,
    pub connections_by_state: HashMap<String, i64>,
    pub connections_by_application: HashMap<String, i64>,
    pub oldest_connection_seconds: i64,
    pub avg_connection_age_seconds: f64,
}

/// Get connection pool statistics
pub async fn get_connection_pool_stats(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    // Get PostgreSQL connection info
    let stats_query = r#"
        SELECT
            (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
            COUNT(*) as total_connections,
            COUNT(*) FILTER (WHERE state = 'active') as active,
            COUNT(*) FILTER (WHERE state = 'idle') as idle,
            COUNT(*) FILTER (WHERE state = 'idle in transaction') as idle_in_transaction,
            COUNT(*) FILTER (WHERE state IS NULL) as null_state,
            EXTRACT(EPOCH FROM (NOW() - MIN(backend_start)))::bigint as oldest_seconds,
            EXTRACT(EPOCH FROM AVG(NOW() - backend_start))::float as avg_age
        FROM pg_stat_activity
        WHERE datname = current_database()
    "#;

    let by_state_query = r#"
        SELECT COALESCE(state, 'unknown') as state, COUNT(*) as count
        FROM pg_stat_activity
        WHERE datname = current_database()
        GROUP BY state
    "#;

    let by_app_query = r#"
        SELECT COALESCE(application_name, 'unknown') as app, COUNT(*) as count
        FROM pg_stat_activity
        WHERE datname = current_database()
        GROUP BY application_name
    "#;

    let stats_result = sqlx::query(stats_query).fetch_one(&state.pool).await;
    let by_state_result = sqlx::query(by_state_query).fetch_all(&state.pool).await;
    let by_app_result = sqlx::query(by_app_query).fetch_all(&state.pool).await;

    match stats_result {
        Ok(row) => {
            let connections_by_state: HashMap<String, i64> = by_state_result
                .unwrap_or_default()
                .iter()
                .map(|r| (r.get::<String, _>("state"), r.get::<i64, _>("count")))
                .collect();

            let connections_by_application: HashMap<String, i64> = by_app_result
                .unwrap_or_default()
                .iter()
                .map(|r| (r.get::<String, _>("app"), r.get::<i64, _>("count")))
                .collect();

            let _total: i64 = row.get("total_connections");
            let active: i64 = row.get("active");
            let idle: i64 = row.get("idle");

            let stats = ConnectionPoolStats {
                pool_size: state.pool.size(),
                active_connections: active as u32,
                idle_connections: idle as u32,
                waiting_requests: 0, // SQLx doesn't expose this directly
                max_connections: row.get("max_connections"),
                connections_by_state,
                connections_by_application,
                oldest_connection_seconds: row.get::<Option<i64>, _>("oldest_seconds").unwrap_or(0),
                avg_connection_age_seconds: row.get::<Option<f64>, _>("avg_age").unwrap_or(0.0),
            };

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::to_value(stats).unwrap()),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get pool stats: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 27: Slow Query Log
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SlowQueryEntry {
    pub id: String,
    pub query: String,
    pub execution_time_ms: f64,
    pub rows_affected: Option<i64>,
    pub user_name: Option<String>,
    pub application_name: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SlowQueryThreshold {
    pub threshold_ms: u32,
}

/// Get slow queries from history
pub async fn get_slow_queries(
    State(state): State<Arc<DbManagerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let threshold_ms: f64 = params.get("threshold_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000.0);

    let limit: i64 = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Get from our query history table
    let query = r#"
        SELECT id, query, execution_time_ms, row_count, executed_at
        FROM dbmanager_query_history
        WHERE execution_time_ms > $1
        ORDER BY execution_time_ms DESC
        LIMIT $2
    "#;

    match sqlx::query(query)
        .bind(threshold_ms)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
    {
        Ok(rows) => {
            let slow_queries: Vec<SlowQueryEntry> = rows.iter().map(|row| {
                SlowQueryEntry {
                    id: row.get::<uuid::Uuid, _>("id").to_string(),
                    query: row.get("query"),
                    execution_time_ms: row.get("execution_time_ms"),
                    rows_affected: row.get("row_count"),
                    user_name: None,
                    application_name: None,
                    recorded_at: row.get::<chrono::DateTime<chrono::Utc>, _>("executed_at").to_rfc3339(),
                }
            }).collect();

            // Also try to get from pg_stat_statements if available
            let pg_stats_query = r#"
                SELECT query, calls, total_exec_time, mean_exec_time, rows
                FROM pg_stat_statements
                WHERE mean_exec_time > $1
                ORDER BY mean_exec_time DESC
                LIMIT $2
            "#;

            let pg_stats: Vec<serde_json::Value> = sqlx::query(pg_stats_query)
                .bind(threshold_ms)
                .bind(limit)
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default()
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "query": row.get::<String, _>("query"),
                        "calls": row.get::<i64, _>("calls"),
                        "total_time_ms": row.get::<f64, _>("total_exec_time"),
                        "mean_time_ms": row.get::<f64, _>("mean_exec_time"),
                        "rows": row.get::<i64, _>("rows"),
                    })
                })
                .collect();

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "slow_queries": slow_queries,
                    "pg_stat_statements": pg_stats,
                    "threshold_ms": threshold_ms,
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get slow queries: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 28: Lock Monitoring
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LockInfo {
    pub pid: i32,
    pub lock_type: String,
    pub database: String,
    pub relation: Option<String>,
    pub mode: String,
    pub granted: bool,
    pub query: Option<String>,
    pub query_start: Option<String>,
    pub wait_duration_seconds: Option<f64>,
    pub blocking_pid: Option<i32>,
}

/// Get current database locks
pub async fn get_locks(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    let query = r#"
        SELECT
            l.pid,
            l.locktype,
            d.datname as database,
            l.relation::regclass::text as relation,
            l.mode,
            l.granted,
            a.query,
            a.query_start::text,
            EXTRACT(EPOCH FROM (NOW() - a.query_start))::float as wait_seconds,
            bl.pid as blocking_pid
        FROM pg_locks l
        JOIN pg_stat_activity a ON l.pid = a.pid
        LEFT JOIN pg_database d ON l.database = d.oid
        LEFT JOIN pg_locks bl ON l.relation = bl.relation AND l.pid != bl.pid AND bl.granted
        WHERE l.relation IS NOT NULL
        ORDER BY l.granted, a.query_start
    "#;

    match sqlx::query(query).fetch_all(&state.pool).await {
        Ok(rows) => {
            let locks: Vec<LockInfo> = rows.iter().map(|row| {
                LockInfo {
                    pid: row.get("pid"),
                    lock_type: row.get("locktype"),
                    database: row.get::<Option<String>, _>("database").unwrap_or_default(),
                    relation: row.get("relation"),
                    mode: row.get("mode"),
                    granted: row.get("granted"),
                    query: row.get("query"),
                    query_start: row.get("query_start"),
                    wait_duration_seconds: row.get("wait_seconds"),
                    blocking_pid: row.get("blocking_pid"),
                }
            }).collect();

            // Get blocking relationships
            let blocked_query = r#"
                SELECT
                    blocked_locks.pid AS blocked_pid,
                    blocked_activity.usename AS blocked_user,
                    blocking_locks.pid AS blocking_pid,
                    blocking_activity.usename AS blocking_user,
                    blocked_activity.query AS blocked_query,
                    blocking_activity.query AS blocking_query
                FROM pg_locks blocked_locks
                JOIN pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
                JOIN pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
                    AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
                    AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
                    AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
                    AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
                    AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
                    AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
                    AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
                    AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
                    AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
                    AND blocking_locks.pid != blocked_locks.pid
                JOIN pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
                WHERE NOT blocked_locks.granted
            "#;

            let blocking: Vec<serde_json::Value> = sqlx::query(blocked_query)
                .fetch_all(&state.pool)
                .await
                .unwrap_or_default()
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "blocked_pid": row.get::<i32, _>("blocked_pid"),
                        "blocked_user": row.get::<String, _>("blocked_user"),
                        "blocking_pid": row.get::<i32, _>("blocking_pid"),
                        "blocking_user": row.get::<String, _>("blocking_user"),
                        "blocked_query": row.get::<String, _>("blocked_query"),
                        "blocking_query": row.get::<String, _>("blocking_query"),
                    })
                })
                .collect();

            Json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "locks": locks,
                    "blocking_relationships": blocking,
                    "total_locks": locks.len(),
                    "waiting_locks": locks.iter().filter(|l| !l.granted).count(),
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to get locks: {}", e)),
        }),
    }
}

// ============================================================================
// ENHANCEMENT 29: Query Permissions Check
// ============================================================================

#[derive(Debug, Serialize)]
pub struct QueryPermissions {
    pub can_execute: bool,
    pub required_permissions: Vec<RequiredPermission>,
    pub missing_permissions: Vec<String>,
    pub affected_objects: Vec<AffectedObject>,
}

#[derive(Debug, Serialize)]
pub struct RequiredPermission {
    pub permission: String,
    pub object_type: String,
    pub object_name: String,
}

#[derive(Debug, Serialize)]
pub struct AffectedObject {
    pub object_type: String,
    pub object_name: String,
    pub operation: String,
}

/// Check what permissions a query requires
pub async fn check_query_permissions(
    State(state): State<Arc<DbManagerState>>,
    Json(request): Json<ExecuteQueryRequest>,
) -> impl IntoResponse {
    let query_upper = request.query.to_uppercase();
    let mut required_permissions = Vec::new();
    let mut affected_objects = Vec::new();

    // Parse query to identify operations and objects
    // This is a simplified parser - production would use proper SQL parsing

    // Check for SELECT
    if query_upper.contains("SELECT") {
        if let Some(tables) = extract_tables_from_query(&request.query, "FROM") {
            for table in tables {
                required_permissions.push(RequiredPermission {
                    permission: "SELECT".to_string(),
                    object_type: "table".to_string(),
                    object_name: table.clone(),
                });
                affected_objects.push(AffectedObject {
                    object_type: "table".to_string(),
                    object_name: table,
                    operation: "read".to_string(),
                });
            }
        }
    }

    // Check for INSERT
    if query_upper.contains("INSERT INTO") {
        if let Some(tables) = extract_tables_from_query(&request.query, "INSERT INTO") {
            for table in tables {
                required_permissions.push(RequiredPermission {
                    permission: "INSERT".to_string(),
                    object_type: "table".to_string(),
                    object_name: table.clone(),
                });
                affected_objects.push(AffectedObject {
                    object_type: "table".to_string(),
                    object_name: table,
                    operation: "write".to_string(),
                });
            }
        }
    }

    // Check for UPDATE
    if query_upper.contains("UPDATE") && !query_upper.contains("FOR UPDATE") {
        if let Some(tables) = extract_tables_from_query(&request.query, "UPDATE") {
            for table in tables {
                required_permissions.push(RequiredPermission {
                    permission: "UPDATE".to_string(),
                    object_type: "table".to_string(),
                    object_name: table.clone(),
                });
                affected_objects.push(AffectedObject {
                    object_type: "table".to_string(),
                    object_name: table,
                    operation: "modify".to_string(),
                });
            }
        }
    }

    // Check for DELETE
    if query_upper.contains("DELETE FROM") {
        if let Some(tables) = extract_tables_from_query(&request.query, "DELETE FROM") {
            for table in tables {
                required_permissions.push(RequiredPermission {
                    permission: "DELETE".to_string(),
                    object_type: "table".to_string(),
                    object_name: table.clone(),
                });
                affected_objects.push(AffectedObject {
                    object_type: "table".to_string(),
                    object_name: table,
                    operation: "delete".to_string(),
                });
            }
        }
    }

    // Check for DDL operations
    if query_upper.contains("CREATE TABLE") {
        required_permissions.push(RequiredPermission {
            permission: "CREATE".to_string(),
            object_type: "schema".to_string(),
            object_name: "public".to_string(),
        });
    }

    if query_upper.contains("DROP TABLE") || query_upper.contains("ALTER TABLE") {
        required_permissions.push(RequiredPermission {
            permission: "ALTER".to_string(),
            object_type: "table".to_string(),
            object_name: "specified_table".to_string(),
        });
    }

    // Try to execute EXPLAIN to verify syntax and permissions
    let explain_query = format!("EXPLAIN {}", request.query);
    let can_execute = sqlx::query(&explain_query)
        .execute(&state.pool)
        .await
        .is_ok();

    // Check actual permissions
    let mut missing_permissions = Vec::new();
    for perm in &required_permissions {
        let check_query = r#"
            SELECT has_table_privilege(current_user, $1, $2) as has_perm
        "#;

        let has_perm: bool = sqlx::query_scalar(check_query)
            .bind(&perm.object_name)
            .bind(&perm.permission)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(true); // Default to true if check fails (might be non-table object)

        if !has_perm {
            missing_permissions.push(format!("{} on {}", perm.permission, perm.object_name));
        }
    }

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!(QueryPermissions {
            can_execute,
            required_permissions,
            missing_permissions,
            affected_objects,
        })),
        error: None,
    })
}

fn extract_tables_from_query(query: &str, keyword: &str) -> Option<Vec<String>> {
    let upper = query.to_uppercase();
    let keyword_upper = keyword.to_uppercase();

    if let Some(start) = upper.find(&keyword_upper) {
        let after_keyword = &query[start + keyword.len()..];
        let trimmed = after_keyword.trim();

        // Get the first word/identifier
        let end_chars = [' ', '\n', '\t', '(', ')'];
        let end_pos = trimmed.find(|c: char| end_chars.contains(&c)).unwrap_or(trimmed.len());
        let table_part = &trimmed[..end_pos];

        // Clean up the table name
        let table = table_part.trim_matches('"').trim();
        if !table.is_empty() && is_valid_identifier(table) {
            return Some(vec![table.to_string()]);
        }
    }
    None
}

// ============================================================================
// ENHANCEMENT 30: Audit Log Dashboard
// ============================================================================

#[derive(Debug, Serialize)]
pub struct AuditDashboard {
    pub total_events: i64,
    pub events_today: i64,
    pub events_this_week: i64,
    pub events_by_type: HashMap<String, i64>,
    pub events_by_user: HashMap<String, i64>,
    pub events_by_hour: Vec<HourlyCount>,
    pub recent_events: Vec<AuditEvent>,
    pub top_tables: Vec<TableActivity>,
}

#[derive(Debug, Serialize)]
pub struct HourlyCount {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TableActivity {
    pub table_name: String,
    pub select_count: i64,
    pub insert_count: i64,
    pub update_count: i64,
    pub delete_count: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: String,
    pub table_name: Option<String>,
    pub query: String,
    pub user_id: Option<String>,
    pub created_at: String,
    pub metadata: Option<serde_json::Value>,
}

/// Get audit log dashboard with analytics
pub async fn get_audit_dashboard(
    State(state): State<Arc<DbManagerState>>,
) -> impl IntoResponse {
    // Total events
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dbmanager_audit_log")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    // Events today
    let today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dbmanager_audit_log WHERE created_at >= CURRENT_DATE"
    )
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    // Events this week
    let this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dbmanager_audit_log WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'"
    )
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    // Events by type
    let by_type_query = r#"
        SELECT event_type, COUNT(*) as count
        FROM dbmanager_audit_log
        GROUP BY event_type
        ORDER BY count DESC
    "#;

    let events_by_type: HashMap<String, i64> = sqlx::query(by_type_query)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| (r.get::<String, _>("event_type"), r.get::<i64, _>("count")))
        .collect();

    // Events by user
    let by_user_query = r#"
        SELECT COALESCE(user_id, 'anonymous') as user_id, COUNT(*) as count
        FROM dbmanager_audit_log
        GROUP BY user_id
        ORDER BY count DESC
        LIMIT 10
    "#;

    let events_by_user: HashMap<String, i64> = sqlx::query(by_user_query)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| (r.get::<String, _>("user_id"), r.get::<i64, _>("count")))
        .collect();

    // Events by hour (last 24 hours)
    let by_hour_query = r#"
        SELECT EXTRACT(HOUR FROM created_at)::int as hour, COUNT(*) as count
        FROM dbmanager_audit_log
        WHERE created_at >= NOW() - INTERVAL '24 hours'
        GROUP BY hour
        ORDER BY hour
    "#;

    let events_by_hour: Vec<HourlyCount> = sqlx::query(by_hour_query)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| HourlyCount {
            hour: r.get("hour"),
            count: r.get("count"),
        })
        .collect();

    // Recent events
    let recent_query = r#"
        SELECT id, event_type, table_name, query, user_id, created_at, metadata
        FROM dbmanager_audit_log
        ORDER BY created_at DESC
        LIMIT 20
    "#;

    let recent_events: Vec<AuditEvent> = sqlx::query(recent_query)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| AuditEvent {
            id: r.get::<uuid::Uuid, _>("id").to_string(),
            event_type: r.get("event_type"),
            table_name: r.get("table_name"),
            query: r.get::<String, _>("query").chars().take(200).collect(),
            user_id: r.get("user_id"),
            created_at: r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            metadata: r.get("metadata"),
        })
        .collect();

    // Top tables by activity
    let top_tables_query = r#"
        SELECT
            table_name,
            COUNT(*) FILTER (WHERE event_type = 'SELECT') as select_count,
            COUNT(*) FILTER (WHERE event_type = 'INSERT') as insert_count,
            COUNT(*) FILTER (WHERE event_type = 'UPDATE') as update_count,
            COUNT(*) FILTER (WHERE event_type = 'DELETE') as delete_count
        FROM dbmanager_audit_log
        WHERE table_name IS NOT NULL
        GROUP BY table_name
        ORDER BY COUNT(*) DESC
        LIMIT 10
    "#;

    let top_tables: Vec<TableActivity> = sqlx::query(top_tables_query)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| TableActivity {
            table_name: r.get("table_name"),
            select_count: r.get("select_count"),
            insert_count: r.get("insert_count"),
            update_count: r.get("update_count"),
            delete_count: r.get("delete_count"),
        })
        .collect();

    Json(ApiResponse {
        success: true,
        data: Some(serde_json::json!(AuditDashboard {
            total_events: total,
            events_today: today,
            events_this_week: this_week,
            events_by_type,
            events_by_user,
            events_by_hour,
            recent_events,
            top_tables,
        })),
        error: None,
    })
}

// ============================================================================
// Router
// ============================================================================

use axum::routing::{delete, get, post, put};
use axum::Router;

/// Create the database manager router with database pool
/// Note: Call `init_dbmanager_tables(&pool)` before using this router
/// to ensure the persistence tables exist
pub fn dbmanager_router(pool: PgPool) -> Router {
    let state = Arc::new(DbManagerState::new(pool));

    Router::new()
        // Status & Info
        .route("/status", get(get_status))
        .route("/stats", get(get_stats))
        // Tables
        .route("/tables", get(list_tables))
        .route("/tables", post(create_table))
        .route("/tables/:table_name", get(get_table))
        .route("/tables/:table_name", put(alter_table))
        .route("/tables/:table_name", delete(drop_table))
        .route("/tables/:table_name/columns", get(get_table_columns))
        .route("/tables/:table_name/indexes", get(get_table_indexes))
        .route("/tables/:table_name/foreign-keys", get(get_table_foreign_keys))
        .route("/tables/:table_name/truncate", post(truncate_table))
        // Table Advanced Operations
        .route("/tables/:table_name/clone", post(clone_table))
        .route("/tables/:table_name/partitions", get(get_table_partitions))
        .route("/tables/:table_name/partitions", post(create_partitions))
        .route("/tables/:table_name/statistics", get(get_table_statistics))
        .route("/tables/:table_name/index-recommendations", get(get_index_recommendations))
        // Data operations
        .route("/tables/:table_name/data", get(get_table_data))
        .route("/tables/:table_name/data", post(insert_row))
        .route("/tables/:table_name/data/:row_id", put(update_row))
        .route("/tables/:table_name/data/:row_id", delete(delete_row))
        .route("/tables/:table_name/data/bulk", delete(bulk_delete_rows))
        // Query - Basic
        .route("/query", post(execute_query))
        .route("/query/explain", post(explain_query))
        .route("/query/history", get(get_query_history))
        .route("/query/save", post(save_query))
        .route("/query/saved", get(get_saved_queries))
        .route("/query/saved/:query_id", delete(delete_saved_query))
        // Query - Advanced
        .route("/query/snippets", get(get_query_snippets))
        .route("/query/snippets", post(save_query_snippet))
        .route("/query/snippets/:snippet_id", put(update_query_snippet))
        .route("/query/snippets/:snippet_id", delete(delete_query_snippet))
        .route("/query/parameterized", post(execute_parameterized_query))
        .route("/query/multi", post(execute_multi_statement))
        .route("/query/timeout", post(execute_query_with_timeout))
        .route("/query/cost", post(estimate_query_cost))
        .route("/query/slow", get(get_slow_queries))
        .route("/query/permissions", post(check_query_permissions))
        // Import/Export
        .route("/export/tables", post(export_tables))
        .route("/export/query", post(export_query_result))
        .route("/export/excel", post(export_to_excel))
        .route("/export/insert", post(export_as_insert))
        .route("/import/sql", post(import_sql))
        .route("/import/csv", post(import_csv))
        // Schema Operations
        .route("/schema/relationships", get(get_foreign_key_graph))
        .route("/schema/compare", post(compare_schemas))
        .route("/schema/versions", get(get_schema_versions))
        .route("/schema/versions", post(create_schema_version))
        .route("/schema/er-diagram", get(generate_er_diagram))
        .route("/schema/search", post(search_schema))
        .route("/schema/ddl", post(generate_ddl))
        // Monitoring
        .route("/pool/stats", get(get_connection_pool_stats))
        .route("/locks", get(get_locks))
        // Audit
        .route("/audit", get(get_audit_log))
        .route("/audit/clear", post(clear_audit_log))
        .route("/audit/dashboard", get(get_audit_dashboard))
        .with_state(state)
}

/// Initialize database tables for dbmanager persistence
/// Call this function once at application startup
pub async fn init_dbmanager_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create saved queries table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dbmanager_saved_queries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            query TEXT NOT NULL,
            description TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await?;

    // Create query history table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dbmanager_query_history (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            query TEXT NOT NULL,
            execution_time_ms DOUBLE PRECISION NOT NULL,
            status VARCHAR(50) NOT NULL,
            row_count BIGINT,
            error_message TEXT,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await?;

    // Create index for faster history retrieval
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_query_history_executed_at
        ON dbmanager_query_history(executed_at DESC)
    "#)
    .execute(pool)
    .await?;

    // Create query snippets table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dbmanager_query_snippets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            description TEXT,
            query TEXT NOT NULL,
            category VARCHAR(100),
            parameters JSONB DEFAULT '[]'::jsonb,
            tags TEXT[] DEFAULT ARRAY[]::TEXT[],
            usage_count INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await?;

    // Create index for snippet search
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_query_snippets_category
        ON dbmanager_query_snippets(category)
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_query_snippets_tags
        ON dbmanager_query_snippets USING GIN(tags)
    "#)
    .execute(pool)
    .await?;

    // Create schema versions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dbmanager_schema_versions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            version VARCHAR(100) NOT NULL,
            description TEXT,
            schema_snapshot JSONB NOT NULL,
            changes JSONB DEFAULT '[]'::jsonb,
            created_by VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await?;

    // Create index for schema version retrieval
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_schema_versions_created_at
        ON dbmanager_schema_versions(created_at DESC)
    "#)
    .execute(pool)
    .await?;

    // Create slow query log table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS dbmanager_slow_queries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            query TEXT NOT NULL,
            execution_time_ms DOUBLE PRECISION NOT NULL,
            rows_affected BIGINT,
            plan JSONB,
            logged_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    "#)
    .execute(pool)
    .await?;

    // Create index for slow query retrieval
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_slow_queries_logged_at
        ON dbmanager_slow_queries(logged_at DESC)
    "#)
    .execute(pool)
    .await?;

    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS idx_dbmanager_slow_queries_execution_time
        ON dbmanager_slow_queries(execution_time_ms DESC)
    "#)
    .execute(pool)
    .await?;

    tracing::info!("Database Manager tables initialized successfully");
    Ok(())
}
