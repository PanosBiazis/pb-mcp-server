#[allow(dead_code)]
pub mod ai_ops;
#[allow(dead_code)]
pub mod audio;
#[allow(dead_code)]
pub mod code_intel;
#[allow(dead_code)]
pub mod convert_ext;
#[allow(dead_code)]
pub mod db;
#[allow(dead_code)]
pub mod debug;
#[allow(dead_code)]
pub mod image;
#[allow(dead_code)]
pub mod notify;
#[allow(dead_code)]
pub mod ocr_tools;
#[allow(dead_code)]
pub mod reverse_eng;
#[allow(dead_code)]
pub mod review;
#[allow(dead_code)]
pub mod sandbox;
#[allow(dead_code)]
pub mod screenshot_tools;
#[allow(dead_code)]
pub mod speech;
/// Stub implementations for tools that are not yet fully implemented.
///
/// This module contains placeholder implementations that either:
/// - Return "Not implemented" messages
/// - Reference actual implementations in other modules
/// - Provide basic functionality for simple operations
///
/// These stubs allow the project to compile while tools are being progressively
/// implemented without breaking existing functionality.
#[allow(dead_code)]
pub mod stubs;
#[allow(dead_code)]
pub mod system;
#[allow(dead_code)]
pub mod web_ext;
// Note: stubs are accessed via stubs::function_name() to avoid shadowing warnings

// ── previously missing modules ──────────────────────────────────────────────
#[allow(dead_code)]
pub mod ai_ml_extended;
#[allow(dead_code)]
pub mod algorithms;
#[allow(dead_code)]
pub mod file_crypto;
#[allow(dead_code)]
pub mod ide_tools;
#[allow(dead_code)]
pub mod logs_charts;
#[allow(dead_code)]
pub mod network_tools;
#[allow(dead_code)]
pub mod neural;
#[allow(dead_code)]
pub mod protection;

#[allow(unused_imports)]
pub use reverse_eng::{
    re_binary_info_work, re_binwalk_scan_work, re_carve_work, re_dependencies_work,
    re_diff_binary_work, re_disassemble_work, re_entropy_work, re_hexdump_work,
    re_pattern_search_work, re_sections_work, re_strace_work, re_strings_work,
    re_struct_parse_work, re_symbols_work,
};

#[allow(unused_imports)]
pub use ai_ops::{
    ai_context_pack_smart_work, ai_ollama_embed_work, ai_ollama_generate_work,
    ai_prompt_template_work, ai_semantic_diff_work,
};

#[allow(unused_imports)]
pub use audio::{
    audio_convert_work, audio_extract_from_video_work, audio_extract_segment_work, audio_info_work,
    audio_merge_work, audio_mix_work, audio_normalize_work, audio_reverse_work,
    audio_silence_detect_work, audio_speed_work, audio_split_work, audio_trim_work,
    audio_volume_work, audio_waveform_work,
};

#[allow(unused_imports)]
pub use code_intel::{
    code_ast_dump_work, code_complexity_score_work, code_dead_code_hints_work,
    code_detect_language_work, code_extract_imports_work,
};

#[allow(unused_imports)]
pub use convert_ext::{
    convert_archive_extract_work, convert_archive_list_work, convert_binary_hexdump_work,
    convert_document_work, convert_encoding_work, convert_pdf_extract_work,
    convert_qr_generate_work,
};

#[allow(unused_imports)]
pub use db::{
    db_json_query_work, db_sqlite_exec_work, db_sqlite_query_work, db_sqlite_schema_work,
};

#[allow(unused_imports)]
pub use debug::{
    debug_failure_cluster_work, debug_replay_run_work, debug_root_cause_report_work,
    debug_trace_run_work,
};

#[allow(unused_imports)]
pub use image::{
    image_compare_work, image_convert_work, image_crop_work, image_extract_exif_work,
    image_flip_work, image_grayscale_work, image_histogram_work, image_info_work,
    image_resize_work, image_rotate_work, image_thumbnail_work, image_watermark_work,
};

#[allow(unused_imports)]
pub use notify::{notify_desktop_work, notify_email_send_work, notify_webhook_work};

#[allow(unused_imports)]
pub use ocr_tools::{
    ocr_batch_work, ocr_detect_text_regions_work, ocr_file_work, ocr_from_clipboard_work,
    ocr_list_languages_work, ocr_pdf_work, ocr_preprocess_work, ocr_screen_region_work,
    ocr_table_work, ocr_with_confidence_work,
};

#[allow(unused_imports)]
pub use review::{
    review_diff_risk_work, review_focused_review_work, review_missing_tests_work,
    review_pr_packet_work, review_rulebook_check_work,
};

#[allow(unused_imports)]
pub use sandbox::{
    sandbox_checksum_work, sandbox_diff_work, sandbox_docker_run_work, sandbox_eval_code_work,
    sandbox_exec_work, sandbox_file_watch_work, sandbox_network_request_work, sandbox_script_work,
};

