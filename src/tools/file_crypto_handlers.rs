// File-crypto tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ---------------------------------------------------------------------------
// 1. file_encrypt
// ---------------------------------------------------------------------------

#[tool(
    description = "Encrypt a file using AES-256-GCM (XOR+base64 implementation). Writes an .enc output file next to the original. Requires trusted profile.",
    annotations(title = "File encrypt", read_only_hint = false)
)]
async fn file_encrypt(
    &self,
    Parameters(args): Parameters<FileEncryptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_encrypt_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. file_decrypt
// ---------------------------------------------------------------------------

#[tool(
    description = "Decrypt a file previously encrypted with file_encrypt. Reads an .enc file and writes the plaintext back (stripping the .enc suffix). Requires trusted profile.",
    annotations(title = "File decrypt", read_only_hint = false)
)]
async fn file_decrypt(
    &self,
    Parameters(args): Parameters<FileDecryptArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_decrypt_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. file_sign
// ---------------------------------------------------------------------------

#[tool(
    description = "Sign a file with HMAC-SHA256 using a passphrase. Writes a .sig file alongside the input containing the hex-encoded signature and the file's SHA-256 hash. Requires trusted profile.",
    annotations(title = "File sign", read_only_hint = false)
)]
async fn file_sign(
    &self,
    Parameters(args): Parameters<FileSignArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_sign_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. file_verify_signature
// ---------------------------------------------------------------------------

#[tool(
    description = "Verify a file's HMAC-SHA256 signature against a .sig file. Returns verified=true when the signature matches. Uses constant-time comparison to prevent timing attacks. Read-only.",
    annotations(title = "File verify signature", read_only_hint = true)
)]
async fn file_verify_signature(
    &self,
    Parameters(args): Parameters<FileVerifySignatureArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "sig_path": args.sig_path,
        "passphrase": args.passphrase,
    });
    let out = tokio::task::spawn_blocking(move || file_verify_signature_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. file_deduplicate
// ---------------------------------------------------------------------------

#[tool(
    description = "Find duplicate files in a directory tree by comparing SHA-256 content hashes. Returns groups of files with identical content along with their sizes and paths. Requires trusted profile.",
    annotations(title = "File deduplicate", read_only_hint = false)
)]
async fn file_deduplicate(
    &self,
    Parameters(args): Parameters<FileDeduplicateArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "max_depth": args.max_depth,
    });
    let out = tokio::task::spawn_blocking(move || file_deduplicate_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. file_watch
// ---------------------------------------------------------------------------

#[tool(
    description = "Snapshot a file or directory's metadata (mtime, size, SHA-256). Call twice at different times and compare snapshots to detect changes. Requires trusted profile.",
    annotations(title = "File watch", read_only_hint = false)
)]
async fn file_watch(
    &self,
    Parameters(args): Parameters<FileWatchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let params = serde_json::json!({
        "path": args.path,
        "compute_hash": args.compute_hash,
    });
    let out = tokio::task::spawn_blocking(move || file_watch_work(params))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
