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
#[allow(dead_code)]
pub mod system;
#[allow(dead_code)]
pub mod web_ext;
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
pub mod neural;
#[allow(dead_code)]
pub mod network_tools;
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
    ai_chain_of_thought_work, ai_classify_work, ai_cluster_texts_work,
    ai_context_pack_smart_work as ai_ml_context_pack_smart_work, ai_debate_work,
    ai_embed_text_work, ai_few_shot_classify_work, ai_fine_tune_prep_work,
    ai_hallucination_check_work, ai_multi_model_compare_work, ai_persona_chat_work,
    ai_rag_build_index_work, ai_rag_delete_work, ai_rag_query_work, ai_rag_update_work,
    ai_rewrite_style_work, ai_self_critique_work, ai_semantic_search_work,
    ai_structured_extract_work, ai_translate_work, ai_verify_factual_work,
    ai_zero_shot_classify_work, ml_attention_visualize_work, ml_benchmark_model_work,
    ml_dataset_validate_work, ml_fine_tune_prep_work, ml_inference_batch_work,
    ml_lora_apply_work, ml_model_info_work, ml_perplexity_score_work, ml_quantize_model_work,
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
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::{
    collections::HashMap,
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
const MAX_GIT_DIFF_STAT_CHARS: usize = 24_000;
const MAX_GIT_LOG_CHARS: usize = 36_000;
const MAX_GIT_SHOW_CHARS: usize = 48_000;
const MAX_GIT_BRANCH_CHARS: usize = 24_000;
const MAX_GIT_BLAME_CHARS: usize = 64_000;
const MAX_TAIL_LINES: usize = 500;
const MAX_STATS_SCAN_BYTES: u64 = 32 * 1024 * 1024;
const MAX_JSON_VALIDATE_BYTES: u64 = 1024 * 1024;
const MAX_BATCH_FILE_INFO: usize = 30;
const MAX_NAME_SEARCH_FILES: usize = 400;
/// Max bytes read/written for `convert_format` when using file paths.
pub const MAX_CONVERT_BYTES: usize = 256 * 1024;
/// Max bytes fed to `terminal_run_stdin` and gzip compress input.
pub const MAX_TERMINAL_STDIN_BYTES: usize = 512 * 1024;
/// Max decompressed size from `decompress_gzip_base64` (mitigate zip bombs).
pub const MAX_GZIP_DECOMPRESSED_BYTES: usize = 2 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Feature-gate helpers (used by workspace_context_work and ai_ops.rs)
// ---------------------------------------------------------------------------

/// Returns true when the `terminal-exec` feature is compiled in.
pub(crate) fn terminal_enabled() -> bool {
    cfg!(feature = "terminal-exec")
}

/// Returns true when the `git-write` feature is compiled in.
pub(crate) fn git_write_enabled() -> bool {
    cfg!(feature = "git-write")
}

/// Returns true when git remote operations are enabled.
pub(crate) fn git_remote_enabled() -> bool {
    cfg!(feature = "git-write")
}

/// Returns true when project test tools (cargo test / pytest) are enabled.
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
        "path: {}\nsize_bytes: {}\nbytes_scanned: {}\nline_estimate: {}\nhas_nul_bytes: {}\npreview_head_utf8_lossy:\n---\n{}{}\n---",
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
                    format!("\n... and {more} more keys")
                } else {
                    String::new()
                }
            )
        }
        serde_json::Value::Array(ref a) => {
            format!("kind: array\nlength: {}", a.len())
        }
        serde_json::Value::String(ref s) => {
            format!("kind: string\nlength_chars: {}", s.chars().count())
        }
        serde_json::Value::Number(ref n) => format!("kind: number\nvalue: {n}"),
        serde_json::Value::Bool(b) => format!("kind: bool\nvalue: {b}"),
        serde_json::Value::Null => "kind: null".to_string(),
    };
    format!("FILE: {}\nOK: valid JSON\n{summary}", path.display())
}

/// Get file information for multiple paths in a single call
pub fn batch_file_info_work(paths: Vec<PathBuf>) -> String {
    if paths.is_empty() {
        return "Error: paths must not be empty".to_string();
    }
    if paths.len() > MAX_BATCH_FILE_INFO {
        return format!("Error: at most {MAX_BATCH_FILE_INFO} paths per call");
    }
    let mut blocks = Vec::new();
    for p in paths {
        blocks.push(file_info_work(p));
    }
    blocks.join("\n\n---\n\n")
}

