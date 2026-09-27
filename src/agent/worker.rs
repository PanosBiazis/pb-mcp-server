//! Agent Worker module — rig-based LLM-backed agent workers.
//!
//! Each agent role runs as an independent `tokio::spawn` task with:
//! - An Ollama-backed LLM agent with a specialty system prompt
//! - A broadcast channel subscription for inter-agent messaging
//!
//! # Agent Roles
//!
//! | Role         | Model          | Specialty                              |
//! |--------------|----------------|----------------------------------------|
//! | rust_expert  | llama3.1       | Rust language expert, idiomatic code   |
//! | architect    | deepseek-coder | System design, patterns, tradeoffs     |
//! | debugger     | codellama      | Bug finding, trace analysis            |
//! | reviewer     | mistral        | Code review, style, maintainability    |
//! | documenter   | llama3.1       | Documentation, comments, READMEs       |
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐   broadcast   ┌─────────────────┐
//! │  Agent Bus   │ ─────────────▶│   AgentWorker   │
//! │     (tx)     │               │ ┌─────────────┐ │
//! └──────────────┘               │ │ OllamaAgent │ │
//!                                │ │   (HTTP)    │ │
//!                                │ └─────────────┘ │
//!                                └─────────────────┘
//! ```
//!
//! Workers are managed through a [`WorkerPool`] that handles spawning,
//! stopping, and listing active workers.

use crate::bus::AgentMessage;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

// ============================================================================
// ID Generation
// ============================================================================

/// Global counter for generating unique worker IDs.
static WORKER_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Generate a unique worker ID of the form `worker-<counter>-<timestamp_hex>`.
fn generate_worker_id() -> String {
    let count = WORKER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("worker-{count}-{ts:x}")
}

// ============================================================================
// Agent Role
// ============================================================================

/// Predefined speciality roles for agent workers.
///
/// Each role carries a default Ollama model and a system prompt that
/// specialises the agent's behaviour. The [`Custom`] variant allows
/// ad-hoc roles with user-defined model and prompt strings.
///
/// [`Custom`]: AgentRole::Custom
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Rust language expert — idiomatic code, ownership, lifetimes, crates.
    RustExpert,
    /// System architect — design patterns, tradeoffs, modularity.
    Architect,
    /// Debugger — bug isolation, trace analysis, root-cause reasoning.
    Debugger,
    /// Code reviewer — style, maintainability, correctness, nits.
    Reviewer,
    /// Documenter — doc comments, READMEs, usage examples.
    Documenter,
    /// User-defined role with a custom name.
    Custom(String),
}

