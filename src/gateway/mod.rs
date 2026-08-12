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
pub use auth::{assert_bind_safe, is_local_network_ip, require_token, verify_signature};
pub use routes::build_router;
pub use state::AppState;
