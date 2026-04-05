//! Profile-based tool registration system for pb-mcp-server
//!
//! This module implements a sophisticated profile system that exposes different
//! tool sets based on workflow and security requirements. Profiles enable:
//!
//! - **Curated tool discovery**: Fewer tools visible per profile, better agent selection
//! - **Security gating**: Dangerous tools only in approved profiles
//! - **Workflow optimization**: Right tools for the right task
//!
//! # Profiles
//!
//! | Profile | Purpose | Risk Level |
//! |---------|---------|------------|
//! | `core-code` | Default coding workflow | Low (read-only) |
//! | `review-refactor` | PR review, architecture | Low (read-only) |
//! | `agent-orchestration` | Multi-agent coordination | Medium |
//! | `local-power` | Local execution, OCR, screenshots | High (requires approval) |
//! | `public-safe` | Remote HTTP subset | Very Low (bounded) |
//! | `experimental-lab` | Advanced ML/AI tools | Variable |

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tool profile defining which tools are exposed to MCP clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProfile {
    /// Default coding workflow in Zed/Cursor
    ///
    /// **Exposed tools:**
    /// - File: `fs.read_file`, `fs.read_tail`, `fs.list`, `fs.tree`, `fs.find`, `fs.info`
    /// - Git read: `git.read.status`, `git.read.diff`, `git.read.log`, `git.read.show`
    /// - Code: `code.index.search`, `code.analyze.inventory`
    /// - Text: `text.transform`, `text.summarize`, `text.stats`
    /// - Context: `pack_context`, `workspace_context`
    #[serde(rename = "core-code")]
    CoreCode,

    /// PR review, architecture understanding, safe codebase inspection
    ///
    /// **Exposed tools (extends core-code):**
    /// - Git: `git.read.blame`, `git.read.branches`, `git.diff_snippets`
    /// - Code: `code.analyze.largest_files`, `code.analyze.duplicate_basenames`
    /// - Review: `review.diff_risk`, `review.focused_review`, `review.missing_tests`
    /// - Reasoning: `reasoning.evidence_matrix`, `plan.risk_register`
    #[serde(rename = "review-refactor")]
    ReviewRefactor,

    /// Multi-agent and subagent coordination
    ///
    /// **Exposed tools:**
    /// - Bus: `agent.bus.send`, `agent.bus.inbox`, `agent.bus.thread`, `agent.bus.recent`
    /// - Coordination: `agent.ask`, `agent.handoff`, `agent.consensus`
    /// - Temp: `agent.temp.write`, `agent.temp.read`, `agent.temp.list`, `agent.temp.delete`
    /// - Wait: `agent.wait.sleep`, `agent.wait.for_path`, `agent.wait.until`
    #[serde(rename = "agent-orchestration")]
    AgentOrchestration,

    /// Explicitly approved local execution and validation
    ///
    /// **Exposed tools (requires explicit enable):**
    /// - Terminal: `local.exec.run`, `local.exec.run_stdin`
    /// - Tests: `local.test.rust_test`, `local.test.rust_check`, `local.test.pytest`
    /// - Git write: `git.write.commit`, `git.write.push`, `git.write.checkout`
    /// - OCR: `local.ocr.tesseract`
    /// - Screenshot: `local.screenshot.capture`
    ///
    /// **Security:** Only available when `MINT_NETWORK_PROFILE=local`
    #[serde(rename = "local-power")]
    LocalPower,

    /// Remote-safe HTTP subset for public/external access
    ///
    /// **Exposed tools (bounded, read-only):**
    /// - Health: `health`, `status`, `security_status`
    /// - Info: `project_info`
    /// - Files: bounded `list_files`, bounded `read_file`
    /// - Search: bounded `search_code`
    /// - Bus: `agent.bus.recent`, `agent.bus.thread` (read-only)
    ///
    /// **Security:** Only available via HTTP gateway with authentication
    #[serde(rename = "public-safe")]
    PublicSafe,

    /// Advanced OCR/ML/AI/algorithm tools (optional)
    ///
    /// **Exposed tools:**
    /// - OCR: `ocr.tesseract_file`, `ocr.tesseract_stream`
    /// - ML: `ml.embeddings`, `ml.classify`, `ml.detect`
    /// - Analysis: `algorithm.complexity`, `algorithm.dependencies`
    ///
    /// **Security:** Only available when `experimental-lab` feature is enabled
    #[serde(rename = "experimental-lab")]
    ExperimentalLab,
}

impl Default for ToolProfile {
    fn default() -> Self {
        ToolProfile::CoreCode
    }
}

