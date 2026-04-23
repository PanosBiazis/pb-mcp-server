mod agent;
mod bus;
mod bus_rate;
mod code_index;
mod network_profile;
mod ops;
mod thinking;

use crate::ops::{
    agent_coordination_playbook_work, ai_context_pack_smart_work,
    ai_inference_integration_guide_work, ai_ollama_embed_work, ai_ollama_generate_work,
    ai_prompt_template_work, ai_semantic_diff_work, ai_stack_probe_work, batch_file_info_work,
    code_ast_dump_work, code_complexity_score_work, code_dead_code_hints_work,
    code_detect_language_work, code_extract_imports_work, codebase_cargo_dependency_names_work,
    codebase_duplicate_basenames_work, codebase_index_work, codebase_largest_files_work,
    codebase_manifest_write_work, codebase_path_list_work, codebase_shallow_hash_work,
    codebase_word_index_work, coding_agent_toolchains_catalog_work,
    compress_collapse_blank_lines_work, compress_context_work, compress_dedent_work,
    compress_double_slash_line_comments_work, compress_drop_lines_matching_regex_work,
    compress_fenced_code_blocks_work, compress_file_work, compress_gzip_base64_work,
    compress_lines_head_work, compress_lines_tail_work, compress_markdown_frontmatter_work,
    compress_markdown_links_work, compress_paragraphs_work, compress_redact_emails_work,
    compress_redact_urls_work, compress_sentences_work, compress_simplify_whitespace_work,
    compress_strip_tags_work, compress_truncate_lines_work, compress_truncate_middle_work,
    compress_unified_diff_stats_work, compress_unique_lines_work, compress_ws_collapse_work,
    convert_archive_extract_work, convert_archive_list_work, convert_binary_hexdump_work,
    convert_document_work, convert_encoding_work, convert_format_work, convert_pdf_extract_work,
    convert_qr_generate_work, db_json_query_work, db_sqlite_exec_work, db_sqlite_query_work,
    db_sqlite_schema_work, debug_failure_cluster_work, debug_replay_run_work,
    debug_root_cause_report_work, debug_trace_run_work, decompress_gzip_base64_work,
    extract_snippets_from_file_work, extract_snippets_work, file_info_work, file_stats_work,
    find_files_by_name_work, find_files_work, format_prefix_lines_work, gh_pr_list_work,
    gh_pr_view_work, git_add_or_commit_work, git_blame_work, git_branch_extension_work,
    git_branches_work, git_checkout_work, git_describe_work, git_diff_snippets_work,
    git_diff_stat_work, git_diff_work, git_fetch_work, git_last_commit_work, git_log_or_diff_work,
    git_log_work, git_ls_files_work, git_merge_base_work, git_pull_work, git_push_work,
    git_reflog_work, git_remote_list_work, git_rev_parse_work, git_shortlog_work, git_show_work,
    git_stash_list_work, git_stash_push_work, git_status_work, git_tag_list_work,
    git_worktree_extension_work, git_worktree_list_work, gitkraken_cloud_catalog_work,
    gitkraken_hosted_tool_redirect_work, glab_mr_list_work, glab_mr_show_work, grep_snippets_work,
    grep_work, list_files_work, mcp_env_hints_work, notify_desktop_work, notify_email_send_work,
    notify_webhook_work, ocr_tesseract_file_work, ollama_list_models_work, pack_context_work,
    project_cargo_check_build_args, project_cargo_test_build_args, project_pytest_build_args,
    project_test_tools_enabled, read_file_tail_work, read_file_work, read_json_file_work,
    repository_get_file_content_work, review_diff_risk_work, review_focused_review_work,
    review_missing_tests_work, review_pr_packet_work, review_rulebook_check_work,
    screenshot_capture_work, summarize_bullets_work, summarize_delimited_header_work,
    summarize_diff_file_list_work, summarize_duplicate_lines_work, summarize_file_work,
    summarize_first_lines_work, summarize_headings_work, summarize_identifier_frequency_work,
    summarize_json_structure_work, summarize_line_histogram_work,
    summarize_line_prefix_groups_work, summarize_markdown_toc_work, summarize_multi_file_work,
    summarize_rank_lines_by_keywords_work, summarize_segments_work, summarize_text_work,
    summarize_token_estimate_work, summarize_top_terms_work, system_disk_usage_work,
    system_env_get_work, system_network_interfaces_work, system_process_list_work,
    terminal_program_version_work, terminal_run_precheck, terminal_which_work, text_djb2_hash_work,
    text_levenshtein_distance_work, text_longest_common_substring_work, text_stats_work,
    text_token_jaccard_work, tree_work, web_api_call_work, web_dns_lookup_work,
    web_download_file_work, web_rss_fetch_work, web_screenshot_work, web_search_work,
    web_tls_inspect_work, workspace_context_work, workspace_temp_delete_work,
    workspace_temp_list_work, workspace_temp_read_work, workspace_temp_write_work,
    MAX_TERMINAL_STDIN_BYTES,
};
use crate::thinking::{
    plan_definition_of_done_work, plan_execution_checklist_work, plan_handoff_brief_work,
    plan_retrospective_work, plan_risk_register_work, plan_test_strategy_work,
    plan_work_buckets_work, reasoning_chain_work, reasoning_devils_advocate_work,
    reasoning_evidence_matrix_work, reasoning_five_whys_work, reasoning_premortem_work,
    thinking_assumption_audit_work, thinking_plan_skeleton_work, thinking_session_append_work,
    thinking_session_read_work, thinking_step_back_work,
};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{stdin, stdout, AsyncWriteExt};

#[derive(Clone)]
struct MintMcp {
    root: PathBuf,
    tool_router: ToolRouter<Self>,
    bus_rate: Arc<bus_rate::BusSendRateLimiter>,
    bus_read_only: bool,
    network_profile: network_profile::NetworkProfile,
    consensus_engine: Arc<std::sync::Mutex<crate::agent::ConsensusEngine>>,
}

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

impl MintMcp {
    fn new() -> Self {
        let root = std::env::var("PB_PROJECT_ROOT")
            .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
            .or_else(|_| std::env::var("PROJECT_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let bus_read_only = std::env::var("PB_BUS_READ_ONLY")
            .or_else(|_| std::env::var("MINT_BUS_READ_ONLY"))
            .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);

        let network_profile = network_profile::NetworkProfile::from_env();

        Self {
            root,
            tool_router: Self::tool_router(),
            bus_rate: Arc::new(bus_rate::BusSendRateLimiter::from_env()),
            bus_read_only,
            network_profile,
            consensus_engine: Arc::new(std::sync::Mutex::new(crate::agent::ConsensusEngine::new())),
        }
    }

    /// Internet / `PB_NETWORK_PROFILE` / `MINT_NETWORK_PROFILE=public` blocks bus mutations, terminal, risky git, and file writes from converters.
    fn require_trusted_deployment(&self) -> Result<(), String> {
        if self.network_profile.is_public() {
            Err(
                "Error: tool blocked when PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE=public. Use local/LAN (default) or read-only tools."
                    .to_string(),
            )
        } else {
            Ok(())
        }
    }

    fn sync_wait_timeout_cap_ms(&self) -> u64 {
        if self.network_profile.is_public() {
            45_000
        } else {
            120_000
        }
    }

    fn ask_agents_timeout_cap_ms(&self) -> u64 {
        if self.network_profile.is_public() {
            8_000
        } else {
            20_000
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
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());

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
            Err("Path escapes project root (set PB_PROJECT_ROOT, MINT_PROJECT_ROOT, or PROJECT_ROOT)".to_string())
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
            _ => Ok(self
                .root
                .canonicalize()
                .unwrap_or_else(|_| self.root.clone())),
        }
    }

    /// GitLens-style `directory` argument: empty string = project root.
    fn git_directory_arg(&self, directory: &str) -> Result<PathBuf, String> {
        if directory.trim().is_empty() {
            Ok(self
                .root
                .canonicalize()
                .unwrap_or_else(|_| self.root.clone()))
        } else {
            self.resolve_path(directory)
        }
    }

    /// Spawn allowlisted `program` with argv under cwd (trusted profile + terminal gate). Used by project_* test helpers.
    async fn run_terminal_tool(
        &self,
        program: &str,
        argv: Vec<String>,
        cwd: PathBuf,
        timeout_ms: u64,
        header: &str,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if let Err(e) = terminal_run_precheck(program) {
            return Self::tool_text(format!("Error: {e}"));
        }
        let timeout_ms = timeout_ms.clamp(1, 600_000);
        use std::process::Stdio;
        use tokio::process::Command;
        use tokio::time::{timeout, Duration};
        let program_owned = program.to_string();
        let fut = Command::new(&program_owned)
            .args(&argv)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        let out = timeout(Duration::from_millis(timeout_ms), fut).await;
        match out {
            Ok(Ok(output)) => {
                let code = output.status.code().unwrap_or(-1);
                let mut s = format!("{header} program={program_owned} exit={code}\n");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let (so, _) = crate::ops::truncate_chars(&stdout, 24_000);
                let (se, _) = crate::ops::truncate_chars(&stderr, 24_000);
                s.push_str("--- STDOUT ---\n");
                s.push_str(&so);
                s.push_str("\n--- STDERR ---\n");
                s.push_str(&se);
                Self::tool_text(s)
            }
            Ok(Err(e)) => Self::tool_text(format!("Error: spawn {e}")),
            Err(_) => Self::tool_text("Error: command timed out (increase timeout_ms)".to_string()),
        }
    }

    fn now_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn new_topic() -> String {
        format!("ask-{}-{}", Self::now_ms(), std::process::id())
    }
}

