//! # pb-lsp-bridge — LSP↔MCP Protocol Bridge
//!
//! Binary that implements an LSP server on stdin/stdout (JSON-RPC over
//! `Content-Length` framing) and bridges each LSP request to the
//! corresponding `pb_mcp_server::ops` work function.
//!
//! ## Supported LSP Methods
//!
//! | LSP Method                 | MCP Tool                       |
//! |----------------------------|--------------------------------|
//! | `initialize`               | — (capabilities handshake)     |
//! | `initialized`              | — (no-op)                      |
//! | `textDocument/hover`       | `read_file` / `code_ast_dump`  |
//! | `textDocument/completion`  | `codebase_index`               |
//! | `textDocument/codeLens`    | `review_focused_review`        |
//! | `textDocument/diagnostic`  | `project_cargo_check` / pytest |
//! | `workspace/symbol`         | `codebase_index`               |
//!
//! ## Usage
//!
//! ```sh
//! # Android Studio / JetBrains external LSP server config:
//! pb-lsp-bridge
//!
//! # Optional env vars:
//! PB_PROJECT_ROOT=/path/to/project # project root (default: cwd)
//! MINT_PROJECT_ROOT=/path/to/project # legacy fallback project root
//! PROJECT_ROOT=/path/to/project # further fallback project root
//! ```

#![cfg(feature = "lsp-bridge")]

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};

use lsp_types::{
    CodeLens, CodeLensOptions, Command, CompletionItem, CompletionItemKind, CompletionList,
    CompletionOptions, Diagnostic, DiagnosticOptions, DiagnosticServerCapabilities,
    DocumentDiagnosticReport, FullDocumentDiagnosticReport, Hover, HoverContents, InitializeResult,
    MarkupContent, MarkupKind, Position, Range, RelatedFullDocumentDiagnosticReport,
    ServerCapabilities, ServerInfo, SymbolInformation, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, WorkspaceSymbolResponse,
};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// MCP Tool Mapping
// ---------------------------------------------------------------------------

/// Maps an LSP method to an MCP tool name and parameter transformation.
#[allow(dead_code)]
struct McpToolMapping {
    /// MCP tool name to invoke.
    mcp_tool: String,
    /// LSP parameter name → MCP parameter name.
    param_map: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Bridge Server
// ---------------------------------------------------------------------------

/// The LSP-to-MCP bridge server.
///
/// Reads JSON-RPC messages from stdin, dispatches to the appropriate
/// `pb_mcp_server::ops` work function, and writes JSON-RPC responses
/// to stdout.
struct LspBridge {
    /// Absolute path to the project root.
    project_root: PathBuf,
    /// Registered LSP method → MCP tool mappings.
    #[allow(dead_code)]
    mcp_tools: HashMap<String, McpToolMapping>,
    /// Whether the `initialized` notification has been received.
    initialized: bool,
}

impl LspBridge {
    /// Create a new bridge rooted at `project_root`.
    fn new(project_root: PathBuf) -> Self {
        let mut mcp_tools = HashMap::new();

        mcp_tools.insert(
            "textDocument/hover".into(),
            McpToolMapping {
                mcp_tool: "read_file".into(),
                param_map: {
                    let mut m = HashMap::new();
                    m.insert("textDocument.uri".into(), "path".into());
                    m.insert("position.line".into(), "start_line".into());
                    m
                },
            },
        );

        mcp_tools.insert(
            "textDocument/completion".into(),
            McpToolMapping {
                mcp_tool: "codebase_index".into(),
                param_map: {
                    let mut m = HashMap::new();
                    m.insert("textDocument.uri".into(), "search_root".into());
                    m
                },
            },
        );

        mcp_tools.insert(
            "textDocument/codeLens".into(),
            McpToolMapping {
                mcp_tool: "review_focused_review".into(),
                param_map: {
                    let mut m = HashMap::new();
                    m.insert("textDocument.uri".into(), "root".into());
                    m
                },
            },
        );

        mcp_tools.insert(
            "textDocument/diagnostic".into(),
            McpToolMapping {
                mcp_tool: "project_cargo_check".into(),
                param_map: HashMap::new(),
            },
        );

        mcp_tools.insert(
            "workspace/symbol".into(),
            McpToolMapping {
                mcp_tool: "codebase_index".into(),
                param_map: {
                    let mut m = HashMap::new();
                    m.insert("query".into(), "search_root".into());
                    m
                },
            },
        );

        Self {
            project_root,
            mcp_tools,
            initialized: false,
        }
    }

