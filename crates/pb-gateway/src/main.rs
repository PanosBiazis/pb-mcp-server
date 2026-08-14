//! Workspace wrapper for the canonical pb-mcp-server gateway.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pb_mcp_server::gateway::serve_from_env().await
}
