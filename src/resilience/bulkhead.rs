//! Bulkhead pattern for isolating failures and limiting concurrent operations
//!
//! Prevents resource exhaustion by limiting the number of concurrent operations.
//!
//! # Features
//!
//! - **Concurrent limit** - Maximum number of simultaneous operations
//! - **Queue support** - Optional waiting queue for excess requests
//! - **Timeout** - Maximum wait time in queue
//! - **Metrics** - Track active operations, queue length
//! - **Isolation** - Separate bulkheads for different operation types
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::resilience::{Bulkhead, BulkheadConfig};
//!
//! let config = BulkheadConfig {
//!     max_concurrent: 10,
//!     max_wait_ms: 5000,
//!     ..Default::default()
//! };
//!
//! let bulkhead = Bulkhead::new("database", config);
//!
//! // Execute with bulkhead protection
//! let result = bulkhead.execute(|| {
//!     // Your operation here
//!     Ok::<_, pb_mcp_server::error::Error>(42)
//! });
//! ```

use crate::error::{Error, Result};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
// Note: AtomicU64, Ordering, and thread imports removed - not currently used
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Bulkhead Configuration
// ============================================================================

/// Configuration for bulkhead behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent: usize,

    /// Maximum time to wait in queue (milliseconds)
    pub max_wait_ms: u64,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Queue size for waiting operations (0 = no queue)
    pub queue_size: usize,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        BulkheadConfig {
            max_concurrent: 10,
            max_wait_ms: 5000,
            enable_metrics: true,
            queue_size: 20,
        }
    }
}

impl BulkheadConfig {
    /// Create configuration for strict limiting
    pub fn strict() -> Self {
        BulkheadConfig {
            max_concurrent: 5,
            max_wait_ms: 1000,
            enable_metrics: true,
            queue_size: 5,
        }
    }

    /// Create configuration for relaxed limiting
    pub fn relaxed() -> Self {
        BulkheadConfig {
            max_concurrent: 50,
            max_wait_ms: 30_000,
            enable_metrics: true,
            queue_size: 100,
        }
    }

    /// Create configuration for no queue (fail fast)
    pub fn no_queue(max_concurrent: usize) -> Self {
        BulkheadConfig {
            max_concurrent,
            max_wait_ms: 0,
            enable_metrics: true,
            queue_size: 0,
        }
    }
}

// ============================================================================
// Bulkhead Metrics
// ============================================================================

/// Metrics for bulkhead operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulkheadMetrics {
    /// Total operations attempted
    pub total_calls: u64,

    /// Operations that were accepted
    pub accepted_calls: u64,

    /// Operations that were rejected (bulkhead full)
    pub rejected_calls: u64,

    /// Operations that timed out waiting
    pub timeout_calls: u64,

    /// Current active operations
    pub current_active: u64,

    /// Current queue length
    pub current_queue_length: u64,

    /// Maximum concurrent operations reached
    pub max_concurrent_reached: u64,

    /// Average wait time in queue (ms)
    pub avg_wait_time_ms: f64,
}

impl BulkheadMetrics {
    /// Get rejection rate (0.0 to 1.0)
    pub fn rejection_rate(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.rejected_calls as f64 / self.total_calls as f64
        }
    }

    /// Get acceptance rate (0.0 to 1.0)
    pub fn acceptance_rate(&self) -> f64 {
        if self.total_calls == 0 {
            1.0
        } else {
            self.accepted_calls as f64 / self.total_calls as f64
        }
    }

    /// Get utilization rate (0.0 to 1.0)
    pub fn utilization_rate(&self, max_concurrent: usize) -> f64 {
        if max_concurrent == 0 {
            0.0
        } else {
            self.current_active as f64 / max_concurrent as f64
        }
    }
}

// ============================================================================
// Waiter Slot
// ============================================================================

/// Represents a waiting operation in the queue
#[allow(dead_code)]
struct Waiter {
    /// When this waiter was created
    created_at: Instant,

    /// Notifier for when slot is available
    notified: Arc<parking_lot::Condvar>,
}

impl Waiter {
    fn new() -> Self {
        Waiter {
            created_at: Instant::now(),
            notified: Arc::new(parking_lot::Condvar::new()),
        }
    }

    fn notify(&self) {
        self.notified.notify_one();
    }
}

// ============================================================================
// Bulkhead State
// ============================================================================

/// Internal state for bulkhead
struct BulkheadState {
    /// Current number of active operations
    active_count: usize,

    /// Queue of waiting operations
    wait_queue: VecDeque<Arc<Waiter>>,
}

// ============================================================================
// Bulkhead Implementation
// ============================================================================

/// Bulkhead for limiting concurrent operations
pub struct Bulkhead {
    /// Name of the bulkhead
    name: String,

    /// Configuration
    config: BulkheadConfig,

    /// Internal state
    state: Mutex<BulkheadState>,

    /// Metrics
    metrics: RwLock<BulkheadMetrics>,
}

