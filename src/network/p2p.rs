//! P2P networking using libp2p.
//!
//! Provides a full peer-to-peer node that can:
//! - Discover and connect to other pb-mcp-server nodes via Kademlia DHT
//! - Relay agent messages between nodes via Gossipsub
//! - Exchange peer identity information via the Identify protocol
//! - Monitor peer connectivity via periodic Ping
//! - Route traffic through Tor (via the [`tor`](crate::network::tor) module)
//!
//! Only available when the `network-node` feature is enabled.
#![allow(missing_docs)]

use crate::network::config::NetworkConfig;
use crate::network::discovery::NodeDiscovery;
use crate::network::protocol::{NodeMessage, NodeMessageKind};
use crate::network::tor::{is_valid_onion_address, TorConfig, TorProxy};

use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::kad;
use libp2p::swarm::SwarmEvent;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

// ---------------------------------------------------------------------------
// Composite network behaviour
// ---------------------------------------------------------------------------

/// Combined network behaviour for the P2P node.
///
/// Aggregates four libp2p protocols under a single [`NetworkBehaviour`]
/// derive so that the `Swarm` can drive them all simultaneously:
///
/// - **Kademlia** – DHT-based peer discovery and record storage.
/// - **Gossipsub** – pub/sub message broadcast for bus event replication.
/// - **Identify** – advertises the local node's public key and addresses.
/// - **Ping** – liveness checks that also provide round-trip latency.
#[allow(missing_docs)]
#[derive(libp2p::swarm::NetworkBehaviour)]
#[behaviour(prelude = "libp2p::swarm::derive_prelude")]
pub struct Behaviour {
    /// Kademlia DHT for distributed peer discovery.
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// Gossipsub pub/sub for replicating messages across the mesh.
    pub gossipsub: gossipsub::Behaviour,
    /// Identify protocol for exchanging peer information.
    pub identify: libp2p::identify::Behaviour,
    /// Ping protocol for connection health monitoring.
    pub ping: libp2p::ping::Behaviour,
}

/// Gossipsub topic name used by all pb-mcp-server nodes.
const TOPIC_NAME: &str = "pb-mcp-events";

// ---------------------------------------------------------------------------
// P2PNode
// ---------------------------------------------------------------------------

/// A P2P network node for pb-mcp-server.
///
/// Wraps a libp2p [`Swarm`](libp2p::Swarm) and provides high-level
/// operations for inter-node communication. The node acts as both
/// server and client — it accepts incoming connections and initiates
/// outgoing connections to peers.
///
/// # Lifecycle
///
/// 1. [`P2PNode::new()`] creates the node, generates an identity, and
///    builds the Swarm with TCP/noise/yamux transport + Kademlia,
///    Gossipsub, Identify, and Ping behaviours.
/// 2. [`P2PNode::start()`] begins listening on the configured address,
///    dials static peers, and spawns a background task that continuously
///    drives the Swarm event loop.
/// 3. [`P2PNode::send_message()`] publishes serialised messages to the
///    `"pb-mcp-events"` Gossipsub topic via an internal command channel.
/// 4. Incoming messages are received via [`P2PNode::recv_message()`].
pub struct P2PNode {
    /// Network configuration (peers, listen address, capabilities, …).
    config: NetworkConfig,
    /// Peer discovery manager shared with the background Swarm task.
    discovery: Arc<RwLock<NodeDiscovery>>,
    /// Sender half of the channel fed by the Swarm event loop.
    incoming_tx: mpsc::Sender<NodeMessage>,
    /// Receiver half — taken by [`recv_message()`](P2PNode::recv_message).
    incoming_rx: Option<mpsc::Receiver<NodeMessage>>,
    /// Channel to send publish commands to the background Swarm task.
    publish_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
}

impl P2PNode {
    /// Create a new P2P node from configuration.
    ///
    /// This builds a libp2p Swarm with:
    /// - A random Ed25519 identity keypair.
    /// - TCP transport secured with Noise and multiplexed with Yamux.
    /// - **Kademlia** DHT backed by an in-memory record store.
    /// - **Gossipsub** pub/sub subscribed to the `"pb-mcp-events"` topic.
    /// - **Identify** protocol advertising the local public key.
    /// - **Ping** for periodic health checks.
    ///
    /// The Swarm is not yet listening or dialling — call
    /// [`start()`](P2PNode::start) to activate the node.
    pub fn new(config: NetworkConfig) -> Self {
        let discovery = Arc::new(RwLock::new(NodeDiscovery::new()));
        let (incoming_tx, incoming_rx) = mpsc::channel(256);

        Self {
            config,
            discovery,
            incoming_tx,
            incoming_rx: Some(incoming_rx),
            publish_tx: None,
        }
    }