    /// Enter the main JSON-RPC read-eval-print loop.
    ///
    /// Reads `Content-Length` framed messages from stdin, dispatches
    /// them, and writes framed responses to stdout. Blocks until
    /// stdin is closed or an unrecoverable I/O error occurs.
    fn run(&mut self) {
        eprintln!(
            "[pb-lsp-bridge] starting; project_root={}",
            self.project_root.display()
        );
        let stdin = io::stdin();
        let mut header_buf = String::new();

        loop {
            header_buf.clear();

            // ── Read header section ──────────────────────────────────
            let stdin_lock = stdin.lock();
            let mut lines = stdin_lock.lines();

            let mut content_length: Option<usize> = None;

            loop {
                header_buf.clear();
                match lines.next() {
                    Some(Ok(line)) => {
                        if line.is_empty() {
                            // Blank line = end of headers
                            break;
                        }
                        if let Some(rest) = line.strip_prefix("Content-Length:") {
                            match rest.trim().parse::<usize>() {
                                Ok(n) => content_length = Some(n),
                                Err(e) => {
                                    eprintln!("[pb-lsp-bridge] bad Content-Length: {e}");
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("[pb-lsp-bridge] stdin read error: {e}");
                        return;
                    }
                    None => {
                        eprintln!("[pb-lsp-bridge] stdin closed");
                        return;
                    }
                }
            }

            let n = match content_length {
                Some(n) => n,
                None => {
                    eprintln!("[pb-lsp-bridge] missing Content-Length header, skipping message");
                    continue;
                }
            };

            // ── Read body ────────────────────────────────────────────
            // We can't reuse the `lines` iterator because we need raw bytes.
            // Drop the lock and re-acquire for raw read.
            drop(lines);

            let mut body_buf = vec![0u8; n];
            if let Err(e) = io::stdin().lock().read_exact(&mut body_buf) {
                eprintln!("[pb-lsp-bridge] failed to read body: {e}");
                return;
            }

            let message: Value = match serde_json::from_slice(&body_buf) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[pb-lsp-bridge] JSON parse error: {e}");
                    continue;
                }
            };

            eprintln!(
                "[pb-lsp-bridge] recv: method={}",
                message
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>")
            );

            // ── Dispatch ─────────────────────────────────────────────
            let response = self.dispatch(&message);
            if let Some(resp) = response {
                if let Err(e) = write_message(&resp) {
                    eprintln!("[pb-lsp-bridge] write error: {e}");
                    return;
                }
            }
        }
    }

    /// Dispatch a parsed JSON-RPC message and return an optional response.
    fn dispatch(&mut self, message: &Value) -> Option<Value> {
        let id = message.get("id").cloned();
        let method = message.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = message.get("params").cloned().unwrap_or(json!({}));

        // Requests have an `id`; notifications don't.
        let is_request = id.is_some();

        match method {
            "initialize" => {
                if let Some(ref id_val) = id {
                    let result = self.handle_initialize(&params);
                    return Some(make_response(id_val.clone(), Ok(result)));
                }
            }

            "initialized" => {
                self.initialized = true;
                eprintln!("[pb-lsp-bridge] client initialized");
                // Notifications don't get responses
                return None;
            }

            "shutdown" => {
                if let Some(ref id_val) = id {
                    return Some(make_response(id_val.clone(), Ok(Value::Null)));
                }
            }

            "exit" => {
                eprintln!("[pb-lsp-bridge] exit notification received, shutting down");
                std::process::exit(0);
            }

            "textDocument/hover" if is_request => {
                if let Some(ref id_val) = id {
                    let result = self.handle_hover(&params);
                    return Some(make_response(id_val.clone(), result));
                }
            }

            "textDocument/completion" if is_request => {
                if let Some(ref id_val) = id {
                    let result = self.handle_completion(&params);
                    return Some(make_response(id_val.clone(), result));
                }
            }

            "textDocument/codeLens" if is_request => {
                if let Some(ref id_val) = id {
                    let result = self.handle_code_lens(&params);
                    return Some(make_response(id_val.clone(), result));
                }
            }

            "textDocument/diagnostic" if is_request => {
                if let Some(ref id_val) = id {
                    let result = self.handle_diagnostic(&params);
                    return Some(make_response(id_val.clone(), result));
                }
            }

            "workspace/symbol" if is_request => {
                if let Some(ref id_val) = id {
                    let result = self.handle_workspace_symbol(&params);
                    return Some(make_response(id_val.clone(), result));
                }
            }

            "$/cancelRequest" => {
                // No-op: we don't have long-running requests to cancel
                eprintln!("[pb-lsp-bridge] cancel request (ignored)");
                return None;
            }

            other => {
                eprintln!("[pb-lsp-bridge] unknown method: {other}");
                if is_request {
                    if let Some(ref id_val) = id {
                        let err = make_error_response(-32601, format!("method not found: {other}"));
                        return Some(make_response(id_val.clone(), Err(err)));
                    }
                }
            }
        }

        None
    }

    // ── LSP Method Handlers ──────────────────────────────────────────

    /// `initialize` — respond with server capabilities.
    fn handle_initialize(&self, _params: &Value) -> Value {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    save: None,
                },
            )),
            hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![".".to_string(), ":".to_string(), "#".to_string()]),
                all_commit_characters: None,
                work_done_progress_options: Default::default(),
                completion_item: None,
            }),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                identifier: Some("pb-lsp-bridge".to_string()),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
                work_done_progress_options: Default::default(),
            })),
            workspace_symbol_provider: Some(lsp_types::OneOf::Left(true)),
            ..Default::default()
        };

        let result = InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: "pb-lsp-bridge".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        };

        serde_json::to_value(result).unwrap_or_else(|e| {
            eprintln!("[pb-lsp-bridge] serialize InitializeResult: {e}");
            json!({})
        })
    }

    /// `textDocument/hover` — read file at position and return hover info.
    fn handle_hover(&self, params: &Value) -> Result<Value, Value> {
        let uri = params
            .get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let line = params
            .get("position")
            .and_then(|v| v.get("line"))
            .and_then(|v| v.as_u64())
            .map(|l| l as usize + 1) // LSP is 0-based, our ops are 1-based
            .unwrap_or(1);

        let file_path = uri_to_path(uri);
        eprintln!(
            "[pb-lsp-bridge] hover: path={}, line={}",
            file_path.display(),
            line
        );

        // Try read_file_work first
        let content = pb_mcp_server::ops::read_file_work(file_path.clone(), Some(line), Some(line));

        // Also try AST dump for richer info
        let ast_info = pb_mcp_server::ops::code_intel::code_ast_dump_work(file_path.clone(), None);

        let mut hover_text = String::new();

        if !content.starts_with("Error") {
            hover_text.push_str(&content);
        }

        if !ast_info.starts_with("Error")
            && !ast_info.starts_with("CODE: AST DUMP\n\nNo structural items")
        {
            if !hover_text.is_empty() {
                hover_text.push_str("\n\n---\n\n");
            }
            hover_text.push_str(&ast_info);
        }

        if hover_text.is_empty() {
            hover_text = format!("No hover information available for {}", file_path.display());
        }

        let hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_text,
            }),
            range: Some(Range {
                start: Position {
                    line: line.saturating_sub(1) as u32,
                    character: 0,
                },
                end: Position {
                    line: line.saturating_sub(1) as u32,
                    character: 0,
                },
            }),
        };

        Ok(serde_json::to_value(hover).unwrap_or_else(|e| {
            eprintln!("[pb-lsp-bridge] serialize Hover: {e}");
            json!({ "contents": { "kind": "markdown", "value": "hover error" } })
        }))
    }

    /// `textDocument/completion` — use codebase index for completions.
    fn handle_completion(&self, params: &Value) -> Result<Value, Value> {
        let uri = params
            .get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let file_path = uri_to_path(uri);
        let search_root = self.project_root.clone();

        eprintln!(
            "[pb-lsp-bridge] completion: file={}, search_root={}",
            file_path.display(),
            search_root.display()
        );

        // Use codebase_index_work to scan the project for symbols
        let index_result = pb_mcp_server::ops::codebase_index_work(
            search_root,
            2000, // max_files_scan
            true, // symbol_sketch
            200,  // max_symbol_entries
        );

        let items = parse_symbols_from_index(&index_result, &file_path);

        let list = CompletionList {
            is_incomplete: true,
            items,
        };

        Ok(serde_json::to_value(list).unwrap_or_else(|e| {
            eprintln!("[pb-lsp-bridge] serialize CompletionList: {e}");
            json!({ "isIncomplete": true, "items": [] })
        }))
    }

    /// `textDocument/codeLens` — review the focused areas of the document.
    fn handle_code_lens(&self, params: &Value) -> Result<Value, Value> {
        let uri = params
            .get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let file_path = uri_to_path(uri);
        let root = self.project_root.clone();

        eprintln!(
            "[pb-lsp-bridge] codeLens: file={}, root={}",
            file_path.display(),
            root.display()
        );

        // Run focused review across security, performance, correctness
        let review_result = pb_mcp_server::ops::review::review_focused_review_work(
            root,
            "security,performance,correctness".to_string(),
            false, // staged
            None,  // revision_range
        );

        let lenses = build_code_lenses_from_review(&review_result);

        Ok(serde_json::to_value(lenses).unwrap_or_else(|e| {
            eprintln!("[pb-lsp-bridge] serialize CodeLens: {e}");
            json!([])
        }))
    }

    /// `textDocument/diagnostic` — run cargo check / pytest and map output.
    fn handle_diagnostic(&self, params: &Value) -> Result<Value, Value> {
        let uri = params
            .get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let file_path = uri_to_path(uri);
        eprintln!("[pb-lsp-bridge] diagnostic: file={}", file_path.display());

        let diagnostics = run_diagnostics(&self.project_root, &file_path);

        let report = DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: None,
                items: diagnostics,
            },
        });

        Ok(serde_json::to_value(report).unwrap_or_else(|e| {
            eprintln!("[pb-lsp-bridge] serialize DocumentDiagnosticReport: {e}");
            json!({ "kind": "full", "items": [] })
        }))
    }

    /// `workspace/symbol` — search the codebase index for matching symbols.
    fn handle_workspace_symbol(&self, params: &Value) -> Result<Value, Value> {
        let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");

        eprintln!("[pb-lsp-bridge] workspace/symbol: query={query}");

        // Use codebase_index_search_work if index exists, fall back to codebase_index_work
        let search_result = pb_mcp_server::code_index::code_index_search_work(
            self.project_root.clone(),
            query.to_string(),
            false, // case_sensitive
            50,    // limit
            None,  // path_prefix
            false, // use_regex
        );

        let symbols = parse_symbol_search_results(&search_result);

        Ok(
            serde_json::to_value(WorkspaceSymbolResponse::Flat(symbols)).unwrap_or_else(|e| {
                eprintln!("[pb-lsp-bridge] serialize workspace/symbol: {e}");
                json!([])
            }),
        )
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC Helpers
// ---------------------------------------------------------------------------

/// Build a JSON-RPC response with the given id and result/error.
fn make_response(id: Value, result: Result<Value, Value>) -> Value {
    match result {
        Ok(r) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": r,
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": e,
        }),
    }
}

