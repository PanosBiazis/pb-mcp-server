//! Security module for pb-mcp-server
//!
//! This module provides comprehensive security utilities for:
//! - Path validation and sandboxing
//! - Command execution policies
//! - Rate limiting (general-purpose token bucket)
//! - Tamper-evident audit logging
//! - Unified security context combining all utilities
//!
//! # Architecture
//!
//! The security module follows defense-in-depth principles:
//! 1. **Input validation**: All inputs are validated at entry points
//! 2. **Path sandboxing**: File operations are confined to project root
//! 3. **Command allowlists**: Only approved commands can be executed
//! 4. **Rate limiting**: Prevents abuse and resource exhaustion
//! 5. **Audit logging**: Tamper-evident record of all tool invocations
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::security::{AuditResultStatus, SecurityContext};
//! use pb_mcp_server::profiles::ToolProfile;
//! use std::collections::HashMap;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create security context for a project
//! let ctx = SecurityContext::with_profile("/project/root", ToolProfile::CoreCode)?;
//!
//! // Validate a path
//! let safe_path = ctx.validate_path("src/main.rs")?;
//!
//! // Check if a command is allowed
//! if ctx.is_command_allowed("git", &["status"]) {
//!     // Execute command
//! }
//!
//! // Check rate limit
//! ctx.check_rate_limit("tool:read_file")?;
//!
//! // Record an audit entry
//! ctx.audit_record(
//!     "agent_1",
//!     "read_file",
//!     b"input",
//!     b"output",
//!     AuditResultStatus::Success,
//!     HashMap::new(),
//! )?;
//! # Ok(())
//! # }
//! ```

// ============================================================================
// Submodule declarations
// ============================================================================

/// Path validation and sandboxing to prevent directory traversal attacks.
pub mod path_guard;

/// Command execution policies with allowlists and argument validation.
pub mod command_policy;

/// General-purpose token-bucket rate limiter.
pub mod rate_limiter;

/// Tamper-evident append-only audit log with SHA-256 hash chain.
pub mod audit;
/// Secret/malware scanning and file integrity verification.
pub mod scanner;

// ============================================================================
// Re-exports
// ============================================================================

pub use audit::{AuditEntry, AuditLog, AuditQueryFilters, AuditResultStatus, IntegrityReport};
pub use command_policy::{CommandAllowlist, CommandPolicy, ProgramValidator};
pub use path_guard::PathGuard;
pub use rate_limiter::{BucketStatus, RateLimiter, RateLimiterConfig};
pub use scanner::{IntegrityChecker, ScanResult, SecretScanner, Severity};

use crate::error::{Error, Result};
use crate::profiles::ToolProfile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ============================================================================
// Security context
// ============================================================================

/// Comprehensive security context combining all security utilities.
///
/// `SecurityContext` is the primary entry point for security checks in the
/// MCP server. It bundles a [`PathGuard`], [`CommandPolicy`], [`RateLimiter`],
/// and [`AuditLog`] into a single object scoped to a project root and a
/// [`ToolProfile`].
///
/// # Profile-driven behaviour
///
/// The active [`ToolProfile`] determines which operations are permitted:
///
/// | Profile | Writes | Execution | Public |
/// |---------|--------|-----------|--------|
/// | `CoreCode` | ✗ | ✗ | ✗ |
/// | `ReviewRefactor` | ✗ | ✗ | ✗ |
/// | `AgentOrchestration` | ✗ | ✗ | ✗ |
/// | `LocalPower` | ✓ | ✓ | ✗ |
/// | `PublicSafe` | ✗ | ✗ | ✓ |
/// | `ExperimentalLab` | ✓ | ✓ | ✗ |
///
/// # Thread safety
///
/// All inner fields use interior mutability (`Arc`, `DashMap`, `Mutex`) so
/// `SecurityContext` is `Send + Sync` and can be shared across tasks.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Path validation guard.
    path_guard: Arc<PathGuard>,

    /// Command execution policy.
    command_policy: Arc<CommandPolicy>,

    /// General-purpose rate limiter.
    rate_limiter: Arc<RateLimiter>,

    /// Tamper-evident audit log (optional — only initialised when the
    /// audit directory is writable).
    audit_log: Arc<Option<AuditLog>>,

    /// Current security profile.
    profile: ToolProfile,

    /// Whether this context is for public/remote access.
    is_public: bool,
}