#[allow(unused_imports)]
pub use speech::{
    speech_detect_language_work, speech_diarize_work, speech_enhance_work,
    speech_extract_words_work, speech_list_models_work, speech_list_voices_work,
    speech_text_to_speech_ssml_work, speech_text_to_speech_work, speech_transcribe_file_work,
    speech_transcribe_microphone_work, speech_transcribe_stream_work, speech_translate_work,
};

#[allow(unused_imports)]
pub use system::{
    system_disk_usage_work, system_env_get_work, system_network_interfaces_work,
    system_process_list_work,
};

#[allow(unused_imports)]
pub use web_ext::{
    web_api_call_work, web_dns_lookup_work, web_download_file_work, web_rss_fetch_work,
    web_screenshot_work, web_search_work, web_tls_inspect_work,
};

#[allow(unused_imports)]
pub use screenshot_tools::{
    screenshot_annotate_work, screenshot_capture_work, screenshot_color_pick_work,
    screenshot_compare_work, screenshot_monitor_info_work, screenshot_ocr_work,
    screenshot_record_work, screenshot_region_work, screenshot_timelapse_work,
    screenshot_window_work,
};

// ── re-exports for newly wired modules ──────────────────────────────────────

#[allow(unused_imports)]
pub use ai_ml_extended::{
    ai_cluster_texts_work, ai_context_pack_smart_work as ai_ml_context_pack_smart_work,
    ai_embed_text_work, ai_few_shot_classify_work, ai_multi_model_compare_work,
    ai_rewrite_style_work, ai_semantic_search_work, ai_structured_extract_work,
    ai_verify_factual_work, ai_zero_shot_classify_work, ml_benchmark_model_work,
    ml_dataset_validate_work, ml_fine_tune_prep_work, ml_model_info_work, ml_perplexity_score_work,
    ml_token_count_work, ml_tokenize_work,
};

#[allow(unused_imports)]
pub use algorithms::{
    algo_bloom_filter_work, algo_edit_distance_work, algo_fft_work, algo_graph_bfs_work,
    algo_graph_dfs_work, algo_graph_shortest_path_work, algo_graph_topo_sort_work,
    algo_hash_text_work, algo_matrix_ops_work, algo_ngram_similarity_work, algo_search_work,
    algo_sort_work, algo_stats_correlation_work, algo_stats_describe_work,
    algo_stats_regression_work,
};

// Fix: use correct function names that actually exist in file_crypto.rs
#[allow(unused_imports)]
pub use file_crypto::{
    file_decrypt_work, file_deduplicate_work, file_encrypt_work, file_sign_work,
    file_verify_signature_work, file_watch_work,
};

#[allow(unused_imports)]
pub use ide_tools::{
    cursor_composer_send_work, cursor_open_file_work, ide_detect_work, ide_find_references_work,
    ide_format_document_work, ide_goto_definition_work, ide_run_test_work,
    jetbrains_adb_shell_work, jetbrains_device_list_work, jetbrains_logcat_tail_work,
    jetbrains_lsp_code_action_work, jetbrains_open_file_work, jetbrains_run_configuration_work,
    vscode_get_diagnostics_work, vscode_install_extension_work, vscode_list_extensions_work,
    vscode_open_diff_work, vscode_open_file_work, zed_context_attach_work,
    zed_get_diagnostics_work, zed_open_file_work, zed_search_workspace_work,
};

#[allow(unused_imports)]
pub use logs_charts::*;

#[allow(unused_imports)]
pub use neural::*;

#[allow(unused_imports)]
pub use network_tools::*;

#[allow(unused_imports)]
pub use protection::*;

use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::RegexBuilder;
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

/// Maximum bytes to read from a single file (512KB)
pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024;
/// Maximum characters in a response before truncation (20,000)
pub const MAX_RESPONSE_CHARS: usize = 20_000;
const MAX_DIR_ENTRIES: usize = 200;
const MAX_GREP_FILE_BYTES: u64 = 384 * 1024;
const MAX_GREP_MATCHES: usize = 200;
const MAX_FIND_FILES: usize = 600;
const MAX_TREE_ENTRIES: usize = 500;
const MAX_DIFF_CHARS: usize = 48_000;
#[allow(dead_code)]
const MAX_GIT_DIFF_STAT_CHARS: usize = 24_000;
const MAX_GIT_LOG_CHARS: usize = 36_000;
const MAX_GIT_SHOW_CHARS: usize = 48_000;
const MAX_GIT_BRANCH_CHARS: usize = 24_000;
const MAX_GIT_BLAME_CHARS: usize = 64_000;
const MAX_TAIL_LINES: usize = 500;
const MAX_STATS_SCAN_BYTES: u64 = 32 * 1024 * 1024;
const MAX_JSON_VALIDATE_BYTES: u64 = 1024 * 1024;
#[allow(dead_code)]
const MAX_BATCH_FILE_INFO: usize = 30;
const MAX_NAME_SEARCH_FILES: usize = 400;
/// Max bytes read/written for `convert_format` when using file paths.
#[allow(dead_code)]
pub const MAX_CONVERT_BYTES: usize = 256 * 1024;
/// Max bytes fed to `terminal_run_stdin` and gzip compress input.
pub const MAX_TERMINAL_STDIN_BYTES: usize = 512 * 1024;
/// Max decompressed size from `decompress_gzip_base64` (mitigate zip bombs).
#[allow(dead_code)]
pub const MAX_GZIP_DECOMPRESSED_BYTES: usize = 2 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Async wrapper stubs for tools_wired.rs (agent sync tools)
// ---------------------------------------------------------------------------

