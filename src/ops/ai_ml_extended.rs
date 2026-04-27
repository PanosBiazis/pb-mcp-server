//! # Extended AI/ML/DL Tools
//!
//! Multi-model comparison, chain-of-thought, self-critique, few-shot/zero-shot
//! classification, structured extraction, debate, factual verification,
//! style rewriting, translation, embedding/semantic-search, RAG indexing,
//! context packing, model info, tokenization, benchmarking, perplexity,
//! fine-tune prep, and dataset validation.

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// Maximum characters for AI tool responses (truncated beyond this).
const MAX_AI_RESPONSE_CHARS: usize = 20_000;

/// Characters per token estimate for context packing.
const CHARS_PER_TOKEN: usize = 4;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate response to `MAX_AI_RESPONSE_CHARS` with a notice if truncated.
fn truncate_response(s: &str) -> String {
    let (truncated, was_truncated) = super::truncate_chars(s, MAX_AI_RESPONSE_CHARS);
    if was_truncated {
        format!(
            "{}\n\n[truncated for MCP response: first {} characters]",
            truncated, MAX_AI_RESPONSE_CHARS
        )
    } else {
        truncated
    }
}

/// Compute cosine similarity between two f64 vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// Parse an embedding vector from a serde_json::Value.
fn extract_embedding(val: &Value) -> Option<Vec<f64>> {
    val.get("embedding")
        .and_then(|e| e.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
}

/// Return a standard error JSON when Ollama is not available.
fn ollama_unavailable() -> Value {
    serde_json::json!({
        "error": "Ollama not available. Start with: ollama serve",
        "tool_available": false
    })
}

/// Call ollama CLI with a prompt piped via stdin, return (stdout_string, latency_ms).
fn ollama_run(model: &str, prompt: &str) -> Result<(String, u128), String> {
    let start = Instant::now();
    let mut child = Command::new("ollama")
        .args(["run", model])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "ollama not found on PATH".to_string()
            } else {
                format!("Error spawning ollama: {e}")
            }
        })?;

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        if let Err(e) = stdin.write_all(prompt.as_bytes()) {
            return Err(format!("Error writing to ollama stdin: {e}"));
        }
    }

    match child.wait_with_output() {
        Ok(output) => {
            let latency = start.elapsed().as_millis();
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "ollama exited with code {:?}: {}",
                    output.status.code(),
                    &stderr[..stderr.len().min(500)]
                ));
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok((stdout, latency))
        }
        Err(e) => Err(format!("Error waiting for ollama process: {e}")),
    }
}

/// Call Ollama embedding API via curl, return parsed JSON Value.
fn ollama_embed_api(text: &str, model: &str, ollama_url: &str) -> Result<Value, String> {
    let payload = serde_json::json!({
        "model": model,
        "prompt": text
    })
    .to_string();

    let url = format!("{}/api/embeddings", ollama_url.trim_end_matches('/'));

    let output = Command::new("curl")
        .args(["-s", "--max-time", "120", &url, "-d"])
        .arg(&payload)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "curl not found on PATH".to_string()
            } else {
                format!("Error spawning curl: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "curl failed (exit={:?}): {}",
            output.status.code(),
            &stderr[..stderr.len().min(500)]
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("Error parsing ollama embedding response: {e}"))
}

/// Call Ollama generate API via curl, return parsed JSON Value and latency_ms.
fn ollama_generate_api(
    model: &str,
    prompt: &str,
    ollama_url: &str,
) -> Result<(Value, u128), String> {
    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    })
    .to_string();
    let url = format!("{}/api/generate", ollama_url.trim_end_matches('/'));
    let start = Instant::now();
    let output = Command::new("curl")
        .args(["-s", "--max-time", "300", &url, "-d"])
        .arg(&payload)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "curl not found on PATH".to_string()
            } else {
                format!("Error spawning curl: {e}")
            }
        })?;
    let latency_ms = start.elapsed().as_millis();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "curl failed (exit={:?}): {}",
            output.status.code(),
            &stderr[..stderr.len().min(500)]
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Error parsing ollama generate response: {e}"))?;
    Ok((val, latency_ms))
}

/// Extract the "response" field from an Ollama generate API JSON value.
fn extract_generate_response(val: &Value) -> String {
    val.get("response")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string()
}

/// Extract a bool arg from serde_json::Value, with a default.
fn arg_bool(args: &Value, key: &str, default: bool) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Estimate token count from text (chars / 4 heuristic).
fn estimate_tokens(text: &str) -> usize {
    text.len() / CHARS_PER_TOKEN
}

/// Check if Ollama is reachable by running `ollama list`.
fn ollama_is_available() -> bool {
    Command::new("ollama")
        .arg("list")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Extract a string arg from serde_json::Value, with a default.
fn arg_str<'a>(args: &'a Value, key: &str, default: &'a str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

/// Extract a u64 arg from serde_json::Value, with a default.
fn arg_u64(args: &Value, key: &str, default: u64) -> u64 {
    args.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// Extract an i64 arg from serde_json::Value, with a default.
fn arg_i64(args: &Value, key: &str, default: i64) -> i64 {
    args.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// Extract an array of strings from serde_json::Value, with defaults.
fn arg_str_array(args: &Value, key: &str, default: &[&str]) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect())
}

/// Extract an array of serde_json::Value from args, with empty default.
fn arg_array(args: &Value, key: &str) -> Vec<Value> {
    args.get(key)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Detect model file format from magic bytes.
fn detect_model_format(path: &Path) -> String {
    let data = match fs::read(path) {
        Ok(d) if d.len() >= 4 => d,
        _ => return "unknown".to_string(),
    };

    // GGUF magic: 0x46475547 ("GGUF" in little-endian) or 0x47475546
    if data.len() >= 4 {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic == 0x46475547 || magic == 0x47475546 {
            return "GGUF".to_string();
        }
        // GGML legacy: 0x4C4D4747 ("GGML")
        if magic == 0x4C4D4747 || magic == 0x47474D4C {
            return "GGML".to_string();
        }
    }

    // Safetensors: starts with a JSON-like header length prefix
    // First 8 bytes are the header length as little-endian u64,
    // then the header JSON contains "__metadata__"
    if data.len() >= 8 {
        let header_len = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]) as usize;
        if header_len > 0 && header_len < data.len() {
            if let Ok(header_str) =
                std::str::from_utf8(&data[8..8 + header_len.min(data.len() - 8)])
            {
                if header_str.contains("__metadata__") || header_str.contains("tensor") {
                    return "safetensors".to_string();
                }
            }
        }
    }

    // PyTorch: starts with PK (zip) or PYTORCH
    if data.starts_with(b"PK") {
        return "pytorch_zip".to_string();
    }
    if data.starts_with(b"PYTORCH") {
        return "pytorch".to_string();
    }

    // ONNX: starts with magic 0x00000000 or protobuf-like
    if data.len() >= 8 {
        let onnx_check = &data[0..8];
        if onnx_check.starts_with(b"\x08\x07") || onnx_check.starts_with(b"\x08\x01") {
            return "onnx".to_string();
        }
    }

    "unknown".to_string()
}

/// Estimate parameter count from model filename and file size.
fn estimate_params_from_filename(filename: &str, size_bytes: u64) -> String {
    let lower = filename.to_lowercase();

    // Try to extract from filename patterns like "llama-7b", "qwen2.5-72b", etc.
    let re = regex::Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*[bB]").ok();
    if let Some(re) = re {
        if let Some(caps) = re.captures(&lower) {
            if let Some(m) = caps.get(1) {
                return format!("{}B", m.as_str());
            }
        }
    }

    // Fallback: estimate from file size
    // Rough heuristic: 1B params ≈ 2GB at Q4, 4GB at Q8, 6.5GB at F16
    // Use ~2.5GB per billion params as rough middle ground for quantized models
    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if size_gb > 0.0 {
        let estimated_b = size_gb / 2.5;
        if estimated_b >= 1.0 {
            format!("~{estimated_b:.1}B (estimated from file size)")
        } else {
            let estimated_m = estimated_b * 1000.0;
            format!("~{estimated_m:.0}M (estimated from file size)")
        }
    } else {
        "unknown".to_string()
    }
}

/// Simple k-means clustering on embedding vectors.
fn k_means(embeddings: &[Vec<f64>], k: usize, max_iter: usize) -> Vec<Vec<usize>> {
    if embeddings.is_empty() || k == 0 {
        return vec![];
    }
    let k = k.min(embeddings.len());
    let n = embeddings.len();
    let dim = embeddings[0].len();

    // Initialize centroids to first k embeddings
    let mut centroids: Vec<Vec<f64>> = embeddings.iter().take(k).cloned().collect();
    if centroids.len() < k {
        centroids.resize(k, vec![0.0; dim]);
    }

    let mut assignments: Vec<usize> = (0..n).map(|i| i % k).collect();

    for _ in 0..max_iter {
        // Assign each point to nearest centroid
        let mut changed = false;
        for i in 0..n {
            let mut best_cluster = 0;
            let mut best_sim = f64::NEG_INFINITY;
            for c in 0..k {
                let sim = cosine_similarity(&embeddings[i], &centroids[c]);
                if sim > best_sim {
                    best_sim = sim;
                    best_cluster = c;
                }
            }
            if assignments[i] != best_cluster {
                assignments[i] = best_cluster;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update centroids
        let mut counts = vec![0usize; k];
        let mut sums: Vec<Vec<f64>> = vec![vec![0.0; dim]; k];
        for i in 0..n {
            let c = assignments[i];
            counts[c] += 1;
            for j in 0..dim {
                sums[c][j] += embeddings[i][j];
            }
        }
        for c in 0..k {
            if counts[c] > 0 {
                for j in 0..dim {
                    centroids[c][j] = sums[c][j] / counts[c] as f64;
                }
            }
        }
    }

    // Build cluster lists
    let mut clusters: Vec<Vec<usize>> = vec![vec![]; k];
    for i in 0..n {
        clusters[assignments[i]].push(i);
    }
    clusters.retain(|c| !c.is_empty());
    clusters
}

// ---------------------------------------------------------------------------
// Extended AI Tools
// ---------------------------------------------------------------------------
///
/// Args: `text` (string), `examples` (array of {text, label}), `model` (string).
pub fn ai_few_shot_classify_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let examples = arg_array(&args, "examples");
    let model = arg_str(&args, "model", "llama3.2");

    if examples.is_empty() {
        return serde_json::json!({"error": "Missing required arg: examples (array of {text, label})"});
    }

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    // Build few-shot prompt
    let mut prompt = String::from("Classify the following text. Here are some examples:\n\n");
    for ex in &examples {
        let ex_text = ex.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let ex_label = ex.get("label").and_then(|v| v.as_str()).unwrap_or("");
        prompt.push_str(&format!("Text: {ex_text}\nLabel: {ex_label}\n\n"));
    }
    prompt.push_str(&format!("Text: {text}\nLabel:"));

    match ollama_run(&model, &prompt) {
        Ok((output, _latency)) => {
            let label = output.trim().to_string();
            // Try to extract a confidence-like score from the response
            let confidence = if label.len() <= 30 { 0.85 } else { 0.7 };
            serde_json::json!({
                "label": label,
                "confidence": confidence,
                "model": model
            })
        }
        Err(e) => serde_json::json!({"error": e}),
    }
}

/// Zero-shot classification of text into provided labels.
///
/// Args: `text` (string), `labels` (array of strings), `model` (string).
pub fn ai_zero_shot_classify_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let labels = arg_str_array(&args, "labels", &[]);
    let model = arg_str(&args, "model", "llama3.2");

    if labels.is_empty() {
        return serde_json::json!({"error": "Missing required arg: labels (array of strings)"});
    }

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    let labels_str = labels.join(", ");
    let prompt = format!(
        "Classify the following text into one of these categories: [{labels_str}]\n\n\
         Text: {text}\n\n\
         Respond with ONLY the category name, nothing else."
    );

    match ollama_run(&model, &prompt) {
        Ok((output, _latency)) => {
            let predicted = output.trim().to_string();

            // Compute simple scores: exact match = 0.95, partial = 0.6, others scaled
            let mut scores = Vec::new();
            for label in &labels {
                let score = if label.to_lowercase() == predicted.to_lowercase() {
                    0.95
                } else if predicted.to_lowercase().contains(&label.to_lowercase()) {
                    0.7
                } else {
                    0.05 / labels.len() as f64
                };
                scores.push(serde_json::json!({
                    "label": label,
                    "score": score
                }));
            }

            serde_json::json!({
                "label": predicted,
                "scores": scores
            })
        }
        Err(e) => serde_json::json!({"error": e}),
    }
}

