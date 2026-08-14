# pb-mcp-server — Architecture Reference

> **Version:** 0.2.1  
> **Status:** Production-ready (see [CHANGELOG.md](../CHANGELOG.md))

This document describes the system architecture, trust model, deployment
topology, multi-agent message flow, and security design for `pb-mcp-server`.
For the full tool catalogue and build instructions, see [info.md](info.md).

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Component Map](#component-map)
3. [Trust Zones](#trust-zones)
4. [Deployment Topology](#deployment-topology)
5. [Request Flows](#request-flows)
6. [Multi-Agent Message Flow](#multi-agent-message-flow)
7. [Security Design](#security-design)
8. [Netlify Control Plane](#netlify-control-plane)
9. [pb-gateway (Local Host)](#pb-gateway-local-host)
10. [Environment Variables](#environment-variables)
11. [Build Phases](#build-phases)

---

## System Overview

`pb-mcp-server` is a Rust-native MCP (Model Context Protocol) server that
exposes **500+ tools** to AI agents running inside Zed, Cursor, and other
MCP-capable editors. The system is composed of:

- A **local stdio MCP server** (`pb-mcp-server`) for full-trust in-editor use
- An **HTTP gateway** (`pb-gateway`) for network-facing access with auth and
  rate limiting
- A **Netlify control plane** (`mcp-control-plane/`) as the public edge layer
- Supporting crates for LSP bridging, P2P networking, and an admin dashboard

The design follows a **local-first** model: the most powerful tools stay on
the developer's machine; only a minimal, allowlisted subset is ever exposed
to the internet.

---

## Component Map

```
┌──────────────────────────────────────────────────────────────────┐
│                     pb-mcp-server workspace                      │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  pb-mcp-server (root crate)                                │  │
│  │                                                            │  │
│  │  ┌──────────────────────────────────────────────────────┐  │  │
│  │  │  Agent Broker  (broadcast bus, consensus, handoffs)  │  │  │
│  │  └────────────────────────────────────────────────────┬─┘  │  │
│  │          │             │             │                 │    │  │
│  │       Agent A       Agent B       Agent C    …  consensus  │  │
│  │       (Claude)      (GPT-4)       (Ollama)               │  │  │
│  │                                                          │  │  │
│  │  ┌──────────────────────────────────────────────────────┘  │  │
│  │  │  Tool layer — src/tools/ args, handlers, catalog,       │  │
│  │  │  registry/profile gates                                 │  │
│  │  └────────────────────────────────────────────────────┬─┘  │  │
│  │          │                                             │    │
│  │          ▼                                             │    │
│  │  ┌──────────────────────────────────────────────────────┐  │
│  │  │  28 ops modules — src/ops/*.rs  (work functions)     │  │
│  │  └──────────────────────────────────────────────────────┘  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌──────────────┐  ┌───────────────┐  ┌─────────────────────┐   │
│  │ pb-gateway   │  │ pb-lsp-bridge │  │  pb-network-node    │   │
│  │ auth/routing │  │ language svcs │  │  libp2p DHT/Gossip  │   │
│  └──────────────┘  └───────────────┘  └─────────────────────┘   │
│                                                                  │
│  ┌──────────────┐                                                │
│  │ pb-dashboard │  (egui admin UI — system status + metrics)     │
│  └──────────────┘                                                │
└──────────────────────────────────────────────────────────────────┘
```

### Workspace crates

| Crate | Path | Purpose |
|---|---|---|
| **pb-mcp-server** | `/` (root) | Main MCP server — hosts all 500+ tools |
| **pb-gateway** | `crates/pb-gateway/` | HTTP gateway: token auth, rate limiting, proxying |
| **pb-lsp-bridge** | `crates/pb-lsp-bridge/` | Bridges language-server protocols into MCP surface |
| **pb-network-node** | `crates/pb-network-node/` | libp2p swarm: Kademlia DHT, Gossipsub, peer discovery |
| **pb-dashboard** | `crates/pb-dashboard/` | Admin dashboard: status, tool metrics, agent monitoring |

---

## Trust Zones

The system is divided into three explicit trust zones. No tool or endpoint
should ever be promoted to a higher-trust zone without review.

| Zone | Who | Tools / Surface | Auth |
|---|---|---|---|
| **Local / LAN** | Developer, local Zed session | Full 500+ tool set | None (stdio, implicit) |
| **Remote-Agent** | Approved AI agents over internet | ~14 allowlisted read tools | Bearer token + HMAC |
| **Admin / Secrets** | Machine owner only | `pb-dashboard`, secrets, keys | Local access only |

### What goes where

**Local/LAN (high trust)**
- All file operations, shell execution, test runners, GPU inference, OCR, etc.
- The full stdio MCP server is the only interface at this level.
- Never expose raw stdio to the network.

**Remote-Agent zone (low trust)**
- `status`, `health`, `project_info`, `list_files`, `read_file`, `git_status`
- `search_code`, `search_docs`, `agent_send`, `agent_recent`, `agent_thread`
- `review_packets_list`
- Enforced in `agent-request.ts` (Netlify) and `pb-gateway` allowlists.

**Admin / Secrets (never remote)**
- `PB_GATEWAY_TOKEN`, `PB_NODE_SECRET`, `AGENT_TOKEN_SECRET`, TLS keys
- Stored only in environment variables on the host machine, never in source.

---

## Deployment Topology

```
 ┌─────────────────────────┐
 │  Browser / AI Agent     │  (anywhere on the internet)
 └────────────┬────────────┘
              │  HTTPS
              ▼
 ┌─────────────────────────────────────────────┐
 │  pb-mcp-server.netlify.app                  │
 │  (Netlify control plane)                    │
 │                                             │
 │  /.netlify/functions/auth                   │
 │  /.netlify/functions/status                 │
 │  /.netlify/functions/agent-request  ←──┐    │
 │  /.netlify/functions/review-packet      │   │
 └─────────────────┬───────────────────────┘   │
                   │ Bearer token + HMAC        │
                   │ (PB_GATEWAY_BASE_URL)      │
                   ▼                            │
 ┌─────────────────────────────┐                │
 │  pb-gateway (local host)    │  127.0.0.1:8080 │
 │                             │                │
 │  GET  /health               │                │
 │  GET  /status               │                │
 │  POST /review-packet        │                │
 │  GET  /review-packets       │                │
 └─────────────────┬───────────┘                │
                   │                            │
                   ▼                            │
 ┌─────────────────────────────┐                │
 │  pb-mcp-server (local)      │  stdio          │
 │  (full tool set)            │  (Zed/Cursor)   │
 └─────────────────────────────┘                │
                   │                            │
              Zed / Cursor  ─────────────────────┘
              (LAN direct)
```

The pattern is:
- **Netlify** = public edge (auth, allowlist enforcement, rate limiting)
- **pb-gateway** = trusted execution gateway on the local host or private server
- **pb-mcp-server** = local stdio server for your own editor sessions

---

## Request Flows

### Local Zed session (full trust)

```
Developer in Zed
    │
    │  MCP protocol (stdio)
    ▼
pb-mcp-server process (full 500+ tools available)
    │
    ├─ src/tools/*.rs  (MCP args, handlers, catalog, registry/profile gates)
    ├─ src/ops/*.rs    (tool work functions)
    └─ src/agent/    (bus, consensus, live workers)
```

No network involved. All tools available. No extra auth needed — the developer
is the operator of the process.

### Remote AI agent (restricted)

```
AI agent (e.g. Claude, GPT-4)
    │
    │  HTTPS POST  /.netlify/functions/agent-request
    │  { "op": "search_code", "args": { ... }, "token": "..." }
    ▼
Netlify Function (agent-request.ts)
    │  1. Verify agent token (AGENT_TOKEN_SECRET)
    │  2. Check op is in allowlist
    │  3. Sign request with PB_GATEWAY_SIGNING_SECRET
    ▼
pb-gateway  (127.0.0.1:8080 on local host)
    │  1. Verify HMAC signature on incoming request
    │  2. Rate-limit by agent-id
    │  3. Dispatch to appropriate handler
    ▼
Local tool / project files (read-only subset)
```

---

## Multi-Agent Message Flow

The agent bus (`src/agent/bus.rs`) is a durable append-only JSONL file under
`.mint-mcp/agent-bus.jsonl`, protected by file-level locking (`flock`). Any
agent process can write to or read from it.

### Message lifecycle

```
Agent A                    Bus (JSONL file)              Agent B
  │                              │                          │
  │── agent_send(topic=T) ──────►│                          │
  │   { id, from, to, kind,      │                          │
  │     nonce, ts, body }        │                          │
  │   HMAC-SHA256 signed         │                          │
  │                              │◄─── agent_inbox(topic=T) ┤
  │                              │──── [messages] ─────────►│
  │                              │                          │
  │◄─ agent_recent(topic=T) ─────┤                          │
  │                              │                          │
```

### ask_agents orchestration

1. Orchestrator calls `ask_agents(topic=T, question=Q, timeout_ms=N)`
2. Server broadcasts `Q` to the bus under topic `T`
3. Other agents (separate processes or editor sessions) call `agent_inbox` and
   respond via `agent_send` with the same topic
4. After `timeout_ms`, the orchestrator collects all replies and returns them

### Consensus protocol

For decisions that require agreement, the consensus subsystem
(`src/agent/consensus.rs`) supports six strategies:

| Strategy | Description |
|---|---|
| `SimpleMajority` | >50% of votes decide |
| `SuperMajority` | Configurable threshold (e.g. 66%, 75%) |
| `Unanimous` | All participating agents must agree |
| `WeightedVoting` | Votes weighted by agent role/trust |
| `RankedChoice` | Instant-runoff voting across proposals |
| `Delegation` | Agent delegates vote to a trusted peer |

### Agent roles (recommended for orchestration)

| Role | Responsibility |
|---|---|
| **Planner** | Converts feature requests into tasks with acceptance criteria |
| **Retriever** | Gathers code context, docs, and prior decisions via MCP tools |
| **Coder** | Proposes implementation patches |
| **Tester** | Runs builds, tests, linting, and smoke checks |
| **Security Reviewer** | Checks scope use, secrets, unsafe commands, auth boundaries |
| **Gatekeeper** | Refuses merge/apply unless tests, policy, and approval pass |

---

## Security Design

The system applies **five layers of defense**:

### Layer 1 — Input validation

- All tool arguments validated at entry with length limits
- Shell metacharacter blocking (`; & | $ ( ) { } < > \` ' "`)
- Command allowlists: only approved executables can be invoked
- Path arguments checked against a project-root sandbox before use

### Layer 2 — Path sandboxing

- `verify_under_root()` uses `Path::starts_with()` (not string prefix)
- Symlink chains resolved and depth-limited (max 8 hops)
- DANGEROUS_EXTENSIONS blocklist prevents loading executable/sensitive file types
- All file operations are confined to the configured project root

### Layer 3 — Message signing (inter-node)

All messages on the `pb-network-node` P2P layer are signed with HMAC-SHA256.
The signed payload includes every field that could affect routing or semantics:

```
HMAC-SHA256(key=PB_NODE_SECRET, msg=
  id || from || to || kind || nonce || ts || body
)
```

- `to_node` and `kind` are included in the signed payload — a MITM cannot
  redirect a message without breaking the signature.
- `nonce` is unique per message (PID + timestamp + monotonic counter), making
  exact-replay attacks detectable.
- Signature verification uses **constant-time XOR comparison** to prevent
  timing-based forgery attacks.

### Layer 4 — Token auth (gateway)

- `pb-gateway` requires `Authorization: Bearer <PB_GATEWAY_TOKEN>` on all
  non-health endpoints.
- Netlify functions sign outgoing requests to the gateway with
  `PB_GATEWAY_SIGNING_SECRET` using HMAC.
- `warn_if_weak_secret()` emits a startup warning if `PB_NODE_SECRET` is
  empty or fewer than 32 characters.

### Layer 5 — Rate limiting

- Bus operations are rate-limited per agent-id with burst prevention.
- `pb-gateway` enforces per-IP and per-agent rate limits.
- Netlify's edge CDN provides additional DDoS mitigation for the public surface.

### Compile-time safety

```rust
#![deny(unsafe_code)]  // in lib.rs — no unsafe except one gated exception
```

- One controlled `unsafe` block exists behind `perf-mmap` feature flag
  (for memory-mapped file reads) with explicit safety documentation.
- `[lints.rust]` and `[lints.clippy]` sections enforce clean code at all times.

---

## Netlify Control Plane

The `mcp-control-plane/` directory contains the public Netlify project.
Deploy it by connecting the repository to Netlify or using drag-and-drop.

### Functions

| Function | Route | Purpose |
|---|---|---|
| `status.ts` | `/.netlify/functions/status` | Authenticated gateway proxy; development-only mock |
| `auth.ts` | `/.netlify/functions/auth` | Mints short-lived agent tokens |
| `agent-request.ts` | `/.netlify/functions/agent-request` | Allowlisted forwarder to pb-gateway |
| `review-packet.ts` | `/.netlify/functions/review-packet` | Validates and stores review packets |

### Allowlisted operations (agent-request.ts)

Only these operations can be forwarded by `agent-request.ts`:

```
status, health, security_status, project_info,
list_files, read_file, git_status,
search_code, search_docs,
agent_send, agent_recent, agent_thread,
review_packets_list
```

All other operations are rejected at the Netlify edge — they never reach the
gateway.

### Netlify environment variables (Functions scope)

| Variable | Purpose |
|---|---|
| `CONTROL_PLANE_TOKEN` | Required production admin bearer token |
| `CONTROL_PLANE_MODE` | Explicit `development` mode only; production is the default |
| `PB_GATEWAY_BASE_URL` | Full base URL of the pb-gateway on the local host or private server |
| `PB_GATEWAY_URL` | Optional: full URL ending in `/status` for `status.ts` |
| `PB_GATEWAY_TOKEN` | Bearer token for gateway auth (keep secret) |
| `PB_GATEWAY_SIGNING_SECRET` | HMAC secret for signing Netlify→gateway requests |
| `AGENT_TOKEN_SECRET` | Required for minting/verifying agent tokens |
| `AGENT_TOKEN_TTL_MS` | Token lifetime in ms (default: 3 600 000 — 1 hour) |
| `AGENT_JOIN_SECRET` | Shared secret required to call `/auth` |
| `AGENT_POW_DIFFICULTY_BITS` | Proof-of-work difficulty (0 disables) |
| `AGENT_ID_ALLOWLIST` | Comma-separated approved agent IDs |
| `RATE_LIMIT_ENABLED` | Production defaults enabled; `0` is explicit opt-out |
| `RATE_LIMIT_WINDOW_MS` | Window (default: 60,000 ms) |
| `RATE_LIMIT_MAX_REQUESTS` | Requests per window (default: 120) |

> **Security note:** never commit real token values. Use Netlify's environment
> variable settings UI to inject secrets at deploy time.

---

## pb-gateway (Local Host)

`pb-gateway` is a small Rust HTTP service that runs alongside the MCP server
on the local host or another private server. It provides the network-facing
endpoint that Netlify forwards to.

### Endpoints

| Method | Route | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | None | Liveness probe |
| `GET` | `/status` | Bearer token | Server status JSON |
| `POST` | `/review-packet` | Bearer token | Append a review packet to JSONL |
| `GET` | `/review-packets` | Bearer token | Tail the most recent packets |

### Gateway environment variables

| Variable | Default | Description |
|---|---|---|
| `PB_GATEWAY_BIND` | `127.0.0.1:8080` | Listen address (keep on localhost) |
| `PB_GATEWAY_TOKEN` | — | Required for protected endpoints |
| `PB_GATEWAY_SIGNING_SECRET` | — | Alternative HMAC request authentication |
| `PB_GATEWAY_TRUST_LOCAL_NETWORK` | `false` | Development-only loopback auth bypass |
| `PB_GATEWAY_EXEC_ENABLE` | `false` | Separate loopback-only `/exec` opt-in |
| `MINT_PROJECT_ROOT` | — | Project root the gateway represents |

### systemd unit

Create `/etc/systemd/system/pb-gateway.service`:

```ini
[Unit]
Description=PB Gateway (status + review packet storage)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=appuser
WorkingDirectory=/opt/app/pb-mcp-server
Environment=MINT_PROJECT_ROOT=/opt/app/projects/your-project
Environment=PB_GATEWAY_BIND=127.0.0.1:8080
Environment=PB_GATEWAY_TOKEN=<rotate-before-use>
ExecStart=/opt/app/pb-mcp-server/target/release/pb-gateway
Restart=always
RestartSec=2

# systemd hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectControlGroups=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
LockPersonality=true
RestrictRealtime=true
RestrictSUIDSGID=true
RestrictNamespaces=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
SystemCallArchitectures=native
SystemCallFilter=@system-service
ReadWritePaths=/opt/app/projects/your-project/.mint-mcp

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pb-gateway
sudo systemctl status pb-gateway
```

### Network exposure

Bind `pb-gateway` to `127.0.0.1` only. Use one of the following options to
make it reachable by Netlify:

| Option | Visibility | Notes |
|---|---|---|
| **Tailscale Serve** | Private (tailnet only) | Best for development and trusted agents |
| **Tailscale Funnel** | Public internet | Enable only after security review; read-only tools first |
| **ngrok / Cloudflare Tunnel** | Public internet | Alternative tunnel options |

Do **not** open a firewall port for the raw gateway address. Always terminate
TLS at the tunnel/proxy and require the bearer token.

---

## Environment Variables

All sensitive values must be provided via environment variables — never
committed to source.

### pb-mcp-server

| Variable | Description |
|---|---|
| `MINT_PROJECT_ROOT` / `PB_PROJECT_ROOT` | Root directory exposed to the server |
| `PB_NODE_SECRET` | HMAC signing key for inter-node messages (≥32 chars) |
| `PB_PROFILE` | Active tool profile (see [info.md](info.md)) |
| `PB_ENABLE_*` | Feature-gated tool enablement flags |

### pb-network-node

| Variable | Description |
|---|---|
| `PB_NODE_SECRET` | HMAC signing secret (same key used for message signing) |
| `PB_LISTEN_ADDR` | libp2p listen address |
| `PB_BOOTSTRAP_PEERS` | Comma-separated multiaddrs for bootstrap peers |

---

## Build Phases

The system was built incrementally. This section records what is complete and
what remains in the roadmap.

### ✅ Complete (v0.2.1)

- [x] Full MCP server with 500+ tools across `src/tools/` handlers/catalog metadata and 28 `src/ops/` work modules
- [x] 5-crate workspace (`pb-mcp-server`, `pb-gateway`, `pb-lsp-bridge`,
      `pb-network-node`, `pb-dashboard`)
- [x] Zero compilation errors, zero code warnings
- [x] 1,292+ unit tests passing (100% pass rate)
- [x] Multi-agent bus (durable JSONL + flock)
- [x] Consensus system (6 strategies)
- [x] libp2p P2P swarm (Kademlia DHT, Gossipsub)
- [x] HMAC-SHA256 message signing (all fields covered)
- [x] Per-message nonce replay protection
- [x] Constant-time auth comparisons
- [x] Path sandboxing + symlink depth limits
- [x] Command allowlists + shell metachar blocking
- [x] Profile-based tool gating (6 profiles)
- [x] Netlify control plane with agent-request allowlist
- [x] pb-gateway with systemd + hardening directives
- [x] `SECURITY.md`, `LICENSE`, complete `README.md`, `CHANGELOG.md`
- [x] GitHub Actions CI (check, test, clippy, fmt, audit)

### ⏳ Roadmap (v0.3.0+)

- [ ] Server-side nonce deduplication (bounded LRU set per peer)
- [ ] GitHub Actions CI/CD release workflow + v0.2.1 tag
- [ ] OpenTelemetry tracing for all tool invocations
- [ ] Caching layer for expensive tools (neural inference, chart generation)
- [ ] Expand test coverage: nonce uniqueness, field malleability
- [ ] Assembly tool support
- [ ] Distributed bus with multi-node consensus
- [ ] Plugin system for custom tools
- [ ] Real-time collaboration features
- [ ] Getting Started guide + Deployment Scenarios docs
- [ ] Docker image with hardened base
- [ ] `cargo audit` in CI for supply-chain scanning

---

*See [info.md](info.md) for the full tool catalogue, feature flags, and
development workflow.*
