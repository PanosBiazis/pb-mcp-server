//! Tor SOCKS5 proxy integration for P2P networking.
//!
//! Implements actual SOCKS5 handshake over `tokio::net::TcpStream` to route
//! connections through the Tor network. This allows pb-mcp-server nodes to
//! communicate anonymously via .onion addresses.
//!
//! # SOCKS5 Handshake Flow
//!
//! 1. Connect to the Tor SOCKS5 proxy (default `127.0.0.1:9050`)
//! 2. Send greeting: `[0x05, 0x01, 0x00]` (version 5, 1 method, no auth)
//! 3. Read server choice: `[0x05, 0x00]` (version 5, no auth accepted)
//! 4. Send connect request with domain name type (`0x03`)
//! 5. Read server reply and verify success (`0x00` reply code)

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ---------------------------------------------------------------------------
// SOCKS5 protocol constants
// ---------------------------------------------------------------------------

/// SOCKS5 protocol version.
const SOCKS5_VERSION: u8 = 0x05;

/// No-authentication method.
const SOCKS5_AUTH_NONE: u8 = 0x00;

/// CONNECT command.
const SOCKS5_CMD_CONNECT: u8 = 0x01;

/// RESOLVE command (Tor extension for DNS via SOCKS).
const SOCKS5_CMD_RESOLVE: u8 = 0xF0;

/// Reserved byte.
const SOCKS5_RESERVED: u8 = 0x00;

/// Address type: domain name.
const SOCKS5_ATYP_DOMAIN: u8 = 0x03;

/// Address type: IPv4.
const SOCKS5_ATYP_IPV4: u8 = 0x01;

/// Address type: IPv6.
const SOCKS5_ATYP_IPV6: u8 = 0x04;

/// SOCKS5 reply code: succeeded.
const SOCKS5_REPLY_SUCCESS: u8 = 0x00;

// ---------------------------------------------------------------------------
// TorConfig
// ---------------------------------------------------------------------------

/// Configuration for the Tor SOCKS5 proxy connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConfig {
    /// SOCKS5 proxy address (default `127.0.0.1:9050`).
    pub socks5_addr: String,
    /// Tor DNS port (default 9053).
    pub dns_port: u16,
    /// Whether Tor integration is enabled.
    pub enabled: bool,
    /// Timeout in milliseconds for circuit building.
    pub circuit_build_timeout_ms: u64,
    /// Maximum number of circuit build retries.
    pub max_circuit_retries: u32,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            socks5_addr: "127.0.0.1:9050".to_string(),
            dns_port: 9053,
            enabled: false,
            circuit_build_timeout_ms: 30_000,
            max_circuit_retries: 3,
        }
    }
}

// ---------------------------------------------------------------------------
// TorConnection
// ---------------------------------------------------------------------------

/// A TCP connection proxied through Tor via SOCKS5.
///
/// Wraps a `tokio::net::TcpStream` that has already completed the SOCKS5
/// handshake, so subsequent reads/writes go through the Tor circuit.
pub struct TorConnection {
    /// Underlying TCP stream (already SOCKS5-negotiated).
    pub stream: TcpStream,
    /// The target address this connection is proxied to.
    target: String,
}

impl TorConnection {
    /// Send data through the Tor-proxied connection.
    pub async fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .await
            .map_err(|e| format!("Tor send error: {e}"))?;
        self.stream
            .flush()
            .await
            .map_err(|e| format!("Tor flush error: {e}"))?;
        Ok(())
    }

    /// Receive data from the Tor-proxied connection.
    ///
    /// Returns the number of bytes read into `buf`.
    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        self.stream
            .read(buf)
            .await
            .map_err(|e| format!("Tor recv error: {e}"))
    }

    /// Return the target address this connection is proxied to.
    pub fn target(&self) -> &str {
        &self.target
    }
}

// ---------------------------------------------------------------------------
// TorProxy
// ---------------------------------------------------------------------------

/// Tor SOCKS5 proxy client.
///
/// Handles the SOCKS5 handshake with the local Tor daemon and produces
/// `TorConnection` instances that route traffic through the Tor network.
pub struct TorProxy {
    /// Tor configuration.
    pub config: TorConfig,
    /// Whether we have verified that Tor is reachable.
    #[allow(dead_code)]
    connected: bool,
}

impl TorProxy {
    /// Create a new `TorProxy` from the given configuration.
    pub fn new(config: TorConfig) -> Self {
        let connected = false;
        Self { config, connected }
    }

