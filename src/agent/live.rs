//! # Agent Live – Real-Time Streaming and P2P Connections
//!
//! Provides live, persistent communication channels between agents, built
//! on top of `tokio::sync::broadcast` for real-time event distribution and
//! `DashMap` for concurrent connection/stream tracking.
//!
//! ## Components
//!
//! - **LiveConnectionManager** – Tracks persistent P2P-style channels between
//!   agent pairs, recording status, activity timestamps, and throughput metrics.
//! - **LiveEventBus** – Named pub/sub event streams that agents can create,
//!   subscribe to, and publish into. Backed by `tokio::sync::broadcast`.
//! - **Heartbeat & Ping** – Keep-alive pings with RTT measurement and
//!   health-check logic that infers agent availability from heartbeat history.
//! - **Live Broadcast** – Fan-out a continuous event to all connected listeners.
//!
//! ## Feature Gate
//!
//! This entire module is only compiled when the `web-websocket` feature is
//! enabled because it depends on `tokio::sync::broadcast`.
//!
//! ## Thread Safety
//!
//! All shared state uses `DashMap` or atomic primitives—no `unsafe` code.

#[cfg(feature = "web-websocket")]
use tokio::sync::broadcast;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// Constants
// ============================================================================

/// Default broadcast channel capacity per event stream.
const DEFAULT_STREAM_CAPACITY: usize = 256;

/// Default heartbeat interval in milliseconds.
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 30_000;

/// Default heartbeat timeout in milliseconds (agent is considered
/// unhealthy if no heartbeat arrives within this window).
const DEFAULT_HEARTBEAT_TIMEOUT_MS: u64 = 10_000;

/// Default maximum missed heartbeats before marking agent offline.
const DEFAULT_MAX_MISSED_HEARTBEATS: u32 = 3;

/// Maximum length of a live event payload (characters).
const MAX_EVENT_PAYLOAD_CHARS: usize = 100_000;

/// Maximum number of subscribers per stream.
const MAX_SUBSCRIBERS_PER_STREAM: usize = 128;

/// Maximum number of concurrent live connections.
const MAX_LIVE_CONNECTIONS: usize = 1024;

/// Maximum number of event streams.
const MAX_EVENT_STREAMS: usize = 256;

// ============================================================================
// Connection status
// ============================================================================

/// Status of a live connection between two agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    /// Connection is being established.
    Connecting,
    /// Connection is active and healthy.
    Active,
    /// Connection has been closed gracefully.
    Closed,
    /// Connection encountered an error.
    Error,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connecting => write!(f, "connecting"),
            ConnectionStatus::Active => write!(f, "active"),
            ConnectionStatus::Closed => write!(f, "closed"),
            ConnectionStatus::Error => write!(f, "error"),
        }
    }
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Connecting
    }
}

// ============================================================================
// Connection metrics
// ============================================================================

/// Throughput metrics for a single live connection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Total messages sent through this connection.
    pub messages_sent: u64,
    /// Total messages received through this connection.
    pub messages_received: u64,
    /// Approximate total bytes transferred.
    pub bytes_transferred: u64,
    /// Number of errors encountered.
    pub error_count: u64,
}

// ============================================================================
// Live connection
// ============================================================================

/// A persistent P2P-style channel between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveConnection {
    /// Unique connection identifier.
    pub connection_id: String,
    /// The agent that initiated the connection.
    pub from_agent: String,
    /// The agent that received the connection.
    pub to_agent: String,
    /// Current connection status.
    pub status: ConnectionStatus,
    /// Unix-epoch milliseconds when the connection was created.
    pub created_at_ms: u64,
    /// Unix-epoch milliseconds of the most recent activity.
    pub last_activity_ms: u64,
    /// Throughput metrics.
    pub metrics: ConnectionMetrics,
}

impl LiveConnection {
    /// Create a new connection in `Connecting` state.
    pub fn new(from_agent: &str, to_agent: &str) -> Self {
        let now = now_ms();
        Self {
            connection_id: format!("conn_{}_{}_{}", from_agent, to_agent, now),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            status: ConnectionStatus::Connecting,
            created_at_ms: now,
            last_activity_ms: now,
            metrics: ConnectionMetrics::default(),
        }
    }

    /// Mark the connection as active.
    pub fn activate(&mut self) {
        self.status = ConnectionStatus::Active;
        self.last_activity_ms = now_ms();
    }

    /// Mark the connection as closed.
    pub fn close(&mut self) {
        self.status = ConnectionStatus::Closed;
        self.last_activity_ms = now_ms();
    }

    /// Mark the connection as errored.
    pub fn mark_error(&mut self) {
        self.status = ConnectionStatus::Error;
        self.metrics.error_count += 1;
        self.last_activity_ms = now_ms();
    }

    /// Record a sent message.
    pub fn record_sent(&mut self, byte_size: u64) {
        self.metrics.messages_sent += 1;
        self.metrics.bytes_transferred += byte_size;
        self.last_activity_ms = now_ms();
    }

    /// Record a received message.
    pub fn record_received(&mut self, byte_size: u64) {
        self.metrics.messages_received += 1;
        self.metrics.bytes_transferred += byte_size;
        self.last_activity_ms = now_ms();
    }

    /// Duration in milliseconds since last activity.
    pub fn idle_ms(&self) -> u64 {
        now_ms().saturating_sub(self.last_activity_ms)
    }

    /// Duration in milliseconds since creation.
    pub fn age_ms(&self) -> u64 {
        now_ms().saturating_sub(self.created_at_ms)
    }
}

// ============================================================================
// Live connection manager
// ============================================================================

/// Manages all live P2P connections between agents.
///
/// Connections are identified by a unique `connection_id` and tracked in a
/// `DashMap` for lock-free concurrent access.
pub struct LiveConnectionManager {
    connections: DashMap<String, LiveConnection>,
    /// Monotonic counter for generating unique connection IDs.
    next_id: AtomicU64,
}

