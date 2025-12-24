//! System health and metrics collection

use crate::status::SystemMetrics;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use std::sync::{Arc, Mutex};

/// System health monitor
pub struct SystemHealth {
    /// System info instance
    system: Arc<Mutex<System>>,
}

impl SystemHealth {
    /// Create a new system health monitor
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        Self {
            system: Arc::new(Mutex::new(system)),
        }
    }

    /// Get current system metrics
    pub fn get_metrics(&self) -> SystemMetrics {
        let mut sys = self.system.lock().unwrap();

        // Refresh system info
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        // Calculate CPU usage
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        // Calculate memory usage
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Get disk usage
        let disk_usage = self.get_disk_usage();

        // Get load average
        let load_average = self.get_load_average();

        SystemMetrics {
            cpu_usage,
            memory_usage,
            memory_total,
            memory_used,
            disk_usage,
            load_average,
            active_connections: None,
            open_files: None,
        }
    }

    /// Get disk usage for root partition
    fn get_disk_usage(&self) -> Option<f32> {
        let disks = Disks::new_with_refreshed_list();

        for disk in disks.list() {
            // Look for root or main partition
            let mount_point = disk.mount_point().to_string_lossy();
            if mount_point == "/" || mount_point == "C:\\" {
                let total = disk.total_space();
                let available = disk.available_space();

                if total > 0 {
                    let used = total - available;
                    return Some((used as f32 / total as f32) * 100.0);
                }
            }
        }

        None
    }

    /// Get system load average
    fn get_load_average(&self) -> Option<[f64; 3]> {
        #[cfg(unix)]
        {
            let load = System::load_average();
            Some([load.one, load.five, load.fifteen])
        }

        #[cfg(not(unix))]
        {
            None
        }
    }

    /// Check if system resources are critical
    pub fn is_critical(&self) -> bool {
        let metrics = self.get_metrics();

        // Critical thresholds
        metrics.cpu_usage > 95.0
            || metrics.memory_usage > 95.0
            || metrics.disk_usage.map(|d| d > 95.0).unwrap_or(false)
    }

    /// Check if system resources are degraded
    pub fn is_degraded(&self) -> bool {
        let metrics = self.get_metrics();

        // Degraded thresholds
        metrics.cpu_usage > 80.0
            || metrics.memory_usage > 85.0
            || metrics.disk_usage.map(|d| d > 85.0).unwrap_or(false)
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed system information for diagnostics
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct DetailedSystemInfo {
    /// OS name
    pub os_name: String,
    /// OS version
    pub os_version: String,
    /// Kernel version
    pub kernel_version: String,
    /// Hostname
    pub hostname: String,
    /// CPU count
    pub cpu_count: usize,
    /// Physical CPU count
    pub physical_cpu_count: usize,
    /// Total memory
    pub total_memory: u64,
    /// Total swap
    pub total_swap: u64,
    /// Boot time
    pub boot_time: u64,
}

#[allow(dead_code)]
impl DetailedSystemInfo {
    /// Collect detailed system information
    pub fn collect() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );

        Self {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: sys.cpus().len(),
            physical_cpu_count: sys.physical_core_count().unwrap_or(0),
            total_memory: sys.total_memory(),
            total_swap: sys.total_swap(),
            boot_time: System::boot_time(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_health() {
        let health = SystemHealth::new();
        let metrics = health.get_metrics();

        // Basic sanity checks
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_total > 0);
    }

    #[test]
    fn test_detailed_system_info() {
        let info = DetailedSystemInfo::collect();

        assert!(!info.os_name.is_empty());
        assert!(info.cpu_count > 0);
        assert!(info.total_memory > 0);
    }
}
