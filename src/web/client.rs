//! HTTP client for web operations with comprehensive safety features
//!
//! This module provides a safe, efficient HTTP client that includes:
//! - Connection pooling for performance
//! - Automatic rate limiting per domain
//! - Robots.txt compliance checking
//! - Response size limits
//! - Timeout handling
//! - Content filtering
//! - Caching with stale-while-revalidate

use crate::error::{Error, Result};
use crate::web::cache::WebCache;
use crate::web::config::WebConfig;
use crate::web::filter::ContentFilter;
use crate::web::robots::RobotsChecker;
use crate::web::safety::UrlValidator;
use crate::web::{validate_url, SafetyLevel, MAX_REDIRECTS};
use reqwest::redirect::Policy;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use url::Url;

/// Rate limit entry for tracking requests per domain
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// First request time in the current window
    window_start: Instant,
    /// Number of requests in the current window
    request_count: u32,
    /// Last request time
    last_request: Instant,
    /// Burst count
    burst_count: u32,
    /// Burst window start
    burst_window_start: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        let now = Instant::now();
        RateLimitEntry {
            window_start: now,
            request_count: 0,
            last_request: now,
            burst_count: 0,
            burst_window_start: now,
        }
    }

    fn can_make_request(&self, config: &crate::web::config::RateLimitConfig) -> bool {
        let now = Instant::now();

        // Check burst limit
        let burst_window = Duration::from_millis(config.burst_cooldown_ms);
        if now.duration_since(self.burst_window_start) < burst_window {
            if self.burst_count >= config.burst_limit {
                return false;
            }
        }

        // Check rate limit
        let window = Duration::from_secs(60);
        if now.duration_since(self.window_start) < window {
            if self.request_count >= config.requests_per_minute {
                return false;
            }
        }

        // Check minimum delay
        let min_delay = Duration::from_millis(config.min_delay_ms);
        if now.duration_since(self.last_request) < min_delay {
            return false;
        }

        true
    }

    fn record_request(&mut self) {
        let now = Instant::now();

        // Reset window if needed
        let window = Duration::from_secs(60);
        if now.duration_since(self.window_start) >= window {
            self.window_start = now;
            self.request_count = 0;
        }

        self.request_count += 1;
        self.last_request = now;

        // Track burst
        let burst_window = Duration::from_millis(config.burst_cooldown_ms);
        if now.duration_since(self.burst_window_start) >= burst_window {
            self.burst_window_start = now;
            self.burst_count = 0;
        }
        self.burst_count += 1;
    }

    fn wait_time(&self, config: &crate::web::config::RateLimitConfig) -> Duration {
        let now = Instant::now();

        // Check min delay
        let min_delay = Duration::from_millis(config.min_delay_ms);
        let delay_remaining = min_delay.saturating_sub(now.duration_since(self.last_request));

        // Check rate limit
        let window = Duration::from_secs(60);
        if self.request_count >= config.requests_per_minute {
            let window_remaining = window.saturating_sub(now.duration_since(self.window_start));
            return std::cmp::max(delay_remaining, window_remaining);
        }

        // Check burst
        let burst_window = Duration::from_millis(config.burst_cooldown_ms);
        if self.burst_count >= config.burst_limit {
            let burst_remaining = burst_window.saturating_sub(now.duration_since(self.burst_window_start));
            return std::cmp::max(delay_remaining, burst_remaining);
        }

        delay_remaining
    }
}

/// Web client for making HTTP requests safely
#[derive(Debug)]
pub struct WebClient {
    /// HTTP client
    client: Client,
    /// Configuration
    config: WebConfig,
    /// Content filter
    filter: ContentFilter,
    /// Robots.txt checker
    robots_checker: RobotsChecker,
    /// URL validator
    url_validator: UrlValidator,
    /// Response cache
    cache: WebCache,
    /// Rate limit tracking per domain
    rate_limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// User agents for rotation
    user_agents: Vec<String>,
    /// Current user agent index
    user_agent_index: Arc<RwLock<usize>>,
}

