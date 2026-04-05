//! Error handling module for pb-mcp-server
//!
//! This module provides a comprehensive error type hierarchy for the server,
//! enabling detailed error context and proper error propagation throughout
//! the codebase.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for pb-mcp-server operations
#[derive(Debug, Error)]
pub enum Error {
    /// File system operation failed
    #[error("File system error for '{path}': {source}")]
    FileSystem {
        /// Path that caused the error
        path: PathBuf,
        /// Underlying IO error
        source: std::io::Error,
    },

    /// Path validation failed (escape attempt, invalid path, etc.)
    #[error("Path validation failed: {reason}")]
    PathValidation {
        /// Human-readable reason for the failure
        reason: String,
        /// The problematic path (if available)
        path: Option<PathBuf>,
    },

    /// Path escapes the project root
    #[error("Path '{path}' escapes project root '{root}'")]
    PathEscape {
        /// The path that attempted to escape
        path: PathBuf,
        /// The project root
        root: PathBuf,
    },

    /// File too large to process
    #[error("File '{path}' exceeds size limit: {size} bytes (max: {max} bytes)")]
    FileTooLarge {
        /// Path to the oversized file
        path: PathBuf,
        /// Actual file size
        size: u64,
        /// Maximum allowed size
        max: u64,
    },

    /// Response too large
    #[error("Response exceeds character limit: {size} chars (max: {max} chars)")]
    ResponseTooLarge {
        /// Actual size
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Git operation failed
    #[error("Git operation failed: {message}")]
    Git {
        /// Error message from git
        message: String,
        /// Git command that failed (if available)
        command: Option<String>,
    },

    /// Terminal command execution failed
    #[error("Terminal execution failed for '{command}': {reason}")]
    TerminalExecution {
        /// The command that was attempted
        command: String,
        /// Reason for failure
        reason: String,
    },

    /// Command not in allowlist
    #[error("Command '{command}' is not in the allowlist")]
    CommandNotAllowed {
        /// The blocked command
        command: String,
        /// The allowlist (truncated if too long)
        allowlist: Vec<String>,
    },

    /// Bus operation failed
    #[error("Agent bus error: {reason}")]
    Bus {
        /// Human-readable error reason
        reason: String,
    },

    /// Bus file lock failed
    #[error("Failed to acquire lock for bus file: {path}")]
    BusLockError {
        /// Path to the lock file
        path: PathBuf,
    },

    /// Message too large for bus
    #[error("Message body exceeds limit: {size} chars (max: {max} chars)")]
    MessageTooLarge {
        /// Actual message size
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {operation} (limit: {limit} per {window_secs}s)")]
    RateLimitExceeded {
        /// Operation that was rate limited
        operation: String,
        /// Maximum allowed operations
        limit: usize,
        /// Window duration in seconds
        window_secs: u64,
    },

    /// JSON parsing/validation failed
    #[error("JSON error{}: {reason}", path.as_ref().map(|p| format!(" for '{}'", p.display())).unwrap_or_default())]
    JsonError {
        /// Path to the JSON file (if applicable)
        path: Option<PathBuf>,

        /// Error description
        reason: String,
    },

    /// YAML parsing failed
    #[error("YAML error: {reason}")]
    YamlError {
        /// Error description
        reason: String,
    },

    /// TOML parsing failed
    #[error("TOML error: {reason}")]
    TomlError {
        /// Error description
        reason: String,
    },

    /// Invalid tool arguments
    #[error("Invalid arguments for tool '{tool}': {reason}")]
    InvalidArguments {
        /// Tool name
        tool: String,
        /// Error description
        reason: String,
    },

    /// Tool not found or not enabled
    #[error("Tool '{tool}' not found or not enabled in current profile")]
    ToolNotFound {
        /// Tool name
        tool: String,
    },

    /// Security violation
    #[error("Security violation: {reason}")]
    SecurityViolation {
        /// Description of the violation
        reason: String,
    },

    /// Operation not permitted in current profile
    #[error("Operation '{operation}' not permitted in '{profile}' profile")]
    OperationNotPermitted {
        /// The operation attempted
        operation: String,
        /// Current security profile
        profile: String,
    },

    /// Network profile error
    #[error("Network profile error: {reason}")]
    NetworkProfile {
        /// Error description
        reason: String,
    },

    /// OCR processing error
    #[cfg(feature = "ocr-tools")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ocr-tools")))]
    #[error("OCR error: {reason}")]
    Ocr {
        /// Error description
        reason: String,
    },

    /// Image processing error
    #[cfg(feature = "screenshot-tools")]
    #[cfg_attr(docsrs, doc(cfg(feature = "screenshot-tools")))]
    #[error("Image processing error: {reason}")]
    Image {
        /// Error description
        reason: String,
    },

    /// ML model error
    #[cfg(feature = "ml-tools")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ml-tools")))]
    #[error("ML model error: {reason}")]
    Model {
        /// Error description
        reason: String,
    },

    /// Timeout exceeded
    #[error("Operation timed out after {timeout_ms}ms: {operation}")]
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
        /// Operation that timed out
        operation: String,
    },

    /// Compression/decompression error
    #[error("Compression error: {reason}")]
    Compression {
        /// Error description
        reason: String,
    },

    /// Code index error
    #[error("Code index error: {reason}")]
    CodeIndex {
        /// Error description
        reason: String,
    },

    /// Generic error with message
    #[error("{0}")]
    Generic(String),

    /// Multiple errors occurred
    #[error("Multiple errors occurred:\n{0}")]
    Multiple(String),
}

/// Result type alias for pb-mcp-server operations
pub type Result<T> = std::result::Result<T, Error>;

// ============================================================================
// Helper constructors
// ============================================================================

impl Error {
    /// Create a file system error with path context
    pub fn file_system(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::FileSystem {
            path: path.into(),
            source,
        }
    }