/// Build a JSON-RPC error object.
fn make_error_response(code: i32, message: String) -> Value {
    json!({
        "code": code,
        "message": message,
    })
}

/// Write a JSON-RPC message to stdout with `Content-Length` framing.
fn write_message(value: &Value) -> io::Result<()> {
    let body = serde_json::to_string(value).unwrap_or_default();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(body.as_bytes())?;
    out.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// URI / Path Conversion
// ---------------------------------------------------------------------------

/// Convert a file URI to a local `PathBuf`.
///
/// Handles `file:///path/to/file` on Linux/macOS and
/// `file:///C:/path/to/file` on Windows.
fn uri_to_path(uri: &str) -> PathBuf {
    if let Some(path_str) = uri.strip_prefix("file://") {
        // On Windows, strip the leading slash before the drive letter
        // e.g. file:///C:/foo → C:/foo
        #[cfg(windows)]
        {
            if path_str.starts_with('/')
                && path_str.len() > 2
                && path_str.chars().nth(2) == Some(':')
            {
                return PathBuf::from(&path_str[1..]);
            }
        }
        PathBuf::from(path_str)
    } else {
        PathBuf::from(uri)
    }
}

// ---------------------------------------------------------------------------
// Result Parsing Helpers
// ---------------------------------------------------------------------------

/// Parse symbol sketch lines from `codebase_index_work` output into
/// `CompletionItem`s, optionally filtering by the given file path.
fn parse_symbols_from_index(index_text: &str, filter_file: &Path) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let filter_str = filter_file.to_string_lossy();

    for line in index_text.lines() {
        // Symbol sketch lines look like:  src/foo.rs:42:fn my_function
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            continue;
        }
        let file = parts[0];
        let line_num = parts[1];
        let symbol = parts[2];

        // If filter_file is non-empty, only include symbols from that file
        if !filter_str.is_empty() && file != filter_str.as_ref() {
            continue;
        }

        let kind = infer_completion_kind(symbol);

        items.push(CompletionItem {
            label: extract_symbol_name(symbol),
            kind: Some(kind),
            detail: Some(format!("{}:{}", file, line_num)),
            sort_text: Some(format!("{:05}", items.len())),
            ..Default::default()
        });

        if items.len() >= 100 {
            break;
        }
    }

    items
}