#[tool_router]
impl MintMcp {
    #[tool(
        description = "Call first when orienting in a repo: canonical project root, git branch, sorted top-level names, whether the multi-agent bus file exists, hard limits for read/grep/tree/diff, and a concise index of multi-agent bus tools/limits. Cheap single round-trip for planning later tool calls.",
        annotations(title = "Workspace context", read_only_hint = true)
    )]
    async fn workspace_context(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let net = self.network_profile.label().to_string();
        let cap = self.network_profile.server_sleep_cap_ms();
        let is_pub = self.network_profile.is_public();
        let out =
            tokio::task::spawn_blocking(move || workspace_context_work(root, net, cap, is_pub))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Lists GitKraken/GitLens *hosted* MCP tools (PRs, issues, Launchpad, workspaces) that need the official GitLens extension MCP + provider sign-in. This server implements local git/file/bus tools only.",
        annotations(title = "GitKraken cloud tool catalog", read_only_hint = true)
    )]
    async fn gitkraken_cloud_catalog(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(|| gitkraken_cloud_catalog_work())
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only index of coding-agent CLIs (OpenClaw, Claude Code, Codex-class tools, Aider, …) and how they combine with this MCP’s bus, git, OCR, and deterministic text tools. No subprocesses; safe on public profile.",
        annotations(title = "Coding agent toolchains catalog", read_only_hint = true)
    )]
    async fn coding_agent_toolchains_catalog(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(|| coding_agent_toolchains_catalog_work())
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Lookup a GitKraken/GitLens **hosted** MCP tool by name: returns a redirect message (enable GitLens MCP + auth) or an error if the name is unknown. Read-only.",
        annotations(title = "GitKraken hosted tool redirect", read_only_hint = true)
    )]
    async fn gitkraken_hosted_tool_redirect(
        &self,
        Parameters(args): Parameters<GitkrakenHostedToolNameArgs>,
    ) -> Result<CallToolResult, McpError> {
        let name = args.tool_name.clone();
        let out = tokio::task::spawn_blocking(move || gitkraken_hosted_tool_redirect_work(&name))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only snapshot of PB_* / MINT_* gate variables (PROJECT_ROOT values redacted to length only). Safe on PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE=public.",
        annotations(title = "MCP env hints", read_only_hint = true)
    )]
    async fn mcp_env_hints(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(|| mcp_env_hints_work())
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Static playbook: async vs sync bus patterns, serial pipelines, parallel fan-out, subagent flow, public-profile limits, and safe compression before posting. Read-only; safe on public profile.",
        annotations(title = "Agent coordination playbook", read_only_hint = true)
    )]
    async fn agent_coordination_playbook(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Self::tool_text(agent_coordination_playbook_work().to_string())
    }

    #[tool(
        description = "Read-only guide: no neural weights in this server — how to use Tesseract OCR, Ollama, and external ML/DL safely with MINT_* gates and public vs trusted profiles.",
        annotations(title = "AI / ML / DL integration guide", read_only_hint = true)
    )]
    async fn ai_inference_integration_guide(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        Self::tool_text(ai_inference_integration_guide_work().to_string())
    }

    #[tool(
        description = "Probe optional CLIs (python3, tesseract, ollama, ffmpeg, openclaw, claude, codex, aider, gemini, …) with --version-style calls. Uses MINT_TERMINAL_ENABLE + MINT_TERMINAL_ALLOW per program (same as terminal_program_version). Safe read-only; may be skipped if terminal is disabled.",
        annotations(title = "AI optional stack probe", read_only_hint = true)
    )]
    async fn ai_stack_probe(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let out = tokio::task::spawn_blocking(ai_stack_probe_work)
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Block the MCP server worker for duration_ms (cross-platform tokio sleep). Cap is 30s on local profile, 5s on MINT_NETWORK_PROFILE=public. Optional jitter_ms_max desynchronizes agents. Does not pause other clients—prefer agent_inbox_wait / agent_topic_wait for coordination.",
        annotations(title = "Server sleep", read_only_hint = true)
    )]
    async fn server_sleep(
        &self,
        Parameters(args): Parameters<ServerSleepArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cap = self.network_profile.server_sleep_cap_ms();
        let base = args.duration_ms.clamp(0, cap);
        let jitter_cap = args.jitter_ms_max.min(cap);
        let jitter = if jitter_cap > 0 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| {
                    let mix = d.subsec_nanos() as u64 ^ d.as_secs().wrapping_mul(1_000_000_007);
                    mix % (jitter_cap + 1)
                })
                .unwrap_or(0)
        } else {
            0
        };
        let ms = (base + jitter).min(cap);
        let jitter_req = args.jitter_ms_max;
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let mut s = format!(
            "SERVER_SLEEP\nslept_ms: {ms}\nbase_ms: {base}\njitter_applied_ms: {jitter}\njitter_ms_max_requested: {jitter_req} (effective_cap: {jitter_cap})\nprofile_cap_ms: {cap}\n"
        );
        if let Some(ref r) = args.reason {
            let t = r.replace(['\r', '\n'], " ").trim().to_string();
            if !t.is_empty() {
                let (head, _) = crate::ops::truncate_chars(&t, 200);
                s.push_str(&format!("reason: {head}\n"));
            }
        }
        s.push_str("hint: prefer agent_inbox_wait / agent_topic_wait for multi-agent sync.\n");
        Self::tool_text(s)
    }

    #[tool(
        description = "Sleep toward an absolute server clock (Unix ms). One call sleeps at most the network profile cap (e.g. 5s public); optional jitter_ms_max like server_sleep. Repeat until remaining_ms is 0.",
        annotations(title = "Wait until timestamp", read_only_hint = true)
    )]
    async fn wait_until_timestamp(
        &self,
        Parameters(args): Parameters<WaitUntilArgs>,
    ) -> Result<CallToolResult, McpError> {
        let now = Self::now_ms();
        let cap = self.network_profile.server_sleep_cap_ms();
        if args.target_ts_ms <= now {
            return Self::tool_text(format!(
                "WAIT_UNTIL\nskipped: target_ts_ms ({}) <= server_ts_ms ({})\n",
                args.target_ts_ms, now
            ));
        }
        let base = (args.target_ts_ms - now).min(cap);
        let jitter_cap = args.jitter_ms_max.min(cap);
        let jitter = if jitter_cap > 0 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| {
                    let mix = d.subsec_nanos() as u64 ^ d.as_secs().wrapping_mul(1_000_000_007);
                    mix % (jitter_cap + 1)
                })
                .unwrap_or(0)
        } else {
            0
        };
        let delta = (base + jitter).min(cap);
        let jitter_req = args.jitter_ms_max;
        tokio::time::sleep(std::time::Duration::from_millis(delta)).await;
        let after = Self::now_ms();
        let remaining = args.target_ts_ms.saturating_sub(after);
        Self::tool_text(format!(
            "WAIT_UNTIL\nslept_ms={delta}\nbase_ms={base}\njitter_applied_ms={jitter}\njitter_ms_max_requested={jitter_req} (effective_cap={jitter_cap})\ntarget_ts_ms={}\nserver_ts_ms_after={after}\nremaining_ms={remaining}\nhint: call again if remaining_ms > 0, or use agent_inbox_wait / agent_topic_wait.\n",
            args.target_ts_ms
        ))
    }

    #[tool(
        description = "Poll until a path under the project root exists (optionally as a regular file). Async-friendly pacing for agents; total wait clamped like other sync tools (lower on public profile).",
        annotations(title = "Wait for path", read_only_hint = true)
    )]
    async fn wait_for_path(
        &self,
        Parameters(args): Parameters<WaitForPathArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cap = self.sync_wait_timeout_cap_ms();
        let timeout = args.timeout_ms.clamp(50, cap);
        let poll = args.poll_interval_ms.clamp(50, 2000);
        let jitter_cap = args.poll_jitter_ms_max.min(5000);
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let must_file = args.must_be_file;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout);
        loop {
            let ok = if must_file {
                std::fs::metadata(&path)
                    .map(|m| m.is_file())
                    .unwrap_or(false)
            } else {
                path.exists()
            };
            if ok {
                return Self::tool_text(format!(
                    "WAIT_FOR_PATH\npath: {}\nready: true\nmust_be_file: {must_file}\n",
                    path.display()
                ));
            }
            if tokio::time::Instant::now() >= deadline {
                return Self::tool_text(format!(
                    "WAIT_FOR_PATH\npath: {}\nready: false\ntimed_out_after_ms: {timeout}\npoll_interval_ms: {poll}\npoll_jitter_ms_max: {jitter_cap}\n",
                    path.display()
                ));
            }
            let extra = if jitter_cap > 0 {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| {
                        let mix = d.subsec_nanos() as u64 ^ d.as_secs().wrapping_mul(1_000_000_007);
                        mix % (jitter_cap + 1)
                    })
                    .unwrap_or(0)
            } else {
                0
            };
            let sleep_ms = poll.saturating_add(extra).min(5000).max(1);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
        }
    }

    #[tool(
        description = "Run one allowlisted program with explicit argv (no shell). Blocked when MINT_NETWORK_PROFILE=public. Requires MINT_TERMINAL_ENABLE=1; program must appear in MINT_TERMINAL_ALLOW (comma-separated bare names). cwd must stay under project root; stdout/stderr capped. The subprocess can read/write per argv—use only on trusted local/LAN hosts.",
        annotations(title = "Terminal run (gated)", read_only_hint = false)
    )]
    async fn terminal_run(
        &self,
        Parameters(args): Parameters<TerminalRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if let Err(e) = terminal_run_precheck(&args.program) {
            return Self::tool_text(format!("Error: {e}"));
        }
        let cwd = match &args.cwd {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => pb,
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => self
                .root
                .canonicalize()
                .unwrap_or_else(|_| self.root.clone()),
        };
        let timeout_ms = args.timeout_ms.clamp(1, 120_000);
        let program = args.program.clone();
        let argv = args.args.clone();

        use std::process::Stdio;
        use tokio::process::Command;
        use tokio::time::{timeout, Duration};

        let fut = Command::new(&program)
            .args(&argv)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let out = timeout(Duration::from_millis(timeout_ms), fut).await;

        match out {
            Ok(Ok(output)) => {
                let code = output.status.code().unwrap_or(-1);
                let mut s = format!("TERMINAL_RUN program={program} exit={code}\n");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let (so, _) = crate::ops::truncate_chars(&stdout, 12_000);
                let (se, _) = crate::ops::truncate_chars(&stderr, 12_000);
                s.push_str("--- STDOUT ---\n");
                s.push_str(&so);
                s.push_str("\n--- STDERR ---\n");
                s.push_str(&se);
                Self::tool_text(s)
            }
            Ok(Err(e)) => Self::tool_text(format!("Error: spawn {e}")),
            Err(_) => Self::tool_text(
                "Error: command timed out (increase timeout_ms or narrow work)".to_string(),
            ),
        }
    }

    #[tool(
        description = "Like terminal_run but connects stdin from stdin_text or raw bytes of stdin_path (under project root, max 512 KiB). Blocked when MINT_NETWORK_PROFILE=public. Same allowlist, cwd, and timeout. Use on trusted LAN to pipe prompts into ollama, filters, etc.",
        annotations(title = "Terminal run with stdin", read_only_hint = false)
    )]
    async fn terminal_run_stdin(
        &self,
        Parameters(args): Parameters<TerminalRunStdinArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if let Err(e) = terminal_run_precheck(&args.program) {
            return Self::tool_text(format!("Error: {e}"));
        }
        let stdin_bytes: Vec<u8> = match (
            args.stdin_text.as_ref().filter(|s| !s.is_empty()),
            args.stdin_path.as_ref().filter(|s| !s.trim().is_empty()),
        ) {
            (Some(t), None) => {
                let b = t.as_bytes();
                if b.len() > MAX_TERMINAL_STDIN_BYTES {
                    return Self::tool_text(format!(
                        "Error: stdin_text exceeds {MAX_TERMINAL_STDIN_BYTES} bytes"
                    ));
                }
                b.to_vec()
            }
            (None, Some(rel)) => {
                let path = match self.resolve_path(rel) {
                    Ok(pb) => pb,
                    Err(e) => return Self::tool_text(format!("Error: {e}")),
                };
                match std::fs::read(&path) {
                    Ok(b) => {
                        if b.len() > MAX_TERMINAL_STDIN_BYTES {
                            return Self::tool_text(format!(
                                "Error: stdin file exceeds {MAX_TERMINAL_STDIN_BYTES} bytes"
                            ));
                        }
                        b
                    }
                    Err(e) => return Self::tool_text(format!("Error: read stdin_path: {e}")),
                }
            }
            (Some(_), Some(_)) => {
                return Self::tool_text(
                    "Error: provide only one of stdin_text or stdin_path".to_string(),
                );
            }
            (None, None) => {
                return Self::tool_text("Error: provide stdin_text or stdin_path".to_string());
            }
        };
        let cwd = match &args.cwd {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => pb,
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => self
                .root
                .canonicalize()
                .unwrap_or_else(|_| self.root.clone()),
        };
        let timeout_ms = args.timeout_ms.clamp(1, 120_000);
        let program = args.program.clone();
        let argv = args.args.clone();

        use std::process::Stdio;
        use tokio::process::Command;
        use tokio::time::{timeout, Duration};

        let mut child = match Command::new(&program)
            .args(&argv)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return Self::tool_text(format!("Error: spawn {e}")),
        };
        if let Some(mut sin) = child.stdin.take() {
            if let Err(e) = sin.write_all(&stdin_bytes).await {
                return Self::tool_text(format!("Error: stdin write: {e}"));
            }
        }
        let fut = child.wait_with_output();
        let out = timeout(Duration::from_millis(timeout_ms), fut).await;

        match out {
            Ok(Ok(output)) => {
                let code = output.status.code().unwrap_or(-1);
                let mut s = format!("TERMINAL_RUN_STDIN program={program} exit={code}\n");
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let (so, _) = crate::ops::truncate_chars(&stdout, 12_000);
                let (se, _) = crate::ops::truncate_chars(&stderr, 12_000);
                s.push_str("--- STDOUT ---\n");
                s.push_str(&so);
                s.push_str("\n--- STDERR ---\n");
                s.push_str(&se);
                Self::tool_text(s)
            }
            Ok(Err(e)) => Self::tool_text(format!("Error: wait {e}")),
            Err(_) => Self::tool_text(
                "Error: command timed out (increase timeout_ms or narrow work)".to_string(),
            ),
        }
    }

    #[tool(
        description = "Run `cargo test` with bounded argv (no shell). Requires trusted/LAN profile, MINT_TERMINAL_ENABLE=1, `cargo` in MINT_TERMINAL_ALLOW, and MINT_PROJECT_TEST_ENABLE=1. Supports --workspace, -p package, --no-run, jobs, optional name filter, --nocapture. timeout_ms clamped to 600s.",
        annotations(title = "Project: cargo test", read_only_hint = false)
    )]
    async fn project_cargo_test(
        &self,
        Parameters(args): Parameters<ProjectCargoTestArgs>,
    ) -> Result<CallToolResult, McpError> {
        if !project_test_tools_enabled() {
            return Self::tool_text(
                "Error: project test tools disabled. Set MINT_PROJECT_TEST_ENABLE=1 (and MINT_TERMINAL_ENABLE=1 with `cargo` in MINT_TERMINAL_ALLOW)."
                    .to_string(),
            );
        }
        let argv = match project_cargo_test_build_args(
            args.package.as_deref(),
            args.test_filter.as_deref(),
            args.no_run,
            args.workspace,
            args.jobs,
            args.nocapture,
        ) {
            Ok(v) => v,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        self.run_terminal_tool("cargo", argv, cwd, args.timeout_ms, "PROJECT_CARGO_TEST")
            .await
    }

    #[tool(
        description = "Run `cargo check` with `--message-format short`. Same gates as project_cargo_test (MINT_PROJECT_TEST_ENABLE + terminal allowlist + trusted profile).",
        annotations(title = "Project: cargo check", read_only_hint = false)
    )]
    async fn project_cargo_check(
        &self,
        Parameters(args): Parameters<ProjectCargoCheckArgs>,
    ) -> Result<CallToolResult, McpError> {
        if !project_test_tools_enabled() {
            return Self::tool_text(
                "Error: project test tools disabled. Set MINT_PROJECT_TEST_ENABLE=1 (and MINT_TERMINAL_ENABLE=1 with `cargo` in MINT_TERMINAL_ALLOW)."
                    .to_string(),
            );
        }
        let argv = match project_cargo_check_build_args(args.package.as_deref(), args.workspace) {
            Ok(v) => v,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        self.run_terminal_tool("cargo", argv, cwd, args.timeout_ms, "PROJECT_CARGO_CHECK")
            .await
    }

    #[tool(
        description = "Run `pytest` with bounded argv (no shell). Requires trusted/LAN, MINT_PROJECT_TEST_ENABLE=1, and `pytest` in MINT_TERMINAL_ALLOW. test_path is relative to directory (no `..`). Optional -q, -k, --maxfail.",
        annotations(title = "Project: pytest", read_only_hint = false)
    )]
    async fn project_pytest(
        &self,
        Parameters(args): Parameters<ProjectPytestArgs>,
    ) -> Result<CallToolResult, McpError> {
        if !project_test_tools_enabled() {
            return Self::tool_text(
                "Error: project test tools disabled. Set MINT_PROJECT_TEST_ENABLE=1 (and MINT_TERMINAL_ENABLE=1 with `pytest` in MINT_TERMINAL_ALLOW)."
                    .to_string(),
            );
        }
        let argv = match project_pytest_build_args(
            &args.test_path,
            args.quiet,
            args.keyword.as_deref(),
            args.maxfail,
        ) {
            Ok(v) => v,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        self.run_terminal_tool("pytest", argv, cwd, args.timeout_ms, "PROJECT_PYTEST")
            .await
    }

    #[tool(
        description = "Resolve an allowlisted program on PATH (which / where). Read-only; uses the same MINT_TERMINAL_ENABLE + MINT_TERMINAL_ALLOW gates as terminal_run. Does not require MINT_NETWORK_PROFILE=local.",
        annotations(title = "Terminal which (gated)", read_only_hint = true)
    )]
    async fn terminal_which(
        &self,
        Parameters(args): Parameters<TerminalWhichArgs>,
    ) -> Result<CallToolResult, McpError> {
        let program = args.program.clone();
        let out = tokio::task::spawn_blocking(move || terminal_which_work(&program))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Run `program --version`, then `-V`, then `version` subcommand (first output). Same MINT_TERMINAL_ENABLE + MINT_TERMINAL_ALLOW as terminal_which; safe on public profile.",
        annotations(title = "Terminal program version", read_only_hint = true)
    )]
    async fn terminal_program_version(
        &self,
        Parameters(args): Parameters<TerminalProgramVersionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let program = args.program.clone();
        let out = tokio::task::spawn_blocking(move || terminal_program_version_work(&program))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "OCR an image under the project root via host `tesseract` (stdout). Blocked on MINT_NETWORK_PROFILE=public. Requires MINT_TERMINAL_ENABLE=1 and `tesseract` in MINT_TERMINAL_ALLOW. Max ~40 MiB file.",
        annotations(title = "OCR image (Tesseract)", read_only_hint = true)
    )]
    async fn ocr_tesseract(
        &self,
        Parameters(args): Parameters<OcrTesseractArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let lang = args.lang.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || ocr_tesseract_file_work(&path, &lang, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Run `ollama list` (local model tags). Blocked on public profile. Requires MINT_TERMINAL_ENABLE=1 and `ollama` in MINT_TERMINAL_ALLOW.",
        annotations(title = "Ollama list models", read_only_hint = true)
    )]
    async fn ollama_list_models(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let out = tokio::task::spawn_blocking(ollama_list_models_work)
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Full-screen capture to `.mint-mcp/screenshots/<output_name>.png` using `grim` (Wayland) or `screencapture` (macOS). Blocked on public profile. Requires MINT_TERMINAL_ENABLE=1 and grim/screencapture in MINT_TERMINAL_ALLOW. Chain ocr_tesseract on the saved path.",
        annotations(
            title = "Screenshot capture (grim / screencapture)",
            read_only_hint = false
        )
    )]
    async fn screenshot_capture(
        &self,
        Parameters(args): Parameters<ScreenshotCaptureArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let backend = args.backend.clone();
        let output_name = args.output_name.clone();
        let out = tokio::task::spawn_blocking(move || {
            let output_path = if output_name.is_empty() {
                None
            } else {
                Some(root.join(&output_name))
            };
            let display_env = if backend.is_empty() {
                None
            } else {
                Some(backend)
            };
            screenshot_capture_work(output_path, display_env)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Write UTF-8 scratch text to `.mint-mcp/agent-temp/<name>`. Blocked on public profile. Max 512 KiB per file; filename ASCII alnum + - _ . only.",
        annotations(title = "Workspace temp write", read_only_hint = false)
    )]
    async fn workspace_temp_write(
        &self,
        Parameters(args): Parameters<WorkspaceTempWriteArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let name = args.name.clone();
        let content = args.content.clone();
        let maxb = args.max_store_bytes;
        let out = tokio::task::spawn_blocking(move || {
            workspace_temp_write_work(root, name, content, maxb)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read a scratch file from `.mint-mcp/agent-temp/`. Blocked on public profile.",
        annotations(title = "Workspace temp read", read_only_hint = true)
    )]
    async fn workspace_temp_read(
        &self,
        Parameters(args): Parameters<WorkspaceTempReadArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let name = args.name.clone();
        let maxr = args.max_read_bytes;
        let out = tokio::task::spawn_blocking(move || workspace_temp_read_work(root, name, maxr))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List files in `.mint-mcp/agent-temp/`. Blocked on public profile.",
        annotations(title = "Workspace temp list", read_only_hint = true)
    )]
    async fn workspace_temp_list(
        &self,
        Parameters(args): Parameters<WorkspaceTempListArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || workspace_temp_list_work(root, limit))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Delete one file from `.mint-mcp/agent-temp/`. Blocked on public profile.",
        annotations(title = "Workspace temp delete", read_only_hint = false)
    )]
    async fn workspace_temp_delete(
        &self,
        Parameters(args): Parameters<WorkspaceTempDeleteArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let name = args.name.clone();
        let out = tokio::task::spawn_blocking(move || workspace_temp_delete_work(root, name))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitHub CLI: `gh pr list` with fixed flags (JSON when supported). Requires trusted profile, MINT_TERMINAL_ENABLE=1, and `gh` in MINT_TERMINAL_ALLOW.",
        annotations(title = "GitHub PR list (gh)", read_only_hint = true)
    )]
    async fn gh_pr_list(
        &self,
        Parameters(args): Parameters<GhPrListArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let limit = args.limit;
        let state = args.state.clone();
        let out = tokio::task::spawn_blocking(move || gh_pr_list_work(cwd, limit, &state))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitHub CLI: `gh pr view <n> --json …`. Same gates as gh_pr_list.",
        annotations(title = "GitHub PR view (gh)", read_only_hint = true)
    )]
    async fn gh_pr_view(
        &self,
        Parameters(args): Parameters<GhPrViewArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let pr = args.pr_number.clone();
        let out = tokio::task::spawn_blocking(move || gh_pr_view_work(cwd, &pr))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLab CLI: `glab mr list` (JSON when supported). Same gates as terminal_run for `glab` + trusted profile.",
        annotations(title = "GitLab MR list (glab)", read_only_hint = true)
    )]
    async fn glab_mr_list(
        &self,
        Parameters(args): Parameters<GlabMrListArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || glab_mr_list_work(cwd, limit))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLab CLI: `glab mr show <iid>`. Same gates as glab_mr_list.",
        annotations(title = "GitLab MR show (glab)", read_only_hint = true)
    )]
    async fn glab_mr_show(
        &self,
        Parameters(args): Parameters<GlabMrShowArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let cwd = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let iid = args.iid.clone();
        let out = tokio::task::spawn_blocking(move || glab_mr_show_work(cwd, &iid))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read a UTF-8 text file under the project root. For large files use start_line/end_line (1-based, line numbers in output). Whole-file reads are capped by byte and response size; prefer line ranges over repeated full reads.",
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
        description = "Search file contents with regex (or literal=true). Prefer over bulk reads; narrow `path`. For a coarse map first, use codebase_index with symbol_sketch then grep here. Respects .gitignore; tune max_matches.",
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
        let out = tokio::task::spawn_blocking(move || find_files_work(search_root, glob, max_f))
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
        description = "Bounded codebase scan: file counts and bytes per extension, optional heuristic symbol sketch (definition-like lines). One pass; respects .gitignore. Pair with grep_code for exact matches.",
        annotations(title = "Codebase index", read_only_hint = true)
    )]
    async fn codebase_index(
        &self,
        Parameters(args): Parameters<CodebaseIndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_files = args.max_files_scan;
        let sketch = args.symbol_sketch;
        let max_sym = args.max_symbol_entries;

        let out = tokio::task::spawn_blocking(move || {
            codebase_index_work(search_root, max_files, sketch, max_sym)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Lightweight code-database hint: sample up to N files (8 KiB each), count identifier-like tokens, return top frequencies. Read-only; complements codebase_index and grep_code.",
        annotations(title = "Codebase word index", read_only_hint = true)
    )]
    async fn codebase_word_index(
        &self,
        Parameters(args): Parameters<CodebaseWordIndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_files = args.max_files;
        let top_n = args.top_n;
        let out = tokio::task::spawn_blocking(move || {
            codebase_word_index_work(search_root, max_files, top_n)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "SHA-256 of the first N bytes per file (sorted paths). Read-only code-index fingerprint for drift detection; not a security guarantee.",
        annotations(title = "Codebase shallow hash", read_only_hint = true)
    )]
    async fn codebase_shallow_hash(
        &self,
        Parameters(args): Parameters<CodebaseShallowHashArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_files = args.max_files;
        let sample = args.sample_bytes;
        let out = tokio::task::spawn_blocking(move || {
            codebase_shallow_hash_work(search_root, max_files, sample)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Sorted list of relative file paths under a subtree (respects .gitignore). Lighter than codebase_shallow_hash when you only need paths; optional max_depth filter.",
        annotations(title = "Codebase path list", read_only_hint = true)
    )]
    async fn codebase_path_list(
        &self,
        Parameters(args): Parameters<CodebasePathListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_paths = args.max_paths;
        let max_depth = args.max_depth;
        let out = tokio::task::spawn_blocking(move || {
            codebase_path_list_work(search_root, max_paths, max_depth)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Largest files by byte size (bounded stat pass, respects .gitignore). Surfaces heavy assets and generated blobs for cleanup or .gitattributes.",
        annotations(title = "Codebase largest files", read_only_hint = true)
    )]
    async fn codebase_largest_files(
        &self,
        Parameters(args): Parameters<CodebaseLargestFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let scan = args.max_files_scan;
        let top = args.top_n;
        let out = tokio::task::spawn_blocking(move || {
            codebase_largest_files_work(search_root, scan, top)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Find basenames shared by multiple relative paths (import / copy-paste smell). Bounded scan, respects .gitignore.",
        annotations(title = "Codebase duplicate basenames", read_only_hint = true)
    )]
    async fn codebase_duplicate_basenames(
        &self,
        Parameters(args): Parameters<CodebaseDuplicateBasenamesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let scan = args.max_files_scan;
        let out = tokio::task::spawn_blocking(move || {
            codebase_duplicate_basenames_work(search_root, scan)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List crate names from [dependencies] in a Cargo.toml (heuristic scan; not a full TOML parser).",
        annotations(title = "Cargo.toml dependency names", read_only_hint = true)
    )]
    async fn codebase_cargo_dependency_names(
        &self,
        Parameters(args): Parameters<CodebaseCargoDepsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let rel = args.manifest_path.trim();
        let manifest_path = if rel.is_empty() {
            self.root.join("Cargo.toml")
        } else {
            match self.resolve_path(rel) {
                Ok(p) => p,
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            }
        };
        let out = tokio::task::spawn_blocking(move || {
            codebase_cargo_dependency_names_work(manifest_path)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Build or refresh a lightweight symbol index at .mint-mcp/code-index.json (regex sketches per file; no LSP). Blocked on MINT_NETWORK_PROFILE=public. Pair with code_index_search.",
        annotations(title = "Code index build", read_only_hint = false)
    )]
    async fn code_index_build(
        &self,
        Parameters(args): Parameters<CodeIndexBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let subtree = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let respect = args.respect_gitignore;
        let max_files = args.max_files;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_build_work(base, subtree, respect, max_files)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Search the persisted code index (substring or regex on paths/symbols). Read-only; run code_index_build first.",
        annotations(title = "Code index search", read_only_hint = true)
    )]
    async fn code_index_search(
        &self,
        Parameters(args): Parameters<CodeIndexSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let needle = args.needle.clone();
        let cs = args.case_sensitive;
        let limit = args.limit;
        let prefix = args.path_prefix.clone();
        let use_regex = args.use_regex;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_search_work(base, needle, cs, limit, prefix, use_regex)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Show whether .mint-mcp/code-index.json exists and basic metadata (read-only).",
        annotations(title = "Code index status", read_only_hint = true)
    )]
    async fn code_index_status(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let out =
            tokio::task::spawn_blocking(move || crate::code_index::code_index_status_work(base))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List symbols for one file from .mint-mcp/code-index.json (read-only). Path must match index entries (forward slashes).",
        annotations(title = "Code index file symbols", read_only_hint = true)
    )]
    async fn code_index_file_symbols(
        &self,
        Parameters(args): Parameters<CodeIndexFileSymbolsPathArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let rel = args.path.clone();
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_file_symbols_work(base, rel)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Aggregate stats over the symbol index: total symbol rows, top files by symbol count (read-only).",
        annotations(title = "Code index aggregate stats", read_only_hint = true)
    )]
    async fn code_index_aggregate_stats(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_aggregate_stats_work(base)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List indexed file paths whose relative path starts with a prefix (path-prefix match only; not substring search). Read-only.",
        annotations(title = "Code index paths by prefix", read_only_hint = true)
    )]
    async fn code_index_paths_prefix(
        &self,
        Parameters(args): Parameters<CodeIndexPathsPrefixArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let prefix = args.prefix.clone();
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_paths_prefix_work(base, prefix, limit)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Histogram of indexed files by extension (from persisted code index). Read-only.",
        annotations(title = "Code index extension histogram", read_only_hint = true)
    )]
    async fn code_index_extension_histogram(
        &self,
        Parameters(args): Parameters<CodeIndexAnalyticsLimitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_extension_histogram_work(base, limit)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Most frequent symbol strings in the persisted index (counts symbol rows across files). Read-only.",
        annotations(title = "Code index symbol frequency", read_only_hint = true)
    )]
    async fn code_index_symbol_frequency(
        &self,
        Parameters(args): Parameters<CodeIndexAnalyticsLimitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_symbol_frequency_work(base, limit)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List indexed relative paths containing a substring (lighter than code_index_search on symbols). Read-only.",
        annotations(title = "Code index path substring", read_only_hint = true)
    )]
    async fn code_index_path_substring(
        &self,
        Parameters(args): Parameters<CodeIndexPathSubstringArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let needle = args.needle.clone();
        let limit = args.limit;
        let cs = args.case_sensitive;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_path_substring_work(base, needle, limit, cs)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Unique indexed symbols whose names start with prefix (case option)—lighter than full code_index_search when exploring APIs.",
        annotations(title = "Code index symbol prefix", read_only_hint = true)
    )]
    async fn code_index_symbol_prefix(
        &self,
        Parameters(args): Parameters<CodeIndexSymbolPrefixArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let prefix = args.prefix.clone();
        let limit = args.limit;
        let cs = args.case_sensitive;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_symbol_prefix_work(base, prefix, cs, limit)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Symbols appearing in at least min_files distinct indexed paths (name collisions). Read-only; requires code_index_build.",
        annotations(title = "Code index duplicate symbols", read_only_hint = true)
    )]
    async fn code_index_duplicate_symbols(
        &self,
        Parameters(args): Parameters<CodeIndexDuplicateSymbolsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let minf = args.min_files;
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_duplicate_symbols_work(base, minf, limit)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Indexed paths with no recorded symbols (empty parse or not yet extracted)—useful to find files missing from symbol extraction. Read-only; requires code_index_build.",
        annotations(title = "Code index paths without symbols", read_only_hint = true)
    )]
    async fn code_index_paths_without_symbols(
        &self,
        Parameters(args): Parameters<CodeIndexPathsWithoutSymbolsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let base = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let limit = args.limit;
        let out = tokio::task::spawn_blocking(move || {
            crate::code_index::code_index_paths_without_symbols_work(base, limit)
        })
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
        description = "Compact `git diff --stat` summary (read-only). Use staged=true OR revision_range (e.g. main..HEAD), not both.",
        annotations(title = "Git diff stat", read_only_hint = true)
    )]
    async fn git_diff_stat(
        &self,
        Parameters(args): Parameters<GitDiffStatArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let staged = args.staged;
        let rev = args.revision_range.clone();
        let out = tokio::task::spawn_blocking(move || git_diff_stat_work(dir, staged, rev))
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
        description = "git stash list (read-only). Complements git_status when you need saved WIP pointers.",
        annotations(title = "Git stash list", read_only_hint = true)
    )]
    async fn git_stash_list(
        &self,
        Parameters(args): Parameters<GitStashListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_stash_list_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "git worktree list (read-only). Useful for multi-worktree setups alongside git_status.",
        annotations(title = "Git worktree list", read_only_hint = true)
    )]
    async fn git_worktree_list(
        &self,
        Parameters(args): Parameters<GitWorktreeListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_worktree_list_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "git checkout <target>. Mutates working tree; requires MINT_GIT_WRITE_ENABLE=1.",
        annotations(title = "Git checkout (gated write)", read_only_hint = false)
    )]
    async fn git_checkout(
        &self,
        Parameters(args): Parameters<GitCheckoutArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let target = args.target.clone();
        let out = tokio::task::spawn_blocking(move || git_checkout_work(dir, target))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "git stash push (optional -m message). Mutates repo state; requires MINT_GIT_WRITE_ENABLE=1.",
        annotations(title = "Git stash push (gated write)", read_only_hint = false)
    )]
    async fn git_stash_push(
        &self,
        Parameters(args): Parameters<GitStashPushArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let message = args.message.clone();
        let out = tokio::task::spawn_blocking(move || git_stash_push_work(dir, message))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style alias: `git stash push` with optional message (`name`). Same gates as git_stash_push (trusted network + MINT_GIT_WRITE_ENABLE).",
        annotations(title = "Git stash (GitLens-style)", read_only_hint = false)
    )]
    async fn git_stash(
        &self,
        Parameters(args): Parameters<GitStashLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let message = args.name.clone();
        let out = tokio::task::spawn_blocking(move || git_stash_push_work(dir, message))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: `git log --oneline` or `git diff` with optional revision_range, since/until (log), authors (log). Read-only.",
        annotations(title = "Git log or diff (GitLens-style)", read_only_hint = true)
    )]
    async fn git_log_or_diff(
        &self,
        Parameters(args): Parameters<GitLogOrDiffLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let rr = args.revision_range.clone();
        let since = args.since.clone();
        let until = args.until.clone();
        let authors = args.authors.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_log_or_diff_work(dir, &action, rr, since, until, authors)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: list branches (`git branch -vv`) or create a branch (trusted + MINT_GIT_WRITE_ENABLE).",
        annotations(title = "Git branch (GitLens-style)", read_only_hint = false)
    )]
    async fn git_branch(
        &self,
        Parameters(args): Parameters<GitBranchLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.action.trim().eq_ignore_ascii_case("create") {
            if let Err(e) = self.require_trusted_deployment() {
                return Self::tool_text(e);
            }
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let bn = args.branch_name.clone();
        let out = tokio::task::spawn_blocking(move || git_branch_extension_work(dir, &action, bn))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: list worktrees or add one at `path` tracking `branch` (trusted + MINT_GIT_WRITE_ENABLE for add).",
        annotations(title = "Git worktree (GitLens-style)", read_only_hint = false)
    )]
    async fn git_worktree(
        &self,
        Parameters(args): Parameters<GitWorktreeLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.action.trim().eq_ignore_ascii_case("add") {
            if let Err(e) = self.require_trusted_deployment() {
                return Self::tool_text(e);
            }
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let wp = match &args.path {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => Some(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };
        let action = args.action.clone();
        let branch = args.branch.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_worktree_extension_work(dir, &action, wp, branch)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: `git add` or `git commit` (trusted network + MINT_GIT_WRITE_ENABLE).",
        annotations(title = "Git add or commit (GitLens-style)", read_only_hint = false)
    )]
    async fn git_add_or_commit(
        &self,
        Parameters(args): Parameters<GitAddOrCommitLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let files = args.files.clone();
        let message = args.message.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_add_or_commit_work(dir, &action, files, message)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: `git push`. Requires trusted network (not MINT_NETWORK_PROFILE=public), MINT_GIT_PUSH_ENABLE=1, and explicit operator approval for remotes.",
        annotations(title = "Git push (GitLens-style)", read_only_hint = false)
    )]
    async fn git_push(
        &self,
        Parameters(args): Parameters<GitPushLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_push_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: `git fetch --prune`. Requires trusted network (not public profile), MINT_GIT_REMOTE_ENABLE=1. Contacts remotes.",
        annotations(title = "Git fetch (GitLens-style)", read_only_hint = false)
    )]
    async fn git_fetch(
        &self,
        Parameters(args): Parameters<GitFetchLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_fetch_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitLens-style: `git pull --ff-only`. Requires trusted network, MINT_GIT_REMOTE_ENABLE=1. Contacts remotes; refuses non-FF merges.",
        annotations(title = "Git pull (GitLens-style)", read_only_hint = false)
    )]
    async fn git_pull(
        &self,
        Parameters(args): Parameters<GitPullLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_pull_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: `git remote -v` (configured remotes; does not contact the network).",
        annotations(title = "Git remote list", read_only_hint = true)
    )]
    async fn git_remote_list(
        &self,
        Parameters(args): Parameters<GitStashListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_working_dir(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_remote_list_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: `git tag -l` with optional pattern. Local GitLens-style repo introspection.",
        annotations(title = "Git tag list", read_only_hint = true)
    )]
    async fn git_tag_list(
        &self,
        Parameters(args): Parameters<GitTagListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let pat = args.pattern.clone();
        let out = tokio::task::spawn_blocking(move || git_tag_list_work(dir, pat))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: `git ls-files` — mode tracked | untracked | all (untracked uses -o --exclude-standard). Path index for agents.",
        annotations(title = "Git ls-files", read_only_hint = true)
    )]
    async fn git_ls_files(
        &self,
        Parameters(args): Parameters<GitLsFilesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let mode = args.mode.clone();
        let out = tokio::task::spawn_blocking(move || git_ls_files_work(dir, &mode))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: `git describe --tags --always --dirty` (human-friendly version pointer; no network).",
        annotations(title = "Git describe", read_only_hint = true)
    )]
    async fn git_describe(
        &self,
        Parameters(args): Parameters<GitDescribeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_describe_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: `git shortlog` top contributors by commit count (reachable from HEAD).",
        annotations(title = "Git shortlog", read_only_hint = true)
    )]
    async fn git_shortlog(
        &self,
        Parameters(args): Parameters<GitShortlogArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let lim = args.limit;
        let out = tokio::task::spawn_blocking(move || git_shortlog_work(dir, lim))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: recent `git reflog` lines (HEAD moves). Useful after checkouts/resets.",
        annotations(title = "Git reflog", read_only_hint = true)
    )]
    async fn git_reflog(
        &self,
        Parameters(args): Parameters<GitReflogArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let lim = args.limit;
        let out = tokio::task::spawn_blocking(move || git_reflog_work(dir, lim))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read-only: latest commit (oneline, subject, full body, or medium one-line).",
        annotations(title = "Git last commit", read_only_hint = true)
    )]
    async fn git_last_commit(
        &self,
        Parameters(args): Parameters<GitLastCommitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let style = args.style.clone();
        let out = tokio::task::spawn_blocking(move || git_last_commit_work(dir, &style))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Local subset of GitLens repository_get_file_content: read working-tree file or `git show ref:path` under the repo. Ignores cloud provider fields.",
        annotations(title = "Repository get file content (local)", read_only_hint = true)
    )]
    async fn repository_get_file_content(
        &self,
        Parameters(args): Parameters<RepositoryGetFileContentArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let fp = args.file_path.clone();
        let gr = args.git_ref.clone();
        let out =
            tokio::task::spawn_blocking(move || repository_get_file_content_work(dir, fp, gr))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP schema alias: `git status --short --branch` using required `directory` (empty = project root). Same output as git_status.",
        annotations(title = "Git status (GitKraken arg shape)", read_only_hint = true)
    )]
    async fn gk_git_status(
        &self,
        Parameters(args): Parameters<GkGitStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_status_work(dir))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP schema alias: `git checkout` with `branch` + `directory` (empty directory = project root). Same gates as git_checkout.",
        annotations(title = "Git checkout (GitKraken arg shape)", read_only_hint = false)
    )]
    async fn gk_git_checkout(
        &self,
        Parameters(args): Parameters<GkGitCheckoutArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let target = args.branch.clone();
        let out = tokio::task::spawn_blocking(move || git_checkout_work(dir, target))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_log_or_diff (same parameters as GitLens-style tool).",
        annotations(title = "Git log or diff (GitKraken name)", read_only_hint = true)
    )]
    async fn gk_git_log_or_diff(
        &self,
        Parameters(args): Parameters<GitLogOrDiffLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let rr = args.revision_range.clone();
        let since = args.since.clone();
        let until = args.until.clone();
        let authors = args.authors.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_log_or_diff_work(dir, &action, rr, since, until, authors)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias: git blame on first 400 lines (directory + file only).",
        annotations(title = "Git blame (GitKraken shape)", read_only_hint = true)
    )]
    async fn gk_git_blame(
        &self,
        Parameters(args): Parameters<GkGitBlameArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repo = match self.git_directory_arg(&args.directory) {
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
        let out = tokio::task::spawn_blocking(move || git_blame_work(repo, file_rel, 1, 400, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_branch.",
        annotations(title = "Git branch (GitKraken name)", read_only_hint = false)
    )]
    async fn gk_git_branch(
        &self,
        Parameters(args): Parameters<GitBranchLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.action.trim().eq_ignore_ascii_case("create") {
            if let Err(e) = self.require_trusted_deployment() {
                return Self::tool_text(e);
            }
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let bn = args.branch_name.clone();
        let out = tokio::task::spawn_blocking(move || git_branch_extension_work(dir, &action, bn))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_stash (stash push + optional name).",
        annotations(title = "Git stash (GitKraken name)", read_only_hint = false)
    )]
    async fn gk_git_stash(
        &self,
        Parameters(args): Parameters<GitStashLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let message = args.name.clone();
        let out = tokio::task::spawn_blocking(move || git_stash_push_work(dir, message))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_worktree.",
        annotations(title = "Git worktree (GitKraken name)", read_only_hint = false)
    )]
    async fn gk_git_worktree(
        &self,
        Parameters(args): Parameters<GitWorktreeLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.action.trim().eq_ignore_ascii_case("add") {
            if let Err(e) = self.require_trusted_deployment() {
                return Self::tool_text(e);
            }
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let wp = match &args.path {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => Some(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };
        let action = args.action.clone();
        let branch = args.branch.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_worktree_extension_work(dir, &action, wp, branch)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_add_or_commit.",
        annotations(title = "Git add or commit (GitKraken name)", read_only_hint = false)
    )]
    async fn gk_git_add_or_commit(
        &self,
        Parameters(args): Parameters<GitAddOrCommitLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let action = args.action.clone();
        let files = args.files.clone();
        let message = args.message.clone();
        let out = tokio::task::spawn_blocking(move || {
            git_add_or_commit_work(dir, &action, files, message)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "GitKraken MCP alias for git_push.",
        annotations(title = "Git push (GitKraken name)", read_only_hint = false)
    )]
    async fn gk_git_push(
        &self,
        Parameters(args): Parameters<GitPushLensArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || git_push_work(dir))
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
        let out =
            tokio::task::spawn_blocking(move || git_log_work(repo, limit, path_in_repo, oneline))
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
        description = "Read-only `git merge-base A B` — common ancestor for comparing branches (local GitLens-style).",
        annotations(title = "Git merge-base", read_only_hint = true)
    )]
    async fn git_merge_base(
        &self,
        Parameters(args): Parameters<GitMergeBaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match self.git_directory_arg(&args.directory) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let a = args.commit_a.clone();
        let b = args.commit_b.clone();
        let out = tokio::task::spawn_blocking(move || git_merge_base_work(dir, a, b))
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
        description = "Deterministically compress text for prompt budgeting (truncate, head_tail, lines_only, dense). Combine with pack_context and summarize_* for layered context shaping. No external models.",
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
        description = "Fast UTF-8 stats (bytes, chars, lines, words, paragraph-like blocks, rough token estimates chars/4 and chars/1.3) for prompt sizing; use summarize_token_estimate for a focused report.",
        annotations(title = "Text stats", read_only_hint = true)
    )]
    async fn text_stats(
        &self,
        Parameters(args): Parameters<TextStatsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let out = tokio::task::spawn_blocking(move || text_stats_work(text))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic token *estimates* (chars/4 and chars/1.3 heuristics) plus word/line counts—use before pack_context or bus posts; not a real tokenizer.",
        annotations(title = "Summarize token estimate", read_only_hint = true)
    )]
    async fn summarize_token_estimate(
        &self,
        Parameters(args): Parameters<SummarizeTokenEstimateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_token_estimate_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Collapse internal whitespace on each line (lines preserved). Complements compress_unique_lines for log cleanup.",
        annotations(title = "Compress whitespace per line", read_only_hint = true)
    )]
    async fn compress_ws_collapse(
        &self,
        Parameters(args): Parameters<CompressWsCollapseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_ws_collapse_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Per line: join whitespace-separated tokens with single spaces (stronger simplification than compress_ws_collapse). Good before summarize_* on noisy logs.",
        annotations(title = "Compress simplify whitespace", read_only_hint = true)
    )]
    async fn compress_simplify_whitespace(
        &self,
        Parameters(args): Parameters<CompressSimplifyWhitespaceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_simplify_whitespace_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Heuristic: replace ![alt](url) with alt and [text](url) with text (nested brackets may truncate). Shrinks markdown before bus posts.",
        annotations(title = "Compress markdown links", read_only_hint = true)
    )]
    async fn compress_markdown_links(
        &self,
        Parameters(args): Parameters<CompressMarkdownLinksArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_markdown_links_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Count hunks and +/- lines in unified diff text (e.g. paste git diff). Ignores +++ --- headers. Safe on public profile.",
        annotations(title = "Compress unified diff stats", read_only_hint = true)
    )]
    async fn compress_unified_diff_stats(
        &self,
        Parameters(args): Parameters<CompressUnifiedDiffStatsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_unified_diff_stats_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Truncate each line to max_line_chars Unicode scalars (suffix …). Tames wide logs and TSV before pack_context or bus posts.",
        annotations(title = "Compress truncate lines", read_only_hint = true)
    )]
    async fn compress_truncate_lines(
        &self,
        Parameters(args): Parameters<CompressTruncateLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let ml = args.max_line_chars;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_truncate_lines_work(text, ml, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List unique file paths from unified diff --- / +++ headers (git-style); sorted, capped. Pair with compress_unified_diff_stats.",
        annotations(title = "Summarize diff file list", read_only_hint = true)
    )]
    async fn summarize_diff_file_list(
        &self,
        Parameters(args): Parameters<SummarizeDiffFileListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mp = args.max_paths;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_diff_file_list_work(text, mp, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic top token counts (simple English stopword list; not an ML keyword extractor).",
        annotations(title = "Summarize top terms", read_only_hint = true)
    )]
    async fn summarize_top_terms(
        &self,
        Parameters(args): Parameters<SummarizeTopTermsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let minl = args.min_token_len;
        let maxt = args.max_terms;
        let mc = args.max_chars;
        let out =
            tokio::task::spawn_blocking(move || summarize_top_terms_work(text, minl, maxt, mc))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Frequency of identifier-shaped tokens [A-Za-z_][A-Za-z0-9_]* (code/logs). Unlike summarize_top_terms, no English stopword list; optional fold_case merges Foo/foo. Safe on public profile.",
        annotations(title = "Summarize identifier frequency", read_only_hint = true)
    )]
    async fn summarize_identifier_frequency(
        &self,
        Parameters(args): Parameters<SummarizeIdentifierFrequencyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let minl = args.min_len;
        let maxi = args.max_identifiers;
        let mc = args.max_chars;
        let fold = args.fold_case;
        let out = tokio::task::spawn_blocking(move || {
            summarize_identifier_frequency_work(text, minl, maxi, mc, fold)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Heuristic strip of HTML/XML-like tags (does not parse full markup; preserves `<` that are not tag starts).",
        annotations(title = "Compress strip tags", read_only_hint = true)
    )]
    async fn compress_strip_tags(
        &self,
        Parameters(args): Parameters<CompressStripTagsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_strip_tags_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Replace ```…``` fenced code blocks with a placeholder line (keeps structure outside fences). Shrinks pasted logs before bus or pack_context.",
        annotations(title = "Compress fenced code blocks", read_only_hint = true)
    )]
    async fn compress_fenced_code_blocks(
        &self,
        Parameters(args): Parameters<CompressFencedCodeBlocksArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_fenced_code_blocks_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Remove lines where the first non-whitespace characters are // (full-line C/JS/Rust comments). Inline code on the same line is kept. Optional drop_blank_lines. Safe on public profile.",
        annotations(title = "Compress full-line // comments", read_only_hint = true)
    )]
    async fn compress_double_slash_line_comments(
        &self,
        Parameters(args): Parameters<CompressDoubleSlashLineCommentsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let dbl = args.drop_blank_lines;
        let out = tokio::task::spawn_blocking(move || {
            compress_double_slash_line_comments_work(text, mc, dbl)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Replace http(s) URLs with [url] placeholders (safer log sharing). Deterministic regex; pair with compress_redact_emails.",
        annotations(title = "Compress redact URLs", read_only_hint = true)
    )]
    async fn compress_redact_urls(
        &self,
        Parameters(args): Parameters<PrivacyRedactTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_redact_urls_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Replace email-shaped tokens with [email] (privacy). Does not understand quoted names or obfuscated addresses.",
        annotations(title = "Compress redact emails", read_only_hint = true)
    )]
    async fn compress_redact_emails(
        &self,
        Parameters(args): Parameters<PrivacyRedactTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_redact_emails_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Strip leading YAML-style markdown frontmatter (first line --- through closing ---). If no frontmatter, returns original text.",
        annotations(title = "Compress strip markdown frontmatter", read_only_hint = true)
    )]
    async fn compress_markdown_frontmatter(
        &self,
        Parameters(args): Parameters<PrivacyRedactTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_markdown_frontmatter_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Remove lines that match a regex (per-line is_match). Inverse of keeping grep hits; useful log noise stripping.",
        annotations(title = "Compress drop lines matching regex", read_only_hint = true)
    )]
    async fn compress_drop_lines_matching_regex(
        &self,
        Parameters(args): Parameters<CompressDropLinesRegexArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let pat = args.pattern.clone();
        let ci = args.case_insensitive;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            compress_drop_lines_matching_regex_work(text, pat, ci, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List lines that repeat at least min_occurrences times (exact line match). Useful for log dedup summaries.",
        annotations(title = "Summarize duplicate lines", read_only_hint = true)
    )]
    async fn summarize_duplicate_lines(
        &self,
        Parameters(args): Parameters<SummarizeDuplicateLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mo = args.min_occurrences;
        let me = args.max_entries;
        let mc = args.max_chars;
        let out =
            tokio::task::spawn_blocking(move || summarize_duplicate_lines_work(text, mo, me, mc))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Histogram of line lengths (Unicode scalar counts) into coarse buckets; suggests compress_lines_tail / compress_truncate_middle for long-line files.",
        annotations(title = "Summarize line length histogram", read_only_hint = true)
    )]
    async fn summarize_line_histogram(
        &self,
        Parameters(args): Parameters<SummarizeLineHistogramArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_line_histogram_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Group lines by first N characters (Unicode scalars); lists top prefixes by frequency. Useful for loggers that prefix timestamps or levels.",
        annotations(title = "Summarize line prefix groups", read_only_hint = true)
    )]
    async fn summarize_line_prefix_groups(
        &self,
        Parameters(args): Parameters<SummarizeLinePrefixGroupsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let pl = args.prefix_len;
        let mg = args.max_groups;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            summarize_line_prefix_groups_work(text, pl, mg, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Rank lines by how often given keywords appear as whole tokens (case-insensitive). Deterministic heuristic — not embedding-based ML.",
        annotations(title = "Summarize rank lines by keywords", read_only_hint = true)
    )]
    async fn summarize_rank_lines_by_keywords(
        &self,
        Parameters(args): Parameters<SummarizeRankLinesByKeywordsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let kw = args.keywords.clone();
        let tl = args.top_lines;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            summarize_rank_lines_by_keywords_work(text, kw, tl, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Prefix every line with a string (default '> '). Useful for quoting excerpts or simulating comment prefixes.",
        annotations(title = "Format prefix lines", read_only_hint = true)
    )]
    async fn format_prefix_lines(
        &self,
        Parameters(args): Parameters<FormatPrefixLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let prefix = args.prefix.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || format_prefix_lines_work(text, prefix, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Keep the first N non-empty paragraphs (blank-line separated), then cap by characters. Good for long markdown or multi-section logs.",
        annotations(title = "Compress paragraphs", read_only_hint = true)
    )]
    async fn compress_paragraphs(
        &self,
        Parameters(args): Parameters<CompressParagraphsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mp = args.max_paragraphs;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_paragraphs_work(text, mp, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic line dedupe: keep first occurrence of each unique line in order, then cap by max_lines and max_chars. Shrinks repetitive logs and diffs.",
        annotations(title = "Compress unique lines", read_only_hint = true)
    )]
    async fn compress_unique_lines(
        &self,
        Parameters(args): Parameters<CompressUniqueLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let ml = args.max_lines;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_unique_lines_work(text, ml, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Keep the first N lines in order (no deduplication), then cap by characters. Use compress_unique_lines for repetitive logs.",
        annotations(title = "Compress lines (head)", read_only_hint = true)
    )]
    async fn compress_lines_head(
        &self,
        Parameters(args): Parameters<CompressLinesHeadArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let hl = args.head_lines;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_lines_head_work(text, hl, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Keep the last N lines (e.g. build / test log tails), then cap by characters.",
        annotations(title = "Compress lines (tail)", read_only_hint = true)
    )]
    async fn compress_lines_tail(
        &self,
        Parameters(args): Parameters<CompressLinesTailArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let tl = args.tail_lines;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_lines_tail_work(text, tl, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Collapse runs of blank lines to a single blank line, then cap by characters (same transform as convert_format collapse_blank_lines).",
        annotations(title = "Compress blank lines", read_only_hint = true)
    )]
    async fn compress_collapse_blank_lines(
        &self,
        Parameters(args): Parameters<CompressCollapseBlankLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_collapse_blank_lines_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "When text exceeds max_chars, keep margin_chars at start and end and omit the middle (Unicode-aware).",
        annotations(title = "Compress truncate middle", read_only_hint = true)
    )]
    async fn compress_truncate_middle(
        &self,
        Parameters(args): Parameters<CompressTruncateMiddleArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mx = args.max_chars;
        let margin = args.margin_chars;
        let out =
            tokio::task::spawn_blocking(move || compress_truncate_middle_work(text, mx, margin))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Remove common leading space/tab prefix from non-empty lines (dedent pasted code blocks).",
        annotations(title = "Compress dedent", read_only_hint = true)
    )]
    async fn compress_dedent(
        &self,
        Parameters(args): Parameters<CompressDedentArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_dedent_work(text, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Keep the first N sentences (split after . ? ! + whitespace), then cap by characters. Complements compress_paragraphs and compress_context.",
        annotations(title = "Compress sentences", read_only_hint = true)
    )]
    async fn compress_sentences(
        &self,
        Parameters(args): Parameters<CompressSentencesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let ms = args.max_sentences;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || compress_sentences_work(text, ms, mc))
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
        description = "Read a file (optional line slice) then compress with the same modes as compress_context (head_tail, truncate, lines_only, dense).",
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
        description = "Extractive summary: scores sentences by term overlap (no LLM). Good for long prose/logs; pair with compress_context for size caps.",
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
        description = "Extract markdown-style bullet / numbered list lines in order for a fast outline (no LLM). Pairs with summarize_text on the same doc.",
        annotations(title = "Summarize bullets (outline)", read_only_hint = true)
    )]
    async fn summarize_bullets(
        &self,
        Parameters(args): Parameters<SummarizeBulletsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mi = args.max_items;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_bullets_work(text, mi, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Extract markdown ATX headings in order for a document outline (no LLM).",
        annotations(title = "Summarize headings", read_only_hint = true)
    )]
    async fn summarize_headings(
        &self,
        Parameters(args): Parameters<SummarizeHeadingsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let mh = args.max_headings;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_headings_work(text, mh, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Indented bullet table-of-contents from ATX markdown headings (tree-style outline; complements flat summarize_headings).",
        annotations(title = "Summarize markdown TOC", read_only_hint = true)
    )]
    async fn summarize_markdown_toc(
        &self,
        Parameters(args): Parameters<SummarizeMarkdownTocArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let me = args.max_entries;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_markdown_toc_work(text, me, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Parse JSON and list paths with types, array lengths, and shallow nesting—fast schema skim without pretty-printing large values. Safe on public profile.",
        annotations(title = "Summarize JSON structure", read_only_hint = true)
    )]
    async fn summarize_json_structure(
        &self,
        Parameters(args): Parameters<SummarizeJsonStructureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let md = args.max_depth;
        let mk = args.max_keys_per_level;
        let ml = args.max_lines;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            summarize_json_structure_work(text, md, mk, ml, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Split the first line of CSV/TSV/pipe/semicolon text into column labels (schema teaser). Safe on public profile.",
        annotations(title = "Summarize delimited header", read_only_hint = true)
    )]
    async fn summarize_delimited_header(
        &self,
        Parameters(args): Parameters<SummarizeDelimitedHeaderArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let delim = args.delimiter.clone();
        let mx = args.max_columns;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            summarize_delimited_header_work(text, &delim, mx, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Levenshtein edit distance on Unicode characters (bounded n×m for CPU safety). Compare agent drafts, filenames, or test output.",
        annotations(title = "Text Levenshtein distance", read_only_hint = true)
    )]
    async fn text_levenshtein_distance(
        &self,
        Parameters(args): Parameters<TextLevenshteinArgs>,
    ) -> Result<CallToolResult, McpError> {
        let a = args.a.clone();
        let b = args.b.clone();
        let m = args.max_len_each;
        let out = tokio::task::spawn_blocking(move || text_levenshtein_distance_work(a, b, m))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Whitespace token Jaccard similarity between two texts (0–1): overlap of distinct tokens; optional ASCII folding via fold_case. Capped for CPU—see approximate_* flags when inputs are huge. Useful for comparing agent drafts, logs, or requirements blurbs without an LLM.",
        annotations(title = "Text token Jaccard similarity", read_only_hint = true)
    )]
    async fn text_token_jaccard(
        &self,
        Parameters(args): Parameters<TextTokenJaccardArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ta = args.text_a.clone();
        let tb = args.text_b.clone();
        let fold = args.fold_case;
        let mc = args.max_chars_each;
        let ms = args.max_tokens_scanned_per_text;
        let out =
            tokio::task::spawn_blocking(move || text_token_jaccard_work(ta, tb, fold, mc, ms))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Longest contiguous substring common to two texts (Unicode scalar match). CPU-bounded via max_len_each and n×m cap—pair with text_levenshtein_distance for edit distance vs shared spans.",
        annotations(title = "Text longest common substring", read_only_hint = true)
    )]
    async fn text_longest_common_substring(
        &self,
        Parameters(args): Parameters<TextLongestCommonSubstringArgs>,
    ) -> Result<CallToolResult, McpError> {
        let a = args.a.clone();
        let b = args.b.clone();
        let m = args.max_len_each;
        let out = tokio::task::spawn_blocking(move || text_longest_common_substring_work(a, b, m))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "DJB2 hash (32-bit) over UTF-8 bytes—cheap fingerprint for caching, dedupe hints, or comparing blobs without storing full text.",
        annotations(title = "Text DJB2 hash", read_only_hint = true)
    )]
    async fn text_djb2_hash(
        &self,
        Parameters(args): Parameters<TextDjb2HashArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let out = tokio::task::spawn_blocking(move || text_djb2_hash_work(text))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "First N non-empty lines—quick teaser for READMEs, logs, or agent dumps.",
        annotations(title = "Summarize first lines", read_only_hint = true)
    )]
    async fn summarize_first_lines(
        &self,
        Parameters(args): Parameters<SummarizeFirstLinesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let lc = args.line_count;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || summarize_first_lines_work(text, lc, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Merge multiple text segments (e.g. parallel agent outputs) with deterministic separators, then run the same extractive summarizer as summarize_text.",
        annotations(title = "Summarize segments", read_only_hint = true)
    )]
    async fn summarize_segments(
        &self,
        Parameters(args): Parameters<SummarizeSegmentsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut segs = args.segments;
        segs.truncate(24);
        let max_c = args.max_chars;
        let max_s = args.max_sentences;
        let out = tokio::task::spawn_blocking(move || summarize_segments_work(segs, max_c, max_s))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read a UTF-8 file (optional line range) and run the same summarizer as summarize_text—one step for docs and logs.",
        annotations(title = "Summarize file", read_only_hint = true)
    )]
    async fn summarize_file(
        &self,
        Parameters(args): Parameters<SummarizeFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let start = args.start_line;
        let end = args.end_line;
        let max_c = args.max_chars;
        let max_s = args.max_sentences;

        let out = tokio::task::spawn_blocking(move || {
            summarize_file_work(path, start, end, max_c, max_s)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Read up to 8 files (first lines_per_file lines each), merge, then one extractive summary—parallel multi-file digest in one round-trip.",
        annotations(title = "Summarize multiple files", read_only_hint = true)
    )]
    async fn summarize_multi_file(
        &self,
        Parameters(args): Parameters<SummarizeMultiFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.paths.is_empty() {
            return Self::tool_text("Error: paths must not be empty".to_string());
        }
        let mut paths: Vec<PathBuf> = Vec::new();
        for p in &args.paths {
            match self.resolve_path(p) {
                Ok(pb) => paths.push(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            }
        }
        let lines = args.lines_per_file;
        let max_c = args.max_chars;
        let max_s = args.max_sentences;
        let out = tokio::task::spawn_blocking(move || {
            summarize_multi_file_work(paths, lines, max_c, max_s)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Pack multiple context segments into one bounded string, then compress. Optional segment_labels (same length as segments) name each block for parallel/serial agent handoffs.",
        annotations(title = "Pack context", read_only_hint = true)
    )]
    async fn pack_context(
        &self,
        Parameters(args): Parameters<PackContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let PackContextArgs {
            segments,
            segment_labels,
            max_chars,
            mode,
            dedupe_lines,
            collapse_whitespace,
        } = args;
        let out = tokio::task::spawn_blocking(move || {
            pack_context_work(
                segments,
                segment_labels,
                max_chars,
                mode,
                dedupe_lines,
                collapse_whitespace,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Safe format helpers: JSON pretty/minify; YAML→JSON (yaml_to_json, yaml_to_json_minify) and JSON→YAML; TOML→JSON (toml_to_json, toml_to_json_minify) and JSON→TOML (null-free); yaml_top_level_keys / toml_top_level_keys for quick config skims; base64/hex; sha256_hex; utf8_bom_strip; line endings; tabs/spaces; strip_ansi; blank line ops; slugify; sort/reverse/trim lines; percent_encode/decode; html_escape. Max 256 KiB/file. output_path disallowed when MINT_NETWORK_PROFILE=public.",
        annotations(title = "Convert format", read_only_hint = true)
    )]
    async fn convert_format(
        &self,
        Parameters(args): Parameters<ConvertFormatArgs>,
    ) -> Result<CallToolResult, McpError> {
        let input_path = match &args.input_path {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => Some(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };
        let output_path = match &args.output_path {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => Some(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };

        let op = args.operation.clone();
        let text = args.text.clone();
        let tab_stops = args.tab_stops;
        let line_number_start = args.line_number_start;
        let wrap_width = args.wrap_width;
        let allow_out = !self.network_profile.is_public();
        let out = tokio::task::spawn_blocking(move || {
            convert_format_work(
                op,
                text,
                input_path,
                output_path,
                allow_out,
                tab_stops,
                line_number_start,
                wrap_width,
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Gzip UTF-8 bytes (from text or input_path under root, max 256 KiB) and return standard base64—compact payloads for agent handoffs or bus posts. Pure Rust; safe on public profile.",
        annotations(title = "Compress gzip base64", read_only_hint = true)
    )]
    async fn compress_gzip_base64(
        &self,
        Parameters(args): Parameters<CompressGzipBase64Args>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.filter(|s| !s.is_empty());
        let path_opt = match args.input_path.as_ref() {
            Some(p) if !p.trim().is_empty() => match self.resolve_path(p) {
                Ok(pb) => Some(pb),
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            },
            _ => None,
        };
        match (&text, &path_opt) {
            (Some(_), Some(_)) => {
                return Self::tool_text(
                    "Error: provide only one of text or input_path".to_string(),
                );
            }
            (None, None) => {
                return Self::tool_text("Error: provide text or input_path".to_string());
            }
            _ => {}
        }
        let out =
            tokio::task::spawn_blocking(move || compress_gzip_base64_work(text, path_opt.as_ref()))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Decode standard base64 gzip to UTF-8 (or report binary as base64). Decompressed output capped (~2 MiB) to mitigate zip bombs. Safe on public profile.",
        annotations(title = "Decompress gzip base64", read_only_hint = true)
    )]
    async fn decompress_gzip_base64(
        &self,
        Parameters(args): Parameters<DecompressGzipBase64Args>,
    ) -> Result<CallToolResult, McpError> {
        let pay = args.payload_base64.clone();
        let out = tokio::task::spawn_blocking(move || decompress_gzip_base64_work(pay))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Walk a subtree (respecting gitignore) and write path/size/mtime JSONL to .mint-mcp/file-manifest.jsonl for a stable, grep-friendly code index snapshot.",
        annotations(title = "Codebase manifest write", read_only_hint = false)
    )]
    async fn codebase_manifest_write(
        &self,
        Parameters(args): Parameters<CodebaseManifestArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let search_root = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_files = args.max_files;
        let out = tokio::task::spawn_blocking(move || {
            codebase_manifest_write_work(search_root, max_files)
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
        description = "Append one JSONL message (flock-safe). Omit to_agent to broadcast; set topic for agent_thread. For many DMs in one lock use agent_fan_out; for ordered multi-post use agent_send_pipeline. Mutates .mint-mcp/agent-bus.jsonl only.",
        annotations(title = "Agent bus: send", read_only_hint = false)
    )]
    async fn agent_send(
        &self,
        Parameters(args): Parameters<AgentSendArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only (unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY or set to 0/false to allow agent_send)."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if let Err(e) = self.bus_rate.try_record_send(&args.from_agent) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let body = args.message.clone();
        let to = args.to_agent.clone();
        let topic = args.topic.clone();
        let kind = args.kind.clone();

        let out = tokio::task::spawn_blocking(move || match crate::bus::agent_send(root, from, body, to.clone(), topic.clone(), kind.clone()) {
            Ok(id) => {
                let mut s = format!("OK message_id={id}\n");
                if let Some(ref t) = topic {
                    if !t.is_empty() {
                        s.push_str(&format!("topic: {t}\n"));
                    }
                }
                if let Some(ref t) = to {
                    if !t.is_empty() {
                        s.push_str(&format!("to_agent: {t}\n"));
                    }
                }
                s.push_str("Other agents should poll `agent_inbox`, `agent_recent`, or `agent_thread` (when topic set).");
                s
            }
            Err(e) => format!("Error: {e}"),
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Structured DM with kind=handoff (headline + body + optional correlation_id). Same gates as agent_send; use for serial multi-agent handoffs.",
        annotations(title = "Agent bus: handoff", read_only_hint = false)
    )]
    async fn agent_handoff(
        &self,
        Parameters(args): Parameters<AgentHandoffArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only (unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY or set to 0/false to allow agent_handoff)."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.to_agent) {
            return Self::tool_text(
                "Error: to_agent must be a non-empty identifier (letters, digits, _.-)".to_string(),
            );
        }
        let from_n = crate::bus::normalized_agent_id(&args.from_agent);
        let to_n = crate::bus::normalized_agent_id(&args.to_agent);
        if from_n == to_n {
            return Self::tool_text("Error: to_agent must differ from from_agent".to_string());
        }
        if let Err(e) = self.bus_rate.try_record_send(&args.from_agent) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let hl = args.headline.trim();
        if hl.is_empty() {
            return Self::tool_text("Error: headline must not be empty".to_string());
        }
        let body_trim = args.body.trim();
        if body_trim.is_empty() {
            return Self::tool_text("Error: body must not be empty".to_string());
        }

        let cid = args
            .correlation_id
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("-");
        let mut msg = String::new();
        msg.push_str("HANDOFF\n");
        msg.push_str(&format!("headline: {hl}\n"));
        msg.push_str(&format!("correlation_id: {cid}\n"));
        msg.push_str("---\n");
        msg.push_str(body_trim);

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let topic = args.topic.clone();
        let to = args.to_agent.clone();

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_send(
                root,
                from,
                msg,
                Some(to),
                topic.clone(),
                Some("handoff".to_string()),
            ) {
                Ok(id) => {
                    let mut s = format!("OK message_id={id} kind=handoff\n");
                    if let Some(ref t) = topic {
                        if !t.is_empty() {
                            s.push_str(&format!("topic: {t}\n"));
                        }
                    }
                    s.push_str("Recipient should poll agent_inbox or agent_thread when topic set.");
                    s
                }
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Structured handoff: DM a sub-agent with kind=subagent_task (title + body + optional correlation_id). Same bus rules as agent_send; use a stable sub_agent id and optional topic for agent_thread. Draft bodies with plan_handoff_brief + thinking_plan_skeleton; parallelize via plan_work_buckets + agent_fan_out.",
        annotations(title = "Agent bus: subagent task", read_only_hint = false)
    )]
    async fn agent_subagent_task(
        &self,
        Parameters(args): Parameters<AgentSubagentTaskArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only (unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY or set to 0/false to allow agent_subagent_task)."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.sub_agent) {
            return Self::tool_text(
                "Error: sub_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        let parent = crate::bus::normalized_agent_id(&args.from_agent);
        let sub = crate::bus::normalized_agent_id(&args.sub_agent);
        if sub == parent {
            return Self::tool_text("Error: sub_agent must differ from from_agent".to_string());
        }
        if let Err(e) = self.bus_rate.try_record_send(&args.from_agent) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let title = args.title.trim();
        if title.is_empty() {
            return Self::tool_text("Error: title must not be empty".to_string());
        }
        let body_trim = args.task_body.trim();
        if body_trim.is_empty() {
            return Self::tool_text("Error: task_body must not be empty".to_string());
        }

        let cid = args
            .correlation_id
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("-");
        let mut body = String::new();
        body.push_str("SUBAGENT_TASK\n");
        body.push_str(&format!("title: {title}\n"));
        body.push_str(&format!("correlation_id: {cid}\n"));
        body.push_str("---\n");
        body.push_str(body_trim);

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let topic = args.topic.clone();

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_send(
                root,
                from,
                body,
                Some(sub),
                topic.clone(),
                Some("subagent_task".to_string()),
            ) {
                Ok(id) => {
                    let mut s = format!("OK message_id={id} kind=subagent_task\n");
                    if let Some(ref t) = topic {
                        if !t.is_empty() {
                            s.push_str(&format!("topic: {t}\n"));
                        }
                    }
                    s.push_str("Sub-agent: poll agent_inbox(for_agent=sub_agent); reply with agent_subagent_done to parent.\n");
                    s.push_str("Coordinator: serial handoffs via agent_send_pipeline; parallel subagents via agent_fan_out to distinct sub_agent ids + agent_multi_inbox; sync with agent_inbox_wait / agent_topic_wait (shared topic); full thread: agent_thread(topic).\n");
                    s.push_str("Cross-host: use stable ids; prefer async polls with since_ts_ms on public profile (stricter wait caps).\n");
                    s
                }
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Sub-agent completion DM to parent with kind=subagent_done (pair with agent_subagent_task). Same gates as agent_send.",
        annotations(title = "Agent bus: subagent done", read_only_hint = false)
    )]
    async fn agent_subagent_done(
        &self,
        Parameters(args): Parameters<AgentSubagentDoneArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only (unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY or set to 0/false to allow agent_subagent_done)."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.parent_agent) {
            return Self::tool_text(
                "Error: parent_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        let sub_n = crate::bus::normalized_agent_id(&args.from_agent);
        let par_n = crate::bus::normalized_agent_id(&args.parent_agent);
        if sub_n == par_n {
            return Self::tool_text("Error: parent_agent must differ from from_agent".to_string());
        }
        if let Err(e) = self.bus_rate.try_record_send(&args.from_agent) {
            return Self::tool_text(format!("Error: {e}"));
        }
        let body_trim = args.result_body.trim();
        if body_trim.is_empty() {
            return Self::tool_text("Error: result_body must not be empty".to_string());
        }
        let cid = args
            .correlation_id
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("-");
        let mut body = String::new();
        body.push_str("SUBAGENT_DONE\n");
        body.push_str(&format!("correlation_id: {cid}\n"));
        body.push_str("---\n");
        body.push_str(body_trim);

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let parent = args.parent_agent.clone();
        let topic = args.topic.clone();

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_send(
                root,
                from,
                body,
                Some(parent.clone()),
                topic.clone(),
                Some("subagent_done".to_string()),
            ) {
                Ok(id) => {
                    format!(
                        "OK message_id={id} kind=subagent_done\nto_agent: {parent}\nParent should poll agent_inbox(for_agent=parent_agent) or agent_thread.\n"
                    )
                }
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Parallel fan-out: post the same DM body to many to_agents in one exclusive lock (fast, atomic batch). Counts as one rate-limit reservation per recipient. Serial multi-part stories use agent_send_pipeline instead.",
        annotations(title = "Agent bus: fan-out (parallel send)", read_only_hint = false)
    )]
    async fn agent_fan_out(
        &self,
        Parameters(args): Parameters<AgentFanOutArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only; unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY to allow agent_fan_out."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if args.to_agents.is_empty() {
            return Self::tool_text("Error: to_agents must not be empty".to_string());
        }
        if args.to_agents.len() > crate::bus::MAX_FAN_OUT_RECIPIENTS {
            return Self::tool_text(format!(
                "Error: at most {} recipients per fan_out",
                crate::bus::MAX_FAN_OUT_RECIPIENTS
            ));
        }
        let n = args.to_agents.len() as u32;
        if let Err(e) = self.bus_rate.try_record_n_sends(&args.from_agent, n) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let body = args.message.clone();
        let targets = args.to_agents.clone();
        let topic = args.topic.clone();
        let kind = args.kind.clone();

        let out = tokio::task::spawn_blocking(move || {
            crate::bus::agent_fan_out(root, from, body, targets, topic, kind)
        })
        .await
        .unwrap_or_else(|e| Err(format!("task failed: {e}")));

        let out = match out {
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
        description = "Serial pipeline: append multiple messages in strict order under one exclusive lock (handoffs, staged notes). Each step is one bus line; counts as N toward rate limits.",
        annotations(title = "Agent bus: send pipeline (serial)", read_only_hint = false)
    )]
    async fn agent_send_pipeline(
        &self,
        Parameters(args): Parameters<AgentSendPipelineArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only; unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY to allow agent_send_pipeline."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&args.from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if args.steps.is_empty() {
            return Self::tool_text("Error: steps must not be empty".to_string());
        }
        if args.steps.len() > crate::bus::MAX_PIPELINE_STEPS {
            return Self::tool_text(format!(
                "Error: too many steps (max {})",
                crate::bus::MAX_PIPELINE_STEPS
            ));
        }
        for (i, st) in args.steps.iter().enumerate() {
            if st.message.trim().is_empty() {
                return Self::tool_text(format!("Error: step {} has an empty message", i + 1));
            }
        }
        let n = args.steps.len() as u32;
        if let Err(e) = self.bus_rate.try_record_n_sends(&args.from_agent, n) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let root = self.project_root_for_bus();
        let from = args.from_agent.clone();
        let steps: Vec<(String, Option<String>, Option<String>, Option<String>)> = args
            .steps
            .into_iter()
            .map(|s| (s.message, s.to_agent, s.topic, s.kind))
            .collect();

        let out =
            tokio::task::spawn_blocking(move || crate::bus::agent_send_pipeline(root, from, steps))
                .await
                .unwrap_or_else(|e| Err(format!("task failed: {e}")));

        let out = match out {
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
        description = "DMs to for_agent plus broadcasts (optionally hide your own broadcasts). Poll after agent_send from others; use since_ts_ms for incremental fetch. For several agents in one read, use agent_multi_inbox.",
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
        let since = args.since_ts_ms;

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_inbox(root, for_a, limit, skip, since) {
                Ok(s) => s,
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Fan-in: one bus read, then inboxes for up to 16 for_agents (same limit/since/skip for all). Safer and faster than many agent_inbox calls when polling a team.",
        annotations(
            title = "Agent bus: multi-inbox (parallel read)",
            read_only_hint = true
        )
    )]
    async fn agent_multi_inbox(
        &self,
        Parameters(args): Parameters<AgentMultiInboxArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let agents = args.for_agents.clone();
        let limit = args.limit;
        let skip = args.skip_own_broadcasts;
        let since = args.since_ts_ms;

        let res = tokio::task::spawn_blocking(move || {
            crate::bus::agent_multi_inbox(root, agents, limit, skip, since)
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
        description = "Ordered thread slice for one topic string (same key as in agent_send). Optional since_ts_ms keeps long threads bounded.",
        annotations(title = "Agent bus: thread", read_only_hint = true)
    )]
    async fn agent_thread(
        &self,
        Parameters(args): Parameters<AgentThreadArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let topic = args.topic.clone();
        let limit = args.limit;
        let since = args.since_ts_ms;

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_thread(root, topic, limit, since) {
                Ok(s) => s,
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "Last N messages on the whole bus—shared situational awareness across agents. Optional since_ts_ms for incremental tail polling. Large logs: without since_ts_ms the server reads from the end of the file only for speed.",
        annotations(title = "Agent bus: recent", read_only_hint = true)
    )]
    async fn agent_recent(
        &self,
        Parameters(args): Parameters<AgentRecentArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let limit = args.limit;
        let since = args.since_ts_ms;

        let out = tokio::task::spawn_blocking(move || {
            match crate::bus::agent_recent(root, limit, since) {
                Ok(s) => s,
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));

        Self::tool_text(out)
    }

    #[tool(
        description = "List active thread topics on the bus with message counts and first/last timestamps—pick a topic before calling agent_thread.",
        annotations(title = "Agent bus: list topics", read_only_hint = true)
    )]
    async fn agent_list_topics(
        &self,
        Parameters(args): Parameters<AgentListTopicsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let limit = args.limit;

        let res = tokio::task::spawn_blocking(move || crate::bus::agent_list_topics(root, limit))
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
        description = "Single read: top senders, hot topics, and a recent tail (compact). Combines roster/topic/recent intent in one round-trip for orchestrators.",
        annotations(title = "Agent bus: coordination snapshot", read_only_hint = true)
    )]
    async fn agent_coordination_snapshot(
        &self,
        Parameters(args): Parameters<AgentCoordinationSnapshotArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let topics_limit = args.topics_limit;
        let recent_limit = args.recent_limit;

        let res = tokio::task::spawn_blocking(move || {
            crate::bus::agent_coordination_snapshot(root, topics_limit, recent_limit)
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
        description = "Summarize who posts on the bus and who receives DMs (counts, last activity, kind mix)—useful when joining a multi-agent session.",
        annotations(title = "Agent bus: roster", read_only_hint = true)
    )]
    async fn agent_roster(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();

        let res = tokio::task::spawn_blocking(move || crate::bus::agent_roster(root))
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
        description = "Fetch a single bus message by exact id (from agent_send or listings) for deep links and follow-ups.",
        annotations(title = "Agent bus: get message", read_only_hint = true)
    )]
    async fn agent_get_message(
        &self,
        Parameters(args): Parameters<AgentGetMessageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let mid = args.message_id.clone();

        let res = tokio::task::spawn_blocking(move || crate::bus::agent_get_message(root, mid))
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
        description = "Deterministic plan skeleton (objective, constraints, steps, risks). No model calls—fill in blanks for structured reasoning.",
        annotations(title = "Thinking: plan skeleton", read_only_hint = true)
    )]
    async fn thinking_plan_skeleton(
        &self,
        Parameters(args): Parameters<ThinkingPlanArgs>,
    ) -> Result<CallToolResult, McpError> {
        let objective = args.objective.clone();
        let constraints = args.constraints.clone();
        let known_steps = args.known_steps.clone();
        let max_chars = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            thinking_plan_skeleton_work(objective, constraints, known_steps, max_chars)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic “step back” checklist: smallest useful outcome, assumptions, falsifiers, parallel vs serial decomposition, which MCP tools give ground truth.",
        annotations(title = "Thinking: step back", read_only_hint = true)
    )]
    async fn thinking_step_back(
        &self,
        Parameters(args): Parameters<ThinkingStepBackArgs>,
    ) -> Result<CallToolResult, McpError> {
        let problem = args.problem.clone();
        let max_chars = args.max_chars;
        let out = tokio::task::spawn_blocking(move || thinking_step_back_work(problem, max_chars))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Turn free-form claims into an audit list (evidence needed, plan B). Deterministic; no external APIs.",
        annotations(title = "Thinking: assumption audit", read_only_hint = true)
    )]
    async fn thinking_assumption_audit(
        &self,
        Parameters(args): Parameters<ThinkingAssumptionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let claims = args.claims.clone();
        let max_chars = args.max_chars;
        let out =
            tokio::task::spawn_blocking(move || thinking_assumption_audit_work(claims, max_chars))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Append one timestamped line to .mint-mcp/agent-thinking.log (size-rotated). Blocked when MINT_NETWORK_PROFILE=public.",
        annotations(title = "Thinking: session append", read_only_hint = false)
    )]
    async fn thinking_session_append(
        &self,
        Parameters(args): Parameters<ThinkingSessionAppendArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let note = args.note.clone();
        let out = tokio::task::spawn_blocking(move || thinking_session_append_work(root, note))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Read the last N lines from .mint-mcp/agent-thinking.log. Blocked when MINT_NETWORK_PROFILE=public to avoid leaking session notes on internet-facing deployments.",
        annotations(title = "Thinking: session read", read_only_hint = true)
    )]
    async fn thinking_session_read(
        &self,
        Parameters(args): Parameters<ThinkingSessionReadArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let tail = args.tail_lines;
        let out = tokio::task::spawn_blocking(move || thinking_session_read_work(root, tail))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic checklist template for milestones ([ ] items). Pair with plan_work_buckets and agent_subagent_task.",
        annotations(title = "Plan execution checklist", read_only_hint = true)
    )]
    async fn plan_execution_checklist(
        &self,
        Parameters(args): Parameters<PlanExecutionChecklistArgs>,
    ) -> Result<CallToolResult, McpError> {
        let m = args.milestones.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_execution_checklist_work(m, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Structured handoff brief (scope, deliverables, acceptance, blockers) to paste into agent_subagent_task.task_body or human tickets.",
        annotations(title = "Plan handoff brief", read_only_hint = true)
    )]
    async fn plan_handoff_brief(
        &self,
        Parameters(args): Parameters<PlanHandoffBriefArgs>,
    ) -> Result<CallToolResult, McpError> {
        let title = args.title.clone();
        let scope = args.scope.clone();
        let del = args.deliverables.clone();
        let acc = args.acceptance.clone();
        let blk = args.blockers.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || {
            plan_handoff_brief_work(title, scope, del, acc, blk, mc)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Round-robin split of work items into N buckets for parallel subagents or agent_fan_out planning.",
        annotations(title = "Plan work buckets", read_only_hint = true)
    )]
    async fn plan_work_buckets(
        &self,
        Parameters(args): Parameters<PlanWorkBucketsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let items = args.items.clone();
        let b = args.buckets;
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_work_buckets_work(items, b, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Risk register template (likelihood, impact, mitigation, owner) for security or release reviews—deterministic scaffold, no model.",
        annotations(title = "Plan risk register", read_only_hint = true)
    )]
    async fn plan_risk_register(
        &self,
        Parameters(args): Parameters<PlanRiskRegisterArgs>,
    ) -> Result<CallToolResult, McpError> {
        let risks = args.risks.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_risk_register_work(risks, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Checklist template for acceptance criteria and sign-off (deterministic). Pair with plan_execution_checklist.",
        annotations(title = "Plan definition of done", read_only_hint = true)
    )]
    async fn plan_definition_of_done(
        &self,
        Parameters(args): Parameters<PlanDefinitionOfDoneArgs>,
    ) -> Result<CallToolResult, McpError> {
        let outcomes = args.outcomes.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_definition_of_done_work(outcomes, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic test-strategy scaffold (unit/integration/smoke, fixtures, how to run project_cargo_test / project_pytest). Does not execute tests.",
        annotations(title = "Plan test strategy", read_only_hint = true)
    )]
    async fn plan_test_strategy(
        &self,
        Parameters(args): Parameters<PlanTestStrategyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let scope = args.scope.clone();
        let rh = args.risk_hotspots.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_test_strategy_work(scope, rh, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Short retrospective scaffold (went well / not well / actions) after a milestone or agent batch—deterministic template.",
        annotations(title = "Plan retrospective", read_only_hint = true)
    )]
    async fn plan_retrospective(
        &self,
        Parameters(args): Parameters<PlanRetrospectiveArgs>,
    ) -> Result<CallToolResult, McpError> {
        let it = args.iteration.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || plan_retrospective_work(it, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Pre-mortem prompts: assume failure and list causes, warnings, and guardrails (you fill content; no model).",
        annotations(title = "Reasoning premortem", read_only_hint = true)
    )]
    async fn reasoning_premortem(
        &self,
        Parameters(args): Parameters<ReasoningPremortemArgs>,
    ) -> Result<CallToolResult, McpError> {
        let project = args.project.clone();
        let timeframe = args.timeframe.clone();
        let mc = args.max_chars;
        let out =
            tokio::task::spawn_blocking(move || reasoning_premortem_work(project, timeframe, mc))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Five-whys ladder template (you answer each why; stops at root-cause hypothesis). No automated inference.",
        annotations(title = "Reasoning five whys", read_only_hint = true)
    )]
    async fn reasoning_five_whys(
        &self,
        Parameters(args): Parameters<ReasoningFiveWhysArgs>,
    ) -> Result<CallToolResult, McpError> {
        let seed = args.seed_problem.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || reasoning_five_whys_work(seed, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Deterministic premise→steps→open-question chain (you provide content; no automated inference).",
        annotations(title = "Reasoning chain scaffold", read_only_hint = true)
    )]
    async fn reasoning_chain(
        &self,
        Parameters(args): Parameters<ReasoningChainArgs>,
    ) -> Result<CallToolResult, McpError> {
        let pr = args.premises.clone();
        let st = args.steps.clone();
        let oq = args.open_question.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || reasoning_chain_work(pr, st, oq, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Checklist to stress-test a claim (alternatives, falsifiers, bias). Not a separate adversarial model.",
        annotations(title = "Reasoning devils advocate", read_only_hint = true)
    )]
    async fn reasoning_devils_advocate(
        &self,
        Parameters(args): Parameters<ReasoningDevilsAdvocateArgs>,
    ) -> Result<CallToolResult, McpError> {
        let c = args.claim.clone();
        let ctx = args.context.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || reasoning_devils_advocate_work(c, ctx, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Markdown-style table scaffold: claim | evidence_needed | source_hint for grounding work.",
        annotations(title = "Reasoning evidence matrix", read_only_hint = true)
    )]
    async fn reasoning_evidence_matrix(
        &self,
        Parameters(args): Parameters<ReasoningEvidenceMatrixArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cl = args.claims.clone();
        let mc = args.max_chars;
        let out = tokio::task::spawn_blocking(move || reasoning_evidence_matrix_work(cl, mc))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Cheap synchronous health check: server clock, project root, bus file presence, read-only and rate-limit hints. Use before coordinating with other agents.",
        annotations(title = "Agent bus: ping", read_only_hint = true)
    )]
    async fn agent_ping(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let ts = Self::now_ms();
        let bus_present = root.join(".mint-mcp").join("agent-bus.jsonl").exists();
        let max_send = std::env::var("PB_BUS_MAX_SENDS_PER_WINDOW")
            .or_else(|_| std::env::var("MINT_BUS_MAX_SENDS_PER_WINDOW"))
            .unwrap_or_else(|_| "120".to_string());
        let win_ms = std::env::var("PB_BUS_RATE_WINDOW_MS")
            .or_else(|_| std::env::var("MINT_BUS_RATE_WINDOW_MS"))
            .unwrap_or_else(|_| "60000".to_string());
        let term_on = crate::ops::terminal_enabled();
        let pt = crate::ops::project_test_tools_enabled();
        let gitw = crate::ops::git_write_enabled();
        let push_on = crate::ops::git_push_enabled();
        let remote_on = crate::ops::git_remote_enabled();
        let net = self.network_profile.label();
        let sleep_cap = self.network_profile.server_sleep_cap_ms();
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let family = std::env::consts::FAMILY;

        let mut s = String::new();
        s.push_str("AGENT_PING\n");
        s.push_str(&format!("server_ts_ms: {ts}\n"));
        s.push_str(&format!("platform: os={os} arch={arch} family={family}\n"));
        s.push_str(&format!("project_root: {}\n", root.display()));
        s.push_str(&format!(
            "PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE: {net} (server_sleep_cap_ms={sleep_cap})\n"
        ));
        s.push_str(&format!("bus_file_present: {bus_present}\n"));
        s.push_str(&format!("bus_read_only: {}\n", self.bus_read_only));
        s.push_str(&format!(
        "rate_limit: PB_BUS_MAX_SENDS_PER_WINDOW / MINT_BUS_MAX_SENDS_PER_WINDOW={max_send} (0=off), PB_BUS_RATE_WINDOW_MS / MINT_BUS_RATE_WINDOW_MS={win_ms}\n"
        ));
        s.push_str(&format!(
        "gates: PB_TERMINAL_ENABLE / MINT_TERMINAL_ENABLE active={term_on} (terminal_run, terminal_run_stdin, terminal_which); PB_PROJECT_TEST_ENABLE / MINT_PROJECT_TEST_ENABLE active={pt} (project_cargo_test, project_cargo_check, project_pytest); PB_GIT_WRITE_ENABLE / MINT_GIT_WRITE_ENABLE active={gitw}; PB_GIT_PUSH_ENABLE / MINT_GIT_PUSH_ENABLE active={push_on}; PB_GIT_REMOTE_ENABLE / MINT_GIT_REMOTE_ENABLE active={remote_on} (git_fetch, git_pull)\n"
        ));
        s.push_str("thinking_session_append/read touch .mint-mcp/agent-thinking.log (same trusted-profile gate as other local writes).\n");
        s.push_str("\nAsync: agent_send + agent_inbox / agent_recent with since_ts_ms (low latency per call).\n");
        s.push_str("Subagent flow: agent_subagent_task -> agent_subagent_done (DM parent).\n");
        s.push_str(
            "Parallel batch: agent_fan_out, agent_multi_inbox, agent_coordination_snapshot.\n",
        );
        s.push_str("Serial batch: agent_send_pipeline (ordered posts, one lock).\n");
        s.push_str("Sync wait: agent_inbox_wait, agent_topic_wait, ask_agents.\n");
        s.push_str("Diagnostics: mcp_env_hints (PB_* / MINT_* gates; root paths redacted).\n");
        s.push_str("Security: paths under project root; PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY; rate limits; PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE=public blocks bus writes, terminal/PR CLI, risky git, manifest, code_index_build, convert file output, and tightens waits.\n");
        s.push_str("Default PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW includes gh,glab for optional PR/MR tools when PB_TERMINAL_ENABLE / MINT_TERMINAL_ENABLE=1.\n");
        Self::tool_text(s)
    }

    #[tool(
        description = "Blocking wait (single MCP round-trip): polls agent_inbox until there is matching traffic or timeout. Prefer agent_inbox+since_ts_ms when you can loop client-side.",
        annotations(title = "Agent bus: inbox wait (sync)", read_only_hint = true)
    )]
    async fn agent_inbox_wait(
        &self,
        Parameters(args): Parameters<AgentInboxWaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let for_a = args.for_agent.clone();
        let limit = args.limit;
        let skip = args.skip_own_broadcasts;
        let since = args.since_ts_ms;
        let cap = self.sync_wait_timeout_cap_ms();
        let timeout_ms = args.timeout_ms.clamp(100, cap);
        let poll_ms = args.poll_interval_ms.clamp(50, 2_000);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        while tokio::time::Instant::now() < deadline {
            let root_c = root.clone();
            let for_c = for_a.clone();
            let has = tokio::task::spawn_blocking(move || {
                crate::bus::agent_inbox_non_empty(root_c, for_c, limit, skip, since)
            })
            .await
            .unwrap_or_else(|e| Err(format!("task failed: {e}")));

            match has {
                Ok(true) => {
                    let root_c = root.clone();
                    let for_c = for_a.clone();
                    let inbox = tokio::task::spawn_blocking(move || {
                        crate::bus::agent_inbox(root_c, for_c, limit, skip, since)
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("task failed: {e}")));
                    let body = match inbox {
                        Ok(t) => t,
                        Err(e) => format!("Error: {e}"),
                    };
                    return Self::tool_text(format!(
                        "AGENT_INBOX_WAIT\nstatus: ready\ntimeout_ms_cap: {timeout_ms}\n\n{body}"
                    ));
                }
                Ok(false) => {}
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            }

            tokio::time::sleep(std::time::Duration::from_millis(poll_ms)).await;
        }

        Self::tool_text(format!(
            "AGENT_INBOX_WAIT\nstatus: timeout\nNo matching inbox traffic within {timeout_ms}ms. Retry with agent_inbox or a longer timeout_ms."
        ))
    }

    #[tool(
        description = "Blocking wait until another agent posts on the same topic (you are exclude_from_agent), or timeout. Then returns agent_thread slice from since_ts_ms.",
        annotations(title = "Agent bus: topic wait (sync)", read_only_hint = true)
    )]
    async fn agent_topic_wait(
        &self,
        Parameters(args): Parameters<AgentTopicWaitArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let topic = args.topic.clone();
        let exclude = args.exclude_from_agent.clone();
        let since = args.since_ts_ms.unwrap_or_else(Self::now_ms);
        let cap = self.sync_wait_timeout_cap_ms();
        let timeout_ms = args.timeout_ms.clamp(100, cap);
        let poll_ms = args.poll_interval_ms.clamp(50, 2_000);
        let topic_display = args.topic.clone();

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

        while tokio::time::Instant::now() < deadline {
            let root_c = root.clone();
            let topic_c = topic.clone();
            let exclude_c = exclude.clone();
            let has = tokio::task::spawn_blocking(move || {
                crate::bus::agent_topic_has_reply_from_others(root_c, topic_c, since, exclude_c)
            })
            .await
            .unwrap_or_else(|e| Err(format!("task failed: {e}")));

            match has {
                Ok(true) => {
                    let root_c = root.clone();
                    let topic_c = topic.clone();
                    let thread = tokio::task::spawn_blocking(move || {
                        crate::bus::agent_thread(root_c, topic_c, 80, Some(since))
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("task failed: {e}")));
                    let body = match thread {
                        Ok(t) => t,
                        Err(e) => format!("Error: {e}"),
                    };
                    return Self::tool_text(format!(
                        "AGENT_TOPIC_WAIT\nstatus: ready\ntopic: {topic_display}\nsince_ts_ms: {since}\ntimeout_ms_cap: {timeout_ms}\n\n{body}"
                    ));
                }
                Ok(false) => {}
                Err(e) => return Self::tool_text(format!("Error: {e}")),
            }

            tokio::time::sleep(std::time::Duration::from_millis(poll_ms)).await;
        }

        Self::tool_text(format!(
            "AGENT_TOPIC_WAIT\nstatus: timeout\ntopic: {topic_display}\nNo reply from other agents within {timeout_ms}ms."
        ))
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

    #[tool(
        description = "Broadcast a question to the durable multi-agent bus and collect replies posted to the same topic within a short time window. Other agents should respond via `agent_send` with the same topic.",
        annotations(title = "Ask agents (bus orchestrator)", read_only_hint = false)
    )]
    async fn ask_agents(
        &self,
        Parameters(args): Parameters<AskAgentsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.project_root_for_bus();
        let from_agent = args.from_agent.clone();
        let query = args.query.clone();
        if query.trim().is_empty() {
            return Self::tool_text("Error: query must not be empty".to_string());
        }

        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        if self.bus_read_only {
            return Self::tool_text(
                "Error: bus is read-only; ask_agents posts to the bus. Unset PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY."
                    .to_string(),
            );
        }
        if !crate::bus::from_agent_acceptable(&from_agent) {
            return Self::tool_text(
                "Error: from_agent must be a non-empty identifier (letters, digits, _.-)"
                    .to_string(),
            );
        }
        if let Err(e) = self.bus_rate.try_record_send(&from_agent) {
            return Self::tool_text(format!("Error: {e}"));
        }

        let topic = args
            .topic
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(Self::new_topic);
        let ask_cap = self.ask_agents_timeout_cap_ms();
        let timeout_ms = args.timeout_ms.clamp(250, ask_cap);
        let limit = args.limit.clamp(1, 50);

        let since = Self::now_ms();
        let send_res = tokio::task::spawn_blocking({
            let root = root.clone();
            let from_agent = from_agent.clone();
            let topic = topic.clone();
            let query = query.clone();
            move || {
                crate::bus::agent_send(
                    root,
                    from_agent,
                    query,
                    None,
                    Some(topic),
                    Some("question".to_string()),
                )
            }
        })
        .await
        .unwrap_or_else(|e| Err(format!("task failed: {e}")));

        let msg_id = match send_res {
            Ok(id) => id,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut replies: Vec<crate::bus::AgentMessage> = Vec::new();

        while tokio::time::Instant::now() < deadline && replies.len() < limit {
            let poll = tokio::task::spawn_blocking({
                let root = root.clone();
                let topic = topic.clone();
                let from_agent = from_agent.clone();
                move || crate::bus::agent_collect_topic_replies(root, topic, since, from_agent, 200)
            })
            .await
            .unwrap_or_else(|e| Err(format!("task failed: {e}")));

            match poll {
                Ok(msgs) => {
                    for m in msgs {
                        if replies.len() >= limit {
                            break;
                        }
                        if seen.insert(m.id.clone()) {
                            replies.push(m);
                        }
                    }
                }
                Err(_e) => {
                    // If polling fails transiently (e.g. file contention), keep waiting.
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(220)).await;
        }

        let mut out = String::new();
        out.push_str("ASK_AGENTS\n");
        out.push_str(&format!(
            "topic: {topic}\nmessage_id: {msg_id}\nfrom_agent: {from_agent}\n"
        ));
        out.push_str(&format!(
            "timeout_ms: {timeout_ms}\nreplies: {}\n\n",
            replies.len()
        ));
        out.push_str("question:\n");
        out.push_str(&query);
        out.push_str("\n\n---\n\n");

        if replies.is_empty() {
            out.push_str("No replies yet. Other agents should call `agent_send` with the same topic, then you can re-run `ask_agents` with that topic or use `agent_thread(topic)`.\n");
        } else {
            for r in replies {
                out.push_str(&format!(
                    "from: {}\nkind: {}\n---\n{}\n\n====\n\n",
                    r.from_agent, r.kind, r.body
                ));
            }
        }

        Self::tool_text(out)
    }

    // ---- Consensus tools ----

    #[tool(
        description = "Create a new consensus proposal. Other agents can then vote on it using consensus_vote. Returns the proposal ID and details.",
        annotations(title = "Consensus: propose", read_only_hint = false)
    )]
    async fn consensus_propose(
        &self,
        Parameters(args): Parameters<ConsensusProposeArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let options: Vec<String> = args
            .options
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if options.len() < 2 {
            return Self::tool_text(
                "Error: options must contain at least 2 comma-separated choices.".to_string(),
            );
        }
        let strategy = match args.strategy.to_lowercase().as_str() {
            "simplemajority" | "simple_majority" | "simple-majority" => crate::agent::consensus::ConsensusStrategy::SimpleMajority,
            "supermajority" | "super_majority" | "super-majority" => crate::agent::consensus::ConsensusStrategy::SuperMajority,
            "unanimous" => crate::agent::consensus::ConsensusStrategy::Unanimous,
            "weightedvoting" | "weighted_voting" | "weighted-voting" | "weighted" => crate::agent::consensus::ConsensusStrategy::WeightedVoting,
            "rankedchoice" | "ranked_choice" | "ranked-choice" | "ranked" => crate::agent::consensus::ConsensusStrategy::RankedChoice,
            "delegation" => crate::agent::consensus::ConsensusStrategy::Delegation,
            _ => return Self::tool_text(format!("Error: unknown strategy '{}'. Valid: SimpleMajority, SuperMajority, Unanimous, WeightedVoting, RankedChoice, Delegation.", args.strategy)),
        };
        let engine = self.consensus_engine.clone();
        let from = args.from_agent.clone();
        let topic = args.topic.clone();
        let desc = args.description.clone();
        let deadline = args.deadline_ms;
        let out = tokio::task::spawn_blocking(move || {
            let mut eng = engine.lock().unwrap();
            let p = eng.create_proposal(from, topic, desc, options, deadline, strategy);
            format!(
                "CONSENSUS_PROPOSE\nid: {}\nproposer: {}\ntopic: {}\nstrategy: {:?}\ndeadline_ms: {}\nstatus: {:?}\noptions: {}\ncreated_at: {}",
                p.id, p.proposer, p.topic, p.strategy, p.deadline_ms, p.status,
                p.options.join(", "), p.created_at
            )
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Cast a vote on an open consensus proposal. Each agent may vote only once per proposal.",
        annotations(title = "Consensus: vote", read_only_hint = false)
    )]
    async fn consensus_vote(
        &self,
        Parameters(args): Parameters<ConsensusVoteArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let engine = self.consensus_engine.clone();
        let pid = args.proposal_id.clone();
        let voter = args.voter.clone();
        let choice = args.choice.clone();
        let conf = args.confidence.clamp(0.0, 1.0);
        let rationale = args.rationale.clone();
        let out = tokio::task::spawn_blocking(move || {
            let mut eng = engine.lock().unwrap();
            match eng.cast_vote(&pid, voter.clone(), choice.clone(), conf, rationale) {
                Ok(()) => format!("CONSENSUS_VOTE\nproposal: {}\nvoter: {}\nchoice: {}\nconfidence: {:.2}\nstatus: recorded", pid, voter, choice, conf),
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Tally the current votes on a proposal without closing it. Shows per-option counts, participation rate, and whether consensus has been reached.",
        annotations(title = "Consensus: tally", read_only_hint = true)
    )]
    async fn consensus_tally(
        &self,
        Parameters(args): Parameters<ConsensusTallyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let engine = self.consensus_engine.clone();
        let pid = args.proposal_id.clone();
        let out = tokio::task::spawn_blocking(move || {
            let eng = engine.lock().unwrap();
            match eng.tally(&pid) {
                Ok(r) => {
                    let mut tally_lines: Vec<String> = r.tally.iter().map(|(k, v)| format!("  {k}: {v}")).collect();
                    tally_lines.sort();
                    format!(
                        "CONSENSUS_TALLY\nproposal: {}\nwinning_option: {}\nconsensus_reached: {}\nparticipation_rate: {:.1}%\ntally:\n{}\ndeciding_voters: {}",
                        pid,
                        r.winning_option.as_deref().unwrap_or("(none)"),
                        r.consensus_reached,
                        r.participation_rate * 100.0,
                        tally_lines.join("\n"),
                        r.deciding_voters.join(", ")
                    )
                }
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List all open proposals that are still accepting votes.",
        annotations(title = "Consensus: list open proposals", read_only_hint = true)
    )]
    async fn consensus_list(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let engine = self.consensus_engine.clone();
        let out = tokio::task::spawn_blocking(move || {
            let eng = engine.lock().unwrap();
            let open = eng.list_open();
            if open.is_empty() {
                return "CONSENSUS_LIST\nNo open proposals.".to_string();
            }
            let mut s = format!("CONSENSUS_LIST\nOpen proposals: {}\n\n", open.len());
            for p in open {
                s.push_str(&format!(
                    "id: {} | proposer: {} | topic: {} | strategy: {:?} | options: {} | votes: {} | deadline_ms: {}\n",
                    p.id, p.proposer, p.topic, p.strategy, p.options.join(", "), p.votes.len(), p.deadline_ms
                ));
            }
            s
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Close a proposal, tally the final results, and persist the outcome. No further votes will be accepted.",
        annotations(title = "Consensus: close proposal", read_only_hint = false)
    )]
    async fn consensus_close(
        &self,
        Parameters(args): Parameters<ConsensusCloseArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let engine = self.consensus_engine.clone();
        let pid = args.proposal_id.clone();
        let out = tokio::task::spawn_blocking(move || {
            let mut eng = engine.lock().unwrap();
            match eng.close_proposal(&pid) {
                Ok(r) => {
                    let mut tally_lines: Vec<String> = r.tally.iter().map(|(k, v)| format!("  {k}: {v}")).collect();
                    tally_lines.sort();
                    format!(
                        "CONSENSUS_CLOSE\nproposal: {}\nwinning_option: {}\nconsensus_reached: {}\nparticipation_rate: {:.1}%\ntally:\n{}\ndeciding_voters: {}\nstatus: Closed",
                        pid,
                        r.winning_option.as_deref().unwrap_or("(none)"),
                        r.consensus_reached,
                        r.participation_rate * 100.0,
                        tally_lines.join("\n"),
                        r.deciding_voters.join(", ")
                    )
                }
                Err(e) => format!("Error: {e}"),
            }
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    // ---- Review tools ----

    #[tool(
        description = "Analyze diff for risk patterns (large diffs, risky file types, complex changes).",
        annotations(title = "Review: diff risk", read_only_hint = true)
    )]
    async fn review_diff_risk(
        &self,
        Parameters(args): Parameters<ReviewDiffRiskArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let staged = args.staged;
        let revision_range = args.revision_range.clone();
        let out: String = tokio::task::spawn_blocking(move || {
            review_diff_risk_work(root, staged, revision_range)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Focused review of diff changes for a specific concern (security, performance, correctness, maintainability, or style).",
        annotations(title = "Review: focused review", read_only_hint = true)
    )]
    async fn review_focused_review(
        &self,
        Parameters(args): Parameters<ReviewFocusedReviewArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let focus = args.focus.clone();
        let staged = args.staged;
        let revision_range = args.revision_range.clone();
        let out: String = tokio::task::spawn_blocking(move || {
            review_focused_review_work(root, focus, staged, revision_range)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Scan diff for functions or modules that changed but lack corresponding test changes.",
        annotations(title = "Review: missing tests", read_only_hint = true)
    )]
    async fn review_missing_tests(
        &self,
        Parameters(args): Parameters<ReviewMissingTestsArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let staged = args.staged;
        let revision_range = args.revision_range.clone();
        let out: String = tokio::task::spawn_blocking(move || {
            review_missing_tests_work(root, staged, revision_range)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Generate a PR packet (summary, changed files, risk notes) from a revision range.",
        annotations(title = "Review: PR packet", read_only_hint = true)
    )]
    async fn review_pr_packet(
        &self,
        Parameters(args): Parameters<ReviewPrPacketArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let revision_range = args.revision_range.clone();
        let out: String =
            tokio::task::spawn_blocking(move || review_pr_packet_work(root, revision_range))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Check diff against a custom rulebook file for policy or convention violations.",
        annotations(title = "Review: rulebook check", read_only_hint = true)
    )]
    async fn review_rulebook_check(
        &self,
        Parameters(args): Parameters<ReviewRulebookCheckArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let rulebook_path = match self.resolve_path(&args.rulebook_path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let staged = args.staged;
        let revision_range = args.revision_range.clone();
        let out: String = tokio::task::spawn_blocking(move || {
            review_rulebook_check_work(root, rulebook_path, staged, revision_range)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    // ---- Debug tools ----

    #[tool(
        description = "Execute a command under a trace, recording invocation and output for later replay.",
        annotations(title = "Debug: trace run", read_only_hint = false)
    )]
    async fn debug_trace_run(
        &self,
        Parameters(args): Parameters<DebugTraceRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self.root.clone();
        let trace_id = args.trace_id.clone();
        let command = args.command.clone();
        let trace_args = args.args.clone();
        let out: String = tokio::task::spawn_blocking(move || {
            debug_trace_run_work(root, trace_id, command, trace_args)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Replay a previously traced run; use dry_run=true to preview without execution.",
        annotations(title = "Debug: replay run", read_only_hint = true)
    )]
    async fn debug_replay_run(
        &self,
        Parameters(args): Parameters<DebugReplayRunArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let trace_id = args.trace_id.clone();
        let dry_run = args.dry_run;
        let out: String =
            tokio::task::spawn_blocking(move || debug_replay_run_work(root, trace_id, dry_run))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Cluster past trace failures by similarity to find recurring error patterns.",
        annotations(title = "Debug: failure cluster", read_only_hint = true)
    )]
    async fn debug_failure_cluster(
        &self,
        Parameters(args): Parameters<DebugFailureClusterArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let trace_dir = args.trace_dir.clone();
        let min_cluster_size = args.min_cluster_size;
        let out: String = tokio::task::spawn_blocking(move || {
            debug_failure_cluster_work(root, trace_dir, min_cluster_size)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Generate a root-cause analysis report from a specific trace.",
        annotations(title = "Debug: root cause report", read_only_hint = true)
    )]
    async fn debug_root_cause_report(
        &self,
        Parameters(args): Parameters<DebugRootCauseReportArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let trace_id = args.trace_id.clone();
        let out: String =
            tokio::task::spawn_blocking(move || debug_root_cause_report_work(root, trace_id))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    // ---- Code intelligence tools ----

    #[tool(
        description = "Dump a lightweight AST outline for a source file (language-agnostic heuristic parser).",
        annotations(title = "Code: AST dump", read_only_hint = true)
    )]
    async fn code_ast_dump(
        &self,
        Parameters(args): Parameters<CodeAstDumpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let language = args.language.clone();
        let out: String = tokio::task::spawn_blocking(move || code_ast_dump_work(path, language))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Extract import/include/use statements from a source file.",
        annotations(title = "Code: extract imports", read_only_hint = true)
    )]
    async fn code_extract_imports(
        &self,
        Parameters(args): Parameters<CodeExtractImportsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out: String = tokio::task::spawn_blocking(move || code_extract_imports_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Detect the programming language of a file (by extension and optional content heuristics).",
        annotations(title = "Code: detect language", read_only_hint = true)
    )]
    async fn code_detect_language(
        &self,
        Parameters(args): Parameters<CodeDetectLanguageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let content = args.content.clone();
        let out: String =
            tokio::task::spawn_blocking(move || code_detect_language_work(path, content))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Compute a cyclomatic-style complexity score for a source file.",
        annotations(title = "Code: complexity score", read_only_hint = true)
    )]
    async fn code_complexity_score(
        &self,
        Parameters(args): Parameters<CodeComplexityScoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let content = args.content.clone();
        let out: String =
            tokio::task::spawn_blocking(move || code_complexity_score_work(path, content))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Scan a file or directory for likely dead code (unreachable, unused exports).",
        annotations(title = "Code: dead code hints", read_only_hint = true)
    )]
    async fn code_dead_code_hints(
        &self,
        Parameters(args): Parameters<CodeDeadCodeHintsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out: String =
            tokio::task::spawn_blocking(move || code_dead_code_hints_work(root, path))
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
                    "Project-scoped file/git tools plus a durable JSONL bus under .mint-mcp/ for multi-agent coordination: async and sync waits, serial pipelines, parallel fan-out/fan-in, optional rate limits and read-only mode. Strong schemas and MCP is_error on failures.",
                ),
        )
        .with_instructions(format!(
            "Workspace root: {root_hint} (env: PB_PROJECT_ROOT / MINT_PROJECT_ROOT or PROJECT_ROOT; all paths must stay inside this directory).\n\
             \n\
             PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE=local (default) vs public|internet|wan: on **public**, bus writes (agent_send, agent_handoff, fan_out, pipeline, ask_agents, subagent_task, subagent_done), terminal_run, terminal_run_stdin, gh_pr_*, glab_mr_*, git_checkout/stash/push/fetch/pull/add/commit/branch-create/worktree-add, gk_git_* write tools, codebase_manifest_write, code_index_build, thinking_session_append/read, workspace_temp_*, screenshot_capture, ocr_tesseract, ollama_list_models, and convert_format output_path are **blocked**; server_sleep, wait_until_timestamp, wait_for_path, agent_inbox_wait, agent_topic_wait, and ask_agents use **lower time caps**.\n\
             \n\
             **GitKraken/GitLens cloud tools** (pull_request_*, issues_*, gitkraken_workspace_*, gitlens_launchpad/start_work/review, hosted APIs) are listed in `gitkraken_cloud_catalog` and require the official GitLens MCP + provider auth. This server implements **local-repo** git tools, **gk_*** aliases matching GitKraken parameter shapes, and optional **gh** / **glab** PR/MR helpers (local CLI, not hosted HTTP) when those binaries are allowlisted.\n\
             \n\
             Suggested workflow for coding agents:\n\
             1) workspace_context — root, git, bus, limits, network_profile, gates.\n\
             2) project_tree, codebase_index, codebase_word_index, codebase_shallow_hash, codebase_path_list, codebase_largest_files, codebase_duplicate_basenames, codebase_cargo_dependency_names, codebase_manifest_write (local only), code_index_build/search/status/file_symbols/aggregate_stats/paths_prefix/path_substring/symbol_prefix/duplicate_symbols/paths_without_symbols/extension_histogram/symbol_frequency, find_files, grep_code.\n\
             3) read_file / read_file_tail; json_validate_file; mcp_env_hints; agent_coordination_playbook (bus patterns); ai_inference_integration_guide + ai_stack_probe + coding_agent_toolchains_catalog (optional CLIs / agent ecosystem map); gitkraken_cloud_catalog + gitkraken_hosted_tool_redirect; convert_format (+ yaml↔json, toml↔json, *_top_level_keys, sha256_hex, wrap_lines, remove_empty_lines, dedupe_adjacent_lines, slugify, c_string_escape, html_escape, …); compress_gzip_base64 / decompress_gzip_base64 (binary-safe blobs).\n\
             4) text_stats; summarize_token_estimate; compress_* (+ ws_collapse, simplify_whitespace, markdown_links, unified_diff_stats, truncate_lines, strip_tags, fenced_code_blocks, redact_urls/emails, markdown_frontmatter, double_slash_line_comments, drop_lines_matching_regex, dedent, lines head/tail, collapse_blank, truncate_middle); summarize_* (+ top_terms, identifier_frequency, duplicate_lines, line_histogram, line_prefix_groups, rank_lines_by_keywords, bullets, headings, markdown_toc, json_structure, delimited_header, diff_file_list, first_lines); text_levenshtein_distance; text_token_jaccard; text_longest_common_substring; text_djb2_hash; format_prefix_lines; pack_context (segment_labels).\n\
             5) git_* and gk_git_* (GitKraken names); gh_pr_list/view + glab_mr_list/show when gh/glab are in PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW; git_push needs PB_GIT_PUSH_ENABLE / MINT_GIT_PUSH_ENABLE=1; git_fetch/git_pull need PB_GIT_REMOTE_ENABLE / MINT_GIT_REMOTE_ENABLE=1; trusted profile for remotes/terminal/PR CLI.\n\
             6) terminal_run; terminal_run_stdin (stdin from text or project file, LAN); project_cargo_test, project_cargo_check, project_pytest when PB_PROJECT_TEST_ENABLE / MINT_PROJECT_TEST_ENABLE=1 + cargo/pytest in PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW (bounded argv; LAN/trusted); terminal_which; terminal_program_version; ocr_tesseract + ollama_list_models on trusted hosts with tesseract/ollama in PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW; wait_for_path (optional poll_jitter_ms_max); server_sleep / wait_until_timestamp (optional reason on sleep, jitter).\n\
             7) thinking_plan_skeleton, thinking_step_back, thinking_assumption_audit (deterministic); thinking_session_append/read on trusted hosts only.\n\
             8) plan_execution_checklist, plan_handoff_brief (paste into subagent bodies), plan_work_buckets (parallel split), plan_risk_register, plan_definition_of_done, plan_test_strategy, plan_retrospective (test pyramid + retro scaffolds; pair with project_cargo_test / project_pytest).\n\
             9) reasoning_chain, reasoning_devils_advocate, reasoning_evidence_matrix, reasoning_premortem, reasoning_five_whys (structured argumentation / pre-mortem / root-cause ladder).\n\
             10) workspace_temp_* for scratch under .mint-mcp/agent-temp; screenshot_capture → .mint-mcp/screenshots/*.png then ocr_tesseract (trusted/LAN).\n\
             \n\
             Multi-agent: stable from_agent; agent_handoff + agent_subagent_task + agent_subagent_done; mcp_env_hints for gate vars; agent_coordination_playbook for serial/parallel/async patterns; async polls with since_ts_ms; sync: agent_inbox_wait, agent_topic_wait, ask_agents (trusted profile). Parallel: agent_fan_out, agent_multi_inbox, agent_coordination_snapshot. Serial: agent_send_pipeline.\n\
             Security: PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY; rate limits; paths under root. Bus: .mint-mcp/agent-bus.jsonl.\n\
             \n\
             Tools: workspace_context, gitkraken_cloud_catalog, coding_agent_toolchains_catalog, gitkraken_hosted_tool_redirect, mcp_env_hints, agent_coordination_playbook, ai_inference_integration_guide, ai_stack_probe, server_sleep, wait_until_timestamp, wait_for_path, terminal_run, terminal_run_stdin, project_cargo_test, project_cargo_check, project_pytest, terminal_which, terminal_program_version, screenshot_capture, workspace_temp_write, workspace_temp_read, workspace_temp_list, workspace_temp_delete, ocr_tesseract, ollama_list_models, gh_pr_list, gh_pr_view, glab_mr_list, glab_mr_show, read_file, read_file_tail, list_files, grep_code, find_files, find_files_by_name, \
             project_tree, codebase_index, codebase_word_index, codebase_shallow_hash, codebase_path_list, codebase_largest_files, codebase_duplicate_basenames, codebase_cargo_dependency_names, codebase_manifest_write, code_index_build, code_index_search, code_index_status, code_index_file_symbols, code_index_aggregate_stats, code_index_paths_prefix, code_index_path_substring, code_index_symbol_prefix, code_index_duplicate_symbols, code_index_paths_without_symbols, code_index_extension_histogram, code_index_symbol_frequency, file_info, file_stats, batch_file_info, json_validate_file, convert_format, compress_gzip_base64, decompress_gzip_base64, \
             git_status, gk_git_status, git_log_or_diff, gk_git_log_or_diff, git_branch, gk_git_branch, git_branches, git_worktree, gk_git_worktree, git_worktree_list, git_stash, gk_git_stash, git_stash_list, git_stash_push, git_checkout, gk_git_checkout, git_add_or_commit, gk_git_add_or_commit, git_push, gk_git_push, git_fetch, git_pull, git_remote_list, git_tag_list, git_ls_files, git_diff_stat, git_merge_base, git_describe, git_shortlog, git_reflog, git_last_commit, \
             git_diff, git_log, git_show, git_blame, gk_git_blame, git_rev_parse, repository_get_file_content, \
             compress_context, text_stats, compress_ws_collapse, compress_simplify_whitespace, compress_markdown_links, compress_unified_diff_stats, compress_truncate_lines, compress_strip_tags, compress_fenced_code_blocks, compress_redact_urls, compress_redact_emails, compress_markdown_frontmatter, compress_double_slash_line_comments, compress_drop_lines_matching_regex, compress_paragraphs, compress_sentences, compress_unique_lines, compress_lines_head, compress_lines_tail, compress_collapse_blank_lines, compress_truncate_middle, compress_dedent, compress_file, extract_snippets, extract_snippets_from_file, grep_snippets, \
             summarize_text, summarize_top_terms, summarize_identifier_frequency, summarize_token_estimate, summarize_duplicate_lines, summarize_line_histogram, summarize_line_prefix_groups, summarize_rank_lines_by_keywords, summarize_bullets, summarize_headings, summarize_markdown_toc, summarize_json_structure, summarize_delimited_header, summarize_diff_file_list, summarize_first_lines, summarize_segments, summarize_file, summarize_multi_file, format_prefix_lines, pack_context, git_diff_snippets, text_levenshtein_distance, text_token_jaccard, text_longest_common_substring, text_djb2_hash, \
             thinking_plan_skeleton, thinking_step_back, thinking_assumption_audit, thinking_session_append, thinking_session_read, plan_execution_checklist, plan_handoff_brief, plan_work_buckets, plan_risk_register, plan_definition_of_done, plan_test_strategy, plan_retrospective, reasoning_premortem, reasoning_five_whys, reasoning_chain, reasoning_devils_advocate, reasoning_evidence_matrix, \
             agent_ping, agent_bus_compact, agent_bus_search, agent_bus_status, agent_coordination_snapshot, agent_list_topics, agent_roster, agent_get_message, \
             agent_send, agent_handoff, agent_subagent_task, agent_subagent_done, agent_fan_out, agent_send_pipeline, agent_inbox, agent_multi_inbox, agent_inbox_wait, agent_thread, agent_topic_wait, agent_recent, ask_agents, consensus_propose, consensus_vote, consensus_tally, consensus_list, consensus_close."
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
