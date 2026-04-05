//! Resilience patterns for pb-mcp-server
//!
//! This module provides enterprise-grade resilience patterns including:
//! - **Circuit Breaker** - Prevent cascading failures
//! - **Retry with Backoff** - Handle transient failures
//! - **Bulkhead** - Isolate failures and limit concurrent operations
//! - **Timeout** - Prevent hanging operations
//!
//! # Features
//!
//! - Thread-safe implementations
//! - Configurable thresholds
//! - Metrics and monitoring
//! - Async-friendly
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::resilience::{CircuitBreaker, CircuitBreakerConfig};
//!
//! let config = CircuitBreakerConfig::default();
//! let breaker = CircuitBreaker::new("database", config);
//!
//! // Execute protected operation
//! match breaker.call(|| {
//!     // Your operation here
//!     Ok::<_, pb_mcp_server::error::Error>(42)
//! }) {
//!     Ok(result) => println!("Success: {}", result),
//!     Err(e) => println!("Failed: {}", e),
//! }
//! ```

pub mod bulkhead;
pub mod circuit_breaker;
pub mod retry;
pub mod timeout;

pub use bulkhead::{Bulkhead, BulkheadConfig, BulkheadMetrics};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use retry::{RetryConfig, RetryMetrics, RetryPolicy};
pub use timeout::{Deadline, Timeout, TimeoutConfig, TimeoutMetrics};

use crate::error::{Error, Result};
// Note: std::time::Duration import removed - not currently used in this module

// ============================================================================
// Common Types
// ============================================================================

/// Result of a resilience operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResilienceResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed but was handled
    Failure(String),
    /// Circuit breaker is open
    CircuitOpen,
    /// Timeout occurred
    Timeout,
    /// Bulkhead is full
    BulkheadFull,
    /// Operation was rejected
    Rejected(&'static str),
}

impl<T> ResilienceResult<T> {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self, ResilienceResult::Success(_))
    }

    /// Check if the result is a failure
    pub fn is_failure(&self) -> bool {
        matches!(self, ResilienceResult::Failure(_))
    }

    /// Get the value if successful
    pub fn ok(self) -> Option<T> {
        match self {
            ResilienceResult::Success(value) => Some(value),
            _ => None,
        }
    }

    /// Convert to a Result type
    pub fn into_result(self) -> Result<T> {
        match self {
            ResilienceResult::Success(value) => Ok(value),
            ResilienceResult::Failure(err) => Err(Error::Generic(err)),
            ResilienceResult::CircuitOpen => {
                Err(Error::Generic("Circuit breaker is open".to_string()))
            }
            ResilienceResult::Timeout => Err(Error::timeout(30000, "operation")),
            ResilienceResult::BulkheadFull => Err(Error::Generic("Bulkhead is full".to_string())),
            ResilienceResult::Rejected(reason) => {
                Err(Error::Generic(format!("Operation rejected: {}", reason)))
            }
        }
    }
}

// ============================================================================
// Metrics for Resilience Patterns
// ============================================================================

/// Metrics for resilience patterns
#[derive(Debug, Clone, Default)]
pub struct ResilienceMetrics {
    /// Total operations attempted
    pub total_attempts: u64,

    /// Successful operations
    pub successes: u64,

    /// Failed operations
    pub failures: u64,

    /// Operations rejected by circuit breaker
    pub circuit_rejections: u64,

    /// Operations that timed out
    pub timeouts: u64,

    /// Operations rejected by bulkhead
    pub bulkhead_rejections: u64,

    /// Retries attempted
    pub retries: u64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
}

impl ResilienceMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        ResilienceMetrics::default()
    }

    /// Record a successful operation
    pub fn record_success(&mut self, latency_ms: f64) {
        self.total_attempts += 1;
        self.successes += 1;
        self.update_avg_latency(latency_ms);
    }

    /// Record a failed operation
    pub fn record_failure(&mut self, latency_ms: f64) {
        self.total_attempts += 1;
        self.failures += 1;
        self.update_avg_latency(latency_ms);
    }

    /// Record a circuit breaker rejection
    pub fn record_circuit_rejection(&mut self) {
        self.total_attempts += 1;
        self.circuit_rejections += 1;
    }

    /// Record a timeout
    pub fn record_timeout(&mut self) {
        self.total_attempts += 1;
        self.timeouts += 1;
    }

    /// Record a bulkhead rejection
    pub fn record_bulkhead_rejection(&mut self) {
        self.total_attempts += 1;
        self.bulkhead_rejections += 1;
    }

    /// Record a retry
    pub fn record_retry(&mut self) {
        self.retries += 1;
    }

    /// Update average latency
    fn update_avg_latency(&mut self, latency_ms: f64) {
        let total = self.successes + self.failures;
        if total == 1 {
            self.avg_latency_ms = latency_ms;
        } else {
            self.avg_latency_ms =
                (self.avg_latency_ms * (total - 1) as f64 + latency_ms) / total as f64;
        }
    }

    /// Get success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            1.0
        } else {
            self.successes as f64 / self.total_attempts as f64
        }
    }

    /// Get failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.failures as f64 / self.total_attempts as f64
        }
    }

    /// Get rejection rate (0.0 to 1.0)
    pub fn rejection_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.circuit_rejections + self.bulkhead_rejections) as f64 / self.total_attempts as f64
        }
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        *self = ResilienceMetrics::default();
    }
}

