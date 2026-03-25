export default async function handler() {
  // Stub: later replace with your real auth check + token validation.
  return new Response(
    JSON.stringify({
      ok: false,
      error: "auth stub (not implemented yet)",
    }),
    {
      status: 501,
      headers: { "content-type": "application/json; charset=utf-8" },
    }
  );
}

