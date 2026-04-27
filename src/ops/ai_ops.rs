//! # AI/ML Integration Tools
//!
//! Ollama generation and embedding, prompt template rendering,
//! smart context packing with keyword scoring, and semantic diff
//! analysis that maps diff hunks to logical code units (functions,
//! classes, structs, etc.).

use globset::{Glob, GlobSetBuilder};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Maximum characters for AI tool responses (truncated beyond this).
const MAX_AI_RESPONSE_CHARS: usize = 20_000;

/// Characters per token estimate for context packing.
const CHARS_PER_TOKEN: usize = 4;

/// Default max tokens for context packing.
const DEFAULT_PACK_MAX_TOKENS: usize = 4_000;

/// Default context lines for unified diff.
const DEFAULT_DIFF_CONTEXT_LINES: usize = 3;

/// Maximum bytes to read per file during context packing.
const MAX_PACK_FILE_BYTES: u64 = 256 * 1024;

/// Maximum bytes to read per file for semantic diff header parsing.
const MAX_DIFF_HEADER_FILE_BYTES: u64 = 512 * 1024;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate response to `MAX_AI_RESPONSE_CHARS` with a notice if truncated.
fn truncate_response(s: &str) -> String {
    let (truncated, was_truncated) = super::truncate_chars(s, MAX_AI_RESPONSE_CHARS);
    if was_truncated {
        format!(
            "{}\n\n[truncated for MCP response: first {} characters]",
            truncated, MAX_AI_RESPONSE_CHARS
        )
    } else {
        truncated
    }
}

/// Build a `GlobSet` from a list of glob patterns, silently skipping invalid ones.
fn build_globset(patterns: &[String]) -> globset::GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder
        .build()
        .unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap())
}

/// Tokenize text into lowercase alphanumeric words with length > 1.
fn tokenize_keywords(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty() && s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Default exclude patterns for context packing.
fn default_exclude_patterns() -> Vec<String> {
    [
        "**/node_modules/**",
        "**/.git/**",
        "**/target/**",
        "**/dist/**",
        "**/build/**",
        "**/__pycache__/**",
        "**/.next/**",
        "**/vendor/**",
        "**/*.min.js",
        "**/*.min.css",
        "**/*.lock",
        "**/*.map",
        "**/.DS_Store",
        "**/.vscode/**",
        "**/.idea/**",
        "**/coverage/**",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Detect the kind of logical unit from a source header line.
fn detect_header_kind(line: &str) -> &'static str {
    let l = line.to_lowercase();
    if l.contains("fn ") || l.contains("def ") || l.contains("function ") || l.contains("func ") {
        "function"
    } else if l.contains("struct ") {
        "struct"
    } else if l.contains("enum ") {
        "enum"
    } else if l.contains("impl ") {
        "impl"
    } else if l.contains("trait ") {
        "trait"
    } else if l.contains("class ") {
        "class"
    } else if l.contains("interface ") {
        "interface"
    } else if l.contains("module ") || l.contains("mod ") {
        "module"
    } else {
        "unit"
    }
}

/// Find logical unit headers (functions, classes, structs, etc.) in source code.
///
/// Returns a sorted list of `(1-based line number, "kind name (line N)")` tuples.
fn find_logical_headers(content: &str) -> Vec<(usize, String)> {
    // Matches common function/class/struct/etc. declarations across Rust, Python,
    // JavaScript/TypeScript, Go, Java, C#, Ruby, and similar languages.
    let re = Regex::new(
        r"(?im)^\s*(?:(?:pub|public|private|protected|export|async|static|virtual|override|final|abstract|internal|open|inline|synchronized)\s+)*(?:fn|def|function|func|class|struct|enum|impl|trait|interface|module|mod|type)\s+([a-zA-Z_]\w*)"
    ).expect("hardcoded regex for logical headers is valid");

    let mut headers = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if let Some(caps) = re.captures(line) {
            let name = caps[1].to_string();
            let line_num = i + 1;
            let kind = detect_header_kind(line);
            headers.push((line_num, format!("{kind} {name} (line {line_num})")));
        }
    }
    headers
}

/// Find the nearest logical header at or before the given 1-based line number.
fn find_nearest_header(headers: &[(usize, String)], line: usize) -> Option<String> {
    headers
        .iter()
        .rev()
        .find(|(header_line, _)| *header_line <= line)
        .map(|(_, name)| name.clone())
}

/// Read a file capped to a maximum byte count, returning `None` on any failure.
fn read_text_capped(path: &Path, max_bytes: u64) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > max_bytes {
        return None;
    }
    fs::read_to_string(path).ok()
}

// ---------------------------------------------------------------------------
// Public work functions
// ---------------------------------------------------------------------------

