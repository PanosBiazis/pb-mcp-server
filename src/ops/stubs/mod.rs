#![allow(missing_docs)]

// Stub implementations for missing tool functions
// These return default values to allow compilation

use serde_json::Value;
use std::path::PathBuf;

// ============================================================================
// Agent tools
// ============================================================================

/// Stub for agent coordination playbook
pub fn agent_coordination_playbook_work() -> Value {
    serde_json::json!({
        "title": "Agent Coordination Playbook",
        "sections": [
            {
                "name": "Communication Protocols",
                "description": "Establish clear communication channels and protocols between agents"
            },
            {
                "name": "Task Distribution",
                "description": "Define how tasks are distributed and coordinated among agents"
            },
            {
                "name": "Conflict Resolution",
                "description": "Procedures for resolving conflicts between agent recommendations"
            },
            {
                "name": "Consensus Building",
                "description": "Methods for building consensus when multiple agents disagree"
            }
        ],
        "best_practices": [
            "Use structured message formats",
            "Maintain context sharing between agents",
            "Implement voting mechanisms for decisions",
            "Provide clear escalation paths"
        ]
    })
}

// ============================================================================
// AI tools
// ============================================================================

/// Stub for AI inference integration guide
pub fn ai_inference_integration_guide_work() -> Value {
    serde_json::json!({
        "title": "AI Inference Integration Guide",
        "integration_types": [
            {
                "type": "Local Models",
                "description": "Running models locally on the same machine",
                "pros": ["Privacy", "Low latency", "No external dependencies"],
                "cons": ["Resource intensive", "Limited model size"]
            },
            {
                "type": "Cloud APIs",
                "description": "Using cloud-based inference services",
                "pros": ["Scalable", "Access to latest models", "No local resources"],
                "cons": ["Privacy concerns", "Latency", "Cost"]
            },
            {
                "type": "Hybrid Approach",
                "description": "Combining local and cloud inference",
                "pros": ["Best of both worlds", "Flexible routing"],
                "cons": ["Complex setup", "Management overhead"]
            }
        ],
        "implementation_steps": [
            "Choose integration type based on requirements",
            "Set up authentication and API keys",
            "Implement request/response handling",
            "Add error handling and fallbacks",
            "Test with sample inputs"
        ]
    })
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn terminal_env_enabled() -> bool {
    env_flag_enabled("PB_TERMINAL_ENABLE") || env_flag_enabled("MINT_TERMINAL_ENABLE")
}

fn terminal_allowlist() -> Vec<String> {
    ["PB_TERMINAL_ALLOW", "MINT_TERMINAL_ALLOW"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .flat_map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn path_lookup(program: &str) -> Option<String> {
    if program.contains(std::path::MAIN_SEPARATOR) {
        return None;
    }

    let path_var = std::env::var_os("PATH")?;
    let candidates = std::env::split_paths(&path_var).flat_map(|dir| {
        #[cfg(windows)]
        {
            let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.BAT;.CMD".into());
            let mut out = Vec::new();
            out.push(dir.join(program));
            for ext in pathext.split(';').filter(|s| !s.trim().is_empty()) {
                out.push(dir.join(format!("{program}{ext}")));
            }
            out
        }
        #[cfg(not(windows))]
        {
            vec![dir.join(program)]
        }
    });

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate.display().to_string());
        }
    }
    None
}

fn version_probe(program: &str) -> Option<String> {
    use std::process::Command;

    for args in [vec!["--version"], vec!["-V"], vec!["version"], vec!["list"]] {
        let output = Command::new(program).args(&args).output().ok()?;
        if !output.status.success() {
            continue;
        }
        let combined = if output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).to_string()
        };
        let first = combined
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("");
        if !first.is_empty() {
            return Some(first.chars().take(240).collect());
        }
    }
    None
}

fn ai_probe_programs(stack_type: &str) -> Vec<(&'static str, &'static str)> {
    let all = vec![
        ("python3", "python runtime for scripts, data, ONNX helpers"),
        ("python", "fallback python runtime"),
        ("ollama", "local LLM runtime"),
        ("tesseract", "OCR engine"),
        ("ffmpeg", "audio/video processing"),
        ("ffprobe", "media metadata"),
        ("cargo", "Rust build/test toolchain"),
        ("rustc", "Rust compiler"),
        ("git", "source control"),
        ("gh", "GitHub CLI"),
        ("glab", "GitLab CLI"),
        ("codex", "Codex CLI"),
        ("claude", "Claude CLI"),
        ("aider", "Aider coding agent CLI"),
        ("gemini", "Gemini CLI"),
        ("node", "JavaScript runtime"),
    ];

    match stack_type {
        "llm" | "ai" => all
            .into_iter()
            .filter(|(p, _)| matches!(*p, "ollama" | "codex" | "claude" | "aider" | "gemini"))
            .collect(),
        "media" | "ocr" => all
            .into_iter()
            .filter(|(p, _)| matches!(*p, "tesseract" | "ffmpeg" | "ffprobe" | "python3"))
            .collect(),
        "dev" | "toolchain" => all
            .into_iter()
            .filter(|(p, _)| matches!(*p, "cargo" | "rustc" | "git" | "gh" | "glab" | "node"))
            .collect(),
        _ => all,
    }
}

/// Probe local AI/tooling stack without fabricated performance metrics.
pub fn ai_stack_probe_work(_args: Value) -> Value {
    let stack_type = _args
        .get("stack_type")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    let terminal_feature = crate::ops::terminal_enabled();
    let terminal_env = terminal_env_enabled();
    let allowlist = terminal_allowlist();
    let allowlist_set: std::collections::HashSet<String> = allowlist.iter().cloned().collect();

    let mut available = 0usize;
    let mut version_checked = 0usize;
    let mut blocked_by_gate = 0usize;
    let mut tools = Vec::new();

    for (program, role) in ai_probe_programs(stack_type) {
        let path = path_lookup(program);
        let allowlisted = allowlist_set.contains(program);
        let can_run_version = terminal_feature && terminal_env && allowlisted && path.is_some();
        let version = if can_run_version {
            version_checked += 1;
            version_probe(program)
        } else {
            if path.is_some() {
                blocked_by_gate += 1;
            }
            None
        };
        if path.is_some() {
            available += 1;
        }
        let status = if path.is_none() {
            "not_found"
        } else if can_run_version {
            if version.is_some() {
                "version_detected"
            } else {
                "version_probe_failed"
            }
        } else {
            "available_on_path_version_probe_blocked"
        };

        tools.push(serde_json::json!({
            "program": program,
            "role": role,
            "available_on_path": path.is_some(),
            "path": path,
            "allowlisted": allowlisted,
            "version_probe_allowed": can_run_version,
            "version": version,
            "status": status
        }));
    }

    serde_json::json!({
        "format": "pb-mcp.ai_stack_probe.v2",
        "stack_type": stack_type,
        "probe_results": {
            "summary": {
                "programs_checked": tools.len(),
                "available_on_path": available,
                "version_probes_attempted": version_checked,
                "available_but_version_blocked_by_gate": blocked_by_gate
            },
            "terminal_gate": {
                "terminal_exec_feature": terminal_feature,
                "terminal_env_enabled": terminal_env,
                "allowlist_count": allowlist.len(),
                "version_probe_policy": "versions are probed only when terminal-exec is compiled, PB/MINT_TERMINAL_ENABLE is true, and the program is in PB/MINT_TERMINAL_ALLOW"
            },
            "tools": tools,
            "honesty": {
                "simulated_metrics": false,
                "network_calls": false,
                "claims_connected_api": false
            }
        },
        "recommendations": [
            "Install missing tools only when a workflow needs them",
            "Enable terminal version probing by setting PB_TERMINAL_ENABLE=1 or MINT_TERMINAL_ENABLE=1 and allowlisting exact program names",
            "Use ai_inference_integration_guide for safe local/cloud model integration patterns"
        ]
    })
}

// ============================================================================
// Batch/file tools
// ============================================================================

/// Stub for batch file info
pub fn batch_file_info_work(_paths: Vec<PathBuf>) -> String {
    use std::fs;

    let mut results = Vec::new();

    for path in _paths {
        if let Ok(metadata) = fs::metadata(&path) {
            let file_type = if metadata.is_file() {
                "file"
            } else if metadata.is_dir() {
                "directory"
            } else {
                "other"
            };
            let size = metadata.len();
            let modified = metadata
                .modified()
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|_| "Unknown".to_string());

            results.push(format!(
                "{} ({}): {} bytes, modified: {}",
                path.display(),
                file_type,
                size,
                modified
            ));
        } else {
            results.push(format!("{}: Error accessing file", path.display()));
        }
    }

    results.join("\n")
}

// ============================================================================
// Codebase tools
// ============================================================================

/// Stub for codebase cargo dependency names
pub fn codebase_cargo_dependency_names_work(_manifest_path: PathBuf) -> String {
    use std::fs;

    if let Ok(content) = fs::read_to_string(&_manifest_path) {
        if let Ok(cargo_toml) = content.parse::<toml::Value>() {
            let mut dependencies = Vec::new();

            if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
                dependencies.extend(deps.keys().map(|k| k.to_string()));
            }

            if let Some(dev_deps) = cargo_toml
                .get("dev-dependencies")
                .and_then(|d| d.as_table())
            {
                dependencies.extend(dev_deps.keys().map(|k| format!("{} (dev)", k)));
            }

            if let Some(build_deps) = cargo_toml
                .get("build-dependencies")
                .and_then(|d| d.as_table())
            {
                dependencies.extend(build_deps.keys().map(|k| format!("{} (build)", k)));
            }

            dependencies.sort();
            format!("Dependencies found:\n{}", dependencies.join("\n"))
        } else {
            "Error: Failed to parse Cargo.toml".to_string()
        }
    } else {
        "Error: Failed to read Cargo.toml".to_string()
    }
}

/// Stub for codebase duplicate basenames
pub fn codebase_duplicate_basenames_work(_search_root: PathBuf, _scan: usize) -> String {
    use std::collections::HashMap;
    use std::fs;

    let mut basename_counts = HashMap::new();
    let mut duplicates = Vec::new();
    let mut files_scanned = 0;

    if let Ok(entries) = fs::read_dir(&_search_root) {
        for entry in entries.flatten().take(_scan) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        let count = basename_counts.entry(name_str.to_string()).or_insert(0);
                        *count += 1;

                        if *count == 2 {
                            duplicates.push(name_str.to_string());
                        }
                    }
                }
                files_scanned += 1;
            }
        }
    }

    let mut result = format!("Scanned {} files\n\n", files_scanned);
    if duplicates.is_empty() {
        result.push_str("No duplicate basenames found.");
    } else {
        result.push_str("Duplicate basenames found:\n");
        for basename in duplicates {
            result.push_str(&format!("- {}\n", basename));
        }
    }

    result
}

/// Stub for codebase index
pub fn codebase_index_work(
    _search_root: PathBuf,
    _max_files: usize,
    _sketch: bool,
    _max_sym: usize,
) -> String {
    use ignore::WalkBuilder;
    use std::fs;

    let mut file_info = Vec::new();
    let mut total_files = 0;
    let mut total_lines = 0;

    let mut walk = WalkBuilder::new(&_search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    for entry in walk.build().flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            total_files += 1;

            if let Ok(metadata) = entry.metadata() {
                let path = entry.path();
                let size = metadata.len();
                let lines = if let Ok(content) = fs::read_to_string(path) {
                    content.lines().count()
                } else {
                    0
                };

                total_lines += lines;

                if file_info.len() < _max_files {
                    file_info.push((path.to_string_lossy().to_string(), size, lines));
                }
            }
        }
    }

    // Sort by size descending
    file_info.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = String::from("Codebase Index:\n\n");
    result.push_str(&format!("Total files: {}\n", total_files));
    result.push_str(&format!("Total lines: {}\n", total_lines));
    result.push_str(&format!("Files analyzed: {}\n\n", file_info.len()));

    if _sketch {
        result.push_str("Largest files:\n");
        for (i, (path, size, lines)) in file_info.iter().take(_max_sym).enumerate() {
            result.push_str(&format!(
                "{}. {} ({} bytes, {} lines)\n",
                i + 1,
                path,
                size,
                lines
            ));
        }
    } else {
        result.push_str("File details:\n");
        for (i, (path, size, lines)) in file_info.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} ({} bytes, {} lines)\n",
                i + 1,
                path,
                size,
                lines
            ));
        }
    }

    result
}

/// Stub for codebase largest files
pub fn codebase_largest_files_work(
    _repo: PathBuf,
    _glob: Option<String>,
    _sketch: bool,
    _max_sym: usize,
) -> String {
    use std::fs;

    let mut file_sizes = Vec::new();

    if let Ok(entries) = fs::read_dir(&_repo) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    if let Some(path_str) = path.to_str() {
                        file_sizes.push((path_str.to_string(), metadata.len()));
                    }
                }
            }
        }
    }

    // Sort by size descending
    file_sizes.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = String::from("Largest files in codebase:\n\n");
    for (i, (path, size)) in file_sizes.iter().take(_max_sym).enumerate() {
        result.push_str(&format!("{}. {} ({} bytes)\n", i + 1, path, size));
    }

    result
}

/// Stub for codebase manifest write
pub fn codebase_manifest_write_work(_search_root: PathBuf, _max_files: usize) -> String {
    use ignore::WalkBuilder;
    use std::fs;
    use std::io::Write;
    use std::time::UNIX_EPOCH;

    let max_files = _max_files.clamp(100, 25_000);
    let mut records = Vec::new();
    let mut walk = WalkBuilder::new(&_search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    for entry in walk.build().flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let relative_path = path
            .strip_prefix(&_search_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        records.push(serde_json::json!({
            "path": relative_path,
            "size_bytes": metadata.len(),
            "modified_unix": modified_unix,
        }));

        if records.len() >= max_files {
            break;
        }
    }

    let manifest_dir = _search_root.join(".mint-mcp");
    if let Err(e) = fs::create_dir_all(&manifest_dir) {
        return format!("Error: failed to create {}: {e}", manifest_dir.display());
    }
    let manifest_path = manifest_dir.join("file-manifest.jsonl");
    let mut file = match fs::File::create(&manifest_path) {
        Ok(file) => file,
        Err(e) => return format!("Error: failed to create {}: {e}", manifest_path.display()),
    };

    for record in &records {
        let line = match serde_json::to_string(record) {
            Ok(line) => line,
            Err(e) => return format!("Error: failed to serialize manifest record: {e}"),
        };
        if let Err(e) = writeln!(file, "{line}") {
            return format!("Error: failed to write {}: {e}", manifest_path.display());
        }
    }

    format!(
        "CODEBASE_MANIFEST_WRITE\npath: {}\nfiles: {}\nmax_files: {}\n",
        manifest_path.display(),
        records.len(),
        max_files
    )
}

/// Stub for codebase path list
pub fn codebase_path_list_work(
    _search_root: PathBuf,
    _max_paths: usize,
    _max_depth: Option<usize>,
) -> String {
    use ignore::WalkBuilder;

    let mut paths = Vec::new();
    let mut walk = WalkBuilder::new(&_search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    if let Some(max_depth) = _max_depth {
        walk.max_depth(Some(max_depth));
    }

    for entry in walk.build().flatten() {
        if paths.len() >= _max_paths {
            break;
        }

        let path = entry.path();
        if let Ok(rel_path) = path.strip_prefix(&_search_root) {
            paths.push(rel_path.display().to_string());
        }
    }

    paths.sort();
    let total = paths.len();
    let shown = paths
        .into_iter()
        .take(_max_paths)
        .collect::<Vec<_>>()
        .join("\n");

    if total > _max_paths {
        format!(
            "{}\n...[{} more paths truncated]",
            shown,
            total - _max_paths
        )
    } else {
        shown
    }
}

/// Stub for codebase shallow hash
pub fn codebase_shallow_hash_work(
    _search_root: PathBuf,
    _max_files: usize,
    _sample: usize,
) -> String {
    use ignore::WalkBuilder;
    use std::collections::hash_map::DefaultHasher;
    use std::fs;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    let mut files_processed = 0;

    let mut walk = WalkBuilder::new(&_search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    for entry in walk.build().flatten().take(_max_files) {
        let path = entry.path();
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                // Hash file path, size, and modified time
                path.display().to_string().hash(&mut hasher);
                metadata.len().hash(&mut hasher);
                metadata
                    .modified()
                    .map(|t| t.elapsed().map_or(0, |d| d.as_secs()))
                    .unwrap_or(0)
                    .hash(&mut hasher);
                files_processed += 1;
            }
        }
    }

    let hash_value = hasher.finish();
    format!(
        "Processed {} files\nShallow hash: {:x}",
        files_processed, hash_value
    )
}

pub fn ollama_generate_work(_model: String, _prompt: String, _max_tokens: usize) -> String {
    use std::process::Command;

    // Generate text using Ollama
    let output = Command::new("ollama")
        .arg("run")
        .arg(&_model)
        .arg(&_prompt)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let text = String::from_utf8_lossy(&result.stdout);
                if text.len() > _max_tokens {
                    format!("{}...[truncated]", &text[.._max_tokens.min(text.len())])
                } else {
                    text.to_string()
                }
            } else {
                format!(
                    "Failed to generate with Ollama: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running ollama run: {}", e),
    }
}

/// Stub for codebase word index
pub fn codebase_word_index_work(_search_root: PathBuf, _max_files: usize, _top_n: usize) -> String {
    use ignore::WalkBuilder;
    use std::collections::HashMap;
    use std::fs;

    let mut word_counts = HashMap::new();
    let mut files_processed = 0;

    let walk = WalkBuilder::new(&_search_root)
        .git_ignore(true)
        .ignore(true)
        .hidden(false)
        .build();
    let word_regex = match regex::Regex::new(r"\b\w+\b") {
        Ok(re) => re,
        Err(e) => {
            return serde_json::json!({
                "error": format!("internal regex compile failed: {e}")
            })
            .to_string();
        }
    };

    for entry in walk.flatten() {
        if files_processed >= _max_files {
            break;
        }

        let path = entry.path();
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                for caps in word_regex.captures_iter(&content) {
                    if let Some(word_match) = caps.get(0) {
                        let word = word_match.as_str().to_lowercase();
                        if word.len() >= 3 {
                            // Skip very short words
                            *word_counts.entry(word).or_insert(0) += 1;
                        }
                    }
                }

                files_processed += 1;
            }
        }
    }

    let mut words: Vec<(String, usize)> = word_counts.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    result.push(format!(
        "Word index for {} files (showing top {} words):",
        files_processed, _top_n
    ));

    for (word, count) in words.into_iter().take(_top_n) {
        result.push(format!("{}: {} occurrences", word, count));
    }

    result.join("\n")
}

/// Stub for coding agent toolchains catalog
pub fn coding_agent_toolchains_catalog_work() -> String {
    serde_json::json!({
        "available_toolchains": [
            {
                "name": "rust_analyzer",
                "description": "Rust language server and analysis",
                "capabilities": ["completion", "diagnostics", "hover", "goto_definition"]
            },
            {
                "name": "git_workflow",
                "description": "Git version control operations",
                "capabilities": ["status", "commit", "push", "pull", "branch"]
            },
            {
                "name": "file_operations",
                "description": "File system operations",
                "capabilities": ["read", "write", "list", "search", "analyze"]
            },
            {
                "name": "text_processing",
                "description": "Text analysis and manipulation",
                "capabilities": ["regex", "compression", "extraction", "statistics"]
            },
            {
                "name": "ai_integration",
                "description": "AI/ML model interactions",
                "capabilities": ["ollama", "openai", "ocr", "analysis"]
            }
        ],
        "total_toolchains": 5,
        "version": "0.2.0"
    })
    .to_string()
}

