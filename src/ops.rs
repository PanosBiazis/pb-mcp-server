use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024;
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

pub fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }
    let t: String = s.chars().take(max).collect();
    (t, true)
}

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
    if buf.iter().any(|&b| b == 0) {
        return None;
    }
    String::from_utf8(buf).ok()
}

fn normalize_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

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
                    .map(|p| normalize_rel_path(p))
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
    let mut result = format!("SEARCH_ROOT: {}\nMATCHES: {hits}\n\n{out}", search_root.display());
    if clipped {
        result.push_str(&format!(
            "\n\n[output truncated to {MAX_RESPONSE_CHARS} characters]"
        ));
    }
    result
}

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

pub fn tree_work(search_root: PathBuf, max_depth: usize) -> String {
    let max_depth = max_depth.max(1).min(12);
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
    walk(
        &search_root,
        "",
        1,
        max_depth,
        &mut lines,
        &mut count,
    );

    if count >= MAX_TREE_ENTRIES {
        lines.push(format!(
            "\n[truncated: max {MAX_TREE_ENTRIES} tree entries; lower max_depth or scope `path`]"
        ));
    }
    lines.join("\n")
}

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
            body.push_str("\n");
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

pub fn path_rel_to_repo(repo: &Path, abs_file: &Path) -> Option<String> {
    abs_file
        .strip_prefix(repo)
        .ok()
        .map(|p| normalize_rel_path(p))
}

pub fn git_log_work(
    repo: PathBuf,
    limit: usize,
    path_in_repo: Option<String>,
    oneline: bool,
) -> String {
    let limit = limit.clamp(1, 150);
    let mut cmd = Command::new("git");
    cmd.current_dir(&repo);
    cmd.args(["log", "--no-color", "-n"])
        .arg(limit.to_string());
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
    let partial_first_line =
        truncated_on_disk && !text.is_empty() && !text.starts_with('\n');
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
            if meta.is_dir() {
                "directory"
            } else {
                "other"
            }
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
    let has_nul = buf.iter().any(|&b| b == 0);
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

/// Non-secret `MINT_*` / gate env visibility for agents (paths redacted).
pub fn mcp_env_hints_work() -> String {
    const KEYS: &[&str] = &[
        "MINT_NETWORK_PROFILE",
        "MINT_PROJECT_ROOT",
        "PROJECT_ROOT",
        "MINT_BUS_READ_ONLY",
        "MINT_BUS_MAX_SENDS_PER_WINDOW",
        "MINT_BUS_RATE_WINDOW_MS",
        "MINT_TERMINAL_ENABLE",
        "MINT_TERMINAL_ALLOW",
        "MINT_GIT_WRITE_ENABLE",
        "MINT_GIT_PUSH_ENABLE",
        "MINT_GIT_REMOTE_ENABLE",
        "MINT_PROJECT_TEST_ENABLE",
    ];
    let mut out = String::from(
        "MCP_ENV_HINTS\n(Paths redacted; use workspace_context for resolved project root.)\n\n",
    );
    for k in KEYS {
        match std::env::var(k) {
            Ok(v) => {
                if *k == "MINT_PROJECT_ROOT" || *k == "PROJECT_ROOT" {
                    out.push_str(&format!("{k}=<<set, len {} chars>>\n", v.len()));
                } else {
                    out.push_str(&format!("{k}={v}\n"));
                }
            }
            Err(_) => out.push_str(&format!("{k}=<unset>\n")),
        }
    }
    out.push_str("\nMINT_NETWORK_PROFILE: unset or local = trusted LAN (default) — full tool surface with MINT_* gates. Value public|internet|wan = internet-facing MCP: bus/terminal/git writes/thinking_session I/O/code_index_build/convert file output blocked; use workspace_context section ai_agent_tool_visibility.\n");
    out.push_str("MINT_PROJECT_TEST_ENABLE: unset/false = project_cargo_test, project_cargo_check, project_pytest blocked even on local profile (extra safety). Set 1/true to allow those tools when MINT_TERMINAL_ENABLE=1 and cargo/pytest are in MINT_TERMINAL_ALLOW.\n");
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
        "network_profile: {} (MINT_NETWORK_PROFILE=local|public). public: no bus writes (incl. subagent_task/done), terminal_run/terminal_run_stdin/gh/glab PR CLI, git push/fetch/pull/add/commit/branch-create/worktree-add/checkout/stash, manifest write, code_index_build, thinking_session read/write, workspace_temp_*, screenshot_capture, ocr_tesseract, ollama_list_models, or convert_format output_path.\n",
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
Env: MINT_BUS_READ_ONLY=1; MINT_BUS_MAX_SENDS_PER_WINDOW (0=off), MINT_BUS_RATE_WINDOW_MS (fan-out/pipeline use N slots).\n",
        crate::bus::MAX_AGENT_MESSAGE_BODY_CHARS
    ));
    out.push_str(&format!(
        "Context shaping: compress_* (+ simplify_whitespace, markdown_links, unified_diff_stats, strip_tags, ws_collapse, drop_lines_matching_regex, dedent, truncate_middle, lines head/tail…); summarize_* (+ top_terms, duplicate_lines, rank_lines_by_keywords, bullets, markdown_toc, json_structure, delimited_header…); text_levenshtein_distance / text_djb2_hash; compress_gzip_base64 / decompress_gzip_base64 (binary-safe blobs); text_stats; pack_context; convert_format (+ wrap_lines, spaces_to_tabs, remove_empty_lines, dedupe_adjacent_lines, slugify…, max {} bytes/file); codebase_*; code_index_build/search/symbol_prefix/duplicate_symbols/path_substring/aggregate_stats (regex optional); gh_pr_* / glab_mr_*; gitkraken_cloud_catalog + hosted_tool_redirect; terminal_program_version; ai_stack_probe / screenshot_capture / ocr_tesseract / ollama_list_models (optional CLIs + gates); workspace_temp_*; mcp_env_hints; server_sleep (optional reason), wait_until_timestamp, wait_for_path (poll jitter); agent waits.\n",
        MAX_CONVERT_BYTES
    ));
    let term_on = terminal_enabled();
    let gitw = git_write_enabled();
    let git_remote = git_remote_enabled();
    let pt = project_test_tools_enabled();
    out.push_str(&format!(
        "Optional gates: terminal_run / terminal_run_stdin / terminal_which need MINT_TERMINAL_ENABLE=1 and MINT_TERMINAL_ALLOW (comma list); project_cargo_test / project_cargo_check / project_pytest also need MINT_PROJECT_TEST_ENABLE=1; git_checkout / git_stash_push need MINT_GIT_WRITE_ENABLE=1; git_fetch / git_pull need MINT_GIT_REMOTE_ENABLE=1. terminal_enabled={term_on} git_write_enabled={gitw} git_remote_enabled={git_remote} project_test_gate={pt}\n"
    ));
    out.push_str("\nPaths are resolved under project_root; absolute paths must stay inside the root.\n");
    out.push_str("\n--- ai_agent_tool_visibility ---\n");
    if is_public_profile {
        out.push_str("deployment: public_internet\n");
        out.push_str("philosophy: read-biased, minimal side effects. Prefer grep/read_file/compress/summarize/thinking_*/plan_*/reasoning_* (stateless text), agent_coordination_playbook, ai_inference_integration_guide, code_index reads, agent inbox/thread reads, capped waits, terminal_which / terminal_program_version / ai_stack_probe (if terminal enabled), mcp_env_hints, gitkraken_cloud_catalog.\n");
        out.push_str("blocked: agent_send and all bus mutators, terminal_run, terminal_run_stdin, gh_pr_*, glab_mr_*, git writes/remotes, codebase_manifest_write, code_index_build, thinking_session_*, workspace_temp_*, screenshot_capture, ocr_tesseract, ollama_list_models, convert_format with output_path.\n");
    } else {
        out.push_str("deployment: local_network (trusted)\n");
        out.push_str("philosophy: full catalog subject to MINT_* gates. terminal_run / terminal_run_stdin may spawn subprocesses that read/write the repo per argv or stdin (allowlist MINT_TERMINAL_ALLOW). Optional OCR/LLM CLIs: ocr_tesseract (tesseract), ollama_list_models; stack probe: ai_stack_probe; gzip helpers compress_gzip_base64 / decompress_gzip_base64 for compact bus/file handoffs. Use agent_coordination_playbook for patterns; agent_fan_out / agent_send_pipeline / agent_subagent_* for parallel vs serial coordination; agent_inbox_wait / agent_topic_wait for sync; since_ts_ms for async polling.\n");
        out.push_str("cross_platform: Rust fs + git CLI + optional gh/glab; PATH resolved via terminal_which (which/where).\n");
    }
    out.push_str("hint: mcp_env_hints — non-secret MINT_* / gate variables (project root values redacted).\n");
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
    s
}

/// Cheap UTF-8 stats for prompt budgeting (no tokenization model).
pub fn text_stats_work(text: String) -> String {
    let bytes = text.len();
    let chars = text.chars().count();
    let lines = text.lines().count();
    let non_empty = text.lines().filter(|l| !l.trim().is_empty()).count();
    let para_like = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    let words = text
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .count();
    let tok4 = chars / 4;
    let tok13 = chars.saturating_mul(10) / 13; // ~ chars/1.3 token heuristic (very rough)
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
    let words = text
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .count();
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

/// Replace email-shaped tokens (privacy when sharing logs).
pub fn compress_redact_emails_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let re = match Regex::new(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,63}\b") {
        Ok(r) => r,
        Err(_) => return "Error: email regex".to_string(),
    };
    let body = re.replace_all(&text, "[email]").to_string();
    let (body, _) = truncate_chars(&body, max_chars);
    format!("COMPRESS_REDACT_EMAILS max_chars={max_chars}\n\n{body}")
}

/// Strip leading YAML-style `---` … `---` block if present (common in markdown READMEs).
pub fn compress_markdown_frontmatter_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let lines: Vec<&str> = text.lines().collect();
    let mut stripped = false;
    let mut skip_lines = 0usize;
    let rest = if lines.first().is_some_and(|l| l.trim() == "---") {
        let mut i = 1usize;
        let mut found = false;
        while i < lines.len() {
            if lines[i].trim() == "---" {
                found = true;
                skip_lines = i + 1;
                break;
            }
            i += 1;
        }
        if found {
            stripped = true;
            lines[skip_lines..].join("\n")
        } else {
            text.clone()
        }
    } else {
        text.clone()
    };
    let (body, _) = truncate_chars(&rest, max_chars);
    format!(
        "COMPRESS_MD_FRONTMATTER stripped={stripped} skipped_lines={skip_lines} max_chars={max_chars}\n\n{body}"
    )
}

/// Group lines by their first `prefix_len` characters (Unicode-safe char count).
pub fn summarize_line_prefix_groups_work(
    text: String,
    prefix_len: usize,
    max_groups: usize,
    max_chars: usize,
) -> String {
    let prefix_len = prefix_len.clamp(1, 96);
    let max_groups = max_groups.clamp(1, 200);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in text.lines() {
        let p: String = line.chars().take(prefix_len).collect();
        let key = if p.is_empty() { "(empty)".to_string() } else { p };
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut v: Vec<(String, usize)> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(max_groups);
    let mut out = format!(
        "SUMMARIZE_LINE_PREFIX_GROUPS prefix_len={prefix_len} groups_shown={} (max {max_groups})\n\n",
        v.len()
    );
    for (pref, n) in v {
        out.push_str(&format!("{n:6}  {pref}\n"));
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Read-only reference for serial vs parallel agent patterns (no disk access).
pub fn agent_coordination_playbook_work() -> &'static str {
    "AGENT_COORDINATION_PLAYBOOK (read-only)\n\
     \n\
     Trust & network:\n\
     - MINT_NETWORK_PROFILE=public: bus writes, terminal_run, terminal_run_stdin, PR CLIs, risky git, manifest, code_index_build, thinking_session_*, convert output_path blocked or capped.\n\
     - Default local/LAN: full tool surface; still respect MINT_BUS_READ_ONLY and rate limits.\n\
     \n\
     Async (low latency, many round-trips):\n\
     - agent_send / agent_handoff then poll agent_inbox(for_agent) or agent_recent with since_ts_ms.\n\
     - agent_thread(topic) for shared topics; agent_multi_inbox for fan-in.\n\
     \n\
     Sync waits (one MCP call blocks):\n\
     - agent_inbox_wait, agent_topic_wait, ask_agents — shorter caps on public profile.\n\
     \n\
     Serial pipelines:\n\
     - agent_send_pipeline: ordered posts under one lock (deterministic handoffs).\n\
     \n\
     Parallel:\n\
     - agent_fan_out: same body to many recipients; combine with agent_multi_inbox.\n\
     - Distinct sub_agent ids per branch; correlate via correlation_id in agent_subagent_task.\n\
     \n\
     Subagents:\n\
     - agent_subagent_task (parent→child DM, kind=subagent_task) then agent_subagent_done (child→parent).\n\
     - plan_handoff_brief + thinking_plan_skeleton help structure task bodies.\n\
     \n\
     Cross-platform:\n\
     - Paths and bus file use UTF-8 JSONL; use forward slashes in bus topics/paths where possible.\n\
     \n\
     Compression before bus:\n\
     - compress_fenced_code_blocks, compress_redact_urls, compress_ws_collapse, compress_unified_diff_stats (on git diff text), pack_context, compress_gzip_base64 (binary-safe).\n\
     \n\
     Tests (trusted LAN + MINT_PROJECT_TEST_ENABLE=1 + terminal allowlist):\n\
     - project_cargo_test, project_cargo_check, project_pytest — bounded argv; use plan_test_strategy to design coverage before running.\n\
     \n\
     OCR / local ML bridges (no bundled weights in this binary):\n\
     - ai_stack_probe + ai_inference_integration_guide; ocr_tesseract (tesseract); ollama_list_models.\n\
     \n\
     Scratch + screenshots (trusted / LAN):\n\
     - workspace_temp_* under .mint-mcp/agent-temp/; screenshot_capture (grim / screencapture) → .mint-mcp/screenshots/*.png then ocr_tesseract.\n"
}

/// Static guide: this server does not ship neural weights; how to combine external ML/OCR/DL safely.
pub fn ai_inference_integration_guide_work() -> &'static str {
    "AI_INFERENCE_INTEGRATION_GUIDE (read-only)\n\
     \n\
     What this server provides:\n\
     - Deterministic text tools (compress_*, summarize_*, convert_format) and optional **CLI bridges** (tesseract, ollama) when you enable the terminal gate and allowlist.\n\
     - No PyTorch/TensorFlow/ONNX runtime inside this crate — keeps the MCP small and auditable.\n\
     \n\
     OCR:\n\
     - Install Tesseract + language packs on the host; add `tesseract` to MINT_TERMINAL_ALLOW; use tool `ocr_tesseract` on image paths under the project root (trusted/LAN deployments).\n\
     \n\
     Screenshots:\n\
     - `screenshot_capture` with `grim` (Wayland) or `screencapture` (macOS) writes `.mint-mcp/screenshots/*.png` when `grim`/`screencapture` are allowlisted; then chain `ocr_tesseract`. Use `workspace_temp_*` for scratch text between agents.\n\
     \n\
     Local LLM / embeddings:\n\
     - Use `ollama` (or similar) as a separate process; `ollama_list_models` inventories tags when allowlisted. On trusted hosts, `terminal_run_stdin` can pipe a prompt file or text into `ollama run <tag>` (keep timeouts tight). Prefer dedicated MCP servers for chat/completions if you need streaming auth.\n\
     \n\
     Heavy ML / DL:\n\
     - Run Python/R notebooks or services in your environment; expose results via files under the project root and use read_file, or wrap CLIs in terminal_run with a tight allowlist.\n\
     \n\
     Deterministic helpers (no weights):\n\
     - summarize_json_structure for API/JSON config skims; text_levenshtein_distance and text_djb2_hash for comparing strings or fingerprints; compress_simplify_whitespace / compress_markdown_links before bus posts.\n\
     \n\
     Security:\n\
     - MINT_NETWORK_PROFILE=public: bus mutations, ocr_tesseract, ollama_list_models, terminal_run_stdin, and unconstrained terminal_run stay blocked — use read-only probes like ai_stack_probe only if MINT_TERMINAL_ENABLE=1.\n\
     - Never pass secrets on the bus; redact with compress_redact_urls / compress_redact_emails before agent_send.\n"
}

/// `--version` snippets for common optional CLIs (same gates as terminal_program_version).
pub fn ai_stack_probe_work() -> String {
    const CANDIDATES: &[&str] = &["python3", "python", "tesseract", "ollama", "ffmpeg"];
    let mut out = String::from("AI_STACK_PROBE\n");
    out.push_str("Uses MINT_TERMINAL_ENABLE + MINT_TERMINAL_ALLOW per program (like terminal_program_version).\n\n");
    for p in CANDIDATES {
        if let Err(e) = terminal_run_precheck(p) {
            out.push_str(&format!("--- {p} ---\n  skip: {e}\n\n"));
            continue;
        }
        let body = terminal_program_version_work(p);
        let snippet: String = body.lines().take(14).collect::<Vec<_>>().join("\n");
        out.push_str(&format!("--- {p} ---\n{snippet}\n\n"));
    }
    out.push_str("next: add missing names to MINT_TERMINAL_ALLOW; on trusted hosts use ocr_tesseract or ollama_list_models.\n");
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

const MAX_OCR_IMAGE_BYTES: u64 = 40 * 1024 * 1024;

/// OCR via host `tesseract` CLI (image file under project root). Requires terminal gate + allowlist.
pub fn ocr_tesseract_file_work(path: &Path, lang: &str, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    if let Err(e) = terminal_run_precheck("tesseract") {
        return format!("Error: {e}");
    }
    if !path.is_file() {
        return format!("Error: not a file: {}", path.display());
    }
    if let Ok(m) = fs::metadata(path) {
        if m.len() > MAX_OCR_IMAGE_BYTES {
            return format!(
                "Error: file too large for OCR (max {} MiB)",
                MAX_OCR_IMAGE_BYTES / (1024 * 1024)
            );
        }
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "tif" | "tiff" | "webp" | "bmp" | "pnm" | "gif"
    ) {
        return format!(
            "Error: unsupported image extension .{ext} (allowed: png,jpg,jpeg,tif,tiff,webp,bmp,pnm,gif)"
        );
    }
    let lang = lang.trim();
    let lang = if lang.is_empty() { "eng" } else { lang };
    if !lang
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return "Error: lang must be ASCII alnum with optional _ -".to_string();
    }
    match Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .args(["-l", lang])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !o.status.success() {
                let (se, _) = truncate_chars(stderr.trim(), 4000);
                return format!(
                    "Error: tesseract exit={:?}\npath={}\nSTDERR:\n{se}\n",
                    o.status.code(),
                    path.display()
                );
            }
            let (body, clipped) = truncate_chars(stdout.trim(), max_chars);
            let mut s = format!(
                "OCR_TESSERACT path={} lang={lang} exit=0\n\n{body}\n",
                path.display()
            );
            if clipped {
                s.push_str(&format!("\n[truncated to {max_chars} chars]\n"));
            }
            if !stderr.trim().is_empty() {
                let (w, _) = truncate_chars(stderr.trim(), 1200);
                s.push_str(&format!("\n--- tesseract stderr (info) ---\n{w}\n"));
            }
            s
        }
        Err(e) => format!("Error: spawn tesseract: {e}"),
    }
}

