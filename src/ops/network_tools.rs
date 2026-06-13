//! # Network Analysis & Protocol Tools
//!
//! ICMP ping, traceroute, port scanning, DNS, WHOIS, TLS inspection,
//! HTTP benchmarking, ARP/WiFi scanning, bandwidth testing, nmap,
//! SSH/SFTP, Wake-on-LAN, serial port I/O, Bluetooth, and MQTT.

use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::process::Command;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool_available(name: &str) -> bool {
    #[cfg(windows)]
    let res = Command::new("where").arg(name).output();
    #[cfg(not(windows))]
    let res = Command::new("which").arg(name).output();
    match res {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

fn error_tool_not_found(tool: &str) -> serde_json::Value {
    json!({"error": format!("{tool} not found"), "tool_available": false})
}

fn args_str<'a>(args: &'a serde_json::Value, key: &str) -> &'a str {
    args.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

fn args_u64(args: &serde_json::Value, key: &str, default: u64) -> u64 {
    args.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

// ---------------------------------------------------------------------------
// 1. net_ping_work
// ---------------------------------------------------------------------------

/// ICMP ping. Args: `host` (string), `count` (int, default 4).
/// Returns: {host, sent, received, packet_loss_pct, rtt_min/avg/max_ms}
pub fn net_ping_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let count = args_u64(&args, "count", 4).clamp(1, 100);

    if !tool_available("ping") {
        return error_tool_not_found("ping");
    }

    let output = match Command::new("ping")
        .args(["-c", &count.to_string(), &host])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run ping: {e}"), "tool_available": true})
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() && stdout.trim().is_empty() {
        return json!({
            "host": host,
            "sent": count,
            "received": 0,
            "packet_loss_pct": 100.0,
            "rtt_min_ms": null,
            "rtt_avg_ms": null,
            "rtt_max_ms": null,
            "stderr": stderr.trim().to_string(),
        });
    }

    // Parse packet loss from "4 packets transmitted, 4 received, 0% packet loss"
    let mut packet_loss_pct: f64 = 100.0;
    let mut sent: u64 = count;
    let mut received: u64 = 0;

    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if lower.contains("packets transmitted") {
            if let Some(tx_idx) = lower.find("packets transmitted") {
                let prefix = lower[..tx_idx].trim();
                if let Ok(n) = prefix
                    .split_whitespace()
                    .last()
                    .unwrap_or("0")
                    .parse::<u64>()
                {
                    sent = n;
                }
            }
            if let Some(rx_part) = lower.split(", ").nth(1) {
                if let Ok(n) = rx_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<u64>()
                {
                    received = n;
                }
            }
            if let Some(pct_idx) = lower.find("% packet loss") {
                let before = &lower[..pct_idx];
                if let Some(pct_str) = before.split_whitespace().last() {
                    if let Ok(p) = pct_str.parse::<f64>() {
                        packet_loss_pct = p;
                    }
                }
            }
        }
    }

    // Parse RTT from "round-trip min/avg/max/stddev = 0.034/0.045/0.067/0.012 ms"
    // or "rtt min/avg/max/mdev = 0.034/0.045/0.067/0.012 ms"
    let mut rtt_min: Option<f64> = None;
    let mut rtt_avg: Option<f64> = None;
    let mut rtt_max: Option<f64> = None;

    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if lower.contains("min/avg/max") && lower.contains("ms") {
            if let Some(eq_idx) = lower.find('=') {
                let after_eq = line[eq_idx + 1..].trim();
                let parts: Vec<&str> = after_eq.split('/').collect();
                if parts.len() >= 3 {
                    rtt_min = parts[0].trim().parse::<f64>().ok();
                    rtt_avg = parts[1].trim().parse::<f64>().ok();
                    rtt_max = parts[2]
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok());
                }
            }
        }
    }

    json!({
        "host": host,
        "sent": sent,
        "received": received,
        "packet_loss_pct": packet_loss_pct,
        "rtt_min_ms": rtt_min,
        "rtt_avg_ms": rtt_avg,
        "rtt_max_ms": rtt_max,
    })
}

// ---------------------------------------------------------------------------
// 2. net_traceroute_work
// ---------------------------------------------------------------------------

/// Traceroute. Args: `host` (string), `max_hops` (int, default 30).
/// Returns: {hops: [{hop_number, host, ip, latency_ms}]}
pub fn net_traceroute_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let max_hops = args_u64(&args, "max_hops", 30).clamp(1, 64) as u32;

    if !tool_available("traceroute") {
        return error_tool_not_found("traceroute");
    }

    let output = match Command::new("traceroute")
        .args(["-m", &max_hops.to_string(), &host])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run traceroute: {e}"), "tool_available": true})
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut hops = Vec::new();

    for line in stdout.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let hop_number = match parts[0].parse::<u32>() {
            Ok(n) => n,
            Err(_) => continue,
        };

        let mut host_name = "*".to_string();
        let mut ip_addr = "*".to_string();
        let mut latency: Option<f64> = None;

        // Check for "* * *" timeout lines
        if parts.len() >= 3 && parts[1] == "*" && parts[2] == "*" {
            hops.push(json!({
                "hop_number": hop_number,
                "host": host_name,
                "ip": ip_addr,
                "latency_ms": latency,
            }));
            continue;
        }

        if parts.len() >= 2 {
            host_name = parts[1].to_string();
            if parts.len() >= 3 && parts[2].starts_with('(') {
                // Format: hostname (ip) latency...
                ip_addr = parts[2].trim_matches('(').trim_matches(')').to_string();
                for p in &parts[3..] {
                    if p.ends_with("ms") {
                        latency = p.trim_end_matches("ms").parse::<f64>().ok();
                        break;
                    }
                }
            } else {
                // Format: ip latency...
                if host_name.contains('.') || host_name.contains(':') {
                    ip_addr = host_name.clone();
                }
                for p in &parts[2..] {
                    if p.ends_with("ms") {
                        latency = p.trim_end_matches("ms").parse::<f64>().ok();
                        break;
                    }
                }
            }
        }

        hops.push(json!({
            "hop_number": hop_number,
            "host": host_name,
            "ip": ip_addr,
            "latency_ms": latency,
        }));
    }

    json!({"hops": hops})
}

// ---------------------------------------------------------------------------
// 3. net_port_scan_work
// ---------------------------------------------------------------------------

/// TCP port scan. Args: `host`, `ports` (array of ints), `timeout_ms` (default 2000).
/// Returns: {host, ports: [{port, open, latency_ms}]}
pub fn net_port_scan_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let timeout_ms = args_u64(&args, "timeout_ms", 2000).clamp(100, 30000);

    let default_ports: Vec<u64> = vec![22, 80, 443, 3000, 5432, 6379, 8080, 8443];
    let ports: Vec<u64> = match args.get("ports") {
        Some(v) => v
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_u64())
                    .filter(|&p| p > 0 && p <= 65535)
                    .collect()
            })
            .unwrap_or(default_ports),
        None => default_ports,
    };

    if ports.is_empty() {
        return json!({"error": "ports list must not be empty"});
    }

    let mut results = Vec::new();
    for port in &ports {
        let target = format!("{host}:{port}");
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let addr: std::net::SocketAddr = match target.parse() {
            Ok(a) => a,
            Err(_) => {
                results.push(json!({"port": port, "open": false, "latency_ms": null}));
                continue;
            }
        };
        let result = TcpStream::connect_timeout(&addr, timeout);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(_stream) => {
                results.push(json!({
                    "port": port,
                    "open": true,
                    "latency_ms": (elapsed * 1000.0).round() / 1000.0,
                }));
            }
            Err(_) => {
                results.push(json!({"port": port, "open": false, "latency_ms": null}));
            }
        }
    }

    json!({"host": host, "ports": results})
}

// ---------------------------------------------------------------------------
// 4. net_dns_lookup_work
// ---------------------------------------------------------------------------

/// DNS lookup. Args: `domain`, `record_type` (default "A").
/// Returns: {domain, record_type, records: [strings]}
pub fn net_dns_lookup_work(args: serde_json::Value) -> serde_json::Value {
    let domain = match args.get("domain").and_then(|v| v.as_str()) {
        Some(d) if !d.is_empty() => d.to_string(),
        _ => return json!({"error": "domain is required"}),
    };
    let record_type = args_str(&args, "record_type").to_uppercase();
    let record_type = if record_type.is_empty() {
        "A".to_string()
    } else {
        record_type
    };

    let valid_types = ["A", "AAAA", "MX", "TXT", "CNAME", "NS", "SOA", "SRV", "PTR"];
    let record_type = if valid_types.contains(&record_type.as_str()) {
        record_type
    } else {
        "A".to_string()
    };

    if tool_available("dig") {
        let output = match Command::new("dig")
            .args(["+short", &domain, &record_type])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run dig: {e}"), "tool_available": true})
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let records: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        return json!({
            "domain": domain,
            "record_type": record_type,
            "records": records,
        });
    }

    if tool_available("nslookup") {
        let output = match Command::new("nslookup")
            .args([&format!("-type={record_type}"), &domain])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run nslookup: {e}"), "tool_available": true})
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut records = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with(';')
                || trimmed.starts_with("Server:")
                || trimmed.starts_with("Address:")
            {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                records.push(trimmed[eq_pos + 1..].trim().to_string());
            }
        }
        return json!({
            "domain": domain,
            "record_type": record_type,
            "records": records,
        });
    }

    error_tool_not_found("dig")
}