/// Non-secret `PB_*` / `MINT_*` gate env visibility for agents (paths redacted).
pub fn mcp_env_hints_work() -> String {
    const KEYS: &[&str] = &[
        "PB_NETWORK_PROFILE",
        "MINT_NETWORK_PROFILE",
        "PB_PROJECT_ROOT",
        "MINT_PROJECT_ROOT",
        "PROJECT_ROOT",
        "PB_BUS_READ_ONLY",
        "MINT_BUS_READ_ONLY",
        "PB_BUS_MAX_SENDS_PER_WINDOW",
        "MINT_BUS_MAX_SENDS_PER_WINDOW",
        "PB_BUS_RATE_WINDOW_MS",
        "MINT_BUS_RATE_WINDOW_MS",
        "PB_TERMINAL_ENABLE",
        "MINT_TERMINAL_ENABLE",
        "PB_TERMINAL_ALLOW",
        "MINT_TERMINAL_ALLOW",
        "PB_GIT_WRITE_ENABLE",
        "MINT_GIT_WRITE_ENABLE",
        "PB_GIT_PUSH_ENABLE",
        "MINT_GIT_PUSH_ENABLE",
        "PB_GIT_REMOTE_ENABLE",
        "MINT_GIT_REMOTE_ENABLE",
        "PB_PROJECT_TEST_ENABLE",
        "MINT_PROJECT_TEST_ENABLE",
    ];
    let mut out = String::from(
        "MCP_ENV_HINTS\n(Paths redacted; use workspace_context for resolved project root.)\n\n",
    );
    for k in KEYS {
        match std::env::var(k) {
            Ok(v) => {
                if *k == "PB_PROJECT_ROOT" || *k == "MINT_PROJECT_ROOT" || *k == "PROJECT_ROOT" {
                    out.push_str(&format!("{k}=<<set, len {} chars>>\n", v.len()));
                } else {
                    out.push_str(&format!("{k}={v}\n"));
                }
            }
            Err(_) => out.push_str(&format!("{k}=<unset>\n")),
        }
    }
    out.push_str("\nPB_NETWORK_PROFILE / MINT_NETWORK_PROFILE: unset or local = trusted LAN (default) — full tool surface with PB_* / MINT_* gates. Value public|internet|wan = internet-facing MCP: bus/terminal/git writes/thinking_session I/O/code_index_build/convert file output blocked; use workspace_context section ai_agent_tool_visibility.\n");
    out.push_str("PB_PROJECT_TEST_ENABLE / MINT_PROJECT_TEST_ENABLE: unset/false = project_cargo_test, project_cargo_check, project_pytest blocked even on local profile (extra safety). Set 1/true to allow those tools when PB_TERMINAL_ENABLE / MINT_TERMINAL_ENABLE=1 and cargo/pytest are in PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW.\n");
    out
}

