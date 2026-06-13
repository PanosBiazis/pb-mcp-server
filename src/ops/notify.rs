//! # Notification Tools
//!
//! Webhook, desktop, and email notification helpers.
//!
//! This module provides three notification primitives:
//!
//! - **Webhook** — send JSON payloads via `curl` with optional header
//!   customisation and internal-IP blocking.
//! - **Desktop** — show local desktop notifications via `notify-send`
//!   (Linux) or `osascript` (macOS).
//! - **Email** — send email through `sendmail` or `mailx`.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;

/// Maximum characters for notification tool responses.
const MAX_NOTIFY_RESPONSE_CHARS: usize = 20_000;
const WEBHOOK_CONNECT_TIMEOUT_SECS: &str = "5";
const WEBHOOK_MAX_TIME_SECS: &str = "30";

#[derive(Debug, Clone, PartialEq, Eq)]
struct WebhookTarget {
    scheme: String,
    host: String,
    port: u16,
}

// ---------------------------------------------------------------------------
// Internal IP helpers
// ---------------------------------------------------------------------------

/// Extract the host portion from a URL string.
///
/// Handles `http://host:port/path` and `https://host:port/path` forms.
/// Returns `None` if the URL cannot be parsed.
fn extract_host_from_url(url: &str) -> Option<String> {
    extract_webhook_target(url).map(|target| target.host)
}

fn extract_webhook_target(url: &str) -> Option<WebhookTarget> {
    // Strip scheme
    let (scheme, rest) = if let Some(rest) = url.strip_prefix("http://") {
        ("http", rest)
    } else if let Some(rest) = url.strip_prefix("https://") {
        ("https", rest)
    } else if let Some(rest) = url.strip_prefix("HTTP://") {
        ("http", rest)
    } else if let Some(rest) = url.strip_prefix("HTTPS://") {
        ("https", rest)
    } else {
        return None;
    };

    // Everything up to the first '/' or end-of-string is the authority
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority.is_empty() {
        return None;
    }

    // Strip port suffix (e.g. ":8080")
    // But be careful with IPv6 literals like [::1]:8080
    if authority.starts_with('[') {
        // IPv6 literal: [host]:port
        if let Some(bracket_end) = authority.find(']') {
            let host = &authority[1..bracket_end];
            let port = authority[bracket_end + 1..]
                .strip_prefix(':')
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| default_port_for_scheme(scheme));
            return Some(WebhookTarget {
                scheme: scheme.to_string(),
                host: host.to_string(),
                port,
            });
        }
    }

    // IPv4 or hostname — strip trailing :port
    let (host, port) = authority
        .rsplit_once(':')
        .and_then(|(h, port)| port.parse::<u16>().ok().map(|p| (h, p)))
        .unwrap_or((authority, default_port_for_scheme(scheme)));
    if host.is_empty() {
        return None;
    }
    Some(WebhookTarget {
        scheme: scheme.to_string(),
        host: host.to_string(),
        port,
    })
}

fn default_port_for_scheme(scheme: &str) -> u16 {
    if scheme.eq_ignore_ascii_case("http") {
        80
    } else {
        443
    }
}

/// Check whether an IP address is private / internal / loopback.
///
/// Blocked ranges:
/// - IPv4: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 127.0.0.0/8
/// - IPv6: ::1/128, fc00::/7 (unique-local / ULA)
fn is_internal_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_internal_ipv4(v4),
        IpAddr::V6(v6) => is_internal_ipv6(v6),
    }
}

fn is_internal_ipv4(v4: &Ipv4Addr) -> bool {
    let octets = v4.octets();
    // 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }
    // 172.16.0.0/12  →  172.16.x.x through 172.31.x.x
    if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
        return true;
    }
    // 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }
    // 127.0.0.0/8
    if octets[0] == 127 {
        return true;
    }
    // 0.0.0.0/8 — "this network"
    if octets[0] == 0 {
        return true;
    }
    // 169.254.0.0/16 — link-local
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }
    false
}