/// Extract the symbol name from a sketch line like `fn my_function` or `struct MyStruct`.
fn extract_symbol_name(sketch: &str) -> String {
    sketch
        .split_whitespace()
        .nth(1)
        .unwrap_or(sketch)
        .trim_matches(&['(', '<', '{', ':'][..])
        .to_string()
}

/// Infer a `CompletionItemKind` from the symbol sketch prefix.
fn infer_completion_kind(sketch: &str) -> CompletionItemKind {
    let lower = sketch.to_lowercase();
    if lower.starts_with("fn ") || lower.starts_with("def ") || lower.starts_with("func ") {
        CompletionItemKind::FUNCTION
    } else if lower.starts_with("struct ") || lower.starts_with("class ") {
        CompletionItemKind::CLASS
    } else if lower.starts_with("enum ") {
        CompletionItemKind::ENUM
    } else if lower.starts_with("interface ") || lower.starts_with("trait ") {
        CompletionItemKind::INTERFACE
    } else if lower.starts_with("mod ") {
        CompletionItemKind::MODULE
    } else if lower.starts_with("type ") {
        CompletionItemKind::TYPE_PARAMETER
    } else {
        CompletionItemKind::VARIABLE
    }
}

/// Build `CodeLens` entries from a focused review output.
fn build_code_lenses_from_review(review_text: &str) -> Vec<CodeLens> {
    let mut lenses = Vec::new();

    for line in review_text.lines() {
        let trimmed = line.trim();

        // Finding lines look like:  L42: src/foo.rs → found "unwrap()"
        if !trimmed.starts_with('L') {
            continue;
        }

        // Parse: L<line>: <file> → <finding>
        let rest = &trimmed[1..]; // strip leading 'L'
        let colon_pos = match rest.find(':') {
            Some(p) => p,
            None => continue,
        };

        let line_num: u32 = match rest[..colon_pos].parse::<u32>() {
            Ok(n) => n.saturating_sub(1), // convert 1-based to 0-based
            Err(_) => continue,
        };

        let after_colon = &rest[colon_pos + 1..];
        let (file_part, finding_part): (&str, &str) = if let Some(arrow) = after_colon.find("→") {
            (
                &after_colon[..arrow].trim(),
                after_colon[arrow + 3..].trim(),
            )
        } else {
            (after_colon.trim(), "")
        };

        let title = if finding_part.is_empty() {
            format!("🔍 Review: {}", file_part)
        } else {
            format!("🔍 {}", finding_part)
        };

        lenses.push(CodeLens {
            range: Range {
                start: Position {
                    line: line_num,
                    character: 0,
                },
                end: Position {
                    line: line_num,
                    character: 0,
                },
            },
            command: Some(Command {
                title,
                command: "pb-lsp-bridge.showReviewFinding".to_string(),
                arguments: Some(vec![json!(finding_part)]),
            }),
            data: None,
        });

        if lenses.len() >= 50 {
            break;
        }
    }

    lenses
}

