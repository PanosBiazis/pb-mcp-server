mod bus;
mod ops;

use crate::ops::{
    batch_file_info_work, compress_context_work, compress_file_work, extract_snippets_from_file_work,
    extract_snippets_work, file_info_work, file_stats_work, find_files_by_name_work,
    find_files_work, git_blame_work, git_branches_work, git_diff_work, git_log_work,
    git_diff_snippets_work, git_rev_parse_work, git_show_work, git_status_work, grep_snippets_work,
    grep_work,
    list_files_work, read_file_tail_work, read_file_work, read_json_file_work, tree_work,
    pack_context_work, summarize_text_work, workspace_context_work,
};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::{stdin, stdout};

#[derive(Clone)]
struct MintMcp {
    root: PathBuf,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PathArgs {
    #[schemars(description = "Directory path relative to project root, or absolute under root. Empty string = project root.")]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReadFileArgs {
    #[schemars(description = "UTF-8 text file path under the project root.")]
    path: String,
    #[schemars(description = "1-based start line (inclusive) for a numbered slice; omit for whole file (subject to size limits).")]
    #[serde(default)]
    start_line: Option<usize>,
    #[schemars(description = "1-based end line (inclusive). With start_line only, defaults to end of loaded content.")]
    #[serde(default)]
    end_line: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitStatusArgs {
    #[schemars(description = "Repo directory relative to root, or omit for project root.")]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GrepArgs {
    #[schemars(description = "Regex pattern unless literal=true (then matched as plain text).")]
    pattern: String,
    #[schemars(description = "Directory or file subtree to search; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Stop after this many matches (server caps at 200).")]
    #[serde(default = "default_grep_max")]
    max_matches: usize,
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[serde(default)]
    literal: bool,
    #[serde(default = "default_true")]
    respect_gitignore: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FindFilesArgs {
    #[schemars(description = "Glob relative to search root, e.g. **/*.rs or src/**/*.ts")]
    glob: String,
    #[schemars(description = "Subtree to search; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max paths returned (server caps at 600).")]
    #[serde(default = "default_find_max")]
    max_files: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TreeArgs {
    #[schemars(description = "Directory to show; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Depth from the given directory (clamped 1–12).")]
    #[serde(default = "default_tree_depth")]
    max_depth: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitDiffArgs {
    #[schemars(description = "Git repo path; omit for project root.")]
    path: Option<String>,
    #[serde(default)]
    staged: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FileInfoArgs {
    #[schemars(description = "File or directory path under the project root.")]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentSendArgs {
    #[schemars(description = "Stable sender id for this session (alphanumeric, _, -, .) e.g. cursor-main, review-bot.")]
    from_agent: String,
    #[schemars(description = "Message body; truncated if extremely long.")]
    message: String,
    #[schemars(description = "Recipient id for a DM; omit to broadcast to every agent polling the bus.")]
    #[serde(default)]
    to_agent: Option<String>,
    #[schemars(description = "Thread key; use the same string on every agent_send in a thread, then read with agent_thread.")]
    #[serde(default)]
    topic: Option<String>,
    #[schemars(description = "Short label: note, question, suggestion, handoff, …")]
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentInboxArgs {
    #[schemars(description = "Same stable id as when this agent calls agent_send.")]
    for_agent: String,
    #[schemars(description = "Max messages (1–200), newest among matches after filtering.")]
    #[serde(default = "default_inbox_limit")]
    limit: usize,
    #[serde(default = "default_true")]
    skip_own_broadcasts: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentThreadArgs {
    #[schemars(description = "Exact topic string shared across agent_send calls.")]
    topic: String,
    #[schemars(description = "Max messages in the thread window (1–200).")]
    #[serde(default = "default_thread_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentRecentArgs {
    #[schemars(description = "Tail size of the global bus (1–150).")]
    #[serde(default = "default_recent_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct EmptyArgs {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitLogArgs {
    #[schemars(description = "Git repo directory (subfolder of project root); omit for project root.")]
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "Max commits (1–150).")]
    #[serde(default = "default_git_log_limit")]
    limit: usize,
    #[serde(default = "default_true")]
    oneline: bool,
    #[schemars(description = "Only history touching this path, relative to project root (e.g. src/main.rs).")]
    #[serde(default)]
    path_filter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitShowArgs {
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "Commit-ish or object (e.g. HEAD, main~1, abc123, or rev:path for a blob).")]
    rev: String,
    #[schemars(description = "If false: diffstat only. If true: full patch (truncated).")]
    #[serde(default)]
    patch: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitBranchesArgs {
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitBlameArgs {
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "File path under project root.")]
    file: String,
    #[schemars(description = "1-based start line (inclusive).")]
    start_line: usize,
    #[schemars(description = "1-based end line (inclusive); max 400 lines per call.")]
    end_line: usize,
    #[schemars(description = "Optional revision (default: HEAD).")]
    #[serde(default)]
    rev: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitRevParseArgs {
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "Refs to resolve (e.g. HEAD, main, MERGE_HEAD); max 16.")]
    refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReadFileTailArgs {
    #[schemars(description = "File path under project root.")]
    path: String,
    #[schemars(description = "How many lines to return from end (1–500).")]
    #[serde(default = "default_tail_lines")]
    last_lines: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FileStatsArgs {
    #[schemars(description = "File path; counts newlines in a scan window and shows a short preview.")]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FindFilesByNameArgs {
    #[schemars(description = "Substring matched against each file path relative to search root (e.g. \"test\", \"Cargo\").")]
    substring: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    case_insensitive: bool,
    #[serde(default = "default_name_find_max")]
    max_files: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReadJsonArgs {
    #[schemars(description = "JSON file path (max 1 MiB).")]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct BatchFileInfoArgs {
    #[schemars(description = "Up to 30 paths relative to project root or absolute under root.")]
    paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressContextArgs {
    #[schemars(description = "Arbitrary context text to compress (LLM prompt budgeting input).")]
    text: String,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "Compression mode: truncate or head_tail.")]
    #[serde(default = "default_compress_mode")]
    mode: String,
    #[schemars(description = "Remove consecutive duplicate lines (helps when context repeats).")]
    #[serde(default)]
    dedupe_lines: bool,
    #[schemars(description = "Collapse intra-line whitespace to a single space (deterministic).")]
    #[serde(default)]
    collapse_whitespace: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ExtractSnippetsArgs {
    #[schemars(description = "Input text to extract snippets from.")]
    text: String,
    #[schemars(description = "Regex pattern (or literal=true for fixed string matches).")]
    pattern: String,
    #[schemars(description = "If true, treat `pattern` as a plain string (escaped).")]
    #[serde(default)]
    literal: bool,
    #[schemars(description = "Match case-sensitively unless false.")]
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "How many surrounding lines to keep on each side of each match.")]
    #[serde(default = "default_extract_context_lines")]
    context_lines: usize,
    #[schemars(description = "Max number of match lines to consider (windows may merge).")]
    #[serde(default = "default_extract_max_snippets")]
    max_snippets: usize,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_extract_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentBusCompactArgs {
    #[schemars(description = "How many recent bus messages to include in the compact window.")]
    #[serde(default = "default_bus_compact_limit")]
    limit: usize,
    #[schemars(description = "Whether to include compact/truncated message bodies in the output.")]
    #[serde(default = "default_true")]
    show_bodies: bool,
    #[schemars(description = "Max characters per message body when show_bodies=true.")]
    #[serde(default = "default_bus_compact_max_body_chars")]
    max_body_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressFileArgs {
    #[schemars(description = "File path under the project root.")]
    path: String,
    #[schemars(description = "1-based start line (inclusive) for a numbered slice; omit for whole file (loaded chunk).")]
    #[serde(default)]
    start_line: Option<usize>,
    #[schemars(description = "1-based end line (inclusive). With start_line only, defaults to end of loaded content.")]
    #[serde(default)]
    end_line: Option<usize>,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "Compression mode: truncate or head_tail.")]
    #[serde(default = "default_compress_mode")]
    mode: String,
    #[schemars(description = "Remove consecutive duplicate lines (helps when context repeats).")]
    #[serde(default)]
    dedupe_lines: bool,
    #[schemars(description = "Collapse intra-line whitespace to a single space (deterministic).")]
    #[serde(default)]
    collapse_whitespace: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ExtractSnippetsFromFileArgs {
    #[schemars(description = "File path under the project root.")]
    path: String,
    #[schemars(description = "Regex pattern (or literal=true for fixed string matches).")]
    pattern: String,
    #[schemars(description = "If true, treat `pattern` as a plain string (escaped).")]
    #[serde(default)]
    literal: bool,
    #[schemars(description = "Match case-sensitively unless false.")]
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "How many surrounding lines to keep on each side of each match.")]
    #[serde(default = "default_extract_context_lines")]
    context_lines: usize,
    #[schemars(description = "Max number of match lines to consider (windows may merge).")]
    #[serde(default = "default_extract_max_snippets")]
    max_snippets: usize,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_extract_max_chars")]
    max_chars: usize,
    #[schemars(description = "Optional 1-based start line for limiting input read.")]
    #[serde(default)]
    start_line: Option<usize>,
    #[schemars(description = "Optional 1-based end line for limiting input read.")]
    #[serde(default)]
    end_line: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GrepSnippetsArgs {
    #[schemars(description = "Regex pattern (or literal=true for fixed string matches).")]
    pattern: String,
    #[schemars(description = "Directory or file subtree to search; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "If true, treat `pattern` as a plain string (escaped).")]
    #[serde(default)]
    literal: bool,
    #[schemars(description = "Match case-sensitively unless false.")]
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "Respect .gitignore / hidden rules (like grep_code).")]
    #[serde(default = "default_true")]
    respect_gitignore: bool,
    #[schemars(description = "How many surrounding lines to keep on each side of each match.")]
    #[serde(default = "default_extract_context_lines")]
    context_lines: usize,
    #[schemars(description = "Max snippet windows to return across all files.")]
    #[serde(default = "default_extract_max_snippets")]
    max_snippets: usize,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_extract_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentBusSearchArgs {
    #[schemars(description = "Substring to search for inside message bodies.")]
    needle: String,
    #[schemars(description = "Case sensitive body matching unless false.")]
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "Max matches to return (newest first).")]
    #[serde(default = "default_bus_search_limit")]
    limit: usize,
    #[schemars(description = "Whether to include truncated message bodies in results.")]
    #[serde(default = "default_true")]
    show_bodies: bool,
    #[schemars(description = "Max characters per message body when show_bodies=true.")]
    #[serde(default = "default_bus_search_max_body_chars")]
    max_body_chars: usize,
    #[schemars(description = "Optional filter: sender id.")]
    #[serde(default)]
    from_agent: Option<String>,
    #[schemars(description = "Optional filter: recipient id.")]
    #[serde(default)]
    to_agent: Option<String>,
    #[schemars(description = "Optional filter: kind (note/question/suggestion...).")]
    #[serde(default)]
    kind: Option<String>,
    #[schemars(description = "Optional filter: exact topic string.")]
    #[serde(default)]
    topic: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeTextArgs {
    #[schemars(description = "Input text to summarize.")]
    text: String,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_summarize_max_chars")]
    max_chars: usize,
    #[schemars(description = "Max number of sentences chosen for the summary.")]
    #[serde(default = "default_summarize_max_sentences")]
    max_sentences: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PackContextArgs {
    #[schemars(description = "Context segments to pack (each segment may be a file excerpt, message, etc.).")]
    segments: Vec<String>,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_pack_max_chars")]
    max_chars: usize,
    #[schemars(description = "Compression mode for the combined packed string.")]
    #[serde(default = "default_compress_mode")]
    mode: String,
    #[schemars(description = "Remove consecutive duplicate lines (helps when context repeats).")]
    #[serde(default)]
    dedupe_lines: bool,
    #[schemars(description = "Collapse intra-line whitespace to a single space (deterministic).")]
    #[serde(default)]
    collapse_whitespace: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitDiffSnippetsArgs {
    #[schemars(description = "Git repo directory (subfolder of project root); omit for project root.")]
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "If true, use --staged diff. If false, compare working tree to index.")]
    #[serde(default)]
    staged: bool,
    #[schemars(description = "Regex pattern (or literal=true for fixed string matches).")]
    pattern: String,
    #[schemars(description = "If true, treat `pattern` as a plain string (escaped).")]
    #[serde(default)]
    literal: bool,
    #[schemars(description = "Match case-sensitively unless false.")]
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "How many surrounding lines to keep on each side of each match.")]
    #[serde(default = "default_extract_context_lines")]
    context_lines: usize,
    #[schemars(description = "Max snippet windows to return across the diff.")]
    #[serde(default = "default_extract_max_snippets")]
    max_snippets: usize,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_extract_max_chars")]
    max_chars: usize,
}

fn default_true() -> bool {
    true
}

fn default_grep_max() -> usize {
    120
}

fn default_find_max() -> usize {
    400
}

fn default_tree_depth() -> usize {
    4
}

fn default_inbox_limit() -> usize {
    40
}

fn default_thread_limit() -> usize {
    60
}

fn default_recent_limit() -> usize {
    80
}

fn default_git_log_limit() -> usize {
    35
}

fn default_tail_lines() -> usize {
    50
}

fn default_name_find_max() -> usize {
    200
}

fn default_compress_max_chars() -> usize {
    6_000
}

fn default_compress_mode() -> String {
    "head_tail".to_string()
}

fn default_extract_context_lines() -> usize {
    2
}

fn default_extract_max_snippets() -> usize {
    25
}

fn default_extract_max_chars() -> usize {
    12_000
}

fn default_bus_compact_limit() -> usize {
    200
}

fn default_bus_compact_max_body_chars() -> usize {
    500
}

fn default_bus_search_limit() -> usize {
    30
}

fn default_bus_search_max_body_chars() -> usize {
    500
}

fn default_summarize_max_chars() -> usize {
    6000
}

fn default_summarize_max_sentences() -> usize {
    8
}

fn default_pack_max_chars() -> usize {
    8000
}

impl MintMcp {
    fn new() -> Self {
        let root = std::env::var("MINT_PROJECT_ROOT")
            .or_else(|_| std::env::var("PROJECT_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        Self {
            root,
            tool_router: Self::tool_router(),
        }
    }

    /// Maps worker output to MCP content; strings starting with `Error:` / `Error ` use `is_error` so clients treat them as failures.
    fn tool_text(text: impl Into<String>) -> Result<CallToolResult, McpError> {
        let s = text.into();
        let is_err = s.starts_with("Error:") || s.starts_with("Error ");
        let content = vec![Content::text(s)];
        Ok(if is_err {
            CallToolResult::error(content)
        } else {
            CallToolResult::success(content)
        })
    }

    fn resolve_path(&self, input: &str) -> Result<PathBuf, String> {
        let base = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());

        let candidate = if input.trim().is_empty() {
            base.clone()
        } else {
            let p = Path::new(input);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                base.join(p)
            }
        };