// ---------------------------------------------------------------------------
// 5. net_whois_work
// ---------------------------------------------------------------------------

/// WHOIS lookup. Args: `domain`.
/// Returns: {domain, registrar, raw_snippet (first 500 chars)}
pub fn net_whois_work(args: serde_json::Value) -> serde_json::Value {
    let domain = match args.get("domain").and_then(|v| v.as_str()) {
        Some(d) if !d.is_empty() => d.to_string(),
        _ => return json!({"error": "domain is required"}),
    };

    if !tool_available("whois") {
        return error_tool_not_found("whois");
    }

    let output = match Command::new("whois").arg(&domain).output() {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run whois: {e}"), "tool_available": true})
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return json!({
            "error": format!("whois exited with status {}", output.status),
            "stderr": stderr.chars().take(500).collect::<String>(),
            "tool_available": true,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let raw_snippet: String = stdout.chars().take(500).collect();

    let mut registrar = String::new();
    for line in stdout.lines() {
        let lower = line.to_lowercase();
        let trimmed = line.trim();
        if (lower.contains("registrar:") || lower.contains("sponsoring registrar"))
            && registrar.is_empty()
        {
            if let Some(colon_idx) = trimmed.find(':') {
                registrar = trimmed[colon_idx + 1..].trim().to_string();
            }
        }
    }

    json!({
        "domain": domain,
        "registrar": registrar,
        "raw_snippet": raw_snippet,
    })
}

// ---------------------------------------------------------------------------
// 6. net_tls_inspect_work
// ---------------------------------------------------------------------------

/// TLS certificate info. Args: `host`, `port` (default 443).
/// Returns: {subject, issuer, not_after, days_until_expiry, san: [strings]}
pub fn net_tls_inspect_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let port = args_u64(&args, "port", 443).clamp(1, 65535) as u16;

    if !tool_available("openssl") {
        return error_tool_not_found("openssl");
    }

    let connect_str = format!("{host}:{port}");

    // Step 1: Get raw cert from s_client
    let s_client_output = match Command::new("openssl")
        .args(["s_client", "-connect", &connect_str, "-servername", &host])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run openssl s_client: {e}"), "tool_available": true})
        }
    };

    // Step 2: Pipe cert to x509 -noout -text
    let x509_result = match Command::new("openssl")
        .args(["x509", "-noout", "-text"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(&s_client_output.stdout);
            }
            child.wait_with_output().ok()
        }
        Err(_) => None,
    };

    let stdout = match &x509_result {
        Some(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        None => return json!({"error": "failed to parse TLS certificate", "tool_available": true}),
    };

    let mut subject = String::new();
    let mut issuer = String::new();
    let mut not_after = String::new();
    let mut san: Vec<String> = Vec::new();
    let mut in_san = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("Issuer:") {
            issuer = stripped.trim().to_string();
        }
        if let Some(stripped) = trimmed.strip_prefix("Subject:") {
            if !trimmed.contains("Subject Alternative") {
                subject = stripped.trim().to_string();
            }
        }
        if trimmed.contains("Not After") {
            if let Some(colon) = trimmed.find(':') {
                not_after = trimmed[colon + 1..].trim().to_string();
            }
        }
        if trimmed.contains("Subject Alternative Name") {
            in_san = true;
            if let Some(colon) = trimmed.find(':') {
                let entries = trimmed[colon + 1..].trim();
                for entry in entries.split(',') {
                    let e = entry.trim().to_string();
                    if !e.is_empty() {
                        san.push(e);
                    }
                }
            }
            continue;
        }
        if in_san {
            if trimmed.is_empty() {
                in_san = false;
            } else {
                for entry in trimmed.split(',') {
                    let e = entry.trim().to_string();
                    if !e.is_empty() {
                        san.push(e);
                    }
                }
            }
        }
    }

    // Calculate days until expiry
    let days_until_expiry: Option<i64> = if !not_after.is_empty() {
        let now = chrono::Utc::now();
        chrono::DateTime::parse_from_str(&not_after, "%b %e %H:%M:%S %Y GMT")
            .or_else(|_| chrono::DateTime::parse_from_str(&not_after, "%Y-%m-%d %H:%M:%S %Z"))
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(&not_after))
            .map(|dt| (dt.with_timezone(&chrono::Utc) - now).num_days())
            .ok()
    } else {
        None
    };

    json!({
        "subject": subject,
        "issuer": issuer,
        "not_after": not_after,
        "days_until_expiry": days_until_expiry,
        "san": san,
    })
}

// ---------------------------------------------------------------------------
// 7. net_http_benchmark_work
// ---------------------------------------------------------------------------

/// HTTP load test. Args: `url`, `requests` (default 50), `concurrency` (default 4).
/// Returns: {rps, p50_ms, p95_ms, errors}
pub fn net_http_benchmark_work(args: serde_json::Value) -> serde_json::Value {
    let url = match args.get("url").and_then(|v| v.as_str()) {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => return json!({"error": "url is required"}),
    };
    let requests = args_u64(&args, "requests", 50).clamp(1, 10000) as usize;
    let concurrency = args_u64(&args, "concurrency", 4).clamp(1, 100) as usize;

    if tool_available("ab") {
        let output = match Command::new("ab")
            .args([
                "-n",
                &requests.to_string(),
                "-c",
                &concurrency.to_string(),
                &url,
            ])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run ab: {e}"), "tool_available": true})
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut rps: Option<f64> = None;
        let mut p50: Option<f64> = None;
        let mut p95: Option<f64> = None;
        let mut errors = 0u64;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Requests per second") {
                if let Some(colon) = trimmed.find(':') {
                    let rest = trimmed[colon + 1..].trim();
                    if let Some(part) = rest.split_whitespace().next() {
                        rps = part.parse::<f64>().ok();
                    }
                }
            }
            if trimmed.contains("50%") && p50.is_none() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    p50 = last.parse::<f64>().ok();
                }
            }
            if trimmed.contains("95%") && p95.is_none() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    p95 = last.parse::<f64>().ok();
                }
            }
            if trimmed.starts_with("Non-2xx responses") || trimmed.starts_with("Failed requests") {
                if let Some(colon) = trimmed.find(':') {
                    if let Ok(n) = trimmed[colon + 1..].trim().parse::<u64>() {
                        errors += n;
                    }
                }
            }
        }

        return json!({
            "rps": rps,
            "p50_ms": p50,
            "p95_ms": p95,
            "errors": errors,
        });
    }

    // Fallback: sequential curl loop
    if tool_available("curl") {
        let mut latencies = Vec::new();
        let mut errors = 0u64;

        for _ in 0..requests.min(50) {
            let start = Instant::now();
            let result = Command::new("curl")
                .args(["-o", "/dev/null", "-s", "-w", "%{http_code}", &url])
                .output();
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(o) => {
                    let code = String::from_utf8_lossy(&o.stdout);
                    if !code.starts_with('2') {
                        errors += 1;
                    }
                    latencies.push(elapsed_ms);
                }
                Err(_) => errors += 1,
            }
        }

        if latencies.is_empty() {
            return json!({"rps": null, "p50_ms": null, "p95_ms": null, "errors": errors});
        }

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p50_idx = (latencies.len() as f64 * 0.50) as usize;
        let p95_idx = (latencies.len() as f64 * 0.95) as usize;
        let total_time: f64 = latencies.iter().sum();
        let rps = if total_time > 0.0 {
            latencies.len() as f64 / (total_time / 1000.0)
        } else {
            0.0
        };

        return json!({
            "rps": (rps * 100.0).round() / 100.0,
            "p50_ms": (latencies[p50_idx.min(latencies.len() - 1)] * 100.0).round() / 100.0,
            "p95_ms": (latencies[p95_idx.min(latencies.len() - 1)] * 100.0).round() / 100.0,
            "errors": errors,
        });
    }

    error_tool_not_found("ab")
}

// ---------------------------------------------------------------------------
// 8. net_arp_scan_work
// ---------------------------------------------------------------------------

