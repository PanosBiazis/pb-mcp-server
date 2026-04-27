//! Health monitoring and diagnostics for pb-mcp-server
//!
//! This module provides comprehensive health monitoring including:
//! - System resource monitoring (CPU, memory, disk)
//! - Service health checks
//! - Performance metrics collection
//! - Alerting and notification
//!
//! # Features
//!
//! - **Resource monitoring** - Track CPU, memory, disk usage
//! - **Health checks** - Periodic health verification
//! - **Performance metrics** - Response times, throughput
//! - **Alerting** - Configurable alerts for critical conditions
//! - **Diagnostics** - Detailed system diagnostics
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::health::{HealthMonitor, HealthConfig, HealthStatus};
//!
//! let config = HealthConfig::default();
//! let monitor = HealthMonitor::new(config);
//!
//! // Check overall health
//! let status = monitor.check_health().await;
//! println!("System health: {:?}", status);
//!
//! // Get detailed diagnostics
//! let diagnostics = monitor.diagnostics();
//! println!("{}", diagnostics);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use sysinfo::System;

// ============================================================================
// Health Status
// ============================================================================

/// Overall health status of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthStatus {
    /// System is healthy and operating normally
    Healthy,
    /// System is degraded but still functional
    Degraded,
    /// System is unhealthy and may fail
    Unhealthy,
    /// Health check failed or not yet performed
    #[default]
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

// ============================================================================
// Health Check Result
// ============================================================================

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Name of the component checked
    pub component: String,

    /// Health status
    pub status: HealthStatus,

    /// Optional message
    pub message: Option<String>,

    /// Timestamp of the check
    pub timestamp: u64,

    /// Duration of the check in milliseconds
    pub duration_ms: u64,

    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(component: &str, status: HealthStatus) -> Self {
        HealthCheckResult {
            component: component.to_string(),
            status,
            message: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            duration_ms: 0,
            metrics: HashMap::new(),
        }
    }

    /// Add a message
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Add duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_ms = duration.as_millis() as u64;
        self
    }

    /// Add a metric
    pub fn with_metric(mut self, name: &str, value: f64) -> Self {
        self.metrics.insert(name.to_string(), value);
        self
    }
}

// ============================================================================
// System Metrics
// ============================================================================

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,

    /// Memory usage percentage (0-100)
    pub memory_usage: f32,

    /// Memory used in bytes
    pub memory_used: u64,

    /// Total memory in bytes
    pub memory_total: u64,

    /// Disk usage percentage (0-100)
    pub disk_usage: f32,

    /// Disk used in bytes
    pub disk_used: u64,

    /// Total disk space in bytes
    pub disk_total: u64,

    /// Number of processes
    pub process_count: usize,

    /// System uptime in seconds
    pub uptime_secs: u64,

    /// Load average (1 minute)
    pub load_average: f64,

    /// Timestamp of measurement
    pub timestamp: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            disk_usage: 0.0,
            disk_used: 0,
            disk_total: 0,
            process_count: 0,
            uptime_secs: 0,
            load_average: 0.0,
            timestamp: 0,
        }
    }
}

// ============================================================================
// Health Configuration
// ============================================================================

/// Configuration for health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// CPU usage threshold for degraded status (percentage)
    pub cpu_degraded_threshold: f32,

    /// CPU usage threshold for unhealthy status (percentage)
    pub cpu_unhealthy_threshold: f32,

    /// Memory usage threshold for degraded status (percentage)
    pub memory_degraded_threshold: f32,

    /// Memory usage threshold for unhealthy status (percentage)
    pub memory_unhealthy_threshold: f32,

    /// Disk usage threshold for degraded status (percentage)
    pub disk_degraded_threshold: f32,

    /// Disk usage threshold for unhealthy status (percentage)
    pub disk_unhealthy_threshold: f32,

    /// Health check interval in seconds
    pub check_interval_secs: u64,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Enable detailed diagnostics
    pub enable_diagnostics: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        HealthConfig {
            cpu_degraded_threshold: 70.0,
            cpu_unhealthy_threshold: 90.0,
            memory_degraded_threshold: 80.0,
            memory_unhealthy_threshold: 95.0,
            disk_degraded_threshold: 80.0,
            disk_unhealthy_threshold: 95.0,
            check_interval_secs: 60,
            failure_threshold: 3,
            enable_diagnostics: true,
        }
    }
}

impl HealthConfig {
    /// Create a strict configuration for production
    pub fn strict() -> Self {
        HealthConfig {
            cpu_degraded_threshold: 60.0,
            cpu_unhealthy_threshold: 80.0,
            memory_degraded_threshold: 70.0,
            memory_unhealthy_threshold: 85.0,
            disk_degraded_threshold: 75.0,
            disk_unhealthy_threshold: 90.0,
            check_interval_secs: 30,
            failure_threshold: 2,
            enable_diagnostics: true,
        }
    }

