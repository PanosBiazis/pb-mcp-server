//! Web tools module for pb-mcp-server
//!
//! This module provides safe, efficient, and powerful web access tools for AI agents.
//! All web operations follow security best practices:
//!
//! - **Robots.txt compliance** - Respects site scraping policies
//! - **Rate limiting** - Prevents abuse and respects server limits
//! - **Content filtering** - Blocks malicious content and scripts
//! - **Size limits** - Prevents memory exhaustion
//! - **Timeout protection** - Prevents hanging operations
//! - **URL validation** - Blocks dangerous protocols and domains
//! - **Caching** - Reduces redundant requests
//!
//! # Profiles
//!
//! Web tools are available in:
//! - `local-power` - Full web access with user-defined restrictions
//! - `public-safe` - Restricted to safe, read-only operations
//!
//! # Example
//!
//! ```no_run
//! use pb_mcp_server::web::{WebClient, WebConfig};
//!
//! let config = WebConfig::safe_default();
//! let client = WebClient::new(config)?;
//!
//! // Fetch a webpage (respects robots.txt)
//! let content = client.fetch_text("https://example.com").await?;
//!
//! // Search for specific content
//! let results = client.search("rust programming").await?;
//! ```

pub mod cache;
pub mod client;
pub mod config;
pub mod extract;
pub mod fetch;
pub mod filter;
pub mod robots;
pub mod safety;
pub mod search;

pub use cache::WebCache;
pub use client::WebClient;
pub use config::WebConfig;
pub use fetch::FetchResult;
pub use filter::ContentFilter;
pub use robots::RobotsChecker;
pub use safety::{SafetyLevel, UrlValidator};
pub use search::SearchResult;

use crate::error::{Error, Result};
use std::time::Duration;

// ============================================================================
// Constants
// ============================================================================

/// Maximum response size (10 MB)
pub const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum response size for public-safe mode (1 MB)
pub const MAX_RESPONSE_SIZE_SAFE: usize = 1024 * 1024;

/// Default request timeout (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum redirects to follow
pub const MAX_REDIRECTS: usize = 10;

/// Default user agent
pub const DEFAULT_USER_AGENT: &str = concat!(
    "pb-mcp-server/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/PanosBiazis/pb-mcp-server)"
);

/// Default rate limit (requests per domain per minute)
pub const DEFAULT_RATE_LIMIT: u32 = 60;

/// Blocked domains (malware, phishing, etc.)
pub const BLOCKED_DOMAINS: &[&str] = &[
    "malware.com",
    "phishing.com",
    "spam.com",
    // Add more as needed
];

/// Allowed protocols
pub const ALLOWED_PROTOCOLS: &[&str] = &["http", "https"];

/// Blocked file extensions (potentially dangerous)
pub const BLOCKED_EXTENSIONS: &[&str] = &[
    ".exe", ".dll", ".so", ".bat", ".cmd", ".sh", ".ps1", ".vbs", ".js", ".jar", ".msi", ".deb",
    ".rpm",
];

// ============================================================================
// URL Validation
// ============================================================================

/// Validate a URL for safety
pub fn validate_url(url: &str, safety_level: SafetyLevel) -> Result<url::Url> {
    let parsed = url::Url::parse(url)
        .map_err(|e| Error::invalid_arguments("url", format!("Invalid URL: {}", e)))?;

    // Check protocol
    let scheme = parsed.scheme();
    if !ALLOWED_PROTOCOLS.contains(&scheme) {
        return Err(Error::security_violation(format!(
            "Protocol '{}' is not allowed. Use http or https.",
            scheme
        )));
    }

    // Check domain
    if let Some(domain) = parsed.host_str() {
        let domain_lower = domain.to_lowercase();

        // Check blocked domains
        for blocked in BLOCKED_DOMAINS {
            if domain_lower.contains(blocked) {
                return Err(Error::security_violation(format!(
                    "Domain '{}' is blocked for safety",
                    domain
                )));
            }
        }

        // Safety level checks
        match safety_level {
            SafetyLevel::Strict => {
                // In strict mode, only allow specific domains
                // This would be configured via allowlist
            }
            SafetyLevel::Moderate => {
                // Block known dangerous TLDs
                let dangerous_tlds = [".tk", ".ml", ".ga", ".cf", ".gq"];
                for tld in dangerous_tlds {
                    if domain_lower.ends_with(tld) {
                        return Err(Error::security_violation(format!(
                            "Domain with TLD '{}' requires additional verification",
                            tld
                        )));
                    }
                }
            }
            SafetyLevel::Permissive => {
                // Allow most domains
            }
        }
    }

    // Check for blocked extensions in path
    let path = parsed.path().to_lowercase();
    for ext in BLOCKED_EXTENSIONS {
        if path.ends_with(ext) {
            return Err(Error::security_violation(format!(
                "File extension '{}' is blocked for safety",
                ext
            )));
        }
    }

    // No credentials in URL
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(Error::security_violation(
            "URLs with credentials are not allowed for security",
        ));
    }

    Ok(parsed)
}

// ============================================================================
// Web Tools Catalog
// ============================================================================