impl AgentRole {
    /// Human-readable name for the role.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            Self::RustExpert => Cow::Borrowed("rust_expert"),
            Self::Architect => Cow::Borrowed("architect"),
            Self::Debugger => Cow::Borrowed("debugger"),
            Self::Reviewer => Cow::Borrowed("reviewer"),
            Self::Documenter => Cow::Borrowed("documenter"),
            Self::Custom(tag) => Cow::Borrowed(tag.as_str()),
        }
    }

    /// Default Ollama model for this role.
    pub fn model_name(&self) -> &'static str {
        match self {
            Self::RustExpert => "llama3.1",
            Self::Architect => "deepseek-coder",
            Self::Debugger => "codellama",
            Self::Reviewer => "mistral",
            Self::Documenter => "llama3.1",
            Self::Custom(_) => "llama3.1",
        }
    }

    /// System prompt that specialises the agent's behaviour.
    pub fn system_prompt(&self) -> Cow<'_, str> {
        match self {
            Self::RustExpert => Cow::Borrowed(
                r#"You are a Rust language expert agent. Your speciality is:
- Writing idiomatic, safe, and performant Rust code
- Explaining ownership, borrowing, and lifetime rules
- Recommending appropriate crates and patterns
- Identifying unsafe pitfalls and suggesting safe alternatives
- Helping with async Rust, trait design, and macro usage
- Always respond with well-structured Rust code and concise explanations."#,
            ),
            Self::Architect => Cow::Borrowed(
                r#"You are a system architect agent. Your speciality is:
- High-level system design and modular decomposition
- Choosing appropriate design patterns for the problem at hand
- Evaluating tradeoffs (performance vs. simplicity, coupling vs. cohesion)
- Defining clear interfaces and data-flow boundaries
- Planning for extensibility, testability, and maintainability
- Respond with architectural rationale, diagrams (text-based), and concrete recommendations."#,
            ),
            Self::Debugger => Cow::Borrowed(
                r#"You are a debugger agent. Your speciality is:
- Isolating bugs from stack traces, logs, and error messages
- Walking through execution flow to find root causes
- Suggesting targeted fixes with minimal side-effects
- Recommending diagnostic additions (logging, assertions, tests)
- Identifying common error patterns (off-by-one, race conditions, UB)
- Respond with step-by-step reasoning, the most likely root cause, and a minimal fix."#,
            ),
            Self::Reviewer => Cow::Borrowed(
                r#"You are a code reviewer agent. Your speciality is:
- Reading diffs and providing actionable, constructive feedback
- Checking for style consistency, naming, and formatting
- Identifying potential correctness or performance issues
- Suggesting improvements to error handling and edge-case coverage
- Flagging security-sensitive patterns
- Keep reviews concise, categorised (nit / suggestion / blocker), and always explain *why*."#,
            ),
            Self::Documenter => Cow::Borrowed(
                r#"You are a documentation agent. Your speciality is:
- Writing clear, idiomatic rustdoc comments
- Producing READMEs, CHANGELOGs, and usage guides
- Ensuring public APIs have examples and panic/safety notes
- Translating complex logic into reader-friendly prose
- Keeping documentation in sync with code changes
- Respond with ready-to-commit documentation in the appropriate format."#,
            ),
            Self::Custom(prompt) => Cow::Borrowed(prompt.as_str()),
        }
    }
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

// ============================================================================
// Agent Worker Status
// ============================================================================

/// Runtime status of an [`AgentWorker`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentWorkerStatus {
    /// Worker is idle, waiting for the next message.
    #[default]
    Idle,
    /// Worker is currently processing a message (payload is a short summary).
    Processing(String),
    /// Worker encountered an error (payload is the error description).
    Error(String),
    /// Worker has been stopped and will not process further messages.
    Stopped,
}

// ============================================================================
// Agent Worker Config
// ============================================================================

/// Configuration for spawning an [`AgentWorker`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWorkerConfig {
    /// The role (and therefore system prompt + default model) for this worker.
    pub role: AgentRole,
    /// Base URL of the Ollama instance.
    #[serde(default = "default_ollama_base_url")]
    pub ollama_base_url: String,
    /// When set, overrides the role's default model name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_override: Option<String>,
    /// Maximum conversation history length kept in the agent context.
    #[serde(default = "default_max_history")]
    pub max_history: usize,
    /// Sampling temperature for the LLM.
    #[serde(default = "default_temperature")]
    pub temperature: f64,
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_max_history() -> usize {
    20
}

fn default_temperature() -> f64 {
    0.7
}

impl Default for AgentWorkerConfig {
    fn default() -> Self {
        Self {
            role: AgentRole::RustExpert,
            ollama_base_url: default_ollama_base_url(),
            model_override: None,
            max_history: default_max_history(),
            temperature: default_temperature(),
        }
    }
}

// ============================================================================
// Ollama HTTP Client (no external deps)
// ============================================================================

/// Lightweight HTTP client for the Ollama `/api/chat` endpoint.
///
/// Uses `tokio::net::TcpStream` with raw HTTP/1.1 so it compiles without
/// pulling in `reqwest` or any other HTTP crate. This is intentionally
/// minimal — it supports exactly one POST endpoint with JSON body and
/// returns the parsed `"message.content"` field.
struct OllamaHttpClient {
    base_url: String,
}

