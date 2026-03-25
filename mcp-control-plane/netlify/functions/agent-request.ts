import { json, softRateLimit, getEnv, requireAgentAuth } from "./_shared";

type AllowedOp = "status" | "health" | "review_packets_list";

function allowedGatewayUrl(op: AllowedOp): string | undefined {
  const base = getEnv("PB_GATEWAY_BASE_URL")?.trim();
  if (!base) return undefined;
  const b = base.endsWith("/") ? base.slice(0, -1) : base;
  if (op === "status") return `${b}/status`;
  if (op === "health") return `${b}/health`;
  if (op === "review_packets_list") return `${b}/review-packets`;
  return undefined;
}

function gatewayAuthHeader(): string | undefined {
  const tok = getEnv("PB_GATEWAY_TOKEN")?.trim();
  if (!tok) return undefined;
  return `Bearer ${tok}`;
}

export default async function handler(req: Request) {
  // Strict allowlisted forwarder to your Asus `pb-gateway`.
  //
  // Env:
  // - PB_GATEWAY_BASE_URL=https://your-host      (no trailing path; we append /status or /health)
  // - PB_GATEWAY_TOKEN=...                      (optional)
  //
  // Required agent auth:
  // - AGENT_TOKEN_SECRET must be set
  // - Agents obtain tokens by POSTing to /.netlify/functions/auth
  //
  // Optional soft rate limit (disabled by default):
  // - RATE_LIMIT_ENABLED=1
  // - RATE_LIMIT_WINDOW_MS=...
  // - RATE_LIMIT_MAX_REQUESTS=...
  const rl = softRateLimit(req);
  if (!rl.ok) return json({ ok: false, error: rl.error }, { status: rl.status });

  const agent = await requireAgentAuth(req);
  if (!agent.ok) return json({ ok: false, error: agent.error }, { status: agent.status });

  if (req.method !== "POST") {
    return json({ ok: false, error: "method not allowed" }, { status: 405 });
  }

  let body: any;
  try {
    body = await req.json();
  } catch {
    return json({ ok: false, error: "invalid JSON" }, { status: 400 });
  }

  const op = String(body?.op ?? "").trim() as AllowedOp;
  if (op !== "status" && op !== "health" && op !== "review_packets_list") {
    return json(
      {
        ok: false,
        error: "op not allowed",
        allowed_ops: ["status", "health", "review_packets_list"],
      },
      { status: 400 }
    );
  }

  const url = allowedGatewayUrl(op);
  if (!url) {
    return json(
      {
        ok: false,
        error: "PB_GATEWAY_BASE_URL not set",
      },
      { status: 500 }
    );
  }

  const headers: Record<string, string> = { accept: "application/json" };
  const authHeader = gatewayAuthHeader();
  if (authHeader) headers.authorization = authHeader;

  try {
    const upstream = await fetch(url, { method: "GET", headers });
    const text = await upstream.text();
    return new Response(text, {
      status: upstream.status,
      headers: {
        "content-type":
          upstream.headers.get("content-type") ?? "application/json; charset=utf-8",
      },
    });
  } catch (e: any) {
    return json(
      {
        ok: false,
        error: "failed to reach gateway",
        detail: String(e?.message ?? e),
        agent_id: agent.agentId,
      },
      { status: 502 }
    );
  }
}

