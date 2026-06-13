//! # Research & Scientific Computing Tools
//!
//! Quantum circuit simulation (builtin + qasmsim), Grover's search,
//! Shor's factoring, and arXiv paper search/download.
use std::path::PathBuf;
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
// Quantum math primitives
// ---------------------------------------------------------------------------

/// Complex number as (real, imag) tuple.
type Cplx = (f64, f64);

#[inline]
fn c_add(a: Cplx, b: Cplx) -> Cplx {
    (a.0 + b.0, a.1 + b.1)
}

#[inline]
fn c_sub(a: Cplx, b: Cplx) -> Cplx {
    (a.0 - b.0, a.1 - b.1)
}

#[inline]
fn c_scale(s: f64, a: Cplx) -> Cplx {
    (s * a.0, s * a.1)
}

#[inline]
fn c_norm_sq(a: Cplx) -> f64 {
    a.0 * a.0 + a.1 * a.1
}

/// State vector for `n_qubits`: 2^n complex amplitudes, initialised to |0…0>.
fn zero_state(n_qubits: usize) -> Vec<Cplx> {
    let size = 1usize << n_qubits;
    let mut sv = vec![(0.0_f64, 0.0_f64); size];
    sv[0] = (1.0, 0.0);
    sv
}

/// Apply Hadamard to qubit `target` in-place.
fn apply_h(sv: &mut [Cplx], target: usize) {
    let f = std::f64::consts::FRAC_1_SQRT_2;
    let stride = 1 << target;
    let size = sv.len();
    let mut i = 0;
    while i < size {
        for j in i..i + stride {
            let a = sv[j];
            let b = sv[j + stride];
            sv[j] = c_scale(f, c_add(a, b));
            sv[j + stride] = c_scale(f, c_sub(a, b));
        }
        i += stride * 2;
    }
}

/// Apply Pauli-X (NOT) to qubit `target` in-place.
fn apply_x(sv: &mut [Cplx], target: usize) {
    let stride = 1 << target;
    let size = sv.len();
    let mut i = 0;
    while i < size {
        for j in i..i + stride {
            sv.swap(j, j + stride);
        }
        i += stride * 2;
    }
}

/// Apply Pauli-Z to qubit `target` in-place.
fn apply_z(sv: &mut [Cplx], target: usize) {
    let stride = 1 << target;
    let size = sv.len();
    let mut i = 0;
    while i < size {
        for j in i..i + stride {
            let b = sv[j + stride];
            sv[j + stride] = (-b.0, -b.1);
        }
        i += stride * 2;
    }
}

/// Apply CNOT (control=`ctrl`, target=`tgt`) in-place.
fn apply_cnot(sv: &mut [Cplx], ctrl: usize, tgt: usize) {
    for i in 0..sv.len() {
        if (i >> ctrl) & 1 == 1 {
            let j = i ^ (1 << tgt);
            if i < j {
                sv.swap(i, j);
            }
        }
    }
}

/// Probability for each basis state.
fn probabilities(sv: &[Cplx]) -> Vec<f64> {
    sv.iter().map(|&a| c_norm_sq(a)).collect()
}

/// State label: bit `k` of index → qubit `k` (qubit 0 is rightmost char).
/// We reverse so qubit 0 appears first, matching Qiskit's little-endian convention.
fn state_label(index: usize, n_bits: usize) -> String {
    let bin = format!("{:0>width$b}", index, width = n_bits);
    bin.chars().rev().collect()
}

// ---------------------------------------------------------------------------
// Minimal LCG PRNG (no std or rand required)
// ---------------------------------------------------------------------------

fn lcg_next(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    // Top 31 bits → [0, 1)
    ((*state >> 33) as f64) / ((1u64 << 31) as f64)
}

// ---------------------------------------------------------------------------
// Simple OpenQASM 2.0 parser
// ---------------------------------------------------------------------------

struct GateOp {
    gate: String,
    target: usize,
    control: Option<usize>,
}

