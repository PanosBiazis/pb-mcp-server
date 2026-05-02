# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cargo workspace with 5 members: root crate + `pb-dashboard`, `pb-gateway`, `pb-lsp-bridge`, `pb-network-node` as independent crates
- 253-entry `finetune_data.jsonl` covering all tool categories for LLM fine-tuning
- Criterion benchmarks: `bus_performance` (JSONL append/read) and `code_index` (indexing/search)
- 49 unit tests for `pb-gateway` (sanitize_agent_name, resolve_under_root, hex_lower, canonical_sign_input, is_local_network_ip, now_ms, default helpers)
- `required-features` declarations for feature-gated `src/bin/` binaries (pb-dashboard, pb-lsp-bridge, pb-network-node) to prevent "main function not found" errors
- `RUST_BACKTRACE=1` to CI environment
- New CI jobs: `check-workspace` (all 5 members), `test-workspace` (all non-GUI crates), `doc` (cargo doc --workspace)
- `rustsec/audit-check@v2` action replaces manual `cargo install cargo-audit`
- `cargo fmt --all` in CI covers entire workspace

### Changed
- `src/main.rs` reduced by 2,665 lines (29%) via extraction of argument structs to `src/mcp_args.rs`
- `mcp_args.rs` (2,694 lines): 186 `*Args` structs + default helper functions
- 23 missing catalog entries added for web, convert, system, notify, and db tools — catalog now complete at 253/253
- Migrated from archived `serde_yaml` to maintained `serde_yml` fork
- `tokio-tungstenite` migrated from `native-tls` to `rustls-tls-native-roots` (OpenSSL eliminated)
- `reqwest` now uses explicit `rustls-tls` feature flag
- LTO profile upgraded from `thin` to `fat` for better dead-code elimination
- `DEFAULT_MAX_FILE_READ_BYTES` increased from 512 KB to 4 MB
- `pb-dashboard/src/main.rs` simplified: `required-features = ["dashboard"]` eliminates `#[cfg(not(feature = ...))]` fallback block and unreachable_code warning

### Fixed
- `verify_under_root()` now uses `Path::starts_with()` instead of unsafe string prefix comparison (path traversal vulnerability)
- `DANGEROUS_EXTENSIONS` blocklist expanded (+8 extensions)
- Audit log now emits `tracing::warn!` when initialization fails (was silently degrading)
- `check_tool_access()` records audit entries for all access checks
- `#![deny(unsafe_code)]` added to `lib.rs`
- `[lints.rust]` and `[lints.clippy]` sections added to `Cargo.toml`
- Replaced archived `serde_yaml` crate with maintained `serde_yml` fork in `error.rs` and `network/tor.rs`
- Fixed 3 failing tests: IDE tools handle missing IDE gracefully; `web.fetch` risk level corrected to "low"; catalog consistency enforced
- Eliminated all compiler warnings (0 warnings on code-level checks)
- IDE test functions (`test_jetbrains_*`) now gracefully skip when IDEA is not installed
- `setup.json` removed from version control; `setup.example.json` added as template
- `tools_wired.rs` orphaned duplicate removed (implementations are inline in `main.rs`)

## [0.2.0] - 2025-04-23

### Added

- Multi-agent consensus/voting system with 6 strategies (SimpleMajority, SuperMajority, Unanimous, WeightedVoting, RankedChoice, Delegation)
- MCP tools: `consensus_propose`, `consensus_vote`, `consensus_tally`, `consensus_list`, `consensus_close`
- Real libp2p Swarm P2P networking with Kademlia DHT, Gossipsub, Identify, and Ping protocols
- Tor SOCKS5 proxy integration with full handshake implementation
- Agent Worker system with Ollama HTTP client and role-based LLM agents
- Admin Dashboard binary (`pb-dashboard`) with egui/eframe
- LSP Bridge binary (`pb-lsp-bridge`) for Android Studio and other LSP-based IDEs
- Network Node binary (`pb-network-node`) for scalable P2P deployments
- GitHub Actions CI workflow (check, test, clippy, fmt, audit)
- Makefile with developer convenience targets
- PB_* environment variable naming (MINT_* preserved as backward-compatible fallbacks)
- Tool catalog and registry with profile-aware gating
- Security audit logging and rate limiting implementations
- Resilience patterns: circuit breaker, retry with backoff, bulkhead, timeout
- File caching with smart invalidation and optional mmap support
- Health monitoring and metrics collection
- Anti-malware/secret scanning tools (SecretScanner, IntegrityChecker)

### Changed

- Gateway routes.rs restored missing std imports
- PooledWorkerHandle made public with proper documentation
- Resilience retry `execute_async` feature gate removed (always available)
- Tor connection validation now uses TorProxy.check_tor_running()

### Fixed

- screenshot_capture_work call signature mismatch in main.rs
- sandbox.rs borrow-after-move errors in tests
- ConsensusEngine Debug/Clone derives for closure field
- Tor SOCKS5 test assertions (domain length, onion address validation)
- Ranked choice IRV test determinism