/// Run diagnostics by invoking the appropriate MCP ops function.
///
/// Tries `cargo check` for Rust projects and falls back to pytest for
/// Python projects.
fn run_diagnostics(project_root: &Path, _file_path: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Try cargo check for Rust projects
    if project_root.join("Cargo.toml").exists() {
        match pb_mcp_server::ops::project_cargo_check_build_args(None, true) {
            Ok(args) => {
                eprintln!("[pb-lsp-bridge] cargo check args: cargo {}", args.join(" "));
                // The build_args function only produces the argv; actually
                // running cargo check requires a terminal. We wrap it as a
                // hint diagnostic.
                diagnostics.push(Diagnostic::new_simple(
                    Range::default(),
                    format!(
                        "pb-lsp-bridge: run `cargo {}` for full diagnostics",
                        args.join(" ")
                    ),
                ));
            }
            Err(e) => {
                diagnostics.push(Diagnostic::new_simple(
                    Range::default(),
                    format!("pb-lsp-bridge: cargo check config error: {e}"),
                ));
            }
        }
    }

    // Try pytest for Python projects
    if project_root.join("pytest.ini").exists()
        || project_root.join("conftest.py").exists()
        || project_root.join("pyproject.toml").exists()
    {
        match pb_mcp_server::ops::project_pytest_build_args(".", true, None, None) {
            Ok(args) => {
                eprintln!("[pb-lsp-bridge] pytest args: pytest {}", args.join(" "));
                diagnostics.push(Diagnostic::new_simple(
                    Range::default(),
                    format!(
                        "pb-lsp-bridge: run `pytest {}` for full diagnostics",
                        args.join(" ")
                    ),
                ));
            }
            Err(e) => {
                diagnostics.push(Diagnostic::new_simple(
                    Range::default(),
                    format!("pb-lsp-bridge: pytest config error: {e}"),
                ));
            }
        }
    }

    // If no project type was detected, add a generic hint
    if diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new_simple(
            Range::default(),
            "pb-lsp-bridge: no Rust or Python project detected for diagnostics".to_string(),
        ));
    }

    diagnostics
}

