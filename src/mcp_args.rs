// MCP tool argument structs and default-value functions.
//
// Extracted from main.rs for maintainability. This file is included at
// module scope via `include!("mcp_args.rs")` in main.rs.
//
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are in scope from the
// parent (main.rs) imports — do NOT add duplicate `use` statements here.

// ============================================================================
// Argument structs
// ============================================================================

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PathArgs {
    #[schemars(
        description = "Directory path relative to project root, or absolute under root. Empty string = project root."
    )]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReadFileArgs {
    #[schemars(description = "UTF-8 text file path under the project root.")]
    path: String,
    #[schemars(
        description = "1-based start line (inclusive) for a numbered slice; omit for whole file (subject to size limits)."
    )]
    #[serde(default)]
    start_line: Option<usize>,
    #[schemars(
        description = "1-based end line (inclusive). With start_line only, defaults to end of loaded content."
    )]
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
    #[schemars(
        description = "Stable sender id for this session (alphanumeric, _, -, .) e.g. cursor-main, review-bot."
    )]
    from_agent: String,
    #[schemars(description = "Message body; truncated if extremely long.")]
    message: String,
    #[schemars(
        description = "Recipient id for a DM; omit to broadcast to every agent polling the bus."
    )]
    #[serde(default)]
    to_agent: Option<String>,
    #[schemars(
        description = "Thread key; use the same string on every agent_send in a thread, then read with agent_thread."
    )]
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
    #[schemars(
        description = "If set, only messages with ts_ms strictly greater than this value (incremental polling; store last seen ts from output headers)."
    )]
    #[serde(default)]
    since_ts_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentThreadArgs {
    #[schemars(description = "Exact topic string shared across agent_send calls.")]
    topic: String,
    #[schemars(description = "Max messages in the thread window (1–200).")]
    #[serde(default = "default_thread_limit")]
    limit: usize,
    #[schemars(
        description = "If set, only messages with ts_ms >= this value (slice the tail of the thread after this time)."
    )]
    #[serde(default)]
    since_ts_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentRecentArgs {
    #[schemars(description = "Tail size of the global bus (1–150), after optional since filter.")]
    #[serde(default = "default_recent_limit")]
    limit: usize,
    #[schemars(
        description = "If set, only messages newer than this ts_ms are eligible, then take the last `limit` of those."
    )]
    #[serde(default)]
    since_ts_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentListTopicsArgs {
    #[schemars(description = "Max topic rows to return (1–200), ordered by most recent activity.")]
    #[serde(default = "default_list_topics_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentGetMessageArgs {
    #[schemars(
        description = "Exact message id from agent_send OK line or bus listing (e.g. 1775308370692-144329)."
    )]
    message_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentInboxWaitArgs {
    #[schemars(description = "Same stable id as agent_inbox.")]
    for_agent: String,
    #[serde(default = "default_inbox_limit")]
    limit: usize,
    #[serde(default = "default_true")]
    skip_own_broadcasts: bool,
    #[serde(default)]
    since_ts_ms: Option<u64>,
    #[schemars(
        description = "Max time for this single MCP call (100–120000ms). Server sleeps between polls—async agents prefer agent_inbox + since_ts_ms instead."
    )]
    #[serde(default = "default_inbox_wait_timeout_ms")]
    timeout_ms: u64,
    #[schemars(description = "Delay between inbox polls (50–2000ms).")]
    #[serde(default = "default_inbox_wait_poll_ms")]
    poll_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentTopicWaitArgs {
    #[schemars(description = "Thread key to watch (same as agent_send topic).")]
    topic: String,
    #[schemars(
        description = "Your from_agent id; their posts are ignored so you wait for other agents."
    )]
    exclude_from_agent: String,
    #[schemars(
        description = "Only consider messages with ts_ms >= this. Omit to use server time at call start (wait for new replies only)."
    )]
    #[serde(default)]
    since_ts_ms: Option<u64>,
    #[serde(default = "default_inbox_wait_timeout_ms")]
    timeout_ms: u64,
    #[serde(default = "default_inbox_wait_poll_ms")]
    poll_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentFanOutArgs {
    from_agent: String,
    #[schemars(
        description = "Identical body for each DM (parallel fan-out: one exclusive lock, many lines)."
    )]
    message: String,
    #[schemars(
        description = "Recipient agent ids (max 32, deduped); each must differ from from_agent."
    )]
    to_agents: Vec<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentPipelineStep {
    message: String,
    #[serde(default)]
    to_agent: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentSendPipelineArgs {
    from_agent: String,
    #[schemars(
        description = "Ordered steps (max 24). Serial: rows appended in sequence under one lock (handoff / multi-part)."
    )]
    steps: Vec<AgentPipelineStep>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentMultiInboxArgs {
    #[schemars(
        description = "Up to 16 agent ids; one shared bus read, sections per id (parallel fan-in)."
    )]
    for_agents: Vec<String>,
    #[serde(default = "default_inbox_limit")]
    limit: usize,
    #[serde(default = "default_true")]
    skip_own_broadcasts: bool,
    #[serde(default)]
    since_ts_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentCoordinationSnapshotArgs {
    #[serde(default = "default_snapshot_topics_limit")]
    topics_limit: usize,
    #[serde(default = "default_snapshot_recent_limit")]
    recent_limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct EmptyArgs {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitLogArgs {
    #[schemars(
        description = "Git repo directory (subfolder of project root); omit for project root."
    )]
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "Max commits (1–150).")]
    #[serde(default = "default_git_log_limit")]
    limit: usize,
    #[serde(default = "default_true")]
    oneline: bool,
    #[schemars(
        description = "Only history touching this path, relative to project root (e.g. src/main.rs)."
    )]
    #[serde(default)]
    path_filter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitShowArgs {
    #[serde(default)]
    path: Option<String>,
    #[schemars(
        description = "Commit-ish or object (e.g. HEAD, main~1, abc123, or rev:path for a blob)."
    )]
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
    #[schemars(
        description = "File path; counts newlines in a scan window and shows a short preview."
    )]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FindFilesByNameArgs {
    #[schemars(
        description = "Substring matched against each file path relative to search root (e.g. \"test\", \"Cargo\")."
    )]
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
    #[schemars(description = "truncate | head_tail | lines_only | dense (see compress_context).")]
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
    #[schemars(
        description = "1-based start line (inclusive) for a numbered slice; omit for whole file (loaded chunk)."
    )]
    #[serde(default)]
    start_line: Option<usize>,
    #[schemars(
        description = "1-based end line (inclusive). With start_line only, defaults to end of loaded content."
    )]
    #[serde(default)]
    end_line: Option<usize>,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "truncate | head_tail | lines_only | dense (see compress_context).")]
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
    #[schemars(
        description = "Input text to summarize (deterministic TF-style extractive summary; no external models)."
    )]
    text: String,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_summarize_max_chars")]
    max_chars: usize,
    #[schemars(description = "Max sentences kept (reordered by salience; cap keeps output safe).")]
    #[serde(default = "default_summarize_max_sentences")]
    max_sentences: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeFileArgs {
    #[schemars(description = "UTF-8 file path under project root.")]
    path: String,
    #[schemars(description = "Optional 1-based line range (same as read_file).")]
    #[serde(default)]
    start_line: Option<usize>,
    #[serde(default)]
    end_line: Option<usize>,
    #[serde(default = "default_summarize_max_chars")]
    max_chars: usize,
    #[serde(default = "default_summarize_max_sentences")]
    max_sentences: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseIndexArgs {
    #[schemars(description = "Subtree to scan; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Stop after this many files (200–20000) for speed.")]
    #[serde(default = "default_codebase_max_files")]
    max_files_scan: usize,
    #[schemars(
        description = "If true, collect heuristic definition-like lines (Rust/TS/Py/Go…); bounded."
    )]
    #[serde(default)]
    symbol_sketch: bool,
    #[schemars(description = "Max symbol sketch lines (0–300).")]
    #[serde(default = "default_codebase_max_symbols")]
    max_symbol_entries: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ConvertFormatArgs {
    #[schemars(
        description = "json_pretty | json_minify | base64/hex | utf8_bom_strip | crlf/lf | tabs_to_spaces | strip_ansi | collapse_blank_lines | remove_empty_lines | slugify | c_string_escape | prefix_line_numbers | text_lowercase | text_uppercase | sort_lines | reverse_lines | trim_each_line | unique_sorted_lines | wrap_lines | percent_encode | percent_decode (url_* aliases)"
    )]
    operation: String,
    #[schemars(description = "Inline payload: JSON text, base64, or hex. For decode operations.")]
    #[serde(default)]
    text: Option<String>,
    #[schemars(
        description = "Read input file (encode) or required for some JSON-from-file flows; max 256 KiB."
    )]
    #[serde(default)]
    input_path: Option<String>,
    #[schemars(
        description = "Write binary output for base64_decode / hex_decode (must stay under project root)."
    )]
    #[serde(default)]
    output_path: Option<String>,
    #[schemars(description = "Tab stop width for tabs_to_spaces (1–16).")]
    #[serde(default = "default_tab_stops")]
    tab_stops: usize,
    #[schemars(description = "Starting line number for prefix_line_numbers (1-based).")]
    #[serde(default = "default_line_number_start")]
    line_number_start: usize,
    #[schemars(description = "Target column width for wrap_lines / hard_wrap_lines (8–200).")]
    #[serde(default = "default_wrap_width")]
    wrap_width: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ServerSleepArgs {
    #[schemars(
        description = "Block this MCP worker (0–30000ms requested; capped lower on MINT_NETWORK_PROFILE=public). Prefer agent_inbox_wait / agent_topic_wait for coordination."
    )]
    #[serde(default = "default_sleep_ms")]
    duration_ms: u64,
    #[schemars(
        description = "Extra random delay 0..=jitter_ms_max (capped by profile sleep cap) to desynchronize agents; 0 disables."
    )]
    #[serde(default)]
    jitter_ms_max: u64,
    #[schemars(
        description = "Optional short label echoed in output for agent logs (max ~200 chars, newlines stripped)."
    )]
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TerminalWhichArgs {
    #[schemars(
        description = "Bare executable name (same allowlist as terminal_run). Resolves via which (Unix) or where (Windows)."
    )]
    program: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TerminalRunArgs {
    #[schemars(
        description = "Bare executable name only (e.g. git, cargo). No paths. Requires MINT_TERMINAL_ENABLE=1 and allowlist MINT_TERMINAL_ALLOW. On trusted/local deployments the child may read/write anything your argv allows (e.g. file editors, package installs); keep cwd under the project root."
    )]
    program: String,
    #[schemars(description = "Argv after program name (no shell interpolation).")]
    #[serde(default)]
    args: Vec<String>,
    #[schemars(description = "Working directory under project root; omit for project root.")]
    #[serde(default)]
    cwd: Option<String>,
    #[schemars(description = "Wall-clock cap in ms (clamped 1–120000).")]
    #[serde(default = "default_terminal_timeout_ms")]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TerminalRunStdinArgs {
    #[schemars(description = "Bare executable name (same allowlist as terminal_run).")]
    program: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default = "default_terminal_timeout_ms")]
    timeout_ms: u64,
    #[schemars(description = "UTF-8 stdin body; max 512 KiB. Omit if stdin_path is set.")]
    #[serde(default)]
    stdin_text: Option<String>,
    #[schemars(
        description = "Project-relative file read as raw stdin bytes; max 512 KiB. Omit if stdin_text is set."
    )]
    #[serde(default)]
    stdin_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ProjectCargoTestArgs {
    #[schemars(
        description = "Working directory under project root (Rust package / workspace root); empty = project root."
    )]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Cargo package name for `-p`; omit for default package.")]
    #[serde(default)]
    package: Option<String>,
    #[schemars(description = "Optional test name filter (cargo test <filter>).")]
    #[serde(default)]
    test_filter: Option<String>,
    #[serde(default)]
    no_run: bool,
    #[serde(default)]
    workspace: bool,
    #[schemars(description = "Parallel job count for cargo (1–32).")]
    #[serde(default)]
    jobs: Option<u32>,
    #[serde(default)]
    nocapture: bool,
    #[serde(default = "default_project_test_timeout_ms")]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ProjectCargoCheckArgs {
    #[serde(default)]
    directory: String,
    #[serde(default)]
    package: Option<String>,
    #[serde(default)]
    workspace: bool,
    #[serde(default = "default_project_test_timeout_ms")]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ProjectPytestArgs {
    #[schemars(description = "Working directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Path segment(s) passed to pytest (e.g. tests or .); no `..`.")]
    #[serde(default)]
    test_path: String,
    #[serde(default)]
    quiet: bool,
    #[schemars(description = "pytest -k expression (restricted charset).")]
    #[serde(default)]
    keyword: Option<String>,
    #[serde(default)]
    maxfail: Option<u32>,
    #[serde(default = "default_project_test_timeout_ms")]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GhPrListArgs {
    #[schemars(description = "Git repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[serde(default = "default_gh_pr_limit")]
    limit: usize,
    #[schemars(description = "open | closed | merged | all")]
    #[serde(default = "default_gh_pr_state")]
    state: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GhPrViewArgs {
    #[schemars(description = "Git repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Pull request number (digits only).")]
    pr_number: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GlabMrListArgs {
    #[schemars(description = "Git repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[serde(default = "default_gh_pr_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GlabMrShowArgs {
    #[schemars(description = "Git repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Merge request IID (digits only).")]
    iid: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitStashListArgs {
    #[schemars(description = "Git repo path under project root; omit for project root.")]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitWorktreeListArgs {
    #[schemars(description = "Git repo path under project root; omit for project root.")]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitCheckoutArgs {
    #[schemars(description = "Git repo path under project root; omit for project root.")]
    path: Option<String>,
    #[schemars(
        description = "Branch, tag, or commit ref for `git checkout`. Requires MINT_GIT_WRITE_ENABLE=1."
    )]
    target: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitStashPushArgs {
    #[schemars(description = "Git repo path under project root; omit for project root.")]
    path: Option<String>,
    #[schemars(
        description = "Optional stash message for `git stash push -m`. Requires MINT_GIT_WRITE_ENABLE=1."
    )]
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressParagraphsArgs {
    #[schemars(description = "Input text; paragraphs are blocks separated by blank lines.")]
    text: String,
    #[schemars(description = "Max non-empty paragraphs to keep (server clamps 1–200).")]
    #[serde(default = "default_compress_paragraphs_max")]
    max_paragraphs: usize,
    #[schemars(description = "Max output characters after paragraph selection.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseManifestArgs {
    #[schemars(
        description = "Subtree root to walk; empty = project root. Writes .mint-mcp/file-manifest.jsonl under that root."
    )]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max file lines in manifest (server clamps 100–25000).")]
    #[serde(default = "default_manifest_max_files")]
    max_files: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeSegmentsArgs {
    #[schemars(
        description = "Parallel text chunks merged then summarized (max 24 segments per call)."
    )]
    segments: Vec<String>,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_summarize_max_chars")]
    max_chars: usize,
    #[schemars(description = "Max sentences kept in extractive summary.")]
    #[serde(default = "default_summarize_max_sentences")]
    max_sentences: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentSubagentTaskArgs {
    #[schemars(description = "Coordinator / parent agent id (sender).")]
    from_agent: String,
    #[schemars(description = "Sub-agent recipient id (DM); must differ from from_agent.")]
    sub_agent: String,
    #[schemars(description = "Short task title for inbox scanning.")]
    title: String,
    #[schemars(description = "Full task instructions / payload.")]
    task_body: String,
    #[schemars(description = "Optional thread key (same as agent_send topic).")]
    #[serde(default)]
    topic: Option<String>,
    #[schemars(description = "Optional correlation id for tracing across agents.")]
    #[serde(default)]
    correlation_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentSubagentDoneArgs {
    #[schemars(
        description = "Sub-agent id sending the completion (must match recipient of the original task)."
    )]
    from_agent: String,
    #[schemars(description = "Coordinator / parent agent id (DM recipient).")]
    parent_agent: String,
    #[schemars(description = "Result summary, errors, or handoff notes.")]
    result_body: String,
    #[schemars(description = "Optional thread key matching the task.")]
    #[serde(default)]
    topic: Option<String>,
    #[schemars(description = "Optional correlation id from the original subagent_task.")]
    #[serde(default)]
    correlation_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressUniqueLinesArgs {
    #[schemars(
        description = "Input text; keeps first occurrence of each line in order, then caps."
    )]
    text: String,
    #[schemars(description = "Max unique lines to keep (server clamps).")]
    #[serde(default = "default_unique_lines_max")]
    max_lines: usize,
    #[schemars(description = "Max output characters after join.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressLinesHeadArgs {
    #[schemars(description = "Input text; keeps the first N lines in order (no deduplication).")]
    text: String,
    #[schemars(description = "Max lines to keep from the start (server clamps 1–10000).")]
    #[serde(default = "default_unique_lines_max")]
    head_lines: usize,
    #[schemars(description = "Max output characters after join.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressLinesTailArgs {
    #[schemars(description = "Input text; keeps the last N lines in order.")]
    text: String,
    #[schemars(description = "Max lines to keep from the end (server clamps 1–10000).")]
    #[serde(default = "default_unique_lines_max")]
    tail_lines: usize,
    #[schemars(description = "Max output characters after join.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressCollapseBlankLinesArgs {
    #[schemars(description = "Input text; collapses runs of blank lines to a single blank line.")]
    text: String,
    #[schemars(description = "Max output characters.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressTruncateMiddleArgs {
    #[schemars(description = "Input text; keeps start and end when over max_chars.")]
    text: String,
    #[schemars(description = "Max output characters (server clamps 400–MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "Characters to keep at each end when truncating (server clamps).")]
    #[serde(default = "default_truncate_middle_margin")]
    margin_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressDedentArgs {
    #[schemars(description = "Pasted code / log block with uniform leading indent.")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeBulletsArgs {
    #[schemars(
        description = "Source text (e.g. markdown); extracts bullet / numbered list lines in order."
    )]
    text: String,
    #[schemars(description = "Max bullet lines to keep.")]
    #[serde(default = "default_summarize_bullets_max")]
    max_items: usize,
    #[schemars(description = "Max output characters.")]
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeHeadingsArgs {
    #[schemars(description = "Markdown source; extracts ATX headings (# .. ######).")]
    text: String,
    #[serde(default = "default_summarize_headings_max")]
    max_headings: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeMarkdownTocArgs {
    #[schemars(description = "Markdown source; builds indented bullet TOC from ATX headings.")]
    text: String,
    #[serde(default = "default_summarize_headings_max")]
    max_entries: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressGzipBase64Args {
    #[schemars(description = "UTF-8 text to gzip; max 256 KiB. Omit if input_path is set.")]
    #[serde(default)]
    text: Option<String>,
    #[schemars(description = "Project file to gzip; max 256 KiB read. Omit if text is set.")]
    #[serde(default)]
    input_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DecompressGzipBase64Args {
    #[schemars(description = "Standard base64 of a gzip stream (e.g. from compress_gzip_base64).")]
    payload_base64: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanRiskRegisterArgs {
    #[schemars(description = "Short risk titles (technical, security, schedule, …).")]
    risks: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanDefinitionOfDoneArgs {
    #[schemars(description = "Observable outcomes / acceptance bullets.")]
    outcomes: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanTestStrategyArgs {
    #[schemars(description = "Feature, crate, or service under test.")]
    #[serde(default)]
    scope: String,
    #[schemars(description = "Risky or high-churn areas to prioritize.")]
    #[serde(default)]
    risk_hotspots: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReasoningPremortemArgs {
    #[schemars(description = "Initiative or decision under review.")]
    #[serde(default)]
    project: String,
    #[schemars(description = "Time horizon for the imagined failure (e.g. next release).")]
    #[serde(default)]
    timeframe: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReasoningFiveWhysArgs {
    #[schemars(description = "Observed bug, failure, or symptom to ladder with five whys.")]
    #[serde(default)]
    seed_problem: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanRetrospectiveArgs {
    #[schemars(description = "Sprint, PR, or agent batch label for the retro window.")]
    #[serde(default)]
    iteration: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressSimplifyWhitespaceArgs {
    #[schemars(description = "Multiline text to simplify per-line.")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressMarkdownLinksArgs {
    #[schemars(description = "Markdown text; strips link targets, keeps visible label text.")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressUnifiedDiffStatsArgs {
    #[schemars(description = "Unified diff text (e.g. git diff output).")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressTruncateLinesArgs {
    #[schemars(
        description = "Multiline text; each line capped to max_line_chars Unicode scalars (suffix …)."
    )]
    text: String,
    #[schemars(description = "Max Unicode scalars per line before ellipsis (clamped 16–512).")]
    #[serde(default = "default_truncate_line_width")]
    max_line_chars: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeDiffFileListArgs {
    #[schemars(description = "Unified diff text with --- / +++ path headers (e.g. git diff).")]
    text: String,
    #[schemars(description = "Max distinct paths to list after sorting (clamped 4–500).")]
    #[serde(default = "default_diff_file_list_max_paths")]
    max_paths: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeDelimitedHeaderArgs {
    #[schemars(description = "File or paste: first line is the header row.")]
    text: String,
    #[schemars(description = "comma | tab | pipe | semicolon")]
    delimiter: String,
    #[serde(default = "default_delimited_max_columns")]
    max_columns: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeJsonStructureArgs {
    #[schemars(description = "Raw JSON document (object or array at root).")]
    text: String,
    #[schemars(
        description = "Max recursion depth for nested objects (0 = types at current level only)."
    )]
    #[serde(default = "default_json_structure_depth")]
    max_depth: usize,
    #[schemars(description = "Max object keys enumerated per nesting level.")]
    #[serde(default = "default_json_structure_keys")]
    max_keys_per_level: usize,
    #[schemars(description = "Max summary lines to emit (budget).")]
    #[serde(default = "default_json_structure_lines")]
    max_lines: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TextLevenshteinArgs {
    #[schemars(description = "First string (Unicode scalars).")]
    a: String,
    #[schemars(description = "Second string.")]
    b: String,
    #[schemars(description = "Max characters taken from each side (32–2500; caps CPU).")]
    #[serde(default = "default_levenshtein_max_len")]
    max_len_each: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TextTokenJaccardArgs {
    #[schemars(description = "First text (whitespace tokenized).")]
    text_a: String,
    #[schemars(description = "Second text.")]
    text_b: String,
    #[schemars(
        description = "Lowercase tokens before comparing (recommended for natural language)."
    )]
    #[serde(default = "default_true")]
    fold_case: bool,
    #[schemars(description = "Max Unicode scalars read per side (256–500000).")]
    #[serde(default = "default_token_jaccard_max_chars")]
    max_chars_each: usize,
    #[schemars(
        description = "Max whitespace tokens processed per side before stopping (50–200000); if EOF not reached, jaccard is approximate."
    )]
    #[serde(default = "default_token_jaccard_max_scan")]
    max_tokens_scanned_per_text: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TextDjb2HashArgs {
    #[schemars(description = "Text to fingerprint (truncated at server response cap).")]
    text: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeFirstLinesArgs {
    #[schemars(description = "Any text; first N non-empty lines.")]
    text: String,
    #[serde(default = "default_summarize_first_lines_count")]
    line_count: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseWordIndexArgs {
    #[schemars(description = "Subtree to sample; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max files to sample (50–5000).")]
    #[serde(default = "default_word_index_max_files")]
    max_files: usize,
    #[schemars(description = "Top token counts to return (10–150).")]
    #[serde(default = "default_word_index_top")]
    top_n: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseShallowHashArgs {
    #[schemars(description = "Subtree to walk; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max files to hash (50–8000).")]
    #[serde(default = "default_shallow_hash_max_files")]
    max_files: usize,
    #[schemars(description = "Bytes read per file for SHA-256 (256–65536).")]
    #[serde(default = "default_shallow_hash_sample")]
    sample_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebasePathListArgs {
    #[schemars(description = "Subtree to walk; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max file paths to return (200–50000).")]
    #[serde(default = "default_path_list_max_paths")]
    max_paths: usize,
    #[schemars(
        description = "If set, skip files deeper than this many path segments (relative to path)."
    )]
    #[serde(default)]
    max_depth: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseLargestFilesArgs {
    #[schemars(description = "Subtree to scan; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max files to stat for ranking (200–50000).")]
    #[serde(default = "default_largest_scan_files")]
    max_files_scan: usize,
    #[schemars(description = "How many largest files to return (10–500).")]
    #[serde(default = "default_largest_top_n")]
    top_n: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseDuplicateBasenamesArgs {
    #[schemars(description = "Subtree to scan; empty = project root.")]
    #[serde(default)]
    path: String,
    #[schemars(description = "Max files to visit when grouping basenames (500–35000).")]
    #[serde(default = "default_dup_basenames_scan")]
    max_files_scan: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexBuildArgs {
    #[schemars(
        description = "Subtree to scan for symbols; empty = project root. Writes project_root/.mint-mcp/code-index.json."
    )]
    #[serde(default)]
    path: String,
    #[serde(default = "default_true")]
    respect_gitignore: bool,
    #[schemars(description = "Max files to visit (server clamps).")]
    #[serde(default = "default_code_index_max_files")]
    max_files: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexSearchArgs {
    #[schemars(description = "Substring or regex (see use_regex) against paths and symbol names.")]
    needle: String,
    #[serde(default = "default_true")]
    case_sensitive: bool,
    #[schemars(description = "Max matching files to list (1–200).")]
    #[serde(default = "default_code_index_search_limit")]
    limit: usize,
    #[schemars(description = "If set, only paths starting with this relative prefix.")]
    #[serde(default)]
    path_prefix: Option<String>,
    #[schemars(
        description = "If true, needle is a Rust regex; case_sensitive maps to case_insensitive flag on the regex."
    )]
    #[serde(default)]
    use_regex: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TextStatsArgs {
    #[schemars(description = "UTF-8 text to measure before compress/summarize.")]
    text: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeTokenEstimateArgs {
    #[schemars(
        description = "UTF-8 text for deterministic token budget heuristics (not a real BPE count)."
    )]
    text: String,
    #[serde(default = "default_summarize_token_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PrivacyRedactTextArgs {
    #[schemars(description = "UTF-8 source text.")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressWsCollapseArgs {
    #[schemars(
        description = "Source text; each line gets internal whitespace collapsed to single spaces."
    )]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeTopTermsArgs {
    #[schemars(
        description = "Source prose or logs for bag-of-words counts (English stopwords removed)."
    )]
    text: String,
    #[schemars(description = "Minimum token length (2–32).")]
    #[serde(default = "default_top_terms_min_len")]
    min_token_len: usize,
    #[schemars(description = "Max distinct terms to list (1–80).")]
    #[serde(default = "default_top_terms_max")]
    max_terms: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeIdentifierFrequencyArgs {
    #[schemars(
        description = "Source code or logs; counts tokens matching [A-Za-z_][A-Za-z0-9_]*."
    )]
    text: String,
    #[schemars(description = "Minimum identifier length (1–64).")]
    #[serde(default = "default_identifier_min_len")]
    min_len: usize,
    #[schemars(description = "Max distinct identifiers to list (1–120).")]
    #[serde(default = "default_identifier_max_list")]
    max_identifiers: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "Lowercase identifiers before counting (merge Foo and foo).")]
    #[serde(default)]
    fold_case: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressDoubleSlashLineCommentsArgs {
    #[schemars(description = "Text with optional full-line // comments (Rust/C/JS style).")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
    #[schemars(description = "Also remove empty/whitespace-only lines after stripping comments.")]
    #[serde(default = "default_true")]
    drop_blank_lines: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TextLongestCommonSubstringArgs {
    #[schemars(description = "First string (Unicode scalars).")]
    a: String,
    #[schemars(description = "Second string.")]
    b: String,
    #[schemars(description = "Max characters taken from each side (32–4000); caps n×m work.")]
    #[serde(default = "default_lcss_max_len")]
    max_len_each: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressStripTagsArgs {
    #[schemars(
        description = "HTML/XML-ish text; strips tags that look like markup (<!…, <?…, </…, <letter…>)."
    )]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeDuplicateLinesArgs {
    #[schemars(description = "Multiline text (e.g. logs).")]
    text: String,
    #[schemars(description = "Minimum occurrences to list a line (≥2).")]
    #[serde(default = "default_dup_lines_min")]
    min_occurrences: usize,
    #[schemars(description = "Max distinct repeated lines to return.")]
    #[serde(default = "default_dup_lines_max_entries")]
    max_entries: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressFencedCodeBlocksArgs {
    #[schemars(
        description = "Source text; ```fenced``` blocks become a single placeholder line each."
    )]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeLineHistogramArgs {
    #[schemars(description = "Multiline text to bucket by Unicode line length.")]
    text: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct FormatPrefixLinesArgs {
    #[schemars(description = "Lines to prefix (e.g. quote with '> ').")]
    text: String,
    #[schemars(description = "String prepended to every line; empty defaults to '> '.")]
    #[serde(default)]
    prefix: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodebaseCargoDepsArgs {
    #[schemars(
        description = "Cargo.toml path under project root; empty = ./Cargo.toml at project root."
    )]
    #[serde(default)]
    manifest_path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexPathsPrefixArgs {
    #[schemars(
        description = "Relative path prefix (forward slashes, e.g. src/); empty matches all indexed paths."
    )]
    #[serde(default)]
    prefix: String,
    #[serde(default = "default_code_index_paths_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexAnalyticsLimitArgs {
    #[schemars(
        description = "Max rows to return (extensions or symbol names, depending on tool)."
    )]
    #[serde(default = "default_code_index_analytics_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ThinkingPlanArgs {
    #[schemars(description = "Primary goal or task statement.")]
    objective: String,
    #[schemars(description = "Constraints or invariants (one string per bullet).")]
    #[serde(default)]
    constraints: Vec<String>,
    #[schemars(description = "Already-known steps or milestones.")]
    #[serde(default)]
    known_steps: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ThinkingStepBackArgs {
    #[schemars(description = "Problem or situation to reframe.")]
    problem: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ThinkingAssumptionArgs {
    #[schemars(description = "Claims or assumptions to audit (one per entry).")]
    claims: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ThinkingSessionAppendArgs {
    #[schemars(
        description = "Single-line note (newlines collapsed); appended to .mint-mcp/agent-thinking.log on trusted deployments."
    )]
    note: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ThinkingSessionReadArgs {
    #[schemars(description = "How many trailing log lines to return (1–500).")]
    #[serde(default = "default_thinking_tail_lines")]
    tail_lines: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanExecutionChecklistArgs {
    #[schemars(description = "Ordered milestone strings (design, implement, verify, …).")]
    milestones: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanHandoffBriefArgs {
    #[schemars(description = "Short title for the delegated task.")]
    title: String,
    #[schemars(description = "What is in / out of scope.")]
    scope: String,
    #[schemars(description = "Concrete deliverables.")]
    #[serde(default)]
    deliverables: Vec<String>,
    #[schemars(description = "Acceptance criteria or verification steps.")]
    #[serde(default)]
    acceptance: Vec<String>,
    #[schemars(description = "Known blockers or upstream dependencies.")]
    #[serde(default)]
    blockers: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PlanWorkBucketsArgs {
    #[schemars(description = "Work items to split round-robin across buckets.")]
    items: Vec<String>,
    #[schemars(description = "Number of parallel buckets (2–16).")]
    #[serde(default = "default_plan_buckets")]
    buckets: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReasoningChainArgs {
    #[schemars(description = "Facts or premises (cite tools where possible).")]
    #[serde(default)]
    premises: Vec<String>,
    #[schemars(description = "Reasoning steps between premises and conclusion.")]
    #[serde(default)]
    steps: Vec<String>,
    #[schemars(description = "Decision or question still open after the chain.")]
    open_question: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReasoningDevilsAdvocateArgs {
    #[schemars(description = "Thesis to challenge.")]
    claim: String,
    #[schemars(description = "Optional background.")]
    #[serde(default)]
    context: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReasoningEvidenceMatrixArgs {
    #[schemars(description = "Atomic claims to map to evidence.")]
    claims: Vec<String>,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WorkspaceTempWriteArgs {
    #[schemars(
        description = "Filename only (ASCII alnum, -, _, .); stored under .mint-mcp/agent-temp/."
    )]
    name: String,
    #[schemars(description = "UTF-8 text body.")]
    content: String,
    #[schemars(description = "Max bytes to store (clamped 1..=512KiB).")]
    #[serde(default = "default_temp_max_store_bytes")]
    max_store_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WorkspaceTempReadArgs {
    #[schemars(description = "Filename in .mint-mcp/agent-temp/.")]
    name: String,
    #[schemars(description = "Max bytes read from disk for this call.")]
    #[serde(default = "default_temp_max_read_bytes")]
    max_read_bytes: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WorkspaceTempListArgs {
    #[serde(default = "default_workspace_temp_list_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WorkspaceTempDeleteArgs {
    #[schemars(description = "Filename in .mint-mcp/agent-temp/.")]
    name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ScreenshotCaptureArgs {
    #[schemars(description = "grim (Wayland) | screencapture (macOS full screen).")]
    backend: String,
    #[schemars(
        description = "Basename ending in .png (e.g. ui-1.png); written under .mint-mcp/screenshots/."
    )]
    output_name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressDropLinesRegexArgs {
    #[schemars(description = "Multiline text.")]
    text: String,
    #[schemars(description = "Regex; lines matching anywhere on the line are removed.")]
    pattern: String,
    #[serde(default)]
    case_insensitive: bool,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexPathSubstringArgs {
    #[schemars(description = "Substring to find in indexed relative paths.")]
    needle: String,
    #[serde(default = "default_code_index_paths_limit")]
    limit: usize,
    #[serde(default)]
    case_sensitive: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexSymbolPrefixArgs {
    #[schemars(description = "Symbol name prefix (e.g. `Foo` for FooBar, Foo::).")]
    prefix: String,
    #[serde(default)]
    case_sensitive: bool,
    #[serde(default = "default_code_index_paths_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexDuplicateSymbolsArgs {
    #[serde(default = "default_duplicate_symbols_min_files")]
    min_files: usize,
    #[serde(default = "default_code_index_paths_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexPathsWithoutSymbolsArgs {
    #[serde(default = "default_code_index_paths_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeLinePrefixGroupsArgs {
    #[schemars(description = "Multiline text (e.g. logs).")]
    text: String,
    #[schemars(description = "Prefix length in Unicode scalars (1–96).")]
    #[serde(default = "default_line_prefix_len")]
    prefix_len: usize,
    #[schemars(description = "Max distinct prefixes to list.")]
    #[serde(default = "default_line_prefix_max_groups")]
    max_groups: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeRankLinesByKeywordsArgs {
    #[schemars(description = "Multiline text to scan.")]
    text: String,
    #[schemars(
        description = "Keywords matched as whole tokens (case-insensitive, alnum/underscore tokens)."
    )]
    keywords: Vec<String>,
    #[schemars(description = "Max lines to return after sorting by hit count.")]
    #[serde(default = "default_rank_keyword_top_lines")]
    top_lines: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct OcrTesseractArgs {
    #[schemars(description = "Image path under project root (png, jpg, tiff, webp, bmp, …).")]
    path: String,
    #[schemars(description = "Tesseract -l language (e.g. eng, deu); empty = eng.")]
    #[serde(default = "default_tesseract_lang")]
    lang: String,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeIndexFileSymbolsPathArgs {
    #[schemars(
        description = "File path relative to project root (as in code index, forward slashes)."
    )]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitkrakenHostedToolNameArgs {
    #[schemars(description = "Hosted GitKraken/GitLens MCP tool name (e.g. pull_request_create).")]
    tool_name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct TerminalProgramVersionArgs {
    #[schemars(description = "Bare executable name (same allowlist as terminal_run).")]
    program: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CompressSentencesArgs {
    #[schemars(description = "Source text.")]
    text: String,
    #[schemars(description = "Max sentences to keep from the start (after .?! splits).")]
    #[serde(default = "default_compress_sentence_count")]
    max_sentences: usize,
    #[serde(default = "default_compress_max_chars")]
    max_chars: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SummarizeMultiFileArgs {
    #[schemars(description = "File paths under project root (max 8 used).")]
    paths: Vec<String>,
    #[schemars(description = "Lines read per file (1–400) before merge-summary.")]
    #[serde(default = "default_summarize_multi_lines")]
    lines_per_file: usize,
    #[serde(default = "default_summarize_max_chars")]
    max_chars: usize,
    #[serde(default = "default_summarize_max_sentences")]
    max_sentences: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WaitUntilArgs {
    #[schemars(
        description = "Unix epoch milliseconds to wait toward. Sleeps at most one profile cap per call; re-call if remaining_ms > 0."
    )]
    target_ts_ms: u64,
    #[schemars(
        description = "Random extra 0..=jitter (capped by profile sleep cap) added to this slice—reduces synchronized wakeups across agents."
    )]
    #[serde(default)]
    jitter_ms_max: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct WaitForPathArgs {
    #[schemars(description = "Path under project root to wait for (file or directory).")]
    path: String,
    #[schemars(
        description = "If true, require a regular file; if false, any path existence suffices."
    )]
    #[serde(default)]
    must_be_file: bool,
    #[schemars(description = "Poll interval between existence checks (50–2000ms).")]
    #[serde(default = "default_wait_poll_ms")]
    poll_interval_ms: u64,
    #[schemars(
        description = "Extra random 0..=poll_jitter_ms_max added to each sleep (desync pollers); capped at 5000ms effective slice."
    )]
    #[serde(default)]
    poll_jitter_ms_max: u64,
    #[schemars(
        description = "Total wait budget (capped by server per-call sync limit; lower on public profile)."
    )]
    #[serde(default = "default_wait_path_timeout_ms")]
    timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AgentHandoffArgs {
    #[schemars(description = "Sender agent id.")]
    from_agent: String,
    #[schemars(description = "Recipient agent id (DM).")]
    to_agent: String,
    #[schemars(description = "One-line summary for inbox scans.")]
    headline: String,
    #[schemars(description = "Full handoff body (context, next steps).")]
    body: String,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    correlation_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitLogOrDiffLensArgs {
    #[schemars(
        description = "Repo directory under project root; empty = project root (GitLens-compatible `directory`)."
    )]
    #[serde(default)]
    directory: String,
    #[schemars(description = "log | diff")]
    action: String,
    #[schemars(description = "Revision range (e.g. main..feature, HEAD~5..HEAD).")]
    #[serde(default)]
    revision_range: Option<String>,
    #[schemars(description = "git log --since (log only).")]
    #[serde(default)]
    since: Option<String>,
    #[schemars(description = "git log --until (log only).")]
    #[serde(default)]
    until: Option<String>,
    #[schemars(description = "Optional author filters (git log --author each).")]
    #[serde(default)]
    authors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitBranchLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "list | create")]
    action: String,
    #[schemars(description = "Branch name when action=create.")]
    #[serde(default)]
    branch_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitWorktreeLensArgs {
    #[schemars(description = "Main repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "list | add")]
    action: String,
    #[schemars(description = "Path for new worktree (add); must stay under project root.")]
    #[serde(default)]
    path: Option<String>,
    #[schemars(description = "Existing branch name (add).")]
    #[serde(default)]
    branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitAddOrCommitLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "add | commit")]
    action: String,
    #[schemars(
        description = "Paths relative to repo; omit add→all (`git add -A`), omit commit→all staged."
    )]
    #[serde(default)]
    files: Option<Vec<String>>,
    #[schemars(description = "Commit message (required for commit).")]
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitPushLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitFetchLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitPullLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitTagListArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Optional `git tag -l` pattern glob; omit for all tags.")]
    #[serde(default)]
    pattern: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitLsFilesArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "tracked (default) | untracked | all")]
    #[serde(default = "default_git_ls_files_mode")]
    mode: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitDiffStatArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "If true, `git diff --stat --staged` (omit revision_range).")]
    #[serde(default)]
    staged: bool,
    #[schemars(
        description = "Single rev/ref or range (e.g. HEAD, main..feature); validated. Omit for unstaged working tree vs index."
    )]
    #[serde(default)]
    revision_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitMergeBaseArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "First commit-ish (e.g. main).")]
    commit_a: String,
    #[schemars(description = "Second commit-ish (e.g. feature).")]
    commit_b: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitDescribeArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitShortlogArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Max contributor rows (5–200).")]
    #[serde(default = "default_shortlog_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitReflogArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Max reflog lines (5–300).")]
    #[serde(default = "default_reflog_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitLastCommitArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "oneline (default) | subject | body | medium")]
    #[serde(default = "default_git_last_commit_style")]
    style: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GitStashLensArgs {
    #[schemars(description = "Repo directory under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "Stash message (GitLens `name`).")]
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct RepositoryGetFileContentArgs {
    #[schemars(description = "Repo root under project root; empty = project root.")]
    #[serde(default)]
    directory: String,
    #[schemars(description = "File path relative to repo.")]
    file_path: String,
    #[schemars(
        description = "Optional git ref for `git show ref:path` (omit = read working tree file)."
    )]
    #[serde(default)]
    git_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GkGitStatusArgs {
    #[schemars(
        description = "GitKraken MCP-compatible: repo directory under project root (empty = project root)."
    )]
    directory: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GkGitCheckoutArgs {
    #[schemars(
        description = "GitKraken MCP-compatible: repo directory under project root (empty = project root)."
    )]
    directory: String,
    #[schemars(description = "GitKraken `branch` argument: branch, tag, or ref for git checkout.")]
    branch: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GkGitBlameArgs {
    #[schemars(
        description = "GitKraken MCP-compatible: repo directory under project root (empty = project root)."
    )]
    directory: String,
    #[schemars(description = "File path under project root (resolved then relativized to repo).")]
    file: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ConsensusProposeArgs {
    #[schemars(description = "Stable proposer id (alphanumeric, _.-).")]
    from_agent: String,
    #[schemars(description = "Short topic / title for the proposal.")]
    topic: String,
    #[schemars(description = "Longer description of what is being proposed.")]
    description: String,
    #[schemars(description = "Comma-separated voting options (e.g. 'yes,no,abstain').")]
    options: String,
    #[schemars(description = "Voting window in milliseconds (e.g. 60000 for 1 minute).")]
    deadline_ms: u64,
    #[schemars(
        description = "Strategy name: SimpleMajority, SuperMajority, Unanimous, WeightedVoting, RankedChoice, Delegation."
    )]
    strategy: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ConsensusVoteArgs {
    #[schemars(description = "ID of the proposal to vote on.")]
    proposal_id: String,
    #[schemars(description = "Stable voter id (alphanumeric, _.-).")]
    voter: String,
    #[schemars(description = "The chosen option (must match one of the proposal's options).")]
    choice: String,
    #[schemars(description = "Confidence level 0.0–1.0.")]
    confidence: f64,
    #[schemars(description = "Optional free-text rationale for the vote.")]
    #[serde(default)]
    rationale: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ConsensusTallyArgs {
    #[schemars(description = "ID of the proposal to tally.")]
    proposal_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ConsensusCloseArgs {
    #[schemars(description = "ID of the proposal to close and finalize.")]
    proposal_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AskAgentsArgs {
    #[schemars(description = "Stable id for the asking agent (e.g. zed_user, cursor-main).")]
    #[serde(default = "default_from_agent")]
    from_agent: String,
    #[schemars(description = "Question to broadcast to all agents.")]
    query: String,
    #[schemars(
        description = "Thread key. If omitted, server generates a unique topic so you can poll later with agent_thread."
    )]
    #[serde(default)]
    topic: Option<String>,
    #[schemars(
        description = "How long to wait for replies (clamped; lower cap when MINT_NETWORK_PROFILE=public)."
    )]
    #[serde(default = "default_ask_timeout_ms")]
    timeout_ms: u64,
    #[schemars(description = "Max replies returned (1–50).")]
    #[serde(default = "default_ask_limit")]
    limit: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct PackContextArgs {
    #[schemars(
        description = "Context segments to pack (file excerpts, agent bus text, notes—useful before agent_send_pipeline or long prompts)."
    )]
    segments: Vec<String>,
    #[schemars(
        description = "Optional labels, one per segment (same length as segments); shown in segment headers for parallel/serial handoffs."
    )]
    #[serde(default)]
    segment_labels: Option<Vec<String>>,
    #[schemars(description = "Max output characters (server cap is MAX_RESPONSE_CHARS).")]
    #[serde(default = "default_pack_max_chars")]
    max_chars: usize,
    #[schemars(
        description = "Same as compress_context: truncate | head_tail | lines_only | dense."
    )]
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
    #[schemars(
        description = "Git repo directory (subfolder of project root); omit for project root."
    )]
    #[serde(default)]
    path: Option<String>,
    #[schemars(
        description = "If true, use --staged diff. If false, compare working tree to index."
    )]
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

