//! Caching layer for pb-mcp-server
//!
//! This module provides high-performance caching for:
//! - File contents (with invalidation on change)
//! - Code index queries
//! - Web request responses
//! - Git operations
//!
//! # Features
//!
//! - **LRU eviction** - Automatic eviction of least-recently-used entries
//! - **TTL support** - Time-to-live for stale entries
//! - **Memory limits** - Bounded memory usage
//! - **Invalidation** - Smart invalidation based on file changes
//! - **Thread-safe** - Concurrent access with DashMap
//! - **Metrics** - Hit/miss ratio tracking
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::cache::{FileCache, CacheConfig};
//!
//! let config = CacheConfig {
//!     max_entries: 1000,
//!     max_bytes: 50 * 1024 * 1024, // 50 MB
//!     default_ttl_secs: 300, // 5 minutes
//! };
//! let cache = FileCache::new(config);
//!
//! // Get or load file
//! let content = cache.get_or_load("src/main.rs", || {
//!     std::fs::read_to_string("src/main.rs")
//! })?;
//! ```

use dashmap::DashMap;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub mod file;

pub use file::FileCache;

// ============================================================================
// Cache Configuration
// ============================================================================

/// Configuration for cache behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,

    /// Maximum total bytes in the cache
    pub max_bytes: usize,

    /// Default time-to-live in seconds
    pub default_ttl_secs: u64,

    /// Whether to enable metrics collection
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_entries: 1000,
            max_bytes: 50 * 1024 * 1024, // 50 MB
            default_ttl_secs: 300,       // 5 minutes
            enable_metrics: true,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with custom limits
    pub fn new(max_entries: usize, max_bytes: usize, ttl_secs: u64) -> Self {
        CacheConfig {
            max_entries,
            max_bytes,
            default_ttl_secs: ttl_secs,
            enable_metrics: true,
        }
    }

    /// Create a small cache for limited environments
    pub fn small() -> Self {
        CacheConfig {
            max_entries: 100,
            max_bytes: 10 * 1024 * 1024, // 10 MB
            default_ttl_secs: 60,        // 1 minute
            enable_metrics: true,
        }
    }

    /// Create a large cache for power users
    pub fn large() -> Self {
        CacheConfig {
            max_entries: 5000,
            max_bytes: 200 * 1024 * 1024, // 200 MB
            default_ttl_secs: 600,        // 10 minutes
            enable_metrics: true,
        }
    }
}

// ============================================================================
// Cache Entry
// ============================================================================

/// A cached entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    /// The cached value
    pub value: V,

    /// Size in bytes
    pub size: usize,

    /// When the entry was created
    pub created_at: Instant,

    /// When the entry was last accessed
    pub last_accessed: Instant,

    /// Time-to-live duration
    pub ttl: Duration,

    /// Number of times this entry has been accessed
    pub access_count: u64,
}

impl<V: Clone> CacheEntry<V> {
    /// Create a new cache entry
    pub fn new(value: V, size: usize, ttl: Duration) -> Self {
        let now = Instant::now();
        CacheEntry {
            value,
            size,
            created_at: now,
            last_accessed: now,
            ttl,
            access_count: 1,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Mark the entry as accessed
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// Get age in seconds
    pub fn age_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }

    /// Get idle time in seconds (time since last access)
    pub fn idle_secs(&self) -> u64 {
        self.last_accessed.elapsed().as_secs()
    }
}

// ============================================================================
// Cache Metrics
// ============================================================================

/// Metrics for cache performance
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// Total cache hits
    pub hits: AtomicU64,

    /// Total cache misses
    pub misses: AtomicU64,

    /// Total evictions
    pub evictions: AtomicU64,

    /// Current entry count
    pub entry_count: AtomicU64,

    /// Current byte usage
    pub byte_usage: AtomicU64,

    /// Total bytes served from cache
    pub bytes_served: AtomicU64,
}

