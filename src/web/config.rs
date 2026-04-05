//! Web configuration for pb-mcp-server
//!
//! This module provides configuration options for web tools including
//! safety levels, timeouts, rate limiting, and caching.

use std::time::Duration;

/// Safety level for web operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafetyLevel {
    /// Strict mode - only allowlisted domains
    Strict,
    /// Moderate mode - block known dangerous content
    Moderate,
    /// Permissive mode - allow most content with basic checks
    Permissive,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        SafetyLevel::Moderate
    }
}

impl SafetyLevel {
    /// Get recommended timeout for this safety level
    pub fn recommended_timeout(&self) -> Duration {
        match self {
            SafetyLevel::Strict => Duration::from_secs(60),
            SafetyLevel::Moderate => Duration::from_secs(30),
            SafetyLevel::Permissive => Duration::from_secs(20),
        }
    }

    /// Get maximum response size for this safety level
    pub fn max_response_size(&self) -> usize {
        match self {
            SafetyLevel::Strict => 5 * 1024 * 1024,      // 5 MB
            SafetyLevel::Moderate => 10 * 1024 * 1024,   // 10 MB
            SafetyLevel::Permissive => 20 * 1024 * 1024, // 20 MB
        }
    }

    /// Get maximum redirects to follow
    pub fn max_redirects(&self) -> usize {
        match self {
            SafetyLevel::Strict => 3,
            SafetyLevel::Moderate => 5,
            SafetyLevel::Permissive => 10,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per domain per minute
    pub requests_per_minute: u32,
    /// Minimum delay between requests to same domain (ms)
    pub min_delay_ms: u64,
    /// Burst limit (requests allowed in rapid succession)
    pub burst_limit: u32,
    /// Cooldown period after burst (ms)
    pub burst_cooldown_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            requests_per_minute: 60,
            min_delay_ms: 1000,
            burst_limit: 10,
            burst_cooldown_ms: 5000,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Maximum cache size in bytes
    pub max_size_bytes: usize,
    /// Time-to-live for cached entries (seconds)
    pub ttl_secs: u64,
    /// Maximum age for stale-while-revalidate (seconds)
    pub stale_while_revalidate_secs: u64,
    /// Cache successful responses
    pub cache_success: bool,
    /// Cache error responses (for short time)
    pub cache_errors: bool,
    /// Error cache TTL (seconds)
    pub error_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: true,
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            ttl_secs: 3600,                    // 1 hour
            stale_while_revalidate_secs: 300,  // 5 minutes
            cache_success: true,
            cache_errors: true,
            error_ttl_secs: 60, // 1 minute
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxyConfig {
    /// Enable proxy
    pub enabled: bool,
    /// HTTP proxy URL
    pub http_proxy: Option<String>,
    /// HTTPS proxy URL
    pub https_proxy: Option<String>,
    /// Proxy authentication username
    pub username: Option<String>,
    /// Proxy authentication password
    pub password: Option<String>,
    /// Bypass proxy for these domains
    pub no_proxy: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig {
            enabled: false,
            http_proxy: None,
            https_proxy: None,
            username: None,
            password: None,
            no_proxy: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        }
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TlsConfig {
    /// Verify SSL certificates
    pub verify_certificates: bool,
    /// Minimum TLS version
    pub min_tls_version: String,
    /// Accept invalid certificates (dangerous!)
    pub accept_invalid_certs: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        TlsConfig {
            verify_certificates: true,
            min_tls_version: "1.2".to_string(),
            accept_invalid_certs: false,
        }
    }
}

/// User agent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserAgentConfig {
    /// Custom user agent string
    pub custom: Option<String>,
    /// Rotate user agents to avoid blocking
    pub rotate: bool,
    /// List of user agents to rotate through
    pub rotation_list: Vec<String>,
}

impl Default for UserAgentConfig {
    fn default() -> Self {
        let default_ua = concat!(
            "pb-mcp-server/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/PanosBiazis/pb-mcp-server)"
        );

        UserAgentConfig {
            custom: None,
            rotate: false,
            rotation_list: vec![
                default_ua.to_string(),
                format!("Mozilla/5.0 (compatible; {default_ua})"),
            ],
        }
    }
}

/// Content filtering configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentFilterConfig {
    /// Strip JavaScript from HTML
    pub strip_javascript: bool,
    /// Strip CSS styles
    pub strip_styles: bool,
    /// Strip images
    pub strip_images: bool,
    /// Strip iframes
    pub strip_iframes: bool,
    /// Strip forms
    pub strip_forms: bool,
    /// Remove tracking scripts
    pub remove_tracking: bool,
    /// Block ad domains
    pub block_ads: bool,
    /// Maximum HTML size to process (bytes)
    pub max_html_size: usize,
}

impl Default for ContentFilterConfig {
    fn default() -> Self {
        ContentFilterConfig {
            strip_javascript: true,
            strip_styles: false,
            strip_images: false,
            strip_iframes: true,
            strip_forms: false,
            remove_tracking: true,
            block_ads: true,
            max_html_size: 5 * 1024 * 1024, // 5 MB
        }
    }
}

/// Robots.txt compliance configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobotsConfig {
    /// Respect robots.txt
    pub respect_robots_txt: bool,
    /// Custom user agent for robots.txt checks
    pub user_agent: String,
    /// Cache robots.txt entries (seconds)
    pub cache_ttl_secs: u64,
    /// Request delay from robots.txt crawl-delay
    pub respect_crawl_delay: bool,
}