    /// Establish a SOCKS5-proxied TCP connection to `target`.
    ///
    /// `target` should be a string in the form `hostname:port`, e.g.
    /// `example.onion:9200`.
    pub async fn connect(&self, target: &str) -> Result<TorConnection, String> {
        let (host, port) = parse_host_port(target)?;

        // 1. TCP connect to the Tor SOCKS5 proxy
        let mut stream = TcpStream::connect(&self.config.socks5_addr)
            .await
            .map_err(|e| {
                format!(
                    "Failed to connect to Tor SOCKS5 proxy at {}: {e}",
                    self.config.socks5_addr
                )
            })?;

        // 2. SOCKS5 greeting
        socks5_greet(&mut stream).await?;

        // 3. SOCKS5 CONNECT request
        socks5_connect(&mut stream, &host, port).await?;

        Ok(TorConnection {
            stream,
            target: target.to_string(),
        })
    }

    /// Resolve a `.onion` (or any) hostname via Tor's SOCKS5 RESOLVE command.
    ///
    /// Returns the raw IP address bytes (4 bytes for IPv4, 16 for IPv6).
    pub async fn resolve_onion(&self, hostname: &str) -> Result<Vec<u8>, String> {
        let mut stream = TcpStream::connect(&self.config.socks5_addr)
            .await
            .map_err(|e| format!("Failed to connect to Tor SOCKS5 proxy: {e}"))?;

        // SOCKS5 greeting
        socks5_greet(&mut stream).await?;

        // Build RESOLVE request
        let hostname_bytes = hostname.as_bytes();
        let hostname_len = hostname_bytes.len() as u8;
        // RESOLVE uses port 0 (no specific port needed for DNS lookups)
        let req = [
            &[
                SOCKS5_VERSION,
                SOCKS5_CMD_RESOLVE,
                SOCKS5_RESERVED,
                SOCKS5_ATYP_DOMAIN,
            ] as &[u8],
            &[hostname_len] as &[u8],
            hostname_bytes,
            &[0u8, 0u8] as &[u8], // port 0
        ]
        .concat();

        stream
            .write_all(&req)
            .await
            .map_err(|e| format!("SOCKS5 resolve write error: {e}"))?;

        // Read reply header: ver, reply, rsv, atyp (4 bytes)
        let mut reply_hdr = [0u8; 4];
        stream
            .read_exact(&mut reply_hdr)
            .await
            .map_err(|e| format!("SOCKS5 resolve reply read error: {e}"))?;

        if reply_hdr[1] != SOCKS5_REPLY_SUCCESS {
            return Err(format!(
                "SOCKS5 RESOLVE failed with reply code 0x{:02x}",
                reply_hdr[1]
            ));
        }

        // Read the bound address based on address type
        let addr_bytes = match reply_hdr[3] {
            SOCKS5_ATYP_IPV4 => {
                let mut buf = [0u8; 4];
                stream
                    .read_exact(&mut buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve IPv4 read error: {e}"))?;
                // Skip port (2 bytes)
                let mut port_buf = [0u8; 2];
                stream
                    .read_exact(&mut port_buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve port read error: {e}"))?;
                buf.to_vec()
            }
            SOCKS5_ATYP_IPV6 => {
                let mut buf = [0u8; 16];
                stream
                    .read_exact(&mut buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve IPv6 read error: {e}"))?;
                // Skip port (2 bytes)
                let mut port_buf = [0u8; 2];
                stream
                    .read_exact(&mut port_buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve port read error: {e}"))?;
                buf.to_vec()
            }
            SOCKS5_ATYP_DOMAIN => {
                let mut len_buf = [0u8; 1];
                stream
                    .read_exact(&mut len_buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve domain len read error: {e}"))?;
                let domain_len = len_buf[0] as usize;
                let mut domain_buf = vec![0u8; domain_len];
                stream
                    .read_exact(&mut domain_buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve domain read error: {e}"))?;
                // Skip port (2 bytes)
                let mut port_buf = [0u8; 2];
                stream
                    .read_exact(&mut port_buf)
                    .await
                    .map_err(|e| format!("SOCKS5 resolve port read error: {e}"))?;
                domain_buf
            }
            other => {
                return Err(format!(
                    "SOCKS5 RESOLVE: unknown address type 0x{:02x}",
                    other
                ));
            }
        };

        Ok(addr_bytes)
    }

    /// Check if Tor is running by attempting a TCP connection to the SOCKS5 port.
    ///
    /// Returns `true` if the port is reachable, `false` otherwise.
    /// This does **not** complete a full SOCKS5 handshake — it only checks
    /// that something is listening on the configured SOCKS5 address.
    pub fn check_tor_running(&self) -> bool {
        use std::net::TcpStream as StdTcpStream;
        StdTcpStream::connect_timeout(
            &self
                .config
                .socks5_addr
                .parse::<SocketAddr>()
                .unwrap_or_else(|_| "127.0.0.1:9050".parse().unwrap()),
            std::time::Duration::from_secs(2),
        ).is_ok()
    }

    /// Send data through a Tor-proxied connection and read the response.
    ///
    /// Convenience method that opens a connection, sends `data`, and reads
    /// up to 64 KiB of response.
    pub async fn proxy_request(&self, target: &str, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut conn = self.connect(target).await?;
        conn.send(data).await?;

        let mut buf = vec![0u8; 65536];
        let n = conn.recv(&mut buf).await?;
        buf.truncate(n);
        Ok(buf)
    }
}

// ---------------------------------------------------------------------------
// SOCKS5 handshake helpers
// ---------------------------------------------------------------------------

/// Perform the SOCKS5 greeting (method negotiation).
///
/// Sends `[0x05, 0x01, 0x00]` (version 5, 1 auth method, no-auth) and
/// expects the server to reply with `[0x05, 0x00]` (no-auth accepted).
async fn socks5_greet(stream: &mut TcpStream) -> Result<(), String> {
    // Greeting: version 5, 1 method, no-auth
    let greeting = [SOCKS5_VERSION, 0x01, SOCKS5_AUTH_NONE];
    stream
        .write_all(&greeting)
        .await
        .map_err(|e| format!("SOCKS5 greeting write error: {e}"))?;

    // Server reply: version, chosen method
    let mut reply = [0u8; 2];
    stream
        .read_exact(&mut reply)
        .await
        .map_err(|e| format!("SOCKS5 greeting read error: {e}"))?;

    if reply[0] != SOCKS5_VERSION {
        return Err(format!(
            "SOCKS5 greeting: unexpected version 0x{:02x}",
            reply[0]
        ));
    }
    if reply[1] != SOCKS5_AUTH_NONE {
        return Err(format!(
            "SOCKS5 greeting: server chose unsupported auth method 0x{:02x}",
            reply[1]
        ));
    }

    Ok(())
}

/// Perform the SOCKS5 CONNECT request.
///
/// Sends a CONNECT request with domain name address type (`0x03`) and
/// reads the server reply, verifying a `0x00` success code.
async fn socks5_connect(stream: &mut TcpStream, host: &str, port: u16) -> Result<(), String> {
    let host_bytes = host.as_bytes();
    let host_len = host_bytes.len() as u8;
    let port_hi = (port >> 8) as u8;
    let port_lo = (port & 0xFF) as u8;

    let req = [
        &[
            SOCKS5_VERSION,
            SOCKS5_CMD_CONNECT,
            SOCKS5_RESERVED,
            SOCKS5_ATYP_DOMAIN,
        ] as &[u8],
        &[host_len] as &[u8],
        host_bytes,
        &[port_hi, port_lo] as &[u8],
    ]
    .concat();

    stream
        .write_all(&req)
        .await
        .map_err(|e| format!("SOCKS5 connect request write error: {e}"))?;

    // Read reply header: ver, reply, rsv, atyp (4 bytes)
    let mut reply_hdr = [0u8; 4];
    stream
        .read_exact(&mut reply_hdr)
        .await
        .map_err(|e| format!("SOCKS5 connect reply read error: {e}"))?;

    if reply_hdr[0] != SOCKS5_VERSION {
        return Err(format!(
            "SOCKS5 connect: unexpected version 0x{:02x}",
            reply_hdr[0]
        ));
    }
    if reply_hdr[1] != SOCKS5_REPLY_SUCCESS {
        return Err(format!(
            "SOCKS5 connect failed with reply code 0x{:02x}: {}",
            reply_hdr[1],
            socks5_reply_description(reply_hdr[1])
        ));
    }

    // Consume the bound address so the stream is positioned at the start
    // of the proxied data.
    let addr_bytes = match reply_hdr[3] {
        SOCKS5_ATYP_IPV4 => 4usize,
        SOCKS5_ATYP_IPV6 => 16usize,
        SOCKS5_ATYP_DOMAIN => {
            let mut len_buf = [0u8; 1];
            stream
                .read_exact(&mut len_buf)
                .await
                .map_err(|e| format!("SOCKS5 connect domain length read error: {e}"))?;
            len_buf[0] as usize
        }
        other => {
            return Err(format!(
                "SOCKS5 connect: unknown bound address type 0x{:02x}",
                other
            ));
        }
    };

    // Read address + port (2 bytes)
    let mut discard = vec![0u8; addr_bytes + 2];
    stream
        .read_exact(&mut discard)
        .await
        .map_err(|e| format!("SOCKS5 connect bound address read error: {e}"))?;

    Ok(())
}

/// Human-readable description for SOCKS5 reply codes.
fn socks5_reply_description(code: u8) -> &'static str {
    match code {
        0x00 => "succeeded",
        0x01 => "general SOCKS server failure",
        0x02 => "connection not allowed by ruleset",
        0x03 => "Network unreachable",
        0x04 => "Host unreachable",
        0x05 => "Connection refused",
        0x06 => "TTL expired",
        0x07 => "Command not supported",
        0x08 => "Address type not supported",
        _ => "unknown error",
    }
}

// ---------------------------------------------------------------------------
// Address helpers
// ---------------------------------------------------------------------------

/// Parse a `host:port` string into `(host, port)`.
fn parse_host_port(target: &str) -> Result<(String, u16), String> {
    // Handle IPv6 addresses in brackets: [::1]:port
    if target.starts_with('[') {
        if let Some(bracket_end) = target.find(']') {
            let host = &target[1..bracket_end];
            let rest = &target[bracket_end + 1..];
            let port = rest
                .strip_prefix(':')
                .ok_or_else(|| format!("Missing port after IPv6 bracket in '{target}'"))?
                .parse::<u16>()
                .map_err(|e| format!("Invalid port in '{target}': {e}"))?;
            return Ok((host.to_string(), port));
        }
        return Err(format!("Malformed IPv6 address in '{target}'"));
    }

    // Standard host:port
    let mut parts = target.rsplitn(2, ':');
    let port_str = parts
        .next()
        .ok_or_else(|| format!("Cannot parse port from '{target}'"))?;
    let host = parts
        .next()
        .ok_or_else(|| format!("Cannot parse host from '{target}'"))?;

    if host.is_empty() {
        return Err(format!("Empty host in '{target}'"));
    }

    let port = port_str
        .parse::<u16>()
        .map_err(|e| format!("Invalid port '{port_str}' in '{target}': {e}"))?;

    Ok((host.to_string(), port))
}

/// Validate that a string looks like a `.onion` address (with optional port).
///
/// A valid `.onion` address is of the form `<hostname>.onion[:<port>]`
/// where `<hostname>` is 16 or 56 alphanumeric characters (v2 or v3).
pub fn is_valid_onion_address(addr: &str) -> bool {
    // Strip port if present
    let hostname = if addr.contains(':') {
        match parse_host_port(addr) {
            Ok((h, _)) => h,
            Err(_) => return false,
        }
    } else {
        addr.to_string()
    };

    if !hostname.ends_with(".onion") {
        return false;
    }

    let name = &hostname[..hostname.len() - ".onion".len()];
    // v2: 16 chars, v3: 56 chars — both alphanumeric
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    matches!(name.len(), 16 | 56)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tor_config_default() {
        let cfg = TorConfig::default();
        assert_eq!(cfg.socks5_addr, "127.0.0.1:9050");
        assert_eq!(cfg.dns_port, 9053);
        assert!(!cfg.enabled);
        assert_eq!(cfg.circuit_build_timeout_ms, 30_000);
        assert_eq!(cfg.max_circuit_retries, 3);
    }

    #[test]
    fn test_tor_config_serialization() {
        let cfg = TorConfig {
            socks5_addr: "127.0.0.1:9150".to_string(),
            dns_port: 9153,
            enabled: true,
            circuit_build_timeout_ms: 10_000,
            max_circuit_retries: 5,
        };

        // JSON roundtrip
        let json = serde_json::to_string(&cfg).unwrap();
        let back: TorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.socks5_addr, back.socks5_addr);
        assert_eq!(cfg.dns_port, back.dns_port);
        assert_eq!(cfg.enabled, back.enabled);
        assert_eq!(cfg.circuit_build_timeout_ms, back.circuit_build_timeout_ms);
        assert_eq!(cfg.max_circuit_retries, back.max_circuit_retries);

        // YAML roundtrip
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let back_yaml: TorConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(cfg.socks5_addr, back_yaml.socks5_addr);
        assert_eq!(cfg.enabled, back_yaml.enabled);
    }

    #[test]
    fn test_onion_address_validation() {
        // Valid v3 .onion (56-char hostname)
        let v3_56: String = "a".repeat(56);
        let v3 = format!("{}.onion", v3_56);
        assert!(is_valid_onion_address(&v3));
        assert!(is_valid_onion_address(&format!("{v3}:9200")));

        // Valid v2 .onion (16-char hostname)
        let v2 = "abcdefghijklmnop.onion";
        assert!(is_valid_onion_address(v2));
        assert!(is_valid_onion_address(&format!("{v2}:9200")));

        // Invalid: not .onion
        assert!(!is_valid_onion_address("example.com"));
        assert!(!is_valid_onion_address("example.com:9200"));

        // Invalid: wrong length
        assert!(!is_valid_onion_address("short.onion"));
        assert!(!is_valid_onion_address(&("a".repeat(57) + ".onion")));

        // Invalid: bad characters
        assert!(!is_valid_onion_address("abcdefg!hijklmnop.onion"));

        // Invalid: empty hostname
        assert!(!is_valid_onion_address(".onion"));
    }

    #[test]
    fn test_socks5_greeting_message_format() {
        // The greeting message must be [0x05, 0x01, 0x00]
        let greeting = [SOCKS5_VERSION, 0x01, SOCKS5_AUTH_NONE];
        assert_eq!(greeting, [0x05, 0x01, 0x00]);
        assert_eq!(greeting[0], 5, "SOCKS version must be 5");
        assert_eq!(greeting[1], 1, "Number of auth methods must be 1");
        assert_eq!(greeting[2], 0x00, "Auth method must be no-auth (0x00)");
    }

    #[test]
    fn test_socks5_connect_request_format() {
        // Build the connect request for "example.onion:9200"
        let host = "example.onion";
        let port: u16 = 9200;
        let host_bytes = host.as_bytes();
        let host_len = host_bytes.len() as u8;
        let port_hi = (port >> 8) as u8;
        let port_lo = (port & 0xFF) as u8;

        let req = [
            &[
                SOCKS5_VERSION,
                SOCKS5_CMD_CONNECT,
                SOCKS5_RESERVED,
                SOCKS5_ATYP_DOMAIN,
            ] as &[u8],
            &[host_len] as &[u8],
            host_bytes,
            &[port_hi, port_lo] as &[u8],
        ]
        .concat();

        // Verify header
        assert_eq!(req[0], 0x05, "SOCKS version");
        assert_eq!(req[1], 0x01, "CONNECT command");
        assert_eq!(req[2], 0x00, "Reserved byte");
        assert_eq!(req[3], 0x03, "Domain name address type");

        // Verify domain length
        assert_eq!(req[4], host_bytes.len() as u8, "domain length byte");

        // Verify domain bytes
        assert_eq!(&req[5..5 + host_bytes.len()], host_bytes);

        // Verify port (9200 = 0x23F0)
        let port_offset = 5 + host_bytes.len();
        assert_eq!(req[port_offset], 0x23);
        assert_eq!(req[port_offset + 1], 0xF0);

        // Full expected message for this specific case
        // "example.onion" = 13 bytes, so domain length byte is 0x0D
        let expected = [
            0x05, 0x01, 0x00, 0x03, 0x0D, // header + domain length (13)
            b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'o', b'n', b'i', b'o', b'n', 0x23,
            0xF0, // port 9200
        ];
        assert_eq!(req, expected);
    }

    #[test]
    fn test_parse_host_port() {
        assert_eq!(
            parse_host_port("example.onion:9200").unwrap(),
            ("example.onion".to_string(), 9200)
        );
        assert_eq!(
            parse_host_port("127.0.0.1:9050").unwrap(),
            ("127.0.0.1".to_string(), 9050)
        );
        assert_eq!(
            parse_host_port("[::1]:9200").unwrap(),
            ("::1".to_string(), 9200)
        );

        // Error cases
        assert!(parse_host_port("no-port").is_err());
        assert!(parse_host_port(":9200").is_err());
        assert!(parse_host_port("host:abc").is_err());
    }

    #[test]
    fn test_socks5_reply_description() {
        assert_eq!(socks5_reply_description(0x00), "succeeded");
        assert_eq!(
            socks5_reply_description(0x01),
            "general SOCKS server failure"
        );
        assert_eq!(socks5_reply_description(0x05), "Connection refused");
        assert_eq!(socks5_reply_description(0xFF), "unknown error");
    }

    #[test]
    fn test_tor_proxy_new() {
        let config = TorConfig::default();
        let proxy = TorProxy::new(config);
        assert!(!proxy.connected);
        assert_eq!(proxy.config.socks5_addr, "127.0.0.1:9050");
    }
}