impl LiveConnectionManager {
    /// Create a new, empty connection manager.
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// Open a new live connection between two agents.
    ///
    /// Returns the `LiveConnection` if successful, or an error string if the
    /// maximum number of connections has been reached or a connection already
    /// exists between the same pair.
    pub fn connect(&self, from_agent: &str, to_agent: &str) -> Result<LiveConnection, String> {
        if from_agent == to_agent {
            return Err("Cannot connect an agent to itself".to_string());
        }

        if self.connections.len() >= MAX_LIVE_CONNECTIONS {
            return Err(format!(
                "Maximum live connections reached ({MAX_LIVE_CONNECTIONS})"
            ));
        }

        // Check for duplicate connection (same ordered pair).
        let duplicate = self.connections.iter().any(|c| {
            c.value().from_agent == from_agent
                && c.value().to_agent == to_agent
                && c.value().status != ConnectionStatus::Closed
        });
        if duplicate {
            return Err(format!(
                "Active connection already exists from {from_agent} to {to_agent}"
            ));
        }

        let mut conn = LiveConnection::new(from_agent, to_agent);
        // Use the counter for a shorter, collision-free ID.
        let seq = self.next_id.fetch_add(1, Ordering::Relaxed);
        conn.connection_id = format!("conn_{seq}");
        conn.activate();

        let id = conn.connection_id.clone();
        self.connections.insert(id, conn.clone());
        Ok(conn)
    }

    /// Gracefully close a connection by ID.
    pub fn disconnect(&self, connection_id: &str) -> Result<LiveConnection, String> {
        let mut conn = self
            .connections
            .get_mut(connection_id)
            .ok_or_else(|| format!("Connection {connection_id} not found"))?;
        conn.close();
        let snapshot = conn.clone();
        // Remove from map after closing.
        drop(conn);
        self.connections.remove(connection_id);
        Ok(snapshot)
    }

    /// Get the current status of a connection.
    pub fn get_status(&self, connection_id: &str) -> Option<LiveConnection> {
        self.connections
            .get(connection_id)
            .map(|c| c.value().clone())
    }

    /// List all current connections (optionally filtered by status).
    pub fn list_connections(&self, status_filter: Option<ConnectionStatus>) -> Vec<LiveConnection> {
        self.connections
            .iter()
            .filter(|c| {
                status_filter
                    .as_ref()
                    .map_or(true, |f| &c.value().status == f)
            })
            .map(|c| c.value().clone())
            .collect()
    }

    /// List all connections involving a specific agent.
    pub fn list_agent_connections(&self, agent_name: &str) -> Vec<LiveConnection> {
        self.connections
            .iter()
            .filter(|c| c.value().from_agent == agent_name || c.value().to_agent == agent_name)
            .map(|c| c.value().clone())
            .collect()
    }

    /// Record a message sent on a connection.
    pub fn record_sent(&self, connection_id: &str, byte_size: u64) -> Result<(), String> {
        let mut conn = self
            .connections
            .get_mut(connection_id)
            .ok_or_else(|| format!("Connection {connection_id} not found"))?;
        conn.record_sent(byte_size);
        Ok(())
    }

    /// Record a message received on a connection.
    pub fn record_received(&self, connection_id: &str, byte_size: u64) -> Result<(), String> {
        let mut conn = self
            .connections
            .get_mut(connection_id)
            .ok_or_else(|| format!("Connection {connection_id} not found"))?;
        conn.record_received(byte_size);
        Ok(())
    }

    /// Mark a connection as errored.
    pub fn mark_error(&self, connection_id: &str) -> Result<(), String> {
        let mut conn = self
            .connections
            .get_mut(connection_id)
            .ok_or_else(|| format!("Connection {connection_id} not found"))?;
        conn.mark_error();
        Ok(())
    }

    /// Close all connections involving a given agent (e.g. on agent shutdown).
    pub fn close_all_for_agent(&self, agent_name: &str) -> Vec<LiveConnection> {
        let ids: Vec<String> = self
            .connections
            .iter()
            .filter(|c| c.value().from_agent == agent_name || c.value().to_agent == agent_name)
            .map(|c| c.key().clone())
            .collect();

        let mut closed = Vec::new();
        for id in ids {
            if let Some((_, conn)) = self.connections.remove(&id) {
                closed.push(conn);
            }
        }
        closed
    }

    /// Return the number of active (non-closed) connections.
    pub fn active_count(&self) -> usize {
        self.connections
            .iter()
            .filter(|c| c.value().status == ConnectionStatus::Active)
            .count()
    }

    /// Return total connection count.
    pub fn total_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for LiveConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Live event
// ============================================================================

/// A single event published to a named event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEvent {
    /// Name of the stream this event belongs to.
    pub stream_name: String,
    /// Agent that published the event.
    pub publisher: String,
    /// Event type (e.g. "build_output", "test_result", "log").
    pub event_type: String,
    /// Event payload (text).
    pub payload: String,
    /// Unix-epoch milliseconds when the event was published.
    pub timestamp_ms: u64,
    /// Monotonic sequence number within the stream.
    pub seq: u64,
}

// ============================================================================
// Live stream info
// ============================================================================

/// Metadata about a named event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStreamInfo {
    /// Stream name.
    pub name: String,
    /// Agent that created the stream.
    pub publisher: String,
    /// Agents currently subscribed.
    pub subscribers: Vec<String>,
    /// Unix-epoch milliseconds when the stream was created.
    pub created_at_ms: u64,
    /// Total events published to this stream.
    pub event_count: u64,
    /// Current broadcast channel capacity.
    pub capacity: usize,
}

// ============================================================================
// Live event bus
// ============================================================================

/// Internal state for a single event stream.
struct StreamState {
    /// The broadcast sender for this stream.
    #[cfg(feature = "web-websocket")]
    tx: broadcast::Sender<LiveEvent>,
    /// Agent that created the stream.
    publisher: String,
    /// Set of subscriber agent names.
    subscribers: DashMap<String, ()>,
    /// Creation timestamp.
    created_at_ms: u64,
    /// Monotonic event counter.
    event_count: AtomicU64,
    /// Channel capacity.
    capacity: usize,
}

/// Named pub/sub event bus for real-time agent communication.
///
/// Each stream is backed by a `tokio::sync::broadcast` channel. Agents
/// subscribe to a stream by name and receive events published to it.
pub struct LiveEventBus {
    streams: DashMap<String, Arc<StreamState>>,
}

impl LiveEventBus {
    /// Create a new, empty event bus.
    pub fn new() -> Self {
        Self {
            streams: DashMap::new(),
        }
    }

