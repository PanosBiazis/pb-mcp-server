//! Retry pattern with exponential backoff
//!
//! Provides configurable retry logic for handling transient failures.
//!
//! # Features
//!
//! - **Exponential backoff** - Increasing delays between retries
//! - **Jitter** - Randomization to prevent thundering herd
//! - **Max retries** - Configurable retry limit
//! - **Retry predicates** - Control which errors to retry
//! - **Circuit breaker integration** - Works with circuit breakers
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::resilience::{RetryPolicy, RetryConfig};
//! use pb_mcp_server::error::{Error, Result};
//!
//! let config = RetryConfig {
//!     max_retries: 3,
//!     initial_delay_ms: 100,
//!     max_delay_ms: 5000,
//!     multiplier: 2.0,
//!     ..Default::default()
//! };
//!
//! let policy = RetryPolicy::new(config);
//!
//! let result = policy.execute(|| {
//!     // Operation that might fail
//!     Ok::<_, Error>(42)
//! });
//! ```

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Retry Configuration
// ============================================================================

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,

    /// Multiplier for exponential backoff
    pub multiplier: f64,

    /// Jitter factor (0.0 to 1.0) - adds randomness to delays
    pub jitter_factor: f64,

    /// Whether to retry on timeout errors
    pub retry_on_timeout: bool,

    /// Whether to retry on connection errors
    pub retry_on_connection_error: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30_000,
            multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_timeout: true,
            retry_on_connection_error: true,
            enable_metrics: true,
        }
    }
}

impl RetryConfig {
    /// Create configuration for aggressive retrying
    pub fn aggressive() -> Self {
        RetryConfig {
            max_retries: 5,
            initial_delay_ms: 50,
            max_delay_ms: 10_000,
            multiplier: 1.5,
            jitter_factor: 0.2,
            retry_on_timeout: true,
            retry_on_connection_error: true,
            enable_metrics: true,
        }
    }

    /// Create configuration for conservative retrying
    pub fn conservative() -> Self {
        RetryConfig {
            max_retries: 2,
            initial_delay_ms: 500,
            max_delay_ms: 60_000,
            multiplier: 3.0,
            jitter_factor: 0.1,
            retry_on_timeout: true,
            retry_on_connection_error: true,
            enable_metrics: true,
        }
    }

    /// Create configuration for no retrying
    pub fn no_retry() -> Self {
        RetryConfig {
            max_retries: 0,
            initial_delay_ms: 0,
            max_delay_ms: 0,
            multiplier: 1.0,
            jitter_factor: 0.0,
            retry_on_timeout: false,
            retry_on_connection_error: false,
            enable_metrics: false,
        }
    }
}

// ============================================================================
// Retry Metrics
// ============================================================================

/// Metrics for retry operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryMetrics {
    /// Total operations attempted
    pub total_attempts: u64,

    /// Successful operations (without retry)
    pub immediate_successes: u64,

    /// Successful operations after retry
    pub retry_successes: u64,

    /// Operations that exhausted retries
    pub exhausted_retries: u64,

    /// Total retries attempted
    pub total_retries: u64,

    /// Total time spent in retries (ms)
    pub total_retry_time_ms: u64,
}

impl RetryMetrics {
    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            1.0
        } else {
            (self.immediate_successes + self.retry_successes) as f64 / self.total_attempts as f64
        }
    }

    /// Get retry rate (0.0 to 1.0)
    pub fn retry_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.total_retries as f64 / self.total_attempts as f64
        }
    }

    /// Get average retries per operation
    pub fn avg_retries(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.total_retries as f64 / self.total_attempts as f64
        }
    }
}

// ============================================================================
// Retry Policy
// ============================================================================

/// Retry policy for handling transient failures
pub struct RetryPolicy {
    /// Configuration
    config: RetryConfig,

