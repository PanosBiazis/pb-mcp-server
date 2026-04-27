//! File system argument structures
//!
//! This module contains argument structures for file system operations like
//! reading files, listing directories, searching, and getting file information.

use super::defaults;
use schemars::JsonSchema;
use serde::Deserialize;

// ============================================================================
// Read File Operations
// ============================================================================

/// Arguments for reading a file
#[derive(Debug, Clone, Deserialize, JsonSchema, Default)]
pub struct ReadFileArgs {
    /// UTF-8 text file path under the project root
    #[serde(default)]
    pub path: String,
    /// 1-based start line (inclusive) for a numbered slice; omit for whole file
    #[serde(default)]
    pub start_line: Option<usize>,
    /// 1-based end line (inclusive). With start_line only, defaults to end of loaded content
    #[serde(default)]
    pub end_line: Option<usize>,
}

impl From<&str> for ReadFileArgs {
    fn from(s: &str) -> Self {
        ReadFileArgs {
            path: s.to_string(),
            start_line: None,
            end_line: None,
        }
    }
}

/// Arguments for reading the last N lines of a file
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadFileTailArgs {
    /// UTF-8 text file path under the project root
    pub path: String,

    /// Number of lines from the end (default 100, max 500)
    #[serde(default = "default_tail_lines")]
    pub n: usize,
}

fn default_tail_lines() -> usize {
    100
}

impl Default for ReadFileTailArgs {
    fn default() -> Self {
        ReadFileTailArgs {
            path: String::new(),
            n: default_tail_lines(),
        }
    }
}

/// Arguments for reading a JSON file
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReadJsonFileArgs {
    /// JSON file path under the project root
    pub path: String,

    /// Whether to pretty-print the JSON output
    #[serde(default)]
    pub pretty: bool,
}

// ============================================================================
// List/Search Operations
// ============================================================================

/// Arguments for listing directory contents
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListFilesArgs {
    /// Directory to list; empty = project root
    #[serde(default)]
    pub path: String,

    /// Maximum entries to return (server caps at 200)
    #[serde(default = "defaults::default_find_max")]
    pub max_entries: usize,

    /// Whether to include hidden files (starting with .)
    #[serde(default = "defaults::default_true")]
    pub include_hidden: bool,
}

impl Default for ListFilesArgs {
    fn default() -> Self {
        ListFilesArgs {
            path: String::new(),
            max_entries: defaults::default_find_max(),
            include_hidden: true,
        }
    }
}

/// Arguments for finding files by glob pattern
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindFilesArgs {
    /// Glob relative to search root, e.g. **/*.rs or src/**/*.ts
    pub glob: String,

    /// Subtree to search; empty = project root
    #[serde(default)]
    pub path: String,

    /// Max paths returned (server caps at 600)
    #[serde(default = "defaults::default_find_max")]
    pub max_files: usize,
}

impl Default for FindFilesArgs {
    fn default() -> Self {
        FindFilesArgs {
            glob: "**/*".to_string(),
            path: String::new(),
            max_files: defaults::default_find_max(),
        }
    }
}

/// Arguments for finding files by name substring
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindFilesByNameArgs {
    /// Substring to match in filename (case-insensitive)
    pub name: String,

    /// Subtree to search; empty = project root
    #[serde(default)]
    pub path: String,

    /// Max paths returned
    #[serde(default = "defaults::default_find_max")]
    pub max_files: usize,
}

/// Arguments for grep/search in files
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GrepArgs {
    /// Regex pattern unless literal=true (then matched as plain text)
    pub pattern: String,

    /// Directory or file subtree to search; empty = project root
    #[serde(default)]
    pub path: String,

    /// Stop after this many matches (server caps at 200)
    #[serde(default = "defaults::default_grep_max")]
    pub max_matches: usize,

    /// Case-sensitive matching
    #[serde(default = "defaults::default_true")]
    pub case_sensitive: bool,

    /// Treat pattern as literal string, not regex
    #[serde(default)]
    pub literal: bool,

    /// Respect .gitignore rules
    #[serde(default = "defaults::default_true")]
    pub respect_gitignore: bool,

    /// Include pattern for files to search (e.g., **/*.rs)
    #[serde(default)]
    pub include_pattern: Option<String>,
}

impl Default for GrepArgs {
    fn default() -> Self {
        GrepArgs {
            pattern: String::new(),
            path: String::new(),
            max_matches: defaults::default_grep_max(),
            case_sensitive: true,
            literal: false,
            respect_gitignore: true,
            include_pattern: None,
        }
    }
}

/// Arguments for directory tree listing
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TreeArgs {
    /// Directory to show; empty = project root
    #[serde(default)]
    pub path: String,

    /// Depth from the given directory (clamped 1–12)
    #[serde(default = "defaults::default_tree_depth")]
    pub max_depth: usize,

    /// Maximum entries to return (server caps at 500)
    #[serde(default = "default_tree_max_entries")]
    pub max_entries: usize,
}

fn default_tree_max_entries() -> usize {
    500
}