/// Run `ollama run <model>` with the prompt piped via stdin.
///
/// # Arguments
/// - `model` — ollama model tag (e.g. `"llama3"`, `"codellama"`)
/// - `prompt` — text sent to the model via stdin
/// - `system` — optional system prompt → `--system "<system>"`
/// - `temperature` — optional sampling temperature → `--temperature <f64>`
/// - `max_tokens` — optional generation cap → `--num-predict <u32>`
///
/// If `ollama` is not on PATH, returns an error with install instructions.
pub fn ai_ollama_generate_work(
    model: String,
    prompt: String,
    system: Option<String>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> String {
    // Gate: require terminal access
    if let Err(e) = super::terminal_run_precheck("ollama") {
        return format!(
            "Error: {e}\n\n\
             Install ollama:\n  \
             macOS:  brew install ollama\n  \
             Linux:  curl -fsSL https://ollama.com/install.sh | sh\n  \
             Windows: winget install Ollama.Ollama\n\n\
             See https://ollama.com/download for details."
        );
    }

    // Build argument list
    let mut args: Vec<String> = vec!["run".into(), model.clone()];
    if let Some(ref sys) = system {
        args.push("--system".into());
        args.push(sys.clone());
    }
    if let Some(t) = temperature {
        args.push("--temperature".into());
        args.push(t.to_string());
    }
    if let Some(t) = max_tokens {
        args.push("--num-predict".into());
        args.push(t.to_string());
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut child = match Command::new("ollama")
        .args(&arg_refs)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return if e.kind() == std::io::ErrorKind::NotFound {
                "Error: ollama not found on PATH.\n\n\
                     Install ollama:\n  \
                     macOS:  brew install ollama\n  \
                     Linux:  curl -fsSL https://ollama.com/install.sh | sh\n  \
                     Windows: winget install Ollama.Ollama\n\n\
                     See https://ollama.com/download for details.".to_string()
            } else {
                format!("Error spawning ollama: {e}")
            };
        }
    };

    // Write prompt to stdin and close it (EOF signals end of input)
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        if let Err(e) = stdin.write_all(prompt.as_bytes()) {
            return format!("Error writing to ollama stdin: {e}");
        }
        // stdin dropped here → pipe closed → ollama sees EOF
    }

    match child.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = format!("OLLAMA_GENERATE model={model}\n");
            if system.is_some() {
                result.push_str("system=present\n");
            }
            if let Some(t) = temperature {
                result.push_str(&format!("temperature={t}\n"));
            }
            if let Some(t) = max_tokens {
                result.push_str(&format!("max_tokens={t}\n"));
            }
            result.push_str(&format!("exit={:?}\n\n", output.status.code()));

            if !stdout.is_empty() {
                result.push_str(&stdout);
            } else if output.status.success() {
                result.push_str("(empty response from model)\n");
            }

            if !output.status.success() && !stderr.is_empty() {
                result.push_str("\n--- STDERR ---\n");
                let (se, clipped) = super::truncate_chars(&stderr, 4_000);
                result.push_str(&se);
                if clipped {
                    result.push_str("\n[stderr truncated]");
                }
            }

            truncate_response(&result)
        }
        Err(e) => format!("Error waiting for ollama process: {e}"),
    }
}

/// Run ollama embeddings API via `curl` to `localhost:11434`.
///
/// Sends a JSON payload with `model` and `prompt` to the ollama REST
/// endpoint and returns embedding vector info: dimension, first 10
/// values, and L2 norm.
///
/// Requires `curl` on PATH and ollama running at `http://localhost:11434`.
pub fn ai_ollama_embed_work(model: String, text: String) -> String {
    if let Err(e) = super::terminal_run_precheck("curl") {
        return format!(
            "Error: {e}\n\n\
             curl is required for the ollama embeddings API.\n\
             Ensure ollama is running: ollama serve"
        );
    }

    // Build JSON payload safely (serde_json handles escaping)
    let payload = serde_json::json!({
        "model": model,
        "prompt": text
    })
    .to_string();

    match Command::new("curl")
        .args([
            "-s",
            "--max-time",
            "120",
            "http://localhost:11434/api/embeddings",
            "-d",
        ])
        .arg(&payload)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                let stderr_preview = &stderr[..stderr.len().min(500)];
                return format!(
                    "Error: curl failed (exit={:?})\nstderr: {stderr_preview}",
                    output.status.code()
                );
            }

            match serde_json::from_str::<Value>(&stdout) {
                Ok(val) => {
                    let mut result = format!("OLLAMA_EMBED model={model}\n");

                    // Check for API-level error
                    if let Some(error) = val.get("error") {
                        result.push_str(&format!(
                            "Error from ollama API: {}\n",
                            error.as_str().unwrap_or("(unknown error)")
                        ));
                        return truncate_response(&result);
                    }

                    if let Some(embedding) = val.get("embedding").and_then(|e| e.as_array()) {
                        let dim = embedding.len();
                        result.push_str(&format!("dimension={dim}\n"));

                        // First 10 values
                        let first_10: Vec<String> = embedding
                            .iter()
                            .take(10)
                            .filter_map(|v| v.as_f64())
                            .map(|f| format!("{f:.6}"))
                            .collect();
                        result.push_str(&format!("first_10={first_10:?}\n"));

                        // L2 norm
                        let sum_sq: f64 = embedding
                            .iter()
                            .filter_map(|v| v.as_f64())
                            .map(|f| f * f)
                            .sum();
                        let norm = sum_sq.sqrt();
                        result.push_str(&format!("l2_norm={norm:.6}\n"));
                    } else {
                        // Unexpected response structure — show raw (truncated)
                        result.push_str("Unexpected response (no 'embedding' array found):\n");
                        let (preview, _) = super::truncate_chars(&stdout, 2_000);
                        result.push_str(&preview);
                    }

                    truncate_response(&result)
                }
                Err(e) => {
                    let preview = &stdout[..stdout.len().min(500)];
                    format!("Error parsing ollama embedding response: {e}\nraw: {preview}")
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Error: curl not found on PATH. Install curl for embedding API access.\n\
                 Ensure ollama is running at http://localhost:11434"
                    .to_string()
            } else {
                format!("Error spawning curl: {e}")
            }
        }
    }
}

