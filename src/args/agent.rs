//! Agent bus argument structures
//!
//! This module contains argument structures for multi-agent coordination tools,
//! including message bus operations, handoffs, and synchronization.

use schemars::JsonSchema;
use serde::Deserialize;

use super::defaults;

// ============================================================================
// Message Sending Operations
// ============================================================================

/// Arguments for sending a message on the agent bus
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentSendArgs {
    /// Stable sender id for this session (alphanumeric, _, -, .)
    /// e.g., "cursor-main", "review-bot"
    pub from_agent: String,

    /// Message body; truncated if extremely long
    pub message: String,

    /// Recipient id for a DM; omit to broadcast to every agent polling the bus
    #[serde(default)]
    pub to_agent: Option<String>,

    /// Thread key; use the same string on every agent_send in a thread,
    /// then read with agent_thread
    #[serde(default)]
    pub topic: Option<String>,

    /// Short label: note, question, suggestion, handoff, etc.
    #[serde(default)]
    pub kind: Option<String>,
}

impl Default for AgentSendArgs {
    fn default() -> Self {
        AgentSendArgs {
            from_agent: String::new(),
            message: String::new(),
            to_agent: None,
            topic: None,
            kind: None,
        }
    }
}

/// Arguments for agent handoff (structured message)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentHandoffArgs {
    /// Sender agent id
    pub from_agent: String,

    /// Recipient agent id
    pub to_agent: String,

    /// Task description
    pub task: String,

    /// Context/data for the task
    #[serde(default)]
    pub context: Option<String>,

    /// Expected deliverables
    #[serde(default)]
    pub deliverables: Option<Vec<String>>,

    /// Deadline hint (ISO 8601 or relative like "2h")
    #[serde(default)]
    pub deadline: Option<String>,
}

/// A single step in a pipeline
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentPipelineStep {
    /// Message for this step
    pub message: String,

    /// Recipient for this step
    #[serde(default)]
    pub to_agent: Option<String>,

    /// Topic for this step
    #[serde(default)]
    pub topic: Option<String>,

    /// Kind label for this step
    #[serde(default)]
    pub kind: Option<String>,
}

/// Arguments for sending a pipeline of messages (serial handoff)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentSendPipelineArgs {
    /// Sender agent id
    pub from_agent: String,

    /// Ordered steps (max 24). Serial: rows appended in sequence under one lock
    pub steps: Vec<AgentPipelineStep>,
}

/// Arguments for fan-out delivery to multiple agents
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentFanOutArgs {
    /// Sender agent id
    pub from_agent: String,

    /// Identical body for each DM (parallel fan-out: one exclusive lock, many lines)
    pub message: String,

    /// Recipient agent ids (max 32, deduped); each must differ from from_agent
    pub to_agents: Vec<String>,

    /// Topic for all messages
    #[serde(default)]
    pub topic: Option<String>,

    /// Kind label for all messages
    #[serde(default)]
    pub kind: Option<String>,
}

// ============================================================================
// Message Retrieval Operations
// ============================================================================

/// Arguments for reading agent inbox
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentInboxArgs {
    /// Same stable id as when this agent calls agent_send
    pub for_agent: String,

    /// Max messages (1–200), newest among matches after filtering
    #[serde(default = "defaults::default_inbox_limit")]
    pub limit: usize,

    /// Skip own broadcasts
    #[serde(default = "defaults::default_true")]
    pub skip_own_broadcasts: bool,

    /// If set, only messages with ts_ms strictly greater than this value
    /// (incremental polling; store last seen ts from output headers)
    #[serde(default)]
    pub since_ts_ms: Option<u64>,
}

impl Default for AgentInboxArgs {
    fn default() -> Self {
        AgentInboxArgs {
            for_agent: String::new(),
            limit: defaults::default_inbox_limit(),
            skip_own_broadcasts: true,
            since_ts_ms: None,
        }
    }
}

/// Arguments for reading a topic thread
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentThreadArgs {
    /// Exact topic string shared across agent_send calls
    pub topic: String,

    /// Max messages in the thread window (1–200)
    #[serde(default = "defaults::default_thread_limit")]
    pub limit: usize,

    /// If set, only messages with ts_ms >= this value
    /// (slice the tail of the thread after this time)
    #[serde(default)]
    pub since_ts_ms: Option<u64>,
}