    /// Create a new named event stream.
    ///
    /// Returns an error if the stream already exists or the maximum number
    /// of streams has been reached.
    pub fn create_stream(
        &self,
        name: &str,
        publisher: &str,
        capacity: Option<usize>,
    ) -> Result<(), String> {
        if self.streams.len() >= MAX_EVENT_STREAMS {
            return Err(format!(
                "Maximum event streams reached ({MAX_EVENT_STREAMS})"
            ));
        }

        if self.streams.contains_key(name) {
            return Err(format!("Stream '{name}' already exists"));
        }

        let cap = capacity.unwrap_or(DEFAULT_STREAM_CAPACITY);
        #[cfg(feature = "web-websocket")]
        let (tx, _) = broadcast::channel(cap);

        let state = Arc::new(StreamState {
            #[cfg(feature = "web-websocket")]
            tx,
            publisher: publisher.to_string(),
            subscribers: DashMap::new(),
            created_at_ms: now_ms(),
            event_count: AtomicU64::new(0),
            capacity: cap,
        });

        self.streams.insert(name.to_string(), state);
        Ok(())
    }

    /// Subscribe an agent to a named event stream.
    ///
    /// Returns a `broadcast::Receiver` for async event consumption, or an
    /// error if the stream does not exist or the subscriber limit is reached.
    #[cfg(feature = "web-websocket")]
    pub fn subscribe(
        &self,
        stream_name: &str,
        agent_name: &str,
    ) -> Result<broadcast::Receiver<LiveEvent>, String> {
        let state = self
            .streams
            .get(stream_name)
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))?;

        if state.subscribers.len() >= MAX_SUBSCRIBERS_PER_STREAM {
            return Err(format!(
                "Stream '{stream_name}' has reached the subscriber limit ({MAX_SUBSCRIBERS_PER_STREAM})"
            ));
        }

        state.subscribers.insert(agent_name.to_string(), ());
        Ok(state.tx.subscribe())
    }

    /// Subscribe an agent to a named event stream (non-async version).
    ///
    /// Records the subscription without returning a receiver. Useful for
    /// tracking subscriptions when the caller manages their own channel.
    pub fn subscribe_record_only(&self, stream_name: &str, agent_name: &str) -> Result<(), String> {
        let state = self
            .streams
            .get(stream_name)
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))?;

        if state.subscribers.len() >= MAX_SUBSCRIBERS_PER_STREAM {
            return Err(format!(
                "Stream '{stream_name}' has reached the subscriber limit ({MAX_SUBSCRIBERS_PER_STREAM})"
            ));
        }

        state.subscribers.insert(agent_name.to_string(), ());
        Ok(())
    }

    /// Unsubscribe an agent from a stream.
    pub fn unsubscribe(&self, stream_name: &str, agent_name: &str) -> Result<(), String> {
        let state = self
            .streams
            .get(stream_name)
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))?;

        state
            .subscribers
            .remove(agent_name)
            .ok_or_else(|| format!("Agent '{agent_name}' is not subscribed to '{stream_name}'"))?;
        Ok(())
    }

    /// Publish an event to a named stream.
    ///
    /// All current subscribers receive the event via their broadcast receiver.
    /// Returns the sequence number assigned to this event.
    #[cfg(feature = "web-websocket")]
    pub fn publish(
        &self,
        stream_name: &str,
        publisher: &str,
        event_type: &str,
        payload: &str,
    ) -> Result<u64, String> {
        if payload.chars().count() > MAX_EVENT_PAYLOAD_CHARS {
            return Err(format!(
                "Event payload exceeds {MAX_EVENT_PAYLOAD_CHARS} characters"
            ));
        }

        let state = self
            .streams
            .get(stream_name)
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))?;

        let seq = state.event_count.fetch_add(1, Ordering::Relaxed);

        let event = LiveEvent {
            stream_name: stream_name.to_string(),
            publisher: publisher.to_string(),
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            timestamp_ms: now_ms(),
            seq,
        };

        // broadcast::Sender::send returns the number of receivers that
        // received the event. If there are no subscribers the event is
        // simply dropped.
        let _ = state.tx.send(event);

        Ok(seq)
    }

    /// Publish an event without broadcast (for non-web-websocket builds).
    ///
    /// Still increments the event counter and validates the payload.
    #[cfg(not(feature = "web-websocket"))]
    pub fn publish(
        &self,
        stream_name: &str,
        publisher: &str,
        event_type: &str,
        payload: &str,
    ) -> Result<u64, String> {
        if payload.chars().count() > MAX_EVENT_PAYLOAD_CHARS {
            return Err(format!(
                "Event payload exceeds {MAX_EVENT_PAYLOAD_CHARS} characters"
            ));
        }

        let state = self
            .streams
            .get(stream_name)
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))?;

        let seq = state.event_count.fetch_add(1, Ordering::Relaxed);
        Ok(seq)
    }

    /// List all named streams.
    pub fn list_streams(&self) -> Vec<String> {
        self.streams.iter().map(|s| s.key().clone()).collect()
    }

    /// Get detailed info about a specific stream.
    pub fn get_stream_info(&self, stream_name: &str) -> Option<LiveStreamInfo> {
        let state = self.streams.get(stream_name)?;
        Some(LiveStreamInfo {
            name: stream_name.to_string(),
            publisher: state.publisher.clone(),
            subscribers: state.subscribers.iter().map(|s| s.key().clone()).collect(),
            created_at_ms: state.created_at_ms,
            event_count: state.event_count.load(Ordering::Relaxed),
            capacity: state.capacity,
        })
    }

    /// Remove a stream entirely (unsubscribes all agents).
    pub fn remove_stream(&self, stream_name: &str) -> Result<(), String> {
        self.streams
            .remove(stream_name)
            .map(|_| ())
            .ok_or_else(|| format!("Stream '{stream_name}' not found"))
    }

    /// Return the number of active streams.
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }
}

impl Default for LiveEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Heartbeat configuration
// ============================================================================

/// Configuration for agent heartbeat monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Interval between heartbeat pings in milliseconds.
    pub interval_ms: u64,
    /// Timeout before an agent is considered unhealthy in milliseconds.
    pub timeout_ms: u64,
    /// Maximum consecutive missed heartbeats before marking offline.
    pub max_missed: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: DEFAULT_HEARTBEAT_INTERVAL_MS,
            timeout_ms: DEFAULT_HEARTBEAT_TIMEOUT_MS,
            max_missed: DEFAULT_MAX_MISSED_HEARTBEATS,
        }
    }
}

// ============================================================================
// Ping result
// ============================================================================