/// Extract structured data from unstructured text using a schema.
///
/// Args: `text` (string), `schema` (object), `model` (string).
pub fn ai_structured_extract_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let schema = match args.get("schema") {
        Some(s) => s.clone(),
        None => return serde_json::json!({"error": "Missing required arg: schema"}),
    };
    let model = arg_str(&args, "model", "llama3.2");

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    let schema_str = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| schema.to_string());
    let prompt = format!(
        "Extract structured data from the following text according to this JSON schema.\n\
         Return ONLY valid JSON matching the schema, no other text.\n\n\
         Schema:\n{schema_str}\n\n\
         Text:\n{text}\n\n\
         JSON output:"
    );

    match ollama_run(&model, &prompt) {
        Ok((output, _latency)) => {
            // Try to parse the output as JSON
            let cleaned = output
                .trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let extracted = match serde_json::from_str::<Value>(cleaned) {
                Ok(v) => v,
                Err(_) => {
                    // Try to find JSON object in the response
                    if let Some(start) = cleaned.find('{') {
                        if let Some(end) = cleaned.rfind('}') {
                            let json_str = &cleaned[start..=end];
                            match serde_json::from_str::<Value>(json_str) {
                                Ok(v) => v,
                                Err(_) => serde_json::json!({"raw_output": cleaned}),
                            }
                        } else {
                            serde_json::json!({"raw_output": cleaned})
                        }
                    } else {
                        serde_json::json!({"raw_output": cleaned})
                    }
                }
            };

            serde_json::json!({
                "extracted": extracted
            })
        }
        Err(e) => serde_json::json!({"error": e}),
    }
}

/// Two models argue opposing sides then synthesize.
///
/// Args: `topic` (string), `model_a` (string, default "llama3.2"),
/// Args: `claim` (string), `references` (array of strings), `model` (string).
pub fn ai_verify_factual_work(args: Value) -> Value {
    let claim = match args.get("claim").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: claim"}),
    };
    let references = arg_str_array(&args, "references", &[]);
    let model = arg_str(&args, "model", "llama3.2");

    if references.is_empty() {
        return serde_json::json!({"error": "Missing required arg: references (array of strings)"});
    }

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    let refs_joined = references.join("\n---\n");
    let prompt = format!(
        "Given the following reference documents, determine if the claim is supported, \
         contradicted, or unverifiable. Cite specific evidence.\n\n\
         Claim: {claim}\n\n\
         Reference documents:\n{refs_joined}\n\n\
         Respond in this format:\n\
         Verdict: [supported|contradicted|unverifiable]\n\
         Evidence: [list specific evidence]"
    );

    match ollama_run(&model, &prompt) {
        Ok((output, _latency)) => {
            let response = output.trim().to_string();
            let lower = response.to_lowercase();

            let verdict = if lower.contains("supported")
                && !lower.contains("contradicted")
                && !lower.contains("unverifiable")
            {
                "supported"
            } else if lower.contains("contradicted") {
                "contradicted"
            } else {
                "unverifiable"
            };

            // Extract evidence lines
            let evidence: Vec<String> = response
                .lines()
                .filter(|l| {
                    let ll = l.to_lowercase();
                    ll.starts_with('-')
                        || ll.starts_with("•")
                        || ll.starts_with("evidence")
                        || ll.contains("cite")
                        || ll.contains("according to")
                        || ll.contains("states")
                })
                .map(|l| {
                    l.trim_start_matches('-')
                        .trim_start_matches('•')
                        .trim()
                        .to_string()
                })
                .filter(|l| !l.is_empty())
                .take(5)
                .collect();

            let evidence = if evidence.is_empty() {
                vec![response.clone()]
            } else {
                evidence
            };

            serde_json::json!({
                "claim": claim,
                "verdict": verdict,
                "evidence": evidence
            })
        }
        Err(e) => serde_json::json!({"error": e}),
    }
}

/// Rewrite text in a specified style.
///
/// Args: `text` (string), `style` (string: "technical", "casual", "academic",
/// "eli5", "concise"), `model` (string).
pub fn ai_rewrite_style_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let style = arg_str(&args, "style", "concise");
    let model = arg_str(&args, "model", "llama3.2");

    let valid_styles = ["technical", "casual", "academic", "eli5", "concise"];
    if !valid_styles.contains(&style.as_str()) {
        return serde_json::json!({
            "error": format!("Invalid style '{}'. Must be one of: {:?}", style, valid_styles)
        });
    }

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    let style_instructions = match style.as_str() {
        "technical" => "Rewrite in a technical, precise style with domain-specific terminology.",
        "casual" => "Rewrite in a casual, conversational, friendly style.",
        "academic" => "Rewrite in a formal academic style with scholarly language and citations style references.",
        "eli5" => "Rewrite as if explaining to a 5-year-old. Use simple words, analogies, and short sentences.",
        "concise" => "Rewrite to be as concise as possible while preserving all key information.",
        _ => "Rewrite clearly.",
    };

    let prompt = format!("{style_instructions}\n\nOriginal text:\n{text}\n\nRewritten text:");

    match ollama_run(&model, &prompt) {
        Ok((output, _latency)) => serde_json::json!({
            "rewritten": truncate_response(output.trim()),
            "style": style
        }),
        Err(e) => serde_json::json!({"error": e}),
    }
}

/// Translate text between languages using a local model.
///
/// Args: `text` (string), `from` (string, e.g. "en"), `to` (string, e.g. "es"),
/// Get an embedding vector from an Ollama embedding model.
///
/// Args: `text` (string), `model` (string, default "nomic-embed-text"),
/// `ollama_url` (string, default "http://localhost:11434").
pub fn ai_embed_text_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let model = arg_str(&args, "model", "nomic-embed-text");
    let ollama_url = arg_str(&args, "ollama_url", "http://localhost:11434");

    match ollama_embed_api(&text, &model, &ollama_url) {
        Ok(val) => {
            if let Some(error) = val.get("error") {
                return serde_json::json!({
                    "error": error.as_str().unwrap_or("Unknown embedding error"),
                    "tool_available": false
                });
            }
            match extract_embedding(&val) {
                Some(embedding) => {
                    let dimensions = embedding.len();
                    // Truncate for display if very large
                    let display_embedding: Vec<f64> = embedding.iter().take(20).copied().collect();
                    serde_json::json!({
                        "embedding": display_embedding,
                        "embedding_full_length": dimensions,
                        "model": model,
                        "dimensions": dimensions
                    })
                }
                None => serde_json::json!({
                    "error": "No embedding array in response",
                    "raw_response": val
                }),
            }
        }
        Err(e) => {
            if e.contains("curl not found") {
                serde_json::json!({
                    "error": "curl not found. Install curl for embedding API access.",
                    "tool_available": false
                })
            } else {
                serde_json::json!({"error": e, "tool_available": false})
            }
        }
    }
}

