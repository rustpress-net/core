//! Database Query Optimization Logging
//!
//! Logs and analyzes database queries for performance optimization opportunities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Query log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogEntry {
    /// The SQL query (normalized)
    pub query: String,
    /// Original query with parameters
    pub original_query: String,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Number of rows affected/returned
    pub rows_affected: u64,
    /// Timestamp when query was executed
    pub timestamp: i64,
    /// Call stack trace (if enabled)
    pub stack_trace: Option<String>,
    /// Query plan (if EXPLAIN was run)
    pub query_plan: Option<String>,
    /// Whether this query used an index
    pub used_index: Option<bool>,
    /// Tables accessed by this query
    pub tables: Vec<String>,
}

/// Query statistics aggregated over time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryStats {
    /// Number of times this query was executed
    pub execution_count: u64,
    /// Total execution time in microseconds
    pub total_time_us: u64,
    /// Minimum execution time
    pub min_time_us: u64,
    /// Maximum execution time
    pub max_time_us: u64,
    /// Average execution time
    pub avg_time_us: u64,
    /// Total rows affected/returned
    pub total_rows: u64,
    /// First seen timestamp
    pub first_seen: i64,
    /// Last seen timestamp
    pub last_seen: i64,
}

impl QueryStats {
    pub fn new() -> Self {
        Self {
            min_time_us: u64::MAX,
            ..Default::default()
        }
    }

    pub fn record(&mut self, execution_time_us: u64, rows: u64, timestamp: i64) {
        self.execution_count += 1;
        self.total_time_us += execution_time_us;
        self.min_time_us = self.min_time_us.min(execution_time_us);
        self.max_time_us = self.max_time_us.max(execution_time_us);
        self.avg_time_us = self.total_time_us / self.execution_count;
        self.total_rows += rows;

        if self.first_seen == 0 {
            self.first_seen = timestamp;
        }
        self.last_seen = timestamp;
    }
}

/// Query optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Type of optimization
    pub suggestion_type: SuggestionType,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: String,
    /// Impact level (1-10)
    pub impact_score: u8,
    /// The query this applies to
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionType {
    MissingIndex,
    NplusOne,
    FullTableScan,
    SlowQuery,
    UnusedIndex,
    SelectStar,
    NoLimit,
    ImplicitConversion,
    SuboptimalJoin,
    DuplicateQuery,
}

/// Query logger configuration
#[derive(Debug, Clone)]
pub struct QueryLoggerConfig {
    /// Enable query logging
    pub enabled: bool,
    /// Log queries slower than this threshold (microseconds)
    pub slow_query_threshold_us: u64,
    /// Maximum number of entries to keep in memory
    pub max_entries: usize,
    /// Enable stack trace capture
    pub capture_stack_traces: bool,
    /// Enable EXPLAIN analysis for slow queries
    pub analyze_slow_queries: bool,
    /// Log all queries (not just slow ones)
    pub log_all_queries: bool,
    /// Sample rate for logging (0.0-1.0)
    pub sample_rate: f64,
}

impl Default for QueryLoggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            slow_query_threshold_us: 100_000, // 100ms
            max_entries: 10000,
            capture_stack_traces: false,
            analyze_slow_queries: true,
            log_all_queries: false,
            sample_rate: 1.0,
        }
    }
}

/// Database query logger and analyzer
pub struct QueryLogger {
    config: QueryLoggerConfig,
    /// Recent query log entries
    entries: Arc<RwLock<Vec<QueryLogEntry>>>,
    /// Aggregated query statistics by normalized query
    stats: Arc<RwLock<HashMap<String, QueryStats>>>,
    /// N+1 query detection: pattern -> count in recent window
    nplusone_detector: Arc<RwLock<HashMap<String, NplusOnePattern>>>,
    /// Generated optimization suggestions
    suggestions: Arc<RwLock<Vec<OptimizationSuggestion>>>,
}

#[derive(Debug, Clone)]
struct NplusOnePattern {
    query_pattern: String,
    occurrences: Vec<i64>, // timestamps
    tables: Vec<String>,
}

