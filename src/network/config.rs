//! Network configuration for pb-mcp-server nodes.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Network configuration for a pb-mcp-server node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Unique node identifier.
    pub node_id: String,
    /// List of peer addresses to connect to.
    pub peers: Vec<String>,
    /// Listen address for incoming connections.
    pub listen_addr: SocketAddr,
    /// Enable Tor routing for outbound connections.
    pub tor_enabled: bool,
    /// Tor SOCKS5 proxy address.
    pub tor_socks_addr: String,
    /// HMAC signing secret for inter-node messages.
    pub signing_secret: String,
    /// Enable gossipsub for bus event replication.
    pub gossip_enabled: bool,
    /// Maximum number of peer connections.
    pub max_peers: usize,
    /// Node capabilities advertised to the network.
    pub capabilities: Vec<String>,
    /// Network type label.
    pub network_type: NetworkType,
}

/// Network transport type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkType {
    /// Direct LAN/WAN TCP connections.
    Lan,
    /// Routed through Tor network.
    Tor,
    /// WebSocket-based (for browser/proxy scenarios).
    WebSocket,
    /// Custom user-defined transport.
    Custom(String),
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            node_id: format!("node-{}", hex::encode(&[0u8; 8])),
            peers: Vec::new(),
            listen_addr: SocketAddr::from(([0, 0, 0, 0], 9200)),
            tor_enabled: false,
            tor_socks_addr: "127.0.0.1:9050".to_string(),
            signing_secret: String::new(),
            gossip_enabled: true,
            max_peers: 50,
            capabilities: vec!["mcp-server".into(), "agent-bus".into()],
            network_type: NetworkType::Lan,
        }
    }
}

impl NetworkConfig {
    /// Build config from environment variables.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(id) = std::env::var("PB_NODE_ID") {
            cfg.node_id = id;
        }
        if let Ok(peers) = std::env::var("PB_NODE_PEERS") {
            cfg.peers = peers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(addr) = std::env::var("PB_NODE_LISTEN") {
            if let Ok(a) = addr.parse() {
                cfg.listen_addr = a;
            }
        }
        if let Ok(v) = std::env::var("PB_TOR_ENABLED") {
            cfg.tor_enabled = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("PB_TOR_SOCKS") {
            cfg.tor_socks_addr = v;
        }
        if let Ok(v) = std::env::var("PB_NODE_SECRET") {
            cfg.signing_secret = v;
        }
        if let Ok(v) = std::env::var("PB_NODE_MAX_PEERS") {
            if let Ok(n) = v.parse() {
                cfg.max_peers = n;
            }
        }
        cfg.warn_if_weak_secret();
        cfg
    }

    /// Returns `true` if the signing secret is non-empty and at least 32 characters long.
    ///
    /// A short or absent secret provides no real security — HMAC-SHA256 accepts
    /// any key length, but a key shorter than 32 bytes is weaker than the output.
    pub fn has_strong_signing_secret(&self) -> bool {
        self.signing_secret.len() >= 32
    }

    /// Emits a [`tracing::warn!`] when the signing secret is missing or weak.
    ///
    /// Call this once after constructing the config via [`NetworkConfig::from_env`].
    pub fn warn_if_weak_secret(&self) {
        if self.signing_secret.is_empty() {
            tracing::warn!(
                "PB_NODE_SECRET is not set — inter-node messages will be signed with an \
                 empty key, which provides no integrity guarantee. \
                 Set PB_NODE_SECRET to a strong secret (≥32 chars) for production use."
            );
        } else if !self.has_strong_signing_secret() {
            tracing::warn!(
                secret_len = self.signing_secret.len(),
                "PB_NODE_SECRET is set but short — use a secret of at least 32 characters \
                 for production security."
            );
        }
    }
}

