//! MCP server entry point — MintMcp handle, build_router, run_server.
//!
//! This module is the single wiring point between the profile system,
//! the tool registry, and the MCP protocol layer. It:
//!
//! 1. Reads `PB_MCP_PROFILE` from the environment (falls back to `CoreCode`).
//! 2. Builds a [`ToolRegistry`] from the full [`ToolCatalog`] + chosen profile.
//! 3. Logs four diagnostic counts so misconfiguration is immediately visible:
//!    - total tools in catalog
//!    - total tools enabled after profile gating
//!    - profile name in use
//! 4. Exposes [`build_router`] / [`run_server`] as the stable API surface.

use crate::profiles::ToolProfile;
use crate::tools::registry::ToolRegistry;

// ---------------------------------------------------------------------------
// MintMcp
// ---------------------------------------------------------------------------

/// Server configuration for the MCP server.
pub struct MintMcp {
    /// Tool profile configuration
    pub profile: ToolProfile,
    /// Tool registry for managing available tools
    pub registry: ToolRegistry,
}

impl MintMcp {
    /// Build a new server handle for the given profile.
    ///
    /// Prints a startup diagnostic to stderr so you can always verify
    /// which profile is active and exactly how many tools are exposed.
    pub fn new(profile: ToolProfile) -> Self {
        let registry = ToolRegistry::new(profile);

        let catalog_count = registry.catalog().count();
        let enabled_count = registry.enabled_tools().len();

        eprintln!(
            "[pb-mcp-server] profile={profile}  catalog={catalog_count}  enabled={enabled_count}",
            profile = profile.as_str(),
            catalog_count = catalog_count,
            enabled_count = enabled_count,
        );

        Self { profile, registry }
    }

    /// Build the server handle from the `PB_MCP_PROFILE` environment variable.
    ///
    /// Falls back to [`ToolProfile::CoreCode`] when the variable is absent or
    /// contains an unrecognised value.
    pub fn from_env() -> Self {
        let profile = match std::env::var("PB_MCP_PROFILE") {
            Ok(raw) => match ToolProfile::from_str(&raw) {
                Some(p) => p,
                None => {
                    // Surface misconfiguration instead of silently falling back,
                    // otherwise a typo in PB_MCP_PROFILE would quietly expose the
                    // wrong (default) tool surface.
                    eprintln!(
                        "[pb-mcp-server] WARN: unrecognised PB_MCP_PROFILE={raw:?}; \
                         falling back to default profile ({})",
                        ToolProfile::default().as_str()
                    );
                    ToolProfile::default()
                }
            },
            Err(_) => ToolProfile::default(),
        };
        Self::new(profile)
    }

    /// Return the list of tool names to advertise in MCP `tools/list`.
    ///
    /// This is the **only** source of truth for what Zed / Cursor / other
    /// MCP clients will see.  Never call `catalog.all()` directly in the
    /// protocol handler — always go through this method.
    pub fn tool_names(&self) -> Vec<String> {
        self.registry
            .enabled_tools()
            .into_iter()
            .map(|t| t.name.clone())
            .collect()
    }

    /// Convenience: check whether a single tool is exposed.
    #[inline]
    pub fn is_tool_enabled(&self, name: &str) -> bool {
        self.registry.is_enabled(name)
    }
}

// ---------------------------------------------------------------------------
// Free-function API (used by main.rs)
// ---------------------------------------------------------------------------

/// Build the MCP tool router from the active profile.
///
/// Returns the enabled tool name list; the real rmcp `Router` construction
/// happens in `main.rs` where the concrete handler types are in scope.
pub fn build_router(server: &MintMcp) -> Vec<String> {
    server.tool_names()
}

/// Construct a [`MintMcp`] handle from `PB_MCP_PROFILE` and return it.
///
/// Equivalent to [`MintMcp::from_env`]; provided as a free function so
/// `main.rs` can call `let server = run_server();` without importing the type.
pub fn run_server() -> MintMcp {
    MintMcp::from_env()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_core_code_exposes_fs_read_file() {
        let s = MintMcp::new(ToolProfile::CoreCode);
        assert!(s.is_tool_enabled("fs.read_file"));
        assert!(!s.is_tool_enabled("local.exec.run"));
    }

    #[test]
    fn server_experimental_lab_exposes_write_tools() {
        let s = MintMcp::new(ToolProfile::ExperimentalLab);
        assert!(s.is_tool_enabled("git.write.commit"));
        assert!(s.is_tool_enabled("git.write.push"));
    }

    #[test]
    fn tool_names_are_unique() {
        let s = MintMcp::new(ToolProfile::ExperimentalLab);
        let names = s.tool_names();
        let deduped: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(
            names.len(),
            deduped.len(),
            "tool_names() must not contain duplicates"
        );
    }

    #[test]
    fn build_router_matches_tool_names() {
        let s = MintMcp::new(ToolProfile::CoreCode);
        assert_eq!(build_router(&s), s.tool_names());
    }
}
