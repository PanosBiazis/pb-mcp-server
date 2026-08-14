//! Canonical HTTP gateway binary.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pb_mcp_server::gateway::serve_from_env().await
}
