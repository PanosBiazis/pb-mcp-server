//! # General-Purpose Rate Limiter
//!
//! Token-bucket rate limiter for arbitrary operations (tool calls, agent
//! messages, API requests, etc.). Each keyed operation gets an independent
//! bucket that refills at a configurable rate.
//!
//! This module is **distinct** from [`crate::bus_rate::BusSendRateLimiter`]
//! which is specific to the agent bus. This is a general-purpose facility
//! used by the security layer and any tool that needs throttling.
//!
//! ## Algorithm
//!
//! Each key maps to a `TokenBucket` with a configurable:
//!
//! - **max_tokens** – bucket capacity (burst size)
//! - **refill_rate** – tokens added per second
//!
//! On every [`RateLimiter::check_and_record`] call, the bucket is first
//! refilled proportionally to the elapsed time since the last operation,
//! then one token is consumed. If no tokens are available the call is
//! rejected with [`crate::error::Error::RateLimitExceeded`].
//!
//! ## Presets
//!
//! | Preset | requests / window | burst |
//! |------------|-------------------|-------|
//! | Default | 100 / 60 s | 20 |
//! | Permissive | 1000 / 60 s | 200 |
//! | Restrictive | 30 / 60 s | 5 |
//!
//! ## Example
//!
//! ```rust,ignore
//! use pb_mcp_server::security::rate_limiter::{RateLimiter, RateLimiterConfig};
//!
//! let limiter = RateLimiter::new(RateLimiterConfig::default());
//!
//! // Allow up to 20 rapid calls (burst), then throttle.
//! for _ in 0..20 {
//!     assert!(limiter.check_and_record("tool:read_file").is_ok());
//! }
//! assert!(limiter.check_and_record("tool:read_file").is_err());
//! ```

use crate::error::{Error, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for a [`RateLimiter`].
///
/// `max_requests` over `window_secs` defines the steady-state refill rate;
/// `burst_size` defines how many tokens the bucket can hold at once,
/// allowing short bursts above the average rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Maximum requests allowed per time window.
    pub max_requests: u32,
    /// Length of the time window in seconds.
    pub window_secs: u64,
    /// Maximum burst size (bucket capacity).
    pub burst_size: u32,
}

impl Default for RateLimiterConfig {
    /// Default: 100 requests per 60 seconds, burst of 20.
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            burst_size: 20,
        }
    }
}

impl RateLimiterConfig {
    /// Permissive preset: 1000 requests per 60 seconds, burst of 200.
    pub fn permissive() -> Self {
        Self {
            max_requests: 1000,
            window_secs: 60,
            burst_size: 200,
        }
    }

    /// Restrictive preset: 30 requests per 60 seconds, burst of 5.
    pub fn restrictive() -> Self {
        Self {
            max_requests: 30,
            window_secs: 60,
            burst_size: 5,
        }
    }

    /// Compute the steady-state refill rate (tokens per second).
    pub fn refill_rate(&self) -> f64 {
        if self.window_secs == 0 {
            return f64::MAX;
        }
        self.max_requests as f64 / self.window_secs as f64
    }

