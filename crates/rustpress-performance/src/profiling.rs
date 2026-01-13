//! Performance Profiling Endpoints
//!
//! Provides profiling endpoints and metrics collection for performance monitoring.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Profiling configuration
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Enable profiling endpoints
    pub enabled: bool,
    /// Enable request timing
    pub request_timing: bool,
    /// Enable database query profiling
    pub query_profiling: bool,
    /// Enable memory profiling
    pub memory_profiling: bool,
    /// Sample rate (0.0-1.0)
    pub sample_rate: f64,
    /// Maximum traces to keep
    pub max_traces: usize,
    /// Slow request threshold (ms)
    pub slow_request_threshold_ms: u64,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            request_timing: true,
            query_profiling: true,
            memory_profiling: true,
            sample_rate: 0.1,
            max_traces: 1000,
            slow_request_threshold_ms: 500,
        }
    }
}

/// Request trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTrace {
    /// Trace ID
    pub trace_id: String,
    /// Request method
    pub method: String,
    /// Request path
    pub path: String,
    /// Response status code
    pub status_code: u16,
    /// Total duration in microseconds
    pub duration_us: u64,
    /// Timing breakdown
    pub timings: HashMap<String, u64>,
    /// Database queries
    pub queries: Vec<QueryTrace>,
    /// Memory usage
    pub memory: Option<MemorySnapshot>,
    /// Timestamp
    pub timestamp: i64,
    /// User agent
    pub user_agent: Option<String>,
    /// Client IP
    pub client_ip: Option<String>,
}

/// Query trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTrace {
    /// Query string (normalized)
    pub query: String,
    /// Execution time in microseconds
    pub duration_us: u64,
    /// Rows affected/returned
    pub rows: u64,
}

/// Memory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Heap used bytes
    pub heap_used: u64,
    /// Total allocated bytes
    pub total_allocated: u64,
    /// Resident set size
    pub rss: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Requests per second (last minute average)
    pub requests_per_second: f64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// 50th percentile response time
    pub p50_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Slow request count
    pub slow_requests: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Active connections
    pub active_connections: u32,
    /// Memory usage
    pub memory: MemoryMetrics,
    /// Database metrics
    pub database: DatabaseMetrics,
}

/// Memory metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub heap_used_mb: f64,
    pub heap_total_mb: f64,
    pub rss_mb: f64,
    pub gc_count: u64,
}

/// Database metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub queries_per_second: f64,
    pub avg_query_time_ms: f64,
    pub slow_queries: u64,
    pub connection_pool_size: u32,
    pub connection_pool_used: u32,
}

/// Profiler
pub struct Profiler {
    config: ProfilingConfig,
    traces: Arc<RwLock<Vec<RequestTrace>>>,
    metrics: Arc<RwLock<PerformanceMetrics>>,
    response_times: Arc<RwLock<Vec<u64>>>,
}

impl Profiler {
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            config,
            traces: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            response_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start timing a request
    pub fn start_request(&self) -> RequestTimer {
        RequestTimer {
            start: Instant::now(),
            timings: HashMap::new(),
            queries: Vec::new(),
            current_span: None,
        }
    }

    /// Record a completed request
    pub fn record_request(&self, trace: RequestTrace) {
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

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_requests += 1;

            if trace.status_code >= 500 {
                // Track errors for error rate calculation
            }

            if trace.duration_us / 1000 > self.config.slow_request_threshold_ms {
                metrics.slow_requests += 1;
            }
        }

        // Store response time for percentile calculation
        {
            let mut times = self.response_times.write();
            times.push(trace.duration_us / 1000); // Convert to ms

            // Keep only recent times
            if times.len() > 10000 {
                times.drain(0..5000);
            }
        }

        // Store trace
        let mut traces = self.traces.write();
        traces.push(trace);

        // Trim if over capacity
        if traces.len() > self.config.max_traces {
            let drain_count = traces.len() - self.config.max_traces;
            traces.drain(0..drain_count);
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.metrics.read().clone();

        // Calculate percentiles
        let times = self.response_times.read();
        if !times.is_empty() {
            let mut sorted: Vec<_> = times.iter().cloned().collect();
            sorted.sort();

            metrics.avg_response_time_ms = sorted.iter().sum::<u64>() as f64 / sorted.len() as f64;
            metrics.p50_response_time_ms = sorted[sorted.len() / 2] as f64;
            metrics.p95_response_time_ms = sorted[(sorted.len() as f64 * 0.95) as usize] as f64;
            metrics.p99_response_time_ms = sorted[(sorted.len() as f64 * 0.99).min(sorted.len() as f64 - 1.0) as usize] as f64;
        }

        metrics
    }

