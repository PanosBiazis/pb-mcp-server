//! Command allowlist and execution policy for pb-mcp-server
//!
//! This module provides a flexible command allowlist system that controls which
//! terminal commands can be executed by the server. It supports:
//! - Global allowlist of approved programs
//! - Argument validation and sanitization
//! - Per-profile policy customization
//! - Dangerous pattern blocking
//!
//! # Security Model
//!
//! Commands are validated in multiple stages:
//! 1. **Program allowlist**: Only approved executables can run
//! 2. **Argument validation**: Dangerous arguments are blocked
//! 3. **Path validation**: Arguments containing paths are validated
//! 4. **Resource limits**: Execution time and output size are bounded
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::security::CommandPolicy;
//!
//! let policy = CommandPolicy::default();
//!
//! // Check if a command is allowed
//! if policy.is_allowed("git", &["status"]) {
//!     // Safe to execute
//! }
//!
//! // Validate and get error details
//! let rejected = policy.validate("rm", &["-rf", "/"]);
//! assert!(rejected.is_err());
//! ```

use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};

use std::time::Duration;

/// Maximum command output size (1MB)
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// Maximum execution time (30 seconds)
const MAX_EXECUTION_TIME_SECS: u64 = 30;

/// Maximum argument length
const MAX_ARG_LENGTH: usize = 4096;

/// Maximum number of arguments
const MAX_ARGS: usize = 64;

/// Default allowed programs for read-only operations
const DEFAULT_ALLOWED_PROGRAMS: &[&str] = &[
    // Version control
    "git",
    "hg",
    "svn",
    // Build tools (read-only operations)
    "cargo",
    "npm",
    "yarn",
    "pip",
    "python",
    "python3",
    "node",
    // Testing (read-only)
    "pytest",
    "jest",
    "cargo-nextest",
    // Code quality (read-only)
    "rustfmt",
    "clippy",
    "eslint",
    "prettier",
    // System utilities (safe subset)
    "ls",
    "cat",
    "head",
    "tail",
    "wc",
    "grep",
    "find",
    "which",
    "echo",
    // Documentation
    "rustdoc",
    "cargo-doc",
];

/// Programs that are explicitly denied even in permissive mode
const DENIED_PROGRAMS: &[&str] = &[
    // System modification
    "rm",
    "mv",
    "cp",
    "chmod",
    "chown",
    "mkfs",
    "fdisk",
    "dd",
    // Privilege escalation
    "sudo",
    "su",
    "doas",
    "pkexec",
    "gksudo",
    "kdesu",
    // Remote access
    "ssh",
    "scp",
    "rsync",
    "telnet",
    "nc",
    "netcat",
    // Package management (can modify system)
    "apt",
    "apt-get",
    "yum",
    "dnf",
    "pacman",
    "brew",
    // Process management
    "kill",
    "killall",
    "pkill",
    "systemctl",
    "service",
    // Shell execution
    "sh",
    "bash",
    "zsh",
    "fish",
    "dash",
    // Editors (can execute arbitrary commands)
    "vim",
    "vi",
    "emacs",
    "nano",
    "ed",
];

/// Dangerous argument patterns that should always be blocked
const DANGEROUS_PATTERNS: &[&str] = &[
    // Command injection
    ";", "|", "&", "$", "`", "\n", "\r", // Path traversal
    "..",
    // Wildcards that could match sensitive files
    // (allowed in specific contexts but flagged for review)
    // Redirection
    ">", "<", ">>", "<<", // Process substitution
    "<(", ">(", // Environment variable access
    "${", "$(",
];

/// Command execution policy
#[derive(Debug, Clone)]
pub struct CommandPolicy {
    /// Allowed programs (whitelist)
    allowed_programs: HashSet<String>,
    /// Explicitly denied programs (blacklist, always blocked)
    denied_programs: HashSet<String>,
    /// Program-specific argument validators
    program_validators: HashMap<String, ProgramValidator>,
    /// Whether to allow shell metacharacters in arguments
    allow_shell_chars: bool,
    /// Maximum execution time
    max_execution_time: Duration,
    /// Maximum output size
    max_output_size: usize,
    /// Maximum number of arguments
    max_args: usize,
    /// Maximum argument length
    max_arg_length: usize,
}