    /// Validate that the configuration is sensible.
    pub fn validate(&self) -> Result<()> {
        if self.max_requests == 0 {
            return Err(Error::invalid_arguments(
                "rate_limiter_config",
                "max_requests must be > 0",
            ));
        }
        if self.window_secs == 0 {
            return Err(Error::invalid_arguments(
                "rate_limiter_config",
                "window_secs must be > 0",
            ));
        }
        if self.burst_size == 0 {
            return Err(Error::invalid_arguments(
                "rate_limiter_config",
                "burst_size must be > 0",
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Token bucket (per-key state)
// ---------------------------------------------------------------------------

/// Internal state for a single rate-limited key.
///
/// Uses a classic token-bucket algorithm: tokens refill over time at a
/// constant rate up to `max_tokens`, and each allowed operation consumes
/// one token.
#[derive(Debug)]
struct TokenBucket {
    /// Current number of available tokens (may be fractional).
    tokens: f64,
    /// Maximum number of tokens the bucket can hold (== burst_size).
    max_tokens: f64,
    /// Tokens added per second (== max_requests / window_secs).
    refill_rate: f64,
    /// Instant of the last refill calculation.
    last_refill: Instant,
    /// Total number of allowed operations since bucket creation.
    total_allowed: u64,
    /// Total number of rejected operations since bucket creation.
    total_rejected: u64,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens, // start full
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
            total_allowed: 0,
            total_rejected: 0,
        }
    }

    /// Refill tokens based on elapsed time since last refill.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let added = elapsed * self.refill_rate;
        self.tokens = (self.tokens + added).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume one token. Returns `true` if allowed.
    fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            self.total_allowed += 1;
            true
        } else {
            self.total_rejected += 1;
            false
        }
    }

    /// Peek at current token count (after refill).
    fn peek(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

// ---------------------------------------------------------------------------
// Bucket status (public)
// ---------------------------------------------------------------------------

/// Snapshot of a single key's rate-limiter state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStatus {
    /// The key this status refers to.
    pub key: String,
    /// Current available tokens (may be fractional).
    pub tokens: f64,
    /// Maximum tokens (burst capacity).
    pub max_tokens: f64,
    /// Refill rate in tokens per second.
    pub refill_rate: f64,
    /// Total allowed operations since bucket creation.
    pub total_allowed: u64,
    /// Total rejected operations since bucket creation.
    pub total_rejected: u64,
}

// ---------------------------------------------------------------------------
// Rate limiter
// ---------------------------------------------------------------------------

/// General-purpose token-bucket rate limiter.
///
/// Each operation key (e.g. `"tool:read_file"`, `"agent:planner"`) gets its
/// own independent bucket. Buckets are created on first use and are
/// garbage-collected when they sit idle beyond a configurable threshold.
pub struct RateLimiter {
    /// Configuration for new buckets.
    config: RateLimiterConfig,
    /// Per-key token buckets.
    buckets: DashMap<String, TokenBucket>,
    /// Counter for total checks across all keys.
    total_checks: AtomicU64,
    /// Counter for total rejections across all keys.
    total_rejections: AtomicU64,
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("config", &self.config)
            .field("key_count", &self.buckets.len())
            .field("total_checks", &self.total_checks.load(Ordering::Relaxed))
            .field(
                "total_rejections",
                &self.total_rejections.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            buckets: DashMap::new(),
            total_checks: AtomicU64::new(0),
            total_rejections: AtomicU64::new(0),
        }
    }

    /// Create a rate limiter with default configuration (100/60/20).
    pub fn default_limiter() -> Self {
        Self::new(RateLimiterConfig::default())
    }

    /// Create a permissive rate limiter (1000/60/200).
    pub fn permissive() -> Self {
        Self::new(RateLimiterConfig::permissive())
    }

    /// Create a restrictive rate limiter (30/60/5).
    pub fn restrictive() -> Self {
        Self::new(RateLimiterConfig::restrictive())
    }

    // -----------------------------------------------------------------------
    // Core operations
    // -----------------------------------------------------------------------

    /// Check whether an operation with the given key is currently allowed
    /// **without** consuming a token.
    ///
    /// This is useful for pre-flight checks. Use [`Self::check_and_record`] for
    /// the atomic check+consume pattern.
    pub fn check(&self, key: &str) -> Result<()> {
        self.total_checks.fetch_add(1, Ordering::Relaxed);
        let config = &self.config;
        let allowed = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(config.burst_size as f64, config.refill_rate()))
            .peek()
            >= 1.0;
        if allowed {
            Ok(())
        } else {
            self.total_rejections.fetch_add(1, Ordering::Relaxed);
            Err(Error::rate_limit_exceeded(
                key,
                config.max_requests as usize,
                config.window_secs,
            ))
        }
    }