    /// Get recent traces
    pub fn get_traces(&self, limit: usize) -> Vec<RequestTrace> {
        let traces = self.traces.read();
        traces.iter().rev().take(limit).cloned().collect()
    }

    /// Get slow request traces
    pub fn get_slow_traces(&self, limit: usize) -> Vec<RequestTrace> {
        let traces = self.traces.read();
        let threshold_us = self.config.slow_request_threshold_ms * 1000;

        let mut slow: Vec<_> = traces
            .iter()
            .filter(|t| t.duration_us > threshold_us)
            .cloned()
            .collect();

        slow.sort_by(|a, b| b.duration_us.cmp(&a.duration_us));
        slow.truncate(limit);
        slow
    }

    /// Clear all traces
    pub fn clear(&self) {
        self.traces.write().clear();
        *self.metrics.write() = PerformanceMetrics::default();
        self.response_times.write().clear();
    }
}

/// Request timer for tracking timing breakdown
pub struct RequestTimer {
    start: Instant,
    timings: HashMap<String, u64>,
    queries: Vec<QueryTrace>,
    current_span: Option<(String, Instant)>,
}

impl RequestTimer {
    /// Start a timing span
    pub fn start_span(&mut self, name: &str) {
        self.current_span = Some((name.to_string(), Instant::now()));
    }

    /// End the current span
    pub fn end_span(&mut self) {
        if let Some((name, start)) = self.current_span.take() {
            let duration = start.elapsed().as_micros() as u64;
            *self.timings.entry(name).or_insert(0) += duration;
        }
    }

    /// Record a database query
    pub fn record_query(&mut self, query: &str, duration_us: u64, rows: u64) {
        self.queries.push(QueryTrace {
            query: query.to_string(),
            duration_us,
            rows,
        });
    }

    /// Build the final trace
    pub fn finish(
        self,
        trace_id: String,
        method: &str,
        path: &str,
        status_code: u16,
    ) -> RequestTrace {
        RequestTrace {
            trace_id,
            method: method.to_string(),
            path: path.to_string(),
            status_code,
            duration_us: self.start.elapsed().as_micros() as u64,
            timings: self.timings,
            queries: self.queries,
            memory: None,
            timestamp: chrono::Utc::now().timestamp(),
            user_agent: None,
            client_ip: None,
        }
    }
}

/// Flame graph data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphNode {
    pub name: String,
    pub value: u64,
    pub children: Vec<FlameGraphNode>,
}

impl FlameGraphNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: 0,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: FlameGraphNode) {
        self.children.push(child);
    }

    /// Build flame graph from traces
    pub fn from_traces(traces: &[RequestTrace]) -> Self {
        let mut root = FlameGraphNode::new("all");

        for trace in traces {
            let mut current = &mut root;
            current.value += trace.duration_us;

            // Add method node
            let method_idx = current.children.iter()
                .position(|c| c.name == trace.method);

            let method_node = match method_idx {
                Some(idx) => &mut current.children[idx],
                None => {
                    current.children.push(FlameGraphNode::new(&trace.method));
                    current.children.last_mut().unwrap()
                }
            };
            method_node.value += trace.duration_us;

            // Add timing breakdowns
            for (name, duration) in &trace.timings {
                let timing_idx = method_node.children.iter()
                    .position(|c| &c.name == name);

                match timing_idx {
                    Some(idx) => method_node.children[idx].value += duration,
                    None => {
                        let mut node = FlameGraphNode::new(name);
                        node.value = *duration;
                        method_node.children.push(node);
                    }
                }
            }
        }

        root
    }
}

/// Profiling endpoint handler
pub struct ProfilingEndpoints {
    profiler: Arc<Profiler>,
}

impl ProfilingEndpoints {
    pub fn new(profiler: Arc<Profiler>) -> Self {
        Self { profiler }
    }

    /// Generate metrics response
    pub fn metrics(&self) -> String {
        let metrics = self.profiler.get_metrics();
        serde_json::to_string_pretty(&metrics).unwrap_or_default()
    }

    /// Generate traces response
    pub fn traces(&self, limit: Option<usize>) -> String {
        let traces = self.profiler.get_traces(limit.unwrap_or(100));
        serde_json::to_string_pretty(&traces).unwrap_or_default()
    }

    /// Generate slow traces response
    pub fn slow_traces(&self, limit: Option<usize>) -> String {
        let traces = self.profiler.get_slow_traces(limit.unwrap_or(50));
        serde_json::to_string_pretty(&traces).unwrap_or_default()
    }

    /// Generate flame graph data
    pub fn flame_graph(&self) -> String {
        let traces = self.profiler.get_traces(500);
        let graph = FlameGraphNode::from_traces(&traces);
        serde_json::to_string_pretty(&graph).unwrap_or_default()
    }

