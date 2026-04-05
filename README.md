# pb-mcp-server

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
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

`pb-mcp-server` is a Rust-based MCP (Model Context Protocol) server designed for IDE agents in Zed, Cursor, and similar editors. It provides:

- **Profile-based tool exposure** - Different tool sets for different workflows
- **Multi-agent coordination** - Durable append-only message bus for agent communication
- **Code intelligence** - Symbol indexing, search, and analysis tools
- **Safe execution** - Sandboxed terminal operations with allowlists
- **Review tools** - PR review, architecture analysis, risk assessment

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

### 🛡️ Security Features

- **Path sandboxing** - All file operations confined to project root
- **Command allowlists** - Only approved executables can run
- **Rate limiting** - Prevent abuse and resource exhaustion
- **Profile gating** - Dangerous tools require explicit enablement
- **Symlink protection** - Prevents escape via symlink resolution
- **Input validation** - All inputs validated at entry points

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
```

### Run in Zed

1. Copy `setup.json` to your Zed configuration:
   ```bash
   cp setup.json ~/.config/zed/
   ```

2. Set the environment variable:
   ```bash
   export MINT_PROJECT_ROOT=/path/to/your/project
   ```

3. Restart Zed - the MCP server will connect automatically

### Verify Installation

```bash
# Check the server version
MINT_PROJECT_ROOT=$(pwd) ./target/release/pb-mcp-server --version

# Quick health check (if gateway is running)
curl http://localhost:8080/health
```

---

## Architecture

```
pb-mcp-server/
├── src/
│   ├── main.rs              # Entry point + MCP handler
│   ├── lib.rs               # Library root with re-exports
│   ├── server/              # MCP server implementation
│   │   ├── mod.rs           # Server core
│   │   ├── router.rs        # Tool routing
│   │   └── profiles.rs      # Profile registration
│   ├── tools/               # Tool handlers by category
│   │   ├── fs.rs            # File system tools
│   │   ├── git_read.rs      # Git read operations
│   │   ├── git_write.rs     # Git write operations
│   │   ├── code_index.rs    # Code indexing
│   │   ├── agent_bus.rs     # Multi-agent bus
│   │   ├── planning.rs      # Planning/reasoning
│   │   └── execution.rs     # Terminal execution
│   ├── ops/                 # Business logic
│   │   ├── file.rs          # File operations
│   │   ├── git.rs           # Git operations
│   │   ├── text.rs          # Text processing
│   │   └── compress.rs      # Compression utilities
│   ├── security/            # Security modules
│   │   ├── path_guard.rs    # Path validation
│   │   ├── command_policy.rs # Execution policy
│   │   └── rate_limiter.rs  # Rate limiting
│   ├── args/                # Argument structures
│   ├── bus.rs               # Durable message bus
│   ├── code_index.rs        # Symbol indexing
│   ├── thinking.rs          # Planning scaffolds
│   └── profiles.rs          # Profile definitions
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
└── docs/                    # Documentation
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
- File: `fs.read_file`, `fs.read_tail`, `fs.list`, `fs.tree`, `fs.find`, `fs.info`
- Git: `git.read.status`, `git.read.diff`, `git.read.log`, `git.read.show`
- Code: `code.index.search`, `code.analyze.inventory`
- Text: `text.transform`, `text.summarize`, `text.stats`
- Context: `pack_context`, `workspace_context`

**Risk Level:** Low (read-only)

### review-refactor

**Purpose:** PR review, architecture understanding, safe codebase inspection

**Additional Tools:**
- Git: `git.read.blame`, `git.read.tags`, `git.diff_snippets`
- Code: `code.analyze.largest_files`, `code.analyze.duplicate_basenames`
- Review: `review.diff_risk`, `review.focused_review`, `review.missing_tests`
- Reasoning: `reasoning.evidence_matrix`, `plan.risk_register`

**Risk Level:** Low (read-only)

### agent-orchestration

**Purpose:** Multi-agent and subagent coordination

**Tools:**
- Bus: `agent.bus.send`, `agent.bus.inbox`, `agent.bus.thread`, `agent.bus.recent`
- Coordination: `agent.ask`, `agent.handoff`, `agent.consensus`
- Temp: `agent.temp.write`, `agent.temp.read`, `agent.temp.list`, `agent.temp.delete`
- Wait: `agent.wait.sleep`, `agent.wait.for_path`, `agent.wait.until`