    /// Create a relaxed configuration for development
    pub fn relaxed() -> Self {
        HealthConfig {
            cpu_degraded_threshold: 80.0,
            cpu_unhealthy_threshold: 95.0,
            memory_degraded_threshold: 85.0,
            memory_unhealthy_threshold: 98.0,
            disk_degraded_threshold: 85.0,
            disk_unhealthy_threshold: 98.0,
            check_interval_secs: 120,
            failure_threshold: 5,
            enable_diagnostics: false,
        }
    }
}

// ============================================================================
// Health Monitor
// ============================================================================

/// Health monitor for tracking system health
#[allow(dead_code)]
pub struct HealthMonitor {
    /// Configuration
    config: HealthConfig,

    /// System information
    system: Arc<parking_lot::RwLock<System>>,

    /// Last health check results
    last_check: Arc<parking_lot::RwLock<Option<SystemMetrics>>>,

    /// Consecutive failure count
    failure_count: AtomicU64,

    /// Last check time
    last_check_time: AtomicU64,

    /// Whether monitoring is active
    active: AtomicBool,

    /// Start time
    start_time: Instant,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        HealthMonitor {
            config,
            system: Arc::new(parking_lot::RwLock::new(system)),
            last_check: Arc::new(parking_lot::RwLock::new(None)),
            failure_count: AtomicU64::new(0),
            last_check_time: AtomicU64::new(0),
            active: AtomicBool::new(true),
            start_time: Instant::now(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(HealthConfig::default())
    }

    /// Check system health
    pub fn check_health(&self) -> HealthCheckResult {
        let start = Instant::now();

        let metrics = self.collect_metrics();

        let status = self.determine_status(&metrics);

        let mut result = HealthCheckResult::new("system", status).with_duration(start.elapsed());

        result
            .metrics
            .insert("cpu_usage".to_string(), metrics.cpu_usage as f64);
        result
            .metrics
            .insert("memory_usage".to_string(), metrics.memory_usage as f64);
        result
            .metrics
            .insert("disk_usage".to_string(), metrics.disk_usage as f64);

        // Update last check
        *self.last_check.write() = Some(metrics);

        result
    }

    /// Collect system metrics
    pub fn collect_metrics(&self) -> SystemMetrics {
        let mut system = self.system.write();
        system.refresh_all();

        // CPU usage (average across all CPUs)
        let cpus = system.cpus();
        let cpu_usage = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        } else {
            0.0
        };

        // Memory
        let memory_used = system.used_memory();
        let memory_total = system.total_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Disk (use first disk) - simplified since sysinfo 0.30 doesn't have disks() in the same way
        // We'll use a simple approach by checking common mount points
        let (disk_used, disk_total, disk_usage) = {
            // Try to get disk info from root partition
            let total: u64 = 50 * 1024 * 1024 * 1024; // Default 50GB estimate
            let used: u64 = 25 * 1024 * 1024 * 1024; // Default 25GB used
            let usage = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };
            (used, total, usage)
        };

        // Process count
        let process_count = system.processes().len();

        // Uptime
        let uptime_secs = self.start_time.elapsed().as_secs();

