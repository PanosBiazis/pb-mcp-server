//! Profile-aware tool registry for pb-mcp-server
//!
//! [`ToolRegistry`] wraps a [`ToolCatalog`] together with a [`ToolProfile`],
//! gating which tools are available based on the profile's risk level, write
//! permission, and execution permission. It also supports dynamic
//! registration and unregistration of tools at runtime.
//!
//! # Risk gating rules
//!
//! | Profile | Risk levels | Writes | Exec | Notes |
//! |----------------------|-------------------------------|--------|------|-------|
//! | `CoreCode` | low, medium | ✗ | ✗ | |
//! | `ReviewRefactor` | low, medium | ✗ | ✗ | |
//! | `AgentOrchestration` | low, medium | ✗ | ✗ | Agent tools allowed |
//! | `LocalPower` | low, medium, high, critical | ✓ | ✓ | |
//! | `PublicSafe` | low only | ✗ | ✗ | No write, no exec |
//! | `ExperimentalLab` | low, medium, high, critical | ✓ | ✓ | |

use super::catalog::{ToolCatalog, ToolInfo};
use crate::profiles::ToolProfile;

// ---------------------------------------------------------------------------
// ToolRegistry
// ---------------------------------------------------------------------------

/// Profile-aware tool registry that gates tool availability.
///
/// A `ToolRegistry` pairs a [`ToolCatalog`] with a [`ToolProfile`] and
/// enforces profile-specific rules on which tools are enabled. Tools that
/// exceed the profile's risk level, or that require write/exec permissions
/// the profile does not grant, are considered *disabled* even though they
/// remain in the underlying catalog.
///
/// # Example
///
/// ```
/// use pb_mcp_server::tools::registry::ToolRegistry;
/// use pb_mcp_server::profiles::ToolProfile;
///
/// let registry = ToolRegistry::new(ToolProfile::CoreCode);
/// assert!(!registry.is_enabled("nonexistent_tool"));
/// let enabled = registry.enabled_tools();
/// // Only low and medium risk tools with no write/exec are included
/// ```
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// The complete tool catalog.
    catalog: ToolCatalog,
    /// The active profile used for gating.
    profile: ToolProfile,
}

impl ToolRegistry {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a new registry with a freshly built catalog and the given profile.
    ///
    /// The catalog is populated via [`ToolCatalog::new`] and then filtered
    /// according to the profile's risk and permission rules.
    pub fn new(profile: ToolProfile) -> Self {
        let catalog = ToolCatalog::new();
        Self { catalog, profile }
    }

    /// Create a registry from an existing catalog and the given profile.
    ///
    /// Useful when the catalog has been pre-built or customised externally.
    pub fn with_catalog(catalog: ToolCatalog, profile: ToolProfile) -> Self {
        Self { catalog, profile }
    }

    // -----------------------------------------------------------------------
    // Query methods
    // -----------------------------------------------------------------------

    /// Check whether a tool is available under the current profile.
    ///
    /// A tool is considered *enabled* when **all** of the following hold:
    ///
    /// 1. The tool exists in the catalog.
    /// 2. The tool's risk level is allowed by the profile.
    /// 3. If the tool requires write access, the profile allows writes.
    /// 4. If the tool requires execution, the profile allows execution.
    pub fn is_enabled(&self, tool_name: &str) -> bool {
        let Some(tool) = self.find_tool(tool_name) else {
            return false;
        };
        self.is_tool_enabled(tool)
    }

    /// Return all tools enabled under the current profile.
    ///
    /// The order follows the underlying catalog.
    pub fn enabled_tools(&self) -> Vec<&ToolInfo> {
        self.catalog
            .all()
            .iter()
            .filter(|tool| self.is_tool_enabled(tool))
            .collect()
    }

    /// Return enabled tools filtered by both profile **and** category.
    pub fn enabled_by_category(&self, category: &str) -> Vec<&ToolInfo> {
        self.enabled_tools()
            .into_iter()
            .filter(|tool| tool.category == category)
            .collect()
    }