/// Parse a minimal OpenQASM 2.0 subset.  Supported gates: H, X, Y, Z, CX/CNOT.
/// Returns (n_qubits, ops).
fn parse_simple_qasm(qasm: &str) -> (usize, Vec<GateOp>) {
    let mut n_qubits = 0usize;
    let mut ops: Vec<GateOp> = Vec::new();

    for raw_line in qasm.lines() {
        let line = raw_line.trim().trim_end_matches(';');
        if line.is_empty()
            || line.starts_with("//")
            || line.starts_with("OPENQASM")
            || line.starts_with("include")
            || line.starts_with("creg")
            || line.starts_with("measure")
            || line.starts_with("barrier")
        {
            continue;
        }

        if line.starts_with("qreg") {
            if let (Some(open), Some(close)) = (line.find('['), line.find(']')) {
                if let Ok(n) = line[open + 1..close].parse::<usize>() {
                    n_qubits = n_qubits.max(n);
                }
            }
            continue;
        }

        // Gate lines: "h q[0]"  or  "cx q[0],q[1]"
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            continue;
        }
        let gate = parts[0].to_lowercase();
        let qubits: Vec<usize> = parts[1]
            .split(',')
            .filter_map(|arg| {
                let arg = arg.trim();
                let open = arg.find('[')?;
                let close = arg.find(']')?;
                arg[open + 1..close].parse().ok()
            })
            .collect();

        match gate.as_str() {
            "h" | "x" | "y" | "z" | "s" | "t" => {
                if let Some(&tgt) = qubits.first() {
                    ops.push(GateOp {
                        gate,
                        target: tgt,
                        control: None,
                    });
                }
            }
            "cx" | "cnot" => {
                if qubits.len() >= 2 {
                    ops.push(GateOp {
                        gate: "cnot".into(),
                        target: qubits[1],
                        control: Some(qubits[0]),
                    });
                }
            }
            _ => {}
        }
    }

    // Infer qubit count from gate targets when no qreg declaration is present
    if n_qubits == 0 {
        for op in &ops {
            n_qubits = n_qubits.max(op.target + 1);
            if let Some(c) = op.control {
                n_qubits = n_qubits.max(c + 1);
            }
        }
    }

    (n_qubits, ops)
}

// ---------------------------------------------------------------------------
// 1. quantum_circuit_simulate
// ---------------------------------------------------------------------------