/// `ollama list` for local model inventory (read-only subprocess). Same allowlist gate as `ollama` terminal use.
pub fn ollama_list_models_work() -> String {
    if let Err(e) = terminal_run_precheck("ollama") {
        return format!("Error: {e}");
    }
    match Command::new("ollama").args(["list"]).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let mut s = format!(
                "OLLAMA_LIST exit={:?}\n--- STDOUT ---\n",
                o.status.code()
            );
            let (so, c1) = truncate_chars(&stdout, 16_000);
            s.push_str(&so);
            if c1 {
                s.push_str("\n[stdout truncated]\n");
            }
            s.push_str("\n--- STDERR ---\n");
            let (se, c2) = truncate_chars(&stderr, 4000);
            s.push_str(&se);
            if c2 {
                s.push_str("\n[stderr truncated]\n");
            }
            if !o.status.success() {
                s.push_str("\n(warning: non-zero exit)\n");
            }
            let (s, _) = truncate_chars(&s, MAX_RESPONSE_CHARS);
            s
        }
        Err(e) => format!("Error: spawn ollama: {e}"),
    }
}

/// Rank lines by total whole-token matches for given keywords (case-insensitive). Deterministic; not neural.
pub fn summarize_rank_lines_by_keywords_work(
    text: String,
    keywords: Vec<String>,
    top_lines: usize,
    max_chars: usize,
) -> String {
    let top_lines = top_lines.clamp(1, 400);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let kws: Vec<String> = keywords
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .take(48)
        .collect();
    if kws.is_empty() {
        return "Error: pass at least one non-empty keyword".to_string();
    }
    let mut scored: Vec<(usize, String)> = Vec::new();
    for line in text.lines() {
        let lower = line.to_lowercase();
        let mut score = 0usize;
        for tok in lower.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if tok.is_empty() {
                continue;
            }
            if kws.iter().any(|k| k == tok) {
                score += 1;
            }
        }
        scored.push((score, line.to_string()));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut out = String::from("SUMMARIZE_RANK_LINES_BY_KEYWORDS\n");
    out.push_str(&format!(
        "keywords: {:?}\nshowing up to {} lines (by token-hit score)\n\n",
        kws, top_lines
    ));
    let mut n = 0usize;
    for (sc, line) in scored.iter().take(top_lines) {
        n += 1;
        let (preview, _) = truncate_chars(line, 500);
        out.push_str(&format!("{sc:5}  {preview}\n"));
    }
    if n == 0 {
        out.push_str("(no lines)\n");
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

const AGENT_TEMP_REL: &str = ".mint-mcp/agent-temp";
const SCREENSHOT_REL: &str = ".mint-mcp/screenshots";

fn sanitize_agent_scratch_filename(name: &str) -> Result<String, String> {
    let t = name.trim();
    if t.is_empty() || t.len() > 128 {
        return Err("name must be 1..=128 chars".into());
    }
    if t.contains('/') || t.contains('\\') || t.contains("..") {
        return Err("name must not contain path separators or ..".into());
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("name must be ASCII alnum, -, _, . only".into());
    }
    if t.starts_with('.') {
        return Err("name must not start with '.'".into());
    }
    Ok(t.to_string())
}

fn sanitize_screenshot_filename(name: &str) -> Result<String, String> {
    let t = name.trim();
    if !t.to_ascii_lowercase().ends_with(".png") {
        return Err("output_name must end with .png".into());
    }
    sanitize_agent_scratch_filename(t)
}

/// Scratch text files under `.mint-mcp/agent-temp/` (trusted deployments).
pub fn workspace_temp_write_work(
    root: PathBuf,
    name: String,
    content: String,
    max_store_bytes: usize,
) -> String {
    let max_store_bytes = max_store_bytes.clamp(1, 512 * 1024);
    let name = match sanitize_agent_scratch_filename(&name) {
        Ok(n) => n,
        Err(e) => return format!("Error: {e}"),
    };
    let bytes = content.as_bytes();
    if bytes.len() > max_store_bytes {
        return format!("Error: content exceeds max_store_bytes={max_store_bytes}");
    }
    let dir = root.join(AGENT_TEMP_REL);
    if let Err(e) = fs::create_dir_all(&dir) {
        return format!("Error: create temp dir: {e}");
    }
    let path = dir.join(&name);
    match fs::write(&path, bytes) {
        Ok(()) => format!(
            "WORKSPACE_TEMP_WRITE\npath: {}\nbytes: {}\n",
            path.display(),
            bytes.len()
        ),
        Err(e) => format!("Error: write {}: {e}", path.display()),
    }
}

/// Read a scratch file from `.mint-mcp/agent-temp/`.
pub fn workspace_temp_read_work(root: PathBuf, name: String, max_read_bytes: usize) -> String {
    let max_read_bytes = max_read_bytes.clamp(1, 512 * 1024);
    let name = match sanitize_agent_scratch_filename(&name) {
        Ok(n) => n,
        Err(e) => return format!("Error: {e}"),
    };
    let path = root.join(AGENT_TEMP_REL).join(&name);
    if !path.is_file() {
        return format!("Error: not a file: {}", path.display());
    }
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    if meta.len() > max_read_bytes as u64 {
        return format!(
            "Error: file too large for this call (max {max_read_bytes} bytes)"
        );
    }
    match fs::read_to_string(&path) {
        Ok(s) => {
            let (body, clipped) = truncate_chars(&s, MAX_RESPONSE_CHARS);
            let mut o = format!(
                "WORKSPACE_TEMP_READ path={}\nbytes_on_disk={}\n\n{body}\n",
                path.display(),
                meta.len()
            );
            if clipped {
                o.push_str("\n[truncated to MAX_RESPONSE_CHARS]\n");
            }
            o
        }
        Err(e) => format!("Error: read {}: {e}", path.display()),
    }
}

/// List files in `.mint-mcp/agent-temp/`.
pub fn workspace_temp_list_work(root: PathBuf, limit: usize) -> String {
    let limit = limit.clamp(1, 500);
    let dir = root.join(AGENT_TEMP_REL);
    if !dir.is_dir() {
        return "WORKSPACE_TEMP_LIST\n(empty — no agent-temp directory yet)\n".to_string();
    }
    let mut names: Vec<String> = match fs::read_dir(&dir) {
        Ok(rd) => rd
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(e) => return format!("Error: {e}"),
    };
    names.sort();
    let mut out = String::from("WORKSPACE_TEMP_LIST\n");
    out.push_str(&format!(
        "dir: {}\nfiles: {}\n\n",
        dir.display(),
        names.len()
    ));
    for n in names.iter().take(limit) {
        out.push_str(n);
        out.push('\n');
    }
    if names.len() > limit {
        out.push_str(&format!("\n... and {} more\n", names.len() - limit));
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// Remove one scratch file from `.mint-mcp/agent-temp/`.
pub fn workspace_temp_delete_work(root: PathBuf, name: String) -> String {
    let name = match sanitize_agent_scratch_filename(&name) {
        Ok(n) => n,
        Err(e) => return format!("Error: {e}"),
    };
    let path = root.join(AGENT_TEMP_REL).join(&name);
    if !path.exists() {
        return format!("WORKSPACE_TEMP_DELETE\nnot_found: {}\n", path.display());
    }
    match fs::remove_file(&path) {
        Ok(()) => format!("WORKSPACE_TEMP_DELETE\nremoved: {}\n", path.display()),
        Err(e) => format!("Error: remove {}: {e}", path.display()),
    }
}

/// Capture full screen to `.mint-mcp/screenshots/<name>.png` via `grim` or `screencapture`.
pub fn screenshot_capture_work(root: PathBuf, backend: &str, output_name: String) -> String {
    let name = match sanitize_screenshot_filename(&output_name) {
        Ok(n) => n,
        Err(e) => return format!("Error: {e}"),
    };
    let dir = root.join(SCREENSHOT_REL);
    if let Err(e) = fs::create_dir_all(&dir) {
        return format!("Error: create screenshot dir: {e}");
    }
    let out_path = dir.join(&name);
    let backend = backend.trim().to_ascii_lowercase();
    match backend.as_str() {
        "grim" => {
            if let Err(e) = terminal_run_precheck("grim") {
                return format!("Error: {e}");
            }
            match Command::new("grim")
                .args(["-t", "png"])
                .arg(&out_path)
                .output()
            {
                Ok(o) => {
                    if !o.status.success() {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        return format!("Error: grim exit {:?}\n{}", o.status.code(), stderr);
                    }
                    format!(
                        "SCREENSHOT_CAPTURE backend=grim\nwrote: {}\nnext: ocr_tesseract with path relative to project (under .mint-mcp/screenshots/)\n",
                        out_path.display()
                    )
                }
                Err(e) => format!("Error: spawn grim: {e}"),
            }
        }
        "screencapture" | "macos" | "osx" => {
            if let Err(e) = terminal_run_precheck("screencapture") {
                return format!("Error: {e}");
            }
            match Command::new("screencapture")
                .args(["-x", "-t", "png"])
                .arg(&out_path)
                .output()
            {
                Ok(o) => {
                    if !o.status.success() {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        return format!(
                            "Error: screencapture exit {:?}\n{}",
                            o.status.code(),
                            stderr
                        );
                    }
                    format!(
                        "SCREENSHOT_CAPTURE backend=screencapture\nwrote: {}\n",
                        out_path.display()
                    )
                }
                Err(e) => format!("Error: spawn screencapture: {e}"),
            }
        }
        _ => "Error: backend must be \"grim\" (Wayland) or \"screencapture\" (macOS). On Windows use a host-specific tool via terminal_run with a narrow allowlist.".to_string(),
    }
}

/// Drop lines that match a regex (inverse of grep keep); lines are tested per line without multiline mode.
pub fn compress_drop_lines_matching_regex_work(
    text: String,
    pattern: String,
    case_insensitive: bool,
    max_chars: usize,
) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut b = RegexBuilder::new(&pattern);
    b.case_insensitive(case_insensitive);
    let re = match b.build() {
        Ok(r) => r,
        Err(e) => return format!("Error: invalid regex: {e}"),
    };
    let mut kept = 0usize;
    let mut dropped = 0usize;
    let mut lines_out: Vec<String> = Vec::new();
    for line in text.lines() {
        if re.is_match(line) {
            dropped += 1;
        } else {
            kept += 1;
            lines_out.push(line.to_string());
        }
    }
    let body = lines_out.join("\n");
    let (body, clipped) = truncate_chars(&body, max_chars);
    let mut o = format!(
        "COMPRESS_DROP_LINES_MATCHING_REGEX pattern={pattern:?} case_insensitive={case_insensitive} kept_lines={kept} dropped_lines={dropped} max_chars={max_chars}\n\n{body}\n"
    );
    if clipped {
        o.push_str("\n[truncated]\n");
    }
    o
}

/// Collapse runs of whitespace within each line (lines preserved).
pub fn compress_ws_collapse_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut v: Vec<String> = Vec::new();
    for line in text.lines() {
        v.push(line.split_whitespace().collect::<Vec<_>>().join(" "));
    }
    let body = v.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("COMPRESS_WS_COLLAPSE max_chars={max_chars}\n\n{body}")
}

/// Deterministic bag-of-words style term frequencies (English stopword filter; no ML).
pub fn summarize_top_terms_work(text: String, min_len: usize, max_terms: usize, max_chars: usize) -> String {
    let min_len = min_len.clamp(2, 32);
    let max_terms = max_terms.clamp(1, 80);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    const STOP: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
        "one", "our", "out", "has", "have", "been", "this", "that", "with", "from", "they",
        "will", "would", "there", "their", "what", "which", "when", "your", "about", "into",
        "than", "then", "them", "these", "those", "also", "just", "like", "some", "such", "over",
        "http", "https", "www", "com", "org",
    ];
    let mut counts: HashMap<String, usize> = HashMap::new();
    let lower = text.to_lowercase();
    for raw in lower.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '\'') {
        let t = raw.trim_matches('\'');
        if t.len() < min_len {
            continue;
        }
        if STOP.iter().any(|s| *s == t) {
            continue;
        }
        *counts.entry(t.to_string()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(max_terms);
    let mut out = String::from("SUMMARIZE_TOP_TERMS\n");
    out.push_str(&format!("min_token_len: {min_len} max_terms: {max_terms}\n\n"));
    for (w, c) in pairs {
        out.push_str(&format!("{c:6}  {w}\n"));
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Remove angle-bracket segments (heuristic tag stripper; keeps `<` that do not look like markup starts).
pub fn compress_strip_tags_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if c == '<' {
            match it.peek().copied() {
                Some(n) if n == '!' || n == '?' || n == '/' || n.is_alphabetic() => {
                    let mut buf = String::new();
                    buf.push('<');
                    if let Some(first_inner) = it.next() {
                        buf.push(first_inner);
                    }
                    let mut closed = false;
                    while let Some(c2) = it.next() {
                        buf.push(c2);
                        if buf.len() > 1024 {
                            break;
                        }
                        if c2 == '>' {
                            closed = true;
                            break;
                        }
                    }
                    if closed {
                        out.push(' ');
                    } else {
                        out.push_str(&buf);
                    }
                }
                _ => out.push('<'),
            }
        } else {
            out.push(c);
        }
    }
    let normalized: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let (body, _) = truncate_chars(&normalized, max_chars);
    format!("COMPRESS_STRIP_TAGS max_chars={max_chars}\n\n{body}")
}

/// Lines that appear at least `min_occurrences` times (exact string match per line, including whitespace).
pub fn summarize_duplicate_lines_work(
    text: String,
    min_occurrences: usize,
    max_entries: usize,
    max_chars: usize,
) -> String {
    let min_occ = min_occurrences.max(2);
    let max_entries = max_entries.clamp(1, 200);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut m: HashMap<String, usize> = HashMap::new();
    for line in text.lines() {
        *m.entry(line.to_string()).or_insert(0) += 1;
    }
    let mut v: Vec<(String, usize)> = m.into_iter().filter(|(_, c)| *c >= min_occ).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(max_entries);
    let mut out = String::from("SUMMARIZE_DUPLICATE_LINES\n");
    out.push_str(&format!("min_occurrences: {min_occ} listed: {}\n\n", v.len()));
    for (line, c) in v {
        let preview = if line.chars().count() > 200 {
            format!("{}…", line.chars().take(200).collect::<String>())
        } else {
            line.clone()
        };
        out.push_str(&format!("{c:6}  {preview}\n"));
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Replace ```…``` fenced blocks with a placeholder line (preserves line structure outside fences).
pub fn compress_fenced_code_blocks_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(i) = rest.find("```") {
        out.push_str(&rest[..i]);
        out.push_str("\n[…fenced code block…]\n");
        rest = &rest[i + 3..];
        if let Some(j) = rest.find("```") {
            rest = &rest[j + 3..];
        } else {
            break;
        }
    }
    out.push_str(rest);
    let (body, _) = truncate_chars(&out, max_chars);
    format!("COMPRESS_FENCED_CODE_BLOCKS max_chars={max_chars}\n\n{body}")
}

/// Histogram of line lengths (by Unicode scalar count).
pub fn summarize_line_histogram_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut b0_40 = 0usize;
    let mut b41_120 = 0usize;
    let mut b121_400 = 0usize;
    let mut b401 = 0usize;
    let mut lines = 0usize;
    let mut chars = 0usize;
    for line in text.lines() {
        lines += 1;
        let n = line.chars().count();
        chars += n;
        match n {
            0..=40 => b0_40 += 1,
            41..=120 => b41_120 += 1,
            121..=400 => b121_400 += 1,
            _ => b401 += 1,
        }
    }
    let mut s = format!(
        "SUMMARIZE_LINE_HISTOGRAM\nlines: {lines}\ntotal_chars_in_lines: {chars}\n\nbucket_0_40_chars: {b0_40}\nbucket_41_120_chars: {b41_120}\nbucket_121_400_chars: {b121_400}\nbucket_401_plus_chars: {b401}\n"
    );
    s.push_str("\nhint: long-line buckets often benefit from compress_lines_tail / compress_truncate_middle.\n");
    let (s, _) = truncate_chars(&s, max_chars);
    s
}

/// List dependency keys from `Cargo.toml` `[dependencies]` (heuristic line scan; no full TOML parse).
pub fn codebase_cargo_dependency_names_work(manifest_path: PathBuf) -> String {
    if !manifest_path.is_file() {
        return format!(
            "Error: not a file: {}",
            manifest_path.display()
        );
    }
    let raw = match read_file_bytes_capped(&manifest_path, 256 * 1024) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => return format!("Error: {e}"),
    };
    let mut in_deps = false;
    let mut keys: Vec<String> = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if t.starts_with('[') {
            in_deps = t == "[dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(eq) = t.find('=') {
            let key = t[..eq]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            if !key.is_empty()
                && key
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                keys.push(key);
            }
        }
    }
    keys.sort();
    keys.dedup();
    let mut out = String::from("CODEBASE_CARGO_DEPENDENCIES\n");
    out.push_str(&format!("manifest: {}\ncount: {}\n\n", manifest_path.display(), keys.len()));
    for k in keys {
        out.push_str(&k);
        out.push('\n');
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// Prefix every line (e.g. `> ` for quotes, `// ` for comment simulation).
pub fn format_prefix_lines_work(text: String, prefix: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let p = if prefix.is_empty() {
        "> ".to_string()
    } else {
        prefix
    };
    let mut out = String::new();
    for line in text.lines() {
        out.push_str(&p);
        out.push_str(line);
        out.push('\n');
    }
    let (body, _) = truncate_chars(&out, max_chars);
    format!("FORMAT_PREFIX_LINES prefix={p:?} max_chars={max_chars}\n\n{body}")
}

fn escape_c_string_literal_body(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii_control() => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Run `program --version`, then `-V`, then `version` subcommand (first success). Same allowlist as terminal_run.
pub fn terminal_program_version_work(program: &str) -> String {
    if let Err(e) = terminal_run_precheck(program) {
        return format!("Error: {e}");
    }
    let p = program.trim();
    let attempts: &[&[&str]] = &[&["--version"], &["-V"], &["version"]];
    for args in attempts {
        match Command::new(p).args(*args).output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let body = if !stdout.trim().is_empty() {
                    stdout
                } else {
                    stderr
                };
                if o.status.success() || !body.trim().is_empty() {
                    let (b, clipped) = truncate_chars(body.trim(), 8000);
                    let mut s = format!(
                        "TERMINAL_PROGRAM_VERSION program={p} argv={:?}\nexit: {}\n\n{b}\n",
                        args,
                        o.status
                    );
                    if clipped {
                        s.push_str("\n[truncated]\n");
                    }
                    return s;
                }
            }
            Err(e) => return format!("Error: spawn {p}: {e}"),
        }
    }
    format!("Error: could not read version from {p} (--version, -V, version)")
}