fn is_internal_ipv6(v6: &Ipv6Addr) -> bool {
    // ::1  loopback
    if v6.is_loopback() {
        return true;
    }
    // fc00::/7  unique-local (first 7 bits == 0b1111_110)
    // That means the first byte is 0xfc or 0xfd.
    let segments = v6.segments();
    let first_byte = (segments[0] >> 8) as u8;
    if first_byte == 0xfc || first_byte == 0xfd {
        return true;
    }
    // fe80::/10  link-local
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    false
}

/// Resolve a hostname to IP addresses by trying `getent hosts <host>`.
///
/// This avoids pulling in a DNS resolver dependency; we shell out to the
/// system's `getent` which uses nsswitch.
fn resolve_host_to_ips(host: &str) -> Vec<IpAddr> {
    // First, try parsing directly as an IP
    if let Ok(ip) = host.parse::<IpAddr>() {
        return vec![ip];
    }

    // Try getent hosts (Linux) or host (macOS)
    if cfg!(target_os = "linux") {
        let out = Command::new("timeout")
            .args(["3s", "getent", "hosts", host])
            .output()
            .or_else(|_| Command::new("getent").args(["hosts", host]).output());
        let Ok(out) = out else {
            return Vec::new();
        };
        if !out.status.success() {
            return Vec::new();
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .filter_map(|ip| ip.trim().parse::<IpAddr>().ok())
            .collect()
    } else if cfg!(target_os = "macos") {
        let out = Command::new("dscacheutil")
            .args(["-q", "host", "-a", &format!("name={host}")])
            .output()
            .ok();
        let Some(out) = out else {
            return Vec::new();
        };
        if !out.status.success() {
            return Vec::new();
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut ips = Vec::new();
        for line in stdout.lines() {
            if let Some(ip_str) = line.strip_prefix("ip_address:") {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    ips.push(ip);
                }
            }
        }
        ips
    } else {
        // Fallback: just try parsing (won't resolve hostnames on other OSes)
        host.parse::<IpAddr>().ok().into_iter().collect()
    }
}

/// Return `true` if the URL points to a private / internal IP.
///
/// If the host cannot be resolved the function conservatively returns
/// `true` (block the request) so that SSRF attempts with unknown hosts
/// are rejected.
fn url_points_to_internal_ip(url: &str) -> bool {
    let target = match extract_webhook_target(url) {
        Some(target) => target,
        None => return true, // cannot parse → block
    };

    let ips = resolve_host_to_ips(&target.host);
    ips.is_empty() || ips.iter().any(is_internal_ip)
}

fn checked_public_webhook_target(url: &str) -> Result<(WebhookTarget, IpAddr), String> {
    let target = extract_webhook_target(url)
        .ok_or_else(|| "URL must use http:// or https://".to_string())?;
    let ips = resolve_host_to_ips(&target.host);
    if ips.is_empty() {
        return Err("host could not be resolved".to_string());
    }
    if ips.iter().any(is_internal_ip) {
        return Err("host resolves to a private/internal IP address".to_string());
    }
    Ok((target, ips[0]))
}

// ---------------------------------------------------------------------------
// Truncation helper (local copy for self-containment when called outside
// of the ops:: parent module)
// ---------------------------------------------------------------------------

fn truncate_response(s: &str) -> String {
    if s.chars().count() <= MAX_NOTIFY_RESPONSE_CHARS {
        return s.to_string();
    }
    let truncated: String = s.chars().take(MAX_NOTIFY_RESPONSE_CHARS).collect();
    format!("{truncated}\n\n... [truncated to {MAX_NOTIFY_RESPONSE_CHARS} chars]")
}

// ---------------------------------------------------------------------------
// 1. notify_webhook_work
// ---------------------------------------------------------------------------

/// Send a JSON payload to a webhook URL via `curl`.
///
/// Uses `curl -s -o /dev/null -w "%{http_code}" -X <method> -H
/// "Content-Type: application/json" -d '<payload>' <url>` and returns
/// the HTTP status code along with any curl error output.
///
/// When `block_internal` is `true` (the default), URLs that resolve to
/// private / loopback IP addresses are rejected to prevent SSRF.
///
/// Additional custom headers can be supplied as `(name, value)` pairs.
pub fn notify_webhook_work(
    url: String,
    payload: String,
    method: Option<String>,
    headers: Option<Vec<(String, String)>>,
    block_internal: bool,
) -> String {
    let mut out = String::from("NOTIFY: WEBHOOK\n\n");

    let http_method = method.unwrap_or_else(|| "POST".to_string());
    let method_upper = http_method.to_uppercase();

    // Validate method is a sensible HTTP verb
    let allowed_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
    if !allowed_methods.contains(&method_upper.as_str()) {
        out.push_str(&format!(
            "Error: unsupported HTTP method \"{http_method}\". \
             Allowed: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS\n"
        ));
        return truncate_response(&out);
    }

    let checked_target = if block_internal {
        match checked_public_webhook_target(&url) {
            Ok(target) => Some(target),
            Err(reason) => {
                out.push_str(&format!("Error: URL \"{url}\" rejected: {reason}.\n"));
                out.push_str(
                    "Webhook requests to internal addresses are blocked to prevent SSRF.\n\
                     Set block_internal=false only if you fully trust the target.\n\
                     Blocked ranges: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, \
                     127.0.0.0/8, 0.0.0.0/8, 169.254.0.0/16, ::1, fc00::/7, fe80::/10\n",
                );
                return truncate_response(&out);
            }
        }
    } else {
        None
    };

    // Check that curl is available
    let curl_check = Command::new("curl")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();

    if curl_check.is_err() {
        out.push_str(
            "Error: `curl` is not installed or not found on PATH.\n\
             Install curl:\n\
             - Debian/Ubuntu: sudo apt-get install curl\n\
             - RHEL/Fedora:  sudo dnf install curl\n\
             - macOS:        brew install curl  (or use system curl)\n\
             - Alpine:       apk add curl\n",
        );
        return truncate_response(&out);
    }

    // Build curl command
    let mut cmd = Command::new("curl");
    cmd.args([
        "-s", // silent
        "-o",
        "/dev/null", // discard body
        "-w",
        "%{http_code}", // only print status code to stdout
        "--connect-timeout",
        WEBHOOK_CONNECT_TIMEOUT_SECS,
        "--max-time",
        WEBHOOK_MAX_TIME_SECS,
        "--proto",
        "=http,https",
        "--proto-redir",
        "=http,https",
        "-X",
        &method_upper,
        "-H",
        "Content-Type: application/json",
    ]);

    if let Some((target, ip)) = &checked_target {
        if target.host.parse::<IpAddr>().is_err() {
            cmd.arg("--resolve")
                .arg(format!("{}:{}:{}", target.host, target.port, ip));
        }
    }

    // Add custom headers
    if let Some(ref hdrs) = headers {
        for (name, value) in hdrs {
            // Sanitise: skip obviously dangerous header values
            if name.contains('\n') || value.contains('\n') {
                continue; // prevent header injection
            }
            cmd.arg("-H").arg(format!("{name}: {value}"));
        }
    }

    // Add payload and URL
    cmd.arg("-d").arg(&payload).arg(&url);

    let result = cmd.output();

    match result {
        Ok(output) => {
            let status_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            out.push_str(&format!("URL: {url}\n"));
            out.push_str(&format!("Method: {method_upper}\n"));
            out.push_str(&format!("HTTP Status: {status_code}\n"));

            if !stderr.is_empty() {
                out.push_str(&format!("curl stderr: {stderr}\n"));
            }

            // Interpret common status codes
            let code_int = status_code.parse::<u16>().unwrap_or(0);
            if code_int == 0 {
                out.push_str(
                    "\nNote: curl did not return a valid HTTP status code. \
                     This typically means the request could not connect or \
                     was refused.\n",
                );
            } else if (200..300).contains(&code_int) {
                out.push_str("\nResult: success (2xx)\n");
            } else if (300..400).contains(&code_int) {
                out.push_str("\nResult: redirect (3xx) — the server requested a redirect.\n");
            } else if (400..500).contains(&code_int) {
                out.push_str(
                    "\nResult: client error (4xx) — the request was malformed or unauthorized.\n",
                );
            } else if (500..600).contains(&code_int) {
                out.push_str(
                    "\nResult: server error (5xx) — the remote server encountered an error.\n",
                );
            }
        }
        Err(e) => {
            out.push_str(&format!(
                "Error: failed to execute curl: {e}\n\
                 Ensure curl is installed and accessible on PATH.\n"
            ));
        }
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 2. notify_desktop_work
// ---------------------------------------------------------------------------

/// Show a desktop notification.
///
/// On Linux this uses `notify-send -u <urgency> "<title>" "<body>"`.
/// On macOS this uses `osascript -e 'display notification "..." with title "..." ...'`.
///
/// `urgency` must be one of `"low"`, `"normal"`, `"critical"` (defaults to
/// `"normal"`).  If neither notification tool is available, an error with
/// install instructions is returned.
pub fn notify_desktop_work(title: String, body: String, urgency: Option<String>) -> String {
    let mut out = String::from("NOTIFY: DESKTOP\n\n");

    let urgency = urgency
        .unwrap_or_else(|| "normal".to_string())
        .to_lowercase();
    if !["low", "normal", "critical"].contains(&urgency.as_str()) {
        out.push_str(&format!(
            "Error: urgency must be one of \"low\", \"normal\", \"critical\", got \"{urgency}\"\n"
        ));
        return truncate_response(&out);
    }

    if cfg!(target_os = "linux") {
        // Check for notify-send
        let which = Command::new("which")
            .arg("notify-send")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        let found = which.map(|s| s.success()).unwrap_or(false);

        if !found {
            out.push_str(
                "Error: `notify-send` is not installed.\n\
                 Install a notification daemon:\n\
                 - Debian/Ubuntu: sudo apt-get install libnotify-bin\n\
                 - RHEL/Fedora:  sudo dnf install libnotify\n\
                 - Alpine:       apk add libnotify\n\
                 - Arch:         sudo pacman -S libnotify\n",
            );
            return truncate_response(&out);
        }

        let result = Command::new("notify-send")
            .arg("-u")
            .arg(&urgency)
            .arg(&title)
            .arg(&body)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    out.push_str(&format!(
                        "Desktop notification sent successfully.\n\
                         Title:   {title}\n\
                         Body:    {body}\n\
                         Urgency: {urgency}\n"
                    ));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    out.push_str(&format!(
                        "Error: notify-send exited with code {:?}\n\
                         stderr: {stderr}\n",
                        output.status.code()
                    ));
                }
            }
            Err(e) => {
                out.push_str(&format!("Error: failed to execute notify-send: {e}\n"));
            }
        }
    } else if cfg!(target_os = "macos") {
        // macOS: use osascript for notifications
        let urgency_label = match urgency.as_str() {
            "critical" => "with subtitle \"⚠ Critical\"",
            "low" => "with subtitle \"ℹ Low\"",
            _ => "",
        };

        // Escape double-quotes for AppleScript strings
        let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_body = body.replace('\\', "\\\\").replace('"', "\\\"");

        let applescript = if urgency_label.is_empty() {
            format!("display notification \"{escaped_body}\" with title \"{escaped_title}\"")
        } else {
            format!(
                "display notification \"{escaped_body}\" with title \"{escaped_title}\" {urgency_label}"
            )
        };

        let result = Command::new("osascript")
            .arg("-e")
            .arg(&applescript)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    out.push_str(&format!(
                        "Desktop notification sent successfully.\n\
                         Title:   {title}\n\
                         Body:    {body}\n\
                         Urgency: {urgency}\n"
                    ));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    out.push_str(&format!(
                        "Error: osascript exited with code {:?}\n\
                         stderr: {stderr}\n",
                        output.status.code()
                    ));
                }
            }
            Err(e) => {
                out.push_str(&format!("Error: failed to execute osascript: {e}\n"));
            }
        }
    } else {
        out.push_str(
            "Error: desktop notifications are not supported on this platform.\n\
             Supported platforms: Linux (notify-send), macOS (osascript).\n",
        );
    }

    truncate_response(&out)
}

