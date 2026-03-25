export default async function handler() {
  // Stub: later accept an agent request, validate it, and forward only safe
  // calls to your Rust MCP server (do not proxy raw “unsafe” tools).
  return new Response(
    JSON.stringify({
      ok: false,
      error: "agent-request stub (not implemented yet)",
    }),
    {
      status: 501,
      headers: { "content-type": "application/json; charset=utf-8" },
    }
  );
}