/// ARP scan. Runs `ip neigh show` or `arp -a`.
/// Returns: {hosts: [{ip, mac, interface, state}]}
#[allow(dead_code)]
pub fn net_arp_scan_work(_args: serde_json::Value) -> serde_json::Value {
    if tool_available("ip") {
        let output = match Command::new("ip").args(["neigh", "show"]).output() {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run ip neigh: {e}"), "tool_available": true})
            }
        };
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return json!({
                "error": format!("ip neigh exited with status {}", output.status),
                "stderr": stderr.chars().take(500).collect::<String>(),
                "tool_available": true,
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut hosts = Vec::new();
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ip = parts[0].to_string();
                // Find lladdr (MAC) and dev (interface)
                let mut mac = String::new();
                let mut interface = String::new();
                let mut state = String::new();
                for i in 0..parts.len() {
                    if parts[i] == "lladdr" && i + 1 < parts.len() {
                        mac = parts[i + 1].to_string();
                    }
                    if parts[i] == "dev" && i + 1 < parts.len() {
                        interface = parts[i + 1].to_string();
                    }
                }
                // State is usually the last token
                if let Some(last) = parts.last() {
                    let l = *last;
                    if l == "REACHABLE"
                        || l == "STALE"
                        || l == "DELAY"
                        || l == "PROBE"
                        || l == "PERMANENT"
                        || l == "FAILED"
                        || l == "INCOMPLETE"
                    {
                        state = l.to_string();
                    }
                }
                hosts.push(json!({
                    "ip": ip,
                    "mac": mac,
                    "interface": interface,
                    "state": state,
                }));
            }
        }
        return json!({"hosts": hosts});
    }

    if tool_available("arp") {
        let output = match Command::new("arp").args(["-a"]).output() {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run arp: {e}"), "tool_available": true})
            }
        };
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return json!({
                "error": format!("arp exited with status {}", output.status),
                "stderr": stderr.chars().take(500).collect::<String>(),
                "tool_available": true,
            });
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut hosts = Vec::new();
        for line in stdout.lines() {
            // Format: "? (192.168.1.1) at aa:bb:cc:dd:ee:ff [ether] on eth0"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let ip = parts[1].trim_matches('(').trim_matches(')').to_string();
                let mac = parts[3].to_string();
                let interface = parts[6].to_string();
                hosts.push(json!({
                    "ip": ip,
                    "mac": mac,
                    "interface": interface,
                    "state": "",
                }));
            }
        }
        return json!({"hosts": hosts});
    }

    error_tool_not_found("ip")
}

// ---------------------------------------------------------------------------
// 9. net_wifi_scan_work
// ---------------------------------------------------------------------------

/// WiFi scan (Linux). Args: `interface` (string, optional, default "wlan0").
/// Tries `nmcli -t -f SSID,SIGNAL,SECURITY dev wifi list` first,
/// falls back to `iw dev <interface> scan`.
/// Returns: {networks: [{ssid, signal, security}], count: N}
#[allow(dead_code)]
pub fn net_wifi_scan_work(args: serde_json::Value) -> serde_json::Value {
    let interface = match args.get("interface").and_then(|v| v.as_str()) {
        Some(i) if !i.is_empty() => i.to_string(),
        _ => "wlan0".to_string(),
    };

    // Try nmcli first (terse mode for reliable parsing)
    if tool_available("nmcli") {
        let output = match Command::new("nmcli")
            .args(["-t", "-f", "SSID,SIGNAL,SECURITY", "dev", "wifi", "list"])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run nmcli: {e}"), "tool_available": true})
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Terse format: SSID:SIGNAL:SECURITY (colon-separated)
            // SSID may contain colons, so we split from the right
            let parts: Vec<&str> = trimmed.rsplitn(3, ':').collect();
            if parts.len() < 3 {
                continue;
            }
            // rsplitn gives reversed order: [security, signal, ssid_rest]
            let security = parts[0].to_string();
            let signal: i64 = parts[1].parse().unwrap_or(0);
            let ssid = parts[2].to_string();
            if ssid.is_empty() {
                continue;
            }
            networks.push(json!({
                "ssid": ssid,
                "signal": signal,
                "security": security,
            }));
        }
        let count = networks.len();
        return json!({"networks": networks, "count": count});
    }

    // Fallback: iw dev <interface> scan
    if tool_available("iw") {
        let output = match Command::new("iw")
            .args(["dev", &interface, "scan"])
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                return json!({"error": format!("failed to run iw: {e}"), "tool_available": true})
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();
        let mut ssid = String::new();
        let mut signal: i64 = 0;
        let mut security = String::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("BSS") {
                // Push previous entry if we have an SSID
                if !ssid.is_empty() {
                    networks.push(json!({
                        "ssid": ssid,
                        "signal": signal,
                        "security": security,
                    }));
                }
                ssid = String::new();
                signal = 0;
                security = String::new();
            } else if trimmed.starts_with("SSID:") {
                ssid = trimmed.trim_start_matches("SSID:").trim().to_string();
            } else if trimmed.starts_with("signal:") {
                // e.g. "signal: -67.00 dBm"
                let sig_str = trimmed.trim_start_matches("signal:").trim();
                signal = sig_str
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|v| v as i64)
                    .unwrap_or(0);
            } else if trimmed.contains("WPA") || trimmed.contains("RSN") || trimmed.contains("WEP")
            {
                security = trimmed.to_string();
            }
        }
        // Push last entry
        if !ssid.is_empty() {
            networks.push(json!({
                "ssid": ssid,
                "signal": signal,
                "security": security,
            }));
        }
        let count = networks.len();
        return json!({"networks": networks, "count": count});
    }

    error_tool_not_found("nmcli")
}

// ---------------------------------------------------------------------------
// 10. net_bandwidth_test_work
// ---------------------------------------------------------------------------

/// Quick bandwidth test. Args: `url` (string, optional, default
/// "http://speedtest.tele2.net/10MB.zip"), `timeout_secs` (int, default 30).
/// Returns: {download_bytes_per_sec: N, details: "..."}
#[allow(dead_code)]
pub fn net_bandwidth_test_work(args: serde_json::Value) -> serde_json::Value {
    let url = match args.get("url").and_then(|v| v.as_str()) {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => "http://speedtest.tele2.net/10MB.zip".to_string(),
    };
    let timeout_secs = args_u64(&args, "timeout_secs", 30).clamp(5, 120);
    if !tool_available("curl") {
        return error_tool_not_found("curl");
    }
    let output = match Command::new("curl")
        .args([
            "-o",
            "/dev/null",
            "-s",
            "-w",
            "%{speed_download}",
            "--connect-timeout",
            &timeout_secs.to_string(),
            "--max-time",
            &timeout_secs.to_string(),
            &url,
        ])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run curl: {e}"), "tool_available": true})
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let speed_bytes: f64 = stdout.trim().parse::<f64>().unwrap_or(0.0);
    if !output.status.success() && speed_bytes == 0.0 {
        return json!({
            "download_bytes_per_sec": 0,
            "details": format!("curl exited with status {:?}: {}", output.status.code(), stderr.trim()),
        });
    }
    let throughput_mbps = (speed_bytes * 8.0) / 1_000_000.0;
    json!({
        "download_bytes_per_sec": speed_bytes as u64,
        "details": format!("{:.2} Mbps download from {}", throughput_mbps, url),
    })
}

// ---------------------------------------------------------------------------
// 11. net_nmap_quick_work
// ---------------------------------------------------------------------------

/// nmap scan. Args: `host` (required), `ports` (string, optional, default "1-1000"),
/// `timeout_secs` (int, optional, default 30).
/// Returns: {host: "...", ports: [{port, state, service}], details: "..."}
#[allow(dead_code)]
pub fn net_nmap_quick_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let ports = match args.get("ports").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => "1-1000".to_string(),
    };
    let timeout_secs = args_u64(&args, "timeout_secs", 30).clamp(5, 300);
    if !tool_available("nmap") {
        return error_tool_not_found("nmap");
    }
    let result = Command::new("nmap")
        .args([
            "-sV",
            "-p",
            &ports,
            &host,
            "--host-timeout",
            &format!("{timeout_secs}s"),
        ])
        .output();

    let output = match result {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run nmap: {e}"), "tool_available": true})
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut port_list = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        // Typical: "22/tcp open ssh OpenSSH 8.9"
        // or: "80/tcp closed http"
        // or: "443/tcp filtered https"
        if trimmed.contains("/tcp") || trimmed.contains("/udp") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let port_proto = parts[0];
                let port_str = port_proto.split('/').next().unwrap_or("0");
                let port: u64 = port_str.parse().unwrap_or(0);
                let state = parts[1].to_string();
                let service = parts[2].to_string();
                if port > 0 {
                    port_list.push(json!({
                        "port": port,
                        "state": state,
                        "service": service,
                    }));
                }
            }
        }
    }
    let details = if port_list.is_empty() && !stderr.trim().is_empty() {
        format!(
            "nmap exited with status {:?}: {}",
            output.status.code(),
            stderr.trim()
        )
    } else {
        format!(
            "nmap -sV -p {} {} completed, {} ports found",
            ports,
            host,
            port_list.len()
        )
    };
    json!({
        "host": host,
        "ports": port_list,
        "details": details,
    })
}