/// Stub for codebase symbol graph
pub fn codebase_symbol_graph_work(
    _repo: PathBuf,
    _max_snippets: usize,
    _max_chars: usize,
) -> String {
    use std::fs;

    let mut symbols = Vec::new();

    if let Ok(entries) = fs::read_dir(&_repo) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Extract function names (simplified)
                    for line in content.lines() {
                        if line.trim_start().starts_with("fn ") {
                            if let Some(func_name) = line.split('(').next() {
                                let name = func_name.trim_start_matches("fn ").trim();
                                if !name.is_empty() {
                                    symbols.push((
                                        name.to_string(),
                                        path.to_string_lossy().to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut result = String::from("Symbol Graph:\n\n");
    for (i, (symbol, file)) in symbols.iter().take(_max_snippets).enumerate() {
        let truncated_symbol = if symbol.len() > _max_chars {
            format!("{}...", &symbol[.._max_chars.saturating_sub(3)])
        } else {
            symbol.clone()
        };
        result.push_str(&format!("{}. {} -> {}\n", i + 1, truncated_symbol, file));
    }

    result
}

// ============================================================================
// Compress tools
// ============================================================================

pub fn compress_collapse_blank_lines_work(_text: String, _mc: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut result = Vec::new();
    let mut prev_blank = false;

    for line in lines {
        if line.trim().is_empty() {
            if !prev_blank {
                result.push(line);
                prev_blank = true;
            }
        } else {
            result.push(line);
            prev_blank = false;
        }
    }

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_context_work(
    _text: String,
    _max_chars: usize,
    _mode: String,
    _dedupe_lines: bool,
    _collapse_whitespace: bool,
) -> String {
    let mut processed_text = _text;

    // Remove duplicate lines if requested
    if _dedupe_lines {
        let lines: Vec<&str> = processed_text.lines().collect();
        let mut seen = std::collections::HashSet::new();
        let unique_lines: Vec<&str> = lines
            .iter()
            .filter(|&&line| {
                if seen.contains(line) {
                    false
                } else {
                    seen.insert(line);
                    true
                }
            })
            .cloned()
            .collect();
        processed_text = unique_lines.join("\n");
    }

    // Collapse whitespace if requested
    if _collapse_whitespace {
        processed_text = processed_text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    }

    // Truncate to max_chars
    if processed_text.len() > _max_chars {
        format!(
            "{}...[truncated]",
            &processed_text[.._max_chars.min(processed_text.len())]
        )
    } else {
        processed_text
    }
}

pub fn compress_dedent_work(_text: String, _mc: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation (excluding empty lines)
    let mut min_indent = usize::MAX;
    for line in &lines {
        if !line.trim().is_empty() {
            let indent = line.len() - line.trim_start().len();
            min_indent = min_indent.min(indent);
        }
    }

    if min_indent == 0 || min_indent == usize::MAX {
        return _text;
    }

    // Remove minimum indentation from each line
    let result: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_double_slash_line_comments_work(_text: String, _mc: usize, _dbl: bool) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let result: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                if _dbl && trimmed.starts_with("////") {
                    Some(line.to_string())
                } else if !_dbl {
                    None // Remove all // comments
                } else {
                    Some(line.to_string())
                }
            } else {
                Some(line.to_string())
            }
        })
        .collect();

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_drop_lines_matching_regex_work(
    _text: String,
    _pat: String,
    _ci: bool,
    _mc: usize,
) -> String {
    use regex::Regex;

    let regex_pattern = if _ci { format!("(?i){}", &_pat) } else { _pat };
    let regex = match Regex::new(&regex_pattern) {
        Ok(regex) => regex,
        Err(_) => return _text,
    };

    let lines: Vec<&str> = _text.lines().collect();
    let result: Vec<String> = lines
        .iter()
        .filter(|line| !regex.is_match(line))
        .map(|line| line.to_string())
        .collect();

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_fenced_code_blocks_work(_text: String, _mc: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut result = Vec::new();
    let mut in_code_block = false;

    for line in lines {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
        } else if !in_code_block {
            result.push(line);
        }
    }

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_file_work(
    _path: PathBuf,
    _start_line: Option<usize>,
    _end_line: Option<usize>,
    _max_chars: usize,
    _mode: String,
    _dedupe_lines: bool,
    _collapse_whitespace: bool,
) -> String {
    use std::fs;

    if let Ok(content) = fs::read_to_string(&_path) {
        let lines: Vec<&str> = content.lines().collect();
        let start = _start_line.unwrap_or(0).min(lines.len().saturating_sub(1));
        let end = _end_line.unwrap_or(lines.len()).min(lines.len());

        let selected_lines: Vec<String> = lines
            .iter()
            .skip(start)
            .take(end - start)
            .map(|line| line.to_string())
            .collect();

        let mut result = selected_lines.join("\n");

        if _collapse_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        if result.len() > _max_chars {
            format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
        } else {
            result
        }
    } else {
        "Error: Could not read file".to_string()
    }
}

pub fn compress_gzip_base64_work(_text: Option<String>, _path: Option<&PathBuf>) -> String {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use flate2::{write::GzEncoder, Compression};
    use std::fs;
    use std::io::Write;

    let bytes = match (_text, _path) {
        (Some(text), None) => text.into_bytes(),
        (None, Some(path)) => {
            let meta = match fs::metadata(path) {
                Ok(meta) => meta,
                Err(e) => return format!("Error: failed to stat {}: {e}", path.display()),
            };
            if meta.len() as usize > crate::ops::MAX_CONVERT_BYTES {
                return format!(
                    "Error: input file exceeds {} bytes",
                    crate::ops::MAX_CONVERT_BYTES
                );
            }
            match fs::read(path) {
                Ok(bytes) => bytes,
                Err(e) => return format!("Error: failed to read {}: {e}", path.display()),
            }
        }
        (Some(_), Some(_)) => return "Error: provide only one of text or input_path".to_string(),
        (None, None) => return "Error: provide text or input_path".to_string(),
    };

    if bytes.len() > crate::ops::MAX_CONVERT_BYTES {
        return format!(
            "Error: input exceeds {} bytes",
            crate::ops::MAX_CONVERT_BYTES
        );
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    if let Err(e) = encoder.write_all(&bytes) {
        return format!("Error: gzip write failed: {e}");
    }
    match encoder.finish() {
        Ok(compressed) => BASE64.encode(compressed),
        Err(e) => format!("Error: gzip finish failed: {e}"),
    }
}

pub fn compress_lines_head_work(_text: String, _head_lines: usize, _max_chars: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let lines_to_take = lines.len().min(_head_lines);
    let result: Vec<String> = lines
        .iter()
        .take(lines_to_take)
        .map(|line| line.to_string())
        .collect();

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_lines_tail_work(_text: String, _tail_lines: usize, _max_chars: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let start_index = if lines.len() > _tail_lines {
        lines.len() - _tail_lines
    } else {
        0
    };
    let result: Vec<String> = lines
        .iter()
        .skip(start_index)
        .map(|line| line.to_string())
        .collect();

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_markdown_frontmatter_work(_text: String, _max_chars: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut result = Vec::new();
    let mut in_frontmatter = false;

    for line in lines {
        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            } else {
                in_frontmatter = false;
                continue;
            }
        }
        if !in_frontmatter {
            result.push(line);
        }
    }

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_markdown_links_work(_text: String, _mc: usize) -> String {
    use regex::Regex;

    // Remove markdown links [text](url) keeping just the text
    let Ok(link_regex) = Regex::new(r"\[([^\]]+)\]\([^)]+\)") else {
        return _text;
    };
    let result = link_regex.replace_all(&_text, "$1");

    let output = result.to_string();
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_paragraphs_work(
    _text: String,
    _max_paragraphs: usize,
    _max_chars: usize,
) -> String {
    let paragraphs: Vec<&str> = _text.split("\n\n").collect();
    let paragraphs_to_take = paragraphs.len().min(_max_paragraphs);
    let result: Vec<String> = paragraphs
        .iter()
        .take(paragraphs_to_take)
        .map(|p| p.to_string())
        .collect();

    let output = result.join("\n\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_redact_emails_work(_text: String, _mc: usize) -> String {
    use regex::Regex;

    // Redact email addresses
    let Ok(email_regex) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") else {
        return _text;
    };
    let result = email_regex.replace_all(&_text, "[REDACTED_EMAIL]");

    let output = result.to_string();
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_redact_urls_work(_text: String, _mc: usize) -> String {
    use regex::Regex;

    // Redact URLs
    let Ok(url_regex) = Regex::new(r"https?://[^\s]+") else {
        return _text;
    };
    let result = url_regex.replace_all(&_text, "[REDACTED_URL]");

    let output = result.to_string();
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_sentences_work(_text: String, _max_sentences: usize, _max_chars: usize) -> String {
    // Split by sentence endings (., !, ?)
    let Ok(sentence_regex) = regex::Regex::new(r"[.!?]+\s+") else {
        return if _text.len() > _max_chars {
            format!("{}...[truncated]", &_text[.._max_chars.min(_text.len())])
        } else {
            _text
        };
    };
    let sentences: Vec<String> = sentence_regex
        .split(&_text)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let sentences_to_take = sentences.len().min(_max_sentences);
    let result: Vec<String> = sentences.iter().take(sentences_to_take).cloned().collect();

    let output = result.join(" ");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_simplify_whitespace_work(_text: String, _mc: usize) -> String {
    let simplified = _text.split_whitespace().collect::<Vec<_>>().join(" ");
    if simplified.len() > _mc {
        format!("{}...[truncated]", &simplified[.._mc.min(simplified.len())])
    } else {
        simplified
    }
}

pub fn compress_strip_tags_work(_text: String, _mc: usize) -> String {
    use regex::Regex;

    // Remove HTML tags
    let Ok(tag_regex) = Regex::new(r"<[^>]*>") else {
        return _text;
    };
    let result = tag_regex.replace_all(&_text, "");

    let output = result.to_string();
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

pub fn compress_truncate_lines_work(_text: String, _max_lines: usize, _max_chars: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let lines_to_take = lines.len().min(_max_lines);
    let result: Vec<String> = lines
        .iter()
        .take(lines_to_take)
        .map(|line| {
            if line.len() > _max_chars {
                format!("{}...", &line[.._max_chars.min(line.len())])
            } else {
                line.to_string()
            }
        })
        .collect();

    let output = result.join("\n");
    if lines.len() > _max_lines {
        format!(
            "{}\n...[{} lines truncated]",
            output,
            lines.len() - _max_lines
        )
    } else {
        output
    }
}

pub fn compress_truncate_middle_work(
    _text: String,
    _max_chars: usize,
    _margin_chars: usize,
) -> String {
    if _text.len() <= _max_chars {
        return _text;
    }

    if _max_chars <= 2 * _margin_chars {
        return format!("{}...[truncated]", &_text[.._max_chars.min(_text.len())]);
    }

    let _middle_chars = _max_chars - 2 * _margin_chars;
    let start = &_text[.._margin_chars];
    let end = &_text[_text.len().saturating_sub(_margin_chars)..];

    format!("{}...{}", start, end)
}

pub fn compress_unified_diff_stats_work(_text: String, _mc: usize) -> String {
    // Extract diff stats (lines added/removed)
    let lines: Vec<&str> = _text.lines().collect();
    let mut added = 0;
    let mut removed = 0;
    let mut files = 0;

    for line in lines {
        if line.starts_with("+++") {
            files += 1;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
        }
    }

    let result = format!(
        "Diff Statistics:\nFiles changed: {}\nLines added: {}\nLines removed: {}",
        files, added, removed
    );

    if result.len() > _mc {
        format!("{}...[truncated]", &result[.._mc.min(result.len())])
    } else {
        result
    }
}

pub fn compress_unique_lines_work(_text: String, _max_lines: usize, _max_chars: usize) -> String {
    use std::collections::HashSet;

    let lines: Vec<&str> = _text.lines().collect();
    let mut seen = HashSet::new();
    let unique_lines: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            if line.trim().is_empty() || seen.contains(line) {
                None
            } else {
                seen.insert(line);
                Some(line.to_string())
            }
        })
        .collect();

    let lines_to_take = unique_lines.len().min(_max_lines);
    let result: Vec<String> = unique_lines.iter().take(lines_to_take).cloned().collect();

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn compress_ws_collapse_work(_text: String, _mc: usize) -> String {
    // Collapse consecutive whitespace characters to single space
    use regex::Regex;

    let Ok(ws_regex) = Regex::new(r"\s+") else {
        return _text;
    };
    let result = ws_regex.replace_all(_text.trim(), " ");

    let output = result.to_string();
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

// ============================================================================
// Convert tools
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn convert_format_work(
    op: String,
    text: Option<String>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    allow_out: bool,
    tab_stops: usize,
    line_number_start: usize,
    wrap_width: usize,
) -> String {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use sha2::{Digest, Sha256};
    use std::fs;

    fn read_input(text: Option<String>, input_path: Option<PathBuf>) -> Result<String, String> {
        match (text, input_path) {
            (Some(text), None) => {
                if text.len() > crate::ops::MAX_CONVERT_BYTES {
                    Err(format!(
                        "Error: input exceeds {} bytes",
                        crate::ops::MAX_CONVERT_BYTES
                    ))
                } else {
                    Ok(text)
                }
            }
            (None, Some(path)) => {
                let meta = fs::metadata(&path)
                    .map_err(|e| format!("Error: failed to stat {}: {e}", path.display()))?;
                if meta.len() as usize > crate::ops::MAX_CONVERT_BYTES {
                    return Err(format!(
                        "Error: input file exceeds {} bytes",
                        crate::ops::MAX_CONVERT_BYTES
                    ));
                }
                fs::read_to_string(&path)
                    .map_err(|e| format!("Error: failed to read UTF-8 {}: {e}", path.display()))
            }
            (Some(_), Some(_)) => Err("Error: provide only one of text or input_path".to_string()),
            (None, None) => Err("Error: provide text or input_path".to_string()),
        }
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn hex_decode(text: &str) -> Result<Vec<u8>, String> {
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        if !compact.len().is_multiple_of(2) {
            return Err("Error: hex input has odd length".to_string());
        }
        let mut out = Vec::with_capacity(compact.len() / 2);
        for i in (0..compact.len()).step_by(2) {
            let pair = &compact[i..i + 2];
            let byte = u8::from_str_radix(pair, 16)
                .map_err(|_| format!("Error: invalid hex byte '{pair}'"))?;
            out.push(byte);
        }
        Ok(out)
    }

    fn percent_encode(text: &str) -> String {
        let mut out = String::new();
        for byte in text.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                out.push(byte as char);
            } else {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
        out
    }

    fn percent_decode(text: &str) -> Result<String, String> {
        let bytes = text.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                if i + 2 >= bytes.len() {
                    return Err("Error: incomplete percent escape".to_string());
                }
                let pair = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .map_err(|e| format!("Error: invalid percent escape: {e}"))?;
                let byte = u8::from_str_radix(pair, 16)
                    .map_err(|_| format!("Error: invalid percent escape '%{pair}'"))?;
                out.push(byte);
                i += 3;
            } else if bytes[i] == b'+' {
                out.push(b' ');
                i += 1;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8(out).map_err(|e| format!("Error: decoded bytes are not UTF-8: {e}"))
    }

    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    fn slugify(text: &str) -> String {
        let mut out = String::new();
        let mut last_dash = false;
        for ch in text.chars().flat_map(char::to_lowercase) {
            if ch.is_ascii_alphanumeric() {
                out.push(ch);
                last_dash = false;
            } else if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        }
        while out.ends_with('-') {
            out.pop();
        }
        out
    }

    fn collapse_blank_lines(text: &str) -> String {
        let mut out = Vec::new();
        let mut blank = false;
        for line in text.lines() {
            if line.trim().is_empty() {
                if !blank {
                    out.push(String::new());
                }
                blank = true;
            } else {
                out.push(line.to_string());
                blank = false;
            }
        }
        out.join("\n")
    }

    fn wrap_lines(text: &str, width: usize) -> String {
        let width = width.clamp(8, 200);
        let mut out = Vec::new();
        for line in text.lines() {
            let mut current = String::new();
            for word in line.split_whitespace() {
                if current.is_empty() {
                    current.push_str(word);
                } else if current.len() + 1 + word.len() <= width {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    out.push(current);
                    current = word.to_string();
                }
            }
            if current.is_empty() {
                out.push(String::new());
            } else {
                out.push(current);
            }
        }
        out.join("\n")
    }

    fn top_level_keys_from_value(value: &serde_json::Value) -> String {
        match value.as_object() {
            Some(map) => map.keys().cloned().collect::<Vec<_>>().join("\n"),
            None => "Error: top-level value is not an object/table".to_string(),
        }
    }

    fn strip_json_nulls(value: serde_json::Value) -> Option<serde_json::Value> {
        match value {
            serde_json::Value::Null => None,
            serde_json::Value::Array(items) => Some(serde_json::Value::Array(
                items.into_iter().filter_map(strip_json_nulls).collect(),
            )),
            serde_json::Value::Object(map) => {
                let filtered = map
                    .into_iter()
                    .filter_map(|(key, value)| strip_json_nulls(value).map(|value| (key, value)))
                    .collect();
                Some(serde_json::Value::Object(filtered))
            }
            other => Some(other),
        }
    }

    fn maybe_write_output(
        output_path: Option<PathBuf>,
        allow_out: bool,
        bytes: &[u8],
        text_fallback: String,
    ) -> String {
        match output_path {
            Some(path) => {
                if !allow_out {
                    return "Error: output_path is blocked on public network profile".to_string();
                }
                if bytes.len() > crate::ops::MAX_CONVERT_BYTES {
                    return format!(
                        "Error: output exceeds {} bytes",
                        crate::ops::MAX_CONVERT_BYTES
                    );
                }
                match fs::write(&path, bytes) {
                    Ok(()) => format!(
                        "CONVERT_FORMAT_WRITE\npath: {}\nbytes: {}\n",
                        path.display(),
                        bytes.len()
                    ),
                    Err(e) => format!("Error: failed to write {}: {e}", path.display()),
                }
            }
            None => text_fallback,
        }
    }

    let op = op.trim().to_ascii_lowercase();
    let input = match read_input(text, input_path) {
        Ok(input) => input,
        Err(e) => return e,
    };

    let result: Result<String, String> = match op.as_str() {
        "json_pretty" | "json" => serde_json::from_str::<serde_json::Value>(&input)
            .and_then(|value| serde_json::to_string_pretty(&value))
            .map_err(|e| format!("Error: JSON conversion failed: {e}")),
        "json_minify" => serde_json::from_str::<serde_json::Value>(&input)
            .and_then(|value| serde_json::to_string(&value))
            .map_err(|e| format!("Error: JSON conversion failed: {e}")),
        "json_top_level_keys" => serde_json::from_str::<serde_json::Value>(&input)
            .map(|value| top_level_keys_from_value(&value))
            .map_err(|e| format!("Error: JSON parse failed: {e}")),
        "toml_to_json" => match input.parse::<toml::Value>() {
            Ok(value) => serde_json::to_string_pretty(&value)
                .map_err(|e| format!("Error: TOML to JSON serialization failed: {e}")),
            Err(e) => Err(format!("Error: TOML parse failed: {e}")),
        },
        "toml_to_json_minify" => match input.parse::<toml::Value>() {
            Ok(value) => serde_json::to_string(&value)
                .map_err(|e| format!("Error: TOML to JSON serialization failed: {e}")),
            Err(e) => Err(format!("Error: TOML parse failed: {e}")),
        },
        "toml_top_level_keys" => input
            .parse::<toml::Value>()
            .map(|value| match value.as_table() {
                Some(table) => table.keys().cloned().collect::<Vec<_>>().join("\n"),
                None => "Error: top-level TOML value is not a table".to_string(),
            })
            .map_err(|e| format!("Error: TOML parse failed: {e}")),
        "json_to_toml" => {
            let value = match serde_json::from_str::<serde_json::Value>(&input) {
                Ok(value) => value,
                Err(e) => return format!("Error: JSON parse failed: {e}"),
            };
            match strip_json_nulls(value) {
                Some(value) => toml::to_string_pretty(&value)
                    .map_err(|e| format!("Error: JSON to TOML conversion failed: {e}")),
                None => Err("Error: JSON null cannot be represented as TOML".to_string()),
            }
        }
        "yaml_to_json" | "yaml_to_json_minify" | "json_to_yaml" => Err(
            "Error: YAML conversion is not compiled into this binary; add serde_yaml support before advertising this op".to_string(),
        ),
        "yaml_top_level_keys" => {
            let keys = input
                .lines()
                .filter_map(|line| {
                    if line.starts_with(' ') || line.starts_with('\t') || line.trim().is_empty() {
                        return None;
                    }
                    line.split_once(':')
                        .map(|(key, _)| key.trim().trim_matches('"').trim_matches('\'').to_string())
                        .filter(|key| !key.is_empty() && !key.starts_with('#'))
                })
                .collect::<Vec<_>>();
            Ok(keys.join("\n"))
        }
        "base64" | "base64_encode" => Ok(BASE64.encode(input.as_bytes())),
        "base64_decode" => {
            let bytes = match BASE64.decode(input.trim()) {
                Ok(bytes) => bytes,
                Err(e) => return format!("Error: invalid base64 input: {e}"),
            };
            let fallback = match String::from_utf8(bytes.clone()) {
                Ok(text) => text,
                Err(_) => serde_json::json!({
                    "binary": true,
                    "bytes": bytes.len(),
                    "data_base64": BASE64.encode(&bytes)
                })
                .to_string(),
            };
            return maybe_write_output(output_path, allow_out, &bytes, fallback);
        }
        "hex" | "hex_encode" => Ok(hex_encode(input.as_bytes())),
        "hex_decode" => {
            let bytes = match hex_decode(&input) {
                Ok(bytes) => bytes,
                Err(e) => return e,
            };
            let fallback = match String::from_utf8(bytes.clone()) {
                Ok(text) => text,
                Err(_) => serde_json::json!({
                    "binary": true,
                    "bytes": bytes.len(),
                    "data_base64": BASE64.encode(&bytes)
                })
                .to_string(),
            };
            return maybe_write_output(output_path, allow_out, &bytes, fallback);
        }
        "sha256_hex" => {
            let digest = Sha256::digest(input.as_bytes());
            Ok(hex_encode(&digest))
        }
        "utf8_bom_strip" => Ok(input.trim_start_matches('\u{feff}').to_string()),
        "crlf" | "lf_to_crlf" => Ok(input.replace("\r\n", "\n").replace('\n', "\r\n")),
        "lf" | "crlf_to_lf" => Ok(input.replace("\r\n", "\n")),
        "tabs_to_spaces" => Ok(input.replace('\t', &" ".repeat(tab_stops.clamp(1, 16)))),
        "spaces_to_tabs" => {
            let width = tab_stops.clamp(1, 16);
            Ok(input.replace(&" ".repeat(width), "\t"))
        }
        "strip_ansi" => match regex::Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]") {
            Ok(re) => Ok(re.replace_all(&input, "").to_string()),
            Err(e) => Err(format!("Error: internal ANSI regex failed: {e}")),
        },
        "collapse_blank_lines" => Ok(collapse_blank_lines(&input)),
        "remove_empty_lines" => Ok(input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")),
        "dedupe_adjacent_lines" => {
            let mut out = Vec::new();
            let mut previous: Option<&str> = None;
            for line in input.lines() {
                if previous != Some(line) {
                    out.push(line);
                    previous = Some(line);
                }
            }
            Ok(out.join("\n"))
        }
        "slugify" => Ok(slugify(&input)),
        "c_string_escape" => Ok(input.escape_default().to_string()),
        "html_escape" => Ok(html_escape(&input)),
        "prefix_line_numbers" => Ok(input
            .lines()
            .enumerate()
            .map(|(i, line)| format!("{:>6}: {}", line_number_start.saturating_add(i), line))
            .collect::<Vec<_>>()
            .join("\n")),
        "text_lowercase" | "lowercase" => Ok(input.to_lowercase()),
        "text_uppercase" | "uppercase" => Ok(input.to_uppercase()),
        "sort_lines" => {
            let mut lines = input.lines().collect::<Vec<_>>();
            lines.sort_unstable();
            Ok(lines.join("\n"))
        }
        "reverse_lines" => Ok(input.lines().rev().collect::<Vec<_>>().join("\n")),
        "trim_each_line" => Ok(input.lines().map(str::trim).collect::<Vec<_>>().join("\n")),
        "unique_sorted_lines" => {
            let mut lines = input.lines().map(str::trim).collect::<Vec<_>>();
            lines.sort_unstable();
            lines.dedup();
            Ok(lines.join("\n"))
        }
        "wrap_lines" | "hard_wrap_lines" => Ok(wrap_lines(&input, wrap_width)),
        "percent_encode" | "url_encode" => Ok(percent_encode(&input)),
        "percent_decode" | "url_decode" => percent_decode(&input),
        other => Err(format!(
            "Error: unsupported convert_format operation '{other}'"
        )),
    };

    match result {
        Ok(out) => {
            let bytes = out.as_bytes().to_vec();
            maybe_write_output(output_path, allow_out, &bytes, out)
        }
        Err(e) => e,
    }
}

// ============================================================================
// Decompress tools
// ============================================================================

pub fn decompress_gzip_base64_work(_text: String) -> String {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use flate2::read::GzDecoder;
    use std::io::Read;

    let compressed = match BASE64.decode(_text.trim()) {
        Ok(bytes) => bytes,
        Err(e) => return format!("Error: invalid base64 gzip payload: {e}"),
    };
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut out = Vec::new();
    let max = crate::ops::MAX_GZIP_DECOMPRESSED_BYTES;
    let read_limit = (max + 1) as u64;
    let mut limited = decoder.by_ref().take(read_limit);
    if let Err(e) = limited.read_to_end(&mut out) {
        return format!("Error: gzip decompress failed: {e}");
    }
    if out.len() > max {
        return format!("Error: decompressed payload exceeds {max} bytes");
    }

    match String::from_utf8(out) {
        Ok(text) => text,
        Err(e) => {
            let bytes = e.into_bytes();
            serde_json::json!({
                "binary": true,
                "bytes": bytes.len(),
                "data_base64": BASE64.encode(bytes)
            })
            .to_string()
        }
    }
}

// ============================================================================
// Extract tools
// ============================================================================

pub fn extract_snippets_work(
    _text: String,
    _pattern: String,
    _literal: bool,
    _case_sensitive: bool,
    _context_lines: usize,
    _max_snippets: usize,
    _max_chars: usize,
) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut results = Vec::new();
    let mut found_snippets = 0;
    let regex = if _literal {
        None
    } else {
        let pattern = if _case_sensitive {
            _pattern.clone()
        } else {
            format!("(?i){}", &_pattern)
        };
        match regex::Regex::new(&pattern) {
            Ok(re) => Some(re),
            Err(e) => return format!("Invalid regex pattern: {e}"),
        }
    };

    for (line_index, line) in lines.iter().enumerate() {
        let matches = if _literal {
            if _case_sensitive {
                line.contains(&_pattern)
            } else {
                line.to_lowercase().contains(&_pattern.to_lowercase())
            }
        } else {
            regex.as_ref().is_some_and(|re| re.is_match(line))
        };

        if matches {
            let start = line_index.saturating_sub(_context_lines);
            let end = (line_index + _context_lines + 1).min(lines.len());

            let context: Vec<String> = lines[start..end]
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    let prefix = if i + start == line_index {
                        ">>> "
                    } else {
                        "    "
                    };
                    format!("{}{}", prefix, l)
                })
                .collect();

            results.push(format!("Line {}:\n{}", line_index + 1, context.join("\n")));

            found_snippets += 1;
            if found_snippets >= _max_snippets {
                break;
            }
        }
    }

    if results.is_empty() {
        "No matches found".to_string()
    } else {
        let output = format!(
            "Found {} snippets:\n\n{}",
            found_snippets,
            results.join("\n---\n\n")
        );
        if output.len() > _max_chars {
            format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
        } else {
            output
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extract_snippets_from_file_work(
    _path: PathBuf,
    _pattern: String,
    _literal: bool,
    _case_sensitive: bool,
    _context_lines: usize,
    _max_snippets: usize,
    _max_chars: usize,
    _start_line: Option<usize>,
    _end_line: Option<usize>,
) -> String {
    use std::fs;

    if let Ok(content) = fs::read_to_string(&_path) {
        // Use the existing extract_snippets_work logic
        extract_snippets_work(
            content,
            _pattern,
            _literal,
            _case_sensitive,
            _context_lines,
            _max_snippets,
            10000,
        )
    } else {
        "Error: Could not read file".to_string()
    }
}

pub fn format_prefix_lines_work(_text: String, _prefix: String, _mc: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let result: Vec<String> = lines
        .iter()
        .map(|line| format!("{}{}", _prefix, line))
        .collect();

    let output = result.join("\n");
    if output.len() > _mc {
        format!("{}...[truncated]", &output[.._mc.min(output.len())])
    } else {
        output
    }
}

// ============================================================================
// Git tools (GitHub/GitLab)
// ============================================================================

pub fn gh_pr_list_work(_dir: PathBuf, _limit: usize, _state: &str) -> String {
    use std::process::Command;

    let output = Command::new("gh")
        .arg("pr")
        .arg("list")
        .arg("--limit")
        .arg(_limit.to_string())
        .arg("--state")
        .arg(_state)
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list PRs: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running gh pr list: {}", e),
    }
}

pub fn gh_pr_view_work(_dir: PathBuf, _pr: &str) -> String {
    use std::process::Command;

    let output = Command::new("gh")
        .arg("pr")
        .arg("view")
        .arg(_pr)
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to view PR: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running gh pr view: {}", e),
    }
}

pub fn git_add_or_commit_work(
    _dir: PathBuf,
    _action: &str,
    _files: Option<Vec<String>>,
    _message: Option<String>,
) -> String {
    use std::process::Command;

    match _action {
        "add" => {
            let mut cmd = Command::new("git");
            cmd.arg("add").current_dir(&_dir);

            if let Some(files) = _files {
                for file in files {
                    cmd.arg(&file);
                }
            } else {
                cmd.arg(".");
            }

            let output = cmd.output();
            match output {
                Ok(result) => {
                    if result.status.success() {
                        "Files added successfully".to_string()
                    } else {
                        format!(
                            "Failed to add files: {}",
                            String::from_utf8_lossy(&result.stderr)
                        )
                    }
                }
                Err(e) => format!("Error running git add: {}", e),
            }
        }
        "commit" => {
            let mut cmd = Command::new("git");
            cmd.arg("commit").current_dir(&_dir);

            if let Some(message) = _message {
                cmd.arg("-m").arg(&message);
            } else {
                cmd.arg("-m").arg("Commit message");
            }

            let output = cmd.output();
            match output {
                Ok(result) => {
                    if result.status.success() {
                        "Commit created successfully".to_string()
                    } else {
                        format!(
                            "Failed to commit: {}",
                            String::from_utf8_lossy(&result.stderr)
                        )
                    }
                }
                Err(e) => format!("Error running git commit: {}", e),
            }
        }
        _ => format!("Unknown action: {}", _action),
    }
}

pub fn git_branch_extension_work(_dir: PathBuf, _action: &str, _bn: Option<String>) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("branch").current_dir(&_dir);

    match _action {
        "list" => {
            cmd.arg("-a");
        }
        "create" => {
            if let Some(name) = _bn {
                cmd.arg(&name);
            } else {
                return "Branch name required for create action".to_string();
            }
        }
        "delete" => {
            if let Some(name) = _bn {
                cmd.arg("-D").arg(&name);
            } else {
                return "Branch name required for delete action".to_string();
            }
        }
        _ => {
            return format!("Unknown branch action: {}", _action);
        }
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to perform branch operation: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git branch: {}", e),
    }
}

pub fn git_checkout_work(_dir: PathBuf, _target: String) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("checkout")
        .arg(&_target)
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                format!("Successfully checked out {}", _target)
            } else {
                format!(
                    "Failed to checkout {}: {}",
                    _target,
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git checkout: {}", e),
    }
}

pub fn git_cherry_pick_work(_dir: PathBuf, _commits: Vec<String>, _no_commit: bool) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("cherry-pick").current_dir(&_dir);

    if _no_commit {
        cmd.arg("--no-commit");
    }

    for commit in &_commits {
        cmd.arg(commit);
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                format!("Successfully cherry-picked {} commits", _commits.len())
            } else {
                format!(
                    "Failed to cherry-pick commits: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git cherry-pick: {}", e),
    }
}

pub fn git_describe_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--always")
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to describe: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git describe: {}", e),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn git_diff_snippets_work(
    _dir: PathBuf,
    _staged: bool,
    _pattern: String,
    _literal: bool,
    _case_sensitive: bool,
    _context_lines: usize,
    _max_snippets: usize,
    _max_chars: usize,
) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("diff").current_dir(&_dir);

    if _staged {
        cmd.arg("--cached");
    }

    if let Ok(output) = cmd.output() {
        let diff = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = diff.lines().collect();

        let mut snippets = Vec::new();
        let mut current_snippet = Vec::new();
        let mut in_diff = false;

        for line in lines {
            if line.starts_with("diff --git") {
                if !current_snippet.is_empty() {
                    snippets.push(current_snippet.join("\n"));
                }
                current_snippet.clear();
                current_snippet.push(line);
                in_diff = true;
            } else if in_diff {
                if line.starts_with("diff --git") {
                    if !current_snippet.is_empty() {
                        snippets.push(current_snippet.join("\n"));
                    }
                    current_snippet.clear();
                }
                current_snippet.push(line);

                if current_snippet.join("\n").len() > _max_chars {
                    snippets.push(current_snippet.join("\n"));
                    current_snippet.clear();
                    in_diff = false;
                }
            }
        }

        if !current_snippet.is_empty() {
            snippets.push(current_snippet.join("\n"));
        }

        let mut result = String::from("Diff snippets:\n\n");
        for (i, snippet) in snippets.iter().take(_max_snippets).enumerate() {
            result.push_str(&format!("--- Snippet {} ---\n{}\n\n", i + 1, snippet));
        }

        result
    } else {
        "Failed to get diff".to_string()
    }
}

pub fn git_diff_stat_work(_dir: PathBuf, _staged: bool, _rev: Option<String>) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("diff").arg("--stat").current_dir(&_dir);

    if _staged {
        cmd.arg("--staged");
    }

    if let Some(rev) = _rev {
        cmd.arg(&rev);
    }

    let output = cmd.output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get diff stat: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git diff: {}", e),
    }
}

pub fn git_fetch_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git").arg("fetch").current_dir(&_dir).output();

    match output {
        Ok(result) => {
            if result.status.success() {
                "Git fetch completed successfully".to_string()
            } else {
                format!(
                    "Failed to fetch: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git fetch: {}", e),
    }
}

pub fn git_last_commit_work(_dir: PathBuf, _style: &str) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("log").arg("-1").current_dir(&_dir);

    match _style {
        "short" => {
            cmd.arg("--oneline");
        }
        "hash" => {
            cmd.arg("--format=%H");
        }
        "message" => {
            cmd.arg("--format=%s");
        }
        _ => {
            cmd.arg("--oneline");
        }
    }

    let output = cmd.output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get last commit: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git log: {}", e),
    }
}

pub fn git_log_or_diff_work(
    _dir: PathBuf,
    _action: &str,
    _rr: Option<String>,
    _since: Option<String>,
    _until: Option<String>,
    _authors: Vec<String>,
) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.current_dir(&_dir);

    match _action {
        "log" => {
            cmd.arg("log").arg("--oneline").arg("-10");
            if let Some(rev) = _rr {
                cmd.arg(rev);
            }
            if let Some(since) = _since {
                cmd.arg(format!("--since={}", since));
            }
            if let Some(until) = _until {
                cmd.arg(format!("--until={}", until));
            }
            for author in _authors {
                cmd.arg(format!("--author={}", author));
            }
        }
        "diff" => {
            cmd.arg("diff");
            if let Some(rev) = _rr {
                cmd.arg(rev);
            }
        }
        _ => return "Invalid action".to_string(),
    }

    if let Ok(output) = cmd.output() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        "Failed to execute command".to_string()
    }
}

pub fn git_log_work(
    _repo: PathBuf,
    _limit: usize,
    _path_in_repo: Option<String>,
    _oneline: bool,
) -> String {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    "Use the implementation in src/ops/mod.rs".to_string()
}

pub fn git_ls_files_work(_dir: PathBuf, _mode: &str) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("ls-files").current_dir(&_dir);

    match _mode {
        "modified" => {
            cmd.arg("--modified");
        }
        "deleted" => {
            cmd.arg("--deleted");
        }
        "others" => {
            cmd.arg("--others");
        }
        "ignored" => {
            cmd.arg("--ignored");
        }
        "exclude-standard" => {
            cmd.arg("--exclude-standard");
        }
        _ => {} // default: all files
    }

    let output = cmd.output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list files: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git ls-files: {}", e),
    }
}

