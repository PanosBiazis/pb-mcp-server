// Sandbox tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ---------------------------------------------------------------------------
// 1. sandbox_exec
// ---------------------------------------------------------------------------

#[tool(
    description = "Execute a command within the sandboxed project root. Command must be a bare executable name (no path separators). Dangerous commands are blocked. Output is capped. Requires trusted profile.",
    annotations(title = "Sandbox exec", read_only_hint = false)
)]
async fn sandbox_exec(
    &self,
    Parameters(args): Parameters<SandboxExecArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let command = args.command.clone();
    let exec_args = args.exec_args.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let max_output_bytes = args.max_output_bytes.unwrap_or(65536);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_exec_work(root, command, exec_args, timeout_secs, max_output_bytes)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. sandbox_script
// ---------------------------------------------------------------------------

#[tool(
    description = "Execute a script source string in a sandboxed temporary directory. Supported interpreters: python3, node, bash, ruby, perl. Script is written to a temp file, run, then deleted. Requires trusted profile.",
    annotations(title = "Sandbox script", read_only_hint = false)
)]
async fn sandbox_script(
    &self,
    Parameters(args): Parameters<SandboxScriptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let script = args.script.clone();
    let language = args.language.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_script_work(root, script, language, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. sandbox_eval_code
// ---------------------------------------------------------------------------

#[tool(
    description = "Evaluate a code snippet with an optional stdin payload. Supports python3, node, bash, ruby, and perl. Runs in a sandboxed environment with a clean PATH. Requires trusted profile.",
    annotations(title = "Sandbox eval code", read_only_hint = false)
)]
async fn sandbox_eval_code(
    &self,
    Parameters(args): Parameters<SandboxEvalCodeArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let code = args.code.clone();
    let language = args.language.clone();
    let input_data = args.input_data.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_eval_code_work(root, code, language, input_data, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. sandbox_docker_run
// ---------------------------------------------------------------------------

#[tool(
    description = "Run a command inside a Docker container with --network=none, CPU=1, and a configurable memory cap. Project root is mounted read-only at /workspace. Requires Docker on PATH and trusted profile.",
    annotations(title = "Sandbox docker run", read_only_hint = false)
)]
async fn sandbox_docker_run(
    &self,
    Parameters(args): Parameters<SandboxDockerRunArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let image = args.image.clone();
    let command = args.command.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let memory_mb = args.memory_mb;
    let out = tokio::task::spawn_blocking(move || {
        sandbox_docker_run_work(root, image, command, timeout_secs, memory_mb)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. sandbox_checksum
// ---------------------------------------------------------------------------

#[tool(
    description = "Compute a checksum / hash of a file under the project root. Supported algorithms: sha256 (default), sha1, md5, blake3. Validates that the path stays within the project root. Requires trusted profile.",
    annotations(title = "Sandbox checksum", read_only_hint = true)
)]
async fn sandbox_checksum(
    &self,
    Parameters(args): Parameters<SandboxChecksumArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let algorithm = args.algorithm.unwrap_or_else(|| "sha256".to_string());
    let out = tokio::task::spawn_blocking(move || sandbox_checksum_work(root, path, algorithm))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. sandbox_diff
// ---------------------------------------------------------------------------

#[tool(
    description = "Compute a unified diff between two files within the project root. Both paths are validated against the project root to prevent path traversal. Requires trusted profile.",
    annotations(title = "Sandbox diff", read_only_hint = true)
)]
async fn sandbox_diff(
    &self,
    Parameters(args): Parameters<SandboxDiffArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let context_lines = args.context_lines;
    let out =
        tokio::task::spawn_blocking(move || sandbox_diff_work(root, path_a, path_b, context_lines))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 7. sandbox_file_watch
// ---------------------------------------------------------------------------

#[tool(
    description = "Watch a file within the project root for mtime changes and trigger a command when a change is detected (single-shot poll). Requires trusted profile.",
    annotations(title = "Sandbox file watch", read_only_hint = false)
)]
async fn sandbox_file_watch(
    &self,
    Parameters(args): Parameters<SandboxFileWatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let command = args.command.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_file_watch_work(root, path, command, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 8. sandbox_network_request
// ---------------------------------------------------------------------------

#[tool(
    description = "Make a sandboxed HTTP/HTTPS request via curl with SSRF protection (blocks internal/private IPs). Only http:// and https:// protocols allowed. Requires trusted profile.",
    annotations(title = "Sandbox network request", read_only_hint = false)
)]
async fn sandbox_network_request(
    &self,
    Parameters(args): Parameters<SandboxNetworkRequestArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let root = self.root.clone();
    let url = args.url.clone();
    let method = args.method.unwrap_or_else(|| "GET".to_string());
    let headers = args.headers.clone();
    let body = args.body.clone();
    let timeout_secs = args.timeout_secs.unwrap_or(30);
    let out = tokio::task::spawn_blocking(move || {
        sandbox_network_request_work(root, url, method, headers, body, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