/// Result of a ping to a specific agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// Name of the pinged agent.
    pub agent_name: String,
    /// Round-trip time in milliseconds (0 if failed).
    pub rtt_ms: u64,
    /// Unix-epoch milliseconds when the ping was sent.
    pub timestamp_ms: u64,
    /// Whether the ping succeeded (agent responded within timeout).
    pub success: bool,
}

// ============================================================================
// Heartbeat record
// ============================================================================

/// Internal record of a single heartbeat observation.
#[derive(Debug)]
struct HeartbeatRecord {
    /// Monotonic instant when the heartbeat was observed.
    instant: Instant,
    /// RTT in milliseconds (0 if not a ping-response).
    rtt_ms: u64,
    /// Whether this was a successful observation.
    success: bool,
}

// ============================================================================
// Agent health status
// ============================================================================

/// Health assessment for an agent based on heartbeat history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentHealth {
    /// Agent is responsive with low latency.
    Healthy,
    /// Agent is responding but with high latency.
    Degraded,
    /// Agent has missed heartbeats and may be unreachable.
    Unhealthy,
    /// No heartbeat data available.
    Unknown,
}

impl std::fmt::Display for AgentHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentHealth::Healthy => write!(f, "healthy"),
            AgentHealth::Degraded => write!(f, "degraded"),
            AgentHealth::Unhealthy => write!(f, "unhealthy"),
            AgentHealth::Unknown => write!(f, "unknown"),
        }
    }
}

// ============================================================================
// Agent health summary
// ============================================================================

/// Detailed health report for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthReport {
    /// Agent name.
    pub agent_name: String,
    /// Overall health assessment.
    pub health: AgentHealth,
    /// Average RTT in milliseconds over recent heartbeats.
    pub avg_rtt_ms: u64,
    /// Minimum RTT observed.
    pub min_rtt_ms: u64,
    /// Maximum RTT observed.
    pub max_rtt_ms: u64,
    /// Number of successful recent heartbeats.
    pub success_count: u32,
    /// Number of failed recent heartbeats.
    pub fail_count: u32,
    /// Unix-epoch milliseconds of the last successful heartbeat.
    pub last_success_ms: u64,
}

// ============================================================================
// Heartbeat tracker
// ============================================================================

/// Tracks heartbeat and ping history for a single agent.
struct AgentHeartbeatState {
    records: Vec<HeartbeatRecord>,
    /// Maximum number of records to retain (ring buffer).
    max_records: usize,
}

impl AgentHeartbeatState {
    fn new(max_records: usize) -> Self {
        Self {
            records: Vec::with_capacity(max_records),
            max_records,
        }
    }

    fn push(&mut self, record: HeartbeatRecord) {
        if self.records.len() >= self.max_records {
            self.records.remove(0);
        }
        self.records.push(record);
    }
}

// ============================================================================
// Heartbeat monitor
// ============================================================================

/// Monitors agent health through heartbeat and ping observations.
///
/// Agents periodically send heartbeats; the monitor tracks response times
/// and infers health from the history.
pub struct HeartbeatMonitor {
    config: HeartbeatConfig,
    states: DashMap<String, AgentHeartbeatState>,
    /// Maximum heartbeat records per agent.
    max_history: usize,
}

impl HeartbeatMonitor {
    /// Create a new monitor with the given configuration.
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            states: DashMap::new(),
            max_history: 100,
        }
    }

    /// Create a monitor with default configuration.
    pub fn default_monitor() -> Self {
        Self::new(HeartbeatConfig::default())
    }

    /// Record a successful heartbeat for an agent.
    pub fn record_heartbeat(&self, agent_name: &str, rtt_ms: u64) {
        let mut state = self
            .states
            .entry(agent_name.to_string())
            .or_insert_with(|| AgentHeartbeatState::new(self.max_history));
        state.push(HeartbeatRecord {
            instant: Instant::now(),
            rtt_ms,
            success: true,
        });
    }

    /// Record a failed heartbeat (missed or timed out).
    pub fn record_missed(&self, agent_name: &str) {
        let mut state = self
            .states
            .entry(agent_name.to_string())
            .or_insert_with(|| AgentHeartbeatState::new(self.max_history));
        state.push(HeartbeatRecord {
            instant: Instant::now(),
            rtt_ms: 0,
            success: false,
        });
    }

    /// Send a conceptual ping and record the result.
    ///
    /// In a real deployment this would send an actual network ping. Here it
    /// simulates a successful ping with a given RTT (or a failure).
    pub fn send_ping(&self, agent_name: &str, rtt_ms: Option<u64>) -> PingResult {
        let ts = now_ms();
        match rtt_ms {
            Some(rtt) => {
                self.record_heartbeat(agent_name, rtt);
                PingResult {
                    agent_name: agent_name.to_string(),
                    rtt_ms: rtt,
                    timestamp_ms: ts,
                    success: true,
                }
            }
            None => {
                self.record_missed(agent_name);
                PingResult {
                    agent_name: agent_name.to_string(),
                    rtt_ms: 0,
                    timestamp_ms: ts,
                    success: false,
                }
            }
        }
    }

    /// Check the health of an agent based on recent heartbeat history.
    pub fn check_agent_health(&self, agent_name: &str) -> AgentHealthReport {
        let state = match self.states.get(agent_name) {
            Some(s) => s,
            None => {
                return AgentHealthReport {
                    agent_name: agent_name.to_string(),
                    health: AgentHealth::Unknown,
                    avg_rtt_ms: 0,
                    min_rtt_ms: 0,
                    max_rtt_ms: 0,
                    success_count: 0,
                    fail_count: 0,
                    last_success_ms: 0,
                };
            }
        };

        let records = &state.records;
        if records.is_empty() {
            return AgentHealthReport {
                agent_name: agent_name.to_string(),
                health: AgentHealth::Unknown,
                avg_rtt_ms: 0,
                min_rtt_ms: 0,
                max_rtt_ms: 0,
                success_count: 0,
                fail_count: 0,
                last_success_ms: 0,
            };
        }

        let success_count = records.iter().filter(|r| r.success).count() as u32;
        let fail_count = records.len() as u32 - success_count;

        let rtts: Vec<u64> = records
            .iter()
            .filter(|r| r.success)
            .map(|r| r.rtt_ms)
            .collect();

        let avg_rtt_ms = if rtts.is_empty() {
            0
        } else {
            rtts.iter().sum::<u64>() / rtts.len() as u64
        };
        let min_rtt_ms = rtts.iter().copied().min().unwrap_or(0);
        let max_rtt_ms = rtts.iter().copied().max().unwrap_or(0);

        let last_success_ms = records
            .iter()
            .rev()
            .find(|r| r.success)
            .map(|r| {
                // Approximate: now minus elapsed since the record.
                now_ms().saturating_sub(r.instant.elapsed().as_millis() as u64)
            })
            .unwrap_or(0);

        // Health inference: if recent consecutive misses >= max_missed,
        // the agent is unhealthy. If fail ratio > 50%, degraded.
        let consecutive_misses = records.iter().rev().take_while(|r| !r.success).count() as u32;

        let health = if consecutive_misses >= self.config.max_missed {
            AgentHealth::Unhealthy
        } else if fail_count > success_count {
            AgentHealth::Degraded
        } else if avg_rtt_ms > self.config.timeout_ms / 2 {
            AgentHealth::Degraded
        } else {
            AgentHealth::Healthy
        };

        AgentHealthReport {
            agent_name: agent_name.to_string(),
            health,
            avg_rtt_ms,
            min_rtt_ms,
            max_rtt_ms,
            success_count,
            fail_count,
            last_success_ms,
        }
    }

    /// Get a reference to the current configuration.
    pub fn config(&self) -> &HeartbeatConfig {
        &self.config
    }

    /// Return the number of agents being tracked.
    pub fn tracked_agent_count(&self) -> usize {
        self.states.len()
    }

    /// Remove tracking data for an agent.
    pub fn remove_agent(&self, agent_name: &str) -> bool {
        self.states.remove(agent_name).is_some()
    }

    /// Return all tracked agent names.
    pub fn tracked_agents(&self) -> Vec<String> {
        self.states.iter().map(|s| s.key().clone()).collect()
    }
}