/// Thin async wrapper called by `agent_sync_all_tool`.
/// A live sync requires a running AgentRegistry; this stub reports the current state.
pub async fn agent_sync_all(_root: std::path::PathBuf) -> Result<String, String> {
    Ok(
        "agent_sync_all: no active agents registered in this session. \
         Use agent_send / agent_fan_out / agent_handoff for multi-agent coordination."
            .to_string(),
    )
}

/// Thin async wrapper called by `agent_sync_confirm_tool`.
pub async fn agent_sync_confirm(
    _root: std::path::PathBuf,
    sync_id: String,
) -> Result<String, String> {
    Ok(format!(
        "agent_sync_confirm: sync_id={sync_id} acknowledged (no pending participants)."
    ))
}

// ---------------------------------------------------------------------------
// Feature-gate helpers (used by workspace_context_work and ai_ops.rs)
// ---------------------------------------------------------------------------

/// Returns true when the `terminal-exec` feature is compiled in.
pub(crate) fn terminal_enabled() -> bool {
    cfg!(feature = "terminal-exec")
}

/// Returns true when the `git-write` feature is compiled in.
#[allow(dead_code)]
pub(crate) fn git_write_enabled() -> bool {
    cfg!(feature = "git-write")
}

/// Returns true when git remote operations are enabled.
#[allow(dead_code)]
pub(crate) fn git_remote_enabled() -> bool {
    cfg!(feature = "git-write")
}

/// Returns true when project test tools (cargo test / pytest) are enabled.
#[allow(dead_code)]
pub(crate) fn project_test_tools_enabled() -> bool {
    cfg!(feature = "core-code")
}

/// Pre-flight check for commands that require the `terminal-exec` feature.
/// Returns `Ok(())` when the feature is active, or an error string otherwise.
pub(crate) fn terminal_run_precheck(cmd: &str) -> Result<(), String> {
    if !terminal_enabled() {
        return Err(format!(
            "`{}` requires the `terminal-exec` feature to be enabled",
            cmd
        ));
    }
    Ok(())
}

/// Truncate a string to a maximum character count
///
/// Returns a tuple of (truncated_string, was_truncated)
pub fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }
    let t: String = s.chars().take(max).collect();
    (t, true)
}

/// Return basic metadata for a single file or directory path.
pub fn file_info_work(path: PathBuf) -> String {
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    let kind = if meta.is_file() {
        "file"
    } else if meta.is_dir() {
        "directory"
    } else {
        "other"
    };
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!(
        "path: {}\nkind: {}\nsize_bytes: {}\nmtime_unix: {}",
        path.display(),
        kind,
        size,
        mtime
    )
}

/// Read a file with optional line range and size limits
pub fn read_file_work(path: PathBuf, start_line: Option<usize>, end_line: Option<usize>) -> String {
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    if !meta.is_file() {
        return "Error: path is not a file".to_string();
    }

    let len = meta.len();
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Error opening {}: {e}", path.display()),
    };

    let mut buf = Vec::new();
    if let Err(e) = file.take(MAX_FILE_READ_BYTES).read_to_end(&mut buf) {
        return format!("Error reading {}: {e}", path.display());
    }

    let text = String::from_utf8_lossy(&buf).into_owned();
    let from_disk = len > MAX_FILE_READ_BYTES;

    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let n_lines = lines.len();

    let body = if start_line.is_some() || end_line.is_some() {
        let start = start_line.unwrap_or(1).max(1);
        let end = end_line.unwrap_or(n_lines).min(n_lines).max(1);
        if start > n_lines {
            return format!(
                "FILE: {}\n\nNo lines in range (file has {} lines in loaded chunk; start_line={})",
                path.display(),
                n_lines,
                start
            );
        }
        let end = end.min(n_lines);
        let mut numbered = String::new();
        for (i, line) in lines[start - 1..end].iter().enumerate() {
            let n = start + i;
            numbered.push_str(&format!("{n:6}| {line}"));
        }
        let header = format!(
            "FILE: {} (lines {start}-{end} of {n_lines} in chunk)",
            path.display()
        );
        let (numbered, clipped) = truncate_chars(&numbered, MAX_RESPONSE_CHARS);
        let mut block = format!("{header}\n\n{numbered}");
        if clipped {
            block.push_str(&format!(
                "\n\n[truncated for MCP response: first {MAX_RESPONSE_CHARS} characters]"
            ));
        }
        block
    } else {
        let (text, clipped_chars) = truncate_chars(&text, MAX_RESPONSE_CHARS);
        let mut out = format!("FILE: {}\n\n{}", path.display(), text);
        if clipped_chars {
            out.push_str(&format!(
                "\n\n[truncated for MCP response: first {MAX_RESPONSE_CHARS} characters]"
            ));
        }
        out
    };

    let mut out = body;
    if from_disk {
        out.push_str(&format!(
            "\n\n[truncated on disk: file exceeds {MAX_FILE_READ_BYTES} bytes; only the first chunk is loaded]"
        ));
    }
    out
}

