# Web Tools Guide for pb-mcp-server

## Overview

The web tools module provides safe, efficient, and powerful web access capabilities for AI agents. All web operations follow security best practices to protect both the server and the targets being accessed.

## Table of Contents

- [Features](#features)
- [Security Model](#security-model)
- [Configuration](#configuration)
- [Available Tools](#available-tools)
- [Usage Examples](#usage-examples)
- [Best Practices](#best-practices)
- [Rate Limiting](#rate-limiting)
- [Caching](#caching)
- [Robots.txt Compliance](#robotstxt-compliance)
- [Troubleshooting](#troubleshooting)

---

## Features

### 🌐 Core Capabilities

| Feature | Description |
|---------|-------------|
| **HTTP Client** | Full-featured HTTP client with connection pooling |
| **Robots.txt** | Automatic compliance with site scraping policies |
| **Rate Limiting** | Per-domain rate limiting to prevent abuse |
| **Content Filtering** | Strip scripts, ads, and tracking codes |
| **URL Validation** | Comprehensive security checks on all URLs |
| **Caching** | Intelligent caching with stale-while-revalidate |
| **Size Limits** | Memory protection with configurable limits |
| **Timeout Protection** | Prevents hanging operations |

### 🛡️ Security Features

- **Protocol Restrictions**: Only HTTP/HTTPS allowed
- **Private IP Blocking**: Prevents SSRF attacks
- **Dangerous Extension Blocking**: .exe, .dll, .jar, etc.
- **IDN Homograph Detection**: Prevents phishing attacks
- **Credential Stripping**: No credentials in URLs
- **Redirect Loop Detection**: Maximum redirect limits
- **Content Size Limits**: Prevents memory exhaustion
- **XSS Protection**: JavaScript stripping from HTML

---

## Security Model

### Defense in Depth

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: URL Validation                                      │
│ - Protocol check (http/https only)                          │
│ - Private IP blocking                                        │
│ - Dangerous extension blocking                               │
│ - IDN homograph detection                                    │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Domain Policy                                       │
│ - Allowlist/blocklist checking                               │
│ - Dangerous TLD blocking (.tk, .ml, etc.)                   │
│ - Safety level enforcement                                   │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Robots.txt Compliance                               │
│ - Automatic robots.txt checking                              │
│ - Crawl-delay respect                                        │
│ - User-agent matching                                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Rate Limiting                                       │
│ - Per-domain request limits                                  │
│ - Burst protection                                           │
│ - Minimum delay enforcement                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 5: Content Filtering                                   │
│ - JavaScript stripping                                       │
│ - Tracking code removal                                      │
│ - Ad blocking                                                │
│ - Size limits                                                │
└─────────────────────────────────────────────────────────────┘
```

### Safety Levels

| Level | Use Case | Restrictions |
|-------|----------|--------------|
| **Strict** | Public-safe mode | Allowlist only, no IPs, aggressive filtering |
| **Moderate** | Default | Block dangerous TLDs, moderate filtering |
| **Permissive** | Local trusted | Minimal restrictions, higher limits |

---

## Configuration

### Environment Variables

```bash
# Safety level (strict, moderate, permissive)
export PB_WEB_SAFETY_LEVEL=moderate

# Request timeout (seconds)
export PB_WEB_TIMEOUT_SECS=30

# Maximum response size (bytes)
export PB_WEB_MAX_RESPONSE_SIZE=10485760

# Enable logging
export PB_WEB_ENABLE_LOGGING=true

# Proxy settings
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080

# Custom user agent
export PB_WEB_USER_AGENT="MyAgent/1.0"

# Search API key (if using search features)
export PB_WEB_SEARCH_API_KEY=your-api-key
```

### Programmatic Configuration

```rust
use pb_mcp_server::web::{WebClient, WebConfig};

// Safe configuration for public use
let config = WebConfig::safe_default();
let client = WebClient::new(config)?;

// Local trusted configuration
let config = WebConfig::local_trusted();
let client = WebClient::new(config)?;

// Custom configuration
let config = WebConfig {
    safety_level: SafetyLevel::Moderate,
    timeout_secs: 60,
    max_response_size: 20 * 1024 * 1024, // 20 MB
    rate_limit: RateLimitConfig {
        requests_per_minute: 120,
        min_delay_ms: 500,
        burst_limit: 20,
        burst_cooldown_ms: 3000,
    },
    ..Default::default()
};
```

### Configuration Files

```toml
# web-config.toml
safety_level = "moderate"
timeout_secs = 30
max_response_size = 10485760

[rate_limit]
requests_per_minute = 60
min_delay_ms = 1000
burst_limit = 10
burst_cooldown_ms = 5000

[cache]
enabled = true
max_size_bytes = 104857600  # 100 MB
ttl_secs = 3600

[content_filter]
strip_javascript = true
strip_iframes = true
remove_tracking = true
block_ads = true

[robots]
respect_robots_txt = true
respect_crawl_delay = true
```

---

## Available Tools

### Public-Safe Profile Tools

These tools are available in the `public-safe` profile with restrictions:

| Tool | Description | Safety |
|------|-------------|--------|
| `web.fetch_text` | Fetch URL and extract text | ✅ Safe |
| `web.fetch_json` | Fetch URL and parse JSON | ✅ Safe |
| `web.check_url` | Check if URL is accessible | ✅ Safe |
| `web.head` | Get HTTP headers only | ✅ Safe |
| `web.check_robots` | Check robots.txt for domain | ✅ Safe |
| `web.url_safety` | Check if URL is safe | ✅ Safe |
| `web.extract_metadata` | Extract page metadata | ✅ Safe |

### Local-Power Profile Tools

These tools require `local-power` profile:

| Tool | Description | Risk |
|------|-------------|------|
| `web.search` | Web search using search engine | ⚠️ Higher |
| `web.download` | Download files with validation | ⚠️ Higher |
| `web.scrape` | Scrape structured data with CSS | ⚠️ Higher |
| `web.to_markdown` | Convert webpage to Markdown | ⚠️ Medium |
| `web.extract_article` | Extract main article content | ⚠️ Medium |

---

## Usage Examples

### Basic Text Fetch

```rust
use pb_mcp_server::web::WebClient;

let client = WebClient::safe_default()?;

// Fetch and extract clean text
let text = client.fetch_html_text("https://example.com").await?;
println!("{}", text);

// Fetch raw HTML (filtered)
let html = client.fetch_html_filtered("https://example.com").await?;

// Fetch JSON API
#[derive(Deserialize)]
struct ApiResponse {
    data: String,
}
let json: ApiResponse = client.fetch_json("https://api.example.com/data").await?;
```

### URL Validation

```rust
use pb_mcp_server::web::{UrlValidator, SafetyLevel};

let validator = UrlValidator::new(SafetyLevel::Moderate);

// Valid URL
let url = validator.validate("https://example.com/page")?;
println!("Valid: {}", url);

// Blocked: Private IP
assert!(validator.validate("http://192.168.1.1/").is_err());

// Blocked: Dangerous extension
assert!(validator.validate("https://example.com/malware.exe").is_err());

// Blocked: Credentials in URL
assert!(validator.validate("https://user:pass@example.com/").is_err());
```

### Rate Limiting

```rust
use pb_mcp_server::web::{WebClient, WebConfig};
use pb_mcp_server::web::config::RateLimitConfig;

let config = WebConfig {
    rate_limit: RateLimitConfig {
        requests_per_minute: 60,  // 60 requests per minute per domain
        min_delay_ms: 1000,        // Minimum 1 second between requests
        burst_limit: 10,           // Allow 10 rapid requests
        burst_cooldown_ms: 5000,   // Then wait 5 seconds
    },
    ..Default::default()
};

let client = WebClient::new(config)?;

// Requests are automatically rate-limited
for i in 0..100 {
    let text = client.fetch_text("https://example.com/page").await?;
    // Automatically enforces rate limits
}
```

### Content Filtering

```rust
use pb_mcp_server::web::filter::ContentFilter;
use pb_mcp_server::web::config::ContentFilterConfig;

let config = ContentFilterConfig {
    strip_javascript: true,
    strip_iframes: true,
    remove_tracking: true,
    block_ads: true,
    ..Default::default()
};

let filter = ContentFilter::new(config);

// Filter HTML content
let clean_html = filter.filter_html(&raw_html)?;

// Extract text only
let text = filter.extract_text(&raw_html);

// Extract metadata
let metadata = filter.extract_metadata(&raw_html)?;
println!("Title: {}", metadata.title);
println!("Description: {:?}", metadata.description);
```

### Checking Robots.txt

```rust
use pb_mcp_server::web::robots::RobotsChecker;
use pb_mcp_server::web::config::RobotsConfig;
use url::Url;

let config = RobotsConfig {
    respect_robots_txt: true,
    respect_crawl_delay: true,
    ..Default::default()
};

let checker = RobotsChecker::new(config);

let url = Url::parse("https://example.com/admin")?;

// Check if URL is allowed
if checker.is_allowed(&url).await? {
    // Proceed with request
} else {
    println!("URL is disallowed by robots.txt");
}

// Get crawl delay
if let Some(delay) = checker.get_crawl_delay(&url).await? {
    println!("Must wait {:?} between requests", delay);
}
```

### Using Cache

```rust
use pb_mcp_server::web::cache::WebCache;
use pb_mcp_server::web::config::CacheConfig;

let config = CacheConfig {
    enabled: true,
    max_size_bytes: 100 * 1024 * 1024, // 100 MB
    ttl_secs: 3600,                     // 1 hour
    stale_while_revalidate_secs: 300,   // 5 minutes
    ..Default::default()
};

let cache = WebCache::new(config);

// Cache is used automatically by WebClient
let client = WebClient::new(WebConfig {
    cache: config,
    ..Default::default()
})?;

// First request: fetches from network
let text1 = client.fetch_text("https://example.com").await?;

// Second request: served from cache (if within TTL)
let text2 = client.fetch_text("https://example.com").await?;

// Clear cache
client.clear_cache().await;
```

---

## Best Practices

### 1. Always Validate URLs First

```rust
// ❌ Bad: Direct fetch without validation
let text = client.fetch_text(user_provided_url).await?;

// ✅ Good: Validate first
let url = client.validate_and_check(user_provided_url).await?;
let text = client.fetch_text(url.as_str()).await?;
```

### 2. Respect Rate Limits

```rust
// ❌ Bad: Aggressive polling
loop {
    let data = client.fetch_text(url).await?;
    // Process immediately
}

// ✅ Good: Respect delays
use tokio::time::{sleep, Duration};

loop {
    let data = client.fetch_text(url).await?;
    // Process data
    
    // Wait before next request
    sleep(Duration::from_secs(5)).await;
}
```

### 3. Handle Errors Gracefully

```rust
use pb_mcp_server::error::Error;

match client.fetch_text(url).await {
    Ok(text) => println!("Fetched: {}", text),
    Err(Error::RateLimitExceeded { operation, limit, .. }) => {
        eprintln!("Rate limited: {}. Limit: {}", operation, limit);
        // Wait and retry
    }
    Err(Error::SecurityViolation { reason }) => {
        eprintln!("Security violation: {}", reason);
        // Don't retry - URL is blocked
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### 4. Use Appropriate Safety Levels

```rust
// For public/external input
let config = WebConfig::safe_default();

// For trusted internal use
let config = WebConfig::local_trusted();

// For testing
let config = WebConfig::test_config();
```

### 5. Filter Content for Display

```rust
// ❌ Bad: Display raw HTML
let html = client.fetch_text(url).await?;
println!("{}", html);

// ✅ Good: Extract and sanitize
let text = client.fetch_html_text(url).await?;
println!("{}", text);

// ✅ Good: Filter for structure
let clean = client.fetch_html_filtered(url).await?;
// Process clean HTML
```

---

## Rate Limiting

### How It Works

Rate limiting is applied per-domain to prevent abuse:

1. **Requests per Minute**: Maximum requests in a 60-second window
2. **Minimum Delay**: Required time between requests to same domain
3. **Burst Limit**: Number of rapid requests allowed
4. **Burst Cooldown**: Wait time after burst before more requests

### Default Limits

| Setting | Safe | Default | Trusted |
|---------|------|---------|---------|
| Requests/min | 30 | 60 | 120 |
| Min delay (ms) | 2000 | 1000 | 500 |
| Burst limit | 5 | 10 | 20 |
| Burst cooldown (ms) | 10000 | 5000 | 3000 |

### Custom Rate Limits

```rust
let config = WebConfig {
    rate_limit: RateLimitConfig {
        requests_per_minute: 30,
        min_delay_ms: 2000,
        burst_limit: 5,
        burst_cooldown_ms: 10000,
    },
    ..Default::default()
};
```

---

## Caching

### Cache Strategy

The cache uses a **stale-while-revalidate** pattern:

1. First request: Fetch from network, cache result
2. Subsequent requests (within TTL): Return cached data
3. After TTL expires: Return stale data, fetch new in background
4. After stale window: Block until fresh data fetched

### Cache Configuration

```rust
let config = CacheConfig {
    enabled: true,
    max_size_bytes: 100 * 1024 * 1024, // 100 MB
    ttl_secs: 3600,                     // 1 hour fresh
    stale_while_revalidate_secs: 300,   // 5 min stale OK
    cache_success: true,
    cache_errors: true,
    error_ttl_secs: 60,                 // Cache errors briefly
};
```

### Cache Invalidation

```rust
// Clear all cache
client.clear_cache().await;

// Cache is automatically invalidated:
// - After TTL expires
// - When size limit exceeded (LRU eviction)
// - On server restart
```

---

## Robots.txt Compliance

### How It Works

1. Before accessing any URL, check robots.txt for the domain
2. Parse the robots.txt file
3. Check if the user-agent is allowed to access the path
4. Respect crawl-delay directives
5. Cache robots.txt for efficiency

### Checking Robots.txt

```rust
use pb_mcp_server::web::robots::RobotsChecker;

let checker = RobotsChecker::default();

let url = Url::parse("https://example.com/private/data")?;

// Check if allowed
match checker.is_allowed(&url).await {
    Ok(true) => println!("Access allowed"),
    Ok(false) => println!("Access denied by robots.txt"),
    Err(e) => println!("Error checking robots.txt: {}", e),
}

// Get required delay
if let Some(delay) = checker.get_crawl_delay(&url).await? {
    println!("Crawl delay: {:?}", delay);
}
```

### Disabling Robots.txt Check

```rust
// Not recommended for production
let config = WebConfig {
    robots: RobotsConfig {
        respect_robots_txt: false,
        ..Default::default()
    },
    ..Default::default()
};
```

---

## Troubleshooting

### Common Errors

#### "URL exceeds maximum length"

```rust
// Problem: URL too long (> 2048 characters by default)
let url = "https://example.com/very/long/path/...";

// Solution: Use shorter URLs or increase limit
let validator = UrlValidator {
    max_url_length: 4096,
    ..Default::default()
};
```

#### "Domain is blocked for safety"

```rust
// Problem: Domain in blocklist or not in allowlist (strict mode)

// Solution 1: Use moderate/permissive mode
let config = WebConfig {
    safety_level: SafetyLevel::Moderate,
    ..Default::default()
};

// Solution 2: Add to allowlist
let config = WebConfig {
    allowlist: vec!["trusted-domain.com".to_string()],
    ..Default::default()
};
```

#### "Rate limit exceeded"

```rust
// Problem: Too many requests to same domain

// Solution 1: Wait and retry
sleep(Duration::from_secs(5)).await;
let result = client.fetch_text(url).await;

// Solution 2: Increase rate limits
let config = WebConfig {
    rate_limit: RateLimitConfig {
        requests_per_minute: 120,
        ..Default::default()
    },
    ..Default::default()
};
```

#### "URL is disallowed by robots.txt"

```rust
// Problem: Path blocked by site's robots.txt

// Solution: Respect the policy (recommended)
// Or use a different URL/path

// Alternative: Disable robots.txt checking (not recommended)
let config = WebConfig {
    robots: RobotsConfig {
        respect_robots_txt: false,
        ..Default::default()
    },
    ..Default::default()
};
```

#### "Response too large"

```rust
// Problem: Response exceeds size limit

// Solution 1: Increase limit
let config = WebConfig {
    max_response_size: 50 * 1024 * 1024, // 50 MB
    ..Default::default()
};

// Solution 2: Use streaming (if available)
// (Check if large file, consider downloading separately)
```

#### "Dangerous URL scheme"

```rust
// Problem: Non-HTTP URL
let url = "ftp://example.com/file";
let result = client.fetch_text(url).await; // Error!

// Solution: Use HTTP/HTTPS only
let url = "https://example.com/file";
let result = client.fetch_text(url).await;
```

### Debug Logging

Enable debug logging for troubleshooting:

```bash
# Set log level
export RUST_LOG=pb_mcp_server::web=debug

# Run server
./target/release/pb-mcp-server
```

---

## Security Considerations

### SSRF Prevention

The web tools implement multiple layers of SSRF (Server-Side Request Forgery) prevention:

1. **Private IP Blocking**: Blocks 10.x.x.x, 192.168.x.x, 172.16-31.x.x, 127.x.x.x
2. **IPv6 Localhost**: Blocks ::1 and link-local addresses
3. **Cloud Metadata**: Blocks 169.254.169.254 (AWS/GCP metadata)
4. **URL Validation**: Prevents bypass via IDN, redirects, etc.

### XSS Prevention

All HTML content is filtered to prevent XSS:

1. **JavaScript Removal**: All `<script>` tags and event handlers removed
2. **Iframe Stripping**: Prevents embedded malicious content
3. **Form Removal**: Prevents CSRF via embedded forms
4. **Tracking Removal**: Removes tracking pixels and scripts

### Data Privacy

- **No Cookies**: Cookies are not persisted between requests
- **No Credential Storage**: Credentials are never stored
- **Cache Encryption**: Sensitive cache data can be encrypted
- **Logging Sanitization**: URLs are logged without query parameters by default

---

## Performance Optimization

### Connection Pooling

HTTP client uses connection pooling automatically:

- Reuses connections to same hosts
- Configurable pool size and timeout
- Automatic connection cleanup

### Parallel Requests

Use `tokio::join!` for parallel requests:

```rust
let (result1, result2, result3) = tokio::join!(
    client.fetch_text("https://api1.example.com"),
    client.fetch_text("https://api2.example.com"),
    client.fetch_text("https://api3.example.com"),
);
```

### Batch Processing

For multiple URLs:

```rust
let urls = vec![
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3",
];

let results = futures::future::join_all(
    urls.into_iter().map(|url| client.fetch_text(url))
).await;

for result in results {
    match result {
        Ok(text) => println!("Fetched {} chars", text.len()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## Conclusion

The web tools module provides a comprehensive, secure, and efficient way for AI agents to interact with web content. By following the security best practices and configuration options outlined in this guide, you can safely leverage web capabilities while protecting against common vulnerabilities.

For more information:
- [Main README](../README.md)
- [API Documentation](./api/)
- [Security Guide](./SECURITY.md)
- [Examples Repository](../examples/)

---

**Version:** 0.2.0  
**Last Updated:** 2025-01-09  
**Author:** pb-mcp-server Team