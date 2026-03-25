export default async function handler(req: Request) {
  // Proxy to your Asus-hosted gateway when configured.
  //
  // Set in Netlify env vars (Functions scope):
  // - PB_GATEWAY_URL=https://<reachable-host>/status   (full status URL)
  // - PB_GATEWAY_BASE_URL=https://<reachable-host>     (base URL; /status will be appended)
  // - PB_GATEWAY_TOKEN=<optional token>               (must match PB_GATEWAY_TOKEN on the gateway)
  //
  // If neither PB_GATEWAY_URL nor PB_GATEWAY_BASE_URL is set, we fall back to mock mode.
  //
  const full = (globalThis as any).process?.env?.PB_GATEWAY_URL as string | undefined;
  const base = (globalThis as any).process?.env?.PB_GATEWAY_BASE_URL as string | undefined;
  const token = (globalThis as any).process?.env?.PB_GATEWAY_TOKEN as string | undefined;
  const normBase = base?.trim().replace(/\/+$/, "");
  const url = (full?.trim() || (normBase ? `${normBase}/status` : undefined));

  if (!url) {
    return new Response(
      JSON.stringify({
        ok: true,
        service: "mcp-control-plane",
        mode: "mock",
        message: "Netlify side is working (PB_GATEWAY_URL / PB_GATEWAY_BASE_URL not set).",
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
        error: "failed to reach configured gateway status URL",
        url,
        detail: String(e?.message ?? e),
        ts: Date.now(),
      }),
      { status: 502, headers: { "content-type": "application/json; charset=utf-8" } }
    );
  }
}

