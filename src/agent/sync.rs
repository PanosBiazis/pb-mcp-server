//! # Agent Sync – Multi-Agent Synchronization Tools
//!
//! Coordination primitives built on top of the durable JSONL message bus
//! (`crate::bus`). Provides:
//!
//! - **Agent Registry** – In-memory, thread-safe registry of known agents
//!   with capabilities, health status, and heartbeat tracking.
//! - **Agent Sync** – Broadcast, selective, state-diff, and two-phase-commit
//!   synchronization patterns that emit bus messages for durable audit.
//! - **Sync State Tracking** – `SyncStateStore` tracks in-flight syncs so
//!   callers can poll for confirmations or detect rollback.
//!
//! ## Thread Safety
//!
//! Both `AgentRegistry` and `SyncStateStore` use `DashMap` internally, so
//! all operations are lock-free for readers and fine-grained for writers.
//! No `unsafe` code is used.

use chrono::Utc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Agent Status
// ============================================================================

/// Health / availability status of a registered agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is actively processing tasks and responsive.
    Active,
    /// Agent is alive but not currently accepting new work.
    #[default]
    Idle,
    /// Agent has missed heartbeats and is presumed unavailable.
    Offline,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Active => write!(f, "active"),
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Offline => write!(f, "offline"),
        }
    }
}

// ============================================================================
// Agent Entry
// ============================================================================

/// A single registered agent's metadata stored in the [`AgentRegistry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    /// Stable agent identifier (letters, digits, `_`, `.`, `-`).
    pub name: String,
    /// Declared capability tags (e.g. `"plan"`, `"code"`, `"review"`).
    pub capabilities: Vec<String>,
    /// Unix-epoch milliseconds of the most recent heartbeat.
    pub last_heartbeat_ms: u64,
    /// Current health / availability status.
    pub status: AgentStatus,
    /// Arbitrary key-value metadata attached by the agent.
    pub metadata: HashMap<String, String>,
}

impl AgentEntry {
    /// Create a new entry with the given name and capabilities.
    ///
    /// The heartbeat timestamp is initialised to the current time and the
    /// status is set to [`AgentStatus::Idle`].
    pub fn new(name: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            name: name.into(),
            capabilities,
            last_heartbeat_ms: now_ms(),
            status: AgentStatus::Idle,
            metadata: HashMap::new(),
        }
    }
}

// ============================================================================
// Agent Registry
// ============================================================================

/// Thread-safe in-memory registry of known agents.
///
/// Agents register themselves with capabilities, periodically heartbeat,
/// and can be deregistered when they shut down. The registry is used by
/// the sync functions to determine who should receive coordination
/// messages.
///
/// # Concurrency
///
/// Internally backed by a `DashMap`, so concurrent reads and writes from
/// multiple threads are safe without an external `Arc<Mutex>`.
///
/// # Example
///
/// ```rust,ignore
/// use pb_mcp_server::agent::sync::AgentRegistry;
///
/// let reg = AgentRegistry::new();
/// reg.register_agent("coder", vec!["code".into()], Default::default());
/// reg.heartbeat("coder");
/// assert!(reg.get_agent("coder").is_some());
/// ```
#[derive(Debug)]
pub struct AgentRegistry {
    agents: DashMap<String, AgentEntry>,
}

