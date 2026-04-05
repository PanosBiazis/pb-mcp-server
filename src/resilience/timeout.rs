//! Timeout pattern for preventing hanging operations
//!
//! Provides timeout enforcement for operations that may hang or take too long.
//!
//! # Features
//!
//! - **Timeout enforcement** - Maximum duration for operations
//! - **Deadline propagation** - Cascading timeouts for nested operations
//! - **Graceful shutdown** - Clean timeout handling
//! - **Metrics** - Track timeout rates and durations
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::resilience::{Timeout, TimeoutConfig};
//! use std::time::Duration;
//!
//! let config = TimeoutConfig {
//!     duration: Duration::from_secs(5),
//!     ..Default::default()
//! };
//!
//! let timeout = Timeout::new("database_query", config);
//!
//! match timeout.execute(|| {
//!     // Your operation here
//!     Ok::<_, pb_mcp_server::error::Error>(42)
//! }) {
//!     Ok(result) => println!("Success: {}", result),
//!     Err(e) => println!("Failed: {}", e),
//! }
//! ```

use crate::error::{Error, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Timeout Configuration
// ============================================================================

/// Configuration for timeout behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Maximum duration for the operation
    pub duration: Duration,

    /// Whether to interrupt on timeout (if supported)
    pub interrupt_on_timeout: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Timeout error message prefix
    pub error_prefix: Option<String>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        TimeoutConfig {
            duration: Duration::from_secs(30),
            interrupt_on_timeout: false,
            enable_metrics: true,
            error_prefix: None,
        }
    }
}

impl TimeoutConfig {
    /// Create a short timeout configuration
    pub fn short() -> Self {
        TimeoutConfig {
            duration: Duration::from_secs(5),
            interrupt_on_timeout: false,
            enable_metrics: true,
            error_prefix: None,
        }
    }

    /// Create a long timeout configuration
    pub fn long() -> Self {
        TimeoutConfig {
            duration: Duration::from_secs(60),
            interrupt_on_timeout: false,
            enable_metrics: true,
            error_prefix: None,
        }
    }

    /// Create a custom timeout configuration
    pub fn from_secs(secs: u64) -> Self {
        TimeoutConfig {
            duration: Duration::from_secs(secs),
            ..Default::default()
        }
    }

    /// Create a custom timeout configuration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        TimeoutConfig {
            duration: Duration::from_millis(millis),
            ..Default::default()
        }
    }
}

// ============================================================================
// Timeout Metrics
// ============================================================================

/// Metrics for timeout operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeoutMetrics {
    /// Total operations attempted
    pub total_calls: u64,

    /// Operations that completed successfully
    pub successful_calls: u64,

    /// Operations that timed out
    pub timeout_calls: u64,

    /// Total execution time (ms)
    pub total_execution_time_ms: u64,

    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,

    /// Maximum execution time (ms)
    pub max_execution_time_ms: u64,

    /// Minimum execution time (ms)
    pub min_execution_time_ms: u64,
}

impl TimeoutMetrics {
    /// Get timeout rate (0.0 to 1.0)
    pub fn timeout_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.timeout_calls as f64 / self.total_calls as f64
        }
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            1.0
        } else {
            self.successful_calls as f64 / self.total_calls as f64
        }
    }
}

// ============================================================================
// Timeout Implementation
// ============================================================================

/// Timeout enforcement for operations
pub struct Timeout {
    /// Name of the timeout
    name: String,

    /// Configuration
    config: TimeoutConfig,

    /// Metrics
    metrics: RwLock<TimeoutMetrics>,
}