/// One-shot overview for AI agents: resolved root, git, top-level names, bus presence, and hard limits.
pub fn workspace_context_work(
    root: PathBuf,
    network_profile: String,
    server_sleep_cap_ms: u64,
    is_public_profile: bool,
) -> String {
    let base = root.canonicalize().unwrap_or(root);
    let mut out = String::new();
    out.push_str("WORKSPACE_CONTEXT\n\n");
    out.push_str(&format!("project_root: {}\n", base.display()));
    out.push_str(&format!(
        "network_profile: {} (PB_NETWORK_PROFILE / MINT_NETWORK_PROFILE=local|public). public: no bus writes (incl. subagent_task/done), terminal_run/terminal_run_stdin/gh/glab PR CLI, git push/fetch/pull/add/commit/branch-create/worktree-add/checkout/stash, manifest write, code_index_build, thinking_session read/write, workspace_temp_*, screenshot_capture, ocr_tesseract, ollama_list_models, or convert_format output_path.\n",
        network_profile.trim()
    ));
    out.push_str(&format!(
        "server_sleep_cap_ms (this profile): {server_sleep_cap_ms}\n"
    ));

    let git_check = Command::new("git")
        .current_dir(&base)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    match git_check {
        Ok(o) if o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true" => {
            out.push_str("git_repo: yes\n");
            if let Ok(b) = Command::new("git")
                .current_dir(&base)
                .args(["branch", "--show-current"])
                .output()
            {
                let name = String::from_utf8_lossy(&b.stdout).trim().to_string();
                if !name.is_empty() {
                    out.push_str(&format!("git_branch: {name}\n"));
                }
            }
        }
        _ => out.push_str("git_repo: no (or git unavailable)\n"),
    }

    out.push_str("\nTop-level entries (max 40, sorted):\n");
    match fs::read_dir(&base) {
        Ok(rd) => {
            let mut names: Vec<String> = rd
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            let total = names.len();
            for n in names.iter().take(40) {
                out.push_str(&format!("  {n}\n"));
            }
            if total > 40 {
                out.push_str(&format!("  ... and {} more\n", total - 40));
            }
            if total == 0 {
                out.push_str("  (empty)\n");
            }
        }
        Err(e) => out.push_str(&format!("  (could not read: {e})\n")),
    }

    let bus = base.join(".mint-mcp").join("agent-bus.jsonl");
    out.push_str(&format!(
        "\nmulti_agent_bus: {}\n",
        if bus.exists() {
            "present (.mint-mcp/agent-bus.jsonl)"
        } else {
            "not created yet (use agent_send to create)"
        }
    ));

    out.push_str("\n--- Tool limits (plan calls accordingly) ---\n");
    out.push_str(&format!(
        "read_file: reads up to {} bytes; full-file text may truncate at {} chars. Prefer start_line/end_line (1-based) for large files.\n",
        MAX_FILE_READ_BYTES, MAX_RESPONSE_CHARS
    ));
    out.push_str(&format!(
        "grep_code: caps at {} matches per call; output may truncate at {} chars.\n",
        MAX_GREP_MATCHES, MAX_RESPONSE_CHARS
    ));
    out.push_str(&format!(
        "find_files: caps at {} paths per call.\n",
        MAX_FIND_FILES
    ));
    out.push_str(&format!(
        "project_tree: caps at {} entries; max_depth clamped 1–12.\n",
        MAX_TREE_ENTRIES
    ));
    out.push_str(&format!(
        "git_diff: truncates at {} characters.\n",
        MAX_DIFF_CHARS
    ));
    out.push_str(&format!(
        "git_log / git_show / git_branches: output capped (log/show up to ~{} / ~{} chars).\n",
        MAX_GIT_LOG_CHARS, MAX_GIT_SHOW_CHARS
    ));
    out.push_str(&format!(
        "read_file_tail: last up to {} lines; tail window up to {} bytes from EOF.\n",
        MAX_TAIL_LINES, MAX_FILE_READ_BYTES
    ));
    out.push_str(&format!(
        "file_stats: scans up to {} bytes for line estimates; json_validate_file: max {} bytes.\n",
        MAX_STATS_SCAN_BYTES, MAX_JSON_VALIDATE_BYTES
    ));
    out.push_str(&format!(
        "find_files_by_name: max {} paths; batch_file_info: max {} paths per call.\n",
        MAX_NAME_SEARCH_FILES, MAX_BATCH_FILE_INFO
    ));
    out.push_str(&format!(
        "multi-agent bus: bodies truncated at {} chars on send; log may rotate when very large. \
Async: agent_send + inbox/recent with since_ts_ms. Parallel batch: agent_fan_out (DMs), agent_multi_inbox (reads), agent_coordination_snapshot. \
Serial batch: agent_send_pipeline. Sync waits: agent_inbox_wait, agent_topic_wait, ask_agents. \
agent_ping: health + env. Large bus: agent_recent without since_ts_ms uses tail read. \
Env: PB_BUS_READ_ONLY / MINT_BUS_READ_ONLY=1; PB_BUS_MAX_SENDS_PER_WINDOW / MINT_BUS_MAX_SENDS_PER_WINDOW (0=off), PB_BUS_RATE_WINDOW_MS / MINT_BUS_RATE_WINDOW_MS (fan-out/pipeline use N slots).\n",
        crate::bus::MAX_AGENT_MESSAGE_BODY_CHARS
    ));
    out.push_str(&format!(
        "Context shaping: compress_* (+ simplify_whitespace, markdown_links, unified_diff_stats, strip_tags, ws_collapse, double_slash_line_comments, drop_lines_matching_regex, dedent, truncate_middle, lines head/tail…); summarize_* (+ top_terms, identifier_frequency, duplicate_lines, rank_lines_by_keywords, bullets, markdown_toc, json_structure, delimited_header…); text_levenshtein_distance / text_token_jaccard / text_longest_common_substring / text_djb2_hash; compress_gzip_base64 / decompress_gzip_base64 (binary-safe blobs); text_stats; pack_context; convert_format (+ yaml↔json, toml↔json, *_top_level_keys, sha256_hex, wrap_lines, spaces_to_tabs, remove_empty_lines, dedupe_adjacent_lines, slugify…, max {} bytes/file); codebase_*; code_index_build/search/symbol_prefix/duplicate_symbols/path_substring/aggregate_stats (regex optional); gh_pr_* / glab_mr_*; gitkraken_cloud_catalog + coding_agent_toolchains_catalog + hosted_tool_redirect; terminal_program_version; ai_stack_probe / screenshot_capture / ocr_tesseract / ollama_list_models (optional CLIs + gates); workspace_temp_*; mcp_env_hints; server_sleep (optional reason), wait_until_timestamp, wait_for_path (poll jitter); agent waits.\n",
        MAX_CONVERT_BYTES
    ));
    let term_on = terminal_enabled();
    let gitw = git_write_enabled();
    let git_remote = git_remote_enabled();
    let pt = project_test_tools_enabled();
    out.push_str(&format!(
        "Optional gates: terminal_run / terminal_run_stdin / terminal_which need PB_TERMINAL_ENABLE / MINT_TERMINAL_ENABLE=1 and PB_TERMINAL_ALLOW / MINT_TERMINAL_ALLOW (comma list); project_cargo_test / project_cargo_check / project_pytest also need PB_PROJECT_TEST_ENABLE / MINT_PROJECT_TEST_ENABLE=1; git_checkout / git_stash_push need PB_GIT_WRITE_ENABLE / MINT_GIT_WRITE_ENABLE=1; git_fetch / git_pull need PB_GIT_REMOTE_ENABLE / MINT_GIT_REMOTE_ENABLE=1. terminal_enabled={term_on} git_write_enabled={gitw} git_remote_enabled={git_remote} project_test_gate={pt}\n"
    ));
    out.push_str(
        "\nPaths are resolved under project_root; absolute paths must stay inside the root.\n",
    );
    out.push_str("\n--- ai_agent_tool_visibility ---\n");
    if is_public_profile {
        out.push_str("deployment: public_internet\n");
        out.push_str("philosophy: read-biased, minimal side effects. Prefer grep/read_file/compress/summarize/thinking_*/plan_*/reasoning_* (stateless text), agent_coordination_playbook, ai_inference_integration_guide, coding_agent_toolchains_catalog, code_index reads, agent inbox/thread reads, capped waits, terminal_which / terminal_program_version / ai_stack_probe (if terminal enabled), mcp_env_hints, gitkraken_cloud_catalog.\n");
        out.push_str("blocked: agent_send and all bus mutators, terminal_run, terminal_run_stdin, gh_pr_*, glab_mr_*, git writes/remotes, codebase_manifest_write, code_index_build, thinking_session_*, workspace_temp_*, screenshot_capture, ocr_tesseract, ollama_list_models, convert_format with output_path.\n");
    } else {
        out.push_str("deployment: local_network (trusted)\n");
        out.push_str("philosophy: full catalog subject to PB_* / MINT_* gates. terminal_run / terminal_run_stdin may spawn subprocesses that read/write the repo per argv or stdin (allowlist MINT_TERMINAL_ALLOW). Optional OCR/LLM CLIs: ocr_tesseract (tesseract), ollama_list_models; stack probe: ai_stack_probe; gzip helpers compress_gzip_base64 / decompress_gzip_base64 for compact bus/file handoffs. Use agent_coordination_playbook for patterns; agent_fan_out / agent_send_pipeline / agent_subagent_* for parallel vs serial coordination; agent_inbox_wait / agent_topic_wait for sync; since_ts_ms for async polling.\n");
        out.push_str("cross_platform: Rust fs + git CLI + optional gh/glab; PATH resolved via terminal_which (which/where).\n");
    }
    out.push_str("hint: mcp_env_hints — non-secret PB_* / MINT_* gate variables (project root values redacted).\n");
    out
}