impl QueryLogger {
    pub fn new(config: QueryLoggerConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
            nplusone_detector: Arc::new(RwLock::new(HashMap::new())),
            suggestions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log a query execution
    pub fn log_query(&self, entry: QueryLogEntry) {
        if !self.config.enabled {
            return;
        }

        // Sample rate check
        if self.config.sample_rate < 1.0 {
            let random: f64 = rand::random();
            if random > self.config.sample_rate {
                return;
            }
        }

        let is_slow = entry.execution_time_us >= self.config.slow_query_threshold_us;

        // Only log if slow or log_all_queries is enabled
        if !is_slow && !self.config.log_all_queries {
            // Still update stats even if not logging
            self.update_stats(&entry);
            return;
        }

        // Update statistics
        self.update_stats(&entry);

        // Check for N+1 pattern
        self.detect_nplusone(&entry);

        // Store entry
        let mut entries = self.entries.write();
        entries.push(entry.clone());

        // Trim if over capacity
        if entries.len() > self.config.max_entries {
            entries.drain(0..entries.len() - self.config.max_entries);
        }

        // Generate suggestions for slow queries
        if is_slow {
            self.analyze_slow_query(&entry);
        }
    }

    /// Start timing a query
    pub fn start_query(&self) -> QueryTimer {
        QueryTimer {
            start: Instant::now(),
        }
    }

    /// Update aggregated statistics
    fn update_stats(&self, entry: &QueryLogEntry) {
        let mut stats = self.stats.write();
        let query_stats = stats.entry(entry.query.clone()).or_insert_with(QueryStats::new);
        query_stats.record(entry.execution_time_us, entry.rows_affected, entry.timestamp);
    }

    /// Detect N+1 query patterns
    fn detect_nplusone(&self, entry: &QueryLogEntry) {
        let pattern = self.extract_query_pattern(&entry.query);
        let mut detector = self.nplusone_detector.write();

        let nplusone = detector.entry(pattern.clone()).or_insert_with(|| NplusOnePattern {
            query_pattern: pattern,
            occurrences: Vec::new(),
            tables: entry.tables.clone(),
        });

        nplusone.occurrences.push(entry.timestamp);

        // Keep only recent occurrences (last 1 second)
        let cutoff = entry.timestamp - 1000;
        nplusone.occurrences.retain(|t| *t > cutoff);

        // If more than 5 similar queries in 1 second, flag as N+1
        if nplusone.occurrences.len() > 5 {
            let mut suggestions = self.suggestions.write();

            // Check if we already have this suggestion
            let exists = suggestions.iter().any(|s| {
                s.suggestion_type == SuggestionType::NplusOne && s.query == entry.query
            });

            if !exists {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::NplusOne,
                    description: format!(
                        "Detected N+1 query pattern: {} similar queries executed in rapid succession",
                        nplusone.occurrences.len()
                    ),
                    suggested_fix: format!(
                        "Consider using eager loading or a JOIN to fetch related data in a single query. Tables: {:?}",
                        nplusone.tables
                    ),
                    impact_score: 8,
                    query: entry.query.clone(),
                });
            }
        }
    }

    /// Extract a pattern from a query (removes specific values)
    fn extract_query_pattern(&self, query: &str) -> String {
        // Simple pattern extraction - replace numeric values and quoted strings
        let mut pattern = query.to_string();

        // Replace numbers
        pattern = regex::Regex::new(r"\b\d+\b")
            .unwrap()
            .replace_all(&pattern, "?")
            .to_string();

        // Replace single-quoted strings
        pattern = regex::Regex::new(r"'[^']*'")
            .unwrap()
            .replace_all(&pattern, "?")
            .to_string();

        // Replace double-quoted strings
        pattern = regex::Regex::new(r#""[^"]*""#)
            .unwrap()
            .replace_all(&pattern, "?")
            .to_string();

        pattern
    }

    /// Analyze a slow query for optimization opportunities
    fn analyze_slow_query(&self, entry: &QueryLogEntry) {
        let mut suggestions = self.suggestions.write();
        let query_upper = entry.query.to_uppercase();

        // Check for SELECT *
        if query_upper.contains("SELECT *") {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::SelectStar,
                description: "Query uses SELECT * which fetches all columns".to_string(),
                suggested_fix: "Specify only the columns you need to reduce data transfer".to_string(),
                impact_score: 4,
                query: entry.query.clone(),
            });
        }

        // Check for missing LIMIT on large result sets
        if !query_upper.contains("LIMIT") && entry.rows_affected > 100 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::NoLimit,
                description: format!(
                    "Query returned {} rows without LIMIT clause",
                    entry.rows_affected
                ),
                suggested_fix: "Add LIMIT clause if you don't need all rows".to_string(),
                impact_score: 5,
                query: entry.query.clone(),
            });
        }

        // Check for full table scan indication
        if let Some(ref plan) = entry.query_plan {
            if plan.contains("Seq Scan") || plan.contains("full table scan") {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: SuggestionType::FullTableScan,
                    description: "Query performs a full table scan".to_string(),
                    suggested_fix: "Consider adding an index on the columns used in WHERE clause".to_string(),
                    impact_score: 7,
                    query: entry.query.clone(),
                });
            }
        }

        // Check if query used an index
        if entry.used_index == Some(false) {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::MissingIndex,
                description: "Query did not use any index".to_string(),
                suggested_fix: "Analyze the query and add appropriate indexes".to_string(),
                impact_score: 8,
                query: entry.query.clone(),
            });
        }

        // General slow query warning
        if entry.execution_time_us >= self.config.slow_query_threshold_us {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::SlowQuery,
                description: format!(
                    "Query took {}ms to execute",
                    entry.execution_time_us / 1000
                ),
                suggested_fix: "Review query plan and consider optimization strategies".to_string(),
                impact_score: 6,
                query: entry.query.clone(),
            });
        }
    }

    /// Get recent log entries
    pub fn get_entries(&self, limit: usize) -> Vec<QueryLogEntry> {
        let entries = self.entries.read();
        entries.iter().rev().take(limit).cloned().collect()
    }

    /// Get slow queries
    pub fn get_slow_queries(&self, limit: usize) -> Vec<QueryLogEntry> {
        let entries = self.entries.read();
        let mut slow: Vec<_> = entries
            .iter()
            .filter(|e| e.execution_time_us >= self.config.slow_query_threshold_us)
            .cloned()
            .collect();
        slow.sort_by(|a, b| b.execution_time_us.cmp(&a.execution_time_us));
        slow.truncate(limit);
        slow
    }

    /// Get query statistics
    pub fn get_stats(&self) -> HashMap<String, QueryStats> {
        self.stats.read().clone()
    }

    /// Get top queries by total time
    pub fn get_top_queries_by_time(&self, limit: usize) -> Vec<(String, QueryStats)> {
        let stats = self.stats.read();
        let mut queries: Vec<_> = stats.iter()
            .map(|(q, s)| (q.clone(), s.clone()))
            .collect();
        queries.sort_by(|a, b| b.1.total_time_us.cmp(&a.1.total_time_us));
        queries.truncate(limit);
        queries
    }

    /// Get top queries by execution count
    pub fn get_top_queries_by_count(&self, limit: usize) -> Vec<(String, QueryStats)> {
        let stats = self.stats.read();
        let mut queries: Vec<_> = stats.iter()
            .map(|(q, s)| (q.clone(), s.clone()))
            .collect();
        queries.sort_by(|a, b| b.1.execution_count.cmp(&a.1.execution_count));
        queries.truncate(limit);
        queries
    }

    /// Get optimization suggestions
    pub fn get_suggestions(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = self.suggestions.read().clone();
        suggestions.sort_by(|a, b| b.impact_score.cmp(&a.impact_score));
        suggestions
    }

    /// Clear all logs and statistics
    pub fn clear(&self) {
        self.entries.write().clear();
        self.stats.write().clear();
        self.nplusone_detector.write().clear();
        self.suggestions.write().clear();
    }

    /// Get summary report
    pub fn get_summary(&self) -> QueryLogSummary {
        let entries = self.entries.read();
        let stats = self.stats.read();
        let suggestions = self.suggestions.read();

        let total_queries = entries.len();
        let total_time: u64 = entries.iter().map(|e| e.execution_time_us).sum();
        let slow_queries = entries
            .iter()
            .filter(|e| e.execution_time_us >= self.config.slow_query_threshold_us)
            .count();

        QueryLogSummary {
            total_queries,
            unique_queries: stats.len(),
            total_time_us: total_time,
            avg_time_us: if total_queries > 0 { total_time / total_queries as u64 } else { 0 },
            slow_queries,
            suggestion_count: suggestions.len(),
            nplusone_detected: suggestions.iter()
                .filter(|s| s.suggestion_type == SuggestionType::NplusOne)
                .count(),
        }
    }
}

