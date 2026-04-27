//! # Agent Module
//!
//! Multi-agent coordination primitives built on top of the durable JSONL
//! message bus (`crate::bus`).
//!
//! ## Submodules
//!
//! - **sync** – Agent registry, state-diff sync, selective broadcast, and
//!   two-phase-commit confirmation patterns.
//! - **consensus** – Multi-agent consensus / voting system with pluggable
//!   strategies (simple majority, super-majority, unanimous, weighted,
//!   ranked-choice, delegation).
//! - **live** – Real-time streaming, P2P connections, event pub/sub,
//!   heartbeat/ping monitoring, and live broadcast (feature-gated behind
//!   `web-websocket`).
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use pb_mcp_server::agent::sync::{
//!     AgentRegistry, SyncStateStore, agent_sync_all, agent_sync_confirm,
//! };
//!
//! let registry = AgentRegistry::new();
//! registry.register_agent("planner", vec!["plan".into()], Default::default());
//!
//! let store = SyncStateStore::new();
//! let n = agent_sync_all(&registry, &store, root, "orchestrator", "{}")?;
//! ```
//!
//! For real-time event streaming:
//!
//! ```rust,ignore
//! use pb_mcp_server::agent::live::{LiveEventBus, LiveConnectionManager};
//!
//! let bus = LiveEventBus::new();
//! bus.create_stream("builds", "ci_bot", None)?;
//! bus.subscribe_record_only("builds", "agent_a")?;
//! bus.publish("builds", "ci_bot", "build_result", "OK")?;
//! ```
//!
//! For multi-agent consensus:
//!
//! ```rust,ignore
//! use pb_mcp_server::agent::ConsensusEngine;
//! use pb_mcp_server::agent::consensus::ConsensusStrategy;
//!
//! let mut engine = ConsensusEngine::new();
//! let proposal = engine.create_proposal(
//!     "orchestrator", "Deploy", "Deploy to prod?",
//!     vec!["yes".into(), "no".into()],
//!     60_000, ConsensusStrategy::SimpleMajority,
//! );
//! engine.cast_vote(&proposal.id, "agent_a", "yes", 0.9, None)?;
//! let result = engine.tally(&proposal.id)?;
//! ```

// ============================================================================
// Submodule declarations
// ============================================================================

/// Agent synchronisation tools: registry, broadcast, state-diff,
/// selective sync, and two-phase-commit confirmation.
pub mod sync;

/// Multi-agent consensus / voting system: proposals, vote casting,
/// delegation, and pluggable tallying strategies.
pub mod consensus;

/// Agent live tools: real-time streaming, P2P connections, event pub/sub,
/// heartbeat/ping monitoring, and live broadcast.
///
/// This module is only available when the `web-websocket` feature is enabled.
#[cfg(feature = "web-websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-websocket")))]
pub mod live;

/// Agent worker system: rig-based LLM-backed agent workers.
/// Only available when the `agent-workers` feature is enabled.
#[cfg(feature = "agent-workers")]
#[cfg_attr(docsrs, doc(cfg(feature = "agent-workers")))]
pub mod worker;

// ============================================================================
// Re-exports for ergonomic access
// ============================================================================

pub use consensus::ConsensusEngine;

#[cfg(feature = "web-websocket")]
pub use live::{
    live_broadcast, live_get_status, live_subscribe, AgentHealth, AgentHealthReport,
    ConnectionStatus, HeartbeatConfig, HeartbeatMonitor, LiveConnection, LiveConnectionManager,
    LiveEventBus, LiveStatusReport, LiveStreamInfo, PingResult,
};

#[cfg(feature = "agent-workers")]
pub use worker::{
    AgentRole, AgentWorker, AgentWorkerConfig, AgentWorkerStatus, WorkerInfo, WorkerPool,
};
