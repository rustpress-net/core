//! Query Result Caching with Invalidation
//!
//! Database query result caching with automatic invalidation based on table changes.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Query cache errors
#[derive(Debug, Error)]
pub enum QueryCacheError {
    #[error("Cache miss")]
    CacheMiss,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Query error: {0}")]
    Query(String),
}

/// Cached query result
struct CachedQuery {
    data: Vec<u8>,
    created: Instant,
    ttl: Duration,
    tables: HashSet<String>,
    hits: u64,
}

/// Query cache configuration
#[derive(Debug, Clone)]
pub struct QueryCacheConfig {
    /// Maximum cached queries
    pub max_entries: usize,
    /// Default TTL for cached queries
    pub default_ttl: Duration,
    /// Enable query fingerprinting
    pub fingerprint_queries: bool,
    /// Maximum result size to cache (bytes)
    pub max_result_size: usize,
    /// Tables to never cache queries for
    pub excluded_tables: Vec<String>,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 5000,
            default_ttl: Duration::from_secs(300),
            fingerprint_queries: true,
            max_result_size: 1024 * 1024, // 1MB
            excluded_tables: vec![
                "sessions".to_string(),
                "cache".to_string(),
                "queue".to_string(),
            ],
        }
    }
}

/// Query cache with table-based invalidation
pub struct QueryCache {
    config: QueryCacheConfig,
    /// Cached queries: fingerprint -> cached result
    cache: Arc<RwLock<HashMap<String, CachedQuery>>>,
    /// Table index: table name -> set of query fingerprints
    table_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Statistics
    stats: Arc<RwLock<QueryCacheStats>>,
}

/// Query cache statistics
#[derive(Debug, Clone, Default)]
pub struct QueryCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub stores: u64,
    pub invalidations: u64,
    pub evictions: u64,
    pub skipped_large: u64,
}

impl QueryCache {
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            table_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(QueryCacheStats::default())),
        }
    }

    /// Generate query fingerprint
    pub fn fingerprint(&self, sql: &str, params: &[&str]) -> String {
        let mut hasher = blake3::Hasher::new();

        if self.config.fingerprint_queries {
            // Normalize query
            let normalized = normalize_sql(sql);
            hasher.update(normalized.as_bytes());
        } else {
            hasher.update(sql.as_bytes());
        }

        // Add parameters
        for param in params {
            hasher.update(b":");
            hasher.update(param.as_bytes());
        }

        hasher.finalize().to_hex()[..16].to_string()
    }

    /// Get cached query result
    pub fn get<T: DeserializeOwned>(&self, fingerprint: &str) -> Result<T, QueryCacheError> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(fingerprint) {
            // Check TTL
            if entry.created.elapsed() > entry.ttl {
                cache.remove(fingerprint);
                self.stats.write().misses += 1;
                return Err(QueryCacheError::CacheMiss);
            }

            entry.hits += 1;
            self.stats.write().hits += 1;

            serde_json::from_slice(&entry.data)
                .map_err(|e| QueryCacheError::Serialization(e.to_string()))
        } else {
            self.stats.write().misses += 1;
            Err(QueryCacheError::CacheMiss)
        }
    }

    /// Store query result
    pub fn store<T: Serialize>(
        &self,
        fingerprint: String,
        result: &T,
        tables: Vec<String>,
        ttl: Option<Duration>,
    ) -> Result<(), QueryCacheError> {
        let data = serde_json::to_vec(result)
            .map_err(|e| QueryCacheError::Serialization(e.to_string()))?;

        // Check size limit
        if data.len() > self.config.max_result_size {
            self.stats.write().skipped_large += 1;
            return Ok(());
        }

        // Check if any table is excluded
        for table in &tables {
            if self.config.excluded_tables.contains(table) {
                return Ok(());
            }
        }

        // Evict if needed
        self.evict_if_needed();

        let table_set: HashSet<String> = tables.iter().cloned().collect();

        // Index by tables
        {
            let mut table_index = self.table_index.write();
            for table in &tables {
                table_index
                    .entry(table.clone())
                    .or_insert_with(HashSet::new)
                    .insert(fingerprint.clone());
            }
        }

        // Store entry
        let entry = CachedQuery {
            data,
            created: Instant::now(),
            ttl: ttl.unwrap_or(self.config.default_ttl),
            tables: table_set,
            hits: 0,
        };

        self.cache.write().insert(fingerprint, entry);
        self.stats.write().stores += 1;

        Ok(())
    }

    /// Invalidate all queries touching a table
    pub fn invalidate_table(&self, table: &str) {
        let fingerprints: Vec<String> = {
            let table_index = self.table_index.read();
            table_index
                .get(table)
                .map(|fps| fps.iter().cloned().collect())
                .unwrap_or_default()
        };

        let mut cache = self.cache.write();
        let mut stats = self.stats.write();

        for fp in fingerprints {
            if cache.remove(&fp).is_some() {
                stats.invalidations += 1;
            }
        }

        // Clear table index entry
        self.table_index.write().remove(table);
    }

    /// Invalidate by multiple tables
    pub fn invalidate_tables(&self, tables: &[String]) {
        for table in tables {
            self.invalidate_table(table);
        }
    }

    /// Invalidate a specific query
    pub fn invalidate(&self, fingerprint: &str) {
        if let Some(entry) = self.cache.write().remove(fingerprint) {
            // Clean up table index
            let mut table_index = self.table_index.write();
            for table in &entry.tables {
                if let Some(fps) = table_index.get_mut(table) {
                    fps.remove(fingerprint);
                }
            }
            self.stats.write().invalidations += 1;
        }
    }

    /// Clear all cached queries
    pub fn clear(&self) {
        self.cache.write().clear();
        self.table_index.write().clear();
    }

    /// Get statistics
    pub fn stats(&self) -> QueryCacheStats {
        self.stats.read().clone()
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    /// Evict entries if over capacity
    fn evict_if_needed(&self) {
        let mut cache = self.cache.write();

        if cache.len() >= self.config.max_entries {
            // Remove expired entries first
            let expired: Vec<String> = cache
                .iter()
                .filter(|(_, e)| e.created.elapsed() > e.ttl)
                .map(|(k, _)| k.clone())
                .collect();

            for key in expired {
                cache.remove(&key);
                self.stats.write().evictions += 1;
            }

            // If still over capacity, remove least used
            if cache.len() >= self.config.max_entries {
                let mut entries: Vec<_> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.hits, v.created))
                    .collect();

                entries.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));

                let to_remove = (self.config.max_entries / 10).max(1);

                for (key, _, _) in entries.into_iter().take(to_remove) {
                    cache.remove(&key);
                    self.stats.write().evictions += 1;
                }
            }
        }
    }
}