impl OllamaHttpClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Parse `http://host:port` into `(host, port)`.
    fn parse_host_port(&self) -> Result<(String, u16), String> {
        let url = self.base_url.trim();
        let stripped = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .ok_or_else(|| format!("Ollama URL must start with http:// or https://: {url}"))?;

        let (host, port) = if let Some(idx) = stripped.rfind(':') {
            let host = &stripped[..idx];
            let port: u16 = stripped[idx + 1..]
                .parse()
                .map_err(|_| format!("Invalid port in Ollama URL: {url}"))?;
            (host.to_string(), port)
        } else if url.starts_with("https://") {
            (stripped.to_string(), 443)
        } else {
            (stripped.to_string(), 11434)
        };

        Ok((host, port))
    }

    /// Send a chat request to Ollama and return the assistant's response text.
    async fn chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        history: &[(String, String)],
        temperature: f64,
    ) -> Result<String, String> {
        let (host, port) = self.parse_host_port()?;

        // Build the JSON request body for Ollama /api/chat
        let mut messages = Vec::new();

        // System message
        messages.push(serde_json::json!({
            "role": "system",
            "content": system_prompt
        }));

        // History messages
        for (user_msg, assistant_msg) in history {
            messages.push(serde_json::json!({
                "role": "user",
                "content": user_msg
            }));
            messages.push(serde_json::json!({
                "role": "assistant",
                "content": assistant_msg
            }));
        }

        // Current user message
        messages.push(serde_json::json!({
            "role": "user",
            "content": user_prompt
        }));

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": temperature
            }
        });

        let body_str = serde_json::to_string(&body)
            .map_err(|e| format!("Failed to serialise Ollama request: {e}"))?;

        // Build raw HTTP/1.1 request
        let request = format!(
            "POST /api/chat HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {body_str}",
            body_str.len()
        );

        // Connect and send
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let mut stream = TcpStream::connect((&*host, port))
            .await
            .map_err(|e| format!("Failed to connect to Ollama at {host}:{port}: {e}"))?;

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| format!("Failed to send request to Ollama: {e}"))?;

        stream
            .shutdown()
            .await
            .map_err(|e| format!("Failed to shutdown write half: {e}"))?;

        // Read the full response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| format!("Failed to read Ollama response: {e}"))?;

        let response_str = String::from_utf8_lossy(&response_bytes);

        // Split headers from body (separated by \r\n\r\n)
        let body_start = response_str
            .find("\r\n\r\n")
            .ok_or_else(|| "Ollama response has no header/body separator".to_string())?;
        let http_body = &response_str[body_start + 4..];

        // Check HTTP status line
        let status_line = &response_str[..response_str.find('\r').unwrap_or(0)];
        if !status_line.contains("200") {
            return Err(format!("Ollama returned non-200 status: {status_line}"));
        }

        // Parse the JSON body
        let json: serde_json::Value = serde_json::from_str(http_body.trim_end_matches('\0'))
            .map_err(|e| format!("Failed to parse Ollama JSON response: {e}\nBody: {http_body}"))?;

        json.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Ollama response missing message.content".to_string())
    }
}

// ============================================================================
// Agent Worker
// ============================================================================

/// A single LLM-backed agent worker.
///
/// Each worker owns an `OllamaHttpClient` configured with an Ollama model
/// and a role-specific system prompt. The main [`run`](AgentWorker::run) loop
/// listens on a broadcast receiver for [`AgentMessage`]s, forwards each
/// message to the LLM, and logs the response.
pub struct AgentWorker {
    /// Worker configuration (role, model, etc.).
    pub config: AgentWorkerConfig,
    /// Unique identifier generated on creation.
    pub id: String,
    /// Current processing status (shared behind `Arc<Mutex>` for interior
    /// mutability across the spawned task and the pool).
    status: Arc<Mutex<AgentWorkerStatus>>,
    /// Number of messages processed so far.
    message_count: Arc<AtomicU64>,
}

