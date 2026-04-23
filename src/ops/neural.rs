//! # Neural Network & AI Inference Helper Tools
//!
//! ONNX Runtime inference, tokenization, embedding cosine similarity,
//! and tensor statistics. Heavy deps (`ort`, `ndarray`, `tokenizers`)
//! are behind the `ml-tools` feature flag; pure-math functions work
//! unconditionally.

use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Metadata about a neural model file.
#[derive(Debug, Clone)]
pub struct NeuralModelInfo {
    /// Model name (derived from filename when not embedded in the file).
    pub name: String,
    /// Format string (e.g. "ONNX", "SavedModel", "TFLite").
    pub format: String,
    /// Human-readable input tensor shapes (e.g. `["float32[1,3,224,224]"]`).
    pub input_shapes: Vec<String>,
    /// Human-readable output tensor shapes.
    pub output_shapes: Vec<String>,
    /// File size in bytes.
    pub file_size_bytes: u64,
}

/// Result of a single output tensor from ONNX inference.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Name of the output tensor.
    pub output_name: String,
    /// Shape of the output tensor.
    pub shape: Vec<usize>,
    /// Summary: first 10 values as formatted string, with "..." appended if more.
    pub values_summary: String,
    /// Wall-clock inference time in milliseconds.
    pub inference_time_ms: u64,
}

// ---------------------------------------------------------------------------
// Internal helpers (always available)
// ---------------------------------------------------------------------------

/// Cosine similarity between two f32 slices (pure math, no deps).
fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for (va, vb) in a.iter().zip(b.iter()) {
        dot += *va as f64 * *vb as f64;
        norm_a += (*va as f64) * (*va as f64);
        norm_b += (*vb as f64) * (*vb as f64);
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return None;
    }
    Some((dot / denom) as f32)
}

/// Build a values-summary string: first 10 formatted, then "..." if more.
fn summarize_values(values: &[f32]) -> String {
    const LIMIT: usize = 10;
    let display: Vec<String> = values
        .iter()
        .take(LIMIT)
        .map(|v| format!("{v:.6}"))
        .collect();
    if values.len() <= LIMIT {
        format!("[{}]", display.join(", "))
    } else {
        format!("[{}, ...]", display.join(", "))
    }
}