// ---------------------------------------------------------------------------
// 12. net_ssh_exec_work
// ---------------------------------------------------------------------------

/// SSH exec. Args: `host` (required), `command` (required),
/// `user` (optional), `key_path` (optional), `port` (default 22).
/// Runs `ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 ...`.
/// Returns: {host, exit_code, stdout, stderr}
#[allow(dead_code)]
pub fn net_ssh_exec_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let command = match args.get("command").and_then(|v| v.as_str()) {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => return json!({"error": "command is required"}),
    };
    let user = args_str(&args, "user").to_string();
    let key_path = args_str(&args, "key_path").to_string();
    let port = args_u64(&args, "port", 22).clamp(1, 65535) as u16;
    if !tool_available("ssh") {
        return error_tool_not_found("ssh");
    }
    // Build target: user@host or just host
    let target = if user.is_empty() {
        host.clone()
    } else {
        format!("{user}@{host}")
    };
    // Build SSH arguments
    let mut ssh_args: Vec<String> = vec![
        "-p".into(),
        port.to_string(),
        "-o".into(),
        "StrictHostKeyChecking=no".into(),
        "-o".into(),
        "ConnectTimeout=10".into(),
    ];
    if !key_path.is_empty() {
        ssh_args.push("-i".into());
        ssh_args.push(key_path);
    }
    ssh_args.push(target.clone());
    ssh_args.push(command);
    let output = match Command::new("ssh").args(&ssh_args).output() {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run ssh: {e}"), "tool_available": true})
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    json!({
        "host": host,
        "exit_code": exit_code,
        "stdout": stdout,
        "stderr": stderr,
    })
}

// ---------------------------------------------------------------------------
// 13. net_sftp_upload_work
// ---------------------------------------------------------------------------

/// Upload via SCP. Args: `local_path`, `remote_path`, `host`, `user`, `port` (default 22).
/// Returns: {success: bool}
pub fn net_sftp_upload_work(args: serde_json::Value) -> serde_json::Value {
    let local_path = match args.get("local_path").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "local_path is required"}),
    };
    let remote_path = match args.get("remote_path").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "remote_path is required"}),
    };
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let user = match args.get("user").and_then(|v| v.as_str()) {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => return json!({"error": "user is required"}),
    };
    let port = args_u64(&args, "port", 22).clamp(1, 65535) as u16;

    if !tool_available("scp") {
        return error_tool_not_found("scp");
    }

    let target = format!("{user}@{host}:{remote_path}");
    let result = Command::new("scp")
        .args([
            "-P",
            &port.to_string(),
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=10",
            &local_path,
            &target,
        ])
        .output();

    match result {
        Ok(o) => json!({"success": o.status.success()}),
        Err(_) => json!({"success": false}),
    }
}

// ---------------------------------------------------------------------------
// 14. net_sftp_download_work
// ---------------------------------------------------------------------------

/// Download via SCP. Args: `remote_path`, `local_path`, `host`, `user`, `port` (default 22).
/// Returns: {success: bool}
pub fn net_sftp_download_work(args: serde_json::Value) -> serde_json::Value {
    let remote_path = match args.get("remote_path").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "remote_path is required"}),
    };
    let local_path = match args.get("local_path").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "local_path is required"}),
    };
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let user = match args.get("user").and_then(|v| v.as_str()) {
        Some(u) if !u.is_empty() => u.to_string(),
        _ => return json!({"error": "user is required"}),
    };
    let port = args_u64(&args, "port", 22).clamp(1, 65535) as u16;

    if !tool_available("scp") {
        return error_tool_not_found("scp");
    }

    let source = format!("{user}@{host}:{remote_path}");
    let result = Command::new("scp")
        .args([
            "-P",
            &port.to_string(),
            "-o",
            "StrictHostKeyChecking=accept-new",
            "-o",
            "ConnectTimeout=10",
            &source,
            &local_path,
        ])
        .output();

    match result {
        Ok(o) => json!({"success": o.status.success()}),
        Err(_) => json!({"success": false}),
    }
}

// ---------------------------------------------------------------------------
// 15. net_wake_on_lan_work
// ---------------------------------------------------------------------------

/// Send WOL magic packet. Args: `mac` (string, required),
/// `broadcast_ip` (string, optional, default "255.255.255.255"),
/// `port` (int, default 9).
/// Returns: {sent: bool, mac: "...", details: "..."}
#[allow(dead_code)]
pub fn net_wake_on_lan_work(args: serde_json::Value) -> serde_json::Value {
    let mac = match args.get("mac").and_then(|v| v.as_str()) {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => return json!({"error": "mac is required"}),
    };
    let broadcast_ip = match args.get("broadcast_ip").and_then(|v| v.as_str()) {
        Some(b) if !b.is_empty() => b.to_string(),
        _ => "255.255.255.255".to_string(),
    };
    let port = args_u64(&args, "port", 9).clamp(1, 65535) as u16;

    // Parse MAC address: "aa:bb:cc:dd:ee:ff" or "aa-bb-cc-dd-ee-ff"
    let mac_clean = mac.replace('-', ":");
    let octets: Vec<u8> = mac_clean
        .split(':')
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if octets.len() != 6 {
        return json!({"error": "invalid MAC address format, expected aa:bb:cc:dd:ee:ff"});
    }

    // Build magic packet: 6 bytes of 0xFF + MAC repeated 16 times
    let mut packet = vec![0xFFu8; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&octets);
    }

    // Send via UDP broadcast
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            return json!({"sent": false, "mac": mac, "details": format!("socket bind failed: {e}")})
        }
    };
    if socket.set_broadcast(true).is_err() {
        return json!({"sent": false, "mac": mac, "details": "failed to enable broadcast"});
    }
    let addr = format!("{broadcast_ip}:{port}");
    let sent = match socket.send_to(&packet, &addr) {
        Ok(n) => n > 0,
        Err(e) => {
            // Try subnet broadcast as fallback if custom broadcast_ip failed
            if broadcast_ip != "255.255.255.255" {
                let fallback = format!("255.255.255.255:{port}");
                match socket.send_to(&packet, &fallback) {
                    Ok(n2) => n2 > 0,
                    Err(e2) => {
                        return json!({"sent": false, "mac": mac, "details": format!("send to {broadcast_ip}:{port} failed: {e}; fallback also failed: {e2}")});
                    }
                }
            } else {
                // Try subnet broadcast as fallback
                match socket.send_to(&packet, format!("192.168.1.255:{port}")) {
                    Ok(n2) => n2 > 0,
                    Err(e2) => {
                        return json!({"sent": false, "mac": mac, "details": format!("send to {broadcast_ip}:{port} failed: {e}; subnet fallback also failed: {e2}")});
                    }
                }
            }
        }
    };
    let details = if sent {
        format!("WOL magic packet sent to {mac} via {broadcast_ip}:{port}")
    } else {
        format!("Failed to send WOL magic packet to {mac} via {broadcast_ip}:{port}")
    };
    json!({"sent": sent, "mac": mac, "details": details})
}

// ---------------------------------------------------------------------------
// 16. net_serial_list_work
// ---------------------------------------------------------------------------

/// List serial ports. Reads /dev/ttyUSB*, /dev/ttyACM*.
/// Returns: {ports: [{path}]}
#[allow(dead_code)]
pub fn net_serial_list_work(_args: serde_json::Value) -> serde_json::Value {
    let mut ports = Vec::new();

    let prefixes = ["/dev/ttyUSB", "/dev/ttyACM", "/dev/ttyS", "/dev/ttyAMA"];
    for prefix in &prefixes {
        let mut idx: u32 = 0;
        while idx < 256 {
            let path = format!("{prefix}{idx}");
            if std::path::Path::new(&path).exists() {
                ports.push(json!({"path": path}));
            }
            idx += 1;
        }
    }

    // Also try ls if available (for any unusual naming)
    if tool_available("ls") {
        if let Ok(output) = Command::new("sh")
            .args(["-c", "ls /dev/ttyUSB* /dev/ttyACM* 2>/dev/null"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !ports.iter().any(|p| p["path"] == trimmed) {
                    ports.push(json!({"path": trimmed}));
                }
            }
        }
    }

    let count = ports.len();
    json!({"ports": ports, "count": count})
}

// ---------------------------------------------------------------------------
// 17. net_serial_send_work
// ---------------------------------------------------------------------------

