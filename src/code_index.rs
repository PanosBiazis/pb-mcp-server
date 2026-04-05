//! Lightweight symbol index under `.mint-mcp/code-index.json` (no external DB).
//! Heuristic extraction for fast lookup; not a full language server.

use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// Relative path for the code index file under the project root
pub const INDEX_REL_PATH: &str = ".mint-mcp/code-index.json";
const MAX_FILES: usize = 2000;
const MAX_READ_BYTES: u64 = 96 * 1024;
const MAX_SYMBOLS_PER_FILE: usize = 120;
const MAX_INDEX_JSON_BYTES: usize = 12 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Code index file structure containing metadata and symbol entries
pub struct CodeIndexFile {
    /// Index format version number
    pub version: u32,
    /// Timestamp when the index was generated (Unix milliseconds)
    pub generated_ts_ms: u64,
    /// Project root path that was indexed
    pub project_root: String,
    /// Total number of files that were indexed
    pub files_indexed: usize,
    /// List of file entries with their symbols
    pub entries: Vec<CodeIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Single file entry in the code index
pub struct CodeIndexEntry {
    /// Relative file path from project root
    pub path: String,
    /// List of symbols found in this file
    pub symbols: Vec<String>,
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn index_path(root: &Path) -> PathBuf {
    root.join(INDEX_REL_PATH)
}

fn allowed_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_ascii_lowercase().as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cpp"
                    | "hpp"
                    | "cc"
                    | "hh"
                    | "rb"
                    | "cs"
                    | "sapp"
                    | "md"
                    | "toml"
                    | "json"
                    | "yaml"
                    | "yml"
            )
        })
        .unwrap_or(false)
}

fn compile_symbol_patterns() -> Vec<Regex> {
    let raw = [
        r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?struct\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?enum\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?trait\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?type\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?mod\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*def\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*class\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)\bfunction\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)\binterface\s+([a-zA-Z_][a-zA-Z0-9_]*)",
        r"(?m)^\s*func\s+([a-zA-Z_][a-zA-Z0-9_]*)",
    ];
    raw.iter().filter_map(|p| Regex::new(p).ok()).collect()
}

fn extract_symbols(text: &str, patterns: &[Regex]) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for re in patterns {
        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                let s = m.as_str().to_string();
                if s.len() <= 96 {
                    set.insert(s);
                }
                if set.len() >= MAX_SYMBOLS_PER_FILE {
                    break;
                }
            }
        }
        if set.len() >= MAX_SYMBOLS_PER_FILE {
            break;
        }
    }
    set.into_iter().collect()
}

fn normalize_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"))
}