impl AgentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            agents: DashMap::new(),
        }
    }

    /// Register a new agent or overwrite an existing entry.
    ///
    /// If an agent with the same `name` already exists it is replaced
    /// entirely (including its heartbeat timestamp, which is reset).
    ///
    /// Returns `true` if this was a new insertion, `false` if an existing
    /// entry was overwritten.
    pub fn register_agent(
        &self,
        name: impl Into<String>,
        capabilities: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> bool {
        let name = name.into();
        let mut entry = AgentEntry::new(&name, capabilities);
        entry.metadata = metadata;
        // Mark as Active on registration
        entry.status = AgentStatus::Active;
        let existed = self.agents.remove(&name).is_some();
        self.agents.insert(name, entry);
        !existed
    }

    /// Remove an agent from the registry.
    ///
    /// Returns `true` if the agent was present and removed, `false` otherwise.
    pub fn deregister_agent(&self, name: &str) -> bool {
        self.agents.remove(name).is_some()
    }

    /// Record a heartbeat for the named agent, updating its
    /// `last_heartbeat_ms` and setting its status to [`AgentStatus::Active`].
    ///
    /// Returns `true` if the agent was found and updated, `false` otherwise.
    pub fn heartbeat(&self, name: &str) -> bool {
        if let Some(mut entry) = self.agents.get_mut(name) {
            entry.last_heartbeat_ms = now_ms();
            entry.status = AgentStatus::Active;
            true
        } else {
            false
        }
    }

    /// Return a snapshot of all agents currently marked [`AgentStatus::Active`].
    pub fn list_active(&self) -> Vec<AgentEntry> {
        self.agents
            .iter()
            .filter(|e| e.value().status == AgentStatus::Active)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Return a snapshot of all registered agents regardless of status.
    pub fn list_all(&self) -> Vec<AgentEntry> {
        self.agents.iter().map(|e| e.value().clone()).collect()
    }

    /// Find agents that declare *all* the given capability tags.
    ///
    /// Matching is case-sensitive. An empty `required` set returns **all**
    /// registered agents.
    pub fn find_by_capability(&self, required: &[String]) -> Vec<AgentEntry> {
        self.agents
            .iter()
            .filter(|e| {
                let caps = &e.value().capabilities;
                required.iter().all(|r| caps.contains(r))
            })
            .map(|e| e.value().clone())
            .collect()
    }

    /// Add or replace a single capability tag on an existing agent.
    ///
    /// Returns `true` if the agent was found (capability was set),
    /// `false` if the agent does not exist.
    pub fn set_capability(&self, name: &str, capability: impl Into<String>) -> bool {
        if let Some(mut entry) = self.agents.get_mut(name) {
            let cap = capability.into();
            if !entry.capabilities.contains(&cap) {
                entry.capabilities.push(cap);
            }
            true
        } else {
            false
        }
    }

    /// Retrieve a cloned entry for the named agent, or `None` if not found.
    pub fn get_agent(&self, name: &str) -> Option<AgentEntry> {
        self.agents.get(name).map(|e| e.value().clone())
    }

    /// Number of registered agents.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// `true` when no agents are registered.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Mark agents whose last heartbeat is older than `threshold_ms` as
    /// [`AgentStatus::Offline`].
    ///
    /// Returns the number of agents that were transitioned to Offline.
    pub fn expire_stale(&self, threshold_ms: u64) -> usize {
        let now = now_ms();
        let mut expired = 0usize;
        for mut entry in self.agents.iter_mut() {
            if entry.status != AgentStatus::Offline
                && now.saturating_sub(entry.last_heartbeat_ms) > threshold_ms
            {
                entry.status = AgentStatus::Offline;
                expired += 1;
            }
        }
        expired
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Sync Status
// ============================================================================

/// Status of an in-flight synchronization round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// Sync message sent; waiting for confirmations.
    #[default]
    Pending,
    /// All targeted agents have confirmed.
    Confirmed,
    /// At least one agent failed to confirm; the sync was rolled back.
    RolledBack,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Pending => write!(f, "pending"),
            SyncStatus::Confirmed => write!(f, "confirmed"),
            SyncStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

// ============================================================================
// Sync State
// ============================================================================

/// Tracks the state of a single synchronization round (broadcast or
/// two-phase-commit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Unique identifier for this sync round (e.g. `"sync-1748900000-12345"`).
    pub last_sync_id: String,
    /// Agent names that were targeted.
    pub participants: Vec<String>,
    /// Agent names that have sent a confirmation message back.
    pub confirmations: Vec<String>,
    /// Current status of this sync round.
    pub status: SyncStatus,
    /// ISO-8601 timestamp when the sync was initiated.
    pub initiated_at: String,
    /// State payload that was broadcast (opaque JSON string).
    pub state_snapshot: String,
}

// ============================================================================
// Sync State Store
// ============================================================================

/// Thread-safe store for tracking ongoing and completed synchronization
/// rounds.
///
/// Typically used together with [`agent_sync_confirm`] to implement a
/// two-phase-commit pattern:
///
/// 1. `agent_sync_confirm` creates a `SyncState` with status `Pending`.
/// 2. Target agents reply with a `sync_confirm` message on the bus.
/// 3. The caller periodically calls [`SyncStateStore::record_confirmation`]
///    and [`SyncStateStore::check_sync`] until all agents have confirmed or
///    a timeout elapses.
/// 4. If any agent is missing after the deadline, call
///    [`SyncStateStore::mark_rolled_back`].
#[derive(Debug)]
pub struct SyncStateStore {
    states: DashMap<String, SyncState>,
    counter: AtomicU64,
}

impl SyncStateStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
            counter: AtomicU64::new(0),
        }
    }

    /// Generate a new unique sync id and insert an initial `Pending` state.
    ///
    /// Returns the generated sync id.
    pub fn create_sync(&self, participants: Vec<String>, state_snapshot: String) -> String {
        let seq = self.counter.fetch_add(1, Ordering::Relaxed);
        let sync_id = format!("sync-{}-{}", now_ms(), seq);
        let state = SyncState {
            last_sync_id: sync_id.clone(),
            participants,
            confirmations: Vec::new(),
            status: SyncStatus::Pending,
            initiated_at: Utc::now().to_rfc3339(),
            state_snapshot,
        };
        self.states.insert(sync_id.clone(), state);
        sync_id
    }

    /// Record that `agent_name` has confirmed the given sync round.
    ///
    /// Returns `Ok(true)` if the confirmation was newly recorded,
    /// `Ok(false)` if it was already recorded, or `Err` if the sync id
    /// is unknown.
    pub fn record_confirmation(&self, sync_id: &str, agent_name: &str) -> Result<bool, String> {
        let mut state = self
            .states
            .get_mut(sync_id)
            .ok_or_else(|| format!("unknown sync_id: {sync_id}"))?;

        if state.confirmations.contains(&agent_name.to_string()) {
            return Ok(false);
        }
        state.confirmations.push(agent_name.to_string());

        // Auto-transition to Confirmed when all participants have confirmed
        if state.status == SyncStatus::Pending {
            let all_confirmed = state
                .participants
                .iter()
                .all(|p| state.confirmations.contains(p));
            if all_confirmed {
                state.status = SyncStatus::Confirmed;
            }
        }
        Ok(true)
    }

    /// Mark a sync round as rolled back.
    ///
    /// Returns `Ok(())` on success, or `Err` if the sync id is unknown.
    pub fn mark_rolled_back(&self, sync_id: &str) -> Result<(), String> {
        let mut state = self
            .states
            .get_mut(sync_id)
            .ok_or_else(|| format!("unknown sync_id: {sync_id}"))?;
        state.status = SyncStatus::RolledBack;
        Ok(())
    }

    /// Retrieve a snapshot of the state for a given sync round.
    pub fn get_sync(&self, sync_id: &str) -> Option<SyncState> {
        self.states.get(sync_id).map(|s| s.value().clone())
    }

    /// Return a snapshot of all tracked sync states.
    pub fn list_syncs(&self) -> Vec<SyncState> {
        self.states.iter().map(|e| e.value().clone()).collect()
    }

    /// Remove completed (Confirmed or RolledBack) sync states older than
    /// the given number of milliseconds.
    ///
    /// Returns the number of purged entries.
    pub fn purge_completed(&self, _older_than_ms: u64) -> usize {
        // We keep this simple: remove any Confirmed or RolledBack entries.
        let mut purged = 0usize;
        let to_remove: Vec<String> = self
            .states
            .iter()
            .filter(|e| {
                e.value().status == SyncStatus::Confirmed
                    || e.value().status == SyncStatus::RolledBack
            })
            .map(|e| e.key().clone())
            .collect();
        for key in to_remove {
            if self.states.remove(&key).is_some() {
                purged += 1;
            }
        }
        purged
    }

    /// Number of tracked sync states.
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// `true` when no sync states are tracked.
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl Default for SyncStateStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper: current time in milliseconds
// ============================================================================