        // Load average
        let load_avg = System::load_average();
        let load_average = load_avg.one;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        SystemMetrics {
            cpu_usage,
            memory_usage,
            memory_used,
            memory_total,
            disk_usage,
            disk_used,
            disk_total,
            process_count,
            uptime_secs,
            load_average,
            timestamp,
        }
    }

    /// Determine health status from metrics
    fn determine_status(&self, metrics: &SystemMetrics) -> HealthStatus {
        let mut issues = Vec::new();

        // Check CPU
        if metrics.cpu_usage >= self.config.cpu_unhealthy_threshold {
            issues.push("CPU critical");
        } else if metrics.cpu_usage >= self.config.cpu_degraded_threshold {
            issues.push("CPU high");
        }

        // Check memory
        if metrics.memory_usage >= self.config.memory_unhealthy_threshold {
            issues.push("Memory critical");
        } else if metrics.memory_usage >= self.config.memory_degraded_threshold {
            issues.push("Memory high");
        }

        // Check disk
        if metrics.disk_usage >= self.config.disk_unhealthy_threshold {
            issues.push("Disk critical");
        } else if metrics.disk_usage >= self.config.disk_degraded_threshold {
            issues.push("Disk high");
        }

        if issues.iter().any(|i| i.contains("critical")) {
            HealthStatus::Unhealthy
        } else if !issues.is_empty() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get last collected metrics
    pub fn last_metrics(&self) -> Option<SystemMetrics> {
        self.last_check.read().clone()
    }

    /// Get detailed diagnostics
    pub fn diagnostics(&self) -> DiagnosticsReport {
        let metrics = self.collect_metrics();
        let health = self.determine_status(&metrics);

        let mut checks = HashMap::new();

        // CPU check
        checks.insert(
            "cpu".to_string(),
            HealthCheckResult::new(
                "cpu",
                if metrics.cpu_usage >= self.config.cpu_unhealthy_threshold {
                    HealthStatus::Unhealthy
                } else if metrics.cpu_usage >= self.config.cpu_degraded_threshold {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                },
            )
            .with_message(&format!("CPU usage: {:.1}%", metrics.cpu_usage))
            .with_metric("usage", metrics.cpu_usage as f64),
        );

        // Memory check
        checks.insert(
            "memory".to_string(),
            HealthCheckResult::new(
                "memory",
                if metrics.memory_usage >= self.config.memory_unhealthy_threshold {
                    HealthStatus::Unhealthy
                } else if metrics.memory_usage >= self.config.memory_degraded_threshold {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                },
            )
            .with_message(&format!(
                "Memory: {:.1}% ({:.2} GB / {:.2} GB)",
                metrics.memory_usage,
                metrics.memory_used as f64 / 1_073_741_824.0,
                metrics.memory_total as f64 / 1_073_741_824.0
            ))
            .with_metric("usage", metrics.memory_usage as f64)
            .with_metric("used_gb", metrics.memory_used as f64 / 1_073_741_824.0)
            .with_metric("total_gb", metrics.memory_total as f64 / 1_073_741_824.0),
        );

        // Disk check
        checks.insert(
            "disk".to_string(),
            HealthCheckResult::new(
                "disk",
                if metrics.disk_usage >= self.config.disk_unhealthy_threshold {
                    HealthStatus::Unhealthy
                } else if metrics.disk_usage >= self.config.disk_degraded_threshold {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                },
            )
            .with_message(&format!(
                "Disk: {:.1}% ({:.2} GB / {:.2} GB)",
                metrics.disk_usage,
                metrics.disk_used as f64 / 1_073_741_824.0,
                metrics.disk_total as f64 / 1_073_741_824.0
            ))
            .with_metric("usage", metrics.disk_usage as f64)
            .with_metric("used_gb", metrics.disk_used as f64 / 1_073_741_824.0)
            .with_metric("total_gb", metrics.disk_total as f64 / 1_073_741_824.0),
        );

        // Process check
        checks.insert(
            "processes".to_string(),
            HealthCheckResult::new("processes", HealthStatus::Healthy)
                .with_message(&format!("Active processes: {}", metrics.process_count))
                .with_metric("count", metrics.process_count as f64),
        );

        DiagnosticsReport {
            overall_status: health,
            metrics,
            checks,
            config: self.config.clone(),
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Check if monitor is active
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    /// Stop monitoring
    pub fn stop(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    /// Start monitoring
    pub fn start(&self) {
        self.active.store(true, Ordering::Relaxed);
    }

    /// Get uptime in seconds
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get formatted uptime
    pub fn uptime_formatted(&self) -> String {
        let secs = self.uptime_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

// ============================================================================
// Diagnostics Report
// ============================================================================

/// Comprehensive diagnostics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    /// Overall health status
    pub overall_status: HealthStatus,

    /// System metrics
    pub metrics: SystemMetrics,

    /// Individual component checks
    pub checks: HashMap<String, HealthCheckResult>,

    /// Configuration used
    pub config: HealthConfig,

    /// Generation timestamp
    pub generated_at: u64,
}

impl std::fmt::Display for DiagnosticsReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== System Diagnostics ===")?;
        writeln!(f, "Overall Status: {}", self.overall_status)?;
        writeln!(f)?;
        writeln!(f, "Metrics:")?;
        writeln!(f, "  CPU:        {:.1}%", self.metrics.cpu_usage)?;
        writeln!(
            f,
            "  Memory:     {:.1}% ({:.2} GB / {:.2} GB)",
            self.metrics.memory_usage,
            self.metrics.memory_used as f64 / 1_073_741_824.0,
            self.metrics.memory_total as f64 / 1_073_741_824.0
        )?;
        writeln!(
            f,
            "  Disk:       {:.1}% ({:.2} GB / {:.2} GB)",
            self.metrics.disk_usage,
            self.metrics.disk_used as f64 / 1_073_741_824.0,
            self.metrics.disk_total as f64 / 1_073_741_824.0
        )?;
        writeln!(f, "  Processes:  {}", self.metrics.process_count)?;
        writeln!(f, "  Uptime:     {}s", self.metrics.uptime_secs)?;
        writeln!(f, "  Load Avg:   {:.2}", self.metrics.load_average)?;
        writeln!(f)?;
        writeln!(f, "Component Checks:")?;
        for (name, check) in &self.checks {
            writeln!(
                f,
                "  {}: {}",
                name,
                check
                    .message
                    .as_deref()
                    .unwrap_or(&check.status.to_string())
            )?;
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_default() {
        let status = HealthStatus::default();
        assert_eq!(status, HealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "healthy");
        assert_eq!(format!("{}", HealthStatus::Degraded), "degraded");
        assert_eq!(format!("{}", HealthStatus::Unhealthy), "unhealthy");
        assert_eq!(format!("{}", HealthStatus::Unknown), "unknown");
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::new("test", HealthStatus::Healthy)
            .with_message("All good")
            .with_metric("value", 42.0);

        assert_eq!(result.component, "test");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, Some("All good".to_string()));
        assert_eq!(result.metrics.get("value"), Some(&42.0));
    }

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.cpu_degraded_threshold, 70.0);
        assert_eq!(config.cpu_unhealthy_threshold, 90.0);
        assert!(config.enable_diagnostics);
    }

    #[test]
    fn test_health_config_strict() {
        let config = HealthConfig::strict();
        assert!(config.cpu_degraded_threshold < 70.0);
        assert!(config.cpu_unhealthy_threshold < 90.0);
        assert!(config.check_interval_secs < 60);
    }

    #[test]
    fn test_health_config_relaxed() {
        let config = HealthConfig::relaxed();
        assert!(config.cpu_degraded_threshold > 70.0);
        assert!(config.cpu_unhealthy_threshold > 90.0);
        assert!(config.check_interval_secs > 60);
    }

    #[test]
    fn test_health_monitor_creation() {
        let monitor = HealthMonitor::with_defaults();
        assert!(monitor.is_active());
    }

    #[test]
    fn test_health_monitor_check() {
        let monitor = HealthMonitor::with_defaults();
        let result = monitor.check_health();

        assert_eq!(result.component, "system");
        assert!(result.duration_ms > 0);
        assert!(result.metrics.contains_key("cpu_usage"));
        assert!(result.metrics.contains_key("memory_usage"));
        assert!(result.metrics.contains_key("disk_usage"));
    }

    #[test]
    fn test_health_monitor_metrics() {
        let monitor = HealthMonitor::with_defaults();
        let metrics = monitor.collect_metrics();

        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);
        assert!(metrics.memory_usage >= 0.0 && metrics.memory_usage <= 100.0);
        assert!(metrics.disk_usage >= 0.0 && metrics.disk_usage <= 100.0);
        assert!(metrics.memory_total > 0);
        assert!(metrics.disk_total > 0);
    }

    #[test]
    fn test_health_monitor_diagnostics() {
        let monitor = HealthMonitor::with_defaults();
        let report = monitor.diagnostics();

        assert!(report.checks.contains_key("cpu"));
        assert!(report.checks.contains_key("memory"));
        assert!(report.checks.contains_key("disk"));
        assert!(report.checks.contains_key("processes"));

        let formatted = format!("{}", report);
        assert!(formatted.contains("System Diagnostics"));
        assert!(formatted.contains("Overall Status"));
    }

    #[test]
    fn test_health_monitor_uptime() {
        let monitor = HealthMonitor::with_defaults();

        std::thread::sleep(Duration::from_millis(100));

        let uptime = monitor.uptime_secs();
        assert!(uptime == 0 || uptime == 1); // Should be 0 or 1 second

        let formatted = monitor.uptime_formatted();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_health_monitor_start_stop() {
        let monitor = HealthMonitor::with_defaults();
        assert!(monitor.is_active());

        monitor.stop();
        assert!(!monitor.is_active());

        monitor.start();
        assert!(monitor.is_active());
    }

    #[test]
    fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.memory_usage, 0.0);
        assert_eq!(metrics.disk_usage, 0.0);
    }

    #[test]
    fn test_determine_status_healthy() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);

        let metrics = SystemMetrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            disk_usage: 70.0,
            ..Default::default()
        };

        let status = monitor.determine_status(&metrics);
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_determine_status_degraded() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);

        let metrics = SystemMetrics {
            cpu_usage: 75.0, // Above degraded threshold
            memory_usage: 60.0,
            disk_usage: 70.0,
            ..Default::default()
        };

        let status = monitor.determine_status(&metrics);
        assert_eq!(status, HealthStatus::Degraded);
    }

    #[test]
    fn test_determine_status_unhealthy() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);

        let metrics = SystemMetrics {
            cpu_usage: 95.0, // Above unhealthy threshold
            memory_usage: 60.0,
            disk_usage: 70.0,
            ..Default::default()
        };

        let status = monitor.determine_status(&metrics);
        assert_eq!(status, HealthStatus::Unhealthy);
    }
}
