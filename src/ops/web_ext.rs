//! # Extended Web Tools
//!
//! CLI-backed web operation work functions for the MCP server.
//!
//! Provides search, page metadata, RSS/Atom feeds, generic HTTP API calls,
//! DNS lookups, TLS certificate inspection, and file downloads — all
//! implemented via external CLI tools (`curl`, `dig`, `openssl`) gated
//! behind terminal execution permissions.

use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::process::Command;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum characters for web tool responses before truncation.
const MAX_WEB_RESPONSE_CHARS: usize = 20_000;

/// Default number of search results.
const DEFAULT_MAX_RESULTS: usize = 10;

/// Default number of RSS feed items.
const DEFAULT_MAX_ITEMS: usize = 20;

/// Default screenshot viewport width.
const DEFAULT_VIEWPORT_WIDTH: u32 = 1280;

/// Default screenshot viewport height.
const DEFAULT_VIEWPORT_HEIGHT: u32 = 800;

/// Default HTTP timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default TLS port.
const DEFAULT_TLS_PORT: u16 = 443;

/// Default maximum download size in megabytes.
const DEFAULT_MAX_DOWNLOAD_MB: u64 = 50;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a string to a maximum character count, appending an ellipsis
/// marker when truncation occurs.
fn truncate_response(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}\n…[truncated at {max} chars]")
    }
}

/// Simple percent-encoding for URL query components.
///
/// Encodes everything except unreserved characters (RFC 3986 §2.3):
/// `A-Z a-z 0-9 - . _ ~`
fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Simple percent-decoding for URL components.
fn percent_decode_simple(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                char::from(bytes[i + 1]).to_digit(16),
                char::from(bytes[i + 2]).to_digit(16),
            ) {
                out.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// Run a `Command`, returning `(exit_code_opt, stdout, stderr)` as Strings.
fn run_command(cmd: &mut Command) -> Result<(Option<i32>, String, String), String> {
    let output = cmd
        .output()
        .map_err(|e| format!("failed to execute command: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    Ok((output.status.code(), stdout, stderr))
}

/// Check that `curl` is available on `$PATH`.
fn require_curl() -> Result<(), String> {
    let status = Command::new("curl")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err("curl is not available or returned an error".to_string()),
        Err(e) => Err(format!("curl not found on PATH: {e}")),
    }
}

/// Extract the `<title>` content from an HTML document.
fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start_tag = "<title>";
    let end_tag = "</title>";
    let si = lower.find(start_tag)?;
    let content_start = si + start_tag.len();
    let ei = lower[content_start..].find(end_tag)?;
    let raw = html[content_start..content_start + ei].trim();
    if raw.is_empty() {
        None
    } else {
        Some(html_unescape(raw))
    }
}

/// Extract `<meta name="description" content="...">` from HTML.
fn extract_meta_description(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    for needle in [r#"name="description""#, r#"name='description'"#] {
        if let Some(idx) = lower.find(needle) {
            let after = &html[idx + needle.len()..];
            if let Some(ci) = after.find("content=") {
                let rest = &after[ci + 8..];
                let quote = rest.chars().next().unwrap_or('"');
                if quote == '"' || quote == '\'' {
                    let rest = &rest[1..];
                    if let Some(ei) = rest.find(quote) {
                        return Some(html_unescape(&rest[..ei]));
                    }
                }
            }
            let before = &html[..idx + needle.len()];
            if let Some(ci) = before.rfind("content=") {
                let rest = &before[ci + 8..];
                let rest_trimmed = rest.trim_start();
                let quote = rest_trimmed.chars().next().unwrap_or('"');
                if quote == '"' || quote == '\'' {
                    let rest2 = &rest_trimmed[1..];
                    if let Some(ei) = rest2.find(quote) {
                        return Some(html_unescape(&rest2[..ei]));
                    }
                }
            }
        }
    }
    None
}

/// Minimal HTML entity unescape for common entities.
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Strip HTML tags from a string (simple approach — not a full parser).
fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace
    let mut result = String::with_capacity(out.len());
    let mut prev_ws = false;
    for ch in out.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                result.push(' ');
                prev_ws = true;
            }
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }
    result.trim().to_string()
}

/// Strip `<![CDATA[...]]>` wrapper if present.
fn strip_cdata(s: &str) -> String {
    let s = s.trim();
    if s.starts_with("<![CDATA[") && s.ends_with("]]>") {
        s[9..s.len() - 3].to_string()
    } else {
        s.to_string()
    }
}

/// Extract the text content of an XML element: `<tag>content</tag>`.
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let idx = xml.find(&open)?;
    let content_start = idx + open.len();
    let rest = &xml[content_start..];
    let end = rest.find(&close)?;
    let content = rest[..end].trim().to_string();
    Some(strip_cdata(&content))
}

/// Extract text from an XML tag where the tag may have attributes:
/// `<link rel="...">text</link>`.
fn extract_xml_tag_with_attrs(xml: &str, tag: &str) -> Option<String> {
    let open_start = format!("<{tag}");
    let close = format!("</{tag}>");
    let idx = xml.find(&open_start)?;
    let after = &xml[idx + open_start.len()..];
    let gt = after.find('>')?;
    let content_start = idx + open_start.len() + gt + 1;
    if content_start >= xml.len() {
        return None;
    }
    let rest = &xml[content_start..];
    let end = rest.find(&close)?;
    let content = rest[..end].trim().to_string();
    Some(strip_cdata(&content))
}

/// Extract `href="..."` value from a string.
fn extract_href_attr(s: &str) -> Option<String> {
    let href_key = "href=\"";
    let idx = s.find(href_key)?;
    let rest = &s[idx + href_key.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Parse HTTP headers to extract status code, content-length, and content-type.
fn parse_http_headers(headers: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut status_code: Option<String> = None;
    let mut content_length: Option<String> = None;
    let mut content_type: Option<String> = None;

    for line in headers.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("http/") {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                status_code = Some(parts[1].to_string());
            }
        }
        if line_lower.starts_with("content-length:") {
            if let Some(v) = line.split(':').nth(1) {
                content_length = Some(v.trim().to_string());
            }
        }
        if line_lower.starts_with("content-type:") {
            if let Some(v) = line.split(':').nth(1) {
                content_type = Some(v.trim().to_string());
            }
        }
    }

    (status_code, content_length, content_type)
}

/// Split an HTTP response (with `-i` headers included) into header and body.
fn split_http_response(response: &str) -> (&str, &str) {
    if let Some(idx) = response.find("\r\n\r\n") {
        (&response[..idx], &response[idx + 4..])
    } else if let Some(idx) = response.find("\n\n") {
        (&response[..idx], &response[idx + 2..])
    } else {
        ("", response)
    }
}

// ---------------------------------------------------------------------------
// 1. web_search_work
// ---------------------------------------------------------------------------