// ---- Review Args ----

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReviewDiffRiskArgs {
    #[schemars(
        description = "If true, analyze staged changes; if false, analyze working tree vs index."
    )]
    #[serde(default)]
    staged: bool,
    #[schemars(
        description = "Git revision range (e.g. main..HEAD); omit for unstaged/staged diff."
    )]
    #[serde(default)]
    revision_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReviewFocusedReviewArgs {
    #[schemars(
        description = "Review focus: security, performance, correctness, maintainability, or style."
    )]
    focus: String,
    #[schemars(
        description = "If true, review staged changes; if false, review working tree vs index."
    )]
    #[serde(default)]
    staged: bool,
    #[schemars(
        description = "Git revision range (e.g. main..HEAD); omit for unstaged/staged diff."
    )]
    #[serde(default)]
    revision_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReviewMissingTestsArgs {
    #[schemars(
        description = "If true, check staged changes; if false, check working tree vs index."
    )]
    #[serde(default)]
    staged: bool,
    #[schemars(
        description = "Git revision range (e.g. main..HEAD); omit for unstaged/staged diff."
    )]
    #[serde(default)]
    revision_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReviewPrPacketArgs {
    #[schemars(description = "Git revision range for the PR (e.g. main..feature).")]
    revision_range: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ReviewRulebookCheckArgs {
    #[schemars(description = "Path to the rulebook file under the project root.")]
    rulebook_path: String,
    #[schemars(
        description = "If true, check staged changes; if false, check working tree vs index."
    )]
    #[serde(default)]
    staged: bool,
    #[schemars(
        description = "Git revision range (e.g. main..HEAD); omit for unstaged/staged diff."
    )]
    #[serde(default)]
    revision_range: Option<String>,
}