        let resolved = candidate
            .canonicalize()
            .map_err(|e| format!("Invalid path: {e}"))?;

        if resolved.starts_with(&base) {
            Ok(resolved)
        } else {
            Err("Path escapes project root (set MINT_PROJECT_ROOT or PROJECT_ROOT)".to_string())
        }
    }

    fn project_root_for_bus(&self) -> PathBuf {
        self.root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone())
    }

    /// Directory passed to `git -C` (repo root), defaulting to the MCP project root.
    fn git_working_dir(&self, path: &Option<String>) -> Result<PathBuf, String> {
        match path {
            Some(p) if !p.trim().is_empty() => self.resolve_path(p),
            _ => Ok(self.root.canonicalize().unwrap_or_else(|_| self.root.clone())),
        }
    }
}

#[tool_router]
impl MintMcp {
    #[tool(
        description = "Call first when orienting in a repo: canonical project root, git branch, sorted top-level names, whether the multi-agent bus file exists, and hard limits for read/grep/tree/diff. Cheap single round-trip for planning later tool calls.",
        annotations(title = "Workspace context", read_only_hint = true)
    )]
    async fn workspace_context(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let out = tokio::task::spawn_blocking(move || workspace_context_work(root))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read a UTF-8 text file under the project root. For large files use start_line/end_line (1-based, line numbers in output). Whole-file reads are capped by byte and response size.",
        annotations(title = "Read file", read_only_hint = true)
    )]
    async fn read_file(
        &self,
        Parameters(args): Parameters<ReadFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let start = args.start_line;
        let end = args.end_line;
        let out = tokio::task::spawn_blocking(move || read_file_work(path, start, end))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "List entries in one directory (sorted). Use project_tree for hierarchy or find_files/grep_code to locate paths by pattern.",
        annotations(title = "List directory", read_only_hint = true)
    )]
    async fn list_files(
        &self,
        Parameters(args): Parameters<PathArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let root_display = self.root.clone();
        let out = tokio::task::spawn_blocking(move || list_files_work(root_display, path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Search file contents with regex (or literal=true for fixed string). Prefer this over reading many files to find symbols. Narrow `path` when possible; respects .gitignore by default.",
        annotations(title = "Grep codebase", read_only_hint = true)
    )]
    async fn grep_code(
        &self,
        Parameters(args): Parameters<GrepArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let pattern = args.pattern.clone();
        let literal = args.literal;
        let case_sensitive = args.case_sensitive;
        let respect = args.respect_gitignore;
        let max_m = args.max_matches;

        let out = tokio::task::spawn_blocking(move || {
            grep_work(
                search_root,
                pattern,
                literal,
                case_sensitive,
                respect,
                max_m,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Find file paths by glob under a subtree (e.g. **/*.rs, src/**/*.ts). Respects .gitignore; returns relative paths.",
        annotations(title = "Find files by glob", read_only_hint = true)
    )]
    async fn find_files(
        &self,
        Parameters(args): Parameters<FindFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let glob = args.glob.clone();
        let max_f = args.max_files;
        let out =
            tokio::task::spawn_blocking(move || find_files_work(search_root, glob, max_f))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "ASCII directory tree (depth-limited, entry-capped). Good for choosing where to narrow find_files or grep_code.",
        annotations(title = "Project tree", read_only_hint = true)
    )]
    async fn project_tree(
        &self,
        Parameters(args): Parameters<TreeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let depth = args.max_depth;
        let out = tokio::task::spawn_blocking(move || tree_work(search_root, depth))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "git diff --no-color from the repo root (or given path); set staged=true for the index. Output may be truncated.",
        annotations(title = "Git diff", read_only_hint = true)
    )]
    async fn git_diff(
        &self,
        Parameters(args): Parameters<GitDiffArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match &args.path {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(path) => path,
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => self.root.clone(),
        };

        let staged = args.staged;
        let out = tokio::task::spawn_blocking(move || git_diff_work(dir, staged))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Size, kind (file/dir), and mtime for a path—use before reading unknown large files.",
        annotations(title = "File metadata", read_only_hint = true)
    )]
    async fn file_info(
        &self,
        Parameters(args): Parameters<FileInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let out = tokio::task::spawn_blocking(move || file_info_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "git status --short --branch for quick working-tree overview before diff or commits.",
        annotations(title = "Git status", read_only_hint = true)
    )]
    async fn git_status(
        &self,
        Parameters(args): Parameters<GitStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = if let Some(ref p) = args.path {
            if p.trim().is_empty() {
                self.root.clone()
            } else {
                match self.resolve_path(p) {
                    Ok(path) => path,
                    Err(e) => return Self::tool_text(format!("Error: {e}")),
                }
            }
        } else {
            self.root.clone()
        };

        let out = tokio::task::spawn_blocking(move || git_status_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "git log with optional path filter. Use for recent history, finding when code changed, or listing commits touching a file.",
        annotations(title = "Git log", read_only_hint = true)
    )]
    async fn git_log(
        &self,
        Parameters(args): Parameters<GitLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let repo_canon = repo.canonicalize().unwrap_or_else(|_| repo.clone());
        let path_in_repo = match &args.path_filter {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(abs) => match crate::ops::path_rel_to_repo(&repo_canon, &abs) {
                    Some(rel) => Some(rel),
                    None => {
                        return Self::tool_text(
                            "Error: path_filter must be inside the selected git working directory"
                                .to_string(),
                        );
                    }
                },
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };
        let limit = args.limit;
        let oneline = args.oneline;
        let out = tokio::task::spawn_blocking(move || {
            git_log_work(repo, limit, path_in_repo, oneline)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "git show for a single rev or object (e.g. HEAD~1, commit hash, or rev:path for a file blob). patch=false returns diffstat only; patch=true returns patch (truncated).",
        annotations(title = "Git show", read_only_hint = true)
    )]
    async fn git_show(
        &self,
        Parameters(args): Parameters<GitShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let rev = args.rev.clone();
        let patch = args.patch;
        let out = tokio::task::spawn_blocking(move || git_show_work(repo, rev, patch))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "All local and remote-tracking branches with last commit (git branch -a -v).",
        annotations(title = "Git branches", read_only_hint = true)
    )]
    async fn git_branches(
        &self,
        Parameters(args): Parameters<GitBranchesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_branches_work(repo))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "git blame for a line range (max 400 lines). Use to see who last changed lines.",
        annotations(title = "Git blame", read_only_hint = true)
    )]
    async fn git_blame(
        &self,
        Parameters(args): Parameters<GitBlameArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let repo_canon = repo.canonicalize().unwrap_or_else(|_| repo.clone());
        let abs_file = match self.resolve_path(&args.file) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let file_rel = match crate::ops::path_rel_to_repo(&repo_canon, &abs_file) {
            Some(r) => r,
            None => {
                return Self::tool_text(
                    "Error: file must be inside the selected git working directory".to_string(),
                );
            }
        };
        let start_line = args.start_line;
        let end_line = args.end_line;
        let rev = args.rev.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_blame_work(repo, file_rel, start_line, end_line, rev)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Resolve one or more refs to full SHAs (git rev-parse). Use for disambiguation or scripting follow-ups.",
        annotations(title = "Git rev-parse", read_only_hint = true)
    )]
    async fn git_rev_parse(
        &self,
        Parameters(args): Parameters<GitRevParseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let refs = args.refs.clone();
        let out = tokio::task::spawn_blocking(move || git_rev_parse_work(repo, refs))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read the last N lines of a text file with line numbers—ideal for logs without loading the whole file.",
        annotations(title = "Read file tail", read_only_hint = true)
    )]
    async fn read_file_tail(
        &self,
        Parameters(args): Parameters<ReadFileTailArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let last_lines = args.last_lines;
        let out = tokio::task::spawn_blocking(move || read_file_tail_work(path, last_lines))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Line-oriented stats: size, newline count in a scan window (full file if under cap), NUL detection, short UTF-8 preview—cheap sanity check before read_file.",
        annotations(title = "File line stats", read_only_hint = true)
    )]
    async fn file_stats(
        &self,
        Parameters(args): Parameters<FileStatsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || file_stats_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Find files whose relative path contains a substring (e.g. \"component\", \"test\"). Complements glob-based find_files.",
        annotations(title = "Find files by path substring", read_only_hint = true)
    )]
    async fn find_files_by_name(
        &self,
        Parameters(args): Parameters<FindFilesByNameArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let substring = args.substring.clone();
        let ci = args.case_insensitive;
        let max_f = args.max_files;
        let out = tokio::task::spawn_blocking(move || {
            find_files_by_name_work(search_root, substring, ci, max_f)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Parse a JSON file and return structure summary (object keys or array length). Fails with parse error details if invalid.",
        annotations(title = "Validate JSON file", read_only_hint = true)
    )]
    async fn json_validate_file(
        &self,
        Parameters(args): Parameters<ReadJsonArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || read_json_file_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "file_info for many paths in one round-trip (max 30). Paths must be under project root.",
        annotations(title = "Batch file metadata", read_only_hint = true)
    )]
    async fn batch_file_info(
        &self,
        Parameters(args): Parameters<BatchFileInfoArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut resolved = Vec::new();
        for p in &args.paths {
            match self.resolve_path(p) {
                Ok(pb) => resolved.push(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            }
        }
        let out = tokio::task::spawn_blocking(move || batch_file_info_work(resolved))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministically compress arbitrary text context into a bounded output (prompt budgeting). No external models—pure heuristics.",
        annotations(title = "Compress context", read_only_hint = true)
    )]
    async fn compress_context(
        &self,
        Parameters(args): Parameters<CompressContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(move || {
            compress_context_work(
                args.text,
                args.max_chars,
                args.mode,
                args.dedupe_lines,
                args.collapse_whitespace,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Extract line snippets around regex matches (optionally literal=true) and return only the relevant windows (bounded).",
        annotations(title = "Extract snippets", read_only_hint = true)
    )]
    async fn extract_snippets(
        &self,
        Parameters(args): Parameters<ExtractSnippetsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(move || {
            extract_snippets_work(
                args.text,
                args.pattern,
                args.literal,
                args.case_sensitive,
                args.context_lines,
                args.max_snippets,
                args.max_chars,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Compact/summarized view of the durable multi-agent bus: counts by sender/kind plus optional truncated recent message bodies.",
        annotations(title = "Agent bus: compact", read_only_hint = true)
    )]
    async fn agent_bus_compact(
        &self,
        Parameters(args): Parameters<AgentBusCompactArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let limit = args.limit;
        let show_bodies = args.show_bodies;
        let max_body_chars = args.max_body_chars;

        let res = tokio::task::spawn_blocking(move || {
            crate::bus::agent_bus_compact(root, limit, show_bodies, max_body_chars)
        })
        .await
        .unwrap_or_else(|e| Err(format!("task failed: {e}")));

        let out = match res {
            Ok(s) => s,
            Err(e) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
        };

        Self::tool_text(out)
    }

    #[tool(
        description = "Read a file (optionally a line slice) and deterministically compress it into a bounded context string.",
        annotations(title = "Compress file", read_only_hint = true)
    )]
    async fn compress_file(
        &self,
        Parameters(args): Parameters<CompressFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let out = tokio::task::spawn_blocking(move || {
            compress_file_work(
                path,
                args.start_line,
                args.end_line,
                args.max_chars,
                args.mode,
                args.dedupe_lines,
                args.collapse_whitespace,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Extract line snippets directly from a file, returning only windows around regex matches (bounded).",
        annotations(title = "Extract snippets from file", read_only_hint = true)
    )]
    async fn extract_snippets_from_file(
        &self,
        Parameters(args): Parameters<ExtractSnippetsFromFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let out = tokio::task::spawn_blocking(move || {
            extract_snippets_from_file_work(
                path,
                args.pattern,
                args.literal,
                args.case_sensitive,
                args.context_lines,
                args.max_snippets,
                args.max_chars,
                args.start_line,
                args.end_line,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Search the codebase with a regex and return snippet windows around matches (bounded).",
        annotations(title = "Grep snippets", read_only_hint = true)
    )]
    async fn grep_snippets(
        &self,
        Parameters(args): Parameters<GrepSnippetsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let out = tokio::task::spawn_blocking(move || {
            grep_snippets_work(
                search_root,
                args.pattern,
                args.literal,
                args.case_sensitive,
                args.respect_gitignore,
                args.context_lines,
                args.max_snippets,
                args.max_chars,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Search the durable multi-agent bus for messages whose body contains `needle`, with optional filters (from_agent/to_agent/kind/topic).",
        annotations(title = "Agent bus: search", read_only_hint = true)
    )]
    async fn agent_bus_search(
        &self,
        Parameters(args): Parameters<AgentBusSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let needle = args.needle;
        let case_sensitive = args.case_sensitive;
        let limit = args.limit;
        let show_bodies = args.show_bodies;
        let max_body_chars = args.max_body_chars;
        let from_agent = args.from_agent;
        let to_agent = args.to_agent;
        let kind = args.kind;
        let topic = args.topic;

        let res = tokio::task::spawn_blocking(move || {
            crate::bus::agent_bus_search(
                root,
                needle,
                case_sensitive,
                limit,
                show_bodies,
                max_body_chars,
                from_agent,
                to_agent,
                kind,
                topic,
            )
        })
        .await
        .unwrap_or_else(|e| Err(format!("task failed: {e}")));

        let out = match res {
            Ok(s) => s,
            Err(e) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
        };

        Self::tool_text(out)
    }

    #[tool(
        description = "Heuristic deterministic summarization of text (term-frequency sentence scoring). Useful for prompt budgeting without external models.",
        annotations(title = "Summarize text", read_only_hint = true)
    )]
    async fn summarize_text(
        &self,
        Parameters(args): Parameters<SummarizeTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(move || {
            summarize_text_work(args.text, args.max_chars, args.max_sentences)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Pack multiple context segments into one bounded string (preserves segment boundaries), then compress to max_chars.",
        annotations(title = "Pack context", read_only_hint = true)
    )]
    async fn pack_context(
        &self,
        Parameters(args): Parameters<PackContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(move || {
            pack_context_work(
                args.segments,
                args.max_chars,
                args.mode,
                args.dedupe_lines,
                args.collapse_whitespace,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Run git diff and extract snippet windows around regex matches within the diff output (bounded).",
        annotations(title = "Git diff snippets", read_only_hint = true)
    )]
    async fn git_diff_snippets(
        &self,
        Parameters(args): Parameters<GitDiffSnippetsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let out = tokio::task::spawn_blocking(move || {
            git_diff_snippets_work(
                dir,
                args.staged,
                args.pattern,
                args.literal,
                args.case_sensitive,
                args.context_lines,
                args.max_snippets,
                args.max_chars,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Append a JSONL message for other agents (flock-safe). Omit to_agent to broadcast; set topic to build a thread read via agent_thread. Mutates .mint-mcp/agent-bus.jsonl only.",
        annotations(title = "Agent bus: send", read_only_hint = false)
    )]
    async fn agent_send(
        &self,
        Parameters(args): Parameters<AgentSendArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let body = args.message.clone();
        let to = args.to_agent.clone();
        let topic = args.topic.clone();
        let kind = args.kind.clone();

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_send(root, from, body, to, topic, kind) {
            Ok(id) => format!("OK message_id={id}\nOther agents should poll `agent_inbox` or `agent_recent`."),
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "DMs to for_agent plus broadcasts (optionally hide your own broadcasts). Poll after agent_send from others.",
        annotations(title = "Agent bus: inbox", read_only_hint = true)
    )]
    async fn agent_inbox(
        &self,
        Parameters(args): Parameters<AgentInboxArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let for_a = args.for_agent.clone();
        let limit = args.limit;
        let skip = args.skip_own_broadcasts;

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_inbox(root, for_a, limit, skip) {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Ordered thread slice for one topic string (same key as in agent_send).",
        annotations(title = "Agent bus: thread", read_only_hint = true)
    )]
    async fn agent_thread(
        &self,
        Parameters(args): Parameters<AgentThreadArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let topic = args.topic.clone();
        let limit = args.limit;

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_thread(root, topic, limit) {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Last N messages on the whole bus—shared situational awareness across agents.",
        annotations(title = "Agent bus: recent", read_only_hint = true)
    )]
    async fn agent_recent(
        &self,
        Parameters(args): Parameters<AgentRecentArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let limit = args.limit;

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_recent(root, limit) {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Bus file path, size, approximate line count—sanity check before relying on the bus.",
        annotations(title = "Agent bus: status", read_only_hint = true)
    )]
    async fn agent_bus_status(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_bus_status(root) {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }
}

#[tool_handler]
impl ServerHandler for MintMcp {
    fn get_info(&self) -> ServerInfo {
        let root_hint = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone())
            .display()
            .to_string();

        ServerInfo::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(
            Implementation::new("mint-mcp-server", env!("CARGO_PKG_VERSION"))
                .with_title("Mint MCP — codebase + multi-agent bus")
                .with_description(
                    "Project-scoped read-only file and git tools, plus a durable JSONL bus under .mint-mcp/ for multi-agent coordination. Designed for LLM agents: strong schemas, explicit limits, and MCP is_error on failures.",
                ),
        )
        .with_instructions(format!(
            "Workspace root: {root_hint} (env: MINT_PROJECT_ROOT or PROJECT_ROOT; all paths must stay inside this directory).\n\
             \n\
             Suggested workflow for coding agents:\n\
             1) workspace_context — root, git branch, top-level layout, bus presence, and tool limits in one call.\n\
             2) project_tree, find_files, find_files_by_name, and grep_code to explore without reading everything.\n\
             3) file_stats or file_info before large reads; read_file_tail for logs; read_file with line range for source.\n\
             4) json_validate_file for config/manifest sanity; batch_file_info for many paths at once.\n\
             5) git_status, git_diff, git_log, git_show, git_blame, git_branches, git_rev_parse for version control.\n\
             \n\
             Multi-agent: pick a stable from_agent id per session. agent_send without to_agent broadcasts; with to_agent is a DM. \
             Reuse the same topic string on agent_send for a thread, then agent_thread(topic). agent_inbox(for_agent) and agent_recent for polling.\n\
             Bus path: .mint-mcp/agent-bus.jsonl — add .mint-mcp/ to .gitignore if you do not want it in version control.\n\
             \n\
             Tools: workspace_context, read_file, read_file_tail, list_files, grep_code, find_files, find_files_by_name, \
             project_tree, file_info, file_stats, batch_file_info, json_validate_file, \
             git_status, git_diff, git_log, git_show, git_branches, git_blame, git_rev_parse, \
             compress_context, compress_file, extract_snippets, extract_snippets_from_file, grep_snippets, \
             summarize_text, pack_context, git_diff_snippets, \
             agent_bus_compact, agent_bus_search, \
             agent_send, agent_inbox, agent_thread, agent_recent, agent_bus_status."
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = (stdin(), stdout());
    let server = MintMcp::new().serve(transport).await?;
    server.waiting().await?;
    Ok(())
}
