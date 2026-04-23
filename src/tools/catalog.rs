//! Tool catalog for pb-mcp-server
//!
//! Defines [`ToolInfo`] metadata for every MCP tool and a [`ToolCatalog`]
//! that holds the complete inventory. The catalog is constructed once at
//! startup and queried by profile, category, or free-text search.
//!
//! # Categories
//!
//! | Category | Prefix | Description |
//! |----------|--------|-------------|
//! | filesystem | `fs.` | File read/list/tree/find/info |
//! | git | `git.` | Git read and write operations |
//! | code | `code.` | Code intelligence, indexing, analysis |
//! | text | `text.` | Text transform, stats, summarization |
//! | compress | `compress.` | Compression, dedup, truncation |
//! | summarize | `summarize.` | Summarization primitives |
//! | agent | `agent.` | Agent bus, coordination, temp storage |
//! | plan | `plan.` | Planning, risk registers, checklists |
//! | reasoning | `reasoning.` | Evidence matrix, devil's advocate |
//! | review | `review.` | Code review, diff risk, rulebook |
//! | debug | `debug.` | Trace recording, failure clustering |
//! | image | `image.` | Image analysis, manipulation |
//! | audio | `audio.` | Audio analysis, conversion, effects |
//! | sandbox | `sandbox.` | Sandboxed execution environments |
//! | reverse-eng | `reverse-eng.` | Binary/string reverse engineering |
//! | ocr | `ocr.` | Optical character recognition |
//! | speech | `speech.` | Text-to-speech, speech-to-text |
//! | screenshot | `screenshot.` | Screen capture tools |
//! | web | `web.` | HTTP fetch, search, scrape |
//! | convert | `convert.` | Format conversion, gzip |
//! | system | `system.` | System info, environment hints |
//! | notify | `notify.` | Desktop notifications |
//! | ai | `ai.` | Ollama, context packing, semantic diff |
//! | db | `db.` | Database query and schema tools |
//! | health | — | Health/status/security endpoints |

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// ToolInfo
// ---------------------------------------------------------------------------

/// Metadata for a single MCP tool.
#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Dotted tool name, e.g. `"fs.read_file"`.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// Category group, e.g. `"filesystem"`, `"git"`, `"code"`.
    pub category: String,
    /// Profile that exposes this tool, e.g. `"core-code"`, `"local-power"`.
    pub profile: String,
    /// Risk level: `"low"`, `"medium"`, `"high"`, or `"critical"`.
    pub risk_level: String,
    /// Whether the tool writes to the filesystem.
    pub requires_write: bool,
    /// Whether the tool executes arbitrary commands.
    pub requires_exec: bool,
}

// ---------------------------------------------------------------------------
// ToolCatalog
// ---------------------------------------------------------------------------

/// Complete inventory of all MCP tools known to the server.
///
/// Constructed via [`ToolCatalog::new`] which populates every tool entry
/// from the project's modules. The catalog is immutable after construction
/// and provides query methods for profile-based filtering, category
/// browsing, and free-text search.
#[derive(Debug, Clone)]
pub struct ToolCatalog {
    tools: Vec<ToolInfo>,
}

impl ToolCatalog {
    /// Build the full catalog from all project modules.
    ///
    /// The tool list is hard-coded from the canonical source in
    /// `src/profiles.rs` and `src/ops/` and covers every tool across
    /// all profiles.
    pub fn new() -> Self {
        let tools = Vec::new();
        let mut cat = Self { tools };

        cat.add_filesystem_tools();
        cat.add_git_read_tools();
        cat.add_git_write_tools();
        cat.add_code_tools();
        cat.add_text_tools();
        cat.add_compress_tools();
        cat.add_summarize_tools();
        cat.add_agent_tools();
        cat.add_plan_tools();
        cat.add_reasoning_tools();
        cat.add_review_tools();
        cat.add_debug_tools();
        cat.add_image_tools();
        cat.add_audio_tools();
        cat.add_sandbox_tools();
        cat.add_reverse_eng_tools();
        cat.add_ocr_tools();
        cat.add_speech_tools();
        cat.add_screenshot_tools();
        cat.add_web_tools();
        cat.add_convert_tools();
        cat.add_system_tools();
        cat.add_notify_tools();
        cat.add_ai_tools();
        cat.add_db_tools();
        cat.add_health_tools();
        cat.add_context_tools();
        cat.add_testing_tools();

        cat
    }