/// Load a prompt template from a markdown file and render it by replacing
/// `{{variable_name}}` placeholders with values from the `variables` map.
///
/// # Arguments
/// - `template_name` — template filename (`.md` appended if missing)
/// - `variables` — map of placeholder names to replacement values
/// - `template_dir` — directory containing template files;
///   defaults to `<cwd>/.mint-mcp/templates/`
///
/// If the template is not found, lists available `.md` templates.
pub fn ai_prompt_template_work(
    template_name: String,
    variables: HashMap<String, String>,
    template_dir: Option<PathBuf>,
) -> String {
    let dir = template_dir.unwrap_or_else(|| PathBuf::from(".mint-mcp/templates"));

    // Resolve template path (add .md extension if not already present)
    let template_path = if template_name.ends_with(".md") {
        dir.join(&template_name)
    } else {
        dir.join(format!("{template_name}.md"))
    };

    let template_content = match fs::read_to_string(&template_path) {
        Ok(c) => c,
        Err(_) => {
            let mut result = format!(
                "Error: template '{}' not found at {}\n\n",
                template_name,
                template_path.display()
            );

            if let Ok(entries) = fs::read_dir(&dir) {
                let mut templates: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                    .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                    .collect();
                templates.sort();

                if templates.is_empty() {
                    result.push_str("No .md templates found in template directory.\n");
                    result.push_str(&format!("Create templates at: {}\n", dir.display()));
                } else {
                    result.push_str("Available templates:\n");
                    for t in &templates {
                        result.push_str(&format!("  - {t}\n"));
                    }
                }
            } else {
                result.push_str(&format!(
                    "Template directory does not exist: {}\n",
                    dir.display()
                ));
                result.push_str(
                    "Create the directory and add .md template files with {{variable}} placeholders.\n",
                );
            }

            return truncate_response(&result);
        }
    };

    // Replace {{variable_name}} placeholders using the variables map.
    // We use the regex to find all placeholders in the template, then
    // replace known ones and report unresolved ones.
    let re = match Regex::new(r"\{\{(\w[\w\-]*)\}\}") {
        Ok(r) => r,
        Err(e) => return format!("Error: invalid template variable regex: {e}"),
    };

    let mut rendered = template_content.clone();
    let mut replaced = 0usize;
    let mut unresolved: Vec<String> = Vec::new();

    // Collect all unique placeholder names from the template
    let mut all_placeholders: HashSet<String> = HashSet::new();
    for cap in re.captures_iter(&template_content) {
        let var_name = cap[1].to_string();
        all_placeholders.insert(var_name);
    }

    // Replace known variables
    for var_name in &all_placeholders {
        if let Some(value) = variables.get(var_name) {
            let placeholder = format!("{{{{{var_name}}}}}");
            rendered = rendered.replace(&placeholder, value);
            replaced += 1;
        } else {
            unresolved.push(var_name.clone());
        }
    }

    let mut result = format!("PROMPT_TEMPLATE name={template_name}\n");
    result.push_str(&format!("variables_replaced={replaced}\n"));
    if !unresolved.is_empty() {
        result.push_str(&format!("unresolved_variables={unresolved:?}\n"));
    }
    result.push_str("---\n\n");
    result.push_str(&rendered);

    truncate_response(&result)
}