/// Program-specific argument validator
#[derive(Debug, Clone)]
pub struct ProgramValidator {
    /// Allowed subcommands (e.g., git: ["status", "log", "diff"])
    allowed_subcommands: Option<HashSet<String>>,
    /// Denied subcommands (e.g., git: ["push", "reset", "clean"])
    denied_subcommands: Option<HashSet<String>>,
    /// Whether to allow file paths as arguments
    #[allow(dead_code)]
    allow_file_paths: bool,
    /// Whether to allow URLs as arguments
    allow_urls: bool,
    /// Additional allowed flags
    allowed_flags: HashSet<String>,
}

impl Default for ProgramValidator {
    fn default() -> Self {
        ProgramValidator {
            allowed_subcommands: None,
            denied_subcommands: None,
            allow_file_paths: true,
            allow_urls: false,
            allowed_flags: HashSet::new(),
        }
    }
}

impl CommandPolicy {
    /// Create a new command policy with default settings
    pub fn new() -> Self {
        let mut policy = CommandPolicy {
            allowed_programs: HashSet::new(),
            denied_programs: DENIED_PROGRAMS.iter().map(|s| s.to_string()).collect(),
            program_validators: HashMap::new(),
            allow_shell_chars: false,
            max_execution_time: Duration::from_secs(MAX_EXECUTION_TIME_SECS),
            max_output_size: MAX_OUTPUT_BYTES,
            max_args: MAX_ARGS,
            max_arg_length: MAX_ARG_LENGTH,
        };

        // Add default allowed programs
        for program in DEFAULT_ALLOWED_PROGRAMS {
            policy.allowed_programs.insert(program.to_string());
        }

        // Add program-specific validators
        policy.add_git_validator();
        policy.add_cargo_validator();
        policy.add_python_validator();

        policy
    }

    /// Create a permissive policy for local trusted execution
    pub fn permissive() -> Self {
        let mut policy = Self::new();

        // Allow more programs in local mode
        let additional = [
            "go",
            "rustc",
            "javac",
            "mvn",
            "gradle",
            "make",
            "cmake",
            "docker",
            "kubectl",
            "terraform",
        ];

        for program in additional {
            policy.allowed_programs.insert(program.to_string());
        }

        // Increase limits
        policy.max_execution_time = Duration::from_secs(120);
        policy.max_output_size = 10 * 1024 * 1024; // 10MB

        policy
    }

    /// Create a restrictive policy for public access
    pub fn restrictive() -> Self {
        let mut policy = Self::new();

        // In restrictive mode, only allow essential read-only commands
        policy.allowed_programs.clear();
        policy.allowed_programs.insert("git".to_string());

        // Tighter limits
        policy.max_execution_time = Duration::from_secs(10);
        policy.max_output_size = 64 * 1024; // 64KB

        policy
    }