impl Default for AgentThreadArgs {
    fn default() -> Self {
        AgentThreadArgs {
            topic: String::new(),
            limit: defaults::default_thread_limit(),
            since_ts_ms: None,
        }
    }
}

/// Arguments for reading recent bus messages
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentRecentArgs {
    /// Tail size of the global bus (1–150), after optional since filter
    #[serde(default = "defaults::default_recent_limit")]
    pub limit: usize,

    /// If set, only messages newer than this ts_ms are eligible,
    /// then take the last `limit` of those
    #[serde(default)]
    pub since_ts_ms: Option<u64>,
}

impl Default for AgentRecentArgs {
    fn default() -> Self {
        AgentRecentArgs {
            limit: defaults::default_recent_limit(),
            since_ts_ms: None,
        }
    }
}

/// Arguments for listing active topics
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentListTopicsArgs {
    /// Max topic rows to return (1–200), ordered by most recent activity
    #[serde(default = "defaults::default_list_topics_limit")]
    pub limit: usize,
}

impl Default for AgentListTopicsArgs {
    fn default() -> Self {
        AgentListTopicsArgs {
            limit: defaults::default_list_topics_limit(),
        }
    }
}

/// Arguments for getting a specific message by ID
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentGetMessageArgs {
    /// Exact message id from agent_send OK line or bus listing
    /// (e.g., "1775308370692-144329")
    pub message_id: String,
}

/// Arguments for multi-agent inbox (parallel fan-in)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentMultiInboxArgs {
    /// Up to 16 agent ids; one shared bus read, sections per id (parallel fan-in)
    pub for_agents: Vec<String>,

    /// Max messages per agent
    #[serde(default = "defaults::default_inbox_limit")]
    pub limit: usize,

    /// Skip broadcasts from each agent to itself
    #[serde(default = "defaults::default_true")]
    pub skip_own_broadcasts: bool,

    /// Filter by timestamp
    #[serde(default)]
    pub since_ts_ms: Option<u64>,
}

// ============================================================================
// Waiting Operations
// ============================================================================

/// Arguments for waiting on inbox messages
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentInboxWaitArgs {
    /// Same stable id as agent_inbox
    pub for_agent: String,

    /// Max messages to return
    #[serde(default = "defaults::default_inbox_limit")]
    pub limit: usize,

    /// Skip own broadcasts
    #[serde(default = "defaults::default_true")]
    pub skip_own_broadcasts: bool,

    /// Timestamp filter
    #[serde(default)]
    pub since_ts_ms: Option<u64>,

    /// Max time for this single MCP call (100–120000ms)
    /// Server sleeps between polls—async agents prefer agent_inbox + since_ts_ms instead
    #[serde(default = "defaults::default_inbox_wait_timeout_ms")]
    pub timeout_ms: u64,

    /// Delay between inbox polls (50–2000ms)
    #[serde(default = "defaults::default_inbox_wait_poll_ms")]
    pub poll_interval_ms: u64,
}

impl Default for AgentInboxWaitArgs {
    fn default() -> Self {
        AgentInboxWaitArgs {
            for_agent: String::new(),
            limit: defaults::default_inbox_limit(),
            skip_own_broadcasts: true,
            since_ts_ms: None,
            timeout_ms: defaults::default_inbox_wait_timeout_ms(),
            poll_interval_ms: defaults::default_inbox_wait_poll_ms(),
        }
    }
}

/// Arguments for waiting on topic replies
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentTopicWaitArgs {
    /// Thread key to watch (same as agent_send topic)
    pub topic: String,

    /// Your from_agent id; their posts are ignored so you wait for other agents
    pub exclude_from_agent: String,

    /// Only consider messages with ts_ms >= this
    /// Omit to use server time at call start (wait for new replies only)
    #[serde(default)]
    pub since_ts_ms: Option<u64>,

    /// Max wait time
    #[serde(default = "defaults::default_inbox_wait_timeout_ms")]
    pub timeout_ms: u64,

    /// Poll interval
    #[serde(default = "defaults::default_inbox_wait_poll_ms")]
    pub poll_interval_ms: u64,
}

impl Default for AgentTopicWaitArgs {
    fn default() -> Self {
        AgentTopicWaitArgs {
            topic: String::new(),
            exclude_from_agent: String::new(),
            since_ts_ms: None,
            timeout_ms: defaults::default_inbox_wait_timeout_ms(),
            poll_interval_ms: defaults::default_inbox_wait_poll_ms(),
        }
    }
}

