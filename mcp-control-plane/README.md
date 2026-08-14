# mcp-control-plane

Fail-closed Netlify control plane described in `docs/architecture.md`.

This can be deployed as a public Netlify site. The recommended setup is:

- **Netlify** = public control plane (auth + allowlisted forwarding)
- **Asus laptop** = runs `pb-gateway` (HTTP) and the real MCP server (stdio) on LAN

## What to deploy first

1. Configure the required production secrets and gateway URL below.
2. Run `npm ci && npm test && npm run typecheck`.
3. Deploy and confirm authenticated status forwarding.

## Endpoints

`netlify/functions/` contains:

- `status.ts`: authenticated proxy to `PB_GATEWAY_URL`; mock mode exists only
  with `CONTROL_PLANE_MODE=development`
- `auth.ts`: mints short-lived **agent tokens** (best-effort) using `AGENT_TOKEN_SECRET`
- `agent-request.ts`: strict allowlisted forwarder to `pb-gateway` endpoints
- `review-packet.ts`: validates a review-packet contract and forwards it to `pb-gateway` for JSONL storage

Per the docs: do not upload the real Rust MCP server to Netlify first. Wire the safe gateway to your real MCP host only after the Netlify side is verified.

## Environment variables (Netlify Functions scope)

- `PB_GATEWAY_BASE_URL`: e.g. `https://your-host` (no trailing path)
- `PB_GATEWAY_URL`: optional convenience for `status.ts` (full URL ending in `/status`)
- `CONTROL_PLANE_TOKEN`: required production admin token
- `CONTROL_PLANE_MODE`: set to `development` only for explicit local/demo mode
- `PB_GATEWAY_TOKEN`: bearer token sent to the gateway
- `PB_GATEWAY_SIGNING_SECRET`: optional but recommended; signs Netlify->gateway requests

### Agent token auth

- `AGENT_TOKEN_SECRET`: required to mint/verify agent tokens
- `AGENT_TOKEN_TTL_MS`: optional (default ~1h)
- `AGENT_JOIN_SECRET`: enrollment secret; production requires this or PoW
- `AGENT_POW_DIFFICULTY_BITS`: alternative enrollment proof-of-work difficulty
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

### Rate limit

- Production defaults to enabled at 120 requests per 60-second window.
- `RATE_LIMIT_ENABLED=0` is honored only with
  `CONTROL_PLANE_MODE=development`; production cannot disable the limiter.
- `RATE_LIMIT_WINDOW_MS` must be between 1 second and 1 hour.
- `RATE_LIMIT_MAX_REQUESTS` must be between 1 and 10,000.

The in-memory limiter is per Netlify instance and remains defense in depth; use
Netlify edge controls for distributed enforcement.
