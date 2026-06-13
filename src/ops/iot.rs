//! # IoT & Embedded Device Tools
//!
//! USB inventory, serial I/O, I2C scanning, GPIO control, CAN bus reading,
//! and SMS sending via gammu or AT commands.
use std::path::Path;
use std::process::Command;

use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tool_available(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// 1. iot_device_inventory
// ---------------------------------------------------------------------------

/// List USB devices and serial ports on the system.
///
/// Runs `lsusb` for USB enumeration and `ls -la /dev/ttyUSB* /dev/ttyACM* /dev/ttyS*`
/// for serial ports.
/// Returns JSON: {usb_devices: [...], serial_ports: [...], usb_count, serial_count}
pub fn iot_device_inventory_work() -> String {
    // ── USB devices via lsusb ────────────────────────────────────────────────
    let usb_devices: Vec<serde_json::Value> = if tool_available("lsusb") {
        match Command::new("lsusb").output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                stdout
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| {
                        // Format: "Bus 001 Device 002: ID 8087:0024 Intel Corp. USB 2.0 Hub"
                        if let Some(id_pos) = l.find(": ID ") {
                            let bus_dev = l[..id_pos].trim().to_string();
                            let rest = l[id_pos + 5..].trim();
                            let (vid_pid, desc) = rest
                                .split_once(' ')
                                .map(|(a, b)| (a.trim().to_string(), b.trim().to_string()))
                                .unwrap_or_else(|| (rest.to_string(), String::new()));
                            json!({
                                "bus_device": bus_dev,
                                "vid_pid":    vid_pid,
                                "description": desc,
                            })
                        } else {
                            json!({ "raw": l })
                        }
                    })
                    .collect()
            }
            Err(e) => vec![json!({"error": format!("lsusb failed: {e}")})],
        }
    } else {
        vec![json!({"note": "lsusb not found; install usbutils for USB device listing"})]
    };

    // ── Serial ports via ls ──────────────────────────────────────────────────
    let mut serial_ports: Vec<serde_json::Value> = Vec::new();

    if let Ok(o) = Command::new("sh")
        .args([
            "-c",
            "ls -la /dev/ttyUSB* /dev/ttyACM* /dev/ttyS* 2>/dev/null",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("total") {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(&path) = parts.last() {
                if path.starts_with("/dev/tty") {
                    let perms = parts.first().copied().unwrap_or("").to_string();
                    let group = if parts.len() >= 4 {
                        parts[3].to_string()
                    } else {
                        String::new()
                    };
                    serial_ports.push(json!({
                        "path":        path,
                        "permissions": perms,
                        "group":       group,
                    }));
                }
            }
        }
    }

    // Supplement with sysfs ttyUSB / ttyACM names that ls may have missed
    if let Ok(o) = Command::new("sh")
        .args([
            "-c",
            "ls /sys/class/tty/ 2>/dev/null | grep -E 'ttyUSB|ttyACM'",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        for name in stdout.lines() {
            let name = name.trim();
            if !name.is_empty() {
                let path = format!("/dev/{name}");
                if !serial_ports
                    .iter()
                    .any(|p| p["path"].as_str() == Some(&path))
                {
                    serial_ports.push(json!({ "path": path }));
                }
            }
        }
    }

    let usb_count = usb_devices.len();
    let serial_count = serial_ports.len();
    json!({
        "usb_devices":  usb_devices,
        "serial_ports": serial_ports,
        "usb_count":    usb_count,
        "serial_count": serial_count,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 2. iot_serial_read
// ---------------------------------------------------------------------------

/// Read bytes from a serial port with a timeout.
///
/// Configures baud rate via `stty`, then reads with `timeout <s>s head -c <bytes> <port>`.
/// Returns JSON: {port, baud, data, bytes_read, encoding}
pub fn iot_serial_read_work(port: String, baud: u32, timeout_ms: u64, bytes: usize) -> String {
    if !Path::new(&port).exists() {
        return json!({
            "error": format!("port {} does not exist", port),
            "port":  port,
        })
        .to_string();
    }

    // Configure baud rate
    if tool_available("stty") {
        let _ = Command::new("stty")
            .args([
                "-F",
                &port,
                &baud.to_string(),
                "raw",
                "-echo",
                "cs8",
                "-cstopb",
                "-parenb",
            ])
            .output();
    }

    let timeout_s = (timeout_ms / 1000).max(1);
    let sh_cmd = format!("timeout {}s head -c {} {}", timeout_s, bytes, port);

    match Command::new("sh").args(["-c", &sh_cmd]).output() {
        Ok(o) => {
            let raw = &o.stdout;
            let bytes_read = raw.len();

            // Prefer ASCII if all printable; otherwise hex-encode
            let (data, encoding) = match std::str::from_utf8(raw) {
                Ok(s)
                    if s.chars()
                        .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t')) =>
                {
                    (s.to_string(), "ascii")
                }
                _ => {
                    let hex = raw
                        .iter()
                        .map(|b| format!("{b:02X}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    (hex, "hex")
                }
            };

            json!({
                "port":       port,
                "baud":       baud,
                "data":       data,
                "bytes_read": bytes_read,
                "encoding":   encoding,
            })
            .to_string()
        }
        Err(e) => json!({
            "error": format!("read failed: {e}"),
            "port":  port,
            "baud":  baud,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 3. iot_serial_write
// ---------------------------------------------------------------------------

/// Write data to a serial port.
///
/// Configures baud rate with `stty`, then writes via `echo -ne "<data>" > <port>`.
/// Returns JSON: {port, written, bytes_sent, baud}
pub fn iot_serial_write_work(port: String, data: String, baud: u32) -> String {
    if !Path::new(&port).exists() {
        return json!({
            "error":   format!("port {} does not exist", port),
            "port":    port,
            "written": false,
        })
        .to_string();
    }

    if tool_available("stty") {
        let _ = Command::new("stty")
            .args(["-F", &port, &baud.to_string(), "raw", "-echo"])
            .output();
    }

    let bytes_len = data.len();
    // Escape backslashes and double-quotes for the shell echo command
    let escaped = data.replace('\\', "\\\\").replace('"', "\\\"");
    let sh_cmd = format!("echo -ne \"{}\" > {}", escaped, port);

    match Command::new("sh").args(["-c", &sh_cmd]).output() {
        Ok(o) => {
            let success = o.status.success();
            json!({
                "port":       port,
                "written":    success,
                "bytes_sent": if success { bytes_len } else { 0 },
                "baud":       baud,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":      format!("write failed: {e}"),
            "port":       port,
            "written":    false,
            "bytes_sent": 0,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 4. iot_i2c_scan
// ---------------------------------------------------------------------------

/// Scan an I2C bus for devices using `i2cdetect`.
///
/// Runs `i2cdetect -y -r <bus>` and parses the grid output for non-`--` tokens.
/// Returns JSON: {bus, devices: [{address, address_dec}], device_count, raw_output}
pub fn iot_i2c_scan_work(bus: u8) -> String {
    if !tool_available("i2cdetect") {
        return json!({
            "error":          "i2cdetect not found; install i2c-tools: sudo apt install i2c-tools",
            "tool_available": false,
            "bus":            bus,
        })
        .to_string();
    }

    match Command::new("i2cdetect")
        .args(["-y", "-r", &bus.to_string()])
        .output()
    {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();

            if !o.status.success() {
                return json!({
                    "error":          format!("i2cdetect failed: {}", stderr.trim()),
                    "bus":            bus,
                    "tool_available": true,
                })
                .to_string();
            }

            // Grid format:
            //      0  1  2  ...  f
            // 00:          -- -- ...
            // 20: -- -- -- ... 27 -- ...
            let mut devices = Vec::new();
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(colon_pos) = trimmed.find(':') {
                    let row_hex = trimmed[..colon_pos].trim();
                    if u8::from_str_radix(row_hex, 16).is_err() {
                        continue; // header line
                    }
                    let cols_part = &trimmed[colon_pos + 1..];
                    for token in cols_part.split_whitespace() {
                        // Skip placeholder tokens
                        if token == "--" || token == "UU" {
                            continue;
                        }
                        if let Ok(addr) = u8::from_str_radix(token, 16) {
                            devices.push(json!({
                                "address":     format!("0x{:02X}", addr),
                                "address_dec": addr,
                            }));
                        }
                    }
                }
            }

            let device_count = devices.len();
            json!({
                "bus":            bus,
                "devices":        devices,
                "device_count":   device_count,
                "raw_output":     stdout.trim(),
                "tool_available": true,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":          format!("failed to run i2cdetect: {e}"),
            "bus":            bus,
            "tool_available": true,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 5. iot_gpio_read
// ---------------------------------------------------------------------------

/// Read a GPIO pin value.
///
/// Tries `/sys/class/gpio/gpio<pin>/value` first; exports the pin if needed;
/// falls back to `gpio read <pin>` (WiringPi).
/// Returns JSON: {pin, value, method}
pub fn iot_gpio_read_work(pin: u8) -> String {
    let sysfs_dir = format!("/sys/class/gpio/gpio{pin}");
    let value_path = format!("{sysfs_dir}/value");

    // ── Direct sysfs read if already exported ───────────────────────────────
    if Path::new(&value_path).exists() {
        match std::fs::read_to_string(&value_path) {
            Ok(content) => {
                let value: u8 = content.trim().parse().unwrap_or(0);
                return json!({
                    "pin":    pin,
                    "value":  value,
                    "method": "sysfs",
                    "path":   value_path,
                })
                .to_string();
            }
            Err(e) => {
                return json!({
                    "error":  format!("failed to read {value_path}: {e}"),
                    "pin":    pin,
                    "method": "sysfs",
                })
                .to_string();
            }
        }
    }

    // ── Try to export then read ──────────────────────────────────────────────
    let export_path = "/sys/class/gpio/export";
    if Path::new(export_path).exists() {
        let _ = std::fs::write(export_path, pin.to_string());
        std::thread::sleep(std::time::Duration::from_millis(80));
        if let Ok(content) = std::fs::read_to_string(&value_path) {
            let value: u8 = content.trim().parse().unwrap_or(0);
            return json!({
                "pin":    pin,
                "value":  value,
                "method": "sysfs",
                "path":   value_path,
            })
            .to_string();
        }
    }

    // ── WiringPi fallback ────────────────────────────────────────────────────
    if tool_available("gpio") {
        match Command::new("gpio")
            .args(["read", &pin.to_string()])
            .output()
        {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let value: u8 = stdout.trim().parse().unwrap_or(0);
                return json!({
                    "pin":    pin,
                    "value":  value,
                    "method": "wiringpi",
                })
                .to_string();
            }
            Err(e) => {
                return json!({
                    "error": format!("gpio read failed: {e}"),
                    "pin":   pin,
                })
                .to_string();
            }
        }
    }

    json!({
        "error": format!(
            "GPIO pin {pin} not exported and neither sysfs nor WiringPi (gpio) available"
        ),
        "pin":        pin,
        "suggestion": format!("Export first: echo {pin} > /sys/class/gpio/export"),
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 6. iot_gpio_write
// ---------------------------------------------------------------------------

/// Write a value (0 or 1) to a GPIO pin via sysfs.
///
/// Exports and sets direction to "out" automatically if the pin is not yet configured.
/// Returns JSON: {pin, value, success}
pub fn iot_gpio_write_work(pin: u8, value: u8) -> String {
    let value = value.min(1);
    let gpio_dir = format!("/sys/class/gpio/gpio{pin}");
    let export_path = "/sys/class/gpio/export";
    let value_path = format!("{gpio_dir}/value");
    let direction_path = format!("{gpio_dir}/direction");

    // ── Export pin if not already present ───────────────────────────────────
    if !Path::new(&gpio_dir).exists() {
        if !Path::new(export_path).exists() {
            return json!({
                "error":   "sysfs GPIO not available on this system",
                "pin":     pin,
                "success": false,
            })
            .to_string();
        }
        if let Err(e) = std::fs::write(export_path, pin.to_string()) {
            return json!({
                "error":   format!("failed to export GPIO {pin}: {e}"),
                "pin":     pin,
                "success": false,
            })
            .to_string();
        }
        // Give the kernel a moment to create the sysfs entry
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    // ── Set direction to output ──────────────────────────────────────────────
    if Path::new(&direction_path).exists() {
        if let Err(e) = std::fs::write(&direction_path, "out") {
            return json!({
                "error":   format!("failed to set direction for GPIO {pin}: {e}"),
                "pin":     pin,
                "success": false,
            })
            .to_string();
        }
    }

    // ── Write value ──────────────────────────────────────────────────────────
    match std::fs::write(&value_path, value.to_string()) {
        Ok(_) => json!({
            "pin":     pin,
            "value":   value,
            "success": true,
            "path":    value_path,
        })
        .to_string(),
        Err(e) => json!({
            "error":   format!("failed to write value to GPIO {pin}: {e}"),
            "pin":     pin,
            "value":   value,
            "success": false,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 7. iot_can_bus_read
// ---------------------------------------------------------------------------

/// Read CAN bus frames using `candump`.
///
/// Runs `timeout <t>s candump -n <count> <interface>`.
/// Parses candump text output: `vcan0  123   [3]  DE AD BE`
/// Returns JSON: {interface, frames: [{id, length, data_hex}], count_read}
pub fn iot_can_bus_read_work(interface: String, count: usize, timeout_ms: u64) -> String {
    // Verify interface exists first
    let iface_ok = Command::new("ip")
        .args(["link", "show", &interface])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !iface_ok {
        return json!({
            "error":      format!("CAN interface '{}' not found", interface),
            "interface":  interface,
            "suggestion": "List interfaces: ip link show | grep -E 'can|vcan'",
        })
        .to_string();
    }

    if !tool_available("candump") {
        return json!({
            "error":          "candump not found; install can-utils: sudo apt install can-utils",
            "interface":      interface,
            "tool_available": false,
        })
        .to_string();
    }

    let timeout_s = (timeout_ms as f64 / 1000.0).max(0.5);
    let sh_cmd = format!(
        "timeout {:.1}s candump -n {} {}",
        timeout_s, count, interface
    );

    match Command::new("sh").args(["-c", &sh_cmd]).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let mut frames = Vec::new();

            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                // Optionally skip leading timestamp "(1234567890.123456)"
                let start = if parts.first().is_some_and(|p| p.starts_with('(')) {
                    1
                } else {
                    0
                };
                // Expect: <iface> <id> [<len>] [<byte> ...]
                if start + 2 < parts.len() {
                    let id = parts[start + 1].to_string();
                    let len_str = parts[start + 2].trim_matches('[').trim_matches(']');
                    let length: usize = len_str.parse().unwrap_or(0);
                    let data_hex = if start + 3 < parts.len() {
                        parts[start + 3..].join(" ")
                    } else {
                        String::new()
                    };
                    frames.push(json!({
                        "id":       id,
                        "length":   length,
                        "data_hex": data_hex,
                    }));
                }
            }

            let count_read = frames.len();
            json!({
                "interface":      interface,
                "frames":         frames,
                "count_read":     count_read,
                "tool_available": true,
            })
            .to_string()
        }
        Err(e) => json!({
            "error":          format!("candump failed: {e}"),
            "interface":      interface,
            "tool_available": true,
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 8. iot_sms_send
// ---------------------------------------------------------------------------

/// Send an SMS via gammu or AT commands over a serial port.
///
/// Tries `gammu --port <port> sendsms TEXT <number> -text "<message>"` first;
/// falls back to raw AT+CMGS commands via shell redirection.
/// Returns JSON: {sent, method, error}
pub fn iot_sms_send_work(port: String, number: String, message: String) -> String {
    // ── gammu first ──────────────────────────────────────────────────────────
    if tool_available("gammu") {
        if let Ok(o) = Command::new("gammu")
            .args([
                "--port", &port, "sendsms", "TEXT", &number, "-text", &message,
            ])
            .output()
        {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                return json!({
                    "sent":   true,
                    "method": "gammu",
                    "output": stdout,
                })
                .to_string();
            }
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            // If gammu is installed but the port failed, fall through to AT commands
            if stderr.contains("Permission denied") || stderr.contains("No such file") {
                return json!({
                    "sent":   false,
                    "method": "gammu",
                    "error":  stderr,
                })
                .to_string();
            }
            // Otherwise fall through to AT
        }
    }

    // ── AT commands fallback ─────────────────────────────────────────────────
    if tool_available("stty") {
        let _ = Command::new("stty")
            .args(["-F", &port, "115200", "raw", "-echo"])
            .output();
    }

    // Build AT command sequence:
    //   AT+CMGF=1         (text mode)
    //   AT+CMGS="<number>"
    //   <message><Ctrl-Z> (0x1A = end of message)
    let escaped_msg = message.replace('"', "\\\"");
    let at_cmd = format!(
        "{{ printf 'AT+CMGF=1\\r'; sleep 0.5; \
         printf 'AT+CMGS=\"{}\"\\r'; sleep 0.5; \
         printf '{}\\x1A'; sleep 3; }} > {} 2>&1",
        number, escaped_msg, port
    );

    match Command::new("sh").args(["-c", &at_cmd]).output() {
        Ok(o) => {
            let success = o.status.success();
            let output = String::from_utf8_lossy(&o.stdout).trim().to_string();
            json!({
                "sent":   success,
                "method": "at-commands",
                "port":   port,
                "error":  if success { serde_json::Value::Null } else { json!(output) },
            })
            .to_string()
        }
        Err(e) => json!({
            "sent":   false,
            "method": "at-commands",
            "error":  format!("shell command failed: {e}"),
        })
        .to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iot_device_inventory_returns_json() {
        let result = iot_device_inventory_work();
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("usb_devices").is_some());
        assert!(parsed.get("serial_ports").is_some());
        assert!(parsed.get("usb_count").is_some());
        assert!(parsed.get("serial_count").is_some());
    }

    #[test]
    fn test_iot_serial_read_nonexistent_port() {
        let result = iot_serial_read_work("/dev/ttyNONEXISTENT_PB999".to_string(), 9600, 500, 64);
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(
            parsed.get("error").is_some(),
            "should error on missing port"
        );
    }

    #[test]
    fn test_iot_serial_write_nonexistent_port() {
        let result = iot_serial_write_work(
            "/dev/ttyNONEXISTENT_PB999".to_string(),
            "hello".to_string(),
            9600,
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("error").is_some());
        assert_eq!(parsed["written"], false);
    }

    #[test]
    fn test_iot_i2c_scan_returns_json() {
        let result = iot_i2c_scan_work(99); // bus 99 almost certainly doesn't exist
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("bus").is_some());
    }

    #[test]
    fn test_iot_gpio_read_high_pin_no_panic() {
        // Pin 254 is almost certainly not exported; should return error JSON, not panic
        let result = iot_gpio_read_work(254);
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("pin").is_some());
    }

    #[test]
    fn test_iot_can_bus_read_invalid_interface() {
        let result = iot_can_bus_read_work("can_pb_nonexistent_xyz".to_string(), 5, 500);
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("error").is_some());
    }

    #[test]
    fn test_iot_sms_send_missing_gammu() {
        // Just verifies it returns valid JSON without panicking
        let result = iot_sms_send_work(
            "/dev/ttyNONEXISTENT".to_string(),
            "+10000000000".to_string(),
            "test".to_string(),
        );
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert!(parsed.get("sent").is_some());
    }
}
