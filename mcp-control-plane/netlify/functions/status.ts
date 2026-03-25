export default async function handler() {
  // Minimal mock endpoint: first milestone should verify the Netlify
  // deployment + routing works, before wiring it to your Rust MCP host.
  return new Response(
    JSON.stringify({
      ok: true,
      service: "mcp-control-plane",
      message: "Netlify side is working (mock status).",
      ts: Date.now(),
    }),
    {
      headers: { "content-type": "application/json; charset=utf-8" },
    }
  );
}

