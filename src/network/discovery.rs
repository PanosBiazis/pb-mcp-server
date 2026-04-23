//! Node discovery mechanisms for finding peers on the network.
//!
//! Supports:
//! - Static peer list (from config)
//! - mDNS for LAN auto-discovery
//! - Kademlia DHT for wider-area discovery (when libp2p is enabled)
//! - Tor hidden service directory (via subprocess)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Information about a discovered network node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Node identifier.
    pub node_id: String,
    /// Network address (host:port or .onion address).
    pub address: String,
    /// Advertised capabilities.
    pub capabilities: Vec<String>,
    /// Last seen timestamp (ms since epoch).
    pub last_seen: u64,
    /// Current connection status.
    pub status: PeerStatus,
    /// Round-trip latency in milliseconds (if measured).
    pub latency_ms: Option<u64>,
}

/// Connection status of a peer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PeerStatus {
    /// Actively connected.
    Connected,
    /// Known but not currently connected.
    Known,
    /// Recently failed connection attempt.
    Failed,
    /// Blacklisted due to misbehavior.
    Blacklisted,
}

/// Node discovery manager.
#[derive(Debug)]
pub struct NodeDiscovery {
    /// Known peers indexed by node_id.
    peers: HashMap<String, PeerInfo>,
    /// Blacklisted addresses.
    blacklist: Vec<String>,
}

impl NodeDiscovery {
    /// Create a new discovery manager.
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            blacklist: Vec::new(),
        }
    }

    /// Add a peer from static configuration.
    pub fn add_static_peer(&mut self, node_id: String, address: String, capabilities: Vec<String>) {
        let peer = PeerInfo {
            node_id: node_id.clone(),
            address,
            capabilities,
            last_seen: now_ms(),
            status: PeerStatus::Known,
            latency_ms: None,
        };
        self.peers.insert(node_id, peer);
    }

    /// Mark a peer as connected.
    pub fn mark_connected(&mut self, node_id: &str) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.status = PeerStatus::Connected;
            peer.last_seen = now_ms();
        }
    }

    /// Mark a peer as failed.
    pub fn mark_failed(&mut self, node_id: &str) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.status = PeerStatus::Failed;
            peer.last_seen = now_ms();
        }
    }

    /// Record a latency measurement.
    pub fn record_latency(&mut self, node_id: &str, latency_ms: u64) {
        if let Some(peer) = self.peers.get_mut(node_id) {
            peer.latency_ms = Some(latency_ms);
        }
    }

    /// Blacklist a peer address.
    pub fn blacklist(&mut self, address: String) {
        self.blacklist.push(address);
    }

    /// Check if an address is blacklisted.
    pub fn is_blacklisted(&self, address: &str) -> bool {
        self.blacklist.iter().any(|a| a == address)
    }

    /// Get all known peers.
    pub fn list_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }

    /// Get only connected peers.
    pub fn connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.status == PeerStatus::Connected)
            .collect()
    }

    /// Find peers with a specific capability.
    pub fn find_by_capability(&self, capability: &str) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.capabilities.iter().any(|c| c == capability))
            .collect()
    }

    /// Number of known peers.
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Whether no peers are known.
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Remove stale peers not seen within the given duration.
    pub fn prune_stale(&mut self, max_age: Duration) {
        let cutoff = now_ms().saturating_sub(max_age.as_millis() as u64);
        self.peers
            .retain(|_, p| p.last_seen >= cutoff || p.status == PeerStatus::Connected);
    }
}

impl Default for NodeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_new_is_empty() {
        let d = NodeDiscovery::new();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn add_and_mark_connected() {
        let mut d = NodeDiscovery::new();
        d.add_static_peer(
            "n1".into(),
            "1.2.3.4:9200".into(),
            vec!["mcp-server".into()],
        );
        assert_eq!(d.len(), 1);
        assert!(d.list_peers()[0].status == PeerStatus::Known);
        d.mark_connected("n1");
        assert!(d.connected_peers().len() == 1);
    }

    #[test]
    fn find_by_capability() {
        let mut d = NodeDiscovery::new();
        d.add_static_peer(
            "n1".into(),
            "1.2.3.4:9200".into(),
            vec!["mcp-server".into(), "ocr".into()],
        );
        d.add_static_peer(
            "n2".into(),
            "5.6.7.8:9200".into(),
            vec!["mcp-server".into()],
        );
        assert_eq!(d.find_by_capability("ocr").len(), 1);
        assert_eq!(d.find_by_capability("mcp-server").len(), 2);
    }

    #[test]
    fn blacklist_check() {
        let mut d = NodeDiscovery::new();
        d.blacklist("evil.onion:9200".into());
        assert!(d.is_blacklisted("evil.onion:9200"));
        assert!(!d.is_blacklisted("good.host:9200"));
    }

    #[test]
    fn prune_stale_removes_old() {
        let mut d = NodeDiscovery::new();
        d.add_static_peer("n1".into(), "1.2.3.4:9200".into(), vec![]);
        d.peers.get_mut("n1").unwrap().last_seen = 0; // ancient
        d.add_static_peer("n2".into(), "5.6.7.8:9200".into(), vec![]);
        d.mark_connected("n2");
        d.prune_stale(Duration::from_secs(60));
        assert_eq!(d.len(), 1); // only connected n2 survives
    }
}
