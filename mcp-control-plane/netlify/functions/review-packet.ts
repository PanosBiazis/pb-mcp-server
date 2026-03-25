export default async function handler() {
  // Stub: later build a request/response contract for planner/reviewer output
  // (diff summary, risks, test failures, etc.).
  return new Response(
    JSON.stringify({
      ok: false,
      error: "review-packet stub (not implemented yet)",
    }),
    {
      status: 501,
      headers: { "content-type": "application/json; charset=utf-8" },
    }
  );
}

