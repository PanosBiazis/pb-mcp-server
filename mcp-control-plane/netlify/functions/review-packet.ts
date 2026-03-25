import { json, softRateLimit, getEnv, requireAgentAuth } from "./_shared";

type ReviewPacket = {
  schema_version: 1;
  project: string;
  task_id: string;
  created_at_ms: number;
  agent: { id: string; kind?: string };
  summary: string;
  design_choices?: string[];
  changed_files?: string[];
  test_results?: string;
  risks?: string[];
  open_questions?: string[];
  links?: string[];
};

function validatePacket(p: any): { ok: true; value: ReviewPacket } | { ok: false; error: string } {
  if (!p || typeof p !== "object" || Array.isArray(p)) return { ok: false, error: "packet must be an object" };
  if (p.schema_version !== 1) return { ok: false, error: "schema_version must be 1" };
  for (const k of ["project", "task_id", "summary"] as const) {
    if (typeof p[k] !== "string" || !p[k].trim()) return { ok: false, error: `${k} must be a non-empty string` };
  }
  if (typeof p.created_at_ms !== "number" || !Number.isFinite(p.created_at_ms)) {
    return { ok: false, error: "created_at_ms must be a number" };
  }
  if (!p.agent || typeof p.agent !== "object") return { ok: false, error: "agent must be an object" };
  if (typeof p.agent.id !== "string" || !p.agent.id.trim()) return { ok: false, error: "agent.id must be a non-empty string" };

  const arrFields = ["design_choices", "changed_files", "risks", "open_questions", "links"] as const;
  for (const f of arrFields) {
    if (p[f] === undefined) continue;
    if (!Array.isArray(p[f]) || p[f].some((x: any) => typeof x !== "string")) return { ok: false, error: `${f} must be string[]` };
  }
  if (p.test_results !== undefined && typeof p.test_results !== "string") return { ok: false, error: "test_results must be a string" };

  // Basic size guards (not "usage limits", just payload sanity).
  const maxSummary = Number(getEnv("REVIEW_PACKET_MAX_SUMMARY_CHARS") ?? "20000");
  if (p.summary.length > maxSummary) return { ok: false, error: `summary too long (>${maxSummary})` };

  return { ok: true, value: p as ReviewPacket };
}

export default async function handler(req: Request) {
  // Store a planner/tester/security "review packet" by forwarding to the Asus `pb-gateway`.
  //
  // Env:
  // - PB_GATEWAY_BASE_URL=https://your-host
  // - PB_GATEWAY_TOKEN=... (optional, but recommended)
  //
  // Required agent auth:
  // - AGENT_TOKEN_SECRET must be set
  // - Agents obtain tokens via /.netlify/functions/auth
  //
  // Optional soft rate limit (disabled by default):
  // - RATE_LIMIT_ENABLED=1 ...
  const rl = softRateLimit(req);
  if (!rl.ok) return json({ ok: false, error: rl.error }, { status: rl.status });

  const agent = await requireAgentAuth(req);
  if (!agent.ok) return json({ ok: false, error: agent.error }, { status: agent.status });

  if (req.method !== "POST") return json({ ok: false, error: "method not allowed" }, { status: 405 });

  let packetRaw: any;
  try {
    packetRaw = await req.json();
  } catch {
    return json({ ok: false, error: "invalid JSON" }, { status: 400 });
  }

  const v = validatePacket(packetRaw);
  if (!v.ok) return json({ ok: false, error: v.error }, { status: 400 });

  // Ensure agent id is consistent between token and payload (simple integrity guard).
  if (v.value.agent?.id && v.value.agent.id !== agent.agentId) {
    return json({ ok: false, error: "agent.id must match bearer token agent_id" }, { status: 403 });
  }

  const base = getEnv("PB_GATEWAY_BASE_URL")?.trim();
  if (!base) return json({ ok: false, error: "PB_GATEWAY_BASE_URL not set" }, { status: 500 });
  const b = base.endsWith("/") ? base.slice(0, -1) : base;
  const url = `${b}/review-packet`;

  const token = getEnv("PB_GATEWAY_TOKEN")?.trim();
  const headers: Record<string, string> = {
    "content-type": "application/json; charset=utf-8",
    accept: "application/json",
  };
  if (token) headers.authorization = `Bearer ${token}`;

  try {
    const upstream = await fetch(url, { method: "POST", headers, body: JSON.stringify(v.value) });
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
      { ok: false, error: "failed to reach gateway", detail: String(e?.message ?? e) },
      { status: 502 }
    );
  }
}