/// Parse `code_index_search_work` output into `SymbolInformation` items.
fn parse_symbol_search_results(search_text: &str) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let mut current_path: Option<String> = None;

    for line in search_text.lines() {
        let trimmed = line.trim();

        // Skip header / hint lines
        if trimmed.is_empty()
            || trimmed.starts_with("CODE_INDEX_SEARCH")
            || trimmed.starts_with("mode:")
            || trimmed.starts_with("matches")
            || trimmed.starts_with("index_ts_ms")
            || trimmed.starts_with("(no matches)")
            || trimmed.starts_with("Tip:")
        {
            continue;
        }

        // Path lines don't start with whitespace and don't start with "symbols:"
        if !trimmed.starts_with("symbols:") && !trimmed.starts_with(' ') && !trimmed.contains("→")
        {
            current_path = Some(trimmed.to_string());
            continue;
        }

        // Symbol lines look like:  symbols: Foo, Bar, Baz
        if let Some(sym_list) = trimmed.strip_prefix("symbols:") {
            if let Some(ref path) = current_path {
                let uri = path_to_uri(path);
                for name in sym_list.split(',') {
                    let name = name.trim();
                    if name.is_empty() {
                        continue;
                    }
                    #[allow(deprecated)]
                    symbols.push(SymbolInformation {
                        name: name.to_string(),
                        kind: infer_symbol_kind(name),
                        deprecated: None,
                        location: lsp_types::Location {
                            uri: uri.clone(),
                            range: Range::default(),
                        },
                        container_name: None,
                        tags: None,
                    });
                    if symbols.len() >= 50 {
                        return symbols;
                    }
                }
            }
        }
    }

    symbols
}