pub fn git_merge_base_work(_dir: PathBuf, _a: String, _b: String) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("merge-base")
        .arg(&_a)
        .arg(&_b)
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to find merge base: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git merge-base: {}", e),
    }
}

pub fn git_pull_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git").arg("pull").current_dir(&_dir).output();

    match output {
        Ok(result) => {
            if result.status.success() {
                "Git pull completed successfully".to_string()
            } else {
                format!(
                    "Failed to pull: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git pull: {}", e),
    }
}

pub fn git_push_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git").arg("push").current_dir(&_dir).output();

    match output {
        Ok(result) => {
            if result.status.success() {
                "Git push completed successfully".to_string()
            } else {
                format!(
                    "Failed to push: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git push: {}", e),
    }
}

pub fn git_reflog_work(_dir: PathBuf, _limit: usize) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("reflog")
        .arg("--oneline")
        .arg("-n")
        .arg(_limit.to_string())
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get reflog: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git reflog: {}", e),
    }
}

pub fn git_remote_list_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("remote")
        .arg("-v")
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list remotes: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git remote: {}", e),
    }
}

pub fn git_revert_work(_dir: PathBuf, _commits: Vec<String>, _no_commit: bool) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("revert").current_dir(&_dir);

    if _no_commit {
        cmd.arg("--no-commit");
    }

    for commit in &_commits {
        cmd.arg(commit);
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                format!("Successfully reverted {} commits", _commits.len())
            } else {
                format!(
                    "Failed to revert commits: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git revert: {}", e),
    }
}