/// Send to serial port. Args: `port`, `data`, `baud_rate` (default 9600).
/// Returns: {bytes_sent}
#[allow(dead_code)]
pub fn net_serial_send_work(args: serde_json::Value) -> serde_json::Value {
    let port = match args.get("port").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "port is required"}),
    };
    let data = match args.get("data").and_then(|v| v.as_str()) {
        Some(d) => d.to_string(),
        None => return json!({"error": "data is required"}),
    };
    let baud_rate = args_u64(&args, "baud_rate", 9600);

    if !tool_available("stty") {
        return error_tool_not_found("stty");
    }

    // Configure baud rate
    let _ = Command::new("stty")
        .args(["-F", &port, &baud_rate.to_string(), "raw", "-echo"])
        .output();

    // Write data
    let bytes_sent = match std::fs::OpenOptions::new().write(true).open(&port) {
        Ok(mut file) => match file.write(data.as_bytes()) {
            Ok(n) => n as u64,
            Err(e) => return json!({"error": format!("write failed: {e}"), "bytes_sent": 0}),
        },
        Err(e) => return json!({"error": format!("open {} failed: {e}", port), "bytes_sent": 0}),
    };

    json!({"bytes_sent": bytes_sent})
}

// ---------------------------------------------------------------------------
// 18. net_serial_read_work
// ---------------------------------------------------------------------------

/// Read from serial port. Args: `port`, `baud_rate` (default 9600), `timeout_ms` (default 3000).
/// Returns: {data, bytes_read}
#[allow(dead_code)]
pub fn net_serial_read_work(args: serde_json::Value) -> serde_json::Value {
    let port = match args.get("port").and_then(|v| v.as_str()) {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => return json!({"error": "port is required"}),
    };
    let baud_rate = args_u64(&args, "baud_rate", 9600);
    let timeout_ms = args_u64(&args, "timeout_ms", 3000).clamp(100, 30000);

    if !tool_available("stty") {
        return error_tool_not_found("stty");
    }

    // Configure baud rate
    let _ = Command::new("stty")
        .args(["-F", &port, &baud_rate.to_string(), "raw", "-echo"])
        .output();

    // Read data with timeout
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    let mut buf = vec![0u8; 4096];
    let mut total_read = 0usize;
    let mut data = Vec::new();

    match std::fs::File::open(&port) {
        Ok(mut file) => {
            // Set non-blocking isn't straightforward, so we use a simple
            // read loop with elapsed time check
            while start.elapsed() < timeout {
                match file.read(&mut buf[total_read..]) {
                    Ok(0) => {
                        // EOF or no data; small sleep then retry
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    Ok(n) => {
                        total_read += n;
                        data.extend_from_slice(&buf[total_read - n..total_read]);
                        if total_read >= buf.len() {
                            break;
                        }
                    }
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        }
        Err(e) => {
            return json!({"error": format!("open {} failed: {e}", port), "data": "", "bytes_read": 0})
        }
    }

    let data_str = String::from_utf8_lossy(&data[..total_read]).to_string();
    json!({
        "data": data_str,
        "bytes_read": total_read,
    })
}

// ---------------------------------------------------------------------------
// 19. net_bluetooth_scan_work
// ---------------------------------------------------------------------------

/// BT scan (Linux). Runs `bluetoothctl scan on` briefly.
/// Returns: {devices: [{mac, name}]}
#[allow(dead_code)]
pub fn net_bluetooth_scan_work(_args: serde_json::Value) -> serde_json::Value {
    if !tool_available("bluetoothctl") {
        return error_tool_not_found("bluetoothctl");
    }

    // Start scan briefly, then read devices
    let _ = Command::new("bluetoothctl").args(["scan", "on"]).output();

    // Give a moment for discovery
    std::thread::sleep(Duration::from_secs(3));

    // Stop scan
    let _ = Command::new("bluetoothctl").args(["scan", "off"]).output();

    // Get devices
    let output = match Command::new("bluetoothctl").args(["devices"]).output() {
        Ok(o) => o,
        Err(e) => {
            return json!({"error": format!("failed to run bluetoothctl: {e}"), "tool_available": true})
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        // Format: "Device AA:BB:CC:DD:EE:FF Device Name"
        let trimmed = line.trim();
        if !trimmed.starts_with("Device") {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
        if parts.len() >= 3 {
            devices.push(json!({
                "mac": parts[1].to_string(),
                "name": parts[2].to_string(),
            }));
        } else if parts.len() == 2 {
            devices.push(json!({
                "mac": parts[1].to_string(),
                "name": "",
            }));
        }
    }

    json!({"devices": devices})
}

// ---------------------------------------------------------------------------
// 20. net_mqtt_publish_work
// ---------------------------------------------------------------------------

/// Publish to MQTT. Args: `broker`, `topic`, `message`, `timeout_secs` (int, default 10).
/// Returns: {published: bool, broker: "...", topic: "..."}
#[allow(dead_code)]
pub fn net_mqtt_publish_work(args: serde_json::Value) -> serde_json::Value {
    let broker = match args.get("broker").and_then(|v| v.as_str()) {
        Some(b) if !b.is_empty() => b.to_string(),
        _ => return json!({"error": "broker is required"}),
    };
    let topic = match args.get("topic").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return json!({"error": "topic is required"}),
    };
    let message = match args.get("message").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return json!({"error": "message is required"}),
    };
    let timeout_secs = args_u64(&args, "timeout_secs", 10).clamp(1, 120);
    if !tool_available("mosquitto_pub") {
        return error_tool_not_found("mosquitto_pub");
    }
    let result = Command::new("mosquitto_pub")
        .args([
            "-h",
            &broker,
            "-t",
            &topic,
            "-m",
            &message,
            "-W",
            &timeout_secs.to_string(),
        ])
        .output();
    match result {
        Ok(o) => json!({
            "published": o.status.success(),
            "broker": broker,
            "topic": topic,
        }),
        Err(e) => json!({
            "published": false,
            "broker": broker,
            "topic": topic,
            "details": format!("mosquitto_pub failed: {e}"),
        }),
    }
}

// ---------------------------------------------------------------------------
// 21. net_mqtt_subscribe_work
// ---------------------------------------------------------------------------

/// Subscribe to MQTT. Args: `broker`, `topic`, `timeout_secs` (default 5).
/// Runs `mosquitto_sub -h <broker> -t <topic> -C 10 -W <timeout>`.
/// Returns: {messages: [strings], count: N}
#[allow(dead_code)]
pub fn net_mqtt_subscribe_work(args: serde_json::Value) -> serde_json::Value {
    let broker = match args.get("broker").and_then(|v| v.as_str()) {
        Some(b) if !b.is_empty() => b.to_string(),
        _ => return json!({"error": "broker is required"}),
    };
    let topic = match args.get("topic").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => return json!({"error": "topic is required"}),
    };
    let timeout_secs = args_u64(&args, "timeout_secs", 5).clamp(1, 300) as u32;

    if !tool_available("mosquitto_sub") {
        return error_tool_not_found("mosquitto_sub");
    }

    let result = Command::new("mosquitto_sub")
        .args([
            "-h",
            &broker,
            "-t",
            &topic,
            "-C",
            "10",
            "-W",
            &timeout_secs.to_string(),
        ])
        .output();

    match result {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let messages: Vec<String> = stdout
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.is_empty())
                .collect();
            let count = messages.len();
            json!({"messages": messages, "count": count})
        }
        Err(_) => json!({"messages": [], "count": 0}),
    }
}

// ---------------------------------------------------------------------------
// 22. net_packet_capture_work
// ---------------------------------------------------------------------------

/// Raw packet capture using `tcpdump` CLI with timeout/count limits.
///
/// Args: `interface` (string, required), `count` (int, default 100),
/// `timeout_secs` (int, default 10, max 60), `filter` (string, optional BPF).
///
/// Returns: `{interface, count, timeout_secs, filter, exit_code, output, tool_available}`.
#[allow(dead_code)]
pub fn net_packet_capture_work(args: serde_json::Value) -> serde_json::Value {
    let interface = match args.get("interface").and_then(|v| v.as_str()) {
        Some(i) if !i.is_empty() => i.to_string(),
        _ => return json!({"error": "interface is required"}),
    };
    let count = args_u64(&args, "count", 100).clamp(1, 10_000);
    let timeout_secs = args_u64(&args, "timeout_secs", 10).clamp(1, 60);
    let filter = args_str(&args, "filter").to_string();

    if !tool_available("tcpdump") {
        return error_tool_not_found("tcpdump");
    }

    let mut cmd = Command::new("tcpdump");
    cmd.args([
        "-i",
        &interface,
        "-c",
        &count.to_string(),
        "-w",
        &format!("/tmp/pb-pcap-{}-{}.pcap", interface, std::process::id()),
        "-W",
        &timeout_secs.to_string(),
        "-G",
        &timeout_secs.to_string(),
    ]);
    if !filter.is_empty() {
        cmd.arg(&filter);
    }

    let result = cmd.output();
    match result {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let exit_code = o.status.code().unwrap_or(-1);
            json!({
                "interface": interface,
                "count": count,
                "timeout_secs": timeout_secs,
                "filter": filter,
                "exit_code": exit_code,
                "output": if stdout.trim().is_empty() { stderr.trim() } else { stdout.trim() },
                "tool_available": true,
            })
        }
        Err(e) => json!({"error": format!("failed to run tcpdump: {e}"), "tool_available": true}),
    }
}