/// Convert a relative file path to a `file://` URI.
fn path_to_uri(path: &str) -> lsp_types::Url {
    let absolute = if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        format!("/{path}")
    };
    lsp_types::Url::parse(&format!("file://{absolute}"))
        .unwrap_or_else(|_| lsp_types::Url::parse("file:///unknown").unwrap())
}

/// Infer `SymbolKind` from a symbol name using heuristics.
fn infer_symbol_kind(name: &str) -> SymbolKind {
    // Common Rust / Python / Java conventions
    if name.starts_with(|c: char| c.is_uppercase()) {
        // PascalCase → class / struct / enum / type
        if name.to_uppercase() == name {
            SymbolKind::CONSTANT
        } else if name.ends_with("Error") || name.ends_with("Err") {
            SymbolKind::ENUM
        } else if name.ends_with("Trait") || name.ends_with("Interface") {
            SymbolKind::INTERFACE
        } else {
            SymbolKind::CLASS
        }
    } else if name.contains('_') || name.starts_with(|c: char| c.is_lowercase()) {
        // snake_case → function / variable / method
        SymbolKind::FUNCTION
    } else {
        SymbolKind::VARIABLE
    }
}

// ---------------------------------------------------------------------------
// Entry Point
// ---------------------------------------------------------------------------