/// Hosted GitKraken / GitLens MCP tools that need the official extension + provider auth (not implemented in this server).
pub fn gitkraken_cloud_catalog_work() -> String {
    let mut s = String::from("GITKRAKEN_CLOUD_CATALOG\n\n");
    s.push_str("These tools exist on the GitLens/GitKraken MCP server and require that MCP plus signed-in provider access. This server does not call GitHub/GitLab/Bitbucket/Azure APIs.\n\n");
    s.push_str("Pull requests & reviews:\n");
    s.push_str("  pull_request_create, pull_request_get_detail, pull_request_get_comments, pull_request_create_review, pull_request_assigned_to_me\n\n");
    s.push_str("Issues:\n");
    s.push_str("  issues_get_detail, issues_add_comment, issues_assigned_to_me\n\n");
    s.push_str("GitKraken / GitLens UX:\n");
    s.push_str("  gitkraken_workspace_list, gitlens_launchpad, gitlens_start_work, gitlens_start_review, gitlens_commit_composer\n\n");
    s.push_str("Local-repo parity on this server: git_status, git_log_or_diff, git_branch, git_worktree, git_stash, git_checkout, git_add_or_commit, git_push, git_fetch, git_pull, repository_get_file_content, gk_* aliases (GitKraken arg names).\n\n");
    s.push_str("Optional PR CLI (not a cloud API — runs local `gh` / `glab` like terminal_run): gh_pr_list, gh_pr_view, glab_mr_list, glab_mr_show (need MINT_TERMINAL_ENABLE + gh/glab in MINT_TERMINAL_ALLOW; trusted profile).\n");
    s.push_str("Per-tool hosted redirect: gitkraken_hosted_tool_redirect(tool_name) for agents that expect a named GitKraken cloud tool.\n");
    s.push_str("Coding-agent CLIs / orchestrators (install separately; see coding_agent_toolchains_catalog): openclaw, claude, codex, aider, etc. Probed in ai_stack_probe when allowlisted.\n");
    s
}