// ---------------------------------------------------------------------------
// 23. net_vpn_status_work
// ---------------------------------------------------------------------------

/// WireGuard / OpenVPN status check via CLI commands.
///
/// Args: `vpn_type` (string, optional — "wireguard", "openvpn", or "all", default "all").
///
/// Returns: `{wireguard: {...}, openvpn: {...}, tool_available}`.
#[allow(dead_code)]
pub fn net_vpn_status_work(args: serde_json::Value) -> serde_json::Value {
    let vpn_type = args_str(&args, "vpn_type").to_string();
    let check_all = vpn_type.is_empty() || vpn_type == "all";

    let mut wireguard = json!({"status": "not_checked"});
    let mut openvpn = json!({"status": "not_checked"});

    if check_all || vpn_type == "wireguard" {
        if tool_available("wg") {
            let result = Command::new("wg").arg("show").output();
            match result {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                    if o.status.success() && !stdout.trim().is_empty() {
                        wireguard = json!({"status": "active", "details": stdout.trim()});
                    } else {
                        wireguard = json!({"status": "inactive", "details": if stdout.trim().is_empty() { stderr.trim() } else { stdout.trim() }});
                    }
                }
                Err(e) => {
                    wireguard =
                        json!({"status": "error", "error": format!("failed to run wg: {e}")});
                }
            }
        } else {
            wireguard = json!({"status": "unavailable", "error": "wg not found"});
        }
    }

    if check_all || vpn_type == "openvpn" {
        if tool_available("systemctl") {
            let result = Command::new("systemctl")
                .args(["is-active", "openvpn"])
                .output();
            match result {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if stdout == "active" {
                        let sessions_result = Command::new("systemctl")
                            .args([
                                "list-units",
                                "--type=service",
                                "--state=running",
                                "openvpn*",
                            ])
                            .output();
                        let details = match sessions_result {
                            Ok(s) => String::from_utf8_lossy(&s.stdout).trim().to_string(),
                            Err(_) => "could not list sessions".to_string(),
                        };
                        openvpn = json!({"status": "active", "details": details});
                    } else {
                        openvpn =
                            json!({"status": stdout, "details": "openvpn service not active"});
                    }
                }
                Err(e) => {
                    openvpn = json!({"status": "error", "error": format!("failed to run systemctl: {e}")});
                }
            }
        } else {
            openvpn = json!({"status": "unavailable", "error": "systemctl not found"});
        }
    }

    json!({
        "wireguard": wireguard,
        "openvpn": openvpn,
        "tool_available": true,
    })
}

// ---------------------------------------------------------------------------
// 24. net_dns_zone_transfer_work
// ---------------------------------------------------------------------------

/// AXFR zone transfer test using `dig` CLI.
///
/// Args: `domain` (string, required), `nameserver` (string, optional).
///
/// Returns: `{domain, nameserver, transferred, records, details, tool_available}`.
#[allow(dead_code)]
pub fn net_dns_zone_transfer_work(args: serde_json::Value) -> serde_json::Value {
    let domain = match args.get("domain").and_then(|v| v.as_str()) {
        Some(d) if !d.is_empty() => d.to_string(),
        _ => return json!({"error": "domain is required"}),
    };
    let nameserver = args_str(&args, "nameserver").to_string();

    if !tool_available("dig") {
        return error_tool_not_found("dig");
    }

    let mut cmd = Command::new("dig");
    if nameserver.is_empty() {
        cmd.args(["AXFR", &domain]);
    } else {
        cmd.args(["AXFR", &domain, &format!("@{nameserver}")]);
    }
    cmd.args(["+timeout=10", "+tries=1"]);

    let result = cmd.output();
    match result {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let exit_code = o.status.code().unwrap_or(-1);

            let records: Vec<String> = stdout
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with(';') && !l.starts_with(";;"))
                .map(|l| l.trim().to_string())
                .take(500)
                .collect();

            let transferred = !records.is_empty();
            let details = if transferred {
                format!(
                    "AXFR zone transfer succeeded, {} records retrieved",
                    records.len()
                )
            } else if stdout.contains("XFR") && stdout.contains("refused") {
                "AXFR zone transfer refused by server".to_string()
            } else if exit_code != 0 {
                format!("dig exited with code {}: {}", exit_code, stderr.trim())
            } else {
                "AXFR zone transfer returned no records (likely refused or not authoritative)"
                    .to_string()
            };

            json!({
                "domain": domain,
                "nameserver": if nameserver.is_empty() { "default" } else { &nameserver },
                "transferred": transferred,
                "records": records,
                "details": details,
                "tool_available": true,
            })
        }
        Err(e) => json!({"error": format!("failed to run dig: {e}"), "tool_available": true}),
    }
}

// ---------------------------------------------------------------------------
// 25. net_ssl_grade_work
// ---------------------------------------------------------------------------

/// TLS configuration grading using `sslscan` or `testssl.sh` CLI.
///
/// Args: `host` (string, required), `port` (int, default 443).
///
/// Returns: `{host, port, grade, tls_versions, strong_ciphers_count, weak_ciphers_count, details, tool_available}`.
#[allow(dead_code)]
pub fn net_ssl_grade_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let port = args_u64(&args, "port", 443).clamp(1, 65535) as u16;
    let target = format!("{host}:{port}");

    // Try sslscan first
    if tool_available("sslscan") {
        let result = Command::new("sslscan").arg(&target).output();
        match result {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                let exit_code = o.status.code().unwrap_or(-1);

                let mut tls_versions: Vec<String> = Vec::new();
                let mut weak_ciphers: Vec<String> = Vec::new();
                let mut strong_ciphers: Vec<String> = Vec::new();

                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if (trimmed.starts_with("TLS") || trimmed.starts_with("SSL"))
                        && (trimmed.contains("enabled") || trimmed.contains("Supported"))
                    {
                        if let Some(version) = trimmed.split_whitespace().next() {
                            tls_versions.push(version.to_string());
                        }
                    }
                    if trimmed.contains("Weak") {
                        weak_ciphers.push(trimmed.to_string());
                    }
                    if trimmed.contains("Strong") {
                        strong_ciphers.push(trimmed.to_string());
                    }
                }

                let grade = if weak_ciphers.is_empty()
                    && !tls_versions
                        .iter()
                        .any(|v| v.starts_with("SSL") || v == "TLSv1.0" || v == "TLSv1.1")
                {
                    "A"
                } else if weak_ciphers.len() < 3 {
                    "B"
                } else {
                    "C"
                };

                let details = format!(
                    "sslscan: {} TLS versions, {} strong ciphers, {} weak ciphers",
                    tls_versions.len(),
                    strong_ciphers.len(),
                    weak_ciphers.len()
                );

                return json!({
                    "host": host,
                    "port": port,
                    "grade": grade,
                    "tls_versions": tls_versions,
                    "strong_ciphers_count": strong_ciphers.len(),
                    "weak_ciphers_count": weak_ciphers.len(),
                    "details": details,
                    "exit_code": exit_code,
                    "stderr": stderr.trim(),
                    "tool_available": true,
                });
            }
            Err(e) => {
                return json!({"error": format!("failed to run sslscan: {e}"), "tool_available": true});
            }
        }
    }

    // Fallback: try testssl.sh
    if tool_available("testssl") {
        let result = Command::new("testssl")
            .args(["--quiet", "--severity", "LOW", &target])
            .output();
        match result {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                let has_issues = !stdout.trim().is_empty() || !stderr.trim().is_empty();
                let grade = if !has_issues { "A" } else { "B" };
                return json!({
                    "host": host,
                    "port": port,
                    "grade": grade,
                    "details": if stdout.trim().is_empty() { "testssl completed with no severe issues" } else { stdout.trim() },
                    "tool_available": true,
                });
            }
            Err(e) => {
                return json!({"error": format!("failed to run testssl: {e}"), "tool_available": true});
            }
        }
    }

    // Neither tool available
    json!({"error": "neither sslscan nor testssl.sh found. Install: apt install sslscan or download testssl.sh", "tool_available": false})
}

// ---------------------------------------------------------------------------
// 26. net_modbus_read_work
// ---------------------------------------------------------------------------