fn main() {
    let project_root = std::env::var("PB_PROJECT_ROOT")
        .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
        .or_else(|_| std::env::var("PROJECT_ROOT"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut bridge = LspBridge::new(project_root);
    bridge.run();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_to_path_handles_file_scheme() {
        let path = uri_to_path("file:///home/user/project/src/main.rs");
        assert_eq!(path, PathBuf::from("/home/user/project/src/main.rs"));
    }

    #[test]
    fn uri_to_path_handles_relative() {
        let path = uri_to_path("src/main.rs");
        assert_eq!(path, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn extract_symbol_name_fn() {
        assert_eq!(extract_symbol_name("fn my_function"), "my_function");
    }

    #[test]
    fn extract_symbol_name_struct() {
        assert_eq!(extract_symbol_name("struct MyStruct"), "MyStruct");
    }

    #[test]
    fn extract_symbol_name_def() {
        assert_eq!(extract_symbol_name("def hello_world"), "hello_world");
    }

    #[test]
    fn extract_symbol_name_class() {
        assert_eq!(extract_symbol_name("class FooBar"), "FooBar");
    }

    #[test]
    fn infer_completion_kind_function() {
        assert_eq!(
            infer_completion_kind("fn foo"),
            CompletionItemKind::FUNCTION
        );
        assert_eq!(
            infer_completion_kind("def bar"),
            CompletionItemKind::FUNCTION
        );
    }

    #[test]
    fn infer_completion_kind_class() {
        assert_eq!(
            infer_completion_kind("struct Foo"),
            CompletionItemKind::CLASS
        );
        assert_eq!(
            infer_completion_kind("class Bar"),
            CompletionItemKind::CLASS
        );
    }

    #[test]
    fn infer_completion_kind_enum() {
        assert_eq!(
            infer_completion_kind("enum Color"),
            CompletionItemKind::ENUM
        );
    }

    #[test]
    fn infer_completion_kind_interface() {
        assert_eq!(
            infer_completion_kind("trait Display"),
            CompletionItemKind::INTERFACE
        );
    }

    #[test]
    fn infer_completion_kind_module() {
        assert_eq!(
            infer_completion_kind("mod utils"),
            CompletionItemKind::MODULE
        );
    }

    #[test]
    fn make_response_ok() {
        let resp = make_response(json!(1), Ok(json!({"hello": "world"})));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert!(resp.get("result").is_some());
        assert!(resp.get("error").is_none());
    }

    #[test]
    fn make_response_err() {
        let err = make_error_response(-32601, "not found".into());
        let resp = make_response(json!(2), Err(err));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 2);
        assert!(resp.get("error").is_some());
        assert!(resp.get("result").is_none());
    }

    #[test]
    fn parse_symbols_from_index_basic() {
        let index = r#"CODEBASE_INDEX
search_root: /project
files_scanned: 10
total_bytes: 1000
walk_truncated: false

--- extensions (count, bytes) ---

--- symbol sketch (heuristic; use grep_code for full search) ---
src/lib.rs:1:fn main
src/lib.rs:5:struct Config
src/utils.rs:12:fn helper
"#;
        let items = parse_symbols_from_index(index, Path::new("src/lib.rs"));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "main");
        assert_eq!(items[1].label, "Config");
    }

    #[test]
    fn parse_symbols_from_index_no_filter() {
        let index = "src/foo.rs:1:fn bar\nsrc/baz.rs:10:struct Qux\n";
        let items = parse_symbols_from_index(index, Path::new(""));
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_build_code_lenses_from_review() {
        let review = r#"REVIEW: FOCUSED REVIEW

Focus areas: security,performance,correctness

=== SECURITY ===
(checking for common security anti-patterns)

 L42: src/auth.rs → found "unwrap()"
 Found 1 finding(s)

=== PERFORMANCE ===
(checking for common performance anti-patterns)

 ✓ No findings

Total findings: 1
"#;
        let lenses = build_code_lenses_from_review(review);
        assert_eq!(lenses.len(), 1);
        assert_eq!(lenses[0].range.start.line, 41); // 0-based
        assert!(lenses[0].command.is_some());
    }

    #[test]
    fn infer_symbol_kind_pascal() {
        assert_eq!(infer_symbol_kind("MyStruct"), SymbolKind::CLASS);
    }

    #[test]
    fn infer_symbol_kind_screaming_snake() {
        assert_eq!(infer_symbol_kind("MAX_SIZE"), SymbolKind::CONSTANT);
    }

    #[test]
    fn infer_symbol_kind_snake() {
        assert_eq!(infer_symbol_kind("my_function"), SymbolKind::FUNCTION);
    }

    #[test]
    fn path_to_uri_absolute() {
        let url = path_to_uri("/home/user/project/src/main.rs");
        assert_eq!(url.as_str(), "file:///home/user/project/src/main.rs");
    }

    #[test]
    fn bridge_new_sets_project_root() {
        let bridge = LspBridge::new(PathBuf::from("/tmp/test"));
        assert_eq!(bridge.project_root, PathBuf::from("/tmp/test"));
        assert!(!bridge.initialized);
        assert!(bridge.mcp_tools.contains_key("textDocument/hover"));
        assert!(bridge.mcp_tools.contains_key("textDocument/completion"));
        assert!(bridge.mcp_tools.contains_key("textDocument/codeLens"));
        assert!(bridge.mcp_tools.contains_key("textDocument/diagnostic"));
        assert!(bridge.mcp_tools.contains_key("workspace/symbol"));
    }
}
