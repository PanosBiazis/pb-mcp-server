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

// ── Net Tools ─────────────────────────────────────────────────────────────────

/// ICMP ping a host and report round-trip statistics.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetPingArgs {
    #[schemars(description = "Hostname or IP address to ping.")]
    pub host: String,
    #[schemars(description = "Number of ICMP echo requests to send (default 4, max 100).")]
    #[serde(default)]
    pub count: Option<u64>,
}

/// Trace the network path to a host hop-by-hop.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetTracerouteArgs {
    #[schemars(description = "Hostname or IP address to trace the route to.")]
    pub host: String,
    #[schemars(description = "Maximum number of hops before giving up (default 30).")]
    #[serde(default)]
    pub max_hops: Option<u64>,
}

/// Scan TCP ports on a remote host.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetPortScanArgs {
    #[schemars(description = "Hostname or IP address to scan.")]
    pub host: String,
    #[schemars(description = "List of TCP port numbers to scan (default: common ports).")]
    #[serde(default)]
    pub ports: Option<Vec<u64>>,
    #[schemars(description = "Per-port connection timeout in milliseconds (default 1000).")]
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Perform a DNS lookup for one or more record types.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetDnsLookupArgs {
    #[schemars(description = "Domain name to query (e.g. example.com).")]
    pub domain: String,
    #[schemars(description = "DNS record type to query: A | AAAA | MX | TXT | CNAME (default A).")]
    #[serde(default)]
    pub record_type: Option<String>,
}

/// Query WHOIS registration data for a domain.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetWhoisArgs {
    #[schemars(description = "Domain name to look up (e.g. example.com).")]
    pub domain: String,
}

/// Inspect the TLS/SSL certificate chain of a remote host.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetTlsInspectArgs {
    #[schemars(description = "Hostname to connect to (used for SNI).")]
    pub host: String,
    #[schemars(description = "TCP port to connect on (default 443).")]
    #[serde(default)]
    pub port: Option<u64>,
}

/// Run an HTTP benchmark (load test) against a URL.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetHttpBenchmarkArgs {
    #[schemars(description = "Target URL to benchmark.")]
    pub url: String,
    #[schemars(description = "Total number of HTTP requests to send (default 100).")]
    #[serde(default)]
    pub requests: Option<u64>,
    #[schemars(description = "Number of concurrent connections (default 10).")]
    #[serde(default)]
    pub concurrency: Option<u64>,
}

/// Scan the local network for hosts using ARP.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetArpScanArgs {
    #[schemars(description = "Network interface to use for ARP scanning (default: auto-detect).")]
    #[serde(default)]
    pub interface: Option<String>,
}

/// Scan for nearby Wi-Fi networks.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetWifiScanArgs {
    #[schemars(description = "Wireless interface to scan on (default: auto-detect).")]
    #[serde(default)]
    pub interface: Option<String>,
}

/// Measure network bandwidth to a target host.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetBandwidthTestArgs {
    #[schemars(description = "Target hostname or IP for the bandwidth test.")]
    pub target: String,
    #[schemars(description = "Duration of the test in seconds (default 10).")]
    #[serde(default)]
    pub duration_s: Option<u64>,
}

/// Run a quick nmap port/service scan.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetNmapQuickArgs {
    #[schemars(description = "Target hostname, IP, or CIDR range for nmap.")]
    pub target: String,
    #[schemars(description = "Port specification string (e.g. \"22,80,443\" or \"1-1024\"); default top 100.")]
    #[serde(default)]
    pub ports: Option<String>,
}

/// Execute a command on a remote host via SSH.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSshExecArgs {
    #[schemars(description = "Remote hostname or IP address.")]
    pub host: String,
    #[schemars(description = "SSH username.")]
    pub user: String,
    #[schemars(description = "Shell command to execute on the remote host.")]
    pub command: String,
    #[schemars(description = "SSH port (default 22).")]
    #[serde(default)]
    pub port: Option<u64>,
    #[schemars(description = "Path to the private key file for authentication.")]
    #[serde(default)]
    pub key_path: Option<String>,
}

/// Upload a local file to a remote host via SFTP.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSftpUploadArgs {
    #[schemars(description = "Remote hostname or IP address.")]
    pub host: String,
    #[schemars(description = "SSH/SFTP username.")]
    pub user: String,
    #[schemars(description = "Local file path to upload.")]
    pub local_path: String,
    #[schemars(description = "Destination path on the remote host.")]
    pub remote_path: String,
    #[schemars(description = "SSH port (default 22).")]
    #[serde(default)]
    pub port: Option<u64>,
}

/// Download a file from a remote host via SFTP.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSftpDownloadArgs {
    #[schemars(description = "Remote hostname or IP address.")]
    pub host: String,
    #[schemars(description = "SSH/SFTP username.")]
    pub user: String,
    #[schemars(description = "Path of the file on the remote host to download.")]
    pub remote_path: String,
    #[schemars(description = "Local destination path for the downloaded file.")]
    pub local_path: String,
    #[schemars(description = "SSH port (default 22).")]
    #[serde(default)]
    pub port: Option<u64>,
}

