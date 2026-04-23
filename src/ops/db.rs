//! # Database Tools
//!
//! SQLite query/exec/schema via `sqlite3` CLI, and dot-notation JSON queries.
//!
//! Four public work functions:
//! - `db_sqlite_query_work`  — read-only SELECT queries via `sqlite3 -json -header`
//! - `db_sqlite_exec_work`   — write statements via `sqlite3` CLI (with dry-run / EXPLAIN)
//! - `db_sqlite_schema_work` — schema dump via `sqlite3 .schema` / `.tables`
//! - `db_json_query_work`    — dot-notation JSON file queries using `serde_json`

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum characters for database tool responses before truncation.
const MAX_DB_RESPONSE_CHARS: usize = 20_000;

/// Maximum bytes for a JSON file that will be read by `db_json_query_work`.
const MAX_JSON_QUERY_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// Default row limit for `db_sqlite_query_work` when `max_rows` is `None`.
const DEFAULT_MAX_ROWS: usize = 100;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a string to `max` chars, appending a notice if truncated.
fn truncate_response(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max).collect();
    format!("{truncated}\n\n... [response truncated at {max} characters]",)
}

/// Check whether the `sqlite3` CLI is available on `$PATH`.
fn sqlite3_available() -> bool {
    Command::new("sqlite3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Validate that `db_path` points to an existing file.
fn validate_db_path(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Err(format!(
            "Error: database file not found: {}",
            db_path.display()
        ));
    }
    if !db_path.is_file() {
        return Err(format!("Error: path is not a file: {}", db_path.display()));
    }
    Ok(())
}

/// Run `sqlite3` with the given arguments and return (stdout, stderr).
#[allow(dead_code)]
fn run_sqlite3(db_path: &Path, args: &[&str]) -> Result<(String, String), String> {
    if !sqlite3_available() {
        return Err(
            "Error: `sqlite3` CLI not found on PATH. Install sqlite3 to use database tools."
                .to_string(),
        );
    }
    validate_db_path(db_path)?;

    let output = Command::new("sqlite3")
        .arg(db_path)
        .args(args)
        .output()
        .map_err(|e| format!("Error: failed to execute sqlite3: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((stdout, stderr))
}

// ---------------------------------------------------------------------------
// 1. db_sqlite_query_work
// ---------------------------------------------------------------------------

/// Execute a read-only SQLite query via `sqlite3 -json -header`.
///
/// Uses `sqlite3 -json -header <db> <query>` to return results as a JSON
/// array of objects. If `sqlite3` is not found, returns an error message.
/// Results are truncated to ~20 000 characters. When `max_rows` is `None`,
/// a default of 100 rows is used.
///
/// # Arguments
/// * `db_path`  — path to the `.sqlite` / `.db` file
/// * `query`    — SQL query (should be a SELECT / read-only statement)
/// * `max_rows` — optional row cap (defaults to 100)
pub fn db_sqlite_query_work(db_path: PathBuf, query: String, max_rows: Option<usize>) -> String {
    let max_rows = max_rows.unwrap_or(DEFAULT_MAX_ROWS).max(1);

    let mut out = String::from("DB: SQLITE QUERY\n\n");

    if let Err(e) = validate_db_path(&db_path) {
        out.push_str(&e);
        return out;
    }

    if !sqlite3_available() {
        out.push_str(
            "Error: `sqlite3` CLI not found on PATH. Install sqlite3 to use database tools.",
        );
        return out;
    }

    // Run the query via sqlite3 -json -header
    let output = match Command::new("sqlite3")
        .args(["-json", "-header"])
        .arg(&db_path)
        .arg(&query)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            out.push_str(&format!("Error: failed to execute sqlite3: {e}"));
            return out;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // If sqlite3 exited with a non-zero status and there's no useful stdout,
    // report the error.
    if !output.status.success() && stdout.trim().is_empty() {
        out.push_str(&format!(
            "Query: {query}\nStatus: error (exit code {:?})\n",
            output.status.code()
        ));
        if !stderr.trim().is_empty() {
            out.push_str(&format!("Error: {}", stderr.trim()));
        } else {
            out.push_str("Error: sqlite3 returned non-zero exit status with no output");
        }
        return truncate_response(&out, MAX_DB_RESPONSE_CHARS);
    }

    out.push_str(&format!("Query: {query}\n"));
    out.push_str(&format!("Database: {}\n\n", db_path.display()));

    // If we got JSON output, parse it to enforce max_rows
    if !stdout.trim().is_empty() {
        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(serde_json::Value::Array(arr)) => {
                let total = arr.len();
                let limited: Vec<&serde_json::Value> = arr.iter().take(max_rows).collect();
                let limited_json =
                    serde_json::to_string_pretty(&limited).unwrap_or_else(|_| stdout.clone());
                out.push_str(&format!("Rows returned: {total}\n"));
                if total > max_rows {
                    out.push_str(&format!(
                        "Rows shown: {max_rows} (use max_rows to see more)\n"
                    ));
                }
                out.push('\n');
                out.push_str(&limited_json);
                out.push('\n');
            }
            Ok(_) => {
                // Not an array — just emit as-is
                out.push_str(&stdout);
            }
            Err(_) => {
                // Not valid JSON — could be empty result or plain-text output
                out.push_str(&stdout);
            }
        }
    } else {
        out.push_str("(no rows returned)\n");
    }

    // Surface stderr warnings if any
    if !stderr.trim().is_empty() {
        out.push_str(&format!("\nWarnings: {}", stderr.trim()));
    }

    truncate_response(&out, MAX_DB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 2. db_sqlite_exec_work
// ---------------------------------------------------------------------------

/// Execute a write SQLite statement via `sqlite3` CLI.
///
/// If `dry_run` is `true`, the statement is prefixed with `EXPLAIN` so
/// only the query plan is returned — no data is modified.
/// If `sqlite3` is not found, returns an error message.
///
/// # Arguments
/// * `db_path`   — path to the `.sqlite` / `.db` file
/// * `statement` — SQL statement (INSERT, UPDATE, DELETE, CREATE, etc.)
/// * `dry_run`   — if true, prefix with EXPLAIN and return the plan
pub fn db_sqlite_exec_work(db_path: PathBuf, statement: String, dry_run: bool) -> String {
    let mut out = String::from("DB: SQLITE EXEC\n\n");

    if let Err(e) = validate_db_path(&db_path) {
        out.push_str(&e);
        return out;
    }

    if !sqlite3_available() {
        out.push_str(
            "Error: `sqlite3` CLI not found on PATH. Install sqlite3 to use database tools.",
        );
        return out;
    }

    let actual_statement = if dry_run {
        format!("EXPLAIN {statement}")
    } else {
        statement.clone()
    };

    out.push_str(&format!(
        "Database: {}\nMode: {}\nStatement: {}\n\n",
        db_path.display(),
        if dry_run {
            "DRY RUN (EXPLAIN)"
        } else {
            "EXECUTE"
        },
        statement,
    ));

    let output = match Command::new("sqlite3")
        .arg(&db_path)
        .arg(&actual_statement)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            out.push_str(&format!("Error: failed to execute sqlite3: {e}"));
            return truncate_response(&out, MAX_DB_RESPONSE_CHARS);
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        out.push_str(&format!(
            "Status: error (exit code {:?})\n",
            output.status.code()
        ));
        if !stderr.trim().is_empty() {
            out.push_str(&format!("Error: {}", stderr.trim()));
        }
        return truncate_response(&out, MAX_DB_RESPONSE_CHARS);
    }

    out.push_str("Status: ok\n");

    if dry_run {
        out.push_str("\n=== Query Plan ===\n");
    }

    if !stdout.trim().is_empty() {
        out.push_str(&stdout);
        if !out.ends_with('\n') {
            out.push('\n');
        }
    } else if !dry_run {
        out.push_str("(statement executed — no output returned)\n");
    }

    if !stderr.trim().is_empty() {
        out.push_str(&format!("\nWarnings: {}", stderr.trim()));
    }

    truncate_response(&out, MAX_DB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 3. db_sqlite_schema_work
// ---------------------------------------------------------------------------

/// Dump SQLite schema via `sqlite3 .schema` and list tables via `.tables`.
///
/// If `table_name` is provided, only the schema for that table is shown
/// (via `.schema <table>`).
///
/// # Arguments
/// * `db_path`    — path to the `.sqlite` / `.db` file
/// * `table_name` — optional table name to filter `.schema` output
pub fn db_sqlite_schema_work(db_path: PathBuf, table_name: Option<String>) -> String {
    let mut out = String::from("DB: SQLITE SCHEMA\n\n");

    if let Err(e) = validate_db_path(&db_path) {
        out.push_str(&e);
        return out;
    }

    if !sqlite3_available() {
        out.push_str(
            "Error: `sqlite3` CLI not found on PATH. Install sqlite3 to use database tools.",
        );
        return out;
    }

    out.push_str(&format!("Database: {}\n\n", db_path.display()));

    // --- .tables ---
    let tables_cmd = Command::new("sqlite3")
        .arg(&db_path)
        .arg(".tables")
        .output();

    match tables_cmd {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            out.push_str("=== Tables ===\n");
            if !stdout.trim().is_empty() {
                out.push_str(&stdout);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            } else {
                out.push_str("(no tables found)\n");
            }
            if !stderr.trim().is_empty() {
                out.push_str(&format!("Error: {}\n", stderr.trim()));
            }
        }
        Err(e) => {
            out.push_str(&format!("Error running .tables: {e}\n"));
        }
    }

    out.push('\n');

    // --- .schema [table_name] ---
    let schema_arg = match &table_name {
        Some(name) => format!(".schema {name}"),
        None => ".schema".to_string(),
    };

    let schema_cmd = Command::new("sqlite3")
        .arg(&db_path)
        .arg(&schema_arg)
        .output();

    match schema_cmd {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            out.push_str("=== Schema ===\n");
            if !stdout.trim().is_empty() {
                out.push_str(&stdout);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
            } else {
                if let Some(ref name) = table_name {
                    out.push_str(&format!("(no schema found for table: {name})\n"));
                } else {
                    out.push_str("(no schema found)\n");
                }
            }
            if !stderr.trim().is_empty() {
                out.push_str(&format!("Error: {}\n", stderr.trim()));
            }
        }
        Err(e) => {
            out.push_str(&format!("Error running .schema: {e}\n"));
        }
    }

    truncate_response(&out, MAX_DB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 4. db_json_query_work
// ---------------------------------------------------------------------------

/// Query a JSON file using dot-notation path navigation.
///
/// Reads the file, parses it as JSON, then navigates using the dot-path.
/// Array indices are supported via numeric segments (e.g. `"users.0.name"`).
///
/// This function does **not** use the `sqlite3` CLI — it parses JSON
/// directly with `serde_json`.
///
/// # Arguments
/// * `json_path` — path to the JSON file
/// * `dot_path`  — dot-notation path (e.g. `"users.0.name"`)
/// * `pretty`    — if true, pretty-print the resulting value
///
/// # Limits
/// - Maximum file size: 10 MB
/// - Response truncated to 20 000 chars
pub fn db_json_query_work(json_path: PathBuf, dot_path: String, pretty: bool) -> String {
    let mut out = String::from("DB: JSON QUERY\n\n");

    // --- Validate file ---
    let meta = match fs::metadata(&json_path) {
        Ok(m) => m,
        Err(e) => {
            out.push_str(&format!(
                "Error: file not found: {} ({e})",
                json_path.display()
            ));
            return out;
        }
    };

    if !meta.is_file() {
        out.push_str(&format!(
            "Error: path is not a file: {}",
            json_path.display()
        ));
        return out;
    }

    if meta.len() > MAX_JSON_QUERY_BYTES {
        out.push_str(&format!(
            "Error: file exceeds maximum size of {} bytes (got {} bytes)",
            MAX_JSON_QUERY_BYTES,
            meta.len()
        ));
        return out;
    }

    // --- Read file ---
    let mut file = match fs::File::open(&json_path) {
        Ok(f) => f,
        Err(e) => {
            out.push_str(&format!("Error: could not open file: {e}"));
            return out;
        }
    };

    let mut buf = Vec::with_capacity(meta.len() as usize);
    if let Err(e) = file.read_to_end(&mut buf) {
        out.push_str(&format!("Error: could not read file: {e}"));
        return out;
    }

    // --- Parse JSON ---
    let root: serde_json::Value = match serde_json::from_slice(&buf) {
        Ok(v) => v,
        Err(e) => {
            out.push_str(&format!("Error: invalid JSON: {e}"));
            return out;
        }
    };

    out.push_str(&format!("File: {}\n", json_path.display()));
    out.push_str(&format!("Path: {dot_path}\n\n"));

    // --- Navigate dot-path ---
    if dot_path.trim().is_empty() {
        // Empty path → return the whole document
        let rendered = if pretty {
            serde_json::to_string_pretty(&root)
                .unwrap_or_else(|e| format!("<serialize error: {e}>"))
        } else {
            serde_json::to_string(&root).unwrap_or_else(|e| format!("<serialize error: {e}>"))
        };
        out.push_str(&rendered);
        out.push('\n');
        return truncate_response(&out, MAX_DB_RESPONSE_CHARS);
    }

    let segments: Vec<&str> = dot_path.split('.').collect();
    let mut current = &root;

    for (i, segment) in segments.iter().enumerate() {
        if segment.is_empty() {
            // Handle double-dot or trailing dot gracefully
            out.push_str(&format!(
                "Error: empty segment at position {} in dot-path '{dot_path}'",
                i + 1
            ));
            return out;
        }

        match current {
            serde_json::Value::Object(map) => {
                if let Some(next) = map.get(*segment) {
                    current = next;
                } else {
                    out.push_str(&format!(
                        "Error: key '{segment}' not found at path '{}'",
                        segments[..=i].join(".")
                    ));
                    // Show available keys for helpful debugging
                    let keys: Vec<&String> = map.keys().collect();
                    if !keys.is_empty() {
                        out.push_str(&format!(
                            "\nAvailable keys at this level: [{}]",
                            keys.iter()
                                .map(|k| k.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    return out;
                }
            }
            serde_json::Value::Array(arr) => {
                // Try to interpret the segment as an array index
                match segment.parse::<usize>() {
                    Ok(idx) => {
                        if let Some(next) = arr.get(idx) {
                            current = next;
                        } else {
                            out.push_str(&format!(
                                "Error: index {idx} out of bounds at path '{}' (array length: {})",
                                segments[..=i].join("."),
                                arr.len()
                            ));
                            return out;
                        }
                    }
                    Err(_) => {
                        out.push_str(&format!(
                            "Error: expected numeric index for array at path '{}', got '{segment}' (array length: {})",
                            segments[..i].join("."),
                            arr.len()
                        ));
                        return out;
                    }
                }
            }
            serde_json::Value::Null => {
                out.push_str(&format!(
                    "Error: null value encountered at path '{}', cannot navigate further",
                    segments[..i].join(".")
                ));
                return out;
            }
            other => {
                let kind = json_value_kind(other);
                out.push_str(&format!(
                    "Error: cannot navigate into {kind} at path '{}'",
                    segments[..i].join(".")
                ));
                return out;
            }
        }
    }

    // --- Render result ---
    let kind = json_value_kind(current);
    out.push_str(&format!("Type: {kind}\n\n"));

    let rendered = if pretty {
        serde_json::to_string_pretty(current).unwrap_or_else(|e| format!("<serialize error: {e}>"))
    } else {
        serde_json::to_string(current).unwrap_or_else(|e| format!("<serialize error: {e}>"))
    };
    out.push_str(&rendered);
    out.push('\n');

    truncate_response(&out, MAX_DB_RESPONSE_CHARS)
}

/// Return a human-readable type name for a `serde_json::Value`.
fn json_value_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;

    // ---- Helper to create a temp SQLite database ----

    fn create_test_sqlite_db() -> tempfile::NamedTempFile {
        let db = tempfile::NamedTempFile::new().expect("create temp file");
        // Create a small SQLite database via the CLI (only works if sqlite3 is installed)
        let path = db.path().to_path_buf();
        let output = Command::new("sqlite3")
            .arg(&path)
            .arg(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT, value REAL); \
                 INSERT INTO items (name, value) VALUES ('alpha', 1.5); \
                 INSERT INTO items (name, value) VALUES ('beta', 2.5); \
                 INSERT INTO items (name, value) VALUES ('gamma', 3.5);",
            )
            .output();
        match output {
            Ok(o) if o.status.success() => db,
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                panic!("sqlite3 setup failed: {stderr}");
            }
            Err(e) => panic!("sqlite3 not available for test setup: {e}"),
        }
    }

    // ---- Helper to create a temp JSON file ----

    fn create_test_json_file(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create temp file");
        write!(f, "{content}").expect("write temp json");
        f
    }

    // =======================================================================
    // Test 1: sqlite3 query — basic SELECT
    // =======================================================================

    #[test]
    fn test_db_sqlite_query_work_basic() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_query_work(
            db.path().to_path_buf(),
            "SELECT * FROM items".to_string(),
            None,
        );

        assert!(
            result.starts_with("DB: SQLITE QUERY"),
            "unexpected prefix: {result}"
        );
        // Should contain the inserted data
        assert!(result.contains("alpha"), "missing 'alpha': {result}");
        assert!(result.contains("beta"), "missing 'beta': {result}");
        assert!(result.contains("gamma"), "missing 'gamma': {result}");
    }

    // =======================================================================
    // Test 2: sqlite3 query — file not found
    // =======================================================================

    #[test]
    fn test_db_sqlite_query_work_file_not_found() {
        let result = db_sqlite_query_work(
            PathBuf::from("/tmp/this_db_does_not_exist_abc123.db"),
            "SELECT 1".to_string(),
            None,
        );
        assert!(
            result.contains("Error"),
            "expected error for missing file: {result}"
        );
        assert!(
            result.contains("not found"),
            "expected 'not found': {result}"
        );
    }

    // =======================================================================
    // Test 3: sqlite3 query — max_rows limiting
    // =======================================================================

    #[test]
    fn test_db_sqlite_query_work_max_rows() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_query_work(
            db.path().to_path_buf(),
            "SELECT * FROM items".to_string(),
            Some(1),
        );
        assert!(
            result.contains("Rows returned:"),
            "missing row count: {result}"
        );
        // With max_rows=1, we should only show 1 row
        assert!(
            result.contains("Rows shown: 1"),
            "expected rows shown=1: {result}"
        );
    }

    // =======================================================================
    // Test 4: sqlite3 exec — dry run (EXPLAIN)
    // =======================================================================

    #[test]
    fn test_db_sqlite_exec_work_dry_run() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_exec_work(
            db.path().to_path_buf(),
            "INSERT INTO items (name, value) VALUES ('delta', 4.5)".to_string(),
            true,
        );
        assert!(
            result.contains("DRY RUN"),
            "missing DRY RUN indicator: {result}"
        );
        assert!(
            result.contains("Query Plan") || result.contains("EXPLAIN"),
            "expected query plan: {result}"
        );
    }

    // =======================================================================
    // Test 5: sqlite3 exec — actual write
    // =======================================================================

    #[test]
    fn test_db_sqlite_exec_work_execute() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_exec_work(
            db.path().to_path_buf(),
            "INSERT INTO items (name, value) VALUES ('delta', 4.5)".to_string(),
            false,
        );
        assert!(
            result.contains("EXECUTE"),
            "missing EXECUTE indicator: {result}"
        );
        assert!(
            result.contains("Status: ok"),
            "expected ok status: {result}"
        );

        // Verify the row was actually inserted
        let verify = db_sqlite_query_work(
            db.path().to_path_buf(),
            "SELECT name FROM items WHERE name = 'delta'".to_string(),
            None,
        );
        assert!(
            verify.contains("delta"),
            "row not found after insert: {verify}"
        );
    }

    // =======================================================================
    // Test 6: sqlite3 exec — file not found
    // =======================================================================

    #[test]
    fn test_db_sqlite_exec_work_file_not_found() {
        let result = db_sqlite_exec_work(
            PathBuf::from("/tmp/this_db_does_not_exist_abc123.db"),
            "CREATE TABLE t (id INT)".to_string(),
            false,
        );
        assert!(result.contains("Error"), "expected error: {result}");
    }

    // =======================================================================
    // Test 7: sqlite3 schema — basic
    // =======================================================================

    #[test]
    fn test_db_sqlite_schema_work_basic() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_schema_work(db.path().to_path_buf(), None);
        assert!(
            result.starts_with("DB: SQLITE SCHEMA"),
            "unexpected prefix: {result}"
        );
        assert!(
            result.contains("Tables"),
            "missing Tables section: {result}"
        );
        assert!(
            result.contains("Schema"),
            "missing Schema section: {result}"
        );
        assert!(result.contains("items"), "missing table name: {result}");
        assert!(
            result.contains("CREATE TABLE"),
            "missing CREATE TABLE: {result}"
        );
    }

    // =======================================================================
    // Test 8: sqlite3 schema — specific table
    // =======================================================================

    #[test]
    fn test_db_sqlite_schema_work_specific_table() {
        if !sqlite3_available() {
            eprintln!("skipping: sqlite3 not available");
            return;
        }
        let db = create_test_sqlite_db();
        let result = db_sqlite_schema_work(db.path().to_path_buf(), Some("items".to_string()));
        assert!(
            result.contains("CREATE TABLE"),
            "missing CREATE TABLE: {result}"
        );
        assert!(result.contains("items"), "missing items table: {result}");
    }

    // =======================================================================
    // Test 9: sqlite3 schema — file not found
    // =======================================================================

    #[test]
    fn test_db_sqlite_schema_work_file_not_found() {
        let result =
            db_sqlite_schema_work(PathBuf::from("/tmp/this_db_does_not_exist_abc123.db"), None);
        assert!(result.contains("Error"), "expected error: {result}");
    }

    // =======================================================================
    // Test 10: JSON query — simple object
    // =======================================================================

    #[test]
    fn test_db_json_query_work_simple_object() {
        let f = create_test_json_file(r#"{"name": "Alice", "age": 30}"#);
        let result = db_json_query_work(f.path().to_path_buf(), "name".to_string(), false);
        assert!(result.contains("Type: string"), "unexpected type: {result}");
        assert!(result.contains("Alice"), "missing value: {result}");
    }

    // =======================================================================
    // Test 11: JSON query — nested object
    // =======================================================================
    #[test]
    fn test_db_json_query_work_nested() {
        let f = create_test_json_file(r#"{"user": {"profile": {"name": "Bob", "score": 42}}}"#);
        let result = db_json_query_work(
            f.path().to_path_buf(),
            "user.profile.name".to_string(),
            false,
        );
        assert!(result.contains("Type: string"), "unexpected type: {result}");
        assert!(result.contains("Bob"), "missing value: {result}");
    }

    // =======================================================================
    // Test 12: JSON query — array index
    // =======================================================================
    #[test]
    fn test_db_json_query_work_array_index() {
        let f = create_test_json_file(r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#);
        let result = db_json_query_work(f.path().to_path_buf(), "users.1.name".to_string(), false);
        assert!(result.contains("Type: string"), "unexpected type: {result}");
        assert!(result.contains("Bob"), "missing value: {result}");
    }

    // =======================================================================
    // Test 13: JSON query — pretty-print
    // =======================================================================
    #[test]
    fn test_db_json_query_work_pretty() {
        let f = create_test_json_file(r#"{"items": [1, 2, 3]}"#);
        let result_pretty = db_json_query_work(f.path().to_path_buf(), "items".to_string(), true);
        assert!(
            result_pretty.contains("Type: array"),
            "unexpected type: {result_pretty}"
        );
        // Pretty output should contain newlines within the array
        assert!(
            result_pretty.contains("\n  ") || result_pretty.contains("[\n"),
            "expected pretty-printed output: {result_pretty}"
        );
    }

    // =======================================================================
    // Test 14: JSON query — empty path returns root
    // =======================================================================
    #[test]
    fn test_db_json_query_work_empty_path() {
        let f = create_test_json_file(r#"{"key": "value"}"#);
        let result = db_json_query_work(f.path().to_path_buf(), "".to_string(), false);
        assert!(result.contains("key"), "missing key: {result}");
        assert!(result.contains("value"), "missing value: {result}");
    }

    // =======================================================================
    // Test 15: JSON query — key not found
    // =======================================================================
    #[test]
    fn test_db_json_query_work_key_not_found() {
        let f = create_test_json_file(r#"{"name": "Alice"}"#);
        let result = db_json_query_work(f.path().to_path_buf(), "nonexistent".to_string(), false);
        assert!(result.contains("Error"), "expected error: {result}");
        assert!(
            result.contains("not found"),
            "expected 'not found': {result}"
        );
        // Should show available keys
        assert!(
            result.contains("Available keys"),
            "missing available keys hint: {result}"
        );
    }

    // =======================================================================
    // Test 16: JSON query — file not found
    // =======================================================================
    #[test]
    fn test_db_json_query_work_file_not_found() {
        let result = db_json_query_work(
            PathBuf::from("/tmp/this_json_does_not_exist_xyz.json"),
            "key".to_string(),
            false,
        );
        assert!(result.contains("Error"), "expected error: {result}");
        assert!(
            result.contains("not found"),
            "expected 'not found': {result}"
        );
    }
}