impl Default for RobotsConfig {
    fn default() -> Self {
        RobotsConfig {
            respect_robots_txt: true,
            user_agent: concat!("pb-mcp-server/", env!("CARGO_PKG_VERSION")).to_string(),
            cache_ttl_secs: 3600, // 1 hour
            respect_crawl_delay: true,
        }
    }
}

/// Search engine configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchConfig {
    /// Enable web search
    pub enabled: bool,
    /// Default search engine
    pub default_engine: SearchEngine,
    /// Search API key (if required)
    pub api_key: Option<String>,
    /// Custom search engine URL
    pub custom_engine_url: Option<String>,
    /// Maximum results per search
    pub max_results: usize,
    /// Safe search setting
    pub safe_search: bool,
}

/// Supported search engines
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    DuckDuckGo,
    Google,
    Bing,
    Custom,
}

impl Default for SearchEngine {
    fn default() -> Self {
        SearchEngine::DuckDuckGo
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            enabled: true,
            default_engine: SearchEngine::DuckDuckGo,
            api_key: None,
            custom_engine_url: None,
            max_results: 10,
            safe_search: true,
        }
    }
}

/// Main web configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebConfig {
    /// Safety level
    pub safety_level: SafetyLevel,
    /// Request timeout (seconds)
    pub timeout_secs: u64,
    /// Maximum response size (bytes)
    pub max_response_size: usize,
    /// Maximum redirects
    pub max_redirects: usize,
    /// Rate limiting config
    pub rate_limit: RateLimitConfig,
    /// Cache config
    pub cache: CacheConfig,
    /// Proxy config
    pub proxy: ProxyConfig,
    /// TLS config
    pub tls: TlsConfig,
    /// User agent config
    pub user_agent: UserAgentConfig,
    /// Content filter config
    pub content_filter: ContentFilterConfig,
    /// Robots.txt config
    pub robots: RobotsConfig,
    /// Search config
    pub search: SearchConfig,
    /// Domain allowlist (for strict mode)
    pub allowlist: Vec<String>,
    /// Domain blocklist
    pub blocklist: Vec<String>,
    /// Enable request logging
    pub enable_logging: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            safety_level: SafetyLevel::Moderate,
            timeout_secs: 30,
            max_response_size: 10 * 1024 * 1024, // 10 MB
            max_redirects: 10,
            rate_limit: RateLimitConfig::default(),
            cache: CacheConfig::default(),
            proxy: ProxyConfig::default(),
            tls: TlsConfig::default(),
            user_agent: UserAgentConfig::default(),
            content_filter: ContentFilterConfig::default(),
            robots: RobotsConfig::default(),
            search: SearchConfig::default(),
            allowlist: vec![],
            blocklist: vec![],
            enable_logging: true,
        }
    }
}

