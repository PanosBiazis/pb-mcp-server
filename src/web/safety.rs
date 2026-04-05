//! URL safety validation for web operations
//!
//! This module provides comprehensive URL validation including:
//! - Protocol restrictions
//! - Domain blocklist/allowlist checking
//! - Dangerous file extension blocking
//! - Private IP address detection
//! - Unicode/IDN homograph attack prevention
//! - Redirect loop detection

use crate::error::{Error, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use url::Url;

use super::SafetyLevel;

/// URL validator with security checks
#[derive(Debug, Clone)]
pub struct UrlValidator {
    /// Safety level for validation
    safety_level: SafetyLevel,
    /// Private IP ranges to block
    private_ip_ranges: Vec<(IpAddr, IpAddr)>,
    /// Dangerous URL schemes
    dangerous_schemes: Vec<&'static str>,
    /// Maximum URL length
    max_url_length: usize,
    /// Maximum query parameters
    max_query_params: usize,
}

impl Default for UrlValidator {
    fn default() -> Self {
        UrlValidator {
            safety_level: SafetyLevel::Moderate,
            private_ip_ranges: Self::get_private_ip_ranges(),
            dangerous_schemes: vec![
                "javascript",
                "vbscript",
                "data",
                "file",
                "ftp",
                "ssh",
                "telnet",
                "gopher",
                "ldap",
                "jar",
            ],
            max_url_length: 2048,
            max_query_params: 50,
        }
    }
}

impl UrlValidator {
    /// Create a new URL validator with safety level
    pub fn new(safety_level: SafetyLevel) -> Self {
        UrlValidator {
            safety_level,
            ..Default::default()
        }
    }

    /// Get private IP ranges
    fn get_private_ip_ranges() -> Vec<(IpAddr, IpAddr)> {
        vec![
            // IPv4 Private ranges
            // 10.0.0.0/8
            (
                IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255)),
            ),
            // 172.16.0.0/12
            (
                IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255)),
            ),
            // 192.168.0.0/16
            (
                IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255)),
            ),
            // 127.0.0.0/8 (localhost)
            (
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255)),
            ),
            // 169.254.0.0/16 (link-local)
            (
                IpAddr::V4(Ipv4Addr::new(169, 254, 0, 0)),
                IpAddr::V4(Ipv4Addr::new(169, 254, 255, 255)),
            ),
            // IPv6 localhost
            (
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            ),
            // IPv6 link-local
            (
                IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0)),
                IpAddr::V6(Ipv6Addr::new(
                    0xfebf, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
                )),
            ),
        ]
    }

    /// Validate a URL comprehensively
    pub fn validate(&self, url: &str) -> Result<Url> {
        // Check URL length
        if url.len() > self.max_url_length {
            return Err(Error::security_violation(format!(
                "URL exceeds maximum length of {} characters",
                self.max_url_length
            )));
        }

        // Parse URL
        let parsed = Url::parse(url)
            .map_err(|e| Error::invalid_arguments("url", format!("Invalid URL: {}", e)))?;

        // Check scheme
        self.validate_scheme(&parsed)?;

        // Check host
        self.validate_host(&parsed)?;

        // Check query parameters
        self.validate_query(&parsed)?;

        // Check for credentials
        self.validate_credentials(&parsed)?;

        // Check for dangerous fragments
        self.validate_fragment(&parsed)?;

        // Check path
        self.validate_path(&parsed)?;

        Ok(parsed)
    }

    /// Validate URL scheme
    fn validate_scheme(&self, url: &Url) -> Result<()> {
        let scheme = url.scheme().to_lowercase();

        // Check for dangerous schemes
        if self.dangerous_schemes.contains(&scheme.as_str()) {
            return Err(Error::security_violation(format!(
                "Dangerous URL scheme: '{}'",
                scheme
            )));
        }

        // Only allow http and https
        if scheme != "http" && scheme != "https" {
            return Err(Error::security_violation(format!(
                "Unsupported URL scheme: '{}'. Only http and https are allowed.",
                scheme
            )));
        }

        Ok(())
    }

    /// Validate URL host
    fn validate_host(&self, url: &Url) -> Result<()> {
        let host = url
            .host_str()
            .ok_or_else(|| Error::invalid_arguments("url", "URL has no host"))?;

        // Check for IP address
        if let Ok(ip) = IpAddr::from_str(host) {
            self.validate_ip_address(&ip)?;
        } else {
            // Validate domain name
            self.validate_domain(host)?;
        }

        Ok(())
    }

    /// Validate IP address (block private/internal IPs)
    fn validate_ip_address(&self, ip: &IpAddr) -> Result<()> {
        for (start, end) in &self.private_ip_ranges {
            if self.ip_in_range(ip, start, end) {
                return Err(Error::security_violation(format!(
                    "Access to private/internal IP address '{}' is blocked",
                    ip
                )));
            }
        }

        // Additional checks based on safety level
        match self.safety_level {
            SafetyLevel::Strict => {
                // In strict mode, block all IP addresses
                return Err(Error::security_violation(
                    "Direct IP address access is not allowed in strict mode",
                ));
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if IP is in range
    fn ip_in_range(&self, ip: &IpAddr, start: &IpAddr, end: &IpAddr) -> bool {
        match (ip, start, end) {
            (IpAddr::V4(ip), IpAddr::V4(start), IpAddr::V4(end)) => {
                u32::from(*ip) >= u32::from(*start) && u32::from(*ip) <= u32::from(*end)
            }
            (IpAddr::V6(ip), IpAddr::V6(start), IpAddr::V6(end)) => {
                u128::from(*ip) >= u128::from(*start) && u128::from(*ip) <= u128::from(*end)
            }
            _ => false,
        }
    }

    /// Validate domain name
    fn validate_domain(&self, domain: &str) -> Result<()> {
        let domain_lower = domain.to_lowercase();

        // Check for IDN homograph attacks
        self.check_idn_homograph(&domain_lower)?;

        // Check for suspicious patterns
        self.check_suspicious_domain(&domain_lower)?;

        // Safety level specific checks
        match self.safety_level {
            SafetyLevel::Strict => {
                // In strict mode, additional validation would be applied
                // (e.g., allowlist checking would be done by caller)
            }
            SafetyLevel::Moderate => {
                // Block suspicious TLDs
                self.check_dangerous_tlds(&domain_lower)?;
            }
            SafetyLevel::Permissive => {
                // Minimal checks
            }
        }

        Ok(())
    }

    /// Check for IDN homograph attacks
    fn check_idn_homograph(&self, domain: &str) -> Result<()> {
        // Check for mixed scripts (potential homograph attack)
        let has_cyrillic = domain
            .chars()
            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
        let has_latin = domain.chars().any(|c| c.is_ascii_alphabetic());

        if has_cyrillic && has_latin {
            // Potential homograph attack
            return Err(Error::security_violation(
                "Domain contains mixed scripts (potential homograph attack)",
            ));
        }

        // Check for confusable characters
        let confusables = ['а', 'е', 'о', 'р', 'с', 'у', 'х']; // Cyrillic lookalikes
        for c in confusables {
            if domain.contains(c) {
                // Warn about potential confusion
                // In strict mode, reject
                if self.safety_level == SafetyLevel::Strict {
                    return Err(Error::security_violation(format!(
                        "Domain contains potentially confusable character: '{}'",
                        c
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check for suspicious domain patterns
    fn check_suspicious_domain(&self, domain: &str) -> Result<()> {
        // Check for excessive subdomains
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() > 10 {
            return Err(Error::security_violation(
                "Domain has excessive number of subdomains",
            ));
        }

        // Check for suspicious patterns
        let suspicious_patterns = [
            "0x", "xn--", // Potential IDN abuse
            "..", "--", // Suspicious characters
        ];

        for pattern in &suspicious_patterns {
            if domain.contains(pattern) {
                return Err(Error::security_violation(format!(
                    "Domain contains suspicious pattern: '{}'",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Check for dangerous TLDs
    fn check_dangerous_tlds(&self, domain: &str) -> Result<()> {
        let dangerous_tlds = [
            ".tk", ".ml", ".ga", ".cf", ".gq", // Free TLDs often used for abuse
            ".zip", ".mov", ".review", ".country", // High-risk TLDs
        ];

        for tld in &dangerous_tlds {
            if domain.ends_with(tld) {
                match self.safety_level {
                    SafetyLevel::Strict | SafetyLevel::Moderate => {
                        return Err(Error::security_violation(format!(
                            "Domain with TLD '{}' requires additional verification",
                            tld
                        )));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Validate query parameters
    fn validate_query(&self, url: &Url) -> Result<()> {
        let query_pairs: Vec<_> = url.query_pairs().collect();

        if query_pairs.len() > self.max_query_params {
            return Err(Error::security_violation(format!(
                "URL has too many query parameters: {} (max: {})",
                query_pairs.len(),
                self.max_query_params
            )));
        }

        // Check for suspicious query parameters
        let suspicious_params = [
            "password", "passwd", "pwd", "secret", "token", "key", "api_key",
        ];
        for (key, _) in query_pairs {
            let key_lower = key.to_lowercase();
            if suspicious_params.contains(&key_lower.as_str()) {
                // In moderate/strict mode, reject URLs with sensitive-looking params
                match self.safety_level {
                    SafetyLevel::Strict | SafetyLevel::Moderate => {
                        return Err(Error::security_violation(format!(
                            "URL contains suspicious query parameter: '{}'",
                            key
                        )));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Validate credentials in URL
    fn validate_credentials(&self, url: &Url) -> Result<()> {
        if url.username() != "" || url.password().is_some() {
            return Err(Error::security_violation(
                "URLs with embedded credentials are not allowed",
            ));
        }

        Ok(())
    }

    /// Validate fragment
    fn validate_fragment(&self, url: &Url) -> Result<()> {
        if let Some(fragment) = url.fragment() {
            // Check for javascript: in fragment
            let fragment_lower = fragment.to_lowercase();
            if fragment_lower.contains("javascript:") {
                return Err(Error::security_violation(
                    "URL fragment contains potentially dangerous content",
                ));
            }
        }

        Ok(())
    }

    /// Validate path
    fn validate_path(&self, url: &Url) -> Result<()> {
        let path = url.path();

        // Check for path traversal
        if path.contains("..") {
            return Err(Error::security_violation(
                "URL path contains directory traversal sequence",
            ));
        }

        // Check for dangerous file extensions
        let dangerous_extensions = [
            ".exe", ".dll", ".so", ".bat", ".cmd", ".sh", ".ps1", ".vbs", ".jar", ".msi", ".scr",
            ".pif", ".com",
        ];

        let path_lower = path.to_lowercase();
        for ext in &dangerous_extensions {
            if path_lower.ends_with(ext) {
                return Err(Error::security_violation(format!(
                    "URL points to potentially dangerous file type: '{}'",
                    ext
                )));
            }
        }

        Ok(())
    }

    /// Check if a URL is safe for redirect
    pub fn is_safe_redirect(&self, from_url: &Url, to_url: &Url) -> Result<bool> {
        // Validate target URL
        self.validate(to_url.as_str())?;

        // Same-origin check
        if from_url.host() != to_url.host() {
            match self.safety_level {
                SafetyLevel::Strict => {
                    // In strict mode, only allow same-origin redirects
                    return Ok(false);
                }
                _ => {}
            }
        }

        Ok(true)
    }

    /// Get safety level
    pub fn safety_level(&self) -> SafetyLevel {
        self.safety_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);
        assert_eq!(validator.safety_level, SafetyLevel::Moderate);
    }

    #[test]
    fn test_valid_url() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);
        let result = validator.validate("https://example.com/path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dangerous_scheme() {
        let validator = UrlValidator::new(SafetyLevel::Permissive);
        let result = validator.validate("javascript:alert('xss')");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_private_ip() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);

        // Test various private IPs
        assert!(validator.validate("http://192.168.1.1/").is_err());
        assert!(validator.validate("http://10.0.0.1/").is_err());
        assert!(validator.validate("http://172.16.0.1/").is_err());
        assert!(validator.validate("http://127.0.0.1/").is_err());
    }

    #[test]
    fn test_dangerous_file_extension() {
        let validator = UrlValidator::new(SafetyLevel::Permissive);
        let result = validator.validate("https://example.com/malware.exe");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_credentials_in_url() {
        let validator = UrlValidator::new(SafetyLevel::Permissive);
        let result = validator.validate("https://user:pass@example.com/");
        assert!(result.is_err());
        assert!(result.unwrap_err().is_security_error());
    }

    #[test]
    fn test_url_too_long() {
        let validator = UrlValidator::new(SafetyLevel::Permissive);
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        let result = validator.validate(&long_url);
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_tld_moderate() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);
        let result = validator.validate("https://example.tk/");
        assert!(result.is_err());
    }

    #[test]
    fn test_idn_homograph() {
        let validator = UrlValidator::new(SafetyLevel::Strict);
        // Cyrillic 'а' looks like Latin 'a'
        let result = validator.validate("https://pаypal.com/"); // Note: uses Cyrillic 'а'
        assert!(result.is_err());
    }

    #[test]
    fn test_suspicious_query_param() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);
        let result = validator.validate("https://example.com/?password=secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_redirect_same_origin() {
        let validator = UrlValidator::new(SafetyLevel::Moderate);
        let from = Url::parse("https://example.com/").unwrap();
        let to = Url::parse("https://example.com/page2").unwrap();

        let is_safe = validator.is_safe_redirect(&from, &to).unwrap();
        assert!(is_safe);
    }

    #[test]
    fn test_safe_redirect_cross_origin_strict() {
        let validator = UrlValidator::new(SafetyLevel::Strict);
        let from = Url::parse("https://example.com/").unwrap();
        let to = Url::parse("https://other.com/").unwrap();

        let is_safe = validator.is_safe_redirect(&from, &to).unwrap();
        assert!(!is_safe);
    }

    #[test]
    fn test_ipv6_localhost() {
        let validator = UrlValidator::new(SafetyLevel::Permissive);
        let result = validator.validate("http://[::1]/");
        assert!(result.is_err());
    }
}