impl ToolProfile {
    /// Get all available profiles
    pub fn all() -> Vec<ToolProfile> {
        vec![
            ToolProfile::CoreCode,
            ToolProfile::ReviewRefactor,
            ToolProfile::AgentOrchestration,
            ToolProfile::LocalPower,
            ToolProfile::PublicSafe,
            ToolProfile::ExperimentalLab,
        ]
    }

    /// Get profile name as kebab-case string
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolProfile::CoreCode => "core-code",
            ToolProfile::ReviewRefactor => "review-refactor",
            ToolProfile::AgentOrchestration => "agent-orchestration",
            ToolProfile::LocalPower => "local-power",
            ToolProfile::PublicSafe => "public-safe",
            ToolProfile::ExperimentalLab => "experimental-lab",
        }
    }

    /// Parse profile from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "core-code" | "core_code" | "core" => Some(ToolProfile::CoreCode),
            "review-refactor" | "review_refactor" | "review" => Some(ToolProfile::ReviewRefactor),
            "agent-orchestration" | "agent_orchestration" | "agent" | "orchestration" => {
                Some(ToolProfile::AgentOrchestration)
            }
            "local-power" | "local_power" | "local" => Some(ToolProfile::LocalPower),
            "public-safe" | "public_safe" | "public" => Some(ToolProfile::PublicSafe),
            "experimental-lab" | "experimental_lab" | "experimental" | "lab" => {
                Some(ToolProfile::ExperimentalLab)
            }
            _ => None,
        }
    }

    /// Check if this profile allows write operations
    pub fn allows_writes(&self) -> bool {
        matches!(
            self,
            ToolProfile::LocalPower | ToolProfile::AgentOrchestration
        )
    }

    /// Check if this profile allows terminal execution
    pub fn allows_execution(&self) -> bool {
        matches!(self, ToolProfile::LocalPower)
    }

    /// Check if this profile is safe for public/remote access
    pub fn is_public_safe(&self) -> bool {
        matches!(self, ToolProfile::PublicSafe)
    }

    /// Get the risk level of this profile (1-5, higher = more dangerous)
    pub fn risk_level(&self) -> u8 {
        match self {
            ToolProfile::PublicSafe => 1,
            ToolProfile::CoreCode => 2,
            ToolProfile::ReviewRefactor => 2,
            ToolProfile::AgentOrchestration => 3,
            ToolProfile::LocalPower => 4,
            ToolProfile::ExperimentalLab => 5,
        }
    }

    /// Get description of this profile
    pub fn description(&self) -> &'static str {
        match self {
            ToolProfile::CoreCode => {
                "Default coding workflow with read-only file, git, and code tools"
            }
            ToolProfile::ReviewRefactor => {
                "PR review and architecture understanding with analysis tools"
            }
            ToolProfile::AgentOrchestration => {
                "Multi-agent coordination with message bus and handoff tools"
            }
            ToolProfile::LocalPower => {
                "Local execution with terminal, git write, OCR, and screenshots"
            }
            ToolProfile::PublicSafe => "Remote-safe read-only subset for HTTP gateway access",
            ToolProfile::ExperimentalLab => "Advanced OCR/ML/AI tools for experimental workflows",
        }
    }

    /// Get all tools exposed by this profile
    pub fn tools(&self) -> Vec<&'static str> {
        match self {
            ToolProfile::CoreCode => vec![
                // File system (read-only)
                "fs.read_file",
                "fs.read_tail",
                "fs.list",
                "fs.tree",
                "fs.find",
                "fs.info",
                "fs.read_json",
                // Git (read-only)
                "git.read.status",
                "git.read.diff",
                "git.read.log",
                "git.read.show",
                "git.read.branches",
                // Code analysis
                "code.index.search",
                "code.index.paths_prefix",
                "code.index.symbol_prefix",
                "code.analyze.inventory",
                // Text processing
                "text.transform",
                "text.summarize",
                "text.stats",
                "text.extract_snippets",
                // Context
                "pack_context",
                "workspace_context",
            ],
            ToolProfile::ReviewRefactor => {
                let mut tools = Self::CoreCode.tools();
                tools.extend(vec![
                    // Git (additional read)
                    "git.read.blame",
                    "git.read.tags",
                    "git.read.reflog",
                    "git.read.shortlog",
                    "git.diff_snippets",
                    // Code analysis (additional)
                    "code.analyze.largest_files",
                    "code.analyze.duplicate_basenames",
                    "code.analyze.deps",
                    "code.analyze.word_index",
                    "code.analyze.shallow_hash",
                    // Review tools
                    "review.diff_risk",
                    "review.focused_review",
                    "review.missing_tests",
                    "review.pr_packet",
                    "review.rulebook_check",
                    // Summarization (advanced)
                    "summarize.multi_file",
                    "summarize.file",
                    // Reasoning
                    "reasoning.evidence_matrix",
                    "reasoning.devils_advocate",
                    "reasoning.premortem",
                    // Planning
                    "plan.risk_register",
                    "plan.definition_of_done",
                    "plan.test_strategy",
                ]);
                tools
            }
            ToolProfile::AgentOrchestration => vec![
                // Bus communication
                "agent.bus.send",
                "agent.bus.inbox",
                "agent.bus.thread",
                "agent.bus.recent",
                "agent.bus.search",
                "agent.bus.status",
                // Coordination
                "agent.ask",
                "agent.handoff",
                "agent.consensus",
                "agent.subagent_task",
                "agent.subagent_done",
                "agent.roster",
                "agent.list_topics",
                // Temp storage
                "agent.temp.write",
                "agent.temp.read",
                "agent.temp.list",
                "agent.temp.delete",
                // Waiting/synchronization
                "agent.wait.sleep",
                "agent.wait.for_path",
                "agent.wait.until",
                // Planning
                "plan.handoff_brief",
                "plan.work_buckets",
                "plan.execution_checklist",
            ],
            ToolProfile::LocalPower => vec![
                // Terminal execution
                "local.exec.run",
                "local.exec.run_stdin",
                "local.exec.which",
                "local.exec.program_version",
                // Testing
                "local.test.rust_test",
                "local.test.rust_check",
                "local.test.pytest",
                // Git (write operations)
                "git.write.checkout",
                "git.write.commit",
                "git.write.push",
                "git.write.pull",
                "git.write.fetch",
                "git.write.stash",
                "git.write.branch",
                // OCR
                "local.ocr.tesseract",
                "local.ocr.tesseract_file",
                // Screenshot
                "local.screenshot.capture",
                // Converters
                "convert.format",
                "convert.gzip_compress",
                "convert.gzip_decompress",
            ],
            ToolProfile::PublicSafe => vec![
                // Health/status
                "health",
                "status",
                "security_status",
                "project_info",
                // Files (bounded)
                "fs.list_bounded",
                "fs.read_file_bounded",
                "fs.search_code_bounded",
                // Bus (read-only)
                "agent.bus.recent",
                "agent.bus.thread",
            ],
            ToolProfile::ExperimentalLab => {
                let mut tools = Self::LocalPower.tools();
                tools.extend(vec![
                    // OCR (advanced)
                    "ocr.tesseract_stream",
                    "ocr.tesseract_batch",
                    // ML/AI
                    "ml.embed",
                    "ml.classify",
                    "ml.detect",
                    "ml.segment",
                    "ml.transcribe",
                    // Algorithms
                    "algorithm.complexity",
                    "algorithm.dependencies",
                    "algorithm.call_graph",
                    "algorithm.data_flow",
                    // Integration
                    "ai.inference",
                    "ai.stack_probe",
                    "ai.model_list",
                ]);
                tools
            }
        }
    }

    /// Check if a tool is available in this profile
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tools().iter().any(|&t| t == tool_name)
    }

    /// Get tools that overlap between this profile and another
    pub fn overlap(&self, other: &ToolProfile) -> Vec<&'static str> {
        let self_tools: HashSet<_> = self.tools().into_iter().collect();
        let other_tools: HashSet<_> = other.tools().into_iter().collect();
        self_tools.intersection(&other_tools).copied().collect()
    }
}

