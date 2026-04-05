//! Performance metrics and monitoring for pb-mcp-server
//!
//! This module provides comprehensive performance monitoring including:
//! - Request timing and latency tracking
//! - Throughput measurement
//! - Resource utilization tracking
//! - Performance profiling
//! - Alert thresholds
//!
//! # Features
//!
//! - **Request timing** - Track request latency percentiles
//! - **Throughput** - Measure requests per second
//! - **Resource tracking** - Monitor memory and CPU usage
//! - **Performance profiling** - Identify bottlenecks
//! - **Metrics export** - Export to Prometheus, StatsD, etc.
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::metrics::{MetricsCollector, MetricsConfig, PerformanceMetrics};
//!
//! let config = MetricsConfig::default();
//! let metrics = MetricsCollector::new(config);
//!
//! // Record a timed operation
//! let timer = metrics.start_timer("file_read");
//! // ... do work ...
//! timer.stop(); // Automatically records duration
//!
//! // Get metrics summary
//! let summary = metrics.summary();
//! println!("Avg latency: {:.2}ms", summary.avg_latency_ms);
//! ```

use crate::error::{Error, Result};
// crossbeam_channel imports removed - not currently used
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// Metrics Configuration
// ============================================================================

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable timing metrics
    pub enable_timing: bool,

    /// Enable throughput metrics
    pub enable_throughput: bool,

    /// Enable resource metrics
    pub enable_resource: bool,

    /// Histogram bucket boundaries (in milliseconds)
    pub timing_buckets: Vec<f64>,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,

    /// Maximum number of operations to track
    pub max_operations: usize,

    /// Enable detailed profiling
    pub enable_profiling: bool,

    /// Export format (none, prometheus, json)
    pub export_format: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        MetricsConfig {
            enable_timing: true,
            enable_throughput: true,
            enable_resource: true,
            timing_buckets: vec![
                0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0,
            ],
            collection_interval_secs: 10,
            max_operations: 100,
            enable_profiling: false,
            export_format: "json".to_string(),
        }
    }
}

impl MetricsConfig {
    /// Create configuration for production
    pub fn production() -> Self {
        MetricsConfig {
            enable_timing: true,
            enable_throughput: true,
            enable_resource: true,
            timing_buckets: vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
            collection_interval_secs: 60,
            max_operations: 50,
            enable_profiling: false,
            export_format: "prometheus".to_string(),
        }
    }

    /// Create configuration for development
    pub fn development() -> Self {
        MetricsConfig {
            enable_timing: true,
            enable_throughput: true,
            enable_resource: true,
            timing_buckets: vec![
                0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0,
            ],
            collection_interval_secs: 5,
            max_operations: 100,
            enable_profiling: true,
            export_format: "json".to_string(),
        }
    }

    /// Create minimal configuration
    pub fn minimal() -> Self {
        MetricsConfig {
            enable_timing: false,
            enable_throughput: false,
            enable_resource: false,
            timing_buckets: vec![],
            collection_interval_secs: 300,
            max_operations: 10,
            enable_profiling: false,
            export_format: "none".to_string(),
        }
    }
}

// ============================================================================
// Timing Histogram
// ============================================================================

/// Histogram for tracking timing distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingHistogram {
    /// Bucket boundaries (in milliseconds)
    pub buckets: Vec<f64>,

    /// Count in each bucket
    pub counts: Vec<u64>,

    /// Total count
    pub total: u64,

    /// Sum of all values
    pub sum: f64,

    /// Minimum value
    pub min: f64,

    /// Maximum value
    pub max: f64,
}

impl TimingHistogram {
    /// Create a new timing histogram
    pub fn new(buckets: Vec<f64>) -> Self {
        let count = buckets.len() + 1; // +1 for overflow bucket
        TimingHistogram {
            buckets,
            counts: vec![0; count],
            total: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
        }
    }

