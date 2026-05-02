// Reverse-engineering tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ---------------------------------------------------------------------------
// 1. re_binary_info
// ---------------------------------------------------------------------------

#[tool(
    description = "Display binary file metadata: file type, target architecture, entry point, and ELF/PE/Mach-O header fields. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE binary info", read_only_hint = true)
)]
async fn re_binary_info(
    &self,
    Parameters(args): Parameters<ReBinaryInfoArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_binary_info_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 2. re_strings
// ---------------------------------------------------------------------------

#[tool(
    description = "Extract printable strings from a binary file with optional minimum length and encoding filters. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE strings", read_only_hint = true)
)]
async fn re_strings(
    &self,
    Parameters(args): Parameters<ReStringsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let min_length = args.min_length;
    let encoding = args.encoding.clone();
    let out = tokio::task::spawn_blocking(move || re_strings_work(path, min_length, encoding))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 3. re_disassemble
// ---------------------------------------------------------------------------

#[tool(
    description = "Disassemble a binary or object file section into human-readable assembly instructions. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE disassemble", read_only_hint = true)
)]
async fn re_disassemble(
    &self,
    Parameters(args): Parameters<ReDisassembleArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let section = args.section.clone();
    let architecture = args.architecture.clone();
    let start_addr = args.start_addr.clone();
    let count = args.count;
    let out = tokio::task::spawn_blocking(move || {
        re_disassemble_work(path, section, architecture, start_addr, count)
    })
    .await
    .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 4. re_symbols
// ---------------------------------------------------------------------------

#[tool(
    description = "List symbols (functions, globals, undefined references) exported by a binary or object file. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE symbols", read_only_hint = true)
)]
async fn re_symbols(
    &self,
    Parameters(args): Parameters<ReSymbolsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let symbol_type = args.symbol_type.clone();
    let out = tokio::task::spawn_blocking(move || re_symbols_work(path, symbol_type))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 5. re_sections
// ---------------------------------------------------------------------------

#[tool(
    description = "List all sections in a binary or object file with their names, sizes, virtual addresses, and flags. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE sections", read_only_hint = true)
)]
async fn re_sections(
    &self,
    Parameters(args): Parameters<ReSectionsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_sections_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 6. re_dependencies
// ---------------------------------------------------------------------------

#[tool(
    description = "List dynamic library dependencies (shared objects / DLLs) required by a binary. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE dependencies", read_only_hint = true)
)]
async fn re_dependencies(
    &self,
    Parameters(args): Parameters<ReDependenciesArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_dependencies_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 7. re_hexdump
// ---------------------------------------------------------------------------

#[tool(
    description = "Produce an annotated hex dump of a binary file at an optional offset and length. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE hexdump", read_only_hint = true)
)]
async fn re_hexdump(
    &self,
    Parameters(args): Parameters<ReHexdumpArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let length = args.length;
    let width = args.width;
    let out = tokio::task::spawn_blocking(move || re_hexdump_work(path, offset, length, width))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 8. re_entropy
// ---------------------------------------------------------------------------

#[tool(
    description = "Compute Shannon entropy of a binary file, optionally broken into fixed-size blocks. High entropy regions indicate compression or encryption. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE entropy", read_only_hint = true)
)]
async fn re_entropy(
    &self,
    Parameters(args): Parameters<ReEntropyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let block_size = args.block_size;
    let out = tokio::task::spawn_blocking(move || re_entropy_work(path, block_size))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 9. re_carve
// ---------------------------------------------------------------------------

#[tool(
    description = "Carve a byte range out of a binary file and write it to an output file. Useful for extracting embedded payloads. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE carve", read_only_hint = false)
)]
async fn re_carve(
    &self,
    Parameters(args): Parameters<ReCarveArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let length = args.length;
    let output = args.output.as_deref().map(|s| PathBuf::from(s));
    let out = tokio::task::spawn_blocking(move || re_carve_work(path, offset, length, output))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 10. re_pattern_search
// ---------------------------------------------------------------------------

#[tool(
    description = "Search a binary file for a byte pattern (hex string or regex) and return all match offsets. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE pattern search", read_only_hint = true)
)]
async fn re_pattern_search(
    &self,
    Parameters(args): Parameters<RePatternSearchArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let pattern = args.pattern.clone();
    let max_results = args.max_results;
    let out =
        tokio::task::spawn_blocking(move || re_pattern_search_work(path, pattern, max_results))
            .await
            .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 11. re_struct_parse
// ---------------------------------------------------------------------------

#[tool(
    description = "Parse a fixed-layout binary structure at a given offset using a format descriptor and return the field values. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE struct parse", read_only_hint = true)
)]
async fn re_struct_parse(
    &self,
    Parameters(args): Parameters<ReStructParseArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let offset = args.offset;
    let format = args.format.clone();
    let out = tokio::task::spawn_blocking(move || re_struct_parse_work(path, offset, format))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 12. re_diff_binary
// ---------------------------------------------------------------------------

#[tool(
    description = "Byte-level diff two binary files and report differing offset ranges and changed byte counts. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE diff binary", read_only_hint = true)
)]
async fn re_diff_binary(
    &self,
    Parameters(args): Parameters<ReDiffBinaryArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path_a = match self.resolve_path(&args.path_a) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let path_b = match self.resolve_path(&args.path_b) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_diff_binary_work(path_a, path_b))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 13. re_strace
// ---------------------------------------------------------------------------

#[tool(
    description = "Execute a binary under strace and capture its system-call trace. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE strace", read_only_hint = false)
)]
async fn re_strace(
    &self,
    Parameters(args): Parameters<ReStraceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let extra_args = args.extra_args.clone();
    let timeout_secs = args.timeout_secs;
    let out = tokio::task::spawn_blocking(move || re_strace_work(path, extra_args, timeout_secs))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}

// ---------------------------------------------------------------------------
// 14. re_binwalk_scan
// ---------------------------------------------------------------------------

#[tool(
    description = "Scan a firmware image or binary blob with binwalk to identify embedded file signatures, compression, and known headers. Requires an experimental-lab deployment. Blocked on public profile.",
    annotations(title = "RE binwalk scan", read_only_hint = true)
)]
async fn re_binwalk_scan(
    &self,
    Parameters(args): Parameters<ReBinwalkScanArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let path = match self.resolve_path(&args.path) {
        Ok(p) => p,
        Err(e) => return Self::tool_text(format!("Error: {e}")),
    };
    let out = tokio::task::spawn_blocking(move || re_binwalk_scan_work(path))
        .await
        .unwrap_or_else(|e| format!("Error: task failed: {e}"));
    Self::tool_text(out)
}