/// Wake a machine on the LAN using Wake-on-LAN magic packet.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetWakeOnLanArgs {
    #[schemars(description = "MAC address of the target machine (e.g. AA:BB:CC:DD:EE:FF).")]
    pub mac_address: String,
    #[schemars(description = "Broadcast address to send the magic packet to (default 255.255.255.255).")]
    #[serde(default)]
    pub broadcast: Option<String>,
}

/// List available serial/COM ports on this system.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSerialListArgs {}

/// Send data to a serial port.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSerialSendArgs {
    #[schemars(description = "Serial port path (e.g. /dev/ttyUSB0 or COM3).")]
    pub port: String,
    #[schemars(description = "Data string to write to the serial port.")]
    pub data: String,
    #[schemars(description = "Baud rate (default 9600).")]
    #[serde(default)]
    pub baud: Option<u64>,
}

/// Read data from a serial port.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSerialReadArgs {
    #[schemars(description = "Serial port path (e.g. /dev/ttyUSB0 or COM3).")]
    pub port: String,
    #[schemars(description = "Baud rate (default 9600).")]
    #[serde(default)]
    pub baud: Option<u64>,
    #[schemars(description = "Read timeout in milliseconds (default 2000).")]
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Maximum bytes to read (default 256).")]
    #[serde(default)]
    pub bytes: Option<u64>,
}

/// Scan for nearby Bluetooth devices.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetBluetoothScanArgs {
    #[schemars(description = "Scan duration in seconds (default 5).")]
    #[serde(default)]
    pub timeout_s: Option<u64>,
}

/// Publish a message to an MQTT broker topic.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetMqttPublishArgs {
    #[schemars(description = "MQTT broker hostname or IP address.")]
    pub broker: String,
    #[schemars(description = "MQTT topic to publish to.")]
    pub topic: String,
    #[schemars(description = "Message payload to publish.")]
    pub message: String,
    #[schemars(description = "MQTT broker port (default 1883).")]
    #[serde(default)]
    pub port: Option<u64>,
    #[schemars(description = "MQTT QoS level: 0 | 1 | 2 (default 0).")]
    #[serde(default)]
    pub qos: Option<u64>,
}

/// Subscribe to an MQTT topic and collect messages.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetMqttSubscribeArgs {
    #[schemars(description = "MQTT broker hostname or IP address.")]
    pub broker: String,
    #[schemars(description = "MQTT topic to subscribe to (supports wildcards).")]
    pub topic: String,
    #[schemars(description = "How long to listen for messages in seconds (default 5).")]
    #[serde(default)]
    pub timeout_s: Option<u64>,
    #[schemars(description = "MQTT broker port (default 1883).")]
    #[serde(default)]
    pub port: Option<u64>,
}

/// Capture live network packets on an interface.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetPacketCaptureArgs {
    #[schemars(description = "Network interface to capture on (e.g. eth0).")]
    pub interface: String,
    #[schemars(description = "Number of packets to capture (default 50).")]
    #[serde(default)]
    pub count: Option<u64>,
    #[schemars(description = "BPF filter expression (e.g. \"tcp port 80\").")]
    #[serde(default)]
    pub filter: Option<String>,
    #[schemars(description = "Maximum capture duration in seconds (default 10).")]
    #[serde(default)]
    pub timeout_s: Option<u64>,
}

/// Show the current status of a VPN connection.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetVpnStatusArgs {
    #[schemars(description = "VPN type to inspect: wireguard | openvpn | auto (default auto).")]
    #[serde(default)]
    pub vpn_type: Option<String>,
}

/// Attempt a DNS zone transfer (AXFR) for a domain.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetDnsZoneTransferArgs {
    #[schemars(description = "Domain name to attempt AXFR against.")]
    pub domain: String,
    #[schemars(description = "Specific nameserver to query (default: authoritative NS for the domain).")]
    #[serde(default)]
    pub nameserver: Option<String>,
}

/// Grade the SSL/TLS configuration of a remote host.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetSslGradeArgs {
    #[schemars(description = "Hostname to assess TLS configuration for.")]
    pub host: String,
    #[schemars(description = "TCP port (default 443).")]
    #[serde(default)]
    pub port: Option<u64>,
}

/// Read registers from a Modbus TCP device.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NetModbusReadArgs {
    #[schemars(description = "Hostname or IP of the Modbus TCP device.")]
    pub host: String,
    #[schemars(description = "Modbus unit/slave ID (default 1).")]
    #[serde(default)]
    pub unit_id: Option<u64>,
    #[schemars(description = "Modbus function code: 1 (coils) | 2 (discrete inputs) | 3 (holding regs) | 4 (input regs) (default 3).")]
    #[serde(default)]
    pub function_code: Option<u64>,
    #[schemars(description = "Starting register address (default 0).")]
    #[serde(default)]
    pub start_address: Option<u64>,
    #[schemars(description = "Number of registers/coils to read (default 10).")]
    #[serde(default)]
    pub count: Option<u64>,
    #[schemars(description = "Modbus TCP port (default 502).")]
    #[serde(default)]
    pub port: Option<u64>,
}

// ── Protection / Security ─────────────────────────────────────────────────────

/// Scan a single file with ClamAV antivirus.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectScanFileArgs {
    #[schemars(description = "Absolute or project-relative path to the file to scan.")]
    pub path: String,
}

