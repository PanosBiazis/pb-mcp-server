// IDE tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ===========================================================================
// Zed Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. zed_open_file
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ZedOpenFileArgs {
    /// File path to open in Zed (relative to project root or absolute).
    path: String,
    /// Line number to navigate to (1-based). Omit to open without scrolling.
    #[serde(default)]
    line: Option<u32>,
    /// Column number to navigate to (1-based). Only used when line is provided.
    #[serde(default)]
    column: Option<u32>,
}

// ---------------------------------------------------------------------------
// 2. zed_get_diagnostics  — no meaningful input
// ---------------------------------------------------------------------------

// Uses EmptyArgs (defined in mcp_args.rs). No struct needed here.

// ---------------------------------------------------------------------------
// 3. zed_search_workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ZedSearchWorkspaceArgs {
    /// Search query to submit to Zed's workspace-wide search.
    query: String,
}

// ---------------------------------------------------------------------------
// 4. zed_context_attach
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct ZedContextAttachArgs {
    /// File path to attach to the Zed AI session context (relative to project root).
    path: String,
    /// Context role for the file (e.g. "reference", "source", "test"). Default: "reference".
    #[serde(default)]
    role: Option<String>,
}

// ===========================================================================
// Cursor Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 5. cursor_open_file
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct CursorOpenFileArgs {
    /// File path to open in the Cursor editor (relative to project root or absolute).
    path: String,
    /// Line number to navigate to (1-based). Omit to open at the top.
    #[serde(default)]
    line: Option<u32>,
    /// Column number to navigate to (1-based). Only used when line is provided.
    #[serde(default)]
    column: Option<u32>,
}

// ---------------------------------------------------------------------------
// 6. cursor_composer_send
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct CursorComposerSendArgs {
    /// Code or text content to send to the Cursor Composer panel.
    content: String,
    /// Optional file path associated with the content (opens the file in Cursor).
    #[serde(default)]
    path: Option<String>,
    /// Instruction for the Composer (e.g. "Review and improve the following code").
    #[serde(default)]
    instruction: Option<String>,
}

// ===========================================================================
// VS Code Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 7. vscode_open_diff
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct VscodeOpenDiffArgs {
    /// Left-hand file path for the diff view (relative to project root or absolute).
    left: String,
    /// Right-hand file path for the diff view (relative to project root or absolute).
    right: String,
}

// ---------------------------------------------------------------------------
// 8. vscode_open_file
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct VscodeOpenFileArgs {
    /// File path to open in VS Code (relative to project root or absolute).
    path: String,
    /// Line number to navigate to (1-based). Omit to open without scrolling.
    #[serde(default)]
    line: Option<u32>,
    /// Column number to navigate to (1-based). Only used when line is provided.
    #[serde(default)]
    column: Option<u32>,
}

// ---------------------------------------------------------------------------
// 9. vscode_list_extensions  — no meaningful input
// ---------------------------------------------------------------------------

// Uses EmptyArgs (defined in mcp_args.rs). No struct needed here.

// ---------------------------------------------------------------------------
// 10. vscode_install_extension
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct VscodeInstallExtensionArgs {
    /// VS Code extension identifier to install (e.g. "rust-lang.rust-analyzer").
    extension_id: String,
}

// ---------------------------------------------------------------------------
// 11. vscode_get_diagnostics
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct VscodeGetDiagnosticsArgs {
    /// File or directory path to scope diagnostics to (optional; omit for workspace-wide).
    #[serde(default)]
    path: Option<String>,
    /// Severity filter: "all" (default), "error", or "warning".
    #[serde(default)]
    severity: Option<String>,
}

// ===========================================================================
// JetBrains / Android Studio Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 12. jetbrains_open_file
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct JetbrainsOpenFileArgs {
    /// File path to open (relative to project root or absolute).
    path: String,
    /// JetBrains IDE to use: "idea", "android-studio", "clion", "pycharm", or "rustrover".
    ide: String,
    /// Line number to navigate to (1-based). Omit to open at the top.
    #[serde(default)]
    line: Option<u32>,
}