/// Parse a JSON array of numbers into f32 values (works without ndarray).
fn parse_json_f32_array(json: &str) -> Result<Vec<f32>, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON: {e}"))?;
    let arr = v
        .as_array()
        .ok_or_else(|| "JSON root must be an array".to_string())?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        match item {
            serde_json::Value::Number(n) => {
                out.push(
                    n.as_f64()
                        .ok_or_else(|| "number out of f32 range".to_string())?
                        as f32,
                );
            }
            _ => return Err(format!("non-numeric element in array: {item}")),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Public functions (all return String — project convention)
// ---------------------------------------------------------------------------

/// Read ONNX model metadata. When the `ort` feature is enabled, actually
/// load the model and extract input/output info; otherwise return basic
/// file-level info derived from the path and file size.
pub fn neural_model_info_work(model_path: &Path) -> String {
    let meta = match fs::metadata(model_path) {
        Ok(m) => m,
        Err(e) => return format!("Error: cannot read model path: {e}"),
    };

    if !meta.is_file() {
        return format!("Error: not a file: {}", model_path.display());
    }

    let file_size_bytes = meta.len();
    let name = model_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let ext = model_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let format_str = match ext.as_str() {
        "onnx" => "ONNX",
        "pb" => "SavedModel",
        "tflite" => "TFLite",
        "pt" | "bin" => "PyTorch",
        "safetensors" => "SafeTensors",
        _ => "unknown",
    };

    #[cfg(feature = "ort")]
    {
        use std::time::Instant;
        let t = Instant::now();
        match ort::session::Session::builder().and_then(|b| b.commit_from_file(model_path)) {
            Ok(session) => {
                let load_ms = t.elapsed().as_millis() as u64;
                let mut input_shapes = Vec::new();
                for info in session.inputs() {
                    let name_label = info.name.clone();
                    let dims: Vec<String> = info
                        .shape
                        .iter()
                        .map(|d| {
                            d.map(|v| v.to_string())
                                .unwrap_or_else(|| "?".to_string())
                        })
                        .collect();
                    input_shapes.push(format!("{}[{}]", name_label, dims.join(",")));
                }
                let mut output_shapes = Vec::new();
                for info in session.outputs() {
                    let name_label = info.name.clone();
                    let dims: Vec<String> = info
                        .shape
                        .iter()
                        .map(|d| {
                            d.map(|v| v.to_string())
                                .unwrap_or_else(|| "?".to_string())
                        })
                        .collect();
                    output_shapes.push(format!("{}[{}]", name_label, dims.join(",")));
                }
                let info = NeuralModelInfo {
                    name,
                    format: format_str.to_string(),
                    input_shapes,
                    output_shapes,
                    file_size_bytes,
                };
                format!(
                    "NEURAL_MODEL_INFO\nname: {}\nformat: {}\nfile_size_bytes: {}\nload_time_ms: {}\ninputs: {}\noutputs: {}\n",
                    info.name,
                    info.format,
                    info.file_size_bytes,
                    load_ms,
                    if info.input_shapes.is_empty() {
                        "(none)".to_string()
                    } else {
                        info.input_shapes.join("; ")
                    },
                    if info.output_shapes.is_empty() {
                        "(none)".to_string()
                    } else {
                        info.output_shapes.join("; ")
                    },
                )
            }
            Err(e) => format!(
                "NEURAL_MODEL_INFO (fallback — ONNX load failed: {e})\nname: {}\nformat: {}\nfile_size_bytes: {}\n",
                name, format_str, file_size_bytes
            ),
        }
    }

    #[cfg(not(feature = "ort"))]
    {
        let info = NeuralModelInfo {
            name,
            format: format_str.to_string(),
            input_shapes: vec!["(enable ml-tools feature for details)".to_string()],
            output_shapes: vec!["(enable ml-tools feature for details)".to_string()],
            file_size_bytes,
        };
        format!(
            "NEURAL_MODEL_INFO\nname: {}\nformat: {}\nfile_size_bytes: {}\ninputs: {}\noutputs: {}\n",
            info.name,
            info.format,
            info.file_size_bytes,
            info.input_shapes.join("; "),
            info.output_shapes.join("; "),
        )
    }
}

/// Run ONNX Runtime inference. `input_json` maps input names to flat arrays.
/// Returns an error message when the `ml-tools` feature is not enabled.
pub fn neural_inference_work(model_path: &Path, input_json: &str) -> String {
    #[cfg(feature = "ort")]
    {
        use std::time::Instant;

        // Parse the input map: { "input_name": [1.0, 2.0, ...], ... }
        let inputs_map: std::collections::HashMap<String, Vec<f32>> =
            match serde_json::from_str(input_json) {
                Ok(m) => m,
                Err(e) => return format!("Error: invalid input JSON: {e}"),
            };

        let session =
            match ort::session::Session::builder().and_then(|b| b.commit_from_file(model_path)) {
                Ok(s) => s,
                Err(e) => return format!("Error: loading ONNX model: {e}"),
            };

        let input_infos = session.inputs();
        if inputs_map.len() != input_infos.len() {
            return format!(
                "Error: model expects {} inputs, got {} in JSON",
                input_infos.len(),
                inputs_map.len()
            );
        }

        // Build ort input values
        let mut ort_inputs: Vec<ort::session::Input> = Vec::new();
        for info in &input_infos {
            let data = match inputs_map.get(&info.name) {
                Some(d) => d,
                None => return format!("Error: missing input '{}'", info.name),
            };

            let dims: Vec<usize> = info
                .shape
                .iter()
                .map(|d| d.map(|v| v as usize).unwrap_or(1))
                .collect();

            // Total elements expected from shape
            let expected: usize = if dims.is_empty() {
                1
            } else {
                dims.iter().product()
            };
            if data.len() != expected {
                return format!(
                    "Error: input '{}' shape {:?} expects {} elements, got {}",
                    info.name,
                    dims,
                    expected,
                    data.len()
                );
            }

            let ndarray_input = ndarray::Array::from_shape_vec(ndarray::IxDyn(&dims), data.clone())
                .map_err(|e| format!("ndarray shape error: {e}"))
                .unwrap();

            ort_inputs.push(ort::session::Input::new(ndarray_input));
        }

        let t = Instant::now();
        let outputs = match session.run(ort_inputs.as_slice()) {
            Ok(o) => o,
            Err(e) => return format!("Error: ONNX inference failed: {e}"),
        };
        let inference_time_ms = t.elapsed().as_millis() as u64;

        let mut results = Vec::new();
        for (idx, output) in outputs.iter().enumerate() {
            let output_name = session
                .outputs()
                .get(idx)
                .map(|o| o.name.clone())
                .unwrap_or_else(|| format!("output_{idx}"));

            let ndarray_out: ndarray::ArrayD<f32> = match output.try_extract_tensor::<f32>() {
                Ok(arr) => arr,
                Err(_) => {
                    results.push(InferenceResult {
                        output_name,
                        shape: vec![],
                        values_summary: "(non-f32 output; enable ndarray for details)".to_string(),
                        inference_time_ms,
                    });
                    continue;
                }
            };

            let shape = ndarray_out.shape().to_vec();
            let values: Vec<f32> = ndarray_out.iter().cloned().collect();
            let values_summary = summarize_values(&values);

            results.push(InferenceResult {
                output_name,
                shape,
                values_summary,
                inference_time_ms,
            });
        }

        let mut out = format!("NEURAL_INFERENCE inference_time_ms={inference_time_ms}\n");
        for r in &results {
            out.push_str(&format!(
                "  output: {} shape={:?} values={}\n",
                r.output_name, r.shape, r.values_summary
            ));
        }
        out
    }

    #[cfg(not(feature = "ort"))]
    {
        let _ = (model_path, input_json);
        "Error: ONNX Runtime inference requires the `ml-tools` feature. \
         Rebuild with `--features ml-tools` to enable ort + ndarray + tokenizers."
            .to_string()
    }
}

/// Tokenize text using the `tokenizers` crate if available.
/// Returns token count, first 20 token IDs, and decoded token strings.
pub fn neural_tokenize_work(text: &str, model_name: &str) -> String {
    #[cfg(feature = "tokenizers")]
    {
        use tokenizers::Tokenizer;

        let model_name = model_name.trim();
        if model_name.is_empty() {
            return "Error: model_name must not be empty".to_string();
        }

        // Try loading from HuggingFace cache path or direct path
        let tokenizer = match Tokenizer::from_file(model_name).or_else(|_| {
            // Try HuggingFace cache convention:
            // ~/.cache/huggingface/hub/models--<model>/snapshots/<hash>/tokenizer.json
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let cache_path = std::path::PathBuf::from(home)
                .join(".cache")
                .join("huggingface")
                .join("hub")
                .join(format!("models--{}", model_name.replace('/', "--")))
                .join("snapshots");
            // Find the first snapshot directory
            if let Ok(entries) = std::fs::read_dir(&cache_path) {
                for entry in entries.flatten() {
                    let tok_path = entry.path().join("tokenizer.json");
                    if tok_path.exists() {
                        if let Ok(t) = Tokenizer::from_file(&tok_path) {
                            return Ok(t);
                        }
                    }
                }
            }
            Err(tokenizers::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("tokenizer not found for model '{model_name}'"),
            )))
        }) {
            Ok(t) => t,
            Err(e) => return format!("Error: loading tokenizer for '{model_name}': {e}"),
        };

        let encoding = match tokenizer.encode(text, true) {
            Ok(enc) => enc,
            Err(e) => return format!("Error: tokenization failed: {e}"),
        };

        let ids: Vec<u32> = encoding.get_ids().to_vec();
        let tokens: Vec<String> = encoding
            .get_tokens()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let token_count = ids.len();

        const MAX_DISPLAY: usize = 20;
        let display_ids: Vec<String> = ids
            .iter()
            .take(MAX_DISPLAY)
            .map(|id| id.to_string())
            .collect();
        let display_tokens: Vec<String> = tokens.iter().take(MAX_DISPLAY).cloned().collect();

        let ids_suffix = if ids.len() > MAX_DISPLAY { "..." } else { "" };
        let tokens_suffix = if tokens.len() > MAX_DISPLAY {
            "..."
        } else {
            ""
        };

        format!(
            "NEURAL_TOKENIZE\nmodel: {}\ntoken_count: {}\ntoken_ids: [{}]{}\ntokens: [{}]{}\n",
            model_name,
            token_count,
            display_ids.join(", "),
            ids_suffix,
            display_tokens
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(", "),
            tokens_suffix,
        )
    }

    #[cfg(not(feature = "tokenizers"))]
    {
        let _ = (text, model_name);
        "Error: tokenization requires the `ml-tools` feature. \
         Rebuild with `--features ml-tools` to enable the tokenizers crate."
            .to_string()
    }
}

