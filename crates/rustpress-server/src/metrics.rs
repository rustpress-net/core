//! Prometheus metrics for monitoring.

use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue},
    metrics::{
        counter::Counter,
        family::Family,
        gauge::Gauge,
        histogram::{exponential_buckets, Histogram},
    },
    registry::Registry,
};
use std::sync::Arc;

/// HTTP request labels
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpRequestLabels {
    pub method: String,
    pub path: String,
    pub status: u16,
}

/// Database operation labels
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DbOperationLabels {
    pub operation: String,
    pub table: String,
}

/// Cache operation labels
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CacheLabels {
    pub operation: CacheOperation,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum CacheOperation {
    Hit,
    Miss,
    Set,
    Delete,
}

/// Job queue labels
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct JobLabels {
    pub job_type: String,
    pub status: JobStatus,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

/// Application metrics
#[derive(Clone)]
pub struct Metrics {
    /// Prometheus registry
    pub registry: Arc<Registry>,

    // HTTP metrics
    /// Total HTTP requests
    pub http_requests_total: Family<HttpRequestLabels, Counter>,
    /// HTTP request duration in seconds
    pub http_request_duration_seconds: Family<HttpRequestLabels, Histogram>,
    /// Currently active HTTP connections
    pub http_connections_active: Gauge,

    // Database metrics
    /// Total database queries
    pub db_queries_total: Family<DbOperationLabels, Counter>,
    /// Database query duration in seconds
    pub db_query_duration_seconds: Family<DbOperationLabels, Histogram>,
    /// Active database connections
    pub db_connections_active: Gauge,
    /// Database connection pool size
    pub db_pool_size: Gauge,

    // Cache metrics
    /// Cache operations total
    pub cache_operations_total: Family<CacheLabels, Counter>,
    /// Cache size in bytes
    pub cache_size_bytes: Gauge,
    /// Cache entries count
    pub cache_entries: Gauge,

    // Job metrics
    /// Jobs total by status
    pub jobs_total: Family<JobLabels, Counter>,
    /// Job processing duration in seconds
    pub job_duration_seconds: Family<JobLabels, Histogram>,
    /// Jobs currently in queue
    pub jobs_queued: Gauge,
    /// Jobs currently being processed
    pub jobs_processing: Gauge,

    // Application metrics
    /// Application uptime in seconds
    pub uptime_seconds: Gauge,
    /// Memory usage in bytes
    pub memory_usage_bytes: Gauge,
    /// Active users (sessions)
    pub active_users: Gauge,
    /// Total registered users
    pub total_users: Gauge,
    /// Total posts
    pub total_posts: Gauge,
    /// Total pages
    pub total_pages: Gauge,
    /// Total media files
    pub total_media: Gauge,
}

impl Metrics {
    /// Create a new metrics instance with all counters registered
    pub fn new() -> Self {
        let mut registry = Registry::default();

        // HTTP metrics
        let http_requests_total = Family::<HttpRequestLabels, Counter>::default();
        registry.register(
            "http_requests_total",
            "Total number of HTTP requests",
            http_requests_total.clone(),
        );

        let http_request_duration_seconds =
            Family::<HttpRequestLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.001, 2.0, 16))
            });
        registry.register(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            http_request_duration_seconds.clone(),
        );

        let http_connections_active = Gauge::default();
        registry.register(
            "http_connections_active",
            "Currently active HTTP connections",
            http_connections_active.clone(),
        );

        // Database metrics
        let db_queries_total = Family::<DbOperationLabels, Counter>::default();
        registry.register(
            "db_queries_total",
            "Total number of database queries",
            db_queries_total.clone(),
        );

        let db_query_duration_seconds =
            Family::<DbOperationLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(exponential_buckets(0.0001, 2.0, 16))
            });
        registry.register(
            "db_query_duration_seconds",
            "Database query duration in seconds",
            db_query_duration_seconds.clone(),
        );

        let db_connections_active = Gauge::default();
        registry.register(
            "db_connections_active",
            "Active database connections",
            db_connections_active.clone(),
        );

        let db_pool_size = Gauge::default();
        registry.register(
            "db_pool_size",
            "Database connection pool size",
            db_pool_size.clone(),
        );

        // Cache metrics
        let cache_operations_total = Family::<CacheLabels, Counter>::default();
        registry.register(
            "cache_operations_total",
            "Total cache operations",
            cache_operations_total.clone(),
        );

        let cache_size_bytes = Gauge::default();
        registry.register(
            "cache_size_bytes",
            "Cache size in bytes",
            cache_size_bytes.clone(),
        );

        let cache_entries = Gauge::default();
        registry.register(
            "cache_entries",
            "Number of cache entries",
            cache_entries.clone(),
        );

        // Job metrics
        let jobs_total = Family::<JobLabels, Counter>::default();
        registry.register("jobs_total", "Total jobs by type and status", jobs_total.clone());

        let job_duration_seconds = Family::<JobLabels, Histogram>::new_with_constructor(|| {
            Histogram::new(exponential_buckets(0.1, 2.0, 12))
        });
        registry.register(
            "job_duration_seconds",
            "Job processing duration in seconds",
            job_duration_seconds.clone(),
        );

        let jobs_queued = Gauge::default();
        registry.register("jobs_queued", "Jobs currently in queue", jobs_queued.clone());

        let jobs_processing = Gauge::default();
        registry.register(
            "jobs_processing",
            "Jobs currently being processed",
            jobs_processing.clone(),
        );

        // Application metrics
        let uptime_seconds = Gauge::default();
        registry.register(
            "uptime_seconds",
            "Application uptime in seconds",
            uptime_seconds.clone(),
        );

        let memory_usage_bytes = Gauge::default();
        registry.register(
            "memory_usage_bytes",
            "Memory usage in bytes",
            memory_usage_bytes.clone(),
        );

        let active_users = Gauge::default();
        registry.register(
            "active_users",
            "Currently active users (sessions)",
            active_users.clone(),
        );

        let total_users = Gauge::default();
        registry.register(
            "total_users",
            "Total registered users",
            total_users.clone(),
        );

        let total_posts = Gauge::default();
        registry.register("total_posts", "Total posts", total_posts.clone());

        let total_pages = Gauge::default();
        registry.register("total_pages", "Total pages", total_pages.clone());

        let total_media = Gauge::default();
        registry.register("total_media", "Total media files", total_media.clone());

        Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_request_duration_seconds,
            http_connections_active,
            db_queries_total,
            db_query_duration_seconds,
            db_connections_active,
            db_pool_size,
            cache_operations_total,
            cache_size_bytes,
            cache_entries,
            jobs_total,
            job_duration_seconds,
            jobs_queued,
            jobs_processing,
            uptime_seconds,
            memory_usage_bytes,
            active_users,
            total_users,
            total_posts,
            total_pages,
            total_media,
        }
    }

    /// Record an HTTP request
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        let labels = HttpRequestLabels {
            method: method.to_string(),
            path: normalize_path(path),
            status,
        };

        self.http_requests_total.get_or_create(&labels).inc();
        self.http_request_duration_seconds
            .get_or_create(&labels)
            .observe(duration_secs);
    }

    /// Record a database query
    pub fn record_db_query(&self, operation: &str, table: &str, duration_secs: f64) {
        let labels = DbOperationLabels {
            operation: operation.to_string(),
            table: table.to_string(),
        };

        self.db_queries_total.get_or_create(&labels).inc();
        self.db_query_duration_seconds
            .get_or_create(&labels)
            .observe(duration_secs);
    }

    /// Record a cache operation
    pub fn record_cache_operation(&self, operation: CacheOperation) {
        let labels = CacheLabels { operation };
        self.cache_operations_total.get_or_create(&labels).inc();
    }

    /// Record a job
    pub fn record_job(&self, job_type: &str, status: JobStatus, duration_secs: Option<f64>) {
        let labels = JobLabels {
            job_type: job_type.to_string(),
            status,
        };

        self.jobs_total.get_or_create(&labels).inc();
        if let Some(duration) = duration_secs {
            self.job_duration_seconds
                .get_or_create(&labels)
                .observe(duration);
        }
    }

    /// Encode metrics to Prometheus format
    pub fn encode(&self) -> String {
        let mut buffer = String::new();
        prometheus_client::encoding::text::encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize path for metrics (replace IDs with placeholders)
fn normalize_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = parts
        .into_iter()
        .map(|part| {
            // Replace UUIDs with :id
            if uuid::Uuid::parse_str(part).is_ok() {
                ":id".to_string()
            }
            // Replace numeric IDs with :id
            else if part.parse::<i64>().is_ok() && !part.is_empty() {
                ":id".to_string()
            } else {
                part.to_string()
            }
        })
        .collect();
    normalized.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.encode().contains("http_requests_total"));
    }

    #[test]
    fn test_record_http_request() {
        let metrics = Metrics::new();
        metrics.record_http_request("GET", "/api/v1/posts", 200, 0.05);

        let encoded = metrics.encode();
        assert!(encoded.contains("http_requests_total"));
    }

    #[test]
    fn test_path_normalization() {
        assert_eq!(normalize_path("/api/v1/posts"), "/api/v1/posts");
        assert_eq!(
            normalize_path("/api/v1/posts/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/posts/:id"
        );
        assert_eq!(normalize_path("/api/v1/posts/123"), "/api/v1/posts/:id");
    }

    #[test]
    fn test_cache_operations() {
        let metrics = Metrics::new();
        metrics.record_cache_operation(CacheOperation::Hit);
        metrics.record_cache_operation(CacheOperation::Miss);

        let encoded = metrics.encode();
        assert!(encoded.contains("cache_operations_total"));
    }
}
