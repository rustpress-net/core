//! D1 Database Service
//!
//! Manage Cloudflare D1 serverless SQL databases

use crate::client::CloudflareClient;
use crate::error::{CloudflareError, CloudflareResult};
use crate::models::{D1Database, D1QueryResult};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};

/// D1 Database service
pub struct D1Service {
    client: Option<Arc<CloudflareClient>>,
    #[allow(dead_code)]
    db: PgPool,
}

impl D1Service {
    /// Create a new D1 service
    pub fn new(client: Arc<CloudflareClient>, db: PgPool) -> Self {
        Self {
            client: Some(client),
            db,
        }
    }

    /// Create a new D1 service without a client
    pub fn new_unconfigured(db: PgPool) -> Self {
        Self { client: None, db }
    }

    /// Get the client or return an error
    fn client(&self) -> CloudflareResult<&CloudflareClient> {
        self.client
            .as_ref()
            .map(|c| c.as_ref())
            .ok_or(CloudflareError::NotConfigured)
    }

    /// List all D1 databases
    pub async fn list_databases(&self) -> CloudflareResult<Vec<D1Database>> {
        let client = self.client()?;
        let databases = client.list_d1_databases().await?;
        debug!("Found {} D1 databases", databases.len());
        Ok(databases)
    }

    /// Get a D1 database by ID
    pub async fn get_database(&self, database_id: &str) -> CloudflareResult<D1Database> {
        let databases = self.list_databases().await?;
        databases
            .into_iter()
            .find(|db| db.uuid == database_id)
            .ok_or_else(|| CloudflareError::D1Error(format!("Database '{}' not found", database_id)))
    }

    /// Create a new D1 database
    pub async fn create_database(&self, name: &str) -> CloudflareResult<D1Database> {
        let client = self.client()?;
        info!("Creating D1 database: {}", name);
        let database = client.create_d1_database(name).await?;
        info!("D1 database created: {}", database.uuid);
        Ok(database)
    }

    /// Execute a SQL query on a D1 database
    pub async fn execute_query(
        &self,
        database_id: &str,
        sql: &str,
    ) -> CloudflareResult<D1QueryResult> {
        let client = self.client()?;
        debug!("Executing SQL on D1 database {}: {}", database_id, sql);
        let result = client.query_d1(database_id, sql).await?;
        debug!(
            "Query executed successfully, {} rows affected",
            result.meta.as_ref().and_then(|m| m.changes).unwrap_or(0)
        );
        Ok(result)
    }

    /// Execute multiple SQL statements in a batch
    pub async fn execute_batch(
        &self,
        database_id: &str,
        statements: Vec<String>,
    ) -> CloudflareResult<Vec<D1QueryResult>> {
        let mut results = Vec::new();
        for sql in statements {
            let result = self.execute_query(database_id, &sql).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Create a table in a D1 database
    pub async fn create_table(
        &self,
        database_id: &str,
        table_name: &str,
        columns: Vec<(&str, &str)>,
    ) -> CloudflareResult<D1QueryResult> {
        let column_defs: Vec<String> = columns
            .iter()
            .map(|(name, def)| format!("{} {}", name, def))
            .collect();
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            column_defs.join(", ")
        );
        self.execute_query(database_id, &sql).await
    }

    /// Drop a table from a D1 database
    pub async fn drop_table(
        &self,
        database_id: &str,
        table_name: &str,
    ) -> CloudflareResult<D1QueryResult> {
        let sql = format!("DROP TABLE IF EXISTS {}", table_name);
        self.execute_query(database_id, &sql).await
    }

    /// List all tables in a D1 database
    pub async fn list_tables(&self, database_id: &str) -> CloudflareResult<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
        let result = self.execute_query(database_id, sql).await?;

        let tables: Vec<String> = result
            .results
            .iter()
            .filter_map(|row| {
                row.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(tables)
    }

    /// Get table schema
    pub async fn get_table_schema(
        &self,
        database_id: &str,
        table_name: &str,
    ) -> CloudflareResult<Vec<serde_json::Value>> {
        let sql = format!("PRAGMA table_info({})", table_name);
        let result = self.execute_query(database_id, &sql).await?;
        Ok(result.results)
    }

    /// Insert a row into a table
    pub async fn insert(
        &self,
        database_id: &str,
        table_name: &str,
        data: serde_json::Value,
    ) -> CloudflareResult<D1QueryResult> {
        if let serde_json::Value::Object(map) = data {
            let columns: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            let placeholders: Vec<String> = map
                .values()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                    serde_json::Value::Null => "NULL".to_string(),
                    _ => v.to_string(),
                })
                .collect();

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table_name,
                columns.join(", "),
                placeholders.join(", ")
            );
            self.execute_query(database_id, &sql).await
        } else {
            Err(CloudflareError::D1Error("Data must be an object".to_string()))
        }
    }

    /// Select rows from a table
    pub async fn select(
        &self,
        database_id: &str,
        table_name: &str,
        columns: Option<Vec<&str>>,
        where_clause: Option<&str>,
        limit: Option<u32>,
    ) -> CloudflareResult<D1QueryResult> {
        let cols = columns
            .map(|c| c.join(", "))
            .unwrap_or_else(|| "*".to_string());

        let mut sql = format!("SELECT {} FROM {}", cols, table_name);

        if let Some(w) = where_clause {
            sql.push_str(&format!(" WHERE {}", w));
        }

        if let Some(l) = limit {
            sql.push_str(&format!(" LIMIT {}", l));
        }

        self.execute_query(database_id, &sql).await
    }

    /// Update rows in a table
    pub async fn update(
        &self,
        database_id: &str,
        table_name: &str,
        data: serde_json::Value,
        where_clause: &str,
    ) -> CloudflareResult<D1QueryResult> {
        if let serde_json::Value::Object(map) = data {
            let set_clauses: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let value = match v {
                        serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                        serde_json::Value::Null => "NULL".to_string(),
                        _ => v.to_string(),
                    };
                    format!("{} = {}", k, value)
                })
                .collect();

            let sql = format!(
                "UPDATE {} SET {} WHERE {}",
                table_name,
                set_clauses.join(", "),
                where_clause
            );
            self.execute_query(database_id, &sql).await
        } else {
            Err(CloudflareError::D1Error("Data must be an object".to_string()))
        }
    }

    /// Delete rows from a table
    pub async fn delete(
        &self,
        database_id: &str,
        table_name: &str,
        where_clause: &str,
    ) -> CloudflareResult<D1QueryResult> {
        let sql = format!("DELETE FROM {} WHERE {}", table_name, where_clause);
        self.execute_query(database_id, &sql).await
    }
}