/// Explains that a named GitKraken/GitLens **hosted** tool requires the official extension MCP (no API in this server).
pub fn gitkraken_hosted_tool_redirect_work(requested: &str) -> String {
    let name = requested.trim();
    if name.is_empty() || name.len() > 120 {
        return "Error: tool_name must be non-empty, max 120 chars".to_string();
    }
    const KNOWN: &[&str] = &[
        "pull_request_create",
        "pull_request_get_detail",
        "pull_request_get_comments",
        "pull_request_create_review",
        "pull_request_assigned_to_me",
        "issues_get_detail",
        "issues_add_comment",
        "issues_assigned_to_me",
        "gitkraken_workspace_list",
        "gitlens_launchpad",
        "gitlens_start_work",
        "gitlens_start_review",
        "gitlens_commit_composer",
    ];
    let lower = name.to_ascii_lowercase();
    if !KNOWN.iter().any(|k| *k == lower.as_str()) {
        return format!(
            "Error: unknown hosted tool {name:?}. Known: {}",
            KNOWN.join(", ")
        );
    }
    let mut s = format!("GITKRAKEN_HOSTED_REDIRECT\ntool: {name}\n\n");
    s.push_str("This operation is implemented by the GitLens/GitKraken MCP server with provider sign-in.\n");
    s.push_str("Enable that MCP in your client. This server provides local git (git_* / gk_git_*), gh/glab PR helpers, and gitkraken_cloud_catalog.\n");
    s
}

/// `gh pr list` with fixed argv (read-only). Same gates as terminal_run for `gh`.
pub fn gh_pr_list_work(cwd: PathBuf, limit: usize, state: &str) -> String {
    if let Err(e) = terminal_run_precheck("gh") {
        return format!("Error: {e}");
    }
    let lim = limit.clamp(1, 50);
    let st_owned = match state.trim().to_ascii_lowercase().as_str() {
        "open" | "closed" | "merged" | "all" => state.trim().to_ascii_lowercase(),
        _ => "open".to_string(),
    };
    let st = st_owned.as_str();
    let json_try = Command::new("gh")
        .current_dir(&cwd)
        .env("GH_PROMPT_DISABLED", "1")
        .env("CI", "1")
        .args([
            "pr",
            "list",
            "--limit",
            &lim.to_string(),
            "--state",
            st,
            "--json",
            "number,title,state,url,headRefName,baseRefName",
        ])
        .output();
    match json_try {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
            let mut s = format!(
                "GH_PR_LIST dir={} limit={lim} state={st}\n\n{body}",
                cwd.display()
            );
            if clipped {
                s.push_str(&format!("\n[truncated to {MAX_RESPONSE_CHARS} chars]\n"));
            }
            s
        }
        Ok(o) => {
            let plain = Command::new("gh")
                .current_dir(&cwd)
                .env("GH_PROMPT_DISABLED", "1")
                .env("CI", "1")
                .args(["pr", "list", "--limit", &lim.to_string(), "--state", st])
                .output();
            match plain {
                Ok(p) if p.status.success() => {
                    let stdout = String::from_utf8_lossy(&p.stdout);
                    let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
                    let mut s = format!(
                        "GH_PR_LIST dir={} limit={lim} state={st} (plain)\n\n{body}",
                        cwd.display()
                    );
                    if clipped {
                        s.push_str(&format!("\n[truncated]\n"));
                    }
                    s
                }
                Ok(p) => format!(
                    "Error: gh pr list failed.\nJSON try stderr: {}\nPlain try stderr: {}\n",
                    String::from_utf8_lossy(&o.stderr).trim(),
                    String::from_utf8_lossy(&p.stderr).trim()
                ),
                Err(e) => format!("Error: gh: {e}"),
            }
        }
        Err(e) => format!("Error: spawn gh: {e}"),
    }
}

/// `gh pr view <n> --json …` (read-only).
pub fn gh_pr_view_work(cwd: PathBuf, pr_number: &str) -> String {
    if let Err(e) = terminal_run_precheck("gh") {
        return format!("Error: {e}");
    }
    let pr = pr_number.trim();
    if pr.is_empty() || !pr.chars().all(|c| c.is_ascii_digit()) || pr.len() > 12 {
        return "Error: pr_number must be non-empty ASCII digits only".to_string();
    }
    match Command::new("gh")
        .current_dir(&cwd)
        .env("GH_PROMPT_DISABLED", "1")
        .env("CI", "1")
        .args([
            "pr",
            "view",
            pr,
            "--json",
            "title,body,state,url,author,commits,files",
        ])
        .output()
    {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
            let mut s = format!("GH_PR_VIEW dir={} pr={pr}\n\n{body}", cwd.display());
            if clipped {
                s.push_str(&format!("\n[truncated to {MAX_RESPONSE_CHARS} chars]\n"));
            }
            s
        }
        Ok(o) => format!(
            "Error: gh pr view: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => format!("Error: spawn gh: {e}"),
    }
}

/// `glab mr list` (read-only).
pub fn glab_mr_list_work(cwd: PathBuf, limit: usize) -> String {
    if let Err(e) = terminal_run_precheck("glab") {
        return format!("Error: {e}");
    }
    let lim = limit.clamp(1, 50);
    let json_try = Command::new("glab")
        .current_dir(&cwd)
        .env("CI", "1")
        .args(["mr", "list", "--per-page", &lim.to_string(), "-F", "json"])
        .output();
    match json_try {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
            let mut s = format!("GLAB_MR_LIST dir={} per_page={lim}\n\n{body}", cwd.display());
            if clipped {
                s.push_str(&format!("\n[truncated]\n"));
            }
            s
        }
        _ => {
            let plain = Command::new("glab")
                .current_dir(&cwd)
                .env("CI", "1")
                .args(["mr", "list", "--per-page", &lim.to_string()])
                .output();
            match plain {
                Ok(p) if p.status.success() => {
                    let stdout = String::from_utf8_lossy(&p.stdout);
                    let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
                    let mut s = format!(
                        "GLAB_MR_LIST dir={} per_page={lim} (plain)\n\n{body}",
                        cwd.display()
                    );
                    if clipped {
                        s.push_str(&format!("\n[truncated]\n"));
                    }
                    s
                }
                Ok(p) => format!(
                    "Error: glab mr list: {}",
                    String::from_utf8_lossy(&p.stderr).trim()
                ),
                Err(e) => format!("Error: spawn glab: {e}"),
            }
        }
    }
}

/// `glab mr show <iid>` (read-only).
pub fn glab_mr_show_work(cwd: PathBuf, iid: &str) -> String {
    if let Err(e) = terminal_run_precheck("glab") {
        return format!("Error: {e}");
    }
    let id = iid.trim();
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) || id.len() > 12 {
        return "Error: iid must be non-empty ASCII digits only".to_string();
    }
    match Command::new("glab")
        .current_dir(&cwd)
        .env("CI", "1")
        .args(["mr", "show", id])
        .output()
    {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let (body, clipped) = truncate_chars(&stdout.trim(), MAX_RESPONSE_CHARS);
            let mut s = format!("GLAB_MR_SHOW dir={} iid={id}\n\n{body}", cwd.display());
            if clipped {
                s.push_str(&format!("\n[truncated]\n"));
            }
            s
        }
        Ok(o) => format!(
            "Error: glab mr show: {}",
            String::from_utf8_lossy(&o.stderr).trim()
        ),
        Err(e) => format!("Error: spawn glab: {e}"),
    }
}

pub fn file_info_work(path: PathBuf) -> String {
    let meta = match fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => return format!("Error: {e}"),
    };
    let kind = if meta.is_dir() {
        "directory"
    } else if meta.is_file() {
        "file"
    } else {
        "other"
    };
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| format!("{}s since UNIX epoch", d.as_secs()))
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "path: {}\nkind: {kind}\nsize_bytes: {}\nmodified: {modified}",
        path.display(),
        meta.len()
    )
}

fn clamp_usize(n: usize, min_v: usize, max_v: usize) -> usize {
    n.max(min_v).min(max_v)
}