impl fmt::Debug for AgentWorker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentWorker")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("status", &self.status_snapshot())
            .field("message_count", &self.message_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl AgentWorker {
    /// Create a new worker with a generated ID.
    pub fn new(config: AgentWorkerConfig) -> Self {
        Self {
            id: generate_worker_id(),
            config,
            status: Arc::new(Mutex::new(AgentWorkerStatus::Idle)),
            message_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Snapshot the current status.
    pub fn status_snapshot(&self) -> AgentWorkerStatus {
        lock_recover(&self.status).clone()
    }

    /// Read the current message count.
    pub fn message_count(&self) -> u64 {
        self.message_count.load(Ordering::Relaxed)
    }

    /// Effective model name: `model_override` if set, otherwise the role default.
    pub fn effective_model(&self) -> &str {
        self.config
            .model_override
            .as_deref()
            .unwrap_or_else(|| self.config.role.model_name())
    }

    /// Run the worker's main loop.
    ///
    /// Creates an Ollama HTTP client connected to the configured model, then
    /// enters an async loop that:
    /// 1. Receives [`AgentMessage`]s from the broadcast channel
    /// 2. Sends each message body as a prompt to the LLM
    /// 3. Logs the agent's response and updates status
    /// 4. Exits cleanly when the channel closes
    pub async fn run(self, mut bus_rx: broadcast::Receiver<AgentMessage>) {
        let model = self.effective_model().to_string();
        let system_prompt = self.config.role.system_prompt().to_string();
        let base_url = self.config.ollama_base_url.clone();
        let worker_id = self.id.clone();
        let role_name = self.config.role.name().to_string();
        let max_history = self.config.max_history;
        let temperature = self.config.temperature;

        tracing::info!(
            worker_id = %worker_id,
            role = %role_name,
            model = %model,
            "AgentWorker starting"
        );

        let client = OllamaHttpClient::new(&base_url);

        // Conversation history (simple ring buffer of past exchanges).
        let mut history: Vec<(String, String)> = Vec::new();

        // -----------------------------------------------------------------------
        // Main receive loop
        // -----------------------------------------------------------------------
        loop {
            let msg = match bus_rx.recv().await {
                Ok(msg) => msg,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        worker_id = %worker_id,
                        skipped = n,
                        "Broadcast channel lagged — skipping stale messages"
                    );
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!(
                        worker_id = %worker_id,
                        "Broadcast channel closed — shutting down worker"
                    );
                    break;
                }
            };

            // Update status to Processing
            {
                let mut st = lock_recover(&self.status);
                let summary = if msg.body.len() > 80 {
                    format!("{}…", &msg.body[..80])
                } else {
                    msg.body.clone()
                };
                *st = AgentWorkerStatus::Processing(summary);
            }

            tracing::debug!(
                worker_id = %worker_id,
                role = %role_name,
                from = %msg.from_agent,
                kind = %msg.kind,
                "Processing bus message"
            );

            // Build the prompt from the message and recent history
            let prompt = build_prompt(&msg, &history, max_history);

            // -------------------------------------------------------------------
            // Call the LLM via Ollama
            // -------------------------------------------------------------------
            match client
                .chat(&model, &system_prompt, &prompt, &history, temperature)
                .await
            {
                Ok(response) => {
                    tracing::info!(
                        worker_id = %worker_id,
                        role = %role_name,
                        response_len = response.len(),
                        "LLM response received"
                    );

                    // Update history ring buffer
                    if max_history > 0 {
                        if history.len() >= max_history {
                            history.remove(0);
                        }
                        history.push((prompt.clone(), response));
                    }

                    // Increment message counter
                    self.message_count.fetch_add(1, Ordering::Relaxed);

                    // Reset status to Idle
                    {
                        let mut st = lock_recover(&self.status);
                        *st = AgentWorkerStatus::Idle;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        worker_id = %worker_id,
                        role = %role_name,
                        error = %e,
                        "LLM call failed"
                    );
                    {
                        let mut st = lock_recover(&self.status);
                        *st = AgentWorkerStatus::Error(e);
                    }

                    // Back off briefly before retrying
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    // Reset to Idle so the worker can try again
                    {
                        let mut st = lock_recover(&self.status);
                        *st = AgentWorkerStatus::Idle;
                    }
                }
            }
        }

        // Mark as stopped
        {
            let mut st = lock_recover(&self.status);
            *st = AgentWorkerStatus::Stopped;
        }

        tracing::info!(
            worker_id = %worker_id,
            role = %role_name,
            total_messages = self.message_count.load(Ordering::Relaxed),
            "AgentWorker stopped"
        );
    }
}

// ============================================================================
// Prompt Builder
// ============================================================================

/// Build the user prompt from the incoming bus message and recent history.
fn build_prompt(msg: &AgentMessage, history: &[(String, String)], max_history: usize) -> String {
    let mut parts = Vec::new();

    // Include recent conversation history for context
    if max_history > 0 && !history.is_empty() {
        parts.push("[Previous context]".to_string());
        for (i, (user_msg, assistant_msg)) in history.iter().enumerate() {
            parts.push(format!("User: {user_msg}"));
            parts.push(format!("Assistant: {assistant_msg}"));
            if i + 1 >= max_history {
                break;
            }
        }
        parts.push("[End previous context]\n".to_string());
    }

    // Current message
    parts.push(format!(
        "[From: {} | Kind: {} | Topic: {}]",
        msg.from_agent,
        msg.kind,
        msg.topic.as_deref().unwrap_or("-")
    ));
    parts.push(msg.body.clone());

    parts.join("\n")
}

// ============================================================================
// Agent Worker Handle
// ============================================================================

/// Handle to a spawned [`AgentWorker`] task.
///
/// Wraps the [`JoinHandle`] and the worker's ID, providing a [`stop`](Self::stop)
/// method that aborts the underlying tokio task.
pub struct AgentWorkerHandle {
    /// The tokio task handle.
    join_handle: JoinHandle<()>,
    /// The worker's unique ID.
    pub id: String,
}

impl AgentWorkerHandle {
    /// Abort the worker's tokio task.
    ///
    /// This sends a cancellation signal to the spawned task. The worker's
    /// status will be set to [`AgentWorkerStatus::Stopped`] on the next
    /// observation.
    pub fn stop(&self) {
        self.join_handle.abort();
    }

    /// Check whether the worker task has finished.
    pub fn is_finished(&self) -> bool {
        self.join_handle.is_finished()
    }
}

impl fmt::Debug for AgentWorkerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentWorkerHandle")
            .field("id", &self.id)
            .field("finished", &self.join_handle.is_finished())
            .finish()
    }
}