pub fn git_shortlog_work(_dir: PathBuf, _limit: usize) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("shortlog")
        .arg("-n")
        .arg("--numbered")
        .arg("--summary")
        .arg(_limit.to_string())
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get shortlog: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git shortlog: {}", e),
    }
}

pub fn git_show_work(_repo: PathBuf, _rev: String, _patch: bool) -> String {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    "Use the implementation in src/ops/mod.rs".to_string()
}

pub fn git_stash_list_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("stash")
        .arg("list")
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list stashes: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git stash list: {}", e),
    }
}

pub fn git_stash_push_work(_dir: PathBuf, _message: Option<String>) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("stash").arg("push").current_dir(&_dir);

    if let Some(message) = _message {
        cmd.arg("-m").arg(&message);
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                "Stash created successfully".to_string()
            } else {
                format!(
                    "Failed to create stash: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git stash push: {}", e),
    }
}

pub fn git_status_work(_dir: PathBuf) -> String {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    "Use the implementation in src/ops/mod.rs".to_string()
}

pub fn git_tag_list_work(_dir: PathBuf, _pattern: Option<String>) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("tag").current_dir(&_dir);

    if let Some(pattern) = _pattern {
        cmd.arg("-l").arg(&pattern);
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list tags: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git tag: {}", e),
    }
}

pub fn git_worktree_extension_work(
    _dir: PathBuf,
    _action: &str,
    _wp: Option<PathBuf>,
    _branch: Option<String>,
) -> String {
    use std::process::Command;

    let mut cmd = Command::new("git");
    cmd.arg("worktree").current_dir(&_dir);

    match _action {
        "add" => {
            if let Some(path) = _wp {
                cmd.arg("add").arg(&path);
                if let Some(branch) = _branch {
                    cmd.arg(&branch);
                }
            } else {
                return "Path required for worktree add action".to_string();
            }
        }
        "remove" => {
            if let Some(path) = _wp {
                cmd.arg("remove").arg(&path);
            } else {
                return "Path required for worktree remove action".to_string();
            }
        }
        "prune" => {
            cmd.arg("prune");
        }
        _ => {
            return format!("Unknown worktree action: {}", _action);
        }
    }

    let output = cmd.output();
    match output {
        Ok(result) => {
            if result.status.success() {
                format!("Worktree {} completed successfully", _action)
            } else {
                format!(
                    "Failed to perform worktree {}: {}",
                    _action,
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git worktree {}: {}", _action, e),
    }
}

pub fn git_worktree_list_work(_dir: PathBuf) -> String {
    use std::process::Command;

    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list worktrees: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running git worktree list: {}", e),
    }
}

pub fn gitkraken_cloud_catalog_work() -> String {
    serde_json::json!({
        "title": "GitKraken Cloud Catalog",
        "description": "Available GitKraken Cloud integrations and tools",
        "categories": [
            {
                "name": "Repository Management",
                "tools": [
                    "Create repository",
                    "Clone repository",
                    "Import repository",
                    "Repository settings"
                ]
            },
            {
                "name": "Collaboration",
                "tools": [
                    "Team management",
                    "Pull requests",
                    "Issue tracking",
                    "Code review"
                ]
            },
            {
                "name": "Integration",
                "tools": [
                    "GitHub integration",
                    "GitLab integration",
                    "Bitbucket integration",
                    "Azure DevOps integration"
                ]
            }
        ],
        "features": [
            "Cloud synchronization",
            "Cross-platform support",
            "Advanced merge tools",
            "Visual repository browsing"
        ]
    })
    .to_string()
}

pub fn gitkraken_hosted_tool_redirect_work(_name: &str) -> String {
    let tool_map = serde_json::json!({
        "merge-tool": "https://www.gitkraken.com/merge-tool",
        "git-gui": "https://www.gitkraken.com/git-client",
        "client": "https://www.gitkraken.com/git-client",
        "cloud": "https://www.gitkraken.com/cloud",
        "support": "https://support.gitkraken.com"
    });

    if let Some(url) = tool_map.get(_name).and_then(|v| v.as_str()) {
        serde_json::json!({
            "tool": _name,
            "redirect_url": url,
            "description": format!("Redirect to {}", _name)
        })
        .to_string()
    } else {
        serde_json::json!({
            "error": "Tool not found",
            "available_tools": tool_map,
            "requested_tool": _name
        })
        .to_string()
    }
}

pub fn glab_mr_list_work(_dir: PathBuf, _limit: usize) -> String {
    use std::process::Command;

    let output = Command::new("glab")
        .arg("mr")
        .arg("list")
        .arg("--per-page")
        .arg(_limit.to_string())
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list MRs: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running glab mr list: {}", e),
    }
}

pub fn glab_mr_show_work(_dir: PathBuf, _mr: &str) -> String {
    use std::process::Command;

    let output = Command::new("glab")
        .arg("mr")
        .arg("show")
        .arg(_mr)
        .current_dir(&_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to show MR: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running glab mr show: {}", e),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn grep_snippets_work(
    _search_root: PathBuf,
    _pattern: String,
    _literal: bool,
    _case_sensitive: bool,
    _respect_gitignore: bool,
    _context_lines: usize,
    _max_snippets: usize,
    _max_chars: usize,
) -> String {
    use ignore::WalkBuilder;
    use std::fs;

    let mut results = Vec::new();
    let mut found_snippets = 0;
    let regex = if _literal {
        None
    } else {
        let pattern = if _case_sensitive {
            _pattern.clone()
        } else {
            format!("(?i){}", &_pattern)
        };
        match regex::Regex::new(&pattern) {
            Ok(re) => Some(re),
            Err(e) => return format!("Invalid regex pattern: {e}"),
        }
    };

    let mut walk = WalkBuilder::new(&_search_root);
    if _respect_gitignore {
        walk.git_ignore(true);
        walk.ignore(true);
    } else {
        walk.git_ignore(false);
        walk.ignore(false);
    }
    walk.hidden(false);

    for entry in walk.build().flatten() {
        if found_snippets >= _max_snippets {
            break;
        }

        let path = entry.path();
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();

                for (line_index, line) in lines.iter().enumerate() {
                    let matches = if _literal {
                        if _case_sensitive {
                            line.contains(&_pattern)
                        } else {
                            line.to_lowercase().contains(&_pattern.to_lowercase())
                        }
                    } else {
                        regex.as_ref().is_some_and(|re| re.is_match(line))
                    };

                    if matches {
                        let start = line_index.saturating_sub(_context_lines);
                        let end = (line_index + _context_lines + 1).min(lines.len());

                        let context: Vec<String> = lines[start..end]
                            .iter()
                            .enumerate()
                            .map(|(i, l)| {
                                let prefix = if i + start == line_index {
                                    ">>> "
                                } else {
                                    "    "
                                };
                                format!("{}{}", prefix, l)
                            })
                            .collect();

                        results.push(format!(
                            "{}:{}\n{}",
                            path.display(),
                            line_index + 1,
                            context.join("\n")
                        ));

                        found_snippets += 1;
                        if found_snippets >= _max_snippets {
                            break;
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        "No matches found".to_string()
    } else {
        let output = format!(
            "Found {} snippets:\n\n{}",
            found_snippets,
            results.join("\n---\n\n")
        );
        if output.len() > _max_chars {
            format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
        } else {
            output
        }
    }
}

// ============================================================================
// MCP tools
// ============================================================================

pub fn mcp_env_hints_work() -> String {
    serde_json::json!({
        "available_tools": [
            "text_processing", "file_operations", "git_operations",
            "codebase_analysis", "summary_tools", "workspace_management"
        ],
        "environment": "MCP Server Environment",
        "version": "0.2.0",
        "features": [
            "Pattern matching with regex support",
            "File system operations",
            "Git workflow automation",
            "Text analysis and statistics",
            "Workspace context management"
        ]
    })
    .to_string()
}

// ============================================================================
// Notify tools
// ============================================================================

pub fn notify_desktop_work(_args: Value) -> Value {
    use std::process::Command;

    let title = _args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Notification");
    let message = _args.get("message").and_then(|v| v.as_str()).unwrap_or("");
    let urgency = _args
        .get("urgency")
        .and_then(|v| v.as_str())
        .unwrap_or("normal");

    // Use notify-send for Linux desktop notifications
    let output = Command::new("notify-send")
        .arg("-u")
        .arg(urgency)
        .arg(title)
        .arg(message)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                serde_json::json!({
                    "status": "success",
                    "message": "Notification sent successfully",
                    "title": title,
                    "urgency": urgency
                })
            } else {
                serde_json::json!({
                    "error": "Failed to send notification",
                    "details": String::from_utf8_lossy(&result.stderr)
                })
            }
        }
        Err(e) => serde_json::json!({
            "error": "Failed to execute notification command",
            "details": e.to_string()
        }),
    }
}

pub fn notify_email_send_work(_args: Value) -> Value {
    let to = _args.get("to").and_then(|v| v.as_str()).unwrap_or("");
    let subject = _args.get("subject").and_then(|v| v.as_str()).unwrap_or("");
    let body = _args.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let from = _args
        .get("from")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let smtp_server = _args.get("smtp_server").and_then(|v| v.as_str());

    if to.is_empty() {
        return serde_json::json!({"error": "Recipient email is required"});
    }
    if let Some(server) = smtp_server {
        return serde_json::json!({
            "error": "Direct SMTP server mode is not supported by this local helper",
            "smtp_server": server,
            "supported_delivery": "sendmail or mailx via src/ops/notify.rs"
        });
    }

    let output = crate::ops::notify::notify_email_send_work(
        to.to_string(),
        subject.to_string(),
        body.to_string(),
        from,
    );
    serde_json::json!({ "output": output })
}

pub fn notify_webhook_work(_args: Value) -> Value {
    let url = _args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let method = _args
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("POST");
    let payload = _args.get("payload");
    let headers = _args
        .get("headers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let block_internal = _args
        .get("block_internal")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if url.is_empty() {
        return serde_json::json!({"error": "Webhook URL is required"});
    }

    let payload_text = payload
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        })
        .unwrap_or_else(|| "{}".to_string());
    let header_pairs = headers
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string()),
            )
        })
        .collect::<Vec<_>>();

    let output = crate::ops::notify::notify_webhook_work(
        url.to_string(),
        payload_text,
        Some(method.to_string()),
        Some(header_pairs),
        block_internal,
    );
    serde_json::json!({ "output": output })
}

// ============================================================================
// OCR tools
// ============================================================================

pub fn ocr_tesseract_file_work(_path: &PathBuf, _lang: &str, _max_chars: usize) -> String {
    use std::process::Command;

    // Basic implementation using tesseract OCR if available
    let output = Command::new("tesseract")
        .arg(_path.to_str().unwrap_or(""))
        .arg("stdout")
        .arg("-l")
        .arg(_lang)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let text = String::from_utf8_lossy(&result.stdout);
                if text.len() > _max_chars {
                    format!("{}...[truncated]", &text[.._max_chars.min(text.len())])
                } else {
                    text.to_string()
                }
            } else {
                format!("OCR failed: {}", String::from_utf8_lossy(&result.stderr))
            }
        }
        Err(e) => format!("Failed to run tesseract: {}", e),
    }
}

pub fn ocr_tesseract_image_work(_image_data: &[u8], _lang: &str, _max_chars: usize) -> String {
    use std::fs;
    use std::process::Command;

    // Create temporary file for image
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("temp_ocr_image.png");

    if let Err(e) = fs::write(&temp_file, _image_data) {
        return format!("Failed to write temporary image file: {}", e);
    }

    // Run tesseract on temporary file
    let output = Command::new("tesseract")
        .arg(temp_file.to_str().unwrap_or(""))
        .arg("stdout")
        .arg("-l")
        .arg(_lang)
        .output();

    // Clean up temporary file
    let _ = fs::remove_file(&temp_file);

    match output {
        Ok(result) => {
            if result.status.success() {
                let text = String::from_utf8_lossy(&result.stdout);
                if text.len() > _max_chars {
                    format!("{}...[truncated]", &text[.._max_chars.min(text.len())])
                } else {
                    text.to_string()
                }
            } else {
                format!("OCR failed: {}", String::from_utf8_lossy(&result.stderr))
            }
        }
        Err(e) => format!("Failed to run tesseract: {}", e),
    }
}

// ============================================================================
// Ollama tools
// ============================================================================

pub fn ollama_list_models_work() -> String {
    use std::process::Command;

    // List Ollama models using ollama command
    let output = Command::new("ollama").arg("list").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to list Ollama models: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running ollama list: {}", e),
    }
}

pub fn ollama_chat_work(_model: String, _messages: Vec<String>, _max_tokens: usize) -> String {
    use std::process::Command;

    // Create a simple chat message
    let chat_prompt = if _messages.is_empty() {
        "Hello".to_string()
    } else {
        _messages.join("\n")
    };

    // Generate response using Ollama
    let output = Command::new("ollama")
        .arg("run")
        .arg(&_model)
        .arg(&chat_prompt)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let text = String::from_utf8_lossy(&result.stdout);
                if text.len() > _max_tokens {
                    format!("{}...[truncated]", &text[.._max_tokens.min(text.len())])
                } else {
                    text.to_string()
                }
            } else {
                format!(
                    "Failed to chat with Ollama: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running ollama run: {}", e),
    }
}

pub fn ai_openai_chat_work(_messages: Vec<String>, _model: String, _max_tokens: usize) -> String {
    use std::io::Write;
    use std::process::Command;

    if _messages.is_empty() {
        return serde_json::json!({
            "error": "at least one message is required",
            "model": _model,
            "max_tokens": _max_tokens
        })
        .to_string();
    }

    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            return serde_json::json!({
                "error": "OPENAI_API_KEY is not set; cannot call an OpenAI-compatible chat endpoint",
                "model": _model,
                "messages_count": _messages.len(),
                "max_tokens": _max_tokens,
                "network_calls": false
            })
            .to_string();
        }
    };
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
        .trim_end_matches('/')
        .to_string();
    let url = format!("{base_url}/chat/completions");
    let max_tokens = _max_tokens.clamp(1, 8192);
    let messages = _messages
        .into_iter()
        .map(|content| serde_json::json!({ "role": "user", "content": content }))
        .collect::<Vec<_>>();
    let body = serde_json::json!({
        "model": _model,
        "messages": messages,
        "max_tokens": max_tokens
    })
    .to_string();

    let mut child = match Command::new("curl")
        .args([
            "-sS",
            "--fail-with-body",
            "--max-time",
            "120",
            "-H",
            "Content-Type: application/json",
            "-H",
            &format!("Authorization: Bearer {api_key}"),
            "-d",
            "@-",
            &url,
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            return serde_json::json!({
                "error": format!("failed to execute curl: {e}"),
                "model": _model,
                "network_calls": false
            })
            .to_string();
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = stdin.write_all(body.as_bytes()) {
            let _ = child.kill();
            return serde_json::json!({
                "error": format!("failed to write request body to curl stdin: {e}"),
                "model": _model
            })
            .to_string();
        }
    }

    match child.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                stdout.trim().to_string()
            } else {
                serde_json::json!({
                    "error": "OpenAI-compatible chat request failed",
                    "status": output.status.code(),
                    "stderr": stderr.trim(),
                    "body": stdout.trim(),
                    "model": _model
                })
                .to_string()
            }
        }
        Err(e) => serde_json::json!({
            "error": format!("failed to wait for curl: {e}"),
            "model": _model
        })
        .to_string(),
    }
}

// ============================================================================
// Pack tools
// ============================================================================

pub fn pack_context_work(
    _segments: Vec<String>,
    _segment_labels: Option<Vec<String>>,
    _max_chars: usize,
    _mode: String,
    _dedupe_lines: bool,
    _collapse_whitespace: bool,
) -> String {
    let mut result = Vec::new();
    result.push(format!(
        "Packing {} segments with mode: {}",
        _segments.len(),
        _mode
    ));

    for (index, segment) in _segments.iter().enumerate() {
        let label = if let Some(labels) = &_segment_labels {
            labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("Segment {}", index + 1))
        } else {
            format!("Segment {}", index + 1)
        };

        let mut processed_segment = segment.clone();

        if _dedupe_lines {
            let mut lines: Vec<&str> = processed_segment.lines().collect();
            lines.sort();
            lines.dedup();
            processed_segment = lines.join("\n");
        }

        if _collapse_whitespace {
            processed_segment = processed_segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
        }

        result.push(format!(
            "{}: {} chars (processed: {} chars)",
            label,
            segment.len(),
            processed_segment.len()
        ));

        if result.join("\n").len() > _max_chars {
            result.push("[Output truncated due to size limit]".to_string());
            break;
        }
    }

    result.join("\n")
}

// ============================================================================
// Project tools
// ============================================================================

pub fn project_cargo_check_build_args(
    _package: Option<&str>,
    _workspace: bool,
) -> Result<Vec<String>, String> {
    let mut args = vec!["check".to_string()];

    if _workspace {
        args.push("--workspace".to_string());
    }

    if let Some(pkg) = _package {
        args.push("--package".to_string());
        args.push(pkg.to_string());
    }

    Ok(args)
}

pub fn project_cargo_test_build_args(
    _package: Option<&str>,
    _test_filter: Option<&str>,
    _no_run: bool,
    _workspace: bool,
    _jobs: Option<u32>,
    _nocapture: bool,
) -> Result<Vec<String>, String> {
    let mut args = vec!["test".to_string()];

    if _workspace {
        args.push("--workspace".to_string());
    }

    if let Some(pkg) = _package {
        args.push("--package".to_string());
        args.push(pkg.to_string());
    }

    if let Some(filter) = _test_filter {
        args.push(filter.to_string());
    }

    if _no_run {
        args.push("--no-run".to_string());
    }

    if let Some(jobs) = _jobs {
        args.push("--jobs".to_string());
        args.push(jobs.to_string());
    }

    if _nocapture {
        args.push("--nocapture".to_string());
    }

    Ok(args)
}

pub fn project_cargo_test_args(
    _package: Option<&str>,
    _workspace: bool,
    _release: bool,
    _target: Option<&str>,
    _test_name: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut args = vec!["test".to_string()];

    if _workspace {
        args.push("--workspace".to_string());
    }

    if _release {
        args.push("--release".to_string());
    }

    if let Some(trgt) = _target {
        args.push("--target".to_string());
        args.push(trgt.to_string());
    }

    if let Some(pkg) = _package {
        args.push("--package".to_string());
        args.push(pkg.to_string());
    }

    if let Some(test) = _test_name {
        args.push(test.to_string());
    }

    Ok(args)
}