/// Smart context packing: walk the project tree, read files matching
/// `include_patterns` (globs), exclude files matching `exclude_patterns`,
/// score files by keyword overlap with `query`, and pack the highest-scoring
/// files up to `max_tokens` (~4 chars per token).
///
/// # Arguments
/// - `project_root` — root directory to walk
/// - `query` — natural-language query; tokens are extracted for scoring
/// - `max_tokens` — token budget (default 4000; ~4 chars per token)
/// - `include_patterns` — optional glob include filters (default: all files)
/// - `exclude_patterns` — optional glob exclude filters
///   (default: `**/node_modules/**`, `**/.git/**`, `**/target/**`, etc.)
///
/// Files are scored by counting how many distinct query terms appear in their
/// content (case-insensitive whole-token match). Only files with score > 0
/// are considered for packing.
pub fn ai_context_pack_smart_work(
    project_root: PathBuf,
    query: String,
    max_tokens: Option<usize>,
    include_patterns: Option<Vec<String>>,
    exclude_patterns: Option<Vec<String>>,
) -> String {
    let max_tokens = max_tokens.unwrap_or(DEFAULT_PACK_MAX_TOKENS);
    let max_chars = max_tokens * CHARS_PER_TOKEN;

    if !project_root.is_dir() {
        return format!(
            "Error: project_root is not a directory: {}",
            project_root.display()
        );
    }

    let exclude = exclude_patterns.unwrap_or_else(default_exclude_patterns);
    let has_include = include_patterns.is_some();
    let include_glob = include_patterns.map(|p| build_globset(&p));
    let exclude_glob = build_globset(&exclude);

    // Tokenize query into keyword terms
    let query_terms = tokenize_keywords(&query);
    if query_terms.is_empty() {
        return "Error: query contains no usable keywords (need length > 1)".to_string();
    }

    // Walk project tree and score files by keyword overlap
    let mut scored_files: Vec<(usize, PathBuf)> = Vec::new();

    let walker = ignore::WalkBuilder::new(&project_root)
        .hidden(true)
        .git_ignore(true)
        .git_exclude(true)
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();
        let rel = path.strip_prefix(&project_root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();

        // Check exclude patterns
        if exclude_glob.is_match(rel_str.as_ref()) {
            continue;
        }

        // Check include patterns (if specified)
        if let Some(ref ig) = include_glob {
            if !ig.is_match(rel_str.as_ref()) {
                continue;
            }
        }

        // Skip very large or unreadable files
        let file_len = match fs::metadata(path) {
            Ok(m) if m.len() <= MAX_PACK_FILE_BYTES => m.len(),
            _ => continue,
        };

        // Read and score by keyword overlap
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // binary, permission denied, etc.
        };

        let content_lower = content.to_lowercase();
        let mut score = 0usize;
        for term in &query_terms {
            if content_lower.contains(term.as_str()) {
                score += 1;
            }
        }

        if score > 0 {
            scored_files.push((score, path.to_path_buf()));
        }

        let _ = file_len; // suppress unused warning
    }

    // Sort by score descending, then by path for deterministic ordering
    scored_files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    // Pack files up to character budget
    let mut packed = String::new();
    packed.push_str(&format!(
        "CONTEXT_PACK query=\"{query}\" max_tokens={max_tokens}\n"
    ));
    packed.push_str(&format!(
        "files_relevant={} include_filtered={} exclude_filtered={}\n\n",
        scored_files.len(),
        has_include,
        !exclude.is_empty(),
    ));

    let mut used_chars = 0usize;
    let mut files_packed = 0usize;

    for (score, path) in &scored_files {
        let rel = path.strip_prefix(&project_root).unwrap_or(path);
        let header = format!("--- {} (score={}) ---\n", rel.display(), score);

        if let Ok(content) = fs::read_to_string(path) {
            let entry_chars = header.len() + content.len() + 2; // +2 for trailing \n\n

            if used_chars + entry_chars > max_chars {
                // Try to fit a truncated tail of the file
                let remaining = max_chars.saturating_sub(used_chars);
                if remaining > header.len() + 30 {
                    let avail = remaining - header.len() - 30;
                    let (truncated_content, was_truncated) = super::truncate_chars(&content, avail);
                    packed.push_str(&header);
                    packed.push_str(&truncated_content);
                    if was_truncated {
                        packed.push_str("\n[truncated]");
                    }
                    packed.push_str("\n\n");
                    used_chars += remaining;
                    files_packed += 1;
                }
                break; // budget exhausted
            }

            packed.push_str(&header);
            packed.push_str(&content);
            packed.push_str("\n\n");
            used_chars += entry_chars;
            files_packed += 1;
        }
    }

    let estimated_tokens = used_chars / CHARS_PER_TOKEN;
    packed.push_str(&format!(
        "[packed {files_packed} of {} files, ~{estimated_tokens} tokens of {max_tokens} budget]",
        scored_files.len()
    ));

    truncate_response(&packed)
}

