// Sandbox tool argument structs.
//
// Included at module scope via include!() in the parent file.
// NOTE: `JsonSchema`, `Deserialize`, and `schemars` are already in scope from
// the parent — do NOT add `use` statements here.

// ---------------------------------------------------------------------------
// 1. sandbox_exec
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxExecArgs {
    /// Command to execute (must be a bare executable name, not a path).
    command: String,
    /// Command arguments.
    #[serde(default)]
    exec_args: Vec<String>,
    /// Timeout in seconds (default 30, max 600).
    #[serde(default)]
    timeout_secs: Option<u64>,
    /// Max output bytes to capture (default 65536, max 1048576).
    #[serde(default)]
    max_output_bytes: Option<usize>,
}

// ---------------------------------------------------------------------------
// 2. sandbox_script
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxScriptArgs {
    /// Script source code to execute.
    script: String,
    /// Language / interpreter: python3, node, bash, ruby, or perl.
    language: String,
    /// Timeout in seconds (default 30, max 600).
    #[serde(default)]
    timeout_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// 3. sandbox_eval_code
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxEvalCodeArgs {
    /// Code snippet to evaluate.
    code: String,
    /// Language / interpreter: python3, node, bash, ruby, or perl.
    language: String,
    /// Optional data to pipe to the process via stdin.
    #[serde(default)]
    input_data: Option<String>,
    /// Timeout in seconds (default 30, max 600).
    #[serde(default)]
    timeout_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// 4. sandbox_docker_run
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxDockerRunArgs {
    /// Docker image to run (e.g. "python:3.11-slim").
    image: String,
    /// Shell command to run inside the container.
    command: String,
    /// Timeout in seconds (default 30, max 600).
    #[serde(default)]
    timeout_secs: Option<u64>,
    /// Memory limit in MB for the container (default 256, range 16–4096).
    #[serde(default)]
    memory_mb: Option<u64>,
}

// ---------------------------------------------------------------------------
// 5. sandbox_checksum
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxChecksumArgs {
    /// Path to the file to checksum, relative to the project root.
    path: String,
    /// Hash algorithm: sha256 (default), sha1, md5, or blake3.
    #[serde(default)]
    algorithm: Option<String>,
}

// ---------------------------------------------------------------------------
// 6. sandbox_diff
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxDiffArgs {
    /// Path to the first file, relative to the project root.
    path_a: String,
    /// Path to the second file to compare against, relative to the project root.
    path_b: String,
    /// Number of context lines around each change in the unified diff (default 3).
    #[serde(default)]
    context_lines: Option<usize>,
}

// ---------------------------------------------------------------------------
// 7. sandbox_file_watch
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxFileWatchArgs {
    /// Path to the file to watch for changes, relative to the project root.
    path: String,
    /// Command to run when a change is detected (must be a bare executable name).
    command: String,
    /// Timeout in seconds to poll before returning (default 30, max 600).
    #[serde(default)]
    timeout_secs: Option<u64>,
}

// ---------------------------------------------------------------------------
// 8. sandbox_network_request
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SandboxNetworkRequestArgs {
    /// Target URL (must start with http:// or https://).
    url: String,
    /// HTTP method: GET (default), POST, PUT, DELETE, PATCH, HEAD, or OPTIONS.
    #[serde(default)]
    method: Option<String>,
    /// Extra HTTP headers in "Name: Value" format.
    #[serde(default)]
    headers: Vec<String>,
    /// Request body (used with POST, PUT, PATCH).
    #[serde(default)]
    body: Option<String>,
    /// Request timeout in seconds (default 30, max 300).
    #[serde(default)]
    timeout_secs: Option<u64>,
}