/// Recursively scan a directory with ClamAV antivirus.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectScanDirectoryArgs {
    #[schemars(description = "Absolute or project-relative path to the directory to scan.")]
    pub path: String,
    #[schemars(description = "Maximum directory recursion depth (default 10, max 50).")]
    #[serde(default)]
    pub max_depth: Option<u64>,
}

/// Scan a file or directory with YARA rules.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectYaraScanArgs {
    #[schemars(description = "Path to the file or directory to scan.")]
    pub path: String,
    #[schemars(description = "Path to the YARA rules file (.yar / .yara).")]
    pub rules_path: String,
}

/// Verify (or update) a SHA-256 integrity manifest for a path.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectIntegrityCheckArgs {
    #[schemars(description = "Path to the file or directory to verify.")]
    pub path: String,
    #[schemars(description = "Path to the JSON manifest file (default: <path>.manifest.json).")]
    #[serde(default)]
    pub manifest_path: Option<String>,
    #[schemars(description = "When true, regenerate and save the manifest instead of verifying.")]
    #[serde(default)]
    pub update: Option<bool>,
}

/// Scan files for accidentally committed secrets (API keys, tokens, passwords).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectSecretScanArgs {
    #[schemars(description = "Path to the directory or file to scan for secrets.")]
    pub path: String,
    #[schemars(description = "Maximum directory recursion depth (default 10).")]
    #[serde(default)]
    pub max_depth: Option<u64>,
}

/// Audit project dependencies for known vulnerabilities.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectDependencyAuditArgs {
    #[schemars(description = "Root path of the project to audit.")]
    pub path: String,
    #[schemars(description = "Package manager: cargo | npm | pip | auto (default auto).")]
    #[serde(default)]
    pub manager: Option<String>,
}

/// Find SUID/SGID binaries under a path.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectSuidFindArgs {
    #[schemars(description = "Root path to search for SUID/SGID binaries.")]
    pub path: String,
}

/// Find world-writable files and directories under a path.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectWorldWritableFindArgs {
    #[schemars(description = "Root path to search for world-writable entries.")]
    pub path: String,
}

/// Run a basic rootkit detection check using rkhunter or chkrootkit.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectRootkitCheckArgs {}

/// Detect anomalous processes (hidden PIDs, suspicious names, missing executables).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectProcessAnomalyArgs {}

/// Detect anomalous network connections (unexpected listeners, unusual destinations).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectNetworkAnomalyArgs {}

/// Audit cron jobs for suspicious or world-writable entries.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectCronAuditArgs {}

/// Audit SSH authorized keys and private key permissions.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectSshKeyAuditArgs {
    #[schemars(description = "Path to the .ssh directory to audit (default: ~/.ssh).")]
    #[serde(default)]
    pub path: Option<String>,
}

/// Report the current firewall status (iptables / nftables / ufw).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectFirewallStatusArgs {}

/// Move a suspicious file to a quarantine directory.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectQuarantineFileArgs {
    #[schemars(description = "Path to the file to quarantine.")]
    pub path: String,
    #[schemars(description = "Quarantine directory path (default: /var/quarantine or project root).")]
    #[serde(default)]
    pub quarantine_dir: Option<String>,
}

/// Check a file's hash against known malware reputation databases.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectFileReputationArgs {
    #[schemars(description = "Path to the file to look up by SHA-256 hash.")]
    pub path: String,
}

/// Scan file permissions under a path for security issues.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ProtectPermissionsScanArgs {
    #[schemars(description = "Root path to scan for permission issues.")]
    pub path: String,
}

// ── IoT ───────────────────────────────────────────────────────────────────────

/// Enumerate IoT/embedded devices visible on the system (USB, serial, I2C, GPIO).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotDeviceInventoryArgs {}

/// Read bytes from a serial port (IoT / embedded device context).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotSerialReadArgs {
    #[schemars(description = "Serial port path (e.g. /dev/ttyUSB0).")]
    pub port: String,
    #[schemars(description = "Baud rate (default 9600).")]
    #[serde(default)]
    pub baud: Option<u32>,
    #[schemars(description = "Read timeout in milliseconds (default 2000).")]
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[schemars(description = "Maximum bytes to read (default 256).")]
    #[serde(default)]
    pub bytes: Option<usize>,
}

/// Write data to a serial port (IoT / embedded device context).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotSerialWriteArgs {
    #[schemars(description = "Serial port path (e.g. /dev/ttyUSB0).")]
    pub port: String,
    #[schemars(description = "Data string to write.")]
    pub data: String,
    #[schemars(description = "Baud rate (default 9600).")]
    #[serde(default)]
    pub baud: Option<u32>,
}

/// Scan an I2C bus for connected device addresses.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotI2cScanArgs {
    #[schemars(description = "I2C bus number to scan (default 1).")]
    #[serde(default)]
    pub bus: Option<u8>,
}

/// Read the digital or analog value of a GPIO pin.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotGpioReadArgs {
    #[schemars(description = "GPIO pin number (BCM numbering on Raspberry Pi).")]
    pub pin: u8,
}

/// Set the output value of a GPIO pin.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotGpioWriteArgs {
    #[schemars(description = "GPIO pin number (BCM numbering on Raspberry Pi).")]
    pub pin: u8,
    #[schemars(description = "Output value: 0 (low) or 1 (high).")]
    pub value: u8,
}

