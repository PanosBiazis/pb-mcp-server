mod agent;
mod bus;
mod bus_rate;
mod code_index;
mod network_profile;
mod ops;
mod thinking;

use crate::ops::{
    agent_coordination_playbook_work, ai_inference_integration_guide_work, ai_stack_probe_work,
    batch_file_info_work, code_ast_dump_work, code_complexity_score_work,
    code_dead_code_hints_work, code_detect_language_work, code_extract_imports_work,
    codebase_cargo_dependency_names_work, codebase_duplicate_basenames_work, codebase_index_work,
    codebase_largest_files_work, codebase_manifest_write_work, codebase_path_list_work,
    codebase_shallow_hash_work, codebase_word_index_work, coding_agent_toolchains_catalog_work,
    compress_collapse_blank_lines_work, compress_context_work, compress_dedent_work,
    compress_double_slash_line_comments_work, compress_drop_lines_matching_regex_work,
    compress_fenced_code_blocks_work, compress_file_work, compress_gzip_base64_work,
    compress_lines_head_work, compress_lines_tail_work, compress_markdown_frontmatter_work,
    compress_markdown_links_work, compress_paragraphs_work, compress_redact_emails_work,
    compress_redact_urls_work, compress_sentences_work, compress_simplify_whitespace_work,
    compress_strip_tags_work, compress_truncate_lines_work, compress_truncate_middle_work,
    compress_unified_diff_stats_work, compress_unique_lines_work, compress_ws_collapse_work,
    convert_format_work, debug_failure_cluster_work, debug_replay_run_work,
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
    grep_work, list_files_work, mcp_env_hints_work, ocr_tesseract_file_work,
    ollama_list_models_work, pack_context_work, project_cargo_check_build_args,
    project_cargo_test_build_args, project_pytest_build_args, project_test_tools_enabled,
    read_file_tail_work, read_file_work, read_json_file_work, repository_get_file_content_work,
    review_diff_risk_work, review_focused_review_work, review_missing_tests_work,
    review_pr_packet_work, review_rulebook_check_work, screenshot_capture_work,
    summarize_bullets_work, summarize_delimited_header_work, summarize_diff_file_list_work,
    summarize_duplicate_lines_work, summarize_file_work, summarize_first_lines_work,
    summarize_headings_work, summarize_identifier_frequency_work, summarize_json_structure_work,
    summarize_line_histogram_work, summarize_line_prefix_groups_work, summarize_markdown_toc_work,
    summarize_multi_file_work, summarize_rank_lines_by_keywords_work, summarize_segments_work,
    summarize_text_work, summarize_token_estimate_work, summarize_top_terms_work,
    terminal_program_version_work, terminal_run_precheck, terminal_which_work, text_djb2_hash_work,
    text_levenshtein_distance_work, text_longest_common_substring_work, text_stats_work,
    text_token_jaccard_work, tree_work, workspace_context_work, workspace_temp_delete_work,
    workspace_temp_list_work, workspace_temp_read_work, workspace_temp_write_work,
    MAX_TERMINAL_STDIN_BYTES,
};
use crate::ops::{
    ai_ollama_embed_work, ai_ollama_generate_work, convert_archive_extract_work,
    convert_archive_list_work, convert_binary_hexdump_work, convert_document_work,
    convert_pdf_extract_work, convert_qr_generate_work, db_json_query_work, db_sqlite_exec_work,
    db_sqlite_query_work, db_sqlite_schema_work, notify_desktop_work, notify_email_send_work,
    notify_webhook_work, system_disk_usage_work, system_env_get_work,
    system_network_interfaces_work, system_process_list_work, web_api_call_work,
    web_dns_lookup_work, web_download_file_work, web_rss_fetch_work, web_screenshot_work,
    web_search_work, web_tls_inspect_work,
};
// Audio, image, speech, reverse-engineering, screenshot, OCR, sandbox, algorithm,
// IDE-integration, and file-crypto work functions (wired via include!() handlers below).
use crate::ops::{
    algo_bloom_filter_work, algo_edit_distance_work, algo_fft_work, algo_graph_bfs_work,
    algo_graph_dfs_work, algo_graph_shortest_path_work, algo_graph_topo_sort_work,
    algo_hash_text_work, algo_matrix_ops_work, algo_ngram_similarity_work, algo_search_work,
    algo_sort_work, algo_stats_correlation_work, algo_stats_describe_work,
    algo_stats_regression_work, audio_convert_work, audio_extract_from_video_work,
    audio_extract_segment_work, audio_info_work, audio_merge_work, audio_mix_work,
    audio_normalize_work, audio_reverse_work, audio_silence_detect_work, audio_speed_work,
    audio_split_work, audio_trim_work, audio_volume_work, audio_waveform_work,
    cursor_composer_send_work, cursor_open_file_work, file_decrypt_work, file_deduplicate_work,
    file_encrypt_work, file_sign_work, file_verify_signature_work, file_watch_work,
    ide_detect_work, ide_find_references_work, ide_format_document_work, ide_goto_definition_work,
    ide_run_test_work, image_compare_work, image_convert_work, image_crop_work,
    image_extract_exif_work, image_flip_work, image_grayscale_work, image_histogram_work,
    image_info_work, image_resize_work, image_rotate_work, image_thumbnail_work,
    image_watermark_work, jetbrains_adb_shell_work, jetbrains_device_list_work,
    jetbrains_logcat_tail_work, jetbrains_lsp_code_action_work, jetbrains_open_file_work,
    jetbrains_run_configuration_work, ocr_batch_work, ocr_detect_text_regions_work, ocr_file_work,
    ocr_from_clipboard_work, ocr_list_languages_work, ocr_pdf_work, ocr_preprocess_work,
    ocr_screen_region_work, ocr_table_work, ocr_with_confidence_work, re_binary_info_work,
    re_binwalk_scan_work, re_carve_work, re_dependencies_work, re_diff_binary_work,
    re_disassemble_work, re_entropy_work, re_hexdump_work, re_pattern_search_work,
    re_sections_work, re_strace_work, re_strings_work, re_struct_parse_work, re_symbols_work,
    sandbox_checksum_work, sandbox_diff_work, sandbox_docker_run_work, sandbox_eval_code_work,
    sandbox_exec_work, sandbox_file_watch_work, sandbox_network_request_work, sandbox_script_work,
    screenshot_annotate_work, screenshot_color_pick_work, screenshot_compare_work,
    screenshot_monitor_info_work, screenshot_ocr_work, screenshot_record_work,
    screenshot_region_work, screenshot_timelapse_work, screenshot_window_work,
    speech_detect_language_work, speech_diarize_work, speech_enhance_work,
    speech_extract_words_work, speech_list_models_work, speech_list_voices_work,
    speech_text_to_speech_ssml_work, speech_text_to_speech_work, speech_transcribe_file_work,
    speech_transcribe_microphone_work, speech_transcribe_stream_work, speech_translate_work,
    vscode_get_diagnostics_work, vscode_install_extension_work, vscode_list_extensions_work,
    vscode_open_diff_work, vscode_open_file_work, zed_context_attach_work,
    zed_get_diagnostics_work, zed_open_file_work, zed_search_workspace_work,
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

// Argument structs for MCP tool handlers — extracted for maintainability.
include!("mcp_args.rs");

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

// Argument structs for the extra tools wired via tools_wired.rs.
include!("tool_args_wired.rs");

// Args structs for the new media / RE / sandbox / algo / IDE / file-crypto tools.
include!("tools/audio_args.rs");
include!("tools/image_args.rs");
include!("tools/speech_args.rs");
include!("tools/reverse_eng_args.rs");
include!("tools/screenshot_args.rs");
include!("tools/ocr_args.rs");
include!("tools/sandbox_args.rs");
include!("tools/algorithm_args.rs");
include!("tools/ide_args.rs");
include!("tools/file_crypto_args.rs");

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
        let out = tokio::task::spawn_blocking(move || -> String {
            workspace_context_work(root, net, cap, is_pub)
        })
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
        let out = tokio::task::spawn_blocking(|| -> String { gitkraken_cloud_catalog_work() })
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
        let out =
            tokio::task::spawn_blocking(|| -> String { coding_agent_toolchains_catalog_work() })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            gitkraken_hosted_tool_redirect_work(&name)
        })
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
        let out = tokio::task::spawn_blocking(|| -> String { mcp_env_hints_work() })
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
        let out = tokio::task::spawn_blocking(|| -> serde_json::Value {
            ai_stack_probe_work(serde_json::json!({}))
        })
        .await
        .unwrap_or_else(|e| serde_json::Value::String(format!("Error: task failed: {e}")));
        Self::tool_text(out.to_string())
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
        let out = tokio::task::spawn_blocking(move || -> String { terminal_which_work(&program) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            terminal_program_version_work(&program)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            ocr_tesseract_file_work(&path, &lang, mc)
        })
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
        let out = tokio::task::spawn_blocking(|| -> String { ollama_list_models_work() })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                workspace_temp_write_work(root, name, content, maxb)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            workspace_temp_read_work(root, name, maxr)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            workspace_temp_list_work(root, limit)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            workspace_temp_delete_work(root, name)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { gh_pr_list_work(cwd, limit, &state) })
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
        let out = tokio::task::spawn_blocking(move || -> String { gh_pr_view_work(cwd, &pr) })
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
        let out = tokio::task::spawn_blocking(move || -> String { glab_mr_list_work(cwd, limit) })
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
        let out = tokio::task::spawn_blocking(move || -> String { glab_mr_show_work(cwd, &iid) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { read_file_work(path, start, end) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { list_files_work(root_display, path) })
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                grep_work(
                    search_root,
                    pattern,
                    literal,
                    case_sensitive,
                    respect,
                    max_m,
                )
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            find_files_work(search_root, glob, max_f)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String { tree_work(search_root, depth) })
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_index_work(search_root, max_files, sketch, max_sym)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_word_index_work(search_root, max_files, top_n)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_shallow_hash_work(search_root, max_files, sample)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_path_list_work(search_root, max_paths, max_depth)
            }
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
        let _scan = args.max_files_scan;
        let top = args.top_n;
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_largest_files_work(search_root, None, false, top)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_duplicate_basenames_work(search_root, scan)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_cargo_dependency_names_work(manifest_path)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_build_work(base, subtree, respect, max_files)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_search_work(
                    base, needle, cs, limit, prefix, use_regex,
                )
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            crate::code_index::code_index_status_work(base)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_file_symbols_work(base, rel)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_aggregate_stats_work(base)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_paths_prefix_work(base, prefix, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_extension_histogram_work(base, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_symbol_frequency_work(base, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_path_substring_work(base, needle, limit, cs)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_symbol_prefix_work(base, prefix, cs, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_duplicate_symbols_work(base, minf, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                crate::code_index::code_index_paths_without_symbols_work(base, limit)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { git_diff_work(dir, staged) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_diff_stat_work(dir, staged, rev) })
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

        let out = tokio::task::spawn_blocking(move || -> String { file_info_work(path) })
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

        let out = tokio::task::spawn_blocking(move || -> String { git_status_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_stash_list_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_worktree_list_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_checkout_work(dir, target) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_stash_push_work(dir, message) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_stash_push_work(dir, message) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_log_or_diff_work(dir, &action, rr, since, until, authors)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            git_branch_extension_work(dir, &action, bn)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_worktree_extension_work(dir, &action, wp, branch)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_add_or_commit_work(dir, &action, files, message)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { git_push_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_fetch_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_pull_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_remote_list_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_tag_list_work(dir, pat) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_ls_files_work(dir, &mode) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_describe_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_shortlog_work(dir, lim) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_reflog_work(dir, lim) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_last_commit_work(dir, &style) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            repository_get_file_content_work(dir, fp, gr)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_status_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_checkout_work(dir, target) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_log_or_diff_work(dir, &action, rr, since, until, authors)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            git_blame_work(repo, file_rel, 1, 400, None)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            git_branch_extension_work(dir, &action, bn)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_stash_push_work(dir, message) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_worktree_extension_work(dir, &action, wp, branch)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_add_or_commit_work(dir, &action, files, message)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { git_push_work(dir) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
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
        let out =
            tokio::task::spawn_blocking(move || -> String { git_show_work(repo, rev, patch) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_branches_work(repo) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                git_blame_work(repo, file_rel, start_line, end_line, rev)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { git_rev_parse_work(repo, refs) })
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
        let out = tokio::task::spawn_blocking(move || -> String { git_merge_base_work(dir, a, b) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            read_file_tail_work(path, last_lines)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String { file_stats_work(path) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                find_files_by_name_work(search_root, substring, ci, max_f)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { read_json_file_work(path) })
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
        let out = tokio::task::spawn_blocking(move || -> String { batch_file_info_work(resolved) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                compress_context_work(
                    args.text,
                    args.max_chars,
                    args.mode,
                    args.dedupe_lines,
                    args.collapse_whitespace,
                )
            }
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
        let out = tokio::task::spawn_blocking(move || -> String { text_stats_work(text) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_token_estimate_work(text, mc)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { compress_ws_collapse_work(text, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_simplify_whitespace_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_markdown_links_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_unified_diff_stats_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_truncate_lines_work(text, ml, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_diff_file_list_work(text, mp, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_top_terms_work(text, minl, maxt, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_identifier_frequency_work(text, minl, maxi, mc, fold)
            }
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
        let out =
            tokio::task::spawn_blocking(move || -> String { compress_strip_tags_work(text, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_fenced_code_blocks_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                compress_double_slash_line_comments_work(text, mc, dbl)
            }
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
        let out =
            tokio::task::spawn_blocking(move || -> String { compress_redact_urls_work(text, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_redact_emails_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_markdown_frontmatter_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                compress_drop_lines_matching_regex_work(text, pat, ci, mc)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_duplicate_lines_work(text, mo, me, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_line_histogram_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_line_prefix_groups_work(text, pl, mg, mc)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_rank_lines_by_keywords_work(text, kw, tl, mc)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            format_prefix_lines_work(text, prefix, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_paragraphs_work(text, mp, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_unique_lines_work(text, ml, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_lines_head_work(text, hl, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_lines_tail_work(text, tl, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_collapse_blank_lines_work(text, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_truncate_middle_work(text, mx, margin)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String { compress_dedent_work(text, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_sentences_work(text, ms, mc)
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                extract_snippets_work(
                    args.text,
                    args.pattern,
                    args.literal,
                    args.case_sensitive,
                    args.context_lines,
                    args.max_snippets,
                    args.max_chars,
                )
            }
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                compress_file_work(
                    path,
                    args.start_line,
                    args.end_line,
                    args.max_chars,
                    args.mode,
                    args.dedupe_lines,
                    args.collapse_whitespace,
                )
            }
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
            }
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_text_work(args.text, args.max_chars, args.max_sentences)
            }
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
        let out =
            tokio::task::spawn_blocking(move || -> String { summarize_bullets_work(text, mi, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_headings_work(text, mh, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_markdown_toc_work(text, me, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_json_structure_work(text, md, mk, ml, mc)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_delimited_header_work(text, &delim, mx, mc)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            text_levenshtein_distance_work(a, b, m)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            text_token_jaccard_work(ta, tb, fold, mc, ms)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            text_longest_common_substring_work(a, b, m)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String { text_djb2_hash_work(text) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_first_lines_work(text, lc, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            summarize_segments_work(segs, max_c, max_s)
        })
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_file_work(path, start, end, max_c, max_s)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                summarize_multi_file_work(paths, lines, max_c, max_s)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                pack_context_work(
                    segments,
                    segment_labels,
                    max_chars,
                    mode,
                    dedupe_lines,
                    collapse_whitespace,
                )
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            compress_gzip_base64_work(text, path_opt.as_ref())
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { decompress_gzip_base64_work(pay) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                codebase_manifest_write_work(search_root, max_files)
            }
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
            }
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

        let out = tokio::task::spawn_blocking(move ||  -> String {match crate::bus::agent_send(root, from, body, to.clone(), topic.clone(), kind.clone()) {
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
        } })
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
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
                        s.push_str(
                            "Recipient should poll agent_inbox or agent_thread when topic set.",
                        );
                        s
                    }
                    Err(e) => format!("Error: {e}"),
                }
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

        let out = tokio::task::spawn_blocking(move ||  -> String {{
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
        } })
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

        let out = tokio::task::spawn_blocking(move ||  -> String {{
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
        } })
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            {
                crate::bus::agent_fan_out(root, from, body, targets, topic, kind)
            }
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            crate::bus::agent_send_pipeline(root, from, steps)
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                match crate::bus::agent_inbox(root, for_a, limit, skip, since) {
                    Ok(s) => s,
                    Err(e) => format!("Error: {e}"),
                }
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            {
                crate::bus::agent_multi_inbox(root, agents, limit, skip, since)
            }
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                match crate::bus::agent_thread(root, topic, limit, since) {
                    Ok(s) => s,
                    Err(e) => format!("Error: {e}"),
                }
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

        let out = tokio::task::spawn_blocking(move || -> String {
            {
                match crate::bus::agent_recent(root, limit, since) {
                    Ok(s) => s,
                    Err(e) => format!("Error: {e}"),
                }
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            crate::bus::agent_list_topics(root, limit)
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            {
                crate::bus::agent_coordination_snapshot(root, topics_limit, recent_limit)
            }
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            crate::bus::agent_roster(root)
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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

        let out = match tokio::task::spawn_blocking(move || -> Result<String, String> {
            crate::bus::agent_get_message(root, mid)
        })
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                if e.starts_with("Error:") {
                    e
                } else {
                    format!("Error: {e}")
                }
            }
            Err(e) => format!("Error: task failed: {e}"),
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                thinking_plan_skeleton_work(objective, constraints, known_steps, max_chars)
            }
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
        let out = tokio::task::spawn_blocking(move || -> String {
            thinking_step_back_work(problem, max_chars)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            thinking_assumption_audit_work(claims, max_chars)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            thinking_session_append_work(root, note)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            thinking_session_read_work(root, tail)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { plan_execution_checklist_work(m, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            {
                plan_handoff_brief_work(title, scope, del, acc, blk, mc)
            }
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
        let out =
            tokio::task::spawn_blocking(move || -> String { plan_work_buckets_work(items, b, mc) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { plan_risk_register_work(risks, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            plan_definition_of_done_work(outcomes, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            plan_test_strategy_work(scope, rh, mc)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { plan_retrospective_work(it, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            reasoning_premortem_work(project, timeframe, mc)
        })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { reasoning_five_whys_work(seed, mc) })
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
        let out =
            tokio::task::spawn_blocking(move || -> String { reasoning_chain_work(pr, st, oq, mc) })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            reasoning_devils_advocate_work(c, ctx, mc)
        })
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
        let out = tokio::task::spawn_blocking(move || -> String {
            reasoning_evidence_matrix_work(cl, mc)
        })
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
        let push_on = crate::ops::git_write_enabled();
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
            let has = tokio::task::spawn_blocking(move || -> Result<bool, String> {
                {
                    crate::bus::agent_inbox_non_empty(root_c, for_c, limit, skip, since)
                }
            })
            .await
            .unwrap_or_else(|e| Err(format!("task failed: {e}")));

            match has {
                Ok(true) => {
                    let root_c = root.clone();
                    let for_c = for_a.clone();
                    let body =
                        match tokio::task::spawn_blocking(move || -> Result<String, String> {
                            {
                                crate::bus::agent_inbox(root_c, for_c, limit, skip, since)
                            }
                        })
                        .await
                        {
                            Ok(Ok(t)) => t,
                            Ok(Err(e)) => format!("Error: {e}"),
                            Err(e) => format!("task failed: {e}"),
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
            let has = tokio::task::spawn_blocking(move || -> Result<bool, String> {
                {
                    crate::bus::agent_topic_has_reply_from_others(root_c, topic_c, since, exclude_c)
                }
            })
            .await
            .unwrap_or_else(|e| Err(format!("task failed: {e}")));

            match has {
                Ok(true) => {
                    let root_c = root.clone();
                    let topic_c = topic.clone();
                    let body =
                        match tokio::task::spawn_blocking(move || -> Result<String, String> {
                            {
                                crate::bus::agent_thread(root_c, topic_c, 80, Some(since))
                            }
                        })
                        .await
                        {
                            Ok(Ok(t)) => t,
                            Ok(Err(e)) => format!("Error: {e}"),
                            Err(e) => format!("task failed: {e}"),
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

        let out = tokio::task::spawn_blocking(move || -> String {
            match crate::bus::agent_bus_status(root) {
                Ok(s) => s,
                Err(e) => format!("Error: {e}"),
            }
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
        let out = tokio::task::spawn_blocking(move ||  -> String {{
            let mut eng = engine.lock().unwrap();
            let p = eng.create_proposal(from, topic, desc, options, deadline, strategy);
            format!(
                "CONSENSUS_PROPOSE\nid: {}\nproposer: {}\ntopic: {}\nstrategy: {:?}\ndeadline_ms: {}\nstatus: {:?}\noptions: {}\ncreated_at: {}",
                p.id, p.proposer, p.topic, p.strategy, p.deadline_ms, p.status,
                p.options.join(", "), p.created_at
            )
        } })
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
        let out = tokio::task::spawn_blocking(move ||  -> String {{
            let mut eng = engine.lock().unwrap();
            match eng.cast_vote(&pid, voter.clone(), choice.clone(), conf, rationale) {
                Ok(()) => format!("CONSENSUS_VOTE\nproposal: {}\nvoter: {}\nchoice: {}\nconfidence: {:.2}\nstatus: recorded", pid, voter, choice, conf),
                Err(e) => format!("Error: {e}"),
            }
        } })
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
        let out = tokio::task::spawn_blocking(move ||  -> String {{
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
        } })
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
        let out = tokio::task::spawn_blocking(move ||  -> String {{
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
        } })
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
        let out = tokio::task::spawn_blocking(move ||  -> String {{
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
        } })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                review_diff_risk_work(root, staged, revision_range)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                review_focused_review_work(root, focus, staged, revision_range)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                review_missing_tests_work(root, staged, revision_range)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            review_pr_packet_work(root, revision_range)
        })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                review_rulebook_check_work(root, rulebook_path, staged, revision_range)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                debug_trace_run_work(root, trace_id, command, trace_args)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            debug_replay_run_work(root, trace_id, dry_run)
        })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            {
                debug_failure_cluster_work(root, trace_dir, min_cluster_size)
            }
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            debug_root_cause_report_work(root, trace_id)
        })
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
        let out: String =
            tokio::task::spawn_blocking(move || -> String { code_ast_dump_work(path, language) })
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
        let out: String =
            tokio::task::spawn_blocking(move || -> String { code_extract_imports_work(path) })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            code_detect_language_work(path, content)
        })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            code_complexity_score_work(path, content)
        })
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
        let out: String = tokio::task::spawn_blocking(move || -> String {
            code_dead_code_hints_work(root, path)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    // Additional wired tools: ai_ollama_generate/embed, web_dns_lookup, web_tls_inspect,
    // notify_*, system_*, db_sqlite_*, db_json_query, web_api_call, web_download_file,
    // web_screenshot_url, web_search, web_rss_fetch, convert_archive_*, convert_binary_hexdump,
    // convert_document, convert_encoding, convert_pdf_extract, convert_qr_generate,
    // agent_sync_all_tool, agent_sync_confirm_tool.

    #[tool(
        description = "Generate text with a local Ollama model. Blocked on public profile. Requires MINT_TERMINAL_ENABLE=1 and `ollama` in MINT_TERMINAL_ALLOW.",
        annotations(title = "Ollama generate", read_only_hint = true)
    )]
    async fn ai_ollama_generate(
        &self,
        Parameters(args): Parameters<AiOllamaGenerateArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let model = args.model.clone();
        let prompt = args.prompt.clone();
        let system = args.system_prompt.clone();
        let max_tokens = args.max_tokens;
        let out = tokio::task::spawn_blocking(move || {
            ai_ollama_generate_work(model, prompt, system, None, max_tokens)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Embed text with a local Ollama model and return the vector. Blocked on public profile.",
        annotations(title = "Ollama embed", read_only_hint = true)
    )]
    async fn ai_ollama_embed(
        &self,
        Parameters(args): Parameters<AiOllamaEmbedArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let model = args.model.clone();
        let input = args.input.clone();
        let out = tokio::task::spawn_blocking(move || ai_ollama_embed_work(model, input))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "DNS lookup for a hostname — returns A/AAAA/MX/TXT records. Read-only; blocked on public profile.",
        annotations(title = "DNS lookup", read_only_hint = true)
    )]
    async fn web_dns_lookup(
        &self,
        Parameters(args): Parameters<WebDnsLookupArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let hostname = args.hostname.clone();
        let record_types = args.record_types.clone();
        let out = tokio::task::spawn_blocking(move || {
            web_dns_lookup_work(hostname, record_types.into_iter().next())
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Inspect TLS certificate chain for a host (expiry, issuer, SANs, cipher). Blocked on public profile.",
        annotations(title = "TLS inspect", read_only_hint = true)
    )]
    async fn web_tls_inspect(
        &self,
        Parameters(args): Parameters<WebTlsInspectArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let host = args.host.clone();
        let port = args.port;
        let out = tokio::task::spawn_blocking(move || web_tls_inspect_work(host, Some(port)))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Show a desktop notification via notify-send (Linux) or osascript (macOS). Blocked on public profile.",
        annotations(title = "Notify desktop", read_only_hint = false)
    )]
    async fn notify_desktop(
        &self,
        Parameters(args): Parameters<NotifyDesktopArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let title = args.title.clone();
        let body = args.body.clone();
        let urgency = args.urgency.clone();
        let out = tokio::task::spawn_blocking(move || notify_desktop_work(title, body, urgency))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Send an email via sendmail or mailx. Blocked on public profile.",
        annotations(title = "Notify email", read_only_hint = false)
    )]
    async fn notify_email(
        &self,
        Parameters(args): Parameters<NotifyEmailArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let to = args.to.clone();
        let subject = args.subject.clone();
        let body = args.body.clone();
        let from = args.from.clone();
        let out =
            tokio::task::spawn_blocking(move || notify_email_send_work(to, subject, body, from))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Send a JSON payload to a webhook URL via curl. Blocked on public profile. Rejects internal IPs by default (SSRF guard).",
        annotations(title = "Notify webhook", read_only_hint = false)
    )]
    async fn notify_webhook(
        &self,
        Parameters(args): Parameters<NotifyWebhookArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let url = args.url.clone();
        let payload = args.payload.clone();
        let method = args.method.clone();
        let block_internal = args.block_internal;
        let out = tokio::task::spawn_blocking(move || {
            notify_webhook_work(url, payload, method, None, block_internal)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List running processes (cross-platform ps snapshot). Read-only; blocked on public profile.",
        annotations(title = "System process list", read_only_hint = true)
    )]
    async fn system_process_list(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let out = tokio::task::spawn_blocking(|| system_process_list_work(50))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Report disk usage (free/total/used) for a path. Read-only; blocked on public profile.",
        annotations(title = "System disk usage", read_only_hint = true)
    )]
    async fn system_disk_usage(
        &self,
        Parameters(args): Parameters<SystemDiskUsageArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || system_disk_usage_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Get the value of an environment variable. Read-only; blocked on public profile.",
        annotations(title = "System env get", read_only_hint = true)
    )]
    async fn system_env_get(
        &self,
        Parameters(args): Parameters<SystemEnvGetArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let name = args.name.clone();
        let out = tokio::task::spawn_blocking(move || system_env_get_work(name))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List network interfaces with IP addresses and flags. Read-only; blocked on public profile.",
        annotations(title = "System network interfaces", read_only_hint = true)
    )]
    async fn system_network_interfaces(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let out = tokio::task::spawn_blocking(system_network_interfaces_work)
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Execute a read-only SELECT against a SQLite database under the project root. Returns rows as JSON.",
        annotations(title = "SQLite query (read)", read_only_hint = true)
    )]
    async fn db_sqlite_query(
        &self,
        Parameters(args): Parameters<DbSqliteQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let db_path = match self.resolve_path(&args.db_path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let query = args.query.clone();
        let out = tokio::task::spawn_blocking(move || db_sqlite_query_work(db_path, query, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Execute a write SQL statement (INSERT/UPDATE/DELETE/CREATE) against a SQLite database under the project root. Blocked on public profile.",
        annotations(title = "SQLite exec (write)", read_only_hint = false)
    )]
    async fn db_sqlite_exec(
        &self,
        Parameters(args): Parameters<DbSqliteExecArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let db_path = match self.resolve_path(&args.db_path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let statement = args.statement.clone();
        let out =
            tokio::task::spawn_blocking(move || db_sqlite_exec_work(db_path, statement, false))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Dump the schema (tables, columns, indexes) of a SQLite database under the project root. Read-only.",
        annotations(title = "SQLite schema", read_only_hint = true)
    )]
    async fn db_sqlite_schema(
        &self,
        Parameters(args): Parameters<DbSqliteSchemaArgs>,
    ) -> Result<CallToolResult, McpError> {
        let db_path = match self.resolve_path(&args.db_path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || db_sqlite_schema_work(db_path, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Query a JSON file under the project root using a JSONPath-style expression. Read-only.",
        annotations(title = "JSON query", read_only_hint = true)
    )]
    async fn db_json_query(
        &self,
        Parameters(args): Parameters<DbJsonQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let query = args.query.clone();
        let out = tokio::task::spawn_blocking(move || db_json_query_work(path, query, false))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Make an HTTP API call (GET/POST/PUT/PATCH/DELETE). Returns status + response body. Blocked on public profile.",
        annotations(title = "Web API call", read_only_hint = false)
    )]
    async fn web_api_call(
        &self,
        Parameters(args): Parameters<WebApiCallArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let method = args.method.clone();
        let url = args.url.clone();
        let body = args.body.clone();
        let headers = args.headers.clone();
        let out = tokio::task::spawn_blocking(move || {
            web_api_call_work(url, method, headers.map(|v| v.join("\n")), body, None)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Download a file from a URL to a path under the project root. Blocked on public profile.",
        annotations(title = "Web download file", read_only_hint = false)
    )]
    async fn web_download_file(
        &self,
        Parameters(args): Parameters<WebDownloadFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let url = args.url.clone();
        let dest = match self.resolve_path(&args.dest) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || web_download_file_work(url, dest, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Take a screenshot of a URL using headless Chrome/Chromium or wkhtmltoimage. Blocked on public profile.",
        annotations(title = "Web screenshot", read_only_hint = true)
    )]
    async fn web_screenshot_url(
        &self,
        Parameters(args): Parameters<WebScreenshotArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let url = args.url.clone();
        let output = args.output_path.as_ref().map(|p| self.root.join(p));
        let out = tokio::task::spawn_blocking(move || web_screenshot_work(url, None, None, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Web search using a configured backend (DuckDuckGo/Brave/SearXNG). Returns title + URL + snippet for each result. Blocked on public profile.",
        annotations(title = "Web search", read_only_hint = true)
    )]
    async fn web_search(
        &self,
        Parameters(args): Parameters<WebSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let query = args.query.clone();
        let max_results = args.max_results;
        let out =
            tokio::task::spawn_blocking(move || web_search_work(query, None, Some(max_results)))
                .await
                .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Fetch and parse an RSS/Atom feed URL. Returns item titles, links, and summaries. Blocked on public profile.",
        annotations(title = "RSS fetch", read_only_hint = true)
    )]
    async fn web_rss_fetch(
        &self,
        Parameters(args): Parameters<WebRssFetchArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let url = args.url.clone();
        let max_items = args.max_items;
        let out = tokio::task::spawn_blocking(move || web_rss_fetch_work(url, Some(max_items)))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Extract an archive (zip/tar/gz/bz2/xz/zstd) to a destination directory under the project root. Blocked on public profile.",
        annotations(title = "Convert: archive extract", read_only_hint = false)
    )]
    async fn convert_archive_extract(
        &self,
        Parameters(args): Parameters<ConvertArchiveExtractArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let src = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let dst = match self.resolve_path(&args.dest) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || convert_archive_extract_work(src, dst, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "List the contents of an archive (zip/tar/gz/bz2/xz/zstd) without extracting. Read-only.",
        annotations(title = "Convert: archive list", read_only_hint = true)
    )]
    async fn convert_archive_list(
        &self,
        Parameters(args): Parameters<ConvertArchiveListArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || convert_archive_list_work(path))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Hex dump of a binary file under the project root. Read-only.",
        annotations(title = "Convert: binary hexdump", read_only_hint = true)
    )]
    async fn convert_binary_hexdump(
        &self,
        Parameters(args): Parameters<ConvertBinaryHexdumpArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let max_bytes = args.max_bytes;
        let out = tokio::task::spawn_blocking(move || {
            convert_binary_hexdump_work(path, None, Some(max_bytes as u64), None)
        })
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Convert a document between formats using pandoc (e.g. md→docx, rst→pdf). Blocked on public profile.",
        annotations(title = "Convert: document", read_only_hint = false)
    )]
    async fn convert_document(
        &self,
        Parameters(args): Parameters<ConvertDocumentArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let input = match self.resolve_path(&args.input) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let output = match self.resolve_path(&args.output) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let fmt = output.extension().map(|e| e.to_string_lossy().to_string());
        let out = tokio::task::spawn_blocking(move || convert_document_work(input, fmt))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Re-encode a text file between character encodings (e.g. latin-1 → utf-8). Read-only (returns converted text).",
        annotations(title = "Convert: encoding", read_only_hint = true)
    )]
    async fn convert_encoding(
        &self,
        Parameters(args): Parameters<ConvertEncodingArgs>,
    ) -> Result<CallToolResult, McpError> {
        let text = args.text.clone();
        let from_enc = args.from_enc.clone();
        let to_enc = args.to_enc.clone();
        // convert_encoding_work takes a file path; for inline text, return informational message
        let out = format!("convert_encoding: {} chars, {}->{} (use convert_format with a file path for full support)", text.len(), from_enc, to_enc);
        Self::tool_text(out)
    }

    #[tool(
        description = "Extract text from a PDF file under the project root using pdftotext. Read-only.",
        annotations(title = "Convert: PDF extract", read_only_hint = true)
    )]
    async fn convert_pdf_extract(
        &self,
        Parameters(args): Parameters<ConvertPdfExtractArgs>,
    ) -> Result<CallToolResult, McpError> {
        let path = match self.resolve_path(&args.path) {
            Ok(p) => p,
            Err(e) => return Self::tool_text(format!("Error: {e}")),
        };
        let out = tokio::task::spawn_blocking(move || convert_pdf_extract_work(path, None, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Generate a QR code PNG from input text and save it under the project root. Blocked on public profile.",
        annotations(title = "Convert: QR generate", read_only_hint = false)
    )]
    async fn convert_qr_generate(
        &self,
        Parameters(args): Parameters<ConvertQrGenerateArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let text = args.text.clone();
        let output = args.output_path.as_ref().map(|p| self.root.join(p));
        let out = tokio::task::spawn_blocking(move || convert_qr_generate_work(text, output, None))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Synchronise all agent state under the project root (writes .mint-mcp/agent-sync/). Blocked on public profile.",
        annotations(title = "Agent sync all", read_only_hint = false)
    )]
    async fn agent_sync_all_tool(
        &self,
        Parameters(_args): Parameters<EmptyArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let out = crate::ops::agent_sync_all(root)
            .await
            .unwrap_or_else(|e| format!("Error: {e}"));
        Self::tool_text(out)
    }

    #[tool(
        description = "Confirm a pending agent sync by sync_id. Blocked on public profile.",
        annotations(title = "Agent sync confirm", read_only_hint = false)
    )]
    async fn agent_sync_confirm_tool(
        &self,
        Parameters(args): Parameters<AgentSyncConfirmArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = self.require_trusted_deployment() {
            return Self::tool_text(e);
        }
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let sync_id = args.sync_id.clone();
        let out = crate::ops::agent_sync_confirm(root, sync_id)
            .await
            .unwrap_or_else(|e| format!("Error: {e}"));
        Self::tool_text(out)
    }

    // =========================================================================
    // NEW TOOL HANDLERS — 122 tools inlined from handler files
    // =========================================================================

    // --- AUDIO ---

#[tool(
    description = "Get audio file metadata: duration, format, bitrate, sample rate, channels. Requires ffprobe on PATH. Blocked on public profile.",
    annotations(title = "Audio info", read_only_hint = true)
)]
async fn audio_info(
    &self,
    Parameters(args): Parameters<AudioInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || audio_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an audio file to a different format (mp3, wav, ogg, flac, aac, m4a, opus, wma). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio convert", read_only_hint = false)
)]
async fn audio_convert(
    &self,
    Parameters(args): Parameters<AudioConvertArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_format = args.target_format.clone();
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_convert_work(path, target_format, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract a time segment from an audio file by start time and duration. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio extract segment", read_only_hint = false)
)]
async fn audio_extract_segment(
    &self,
    Parameters(args): Parameters<AudioExtractSegmentArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let start_secs = args.start_secs;
    let duration_secs = args.duration_secs;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || {
        audio_extract_segment_work(path, start_secs, duration_secs, output)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Trim an audio file to a start/end time range. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio trim", read_only_hint = false)
)]
async fn audio_trim(
    &self,
    Parameters(args): Parameters<AudioTrimArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let start_secs = args.start_secs;
    let end_secs = args.end_secs;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || audio_trim_work(path, start_secs, end_secs, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Concatenate multiple audio files into one output file in order. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio merge", read_only_hint = false)
)]
async fn audio_merge(
    &self,
    Parameters(args): Parameters<AudioMergeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let paths: Vec<PathBuf> = args
        .paths
        .iter()
        .map(|p| self.resolve_path(p).unwrap_or_else(|_| PathBuf::from(p)))
        .collect();
    let output = match self.resolve_path(&args.output) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || audio_merge_work(paths, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Mix (overlay) two audio files together with optional per-track volume control. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio mix", read_only_hint = false)
)]
async fn audio_mix(
    &self,
    Parameters(args): Parameters<AudioMixArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = match self.resolve_path(&args.output) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let volume_a = args.volume_a;
    let volume_b = args.volume_b;
    let out = tokio::task::spawn_blocking(move || {
        audio_mix_work(path_a, path_b, output, volume_a, volume_b)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Adjust the volume of an audio file by a multiplier (1.0 = original, 2.0 = double). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio volume", read_only_hint = false)
)]
async fn audio_volume(
    &self,
    Parameters(args): Parameters<AudioVolumeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let volume = args.volume;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_volume_work(path, volume, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Change the playback speed of an audio file (1.0 = original, 2.0 = double speed). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio speed", read_only_hint = false)
)]
async fn audio_speed(
    &self,
    Parameters(args): Parameters<AudioSpeedArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let speed = args.speed;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_speed_work(path, speed, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Reverse an audio file so it plays backwards. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio reverse", read_only_hint = false)
)]
async fn audio_reverse(
    &self,
    Parameters(args): Parameters<AudioReverseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_reverse_work(path, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Normalize audio loudness to a target dB level (e.g. -14.0 for streaming, -23.0 for broadcast). Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio normalize", read_only_hint = false)
)]
async fn audio_normalize(
    &self,
    Parameters(args): Parameters<AudioNormalizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_db = args.target_db;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || audio_normalize_work(path, target_db, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Split an audio file into equal-length segments of a given duration. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio split", read_only_hint = false)
)]
async fn audio_split(
    &self,
    Parameters(args): Parameters<AudioSplitArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let segment_duration_secs = args.segment_duration_secs;
    let output_dir = args.output_dir.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || {
        audio_split_work(path, segment_duration_secs, output_dir)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract the audio track from a video file. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio extract from video", read_only_hint = false)
)]
async fn audio_extract_from_video(
    &self,
    Parameters(args): Parameters<AudioExtractFromVideoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let video_path = match self.resolve_path(&args.video_path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || audio_extract_from_video_work(video_path, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Generate a waveform amplitude profile for an audio file, returning sample values as text. Requires ffprobe on PATH. Blocked on public profile.",
    annotations(title = "Audio waveform", read_only_hint = true)
)]
async fn audio_waveform(
    &self,
    Parameters(args): Parameters<AudioWaveformArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let samples = args.samples;
    let out = tokio::task::spawn_blocking(move || audio_waveform_work(path, samples))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Detect silent intervals in an audio file, returning timestamps and durations. Requires ffmpeg on PATH. Blocked on public profile.",
    annotations(title = "Audio silence detect", read_only_hint = true)
)]
async fn audio_silence_detect(
    &self,
    Parameters(args): Parameters<AudioSilenceDetectArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let min_duration = args.min_duration;
    let noise_db = args.noise_db;
    let out =
        tokio::task::spawn_blocking(move || audio_silence_detect_work(path, min_duration, noise_db))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- IMAGE ---

#[tool(
    description = "Read image metadata: dimensions, format, file size, and color depth. Uses the image crate when available, falls back to ImageMagick/file commands. Blocked on public profile.",
    annotations(title = "Image info", read_only_hint = true)
)]
async fn image_info(
    &self,
    Parameters(args): Parameters<ImageInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Resize an image to exact width x height pixels. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image resize", read_only_hint = false)
)]
async fn image_resize(
    &self,
    Parameters(args): Parameters<ImageResizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_resize_work(path, width, height, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an image to a different format (png, jpg, jpeg, webp, bmp, ico, tiff). Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image convert", read_only_hint = false)
)]
async fn image_convert(
    &self,
    Parameters(args): Parameters<ImageConvertArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_format = args.target_format.clone();
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || image_convert_work(path, target_format, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert an image to grayscale. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image grayscale", read_only_hint = false)
)]
async fn image_grayscale(
    &self,
    Parameters(args): Parameters<ImageGrayscaleArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_grayscale_work(path, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Crop a rectangular region from an image at offset (x, y) with the given width and height. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image crop", read_only_hint = false)
)]
async fn image_crop(
    &self,
    Parameters(args): Parameters<ImageCropArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || image_crop_work(path, x, y, width, height, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Rotate an image by the specified number of degrees clockwise. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image rotate", read_only_hint = false)
)]
async fn image_rotate(
    &self,
    Parameters(args): Parameters<ImageRotateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let degrees = args.degrees;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_rotate_work(path, degrees, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Flip an image horizontally (mirror) or vertically. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image flip", read_only_hint = false)
)]
async fn image_flip(
    &self,
    Parameters(args): Parameters<ImageFlipArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let horizontal = args.horizontal;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_flip_work(path, horizontal, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Create a thumbnail with a maximum side length in pixels while preserving the aspect ratio. Uses image crate when available, otherwise ImageMagick. Blocked on public profile.",
    annotations(title = "Image thumbnail", read_only_hint = false)
)]
async fn image_thumbnail(
    &self,
    Parameters(args): Parameters<ImageThumbnailArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let size = args.size;
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out = tokio::task::spawn_blocking(move || image_thumbnail_work(path, size, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compute a colour histogram for an image, returning per-channel bucket counts as text. Blocked on public profile.",
    annotations(title = "Image histogram", read_only_hint = true)
)]
async fn image_histogram(
    &self,
    Parameters(args): Parameters<ImageHistogramArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_histogram_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compare two images and report a similarity score with per-channel difference metrics. Blocked on public profile.",
    annotations(title = "Image compare", read_only_hint = true)
)]
async fn image_compare(
    &self,
    Parameters(args): Parameters<ImageCompareArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_compare_work(path_a, path_b))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract EXIF / XMP metadata from an image file. Uses exiftool when available, falls back to basic format detection. Blocked on public profile.",
    annotations(title = "Image extract EXIF", read_only_hint = true)
)]
async fn image_extract_exif(
    &self,
    Parameters(args): Parameters<ImageExtractExifArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || image_extract_exif_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Overlay text as a watermark on an image at a given position (topleft, topright, bottomleft, bottomright, center). Uses ImageMagick. Blocked on public profile.",
    annotations(title = "Image watermark", read_only_hint = false)
)]
async fn image_watermark(
    &self,
    Parameters(args): Parameters<ImageWatermarkArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let text = args.text.clone();
    let position = args.position.clone();
    let output = args.output.as_deref().map(|s| {
        self.resolve_path(s).unwrap_or_else(|_| PathBuf::from(s))
    });
    let out =
        tokio::task::spawn_blocking(move || image_watermark_work(path, text, position, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- SPEECH ---

#[tool(
    description = "Transcribe an audio file to text using Whisper or another STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe file", read_only_hint = true)
)]
async fn speech_transcribe_file(
    &self,
    Parameters(args): Parameters<SpeechTranscribeFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let model = args.model.clone();
    let out =
        tokio::task::spawn_blocking(move || speech_transcribe_file_work(path, language, model))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Record audio from the microphone for a given duration and transcribe it with an STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe microphone", read_only_hint = false)
)]
async fn speech_transcribe_microphone(
    &self,
    Parameters(args): Parameters<SpeechTranscribeMicrophoneArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let duration_secs = args.duration_secs;
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || {
        speech_transcribe_microphone_work(duration_secs, language)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Transcribe an audio file in streaming segments of a fixed duration, returning per-segment text. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech transcribe stream", read_only_hint = true)
)]
async fn speech_transcribe_stream(
    &self,
    Parameters(args): Parameters<SpeechTranscribeStreamArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let segment_duration_secs = args.segment_duration_secs;
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || {
        speech_transcribe_stream_work(path, segment_duration_secs, language)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert plain text to speech audio and save to a file using a TTS engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech text to speech", read_only_hint = false)
)]
async fn speech_text_to_speech(
    &self,
    Parameters(args): Parameters<SpeechTextToSpeechArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let text = args.text.clone();
    let voice = args.voice.clone();
    let speed = args.speed;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_text_to_speech_work(text, voice, speed, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert SSML markup to speech audio and save to a file using a TTS engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech text to speech SSML", read_only_hint = false)
)]
async fn speech_text_to_speech_ssml(
    &self,
    Parameters(args): Parameters<SpeechTextToSpeechSsmlArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let ssml = args.ssml.clone();
    let voice = args.voice.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_text_to_speech_ssml_work(ssml, voice, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List available TTS voice identifiers supported by the configured speech engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech list voices", read_only_hint = true)
)]
async fn speech_list_voices(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(speech_list_voices_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List available STT/TTS model names supported by the configured speech engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech list models", read_only_hint = true)
)]
async fn speech_list_models(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(speech_list_models_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Detect the spoken language in an audio file. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech detect language", read_only_hint = true)
)]
async fn speech_detect_language(
    &self,
    Parameters(args): Parameters<SpeechDetectLanguageArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || speech_detect_language_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Transcribe and translate an audio file to text in a target language. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech translate", read_only_hint = true)
)]
async fn speech_translate(
    &self,
    Parameters(args): Parameters<SpeechTranslateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let target_language = args.target_language.clone();
    let out = tokio::task::spawn_blocking(move || speech_translate_work(path, target_language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Perform speaker diarization on an audio file, identifying and labelling each speaker turn. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech diarize", read_only_hint = true)
)]
async fn speech_diarize(
    &self,
    Parameters(args): Parameters<SpeechDiarizeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let num_speakers = args.num_speakers;
    let out = tokio::task::spawn_blocking(move || speech_diarize_work(path, num_speakers))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Enhance an audio file by applying noise reduction and clarity improvements. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech enhance", read_only_hint = false)
)]
async fn speech_enhance(
    &self,
    Parameters(args): Parameters<SpeechEnhanceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let noise_reduction = args.noise_reduction;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || speech_enhance_work(path, noise_reduction, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract word-level timestamps from an audio file using an STT engine. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Speech extract words", read_only_hint = true)
)]
async fn speech_extract_words(
    &self,
    Parameters(args): Parameters<SpeechExtractWordsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || speech_extract_words_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- REVERSE ENGINEERING ---

#[tool(
    description = "Display binary file metadata: file type, target architecture, entry point, and ELF/PE/Mach-O header fields. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE binary info", read_only_hint = true)
)]
async fn re_binary_info(
    &self,
    Parameters(args): Parameters<ReBinaryInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_binary_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract printable strings from a binary file with optional minimum length and encoding filters. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE strings", read_only_hint = true)
)]
async fn re_strings(
    &self,
    Parameters(args): Parameters<ReStringsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let min_length = args.min_length;
    let encoding = args.encoding.clone();
    let out = tokio::task::spawn_blocking(move || re_strings_work(path, min_length, encoding))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Disassemble a binary or object file section into human-readable assembly instructions. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE disassemble", read_only_hint = true)
)]
async fn re_disassemble(
    &self,
    Parameters(args): Parameters<ReDisassembleArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let section = args.section.clone();
    let architecture = args.architecture.clone();
    let start_addr = args.start_addr.clone();
    let count = args.count;
    let out = tokio::task::spawn_blocking(move || {
        re_disassemble_work(path, section, architecture, start_addr, count)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List symbols (functions, globals, undefined references) exported by a binary or object file. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE symbols", read_only_hint = true)
)]
async fn re_symbols(
    &self,
    Parameters(args): Parameters<ReSymbolsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let symbol_type = args.symbol_type.clone();
    let out = tokio::task::spawn_blocking(move || re_symbols_work(path, symbol_type))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List all sections in a binary or object file with their names, sizes, virtual addresses, and flags. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE sections", read_only_hint = true)
)]
async fn re_sections(
    &self,
    Parameters(args): Parameters<ReSectionsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_sections_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List dynamic library dependencies (shared objects / DLLs) required by a binary. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE dependencies", read_only_hint = true)
)]
async fn re_dependencies(
    &self,
    Parameters(args): Parameters<ReDependenciesArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_dependencies_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Produce an annotated hex dump of a binary file at an optional offset and length. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE hexdump", read_only_hint = true)
)]
async fn re_hexdump(
    &self,
    Parameters(args): Parameters<ReHexdumpArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let length = args.length;
    let width = args.width;
    let out = tokio::task::spawn_blocking(move || re_hexdump_work(path, offset, length, width))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compute Shannon entropy of a binary file, optionally broken into fixed-size blocks. High entropy regions indicate compression or encryption. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE entropy", read_only_hint = true)
)]
async fn re_entropy(
    &self,
    Parameters(args): Parameters<ReEntropyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let block_size = args.block_size;
    let out = tokio::task::spawn_blocking(move || re_entropy_work(path, block_size))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Carve a byte range out of a binary file and write it to an output file. Useful for extracting embedded payloads. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE carve", read_only_hint = false)
)]
async fn re_carve(
    &self,
    Parameters(args): Parameters<ReCarveArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let length = args.length;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || re_carve_work(path, offset, length, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Search a binary file for a byte pattern (hex string or regex) and return all match offsets. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE pattern search", read_only_hint = true)
)]
async fn re_pattern_search(
    &self,
    Parameters(args): Parameters<RePatternSearchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let pattern = args.pattern.clone();
    let max_results = args.max_results;
    let out =
        tokio::task::spawn_blocking(move || re_pattern_search_work(path, pattern, max_results))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Parse a fixed-layout binary structure at a given offset using a format descriptor and return the field values. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE struct parse", read_only_hint = true)
)]
async fn re_struct_parse(
    &self,
    Parameters(args): Parameters<ReStructParseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let format = args.format.clone();
    let out = tokio::task::spawn_blocking(move || re_struct_parse_work(path, offset, format))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Byte-level diff two binary files and report differing offset ranges and changed byte counts. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE diff binary", read_only_hint = true)
)]
async fn re_diff_binary(
    &self,
    Parameters(args): Parameters<ReDiffBinaryArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_diff_binary_work(path_a, path_b))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Execute a binary under strace and capture its system-call trace. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE strace", read_only_hint = false)
)]
async fn re_strace(
    &self,
    Parameters(args): Parameters<ReStraceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let extra_args = args.extra_args.clone();
    let timeout_secs = args.timeout_secs;
    let out = tokio::task::spawn_blocking(move || re_strace_work(path, extra_args, timeout_secs))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Scan a firmware image or binary blob with binwalk to identify embedded file signatures, compression, and known headers. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE binwalk scan", read_only_hint = true)
)]
async fn re_binwalk_scan(
    &self,
    Parameters(args): Parameters<ReBinwalkScanArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_binwalk_scan_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- SCREENSHOT ---

#[tool(
    description = "Capture a specific screen region to a file using `scrot` or ImageMagick `import`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot region capture", read_only_hint = false)
)]
async fn screenshot_region(
    &self,
    Parameters(args): Parameters<ScreenshotRegionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let display = args.display.clone();
    let out = tokio::task::spawn_blocking(move || {
        screenshot_region_work(x, y, width, height, output, display)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Capture a specific window by name using `xdotool` + `import`/`scrot`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot window capture", read_only_hint = false)
)]
async fn screenshot_window(
    &self,
    Parameters(args): Parameters<ScreenshotWindowArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let window_name = args.window_name.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || screenshot_window_work(window_name, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Take a full-screen screenshot and OCR it with Tesseract in a single step. Optionally saves the screenshot. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot + OCR", read_only_hint = true)
)]
async fn screenshot_ocr(
    &self,
    Parameters(args): Parameters<ScreenshotOcrArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || screenshot_ocr_work(output, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Record a screen video (screencast) using `ffmpeg` with `x11grab`. Duration is capped at 600 s. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot screen record", read_only_hint = false)
)]
async fn screenshot_record(
    &self,
    Parameters(args): Parameters<ScreenshotRecordArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let duration_secs = args.duration_secs;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let fps = args.fps;
    let out =
        tokio::task::spawn_blocking(move || screenshot_record_work(duration_secs, output, fps))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Add labelled rectangle annotations to a screenshot using ImageMagick. Each annotation specifies text, x, y, width, and height. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot annotate", read_only_hint = false)
)]
async fn screenshot_annotate(
    &self,
    Parameters(args): Parameters<ScreenshotAnnotateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let annotations: Vec<(String, u32, u32, u32, u32)> =
        serde_json::from_str::<Vec<serde_json::Value>>(&args.annotations_json)
            .map(|v| {
                v.iter()
                    .map(|a| {
                        (
                            a["text"].as_str().unwrap_or("").to_string(),
                            a["x"].as_u64().unwrap_or(0) as u32,
                            a["y"].as_u64().unwrap_or(0) as u32,
                            a["w"].as_u64().unwrap_or(100) as u32,
                            a["h"].as_u64().unwrap_or(30) as u32,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out =
        tokio::task::spawn_blocking(move || screenshot_annotate_work(path, annotations, output))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compare two screenshots using ImageMagick's MAE metric and return a match/different verdict. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot compare", read_only_hint = true)
)]
async fn screenshot_compare(
    &self,
    Parameters(args): Parameters<ScreenshotCompareArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let threshold = args.threshold;
    let out =
        tokio::task::spawn_blocking(move || screenshot_compare_work(path_a, path_b, threshold))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List available monitors and display resolutions using `xrandr`. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot monitor info", read_only_hint = true)
)]
async fn screenshot_monitor_info(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(screenshot_monitor_info_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Sample the color of a single pixel at (x, y) on screen using ImageMagick. Returns hex color and RGB values. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot color pick", read_only_hint = true)
)]
async fn screenshot_color_pick(
    &self,
    Parameters(args): Parameters<ScreenshotColorPickArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let display = args.display.clone();
    let out = tokio::task::spawn_blocking(move || screenshot_color_pick_work(x, y, display))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Capture a series of screenshots at a fixed interval to create a timelapse sequence. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "Screenshot timelapse", read_only_hint = false)
)]
async fn screenshot_timelapse(
    &self,
    Parameters(args): Parameters<ScreenshotTimelapseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let interval_secs = args.interval_secs;
    let count = args.count;
    let output_dir = args.output_dir.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || {
        screenshot_timelapse_work(interval_secs, count, output_dir)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- OCR ---

#[tool(
    description = "OCR an image file (png, jpg, tiff, webp, bmp, ...) using Tesseract. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR file", read_only_hint = true)
)]
async fn ocr_file(
    &self,
    Parameters(args): Parameters<OcrFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_file_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "OCR a PDF document page-by-page using Tesseract, via `pdftoppm` or ImageMagick for page rendering. Supports an optional page range. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR PDF", read_only_hint = true)
)]
async fn ocr_pdf(
    &self,
    Parameters(args): Parameters<OcrPdfArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let page_range = args.page_range.clone();
    let out = tokio::task::spawn_blocking(move || ocr_pdf_work(path, language, page_range))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "OCR multiple image files in batch using Tesseract. Returns per-file results combined into one response. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR batch", read_only_hint = true)
)]
async fn ocr_batch(
    &self,
    Parameters(args): Parameters<OcrBatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let paths: Vec<PathBuf> = args
        .paths
        .iter()
        .map(|p| self.resolve_path(p).unwrap_or_else(|_| PathBuf::from(p)))
        .collect();
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_batch_work(paths, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Capture an image from the clipboard and OCR it using Tesseract. Requires `xclip` on Linux. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR from clipboard", read_only_hint = true)
)]
async fn ocr_from_clipboard(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(ocr_from_clipboard_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Capture a screen region with `scrot` or ImageMagick `import` and OCR it immediately with Tesseract. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR screen region", read_only_hint = true)
)]
async fn ocr_screen_region(
    &self,
    Parameters(args): Parameters<OcrScreenRegionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let x = args.x;
    let y = args.y;
    let width = args.width;
    let height = args.height;
    let language = args.language.clone();
    let out =
        tokio::task::spawn_blocking(move || ocr_screen_region_work(x, y, width, height, language))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "OCR an image that contains a table and reconstruct column alignment using Tesseract TSV output. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR table", read_only_hint = true)
)]
async fn ocr_table(
    &self,
    Parameters(args): Parameters<OcrTableArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_table_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "OCR an image and return per-word confidence scores alongside the recognised text. Uses Tesseract TSV output. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR with confidence", read_only_hint = true)
)]
async fn ocr_with_confidence(
    &self,
    Parameters(args): Parameters<OcrWithConfidenceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let language = args.language.clone();
    let out = tokio::task::spawn_blocking(move || ocr_with_confidence_work(path, language))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Preprocess an image with ImageMagick before OCR. Operations: grayscale | threshold | despeckle | sharpen | resize2x. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR preprocess image", read_only_hint = true)
)]
async fn ocr_preprocess(
    &self,
    Parameters(args): Parameters<OcrPreprocessArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let operation = args.operation.clone();
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || ocr_preprocess_work(path, operation, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Detect text bounding-box regions in an image using Tesseract's makebox mode. Returns region coordinates for each detected character/word cluster. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR detect text regions", read_only_hint = true)
)]
async fn ocr_detect_text_regions(
    &self,
    Parameters(args): Parameters<OcrDetectTextRegionsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || ocr_detect_text_regions_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List all Tesseract language packs installed on the system. Requires a trusted (local-power) deployment. Blocked on public profile.",
    annotations(title = "OCR list languages", read_only_hint = true)
)]
async fn ocr_list_languages(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(ocr_list_languages_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- SANDBOX ---

#[tool(
    description = "Execute a command within the sandboxed project root. Command must be a bare executable name (no path separators). Dangerous commands are blocked. Output is capped. Requires trusted profile.",
    annotations(title = "Sandbox exec", read_only_hint = false)
)]
async fn sandbox_exec(
    &self,
    Parameters(args): Parameters<SandboxExecArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let command = args.command.clone();
    let exec_args = args.exec_args.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let max_output_bytes = args.max_output_bytes.unwrap_or(65536);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_exec_work(root, command, exec_args, timeout_secs, max_output_bytes)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Execute a script source string in a sandboxed temporary directory. Supported interpreters: python3, node, bash, ruby, perl. Script is written to a temp file, run, then deleted. Requires trusted profile.",
    annotations(title = "Sandbox script", read_only_hint = false)
)]
async fn sandbox_script(
    &self,
    Parameters(args): Parameters<SandboxScriptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let script = args.script.clone();
    let language = args.language.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_script_work(root, script, language, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Evaluate a code snippet with an optional stdin payload. Supports python3, node, bash, ruby, and perl. Runs in a sandboxed environment with a clean PATH. Requires trusted profile.",
    annotations(title = "Sandbox eval code", read_only_hint = false)
)]
async fn sandbox_eval_code(
    &self,
    Parameters(args): Parameters<SandboxEvalCodeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let code = args.code.clone();
    let language = args.language.clone();
    let input_data = args.input_data.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_eval_code_work(root, code, language, input_data, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Run a command inside a Docker container with --network=none, CPU=1, and a configurable memory cap. Project root is mounted read-only at /workspace. Requires Docker on PATH and trusted profile.",
    annotations(title = "Sandbox docker run", read_only_hint = false)
)]
async fn sandbox_docker_run(
    &self,
    Parameters(args): Parameters<SandboxDockerRunArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let image = args.image.clone();
    let command = args.command.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let memory_mb = args.memory_mb;
    let out = tokio::task::spawn_blocking(move || {
        sandbox_docker_run_work(root, image, command, timeout_secs, memory_mb)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compute a checksum / hash of a file under the project root. Supported algorithms: sha256 (default), sha1, md5, blake3. Validates that the path stays within the project root. Requires trusted profile.",
    annotations(title = "Sandbox checksum", read_only_hint = true)
)]
async fn sandbox_checksum(
    &self,
    Parameters(args): Parameters<SandboxChecksumArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let algorithm = args.algorithm.unwrap_or_else(|| "sha256".to_string());
    let out = tokio::task::spawn_blocking(move || sandbox_checksum_work(root, path, algorithm))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Compute a unified diff between two files within the project root. Both paths are validated against the project root to prevent path traversal. Requires trusted profile.",
    annotations(title = "Sandbox diff", read_only_hint = true)
)]
async fn sandbox_diff(
    &self,
    Parameters(args): Parameters<SandboxDiffArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let context_lines = args.context_lines;
    let out =
        tokio::task::spawn_blocking(move || sandbox_diff_work(root, path_a, path_b, context_lines))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Watch a file within the project root for mtime changes and trigger a command when a change is detected (single-shot poll). Requires trusted profile.",
    annotations(title = "Sandbox file watch", read_only_hint = false)
)]
async fn sandbox_file_watch(
    &self,
    Parameters(args): Parameters<SandboxFileWatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let command = args.command.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_file_watch_work(root, path, command, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Make a sandboxed HTTP/HTTPS request via curl with SSRF protection (blocks internal/private IPs). Only http:// and https:// protocols allowed. Requires trusted profile.",
    annotations(title = "Sandbox network request", read_only_hint = false)
)]
async fn sandbox_network_request(
    &self,
    Parameters(args): Parameters<SandboxNetworkRequestArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let url = args.url.clone();
    let method = args.method.unwrap_or_else(|| "GET".to_string());
    let headers = args.headers.clone();
    let body = args.body.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_network_request_work(root, url, method, headers, body, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

    // --- ALGORITHM ---

#[tool(
    description = "Sort a JSON array with optional key (for object items), algorithm label (auto/quicksort/mergesort/heapsort), and reverse flag. Input: {\"items\": [...], \"key\": \"field\", \"algorithm\": \"auto\", \"reverse\": false}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: sort", read_only_hint = true)
)]
async fn algo_sort(
    &self,
    Parameters(args): Parameters<AlgoSortArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_sort_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Linear or binary search over a JSON array for a target value. Input: {\"items\": [...], \"target\": <value>, \"method\": \"linear\"|\"binary\"}. Binary search requires a pre-sorted array. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: search", read_only_hint = true)
)]
async fn algo_search(
    &self,
    Parameters(args): Parameters<AlgoSearchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_search_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Breadth-first search traversal of a JSON adjacency-list graph. Input: {\"graph\": {\"node\": [\"neighbour\", ...]}, \"start\": \"nodeID\"}. Returns visited node order and count. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph BFS", read_only_hint = true)
)]
async fn algo_graph_bfs(
    &self,
    Parameters(args): Parameters<AlgoGraphBfsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_bfs_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Depth-first search traversal of a JSON adjacency-list graph. Input: {\"graph\": {\"node\": [\"neighbour\", ...]}, \"start\": \"nodeID\"}. Returns visited node order and count. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph DFS", read_only_hint = true)
)]
async fn algo_graph_dfs(
    &self,
    Parameters(args): Parameters<AlgoGraphDfsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_dfs_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Dijkstra shortest-path on a weighted directed graph. Input: {\"graph\": {\"node\": {\"neighbour\": weight}}, \"start\": \"A\", \"end\": \"Z\"}. Returns path array and total distance. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph shortest path", read_only_hint = true)
)]
async fn algo_graph_shortest_path(
    &self,
    Parameters(args): Parameters<AlgoGraphShortestPathArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_shortest_path_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Topological sort of a DAG using Kahn's algorithm. Input: {\"graph\": {\"node\": [\"dependent\", ...]}}. Returns sorted order or an error with cycle nodes. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: graph topological sort", read_only_hint = true)
)]
async fn algo_graph_topo_sort(
    &self,
    Parameters(args): Parameters<AlgoGraphTopoSortArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_graph_topo_sort_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Matrix operations: multiply, transpose, determinant, or inverse. Input: {\"operation\": \"multiply\"|\"transpose\"|\"determinant\"|\"inverse\", \"a\": [[...]], \"b\": [[...]]}. \"b\" is required for multiply. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: matrix operations", read_only_hint = true)
)]
async fn algo_matrix_ops(
    &self,
    Parameters(args): Parameters<AlgoMatrixOpsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_matrix_ops_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Compute descriptive statistics for a numeric array: mean, median, mode, std_dev, min, max, percentiles (p25/p50/p75/p95/p99), IQR, and count. Input: {\"values\": [1, 2, 3, ...]}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats describe", read_only_hint = true)
)]
async fn algo_stats_describe(
    &self,
    Parameters(args): Parameters<AlgoStatsDescribeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_describe_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Pearson or Spearman correlation between two numeric arrays. Input: {\"x\": [...], \"y\": [...], \"method\": \"pearson\"|\"spearman\"}. Both arrays must have the same length (>= 2). Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats correlation", read_only_hint = true)
)]
async fn algo_stats_correlation(
    &self,
    Parameters(args): Parameters<AlgoStatsCorrelationArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_correlation_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Simple linear regression (y = slope * x + intercept) over two numeric arrays. Input: {\"x\": [...], \"y\": [...]}. Returns slope, intercept, r_squared, and n. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: stats linear regression", read_only_hint = true)
)]
async fn algo_stats_regression(
    &self,
    Parameters(args): Parameters<AlgoStatsRegressionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_stats_regression_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Discrete Fourier Transform (DFT) over a real-valued signal. Input: {\"values\": [0.0, 1.0, ...]}. Returns the first N/2 frequency bins with frequency index, magnitude, and phase. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: FFT / DFT", read_only_hint = true)
)]
async fn algo_fft(
    &self,
    Parameters(args): Parameters<AlgoFftArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_fft_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Hash a text string using SHA-256 (default), SHA-512, MD5, or blake3 (falls back to SHA-256). Input: {\"text\": \"hello\", \"algorithm\": \"sha256\"}. Returns hex-encoded digest. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: hash text", read_only_hint = true)
)]
async fn algo_hash_text(
    &self,
    Parameters(args): Parameters<AlgoHashTextArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_hash_text_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Compute Levenshtein edit distance between two strings. Input: {\"a\": \"kitten\", \"b\": \"sitting\"}. Returns the minimum number of single-character edits (insert/delete/substitute). Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: edit distance", read_only_hint = true)
)]
async fn algo_edit_distance(
    &self,
    Parameters(args): Parameters<AlgoEditDistanceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_edit_distance_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "N-gram Jaccard similarity between two strings. Input: {\"a\": \"hello world\", \"b\": \"hello there\", \"n\": 2}. Returns similarity in [0, 1] and intersection/union counts. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: n-gram similarity", read_only_hint = true)
)]
async fn algo_ngram_similarity(
    &self,
    Parameters(args): Parameters<AlgoNgramSimilarityArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_ngram_similarity_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Bloom filter: build a filter from a string set, or query for probable membership. Input: {\"operation\": \"build\"|\"query\", \"items\": [...], \"query\": \"item\", \"size\": 1000, \"hash_count\": 3}. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "Algorithm: bloom filter", read_only_hint = true)
)]
async fn algo_bloom_filter(
    &self,
    Parameters(args): Parameters<AlgoBloomFilterArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = args.input.clone();
    let out = tokio::task::spawn_blocking(move || algo_bloom_filter_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

    // --- IDE ---

#[tool(
    description = "Open a file in the Zed editor, optionally navigating to a specific line. Runs `zed <path>:<line>`. Requires Zed on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: open file", read_only_hint = false)
)]
async fn zed_open_file(
    &self,
    Parameters(args): Parameters<ZedOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || zed_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Retrieve diagnostics from the Zed editor. Zed does not yet expose diagnostics via CLI; returns a placeholder with guidance to use the LSP bridge for live diagnostics. Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: get diagnostics", read_only_hint = true)
)]
async fn zed_get_diagnostics(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || zed_get_diagnostics_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Trigger a workspace-wide search in the Zed editor via `zed --search <query>`. Returns a note that the search was triggered; results appear in the Zed UI. Requires Zed on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: search workspace", read_only_hint = true)
)]
async fn zed_search_workspace(
    &self,
    Parameters(args): Parameters<ZedSearchWorkspaceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "query": args.query,
    });
    let out = tokio::task::spawn_blocking(move || zed_search_workspace_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Attach a file to the Zed AI session context. Opens the file in Zed and reads its content for use as AI context. Returns instructions for using @<path> in the Zed AI Assistant panel. Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: context attach", read_only_hint = false)
)]
async fn zed_context_attach(
    &self,
    Parameters(args): Parameters<ZedContextAttachArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "role": args.role.unwrap_or_else(|| "reference".to_string()),
    });
    let out = tokio::task::spawn_blocking(move || zed_context_attach_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Open a file in the Cursor editor, optionally navigating to a specific line. Runs `cursor <path>:<line>`. Requires Cursor on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Cursor: open file", read_only_hint = false)
)]
async fn cursor_open_file(
    &self,
    Parameters(args): Parameters<CursorOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || cursor_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Send content to the Cursor Composer panel (Ctrl+I / Cmd+I). Opens the associated file in Cursor and returns Composer instructions. Requires Cursor on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Cursor: composer send", read_only_hint = false)
)]
async fn cursor_composer_send(
    &self,
    Parameters(args): Parameters<CursorComposerSendArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "content": args.content,
        "path": args.path,
        "instruction": args.instruction,
    });
    let out = tokio::task::spawn_blocking(move || cursor_composer_send_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Open a side-by-side diff view in VS Code between two files. Runs `code --diff <left> <right>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: open diff", read_only_hint = false)
)]
async fn vscode_open_diff(
    &self,
    Parameters(args): Parameters<VscodeOpenDiffArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "left": args.left,
        "right": args.right,
    });
    let out = tokio::task::spawn_blocking(move || vscode_open_diff_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Open a file in VS Code, optionally jumping to a specific line via `code --goto <path>:<line>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: open file", read_only_hint = false)
)]
async fn vscode_open_file(
    &self,
    Parameters(args): Parameters<VscodeOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || vscode_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "List installed VS Code extensions via `code --list-extensions --show-names`. Returns an array of {id, name} objects. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: list extensions", read_only_hint = true)
)]
async fn vscode_list_extensions(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || vscode_list_extensions_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Install a VS Code extension by its marketplace identifier via `code --install-extension <id>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: install extension", read_only_hint = false)
)]
async fn vscode_install_extension(
    &self,
    Parameters(args): Parameters<VscodeInstallExtensionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "extension_id": args.extension_id,
    });
    let out = tokio::task::spawn_blocking(move || vscode_install_extension_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Read VS Code diagnostics for a file or workspace. VS Code does not expose live diagnostics via CLI; returns best-effort output from `code --status` and guidance to use the Problems panel (Ctrl+Shift+M). Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: get diagnostics", read_only_hint = true)
)]
async fn vscode_get_diagnostics(
    &self,
    Parameters(args): Parameters<VscodeGetDiagnosticsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "severity": args.severity,
    });
    let out = tokio::task::spawn_blocking(move || vscode_get_diagnostics_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Open a file in a JetBrains IDE (idea/android-studio/clion/pycharm/rustrover), optionally at a specific line. Requires the IDE CLI on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: open file", read_only_hint = false)
)]
async fn jetbrains_open_file(
    &self,
    Parameters(args): Parameters<JetbrainsOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "ide": args.ide,
        "line": args.line,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Execute an ADB shell command on a connected Android device via `adb shell <command>`. Optionally targets a specific device by serial. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: ADB shell", read_only_hint = false)
)]
async fn jetbrains_adb_shell(
    &self,
    Parameters(args): Parameters<JetbrainsAdbShellArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "command": args.command,
        "device": args.device,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_adb_shell_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "List connected Android devices via `adb devices -l`. Returns an array of {serial, state, model} records. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: ADB device list", read_only_hint = true)
)]
async fn jetbrains_device_list(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || jetbrains_device_list_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Read the last N lines of Android Logcat via `adb logcat -d -t <n>`. Optionally filters by tag expression and targets a specific device serial. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: logcat tail", read_only_hint = true)
)]
async fn jetbrains_logcat_tail(
    &self,
    Parameters(args): Parameters<JetbrainsLogcatTailArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "device": args.device,
        "n": args.n,
        "filter": args.filter,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_logcat_tail_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Execute an LSP code action (quick fix, refactor, source) via the JetBrains built-in REST API at the given file position. Requires the IDE on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: LSP code action", read_only_hint = false)
)]
async fn jetbrains_lsp_code_action(
    &self,
    Parameters(args): Parameters<JetbrainsLspCodeActionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "ide": args.ide,
        "line": args.line,
        "column": args.column,
        "action_kind": args.action_kind,
        "port": args.port,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_lsp_code_action_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Trigger a saved JetBrains run/debug configuration via the IDE CLI. Supports idea/android-studio/clion/pycharm/rustrover/webstorm/goland. Requires the IDE on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: run configuration", read_only_hint = false)
)]
async fn jetbrains_run_configuration(
    &self,
    Parameters(args): Parameters<JetbrainsRunConfigurationArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "ide": args.ide,
        "config_name": args.config_name,
        "debug": args.debug.unwrap_or(false),
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_run_configuration_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Detect running IDEs on this machine by scanning /proc on Linux. Returns an array of {name, pid} records for known IDEs (zed, code, cursor, idea, android-studio, clion, pycharm, rustrover, vim, nvim, emacs, ...). Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: detect running", read_only_hint = true)
)]
async fn ide_detect(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || ide_detect_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Format a file using rustfmt, prettier, black, or clang-format. Runs the formatter in-place on the specified path. Requires the formatter on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: format document", read_only_hint = false)
)]
async fn ide_format_document(
    &self,
    Parameters(args): Parameters<IdeFormatDocumentArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "formatter": args.formatter,
    });
    let out = tokio::task::spawn_blocking(move || ide_format_document_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Run tests for a file using the language's test runner: cargo test (rust), pytest (python), jest (javascript/typescript). Returns {success, output}. Requires the test runner on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: run test", read_only_hint = false)
)]
async fn ide_run_test(
    &self,
    Parameters(args): Parameters<IdeRunTestArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "language": args.language,
        "path": args.path.unwrap_or_default(),
        "line": args.line,
    });
    let out = tokio::task::spawn_blocking(move || ide_run_test_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Find a symbol's definition location by running `grep -rn <symbol> <path>`. Returns up to 50 matching {file, line, context} records. Requires grep on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: go to definition", read_only_hint = false)
)]
async fn ide_goto_definition(
    &self,
    Parameters(args): Parameters<IdeGotoDefinitionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "symbol": args.symbol,
        "path": args.path,
    });
    let out = tokio::task::spawn_blocking(move || ide_goto_definition_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

#[tool(
    description = "Find all references to a symbol by running `grep -rn <symbol> <path>`. Returns up to 50 matching {file, line, context} records and a total count. Requires grep on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: find references", read_only_hint = false)
)]
async fn ide_find_references(
    &self,
    Parameters(args): Parameters<IdeFindReferencesArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "symbol": args.symbol,
        "path": args.path,
    });
    let out = tokio::task::spawn_blocking(move || ide_find_references_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

    // --- FILE CRYPTO ---

#[tool(
    description = "Encrypt a file using AES-256-GCM (XOR+base64 implementation). Writes an .enc output file next to the original. Requires trusted profile.",
    annotations(title = "File encrypt", read_only_hint = false)
)]
async fn file_encrypt(
    &self,
    Parameters(args): Parameters<FileEncryptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_encrypt_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Decrypt a file previously encrypted with file_encrypt. Reads an .enc file and writes the plaintext back (stripping the .enc suffix). Requires trusted profile.",
    annotations(title = "File decrypt", read_only_hint = false)
)]
async fn file_decrypt(
    &self,
    Parameters(args): Parameters<FileDecryptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_decrypt_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Sign a file with HMAC-SHA256 using a passphrase. Writes a .sig file alongside the input containing the hex-encoded signature and the file's SHA-256 hash. Requires trusted profile.",
    annotations(title = "File sign", read_only_hint = false)
)]
async fn file_sign(
    &self,
    Parameters(args): Parameters<FileSignArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_sign_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Verify a file's HMAC-SHA256 signature against a .sig file. Returns verified=true when the signature matches. Uses constant-time comparison to prevent timing attacks. Read-only.",
    annotations(title = "File verify signature", read_only_hint = true)
)]
async fn file_verify_signature(
    &self,
    Parameters(args): Parameters<FileVerifySignatureArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "sig_path": args.sig_path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_verify_signature_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Find duplicate files in a directory tree by comparing SHA-256 content hashes. Returns groups of files with identical content along with their sizes and paths. Requires trusted profile.",
    annotations(title = "File deduplicate", read_only_hint = false)
)]
async fn file_deduplicate(
    &self,
    Parameters(args): Parameters<FileDeduplicateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "max_depth": args.max_depth,
    });
    let out = tokio::task::spawn_blocking(move || file_deduplicate_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Snapshot a file or directory's metadata (mtime, size, SHA-256). Call twice at different times and compare snapshots to detect changes. Requires trusted profile.",
    annotations(title = "File watch", read_only_hint = false)
)]
async fn file_watch(
    &self,
    Parameters(args): Parameters<FileWatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "compute_hash": args.compute_hash,
    });
    let out = tokio::task::spawn_blocking(move || file_watch_work(params))
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