    /// Metrics
    metrics: Arc<parking_lot::RwLock<RetryMetrics>>,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        RetryPolicy {
            config,
            metrics: Arc::new(parking_lot::RwLock::new(RetryMetrics::default())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Get configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }

    /// Get metrics
    pub fn metrics(&self) -> RetryMetrics {
        self.metrics.read().clone()
    }

    /// Calculate delay for a given attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let base_delay_ms =
            self.config.initial_delay_ms as f64 * self.config.multiplier.powi(attempt as i32 - 1);

        let capped_delay_ms = base_delay_ms.min(self.config.max_delay_ms as f64);

        // Apply jitter
        let jitter = if self.config.jitter_factor > 0.0 {
            let jitter_range = capped_delay_ms * self.config.jitter_factor;
            let random_factor = (rand_simple() as f64 / u64::MAX as f64) * 2.0 - 1.0;
            capped_delay_ms + random_factor * jitter_range
        } else {
            capped_delay_ms
        };

        Duration::from_millis(jitter.max(0.0) as u64)
    }

    /// Check if an error should be retried
    pub fn should_retry(&self, error: &Error, attempt: u32) -> bool {
        if attempt >= self.config.max_retries {
            return false;
        }

        // Check error type
        match error {
            Error::Timeout { .. } => self.config.retry_on_timeout,
            Error::FileSystem { .. } => true, // File system errors are often transient
            Error::Git { .. } => true,        // Git operations can be retried
            Error::RateLimitExceeded { .. } => true,
            Error::BusLockError { .. } => true,
            Error::Generic(_) => false, // Generic errors shouldn't be retried by default
            _ => false,
        }
    }

    /// Execute an operation with retry logic
    pub fn execute<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let _start = Instant::now();
        let mut attempt = 0;
        let mut total_retry_time = Duration::from_secs(0);

        self.metrics.write().total_attempts += 1;

