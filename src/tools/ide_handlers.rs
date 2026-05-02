// IDE tool handler methods.
//
// Included inside `#[tool_router] impl MintMcp { ... }` via include!().
// NOTE: No `impl` block, no `use` statements — raw method bodies only.

// ===========================================================================
// Zed Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. zed_open_file
// ---------------------------------------------------------------------------

#[tool(
    description = "Open a file in the Zed editor, optionally navigating to a specific line. Runs `zed <path>:<line>`. Requires Zed on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: open file", read_only_hint = false)
)]
async fn zed_open_file(
    &self,
    Parameters(args): Parameters<ZedOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || zed_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 2. zed_get_diagnostics
// ---------------------------------------------------------------------------

#[tool(
    description = "Retrieve diagnostics from the Zed editor. Zed does not yet expose diagnostics via CLI; returns a placeholder with guidance to use the LSP bridge for live diagnostics. Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: get diagnostics", read_only_hint = true)
)]
async fn zed_get_diagnostics(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || zed_get_diagnostics_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 3. zed_search_workspace
// ---------------------------------------------------------------------------

#[tool(
    description = "Trigger a workspace-wide search in the Zed editor via `zed --search <query>`. Returns a note that the search was triggered; results appear in the Zed UI. Requires Zed on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: search workspace", read_only_hint = true)
)]
async fn zed_search_workspace(
    &self,
    Parameters(args): Parameters<ZedSearchWorkspaceArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "query": args.query,
    });
    let out = tokio::task::spawn_blocking(move || zed_search_workspace_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 4. zed_context_attach
// ---------------------------------------------------------------------------

#[tool(
    description = "Attach a file to the Zed AI session context. Opens the file in Zed and reads its content for use as AI context. Returns instructions for using @<path> in the Zed AI Assistant panel. Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "Zed: context attach", read_only_hint = false)
)]
async fn zed_context_attach(
    &self,
    Parameters(args): Parameters<ZedContextAttachArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "role": args.role.unwrap_or_else(|| "reference".to_string()),
    });
    let out = tokio::task::spawn_blocking(move || zed_context_attach_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ===========================================================================
// Cursor Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 5. cursor_open_file
// ---------------------------------------------------------------------------

#[tool(
    description = "Open a file in the Cursor editor, optionally navigating to a specific line. Runs `cursor <path>:<line>`. Requires Cursor on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Cursor: open file", read_only_hint = false)
)]
async fn cursor_open_file(
    &self,
    Parameters(args): Parameters<CursorOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || cursor_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 6. cursor_composer_send
// ---------------------------------------------------------------------------

#[tool(
    description = "Send content to the Cursor Composer panel (Ctrl+I / Cmd+I). Opens the associated file in Cursor and returns Composer instructions. Requires Cursor on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "Cursor: composer send", read_only_hint = false)
)]
async fn cursor_composer_send(
    &self,
    Parameters(args): Parameters<CursorComposerSendArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "content": args.content,
        "path": args.path,
        "instruction": args.instruction,
    });
    let out = tokio::task::spawn_blocking(move || cursor_composer_send_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ===========================================================================
// VS Code Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 7. vscode_open_diff
// ---------------------------------------------------------------------------

#[tool(
    description = "Open a side-by-side diff view in VS Code between two files. Runs `code --diff <left> <right>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: open diff", read_only_hint = false)
)]
async fn vscode_open_diff(
    &self,
    Parameters(args): Parameters<VscodeOpenDiffArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "left": args.left,
        "right": args.right,
    });
    let out = tokio::task::spawn_blocking(move || vscode_open_diff_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 8. vscode_open_file
// ---------------------------------------------------------------------------

#[tool(
    description = "Open a file in VS Code, optionally jumping to a specific line via `code --goto <path>:<line>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: open file", read_only_hint = false)
)]
async fn vscode_open_file(
    &self,
    Parameters(args): Parameters<VscodeOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "line": args.line,
        "column": args.column,
    });
    let out = tokio::task::spawn_blocking(move || vscode_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 9. vscode_list_extensions
// ---------------------------------------------------------------------------

#[tool(
    description = "List installed VS Code extensions via `code --list-extensions --show-names`. Returns an array of {id, name} objects. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: list extensions", read_only_hint = true)
)]
async fn vscode_list_extensions(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || vscode_list_extensions_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 10. vscode_install_extension
// ---------------------------------------------------------------------------

#[tool(
    description = "Install a VS Code extension by its marketplace identifier via `code --install-extension <id>`. Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: install extension", read_only_hint = false)
)]
async fn vscode_install_extension(
    &self,
    Parameters(args): Parameters<VscodeInstallExtensionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "extension_id": args.extension_id,
    });
    let out = tokio::task::spawn_blocking(move || vscode_install_extension_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 11. vscode_get_diagnostics
// ---------------------------------------------------------------------------

#[tool(
    description = "Read VS Code diagnostics for a file or workspace. VS Code does not expose live diagnostics via CLI; returns best-effort output from `code --status` and guidance to use the Problems panel (Ctrl+Shift+M). Requires VS Code CLI (`code`) on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "VS Code: get diagnostics", read_only_hint = true)
)]
async fn vscode_get_diagnostics(
    &self,
    Parameters(args): Parameters<VscodeGetDiagnosticsArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "severity": args.severity,
    });
    let out = tokio::task::spawn_blocking(move || vscode_get_diagnostics_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ===========================================================================
// JetBrains / Android Studio Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 12. jetbrains_open_file
// ---------------------------------------------------------------------------

#[tool(
    description = "Open a file in a JetBrains IDE (idea/android-studio/clion/pycharm/rustrover), optionally at a specific line. Requires the IDE CLI on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: open file", read_only_hint = false)
)]
async fn jetbrains_open_file(
    &self,
    Parameters(args): Parameters<JetbrainsOpenFileArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "ide": args.ide,
        "line": args.line,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_open_file_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 13. jetbrains_adb_shell
// ---------------------------------------------------------------------------

#[tool(
    description = "Execute an ADB shell command on a connected Android device via `adb shell <command>`. Optionally targets a specific device by serial. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: ADB shell", read_only_hint = false)
)]
async fn jetbrains_adb_shell(
    &self,
    Parameters(args): Parameters<JetbrainsAdbShellArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "command": args.command,
        "device": args.device,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_adb_shell_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 14. jetbrains_device_list
// ---------------------------------------------------------------------------

#[tool(
    description = "List connected Android devices via `adb devices -l`. Returns an array of {serial, state, model} records. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: ADB device list", read_only_hint = true)
)]
async fn jetbrains_device_list(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || jetbrains_device_list_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 15. jetbrains_logcat_tail
// ---------------------------------------------------------------------------

#[tool(
    description = "Read the last N lines of Android Logcat via `adb logcat -d -t <n>`. Optionally filters by tag expression and targets a specific device serial. Requires ADB on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: logcat tail", read_only_hint = true)
)]
async fn jetbrains_logcat_tail(
    &self,
    Parameters(args): Parameters<JetbrainsLogcatTailArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "device": args.device,
        "n": args.n,
        "filter": args.filter,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_logcat_tail_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 16. jetbrains_lsp_code_action
// ---------------------------------------------------------------------------

#[tool(
    description = "Execute an LSP code action (quick fix, refactor, source) via the JetBrains built-in REST API at the given file position. Requires the IDE on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: LSP code action", read_only_hint = false)
)]
async fn jetbrains_lsp_code_action(
    &self,
    Parameters(args): Parameters<JetbrainsLspCodeActionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "ide": args.ide,
        "line": args.line,
        "column": args.column,
        "action_kind": args.action_kind,
        "port": args.port,
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_lsp_code_action_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 17. jetbrains_run_configuration
// ---------------------------------------------------------------------------

#[tool(
    description = "Trigger a saved JetBrains run/debug configuration via the IDE CLI. Supports idea/android-studio/clion/pycharm/rustrover/webstorm/goland. Requires the IDE on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "JetBrains: run configuration", read_only_hint = false)
)]
async fn jetbrains_run_configuration(
    &self,
    Parameters(args): Parameters<JetbrainsRunConfigurationArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "ide": args.ide,
        "config_name": args.config_name,
        "debug": args.debug.unwrap_or(false),
    });
    let out = tokio::task::spawn_blocking(move || jetbrains_run_configuration_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ===========================================================================
// Generic IDE Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 18. ide_detect
// ---------------------------------------------------------------------------

#[tool(
    description = "Detect running IDEs on this machine by scanning /proc on Linux. Returns an array of {name, pid} records for known IDEs (zed, code, cursor, idea, android-studio, clion, pycharm, rustrover, vim, nvim, emacs, …). Requires a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: detect running", read_only_hint = true)
)]
async fn ide_detect(
    &self,
    Parameters(_args): Parameters<EmptyArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({});
    let out = tokio::task::spawn_blocking(move || ide_detect_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 19. ide_format_document
// ---------------------------------------------------------------------------

#[tool(
    description = "Format a file using rustfmt, prettier, black, or clang-format. Runs the formatter in-place on the specified path. Requires the formatter on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: format document", read_only_hint = false)
)]
async fn ide_format_document(
    &self,
    Parameters(args): Parameters<IdeFormatDocumentArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "path": args.path,
        "formatter": args.formatter,
    });
    let out = tokio::task::spawn_blocking(move || ide_format_document_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 20. ide_run_test
// ---------------------------------------------------------------------------

#[tool(
    description = "Run tests for a file using the language's test runner: cargo test (rust), pytest (python), jest (javascript/typescript). Returns {success, output}. Requires the test runner on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: run test", read_only_hint = false)
)]
async fn ide_run_test(
    &self,
    Parameters(args): Parameters<IdeRunTestArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "language": args.language,
        "path": args.path.unwrap_or_default(),
        "line": args.line,
    });
    let out = tokio::task::spawn_blocking(move || ide_run_test_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 21. ide_goto_definition
// ---------------------------------------------------------------------------

#[tool(
    description = "Find a symbol's definition location by running `grep -rn <symbol> <path>`. Returns up to 50 matching {file, line, context} records. Requires grep on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: go to definition", read_only_hint = false)
)]
async fn ide_goto_definition(
    &self,
    Parameters(args): Parameters<IdeGotoDefinitionArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "symbol": args.symbol,
        "path": args.path,
    });
    let out = tokio::task::spawn_blocking(move || ide_goto_definition_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}

// ---------------------------------------------------------------------------
// 22. ide_find_references
// ---------------------------------------------------------------------------

#[tool(
    description = "Find all references to a symbol by running `grep -rn <symbol> <path>`. Returns up to 50 matching {file, line, context} records and a total count. Requires grep on PATH and a trusted deployment. Blocked on public profile.",
    annotations(title = "IDE: find references", read_only_hint = false)
)]
async fn ide_find_references(
    &self,
    Parameters(args): Parameters<IdeFindReferencesArgs>,
) -> Result<CallToolResult, McpError> {
    if let Err(e) = self.require_trusted_deployment() {
        return Self::tool_text(e);
    }
    let input = serde_json::json!({
        "symbol": args.symbol,
        "path": args.path,
    });
    let out = tokio::task::spawn_blocking(move || ide_find_references_work(input))
        .await
        .unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let text = serde_json::to_string_pretty(&out).unwrap_or_else(|_| out.to_string());
    Self::tool_text(text)
}