impl CacheMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        CacheMetrics::default()
    }

    /// Record a cache hit
    pub fn record_hit(&self, bytes: usize) {
        self.hits.fetch_add(1, Ordering::Relaxed);
        self.bytes_served.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn set_entry_count(&self, count: usize) {
        self.entry_count.store(count as u64, Ordering::Relaxed);
    }

    /// Update byte usage
    pub fn set_byte_usage(&self, bytes: usize) {
        self.byte_usage.store(bytes as u64, Ordering::Relaxed);
    }

    /// Get hit ratio (0.0 to 1.0)
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get total requests
    pub fn total_requests(&self) -> u64 {
        self.hits.load(Ordering::Relaxed) + self.misses.load(Ordering::Relaxed)
    }

    /// Get formatted summary
    pub fn summary(&self) -> CacheMetricsSummary {
        CacheMetricsSummary {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            entry_count: self.entry_count.load(Ordering::Relaxed),
            byte_usage: self.byte_usage.load(Ordering::Relaxed),
            bytes_served: self.bytes_served.load(Ordering::Relaxed),
            hit_ratio: self.hit_ratio(),
        }
    }
}

/// Snapshot of cache metrics
#[derive(Debug, Clone, Serialize)]
pub struct CacheMetricsSummary {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache evictions
    pub evictions: u64,
    /// Current number of entries in cache
    pub entry_count: u64,
    /// Current byte usage of cache
    pub byte_usage: u64,
    /// Total bytes served from cache
    pub bytes_served: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
}

impl std::fmt::Display for CacheMetricsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Metrics:\n Hits: {}\n Misses: {}\n Hit Ratio: {:.2}%\n Entries: {}\n Bytes: {:.2} MB\n Served: {:.2} MB\n Evictions: {}",
            self.hits,
            self.misses,
            self.hit_ratio * 100.0,
            self.entry_count,
            self.byte_usage as f64 / 1_048_576.0,
            self.bytes_served as f64 / 1_048_576.0,
            self.evictions
        )
    }
}

// ============================================================================
// LRU Cache Implementation
// ============================================================================

/// Thread-safe LRU cache with TTL support
pub struct LruCache<K, V> {
    /// The underlying storage
    data: DashMap<K, CacheEntry<V>>,

    /// Cache configuration
    config: CacheConfig,

    /// Metrics tracking
    metrics: CacheMetrics,

    /// Current byte usage
    current_bytes: AtomicU64,
}