/// Build or rebuild the code index under `project_root/.mint-mcp/code-index.json`.
pub fn code_index_build_work(
    project_root: PathBuf,
    subtree: PathBuf,
    respect_gitignore: bool,
    max_files: usize,
) -> String {
    let max_files = max_files.clamp(1, MAX_FILES);
    let patterns = compile_symbol_patterns();

    let mut walk = WalkBuilder::new(&subtree);
    walk.git_ignore(respect_gitignore);
    walk.ignore(respect_gitignore);
    walk.hidden(false);

    let mut entries: Vec<CodeIndexEntry> = Vec::new();
    let mut scanned = 0usize;

    for entry in walk.build().flatten() {
        if scanned >= max_files {
            break;
        }
        let path = entry.path();
        if !path.is_file() || !allowed_ext(path) {
            continue;
        }
        let rel = normalize_rel(&project_root, path);
        if rel.contains("..") {
            continue;
        }

        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() > MAX_READ_BYTES * 4 {
            continue;
        }

        let f = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        use std::io::Read;
        let mut buf = Vec::new();
        if f.take(MAX_READ_BYTES).read_to_end(&mut buf).is_err() {
            continue;
        }
        if buf.iter().any(|&b| b == 0) {
            continue;
        }
        let text = String::from_utf8_lossy(&buf);
        let symbols = extract_symbols(&text, &patterns);
        if symbols.is_empty() {
            scanned += 1;
            continue;
        }

        entries.push(CodeIndexEntry { path: rel, symbols });
        scanned += 1;
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let idx = CodeIndexFile {
        version: 1,
        generated_ts_ms: now_ms(),
        project_root: project_root.display().to_string(),
        files_indexed: entries.len(),
        entries,
    };

    let json = match serde_json::to_string_pretty(&idx) {
        Ok(j) => j,
        Err(e) => return format!("Error serializing index: {e}"),
    };
    if json.len() > MAX_INDEX_JSON_BYTES {
        return format!(
            "Error: index too large ({} bytes); narrow path or lower max_files",
            json.len()
        );
    }

    let out_path = index_path(&project_root);
    if let Some(parent) = out_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return format!("Error creating .mint-mcp: {e}");
        }
    }
    let tmp = out_path.with_extension("json.tmp");
    if let Err(e) = fs::write(&tmp, &json) {
        return format!("Error writing temp index: {e}");
    }
    if let Err(e) = fs::rename(&tmp, &out_path) {
        let _ = fs::remove_file(&tmp);
        return format!("Error finalizing index: {e}");
    }

    format!(
        "CODE_INDEX_BUILD\npath: {}\nfiles_with_symbols: {}\nbytes: {}\nrespect_gitignore: {}\n",
        out_path.display(),
        idx.files_indexed,
        json.len(),
        respect_gitignore
    )
}

/// Search the persisted index (substring or regex on path or symbol).
pub fn code_index_search_work(
    project_root: PathBuf,
    needle: String,
    case_sensitive: bool,
    limit: usize,
    path_prefix: Option<String>,
    use_regex: bool,
) -> String {
    let limit = limit.clamp(1, 200);
    let needle = needle.trim();
    if needle.is_empty() {
        return "Error: needle must not be empty".to_string();
    }
    if use_regex && needle.len() > 512 {
        return "Error: regex pattern too long (max 512)".to_string();
    }

    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_SEARCH\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }

    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };

    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let regex_opt = if use_regex {
        match RegexBuilder::new(needle)
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => Some(r),
            Err(e) => return format!("Error: invalid regex: {e}"),
        }
    } else {
        None
    };

    let needle_cmp = if case_sensitive {
        needle.to_string()
    } else {
        needle.to_lowercase()
    };

    let prefix = path_prefix
        .as_ref()
        .map(|p| p.trim().trim_matches('/').to_string())
        .filter(|p| !p.is_empty());

    let mut out = String::new();
    out.push_str("CODE_INDEX_SEARCH\n");
    out.push_str(if use_regex {
        "mode: regex\n"
    } else {
        "mode: substring\n"
    });
    out.push_str(&format!("matches (max {}):\n\n", limit));

    let mut n = 0usize;
    for e in &idx.entries {
        if n >= limit {
            break;
        }
        if let Some(ref pre) = prefix {
            if !e.path.starts_with(pre) {
                continue;
            }
        }
        let path_hay = if case_sensitive {
            e.path.clone()
        } else {
            e.path.to_lowercase()
        };
        let mut hit_symbols: Vec<&str> = Vec::new();
        for s in &e.symbols {
            let matched = if let Some(ref re) = regex_opt {
                re.is_match(s)
            } else {
                let sym_lower = if case_sensitive {
                    None
                } else {
                    Some(s.to_lowercase())
                };
                if case_sensitive {
                    s.contains(needle)
                } else {
                    sym_lower.as_ref().unwrap().contains(&needle_cmp)
                }
            };
            if matched {
                hit_symbols.push(s.as_str());
            }
        }
        let path_match = if let Some(ref re) = regex_opt {
            re.is_match(&e.path)
        } else if case_sensitive {
            e.path.contains(needle)
        } else {
            path_hay.contains(&needle_cmp)
        };

        if path_match || !hit_symbols.is_empty() {
            n += 1;
            out.push_str(&format!("{}\n", e.path));
            if !hit_symbols.is_empty() {
                let shown: Vec<_> = hit_symbols.into_iter().take(24).collect();
                out.push_str(&format!("  symbols: {}\n", shown.join(", ")));
            }
            out.push('\n');
        }
    }

    if n == 0 {
        out.push_str("(no matches)\n");
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} entries: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));

    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Get the status of the code index for a project
