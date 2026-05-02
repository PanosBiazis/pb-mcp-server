// Argument structs for tools wired via tools_wired.rs.
// include!("tool_args_wired.rs") is called at module scope in main.rs.

// NOTE: JsonSchema and Deserialize are already in scope from main.rs imports.

// ── AI / Ollama ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiOllamaGenerateArgs {
    #[schemars(description = "Ollama model tag (e.g. mistral, llama3:8b).")]
    pub model: String,
    #[schemars(description = "Prompt text sent to the model.")]
    pub prompt: String,
    #[schemars(description = "Optional system prompt prepended before the user turn.")]
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[schemars(description = "Max tokens to generate (0 = model default).")]
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiOllamaEmbedArgs {
    #[schemars(description = "Ollama embedding model tag (e.g. nomic-embed-text).")]
    pub model: String,
    #[schemars(description = "Text to embed.")]
    pub input: String,
}

// ── Network ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebDnsLookupArgs {
    #[schemars(description = "Hostname to look up (e.g. example.com).")]
    pub hostname: String,
    #[schemars(
        description = "Record types to query (e.g. [\"A\",\"MX\",\"TXT\"]); empty = A + AAAA."
    )]
    #[serde(default)]
    pub record_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebTlsInspectArgs {
    #[schemars(description = "Hostname to connect to (SNI).")]
    pub host: String,
    #[schemars(description = "TCP port (default 443).")]
    #[serde(default = "default_tls_port")]
    pub port: u16,
}

fn default_tls_port() -> u16 {
    443
}

// ── Notify ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NotifyDesktopArgs {
    #[schemars(description = "Notification title.")]
    pub title: String,
    #[schemars(description = "Notification body text.")]
    pub body: String,
    #[schemars(description = "Urgency: low | normal | critical (default normal).")]
    #[serde(default)]
    pub urgency: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NotifyEmailArgs {
    #[schemars(description = "Recipient email address.")]
    pub to: String,
    #[schemars(description = "Email subject line.")]
    pub subject: String,
    #[schemars(description = "Plain-text email body.")]
    pub body: String,
    #[schemars(description = "From address (default noreply@localhost).")]
    #[serde(default)]
    pub from: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NotifyWebhookArgs {
    #[schemars(description = "Webhook URL (https:// strongly recommended).")]
    pub url: String,
    #[schemars(description = "JSON payload string to POST.")]
    pub payload: String,
    #[schemars(description = "HTTP method (default POST).")]
    #[serde(default)]
    pub method: Option<String>,
    #[schemars(description = "Block internal/private IPs (SSRF guard, default true).")]
    #[serde(default = "default_block_internal")]
    pub block_internal: bool,
}

fn default_block_internal() -> bool {
    true
}

// ── System ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemDiskUsageArgs {
    #[schemars(description = "Path under project root to check disk usage for.")]
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemEnvGetArgs {
    #[schemars(description = "Environment variable name.")]
    pub name: String,
}

// ── Database ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DbSqliteQueryArgs {
    #[schemars(description = "Path to .sqlite/.db file under project root.")]
    pub db_path: String,
    #[schemars(description = "SQL SELECT query to execute.")]
    pub query: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DbSqliteExecArgs {
    #[schemars(description = "Path to .sqlite/.db file under project root.")]
    pub db_path: String,
    #[schemars(description = "SQL statement to execute (INSERT/UPDATE/DELETE/CREATE).")]
    pub statement: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DbSqliteSchemaArgs {
    #[schemars(description = "Path to .sqlite/.db file under project root.")]
    pub db_path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DbJsonQueryArgs {
    #[schemars(description = "Path to JSON file under project root.")]
    pub path: String,
    #[schemars(description = "JSONPath expression (e.g. $.items[*].name).")]
    pub query: String,
}

// ── Web extended ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebApiCallArgs {
    #[schemars(description = "HTTP method: GET | POST | PUT | PATCH | DELETE.")]
    pub method: String,
    #[schemars(description = "Full URL.")]
    pub url: String,
    #[schemars(description = "Request body (for POST/PUT/PATCH).")]
    #[serde(default)]
    pub body: Option<String>,
    #[schemars(description = "Extra headers as key=value pairs.")]
    #[serde(default)]
    pub headers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebDownloadFileArgs {
    #[schemars(description = "URL to download.")]
    pub url: String,
    #[schemars(description = "Destination path under project root.")]
    pub dest: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebScreenshotArgs {
    #[schemars(description = "URL to screenshot.")]
    pub url: String,
    #[schemars(
        description = "Output file path relative to project root (e.g. screenshots/page.png)."
    )]
    #[serde(default)]
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebSearchArgs {
    #[schemars(description = "Search query string.")]
    pub query: String,
    #[schemars(description = "Max results to return (default 10).")]
    #[serde(default = "default_web_search_max")]
    pub max_results: usize,
}

fn default_web_search_max() -> usize {
    10
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebRssFetchArgs {
    #[schemars(description = "RSS/Atom feed URL.")]
    pub url: String,
    #[schemars(description = "Max feed items to return (default 20).")]
    #[serde(default = "default_rss_max")]
    pub max_items: usize,
}

fn default_rss_max() -> usize {
    20
}

// ── Convert ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertArchiveExtractArgs {
    #[schemars(description = "Archive file path under project root.")]
    pub path: String,
    #[schemars(description = "Destination directory under project root.")]
    pub dest: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertArchiveListArgs {
    #[schemars(description = "Archive file path under project root.")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertBinaryHexdumpArgs {
    #[schemars(description = "Binary file path under project root.")]
    pub path: String,
    #[schemars(description = "Max bytes to dump (default 4096).")]
    #[serde(default = "default_hexdump_max")]
    pub max_bytes: usize,
}

fn default_hexdump_max() -> usize {
    4096
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertDocumentArgs {
    #[schemars(description = "Input document path under project root.")]
    pub input: String,
    #[schemars(description = "Output document path under project root.")]
    pub output: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertEncodingArgs {
    #[schemars(description = "Text to re-encode.")]
    pub text: String,
    #[schemars(description = "Source encoding (e.g. latin-1, cp1252).")]
    pub from_enc: String,
    #[schemars(description = "Target encoding (e.g. utf-8).")]
    pub to_enc: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertPdfExtractArgs {
    #[schemars(description = "PDF file path under project root.")]
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConvertQrGenerateArgs {
    #[schemars(description = "Text to encode as QR.")]
    pub text: String,
    #[schemars(description = "Output PNG path relative to project root.")]
    #[serde(default)]
    pub output_path: Option<String>,
}

// ── Agent sync ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentSyncConfirmArgs {
    #[schemars(description = "Sync ID returned by a previous agent_sync_all call.")]
    pub sync_id: String,
}