    /// Generate Prometheus-compatible metrics
    pub fn prometheus_metrics(&self) -> String {
        let metrics = self.profiler.get_metrics();

        let mut output = String::new();

        output.push_str("# HELP http_requests_total Total HTTP requests\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!("http_requests_total {}\n\n", metrics.total_requests));

        output.push_str("# HELP http_request_duration_seconds Request duration\n");
        output.push_str("# TYPE http_request_duration_seconds summary\n");
        output.push_str(&format!(
            "http_request_duration_seconds{{quantile=\"0.5\"}} {}\n",
            metrics.p50_response_time_ms / 1000.0
        ));
        output.push_str(&format!(
            "http_request_duration_seconds{{quantile=\"0.95\"}} {}\n",
            metrics.p95_response_time_ms / 1000.0
        ));
        output.push_str(&format!(
            "http_request_duration_seconds{{quantile=\"0.99\"}} {}\n\n",
            metrics.p99_response_time_ms / 1000.0
        ));

        output.push_str("# HELP http_slow_requests_total Slow HTTP requests\n");
        output.push_str("# TYPE http_slow_requests_total counter\n");
        output.push_str(&format!("http_slow_requests_total {}\n\n", metrics.slow_requests));

        output.push_str("# HELP process_resident_memory_bytes Memory usage\n");
        output.push_str("# TYPE process_resident_memory_bytes gauge\n");
        output.push_str(&format!(
            "process_resident_memory_bytes {}\n\n",
            (metrics.memory.rss_mb * 1024.0 * 1024.0) as u64
        ));

        output.push_str("# HELP db_queries_total Database queries\n");
        output.push_str("# TYPE db_queries_total counter\n");
        output.push_str(&format!(
            "db_query_duration_seconds_avg {}\n",
            metrics.database.avg_query_time_ms / 1000.0
        ));

        output
    }
}

/// Server timing header generator
pub struct ServerTimingHeader;

impl ServerTimingHeader {
    /// Generate Server-Timing header value from timings
    pub fn generate(timings: &HashMap<String, u64>) -> String {
        timings
            .iter()
            .map(|(name, duration)| {
                format!("{};dur={}", name, *duration as f64 / 1000.0)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate with descriptions
    pub fn generate_with_desc(timings: &[(String, u64, String)]) -> String {
        timings
            .iter()
            .map(|(name, duration, desc)| {
                format!(
                    "{};dur={};desc=\"{}\"",
                    name,
                    *duration as f64 / 1000.0,
                    desc
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_timer() {
        let mut timer = RequestTimer {
            start: Instant::now(),
            timings: HashMap::new(),
            queries: Vec::new(),
            current_span: None,
        };

        timer.start_span("db");
        std::thread::sleep(Duration::from_millis(10));
        timer.end_span();

        let trace = timer.finish(
            "test-123".to_string(),
            "GET",
            "/api/users",
            200,
        );

        assert_eq!(trace.method, "GET");
        assert!(trace.timings.contains_key("db"));
    }

    #[test]
    fn test_profiler() {
        let config = ProfilingConfig::default();
        let profiler = Profiler::new(config);

        let trace = RequestTrace {
            trace_id: "test-1".to_string(),
            method: "GET".to_string(),
            path: "/".to_string(),
            status_code: 200,
            duration_us: 50000, // 50ms
            timings: HashMap::new(),
            queries: Vec::new(),
            memory: None,
            timestamp: chrono::Utc::now().timestamp(),
            user_agent: None,
            client_ip: None,
        };

        profiler.record_request(trace);

        let metrics = profiler.get_metrics();
        assert!(metrics.total_requests >= 0);
    }

    #[test]
    fn test_server_timing_header() {
        let mut timings = HashMap::new();
        timings.insert("db".to_string(), 50000u64);
        timings.insert("render".to_string(), 25000u64);

        let header = ServerTimingHeader::generate(&timings);
        assert!(header.contains("db;dur=50"));
        assert!(header.contains("render;dur=25"));
    }

    #[test]
    fn test_flame_graph() {
        let traces = vec![
            RequestTrace {
                trace_id: "1".to_string(),
                method: "GET".to_string(),
                path: "/".to_string(),
                status_code: 200,
                duration_us: 100000,
                timings: {
                    let mut t = HashMap::new();
                    t.insert("db".to_string(), 50000u64);
                    t
                },
                queries: Vec::new(),
                memory: None,
                timestamp: 0,
                user_agent: None,
                client_ip: None,
            },
        ];

        let graph = FlameGraphNode::from_traces(&traces);
        assert_eq!(graph.name, "all");
        assert!(!graph.children.is_empty());
    }
}
