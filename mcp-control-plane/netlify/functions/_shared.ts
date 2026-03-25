type AuthDecision =
  | { ok: true }
  | { ok: false; status: number; error: string };

type AgentAuthDecision =
  | { ok: true; agentId: string }
  | { ok: false; status: number; error: string };

export function json(
  body: unknown,
  init?: { status?: number; headers?: Record<string, string> }
) {
  return new Response(JSON.stringify(body), {
    status: init?.status ?? 200,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
      "x-content-type-options": "nosniff",
      ...(init?.headers ?? {}),
    },
  });
}

export function getEnv(name: string): string | undefined {
  return (globalThis as any).process?.env?.[name] as string | undefined;
}

export function readBearerToken(req: Request): string | undefined {
  const raw = req.headers.get("authorization")?.trim();
  if (!raw) return undefined;
  const m = raw.match(/^\s*Bearer\s+(.+)\s*$/i);
  return (m?.[1] ?? raw).trim();
}

function toBase64Url(bytes: Uint8Array): string {
  // Node-less base64url
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  const b64 = btoa(s);
  return b64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function fromBase64Url(s: string): Uint8Array | null {
  const b64 = s.replace(/-/g, "+").replace(/_/g, "/") + "===".slice((s.length + 3) % 4);
  try {
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  } catch {
    return null;
  }
}

async function hmacSha256(key: string, data: string): Promise<Uint8Array> {
  const enc = new TextEncoder();
  const k = await crypto.subtle.importKey(
    "raw",
    enc.encode(key),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"]
  );
  const sig = await crypto.subtle.sign("HMAC", k, enc.encode(data));
  return new Uint8Array(sig);
}

function toHex(bytes: Uint8Array): string {
  return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function hmacVerifySha256(key: string, data: string, sig: Uint8Array): Promise<boolean> {
  const enc = new TextEncoder();
  const k = await crypto.subtle.importKey(
    "raw",
    enc.encode(key),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["verify"]
  );
  return await crypto.subtle.verify("HMAC", k, sig, enc.encode(data));
}

async function sha256Hex(data: string): Promise<string> {
  const enc = new TextEncoder();
  const h = await crypto.subtle.digest("SHA-256", enc.encode(data));
  const u = new Uint8Array(h);
  return [...u].map((b) => b.toString(16).padStart(2, "0")).join("");
}

function safeJsonParse(s: string): any | null {
  try {
    return JSON.parse(s);
  } catch {
    return null;
  }
}

export async function mintAgentToken(agentId: string): Promise<string | null> {
  const secret = getEnv("AGENT_TOKEN_SECRET")?.trim();
  if (!secret) return null;
  const now = Date.now();
  const ttlMs = Number(getEnv("AGENT_TOKEN_TTL_MS") ?? "3600000");
  const exp = now + (Number.isFinite(ttlMs) && ttlMs > 1000 ? ttlMs : 3600000);
  const payload = { v: 1, agent_id: agentId, iat_ms: now, exp_ms: exp };
  const payloadStr = JSON.stringify(payload);
  const payloadB64 = toBase64Url(new TextEncoder().encode(payloadStr));
  const sig = toBase64Url(await hmacSha256(secret, payloadB64));
  return `${payloadB64}.${sig}`;
}

export async function verifyAgentToken(token: string): Promise<{ ok: true; agentId: string } | { ok: false; error: string }> {
  const secret = getEnv("AGENT_TOKEN_SECRET")?.trim();
  if (!secret) return { ok: false, error: "AGENT_TOKEN_SECRET not configured" };
  const parts = token.split(".");
  if (parts.length !== 2) return { ok: false, error: "invalid token format" };
  const [payloadB64, sigB64] = parts;
  if (!payloadB64 || !sigB64) return { ok: false, error: "invalid token format" };

  const sig = fromBase64Url(sigB64);
  if (!sig) return { ok: false, error: "invalid signature encoding" };
  const okSig = await hmacVerifySha256(secret, payloadB64, sig);
  if (!okSig) return { ok: false, error: "bad signature" };

  const bytes = fromBase64Url(payloadB64);
  if (!bytes) return { ok: false, error: "invalid payload encoding" };
  const payloadStr = new TextDecoder().decode(bytes);
  const payload = safeJsonParse(payloadStr);
  if (!payload || typeof payload !== "object") return { ok: false, error: "invalid payload" };
  if (payload.v !== 1) return { ok: false, error: "unsupported token version" };
  if (typeof payload.agent_id !== "string" || !payload.agent_id.trim()) return { ok: false, error: "missing agent_id" };
  if (typeof payload.exp_ms !== "number" || !Number.isFinite(payload.exp_ms)) return { ok: false, error: "missing exp_ms" };
  if (Date.now() > payload.exp_ms) return { ok: false, error: "token expired" };
  return { ok: true, agentId: payload.agent_id };
}

// "AI-agent only" authentication is not reliably solvable.
// This is a best-effort proof-of-work gate to reduce casual/manual abuse.
export async function verifyPow(agentId: string, nonce: string, solution: string, difficulty: number): Promise<boolean> {
  const d = Math.max(0, Math.min(28, Math.floor(difficulty)));
  if (!nonce || nonce.length < 8) return false;
  if (!solution || solution.length < 8) return false;
  const hex = await sha256Hex(`${nonce}:${agentId}:${solution}`);
  // difficulty as leading hex zeros (4 bits each)
  const zeros = Math.floor(d / 4);
  const rem = d % 4;
  if (!hex.startsWith("0".repeat(zeros))) return false;
  if (rem === 0) return true;
  const nibble = parseInt(hex[zeros] ?? "f", 16);
  return nibble < (1 << (4 - rem));
}

function canonicalSignInput(
  tsMs: string,
  nonce: string,
  method: string,
  pathname: string,
  query: string,
  bodySha256Hex: string
): string {
  return `${tsMs}\n${nonce}\n${method.toUpperCase()}\n${pathname}\n${query}\n${bodySha256Hex}`;
}

export async function gatewaySignedHeaders(
  method: string,
  url: string,
  bodyText: string
): Promise<Record<string, string>> {
  const secret = getEnv("PB_GATEWAY_SIGNING_SECRET")?.trim();
  if (!secret) return {};

  const tsMs = String(Date.now());
  const nonceBytes = crypto.getRandomValues(new Uint8Array(16));
  const nonce = toBase64Url(nonceBytes);
  const u = new URL(url);
  const path = u.pathname || "/";
  const query = u.search.startsWith("?") ? u.search.slice(1) : u.search;
  const bodyDigest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(bodyText)
  );
  const bodySha256Hex = toHex(new Uint8Array(bodyDigest));
  const input = canonicalSignInput(tsMs, nonce, method, path, query, bodySha256Hex);
  const sigHex = toHex(await hmacSha256(secret, input));

  return {
    "x-pb-ts": tsMs,
    "x-pb-nonce": nonce,
    "x-pb-signature": sigHex,
  };
}

// Optional auth:
// - If CONTROL_PLANE_TOKEN is unset: allow everything.
// - If set: require Authorization header with that token.
export function requireControlPlaneAuth(req: Request): AuthDecision {
  const expected = getEnv("CONTROL_PLANE_TOKEN")?.trim();
  if (!expected) return { ok: true };
  const got = readBearerToken(req);
  if (!got) return { ok: false, status: 401, error: "missing authorization token" };
  if (got !== expected) return { ok: false, status: 403, error: "invalid token" };
  return { ok: true };
}

// Agent auth:
// - Requires Authorization: Bearer <agent-token> where token is minted by /auth using AGENT_TOKEN_SECRET.
export async function requireAgentAuth(req: Request): Promise<AgentAuthDecision> {
  const got = readBearerToken(req);
  if (!got) return { ok: false, status: 401, error: "missing agent token" };
  const v = await verifyAgentToken(got);
  if (!v.ok) return { ok: false, status: 403, error: v.error };
  const allowlist = getEnv("AGENT_ID_ALLOWLIST")
    ?.split(",")
    .map((s) => s.trim())
    .filter(Boolean);
  if (allowlist && allowlist.length > 0 && !allowlist.includes(v.agentId)) {
    return { ok: false, status: 403, error: "agent_id not allowed" };
  }
  return { ok: true, agentId: v.agentId };
}

// Optional rate limiter:
// - Disabled unless RATE_LIMIT_ENABLED=1
// - Designed to be "soft" (very high ceiling possible) and per-instance best-effort.
const buckets = new Map<string, { windowStartMs: number; count: number }>();

export function softRateLimit(req: Request): AuthDecision {
  const enabled = getEnv("RATE_LIMIT_ENABLED")?.trim() === "1";
  if (!enabled) return { ok: true };

  const windowMs = Number(getEnv("RATE_LIMIT_WINDOW_MS") ?? "1000");
  const maxReq = Number(getEnv("RATE_LIMIT_MAX_REQUESTS") ?? "1000000");

  if (!Number.isFinite(windowMs) || windowMs <= 0) return { ok: true };
  if (!Number.isFinite(maxReq) || maxReq <= 0) return { ok: true };

  // Key: token if present else IP-ish header (may be absent).
  const token = readBearerToken(req);
  const ip =
    req.headers.get("x-nf-client-connection-ip") ??
    req.headers.get("x-forwarded-for") ??
    "unknown";
  const key = token ? `tok:${token}` : `ip:${ip}`;

  const now = Date.now();
  const b = buckets.get(key);
  if (!b || now - b.windowStartMs >= windowMs) {
    buckets.set(key, { windowStartMs: now, count: 1 });
    return { ok: true };
  }
  b.count += 1;
  if (b.count > maxReq) {
    return { ok: false, status: 429, error: "rate limited (soft limiter)" };
  }
  return { ok: true };
}

