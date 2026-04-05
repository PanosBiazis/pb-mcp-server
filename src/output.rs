//! Output formatting utilities for pb-mcp-server
//!
//! This module provides standardized output formatting for MCP tool responses.
//! All tools should use these utilities to ensure consistent, predictable output
//! that is easy for AI agents to parse and understand.
//!
//! # Design Principles
//!
//! 1. **Structured output**: Every tool returns a predictable envelope
//! 2. **Truncation handling**: Clear indication when output is truncated
//! 3. **Metadata inclusion**: Size, timing, and status information
//! 4. **Error formatting**: Consistent error messages with context
//!
//! # Output Envelope
//!
//! All tools return output in this format:
//!
//! ```text
//! STATUS: ok | error | truncated
//! SUMMARY: One-line description
//!
//! DATA:
//! [actual content]
//!
//! METADATA:
//! - bytes: 1234
//! - truncated: false
//! - next_suggested_tools: [tool1, tool2]
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Maximum characters in a response before truncation
pub const MAX_RESPONSE_CHARS: usize = 20_000;

/// Maximum bytes that can be read from a file
pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024;

/// Maximum terminal stdin bytes
pub const MAX_TERMINAL_STDIN_BYTES: usize = 512 * 1024;

/// Maximum matches for grep operations
pub const MAX_GREP_MATCHES: usize = 200;

/// Maximum files for find operations
pub const MAX_FIND_FILES: usize = 600;

/// Maximum entries for directory listing
pub const MAX_DIR_ENTRIES: usize = 200;

/// Maximum entries for tree operations
pub const MAX_TREE_ENTRIES: usize = 500;

/// Output status indicating success, error, or truncation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStatus {
    /// Operation completed successfully
    Ok,
    /// Operation failed with an error
    Error,
    /// Operation succeeded but output was truncated
    Truncated,
    /// Operation completed with warnings
    Warning,
    /// Operation is pending (for async operations)
    Pending,
}

impl Default for OutputStatus {
    fn default() -> Self {
        OutputStatus::Ok
    }
}

impl std::fmt::Display for OutputStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputStatus::Ok => write!(f, "ok"),
            OutputStatus::Error => write!(f, "error"),
            OutputStatus::Truncated => write!(f, "truncated"),
            OutputStatus::Warning => write!(f, "warning"),
            OutputStatus::Pending => write!(f, "pending"),
        }
    }
}

/// Standard output envelope for all MCP tool responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEnvelope {
    /// Status of the operation
    pub status: OutputStatus,

    /// One-line summary of what was done
    pub summary: String,

    /// The actual content/data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Error message if status is error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Warning messages
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,

    /// Metadata about the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OutputMetadata>,

    /// Suggested tools to use next
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub next_suggested_tools: Vec<String>,
}

/// Metadata about the output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    /// Size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,

    /// Number of lines (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<usize>,

    /// Whether output was truncated
    pub truncated: bool,

    /// Original size before truncation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_size: Option<usize>,

    /// Execution time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Number of items (for lists)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_count: Option<usize>,

    /// File path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Additional custom metadata
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub extra: std::collections::HashMap<String, String>,
}

impl Default for OutputMetadata {
    fn default() -> Self {
        OutputMetadata {
            bytes: None,
            lines: None,
            truncated: false,
            original_size: None,
            duration_ms: None,
            item_count: None,
            path: None,
            extra: std::collections::HashMap::new(),
        }
    }
}

impl OutputEnvelope {
    /// Create a successful output envelope
    pub fn ok(summary: impl Into<String>, data: impl Into<String>) -> Self {
        OutputEnvelope {
            status: OutputStatus::Ok,
            summary: summary.into(),
            data: Some(data.into()),
            error: None,
            warnings: Vec::new(),
            metadata: None,
            next_suggested_tools: Vec::new(),
        }
    }

    /// Create an error output envelope
    pub fn error(summary: impl Into<String>, error: impl Into<String>) -> Self {
        OutputEnvelope {
            status: OutputStatus::Error,
            summary: summary.into(),
            data: None,
            error: Some(error.into()),
            warnings: Vec::new(),
            metadata: None,
            next_suggested_tools: Vec::new(),
        }
    }

    /// Create a truncated output envelope
    pub fn truncated(
        summary: impl Into<String>,
        data: impl Into<String>,
        original_size: usize,
    ) -> Self {
        OutputEnvelope {
            status: OutputStatus::Truncated,
            summary: summary.into(),
            data: Some(data.into()),
            error: None,
            warnings: vec![format!(
                "Output truncated from {} to {} characters",
                original_size, MAX_RESPONSE_CHARS
            )],
            metadata: Some(OutputMetadata {
                truncated: true,
                original_size: Some(original_size),
                ..Default::default()
            }),
            next_suggested_tools: Vec::new(),
        }
    }

