//! Security scanning utilities for detecting secrets and verifying file integrity.
//!
//! This module provides:
//! - **`SecretScanner`**: Pattern-based detection of leaked credentials, API keys,
//!   tokens, and other sensitive data in source files and text.
//! - **`IntegrityChecker`**: SHA-256 hash computation and verification for files
//!   and directory trees.
//!
//! # Example
//!
//! ```
//! use pb_mcp_server::security::scanner::SecretScanner;
//!
//! let scanner = SecretScanner::new();
//! let results = scanner.scan_text("key=AKIAIOSFODNN7EXAMPLE", "config.rs");
//! assert!(!results.is_empty());
//! ```

use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

// ============================================================================
// Severity enum
// ============================================================================

/// Severity level for a detected secret or vulnerability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    /// Low-risk informational finding.
    Low,
    /// Medium-risk finding that should be reviewed.
    Medium,
    /// High-risk finding requiring prompt attention.
    High,
    /// Critical finding — immediate remediation required.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

impl serde::Serialize for Severity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Severity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            _ => Err(serde::de::Error::custom(format!("unknown severity: {}", s))),
        }
    }
}

// ============================================================================
// SecretPattern
// ============================================================================

/// A single pattern used by [`SecretScanner`] to detect leaked secrets.
#[derive(Debug, Clone)]
pub struct SecretPattern {
    /// Human-readable name, e.g. "AWS Access Key", "GitHub Token".
    pub name: String,
    /// Compiled regex that matches the secret.
    pub pattern: Regex,
    /// Severity level assigned to matches of this pattern.
    pub severity: Severity,
    /// Description of what this pattern detects.
    pub description: String,
}

// ============================================================================
// ScanResult
// ============================================================================

/// A single finding produced by [`SecretScanner`].
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Path of the file where the secret was found.
    pub file_path: String,
    /// 1-based line number within the file.
    pub line_number: usize,
    /// Name of the [`SecretPattern`] that matched.
    pub pattern_name: String,
    /// Severity of the finding.
    pub severity: Severity,
    /// First 20 characters of the matched text (redacted).
    pub matched_text: String,
    /// The full source line with the secret replaced by `[REDACTED]`.
    pub line_content: String,
}

// ============================================================================
// SecretScanner
// ============================================================================

/// Scans text, files, and directory trees for leaked secrets.
///
/// # Built-in patterns
///
/// The scanner ships with patterns for common credential types including AWS
/// keys, GitHub tokens, private key headers, generic API keys, Bearer tokens,
/// Slack tokens, database URLs, and JWT tokens.
///
/// # File-size guard
///
/// Files larger than `max_file_size` (default **1 MiB**) are silently skipped
/// to avoid scanning binary blobs or enormous generated files.
#[derive(Debug, Clone)]
pub struct SecretScanner {
    /// List of patterns to detect.
    pub patterns: Vec<SecretPattern>,
    /// Maximum file size (in bytes) to scan. Files larger than this are skipped.
    pub max_file_size: u64,
}

impl SecretScanner {
    /// Default maximum file size: 1 MiB.
    pub const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;