// ============================================================================
// Coordination Operations
// ============================================================================

/// Arguments for asking multiple agents (broadcast + wait for replies)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AskAgentsArgs {
    /// Sender agent id
    pub from_agent: String,

    /// Question or request to broadcast
    pub question: String,

    /// Topic key for responses
    #[serde(default)]
    pub topic: Option<String>,

    /// Expected number of respondents (wait until this many or timeout)
    #[serde(default)]
    pub expected_count: Option<usize>,

    /// Max wait time in milliseconds
    #[serde(default = "defaults::default_inbox_wait_timeout_ms")]
    pub timeout_ms: u64,

    /// Poll interval in milliseconds
    #[serde(default = "defaults::default_inbox_wait_poll_ms")]
    pub poll_interval_ms: u64,
}

impl Default for AskAgentsArgs {
    fn default() -> Self {
        AskAgentsArgs {
            from_agent: String::new(),
            question: String::new(),
            topic: None,
            expected_count: None,
            timeout_ms: defaults::default_inbox_wait_timeout_ms(),
            poll_interval_ms: defaults::default_inbox_wait_poll_ms(),
        }
    }
}

/// Arguments for consensus-based decision making
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentConsensusArgs {
    /// Sender agent id
    pub from_agent: String,

    /// Question or proposal for consensus
    pub proposal: String,

    /// Agent ids to ask for consensus
    pub participants: Vec<String>,

    /// Topic key for coordination
    pub topic: String,

    /// Required agreement threshold (e.g., 0.6 for 60%)
    #[serde(default = "default_consensus_threshold")]
    pub threshold: f64,

    /// Max wait time
    #[serde(default = "default_consensus_timeout")]
    pub timeout_ms: u64,
}

fn default_consensus_threshold() -> f64 {
    0.6
}

fn default_consensus_timeout() -> u64 {
    60_000
}

impl Default for AgentConsensusArgs {
    fn default() -> Self {
        AgentConsensusArgs {
            from_agent: String::new(),
            proposal: String::new(),
            participants: Vec::new(),
            topic: String::new(),
            threshold: default_consensus_threshold(),
            timeout_ms: default_consensus_timeout(),
        }
    }
}

/// Arguments for subagent task delegation
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SubagentTaskArgs {
    /// Parent agent id
    pub from_agent: String,

    /// Subagent id (or auto-generate if None)
    #[serde(default)]
    pub subagent_id: Option<String>,

    /// Task description
    pub task: String,

    /// Task priority (1-5, higher = more important)
    #[serde(default = "default_task_priority")]
    pub priority: u8,

    /// Task deadline (ISO 8601 or relative)
    #[serde(default)]
    pub deadline: Option<String>,

    /// Context data for the task
    #[serde(default)]
    pub context: Option<String>,
}

fn default_task_priority() -> u8 {
    3
}

impl Default for SubagentTaskArgs {
    fn default() -> Self {
        SubagentTaskArgs {
            from_agent: String::new(),
            subagent_id: None,
            task: String::new(),
            priority: default_task_priority(),
            deadline: None,
            context: None,
        }
    }
}

/// Arguments for marking subagent task as done
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SubagentDoneArgs {
    /// Subagent id
    pub subagent_id: String,

    /// Task id that was completed
    pub task_id: String,

    /// Result or deliverable
    pub result: String,

    /// Whether task was successful
    #[serde(default = "defaults::default_true")]
    pub success: bool,

    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,
}

// ============================================================================
// Bus Management Operations
// ============================================================================

/// Arguments for bus status check
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentBusStatusArgs {
    /// Whether to include message count
    #[serde(default = "defaults::default_true")]
    pub include_count: bool,

    /// Whether to include file size
    #[serde(default = "defaults::default_true")]
    pub include_size: bool,
}

impl Default for AgentBusStatusArgs {
    fn default() -> Self {
        AgentBusStatusArgs {
            include_count: true,
            include_size: true,
        }
    }
}

/// Arguments for bus compaction
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentBusCompactArgs {
    /// Keep messages newer than this timestamp
    #[serde(default)]
    pub keep_since_ts_ms: Option<u64>,

    /// Keep at least this many messages (tail)
    #[serde(default = "default_compact_keep_count")]
    pub keep_count: usize,
}