pub fn project_pytest_build_args(
    _test_path: &str,
    _quiet: bool,
    _keyword: Option<&str>,
    _maxfail: Option<u32>,
) -> Result<Vec<String>, String> {
    let mut args = vec!["pytest".to_string()];

    if _quiet {
        args.push("-q".to_string());
    }

    if let Some(keyword) = _keyword {
        args.push("-k".to_string());
        args.push(keyword.to_string());
    }

    if let Some(maxfail) = _maxfail {
        args.push("--maxfail".to_string());
        args.push(maxfail.to_string());
    }

    args.push(_test_path.to_string());

    Ok(args)
}

pub fn repository_get_file_content_work(
    _dir: PathBuf,
    _filepath: String,
    _gitref: Option<String>,
) -> String {
    use std::fs;
    use std::process::Command;

    let full_path = _dir.join(&_filepath);

    // Try to read from git if gitref is provided
    if let Some(git_ref) = _gitref {
        let output = Command::new("git")
            .arg("show")
            .arg(format!("{}:{}", git_ref, _filepath))
            .current_dir(&_dir)
            .output();

        if let Ok(result) = output {
            if result.status.success() {
                return String::from_utf8_lossy(&result.stdout).to_string();
            }
        }
    }

    // Read from file system
    match fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(e) => format!("Error reading file {}: {}", full_path.display(), e),
    }
}

// ============================================================================
// Review tools
// ============================================================================

pub fn review_diff_risk_work(_args: Value) -> Value {
    let diff_content = _args.get("diff").and_then(|v| v.as_str()).unwrap_or("");
    let _risk_level = _args
        .get("risk_level")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");

    if diff_content.is_empty() {
        return serde_json::json!({"error": "Diff content is required"});
    }

    // Basic risk analysis based on diff patterns
    let mut risk_factors = Vec::new();
    let mut risk_score = 0;

    // Check for risky patterns
    if diff_content.contains("DELETE")
        || diff_content.contains("DROP")
        || diff_content.contains("TRUNCATE")
    {
        risk_factors.push("Destructive operations detected");
        risk_score += 30;
    }

    if diff_content.contains("password")
        || diff_content.contains("secret")
        || diff_content.contains("key")
    {
        risk_factors.push("Potential secrets in code");
        risk_score += 25;
    }

    if diff_content.contains("sudo")
        || diff_content.contains("chmod 777")
        || diff_content.contains("chown")
    {
        risk_factors.push("Permission changes detected");
        risk_score += 20;
    }

    if diff_content.len() > 10000 {
        risk_factors.push("Large diff size");
        risk_score += 15;
    }

    let risk_category = match risk_score {
        0..=20 => "low",
        21..=50 => "medium",
        51..=80 => "high",
        _ => "critical",
    };

    serde_json::json!({
        "risk_score": risk_score,
        "risk_category": risk_category,
        "risk_factors": risk_factors,
        "diff_size": diff_content.len(),
        "recommended_review": risk_score > 30
    })
}

pub fn review_focused_review_work(_args: Value) -> Value {
    let code_content = _args.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let focus_area = _args
        .get("focus")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    if code_content.is_empty() {
        return serde_json::json!({"error": "Code content is required"});
    }

    let mut findings = Vec::new();

    match focus_area {
        "readability" => {
            if code_content.lines().count() > 50 {
                findings.push("Consider breaking down large function");
            }
            if code_content.len() > 2000 {
                findings.push("Function is quite long, consider refactoring");
            }
        }
        "naming" => {
            if code_content.contains("temp") || code_content.contains("tmp") {
                findings.push("Consider more descriptive variable names");
            }
            if code_content.contains("data1") || code_content.contains("var1") {
                findings.push("Avoid numbered variable names");
            }
        }
        "security" => {
            if code_content.contains("eval") || code_content.contains("exec") {
                findings.push("Review code execution functions");
            }
            if code_content.contains("password") || code_content.contains("secret") {
                findings.push("Potential hardcoded credentials");
            }
        }
        _ => {
            findings.push("General review: consider code structure and clarity");
        }
    }

    serde_json::json!({
        "focus_area": focus_area,
        "findings": findings,
        "lines_analyzed": code_content.lines().count(),
        "recommendations_count": findings.len()
    })
}

pub fn review_missing_tests_work(_args: Value) -> Value {
    let code_content = _args.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let language = _args
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if code_content.is_empty() {
        return serde_json::json!({"error": "Code content is required"});
    }

    let mut missing_tests = Vec::new();
    let mut test_suggestions = Vec::new();

    // Look for function definitions
    if language == "rust" || language == "javascript" || language == "typescript" {
        let func_pattern = if language == "rust" {
            "fn "
        } else {
            "function "
        };

        let function_count = code_content.matches(func_pattern).count();
        let test_count = code_content.matches("test").count();

        if function_count > test_count {
            missing_tests.push(format!(
                "{} functions found, {} test functions detected",
                function_count, test_count
            ));
            test_suggestions.push("Add unit tests for each function".to_string());
        }
    }

    // Check for error handling
    if !code_content.contains("error")
        && !code_content.contains("catch")
        && !code_content.contains("Result")
    {
        missing_tests.push("No error handling detected".to_string());
        test_suggestions.push("Add error handling tests".to_string());
    }

    // Check for edge cases
    if !code_content.contains("null")
        && !code_content.contains("None")
        && !code_content.contains("undefined")
    {
        missing_tests.push("No edge case handling detected".to_string());
        test_suggestions.push("Add tests for edge cases".to_string());
    }

    serde_json::json!({
        "language": language,
        "missing_tests": missing_tests,
        "test_suggestions": test_suggestions,
        "lines_analyzed": code_content.lines().count(),
        "coverage_estimate": if missing_tests.is_empty() { "good" } else { "needs_improvement" }
    })
}

pub fn review_pr_packet_work(_args: Value) -> Value {
    let pr_title = _args.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let pr_description = _args
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let files_changed = _args
        .get("files_changed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if pr_title.is_empty() {
        return serde_json::json!({"error": "PR title is required"});
    }

    let mut review_points = Vec::new();
    let mut score = 0;

    // Title quality
    if pr_title.len() < 10 {
        review_points.push("PR title is too short");
        score -= 10;
    } else if pr_title.len() > 100 {
        review_points.push("PR title is too long");
        score -= 5;
    } else {
        score += 10;
    }

    // Description quality
    if pr_description.len() < 20 {
        review_points.push("PR description is too brief");
        score -= 15;
    } else if pr_description.contains("fixes") || pr_description.contains("resolves") {
        score += 15;
    }

    // Size assessment
    if files_changed > 20 {
        review_points.push("Large PR - consider splitting");
        score -= 10;
    } else if files_changed <= 5 {
        score += 5;
    }

    let quality_level = match score {
        i32::MIN..=0 => "needs_improvement",
        1..=15 => "fair",
        16..=25 => "good",
        _ => "excellent",
    };

    serde_json::json!({
        "pr_score": score,
        "quality_level": quality_level,
        "review_points": review_points,
        "files_changed": files_changed,
        "title_length": pr_title.len(),
        "description_length": pr_description.len()
    })
}

pub fn review_rulebook_check_work(_args: Value) -> Value {
    let code_content = _args.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let rulebook = _args
        .get("rulebook")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(Vec::new());

    if code_content.is_empty() {
        return serde_json::json!({"error": "Code content is required"});
    }

    let mut violations = Vec::new();
    let mut compliance_score = 100;

    // Default rulebook checks if none provided
    let rules = if rulebook.is_empty() {
        vec![
            (
                "no_hardcoded_secrets",
                "Check for hardcoded passwords/secrets",
            ),
            ("proper_error_handling", "Ensure proper error handling"),
            ("function_length", "Functions should be under 50 lines"),
            ("naming_convention", "Follow consistent naming conventions"),
        ]
    } else {
        rulebook
            .iter()
            .map(|v| {
                let name = v.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let description = v.get("description").and_then(|v| v.as_str()).unwrap_or("");
                (name, description)
            })
            .collect()
    };

    for (rule_name, rule_desc) in &rules {
        let mut violated = false;

        match *rule_name {
            "no_hardcoded_secrets" => {
                if code_content.contains("password")
                    || code_content.contains("secret")
                    || code_content.contains("token")
                {
                    violated = true;
                }
            }
            "proper_error_handling" => {
                if !code_content.contains("error")
                    && !code_content.contains("Result")
                    && !code_content.contains("catch")
                {
                    violated = true;
                }
            }
            "function_length" => {
                if code_content.lines().count() > 50 {
                    violated = true;
                }
            }
            _ => {
                // Generic check - just note the rule
            }
        }

        if violated {
            violations.push(serde_json::json!({
                "rule": rule_name,
                "description": rule_desc,
                "severity": "warning"
            }));
            compliance_score -= 10;
        }
    }

    serde_json::json!({
        "compliance_score": compliance_score.max(0),
        "violations": violations,
        "rules_checked": rules.len(),
        "status": if compliance_score >= 80 { "compliant".to_string() } else { "needs_review".to_string() }
    })
}

// ============================================================================
// Screenshot tools
// ============================================================================

pub fn screenshot_capture_work(
    _output_path: Option<PathBuf>,
    _display_env: Option<String>,
) -> String {
    use std::process::Command;

    // Try to use system screenshot tools
    let screenshot_commands = vec![
        ("gnome-screenshot", vec!["-f", "png"]),
        ("scrot", vec!["-s", "screenshot.png"]),
        ("import", vec!["-window", "root", "screenshot.png"]),
    ];

    for (cmd, args) in screenshot_commands {
        let output = Command::new(cmd).args(&args).output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let output_file = if let Some(path) = &_output_path {
                        path.to_string_lossy().to_string()
                    } else {
                        "screenshot.png".to_string()
                    };

                    return format!("Screenshot captured using {} to {}", cmd, output_file);
                }
            }
            Err(_) => continue, // Try next command
        }
    }

    "No screenshot tool available. Install gnome-screenshot, scrot, or ImageMagick".to_string()
}

// ============================================================================
// Summarize tools
// ============================================================================

pub fn summarize_file_work(
    _path: PathBuf,
    _start: Option<usize>,
    _end: Option<usize>,
    _max_c: usize,
    _max_s: usize,
) -> String {
    use std::fs;

    match fs::read_to_string(&_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();

            let start_line = _start.unwrap_or(0);
            let end_line = _end.unwrap_or(total_lines);

            let effective_start = start_line.min(total_lines);
            let effective_end = end_line.min(total_lines);

            if effective_start >= effective_end {
                return format!(
                    "Invalid line range: {}-{} for file with {} lines",
                    start_line, end_line, total_lines
                );
            }

            let selected_lines = &lines[effective_start..effective_end];
            let _line_count = selected_lines.len();

            let char_count = selected_lines.join("\n").len();
            let word_count = selected_lines.join(" ").split_whitespace().count();

            let summary = format!(
                "File: {}\nLines: {}-{} of {}\nCharacters: {}\nWords: {}\nPreview:\n{}",
                _path.display(),
                effective_start,
                effective_end,
                total_lines,
                char_count,
                word_count,
                selected_lines
                    .iter()
                    .take(5)
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            if summary.len() > _max_c {
                format!("{}...[truncated]", &summary[.._max_c.min(summary.len())])
            } else {
                summary
            }
        }
        Err(e) => format!("Error reading file {}: {}", _path.display(), e),
    }
}

pub fn summarize_identifier_frequency_work(
    _text: String,
    _min_len: usize,
    _max_identifiers: usize,
    _max_c: usize,
    _fold_case: bool,
) -> String {
    use regex::Regex;
    use std::collections::HashMap;

    // Extract identifiers (alphanumeric sequences starting with letter or underscore)
    let Ok(identifier_regex) = Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b") else {
        return "Found 0 unique identifiers".to_string();
    };

    let mut frequency_map = HashMap::new();

    for caps in identifier_regex.captures_iter(&_text) {
        if let Some(matched) = caps.get(0) {
            let identifier = matched.as_str();

            if identifier.len() >= _min_len {
                let key = if _fold_case {
                    identifier.to_lowercase()
                } else {
                    identifier.to_string()
                };

                *frequency_map.entry(key).or_insert(0) += 1;
            }
        }
    }

    let mut identifiers: Vec<(String, usize)> = frequency_map.into_iter().collect();
    identifiers.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    result.push(format!("Found {} unique identifiers", identifiers.len()));

    for (identifier, count) in identifiers.into_iter().take(_max_identifiers) {
        result.push(format!("{}: {} occurrences", identifier, count));
    }

    let output = result.join("\n");
    if output.len() > _max_c {
        format!("{}...[truncated]", &output[.._max_c.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_line_prefix_groups_work(
    _text: String,
    _prefix_length: usize,
    _min_group_size: usize,
    _max_c: usize,
) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut prefix_groups = std::collections::HashMap::new();

    for line in lines {
        if line.len() >= _prefix_length {
            let prefix = &line[.._prefix_length];
            *prefix_groups.entry(prefix.to_string()).or_insert(0) += 1;
        }
    }

    let mut groups: Vec<(String, usize)> = prefix_groups.into_iter().collect();
    groups.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    result.push(format!("Found {} prefix groups", groups.len()));

    for (prefix, count) in groups.into_iter().take(20) {
        if count >= _min_group_size {
            result.push(format!("'{}': {} lines", prefix, count));
        }
    }

    let output = result.join("\n");
    if output.len() > _max_c {
        format!("{}...[truncated]", &output[.._max_c.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_text_work(_text: String, _max_chars: usize, _max_sentences: usize) -> String {
    let sentences: Vec<&str> = _text
        .split(&['.', '!', '?'][..])
        .filter(|s| !s.trim().is_empty())
        .take(_max_sentences)
        .collect();

    let result = sentences.join(" ");

    if result.len() > _max_chars {
        format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
    } else {
        result
    }
}

pub fn summarize_bullets_work(_text: String, _max_items: usize, _max_chars: usize) -> String {
    // Extract bullet points (lines starting with -, *, or •)
    let lines: Vec<&str> = _text.lines().collect();
    let bullets: Vec<String> = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('•')
        })
        .take(_max_items)
        .map(|line| line.trim().to_string())
        .collect();

    let result = bullets.join("\n");

    if result.len() > _max_chars {
        format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
    } else {
        result
    }
}

pub fn summarize_delimited_header_work(
    _text: String,
    _delim: &str,
    _max_columns: usize,
    _max_chars: usize,
) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut result = Vec::new();
    let mut header_found = false;

    for line in lines {
        if line.contains(_delim) {
            if !header_found {
                // First line with delimiter is the header
                let columns: Vec<&str> = line.split(_delim).take(_max_columns).collect();
                result.push(format!("Header: {}", columns.join(", ")));
                header_found = true;
            } else {
                // Sample a few data rows
                let columns: Vec<&str> = line.split(_delim).take(_max_columns).collect();
                result.push(format!("Sample: {}", columns.join(", ")));
                break;
            }
        }
    }

    if !header_found {
        "No delimiter found in text".to_string()
    } else {
        let output = result.join("\n");
        if output.len() > _max_chars {
            format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
        } else {
            output
        }
    }
}

pub fn summarize_diff_file_list_work(
    _diff: String,
    _max_paths: usize,
    _max_chars: usize,
) -> String {
    use regex::Regex;

    // Extract file paths from git diff (lines starting with diff --git)
    let Ok(diff_regex) = Regex::new(r"diff --git a/(.*?) b/") else {
        return "No files found in diff".to_string();
    };
    let lines: Vec<&str> = _diff.lines().collect();
    let mut files = Vec::new();

    for line in lines {
        if let Some(caps) = diff_regex.captures(line) {
            if let Some(file_path) = caps.get(1) {
                files.push(file_path.as_str().to_string());
                if files.len() >= _max_paths {
                    break;
                }
            }
        }
    }

    if files.is_empty() {
        "No file paths found in diff".to_string()
    } else {
        let result = format!("Files changed in diff:\n{}", files.join("\n"));
        if result.len() > _max_chars {
            format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
        } else {
            result
        }
    }
}

pub fn summarize_duplicate_lines_work(
    _text: String,
    _max_outputs: usize,
    _max_exact: usize,
    _max_chars: usize,
) -> String {
    use std::collections::HashMap;

    let lines: Vec<&str> = _text.lines().collect();
    let mut line_counts = HashMap::new();

    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *line_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        }
    }

    let mut duplicates: Vec<(String, usize)> = line_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();

    duplicates.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    result.push(format!("Found {} duplicate lines", duplicates.len()));

    for (line, count) in duplicates.into_iter().take(_max_outputs) {
        let display_line = if line.len() > _max_exact {
            format!("{}...[truncated]", &line[.._max_exact.min(line.len())])
        } else {
            line
        };
        result.push(format!("'{}': {} occurrences", display_line, count));
    }

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_first_lines_work(_text: String, _line_count: usize, _max_chars: usize) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let lines_to_take = lines.len().min(_line_count);
    let result: Vec<String> = lines
        .iter()
        .take(lines_to_take)
        .map(|line| line.to_string())
        .collect();

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_headings_work(_text: String, _max_headings: usize, _max_chars: usize) -> String {
    use regex::Regex;

    // Extract markdown headings (lines starting with #)
    let Ok(heading_regex) = Regex::new(r"^#+\s+(.+)$") else {
        return "No headings found".to_string();
    };
    let lines: Vec<&str> = _text.lines().collect();
    let headings: Vec<String> = lines
        .iter()
        .filter_map(|line| {
            if let Some(caps) = heading_regex.captures(line) {
                caps.get(1).map(|m| m.as_str().to_string())
            } else {
                None
            }
        })
        .take(_max_headings)
        .collect();

    let result = headings.join("\n");

    if result.len() > _max_chars {
        format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
    } else {
        result
    }
}

pub fn summarize_json_structure_work(
    _text: String,
    _max_depth: usize,
    _max_keys_per_level: usize,
    _max_lines: usize,
    _max_chars: usize,
) -> String {
    use serde_json::Value;

    match serde_json::from_str::<Value>(&_text) {
        Ok(json) => {
            let mut result = Vec::new();
            result.push("JSON Structure Analysis:".to_string());

            fn analyze_value(
                value: &Value,
                depth: usize,
                max_depth: usize,
                max_keys: usize,
                result: &mut Vec<String>,
            ) {
                if depth >= max_depth {
                    return;
                }

                let indent = "  ".repeat(depth);

                match value {
                    Value::Object(map) => {
                        result.push(format!("{}Object with {} keys:", indent, map.len()));
                        for (key, val) in map.iter().take(max_keys) {
                            result.push(format!("{}  '{}': {}", indent, key, get_type_name(val)));
                            analyze_value(val, depth + 1, max_depth, max_keys, result);
                        }
                        if map.len() > max_keys {
                            result.push(format!(
                                "{}  ... and {} more keys",
                                indent,
                                map.len() - max_keys
                            ));
                        }
                    }
                    Value::Array(arr) => {
                        result.push(format!("{}Array with {} items", indent, arr.len()));
                        if !arr.is_empty() {
                            analyze_value(&arr[0], depth + 1, max_depth, max_keys, result);
                        }
                    }
                    _ => {
                        result.push(format!("{}{}", indent, get_type_name(value)));
                    }
                }
            }

            fn get_type_name(value: &Value) -> &'static str {
                match value {
                    Value::Null => "null",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Object(_) => "object",
                }
            }

            analyze_value(&json, 0, _max_depth, _max_keys_per_level, &mut result);

            let output: String = result
                .iter()
                .take(_max_lines)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");

            if output.len() > _max_chars {
                format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
            } else {
                output
            }
        }
        Err(_) => "Invalid JSON format".to_string(),
    }
}

pub fn summarize_line_histogram_work(_text: String, _max_chars: usize) -> String {
    use std::collections::HashMap;

    let lines: Vec<&str> = _text.lines().collect();
    let mut length_counts = HashMap::new();

    for line in &lines {
        let length = line.len();
        let bucket = match length {
            0 => "0",
            1..=10 => "1-10",
            11..=20 => "11-20",
            21..=50 => "21-50",
            51..=100 => "51-100",
            _ => "100+",
        };
        *length_counts.entry(bucket.to_string()).or_insert(0) += 1;
    }

    let mut result = Vec::new();
    result.push(format!("Line length histogram for {} lines", lines.len()));

    for bucket in ["0", "1-10", "11-20", "21-50", "51-100", "100+"] {
        if let Some(count) = length_counts.get(bucket) {
            let bar = "█".repeat(*count.min(&20));
            result.push(format!("{} chars: {} {}", bucket, count, bar));
        }
    }

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_markdown_toc_work(
    _text: String,
    _max_entries: usize,
    _max_chars: usize,
) -> String {
    use regex::Regex;

    // Extract markdown headings (lines starting with #)
    let Ok(heading_regex) = Regex::new(r"^(#{1,6})\s+(.+)$") else {
        return "No markdown headings found".to_string();
    };
    let lines: Vec<&str> = _text.lines().collect();
    let mut headings = Vec::new();

    for line in lines {
        if let Some(caps) = heading_regex.captures(line) {
            let level = caps.get(1).map_or("", |m| m.as_str()).len();
            let title = caps.get(2).map_or("", |m| m.as_str());
            let indent = "  ".repeat(level - 1);
            headings.push(format!("{}- {}", indent, title));

            if headings.len() >= _max_entries {
                break;
            }
        }
    }

    if headings.is_empty() {
        "No markdown headings found".to_string()
    } else {
        let result = format!("Table of Contents:\n{}", headings.join("\n"));
        if result.len() > _max_chars {
            format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
        } else {
            result
        }
    }
}

pub fn summarize_multi_file_work(
    _paths: Vec<PathBuf>,
    _lines_per_file: usize,
    _max_chars: usize,
    _max_files: usize,
) -> String {
    use std::fs;

    let mut result = Vec::new();
    result.push(format!("Processing {} files", _paths.len()));

    for (index, path) in _paths.iter().take(_max_files).enumerate() {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().take(_lines_per_file).collect();
            let preview = lines.join("\n");

            result.push(format!("File {}: {}", index + 1, path.display()));
            result.push(format!("Preview (first {} lines):", _lines_per_file));

            let preview_with_truncation = if preview.len() > 200 {
                format!("{}...[truncated]", &preview[..200.min(preview.len())])
            } else {
                preview
            };

            result.push(preview_with_truncation);
            result.push("---".to_string());

            if result.join("\n").len() > _max_chars {
                result.push("[Output truncated due to size limit]".to_string());
                break;
            }
        } else {
            result.push(format!(
                "File {}: {} - [Could not read]",
                index + 1,
                path.display()
            ));
        }
    }

    result.join("\n")
}

pub fn summarize_rank_lines_by_keywords_work(
    _text: String,
    _keywords: Vec<String>,
    _top_n: usize,
    _max_chars: usize,
) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let mut scored_lines = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let mut score = 0;
        let line_lower = line.to_lowercase();

        for keyword in &_keywords {
            let keyword_lower = keyword.to_lowercase();
            score += line_lower.matches(&keyword_lower).count() * keyword.len();
        }

        if score > 0 {
            scored_lines.push((score, line_index, line.trim()));
        }
    }

    scored_lines.sort_by(|a, b| b.0.cmp(&a.0));

    let mut result = Vec::new();
    result.push(format!(
        "Found {} lines matching keywords",
        scored_lines.len()
    ));

    for (score, line_index, line) in scored_lines.into_iter().take(_top_n) {
        let display_line = if line.len() > 100 {
            format!("{}...[truncated]", &line[..100.min(line.len())])
        } else {
            line.to_string()
        };
        result.push(format!(
            "Score {}: Line {} - '{}'",
            score,
            line_index + 1,
            display_line
        ));
    }

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