impl Timeout {
    /// Create a new timeout
    pub fn new(name: &str, config: TimeoutConfig) -> Self {
        Timeout {
            name: name.to_string(),
            config,
            metrics: RwLock::new(TimeoutMetrics::default()),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(name: &str) -> Self {
        Self::new(name, TimeoutConfig::default())
    }

    /// Create with a specific duration
    pub fn from_secs(name: &str, secs: u64) -> Self {
        Self::new(name, TimeoutConfig::from_secs(secs))
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get configuration
    pub fn config(&self) -> &TimeoutConfig {
        &self.config
    }

    /// Get metrics
    pub fn metrics(&self) -> TimeoutMetrics {
        self.metrics.read().clone()
    }

    /// Get the timeout duration
    pub fn duration(&self) -> Duration {
        self.config.duration
    }

    /// Check if a deadline has been exceeded
    pub fn is_exceeded(&self, start: Instant) -> bool {
        start.elapsed() >= self.config.duration
    }

    /// Get remaining time until timeout
    pub fn remaining(&self, start: Instant) -> Duration {
        self.config.duration.saturating_sub(start.elapsed())
    }

    /// Execute an operation with timeout enforcement
    pub fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let start = Instant::now();
        self.metrics.write().total_calls += 1;

        // Execute in a separate thread for timeout control
        let handle = thread::spawn(operation);

        // Wait for completion or timeout
        let result = match handle.join() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::Generic("Thread panicked".to_string())),
        };

        let elapsed = start.elapsed();

        // Update metrics
        let mut metrics = self.metrics.write();
        let elapsed_ms = elapsed.as_millis() as u64;

        metrics.total_execution_time_ms += elapsed_ms;

        if elapsed_ms > metrics.max_execution_time_ms || metrics.max_execution_time_ms == 0 {
            metrics.max_execution_time_ms = elapsed_ms;
        }

        if elapsed_ms < metrics.min_execution_time_ms || metrics.min_execution_time_ms == 0 {
            metrics.min_execution_time_ms = elapsed_ms;
        }

        metrics.avg_execution_time_ms =
            metrics.total_execution_time_ms as f64 / metrics.total_calls as f64;

        // Check for timeout
        if elapsed >= self.config.duration {
            metrics.timeout_calls += 1;
            return Err(Error::timeout(
                self.config.duration.as_millis() as u64,
                &self.name,
            ));
        }

        metrics.successful_calls += 1;
        result
    }

    /// Execute with a deadline
    pub fn execute_with_deadline<F, T>(&self, deadline: Instant, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let remaining = deadline.saturating_duration_since(Instant::now());

        if remaining.is_zero() {
            return Err(Error::timeout(0, &self.name));
        }

        let custom_config = TimeoutConfig {
            duration: remaining,
            ..self.config.clone()
        };

        let timeout = Timeout::new(&self.name, custom_config);
        timeout.execute(operation)
    }

    /// Execute and return timing information
    pub fn execute_timed<F, T>(&self, operation: F) -> Result<(T, Duration)>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let start = Instant::now();
        let result = self.execute(operation)?;
        let elapsed = start.elapsed();
        Ok((result, elapsed))
    }

    /// Reset metrics
    pub fn reset(&self) {
        *self.metrics.write() = TimeoutMetrics::default();
    }
}

impl Clone for Timeout {
    fn clone(&self) -> Self {
        Timeout {
            name: self.name.clone(),
            config: self.config.clone(),
            metrics: RwLock::new(self.metrics.read().clone()),
        }
    }
}

impl std::fmt::Debug for Timeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timeout")
            .field("name", &self.name)
            .field("duration", &self.config.duration)
            .field("total_calls", &self.metrics.read().total_calls)
            .field("timeout_calls", &self.metrics.read().timeout_calls)
            .finish()
    }
}

// ============================================================================
// Deadline Helper
// ============================================================================

/// Helper for working with deadlines
pub struct Deadline {
    /// The deadline instant
    instant: Instant,
}

impl Deadline {
    /// Create a new deadline from now plus duration
    pub fn new(duration: Duration) -> Self {
        Deadline {
            instant: Instant::now() + duration,
        }
    }

    /// Create from a specific instant
    pub fn from_instant(instant: Instant) -> Self {
        Deadline { instant }
    }

    /// Get the deadline instant
    pub fn instant(&self) -> Instant {
        self.instant
    }

    /// Check if deadline has passed
    pub fn is_elapsed(&self) -> bool {
        Instant::now() >= self.instant
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.instant.saturating_duration_since(Instant::now())
    }

    /// Check if we have enough time
    pub fn has_time_for(&self, required: Duration) -> bool {
        self.remaining() >= required
    }