///
/// Returns information about whether the index exists, its size, version,
/// generation timestamp, number of files indexed, and project root.
pub fn code_index_status_work(project_root: PathBuf) -> String {
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_STATUS\nexists: false\nexpected: {}\nhint: run code_index_build\n",
            path.display()
        );
    }
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index: {e}"),
    };
    format!(
        "CODE_INDEX_STATUS\nexists: true\npath: {}\nsize_bytes: {}\nversion: {}\ngenerated_ts_ms: {}\nfiles_indexed: {}\nproject_root: {}\n",
        path.display(),
        meta.len(),
        idx.version,
        idx.generated_ts_ms,
        idx.files_indexed,
        idx.project_root
    )
}

/// Symbols for one indexed file (relative path under project root).
pub fn code_index_file_symbols_work(project_root: PathBuf, file_rel: String) -> String {
    let mut rel = file_rel.trim().replace('\\', "/");
    rel = rel.trim_start_matches("./").to_string();

    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_FILE_SYMBOLS\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    for e in &idx.entries {
        if e.path == rel {
            let mut out = String::from("CODE_INDEX_FILE_SYMBOLS\n");
            out.push_str(&format!("path: {}\n", e.path));
            out.push_str(&format!("symbol_count: {}\n\n", e.symbols.len()));
            for s in &e.symbols {
                out.push_str(s);
                out.push('\n');
            }
            let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
            return out;
        }
    }

    format!(
        "CODE_INDEX_FILE_SYMBOLS\nnot_found: true\nrequested: {rel}\nhint: paths are relative to project root as stored in the index (forward slashes).\n"
    )
}