**Risk Level:** Medium

### local-power

**Purpose:** Explicitly approved local execution and validation

**Tools:**
- Terminal: `local.exec.run`, `local.exec.run_stdin`
- Tests: `local.test.rust_test`, `local.test.rust_check`, `local.test.pytest`
- Git write: `git.write.commit`, `git.write.push`, `git.write.checkout`
- OCR: `local.ocr.tesseract`
- Screenshot: `local.screenshot.capture`

**Risk Level:** High (requires explicit enablement)

**Security:** Only available when `MINT_NETWORK_PROFILE=local`

### public-safe

**Purpose:** Remote-safe HTTP subset for public/external access

**Tools:**
- Health: `health`, `status`, `security_status`
- Info: `project_info`
- Files: bounded `list_files`, bounded `read_file`
- Search: bounded `search_code`
- Bus: `agent.bus.recent`, `agent.bus.thread` (read-only)

**Risk Level:** Very Low (bounded, read-only)

**Security:** Only available via HTTP gateway with authentication

### experimental-lab

**Purpose:** Advanced OCR/ML/AI/algorithm tools

**Additional Tools:**
- OCR: `ocr.tesseract_stream`, `ocr.tesseract_batch`
- ML: `ml.embed`, `ml.classify`, `ml.detect`
- Algorithms: `algorithm.complexity`, `algorithm.dependencies`

**Risk Level:** Variable

**Security:** Requires `experimental-lab` feature flag

---

## Tools Reference

### File System Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `fs.read_file` | core-code | Read file with line slicing |
| `fs.read_tail` | core-code | Read last N lines |
| `fs.list` | core-code | Directory listing |
| `fs.tree` | core-code | Directory tree with depth limit |
| `fs.find` | core-code | Glob-based file search |
| `fs.info` | core-code | File/directory metadata |
| `fs.read_json` | core-code | Parse and validate JSON file |
| `fs.stats` | core-code | Line/word/character statistics |

### Git Tools (Read)

| Tool | Profile | Description |
|------|---------|-------------|
| `git.read.status` | core-code | Working directory status |
| `git.read.diff` | core-code | Diff with options |
| `git.read.log` | core-code | Commit history |
| `git.read.show` | core-code | Show commit/object |
| `git.read.blame` | review-refactor | Line blame information |
| `git.read.branches` | core-code | List branches |
| `git.read.tags` | review-refactor | List tags |
| `git.read.reflog` | review-refactor | Reference log |

### Git Tools (Write)

| Tool | Profile | Description |
|------|---------|-------------|
| `git.write.checkout` | local-power | Switch branches |
| `git.write.commit` | local-power | Create commit |
| `git.write.push` | local-power | Push to remote |
| `git.write.fetch` | local-power | Fetch from remote |
| `git.write.stash` | local-power | Stash changes |

### Code Analysis Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `code.index.build` | core-code | Build symbol index |
| `code.index.search` | core-code | Search indexed symbols |
| `code.index.paths_prefix` | core-code | Search paths by prefix |
| `code.analyze.inventory` | core-code | Extension histogram |
| `code.analyze.largest_files` | review-refactor | Size ranking |
| `code.analyze.duplicate_basenames` | review-refactor | Find duplicate filenames |
| `code.analyze.deps` | review-refactor | Dependency analysis |
| `code.analyze.word_index` | review-refactor | Word frequency |

### Text Processing Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `text.transform` | core-code | Apply transformations |
| `text.summarize` | core-code | Generate summary |
| `text.stats` | core-code | Text statistics |
| `text.extract_snippets` | core-code | Extract code snippets |
| `text.compare` | review-refactor | Compare texts |

### Agent Bus Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `agent.bus.send` | agent-orchestration | Send message to bus |
| `agent.bus.inbox` | agent-orchestration | Poll inbox for messages |
| `agent.bus.thread` | agent-orchestration | Read topic thread |
| `agent.bus.recent` | agent-orchestration | Recent bus messages |
| `agent.ask` | agent-orchestration | Broadcast + wait for replies |
| `agent.handoff` | agent-orchestration | Structured handoff message |
| `agent.consensus` | agent-orchestration | Consensus decision making |