impl WebClient {
    /// Create a new web client with configuration
    pub fn new(config: WebConfig) -> Result<Self> {
        // Build HTTP client with safe defaults
        let mut client_builder = Client::builder()
            .timeout(config.timeout())
            .connect_timeout(Duration::from_secs(10))
            .max_redirects(config.max_redirects)
            .redirect(Policy::limited(config.max_redirects))
            .gzip(true)
            .brotli(true)
            .deflate(true);

        // Configure TLS
        if !config.tls.verify_certificates || config.tls.accept_invalid_certs {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        // Configure proxy
        if config.proxy.enabled {
            if let Some(ref http_proxy) = config.proxy.http_proxy {
                let proxy = reqwest::Proxy::http(http_proxy)
                    .map_err(|e| Error::Generic(format!("Proxy error: {}", e)))?;
                client_builder = client_builder.proxy(proxy);
            }
            if let Some(ref https_proxy) = config.proxy.https_proxy {
                let proxy = reqwest::Proxy::https(https_proxy)
                    .map_err(|e| Error::Generic(format!("Proxy error: {}", e)))?;
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder
            .build()
            .map_err(|e| Error::Generic(format!("Failed to build HTTP client: {}", e)))?;

        // Initialize user agents
        let user_agents = if config.user_agent.rotate {
            config.user_agent.rotation_list.clone()
        } else {
            config.user_agent.custom.clone().into_iter().collect()
        };

        let filter = ContentFilter::new(config.content_filter.clone());
        let robots_checker = RobotsChecker::new(config.robots.clone());
        let url_validator = UrlValidator::new(config.safety_level);
        let cache = WebCache::new(config.cache.clone());

        Ok(WebClient {
            client,
            config,
            filter,
            robots_checker,
            url_validator,
            cache,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            user_agents,
            user_agent_index: Arc::new(RwLock::new(0)),
        })
    }

    /// Create a client with safe defaults
    pub fn safe_default() -> Result<Self> {
        Self::new(WebConfig::safe_default())
    }

    /// Create a client for local trusted use
    pub fn local_trusted() -> Result<Self> {
        Self::new(WebConfig::local_trusted())
    }

    /// Get the next user agent
    async fn get_user_agent(&self) -> String {
        if self.user_agents.is_empty() {
            return concat!("pb-mcp-server/", env!("CARGO_PKG_VERSION")).to_string();
        }

        if self.config.user_agent.rotate {
            let mut index = self.user_agent_index.write().await;
            let ua = self.user_agents[*index % self.user_agents.len()].clone();
            *index = (*index + 1) % self.user_agents.len();
            ua
        } else {
            self.user_agents.first().cloned().unwrap_or_else(|| {
                concat!("pb-mcp-server/", env!("CARGO_PKG_VERSION)).to_string()
            })
        }
    }

    /// Check rate limit for a domain
    async fn check_rate_limit(&self, domain: &str) -> Result<()> {
        let mut rate_limits = self.rate_limits.write().await;
        let entry = rate_limits.entry(domain.to_string()).or_insert_with(RateLimitEntry::new);

        if !entry.can_make_request(&self.config.rate_limit) {
            let wait_time = entry.wait_time(&self.config.rate_limit);
            return Err(Error::RateLimitExceeded {
                operation: format!("web request to {}", domain),
                limit: self.config.rate_limit.requests_per_minute as usize,
                window_secs: 60,
            });
        }

        Ok(())
    }

    /// Record a request for rate limiting
    async fn record_request(&self, domain: &str) {
        let mut rate_limits = self.rate_limits.write().await;
        if let Some(entry) = rate_limits.get_mut(domain) {
            entry.record_request();
        }
    }

    /// Validate URL and check robots.txt
    pub async fn validate_and_check(&self, url: &str) -> Result<Url> {
        // Validate URL structure
        let parsed = validate_url(url, self.config.safety_level)?;

        // Check domain allowlist/blocklist
        if let Some(domain) = parsed.host_str() {
            if !self.config.is_domain_allowed(domain) {
                return Err(Error::security_violation(format!(
                    "Domain '{}' is not allowed by policy",
                    domain
                )));
            }
        }

        // Check robots.txt
        if self.config.robots.respect_robots_txt {
            if !self.robots_checker.is_allowed(&parsed).await? {
                return Err(Error::security_violation(format!(
                    "URL '{}' is disallowed by robots.txt",
                    url
                )));
            }
        }

        Ok(parsed)
    }

    /// Fetch a URL and return raw bytes
    pub async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let parsed = self.validate_and_check(url).await?;
        let domain = parsed.host_str().unwrap_or("").to_string();

        // Check cache
        if self.cache.is_enabled() {
            if let Some(cached) = self.cache.get(&parsed).await {
                return Ok(cached);
            }
        }

        // Check rate limit
        self.check_rate_limit(&domain).await?;

        // Get user agent
        let user_agent = self.get_user_agent().await;

        // Make request
        let response = self.client
            .get(parsed.clone())
            .header("User-Agent", &user_agent)
            .header("Accept", "*/*")
            .send()
            .await
            .map_err(|e| Error::Generic(format!("Request failed: {}", e)))?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Generic(format!(
                "HTTP {} for {}",
                status.as_u16(),
                url
            )));
        }

        // Check content length
        if let Some(content_length) = response.content_length() {
            if content_length > self.config.max_response_size as u64 {
                return Err(Error::Generic(format!(
                    "Response too large: {} bytes (max: {})",
                    content_length,
                    self.config.max_response_size
                )));
            }
        }

        // Read response body with size limit
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::Generic(format!("Failed to read response: {}", e)))?;

        // Check actual size
        if bytes.len() > self.config.max_response_size {
            return Err(Error::Generic(format!(
                "Response too large: {} bytes",
                bytes.len()
            )));
        }

        // Record request
        self.record_request(&domain).await;

        // Cache response
        if self.cache.is_enabled() {
            self.cache.set(&parsed, &bytes).await;
        }

        Ok(bytes.to_vec())
    }

    /// Fetch a URL and return text content
    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        let bytes = self.fetch_bytes(url).await?;

        // Convert to string
        String::from_utf8(bytes.clone())
            .or_else(|_| {
                // Try with lossy conversion
                Ok(String::from_utf8_lossy(&bytes).to_string())
            })
    }

