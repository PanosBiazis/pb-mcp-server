/// Deployment trust tier: LAN/local vs internet-exposed MCP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkProfile {
    /// Trusted network (default): full surface subject to `MINT_*` gates.
    Local,
    /// Untrusted / public internet: block mutations (bus, terminal, risky git, disk writes from tools).
    Public,
}

impl NetworkProfile {
    /// Create a NetworkProfile from environment variables
    ///
    /// Reads from `MINT_NETWORK_PROFILE` environment variable:
    /// - "public", "internet", or "wan" → Public profile
    /// - Any other value or unset → Local profile (default)
    pub fn from_env() -> Self {
        match std::env::var("MINT_NETWORK_PROFILE") {
            Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
                "public" | "internet" | "wan" => Self::Public,
                _ => Self::Local,
            },
            Err(_) => Self::Local,
        }
    }

    /// Get the string label for this profile
    ///
    /// Returns "local" for Local profile, "public" for Public profile
    pub const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Public => "public",
        }
    }

    /// Check if this is a public network profile
    ///
    /// Returns true for Public profile, false for Local profile
    pub const fn is_public(self) -> bool {
        matches!(self, Self::Public)
    }

    /// Max duration for `server_sleep` on this profile.
    pub const fn server_sleep_cap_ms(self) -> u64 {
        match self {
            Self::Local => 30_000,
            Self::Public => 5_000,
        }
    }
}
