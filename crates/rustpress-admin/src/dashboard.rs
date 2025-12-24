//! Dashboard data models and services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

/// Main dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// System status
    pub system: SystemStatus,

    /// Application status
    pub app: AppStatus,

    /// Database status
    pub database: DatabaseStatus,

    /// Cache status
    pub cache: CacheStatus,

    /// CDN status
    pub cdn: Option<CdnStatus>,

    /// Recent activity
    pub activity: Vec<ActivityItem>,

    /// Quick stats
    pub stats: QuickStats,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// CPU usage percentage
    pub cpu_usage: f32,

    /// Memory usage percentage
    pub memory_usage: f32,

    /// Memory used in bytes
    pub memory_used: u64,

    /// Memory total in bytes
    pub memory_total: u64,

    /// Disk usage percentage
    pub disk_usage: f32,

    /// Disk used in bytes
    pub disk_used: u64,

    /// Disk total in bytes
    pub disk_total: u64,

    /// System uptime in seconds
    pub uptime: u64,

    /// Load average (1, 5, 15 min)
    pub load_average: Option<[f64; 3]>,

    /// OS name
    pub os_name: String,

    /// OS version
    pub os_version: String,

    /// Hostname
    pub hostname: String,
}

impl SystemStatus {
    /// Collect current system status
    pub fn collect() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        std::thread::sleep(std::time::Duration::from_millis(200));
        sys.refresh_cpu_usage();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Get disk info
        let disks = Disks::new_with_refreshed_list();
        let (disk_used, disk_total) = disks
            .list()
            .iter()
            .filter(|d| {
                let mount = d.mount_point().to_string_lossy();
                mount == "/" || mount == "C:\\"
            })
            .map(|d| (d.total_space() - d.available_space(), d.total_space()))
            .next()
            .unwrap_or((0, 0));

        let disk_usage = if disk_total > 0 {
            (disk_used as f32 / disk_total as f32) * 100.0
        } else {
            0.0
        };

        #[cfg(unix)]
        let load_average = {
            let load = System::load_average();
            Some([load.one, load.five, load.fifteen])
        };

        #[cfg(not(unix))]
        let load_average = None;

        Self {
            cpu_usage,
            memory_usage,
            memory_used,
            memory_total,
            disk_usage,
            disk_used,
            disk_total,
            uptime: System::uptime(),
            load_average,
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        }
    }
}

/// Application status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    /// Application version
    pub version: String,

    /// Environment (development, staging, production)
    pub environment: String,

    /// Application uptime in seconds
    pub uptime: u64,

    /// Number of active requests
    pub active_requests: u64,

    /// Requests per minute
    pub requests_per_minute: f64,

    /// Average response time in ms
    pub avg_response_time: f64,

    /// Error rate percentage
    pub error_rate: f64,

    /// Start time
    pub started_at: DateTime<Utc>,

    /// Last deployment
    pub last_deployed: Option<DateTime<Utc>>,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("RUSTPRESS_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            uptime: 0,
            active_requests: 0,
            requests_per_minute: 0.0,
            avg_response_time: 0.0,
            error_rate: 0.0,
            started_at: Utc::now(),
            last_deployed: None,
        }
    }
}

/// Database status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    /// Connection status
    pub connected: bool,

    /// Database type
    pub db_type: String,

    /// Database version
    pub version: String,

    /// Database size in bytes
    pub size: u64,

    /// Number of tables
    pub table_count: u64,

    /// Active connections
    pub active_connections: u32,

    /// Max connections
    pub max_connections: u32,

    /// Pool size
    pub pool_size: u32,

    /// Idle connections
    pub idle_connections: u32,

    /// Recent queries per second
    pub queries_per_second: f64,

    /// Average query time in ms
    pub avg_query_time: f64,

    /// Slow queries count (last hour)
    pub slow_queries: u64,
}

