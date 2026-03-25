export default async function handler(req: Request) {
  // Proxy to your Asus-hosted gateway when configured.
  //
  // Set in Netlify env vars (Functions scope):
  // - PB_GATEWAY_URL=https://<reachable-host>/status   (or http://127.0.0.1:8080/status for local netlify dev)
  // - PB_GATEWAY_TOKEN=<optional token>               (must match PB_GATEWAY_TOKEN on the gateway)
  //
  // If PB_GATEWAY_URL is missing, we fall back to a deterministic mock response.
  //
  // Note: `agent-request.ts` prefers PB_GATEWAY_BASE_URL (base without path).
  const url = (globalThis as any).process?.env?.PB_GATEWAY_URL as string | undefined;
  const token = (globalThis as any).process?.env?.PB_GATEWAY_TOKEN as string | undefined;

  if (!url) {
    return new Response(
      JSON.stringify({
        ok: true,
        service: "mcp-control-plane",
        mode: "mock",
        message: "Netlify side is working (PB_GATEWAY_URL not set).",
        ts: Date.now(),
      }),
      { headers: { "content-type": "application/json; charset=utf-8" } }
    );
  }

  const headers: Record<string, string> = {
    "accept": "application/json",
  };
  if (token && token.trim()) {
    headers["authorization"] = `Bearer ${token.trim()}`;
  }

  try {
    const upstream = await fetch(url, { method: "GET", headers });
    const text = await upstream.text();
    return new Response(text, {
      status: upstream.status,
      headers: { "content-type": upstream.headers.get("content-type") ?? "application/json; charset=utf-8" },
    });
  } catch (e: any) {
    return new Response(
      JSON.stringify({
        ok: false,
        service: "mcp-control-plane",
        mode: "proxy",
        error: "failed to reach PB_GATEWAY_URL",
        detail: String(e?.message ?? e),
        ts: Date.now(),
      }),
      { status: 502, headers: { "content-type": "application/json; charset=utf-8" } }
    );
  }
}