/// Profile manager for handling multiple active profiles
#[derive(Debug, Clone)]
pub struct ProfileManager {
    /// Active profiles (can have multiple for additive permissions)
    active_profiles: HashSet<ToolProfile>,
    /// Tool overrides (explicit enable/disable)
    tool_overrides: HashMap<String, bool>,
    /// Whether to use additive mode (union of profiles) or exclusive mode
    additive_mode: bool,
}

impl Default for ProfileManager {
    fn default() -> Self {
        ProfileManager {
            active_profiles: std::iter::once(ToolProfile::CoreCode).collect(),
            tool_overrides: HashMap::new(),
            additive_mode: true,
        }
    }
}

impl ProfileManager {
    /// Create a new profile manager with default profile
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a profile manager with a specific profile
    pub fn with_profile(profile: ToolProfile) -> Self {
        ProfileManager {
            active_profiles: std::iter::once(profile).collect(),
            tool_overrides: HashMap::new(),
            additive_mode: true,
        }
    }

    /// Create a profile manager with multiple profiles (additive mode)
    pub fn with_profiles(profiles: Vec<ToolProfile>) -> Self {
        ProfileManager {
            active_profiles: profiles.into_iter().collect(),
            tool_overrides: HashMap::new(),
            additive_mode: true,
        }
    }