/// Read CAN bus frames from an interface.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotCanBusReadArgs {
    #[schemars(description = "CAN interface name (e.g. can0).")]
    pub interface: String,
    #[schemars(description = "Number of CAN frames to read (default 10).")]
    #[serde(default)]
    pub count: Option<usize>,
    #[schemars(description = "Read timeout in milliseconds (default 3000).")]
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Send an SMS message via an AT-command-compatible modem on a serial port.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct IotSmsSendArgs {
    #[schemars(description = "Serial port path of the GSM modem (e.g. /dev/ttyUSB0).")]
    pub port: String,
    #[schemars(description = "Destination phone number in international format (e.g. +12025551234).")]
    pub number: String,
    #[schemars(description = "SMS message text to send.")]
    pub message: String,
}

// ── Data Analytics ────────────────────────────────────────────────────────────

/// Analyze a CSV file and return column statistics and a sample.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataCsvAnalyzeArgs {
    #[schemars(description = "Path to the CSV file to analyze.")]
    pub path: String,
    #[schemars(description = "Maximum number of rows to read (default 10 000).")]
    #[serde(default)]
    pub max_rows: Option<usize>,
}

/// Flatten a nested JSON file into a flat key-value structure.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataJsonFlattenArgs {
    #[schemars(description = "Path to the JSON file to flatten.")]
    pub path: String,
    #[schemars(description = "Key separator for nested fields (default \".\").")]
    #[serde(default)]
    pub separator: Option<String>,
}

/// Diff two JSONL files record-by-record using a key field.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataDiffJsonlArgs {
    #[schemars(description = "Path to the first JSONL file.")]
    pub path_a: String,
    #[schemars(description = "Path to the second JSONL file.")]
    pub path_b: String,
    #[schemars(description = "JSON field name used as the record key for matching (default \"id\").")]
    #[serde(default)]
    pub key_field: Option<String>,
}

/// Extract all regex pattern matches from a text file.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataRegexExtractArgs {
    #[schemars(description = "Path to the file to search.")]
    pub path: String,
    #[schemars(description = "Regular expression pattern to match (Rust regex syntax).")]
    pub pattern: String,
    #[schemars(description = "Maximum number of matches to return (default 1000).")]
    #[serde(default)]
    pub max_matches: Option<usize>,
}

/// Detect anomalies in a numeric time-series using a rolling z-score window.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataTimeseriesDetectArgs {
    #[schemars(description = "Ordered list of numeric values forming the time-series.")]
    #[serde(default)]
    pub values: Vec<f64>,
    #[schemars(description = "Rolling window size for computing mean/stddev (default 20).")]
    #[serde(default)]
    pub window: Option<usize>,
    #[schemars(description = "Z-score threshold above which a point is flagged as anomalous (default 2.5).")]
    #[serde(default)]
    pub threshold: Option<f64>,
}

/// Read rows from a Parquet file and return them as JSON.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DataParquetReadArgs {
    #[schemars(description = "Path to the .parquet file to read.")]
    pub path: String,
    #[schemars(description = "Maximum number of rows to return (default 100).")]
    #[serde(default)]
    pub limit: Option<usize>,
}

// ── Quantum / Research ────────────────────────────────────────────────────────

/// Simulate a quantum circuit defined in OpenQASM 2.0.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct QuantumCircuitSimulateArgs {
    #[schemars(description = "OpenQASM 2.0 source string describing the circuit.")]
    pub circuit_qasm: String,
    #[schemars(description = "Number of measurement shots to simulate (default 1024).")]
    #[serde(default)]
    pub shots: Option<usize>,
}

/// Simulate Grover's quantum search algorithm.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct QuantumGroverSearchArgs {
    #[schemars(description = "Size of the search space (number of elements).")]
    pub space_size: usize,
    #[schemars(description = "Index of the target element to find.")]
    pub target_value: usize,
}

/// Simulate Shor's quantum integer factorisation algorithm.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct QuantumShorFactorArgs {
    #[schemars(description = "Positive integer to factorise (must be composite and > 1).")]
    pub n: u64,
}

/// Search arXiv for academic papers matching a query.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ResearchArxivSearchArgs {
    #[schemars(description = "Full-text or keyword search query.")]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (default 10).")]
    #[serde(default)]
    pub max_results: Option<usize>,
    #[schemars(description = "arXiv category filter (e.g. cs.AI, quant-ph, physics).")]
    #[serde(default)]
    pub category: Option<String>,
}

/// Download a paper PDF from arXiv by its ID.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ResearchArxivDownloadArgs {
    #[schemars(description = "arXiv paper ID (e.g. 2401.12345 or abs/2401.12345).")]
    pub arxiv_id: String,
    #[schemars(description = "Local directory to save the PDF (default: current directory).")]
    #[serde(default)]
    pub dest_dir: Option<String>,
}

// ── Extended System ───────────────────────────────────────────────────────────

