//! # pb-mcp-server
//!
//! Profile-based IDE agent MCP server for Zed/Cursor with multi-agent bus,
//! code intelligence, and execution tools.
//!
//! ## Features
//!
//! - **Profile-based tool exposure**: Different tool sets for different workflows
//! - **Multi-agent coordination**: Durable append-only message bus
//! - **Code intelligence**: Symbol indexing, search, and analysis
//! - **Safe execution**: Sandboxed terminal operations with allowlists
//! - **Review tools**: PR review, architecture analysis, risk assessment
//!
//! ## Profiles
//!
//! | Profile | Purpose |
//! |---------|---------|
//! | `core-code` | Default coding workflow in Zed/Cursor |
//! | `review-refactor` | PR review, architecture understanding |
//! | `agent-orchestration` | Multi-agent and subagent coordination |
//! | `local-power` | Explicitly approved local execution |
//! | `public-safe` | Remote-safe HTTP subset only |
//! | `experimental-lab` | Advanced OCR/ML/AI/algorithm tools |
//!
//! ## Example
//!
//! ```no_run
//! use pb_mcp_server::server::MintMcp;
//! use pb_mcp_server::profiles::ToolProfile;
//!
//! let profile = ToolProfile::CoreCode;
//! let server = MintMcp::new(profile);
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ============================================================================
// Module declarations
// ============================================================================

/// Argument structures for MCP tool handlers
pub mod args;

/// Multi-agent message bus (JSONL append-only)
pub mod bus;

/// Rate limiting for agent bus
pub mod bus_rate;

/// Code symbol indexing and search
pub mod code_index;

/// Network security profiles
pub mod network_profile;

/// Core operation implementations
pub mod ops;

/// Planning and reasoning scaffolding tools
pub mod thinking;

/// Profile-based tool registration
pub mod profiles;

/// Security utilities (path validation, command policy)
pub mod security;

/// MCP server implementation
pub mod server;

/// Caching layer for performance optimization
pub mod cache;
/// Error types
pub mod error;
/// Health monitoring and diagnostics
pub mod health;
/// Performance metrics and monitoring
pub mod metrics;
/// Output formatting utilities
pub mod output;
/// Resilience patterns (circuit breaker, retry, bulkhead)
pub mod resilience;
/// Tool handlers organized by category
pub mod tools;

// ============================================================================
// Re-exports for convenience
// ============================================================================

pub use args::*;
pub use bus::{AgentMessage, MAX_AGENT_MESSAGE_BODY_CHARS};
pub use error::{Error, Result};
pub use ops::MAX_RESPONSE_CHARS;
pub use profiles::ToolProfile;
// pub use server::MintMcp; // TODO: Implement server module

/// Current version of the server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported Rust version
pub const RUST_VERSION: &str = "1.75.0";

/// Default bus directory name
pub const BUS_DIR: &str = ".mint-mcp";

/// Default maximum file read size in bytes
pub const DEFAULT_MAX_FILE_READ_BYTES: u64 = 512 * 1024;

/// Default maximum response characters
pub const DEFAULT_MAX_RESPONSE_CHARS: usize = 20_000;

/// Cache time-to-live in seconds (5 minutes)
pub const DEFAULT_CACHE_TTL_SECS: u64 = 300;

/// Maximum cache entries
pub const MAX_CACHE_ENTRIES: usize = 1000;

/// Health check interval in seconds
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 60;

/// Metrics collection interval in seconds
pub const METRICS_INTERVAL_SECS: u64 = 10;

// ============================================================================
// Feature-gated modules
// ============================================================================

#[cfg(feature = "ocr-tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "ocr-tools")))]
pub mod ocr;

#[cfg(feature = "screenshot-tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "screenshot-tools")))]
pub mod screenshot;

#[cfg(feature = "ml-tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "ml-tools")))]
pub mod ml;

#[cfg(feature = "terminal-exec")]
#[cfg_attr(docsrs, doc(cfg(feature = "terminal-exec")))]
pub mod terminal;

// ============================================================================
// Prelude module for common imports
// ============================================================================

pub mod prelude {
    //! Common imports for using pb-mcp-server
    //!
    //! This module re-exports the most commonly used items from the crate.
    pub use crate::cache::FileCache;
    pub use crate::error::{Error, Result};
    pub use crate::health::HealthStatus;
    pub use crate::metrics::MetricsCollector;
    pub use crate::ops::MAX_RESPONSE_CHARS;
    pub use crate::profiles::ToolProfile;
    pub use crate::resilience::{CircuitBreaker, CircuitBreakerConfig};
    // pub use crate::server::MintMcp; // TODO: Implement server module
    pub use schemars::JsonSchema;
    pub use serde::{Deserialize, Serialize};
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_valid() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(parts.len() >= 2, "Version should have at least major.minor");
    }

    #[test]
    fn bus_dir_is_valid() {
        assert!(!BUS_DIR.is_empty());
        assert!(!BUS_DIR.contains('/'));
        assert!(!BUS_DIR.contains('\\'));
    }
}
