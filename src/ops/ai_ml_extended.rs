//! # Extended AI/ML/DL Tools
//!
//! Multi-model comparison, chain-of-thought, self-critique, few-shot/zero-shot
//! classification, structured extraction, debate, factual verification,
//! style rewriting, translation, embedding/semantic-search, RAG indexing,
//! context packing, model info, tokenization, benchmarking, perplexity,
//! fine-tune prep, and dataset validation.

use serde_json::Value;
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

    let critique_prompt = format!(
        "Review the following AI response and identify any errors, inconsistencies, \
         or areas for improvement. Be specific and constructive.\n\n\
         Original question: {original_prompt}\n\n\
         AI response to critique:\n{original_response}\n\n\
         Critique:"
    );

    match ollama_generate_api(&model, &critique_prompt, &ollama_url) {
        Ok((val, _latency_ms)) => {
            let critique = extract_generate_response(&val);

            // Now generate an improved response
            let improve_prompt = format!(
                "Based on this critique, provide an improved response to the original question.\n\n\
                 Original question: {original_prompt}\n\n\
                 Original response:\n{original_response}\n\n\
                 Critique:\n{critique}\n\n\
                 Improved response:"
            );

            let improved = match ollama_generate_api(&model, &improve_prompt, &ollama_url) {
                Ok((v2, _)) => extract_generate_response(&v2),
                Err(_) => String::new(),
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

/// Debate two positions on a topic using two model calls.
///
/// Args: `topic` (string), `model_a` (string), `model_b` (string),
/// `rounds` (int, default 2), `ollama_url` (string).
#[allow(dead_code)]
pub fn ai_debate_work(params: Value) -> String {
    let topic = match params.get("topic").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return serde_json::json!({"error": "Missing required arg: topic"}).to_string(),
    };
    let model_a = arg_str(&params, "model_a", "llama3.2");
    let model_b = arg_str(&params, "model_b", "llama3.2");
    let rounds = arg_u64(&params, "rounds", 2).clamp(1, 5) as usize;
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    let mut debate_log: Vec<Value> = Vec::new();

    for round in 1..=rounds {
        // Model A argues FOR
        let prompt_a = format!(
            "You are debating the topic: '{topic}'.\n\
             Round {round}: Argue strongly IN FAVOR of this topic. \
             Be concise (2-3 sentences)."
        );
        let arg_a = match ollama_generate_api(&model_a, &prompt_a, &ollama_url) {
            Ok((v, _)) => extract_generate_response(&v),
            Err(e) => format!("[Error: {e}]"),
        };
        debate_log.push(serde_json::json!({
            "round": round,
            "side": "for",
            "model": model_a,
            "argument": truncate_response(&arg_a)
        }));

        // Model B argues AGAINST
        let prompt_b = format!(
            "You are debating the topic: '{topic}'.\n\
             Round {round}: Argue strongly AGAINST this topic. \
             Be concise (2-3 sentences)."
        );
        let arg_b = match ollama_generate_api(&model_b, &prompt_b, &ollama_url) {
            Ok((v, _)) => extract_generate_response(&v),
            Err(e) => format!("[Error: {e}]"),
        };
        debate_log.push(serde_json::json!({
            "round": round,
            "side": "against",
            "model": model_b,
            "argument": truncate_response(&arg_b)
        }));
    }

    // Synthesis
    let synthesis_prompt = format!(
        "Summarize the key points from both sides of this debate on '{topic}' \
         and provide a balanced synthesis in 2-3 sentences."
    );
    let synthesis = match ollama_generate_api(&model_a, &synthesis_prompt, &ollama_url) {
        Ok((v, _)) => extract_generate_response(&v),
        Err(_) => String::new(),
    };

    serde_json::json!({
        "topic": topic,
        "debate_log": debate_log,
        "synthesis": truncate_response(&synthesis)
    })
    .to_string()
}

/// Convert a local model to GGUF format using llama.cpp tools.
///
/// Args: `input_path` (string), `output_path` (string, optional),
/// `quantization` (string, default "q4_k_m"), `python_path` (string, default "python3").
#[allow(dead_code)]
pub fn ml_convert_to_gguf_work(params: Value) -> String {
    let input_path = match params.get("input_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return serde_json::json!({"error": "Missing required arg: input_path"}).to_string()
        }
    };
    let quantization = arg_str(&params, "quantization", "q4_k_m");
    let python_path = arg_str(&params, "python_path", "python3");

    let input = Path::new(&input_path);
    if !input.exists() {
        return serde_json::json!({"error": format!("input_path does not exist: {input_path}")}).to_string();
    }

    let output_path = arg_str(&params, "output_path", "");
    let output_path = if output_path.is_empty() {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model");
        format!("{stem}-{quantization}.gguf")
    } else {
        output_path
    };

    // Try llama-cpp-convert script
    let _convert_script = "llama-cpp-convert";

    // Check for convert.py in common llama.cpp locations
    let convert_candidates = [
        "./convert.py",
        "./llama.cpp/convert.py",
        "/usr/local/lib/llama.cpp/convert.py",
    ];

    let convert_py = convert_candidates
        .iter()
        .find(|&&p| std::path::Path::new(p).exists())
        .copied();

    let result = if let Some(script) = convert_py {
        Command::new(&python_path)
            .args([script, &input_path, "--outfile", &output_path])
            .output()
    } else {
        return serde_json::json!({
            "error": "convert.py not found. Clone llama.cpp and place convert.py in the working directory.",
            "tool_available": false
        }).to_string();
    };

    match result {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let exit_code = o.status.code().unwrap_or(-1);
            let success = o.status.success();

            // Check if output file was created
            let output_exists = std::path::Path::new(&output_path).exists();

            serde_json::json!({
                "success": success && output_exists,
                "output_path": output_path,
                "exit_code": exit_code,
                "stdout": &stdout[..stdout.len().min(1000)],
                "stderr": &stderr[..stderr.len().min(1000)],
                "tool_available": true
            }).to_string()
        }
        Err(e) => serde_json::json!({
            "error": format!("Failed to run convert.py: {e}"),
            "tool_available": false
        }).to_string(),
    }
}