    /// Add metadata to the envelope
    pub fn with_metadata(mut self, metadata: OutputMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add a warning to the envelope
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        if self.status == OutputStatus::Ok {
            self.status = OutputStatus::Warning;
        }
        self
    }

    /// Add suggested tools
    pub fn with_suggested_tools(mut self, tools: Vec<&str>) -> Self {
        self.next_suggested_tools = tools.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add execution timing
    pub fn with_timing(mut self, duration: Duration) -> Self {
        let metadata = self.metadata.get_or_insert_with(OutputMetadata::default);
        metadata.duration_ms = Some(duration.as_millis() as u64);
        self
    }

    /// Convert to formatted string for MCP response
    pub fn to_mcp_response(&self) -> String {
        let mut output = String::new();

        // Status line
        output.push_str(&format!("STATUS: {}\n", self.status));

        // Summary
        output.push_str(&format!("SUMMARY: {}\n", self.summary));

        // Error if present
        if let Some(ref error) = self.error {
            output.push_str(&format!("ERROR: {}\n", error));
        }

        output.push('\n');

        // Data section
        if let Some(ref data) = self.data {
            output.push_str("```\n");
            output.push_str(data);
            output.push_str("\n```\n");
        }

        // Warnings
        if !self.warnings.is_empty() {
            output.push_str("\nWARNINGS:\n");
            for warning in &self.warnings {
                output.push_str(&format!("- {}\n", warning));
            }
        }

        // Metadata
        if let Some(ref metadata) = self.metadata {
            output.push_str("\nMETADATA:\n");
            if let Some(bytes) = metadata.bytes {
                output.push_str(&format!("- bytes: {}\n", bytes));
            }
            if let Some(lines) = metadata.lines {
                output.push_str(&format!("- lines: {}\n", lines));
            }
            if metadata.truncated {
                output.push_str("- truncated: true\n");
            }
            if let Some(orig) = metadata.original_size {
                output.push_str(&format!("- original_size: {}\n", orig));
            }
            if let Some(duration) = metadata.duration_ms {
                output.push_str(&format!("- duration_ms: {}\n", duration));
            }
            if let Some(count) = metadata.item_count {
                output.push_str(&format!("- item_count: {}\n", count));
            }
            if let Some(ref path) = metadata.path {
                output.push_str(&format!("- path: {}\n", path));
            }
            for (key, value) in &metadata.extra {
                output.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        // Suggested tools
        if !self.next_suggested_tools.is_empty() {
            output.push_str("\nNEXT_SUGGESTED_TOOLS:\n");
            for tool in &self.next_suggested_tools {
                output.push_str(&format!("- {}\n", tool));
            }
        }

        output
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ============================================================================
// Helper functions for common output patterns
// ============================================================================

/// Truncate a string to maximum characters, returning (truncated_string, was_truncated)
pub fn truncate_chars(s: &str, max: usize) -> (String, bool) {
    if s.chars().count() <= max {
        return (s.to_string(), false);
    }

    let truncated: String = s.chars().take(max).collect();
    (truncated, true)
}

/// Truncate with ellipsis
pub fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }

    if max <= 3 {
        return "...".chars().take(max).collect();
    }

    let truncated: String = s.chars().take(max - 3).collect();
    format!("{}...", truncated)
}

/// Format a file path with line numbers
pub fn format_file_with_lines(path: &str, content: &str, start_line: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut output = format!("FILE: {}\n\n", path);

    for (i, line) in lines.iter().enumerate() {
        let line_num = start_line + i;
        output.push_str(&format!("{:6}| {}\n", line_num, line));
    }

    output
}

/// Format a list of items with optional numbering
pub fn format_list(items: &[String], numbered: bool) -> String {
    if items.is_empty() {
        return "(empty)".to_string();
    }

    let mut output = String::new();
    for (i, item) in items.iter().enumerate() {
        if numbered {
            output.push_str(&format!("{}. {}\n", i + 1, item));
        } else {
            output.push_str(&format!("- {}\n", item));
        }
    }

    output
}

/// Format a key-value section
pub fn format_kv_section(title: &str, items: &[(String, String)]) -> String {
    let mut output = format!("{}\n", title);
    output.push_str(&"=".repeat(title.len()));
    output.push('\n');

    for (key, value) in items {
        output.push_str(&format!("{}: {}\n", key, value));
    }

    output
}

/// Format a table of data
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(no data)".to_string();
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Add some padding
    for width in &mut widths {
        *width += 2;
    }

    let mut output = String::new();

    // Header row
    for (i, header) in headers.iter().enumerate() {
        output.push_str(&format!(
            " {:width$}",
            header,
            width = widths.get(i).unwrap_or(&10)
        ));
    }
    output.push('\n');

    // Separator
    for width in &widths {
        output.push_str(&format!("+{}", "-".repeat(*width)));
    }
    output.push_str("+\n");

    // Data rows
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            output.push_str(&format!(
                " {:width$}",
                cell,
                width = widths.get(i).unwrap_or(&10)
            ));
        }
        output.push('\n');
    }