/// Search the web using DuckDuckGo HTML endpoint via `curl`.
///
/// # Arguments
/// * `query` — search query string
/// * `engine` — currently only `"duckduckgo"` (default)
/// * `max_results` — maximum number of results to return (default 10)
///
/// # Returns
/// Formatted list of search results with title, snippet, and URL.
pub fn web_search_work(
    query: String,
    engine: Option<String>,
    max_results: Option<usize>,
) -> String {
    let engine = engine.unwrap_or_else(|| "duckduckgo".to_string());
    let max = max_results.unwrap_or(DEFAULT_MAX_RESULTS).clamp(1, 100);

    if engine.to_lowercase() != "duckduckgo" {
        return format!(
            "WEB_SEARCH\n\nError: unsupported engine '{engine}'. Only 'duckduckgo' is supported."
        );
    }

    if query.trim().is_empty() {
        return "WEB_SEARCH\n\nError: query must not be empty".to_string();
    }

    if let Err(e) = require_curl() {
        return format!("WEB_SEARCH\n\nError: {e}");
    }

    let encoded = url_encode(query.trim());
    let url = format!("https://html.duckduckgo.com/html/?q={encoded}");

    let result = run_command(Command::new("curl").args([
        "-s",
        "-L",
        "--max-time",
        &DEFAULT_TIMEOUT_SECS.to_string(),
        "-A",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        &url,
    ]));

    let (exit_code, stdout, stderr) = match result {
        Ok(v) => v,
        Err(e) => return format!("WEB_SEARCH\n\nError: {e}"),
    };

    if let Some(code) = exit_code {
        if code != 0 && stdout.trim().is_empty() {
            return format!(
                "WEB_SEARCH\n\nError: curl exited with code {code}\n{}",
                stderr.trim()
            );
        }
    }

    let results = parse_ddg_results(&stdout, max);

    if results.is_empty() {
        return format!(
            "WEB_SEARCH\n\nQuery: {}\nEngine: duckduckgo\nNo results found.",
            query.trim()
        );
    }

    let mut out = format!(
        "WEB_SEARCH\n\nQuery: {}\nEngine: duckduckgo\nResults: {}/{}\n\n",
        query.trim(),
        results.len(),
        max
    );

    for (i, r) in results.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, r.title));
        out.push_str(&format!("   URL: {}\n", r.url));
        if !r.snippet.is_empty() {
            out.push_str(&format!("   {}\n", r.snippet));
        }
        out.push('\n');
    }

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

/// A single DuckDuckGo search result.
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

/// Parse DuckDuckGo HTML response to extract search results.
fn parse_ddg_results(html: &str, max: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // DDG HTML uses class="result" or class="result ..." on result container divs.
    // We find each occurrence of `class="result"` that is followed by a space or
    // a closing quote, but NOT by `__` (which would match `result__a` / `result__snippet`).
    let marker = "class=\"result";
    let marker_len = marker.len();
    let mut search_from = 0;

    while results.len() < max {
        let offset = match html[search_from..].find(marker) {
            Some(i) => i,
            None => break,
        };
        let abs = search_from + offset;
        let after_marker = abs + marker_len;

        // Check the character right after `class="result`:
        //   - `"`  → class="result"   (result-only div, e.g. ads)
        //   - ` `  → class="result ..." (result with extra classes)
        //   - `_`  → class="result__a" or "result__snippet" — SKIP
        let next_char = html.as_bytes().get(after_marker).copied();
        match next_char {
            Some(b'"') | Some(b' ') => { /* valid result block */ }
            Some(b'_') => {
                // class="result__a" or "result__snippet" — not a result container
                search_from = after_marker;
                continue;
            }
            _ => {
                search_from = after_marker;
                continue;
            }
        }

        // The part from after the closing quote of class="..." to the next marker
        let _part_start = after_marker + 1; // skip the closing quote or space+rest
                                            // For `class="result"`, the `"` at after_marker is the close-quote, part starts after it
                                            // For `class="result ...">`, we need to find the closing `"`
        let part = if next_char == Some(b'"') {
            &html[after_marker + 1..]
        } else {
            // skip space + extra class names until the closing `"`
            let rest = &html[after_marker..];
            match rest.find('"') {
                Some(end) => &html[after_marker + end + 1..],
                None => &html[after_marker..],
            }
        };

        let title = extract_ddg_link_text(part, "result__a");
        let url = extract_ddg_link_href(part, "result__a");
        let snippet = extract_ddg_snippet(part);

        if let (Some(t), Some(u)) = (title, url) {
            // Skip ad results
            if u.contains("//ad.ddg") || u.starts_with("https://duckduckgo.com/y.js") {
                search_from = after_marker;
                continue;
            }
            results.push(SearchResult {
                title: strip_html_tags(&t),
                url: clean_ddg_url(&u),
                snippet: strip_html_tags(&snippet),
            });
        }

        search_from = after_marker;
    }

    results
}

/// Extract the text content of an `<a>` tag with a specific class from DDG HTML.
fn extract_ddg_link_text(html: &str, class: &str) -> Option<String> {
    let class_attr = format!("class=\"{class}\"");
    let idx = html.find(&class_attr)?;
    let after = &html[idx + class_attr.len()..];
    let gt = after.find('>')?;
    let content_start = idx + class_attr.len() + gt + 1;
    if content_start >= html.len() {
        return None;
    }
    let rest = &html[content_start..];
    let end_tag = "</a>";
    let end_idx = rest.find(end_tag)?;
    Some(rest[..end_idx].trim().to_string())
}

/// Extract the `href` from an `<a>` tag with a specific class from DDG HTML.
///
/// Searches the entire opening `<a …>` tag (from `<a` to the first `>`)
/// for the `href` attribute, regardless of attribute order.
fn extract_ddg_link_href(html: &str, class: &str) -> Option<String> {
    let class_attr = format!("class=\"{class}\"");
    let class_idx = html.find(&class_attr)?;

    // Walk backwards from the class attribute to find the <a opening
    let before = &html[..class_idx + class_attr.len()];
    let a_start = before.rfind("<a ")?;

    // Walk forwards from the class attribute to find the > closing the tag
    let after = &html[class_idx + class_attr.len()..];
    let tag_close = after.find('>')?;

    let full_tag = &html[a_start..class_idx + class_attr.len() + tag_close + 1];

    // Find href="..." within the full tag
    extract_href_attr(full_tag)
}

/// Extract the snippet text from a DDG result block.
fn extract_ddg_snippet(html: &str) -> String {
    let class_attr = "class=\"result__snippet\"";
    if let Some(idx) = html.find(class_attr) {
        let after = &html[idx + class_attr.len()..];
        if let Some(gt) = after.find('>') {
            let content_start = idx + class_attr.len() + gt + 1;
            if content_start < html.len() {
                let rest = &html[content_start..];
                if let Some(end) = rest.find("</a>") {
                    return rest[..end].trim().to_string();
                }
                if let Some(end) = rest.find("</td>") {
                    return rest[..end].trim().to_string();
                }
            }
        }
    }
    String::new()
}

/// Clean a DDG redirect URL.
///
/// DDG wraps external URLs through `/l/?uddg=<encoded>&rut=...`.
/// We extract and decode the `uddg` parameter if present.
fn clean_ddg_url(raw: &str) -> String {
    if let Some(idx) = raw.find("uddg=") {
        let rest = &raw[idx + 5..];
        let end = rest.find('&').unwrap_or(rest.len());
        let encoded = &rest[..end];
        let decoded = percent_decode_simple(encoded);
        if decoded.starts_with("http://") || decoded.starts_with("https://") {
            return decoded;
        }
    }
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else if raw.starts_with("//") {
        format!("https:{}", raw)
    } else {
        raw.to_string()
    }
}