/// Compute cosine similarity between two embedding vectors (pure math, no deps).
pub fn neural_embed_cosine_work(vec_a: &[f32], vec_b: &[f32]) -> String {
    if vec_a.len() != vec_b.len() {
        return format!(
            "Error: vector length mismatch (a={}, b={})",
            vec_a.len(),
            vec_b.len()
        );
    }
    if vec_a.is_empty() {
        return "Error: empty vectors".to_string();
    }
    match cosine_similarity(vec_a, vec_b) {
        Some(sim) => format!(
            "NEURAL_EMBED_COSINE\nvector_len: {}\ncosine_similarity: {:.6}\n",
            vec_a.len(),
            sim
        ),
        None => "Error: zero-norm vector (cannot normalize)".to_string(),
    }
}

/// Parse a JSON array of numbers, compute mean, std, min, max, shape info.
/// No ML deps required — pure statistics on a flat array.
pub fn neural_tensor_stats_work(tensor_json: &str) -> String {
    let values = match parse_json_f32_array(tensor_json) {
        Ok(v) => v,
        Err(e) => return format!("Error: {e}"),
    };

    if values.is_empty() {
        return "NEURAL_TENSOR_STATS\nelements: 0\n(no data)".to_string();
    }

    let n = values.len();
    let sum: f64 = values.iter().map(|v| *v as f64).sum();
    let mean = sum / n as f64;

    let variance: f64 = values
        .iter()
        .map(|v| {
            let d = *v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n as f64;
    let std_dev = variance.sqrt();

    let mut min_val = values[0];
    let mut max_val = values[0];
    for &v in &values[1..] {
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }

    // Heuristic shape: if the total is divisible by common dims, suggest
    // but always report flat length as the canonical shape.
    let shape_info = format!("[{n}]");

    format!(
        "NEURAL_TENSOR_STATS\nelements: {}\nshape: {}\nmean: {:.6}\nstd: {:.6}\nmin: {:.6}\nmax: {:.6}\nrange: {:.6}\nfirst_10: {}\n",
        n,
        shape_info,
        mean,
        std_dev,
        min_val,
        max_val,
        (max_val - min_val) as f64,
        summarize_values(&values),
    )
}

// ---------------------------------------------------------------------------
// Tests (NOT feature-gated — pure math only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0_f32, 2.0, 3.0, 4.0, 5.0];
        let sim = cosine_similarity(&a, &a).unwrap();
        assert!(
            (sim - 1.0_f32).abs() < 1e-5,
            "cosine(a, a) should be ~1.0, got {sim}"
        );

        // Also test the public function
        let result = neural_embed_cosine_work(&a, &a);
        assert!(result.contains("NEURAL_EMBED_COSINE"), "{result}");
        assert!(result.contains("cosine_similarity: 1.000000"), "{result}");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        let sim = cosine_similarity(&a, &b).unwrap();
        assert!(
            sim.abs() < 1e-5,
            "cosine([1,0], [0,1]) should be ~0.0, got {sim}"
        );

        let result = neural_embed_cosine_work(&a, &b);
        assert!(result.contains("cosine_similarity: 0.000000"), "{result}");
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![-1.0_f32, 0.0];
        let sim = cosine_similarity(&a, &b).unwrap();
        assert!(
            (sim - (-1.0_f32)).abs() < 1e-5,
            "cosine([1,0], [-1,0]) should be ~-1.0, got {sim}"
        );

        let result = neural_embed_cosine_work(&a, &b);
        assert!(result.contains("cosine_similarity: -1.000000"), "{result}");
    }

    #[test]
    fn test_tensor_stats() {
        // Known array: [1.0, 2.0, 3.0, 4.0, 5.0]
        let json = "[1.0, 2.0, 3.0, 4.0, 5.0]";
        let result = neural_tensor_stats_work(json);

        assert!(result.contains("NEURAL_TENSOR_STATS"), "{result}");
        assert!(result.contains("elements: 5"), "{result}");
        assert!(result.contains("mean: 3.000000"), "{result}");
        assert!(result.contains("min: 1.000000"), "{result}");
        assert!(result.contains("max: 5.000000"), "{result}");

        // Std of [1,2,3,4,5] (population): sqrt(2.0) ≈ 1.414214
        assert!(result.contains("std: 1.414"), "{result}");
    }

    #[test]
    fn test_tensor_stats_empty() {
        let result = neural_tensor_stats_work("[]");
        assert!(result.contains("elements: 0"), "{result}");
    }

    #[test]
    fn test_tensor_stats_invalid_json() {
        let result = neural_tensor_stats_work("not json");
        assert!(result.starts_with("Error:"), "{result}");
    }

    #[test]
    fn test_cosine_similarity_length_mismatch() {
        let a = vec![1.0_f32, 2.0];
        let b = vec![1.0_f32];
        let result = neural_embed_cosine_work(&a, &b);
        assert!(result.contains("Error: vector length mismatch"), "{result}");
    }

    #[test]
    fn test_cosine_similarity_zero_norm() {
        let a = vec![0.0_f32, 0.0];
        let b = vec![1.0_f32, 0.0];
        let result = neural_embed_cosine_work(&a, &b);
        assert!(result.contains("Error: zero-norm"), "{result}");
    }

    #[test]
    fn test_neural_model_info_nonexistent() {
        let result = neural_model_info_work(Path::new("/nonexistent/model.onnx"));
        assert!(result.starts_with("Error:"), "{result}");
    }

    #[test]
    fn test_neural_inference_without_feature() {
        #[cfg(not(feature = "ort"))]
        {
            let result = neural_inference_work(Path::new("/tmp/test.onnx"), r#"{"input": [1.0]}"#);
            assert!(result.contains("ml-tools"), "{result}");
        }
    }

    #[test]
    fn test_neural_tokenize_without_feature() {
        #[cfg(not(feature = "tokenizers"))]
        {
            let result = neural_tokenize_work("hello world", "gpt2");
            assert!(result.contains("ml-tools"), "{result}");
        }
    }

    #[test]
    fn test_summarize_values_short() {
        let vals = vec![1.0_f32, 2.0, 3.0];
        let s = summarize_values(&vals);
        assert!(!s.contains("..."), "{s}");
        assert!(s.contains("1.000000"), "{s}");
    }

    #[test]
    fn test_summarize_values_long() {
        let vals: Vec<f32> = (0..15).map(|v| v as f32).collect();
        let s = summarize_values(&vals);
        assert!(s.contains("..."), "{s}");
    }
}