/// Industrial Modbus register read using `modpoll` or `python3` CLI.
///
/// Args: `host` (string, required), `port` (int, default 502),
/// `register` (int, default 0), `count` (int, default 1),
/// `unit_id` (int, default 1), `function` (string, default "holding",
/// one of "holding"/"input"/"coil"/"discrete"),
/// `timeout_secs` (int, default 5).
///
/// Returns: `{host, port, register, count, unit_id, function, values, details, tool_available}`.
#[allow(dead_code)]
pub fn net_modbus_read_work(args: serde_json::Value) -> serde_json::Value {
    let host = match args.get("host").and_then(|v| v.as_str()) {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => return json!({"error": "host is required"}),
    };
    let port = args_u64(&args, "port", 502).clamp(1, 65535) as u16;
    let register = args_u64(&args, "register", 0);
    let count = args_u64(&args, "count", 1).clamp(1, 125);
    let unit_id = args_u64(&args, "unit_id", 1).clamp(1, 247);
    let function = args_str(&args, "function").to_string();
    let function = if function.is_empty() {
        "holding".to_string()
    } else {
        function
    };
    let timeout_secs = args_u64(&args, "timeout_secs", 5).clamp(1, 30);

    // Validate function code
    let func_code = match function.as_str() {
        "holding" => 3,
        "input" => 4,
        "coil" => 1,
        "discrete" => 2,
        _ => {
            return json!({"error": format!("invalid function '{}'. Supported: holding, input, coil, discrete", function)})
        }
    };

    // Try modpoll first
    if tool_available("modpoll") {
        let mut cmd = Command::new("modpoll");
        cmd.args([
            "-m",
            "-t",
            &format!("{}", func_code),
            "-a",
            &unit_id.to_string(),
            "-r",
            &register.to_string(),
            "-c",
            &count.to_string(),
            "-1",
            &timeout_secs.to_string(),
            &format!("{}:{}", host, port),
        ]);

        let result = cmd.output();
        match result {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                let exit_code = o.status.code().unwrap_or(-1);

                let values: Vec<String> = stdout
                    .lines()
                    .filter(|l| {
                        let trimmed = l.trim();
                        !trimmed.is_empty()
                            && (trimmed.starts_with('[')
                                || trimmed.chars().all(|c| {
                                    c.is_ascii_digit() || c == ' ' || c == '-' || c == '.'
                                }))
                    })
                    .map(|l| l.trim().to_string())
                    .collect();

                let details = if values.is_empty() && exit_code != 0 {
                    format!("modpoll exited with code {}: {}", exit_code, stderr.trim())
                } else {
                    format!(
                        "modpoll: {} register(s) read from {}:{} register {}",
                        values.len(),
                        host,
                        port,
                        register
                    )
                };

                return json!({
                    "host": host,
                    "port": port,
                    "register": register,
                    "count": count,
                    "unit_id": unit_id,
                    "function": function,
                    "values": values,
                    "details": details,
                    "tool_available": true,
                });
            }
            Err(e) => {
                return json!({"error": format!("failed to run modpoll: {e}"), "tool_available": true});
            }
        }
    }

    // Fallback: python3 with pymodbus
    if tool_available("python3") {
        let script = format!(
            r#"
try:
    from pymodbus.client import ModbusTcpClient
    c = ModbusTcpClient('{}', port={})
    c.connect()
    c.timeout = {}
    if {func_code} == 3:
        rr = c.read_holding_registers({}, {})
    elif {func_code} == 4:
        rr = c.read_input_registers({}, {})
    elif {func_code} == 1:
        rr = c.read_coils({}, {})
    elif {func_code} == 2:
        rr = c.read_discrete_inputs({}, {})
    else:
        rr = None
    if rr and not rr.isError():
        print(rr.registers if hasattr(rr, 'registers') else rr.bits)
    else:
        print('ERROR:', rr)
    c.close()
except Exception as e:
    print('ERROR:', e)
"#,
            host,
            port,
            timeout_secs,
            register,
            count,
            register,
            count,
            register,
            count,
            register,
            count
        );

        let result = Command::new("python3").args(["-c", &script]).output();

        match result {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                let values: Vec<String> = stdout
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("ERROR"))
                    .map(|l| l.trim().to_string())
                    .collect();

                let details = if stdout.starts_with("ERROR") {
                    format!("pymodbus error: {}", stdout.trim_start_matches("ERROR: "))
                } else if !stderr.is_empty() {
                    format!("python3 stderr: {}", stderr)
                } else {
                    format!(
                        "pymodbus: {} register(s) read from {}:{} register {}",
                        values.len(),
                        host,
                        port,
                        register
                    )
                };

                return json!({
                    "host": host,
                    "port": port,
                    "register": register,
                    "count": count,
                    "unit_id": unit_id,
                    "function": function,
                    "values": values,
                    "details": details,
                    "tool_available": true,
                });
            }
            Err(e) => {
                return json!({"error": format!("failed to run python3: {e}"), "tool_available": true});
            }
        }
    }

    // Neither tool available
    json!({"error": "neither modpoll nor python3 found. Install modpoll or: pip install pymodbus", "tool_available": false})
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // ---- net_ping_work ----
    #[test]
    fn test_ping_missing_host() {
        let result = net_ping_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_ping_empty_host() {
        let result = net_ping_work(json!({"host": ""}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_ping_returns_structure() {
        // Only runs if ping is available
        if !tool_available("ping") {
            return;
        }
        let result = net_ping_work(json!({"host": "127.0.0.1", "count": 1}));
        assert!(result.get("host").is_some());
        assert!(result.get("sent").is_some());
        assert!(result.get("received").is_some());
        assert!(result.get("packet_loss_pct").is_some());
    }

    // ---- net_traceroute_work ----

    #[test]
    fn test_traceroute_missing_host() {
        let result = net_traceroute_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_port_scan_work ----

    #[test]
    fn test_port_scan_missing_host() {
        let result = net_port_scan_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_port_scan_default_ports() {
        let result = net_port_scan_work(json!({"host": "127.0.0.1", "timeout_ms": 200}));
        assert!(result.get("ports").is_some());
        let ports = result["ports"].as_array().unwrap();
        assert!(!ports.is_empty());
        // Every port entry should have the required fields
        for p in ports {
            assert!(p.get("port").is_some());
            assert!(p.get("open").is_some());
        }
    }

    #[test]
    fn test_port_scan_custom_ports() {
        let result = net_port_scan_work(json!({
            "host": "127.0.0.1",
            "ports": [22, 80],
            "timeout_ms": 200
        }));
        let ports = result["ports"].as_array().unwrap();
        assert_eq!(ports.len(), 2);
    }

    // ---- net_dns_lookup_work ----

    #[test]
    fn test_dns_lookup_missing_domain() {
        let result = net_dns_lookup_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_dns_lookup_returns_structure() {
        if !tool_available("dig") && !tool_available("nslookup") {
            let result = net_dns_lookup_work(json!({"domain": "example.com"}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_dns_lookup_work(json!({"domain": "example.com"}));
        assert!(result.get("domain").is_some());
        assert!(result.get("record_type").is_some());
        assert!(result.get("records").is_some());
    }

    // ---- net_whois_work ----

    #[test]
    fn test_whois_missing_domain() {
        let result = net_whois_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_tls_inspect_work ----

    #[test]
    fn test_tls_inspect_missing_host() {
        let result = net_tls_inspect_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_http_benchmark_work ----

    #[test]
    fn test_http_benchmark_missing_url() {
        let result = net_http_benchmark_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_arp_scan_work ----
    #[test]
    fn test_arp_scan_returns_structure() {
        if !tool_available("ip") && !tool_available("arp") {
            let result = net_arp_scan_work(json!({}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_arp_scan_work(json!({}));
        assert!(result.get("hosts").is_some());
    }

    // ---- net_wifi_scan_work ----
    #[test]
    fn test_wifi_scan_returns_structure() {
        if !tool_available("nmcli") && !tool_available("iw") {
            let result = net_wifi_scan_work(json!({}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_wifi_scan_work(json!({}));
        assert!(result.get("networks").is_some());
        assert!(result.get("count").is_some());
    }

    // ---- net_bandwidth_test_work ----
    #[test]
    fn test_bandwidth_default_url() {
        // URL is now optional with a default; verify result has download_bytes_per_sec
        if !tool_available("curl") {
            return;
        }
        let result = net_bandwidth_test_work(json!({}));
        assert!(result.get("download_bytes_per_sec").is_some());
        assert!(result.get("details").is_some());
    }

    // ---- net_nmap_quick_work ----
    #[test]
    fn test_nmap_missing_host() {
        let result = net_nmap_quick_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_nmap_default_ports() {
        if !tool_available("nmap") {
            return;
        }
        let result = net_nmap_quick_work(json!({"host": "127.0.0.1"}));
        assert!(result.get("host").is_some());
        assert!(result.get("ports").is_some());
        assert!(result.get("details").is_some());
    }

    // ---- net_ssh_exec_work ----

    #[test]
    fn test_ssh_exec_missing_host() {
        let result = net_ssh_exec_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_ssh_exec_user_is_optional() {
        // user is now optional; without it only host+command are required
        let result = net_ssh_exec_work(json!({"host": "localhost"}));
        assert!(result["error"].is_string()); // still fails because command is missing
        let result = net_ssh_exec_work(json!({"host": "localhost", "command": "echo hi"}));
        // Should not error on missing user
        assert!(result.get("host").is_some());
        assert!(result.get("exit_code").is_some());
    }

    #[test]
    fn test_ssh_exec_missing_command() {
        let result = net_ssh_exec_work(json!({"host": "localhost", "user": "test"}));
        assert!(result["error"].is_string());
    }

    // ---- net_sftp_upload_work ----

    #[test]
    fn test_sftp_upload_missing_local_path() {
        let result = net_sftp_upload_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_sftp_upload_missing_host() {
        let result = net_sftp_upload_work(json!({"local_path": "/tmp/test"}));
        assert!(result["error"].is_string());
    }

    // ---- net_sftp_download_work ----

    #[test]
    fn test_sftp_download_missing_remote_path() {
        let result = net_sftp_download_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_wake_on_lan_work ----

    #[test]
    fn test_wol_missing_mac() {
        let result = net_wake_on_lan_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_wol_invalid_mac() {
        let result = net_wake_on_lan_work(json!({"mac": "not-a-mac"}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_wol_valid_mac_format() {
        // We test that a valid MAC produces the right structure
        // (the actual send may fail on a non-networked test env)
        let result = net_wake_on_lan_work(json!({"mac": "aa:bb:cc:dd:ee:ff"}));
        assert!(result.get("mac").is_some());
        // sent may be true or false depending on network config
        assert!(result.get("sent").is_some());
        assert!(result.get("details").is_some());
    }

    #[test]
    fn test_wol_broadcast_ip_param() {
        let result = net_wake_on_lan_work(
            json!({"mac": "aa:bb:cc:dd:ee:ff", "broadcast_ip": "192.168.1.255"}),
        );
        assert!(result.get("mac").is_some());
        assert!(result.get("sent").is_some());
        assert!(result.get("details").is_some());
        let details = result["details"].as_str().unwrap_or("");
        assert!(details.contains("192.168.1.255"));
    }

    // ---- net_serial_list_work ----

    #[test]
    fn test_serial_list_returns_structure() {
        let result = net_serial_list_work(json!({}));
        assert!(result.get("ports").is_some());
        assert!(result["ports"].is_array());
        assert!(result.get("count").is_some());
    }

    // ---- net_serial_send_work ----

    #[test]
    fn test_serial_send_missing_port() {
        let result = net_serial_send_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_serial_read_work ----

    #[test]
    fn test_serial_read_missing_port() {
        let result = net_serial_read_work(json!({}));
        assert!(result["error"].is_string());
    }

    // ---- net_bluetooth_scan_work ----

    #[test]
    fn test_bluetooth_scan_returns_structure() {
        if !tool_available("bluetoothctl") {
            let result = net_bluetooth_scan_work(json!({}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_bluetooth_scan_work(json!({}));
        assert!(result.get("devices").is_some());
    }

    // ---- net_mqtt_publish_work ----

    #[test]
    fn test_mqtt_publish_missing_broker() {
        let result = net_mqtt_publish_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_mqtt_publish_missing_topic() {
        let result = net_mqtt_publish_work(json!({"broker": "localhost"}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_mqtt_publish_missing_message() {
        let result = net_mqtt_publish_work(json!({"broker": "localhost", "topic": "test"}));
        assert!(result["error"].is_string());
    }

    // ---- net_mqtt_subscribe_work ----
    #[test]
    fn test_mqtt_subscribe_missing_broker() {
        let result = net_mqtt_subscribe_work(json!({}));
        assert!(result["error"].is_string());
    }
    #[test]
    fn test_mqtt_subscribe_missing_topic() {
        let result = net_mqtt_subscribe_work(json!({"broker": "localhost"}));
        assert!(result["error"].is_string());
    }
    #[test]
    fn test_mqtt_subscribe_has_count_field() {
        if !tool_available("mosquitto_sub") {
            return;
        }
        let result = net_mqtt_subscribe_work(
            json!({"broker": "localhost", "topic": "test", "timeout_secs": 1}),
        );
        assert!(result.get("messages").is_some());
        assert!(result.get("count").is_some());
    }

    // ---- Helper tests ----

    #[test]
    fn test_args_str_default() {
        let args = json!({});
        assert_eq!(args_str(&args, "missing"), "");
    }

    #[test]
    fn test_args_u64_default() {
        let args = json!({});
        assert_eq!(args_u64(&args, "missing", 42), 42);
    }

    #[test]
    fn test_error_tool_not_found_format() {
        let result = error_tool_not_found("foobar");
        assert_eq!(result["tool_available"], false);
        assert!(result["error"].as_str().unwrap().contains("foobar"));
    }

    // ---- net_packet_capture_work ----
    #[test]
    fn test_packet_capture_missing_interface() {
        let result = net_packet_capture_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_packet_capture_empty_interface() {
        let result = net_packet_capture_work(json!({"interface": ""}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_packet_capture_returns_structure() {
        if !tool_available("tcpdump") {
            let result = net_packet_capture_work(json!({"interface": "eth0"}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result =
            net_packet_capture_work(json!({"interface": "lo", "count": 1, "timeout_secs": 2}));
        assert!(result.get("interface").is_some());
        assert!(result.get("tool_available").is_some());
    }

    // ---- net_vpn_status_work ----
    #[test]
    fn test_vpn_status_returns_structure() {
        let result = net_vpn_status_work(json!({}));
        assert!(result.get("wireguard").is_some());
        assert!(result.get("openvpn").is_some());
        assert!(result.get("tool_available").is_some());
    }

    #[test]
    fn test_vpn_status_wireguard_only() {
        let result = net_vpn_status_work(json!({"vpn_type": "wireguard"}));
        assert!(result.get("wireguard").is_some());
        assert_eq!(result["openvpn"]["status"], "not_checked");
    }

    #[test]
    fn test_vpn_status_openvpn_only() {
        let result = net_vpn_status_work(json!({"vpn_type": "openvpn"}));
        assert!(result.get("openvpn").is_some());
        assert_eq!(result["wireguard"]["status"], "not_checked");
    }

    // ---- net_dns_zone_transfer_work ----
    #[test]
    fn test_dns_zone_transfer_missing_domain() {
        let result = net_dns_zone_transfer_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_dns_zone_transfer_empty_domain() {
        let result = net_dns_zone_transfer_work(json!({"domain": ""}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_dns_zone_transfer_returns_structure() {
        if !tool_available("dig") {
            let result = net_dns_zone_transfer_work(json!({"domain": "example.com"}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_dns_zone_transfer_work(json!({"domain": "example.com"}));
        assert!(result.get("domain").is_some());
        assert!(result.get("transferred").is_some());
        assert!(result.get("tool_available").is_some());
    }

    // ---- net_ssl_grade_work ----
    #[test]
    fn test_ssl_grade_missing_host() {
        let result = net_ssl_grade_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_ssl_grade_empty_host() {
        let result = net_ssl_grade_work(json!({"host": ""}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_ssl_grade_returns_structure() {
        if !tool_available("sslscan") && !tool_available("testssl") {
            let result = net_ssl_grade_work(json!({"host": "example.com"}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result = net_ssl_grade_work(json!({"host": "example.com", "timeout_secs": 5}));
        assert!(result.get("host").is_some());
        assert!(result.get("grade").is_some() || result.get("error").is_some());
    }

    // ---- net_modbus_read_work ----
    #[test]
    fn test_modbus_read_missing_host() {
        let result = net_modbus_read_work(json!({}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_modbus_read_empty_host() {
        let result = net_modbus_read_work(json!({"host": ""}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_modbus_read_invalid_function() {
        let result = net_modbus_read_work(json!({"host": "127.0.0.1", "function": "invalid"}));
        assert!(result["error"].is_string());
    }

    #[test]
    fn test_modbus_read_returns_structure() {
        if !tool_available("modpoll") && !tool_available("python3") {
            let result = net_modbus_read_work(json!({"host": "127.0.0.1", "register": 0}));
            assert_eq!(result["tool_available"], false);
            return;
        }
        let result =
            net_modbus_read_work(json!({"host": "127.0.0.1", "register": 0, "timeout_secs": 2}));
        assert!(result.get("host").is_some());
        assert!(result.get("tool_available").is_some());
    }
}