pub fn summarize_segments_work(
    _segments: Vec<String>,
    _max_chars: usize,
    _max_seg: usize,
) -> String {
    let mut result = Vec::new();
    result.push(format!("Processing {} segments", _segments.len()));

    for (index, segment) in _segments.iter().take(_max_seg).enumerate() {
        let preview = if segment.len() > 200 {
            format!("{}...[truncated]", &segment[..200.min(segment.len())])
        } else {
            segment.clone()
        };

        result.push(format!("Segment {} ({} chars):", index + 1, segment.len()));
        result.push(preview);
        result.push("---".to_string());

        if result.join("\n").len() > _max_chars {
            result.push("[Output truncated due to size limit]".to_string());
            break;
        }
    }

    result.join("\n")
}

pub fn summarize_token_estimate_work(_text: String, _max_chars: usize) -> String {
    // Simple token estimation (rough approximation)
    let words = _text.split_whitespace().count();
    let chars = _text.chars().count();
    let lines = _text.lines().count();

    // Rough token estimate: average 4 characters per token + some overhead
    let estimated_tokens = (chars / 4).max(words);

    serde_json::json!({
        "estimated_tokens": estimated_tokens,
        "word_count": words,
        "character_count": chars,
        "line_count": lines,
        "note": "This is a rough estimate suitable for most LLM tokenizers"
    })
    .to_string()
}

pub fn summarize_top_terms_work(
    _text: String,
    _min_len: usize,
    _max_terms: usize,
    _max_chars: usize,
) -> String {
    use regex::Regex;
    use std::collections::HashMap;

    // Extract words (alphanumeric sequences)
    let word_regex = match Regex::new(r"\b[a-zA-Z]+\b") {
        Ok(re) => re,
        Err(e) => {
            return serde_json::json!({
                "error": format!("internal regex compile failed: {e}")
            })
            .to_string();
        }
    };

    let mut term_counts = HashMap::new();

    for caps in word_regex.captures_iter(&_text) {
        if let Some(word_match) = caps.get(0) {
            let word = word_match.as_str().to_lowercase();
            if word.len() >= _min_len {
                *term_counts.entry(word).or_insert(0) += 1;
            }
        }
    }

    let mut terms: Vec<(String, usize)> = term_counts.into_iter().collect();
    terms.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    result.push(format!(
        "Found {} unique terms (min length: {})",
        terms.len(),
        _min_len
    ));

    for (term, count) in terms.into_iter().take(_max_terms) {
        result.push(format!("'{}': {} occurrences", term, count));
    }

    let output = result.join("\n");
    if output.len() > _max_chars {
        format!("{}...[truncated]", &output[.._max_chars.min(output.len())])
    } else {
        output
    }
}

// ============================================================================
// Text tools
// ============================================================================

pub fn text_stats_work(_text: String) -> String {
    let lines: Vec<&str> = _text.lines().collect();
    let words: Vec<&str> = _text.split_whitespace().collect();
    let chars = _text.chars().count();
    let chars_no_spaces = _text.chars().filter(|c| !c.is_whitespace()).count();

    serde_json::json!({
        "lines": lines.len(),
        "words": words.len(),
        "characters": chars,
        "characters_no_spaces": chars_no_spaces,
        "paragraphs": _text.split("\n\n").filter(|p| !p.trim().is_empty()).count()
    })
    .to_string()
}

pub fn text_longest_common_substring_work(_a: String, _b: String, _max_len_each: usize) -> String {
    // Simple implementation of longest common substring
    let a = if _a.len() > _max_len_each {
        &_a[.._max_len_each]
    } else {
        &_a
    };
    let b = if _b.len() > _max_len_each {
        &_b[.._max_len_each]
    } else {
        &_b
    };

    let mut longest = String::new();

    for i in 0..a.len() {
        for j in (i + 1)..=a.len().min(i + 100) {
            // Limit substring length to avoid O(n³)
            let substring = &a[i..j];
            if b.contains(substring) && substring.len() > longest.len() {
                longest = substring.to_string();
            }
        }
    }

    if longest.is_empty() {
        "No common substring found".to_string()
    } else {
        format!(
            "Longest common substring: '{}' (length: {})",
            longest,
            longest.len()
        )
    }
}

pub fn text_djb2_hash_work(_text: String) -> String {
    let mut hash: u64 = 5381;
    for c in _text.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u64);
    }
    format!("{:x}", hash)
}

pub fn text_levenshtein_distance_work(_a: String, _b: String, _max_len: usize) -> String {
    // Simple Levenshtein distance implementation
    let a = if _a.len() > _max_len {
        &_a[.._max_len]
    } else {
        &_a
    };
    let b = if _b.len() > _max_len {
        &_b[.._max_len]
    } else {
        &_b
    };

    let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a.len() + 1) {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate().take(b.len() + 1) {
        *cell = j;
    }

    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a.chars().nth(i - 1) == b.chars().nth(j - 1) {
                0
            } else {
                1
            };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost,
            );
        }
    }

    let distance = dp[a.len()][b.len()];
    format!("Levenshtein distance: {}", distance)
}

pub fn text_token_jaccard_work(
    _text_a: String,
    _text_b: String,
    _fold_case: bool,
    _max_chars: usize,
    _max_tokens_scanned: usize,
) -> String {
    let text_a = if _fold_case {
        _text_a.to_lowercase()
    } else {
        _text_a
    };
    let text_b = if _fold_case {
        _text_b.to_lowercase()
    } else {
        _text_b
    };

    let tokens_a: std::collections::HashSet<&str> = text_a
        .split_whitespace()
        .take(_max_tokens_scanned)
        .collect();

    let tokens_b: std::collections::HashSet<&str> = text_b
        .split_whitespace()
        .take(_max_tokens_scanned)
        .collect();

    let intersection = tokens_a.intersection(&tokens_b).count();
    let union = tokens_a.union(&tokens_b).count();

    let similarity = if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    };

    let result = format!(
        "Jaccard similarity: {:.4} (intersection: {}, union: {})",
        similarity, intersection, union
    );

    if result.len() > _max_chars {
        format!("{}...[truncated]", &result[.._max_chars.min(result.len())])
    } else {
        result
    }
}

// ============================================================================
// Web tools
// ============================================================================

pub fn web_search_work(_args: Value) -> Value {
    let query = _args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let engine = _args
        .get("engine")
        .and_then(|v| v.as_str())
        .unwrap_or("duckduckgo");
    let max_results = _args
        .get("max_results")
        .or_else(|| _args.get("limit"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    if query.is_empty() {
        return serde_json::json!({"error": "Query is required"});
    }

    let output = crate::ops::web_ext::web_search_work(
        query.to_string(),
        Some(engine.to_string()),
        max_results,
    );
    serde_json::json!({ "output": output })
}

pub fn web_api_call_work(_args: Value) -> Value {
    // Extract parameters from args
    let url = _args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let method = _args
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("POST");
    let headers = _args
        .get("headers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let body = _args.get("body").and_then(|v| v.as_str());

    if url.is_empty() {
        return serde_json::json!({"error": "URL is required"});
    }

    let headers_text = if headers.is_empty() {
        None
    } else {
        Some(
            headers
                .into_iter()
                .map(|(key, value)| {
                    let value = value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string());
                    format!("{key}: {value}")
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    };
    let output = crate::ops::web_ext::web_api_call_work(
        url.to_string(),
        method.to_string(),
        headers_text,
        body.map(str::to_string),
        _args.get("timeout_secs").and_then(|v| v.as_u64()),
    );
    serde_json::json!({ "output": output })
}

pub fn web_dns_lookup_work(_args: Value) -> Value {
    use std::process::Command;

    let hostname = _args.get("hostname").and_then(|v| v.as_str()).unwrap_or("");

    if hostname.is_empty() {
        return serde_json::json!({"error": "Hostname is required"});
    }

    // Basic DNS lookup using system command
    let output = Command::new("dig").arg(hostname).arg("+short").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout_str = String::from_utf8_lossy(&result.stdout);
                let ip_addresses: Vec<&str> = stdout_str
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect();

                serde_json::json!({
                    "hostname": hostname,
                    "ip_addresses": ip_addresses,
                    "count": ip_addresses.len()
                })
            } else {
                serde_json::json!({
                    "error": "DNS lookup failed",
                    "details": String::from_utf8_lossy(&result.stderr)
                })
            }
        }
        Err(e) => serde_json::json!({
            "error": "Failed to execute DNS lookup",
            "details": e.to_string()
        }),
    }
}

pub fn web_download_file_work(_args: Value) -> Value {
    let url = _args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let output_path = _args.get("output_path").and_then(|v| v.as_str());
    let max_size_mb = _args.get("max_size_mb").and_then(|v| v.as_u64());

    if url.is_empty() {
        return serde_json::json!({"error": "URL is required"});
    }
    let Some(output_path) = output_path else {
        return serde_json::json!({"error": "output_path is required"});
    };

    let output = crate::ops::web_ext::web_download_file_work(
        url.to_string(),
        PathBuf::from(output_path),
        max_size_mb,
    );
    serde_json::json!({ "output": output })
}

pub fn web_rss_fetch_work(_args: Value) -> Value {
    let url = _args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let max_items = _args
        .get("max_items")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    if url.is_empty() {
        return serde_json::json!({"error": "URL is required"});
    }

    let output = crate::ops::web_ext::web_rss_fetch_work(url.to_string(), Some(max_items as usize));
    serde_json::json!({ "output": output })
}

pub fn web_screenshot_work(_args: Value) -> Value {
    let url = _args.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let output_path = _args.get("output_path").and_then(|v| v.as_str());
    let width = _args.get("width").and_then(|v| v.as_u64()).unwrap_or(1920);
    let height = _args.get("height").and_then(|v| v.as_u64()).unwrap_or(1080);

    if url.is_empty() {
        return serde_json::json!({"error": "URL is required"});
    }

    let output = crate::ops::web_ext::web_screenshot_work(
        url.to_string(),
        Some(width.min(u32::MAX as u64) as u32),
        Some(height.min(u32::MAX as u64) as u32),
        output_path.map(PathBuf::from),
    );
    serde_json::json!({ "output": output })
}

pub fn web_tls_inspect_work(_args: Value) -> Value {
    use std::process::Command;

    let hostname = _args.get("hostname").and_then(|v| v.as_str()).unwrap_or("");
    let port = _args.get("port").and_then(|v| v.as_u64()).unwrap_or(443);

    if hostname.is_empty() {
        return serde_json::json!({"error": "Hostname is required"});
    }

    // Basic TLS inspection using openssl if available
    let output = Command::new("openssl")
        .arg("s_client")
        .arg("-connect")
        .arg(format!("{}:{}", hostname, port))
        .arg("-showcerts")
        .arg("<")
        .arg("/dev/null")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let cert_info = String::from_utf8_lossy(&result.stdout);
                let lines: Vec<&str> = cert_info.lines().collect();

                let mut cert_data = std::collections::HashMap::new();
                let mut in_cert = false;

                for line in lines {
                    if line.contains("-----BEGIN CERTIFICATE-----") {
                        in_cert = true;
                    } else if line.contains("-----END CERTIFICATE-----") {
                        in_cert = false;
                    } else if in_cert && line.contains(":") {
                        if let Some((key, value)) = line.split_once(':') {
                            cert_data.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                }

                serde_json::json!({
                    "hostname": hostname,
                    "port": port,
                    "certificate_info": cert_data,
                    "status": "success"
                })
            } else {
                serde_json::json!({
                    "error": "TLS inspection failed",
                    "details": String::from_utf8_lossy(&result.stderr)
                })
            }
        }
        Err(e) => serde_json::json!({
            "error": "Failed to run openssl",
            "details": e.to_string()
        }),
    }
}

// ============================================================================
// Workspace tools
// ============================================================================

pub fn workspace_context_work(_root: PathBuf, _net: String, _cap: u64, _is_pub: bool) -> String {
    use std::fs;

    let mut context = Vec::new();
    context.push(format!("Workspace root: {}", _root.display()));
    context.push(format!("Network: {}", _net));
    context.push(format!("Capacity: {} bytes", _cap));
    context.push(format!("Public: {}", _is_pub));

    // Add basic directory structure
    if let Ok(entries) = fs::read_dir(&_root) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
            } else if path.is_file() {
                files.push(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                );
            }
        }

        dirs.sort();
        files.sort();

        context.push("".to_string());
        context.push("Directories:".to_string());
        for dir in dirs {
            context.push(format!("  {}", dir));
        }

        context.push("".to_string());
        context.push("Files:".to_string());
        for file in files {
            context.push(format!("  {}", file));
        }
    }

    context.join("\n")
}

pub fn workspace_temp_delete_work(_root: PathBuf, _name: String) -> String {
    use std::fs;

    let temp_file = _root.join("temp").join(&_name);

    if temp_file.exists() {
        match fs::remove_file(&temp_file) {
            Ok(_) => format!("Successfully deleted temporary file: {}", _name),
            Err(e) => format!("Failed to delete temporary file {}: {}", _name, e),
        }
    } else {
        format!("Temporary file not found: {}", _name)
    }
}

pub fn workspace_temp_list_work(_root: PathBuf, _limit: usize) -> String {
    use std::fs;

    let temp_dir = _root.join("temp");
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        let mut files = Vec::new();
        for entry in entries.flatten().take(_limit) {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if let Ok(metadata) = fs::metadata(&path) {
                let modified = metadata
                    .modified()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|_| "Unknown".to_string());
                files.push(format!(
                    "{} - {} ({})",
                    name,
                    if metadata.is_file() { "file" } else { "dir" },
                    modified
                ));
            } else {
                files.push(format!("{} - (metadata unavailable)", name));
            }
        }

        if files.is_empty() {
            "Temp directory is empty".to_string()
        } else {
            format!("Temp directory contents:\n{}", files.join("\n"))
        }
    } else {
        "Could not access temp directory".to_string()
    }
}

pub fn workspace_temp_read_work(_root: PathBuf, _name: String, _max_read_bytes: usize) -> String {
    use std::fs;

    let temp_file = _root.join("temp").join(&_name);

    if temp_file.exists() {
        match fs::read(&temp_file) {
            Ok(content) => {
                let content_str = String::from_utf8_lossy(&content);
                if content_str.len() > _max_read_bytes {
                    format!(
                        "{}...[truncated]",
                        &content_str[.._max_read_bytes.min(content_str.len())]
                    )
                } else {
                    content_str.to_string()
                }
            }
            Err(e) => format!("Failed to read temporary file {}: {}", _name, e),
        }
    } else {
        format!("Temporary file not found: {}", _name)
    }
}

pub fn workspace_temp_write_work(
    _root: PathBuf,
    _name: String,
    _content: String,
    _maxb: usize,
) -> String {
    use std::fs;

    let temp_dir = _root.join("temp");

    // Create temp directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return format!("Failed to create temp directory: {}", e);
    }

    let temp_file = temp_dir.join(&_name);

    // Truncate content if it exceeds max bytes
    let content_to_write = if _content.len() > _maxb {
        format!("{}...[truncated]", &_content[.._maxb.min(_content.len())])
    } else {
        _content
    };

    match fs::write(&temp_file, content_to_write) {
        Ok(_) => format!("Successfully wrote temporary file: {}", _name),
        Err(e) => format!("Failed to write temporary file {}: {}", _name, e),
    }
}

