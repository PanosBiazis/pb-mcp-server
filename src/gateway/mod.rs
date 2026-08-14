//! HTTP Gateway module for pb-mcp-server
//!
//! Provides a safe, authenticated HTTP gateway for remote/LAN access
//! to the MCP server's tools. Used by the `pb-gateway` binary and
//! the Netlify control plane.
//!
//! # Security Model
//!
//! - Bearer token authentication (optional)
//! - HMAC-SHA256 request signing (optional)
//! - Anti-replay nonce protection
//! - Local network trust bypass (Tailscale/LAN)
//! - Rate limiting per endpoint
//!
//! # Endpoints
//!
//! | Endpoint        | Method | Access     | Description               |
//! |-----------------|--------|------------|---------------------------|
//! | `/health`       | GET    | Public     | Health check              |
//! | `/status`       | GET    | Auth       | Server status and profile |
//! | `/read-file`    | POST   | Auth       | Read file from project    |
//! | `/search-code`  | POST   | Auth       | Search code in project    |
//! | `/list-files`   | POST   | Auth       | List project files        |
//! | `/exec`         | POST   | Local-only | Execute terminal command  |
//! | `/agent-send`   | POST   | Auth       | Send agent bus message    |
//! | `/agent-recent` | GET    | Auth       | Recent bus messages       |
//! | `/agent-thread` | POST   | Auth       | Get bus thread            |
//! | `/review-packet`| POST   | Auth       | Generate review packet    |
//! | `/project-info` | GET    | Auth       | Project metadata          |
//! | `/security-status` | GET | Auth       | Security audit status     |
//! | `/metrics`      | GET    | Auth       | Performance metrics       |

pub mod auth;
pub mod routes;
pub mod state;

// Re-export key types
pub use auth::{
    assert_bind_safe, is_local_network_ip, is_loopback_bind, require_token, verify_signature,
};
pub use routes::build_router;
pub use state::AppState;

fn validate_startup(bind: &str, state: &AppState) -> Result<(), String> {
    assert_bind_safe(bind, state.trust_local_network)?;
    if !is_loopback_bind(bind) && state.token.is_none() && state.signing_secret.is_none() {
        return Err(
            "non-loopback gateway requires PB_GATEWAY_TOKEN or PB_GATEWAY_SIGNING_SECRET"
                .to_string(),
        );
    }
    if state.exec_enabled && !(state.trust_local_network && is_loopback_bind(bind)) {
        return Err(
            "PB_GATEWAY_EXEC_ENABLE requires trusted local mode on a loopback bind".to_string(),
        );
    }
    Ok(())
}

/// Start the canonical gateway from environment configuration.
///
/// Both workspace gateway binaries call this function. Startup fails closed
/// when a non-loopback listener has no authentication or when execution is
/// enabled without the explicit trusted-loopback development profile.
pub async fn serve_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let bind = std::env::var("PB_GATEWAY_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let state = std::sync::Arc::new(AppState::from_env());

    validate_startup(&bind, &state)?;

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    let app = build_router(state);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn state(token: Option<&str>, trust: bool, exec: bool) -> AppState {
        let mut state = AppState::new(
            token.map(str::to_string),
            None,
            120_000,
            trust,
            PathBuf::from("."),
        );
        state.exec_enabled = exec;
        state
    }

    #[test]
    fn startup_refuses_unauthenticated_non_loopback_listener() {
        assert!(validate_startup("0.0.0.0:8080", &state(None, false, false)).is_err());
        assert!(validate_startup("0.0.0.0:8080", &state(Some("token"), false, false)).is_ok());
    }

    #[test]
    fn startup_exec_requires_trusted_loopback_mode() {
        assert!(validate_startup("127.0.0.1:8080", &state(None, false, true)).is_err());
        assert!(validate_startup("127.0.0.1:8080", &state(None, true, true)).is_ok());
        assert!(validate_startup("0.0.0.0:8080", &state(Some("token"), true, true)).is_err());
    }
}