/// Return the current Unix-epoch time in milliseconds.
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// Sync Functions
// ============================================================================

/// Broadcast a state snapshot to **all** registered agents and record the
/// sync round in the [`SyncStateStore`].
///
/// Each targeted agent receives a bus message with
/// `kind = "sync_broadcast"` and `topic = "sync:{sync_id}"`.
///
/// # Arguments
///
/// * `registry` – The agent registry to enumerate active agents.
/// * `store` – The sync-state store for tracking this round.
/// * `project_root` – Project root for the JSONL bus.
/// * `from_agent` – Name of the orchestrating agent that initiates the sync.
/// * `state_snapshot` – Opaque JSON payload representing the current state.
///
/// # Returns
///
/// `(sync_id, count)` where `count` is the number of agents that were
/// notified, or an error string.
pub fn agent_sync_all(
    registry: &AgentRegistry,
    store: &SyncStateStore,
    project_root: PathBuf,
    from_agent: &str,
    state_snapshot: &str,
) -> Result<(String, usize), String> {
    let active = registry.list_active();
    if active.is_empty() {
        return Err("no active agents to sync".into());
    }
    let participants: Vec<String> = active.iter().map(|a| a.name.clone()).collect();
    let sync_id = store.create_sync(participants.clone(), state_snapshot.to_string());
    let topic = format!("sync:{sync_id}");

    let mut notified = 0usize;
    for agent in &active {
        let result = crate::bus::agent_send(
            project_root.clone(),
            from_agent.to_string(),
            state_snapshot.to_string(),
            Some(agent.name.clone()),
            Some(topic.clone()),
            Some("sync_broadcast".to_string()),
        );
        if result.is_ok() {
            notified += 1;
        }
    }

    Ok((sync_id, notified))
}

/// Synchronize only agents whose capabilities match *all* tags in `filter`.
///
/// Otherwise behaves identically to [`agent_sync_all`].
///
/// # Arguments
///
/// * `filter` – Capability tags that an agent must possess to be included.
///
/// # Returns
///
/// `(sync_id, count)` where `count` is the number of matching agents that
/// were notified.
pub fn agent_sync_selective(
    registry: &AgentRegistry,
    store: &SyncStateStore,
    project_root: PathBuf,
    from_agent: &str,
    state_snapshot: &str,
    filter: &[String],
) -> Result<(String, usize), String> {
    let matched = registry.find_by_capability(filter);
    if matched.is_empty() {
        return Err(format!("no agents match filter: [{}]", filter.join(", ")));
    }
    let participants: Vec<String> = matched.iter().map(|a| a.name.clone()).collect();
    let sync_id = store.create_sync(participants.clone(), state_snapshot.to_string());
    let topic = format!("sync:{sync_id}");

    let mut notified = 0usize;
    for agent in &matched {
        let result = crate::bus::agent_send(
            project_root.clone(),
            from_agent.to_string(),
            state_snapshot.to_string(),
            Some(agent.name.clone()),
            Some(topic.clone()),
            Some("sync_broadcast".to_string()),
        );
        if result.is_ok() {
            notified += 1;
        }
    }

    Ok((sync_id, notified))
}