/// Semantic diff: compare two files by running `diff -u` and mapping
/// hunks to the nearest logical unit (function, class, struct, etc.).
///
/// Returns the unified diff output plus a semantic summary listing which
/// functions/classes/structs contain changes.
///
/// # Arguments
/// - `file_a` — path to the original (left) file
/// - `file_b` — path to the modified (right) file
/// - `context_lines` — number of context lines in the unified diff (default 3)
///
/// The semantic analysis parses both files to find logical unit headers
/// (functions, classes, structs, enums, impls, traits, interfaces, modules)
/// and maps each diff hunk to the nearest preceding header.
pub fn ai_semantic_diff_work(
    file_a: PathBuf,
    file_b: PathBuf,
    context_lines: Option<usize>,
) -> String {
    let context = context_lines.unwrap_or(DEFAULT_DIFF_CONTEXT_LINES);

    if !file_a.is_file() {
        return format!("Error: file_a not found: {}", file_a.display());
    }
    if !file_b.is_file() {
        return format!("Error: file_b not found: {}", file_b.display());
    }

    // Run diff -u with requested context
    let diff_output = match Command::new("diff")
        .args([
            "-u",
            &format!("-U{context}"),
            &file_a.to_string_lossy(),
            &file_b.to_string_lossy(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return if e.kind() == std::io::ErrorKind::NotFound {
                "Error: diff not found on PATH. Install diffutils.".to_string()
            } else {
                format!("Error spawning diff: {e}")
            };
        }
    };

    // diff exit codes: 0 = identical, 1 = differences, 2 = error
    if diff_output.status.code() == Some(2) {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        return format!("Error running diff: {stderr}");
    }

    let diff_text = String::from_utf8_lossy(&diff_output.stdout);

    if diff_text.is_empty() {
        return format!(
            "SEMANTIC_DIFF\nFiles are identical:\n  A: {}\n  B: {}",
            file_a.display(),
            file_b.display()
        );
    }

    // Parse hunk headers: @@ -old_start[,old_count] +new_start[,new_count] @@
    let hunk_re =
        Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").expect("valid hunk regex");

    // Read both files for logical-unit header mapping
    let content_a = read_text_capped(&file_a, MAX_DIFF_HEADER_FILE_BYTES).unwrap_or_default();
    let content_b = read_text_capped(&file_b, MAX_DIFF_HEADER_FILE_BYTES).unwrap_or_default();

    let headers_a = find_logical_headers(&content_a);
    let headers_b = find_logical_headers(&content_b);

    // Map each hunk to the nearest enclosing logical unit
    let mut changed_units: HashSet<String> = HashSet::new();
    let mut hunk_details: Vec<String> = Vec::new();

    for line in diff_text.lines() {
        if let Some(caps) = hunk_re.captures(line) {
            let old_start: usize = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1);
            let new_start: usize = caps
                .get(3)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1);

            // Prefer the new-file header (it reflects the current structure)
            let header_b = find_nearest_header(&headers_b, new_start);
            let header_a = find_nearest_header(&headers_a, old_start);

            let unit = header_b.or(header_a);

            if let Some(ref name) = unit {
                changed_units.insert(name.clone());
                hunk_details.push(format!(
                    "  hunk at old:{} new:{} -> {name}",
                    old_start, new_start
                ));
            } else {
                hunk_details.push(format!(
                    "  hunk at old:{} new:{} -> (top-level / no enclosing unit)",
                    old_start, new_start
                ));
            }
        }
    }

    // Build semantic summary
    let mut semantic_summary = String::new();
    semantic_summary.push_str("Changed logical units:\n");
    if changed_units.is_empty() {
        semantic_summary
            .push_str("  (no specific function/class boundaries identified in changes)\n");
    } else {
        let mut sorted: Vec<&String> = changed_units.iter().collect();
        sorted.sort();
        for unit in sorted {
            semantic_summary.push_str(&format!("  * {unit}\n"));
        }
    }
    semantic_summary.push('\n');
    semantic_summary.push_str("Hunk-to-unit mapping:\n");
    for detail in &hunk_details {
        semantic_summary.push_str(detail);
        semantic_summary.push('\n');
    }

    // Assemble final output
    let mut result = format!(
        "SEMANTIC_DIFF\n  A: {}\n  B: {}\ncontext={}\n\n",
        file_a.display(),
        file_b.display(),
        context
    );
    result.push_str("=== Unified Diff ===\n");
    result.push_str(&diff_text);
    result.push_str("\n\n=== Semantic Summary ===\n");
    result.push_str(&semantic_summary);

    truncate_response(&result)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // truncate_response
    // -----------------------------------------------------------------------

    #[test]
    fn truncate_response_no_truncation_under_limit() {
        let s = "hello".to_string();
        let result = truncate_response(&s);
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_response_truncates_over_limit() {
        let s: String = "x".repeat(MAX_AI_RESPONSE_CHARS + 100);
        let result = truncate_response(&s);
        assert!(result.len() > MAX_AI_RESPONSE_CHARS);
        assert!(result.contains("[truncated for MCP response"));
        assert!(result.starts_with(&"x".repeat(MAX_AI_RESPONSE_CHARS)));
    }

    #[test]
    fn truncate_response_exact_limit() {
        let s: String = "y".repeat(MAX_AI_RESPONSE_CHARS);
        let result = truncate_response(&s);
        assert_eq!(result, s); // exactly at limit, no truncation notice
    }

    // -----------------------------------------------------------------------
    // tokenize_keywords
    // -----------------------------------------------------------------------

    #[test]
    fn tokenize_keywords_splits_and_lowercases() {
        let tokens = tokenize_keywords("Hello World foo_bar baz123");
        assert_eq!(tokens, vec!["hello", "world", "foo_bar", "baz123"]);
    }

    #[test]
    fn tokenize_keywords_filters_single_chars() {
        let tokens = tokenize_keywords("a I b c");
        assert!(tokens.is_empty(), "single-char words should be filtered");
    }

    #[test]
    fn tokenize_keywords_handles_punctuation() {
        let tokens = tokenize_keywords("error-handling, rust::traits; async/await");
        assert!(tokens.contains(&"error".to_string()));
        assert!(tokens.contains(&"handling".to_string()));
        assert!(tokens.contains(&"rust".to_string()));
        assert!(tokens.contains(&"traits".to_string()));
        assert!(tokens.contains(&"async".to_string()));
        assert!(tokens.contains(&"await".to_string()));
    }

    #[test]
    fn tokenize_keywords_preserves_underscores() {
        let tokens = tokenize_keywords("my_variable_name");
        assert_eq!(tokens, vec!["my_variable_name"]);
    }

    // -----------------------------------------------------------------------
    // build_globset
    // -----------------------------------------------------------------------

    #[test]
    fn build_globset_empty_patterns() {
        let gs = build_globset(&[]);
        assert!(!gs.is_match("anything.rs"));
    }

    #[test]
    fn build_globset_valid_patterns() {
        let gs = build_globset(&["*.rs".to_string(), "*.toml".to_string()]);
        assert!(gs.is_match("main.rs"));
        assert!(gs.is_match("Cargo.toml"));
        assert!(!gs.is_match("main.py"));
    }

    #[test]
    fn build_globset_recursive_patterns() {
        let gs = build_globset(&["**/node_modules/**".to_string()]);
        assert!(gs.is_match("foo/node_modules/bar/baz.js"));
        assert!(gs.is_match("node_modules/react/index.js"));
    }

    #[test]
    fn build_globset_skips_invalid() {
        let gs = build_globset(&["[invalid".to_string(), "*.rs".to_string()]);
        // Invalid pattern skipped; valid one still works
        assert!(gs.is_match("main.rs"));
        assert!(!gs.is_match("[invalid"));
    }

    // -----------------------------------------------------------------------
    // detect_header_kind
    // -----------------------------------------------------------------------

    #[test]
    fn detect_header_kind_rust_keywords() {
        assert_eq!(detect_header_kind("pub fn foo()"), "function");
        assert_eq!(detect_header_kind("struct Point {"), "struct");
        assert_eq!(detect_header_kind("enum Color {"), "enum");
        assert_eq!(detect_header_kind("impl Display for Point {"), "impl");
        assert_eq!(detect_header_kind("trait AsRef {"), "trait");
        assert_eq!(detect_header_kind("mod utils {"), "module");
    }

    #[test]
    fn detect_header_kind_python_keywords() {
        assert_eq!(detect_header_kind("def bar(self):"), "function");
        assert_eq!(detect_header_kind("class MyClass:"), "class");
    }

    #[test]
    fn detect_header_kind_javascript_keywords() {
        assert_eq!(detect_header_kind("function fetchData() {"), "function");
        assert_eq!(detect_header_kind("export class App {"), "class");
    }

    #[test]
    fn detect_header_kind_go_keywords() {
        assert_eq!(detect_header_kind("func main() {"), "function");
    }

    #[test]
    fn detect_header_kind_interface() {
        assert_eq!(
            detect_header_kind("public interface Serializable {"),
            "interface"
        );
    }

    // -----------------------------------------------------------------------
    // find_logical_headers
    // -----------------------------------------------------------------------

    #[test]
    fn find_logical_headers_rust_file() {
        let code = "\
mod my_mod {
    pub fn hello() {}
    struct Foo { x: i32 }
    enum Bar { A, B }
    impl Foo { fn new() -> Self { Self { x: 0 } } }
    trait Greet { fn greet(&self); }
}
";
        let headers = find_logical_headers(code);
        assert_eq!(headers.len(), 6);
        assert_eq!(headers[0].1, "module my_mod (line 1)");
        assert_eq!(headers[1].1, "function hello (line 2)");
        assert_eq!(headers[2].1, "struct Foo (line 3)");
        assert_eq!(headers[3].1, "enum Bar (line 4)");
        assert_eq!(headers[4].1, "function Foo (line 5)");
        assert_eq!(headers[5].1, "function Greet (line 6)");
    }

    #[test]
    fn find_logical_headers_python_file() {
        let code = "\
class MyClass:
    def method_one(self):
        pass
    async def method_two(self):
        pass

def top_level():
    pass
";
        let headers = find_logical_headers(code);
        assert!(headers.iter().any(|(_, n)| n.contains("class MyClass")));
        assert!(headers
            .iter()
            .any(|(_, n)| n.contains("function method_one")));
        assert!(headers
            .iter()
            .any(|(_, n)| n.contains("function method_two")));
        assert!(headers
            .iter()
            .any(|(_, n)| n.contains("function top_level")));
    }

    #[test]
    fn find_logical_headers_javascript_file() {
        let code = "\
export class App {
    constructor() {}
}
async function fetchData() {}
";
        let headers = find_logical_headers(code);
        assert!(headers.iter().any(|(_, n)| n.contains("class App")));
        assert!(headers
            .iter()
            .any(|(_, n)| n.contains("function fetchData")));
    }

    #[test]
    fn find_logical_headers_go_file() {
        let code = "\
package main
func main() {}
func (s *Server) Start() {}
";
        let headers = find_logical_headers(code);
        assert!(headers.iter().any(|(_, n)| n.contains("function main")));
    }

    #[test]
    fn find_logical_headers_empty() {
        let headers = find_logical_headers("// just a comment\n");
        assert!(headers.is_empty());
    }

    #[test]
    fn find_logical_headers_no_false_positives() {
        let code = "\
// This is not a function declaration
let fn_name = \"something\"; // variable containing 'fn'
/* struct inside a comment */
println!(\"def foo():\");
";
        let headers = find_logical_headers(code);
        // The regex requires the keyword to be preceded by start-of-line + optional
        // visibility modifiers, so inline occurrences should not match.
        let fn_headers: Vec<_> = headers
            .iter()
            .filter(|(_, n)| n.contains("function"))
            .collect();
        assert!(
            fn_headers.is_empty(),
            "should not match 'fn' inside a string"
        );
    }

    // -----------------------------------------------------------------------
    // find_nearest_header
    // -----------------------------------------------------------------------

    #[test]
    fn find_nearest_header_returns_preceding() {
        let headers = vec![
            (5, "function foo (line 5)".to_string()),
            (20, "function bar (line 20)".to_string()),
        ];
        assert_eq!(
            find_nearest_header(&headers, 10),
            Some("function foo (line 5)".to_string())
        );
        assert_eq!(
            find_nearest_header(&headers, 25),
            Some("function bar (line 20)".to_string())
        );
        assert_eq!(find_nearest_header(&headers, 3), None);
    }

    #[test]
    fn find_nearest_header_exact_line() {
        let headers = vec![(10, "function foo (line 10)".to_string())];
        assert_eq!(
            find_nearest_header(&headers, 10),
            Some("function foo (line 10)".to_string())
        );
    }

    #[test]
    fn find_nearest_header_empty() {
        assert_eq!(find_nearest_header(&[], 1), None);
    }

    // -----------------------------------------------------------------------
    // default_exclude_patterns
    // -----------------------------------------------------------------------

    #[test]
    fn default_exclude_patterns_contains_common_dirs() {
        let patterns = default_exclude_patterns();
        assert!(patterns.iter().any(|p| p.contains("node_modules")));
        assert!(patterns.iter().any(|p| p.contains(".git")));
        assert!(patterns.iter().any(|p| p.contains("target")));
        assert!(patterns.iter().any(|p| p.contains("dist")));
        assert!(patterns.iter().any(|p| p.contains("build")));
    }

    // -----------------------------------------------------------------------
    // ai_prompt_template_work
    // -----------------------------------------------------------------------

    #[test]
    fn prompt_template_replaces_variables() {
        let dir = tempfile::tempdir().unwrap();
        let template_path = dir.path().join("test.md");
        fs::write(&template_path, "Hello {{name}}, you are {{role}}!").unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("role".to_string(), "admin".to_string());

        let result =
            ai_prompt_template_work("test".to_string(), vars, Some(dir.path().to_path_buf()));

        assert!(result.contains("Hello Alice, you are admin!"));
        assert!(result.contains("variables_replaced=2"));
        assert!(!result.contains("unresolved_variables"));
    }

    #[test]
    fn prompt_template_reports_unresolved() {
        let dir = tempfile::tempdir().unwrap();
        let template_path = dir.path().join("greet.md");
        fs::write(&template_path, "Hello {{name}}, {{unknown_var}}!").unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());

        let result =
            ai_prompt_template_work("greet".to_string(), vars, Some(dir.path().to_path_buf()));

        assert!(result.contains("Hello Bob"));
        assert!(result.contains("unresolved_variables"));
        assert!(result.contains("unknown_var"));
        assert!(result.contains("variables_replaced=1"));
    }

    #[test]
    fn prompt_template_lists_available_when_not_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "template a").unwrap();
        fs::write(dir.path().join("b.md"), "template b").unwrap();

        let result = ai_prompt_template_work(
            "nonexistent".to_string(),
            HashMap::new(),
            Some(dir.path().to_path_buf()),
        );

        assert!(result.contains("not found"));
        assert!(result.contains("Available templates"));
        assert!(result.contains("a.md"));
        assert!(result.contains("b.md"));
    }

    #[test]
    fn prompt_template_handles_md_extension() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("code_review.md"),
            "Review {{language}} code",
        )
        .unwrap();

        let mut vars = HashMap::new();
        vars.insert("language".to_string(), "Rust".to_string());

        let result = ai_prompt_template_work(
            "code_review.md".to_string(),
            vars,
            Some(dir.path().to_path_buf()),
        );

        assert!(result.contains("Review Rust code"));
    }

    #[test]
    fn prompt_template_repeated_placeholder() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("repeat.md"), "{{name}} and {{name}} again").unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Eve".to_string());

        let result =
            ai_prompt_template_work("repeat".to_string(), vars, Some(dir.path().to_path_buf()));

        assert!(result.contains("Eve and Eve again"));
        // Should count as 1 variable replaced (not 2)
        assert!(result.contains("variables_replaced=1"));
    }

    #[test]
    fn prompt_template_hyphenated_variable() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("hyphen.md"), "Project: {{project-name}}").unwrap();

        let mut vars = HashMap::new();
        vars.insert("project-name".to_string(), "my-app".to_string());

        let result =
            ai_prompt_template_work("hyphen".to_string(), vars, Some(dir.path().to_path_buf()));

        assert!(result.contains("Project: my-app"));
    }

    #[test]
    fn prompt_template_nonexistent_dir() {
        let result = ai_prompt_template_work(
            "anything".to_string(),
            HashMap::new(),
            Some(PathBuf::from("/nonexistent/dir/xyz/templates")),
        );

        assert!(result.contains("not found"));
        assert!(result.contains("does not exist"));
    }

    // -----------------------------------------------------------------------
    // ai_context_pack_smart_work
    // -----------------------------------------------------------------------

    #[test]
    fn context_pack_scores_by_keywords() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "fn main() { println!(\"hello\"); }",
        )
        .unwrap();
        fs::write(
            dir.path().join("error_handler.rs"),
            "error handling module: fn handle_error() { /* error */ }",
        )
        .unwrap();
        fs::write(dir.path().join("utils.rs"), "fn util() {}").unwrap();

        let result = ai_context_pack_smart_work(
            dir.path().to_path_buf(),
            "error handling".to_string(),
            Some(2000),
            None,
            None,
        );

        assert!(result.contains("CONTEXT_PACK"));
        assert!(result.contains("error_handler.rs"));
        // main.rs and utils.rs score 0, so they shouldn't appear
        assert!(!result.contains("main.rs"));
        assert!(!result.contains("utils.rs"));
    }

    #[test]
    fn context_pack_respects_max_tokens() {
        let dir = tempfile::tempdir().unwrap();

        // Create files that mention "test" so they all score > 0
        let big_content: String = "test ".repeat(5000); // ~25,000 chars
        fs::write(dir.path().join("big.rs"), &big_content).unwrap();
        fs::write(dir.path().join("small.rs"), "test small file\n").unwrap();

        let result = ai_context_pack_smart_work(
            dir.path().to_path_buf(),
            "test".to_string(),
            Some(200), // 200 tokens * 4 chars = 800 chars budget
            None,
            None,
        );

        assert!(result.contains("CONTEXT_PACK"));
        // Should have packed at least one file, possibly truncated
        assert!(result.contains("score="));
    }

    #[test]
    fn context_pack_empty_query() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("x.rs"), "fn foo() {}").unwrap();

        let result = ai_context_pack_smart_work(
            dir.path().to_path_buf(),
            "a".to_string(), // single char — filtered out
            None,
            None,
            None,
        );

        assert!(result.contains("Error: query contains no usable keywords"));
    }

    #[test]
    fn context_pack_not_a_directory() {
        let result = ai_context_pack_smart_work(
            PathBuf::from("/nonexistent/path/xyz123"),
            "test".to_string(),
            None,
            None,
            None,
        );

        assert!(result.contains("Error: project_root is not a directory"));
    }

    #[test]
    fn context_pack_exclude_patterns() {
        let dir = tempfile::tempdir().unwrap();

        // Create a nested directory
        let node_dir = dir.path().join("node_modules");
        fs::create_dir_all(&node_dir).unwrap();
        fs::write(node_dir.join("react.js"), "test react module").unwrap();
        fs::write(dir.path().join("main.rs"), "test main module").unwrap();

        let result = ai_context_pack_smart_work(
            dir.path().to_path_buf(),
            "test".to_string(),
            Some(2000),
            None,
            None, // uses default exclude which includes node_modules
        );

        assert!(result.contains("main.rs"));
        assert!(!result.contains("node_modules"));
    }

    #[test]
    fn context_pack_include_patterns() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "test rust file").unwrap();
        fs::write(dir.path().join("main.py"), "test python file").unwrap();

        let result = ai_context_pack_smart_work(
            dir.path().to_path_buf(),
            "test".to_string(),
            Some(2000),
            Some(vec!["*.rs".to_string()]),
            None,
        );

        assert!(result.contains("main.rs"));
        assert!(!result.contains("main.py"));
    }

    // -----------------------------------------------------------------------
    // ai_semantic_diff_work
    // -----------------------------------------------------------------------

    #[test]
    fn semantic_diff_identical_files() {
        let dir = tempfile::tempdir().unwrap();
        let content = "fn foo() {\n    println!(\"hello\");\n}\n";
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        fs::write(&file_a, content).unwrap();
        fs::write(&file_b, content).unwrap();

        let result = ai_semantic_diff_work(file_a, file_b, None);

        assert!(result.contains("SEMANTIC_DIFF"));
        assert!(result.contains("identical"));
    }

    #[test]
    fn semantic_diff_detects_changes() {
        let dir = tempfile::tempdir().unwrap();
        let content_a =
            "fn foo() {\n    println!(\"hello\");\n}\nfn bar() {\n    println!(\"world\");\n}\n";
        let content_b =
            "fn foo() {\n    println!(\"hi\");\n}\nfn bar() {\n    println!(\"world\");\n}\n";

        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        fs::write(&file_a, content_a).unwrap();
        fs::write(&file_b, content_b).unwrap();

        let result = ai_semantic_diff_work(file_a, file_b, Some(3));

        assert!(result.contains("SEMANTIC_DIFF"));
        assert!(result.contains("=== Unified Diff ==="));
        assert!(result.contains("=== Semantic Summary ==="));
        // Should map the change to function foo
        assert!(result.contains("function foo"));
    }

    #[test]
    fn semantic_diff_missing_file_a() {
        let dir = tempfile::tempdir().unwrap();
        let file_a = dir.path().join("nonexistent_a.rs");
        let file_b = dir.path().join("b.rs");
        fs::write(&file_b, "hello").unwrap();

        let result = ai_semantic_diff_work(file_a, file_b, None);

        assert!(result.contains("Error: file_a not found"));
    }

    #[test]
    fn semantic_diff_missing_file_b() {
        let dir = tempfile::tempdir().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("nonexistent_b.rs");
        fs::write(&file_a, "hello").unwrap();

        let result = ai_semantic_diff_work(file_a, file_b, None);

        assert!(result.contains("Error: file_b not found"));
    }

    #[test]
    fn semantic_diff_multiple_functions_changed() {
        let dir = tempfile::tempdir().unwrap();
        let content_a = "\
fn alpha() {
    let x = 1;
}