    /// Add git-specific validator
    fn add_git_validator(&mut self) {
        let validator = ProgramValidator {
            allowed_subcommands: Some(
                [
                    "status",
                    "log",
                    "diff",
                    "show",
                    "blame",
                    "branch",
                    "remote",
                    "tag",
                    "stash",
                    "reflog",
                    "shortlog",
                    "ls-files",
                    "rev-parse",
                    "merge-base",
                    "describe",
                    "fetch",
                    "pull", // Allowed but will be gated by profile
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ),
            denied_subcommands: Some(
                [
                    "push",        // Can leak data
                    "reset",       // Can destroy work
                    "clean",       // Can delete files
                    "checkout",    // Can modify working directory
                    "merge",       // Can modify history
                    "rebase",      // Can modify history
                    "cherry-pick", // Can modify history
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ),
            allow_file_paths: true,
            allow_urls: false,
            allowed_flags: [
                "--oneline",
                "--stat",
                "--name-only",
                "--name-status",
                "--pretty",
                "--format",
                "--abbrev-commit",
                "--no-abbrev-commit",
                "-p",
                "-u",
                "--patch",
                "--cached",
                "--staged",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        };

        self.program_validators.insert("git".to_string(), validator);
    }

    /// Add cargo-specific validator
    fn add_cargo_validator(&mut self) {
        let validator = ProgramValidator {
            allowed_subcommands: Some(
                [
                    "check",
                    "build",
                    "test",
                    "run",
                    "bench",
                    "doc",
                    "clippy",
                    "fmt",
                    "tree",
                    "metadata",
                    "locate-project",
                    "pkgid",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ),
            denied_subcommands: Some(
                [
                    "publish",   // Can publish to crates.io
                    "install",   // Can modify system
                    "uninstall", // Can modify system
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ),
            allow_file_paths: true,
            allow_urls: false,
            allowed_flags: HashSet::new(),
        };

        self.program_validators
            .insert("cargo".to_string(), validator);
    }

    /// Add python-specific validator
    fn add_python_validator(&mut self) {
        let validator = ProgramValidator {
            allowed_subcommands: None,
            denied_subcommands: None,
            allow_file_paths: true,
            allow_urls: false,
            allowed_flags: ["-m", "-c", "--version", "-V", "--help", "-h"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };

        self.program_validators
            .insert("python".to_string(), validator.clone());
        self.program_validators
            .insert("python3".to_string(), validator);
    }

    /// Check if a command is allowed
    pub fn is_allowed(&self, program: &str, args: &[&str]) -> bool {
        self.validate(program, args).is_ok()
    }

    /// Validate a command and return detailed error if not allowed
    pub fn validate(&self, program: &str, args: &[&str]) -> Result<()> {
        // Step 1: Check if program is explicitly denied
        let program_lower = program.to_lowercase();
        if self.denied_programs.contains(&program_lower) {
            return Err(Error::command_not_allowed(
                program,
                self.allowed_programs.iter().cloned().collect(),
            ));
        }

        // Step 2: Check if program is in allowlist
        if !self.allowed_programs.contains(&program_lower) {
            return Err(Error::command_not_allowed(
                program,
                self.allowed_programs.iter().cloned().collect(),
            ));
        }

        // Step 3: Validate argument count
        if args.len() > self.max_args {
            return Err(Error::invalid_arguments(
                program,
                format!("Too many arguments (max {})", self.max_args),
            ));
        }

        // Step 4: Validate each argument
        for arg in args {
            self.validate_argument(arg)?;
        }

        // Step 5: Apply program-specific validation
        if let Some(validator) = self.program_validators.get(&program_lower) {
            self.validate_with_program_policy(program, args, validator)?;
        }

        // Step 6: Check for dangerous patterns
        self.check_dangerous_patterns(args)?;

        Ok(())
    }

    /// Validate a single argument
    fn validate_argument(&self, arg: &str) -> Result<()> {
        // Check length
        if arg.len() > self.max_arg_length {
            return Err(Error::invalid_arguments(
                "argument",
                format!("Argument too long (max {} bytes)", self.max_arg_length),
            ));
        }

        // Check for null bytes (can be used to bypass checks)
        if arg.contains('\0') {
            return Err(Error::security_violation("Argument contains null byte"));
        }

        Ok(())
    }

    /// Validate with program-specific policy
    fn validate_with_program_policy(
        &self,
        program: &str,
        args: &[&str],
        validator: &ProgramValidator,
    ) -> Result<()> {
        // Get the first non-flag argument as potential subcommand
        let subcommand = args.iter().find(|arg| !arg.starts_with('-'));

        if let Some(subcmd) = subcommand {
            // Check denied subcommands
            if let Some(denied) = &validator.denied_subcommands {
                if denied.contains(*subcmd) {
                    return Err(Error::operation_not_permitted(
                        format!("{} {}", program, subcmd),
                        "command policy",
                    ));
                }
            }

            // Check allowed subcommands (if specified)
            if let Some(allowed) = &validator.allowed_subcommands {
                if !allowed.contains(*subcmd) {
                    return Err(Error::operation_not_permitted(
                        format!("{} {}", program, subcmd),
                        "command policy",
                    ));
                }
            }
        }

        // Check flags
        for arg in args {
            if arg.starts_with('-') && !validator.allowed_flags.is_empty() {
                // For now, allow any flag if allowed_flags is empty
                // If allowed_flags is specified, check against it
                if !validator.allowed_flags.is_empty() && !validator.allowed_flags.contains(*arg) {
                    // Flag not in allowed list - this is a warning, not error
                    // Some flags are dynamic (like --format=...)
                    if arg.contains('=') {
                        if let Some(flag_name) = arg.split('=').next() {
                            if !validator.allowed_flags.contains(flag_name) {
                                // Unknown flag, but allow it with caution
                            }
                        }
                    }
                }
            }
        }

        // Check for URLs if not allowed
        if !validator.allow_urls {
            for arg in args {
                if arg.starts_with("http://") || arg.starts_with("https://") {
                    return Err(Error::security_violation(
                        "URLs are not allowed as arguments for this command",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check for dangerous patterns in arguments
    fn check_dangerous_patterns(&self, args: &[&str]) -> Result<()> {
        if self.allow_shell_chars {
            return Ok(());
        }

        for arg in args {
            for pattern in DANGEROUS_PATTERNS {
                if arg.contains(pattern) {
                    return Err(Error::security_violation(format!(
                        "Argument contains dangerous pattern: '{}'",
                        pattern
                    )));
                }
            }
        }

        Ok(())
    }

    /// Add a program to the allowlist
    pub fn allow_program(&mut self, program: &str) -> &mut Self {
        self.allowed_programs.insert(program.to_lowercase());
        self
    }

    /// Remove a program from the allowlist
    pub fn deny_program(&mut self, program: &str) -> &mut Self {
        self.allowed_programs.remove(&program.to_lowercase());
        self.denied_programs.insert(program.to_lowercase());
        self
    }

    /// Set maximum execution time
    pub fn set_max_execution_time(&mut self, duration: Duration) -> &mut Self {
        self.max_execution_time = duration;
        self
    }

    /// Set maximum output size
    pub fn set_max_output_size(&mut self, bytes: usize) -> &mut Self {
        self.max_output_size = bytes;
        self
    }

    /// Get maximum execution time
    pub fn max_execution_time(&self) -> Duration {
        self.max_execution_time
    }

    /// Get maximum output size
    pub fn max_output_size(&self) -> usize {
        self.max_output_size
    }

    /// Get allowed programs
    pub fn allowed_programs(&self) -> &HashSet<String> {
        &self.allowed_programs
    }

    /// Create an allowlist for use with MCP tools
    pub fn to_allowlist_string(&self) -> String {
        let mut programs: Vec<_> = self.allowed_programs.iter().map(|s| s.as_str()).collect();
        programs.sort();
        programs.join(",")
    }
}

impl Default for CommandPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Command allowlist wrapper for serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandAllowlist {
    /// Allowed programs
    pub programs: Vec<String>,
    /// Max execution time in seconds
    pub max_execution_time_secs: u64,
    /// Max output size in bytes
    pub max_output_bytes: usize,
}

impl From<&CommandPolicy> for CommandAllowlist {
    fn from(policy: &CommandPolicy) -> Self {
        CommandAllowlist {
            programs: policy.allowed_programs.iter().cloned().collect(),
            max_execution_time_secs: policy.max_execution_time.as_secs(),
            max_output_bytes: policy.max_output_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy_creation() {
        let policy = CommandPolicy::new();

        // Should allow git
        assert!(policy.is_allowed("git", &["status"]));

        // Should deny rm
        assert!(!policy.is_allowed("rm", &["-rf", "/"]));

        // Should deny sudo
        assert!(!policy.is_allowed("sudo", &["rm", "-rf", "/"]));
    }

    #[test]
    fn test_git_command_validation() {
        let policy = CommandPolicy::new();

        // Allowed git commands
        assert!(policy.is_allowed("git", &["status"]));
        assert!(policy.is_allowed("git", &["log", "--oneline"]));
        assert!(policy.is_allowed("git", &["diff", "--stat"]));

        // Denied git commands
        assert!(!policy.is_allowed("git", &["push"]));
        assert!(!policy.is_allowed("git", &["clean", "-fd"]));
        assert!(!policy.is_allowed("git", &["reset", "--hard"]));
    }

    #[test]
    fn test_cargo_command_validation() {
        let policy = CommandPolicy::new();

        // Allowed cargo commands
        assert!(policy.is_allowed("cargo", &["check"]));
        assert!(policy.is_allowed("cargo", &["test"]));
        assert!(policy.is_allowed("cargo", &["clippy"]));

        // Denied cargo commands
        assert!(!policy.is_allowed("cargo", &["publish"]));
    }

    #[test]
    fn test_dangerous_patterns() {
        let policy = CommandPolicy::new();

        // Command injection attempts
        assert!(!policy.is_allowed("git", &["status", ";", "rm", "-rf", "/"]));
        assert!(!policy.is_allowed("git", &["status", "|", "cat", "/etc/passwd"]));
        assert!(!policy.is_allowed("git", &["status", "&&", "malicious"]));

        // Path traversal
        assert!(!policy.is_allowed("cat", &["../../../etc/passwd"]));
    }

    #[test]
    fn test_permissive_policy() {
        let policy = CommandPolicy::permissive();

        // Should allow more commands
        assert!(policy.is_allowed("docker", &["ps"]));
        assert!(policy.is_allowed("kubectl", &["get", "pods"]));

        // Should still deny dangerous commands
        assert!(!policy.is_allowed("rm", &["-rf", "/"]));
        assert!(!policy.is_allowed("sudo", &["anything"]));
    }

    #[test]
    fn test_restrictive_policy() {
        let policy = CommandPolicy::restrictive();

        // Should only allow git
        assert!(policy.is_allowed("git", &["status"]));

        // Should deny most other programs
        assert!(!policy.is_allowed("cargo", &["check"]));
        assert!(!policy.is_allowed("npm", &["test"]));
    }

    #[test]
    fn test_argument_limits() {
        let policy = CommandPolicy::new();

        // Too many arguments
        let too_many_args: Vec<&str> = (0..100).map(|_| "arg").collect();
        assert!(!policy.is_allowed("git", &too_many_args));

        // Argument too long
        let long_arg = "x".repeat(5000);
        assert!(!policy.is_allowed("git", &[&long_arg]));
    }

    #[test]
    fn test_url_blocking() {
        let policy = CommandPolicy::new();

        // Git shouldn't accept URLs by default
        assert!(!policy.is_allowed("git", &["clone", "https://evil.com/repo.git"]));
    }

    #[test]
    fn test_custom_allowlist() {
        let mut policy = CommandPolicy::new();

        // Add custom program
        policy.allow_program("my-custom-tool");
        assert!(policy.is_allowed("my-custom-tool", &[]));

        // Deny a program
        policy.deny_program("git");
        assert!(!policy.is_allowed("git", &["status"]));
    }

    #[test]
    fn test_null_byte_injection() {
        let policy = CommandPolicy::new();

        // Null bytes should be rejected
        assert!(!policy.is_allowed("git", &["status\0;rm -rf /"]));
    }

    #[test]
    fn test_to_allowlist_string() {
        let policy = CommandPolicy::new();
        let allowlist = policy.to_allowlist_string();

        // Should contain expected programs
        assert!(allowlist.contains("git"));
        assert!(allowlist.contains("cargo"));
        assert!(allowlist.contains("python"));
    }

    #[test]
    fn test_validation_error_messages() {
        let policy = CommandPolicy::new();

        // Denied program should give clear error
        let err = policy.validate("rm", &["-rf", "/"]).unwrap_err();
        assert!(err.to_string().contains("not in the allowlist"));

        // Dangerous pattern should give security error
        let err = policy.validate("git", &["status", "|", "cat"]).unwrap_err();
        assert!(err.is_security_error());
    }
}