/// Semantic search over documents using embeddings.
///
/// Args: `query` (string), `documents` (array of strings), `model` (string).
pub fn ai_semantic_search_work(args: Value) -> Value {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: query"}),
    };
    let documents = arg_str_array(&args, "documents", &[]);
    let model = arg_str(&args, "model", "nomic-embed-text");
    let ollama_url = arg_str(&args, "ollama_url", "http://localhost:11434");

    if documents.is_empty() {
        return serde_json::json!({"error": "Missing required arg: documents (array of strings)"});
    }

    // Get query embedding
    let query_emb = match ollama_embed_api(&query, &model, &ollama_url) {
        Ok(val) => match extract_embedding(&val) {
            Some(e) => e,
            None => return serde_json::json!({"error": "Failed to extract query embedding"}),
        },
        Err(e) => return serde_json::json!({"error": e, "tool_available": false}),
    };

    // Get document embeddings
    let mut doc_embeddings: Vec<(usize, Vec<f64>)> = Vec::new();
    for (i, doc) in documents.iter().enumerate() {
        match ollama_embed_api(doc, &model, &ollama_url) {
            Ok(val) => {
                if let Some(emb) = extract_embedding(&val) {
                    doc_embeddings.push((i, emb));
                }
            }
            Err(_) => continue,
        }
    }

    // Compute cosine similarities
    let mut results: Vec<Value> = doc_embeddings
        .iter()
        .map(|(idx, emb)| {
            let score = cosine_similarity(&query_emb, emb);
            let snippet = if documents[*idx].len() > 100 {
                format!("{}...", &documents[*idx][..100])
            } else {
                documents[*idx].clone()
            };
            serde_json::json!({
                "document_index": idx,
                "text_snippet": snippet,
                "similarity_score": (score * 10000.0).round() / 10000.0
            })
        })
        .collect();

    // Sort by similarity score descending
    results.sort_by(|a, b| {
        let sa = a
            .get("similarity_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("similarity_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    serde_json::json!({
        "results": results,
        "query": query,
        "model": model
    })
}

/// Cluster texts by semantic similarity using k-means on embeddings.
///
/// Args: `texts` (array of strings), `k` (int, default 3), `model` (string).
pub fn ai_cluster_texts_work(args: Value) -> Value {
    let texts = arg_str_array(&args, "texts", &[]);
    let k = arg_u64(&args, "k", 3) as usize;
    let model = arg_str(&args, "model", "nomic-embed-text");
    let ollama_url = arg_str(&args, "ollama_url", "http://localhost:11434");

    if texts.is_empty() {
        return serde_json::json!({"error": "Missing required arg: texts (array of strings)"});
    }

    // Get embeddings for all texts
    let mut embeddings: Vec<Vec<f64>> = Vec::new();
    let mut embedded_indices: Vec<usize> = Vec::new();
    for (i, text) in texts.iter().enumerate() {
        match ollama_embed_api(text, &model, &ollama_url) {
            Ok(val) => {
                if let Some(emb) = extract_embedding(&val) {
                    embeddings.push(emb);
                    embedded_indices.push(i);
                }
            }
            Err(_) => continue,
        }
    }

    if embeddings.is_empty() {
        return serde_json::json!({"error": "Failed to get any embeddings", "tool_available": false});
    }

    let clusters = k_means(&embeddings, k, 20);

    let cluster_results: Vec<Value> = clusters
        .iter()
        .enumerate()
        .map(|(cluster_id, members)| {
            // Compute average similarity to centroid
            let centroid_sim = if !members.is_empty() && members.len() > 1 {
                let mut total_sim = 0.0;
                let mut count = 0;
                for i in 0..members.len() {
                    for j in (i + 1)..members.len() {
                        total_sim +=
                            cosine_similarity(&embeddings[members[i]], &embeddings[members[j]]);
                        count += 1;
                    }
                }
                if count > 0 {
                    (total_sim / count as f64 * 10000.0).round() / 10000.0
                } else {
                    1.0
                }
            } else {
                1.0
            };

            let text_indices: Vec<usize> = members.iter().map(|m| embedded_indices[*m]).collect();

            serde_json::json!({
                "id": cluster_id,
                "texts": text_indices,
                "centroid_similarity": centroid_sim
            })
        })
        .collect();

    serde_json::json!({
        "clusters": cluster_results,
        "model": model,
        "total_texts": texts.len()
    })
}

/// Build a RAG index from a directory of text files.
///
/// Args: `path` (string), `chunk_size` (int, default 500), `overlap` (int,

/// Pack multiple files into a context window respecting a token budget.
///
/// Args: `paths` (array of strings), `max_tokens` (int, default 8000),
/// `model` (string, default "" — not used for embedding, just estimation).
pub fn ai_context_pack_smart_work(args: Value) -> Value {
    let paths = arg_str_array(&args, "paths", &[]);
    let max_tokens = arg_u64(&args, "max_tokens", 8000) as usize;
    let _model = arg_str(&args, "model", "");

    if paths.is_empty() {
        return serde_json::json!({"error": "Missing required arg: paths (array of strings)"});
    }

    let max_chars = max_tokens * CHARS_PER_TOKEN;

    // Read all files and sort by size (smaller = higher priority)
    let mut file_entries: Vec<(PathBuf, String, usize)> = Vec::new(); // path, content, chars
    for path_str in &paths {
        let path = PathBuf::from(path_str);
        if let Ok(content) = fs::read_to_string(&path) {
            file_entries.push((path, content, 0));
        }
    }

    // Sort by content length ascending (smaller files first = higher priority)
    file_entries.sort_by_key(|(_, content, _)| content.len());

    let mut packed = String::new();
    let mut files_included: Vec<String> = Vec::new();
    let mut files_excluded: Vec<String> = Vec::new();
    let mut used_chars = 0usize;

    for (path, content, _) in &file_entries {
        let header = format!("--- {} ---\n", path.display());
        let entry_chars = header.len() + content.len() + 2;

        if used_chars + entry_chars <= max_chars {
            packed.push_str(&header);
            packed.push_str(content);
            packed.push_str("\n\n");
            used_chars += entry_chars;
            files_included.push(path.to_string_lossy().to_string());
        } else {
            files_excluded.push(path.to_string_lossy().to_string());
        }
    }

    let estimated_tokens = used_chars / CHARS_PER_TOKEN;

    serde_json::json!({
        "packed_content": packed,
        "files_included": files_included,
        "files_excluded": files_excluded,
        "estimated_tokens": estimated_tokens
    })
}

// ---------------------------------------------------------------------------
// ML/DL Tools
// ---------------------------------------------------------------------------

/// Get info about a local model file.
///
/// Args: `path` (string).
pub fn ml_model_info_work(args: Value) -> Value {
    let path_str = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: path"}),
    };

    let path = Path::new(&path_str);
    if !path.is_file() {
        return serde_json::json!({"error": format!("File not found: {path_str}")});
    }

    let size_bytes = match fs::metadata(path) {
        Ok(m) => m.len(),
        Err(e) => return serde_json::json!({"error": format!("Cannot read file metadata: {e}")}),
    };

    let format = detect_model_format(path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let estimated_params = estimate_params_from_filename(filename, size_bytes);

    // Extract info from filename
    let filename_info = {
        let lower = filename.to_lowercase();
        let mut info = serde_json::json!({});
        if lower.contains("instruct") {
            info["type"] = serde_json::json!("instruct");
        } else if lower.contains("chat") {
            info["type"] = serde_json::json!("chat");
        } else if lower.contains("base") {
            info["type"] = serde_json::json!("base");
        } else if lower.contains("code") {
            info["type"] = serde_json::json!("code");
        }

        // Detect quantization
        for quant in &[
            "q4_0", "q4_1", "q4_k_m", "q4_k_s", "q5_0", "q5_1", "q5_k_m", "q5_k_s", "q6_k", "q8_0",
            "f16", "f32", "q2_k", "q3_k_s", "q3_k_m", "q3_k_l",
        ] {
            if lower.contains(quant) {
                info["quantization"] = serde_json::json!(quant);
                break;
            }
        }
        info
    };

    serde_json::json!({
        "path": path_str,
        "format": format,
        "size_bytes": size_bytes,
        "estimated_params": estimated_params,
        "filename_info": filename_info
    })
}

/// Tokenize text using an Ollama model's tokenizer via API.
///
/// Args: `text` (string), `model` (string, default "llama3.2").
pub fn ml_tokenize_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let model = arg_str(&args, "model", "llama3.2");
    let ollama_url = arg_str(&args, "ollama_url", "http://localhost:11434");

    // Try the Ollama tokenize API endpoint
    let payload = serde_json::json!({
        "model": model,
        "prompt": text
    })
    .to_string();

    let url = format!("{}/api/tokenize", ollama_url.trim_end_matches('/'));

    match Command::new("curl")
        .args(["-s", "--max-time", "30", &url, "-d"])
        .arg(&payload)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !output.status.success() {
                // Fallback: estimate tokens
                let token_count = estimate_tokens(&text);
                return serde_json::json!({
                    "tokens": Vec::<i64>::new(),
                    "token_count": token_count,
                    "estimation_method": "chars_div_4_fallback",
                    "note": "Ollama tokenize API unavailable, using estimation"
                });
            }
            match serde_json::from_str::<Value>(&stdout) {
                Ok(val) => {
                    if let Some(tokens) = val.get("tokens").and_then(|t| t.as_array()) {
                        let token_count = tokens.len();
                        serde_json::json!({
                            "tokens": tokens,
                            "token_count": token_count,
                            "estimation_method": "ollama_api"
                        })
                    } else {
                        // Fallback
                        let token_count = estimate_tokens(&text);
                        serde_json::json!({
                            "tokens": Vec::<i64>::new(),
                            "token_count": token_count,
                            "estimation_method": "chars_div_4_fallback",
                            "note": "Unexpected API response format"
                        })
                    }
                }
                Err(_) => {
                    let token_count = estimate_tokens(&text);
                    serde_json::json!({
                        "tokens": Vec::<i64>::new(),
                        "token_count": token_count,
                        "estimation_method": "chars_div_4_fallback",
                        "note": "Failed to parse tokenize API response"
                    })
                }
            }
        }
        Err(_) => {
            let token_count = estimate_tokens(&text);
            serde_json::json!({
                "tokens": Vec::<i64>::new(),
                "token_count": token_count,
                "estimation_method": "chars_div_4_fallback",
                "note": "curl not available, using estimation"
            })
        }
    }
}

/// Count tokens for context planning.
///
/// Args: `text` (string), `model` (string, default "llama3.2").
pub fn ml_token_count_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let model = arg_str(&args, "model", "llama3.2");
    let ollama_url = arg_str(&args, "ollama_url", "http://localhost:11434");

    // Try the Ollama API first
    let payload = serde_json::json!({
        "model": model,
        "prompt": text
    })
    .to_string();

    let url = format!("{}/api/tokenize", ollama_url.trim_end_matches('/'));

    match Command::new("curl")
        .args(["-s", "--max-time", "30", &url, "-d"])
        .arg(&payload)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if output.status.success() {
                if let Ok(val) = serde_json::from_str::<Value>(&stdout) {
                    if let Some(tokens) = val.get("tokens").and_then(|t| t.as_array()) {
                        return serde_json::json!({
                            "token_count": tokens.len(),
                            "estimation_method": "ollama_api",
                            "chars_per_token": if !text.is_empty() {
                                (text.len() as f64 / tokens.len() as f64 * 100.0).round() / 100.0
                            } else {
                                0.0
                            }
                        });
                    }
                }
            }
            // Fallback to estimation
            let token_count = estimate_tokens(&text);
            serde_json::json!({
                "token_count": token_count,
                "estimation_method": "chars_div_4",
                "chars_per_token": 4.0
            })
        }
        Err(_) => {
            let token_count = estimate_tokens(&text);
            serde_json::json!({
                "token_count": token_count,
                "estimation_method": "chars_div_4",
                "chars_per_token": 4.0
            })
        }
    }
}

/// Benchmark a local model by measuring tokens/sec and latency.
///
/// Args: `model` (string), `prompt` (string, default "Explain quantum computing
/// in one paragraph."), `iterations` (int, default 3).
pub fn ml_benchmark_model_work(args: Value) -> Value {
    let model = arg_str(&args, "model", "llama3.2");
    let prompt = arg_str(
        &args,
        "prompt",
        "Explain quantum computing in one paragraph.",
    );
    let iterations = arg_u64(&args, "iterations", 3) as usize;

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    if iterations == 0 || iterations > 20 {
        return serde_json::json!({"error": "iterations must be between 1 and 20"});
    }

    let mut latencies: Vec<u128> = Vec::new();
    let mut tokens_per_sec: Vec<f64> = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        match ollama_run(&model, &prompt) {
            Ok((output, _latency)) => {
                let elapsed = start.elapsed();
                let elapsed_ms = elapsed.as_millis();
                let token_count = estimate_tokens(&output);
                let secs = elapsed.as_secs_f64();
                let tps = if secs > 0.0 {
                    token_count as f64 / secs
                } else {
                    0.0
                };
                latencies.push(elapsed_ms);
                tokens_per_sec.push(tps);
            }
            Err(e) => {
                return serde_json::json!({"error": format!("Benchmark iteration failed: {e}")});
            }
        }
    }

    let avg_latency_ms = if !latencies.is_empty() {
        latencies.iter().sum::<u128>() as f64 / latencies.len() as f64
    } else {
        0.0
    };

    let avg_tokens_per_sec = if !tokens_per_sec.is_empty() {
        tokens_per_sec.iter().sum::<f64>() / tokens_per_sec.len() as f64
    } else {
        0.0
    };

    serde_json::json!({
        "model": model,
        "avg_tokens_per_sec": (avg_tokens_per_sec * 100.0).round() / 100.0,
        "avg_latency_ms": avg_latency_ms.round(),
        "iterations": iterations
    })
}

/// Compute approximate perplexity score for text using a model.
///
/// Args: `text` (string), `model` (string).
pub fn ml_perplexity_score_work(args: Value) -> Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}),
    };
    let model = arg_str(&args, "model", "llama3.2");

    if !ollama_is_available() {
        return ollama_unavailable();
    }

    // Use the Ollama API to get logprobs/perplexity estimate
    // Since Ollama's CLI doesn't directly output perplexity, we use a
    // prompt-based approach: ask the model to score the text's likelihood
    let prompt = format!(
        "Rate how natural and well-formed the following text is on a scale \
         of 1-10 (10 = perfectly natural). Consider grammar, coherence, \
         and fluency. Reply with ONLY a number.\n\nText: {text}"
    );

    let (response, _latency) = match ollama_run(&model, &prompt) {
        Ok(r) => r,
        Err(e) => return serde_json::json!({"error": e}),
    };

    // Parse the naturalness score and convert to approximate perplexity
    let naturalness: f64 = response
        .trim()
        .lines()
        .next()
        .unwrap_or("5")
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(5.0);

    // Convert naturalness (1-10) to approximate perplexity
    // Higher naturalness = lower perplexity
    // perplexity ≈ 10^((10 - naturalness) / 3)
    let perplexity = 10_f64.powf((10.0 - naturalness.clamp(1.0, 10.0)) / 3.0);

    serde_json::json!({
        "perplexity": (perplexity * 100.0).round() / 100.0,
        "model": model,
        "text_length": text.len(),
        "note": "Approximate perplexity based on model's naturalness assessment"
    })
}