// ---------------------------------------------------------------------------
// 2. web_screenshot_work
// ---------------------------------------------------------------------------

/// Fetch page metadata (since real screenshots require a browser).
///
/// # Arguments
/// * `url` — target URL
/// * `width` — viewport width (default 1280)
/// * `height` — viewport height (default 800)
/// * `output_path` — where a screenshot would be saved (informational only)
///
/// # Returns
/// Page title, meta description, HTTP status code, and content length.
pub fn web_screenshot_work(
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    output_path: Option<PathBuf>,
) -> String {
    let w = width.unwrap_or(DEFAULT_VIEWPORT_WIDTH);
    let h = height.unwrap_or(DEFAULT_VIEWPORT_HEIGHT);

    if url.trim().is_empty() {
        return "WEB_SCREENSHOT\n\nError: URL must not be empty".to_string();
    }

    if let Err(e) = require_curl() {
        return format!("WEB_SCREENSHOT\n\nError: {e}");
    }

    let url_trimmed = url.trim();

    // Step 1: Fetch headers for status code and content-length
    let header_result = run_command(Command::new("curl").args([
        "-sI",
        "-L",
        "--max-time",
        &DEFAULT_TIMEOUT_SECS.to_string(),
        url_trimmed,
    ]));

    let (status_code, content_length, content_type_header) = match header_result {
        Ok((_, stdout, _)) => parse_http_headers(&stdout),
        Err(e) => return format!("WEB_SCREENSHOT\n\nError fetching headers: {e}"),
    };

    // Step 2: Fetch body for title and meta description
    let body_result = run_command(Command::new("curl").args([
        "-s",
        "-L",
        "--max-time",
        &DEFAULT_TIMEOUT_SECS.to_string(),
        url_trimmed,
    ]));

    let (title, meta_desc) = match body_result {
        Ok((_, stdout, _)) => {
            let html_prefix = if stdout.len() > 200_000 {
                &stdout[..200_000]
            } else {
                &stdout
            };
            (
                extract_html_title(html_prefix),
                extract_meta_description(html_prefix),
            )
        }
        Err(_) => (None, None),
    };

    let mut out = String::from("WEB_SCREENSHOT\n\n");
    out.push_str(&format!("URL: {url_trimmed}\n"));
    out.push_str(&format!("Viewport: {w}x{h}\n"));
    if let Some(ref p) = output_path {
        out.push_str(&format!("Output path: {}\n", p.display()));
    } else {
        out.push_str("Output path: (not specified)\n");
    }
    out.push('\n');
    out.push_str("--- Page Metadata ---\n");
    out.push_str(&format!(
        "Title: {}\n",
        title.unwrap_or_else(|| "(no title found)".to_string())
    ));
    out.push_str(&format!(
        "Description: {}\n",
        meta_desc.unwrap_or_else(|| "(no meta description)".to_string())
    ));
    out.push_str(&format!(
        "HTTP Status: {}\n",
        status_code.unwrap_or_else(|| "unknown".to_string())
    ));
    out.push_str(&format!(
        "Content-Length: {}\n",
        content_length.as_deref().unwrap_or("unknown")
    ));
    out.push_str(&format!(
        "Content-Type: {}\n",
        content_type_header.unwrap_or_else(|| "unknown".to_string())
    ));
    out.push('\n');
    out.push_str("Note: Full page screenshot requires a headless browser.\n");
    out.push_str("This tool fetches page metadata as a lightweight alternative.\n");

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 3. web_rss_fetch_work
// ---------------------------------------------------------------------------

/// Fetch and parse an RSS or Atom feed via `curl`.
///
/// # Arguments
/// * `url` — URL of the RSS/Atom feed
/// * `max_items` — maximum number of items to return (default 20)
///
/// # Returns
/// Formatted list of feed items with title, link, description, and pub date.
pub fn web_rss_fetch_work(url: String, max_items: Option<usize>) -> String {
    let max = max_items.unwrap_or(DEFAULT_MAX_ITEMS).clamp(1, 500);

    if url.trim().is_empty() {
        return "WEB_RSS_FETCH\n\nError: URL must not be empty".to_string();
    }

    if let Err(e) = require_curl() {
        return format!("WEB_RSS_FETCH\n\nError: {e}");
    }

    let result = run_command(Command::new("curl").args([
        "-s",
        "-L",
        "--max-time",
        &DEFAULT_TIMEOUT_SECS.to_string(),
        url.trim(),
    ]));

    let (exit_code, stdout, stderr) = match result {
        Ok(v) => v,
        Err(e) => return format!("WEB_RSS_FETCH\n\nError: {e}"),
    };

    if let Some(code) = exit_code {
        if code != 0 && stdout.trim().is_empty() {
            return format!(
                "WEB_RSS_FETCH\n\nError: curl exited with code {code}\n{}",
                stderr.trim()
            );
        }
    }

    if stdout.trim().is_empty() {
        return "WEB_RSS_FETCH\n\nError: empty response from server".to_string();
    }

    let (feed_title, items) = parse_rss_atom(&stdout, max);

    let mut out = String::from("WEB_RSS_FETCH\n\n");
    out.push_str(&format!(
        "Feed: {}\n",
        feed_title.unwrap_or_else(|| "(untitled)".to_string())
    ));
    out.push_str(&format!("URL: {}\n", url.trim()));
    out.push_str(&format!("Items: {}/{}\n\n", items.len(), max));

    for (i, item) in items.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, item.title));
        if !item.link.is_empty() {
            out.push_str(&format!("   Link: {}\n", item.link));
        }
        if !item.pub_date.is_empty() {
            out.push_str(&format!("   Date: {}\n", item.pub_date));
        }
        if !item.description.is_empty() {
            let desc_preview = if item.description.chars().count() > 300 {
                let s: String = item.description.chars().take(300).collect();
                format!("{s}…")
            } else {
                item.description.clone()
            };
            out.push_str(&format!("   {desc_preview}\n"));
        }
        out.push('\n');
    }

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

/// A parsed RSS/Atom feed item.
struct FeedItem {
    title: String,
    link: String,
    description: String,
    pub_date: String,
}

/// Parse RSS 2.0 or Atom XML to extract feed items.
fn parse_rss_atom(xml: &str, max: usize) -> (Option<String>, Vec<FeedItem>) {
    let feed_title = extract_xml_tag(xml, "title");

    // Detect Atom vs RSS
    let is_atom = xml.contains("<feed") && (xml.contains("xmlns") || xml.contains("Atom"));
    let is_rss = xml.contains("<rss") || xml.contains("<RDF") || xml.contains("<channel");

    if is_atom && !is_rss {
        (feed_title, parse_atom_entries(xml, max))
    } else {
        let items = parse_rss_items(xml, max);
        if items.is_empty() && is_atom {
            (feed_title, parse_atom_entries(xml, max))
        } else {
            (feed_title, items)
        }
    }
}