impl Bulkhead {
    /// Create a new bulkhead
    pub fn new(name: &str, config: BulkheadConfig) -> Self {
        Bulkhead {
            name: name.to_string(),
            config,
            state: Mutex::new(BulkheadState {
                active_count: 0,
                wait_queue: VecDeque::new(),
            }),
            metrics: RwLock::new(BulkheadMetrics::default()),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(name: &str) -> Self {
        Self::new(name, BulkheadConfig::default())
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get configuration
    pub fn config(&self) -> &BulkheadConfig {
        &self.config
    }

    /// Get current metrics
    pub fn metrics(&self) -> BulkheadMetrics {
        self.metrics.read().clone()
    }

    /// Get current active count
    pub fn active_count(&self) -> usize {
        self.state.lock().active_count
    }

    /// Get current queue length
    pub fn queue_length(&self) -> usize {
        self.state.lock().wait_queue.len()
    }

    /// Check if bulkhead has available slots
    pub fn has_available_slots(&self) -> bool {
        self.state.lock().active_count < self.config.max_concurrent
    }

    /// Try to acquire a permit (non-blocking)
    fn try_acquire(&self) -> Result<()> {
        let mut state = self.state.lock();

        if state.active_count < self.config.max_concurrent {
            state.active_count += 1;

            // Update metrics
            let mut metrics = self.metrics.write();
            metrics.current_active = state.active_count as u64;
            if state.active_count as u64 > metrics.max_concurrent_reached {
                metrics.max_concurrent_reached = state.active_count as u64;
            }

            return Ok(());
        }

        Err(Error::Generic(format!(
            "Bulkhead '{}' is at capacity ({}/{})",
            self.name, state.active_count, self.config.max_concurrent
        )))
    }

    /// Acquire a permit, waiting if necessary
    fn acquire(&self) -> Result<()> {
        // Try to acquire immediately
        if self.try_acquire().is_ok() {
            return Ok(());
        }

        // If no queue, fail fast
        if self.config.queue_size == 0 {
            self.metrics.write().rejected_calls += 1;
            return Err(Error::Generic(format!(
                "Bulkhead '{}' is full and queuing is disabled",
                self.name
            )));
        }

        // Try to add to queue
        let waiter = Arc::new(Waiter::new());
        let wait_start = Instant::now();

        {
            let mut state = self.state.lock();

            if state.wait_queue.len() >= self.config.queue_size {
                self.metrics.write().rejected_calls += 1;
                return Err(Error::Generic(format!(
                    "Bulkhead '{}' queue is full ({}/{})",
                    self.name,
                    state.wait_queue.len(),
                    self.config.queue_size
                )));
            }

            state.wait_queue.push_back(waiter.clone());
        }

        // Wait for notification or timeout
        let timeout = Duration::from_millis(self.config.max_wait_ms);
        let mutex = Mutex::new(());
        let mut guard = mutex.lock();
        let timed_out = waiter.notified.wait_for(&mut guard, timeout).timed_out();

        if timed_out {
            // Remove from queue
            self.remove_waiter(&waiter);
            self.metrics.write().timeout_calls += 1;
            return Err(Error::Generic(format!(
                "Bulkhead '{}' wait timeout after {}ms",
                self.name, self.config.max_wait_ms
            )));
        }

        // Successfully acquired after waiting
        let wait_time = wait_start.elapsed().as_millis() as f64;
        let mut metrics = self.metrics.write();
        metrics.avg_wait_time_ms = (metrics.avg_wait_time_ms + wait_time) / 2.0;

        Ok(())
    }

    /// Release a permit
    fn release(&self) {
        let mut state = self.state.lock();
        state.active_count = state.active_count.saturating_sub(1);

        // Notify next waiter if any
        if let Some(waiter) = state.wait_queue.pop_front() {
            state.active_count += 1;
            waiter.notify();
        }

        // Update metrics
        self.metrics.write().current_active = state.active_count as u64;
    }

    /// Remove a waiter from the queue
    fn remove_waiter(&self, waiter: &Arc<Waiter>) {
        let mut state = self.state.lock();
        state.wait_queue.retain(|w| !Arc::ptr_eq(w, waiter));
    }

    /// Execute an operation with bulkhead protection
    pub fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        self.metrics.write().total_calls += 1;

        // Acquire permit
        self.acquire()?;

        self.metrics.write().accepted_calls += 1;

        // Execute operation
        let result = operation();

        // Release permit
        self.release();

        result
    }

    /// Try to execute without waiting
    pub fn try_execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        self.metrics.write().total_calls += 1;

        // Try to acquire permit
        if let Err(e) = self.try_acquire() {
            self.metrics.write().rejected_calls += 1;
            return Err(e);
        }

        self.metrics.write().accepted_calls += 1;

        // Execute operation
        let result = operation();

        // Release permit
        self.release();

        result
    }

    /// Reset metrics
    pub fn reset(&self) {
        *self.metrics.write() = BulkheadMetrics::default();
    }
}

impl std::fmt::Debug for Bulkhead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bulkhead")
            .field("name", &self.name)
            .field("max_concurrent", &self.config.max_concurrent)
            .field("active_count", &self.active_count())
            .field("queue_length", &self.queue_length())
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_bulkhead_config_default() {
        let config = BulkheadConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.max_wait_ms, 5000);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_bulkhead_config_presets() {
        let strict = BulkheadConfig::strict();
        assert!(strict.max_concurrent < 10);

        let relaxed = BulkheadConfig::relaxed();
        assert!(relaxed.max_concurrent > 10);

        let no_queue = BulkheadConfig::no_queue(5);
        assert_eq!(no_queue.max_concurrent, 5);
        assert_eq!(no_queue.queue_size, 0);
    }