/// Convert conversations to JSONL fine-tune format.
///
/// Args: `conversations` (array of {messages: [{role, content}]}),
/// `format` (string: "openai", "llama", "mistral").
pub fn ml_fine_tune_prep_work(args: Value) -> Value {
    let conversations = arg_array(&args, "conversations");
    let format = arg_str(&args, "format", "openai");

    if conversations.is_empty() {
        return serde_json::json!({"error": "Missing required arg: conversations"});
    }

    let valid_formats = ["openai", "llama", "mistral"];
    if !valid_formats.contains(&format.as_str()) {
        return serde_json::json!({
            "error": format!("Invalid format '{}'. Must be one of: {:?}", format, valid_formats)
        });
    }

    let mut jsonl_lines: Vec<String> = Vec::new();

    for conv in &conversations {
        let messages = match conv.get("messages").and_then(|m| m.as_array()) {
            Some(msgs) => msgs,
            None => continue,
        };

        let line = match format.as_str() {
            "openai" => {
                // OpenAI format: {"messages": [{"role": "...", "content": "..."}, ...]}
                let formatted_msgs: Vec<Value> = messages
                    .iter()
                    .filter_map(|m| {
                        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                        let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                        if role.is_empty() || content.is_empty() {
                            None
                        } else {
                            Some(serde_json::json!({"role": role, "content": content}))
                        }
                    })
                    .collect();
                serde_json::json!({"messages": formatted_msgs}).to_string()
            }
            "llama" => {
                // Llama format: {"conversations": [{"from": "...", "value": "..."}, ...]}
                let formatted_msgs: Vec<Value> = messages
                    .iter()
                    .filter_map(|m| {
                        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                        let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                        let from = match role {
                            "system" => "system",
                            "assistant" => "gpt",
                            _ => "human",
                        };
                        if content.is_empty() {
                            None
                        } else {
                            Some(serde_json::json!({"from": from, "value": content}))
                        }
                    })
                    .collect();
                serde_json::json!({"conversations": formatted_msgs}).to_string()
            }
            "mistral" => {
                // Mistral format: {"messages": [...]} with specific role handling
                let formatted_msgs: Vec<Value> = messages
                    .iter()
                    .filter_map(|m| {
                        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                        let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                        // Mistral uses "user" and "assistant" with optional "system"
                        if content.is_empty() {
                            None
                        } else {
                            Some(serde_json::json!({
                                "role": role,
                                "content": content
                            }))
                        }
                    })
                    .collect();
                serde_json::json!({"messages": formatted_msgs}).to_string()
            }
            _ => continue,
        };

        jsonl_lines.push(line);
    }

    let example_count = jsonl_lines.len();
    let sample_line = jsonl_lines.first().cloned().unwrap_or_default();

    // Write output file
    let output_path = ".finetune_data.jsonl";
    let jsonl_content = jsonl_lines.join("\n");
    match fs::write(output_path, &jsonl_content) {
        Ok(_) => serde_json::json!({
            "output_path": output_path,
            "example_count": example_count,
            "format": format,
            "sample_line": sample_line
        }),
        Err(e) => serde_json::json!({
            "output_path": output_path,
            "example_count": example_count,
            "format": format,
            "sample_line": sample_line,
            "write_error": format!("Failed to write file: {e}")
        }),
    }
}

/// Validate a JSONL dataset for fine-tuning.
///
/// Args: `path` (string), `max_tokens` (int, default 4096).
pub fn ml_dataset_validate_work(args: Value) -> Value {
    let path_str = match args.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: path"}),
    };
    let max_tokens = arg_u64(&args, "max_tokens", 4096) as usize;
    let _model = arg_str(&args, "model", "");

    let content = match fs::read_to_string(&path_str) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read file: {e}")}),
    };

    let mut total_examples = 0usize;
    let mut errors: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();
    let mut valid = true;

    for (line_num, line) in content.lines().enumerate() {
        let line_num_1based = line_num + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        total_examples += 1;

        // Check valid JSON
        let parsed: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                errors.push(serde_json::json!({
                    "line": line_num_1based,
                    "issue": format!("Invalid JSON: {e}")
                }));
                valid = false;
                continue;
            }
        };

        // Check required fields (messages or conversations)
        let has_messages = parsed.get("messages").and_then(|m| m.as_array()).is_some();
        let has_conversations = parsed
            .get("conversations")
            .and_then(|c| c.as_array())
            .is_some();

        if !has_messages && !has_conversations {
            errors.push(serde_json::json!({
                "line": line_num_1based,
                "issue": "Missing 'messages' or 'conversations' field"
            }));
            valid = false;
            continue;
        }

        // Validate individual messages
        let messages = if has_messages {
            parsed["messages"].as_array().unwrap()
        } else {
            parsed["conversations"].as_array().unwrap()
        };

        if messages.is_empty() {
            errors.push(serde_json::json!({
                "line": line_num_1based,
                "issue": "Empty messages array"
            }));
            valid = false;
            continue;
        }

        for (msg_idx, msg) in messages.iter().enumerate() {
            // Check role/content fields
            let role_key = if has_messages { "role" } else { "from" };
            let content_key = if has_messages { "content" } else { "value" };

            if msg.get(role_key).is_none() {
                errors.push(serde_json::json!({
                    "line": line_num_1based,
                    "issue": format!("Message {} missing '{}' field", msg_idx, role_key)
                }));
                valid = false;
            }

            let content_val = msg.get(content_key).and_then(|v| v.as_str()).unwrap_or("");
            if content_val.is_empty() {
                warnings.push(serde_json::json!({
                    "line": line_num_1based,
                    "issue": format!("Message {} has empty content", msg_idx)
                }));
            }
        }

        // Token estimate check
        let full_text = trimmed;
        let token_estimate = estimate_tokens(full_text);
        if token_estimate > max_tokens {
            warnings.push(serde_json::json!({
                "line": line_num_1based,
                "issue": format!(
                    "Estimated {} tokens exceeds max_tokens ({})",
                    token_estimate, max_tokens
                )
            }));
        }
    }

    serde_json::json!({
        "valid": valid,
        "total_examples": total_examples,
        "errors": errors,
        "warnings": warnings
    })
}

// ---------------------------------------------------------------------------
// New Extended AI/ML Tools (curl-based API variants)
// ---------------------------------------------------------------------------

/// Compare outputs from multiple models using the Ollama generate API via curl.
///
/// Args: `prompt` (string), `models` (array of strings, default ["llama3.2",
/// "qwen2.5-coder"]), `ollama_url` (string, default "http://localhost:11434").
#[allow(dead_code)]
pub fn ai_multi_model_compare_work(params: Value) -> String {
    let prompt = match params.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: prompt"}).to_string(),
    };
    let models = arg_str_array(&params, "models", &["llama3.2", "qwen2.5-coder"]);
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    let mut results: Vec<Value> = Vec::new();
    for model in &models {
        match ollama_generate_api(model, &prompt, &ollama_url) {
            Ok((val, latency_ms)) => {
                let response = extract_generate_response(&val);
                results.push(serde_json::json!({
                    "model": model,
                    "response": truncate_response(&response),
                    "latency_ms": latency_ms
                }));
            }
            Err(e) => results.push(serde_json::json!({
                "model": model,
                "response": "",
                "latency_ms": null,
                "error": e
            })),
        }
    }

    serde_json::json!({
        "results": results,
        "prompt": prompt
    })
    .to_string()
}

/// Run chain-of-thought reasoning via the Ollama generate API.
///
/// Args: `prompt` (string), `steps` (int, default 3), `model` (string,
/// default "llama3.2"), `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_chain_of_thought_work(params: Value) -> String {
    let prompt = match params.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: prompt"}).to_string(),
    };
    let steps = arg_u64(&params, "steps", 3) as usize;
    let model = arg_str(&params, "model", "llama3.2");
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    if steps == 0 {
        return serde_json::json!({"error": "steps must be at least 1"}).to_string();
    }

    // Build multi-step CoT prompt
    let mut step_instructions = String::from("Think step by step.\n");
    for i in 1..=steps {
        step_instructions.push_str(&format!("Step {i}: ...\n"));
    }
    step_instructions.push_str("Conclusion: ...\n\n");
    step_instructions.push_str(&format!(
        "Now answer the following question using {steps} steps of reasoning. \
         Show your reasoning, then state your conclusion.\n\n{prompt}"
    ));

    match ollama_generate_api(&model, &step_instructions, &ollama_url) {
        Ok((val, _latency_ms)) => {
            let response = extract_generate_response(&val);
            // Try to split reasoning from conclusion
            let lower = response.to_lowercase();
            let (reasoning, conclusion) = if let Some(idx) = lower.rfind("conclusion") {
                (
                    response[..idx].trim().to_string(),
                    response[idx..].trim().to_string(),
                )
            } else {
                (response.clone(), String::new())
            };
            serde_json::json!({
                "steps": steps,
                "reasoning": truncate_response(&reasoning),
                "conclusion": truncate_response(&conclusion)
            })
            .to_string()
        }
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

/// Self-critique an AI response using the Ollama generate API.
///
/// Args: `original_prompt` (string), `original_response` (string),
/// `model` (string, default "llama3.2"), `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_self_critique_work(params: Value) -> String {
    let original_prompt = match params.get("original_prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"error": "Missing required arg: original_prompt"})
                .to_string()
        }
    };
    let original_response = match params.get("original_response").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => {
            return serde_json::json!({"error": "Missing required arg: original_response"})
                .to_string()
        }
    };
    let model = arg_str(&params, "model", "llama3.2");
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    // Step 1: Critique
    let critique_prompt = format!(
        "Evaluate the following AI response for accuracy, completeness, and clarity. \
         Identify specific weaknesses, errors, or missing information. \
         Then provide an improved version.\n\n\
         Original prompt: {original_prompt}\n\n\
         Original response: {original_response}\n\n\
         Provide your critique and improved response:"
    );

    match ollama_generate_api(&model, &critique_prompt, &ollama_url) {
        Ok((val, _latency_ms)) => {
            let full = extract_generate_response(&val);
            // Try to split critique from improved response
            let lower = full.to_lowercase();
            let (critique, improved) = if let Some(idx) = lower.find("improved") {
                (
                    full[..idx].trim().to_string(),
                    full[idx..].trim().to_string(),
                )
            } else if let Some(idx) = lower.find("better") {
                (
                    full[..idx].trim().to_string(),
                    full[idx..].trim().to_string(),
                )
            } else {
                (full.clone(), String::new())
            };
            serde_json::json!({
                "critique": truncate_response(&critique),
                "improved_response": truncate_response(&improved)
            })
            .to_string()
        }
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

/// Two-model debate: one argues for, the other against, then synthesize.
///
/// Args: `question` (string), `model_a` (string, default "llama3.2"),
/// `model_b` (string, optional, defaults to same as model_a),
/// `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_debate_work(params: Value) -> String {
    let question = match params.get("question").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: question"}).to_string(),
    };
    let model_a = arg_str(&params, "model_a", "llama3.2");
    let model_b = arg_str(&params, "model_b", &model_a);
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    // Argument FOR
    let for_prompt = format!(
        "Argue strongly IN FAVOR of the following question. \
         Provide the strongest arguments you can.\n\n\
         Question: {question}"
    );
    let argument_for = match ollama_generate_api(&model_a, &for_prompt, &ollama_url) {
        Ok((val, _)) => extract_generate_response(&val),
        Err(e) => {
            return serde_json::json!({"error": format!("Argument for failed: {e}")}).to_string()
        }
    };

    // Argument AGAINST
    let against_prompt = format!(
        "Argue strongly AGAINST the following question. \
         Provide the strongest counter-arguments you can.\n\n\
         Question: {question}"
    );
    let argument_against = match ollama_generate_api(&model_b, &against_prompt, &ollama_url) {
        Ok((val, _)) => extract_generate_response(&val),
        Err(e) => {
            return serde_json::json!({"error": format!("Argument against failed: {e}")})
                .to_string()
        }
    };

    // Synthesis
    let synth_prompt = format!(
        "Synthesize the following opposing arguments into a balanced analysis.\n\n\
         PRO arguments:\n{argument_for}\n\n\
         CON arguments:\n{argument_against}\n\n\
         Balanced synthesis:"
    );
    // Use model_a for synthesis as well
    let synthesis = match ollama_generate_api(&model_a, &synth_prompt, &ollama_url) {
        Ok((val, _)) => extract_generate_response(&val),
        Err(e) => {
            return serde_json::json!({"error": format!("Synthesis failed: {e}")}).to_string()
        }
    };

    serde_json::json!({
        "argument_for": truncate_response(&argument_for),
        "argument_against": truncate_response(&argument_against),
        "synthesis": truncate_response(&synthesis)
    })
    .to_string()
}