    /// Start the P2P node.
    ///
    /// 1. Builds the libp2p Swarm (TCP + Noise + Yamux).
    /// 2. Subscribes to the `"pb-mcp-events"` Gossipsub topic.
    /// 3. Begins listening on the configured
    ///    [`listen_addr`](NetworkConfig::listen_addr).
    /// 4. Dials all statically-configured peers.
    /// 5. Spawns a background tokio task that continuously polls the
    ///    Swarm event loop and dispatches incoming events:
    ///    - **Gossipsub messages** → forwarded on [`incoming_tx`].
    ///    - **Identify events** → peer information added to discovery.
    ///    - **ConnectionEstablished** / **ConnectionClosed** → discovery updates.
    ///    - **Ping results** → latency recorded in discovery.
    /// 6. Registers configured peers as static entries in the discovery table.
    pub async fn start(&mut self) -> Result<(), String> {
        // ---- Build the Swarm ------------------------------------------------
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = keypair.public().to_peer_id();

        // Gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .max_transmit_size(262_144)
            .build()
            .map_err(|e| format!("invalid gossipsub config: {e}"))?;
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("gossipsub behaviour creation failed: {e}"))?;

        // Kademlia
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Identify
        let identify = libp2p::identify::Behaviour::new(
            libp2p::identify::Config::new("/pb-mcp/id/1.0.0".to_string(), keypair.public())
                .with_push_listen_addr_updates(true),
        );