/// List files in a directory with entry limits
pub fn list_files_work(root_display: PathBuf, path: PathBuf) -> String {
    let entries = match fs::read_dir(&path) {
        Ok(rd) => rd,
        Err(e) => return format!("Error reading directory: {e}"),
    };

    let mut lines = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let prefix = match entry.file_type() {
            Ok(ft) if ft.is_dir() => "[D]",
            Ok(ft) if ft.is_file() => "[F]",
            _ => "[?]",
        };
        lines.push(format!("{prefix} {name}"));
    }

    lines.sort();

    let total = lines.len();
    let shown = lines
        .into_iter()
        .take(MAX_DIR_ENTRIES)
        .collect::<Vec<_>>()
        .join("\n");

    let mut body = format!(
        "ROOT: {}\nPATH: {}\n\n{}",
        root_display.display(),
        path.display(),
        shown
    );

    if total > MAX_DIR_ENTRIES {
        body.push_str(&format!(
            "\n\n[truncated: showing {} of {} entries]",
            MAX_DIR_ENTRIES, total
        ));
    }

    body
}

/// Get git status summary for a repository
pub fn git_status_work(dir: PathBuf) -> String {
    let output = Command::new("git")
        .args(["status", "--short", "--branch"])
        .current_dir(&dir)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            let mut body = format!(
                "DIR: {}\nEXIT CODE: {}\n",
                dir.display(),
                out.status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".to_string())
            );

            if !stdout.trim().is_empty() {
                body.push_str("\nSTDOUT:\n");
                body.push_str(&stdout);
            }

            if !stderr.trim().is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&stderr);
            }

            body
        }
        Err(e) => format!("Error running git: {e}"),
    }
}

fn read_text_file_limited(path: &Path) -> Option<String> {
    let f = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    let mut take = f.take(MAX_GREP_FILE_BYTES);
    take.read_to_end(&mut buf).ok()?;
    if buf.contains(&0) {
        return None;
    }
    String::from_utf8(buf).ok()
}

fn normalize_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Search for pattern matches in files
pub fn grep_work(
    search_root: PathBuf,
    pattern: String,
    literal: bool,
    case_sensitive: bool,
    respect_gitignore: bool,
    max_matches: usize,
) -> String {
    let max_matches = max_matches.min(MAX_GREP_MATCHES);
    let re = if literal {
        match RegexBuilder::new(&regex::escape(&pattern))
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Error building search: {e}"),
        }
    } else {
        match RegexBuilder::new(&pattern)
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Invalid regex: {e}"),
        }
    };

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(respect_gitignore);
    walk.ignore(respect_gitignore);
    walk.hidden(false);

    let mut hits = 0usize;
    let mut out = String::new();

    for entry in walk.build().flatten() {
        if hits >= max_matches {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(text) = read_text_file_limited(path) else {
            continue;
        };

        for (i, line) in text.lines().enumerate() {
            if hits >= max_matches {
                break;
            }
            if re.is_match(line) {
                let line_no = i + 1;
                let rel = path
                    .strip_prefix(&search_root)
                    .map(normalize_rel_path)
                    .unwrap_or_else(|_| normalize_rel_path(path));
                out.push_str(&format!("{rel}:{line_no}:{line}\n"));
                hits += 1;
            }
        }
    }

    if hits >= max_matches {
        out.push_str(&format!(
            "\n[stopped at {max_matches} matches; narrow `path` or refine `pattern`]\n"
        ));
    }

    let (out, clipped) = truncate_chars(out.trim_end(), MAX_RESPONSE_CHARS);
    let mut result = format!(
        "SEARCH_ROOT: {}\nMATCHES: {hits}\n\n{out}",
        search_root.display()
    );
    if clipped {
        result.push_str(&format!(
            "\n\n[output truncated to {MAX_RESPONSE_CHARS} characters]"
        ));
    }
    result
}