    #[test]
    fn test_bulkhead_creation() {
        let bulkhead = Bulkhead::with_defaults("test");
        assert_eq!(bulkhead.name(), "test");
        assert_eq!(bulkhead.active_count(), 0);
        assert!(bulkhead.has_available_slots());
    }

    #[test]
    fn test_bulkhead_execute_success() {
        let bulkhead = Bulkhead::new("test", BulkheadConfig::no_queue(5));

        let result = bulkhead.execute(|| Ok::<_, Error>(42));
        assert_eq!(result.unwrap(), 42);

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.accepted_calls, 1);
        assert_eq!(bulkhead.active_count(), 0);
    }

    #[test]
    fn test_bulkhead_concurrent_limit() {
        let bulkhead = Arc::new(Bulkhead::new("test", BulkheadConfig::no_queue(2)));

        let mut handles = vec![];

        for _ in 0..5 {
            let bh = bulkhead.clone();
            let handle = thread::spawn(move || {
                bh.execute(|| {
                    thread::sleep(Duration::from_millis(100));
                    Ok::<_, Error>(42)
                })
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.iter().filter(|r| r.is_err()).count();

        // Should have exactly 2 successes (max_concurrent)
        assert_eq!(success_count, 2);
        assert!(failure_count >= 3);

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.rejected_calls, failure_count as u64);
    }

    #[test]
    fn test_bulkhead_try_execute() {
        let bulkhead = Arc::new(Bulkhead::new("test", BulkheadConfig::no_queue(1)));

        // Use threads to test concurrent access
        let bh = bulkhead.clone();
        let handle1 = thread::spawn(move || {
            bh.try_execute(|| {
                thread::sleep(Duration::from_millis(200)); // Hold the permit
                Ok::<_, Error>(1)
            })
        });

        // Wait a bit for first thread to acquire
        thread::sleep(Duration::from_millis(50));

        // Second should fail (first thread holds the permit)
        let result = bulkhead.try_execute(|| Ok::<_, Error>(2));
        assert!(result.is_err());

        // Wait for first to complete
        handle1.join().unwrap().unwrap();
    }

    #[test]
    fn test_bulkhead_metrics() {
        let bulkhead = Bulkhead::new("test", BulkheadConfig::no_queue(10));

        for i in 0..10 {
            let _ = bulkhead.execute(|| Ok::<_, Error>(i));
        }

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.total_calls, 10);
        assert_eq!(metrics.accepted_calls, 10);
        assert_eq!(metrics.rejected_calls, 0);
        assert!((metrics.acceptance_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_bulkhead_rejection_rate() {
        let bulkhead = Arc::new(Bulkhead::new("test", BulkheadConfig::no_queue(2)));

        // Spawn multiple threads to test concurrent rejection
        let mut handles = vec![];
        for _ in 0..10 {
            let bh = bulkhead.clone();
            let handle = thread::spawn(move || {
                bh.try_execute(|| {
                    thread::sleep(Duration::from_millis(100)); // Hold permit
                    Ok::<_, Error>(42)
                })
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let failure_count = results.iter().filter(|r| r.is_err()).count();

        // Should have at most 2 successes (max_concurrent)
        assert!(success_count <= 2);
        assert!(failure_count >= 8);

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.total_calls, 10);
        assert_eq!(metrics.accepted_calls, success_count as u64);
        assert_eq!(metrics.rejected_calls, failure_count as u64);
    }

    #[test]
    fn test_bulkhead_reset() {
        let bulkhead = Bulkhead::new("test", BulkheadConfig::no_queue(10));

        for i in 0..10 {
            let _ = bulkhead.execute(|| Ok::<_, Error>(i));
        }

        assert!(bulkhead.metrics().total_calls > 0);

        bulkhead.reset();

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.total_calls, 0);
        assert_eq!(metrics.accepted_calls, 0);
    }

    #[test]
    fn test_bulkhead_debug() {
        let bulkhead = Bulkhead::with_defaults("test");
        let debug_str = format!("{:?}", bulkhead);

        assert!(debug_str.contains("Bulkhead"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_bulkhead_utilization() {
        let config = BulkheadConfig::no_queue(10);
        let metrics = BulkheadMetrics {
            current_active: 5,
            ..Default::default()
        };

        let utilization = metrics.utilization_rate(config.max_concurrent);
        assert!((utilization - 0.5).abs() < 0.01);
    }
}