### Execution Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `local.exec.run` | local-power | Execute allowlisted command |
| `local.exec.run_stdin` | local-power | Execute with stdin input |
| `local.test.rust_test` | local-power | Run cargo test |
| `local.test.rust_check` | local-power | Run cargo check |
| `local.test.pytest` | local-power | Run pytest |

### Planning/Reasoning Tools

| Tool | Profile | Description |
|------|---------|-------------|
| `plan.definition_of_done` | review-refactor | DoD template |
| `plan.execution_checklist` | review-refactor | Execution checklist |
| `plan.risk_register` | review-refactor | Risk assessment |
| `reasoning.evidence_matrix` | review-refactor | Evidence collection |
| `reasoning.premortem` | review-refactor | Premortem analysis |

---

## Security Model

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
│  - Symlink resolution with depth limit                       │
│  - Path traversal prevention                                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Command Allowlist                         │
│  - Only approved executables                                 │
│  - Argument validation                                       │
│  - Subcommand restrictions                                   │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                     Rate Limiting                            │
│  - Operation-level limits                                    │
│  - Per-agent quotas                                          │
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
|----------|---------|---------|
| `MINT_PROJECT_ROOT` | Path confinement | Required |
| `MINT_NETWORK_PROFILE` | `local` or `public` | `local` |
| `MINT_BUS_READ_ONLY` | Disable bus writes | `false` |
| `MINT_TERMINAL_ENABLE` | Enable terminal tools | `false` |
| `MINT_TERMINAL_ALLOW` | Command allowlist | (see docs) |
| `MINT_GIT_PUSH_ENABLE` | Enable git push | `false` |
| `MINT_GIT_REMOTE_ENABLE` | Enable fetch/pull | `false` |
| `MINT_PROJECT_TEST_ENABLE` | Enable test runners | `false` |

---

## Configuration

### Environment Variables

Create a `.env` file or export variables:

```bash
# Required: Project root
export MINT_PROJECT_ROOT=/path/to/your/project

# Profile selection (optional)
export PB_MCP_PROFILE=core-code

# Network profile (optional)
export MINT_NETWORK_PROFILE=local

# Enable terminal execution (optional)
export MINT_TERMINAL_ENABLE=1
export MINT_TERMINAL_ALLOW=git,cargo,npm,python

# Rate limiting (optional)
export MINT_BUS_MAX_SENDS_PER_WINDOW=100
export MINT_BUS_WINDOW_SECS=60
```

### Profile Selection

Select profile via environment variable:

```bash
# Use default profile (core-code)
export PB_MCP_PROFILE=default

# Use multiple profiles (additive)
export PB_MCP_PROFILE=core-code,agent-orchestration

# Use specific profile
export PB_MCP_PROFILE=local-power
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
```

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

3. **Add handler** in `src/tools/<category>.rs`:

```rust
#[tool]
async fn my_tool(&self, args: MyToolArgs) -> Result<String, McpError> {
    Ok(ops::my_tool_work(args)?)
}
```

4. **Register in profile** in `src/profiles.rs`:

```rust
ToolProfile::CoreCode => vec![
    // ... existing tools
    "my_tool",
]
```

5. **Add tests** in `tests/<category>_tests.rs`:

```rust
#[test]
fn test_my_tool() {
    let args = MyToolArgs { input: "test".into(), option: None };
    let result = ops::my_tool_work(args).unwrap();
    assert!(result.contains("expected"));
}
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
chore: update dependencies
```

---

## Roadmap

### v0.3.0 (Planned)

- [ ] WebAssembly tool support
- [ ] Distributed bus (multi-node)
- [ ] Advanced ML model integration
- [ ] Real-time collaboration features
- [ ] Plugin system for custom tools

### v0.2.0 (Current)

- [x] Profile-based tool system
- [x] Enhanced security model
- [x] Comprehensive test suite
- [x] Modular architecture
- [x] Extended tool catalog

### v0.1.0 (Initial)

- [x] Basic MCP server
- [x] File/git/search tools
- [x] Multi-agent bus
- [x] HTTP gateway
- [x] Netlify control plane

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

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