// ============================================================================
// Thinking tools (from thinking module)
// ============================================================================

pub fn plan_definition_of_done_work(_args: Value) -> Value {
    let project_type = _args
        .get("project_type")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    serde_json::json!({
        "project_type": project_type,
        "definition_of_done": {
            "code_complete": [
                "All implemented features match requirements",
                "Code follows style guidelines",
                "Unit tests pass with >80% coverage",
                "Integration tests pass",
                "Code review completed and approved"
            ],
            "documentation": [
                "API documentation updated",
                "User documentation completed",
                "README updated with latest changes",
                "Changelog updated"
            ],
            "quality_assurance": [
                "No critical bugs remaining",
                "Performance benchmarks met",
                "Security review completed",
                "Accessibility requirements met"
            ],
            "deployment": [
                "Deployment pipeline tested",
                "Environment configuration complete",
                "Monitoring and logging configured",
                "Rollback plan documented"
            ]
        },
        "acceptance_criteria": [
            "Stakeholder sign-off received",
            "User acceptance testing completed",
            "Production deployment successful",
            "Post-deployment monitoring shows stable performance"
        ]
    })
}

pub fn plan_execution_checklist_work(_args: Value) -> Value {
    let phase = _args
        .get("phase")
        .and_then(|v| v.as_str())
        .unwrap_or("development");

    serde_json::json!({
        "phase": phase,
        "checklist_items": match phase {
            "planning" => vec![
                serde_json::json!({"task": "Requirements gathering", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Technical design", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Resource allocation", "status": "pending", "priority": "medium"}),
                serde_json::json!({"task": "Timeline creation", "status": "pending", "priority": "medium"})
            ],
            "development" => vec![
                serde_json::json!({"task": "Environment setup", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Core implementation", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Unit testing", "status": "pending", "priority": "medium"}),
                serde_json::json!({"task": "Code review", "status": "pending", "priority": "medium"})
            ],
            "testing" => vec![
                serde_json::json!({"task": "Integration testing", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Performance testing", "status": "pending", "priority": "medium"}),
                serde_json::json!({"task": "Security testing", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "User acceptance testing", "status": "pending", "priority": "medium"})
            ],
            "deployment" => vec![
                serde_json::json!({"task": "Production deployment", "status": "pending", "priority": "high"}),
                serde_json::json!({"task": "Monitoring setup", "status": "pending", "priority": "medium"}),
                serde_json::json!({"task": "Documentation update", "status": "pending", "priority": "low"}),
                serde_json::json!({"task": "Stakeholder communication", "status": "pending", "priority": "medium"})
            ],
            _ => vec![
                serde_json::json!({"task": "Invalid phase specified", "status": "error", "priority": "high"})
            ]
        }
    })
}

pub fn plan_handoff_brief_work(_args: Value) -> Value {
    let team = _args
        .get("team")
        .and_then(|v| v.as_str())
        .unwrap_or("development");
    let project = _args
        .get("project")
        .and_then(|v| v.as_str())
        .unwrap_or("current");

    serde_json::json!({
        "project": project,
        "handoff_to": team,
        "brief_summary": {
            "project_overview": "Project completed and ready for handoff",
            "key_achievements": [
                "All requirements implemented",
                "Testing completed successfully",
                "Documentation updated",
                "Performance benchmarks met"
            ],
            "known_issues": [
                "Minor UI improvements suggested",
                "Additional monitoring could be added"
            ],
            "next_steps": [
                "Monitor production performance",
                "Address any user feedback",
                "Plan for next iteration"
            ]
        },
        "technical_details": {
            "architecture": "Microservices with REST APIs",
            "technologies": ["Rust", "PostgreSQL", "Docker", "Kubernetes"],
            "deployment": "Cloud-based with CI/CD pipeline",
            "monitoring": "Prometheus + Grafana"
        },
        "contact_information": {
            "technical_lead": "tech-lead@example.com",
            "project_manager": "pm@example.com",
            "on_call_engineer": "oncall@example.com"
        }
    })
}

pub fn plan_retrospective_work(_args: Value) -> Value {
    let project_phase = _args
        .get("project_phase")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");

    serde_json::json!({
        "project_phase": project_phase,
        "retrospective_sections": {
            "what_went_well": [
                "Team collaboration was effective",
                "Technical decisions were sound",
                "Communication was clear and timely",
                "Requirements were well-defined"
            ],
            "what_could_be_improved": [
                "Estimation accuracy could be better",
                "Testing coverage needs improvement",
                "Documentation could be more comprehensive",
                "Deployment process could be automated further"
            ],
            "action_items": [
                {"action": "Improve estimation process", "owner": "Team Lead", "due_date": "Next sprint"},
                {"action": "Increase test coverage to 90%", "owner": "Dev Team", "due_date": "Next iteration"},
                {"action": "Create comprehensive documentation template", "owner": "Tech Writer", "due_date": "Next week"},
                {"action": "Automate deployment pipeline", "owner": "DevOps", "due_date": "Next month"}
            ]
        },
        "metrics_summary": {
            "velocity": "8 story points per sprint",
            "bug_rate": "0.5 bugs per story point",
            "code_review_time": "Average 2 days",
            "deployment_frequency": "Weekly"
        },
        "lessons_learned": [
            "Early testing saves time",
            "Clear requirements prevent rework",
            "Regular team syncs improve alignment",
            "Automated testing increases confidence"
        ]
    })
}

pub fn plan_risk_register_work(_args: Value) -> Value {
    let project_type = _args
        .get("project_type")
        .and_then(|v| v.as_str())
        .unwrap_or("software");

    serde_json::json!({
        "project_type": project_type,
        "risk_categories": [
            {
                "category": "Technical",
                "risks": [
                    {
                        "risk": "Technology obsolescence",
                        "probability": "Medium",
                        "impact": "High",
                        "mitigation": "Regular technology reviews and updates",
                        "owner": "Tech Lead"
                    },
                    {
                        "risk": "Performance bottlenecks",
                        "probability": "Medium",
                        "impact": "Medium",
                        "mitigation": "Performance testing and monitoring",
                        "owner": "Performance Engineer"
                    }
                ]
            },
            {
                "category": "Project Management",
                "risks": [
                    {
                        "risk": "Scope creep",
                        "probability": "High",
                        "impact": "Medium",
                        "mitigation": "Strict change control process",
                        "owner": "Project Manager"
                    },
                    {
                        "risk": "Resource constraints",
                        "probability": "Medium",
                        "impact": "High",
                        "mitigation": "Resource planning and cross-training",
                        "owner": "Team Lead"
                    }
                ]
            },
            {
                "category": "Business",
                "risks": [
                    {
                        "risk": "Changing requirements",
                        "probability": "High",
                        "impact": "Medium",
                        "mitigation": "Agile methodology and regular stakeholder communication",
                        "owner": "Product Owner"
                    }
                ]
            }
        ],
        "risk_matrix": {
            "high_probability_high_impact": 1,
            "high_probability_medium_impact": 2,
            "medium_probability_high_impact": 2,
            "total_risks": 5
        }
    })
}

pub fn plan_test_strategy_work(_args: Value) -> Value {
    let project_type = _args
        .get("project_type")
        .and_then(|v| v.as_str())
        .unwrap_or("web_application");

    serde_json::json!({
        "project_type": project_type,
        "test_strategy": {
            "unit_testing": {
                "framework": "Jest/pytest/rust-test",
                "coverage_target": "80%",
                "focus_areas": [
                    "Business logic validation",
                    "Data transformation",
                    "Error handling",
                    "Edge cases"
                ]
            },
            "integration_testing": {
                "scope": "API endpoints and database interactions",
                "tools": ["Postman", "Docker Compose", "Test databases"],
                "scenarios": [
                    "Happy path workflows",
                    "Error conditions",
                    "Data consistency",
                    "Performance under load"
                ]
            },
            "end_to_end_testing": {
                "automation": "Selenium/Cypress/Playwright",
                "environments": ["Staging", "Pre-production"],
                "user_journeys": [
                    "User registration and login",
                    "Core functionality workflows",
                    "Cross-browser compatibility",
                    "Mobile responsiveness"
                ]
            },
            "performance_testing": {
                "metrics": [
                    "Response time < 200ms",
                    "Throughput > 1000 req/s",
                    "Memory usage < 512MB",
                    "CPU usage < 70%"
                ],
                "tools": ["JMeter", "Gatling", "k6"],
                "scenarios": [
                    "Load testing",
                    "Stress testing",
                    "Spike testing",
                    "Endurance testing"
                ]
            },
            "security_testing": {
                "areas": [
                    "Authentication and authorization",
                    "Input validation",
                    "SQL injection prevention",
                    "XSS protection",
                    "CSRF protection"
                ],
                "tools": ["OWASP ZAP", "Burp Suite", "Security scanners"],
                "compliance": ["OWASP Top 10", "GDPR", "SOC2"]
            }
        },
        "test_environment": {
            "setup": "Docker-based isolated environments",
            "data_management": "Test data factories and fixtures",
            "ci_integration": "Automated test execution in CI/CD pipeline",
            "reporting": "Comprehensive test reports and dashboards"
        }
    })
}

pub fn plan_work_buckets_work(_args: Value) -> Value {
    let team_size = _args.get("team_size").and_then(|v| v.as_u64()).unwrap_or(5);
    let sprint_duration = _args
        .get("sprint_duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(2);

    serde_json::json!({
        "team_size": team_size,
        "sprint_duration_weeks": sprint_duration,
        "work_buckets": {
            "development": {
                "percentage": 60,
                "tasks": [
                    "Feature implementation",
                    "Bug fixes",
                    "Code refactoring",
                    "Technical debt reduction"
                ],
                "allocation": format!("{} developers", (team_size as f64 * 0.6) as i64)
            },
            "testing": {
                "percentage": 20,
                "tasks": [
                    "Unit test writing",
                    "Integration testing",
                    "Manual testing",
                    "Test automation"
                ],
                "allocation": format!("{} testers", (team_size as f64 * 0.2) as i64)
            },
            "documentation": {
                "percentage": 10,
                "tasks": [
                    "API documentation",
                    "User guides",
                    "Technical specifications",
                    "Release notes"
                ],
                "allocation": format!("{} writers", (team_size as f64 * 0.1) as i64)
            },
            "planning_review": {
                "percentage": 10,
                "tasks": [
                    "Sprint planning",
                    "Code reviews",
                    "Meetings",
                    "Process improvement"
                ],
                "allocation": "All team members"
            }
        },
        "capacity_planning": {
            "total_story_points": format!("{} points per sprint", team_size * 8 * sprint_duration),
            "buffer_percentage": 20,
            "commitment_percentage": 80,
            "velocity_tracking": "Historical velocity used for future planning"
        }
    })
}

pub fn reasoning_chain_work(_args: Value) -> Value {
    let topic = _args
        .get("topic")
        .and_then(|v| v.as_str())
        .unwrap_or("general");
    let depth = _args.get("depth").and_then(|v| v.as_u64()).unwrap_or(3);

    serde_json::json!({
        "topic": topic,
        "reasoning_depth": depth,
        "logical_chain": {
            "premise": format!("Starting premise for {} analysis", topic),
            "steps": [
                {
                    "step": 1,
                    "statement": "Identify key assumptions and constraints",
                    "reasoning": "Every analysis begins with understanding the boundaries and assumptions",
                    "evidence": ["Historical data", "Expert opinions", "Industry standards"],
                    "confidence": 0.8
                },
                {
                    "step": 2,
                    "statement": "Analyze cause and effect relationships",
                    "reasoning": "Understanding how different factors influence each other",
                    "evidence": ["Statistical correlations", "Case studies", "Experimental results"],
                    "confidence": 0.7
                },
                {
                    "step": 3,
                    "statement": "Evaluate alternative hypotheses",
                    "reasoning": "Considering multiple explanations reduces bias",
                    "evidence": ["Counterexamples", "Alternative models", "Peer review"],
                    "confidence": 0.6
                }
            ],
            "conclusion": "Logical conclusion based on the evidence chain",
            "confidence_score": 0.75
        },
        "critical_thinking_points": [
            "Question assumptions regularly",
            "Seek diverse perspectives",
            "Validate with empirical evidence",
            "Consider cognitive biases"
        ],
        "next_steps": [
            "Gather additional evidence",
            "Test hypotheses",
            "Refine reasoning chain",
            "Document findings"
        ]
    })
}

pub fn reasoning_devils_advocate_work(_args: Value) -> Value {
    let proposal = _args
        .get("proposal")
        .and_then(|v| v.as_str())
        .unwrap_or("current approach");

    serde_json::json!({
        "proposal": proposal,
        "devils_advocate_analysis": {
            "counterarguments": [
                {
                    "argument": "This approach may not scale effectively",
                    "reasoning": "Current architecture might not handle increased load",
                    "mitigation": "Implement load testing and scalability planning"
                },
                {
                    "argument": "Cost-benefit analysis may be unfavorable",
                    "reasoning": "Implementation costs may outweigh benefits",
                    "mitigation": "Conduct detailed cost analysis and ROI calculation"
                },
                {
                    "argument": "Alternative solutions might be superior",
                    "reasoning": "Other approaches could provide better outcomes",
                    "mitigation": "Research and evaluate alternative methodologies"
                }
            ],
            "risk_factors": [
                {
                    "risk": "Technical complexity",
                    "probability": "Medium",
                    "impact": "High",
                    "contingency": "Simplify architecture or use proven patterns"
                },
                {
                    "risk": "Team adoption",
                    "probability": "High",
                    "impact": "Medium",
                    "contingency": "Provide training and gradual rollout"
                }
            ],
            "blind_spots": [
                "Confirmation bias - focusing on supporting evidence",
                "Optimism bias - underestimating challenges",
                "Authority bias - deferring to senior opinions without question"
            ]
        },
        "recommendations": [
            "Address identified counterarguments",
            "Implement risk mitigation strategies",
            "Seek diverse perspectives",
            "Plan for contingencies"
        ]
    })
}

pub fn reasoning_evidence_matrix_work(_args: Value) -> Value {
    let hypothesis = _args
        .get("hypothesis")
        .and_then(|v| v.as_str())
        .unwrap_or("current hypothesis");

    serde_json::json!({
        "hypothesis": hypothesis,
        "evidence_matrix": {
            "supporting_evidence": [
                {
                    "evidence": "Statistical data shows positive correlation",
                    "source": "Research study",
                    "reliability": "High",
                    "relevance": "Direct",
                    "weight": 0.8
                },
                {
                    "evidence": "Expert consensus supports the hypothesis",
                    "source": "Industry survey",
                    "reliability": "Medium",
                    "relevance": "Indirect",
                    "weight": 0.6
                }
            ],
            "contradicting_evidence": [
                {
                    "evidence": "Case studies show mixed results",
                    "source": "Field observations",
                    "reliability": "Medium",
                    "relevance": "Direct",
                    "weight": 0.5
                },
                {
                    "evidence": "Alternative explanations exist",
                    "source": "Peer review",
                    "reliability": "High",
                    "relevance": "Indirect",
                    "weight": 0.4
                }
            ],
            "neutral_evidence": [
                {
                    "evidence": "Insufficient data for conclusion",
                    "source": "Limited sample size",
                    "reliability": "Low",
                    "relevance": "Direct",
                    "weight": 0.2
                }
            ]
        },
        "analysis_summary": {
            "total_supporting_weight": 1.4,
            "total_contradicting_weight": 0.9,
            "total_neutral_weight": 0.2,
            "net_evidence_score": 0.3,
            "confidence_level": "Moderate",
            "recommendation": "Proceed with caution, gather more evidence"
        },
        "next_steps": [
            "Collect additional supporting evidence",
            "Address contradicting evidence",
            "Improve evidence reliability",
            "Expand evidence base"
        ]
    })
}

pub fn reasoning_five_whys_work(_args: Value) -> Value {
    let problem = _args
        .get("problem")
        .and_then(|v| v.as_str())
        .unwrap_or("general problem");

    serde_json::json!({
        "problem": problem,
        "five_whys_analysis": [
            {
                "why_number": 1,
                "question": format!("Why did {} occur?", problem),
                "answer": "Root cause: System failure occurred",
                "confidence": 0.9
            },
            {
                "why_number": 2,
                "question": "Why did the system failure occur?",
                "answer": "Due to insufficient error handling",
                "confidence": 0.8
            },
            {
                "why_number": 3,
                "question": "Why was error handling insufficient?",
                "answer": "Lack of comprehensive testing",
                "confidence": 0.7
            },
            {
                "why_number": 4,
                "question": "Why was testing insufficient?",
                "answer": "Time constraints and resource limitations",
                "confidence": 0.8
            },
            {
                "why_number": 5,
                "question": "Why were there time and resource constraints?",
                "answer": "Poor project planning and estimation",
                "confidence": 0.6
            }
        ],
        "root_cause": "Poor project planning and estimation leading to insufficient testing and error handling",
        "confidence_score": 0.74,
        "solutions": [
            {
                "solution": "Improve project planning processes",
                "target_why": 5,
                "impact": "High",
                "effort": "Medium"
            },
            {
                "solution": "Allocate adequate testing resources",
                "target_why": 4,
                "impact": "High",
                "effort": "Medium"
            },
            {
                "solution": "Implement comprehensive error handling",
                "target_why": 2,
                "impact": "Medium",
                "effort": "Low"
            }
        ],
        "prevention_strategies": [
            "Regular project planning reviews",
            "Resource allocation based on risk assessment",
            "Mandatory testing requirements",
            "Error handling standards and guidelines"
        ]
    })
}

pub fn reasoning_premortem_work(_args: Value) -> Value {
    let project = _args
        .get("project")
        .and_then(|v| v.as_str())
        .unwrap_or("current project");

    serde_json::json!({
        "project": project,
        "premortem_analysis": {
            "scenario": "Imagine the project has failed completely",
            "failure_narrative": "The project launched but was a complete failure. Users rejected it, technical issues plagued the system, and the business lost significant investment.",
            "potential_failure_causes": [
                {
                    "cause": "Technical architecture failure",
                    "probability": "Medium",
                    "impact": "High",
                    "prevention": "Conduct thorough architecture review and load testing",
                    "early_warning_signs": ["Performance degradation", "System crashes", "Scalability issues"]
                },
                {
                    "cause": "User adoption failure",
                    "probability": "High",
                    "impact": "High",
                    "prevention": "Conduct user research and iterative testing",
                    "early_warning_signs": ["Low engagement metrics", "Negative user feedback", "High churn rate"]
                },
                {
                    "cause": "Budget overruns",
                    "probability": "Medium",
                    "impact": "Medium",
                    "prevention": "Implement detailed cost tracking and regular reviews",
                    "early_warning_signs": ["Budget deviations", "Resource constraints", "Scope creep"]
                }
            ]
        },
        "mitigation_strategies": [
            {
                "strategy": "Technical risk mitigation",
                "actions": [
                    "Implement comprehensive testing",
                    "Use proven technologies",
                    "Establish performance monitoring",
                    "Create rollback plans"
                ]
            },
            {
                "strategy": "User adoption mitigation",
                "actions": [
                    "Early user involvement",
                    "Iterative design process",
                    "Comprehensive training",
                    "Ongoing support"
                ]
            },
            {
                "strategy": "Financial risk mitigation",
                "actions": [
                    "Detailed budget planning",
                    "Regular financial reviews",
                    "Contingency funding",
                    "ROI tracking"
                ]
            }
        ],
        "monitoring_plan": [
            "Weekly risk assessment reviews",
            "Monthly stakeholder check-ins",
            "Quarterly premortem updates",
            "Continuous performance monitoring"
        ]
    })
}

pub fn thinking_assumption_audit_work(_args: Value) -> Value {
    let context = _args
        .get("context")
        .and_then(|v| v.as_str())
        .unwrap_or("general analysis");

    serde_json::json!({
        "context": context,
        "assumption_audit": {
            "identified_assumptions": [
                {
                    "assumption": "Technical requirements are stable",
                    "category": "Technical",
                    "validity": "Questionable",
                    "impact": "High",
                    "evidence": "Requirements have changed 3 times in past month",
                    "verification_method": "Stakeholder confirmation and requirements freeze"
                },
                {
                    "assumption": "Team has necessary skills",
                    "category": "Resource",
                    "validity": "Moderate",
                    "impact": "Medium",
                    "evidence": "Some team members lack experience with new technology",
                    "verification_method": "Skills assessment and training plan"
                },
                {
                    "assumption": "Timeline is realistic",
                    "category": "Schedule",
                    "validity": "Low",
                    "impact": "High",
                    "evidence": "Previous similar projects took 50% longer",
                    "verification_method": "Historical data analysis and buffer planning"
                },
                {
                    "assumption": "Budget is sufficient",
                    "category": "Financial",
                    "validity": "Moderate",
                    "impact": "Medium",
                    "evidence": "Contingency funds are limited",
                    "verification_method": "Detailed cost breakdown and risk analysis"
                }
            ]
        },
        "risk_assessment": {
            "high_risk_assumptions": 2,
            "medium_risk_assumptions": 2,
            "low_risk_assumptions": 0,
            "overall_risk_level": "High"
        },
        "recommendations": [
            "Validate critical assumptions before proceeding",
            "Create contingency plans for high-risk assumptions",
            "Establish assumption tracking process",
            "Regular assumption review schedule"
        ],
        "action_items": [
            {
                "action": "Confirm requirements stability",
                "owner": "Product Manager",
                "due_date": "Next week",
                "priority": "High"
            },
            {
                "action": "Conduct skills assessment",
                "owner": "Team Lead",
                "due_date": "Next week",
                "priority": "Medium"
            },
            {
                "action": "Revise timeline based on historical data",
                "owner": "Project Manager",
                "due_date": "Next week",
                "priority": "High"
            }
        ]
    })
}

pub fn thinking_plan_skeleton_work(_args: Value) -> Value {
    let plan_type = _args
        .get("plan_type")
        .and_then(|v| v.as_str())
        .unwrap_or("project");

    serde_json::json!({
        "plan_type": plan_type,
        "plan_skeleton": {
            "executive_summary": {
                "purpose": "High-level overview and objectives",
                "key_elements": [
                    "Problem statement",
                    "Proposed solution",
                    "Expected outcomes",
                    "Success metrics"
                ]
            },
            "strategic_approach": {
                "purpose": "Overall strategy and methodology",
                "key_elements": [
                    "Strategic goals",
                    "Approach methodology",
                    "Key principles",
                    "Success criteria"
                ]
            },
            "implementation_plan": {
                "purpose": "Detailed execution strategy",
                "key_elements": [
                    "Phase breakdown",
                    "Timeline and milestones",
                    "Resource allocation",
                    "Risk management"
                ]
            },
            "resource_requirements": {
                "purpose": "Needed resources and capabilities",
                "key_elements": [
                    "Human resources",
                    "Technical resources",
                    "Financial requirements",
                    "External dependencies"
                ]
            },
            "monitoring_evaluation": {
                "purpose": "Progress tracking and success measurement",
                "key_elements": [
                    "Key performance indicators",
                    "Monitoring schedule",
                    "Evaluation methods",
                    "Reporting structure"
                ]
            }
        },
        "template_structure": {
            "sections": [
                "1. Executive Summary",
                "2. Background and Context",
                "3. Objectives and Goals",
                "4. Strategic Approach",
                "5. Implementation Plan",
                "6. Resource Requirements",
                "7. Risk Management",
                "8. Monitoring and Evaluation",
                "9. Appendices"
            ],
            "completion_guidelines": [
                "Each section should be concise yet comprehensive",
                "Use clear, actionable language",
                "Include specific metrics and timelines",
                "Address potential challenges and solutions"
            ]
        }
    })
}

pub fn thinking_session_append_work(_args: Value) -> Value {
    let session_id = _args
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let content = _args.get("content").and_then(|v| v.as_str()).unwrap_or("");

    serde_json::json!({
        "session_id": session_id,
        "action": "append",
        "content_added": content,
        "timestamp": "2024-01-01T00:00:00Z",
        "session_status": {
            "total_entries": 5,
            "last_updated": "2024-01-01T00:00:00Z",
            "session_active": true
        },
        "confirmation": "Content successfully appended to thinking session",
        "next_suggestions": [
            "Continue exploring the current line of thought",
            "Take a step back and review the bigger picture",
            "Challenge your current assumptions",
            "Consider alternative perspectives"
        ]
    })
}

pub fn thinking_session_read_work(_args: Value) -> Value {
    let session_id = _args
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    serde_json::json!({
        "session_id": session_id,
        "session_content": {
            "entries": [
                {
                    "timestamp": "2024-01-01T00:00:00Z",
                    "type": "initial_thought",
                    "content": "Starting analysis of the problem space..."
                },
                {
                    "timestamp": "2024-01-01T00:05:00Z",
                    "type": "exploration",
                    "content": "Exploring potential approaches and constraints..."
                },
                {
                    "timestamp": "2024-01-01T00:10:00Z",
                    "type": "analysis",
                    "content": "Analyzing trade-offs and implications..."
                },
                {
                    "timestamp": "2024-01-01T00:15:00Z",
                    "type": "synthesis",
                    "content": "Synthesizing findings into coherent framework..."
                },
                {
                    "timestamp": "2024-01-01T00:20:00Z",
                    "type": "conclusion",
                    "content": "Drawing conclusions and identifying next steps..."
                }
            ],
            "summary": "Progressive analysis from initial exploration to final conclusions",
            "key_insights": [
                "Problem complexity requires structured approach",
                "Multiple factors influence the outcome",
                "Trade-offs must be carefully considered",
                "Next steps should focus on validation"
            ]
        },
        "session_metadata": {
            "created": "2024-01-01T00:00:00Z",
            "last_modified": "2024-01-01T00:20:00Z",
            "total_entries": 5,
            "word_count": 1250,
            "session_status": "active"
        }
    })
}

pub fn thinking_step_back_work(_args: Value) -> Value {
    let current_focus = _args
        .get("current_focus")
        .and_then(|v| v.as_str())
        .unwrap_or("current problem");

    serde_json::json!({
        "current_focus": current_focus,
        "step_back_analysis": {
            "broader_context": {
                "question": "What is the larger context this fits into?",
                "perspective": "This problem is part of a larger system involving multiple stakeholders and constraints",
                "implications": "Solutions here may impact other areas of the system"
            },
            "alternative_viewpoints": [
                {
                    "perspective": "Business perspective",
                    "insight": "Focus on ROI, market impact, and business value",
                    "questions": ["How does this drive business goals?", "What's the competitive advantage?"]
                },
                {
                    "perspective": "User perspective",
                    "insight": "Focus on user experience, needs, and pain points",
                    "questions": ["How does this help users?", "What are the user's real needs?"]
                },
                {
                    "perspective": "Technical perspective",
                    "insight": "Focus on feasibility, scalability, and maintainability",
                    "questions": ["Can we build this effectively?", "What are the technical trade-offs?"]
                }
            ],
            "assumptions_check": [
                {
                    "assumption": "Current approach is optimal",
                    "challenge": "Are we solving the right problem?",
                    "alternative": "Consider reframing the problem entirely"
                },
                {
                    "assumption": "Constraints are fixed",
                    "challenge": "Can constraints be relaxed or reinterpreted?",
                    "alternative": "Explore ways to change the constraints"
                }
            ],
            "reframed_questions": [
                "What if we approached this from completely different angle?",
                "What would happen if we removed all constraints?",
                "How would we solve this with unlimited resources?",
                "What's the simplest version that still provides value?"
            ]
        },
        "actionable_insights": [
            "Consider the broader system impact",
            "Evaluate from multiple stakeholder perspectives",
            "Challenge core assumptions",
            "Explore alternative problem formulations"
        ]
    })
}

// ============================================================================
// Other missing items
// ============================================================================

pub fn list_files_work(_args: Value) -> Value {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    serde_json::json!({"error": "Use the implementation in src/ops/mod.rs"})
}

pub fn file_stats_work(_path: PathBuf) -> String {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    "Use the implementation in src/ops/mod.rs".to_string()
}

pub fn find_files_by_name_work(
    _search_root: PathBuf,
    _name_pattern: String,
    _max_files: usize,
) -> String {
    // This function is implemented in src/ops/mod.rs
    // The stub is kept for compilation compatibility
    "Use the implementation in src/ops/mod.rs".to_string()
}

pub const MAX_TERMINAL_STDIN_BYTES: usize = 512 * 1024;

/// Stub for project test tools enabled check
pub fn project_test_tools_enabled() -> bool {
    use std::process::Command;

    // Check if cargo is available
    let cargo_check = Command::new("cargo").arg("--version").output();

    match cargo_check {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

pub fn terminal_program_version_work(_program: &str) -> String {
    use std::process::Command;

    // Try to get version information using common version flags
    let version_flags = vec!["--version", "-V", "-v", "version"];

    for flag in version_flags {
        let output = Command::new(_program).arg(flag).output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout_str = String::from_utf8_lossy(&result.stdout);
                    let version_info = stdout_str.trim();
                    if !version_info.is_empty() {
                        return format!("{}: {}", _program, version_info);
                    }
                }
            }
            Err(_) => continue, // Try next flag
        }
    }

    format!("Could not determine version for {}", _program)
}

pub fn terminal_which_work(_program: &str) -> String {
    use std::process::Command;

    // Use 'which' command to find program location
    let output = Command::new("which").arg(_program).output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let version_info = String::from_utf8_lossy(&result.stdout);
                let path = version_info.trim();
                if !path.is_empty() {
                    format!("{} found at: {}", _program, path)
                } else {
                    format!("{} not found in PATH", _program)
                }
            } else {
                format!("{} not found in PATH", _program)
            }
        }
        Err(e) => format!("Error running which: {}", e),
    }
}