// Simple hex encode (no external dep needed)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_defaults() {
        let cfg = NetworkConfig::default();
        assert!(!cfg.node_id.is_empty());
        assert!(cfg.peers.is_empty());
        assert!(!cfg.tor_enabled);
        assert!(cfg.gossip_enabled);
        assert_eq!(cfg.max_peers, 50);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let cfg = NetworkConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: NetworkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.node_id, back.node_id);
        assert_eq!(cfg.tor_enabled, back.tor_enabled);
    }

    #[test]
    fn network_type_serialization() {
        let nt = NetworkType::Tor;
        let json = serde_json::to_string(&nt).unwrap();
        assert_eq!(json, "\"tor\"");
    }

    // -------------------------------------------------------------------------
    // Test 4 — from_env env-var injection
    //
    // NOTE: These tests mutate the process environment.  Because Rust runs
    // tests in parallel threads by default each test sets a *distinct* var
    // (no two tests below touch the same key) to avoid cross-test races.
    // If you ever add two tests for the same key, protect them with a mutex
    // or run the suite with RUST_TEST_THREADS=1.
    // -------------------------------------------------------------------------

    #[test]
    fn from_env_picks_up_node_id() {
        std::env::set_var("PB_NODE_ID", "integration-node-42");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_ID");
        assert_eq!(
            cfg.node_id, "integration-node-42",
            "PB_NODE_ID should override the default node_id"
        );
    }

    #[test]
    fn from_env_picks_up_peers_list() {
        // Comma-separated list with surrounding whitespace should be trimmed.
        std::env::set_var("PB_NODE_PEERS", "10.0.0.1:9200, 10.0.0.2:9200");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_PEERS");
        assert_eq!(cfg.peers.len(), 2, "expected exactly 2 peers");
        assert!(
            cfg.peers.contains(&"10.0.0.1:9200".to_string()),
            "10.0.0.1:9200 should be in the peers list"
        );
        assert!(
            cfg.peers.contains(&"10.0.0.2:9200".to_string()),
            "10.0.0.2:9200 should be in the peers list"
        );
    }

    #[test]
    fn from_env_picks_up_listen_addr() {
        std::env::set_var("PB_NODE_LISTEN", "127.0.0.1:19200");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_LISTEN");
        assert_eq!(
            cfg.listen_addr.to_string(),
            "127.0.0.1:19200",
            "PB_NODE_LISTEN should override the listen address"
        );
    }

    #[test]
    fn from_env_picks_up_tor_flag_numeric() {
        std::env::set_var("PB_TOR_ENABLED", "1");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_TOR_ENABLED");
        assert!(
            cfg.tor_enabled,
            "PB_TOR_ENABLED=1 should set tor_enabled to true"
        );
    }

    #[test]
    fn from_env_picks_up_tor_flag_text_true() {
        // Uses a distinct env-var value from the numeric test to avoid
        // interference if both run concurrently on the same key.
        std::env::set_var("PB_TOR_ENABLED", "True");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_TOR_ENABLED");
        assert!(
            cfg.tor_enabled,
            "PB_TOR_ENABLED=True (case-insensitive) should set tor_enabled to true"
        );
    }

    #[test]
    fn from_env_tor_flag_stays_false_when_zero() {
        std::env::set_var("PB_TOR_ENABLED", "0");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_TOR_ENABLED");
        assert!(
            !cfg.tor_enabled,
            "PB_TOR_ENABLED=0 should leave tor_enabled as false"
        );
    }

    #[test]
    fn from_env_picks_up_max_peers() {
        std::env::set_var("PB_NODE_MAX_PEERS", "7");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_MAX_PEERS");
        assert_eq!(
            cfg.max_peers, 7,
            "PB_NODE_MAX_PEERS should override the default max_peers"
        );
    }

    #[test]
    fn from_env_picks_up_signing_secret() {
        std::env::set_var("PB_NODE_SECRET", "s3cr3t-hmac-key");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_SECRET");
        assert_eq!(
            cfg.signing_secret, "s3cr3t-hmac-key",
            "PB_NODE_SECRET should set the signing_secret field"
        );
    }

    #[test]
    fn from_env_empty_peers_var_produces_no_peers() {
        // An empty PB_NODE_PEERS string should parse to zero peers after
        // filtering — not a single peer with an empty string address.
        std::env::set_var("PB_NODE_PEERS", "");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_PEERS");
        assert!(
            cfg.peers.is_empty(),
            "an empty PB_NODE_PEERS value should produce an empty peers list"
        );
    }

    #[test]
    fn from_env_node_id_overrides_default() {
        // Confirm the env-supplied node_id differs from the hardcoded default.
        std::env::set_var("PB_NODE_ID", "override-id");
        let cfg = NetworkConfig::from_env();
        std::env::remove_var("PB_NODE_ID");
        let default_id = NetworkConfig::default().node_id;
        assert_ne!(
            cfg.node_id, default_id,
            "a set PB_NODE_ID must differ from the default node_id"
        );
        assert_eq!(cfg.node_id, "override-id");
    }

    // -------------------------------------------------------------------------
    // Test 5 — Node ID generation
    // -------------------------------------------------------------------------

    /// The default `node_id` is produced by hex-encoding a fixed `[0u8; 8]`
    /// seed, so it must always start with `"node-"` and be followed by
    /// exactly 16 lowercase hex digits.
    #[test]
    fn default_node_id_has_correct_prefix_and_hex_format() {
        let id = NetworkConfig::default().node_id;

        assert!(
            id.starts_with("node-"),
            "node_id must start with 'node-', got: {id:?}"
        );

        let hex_part = &id["node-".len()..];
        assert_eq!(
            hex_part.len(),
            16,
            "hex suffix should be 16 chars (8 bytes × 2 hex digits), got {hex_part:?}"
        );
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "hex suffix must contain only 0-9 / a-f characters, got {hex_part:?}"
        );
    }

    /// Because the seed is a fixed `[0u8; 8]` array (not random), calling
    /// `NetworkConfig::default()` twice must produce the *identical*
    /// `node_id`.  The ID must also be non-empty in every call.
    #[test]
    fn default_node_id_is_deterministic_and_non_empty() {
        let id1 = NetworkConfig::default().node_id;
        let id2 = NetworkConfig::default().node_id;

        assert!(!id1.is_empty(), "node_id must never be empty");
        assert_eq!(
            id1, id2,
            "two NetworkConfig::default() calls must produce the same node_id \
             (seed is a fixed zero-byte array, not random)"
        );
    }

    /// The fixed zero-byte seed should produce the all-zeros hex string.
    /// This test pins the exact expected value so regressions are obvious.
    #[test]
    fn default_node_id_expected_value() {
        let id = NetworkConfig::default().node_id;
        // hex::encode(&[0u8; 8]) == "0000000000000000"
        assert_eq!(
            id, "node-0000000000000000",
            "default node_id should be the hex encoding of eight zero bytes"
        );
    }

    #[test]
    fn has_strong_signing_secret_empty() {
        let cfg = NetworkConfig::default(); // signing_secret is ""
        assert!(!cfg.has_strong_signing_secret());
    }

    #[test]
    fn has_strong_signing_secret_short() {
        let mut cfg = NetworkConfig::default();
        cfg.signing_secret = "tooshort".to_string();
        assert!(!cfg.has_strong_signing_secret());
    }

    #[test]
    fn has_strong_signing_secret_exactly_32() {
        let mut cfg = NetworkConfig::default();
        cfg.signing_secret = "a".repeat(32);
        assert!(cfg.has_strong_signing_secret());
    }

    #[test]
    fn has_strong_signing_secret_long() {
        let mut cfg = NetworkConfig::default();
        cfg.signing_secret =
            "a-very-long-secret-that-is-definitely-more-than-32-characters".to_string();
        assert!(cfg.has_strong_signing_secret());
    }
}
