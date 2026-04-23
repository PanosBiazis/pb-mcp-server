//! File caching with smart invalidation
//!
//! This module provides caching specifically optimized for file operations:
//! - Memory-mapped reading for large files
//! - Automatic invalidation on file modification
//! - Path-based cache keys
//! - Integration with file system events

use crate::cache::{CacheConfig, CacheMetricsSummary, LruCache};
use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ============================================================================
// File Cache Entry
// ============================================================================

/// Metadata for cached file content
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File modification time when cached
    pub modified_time: SystemTime,

    /// File size in bytes
    pub size: u64,

    /// Hash of file content (for quick validation)
    pub content_hash: Option<u64>,
}

impl FileMetadata {
    /// Create metadata from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path).map_err(|e| Error::file_system(path, e))?;

        Ok(FileMetadata {
            modified_time: metadata
                .modified()
                .map_err(|e| Error::file_system(path, e))?,
            size: metadata.len(),
            content_hash: None,
        })
    }

    /// Check if file has been modified since caching
    pub fn is_stale(&self, path: &Path) -> Result<bool> {
        let current = FileMetadata::from_path(path)?;
        Ok(current.modified_time != self.modified_time || current.size != self.size)
    }
}

/// Cached file content with metadata
#[derive(Debug, Clone)]
pub struct CachedFile {
    /// File content
    pub content: String,

    /// File metadata
    pub metadata: FileMetadata,

    /// Original file path
    pub path: PathBuf,

    /// Whether this was memory-mapped
    pub was_mapped: bool,
}

// ============================================================================
// File Cache Configuration
// ============================================================================

/// Configuration specific to file caching
#[derive(Debug, Clone)]
pub struct FileCacheConfig {
    /// Base cache configuration
    pub base: CacheConfig,

    /// Minimum file size to use memory mapping (bytes)
    pub mmap_threshold: usize,

    /// Maximum file size to cache (bytes)
    pub max_file_size: u64,

    /// Whether to validate file modification times
    pub validate_mtime: bool,

    /// Whether to compute content hashes
    pub compute_hashes: bool,
}

impl Default for FileCacheConfig {
    fn default() -> Self {
        FileCacheConfig {
            base: CacheConfig::default(),
            mmap_threshold: 64 * 1024,       // 64 KB
            max_file_size: 10 * 1024 * 1024, // 10 MB
            validate_mtime: true,
            compute_hashes: false,
        }
    }
}

impl FileCacheConfig {
    /// Create config for small files
    pub fn small_files() -> Self {
        FileCacheConfig {
            base: CacheConfig::small(),
            mmap_threshold: 16 * 1024,  // 16 KB
            max_file_size: 1024 * 1024, // 1 MB
            validate_mtime: true,
            compute_hashes: false,
        }
    }

    /// Create config for large files
    pub fn large_files() -> Self {
        FileCacheConfig {
            base: CacheConfig::large(),
            mmap_threshold: 256 * 1024,      // 256 KB
            max_file_size: 50 * 1024 * 1024, // 50 MB
            validate_mtime: true,
            compute_hashes: true,
        }
    }
}

// ============================================================================
// File Cache Implementation
// ============================================================================

/// High-performance file cache with invalidation support
pub struct FileCache {
    /// The underlying LRU cache
    cache: LruCache<PathBuf, CachedFile>,

    /// Configuration
    config: FileCacheConfig,
}

