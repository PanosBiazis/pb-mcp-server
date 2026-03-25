import { json, requireControlPlaneAuth, softRateLimit, getEnv, mintAgentToken, verifyPow } from "./_shared";

export default async function handler(req: Request) {
  // Agent authentication endpoint (best-effort).
  //
  // Important: there is no reliable way to prove "AI agent" vs "human".
  // This endpoint implements a practical alternative:
  // - an optional shared join secret (AGENT_JOIN_SECRET)
  // - an optional proof-of-work puzzle (AGENT_POW_DIFFICULTY_BITS)
  // - returns a signed short-lived agent token (AGENT_TOKEN_SECRET)
  //
  // If you want to keep the control plane public, you should still set CONTROL_PLANE_TOKEN
  // for admin-only endpoints, and keep PB_GATEWAY_TOKEN for the gateway itself.
  //
  // Optional soft rate limit (disabled by default; set extremely high if you enable it).
  const rl = softRateLimit(req);
  if (!rl.ok) return json({ ok: false, error: rl.error }, { status: rl.status });

  // If CONTROL_PLANE_TOKEN is set, require it even here (keeps the endpoint private).
  // If you want agents to self-serve, leave CONTROL_PLANE_TOKEN unset and rely on AGENT_JOIN_SECRET + PoW.
  const admin = requireControlPlaneAuth(req);
  if (!admin.ok) return json({ ok: false, error: admin.error }, { status: admin.status });

  if (req.method !== "POST") {
    return json(
      {
        ok: false,
        error: "method not allowed",
        hint: "POST JSON: { agent_id, join_secret?, nonce?, solution? }",
      },
      { status: 405 }
    );
  }

  let body: any;
  try {
    body = await req.json();
  } catch {
    return json({ ok: false, error: "invalid JSON" }, { status: 400 });
  }

  const agentId = String(body?.agent_id ?? "").trim();
  if (!agentId) return json({ ok: false, error: "agent_id must be a non-empty string" }, { status: 400 });
  if (agentId.length > 80) return json({ ok: false, error: "agent_id too long" }, { status: 400 });

  const joinSecret = getEnv("AGENT_JOIN_SECRET")?.trim();
  if (joinSecret) {
    const got = String(body?.join_secret ?? "").trim();
    if (!got) return json({ ok: false, error: "missing join_secret" }, { status: 401 });
    if (got !== joinSecret) return json({ ok: false, error: "invalid join_secret" }, { status: 403 });
  }

  const powBits = Number(getEnv("AGENT_POW_DIFFICULTY_BITS") ?? "0");
  if (Number.isFinite(powBits) && powBits > 0) {
    const nonce = String(body?.nonce ?? "").trim();
    const solution = String(body?.solution ?? "").trim();
    if (!nonce || !solution) {
      return json(
        {
          ok: false,
          error: "missing nonce/solution for PoW",
          difficulty_bits: powBits,
          example_hash: "sha256(`${nonce}:${agent_id}:${solution}`) must have leading difficulty_bits zeros",
        },
        { status: 400 }
      );
    }
    const ok = await verifyPow(agentId, nonce, solution, powBits);
    if (!ok) return json({ ok: false, error: "PoW failed" }, { status: 403 });
  }

  const tok = await mintAgentToken(agentId);
  if (!tok) {
    return json(
      {
        ok: false,
        error: "AGENT_TOKEN_SECRET not configured",
        hint: "Set AGENT_TOKEN_SECRET (and optionally AGENT_TOKEN_TTL_MS).",
      },
      { status: 500 }
    );
  }

  return json({
    ok: true,
    service: "mcp-control-plane",
    agent_id: agentId,
    token_type: "Bearer",
    access_token: tok,
    expires_hint_ms: Number(getEnv("AGENT_TOKEN_TTL_MS") ?? "3600000"),
    ts: Date.now(),
  });
}