    /// Create a new scanner pre-loaded with common secret patterns.
    pub fn new() -> Self {
        let patterns = vec![
            // AWS Access Key ID
            SecretPattern {
                name: "AWS Access Key ID".into(),
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                severity: Severity::Critical,
                description: "AWS Access Key ID (starts with AKIA)".into(),
            },
            // AWS Secret Access Key
            SecretPattern {
                name: "AWS Secret Key".into(),
                pattern: Regex::new(r"\b[0-9a-zA-Z/+]{40}\b").unwrap(),
                severity: Severity::Critical,
                description: "Potential AWS Secret Access Key (40-char base64-ish string)".into(),
            },
            // GitHub Personal Access Token
            SecretPattern {
                name: "GitHub Token".into(),
                pattern: Regex::new(r"gh[ps]_[0-9a-zA-Z]{36}").unwrap(),
                severity: Severity::Critical,
                description: "GitHub Personal Access Token (ghp_/ghs_ prefix)".into(),
            },
            // GitHub OAuth Access Token
            SecretPattern {
                name: "GitHub OAuth".into(),
                pattern: Regex::new(r"gho_[0-9a-zA-Z]{36}").unwrap(),
                severity: Severity::High,
                description: "GitHub OAuth Access Token (gho_ prefix)".into(),
            },
            // Private Key header
            SecretPattern {
                name: "Private Key".into(),
                pattern: Regex::new(r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----")
                    .unwrap(),
                severity: Severity::Critical,
                description: "PEM private key header".into(),
            },
            // Generic API key assignment
            SecretPattern {
                name: "Generic API Key".into(),
                pattern: Regex::new(
                    r#"(?i)(api[-_]?key|apikey|access[-_]?key)\s*[:=]\s*["\']?[0-9a-zA-Z]{20,}"#,
                )
                .unwrap(),
                severity: Severity::High,
                description: "Generic API key assignment pattern".into(),
            },
            // Bearer token
            SecretPattern {
                name: "Bearer Token".into(),
                pattern: Regex::new(r"Bearer [0-9a-zA-Z\-._~+/]+=*").unwrap(),
                severity: Severity::High,
                description: "HTTP Bearer authorization token".into(),
            },
            // Slack token
            SecretPattern {
                name: "Slack Token".into(),
                pattern: Regex::new(r"xox[bpors]-[0-9a-zA-Z-]+").unwrap(),
                severity: Severity::High,
                description: "Slack bot/user/token (xoxb/xoxp/xoxx prefix)".into(),
            },
            // Database URL
            SecretPattern {
                name: "Database URL".into(),
                pattern: Regex::new(r"(mysql|postgres|mongodb)://[^\s\x27\x22]+").unwrap(),
                severity: Severity::Critical,
                description: "Database connection string with embedded credentials".into(),
            },
            // JSON Web Token
            SecretPattern {
                name: "JWT Token".into(),
                pattern: Regex::new(r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+").unwrap(),
                severity: Severity::High,
                description: "JSON Web Token (JWT) header.payload fragment".into(),
            },
        ];

        Self {
            patterns,
            max_file_size: Self::DEFAULT_MAX_FILE_SIZE,
        }
    }

    /// Scan a text string for secrets.
    ///
    /// Returns a `Vec<ScanResult>` with one entry per match. Each result
    /// contains the redacted line content and the first 20 characters of the
    /// matched secret.
    pub fn scan_text(&self, text: &str, file_path: &str) -> Vec<ScanResult> {
        let mut results = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let line_number = line_idx + 1; // 1-based

            for pattern in &self.patterns {
                for mat in pattern.pattern.find_iter(line) {
                    let matched_text = mat.as_str();
                    // Show first 20 chars of the match (redacted)
                    let matched_display = if matched_text.len() > 20 {
                        format!("{}...", &matched_text[..20])
                    } else {
                        matched_text.to_string()
                    };

                    let line_content = Self::redact_secret(line);

                    results.push(ScanResult {
                        file_path: file_path.to_string(),
                        line_number,
                        pattern_name: pattern.name.clone(),
                        severity: pattern.severity,
                        matched_text: matched_display,
                        line_content,
                    });
                }
            }
        }

        results
    }

    /// Scan a single file for secrets.
    ///
    /// Returns an error if the file cannot be read. Files larger than
    /// `max_file_size` are silently skipped (empty result).
    pub fn scan_file(&self, path: &Path) -> Result<Vec<ScanResult>, String> {
        let metadata = fs::metadata(path).map_err(|e| format!("Cannot read metadata: {e}"))?;

        if metadata.len() > self.max_file_size {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read file {}: {e}", path.display()))?;

        Ok(self.scan_text(&content, &path.display().to_string()))
    }

    /// Scan a directory tree for secrets.
    ///
    /// Walks the directory recursively (using `ignore` crate to respect
    /// `.gitignore`). Scans up to `max_files` files. Returns an error if the
    /// path is not a directory or cannot be read.
    pub fn scan_directory(&self, path: &Path, max_files: usize) -> Result<Vec<ScanResult>, String> {
        if !path.is_dir() {
            return Err(format!("{} is not a directory", path.display()));
        }

        let mut all_results = Vec::new();
        let mut files_scanned = 0;

        let walker = ignore::WalkBuilder::new(path)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker {
            let entry = entry.map_err(|e| format!("Walk error: {e}"))?;

            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }

            if files_scanned >= max_files {
                break;
            }

            files_scanned += 1;

            let file_path = entry.path();
            match self.scan_file(file_path) {
                Ok(results) => all_results.extend(results),
                Err(_) => continue, // Skip files we cannot read
            }
        }

        Ok(all_results)
    }

    /// Replace detected secrets in a line of text with `[REDACTED]`.
    ///
    /// Uses a broad regex that catches common credential shapes (long
    /// alphanumeric strings, key-value assignments, Bearer tokens, PEM
    /// headers, database URLs, and JWT fragments).
    pub fn redact_secret(text: &str) -> String {
        let patterns = [
            // AWS Access Key IDs
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            // GitHub tokens
            Regex::new(r"gh[ps]_[0-9a-zA-Z]{36}").unwrap(),
            Regex::new(r"gho_[0-9a-zA-Z]{36}").unwrap(),
            // Slack tokens
            Regex::new(r"xox[bpors]-[0-9a-zA-Z-]+").unwrap(),
            // Bearer tokens
            Regex::new(r"Bearer [0-9a-zA-Z\-._~+/]+=*").unwrap(),
            // Private key headers
            Regex::new(r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----").unwrap(),
            // Database URLs (redact the credentials part)
            Regex::new(r"(mysql|postgres|mongodb)://[^\s\x27\x22]+").unwrap(),
            // JWT tokens
            Regex::new(r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+[A-Za-z0-9-_.+/]*=*").unwrap(),
            // Generic API key values
            Regex::new(
                r#"(?i)(api[-_]?key|apikey|access[-_]?key)\s*[:=]\s*["\']?[0-9a-zA-Z]{20,}"#,
            )
            .unwrap(),
            // Long base64-ish strings (AWS secret key candidates)
            Regex::new(r"[0-9a-zA-Z/+]{40,}").unwrap(),
        ];

        let mut result = text.to_string();
        for re in &patterns {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
        result
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// IntegrityChecker
// ============================================================================

/// SHA-256 hash computation and verification for files and directory trees.
///
/// Use this to detect tampering or unexpected changes to files.
pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Compute the SHA-256 hex digest of a file.
    ///
    /// Reads the file in chunks to avoid loading the entire file into memory.
    pub fn compute_file_hash(path: &Path) -> Result<String, String> {
        let mut file = fs::File::open(path)
            .map_err(|e| format!("Cannot open file {}: {e}", path.display()))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| format!("Cannot read file {}: {e}", path.display()))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Verify that a file's SHA-256 hash matches an expected value.
    ///
    /// Returns `Ok(true)` if it matches, `Ok(false)` if it doesn't.
    pub fn verify_file_hash(path: &Path, expected: &str) -> Result<bool, String> {
        let actual = Self::compute_file_hash(path)?;
        Ok(actual.eq_ignore_ascii_case(expected))
    }

    /// Compute SHA-256 hashes for all files in a directory tree.
    ///
    /// Returns a `HashMap` mapping relative file paths (relative to `path`) to
    /// their hex digests. Walks up to `max_files` entries.
    pub fn compute_directory_manifest(
        path: &Path,
        max_files: usize,
    ) -> Result<HashMap<String, String>, String> {
        if !path.is_dir() {
            return Err(format!("{} is not a directory", path.display()));
        }

        let mut manifest = HashMap::new();
        let mut files_scanned = 0;

        let walker = ignore::WalkBuilder::new(path)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker {
            let entry = entry.map_err(|e| format!("Walk error: {e}"))?;

            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }

            if files_scanned >= max_files {
                break;
            }

            files_scanned += 1;

            let file_path = entry.path();
            let hash = Self::compute_file_hash(file_path)?;

            let relative = file_path
                .strip_prefix(path)
                .unwrap_or(file_path)
                .display()
                .to_string();

            manifest.insert(relative, hash);
        }

        Ok(manifest)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn new_scanner() -> SecretScanner {
        SecretScanner::new()
    }

    // ---- Secret scanner detection tests ----

    #[test]
    fn test_secret_scanner_detects_aws_key() {
        let scanner = new_scanner();
        let results = scanner.scan_text("aws_access_key = AKIAIOSFODNN7EXAMPLE\n", "config.rs");

        assert!(!results.is_empty(), "Should detect AWS Access Key ID");

        let aws_findings: Vec<&ScanResult> = results
            .iter()
            .filter(|r| r.pattern_name == "AWS Access Key ID")
            .collect();
        assert!(
            !aws_findings.is_empty(),
            "Should have at least one AWS Access Key ID finding"
        );

        let finding = aws_findings[0];
        assert_eq!(finding.line_number, 1);
        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.matched_text.starts_with("AKIA"));
    }

    #[test]
    fn test_secret_scanner_detects_github_token() {
        let scanner = new_scanner();
        // ghp_ followed by 36 alphanumeric chars (40 chars total)
        let token = "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let results = scanner.scan_text(&format!("GITHUB_TOKEN={token}\n"), "env.rs");

        assert!(!results.is_empty(), "Should detect GitHub Token");

        let gh_findings: Vec<&ScanResult> = results
            .iter()
            .filter(|r| r.pattern_name == "GitHub Token")
            .collect();
        assert!(
            !gh_findings.is_empty(),
            "Should have at least one GitHub Token finding"
        );

        let finding = gh_findings[0];
        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn test_secret_scanner_detects_private_key() {
        let scanner = new_scanner();
        let results = scanner.scan_text(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----\n",
            "key.pem",
        );

        assert!(!results.is_empty(), "Should detect Private Key header");

        let pk_findings: Vec<&ScanResult> = results
            .iter()
            .filter(|r| r.pattern_name == "Private Key")
            .collect();
        assert!(
            !pk_findings.is_empty(),
            "Should have at least one Private Key finding"
        );

        let finding = pk_findings[0];
        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.line_content.contains("[REDACTED]"));
    }

    #[test]
    fn test_secret_scanner_redacts_secrets() {
        let input = "aws_key=AKIAIOSFODNN7EXAMPLE some other text";
        let redacted = SecretScanner::redact_secret(input);

        assert!(
            !redacted.contains("AKIAIOSFODNN7EXAMPLE"),
            "AWS key should be redacted"
        );
        assert!(
            redacted.contains("[REDACTED]"),
            "Redacted output should contain [REDACTED]"
        );
    }

    // ---- Integrity checker tests ----

    #[test]
    fn test_integrity_compute_file_hash() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test_hash.txt");

        let mut f = fs::File::create(&file_path).expect("Failed to create file");
        f.write_all(b"hello world").expect("Failed to write");

        let hash = IntegrityChecker::compute_file_hash(&file_path)
            .expect("compute_file_hash should succeed");

        // SHA-256 of "hello world" is well-known
        assert_eq!(
            hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            "SHA-256 hash should match expected value"
        );
    }

    #[test]
    fn test_integrity_verify_file_hash() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("verify.txt");

        let mut f = fs::File::create(&file_path).expect("Failed to create file");
        f.write_all(b"test content").expect("Failed to write");

        let expected = IntegrityChecker::compute_file_hash(&file_path)
            .expect("compute_file_hash should succeed");

        // Correct hash
        assert!(
            IntegrityChecker::verify_file_hash(&file_path, &expected)
                .expect("verify should succeed"),
            "Hash verification should succeed with correct hash"
        );

        // Incorrect hash
        assert!(
            !IntegrityChecker::verify_file_hash(&file_path, "0000000000000000")
                .expect("verify should succeed"),
            "Hash verification should fail with incorrect hash"
        );
    }
}