// ---- Debug Args ----

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DebugTraceRunArgs {
    #[schemars(description = "Unique identifier for this trace (alphanumeric, _.-).")]
    trace_id: String,
    #[schemars(description = "Bare executable name to run under trace.")]
    command: String,
    #[schemars(description = "Argv after the command name.")]
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DebugReplayRunArgs {
    #[schemars(description = "Trace id from a previous debug_trace_run.")]
    trace_id: String,
    #[schemars(description = "If true, preview the replay without executing (default true).")]
    #[serde(default = "default_true")]
    dry_run: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DebugFailureClusterArgs {
    #[schemars(
        description = "Directory containing trace files; omit for default .mint-mcp/traces/."
    )]
    #[serde(default)]
    trace_dir: Option<String>,
    #[schemars(description = "Minimum traces to form a cluster (default 2).")]
    #[serde(default = "default_min_cluster_size")]
    min_cluster_size: usize,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DebugRootCauseReportArgs {
    #[schemars(description = "Trace id to generate the root-cause report for.")]
    trace_id: String,
}

// ---- Code intelligence Args ----

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeAstDumpArgs {
    #[schemars(description = "Source file path under the project root.")]
    path: String,
    #[schemars(
        description = "Language hint (e.g. rust, python, typescript); omit for auto-detection."
    )]
    #[serde(default)]
    language: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeExtractImportsArgs {
    #[schemars(description = "Source file path under the project root.")]
    path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeDetectLanguageArgs {
    #[schemars(description = "File path under the project root.")]
    path: String,
    #[schemars(description = "Optional file content override for language detection heuristics.")]
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeComplexityScoreArgs {
    #[schemars(description = "Source file path under the project root.")]
    path: String,
    #[schemars(description = "Optional file content override for complexity scoring.")]
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CodeDeadCodeHintsArgs {
    #[schemars(
        description = "File or directory path under the project root to scan for dead code."
    )]
    path: String,
}

// ============================================================================
// Default-value functions for serde
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_min_cluster_size() -> usize {
    2
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

fn default_list_topics_limit() -> usize {
    50
}

fn default_inbox_wait_timeout_ms() -> u64 {
    15_000
}

fn default_inbox_wait_poll_ms() -> u64 {
    250
}

fn default_snapshot_topics_limit() -> usize {
    20
}

fn default_snapshot_recent_limit() -> usize {
    25
}

fn default_codebase_max_files() -> usize {
    6000
}

fn default_codebase_max_symbols() -> usize {
    120
}

fn default_sleep_ms() -> u64 {
    1000
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

fn default_json_structure_depth() -> usize {
    3
}

fn default_json_structure_keys() -> usize {
    48
}

fn default_json_structure_lines() -> usize {
    100
}

fn default_levenshtein_max_len() -> usize {
    1_000
}

fn default_token_jaccard_max_chars() -> usize {
    80_000
}

fn default_token_jaccard_max_scan() -> usize {
    50_000
}

fn default_compress_paragraphs_max() -> usize {
    12
}

fn default_manifest_max_files() -> usize {
    5_000
}

fn default_terminal_timeout_ms() -> u64 {
    60_000
}

fn default_project_test_timeout_ms() -> u64 {
    300_000
}

fn default_duplicate_symbols_min_files() -> usize {
    2
}

fn default_truncate_line_width() -> usize {
    120
}

fn default_diff_file_list_max_paths() -> usize {
    100
}

fn default_delimited_max_columns() -> usize {
    64
}

fn default_unique_lines_max() -> usize {
    400
}

fn default_word_index_max_files() -> usize {
    400
}

fn default_word_index_top() -> usize {
    60
}

fn default_shallow_hash_max_files() -> usize {
    800
}

fn default_shallow_hash_sample() -> usize {
    4096
}

fn default_path_list_max_paths() -> usize {
    8000
}

fn default_largest_scan_files() -> usize {
    8000
}

fn default_largest_top_n() -> usize {
    80
}

fn default_dup_basenames_scan() -> usize {
    12_000
}

fn default_code_index_max_files() -> usize {
    2000
}

fn default_code_index_search_limit() -> usize {
    50
}

fn default_code_index_paths_limit() -> usize {
    200
}

fn default_thinking_tail_lines() -> usize {
    50
}

fn default_code_index_analytics_limit() -> usize {
    50
}

fn default_summarize_token_max_chars() -> usize {
    4000
}

fn default_line_prefix_len() -> usize {
    16
}

fn default_line_prefix_max_groups() -> usize {
    60
}

fn default_plan_buckets() -> usize {
    4
}

fn default_rank_keyword_top_lines() -> usize {
    40
}

fn default_tesseract_lang() -> String {
    "eng".to_string()
}

fn default_temp_max_store_bytes() -> usize {
    256 * 1024
}

fn default_temp_max_read_bytes() -> usize {
    512 * 1024
}

fn default_workspace_temp_list_limit() -> usize {
    100
}

fn default_wait_poll_ms() -> u64 {
    250
}

fn default_wait_path_timeout_ms() -> u64 {
    30_000
}

fn default_gh_pr_limit() -> usize {
    20
}

fn default_gh_pr_state() -> String {
    "open".to_string()
}

fn default_top_terms_min_len() -> usize {
    3
}

fn default_top_terms_max() -> usize {
    40
}

fn default_identifier_min_len() -> usize {
    2
}

fn default_identifier_max_list() -> usize {
    60
}

fn default_lcss_max_len() -> usize {
    1_500
}

fn default_dup_lines_min() -> usize {
    2
}

fn default_dup_lines_max_entries() -> usize {
    60
}

fn default_truncate_middle_margin() -> usize {
    1200
}

fn default_summarize_bullets_max() -> usize {
    50
}

fn default_summarize_headings_max() -> usize {
    80
}

fn default_summarize_first_lines_count() -> usize {
    24
}

fn default_git_last_commit_style() -> String {
    "oneline".to_string()
}

fn default_shortlog_limit() -> usize {
    40
}

fn default_reflog_limit() -> usize {
    50
}

fn default_git_ls_files_mode() -> String {
    "tracked".to_string()
}

fn default_compress_sentence_count() -> usize {
    28
}

fn default_summarize_multi_lines() -> usize {
    120
}

fn default_tab_stops() -> usize {
    4
}

fn default_line_number_start() -> usize {
    1
}

fn default_wrap_width() -> usize {
    88
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

fn default_from_agent() -> String {
    "zed_user".to_string()
}

fn default_ask_timeout_ms() -> u64 {
    5_000
}

fn default_ask_limit() -> usize {
    12
}