    // -----------------------------------------------------------------------
    // Query methods
    // -----------------------------------------------------------------------

    /// Return tools available under the given profile name.
    pub fn by_profile(&self, profile: &str) -> Vec<&ToolInfo> {
        self.tools.iter().filter(|t| t.profile == profile).collect()
    }

    /// Return tools in the given category.
    pub fn by_category(&self, category: &str) -> Vec<&ToolInfo> {
        self.tools
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Free-text search across name and description (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&ToolInfo> {
        let q = query.to_lowercase();
        self.tools
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&q)
                    || t.description.to_lowercase().contains(&q)
                    || t.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Total number of tools in the catalog.
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Unique category names present in the catalog (sorted).
    pub fn categories(&self) -> Vec<String> {
        let set: HashSet<_> = self.tools.iter().map(|t| t.category.clone()).collect();
        let mut v: Vec<_> = set.into_iter().collect();
        v.sort();
        v
    }

    /// Reference to the underlying tool slice.
    pub fn all(&self) -> &[ToolInfo] {
        &self.tools
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Add a tool to the catalog.
    pub fn add(&mut self, info: ToolInfo) {
        self.tools.push(info);
    }

    /// Remove the first tool whose name matches and return `true`.
    /// Returns `false` if no tool with that name exists.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(pos) = self.tools.iter().position(|t| t.name == name) {
            self.tools.remove(pos);
            true
        } else {
            false
        }
    }

    fn add_filesystem_tools(&mut self) {
        let profile = "core-code";
        let cat = "filesystem";
        self.add(ToolInfo {
            name: "fs.read_file".into(),
            description: "Read a file's contents with optional line range".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.read_tail".into(),
            description: "Read the last N lines of a file".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.list".into(),
            description: "List directory contents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.tree".into(),
            description: "Print directory tree structure".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.find".into(),
            description: "Find files by pattern".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.info".into(),
            description: "Get file metadata and size info".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.read_json".into(),
            description: "Read and validate a JSON file".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.list_bounded".into(),
            description: "List directory with bounded results (public-safe)".into(),
            category: cat.into(),
            profile: "public-safe".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.read_file_bounded".into(),
            description: "Read file with bounded size (public-safe)".into(),
            category: cat.into(),
            profile: "public-safe".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.search_code_bounded".into(),
            description: "Search code with bounded results (public-safe)".into(),
            category: cat.into(),
            profile: "public-safe".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.find_by_name".into(),
            description: "Find files by name pattern".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.file_stats".into(),
            description: "Compute line/word/byte statistics for a file".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.batch_file_info".into(),
            description: "Get metadata for multiple files at once".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "fs.read_json_validate".into(),
            description: "Validate JSON file against size limits".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_git_read_tools(&mut self) {
        let profile = "core-code";
        let cat = "git";
        self.add(ToolInfo {
            name: "git.read.status".into(),
            description: "Show working-tree status".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.diff".into(),
            description: "Show changes between commits, trees, or working tree".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.log".into(),
            description: "Show commit logs".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.show".into(),
            description: "Show various types of objects".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.branches".into(),
            description: "List local and remote branches".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.blame".into(),
            description: "Show line-level annotation for a file".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.tags".into(),
            description: "List git tags".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.reflog".into(),
            description: "Show reference log entries".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.shortlog".into(),
            description: "Summarize git log by author".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.rev_parse".into(),
            description: "Resolve revision parameters".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.merge_base".into(),
            description: "Find the common ancestor of commits".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.describe".into(),
            description: "Describe a commit using the nearest tag".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.ls_files".into(),
            description: "List tracked files in the index".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.diff_stat".into(),
            description: "Show diffstat summary of changes".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.last_commit".into(),
            description: "Show the last commit for a path".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.stash_list".into(),
            description: "List stash entries".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.worktree_list".into(),
            description: "List working trees".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.remote_list".into(),
            description: "List remote repositories".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.read.repository_get_file".into(),
            description: "Get file content at a specific revision".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_git_write_tools(&mut self) {
        let profile = "local-power";
        let cat = "git";
        self.add(ToolInfo {
            name: "git.write.checkout".into(),
            description: "Switch branches or restore working tree files".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.commit".into(),
            description: "Record changes to the repository".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.push".into(),
            description: "Push commits to a remote".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "critical".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.pull".into(),
            description: "Pull changes from a remote".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.fetch".into(),
            description: "Download objects and refs from another repository".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.stash".into(),
            description: "Stash changes away".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.branch".into(),
            description: "Create, rename, or delete branches".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.worktree".into(),
            description: "Manage working trees".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.add".into(),
            description: "Stage file changes".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "git.write.tag".into(),
            description: "Create, list, or delete tags".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
    }

    fn add_code_tools(&mut self) {
        let profile = "core-code";
        let cat = "code";
        self.add(ToolInfo {
            name: "code.index.search".into(),
            description: "Search the code symbol index".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.index.paths_prefix".into(),
            description: "List indexed paths by prefix".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.index.symbol_prefix".into(),
            description: "Search symbols by prefix".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.inventory".into(),
            description: "Inventory of languages and symbols".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.largest_files".into(),
            description: "Find largest source files".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.duplicate_basenames".into(),
            description: "Find files with duplicate basenames".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.deps".into(),
            description: "List project dependencies".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.word_index".into(),
            description: "Build a word occurrence index".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.shallow_hash".into(),
            description: "Compute shallow content hash for files".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.analyze.path_list".into(),
            description: "List all project file paths".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.ast_dump".into(),
            description: "Dump a structured AST sketch of a source file".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.imports".into(),
            description: "Extract imports from a source file".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.dead_code".into(),
            description: "Scan for potentially dead code".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.complexity".into(),
            description: "Compute cyclomatic complexity of a function".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "code.snippet_extract".into(),
            description: "Extract code snippets from files".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.index".into(),
            description: "Full codebase symbol index".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.manifest_write".into(),
            description: "Write a codebase manifest file".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.word_index".into(),
            description: "Build a word occurrence index for the codebase".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.shallow_hash".into(),
            description: "Compute shallow content hash for all files".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.path_list".into(),
            description: "List all file paths in the codebase".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.largest_files".into(),
            description: "Find the largest files in the codebase".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.duplicate_basenames".into(),
            description: "Find duplicate basenames across the codebase".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "codebase.cargo_deps".into(),
            description: "List Cargo dependency names".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_text_tools(&mut self) {
        let profile = "core-code";
        let cat = "text";
        self.add(ToolInfo {
            name: "text.transform".into(),
            description: "Apply text transformations (case, trim, etc.)".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.summarize".into(),
            description: "Summarize text content".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.stats".into(),
            description: "Compute text statistics (chars, words, lines)".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.extract_snippets".into(),
            description: "Extract text snippets matching a pattern".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.levenshtein".into(),
            description: "Compute Levenshtein edit distance between two strings".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.djb2_hash".into(),
            description: "Compute djb2 hash of a string".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.token_jaccard".into(),
            description: "Compute token-level Jaccard similarity between two strings".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "text.longest_common_substring".into(),
            description: "Find the longest common substring".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_compress_tools(&mut self) {
        let profile = "core-code";
        let cat = "compress";
        self.add(ToolInfo {
            name: "compress.context".into(),
            description: "Compress and format context for LLM consumption".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.file".into(),
            description: "Compress a single file's contents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.redact_urls".into(),
            description: "Redact URLs in text".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.redact_emails".into(),
            description: "Redact email addresses in text".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.markdown_frontmatter".into(),
            description: "Strip or compress markdown frontmatter".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.line_prefix_groups".into(),
            description: "Summarize groups of lines by common prefix".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.drop_lines_matching_regex".into(),
            description: "Drop lines matching a regex pattern".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.ws_collapse".into(),
            description: "Collapse whitespace in text".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.double_slash_line_comments".into(),
            description: "Strip // line comments".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.strip_tags".into(),
            description: "Strip HTML/XML tags from text".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.fenced_code_blocks".into(),
            description: "Compress fenced code blocks".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.unique_lines".into(),
            description: "Deduplicate lines while preserving order".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.lines_head".into(),
            description: "Keep only the first N lines".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.lines_tail".into(),
            description: "Keep only the last N lines".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.collapse_blank_lines".into(),
            description: "Collapse consecutive blank lines".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.truncate_middle".into(),
            description: "Truncate text in the middle with ellipsis".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.simplify_whitespace".into(),
            description: "Simplify all whitespace to single spaces".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.truncate_lines".into(),
            description: "Truncate each line to a max length".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.markdown_links".into(),
            description: "Compress markdown links to display text".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.gzip_base64".into(),
            description: "Gzip compress and base64 encode text".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.dedent".into(),
            description: "Remove common leading indentation".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.paragraphs".into(),
            description: "Compress text by paragraph boundaries".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.unified_diff_stats".into(),
            description: "Extract stats from a unified diff".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "compress.sentences".into(),
            description: "Compress text by sentence boundaries".into(),
            category: cat.into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_summarize_tools(&mut self) {
        let profile = "review-refactor";
        let cat = "summarize";
        self.add(ToolInfo {
            name: "summarize.multi_file".into(),
            description: "Summarize multiple files into a combined overview".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.file".into(),
            description: "Summarize a single file".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.token_estimate".into(),
            description: "Estimate token count for text".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.line_prefix_groups".into(),
            description: "Summarize lines by prefix groups".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.top_terms".into(),
            description: "Extract top terms from text".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.identifier_frequency".into(),
            description: "Count identifier frequency in code".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.duplicate_lines".into(),
            description: "Find duplicate lines in text".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.line_histogram".into(),
            description: "Build a line-length histogram".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.delimited_header".into(),
            description: "Summarize CSV/TSV header columns".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.json_structure".into(),
            description: "Summarize the structure of a JSON document".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.bullets".into(),
            description: "Extract and summarize markdown bullet points".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.headings".into(),
            description: "Extract markdown headings".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.markdown_toc".into(),
            description: "Generate a markdown table of contents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.diff_file_list".into(),
            description: "List changed files from a diff".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.segments".into(),
            description: "Segment text and summarize each part".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.first_lines".into(),
            description: "Show the first few lines of text".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.rank_lines_by_keywords".into(),
            description: "Rank lines by keyword relevance".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "summarize.text".into(),
            description: "General text summarization".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_agent_tools(&mut self) {
        let profile = "agent-orchestration";
        let cat = "agent";
        self.add(ToolInfo {
            name: "agent.bus.send".into(),
            description: "Send a message to the agent bus".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.bus.inbox".into(),
            description: "Read messages addressed to an agent".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.bus.thread".into(),
            description: "Read messages in a thread/topic".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.bus.recent".into(),
            description: "Read recent bus messages".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.bus.search".into(),
            description: "Search bus messages by content".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.bus.status".into(),
            description: "Get bus status and message counts".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.ask".into(),
            description: "Ask another agent a question".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.handoff".into(),
            description: "Hand off a task to another agent".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.consensus".into(),
            description: "Initiate a consensus round among agents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.subagent_task".into(),
            description: "Assign a task to a subagent".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.subagent_done".into(),
            description: "Mark a subagent task as done".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.roster".into(),
            description: "List registered agents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.list_topics".into(),
            description: "List active bus topics".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.temp.write".into(),
            description: "Write to agent temp storage".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.temp.read".into(),
            description: "Read from agent temp storage".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.temp.list".into(),
            description: "List agent temp storage entries".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.temp.delete".into(),
            description: "Delete an agent temp storage entry".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.wait.sleep".into(),
            description: "Sleep for a specified duration".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.wait.for_path".into(),
            description: "Wait until a file path exists".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.wait.until".into(),
            description: "Wait until a condition is met".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.sync_all".into(),
            description: "Synchronize agent state across all agents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.sync_confirm".into(),
            description: "Confirm synchronization with two-phase commit".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "agent.coordination_playbook".into(),
            description: "Generate agent coordination playbook".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_plan_tools(&mut self) {
        let profile = "review-refactor";
        let cat = "plan";
        self.add(ToolInfo {
            name: "plan.risk_register".into(),
            description: "Create or update a risk register".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "plan.definition_of_done".into(),
            description: "Generate a definition-of-done checklist".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "plan.test_strategy".into(),
            description: "Generate a test strategy document".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "plan.handoff_brief".into(),
            description: "Generate a handoff brief for agent transitions".into(),
            category: cat.into(),
            profile: "agent-orchestration".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "plan.work_buckets".into(),
            description: "Partition work into parallel buckets".into(),
            category: cat.into(),
            profile: "agent-orchestration".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "plan.execution_checklist".into(),
            description: "Generate an execution checklist".into(),
            category: cat.into(),
            profile: "agent-orchestration".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_reasoning_tools(&mut self) {
        let profile = "review-refactor";
        let cat = "reasoning";
        self.add(ToolInfo {
            name: "reasoning.evidence_matrix".into(),
            description: "Build an evidence matrix for hypothesis evaluation".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "reasoning.devils_advocate".into(),
            description: "Generate devil's advocate counter-arguments".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "reasoning.premortem".into(),
            description: "Run a premortem analysis".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_review_tools(&mut self) {
        let profile = "review-refactor";
        let cat = "review";
        self.add(ToolInfo {
            name: "review.diff_risk".into(),
            description: "Classify diff risk level".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "review.focused_review".into(),
            description: "Generate a focused code review for changed files".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "review.missing_tests".into(),
            description: "Identify missing test coverage".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "review.pr_packet".into(),
            description: "Generate a PR review packet".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "review.rulebook_check".into(),
            description: "Check code against a rulebook".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "git.diff_snippets".into(),
            description: "Extract diff snippets for review".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_debug_tools(&mut self) {
        let profile = "local-power";
        let cat = "debug";
        self.add(ToolInfo {
            name: "debug.trace_start".into(),
            description: "Start recording a debug trace".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "debug.trace_stop".into(),
            description: "Stop recording a debug trace".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "debug.trace_replay".into(),
            description: "Replay a recorded debug trace".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "debug.failure_cluster".into(),
            description: "Cluster failures by root cause".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "debug.root_cause_analysis".into(),
            description: "Perform root cause analysis on failures".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_image_tools(&mut self) {
        let profile = "local-power";
        let cat = "image";
        self.add(ToolInfo {
            name: "image.info".into(),
            description: "Get image metadata and dimensions".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.resize".into(),
            description: "Resize an image".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.convert".into(),
            description: "Convert image format".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.crop".into(),
            description: "Crop a region of an image".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.grayscale".into(),
            description: "Convert image to grayscale".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.blur".into(),
            description: "Apply gaussian blur to image".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "image.thumbnail".into(),
            description: "Generate a thumbnail of an image".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: false,
        });
    }

    fn add_audio_tools(&mut self) {
        let profile = "local-power";
        let cat = "audio";
        self.add(ToolInfo {
            name: "audio.info".into(),
            description: "Get audio file metadata".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.convert".into(),
            description: "Convert audio file format".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.extract_segment".into(),
            description: "Extract a segment of an audio file".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.trim".into(),
            description: "Trim audio between start and end times".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.merge".into(),
            description: "Merge multiple audio files".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.mix".into(),
            description: "Mix two audio tracks".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.volume".into(),
            description: "Adjust audio volume".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.speed".into(),
            description: "Change audio playback speed".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.reverse".into(),
            description: "Reverse an audio track".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.normalize".into(),
            description: "Normalize audio loudness".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.split".into(),
            description: "Split audio into segments".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.extract_from_video".into(),
            description: "Extract audio track from video".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.waveform".into(),
            description: "Generate waveform data for audio".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "audio.silence_detect".into(),
            description: "Detect silent sections in audio".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_sandbox_tools(&mut self) {
        let profile = "local-power";
        let cat = "sandbox";
        self.add(ToolInfo {
            name: "sandbox.run".into(),
            description: "Run a command in a sandboxed environment".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "sandbox.run_stdin".into(),
            description: "Run a sandboxed command with stdin input".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "sandbox.which".into(),
            description: "Check if a program is in the allowlist".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_reverse_eng_tools(&mut self) {
        let profile = "experimental-lab";
        let cat = "reverse-eng";
        self.add(ToolInfo {
            name: "reverse-eng.strings".into(),
            description: "Extract printable strings from a binary".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "reverse-eng.hex_dump".into(),
            description: "Hex dump of a file region".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "reverse-eng.symbol_table".into(),
            description: "List symbols from an ELF/binary".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "reverse-eng.disassemble".into(),
            description: "Disassemble a binary section".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "reverse-eng.header_info".into(),
            description: "Show ELF/binary header information".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_ocr_tools(&mut self) {
        let profile = "local-power";
        let cat = "ocr";
        self.add(ToolInfo {
            name: "local.ocr.tesseract".into(),
            description: "OCR a small inline image with Tesseract".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.ocr.tesseract_file".into(),
            description: "OCR a file with Tesseract".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ocr.tesseract_stream".into(),
            description: "Stream OCR results for large images".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ocr.tesseract_batch".into(),
            description: "Batch OCR multiple images".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_speech_tools(&mut self) {
        let profile = "experimental-lab";
        let cat = "speech";
        self.add(ToolInfo {
            name: "speech.tts".into(),
            description: "Convert text to speech".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: true,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "speech.stt".into(),
            description: "Convert speech to text (transcription)".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "speech.stt_stream".into(),
            description: "Stream speech-to-text transcription".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_screenshot_tools(&mut self) {
        let profile = "local-power";
        let cat = "screenshot";
        self.add(ToolInfo {
            name: "local.screenshot.capture".into(),
            description: "Capture a screenshot of the display".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "high".into(),
            requires_write: true,
            requires_exec: true,
        });
    }

    fn add_web_tools(&mut self) {
        let profile = "public-safe";
        let cat = "web";
        self.add(ToolInfo {
            name: "web.fetch".into(),
            description: "Fetch a URL and return content".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "web.search".into(),
            description: "Search the web".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "web.scrape".into(),
            description: "Scrape a web page for structured content".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "web.head".into(),
            description: "Fetch HTTP headers for a URL".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "web.websocket".into(),
            description: "Open a WebSocket connection".into(),
            category: cat.into(),
            profile: "experimental-lab".into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_convert_tools(&mut self) {
        let profile = "local-power";
        let cat = "convert";
        self.add(ToolInfo {
            name: "convert.format".into(),
            description: "Convert between data formats (JSON/YAML/TOML/CSV/etc.)".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "convert.gzip_compress".into(),
            description: "Gzip compress and base64 encode".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "convert.gzip_decompress".into(),
            description: "Base64 decode and gzip decompress".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_system_tools(&mut self) {
        let profile = "core-code";
        let cat = "system";
        self.add(ToolInfo {
            name: "system.env_hints".into(),
            description: "Get environment and toolchain hints".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "system.program_version".into(),
            description: "Check the installed version of a program".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.exec.run".into(),
            description: "Execute a terminal command".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "critical".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.exec.run_stdin".into(),
            description: "Execute a command with stdin input".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "critical".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.exec.which".into(),
            description: "Check if a program is available on PATH".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "local.exec.program_version".into(),
            description: "Check installed program version".into(),
            category: cat.into(),
            profile: "local-power".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_notify_tools(&mut self) {
        let profile = "local-power";
        let cat = "notify";
        self.add(ToolInfo {
            name: "notify.desktop".into(),
            description: "Send a desktop notification".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "notify.sound".into(),
            description: "Play a notification sound".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "notify.log".into(),
            description: "Write a message to the notification log".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: true,
            requires_exec: false,
        });
    }

    fn add_ai_tools(&mut self) {
        let profile = "experimental-lab";
        let cat = "ai";
        self.add(ToolInfo {
            name: "ai.ollama_generate".into(),
            description: "Generate text with an Ollama model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ai.ollama_embed".into(),
            description: "Generate embeddings with an Ollama model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ai.prompt_template".into(),
            description: "Render a prompt template with variables".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "ai.context_pack_smart".into(),
            description: "Smart context packing with keyword scoring".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "ai.semantic_diff".into(),
            description: "Semantic diff analysis mapping hunks to code units".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "ai.inference".into(),
            description: "Run AI inference on local models".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ai.stack_probe".into(),
            description: "Probe the local AI stack for available models".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ai.model_list".into(),
            description: "List available AI models".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ml.embed".into(),
            description: "Generate ML embeddings".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ml.classify".into(),
            description: "Classify input with an ML model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ml.detect".into(),
            description: "Object detection with an ML model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ml.segment".into(),
            description: "Image segmentation with an ML model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "ml.transcribe".into(),
            description: "Audio transcription with an ML model".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_db_tools(&mut self) {
        let profile = "local-power";
        let cat = "db";
        self.add(ToolInfo {
            name: "db.query".into(),
            description: "Execute a read-only SQL query".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "db.schema".into(),
            description: "Show database schema".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "db.tables".into(),
            description: "List database tables".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "db.describe".into(),
            description: "Describe a database table's columns".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: true,
        });
    }

    fn add_health_tools(&mut self) {
        let profile = "public-safe";
        let cat = "health";
        self.add(ToolInfo {
            name: "health".into(),
            description: "Health check endpoint".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "status".into(),
            description: "Server status and profile".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "security_status".into(),
            description: "Security audit status".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "project_info".into(),
            description: "Project metadata".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "metrics".into(),
            description: "Performance metrics".into(),
            category: cat.into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_context_tools(&mut self) {
        let profile = "core-code";
        let cat = "context";
        self.add(ToolInfo {
            name: "pack_context".into(),
            description: "Pack files into a compact context block".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "workspace_context".into(),
            description: "Gather workspace context for AI agents".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }

    fn add_testing_tools(&mut self) {
        let profile = "local-power";
        let cat = "testing";
        self.add(ToolInfo {
            name: "local.test.rust_test".into(),
            description: "Run Rust tests with cargo test".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.test.rust_check".into(),
            description: "Run cargo check".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "local.test.pytest".into(),
            description: "Run Python tests with pytest".into(),
            category: cat.into(),
            profile: profile.into(),
            risk_level: "medium".into(),
            requires_write: false,
            requires_exec: true,
        });
        self.add(ToolInfo {
            name: "algorithm.complexity".into(),
            description: "Analyze algorithmic complexity".into(),
            category: "algorithm".into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "algorithm.dependencies".into(),
            description: "Analyze code dependency graph".into(),
            category: "algorithm".into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "algorithm.call_graph".into(),
            description: "Build a function call graph".into(),
            category: "algorithm".into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
        self.add(ToolInfo {
            name: "algorithm.data_flow".into(),
            description: "Trace data flow through code".into(),
            category: "algorithm".into(),
            profile: "experimental-lab".into(),
            risk_level: "low".into(),
            requires_write: false,
            requires_exec: false,
        });
    }
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_populated() {
        let cat = ToolCatalog::new();
        assert!(
            cat.count() > 100,
            "catalog should have 100+ tools, got {}",
            cat.count()
        );
    }

    #[test]
    fn categories_are_unique() {
        let cat = ToolCatalog::new();
        let cats = cat.categories();
        let mut seen = HashSet::new();
        for c in &cats {
            assert!(seen.insert(c.clone()), "duplicate category: {c}");
        }
    }

    #[test]
    fn by_profile_returns_subset() {
        let cat = ToolCatalog::new();
        let core = cat.by_profile("core-code");
        assert!(!core.is_empty(), "core-code should have tools");
        for t in &core {
            assert_eq!(t.profile, "core-code");
        }
    }

    #[test]
    fn by_category_returns_subset() {
        let cat = ToolCatalog::new();
        let fs = cat.by_category("filesystem");
        assert!(!fs.is_empty(), "filesystem should have tools");
        for t in &fs {
            assert_eq!(t.category, "filesystem");
        }
    }

    #[test]
    fn search_finds_matching_tools() {
        let cat = ToolCatalog::new();
        let results = cat.search("git");
        assert!(
            !results.is_empty(),
            "search for 'git' should return results"
        );
    }

    #[test]
    fn all_tools_have_nonempty_names() {
        let cat = ToolCatalog::new();
        for t in cat.all() {
            assert!(!t.name.is_empty(), "tool name must not be empty");
            assert!(
                !t.description.is_empty(),
                "tool {} description must not be empty",
                t.name
            );
            assert!(
                !t.category.is_empty(),
                "tool {} category must not be empty",
                t.name
            );
            assert!(
                !t.profile.is_empty(),
                "tool {} profile must not be empty",
                t.name
            );
            assert!(
                !t.risk_level.is_empty(),
                "tool {} risk_level must not be empty",
                t.name
            );
        }
    }

    #[test]
    fn risk_levels_are_valid() {
        let cat = ToolCatalog::new();
        let valid = ["low", "medium", "high", "critical"];
        for t in cat.all() {
            assert!(
                valid.contains(&t.risk_level.as_str()),
                "tool {} has invalid risk_level: {}",
                t.name,
                t.risk_level
            );
        }
    }
}
