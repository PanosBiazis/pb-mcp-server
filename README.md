# pb-mcp-server

Rust MCP server for Zed (stdio) plus a durable multi-agent JSONL bus and a tiny optional HTTP gateway.

## What’s in this repo

- **`pb-mcp-server` (Rust MCP / stdio)**: the main binary Zed connects to. Exposes project-scoped read-only file/git/search tools plus multi-agent bus tools (`agent_send`, `agent_inbox`, `agent_thread`, `agent_recent`, …) and an orchestration tool `ask_agents`.
- **Durable bus**: append-only JSONL at `.mint-mcp/agent-bus.jsonl` (already ignored by git via `.gitignore`).
- **`pb-gateway` (HTTP, optional)**: minimal LAN/remote gateway with `/health` and `/status` (intended for Netlify control-plane proxying).
- **`mcp-control-plane/` (Netlify)**: a tiny public dashboard + functions (status proxy implemented; other endpoints are stubs).

## Build

```bash
cargo build --release
```

## Run in Zed (stdio MCP)

Zed should point to the built binary (example `setup.json` exists in the repo).

Environment variables:

- **`MINT_PROJECT_ROOT` / `PROJECT_ROOT`**: limits all file paths to stay inside this root (recommended).

## Multi-agent “messenger” flow

All agents share a durable bus file under the project root:

- Send a message: `agent_send(from_agent, message, to_agent?, topic?, kind?)`
- Poll inbox: `agent_inbox(for_agent)`
- Read a thread: `agent_thread(topic)`
- Quick global tail: `agent_recent`
- Orchestrate: `ask_agents(query, topic?, timeout_ms?)` (broadcasts and waits for replies on the same topic)

## Optional HTTP gateway (`pb-gateway`)

This is **not** an MCP transport; it’s a small HTTP helper for a “control plane” to query your server.

Run it:

```bash
PB_GATEWAY_BIND=127.0.0.1:8080 \
PB_GATEWAY_TOKEN=change-me \
MINT_PROJECT_ROOT=/path/to/project/root \
./target/release/pb-gateway
```

Safer defaults:

- Keep **`PB_GATEWAY_BIND=127.0.0.1:8080`** (localhost only) and put it behind your VPN/reverse-proxy.
- Always set **`PB_GATEWAY_TOKEN`** if the gateway is reachable by anything except localhost.
- Always set **`MINT_PROJECT_ROOT`** to the single repo you want the gateway to touch.

Endpoints:

- `GET /health` (no auth)
- `GET /status` (requires `Authorization: Bearer <PB_GATEWAY_TOKEN>` if token is set)
- `POST /review-packet` (stores one JSON object as JSONL; requires auth if token is set)
- `GET /review-packets?limit=30` (returns tail window; requires auth if token is set)

## Netlify control-plane (`mcp-control-plane/`)

`mcp-control-plane/netlify/functions/status.ts` proxies to `PB_GATEWAY_URL` when configured:

- **`PB_GATEWAY_URL`**: e.g. `https://your-host/status`
- **`PB_GATEWAY_TOKEN`**: optional; sent as `Authorization: Bearer …`

## Asus deployment notes

See `deploy/systemd/` for a `systemd` unit example and hardened defaults:

- `deploy/systemd/pb-gateway.service`
- `deploy/systemd/pb-gateway.env.example`