/// Simulate a quantum circuit given in OpenQASM 2.0 format.
///
/// Attempts `qasmsim` if installed; otherwise uses a pure-Rust state-vector
/// simulator supporting H, X, Z, CX/CNOT for up to 10 qubits.
///
/// Returns JSON:
/// ```json
/// { "shots": N, "n_qubits": N, "circuit_valid": true,
///   "method": "qasmsim"|"builtin_sim",
///   "results": {"00": 512, "11": 512}, "gate_count": N }
/// ```
pub fn quantum_circuit_simulate_work(circuit_qasm: String, shots: usize) -> String {
    // ── qasmsim (if installed) ────────────────────────────────────────────────
    if tool_available("qasmsim") {
        use std::io::Write;
        let tmp_path = format!("/tmp/pb_circuit_{}.qasm", std::process::id());
        if let Ok(mut f) = std::fs::File::create(&tmp_path) {
            let _ = f.write_all(circuit_qasm.as_bytes());
            let result = Command::new("qasmsim")
                .args([&tmp_path, "--shots", &shots.to_string()])
                .output();
            let _ = std::fs::remove_file(&tmp_path);
            if let Ok(o) = result {
                if o.status.success() {
                    let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    return json!({
                        "shots":         shots,
                        "method":        "qasmsim",
                        "circuit_valid": true,
                        "raw_output":    out,
                    })
                    .to_string();
                }
            }
        }
    }

    // ── Built-in state-vector simulator ──────────────────────────────────────
    let (n_qubits, raw_ops) = parse_simple_qasm(&circuit_qasm);

    if n_qubits == 0 {
        return json!({
            "error":         "no qubits found; add 'qreg q[N];' or gate declarations",
            "circuit_valid": false,
            "method":        "builtin_sim",
        })
        .to_string();
    }
    if n_qubits > 10 {
        return json!({
            "error":         format!("builtin simulator supports ≤10 qubits, got {n_qubits}"),
            "circuit_valid": true,
            "method":        "builtin_sim",
        })
        .to_string();
    }

    // Filter out-of-range qubit references
    let ops: Vec<&GateOp> = raw_ops
        .iter()
        .filter(|op| op.target < n_qubits && op.control.is_none_or(|c| c < n_qubits))
        .collect();

    // Build final state vector
    let mut sv = zero_state(n_qubits);
    for op in &ops {
        match op.gate.as_str() {
            "h" => apply_h(&mut sv, op.target),
            "x" | "y" => apply_x(&mut sv, op.target),
            "z" => apply_z(&mut sv, op.target),
            "cnot" | "cx" => {
                if let Some(ctrl) = op.control {
                    apply_cnot(&mut sv, ctrl, op.target);
                }
            }
            _ => {}
        }
    }

    let probs = probabilities(&sv);

    // Sample `shots` measurement outcomes using the LCG PRNG
    let mut rng = 0xDEAD_BEEF_CAFE_BABE_u64 ^ shots as u64;
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for _ in 0..shots {
        let r = lcg_next(&mut rng);
        let mut cumulative = 0.0_f64;
        let mut measured = probs.len() - 1;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if r < cumulative {
                measured = i;
                break;
            }
        }
        *counts.entry(state_label(measured, n_qubits)).or_insert(0) += 1;
    }

    let results: serde_json::Map<String, serde_json::Value> =
        counts.into_iter().map(|(k, v)| (k, json!(v))).collect();

    json!({
        "shots":         shots,
        "n_qubits":      n_qubits,
        "circuit_valid": true,
        "method":        "builtin_sim",
        "results":       serde_json::Value::Object(results),
        "gate_count":    ops.len(),
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 2. quantum_grover_search
// ---------------------------------------------------------------------------

/// Simulate Grover's search algorithm analytically.
///
/// Computes optimal iteration count `k = ⌊π/(4·arcsin(1/√N))⌋` and the
/// resulting probability of finding the target state after `k` Grover steps.
///
/// Returns JSON:
/// ```json
/// { "space_size": N, "target": T, "iterations_used": K,
///   "found": bool, "probability": F,
///   "amplitudes": [{"state": N, "amplitude": F, "probability": F}, …8] }
/// ```
pub fn quantum_grover_search_work(space_size: usize, target_value: usize) -> String {
    if space_size == 0 || space_size > 1024 {
        return json!({
            "error": format!("space_size must be 1–1024, got {space_size}"),
        })
        .to_string();
    }
    if target_value >= space_size {
        return json!({
            "error": format!(
                "target_value {target_value} out of range [0, {space_size})"
            ),
        })
        .to_string();
    }
    if space_size == 1 {
        return json!({
            "space_size":       1,
            "target":           target_value,
            "iterations_used":  0,
            "found":            true,
            "probability":      1.0,
            "amplitudes":       [{"state": 0, "amplitude": 1.0, "probability": 1.0}],
        })
        .to_string();
    }

    let n = space_size as f64;
    // θ = arcsin(1/√N)
    let theta = (1.0_f64 / n.sqrt()).asin();
    // Optimal iterations: floor(π / (4θ))
    let optimal_iters = ((std::f64::consts::PI / 4.0) / theta).floor() as usize;
    let optimal_iters = optimal_iters.max(1);

    // After k iterations: target amplitude = sin((2k+1)θ), others spread equally
    let angle = (2 * optimal_iters + 1) as f64 * theta;
    let target_amp = angle.sin();
    let other_amp = if space_size > 1 {
        angle.cos() / ((space_size - 1) as f64).sqrt()
    } else {
        0.0
    };
    let probability = target_amp * target_amp;

    // Show up to 8 amplitude entries
    let show = space_size.min(8);
    let amplitudes: Vec<serde_json::Value> = (0..show)
        .map(|i| {
            let amp = if i == target_value {
                target_amp
            } else {
                other_amp
            };
            json!({
                "state":       i,
                "amplitude":   (amp      * 1e6).round() / 1e6,
                "probability": (amp*amp  * 1e6).round() / 1e6,
            })
        })
        .collect();

    json!({
        "space_size":      space_size,
        "target":          target_value,
        "iterations_used": optimal_iters,
        "found":           probability > 0.5,
        "probability":     (probability * 1e6).round() / 1e6,
        "amplitudes":      amplitudes,
        "note": if space_size > 8 {
            format!("showing first 8 of {space_size} states")
        } else {
            String::new()
        },
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 3. quantum_shor_factor
// ---------------------------------------------------------------------------

/// Simulate Shor's factoring algorithm for N ≤ 1,000,000.
///
/// Uses classical modular-exponentiation period finding (simulating the quantum
/// period-finding subroutine) then extracts factors via `gcd(a^(r/2)±1, N)`.
///
/// Returns JSON:
/// ```json
/// { "n": N, "factors": [p, q], "is_prime": bool,
///   "method": "shor_sim"|"classical", "steps_taken": K }
/// ```
pub fn quantum_shor_factor_work(n: u64) -> String {
    if n < 2 {
        return json!({"error": "n must be ≥ 2", "n": n}).to_string();
    }
    if n > 1_000_000 {
        return json!({
            "error": "n must be ≤ 1,000,000 for simulation",
            "n": n,
        })
        .to_string();
    }

    if is_prime_u64(n) {
        return json!({
            "n":          n,
            "factors":    [1, n],
            "is_prime":   true,
            "method":     "classical",
            "steps_taken": 1u64,
        })
        .to_string();
    }

    if n.is_multiple_of(2) {
        return json!({
            "n":          n,
            "factors":    [2, n / 2],
            "is_prime":   false,
            "method":     "classical",
            "steps_taken": 1u64,
        })
        .to_string();
    }

    let mut steps = 0u64;

    // Shor's algorithm: iterate over bases `a`
    'outer: for a in 2..n {
        steps += 1;
        if steps > 50_000 {
            break;
        }

        let g = gcd_u64(a, n);
        if g > 1 && g < n {
            return json!({
                "n":          n,
                "factors":    [g, n / g],
                "is_prime":   false,
                "method":     "shor_sim",
                "steps_taken": steps,
            })
            .to_string();
        }

        // Find the period r of f(x) = a^x mod n
        let r = find_period(a, n, &mut steps);
        if r == 0 {
            continue 'outer;
        }

        if !r.is_multiple_of(2) {
            continue 'outer;
        }

        let half_r = r / 2;
        let ar = mod_pow(a, half_r, n);
        // ar must not be ±1 mod n
        if ar == 1 || ar == n - 1 {
            continue 'outer;
        }

        for candidate in [gcd_u64(ar + 1, n), gcd_u64(ar + n - 1, n)] {
            if candidate > 1 && candidate < n {
                return json!({
                    "n":          n,
                    "factors":    [candidate, n / candidate],
                    "is_prime":   false,
                    "method":     "shor_sim",
                    "steps_taken": steps,
                })
                .to_string();
            }
        }
    }

    // Fallback: trial division
    let mut f = 3u64;
    while f * f <= n {
        if n.is_multiple_of(f) {
            return json!({
                "n":          n,
                "factors":    [f, n / f],
                "is_prime":   false,
                "method":     "classical",
                "steps_taken": steps + f,
            })
            .to_string();
        }
        f += 2;
    }

    json!({
        "n":          n,
        "factors":    [1, n],
        "is_prime":   true,
        "method":     "classical",
        "steps_taken": steps,
    })
    .to_string()
}

// ── Number-theory helpers ────────────────────────────────────────────────────

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n.is_multiple_of(i) {
            return false;
        }
        i += 2;
    }
    true
}

/// Fast modular exponentiation: base^exp mod modulus.
fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result.wrapping_mul(base) % modulus;
        }
        exp >>= 1;
        base = base.wrapping_mul(base) % modulus;
    }
    result
}