/// Kill a process by PID or by name.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysProcessKillArgs {
    #[schemars(description = "Process ID to kill.")]
    #[serde(default)]
    pub pid: Option<u32>,
    #[schemars(description = "Process name pattern to match and kill (used if pid is absent).")]
    #[serde(default)]
    pub name: Option<String>,
    #[schemars(description = "Signal to send (e.g. SIGTERM, SIGKILL, 9); default SIGTERM.")]
    #[serde(default)]
    pub signal: Option<String>,
}

/// Control a systemd (or init) service.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysServiceControlArgs {
    #[schemars(description = "Name of the service unit (e.g. nginx or nginx.service).")]
    pub service: String,
    #[schemars(description = "Action to perform: start | stop | restart | status | enable | disable.")]
    pub action: String,
}

/// List all cron jobs for the current user (and system crontabs).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysCronListArgs {}

/// Add a cron job for the current user.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysCronAddArgs {
    #[schemars(description = "Cron schedule expression (e.g. \"0 * * * *\").")]
    pub schedule: String,
    #[schemars(description = "Shell command to run on the schedule.")]
    pub command: String,
}

/// Tail the systemd journal for a specific service unit.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysJournalTailArgs {
    #[schemars(description = "Systemd unit name to read logs from (e.g. nginx.service).")]
    pub unit: String,
    #[schemars(description = "Number of recent log lines to return (default 50).")]
    #[serde(default)]
    pub lines: Option<usize>,
}

/// List running Docker containers.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysDockerPsArgs {
    #[schemars(description = "When true, include stopped containers as well as running ones.")]
    #[serde(default)]
    pub all: Option<bool>,
}

/// Fetch recent log output from a Docker container.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysDockerLogsArgs {
    #[schemars(description = "Container name or ID.")]
    pub container: String,
    #[schemars(description = "Number of log lines to return from the tail (default 100).")]
    #[serde(default)]
    pub lines: Option<usize>,
}

/// Execute a command inside a running Docker container.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysDockerExecArgs {
    #[schemars(description = "Container name or ID to exec into.")]
    pub container: String,
    #[schemars(description = "Shell command to run inside the container.")]
    pub command: String,
}

/// Build a Docker image from a Dockerfile in a directory.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysDockerBuildArgs {
    #[schemars(description = "Path to the build context directory (containing the Dockerfile).")]
    pub path: String,
    #[schemars(description = "Image tag to assign (e.g. myapp:latest).")]
    pub tag: String,
}

/// List pods in a Kubernetes namespace.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysK8sPodsArgs {
    #[schemars(description = "Kubernetes namespace to list pods from (default: default).")]
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Fetch logs from a Kubernetes pod.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysK8sLogsArgs {
    #[schemars(description = "Name of the pod to retrieve logs from.")]
    pub pod: String,
    #[schemars(description = "Kubernetes namespace the pod lives in (default: default).")]
    #[serde(default)]
    pub namespace: Option<String>,
    #[schemars(description = "Number of recent log lines to return (default 100).")]
    #[serde(default)]
    pub lines: Option<usize>,
    #[schemars(description = "Specific container name within the pod (optional for single-container pods).")]
    #[serde(default)]
    pub container: Option<String>,
}

/// Apply a Kubernetes manifest file with kubectl.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysK8sApplyArgs {
    #[schemars(description = "Path to the Kubernetes manifest YAML/JSON file to apply.")]
    pub manifest_path: String,
}

/// Gather detailed hardware information (CPU, memory, storage, PCI, USB).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SysHardwareInfoArgs {}

// ── AI / ML Extended ─────────────────────────────────────────────────────────

