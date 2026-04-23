//! Gateway authentication and request-signing module.
//!
//! Provides three layers of security for the HTTP gateway:
//!
//! 1. **Bearer token** validation via [`require_token`] with constant-time
//!    comparison to mitigate timing-oracle attacks.
//! 2. **HMAC-SHA256 request signing** via [`verify_signature`] with
//!    timestamp-skew and anti-replay nonce checks.
//! 3. **Local-network trust bypass** via [`is_local_network_ip`] which
//!    recognises loopback, RFC-1918 private ranges, and Tailscale CGNAT.
//!
//! The gateway's `authorize_request` flow (implemented in
//! [`routes`](super::routes)) checks local-network trust first, then
//! token, then signature.

use axum::http::{HeaderMap, Method, StatusCode, Uri};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;

use super::state::AppState;

/// HMAC type alias for SHA-256.
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Bearer token validation
// ---------------------------------------------------------------------------

/// Validate the `Authorization: Bearer <token>` header against the
/// configured token.
///
/// Returns `Ok(())` when:
/// - No token is configured (`state.token` is `None`), **or**
/// - The supplied credential matches the configured token using
///   constant-time comparison.
///
/// Returns `Err((StatusCode, message))` when the header is missing or
/// the token does not match.
pub fn require_token(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, &'static str)> {
    let Some(expected) = state.token.as_deref() else {
        return Ok(());
    };

    let Some(got) = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
    else {
        return Err((StatusCode::UNAUTHORIZED, "missing authorization header"));
    };

    // Accept either "Bearer <token>" or bare "<token>" for curl convenience.
    let got = got
        .strip_prefix("Bearer ")
        .or_else(|| got.strip_prefix("bearer "))
        .unwrap_or(got);

    // Constant-time compare to reduce token oracle leakage.
    if bool::from(got.as_bytes().ct_eq(expected.as_bytes())) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "invalid token"))
    }
}

// ---------------------------------------------------------------------------
// HMAC-SHA256 signature verification
// ---------------------------------------------------------------------------

/// Return the current time as milliseconds since UNIX epoch.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Encode `bytes` as lowercase hex.
fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Build the canonical signing input from request components.
///
/// Format (each field on its own line):
/// ```text
/// <timestamp_ms>
/// <nonce>
/// <METHOD>
/// <path>
/// <query>
/// <body_sha256_hex>
/// ```
fn canonical_sign_input(
    ts_ms: &str,
    nonce: &str,
    method: &Method,
    uri: &Uri,
    body_sha256_hex: &str,
) -> String {
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    format!(
        "{ts_ms}\n{nonce}\n{}\n{path}\n{query}\n{body_sha256_hex}",
        method.as_str().to_ascii_uppercase()
    )
}

/// Verify the HMAC-SHA256 signature on an incoming request.
///
/// The client must send three headers:
///
/// | Header | Description |
/// |---|---|
/// | `x-pb-ts` | Request timestamp in milliseconds since epoch |
/// | `x-pb-nonce` | Unique nonce for replay protection |
/// | `x-pb-signature` | Hex-encoded HMAC-SHA256 of the canonical input |
///
/// Checks performed (in order):
///
/// 1. If no `signing_secret` is configured, return `Ok(())` immediately.
/// 2. All three headers must be present and non-empty.
/// 3. `x-pb-ts` must parse as `u64` and be within
///    `state.signing_max_skew_ms` of the server clock.
/// 4. The nonce must not have been seen before (anti-replay).
/// 5. The HMAC-SHA256 of the canonical input must match the supplied
///    signature (constant-time compare).
///
/// Old nonces are pruned from `state.nonce_seen` on every call.
pub fn verify_signature(
    state: &AppState,
    headers: &HeaderMap,
    method: &Method,
    uri: &Uri,
    body: &[u8],
) -> Result<(), (StatusCode, &'static str)> {
    let Some(secret) = state.signing_secret.as_deref() else {
        return Ok(());
    };

    let ts = headers
        .get("x-pb-ts")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-ts"))?;

    let nonce = headers
        .get("x-pb-nonce")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-nonce"))?;

    let sig = headers
        .get("x-pb-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or((StatusCode::UNAUTHORIZED, "missing x-pb-signature"))?;

    // --- Timestamp skew check ---
    let ts_u = ts
        .parse::<u64>()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid x-pb-ts"))?;

    let now = now_ms();
    let delta = now.abs_diff(ts_u);
    if delta > state.signing_max_skew_ms {
        return Err((
            StatusCode::UNAUTHORIZED,
            "signature timestamp out of window",
        ));
    }

    // --- Anti-replay nonce check ---
    {
        let mut map = state
            .nonce_seen
            .lock()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "nonce cache lock failed"))?;

        let min_ts = now.saturating_sub(state.signing_max_skew_ms);
        map.retain(|_, seen_ts| *seen_ts >= min_ts);

        if map.contains_key(nonce) {
            return Err((StatusCode::UNAUTHORIZED, "replayed nonce"));
        }
        map.insert(nonce.to_string(), now);
    }

    // --- HMAC-SHA256 verification ---
    let body_hash = Sha256::digest(body);
    let body_hex = hex_lower(&body_hash);
    let input = canonical_sign_input(ts, nonce, method, uri, &body_hex);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "invalid signing key"))?;
    mac.update(input.as_bytes());
    let expected = hex_lower(&mac.finalize().into_bytes());

    if !bool::from(sig.as_bytes().ct_eq(expected.as_bytes())) {
        return Err((StatusCode::FORBIDDEN, "bad request signature"));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Local-network trust
// ---------------------------------------------------------------------------

/// Check whether an IP address belongs to a local/private/Tailscale network.
///
/// Returns `true` for:
///
/// - IPv4 loopback (`127.0.0.0/8`)
/// - IPv4 RFC-1918 private ranges (`10.0.0.0/8`, `172.16.0.0/12`,
///   `192.168.0.0/16`)
/// - Tailscale CGNAT range (`100.64.0.0/10`)
/// - IPv6 loopback (`::1`)
/// - IPv6 unique-local (`fc00::/7`)
pub fn is_local_network_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            if v4.is_loopback() || v4.is_private() {
                return true;
            }
            let o = v4.octets();
            // Tailscale CGNAT (100.64.0.0/10)
            o[0] == 100 && (64..=127).contains(&o[1])
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
    }
}

