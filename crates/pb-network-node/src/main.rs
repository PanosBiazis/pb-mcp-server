//! pb-network-node — Network node binary for pb-mcp-server.
//!
//! Runs a standalone network node that can connect to other
//! pb-mcp-server instances and relay agent messages, tasks,
//! and bus events across the network.

// Feature gate removed: this is now a standalone crate that always
// depends on pb-mcp-server with the "network-node" feature enabled.

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use pb_mcp_server::network::NetworkConfig;

    // -- helpers -------------------------------------------------------------

    /// Remove every env var that `NetworkConfig::from_env` reads so that each
    /// test starts from a known-clean slate.  Because Rust runs tests in
    /// parallel, callers that *set* vars should also call this helper
    /// afterwards to avoid leaking state into other tests.
    fn clear_network_env_vars() {
        for var in &[
            "PB_NODE_ID",
            "PB_NODE_PEERS",
            "PB_NODE_LISTEN",
            "PB_TOR_ENABLED",
            "PB_TOR_SOCKS",
            "PB_NODE_SECRET",
            "PB_NODE_MAX_PEERS",
        ] {
            std::env::remove_var(var);
        }
    }

    // -- Test 1: from_env with NO env vars set --------------------------------

    /// `NetworkConfig::from_env` must return a valid config (non-empty
    /// `node_id`, sane defaults) when none of the `PB_NODE_*` variables are
    /// present in the environment.
    #[test]
    fn from_env_without_vars_returns_valid_config() {
        clear_network_env_vars();

        let cfg = NetworkConfig::from_env();

        assert!(!cfg.node_id.is_empty(), "node_id must be non-empty");
        assert!(
            cfg.peers.is_empty(),
            "peers should be empty when PB_NODE_PEERS is unset"
        );
        assert!(
            !cfg.tor_enabled,
            "Tor should be disabled when PB_TOR_ENABLED is unset"
        );
        assert_eq!(
            cfg.max_peers, 50,
            "max_peers should fall back to the default of 50"
        );
        assert!(
            cfg.gossip_enabled,
            "gossip should be enabled in the default config"
        );
    }

    // -- Test 2: NetworkConfig default field values ---------------------------

    /// `NetworkConfig::default()` must produce a config with well-formed,
    /// operationally reasonable values that the binary can actually use.
    #[test]
    fn default_config_fields_are_reasonable() {
        let cfg = NetworkConfig::default();

        // node_id is derived from a fixed zero-byte seed.
        assert!(!cfg.node_id.is_empty(), "node_id must be non-empty");
        assert!(
            cfg.node_id.starts_with("node-"),
            "default node_id should start with 'node-'"
        );

        // Port 9200 is the well-known default for pb-network-node.
        assert_eq!(
            cfg.listen_addr.port(),
            9200,
            "default listen port should be 9200"
        );

        // Max-peers cap should be a positive, usable value.
        assert!(cfg.max_peers > 0, "max_peers must be positive");
        assert_eq!(cfg.max_peers, 50, "default max_peers should be 50");

        // Tor is intentionally off by default (requires explicit opt-in).
        assert!(!cfg.tor_enabled, "Tor should be disabled by default");

        // Gossip is on by default so nodes form a mesh automatically.
        assert!(cfg.gossip_enabled, "gossip should be enabled by default");

        // Tor SOCKS proxy default — standard Tor Browser / tor daemon port.
        assert_eq!(
            cfg.tor_socks_addr, "127.0.0.1:9050",
            "default Tor SOCKS address should be 127.0.0.1:9050"
        );

        // At least the core capability tags are advertised.
        assert!(
            cfg.capabilities.contains(&"mcp-server".to_string()),
            "default capabilities should include 'mcp-server'"
        );
        assert!(
            cfg.capabilities.contains(&"agent-bus".to_string()),
            "default capabilities should include 'agent-bus'"
        );
    }

    // -- Test 3: tracing subscriber smoke test --------------------------------

    /// The env-filter directive string used in `main()` must parse
    /// successfully.  A broken string would cause `main()` to panic at
    /// startup, so we validate it here without touching the global
    /// subscriber (to avoid "already initialised" panics in parallel tests).
    #[test]
    fn tracing_env_filter_string_is_valid() {
        let result =
            tracing_subscriber::EnvFilter::try_new("pb_network_node=info,pb_mcp_server=info");
        assert!(
            result.is_ok(),
            "the env-filter directive used in main() must parse without error: {:?}",
            result.err()
        );
    }
}