/// Few-shot text classification using an Ollama model.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiFewShotClassifyArgs {
    #[schemars(description = "Text to classify.")]
    pub text: String,
    #[schemars(description = "Training examples as array of {text, label} objects.")]
    pub examples: Vec<serde_json::Value>,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Zero-shot text classification into provided label categories.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiZeroShotClassifyArgs {
    #[schemars(description = "Text to classify.")]
    pub text: String,
    #[schemars(description = "List of candidate category labels.")]
    pub labels: Vec<String>,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Extract structured JSON data from unstructured text using a schema.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiStructuredExtractArgs {
    #[schemars(description = "Source text to extract data from.")]
    pub text: String,
    #[schemars(description = "JSON schema object describing the desired output structure.")]
    pub schema: serde_json::Value,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Verify a factual claim against reference documents.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiVerifyFactualArgs {
    #[schemars(description = "The claim to verify.")]
    pub claim: String,
    #[schemars(description = "Array of reference document strings to check against.")]
    pub references: Vec<String>,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Rewrite text in a different style.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiRewriteStyleArgs {
    #[schemars(description = "Text to rewrite.")]
    pub text: String,
    #[schemars(description = "Target style: technical | casual | academic | eli5 | concise (default: concise).")]
    #[serde(default)]
    pub style: Option<String>,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Embed a text string using an Ollama embedding model.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiEmbedTextArgs {
    #[schemars(description = "Text to embed.")]
    pub text: String,
    #[schemars(description = "Ollama embedding model name (default: nomic-embed-text).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Semantic search over documents using embedding cosine similarity.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiSemanticSearchArgs {
    #[schemars(description = "Search query text.")]
    pub query: String,
    #[schemars(description = "Array of document strings to search over.")]
    pub documents: Vec<String>,
    #[schemars(description = "Ollama embedding model name (default: nomic-embed-text).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Cluster texts by semantic similarity using k-means on embeddings.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiClusterTextsArgs {
    #[schemars(description = "Array of text strings to cluster.")]
    pub texts: Vec<String>,
    #[schemars(description = "Number of clusters k (default: 3).")]
    #[serde(default)]
    pub k: Option<u32>,
    #[schemars(description = "Ollama embedding model name (default: nomic-embed-text).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Pack multiple files into a context window respecting a token budget.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiContextPackSmartArgs {
    #[schemars(description = "Array of file paths to pack into the context window.")]
    pub paths: Vec<String>,
    #[schemars(description = "Maximum token budget (default: 8000).")]
    #[serde(default)]
    pub max_tokens: Option<u64>,
    #[schemars(description = "Model hint for token estimation (optional).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Get metadata for a local ML model file (ONNX, PyTorch, SafeTensors, etc.).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlModelInfoArgs {
    #[schemars(description = "Path to the model file.")]
    pub path: String,
}

/// Tokenize text using an Ollama model's tokenizer API.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlTokenizeArgs {
    #[schemars(description = "Text to tokenize.")]
    pub text: String,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Count tokens in text using the Ollama tokenize API (with char-based fallback).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlTokenCountArgs {
    #[schemars(description = "Text to count tokens for.")]
    pub text: String,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Benchmark an Ollama model's throughput (tokens/sec) and latency.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlBenchmarkModelArgs {
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Benchmark prompt text (optional).")]
    #[serde(default)]
    pub prompt: Option<String>,
    #[schemars(description = "Number of benchmark iterations 1-20 (default: 3).")]
    #[serde(default)]
    pub iterations: Option<u32>,
}

/// Estimate approximate perplexity of text using an Ollama model.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlPerplexityScoreArgs {
    #[schemars(description = "Text to score for naturalness / perplexity.")]
    pub text: String,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
}

/// Convert conversations to JSONL fine-tuning format (openai / llama / mistral).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlFineTunePrepArgs {
    #[schemars(description = "Array of conversation objects: [{messages: [{role, content}]}].")]
    pub conversations: Vec<serde_json::Value>,
    #[schemars(description = "Output format: openai | llama | mistral (default: openai).")]
    #[serde(default)]
    pub format: Option<String>,
}

/// Validate a JSONL fine-tuning dataset for quality (roles, content, token length).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlDatasetValidateArgs {
    #[schemars(description = "Path to the JSONL dataset file.")]
    pub path: String,
    #[schemars(description = "Maximum tokens per example (default: 4096).")]
    #[serde(default)]
    pub max_tokens: Option<u64>,
}

/// Compare outputs from multiple Ollama models on the same prompt.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiMultiModelCompareArgs {
    #[schemars(description = "Prompt to send to all models.")]
    pub prompt: String,
    #[schemars(description = "Array of Ollama model names (default: [llama3.2, qwen2.5-coder]).")]
    #[serde(default)]
    pub models: Option<Vec<String>>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Run multi-step chain-of-thought reasoning using an Ollama model.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiChainOfThoughtArgs {
    #[schemars(description = "Question or problem to reason about step by step.")]
    pub prompt: String,
    #[schemars(description = "Number of reasoning steps (default: 3).")]
    #[serde(default)]
    pub steps: Option<u32>,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Self-critique an AI response and generate an improved version.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiSelfCritiqueArgs {
    #[schemars(description = "The original question/prompt given to the AI.")]
    pub original_prompt: String,
    #[schemars(description = "The AI response to critique and improve.")]
    pub original_response: String,
    #[schemars(description = "Ollama model name (default: llama3.2).")]
    #[serde(default)]
    pub model: Option<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Simulate a debate between two Ollama model instances on a topic.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AiDebateArgs {
    #[schemars(description = "Topic or proposition to debate.")]
    pub topic: String,
    #[schemars(description = "Ollama model for the 'for' side (default: llama3.2).")]
    #[serde(default)]
    pub model_a: Option<String>,
    #[schemars(description = "Ollama model for the 'against' side (default: llama3.2).")]
    #[serde(default)]
    pub model_b: Option<String>,
    #[schemars(description = "Number of debate rounds 1-5 (default: 2).")]
    #[serde(default)]
    pub rounds: Option<u32>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

/// Convert a model to GGUF format using llama.cpp's convert.py script.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlConvertToGgufArgs {
    #[schemars(description = "Path to the input model directory or file.")]
    pub input_path: String,
    #[schemars(description = "Output GGUF file path (default: auto-generated from input name).")]
    #[serde(default)]
    pub output_path: Option<String>,
    #[schemars(description = "Quantization type (default: q4_k_m).")]
    #[serde(default)]
    pub quantization: Option<String>,
    #[schemars(description = "Python interpreter path (default: python3).")]
    #[serde(default)]
    pub python_path: Option<String>,
}

/// Run a multi-model comparison benchmark on a fixed set of prompts.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MlModelCompareBenchmarkArgs {
    #[schemars(description = "Array of Ollama model names to benchmark.")]
    pub models: Vec<String>,
    #[schemars(description = "Array of prompt strings to test each model with.")]
    pub prompts: Vec<String>,
    #[schemars(description = "Ollama API URL (default: http://localhost:11434).")]
    #[serde(default)]
    pub ollama_url: Option<String>,
}

// ── Logs / Charts ─────────────────────────────────────────────────────────────

/// Read the last N lines of a log file, optionally filtering by log level.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogTailArgs {
    #[schemars(description = "Path to the log file.")]
    pub path: String,
    #[schemars(description = "Number of tail lines to read (default: 100).")]
    #[serde(default)]
    pub n: Option<u64>,
    #[schemars(description = "Filter by log level: DEBUG | INFO | WARN | ERROR (optional).")]
    #[serde(default)]
    pub filter_level: Option<String>,
}

/// Grep a log file for lines matching a regex pattern.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogSearchArgs {
    #[schemars(description = "Path to the log file.")]
    pub path: String,
    #[schemars(description = "Regex pattern to search for.")]
    pub pattern: String,
    #[schemars(description = "Maximum number of matches to return (default: 50).")]
    #[serde(default)]
    pub max_results: Option<u64>,
}

