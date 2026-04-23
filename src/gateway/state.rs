//! Gateway application state module.
//!
//! Defines the shared [`AppState`] struct that holds configuration
//! for authentication, request signing, and project root confinement.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Shared application state for the HTTP gateway.
///
/// Constructed once at startup and wrapped in `Arc` for cheap cloning
/// across route handlers. Fields mirror the environment-variable
/// configuration consumed by the `pb-gateway` binary.
///
/// # Fields
///
/// - `token` — Optional bearer token for `Authorization: Bearer <token>`.
///   When `None`, token authentication is disabled.
/// - `signing_secret` — Optional HMAC-SHA256 secret for request signing.
///   When `None`, signed-request verification is skipped.
/// - `signing_max_skew_ms` — Maximum clock-skew (in milliseconds) tolerated
///   between the `x-pb-ts` header and the server clock.
/// - `nonce_seen` — In-memory set of recently-seen nonces, used for
///   anti-replay protection. Old entries are pruned on every
///   [`verify_signature`](super::auth::verify_signature) call.
/// - `trust_local_network` — When `true`, requests originating from
///   loopback, private-range, or Tailscale CGNAT IPs bypass auth entirely.
/// - `project_root` — The directory the gateway confines file access to.
pub struct AppState {
    /// Optional bearer token for `Authorization` header validation.
    pub token: Option<String>,
    /// Optional HMAC-SHA256 signing secret for request-signature checks.
    pub signing_secret: Option<String>,
    /// Maximum tolerated clock skew (ms) between client and server.
    pub signing_max_skew_ms: u64,
    /// Recently-seen nonces for anti-replay protection.
    pub nonce_seen: Mutex<HashMap<String, u64>>,
    /// When true, local/private/Tailscale IPs bypass auth.
    pub trust_local_network: bool,
    /// Project root directory that confines file access.
    pub project_root: PathBuf,
}

impl AppState {
    /// Build an `AppState` from the standard environment variables.
    ///
    /// | Variable | Default |
    /// |---|---|
    /// | `PB_GATEWAY_TOKEN` | `None` (token auth disabled) |
    /// | `PB_GATEWAY_SIGNING_SECRET` | `None` (signing disabled) |
    /// | `PB_GATEWAY_SIGNING_MAX_SKEW_MS` | `120_000` |
    /// | `PB_GATEWAY_TRUST_LOCAL_NETWORK` | `false` |
    /// | `PB_PROJECT_ROOT` / `MINT_PROJECT_ROOT` / `PROJECT_ROOT` | current directory |
    pub fn from_env() -> Self {
        let token = std::env::var("PB_GATEWAY_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let signing_secret = std::env::var("PB_GATEWAY_SIGNING_SECRET")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let signing_max_skew_ms: u64 = std::env::var("PB_GATEWAY_SIGNING_MAX_SKEW_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(120_000);

        let trust_local_network = std::env::var("PB_GATEWAY_TRUST_LOCAL_NETWORK")
            .ok()
            .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let project_root = std::env::var("PB_PROJECT_ROOT")
            .or_else(|_| std::env::var("MINT_PROJECT_ROOT"))
            .or_else(|_| std::env::var("PROJECT_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        Self {
            token,
            signing_secret,
            signing_max_skew_ms,
            nonce_seen: Mutex::new(HashMap::new()),
            trust_local_network,
            project_root,
        }
    }

    /// Convenience constructor for tests or programmatic setup.
    pub fn new(
        token: Option<String>,
        signing_secret: Option<String>,
        signing_max_skew_ms: u64,
        trust_local_network: bool,
        project_root: PathBuf,
    ) -> Self {
        Self {
            token,
            signing_secret,
            signing_max_skew_ms,
            nonce_seen: Mutex::new(HashMap::new()),
            trust_local_network,
            project_root,
        }
    }
}