impl std::fmt::Display for ResilienceMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Resilience Metrics:")?;
        writeln!(f, "  Total Attempts: {}", self.total_attempts)?;
        writeln!(f, "  Successes: {}", self.successes)?;
        writeln!(f, "  Failures: {}", self.failures)?;
        writeln!(f, "  Circuit Rejections: {}", self.circuit_rejections)?;
        writeln!(f, "  Timeouts: {}", self.timeouts)?;
        writeln!(f, "  Bulkhead Rejections: {}", self.bulkhead_rejections)?;
        writeln!(f, "  Retries: {}", self.retries)?;
        writeln!(f, "  Success Rate: {:.2}%", self.success_rate() * 100.0)?;
        writeln!(f, "  Failure Rate: {:.2}%", self.failure_rate() * 100.0)?;
        writeln!(f, "  Avg Latency: {:.2}ms", self.avg_latency_ms)?;
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
    fn test_resilience_result_success() {
        let result = ResilienceResult::Success(42);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_resilience_result_failure() {
        let result: ResilienceResult<()> = ResilienceResult::Failure("test error".to_string());
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert!(result.ok().is_none());
    }

    #[test]
    fn test_resilience_result_into_result() {
        let success = ResilienceResult::Success(42);
        assert!(success.into_result().is_ok());

        let circuit_open: ResilienceResult<()> = ResilienceResult::CircuitOpen;
        assert!(circuit_open.into_result().is_err());
    }

    #[test]
    fn test_resilience_metrics() {
        let mut metrics = ResilienceMetrics::new();

        metrics.record_success(10.0);
        metrics.record_success(20.0);
        metrics.record_failure(15.0);
        metrics.record_circuit_rejection();

        assert_eq!(metrics.total_attempts, 4);
        assert_eq!(metrics.successes, 2);
        assert_eq!(metrics.failures, 1);
        assert_eq!(metrics.circuit_rejections, 1);

        let success_rate = metrics.success_rate();
        assert!((success_rate - 0.5).abs() < 0.01);

        let avg_latency = metrics.avg_latency_ms;
        assert!((avg_latency - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_resilience_metrics_rates() {
        let mut metrics = ResilienceMetrics::new();

        // Empty metrics
        assert!((metrics.success_rate() - 1.0).abs() < 0.01);
        assert!((metrics.failure_rate() - 0.0).abs() < 0.01);

        // Add some data
        for _ in 0..10 {
            metrics.record_success(10.0);
        }
        for _ in 0..5 {
            metrics.record_failure(10.0);
        }

        let success_rate = metrics.success_rate();
        let failure_rate = metrics.failure_rate();

        assert!((success_rate - 0.666).abs() < 0.01);
        assert!((failure_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_resilience_metrics_reset() {
        let mut metrics = ResilienceMetrics::new();

        metrics.record_success(10.0);
        metrics.record_failure(10.0);

        assert!(metrics.total_attempts > 0);

        metrics.reset();

        assert_eq!(metrics.total_attempts, 0);
        assert_eq!(metrics.successes, 0);
        assert_eq!(metrics.failures, 0);
    }

    #[test]
    fn test_resilience_metrics_display() {
        let mut metrics = ResilienceMetrics::new();
        metrics.record_success(10.0);

        let output = format!("{}", metrics);
        assert!(output.contains("Total Attempts: 1"));
        assert!(output.contains("Successes: 1"));
        assert!(output.contains("Success Rate:"));
    }

    #[test]
    fn test_resilience_metrics_retries() {
        let mut metrics = ResilienceMetrics::new();

        metrics.record_retry();
        metrics.record_retry();

        assert_eq!(metrics.retries, 2);
    }

    #[test]
    fn test_resilience_metrics_rejection_rate() {
        let mut metrics = ResilienceMetrics::new();

        metrics.record_success(10.0);
        metrics.record_circuit_rejection();
        metrics.record_bulkhead_rejection();

        let rejection_rate = metrics.rejection_rate();
        assert!((rejection_rate - 0.666).abs() < 0.01);
    }
}