/// Find files matching a glob pattern
pub fn find_files_work(search_root: PathBuf, glob_pat: String, max_files: usize) -> String {
    let max_files = max_files.min(MAX_FIND_FILES);
    let mut gs = GlobSetBuilder::new();
    let g = match Glob::new(&glob_pat) {
        Ok(g) => g,
        Err(e) => return format!("Invalid glob: {e}"),
    };
    gs.add(g);
    let set = match gs.build() {
        Ok(s) => s,
        Err(e) => return format!("Invalid glob set: {e}"),
    };

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut paths = Vec::new();
    for entry in walk.build().flatten() {
        if paths.len() >= max_files {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = match path.strip_prefix(&search_root) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let rel_str = normalize_rel_path(rel);
        if set.is_match(Path::new(&rel_str)) {
            paths.push(rel_str);
        }
    }

    paths.sort();
    let n = paths.len();
    let listed = paths.join("\n");
    let mut out = format!(
        "SEARCH_ROOT: {}\nGLOB: {glob_pat}\nFILES: {n}\n\n{listed}",
        search_root.display()
    );
    if n >= max_files {
        out.push_str(&format!(
            "\n\n[stopped at {max_files} paths; narrow `glob` or `path`]"
        ));
    }
    out
}

/// Generate a directory tree structure
pub fn tree_work(search_root: PathBuf, max_depth: usize) -> String {
    let max_depth = max_depth.clamp(1, 12);
    let mut lines: Vec<String> = Vec::new();
    let mut count = 0usize;

    fn walk(
        dir: &Path,
        prefix: &str,
        depth: usize,
        max_depth: usize,
        lines: &mut Vec<String>,
        count: &mut usize,
    ) {
        if depth > max_depth || *count >= MAX_TREE_ENTRIES {
            return;
        }
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        let mut kids: Vec<_> = rd.flatten().collect();
        kids.sort_by_key(|e| e.file_name());

        for (i, entry) in kids.iter().enumerate() {
            if *count >= MAX_TREE_ENTRIES {
                break;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let connector = if i + 1 == kids.len() {
                "└── "
            } else {
                "├── "
            };
            let path = entry.path();
            let ft = entry.file_type().ok();
            let tag = match ft {
                Some(t) if t.is_dir() => "[D]",
                Some(t) if t.is_file() => "[F]",
                _ => "[?]",
            };
            lines.push(format!("{prefix}{connector}{tag} {name}"));
            *count += 1;

            if ft.map(|t| t.is_dir()).unwrap_or(false) && depth < max_depth {
                let ext = if i + 1 == kids.len() {
                    "    "
                } else {
                    "│   "
                };
                walk(
                    &path,
                    &format!("{prefix}{ext}"),
                    depth + 1,
                    max_depth,
                    lines,
                    count,
                );
            }
        }
    }

    lines.push(search_root.display().to_string());
    walk(&search_root, "", 1, max_depth, &mut lines, &mut count);

    if count >= MAX_TREE_ENTRIES {
        lines.push(format!(
            "\n[truncated: max {MAX_TREE_ENTRIES} tree entries; lower max_depth or scope `path`]"
        ));
    }
    lines.join("\n")
}

/// Get git diff output for a repository
pub fn git_diff_work(dir: PathBuf, staged: bool) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("diff").arg("--no-color").current_dir(&dir);
    if staged {
        cmd.arg("--staged");
    }
    let output = cmd.output();
    match output {
        Ok(out) => {
            let mut body = format!(
                "DIR: {}\nSTAGED: {staged}\nEXIT: {}\n",
                dir.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let diff = String::from_utf8_lossy(&out.stdout);
            let (diff, clipped) = truncate_chars(diff.trim_end(), MAX_DIFF_CHARS);
            body.push('\n');
            body.push_str(&diff);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[diff truncated to {MAX_DIFF_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git diff: {e}"),
    }
}

/// Reject refs that could break out of a single `git` argument (injection / odd behavior).
pub fn validate_git_refspec(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("rev/ref must not be empty".into());
    }
    if t.len() > 280 {
        return Err("rev/ref too long (max 280 chars)".into());
    }
    if t.chars().any(|c| c.is_whitespace()) {
        return Err("rev/ref must not contain whitespace".into());
    }
    for ch in t.chars() {
        let ok = ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '.' | '/' | '_' | '-' | ':' | '@' | '{' | '}' | '^' | '~'
            );
        if !ok {
            return Err(format!(
                "rev/ref contains disallowed character: {:?} (use e.g. HEAD, main, abc123, v1.0.0, or rev:path)",
                ch
            ));
        }
    }
    Ok(t.to_string())
}

/// Convert an absolute path to a repository-relative path
pub fn path_rel_to_repo(repo: &Path, abs_file: &Path) -> Option<String> {
    abs_file.strip_prefix(repo).ok().map(normalize_rel_path)
}

/// Get git log entries for a repository
pub fn git_log_work(
    repo: PathBuf,
    limit: usize,
    path_in_repo: Option<String>,
    oneline: bool,
) -> String {
    let limit = limit.clamp(1, 150);
    let mut cmd = Command::new("git");
    cmd.current_dir(&repo);
    cmd.args(["log", "--no-color", "-n"]).arg(limit.to_string());
    if oneline {
        cmd.arg("--oneline");
    }
    if let Some(p) = path_in_repo {
        if !p.is_empty() {
            cmd.arg("--");
            cmd.arg(p);
        }
    }
    match cmd.output() {
        Ok(out) => {
            let mut body = format!(
                "REPO: {}\nEXIT: {}\n",
                repo.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (stdout, clipped) = truncate_chars(stdout.trim_end(), MAX_GIT_LOG_CHARS);
            body.push_str(&stdout);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[log truncated to {MAX_GIT_LOG_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git log: {e}"),
    }
}

/// `patch=false` returns `--stat` only (compact). `patch=true` returns unified diff (truncated).
pub fn git_show_work(repo: PathBuf, rev: String, patch: bool) -> String {
    let rev = match validate_git_refspec(&rev) {
        Ok(r) => r,
        Err(e) => return format!("Error: {e}"),
    };
    let mut cmd = Command::new("git");
    cmd.current_dir(&repo);
    cmd.arg("show").arg("--no-color").arg(&rev);
    if !patch {
        cmd.arg("--stat");
    }
    match cmd.output() {
        Ok(out) => {
            let mut body = format!(
                "REPO: {}\nREV: {rev}\nPATCH: {patch}\nEXIT: {}\n",
                repo.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (stdout, clipped) = truncate_chars(stdout.trim_end(), MAX_GIT_SHOW_CHARS);
            body.push_str(&stdout);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[show output truncated to {MAX_GIT_SHOW_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git show: {e}"),
    }
}

/// List all git branches in a repository
pub fn git_branches_work(repo: PathBuf) -> String {
    let output = Command::new("git")
        .current_dir(&repo)
        .args(["branch", "-a", "-v", "--no-abbrev", "--no-color"])
        .output();
    match output {
        Ok(out) => {
            let mut body = format!(
                "REPO: {}\nEXIT: {}\n",
                repo.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (stdout, clipped) = truncate_chars(stdout.trim_end(), MAX_GIT_BRANCH_CHARS);
            body.push_str(&stdout);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[truncated to {MAX_GIT_BRANCH_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git branch: {e}"),
    }
}

/// Get git blame information for file lines
pub fn git_blame_work(
    repo: PathBuf,
    file_rel: String,
    start_line: usize,
    end_line: usize,
    rev: Option<String>,
) -> String {
    let start_line = start_line.max(1);
    let mut end_line = end_line.max(start_line);
    if end_line - start_line + 1 > 400 {
        end_line = start_line + 399;
    }
    let rev_opt = match rev {
        Some(r) => match validate_git_refspec(&r) {
            Ok(r) => Some(r),
            Err(e) => return format!("Error: {e}"),
        },
        None => None,
    };
    let mut cmd = Command::new("git");
    cmd.current_dir(&repo);
    cmd.arg("blame")
        .arg("-b")
        .arg("--no-color")
        .arg("-L")
        .arg(format!("{start_line},{end_line}"));
    if let Some(r) = rev_opt {
        cmd.arg(r);
    }
    cmd.arg("--");
    cmd.arg(&file_rel);
    match cmd.output() {
        Ok(out) => {
            let mut body = format!(
                "REPO: {}\nFILE: {file_rel}\nLINES: {start_line}-{end_line}\nEXIT: {}\n\n",
                repo.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (stdout, clipped) = truncate_chars(stdout.trim_end(), MAX_GIT_BLAME_CHARS);
            body.push_str(&stdout);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[blame truncated to {MAX_GIT_BLAME_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git blame: {e}"),
    }
}

/// Resolve git references to commit hashes
pub fn git_rev_parse_work(repo: PathBuf, refs: Vec<String>) -> String {
    if refs.is_empty() {
        return "Error: refs list must not be empty".to_string();
    }
    if refs.len() > 16 {
        return "Error: at most 16 refs per call".to_string();
    }
    let mut validated: Vec<String> = Vec::new();
    for r in refs {
        match validate_git_refspec(&r) {
            Ok(v) => validated.push(v),
            Err(e) => return format!("Error: {e}"),
        }
    }
    let mut cmd = Command::new("git");
    cmd.current_dir(&repo);
    cmd.arg("rev-parse");
    for v in &validated {
        cmd.arg(v);
    }
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let mut body = format!(
                "REPO: {}\nEXIT: {}\n",
                repo.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            body.push_str("ARGS: ");
            body.push_str(&validated.join(" "));
            body.push_str("\n\n");
            body.push_str(stdout.trim_end());
            if !stderr.trim().is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(stderr.trim_end());
            }
            body
        }
        Err(e) => format!("Error running git rev-parse: {e}"),
    }
}

/// Last lines with line numbers. If the file is larger than `MAX_FILE_READ_BYTES`, only the tail byte window is read (line numbers are then relative to that window).
pub fn read_file_tail_work(path: PathBuf, last_lines: usize) -> String {
    let last_lines = last_lines.clamp(1, MAX_TAIL_LINES);
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    if !meta.is_file() {
        return "Error: path is not a file".to_string();
    }
    let len = meta.len();
    let mut file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Error opening {}: {e}", path.display()),
    };
    let take = len.min(MAX_FILE_READ_BYTES);
    let truncated_on_disk = len > MAX_FILE_READ_BYTES;
    if take == 0 {
        return format!("FILE: {}\n\n(empty file)", path.display());
    }
    if let Err(e) = file.seek(SeekFrom::End(-(take as i64))) {
        return format!("Error seeking {}: {e}", path.display());
    }
    let mut buf = Vec::new();
    if let Err(e) = file.take(take).read_to_end(&mut buf) {
        return format!("Error reading {}: {e}", path.display());
    }
    let text = String::from_utf8_lossy(&buf);
    let partial_first_line = truncated_on_disk && !text.is_empty() && !text.starts_with('\n');
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let n = lines.len();
    let start_idx = n.saturating_sub(last_lines);
    let slice = &lines[start_idx..];
    let mut numbered = String::new();
    if partial_first_line {
        numbered.push_str("(first line in this window may be incomplete — started mid-line)\n\n");
    }
    if truncated_on_disk {
        for (i, line) in slice.iter().enumerate() {
            numbered.push_str(&format!("{:6}| {line}", i + 1));
        }
        let header = format!(
            "FILE: {}\nMODE: tail_window (file larger than {} bytes; line numbers are 1-based within this window only)\nLINES_SHOWN: {}",
            path.display(),
            MAX_FILE_READ_BYTES,
            slice.len()
        );
        let (numbered, clipped) = truncate_chars(&numbered, MAX_RESPONSE_CHARS);
        let mut out = format!("{header}\n\n{numbered}");
        if clipped {
            out.push_str(&format!(
                "\n\n[truncated for MCP response: first {MAX_RESPONSE_CHARS} characters]"
            ));
        }
        return out;
    }
    let first_global = n - slice.len() + 1;
    for (i, line) in slice.iter().enumerate() {
        numbered.push_str(&format!("{:6}| {line}", first_global + i));
    }
    let header = format!(
        "FILE: {} (last {} of {} lines in file)",
        path.display(),
        slice.len(),
        n
    );
    let (numbered, clipped) = truncate_chars(&numbered, MAX_RESPONSE_CHARS);
    let mut out = format!("{header}\n\n{numbered}");
    if clipped {
        out.push_str(&format!(
            "\n\n[truncated for MCP response: first {MAX_RESPONSE_CHARS} characters]"
        ));
    }
    out
}

/// Line-oriented stats without dumping the whole file: newline count in scanned prefix, NUL probe, small preview.
pub fn file_stats_work(path: PathBuf) -> String {
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    if !meta.is_file() {
        return format!(
            "path: {}\nkind: {}\nhint: file_stats is for files; use file_info for metadata.",
            path.display(),
            if meta.is_dir() { "directory" } else { "other" }
        );
    }
    let size = meta.len();
    let scan = size.min(MAX_STATS_SCAN_BYTES);
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Error: {e}"),
    };
    let mut buf = Vec::new();
    if scan > 0 {
        if let Err(e) = file.take(scan).read_to_end(&mut buf) {
            return format!("Error reading file: {e}");
        }
    }
    let has_nul = buf.contains(&0);
    let newline_count = buf.iter().filter(|&&b| b == b'\n').count();
    let complete = scan >= size;
    let line_estimate = if complete {
        let mut lines = newline_count;
        if size > 0 && buf.last() != Some(&b'\n') {
            lines += 1;
        }
        format!("{lines} (full file, {size} bytes)")
    } else {
        format!("{newline_count}+ newlines in first {scan} bytes (file larger; scan capped at {MAX_STATS_SCAN_BYTES})")
    };
    let preview_len = 512.min(buf.len());
    let preview = if preview_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(&buf[..preview_len]).into_owned()
    };
    let preview_note = if has_nul {
        "\n(note: NUL bytes in scan range — likely binary; preview is lossy UTF-8)"
    } else {
        ""
    };
    format!(
        "path: {}\nsize_bytes: {}\nbytes_scanned: {}\nline_estimate: {}\nhas_nul_bytes: {}\npreview_head_utf8_lossy:\n---\n{}{}\\n---",
        path.display(),
        size,
        scan,
        line_estimate,
        has_nul,
        preview,
        preview_note
    )
}

/// Find files by name substring match
pub fn find_files_by_name_work(
    search_root: PathBuf,
    needle: String,
    case_insensitive: bool,
    max_files: usize,
) -> String {
    let max_files = max_files.min(MAX_NAME_SEARCH_FILES);
    let needle = needle.trim();
    if needle.is_empty() {
        return "Error: substring must not be empty".to_string();
    }
    let needle_cmp: String = if case_insensitive {
        needle.to_lowercase()
    } else {
        needle.to_string()
    };
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);
    let mut paths = Vec::new();
    for entry in walk.build().flatten() {
        if paths.len() >= max_files {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = match path.strip_prefix(&search_root) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let rel_str = normalize_rel_path(rel);
        let hay = if case_insensitive {
            rel_str.to_lowercase()
        } else {
            rel_str.clone()
        };
        if hay.contains(&needle_cmp) {
            paths.push(rel_str);
        }
    }
    paths.sort();
    let n = paths.len();
    let listed = paths.join("\n");
    let mut out = format!(
        "SEARCH_ROOT: {}\nSUBSTRING: {}\nCASE_INSENSITIVE: {}\nFILES: {}\n\n{}",
        search_root.display(),
        needle,
        case_insensitive,
        n,
        listed
    );
    if n >= max_files {
        out.push_str(&format!(
            "\n\n[stopped at {max_files} paths; narrow substring or path]"
        ));
    }
    out
}

/// Read and validate a JSON file
pub fn read_json_file_work(path: PathBuf) -> String {
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    if !meta.is_file() {
        return "Error: path is not a file".to_string();
    }
    if meta.len() > MAX_JSON_VALIDATE_BYTES {
        return format!(
            "Error: file exceeds max {} bytes for json_validate",
            MAX_JSON_VALIDATE_BYTES
        );
    }
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Error: {e}"),
    };
    let mut buf = Vec::new();
    if let Err(e) = file.take(MAX_JSON_VALIDATE_BYTES).read_to_end(&mut buf) {
        return format!("Error reading: {e}");
    }
    let v: serde_json::Value = match serde_json::from_slice(&buf) {
        Ok(v) => v,
        Err(e) => return format!("Error: invalid JSON: {e}"),
    };
    let summary = match v {
        serde_json::Value::Object(ref m) => {
            let mut keys: Vec<_> = m.keys().cloned().collect();
            keys.sort();
            let key_preview: Vec<_> = keys.iter().take(40).cloned().collect();
            let more = keys.len().saturating_sub(40);
            format!(
                "kind: object\nkey_count: {}\nkeys_sample: [{}]{}",
                keys.len(),
                key_preview.join(", "),
                if more > 0 {
                    format!("... and {more} more keys")
                } else {
                    String::new()
                }
            )
        }
        serde_json::Value::Array(ref a) => {
            format!("kind: array\nlength: {}", a.len())
        }
        _ => "kind: scalar".to_string(),
    };
    format!("FILE: {}\n\nkind: {}\n", path.display(), summary)
}

// ── re-exports for stub implementations ─────────────────────────────────────
#[allow(unused_imports)]
pub use stubs::{
    agent_coordination_playbook_work, ai_inference_integration_guide_work, ai_stack_probe_work,
    batch_file_info_work, codebase_cargo_dependency_names_work, codebase_duplicate_basenames_work,
    codebase_index_work, codebase_largest_files_work, codebase_manifest_write_work,
    codebase_path_list_work, codebase_shallow_hash_work, codebase_word_index_work,
    coding_agent_toolchains_catalog_work, compress_collapse_blank_lines_work,
    compress_context_work, compress_dedent_work, compress_double_slash_line_comments_work,
    compress_drop_lines_matching_regex_work, compress_fenced_code_blocks_work, compress_file_work,
    compress_gzip_base64_work, compress_lines_head_work, compress_lines_tail_work,
    compress_markdown_frontmatter_work, compress_markdown_links_work, compress_paragraphs_work,
    compress_redact_emails_work, compress_redact_urls_work, compress_sentences_work,
    compress_simplify_whitespace_work, compress_strip_tags_work, compress_truncate_lines_work,
    compress_truncate_middle_work, compress_unified_diff_stats_work, compress_unique_lines_work,
    compress_ws_collapse_work, convert_format_work, decompress_gzip_base64_work,
    extract_snippets_from_file_work, extract_snippets_work, format_prefix_lines_work,
    gh_pr_list_work, gh_pr_view_work, git_add_or_commit_work, git_branch_extension_work,
    git_checkout_work, git_describe_work, git_diff_snippets_work, git_diff_stat_work,
    git_fetch_work, git_last_commit_work, git_log_or_diff_work, git_ls_files_work,
    git_merge_base_work, git_pull_work, git_push_work, git_reflog_work, git_remote_list_work,
    git_revert_work, git_shortlog_work, git_stash_list_work, git_stash_push_work,
    git_tag_list_work, git_worktree_extension_work, git_worktree_list_work,
    gitkraken_cloud_catalog_work, gitkraken_hosted_tool_redirect_work, glab_mr_list_work,
    glab_mr_show_work, grep_snippets_work, mcp_env_hints_work, ocr_tesseract_file_work,
    ollama_list_models_work, pack_context_work, plan_definition_of_done_work,
    plan_execution_checklist_work, plan_handoff_brief_work, plan_retrospective_work,
    plan_risk_register_work, plan_test_strategy_work, plan_work_buckets_work,
    project_cargo_check_build_args, project_cargo_test_build_args, project_pytest_build_args,
    reasoning_chain_work, reasoning_devils_advocate_work, reasoning_evidence_matrix_work,
    reasoning_five_whys_work, reasoning_premortem_work, repository_get_file_content_work,
    summarize_bullets_work, summarize_delimited_header_work, summarize_diff_file_list_work,
    summarize_duplicate_lines_work, summarize_file_work, summarize_first_lines_work,
    summarize_headings_work, summarize_identifier_frequency_work, summarize_json_structure_work,
    summarize_line_histogram_work, summarize_line_prefix_groups_work, summarize_markdown_toc_work,
    summarize_multi_file_work, summarize_rank_lines_by_keywords_work, summarize_segments_work,
    summarize_text_work, summarize_token_estimate_work, summarize_top_terms_work,
    terminal_program_version_work, terminal_which_work, text_djb2_hash_work,
    text_levenshtein_distance_work, text_longest_common_substring_work, text_stats_work,
    text_token_jaccard_work, thinking_assumption_audit_work, thinking_plan_skeleton_work,
    thinking_session_append_work, thinking_session_read_work, thinking_step_back_work,
    workspace_context_work, workspace_temp_delete_work, workspace_temp_list_work,
    workspace_temp_read_work, workspace_temp_write_work,
};
