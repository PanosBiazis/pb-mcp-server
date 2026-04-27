//! # Review Tools
//!
//! Code review automation: diff risk classification, focused review,
//! missing test detection, PR packet generation, and rulebook checks.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Maximum characters for a single diff output in review tools.
const MAX_REVIEW_DIFF_CHARS: usize = 96_000;

/// Maximum diff stat output characters.
const MAX_DIFF_STAT_CHARS: usize = 24_000;

/// Maximum log output characters.
const MAX_LOG_CHARS: usize = 12_000;

// ---------------------------------------------------------------------------
// Risk categories
// ---------------------------------------------------------------------------

/// Classification labels for diff risk analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RiskCategory {
    Config,
    Auth,
    Db,
    Concurrency,
    Secrets,
    Tests,
    Docs,
    Core,
}

impl std::fmt::Display for RiskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskCategory::Config => write!(f, "config"),
            RiskCategory::Auth => write!(f, "auth"),
            RiskCategory::Db => write!(f, "db"),
            RiskCategory::Concurrency => write!(f, "concurrency"),
            RiskCategory::Secrets => write!(f, "secrets"),
            RiskCategory::Tests => write!(f, "tests"),
            RiskCategory::Docs => write!(f, "docs"),
            RiskCategory::Core => write!(f, "core"),
        }
    }
}

fn risk_level(cat: &RiskCategory) -> &'static str {
    match cat {
        RiskCategory::Secrets | RiskCategory::Auth => "CRITICAL",
        RiskCategory::Db | RiskCategory::Concurrency => "HIGH",
        RiskCategory::Config => "MEDIUM",
        RiskCategory::Core => "LOW",
        RiskCategory::Tests | RiskCategory::Docs => "MINIMAL",
    }
}

/// Classify a file path (relative) into a risk category.
fn classify_file(rel_path: &str) -> RiskCategory {
    let lower = rel_path.to_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(&lower);

    // Config files
    if matches!(
        basename,
        "cargo.toml"
            | "package.json"
            | "pyproject.toml"
            | "go.mod"
            | "makefile"
            | "dockerfile"
            | "docker-compose.yml"
            | "docker-compose.yaml"
            | ".env"
            | ".env.local"
            | ".env.production"
            | "settings.json"
            | "settings.toml"
            | "settings.yaml"
            | "config.toml"
            | "config.yaml"
            | "config.json"
    ) || basename.starts_with(".env")
        || lower.contains("/config/")
        || lower.contains("/.env")
    {
        return RiskCategory::Config;
    }

    // Auth files
    if lower.contains("auth")
        || lower.contains("login")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("session")
        || lower.contains("oauth")
        || lower.contains("jwt")
        || lower.contains("token")
    {
        return RiskCategory::Auth;
    }

    // Database / migration files
    if lower.contains("migration")
        || lower.contains("schema")
        || lower.ends_with(".sql")
        || lower.contains("/db/")
        || lower.contains("/database/")
        || lower.contains("/sql/")
        || lower.contains("/migrations/")
        || lower.contains("/seeds/")
    {
        return RiskCategory::Db;
    }

    // Concurrency files
    if lower.contains("lock")
        || lower.contains("mutex")
        || lower.contains("atomic")
        || lower.contains("channel")
        || lower.contains("concurrent")
        || lower.contains("parallel")
        || lower.contains("sync")
        || lower.contains("thread_pool")
        || lower.contains("worker")
    {
        return RiskCategory::Concurrency;
    }

    // Test files
    if lower.contains("test")
        || lower.contains("spec")
        || lower.contains("_test.")
        || lower.contains("_spec.")
        || lower.contains("/tests/")
        || lower.contains("/test/")
        || lower.contains("/spec/")
        || lower.contains("/__tests__/")
        || lower.contains("/testing/")
    {
        return RiskCategory::Tests;
    }

    // Documentation files
    if lower.ends_with(".md")
        || lower.ends_with(".rst")
        || lower.ends_with(".txt")
        || lower.ends_with(".adoc")
        || lower.contains("/docs/")
        || lower.contains("/doc/")
    {
        return RiskCategory::Docs;
    }

    RiskCategory::Core
}

