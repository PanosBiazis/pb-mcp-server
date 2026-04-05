//! Security module for pb-mcp-server
//!
//! This module provides comprehensive security utilities for:
//! - Path validation and sandboxing
//! - Command execution policies
//! - Rate limiting
//! - Request authentication
//!
//! # Architecture
//!
//! The security module follows defense-in-depth principles:
//! 1. **Input validation**: All inputs are validated at entry points
//! 2. **Path sandboxing**: File operations are confined to project root
//! 3. **Command allowlists**: Only approved commands can be executed
//! 4. **Rate limiting**: Prevents abuse and resource exhaustion
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::security::{PathGuard, CommandPolicy, SecurityContext};
//!
//! // Create security context for a project
//! let ctx = SecurityContext::new("/project/root")?;
//!
//! // Validate a path
//! let safe_path = ctx.validate_path("src/main.rs")?;
//!
//! // Check if a command is allowed
//! if ctx.is_command_allowed("git", &["status"]) {
//!     // Execute command
//! }
//! ```

// Error and Result imports not needed - SecurityContext is commented out
// TODO: Implement security modules (path_guard, command_policy, rate_limiter)
// SecurityContext temporarily commented out until dependencies are implemented

/*
use crate::profiles::ToolProfile;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Comprehensive security context combining all security utilities
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Path validation guard
    path_guard: Arc<PathGuard>,

    /// Command execution policy
    command_policy: Arc<CommandPolicy>,

    /// Rate limiter for operations
    rate_limiter: Arc<RateLimiter>,

    /// Current security profile
    profile: ToolProfile,

    /// Whether this context is for public/remote access
    is_public: bool,
}

impl SecurityContext {
    /// Create a new security context for the given project root
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let path_guard = PathGuard::new(&root)?;

        Ok(SecurityContext {
            path_guard: Arc::new(path_guard),
            command_policy: Arc::new(CommandPolicy::default()),
            rate_limiter: Arc::new(RateLimiter::default()),
            profile: ToolProfile::default(),
            is_public: false,
        })
    }

    /// Create a security context for a specific profile
    pub fn with_profile(root: impl Into<PathBuf>, profile: ToolProfile) -> Result<Self> {
        let root = root.into();
        let path_guard = match profile {
            ToolProfile::PublicSafe => PathGuard::new_public_safe(&root)?,
            ToolProfile::LocalPower => PathGuard::new_local_trusted(&root)?,
            _ => PathGuard::new(&root)?,
        };

        let command_policy = match profile {
            ToolProfile::LocalPower => CommandPolicy::permissive(),
            ToolProfile::PublicSafe => CommandPolicy::restrictive(),
            _ => CommandPolicy::default(),
        };

        Ok(SecurityContext {
            path_guard: Arc::new(path_guard),
            command_policy: Arc::new(command_policy),
            rate_limiter: Arc::new(RateLimiter::default()),
            profile,
            is_public: profile == ToolProfile::PublicSafe,
        })
    }

    /// Create a security context for public/remote access
    pub fn new_public(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_profile(root, ToolProfile::PublicSafe)
    }

    /// Create a security context for local trusted access
    pub fn new_local(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_profile(root, ToolProfile::LocalPower)
    }

    /// Validate a path for read operations
    pub fn validate_path(&self, path: &str) -> Result<PathBuf> {
        self.path_guard.validate_for_read(path)
    }

    /// Validate a path for write operations
    pub fn validate_path_for_write(&self, path: &str) -> Result<PathBuf> {
        if !self.profile.allows_writes() {
            return Err(Error::operation_not_permitted(
                "write_operation",
                self.profile.as_str(),
            ));
        }
        self.path_guard.validate_for_write(path)
    }

    /// Check if a command is allowed
    pub fn is_command_allowed(&self, command: &str, args: &[&str]) -> bool {
        if !self.profile.allows_execution() {
            return false;
        }
        self.command_policy.is_allowed(command, args)
    }

    /// Validate and get command for execution
    pub fn validate_command(&self, command: &str, args: &[&str]) -> Result<()> {
        if !self.profile.allows_execution() {
            return Err(Error::operation_not_permitted(
                "terminal_execution",
                self.profile.as_str(),
            ));
        }
        self.command_policy.validate(command, args)
    }

    /// Check rate limit for an operation
    pub fn check_rate_limit(&self, operation: &str) -> Result<()> {
        self.rate_limiter.check(operation)
    }

    /// Record an operation for rate limiting
    pub fn record_operation(&self, operation: &str) {
        self.rate_limiter.record(operation);
    }

    /// Get the current profile
    pub fn profile(&self) -> ToolProfile {
        self.profile
    }

    /// Check if this is a public context
    pub fn is_public(&self) -> bool {
        self.is_public
    }

    /// Check if writes are allowed
    pub fn allows_writes(&self) -> bool {
        self.profile.allows_writes()
    }

    /// Check if execution is allowed
    pub fn allows_execution(&self) -> bool {
        self.profile.allows_execution()
    }

    /// Get the project root
    pub fn root(&self) -> &std::path::Path {
        self.path_guard.root()
    }

    /// Get a reference to the path guard
    pub fn path_guard(&self) -> &PathGuard {
        &self.path_guard
    }

    /// Get a reference to the command policy
    pub fn command_policy(&self) -> &CommandPolicy {
        &self.command_policy
    }

    /// Get a reference to the rate limiter
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // TODO: Re-enable tests when SecurityContext is implemented
    /*
    #[test]
    fn test_security_context_new() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::CoreCode);
        assert!(!ctx.is_public());
        assert!(!ctx.allows_writes());
        assert!(!ctx.allows_execution());
    }

    #[test]
    fn test_security_context_with_profile() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::LocalPower);
        assert!(ctx.allows_writes());
        assert!(ctx.allows_execution());
    }

    #[test]
    fn test_security_context_public() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new_public(temp.path()).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::PublicSafe);
        assert!(ctx.is_public());
        assert!(!ctx.allows_writes());
        assert!(!ctx.allows_execution());
    }

    #[test]
    fn test_validate_path() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.txt"), "content").unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        let result = ctx.validate_path("test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_for_write_denied() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        // CoreCode profile doesn't allow writes
        let result = ctx.validate_path_for_write("test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_for_write_allowed() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        // LocalPower profile allows writes
        let result = ctx.validate_path_for_write("test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_validation_denied() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        // CoreCode profile doesn't allow execution
        let result = ctx.validate_command("git", &["status"]);
        assert!(result.is_err());
    }
    */
}