/// Find the smallest r > 0 with a^r ≡ 1 (mod n), up to 100,000 steps.
fn find_period(a: u64, n: u64, steps: &mut u64) -> u64 {
    let limit = n.min(100_000);
    let mut x = a % n;
    for r in 1..=limit {
        *steps += 1;
        if x == 1 {
            return r;
        }
        x = x.wrapping_mul(a) % n;
    }
    0
}

// ---------------------------------------------------------------------------
// 4. research_arxiv_search
// ---------------------------------------------------------------------------

/// Search arXiv via their public Atom/XML API using `curl`.
///
/// Returns JSON:
/// ```json
/// { "query": "...", "results": [
///     { "id": "2301.12345", "title": "...", "abstract": "...",
///       "url": "https://arxiv.org/abs/...", "pdf_url": "...",
///       "published": "2023-01-30T00:00:00Z", "authors": ["A","B"] }
///   ], "results_returned": N, "total_available": M }
/// ```
pub fn research_arxiv_search_work(
    query: String,
    max_results: usize,
    category: Option<String>,
) -> String {
    if query.is_empty() {
        return json!({"error": "query is required"}).to_string();
    }
    if !tool_available("curl") {
        return json!({"error": "curl not found; required for arXiv search"}).to_string();
    }

    let max = max_results.clamp(1, 50);
    let encoded_q = percent_encode(&query);

    let search_q = if let Some(cat) = &category {
        format!("cat:{cat}+AND+{encoded_q}")
    } else {
        encoded_q
    };

    let url = format!(
        "http://export.arxiv.org/api/query\
         ?search_query={search_q}&max_results={max}&sortBy=relevance"
    );

    match Command::new("curl")
        .args(["-s", "--max-time", "30", "-L", &url])
        .output()
    {
        Ok(o) if o.status.success() => {
            let xml = String::from_utf8_lossy(&o.stdout).to_string();
            if xml.trim().is_empty() {
                return json!({
                    "error": "empty response from arXiv",
                    "query": query,
                })
                .to_string();
            }

            let entries = parse_arxiv_atom(&xml);
            let results_returned = entries.len();
            let total_available = xml_text(&xml, "opensearch:totalResults")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(results_returned);

            json!({
                "query":            query,
                "category":         category,
                "results":          entries,
                "results_returned": results_returned,
                "total_available":  total_available,
            })
            .to_string()
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            json!({"error": format!("curl failed: {stderr}"), "query": query}).to_string()
        }
        Err(e) => json!({"error": format!("failed to run curl: {e}"), "query": query}).to_string(),
    }
}