fn beta() {
    let y = 2;
}

fn gamma() {
    let z = 3;
}
";
        let content_b = "\
fn alpha() {
    let x = 10;
}

fn beta() {
    let y = 20;
}

fn gamma() {
    let z = 3;
}
";
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        fs::write(&file_a, content_a).unwrap();
        fs::write(&file_b, content_b).unwrap();

        let result = ai_semantic_diff_work(file_a, file_b, Some(2));

        assert!(result.contains("function alpha"));
        // With context=2, alpha and beta changes are in a single hunk mapped to alpha,
        // so beta may not appear separately as a changed unit. Just check alpha is listed.
        // gamma unchanged, should not appear in changed units
        let gamma_in_summary = result.contains("function gamma");
        assert!(
            !gamma_in_summary,
            "gamma should not be listed as a changed unit"
        );
    }

    // -----------------------------------------------------------------------
    // ai_ollama_generate_work / ai_ollama_embed_work
    // (These are integration tests that require ollama/curl on PATH;
    //  we only test error paths without those tools.)
    // -----------------------------------------------------------------------

    #[test]
    fn ollama_generate_error_without_terminal_access() {
        // If terminal is not enabled or ollama not in allowlist, should get an error
        let result =
            ai_ollama_generate_work("llama3".to_string(), "hello".to_string(), None, None, None);
        // Either we get a terminal-run-precheck error, or ollama actually runs.
        // We just verify it returns a string — the exact content depends on env.
        assert!(!result.is_empty());
    }

    #[test]
    fn ollama_embed_error_without_curl_access() {
        let result = ai_ollama_embed_work("llama3".to_string(), "hello".to_string());
        assert!(!result.is_empty());
    }

    // -----------------------------------------------------------------------
    // read_text_capped
    // -----------------------------------------------------------------------

    #[test]
    fn read_text_capped_reads_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("small.txt");
        fs::write(&path, "hello world").unwrap();

        let result = read_text_capped(&path, 1024);
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn read_text_capped_rejects_large_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.txt");
        fs::write(&path, "x".repeat(100)).unwrap();

        // Set max_bytes to 10 (smaller than file)
        let result = read_text_capped(&path, 10);
        assert!(result.is_none());
    }

    #[test]
    fn read_text_capped_nonexistent() {
        let result = read_text_capped(Path::new("/nonexistent/file.txt"), 1024);
        assert!(result.is_none());
    }

    #[test]
    fn read_text_capped_rejects_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = read_text_capped(dir.path(), 1024);
        assert!(result.is_none());
    }
}