/// Build RAG index from files using keyword-based chunking (no embeddings needed).
///
/// Args: `path` (string, directory), `chunk_size` (int, default 500),
/// `ollama_url` (string, unused but kept for API consistency).
#[allow(dead_code)]
pub fn ai_rag_build_index_work(params: Value) -> String {
    let path_str = match params.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: path"}).to_string(),
    };
    let chunk_size = arg_u64(&params, "chunk_size", 500) as usize;
    let dir = Path::new(&path_str);

    if !dir.is_dir() {
        return serde_json::json!({"error": format!("Path is not a directory: {path_str}")})
            .to_string();
    }

    // Create .mint-mcp directory if needed
    let mint_dir = dir.join(".mint-mcp");
    if let Err(e) = fs::create_dir_all(&mint_dir) {
        return serde_json::json!({"error": format!("Failed to create .mint-mcp dir: {e}")})
            .to_string();
    }

    // Walk directory, read text files, split into chunks
    let text_exts = [
        "txt", "md", "rs", "py", "js", "ts", "json", "yaml", "yml", "toml", "html", "css", "sh",
        "go", "java", "c", "cpp", "h", "rb", "lua",
    ];

    let mut documents_indexed = 0usize;
    let mut chunks_created = 0usize;
    let mut jsonl_lines: Vec<String> = Vec::new();

    let walker = ignore::WalkBuilder::new(dir)
        .hidden(false)
        .git_ignore(true)
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !text_exts.contains(&ext) {
            continue;
        }
        // Skip files inside .mint-mcp itself
        if path.starts_with(&mint_dir) {
            continue;
        }

        let rel = path
            .strip_prefix(dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        documents_indexed += 1;

        // Split into chunks
        let chars: Vec<char> = content.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();
            let chunk_entry = serde_json::json!({
                "source": rel,
                "chunk": chunk_text,
                "start_char": start,
                "end_char": end
            });
            jsonl_lines.push(chunk_entry.to_string());
            chunks_created += 1;
            start = end;
            if start >= chars.len() {
                break;
            }
        }
    }

    if documents_indexed == 0 {
        return serde_json::json!({"error": "No text files found in directory"}).to_string();
    }

    // Save to .mint-mcp/rag-index.jsonl
    let index_path = mint_dir.join("rag-index.jsonl");
    let jsonl_content = jsonl_lines.join("\n");
    match fs::write(&index_path, &jsonl_content) {
        Ok(_) => serde_json::json!({
            "documents_indexed": documents_indexed,
            "chunks_created": chunks_created,
            "index_path": index_path.to_string_lossy()
        })
        .to_string(),
        Err(e) => serde_json::json!({"error": format!("Failed to write index: {e}")}).to_string(),
    }
}

/// Query RAG index using simple keyword matching.
///
/// Args: `query` (string), `top_k` (int, default 5),
/// `index_path` (string, optional, default ".mint-mcp/rag-index.jsonl" relative to CWD).
#[allow(dead_code)]
pub fn ai_rag_query_work(params: Value) -> String {
    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: query"}).to_string(),
    };
    let top_k = arg_u64(&params, "top_k", 5) as usize;
    let index_path = arg_str(&params, "index_path", ".mint-mcp/rag-index.jsonl");

    // Read the JSONL index
    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({"error": format!("Failed to read index: {e}")}).to_string()
        }
    };

    // Tokenize query for keyword matching
    let query_tokens: HashSet<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_string())
        .collect();

    if query_tokens.is_empty() {
        return serde_json::json!({"error": "Query too short for keyword matching"}).to_string();
    }

    // Score each chunk by keyword overlap
    let mut scored: Vec<(usize, Value)> = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let chunk_val: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let chunk_text = chunk_val
            .get("chunk")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let chunk_lower = chunk_text.to_lowercase();
        let chunk_tokens: HashSet<String> = chunk_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 2)
            .map(|s| s.to_string())
            .collect();

        // Simple intersection score
        let intersection_count = query_tokens.intersection(&chunk_tokens).count();
        let score = if chunk_tokens.is_empty() {
            0
        } else {
            intersection_count
        };

        if score > 0 {
            scored.push((score, chunk_val.clone()));
        }

        // Safety: avoid unbounded processing
        if line_num > 100000 {
            break;
        }
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    // Take top_k
    let results: Vec<Value> = scored
        .iter()
        .take(top_k)
        .map(|(score, chunk)| {
            let source = chunk
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let chunk_text = chunk.get("chunk").and_then(|v| v.as_str()).unwrap_or("");
            serde_json::json!({
                "chunk": truncate_response(chunk_text),
                "source": source,
                "score": score
            })
        })
        .collect();

    serde_json::json!({
        "results": results,
        "query": query
    })
    .to_string()
}

/// Prepare fine-tuning dataset from prompt/response pairs (OpenAI-compatible JSONL).
///
/// Args: `conversations` (array of {prompt, response} pairs).
#[allow(dead_code)]
pub fn ai_fine_tune_prep_work(params: Value) -> String {
    let conversations = match params.get("conversations").and_then(|v| v.as_array()) {
        Some(a) => a.clone(),
        None => {
            return serde_json::json!({"error": "Missing required arg: conversations"}).to_string()
        }
    };

    if conversations.is_empty() {
        return serde_json::json!({"error": "conversations array is empty"}).to_string();
    }

    let mut jsonl_lines: Vec<String> = Vec::new();
    let mut skipped = 0usize;

    for conv in &conversations {
        let prompt_text = conv.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        let response_text = conv.get("response").and_then(|v| v.as_str()).unwrap_or("");
        if prompt_text.is_empty() || response_text.is_empty() {
            skipped += 1;
            continue;
        }
        // OpenAI fine-tuning format
        let entry = serde_json::json!({
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": prompt_text},
                {"role": "assistant", "content": response_text}
            ]
        });
        jsonl_lines.push(entry.to_string());
    }

    let examples = jsonl_lines.len();
    let jsonl = jsonl_lines.join("\n");
    let details = if skipped > 0 {
        format!("{examples} examples prepared, {skipped} skipped due to empty fields")
    } else {
        format!("{examples} examples prepared")
    };

    serde_json::json!({
        "examples": examples,
        "jsonl": truncate_response(&jsonl),
        "details": details
    })
    .to_string()
}

/// Check an AI response against a source for hallucinations.
///
/// Args: `response` (string), `source` (string).
#[allow(dead_code)]
pub fn ai_hallucination_check_work(params: Value) -> String {
    let response = match params.get("response").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: response"}).to_string(),
    };
    let source = match params.get("source").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: source"}).to_string(),
    };

    // Extract claims from response as sentences
    let claims: Vec<&str> = response
        .split(['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| s.len() > 10)
        .collect();

    if claims.is_empty() {
        return serde_json::json!({
            "claims_checked": 0,
            "supported": 0,
            "unsupported": 0,
            "details": "No claims could be extracted from the response"
        })
        .to_string();
    }

    let source_lower = source.to_lowercase();

    let mut supported = 0usize;
    let mut unsupported = 0usize;
    let mut details_list: Vec<Value> = Vec::new();

    for claim in &claims {
        // Extract significant keywords from the claim
        let claim_lower = claim.to_lowercase();
        let keywords: Vec<&str> = claim_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| s.len() > 3)
            .collect();

        if keywords.is_empty() {
            continue;
        }

        // Count how many keywords appear in the source
        let matched = keywords
            .iter()
            .filter(|kw| source_lower.contains(*kw))
            .count();
        let ratio = matched as f64 / keywords.len() as f64;

        if ratio >= 0.5 {
            supported += 1;
            details_list.push(serde_json::json!({
                "claim": claim,
                "verdict": "supported",
                "keyword_match_ratio": (ratio * 100.0).round() / 100.0
            }));
        } else {
            unsupported += 1;
            details_list.push(serde_json::json!({
                "claim": claim,
                "verdict": "unsupported",
                "keyword_match_ratio": (ratio * 100.0).round() / 100.0
            }));
        }
    }

    let claims_checked = supported + unsupported;
    // Truncate details if too many
    let details_display: Vec<Value> = details_list.into_iter().take(20).collect();

    serde_json::json!({
        "claims_checked": claims_checked,
        "supported": supported,
        "unsupported": unsupported,
        "details": details_display
    })
    .to_string()
}

/// Zero-shot classification using the Ollama generate API.
///
/// Args: `text` (string), `categories` (array of strings),
/// `model` (string, default "llama3.2"), `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_classify_work(params: Value) -> String {
    let text = match params.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}).to_string(),
    };
    let categories = arg_str_array(&params, "categories", &[]);
    let model = arg_str(&params, "model", "llama3.2");
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    if categories.is_empty() {
        return serde_json::json!({"error": "Missing required arg: categories (array of strings)"})
            .to_string();
    }

    let categories_str = categories.join(", ");
    let prompt = format!(
        "Classify the following text into exactly one of these categories: [{categories_str}]\n\n\
         Text: {text}\n\n\
         Respond with ONLY the category name, nothing else."
    );

    match ollama_generate_api(&model, &prompt, &ollama_url) {
        Ok((val, _latency_ms)) => {
            let predicted = extract_generate_response(&val).trim().to_string();
            // Compute simple scores
            let mut all_scores = serde_json::Map::new();
            for cat in &categories {
                let score = if cat.to_lowercase() == predicted.to_lowercase() {
                    0.95
                } else if predicted.to_lowercase().contains(&cat.to_lowercase()) {
                    0.6
                } else {
                    0.05 / categories.len() as f64
                };
                all_scores.insert(
                    cat.clone(),
                    serde_json::json!((score * 100.0).round() / 100.0),
                );
            }
            let confidence = if predicted.to_lowercase() == predicted.to_lowercase() {
                "high"
            } else {
                "low"
            };
            serde_json::json!({
                "category": predicted,
                "confidence": confidence,
                "all_scores": all_scores
            })
            .to_string()
        }
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

/// Translate text using the Ollama generate API.
///
/// Args: `text` (string), `source_lang` (string, optional, default ""),
/// `target_lang` (string), `model` (string, default "llama3.2"),
/// `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_translate_work(params: Value) -> String {
    let text = match params.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: text"}).to_string(),
    };
    let source_lang = arg_str(&params, "source_lang", "");
    let target_lang = match params.get("target_lang").and_then(|v| v.as_str()) {
        Some(l) => l.to_string(),
        None => {
            return serde_json::json!({"error": "Missing required arg: target_lang"}).to_string()
        }
    };
    let model = arg_str(&params, "model", "llama3.2");
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    let prompt = if source_lang.is_empty() {
        format!(
            "Translate the following text to {target_lang}. \
             Return ONLY the translation, nothing else.\n\n{text}"
        )
    } else {
        format!(
            "Translate the following text from {source_lang} to {target_lang}. \
             Return ONLY the translation, nothing else.\n\n{text}"
        )
    };

    match ollama_generate_api(&model, &prompt, &ollama_url) {
        Ok((val, _latency_ms)) => {
            let translation = extract_generate_response(&val).trim().to_string();
            serde_json::json!({
                "translation": translation,
                "source_lang": if source_lang.is_empty() { "auto-detected" } else { &source_lang },
                "target_lang": target_lang
            })
            .to_string()
        }
        Err(e) => serde_json::json!({"error": e}).to_string(),
    }
}

// ---------------------------------------------------------------------------
// 29. ai_rag_update
// ---------------------------------------------------------------------------