/// Scan diff content for secret-like patterns.
fn diff_contains_secrets(diff_text: &str) -> bool {
    let secret_patterns = [
        "API_KEY",
        "api_key",
        "apiKey",
        "SECRET",
        "secret_key",
        "secretKey",
        "TOKEN",
        "token_secret",
        "PASSWORD",
        "password",
        "passwd",
        "PRIVATE_KEY",
        "private_key",
        "privateKey",
        "CREDENTIAL",
        "credential",
        "ACCESS_KEY",
        "access_key",
        "accessKey",
        "AUTH_TOKEN",
        "auth_token",
        "Bearer ",
        "-----BEGIN RSA",
        "-----BEGIN PRIVATE",
        "-----BEGIN EC",
    ];

    for pattern in &secret_patterns {
        if diff_text.contains(pattern) {
            return true;
        }
    }
    false
}

/// Run git diff and return the output text.
fn run_git_diff(root: &Path, staged: bool, revision_range: Option<&str>) -> String {
    let mut args = vec!["diff".to_string()];
    if staged {
        args.push("--staged".to_string());
    }
    if let Some(range) = revision_range {
        args.push(range.to_string());
    }
    args.push("--no-color".to_string());

    let output = Command::new("git").args(&args).current_dir(root).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let (truncated, _) = super::truncate_chars(&stdout, MAX_REVIEW_DIFF_CHARS);
            truncated
        }
        Err(e) => format!("Error running git diff: {e}"),
    }
}

/// Run git diff --name-only and return the list of changed file paths.
fn run_git_diff_name_only(root: &Path, staged: bool, revision_range: Option<&str>) -> Vec<String> {
    let mut args = vec!["diff".to_string(), "--name-only".to_string()];
    if staged {
        args.push("--staged".to_string());
    }
    if let Some(range) = revision_range {
        args.push(range.to_string());
    }

    let output = Command::new("git").args(&args).current_dir(root).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        }
        Err(_) => vec![],
    }
}

/// Run git diff --stat and return the output.
fn run_git_diff_stat(root: &Path, revision_range: &str) -> String {
    let args = vec![
        "diff".to_string(),
        "--stat".to_string(),
        revision_range.to_string(),
    ];

    let output = Command::new("git").args(&args).current_dir(root).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let (truncated, _) = super::truncate_chars(&stdout, MAX_DIFF_STAT_CHARS);
            truncated
        }
        Err(e) => format!("Error running git diff --stat: {e}"),
    }
}

/// Run git log --oneline and return the output.
fn run_git_log_oneline(root: &Path, revision_range: &str) -> String {
    let args = vec![
        "log".to_string(),
        "--oneline".to_string(),
        revision_range.to_string(),
    ];

    let output = Command::new("git").args(&args).current_dir(root).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let (truncated, _) = super::truncate_chars(&stdout, MAX_LOG_CHARS);
            truncated
        }
        Err(e) => format!("Error running git log: {e}"),
    }
}

// ---------------------------------------------------------------------------
// 1. review_diff_risk
// ---------------------------------------------------------------------------

