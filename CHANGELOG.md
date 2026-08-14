# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- **Canonical bounded gateway:** consolidated the three gateway implementations
  into `src/gateway/`; both binaries are thin wrappers around the same
  `serve_from_env`. `/read-file` now streams at most 512 KiB instead of reading
  an arbitrary file into memory. `/exec` is default-deny behind a separate
  `PB_GATEWAY_EXEC_ENABLE` flag, is restricted to trusted loopback requests,
  bounds stdout/stderr while draining, and terminates the Unix process group
  plus discovered descendants on timeout. Non-loopback startup fails closed
  unless bearer or signature authentication is configured. Metrics now report
  monotonic process uptime rather than Unix epoch seconds.
- **Reproducible patched dependencies:** the two minimal `libp2p` and `eframe`
  patch trees referenced by `[patch.crates-io]` are now explicitly versioned
  instead of hidden by `vendor/`, restoring clean-checkout resolution.

- **Gateway `trust_local_network` bind-exposure gap (High)**: the guard that refuses
  `PB_GATEWAY_TRUST_LOCAL_NETWORK=true` combined with a non-loopback bind lived only in the
  standalone `pb-gateway` binary's `main()`; the reusable library serve path (`AppState::from_env`
  + `build_router` + `axum::serve`, the documented embedding) had none — exposing the token/
  signature auth bypass and `POST /exec` (shell) to the local subnet on a `0.0.0.0` bind. Added a
  reusable, fail-closed `gateway::assert_bind_safe(bind, trust_local_network)` (exported), used it
  to document `build_router`, and fixed the same "unparseable hostname ⇒ assumed loopback" hole in
  the standalone binary's inline check. 3 new unit tests.
- **Gateway anti-replay nonce cache DoS (Medium)**: bounded the in-memory
  nonce map in `src/gateway/auth.rs` (`verify_signature`) and the duplicated
  logic in `src/bin/pb-gateway.rs`. A new `MAX_NONCES = 100_000` cap evicts the
  oldest half of entries (by recorded timestamp) before inserting when the cap
  is hit, so an attacker streaming unique nonces within the skew window can no
  longer grow the map without bound. Replay protection for recent nonces is
  preserved.
- **Gateway request-body hashing DoS (Medium)**: added
  `tower_http::limit::RequestBodyLimitLayer` (10 MiB) to both the shared
  `gateway::build_router` (`src/gateway/routes.rs`) and the `pb-gateway`
  binary router (`src/bin/pb-gateway.rs`). Oversized bodies are rejected with
  `413 Payload Too Large` before handlers run, so a giant body can no longer be
  SHA-256/HMAC-hashed during signature verification. Uses the already-present
  `tower-http` `limit` feature; no new dependency.
- **`trust_local_network` misuse hardening (Medium)**: `PB_GATEWAY_TRUST_LOCAL_NETWORK=true`
  bypasses both token and signature auth for loopback/private/Tailscale peers.
  The `pb-gateway` binary now emits a loud `WARNING` at startup when it is
  enabled and **refuses to start** (fatal) when it is combined with a bind on a
  non-loopback interface, which would expose the auth bypass to the local
  subnet. The feature itself is retained for local development.
- **Sandbox shell command-substitution injection (Medium)**: extended
  `DANGEROUS_COMMAND_PATTERNS` in `src/ops/sandbox.rs` to block backtick
  (`` ` ``) and `$(` command substitution. These reach the shell-backed sandbox
  entry points (`sandbox_script` / `sandbox_docker_run`, which run through
  `sh -c`) and previously slipped past the phrase-based denylist. Added the
  `test_command_substitution_blocked` unit test; existing accepted inputs are
  unchanged.
- **`PB_MCP_PROFILE` silent-fallback misconfig (Low)**: `MintMcp::from_env`
  in `src/server.rs` now logs a `WARN` to stderr when `PB_MCP_PROFILE` is set to
  an unrecognised value before falling back to the default profile, instead of
  falling back silently (which could quietly expose the wrong tool surface).
- Verified existing path-guard symlink protections in
  `src/security/path_guard.rs`: the `MAX_SYMLINK_DEPTH = 8` cap, the clear
  "Symlink resolution depth exceeded" error, and the post-resolution
  `verify_under_root` re-check (blocking symlink escapes out of the project
  root) are correct and were left tightened as-is.
- Cleared the RustSec vulnerability set reported on 2026-06-13: updated TLS,
  websocket, tokenizer, scraper, and random dependencies; replaced unsound
  `serde_yml`/`libyml` with `serde_norway`; and patched the local `libp2p`
  dependency surface so unused vulnerable DNS/mDNS transitive crates are not
  locked.

### Changed

- Rebuilt all-features release binaries for Linux `x86_64-unknown-linux-gnu`
  and Windows `x86_64-pc-windows-gnu`; refreshed `dist/SHA256SUMS`.

## [0.2.1] - 2025-07-01

### Security

- **CRITICAL fix**: Removed hardcoded `PB_GATEWAY_TOKEN` value that was committed in `docs/architecture.md` — replaced with placeholder. If you were using this token, rotate it immediately.
- **High fix**: Fixed HMAC signature verification timing oracle in `network/protocol.rs` — replaced standard `==` string comparison with constant-time XOR comparison to prevent timing-based forgery attacks.
- **Medium fix**: `NodeMessage` now includes a `to_node`, `kind`, and `nonce` field in the HMAC-SHA256 signed payload — previously these fields were unsigned and could be modified by a man-in-the-middle attacker without breaking the signature.
- **Medium fix**: Added nonce field to `NodeMessage` for replay protection — each message now gets a unique nonce derived from PID, timestamp, and monotonic counter, making replayed messages detectable.
- **Medium fix**: Added `NetworkConfig::warn_if_weak_secret()` — emits `tracing::warn!` when `PB_NODE_SECRET` is empty or fewer than 32 characters at startup.
- Added four new security tests in `protocol.rs`: `sign_is_not_malleable_to_node_change`, `replay_protection_nonce_is_unique`, and extended existing roundtrip tests to assert nonce presence.
- Added four new config validation tests in `config.rs`: `has_strong_signing_secret_*` variants.

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
- Neural network tools (`neural.rs`) updated for ort 2.0.0-rc.12 API: `Outlet::name()` accessor, `dtype()` method for type info, `ort::value::Tensor::from_array()` for input creation, `try_extract_tensor()` returning `(&Shape, &[T])`
- Image rotation for arbitrary angles now snaps to nearest 90° with caller notification (image 0.25 removed `rotate_about_center`)
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
- Compilation errors (20 → 0): missing generated module files, ort 2.0.0-rc.12 API incompatibilities, image 0.25 API change (`rotate_about_center` removed), deprecated `image::io::Reader`
- Compiler warnings (6 → 0): unused imports in `agent/live.rs`, `cache/file.rs`, `ops/image.rs`; feature-gated re-exports in `agent/mod.rs`
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

### Documentation

- `README.md`: Updated tool count (253 → 484+), corrected architecture tree to match real directory structure, added 6 new tool category tables, updated experimental-lab profile listing, expanded v0.2.0 roadmap checklist
- `docs/info.md`: Complete rewrite — comprehensive project reference with architecture diagram, all 28 ops modules documented, feature flag table, security model, and expansion history
- `docs/architecture.md`: Removed hardcoded gateway token
- Created `SECURITY.md` with vulnerability reporting policy
- Created `LICENSE` (MIT)

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