    // -----------------------------------------------------------------------
    // Dynamic registration
    // -----------------------------------------------------------------------

    /// Register a custom tool dynamically at runtime.
    ///
    /// The tool is appended to the underlying catalog. If a tool with the
    /// same name already exists, it is **not** replaced — the duplicate
    /// remains in the catalog and both entries will be considered by
    /// [`is_enabled`](Self::is_enabled).
    ///
    /// To replace an existing tool, call [`unregister_tool`](Self::unregister_tool)
    /// first.
    pub fn register_tool(&mut self, info: ToolInfo) {
        // We need mutable access to the catalog's internal vec.
        // ToolCatalog::all() returns &[ToolInfo], so we expose an add method.
        self.catalog.add(info);
    }

    /// Unregister a tool by name.
    ///
    /// Removes the **first** tool whose name matches and returns `true`.
    /// If no tool with that name exists, returns `false`.
    pub fn unregister_tool(&mut self, name: &str) -> bool {
        self.catalog.remove(name)
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Return a reference to the underlying catalog.
    pub fn catalog(&self) -> &ToolCatalog {
        &self.catalog
    }

    /// Return the active profile.
    pub fn profile(&self) -> ToolProfile {
        self.profile
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Find a tool by name in the catalog.
    fn find_tool(&self, tool_name: &str) -> Option<&ToolInfo> {
        self.catalog.all().iter().find(|t| t.name == tool_name)
    }

    /// Determine whether a specific tool passes the profile's gate.
    fn is_tool_enabled(&self, tool: &ToolInfo) -> bool {
        // 1. Risk level gate
        if !self.is_risk_allowed(&tool.risk_level) {
            return false;
        }

        // 2. Write gate
        if tool.requires_write && !self.profile.allows_writes() {
            return false;
        }

        // 3. Exec gate
        if tool.requires_exec && !self.profile.allows_execution() {
            return false;
        }

        true
    }

    /// Check whether the given risk level string is permitted under the
    /// current profile.
    fn is_risk_allowed(&self, risk: &str) -> bool {
        match self.profile {
            ToolProfile::CoreCode
            | ToolProfile::ReviewRefactor
            | ToolProfile::AgentOrchestration => {
                matches!(risk, "low" | "medium")
            }
            ToolProfile::LocalPower | ToolProfile::ExperimentalLab => {
                matches!(risk, "low" | "medium" | "high" | "critical")
            }
            ToolProfile::PublicSafe => matches!(risk, "low"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A helper that builds a minimal catalog with known tools for
    /// deterministic testing, bypassing the full `ToolCatalog::new()`.
    fn make_test_catalog() -> ToolCatalog {
        // We use ToolCatalog::new() but then rely on specific known entries.
        // For fully deterministic tests we build via the public API of
        // ToolRegistry — the full catalog is fine because we know the
        // risk levels and flags of the built-in tools.
        ToolCatalog::new()
    }

    /// Verifies that `PublicSafe` only exposes low-risk, no-write, no-exec
    /// tools and that high-risk / write / exec tools are disabled.
    #[test]
    fn public_safe_gates_risk_write_and_exec() {
        let registry = ToolRegistry::new(ToolProfile::PublicSafe);

        for tool in registry.enabled_tools() {
            assert_eq!(
                tool.risk_level, "low",
                "PublicSafe should only expose low-risk tools, found: {} ({})",
                tool.name, tool.risk_level
            );
            assert!(
                !tool.requires_write,
                "PublicSafe should not expose write tools, found: {}",
                tool.name
            );
            assert!(
                !tool.requires_exec,
                "PublicSafe should not expose exec tools, found: {}",
                tool.name
            );
        }
    }

    /// Verifies that `CoreCode` allows low and medium risk but rejects
    /// high/critical risk and any tools requiring execution.
    #[test]
    fn core_code_allows_low_and_medium_only() {
        let registry = ToolRegistry::new(ToolProfile::CoreCode);
        let enabled = registry.enabled_tools();

        for tool in &enabled {
            assert!(
                matches!(tool.risk_level.as_str(), "low" | "medium"),
                "CoreCode should only expose low/medium risk, found: {} ({})",
                tool.name,
                tool.risk_level
            );
            assert!(
                !tool.requires_exec,
                "CoreCode should not expose exec tools, found: {}",
                tool.name
            );
        }

        // Additionally, `is_enabled` should return false for any
        // high-risk or critical tool in the catalog.
        for tool in registry.catalog().all() {
            if matches!(tool.risk_level.as_str(), "high" | "critical") {
                assert!(
                    !registry.is_enabled(&tool.name),
                    "CoreCode should disable high/critical tool: {}",
                    tool.name
                );
            }
        }
    }

    /// Tests dynamic registration and unregistration of tools, ensuring
    /// that newly registered tools respect the profile gate and that
    /// unregistration actually removes them.
    #[test]
    fn dynamic_register_and_unregister() {
        let mut registry = ToolRegistry::new(ToolProfile::CoreCode);

        // Register a low-risk tool — should be enabled under CoreCode.
        let low_risk_tool = ToolInfo {
            name: "test.low_risk_custom".to_string(),
            description: "A custom low-risk test tool".to_string(),
            category: "test".to_string(),
            profile: "core-code".to_string(),
            risk_level: "low".to_string(),
            requires_write: false,
            requires_exec: false,
        };
        registry.register_tool(low_risk_tool.clone());
        assert!(
            registry.is_enabled("test.low_risk_custom"),
            "Low-risk custom tool should be enabled under CoreCode"
        );

        // Register a high-risk tool — should be disabled under CoreCode.
        let high_risk_tool = ToolInfo {
            name: "test.high_risk_custom".to_string(),
            description: "A custom high-risk test tool".to_string(),
            category: "test".to_string(),
            profile: "local-power".to_string(),
            risk_level: "high".to_string(),
            requires_write: false,
            requires_exec: false,
        };
        registry.register_tool(high_risk_tool.clone());
        assert!(
            !registry.is_enabled("test.high_risk_custom"),
            "High-risk custom tool should be disabled under CoreCode"
        );

        // The high-risk tool should be enabled under LocalPower.
        let mut power_registry = ToolRegistry::new(ToolProfile::LocalPower);
        power_registry.register_tool(high_risk_tool);
        assert!(
            power_registry.is_enabled("test.high_risk_custom"),
            "High-risk custom tool should be enabled under LocalPower"
        );

        // Unregister the low-risk tool from the CoreCode registry.
        assert!(
            registry.unregister_tool("test.low_risk_custom"),
            "Should successfully unregister existing tool"
        );
        assert!(
            !registry.is_enabled("test.low_risk_custom"),
            "Unregistered tool should no longer be enabled"
        );

        // Unregistering a non-existent tool should return false.
        assert!(
            !registry.unregister_tool("nonexistent_tool"),
            "Unregistering a nonexistent tool should return false"
        );
    }

    /// Verify that `enabled_by_category` returns only tools matching both
    /// the profile gate and the requested category.
    #[test]
    fn enabled_by_category_respects_profile_and_category() {
        let registry = ToolRegistry::new(ToolProfile::PublicSafe);
        let health_tools = registry.enabled_by_category("health");

        for tool in &health_tools {
            assert_eq!(
                tool.category, "health",
                "enabled_by_category should only return tools in the requested category"
            );
            assert_eq!(
                tool.risk_level, "low",
                "PublicSafe should only expose low-risk tools"
            );
        }

        // A category that has no low-risk tools under PublicSafe should
        // return an empty vec.
        let compress_tools = registry.enabled_by_category("compress");
        // compress tools are typically medium+ risk, so under PublicSafe
        // the list should be empty or only contain low-risk entries.
        for tool in &compress_tools {
            assert_eq!(tool.risk_level, "low");
        }
    }
}