impl FileCache {
    /// Create a new file cache
    pub fn new(config: FileCacheConfig) -> Self {
        FileCache {
            cache: LruCache::new(config.base.clone()),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(FileCacheConfig::default())
    }

    /// Read a file, using cache if available
    pub fn read(&self, path: &Path) -> Result<String> {
        // Check if file is too large
        let metadata = fs::metadata(path).map_err(|e| Error::file_system(path, e))?;

        if metadata.len() > self.config.max_file_size {
            return Err(Error::file_too_large(
                path,
                metadata.len(),
                self.config.max_file_size,
            ));
        }

        // Try to get from cache
        if let Some(cached) = self.cache.get(&path.to_path_buf()) {
            // Validate if configured
            if self.config.validate_mtime {
                if !cached.metadata.is_stale(path)? {
                    return Ok(cached.content);
                }
            } else {
                return Ok(cached.content);
            }
        }

        // Load file
        let content = self.load_file(path)?;

        // Cache it
        let cached = CachedFile {
            content: content.clone(),
            metadata: FileMetadata::from_path(path)?,
            path: path.to_path_buf(),
            was_mapped: metadata.len() as usize >= self.config.mmap_threshold,
        };

        self.cache.insert(path.to_path_buf(), cached, content.len());

        Ok(content)
    }

    /// Read a file with optional line range
    pub fn read_lines(
        &self,
        path: &Path,
        start_line: Option<usize>,
        end_line: Option<usize>,
    ) -> Result<String> {
        let content = self.read(path)?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let (selected_lines, line_info) = if start_line.is_some() || end_line.is_some() {
            let start = start_line.unwrap_or(1).max(1).min(total_lines);
            let end = end_line.unwrap_or(total_lines).max(start).min(total_lines);

            if start > total_lines {
                return Ok(format!(
                    "FILE: {}\n\nNo lines in range (file has {} lines; start_line={})",
                    path.display(),
                    total_lines,
                    start
                ));
            }

            let range = &lines[start - 1..end];
            (
                range.to_vec(),
                format!("lines {}-{} of {}", start, end, total_lines),
            )
        } else {
            (lines.clone(), format!("{} lines", total_lines))
        };

        // Format with line numbers
        let mut output = String::new();
        output.push_str(&format!("FILE: {} ({})\n\n", path.display(), line_info));

        let start_num = start_line.unwrap_or(1).max(1);
        for (i, line) in selected_lines.iter().enumerate() {
            output.push_str(&format!("{:6}| {}\n", start_num + i, line));
        }

        Ok(output)
    }

    /// Load a file, optionally using memory mapping
    fn load_file(&self, path: &Path) -> Result<String> {
        let metadata = fs::metadata(path).map_err(|e| Error::file_system(path, e))?;

        // For large files, consider memory mapping
        if metadata.len() as usize >= self.config.mmap_threshold {
            #[cfg(feature = "perf-mmap")]
            {
                return self.load_file_mmap(path);
            }
        }

        // Standard file reading
        fs::read_to_string(path).map_err(|e| Error::file_system(path, e))
    }

    /// Load file using memory mapping (feature-gated)
    #[cfg(feature = "perf-mmap")]
    pub fn load_file_mmap(&self, path: &Path) -> Result<String> {
        use std::io::Read;

        let file = fs::File::open(path).map_err(|e| Error::file_system(path, e))?;

        // Safety: We trust the file system and don't modify the file
        let mmap = unsafe { memmap2::Mmap::map(&file).map_err(|e| Error::file_system(path, e))? };

        // Convert to string (validates UTF-8)
        String::from_utf8(mmap.to_vec())
            .map_err(|e| Error::invalid_arguments("path", format!("Invalid UTF-8: {}", e)))
    }

    /// Invalidate a specific file
    pub fn invalidate(&self, path: &Path) -> bool {
        self.cache.remove(&path.to_path_buf()).is_some()
    }

    /// Invalidate all files matching a pattern
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        let mut count = 0;
        let pattern_lower = pattern.to_lowercase();

        // Collect paths to invalidate
        let paths: Vec<PathBuf> = self
            .cache
            .data
            .iter()
            .filter(|entry| {
                entry
                    .key()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&pattern_lower)
            })
            .map(|entry| entry.key().clone())
            .collect();

        for path in paths {
            if self.cache.remove(&path).is_some() {
                count += 1;
            }
        }

        count
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get current byte usage
    pub fn byte_usage(&self) -> usize {
        self.cache.byte_usage()
    }

    /// Get cache metrics
    pub fn metrics(&self) -> CacheMetricsSummary {
        self.cache.metrics()
    }

    /// Check if a file is cached
    pub fn is_cached(&self, path: &Path) -> bool {
        self.cache.contains_key(&path.to_path_buf())
    }

    /// Get cached file metadata without loading
    pub fn get_metadata(&self, path: &Path) -> Option<FileMetadata> {
        self.cache.get(&path.to_path_buf()).map(|f| f.metadata)
    }

    /// Prefetch files into cache
    pub fn prefetch(&self, paths: &[PathBuf]) -> Vec<Result<()>> {
        paths
            .iter()
            .map(|path| {
                self.read(path)?;
                Ok(())
            })
            .collect()
    }
}