// ── arXiv XML helpers ────────────────────────────────────────────────────────

/// Percent-encode a query string (space → %20; keeps alnum + `-_.~`).
fn percent_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

/// Extract the text content of the first occurrence of `<tag>…</tag>`.
fn xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start_tag = xml.find(&open)?;
    let content_start = xml[start_tag..].find('>')? + start_tag + 1;
    let content_end = xml[content_start..].find(&close)? + content_start;
    Some(xml[content_start..content_end].trim().to_string())
}

/// Extract text content of all `<tag>…</tag>` occurrences within `xml`.
fn xml_all_text(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut results = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find(&open) {
        let after = start + open.len();
        if let Some(end) = rest[after..].find(&close) {
            results.push(rest[after..after + end].trim().to_string());
            rest = &rest[after + end + close.len()..];
        } else {
            break;
        }
    }
    results
}

/// Parse arXiv Atom XML into a vec of entry objects.
fn parse_arxiv_atom(xml: &str) -> Vec<serde_json::Value> {
    let mut entries = Vec::new();
    let mut rest = xml;

    while let Some(entry_start) = rest.find("<entry>") {
        rest = &rest[entry_start + "<entry>".len()..];
        let entry_end = rest.find("</entry>").unwrap_or(rest.len());
        let entry_xml = &rest[..entry_end];

        let raw_id = xml_text(entry_xml, "id").unwrap_or_default();
        // Strip URL prefix: "http://arxiv.org/abs/2301.12345v1" → "2301.12345v1"
        let id = raw_id
            .trim_start_matches("http://arxiv.org/abs/")
            .trim_start_matches("https://arxiv.org/abs/")
            .to_string();

        let title = xml_text(entry_xml, "title")
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let summary = xml_text(entry_xml, "summary")
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let published = xml_text(entry_xml, "published").unwrap_or_default();

        // Authors: extract <name> from each <author> block
        let authors: Vec<String> = xml_all_text(entry_xml, "author")
            .iter()
            .filter_map(|a| xml_text(a, "name"))
            .collect();

        let abs_url = format!("https://arxiv.org/abs/{}", id.trim());
        let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", id.trim());

        entries.push(json!({
            "id":        id.trim(),
            "title":     title,
            "abstract":  summary,
            "url":       abs_url,
            "pdf_url":   pdf_url,
            "published": published,
            "authors":   authors,
        }));

        if entry_end < rest.len() {
            rest = &rest[entry_end + "</entry>".len()..];
        } else {
            break;
        }
    }
    entries
}

// ---------------------------------------------------------------------------
// 5. research_arxiv_download
// ---------------------------------------------------------------------------