    /// Add a profile to the active set
    pub fn add_profile(&mut self, profile: ToolProfile) -> &mut Self {
        self.active_profiles.insert(profile);
        self
    }

    /// Remove a profile from the active set
    pub fn remove_profile(&mut self, profile: &ToolProfile) -> &mut Self {
        self.active_profiles.remove(profile);
        self
    }

    /// Set a tool override (explicit enable or disable)
    pub fn set_tool_override(&mut self, tool: String, enabled: bool) -> &mut Self {
        self.tool_overrides.insert(tool, enabled);
        self
    }

    /// Clear all tool overrides
    pub fn clear_overrides(&mut self) -> &mut Self {
        self.tool_overrides.clear();
        self
    }

    /// Set additive mode (union of profiles) or exclusive mode
    pub fn set_additive_mode(&mut self, additive: bool) -> &mut Self {
        self.additive_mode = additive;
        self
    }

    /// Get all active profiles
    pub fn active_profiles(&self) -> &HashSet<ToolProfile> {
        &self.active_profiles
    }

    /// Check if a tool is enabled
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        // Check explicit overrides first
        if let Some(&enabled) = self.tool_overrides.get(tool_name) {
            return enabled;
        }

        // Check profiles
        if self.additive_mode {
            // Additive: enabled if ANY profile has it
            self.active_profiles.iter().any(|p| p.has_tool(tool_name))
        } else {
            // Exclusive: enabled if ALL profiles have it
            !self.active_profiles.is_empty()
                && self.active_profiles.iter().all(|p| p.has_tool(tool_name))
        }
    }

    /// Get all enabled tools
    pub fn enabled_tools(&self) -> Vec<String> {
        let mut tools: HashSet<String> = HashSet::new();

        if self.additive_mode {
            // Union of all profile tools
            for profile in &self.active_profiles {
                for tool in profile.tools() {
                    tools.insert(tool.to_string());
                }
            }
        } else {
            // Intersection of all profile tools
            if let Some(first) = self.active_profiles.iter().next() {
                tools = first.tools().iter().map(|s| s.to_string()).collect();
                for profile in &self.active_profiles {
                    let profile_tools: HashSet<_> =
                        profile.tools().iter().map(|s| s.to_string()).collect();
                    tools = tools.intersection(&profile_tools).cloned().collect();
                }
            }
        }

        // Apply overrides
        for (tool, enabled) in &self.tool_overrides {
            if *enabled {
                tools.insert(tool.clone());
            } else {
                tools.remove(tool);
            }
        }

        let mut tools: Vec<String> = tools.into_iter().collect();
        tools.sort();
        tools
    }

    /// Get tools grouped by category
    pub fn tools_by_category(&self) -> HashMap<String, Vec<String>> {
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();

        for tool in self.enabled_tools() {
            let category = tool.split('.').next().unwrap_or("other");
            categories
                .entry(category.to_string())
                .or_default()
                .push(tool);
        }

        // Sort each category
        for tools in categories.values_mut() {
            tools.sort();
        }

        categories
    }

    /// Calculate the maximum risk level among active profiles
    pub fn max_risk_level(&self) -> u8 {
        self.active_profiles
            .iter()
            .map(|p| p.risk_level())
            .max()
            .unwrap_or(1)
    }

    /// Check if any active profile allows writes
    pub fn allows_writes(&self) -> bool {
        self.active_profiles.iter().any(|p| p.allows_writes())
    }

    /// Check if any active profile allows execution
    pub fn allows_execution(&self) -> bool {
        self.active_profiles.iter().any(|p| p.allows_execution())
    }
}

impl Serialize for ProfileManager {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ProfileManager", 4)?;
        s.serialize_field(
            "active_profiles",
            &self.active_profiles.iter().collect::<Vec<_>>(),
        )?;
        s.serialize_field("tool_overrides", &self.tool_overrides)?;
        s.serialize_field("additive_mode", &self.additive_mode)?;
        s.serialize_field("enabled_tools", &self.enabled_tools())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_default() {
        let profile = ToolProfile::default();
        assert_eq!(profile, ToolProfile::CoreCode);
    }