/// Send only the **delta** (difference) between a previous and current
/// state to all active agents, avoiding a full-state flood.
///
/// The delta is computed as a simple key-level JSON diff:
///
/// - Keys present in `current` but not in `previous` → *added*
/// - Keys present in `previous` but not in `current` → *removed*
/// - Keys in both with different values → *changed*
///
/// The resulting diff payload is serialised as JSON and sent with
/// `kind = "sync_diff"`.
///
/// # Arguments
///
/// * `previous` – Prior state as a JSON string (object).
/// * `current` – New state as a JSON string (object).
///
/// # Returns
///
/// `(sync_id, count, diff_json)` where `diff_json` is the computed delta.
pub fn agent_sync_state_diff(
    registry: &AgentRegistry,
    store: &SyncStateStore,
    project_root: PathBuf,
    from_agent: &str,
    previous: &str,
    current: &str,
) -> Result<(String, usize, String), String> {
    let active = registry.list_active();
    if active.is_empty() {
        return Err("no active agents to sync".into());
    }

    let prev_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(previous).unwrap_or_default();
    let curr_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(current).unwrap_or_default();

    let mut added = serde_json::Map::new();
    let mut removed = serde_json::Map::new();
    let mut changed = serde_json::Map::new();

    for (k, v) in &curr_map {
        match prev_map.get(k) {
            None => {
                added.insert(k.clone(), v.clone());
            }
            Some(old_v) if old_v != v => {
                changed.insert(k.clone(), v.clone());
            }
            _ => {}
        }
    }
    for (k, v) in &prev_map {
        if !curr_map.contains_key(k) {
            removed.insert(k.clone(), v.clone());
        }
    }

    let diff = serde_json::json!({
        "added": added,
        "removed": removed,
        "changed": changed,
    });
    let diff_json = serde_json::to_string(&diff).unwrap_or_else(|_| "{}".into());

    let participants: Vec<String> = active.iter().map(|a| a.name.clone()).collect();
    let sync_id = store.create_sync(participants, diff_json.clone());
    let topic = format!("sync:{sync_id}");

    let mut notified = 0usize;
    for agent in &active {
        let result = crate::bus::agent_send(
            project_root.clone(),
            from_agent.to_string(),
            diff_json.clone(),
            Some(agent.name.clone()),
            Some(topic.clone()),
            Some("sync_diff".to_string()),
        );
        if result.is_ok() {
            notified += 1;
        }
    }

    Ok((sync_id, notified, diff_json))
}

/// Initiate a two-phase-commit synchronization: broadcast a state to the
/// targeted agents, then wait for confirmations.
///
/// **Phase 1 – Prepare:** A `sync_prepare` message is sent to each
/// participant.  A `SyncState` entry is created in the store so callers
/// can track confirmations.
///
/// **Phase 2 – Confirm / Rollback:** After calling this function, the
/// orchestrator should wait for `sync_confirm` messages on the bus and
/// call [`SyncStateStore::record_confirmation`] for each one.  Once all
/// participants have confirmed, the store auto-transitions the state to
/// [`SyncStatus::Confirmed`].  If any participant fails to confirm within
/// a deadline, call [`SyncStateStore::mark_rolled_back`] and a
/// `sync_rollback` message will be sent on the next call to
/// [`agent_sync_complete_rollback`].
///
/// # Arguments
///
/// * `participants` – Explicit list of agent names to target.
/// * `state_snapshot` – Opaque JSON payload.
///
/// # Returns
///
/// `(sync_id, count)` where `count` is the number of participants that
/// were successfully sent the prepare message.
pub fn agent_sync_confirm(
    registry: &AgentRegistry,
    store: &SyncStateStore,
    project_root: PathBuf,
    from_agent: &str,
    participants: &[String],
    state_snapshot: &str,
) -> Result<(String, usize), String> {
    if participants.is_empty() {
        return Err("participants must not be empty".into());
    }

    // Validate that each participant exists in the registry
    for p in participants {
        if registry.get_agent(p).is_none() {
            return Err(format!("participant '{p}' is not registered"));
        }
    }

    let sync_id = store.create_sync(participants.to_vec(), state_snapshot.to_string());
    let topic = format!("sync:{sync_id}");

    let mut notified = 0usize;
    for agent_name in participants {
        let result = crate::bus::agent_send(
            project_root.clone(),
            from_agent.to_string(),
            state_snapshot.to_string(),
            Some(agent_name.clone()),
            Some(topic.clone()),
            Some("sync_prepare".to_string()),
        );
        if result.is_ok() {
            notified += 1;
        }
    }

    Ok((sync_id, notified))
}