impl Default for DatabaseStatus {
    fn default() -> Self {
        Self {
            connected: false,
            db_type: "PostgreSQL".to_string(),
            version: String::new(),
            size: 0,
            table_count: 0,
            active_connections: 0,
            max_connections: 100,
            pool_size: 0,
            idle_connections: 0,
            queries_per_second: 0.0,
            avg_query_time: 0.0,
            slow_queries: 0,
        }
    }
}

/// Cache status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatus {
    /// Cache driver
    pub driver: String,

    /// Connection status
    pub connected: bool,

    /// Total keys
    pub total_keys: u64,

    /// Memory used in bytes
    pub memory_used: u64,

    /// Memory max in bytes
    pub memory_max: u64,

    /// Hit rate percentage
    pub hit_rate: f64,

    /// Hits count
    pub hits: u64,

    /// Misses count
    pub misses: u64,

    /// Evicted keys
    pub evicted_keys: u64,
}

impl Default for CacheStatus {
    fn default() -> Self {
        Self {
            driver: "redis".to_string(),
            connected: false,
            total_keys: 0,
            memory_used: 0,
            memory_max: 0,
            hit_rate: 0.0,
            hits: 0,
            misses: 0,
            evicted_keys: 0,
        }
    }
}

/// CDN status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnStatus {
    /// CDN provider
    pub provider: String,

    /// Connection status
    pub connected: bool,

    /// CDN domain
    pub domain: String,

    /// Cache hit ratio
    pub cache_hit_ratio: f64,

    /// Total requests (24h)
    pub total_requests: u64,

    /// Bandwidth saved (24h)
    pub bandwidth_saved: u64,

    /// SSL status
    pub ssl_enabled: bool,

    /// Last purge time
    pub last_purge: Option<DateTime<Utc>>,
}

/// Activity item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    /// Activity ID
    pub id: String,

    /// Activity type
    pub activity_type: ActivityType,

    /// Description
    pub description: String,

    /// User who performed the action
    pub user: Option<String>,

    /// IP address
    pub ip_address: Option<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Activity types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Login,
    Logout,
    PostCreated,
    PostUpdated,
    PostDeleted,
    UserCreated,
    UserUpdated,
    SettingsChanged,
    CachePurged,
    BackupCreated,
    PluginInstalled,
    PluginUpdated,
    ThemeChanged,
    Error,
    Warning,
    System,
}

/// Quick statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuickStats {
    /// Total posts
    pub total_posts: u64,

    /// Published posts
    pub published_posts: u64,

    /// Draft posts
    pub draft_posts: u64,

    /// Total pages
    pub total_pages: u64,

    /// Total users
    pub total_users: u64,

    /// Total comments
    pub total_comments: u64,

    /// Pending comments
    pub pending_comments: u64,

    /// Total media files
    pub total_media: u64,

    /// Storage used in bytes
    pub storage_used: u64,

    /// Page views today
    pub views_today: u64,

    /// Page views this week
    pub views_week: u64,

    /// Page views this month
    pub views_month: u64,
}

/// Dashboard service for collecting data
pub struct DashboardService {
    app_started_at: DateTime<Utc>,
}

impl DashboardService {
    /// Create a new dashboard service
    pub fn new() -> Self {
        Self {
            app_started_at: Utc::now(),
        }
    }

    /// Collect all dashboard data
    pub async fn collect_dashboard_data(&self) -> DashboardData {
        let system = SystemStatus::collect();

        let app = AppStatus {
            started_at: self.app_started_at,
            uptime: Utc::now()
                .signed_duration_since(self.app_started_at)
                .num_seconds() as u64,
            ..Default::default()
        };

        DashboardData {
            system,
            app,
            database: DatabaseStatus::default(),
            cache: CacheStatus::default(),
            cdn: None,
            activity: Vec::new(),
            stats: QuickStats::default(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for DashboardService {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration to human readable string
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "0m");
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(90000), "1d 1h 0m");
    }

    #[test]
    fn test_system_status() {
        let status = SystemStatus::collect();
        assert!(status.cpu_usage >= 0.0);
        assert!(status.memory_total > 0);
    }
}