impl WebConfig {
    /// Create a safe configuration for public use
    pub fn safe_default() -> Self {
        WebConfig {
            safety_level: SafetyLevel::Strict,
            timeout_secs: 20,
            max_response_size: 1024 * 1024, // 1 MB
            max_redirects: 3,
            rate_limit: RateLimitConfig {
                requests_per_minute: 30,
                min_delay_ms: 2000,
                burst_limit: 5,
                burst_cooldown_ms: 10000,
            },
            cache: CacheConfig {
                enabled: true,
                max_size_bytes: 50 * 1024 * 1024,
                ttl_secs: 1800, // 30 minutes
                ..Default::default()
            },
            content_filter: ContentFilterConfig {
                strip_javascript: true,
                strip_styles: true,
                strip_images: true,
                strip_iframes: true,
                strip_forms: true,
                remove_tracking: true,
                block_ads: true,
                max_html_size: 1024 * 1024, // 1 MB
            },
            search: SearchConfig {
                enabled: false,
                ..Default::default()
            },
            enable_logging: true,
            ..Default::default()
        }
    }

    /// Create a configuration for local trusted use
    pub fn local_trusted() -> Self {
        WebConfig {
            safety_level: SafetyLevel::Permissive,
            timeout_secs: 60,
            max_response_size: 50 * 1024 * 1024, // 50 MB
            max_redirects: 15,
            rate_limit: RateLimitConfig {
                requests_per_minute: 120,
                min_delay_ms: 500,
                burst_limit: 20,
                burst_cooldown_ms: 3000,
            },
            cache: CacheConfig {
                max_size_bytes: 200 * 1024 * 1024, // 200 MB
                ttl_secs: 7200,                    // 2 hours
                ..Default::default()
            },
            search: SearchConfig {
                enabled: true,
                max_results: 20,
                ..Default::default()
            },
            enable_logging: true,
            ..Default::default()
        }
    }

    /// Create a minimal configuration for testing
    pub fn test_config() -> Self {
        WebConfig {
            safety_level: SafetyLevel::Permissive,
            timeout_secs: 5,
            max_response_size: 1024 * 1024, // 1 MB
            max_redirects: 2,
            cache: CacheConfig {
                enabled: false,
                ..Default::default()
            },
            enable_logging: false,
            ..Default::default()
        }
    }

    /// Get effective timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Check if a domain is allowed based on allowlist/blocklist
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();

        // Check blocklist first
        for blocked in &self.blocklist {
            if domain_lower.contains(&blocked.to_lowercase()) {
                return false;
            }
        }

        // In strict mode, check allowlist
        match self.safety_level {
            SafetyLevel::Strict => {
                if self.allowlist.is_empty() {
                    true // If no allowlist, allow all non-blocked
                } else {
                    self.allowlist
                        .iter()
                        .any(|allowed| domain_lower.contains(&allowed.to_lowercase()))
                }
            }
            _ => true,
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = WebConfig::default();

        // Safety level
        if let Ok(level) = std::env::var("PB_WEB_SAFETY_LEVEL") {
            config.safety_level = match level.to_lowercase().as_str() {
                "strict" => SafetyLevel::Strict,
                "moderate" => SafetyLevel::Moderate,
                "permissive" => SafetyLevel::Permissive,
                _ => SafetyLevel::Moderate,
            };
        }

        // Timeout
        if let Ok(timeout) = std::env::var("PB_WEB_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse() {
                config.timeout_secs = secs;
            }
        }

        // Max response size
        if let Ok(size) = std::env::var("PB_WEB_MAX_RESPONSE_SIZE") {
            if let Ok(bytes) = size.parse() {
                config.max_response_size = bytes;
            }
        }

        // Enable logging
        if let Ok(logging) = std::env::var("PB_WEB_ENABLE_LOGGING") {
            config.enable_logging = logging != "0" && logging.to_lowercase() != "false";
        }

        // Proxy settings
        if let Ok(http_proxy) = std::env::var("HTTP_PROXY") {
            config.proxy.enabled = true;
            config.proxy.http_proxy = Some(http_proxy);
        }
        if let Ok(https_proxy) = std::env::var("HTTPS_PROXY") {
            config.proxy.enabled = true;
            config.proxy.https_proxy = Some(https_proxy);
        }