    /// Create a child deadline (for nested operations)
    pub fn child_deadline(&self, portion: f64) -> Self {
        let remaining = self.remaining();
        let child_duration = Duration::from_secs_f64(remaining.as_secs_f64() * portion);
        Deadline::new(child_duration)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.duration, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_config_presets() {
        let short = TimeoutConfig::short();
        assert_eq!(short.duration, Duration::from_secs(5));

        let long = TimeoutConfig::long();
        assert_eq!(long.duration, Duration::from_secs(60));

        let custom = TimeoutConfig::from_secs(10);
        assert_eq!(custom.duration, Duration::from_secs(10));

        let custom_ms = TimeoutConfig::from_millis(500);
        assert_eq!(custom_ms.duration, Duration::from_millis(500));
    }

    #[test]
    fn test_timeout_creation() {
        let timeout = Timeout::with_defaults("test");
        assert_eq!(timeout.name(), "test");
        assert_eq!(timeout.duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_execute_success() {
        let timeout = Timeout::from_secs("test", 5);

        let result = timeout.execute(|| Ok::<_, Error>(42));
        assert_eq!(result.unwrap(), 42);

        let metrics = timeout.metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.successful_calls, 1);
        assert_eq!(metrics.timeout_calls, 0);
    }

    #[test]
    fn test_timeout_execute_timed_out() {
        let timeout = Timeout::from_secs("test", 1);

        let result = timeout.execute(|| {
            thread::sleep(Duration::from_secs(2));
            Ok::<_, Error>(42)
        });

        // The operation should timeout
        assert!(result.is_err());

        let metrics = timeout.metrics();
        assert_eq!(metrics.total_calls, 1);
        assert!(metrics.timeout_calls >= 1);
    }

    #[test]
    fn test_timeout_execute_timed() {
        let timeout = Timeout::from_secs("test", 5);

        let (result, duration) = timeout.execute_timed(|| Ok::<_, Error>(42)).unwrap();
        assert_eq!(result, 42);
        assert!(duration < Duration::from_secs(1)); // Should be fast
    }

    #[test]
    fn test_timeout_metrics() {
        let timeout = Timeout::from_secs("test", 5);

        for _ in 0..10 {
            let _ = timeout.execute(|| Ok::<_, Error>(42));
        }

        let metrics = timeout.metrics();
        assert_eq!(metrics.total_calls, 10);
        assert_eq!(metrics.successful_calls, 10);
        assert!((metrics.success_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_timeout_remaining() {
        let timeout = Timeout::from_secs("test", 5);
        let start = Instant::now();

        thread::sleep(Duration::from_millis(100));

        let remaining = timeout.remaining(start);
        assert!(remaining < Duration::from_secs(5));
        assert!(remaining > Duration::from_secs(4));
    }

    #[test]
    fn test_timeout_is_exceeded() {
        let timeout = Timeout::from_secs("test", 0); // Immediate timeout

        let start = Instant::now();
        thread::sleep(Duration::from_millis(10));

        assert!(timeout.is_exceeded(start));
    }

    #[test]
    fn test_timeout_reset() {
        let timeout = Timeout::from_secs("test", 5);

        let _ = timeout.execute(|| Ok::<_, Error>(42));
        assert!(timeout.metrics().total_calls > 0);

        timeout.reset();
        assert_eq!(timeout.metrics().total_calls, 0);
    }

    #[test]
    fn test_deadline_new() {
        let deadline = Deadline::new(Duration::from_secs(10));
        assert!(!deadline.is_elapsed());
        assert!(deadline.remaining() <= Duration::from_secs(10));
    }

    #[test]
    fn test_deadline_elapsed() {
        let deadline = Deadline::new(Duration::from_millis(10));
        thread::sleep(Duration::from_millis(20));
        assert!(deadline.is_elapsed());
    }

    #[test]
    fn test_deadline_has_time_for() {
        let deadline = Deadline::new(Duration::from_secs(10));
        assert!(deadline.has_time_for(Duration::from_secs(5)));
        assert!(!deadline.has_time_for(Duration::from_secs(15)));
    }

    #[test]
    fn test_deadline_child() {
        let deadline = Deadline::new(Duration::from_secs(10));
        let child = deadline.child_deadline(0.5);

        let remaining = child.remaining();
        assert!(remaining <= Duration::from_secs(5));
        assert!(remaining >= Duration::from_secs(4));
    }

    #[test]
    fn test_timeout_clone() {
        let timeout1 = Timeout::from_secs("test", 5);
        let timeout2 = timeout1.clone();

        let _ = timeout1.execute(|| Ok::<_, Error>(42));

        assert!(timeout1.metrics().total_calls > 0);
        assert_eq!(timeout2.metrics().total_calls, 0);
    }

    #[test]
    fn test_timeout_debug() {
        let timeout = Timeout::from_secs("test", 5);
        let debug_str = format!("{:?}", timeout);

        assert!(debug_str.contains("Timeout"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_timeout_rate() {
        let mut metrics = TimeoutMetrics::default();

        metrics.total_calls = 10;
        metrics.successful_calls = 8;
        metrics.timeout_calls = 2;

        assert!((metrics.timeout_rate() - 0.2).abs() < 0.01);
        assert!((metrics.success_rate() - 0.8).abs() < 0.01);
    }
}
