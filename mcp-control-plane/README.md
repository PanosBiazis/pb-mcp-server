# mcp-control-plane

Starter scaffold for the Netlify “control plane” described in `docs/` (see `docs/architecture.md`).

This can be deployed as a public Netlify site. The recommended setup is:

- **Netlify** = public control plane (auth + allowlisted forwarding)
- **Asus laptop** = runs `pb-gateway` (HTTP) and the real MCP server (stdio) on LAN

## What to deploy first

1. Deploy this Netlify project as-is.
2. Confirm `https://<your-site>.netlify.app/` loads and the status JSON appears.

## Endpoints

`netlify/functions/` contains:

- `status.ts`: proxies to `PB_GATEWAY_URL` if set, otherwise returns mock JSON
- `auth.ts`: mints short-lived **agent tokens** (best-effort) using `AGENT_TOKEN_SECRET`
- `agent-request.ts`: strict allowlisted forwarder to `pb-gateway` endpoints
- `review-packet.ts`: validates a review-packet contract and forwards it to `pb-gateway` for JSONL storage

Per the docs: do not upload the real Rust MCP server to Netlify first. Wire the safe gateway to your real MCP host only after the Netlify side is verified.

## Environment variables (Netlify Functions scope)

- `PB_GATEWAY_BASE_URL`: e.g. `https://your-host` (no trailing path)
- `PB_GATEWAY_URL`: optional convenience for `status.ts` (full URL ending in `/status`)
- `PB_GATEWAY_TOKEN`: optional; sent as `Authorization: Bearer ...` to the gateway
- `PB_GATEWAY_SIGNING_SECRET`: optional but recommended; signs Netlify->gateway requests

### Agent token auth

- `AGENT_TOKEN_SECRET`: required to mint/verify agent tokens
- `AGENT_TOKEN_TTL_MS`: optional (default ~1h)
- `AGENT_JOIN_SECRET`: optional shared secret that must be provided to `/auth`
- `AGENT_POW_DIFFICULTY_BITS`: optional proof-of-work difficulty (0 disables)
- `AGENT_ID_ALLOWLIST`: optional comma-separated allowed ids (e.g. `planner-bot,reviewer-bot`)

Note: you cannot reliably prove “AI agent vs human”; this is a best-effort gate.

## Allowlisted remote ops

`agent-request.ts` only forwards these operations:

- `status`
- `health`
- `security_status`
- `project_info`
- `list_files`
- `read_file`
- `git_status`
- `search_code`
- `search_docs`
- `agent_send`
- `agent_recent`
- `agent_thread`
- `review_packets_list`

### Optional soft rate limit (disabled by default)

- `RATE_LIMIT_ENABLED=1`
- `RATE_LIMIT_WINDOW_MS=1000`
- `RATE_LIMIT_MAX_REQUESTS=1000000` (set extremely high if you enable)