/// Web tools available in the MCP server
pub enum WebTool {
    /// Fetch a URL and return text content
    WebFetchText,
    /// Fetch a URL and return structured data
    WebFetchJson,
    /// Extract links from a webpage
    WebExtractLinks,
    /// Extract metadata from a webpage
    WebExtractMetadata,
    /// Search the web using a search engine
    WebSearch,
    /// Check if a URL is accessible
    WebCheckUrl,
    /// Get HTTP headers for a URL
    WebHead,
    /// Download a file with size limits
    WebDownload,
    /// Scrape structured data from a webpage
    WebScrape,
    /// Check robots.txt for a domain
    WebCheckRobots,
    /// Get a screenshot of a webpage (requires screenshot-tools)
    WebScreenshot,
    /// Convert webpage to markdown
    WebToMarkdown,
    /// Extract text from webpage (ad-free)
    WebExtractArticle,
    /// Check if URL is safe
    WebUrlSafety,
}

impl WebTool {
    /// Get tool name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            WebTool::WebFetchText => "web.fetch_text",
            WebTool::WebFetchJson => "web.fetch_json",
            WebTool::WebExtractLinks => "web.extract_links",
            WebTool::WebExtractMetadata => "web.extract_metadata",
            WebTool::WebSearch => "web.search",
            WebTool::WebCheckUrl => "web.check_url",
            WebTool::WebHead => "web.head",
            WebTool::WebDownload => "web.download",
            WebTool::WebScrape => "web.scrape",
            WebTool::WebCheckRobots => "web.check_robots",
            WebTool::WebScreenshot => "web.screenshot",
            WebTool::WebToMarkdown => "web.to_markdown",
            WebTool::WebExtractArticle => "web.extract_article",
            WebTool::WebUrlSafety => "web.url_safety",
        }
    }

    /// Get tool description
    pub fn description(&self) -> &'static str {
        match self {
            WebTool::WebFetchText => "Fetch a URL and extract clean text content",
            WebTool::WebFetchJson => "Fetch a URL and parse as JSON",
            WebTool::WebExtractLinks => "Extract all links from a webpage",
            WebTool::WebExtractMetadata => {
                "Extract metadata (title, description, etc.) from a webpage"
            }
            WebTool::WebSearch => "Search the web using configured search engine",
            WebTool::WebCheckUrl => "Check if a URL is accessible and get basic info",
            WebTool::WebHead => "Get HTTP headers for a URL without fetching content",
            WebTool::WebDownload => "Download a file with size and type validation",
            WebTool::WebScrape => "Scrape structured data using CSS selectors",
            WebTool::WebCheckRobots => "Check robots.txt for a domain",
            WebTool::WebScreenshot => "Take a screenshot of a webpage",
            WebTool::WebToMarkdown => "Convert webpage content to Markdown",
            WebTool::WebExtractArticle => {
                "Extract main article content (ad-free, distraction-free)"
            }
            WebTool::WebUrlSafety => "Check if a URL is safe to access",
        }
    }

    /// Get required profile
    pub fn required_profile(&self) -> &'static str {
        match self {
            WebTool::WebFetchText
            | WebTool::WebFetchJson
            | WebTool::WebExtractLinks
            | WebTool::WebExtractMetadata
            | WebTool::WebCheckUrl
            | WebTool::WebHead
            | WebTool::WebCheckRobots
            | WebTool::WebUrlSafety => "public-safe",

            WebTool::WebSearch
            | WebTool::WebDownload
            | WebTool::WebScrape
            | WebTool::WebToMarkdown
            | WebTool::WebExtractArticle => "local-power",

            WebTool::WebScreenshot => "local-power",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        let url = "https://example.com/path";
        let result = validate_url(url, SafetyLevel::Permissive);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_url_invalid_protocol() {
        let url = "ftp://example.com/file";
        let result = validate_url(url, SafetyLevel::Permissive);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_validate_url_blocked_extension() {
        let url = "https://example.com/malware.exe";
        let result = validate_url(url, SafetyLevel::Permissive);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_validate_url_with_credentials() {
        let url = "https://user:pass@example.com/";
        let result = validate_url(url, SafetyLevel::Permissive);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_validate_url_blocked_domain() {
        let url = "https://malware.com/payload";
        let result = validate_url(url, SafetyLevel::Permissive);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_safety_level_strict() {
        let url = "https://example.tk/";
        let result = validate_url(url, SafetyLevel::Strict);
        // Would need allowlist configuration for strict mode
    }

    #[test]
    fn test_safety_level_moderate() {
        let url = "https://example.tk/";
        let result = validate_url(url, SafetyLevel::Moderate);
        assert!(result.is_err()); // .tk is blocked in moderate mode
    }

    #[test]
    fn test_web_tool_names() {
        assert_eq!(WebTool::WebFetchText.as_str(), "web.fetch_text");
        assert_eq!(WebTool::WebSearch.as_str(), "web.search");
        assert_eq!(WebTool::WebCheckRobots.as_str(), "web.check_robots");
    }

    #[test]
    fn test_web_tool_profiles() {
        // Public-safe tools
        assert_eq!(WebTool::WebFetchText.required_profile(), "public-safe");
        assert_eq!(WebTool::WebCheckUrl.required_profile(), "public-safe");

        // Local-power tools
        assert_eq!(WebTool::WebSearch.required_profile(), "local-power");
        assert_eq!(WebTool::WebDownload.required_profile(), "local-power");
    }
}