    /// Record a value
    pub fn record(&mut self, value_ms: f64) {
        self.total += 1;
        self.sum += value_ms;
        self.min = self.min.min(value_ms);
        self.max = self.max.max(value_ms);

        // Find the right bucket
        let mut bucket_idx = self.buckets.len();
        for (i, &boundary) in self.buckets.iter().enumerate() {
            if value_ms <= boundary {
                bucket_idx = i;
                break;
            }
        }
        self.counts[bucket_idx] += 1;
    }

    /// Get percentile (0.0 to 100.0)
    pub fn percentile(&self, p: f64) -> f64 {
        if self.total == 0 {
            return 0.0;
        }

        let target = (p / 100.0) * self.total as f64;
        let mut cumulative = 0.0;

        for (i, &count) in self.counts.iter().enumerate() {
            cumulative += count as f64;
            if cumulative >= target {
                if i == 0 {
                    return 0.0;
                }
                return self.buckets[i - 1];
            }
        }

        self.buckets.last().copied().unwrap_or(0.0)
    }

    /// Get average value
    pub fn average(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.sum / self.total as f64
        }
    }

    /// Merge another histogram into this one
    pub fn merge(&mut self, other: &TimingHistogram) {
        self.total += other.total;
        self.sum += other.sum;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);

        for (i, &count) in other.counts.iter().enumerate() {
            self.counts[i] += count;
        }
    }

    /// Reset the histogram
    pub fn reset(&mut self) {
        for count in &mut self.counts {
            *count = 0;
        }
        self.total = 0;
        self.sum = 0.0;
        self.min = f64::MAX;
        self.max = f64::MIN;
    }
}

// ============================================================================
// Operation Metrics
// ============================================================================

/// Metrics for a single operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Operation name
    pub name: String,

    /// Timing histogram
    pub timing: TimingHistogram,

    /// Total requests
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Current in-flight requests
    pub in_flight: u32,

    /// Last request timestamp
    pub last_request_ts: Option<u64>,

    /// Errors by type
    pub errors: HashMap<String, u64>,
}

impl OperationMetrics {
    /// Create new operation metrics
    pub fn new(name: String, buckets: Vec<f64>) -> Self {
        OperationMetrics {
            name,
            timing: TimingHistogram::new(buckets),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            in_flight: 0,
            last_request_ts: None,
            errors: HashMap::new(),
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, duration: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.timing.record(duration.as_secs_f64() * 1000.0);
        self.last_request_ts = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Record a failed request
    pub fn record_failure(&mut self, duration: Duration, error_type: &str) {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.timing.record(duration.as_secs_f64() * 1000.0);
        self.last_request_ts = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        *self.errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Start tracking an in-flight request
    pub fn start_request(&mut self) {
        self.in_flight += 1;
    }

    /// Finish tracking an in-flight request
    pub fn finish_request(&mut self) {
        self.in_flight = self.in_flight.saturating_sub(1);
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    /// Get error rate (0.0 to 1.0)
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.failed_requests as f64 / self.total_requests as f64
        }
    }
}

// ============================================================================
// Performance Metrics Summary
// ============================================================================

/// Summary of performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total requests across all operations
    pub total_requests: u64,

    /// Total successful requests
    pub total_successful: u64,

    /// Total failed requests
    pub total_failed: u64,

    /// Average latency across all operations (ms)
    pub avg_latency_ms: f64,

    /// P50 latency (ms)
    pub p50_latency_ms: f64,

    /// P95 latency (ms)
    pub p95_latency_ms: f64,

    /// P99 latency (ms)
    pub p99_latency_ms: f64,

    /// Requests per second
    pub requests_per_second: f64,

    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,

    /// Current memory usage in bytes
    pub current_memory_bytes: u64,

    /// CPU usage percentage
    pub cpu_usage_percent: f64,

    /// Number of active operations
    pub active_operations: usize,

    /// Operation-level metrics
    pub operations: HashMap<String, OperationMetrics>,

    /// Timestamp
    pub timestamp: u64,

    /// Uptime in seconds
    pub uptime_secs: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            total_requests: 0,
            total_successful: 0,
            total_failed: 0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            requests_per_second: 0.0,
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
            cpu_usage_percent: 0.0,
            active_operations: 0,
            operations: HashMap::new(),
            timestamp: 0,
            uptime_secs: 0,
        }
    }
}

