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

pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024;
pub const MAX_RESPONSE_CHARS: usize = 20_000;
const MAX_DIR_ENTRIES: usize = 200;
const MAX_GREP_FILE_BYTES: u64 = 384 * 1024;
const MAX_GREP_MATCHES: usize = 200;
const MAX_FIND_FILES: usize = 600;
const MAX_TREE_ENTRIES: usize = 500;
const MAX_DIFF_CHARS: usize = 48_000;
const MAX_GIT_LOG_CHARS: usize = 36_000;
const MAX_GIT_SHOW_CHARS: usize = 48_000;
const MAX_GIT_BRANCH_CHARS: usize = 24_000;
const MAX_GIT_BLAME_CHARS: usize = 64_000;
const MAX_TAIL_LINES: usize = 500;
const MAX_STATS_SCAN_BYTES: u64 = 32 * 1024 * 1024;
const MAX_JSON_VALIDATE_BYTES: u64 = 1024 * 1024;
const MAX_BATCH_FILE_INFO: usize = 30;
const MAX_NAME_SEARCH_FILES: usize = 400;

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

/// One-shot overview for AI agents: resolved root, git, top-level names, bus presence, and hard limits.
pub fn workspace_context_work(root: PathBuf) -> String {
    let base = root.canonicalize().unwrap_or(root);
    let mut out = String::new();
    out.push_str("WORKSPACE_CONTEXT\n\n");
    out.push_str(&format!("project_root: {}\n", base.display()));

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
    out.push_str("\nPaths are resolved under project_root; absolute paths must stay inside the root.\n");
    out
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
pub fn compress_context_work(
    text: String,
    max_chars: usize,
    mode: String,
    dedupe_lines: bool,
    collapse_whitespace: bool,
) -> String {
    let max_chars = clamp_usize(max_chars, 1, MAX_RESPONSE_CHARS);
    let mut s = text;

    if collapse_whitespace {
        s = collapse_whitespace_work(&s);
    }
    if dedupe_lines {
        s = dedupe_consecutive_lines_work(&s);
    }

    let input_chars = s.chars().count();
    if input_chars <= max_chars {
        let out = format!("COMPRESS_CONTEXT mode={} input_chars={}\n\n{}", mode, input_chars, s);
        let (out, _) = truncate_chars(&out, max_chars);
        return out;
    }

    let mode_norm = mode.trim().to_ascii_lowercase();
    let body = match mode_norm.as_str() {
        "truncate" => {
            let (t, _) = truncate_chars(&s, max_chars);
            t
        }
        "head_tail" | _ => {
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
pub fn pack_context_work(
    segments: Vec<String>,
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
    let mut joined = String::new();
    joined.push_str(&format!("PACK_CONTEXT segments={} \n", n));
    for (i, seg) in segments.into_iter().enumerate() {
        joined.push_str(&format!("\n--- SEGMENT {} / {} ---\n", i + 1, n));
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