    /// Fetch a URL and parse as JSON
    pub async fn fetch_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let text = self.fetch_text(url).await?;

        serde_json::from_str(&text)
            .map_err(|e| Error::json(format!("JSON parse error: {}", e)))
    }

    /// Fetch and filter HTML content
    pub async fn fetch_html_filtered(&self, url: &str) -> Result<String> {
        let html = self.fetch_text(url).await?;
        Ok(self.filter.filter_html(&html)?)
    }

    /// Fetch and extract clean text from HTML
    pub async fn fetch_html_text(&self, url: &str) -> Result<String> {
        let html = self.fetch_text(url).await?;
        Ok(self.filter.extract_text(&html))
    }

    /// Check if a URL is accessible
    pub async fn check_url(&self, url: &str) -> Result<UrlInfo> {
        let parsed = self.validate_and_check(url).await?;
        let domain = parsed.host_str().unwrap_or("").to_string();

        // Check rate limit
        self.check_rate_limit(&domain).await?;

        // Make HEAD request
        let user_agent = self.get_user_agent().await;
        let response = self.client
            .head(parsed.clone())
            .header("User-Agent", &user_agent)
            .send()
            .await
            .map_err(|e| Error::Generic(format!("Request failed: {}", e)))?;

        self.record_request(&domain).await;

        let status = response.status();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        Ok(UrlInfo {
            url: parsed.to_string(),
            status: status.as_u16(),
            content_type,
            content_length,
            accessible: status.is_success(),
        })
    }

    /// Get HTTP headers for a URL
    pub async fn head(&self, url: &str) -> Result<HashMap<String, String>> {
        let parsed = self.validate_and_check(url).await?;
        let domain = parsed.host_str().unwrap_or("").to_string();

        // Check rate limit
        self.check_rate_limit(&domain).await?;

        // Make HEAD request
        let user_agent = self.get_user_agent().await;
        let response = self.client
            .head(parsed.clone())
            .header("User-Agent", &user_agent)
            .send()
            .await
            .map_err(|e| Error::Generic(format!("Request failed: {}", e)))?;

        self.record_request(&domain).await;

        // Extract headers
        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        Ok(headers)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    /// Clear rate limits
    pub async fn clear_rate_limits(&self) {
        let mut rate_limits = self.rate_limits.write().await;
        rate_limits.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &WebConfig {
        &self.config
    }
}

/// URL information from HEAD request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UrlInfo {
    /// The URL
    pub url: String,
    /// HTTP status code
    pub status: u16,
    /// Content type
    pub content_type: String,
    /// Content length in bytes
    pub content_length: u64,
    /// Whether the URL is accessible
    pub accessible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::config::{RateLimitConfig, WebConfig};

    #[tokio::test]
    async fn test_client_creation() {
        let config = WebConfig::test_config();
        let client = WebClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_rate_limit_entry() {
        let config = RateLimitConfig::default();
        let entry = RateLimitEntry::new();

        // Should allow initial requests
        assert!(entry.can_make_request(&config));

        // Record some requests
        let mut entry = RateLimitEntry::new();
        for _ in 0..config.burst_limit {
            entry.record_request();
        }

        // Should be at burst limit
        assert!(!entry.can_make_request(&config));
    }

    #[tokio::test]
    async fn test_user_agent_rotation() {
        let mut config = WebConfig::test_config();
        config.user_agent.rotate = true;
        config.user_agent.rotation_list = vec![
            "agent1".to_string(),
            "agent2".to_string(),
            "agent3".to_string(),
        ];

        let client = WebClient::new(config).unwrap();

        // Test rotation
        let ua1 = client.get_user_agent().await;
        let ua2 = client.get_user_agent().await;
        let ua3 = client.get_user_agent().await;
        let ua4 = client.get_user_agent().await;

        assert_ne!(ua1, ua2);
        assert_ne!(ua2, ua3);
        assert_eq!(ua1, ua4); // Should wrap around
    }

    #[test]
    fn test_url_info_serialization() {
        let info = UrlInfo {
            url: "https://example.com".to_string(),
            status: 200,
            content_type: "text/html".to_string(),
            content_length: 12345,
            accessible: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("example.com"));
        assert!(json.contains("200"));
    }
}