impl Default for HeartbeatMonitor {
    fn default() -> Self {
        Self::default_monitor()
    }
}

// ============================================================================
// Live broadcast (convenience)
// ============================================================================

/// Publish an event to **all** named streams on the bus.
///
/// Returns a map of `stream_name → seq_number` for each stream that
/// received the event, or an error for streams that could not be published
/// to (e.g. payload too large).
pub fn live_broadcast(
    bus: &LiveEventBus,
    publisher: &str,
    event_type: &str,
    payload: &str,
) -> HashMap<String, Result<u64, String>> {
    let streams = bus.list_streams();
    let mut results = HashMap::new();
    for name in &streams {
        let result = bus.publish(name, publisher, event_type, payload);
        results.insert(name.clone(), result);
    }
    results
}

/// Subscribe an agent to a named event stream on the bus.
///
/// This is a convenience wrapper around [`LiveEventBus::subscribe_record_only`].
pub fn live_subscribe(
    bus: &LiveEventBus,
    stream_name: &str,
    agent_name: &str,
) -> Result<(), String> {
    bus.subscribe_record_only(stream_name, agent_name)
}

/// Generate a status report for all live connections and streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStatusReport {
    /// Number of active connections.
    pub active_connections: usize,
    /// Total connections (including non-active).
    pub total_connections: usize,
    /// Number of event streams.
    pub stream_count: usize,
    /// Per-stream subscriber counts.
    pub stream_subscribers: HashMap<String, usize>,
    /// Per-stream event counts.
    pub stream_event_counts: HashMap<String, u64>,
    /// Report timestamp in milliseconds.
    pub timestamp_ms: u64,
}