/// Read-only map of popular coding-agent CLIs and how they relate to this MCP (no subprocesses here).
pub fn coding_agent_toolchains_catalog_work() -> String {
    let mut s = String::from("CODING_AGENT_TOOLCHAINS_CATALOG\n\n");
    s.push_str("This server stays small and auditable: it does **not** embed Claude/GPT weights or call vendor HTTP APIs. Use the tools below **alongside** optional host CLIs (terminal_run / terminal_run_stdin with MINT_TERMINAL_ENABLE + MINT_TERMINAL_ALLOW) or dedicated vendor MCP servers.\n\n");
    s.push_str("Orchestration / multi-agent:\n");
    s.push_str("  - This MCP: agent_bus (agent_send, agent_fan_out, agent_send_pipeline, agent_subagent_task/done, agent_inbox_wait, agent_topic_wait), plan_*, reasoning_*, thinking_* — serial + parallel patterns in agent_coordination_playbook.\n");
    s.push_str("  - OpenClaw CLI (https://docs.openclaw.ai/cli): gateway/agent automation; pair with this server for repo-scoped file/git tools and durable JSONL bus under .mint-mcp/.\n\n");
    s.push_str("--- OpenClaw CLI recipes (run on trusted LAN; allowlist `openclaw` in MINT_TERMINAL_ALLOW + MINT_TERMINAL_ENABLE=1) ---\n");
    s.push_str("  - One-shot turn (scripting): `openclaw agent --message \"Summarize src/\"` — add `--json` for machine-readable output; `--local` to pin embedded runtime when docs match your install.\n");
    s.push_str("  - Session/thread: reuse `--session-id` across calls; align with this MCP by posting the same correlation string as agent_send `topic` for humans reading both channels.\n");
    s.push_str("  - Pairing with this MCP: use read_file / grep_code / code_index_* here for repo truth; use OpenClaw for provider-backed reasoning; never paste API keys into agent_send (compress_redact_urls first).\n");
    s.push_str("  - After OpenClaw edits: git_diff, project_cargo_check (if gated), then agent_subagent_done back to parent with file list + test status.\n\n");
    s.push_str("--- Claude Code / Anthropic CLI ---\n");
    s.push_str("  - Prefer the editor's Anthropic MCP for inline chat; for headless runs use the host `claude` binary only when allowlisted — same redaction rules as OpenClaw.\n");
    s.push_str("  - Handoff pattern: plan_handoff_brief → agent_subagent_task(task_body=…) → child uses claude CLI or IDE, then agent_subagent_done.\n\n");
    s.push_str("Editor-integrated agents:\n");
    s.push_str("  - Anthropic Claude / Claude Code: use the vendor MCP or host `claude` CLI when installed; keep secrets in the client, not on the bus (compress_redact_* before agent_send).\n");
    s.push_str("  - Cursor / Zed / VS Code: attach **this** MCP for codebase_index, grep_code, git_*, and bus tools; attach vendor MCPs for completions.\n\n");
    s.push_str("CLI coding assistants (examples):\n");
    s.push_str("  - OpenAI Codex CLI (`codex`), Google Gemini CLI (`gemini`), Aider (`aider`) — run in terminal_run with tight timeouts; prefer project root as cwd.\n\n");
    s.push_str("OCR / local ML bridges (this server):\n");
    s.push_str("  - ocr_tesseract, ollama_list_models, ai_stack_probe, ai_inference_integration_guide; screenshot_capture → PNG → OCR on trusted/LAN profiles.\n\n");
    s.push_str("Deterministic helpers for agents (no network):\n");
    s.push_str("  - convert_format (incl. sha256_hex, yaml_to_json*, json_to_yaml, toml_to_json*, json_to_toml, *_top_level_keys), compress_gzip_base64, compress_double_slash_line_comments, summarize_identifier_frequency, text_token_jaccard, text_longest_common_substring, text_levenshtein_distance, text_djb2_hash, summarize_*, compress_*.\n\n");
    s.push_str("Security recap:\n");
    s.push_str("  - MINT_NETWORK_PROFILE=public blocks bus writes, terminal_run, stdin-heavy tools, OCR/ollama, and convert_format output_path — see workspace_context → ai_agent_tool_visibility.\n");
    s
}

