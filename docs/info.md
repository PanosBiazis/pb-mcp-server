# pb-mcp-server — Project Overview

> A Rust-native MCP (Model Context Protocol) server that exposes **500+ tools**
> to AI agents running inside Zed, Cursor, VSCode, and other MCP-capable editors.
> Designed for multi-agent orchestration, local-first AI workflows, and
> comprehensive developer tooling — all from a single binary.

---

## Table of Contents

1. [Architecture](#architecture)
2. [Workspace Members](#workspace-members)
3. [Build & Test Status](#build--test-status)
4. [Tool Catalogue (28 Modules, 500+ Tools)](#tool-catalogue-28-modules-500-tools)
5. [Feature Flags](#feature-flags)
6. [Security Model](#security-model)
7. [Tool Expansion History](#tool-expansion-history)
8. [Development Workflow](#development-workflow)

---

## Architecture

```
Zed / Cursor / VSCode
    │  (MCP protocol over stdio)
    ▼
┌──────────────────────────────────────┐
│   pb-mcp-server  (Rust binary)       │
│                                      │
│  ┌────────────────────────────────┐  │
│  │  Agent Broker (broadcast bus)  │  │
│  └──┬────────┬────────┬──────────┘  │
│     │        │        │              │
│  Agent A  Agent B  Agent C  …        │
│  (Claude) (GPT-4) (Ollama)           │
│                                      │
│  ┌────────────────────────────────┐  │
│  │  tools layer + 28 ops modules  │  │
│  └────────────────────────────────┘  │
│                                      │
│  pb-gateway ──── auth / routing      │
│  pb-lsp-bridge ─ language servers    │
│  pb-network-node ─ libp2p P2P       │
│  pb-dashboard ── admin UI            │
└──────────────────────────────────────┘
```

The MCP server runs as a long-lived process. Editors connect via **stdio**
(or SSE, depending on configuration). Each incoming tool call is dispatched to
one of 28 operational modules in `src/ops/`, which contain the actual work
functions. The Agent Broker enables multi-agent communication: agents can
broadcast messages, suggest improvements to each other, vote via consensus
protocols, and coordinate on complex tasks.

---

## Workspace Members

The Cargo workspace contains **5 crates**:

| Crate | Path | Purpose |
|---|---|---|
| **pb-mcp-server** | `/` (root) | Main MCP server binary — hosts all 500+ tools |
| **pb-gateway** | `crates/pb-gateway/` | HTTP gateway with token auth, rate limiting, route proxying |
| **pb-lsp-bridge** | `crates/pb-lsp-bridge/` | Bridges language-server protocols into the MCP tool surface |
| **pb-network-node** | `crates/pb-network-node/` | libp2p swarm node — Kademlia DHT, Gossipsub, peer discovery |
| **pb-dashboard** | `crates/pb-dashboard/` | Admin dashboard — system status, tool metrics, agent monitoring |

---

## Build & Test Status

| Metric | Value |
|---|---|
| **Exposed MCP Tools** | **500+** |
| **Operational Modules** | **28** (`src/ops/`) |
| **Unit Tests Passing** | **1,220+** |
| **Compiler Errors** | **0** |
| **Code Warnings** | **0** (only filesystem hard-link warnings on external HD) |
| **Build** | Clean — `cargo build` and `cargo test` pass without issues |

---

## Tool Catalogue (28 Modules, 500+ Tools)

The current tool surface is split between the MCP wiring layer and the work
modules:

- `src/tools/*_args.rs` defines typed arguments for newer extracted handlers.
- `src/tools/*_handlers.rs` contains extracted handler glue for expanded tool
  families.
- `src/tools/catalog.rs` and `src/tools/registry.rs` expose metadata,
  stub markers, and profile gates.
- `src/tool_args_wired.rs` and `src/main.rs` still host older wired argument
  structs and handlers.
- `src/ops/*.rs` contains the underlying `*_work` functions for the 28 ops
  modules.

### 1. `ai_ops.rs` — Ollama AI Generation & Embedding

Local LLM interaction via the Ollama HTTP API: text generation, chat
completion, embedding vectors, prompt templating, context packing, and
semantic diff.

### 2. `ai_ml_extended.rs` — AI/ML Classification, Reasoning & Model Management (22 tools)

Advanced AI/ML workflows built on top of Ollama:

| Tool | Description |
|---|---|
| `ai_few_shot_classify` | Few-shot text classification with labeled examples |
| `ai_zero_shot_classify` | Zero-shot classification into candidate labels |
| `ai_structured_extract` | Extract typed JSON from text using a schema |
| `ai_verify_factual` | Verify a claim against reference documents |
| `ai_rewrite_style` | Rewrite text in technical / casual / academic / eli5 / concise style |
| `ai_embed_text` | Get an embedding vector via Ollama (nomic-embed-text, etc.) |
| `ai_semantic_search` | Cosine-similarity search over document arrays |
| `ai_cluster_texts` | k-means clustering on embeddings |
| `ai_context_pack_smart` | Pack files into a token-budgeted context window |
| `ml_model_info` | Metadata for ONNX / PyTorch / SafeTensors model files |
| `ml_tokenize` | Tokenize text via Ollama tokenize API |
| `ml_token_count` | Count tokens (Ollama API with char/4 fallback) |
| `ml_benchmark_model` | Measure tokens/sec and latency over N iterations |
| `ml_perplexity_score` | Approximate perplexity via naturalness rating |
| `ml_fine_tune_prep` | Convert conversations to OpenAI / Llama / Mistral JSONL |
| `ml_dataset_validate` | Validate JSONL datasets for fine-tuning quality |
| `ai_multi_model_compare` | Run same prompt across multiple Ollama models |
| `ai_chain_of_thought` | Multi-step Chain-of-Thought reasoning (N steps + conclusion) |
| `ai_self_critique` | Critique + improve an AI response |
| `ai_debate` | Two model instances debate a topic N rounds + synthesis |
| `ml_convert_to_gguf` | Convert model to GGUF via llama.cpp `convert.py` |
| `ml_model_compare_benchmark` | Benchmark multiple models on a prompt set |

### 3. `algorithms.rs` — Sorting, Graph Algorithms, Statistics, FFT

General-purpose algorithm tools: sorting (merge, quick, heap), graph
traversal (BFS, DFS, Dijkstra, A*), statistical analysis (mean, median,
std-dev, percentiles), and Fast Fourier Transform.

### 4. `audio.rs` — Audio Processing (ffmpeg-based)

Audio file manipulation via ffmpeg: format conversion, trimming, volume
adjustment, mixing, waveform analysis, and metadata extraction.

### 5. `code_intel.rs` — AST Dump, Imports, Language Detection, Complexity

Static code analysis tools: AST parsing and dumping, import/dependency
extraction, programming language detection, cyclomatic complexity
calculation, and code structure analysis.

### 6. `convert_ext.rs` — Archive, Binary, Document & QR Conversion

File format conversions: archive creation/extraction (zip, tar, gz),
binary encoding (base64, hex), document conversion, and QR code
generation/reading.

### 7. `data_analytics.rs` — CSV, JSON, Timeseries & Parquet Analysis

Data analysis tools: CSV parsing and querying, JSON structure analysis,
timeseries aggregation, Parquet file reading, statistical summaries,
and data transformation pipelines.

### 8. `db.rs` — SQLite Query / Exec / Schema

Database tools: execute SQL queries and statements against SQLite
databases, inspect schema (tables, columns, indexes), and export
query results.

### 9. `debug.rs` — Trace Replay, Failure Clustering, Root Cause Analysis

Debugging assistance: replay execution traces, cluster failures by
pattern, perform root-cause analysis, and generate debugging reports.

### 10. `file_crypto.rs` — File Encrypt / Decrypt, HMAC Sign / Verify

Cryptographic file operations: AES encryption/decryption, HMAC-SHA256
signing and verification, file integrity checking, and secure file
deletion.

### 11. `ide_tools.rs` — Zed, Cursor, VSCode & JetBrains Integration

IDE-specific tools: open files in editors, navigate to symbols, manage
editor settings, synchronize state across IDEs, and extract editor
context.

### 12. `image.rs` — Image Processing & Manipulation

Image tools: resize, crop, rotate, apply filters, format conversion
(PNG, JPEG, WebP, etc.), thumbnail generation, and metadata extraction.

### 13. `iot.rs` — IoT Device Management, Serial, GPIO & CAN Bus

IoT and hardware tools: serial port communication, GPIO pin control,
CAN bus message send/receive, device discovery, and sensor data reading.

### 14. `logs_charts.rs` — Log Analysis & SVG Chart Generation (22 tools)

**Log analysis tools:**

| Tool | Description |
|---|---|
| `log_tail` | Last N lines with optional level filter |
| `log_search` | Regex grep over log file |
| `log_parse_json` | Parse JSONL logs with optional field filter |
| `log_parse_nginx` | Parse Nginx access logs |
| `log_parse_syslog` | Parse syslog format logs |
| `log_level_summary` | Count DEBUG / INFO / WARN / ERROR |
| `log_error_cluster` | Group errors by normalized pattern |
| `log_time_range_filter` | Extract lines within ISO timestamp range |
| `log_export` | Export entries to JSON / CSV / Markdown file |
| `log_audit_query` | Query `.mint-mcp` audit log |

**History & replay tools:**

| Tool | Description |
|---|---|
| `history_tool_calls` | Recent tool calls from audit log |
| `history_bus_replay` | Replay messages from a JSONL bus file |
| `history_diff_timeline` | File change history over N git commits |
| `history_search_past_calls` | Full-text search across audit log entries |

**SVG chart generation tools:**

| Tool | Description |
|---|---|
| `chart_line` | SVG line chart (multi-series) |
| `chart_bar` | SVG bar chart |
| `chart_pie` | SVG pie / donut chart |
| `chart_histogram` | SVG histogram with bin edges + counts |
| `chart_scatter` | SVG scatter plot |
| `chart_timeseries` | SVG time-series with moving-average overlay |
| `chart_dependency_graph` | Graphviz DOT dependency graph |
| `chart_gantt` | SVG Gantt chart |

### 15. `network_tools.rs` — Ping, Traceroute, Port Scan, SSH, MQTT & Bluetooth

Network diagnostics and communication: ICMP ping, traceroute, TCP port
scanning, SSH command execution, MQTT publish/subscribe, and Bluetooth
device discovery.

### 16. `neural.rs` — ONNX Runtime Inference, Tokenization & Embeddings (5 tools)

Neural network tools (behind `ml-tools` feature flag):

| Tool | Description |
|---|---|
| `neural_model_info` | ONNX / PT / SafeTensors metadata + I/O shapes |
| `neural_inference` | ONNX Runtime inference (requires `ml-tools`) |
| `neural_tokenize` | HuggingFace tokenizer (requires `ml-tools`) |
| `neural_embed_cosine` | Cosine similarity between two float vectors (pure Rust) |
| `neural_tensor_stats` | Mean / std / min / max / percentiles for flat tensor arrays |

### 17. `notify.rs` — Desktop, Email & Webhook Notifications

Notification tools: desktop notifications (libnotify), email sending
(SMTP), webhook dispatching (HTTP POST), and notification history.

### 18. `ocr_tools.rs` — OCR (Tesseract-based)

Optical character recognition: extract text from images using Tesseract,
with support for multiple languages, confidence scoring, and region
selection.

### 19. `protection.rs` — Security Scanning, YARA, Integrity & Secret Detection

Security tools: YARA rule matching, file integrity monitoring, secret
detection (API keys, tokens, private keys — 10 patterns), vulnerability
scanning, and security report generation.

### 20. `research.rs` — arXiv Search & Quantum Simulation

Research tools: search and retrieve papers from arXiv, and quantum
circuit simulation.

### 21. `reverse_eng.rs` — Binary Analysis, Disassembly & Entropy Analysis

Reverse engineering tools: binary file analysis, disassembly (x86/ARM),
entropy analysis for packed/encrypted section detection, string
extraction, and symbol table inspection.

### 22. `review.rs` — PR Review, Diff Risk Assessment & Missing Test Detection

Code review tools: automated pull request review, diff risk scoring,
missing test detection, code quality assessment, and review comment
generation.

### 23. `sandbox.rs` — Sandboxed Execution & Docker Integration

Execution isolation: run commands in sandboxed environments, Docker
container management, resource limits, and output capture.

### 24. `screenshot_tools.rs` — Screen Capture, OCR & Timelapse

Screenshot tools: full-screen and region capture, OCR on screenshots,
timelapse recording, and screenshot annotation.

### 25. `speech.rs` — Speech-to-Text, TTS & Diarization

Speech processing: speech-to-text transcription, text-to-speech
synthesis, speaker diarization, and audio language detection.

### 26. `system.rs` — Process Management, Disk, Docker & K8s

System administration tools: process listing and management, disk usage
analysis, Docker container operations, Kubernetes cluster interaction,
and system resource monitoring.

### 27. `web_ext.rs` — HTTP API Calls, RSS & Web Search

Web interaction tools: HTTP request execution (GET/POST/PUT/DELETE),
RSS feed parsing, web search, HTML scraping, and URL analysis.

### 28. *(Multi-Agent Consensus — `src/agent/consensus.rs`)*

Not in `src/ops/` but exposed as MCP tools: `consensus_propose`,
`consensus_vote`, `consensus_tally`, `consensus_list`, `consensus_close`.
Six strategies: Simple Majority, Super Majority, Unanimous, Weighted
Voting, Ranked Choice, and Delegation.

---

## Feature Flags

The project uses Cargo feature flags to control which capabilities are
compiled in:

| Feature | Includes | Use Case |
|---|---|---|
| **`default`** | `core-code` + `agent-orchestration` + `review-refactor` | Basic MCP server with code tools and agent coordination |
| **`local-power`** | `tesseract` + `image` + `web-tools` | Add OCR, image processing, and web capabilities |
| **`experimental-lab`** | `local-power` + `ort` + `ndarray` + `tokenizers` + `web-tools` | Full ML/neural pipeline with ONNX Runtime |
| **`full`** | All features including dashboard | Everything compiled in |
| **`ml-tools`** | `ort` + `ndarray` + `tokenizers` | ONNX inference, tensor ops, HuggingFace tokenizers |
| **`web-tools`** | `reqwest` + `scraper` + `url` + `mime` | HTTP client, HTML parsing, URL handling |
| **`network-node`** | `libp2p` | P2P networking via pb-network-node |

### Building with Features

```
# Default build (core tools + agent orchestration)
cargo build --release

# Full build with all features
cargo build --release --features full

# ML/neural tools only
cargo build --release --features ml-tools

# Local power-user setup (OCR + image + web)
cargo build --release --features local-power
```

---

## Security Model

Security is enforced at multiple layers throughout the codebase:

### Compile-Time Safety

- **`#![deny(unsafe_code)]`** applied globally — the entire codebase is
  safe Rust with a single `#[allow(unsafe_code)]` exception for
  memory-mapped file operations (mmap).

### Runtime Protections

| Layer | Mechanism |
|---|---|
| **Path sandboxing** | All file operations are confined to allowed directories with symlink depth limits to prevent traversal attacks |
| **Command allowlists** | Shell command execution is restricted to an explicit allowlist; shell metacharacters (`; | & $ \` >`) are blocked |
| **Rate limiting** | Bus operations are rate-limited to prevent flooding and denial-of-service |
| **HMAC-SHA256 signing** | Network messages between nodes are signed and verified |
| **Constant-time auth** | Gateway token comparison uses constant-time equality to prevent timing attacks |
| **Network profile gating** | Tools are gated by network profile (local vs. public) to prevent accidental exposure of sensitive operations |
| **Secret detection** | Built-in scanner with 10 patterns (AWS keys, GitHub tokens, JWTs, private keys, etc.) |
| **Integrity checking** | SHA-256 file integrity verification |

---

## Tool Expansion History

The tool count has grown across three major phases:

| Version / Phase | Tool Count | Notes |
|---|---|---|
| **v0.1** (initial) | ~253 | First release with core tool set |
| **Phase 2** | 435 | +182 tools — new modules (neural, protection, debug, review, etc.) |
| **Phase 3** | 484+ | +49 tools — wired existing work functions (AI/ML Extended, Logs/Charts, Neural) |
| **Phase 4** (latest) | **501** | +17 tools — algorithm/math expansion: number theory, base conversion, frequency tables, set ops, percentiles, and histograms |

### Phase 3 Expansion Details (435 → 484+)

49 previously-implemented work functions were wired as live MCP tools
with **zero new work-function code** — only wiring in `main.rs` and
argument structs in `tool_args_wired.rs`:

- **AI/ML Extended:** 22 tools from `ai_ml_extended.rs`
- **Logs & Charts:** 22 tools from `logs_charts.rs`
- **Neural:** 5 tools from `neural.rs`

### Phase 4 Expansion Details (484+ → 501)

17 pure Rust algorithm/math tools were added as live MCP handlers with bounded
inputs and focused regression coverage:

- **Number theory:** GCD/LCM, prime factors, primality, prime sieve, Fibonacci
- **Conversion/counting:** base conversion, Roman numerals, combinations, permutations
- **Text/data math:** entropy, frequency table, run-length encoding, binary-search bounds, set ops, moving average, percentile, numeric histogram

**Files changed:**

| File | Change |
|---|---|
| `src/ops/mod.rs` | Added 5 missing re-exports from `ai_ml_extended` |
| `src/tool_args_wired.rs` | +49 new argument structs (+597 lines) |
| `src/main.rs` | +49 `#[tool(...)] async fn` handlers (+1,253 lines) |

---

## Development Workflow

### Prerequisites

- **Rust** (stable toolchain)
- **Ollama** (for AI/ML tools — runs locally)
- **ffmpeg** (for audio tools)
- **Tesseract** (for OCR tools, behind `local-power` feature)
- **Docker** (for sandbox and container tools, optional)

### Common Commands

```
# Build
cargo build --release

# Run all tests
cargo test

# Run with specific features
cargo test --features full

# Clippy lint check
cargo clippy --all-targets --all-features

# Format check
cargo fmt --check
```

### CI Pipeline

GitHub Actions CI (`.github/workflows/ci.yml`) runs 6 jobs:

1. **check** — `cargo check` across all features
2. **test** — `cargo test` with default features
3. **test-features** — `cargo test` with `--features full`
4. **clippy** — lint with `-D warnings`
5. **fmt** — format check
6. **audit** — `cargo audit` for known vulnerabilities

### Editor Integration

Add to Zed's `settings.json`:

```
{
  "context_servers": {
    "pb-mcp-server": {
      "command": {
        "path": "/path/to/target/release/pb-mcp-server",
        "args": []
      }
    }
  }
}
```

The server communicates via **stdio** using the MCP protocol. On startup
it registers all 500+ tools, which appear in the editor's AI tool panel.