impl Default for TreeArgs {
    fn default() -> Self {
        TreeArgs {
            path: String::new(),
            max_depth: defaults::default_tree_depth(),
            max_entries: default_tree_max_entries(),
        }
    }
}

// ============================================================================
// File Information Operations
// ============================================================================

/// Arguments for getting file/directory information
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FileInfoArgs {
    /// File or directory path under the project root
    pub path: String,
}

impl From<&str> for FileInfoArgs {
    fn from(s: &str) -> Self {
        FileInfoArgs {
            path: s.to_string(),
        }
    }
}

/// Arguments for getting file statistics
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FileStatsArgs {
    /// File path under the project root
    pub path: String,

    /// Whether to count words
    #[serde(default = "defaults::default_true")]
    pub count_words: bool,

    /// Whether to count characters
    #[serde(default = "defaults::default_true")]
    pub count_chars: bool,
}

/// Arguments for batch file information
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BatchFileInfoArgs {
    /// List of file paths to get information for (max 30)
    pub paths: Vec<String>,

    /// Whether to include size in bytes
    #[serde(default = "defaults::default_true")]
    pub include_size: bool,

    /// Whether to include modification time
    #[serde(default = "defaults::default_true")]
    pub include_mtime: bool,
}

impl Default for BatchFileInfoArgs {
    fn default() -> Self {
        BatchFileInfoArgs {
            paths: Vec::new(),
            include_size: true,
            include_mtime: true,
        }
    }
}

// ============================================================================
// Temporary File Operations
// ============================================================================

/// Arguments for writing to a temporary file
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TempWriteArgs {
    /// Content to write
    pub content: String,

    /// Optional filename suffix (e.g., ".txt")
    #[serde(default)]
    pub suffix: Option<String>,

    /// Optional filename prefix
    #[serde(default)]
    pub prefix: Option<String>,
}

/// Arguments for reading a temporary file
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TempReadArgs {
    /// Temporary file name (basename only)
    pub name: String,
}

/// Arguments for listing temporary files
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TempListArgs {
    /// Maximum files to list
    #[serde(default = "defaults::default_find_max")]
    pub limit: usize,
}

impl Default for TempListArgs {
    fn default() -> Self {
        TempListArgs {
            limit: defaults::default_find_max(),
        }
    }
}

/// Arguments for deleting a temporary file
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TempDeleteArgs {
    /// Temporary file name (basename only)
    pub name: String,
}

// ============================================================================
// Workspace Context
// ============================================================================

/// Arguments for workspace context (orientation)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WorkspaceContextArgs {
    /// Whether to include git status summary
    #[serde(default = "defaults::default_true")]
    pub include_git: bool,

    /// Whether to include file tree overview
    #[serde(default)]
    pub include_tree: bool,

    /// Tree depth if included
    #[serde(default = "defaults::default_tree_depth")]
    pub tree_depth: usize,
}

impl Default for WorkspaceContextArgs {
    fn default() -> Self {
        WorkspaceContextArgs {
            include_git: true,
            include_tree: false,
            tree_depth: defaults::default_tree_depth(),
        }
    }
}

// ============================================================================
// Pack Context
// ============================================================================

/// Single segment in a pack_context request
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ContextSegment {
    /// Label for this segment
    pub label: String,

    /// Content to include
    pub content: String,

    /// Optional compression to apply
    #[serde(default)]
    pub compress: Option<String>,
}

/// Arguments for packing multiple context segments
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PackContextArgs {
    /// Segments to pack
    pub segments: Vec<ContextSegment>,

    /// Overall maximum characters
    #[serde(default = "defaults::default_max_response_chars")]
    pub max_chars: usize,
}

impl Default for PackContextArgs {
    fn default() -> Self {
        PackContextArgs {
            segments: Vec::new(),
            max_chars: defaults::default_max_response_chars(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_args_default() {
        let args = ReadFileArgs::default();
        assert!(args.path.is_empty());
        assert!(args.start_line.is_none());
        assert!(args.end_line.is_none());
    }

    #[test]
    fn test_read_file_args_from_str() {
        let args = ReadFileArgs::from("test.txt");
        assert_eq!(args.path, "test.txt");
    }

    #[test]
    fn test_list_files_args_default() {
        let args = ListFilesArgs::default();
        assert!(args.path.is_empty());
        assert!(args.include_hidden);
    }

    #[test]
    fn test_grep_args_default() {
        let args = GrepArgs::default();
        assert!(args.pattern.is_empty());
        assert!(args.case_sensitive);
        assert!(args.respect_gitignore);
        assert!(!args.literal);
    }

    #[test]
    fn test_tree_args_default() {
        let args = TreeArgs::default();
        assert!(args.path.is_empty());
        assert_eq!(args.max_depth, 3);
    }

    #[test]
    fn test_file_info_args_from_str() {
        let args = FileInfoArgs::from("/path/to/file.txt");
        assert_eq!(args.path, "/path/to/file.txt");
    }

    #[test]
    fn test_workspace_context_default() {
        let args = WorkspaceContextArgs::default();
        assert!(args.include_git);
        assert!(!args.include_tree);
    }
}