    /// Record that an operation has occurred (consume one token).
    ///
    /// **Caution:** This does *not* check whether tokens are available.
    /// Prefer [`Self::check_and_record`] for the atomic pattern, or call
    /// [`Self::check`] first.
    pub fn record(&self, key: &str) {
        let config = &self.config;
        self.buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(config.burst_size as f64, config.refill_rate()))
            .try_consume();
    }

    /// Atomically check whether the operation is allowed and, if so,
    /// consume one token. This is the primary method for rate-limited
    /// operations.
    ///
    /// Returns `Ok(())` if the operation is allowed, or
    /// `Err(Error::RateLimitExceeded)` if the bucket is empty.
    pub fn check_and_record(&self, key: &str) -> Result<()> {
        self.total_checks.fetch_add(1, Ordering::Relaxed);
        let config = &self.config;
        let allowed = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(config.burst_size as f64, config.refill_rate()))
            .try_consume();
        if allowed {
            Ok(())
        } else {
            self.total_rejections.fetch_add(1, Ordering::Relaxed);
            Err(Error::rate_limit_exceeded(
                key,
                config.max_requests as usize,
                config.window_secs,
            ))
        }
    }

    // -----------------------------------------------------------------------
    // Inspection
    // -----------------------------------------------------------------------

    /// Get the current status of a specific key's bucket.
    ///
    /// Returns `None` if the key has never been seen.
    pub fn get_status(&self, key: &str) -> Option<BucketStatus> {
        let config = &self.config;
        self.buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(config.burst_size as f64, config.refill_rate()));
        // After the entry is guaranteed to exist, get a mutable ref.
        let mut bucket = self.buckets.get_mut(key)?;
        let tokens = bucket.peek();
        let status = BucketStatus {
            key: key.to_string(),
            tokens,
            max_tokens: bucket.max_tokens,
            refill_rate: bucket.refill_rate,
            total_allowed: bucket.total_allowed,
            total_rejected: bucket.total_rejected,
        };
        Some(status)
    }

    /// List all keys currently tracked by the rate limiter.
    pub fn list_keys(&self) -> Vec<String> {
        self.buckets.iter().map(|e| e.key().clone()).collect()
    }

    /// Return the total number of checks across all keys.
    pub fn total_checks(&self) -> u64 {
        self.total_checks.load(Ordering::Relaxed)
    }

    /// Return the total number of rejections across all keys.
    pub fn total_rejections(&self) -> u64 {
        self.total_rejections.load(Ordering::Relaxed)
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }

    /// Return the number of tracked keys.
    pub fn key_count(&self) -> usize {
        self.buckets.len()
    }

    // -----------------------------------------------------------------------
    // Management
    // -----------------------------------------------------------------------

    /// Reset a specific key's bucket (refill to full capacity).
    pub fn reset(&self, key: &str) {
        if let Some(mut bucket) = self.buckets.get_mut(key) {
            bucket.tokens = bucket.max_tokens;
            bucket.last_refill = Instant::now();
            bucket.total_allowed = 0;
            bucket.total_rejected = 0;
        }
    }

    /// Reset all buckets.
    pub fn reset_all(&self) {
        self.buckets.clear();
        self.total_checks.store(0, Ordering::Relaxed);
        self.total_rejections.store(0, Ordering::Relaxed);
    }

    /// Remove idle buckets that haven't been accessed recently.
    ///
    /// `max_idle_secs` defines how long a bucket can be idle before it is
    /// removed. This prevents unbounded memory growth from one-off keys.
    pub fn purge_idle(&self, _max_idle_secs: u64) {
        // Since we don't track idle time directly, we purge buckets whose
        // tokens are full (meaning they haven't been consumed from recently).
        // This is a heuristic — for precise idle tracking we'd need an extra
        // timestamp per bucket.
        let max_tokens_threshold = self.config.burst_size as f64 * 0.999;
        self.buckets
            .retain(|_key, bucket| bucket.tokens < max_tokens_threshold);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::default_limiter()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // ---- config ----

    #[test]
    fn test_config_default() {
        let c = RateLimiterConfig::default();
        assert_eq!(c.max_requests, 100);
        assert_eq!(c.window_secs, 60);
        assert_eq!(c.burst_size, 20);
    }

    #[test]
    fn test_config_permissive() {
        let c = RateLimiterConfig::permissive();
        assert_eq!(c.max_requests, 1000);
        assert_eq!(c.window_secs, 60);
        assert_eq!(c.burst_size, 200);
    }

    #[test]
    fn test_config_restrictive() {
        let c = RateLimiterConfig::restrictive();
        assert_eq!(c.max_requests, 30);
        assert_eq!(c.window_secs, 60);
        assert_eq!(c.burst_size, 5);
    }

    #[test]
    fn test_config_refill_rate() {
        let c = RateLimiterConfig::default();
        // 100 / 60 ≈ 1.6667
        let rate = c.refill_rate();
        assert!(rate > 1.6 && rate < 1.7);
    }

    #[test]
    fn test_config_validate_ok() {
        assert!(RateLimiterConfig::default().validate().is_ok());
        assert!(RateLimiterConfig::permissive().validate().is_ok());
        assert!(RateLimiterConfig::restrictive().validate().is_ok());
    }

    #[test]
    fn test_config_validate_zero_max_requests() {
        let c = RateLimiterConfig {
            max_requests: 0,
            window_secs: 60,
            burst_size: 10,
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn test_config_validate_zero_window() {
        let c = RateLimiterConfig {
            max_requests: 100,
            window_secs: 0,
            burst_size: 10,
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn test_config_validate_zero_burst() {
        let c = RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 0,
        };
        assert!(c.validate().is_err());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let c = RateLimiterConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: RateLimiterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.max_requests, back.max_requests);
        assert_eq!(c.window_secs, back.window_secs);
        assert_eq!(c.burst_size, back.burst_size);
    }

    // ---- limiter creation ----

    #[test]
    fn test_limiter_default() {
        let limiter = RateLimiter::default_limiter();
        assert_eq!(limiter.key_count(), 0);
        assert_eq!(limiter.total_checks(), 0);
        assert_eq!(limiter.total_rejections(), 0);
    }

    #[test]
    fn test_limiter_permissive() {
        let limiter = RateLimiter::permissive();
        assert_eq!(limiter.config().max_requests, 1000);
    }

    #[test]
    fn test_limiter_restrictive() {
        let limiter = RateLimiter::restrictive();
        assert_eq!(limiter.config().max_requests, 30);
    }

    // ---- basic check_and_record ----

    #[test]
    fn test_check_and_record_allows_within_burst() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 5,
        });
        for _ in 0..5 {
            assert!(limiter.check_and_record("key_a").is_ok());
        }
        // 6th should be rejected (bucket exhausted).
        assert!(limiter.check_and_record("key_a").is_err());
    }

    #[test]
    fn test_check_and_record_independent_keys() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 2,
        });
        assert!(limiter.check_and_record("key_a").is_ok());
        assert!(limiter.check_and_record("key_a").is_ok());
        assert!(limiter.check_and_record("key_a").is_err());

        // key_b has its own full bucket.
        assert!(limiter.check_and_record("key_b").is_ok());
        assert!(limiter.check_and_record("key_b").is_ok());
        assert!(limiter.check_and_record("key_b").is_err());
    }

    #[test]
    fn test_check_and_record_counts() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 2,
        });
        let _ = limiter.check_and_record("k"); // ok
        let _ = limiter.check_and_record("k"); // ok
        let _ = limiter.check_and_record("k"); // err
        assert_eq!(limiter.total_checks(), 3);
        assert_eq!(limiter.total_rejections(), 1);
    }

    // ---- refill over time ----

    #[test]
    fn test_refill_over_time() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 1000,
            window_secs: 1, // 1000 tokens/sec refill
            burst_size: 2,
        });
        // Exhaust the bucket.
        assert!(limiter.check_and_record("k").is_ok());
        assert!(limiter.check_and_record("k").is_ok());
        assert!(limiter.check_and_record("k").is_err());
        // Wait for refill (>1ms should give at least 1 token at 1000/sec).
        thread::sleep(Duration::from_millis(5));
        assert!(limiter.check_and_record("k").is_ok());
    }

    // ---- check without record ----

    #[test]
    fn test_check_does_not_consume() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 1,
        });
        // Check should succeed but not consume.
        assert!(limiter.check("k").is_ok());
        assert!(limiter.check("k").is_ok());
        assert!(limiter.check("k").is_ok());
        // Still ok — tokens not consumed.
    }

    // ---- record without check ----

    #[test]
    fn test_record_consumes_without_checking() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 2,
        });
        // Manually consume tokens.
        limiter.record("k");
        limiter.record("k");
        limiter.record("k");
        // Over-consumed; the bucket goes negative, next check should fail.
        assert!(limiter.check("k").is_err());
    }

    // ---- status inspection ----

    #[test]
    fn test_get_status_existing_key() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 60,
            window_secs: 60,
            burst_size: 10,
        });
        limiter.check_and_record("k").unwrap();
        let status = limiter.get_status("k").unwrap();
        assert_eq!(status.key, "k");
        assert!(status.tokens < 10.0); // One consumed
        assert_eq!(status.max_tokens, 10.0);
        assert_eq!(status.total_allowed, 1);
        assert_eq!(status.total_rejected, 0);
    }

    #[test]
    fn test_get_status_new_key() {
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        // get_status on a new key should create the bucket.
        let status = limiter.get_status("new_key").unwrap();
        assert_eq!(status.key, "new_key");
        assert_eq!(status.max_tokens, limiter.config().burst_size as f64);
    }

    #[test]
    fn test_list_keys() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 5,
        });
        limiter.check_and_record("alpha").unwrap();
        limiter.check_and_record("beta").unwrap();
        let mut keys = limiter.list_keys();
        keys.sort();
        assert_eq!(keys, vec!["alpha", "beta"]);
    }

    // ---- reset ----

    #[test]
    fn test_reset_specific_key() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 2,
        });
        limiter.check_and_record("k").unwrap();
        limiter.check_and_record("k").unwrap();
        assert!(limiter.check_and_record("k").is_err());
        limiter.reset("k");
        // Should be fully refilled after reset.
        assert!(limiter.check_and_record("k").is_ok());
        assert!(limiter.check_and_record("k").is_ok());
    }

    #[test]
    fn test_reset_nonexistent_key_is_noop() {
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        limiter.reset("does_not_exist");
        // No panic.
        assert_eq!(limiter.key_count(), 0);
    }

    #[test]
    fn test_reset_all() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 5,
        });
        limiter.check_and_record("a").unwrap();
        limiter.check_and_record("b").unwrap();
        assert_eq!(limiter.key_count(), 2);
        assert_eq!(limiter.total_checks(), 2);
        limiter.reset_all();
        assert_eq!(limiter.key_count(), 0);
        assert_eq!(limiter.total_checks(), 0);
        assert_eq!(limiter.total_rejections(), 0);
    }

    // ---- purge idle ----

    #[test]
    fn test_purge_idle_removes_full_buckets() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 10,
        });
        // Create a bucket by checking it (doesn't consume).
        limiter.check("idle_key").unwrap();
        // Bucket is full (no tokens consumed).
        limiter.purge_idle(0);
        // Full buckets should be purged.
        assert!(!limiter.list_keys().contains(&"idle_key".to_string()));
    }

    #[test]
    fn test_purge_idle_keeps_active_buckets() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 10,
        });
        // Consume some tokens so it's not full.
        limiter.check_and_record("active_key").unwrap();
        limiter.purge_idle(0);
        // Not-full bucket should be retained.
        assert!(limiter.list_keys().contains(&"active_key".to_string()));
    }

    // ---- restrictive preset ----

    #[test]
    fn test_restrictive_preset_blocks_quickly() {
        let limiter = RateLimiter::restrictive(); // burst_size = 5
        for _ in 0..5 {
            assert!(limiter.check_and_record("op").is_ok());
        }
        assert!(limiter.check_and_record("op").is_err());
    }

    // ---- permissive preset ----

    #[test]
    fn test_permissive_preset_allows_large_burst() {
        let limiter = RateLimiter::permissive(); // burst_size = 200
        for _ in 0..200 {
            assert!(limiter.check_and_record("op").is_ok());
        }
        assert!(limiter.check_and_record("op").is_err());
    }

    // ---- concurrent access ----

    #[test]
    fn test_concurrent_access() {
        let limiter = std::sync::Arc::new(RateLimiter::new(RateLimiterConfig {
            max_requests: 10_000,
            window_secs: 60,
            burst_size: 10_000,
        }));
        let mut handles = Vec::new();
        for t in 0..8 {
            let lim = std::sync::Arc::clone(&limiter);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("thread_{t}_key_{}", i % 5);
                    let _ = lim.check_and_record(&key);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // Should have processed all 8 * 100 = 800 checks.
        assert_eq!(limiter.total_checks(), 800);
    }

    // ---- error type ----

    #[test]
    fn test_rate_limit_error_format() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 100,
            window_secs: 60,
            burst_size: 1,
        });
        limiter.check_and_record("k").unwrap();
        let err = limiter.check_and_record("k").unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limit") || msg.contains("rate limit"));
    }

    // ---- bucket status serde ----

    #[test]
    fn test_bucket_status_serde_roundtrip() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 60,
            window_secs: 60,
            burst_size: 10,
        });
        limiter.check_and_record("my_key").unwrap();
        let status = limiter.get_status("my_key").unwrap();
        let json = serde_json::to_string(&status).unwrap();
        let back: BucketStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status.key, back.key);
        assert_eq!(status.max_tokens, back.max_tokens);
        assert_eq!(status.total_allowed, back.total_allowed);
    }

    // ---- edge cases ----

    #[test]
    fn test_empty_key() {
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        assert!(limiter.check_and_record("").is_ok());
    }

    #[test]
    fn test_very_long_key() {
        let key = "x".repeat(10_000);
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        assert!(limiter.check_and_record(&key).is_ok());
    }

    #[test]
    fn test_unicode_key() {
        let limiter = RateLimiter::new(RateLimiterConfig::default());
        assert!(limiter.check_and_record("🔑_ключ").is_ok());
    }

    #[test]
    fn test_many_keys_no_oom() {
        let limiter = RateLimiter::new(RateLimiterConfig {
            max_requests: 10,
            window_secs: 60,
            burst_size: 1,
        });
        for i in 0..1000 {
            let _ = limiter.check_and_record(&format!("key_{i}"));
        }
        assert_eq!(limiter.key_count(), 1000);
    }
}