/// Parse a JSON Lines log file into structured entries with optional field filters.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogParseJsonArgs {
    #[schemars(description = "Path to the JSONL log file.")]
    pub path: String,
    #[schemars(description = "Maximum entries to return (default: 100).")]
    #[serde(default)]
    pub limit: Option<u64>,
    #[schemars(description = "Optional key-value filter object to match fields (e.g. {\"level\": \"error\"}).")]
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

/// Parse an Nginx access log into structured JSON entries.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogParseNginxArgs {
    #[schemars(description = "Path to the Nginx access log file.")]
    pub path: String,
    #[schemars(description = "Maximum entries to return (default: 100).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Parse a syslog format file into structured JSON entries.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogParseSyslogArgs {
    #[schemars(description = "Path to the syslog file.")]
    pub path: String,
    #[schemars(description = "Maximum entries to return (default: 100).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Count DEBUG / INFO / WARN / ERROR occurrences in a log file.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogLevelSummaryArgs {
    #[schemars(description = "Path to the log file.")]
    pub path: String,
}

/// Group repeated error lines by normalized pattern and count occurrences.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogErrorClusterArgs {
    #[schemars(description = "Path to the log file.")]
    pub path: String,
    #[schemars(description = "Maximum cluster groups to return (default: 10).")]
    #[serde(default)]
    pub max_clusters: Option<u64>,
}

/// Extract log lines within an ISO timestamp range.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogTimeRangeFilterArgs {
    #[schemars(description = "Path to the log file.")]
    pub path: String,
    #[schemars(description = "Start timestamp ISO string (inclusive, e.g. 2024-01-15T10:00).")]
    #[serde(default)]
    pub since: Option<String>,
    #[schemars(description = "End timestamp ISO string (exclusive).")]
    #[serde(default)]
    pub until: Option<String>,
    #[schemars(description = "Maximum entries to return (default: 1000).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Export an array of log entry objects to a file in JSON, CSV, or Markdown format.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogExportArgs {
    #[schemars(description = "Array of log entry objects to export.")]
    pub entries: Vec<serde_json::Value>,
    #[schemars(description = "Output format: json | csv | markdown (default: json).")]
    #[serde(default)]
    pub format: Option<String>,
    #[schemars(description = "Output file path to write the exported data.")]
    pub output_path: String,
}

/// Query the .mint-mcp audit log for tool calls by agent, tool name, or timestamp.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LogAuditQueryArgs {
    #[schemars(description = "Filter by agent ID (optional).")]
    #[serde(default)]
    pub agent_id: Option<String>,
    #[schemars(description = "Filter by tool name (optional).")]
    #[serde(default)]
    pub tool_name: Option<String>,
    #[schemars(description = "Only include entries at or after this UNIX millisecond timestamp (optional).")]
    #[serde(default)]
    pub since_ms: Option<u64>,
    #[schemars(description = "Maximum entries to return (default: 100).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Get recent tool calls from the .mint-mcp audit log.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HistoryToolCallsArgs {
    #[schemars(description = "Filter by agent ID (optional).")]
    #[serde(default)]
    pub agent_id: Option<String>,
    #[schemars(description = "Maximum entries to return (default: 50).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Replay and filter messages from a JSONL agent-bus file.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HistoryBusReplayArgs {
    #[schemars(description = "Path to the JSONL bus file (e.g. .mint-mcp/agent-bus.jsonl).")]
    pub path: String,
    #[schemars(description = "Only include messages after this ISO timestamp (optional).")]
    #[serde(default)]
    pub since: Option<String>,
    #[schemars(description = "Filter by agent ID in from or to fields (optional).")]
    #[serde(default)]
    pub agent: Option<String>,
    #[schemars(description = "Maximum messages to return (default: 100).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Show how a file changed across its last N git commits (content size + diffs).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HistoryDiffTimelineArgs {
    #[schemars(description = "Path to the file (relative to git repo root).")]
    pub path: String,
    #[schemars(description = "Number of commits to inspect (default: 5).")]
    #[serde(default)]
    pub count: Option<u64>,
}