// ============================================================================
// Worker Info
// ============================================================================

/// Serializable summary of a worker's current state.
///
/// Returned by [`WorkerPool::list`] for introspection and status endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Unique worker identifier.
    pub id: String,
    /// The worker's assigned role.
    pub role: AgentRole,
    /// Current processing status.
    pub status: AgentWorkerStatus,
    /// Total messages processed since spawn.
    pub message_count: u64,
}

// ============================================================================
// Internal state for StatefulWorkerPool
// ============================================================================

/// Internal state shared between a spawned task and the pool.
#[derive(Debug)]
pub struct WorkerStateInner {
    role: AgentRole,
    status: AgentWorkerStatus,
    message_count: u64,
}

/// Enhanced worker handle that carries shared state accessible from the pool.
pub struct PooledWorkerHandle {
    /// Handle to the spawned tokio task, used to abort or join the worker.
    pub join_handle: JoinHandle<()>,
    /// Unique identifier of this worker within the pool.
    pub id: String,
    /// Shared inner state accessible from both the worker task and the pool.
    pub state: Arc<Mutex<WorkerStateInner>>,
}

impl PooledWorkerHandle {
    /// Abort the worker's spawned task.
    pub fn stop(&self) {
        self.join_handle.abort();
    }

    /// Returns `true` if the worker's task has finished.
    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        self.join_handle.is_finished()
    }

    /// Returns a snapshot of the worker's current information.
    pub fn info(&self) -> WorkerInfo {
        let st = lock_recover(&self.state);
        WorkerInfo {
            id: self.id.clone(),
            role: st.role.clone(),
            status: st.status.clone(),
            message_count: st.message_count,
        }
    }
}

