//! Network module — scalable pb-mcp-server node networking.
//!
//! Enables pb-mcp-server instances to act as both servers AND clients,
//! forming a network where AI agents can communicate, share tasks,
//! and pool knowledge across nodes.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐  P2P/gossip  ┌─────────────┐
//! │ pb-mcp-node │ ◄───────────────► │ pb-mcp-node │
//! │ (server +  │               │ (server +  │
//! │  client)   │               │  client)   │
//! └─────────────┘               └─────────────┘
//!       │                              │
//!   Agent Bus                     Agent Bus
//!   (local)                       (local)
//! ```
//!
//! # Network Types
//!
//! | Type     | Purpose                                |
//! |----------|----------------------------------------|
//! | LAN      | Direct peer-to-peer on local network   |
//! | Tor      | Anonymous routing via Tor network      |
//! | WebSocket| Real-time agent-to-agent streaming     |
//! | Custom   | User-defined network transports        |

pub mod config;
pub mod discovery;
#[cfg(feature = "network-node")]
pub mod p2p;
pub mod protocol;
pub mod tor;

pub use config::NetworkConfig;
pub use discovery::NodeDiscovery;
#[cfg(feature = "network-node")]
pub use p2p::P2PNode;
pub use protocol::{NodeMessage, NodeMessageKind};
pub use tor::{is_valid_onion_address, TorConfig, TorConnection, TorProxy};