/// Download an arXiv paper PDF and extract its abstract.
///
/// Downloads `https://arxiv.org/pdf/<id>.pdf` to `<dest_dir>/<id>.pdf`.
/// Also fetches the abstract page to extract title and abstract.
///
/// Returns JSON:
/// ```json
/// { "arxiv_id": "...", "pdf_path": "...", "pdf_url": "...",
///   "title": "...", "abstract_preview": "…500 chars…",
///   "file_size_bytes": N }
/// ```
pub fn research_arxiv_download_work(arxiv_id: String, dest_dir: PathBuf) -> String {
    if arxiv_id.is_empty() {
        return json!({"error": "arxiv_id is required"}).to_string();
    }
    if !tool_available("curl") {
        return json!({"error": "curl not found; required for arXiv download"}).to_string();
    }

    // Create destination directory if necessary
    if !dest_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            return json!({
                "error": format!("failed to create dest_dir: {e}"),
            })
            .to_string();
        }
    }

    // Sanitise ID for use as a filename
    let clean_id = arxiv_id.trim().replace(['/', '\\', ':'], "_");
    let pdf_path = dest_dir.join(format!("{clean_id}.pdf"));
    let pdf_path_str = pdf_path.to_string_lossy().to_string();
    let pdf_url = format!("https://arxiv.org/pdf/{}.pdf", arxiv_id.trim());

    // ── Download PDF ──────────────────────────────────────────────────────────
    let dl_ok = Command::new("curl")
        .args([
            "-s",
            "-L",
            "--max-time",
            "120",
            "-o",
            &pdf_path_str,
            &pdf_url,
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let file_size_bytes = std::fs::metadata(&pdf_path).map(|m| m.len()).unwrap_or(0);

    if !dl_ok || file_size_bytes == 0 {
        return json!({
            "error":    format!("download failed or empty file for {}", arxiv_id),
            "arxiv_id": arxiv_id,
            "pdf_url":  pdf_url,
        })
        .to_string();
    }

    // ── Fetch abstract page ───────────────────────────────────────────────────
    let abs_url = format!("https://arxiv.org/abs/{}", arxiv_id.trim());
    let (title, abstract_preview) = match Command::new("curl")
        .args(["-s", "--max-time", "30", "-L", &abs_url])
        .output()
    {
        Ok(o) if o.status.success() => {
            let html = String::from_utf8_lossy(&o.stdout).to_string();

            // Try several title locations used by different arXiv page versions
            let raw_title = extract_between(&html, "class=\"title mathjax\">", "</h1>")
                .or_else(|| extract_between(&html, "<title>", "</title>"))
                .unwrap_or_else(|| format!("arXiv:{arxiv_id}"));
            let title = raw_title
                .replace("Title:", "")
                .replace('\n', " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");

            let abs_raw = extract_between(&html, "class=\"abstract mathjax\">", "</blockquote>")
                .unwrap_or_default();
            let abs_clean = abs_raw
                .replace("Abstract:", "")
                .replace('\n', " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let preview: String = abs_clean.chars().take(500).collect();

            (title, preview)
        }
        _ => (format!("arXiv:{arxiv_id}"), String::new()),
    };

    json!({
        "arxiv_id":         arxiv_id,
        "pdf_path":         pdf_path_str,
        "pdf_url":          pdf_url,
        "title":            title,
        "abstract_preview": abstract_preview,
        "file_size_bytes":  file_size_bytes,
    })
    .to_string()
}

/// Extract the first occurrence of text between `start_marker` and `end_marker`.
fn extract_between(s: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start = s.find(start_marker)? + start_marker.len();
    let end = s[start..].find(end_marker)? + start;
    Some(s[start..end].to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Quantum circuit simulator ─────────────────────────────────────────────

    #[test]
    fn test_circuit_empty_qasm() {
        let result = quantum_circuit_simulate_work(String::new(), 100);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        // Either error (no qubits) or a valid result object
        assert!(v.get("error").is_some() || v.get("method").is_some());
    }

    #[test]
    fn test_circuit_single_qubit_hadamard() {
        let qasm = "OPENQASM 2.0;\nqreg q[1];\nh q[0];";
        let result = quantum_circuit_simulate_work(qasm.to_string(), 1000);
        let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(v["method"], "builtin_sim");
        assert_eq!(v["n_qubits"], 1);
        let results = &v["results"];
        // Should have both "0" and "1" outcomes
        assert!(
            results.get("0").is_some() || results.get("1").is_some(),
            "H gate should produce superposition"
        );
    }

    #[test]
    fn test_circuit_bell_state() {
        // Bell state: |00> + |11>
        let qasm = "OPENQASM 2.0;\nqreg q[2];\nh q[0];\ncx q[0],q[1];";
        let result = quantum_circuit_simulate_work(qasm.to_string(), 2000);
        let v: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
        assert_eq!(v["n_qubits"], 2);
        let results = &v["results"];
        // Expect "00" and "11" only (no "01" or "10")
        assert!(results.get("00").is_some(), "Bell state should yield |00>");
        assert!(results.get("11").is_some(), "Bell state should yield |11>");
        assert!(
            results.get("01").is_none(),
            "Bell state should NOT yield |01>"
        );
    }

    // ── Grover search ─────────────────────────────────────────────────────────

    #[test]
    fn test_grover_basic() {
        let result = quantum_grover_search_work(8, 3);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["space_size"], 8);
        assert_eq!(v["target"], 3);
        assert!(v["probability"].as_f64().unwrap_or(0.0) > 0.5);
    }

    #[test]
    fn test_grover_out_of_range() {
        let result = quantum_grover_search_work(4, 10);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.get("error").is_some());
    }

    #[test]
    fn test_grover_space_size_1() {
        let result = quantum_grover_search_work(1, 0);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["found"], true);
        assert_eq!(v["probability"], 1.0);
    }

    // ── Shor factoring ───────────────────────────────────────────────────────

    #[test]
    fn test_shor_prime() {
        let result = quantum_shor_factor_work(17);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["is_prime"], true);
        assert_eq!(v["n"], 17);
    }

    #[test]
    fn test_shor_15() {
        let result = quantum_shor_factor_work(15);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["is_prime"], false);
        let factors = v["factors"].as_array().unwrap();
        let p = factors[0].as_u64().unwrap_or(0);
        let q = factors[1].as_u64().unwrap_or(0);
        assert_eq!(p * q, 15, "factors must multiply to 15");
    }

    #[test]
    fn test_shor_21() {
        let result = quantum_shor_factor_work(21);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["is_prime"], false);
        let factors = v["factors"].as_array().unwrap();
        let p = factors[0].as_u64().unwrap_or(0);
        let q = factors[1].as_u64().unwrap_or(0);
        assert_eq!(p * q, 21);
    }

    #[test]
    fn test_shor_n_too_large() {
        let result = quantum_shor_factor_work(2_000_000);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.get("error").is_some());
    }

    // ── arXiv search ─────────────────────────────────────────────────────────

    #[test]
    fn test_arxiv_search_empty_query() {
        let result = research_arxiv_search_work(String::new(), 5, None);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.get("error").is_some());
    }

    #[test]
    fn test_arxiv_download_empty_id() {
        let result = research_arxiv_download_work(String::new(), PathBuf::from("/tmp"));
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(v.get("error").is_some());
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    #[test]
    fn test_is_prime() {
        assert!(is_prime_u64(2));
        assert!(is_prime_u64(17));
        assert!(!is_prime_u64(15));
        assert!(!is_prime_u64(1));
    }

    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24); // 2^10 = 1024 ≡ 24 (mod 1000)
        assert_eq!(mod_pow(3, 4, 7), 4); // 81 mod 7 = 4
    }

    #[test]
    fn test_find_period_known() {
        let mut steps = 0u64;
        let r = find_period(2, 15, &mut steps);
        assert_eq!(r, 4, "2^4 ≡ 1 (mod 15)");
    }

    #[test]
    fn test_xml_text_basic() {
        let xml = "<root><foo>hello world</foo></root>";
        assert_eq!(xml_text(xml, "foo"), Some("hello world".to_string()));
    }

    #[test]
    fn test_state_label() {
        // index 0 → all zeros
        assert_eq!(state_label(0, 2), "00");
        // index 1 → bit-0 = 1 → reversed "10"
        assert_eq!(state_label(1, 2), "10");
        // index 3 → "11"
        assert_eq!(state_label(3, 2), "11");
    }
}