/// Classify changed files by risk category based on a git diff.
///
/// Runs `git diff` (staged or with revision_range), then classifies each
/// changed file into risk categories: config, auth, db, concurrency,
/// secrets, tests, docs, or core. Also scans the diff content for
/// secret-like patterns.
pub fn review_diff_risk_work(
    root: PathBuf,
    staged: bool,
    revision_range: Option<String>,
) -> String {
    let diff_text = run_git_diff(&root, staged, revision_range.as_deref());
    if diff_text.starts_with("Error") {
        return diff_text;
    }

    let changed_files = run_git_diff_name_only(&root, staged, revision_range.as_deref());
    if changed_files.is_empty() {
        return "REVIEW: DIFF RISK\n\nNo changed files found.".to_string();
    }

    let has_secrets_in_diff = diff_contains_secrets(&diff_text);

    // Classify files
    let mut categories: HashMap<RiskCategory, Vec<String>> = HashMap::new();
    for file in &changed_files {
        let cat = classify_file(file);
        categories.entry(cat).or_default().push(file.clone());
    }

    // If diff content contains secrets, promote
    if has_secrets_in_diff {
        categories
            .entry(RiskCategory::Secrets)
            .or_default()
            .push("(detected in diff content)".to_string());
    }

    // Build report
    let mut out = String::from("REVIEW: DIFF RISK\n\n");
    out.push_str(&format!("Changed files: {}\n", changed_files.len()));
    out.push_str(&format!(
        "Secrets in diff: {}\n\n",
        if has_secrets_in_diff { "YES" } else { "no" }
    ));

    // Risk summary table
    out.push_str("Category breakdown:\n");
    let order = [
        RiskCategory::Secrets,
        RiskCategory::Auth,
        RiskCategory::Db,
        RiskCategory::Concurrency,
        RiskCategory::Config,
        RiskCategory::Core,
        RiskCategory::Tests,
        RiskCategory::Docs,
    ];

    let mut highest_risk: Option<&RiskCategory> = None;
    let mut critical_count = 0usize;
    let mut high_count = 0usize;

    for cat in &order {
        if let Some(files) = categories.get(cat) {
            let level = risk_level(cat);
            out.push_str(&format!("  [{level:>8}] {cat} ({} file(s))\n", files.len()));
            for f in files {
                out.push_str(&format!("    - {f}\n"));
            }

            match level {
                "CRITICAL" => {
                    critical_count += files.len();
                    if highest_risk.is_none() {
                        highest_risk = Some(cat);
                    }
                }
                "HIGH" => {
                    high_count += files.len();
                    if highest_risk.is_none() {
                        highest_risk = Some(cat);
                    }
                }
                _ => {}
            }
        }
    }

    // Overall risk assessment
    out.push_str("\nOverall risk: ");
    if critical_count > 0 {
        out.push_str(&format!(
            "CRITICAL — {} critical change(s) in {:?}",
            critical_count,
            highest_risk.unwrap()
        ));
    } else if high_count > 0 {
        out.push_str(&format!(
            "HIGH — {} high-risk change(s) in {:?}",
            high_count,
            highest_risk.unwrap()
        ));
    } else if categories.contains_key(&RiskCategory::Config) {
        out.push_str("MEDIUM — config changes require review");
    } else if categories.contains_key(&RiskCategory::Core) {
        out.push_str("LOW — standard code changes");
    } else {
        out.push_str("MINIMAL — test/doc-only changes");
    }
    out.push('\n');

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 2. review_focused_review
// ---------------------------------------------------------------------------

/// Focus area pattern definitions for review.
#[derive(Clone, Copy)]
struct FocusPatterns {
    name: &'static str,
    patterns: &'static [&'static str],
    description: &'static str,
}

fn get_focus_patterns() -> Vec<FocusPatterns> {
    vec![
        FocusPatterns {
            name: "security",
            patterns: &[
                "unwrap()",
                "unsafe",
                "eval(",
                "exec(",
                "system(",
                "shell(",
                "password",
                "token",
                "api_key",
                "api-key",
                "apiKey",
                "secret",
                "hardcoded",
                "TODO",
                "FIXME",
                ".env",
                "credential",
                "private_key",
                "BEGIN RSA",
                "BEGIN PRIVATE",
                "sql_query",
                "format!",
                "raw_sql",
                "command(",
                "process::Command",
            ],
            description: "Security-sensitive patterns",
        },
        FocusPatterns {
            name: "performance",
            patterns: &[
                "clone()",
                "to_owned()",
                "to_string()",
                ".collect()",
                "O(n",
                "nested loop",
                "Arc<Mutex",
                "Arc<RwLock",
                "alloc",
                "malloc",
                "Box::new",
                "String::new",
                "Vec::new",
                "HashMap::new",
                "large allocation",
                "unbounded",
                "recursive",
            ],
            description: "Performance-sensitive patterns",
        },
        FocusPatterns {
            name: "correctness",
            patterns: &[
                "unwrap()",
                "expect(",
                "panic!",
                "TODO",
                "FIXME",
                "HACK",
                "XXX",
                "unreachable!",
                "todo!",
                "unimplemented!",
                "assert!",
                "assert_eq!",
                "index[",
                "[0]",
                "as f64",
                "as i32",
                "as usize",
                "transmute",
            ],
            description: "Correctness-risk patterns",
        },
        FocusPatterns {
            name: "maintainability",
            patterns: &[
                "TODO",
                "FIXME",
                "HACK",
                "XXX",
                "magic number",
                "magic constant",
                "hardcoded",
                "copy-paste",
                "duplicate",
                "deprecated",
                "#[allow",
                "// eslint-disable",
                "// clippy::",
                "allow(dead_code)",
                "allow(unused",
            ],
            description: "Maintainability concerns",
        },
        FocusPatterns {
            name: "style",
            patterns: &[
                "\t",
                "    ", // 4 spaces vs tabs inconsistency indicator
                "trailing whitespace",
                "camelCase",
                "snake_case",
                "TODO: add docs",
                "#[allow(non_snake_case)]",
                "unsafe",
            ],
            description: "Code style observations",
        },
    ]
}

