//! Profile-aware tool registry for pb-mcp-server
//!
//! [`ToolRegistry`] wraps a [`ToolCatalog`] together with a [`ToolProfile`],
//! gating which tools are available based on the profile's risk level, write
//! permission, execution permission, **and profile visibility**.
//! It also supports dynamic registration and unregistration of tools at runtime.
//!
//! # Risk gating rules
//!
//! | Profile              | Risk levels                   | Writes | Exec | Profile visible to              |
//! |----------------------|-------------------------------|--------|------|---------------------------------|
//! | `PublicSafe`         | low only                      | ✗      | ✗    | public-safe                     |
//! | `CoreCode`           | low, medium                   | ✗      | ✗    | public-safe, core-code          |
//! | `ReviewRefactor`     | low, medium                   | ✗      | ✗    | + review-refactor               |
//! | `AgentOrchestration` | low, medium                   | ✓      | ✗    | + agent-orchestration           |
//! | `LocalPower`         | low, medium, high, critical   | ✓      | ✓    | + local-power                   |
//! | `ExperimentalLab`    | low, medium, high, critical   | ✓      | ✓    | all profiles                    |

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
/// the profile does not grant, or that belong to a higher-tier profile, are
/// considered *disabled* even though they remain in the underlying catalog.
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
/// // Only low/medium risk tools whose profile is public-safe or core-code
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
    pub fn new(profile: ToolProfile) -> Self {
        let catalog = ToolCatalog::new();
        Self { catalog, profile }
    }

    /// Create a registry from an existing catalog and the given profile.
    pub fn with_catalog(catalog: ToolCatalog, profile: ToolProfile) -> Self {
        Self { catalog, profile }
    }

    // -----------------------------------------------------------------------
    // Query methods
    // -----------------------------------------------------------------------

    /// Check whether a tool is available under the current profile.
    pub fn is_enabled(&self, tool_name: &str) -> bool {
        let Some(tool) = self.find_tool(tool_name) else {
            return false;
        };
        self.is_tool_enabled(tool)
    }

    /// Return all tools enabled under the current profile.
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

    /// Return enabled tools that are currently backed by placeholder/stub implementations.
    pub fn enabled_stubs(&self) -> Vec<&ToolInfo> {
        self.enabled_tools()
            .into_iter()
            .filter(|tool| tool.is_stub())
            .collect()
    }

    // -----------------------------------------------------------------------
    // Dynamic registration
    // -----------------------------------------------------------------------

    /// Register a custom tool dynamically at runtime.
    ///
    /// If a tool with the same name already exists in the catalog, this call
    /// is a **no-op** — the existing entry is preserved and no duplicate is
    /// created. To replace an existing tool, call
    /// [`unregister_tool`](Self::unregister_tool) first.
    pub fn register_tool(&mut self, info: ToolInfo) {
        // Deduplicate: skip if a tool with the same name already exists.
        if self.catalog.all().iter().any(|t| t.name == info.name) {
            return;
        }
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
    ///
    /// Gates are checked in order:
    /// 0. Profile visibility — the tool's declared profile must be reachable
    ///    from the active profile.
    /// 1. Risk level — must be within the profile's allowed risk levels.
    /// 2. Write gate — if the tool requires write access, the profile must
    ///    allow writes.
    /// 3. Exec gate — if the tool requires execution, the profile must allow
    ///    execution.
    fn is_tool_enabled(&self, tool: &ToolInfo) -> bool {
        // 0. Profile visibility gate — tool's declared profile must be
        //    reachable from the active profile. This prevents experimental-lab
        //    tools from appearing under public-safe/core-code, etc.
        if !self.is_profile_visible(&tool.profile) {
            return false;
        }

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

    /// Return the set of tool-profile strings visible to the active profile.
    ///
    /// A profile can see itself and all lower-tier profiles. The tiers are:
    ///
    /// ```text
    /// public-safe  (lowest)
    ///   core-code
    ///     review-refactor
    ///       agent-orchestration
    ///         local-power
    ///           experimental-lab  (highest)
    /// ```
    fn is_profile_visible(&self, tool_profile: &str) -> bool {
        let allowed: &[&str] = match self.profile {
            ToolProfile::PublicSafe => &["public-safe"],
            ToolProfile::CoreCode => &["public-safe", "core-code"],
            ToolProfile::ReviewRefactor => &["public-safe", "core-code", "review-refactor"],
            ToolProfile::AgentOrchestration => &[
                "public-safe",
                "core-code",
                "review-refactor",
                "agent-orchestration",
            ],
            ToolProfile::LocalPower => &[
                "public-safe",
                "core-code",
                "review-refactor",
                "agent-orchestration",
                "local-power",
            ],
            ToolProfile::ExperimentalLab => &[
                "public-safe",
                "core-code",
                "review-refactor",
                "agent-orchestration",
                "local-power",
                "experimental-lab",
            ],
        };
        allowed.contains(&tool_profile)
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
            assert!(
                !tool.requires_write,
                "CoreCode should not expose write tools, found: {}",
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

    /// Verifies that the profile visibility gate blocks tools whose declared
    /// profile is higher-tier than the active profile, even if they would
    /// otherwise pass the risk/write/exec gates.
    ///
    /// Specifically: experimental-lab tools with risk_level "low" and no
    /// write/exec requirements must NOT appear under PublicSafe or CoreCode.
    #[test]
    fn profile_gate_blocks_downlevel_tools() {
        // These profiles must NOT see experimental-lab tools.
        let restricted = [
            ToolProfile::PublicSafe,
            ToolProfile::CoreCode,
            ToolProfile::ReviewRefactor,
            ToolProfile::AgentOrchestration,
            ToolProfile::LocalPower,
        ];

        for profile in restricted {
            let registry = ToolRegistry::new(profile);
            for tool in registry.enabled_tools() {
                assert_ne!(
                    tool.profile, "experimental-lab",
                    "profile {:?} must not expose experimental-lab tool: {}",
                    profile, tool.name
                );
            }
        }

        // ExperimentalLab itself MUST see experimental-lab tools.
        let lab = ToolRegistry::new(ToolProfile::ExperimentalLab);
        let lab_tools: Vec<_> = lab
            .enabled_tools()
            .into_iter()
            .filter(|t| t.profile == "experimental-lab")
            .collect();
        assert!(
            !lab_tools.is_empty(),
            "ExperimentalLab should expose experimental-lab tools"
        );
    }

    #[test]
    fn deterministic_scaffolds_are_not_enabled_stubs() {
        let registry = ToolRegistry::new(ToolProfile::ReviewRefactor);
        let enabled_names: Vec<_> = registry
            .enabled_tools()
            .into_iter()
            .map(|tool| tool.name.as_str())
            .collect();
        assert!(enabled_names.contains(&"plan.risk_register"));
        assert!(enabled_names.contains(&"reasoning.evidence_matrix"));

        let names: Vec<_> = registry
            .enabled_stubs()
            .into_iter()
            .map(|tool| tool.name.as_str())
            .collect();
        assert!(!names.contains(&"plan.risk_register"));
        assert!(!names.contains(&"reasoning.evidence_matrix"));
    }

    /// Verifies that registering a tool with a duplicate name is a no-op:
    /// the catalog count stays the same and the original entry is preserved.
    #[test]
    fn register_tool_deduplicates() {
        let mut registry = ToolRegistry::new(ToolProfile::CoreCode);
        let before = registry.catalog().count();

        // Register a new tool.
        let tool = ToolInfo {
            name: "test.dedup_target".to_string(),
            description: "Original description".to_string(),
            category: "test".to_string(),
            profile: "core-code".to_string(),
            risk_level: "low".to_string(),
            requires_write: false,
            requires_exec: false,
        };
        registry.register_tool(tool);
        assert_eq!(registry.catalog().count(), before + 1);

        // Try to register a duplicate — count must stay the same.
        let duplicate = ToolInfo {
            name: "test.dedup_target".to_string(),
            description: "Duplicate — should be ignored".to_string(),
            category: "test".to_string(),
            profile: "core-code".to_string(),
            risk_level: "low".to_string(),
            requires_write: false,
            requires_exec: false,
        };
        registry.register_tool(duplicate);
        assert_eq!(
            registry.catalog().count(),
            before + 1,
            "duplicate registration should be a no-op"
        );

        // The original description must still be intact.
        let found = registry
            .catalog()
            .all()
            .iter()
            .find(|t| t.name == "test.dedup_target")
            .expect("tool must exist");
        assert_eq!(found.description, "Original description");
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
        for tool in &compress_tools {
            assert_eq!(tool.risk_level, "low");
        }
    }

    /// Verifies that every tool exposed by PublicSafe has profile "public-safe".
    /// This is a canary test: if any future catalog entry with a higher-tier
    /// profile leaks through the profile gate it will fail here.
    #[test]
    fn public_safe_only_exposes_public_safe_profile_tools() {
        let registry = ToolRegistry::new(ToolProfile::PublicSafe);
        for tool in registry.enabled_tools() {
            assert_eq!(
                tool.profile, "public-safe",
                "PublicSafe must only expose public-safe profile tools, found {} (profile: {})",
                tool.name, tool.profile
            );
        }
    }
}