/// Cheap UTF-8 stats for prompt budgeting (no tokenization model).
pub fn text_stats_work(text: String) -> String {
    let bytes = text.len();
    let chars = text.chars().count();
    let lines = text.lines().count();
    let non_empty = text.lines().filter(|l| !l.trim().is_empty()).count();
    let para_like = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    let words = text.split_whitespace().filter(|w| !w.is_empty()).count();
    let tok4 = chars / 4;
    let tok13 = chars.saturating_mul(10) / 13;
    format!(
        "TEXT_STATS\nbytes: {bytes}\nchars: {chars}\nlines: {lines}\nnon_empty_lines: {non_empty}\nparagraph_like_blocks: {para_like}\nwhitespace_separated_tokens: {words}\nrough_llm_tokens_est_chars_div_4: {tok4}\nrough_llm_tokens_est_chars_div_1_3: {tok13}\nhint: real tokenizer counts differ; use summarize_token_estimate for the same metrics in a dedicated tool; shrink with compress_* / summarize_* / compress_redact_urls.\n"
    )
}

/// Same heuristics as `text_stats_work` but focused on budget checks before bus/pack_context.
pub fn summarize_token_estimate_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let bytes = text.len();
    let chars = text.chars().count();
    let lines = text.lines().count();
    let words = text.split_whitespace().filter(|w| !w.is_empty()).count();
    let tok4 = chars / 4;
    let tok13 = chars.saturating_mul(10) / 13;
    let s = format!(
        "SUMMARIZE_TOKEN_ESTIMATE\nbytes: {bytes}\nunicode_scalars_approx_chars: {chars}\nlines: {lines}\nwhitespace_words: {words}\nest_tokens_chars_div_4: {tok4}\nest_tokens_chars_div_1_3: {tok13}\n\nnotes:\n- These are deterministic bounds/heuristics, not true BPE counts.\n- For English-like text, div_4 is a common rule-of-thumb; div_1_3 is more aggressive.\n- Pair with compress_fenced_code_blocks, compress_ws_collapse, summarize_line_histogram.\n"
    );
    let (s, _) = truncate_chars(&s, max_chars);
    s
}

/// Replace http(s) URLs (safe for logs pasted into tickets). Does not touch bare paths.
pub fn compress_redact_urls_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let re = match Regex::new(r#"https?://[^\s<>\[\]()"'`,]+"#) {
        Ok(r) => r,
        Err(_) => return "Error: url regex".to_string(),
    };
    let body = re.replace_all(&text, "[url]").to_string();
    let (body, _) = truncate_chars(&body, max_chars);
    format!("COMPRESS_REDACT_URLS max_chars={max_chars}\n\n{body}")
}