/// Parse RSS 2.0 `<item>` elements.
fn parse_rss_items(xml: &str, max: usize) -> Vec<FeedItem> {
    let mut items = Vec::new();
    let parts: Vec<&str> = xml.split("<item>").collect();

    for part in parts.iter().skip(1) {
        if items.len() >= max {
            break;
        }
        let block = match part.find("</item>") {
            Some(end) => &part[..end],
            None => part,
        };

        let title = extract_xml_tag(block, "title")
            .map(|t| strip_html_tags(&t))
            .unwrap_or_default();
        let link = extract_xml_tag_with_attrs(block, "link")
            .or_else(|| extract_xml_tag(block, "link"))
            .unwrap_or_default();
        let description = extract_xml_tag(block, "description")
            .or_else(|| extract_xml_tag(block, "content:encoded"))
            .map(|d| strip_html_tags(&d))
            .unwrap_or_default();
        let pub_date = extract_xml_tag(block, "pubDate")
            .or_else(|| extract_xml_tag(block, "dc:date"))
            .or_else(|| extract_xml_tag(block, "published"))
            .unwrap_or_default();

        if !title.is_empty() || !link.is_empty() {
            items.push(FeedItem {
                title,
                link: link.trim().to_string(),
                description,
                pub_date,
            });
        }
    }

    items
}

/// Parse Atom `<entry>` elements.
fn parse_atom_entries(xml: &str, max: usize) -> Vec<FeedItem> {
    let mut items = Vec::new();
    let parts: Vec<&str> = xml.split("<entry").collect();

    for part in parts.iter().skip(1) {
        if items.len() >= max {
            break;
        }
        let block = match part.find("</entry>") {
            Some(end) => &part[..end],
            None => part,
        };

        let title = extract_xml_tag(block, "title")
            .map(|t| strip_html_tags(&t))
            .unwrap_or_default();
        let link = extract_atom_link(block);
        let description = extract_xml_tag(block, "summary")
            .or_else(|| extract_xml_tag(block, "content"))
            .map(|d| strip_html_tags(&d))
            .unwrap_or_default();
        let pub_date = extract_xml_tag(block, "updated")
            .or_else(|| extract_xml_tag(block, "published"))
            .unwrap_or_default();

        if !title.is_empty() || !link.is_empty() {
            items.push(FeedItem {
                title,
                link,
                description,
                pub_date,
            });
        }
    }

    items
}

/// Extract the `href` from an Atom `<link>` element with `rel="alternate"`,
/// or the first non-self link.
fn extract_atom_link(block: &str) -> String {
    let parts: Vec<&str> = block.split("<link ").collect();
    // Prefer rel="alternate"
    for part in parts.iter().skip(1) {
        let lower = part.to_lowercase();
        if lower.contains("rel=\"alternate\"") || lower.contains("rel='alternate'") {
            if let Some(href) = extract_href_attr(part) {
                return href;
            }
        }
    }
    // Fallback: first link that is not rel="self"
    for part in parts.iter().skip(1) {
        let lower = part.to_lowercase();
        if !lower.contains("rel=\"self\"") && !lower.contains("rel='self'") {
            if let Some(href) = extract_href_attr(part) {
                return href;
            }
        }
    }
    String::new()
}

// ---------------------------------------------------------------------------
// 4. web_api_call_work
// ---------------------------------------------------------------------------

/// Generic HTTP API call via `curl`.
///
/// # Arguments
/// * `url` — target URL
/// * `method` — HTTP method (GET, POST, PUT, DELETE, PATCH)
/// * `headers` — optional headers in `"Key: Value\nKey: Value"` format
/// * `body` — optional request body (for POST/PUT/PATCH)
/// * `timeout_secs` — timeout in seconds (default 30)
///
/// # Returns
/// HTTP status code and response body (truncated if large).
pub fn web_api_call_work(
    url: String,
    method: String,
    headers: Option<String>,
    body: Option<String>,
    timeout_secs: Option<u64>,
) -> String {
    let timeout = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS).clamp(1, 300);
    let method_upper = method.trim().to_uppercase();

    if url.trim().is_empty() {
        return "WEB_API_CALL\n\nError: URL must not be empty".to_string();
    }

    match method_upper.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {}
        _ => {
            return format!(
                "WEB_API_CALL\n\nError: unsupported HTTP method '{}'. \
                 Supported: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS",
                method
            );
        }
    }

    if let Err(e) = require_curl() {
        return format!("WEB_API_CALL\n\nError: {e}");
    }

    let mut cmd = Command::new("curl");
    cmd.args([
        "-s",
        "-i",
        "-L",
        "--max-time",
        &timeout.to_string(),
        "-X",
        &method_upper,
    ]);

    if let Some(ref hdrs) = headers {
        for line in hdrs.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                cmd.arg("-H").arg(trimmed);
            }
        }
    }

    if let Some(ref b) = body {
        if !b.is_empty() && matches!(method_upper.as_str(), "POST" | "PUT" | "PATCH") {
            cmd.args(["-d", b.as_str()]);
        }
    }

    cmd.arg(url.trim());

    let result = run_command(&mut cmd);

    let (exit_code, stdout, stderr) = match result {
        Ok(v) => v,
        Err(e) => return format!("WEB_API_CALL\n\nError: {e}"),
    };

    if let Some(code) = exit_code {
        if code != 0 && stdout.trim().is_empty() {
            return format!(
                "WEB_API_CALL\n\nError: curl exited with code {code}\n{}",
                stderr.trim()
            );
        }
    }

    let (resp_headers, resp_body) = split_http_response(&stdout);
    let (status_code, _, content_type) = parse_http_headers(resp_headers);

    let mut out = String::from("WEB_API_CALL\n\n");
    out.push_str(&format!("URL: {}\n", url.trim()));
    out.push_str(&format!("Method: {method_upper}\n"));
    out.push_str(&format!(
        "Status: {}\n",
        status_code.unwrap_or_else(|| "unknown".to_string())
    ));
    if let Some(ct) = content_type {
        out.push_str(&format!("Content-Type: {ct}\n"));
    }
    out.push('\n');

    if !resp_body.trim().is_empty() {
        out.push_str("--- Response Body ---\n");
        if resp_body.chars().count() > 15_000 {
            let preview: String = resp_body.chars().take(15_000).collect();
            out.push_str(&preview);
            out.push_str("\n\n…[response body truncated]");
        } else {
            out.push_str(resp_body);
        }
    } else {
        out.push_str("(empty response body)");
    }

    out.push('\n');

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// 5. web_dns_lookup_work
// ---------------------------------------------------------------------------