impl<K, V> LruCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new LRU cache
    pub fn new(config: CacheConfig) -> Self {
        LruCache {
            data: DashMap::new(),
            config,
            metrics: CacheMetrics::new(),
            current_bytes: AtomicU64::new(0),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(mut entry) = self.data.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.remove(key);
                self.metrics.record_miss();
                return None;
            }

            entry.touch();
            let value = entry.value.clone();
            self.metrics.record_hit(entry.size);
            Some(value)
        } else {
            self.metrics.record_miss();
            None
        }
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V, size: usize) {
        self.insert_with_ttl(
            key,
            value,
            size,
            Duration::from_secs(self.config.default_ttl_secs),
        );
    }

    /// Insert a value with custom TTL
    pub fn insert_with_ttl(&self, key: K, value: V, size: usize, ttl: Duration) {
        // Check if we need to evict
        while self.data.len() >= self.config.max_entries
            || self.current_bytes.load(Ordering::Relaxed) + size as u64
                > self.config.max_bytes as u64
        {
            if !self.evict_one() {
                break;
            }
        }

        let entry = CacheEntry::new(value, size, ttl);
        let old_size = if let Some(old) = self.data.get(&key) {
            old.size
        } else {
            0
        };

        self.current_bytes
            .fetch_add((size - old_size) as u64, Ordering::Relaxed);
        self.data.insert(key, entry);

        self.metrics.set_entry_count(self.data.len());
        self.metrics
            .set_byte_usage(self.current_bytes.load(Ordering::Relaxed) as usize);
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        if let Some((_, entry)) = self.data.remove(key) {
            self.current_bytes
                .fetch_sub(entry.size as u64, Ordering::Relaxed);
            self.metrics.set_entry_count(self.data.len());
            self.metrics
                .set_byte_usage(self.current_bytes.load(Ordering::Relaxed) as usize);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.get(key).map(|e| !e.is_expired()).unwrap_or(false)
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.data.clear();
        self.current_bytes.store(0, Ordering::Relaxed);
        self.metrics.set_entry_count(0);
        self.metrics.set_byte_usage(0);
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get current byte usage
    pub fn byte_usage(&self) -> usize {
        self.current_bytes.load(Ordering::Relaxed) as usize
    }

    /// Get metrics summary
    pub fn metrics(&self) -> CacheMetricsSummary {
        self.metrics.summary()
    }

    /// Evict least recently used entry
    fn evict_one(&self) -> bool {
        let mut oldest_key: Option<K> = None;
        let mut oldest_time = Instant::now();

        for entry in self.data.iter() {
            let last_accessed = entry.last_accessed;
            if last_accessed < oldest_time {
                oldest_time = last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            self.remove(&key);
            self.metrics.record_eviction();
            true
        } else {
            false
        }
    }

    /// Evict all expired entries
    pub fn evict_expired(&self) -> usize {
        let mut expired_keys = Vec::new();

        for entry in self.data.iter() {
            if entry.is_expired() {
                expired_keys.push(entry.key().clone());
            }
        }

        let count = expired_keys.len();
        for key in expired_keys {
            self.remove(&key);
            self.metrics.record_eviction();
        }

        count
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
    fn test_cache_basic() {
        let config = CacheConfig::default();
        let cache = LruCache::<String, String>::new(config);

        cache.insert("key1".to_string(), "value1".to_string(), 10);
        cache.insert("key2".to_string(), "value2".to_string(), 10);

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig::new(2, 1000, 60);
        let cache = LruCache::<String, String>::new(config);

        cache.insert("key1".to_string(), "value1".to_string(), 10);
        cache.insert("key2".to_string(), "value2".to_string(), 10);
        cache.insert("key3".to_string(), "value3".to_string(), 10);

        // Should have evicted at least one entry
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_ttl() {
        let config = CacheConfig::new(10, 1000, 1); // 1 second TTL
        let cache = LruCache::<String, String>::new(config);

        cache.insert("key1".to_string(), "value1".to_string(), 10);

        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Wait for TTL
        thread::sleep(Duration::from_millis(1100));

        // Should be expired now
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_metrics() {
        let config = CacheConfig::default();
        let cache = LruCache::<String, String>::new(config);

        cache.insert("key1".to_string(), "value1".to_string(), 10);

        // Hit
        let _ = cache.get(&"key1".to_string());

        // Miss
        let _ = cache.get(&"key2".to_string());

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert!((metrics.hit_ratio - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cache_concurrent() {
        let config = CacheConfig::default();
        let cache = std::sync::Arc::new(LruCache::<String, String>::new(config));

        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key-{}-{}", i, j);
                    cache_clone.insert(key.clone(), format!("value-{}", j), 10);
                    let _ = cache_clone.get(&key);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let metrics = cache.metrics();
        assert!(metrics.hits + metrics.misses > 0);
    }

    #[test]
    fn test_cache_config() {
        let small = CacheConfig::small();
        assert_eq!(small.max_entries, 100);

        let large = CacheConfig::large();
        assert_eq!(large.max_entries, 5000);
    }

    #[test]
    fn test_evict_expired() {
        let config = CacheConfig::new(10, 10000, 1);
        let cache = LruCache::<String, String>::new(config);

        cache.insert_with_ttl(
            "key1".to_string(),
            "value1".to_string(),
            10,
            Duration::from_millis(100),
        );
        cache.insert_with_ttl(
            "key2".to_string(),
            "value2".to_string(),
            10,
            Duration::from_secs(60),
        );

        thread::sleep(Duration::from_millis(150));

        let evicted = cache.evict_expired();
        assert_eq!(evicted, 1);
        assert_eq!(cache.len(), 1);
    }
}