// ---------------------------------------------------------------------------
// 13. jetbrains_adb_shell
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct JetbrainsAdbShellArgs {
    /// ADB shell command to execute (e.g. "ls /data/local/tmp").
    command: String,
    /// Target device serial number (from `adb devices`). Omit for the only connected device.
    #[serde(default)]
    device: Option<String>,
}

// ---------------------------------------------------------------------------
// 14. jetbrains_device_list  — no meaningful input
// ---------------------------------------------------------------------------

// Uses EmptyArgs (defined in mcp_args.rs). No struct needed here.

// ---------------------------------------------------------------------------
// 15. jetbrains_logcat_tail
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct JetbrainsLogcatTailArgs {
    /// Target device serial number. Omit for the only connected device.
    #[serde(default)]
    device: Option<String>,
    /// Number of recent logcat lines to retrieve (1–10000, default 100).
    #[serde(default)]
    n: Option<u32>,
    /// Optional logcat filter expression (e.g. "MyTag:D *:S").
    #[serde(default)]
    filter: Option<String>,
}

// ---------------------------------------------------------------------------
// 16. jetbrains_lsp_code_action
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct JetbrainsLspCodeActionArgs {
    /// File path where the code action should be applied (relative to project root).
    path: String,
    /// JetBrains IDE to target: "idea", "android-studio", "clion", "pycharm",
    /// "rustrover", "webstorm", or "goland".
    ide: String,
    /// Line number for the code action position (1-based, default 1).
    #[serde(default)]
    line: Option<i64>,
    /// Column number for the code action position (1-based, default 1).
    #[serde(default)]
    column: Option<i64>,
    /// Code action kind: "quickfix" (default), "refactor", or "source".
    #[serde(default)]
    action_kind: Option<String>,
    /// JetBrains built-in server port (default 63342).
    #[serde(default)]
    port: Option<i64>,
}

// ---------------------------------------------------------------------------
// 17. jetbrains_run_configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct JetbrainsRunConfigurationArgs {
    /// JetBrains IDE to use: "idea", "android-studio", "clion", "pycharm",
    /// "rustrover", "webstorm", or "goland".
    ide: String,
    /// Name of the saved run/debug configuration to execute.
    config_name: String,
    /// Set true to run in debug mode instead of run mode (default false).
    #[serde(default)]
    debug: Option<bool>,
}

// ===========================================================================
// Generic IDE Tools
// ===========================================================================

// ---------------------------------------------------------------------------
// 18. ide_detect  — no meaningful input
// ---------------------------------------------------------------------------

// Uses EmptyArgs (defined in mcp_args.rs). No struct needed here.

// ---------------------------------------------------------------------------
// 19. ide_format_document
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct IdeFormatDocumentArgs {
    /// File path to format (relative to project root or absolute).
    path: String,
    /// Formatter to use: "rustfmt", "prettier", "black", or "clang-format".
    formatter: String,
}

// ---------------------------------------------------------------------------
// 20. ide_run_test
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct IdeRunTestArgs {
    /// Language / test runner: "rust", "python" / "py", "javascript" / "typescript" / "js" / "ts".
    language: String,
    /// File path for the test file (used by pytest / jest; omit for `cargo test`).
    #[serde(default)]
    path: Option<String>,
    /// Line number hint (reserved for future use).
    #[serde(default)]
    line: Option<i64>,
}

// ---------------------------------------------------------------------------
// 21. ide_goto_definition
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct IdeGotoDefinitionArgs {
    /// Symbol name to locate (searched via grep -rn).
    symbol: String,
    /// Directory or file path to search within (relative to project root).
    path: String,
}

// ---------------------------------------------------------------------------
// 22. ide_find_references
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct IdeFindReferencesArgs {
    /// Symbol name whose references should be found (searched via grep -rn).
    symbol: String,
    /// Directory or file path to search within (relative to project root).
    path: String,
}