/// Normalize SQL for fingerprinting
fn normalize_sql(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut in_string = false;
    let mut string_char = ' ';

    while let Some(c) = chars.next() {
        if in_string {
            if c == string_char && chars.peek() != Some(&string_char) {
                in_string = false;
            }
            result.push('?');
        } else {
            match c {
                '\'' | '"' => {
                    in_string = true;
                    string_char = c;
                    result.push('?');
                }
                '0'..='9' if !result.ends_with(|c: char| c.is_alphanumeric() || c == '_') => {
                    // Replace numeric literals
                    result.push('?');
                    while chars.peek().map_or(false, |c| c.is_numeric() || *c == '.') {
                        chars.next();
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                }
                _ => {
                    result.push(c.to_ascii_uppercase());
                }
            }
        }
    }

    result.trim().to_string()
}

/// Query result wrapper that tracks tables
#[derive(Debug, Clone)]
pub struct TrackedQuery<T> {
    pub result: T,
    pub tables: Vec<String>,
    pub execution_time: Duration,
}

/// Query cache middleware for tracking
pub struct QueryCacheMiddleware {
    cache: Arc<QueryCache>,
    /// Table change subscribers
    subscribers: Arc<RwLock<Vec<Box<dyn Fn(&str) + Send + Sync>>>>,
}

impl QueryCacheMiddleware {
    pub fn new(cache: Arc<QueryCache>) -> Self {
        Self {
            cache,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Subscribe to table change events
    pub fn on_table_change<F>(&self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.subscribers.write().push(Box::new(callback));
    }

    /// Notify table change (called after INSERT/UPDATE/DELETE)
    pub fn notify_table_change(&self, table: &str) {
        // Invalidate cache
        self.cache.invalidate_table(table);

        // Notify subscribers
        for subscriber in self.subscribers.read().iter() {
            subscriber(table);
        }
    }

    /// Parse SQL to extract affected tables
    pub fn extract_tables(&self, sql: &str) -> Vec<String> {
        extract_tables_from_sql(sql)
    }

    /// Check if query is cacheable
    pub fn is_cacheable(&self, sql: &str) -> bool {
        let upper = sql.trim().to_uppercase();

        // Only cache SELECT queries
        if !upper.starts_with("SELECT") {
            return false;
        }

        // Don't cache queries with volatile functions
        let volatile_patterns = [
            "NOW()",
            "CURRENT_TIMESTAMP",
            "RAND()",
            "RANDOM()",
            "UUID()",
            "CURRENT_DATE",
            "CURRENT_TIME",
        ];

        for pattern in &volatile_patterns {
            if upper.contains(pattern) {
                return false;
            }
        }

        true
    }
}

/// Extract table names from SQL
fn extract_tables_from_sql(sql: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let upper = sql.to_uppercase();

    // Simple regex-based extraction
    let from_regex = regex::Regex::new(r"(?i)FROM\s+([`\"]?[\w]+[`\"]?)").unwrap();
    let join_regex = regex::Regex::new(r"(?i)JOIN\s+([`\"]?[\w]+[`\"]?)").unwrap();
    let update_regex = regex::Regex::new(r"(?i)UPDATE\s+([`\"]?[\w]+[`\"]?)").unwrap();
    let insert_regex = regex::Regex::new(r"(?i)INSERT\s+INTO\s+([`\"]?[\w]+[`\"]?)").unwrap();
    let delete_regex = regex::Regex::new(r"(?i)DELETE\s+FROM\s+([`\"]?[\w]+[`\"]?)").unwrap();

    for cap in from_regex.captures_iter(sql) {
        if let Some(m) = cap.get(1) {
            tables.push(m.as_str().trim_matches(|c| c == '`' || c == '"').to_string());
        }
    }

    for cap in join_regex.captures_iter(sql) {
        if let Some(m) = cap.get(1) {
            tables.push(m.as_str().trim_matches(|c| c == '`' || c == '"').to_string());
        }
    }

    for cap in update_regex.captures_iter(sql) {
        if let Some(m) = cap.get(1) {
            tables.push(m.as_str().trim_matches(|c| c == '`' || c == '"').to_string());
        }
    }

    for cap in insert_regex.captures_iter(sql) {
        if let Some(m) = cap.get(1) {
            tables.push(m.as_str().trim_matches(|c| c == '`' || c == '"').to_string());
        }
    }

    for cap in delete_regex.captures_iter(sql) {
        if let Some(m) = cap.get(1) {
            tables.push(m.as_str().trim_matches(|c| c == '`' || c == '"').to_string());
        }
    }

    tables.sort();
    tables.dedup();
    tables
}

/// Prepared statement cache
pub struct PreparedStatementCache {
    statements: Arc<RwLock<HashMap<String, PreparedStatement>>>,
    max_size: usize,
}

#[derive(Clone)]
struct PreparedStatement {
    sql: String,
    tables: Vec<String>,
    created: Instant,
    uses: u64,
}

impl PreparedStatementCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            statements: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub fn get_or_prepare(&self, sql: &str) -> (String, Vec<String>) {
        let fingerprint = blake3::hash(sql.as_bytes()).to_hex()[..16].to_string();

        let mut statements = self.statements.write();

        if let Some(stmt) = statements.get_mut(&fingerprint) {
            stmt.uses += 1;
            return (fingerprint.clone(), stmt.tables.clone());
        }

        // Extract tables
        let tables = extract_tables_from_sql(sql);

        let stmt = PreparedStatement {
            sql: sql.to_string(),
            tables: tables.clone(),
            created: Instant::now(),
            uses: 1,
        };

        // Evict if needed
        if statements.len() >= self.max_size {
            let mut entries: Vec<_> = statements
                .iter()
                .map(|(k, v)| (k.clone(), v.uses, v.created))
                .collect();

            entries.sort_by(|a, b| a.1.cmp(&b.1));

            if let Some((key, _, _)) = entries.first() {
                statements.remove(key);
            }
        }

        statements.insert(fingerprint.clone(), stmt);

        (fingerprint, tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_normalization() {
        let sql1 = "SELECT * FROM users WHERE id = 123";
        let sql2 = "SELECT * FROM users WHERE id = 456";

        let norm1 = normalize_sql(sql1);
        let norm2 = normalize_sql(sql2);

        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_table_extraction() {
        let sql = "SELECT u.*, p.title FROM users u JOIN posts p ON u.id = p.author_id WHERE p.status = 'published'";
        let tables = extract_tables_from_sql(sql);

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"posts".to_string()));
    }

    #[test]
    fn test_query_cache() {
        let cache = QueryCache::new(QueryCacheConfig::default());

        let fingerprint = cache.fingerprint("SELECT * FROM posts WHERE id = ?", &["1"]);

        cache.store(
            fingerprint.clone(),
            &vec!["post1", "post2"],
            vec!["posts".to_string()],
            None,
        ).unwrap();

        let result: Vec<String> = cache.get(&fingerprint).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_table_invalidation() {
        let cache = QueryCache::new(QueryCacheConfig::default());

        let fp1 = "query1".to_string();
        let fp2 = "query2".to_string();

        cache.store(fp1.clone(), &"result1", vec!["posts".to_string()], None).unwrap();
        cache.store(fp2.clone(), &"result2", vec!["users".to_string()], None).unwrap();

        assert!(cache.get::<String>(&fp1).is_ok());
        assert!(cache.get::<String>(&fp2).is_ok());

        cache.invalidate_table("posts");

        assert!(cache.get::<String>(&fp1).is_err());
        assert!(cache.get::<String>(&fp2).is_ok());
    }
}