/// DNS lookup using `dig` (or `nslookup` as fallback).
///
/// # Arguments
/// * `domain` — domain name to look up
/// * `record_type` — DNS record type: "A", "AAAA", "MX", "CNAME", "TXT", "NS"
///   (default: "A")
///
/// # Returns
/// DNS records for the given domain and record type.
pub fn web_dns_lookup_work(domain: String, record_type: Option<String>) -> String {
    let rtype = record_type
        .unwrap_or_else(|| "A".to_string())
        .trim()
        .to_uppercase();

    if domain.trim().is_empty() {
        return "WEB_DNS_LOOKUP\n\nError: domain must not be empty".to_string();
    }

    match rtype.as_str() {
        "A" | "AAAA" | "MX" | "CNAME" | "TXT" | "NS" | "SOA" | "SRV" | "PTR" | "CAA" => {}
        _ => {
            return format!(
                "WEB_DNS_LOOKUP\n\nError: unsupported record type '{rtype}'. \
                 Supported: A, AAAA, MX, CNAME, TXT, NS, SOA, SRV, PTR, CAA"
            );
        }
    }

    let domain_trimmed = domain.trim();

    // Try dig first
    let dig_result = run_command(Command::new("dig").args([
        domain_trimmed,
        &rtype,
        "+short",
        "+timeout=10",
        "+tries=2",
    ]));

    match dig_result {
        Ok((Some(code), ref stdout, _)) if code == 0 && !stdout.trim().is_empty() => {
            let mut out = format!("WEB_DNS_LOOKUP\n\nDomain: {domain_trimmed}\nType: {rtype}\n\n");
            out.push_str("Records:\n");
            for (i, line) in stdout.lines().enumerate() {
                if i >= 100 {
                    out.push_str("…[too many records, truncated]\n");
                    break;
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("   {trimmed}\n"));
                }
            }
            return truncate_response(&out, MAX_WEB_RESPONSE_CHARS);
        }
        Ok((_, ref stdout, _)) if !stdout.trim().is_empty() => {
            let mut out = format!("WEB_DNS_LOOKUP\n\nDomain: {domain_trimmed}\nType: {rtype}\n\n");
            out.push_str("Records:\n");
            for (i, line) in stdout.lines().enumerate() {
                if i >= 100 {
                    out.push_str("…[too many records, truncated]\n");
                    break;
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    out.push_str(&format!("   {trimmed}\n"));
                }
            }
            return truncate_response(&out, MAX_WEB_RESPONSE_CHARS);
        }
        _ => {} // Fall through to nslookup
    }

    // Fallback: nslookup
    let ns_result = run_command(
        Command::new("nslookup")
            .args(["-type=", &rtype, domain_trimmed])
            .env("LANG", "C"),
    );

    match ns_result {
        Ok((_, stdout, stderr)) => {
            if stdout.trim().is_empty() && !stderr.trim().is_empty() {
                format!(
                    "WEB_DNS_LOOKUP\n\nDomain: {domain_trimmed}\nType: {rtype}\n\n\
                     Error: {}\n(dig and nslookup both failed)",
                    stderr.trim()
                )
            } else {
                let mut out =
                    format!("WEB_DNS_LOOKUP\n\nDomain: {domain_trimmed}\nType: {rtype}\n\n");
                out.push_str("Records (via nslookup):\n");
                out.push_str(&stdout);
                truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
            }
        }
        Err(e) => format!(
            "WEB_DNS_LOOKUP\n\nDomain: {domain_trimmed}\nType: {rtype}\n\n\
             Error: neither dig nor nslookup available: {e}"
        ),
    }
}

// ---------------------------------------------------------------------------
// 6. web_tls_inspect_work
// ---------------------------------------------------------------------------