    /// Create a path validation error
    pub fn path_validation(reason: impl Into<String>) -> Self {
        Error::PathValidation {
            reason: reason.into(),
            path: None,
        }
    }

    /// Create a path validation error with path context
    pub fn path_validation_with_path(reason: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Error::PathValidation {
            reason: reason.into(),
            path: Some(path.into()),
        }
    }

    /// Create a path escape error
    pub fn path_escape(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Self {
        Error::PathEscape {
            path: path.into(),
            root: root.into(),
        }
    }

    /// Create a file too large error
    pub fn file_too_large(path: impl Into<PathBuf>, size: u64, max: u64) -> Self {
        Error::FileTooLarge {
            path: path.into(),
            size,
            max,
        }
    }

    /// Create a git error
    pub fn git(message: impl Into<String>) -> Self {
        Error::Git {
            message: message.into(),
            command: None,
        }
    }

    /// Create a git error with command context
    pub fn git_with_command(message: impl Into<String>, command: impl Into<String>) -> Self {
        Error::Git {
            message: message.into(),
            command: Some(command.into()),
        }
    }

    /// Create a terminal execution error
    pub fn terminal_execution(command: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::TerminalExecution {
            command: command.into(),
            reason: reason.into(),
        }
    }

    /// Create a command not allowed error
    pub fn command_not_allowed(command: impl Into<String>, allowlist: Vec<String>) -> Self {
        Error::CommandNotAllowed {
            command: command.into(),
            allowlist,
        }
    }

    /// Create a bus error
    pub fn bus(reason: impl Into<String>) -> Self {
        Error::Bus {
            reason: reason.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(
        operation: impl Into<String>,
        limit: usize,
        window_secs: u64,
    ) -> Self {
        Error::RateLimitExceeded {
            operation: operation.into(),
            limit,
            window_secs,
        }
    }

    /// Create a JSON error
    pub fn json(reason: impl Into<String>) -> Self {
        Error::JsonError {
            path: None,
            reason: reason.into(),
        }
    }

    /// Create a JSON error with path context
    pub fn json_with_path(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Error::JsonError {
            path: Some(path.into()),
            reason: reason.into(),
        }
    }

    /// Create invalid arguments error
    pub fn invalid_arguments(tool: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::InvalidArguments {
            tool: tool.into(),
            reason: reason.into(),
        }
    }

    /// Create a security violation error
    pub fn security_violation(reason: impl Into<String>) -> Self {
        Error::SecurityViolation {
            reason: reason.into(),
        }
    }

    /// Create an operation not permitted error
    pub fn operation_not_permitted(
        operation: impl Into<String>,
        profile: impl Into<String>,
    ) -> Self {
        Error::OperationNotPermitted {
            operation: operation.into(),
            profile: profile.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(timeout_ms: u64, operation: impl Into<String>) -> Self {
        Error::Timeout {
            timeout_ms,
            operation: operation.into(),
        }
    }

    /// Create a compression error
    pub fn compression(reason: impl Into<String>) -> Self {
        Error::Compression {
            reason: reason.into(),
        }
    }

    /// Combine multiple errors into one
    pub fn combine(errors: Vec<Error>) -> Self {
        if errors.is_empty() {
            Error::Generic("No errors to combine".to_string())
        } else if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            let messages: Vec<String> = errors.iter().map(|e| format!("  - {}", e)).collect();
            Error::Multiple(messages.join("\n"))
        }
    }

    /// Check if this error is a security-related error
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            Error::PathValidation { .. }
                | Error::PathEscape { .. }
                | Error::CommandNotAllowed { .. }
                | Error::SecurityViolation { .. }
                | Error::OperationNotPermitted { .. }
        )
    }

    /// Check if this error is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::FileTooLarge { .. }
                | Error::ResponseTooLarge { .. }
                | Error::RateLimitExceeded { .. }
                | Error::Timeout { .. }
                | Error::MessageTooLarge { .. }
        )
    }