// ============================================================================
// Worker Pool
// ============================================================================

/// Pool of running [`AgentWorker`] tasks.
///
/// Manages the full lifecycle: spawn workers with a config, stop them by ID,
/// and list their current status.
pub struct WorkerPool {
    /// Active worker handles keyed by worker ID.
    workers: HashMap<String, PooledWorkerHandle>,
}

impl WorkerPool {
    /// Create an empty worker pool.
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }

    /// Spawn a new agent worker and register it in the pool.
    ///
    /// Returns the worker's unique ID on success, or an error string on
    /// failure.
    ///
    /// The worker runs as an independent `tokio::spawn` task that:
    /// 1. Creates an Ollama HTTP client connected to the configured model
    /// 2. Listens on the broadcast receiver for [`AgentMessage`]s
    /// 3. Processes each message through the LLM
    /// 4. Exits when the channel closes or is stopped
    pub fn spawn(
        &mut self,
        config: AgentWorkerConfig,
        bus_rx: broadcast::Receiver<AgentMessage>,
    ) -> Result<String, String> {
        let role = config.role.clone();
        let worker = AgentWorker::new(config);
        let worker_id = worker.id.clone();

        // Shared state for pool-level introspection
        let shared_state = Arc::new(Mutex::new(WorkerStateInner {
            role: role.clone(),
            status: AgentWorkerStatus::Idle,
            message_count: 0,
        }));

        let state_for_sync = Arc::clone(&shared_state);
        let status_arc = Arc::clone(&worker.status);
        let count_arc = Arc::clone(&worker.message_count);
        let sync_id = worker_id.clone();

        // Spawn the worker task
        let join_handle = tokio::spawn(async move {
            worker.run(bus_rx).await;
        });

        // Spawn a background sync task that periodically mirrors the worker's
        // internal status/count into the shared state accessible from the pool.
        let sync_state = state_for_sync;
        let sync_status = status_arc;
        let sync_count = count_arc;
        let sync_worker_id = sync_id;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let mut st = lock_recover(&sync_state);
                st.status = lock_recover(&sync_status).clone();
                st.message_count = sync_count.load(Ordering::Relaxed);
                if matches!(st.status, AgentWorkerStatus::Stopped) {
                    tracing::debug!(
                        worker_id = %sync_worker_id,
                        "Status sync exiting — worker stopped"
                    );
                    break;
                }
            }
        });

        let handle = PooledWorkerHandle {
            join_handle,
            id: worker_id.clone(),
            state: shared_state,
        };

        self.workers.insert(worker_id.clone(), handle);

        tracing::info!(
            worker_id = %worker_id,
            role = %role,
            "Worker spawned and registered in pool"
        );

        Ok(worker_id)
    }

    /// Stop a worker by ID. Returns `true` if the worker was found and
    /// aborted, `false` if no worker with that ID exists.
    pub fn stop(&mut self, id: &str) -> bool {
        if let Some(handle) = self.workers.remove(id) {
            handle.stop();
            tracing::info!(worker_id = %id, "Worker stopped and removed from pool");
            true
        } else {
            tracing::warn!(worker_id = %id, "Attempted to stop unknown worker");
            false
        }
    }

    /// List all workers with their current status info.
    ///
    /// Returns up-to-date status, role, and message counts by reading from
    /// the shared state that is synced every second from the worker task.
    pub fn list(&self) -> Vec<WorkerInfo> {
        self.workers.values().map(|h| h.info()).collect()
    }

    /// Get a reference to a pooled handle by ID.
    pub fn get(&self, id: &str) -> Option<&PooledWorkerHandle> {
        self.workers.get(id)
    }

    /// Number of workers currently in the pool.
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_role_names() {
        assert_eq!(AgentRole::RustExpert.name(), "rust_expert");
        assert_eq!(AgentRole::Architect.name(), "architect");
        assert_eq!(AgentRole::Debugger.name(), "debugger");
        assert_eq!(AgentRole::Reviewer.name(), "reviewer");
        assert_eq!(AgentRole::Documenter.name(), "documenter");
        assert_eq!(AgentRole::Custom("analyst".into()).name(), "analyst");
    }

    #[test]
    fn agent_role_models() {
        assert_eq!(AgentRole::RustExpert.model_name(), "llama3.1");
        assert_eq!(AgentRole::Architect.model_name(), "deepseek-coder");
        assert_eq!(AgentRole::Debugger.model_name(), "codellama");
        assert_eq!(AgentRole::Reviewer.model_name(), "mistral");
        assert_eq!(AgentRole::Documenter.model_name(), "llama3.1");
    }

    #[test]
    fn agent_role_system_prompts_non_empty() {
        assert!(!AgentRole::RustExpert.system_prompt().is_empty());
        assert!(!AgentRole::Architect.system_prompt().is_empty());
        assert!(!AgentRole::Debugger.system_prompt().is_empty());
        assert!(!AgentRole::Reviewer.system_prompt().is_empty());
        assert!(!AgentRole::Documenter.system_prompt().is_empty());
        let custom = AgentRole::Custom("custom prompt".into());
        assert_eq!(custom.system_prompt(), "custom prompt");
    }

    #[test]
    fn agent_role_display() {
        assert_eq!(format!("{}", AgentRole::RustExpert), "rust_expert");
        assert_eq!(format!("{}", AgentRole::Architect), "architect");
    }

    #[test]
    fn agent_role_serialization() {
        let role = AgentRole::RustExpert;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"rust_expert\"");
        let back: AgentRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, role);
    }

    #[test]
    fn agent_worker_config_defaults() {
        let cfg = AgentWorkerConfig::default();
        assert_eq!(cfg.role, AgentRole::RustExpert);
        assert_eq!(cfg.ollama_base_url, "http://localhost:11434");
        assert!(cfg.model_override.is_none());
        assert_eq!(cfg.max_history, 20);
        assert!((cfg.temperature - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn agent_worker_config_serialization_roundtrip() {
        let cfg = AgentWorkerConfig {
            role: AgentRole::Debugger,
            ollama_base_url: "http://ollama.local:11434".into(),
            model_override: Some("llama3.1:70b".into()),
            max_history: 50,
            temperature: 0.3,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AgentWorkerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.role, AgentRole::Debugger);
        assert_eq!(back.ollama_base_url, "http://ollama.local:11434");
        assert_eq!(back.model_override.as_deref(), Some("llama3.1:70b"));
        assert_eq!(back.max_history, 50);
        assert!((back.temperature - 0.3).abs() < 1e-10);
    }

    #[test]
    fn agent_worker_new_generates_unique_ids() {
        let w1 = AgentWorker::new(AgentWorkerConfig::default());
        let w2 = AgentWorker::new(AgentWorkerConfig::default());
        assert_ne!(w1.id, w2.id);
        assert!(w1.id.starts_with("worker-"));
    }

    #[test]
    fn agent_worker_status_default() {
        let st = AgentWorkerStatus::default();
        matches!(st, AgentWorkerStatus::Idle);
    }

    #[test]
    fn agent_worker_status_serialization() {
        let st = AgentWorkerStatus::Processing("test…".into());
        let json = serde_json::to_string(&st).unwrap();
        assert!(json.contains("processing"));
    }

    #[test]
    fn effective_model_override() {
        let cfg = AgentWorkerConfig {
            role: AgentRole::RustExpert,
            model_override: Some("custom-model".into()),
            ..Default::default()
        };
        let w = AgentWorker::new(cfg);
        assert_eq!(w.effective_model(), "custom-model");

        let cfg2 = AgentWorkerConfig {
            role: AgentRole::RustExpert,
            model_override: None,
            ..Default::default()
        };
        let w2 = AgentWorker::new(cfg2);
        assert_eq!(w2.effective_model(), "llama3.1");
    }

    #[test]
    fn worker_pool_new_is_empty() {
        let pool = WorkerPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert!(pool.list().is_empty());
    }

    #[test]
    fn worker_pool_get_nonexistent() {
        let pool = WorkerPool::new();
        assert!(pool.get("worker-999").is_none());
    }

    #[test]
    fn worker_pool_stop_nonexistent() {
        let mut pool = WorkerPool::new();
        assert!(!pool.stop("worker-999"));
    }

    #[test]
    fn build_prompt_basic() {
        let msg = AgentMessage {
            id: "msg-1".into(),
            ts: 1000,
            from_agent: "planner".into(),
            to_agent: Some("rust_expert".into()),
            topic: Some("code_review".into()),
            kind: "request".into(),
            body: "Review this function".into(),
        };
        let prompt = build_prompt(&msg, &[], 20);
        assert!(prompt.contains("[From: planner"));
        assert!(prompt.contains("Review this function"));
    }

    #[test]
    fn build_prompt_with_history() {
        let msg = AgentMessage {
            id: "msg-2".into(),
            ts: 2000,
            from_agent: "planner".into(),
            to_agent: None,
            topic: None,
            kind: "request".into(),
            body: "Next step?".into(),
        };
        let history = vec![
            ("What is 2+2?".into(), "4".into()),
            ("And 3+3?".into(), "6".into()),
        ];
        let prompt = build_prompt(&msg, &history, 20);
        assert!(prompt.contains("[Previous context]"));
        assert!(prompt.contains("What is 2+2?"));
        assert!(prompt.contains("And 3+3?"));
        assert!(prompt.contains("Next step?"));
    }

    #[test]
    fn ollama_client_parse_host_port() {
        let c = OllamaHttpClient::new("http://localhost:11434");
        let (h, p) = c.parse_host_port().unwrap();
        assert_eq!(h, "localhost");
        assert_eq!(p, 11434);

        let c = OllamaHttpClient::new("http://192.168.1.50:8080");
        let (h, p) = c.parse_host_port().unwrap();
        assert_eq!(h, "192.168.1.50");
        assert_eq!(p, 8080);

        let c = OllamaHttpClient::new("http://ollama.example.com");
        let (h, p) = c.parse_host_port().unwrap();
        assert_eq!(h, "ollama.example.com");
        assert_eq!(p, 11434);

        let c = OllamaHttpClient::new("badurl");
        assert!(c.parse_host_port().is_err());
    }

    #[tokio::test]
    async fn worker_exits_on_closed_channel() {
        let (tx, rx) = tokio::sync::broadcast::channel(16);
        let worker = AgentWorker::new(AgentWorkerConfig {
            role: AgentRole::Debugger,
            ollama_base_url: "http://localhost:11434".into(),
            ..Default::default()
        });
        let _worker_id = worker.id.clone();
        let status = Arc::clone(&worker.status);

        drop(tx); // Close the channel immediately

        let handle = tokio::spawn(async move {
            worker.run(rx).await;
        });

        // Wait for the worker to finish
        let _ = handle.await;

        let final_status = status.lock().unwrap().clone();
        assert!(matches!(final_status, AgentWorkerStatus::Stopped));
    }
}