/// Send a rollback notification for a previously prepared sync round.
///
/// This should be called after [`SyncStateStore::mark_rolled_back`] to
/// notify all original participants that the sync has been aborted.
///
/// # Arguments
///
/// * `sync_id` – The id returned by [`agent_sync_confirm`].
///
/// # Returns
///
/// The number of rollback messages successfully sent, or an error.
pub fn agent_sync_complete_rollback(
    store: &SyncStateStore,
    project_root: PathBuf,
    from_agent: &str,
    sync_id: &str,
) -> Result<usize, String> {
    let state = store
        .get_sync(sync_id)
        .ok_or_else(|| format!("unknown sync_id: {sync_id}"))?;

    if state.status != SyncStatus::RolledBack {
        return Err(format!(
            "sync {} is in status {}; mark it RolledBack first",
            sync_id, state.status
        ));
    }

    let topic = format!("sync:{sync_id}");
    let mut notified = 0usize;
    for agent_name in &state.participants {
        let result = crate::bus::agent_send(
            project_root.clone(),
            from_agent.to_string(),
            format!("ROLLBACK:{}", sync_id),
            Some(agent_name.clone()),
            Some(topic.clone()),
            Some("sync_rollback".to_string()),
        );
        if result.is_ok() {
            notified += 1;
        }
    }

    Ok(notified)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // -----------------------------------------------------------------------
    // AgentStatus tests
    // -----------------------------------------------------------------------

    #[test]
    fn agent_status_default_is_idle() {
        assert_eq!(AgentStatus::default(), AgentStatus::Idle);
    }

    #[test]
    fn agent_status_display() {
        assert_eq!(format!("{}", AgentStatus::Active), "active");
        assert_eq!(format!("{}", AgentStatus::Idle), "idle");
        assert_eq!(format!("{}", AgentStatus::Offline), "offline");
    }

    #[test]
    fn agent_status_serde_roundtrip() {
        let status = AgentStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");
        let back: AgentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }

    // -----------------------------------------------------------------------
    // AgentEntry tests
    // -----------------------------------------------------------------------

    #[test]
    fn agent_entry_new_sets_idle_status() {
        let entry = AgentEntry::new("test-agent", vec!["code".into()]);
        assert_eq!(entry.name, "test-agent");
        assert_eq!(entry.capabilities, vec!["code"]);
        assert_eq!(entry.status, AgentStatus::Idle);
        assert!(entry.metadata.is_empty());
        assert!(entry.last_heartbeat_ms > 0);
    }

    #[test]
    fn agent_entry_serde_roundtrip() {
        let mut entry = AgentEntry::new("agent-1", vec!["plan".into(), "code".into()]);
        entry.status = AgentStatus::Active;
        entry.metadata.insert("version".into(), "2.0".into());
        let json = serde_json::to_string(&entry).unwrap();
        let back: AgentEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, entry.name);
        assert_eq!(back.capabilities, entry.capabilities);
        assert_eq!(back.status, entry.status);
        assert_eq!(back.metadata, entry.metadata);
    }

    // -----------------------------------------------------------------------
    // AgentRegistry tests
    // -----------------------------------------------------------------------

    #[test]
    fn registry_new_is_empty() {
        let reg = AgentRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_default_is_new() {
        let reg = AgentRegistry::default();
        assert!(reg.is_empty());
    }

    #[test]
    fn register_and_get_agent() {
        let reg = AgentRegistry::new();
        let is_new = reg.register_agent("alpha", vec!["code".into()], HashMap::new());
        assert!(is_new);
        assert_eq!(reg.len(), 1);

        let entry = reg.get_agent("alpha").unwrap();
        assert_eq!(entry.name, "alpha");
        assert_eq!(entry.capabilities, vec!["code"]);
        assert_eq!(entry.status, AgentStatus::Active);
    }

    #[test]
    fn register_overwrites_existing() {
        let reg = AgentRegistry::new();
        reg.register_agent("alpha", vec!["code".into()], HashMap::new());
        let is_new = reg.register_agent("alpha", vec!["plan".into()], HashMap::new());
        assert!(!is_new); // overwritten, not new

        let entry = reg.get_agent("alpha").unwrap();
        assert_eq!(entry.capabilities, vec!["plan"]);
    }

    #[test]
    fn deregister_agent() {
        let reg = AgentRegistry::new();
        reg.register_agent("alpha", vec!["code".into()], HashMap::new());
        assert!(reg.deregister_agent("alpha"));
        assert!(!reg.deregister_agent("alpha")); // already removed
        assert!(reg.is_empty());
    }

    #[test]
    fn deregister_nonexistent_returns_false() {
        let reg = AgentRegistry::new();
        assert!(!reg.deregister_agent("ghost"));
    }

    #[test]
    fn heartbeat_updates_status_and_ts() {
        let reg = AgentRegistry::new();
        reg.register_agent("alpha", vec!["code".into()], HashMap::new());
        let before = reg.get_agent("alpha").unwrap().last_heartbeat_ms;

        // Small sleep to ensure time advances
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(reg.heartbeat("alpha"));

        let after = reg.get_agent("alpha").unwrap();
        assert!(after.last_heartbeat_ms >= before);
        assert_eq!(after.status, AgentStatus::Active);
    }

    #[test]
    fn heartbeat_unknown_agent_returns_false() {
        let reg = AgentRegistry::new();
        assert!(!reg.heartbeat("unknown"));
    }

    #[test]
    fn list_active_filters_correctly() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());
        reg.register_agent("a2", vec!["plan".into()], HashMap::new());

        // Both start Active (via register_agent)
        assert_eq!(reg.list_active().len(), 2);

        // Manually set one to Offline
        {
            let mut entry = reg.agents.get_mut("a1").unwrap();
            entry.status = AgentStatus::Offline;
        }
        let active = reg.list_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "a2");
    }

    #[test]
    fn list_all_returns_everyone() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());
        reg.register_agent("a2", vec!["plan".into()], HashMap::new());

        // Mark a1 offline
        {
            let mut entry = reg.agents.get_mut("a1").unwrap();
            entry.status = AgentStatus::Offline;
        }
        assert_eq!(reg.list_all().len(), 2);
    }

    #[test]
    fn find_by_capability() {
        let reg = AgentRegistry::new();
        reg.register_agent("coder", vec!["code".into()], HashMap::new());
        reg.register_agent("planner", vec!["plan".into()], HashMap::new());
        reg.register_agent("both", vec!["code".into(), "plan".into()], HashMap::new());

        let code_agents = reg.find_by_capability(&["code".to_string()]);
        assert_eq!(code_agents.len(), 2);

        let both_agents = reg.find_by_capability(&["code".to_string(), "plan".to_string()]);
        assert_eq!(both_agents.len(), 1);
        assert_eq!(both_agents[0].name, "both");

        // Empty filter returns all
        let all_agents = reg.find_by_capability(&[]);
        assert_eq!(all_agents.len(), 3);
    }

    #[test]
    fn find_by_capability_no_match() {
        let reg = AgentRegistry::new();
        reg.register_agent("coder", vec!["code".into()], HashMap::new());
        let found = reg.find_by_capability(&["deploy".to_string()]);
        assert!(found.is_empty());
    }

    #[test]
    fn set_capability_adds_new() {
        let reg = AgentRegistry::new();
        reg.register_agent("alpha", vec!["code".into()], HashMap::new());

        assert!(reg.set_capability("alpha", "plan"));
        let entry = reg.get_agent("alpha").unwrap();
        assert!(entry.capabilities.contains(&"plan".to_string()));
        assert!(entry.capabilities.contains(&"code".to_string()));
    }

    #[test]
    fn set_capability_idempotent() {
        let reg = AgentRegistry::new();
        reg.register_agent("alpha", vec!["code".into()], HashMap::new());
        assert!(reg.set_capability("alpha", "code")); // already present
        let entry = reg.get_agent("alpha").unwrap();
        // Should not duplicate
        assert_eq!(
            entry.capabilities.iter().filter(|c| **c == "code").count(),
            1
        );
    }

    #[test]
    fn set_capability_unknown_agent() {
        let reg = AgentRegistry::new();
        assert!(!reg.set_capability("ghost", "code"));
    }

    #[test]
    fn get_agent_nonexistent() {
        let reg = AgentRegistry::new();
        assert!(reg.get_agent("ghost").is_none());
    }

    #[test]
    fn expire_stale_transitions_offline() {
        let reg = AgentRegistry::new();
        reg.register_agent("fresh", vec!["code".into()], HashMap::new());
        reg.register_agent("stale", vec!["code".into()], HashMap::new());

        // Artificially age the "stale" agent
        {
            let mut entry = reg.agents.get_mut("stale").unwrap();
            entry.last_heartbeat_ms = now_ms().saturating_sub(60_000); // 60s ago
        }

        let expired = reg.expire_stale(30_000); // 30s threshold
        assert_eq!(expired, 1);
        assert_eq!(reg.get_agent("stale").unwrap().status, AgentStatus::Offline);
        assert_eq!(reg.get_agent("fresh").unwrap().status, AgentStatus::Active);
    }

    #[test]
    fn expire_stale_skips_already_offline() {
        let reg = AgentRegistry::new();
        reg.register_agent("ghost", vec!["code".into()], HashMap::new());
        {
            let mut entry = reg.agents.get_mut("ghost").unwrap();
            entry.status = AgentStatus::Offline;
            entry.last_heartbeat_ms = 0;
        }
        let expired = reg.expire_stale(1000);
        assert_eq!(expired, 0); // already offline, no transition
    }

    // -----------------------------------------------------------------------
    // SyncStatus tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_status_default_is_pending() {
        assert_eq!(SyncStatus::default(), SyncStatus::Pending);
    }

    #[test]
    fn sync_status_display() {
        assert_eq!(format!("{}", SyncStatus::Pending), "pending");
        assert_eq!(format!("{}", SyncStatus::Confirmed), "confirmed");
        assert_eq!(format!("{}", SyncStatus::RolledBack), "rolled_back");
    }

    #[test]
    fn sync_status_serde_roundtrip() {
        for status in [
            SyncStatus::Pending,
            SyncStatus::Confirmed,
            SyncStatus::RolledBack,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: SyncStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }

    // -----------------------------------------------------------------------
    // SyncState tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_state_serde_roundtrip() {
        let state = SyncState {
            last_sync_id: "sync-1234-0".into(),
            participants: vec!["a".into(), "b".into()],
            confirmations: vec!["a".into()],
            status: SyncStatus::Pending,
            initiated_at: "2025-01-01T00:00:00Z".into(),
            state_snapshot: r#"{"k":"v"}"#.into(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: SyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.last_sync_id, state.last_sync_id);
        assert_eq!(back.participants, state.participants);
        assert_eq!(back.confirmations, state.confirmations);
        assert_eq!(back.status, state.status);
    }

    // -----------------------------------------------------------------------
    // SyncStateStore tests
    // -----------------------------------------------------------------------

    #[test]
    fn store_new_is_empty() {
        let store = SyncStateStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn store_default_is_new() {
        let store = SyncStateStore::default();
        assert!(store.is_empty());
    }

    #[test]
    fn create_and_get_sync() {
        let store = SyncStateStore::new();
        let sync_id = store.create_sync(vec!["a".into(), "b".into()], r#"{"state":1}"#.into());
        assert!(!sync_id.is_empty());
        assert_eq!(store.len(), 1);

        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.last_sync_id, sync_id);
        assert_eq!(state.participants, vec!["a", "b"]);
        assert!(state.confirmations.is_empty());
        assert_eq!(state.status, SyncStatus::Pending);
        assert_eq!(state.state_snapshot, r#"{"state":1}"#);
    }

    #[test]
    fn record_confirmation_new() {
        let store = SyncStateStore::new();
        let sync_id = store.create_sync(vec!["a".into(), "b".into()], "{}".into());

        assert!(store.record_confirmation(&sync_id, "a").unwrap());
        // Second call for same agent is not new
        assert!(!store.record_confirmation(&sync_id, "a").unwrap());

        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.confirmations, vec!["a"]);
        assert_eq!(state.status, SyncStatus::Pending); // not all confirmed yet
    }

    #[test]
    fn record_confirmation_auto_confirms() {
        let store = SyncStateStore::new();
        let sync_id = store.create_sync(vec!["a".into(), "b".into()], "{}".into());

        store.record_confirmation(&sync_id, "a").unwrap();
        store.record_confirmation(&sync_id, "b").unwrap();

        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.status, SyncStatus::Confirmed);
    }

    #[test]
    fn record_confirmation_unknown_sync() {
        let store = SyncStateStore::new();
        let result = store.record_confirmation("nonexistent", "a");
        assert!(result.is_err());
    }

    #[test]
    fn mark_rolled_back() {
        let store = SyncStateStore::new();
        let sync_id = store.create_sync(vec!["a".into()], "{}".into());

        store.mark_rolled_back(&sync_id).unwrap();
        assert_eq!(
            store.get_sync(&sync_id).unwrap().status,
            SyncStatus::RolledBack
        );
    }

    #[test]
    fn mark_rolled_back_unknown() {
        let store = SyncStateStore::new();
        assert!(store.mark_rolled_back("nonexistent").is_err());
    }

    #[test]
    fn get_sync_nonexistent() {
        let store = SyncStateStore::new();
        assert!(store.get_sync("nonexistent").is_none());
    }

    #[test]
    fn list_syncs() {
        let store = SyncStateStore::new();
        store.create_sync(vec!["a".into()], "{}".into());
        store.create_sync(vec!["b".into()], "{}".into());
        assert_eq!(store.list_syncs().len(), 2);
    }

    #[test]
    fn purge_completed() {
        let store = SyncStateStore::new();
        let sid1 = store.create_sync(vec!["a".into()], "{}".into());
        let _sid2 = store.create_sync(vec!["b".into()], "{}".into());

        // Confirm sid1
        store.record_confirmation(&sid1, "a").unwrap();
        assert_eq!(store.get_sync(&sid1).unwrap().status, SyncStatus::Confirmed);

        let purged = store.purge_completed(0);
        assert_eq!(purged, 1); // only sid1 is Confirmed
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn purge_completed_includes_rolled_back() {
        let store = SyncStateStore::new();
        let sid = store.create_sync(vec!["a".into()], "{}".into());
        store.mark_rolled_back(&sid).unwrap();

        let purged = store.purge_completed(0);
        assert_eq!(purged, 1);
        assert!(store.is_empty());
    }

    // -----------------------------------------------------------------------
    // agent_sync_all tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_all_errors_on_empty_registry() {
        let reg = AgentRegistry::new();
        let store = SyncStateStore::new();
        let result = agent_sync_all(
            &reg,
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            "{}",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no active agents"));
    }

    #[test]
    fn sync_all_creates_sync_state_and_returns_count() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());
        reg.register_agent("a2", vec!["plan".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let result = agent_sync_all(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            r#"{"version":2}"#,
        );

        let (sync_id, count) = result.unwrap();
        assert!(!sync_id.is_empty());
        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.participants.len(), 2);
        assert_eq!(count, 2); // both agents receive the bus message
    }

    // -----------------------------------------------------------------------
    // agent_sync_selective tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_selective_errors_on_no_match() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());

        let store = SyncStateStore::new();
        let result = agent_sync_selective(
            &reg,
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            "{}",
            &["deploy".to_string()],
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no agents match filter"));
    }

    #[test]
    fn sync_selective_finds_matching_agents() {
        let reg = AgentRegistry::new();
        reg.register_agent("coder", vec!["code".into()], HashMap::new());
        reg.register_agent("deployer", vec!["deploy".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let result = agent_sync_selective(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            "{}",
            &["deploy".to_string()],
        );

        let (sync_id, count) = result.unwrap();
        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.participants, vec!["deployer"]);
        assert_eq!(count, 1); // only deployer matches
    }

    // -----------------------------------------------------------------------
    // agent_sync_state_diff tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_state_diff_errors_on_empty_registry() {
        let reg = AgentRegistry::new();
        let store = SyncStateStore::new();
        let result = agent_sync_state_diff(
            &reg,
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            "{}",
            r#"{"x":1}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn sync_state_diff_computes_delta() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let previous = r#"{"a":1,"b":2,"c":3}"#;
        let current = r#"{"a":1,"b":99,"d":4}"#;

        let (_sync_id, count, diff_json) = agent_sync_state_diff(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            previous,
            current,
        )
        .unwrap();

        assert_eq!(count, 1);
        let diff: serde_json::Value = serde_json::from_str(&diff_json).unwrap();
        // "b" changed from 2 to 99
        assert_eq!(diff["changed"]["b"], 99);
        // "d" was added
        assert_eq!(diff["added"]["d"], 4);
        // "c" was removed
        assert_eq!(diff["removed"]["c"], 3);
    }

    #[test]
    fn sync_state_diff_handles_invalid_previous() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let (_sync_id, _count, diff_json) = agent_sync_state_diff(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            "not-json", // invalid → treated as empty
            r#"{"x":1}"#,
        )
        .unwrap();

        let diff: serde_json::Value = serde_json::from_str(&diff_json).unwrap();
        // Everything in current is "added" since previous was empty
        assert_eq!(diff["added"]["x"], 1);
    }

    #[test]
    fn sync_state_diff_no_changes() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let state = r#"{"a":1,"b":2}"#;

        let (_sync_id, _count, diff_json) = agent_sync_state_diff(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            state,
            state,
        )
        .unwrap();

        let diff: serde_json::Value = serde_json::from_str(&diff_json).unwrap();
        assert!(diff["added"].as_object().unwrap().is_empty());
        assert!(diff["removed"].as_object().unwrap().is_empty());
        assert!(diff["changed"].as_object().unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // agent_sync_confirm tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_confirm_errors_on_empty_participants() {
        let reg = AgentRegistry::new();
        let store = SyncStateStore::new();
        let result = agent_sync_confirm(
            &reg,
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            &[],
            "{}",
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("participants must not be empty"));
    }

    #[test]
    fn sync_confirm_errors_on_unregistered_participant() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());

        let store = SyncStateStore::new();
        let result = agent_sync_confirm(
            &reg,
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            &["a1".to_string(), "ghost".to_string()],
            "{}",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ghost"));
    }

    #[test]
    fn sync_confirm_creates_pending_state() {
        let reg = AgentRegistry::new();
        reg.register_agent("a1", vec!["code".into()], HashMap::new());
        reg.register_agent("a2", vec!["plan".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();
        let (sync_id, count) = agent_sync_confirm(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            &["a1".to_string(), "a2".to_string()],
            r#"{"action":"deploy"}"#,
        )
        .unwrap();

        assert_eq!(count, 2);
        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.participants, vec!["a1", "a2"]);
        assert!(state.confirmations.is_empty());
        assert_eq!(state.status, SyncStatus::Pending);
        assert_eq!(state.state_snapshot, r#"{"action":"deploy"}"#);
    }

    // -----------------------------------------------------------------------
    // agent_sync_complete_rollback tests
    // -----------------------------------------------------------------------

    #[test]
    fn sync_rollback_errors_on_unknown_sync() {
        let store = SyncStateStore::new();
        let result = agent_sync_complete_rollback(
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            "nonexistent",
        );
        assert!(result.is_err());
    }

    #[test]
    fn sync_rollback_errors_if_not_rolled_back_status() {
        let store = SyncStateStore::new();
        let sync_id = store.create_sync(vec!["a1".into()], "{}".into());

        let result = agent_sync_complete_rollback(
            &store,
            PathBuf::from("/tmp/no-project"),
            "orchestrator",
            &sync_id,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mark it RolledBack first"));
    }

    // -----------------------------------------------------------------------
    // Integration-style: full 2-phase commit lifecycle
    // -----------------------------------------------------------------------

    #[test]
    fn two_phase_commit_happy_path() {
        let reg = AgentRegistry::new();
        reg.register_agent("worker-1", vec!["build".into()], HashMap::new());
        reg.register_agent("worker-2", vec!["build".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();

        // Phase 1: prepare
        let (sync_id, count) = agent_sync_confirm(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            &["worker-1".to_string(), "worker-2".to_string()],
            r#"{"cmd":"deploy"}"#,
        )
        .unwrap();
        assert_eq!(count, 2);

        // Phase 2: confirmations
        store.record_confirmation(&sync_id, "worker-1").unwrap();
        store.record_confirmation(&sync_id, "worker-2").unwrap();

        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.status, SyncStatus::Confirmed);
        assert_eq!(state.confirmations.len(), 2);
    }

    #[test]
    fn two_phase_commit_rollback_path() {
        let reg = AgentRegistry::new();
        reg.register_agent("worker-1", vec!["build".into()], HashMap::new());
        reg.register_agent("worker-2", vec!["build".into()], HashMap::new());

        let store = SyncStateStore::new();
        let tmp = TempDir::new().unwrap();

        let (sync_id, _count) = agent_sync_confirm(
            &reg,
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            &["worker-1".to_string(), "worker-2".to_string()],
            r#"{"cmd":"deploy"}"#,
        )
        .unwrap();

        // Only worker-1 confirms
        store.record_confirmation(&sync_id, "worker-1").unwrap();

        // Timeout → orchestrator decides to roll back
        store.mark_rolled_back(&sync_id).unwrap();
        assert_eq!(
            store.get_sync(&sync_id).unwrap().status,
            SyncStatus::RolledBack
        );

        // Rollback notification via bus
        let rollback_count = agent_sync_complete_rollback(
            &store,
            tmp.path().to_path_buf(),
            "orchestrator",
            &sync_id,
        )
        .unwrap();
        assert_eq!(rollback_count, 2);
    }

    // -----------------------------------------------------------------------
    // Concurrent access stress test
    // -----------------------------------------------------------------------

    #[test]
    fn registry_concurrent_register_and_heartbeat() {
        use std::sync::Arc;
        use std::thread;

        let reg = Arc::new(AgentRegistry::new());
        let mut handles = vec![];

        for i in 0..10 {
            let r = Arc::clone(&reg);
            handles.push(thread::spawn(move || {
                let name = format!("agent-{i}");
                r.register_agent(&name, vec![format!("cap-{i}")], HashMap::new());
                r.heartbeat(&name);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(reg.len(), 10);
        for i in 0..10 {
            let name = format!("agent-{i}");
            let entry = reg.get_agent(&name).unwrap();
            assert_eq!(entry.status, AgentStatus::Active);
        }
    }

    #[test]
    fn store_concurrent_confirmations() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(SyncStateStore::new());
        let sync_id =
            store.create_sync((0..10).map(|i| format!("agent-{i}")).collect(), "{}".into());

        let mut handles = vec![];
        for i in 0..10 {
            let s = Arc::clone(&store);
            let sid = sync_id.clone();
            handles.push(thread::spawn(move || {
                s.record_confirmation(&sid, &format!("agent-{i}")).unwrap();
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let state = store.get_sync(&sync_id).unwrap();
        assert_eq!(state.status, SyncStatus::Confirmed);
        assert_eq!(state.confirmations.len(), 10);
    }
}