// ---------------------------------------------------------------------------
// Composite authorisation helper
// ---------------------------------------------------------------------------

/// Full authorisation check combining local-network trust, token, and
/// signature verification.
///
/// Short-circuits to `Ok(())` when `trust_local_network` is enabled and
/// the peer IP is local. Otherwise, validates the bearer token first,
/// then the HMAC-SHA256 signature.
pub fn authorize_request(
    state: &AppState,
    peer_ip: IpAddr,
    headers: &HeaderMap,
    method: &Method,
    uri: &Uri,
    body: &[u8],
) -> Result<(), (StatusCode, &'static str)> {
    if state.trust_local_network && is_local_network_ip(peer_ip) {
        return Ok(());
    }
    require_token(state, headers)?;
    verify_signature(state, headers, method, uri, body)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::path::PathBuf;

    fn test_state() -> AppState {
        AppState::new(
            Some("test-token".to_string()),
            Some("test-secret".to_string()),
            120_000,
            false,
            std::path::PathBuf::from("."),
        )
    }

    #[test]
    fn require_token_none_when_no_token_configured() {
        let state = AppState::new(None, None, 120_000, false, PathBuf::from("."));
        let headers = HeaderMap::new();
        assert!(require_token(&state, &headers).is_ok());
    }

    #[test]
    fn require_token_rejects_missing_header() {
        let state = test_state();
        let headers = HeaderMap::new();
        assert!(require_token(&state, &headers).is_err());
    }

    #[test]
    fn require_token_accepts_valid_bearer() {
        let state = test_state();
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-token".parse().unwrap());
        assert!(require_token(&state, &headers).is_ok());
    }

    #[test]
    fn require_token_rejects_bad_token() {
        let state = test_state();
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer wrong".parse().unwrap());
        assert!(require_token(&state, &headers).is_err());
    }

    #[test]
    fn is_local_loopback_v4() {
        assert!(is_local_network_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn is_local_private_v4() {
        assert!(is_local_network_ip(IpAddr::V4(Ipv4Addr::new(
            192, 168, 1, 1
        ))));
        assert!(is_local_network_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_local_network_ip(IpAddr::V4(Ipv4Addr::new(
            172, 16, 0, 1
        ))));
    }

    #[test]
    fn is_local_tailscale_cgnat() {
        assert!(is_local_network_ip(IpAddr::V4(Ipv4Addr::new(
            100, 100, 0, 1
        ))));
        assert!(!is_local_network_ip(IpAddr::V4(Ipv4Addr::new(
            100, 63, 0, 1
        ))));
        assert!(!is_local_network_ip(IpAddr::V4(Ipv4Addr::new(
            100, 128, 0, 1
        ))));
    }

    #[test]
    fn is_local_loopback_v6() {
        assert!(is_local_network_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn is_not_local_public_v4() {
        assert!(!is_local_network_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn authorize_bypasses_local_network() {
        let mut state = test_state();
        state.trust_local_network = true;
        let headers = HeaderMap::new();
        let uri: Uri = "/health".parse().unwrap();
        let result = authorize_request(
            &state,
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            &headers,
            &Method::GET,
            &uri,
            b"",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn authorize_rejects_remote_without_token() {
        let state = test_state();
        let headers = HeaderMap::new();
        let uri: Uri = "/status".parse().unwrap();
        let result = authorize_request(
            &state,
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            &headers,
            &Method::GET,
            &uri,
            b"",
        );
        assert!(result.is_err());
    }
}