/// Query timer for measuring execution time
pub struct QueryTimer {
    start: Instant,
}

impl QueryTimer {
    /// Finish timing and return duration in microseconds
    pub fn finish(self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

/// Summary of query log data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogSummary {
    pub total_queries: usize,
    pub unique_queries: usize,
    pub total_time_us: u64,
    pub avg_time_us: u64,
    pub slow_queries: usize,
    pub suggestion_count: usize,
    pub nplusone_detected: usize,
}

/// Query log middleware for automatic logging
pub struct QueryLogMiddleware {
    logger: Arc<QueryLogger>,
}

impl QueryLogMiddleware {
    pub fn new(logger: Arc<QueryLogger>) -> Self {
        Self { logger }
    }

    /// Wrap a query execution with logging
    pub async fn execute<F, T, E>(&self, query: &str, tables: Vec<String>, f: F) -> Result<(T, u64), E>
    where
        F: std::future::Future<Output = Result<(T, u64), E>>,
    {
        let timer = self.logger.start_query();
        let result = f.await;
        let execution_time = timer.finish();

        match &result {
            Ok((_, rows)) => {
                let entry = QueryLogEntry {
                    query: normalize_query(query),
                    original_query: query.to_string(),
                    execution_time_us: execution_time,
                    rows_affected: *rows,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    stack_trace: None,
                    query_plan: None,
                    used_index: None,
                    tables,
                };
                self.logger.log_query(entry);
            }
            Err(_) => {}
        }

        result
    }
}

/// Normalize a SQL query for grouping
fn normalize_query(query: &str) -> String {
    let mut normalized = query.trim().to_uppercase();

    // Remove extra whitespace
    normalized = regex::Regex::new(r"\s+")
        .unwrap()
        .replace_all(&normalized, " ")
        .to_string();

    // Replace IN lists with placeholder
    normalized = regex::Regex::new(r"IN\s*\([^)]+\)")
        .unwrap()
        .replace_all(&normalized, "IN (?)")
        .to_string();

    // Replace numeric values
    normalized = regex::Regex::new(r"\b\d+\b")
        .unwrap()
        .replace_all(&normalized, "?")
        .to_string();

    // Replace string values
    normalized = regex::Regex::new(r"'[^']*'")
        .unwrap()
        .replace_all(&normalized, "?")
        .to_string();

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_normalization() {
        let query = "SELECT * FROM users WHERE id = 123 AND name = 'John'";
        let normalized = normalize_query(query);
        assert!(normalized.contains("?"));
        assert!(!normalized.contains("123"));
        assert!(!normalized.contains("John"));
    }

    #[test]
    fn test_query_stats() {
        let mut stats = QueryStats::new();
        stats.record(100, 10, 1000);
        stats.record(200, 20, 2000);

        assert_eq!(stats.execution_count, 2);
        assert_eq!(stats.total_time_us, 300);
        assert_eq!(stats.min_time_us, 100);
        assert_eq!(stats.max_time_us, 200);
        assert_eq!(stats.avg_time_us, 150);
    }

    #[test]
    fn test_slow_query_detection() {
        let config = QueryLoggerConfig {
            slow_query_threshold_us: 1000,
            log_all_queries: true,
            ..Default::default()
        };
        let logger = QueryLogger::new(config);

        let entry = QueryLogEntry {
            query: "SELECT * FROM large_table".to_string(),
            original_query: "SELECT * FROM large_table".to_string(),
            execution_time_us: 5000,
            rows_affected: 1000,
            timestamp: chrono::Utc::now().timestamp_millis(),
            stack_trace: None,
            query_plan: None,
            used_index: Some(false),
            tables: vec!["large_table".to_string()],
        };

        logger.log_query(entry);

        let suggestions = logger.get_suggestions();
        assert!(!suggestions.is_empty());
    }
}