/// Inspect a TLS/SSL certificate using `openssl s_client` + `openssl x509`.
///
/// # Arguments
/// * `host` — hostname to inspect
/// * `port` — port number (default 443)
///
/// # Returns
/// Certificate details: issuer, subject, validity dates, and SANs.
pub fn web_tls_inspect_work(host: String, port: Option<u16>) -> String {
    let port = port.unwrap_or(DEFAULT_TLS_PORT);

    if host.trim().is_empty() {
        return "WEB_TLS_INSPECT\n\nError: host must not be empty".to_string();
    }

    let host_trimmed = host.trim();

    // Verify openssl is available
    let openssl_check = Command::new("openssl")
        .arg("version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match openssl_check {
        Ok(s) if s.success() => {}
        Ok(_) => return "WEB_TLS_INSPECT\n\nError: openssl is not available on PATH".to_string(),
        Err(e) => {
            return format!("WEB_TLS_INSPECT\n\nError: openssl not found on PATH: {e}");
        }
    }

    let connect_str = format!("{host_trimmed}:{port}");

    // Step 1: Run openssl s_client to fetch the certificate
    let s_client_output = Command::new("openssl")
        .args([
            "s_client",
            "-connect",
            &connect_str,
            "-servername",
            host_trimmed,
        ])
        .stdin(std::process::Stdio::null())
        .output();

    let (s_client_stdout, s_client_stderr) = match s_client_output {
        Ok(o) => (
            String::from_utf8_lossy(&o.stdout).into_owned(),
            String::from_utf8_lossy(&o.stderr).into_owned(),
        ),
        Err(e) => return format!("WEB_TLS_INSPECT\n\nError running openssl s_client: {e}"),
    };

    if s_client_stdout.trim().is_empty() {
        return format!(
            "WEB_TLS_INSPECT\n\nError: no output from openssl s_client\n{}",
            s_client_stderr.trim()
        );
    }

    // Step 2: Pipe the certificate to openssl x509 -noout -text
    let x509_child = Command::new("openssl")
        .args(["x509", "-noout", "-text"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut child = match x509_child {
        Ok(c) => c,
        Err(e) => return format!("WEB_TLS_INSPECT\n\nError running openssl x509: {e}"),
    };

    if let Some(ref mut stdin) = child.stdin {
        if let Err(e) = stdin.write_all(s_client_stdout.as_bytes()) {
            return format!("WEB_TLS_INSPECT\n\nError piping to openssl x509: {e}");
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return format!("WEB_TLS_INSPECT\n\nError waiting for openssl x509: {e}"),
    };

    let x509_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let x509_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if x509_stdout.trim().is_empty() {
        return format!(
            "WEB_TLS_INSPECT\n\nError: no certificate output from openssl x509\n{}",
            x509_stderr.trim()
        );
    }

    // Parse the x509 -text output for key fields
    let mut out = format!("WEB_TLS_INSPECT\n\nHost: {host_trimmed}\nPort: {port}\n\n");

    out.push_str("--- Certificate Details ---\n\n");

    if let Some(v) = extract_x509_field(&x509_stdout, "Issuer:") {
        out.push_str(&format!("Issuer: {v}\n"));
    }
    if let Some(v) = extract_x509_field(&x509_stdout, "Subject:") {
        out.push_str(&format!("Subject: {v}\n"));
    }
    if let Some(v) = extract_x509_field(&x509_stdout, "Not Before") {
        out.push_str(&format!("Valid From: {v}\n"));
    }
    if let Some(v) = extract_x509_field(&x509_stdout, "Not After") {
        out.push_str(&format!("Valid Until: {v}\n"));
    }

    out.push_str(&extract_san_list(&x509_stdout));

    out.push_str("\n--- Raw Certificate Text ---\n");
    out.push_str(&x509_stdout);

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

/// Extract a field value from `openssl x509 -text` output.
fn extract_x509_field(text: &str, prefix: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(prefix) {
            let value = trimmed[prefix.len()..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Extract Subject Alternative Names from `openssl x509 -text` output.
///
/// The openssl output typically looks like:
/// ```text
/// X509v3 Subject Alternative Name: critical
///     DNS:example.com, DNS:www.example.com, IP Address:1.2.3.4
/// ```
fn extract_san_list(text: &str) -> String {
    let mut san_text = String::new();
    let mut in_san_section = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Subject Alternative Name") {
            in_san_section = true;
            if let Some(idx) = trimmed.find(':') {
                let rest = trimmed[idx + 1..].trim();
                if !rest.is_empty() && rest != "critical" {
                    san_text.push_str(rest);
                    san_text.push(',');
                }
            }
            continue;
        }
        if in_san_section {
            // Stop at non-indented, non-empty line
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                break;
            }
            // Collect continuation line entries
            if !trimmed.is_empty() {
                san_text.push_str(trimmed);
                san_text.push(',');
            }
        }
    }

    // Parse collected SAN text for known entry types
    let sans: Vec<String> = san_text
        .split(',')
        .map(|e| e.trim())
        .filter(|e| {
            !e.is_empty()
                && (e.starts_with("DNS:")
                    || e.starts_with("IP Address:")
                    || e.starts_with("email:")
                    || e.starts_with("URI:")
                    || e.starts_with("RID:"))
        })
        .map(|e| e.to_string())
        .collect();

    if sans.is_empty() {
        return "SANs: (none)\n".to_string();
    }

    let mut out = "SANs:\n".to_string();
    for san in &sans {
        out.push_str(&format!("  - {san}\n"));
    }
    out
}

// ---------------------------------------------------------------------------
// 7. web_download_file_work
// ---------------------------------------------------------------------------

/// Download a file via `curl`.
///
/// # Arguments
/// * `url` — URL of the file to download
/// * `output_path` — local path where the file will be saved
/// * `max_size_mb` — maximum allowed download size in MB (default 50)
///
/// # Returns
/// File path, file size, and content type of the downloaded file.
pub fn web_download_file_work(
    url: String,
    output_path: PathBuf,
    max_size_mb: Option<u64>,
) -> String {
    let max_mb = max_size_mb
        .unwrap_or(DEFAULT_MAX_DOWNLOAD_MB)
        .clamp(1, 10_000);
    let max_bytes = max_mb * 1024 * 1024;

    if url.trim().is_empty() {
        return "WEB_DOWNLOAD_FILE\n\nError: URL must not be empty".to_string();
    }

    if output_path.as_os_str().is_empty() {
        return "WEB_DOWNLOAD_FILE\n\nError: output_path must not be empty".to_string();
    }

    if let Err(e) = require_curl() {
        return format!("WEB_DOWNLOAD_FILE\n\nError: {e}");
    }

    let url_trimmed = url.trim();
    let out_str = output_path.display().to_string();

    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return format!(
                    "WEB_DOWNLOAD_FILE\n\nError: could not create parent directory {}: {e}",
                    parent.display()
                );
            }
        }
    }

    // Step 1: Get content-type and content-length first (HEAD request)
    let head_result = run_command(Command::new("curl").args([
        "-sI",
        "-L",
        "--max-time",
        &DEFAULT_TIMEOUT_SECS.to_string(),
        url_trimmed,
    ]));

    let (content_type, content_length) = match head_result {
        Ok((_, stdout, _)) => {
            let (_, cl, ct) = parse_http_headers(&stdout);
            (ct, cl)
        }
        Err(_) => (None, None),
    };

    // Check content-length against max size if available
    if let Some(ref cl_str) = content_length {
        if let Ok(cl) = cl_str.parse::<u64>() {
            if cl > max_bytes {
                return format!(
                    "WEB_DOWNLOAD_FILE\n\nError: remote file size ({cl} bytes) \
                     exceeds maximum ({max_bytes} bytes = {max_mb} MB).\nURL: {url_trimmed}"
                );
            }
        }
    }

    // Step 2: Download the file
    let download_result = run_command(Command::new("curl").args([
        "-sL",
        "--max-time",
        "300",
        "--max-filesize",
        &max_bytes.to_string(),
        "-o",
        &out_str,
        url_trimmed,
    ]));

    match download_result {
        Ok((Some(code), _, stderr)) if code != 0 => {
            let err_msg = stderr.trim();
            return format!(
                "WEB_DOWNLOAD_FILE\n\nError: curl exited with code {code}\n{err_msg}\nURL: {url_trimmed}"
            );
        }
        Ok((None, _, stderr)) => {
            return format!(
                "WEB_DOWNLOAD_FILE\n\nError: curl was terminated by signal\n{}\nURL: {url_trimmed}",
                stderr.trim()
            );
        }
        Err(e) => {
            return format!("WEB_DOWNLOAD_FILE\n\nError: {e}\nURL: {url_trimmed}");
        }
        _ => {}
    }

    // Step 3: Verify the downloaded file
    let file_meta = match std::fs::metadata(&output_path) {
        Ok(m) => m,
        Err(e) => {
            return format!(
                "WEB_DOWNLOAD_FILE\n\nError: downloaded file not found at {}: {e}",
                output_path.display()
            );
        }
    };

    let file_size = file_meta.len();
    if file_size > max_bytes {
        let _ = std::fs::remove_file(&output_path);
        return format!(
            "WEB_DOWNLOAD_FILE\n\nError: downloaded file size ({file_size} bytes) \
             exceeds maximum ({max_bytes} bytes = {max_mb} MB). File removed.\nURL: {url_trimmed}"
        );
    }

    if file_size == 0 {
        return format!(
            "WEB_DOWNLOAD_FILE\n\nWarning: downloaded file is empty (0 bytes).\n\
             URL: {url_trimmed}\nPath: {}",
            output_path.display()
        );
    }

    let mut out = String::from("WEB_DOWNLOAD_FILE\n\n");
    out.push_str(&format!("URL: {url_trimmed}\n"));
    out.push_str(&format!("Path: {}\n", output_path.display()));
    out.push_str(&format!(
        "Size: {file_size} bytes ({:.2} MB)\n",
        file_size as f64 / (1024.0 * 1024.0)
    ));
    out.push_str(&format!(
        "Content-Type: {}\n",
        content_type.unwrap_or_else(|| "unknown".to_string())
    ));
    out.push_str(&format!("Max allowed: {max_mb} MB\n"));
    out.push_str("Status: download complete\n");

    truncate_response(&out, MAX_WEB_RESPONSE_CHARS)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- URL encoding tests ---

    #[test]
    fn url_encode_spaces() {
        assert_eq!(url_encode("hello world"), "hello%20world");
    }

    #[test]
    fn url_encode_special_chars() {
        assert_eq!(url_encode("a&b=c+d"), "a%26b%3Dc%2Bd");
    }

    #[test]
    fn url_encode_unreserved() {
        assert_eq!(url_encode("A-Z.0-9_hello-world"), "A-Z.0-9_hello-world");
    }

    #[test]
    fn url_encode_unicode() {
        // UTF-8 bytes of é: 0xC3 0xA9
        assert_eq!(url_encode("café"), "caf%C3%A9");
    }

    // --- Percent decode tests ---

    #[test]
    fn percent_decode_basic() {
        assert_eq!(percent_decode_simple("hello%20world"), "hello world");
    }

    #[test]
    fn percent_decode_plus() {
        assert_eq!(percent_decode_simple("a+b"), "a b");
    }

    #[test]
    fn percent_decode_identity() {
        assert_eq!(
            percent_decode_simple("no-encoding-needed"),
            "no-encoding-needed"
        );
    }

    // --- Truncation tests ---

    #[test]
    fn truncate_response_short() {
        let s = "hello".to_string();
        let result = truncate_response(&s, 100);
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_response_exact() {
        let s = "hello".to_string();
        let result = truncate_response(&s, 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn truncate_response_long() {
        let s = "abcdefghij".to_string();
        let result = truncate_response(&s, 5);
        assert!(result.contains("abcde"));
        assert!(result.contains("truncated"));
    }

    // --- HTML parsing tests ---

    #[test]
    fn extract_html_title_basic() {
        let html = "<html><head><title>My Page</title></head><body></body></html>";
        assert_eq!(extract_html_title(html), Some("My Page".to_string()));
    }

    #[test]
    fn extract_html_title_case_insensitive() {
        let html = "<html><head><TITLE>My Page</TITLE></head><body></body></html>";
        assert_eq!(extract_html_title(html), Some("My Page".to_string()));
    }

    #[test]
    fn extract_html_title_missing() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(extract_html_title(html), None);
    }

    #[test]
    fn extract_html_title_empty() {
        let html = "<html><head><title> </title></head><body></body></html>";
        assert_eq!(extract_html_title(html), None);
    }

    #[test]
    fn extract_meta_description_basic() {
        let html = r#"<html><head><meta name="description" content="A test page"></head><body></body></html>"#;
        assert_eq!(
            extract_meta_description(html),
            Some("A test page".to_string())
        );
    }

    #[test]
    fn extract_meta_description_single_quotes() {
        let html = r#"<html><head><meta name='description' content='A test page'></head><body></body></html>"#;
        assert_eq!(
            extract_meta_description(html),
            Some("A test page".to_string())
        );
    }

    #[test]
    fn extract_meta_description_missing() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(extract_meta_description(html), None);
    }

    // --- HTML unescape tests ---

    #[test]
    fn html_unescape_entities() {
        assert_eq!(html_unescape("&amp;&lt;&gt;"), "&<>");
        assert_eq!(html_unescape("&quot;hello&quot;"), "\"hello\"");
    }

    // --- Strip HTML tags tests ---

    #[test]
    fn strip_html_tags_basic() {
        assert_eq!(strip_html_tags("<b>bold</b> text"), "bold text");
    }

    #[test]
    fn strip_html_tags_nested() {
        assert_eq!(strip_html_tags("<div><p>Hello</p></div>"), "Hello");
    }

    #[test]
    fn strip_html_tags_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    // --- CDATA tests ---

    #[test]
    fn strip_cdata_wrapper() {
        assert_eq!(
            strip_cdata("<![CDATA[Hello <b>world</b>]]>"),
            "Hello <b>world</b>"
        );
    }

    #[test]
    fn strip_cdata_no_wrapper() {
        assert_eq!(strip_cdata("plain text"), "plain text");
    }

    // --- XML tag extraction tests ---

    #[test]
    fn extract_xml_tag_basic() {
        let xml = "<item><title>My Title</title><link>http://example.com</link></item>";
        assert_eq!(extract_xml_tag(xml, "title"), Some("My Title".to_string()));
        assert_eq!(
            extract_xml_tag(xml, "link"),
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn extract_xml_tag_cdata() {
        let xml = "<item><title><![CDATA[My <b>Title</b>]]></title></item>";
        assert_eq!(
            extract_xml_tag(xml, "title"),
            Some("My <b>Title</b>".to_string())
        );
    }

    #[test]
    fn extract_xml_tag_missing() {
        let xml = "<item><title>Hello</title></item>";
        assert_eq!(extract_xml_tag(xml, "description"), None);
    }

    #[test]
    fn extract_xml_tag_with_attrs_test() {
        let xml = r#"<link rel="alternate" type="text/html">http://example.com</link>"#;
        let result = extract_xml_tag_with_attrs(xml, "link");
        assert_eq!(result, Some("http://example.com".to_string()));
    }

    // --- HTTP header parsing tests ---

    #[test]
    fn parse_http_headers_basic() {
        let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1234\r\n\r\n";
        let (status, cl, ct) = parse_http_headers(headers);
        assert_eq!(status, Some("200".to_string()));
        assert_eq!(cl, Some("1234".to_string()));
        assert_eq!(ct, Some("text/html".to_string()));
    }

    #[test]
    fn parse_http_headers_redirect() {
        let headers =
            "HTTP/1.1 301 Moved\r\nLocation: /new\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 42\r\n\r\n";
        let (status, cl, _) = parse_http_headers(headers);
        // Should get the last status line
        assert_eq!(status, Some("200".to_string()));
        assert_eq!(cl, Some("42".to_string()));
    }

    // --- HTTP response splitting tests ---

    #[test]
    fn split_http_response_basic() {
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let (headers, body) = split_http_response(resp);
        assert!(headers.contains("200 OK"));
        assert_eq!(body, "hello");
    }

    #[test]
    fn split_http_response_lf_only() {
        let resp = "HTTP/1.1 200 OK\nContent-Length: 5\n\nhello";
        let (headers, body) = split_http_response(resp);
        assert!(headers.contains("200 OK"));
        assert_eq!(body, "hello");
    }

    // --- DDG URL cleaning tests ---

    #[test]
    fn clean_ddg_url_direct() {
        assert_eq!(
            clean_ddg_url("https://example.com/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn clean_ddg_url_protocol_relative() {
        assert_eq!(
            clean_ddg_url("//example.com/page"),
            "https://example.com/page"
        );
    }

    #[test]
    fn clean_ddg_url_uddg() {
        assert_eq!(
            clean_ddg_url("//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&rut=abc"),
            "https://example.com"
        );
    }

    // --- DDG result parsing tests ---

    #[test]
    fn parse_ddg_results_empty() {
        let results = parse_ddg_results("", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_ddg_results_no_results_class() {
        let html = "<html><body><p>No results</p></body></html>";
        let results = parse_ddg_results(html, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_ddg_results_skips_ads() {
        let html = concat!(
            r#"<div class="result"><a class="result__a" "#,
            r#"href="https://duckduckgo.com/y.js?ad=1">Ad</a>"#,
            r#"<a class="result__snippet" href="https://duckduckgo.com/y.js?ad=1">Ad snippet</a></div>"#,
        );
        let results = parse_ddg_results(html, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_ddg_results_realistic() {
        let html = concat!(
            r#"<div class="result results_links">"#,
            r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org&amp;rut=abc">"#,
            "The Rust Programming Language</a>",
            r#"<a class="result__snippet" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org&amp;rut=abc">"#,
            "A language empowering everyone to build reliable and efficient software.</a>",
            "</div>",
            r#"<div class="result results_links">"#,
            r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fdoc.rust-lang.org&amp;rut=def">"#,
            "Rust Documentation</a>",
            r#"<a class="result__snippet" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fdoc.rust-lang.org&amp;rut=def">"#,
            "Learn Rust with examples and comprehensive guides.</a>",
            "</div>",
        );
        let results = parse_ddg_results(html, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "The Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org");
        assert_eq!(results[1].title, "Rust Documentation");
    }

    // --- X509 field extraction tests ---

    #[test]
    fn extract_x509_field_issuer() {
        let text = " Issuer: C = US, O = Let's Encrypt, CN = R3\n Subject: CN = example.com";
        assert_eq!(
            extract_x509_field(text, "Issuer:"),
            Some("C = US, O = Let's Encrypt, CN = R3".to_string())
        );
        assert_eq!(
            extract_x509_field(text, "Subject:"),
            Some("CN = example.com".to_string())
        );
    }

    #[test]
    fn extract_x509_field_missing() {
        let text = "some other text";
        assert_eq!(extract_x509_field(text, "Issuer:"), None);
    }

    // --- SAN extraction tests ---

    #[test]
    fn extract_san_list_basic() {
        let text =
            "X509v3 Subject Alternative Name: critical\n    DNS:example.com, DNS:www.example.com\n";
        let result = extract_san_list(text);
        assert!(result.contains("DNS:example.com"), "result: {result}");
        assert!(result.contains("DNS:www.example.com"), "result: {result}");
    }

    #[test]
    fn extract_san_list_none() {
        let text = "some text without SANs";
        let result = extract_san_list(text);
        assert!(result.contains("(none)"));
    }

    #[test]
    fn extract_san_list_with_ip() {
        let text = "X509v3 Subject Alternative Name:\n    DNS:example.com, IP Address:1.2.3.4\n";
        let result = extract_san_list(text);
        assert!(result.contains("DNS:example.com"), "result: {result}");
        assert!(result.contains("IP Address:1.2.3.4"), "result: {result}");
    }

    #[test]
    fn extract_san_list_inline() {
        let text = "X509v3 Subject Alternative Name: DNS:example.com, DNS:www.example.com\n";
        let result = extract_san_list(text);
        assert!(result.contains("DNS:example.com"), "result: {result}");
        assert!(result.contains("DNS:www.example.com"), "result: {result}");
    }

    // --- RSS parsing tests ---

    #[test]
    fn parse_rss_items_basic() {
        let xml = concat!(
            "<rss><channel><title>Test Feed</title>",
            "<item><title>Item 1</title><link>http://example.com/1</link>",
            "<description>Desc 1</description>",
            "<pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate></item>",
            "<item><title>Item 2</title><link>http://example.com/2</link>",
            "<description>Desc 2</description>",
            "<pubDate>Tue, 02 Jan 2024 00:00:00 GMT</pubDate></item>",
            "</channel></rss>",
        );
        let items = parse_rss_items(xml, 10);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Item 1");
        assert_eq!(items[0].link, "http://example.com/1");
        assert_eq!(items[1].title, "Item 2");
    }

    #[test]
    fn parse_rss_items_max() {
        let mut xml = String::from("<rss><channel><title>T</title>");
        for i in 0..20 {
            xml.push_str(&format!(
                "<item><title>Item {i}</title><link>http://example.com/{i}</link></item>"
            ));
        }
        xml.push_str("</channel></rss>");
        let items = parse_rss_items(&xml, 5);
        assert_eq!(items.len(), 5);
    }

    // --- Atom parsing tests ---

    #[test]
    fn parse_atom_entries_basic() {
        let xml = concat!(
            "<feed xmlns=\"http://www.w3.org/2005/Atom\">",
            "<entry>",
            "<title>Atom Entry 1</title>",
            "<link href=\"http://example.com/1\" rel=\"alternate\"/>",
            "<summary>Summary 1</summary>",
            "<updated>2024-01-01T00:00:00Z</updated>",
            "</entry>",
            "</feed>",
        );
        let items = parse_atom_entries(xml, 10);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Atom Entry 1");
        assert_eq!(items[0].link, "http://example.com/1");
        assert_eq!(items[0].pub_date, "2024-01-01T00:00:00Z");
    }

    // --- Atom link extraction tests ---

    #[test]
    fn extract_atom_link_alternate() {
        let block = r#"<link href="http://example.com/page" rel="alternate" type="text/html"/>"#;
        assert_eq!(extract_atom_link(block), "http://example.com/page");
    }

    #[test]
    fn extract_atom_link_no_alternate() {
        let block = concat!(
            r#"<link href="http://example.com/self" rel="self"/>"#,
            r#"<link href="http://example.com/page" type="text/html"/>"#,
        );
        assert_eq!(extract_atom_link(block), "http://example.com/page");
    }

    #[test]
    fn extract_href_attr_basic() {
        let s = r#"href="http://example.com" rel="alternate""#;
        assert_eq!(extract_href_attr(s), Some("http://example.com".to_string()));
    }

    // --- Work function contract tests (input validation) ---

    #[test]
    fn web_search_empty_query() {
        let result = web_search_work(String::new(), None, None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_search_unsupported_engine() {
        let result = web_search_work("test".to_string(), Some("google".to_string()), None);
        assert!(result.contains("unsupported"));
    }

    #[test]
    fn web_screenshot_empty_url() {
        let result = web_screenshot_work(String::new(), None, None, None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_rss_empty_url() {
        let result = web_rss_fetch_work(String::new(), None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_api_empty_url() {
        let result = web_api_call_work(String::new(), "GET".to_string(), None, None, None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_api_unsupported_method() {
        let result = web_api_call_work(
            "http://example.com".to_string(),
            "TRACE".to_string(),
            None,
            None,
            None,
        );
        assert!(result.contains("unsupported"));
    }

    #[test]
    fn web_dns_empty_domain() {
        let result = web_dns_lookup_work(String::new(), None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_dns_unsupported_record_type() {
        let result = web_dns_lookup_work("example.com".to_string(), Some("AXFR".to_string()));
        assert!(result.contains("unsupported"));
    }

    #[test]
    fn web_tls_empty_host() {
        let result = web_tls_inspect_work(String::new(), None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_download_empty_url() {
        let result = web_download_file_work(String::new(), PathBuf::from("/tmp/test.bin"), None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    #[test]
    fn web_download_empty_path() {
        let result =
            web_download_file_work("http://example.com/file".to_string(), PathBuf::new(), None);
        assert!(result.contains("Error") || result.contains("empty"));
    }

    // --- require_curl test ---

    #[test]
    fn require_curl_runs() {
        // This test just exercises the function — it may return Ok or Err
        // depending on the test environment
        let _ = require_curl();
    }

    // --- Split and strip edge cases ---

    #[test]
    fn strip_html_tags_br() {
        assert_eq!(strip_html_tags("line1<br>line2"), "line1 line2");
    }

    #[test]
    fn split_http_response_no_headers() {
        let (h, b) = split_http_response("just body text");
        assert_eq!(h, "");
        assert_eq!(b, "just body text");
    }
}
