# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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