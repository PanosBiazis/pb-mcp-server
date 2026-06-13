# pb-mcp-server

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/PanosBiazis/pb-mcp-server/workflows/CI/badge.svg)](https://github.com/PanosBiazis/pb-mcp-server/actions)
[![MCP](https://img.shields.io/badge/Protocol-MCP-blue.svg)](https://modelcontextprotocol.io/)

> **Profile-based IDE agent MCP server for Zed/Cursor** with multi-agent bus, code intelligence, and execution tools.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Profiles](#profiles)
- [Tools Reference](#tools-reference)
- [Security Model](#security-model)
- [Configuration](#configuration)
- [Multi-Agent Bus](#multi-agent-bus)
- [HTTP Gateway](#http-gateway)
- [Development](#development)
- [Contributing](#contributing)

---

## Overview

`pb-mcp-server` is a Rust-based MCP (Model Context Protocol) server designed for IDE agents in Zed, Cursor, and similar editors. It exposes **500+ tools** across six profiles and is built as a Cargo workspace with 5 members covering the MCP core, HTTP gateway, LSP bridge, P2P networking, and an admin dashboard. It provides:

- **Profile-based tool exposure** - Different tool sets for different workflows
- **Multi-agent coordination** - Durable append-only message bus for agent communication
- **Code intelligence** - Symbol indexing, search, and analysis tools
- **Safe execution** - Sandboxed terminal operations with allowlists
- **Review tools** - PR review, architecture analysis, risk assessment
- **App/workflow equivalents** - Local deterministic helpers inspired by n8n,
  MCP builders, skills/plugins/commands, Codex, Claude Code, OpenCode,
  OpenClaw, Hermes, bash, PowerShell, batch/cmd, and macOS Terminal workflows

## Features

### 🎯 Core Capabilities

| Feature | Description |
|---------|-------------|
| **File Operations** | Read, list, search, tree, grep with project sandboxing |
| **Git Integration** | Status, diff, log, blame, branches, and more |
| **Code Indexing** | Symbol extraction, path indexing, dependency analysis |
| **Text Processing** | Compression, summarization, format conversion |
| **Agent Bus** | Multi-agent messaging, handoffs, consensus |
| **Execution** | Terminal commands, test runners, OCR (gated) |
| **App / Workflow Tools** | n8n-style workflow validation/templates, MCP manifest/spec helpers, skills/plugins/commands, shell command helpers, agent prompt/session/patch/context/task planning |

### 🛡️ Security Features

- **`#![deny(unsafe_code)]`** - Unsafe code forbidden at compile time (one gated exception for `perf-mmap`)
- **Path sandboxing** - All file operations confined to project root, symlink depth-limited
- **Command allowlists** - Only approved executables can run; shell metacharacters blocked
- **HMAC-SHA256 message signing** - All inter-node messages are signed; covers `id`, `from`, `to`, `kind`, `nonce`, `ts`, `body`
- **Replay protection** - Per-message nonces prevent exact-replay attacks
- **Constant-time auth** - Token and signature comparisons use constant-time routines
- **Rate limiting** - Bus operations are rate-limited per agent
- **Profile gating** - Dangerous tools require explicit feature flags and env vars
- **Input validation** - All inputs validated with length limits at entry points

### 📊 Profile System

Tools are organized into profiles for better discoverability and security:

```
┌─────────────────────────────────────────────────────────────┐
│                      Tool Profiles                          │
├─────────────────────────────────────────────────────────────┤
│ core-code        │ Default coding (read-only)               │
│ review-refactor  │ PR review + architecture analysis        │
│ agent-orchestration │ Multi-agent coordination            │
│ local-power      │ Local execution (requires approval)      │
│ public-safe      │ Remote HTTP subset (bounded, read-only)  │
│ experimental-lab │ Advanced ML/AI tools                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- Rust 1.75+ (use `rustup` for installation)
- A project directory to work with

### Build

```bash
# Clone the repository
git clone https://github.com/PanosBiazis/pb-mcp-server.git
cd pb-mcp-server

# Build release binary
cargo build --release

# The binary will be at:
# ./target/release/pb-mcp-server

# Linux full release binary
cargo build --release -p pb-mcp-server --all-features --bin pb-mcp-server

# Smart release matrix for available Linux/Windows/macOS targets
make release-matrix

# Windows x64 exact all-features release binary from Linux
rustup target add x86_64-pc-windows-gnu
cargo build --release -p pb-mcp-server --all-features --bin pb-mcp-server --target x86_64-pc-windows-gnu

# The Windows binary will be at:
# ./target/x86_64-pc-windows-gnu/release/pb-mcp-server.exe
```

### Run in Zed

1. Copy the example configuration into your Zed MCP settings:
   ```bash
   # Zed stores MCP server configs in its settings.json
   # Copy and fill in your paths
   cp setup.example.json setup.json
   # Edit setup.json: replace /path/to/pb-mcp-server and /path/to/your/project
   ```

2. In Zed, open **Settings → MCP Servers** and add the entry from `setup.json`, or merge it into `~/.config/zed/settings.json` under the `"context_servers"` key.

3. Restart Zed — the MCP server will connect automatically and expose all tools.

### Verify Installation

```bash
# Quick sanity check — the binary should print its version and exit
PB_PROJECT_ROOT=$(pwd) ./target/release/pb-mcp-server 2>&1 | head -5

# If you also start the gateway binary:
curl http://localhost:8080/health
```

---

## Workspace Architecture

The repository is a Cargo workspace with five members:

| Crate | Description |
|-------|-------------|
| `pb-mcp-server` | Core library + main MCP server binary |
| `crates/pb-gateway` | HTTP REST gateway with auth and project inspection endpoints |
| `crates/pb-lsp-bridge` | LSP↔MCP bridge for Android Studio and JetBrains IDEs |
| `crates/pb-network-node` | P2P networking node (libp2p Kademlia + Gossipsub) |
| `crates/pb-dashboard` | egui/eframe admin dashboard (`--features dashboard`) |

---

## Architecture

```
pb-mcp-server/
├── src/
│   ├── main.rs              # Entry point + all MCP tool handlers (~12k LOC)
│   ├── lib.rs               # Library root with module declarations & re-exports
│   ├── server.rs            # MCP server core implementation
│   ├── tool_args_wired.rs   # Strongly-typed arg structs (JsonSchema)
│   ├── mcp_args.rs          # Additional MCP argument types
│   ├── bus.rs               # Durable JSONL message bus
│   ├── bus_rate.rs          # Bus rate limiting
│   ├── code_index.rs        # Symbol indexing engine
│   ├── thinking.rs          # Planning/reasoning scaffolds
│   ├── profiles.rs          # Profile definitions
│   ├── network_profile.rs   # Network security profiles
│   ├── output.rs            # Output formatting utilities
│   ├── error.rs             # Error types
│   ├── health.rs            # Health monitoring
│   ├── metrics.rs           # Performance metrics
│   ├── agent/               # Multi-agent coordination
│   │   ├── mod.rs           # Agent registry, sync, consensus
│   │   └── live.rs          # Real-time streaming & P2P channels
│   ├── args/                # Argument structures by category
│   ├── ops/                 # Business logic (28 modules)
│   │   ├── ai_ops.rs        # Ollama AI generation & embedding
│   │   ├── ai_ml_extended.rs # 22 AI/ML classification & reasoning tools
│   │   ├── algorithms.rs    # Sorting, graph, stats, FFT
│   │   ├── audio.rs         # Audio processing (ffmpeg)
│   │   ├── code_intel.rs    # AST, imports, language detection
│   │   ├── convert_ext.rs   # Archive, binary, document conversion
│   │   ├── data_analytics.rs # CSV, JSON, timeseries analysis
│   │   ├── db.rs            # SQLite query/exec/schema
│   │   ├── debug.rs         # Trace, replay, failure clustering
│   │   ├── file_crypto.rs   # Encrypt, sign, verify (HMAC-SHA256)
│   │   ├── ide_tools.rs     # Zed, Cursor, VSCode, JetBrains integration
│   │   ├── image.rs         # Image processing & manipulation
│   │   ├── iot.rs           # IoT device, serial, GPIO, CAN bus
│   │   ├── logs_charts.rs   # 22 log analysis & SVG chart tools
│   │   ├── network_tools.rs # Ping, traceroute, port scan, SSH, MQTT
│   │   ├── neural.rs        # ONNX inference, tokenization, embeddings
│   │   ├── notify.rs        # Desktop, email, webhook notifications
│   │   ├── ocr_tools.rs     # OCR (Tesseract-based)
│   │   ├── protection.rs    # Security scanning, YARA, integrity
│   │   ├── research.rs      # arXiv search & quantum simulation
│   │   ├── reverse_eng.rs   # Binary analysis, disassembly, entropy
│   │   ├── review.rs        # PR review, diff risk, missing tests
│   │   ├── sandbox.rs       # Sandboxed execution & Docker
│   │   ├── screenshot_tools.rs # Screen capture, OCR, timelapse
│   │   ├── speech.rs        # Speech-to-text, TTS, diarization
│   │   ├── system.rs        # Process, disk, Docker, K8s management
│   │   └── web_ext.rs       # HTTP API, RSS, web search
│   ├── security/            # Security modules
│   │   ├── path_guard.rs    # Path traversal prevention
│   │   ├── command_policy.rs # Command execution policy
│   │   ├── rate_limiter.rs  # Rate limiting
│   │   ├── scanner.rs       # Secret & vulnerability scanning
│   │   └── audit.rs         # Audit logging
│   ├── tools/               # Tool handler modules by category
│   ├── cache/               # File caching layer
│   ├── gateway/             # HTTP gateway (auth, routes, state)
│   ├── network/             # P2P networking (libp2p)
│   ├── resilience/          # Circuit breaker, retry, bulkhead
│   └── web/                 # Web utilities
├── crates/                  # Workspace members
│   ├── pb-gateway/          # HTTP REST gateway binary
│   ├── pb-lsp-bridge/       # LSP↔MCP bridge
│   ├── pb-network-node/     # P2P networking node
│   └── pb-dashboard/        # egui admin dashboard
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
├── docs/                    # Documentation
└── deploy/                  # Deployment configs
```

### Module Dependencies

```
┌─────────────┐
│   main.rs   │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌──────────┐
│   server/   │────▶│ profiles │
└──────┬──────┘     └──────────┘
       │
       ▼
┌─────────────┐     ┌──────────┐     ┌──────────┐
│    tools/   │────▶│   ops/   │────▶│ security │
└─────────────┘     └──────────┘     └──────────┘
       │                                    │
       ▼                                    ▼
┌─────────────┐                     ┌──────────┐
│    args/    │                     │  error   │
└─────────────┘                     └──────────┘
```

---

## Profiles

### core-code (Default)

**Purpose:** Default coding workflow in Zed/Cursor

**Tools:**
- File: `read_file`, `read_file_tail`, `list_files`, `project_tree`, `find_files`, `file_info`, `file_stats`
- Git: `git_status`, `git_diff`, `git_log`, `git_show`, `git_branches`
- Code: `code_index_build`, `code_index_search`, `codebase_index`, `codebase_path_list`
- Text: `compress_context`, `pack_context`, `summarize_text`, `text_stats`
- Context: `workspace_context`, `grep_code`

**Risk Level:** Low (read-only)

### review-refactor

**Purpose:** PR review, architecture understanding, safe codebase inspection

**Additional Tools:**
- Git: `git_blame`, `git_tag_list`, `git_diff_snippets`, `git_reflog`
- Code: `codebase_largest_files`, `codebase_duplicate_basenames`, `code_index_duplicate_symbols`
- Review: `review_diff_risk`, `review_focused_review`, `review_missing_tests`, `review_pr_packet`
- Reasoning: `reasoning_evidence_matrix`, `plan_risk_register`, `plan_definition_of_done`

**Risk Level:** Low (read-only)

### agent-orchestration

**Purpose:** Multi-agent and subagent coordination

**Tools:**
- Bus: `agent_send`, `agent_inbox`, `agent_thread`, `agent_recent`, `agent_bus_status`
- Coordination: `ask_agents`, `agent_handoff`, `consensus_propose`, `consensus_vote`, `consensus_tally`
- Fan-out: `agent_fan_out`, `agent_send_pipeline`, `agent_multi_inbox`
- Temp: `workspace_temp_write`, `workspace_temp_read`, `workspace_temp_list`, `workspace_temp_delete`
- Wait: `server_sleep`, `wait_for_path`, `wait_until_timestamp`, `agent_inbox_wait`

**Risk Level:** Medium

### local-power

**Purpose:** Explicitly approved local execution and validation

**Tools:**
- Terminal: `terminal_run`, `terminal_run_stdin`, `terminal_which`, `terminal_program_version`
- Tests: `project_cargo_test`, `project_cargo_check`, `project_pytest`
- Git write: `git_add_or_commit`, `git_push`, `git_checkout`, `git_fetch`, `git_pull`
- Audio/Image: `audio_info`, `audio_convert`, `image_info`, `image_resize`, `image_convert`
- Network: `net_ping`, `net_port_scan`, `net_tls_inspect`, `net_ssh_exec`
- OCR: `ocr_tesseract`, `ocr_file`, `ocr_batch`
- Screenshot: `screenshot_capture`, `screenshot_region`, `screenshot_ocr`

**Risk Level:** High (requires explicit enablement)

**Security:** Only available when `PB_NETWORK_PROFILE=local` and `PB_TERMINAL_ENABLE=1`

### public-safe

**Purpose:** Remote-safe HTTP subset for public/external access

**Tools:**
- Health: `workspace_context` (read-only fields), `agent_bus_status`
- Files: bounded `read_file`, `list_files`, `grep_code`
- Bus: `agent_recent`, `agent_thread`, `agent_inbox` (read-only)
- Web: `web_dns_lookup`, `web_tls_inspect`, `web_rss_fetch`

**Risk Level:** Very Low (bounded, read-only)

**Security:** Only available via HTTP gateway with `PB_GATEWAY_TOKEN` authentication

### experimental-lab

**Purpose:** Advanced OCR/ML/AI/algorithm tools

**Additional Tools:**
- AI/ML: `ai_few_shot_classify`, `ai_zero_shot_classify`, `ai_structured_extract`, `ai_embed_text`, `ai_semantic_search`, `ai_chain_of_thought`, `ai_self_critique`, `ai_debate`, and 14 more
- Logging: `log_tail`, `log_search`, `log_level_summary`, `log_error_cluster`, `log_parse_json`, `log_parse_nginx`, `log_parse_syslog`, and 7 more
- Charts: `chart_line`, `chart_bar`, `chart_pie`, `chart_histogram`, `chart_scatter`, `chart_timeseries`, `chart_dependency_graph`, `chart_gantt`
- Neural: `neural_model_info`, `neural_inference`, `neural_tokenize`, `neural_embed_cosine`, `neural_tensor_stats`
- Algorithms: `algo_sort`, `algo_search`, `algo_graph_bfs`, `algo_graph_shortest_path`, `algo_fft`, and more

**Risk Level:** Variable

**Security:** Requires `experimental-lab` feature flag

---

## Tools Reference

### File System Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `read_file` | core-code | Read file with optional line range |
| `read_file_tail` | core-code | Read last N lines of a file |
| `list_files` | core-code | Directory listing |
| `project_tree` | core-code | Directory tree with depth limit |
| `find_files` | core-code | Glob-based file search |
| `find_files_by_name` | core-code | Find files by name pattern |
| `file_info` | core-code | File/directory metadata |
| `file_stats` | core-code | Line/word/character statistics |
| `json_validate_file` | core-code | Parse and validate JSON file |
| `batch_file_info` | core-code | Metadata for multiple paths |

### Git Tools (Read)

| Tool | Profile | Description |
|------|---------|-------------|
| `git_status` | core-code | Working directory status |
| `git_diff` | core-code | Diff with options |
| `git_log` | core-code | Commit history |
| `git_show` | core-code | Show commit/object |
| `git_blame` | review-refactor | Line blame information |
| `git_branches` | core-code | List all branches |
| `git_tag_list` | review-refactor | List tags |
| `git_reflog` | review-refactor | Reference log |
| `git_diff_stat` | core-code | Diff statistics summary |
| `git_last_commit` | core-code | Most recent commit info |
| `git_shortlog` | review-refactor | Contribution summary |

### Git Tools (Write)

| Tool | Profile | Description |
|------|---------|-------------|
| `git_checkout` | local-power | Switch branches or restore files |
| `git_add_or_commit` | local-power | Stage and create a commit |
| `git_push` | local-power | Push to remote (requires `PB_GIT_PUSH_ENABLE=1`) |
| `git_fetch` | local-power | Fetch from remote |
| `git_pull` | local-power | Pull from remote |
| `git_stash` | local-power | Stash changes |
| `git_stash_push` | local-power | Push named stash |
| `git_branch` | local-power | Create/delete/rename branches |
| `git_worktree` | local-power | Manage worktrees |

### Code Analysis Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `code_index_build` | core-code | Build symbol index for the project |
| `code_index_search` | core-code | Search indexed symbols |
| `code_index_paths_prefix` | core-code | Search paths by prefix |
| `code_index_extension_histogram` | core-code | Extension distribution |
| `codebase_largest_files` | review-refactor | Largest files by size |
| `codebase_duplicate_basenames` | review-refactor | Find duplicate filenames |
| `codebase_cargo_dependency_names` | review-refactor | List Cargo dependencies |
| `codebase_word_index` | review-refactor | Word frequency across codebase |
| `code_complexity_score` | review-refactor | Cyclomatic complexity estimate |
| `code_dead_code_hints` | review-refactor | Unused symbol hints |

### Text Processing Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `compress_context` | core-code | Compress/deduplicate context text |
| `pack_context` | core-code | Pack multiple files into context |
| `summarize_text` | core-code | Generate text summary |
| `text_stats` | core-code | Word/line/char statistics |
| `extract_snippets` | core-code | Extract code snippets |
| `text_levenshtein_distance` | core-code | Edit distance between strings |
| `text_token_jaccard` | review-refactor | Token similarity score |
| `summarize_diff_file_list` | review-refactor | Files changed in a diff |

### Agent Bus Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `agent_send` | agent-orchestration | Send message to the bus |
| `agent_inbox` | agent-orchestration | Poll inbox for messages |
| `agent_thread` | agent-orchestration | Read messages by topic |
| `agent_recent` | agent-orchestration | Recent bus messages |
| `ask_agents` | agent-orchestration | Broadcast question + await replies |
| `agent_handoff` | agent-orchestration | Structured agent handoff |
| `consensus_propose` | agent-orchestration | Propose a consensus vote |
| `consensus_vote` | agent-orchestration | Cast a vote |
| `consensus_tally` | agent-orchestration | Tally votes and get decision |
| `agent_fan_out` | agent-orchestration | Parallel fan-out to multiple agents |
| `agent_send_pipeline` | agent-orchestration | Serial pipeline between agents |

### Execution Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `terminal_run` | local-power | Execute allowlisted command |
| `terminal_run_stdin` | local-power | Execute command with stdin input |
| `project_cargo_test` | local-power | Run `cargo test` |
| `project_cargo_check` | local-power | Run `cargo check` |
| `project_pytest` | local-power | Run `pytest` |
| `terminal_which` | local-power | Locate binary in PATH |
| `terminal_program_version` | local-power | Get program version string |

### Planning/Reasoning Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `plan_definition_of_done` | review-refactor | Definition of Done template |
| `plan_execution_checklist` | review-refactor | Step-by-step execution checklist |
| `plan_risk_register` | review-refactor | Risk identification and mitigation |
| `plan_handoff_brief` | review-refactor | Agent handoff brief template |
| `plan_work_buckets` | review-refactor | Split work into parallel buckets |
| `reasoning_evidence_matrix` | review-refactor | Evidence collection matrix |
| `reasoning_premortem` | review-refactor | Pre-mortem failure analysis |
| `reasoning_five_whys` | review-refactor | Root-cause five-whys ladder |
| `reasoning_chain` | review-refactor | Structured reasoning chain |
| `reasoning_devils_advocate` | review-refactor | Counter-argument generation |

### Audio Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `audio_info` | local-power | Get audio file metadata |
| `audio_convert` | local-power | Convert audio formats |
| `audio_trim` | local-power | Trim audio segments |
| `audio_merge` | local-power | Merge audio files |
| `audio_normalize` | local-power | Normalize audio levels |

### Image Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `image_info` | local-power | Image metadata and dimensions |
| `image_resize` | local-power | Resize images |
| `image_convert` | local-power | Convert image formats |
| `image_crop` | local-power | Crop images |
| `image_rotate` | local-power | Rotate images |

### AI/ML Tools (Ollama-based)

| Tool | Profile | Description |
|------|---------|-------------|
| `ai_few_shot_classify` | experimental-lab | Few-shot text classification |
| `ai_zero_shot_classify` | experimental-lab | Zero-shot classification |
| `ai_structured_extract` | experimental-lab | Structured data extraction |
| `ai_embed_text` | experimental-lab | Text embedding generation |
| `ai_semantic_search` | experimental-lab | Semantic similarity search |
| `ai_chain_of_thought` | experimental-lab | Chain-of-thought reasoning |
| `ai_self_critique` | experimental-lab | Self-critique analysis |
| `ai_debate` | experimental-lab | Multi-model debate |

### Logging & Charts

| Tool | Profile | Description |
|------|---------|-------------|
| `log_tail` | experimental-lab | Tail log files |
| `log_search` | experimental-lab | Search log entries |
| `log_level_summary` | experimental-lab | Log level distribution |
| `log_error_cluster` | experimental-lab | Cluster similar errors |
| `chart_line` | experimental-lab | Generate SVG line charts |
| `chart_bar` | experimental-lab | Generate SVG bar charts |
| `chart_pie` | experimental-lab | Generate SVG pie charts |
| `chart_gantt` | experimental-lab | Generate SVG Gantt charts |

### Neural Network Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `neural_model_info` | experimental-lab | ONNX model metadata inspection |
| `neural_inference` | experimental-lab | ONNX Runtime inference |
| `neural_tokenize` | experimental-lab | HuggingFace tokenization |
| `neural_embed_cosine` | experimental-lab | Cosine similarity (pure math) |
| `neural_tensor_stats` | experimental-lab | Tensor statistics |

### Network & Security Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `net_ping` | local-power | Network ping |
| `net_port_scan` | local-power | Port scanning |
| `net_tls_inspect` | local-power | TLS certificate inspection |
| `protect_scan_file` | local-power | Malware/vulnerability scan |
| `protect_secret_scan` | local-power | Secret detection in files |
| `protect_dependency_audit` | local-power | Dependency vulnerability audit |

> **Note:** The tables above show representative tools from each category. The full MCP handler surface includes 500+ tools. Run `workspace_context` in an agent to see the complete list.

---

## Security Model

> See [SECURITY.md](SECURITY.md) for the vulnerability reporting policy.

### Defense in Depth

pb-mcp-server follows defense-in-depth principles:

```
┌─────────────────────────────────────────────────────────────┐
│                      Input Validation                        │
│  - Path sanitization                                        │
│  - Argument validation                                       │
│  - Input length limits                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      Path Sandboxing                         │
│  - All paths confined to project root                        │
│  - Symlink resolution with depth limit (max 8)               │
│  - Component-level path traversal prevention                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Command Allowlist                         │
│  - Only approved executables can run                         │
│  - Shell metacharacter blocking                              │
│  - Per-program subcommand restrictions                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                Network Message Signing                       │
│  - HMAC-SHA256 on all fields (id, from, to, kind, nonce,     │
│    timestamp, body)                                          │
│  - Per-message nonce prevents replay attacks                 │
│  - Constant-time signature verification                      │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                     Rate Limiting                            │
│  - Operation-level limits                                    │
│  - Per-agent bus quotas                                      │
│  - Burst prevention                                          │
└─────────────────────────────────────────────────────────────┘
```

### Path Validation

All file operations use `PathGuard` which:

1. **Canonicalizes paths** - Resolves `.` and `..` components
2. **Resolves symlinks** - With depth limit (max 8)
3. **Verifies containment** - Ensures path stays under root
4. **Blocks escapes** - Prevents traversal attacks
5. **Checks extensions** - Blocks dangerous file types
6. **Enforces limits** - File size and path length

```rust
// Example: Path validation in action
let guard = PathGuard::new("/project/root")?;

// Valid: stays within root
guard.validate_for_read("src/main.rs")?; // ✓

// Blocked: path traversal attempt
guard.validate_for_read("../../../etc/passwd")?; // ✗ Error

// Blocked: symlink escape
guard.validate_for_read("symlink_to_outside")?; // ✗ Error
```

### Command Policy

Terminal execution uses `CommandPolicy` which:

1. **Allowlists programs** - Only approved executables
2. **Validates arguments** - Length, content, patterns
3. **Restricts subcommands** - Per-program policies
4. **Blocks patterns** - Shell metacharacters, injection attempts
5. **Enforces limits** - Execution time, output size

```rust
// Example: Command validation
let policy = CommandPolicy::new();

// Allowed: git status
policy.validate("git", &["status"])?; // ✓

// Blocked: not in allowlist
policy.validate("rm", &["-rf", "/"])?; // ✗ Error

// Blocked: dangerous subcommand
policy.validate("git", &["push"])?; // ✗ Error (profile-gated)

// Blocked: injection attempt
policy.validate("git", &["status", ";", "rm", "-rf", "/"])?; // ✗ Error
```

### Environment Gates

| Variable | Purpose | Default |
|----------|---------|--------|
| `PB_PROJECT_ROOT` | Path confinement root | Required |
| `PB_MCP_PROFILE` | Tool profile selection | `core-code` |
| `PB_NETWORK_PROFILE` | `local` or `public` | `local` |
| `PB_BUS_READ_ONLY` | Disable all bus writes | `false` |
| `PB_TERMINAL_ENABLE` | Enable terminal execution tools | `false` |
| `PB_TERMINAL_ALLOW` | Comma-separated command allowlist | (none) |
| `PB_GIT_PUSH_ENABLE` | Enable `git_push` | `false` |
| `PB_GIT_REMOTE_ENABLE` | Enable `git_fetch` / `git_pull` | `false` |
| `PB_PROJECT_TEST_ENABLE` | Enable test runner tools | `false` |
| `PB_GATEWAY_TOKEN` | Bearer token for HTTP gateway auth | (none) |
| `PB_GATEWAY_SIGNING_SECRET` | HMAC secret for request signing | (none) |
| `PB_NODE_SECRET` | HMAC secret for inter-node signing | (none) |

> **Note:** `MINT_*` variable names are supported as backward-compatible aliases (e.g. `MINT_PROJECT_ROOT` = `PB_PROJECT_ROOT`).

---

## Configuration

### Environment Variables

Create a `.env` file or export variables before running the server:

```bash
# Required: Project root (all file operations are sandboxed here)
export PB_PROJECT_ROOT=/path/to/your/project

# Profile selection (optional, default: core-code)
export PB_MCP_PROFILE=core-code

# Network profile (optional, default: local)
export PB_NETWORK_PROFILE=local

# Enable terminal execution (optional, off by default)
export PB_TERMINAL_ENABLE=1
export PB_TERMINAL_ALLOW=git,cargo,npm,python

# Enable git remote operations (optional)
export PB_GIT_PUSH_ENABLE=1
export PB_GIT_REMOTE_ENABLE=1

# Rate limiting (optional)
export PB_BUS_MAX_SENDS_PER_WINDOW=100
export PB_BUS_WINDOW_SECS=60

# Logging
export RUST_LOG=info
```

> **Tip:** Copy `setup.example.json` to `setup.json` and set values there — Zed reads environment variables directly from the MCP config.

### Profile Selection

Select profile via `PB_MCP_PROFILE` environment variable:

```bash
# Default read-only coding profile
export PB_MCP_PROFILE=core-code

# PR review and architecture analysis
export PB_MCP_PROFILE=review-refactor

# Multi-agent coordination
export PB_MCP_PROFILE=agent-orchestration

# Full local execution (requires TERMINAL_ENABLE and NETWORK_PROFILE=local)
export PB_MCP_PROFILE=local-power

# Advanced ML/AI tools (requires experimental-lab feature)
export PB_MCP_PROFILE=experimental-lab
```

### Feature Flags

Enable features at compile time:

```bash
# Build with default features
cargo build --release

# Build with local-power tools
cargo build --release --features local-power

# Build with experimental ML tools
cargo build --release --features experimental-lab

# Build with all features
cargo build --release --features full

# Build all package binaries for available release targets into dist/
make release-matrix

# Build Windows x64 with exact all-features
cargo build --release -p pb-mcp-server --all-features --bins --target x86_64-pc-windows-gnu

# Build the lighter Windows-compatible profile
cargo build --release -p pb-mcp-server --features full-windows --bin pb-mcp-server --target x86_64-pc-windows-gnu
```

`scripts/build-release-matrix.sh` builds every package binary with exact
`--all-features` by default, copies artifacts into `dist/<target>/`, and writes
`dist/SHA256SUMS` plus `dist/build-manifest.jsonl`. macOS targets are skipped
from Linux unless an Apple SDK/osxcross-style linker is configured. The
`full-windows` profile remains available as a lighter Windows-compatible build,
but exact Windows `--all-features` is supported when the MinGW target is
installed.

---

## Multi-Agent Bus

### Overview

The agent bus is an append-only JSONL file for multi-agent coordination:

```
.mint-mcp/
└── agent-bus.jsonl    # Durable message log
```

### Message Format

Each message is a JSON object:

```json
{
  "id": "1775308370692-144329",
  "ts": 1775308370692,
  "from_agent": "cursor-main",
  "to_agent": "review-bot",
  "topic": "pr-review-42",
  "kind": "handoff",
  "body": "Please review the auth module changes"
}
```

### Usage Examples

#### Send a Message

```bash
# Direct message
agent_send(from_agent="cursor-main", message="Check this", to_agent="review-bot")

# Broadcast
agent_send(from_agent="cursor-main", message="Anyone available?", kind="question")

# Topic message
agent_send(from_agent="cursor-main", message="Starting review", topic="pr-42")
```

#### Poll Inbox

```bash
# Get latest messages
agent_inbox(for_agent="review-bot", limit=50)

# Incremental polling (store last_ts from response)
agent_inbox(for_agent="review-bot", since_ts_ms=1775308370692)
```

#### Read Thread

```bash
# Get all messages in a topic
agent_thread(topic="pr-42", limit=100)
```

#### Orchestration

```bash
# Ask multiple agents and wait for replies
ask_agents(
  from_agent="cursor-main",
  question="What's the best approach for auth?",
  expected_count=3,
  timeout_ms=30000
)
```

### Bus Management

```bash
# Check bus status
agent_bus_status()

# Compact old messages
agent_bus_compact(keep_count=1000)

# Search messages
agent_bus_search(pattern="auth", from_agent="review-bot")
```

---

## HTTP Gateway

### Overview

The HTTP gateway provides remote access to a subset of tools:

```bash
# Start gateway
PB_GATEWAY_BIND=127.0.0.1:8080 \
PB_GATEWAY_TOKEN=your-secure-token \
MINT_PROJECT_ROOT=/path/to/project \
./target/release/pb-gateway
```

### Endpoints

#### Health & Status

| Endpoint | Auth | Description |
|----------|------|-------------|
| `GET /health` | No | Health check |
| `GET /status` | Yes | Server status |
| `GET /security-status` | Yes | Security posture |

#### Files (Bounded)

| Endpoint | Auth | Description |
|----------|------|-------------|
| `GET /list-files?path=...` | Optional | Directory listing |
| `GET /read-file?path=...` | Optional | File read (bounded) |
| `GET /search-code?pattern=...` | Yes | Code search |

#### Agent Bus (Read-only)

| Endpoint | Auth | Description |
|----------|------|-------------|
| `POST /agent-send` | Yes | Send message |
| `GET /agent-recent?limit=...` | Optional | Recent messages |
| `GET /agent-thread?topic=...` | Optional | Thread view |

#### Review

| Endpoint | Auth | Description |
|----------|------|-------------|
| `POST /review-packet` | Yes | Store review data |
| `GET /review-packets?limit=...` | Yes | Get review data |

#### Execution (Local Only)

| Endpoint | Auth | Description |
|----------|------|-------------|
| `POST /exec` | Local only | Execute command |

### Security Configuration

```bash
# Gateway authentication
export PB_GATEWAY_TOKEN=your-secure-token-here

# Request signing (anti-replay)
export PB_GATEWAY_SIGNING_SECRET=your-signing-secret

# Trust local network (disables auth for LAN/Tailscale)
export PB_GATEWAY_TRUST_LOCAL_NETWORK=1

# Bind address (keep localhost for security)
export PB_GATEWAY_BIND=127.0.0.1:8080
```

### Example Requests

```bash
# Health check (no auth)
curl http://localhost:8080/health

# Status (with auth)
curl -H "Authorization: Bearer your-token" \
  http://localhost:8080/status

# Read file (bounded)
curl -H "Authorization: Bearer your-token" \
  "http://localhost:8080/read-file?path=src/main.rs&start_line=1&end_line=100"

# Send agent message
curl -X POST \
  -H "Authorization: Bearer your-token" \
  -H "Content-Type: application/json" \
  -d '{"from_agent":"remote","message":"Hello from remote"}' \
  http://localhost:8080/agent-send
```

---

## Development

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/PanosBiazis/pb-mcp-server.git
cd pb-mcp-server
cargo build

# Check all workspace members
cargo check --workspace

# Test all crates (excluding GUI dashboard)
cargo test --workspace --exclude pb-dashboard

# Lint entire workspace
cargo clippy --workspace --exclude pb-dashboard -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_path_validation

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check

# Security audit
cargo audit
```

### Documentation

```bash
# Build docs
cargo doc --no-deps --open

# Build with all features
cargo doc --all-features --open
```

### Adding New Tools

1. **Define arguments** in `src/args/<category>.rs`:

```rust
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MyToolArgs {
    pub input: String,
    #[serde(default)]
    pub option: Option<String>,
}
```

2. **Implement logic** in `src/ops/<category>.rs`:

```rust
pub fn my_tool_work(args: MyToolArgs) -> Result<String> {
    // Implementation
}
```

3. **Add handler** in `src/main.rs` (inside `impl MintMcp`):

```rust
#[tool(description = "My tool does X")]
async fn my_tool(&self, Parameters(args): Parameters<MyToolArgs>) -> Result<CallToolResult, McpError> {
    self.require_trusted_deployment()?;
    let root = self.resolve_path(&args.input)?;
    let out = tokio::task::spawn_blocking(move || {
        ops::my_tool_work(serde_json::json!({ "path": root }))
    }).await.unwrap_or_else(|e| format!("Error: {e}"));
    Self::tool_text(out)
}
4. **Add tests** in the `#[cfg(test)]` mod block of the relevant `src/ops/*.rs` file:

```rust
#[test]
fn test_my_tool() {
    let args = MyToolArgs { input: "test".into(), option: None };
    let result = ops::my_tool_work(args).unwrap();
    assert!(result.contains("expected"));
}
```

---

## Benchmarks

Criterion benchmarks measure key subsystems:

```sh
# Run all benchmarks
cargo bench

# Bus I/O (JSONL append + read performance, 10–1000 messages)
cargo bench --bench bus_performance

# Code indexing (symbol extraction, grep search, 10–100 files)
cargo bench --bench code_index
```

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy && cargo fmt --check`)
6. Commit with conventional commits (`feat: add amazing feature`)
7. Push to your branch
8. Open a Pull Request

### Code Style

- Use `rustfmt` for formatting
- Follow clippy recommendations
- Add documentation comments
- Write tests for new functionality
- Update documentation

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new tool for X
fix: resolve path validation issue
docs: update README with examples
test: add tests for Y
refactor: reorganize module structure
security: fix timing oracle in HMAC verify
chore: update dependencies
```

### Reporting Security Issues

> **Do not open a public issue for security vulnerabilities.**
>
> Please follow the process described in [SECURITY.md](SECURITY.md).
> Email **panosbiazis2003@gmail.com** with subject `[SECURITY] pb-mcp-server`.

---

## Roadmap

### v0.4.0 (Planned)

- [ ] WebAssembly tool support
- [ ] Distributed bus (multi-node with nonce deduplication)
- [ ] Advanced ML model integration
- [ ] Real-time collaboration features
- [ ] Runtime plugin loading for custom tools
- [ ] Rate limiting for expensive tools (neural inference, chart generation)
- [ ] OpenTelemetry tracing for all tool invocations

### v0.3.0

- [x] Expanded app/workflow local-equivalent tools for n8n, MCP builders, Codex, Claude Code, OpenCode, OpenClaw, and Hermes style workflows
- [x] Added local skill/plugin/command planning and validation helpers
- [x] Added bash, PowerShell, batch/cmd, and macOS Terminal command catalog, translation, script-template, and safety-check helpers
- [x] Increased active MCP handler surface to 564 handlers

### v0.2.1

- [x] Fixed HMAC timing oracle in `network/protocol.rs`
- [x] Added nonce-based replay protection for inter-node messages
- [x] HMAC now covers all message fields (`to_node`, `kind`, `nonce`)
- [x] `NetworkConfig::warn_if_weak_secret()` warns on empty/short `PB_NODE_SECRET`
- [x] Fixed 20 compilation errors (ort 2.x, image 0.25 API changes)
- [x] Created `LICENSE` (MIT) and `SECURITY.md`
- [x] Complete documentation refresh

### v0.2.0

- [x] Profile-based tool system (6 profiles)
- [x] Enhanced security model (path guard, command policy, rate limiter)
- [x] Comprehensive test suite (1,292+ tests)
- [x] Modular architecture (28 operational modules)
- [x] Extended tool catalog (500+ tools)
- [x] AI/ML integration (22 Ollama-based tools)
- [x] Logging & SVG chart generation (22 tools)
- [x] Neural network tools (5 ONNX/tokenizer tools)
- [x] Multi-agent consensus engine
- [x] HTTP gateway with HMAC-SHA256 auth
- [x] P2P networking via libp2p

### v0.1.0 (Initial)

- [x] Basic MCP server
- [x] File/git/search tools
- [x] Multi-agent bus
- [x] HTTP gateway
- [x] Netlify control plane

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- [MCP Protocol](https://modelcontextprotocol.io/) - Model Context Protocol
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- [Zed](https://zed.dev/) - High-performance editor
- All contributors and users

---

## Support

- **Issues**: [GitHub Issues](https://github.com/PanosBiazis/pb-mcp-server/issues)
- **Discussions**: [GitHub Discussions](https://github.com/PanosBiazis/pb-mcp-server/discussions)
- **Documentation**: [docs/](docs/)

---

**Built with ❤️ for the AI agent community**
