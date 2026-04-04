//! Optional rate limiting for `agent_send` / orchestrated posts to reduce bus spam and abuse.
//! Configure with `MINT_BUS_MAX_SENDS_PER_WINDOW` (0 = disabled) and `MINT_BUS_RATE_WINDOW_MS`.

use std::collections::HashMap;
use std::sync::Mutex;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

struct WindowState {
    window_id: u64,
    count: u32,
}

/// Sliding fixed windows keyed by `from_agent` (caller should pass a normalized id).
pub struct BusSendRateLimiter {
    max_per_window: u32,
    window_ms: u64,
    inner: Mutex<HashMap<String, WindowState>>,
}

impl BusSendRateLimiter {
    pub fn from_env() -> Self {
        let max_per_window = std::env::var("MINT_BUS_MAX_SENDS_PER_WINDOW")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(120);

        let window_ms = std::env::var("MINT_BUS_RATE_WINDOW_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60_000)
            .max(1_000);

        Self {
            max_per_window,
            window_ms,
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Reserve quota for `n` bus appends in one logical operation (fan-out / pipeline).
    pub fn try_record_n_sends(&self, from_agent_key: &str, n: u32) -> Result<(), String> {
        if self.max_per_window == 0 || n == 0 {
            return Ok(());
        }
        let key = from_agent_key.trim();
        if key.is_empty() {
            return Ok(());
        }
        let key = if key.len() > 96 { &key[..96] } else { key };

        let now = now_ms();
        let wid = now / self.window_ms;

        let mut g = self
            .inner
            .lock()
            .map_err(|_| "bus rate limiter lock poisoned".to_string())?;

        let w = g
            .entry(key.to_string())
            .or_insert(WindowState {
                window_id: wid,
                count: 0,
            });

        if w.window_id != wid {
            w.window_id = wid;
            w.count = 0;
        }

        let next = w.count.saturating_add(n);
        if next > self.max_per_window {
            return Err(format!(
                "Rate limit: from_agent {:?} would exceed {} sends per {}ms (need {n} slots; MINT_BUS_MAX_SENDS_PER_WINDOW / MINT_BUS_RATE_WINDOW_MS).",
                key, self.max_per_window, self.window_ms
            ));
        }

        w.count = next;
        Ok(())
    }

    /// Single send; equivalent to `try_record_n_sends(..., 1)`.
    pub fn try_record_send(&self, from_agent_key: &str) -> Result<(), String> {
        self.try_record_n_sends(from_agent_key, 1)
    }
}