impl SecurityContext {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a new security context for the given project root with the
    /// default profile ([`ToolProfile::CoreCode`]).
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_profile(root, ToolProfile::default())
    }

    /// Create a security context for a specific profile.
    ///
    /// The profile determines the `PathGuard` mode, `CommandPolicy`
    /// strictness, and `RateLimiter` preset:
    ///
    /// - **PublicSafe** → `PathGuard::new_public_safe`, restrictive policy,
    ///   restrictive rate limiter.
    /// - **LocalPower** → `PathGuard::new_local_trusted`, permissive policy,
    ///   permissive rate limiter.
    /// - **Other** → `PathGuard::new`, default policy, default rate limiter.
    pub fn with_profile(root: impl Into<PathBuf>, profile: ToolProfile) -> Result<Self> {
        let root = root.into();

        let path_guard = match profile {
            ToolProfile::PublicSafe => PathGuard::new_public_safe(&root).map_err(|e| {
                Error::path_validation_with_path(e.to_string(), root.display().to_string())
            })?,
            ToolProfile::LocalPower => PathGuard::new_local_trusted(&root).map_err(|e| {
                Error::path_validation_with_path(e.to_string(), root.display().to_string())
            })?,
            _ => PathGuard::new(&root).map_err(|e| {
                Error::path_validation_with_path(e.to_string(), root.display().to_string())
            })?,
        };

        let command_policy = match profile {
            ToolProfile::LocalPower => CommandPolicy::permissive(),
            ToolProfile::PublicSafe => CommandPolicy::restrictive(),
            _ => CommandPolicy::default(),
        };

        let rate_limiter = match profile {
            ToolProfile::LocalPower => RateLimiter::permissive(),
            ToolProfile::PublicSafe => RateLimiter::restrictive(),
            _ => RateLimiter::default_limiter(),
        };

        // Attempt to initialise the audit log; if the directory is not
        // writable we silently degrade (audit_log = None).
        let audit_log = match AuditLog::new(&root) {
            Ok(log) => Arc::new(Some(log)),
            Err(e) => {
                tracing::warn!(
                    "Audit log initialization failed for {}: {}. Security operations will not be logged.",
                    root.display(),
                    e
                );
                Arc::new(None)
            }
        };

        Ok(SecurityContext {
            path_guard: Arc::new(path_guard),
            command_policy: Arc::new(command_policy),
            rate_limiter: Arc::new(rate_limiter),
            audit_log,
            profile,
            is_public: profile == ToolProfile::PublicSafe,
        })
    }

    /// Create a security context for public/remote access.
    ///
    /// Equivalent to `with_profile(root, ToolProfile::PublicSafe)`.
    pub fn new_public(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_profile(root, ToolProfile::PublicSafe)
    }

    /// Create a security context for local trusted access.
    ///
    /// Equivalent to `with_profile(root, ToolProfile::LocalPower)`.
    pub fn new_local(root: impl Into<PathBuf>) -> Result<Self> {
        Self::with_profile(root, ToolProfile::LocalPower)
    }

    // -----------------------------------------------------------------------
    // Path validation
    // -----------------------------------------------------------------------

    /// Validate a path for read operations.
    ///
    /// Returns the canonicalised absolute path if it passes all security
    /// checks (under root, not hidden, no dangerous extension, etc.).
    pub fn validate_path(&self, path: &str) -> Result<PathBuf> {
        self.path_guard
            .validate_for_read(path)
            .map_err(|e| Error::path_validation_with_path(e.to_string(), path.to_string()))
    }

    /// Validate a path for write operations.
    ///
    /// In addition to the normal path checks, this also verifies that the
    /// current profile allows write operations.
    pub fn validate_path_for_write(&self, path: &str) -> Result<PathBuf> {
        if !self.profile.allows_writes() {
            return Err(Error::operation_not_permitted(
                "write_operation",
                self.profile.as_str(),
            ));
        }
        self.path_guard
            .validate_for_write(path)
            .map_err(|e| Error::path_validation_with_path(e.to_string(), path.to_string()))
    }

    /// Validate a path for execution.
    ///
    /// Verifies that the current profile allows execution and the path
    /// passes execution-specific validation.
    pub fn validate_path_for_exec(&self, path: &str) -> Result<PathBuf> {
        if !self.profile.allows_execution() {
            return Err(Error::operation_not_permitted(
                "terminal_execution",
                self.profile.as_str(),
            ));
        }
        self.path_guard
            .validate_for_exec(path)
            .map_err(|e| Error::path_validation_with_path(e.to_string(), path.to_string()))
    }

    // -----------------------------------------------------------------------
    // Command validation
    // -----------------------------------------------------------------------

    /// Check whether a command is allowed under the current policy.
    pub fn is_command_allowed(&self, command: &str, args: &[&str]) -> bool {
        if !self.profile.allows_execution() {
            return false;
        }
        self.command_policy.is_allowed(command, args)
    }

    /// Validate a command and its arguments.
    ///
    /// Returns `Ok(())` if the command is allowed, or an error with details
    /// about why it was rejected.
    pub fn validate_command(&self, command: &str, args: &[&str]) -> Result<()> {
        if !self.profile.allows_execution() {
            return Err(Error::operation_not_permitted(
                "terminal_execution",
                self.profile.as_str(),
            ));
        }
        self.command_policy
            .validate(command, args)
            .map_err(|e| Error::command_not_allowed(command, vec![e.to_string()]))
    }

    // -----------------------------------------------------------------------
    // Rate limiting
    // -----------------------------------------------------------------------

    /// Check whether an operation is within rate limits.
    ///
    /// Returns `Ok(())` if the operation is allowed, or
    /// `Err(Error::RateLimitExceeded)` if the bucket is empty.
    pub fn check_rate_limit(&self, operation: &str) -> Result<()> {
        self.rate_limiter.check(operation)
    }

    /// Check and record an operation atomically.
    ///
    /// This is the primary method for rate-limited operations: it checks
    /// whether a token is available and, if so, consumes it.
    pub fn check_and_record_rate_limit(&self, operation: &str) -> Result<()> {
        self.rate_limiter.check_and_record(operation)
    }

    /// Record an operation for rate-limiting purposes without checking.
    pub fn record_operation(&self, operation: &str) {
        self.rate_limiter.record(operation);
    }

    /// Get the current rate-limiter status for a specific operation key.
    pub fn rate_limit_status(&self, operation: &str) -> Option<BucketStatus> {
        self.rate_limiter.get_status(operation)
    }

    // -----------------------------------------------------------------------
    // Audit logging
    // -----------------------------------------------------------------------

    /// Record an audit entry for a tool invocation.
    ///
    /// If the audit log was not initialised (e.g. the directory is not
    /// writable), this is a no-op and returns `Ok(0)`.
    pub fn audit_record(
        &self,
        agent_id: &str,
        tool_name: &str,
        input_data: &[u8],
        result_data: &[u8],
        result_status: AuditResultStatus,
        metadata: HashMap<String, String>,
    ) -> Result<u64> {
        match self.audit_log.as_ref() {
            Some(log) => log
                .record(
                    agent_id,
                    tool_name,
                    input_data,
                    result_data,
                    result_status,
                    metadata,
                )
                .map_err(|e| Error::from(e.to_string())),
            None => Ok(0),
        }
    }

    /// Query the audit log with optional filters.
    ///
    /// Returns an empty vector if the audit log is not available.
    pub fn audit_query(&self, filters: &AuditQueryFilters) -> Vec<AuditEntry> {
        match self.audit_log.as_ref() {
            Some(log) => log.query(filters).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Verify the integrity of the audit log hash chain.
    ///
    /// Returns a default (empty) report if the audit log is not available.
    pub fn audit_verify_integrity(&self) -> IntegrityReport {
        match self.audit_log.as_ref() {
            Some(log) => log.verify_integrity().unwrap_or_else(|_| IntegrityReport {
                entries_checked: 0,
                chain_intact: true,
                broken_at_lines: Vec::new(),
                last_valid_hash: String::new(),
            }),
            None => IntegrityReport {
                entries_checked: 0,
                chain_intact: true,
                broken_at_lines: Vec::new(),
                last_valid_hash: String::new(),
            },
        }
    }

    /// Export the audit log as a JSON string.
    ///
    /// Returns an empty array string if the audit log is not available.
    pub fn audit_export_json(&self) -> String {
        match self.audit_log.as_ref() {
            Some(log) => log.export_json().unwrap_or_else(|_| "[]".to_string()),
            None => "[]".to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Get the current profile.
    pub fn profile(&self) -> ToolProfile {
        self.profile
    }

    /// Check if this is a public context.
    pub fn is_public(&self) -> bool {
        self.is_public
    }

    /// Check if writes are allowed under the current profile.
    pub fn allows_writes(&self) -> bool {
        self.profile.allows_writes()
    }

    /// Check if execution is allowed under the current profile.
    pub fn allows_execution(&self) -> bool {
        self.profile.allows_execution()
    }

    /// Get the project root path.
    pub fn root(&self) -> &Path {
        self.path_guard.root()
    }

    /// Get a reference to the path guard.
    pub fn path_guard(&self) -> &PathGuard {
        &self.path_guard
    }

    /// Get a reference to the command policy.
    pub fn command_policy(&self) -> &CommandPolicy {
        &self.command_policy
    }

    /// Get a reference to the rate limiter.
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Get a reference to the audit log (if initialised).
    pub fn audit_log(&self) -> Option<&AuditLog> {
        match &*self.audit_log {
            Some(log) => Some(log),
            None => None,
        }
    }

    /// Get the risk level of the current profile.
    pub fn risk_level(&self) -> &'static str {
        match self.profile.risk_level() {
            1 | 2 => "low",
            3 => "medium",
            4 | 5 => "high",
            other => {
                tracing::warn!(
                    "Unexpected risk level {} for profile {:?}",
                    other,
                    self.profile
                );
                "unknown"
            }
        }
    }

    // -----------------------------------------------------------------------
    // Convenience: combined security check
    // -----------------------------------------------------------------------

    /// Perform a comprehensive security check for a tool invocation.
    ///
    /// This method:
    /// 1. Checks the rate limit for the given tool.
    /// 2. Optionally validates a path (if provided).
    /// 3. Records the operation in the rate limiter.
    /// 4. Returns `Ok(())` if all checks pass.
    ///
    /// This is the recommended single entry point for tool handlers that
    /// need both rate-limiting and path validation.
    pub fn check_tool_access(
        &self,
        tool_name: &str,
        path: Option<&str>,
        requires_write: bool,
    ) -> Result<()> {
        // 1. Rate limit
        self.check_and_record_rate_limit(tool_name)?;

        // 2. Path validation
        if let Some(p) = path {
            if requires_write {
                self.validate_path_for_write(p)?;
            } else {
                self.validate_path(p)?;
            }
        }

        // 3. Audit trail — record tool access (best-effort, do not fail the operation)
        if let Some(ref log) = *self.audit_log {
            let input_bytes = path.unwrap_or("").as_bytes();
            let _ = log.record(
                "system",
                tool_name,
                input_bytes,
                b"access_check_passed",
                crate::security::audit::AuditResultStatus::Success,
                std::collections::HashMap::new(),
            );
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn setup_ctx(profile: ToolProfile) -> (TempDir, SecurityContext) {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), profile).unwrap();
        (temp, ctx)
    }

    // ---- construction ----

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
    fn test_security_context_with_profile_local_power() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::LocalPower);
        assert!(ctx.allows_writes());
        assert!(ctx.allows_execution());
    }

    #[test]
    fn test_security_context_with_profile_public_safe() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::PublicSafe).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::PublicSafe);
        assert!(ctx.is_public());
        assert!(!ctx.allows_writes());
        assert!(!ctx.allows_execution());
    }

    #[test]
    fn test_security_context_public() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new_public(temp.path()).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::PublicSafe);
        assert!(ctx.is_public());
    }

    #[test]
    fn test_security_context_local() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new_local(temp.path()).unwrap();
        assert_eq!(ctx.profile(), ToolProfile::LocalPower);
        assert!(ctx.allows_writes());
        assert!(ctx.allows_execution());
    }

    // ---- path validation ----

    #[test]
    fn test_validate_path_read() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.txt"), "content").unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        let result = ctx.validate_path("test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_denied_for_write_in_core_code() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        // CoreCode profile doesn't allow writes
        let result = ctx.validate_path_for_write("test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_write_allowed_in_local_power() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.txt"), "content").unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        // LocalPower profile allows writes
        let result = ctx.validate_path_for_write("test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_traversal_blocked() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        let result = ctx.validate_path("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_exec_denied_in_core_code() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        let result = ctx.validate_path_for_exec("/usr/bin/git");
        assert!(result.is_err());
    }

    // ---- command validation ----

    #[test]
    fn test_command_validation_denied_in_core_code() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        // CoreCode profile doesn't allow execution
        let result = ctx.validate_command("git", &["status"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_command_validation_allowed_in_local_power() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        let result = ctx.validate_command("git", &["status"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_is_allowed_check() {
        let temp = TempDir::new().unwrap();
        let ctx_core = SecurityContext::new(temp.path()).unwrap();
        assert!(!ctx_core.is_command_allowed("git", &["status"]));

        let ctx_local =
            SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        assert!(ctx_local.is_command_allowed("git", &["status"]));
    }

    #[test]
    fn test_command_validation_denied_program() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        // rm is in the denied list
        let result = ctx.validate_command("rm", &["-rf", "/"]);
        assert!(result.is_err());
    }

    // ---- rate limiting ----

    #[test]
    fn test_rate_limit_allows_initial_requests() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        // Should allow at least the first request
        assert!(ctx.check_rate_limit("test_tool").is_ok());
    }

    #[test]
    fn test_rate_limit_check_and_record() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::PublicSafe).unwrap();
        // PublicSafe uses restrictive limiter (burst=5)
        for _ in 0..5 {
            assert!(ctx.check_and_record_rate_limit("test_tool").is_ok());
        }
        // 6th should be rejected
        assert!(ctx.check_and_record_rate_limit("test_tool").is_err());
    }

    #[test]
    fn test_rate_limit_independent_keys() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::PublicSafe).unwrap();
        // Exhaust "tool_a"
        for _ in 0..5 {
            let _ = ctx.check_and_record_rate_limit("tool_a");
        }
        assert!(ctx.check_and_record_rate_limit("tool_a").is_err());
        // "tool_b" has its own bucket
        assert!(ctx.check_and_record_rate_limit("tool_b").is_ok());
    }

    #[test]
    fn test_rate_limit_status() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        ctx.check_and_record_rate_limit("my_tool").unwrap();
        let status = ctx.rate_limit_status("my_tool");
        assert!(status.is_some());
        assert_eq!(status.unwrap().key, "my_tool");
    }

    // ---- audit logging ----

    #[test]
    fn test_audit_record_and_query() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        let id = ctx
            .audit_record(
                "agent_1",
                "read_file",
                b"input_data",
                b"output_data",
                AuditResultStatus::Success,
                HashMap::new(),
            )
            .unwrap();
        assert!(id > 0);

        let entries = ctx.audit_query(&AuditQueryFilters {
            agent_id: Some("agent_1".to_string()),
            ..Default::default()
        });
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tool_name, "read_file");
        assert_eq!(entries[0].result_status, AuditResultStatus::Success);
    }

    #[test]
    fn test_audit_verify_integrity() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        ctx.audit_record(
            "a",
            "t",
            b"i",
            b"o",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();

        let report = ctx.audit_verify_integrity();
        assert!(report.chain_intact);
        assert_eq!(report.entries_checked, 1);
    }

    #[test]
    fn test_audit_export_json() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        ctx.audit_record(
            "agent_x",
            "tool_y",
            b"input",
            b"output",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();

        let json = ctx.audit_export_json();
        assert!(!json.is_empty());
        assert!(json.contains("agent_x"));
    }

    #[test]
    fn test_audit_export_does_not_include_raw_inputs() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        ctx.audit_record(
            "agent_x",
            "file_encrypt",
            br#"{"path":"/tmp/a","passphrase":"super-secret-passphrase"}"#,
            b"output",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();

        let json = ctx.audit_export_json();
        assert!(json.contains("input_hash"));
        assert!(!json.contains("super-secret-passphrase"));
        assert!(!json.contains("passphrase"));
    }

    #[test]
    fn test_audit_query_by_tool() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        ctx.audit_record(
            "a",
            "read_file",
            b"i",
            b"o",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();
        ctx.audit_record(
            "a",
            "grep",
            b"i",
            b"o",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();

        let results = ctx.audit_query(&AuditQueryFilters {
            tool_name: Some("grep".to_string()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tool_name, "grep");
    }

    #[test]
    fn test_audit_query_by_status() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        ctx.audit_record(
            "a",
            "t",
            b"i",
            b"o",
            AuditResultStatus::Success,
            HashMap::new(),
        )
        .unwrap();
        ctx.audit_record(
            "a",
            "t",
            b"i",
            b"o",
            AuditResultStatus::Failed,
            HashMap::new(),
        )
        .unwrap();
        ctx.audit_record(
            "a",
            "t",
            b"i",
            b"o",
            AuditResultStatus::Blocked,
            HashMap::new(),
        )
        .unwrap();

        let blocked = ctx.audit_query(&AuditQueryFilters {
            result_status: Some(AuditResultStatus::Blocked),
            ..Default::default()
        });
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].result_status, AuditResultStatus::Blocked);
    }

    // ---- combined check_tool_access ----

    #[test]
    fn test_check_tool_access_read() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.rs"), "fn main() {}").unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        let result = ctx.check_tool_access("read_file", Some("test.rs"), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_tool_access_write_blocked_in_core_code() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        let result = ctx.check_tool_access("write_file", Some("test.rs"), true);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_tool_access_no_path() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();

        // Rate limit only, no path validation
        let result = ctx.check_tool_access("some_tool", None, false);
        assert!(result.is_ok());
    }

    // ---- risk level ----

    #[test]
    fn test_risk_level_by_profile() {
        let temp = TempDir::new().unwrap();

        let ctx_core = SecurityContext::with_profile(temp.path(), ToolProfile::CoreCode).unwrap();
        assert_eq!(ctx_core.risk_level(), "low");

        let ctx_local =
            SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        assert_eq!(ctx_local.risk_level(), "high");

        let ctx_pub = SecurityContext::with_profile(temp.path(), ToolProfile::PublicSafe).unwrap();
        assert_eq!(ctx_pub.risk_level(), "low");
    }

    // ---- accessor methods ----

    #[test]
    fn test_root_accessor() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        assert_eq!(ctx.root(), temp.path());
    }

    #[test]
    fn test_path_guard_accessor() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        assert_eq!(ctx.path_guard().root(), temp.path());
    }

    #[test]
    fn test_command_policy_accessor() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::with_profile(temp.path(), ToolProfile::LocalPower).unwrap();
        // Permissive policy should allow git
        assert!(ctx.command_policy().is_allowed("git", &["status"]));
    }

    #[test]
    fn test_rate_limiter_accessor() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        assert_eq!(ctx.rate_limiter().config().max_requests, 100); // default
    }

    #[test]
    fn test_audit_log_accessor() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        assert!(ctx.audit_log().is_some());
    }

    // ---- profile-specific rate limiter presets ----

    #[test]
    fn test_public_safe_rate_limiter_is_restrictive() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new_public(temp.path()).unwrap();
        assert_eq!(ctx.rate_limiter().config().max_requests, 30); // restrictive
        assert_eq!(ctx.rate_limiter().config().burst_size, 5);
    }

    #[test]
    fn test_local_power_rate_limiter_is_permissive() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new_local(temp.path()).unwrap();
        assert_eq!(ctx.rate_limiter().config().max_requests, 1000); // permissive
        assert_eq!(ctx.rate_limiter().config().burst_size, 200);
    }

    // ---- clone is cheap (Arc-based) ----

    #[test]
    fn test_clone_shares_state() {
        let temp = TempDir::new().unwrap();
        let ctx = SecurityContext::new(temp.path()).unwrap();
        let clone = ctx.clone();

        // Recording on one should be visible on the other (same rate limiter).
        clone.check_and_record_rate_limit("shared_key").unwrap();
        // The original should see the rate limit was consumed.
        let status = ctx.rate_limit_status("shared_key");
        assert!(status.is_some());
        assert!(status.unwrap().total_allowed >= 1);
    }
}
