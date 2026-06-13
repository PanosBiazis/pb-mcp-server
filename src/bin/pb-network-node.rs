//! pb-network-node — Network node binary for pb-mcp-server.
//!
//! Runs a standalone network node that can connect to other
//! pb-mcp-server instances and relay agent messages, tasks,
//! and bus events across the network.

#![cfg(feature = "network-node")]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::field_reassign_with_default,
        clippy::io_other_error,
        clippy::needless_borrows_for_generic_args,
        clippy::panic,
        clippy::unwrap_used
    )
)]

use pb_mcp_server::network::{NetworkConfig, P2PNode};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("pb_network_node=info,pb_mcp_server=info")
        .init();

    let config = NetworkConfig::from_env();

    tracing::info!(node_id = %config.node_id, "Starting pb-network-node");

    let mut node = P2PNode::new(config);

    if let Err(e) = node.start().await {
        tracing::error!("Failed to start P2P node: {e}");
        std::process::exit(1);
    }

    tracing::info!("pb-network-node running. Press Ctrl-C to stop.");

    tokio::signal::ctrl_c().await.ok();

    tracing::info!("Shutting down.");
}