    output
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human-readable time
pub fn format_duration(duration: Duration) -> String {
    let ms = duration.as_millis();

    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}

/// Format a count with singular/plural
pub fn format_count(count: usize, singular: &str, plural: Option<&str>) -> String {
    if count == 1 {
        format!("{} {}", count, singular)
    } else {
        match plural {
            Some(p) => format!("{} {}", count, p),
            None => format!("{} {}s", count, singular),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_envelope_ok() {
        let envelope = OutputEnvelope::ok("Test summary", "Test data");
        assert_eq!(envelope.status, OutputStatus::Ok);
        assert_eq!(envelope.summary, "Test summary");
        assert!(envelope.data.is_some());
        assert!(envelope.error.is_none());
    }

    #[test]
    fn test_output_envelope_error() {
        let envelope = OutputEnvelope::error("Error occurred", "Something went wrong");
        assert_eq!(envelope.status, OutputStatus::Error);
        assert!(envelope.error.is_some());
        assert!(envelope.data.is_none());
    }

    #[test]
    fn test_output_envelope_with_metadata() {
        let metadata = OutputMetadata {
            bytes: Some(1000),
            lines: Some(50),
            truncated: false,
            ..Default::default()
        };

        let envelope = OutputEnvelope::ok("Test", "Data").with_metadata(metadata);

        assert!(envelope.metadata.is_some());
        assert_eq!(envelope.metadata.as_ref().unwrap().bytes, Some(1000));
    }

    #[test]
    fn test_truncate_chars() {
        let s = "Hello, world!";
        let (truncated, was_truncated) = truncate_chars(s, 5);

        assert_eq!(truncated, "Hello");
        assert!(was_truncated);

        let (not_truncated, was_not) = truncate_chars(s, 100);
        assert_eq!(not_truncated, s);
        assert!(!was_not);
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        let s = "Hello, world!";
        let truncated = truncate_with_ellipsis(s, 8);

        assert_eq!(truncated, "Hello...");
        assert!(truncated.len() <= 8);
    }

    #[test]
    fn test_format_list() {
        let items = vec!["one".to_string(), "two".to_string(), "three".to_string()];

        let numbered = format_list(&items, true);
        assert!(numbered.contains("1. one"));
        assert!(numbered.contains("2. two"));

        let bulleted = format_list(&items, false);
        assert!(bulleted.contains("- one"));
        assert!(bulleted.contains("- two"));
    }

    #[test]
    fn test_format_table() {
        let headers = ["Name", "Value"];
        let rows = vec![
            vec!["foo".to_string(), "bar".to_string()],
            vec!["baz".to_string(), "qux".to_string()],
        ];

        let table = format_table(&headers, &rows);
        assert!(table.contains("Name"));
        assert!(table.contains("Value"));
        assert!(table.contains("foo"));
        assert!(table.contains("bar"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
    }

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(1, "file", None), "1 file");
        assert_eq!(format_count(2, "file", None), "2 files");
        assert_eq!(format_count(2, "child", Some("children")), "2 children");
    }

    #[test]
    fn test_output_status_display() {
        assert_eq!(format!("{}", OutputStatus::Ok), "ok");
        assert_eq!(format!("{}", OutputStatus::Error), "error");
        assert_eq!(format!("{}", OutputStatus::Truncated), "truncated");
    }

    #[test]
    fn test_envelope_to_mcp_response() {
        let envelope = OutputEnvelope::ok("Test summary", "Test data").with_warning("Test warning");

        let response = envelope.to_mcp_response();
        assert!(response.contains("STATUS: warning"));
        assert!(response.contains("SUMMARY: Test summary"));
        assert!(response.contains("Test data"));
        assert!(response.contains("WARNINGS:"));
    }

    #[test]
    fn test_envelope_json_serialization() {
        let envelope = OutputEnvelope::ok("Test", "Data");
        let json = envelope.to_json().unwrap();

        assert!(json.contains("\"status\":"));
        assert!(json.contains("\"summary\":"));

        let parsed = OutputEnvelope::from_json(&json).unwrap();
        assert_eq!(parsed.summary, "Test");
    }
}