/// Full-text search across .mint-mcp audit log entries by query string.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct HistorySearchPastCallsArgs {
    #[schemars(description = "Search query string to match against tool names, agents, and metadata.")]
    pub query: String,
    #[schemars(description = "Maximum results to return (default: 20).")]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Generate an SVG line chart from one or more named data series.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartLineArgs {
    #[schemars(description = "Array of series objects: [{label: string, values: [{x: number, y: number}]}].")]
    pub series: Vec<serde_json::Value>,
    #[schemars(description = "Chart title (default: Line Chart).")]
    #[serde(default)]
    pub title: Option<String>,
    #[schemars(description = "X-axis label (default: X).")]
    #[serde(default)]
    pub x_label: Option<String>,
    #[schemars(description = "Y-axis label (default: Y).")]
    #[serde(default)]
    pub y_label: Option<String>,
}

/// Generate an SVG bar chart from labeled values.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartBarArgs {
    #[schemars(description = "Array of bar objects: [{label: string, value: number}].")]
    pub values: Vec<serde_json::Value>,
    #[schemars(description = "Chart title (default: Bar Chart).")]
    #[serde(default)]
    pub title: Option<String>,
}

/// Generate an SVG pie or donut chart from labeled values.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartPieArgs {
    #[schemars(description = "Array of slice objects: [{label: string, value: number}].")]
    pub values: Vec<serde_json::Value>,
    #[schemars(description = "Chart title (default: Pie Chart).")]
    #[serde(default)]
    pub title: Option<String>,
    #[schemars(description = "Render as a donut chart with a hole in the center (default: false).")]
    #[serde(default)]
    pub donut: Option<bool>,
}

/// Generate an SVG histogram from a list of numeric values.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartHistogramArgs {
    #[schemars(description = "Array of numeric values to bin into the histogram.")]
    pub values: Vec<f64>,
    #[schemars(description = "Number of histogram bins (default: 10).")]
    #[serde(default)]
    pub bins: Option<u64>,
    #[schemars(description = "Chart title (default: Histogram).")]
    #[serde(default)]
    pub title: Option<String>,
}

/// Generate an SVG scatter plot from x/y point data.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartScatterArgs {
    #[schemars(description = "Array of point objects: [{x: number, y: number}].")]
    pub points: Vec<serde_json::Value>,
    #[schemars(description = "Chart title (default: Scatter Plot).")]
    #[serde(default)]
    pub title: Option<String>,
}

/// Generate an SVG time-series chart with an optional moving-average overlay.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartTimeseriesArgs {
    #[schemars(description = "Array of time-value objects: [{timestamp: string, value: number}].")]
    pub values: Vec<serde_json::Value>,
    #[schemars(description = "Moving average window size in data points (default: 5).")]
    #[serde(default)]
    pub window: Option<u64>,
    #[schemars(description = "Chart title (default: Time Series).")]
    #[serde(default)]
    pub title: Option<String>,
}

/// Render a module / file dependency graph in DOT (Graphviz) format.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartDependencyGraphArgs {
    #[schemars(description = "Array of node name strings.")]
    pub nodes: Vec<String>,
    #[schemars(description = "Array of edge objects: [{from: string, to: string, label?: string}].")]
    pub edges: Vec<serde_json::Value>,
    #[schemars(description = "Graph title (default: Dependency Graph).")]
    #[serde(default)]
    pub title: Option<String>,
}

/// Generate an SVG Gantt chart from task start / end time data.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChartGanttArgs {
    #[schemars(description = "Array of task objects: [{name: string, start_time: string, end_time: string, color?: string}].")]
    pub tasks: Vec<serde_json::Value>,
    #[schemars(description = "Chart title (default: Gantt Chart).")]
    #[serde(default)]
    pub title: Option<String>,
}

// ── Neural ────────────────────────────────────────────────────────────────────

/// Read ONNX / PyTorch / SafeTensors model metadata and I/O shape information.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NeuralModelInfoArgs {
    #[schemars(description = "Path to the model file (.onnx, .pt, .bin, .safetensors, etc.).")]
    pub model_path: String,
}

/// Run ONNX Runtime inference on a model file (requires ml-tools feature).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NeuralInferenceArgs {
    #[schemars(description = "Path to the ONNX model file.")]
    pub model_path: String,
    #[schemars(description = "JSON map of input tensors: {\"input_name\": [f32 values...]}.")]
    pub input_json: String,
}

/// Tokenize text using a HuggingFace tokenizer file (requires ml-tools feature).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NeuralTokenizeArgs {
    #[schemars(description = "Text to tokenize.")]
    pub text: String,
    #[schemars(description = "Model name or path to tokenizer.json (e.g. bert-base-uncased).")]
    pub model_name: String,
}

/// Compute cosine similarity between two embedding vectors (no ML dependencies).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NeuralEmbedCosineArgs {
    #[schemars(description = "First embedding vector as array of floats.")]
    pub vec_a: Vec<f32>,
    #[schemars(description = "Second embedding vector as array of floats (must be same length as vec_a).")]
    pub vec_b: Vec<f32>,
}

/// Compute statistics (mean, std, min, max, percentiles) for a flat numeric tensor.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NeuralTensorStatsArgs {
    #[schemars(description = "JSON array of numeric values representing the flat tensor (e.g. [1.0, 2.5, 3.7]).")]
    pub tensor_json: String,
}