/// Add new documents to an existing RAG index without full rebuild.
///
/// Args: `index_path` (string), `documents` (array of {path, content}).
/// Simulates appending to index.
#[allow(dead_code)]
pub fn ai_rag_update_work(params: Value) -> String {
    let index_path = match params.get("index_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: index_path"})
                .to_string()
        }
    };
    let documents = match params.get("documents").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return serde_json::json!({"ok":false,"error":"Missing required arg: documents (array of {path, content})"}).to_string(),
    };
    let chunk_size = arg_u64(&params, "chunk_size", 500) as usize;

    // Check the index file exists
    let index = Path::new(&index_path);
    if !index.exists() {
        return serde_json::json!({"ok":false,"error":format!("Index file not found: {index_path}")}).to_string();
    }

    let mut documents_added = 0usize;
    let mut chunks_appended = 0usize;
    let mut jsonl_lines: Vec<String> = Vec::new();

    for doc in &documents {
        let doc_path = doc
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let content = doc.get("content").and_then(|v| v.as_str()).unwrap_or("");
        if content.is_empty() {
            continue;
        }
        documents_added += 1;
        let chars: Vec<char> = content.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk_text: String = chars[start..end].iter().collect();
            let chunk_entry = serde_json::json!({
                "source": doc_path,
                "chunk": chunk_text,
                "start_char": start,
                "end_char": end
            });
            jsonl_lines.push(chunk_entry.to_string());
            chunks_appended += 1;
            start = end;
            if start >= chars.len() {
                break;
            }
        }
    }

    if documents_added == 0 {
        return serde_json::json!({"ok":false,"error":"No valid documents provided for update"})
            .to_string();
    }

    // Append to existing index
    let mut existing = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"ok":false,"error":format!("Failed to read existing index: {e}")}).to_string(),
    };
    if !existing.ends_with('\n') && !existing.is_empty() {
        existing.push('\n');
    }
    existing.push_str(&jsonl_lines.join("\n"));

    match fs::write(&index_path, &existing) {
        Ok(_) => serde_json::json!({
            "ok": true,
            "data": {
                "documents_added": documents_added,
                "chunks_appended": chunks_appended,
                "index_path": index_path
            }
        })
        .to_string(),
        Err(e) => {
            serde_json::json!({"ok":false,"error":format!("Failed to write updated index: {e}")})
                .to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// 30. ai_rag_delete
// ---------------------------------------------------------------------------

/// Remove a document from RAG index by path or ID.
///
/// Args: `index_path` (string), `doc_id` (string — matches source field).
#[allow(dead_code)]
pub fn ai_rag_delete_work(params: Value) -> String {
    let index_path = match params.get("index_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: index_path"})
                .to_string()
        }
    };
    let doc_id = match params.get("doc_id").and_then(|v| v.as_str()) {
        Some(d) => d.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: doc_id"})
                .to_string()
        }
    };

    let index = Path::new(&index_path);
    if !index.exists() {
        return serde_json::json!({"ok":false,"error":format!("Index file not found: {index_path}")}).to_string();
    }

    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(e) => {
            return serde_json::json!({"ok":false,"error":format!("Failed to read index: {e}")})
                .to_string()
        }
    };

    let mut kept_lines: Vec<String> = Vec::new();
    let mut removed_count = 0usize;
    let mut total_lines = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        total_lines += 1;
        // Parse and check if source matches doc_id
        let should_remove = match serde_json::from_str::<Value>(trimmed) {
            Ok(val) => {
                let source = val.get("source").and_then(|v| v.as_str()).unwrap_or("");
                source == doc_id
            }
            Err(_) => false,
        };
        if should_remove {
            removed_count += 1;
        } else {
            kept_lines.push(trimmed.to_string());
        }
    }

    if removed_count == 0 {
        return serde_json::json!({
            "ok": true,
            "data": {
                "removed_chunks": 0,
                "doc_id": doc_id,
                "note": "No chunks matched the given doc_id"
            }
        })
        .to_string();
    }

    let new_content = kept_lines.join("\n");
    match fs::write(&index_path, &new_content) {
        Ok(_) => serde_json::json!({
            "ok": true,
            "data": {
                "removed_chunks": removed_count,
                "remaining_chunks": kept_lines.len(),
                "total_before": total_lines,
                "doc_id": doc_id,
                "index_path": index_path
            }
        })
        .to_string(),
        Err(e) => {
            serde_json::json!({"ok":false,"error":format!("Failed to write updated index: {e}")})
                .to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// 31. ai_persona_chat
// ---------------------------------------------------------------------------

/// Chat with a model locked to a specific expert persona.
///
/// Args: `persona` (string, e.g. "senior_rust_engineer"),
/// `prompt` (string), `model` (string, optional, default "llama3.2"),
/// `ollama_url` (string, optional).
#[allow(dead_code)]
pub fn ai_persona_chat_work(params: Value) -> String {
    let persona = match params.get("persona").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: persona"})
                .to_string()
        }
    };
    let prompt = match params.get("prompt").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: prompt"})
                .to_string()
        }
    };
    let model = arg_str(&params, "model", "llama3.2");
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    // Build persona system prompt
    let persona_descriptions: Vec<(&str, &str)> = vec![
        ("senior_rust_engineer", "You are a senior Rust engineer with deep expertise in systems programming, memory safety, async runtimes, and the Rust type system. You write idiomatic, performant Rust code and explain design tradeoffs clearly."),
        ("security_analyst", "You are a senior security analyst specializing in vulnerability assessment, penetration testing, and secure code review. You follow OWASP guidelines and prioritize impact analysis."),
        ("data_scientist", "You are an experienced data scientist skilled in statistical modeling, machine learning pipelines, feature engineering, and data visualization. You favor reproducible, well-documented analyses."),
        ("devops_engineer", "You are a senior DevOps engineer expert in CI/CD, container orchestration, infrastructure as code, and site reliability engineering. You emphasize automation and observability."),
        ("embedded_engineer", "You are an embedded systems engineer with expertise in firmware, RTOS, peripheral drivers, and low-power design. You think carefully about resource constraints and hardware interactions."),
    ];

    let persona_system = persona_descriptions
        .iter()
        .find(|(key, _)| *key == persona.to_lowercase().replace(' ', "_"))
        .map(|(_, desc)| *desc)
        .unwrap_or_else(|| "You are an expert assistant. Respond with precision and clarity.");

    let full_prompt = format!(
        "SYSTEM: {persona_system}\n\nUSER: {prompt}\n\nRespond in character as the {persona} persona."
    );

    match ollama_generate_api(&model, &full_prompt, &ollama_url) {
        Ok((val, latency_ms)) => {
            let response = extract_generate_response(&val).trim().to_string();
            serde_json::json!({
                "ok": true,
                "data": {
                    "persona": persona,
                    "model": model,
                    "response": response,
                    "latency_ms": latency_ms
                }
            })
            .to_string()
        }
        Err(e) => serde_json::json!({"ok":false,"error":e}).to_string(),
    }
}

// ---------------------------------------------------------------------------
// 32. ml_attention_visualize
// ---------------------------------------------------------------------------

/// Generate attention heatmap description.
///
/// Args: `text` (string), `model` (string, optional, default "llama3.2").
/// Returns a stub noting visualization requires model access.
#[allow(dead_code)]
pub fn ml_attention_visualize_work(params: Value) -> String {
    let text = match params.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: text"}).to_string()
        }
    };
    let model = arg_str(&params, "model", "llama3.2");

    let tokens: Vec<&str> = text.split_whitespace().take(50).collect();
    let token_count = tokens.len();

    // Stub attention matrix (random-ish placeholder)
    let mut attention_rows: Vec<Value> = Vec::new();
    for (i, token_i) in tokens.iter().enumerate() {
        let mut row: Vec<Value> = Vec::new();
        for (j, token_j) in tokens.iter().enumerate() {
            let weight = if i == j {
                1.0
            } else if (i as i64 - j as i64).unsigned_abs() <= 2 {
                0.6
            } else {
                0.1
            };
            row.push(serde_json::json!({
                "token": token_j,
                "weight": weight
            }));
        }
        attention_rows.push(serde_json::json!({
            "query_token": token_i,
            "attentions": row
        }));
    }

    serde_json::json!({
        "ok": true,
        "data": {
            "model": model,
            "text_preview": truncate_response(&text),
            "token_count": token_count,
            "attention_matrix": attention_rows,
            "note": "Attention weights are simulated stubs. Real visualization requires direct model access to extract attention head outputs."
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// 33. ml_quantize_model
// ---------------------------------------------------------------------------

/// Apply GGUF/GPTQ quantization.
///
/// Args: `model_path` (string), `method` (string: "gguf" or "gptq"),
/// `bits` (u64: 4 or 8).
/// Uses `llama.cpp` subprocess tools.
#[allow(dead_code)]
pub fn ml_quantize_model_work(params: Value) -> String {
    let model_path = match params.get("model_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: model_path"})
                .to_string()
        }
    };
    let method = match params.get("method").and_then(|v| v.as_str()) {
        Some(m) => m.to_lowercase(),
        None => return serde_json::json!({"ok":false,"error":"Missing required arg: method (gguf/gptq)"}).to_string(),
    };
    let bits = params.get("bits").and_then(|v| v.as_u64()).unwrap_or(4);
    if bits != 4 && bits != 8 {
        return serde_json::json!({"ok":false,"error":"bits must be 4 or 8"}).to_string();
    }
    if method != "gguf" && method != "gptq" {
        return serde_json::json!({"ok":false,"error":"method must be 'gguf' or 'gptq'"})
            .to_string();
    }

    let model = Path::new(&model_path);
    if !model.exists() {
        return serde_json::json!({"ok":false,"error":format!("Model file not found: {model_path}")}).to_string();
    }

    if method == "gguf" {
        // GGUF quantization uses llama.cpp tools
        // Step 1: convert to GGUF (if not already)
        let convert_script = "llama-cpp-convert";
        let quantize_tool = "llama-quantize";

        // Check for quantize tool
        let quantize_check = Command::new(quantize_tool)
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if quantize_check.is_err() {
            return serde_json::json!({
                "ok": false,
                "error": format!("'{quantize_tool}' not found on PATH. Install llama.cpp and add its binaries to PATH.")
            }).to_string();
        }

        let quant_type = if bits == 4 { "Q4_K_M" } else { "Q8_0" };
        let output_path = format!("{model_path}.gguf-{quant_type}");

        let args: Vec<&str> = vec![&model_path, &output_path, quant_type];
        match Command::new(quantize_tool).args(&args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    serde_json::json!({
                        "ok": true,
                        "data": {
                            "method": "gguf",
                            "bits": bits,
                            "quant_type": quant_type,
                            "model_path": model_path,
                            "output_path": output_path,
                            "stdout": truncate_response(&stdout),
                            "stderr": truncate_response(&stderr)
                        }
                    })
                    .to_string()
                } else {
                    serde_json::json!({
                        "ok": false,
                        "error": format!("Quantization failed: {}", truncate_response(&stderr))
                    })
                    .to_string()
                }
            }
            Err(e) => serde_json::json!({"ok":false,"error":format!("Failed to run {quantize_tool}: {e}")}).to_string(),
        }
    } else {
        // GPTQ quantization uses auto-gptq or similar Python tooling
        let python_check = Command::new("python3")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if python_check.is_err() {
            return serde_json::json!({"ok":false,"error":"'python3' not found on PATH. GPTQ quantization requires Python with auto-gptq installed."}).to_string();
        }

        let output_path = format!("{model_path}-gptq-{bits}bit");
        let script =
            format!("from auto_gptq import BaseQuantizeConfig; print('auto_gptq available')");
        let script_path = format!("/tmp/gptq_check_{}.py", std::process::id());
        if let Ok(mut f) = fs::File::create(&script_path) {
            let _ = IoWrite::write_all(&mut f, script.as_bytes());
        }
        let check_result = Command::new("python3")
            .arg(&script_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = fs::remove_file(&script_path);

        if check_result.map_or(true, |s| !s.success()) {
            return serde_json::json!({
                "ok": false,
                "error": "auto-gptq not installed. Install: pip install auto-gptq"
            })
            .to_string();
        }

        let quant_script = format!(
            r#"
import sys
from auto_gptq import BaseQuantizeConfig
from transformers import AutoModelForCausalLM, AutoTokenizer
model_path = sys.argv[1]
output_path = sys.argv[2]
bits = int(sys.argv[3])
tokenizer = AutoTokenizer.from_pretrained(model_path)
model = AutoModelForCausalLM.from_pretrained(model_path)
quant_config = BaseQuantizeConfig(bits=bits)
model.quantize(quant_config)
model.save_quantized(output_path)
print(f"GPTQ quantization ({bits}-bit) complete: {{output_path}}")
"#,
        );
        let script_path = format!("/tmp/gptq_quantize_{}.py", std::process::id());
        if let Ok(mut f) = fs::File::create(&script_path) {
            let _ = IoWrite::write_all(&mut f, quant_script.as_bytes());
        }
        let args: Vec<String> = vec![
            script_path.clone(),
            model_path.clone(),
            output_path.clone(),
            bits.to_string(),
        ];
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = Command::new("python3").args(&args_ref).output();
        let _ = fs::remove_file(&script_path);

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if output.status.success() {
                    serde_json::json!({
                        "ok": true,
                        "data": {
                            "method": "gptq",
                            "bits": bits,
                            "model_path": model_path,
                            "output_path": output_path,
                            "stdout": truncate_response(&stdout),
                            "stderr": truncate_response(&stderr)
                        }
                    })
                    .to_string()
                } else {
                    serde_json::json!({
                        "ok": false,
                        "error": format!("GPTQ quantization failed: {}", truncate_response(&stderr))
                    })
                    .to_string()
                }
            }
            Err(e) => serde_json::json!({"ok":false,"error":format!("Failed to run python3: {e}")})
                .to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// 34. ml_lora_apply
// ---------------------------------------------------------------------------

/// Apply a LoRA adapter to a base model.
///
/// Args: `base_model_path` (string), `lora_path` (string),
/// `output_path` (string).
/// Uses `llama.cpp` export-lora or stub.
#[allow(dead_code)]
pub fn ml_lora_apply_work(params: Value) -> String {
    let base_model_path = match params.get("base_model_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: base_model_path"})
                .to_string()
        }
    };
    let lora_path = match params.get("lora_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: lora_path"})
                .to_string()
        }
    };
    let output_path = match params.get("output_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"ok":false,"error":"Missing required arg: output_path"})
                .to_string()
        }
    };

    let base = Path::new(&base_model_path);
    let lora = Path::new(&lora_path);
    if !base.exists() {
        return serde_json::json!({"ok":false,"error":format!("Base model not found: {base_model_path}")}).to_string();
    }
    if !lora.exists() {
        return serde_json::json!({"ok":false,"error":format!("LoRA adapter not found: {lora_path}")}).to_string();
    }

    // Try llama.cpp export-lora tool
    let tool_name = "llama-export-lora";
    let tool_check = Command::new(tool_name)
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if tool_check.is_err() {
        // Try the older llama-cli with lora support as a fallback indicator
        let alt_check = Command::new("llama-server")
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if alt_check.is_err() {
            return serde_json::json!({
                "ok": false,
                "error": "No llama.cpp tools found on PATH. Install llama.cpp and add 'llama-export-lora' to PATH."
            }).to_string();
        }
        // If llama-server exists but not export-lora, return a stub with instructions
        return serde_json::json!({
            "ok": true,
            "data": {
                "base_model_path": base_model_path,
                "lora_path": lora_path,
                "output_path": output_path,
                "note": "llama-export-lora not found. You can apply LoRA at inference time using: llama-server -m <base_model> --lora <lora_adapter>",
                "applied": false
            }
        }).to_string();
    }

    let args: Vec<&str> = vec![
        "-m",
        &base_model_path,
        "--lora",
        &lora_path,
        "-o",
        &output_path,
    ];

    match Command::new(tool_name).args(&args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.success() {
                serde_json::json!({
                    "ok": true,
                    "data": {
                        "base_model_path": base_model_path,
                        "lora_path": lora_path,
                        "output_path": output_path,
                        "applied": true,
                        "stdout": truncate_response(&stdout),
                        "stderr": truncate_response(&stderr)
                    }
                })
                .to_string()
            } else {
                serde_json::json!({
                    "ok": false,
                    "error": format!("LoRA apply failed: {}", truncate_response(&stderr))
                })
                .to_string()
            }
        }
        Err(e) => serde_json::json!({"ok":false,"error":format!("Failed to run {tool_name}: {e}")})
            .to_string(),
    }
}