fn default_compact_keep_count() -> usize {
    1000
}

impl Default for AgentBusCompactArgs {
    fn default() -> Self {
        AgentBusCompactArgs {
            keep_since_ts_ms: None,
            keep_count: default_compact_keep_count(),
        }
    }
}

/// Arguments for bus search
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentBusSearchArgs {
    /// Regex pattern to search in message bodies
    pub pattern: String,

    /// Filter by sender
    #[serde(default)]
    pub from_agent: Option<String>,

    /// Filter by recipient
    #[serde(default)]
    pub to_agent: Option<String>,

    /// Filter by topic
    #[serde(default)]
    pub topic: Option<String>,

    /// Filter by kind
    #[serde(default)]
    pub kind: Option<String>,

    /// Max results
    #[serde(default = "defaults::default_grep_max")]
    pub limit: usize,

    /// Case-sensitive search
    #[serde(default = "defaults::default_true")]
    pub case_sensitive: bool,
}

impl Default for AgentBusSearchArgs {
    fn default() -> Self {
        AgentBusSearchArgs {
            pattern: String::new(),
            from_agent: None,
            to_agent: None,
            topic: None,
            kind: None,
            limit: defaults::default_grep_max(),
            case_sensitive: true,
        }
    }
}

/// Arguments for agent roster (activity stats)
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AgentRosterArgs {
    /// Only show agents active since this timestamp
    #[serde(default)]
    pub active_since_ts_ms: Option<u64>,

    /// Max agents to return
    #[serde(default = "default_roster_limit")]
    pub limit: usize,
}

fn default_roster_limit() -> usize {
    50
}

impl Default for AgentRosterArgs {
    fn default() -> Self {
        AgentRosterArgs {
            active_since_ts_ms: None,
            limit: default_roster_limit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_send_args_default() {
        let args = AgentSendArgs::default();
        assert!(args.from_agent.is_empty());
        assert!(args.message.is_empty());
        assert!(args.to_agent.is_none());
        assert!(args.topic.is_none());
        assert!(args.kind.is_none());
    }

    #[test]
    fn test_agent_inbox_args_default() {
        let args = AgentInboxArgs::default();
        assert!(args.for_agent.is_empty());
        assert_eq!(args.limit, 50);
        assert!(args.skip_own_broadcasts);
        assert!(args.since_ts_ms.is_none());
    }

    #[test]
    fn test_agent_thread_args_default() {
        let args = AgentThreadArgs::default();
        assert!(args.topic.is_empty());
        assert_eq!(args.limit, 100);
    }

    #[test]
    fn test_agent_inbox_wait_args_default() {
        let args = AgentInboxWaitArgs::default();
        assert!(args.for_agent.is_empty());
        assert_eq!(args.timeout_ms, 30_000);
        assert_eq!(args.poll_interval_ms, 200);
    }

    #[test]
    fn test_ask_agents_args_default() {
        let args = AskAgentsArgs::default();
        assert!(args.from_agent.is_empty());
        assert!(args.question.is_empty());
        assert!(args.topic.is_none());
        assert!(args.expected_count.is_none());
    }

    #[test]
    fn test_consensus_args_default() {
        let args = AgentConsensusArgs::default();
        assert!(args.from_agent.is_empty());
        assert!(args.proposal.is_empty());
        assert!(args.participants.is_empty());
        assert!((args.threshold - 0.6).abs() < 0.01);
        assert_eq!(args.timeout_ms, 60_000);
    }

    #[test]
    fn test_subagent_task_args_default() {
        let args = SubagentTaskArgs::default();
        assert!(args.from_agent.is_empty());
        assert!(args.subagent_id.is_none());
        assert!(args.task.is_empty());
        assert_eq!(args.priority, 3);
        assert!(args.deadline.is_none());
    }

    #[test]
    fn test_agent_bus_status_args_default() {
        let args = AgentBusStatusArgs::default();
        assert!(args.include_count);
        assert!(args.include_size);
    }

    #[test]
    fn test_agent_bus_search_args_default() {
        let args = AgentBusSearchArgs::default();
        assert!(args.pattern.is_empty());
        assert!(args.from_agent.is_none());
        assert!(args.case_sensitive);
        assert_eq!(args.limit, 200);
    }
}
