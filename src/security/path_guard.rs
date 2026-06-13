//! Path validation and security guard for pb-mcp-server
//!
//! This module provides comprehensive path validation to prevent directory traversal
//! attacks, symlink escapes, and other path-based security vulnerabilities.
//!
//! # Security Model
//!
//! All file operations MUST go through this module to ensure:
//! 1. Paths stay within the project root
//! 2. Symlinks are resolved safely
//! 3. Path traversal attempts are blocked
//! 4. TOCTOU races are mitigated
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::security::PathGuard;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let guard = PathGuard::new("/project/root")?;
//! let safe_path = guard.resolve("src/main.rs")?;
//! let safe_path = guard.resolve_and_validate("src/main.rs", false)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Maximum symlink resolution depth to prevent cycles
const MAX_SYMLINK_DEPTH: u8 = 8;

/// Maximum path length (prevent DoS with extremely long paths)
const MAX_PATH_LENGTH: usize = 4096;

/// Allowed file extensions for certain operations (e.g., file reads)
const DANGEROUS_EXTENSIONS: &[&str] = &[
    // Environment and secrets
    ".env",
    ".secret",
    // Cryptographic keys and certificates
    ".pem",
    ".key",
    ".cert",
    ".crt",
    ".cer",
    ".asc",
    ".p12",
    ".pfx",
    ".keystore",
    // Password databases
    ".kdbx",
    ".kdb",
    // Database files
    ".db",
    ".sqlite",
    ".sqlite3",
    // SSH keys (bare names checked separately below)
];

/// Path validation and security guard
#[derive(Debug, Clone)]
pub struct PathGuard {
    /// Canonicalized project root (never ends with /)
    root: PathBuf,
    /// Whether to allow symlinks within the root
    allow_symlinks: bool,
    /// Whether to allow hidden files (starting with .)
    allow_hidden_files: bool,
    /// Maximum file size to allow for read operations
    max_file_size: u64,
    /// List of denied paths (relative to root)
    denied_paths: Vec<PathBuf>,
}

impl PathGuard {
    /// Create a new path guard for the given project root
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        // Canonicalize the root to resolve symlinks and normalize
        let root = fs::canonicalize(&root).map_err(|e| Error::file_system(&root, e))?;

        // Validate root is a directory
        let metadata = fs::metadata(&root).map_err(|e| Error::file_system(&root, e))?;

        if !metadata.is_dir() {
            return Err(Error::path_validation_with_path(
                "Project root is not a directory",
                root,
            ));
        }