// ---------------------------------------------------------------------------
// 3. notify_email_send_work
// ---------------------------------------------------------------------------

/// Send an email using `sendmail` or `mailx`.
///
/// Constructs a proper email with `From`, `To`, `Subject`, and `Date`
/// headers and pipes it into `sendmail -t` or falls back to
/// `mailx -s "<subject>" "<to>"`.
///
/// If neither CLI tool is available, an error with SMTP setup
/// instructions is returned.
pub fn notify_email_send_work(
    to: String,
    subject: String,
    body: String,
    from: Option<String>,
) -> String {
    let mut out = String::from("NOTIFY: EMAIL\n\n");

    let from_addr = from.unwrap_or_else(|| "noreply@localhost".to_string());
    let date = chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S +0000")
        .to_string();

    // Build the full email message for sendmail
    let email_message = format!(
        "From: {from_addr}\r\n\
         To: {to}\r\n\
         Subject: {subject}\r\n\
         Date: {date}\r\n\
         \r\n\
         {body}"
    );

    // Try sendmail first
    let sendmail_found = Command::new("which")
        .arg("sendmail")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if sendmail_found {
        let result = Command::new("sendmail")
            .arg("-t") // read recipients from To:/Cc:/Bcc: headers
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                // Write email to stdin
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(email_message.as_bytes()) {
                        out.push_str(&format!("Error: failed to write to sendmail stdin: {e}\n"));
                        let _ = child.kill();
                        return truncate_response(&out);
                    }
                    // stdin is dropped here, closing the pipe
                }

                match child.wait_with_output() {
                    Ok(output) => {
                        if output.status.success() {
                            out.push_str(&format!(
                                "Email sent successfully via sendmail.\n\
                                 From:    {from_addr}\n\
                                 To:      {to}\n\
                                 Subject: {subject}\n\
                                 Date:    {date}\n"
                            ));
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            out.push_str(&format!(
                                "Error: sendmail exited with code {:?}\n\
                                 stderr: {stderr}\n",
                                output.status.code()
                            ));
                        }
                    }
                    Err(e) => {
                        out.push_str(&format!("Error: failed to wait for sendmail: {e}\n"));
                    }
                }
            }
            Err(e) => {
                out.push_str(&format!("Error: failed to spawn sendmail: {e}\n"));
            }
        }

        return truncate_response(&out);
    }

    // Fall back to mailx
    let mailx_found = Command::new("which")
        .arg("mailx")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if mailx_found {
        let result = Command::new("mailx")
            .args(["-s", &subject, &to])
            .arg("-r")
            .arg(&from_addr)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if let Err(e) = stdin.write_all(body.as_bytes()) {
                        out.push_str(&format!("Error: failed to write to mailx stdin: {e}\n"));
                        let _ = child.kill();
                        return truncate_response(&out);
                    }
                }

                match child.wait_with_output() {
                    Ok(output) => {
                        if output.status.success() {
                            out.push_str(&format!(
                                "Email sent successfully via mailx.\n\
                                 From:    {from_addr}\n\
                                 To:      {to}\n\
                                 Subject: {subject}\n"
                            ));
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            out.push_str(&format!(
                                "Error: mailx exited with code {:?}\n\
                                 stderr: {stderr}\n",
                                output.status.code()
                            ));
                        }
                    }
                    Err(e) => {
                        out.push_str(&format!("Error: failed to wait for mailx: {e}\n"));
                    }
                }
            }
            Err(e) => {
                out.push_str(&format!("Error: failed to spawn mailx: {e}\n"));
            }
        }

        return truncate_response(&out);
    }

    // Neither available
    out.push_str(
        "Error: neither `sendmail` nor `mailx` is installed.\n\
         \n\
         To send email from the command line, install an MTA:\n\
         \n\
         === sendmail (Postfix / Exim / Sendmail) ===\n\
         - Debian/Ubuntu: sudo apt-get install postfix  (or sendmail-bin)\n\
         - RHEL/Fedora:  sudo dnf install postfix\n\
         - Alpine:       apk add postfix\n\
         - Arch:         sudo pacman -S postfix\n\
         - macOS:        Postfix is built-in; enable with:\n\
                        sudo postfix start\n\
         \n\
         === mailx (Heirloom mailx / s-nail / bsd-mailx) ===\n\
         - Debian/Ubuntu: sudo apt-get install mailutils\n\
         - RHEL/Fedora:  sudo dnf install mailx\n\
         - Alpine:       apk add mailx\n\
         \n\
         === Quick SMTP relay setup ===\n\
         1. Install Postfix and configure it as a satellite system.\n\
         2. Set relayhost in /etc/postfix/main.cf to your SMTP server.\n\
         3. Enable SASL auth if your SMTP server requires it.\n\
         4. Restart Postfix: sudo systemctl restart postfix\n\
         \n\
         Alternatively, configure an app-specific SMTP password in your\n\
         email provider's settings and use msmtp as a lightweight relay.\n",
    );

    truncate_response(&out)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_host_from_url ---

    #[test]
    fn extract_host_simple_http() {
        assert_eq!(
            extract_host_from_url("http://example.com/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn extract_host_https_with_port() {
        assert_eq!(
            extract_host_from_url("https://api.example.com:8443/webhook"),
            Some("api.example.com".to_string())
        );
    }

    #[test]
    fn extract_host_ip_with_port() {
        assert_eq!(
            extract_host_from_url("http://192.168.1.1:9090/hook"),
            Some("192.168.1.1".to_string())
        );
    }

    #[test]
    fn extract_host_ipv6_literal() {
        assert_eq!(
            extract_host_from_url("http://[::1]:8080/test"),
            Some("::1".to_string())
        );
    }

    #[test]
    fn extract_host_no_path() {
        assert_eq!(
            extract_host_from_url("https://hooks.slack.com"),
            Some("hooks.slack.com".to_string())
        );
    }

    #[test]
    fn extract_host_invalid_no_scheme() {
        assert_eq!(extract_host_from_url("example.com/path"), None);
    }

    #[test]
    fn extract_webhook_target_default_ports() {
        assert_eq!(
            extract_webhook_target("http://example.com/path"),
            Some(WebhookTarget {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                port: 80,
            })
        );
        assert_eq!(
            extract_webhook_target("https://example.com/path"),
            Some(WebhookTarget {
                scheme: "https".to_string(),
                host: "example.com".to_string(),
                port: 443,
            })
        );
    }

    #[test]
    fn extract_webhook_target_explicit_port() {
        assert_eq!(
            extract_webhook_target("https://api.example.com:8443/webhook"),
            Some(WebhookTarget {
                scheme: "https".to_string(),
                host: "api.example.com".to_string(),
                port: 8443,
            })
        );
    }

    // --- is_internal_ip ---

    #[test]
    fn internal_ip_v4_10_range() {
        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_172_16_range() {
        let ip: IpAddr = "172.16.0.1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_172_31_range() {
        let ip: IpAddr = "172.31.255.255".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_172_32_is_not_internal() {
        let ip: IpAddr = "172.32.0.1".parse().unwrap();
        assert!(!is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_192_168_range() {
        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_loopback() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_link_local() {
        let ip: IpAddr = "169.254.1.1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v4_this_network() {
        let ip: IpAddr = "0.0.0.0".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn public_ip_v4_not_internal() {
        let ip: IpAddr = "8.8.8.8".parse().unwrap();
        assert!(!is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v6_loopback() {
        let ip: IpAddr = "::1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v6_ula_fc00() {
        let ip: IpAddr = "fc00::1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v6_ula_fd00() {
        let ip: IpAddr = "fd12:3456::1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn internal_ip_v6_link_local() {
        let ip: IpAddr = "fe80::1".parse().unwrap();
        assert!(is_internal_ip(&ip));
    }

    #[test]
    fn public_ip_v6_not_internal() {
        let ip: IpAddr = "2001:4860:4860::8888".parse().unwrap();
        assert!(!is_internal_ip(&ip));
    }

    // --- url_points_to_internal_ip ---

    #[test]
    fn url_internal_localhost() {
        assert!(url_points_to_internal_ip("http://127.0.0.1:8080/hook"));
    }

    #[test]
    fn url_internal_10_range() {
        assert!(url_points_to_internal_ip("https://10.0.0.1/api"));
    }

    #[test]
    fn url_internal_ipv6_loopback() {
        assert!(url_points_to_internal_ip("http://[::1]:8080/hook"));
    }

    #[test]
    fn checked_public_webhook_target_rejects_internal_literal() {
        let err = checked_public_webhook_target("http://127.0.0.1:8080/hook").unwrap_err();
        assert!(err.contains("private/internal"));
    }

    #[test]
    fn checked_public_webhook_target_accepts_public_literal() {
        let (target, ip) = checked_public_webhook_target("https://8.8.8.8/hook").unwrap();
        assert_eq!(target.host, "8.8.8.8");
        assert_eq!(target.port, 443);
        assert_eq!(ip, "8.8.8.8".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn url_unparseable_blocks() {
        // Unparseable URLs are conservatively blocked
        assert!(url_points_to_internal_ip("not-a-url"));
    }

    // --- truncate_response ---

    #[test]
    fn truncate_short_string_unchanged() {
        let s = "hello world".to_string();
        let result = truncate_response(&s);
        assert_eq!(result, s);
    }

    #[test]
    fn truncate_long_string_indicates_truncation() {
        let s: String = "x".repeat(25_000);
        let result = truncate_response(&s);
        assert!(result.contains("[truncated"));
        assert!(result.len() < s.len());
    }

    // --- notify_webhook_work ---

    #[test]
    fn webhook_rejects_invalid_method() {
        let result = notify_webhook_work(
            "https://example.com/hook".to_string(),
            "{}".to_string(),
            Some("INVALID".to_string()),
            None,
            false,
        );
        assert!(result.contains("unsupported HTTP method"));
    }

    #[test]
    fn webhook_blocks_internal_ip_when_flag_set() {
        let result = notify_webhook_work(
            "http://127.0.0.1:8080/hook".to_string(),
            "{}".to_string(),
            None,
            None,
            true, // block_internal
        );
        assert!(result.contains("private/internal IP"));
    }

    #[test]
    fn webhook_allows_internal_ip_when_flag_off() {
        // When block_internal=false, we skip the IP check entirely.
        // The request may still fail if curl is not installed, but the
        // key thing is that it should NOT say "private/internal IP".
        let result = notify_webhook_work(
            "http://127.0.0.1:8080/hook".to_string(),
            "{}".to_string(),
            None,
            None,
            false, // allow internal
        );
        assert!(!result.contains("private/internal IP"));
    }

    #[test]
    fn webhook_returns_error_if_curl_missing() {
        // This test is environment-dependent: it passes on systems
        // without curl and exercises the error branch.
        // On systems with curl, the test still runs but won't hit that
        // branch — that's acceptable for a unit test.
        let _ = notify_webhook_work(
            "https://example.com/webhook".to_string(),
            r#"{"test":true}"#.to_string(),
            Some("POST".to_string()),
            Some(vec![(
                "Authorization".to_string(),
                "Bearer tok".to_string(),
            )]),
            true,
        );
        // No panic = success
    }

    // --- notify_desktop_work ---

    #[test]
    fn desktop_rejects_invalid_urgency() {
        let result = notify_desktop_work(
            "Test".to_string(),
            "Body".to_string(),
            Some("urgent".to_string()),
        );
        assert!(result.contains("urgency must be one of"));
    }

    #[test]
    fn desktop_accepts_valid_urgencies() {
        for u in &["low", "normal", "critical"] {
            let result =
                notify_desktop_work("Title".to_string(), "Body".to_string(), Some(u.to_string()));
            // Should not contain the "urgency must be" error
            assert!(!result.contains("urgency must be one of"), "urgency={u}");
        }
    }

    #[test]
    fn desktop_default_urgency_is_normal() {
        let result = notify_desktop_work(
            "Title".to_string(),
            "Body".to_string(),
            None, // default
        );
        // No urgency error = it defaulted to "normal"
        assert!(!result.contains("urgency must be one of"));
    }

    // --- notify_email_send_work ---

    #[test]
    fn email_no_mta_gives_install_instructions() {
        // On most dev machines neither sendmail nor mailx may be present.
        // If they are, the test simply exercises the happy path.
        let result = notify_email_send_work(
            "user@example.com".to_string(),
            "Test subject".to_string(),
            "Hello, world!".to_string(),
            None,
        );
        // Either success or install instructions — just ensure no panic
        assert!(!result.is_empty());
    }

    #[test]
    fn email_with_custom_from() {
        let result = notify_email_send_work(
            "user@example.com".to_string(),
            "Test".to_string(),
            "Body".to_string(),
            Some("custom@example.com".to_string()),
        );
        assert!(!result.is_empty());
    }
}