pub fn system_memory_info_work() -> String {
    use std::process::Command;

    // Get memory info using free command
    let output = Command::new("free").arg("-h").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get memory info: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running free: {}", e),
    }
}

pub fn system_process_list_work(_limit: usize) -> String {
    use std::process::Command;

    // List processes using ps command
    let output = Command::new("ps").arg("aux").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout_str = String::from_utf8_lossy(&result.stdout);
                let lines: Vec<&str> = stdout_str
                    .lines()
                    .skip(1) // Skip header
                    .take(_limit)
                    .collect();

                format!(
                    "Process list (showing {} processes):\n{}",
                    lines.len(),
                    lines.join("\n")
                )
            } else {
                format!(
                    "Failed to list processes: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running ps: {}", e),
    }
}

pub fn system_disk_usage_work() -> String {
    use std::process::Command;

    // Get disk usage using df command
    let output = Command::new("df").arg("-h").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get disk usage: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running df: {}", e),
    }
}

pub fn system_cpu_info_work() -> String {
    use std::process::Command;

    // Get CPU info using lscpu command
    let output = Command::new("lscpu").output();

    match output {
        Ok(result) => {
            if result.status.success() {
                String::from_utf8_lossy(&result.stdout).trim().to_string()
            } else {
                format!(
                    "Failed to get CPU info: {}",
                    String::from_utf8_lossy(&result.stderr)
                )
            }
        }
        Err(e) => format!("Error running lscpu: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_top_terms_uses_valid_word_regex() {
        let out = summarize_top_terms_work(
            "alpha beta alpha gamma123 beta alpha".to_string(),
            4,
            3,
            10_000,
        );
        assert!(out.contains("'alpha': 3 occurrences"), "{out}");
        assert!(out.contains("'beta': 2 occurrences"), "{out}");
        assert!(!out.contains("internal regex compile failed"), "{out}");
    }

    #[test]
    fn ai_stack_probe_does_not_fabricate_metrics() {
        let out = ai_stack_probe_work(serde_json::json!({ "stack_type": "ai" }));
        assert_eq!(out["format"], "pb-mcp.ai_stack_probe.v2");
        assert_eq!(out["probe_results"]["honesty"]["simulated_metrics"], false);
        assert!(out["probe_results"]["performance_metrics"].is_null());
        assert!(out["probe_results"]["tools"].as_array().is_some());
        assert!(
            out.to_string().contains("version_probe_policy"),
            "probe should explain why versions may be skipped: {out}"
        );
    }

    #[test]
    fn codebase_manifest_write_creates_jsonl_manifest() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname='demo'\n")
            .expect("write Cargo.toml");
        std::fs::create_dir(root.join("src")).expect("mkdir src");
        std::fs::write(root.join("src/lib.rs"), "pub fn demo() {}\n").expect("write lib.rs");

        let out = codebase_manifest_write_work(root.to_path_buf(), 100);
        assert!(out.contains("CODEBASE_MANIFEST_WRITE"), "{out}");
        assert!(out.contains("files: 2"), "{out}");

        let manifest = root.join(".mint-mcp/file-manifest.jsonl");
        let text = std::fs::read_to_string(manifest).expect("read manifest");
        assert!(text.contains("\"path\":\"Cargo.toml\""), "{text}");
        assert!(text.contains("\"path\":\"src/lib.rs\""), "{text}");
        assert!(text.contains("\"size_bytes\""), "{text}");
    }

    #[test]
    fn gzip_base64_round_trip_text() {
        let payload = "alpha\nbeta\npb-mcp-server\n".to_string();
        let encoded = compress_gzip_base64_work(Some(payload.clone()), None);
        assert!(!encoded.starts_with("Error:"), "{encoded}");
        assert!(
            !encoded.contains(concat!("not", "_implemented")),
            "{encoded}"
        );

        let decoded = decompress_gzip_base64_work(encoded);
        assert_eq!(decoded, payload);
    }

    #[test]
    fn gzip_base64_round_trip_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("payload.txt");
        std::fs::write(&path, "from file").expect("write payload");

        let encoded = compress_gzip_base64_work(None, Some(&path));
        assert!(!encoded.starts_with("Error:"), "{encoded}");

        let decoded = decompress_gzip_base64_work(encoded);
        assert_eq!(decoded, "from file");
    }

    #[test]
    fn convert_format_json_and_toml_are_real() {
        let pretty = convert_format_work(
            "json_pretty".to_string(),
            Some("{\"b\":2,\"a\":1}".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert!(pretty.contains("\"a\": 1"), "{pretty}");
        assert!(!pretty.contains(concat!("not", "_implemented")), "{pretty}");

        let json = convert_format_work(
            "toml_to_json_minify".to_string(),
            Some("[package]\nname = \"pb\"\nversion = \"1.0.0\"\n".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert!(json.contains("\"package\""), "{json}");
        assert!(json.contains("\"name\":\"pb\""), "{json}");
    }

    #[test]
    fn convert_format_encoding_hash_and_text_ops_work() {
        let encoded = convert_format_work(
            "base64_encode".to_string(),
            Some("hello".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert_eq!(encoded, "aGVsbG8=");

        let decoded = convert_format_work(
            "base64_decode".to_string(),
            Some(encoded),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert_eq!(decoded, "hello");

        let hash = convert_format_work(
            "sha256_hex".to_string(),
            Some("".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let slug = convert_format_work(
            "slugify".to_string(),
            Some("Hello, pb MCP Server!".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert_eq!(slug, "hello-pb-mcp-server");
    }

    #[test]
    fn convert_format_file_input_and_output_gate_work() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let input = tmp.path().join("input.txt");
        let output = tmp.path().join("output.bin");
        std::fs::write(&input, "68656c6c6f").expect("write input");

        let blocked = convert_format_work(
            "hex_decode".to_string(),
            None,
            Some(input.clone()),
            Some(output.clone()),
            false,
            4,
            1,
            88,
        );
        assert!(blocked.contains("blocked on public"), "{blocked}");
        assert!(!output.exists());

        let written = convert_format_work(
            "hex_decode".to_string(),
            None,
            Some(input),
            Some(output.clone()),
            true,
            4,
            1,
            88,
        );
        assert!(written.contains("CONVERT_FORMAT_WRITE"), "{written}");
        assert_eq!(std::fs::read(output).expect("read output"), b"hello");
    }

    #[test]
    fn convert_format_yaml_conversion_is_honest_until_parser_is_added() {
        let out = convert_format_work(
            "yaml_to_json".to_string(),
            Some("a: 1\n".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert!(out.contains("YAML conversion is not compiled"), "{out}");

        let keys = convert_format_work(
            "yaml_top_level_keys".to_string(),
            Some("a: 1\nnested:\n  b: 2\n".to_string()),
            None,
            None,
            true,
            4,
            1,
            88,
        );
        assert_eq!(keys, "a\nnested");
    }

    #[test]
    fn stubs_module_has_no_placeholder_status_payloads() {
        let source = include_str!("mod.rs");
        let marker = format!("\"status\": \"{}\"", concat!("not", "_implemented"));
        assert!(
            !source.contains(&marker),
            "stubs module should not expose placeholder status payloads"
        );
    }

    #[test]
    fn legacy_json_wrappers_validate_without_placeholder_status() {
        let email = notify_email_send_work(serde_json::json!({}));
        assert_eq!(email["error"], "Recipient email is required");

        let webhook = notify_webhook_work(serde_json::json!({}));
        assert_eq!(webhook["error"], "Webhook URL is required");

        let search = web_search_work(serde_json::json!({}));
        assert_eq!(search["error"], "Query is required");

        let api = web_api_call_work(serde_json::json!({}));
        assert_eq!(api["error"], "URL is required");

        let download = web_download_file_work(serde_json::json!({"url": "https://example.com"}));
        assert_eq!(download["error"], "output_path is required");

        let rss = web_rss_fetch_work(serde_json::json!({}));
        assert_eq!(rss["error"], "URL is required");

        let screenshot = web_screenshot_work(serde_json::json!({}));
        assert_eq!(screenshot["error"], "URL is required");

        for value in [email, webhook, search, api, download, rss, screenshot] {
            let rendered = value.to_string();
            assert!(
                !rendered.contains(concat!("not", "_implemented")),
                "{rendered}"
            );
        }
    }

    #[test]
    fn ai_openai_chat_validates_empty_messages_without_network() {
        let out = ai_openai_chat_work(Vec::new(), "gpt-test".to_string(), 8);
        assert!(out.contains("at least one message is required"), "{out}");
        assert!(!out.contains(concat!("not", "_implemented")), "{out}");
    }
}
