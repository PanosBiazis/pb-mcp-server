# mcp-control-plane

Starter scaffold for the Netlify “control plane” described in `docs/` (see `docs/architecture.md`).

This is intentionally minimal: first make a public Netlify site work with a mock `/status` endpoint.

## What to deploy first

1. Deploy this Netlify project as-is.
2. Confirm `https://<your-site>.netlify.app/` loads and the status JSON appears.

## Stub endpoints

`netlify/functions/` contains:

- `status.ts` (mock, returns JSON)
- `auth.ts` (stub)
- `agent-request.ts` (stub)
- `review-packet.ts` (stub)

Per the docs: do not upload the real Rust MCP server to Netlify first. Wire the safe gateway to your real MCP host only after the Netlify side is verified.