    /// Get a user-friendly error message for MCP responses
    pub fn to_mcp_message(&self) -> String {
        match self {
            Error::FileSystem { path, .. } => format!("File system error: {}", path.display()),
            Error::PathValidation { reason, .. } => format!("Invalid path: {}", reason),
            Error::PathEscape { path, .. } => {
                format!("Path escapes project root: {}", path.display())
            }
            Error::FileTooLarge { path, size, max } => {
                format!(
                    "File {} is too large ({} bytes, max {} bytes)",
                    path.display(),
                    size,
                    max
                )
            }
            Error::ResponseTooLarge { size, max } => {
                format!("Response too large ({} chars, max {} chars)", size, max)
            }
            Error::Git { message, .. } => format!("Git error: {}", message),
            Error::TerminalExecution { command, reason } => {
                format!("Failed to execute '{}': {}", command, reason)
            }
            Error::CommandNotAllowed { command, .. } => {
                format!("Command '{}' is not allowed", command)
            }
            Error::Bus { reason } => format!("Bus error: {}", reason),
            Error::BusLockError { path } => format!("Failed to lock bus file: {}", path.display()),
            Error::MessageTooLarge { size, max } => {
                format!("Message too large ({} chars, max {} chars)", size, max)
            }
            Error::RateLimitExceeded {
                operation,
                limit,
                window_secs,
            } => {
                format!(
                    "Rate limit exceeded for {} (max {} per {}s)",
                    operation, limit, window_secs
                )
            }
            Error::JsonError { path, reason } => match path {
                Some(p) => format!("JSON error in {}: {}", p.display(), reason),
                None => format!("JSON error: {}", reason),
            },
            Error::YamlError { reason } => format!("YAML error: {}", reason),
            Error::TomlError { reason } => format!("TOML error: {}", reason),
            Error::InvalidArguments { tool, reason } => {
                format!("Invalid arguments for {}: {}", tool, reason)
            }
            Error::ToolNotFound { tool } => format!("Tool '{}' not found or not enabled", tool),
            Error::SecurityViolation { reason } => format!("Security violation: {}", reason),
            Error::OperationNotPermitted { operation, profile } => {
                format!("'{}' not permitted in '{}' profile", operation, profile)
            }
            Error::NetworkProfile { reason } => format!("Network profile error: {}", reason),
            Error::Timeout {
                timeout_ms,
                operation,
            } => {
                format!("{} timed out after {}ms", operation, timeout_ms)
            }
            Error::Compression { reason } => format!("Compression error: {}", reason),
            Error::CodeIndex { reason } => format!("Code index error: {}", reason),
            Error::Generic(msg) => msg.clone(),
            Error::Multiple(msgs) => msgs.clone(),
            #[cfg(feature = "ocr-tools")]
            Error::Ocr { reason } => format!("OCR error: {}", reason),
            #[cfg(feature = "screenshot-tools")]
            Error::Image { reason } => format!("Image error: {}", reason),
            #[cfg(feature = "ml-tools")]
            Error::Model { reason } => format!("ML model error: {}", reason),
        }
    }
}

// ============================================================================
// From implementations for automatic conversion
// ============================================================================

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Generic(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::json(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::YamlError {
            reason: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlError {
            reason: err.to_string(),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Generic(format!("Regex error: {}", err))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Generic(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Generic(s.to_string())
    }
}

#[cfg(feature = "ocr-tools")]
impl From<tesseract::TesseractError> for Error {
    fn from(err: tesseract::TesseractError) -> Self {
        Error::Ocr {
            reason: err.to_string(),
        }
    }
}

#[cfg(feature = "screenshot-tools")]
impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::Image {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::path_validation("test path");
        let msg = err.to_string();
        assert!(msg.contains("Path validation failed"));
        assert!(msg.contains("test path"));
    }

    #[test]
    fn test_error_is_security() {
        assert!(Error::security_violation("test").is_security_error());
        assert!(Error::path_escape(PathBuf::new(), PathBuf::new()).is_security_error());
        assert!(!Error::git("test").is_security_error());
    }

    #[test]
    fn test_error_is_recoverable() {
        assert!(Error::timeout(100, "test").is_recoverable());
        assert!(Error::rate_limit_exceeded("test", 10, 60).is_recoverable());
        assert!(!Error::security_violation("test").is_recoverable());
    }

    #[test]
    fn test_error_combine_empty() {
        let err = Error::combine(vec![]);
        assert!(matches!(err, Error::Generic(_)));
    }

    #[test]
    fn test_error_combine_single() {
        let err = Error::combine(vec![Error::git("test")]);
        assert!(matches!(err, Error::Git { .. }));
    }

    #[test]
    fn test_error_combine_multiple() {
        let err = Error::combine(vec![Error::git("error1"), Error::bus("error2")]);
        assert!(matches!(err, Error::Multiple(_)));
    }

    #[test]
    fn test_mcp_message_format() {
        let err = Error::file_too_large(PathBuf::from("/test/file.txt"), 1000, 500);
        let msg = err.to_mcp_message();
        assert!(msg.contains("/test/file.txt"));
        assert!(msg.contains("1000"));
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Generic(_)));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::JsonError { .. }));
    }
}