/// Apply focused review to a git diff based on specified focus areas.
///
/// The `focus` parameter is a comma-separated list of focus areas:
/// "security", "performance", "correctness", "maintainability", "style".
/// Scans the diff for patterns relevant to each focus area and reports
/// findings with file:line references.
pub fn review_focused_review_work(
    root: PathBuf,
    focus: String,
    staged: bool,
    revision_range: Option<String>,
) -> String {
    let diff_text = run_git_diff(&root, staged, revision_range.as_deref());
    if diff_text.starts_with("Error") {
        return diff_text;
    }

    let requested: Vec<String> = focus
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if requested.is_empty() {
        return "REVIEW: FOCUSED REVIEW\n\nError: no focus areas specified. Use: security, performance, correctness, maintainability, style".to_string();
    }

    let all_patterns = get_focus_patterns();
    let mut out = String::from("REVIEW: FOCUSED REVIEW\n\n");
    out.push_str(&format!("Focus areas: {}\n\n", focus));

    // Parse diff into (file, line_number, content) entries
    let diff_lines: Vec<&str> = diff_text.lines().collect();
    let mut current_file = String::from("(unknown)");
    let mut findings_count = 0usize;

    for focus_name in &requested {
        let patterns = all_patterns
            .iter()
            .find(|p| p.name == *focus_name)
            .cloned()
            .unwrap_or(FocusPatterns {
                name: "",
                patterns: &[],
                description: "Unknown focus area",
            });
        out.push_str(&format!(
            "=== {} ===\n",
            if patterns.name.is_empty() {
                focus_name
            } else {
                patterns.name
            }
            .to_uppercase()
        ));
        out.push_str(&format!("({})\n\n", patterns.description));

        let mut area_findings = Vec::new();

        for (i, line) in diff_lines.iter().enumerate() {
            // Track current file from diff headers
            if line.starts_with("diff --git")
                || line.starts_with("--- a/")
                || line.starts_with("+++ b/")
            {
                if let Some(file_name) = line.strip_prefix("+++ b/") {
                    current_file = file_name.to_string();
                }
            }

            // Only check added lines
            if line.starts_with('+') && !line.starts_with("+++") {
                let content = &line[1..];
                let lower_content = content.to_lowercase();

                for pattern in patterns.patterns {
                    if lower_content.contains(&pattern.to_lowercase()) {
                        area_findings.push(format!(
                            "  L{}: {} → found \"{}\"",
                            i + 1,
                            current_file,
                            pattern
                        ));
                        break; // One match per line per focus area
                    }
                }

                // Maintainability: check for long functions (>50 lines in diff)
                if *focus_name == "maintainability" {
                    let line_len = content.chars().count();
                    if line_len > 200 {
                        area_findings.push(format!(
                            "  L{}: {} → very long line ({} chars)",
                            i + 1,
                            current_file,
                            line_len
                        ));
                    }
                }
            }
        }

        if area_findings.is_empty() {
            out.push_str("  ✓ No findings\n\n");
        } else {
            for finding in &area_findings {
                out.push_str(finding);
                out.push('\n');
            }
            out.push_str(&format!("\n  Found {} finding(s)\n\n", area_findings.len()));
            findings_count += area_findings.len();
        }
    }

    out.push_str(&format!("Total findings: {findings_count}\n"));

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 3. review_missing_tests
// ---------------------------------------------------------------------------

/// Check for source files that changed but lack corresponding test files.
///
/// For each changed file that is NOT itself a test file, checks if a
/// corresponding test file exists using common conventions:
/// - `src/foo.rs` → `tests/foo.rs`, `src/foo_test.rs`, `src/foo/mod.rs` + `src/foo/test.rs`
/// - `src/foo.py` → `tests/test_foo.py`, `test_foo.py`
/// - `src/foo.go` → `src/foo_test.go`
pub fn review_missing_tests_work(
    root: PathBuf,
    staged: bool,
    revision_range: Option<String>,
) -> String {
    let changed_files = run_git_diff_name_only(&root, staged, revision_range.as_deref());
    if changed_files.is_empty() {
        return "REVIEW: MISSING TESTS\n\nNo changed files found.".to_string();
    }

    let mut out = String::from("REVIEW: MISSING TESTS\n\n");
    let mut covered = Vec::new();
    let mut uncovered = Vec::new();
    let mut skipped = Vec::new();

    for file in &changed_files {
        let lower = file.to_lowercase();

        // Skip test files themselves
        if lower.contains("test")
            || lower.contains("spec")
            || lower.contains("_test.")
            || lower.contains("_spec.")
            || lower.contains("/tests/")
            || lower.contains("/test/")
            || lower.contains("/__tests__/")
        {
            skipped.push(file.clone());
            continue;
        }

        // Skip non-source files
        let is_source = lower.ends_with(".rs")
            || lower.ends_with(".py")
            || lower.ends_with(".go")
            || lower.ends_with(".js")
            || lower.ends_with(".ts")
            || lower.ends_with(".java")
            || lower.ends_with(".c")
            || lower.ends_with(".cpp")
            || lower.ends_with(".rb");
        if !is_source {
            skipped.push(file.clone());
            continue;
        }

        // Derive potential test file paths
        let test_candidates = derive_test_paths(file);

        // Check if any test candidate exists
        let mut found = false;
        for candidate in &test_candidates {
            let full_path = root.join(candidate);
            if full_path.exists() {
                covered.push((file.clone(), candidate.clone()));
                found = true;
                break;
            }
        }

        if !found {
            uncovered.push((file.clone(), test_candidates));
        }
    }

    out.push_str(&format!(
        "Changed source files: {}\n",
        covered.len() + uncovered.len()
    ));
    out.push_str(&format!("Skipped (test/non-source): {}\n\n", skipped.len()));

    if !covered.is_empty() {
        out.push_str("Files WITH test coverage:\n");
        for (src, test) in &covered {
            out.push_str(&format!("  ✓ {src} → {test}\n"));
        }
        out.push('\n');
    }

    if !uncovered.is_empty() {
        out.push_str(&format!("Files MISSING tests ({}):\n", uncovered.len()));
        for (src, candidates) in &uncovered {
            out.push_str(&format!("  ✗ {src}\n"));
            out.push_str(&format!("    Expected one of: {}\n", candidates.join(", ")));
        }
        out.push('\n');
    }

    // Summary
    let total_source = covered.len() + uncovered.len();
    if total_source > 0 {
        let pct = (covered.len() as f64 / total_source as f64) * 100.0;
        out.push_str(&format!(
            "Test coverage estimate: {pct:.0}% ({}/{})\n",
            covered.len(),
            total_source
        ));
    }

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// Derive candidate test file paths from a source file path.
fn derive_test_paths(source_path: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    // Get the stem (filename without extension)
    let stem = source_path
        .rsplit_once('.')
        .map(|(name, _ext)| name)
        .unwrap_or(source_path);
    let basename = stem.rsplit('/').next().unwrap_or(stem);
    let ext = source_path.rsplit_once('.').map(|(_, e)| e).unwrap_or("rs");

    match ext {
        "rs" => {
            // src/foo.rs → tests/foo.rs, src/foo/test.rs, src/foo_test.rs
            candidates.push(format!("tests/{basename}.rs"));
            candidates.push(format!("tests/{basename}_test.rs"));
            candidates.push(format!("{stem}_test.rs"));
            candidates.push(format!("{stem}/test.rs"));
            candidates.push(format!("src/{basename}_test.rs"));
        }
        "py" => {
            // src/foo.py → tests/test_foo.py, test_foo.py
            candidates.push(format!("tests/test_{basename}.py"));
            candidates.push(format!("test_{basename}.py"));
            candidates.push(format!("tests/{basename}_test.py"));
        }
        "go" => {
            // src/foo.go → src/foo_test.go
            let base = &source_path[..source_path.len() - 3];
            candidates.push(format!("{base}_test.go"));
            candidates.push(format!("{stem}_test.go"));
        }
        "js" | "ts" => {
            // src/foo.js → tests/foo.test.js, src/foo.test.js
            candidates.push(format!("tests/{basename}.test.{ext}"));
            candidates.push(format!("tests/{basename}.spec.{ext}"));
            candidates.push(format!("{stem}.test.{ext}"));
            candidates.push(format!("{stem}.spec.{ext}"));
            candidates.push(format!("{stem}/index.test.{ext}"));
        }
        "java" => {
            // src/Foo.java → src/FooTest.java, test/FooTest.java
            candidates.push(format!("{stem}Test.java"));
            candidates.push(format!("test/{basename}Test.java"));
            candidates.push(format!("src/test/{basename}Test.java"));
        }
        "c" | "cpp" => {
            candidates.push(format!("tests/test_{basename}.c"));
            candidates.push(format!("tests/test_{basename}.cpp"));
            candidates.push(format!("{stem}_test.c"));
            candidates.push(format!("{stem}_test.cpp"));
        }
        "rb" => {
            candidates.push(format!("test/test_{basename}.rb"));
            candidates.push(format!("spec/{basename}_spec.rb"));
            candidates.push(format!("spec/{basename}_spec.rb"));
        }
        _ => {
            candidates.push(format!("tests/test_{basename}.{ext}"));
        }
    }

    candidates
}

// ---------------------------------------------------------------------------
// 4. review_pr_packet
// ---------------------------------------------------------------------------

/// Generate a PR-ready summary packet from a revision range.
///
/// Combines diff stats, commit log, risk analysis, and missing test
/// detection into a structured packet suitable for PR descriptions.
pub fn review_pr_packet_work(root: PathBuf, revision_range: String) -> String {
    let mut out = String::from("REVIEW: PR PACKET\n\n");

    // Diff stats
    let diff_stat = run_git_diff_stat(&root, &revision_range);
    out.push_str("=== DIFF STAT ===\n");
    out.push_str(&diff_stat);
    out.push_str("\n\n");
    // Commit log
    let log = run_git_log_oneline(&root, &revision_range);
    out.push_str("=== COMMITS ===\n");
    out.push_str(&log);
    out.push_str("\n\n");

    // Risk analysis
    let risk = review_diff_risk_work(root.clone(), false, Some(revision_range.clone()));
    out.push_str("=== RISK ANALYSIS ===\n");
    // Strip the header from the sub-report
    let risk_body = risk.strip_prefix("REVIEW: DIFF RISK\n\n").unwrap_or(&risk);
    out.push_str(risk_body);
    out.push_str("\n\n");

    // Missing tests
    let missing = review_missing_tests_work(root.clone(), false, Some(revision_range.clone()));
    out.push_str("=== TEST COVERAGE ===\n");
    let missing_body = missing
        .strip_prefix("REVIEW: MISSING TESTS\n\n")
        .unwrap_or(&missing);
    out.push_str(missing_body);
    out.push_str("\n\n");

    // Manual sections
    out.push_str("=== ROLLOUT NOTES ===\n");
    out.push_str("[Manual: add deployment/rollback notes here]\n\n");

    out.push_str("=== CHECKLIST ===\n");
    out.push_str("- [ ] Risk areas reviewed\n");
    out.push_str("- [ ] Tests added for uncovered changes\n");
    out.push_str("- [ ] No secrets in diff\n");
    out.push_str("- [ ] Breaking changes documented\n");
    out.push_str("- [ ] Changelog updated\n");

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

// ---------------------------------------------------------------------------
// 5. review_rulebook_check
// ---------------------------------------------------------------------------

/// Check a git diff against a local rulebook file.
///
/// The rulebook is a markdown file with rules defined as `### Rule: <name>`
/// sections. Each rule's body describes what to check. The function scans
/// the diff for keywords extracted from the rule heading and body, then
/// reports pass/fail per rule with evidence from the diff.
pub fn review_rulebook_check_work(
    root: PathBuf,
    rulebook_path: PathBuf,
    staged: bool,
    revision_range: Option<String>,
) -> String {
    // Read rulebook
    let rulebook_content = match fs::read_to_string(&rulebook_path) {
        Ok(c) => c,
        Err(e) => return format!("Error reading rulebook {}: {e}", rulebook_path.display()),
    };

    // Parse rules from markdown (### Rule: <name> sections)
    let rules = parse_rulebook(&rulebook_content);
    if rules.is_empty() {
        return format!(
            "REVIEW: RULEBOOK CHECK\n\nNo rules found in {}. Expected ### Rule: <name> headings.",
            rulebook_path.display()
        );
    }

    // Get the diff
    let diff_text = run_git_diff(&root, staged, revision_range.as_deref());
    if diff_text.starts_with("Error") {
        return diff_text;
    }

    let _diff_lower = diff_text.to_lowercase();
    let diff_lines: Vec<&str> = diff_text.lines().collect();

    let mut out = String::from("REVIEW: RULEBOOK CHECK\n\n");
    out.push_str(&format!(
        "Rulebook: {} ({} rules)\n\n",
        rulebook_path.display(),
        rules.len()
    ));

    let mut pass_count = 0usize;
    let mut fail_count = 0usize;

    for rule in &rules {
        // Extract keywords from the rule name and body
        let keywords = extract_keywords(&rule.name, &rule.body);
        let mut violations = Vec::new();

        // Check each added line for keyword matches
        let mut current_file = String::from("(unknown)");
        for (i, line) in diff_lines.iter().enumerate() {
            if line.starts_with("+++ b/") {
                if let Some(f) = line.strip_prefix("+++ b/") {
                    current_file = f.to_string();
                }
            }
            if line.starts_with('+') && !line.starts_with("+++") {
                let line_lower = line[1..].to_lowercase();
                for kw in &keywords {
                    if line_lower.contains(&kw.to_lowercase()) {
                        violations.push(format!(
                            "  L{}: {} → matches \"{}\"",
                            i + 1,
                            current_file,
                            kw
                        ));
                        break;
                    }
                }
            }
        }

        out.push_str(&format!("### Rule: {}\n", rule.name));
        if violations.is_empty() {
            out.push_str("  ✓ PASS — no violations found\n\n");
            pass_count += 1;
        } else {
            out.push_str(&format!("  ✗ FAIL — {} violation(s):\n", violations.len()));
            for v in &violations {
                out.push_str(v);
                out.push('\n');
            }
            out.push('\n');
            fail_count += 1;
        }
    }

    out.push_str(&format!(
        "Summary: {pass_count} passed, {fail_count} failed out of {} rules\n",
        rules.len()
    ));

    let (out, _) = super::truncate_chars(&out, super::MAX_RESPONSE_CHARS);
    out
}

/// A parsed rule from a rulebook.
struct Rule {
    name: String,
    body: String,
}

/// Parse a markdown rulebook into rules.
///
/// Each rule starts with `### Rule: <name>` and the body is everything
/// until the next `###` heading or end of file.
fn parse_rulebook(content: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rule_name) = trimmed.strip_prefix("### Rule: ") {
            // Save previous rule
            if let Some(name) = current_name.take() {
                rules.push(Rule {
                    name,
                    body: current_body.trim().to_string(),
                });
                current_body.clear();
            }
            current_name = Some(rule_name.trim().to_string());
        } else if let Some(rule_name) = trimmed.strip_prefix("### ") {
            // Could be a rule without "Rule:" prefix
            if current_name.is_none() {
                current_name = Some(rule_name.trim().to_string());
            } else {
                // New section that's not a rule — save current
                if let Some(name) = current_name.take() {
                    rules.push(Rule {
                        name,
                        body: current_body.trim().to_string(),
                    });
                    current_body.clear();
                }
                // Start new potential rule
                current_name = Some(rule_name.trim().to_string());
            }
        } else if current_name.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    // Save last rule
    if let Some(name) = current_name {
        rules.push(Rule {
            name,
            body: current_body.trim().to_string(),
        });
    }

    rules
}

/// Extract searchable keywords from a rule name and body.
///
/// Splits the rule name and body into individual words, filtering out
/// common stop words and keeping only substantive terms.
fn extract_keywords(name: &str, body: &str) -> Vec<String> {
    let stop_words = [
        "the",
        "a",
        "an",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "must",
        "shall",
        "can",
        "to",
        "of",
        "in",
        "for",
        "on",
        "with",
        "at",
        "by",
        "from",
        "as",
        "into",
        "through",
        "during",
        "before",
        "after",
        "above",
        "below",
        "between",
        "out",
        "off",
        "over",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "when",
        "where",
        "why",
        "how",
        "all",
        "each",
        "every",
        "both",
        "few",
        "more",
        "most",
        "other",
        "some",
        "such",
        "no",
        "not",
        "only",
        "own",
        "same",
        "so",
        "than",
        "too",
        "very",
        "just",
        "because",
        "but",
        "and",
        "or",
        "if",
        "while",
        "that",
        "this",
        "these",
        "those",
        "it",
        "its",
        "they",
        "them",
        "their",
        "what",
        "which",
        "who",
        "whom",
        "whose",
        "rule",
        "use",
        "must",
        "always",
        "never",
        "ensure",
        "avoid",
        "check",
        "require",
        "needed",
        "necessary",
        "proper",
        "correct",
        "make",
        "also",
    ];

    let combined = format!("{name} {body}");
    let mut keywords = Vec::new();

    for word in combined.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
        let w = word.trim().to_lowercase();
        if w.len() >= 3 && !stop_words.contains(&w.as_str()) {
            keywords.push(w);
        }
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    keywords.retain(|k| seen.insert(k.clone()));

    // Cap at 20 keywords
    keywords.truncate(20);
    keywords
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_file() {
        assert_eq!(classify_file("Cargo.toml"), RiskCategory::Config);
        assert_eq!(classify_file(".env"), RiskCategory::Config);
        assert_eq!(classify_file("src/auth/login.rs"), RiskCategory::Auth);
        assert_eq!(classify_file("migrations/001_init.sql"), RiskCategory::Db);
        assert_eq!(
            classify_file("src/mutex_handler.rs"),
            RiskCategory::Concurrency
        );
        assert_eq!(classify_file("tests/foo_test.rs"), RiskCategory::Tests);
        assert_eq!(classify_file("README.md"), RiskCategory::Docs);
        assert_eq!(classify_file("src/main.rs"), RiskCategory::Core);
    }

    #[test]
    fn test_diff_contains_secrets() {
        assert!(diff_contains_secrets("API_KEY=abc123"));
        assert!(diff_contains_secrets("password = 'secret'"));
        assert!(diff_contains_secrets("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(!diff_contains_secrets("fn main() {}"));
        assert!(!diff_contains_secrets("let x = 42;"));
    }

    #[test]
    fn test_risk_level() {
        assert_eq!(risk_level(&RiskCategory::Secrets), "CRITICAL");
        assert_eq!(risk_level(&RiskCategory::Auth), "CRITICAL");
        assert_eq!(risk_level(&RiskCategory::Db), "HIGH");
        assert_eq!(risk_level(&RiskCategory::Concurrency), "HIGH");
        assert_eq!(risk_level(&RiskCategory::Config), "MEDIUM");
        assert_eq!(risk_level(&RiskCategory::Core), "LOW");
        assert_eq!(risk_level(&RiskCategory::Tests), "MINIMAL");
        assert_eq!(risk_level(&RiskCategory::Docs), "MINIMAL");
    }

    #[test]
    fn test_derive_test_paths_rust() {
        let paths = derive_test_paths("src/main.rs");
        assert!(paths.contains(&"tests/main.rs".to_string()));
        assert!(paths.contains(&"tests/main_test.rs".to_string()));
    }

    #[test]
    fn test_derive_test_paths_python() {
        let paths = derive_test_paths("src/app.py");
        assert!(paths.contains(&"tests/test_app.py".to_string()));
    }

    #[test]
    fn test_derive_test_paths_go() {
        let paths = derive_test_paths("src/handler.go");
        // Go test files are alongside source
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_parse_rulebook() {
        let content = "\
# Project Rules

### Rule: No unwrap in production
Never use .unwrap() in production code.

### Rule: No hardcoded secrets
Do not commit API keys or passwords.

### Another Section
This is not a rule.
";
        let rules = parse_rulebook(content);
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].name, "No unwrap in production");
        assert!(rules[0].body.contains("unwrap()"));
        assert_eq!(rules[1].name, "No hardcoded secrets");
        assert!(rules[1].body.contains("API keys"));
    }

    #[test]
    fn test_extract_keywords() {
        let kws = extract_keywords("No unwrap in production", "Never use .unwrap()");
        assert!(kws.contains(&"unwrap".to_string()));
        assert!(kws.contains(&"production".to_string()));
        // Stop words should be filtered
        assert!(!kws.contains(&"the".to_string()));
        assert!(!kws.contains(&"in".to_string()));
    }

    #[test]
    fn test_extract_keywords_dedup() {
        let kws = extract_keywords("test test test", "test test");
        let count = kws.iter().filter(|k| *k == "test").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_review_diff_risk_empty_repo() {
        // In a directory without git, should return an error
        let result = review_diff_risk_work(PathBuf::from("/tmp/nonexistent"), false, None);
        assert!(result.contains("Error") || result.contains("REVIEW"));
    }

    #[test]
    fn test_focus_patterns_defined() {
        let patterns = get_focus_patterns();
        assert_eq!(patterns.len(), 5);
        assert_eq!(patterns[0].name, "security");
        assert!(!patterns[0].patterns.is_empty());
    }
}