/// Aggregate stats over the persisted index (lightweight code-database view).
pub fn code_index_aggregate_stats_work(project_root: PathBuf) -> String {
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_AGGREGATE_STATS\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let total_symbols: usize = idx.entries.iter().map(|e| e.symbols.len()).sum();
    let mut by_file: Vec<(&CodeIndexEntry, usize)> =
        idx.entries.iter().map(|e| (e, e.symbols.len())).collect();
    by_file.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.path.cmp(&b.0.path)));
    by_file.truncate(30);

    let mut out = String::from("CODE_INDEX_AGGREGATE_STATS\n");
    out.push_str(&format!("files_with_symbols: {}\n", idx.files_indexed));
    out.push_str(&format!("total_symbol_rows: {total_symbols}\n"));
    out.push_str(&format!("index_ts_ms: {}\n\n", idx.generated_ts_ms));
    out.push_str("top_files_by_symbol_count (max 30):\n");
    for (e, n) in by_file {
        out.push_str(&format!("  {:4}  {}\n", n, e.path));
    }
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// List indexed file paths whose relative path starts with `prefix` (path prefix match only).
pub fn code_index_paths_prefix_work(project_root: PathBuf, prefix: String, limit: usize) -> String {
    let limit = limit.clamp(1, 500);
    let pre = prefix.trim().replace('\\', "/");

    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_PATHS_PREFIX\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let mut out = String::from("CODE_INDEX_PATHS_PREFIX\n");
    out.push_str(&format!("prefix: {:?}\nmatches (max {}):\n\n", pre, limit));
    let mut n = 0usize;
    for e in &idx.entries {
        if n >= limit {
            break;
        }
        if pre.is_empty() || e.path.starts_with(&pre) {
            n += 1;
            out.push_str(&format!("{}\n", e.path));
        }
    }
    if n == 0 {
        out.push_str("(no paths matched)\n");
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} entries: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Histogram of indexed paths by file extension (from stored relative paths).
pub fn code_index_extension_histogram_work(project_root: PathBuf, limit: usize) -> String {
    let limit = limit.clamp(1, 80);
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_EXTENSION_HISTOGRAM\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let mut counts: HashMap<String, usize> = HashMap::new();
    for e in &idx.entries {
        let ext = std::path::Path::new(&e.path)
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.to_ascii_lowercase())
            .unwrap_or_else(|| "(none)".to_string());
        *counts.entry(ext).or_insert(0) += 1;
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(limit);

    let mut out = String::from("CODE_INDEX_EXTENSION_HISTOGRAM\n");
    out.push_str(&format!(
        "files_indexed: {} unique_extensions_shown: {}\n\n",
        idx.files_indexed,
        v.len()
    ));
    for (ext, n) in v {
        if ext == "(none)" {
            out.push_str(&format!("{n:6}  (no extension)\n"));
        } else {
            out.push_str(&format!("{n:6}  .{ext}\n"));
        }
    }
    out.push_str(&format!("\nindex_ts_ms: {}\n", idx.generated_ts_ms));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Most frequent symbol strings across the index (row counts; same name in one file counts once per listed symbol).
pub fn code_index_symbol_frequency_work(project_root: PathBuf, limit: usize) -> String {
    let limit = limit.clamp(1, 200);
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_SYMBOL_FREQUENCY\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let mut counts: HashMap<String, usize> = HashMap::new();
    for e in &idx.entries {
        for s in &e.symbols {
            if s.len() > 96 {
                continue;
            }
            *counts.entry(s.clone()).or_insert(0) += 1;
        }
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(limit);

    let mut out = String::from("CODE_INDEX_SYMBOL_FREQUENCY\n");
    out.push_str(&format!(
        "unique_symbols_shown: {} (max {})\n\n",
        v.len(),
        limit
    ));
    for (name, n) in v {
        out.push_str(&format!("{n:6}  {name}\n"));
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} files: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// List indexed paths whose relative path contains `needle` (substring; not full regex).
pub fn code_index_path_substring_work(
    project_root: PathBuf,
    needle: String,
    limit: usize,
    case_sensitive: bool,
) -> String {
    let limit = limit.clamp(1, 500);
    let n = needle.trim();
    if n.is_empty() {
        return "Error: needle must not be empty".to_string();
    }
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_PATH_SUBSTRING\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };

    let n_cmp = if case_sensitive {
        n.to_string()
    } else {
        n.to_lowercase()
    };
    let mut out = String::from("CODE_INDEX_PATH_SUBSTRING\n");
    out.push_str(&format!(
        "needle: {:?} case_sensitive: {} max: {}\n\n",
        n, case_sensitive, limit
    ));
    let mut c = 0usize;
    for e in &idx.entries {
        if c >= limit {
            break;
        }
        let hit = if case_sensitive {
            e.path.contains(n)
        } else {
            e.path.to_lowercase().contains(&n_cmp)
        };
        if hit {
            c += 1;
            out.push_str(&format!("{}\n", e.path));
        }
    }
    if c == 0 {
        out.push_str("(no paths matched)\n");
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} entries: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Unique symbol names starting with `prefix` (fast navigation vs full code_index_search).
pub fn code_index_symbol_prefix_work(
    project_root: PathBuf,
    prefix: String,
    case_sensitive: bool,
    limit: usize,
) -> String {
    let limit = limit.clamp(1, 500);
    let pre = prefix.trim();
    if pre.is_empty() {
        return "Error: prefix must not be empty".to_string();
    }
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_SYMBOL_PREFIX\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };
    let pre_cmp = if case_sensitive {
        pre.to_string()
    } else {
        pre.to_lowercase()
    };
    let mut set: BTreeSet<String> = BTreeSet::new();
    for e in &idx.entries {
        for s in &e.symbols {
            let hit = if case_sensitive {
                s.starts_with(pre)
            } else {
                s.to_lowercase().starts_with(&pre_cmp)
            };
            if hit {
                set.insert(s.clone());
            }
        }
    }
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v.truncate(limit);
    let mut out = String::from("CODE_INDEX_SYMBOL_PREFIX\n");
    out.push_str(&format!(
        "prefix: {:?} case_sensitive: {} unique_matches_shown: {} (cap {})\n\n",
        pre,
        case_sensitive,
        v.len(),
        limit
    ));
    for name in &v {
        out.push_str(name);
        out.push('\n');
    }
    if v.is_empty() {
        out.push_str("(no symbols matched prefix)\n");
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} files_indexed: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Symbols that appear in at least `min_files` distinct indexed paths (collisions / shared names).
pub fn code_index_duplicate_symbols_work(
    project_root: PathBuf,
    min_files: usize,
    limit: usize,
) -> String {
    let min_files = min_files.clamp(2, 50);
    let limit = limit.clamp(1, 200);
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_DUPLICATE_SYMBOLS\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };
    let mut sym_to_files: HashMap<String, BTreeSet<String>> = HashMap::new();
    for e in &idx.entries {
        for s in &e.symbols {
            sym_to_files
                .entry(s.clone())
                .or_default()
                .insert(e.path.clone());
        }
    }
    let mut rows: Vec<(String, usize, Vec<String>)> = sym_to_files
        .into_iter()
        .filter(|(_, paths)| paths.len() >= min_files)
        .map(|(k, paths)| {
            let n = paths.len();
            let mut v: Vec<String> = paths.into_iter().collect();
            v.sort();
            (k, n, v)
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.truncate(limit);
    let mut out = String::from("CODE_INDEX_DUPLICATE_SYMBOLS\n");
    out.push_str(&format!(
        "min_files: {} symbols_shown: {} (cap {})\n\n",
        min_files,
        rows.len(),
        limit
    ));
    if rows.is_empty() {
        out.push_str("(no symbols matched min_files threshold)\n");
    }
    for (name, n, paths) in rows {
        out.push_str(&format!("{name}  (files={n})\n"));
        for p in paths.iter().take(12) {
            out.push_str(&format!("  - {p}\n"));
        }
        if paths.len() > 12 {
            out.push_str(&format!("  ... and {} more paths\n", paths.len() - 12));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "index_ts_ms: {} files_indexed: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Indexed paths whose symbol list is empty (config/data files or extractor gaps).
pub fn code_index_paths_without_symbols_work(project_root: PathBuf, limit: usize) -> String {
    let limit = limit.clamp(1, 500);
    let path = index_path(&project_root);
    if !path.exists() {
        return format!(
            "CODE_INDEX_PATHS_WITHOUT_SYMBOLS\nNo index at {}. Run code_index_build first.\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error reading index: {e}"),
    };
    let idx: CodeIndexFile = match serde_json::from_str(&raw) {
        Ok(i) => i,
        Err(e) => return format!("Error parsing index JSON: {e}"),
    };
    let mut paths: Vec<String> = idx
        .entries
        .iter()
        .filter(|e| e.symbols.is_empty())
        .map(|e| e.path.clone())
        .collect();
    paths.sort();
    let total = paths.len();
    paths.truncate(limit);
    let mut out = String::from("CODE_INDEX_PATHS_WITHOUT_SYMBOLS\n");
    out.push_str(&format!(
        "empty_symbol_paths_total: {} shown: {} (cap {})\n\n",
        total,
        paths.len(),
        limit
    ));
    for p in &paths {
        out.push_str(p);
        out.push('\n');
    }
    if paths.is_empty() {
        out.push_str("(every indexed file has at least one extracted symbol)\n");
    }
    out.push_str(&format!(
        "\nindex_ts_ms: {} files_indexed: {}\n",
        idx.generated_ts_ms, idx.files_indexed
    ));
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}
