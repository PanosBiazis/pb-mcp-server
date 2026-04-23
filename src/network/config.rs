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
            listen_addr: "0.0.0.0:9200".parse().unwrap(),
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
        cfg
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
}