fn collapse_whitespace_work(text: &str) -> String {
    // Collapse intra-line whitespace while preserving newlines between lines.
    // This is intentionally deterministic and language-agnostic.
    text.lines()
        .map(|line| {
            let t = line.trim();
            if t.is_empty() {
                String::new()
            } else {
                t.split_whitespace().collect::<Vec<_>>().join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn dedupe_consecutive_lines_work(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        if out.last().map(|p| p == line).unwrap_or(false) {
            continue;
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn head_tail_chars_work(text: &str, keep_each: usize) -> (String, String) {
    let head: String = text.chars().take(keep_each).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(keep_each)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    (head, tail)
}

/// Deterministically compress arbitrary context text into a bounded string.
///
/// Intended for reducing prompt/context size before sending to an LLM.
///
/// Modes: `truncate`, `head_tail` (default), `lines_only` (drop blank lines first),
/// `dense` (collapse whitespace + dedupe consecutive lines, then head/tail).
pub fn compress_context_work(
    text: String,
    max_chars: usize,
    mode: String,
    dedupe_lines: bool,
    collapse_whitespace: bool,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let mode_norm = mode.trim().to_ascii_lowercase();
    let mut s = text;

    match mode_norm.as_str() {
        "lines_only" => {
            s = s
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n");
        }
        "dense" => {
            s = collapse_whitespace_work(&s);
            s = dedupe_consecutive_lines_work(&s);
        }
        _ => {
            if collapse_whitespace {
                s = collapse_whitespace_work(&s);
            }
            if dedupe_lines {
                s = dedupe_consecutive_lines_work(&s);
            }
        }
    }

    let input_chars = s.chars().count();
    if input_chars <= max_chars {
        let out = format!("COMPRESS_CONTEXT mode={} input_chars={}\n\n{}", mode, input_chars, s);
        let (out, _) = truncate_chars(&out, max_chars);
        return out;
    }

    let body = match mode_norm.as_str() {
        "truncate" => {
            let (t, _) = truncate_chars(&s, max_chars);
            t
        }
        "head_tail" | "lines_only" | "dense" | _ => {
            let keep_each = (max_chars / 2).max(1);
            let (head, tail) = head_tail_chars_work(&s, keep_each);
            let omitted = input_chars.saturating_sub(head.chars().count() + tail.chars().count());
            let marker = format!("\n\n[...omitted {omitted} chars...]\n\n");
            let mut out = String::new();
            out.push_str(&head);
            out.push_str(&marker);
            out.push_str(&tail);
            let (out, _) = truncate_chars(&out, max_chars);
            out
        }
    };

    let out = format!(
        "COMPRESS_CONTEXT mode={} input_chars={} max_chars={}\n\n{}",
        mode_norm, input_chars, max_chars, body
    );
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Extract line windows around regex matches and return a bounded snippet view.
pub fn extract_snippets_work(
    text: String,
    pattern: String,
    literal: bool,
    case_sensitive: bool,
    context_lines: usize,
    max_snippets: usize,
    max_chars: usize,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let context_lines = clamp_usize(context_lines, 0, 30);
    let max_snippets = clamp_usize(max_snippets, 1, 50);

    let re = if literal {
        match RegexBuilder::new(&regex::escape(&pattern))
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Error: failed to build literal matcher: {e}"),
        }
    } else {
        match RegexBuilder::new(&pattern)
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Error: invalid regex pattern: {e}"),
        }
    };

    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        let out = "EXTRACT_SNIPPETS input had no lines".to_string();
        let (out, _) = truncate_chars(&out, max_chars);
        return out;
    }

    let mut hits: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if re.is_match(line) {
            hits.push(i);
            if hits.len() >= max_snippets {
                break;
            }
        }
    }

    if hits.is_empty() {
        return compress_context_work(text, max_chars, "head_tail".to_string(), false, false);
    }

    let mut ranges: Vec<(usize, usize)> = hits
        .into_iter()
        .map(|idx| {
            let start = idx.saturating_sub(context_lines);
            let end = (idx + context_lines).min(lines.len().saturating_sub(1));
            (start, end)
        })
        .collect();

    ranges.sort_by_key(|(s, _e)| *s);

    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in ranges {
        if let Some(last) = merged.last_mut() {
            if s <= last.1.saturating_add(1) {
                last.1 = last.1.max(e);
                continue;
            }
        }
        merged.push((s, e));
    }

    let mut out = String::new();
    out.push_str(&format!(
        "EXTRACT_SNIPPETS lines_in_input={} matches_kept={} context_lines={} windows_kept={}\n\n",
        lines.len(),
        merged.len().min(max_snippets),
        context_lines,
        merged.len()
    ));

    let mut prev_end: Option<usize> = None;
    for (s, e) in merged {
        if let Some(pe) = prev_end {
            if s > pe + 1 {
                out.push_str(&format!("\n[... skipped {} lines ...]\n\n", s - pe - 1));
            }
        }
        for i in s..=e {
            out.push_str(&format!("{:6}| {}\n", i + 1, lines[i]));
        }
        prev_end = Some(e);
    }

    let (out, _) = truncate_chars(&out, max_chars);
    out
}

fn read_file_text_raw_work(
    path: PathBuf,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<(String, bool, usize), String> {
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if !meta.is_file() {
        return Err("path is not a file".to_string());
    }

    let len = meta.len();
    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;

    let mut buf = Vec::new();
    if let Err(e) = file
        .take(MAX_FILE_READ_BYTES)
        .read_to_end(&mut buf)
    {
        return Err(e.to_string());
    }

    let text = String::from_utf8_lossy(&buf).into_owned();
    let truncated_on_disk = len > MAX_FILE_READ_BYTES;

    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let n_lines = lines.len();

    if start_line.is_some() || end_line.is_some() {
        let start = start_line.unwrap_or(1).max(1);
        let end = end_line.unwrap_or(n_lines).min(n_lines).max(1);
        if start > n_lines {
            return Err(format!(
                "No lines in range (file has {} lines in loaded chunk; start_line={})",
                n_lines, start
            ));
        }

        let slice = &lines[start - 1..end];
        let out = slice.join("");
        return Ok((out, truncated_on_disk, n_lines));
    }

    Ok((text, truncated_on_disk, n_lines))
}

/// Read a file (optionally a line range) and deterministically compress it.
pub fn compress_file_work(
    path: PathBuf,
    start_line: Option<usize>,
    end_line: Option<usize>,
    max_chars: usize,
    mode: String,
    dedupe_lines: bool,
    collapse_whitespace: bool,
) -> String {
    let (text, truncated_on_disk, _n_lines) =
        match read_file_text_raw_work(path.clone(), start_line, end_line) {
            Ok(v) => v,
            Err(e) => return format!("Error: {e}"),
        };

    let body = compress_context_work(
        text,
        max_chars,
        mode,
        dedupe_lines,
        collapse_whitespace,
    );

    let header = format!(
        "COMPRESS_FILE path={} truncated_on_disk={}\n",
        path.display(),
        truncated_on_disk
    );
    let out = format!("{header}\n{body}");
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Extract line snippets directly from a file, then return only the relevant windows.
pub fn extract_snippets_from_file_work(
    path: PathBuf,
    pattern: String,
    literal: bool,
    case_sensitive: bool,
    context_lines: usize,
    max_snippets: usize,
    max_chars: usize,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> String {
    let (text, truncated_on_disk, _n_lines) =
        match read_file_text_raw_work(path.clone(), start_line, end_line) {
            Ok(v) => v,
            Err(e) => return format!("Error: {e}"),
        };

    let body = extract_snippets_work(
        text,
        pattern,
        literal,
        case_sensitive,
        context_lines,
        max_snippets,
        max_chars,
    );

    let header = format!(
        "EXTRACT_SNIPPETS_FROM_FILE path={} truncated_on_disk={}\n",
        path.display(),
        truncated_on_disk
    );
    let out = format!("{header}\n{body}");
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Regex search across the codebase, returning bounded snippet windows near matches.
pub fn grep_snippets_work(
    search_root: PathBuf,
    pattern: String,
    literal: bool,
    case_sensitive: bool,
    respect_gitignore: bool,
    context_lines: usize,
    max_snippets: usize,
    max_chars: usize,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let context_lines = clamp_usize(context_lines, 0, 30);
    let max_snippets = clamp_usize(max_snippets, 1, 50);

    let re = if literal {
        match RegexBuilder::new(&regex::escape(&pattern))
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Error: failed to build literal matcher: {e}"),
        }
    } else {
        match RegexBuilder::new(&pattern)
            .case_insensitive(!case_sensitive)
            .build()
        {
            Ok(r) => r,
            Err(e) => return format!("Error: invalid regex pattern: {e}"),
        }
    };

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(respect_gitignore);
    walk.ignore(respect_gitignore);
    walk.hidden(false);

    let mut windows_kept = 0usize;
    let mut hits = 0usize;
    let mut out = String::new();

    for entry in walk.build().flatten() {
        if windows_kept >= max_snippets {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(text) = read_text_file_limited(path) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let mut match_idxs: Vec<usize> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                match_idxs.push(i);
                hits += 1;
            }
        }

        if match_idxs.is_empty() {
            continue;
        }

        // Convert match indices into context windows, then merge overlapping ranges.
        let mut ranges: Vec<(usize, usize)> = match_idxs
            .into_iter()
            .map(|idx| {
                let start = idx.saturating_sub(context_lines);
                let end = (idx + context_lines).min(lines.len().saturating_sub(1));
                (start, end)
            })
            .collect();
        ranges.sort_by_key(|(s, _e)| *s);

        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (s, e) in ranges {
            if let Some(last) = merged.last_mut() {
                if s <= last.1.saturating_add(1) {
                    last.1 = last.1.max(e);
                    continue;
                }
            }
            merged.push((s, e));
        }

        // Append windows in file order until we reach max_snippets.
        let rel = path
            .strip_prefix(&search_root)
            .map(|p| normalize_rel_path(p))
            .unwrap_or_else(|_| normalize_rel_path(&path));

        for (s, e) in merged {
            if windows_kept >= max_snippets {
                break;
            }

            out.push_str(&format!(
                "\nFILE: {}\nWINDOW: lines {}-{}\n",
                rel,
                s + 1,
                e + 1
            ));
            for i in s..=e {
                out.push_str(&format!("{:6}| {}\n", i + 1, lines[i]));
            }
            windows_kept += 1;
        }
    }

    let header = format!(
        "GREPPED_SNIPPETS\nSEARCH_ROOT: {}\nHITS: {}\nWINDOWS_KEPT: {}\nPATTERN: {}\n\n",
        search_root.display(),
        hits,
        windows_kept,
        pattern
    );
    out = format!("{header}{out}");
    let (out, clipped) = truncate_chars(out.trim_end(), max_chars);
    if clipped {
        out + &format!("\n\n[output truncated to {max_chars} characters]")
    } else {
        out
    }
}

fn english_stopwords() -> &'static [&'static str] {
    // Small, fixed list for deterministic scoring. This is not intended to be exhaustive.
    &[
        "the", "a", "an", "and", "or", "but", "if", "then", "else", "when", "while", "of", "to",
        "in", "on", "for", "with", "as", "at", "by", "from", "into", "during", "including", "until",
        "through", "before", "after", "above", "below", "up", "down", "out", "over", "under", "again",
        "further", "once", "here", "there", "all", "any", "both", "each", "few", "more", "most",
        "other", "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
        "very", "can", "will", "just", "don", "should", "now", "is", "are", "was", "were", "be", "been",
        "being", "it", "its", "this", "that", "these", "those", "i", "you", "he", "she", "they", "we",
        "them", "his", "her", "your", "my",
    ]
}

fn tokenize_words_lower(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch.to_ascii_lowercase());
        } else {
            if !cur.is_empty() {
                out.push(cur.clone());
                cur.clear();
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn split_sentences(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        cur.push(ch);
        if matches!(ch, '.' | '!' | '?' | '\n') {
            let t = cur.trim();
            if !t.is_empty() {
                out.push(t.to_string());
            }
            cur.clear();
        }
    }
    let t = cur.trim();
    if !t.is_empty() {
        out.push(t.to_string());
    }
    out
}

/// Join segments and summarize as one document (parallel sources → one extractive summary).
pub fn summarize_segments_work(segments: Vec<String>, max_chars: usize, max_sentences: usize) -> String {
    if segments.is_empty() {
        return "SUMMARIZE_SEGMENTS: no segments\n".to_string();
    }
    let n = segments.len();
    let joined = segments.join("\n\n--- segment ---\n\n");
    let body = summarize_text_work(joined, max_chars, max_sentences);
    format!("SUMMARIZE_SEGMENTS merged_segments={n}\n\n{body}")
}

/// Keep the first N sentences (split after `.` `?` `!` + whitespace); complements `compress_paragraphs`.
pub fn compress_sentences_work(text: String, max_sentences: usize, max_chars: usize) -> String {
    let max_sentences = max_sentences.clamp(1, 200);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let re = match Regex::new(r"(?<=[.!?])\s+") {
        Ok(r) => r,
        Err(e) => return format!("Error: regex: {e}"),
    };
    let mut sentences: Vec<String> = re
        .split(text.trim())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .take(max_sentences)
        .map(|s| s.to_string())
        .collect();
    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim().to_string());
    }
    let n = sentences.len();
    let body = sentences.join(" ");
    let (body, _) = truncate_chars(&body, max_chars);
    format!(
        "COMPRESS_SENTENCES max_sentences_cap={max_sentences} sentences_kept={n} max_chars={max_chars}\n\n{body}"
    )
}

/// Deterministically summarize text via heuristic sentence scoring (term-frequency overlap).
pub fn summarize_text_work(
    text: String,
    max_chars: usize,
    max_sentences: usize,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let max_sentences = clamp_usize(max_sentences, 1, 30);

    let cleaned = text.trim();
    if cleaned.is_empty() {
        return "SUMMARY_TEXT: empty input\n".to_string();
    }

    let sentences = split_sentences(cleaned);
    if sentences.is_empty() {
        return "SUMMARY_TEXT: no sentences detected\n".to_string();
    }

    // Score each sentence by sum of term frequencies for non-stopwords.
    let words = tokenize_words_lower(cleaned);
    let mut tf: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for w in words {
        if w.len() < 3 {
            continue;
        }
        if english_stopwords().contains(&w.as_str()) {
            continue;
        }
        *tf.entry(w).or_insert(0) += 1;
    }

    let mut scored: Vec<(usize, f64)> = Vec::new(); // (index, score)
    for (i, sent) in sentences.iter().enumerate() {
        let ws = tokenize_words_lower(sent);
        let mut score: f64 = 0.0;
        for w in ws {
            if let Some(c) = tf.get(&w) {
                score += *c as f64;
            }
        }
        scored.push((i, score));
    }

    // Pick top-N by score, but keep original ordering for readability.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let chosen_idx: Vec<usize> = scored.iter().take(max_sentences).map(|(i, _)| *i).collect();
    let chosen_set: std::collections::HashSet<usize> = chosen_idx.into_iter().collect();

    let mut summary_parts: Vec<String> = Vec::new();
    for (i, sent) in sentences.into_iter().enumerate() {
        if chosen_set.contains(&i) {
            summary_parts.push(sent);
        }
    }

    let joined = summary_parts.join(" ");
    let out = format!(
        "SUMMARY_TEXT max_sentences={} chars_in_output_limit={} \n\n{}",
        max_sentences, max_chars, joined
    );
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Pack multiple context segments into one bounded string, preserving segment boundaries.
/// Optional `segment_labels` (same length as `segments`) appear in each segment header for handoffs.
pub fn pack_context_work(
    segments: Vec<String>,
    segment_labels: Option<Vec<String>>,
    max_chars: usize,
    mode: String,
    dedupe_lines: bool,
    collapse_whitespace: bool,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    if segments.is_empty() {
        return "PACK_CONTEXT: no segments provided\n".to_string();
    }

    let n = segments.len();
    let labels_ok = segment_labels
        .as_ref()
        .map(|v| v.len() == n)
        .unwrap_or(false);
    let mut joined = String::new();
    joined.push_str(&format!("PACK_CONTEXT segments={} \n", n));
    for (i, seg) in segments.into_iter().enumerate() {
        let header = if labels_ok {
            let lab = segment_labels
                .as_ref()
                .and_then(|v| v.get(i))
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .unwrap_or("—");
            format!("\n--- SEGMENT {} / {} :: {} ---\n", i + 1, n, lab)
        } else {
            format!("\n--- SEGMENT {} / {} ---\n", i + 1, n)
        };
        joined.push_str(&header);
        joined.push_str(&seg);
    }

    compress_context_work(joined, max_chars, mode, dedupe_lines, collapse_whitespace)
}

/// Extract relevant windows from `git diff` output by regex matches (bounded).
pub fn git_diff_snippets_work(
    dir: PathBuf,
    staged: bool,
    pattern: String,
    literal: bool,
    case_sensitive: bool,
    context_lines: usize,
    max_snippets: usize,
    max_chars: usize,
) -> String {
    let diff = git_diff_work(dir, staged);
    if diff.starts_with("Error:") {
        return diff;
    }
    let body = extract_snippets_work(
        diff,
        pattern,
        literal,
        case_sensitive,
        context_lines,
        max_snippets,
        max_chars,
    );
    let header = format!("GIT_DIFF_SNIPPETS staged={}\n\n", staged);
    let out = format!("{}{}", header, body);
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Read file as UTF-8 text and run the same summarizer as `summarize_text` (bounded).
pub fn summarize_file_work(
    path: PathBuf,
    start_line: Option<usize>,
    end_line: Option<usize>,
    max_chars: usize,
    max_sentences: usize,
) -> String {
    let (text, truncated_on_disk, _n_lines) =
        match read_file_text_raw_work(path.clone(), start_line, end_line) {
            Ok(v) => v,
            Err(e) => return format!("Error: {e}"),
        };

    let body = summarize_text_work(text, max_chars, max_sentences);
    format!(
        "SUMMARIZE_FILE path={} truncated_on_disk={}\n\n{}",
        path.display(),
        truncated_on_disk,
        body
    )
}

/// Read up to `lines_per_file` lines from each path (max 8 files), then one merged extractive summary.
pub fn summarize_multi_file_work(
    paths: Vec<PathBuf>,
    lines_per_file: usize,
    max_chars: usize,
    max_sentences: usize,
) -> String {
    let lines_per_file = lines_per_file.clamp(1, 400);
    let mut segments: Vec<String> = Vec::new();
    for p in paths.into_iter().take(8) {
        let label = normalize_rel_path(&p);
        match read_file_text_raw_work(p.clone(), Some(1), Some(lines_per_file)) {
            Ok((text, trunc, n_lines)) => {
                let end = lines_per_file.min(n_lines.max(1));
                segments.push(format!(
                    "FILE: {label} lines_1..{end} truncated_on_disk={trunc}\n{text}"
                ));
            }
            Err(e) => segments.push(format!("FILE: {label}\nError: {e}")),
        }
    }
    if segments.is_empty() {
        return "SUMMARIZE_MULTI_FILE: no paths\n".to_string();
    }
    let head = format!("SUMMARIZE_MULTI_FILE files_in_batch={}\n\n", segments.len());
    let body = summarize_segments_work(segments, max_chars, max_sentences);
    format!("{head}{body}")
}

fn read_file_bytes_capped(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.take(max as u64)
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

fn hex_encode_bytes(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode_str(s: &str) -> Result<Vec<u8>, String> {
    let hex: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() % 2 != 0 {
        return Err("hex decode: odd number of hex digits".into());
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let pair = std::str::from_utf8(chunk).map_err(|_| "hex decode: invalid utf8 chunk".to_string())?;
        let b = u8::from_str_radix(pair, 16).map_err(|_| format!("hex decode: invalid pair {pair}"))?;
        out.push(b);
    }
    Ok(out)
}

fn convert_format_load_raw(
    text_input: Option<String>,
    input_path: Option<&PathBuf>,
    op: &str,
) -> Result<String, String> {
    if let Some(t) = text_input.filter(|s| !s.is_empty()) {
        Ok(t)
    } else if let Some(p) = input_path {
        let b = read_file_bytes_capped(p, MAX_CONVERT_BYTES).map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&b).into_owned())
    } else {
        Err(format!("Error: {op} needs `text` or `input_path`"))
    }
}

/// JSON pretty/minify, base64/hex encode or decode. Paths must be validated by the caller (project root).
/// When `allow_file_output` is false, `output_path` must be absent (public / internet profile).
/// `tab_stops` applies to `tabs_to_spaces` (1–16). `line_number_start` applies to `prefix_line_numbers` (1-based).
pub fn convert_format_work(
    operation: String,
    text_input: Option<String>,
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    allow_file_output: bool,
    tab_stops: usize,
    line_number_start: usize,
    wrap_width: usize,
) -> String {
    if !allow_file_output && output_path.is_some() {
        return "Error: convert_format file output is disabled when MINT_NETWORK_PROFILE=public (omit output_path).".to_string();
    }
    let op = operation.trim().to_ascii_lowercase();
    match op.as_str() {
        "json_pretty" | "json_minify" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: json_pretty/json_minify need `text` or `input_path`".to_string();
            };
            let v: Value = match serde_json::from_str(raw.trim()) {
                Ok(v) => v,
                Err(e) => return format!("Error: invalid JSON: {e}"),
            };
            let out = match if op == "json_pretty" {
                serde_json::to_string_pretty(&v)
            } else {
                serde_json::to_string(&v)
            } {
                Ok(s) => s,
                Err(e) => return format!("Error: JSON serialize: {e}"),
            };
            let (out, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
            let mut s = format!("CONVERT_FORMAT operation={op} output_chars={}\n\n", out.len());
            s.push_str(&out);
            if clipped {
                s.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
            }
            s
        }
        "base64_encode" => {
            let bytes = if let Some(t) = text_input {
                t.into_bytes()
            } else if let Some(ref p) = input_path {
                match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                }
            } else {
                return "Error: base64_encode needs `text` or `input_path`".to_string();
            };
            let enc = STANDARD.encode(&bytes);
            format!(
                "CONVERT_FORMAT operation=base64_encode input_bytes={}\n\n{enc}\n",
                bytes.len()
            )
        }
        "base64_decode" => {
            let Some(t) = text_input.filter(|s| !s.is_empty()) else {
                return "Error: base64_decode needs `text` (base64 payload)".to_string();
            };
            let bytes = match STANDARD.decode(t.trim()) {
                Ok(b) => b,
                Err(e) => return format!("Error: base64 decode: {e}"),
            };
            if let Some(out_p) = output_path {
                if let Err(e) = fs::write(&out_p, &bytes) {
                    return format!("Error: write {}: {e}", out_p.display());
                }
                format!(
                    "CONVERT_FORMAT operation=base64_decode wrote_bytes={} path={}\n",
                    bytes.len(),
                    out_p.display()
                )
            } else {
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => {
                        let (s, clipped) = truncate_chars(&s, MAX_RESPONSE_CHARS);
                        let mut o = format!(
                            "CONVERT_FORMAT operation=base64_decode utf8 output_chars={}\n\n{s}\n",
                            s.len()
                        );
                        if clipped {
                            o.push_str(&format!("\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                        }
                        o
                    }
                    Err(_) => format!(
                        "CONVERT_FORMAT operation=base64_decode binary_bytes={} (not valid UTF-8; set output_path to save)\n",
                        bytes.len()
                    ),
                }
            }
        }
        "hex_encode" => {
            let bytes = if let Some(t) = text_input {
                t.into_bytes()
            } else if let Some(ref p) = input_path {
                match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                }
            } else {
                return "Error: hex_encode needs `text` or `input_path`".to_string();
            };
            let enc = hex_encode_bytes(&bytes);
            format!(
                "CONVERT_FORMAT operation=hex_encode input_bytes={}\n\n{enc}\n",
                bytes.len()
            )
        }
        "hex_decode" => {
            let Some(t) = text_input.filter(|s| !s.is_empty()) else {
                return "Error: hex_decode needs `text` (hex string)".to_string();
            };
            let bytes = match hex_decode_str(&t) {
                Ok(b) => b,
                Err(e) => return format!("Error: {e}"),
            };
            if let Some(out_p) = output_path {
                if let Err(e) = fs::write(&out_p, &bytes) {
                    return format!("Error: write {}: {e}", out_p.display());
                }
                format!(
                    "CONVERT_FORMAT operation=hex_decode wrote_bytes={} path={}\n",
                    bytes.len(),
                    out_p.display()
                )
            } else {
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => {
                        let (s, clipped) = truncate_chars(&s, MAX_RESPONSE_CHARS);
                        let mut o = format!(
                            "CONVERT_FORMAT operation=hex_decode utf8 output_chars={}\n\n{s}\n",
                            s.len()
                        );
                        if clipped {
                            o.push_str(&format!("\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                        }
                        o
                    }
                    Err(_) => format!(
                        "CONVERT_FORMAT operation=hex_decode binary_bytes={} (not UTF-8; set output_path)\n",
                        bytes.len()
                    ),
                }
            }
        }
        "utf8_bom_strip" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: utf8_bom_strip needs `text` or `input_path`".to_string();
            };
            let stripped = raw.strip_prefix('\u{feff}').unwrap_or(raw.as_str());
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, stripped.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=utf8_bom_strip wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(stripped, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=utf8_bom_strip\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "crlf_to_lf" | "normalize_lf" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: crlf_to_lf needs `text` or `input_path`".to_string();
            };
            let out = crlf_to_lf_text(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=crlf_to_lf wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=crlf_to_lf\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "lf_to_crlf" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: lf_to_crlf needs `text` or `input_path`".to_string();
            };
            let out = lf_to_crlf_text(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=lf_to_crlf wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=lf_to_crlf\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "tabs_to_spaces" => {
            let width = tab_stops.clamp(1, 16);
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: tabs_to_spaces needs `text` or `input_path`".to_string();
            };
            let out = tabs_to_spaces_lines(&raw, width);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!(
                    "CONVERT_FORMAT operation=tabs_to_spaces tab_stops={width} wrote path={disp}\n"
                )
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o =
                    format!("CONVERT_FORMAT operation=tabs_to_spaces tab_stops={width}\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "spaces_to_tabs" | "leading_spaces_to_tabs" => {
            let width = tab_stops.clamp(1, 16);
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: spaces_to_tabs needs `text` or `input_path`".to_string();
            };
            let out = leading_spaces_to_tabs_str(&raw, width);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!(
                    "CONVERT_FORMAT operation=spaces_to_tabs tab_stops={width} wrote path={disp}\n"
                )
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o =
                    format!("CONVERT_FORMAT operation=spaces_to_tabs tab_stops={width}\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "strip_ansi" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: strip_ansi needs `text` or `input_path`".to_string();
            };
            let out = strip_ansi_str(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=strip_ansi wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=strip_ansi\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "collapse_blank_lines" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: collapse_blank_lines needs `text` or `input_path`".to_string();
            };
            let out = collapse_blank_lines_str(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=collapse_blank_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=collapse_blank_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "remove_empty_lines" => {
            let raw = match convert_format_load_raw(
                text_input,
                input_path.as_ref(),
                "remove_empty_lines",
            ) {
                Ok(s) => s,
                Err(e) => return e,
            };
            let out = remove_empty_lines_str(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=remove_empty_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=remove_empty_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "slugify" | "slugify_lines" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "slugify") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let out = slugify_lines_str(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=slugify wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=slugify\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "c_string_escape" | "escape_c_string" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: c_string_escape needs `text` or `input_path`".to_string();
            };
            let out = escape_c_string_literal_body(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=c_string_escape wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=c_string_escape\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "text_lowercase" | "lowercase" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: text_lowercase needs `text` or `input_path`".to_string();
            };
            let out = raw.to_lowercase();
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=text_lowercase wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=text_lowercase\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "text_uppercase" | "uppercase" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: text_uppercase needs `text` or `input_path`".to_string();
            };
            let out = raw.to_uppercase();
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=text_uppercase wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=text_uppercase\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "prefix_line_numbers" => {
            let raw = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
                t
            } else if let Some(ref p) = input_path {
                let b = match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
                    Ok(b) => b,
                    Err(e) => return format!("Error: {e}"),
                };
                String::from_utf8_lossy(&b).into_owned()
            } else {
                return "Error: prefix_line_numbers needs `text` or `input_path`".to_string();
            };
            let start = line_number_start.max(1);
            let out = prefix_line_numbers_str(&raw, start);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!(
                    "CONVERT_FORMAT operation=prefix_line_numbers start={start} wrote path={disp}\n"
                )
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!(
                    "CONVERT_FORMAT operation=prefix_line_numbers start={start}\n\n{s}"
                );
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "sort_lines" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "sort_lines") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let mut lines: Vec<&str> = raw.lines().collect();
            lines.sort();
            let out = lines.join("\n");
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=sort_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=sort_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "reverse_lines" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "reverse_lines") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let mut lines: Vec<&str> = raw.lines().collect();
            lines.reverse();
            let out = lines.join("\n");
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=reverse_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=reverse_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "trim_each_line" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "trim_each_line") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let out: String = raw.lines().map(str::trim).collect::<Vec<_>>().join("\n");
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=trim_each_line wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=trim_each_line\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "unique_sorted_lines" => {
            let raw = match convert_format_load_raw(
                text_input,
                input_path.as_ref(),
                "unique_sorted_lines",
            ) {
                Ok(s) => s,
                Err(e) => return e,
            };
            let mut lines: Vec<String> = raw.lines().map(|l| l.to_string()).collect();
            lines.sort();
            lines.dedup();
            let out = lines.join("\n");
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=unique_sorted_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=unique_sorted_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "wrap_lines" | "hard_wrap_lines" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "wrap_lines") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let w = wrap_width.clamp(8, 200);
            let out = wrap_lines_str(&raw, w);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=wrap_lines width={w} wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=wrap_lines width={w}\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "dedupe_adjacent_lines" | "dedupe_consecutive_lines" => {
            let raw = match convert_format_load_raw(
                text_input,
                input_path.as_ref(),
                "dedupe_adjacent_lines",
            ) {
                Ok(s) => s,
                Err(e) => return e,
            };
            let mut out: Vec<String> = Vec::new();
            let mut last: Option<&str> = None;
            for line in raw.lines() {
                if last == Some(line) {
                    continue;
                }
                last = Some(line);
                out.push(line.to_string());
            }
            let merged = out.join("\n");
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, merged.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=dedupe_adjacent_lines wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&merged, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=dedupe_adjacent_lines\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        "percent_encode" | "url_encode" => {
            let Some(t) = text_input.filter(|s| !s.is_empty()) else {
                return "Error: percent_encode needs `text`".to_string();
            };
            let enc = percent_encode_component(&t);
            format!("CONVERT_FORMAT operation=percent_encode\n\n{enc}\n")
        }
        "percent_decode" | "url_decode" => {
            let Some(t) = text_input.filter(|s| !s.is_empty()) else {
                return "Error: percent_decode needs `text`".to_string();
            };
            match percent_decode_component(&t) {
                Ok(s) => {
                    let (s, clipped) = truncate_chars(&s, MAX_RESPONSE_CHARS);
                    let mut o = format!("CONVERT_FORMAT operation=percent_decode\n\n{s}");
                    if clipped {
                        o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                    }
                    o
                }
                Err(e) => format!("Error: {e}"),
            }
        }
        "html_escape" | "escape_html" => {
            let raw = match convert_format_load_raw(text_input, input_path.as_ref(), "html_escape") {
                Ok(s) => s,
                Err(e) => return e,
            };
            let out = html_escape_str(&raw);
            if let Some(out_p) = output_path {
                let disp = out_p.display().to_string();
                if let Err(e) = fs::write(&out_p, out.as_bytes()) {
                    return format!("Error: write {disp}: {e}");
                }
                format!("CONVERT_FORMAT operation=html_escape wrote path={disp}\n")
            } else {
                let (s, clipped) = truncate_chars(&out, MAX_RESPONSE_CHARS);
                let mut o = format!("CONVERT_FORMAT operation=html_escape\n\n{s}");
                if clipped {
                    o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
                }
                o
            }
        }
        _ => format!(
            "Error: unknown operation {:?}; use json_pretty | json_minify | base64_encode | base64_decode | hex_encode | hex_decode | utf8_bom_strip | crlf_to_lf | lf_to_crlf | tabs_to_spaces | spaces_to_tabs | strip_ansi | collapse_blank_lines | remove_empty_lines | slugify | c_string_escape | prefix_line_numbers | text_lowercase | text_uppercase | sort_lines | reverse_lines | trim_each_line | unique_sorted_lines | dedupe_adjacent_lines | wrap_lines | percent_encode | percent_decode | html_escape",
            operation
        ),
    }
}

fn html_escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len().saturating_add(s.len() / 8));
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Convert runs of leading ASCII spaces into tabs (tab_stops spaces → one tab); rest of line unchanged.
fn leading_spaces_to_tabs_str(s: &str, tab_stops: usize) -> String {
    let w = tab_stops.max(1);
    s.lines()
        .map(|line| {
            let nsp = line.chars().take_while(|c| *c == ' ').count();
            let rest: String = line.chars().skip(nsp).collect();
            let tabs = nsp / w;
            let rem = nsp % w;
            format!("{}{}{}", "\t".repeat(tabs), " ".repeat(rem), rest)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap_lines_str(s: &str, width: usize) -> String {
    let w = width.max(8);
    let mut out: Vec<String> = Vec::new();
    for line in s.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut remainder = line.to_string();
        while !remainder.is_empty() {
            let trimmed = remainder.trim_start();
            if trimmed.is_empty() {
                break;
            }
            let chars: Vec<char> = trimmed.chars().collect();
            if chars.len() <= w {
                out.push(chars.iter().collect());
                break;
            }
            let mut break_at = w;
            for i in (1..w).rev() {
                if chars[i].is_whitespace() {
                    break_at = i;
                    break;
                }
            }
            if break_at == 0 {
                break_at = w;
            }
            let piece: String = chars[..break_at].iter().collect();
            out.push(piece.trim_end().to_string());
            remainder = chars[break_at..].iter().collect();
        }
    }
    out.join("\n")
}

fn crlf_to_lf_text(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn lf_to_crlf_text(s: &str) -> String {
    crlf_to_lf_text(s).replace('\n', "\r\n")
}

/// Remove common ANSI CSI / OSC escape sequences (terminal color, cursor).
fn collapse_blank_lines_str(s: &str) -> String {
    let mut v: Vec<String> = Vec::new();
    let mut prev_blank = false;
    for line in s.lines() {
        let blank = line.trim().is_empty();
        if blank {
            if !prev_blank {
                v.push(String::new());
            }
            prev_blank = true;
        } else {
            v.push(line.to_string());
            prev_blank = false;
        }
    }
    v.join("\n")
}

fn remove_empty_lines_str(s: &str) -> String {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn slugify_line_token(line: &str) -> String {
    let mut out = String::new();
    let mut prev_us = false;
    for c in line.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_us = false;
        } else if c.is_whitespace() || c == '_' || c == '-' || c == '.' || c == '/' {
            if !prev_us && !out.is_empty() {
                out.push('_');
                prev_us = true;
            }
        }
    }
    out.trim_matches('_').to_string()
}

fn slugify_lines_str(s: &str) -> String {
    s.lines().map(slugify_line_token).collect::<Vec<_>>().join("\n")
}

fn prefix_line_numbers_str(s: &str, start: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let last_no = start.saturating_add(lines.len().saturating_sub(1));
    let w = last_no.to_string().len().max(4);
    let mut out = String::with_capacity(s.len() + lines.len() * 8);
    for (i, line) in lines.iter().enumerate() {
        let n = start + i;
        out.push_str(&format!("{0:>1$}| {2}\n", n, w, line));
    }
    out
}

fn strip_ansi_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            match it.peek() {
                Some('[') => {
                    it.next();
                    while let Some(x) = it.next() {
                        if ('\x40'..='\x7e').contains(&x) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    it.next();
                    while let Some(x) = it.next() {
                        if x == '\u{7}' {
                            break;
                        }
                        if x == '\u{1b}' && it.peek() == Some(&'\\') {
                            it.next();
                            break;
                        }
                    }
                }
                Some('(') | Some(')') => {
                    it.next();
                    let _ = it.next();
                }
                _ => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn tabs_to_spaces_lines(text: &str, width: usize) -> String {
    let width = width.max(1);
    text.lines()
        .map(|line| {
            let mut col = 0usize;
            let mut out = String::with_capacity(line.len().saturating_add(8));
            for ch in line.chars() {
                if ch == '\t' {
                    let next_stop = ((col / width) + 1) * width;
                    for _ in col..next_stop {
                        out.push(' ');
                    }
                    col = next_stop;
                } else {
                    out.push(ch);
                    col += 1;
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn percent_encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn percent_decode_component(s: &str) -> Result<String, String> {
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char)
                .to_digit(16)
                .ok_or_else(|| "percent_decode: invalid hex".to_string())?;
            let lo = (bytes[i + 2] as char)
                .to_digit(16)
                .ok_or_else(|| "percent_decode: invalid hex".to_string())?;
            out.push(((hi << 4) | lo) as u8);
            i += 3;
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|e| format!("percent_decode: invalid UTF-8: {e}"))
}

fn sketch_lang_from_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "rs" | "sapp" => Some("rs"),
        "py" => Some("py"),
        "go" => Some("go"),
        "js" | "mjs" | "cjs" => Some("js"),
        "ts" | "tsx" => Some("ts"),
        "jsx" => Some("jsx"),
        "java" | "cs" => Some("jvm"),
        _ => None,
    }
}

fn symbol_sketch_line(line: &str, lang: &str) -> Option<String> {
    let t = line.trim_start();
    if t.is_empty() {
        return None;
    }
    if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with('#') {
        return None;
    }
    let cap = || line.trim().chars().take(200).collect::<String>();
    match lang {
        "rs" => {
            if t.starts_with("pub fn ")
                || t.starts_with("pub async fn ")
                || t.starts_with("fn ")
                || t.starts_with("pub struct ")
                || t.starts_with("struct ")
                || t.starts_with("pub enum ")
                || t.starts_with("enum ")
                || t.starts_with("impl ")
                || t.starts_with("pub trait ")
                || t.starts_with("trait ")
                || t.starts_with("pub type ")
                || t.starts_with("type ")
                || t.starts_with("pub mod ")
                || t.starts_with("mod ")
            {
                Some(cap())
            } else {
                None
            }
        }
        "py" => {
            if t.starts_with("def ") || t.starts_with("async def ") || t.starts_with("class ") {
                Some(cap())
            } else {
                None
            }
        }
        "go" => {
            if t.starts_with("func ") || t.starts_with("type ") {
                Some(cap())
            } else {
                None
            }
        }
        "js" | "jsx" => {
            if t.starts_with("function ")
                || t.starts_with("export function")
                || t.starts_with("export async function")
                || t.starts_with("async function")
                || t.starts_with("class ")
                || t.starts_with("export class")
                || t.starts_with("export default function")
            {
                Some(cap())
            } else {
                None
            }
        }
        "ts" | "tsx" => {
            if t.starts_with("function ")
                || t.starts_with("export function")
                || t.starts_with("export async function")
                || t.starts_with("async function")
                || t.starts_with("class ")
                || t.starts_with("export class")
                || t.starts_with("export interface")
                || t.starts_with("interface ")
                || t.starts_with("type ")
                || t.starts_with("export type")
            {
                Some(cap())
            } else {
                None
            }
        }
        "jvm" => {
            if t.contains(" class ") || t.contains(" interface ") || t.contains(" enum ") {
                Some(cap())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Bounded walk: extension histogram + optional lightweight definition-line sketch (code indexing aid).
pub fn codebase_index_work(
    search_root: PathBuf,
    max_files_scan: usize,
    symbol_sketch: bool,
    max_symbol_entries: usize,
) -> String {
    let max_files_scan = max_files_scan.clamp(200, 20_000);
    let max_symbol_entries = max_symbol_entries.clamp(0, 300);

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut files_seen: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut ext_hist: HashMap<String, (usize, u64)> = HashMap::new();
    let mut symbols: Vec<String> = Vec::new();
    let mut symbol_hits = 0usize;
    let mut walk_truncated = false;

    for entry in walk.build().flatten() {
        if files_seen >= max_files_scan {
            walk_truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        files_seen += 1;
        let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        total_bytes += len;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let key = if ext.is_empty() {
            "(no ext)".to_string()
        } else {
            ext
        };
        let e = ext_hist.entry(key).or_insert((0, 0));
        e.0 += 1;
        e.1 += len;

        if symbol_sketch && symbol_hits < max_symbol_entries {
            if let Some(lang) = sketch_lang_from_path(path) {
                let Some(text) = read_text_file_limited(path) else {
                    continue;
                };
                for (i, line) in text.lines().enumerate() {
                    if symbol_hits >= max_symbol_entries {
                        break;
                    }
                    if i >= 500 {
                        break;
                    }
                    if let Some(sym) = symbol_sketch_line(line, lang) {
                        let rel = path
                            .strip_prefix(&search_root)
                            .map(|p| normalize_rel_path(p))
                            .unwrap_or_else(|_| normalize_rel_path(path));
                        symbols.push(format!("{rel}:{}:{sym}", i + 1));
                        symbol_hits += 1;
                    }
                }
            }
        }
    }

    let mut ext_rows: Vec<(String, usize, u64)> = ext_hist
        .into_iter()
        .map(|(k, (c, b))| (k, c, b))
        .collect();
    ext_rows.sort_by(|a, b| b.1.cmp(&a.1));

    let mut out = String::new();
    out.push_str("CODEBASE_INDEX\n");
    out.push_str(&format!(
        "search_root: {}\nfiles_scanned: {}\ntotal_bytes: {}\nwalk_truncated: {walk_truncated}\n",
        search_root.display(),
        files_seen,
        total_bytes
    ));
    out.push_str("\n--- extensions (count, bytes) ---\n");
    for (ext, c, b) in ext_rows.into_iter().take(40) {
        out.push_str(&format!("  {ext}: files={c} bytes={b}\n"));
    }
    if symbol_sketch && !symbols.is_empty() {
        out.push_str("\n--- symbol sketch (heuristic; use grep_code for full search) ---\n");
        for s in symbols {
            out.push_str(&format!("{s}\n"));
        }
    } else if symbol_sketch {
        out.push_str("\n--- symbol sketch: (no matching definition-like lines in scanned files) ---\n");
    }
    out.push_str("\nTip: combine with project_tree, find_files, grep_code for precise navigation.\n");
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// True when `MINT_TERMINAL_ENABLE` allows [`terminal_run_precheck`].
pub fn terminal_enabled() -> bool {
    std::env::var("MINT_TERMINAL_ENABLE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn terminal_allowlist_programs() -> Vec<String> {
    let default = "git,cargo,rustc,rustfmt,npm,pnpm,yarn,node,deno,go,python,python3,pip,pytest,make,gh,glab,tesseract,ollama,ffmpeg,grim,screencapture";
    std::env::var("MINT_TERMINAL_ALLOW")
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Validates executable name for `terminal_run` (no shell; pass argv explicitly).
pub fn terminal_run_precheck(program: &str) -> Result<(), String> {
    if !terminal_enabled() {
        return Err(
            "terminal_run disabled. Set MINT_TERMINAL_ENABLE=1 and MINT_TERMINAL_ALLOW (comma-separated executables)."
                .into(),
        );
    }
    let p = program.trim();
    if p.is_empty() {
        return Err("program must not be empty".into());
    }
    if p.contains('/') || p.contains('\\') || p.contains("..") {
        return Err("program must be a bare executable name (e.g. git), not a path".into());
    }
    let base = Path::new(p)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(p)
        .to_ascii_lowercase();
    let allow = terminal_allowlist_programs();
    let pl = p.to_ascii_lowercase();
    if !allow.iter().any(|a| a == &base || a == &pl) {
        return Err(format!(
            "program {:?} not allowed. Allowed: {} (set MINT_TERMINAL_ALLOW).",
            p,
            allow.join(", ")
        ));
    }
    Ok(())
}

/// Resolve `program` on PATH (`which` / `where`). Same allowlist as `terminal_run`.
pub fn terminal_which_work(program: &str) -> String {
    if let Err(e) = terminal_run_precheck(program) {
        return format!("Error: {e}");
    }
    let p = program.trim();
    #[cfg(windows)]
    let out = Command::new("where").arg(p).output();
    #[cfg(not(windows))]
    let out = Command::new("which").arg(p).output();

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            format!(
                "TERMINAL_WHICH program={p}\nexit: {}\nPATH_RESOLUTION:\n{stdout}\nSTDERR:\n{stderr}\n",
                o.status
            )
        }
        Err(e) => format!("Error: spawn which/where: {e}"),
    }
}

/// Opt-in gate for `project_cargo_test`, `project_cargo_check`, `project_pytest` (in addition to `MINT_TERMINAL_ENABLE` + allowlist).
pub fn project_test_tools_enabled() -> bool {
    std::env::var("MINT_PROJECT_TEST_ENABLE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn sanitize_cargo_package_token(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("package name must not be empty".into());
    }
    if t.len() > 128 {
        return Err("package name too long (max 128)".into());
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("package name: use only ASCII letters, digits, _ and -".into());
    }
    Ok(t.to_string())
}

fn sanitize_cargo_test_filter(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.len() > 256 {
        return Err("test filter too long (max 256)".into());
    }
    if !t.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-' | '*')
    }) {
        return Err("test filter: use only alnum, _, :, ., -, *".into());
    }
    Ok(t.to_string())
}

/// argv after `cargo` for a bounded `cargo test` (no shell).
pub fn project_cargo_test_build_args(
    package: Option<&str>,
    test_filter: Option<&str>,
    no_run: bool,
    workspace: bool,
    jobs: Option<u32>,
    nocapture: bool,
) -> Result<Vec<String>, String> {
    let mut v = vec![
        "test".to_string(),
        "--message-format".to_string(),
        "short".to_string(),
    ];
    if workspace {
        v.push("--workspace".to_string());
    }
    if no_run {
        v.push("--no-run".to_string());
    }
    if let Some(j) = jobs {
        let j = j.clamp(1, 32);
        v.push("--jobs".to_string());
        v.push(j.to_string());
    }
    if let Some(p) = package {
        let p = sanitize_cargo_package_token(p)?;
        v.push("-p".to_string());
        v.push(p);
    }
    if let Some(f) = test_filter {
        let f = sanitize_cargo_test_filter(f)?;
        v.push(f);
    }
    if nocapture && !no_run {
        v.push("--".to_string());
        v.push("--nocapture".to_string());
    }
    Ok(v)
}

/// argv after `cargo` for `cargo check`.
pub fn project_cargo_check_build_args(
    package: Option<&str>,
    workspace: bool,
) -> Result<Vec<String>, String> {
    let mut v = vec![
        "check".to_string(),
        "--message-format".to_string(),
        "short".to_string(),
    ];
    if workspace {
        v.push("--workspace".to_string());
    }
    if let Some(p) = package {
        let p = sanitize_cargo_package_token(p)?;
        v.push("-p".to_string());
        v.push(p);
    }
    Ok(v)
}

fn sanitize_pytest_rel_path(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(".".to_string());
    }
    if t.contains("..") {
        return Err("test_path must not contain ..".into());
    }
    if t.len() > 512 {
        return Err("test_path too long".into());
    }
    if !t.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '/' | '_' | '-' | '.')
    }) {
        return Err("test_path: use only ASCII alnum, /, _, -, .".into());
    }
    Ok(t.to_string())
}

fn sanitize_pytest_keyword(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("keyword empty".into());
    }
    if t.len() > 240 {
        return Err("keyword too long (max 240)".into());
    }
    if !t.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '_' | ' ' | ':' | '-' | '=' | '.')
    }) {
        return Err("keyword: restricted charset for -k expression".into());
    }
    Ok(t.to_string())
}

/// argv after `pytest` (no shell); first arg is path under cwd.
pub fn project_pytest_build_args(
    test_path: &str,
    quiet: bool,
    keyword: Option<&str>,
    maxfail: Option<u32>,
) -> Result<Vec<String>, String> {
    let rel = sanitize_pytest_rel_path(test_path)?;
    let mut v = Vec::new();
    if quiet {
        v.push("-q".to_string());
    }
    if let Some(m) = maxfail {
        let m = m.clamp(1, 100);
        v.push("--maxfail".to_string());
        v.push(m.to_string());
    }
    if let Some(k) = keyword {
        let k = sanitize_pytest_keyword(k)?;
        v.push("-k".to_string());
        v.push(k);
    }
    v.push(rel);
    Ok(v)
}

/// Count `+`/`-` lines and `@@` hunks in unified diff text (git-style).
pub fn compress_unified_diff_stats_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let mut plus = 0usize;
    let mut minus = 0usize;
    let mut hunks = 0usize;
    for line in text.lines() {
        if line.starts_with("@@") {
            hunks += 1;
        } else if line.starts_with('+') && !line.starts_with("+++") {
            plus += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            minus += 1;
        }
    }
    let mut s = format!(
        "COMPRESS_UNIFIED_DIFF_STATS hunks={hunks} plus_lines={plus} minus_lines={minus} net_delta={}\n",
        plus as i64 - minus as i64
    );
    s.push_str("(excludes +++ / --- file headers; heuristic on unified diff)\n");
    let (out, _) = truncate_chars(&s, max_chars);
    out
}

/// First line split into columns (CSV/TSV/pipe/semicolon) for quick schema hints.
pub fn summarize_delimited_header_work(
    text: String,
    delimiter: &str,
    max_columns: usize,
    max_chars: usize,
) -> String {
    let max_columns = max_columns.clamp(4, 200);
    let max_chars = max_chars.clamp(128, MAX_RESPONSE_CHARS);
    let delim = match delimiter.trim().to_ascii_lowercase().as_str() {
        "comma" | "," => Some(","),
        "tab" | "tsv" | "\\t" => Some("\t"),
        "pipe" | "|" => Some("|"),
        "semicolon" | ";" => Some(";"),
        _ => None,
    };
    let Some(d) = delim else {
        return format!(
            "Error: unknown delimiter {:?}; use comma | tab | pipe | semicolon",
            delimiter
        );
    };
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let first = text.lines().next().unwrap_or("");
    let parts: Vec<&str> = if d == "\t" {
        first.split('\t').collect()
    } else {
        first.split(d).collect()
    };
    let n = parts.len().min(max_columns);
    let mut out = String::from("SUMMARIZE_DELIMITED_HEADER\n");
    out.push_str(&format!(
        "delimiter={:?} columns_total={} columns_shown={}\n\n",
        delimiter.trim(),
        parts.len(),
        n
    ));
    for (i, p) in parts.iter().take(n).enumerate() {
        let col = p.trim();
        let label = if col.is_empty() {
            "(empty)"
        } else {
            col
        };
        out.push_str(&format!("{i}: {:?}\n", label));
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

pub fn git_write_enabled() -> bool {
    std::env::var("MINT_GIT_WRITE_ENABLE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

pub fn git_stash_list_work(dir: PathBuf) -> String {
    match Command::new("git")
        .current_dir(&dir)
        .args(["stash", "list"])
        .output()
    {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            format!(
                "GIT_STASH_LIST dir={}\nexit: {}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}\n",
                dir.display(),
                out.status
            )
        }
        Err(e) => format!("Error: failed to run git: {e}"),
    }
}

pub fn git_worktree_list_work(dir: PathBuf) -> String {
    match Command::new("git")
        .current_dir(&dir)
        .args(["worktree", "list"])
        .output()
    {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            format!(
                "GIT_WORKTREE_LIST dir={}\nexit: {}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}\n",
                dir.display(),
                out.status
            )
        }
        Err(e) => format!("Error: failed to run git: {e}"),
    }
}

pub fn git_checkout_work(dir: PathBuf, target: String) -> String {
    if !git_write_enabled() {
        return "Error: git checkout disabled. Set MINT_GIT_WRITE_ENABLE=1 (mutates working tree)."
            .to_string();
    }
    let t = target.trim();
    if t.is_empty() {
        return "Error: target branch/ref must not be empty".to_string();
    }
    match Command::new("git")
        .current_dir(&dir)
        .args(["checkout", t])
        .output()
    {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            format!(
                "GIT_CHECKOUT dir={} target={}\nexit: {}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}\n",
                dir.display(),
                t,
                out.status
            )
        }
        Err(e) => format!("Error: failed to run git: {e}"),
    }
}

pub fn git_stash_push_work(dir: PathBuf, message: Option<String>) -> String {
    if !git_write_enabled() {
        return "Error: git stash push disabled. Set MINT_GIT_WRITE_ENABLE=1.".to_string();
    }
    let mut cmd = Command::new("git");
    cmd.current_dir(&dir).arg("stash").arg("push");
    if let Some(m) = message.filter(|s| !s.trim().is_empty()) {
        cmd.args(["-m", m.trim()]);
    }
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            format!(
                "GIT_STASH_PUSH dir={}\nexit: {}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}\n",
                dir.display(),
                out.status
            )
        }
        Err(e) => format!("Error: failed to run git: {e}"),
    }
}

/// Keep first N non-empty paragraphs (split on blank lines), then bound by max_chars.
pub fn compress_paragraphs_work(text: String, max_paragraphs: usize, max_chars: usize) -> String {
    let max_paragraphs = clamp_usize(max_paragraphs, 1, 200);
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let paras: Vec<&str> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    let n_para = paras.len();
    let kept: Vec<&str> = paras.into_iter().take(max_paragraphs).collect();
    let n_kept = kept.len();
    let s = kept.join("\n\n");
    let (s, _) = truncate_chars(&s, max_chars);
    format!(
        "COMPRESS_PARAGRAPHS max_paragraphs={} paragraphs_in={} paragraphs_kept={} max_chars={}\n\n{s}",
        max_paragraphs, n_para, n_kept, max_chars,
    )
}

/// Write path/size/mtime JSONL manifest under `.mint-mcp/file-manifest.jsonl`.
pub fn codebase_manifest_write_work(search_root: PathBuf, max_files: usize) -> String {
    let max_files = max_files.clamp(100, 25_000);
    let mint_dir = search_root.join(".mint-mcp");
    if let Err(e) = fs::create_dir_all(&mint_dir) {
        return format!("Error: create .mint-mcp: {e}");
    }
    let out_path = mint_dir.join("file-manifest.jsonl");

    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut f = match fs::File::create(&out_path) {
        Ok(f) => f,
        Err(e) => return format!("Error: create manifest: {e}"),
    };

    let mut n: usize = 0;
    let mut truncated = false;
    for entry in walk.build().flatten() {
        if n >= max_files {
            truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(&search_root)
            .map(|p| normalize_rel_path(p))
            .unwrap_or_else(|_| normalize_rel_path(path));
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let line = serde_json::json!({
            "path": rel,
            "size_bytes": meta.len(),
            "mtime_unix": mtime,
        })
        .to_string();
        if writeln!(f, "{line}").is_err() {
            return format!("Error: write manifest at {}", out_path.display());
        }
        n += 1;
    }

    format!(
        "CODEBASE_MANIFEST_WROTE path={} lines_written={} truncated={truncated}\nhint: read this file with read_file for a stable code-database slice; re-run to refresh.\n",
        out_path.display(),
        n
    )
}

pub fn git_push_enabled() -> bool {
    std::env::var("MINT_GIT_PUSH_ENABLE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

pub fn git_remote_enabled() -> bool {
    std::env::var("MINT_GIT_REMOTE_ENABLE")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

/// GitLens-style `git_log_or_diff`: `log` (oneline) or `diff` for a revision range.
pub fn git_log_or_diff_work(
    dir: PathBuf,
    action: &str,
    revision_range: Option<String>,
    since: Option<String>,
    until: Option<String>,
    authors: Vec<String>,
) -> String {
    let mut cmd = Command::new("git");
    cmd.current_dir(&dir).env("GIT_TERMINAL_PROMPT", "0");
    match action.trim().to_ascii_lowercase().as_str() {
        "log" => {
            cmd.arg("log").arg("--oneline").arg("--no-color");
            if let Some(r) = revision_range.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                cmd.arg(r);
            } else {
                cmd.arg("-n").arg("120");
            }
            if let Some(s) = since.as_ref().map(|x| x.trim()).filter(|s| !s.is_empty()) {
                cmd.args(["--since", s]);
            }
            if let Some(u) = until.as_ref().map(|x| x.trim()).filter(|s| !s.is_empty()) {
                cmd.args(["--until", u]);
            }
            for a in authors {
                let t = a.trim();
                if !t.is_empty() {
                    cmd.args(["--author", t]);
                }
            }
        }
        "diff" => {
            cmd.args(["diff", "--no-color"]);
            if let Some(r) = revision_range.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                cmd.arg(r);
            }
        }
        other => return format!("Error: unknown action {other:?}; use log | diff"),
    }
    match cmd.output() {
        Ok(out) => {
            let cap = if action.trim().eq_ignore_ascii_case("diff") {
                MAX_DIFF_CHARS
            } else {
                MAX_GIT_LOG_CHARS
            };
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (body, clipped) = truncate_chars(&stdout, cap);
            let mut s = format!(
                "GIT_LOG_OR_DIFF action={} dir={} exit={}\n",
                action,
                dir.display(),
                out.status
            );
            s.push_str(&body);
            if clipped {
                s.push_str("\n[truncated]\n");
            }
            if !out.stderr.is_empty() {
                s.push_str("\n--- STDERR ---\n");
                s.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            s
        }
        Err(e) => format!("Error: git: {e}"),
    }
}

/// GitLens-style `git branch`: list (`git branch -vv`) or create.
pub fn git_branch_extension_work(dir: PathBuf, action: &str, branch_name: Option<String>) -> String {
    match action.trim().to_ascii_lowercase().as_str() {
        "list" => match Command::new("git")
            .current_dir(&dir)
            .args(["branch", "-vv", "--no-color"])
            .output()
        {
            Ok(out) => {
                let st = String::from_utf8_lossy(&out.stdout);
                let (b, c) = truncate_chars(&st, MAX_GIT_BRANCH_CHARS);
                format!(
                    "GIT_BRANCH action=list dir={} exit={}\n{}{}",
                    dir.display(),
                    out.status,
                    b,
                    if c { "\n[truncated]\n" } else { "" }
                )
            }
            Err(e) => format!("Error: {e}"),
        },
        "create" => {
            if !git_write_enabled() {
                return "Error: branch create needs MINT_GIT_WRITE_ENABLE=1".to_string();
            }
            let Some(n) = branch_name
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            else {
                return "Error: branch create needs branch_name".to_string();
            };
            match Command::new("git")
                .current_dir(&dir)
                .args(["branch", n])
                .output()
            {
                Ok(out) => format!(
                    "GIT_BRANCH action=create dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
                    dir.display(),
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                Err(e) => format!("Error: {e}"),
            }
        }
        _ => "Error: action must be list | create".to_string(),
    }
}

/// GitLens-style `git worktree`: list or add (path + existing branch).
pub fn git_worktree_extension_work(
    dir: PathBuf,
    action: &str,
    worktree_path: Option<PathBuf>,
    branch: Option<String>,
) -> String {
    match action.trim().to_ascii_lowercase().as_str() {
        "list" => git_worktree_list_work(dir),
        "add" => {
            if !git_write_enabled() {
                return "Error: worktree add needs MINT_GIT_WRITE_ENABLE=1".to_string();
            }
            let Some(wp) = worktree_path else {
                return "Error: worktree add needs path (directory for new worktree)".to_string();
            };
            let Some(b) = branch
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            else {
                return "Error: worktree add needs branch (existing branch name)".to_string();
            };
            match Command::new("git")
                .current_dir(&dir)
                .args(["worktree", "add"])
                .arg(wp.as_os_str())
                .arg(b)
                .output()
            {
                Ok(out) => format!(
                    "GIT_WORKTREE action=add dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
                    dir.display(),
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                Err(e) => format!("Error: {e}"),
            }
        }
        _ => "Error: action must be list | add".to_string(),
    }
}

/// GitLens-style `git add` / `git commit`.
pub fn git_add_or_commit_work(
    dir: PathBuf,
    action: &str,
    files: Option<Vec<String>>,
    message: Option<String>,
) -> String {
    if !git_write_enabled() {
        return "Error: git add/commit needs MINT_GIT_WRITE_ENABLE=1".to_string();
    }
    match action.trim().to_ascii_lowercase().as_str() {
        "add" => {
            let mut cmd = Command::new("git");
            cmd.current_dir(&dir).arg("add").arg("--");
            match &files {
                Some(paths) if !paths.is_empty() => {
                    for p in paths {
                        let t = p.trim();
                        if !t.is_empty() {
                            cmd.arg(t);
                        }
                    }
                }
                _ => {
                    cmd.arg("-A");
                }
            }
            match cmd.output() {
                Ok(out) => format!(
                    "GIT_ADD_OR_COMMIT action=add dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
                    dir.display(),
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                Err(e) => format!("Error: {e}"),
            }
        }
        "commit" => {
            let Some(m) = message
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            else {
                return "Error: commit needs message".to_string();
            };
            let mut cmd = Command::new("git");
            cmd.current_dir(&dir).args(["commit", "-m", m]);
            if let Some(paths) = files {
                for p in paths {
                    let t = p.trim();
                    if !t.is_empty() {
                        cmd.arg(t);
                    }
                }
            }
            match cmd.output() {
                Ok(out) => format!(
                    "GIT_ADD_OR_COMMIT action=commit dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
                    dir.display(),
                    out.status,
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                Err(e) => format!("Error: {e}"),
            }
        }
        _ => "Error: action must be add | commit".to_string(),
    }
}

/// `git push` — requires `MINT_GIT_PUSH_ENABLE`; caller must enforce trusted network profile.
pub fn git_push_work(dir: PathBuf) -> String {
    if !git_push_enabled() {
        return "Error: git push disabled. Set MINT_GIT_PUSH_ENABLE=1 (trusted networks only).".to_string();
    }
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["push"])
        .output()
    {
        Ok(out) => format!(
            "GIT_PUSH dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// `git fetch --prune` — requires `MINT_GIT_REMOTE_ENABLE`; caller enforces trusted network.
pub fn git_fetch_work(dir: PathBuf) -> String {
    if !git_remote_enabled() {
        return "Error: git fetch disabled. Set MINT_GIT_REMOTE_ENABLE=1 (trusted networks only)."
            .to_string();
    }
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["fetch", "--prune"])
        .output()
    {
        Ok(out) => format!(
            "GIT_FETCH dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// `git pull --ff-only` — requires `MINT_GIT_REMOTE_ENABLE`; caller enforces trusted network.
pub fn git_pull_work(dir: PathBuf) -> String {
    if !git_remote_enabled() {
        return "Error: git pull disabled. Set MINT_GIT_REMOTE_ENABLE=1 (trusted networks only)."
            .to_string();
    }
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["pull", "--ff-only"])
        .output()
    {
        Ok(out) => format!(
            "GIT_PULL dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// `git remote -v` (read-only; no network I/O).
pub fn git_remote_list_work(dir: PathBuf) -> String {
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["remote", "-v"])
        .output()
    {
        Ok(out) => format!(
            "GIT_REMOTE_LIST dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// `git tag -l` with optional glob pattern (read-only).
pub fn git_tag_list_work(dir: PathBuf, pattern: Option<String>) -> String {
    let mut cmd = Command::new("git");
    cmd.current_dir(&dir).env("GIT_TERMINAL_PROMPT", "0");
    cmd.args(["tag", "-l"]);
    if let Some(p) = pattern.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        cmd.arg(p);
    }
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let (so, clipped) = truncate_chars(&stdout, MAX_GIT_BRANCH_CHARS);
            let mut s = format!(
                "GIT_TAG_LIST dir={} exit={}\nSTDOUT:\n{so}\nSTDERR:\n{stderr}\n",
                dir.display(),
                out.status
            );
            if clipped {
                s.push_str(&format!("\n[stdout truncated to {MAX_GIT_BRANCH_CHARS} chars]\n"));
            }
            s
        }
        Err(e) => format!("Error: {e}"),
    }
}

fn git_ls_files_lines(dir: &Path, extra: &[&str]) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir).env("GIT_TERMINAL_PROMPT", "0").arg("ls-files");
    for a in extra {
        cmd.arg(a);
    }
    match cmd.output() {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            let v: Vec<String> = s
                .lines()
                .map(|l| l.trim_end().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            Ok(v)
        }
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Tracked / untracked paths from the index (read-only). `mode`: tracked | untracked | all.
pub fn git_ls_files_work(dir: PathBuf, mode: &str) -> String {
    const MAX_LINES: usize = 12_000;
    let m = mode.trim().to_ascii_lowercase();
    let res: Result<Vec<String>, String> = match m.as_str() {
        "untracked" => git_ls_files_lines(&dir, &["-o", "--exclude-standard"]),
        "all" => match (
            git_ls_files_lines(&dir, &[]),
            git_ls_files_lines(&dir, &["-o", "--exclude-standard"]),
        ) {
            (Ok(mut a), Ok(mut b)) => {
                a.append(&mut b);
                a.sort();
                a.dedup();
                Ok(a)
            }
            (Err(e), _) | (_, Err(e)) => Err(e),
        },
        _ => git_ls_files_lines(&dir, &[]),
    };

    match res {
        Err(e) => format!("GIT_LS_FILES dir={} mode={m}\n\nError: {e}", dir.display()),
        Ok(mut lines) => {
            let n = lines.len();
            let truncated = n > MAX_LINES;
            lines.truncate(MAX_LINES);
            let mut body = String::new();
            for row in &lines {
                body.push_str(row);
                body.push('\n');
            }
            let s = format!(
                "GIT_LS_FILES dir={} mode={} lines_total={} lines_returned={}{}\n\n{body}",
                dir.display(),
                m,
                n,
                lines.len(),
                if truncated { " (truncated)" } else { "" }
            );
            let (s, _) = truncate_chars(&s, MAX_RESPONSE_CHARS);
            s
        }
    }
}

/// `git diff --stat` — compact change summary (read-only). Use `staged` **or** `revision_range`, not both.
pub fn git_diff_stat_work(
    dir: PathBuf,
    staged: bool,
    revision_range: Option<String>,
) -> String {
    if staged && revision_range.is_some() {
        return "Error: pass either staged=true or revision_range, not both".to_string();
    }
    let mut cmd = Command::new("git");
    cmd.current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["diff", "--stat", "--no-color"]);
    if staged {
        cmd.arg("--staged");
    }
    if let Some(r) = revision_range {
        let v = match validate_git_refspec(&r) {
            Ok(v) => v,
            Err(e) => return format!("Error: {e}"),
        };
        cmd.arg(v);
    }
    match cmd.output() {
        Ok(out) => {
            let mut body = format!(
                "GIT_DIFF_STAT dir={} staged={staged} exit={}\n",
                dir.display(),
                out.status.code().map(|c| c.to_string()).unwrap_or_default()
            );
            let stdout = String::from_utf8_lossy(&out.stdout);
            let (stdout, clipped) = truncate_chars(stdout.trim_end(), MAX_GIT_DIFF_STAT_CHARS);
            body.push_str(&stdout);
            if !out.stderr.is_empty() {
                body.push_str("\nSTDERR:\n");
                body.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            if clipped {
                body.push_str(&format!(
                    "\n\n[truncated to {MAX_GIT_DIFF_STAT_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git diff --stat: {e}"),
    }
}

/// `git merge-base` for two commits (read-only).
pub fn git_merge_base_work(dir: PathBuf, commit_a: String, commit_b: String) -> String {
    let a = match validate_git_refspec(&commit_a) {
        Ok(v) => v,
        Err(e) => return format!("Error: commit_a: {e}"),
    };
    let b = match validate_git_refspec(&commit_b) {
        Ok(v) => v,
        Err(e) => return format!("Error: commit_b: {e}"),
    };
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["merge-base", &a, &b])
        .output()
    {
        Ok(out) if out.status.success() => {
            let hex = String::from_utf8_lossy(&out.stdout).trim().to_string();
            format!("GIT_MERGE_BASE dir={}\n{hex}\n", dir.display())
        }
        Ok(out) => format!(
            "GIT_MERGE_BASE dir={} exit={}\nSTDERR:\n{}\n",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// `git describe --tags --always --dirty` (read-only).
pub fn git_describe_work(dir: PathBuf) -> String {
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
    {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            format!("GIT_DESCRIBE dir={}\n{s}\n", dir.display())
        }
        Ok(out) => format!(
            "GIT_DESCRIBE dir={} exit={}\nSTDOUT:\n{}\nSTDERR:\n{}",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// Top contributors by commit count (`git shortlog`), read-only.
pub fn git_shortlog_work(dir: PathBuf, limit: usize) -> String {
    let n = limit.clamp(5, 200);
    let n_s = n.to_string();
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "shortlog",
            "-sn",
            "--no-merges",
            "-n",
            &n_s,
            "HEAD",
        ])
        .output()
    {
        Ok(out) => {
            let mut body = format!(
                "GIT_SHORTLOG dir={} limit={n} exit={}\n",
                dir.display(),
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
                    "\n\n[truncated to {MAX_GIT_LOG_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git shortlog: {e}"),
    }
}

/// Recent `git reflog` entries (read-only).
pub fn git_reflog_work(dir: PathBuf, limit: usize) -> String {
    let n = limit.clamp(5, 300);
    let n_s = n.to_string();
    match Command::new("git")
        .current_dir(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["reflog", "-n", &n_s, "--no-color"])
        .output()
    {
        Ok(out) => {
            let mut body = format!(
                "GIT_REFLOG dir={} limit={n} exit={}\n",
                dir.display(),
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
                    "\n\n[truncated to {MAX_GIT_LOG_CHARS} characters]"
                ));
            }
            body
        }
        Err(e) => format!("Error running git reflog: {e}"),
    }
}

/// Latest commit (read-only). `style`: oneline (default) | subject | body | medium (%h %s committer).
pub fn git_last_commit_work(dir: PathBuf, style: &str) -> String {
    let st = style.trim().to_ascii_lowercase();
    let mut cmd = Command::new("git");
    cmd.current_dir(&dir).env("GIT_TERMINAL_PROMPT", "0");
    match st.as_str() {
        "body" | "message" | "full" => {
            cmd.args(["log", "-1", "--pretty=format:%B"]);
        }
        "subject" | "title" => {
            cmd.args(["log", "-1", "--pretty=format:%s"]);
        }
        "medium" => {
            cmd.args(["log", "-1", "--pretty=format:%h %s (%cn)"]);
        }
        _ => {
            cmd.args(["log", "-1", "--oneline"]);
        }
    }
    match cmd.output() {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim_end().to_string();
            let (s, clipped) = truncate_chars(&s, MAX_GIT_LOG_CHARS);
            let mut body = format!(
                "GIT_LAST_COMMIT dir={} style={} exit=0\n{s}\n",
                dir.display(),
                st
            );
            if clipped {
                body.push_str(&format!("\n[truncated to {MAX_GIT_LOG_CHARS} chars]\n"));
            }
            body
        }
        Ok(out) => format!(
            "GIT_LAST_COMMIT dir={} exit={}\nSTDERR:\n{}\n",
            dir.display(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(e) => format!("Error: {e}"),
    }
}

/// Local file read or `git show ref:path` (GitLens `repository_get_file_content` subset).
pub fn repository_get_file_content_work(
    dir: PathBuf,
    file_path: String,
    git_ref: Option<String>,
) -> String {
    let fp = file_path.trim();
    if fp.is_empty() {
        return "Error: file_path required".to_string();
    }
    if fp.contains("..") {
        return "Error: file_path must not contain ..".to_string();
    }
    if let Some(r) = git_ref.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let obj = format!("{}:{}", r, fp.replace('\\', "/"));
        match Command::new("git")
            .current_dir(&dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(["show", &obj])
            .output()
        {
            Ok(out) if out.status.success() => {
                let s = String::from_utf8_lossy(&out.stdout);
                let (b, c) = truncate_chars(&s, MAX_RESPONSE_CHARS);
                format!(
                    "REPOSITORY_GET_FILE_CONTENT ref={r} path={fp}\n\n{b}{}",
                    if c { "\n[truncated]\n" } else { "" }
                )
            }
            Ok(out) => format!(
                "Error: git show: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
            Err(e) => format!("Error: {e}"),
        }
    } else {
        let full = dir.join(fp);
        read_file_work(full, None, None)
    }
}

/// First-occurrence line preservation: drop duplicate lines in order, then cap.
pub fn compress_unique_lines_work(text: String, max_lines: usize, max_chars: usize) -> String {
    let max_lines = max_lines.clamp(1, 10_000);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut seen = std::collections::HashSet::<String>::new();
    let mut kept: Vec<String> = Vec::new();
    for line in text.lines() {
        let owned = line.to_string();
        if seen.insert(owned.clone()) {
            kept.push(owned);
            if kept.len() >= max_lines {
                break;
            }
        }
    }
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("COMPRESS_UNIQUE_LINES max_lines={max_lines} unique_lines_kept={n} max_chars={max_chars}\n\n{body}")
}

/// Keep the first N lines of text (in order), then cap by characters. Unlike compress_unique_lines, does not dedupe.
pub fn compress_lines_head_work(text: String, head_lines: usize, max_chars: usize) -> String {
    let head_lines = head_lines.clamp(1, 10_000);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let kept: Vec<&str> = text.lines().take(head_lines).collect();
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("COMPRESS_LINES_HEAD head_lines={head_lines} lines_kept={n} max_chars={max_chars}\n\n{body}")
}

/// Keep the last N lines (in order), then cap by characters.
pub fn compress_lines_tail_work(text: String, tail_lines: usize, max_chars: usize) -> String {
    let tail_lines = tail_lines.clamp(1, 10_000);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let lines: Vec<&str> = text.lines().collect();
    let n_src = lines.len();
    let start = n_src.saturating_sub(tail_lines);
    let kept: Vec<&str> = lines[start..].to_vec();
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!(
        "COMPRESS_LINES_TAIL tail_lines={tail_lines} lines_kept={n} (from total_lines={n_src}) max_chars={max_chars}\n\n{body}"
    )
}

/// Collapse runs of blank lines to a single blank line, then cap by characters.
pub fn compress_collapse_blank_lines_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let collapsed = collapse_blank_lines_str(&text);
    let (body, _) = truncate_chars(&collapsed, max_chars);
    format!("COMPRESS_COLLAPSE_BLANK_LINES max_chars={max_chars}\n\n{body}")
}

/// Keep start + end character runs; omit the middle when over `max_chars` (Unicode scalar aware).
pub fn compress_truncate_middle_work(text: String, max_chars: usize, margin_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, MAX_RESPONSE_CHARS);
    let half_budget = (max_chars / 2).saturating_sub(80).max(120);
    let margin_chars = margin_chars.clamp(80, half_budget);
    let n = text.chars().count();
    if n <= max_chars {
        return format!(
            "COMPRESS_TRUNCATE_MIDDLE input_chars={n} max_chars={max_chars} margin_chars={margin_chars} (no truncation)\n\n{text}"
        );
    }
    let head: String = text.chars().take(margin_chars).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(margin_chars)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let omitted = n.saturating_sub(margin_chars * 2);
    let body = format!(
        "{head}\n\n[... omitted {omitted} chars ...]\n\n{tail}"
    );
    let (body, clipped) = truncate_chars(&body, max_chars);
    let mut out = format!(
        "COMPRESS_TRUNCATE_MIDDLE input_chars={n} max_chars={max_chars} margin_chars={margin_chars} omitted_chars={omitted}\n\n{body}"
    );
    if clipped {
        out.push_str(&format!("\n\n[further truncated to {max_chars} chars]"));
    }
    out
}

fn line_looks_like_markdown_bullet(t: &str) -> bool {
    let s = t.trim_start();
    s.starts_with("- ")
        || s.starts_with("* ")
        || s.starts_with("+ ")
        || s.starts_with("• ")
        || (s.len() >= 3
            && s
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            && s.contains(". "))
}

/// Pull bullet / numbered list lines (markdown-style) for a quick outline summary.
pub fn summarize_bullets_work(text: String, max_items: usize, max_chars: usize) -> String {
    let max_items = max_items.clamp(1, 250);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut kept: Vec<String> = Vec::new();
    for line in text.lines() {
        if line_looks_like_markdown_bullet(line) {
            kept.push(line.to_string());
            if kept.len() >= max_items {
                break;
            }
        }
    }
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("SUMMARIZE_BULLETS items_kept={n} max_items={max_items} max_chars={max_chars}\n\n{body}")
}

fn line_looks_like_markdown_heading(line: &str) -> bool {
    let s = line.trim_start();
    if !s.starts_with('#') {
        return false;
    }
    let mut n = 0usize;
    for ch in s.chars() {
        if ch == '#' && n < 6 {
            n += 1;
        } else {
            break;
        }
    }
    if n == 0 {
        return false;
    }
    s.chars().nth(n).map(|c| c == ' ' || c == '\t').unwrap_or(true)
}

/// Markdown ATX headings (`#`..`######`) in file order.
pub fn summarize_headings_work(text: String, max_headings: usize, max_chars: usize) -> String {
    let max_headings = max_headings.clamp(1, 300);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut kept: Vec<String> = Vec::new();
    for line in text.lines() {
        if line_looks_like_markdown_heading(line) {
            kept.push(line.to_string());
            if kept.len() >= max_headings {
                break;
            }
        }
    }
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("SUMMARIZE_HEADINGS headings_kept={n} max_headings={max_headings} max_chars={max_chars}\n\n{body}")
}

fn parse_atx_heading_title(line: &str) -> Option<(usize, String)> {
    if !line_looks_like_markdown_heading(line) {
        return None;
    }
    let t = line.trim_start();
    let mut level = 0usize;
    for ch in t.chars() {
        if ch == '#' && level < 6 {
            level += 1;
        } else {
            break;
        }
    }
    if level == 0 {
        return None;
    }
    let title: String = t.chars().skip(level).collect::<String>().trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some((level, title))
    }
}

/// Indented bullet TOC from ATX headings (tree-style outline; no LLM).
pub fn summarize_markdown_toc_work(text: String, max_entries: usize, max_chars: usize) -> String {
    let max_entries = max_entries.clamp(1, 400);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut lines_out: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some((level, title)) = parse_atx_heading_title(line) {
            let indent = "  ".repeat(level.saturating_sub(1));
            lines_out.push(format!("{indent}- {title}"));
            if lines_out.len() >= max_entries {
                break;
            }
        }
    }
    let n = lines_out.len();
    let body = lines_out.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("SUMMARIZE_MARKDOWN_TOC entries={n} max_entries={max_entries} max_chars={max_chars}\n\n{body}")
}

/// Per line: trim internal runs of spaces/tabs to single spaces (split_whitespace join).
pub fn compress_simplify_whitespace_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let simplified: String = text
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n");
    let (out, clipped) = truncate_chars(&simplified, max_chars);
    let mut s = format!(
        "COMPRESS_SIMPLIFY_WHITESPACE output_chars={} clipped={clipped}\n\n{out}",
        out.len()
    );
    if clipped {
        s.push_str(&format!("\n\n[truncated to {max_chars} chars]"));
    }
    s
}

/// Truncate each line to at most `max_line_chars` Unicode scalars (suffix … when cut). Tames wide logs/TSV.
pub fn compress_truncate_lines_work(text: String, max_line_chars: usize, max_chars: usize) -> String {
    let max_line_chars = max_line_chars.clamp(16, 512);
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let mut truncated_lines = 0usize;
    let out: String = text
        .lines()
        .map(|line| {
            let n = line.chars().count();
            if n <= max_line_chars {
                line.to_string()
            } else {
                truncated_lines += 1;
                let take = max_line_chars.saturating_sub(1);
                let head: String = line.chars().take(take).collect();
                format!("{head}…")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let (body, clipped) = truncate_chars(&out, max_chars);
    let mut s = format!(
        "COMPRESS_TRUNCATE_LINES max_line_chars={max_line_chars} lines_truncated={truncated_lines} output_chars={} clipped={clipped}\n\n{body}",
        body.len()
    );
    if clipped {
        s.push_str(&format!("\n\n[truncated to {max_chars} chars]"));
    }
    s
}

fn normalize_diff_path_token(pathish: &str) -> Option<String> {
    let t = pathish.trim();
    if t.is_empty() || t == "/dev/null" {
        return None;
    }
    let t = t
        .strip_prefix("a/")
        .or_else(|| t.strip_prefix("b/"))
        .unwrap_or(t);
    if t.is_empty() {
        None
    } else {
        Some(t.replace('\\', "/"))
    }
}

/// Unique file paths from unified diff `---` / `+++` headers (git-style).
pub fn summarize_diff_file_list_work(text: String, max_paths: usize, max_chars: usize) -> String {
    let max_paths = max_paths.clamp(4, 500);
    let max_chars = max_chars.clamp(128, MAX_RESPONSE_CHARS);
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let mut set: BTreeSet<String> = BTreeSet::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            if let Some(p) = normalize_diff_path_token(rest) {
                set.insert(p);
            }
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            if let Some(p) = normalize_diff_path_token(rest) {
                set.insert(p);
            }
        }
    }
    let total_unique = set.len();
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v.truncate(max_paths);
    let mut out = String::from("SUMMARIZE_DIFF_FILE_LIST\n");
    out.push_str(&format!(
        "paths_unique_total: {} paths_shown: {} (cap {})\n\n",
        total_unique,
        v.len(),
        max_paths
    ));
    for p in &v {
        out.push_str(p);
        out.push('\n');
    }
    if v.is_empty() {
        out.push_str("(no ---/+++ paths parsed; paste unified diff text)\n");
    }
    let (out, _) = truncate_chars(&out, max_chars);
    out
}

/// Replace `![alt](url)` with alt and `[text](url)` with text (heuristic; nested brackets may truncate).
pub fn compress_markdown_links_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let re_img = match Regex::new(r"!\[([^\]]*)\]\([^)]*\)") {
        Ok(r) => r,
        Err(_) => return "Error: internal regex".to_string(),
    };
    let re_link = match Regex::new(r"\[([^\]]+)\]\([^)]*\)") {
        Ok(r) => r,
        Err(_) => return "Error: internal regex".to_string(),
    };
    let after_img = re_img.replace_all(&text, "$1");
    let s = re_link.replace_all(&after_img, "$1");
    let (out, clipped) = truncate_chars(&s.to_string(), max_chars);
    let mut o = format!(
        "COMPRESS_MARKDOWN_LINKS output_chars={} clipped={clipped}\n\n{out}",
        out.len()
    );
    if clipped {
        o.push_str(&format!("\n\n[truncated to {max_chars} chars]"));
    }
    o
}

fn json_value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn summarize_json_walk(
    v: &Value,
    path: &str,
    depth: usize,
    max_depth: usize,
    max_keys: usize,
    lines: &mut Vec<String>,
    budget: &mut usize,
) {
    if *budget == 0 {
        return;
    }
    match v {
        Value::Object(map) if depth <= max_depth => {
            if map.is_empty() {
                let label = if path.is_empty() {
                    "root".to_string()
                } else {
                    path.to_string()
                };
                lines.push(format!("{label}: object (0 keys)"));
                *budget = budget.saturating_sub(1);
                return;
            }
            let mut keys: Vec<_> = map.keys().map(|s| s.as_str()).collect();
            keys.sort();
            for k in keys.into_iter().take(max_keys) {
                if *budget == 0 {
                    break;
                }
                let child = &map[k];
                let p = if path.is_empty() {
                    k.to_string()
                } else {
                    format!("{path}.{k}")
                };
                match child {
                    Value::Object(inner) => {
                        lines.push(format!("{p}: object ({} keys)", inner.len()));
                        *budget = budget.saturating_sub(1);
                        if depth < max_depth && *budget > 0 {
                            summarize_json_walk(
                                child,
                                &p,
                                depth + 1,
                                max_depth,
                                max_keys,
                                lines,
                                budget,
                            );
                        }
                    }
                    Value::Array(a) => {
                        let first = a.first().map(json_value_kind).unwrap_or("empty");
                        lines.push(format!("{p}: array[len={}, first={first}]", a.len()));
                        *budget = budget.saturating_sub(1);
                        if depth < max_depth && *budget > 0 && !a.is_empty() {
                            let p0 = format!("{p}[0]");
                            summarize_json_walk(&a[0], &p0, depth + 1, max_depth, max_keys, lines, budget);
                        }
                    }
                    Value::String(s) => {
                        let n = s.chars().count();
                        lines.push(format!("{p}: string(chars={n})"));
                        *budget = budget.saturating_sub(1);
                    }
                    Value::Number(_) | Value::Bool(_) | Value::Null => {
                        lines.push(format!("{p}: {}", json_value_kind(child)));
                        *budget = budget.saturating_sub(1);
                    }
                }
            }
        }
        Value::Array(a) => {
            let label = if path.is_empty() { "root" } else { path };
            let first = a.first().map(json_value_kind).unwrap_or("empty");
            lines.push(format!("{label}: array[len={}, first={first}]", a.len()));
            *budget = budget.saturating_sub(1);
            if depth < max_depth && *budget > 0 && !a.is_empty() {
                let p0 = format!("{label}[0]");
                summarize_json_walk(&a[0], &p0, depth + 1, max_depth, max_keys, lines, budget);
            }
        }
        _ => {
            let label = if path.is_empty() { "root" } else { path };
            lines.push(format!("{label}: {}", json_value_kind(v)));
            *budget = budget.saturating_sub(1);
        }
    }
}

/// Shallow structural summary of JSON (keys, array lengths, types) without pretty-printing full values.
pub fn summarize_json_structure_work(
    raw: String,
    max_depth: usize,
    max_keys_per_level: usize,
    max_lines: usize,
    max_chars: usize,
) -> String {
    let max_depth = max_depth.clamp(0, 6);
    let max_keys_per_level = max_keys_per_level.clamp(4, 120);
    let max_lines = max_lines.clamp(8, 300);
    let max_chars = max_chars.clamp(256, MAX_RESPONSE_CHARS);
    let v: Value = match serde_json::from_str(raw.trim()) {
        Ok(v) => v,
        Err(e) => return format!("Error: invalid JSON: {e}"),
    };
    let mut lines: Vec<String> = Vec::new();
    let mut budget = max_lines;
    summarize_json_walk(
        &v,
        "",
        0,
        max_depth,
        max_keys_per_level,
        &mut lines,
        &mut budget,
    );
    let body = lines.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!(
        "SUMMARIZE_JSON_STRUCTURE max_depth={max_depth} max_keys_per_level={max_keys_per_level} lines_emitted={}\n\n{body}",
        lines.len()
    )
}

/// Levenshtein edit distance on Unicode scalar sequences (bounded for CPU safety).
pub fn text_levenshtein_distance_work(a: String, b: String, max_len_each: usize) -> String {
    let max_len_each = max_len_each.clamp(32, 2500);
    let (a, _) = truncate_chars(&a, max_len_each);
    let (b, _) = truncate_chars(&b, max_len_each);
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let n = ac.len();
    let m = bc.len();
    const MAX_CELLS: usize = 2_000_000;
    if n.saturating_mul(m) > MAX_CELLS {
        return format!(
            "Error: char product n*m={} exceeds {MAX_CELLS}; shorten strings or lower max_len_each",
            n.saturating_mul(m)
        );
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = usize::from(ac[i - 1] != bc[j - 1]);
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    let dist = prev[m];
    format!(
        "LEVENSHTEIN_DISTANCE a_chars={n} b_chars={m} distance={dist} max_len_each={max_len_each}\n"
    )
}

/// DJB2 over UTF-8 bytes (fast fingerprint for comparing agent outputs / cache keys).
pub fn text_djb2_hash_work(text: String) -> String {
    let (text, _) = truncate_chars(&text, MAX_RESPONSE_CHARS);
    let mut h: u32 = 5381;
    for byte in text.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(*byte));
    }
    format!(
        "DJB2_HASH hex={:08x} utf8_bytes={}\n",
        h,
        text.as_bytes().len()
    )
}

/// Gzip raw UTF-8 bytes and return standard base64 (compact agent payloads; no LLM).
pub fn compress_gzip_base64_work(text_input: Option<String>, input_path: Option<&PathBuf>) -> String {
    let raw: Vec<u8> = if let Some(t) = text_input.filter(|s| !s.is_empty()) {
        let b = t.into_bytes();
        if b.len() > MAX_CONVERT_BYTES {
            return format!("Error: text exceeds {MAX_CONVERT_BYTES} bytes for compress_gzip_base64");
        }
        b
    } else if let Some(p) = input_path {
        match read_file_bytes_capped(p, MAX_CONVERT_BYTES) {
            Ok(b) => b,
            Err(e) => return format!("Error: {e}"),
        }
    } else {
        return "Error: compress_gzip_base64 needs `text` or `input_path`".to_string();
    };
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    if let Err(e) = enc.write_all(&raw) {
        return format!("Error: gzip compress write: {e}");
    }
    let compressed = match enc.finish() {
        Ok(v) => v,
        Err(e) => return format!("Error: gzip finish: {e}"),
    };
    let b64 = STANDARD.encode(&compressed);
    format!(
        "COMPRESS_GZIP_BASE64 uncompressed_bytes={} compressed_bytes={} base64_chars={}\n\n{b64}\n",
        raw.len(),
        compressed.len(),
        b64.len()
    )
}

/// Decode standard base64 gzip and emit UTF-8 text, or describe binary (decompressed cap against zip bombs).
pub fn decompress_gzip_base64_work(payload_base64: String) -> String {
    let trimmed = payload_base64.trim();
    if trimmed.is_empty() {
        return "Error: decompress_gzip_base64 needs non-empty base64 payload".to_string();
    }
    let gzipped = match STANDARD.decode(trimmed) {
        Ok(b) => b,
        Err(e) => return format!("Error: base64 decode: {e}"),
    };
    let decoder = GzDecoder::new(std::io::Cursor::new(gzipped));
    let mut limited = decoder.take(MAX_GZIP_DECOMPRESSED_BYTES as u64);
    let mut buf = Vec::new();
    if let Err(e) = limited.read_to_end(&mut buf) {
        return format!("Error: gzip read: {e}");
    }
    match String::from_utf8(buf.clone()) {
        Ok(s) => {
            let (s, clipped) = truncate_chars(&s, MAX_RESPONSE_CHARS);
            let mut o = format!(
                "DECOMPRESS_GZIP_BASE64 utf8 output_chars={} clipped={clipped}\n\n{s}",
                s.len()
            );
            if clipped {
                o.push_str(&format!("\n\n[truncated to {MAX_RESPONSE_CHARS} chars]"));
            }
            o
        }
        Err(_) => {
            let b64 = STANDARD.encode(&buf);
            format!(
                "DECOMPRESS_GZIP_BASE64 binary_bytes={} (not valid UTF-8; payload as base64 below)\n\n{b64}\n",
                buf.len()
            )
        }
    }
}

/// First N non-empty lines (doc / log “teaser”).
pub fn summarize_first_lines_work(text: String, line_count: usize, max_chars: usize) -> String {
    let line_count = line_count.clamp(1, 500);
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let mut kept: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        kept.push(line.to_string());
        if kept.len() >= line_count {
            break;
        }
    }
    let n = kept.len();
    let body = kept.join("\n");
    let (body, _) = truncate_chars(&body, max_chars);
    format!("SUMMARIZE_FIRST_LINES lines_kept={n} requested={line_count} max_chars={max_chars}\n\n{body}")
}

fn leading_ascii_ws_width(s: &str) -> usize {
    s.chars()
        .take_while(|c| matches!(c, ' ' | '\t'))
        .count()
}

/// Strip the longest common leading ASCII space/tab prefix from non-empty lines (dedent pasted blocks).
pub fn compress_dedent_work(text: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1, MAX_RESPONSE_CHARS);
    let non_empty: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    let min_indent = if non_empty.is_empty() {
        0
    } else {
        non_empty
            .iter()
            .map(|l| leading_ascii_ws_width(l))
            .min()
            .unwrap_or(0)
    };
    let out: String = text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                line.chars().skip(min_indent).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let (body, _) = truncate_chars(&out, max_chars);
    format!(
        "COMPRESS_DEDENT min_common_indent={min_indent} max_chars={max_chars}\n\n{body}"
    )
}

/// Cheap token-frequency map from a bounded file sample (code-database style hint).
pub fn codebase_word_index_work(search_root: PathBuf, max_files: usize, top_n: usize) -> String {
    let max_files = max_files.clamp(50, 5000);
    let top_n = top_n.clamp(10, 150);
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);
    let mut n_files = 0usize;
    for entry in walk.build().flatten() {
        if n_files >= max_files {
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() > 512 * 1024 {
            continue;
        }
        let mut f = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut buf = vec![0u8; 8192];
        let n = match f.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let s = String::from_utf8_lossy(&buf[..n]);
        for w in s.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            let t = w.trim();
            if t.len() < 3 || t.len() > 48 {
                continue;
            }
            let key = t.to_ascii_lowercase();
            if key.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            *counts.entry(key).or_insert(0) += 1;
        }
        n_files += 1;
    }
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1));
    let mut out = format!(
        "CODEBASE_WORD_INDEX search_root={} files_sampled={} top_n={top_n}\n\n",
        search_root.display(),
        n_files
    );
    for (w, c) in v.into_iter().take(top_n) {
        out.push_str(&format!("{c}\t{w}\n"));
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// SHA-256 of the first `sample_bytes` of each file (bounded walk) — drift / index fingerprint.
pub fn codebase_shallow_hash_work(
    search_root: PathBuf,
    max_files: usize,
    sample_bytes: usize,
) -> String {
    let max_files = max_files.clamp(50, 8000);
    let sample_bytes = sample_bytes.clamp(256, 64 * 1024);
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut rows: Vec<(String, String)> = Vec::new();
    let mut truncated = false;

    for entry in walk.build().flatten() {
        if rows.len() >= max_files {
            truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(&search_root)
            .map(normalize_rel_path)
            .unwrap_or_else(|_| normalize_rel_path(path));
        let mut f = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let mut buf = vec![0u8; sample_bytes];
        let n = match f.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let digest = Sha256::digest(&buf[..n]);
        let hex = hex_encode_bytes(digest.as_slice());
        rows.push((rel, hex));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = format!(
        "CODEBASE_SHALLOW_HASH search_root={} sample_bytes={} entries={} walk_truncated={truncated}\n",
        search_root.display(),
        sample_bytes,
        rows.len()
    );
    out.push_str("format: sha256_hex<TAB>relative_path\n\n");
    for (rel, h) in rows {
        out.push_str(&format!("{h}\t{rel}\n"));
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// Sorted list of relative file paths (respects .gitignore). Bounded walk for lightweight code indexing / manifests.
pub fn codebase_path_list_work(
    search_root: PathBuf,
    max_paths: usize,
    max_depth: Option<usize>,
) -> String {
    let max_paths = max_paths.clamp(200, 50_000);
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut paths: Vec<String> = Vec::new();
    let mut truncated = false;

    for entry in walk.build().flatten() {
        if paths.len() >= max_paths {
            truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(&search_root)
            .map(normalize_rel_path)
            .unwrap_or_else(|_| normalize_rel_path(path));
        if let Some(md) = max_depth {
            let depth = rel.split('/').filter(|s| !s.is_empty()).count();
            if depth > md {
                continue;
            }
        }
        paths.push(rel);
    }

    paths.sort();
    let mut out = format!(
        "CODEBASE_PATH_LIST search_root={} max_depth={:?} entries={} walk_truncated={truncated}\n",
        search_root.display(),
        max_depth,
        paths.len()
    );
    out.push_str("format: one relative path per line\n\n");
    for p in paths {
        out.push_str(&p);
        out.push('\n');
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// Largest files by size (bounded scan, `.gitignore`-aware). Complements `codebase_path_list` / `codebase_shallow_hash`.
pub fn codebase_largest_files_work(
    search_root: PathBuf,
    max_files_scan: usize,
    top_n: usize,
) -> String {
    let scan_budget = max_files_scan.clamp(200, 50_000);
    let top_n = top_n.clamp(10, 500);
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut rows: Vec<(u64, String)> = Vec::new();
    let mut files_seen = 0usize;
    let mut scan_truncated = false;

    for entry in walk.build().flatten() {
        if files_seen >= scan_budget {
            scan_truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let rel = path
            .strip_prefix(&search_root)
            .map(normalize_rel_path)
            .unwrap_or_else(|_| normalize_rel_path(path));
        rows.push((meta.len(), rel));
        files_seen += 1;
    }

    rows.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = format!(
        "CODEBASE_LARGEST_FILES search_root={} files_scanned={} top_n={} scan_truncated={scan_truncated}\n",
        search_root.display(),
        rows.len().min(scan_budget),
        top_n
    );
    out.push_str("format: size_bytes<TAB>relative_path\n\n");
    for (sz, rel) in rows.into_iter().take(top_n) {
        out.push_str(&format!("{sz}\t{rel}\n"));
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

/// File basenames that appear under more than one relative path (bounded scan, `.gitignore`-aware).
pub fn codebase_duplicate_basenames_work(search_root: PathBuf, max_files_scan: usize) -> String {
    let scan_budget = max_files_scan.clamp(500, 35_000);
    let mut walk = WalkBuilder::new(&search_root);
    walk.git_ignore(true);
    walk.ignore(true);
    walk.hidden(false);

    let mut by_base: HashMap<String, Vec<String>> = HashMap::new();
    let mut files_seen = 0usize;
    let mut scan_truncated = false;

    for entry in walk.build().flatten() {
        if files_seen >= scan_budget {
            scan_truncated = true;
            break;
        }
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let base = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if base.is_empty() {
            continue;
        }
        let rel = path
            .strip_prefix(&search_root)
            .map(normalize_rel_path)
            .unwrap_or_else(|_| normalize_rel_path(path));
        by_base.entry(base).or_default().push(rel);
        files_seen += 1;
    }

    let mut dups: Vec<(String, Vec<String>)> = by_base
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();
    dups.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, paths) in dups.iter_mut() {
        paths.sort();
    }

    let mut out = format!(
        "CODEBASE_DUPLICATE_BASENAMES search_root={} files_scanned={} duplicate_name_count={} scan_truncated={scan_truncated}\n",
        search_root.display(),
        files_seen,
        dups.len()
    );
    out.push_str("format: basename then indented relative paths\n\n");
    for (name, paths) in dups {
        out.push_str(&format!("{name}\n"));
        for p in paths {
            out.push_str(&format!("  {p}\n"));
        }
        out.push('\n');
    }
    let (out, _) = truncate_chars(&out, MAX_RESPONSE_CHARS);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_unified_diff_stats_counts_hunks_and_lines() {
        let d = "+++ a\n--- b\n@@ -1 +1 @@\n-a\n+b\n";
        let s = compress_unified_diff_stats_work(d.to_string(), 2000);
        assert!(s.contains("hunks=1"), "{s}");
        assert!(s.contains("plus_lines=1"), "{s}");
        assert!(s.contains("minus_lines=1"), "{s}");
    }

    #[test]
    fn summarize_delimited_header_comma() {
        let s = summarize_delimited_header_work("a,b,c\n1,2,3".to_string(), "comma", 10, 2000);
        assert!(s.starts_with("SUMMARIZE_DELIMITED_HEADER"), "{s}");
        assert!(s.contains("columns_total=3"), "{s}");
    }

    #[test]
    fn project_cargo_test_build_args_smoke() {
        let v = project_cargo_test_build_args(None, None, false, false, None, false).unwrap();
        assert_eq!(v[0], "test");
        assert!(v.contains(&"short".to_string()));
    }

    #[test]
    fn project_cargo_check_build_args_workspace() {
        let v = project_cargo_check_build_args(None, true).unwrap();
        assert_eq!(v[0], "check");
        assert!(v.contains(&"--workspace".to_string()));
    }

    #[test]
    fn project_pytest_build_args_relative() {
        let v = project_pytest_build_args("tests/unit", true, None, Some(3)).unwrap();
        assert!(v.contains(&"-q".to_string()));
        assert!(v.contains(&"tests/unit".to_string()));
        assert!(v.contains(&"--maxfail".to_string()));
    }

    #[test]
    fn compress_truncate_lines_truncates_long_line() {
        let line = "x".repeat(200);
        let s = compress_truncate_lines_work(format!("ok\n{line}"), 32, 4000);
        assert!(s.contains("COMPRESS_TRUNCATE_LINES"), "{s}");
        assert!(s.contains("lines_truncated=1"), "{s}");
        assert!(s.contains('…'), "{s}");
    }

    #[test]
    fn summarize_diff_file_list_parses_paths() {
        let d = "--- a/src/foo.rs\n+++ b/src/foo.rs\n@@ -1 +1 @@\n";
        let s = summarize_diff_file_list_work(d.to_string(), 50, 2000);
        assert!(s.contains("SUMMARIZE_DIFF_FILE_LIST"), "{s}");
        assert!(s.contains("src/foo.rs"), "{s}");
        assert!(s.contains("paths_unique_total: 1"), "{s}");
    }

    #[test]
    fn convert_format_html_escape() {
        let s = convert_format_work(
            "html_escape".to_string(),
            Some("<a>\"b\"</a>".to_string()),
            None,
            None,
            true,
            4,
            1,
            80,
        );
        assert!(s.contains("CONVERT_FORMAT operation=html_escape"), "{s}");
        assert!(s.contains("&lt;a&gt;"), "{s}");
        assert!(s.contains("&quot;b&quot;"), "{s}");
    }
}