        // User agent
        if let Ok(ua) = std::env::var("PB_WEB_USER_AGENT") {
            config.user_agent.custom = Some(ua);
        }

        // Search API key
        if let Ok(key) = std::env::var("PB_WEB_SEARCH_API_KEY") {
            config.search.api_key = Some(key);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_level_defaults() {
        let strict = SafetyLevel::Strict;
        assert_eq!(strict.recommended_timeout(), Duration::from_secs(60));
        assert_eq!(strict.max_response_size(), 5 * 1024 * 1024);
        assert_eq!(strict.max_redirects(), 3);

        let moderate = SafetyLevel::Moderate;
        assert_eq!(moderate.recommended_timeout(), Duration::from_secs(30));

        let permissive = SafetyLevel::Permissive;
        assert_eq!(permissive.max_redirects(), 10);
    }

    #[test]
    fn test_web_config_default() {
        let config = WebConfig::default();
        assert_eq!(config.safety_level, SafetyLevel::Moderate);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.cache.enabled);
        assert!(config.tls.verify_certificates);
    }

    #[test]
    fn test_web_config_safe_default() {
        let config = WebConfig::safe_default();
        assert_eq!(config.safety_level, SafetyLevel::Strict);
        assert_eq!(config.max_response_size, 1024 * 1024);
        assert!(!config.search.enabled);
    }

    #[test]
    fn test_web_config_local_trusted() {
        let config = WebConfig::local_trusted();
        assert_eq!(config.safety_level, SafetyLevel::Permissive);
        assert!(config.search.enabled);
        assert_eq!(config.rate_limit.requests_per_minute, 120);
    }

    #[test]
    fn test_domain_allowed() {
        let mut config = WebConfig::default();
        config.blocklist = vec!["blocked.com".to_string()];
        config.allowlist = vec!["allowed.com".to_string()];

        // Test blocklist
        assert!(!config.is_domain_allowed("blocked.com"));
        assert!(!config.is_domain_allowed("sub.blocked.com"));

        // Test normal domains in moderate mode
        assert!(config.is_domain_allowed("example.com"));
    }

    #[test]
    fn test_domain_allowed_strict() {
        let mut config = WebConfig::default();
        config.safety_level = SafetyLevel::Strict;
        config.allowlist = vec!["allowed.com".to_string()];

        // Only allowlisted domains
        assert!(config.is_domain_allowed("allowed.com"));
        assert!(config.is_domain_allowed("sub.allowed.com"));
        assert!(!config.is_domain_allowed("other.com"));
    }

    #[test]
    fn test_rate_limit_defaults() {
        let rl = RateLimitConfig::default();
        assert_eq!(rl.requests_per_minute, 60);
        assert_eq!(rl.min_delay_ms, 1000);
        assert_eq!(rl.burst_limit, 10);
    }

    #[test]
    fn test_cache_defaults() {
        let cache = CacheConfig::default();
        assert!(cache.enabled);
        assert_eq!(cache.max_size_bytes, 100 * 1024 * 1024);
        assert_eq!(cache.ttl_secs, 3600);
    }

    #[test]
    fn test_content_filter_defaults() {
        let filter = ContentFilterConfig::default();
        assert!(filter.strip_javascript);
        assert!(filter.strip_iframes);
        assert!(filter.remove_tracking);
        assert!(!filter.strip_styles);
        assert!(!filter.strip_images);
    }

    #[test]
    fn test_robots_config_defaults() {
        let robots = RobotsConfig::default();
        assert!(robots.respect_robots_txt);
        assert!(robots.respect_crawl_delay);
        assert_eq!(robots.cache_ttl_secs, 3600);
    }

    #[test]
    fn test_search_config_defaults() {
        let search = SearchConfig::default();
        assert!(search.enabled);
        assert_eq!(search.max_results, 10);
        assert!(search.safe_search);
    }

    #[test]
    fn test_config_timeout_duration() {
        let config = WebConfig::default();
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }
}