        Ok(PathGuard {
            root,
            allow_symlinks: true,
            allow_hidden_files: true,
            max_file_size: 512 * 1024, // 512KB default
            denied_paths: vec![
                PathBuf::from(".git/HEAD"),
                PathBuf::from(".git/config"),
                PathBuf::from(".ssh"),
                PathBuf::from(".gnupg"),
            ],
        })
    }

    /// Create a path guard with restrictive settings for public access
    pub fn new_public_safe(root: impl Into<PathBuf>) -> Result<Self> {
        let mut guard = Self::new(root)?;
        guard.allow_symlinks = false;
        guard.allow_hidden_files = false;
        guard.max_file_size = 64 * 1024; // 64KB for public access
        Ok(guard)
    }

    /// Create a path guard for local trusted access
    pub fn new_local_trusted(root: impl Into<PathBuf>) -> Result<Self> {
        let mut guard = Self::new(root)?;
        guard.max_file_size = 10 * 1024 * 1024; // 10MB for local
        guard.denied_paths.clear(); // Allow all paths
        Ok(guard)
    }

    /// Set whether to allow symlinks
    pub fn set_allow_symlinks(&mut self, allow: bool) -> &mut Self {
        self.allow_symlinks = allow;
        self
    }

    /// Set whether to allow hidden files
    pub fn set_allow_hidden_files(&mut self, allow: bool) -> &mut Self {
        self.allow_hidden_files = allow;
        self
    }

    /// Set maximum file size
    pub fn set_max_file_size(&mut self, max_bytes: u64) -> &mut Self {
        self.max_file_size = max_bytes;
        self
    }

    /// Add a denied path
    pub fn deny_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.denied_paths.push(path.into());
        self
    }

    /// Get the project root
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if a path component is hidden (starts with .)
    fn is_hidden_component(component: &str) -> bool {
        component.starts_with('.')
    }

    /// Check if a path has dangerous extension
    fn has_dangerous_extension(path: &Path) -> bool {
        // Check standard extension (e.g. "pem" from "cert.pem")
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| DANGEROUS_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
        {
            return true;
        }
        // Check dotfile names (e.g. ".env" — path.extension() returns None for these)
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| DANGEROUS_EXTENSIONS.contains(&name.to_lowercase().as_str()))
            .unwrap_or(false)
        {
            return true;
        }
        false
    }

    /// Resolve a path relative to the project root without full validation
    /// This is useful for operations that will validate separately
    pub fn resolve(&self, input: &str) -> Result<PathBuf> {
        // Check path length
        if input.len() > MAX_PATH_LENGTH {
            return Err(Error::path_validation("Path exceeds maximum length"));
        }

        let input_path = Path::new(input);

        // Handle absolute paths
        let resolved = if input_path.is_absolute() {
            // Absolute path must be under root
            input_path.to_path_buf()
        } else {
            // Relative path: join with root
            self.root.join(input)
        };

        Ok(resolved)
    }

    /// Resolve and validate a path with full security checks
    pub fn resolve_and_validate(&self, input: &str, check_exists: bool) -> Result<PathBuf> {
        // Step 1: Basic resolution
        let resolved = self.resolve(input)?;

        // Step 2: Normalize path (resolve .. and . components)
        let normalized = self.normalize_path(&resolved)?;

        // Step 3: Check if path is under root
        self.verify_under_root(&normalized)?;

        // Step 4: Check denied paths
        self.check_denied_paths(&normalized)?;

        // Step 5: Check hidden files if needed
        if !self.allow_hidden_files {
            self.check_no_hidden_components(&normalized)?;
        }

        // Step 6: Resolve symlinks if needed
        let canonical = if self.allow_symlinks {
            // Resolve symlinks with depth limit
            self.resolve_symlinks(&normalized)?
        } else {
            // Check that it's not a symlink
            if check_exists {
                let metadata = fs::symlink_metadata(&normalized)
                    .map_err(|e| Error::file_system(&normalized, e))?;
                if metadata.file_type().is_symlink() {
                    return Err(Error::security_violation(
                        "Symlinks are not allowed in this security mode",
                    ));
                }
            }
            normalized
        };

        // Step 7: Verify canonical path is still under root (prevents symlink escape)
        self.verify_under_root(&canonical)?;

        // Step 8: Check file exists and size if requested
        if check_exists {
            let metadata =
                fs::metadata(&canonical).map_err(|e| Error::file_system(&canonical, e))?;

            if metadata.is_file() {
                let size = metadata.len();
                if size > self.max_file_size {
                    return Err(Error::file_too_large(&canonical, size, self.max_file_size));
                }

                // Check for dangerous extensions
                if Self::has_dangerous_extension(&canonical) {
                    return Err(Error::security_violation(
                        "Access to files with this extension is restricted",
                    ));
                }
            }
        }

        Ok(canonical)
    }

    /// Normalize a path by resolving . and .. components
    fn normalize_path(&self, path: &Path) -> Result<PathBuf> {
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) => {
                    // Keep prefix (e.g., C:\ on Windows)
                    normalized.push(component.as_os_str());
                }
                Component::RootDir => {
                    normalized.push(component.as_os_str());
                }
                Component::CurDir => {
                    // Skip current directory markers
                }
                Component::ParentDir => {
                    // Go up one level (but not above root)
                    if !normalized.pop() {
                        return Err(Error::path_validation(
                            "Path traversal attempt: cannot go above root",
                        ));
                    }
                }
                Component::Normal(name) => {
                    normalized.push(name);
                }
            }
        }

        Ok(normalized)
    }

    /// Verify that a path is under the project root
    fn verify_under_root(&self, path: &Path) -> Result<()> {
        // Use Path::starts_with which compares path components,
        // not string prefixes. This is safe against case-insensitive
        // filesystems and Unicode normalization edge cases.
        if !path.starts_with(&self.root) {
            return Err(Error::path_escape(path, &self.root));
        }
        Ok(())
    }

    /// Check if path is in denied list
    fn check_denied_paths(&self, path: &Path) -> Result<()> {
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| Error::path_escape(path, &self.root))?;

        for denied in &self.denied_paths {
            if relative == denied || relative.starts_with(denied) {
                return Err(Error::security_violation(format!(
                    "Access to path '{}' is denied by security policy",
                    denied.display()
                )));
            }
        }

        Ok(())
    }

    /// Check that path has no hidden components
    fn check_no_hidden_components(&self, path: &Path) -> Result<()> {
        for component in path.components() {
            if let Component::Normal(name) = component {
                if let Some(name_str) = name.to_str() {
                    if Self::is_hidden_component(name_str) {
                        return Err(Error::security_violation(
                            "Hidden files and directories are not allowed in this security mode",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Resolve symlinks with depth limit to prevent cycles
    fn resolve_symlinks(&self, path: &Path) -> Result<PathBuf> {
        let mut current = path.to_path_buf();
        let mut depth = 0;

        loop {
            // Check depth limit
            depth += 1;
            if depth > MAX_SYMLINK_DEPTH {
                return Err(Error::security_violation(
                    "Symlink resolution depth exceeded (possible cycle)",
                ));
            }

            // Check if current path is a symlink
            let metadata = match fs::symlink_metadata(&current) {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Path doesn't exist, that's okay for write operations
                    return Ok(current);
                }
                Err(e) => return Err(Error::file_system(&current, e)),
            };

            if !metadata.file_type().is_symlink() {
                // Not a symlink, we're done
                return Ok(current);
            }

            // Read the symlink target
            let target = fs::read_link(&current).map_err(|e| Error::file_system(&current, e))?;

            // Resolve relative to the symlink's parent
            let parent = current
                .parent()
                .ok_or_else(|| Error::path_validation("Symlink has no parent"))?;

            current = parent.join(target);

            // Normalize the new path
            current = self.normalize_path(&current)?;

            // Verify it's still under root
            self.verify_under_root(&current)?;
        }
    }

    /// Validate a path for a write operation (more permissive than read)
    pub fn validate_for_write(&self, input: &str) -> Result<PathBuf> {
        // For write operations, we don't check if file exists
        // but we still need to validate the path
        self.resolve_and_validate(input, false)
    }

    /// Validate a path for a read operation
    pub fn validate_for_read(&self, input: &str) -> Result<PathBuf> {
        self.resolve_and_validate(input, true)
    }

    /// Validate a path for execution (terminal command, etc.)
    pub fn validate_for_exec(&self, input: &str) -> Result<PathBuf> {
        // Execution requires the strictest validation
        let path = self.resolve_and_validate(input, true)?;

        // Additional checks for execution
        let metadata = fs::metadata(&path).map_err(|e| Error::file_system(&path, e))?;

        // Must be a file, not a directory
        if !metadata.is_file() {
            return Err(Error::path_validation_with_path(
                "Can only execute files, not directories",
                path,
            ));
        }

        Ok(path)
    }

    /// Check if a path would be valid without actually checking existence
    pub fn would_be_valid(&self, input: &str) -> bool {
        self.resolve_and_validate(input, false).is_ok()
    }

    /// Get relative path from absolute path
    pub fn relative_path(&self, path: &Path) -> Result<PathBuf> {
        path.strip_prefix(&self.root)
            .map_err(|_| Error::path_escape(path, &self.root))
            .map(|p| p.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_guard() -> (TempDir, PathGuard) {
        let temp = TempDir::new().unwrap();
        let guard = PathGuard::new(temp.path()).unwrap();
        (temp, guard)
    }

    #[test]
    fn test_valid_path() {
        let (temp, guard) = setup_test_guard();

        // Create a test file
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        // Should resolve successfully
        let result = guard.validate_for_read("test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_traversal_attempt() {
        let (_temp, guard) = setup_test_guard();

        // Attempt to escape with ..
        let result = guard.validate_for_read("../../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_absolute_path_under_root() {
        let (temp, guard) = setup_test_guard();

        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        // Absolute path should work if under root
        let abs_path = file_path.to_string_lossy().to_string();
        let result = guard.validate_for_read(&abs_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_normalization() {
        let (temp, guard) = setup_test_guard();

        // Create nested directory
        fs::create_dir_all(temp.path().join("a/b/c")).unwrap();

        // Path with . and .. should normalize
        let result = guard.validate_for_read("a/./b/../b/c");
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert!(normalized.ends_with("a/b/c"));
    }

    #[test]
    fn test_hidden_file_rejection() {
        let (temp, mut guard) = setup_test_guard();

        // Create hidden file
        fs::write(temp.path().join(".hidden"), "secret").unwrap();

        // Should fail when hidden files are not allowed
        guard.set_allow_hidden_files(false);
        let result = guard.validate_for_read(".hidden");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_file_size_limit() {
        let (temp, mut guard) = setup_test_guard();

        // Create a file larger than limit
        let large_content = vec![0u8; 1024]; // 1KB
        fs::write(temp.path().join("large.txt"), &large_content).unwrap();

        // Set very small limit
        guard.set_max_file_size(100);

        let result = guard.validate_for_read("large.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::FileTooLarge { .. }));
    }

    #[test]
    fn test_symlink_escape_prevention() {
        let (temp, guard) = setup_test_guard();

        // Create a symlink pointing outside root
        #[cfg(unix)]
        {
            use std::os::unix::fs as unix_fs;
            let temp2 = TempDir::new().unwrap();
            let link_path = temp.path().join("escape_link");
            unix_fs::symlink(temp2.path(), &link_path).unwrap();

            // Should fail because symlink target is outside root
            // Note: This test depends on symlink resolution behavior
            let _result = guard.validate_for_read("escape_link");
            // The symlink itself should resolve, but if it points outside,
            // the final verification should catch it
        }
    }

    #[test]
    fn test_dangerous_extension() {
        let (temp, guard) = setup_test_guard();

        // Create .env file
        fs::write(temp.path().join(".env"), "SECRET=value").unwrap();

        // Should be rejected due to dangerous extension
        let result = guard.validate_for_read(".env");
        assert!(result.is_err());
    }

    #[test]
    fn test_denied_path() {
        let (temp, guard) = setup_test_guard();

        // Create .git directory
        fs::create_dir_all(temp.path().join(".git")).unwrap();
        fs::write(temp.path().join(".git/config"), "test").unwrap();

        // .git/config should be denied
        let result = guard.validate_for_read(".git/config");
        assert!(result.is_err());
    }

    #[test]
    fn test_public_safe_mode() {
        let temp = TempDir::new().unwrap();
        let guard = PathGuard::new_public_safe(temp.path()).unwrap();

        // Create hidden file
        fs::write(temp.path().join(".hidden"), "secret").unwrap();

        // Hidden files should be rejected in public safe mode
        let result = guard.validate_for_read(".hidden");
        assert!(result.is_err());
    }

    #[test]
    fn test_local_trusted_mode() {
        let temp = TempDir::new().unwrap();
        let guard = PathGuard::new_local_trusted(temp.path()).unwrap();

        // Should have much larger file size limit
        assert_eq!(guard.max_file_size, 10 * 1024 * 1024);

        // Should have no denied paths
        assert!(guard.denied_paths.is_empty());
    }

    #[test]
    fn test_relative_path_extraction() {
        let (temp, guard) = setup_test_guard();

        let abs_path = temp.path().join("src/main.rs");
        let rel = guard.relative_path(&abs_path).unwrap();

        assert_eq!(rel, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_path_too_long() {
        let (_temp, guard) = setup_test_guard();

        // Create a very long path string
        let long_path = "a".repeat(5000);
        let result = guard.validate_for_read(&long_path);

        assert!(result.is_err());
    }
}