        loop {
            match operation() {
                Ok(result) => {
                    let mut metrics = self.metrics.write();
                    if attempt == 0 {
                        metrics.immediate_successes += 1;
                    } else {
                        metrics.retry_successes += 1;
                    }
                    return Ok(result);
                }
                Err(error) => {
                    attempt += 1;

                    // Check if we've exhausted retries
                    if attempt > self.config.max_retries {
                        self.metrics.write().exhausted_retries += 1;
                        return Err(error);
                    }

                    // Check if this error type should be retried
                    if !self.should_retry(&error, attempt - 1) {
                        return Err(error);
                    }

                    let delay = self.calculate_delay(attempt);
                    total_retry_time += delay;

                    self.metrics.write().total_retries += 1;

                    // Wait before retry
                    thread::sleep(delay);
                }
            }
        }
    }

    /// Execute an async operation with retry logic
    pub async fn execute_async<F, T, Fut>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;

        self.metrics.write().total_attempts += 1;

        loop {
            match operation().await {
                Ok(result) => {
                    let mut metrics = self.metrics.write();
                    if attempt == 0 {
                        metrics.immediate_successes += 1;
                    } else {
                        metrics.retry_successes += 1;
                    }
                    return Ok(result);
                }
                Err(error) => {
                    if !self.should_retry(&error, attempt) {
                        return Err(error);
                    }

                    attempt += 1;
                    if attempt > self.config.max_retries {
                        let mut metrics = self.metrics.write();
                        metrics.exhausted_retries += 1;
                        return Err(error);
                    }

                    let delay = self.calculate_delay(attempt);
                    self.metrics.write().total_retries += 1;

                    // Wait before retry
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Reset metrics
    pub fn reset(&self) {
        *self.metrics.write() = RetryMetrics::default();
    }
}

impl Clone for RetryPolicy {
    fn clone(&self) -> Self {
        RetryPolicy {
            config: self.config.clone(),
            metrics: Arc::new(parking_lot::RwLock::new(self.metrics.read().clone())),
        }
    }
}

// Simple random number generator for jitter
fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert!(config.multiplier > 1.0);
    }

    #[test]
    fn test_retry_config_presets() {
        let aggressive = RetryConfig::aggressive();
        assert!(aggressive.max_retries > 3);

        let conservative = RetryConfig::conservative();
        assert!(conservative.max_retries < 3);

        let no_retry = RetryConfig::no_retry();
        assert_eq!(no_retry.max_retries, 0);
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable test
            ..Default::default()
        };
        let policy = RetryPolicy::new(config);

        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0, Duration::from_millis(0));

        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_millis(100));

        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_millis(200));

        let delay3 = policy.calculate_delay(3);
        assert_eq!(delay3, Duration::from_millis(400));
    }

    #[test]
    fn test_calculate_delay_with_max() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay_ms: 100,
            max_delay_ms: 500,
            multiplier: 10.0,
            jitter_factor: 0.0,
            ..Default::default()
        };
        let policy = RetryPolicy::new(config);

        let delay3 = policy.calculate_delay(3);
        // Would be 10000 without cap, but should be capped at 500
        assert_eq!(delay3, Duration::from_millis(500));
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::with_defaults();

        // Should retry timeout errors
        assert!(policy.should_retry(&Error::timeout(1000, "test"), 0));

        // Should not retry after max attempts
        assert!(!policy.should_retry(&Error::timeout(1000, "test"), 3));

        // Should retry file system errors
        use std::path::PathBuf;
        assert!(policy.should_retry(
            &Error::file_system(
                &PathBuf::from("test"),
                std::io::Error::new(std::io::ErrorKind::Other, "test")
            ),
            0
        ));
    }

    #[test]
    fn test_execute_immediate_success() {
        let policy = RetryPolicy::with_defaults();

        let result: Result<i32> = policy.execute(|| Ok(42));

        assert_eq!(result.unwrap(), 42);

        let metrics = policy.metrics();
        assert_eq!(metrics.total_attempts, 1);
        assert_eq!(metrics.immediate_successes, 1);
        assert_eq!(metrics.total_retries, 0);
    }

    #[test]
    fn test_execute_success_after_retry() {
        let policy = RetryPolicy::new(RetryConfig {
            max_retries: 3,
            initial_delay_ms: 1, // Very short for testing
            max_delay_ms: 10,
            multiplier: 1.0,
            jitter_factor: 0.0,
            ..Default::default()
        });

        let mut attempts = 0;
        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err(Error::timeout(100, "test"))
            } else {
                Ok(42)
            }
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);

        let metrics = policy.metrics();
        assert_eq!(metrics.total_attempts, 1);
        assert_eq!(metrics.retry_successes, 1);
        assert_eq!(metrics.total_retries, 2);
    }

    #[test]
    fn test_execute_exhausted_retries() {
        let policy = RetryPolicy::new(RetryConfig {
            max_retries: 2,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            multiplier: 1.0,
            jitter_factor: 0.0,
            ..Default::default()
        });

        let mut attempts = 0;
        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            Err(Error::timeout(100, "test"))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 3); // Initial + 2 retries

        let metrics = policy.metrics();
        assert_eq!(metrics.exhausted_retries, 1);
    }

    #[test]
    fn test_execute_no_retry_on_non_retryable_error() {
        let policy = RetryPolicy::with_defaults();

        let mut attempts = 0;
        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            Err(Error::Generic("non-retryable".to_string()))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 1); // Should not retry
    }

    #[test]
    fn test_retry_metrics() {
        let mut metrics = RetryMetrics::default();

        metrics.total_attempts = 10;
        metrics.immediate_successes = 7;
        metrics.retry_successes = 2;
        metrics.exhausted_retries = 1;
        metrics.total_retries = 5;

        assert!((metrics.success_rate() - 0.9).abs() < 0.01);
        assert!((metrics.retry_rate() - 0.5).abs() < 0.01);
        assert!((metrics.avg_retries() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let policy = RetryPolicy::with_defaults();

        // Execute some operations
        let _: Result<i32> = policy.execute(|| Ok(42));

        let metrics = policy.metrics();
        assert!(metrics.total_attempts > 0);

        // Reset
        policy.reset();

        let metrics = policy.metrics();
        assert_eq!(metrics.total_attempts, 0);
        assert_eq!(metrics.immediate_successes, 0);
    }

    #[test]
    fn test_clone() {
        let policy1 = RetryPolicy::with_defaults();
        let policy2 = policy1.clone();

        // Execute on policy1
        let _: Result<i32> = policy1.execute(|| Ok(42));

        // Metrics should be separate
        assert!(policy1.metrics().total_attempts > 0);
        assert_eq!(policy2.metrics().total_attempts, 0);
    }
}