// ============================================================================
// Async File Cache
// ============================================================================

// Async wrapper for file cache
// Note: async feature is not defined in Cargo.toml
// AsyncFileCache temporarily commented out - entire async block commented
/*
#[cfg(feature = "async")]
pub struct AsyncFileCache {
    inner: Arc<FileCache>,
    runtime: tokio::runtime::Handle,
}

#[cfg(feature = "async")]
impl AsyncFileCache {
    pub fn new(cache: FileCache, runtime: tokio::runtime::Handle) -> Self {
        AsyncFileCache {
            inner: Arc::new(cache),
            runtime,
        }
    }

    pub async fn read(&self, path: PathBuf) -> Result<String> {
        let cache = self.inner.clone();
        self.runtime
            .spawn_blocking(move || cache.read(&path))
            .await
            .map_err(|e| Error::Generic(format!("Task join error: {}", e)))?
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        write!(file, "{}", content).expect("Failed to write to temp file");
        file
    }

    #[test]
    fn test_file_cache_basic() {
        let content = "Hello, world!\nLine 2\nLine 3";
        let file = create_test_file(content);

        let cache = FileCache::with_defaults();

        // First read - cache miss
        let result1 = cache.read(file.path()).unwrap();
        assert!(result1.contains("Hello, world!"));

        // Second read - cache hit
        let result2 = cache.read(file.path()).unwrap();
        assert!(result2.contains("Hello, world!"));

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
    }

    #[test]
    fn test_file_cache_read_lines() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let file = create_test_file(content);

        let cache = FileCache::with_defaults();

        let result = cache.read_lines(file.path(), Some(2), Some(4)).unwrap();
        assert!(result.contains("Line 2"));
        assert!(result.contains("Line 3"));
        assert!(result.contains("Line 4"));
        assert!(!result.contains("Line 1"));
        assert!(!result.contains("Line 5"));
    }

    #[test]
    fn test_file_cache_invalidation() {
        let content = "Original content";
        let file = create_test_file(content);

        let cache = FileCache::with_defaults();
        let _ = cache.read(file.path()).unwrap();

        assert!(cache.is_cached(file.path()));

        // Invalidate
        let invalidated = cache.invalidate(file.path());
        assert!(invalidated);

        assert!(!cache.is_cached(file.path()));
    }

    #[test]
    fn test_file_cache_stale_detection() {
        let content = "Original content";
        let mut file = create_test_file(content);

        let config = FileCacheConfig {
            validate_mtime: true,
            ..Default::default()
        };
        let cache = FileCache::new(config);

        // First read
        let _ = cache.read(file.path()).unwrap();

        // Modify file
        std::thread::sleep(Duration::from_millis(100));
        write!(file, "Modified content").unwrap();

        // Should detect stale and reload
        let result = cache.read(file.path()).unwrap();
        assert!(result.contains("Modified content"));
    }

    #[test]
    fn test_file_cache_too_large() {
        let config = FileCacheConfig {
            max_file_size: 100,
            ..Default::default()
        };
        let cache = FileCache::new(config);

        let content = "x".repeat(200);
        let file = create_test_file(&content);

        let result = cache.read(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_file_cache_prefetch() {
        let file1 = create_test_file("Content 1");
        let file2 = create_test_file("Content 2");
        let file3 = create_test_file("Content 3");

        let cache = FileCache::with_defaults();

        let results = cache.prefetch(&[
            file1.path().to_path_buf(),
            file2.path().to_path_buf(),
            file3.path().to_path_buf(),
        ]);

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));

        assert!(cache.is_cached(file1.path()));
        assert!(cache.is_cached(file2.path()));
        assert!(cache.is_cached(file3.path()));
    }

    #[test]
    fn test_file_metadata() {
        let content = "Test content";
        let file = create_test_file(content);

        let metadata = FileMetadata::from_path(file.path()).unwrap();

        assert_eq!(metadata.size, content.len() as u64);
        assert!(!metadata.is_stale(file.path()).unwrap());
    }

    #[test]
    fn test_file_cache_config() {
        let small_config = FileCacheConfig::small_files();
        assert_eq!(small_config.max_file_size, 1024 * 1024);

        let large_config = FileCacheConfig::large_files();
        assert_eq!(large_config.max_file_size, 50 * 1024 * 1024);
    }
}
*/
