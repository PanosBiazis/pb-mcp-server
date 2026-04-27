//! Wired tool dispatch methods for all previously unused *_work imports.
//!
//! This file is included at the end of the `#[tool_router] impl MintMcp` block
//! in main.rs via `include!("tools_wired.rs")`. Every `#[tool]` method here
//! calls the corresponding `*_work` function that was previously imported but
//! never dispatched, making it visible to Zed / Cursor via MCP tools/list.

// ── AI / Ollama ─────────────────────────────────────────────────────────────

#[tool(
    description = "Generate text with a local Ollama model. Blocked on public profile. Requires MINT_TERMINAL_ENABLE=1 and `ollama` in MINT_TERMINAL_ALLOW.",
    annotations(title = "Ollama generate", read_only_hint = true)
)]
async fn ai_ollama_generate(
    &self,
    Parameters(args): Parameters<AiOllamaGenerateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let model = args.model.clone();
    let prompt = args.prompt.clone();
    let system = args.system_prompt.clone();
    let max_tokens = args.max_tokens;
    let out = tokio::task::spawn_blocking(move || {
        ai_ollama_generate_work(model, prompt, system, max_tokens)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Embed text with a local Ollama model and return the vector. Blocked on public profile.",
    annotations(title = "Ollama embed", read_only_hint = true)
)]
async fn ai_ollama_embed(
    &self,
    Parameters(args): Parameters<AiOllamaEmbedArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let model = args.model.clone();
    let input = args.input.clone();
    let out =
        tokio::task::spawn_blocking(move || ai_ollama_embed_work(model, input))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Network ──────────────────────────────────────────────────────────────────

#[tool(
    description = "DNS lookup for a hostname — returns A/AAAA/MX/TXT records. Read-only; blocked on public profile.",
    annotations(title = "DNS lookup", read_only_hint = true)
)]
async fn web_dns_lookup(
    &self,
    Parameters(args): Parameters<WebDnsLookupArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let hostname = args.hostname.clone();
    let record_types = args.record_types.clone();
    let out = tokio::task::spawn_blocking(move || web_dns_lookup_work(hostname, record_types))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Inspect TLS certificate chain for a host (expiry, issuer, SANs, cipher). Blocked on public profile.",
    annotations(title = "TLS inspect", read_only_hint = true)
)]
async fn web_tls_inspect(
    &self,
    Parameters(args): Parameters<WebTlsInspectArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let host = args.host.clone();
    let port = args.port;
    let out =
        tokio::task::spawn_blocking(move || web_tls_inspect_work(host, port))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Notify ────────────────────────────────────────────────────────────────────

#[tool(
    description = "Show a desktop notification via notify-send (Linux) or osascript (macOS). Blocked on public profile.",
    annotations(title = "Notify desktop", read_only_hint = false)
)]
async fn notify_desktop(
    &self,
    Parameters(args): Parameters<NotifyDesktopArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let title = args.title.clone();
    let body = args.body.clone();
    let urgency = args.urgency.clone();
    let out = tokio::task::spawn_blocking(move || notify_desktop_work(title, body, urgency))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Send an email via sendmail or mailx. Blocked on public profile.",
    annotations(title = "Notify email", read_only_hint = false)
)]
async fn notify_email(
    &self,
    Parameters(args): Parameters<NotifyEmailArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let to = args.to.clone();
    let subject = args.subject.clone();
    let body = args.body.clone();
    let from = args.from.clone();
    let out =
        tokio::task::spawn_blocking(move || notify_email_send_work(to, subject, body, from))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Send a JSON payload to a webhook URL via curl. Blocked on public profile. Rejects internal IPs by default (SSRF guard).",
    annotations(title = "Notify webhook", read_only_hint = false)
)]
async fn notify_webhook(
    &self,
    Parameters(args): Parameters<NotifyWebhookArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let url = args.url.clone();
    let payload = args.payload.clone();
    let method = args.method.clone();
    let block_internal = args.block_internal;
    let out = tokio::task::spawn_blocking(move || {
        notify_webhook_work(url, payload, method, None, block_internal)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── System ────────────────────────────────────────────────────────────────────

#[tool(
    description = "List running processes (cross-platform ps snapshot). Read-only; blocked on public profile.",
    annotations(title = "System process list", read_only_hint = true)
)]
async fn system_process_list(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(system_process_list_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Report disk usage (free/total/used) for a path. Read-only; blocked on public profile.",
    annotations(title = "System disk usage", read_only_hint = true)
)]
async fn system_disk_usage(
    &self,
    Parameters(args): Parameters<SystemDiskUsageArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || system_disk_usage_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Get the value of an environment variable. Read-only; blocked on public profile.",
    annotations(title = "System env get", read_only_hint = true)
)]
async fn system_env_get(
    &self,
    Parameters(args): Parameters<SystemEnvGetArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let name = args.name.clone();
    let out = tokio::task::spawn_blocking(move || system_env_get_work(name))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List network interfaces with IP addresses and flags. Read-only; blocked on public profile.",
    annotations(title = "System network interfaces", read_only_hint = true)
)]
async fn system_network_interfaces(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let out = tokio::task::spawn_blocking(system_network_interfaces_work)
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Database ──────────────────────────────────────────────────────────────────

#[tool(
    description = "Execute a read-only SELECT against a SQLite database under the project root. Returns rows as JSON.",
    annotations(title = "SQLite query (read)", read_only_hint = true)
)]
async fn db_sqlite_query(
    &self,
    Parameters(args): Parameters<DbSqliteQueryArgs>,
) -> Result<CallToolResult, McpError> {
    let db_path = match self.resolve_path(&args.db_path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let query = args.query.clone();
    let out = tokio::task::spawn_blocking(move || db_sqlite_query_work(db_path, query))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Execute a write SQL statement (INSERT/UPDATE/DELETE/CREATE) against a SQLite database under the project root. Blocked on public profile.",
    annotations(title = "SQLite exec (write)", read_only_hint = false)
)]
async fn db_sqlite_exec(
    &self,
    Parameters(args): Parameters<DbSqliteExecArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let db_path = match self.resolve_path(&args.db_path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let statement = args.statement.clone();
    let out = tokio::task::spawn_blocking(move || db_sqlite_exec_work(db_path, statement))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Dump the schema (tables, columns, indexes) of a SQLite database under the project root. Read-only.",
    annotations(title = "SQLite schema", read_only_hint = true)
)]
async fn db_sqlite_schema(
    &self,
    Parameters(args): Parameters<DbSqliteSchemaArgs>,
) -> Result<CallToolResult, McpError> {
    let db_path = match self.resolve_path(&args.db_path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || db_sqlite_schema_work(db_path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Query a JSON file under the project root using a JSONPath-style expression. Read-only.",
    annotations(title = "JSON query", read_only_hint = true)
)]
async fn db_json_query(
    &self,
    Parameters(args): Parameters<DbJsonQueryArgs>,
) -> Result<CallToolResult, McpError> {
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let query = args.query.clone();
    let out = tokio::task::spawn_blocking(move || db_json_query_work(path, query))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Web extended ──────────────────────────────────────────────────────────────

#[tool(
    description = "Make an HTTP API call (GET/POST/PUT/PATCH/DELETE). Returns status + response body. Blocked on public profile.",
    annotations(title = "Web API call", read_only_hint = false)
)]
async fn web_api_call(
    &self,
    Parameters(args): Parameters<WebApiCallArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let method = args.method.clone();
    let url = args.url.clone();
    let body = args.body.clone();
    let headers = args.headers.clone();
    let out = tokio::task::spawn_blocking(move || web_api_call_work(method, url, body, headers))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Download a file from a URL to a path under the project root. Blocked on public profile.",
    annotations(title = "Web download file", read_only_hint = false)
)]
async fn web_download_file(
    &self,
    Parameters(args): Parameters<WebDownloadFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let url = args.url.clone();
    let dest = match self.resolve_path(&args.dest) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || web_download_file_work(url, dest))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Take a screenshot of a URL using headless Chrome/Chromium or wkhtmltoimage. Blocked on public profile.",
    annotations(title = "Web screenshot", read_only_hint = true)
)]
async fn web_screenshot_url(
    &self,
    Parameters(args): Parameters<WebScreenshotArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let url = args.url.clone();
    let output = args.output_path.as_ref().map(|p| {
        self.root.join(p)
    });
    let out = tokio::task::spawn_blocking(move || web_screenshot_work(url, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Web search using a configured backend (DuckDuckGo/Brave/SearXNG). Returns title + URL + snippet for each result. Blocked on public profile.",
    annotations(title = "Web search", read_only_hint = true)
)]
async fn web_search(
    &self,
    Parameters(args): Parameters<WebSearchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let query = args.query.clone();
    let max_results = args.max_results;
    let out = tokio::task::spawn_blocking(move || web_search_work(query, max_results))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Fetch and parse an RSS/Atom feed URL. Returns item titles, links, and summaries. Blocked on public profile.",
    annotations(title = "RSS fetch", read_only_hint = true)
)]
async fn web_rss_fetch(
    &self,
    Parameters(args): Parameters<WebRssFetchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let url = args.url.clone();
    let max_items = args.max_items;
    let out = tokio::task::spawn_blocking(move || web_rss_fetch_work(url, max_items))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Convert ───────────────────────────────────────────────────────────────────

#[tool(
    description = "Extract an archive (zip/tar/gz/bz2/xz/zstd) to a destination directory under the project root. Blocked on public profile.",
    annotations(title = "Convert: archive extract", read_only_hint = false)
)]
async fn convert_archive_extract(
    &self,
    Parameters(args): Parameters<ConvertArchiveExtractArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let src = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let dst = match self.resolve_path(&args.dest) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || convert_archive_extract_work(src, dst))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "List the contents of an archive (zip/tar/gz/bz2/xz/zstd) without extracting. Read-only.",
    annotations(title = "Convert: archive list", read_only_hint = true)
)]
async fn convert_archive_list(
    &self,
    Parameters(args): Parameters<ConvertArchiveListArgs>,
) -> Result<CallToolResult, McpError> {
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || convert_archive_list_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Hex dump of a binary file under the project root. Read-only.",
    annotations(title = "Convert: binary hexdump", read_only_hint = true)
)]
async fn convert_binary_hexdump(
    &self,
    Parameters(args): Parameters<ConvertBinaryHexdumpArgs>,
) -> Result<CallToolResult, McpError> {
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let max_bytes = args.max_bytes;
    let out = tokio::task::spawn_blocking(move || convert_binary_hexdump_work(path, max_bytes))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Convert a document between formats using pandoc (e.g. md→docx, rst→pdf). Blocked on public profile.",
    annotations(title = "Convert: document", read_only_hint = false)
)]
async fn convert_document(
    &self,
    Parameters(args): Parameters<ConvertDocumentArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = match self.resolve_path(&args.input) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let output = match self.resolve_path(&args.output) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || convert_document_work(input, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Re-encode a text file between character encodings (e.g. latin-1 → utf-8). Read-only (returns converted text).",
    annotations(title = "Convert: encoding", read_only_hint = true)
)]
async fn convert_encoding(
    &self,
    Parameters(args): Parameters<ConvertEncodingArgs>,
) -> Result<CallToolResult, McpError> {
    let text = args.text.clone();
    let from_enc = args.from_enc.clone();
    let to_enc = args.to_enc.clone();
    let out = tokio::task::spawn_blocking(move || convert_encoding_work(text, from_enc, to_enc))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Extract text from a PDF file under the project root using pdftotext. Read-only.",
    annotations(title = "Convert: PDF extract", read_only_hint = true)
)]
async fn convert_pdf_extract(
    &self,
    Parameters(args): Parameters<ConvertPdfExtractArgs>,
) -> Result<CallToolResult, McpError> {
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || convert_pdf_extract_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Generate a QR code PNG from input text and save it under the project root. Blocked on public profile.",
    annotations(title = "Convert: QR generate", read_only_hint = false)
)]
async fn convert_qr_generate(
    &self,
    Parameters(args): Parameters<ConvertQrGenerateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let text = args.text.clone();
    let output = args.output_path.as_ref().map(|p| self.root.join(p));
    let out = tokio::task::spawn_blocking(move || convert_qr_generate_work(text, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ── Agent sync ────────────────────────────────────────────────────────────────

#[tool(
    description = "Synchronise all agent state under the project root (writes .mint-mcp/agent-sync/). Blocked on public profile.",
    annotations(title = "Agent sync all", read_only_hint = false)
)]
async fn agent_sync_all_tool(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self
        .root
        .canonicalize()
        .unwrap_or_else(|_| self.root.clone());
    let out = crate::ops::agent_sync_all(root)
        .await
        .unwrap_or_else(|e| format!("Error: {e}"));
    Self::tool_text(out)
}

#[tool(
    description = "Confirm a pending agent sync by sync_id. Blocked on public profile.",
    annotations(title = "Agent sync confirm", read_only_hint = false)
)]
async fn agent_sync_confirm_tool(
    &self,
    Parameters(args): Parameters<AgentSyncConfirmArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self
        .root
        .canonicalize()
        .unwrap_or_else(|_| self.root.clone());
    let sync_id = args.sync_id.clone();
    let out = crate::ops::agent_sync_confirm(root, sync_id)
        .await
        .unwrap_or_else(|e| format!("Error: {e}"));
    Self::tool_text(out)
}