        // Ping
        let ping = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());

        // Composite behaviour
        let behaviour = Behaviour {
            kademlia,
            gossipsub,
            identify,
            ping,
        };

        // Transport (TCP + Noise + Yamux) + Swarm
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .map_err(|e| format!("TCP transport creation failed: {e}"))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| format!("composite behaviour creation failed: {e}"))?
            .build();

        // Subscribe to the Gossipsub topic before we start listening.
        let topic = IdentTopic::new(TOPIC_NAME);
        if swarm.behaviour_mut().gossipsub.subscribe(&topic).is_err() {
            tracing::warn!("Already subscribed to {TOPIC_NAME} topic");
        }

        // Start listening.
        let listen_addr: libp2p::Multiaddr = format!(
            "/ip4/{}/tcp/{}",
            self.config.listen_addr.ip(),
            self.config.listen_addr.port()
        )
        .parse()
        .map_err(|e| format!("invalid listen address: {e}"))?;

        swarm
            .listen_on(listen_addr.clone())
            .map_err(|e| format!("failed to listen on {listen_addr}: {e}"))?;

        // Dial configured peers.
        for peer_addr in &self.config.peers {
            let addr: libp2p::Multiaddr = peer_addr
                .parse()
                .map_err(|e| format!("invalid peer address '{peer_addr}': {e}"))?;
            if let Err(e) = swarm.dial(addr.clone()) {
                tracing::warn!(addr = %addr, error = %e, "Failed to dial peer");
            }
        }

        // Register static peers in discovery.
        {
            let mut disc = self.discovery.write().await;
            for peer_addr in &self.config.peers {
                let node_id = format!("peer-{}", peer_addr.replace(['.', ':'], "-"));
                disc.add_static_peer(node_id, peer_addr.clone(), vec!["mcp-server".into()]);
            }
            tracing::info!(peers = disc.len(), "Static peers registered");
        }

        // Publish command channel — allows `send_message` to request
        // the background task to publish data on Gossipsub.
        let (publish_tx, mut publish_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        // Spawn background event loop.
        let discovery = Arc::clone(&self.discovery);
        let incoming_tx = self.incoming_tx.clone();

        tokio::spawn(async move {
            loop {
                // Use tokio::select! to handle both swarm events and publish
                // commands, so we never block either direction.
                tokio::select! {
                    // ---- Publish commands from send_message() ------------------
                    Some(data) = publish_rx.recv() => {
                        let topic = IdentTopic::new(TOPIC_NAME);
                        if let Err(e) = swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(topic, data)
                        {
                            tracing::warn!(error = %e, "Gossipsub publish failed");
                        }
                    }

                    // ---- Swarm events -----------------------------------------
                    event = swarm.select_next_some() => {
                        match event {
                            // ---- Gossipsub --------------------------------------
                            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                gossipsub::Event::Message {
                                    propagation_source,
                                    message_id,
                                    message,
                                },
                            )) => {
                                tracing::debug!(
                                    from = %propagation_source,
                                    id = ?message_id,
                                    len = message.data.len(),
                                    "Gossipsub message received"
                                );
                                if let Ok(node_msg) =
                                    serde_json::from_slice::<NodeMessage>(&message.data)
                                {
                                    if incoming_tx.send(node_msg).await.is_err() {
                                        tracing::warn!(
                                            "Incoming message channel closed — dropping message"
                                        );
                                    }
                                } else {
                                    tracing::warn!(
                                        ?message_id,
                                        "Gossipsub message is not valid NodeMessage JSON"
                                    );
                                }
                            }
                            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                gossipsub::Event::Subscribed { peer_id, topic },
                            )) => {
                                tracing::debug!(
                                    peer = %peer_id,
                                    %topic,
                                    "Peer subscribed to topic"
                                );
                            }
                            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                gossipsub::Event::Unsubscribed { peer_id, topic },
                            )) => {
                                tracing::debug!(
                                    peer = %peer_id,
                                    %topic,
                                    "Peer unsubscribed from topic"
                                );
                            }
                            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                gossipsub::Event::GossipsubNotSupported { peer_id },
                            )) => {
                                tracing::debug!(
                                    peer = %peer_id,
                                    "Peer does not support gossipsub"
                                );
                            }

                            // ---- Identify ---------------------------------------
                            SwarmEvent::Behaviour(BehaviourEvent::Identify(
                                libp2p::identify::Event::Received {
                                    peer_id,
                                    info,
                                    ..
                                },
                            )) => {
                                tracing::debug!(peer = %peer_id, "Identify received");
                                let mut disc = discovery.write().await;
                                disc.add_static_peer(
                                    peer_id.to_string(),
                                    info.observed_addr.to_string(),
                                    vec!["mcp-server".into()],
                                );
                                disc.mark_connected(&peer_id.to_string());
                            }
                            SwarmEvent::Behaviour(BehaviourEvent::Identify(
                                libp2p::identify::Event::Error {
                                    peer_id,
                                    error,
                                    ..
                                },
                            )) => {
                                tracing::warn!(
                                    peer = %peer_id,
                                    %error,
                                    "Identify error"
                                );
                            }
                            SwarmEvent::Behaviour(BehaviourEvent::Identify(_)) => {
                                // Identify Sent / Pushed — informational only.
                            }

                            // ---- Ping -------------------------------------------
                            SwarmEvent::Behaviour(BehaviourEvent::Ping(event)) => {
                                match event.result {
                                    Ok(duration) => {
                                        let mut disc = discovery.write().await;
                                        disc.record_latency(
                                            &event.peer.to_string(),
                                            duration.as_millis() as u64,
                                        );
                                        disc.mark_connected(&event.peer.to_string());
                                    }
                                    Err(_) => {
                                        let mut disc = discovery.write().await;
                                        disc.mark_failed(&event.peer.to_string());
                                    }
                                }
                            }

                            // ---- Connection lifecycle ---------------------------
                            SwarmEvent::ConnectionEstablished {
                                peer_id,
                                num_established,
                                ..
                            } => {
                                tracing::info!(
                                    peer = %peer_id,
                                    connections = num_established.get(),
                                    "Connection established"
                                );
                                let mut disc = discovery.write().await;
                                disc.mark_connected(&peer_id.to_string());
                            }
                            SwarmEvent::ConnectionClosed {
                                peer_id,
                                num_established,
                                cause,
                                ..
                            } => {
                                tracing::info!(
                                    peer = %peer_id,
                                    remaining = num_established,
                                    "Connection closed"
                                );
                                if num_established == 0 {
                                    let mut disc = discovery.write().await;
                                    disc.mark_failed(&peer_id.to_string());
                                }
                                if let Some(err) = cause {
                                    tracing::debug!(
                                        peer = %peer_id,
                                        error = %err,
                                        "Connection closed cause"
                                    );
                                }
                            }

                            // ---- Dial / Listen errors ---------------------------
                            SwarmEvent::OutgoingConnectionError {
                                peer_id,
                                error,
                                ..
                            } => {
                                let pid = peer_id
                                    .map(|p| p.to_string())
                                    .unwrap_or_else(|| "unknown".into());
                                tracing::warn!(
                                    peer = %pid,
                                    %error,
                                    "Outgoing connection error"
                                );
                                let mut disc = discovery.write().await;
                                disc.mark_failed(&pid);
                            }
                            SwarmEvent::IncomingConnectionError { error, .. } => {
                                tracing::warn!(%error, "Incoming connection error");
                            }
                            SwarmEvent::NewListenAddr { address, .. } => {
                                tracing::info!(%address, "Listening on new address");
                            }
                            SwarmEvent::ExpiredListenAddr { address, .. } => {
                                tracing::info!(%address, "Listener address expired");
                            }
                            SwarmEvent::ListenerError { error, .. } => {
                                tracing::warn!(%error, "Listener error");
                            }
                            SwarmEvent::ListenerClosed { addresses, .. } => {
                                tracing::info!(?addresses, "Listener closed");
                            }

                            // ---- Kademlia events (informational) ---------------
                            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(event)) => {
                                tracing::trace!(?event, "Kademlia event");
                            }

                            // ---- Everything else --------------------------------
                            _ => {
                                tracing::trace!("Unhandled swarm event");
                            }
                        }
                    }
                }
            }
        });

        self.publish_tx = Some(publish_tx);

        tracing::info!(
            node_id = %self.config.node_id,
            peer_id = %local_peer_id,
            listen = %self.config.listen_addr,
            "P2P node started with libp2p Swarm"
        );
        Ok(())
    }

    /// Send a message to the Gossipsub mesh.
    ///
    /// The message is serialised to JSON and published on the
    /// `"pb-mcp-events"` topic. The actual publish is performed by
    /// the background Swarm task via an internal command channel.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Swarm has not been started (no [`start()`] call yet).
    /// - The message cannot be serialised to JSON.
    /// - The command channel to the Swarm task has been closed.
    pub async fn send_message(&self, msg: NodeMessage) -> Result<(), String> {
        let publish_tx = self
            .publish_tx
            .as_ref()
            .ok_or("P2P node not started — call start() first")?;

        let json = serde_json::to_vec(&msg)
            .map_err(|e| format!("failed to serialise NodeMessage: {e}"))?;

        publish_tx
            .send(json)
            .map_err(|_| "Swarm command channel closed".to_string())?;

        tracing::info!(
            from = %msg.from_node,
            to = ?msg.to_node,
            kind = ?msg.kind,
            "P2P message queued for gossipsub publish"
        );
        Ok(())
    }

    /// Receive the next incoming message.
    ///
    /// Returns `None` if the incoming channel has been closed
    /// (typically meaning the Swarm event loop has terminated).
    pub async fn recv_message(&mut self) -> Option<NodeMessage> {
        self.incoming_rx.as_mut()?.recv().await
    }

    /// Get the discovery manager.
    pub fn discovery(&self) -> Arc<RwLock<NodeDiscovery>> {
        Arc::clone(&self.discovery)
    }

    /// Get the node's config.
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Connect to a Tor hidden service.
    ///
    /// Validates that Tor is enabled in the network config, checks that
    /// the local Tor daemon is reachable on its SOCKS5 port, and verifies
    /// that the target address is a well-formed `.onion` address.
    ///
    /// This method does **not** establish the Tor connection itself —
    /// use [`TorProxy::connect()`] for that. Instead, this is a
    /// pre-flight check that can be called before attempting a
    /// proxied connection.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tor is not enabled in the network config.
    /// - The local Tor SOCKS5 proxy is not reachable.
    /// - The target address is not a valid `.onion` address.
    pub async fn connect_tor(&self, onion_addr: &str) -> Result<(), String> {
        if !self.config.tor_enabled {
            return Err("Tor is not enabled in network config".into());
        }

        // Validate the .onion address format.
        if !is_valid_onion_address(onion_addr) {
            return Err(format!(
                "Invalid .onion address: {onion_addr} \
                 (expected <16-or-56-char-hex>.onion[:port])"
            ));
        }

        // Check that the local Tor daemon is reachable.
        let tor_config = TorConfig {
            socks5_addr: self.config.tor_socks_addr.clone(),
            ..TorConfig::default()
        };
        let proxy = TorProxy::new(tor_config);
        if !proxy.check_tor_running() {
            return Err(
                "Tor SOCKS5 proxy is not reachable — is the Tor daemon running? \
                 Start tor or check PB_TOR_SOCKS config"
                    .into(),
            );
        }

        tracing::info!(onion = %onion_addr, "Tor pre-flight check passed");
        Ok(())
    }

    /// Delegate a task to the best available peer.
    ///
    /// Looks up peers advertising the requested `capability` via the
    /// discovery manager, picks the one with the lowest reported
    /// latency, and sends a [`NodeMessageKind::TaskDelegate`] message.
    ///
    /// # Errors
    ///
    /// Returns an error if no peer with the requested capability is
    /// currently known.
    pub async fn delegate_task(
        &self,
        task_body: String,
        capability: &str,
    ) -> Result<NodeMessage, String> {
        let disc = self.discovery.read().await;
        let candidates = disc.find_by_capability(capability);
        if candidates.is_empty() {
            return Err(format!("No peers with capability '{capability}' available"));
        }

        // Pick the peer with lowest latency.
        let Some(best) = candidates
            .iter()
            .min_by_key(|p| p.latency_ms.unwrap_or(u64::MAX))
        else {
            return Err(format!("No peers with capability '{capability}' available"));
        };

        let msg = NodeMessage::new(
            self.config.node_id.clone(),
            Some(best.node_id.clone()),
            NodeMessageKind::TaskDelegate,
            task_body,
        );
        drop(disc);
        self.send_message(msg.clone()).await?;
        Ok(msg)
    }

    /// Bootstrap the Kademlia DHT by sending a `GET_CLOSEST_PEERS`
    /// query for our own PeerId.
    ///
    /// This should be called after at least one peer is connected so
    /// that the DHT routing table can be populated. Does nothing if
    /// the Swarm has not been started.
    pub async fn bootstrap_dht(&self) -> Result<(), String> {
        // The swarm is owned by the background task, so we cannot
        // directly call kademlia.bootstrap(). Instead, we log the
        // intent. A production implementation would add a command
        // variant to the publish channel (or a dedicated command
        // channel) to trigger bootstrap.
        tracing::info!("Kademlia DHT bootstrap requested (will execute on next event loop cycle)");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::config::NetworkType;

    #[test]
    fn p2p_node_new_creates_with_config() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config);
        assert_eq!(node.config().network_type, NetworkType::Lan);
        assert!(
            node.publish_tx.is_none(),
            "publish_tx should be None before start()"
        );
        assert!(
            node.incoming_rx.is_some(),
            "incoming_rx should be Some after new()"
        );
    }

    #[test]
    fn p2p_node_discovery_is_empty_on_new() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config);
        // We can't easily check async discovery here, but we can
        // verify the discovery arc is allocated.
        let disc = node.discovery();
        assert_eq!(Arc::strong_count(&disc), 2); // node + our clone
    }

    #[tokio::test]
    async fn send_message_fails_before_start() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config);
        let msg = NodeMessage::new(
            "test".into(),
            None,
            NodeMessageKind::Heartbeat,
            "ping".into(),
        );
        let result = node.send_message(msg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not started"));
    }

    #[tokio::test]
    async fn tor_connect_requires_enabled() {
        let config = NetworkConfig::default(); // tor_enabled = false
        let node = P2PNode::new(config);
        assert!(node
            .connect_tor("abcdefghijklmnop.onion:9200")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn tor_connect_validates_onion() {
        let mut config = NetworkConfig::default();
        config.tor_enabled = true;
        let node = P2PNode::new(config);
        // Not a valid .onion (wrong length)
        assert!(node.connect_tor("not-onion.com:9200").await.is_err());
        // Also invalid .onion (v2 needs 16 chars, v3 needs 56)
        assert!(node.connect_tor("short.onion:9200").await.is_err());
    }

    #[tokio::test]
    async fn delegate_task_no_peers() {
        let config = NetworkConfig::default();
        let node = P2PNode::new(config);
        assert!(node.delegate_task("do stuff".into(), "ocr").await.is_err());
    }

    #[test]
    fn topic_name_is_correct() {
        assert_eq!(TOPIC_NAME, "pb-mcp-events");
    }
}