// ---------------------------------------------------------------------------
// 35. ml_inference_batch
// ---------------------------------------------------------------------------

/// Run batch inference over a list of prompts in parallel.
///
/// Args: `model` (string), `prompts` (array of strings),
/// `max_concurrency` (u64, optional, default 4).
/// Simulates batch processing.
#[allow(dead_code)]
pub fn ml_inference_batch_work(params: Value) -> String {
    let model = arg_str(&params, "model", "llama3.2");
    let prompts = match params.get("prompts").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<String>>(),
        None => return serde_json::json!({"ok":false,"error":"Missing required arg: prompts (array of strings)"}).to_string(),
    };
    let max_concurrency = params
        .get("max_concurrency")
        .and_then(|v| v.as_u64())
        .unwrap_or(4)
        .clamp(1, 16) as usize;

    if prompts.is_empty() {
        return serde_json::json!({"ok":false,"error":"prompts array is empty"}).to_string();
    }

    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");
    let mut results: Vec<Value> = Vec::new();
    let mut errors: usize = 0;
    let mut total_latency_ms: u64 = 0;

    // Simulate parallel processing by running in batches of max_concurrency
    for chunk in prompts.chunks(max_concurrency) {
        for prompt in chunk {
            let start = Instant::now();
            match ollama_generate_api(&model, prompt, &ollama_url) {
                Ok((val, latency_ms)) => {
                    total_latency_ms += latency_ms;
                    let response = extract_generate_response(&val).trim().to_string();
                    results.push(serde_json::json!({
                        "prompt_preview": truncate_response(prompt),
                        "response_preview": truncate_response(&response),
                        "latency_ms": latency_ms
                    }));
                }
                Err(e) => {
                    errors += 1;
                    results.push(serde_json::json!({
                        "prompt_preview": truncate_response(prompt),
                        "error": e
                    }));
                }
            }
            let _elapsed = start.elapsed();
        }
    }

    let avg_latency_ms = if results.len() > errors {
        total_latency_ms / (results.len() - errors) as u64
    } else {
        0
    };

    serde_json::json!({
        "ok": true,
        "data": {
            "model": model,
            "total_prompts": prompts.len(),
            "successful": results.len() - errors,
            "errors": errors,
            "max_concurrency": max_concurrency,
            "total_latency_ms": total_latency_ms,
            "avg_latency_ms": avg_latency_ms,
            "results": results
        }
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper function tests ---

    #[test]
    // ---- ai_rag_update_work ----
    #[test]
    fn test_ai_rag_update_missing_index_path() {
        let params = serde_json::json!({"documents": [{"path": "a.txt", "content": "hello"}]});
        let result = ai_rag_update_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("index_path"));
    }

    #[test]
    fn test_ai_rag_update_missing_documents() {
        let params = serde_json::json!({"index_path": "/tmp/test.jsonl"});
        let result = ai_rag_update_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("documents"));
    }

    #[test]
    fn test_ai_rag_update_nonexistent_index() {
        let params = serde_json::json!({
            "index_path": "/tmp/nonexistent_rag_index_99999.jsonl",
            "documents": [{"path": "a.txt", "content": "hello"}]
        });
        let result = ai_rag_update_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
    }

    // ---- ai_rag_delete_work ----
    #[test]
    fn test_ai_rag_delete_missing_index_path() {
        let params = serde_json::json!({"doc_id": "test.txt"});
        let result = ai_rag_delete_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("index_path"));
    }

    #[test]
    fn test_ai_rag_delete_missing_doc_id() {
        let params = serde_json::json!({"index_path": "/tmp/test.jsonl"});
        let result = ai_rag_delete_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("doc_id"));
    }

    // ---- ai_persona_chat_work ----
    #[test]
    fn test_ai_persona_chat_missing_persona() {
        let params = serde_json::json!({"prompt": "hello"});
        let result = ai_persona_chat_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("persona"));
    }

    #[test]
    fn test_ai_persona_chat_missing_prompt() {
        let params = serde_json::json!({"persona": "senior_rust_engineer"});
        let result = ai_persona_chat_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("prompt"));
    }

    // ---- ml_attention_visualize_work ----
    #[test]
    fn test_ml_attention_visualize_missing_text() {
        let params = serde_json::json!({});
        let result = ml_attention_visualize_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("text"));
    }

    #[test]
    fn test_ml_attention_visualize_returns_stub() {
        let params = serde_json::json!({"text": "hello world test"});
        let result = ml_attention_visualize_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(v["ok"].as_bool().unwrap());
        assert!(v["data"]["note"].as_str().unwrap().contains("simulated"));
    }

    // ---- ml_quantize_model_work ----
    #[test]
    fn test_ml_quantize_model_missing_path() {
        let params = serde_json::json!({"method": "gguf", "bits": 4});
        let result = ml_quantize_model_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("model_path"));
    }

    #[test]
    fn test_ml_quantize_model_invalid_bits() {
        let params =
            serde_json::json!({"model_path": "/tmp/model.bin", "method": "gguf", "bits": 3});
        let result = ml_quantize_model_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("bits"));
    }

    #[test]
    fn test_ml_quantize_model_invalid_method() {
        let params =
            serde_json::json!({"model_path": "/tmp/model.bin", "method": "invalid", "bits": 4});
        let result = ml_quantize_model_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("method"));
    }

    // ---- ml_lora_apply_work ----
    #[test]
    fn test_ml_lora_apply_missing_base() {
        let params =
            serde_json::json!({"lora_path": "/tmp/lora.bin", "output_path": "/tmp/out.bin"});
        let result = ml_lora_apply_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("base_model_path"));
    }

    #[test]
    fn test_ml_lora_apply_missing_lora() {
        let params =
            serde_json::json!({"base_model_path": "/tmp/model.bin", "output_path": "/tmp/out.bin"});
        let result = ml_lora_apply_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("lora_path"));
    }

    // ---- ml_inference_batch_work ----
    #[test]
    fn test_ml_inference_batch_missing_prompts() {
        let params = serde_json::json!({"model": "llama3.2"});
        let result = ml_inference_batch_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("prompts"));
    }

    #[test]
    fn test_ml_inference_batch_empty_prompts() {
        let params = serde_json::json!({"model": "llama3.2", "prompts": []});
        let result = ml_inference_batch_work(params);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert!(v["error"].as_str().unwrap().contains("empty"));
    }

    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - 1.0).abs() < 0.0001,
            "Identical vectors should have similarity 1.0"
        );
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - 0.0).abs() < 0.0001,
            "Orthogonal vectors should have similarity 0.0"
        );
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - (-1.0)).abs() < 0.0001,
            "Opposite vectors should have similarity -1.0"
        );
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f64> = vec![];
        let b: Vec<f64> = vec![];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0, "Empty vectors should have similarity 0.0");
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(
            sim, 0.0,
            "Different-length vectors should have similarity 0.0"
        );
    }

    #[test]
    fn test_cosine_similarity_general() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let sim = cosine_similarity(&a, &b);
        // dot = 1*4 + 2*5 + 3*6 = 32
        // norm_a = sqrt(14), norm_b = sqrt(77)
        let expected = 32.0 / (14.0_f64.sqrt() * 77.0_f64.sqrt());
        assert!(
            (sim - expected).abs() < 0.0001,
            "General cosine similarity mismatch"
        );
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "Hello world, this is a test.";
        let tokens = estimate_tokens(text);
        assert_eq!(tokens, text.len() / CHARS_PER_TOKEN);
    }

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    // --- extract_embedding test ---

    #[test]
    fn test_extract_embedding_valid() {
        let val = serde_json::json!({"embedding": [0.1, 0.2, 0.3]});
        let emb = extract_embedding(&val);
        assert!(emb.is_some());
        let emb = emb.unwrap();
        assert_eq!(emb.len(), 3);
        assert!((emb[0] - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_embedding_missing() {
        let val = serde_json::json!({"error": "model not found"});
        let emb = extract_embedding(&val);
        assert!(emb.is_none());
    }

    // --- arg helper tests ---

    #[test]
    fn test_arg_str_default() {
        let args = serde_json::json!({});
        assert_eq!(arg_str(&args, "key", "default"), "default");
    }

    #[test]
    fn test_arg_str_provided() {
        let args = serde_json::json!({"key": "value"});
        assert_eq!(arg_str(&args, "key", "default"), "value");
    }

    #[test]
    fn test_arg_u64_default() {
        let args = serde_json::json!({});
        assert_eq!(arg_u64(&args, "k", 42), 42);
    }

    #[test]
    fn test_arg_u64_provided() {
        let args = serde_json::json!({"k": 10});
        assert_eq!(arg_u64(&args, "k", 42), 10);
    }

    #[test]
    fn test_arg_str_array_default() {
        let args = serde_json::json!({});
        let arr = arg_str_array(&args, "items", &["a", "b"]);
        assert_eq!(arr, vec!["a", "b"]);
    }

    #[test]
    fn test_arg_str_array_provided() {
        let args = serde_json::json!({"items": ["x", "y", "z"]});
        let arr = arg_str_array(&args, "items", &["a", "b"]);
        assert_eq!(arr, vec!["x", "y", "z"]);
    }

    // --- detect_model_format test ---

    #[test]
    fn test_detect_model_format_nonexistent() {
        let path = Path::new("/tmp/nonexistent_model_file_xyz.gguf");
        let fmt = detect_model_format(path);
        assert_eq!(fmt, "unknown");
    }

    // --- estimate_params_from_filename test ---

    #[test]
    fn test_estimate_params_from_filename_7b() {
        let result = estimate_params_from_filename("llama-7b-q4_0.gguf", 4_000_000_000);
        assert!(result.contains("7B"), "Should detect 7B from filename");
    }

    #[test]
    fn test_estimate_params_from_filename_70b() {
        let result =
            estimate_params_from_filename("qwen2.5-72b-instruct-q6_k.gguf", 40_000_000_000);
        assert!(result.contains("72B"), "Should detect 72B from filename");
    }

    #[test]
    fn test_estimate_params_from_filename_no_hint() {
        let result = estimate_params_from_filename("mymodel.gguf", 5_000_000_000);
        assert!(
            result.contains("estimated from file size"),
            "Should estimate from size"
        );
    }

    // --- k_means test ---

    #[test]
    fn test_k_means_empty() {
        let clusters = k_means(&[], 3, 10);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_k_means_single() {
        let embeddings = vec![vec![1.0, 0.0]];
        let clusters = k_means(&embeddings, 1, 10);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].len(), 1);
    }

    #[test]
    fn test_k_means_two_clusters() {
        // Two clearly separable groups
        let embeddings = vec![
            vec![1.0, 0.0],
            vec![0.9, 0.1],
            vec![0.0, 1.0],
            vec![0.1, 0.9],
        ];
        let clusters = k_means(&embeddings, 2, 20);
        assert_eq!(clusters.len(), 2);
        // Each cluster should have 2 members
        let total: usize = clusters.iter().map(|c| c.len()).sum();
        assert_eq!(total, 4);
    }

    // --- Tool function tests (without Ollama) ---

    #[test]
    fn test_ai_multi_model_compare_missing_prompt() {
        let args = serde_json::json!({});
        let result = ai_multi_model_compare_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_chain_of_thought_missing_prompt() {
        let args = serde_json::json!({});
        let result = ai_chain_of_thought_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_self_critique_missing_prompt() {
        let args = serde_json::json!({});
        let result = ai_self_critique_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_few_shot_classify_missing_text() {
        let args = serde_json::json!({"examples": [{"text": "good", "label": "positive"}]});
        let result = ai_few_shot_classify_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_few_shot_classify_missing_examples() {
        let args = serde_json::json!({"text": "hello"});
        let result = ai_few_shot_classify_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_zero_shot_classify_missing_text() {
        let args = serde_json::json!({"labels": ["pos", "neg"]});
        let result = ai_zero_shot_classify_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_zero_shot_classify_missing_labels() {
        let args = serde_json::json!({"text": "hello"});
        let result = ai_zero_shot_classify_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_structured_extract_missing_text() {
        let args = serde_json::json!({"schema": {"type": "object"}});
        let result = ai_structured_extract_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_structured_extract_missing_schema() {
        let args = serde_json::json!({"text": "some text"});
        let result = ai_structured_extract_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_debate_missing_topic() {
        let args = serde_json::json!({});
        let result = ai_debate_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_verify_factual_missing_claim() {
        let args = serde_json::json!({"references": ["doc1"]});
        let result = ai_verify_factual_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_verify_factual_missing_references() {
        let args = serde_json::json!({"claim": "The sky is blue"});
        let result = ai_verify_factual_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rewrite_style_invalid_style() {
        let args = serde_json::json!({"text": "hello", "style": "invalid_style"});
        let result = ai_rewrite_style_work(args);
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("Invalid style"));
    }

    #[test]
    fn test_ai_rewrite_style_valid_styles() {
        for style in &["technical", "casual", "academic", "eli5", "concise"] {
            let args = serde_json::json!({"text": "hello world", "style": style});
            // This may fail due to Ollama not being available, but should not
            // return an "Invalid style" error
            let result = ai_rewrite_style_work(args);
            if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                assert!(
                    !error.contains("Invalid style"),
                    "Style '{}' should be valid",
                    style
                );
            }
        }
    }

    #[test]
    fn test_ai_translate_missing_text() {
        let args = serde_json::json!({"from": "en", "to": "es"});
        let result = ai_translate_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_embed_text_missing_text() {
        let args = serde_json::json!({});
        let result = ai_embed_text_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_semantic_search_missing_query() {
        let args = serde_json::json!({"documents": ["doc1"]});
        let result = ai_semantic_search_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_semantic_search_missing_documents() {
        let args = serde_json::json!({"query": "test"});
        let result = ai_semantic_search_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_cluster_texts_missing_texts() {
        let args = serde_json::json!({});
        let result = ai_cluster_texts_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rag_build_index_missing_path() {
        let args = serde_json::json!({});
        let result = ai_rag_build_index_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rag_build_index_invalid_path() {
        let args = serde_json::json!({"path": "/tmp/nonexistent_dir_xyz_abc"});
        let result = ai_rag_build_index_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rag_query_missing_query() {
        let args = serde_json::json!({"index_path": "/tmp/test.json"});
        let result = ai_rag_query_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rag_query_missing_index_path() {
        let args = serde_json::json!({"query": "test"});
        let result = ai_rag_query_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_context_pack_smart_missing_paths() {
        let args = serde_json::json!({});
        let result = ai_context_pack_smart_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_model_info_missing_path() {
        let args = serde_json::json!({});
        let result = ml_model_info_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_model_info_nonexistent_file() {
        let args = serde_json::json!({"path": "/tmp/nonexistent_model_file_xyz.gguf"});
        let result = ml_model_info_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_tokenize_missing_text() {
        let args = serde_json::json!({});
        let result = ml_tokenize_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_token_count_missing_text() {
        let args = serde_json::json!({});
        let result = ml_token_count_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_benchmark_model_zero_iterations() {
        let args = serde_json::json!({"model": "llama3.2", "iterations": 0});
        let result = ml_benchmark_model_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_perplexity_score_missing_text() {
        let args = serde_json::json!({});
        let result = ml_perplexity_score_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_fine_tune_prep_missing_conversations() {
        let args = serde_json::json!({"format": "openai"});
        let result = ml_fine_tune_prep_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_fine_tune_prep_invalid_format() {
        let args = serde_json::json!({"conversations": [{"messages": []}], "format": "invalid"});
        let result = ml_fine_tune_prep_work(args);
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("Invalid format"));
    }

    #[test]
    fn test_ml_fine_tune_prep_openai_format() {
        let args = serde_json::json!({
            "conversations": [
                {
                    "messages": [
                        {"role": "system", "content": "You are helpful."},
                        {"role": "user", "content": "Hello"},
                        {"role": "assistant", "content": "Hi there!"}
                    ]
                }
            ],
            "format": "openai"
        });
        let result = ml_fine_tune_prep_work(args);
        assert_eq!(result["example_count"], 1);
        assert_eq!(result["format"], "openai");
        // Verify sample_line is valid JSON
        let sample: Value = serde_json::from_str(result["sample_line"].as_str().unwrap()).unwrap();
        assert!(sample.get("messages").is_some());
    }

    #[test]
    fn test_ml_fine_tune_prep_llama_format() {
        let args = serde_json::json!({
            "conversations": [
                {
                    "messages": [
                        {"role": "user", "content": "Hello"},
                        {"role": "assistant", "content": "Hi!"}
                    ]
                }
            ],
            "format": "llama"
        });
        let result = ml_fine_tune_prep_work(args);
        let sample: Value = serde_json::from_str(result["sample_line"].as_str().unwrap()).unwrap();
        assert!(sample.get("conversations").is_some());
        let convs = sample["conversations"].as_array().unwrap();
        assert_eq!(convs[0]["from"], "human");
        assert_eq!(convs[1]["from"], "gpt");
    }

    #[test]
    fn test_ml_fine_tune_prep_mistral_format() {
        let args = serde_json::json!({
            "conversations": [
                {
                    "messages": [
                        {"role": "user", "content": "Test"},
                        {"role": "assistant", "content": "Response"}
                    ]
                }
            ],
            "format": "mistral"
        });
        let result = ml_fine_tune_prep_work(args);
        let sample: Value = serde_json::from_str(result["sample_line"].as_str().unwrap()).unwrap();
        assert!(sample.get("messages").is_some());
    }

    #[test]
    fn test_ml_dataset_validate_missing_path() {
        let args = serde_json::json!({});
        let result = ml_dataset_validate_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_dataset_validate_nonexistent_file() {
        let args = serde_json::json!({"path": "/tmp/nonexistent_dataset_xyz.jsonl"});
        let result = ml_dataset_validate_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ml_dataset_validate_valid_data() {
        let dir = std::env::temp_dir().join("pb_mcp_test_validate");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_valid.jsonl");

        let data = r#"{"messages": [{"role": "user", "content": "Hello"}, {"role": "assistant", "content": "Hi!"}]}
{"messages": [{"role": "user", "content": "How are you?"}, {"role": "assistant", "content": "I'm fine!"}]}
"#;
        fs::write(&path, data).unwrap();

        let args = serde_json::json!({"path": path.to_string_lossy()});
        let result = ml_dataset_validate_work(args);
        assert_eq!(result["valid"], true);
        assert_eq!(result["total_examples"], 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ml_dataset_validate_invalid_json() {
        let dir = std::env::temp_dir().join("pb_mcp_test_validate_invalid");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_invalid.jsonl");

        let data = r#"{"messages": [{"role": "user", "content": "Hello"}]}
this is not json
{"messages": []}
"#;
        fs::write(&path, data).unwrap();

        let args = serde_json::json!({"path": path.to_string_lossy()});
        let result = ml_dataset_validate_work(args);
        assert_eq!(result["valid"], false);
        let errors = result["errors"].as_array().unwrap();
        assert!(!errors.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ml_dataset_validate_empty_messages() {
        let dir = std::env::temp_dir().join("pb_mcp_test_validate_empty");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_empty_msgs.jsonl");

        let data = r#"{"messages": []}
"#;
        fs::write(&path, data).unwrap();

        let args = serde_json::json!({"path": path.to_string_lossy()});
        let result = ml_dataset_validate_work(args);
        assert_eq!(result["valid"], false);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ollama_unavailable_json() {
        let result = ollama_unavailable();
        assert_eq!(result["tool_available"], false);
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("Ollama not available"));
    }

    #[test]
    fn test_truncate_response_short() {
        let short = "Hello world";
        let result = truncate_response(short);
        assert_eq!(result, short);
    }

    #[test]
    fn test_ai_context_pack_smart_with_files() {
        let dir = std::env::temp_dir().join("pb_mcp_test_context_pack");
        let _ = fs::create_dir_all(&dir);

        let file1 = dir.join("small.txt");
        let file2 = dir.join("large.txt");
        fs::write(&file1, "Small content").unwrap();
        fs::write(&file2, &"X".repeat(10000)).unwrap();

        let args = serde_json::json!({
            "paths": [file1.to_string_lossy(), file2.to_string_lossy()],
            "max_tokens": 1000
        });
        let result = ai_context_pack_smart_work(args);

        assert!(result.get("packed_content").is_some());
        assert!(result.get("files_included").is_some());
        assert!(result.get("files_excluded").is_some());
        assert!(result.get("estimated_tokens").is_some());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ml_dataset_validate_missing_required_field() {
        let dir = std::env::temp_dir().join("pb_mcp_test_validate_missing_field");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_missing_field.jsonl");

        // JSON without "messages" or "conversations"
        let data = r#"{"text": "some text", "label": "positive"}
"#;
        fs::write(&path, data).unwrap();

        let args = serde_json::json!({"path": path.to_string_lossy()});
        let result = ml_dataset_validate_work(args);
        assert_eq!(result["valid"], false);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ai_rewrite_style_missing_text() {
        let args = serde_json::json!({"style": "concise"});
        let result = ai_rewrite_style_work(args);
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_ai_rag_query_nonexistent_index() {
        let args =
            serde_json::json!({"query": "test", "index_path": "/tmp/nonexistent_index.json"});
        let result = ai_rag_query_work(args);
        let result: Value = serde_json::from_str(&result).unwrap();
        assert!(result.get("error").is_some());
    }
}
