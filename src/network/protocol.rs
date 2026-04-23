//! Inter-node message protocol.
//!
//! Defines the envelope format for messages exchanged between
//! pb-mcp-server nodes on the network. Every message is signed
//! with HMAC-SHA256 for integrity.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique node identifier (ed25519 public key fingerprint or UUID).
pub type NodeId = String;

/// Inter-node message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMessage {
    /// Unique message ID.
    pub id: String,
    /// Sender node ID.
    pub from_node: NodeId,
    /// Target node ID (None = broadcast to all peers).
    pub to_node: Option<NodeId>,
    /// Message kind/category.
    pub kind: NodeMessageKind,
    /// Message payload (typically JSON).
    pub body: String,
    /// Timestamp in ms since epoch.
    pub ts: u64,
    /// HMAC-SHA256 signature (hex).
    pub signature: Option<String>,
}

/// Categories of inter-node messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeMessageKind {
    /// Agent-to-agent communication across nodes.
    AgentRelay,
    /// Task delegation from one node to another.
    TaskDelegate,
    /// Task result coming back.
    TaskResult,
    /// Health/heartbeat ping.
    Heartbeat,
    /// Node capability advertisement.
    Capabilities,
    /// Bus event replication.
    BusReplication,
    /// Network management (join, leave, topology).
    NetworkControl,
    /// Custom user-defined message type.
    Custom(String),
}

impl NodeMessage {
    /// Create a new message with current timestamp.
    pub fn new(
        from_node: NodeId,
        to_node: Option<NodeId>,
        kind: NodeMessageKind,
        body: String,
    ) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let id = format!("node-msg-{ts:x}-{}", &from_node[..8.min(from_node.len())]);
        Self {
            id,
            from_node,
            to_node,
            kind,
            body,
            ts,
            signature: None,
        }
    }

    /// Sign this message with HMAC-SHA256.
    pub fn sign(&mut self, secret: &str) {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let input = format!("{}:{}:{}:{}", self.id, self.from_node, self.ts, self.body);
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key length is valid");
        mac.update(input.as_bytes());
        let result = mac.finalize().into_bytes();
        self.signature = Some(hex_encode(&result));
    }

    /// Verify this message's HMAC-SHA256 signature.
    pub fn verify(&self, secret: &str) -> bool {
        let Some(sig) = &self.signature else {
            return false;
        };
        let mut copy = self.clone();
        copy.signature = None;
        copy.sign(secret);
        copy.signature.as_deref() == Some(sig.as_str())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_message_new_has_timestamp() {
        let msg = NodeMessage::new(
            "node-1".into(),
            None,
            NodeMessageKind::Heartbeat,
            "ping".into(),
        );
        assert!(msg.ts > 0);
        assert!(msg.id.starts_with("node-msg-"));
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let mut msg = NodeMessage::new(
            "node-1".into(),
            None,
            NodeMessageKind::AgentRelay,
            "hello".into(),
        );
        msg.sign("test-secret");
        assert!(msg.signature.is_some());
        assert!(msg.verify("test-secret"));
        assert!(!msg.verify("wrong-secret"));
    }

    #[test]
    fn verify_fails_without_signature() {
        let msg = NodeMessage::new(
            "node-1".into(),
            None,
            NodeMessageKind::Heartbeat,
            "ping".into(),
        );
        assert!(!msg.verify("any-secret"));
    }

    #[test]
    fn custom_message_kind() {
        let kind = NodeMessageKind::Custom("neural_train".into());
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains("custom"));
    }
}