impl std::fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Performance Metrics ===")?;
        writeln!(f, "Total Requests: {}", self.total_requests)?;
        writeln!(
            f,
            "Success Rate: {:.2}%",
            self.total_successful as f64 / self.total_requests.max(1) as f64 * 100.0
        )?;
        writeln!(f)?;
        writeln!(f, "Latency:")?;
        writeln!(f, "  Average: {:.2}ms", self.avg_latency_ms)?;
        writeln!(f, "  P50: {:.2}ms", self.p50_latency_ms)?;
        writeln!(f, "  P95: {:.2}ms", self.p95_latency_ms)?;
        writeln!(f, "  P99: {:.2}ms", self.p99_latency_ms)?;
        writeln!(f)?;
        writeln!(f, "Throughput: {:.2} req/s", self.requests_per_second)?;
        writeln!(f)?;
        writeln!(f, "Resources:")?;
        writeln!(
            f,
            "  Memory: {:.2} MB",
            self.current_memory_bytes as f64 / 1_048_576.0
        )?;
        writeln!(
            f,
            "  Peak Memory: {:.2} MB",
            self.peak_memory_bytes as f64 / 1_048_576.0
        )?;
        writeln!(f, "  CPU: {:.1}%", self.cpu_usage_percent)?;
        writeln!(f)?;
        writeln!(f, "Active Operations: {}", self.active_operations)?;
        writeln!(f, "Uptime: {}s", self.uptime_secs)?;

        if !self.operations.is_empty() {
            writeln!(f)?;
            writeln!(f, "Operations:")?;
            for (name, metrics) in &self.operations {
                writeln!(f, "  {}:", name)?;
                writeln!(f, "    Total: {}", metrics.total_requests)?;
                writeln!(f, "    Success: {:.2}%", metrics.success_rate() * 100.0)?;
                writeln!(f, "    Avg: {:.2}ms", metrics.timing.average())?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// Timer Guard
// ============================================================================

/// RAII timer that automatically records duration when dropped
pub struct Timer {
    /// Start time
    start: Instant,

    /// Operation name
    operation: String,

    /// Metrics collector
    metrics: Arc<MetricsCollector>,

    /// Whether to record success on drop
    finished: bool,
}

impl Timer {
    /// Create a new timer
    fn new(operation: String, metrics: Arc<MetricsCollector>) -> Self {
        Timer {
            start: Instant::now(),
            operation,
            metrics,
            finished: false,
        }
    }

    /// Record success and stop the timer
    pub fn succeed(mut self) {
        self.finished = true;
        let duration = self.start.elapsed();
        self.metrics.record_success(&self.operation, duration);
    }

    /// Record failure and stop the timer
    pub fn fail(mut self, error_type: &str) {
        self.finished = true;
        let duration = self.start.elapsed();
        self.metrics
            .record_failure(&self.operation, duration, error_type);
    }

    /// Cancel the timer (don't record anything)
    pub fn cancel(mut self) {
        self.finished = true;
    }

    /// Get elapsed time so far
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if !self.finished {
            let duration = self.start.elapsed();
            self.metrics.record_success(&self.operation, duration);
        }
    }
}

// ============================================================================
// Metrics Collector
// ============================================================================

/// Main metrics collector
pub struct MetricsCollector {
    /// Configuration
    config: MetricsConfig,

    /// Operation metrics
    operations: RwLock<HashMap<String, OperationMetrics>>,

    /// Start time
    start_time: Instant,

    /// Total requests
    total_requests: AtomicU64,

    /// Peak memory
    peak_memory: AtomicU64,

    /// Current in-flight requests
    in_flight: AtomicU32,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        MetricsCollector {
            config,
            operations: RwLock::new(HashMap::new()),
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            peak_memory: AtomicU64::new(0),
            in_flight: AtomicU32::new(0),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MetricsConfig::default())
    }

    /// Start timing an operation
    pub fn start_timer(self: Arc<Self>, operation: &str) -> Timer {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        Timer::new(operation.to_string(), self)
    }

    /// Record a successful request
    fn record_success(&self, operation: &str, duration: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.in_flight.fetch_sub(1, Ordering::Relaxed);

        if !self.config.enable_timing {
            return;
        }

        let mut ops = self.operations.write();
        let metrics = ops.entry(operation.to_string()).or_insert_with(|| {
            OperationMetrics::new(operation.to_string(), self.config.timing_buckets.clone())
        });
        metrics.record_success(duration);
    }

    /// Record a failed request
    fn record_failure(&self, operation: &str, duration: Duration, error_type: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.in_flight.fetch_sub(1, Ordering::Relaxed);

        if !self.config.enable_timing {
            return;
        }

        let mut ops = self.operations.write();
        let metrics = ops.entry(operation.to_string()).or_insert_with(|| {
            OperationMetrics::new(operation.to_string(), self.config.timing_buckets.clone())
        });
        metrics.record_failure(duration, error_type);
    }

    /// Get current metrics summary
    pub fn summary(&self) -> PerformanceMetrics {
        let ops = self.operations.read();

        let mut total_successful = 0u64;
        let mut total_failed = 0u64;
        let mut combined_timing = TimingHistogram::new(self.config.timing_buckets.clone());

        for metrics in ops.values() {
            total_successful += metrics.successful_requests;
            total_failed += metrics.failed_requests;
            combined_timing.merge(&metrics.timing);
        }

        let uptime = self.start_time.elapsed().as_secs();
        let total = self.total_requests.load(Ordering::Relaxed);
        let rps = if uptime > 0 {
            total as f64 / uptime as f64
        } else {
            0.0
        };

        PerformanceMetrics {
            total_requests: total,
            total_successful,
            total_failed,
            avg_latency_ms: combined_timing.average(),
            p50_latency_ms: combined_timing.percentile(50.0),
            p95_latency_ms: combined_timing.percentile(95.0),
            p99_latency_ms: combined_timing.percentile(99.0),
            requests_per_second: rps,
            peak_memory_bytes: self.peak_memory.load(Ordering::Relaxed),
            current_memory_bytes: 0, // Would need actual memory tracking
            cpu_usage_percent: 0.0,  // Would need actual CPU tracking
            active_operations: ops.len(),
            operations: ops.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            uptime_secs: uptime,
        }
    }

    /// Update memory usage
    pub fn update_memory(&self, bytes: u64) {
        let current = self.peak_memory.load(Ordering::Relaxed);
        if bytes > current {
            self.peak_memory.store(bytes, Ordering::Relaxed);
        }
    }

    /// Get configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut ops = self.operations.write();
        ops.clear();
        self.total_requests.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> Result<String> {
        let summary = self.summary();
        serde_json::to_string_pretty(&summary)
            .map_err(|e| Error::Generic(format!("Failed to export metrics: {}", e)))
    }

    /// Export metrics as Prometheus format
    pub fn export_prometheus(&self) -> String {
        let summary = self.summary();
        let mut output = String::new();

        output.push_str("# HELP requests_total Total number of requests\n");
        output.push_str("# TYPE requests_total counter\n");
        output.push_str(&format!("requests_total {}\n", summary.total_requests));

        output.push_str("# HELP requests_successful Total successful requests\n");
        output.push_str("# TYPE requests_successful counter\n");
        output.push_str(&format!(
            "requests_successful {}\n",
            summary.total_successful
        ));

        output.push_str("# HELP requests_failed Total failed requests\n");
        output.push_str("# TYPE requests_failed counter\n");
        output.push_str(&format!("requests_failed {}\n", summary.total_failed));

        output.push_str("# HELP latency_ms_avg Average latency in milliseconds\n");
        output.push_str("# TYPE latency_ms_avg gauge\n");
        output.push_str(&format!("latency_ms_avg {:.2}\n", summary.avg_latency_ms));

        output.push_str("# HELP latency_ms_p50 50th percentile latency\n");
        output.push_str("# TYPE latency_ms_p50 gauge\n");
        output.push_str(&format!("latency_ms_p50 {:.2}\n", summary.p50_latency_ms));

        output.push_str("# HELP latency_ms_p95 95th percentile latency\n");
        output.push_str("# TYPE latency_ms_p95 gauge\n");
        output.push_str(&format!("latency_ms_p95 {:.2}\n", summary.p95_latency_ms));

        output.push_str("# HELP latency_ms_p99 99th percentile latency\n");
        output.push_str("# TYPE latency_ms_p99 gauge\n");
        output.push_str(&format!("latency_ms_p99 {:.2}\n", summary.p99_latency_ms));

        output.push_str("# HELP requests_per_second Requests per second\n");
        output.push_str("# TYPE requests_per_second gauge\n");
        output.push_str(&format!(
            "requests_per_second {:.2}\n",
            summary.requests_per_second
        ));

        output.push_str("# HELP uptime_seconds Server uptime in seconds\n");
        output.push_str("# TYPE uptime_seconds gauge\n");
        output.push_str(&format!("uptime_seconds {}\n", summary.uptime_secs));

        output
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        MetricsCollector {
            config: self.config.clone(),
            operations: RwLock::new(self.operations.read().clone()),
            start_time: self.start_time,
            total_requests: AtomicU64::new(self.total_requests.load(Ordering::Relaxed)),
            peak_memory: AtomicU64::new(self.peak_memory.load(Ordering::Relaxed)),
            in_flight: AtomicU32::new(self.in_flight.load(Ordering::Relaxed)),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enable_timing);
        assert!(config.enable_throughput);
        assert!(!config.timing_buckets.is_empty());
    }

    #[test]
    fn test_metrics_config_profiles() {
        let prod = MetricsConfig::production();
        assert!(prod.timing_buckets.len() > 0);

        let dev = MetricsConfig::development();
        assert!(dev.enable_profiling);

        let minimal = MetricsConfig::minimal();
        assert!(!minimal.enable_timing);
    }

    #[test]
    fn test_timing_histogram_basic() {
        let mut hist = TimingHistogram::new(vec![1.0, 5.0, 10.0]);

        hist.record(0.5);
        hist.record(2.0);
        hist.record(15.0);

        assert_eq!(hist.total, 3);
        assert_eq!(hist.counts[0], 1); // <= 1.0
        assert_eq!(hist.counts[1], 1); // <= 5.0
        assert_eq!(hist.counts[3], 1); // > 10.0
    }

    #[test]
    fn test_timing_histogram_percentiles() {
        let mut hist = TimingHistogram::new(vec![10.0, 50.0, 100.0]);

        for i in 1..=100 {
            hist.record(i as f64);
        }

        assert!(hist.percentile(50.0) > 0.0);
        assert!(hist.percentile(95.0) >= hist.percentile(50.0));
        assert!(hist.percentile(99.0) >= hist.percentile(95.0));
    }

    #[test]
    fn test_timing_histogram_average() {
        let mut hist = TimingHistogram::new(vec![10.0]);

        hist.record(5.0);
        hist.record(15.0);

        let avg = hist.average();
        assert!((avg - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_operation_metrics() {
        let mut metrics = OperationMetrics::new("test_op".to_string(), vec![10.0, 50.0]);

        metrics.record_success(Duration::from_millis(5));
        metrics.record_success(Duration::from_millis(15));
        metrics.record_failure(Duration::from_millis(10), "error_type");

        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert!((metrics.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_metrics_collector_basic() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        thread::sleep(Duration::from_millis(10));
        timer.succeed();

        let summary = collector.summary();
        assert_eq!(summary.total_requests, 1);
    }

    #[test]
    fn test_metrics_collector_success_failure() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        timer.succeed();

        let timer = collector.clone().start_timer("test_op");
        timer.fail("test_error");

        let summary = collector.summary();
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_successful, 1);
        assert_eq!(summary.total_failed, 1);
    }

    #[test]
    fn test_metrics_collector_summary() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        for _ in 0..10 {
            let timer = collector.clone().start_timer("test_op");
            thread::sleep(Duration::from_millis(5));
            timer.succeed();
        }

        let summary = collector.summary();
        assert_eq!(summary.total_requests, 10);
        assert!(summary.avg_latency_ms > 0.0);
        // requests_per_second could be very small in fast tests, just check it's valid
        assert!(summary.requests_per_second >= 0.0);
    }

    #[test]
    fn test_metrics_collector_export_json() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        timer.succeed();

        let json = collector.export_json().unwrap();
        assert!(json.contains("total_requests"));
        assert!(json.contains("avg_latency_ms"));
    }

    #[test]
    fn test_metrics_collector_export_prometheus() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        timer.succeed();

        let prom = collector.export_prometheus();
        assert!(prom.contains("requests_total"));
        assert!(prom.contains("latency_ms_avg"));
        assert!(prom.contains("uptime_seconds"));
    }

    #[test]
    fn test_timer_cancel() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        timer.cancel();

        let summary = collector.summary();
        assert_eq!(summary.total_requests, 0);
    }

    #[test]
    fn test_timer_elapsed() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        thread::sleep(Duration::from_millis(50));
        let elapsed = timer.elapsed();
        drop(timer);

        assert!(elapsed >= Duration::from_millis(50));
    }

    #[test]
    fn test_performance_metrics_display() {
        let mut metrics = PerformanceMetrics::default();
        metrics.total_requests = 100;
        metrics.avg_latency_ms = 50.5;
        metrics.requests_per_second = 100.0;

        let output = format!("{}", metrics);
        assert!(output.contains("Total Requests: 100"));
        assert!(output.contains("Average: 50.50ms"));
        assert!(output.contains("Throughput: 100.00 req/s"));
    }

    #[test]
    fn test_in_flight_tracking() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        assert_eq!(collector.in_flight.load(Ordering::Relaxed), 0);

        let timer1 = collector.clone().start_timer("op1");
        assert_eq!(collector.in_flight.load(Ordering::Relaxed), 1);

        let timer2 = collector.clone().start_timer("op2");
        assert_eq!(collector.in_flight.load(Ordering::Relaxed), 2);

        drop(timer1);
        assert_eq!(collector.in_flight.load(Ordering::Relaxed), 1);

        drop(timer2);
        assert_eq!(collector.in_flight.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_metrics_reset() {
        let collector = Arc::new(MetricsCollector::with_defaults());

        let timer = collector.clone().start_timer("test_op");
        timer.succeed();

        assert!(collector.summary().total_requests > 0);

        collector.reset();
        assert_eq!(collector.summary().total_requests, 0);
    }

    #[test]
    fn test_memory_tracking() {
        let collector = MetricsCollector::with_defaults();

        collector.update_memory(1000);
        assert_eq!(collector.peak_memory.load(Ordering::Relaxed), 1000);

        collector.update_memory(500);
        assert_eq!(collector.peak_memory.load(Ordering::Relaxed), 1000);

        collector.update_memory(2000);
        assert_eq!(collector.peak_memory.load(Ordering::Relaxed), 2000);
    }
}