    #[test]
    fn test_profile_from_str() {
        assert_eq!(
            ToolProfile::from_str("core-code"),
            Some(ToolProfile::CoreCode)
        );
        assert_eq!(ToolProfile::from_str("core"), Some(ToolProfile::CoreCode));
        assert_eq!(
            ToolProfile::from_str("local-power"),
            Some(ToolProfile::LocalPower)
        );
        assert_eq!(ToolProfile::from_str("invalid"), None);
    }

    #[test]
    fn test_profile_risk_levels() {
        assert_eq!(ToolProfile::PublicSafe.risk_level(), 1);
        assert_eq!(ToolProfile::CoreCode.risk_level(), 2);
        assert_eq!(ToolProfile::AgentOrchestration.risk_level(), 3);
        assert_eq!(ToolProfile::LocalPower.risk_level(), 4);
        assert_eq!(ToolProfile::ExperimentalLab.risk_level(), 5);
    }

    #[test]
    fn test_profile_permissions() {
        assert!(!ToolProfile::CoreCode.allows_writes());
        assert!(!ToolProfile::CoreCode.allows_execution());

        assert!(ToolProfile::AgentOrchestration.allows_writes());
        assert!(!ToolProfile::AgentOrchestration.allows_execution());

        assert!(ToolProfile::LocalPower.allows_writes());
        assert!(ToolProfile::LocalPower.allows_execution());
    }

    #[test]
    fn test_profile_tools() {
        let core_tools = ToolProfile::CoreCode.tools();
        assert!(core_tools.contains(&"fs.read_file"));
        assert!(core_tools.contains(&"git.read.status"));
        assert!(!core_tools.contains(&"local.exec.run"));

        let local_tools = ToolProfile::LocalPower.tools();
        assert!(local_tools.contains(&"local.exec.run"));
        assert!(local_tools.contains(&"git.write.commit"));
    }

    #[test]
    fn test_profile_has_tool() {
        assert!(ToolProfile::CoreCode.has_tool("fs.read_file"));
        assert!(!ToolProfile::CoreCode.has_tool("local.exec.run"));
    }

    #[test]
    fn test_profile_manager_default() {
        let manager = ProfileManager::new();
        assert!(manager.is_tool_enabled("fs.read_file"));
        assert!(!manager.is_tool_enabled("local.exec.run"));
    }

    #[test]
    fn test_profile_manager_add_profile() {
        let mut manager = ProfileManager::new();
        manager.add_profile(ToolProfile::LocalPower);

        assert!(manager.is_tool_enabled("fs.read_file")); // From CoreCode
        assert!(manager.is_tool_enabled("local.exec.run")); // From LocalPower
    }

    #[test]
    fn test_profile_manager_tool_override() {
        let mut manager = ProfileManager::new();
        manager.set_tool_override("fs.read_file".to_string(), false);

        assert!(!manager.is_tool_enabled("fs.read_file"));
    }

    #[test]
    fn test_profile_manager_enabled_tools() {
        let manager = ProfileManager::with_profile(ToolProfile::CoreCode);
        let tools = manager.enabled_tools();

        assert!(!tools.is_empty());
        assert!(tools.contains(&"fs.read_file".to_string()));
    }

    #[test]
    fn test_profile_manager_additive_mode() {
        let mut manager = ProfileManager::with_profiles(vec![
            ToolProfile::CoreCode,
            ToolProfile::AgentOrchestration,
        ]);

        // Additive: should have tools from both
        assert!(manager.is_tool_enabled("fs.read_file")); // CoreCode
        assert!(manager.is_tool_enabled("agent.bus.send")); // AgentOrchestration

        // Switch to exclusive mode
        manager.set_additive_mode(false);
        // Exclusive: only tools in BOTH profiles (none overlap in this case)
        assert!(!manager.is_tool_enabled("fs.read_file"));
        assert!(!manager.is_tool_enabled("agent.bus.send"));
    }

    #[test]
    fn test_profile_manager_risk_level() {
        let manager = ProfileManager::with_profile(ToolProfile::CoreCode);
        assert_eq!(manager.max_risk_level(), 2);

        let mut manager = ProfileManager::new();
        manager.add_profile(ToolProfile::LocalPower);
        assert_eq!(manager.max_risk_level(), 4);
    }

    #[test]
    fn test_tools_by_category() {
        let manager = ProfileManager::with_profile(ToolProfile::CoreCode);
        let categories = manager.tools_by_category();

        assert!(categories.contains_key("fs"));
        assert!(categories.contains_key("git"));
        assert!(categories.contains_key("code"));

        // Verify tools are sorted
        if let Some(fs_tools) = categories.get("fs") {
            assert!(fs_tools.windows(2).all(|w| w[0] <= w[1]));
        }
    }
}