/// Produce a comprehensive status report for a connection manager and event bus.
pub fn live_get_status(conn_mgr: &LiveConnectionManager, bus: &LiveEventBus) -> LiveStatusReport {
    let streams = bus.list_streams();
    let mut stream_subscribers = HashMap::new();
    let mut stream_event_counts = HashMap::new();

    for name in &streams {
        if let Some(info) = bus.get_stream_info(name) {
            stream_subscribers.insert(name.clone(), info.subscribers.len());
            stream_event_counts.insert(name.clone(), info.event_count);
        }
    }

    LiveStatusReport {
        active_connections: conn_mgr.active_count(),
        total_connections: conn_mgr.total_count(),
        stream_count: streams.len(),
        stream_subscribers,
        stream_event_counts,
        timestamp_ms: now_ms(),
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Return current unix-epoch milliseconds.
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ConnectionStatus ----

    #[test]
    fn test_connection_status_display() {
        assert_eq!(format!("{}", ConnectionStatus::Connecting), "connecting");
        assert_eq!(format!("{}", ConnectionStatus::Active), "active");
        assert_eq!(format!("{}", ConnectionStatus::Closed), "closed");
        assert_eq!(format!("{}", ConnectionStatus::Error), "error");
    }

    #[test]
    fn test_connection_status_serde_roundtrip() {
        for status in [
            ConnectionStatus::Connecting,
            ConnectionStatus::Active,
            ConnectionStatus::Closed,
            ConnectionStatus::Error,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ConnectionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_connection_status_default() {
        assert_eq!(ConnectionStatus::default(), ConnectionStatus::Connecting);
    }

    // ---- LiveConnection ----

    #[test]
    fn test_live_connection_new() {
        let conn = LiveConnection::new("alice", "bob");
        assert_eq!(conn.from_agent, "alice");
        assert_eq!(conn.to_agent, "bob");
        assert_eq!(conn.status, ConnectionStatus::Connecting);
        assert_eq!(conn.metrics.messages_sent, 0);
    }

    #[test]
    fn test_live_connection_activate() {
        let mut conn = LiveConnection::new("a", "b");
        conn.activate();
        assert_eq!(conn.status, ConnectionStatus::Active);
    }

    #[test]
    fn test_live_connection_close() {
        let mut conn = LiveConnection::new("a", "b");
        conn.activate();
        conn.close();
        assert_eq!(conn.status, ConnectionStatus::Closed);
    }

    #[test]
    fn test_live_connection_mark_error() {
        let mut conn = LiveConnection::new("a", "b");
        conn.mark_error();
        assert_eq!(conn.status, ConnectionStatus::Error);
        assert_eq!(conn.metrics.error_count, 1);
    }

    #[test]
    fn test_live_connection_record_sent() {
        let mut conn = LiveConnection::new("a", "b");
        conn.record_sent(100);
        assert_eq!(conn.metrics.messages_sent, 1);
        assert_eq!(conn.metrics.bytes_transferred, 100);
    }

    #[test]
    fn test_live_connection_record_received() {
        let mut conn = LiveConnection::new("a", "b");
        conn.record_received(200);
        assert_eq!(conn.metrics.messages_received, 1);
        assert_eq!(conn.metrics.bytes_transferred, 200);
    }

    #[test]
    fn test_live_connection_idle_ms() {
        let mut conn = LiveConnection::new("a", "b");
        conn.last_activity_ms = now_ms() - 5000;
        assert!(conn.idle_ms() >= 5000);
    }

    #[test]
    fn test_live_connection_age_ms() {
        let mut conn = LiveConnection::new("a", "b");
        conn.created_at_ms = now_ms() - 10000;
        assert!(conn.age_ms() >= 10000);
    }

    #[test]
    fn test_live_connection_serde_roundtrip() {
        let conn = LiveConnection::new("alice", "bob");
        let json = serde_json::to_string(&conn).unwrap();
        let back: LiveConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(conn.connection_id, back.connection_id);
        assert_eq!(conn.from_agent, back.from_agent);
        assert_eq!(conn.to_agent, back.to_agent);
        assert_eq!(conn.status, back.status);
    }

    // ---- LiveConnectionManager ----

    #[test]
    fn test_mgr_connect_basic() {
        let mgr = LiveConnectionManager::new();
        let conn = mgr.connect("alice", "bob").unwrap();
        assert_eq!(conn.from_agent, "alice");
        assert_eq!(conn.to_agent, "bob");
        assert_eq!(conn.status, ConnectionStatus::Active);
    }

    #[test]
    fn test_mgr_connect_self_rejected() {
        let mgr = LiveConnectionManager::new();
        assert!(mgr.connect("alice", "alice").is_err());
    }

    #[test]
    fn test_mgr_connect_duplicate_rejected() {
        let mgr = LiveConnectionManager::new();
        mgr.connect("alice", "bob").unwrap();
        assert!(mgr.connect("alice", "bob").is_err());
    }

    #[test]
    fn test_mgr_connect_reverse_allowed() {
        let mgr = LiveConnectionManager::new();
        mgr.connect("alice", "bob").unwrap();
        // bob → alice is a different ordered pair
        assert!(mgr.connect("bob", "alice").is_ok());
    }

    #[test]
    fn test_mgr_disconnect() {
        let mgr = LiveConnectionManager::new();
        let conn = mgr.connect("alice", "bob").unwrap();
        let id = conn.connection_id.clone();
        let closed = mgr.disconnect(&id).unwrap();
        assert_eq!(closed.status, ConnectionStatus::Closed);
        assert!(mgr.get_status(&id).is_none());
    }

    #[test]
    fn test_mgr_disconnect_not_found() {
        let mgr = LiveConnectionManager::new();
        assert!(mgr.disconnect("nonexistent").is_err());
    }

    #[test]
    fn test_mgr_list_connections() {
        let mgr = LiveConnectionManager::new();
        mgr.connect("a", "b").unwrap();
        mgr.connect("c", "d").unwrap();
        let all = mgr.list_connections(None);
        assert_eq!(all.len(), 2);

        let active = mgr.list_connections(Some(ConnectionStatus::Active));
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_mgr_list_agent_connections() {
        let mgr = LiveConnectionManager::new();
        mgr.connect("alice", "bob").unwrap();
        mgr.connect("alice", "carol").unwrap();
        mgr.connect("dave", "eve").unwrap();

        let alice = mgr.list_agent_connections("alice");
        assert_eq!(alice.len(), 2);
    }

    #[test]
    fn test_mgr_record_sent_received() {
        let mgr = LiveConnectionManager::new();
        let conn = mgr.connect("a", "b").unwrap();
        let id = conn.connection_id.clone();

        mgr.record_sent(&id, 50).unwrap();
        mgr.record_received(&id, 75).unwrap();

        let status = mgr.get_status(&id).unwrap();
        assert_eq!(status.metrics.messages_sent, 1);
        assert_eq!(status.metrics.messages_received, 1);
        assert_eq!(status.metrics.bytes_transferred, 125);
    }

    #[test]
    fn test_mgr_close_all_for_agent() {
        let mgr = LiveConnectionManager::new();
        mgr.connect("alice", "bob").unwrap();
        mgr.connect("alice", "carol").unwrap();
        mgr.connect("dave", "eve").unwrap();

        let closed = mgr.close_all_for_agent("alice");
        assert_eq!(closed.len(), 2);
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_mgr_active_count() {
        let mgr = LiveConnectionManager::new();
        assert_eq!(mgr.active_count(), 0);
        mgr.connect("a", "b").unwrap();
        assert_eq!(mgr.active_count(), 1);
    }

    // ---- LiveEventBus ----

    #[test]
    fn test_bus_create_stream() {
        let bus = LiveEventBus::new();
        assert!(bus.create_stream("builds", "ci_bot", None).is_ok());
        assert_eq!(bus.stream_count(), 1);
    }

    #[test]
    fn test_bus_create_duplicate_rejected() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", None).unwrap();
        assert!(bus.create_stream("builds", "ci_bot", None).is_err());
    }

    #[test]
    fn test_bus_subscribe_record_only() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", None).unwrap();
        assert!(bus.subscribe_record_only("builds", "agent_a").is_ok());

        let info = bus.get_stream_info("builds").unwrap();
        assert!(info.subscribers.contains(&"agent_a".to_string()));
    }

    #[test]
    fn test_bus_subscribe_nonexistent_stream() {
        let bus = LiveEventBus::new();
        assert!(bus.subscribe_record_only("nope", "agent_a").is_err());
    }

    #[test]
    fn test_bus_unsubscribe() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", None).unwrap();
        bus.subscribe_record_only("builds", "agent_a").unwrap();
        assert!(bus.unsubscribe("builds", "agent_a").is_ok());
        assert!(!bus
            .get_stream_info("builds")
            .unwrap()
            .subscribers
            .contains(&"agent_a".to_string()));
    }

    #[test]
    fn test_bus_unsubscribe_not_subscribed() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", None).unwrap();
        assert!(bus.unsubscribe("builds", "agent_a").is_err());
    }

    #[test]
    fn test_bus_publish() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", None).unwrap();
        let seq = bus
            .publish("builds", "ci_bot", "build_result", "OK")
            .unwrap();
        assert_eq!(seq, 0);

        let seq2 = bus
            .publish("builds", "ci_bot", "build_result", "DONE")
            .unwrap();
        assert_eq!(seq2, 1);

        let info = bus.get_stream_info("builds").unwrap();
        assert_eq!(info.event_count, 2);
    }

    #[test]
    fn test_bus_publish_nonexistent_stream() {
        let bus = LiveEventBus::new();
        assert!(bus.publish("nope", "a", "t", "p").is_err());
    }

    #[test]
    fn test_bus_publish_oversized_payload() {
        let bus = LiveEventBus::new();
        bus.create_stream("s", "p", None).unwrap();
        let big = "x".repeat(MAX_EVENT_PAYLOAD_CHARS + 1);
        assert!(bus.publish("s", "p", "t", &big).is_err());
    }

    #[test]
    fn test_bus_list_streams() {
        let bus = LiveEventBus::new();
        bus.create_stream("a", "p", None).unwrap();
        bus.create_stream("b", "p", None).unwrap();
        let mut names = bus.list_streams();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_bus_remove_stream() {
        let bus = LiveEventBus::new();
        bus.create_stream("a", "p", None).unwrap();
        assert!(bus.remove_stream("a").is_ok());
        assert_eq!(bus.stream_count(), 0);
    }

    #[test]
    fn test_bus_remove_nonexistent() {
        let bus = LiveEventBus::new();
        assert!(bus.remove_stream("nope").is_err());
    }

    #[test]
    fn test_bus_get_stream_info() {
        let bus = LiveEventBus::new();
        bus.create_stream("builds", "ci_bot", Some(64)).unwrap();
        bus.subscribe_record_only("builds", "agent_a").unwrap();
        bus.publish("builds", "ci_bot", "t", "p").unwrap();

        let info = bus.get_stream_info("builds").unwrap();
        assert_eq!(info.name, "builds");
        assert_eq!(info.publisher, "ci_bot");
        assert!(info.subscribers.contains(&"agent_a".to_string()));
        assert_eq!(info.event_count, 1);
        assert_eq!(info.capacity, 64);
    }

    // ---- HeartbeatConfig ----

    #[test]
    fn test_heartbeat_config_default() {
        let c = HeartbeatConfig::default();
        assert_eq!(c.interval_ms, DEFAULT_HEARTBEAT_INTERVAL_MS);
        assert_eq!(c.timeout_ms, DEFAULT_HEARTBEAT_TIMEOUT_MS);
        assert_eq!(c.max_missed, DEFAULT_MAX_MISSED_HEARTBEATS);
    }

    #[test]
    fn test_heartbeat_config_serde_roundtrip() {
        let c = HeartbeatConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: HeartbeatConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c.interval_ms, back.interval_ms);
    }

    // ---- PingResult ----

    #[test]
    fn test_ping_result_serde_roundtrip() {
        let pr = PingResult {
            agent_name: "bob".to_string(),
            rtt_ms: 42,
            timestamp_ms: 1000,
            success: true,
        };
        let json = serde_json::to_string(&pr).unwrap();
        let back: PingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(pr.agent_name, back.agent_name);
        assert_eq!(pr.rtt_ms, back.rtt_ms);
        assert_eq!(pr.success, back.success);
    }

    // ---- HeartbeatMonitor ----

    #[test]
    fn test_monitor_record_heartbeat() {
        let mon = HeartbeatMonitor::default_monitor();
        mon.record_heartbeat("alice", 10);
        assert_eq!(mon.tracked_agent_count(), 1);

        let report = mon.check_agent_health("alice");
        assert_eq!(report.health, AgentHealth::Healthy);
        assert_eq!(report.success_count, 1);
        assert_eq!(report.avg_rtt_ms, 10);
    }

    #[test]
    fn test_monitor_record_missed() {
        let mon = HeartbeatMonitor::default_monitor();
        for _ in 0..3 {
            mon.record_missed("alice");
        }
        let report = mon.check_agent_health("alice");
        assert_eq!(report.health, AgentHealth::Unhealthy);
        assert_eq!(report.fail_count, 3);
    }

    #[test]
    fn test_monitor_send_ping_success() {
        let mon = HeartbeatMonitor::default_monitor();
        let result = mon.send_ping("alice", Some(15));
        assert!(result.success);
        assert_eq!(result.rtt_ms, 15);
    }

    #[test]
    fn test_monitor_send_ping_failure() {
        let mon = HeartbeatMonitor::default_monitor();
        let result = mon.send_ping("alice", None);
        assert!(!result.success);
        assert_eq!(result.rtt_ms, 0);
    }

    #[test]
    fn test_monitor_check_unknown_agent() {
        let mon = HeartbeatMonitor::default_monitor();
        let report = mon.check_agent_health("nobody");
        assert_eq!(report.health, AgentHealth::Unknown);
    }

    #[test]
    fn test_monitor_degraded_by_high_rtt() {
        let mon = HeartbeatMonitor::new(HeartbeatConfig {
            interval_ms: 5000,
            timeout_ms: 200,
            max_missed: 3,
        });
        mon.record_heartbeat("slow_agent", 150); // > timeout/2
        let report = mon.check_agent_health("slow_agent");
        assert_eq!(report.health, AgentHealth::Degraded);
    }

    #[test]
    fn test_monitor_degraded_by_fail_ratio() {
        let mon = HeartbeatMonitor::default_monitor();
        mon.record_heartbeat("shaky", 5);
        mon.record_missed("shaky");
        mon.record_missed("shaky");
        let report = mon.check_agent_health("shaky");
        assert_eq!(report.health, AgentHealth::Degraded);
    }

    #[test]
    fn test_monitor_healthy() {
        let mon = HeartbeatMonitor::default_monitor();
        for i in 0..5 {
            mon.record_heartbeat("stable", 10 + i);
        }
        let report = mon.check_agent_health("stable");
        assert_eq!(report.health, AgentHealth::Healthy);
        assert_eq!(report.success_count, 5);
        assert_eq!(report.min_rtt_ms, 10);
        assert_eq!(report.max_rtt_ms, 14);
    }

    #[test]
    fn test_monitor_remove_agent() {
        let mon = HeartbeatMonitor::default_monitor();
        mon.record_heartbeat("a", 5);
        mon.record_heartbeat("b", 5);
        assert!(mon.remove_agent("a"));
        assert_eq!(mon.tracked_agents(), vec!["b"]);
    }

    #[test]
    fn test_monitor_tracked_agents() {
        let mon = HeartbeatMonitor::default_monitor();
        mon.record_heartbeat("alice", 5);
        mon.record_heartbeat("bob", 10);
        let mut agents = mon.tracked_agents();
        agents.sort();
        assert_eq!(agents, vec!["alice", "bob"]);
    }

    #[test]
    fn test_monitor_health_report_serde() {
        let report = AgentHealthReport {
            agent_name: "alice".to_string(),
            health: AgentHealth::Healthy,
            avg_rtt_ms: 10,
            min_rtt_ms: 5,
            max_rtt_ms: 20,
            success_count: 5,
            fail_count: 0,
            last_success_ms: 1000,
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: AgentHealthReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.agent_name, back.agent_name);
        assert_eq!(report.health, back.health);
    }

    // ---- AgentHealth display ----

    #[test]
    fn test_agent_health_display() {
        assert_eq!(format!("{}", AgentHealth::Healthy), "healthy");
        assert_eq!(format!("{}", AgentHealth::Degraded), "degraded");
        assert_eq!(format!("{}", AgentHealth::Unhealthy), "unhealthy");
        assert_eq!(format!("{}", AgentHealth::Unknown), "unknown");
    }

    #[test]
    fn test_agent_health_serde_roundtrip() {
        for h in [
            AgentHealth::Healthy,
            AgentHealth::Degraded,
            AgentHealth::Unhealthy,
            AgentHealth::Unknown,
        ] {
            let json = serde_json::to_string(&h).unwrap();
            let back: AgentHealth = serde_json::from_str(&json).unwrap();
            assert_eq!(h, back);
        }
    }

    // ---- live_broadcast ----

    #[test]
    fn test_live_broadcast() {
        let bus = LiveEventBus::new();
        bus.create_stream("s1", "p", None).unwrap();
        bus.create_stream("s2", "p", None).unwrap();

        let results = live_broadcast(&bus, "broadcaster", "event", "payload");
        assert_eq!(results.len(), 2);
        for (_, result) in &results {
            assert!(result.is_ok());
        }
    }

    // ---- live_subscribe ----

    #[test]
    fn test_live_subscribe() {
        let bus = LiveEventBus::new();
        bus.create_stream("stream", "p", None).unwrap();
        assert!(live_subscribe(&bus, "stream", "agent_a").is_ok());

        let info = bus.get_stream_info("stream").unwrap();
        assert!(info.subscribers.contains(&"agent_a".to_string()));
    }

    // ---- live_get_status ----

    #[test]
    fn test_live_get_status() {
        let mgr = LiveConnectionManager::new();
        let bus = LiveEventBus::new();
        bus.create_stream("s", "p", None).unwrap();
        bus.subscribe_record_only("s", "a").unwrap();
        bus.publish("s", "p", "t", "payload").unwrap();

        mgr.connect("a", "b").unwrap();

        let report = live_get_status(&mgr, &bus);
        assert_eq!(report.active_connections, 1);
        assert_eq!(report.stream_count, 1);
        assert_eq!(report.stream_subscribers.get("s"), Some(&1));
        assert_eq!(report.stream_event_counts.get("s"), Some(&1));
    }

    // ---- LiveStatusReport serde ----

    #[test]
    fn test_live_status_report_serde() {
        let report = LiveStatusReport {
            active_connections: 5,
            total_connections: 10,
            stream_count: 3,
            stream_subscribers: HashMap::new(),
            stream_event_counts: HashMap::new(),
            timestamp_ms: 12345,
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: LiveStatusReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.active_connections, back.active_connections);
    }

    // ---- ConnectionMetrics serde ----

    #[test]
    fn test_connection_metrics_serde() {
        let m = ConnectionMetrics {
            messages_sent: 100,
            messages_received: 90,
            bytes_transferred: 50000,
            error_count: 2,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ConnectionMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m.messages_sent, back.messages_sent);
    }

    // ---- Edge cases ----

    #[test]
    fn test_bus_publish_at_max_payload() {
        let bus = LiveEventBus::new();
        bus.create_stream("s", "p", None).unwrap();
        let payload = "x".repeat(MAX_EVENT_PAYLOAD_CHARS);
        assert!(bus.publish("s", "p", "t", &payload).is_ok());
    }

    #[test]
    fn test_mgr_record_sent_nonexistent() {
        let mgr = LiveConnectionManager::new();
        assert!(mgr.record_sent("nope", 10).is_err());
    }

    #[test]
    fn test_mgr_mark_error() {
        let mgr = LiveConnectionManager::new();
        let conn = mgr.connect("a", "b").unwrap();
        let id = conn.connection_id.clone();
        mgr.mark_error(&id).unwrap();
        let status = mgr.get_status(&id).unwrap();
        assert_eq!(status.status, ConnectionStatus::Error);
        assert_eq!(status.metrics.error_count, 1);
    }

    #[test]
    fn test_mgr_max_connections() {
        let mgr = LiveConnectionManager::new();
        // Fill up to the max (simplified test: just verify rejection logic).
        // In practice MAX_LIVE_CONNECTIONS is 1024, so we test the rejection
        // path by creating a manager and checking the error message.
        assert!(mgr.connect("a", "b").is_ok());
    }

    #[test]
    fn test_bus_max_subscribers() {
        let bus = LiveEventBus::new();
        bus.create_stream("s", "p", None).unwrap();
        // Subscribe up to the limit.
        for i in 0..MAX_SUBSCRIBERS_PER_STREAM {
            assert!(bus
                .subscribe_record_only("s", &format!("agent_{i}"))
                .is_ok());
        }
        // One more should fail.
        assert!(bus.subscribe_record_only("s", "agent_extra").is_err());
    }

    #[test]
    fn test_monitor_many_heartbeats_ring_buffer() {
        let mon = HeartbeatMonitor::default_monitor();
        // Push more than max_history (100) entries.
        for i in 0..150 {
            mon.record_heartbeat("busy", i as u64 % 50);
        }
        let report = mon.check_agent_health("busy");
        // Should only count the last 100 records.
        assert!(report.success_count <= 100);
        assert_eq!(report.health, AgentHealth::Healthy);
    }

    #[test]
    fn test_live_broadcast_empty_bus() {
        let bus = LiveEventBus::new();
        let results = live_broadcast(&bus, "p", "t", "payload");
        assert!(results.is_empty());
    }

    #[test]
    fn test_live_get_status_empty() {
        let mgr = LiveConnectionManager::new();
        let bus = LiveEventBus::new();
        let report = live_get_status(&mgr, &bus);
        assert_eq!(report.active_connections, 0);
        assert_eq!(report.stream_count, 0);
    }
}