/// Run a multi-turn benchmark comparing several models on a fixed prompt set.
///
/// Args: `models` (array of strings), `prompts` (array of strings),
/// `ollama_url` (string, default "http://localhost:11434").
#[allow(dead_code)]
pub fn ml_model_compare_benchmark_work(params: Value) -> String {
    let models = arg_str_array(&params, "models", &["llama3.2"]);
    let prompts = arg_str_array(&params, "prompts", &["Explain quantum computing in one sentence."]);
    let ollama_url = arg_str(&params, "ollama_url", "http://localhost:11434");

    if models.is_empty() {
        return serde_json::json!({"error": "models array is empty"}).to_string();
    }
    if prompts.is_empty() {
        return serde_json::json!({"error": "prompts array is empty"}).to_string();
    }

    let mut model_results: Vec<Value> = Vec::new();

    for model in &models {
        let mut total_latency_ms: u64 = 0;
        let mut total_tokens: usize = 0;
        let mut prompt_results: Vec<Value> = Vec::new();
        let mut errors = 0usize;

        for prompt in &prompts {
            match ollama_generate_api(model, prompt, &ollama_url) {
                Ok((val, latency_ms)) => {
                    let response = extract_generate_response(&val);
                    let tokens = estimate_tokens(&response);
                    total_latency_ms += latency_ms as u64;
                    total_tokens += tokens;
                    prompt_results.push(serde_json::json!({
                        "prompt": &prompt[..prompt.len().min(80)],
                        "response_snippet": &response[..response.len().min(200)],
                        "latency_ms": latency_ms,
                        "estimated_tokens": tokens
                    }));
                }
                Err(e) => {
                    errors += 1;
                    prompt_results.push(serde_json::json!({
                        "prompt": &prompt[..prompt.len().min(80)],
                        "error": e
                    }));
                }
            }
        }

        let avg_latency = if !prompt_results.is_empty() && errors < prompt_results.len() {
            total_latency_ms as f64 / (prompt_results.len() - errors) as f64
        } else {
            0.0
        };

        model_results.push(serde_json::json!({
            "model": model,
            "avg_latency_ms": avg_latency.round(),
            "total_tokens": total_tokens,
            "errors": errors,
            "prompt_results": prompt_results
        }));
    }

    serde_json::json!({
        "models_compared": models.len(),
        "prompts_used": prompts.len(),
        "results": model_results
    })
    .to_string()
}
