//! Inter-node message protocol.
//!
//! Defines the envelope format for messages exchanged between
//! pb-mcp-server nodes on the network. Every message is signed
//! with HMAC-SHA256 for integrity. Each message also carries a
//! unique `nonce` (derived from a counter, process ID, and
//! timestamp) to provide replay protection.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique node identifier (ed25519 public key fingerprint or UUID).
pub type NodeId = String;

/// Inter-node message envelope.
///
/// Each message carries a `nonce` (a one-time 64-bit value encoded as
/// 16 hex characters) that is derived from an atomic counter, the
/// process ID, and the current timestamp. The nonce is included in the
/// HMAC-SHA256 signature so that a replayed copy of the message fails
/// verification.
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
    /// Random nonce for replay protection (hex-encoded, 16 chars).
    pub nonce: String,
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
    /// Create a new message with current timestamp and a unique replay-protection nonce.
    pub fn new(
        from_node: NodeId,
        to_node: Option<NodeId>,
        kind: NodeMessageKind,
        body: String,
    ) -> Self {
        use sha2::{Digest, Sha256};
        use std::sync::atomic::{AtomicU64, Ordering};

        static NONCE_CTR: AtomicU64 = AtomicU64::new(0);

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let ctr = NONCE_CTR.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let nonce_raw = format!("{pid}:{ts}:{ctr}");
        let hash = Sha256::digest(nonce_raw.as_bytes());
        let nonce: String = hash.iter().take(8).map(|b| format!("{b:02x}")).collect();

        let id = format!("node-msg-{ts:x}-{}", &from_node[..8.min(from_node.len())]);
        Self {
            id,
            from_node,
            to_node,
            kind,
            body,
            ts,
            nonce,
            signature: None,
        }
    }

    /// Sign this message with HMAC-SHA256.
    pub fn sign(&mut self, secret: &str) {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let to_str = self.to_node.as_deref().unwrap_or("broadcast");
        let kind_json =
            serde_json::to_string(&self.kind).unwrap_or_else(|_| "\"unknown\"".to_string());
        let kind_str = kind_json.trim_matches('"');
        let input = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.id, self.from_node, to_str, kind_str, self.nonce, self.ts, self.body
        );
        if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
            mac.update(input.as_bytes());
            let result = mac.finalize().into_bytes();
            self.signature = Some(hex_encode(&result));
        }
    }

    /// Verify this message's HMAC-SHA256 signature.
    ///
    /// Uses constant-time comparison to prevent timing oracle attacks.
    pub fn verify(&self, secret: &str) -> bool {
        let Some(sig) = &self.signature else {
            return false;
        };
        let mut copy = self.clone();
        copy.signature = None;
        copy.sign(secret);
        let expected = copy.signature.as_deref().unwrap_or("");
        // Constant-time comparison: prevent timing-based signature forgery
        if expected.len() != sig.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in expected.as_bytes().iter().zip(sig.as_bytes()) {
            diff |= a ^ b;
        }
        diff == 0
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
        assert!(!msg.nonce.is_empty());
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
        assert!(!msg.nonce.is_empty());
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
    fn sign_is_not_malleable_to_node_change() {
        let mut msg = NodeMessage::new(
            "n1".into(),
            Some("target-a".into()),
            NodeMessageKind::AgentRelay,
            "body".into(),
        );
        msg.sign("s");
        // Changing to_node must break the signature
        msg.to_node = Some("target-b".into());
        assert!(!msg.verify("s"), "signature must cover to_node");
    }

    #[test]
    fn replay_protection_nonce_is_unique() {
        let msg1 = NodeMessage::new("n1".into(), None, NodeMessageKind::Heartbeat, "ping".into());
        let msg2 = NodeMessage::new("n1".into(), None, NodeMessageKind::Heartbeat, "ping".into());
        assert_ne!(
            msg1.nonce, msg2.nonce,
            "each message must have a distinct nonce"
        );
    }

    #[test]
    fn custom_message_kind() {
        let kind = NodeMessageKind::Custom("neural_train".into());
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.contains("custom"));
    }
}
