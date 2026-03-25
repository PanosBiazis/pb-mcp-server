import { json, softRateLimit, getEnv, requireAgentAuth, gatewaySignedHeaders } from "./_shared";

type AllowedOp =
  | "status"
  | "health"
  | "security_status"
  | "project_info"
  | "list_files"
  | "read_file"
  | "git_status"
  | "agent_send"
  | "agent_recent"
  | "agent_thread"
  | "review_packets_list"
  | "search_code"
  | "search_docs";

function allowedGatewayUrl(op: AllowedOp): string | undefined {
  const base = getEnv("PB_GATEWAY_BASE_URL")?.trim();
  if (!base) return undefined;
  const b = base.endsWith("/") ? base.slice(0, -1) : base;
  if (op === "status") return `${b}/status`;
  if (op === "health") return `${b}/health`;
  if (op === "security_status") return `${b}/security-status`;
  if (op === "project_info") return `${b}/project-info`;
  if (op === "list_files") return `${b}/list-files`;
  if (op === "read_file") return `${b}/read-file`;
  if (op === "git_status") return `${b}/git-status`;
  if (op === "agent_send") return `${b}/agent-send`;
  if (op === "agent_recent") return `${b}/agent-recent`;
  if (op === "agent_thread") return `${b}/agent-thread`;
  if (op === "review_packets_list") return `${b}/review-packets`;
  if (op === "search_code") return `${b}/search-code`;
  if (op === "search_docs") return `${b}/search-docs`;
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
  if (
    op !== "status" &&
    op !== "health" &&
    op !== "security_status" &&
    op !== "project_info" &&
    op !== "list_files" &&
    op !== "read_file" &&
    op !== "git_status" &&
    op !== "agent_send" &&
    op !== "agent_recent" &&
    op !== "agent_thread" &&
    op !== "review_packets_list" &&
    op !== "search_code" &&
    op !== "search_docs"
  ) {
    return json(
      {
        ok: false,
        error: "op not allowed",
        allowed_ops: [
          "status",
          "health",
          "security_status",
          "project_info",
          "list_files",
          "read_file",
          "git_status",
          "agent_send",
          "agent_recent",
          "agent_thread",
          "review_packets_list",
          "search_code",
          "search_docs",
        ],
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

  const qp = new URLSearchParams();
  const args = (body?.args && typeof body.args === "object") ? body.args : {};
  if (op === "review_packets_list") {
    if (args.limit !== undefined) qp.set("limit", String(args.limit));
  }
  if (op === "list_files" || op === "git_status") {
    if (args.path !== undefined) qp.set("path", String(args.path));
  }
  if (op === "read_file") {
    const path = String(args.path ?? "").trim();
    if (!path) return json({ ok: false, error: "args.path is required" }, { status: 400 });
    qp.set("path", path);
    if (args.start_line !== undefined) qp.set("start_line", String(args.start_line));
    if (args.end_line !== undefined) qp.set("end_line", String(args.end_line));
  }
  if (op === "agent_recent") {
    if (args.limit !== undefined) qp.set("limit", String(args.limit));
  }
  if (op === "agent_thread") {
    const topic = String(args.topic ?? "").trim();
    if (!topic) return json({ ok: false, error: "args.topic is required" }, { status: 400 });
    qp.set("topic", topic);
    if (args.limit !== undefined) qp.set("limit", String(args.limit));
  }
  if (op === "search_code" || op === "search_docs") {
    const pattern = String(args.pattern ?? "").trim();
    if (!pattern) return json({ ok: false, error: "args.pattern is required" }, { status: 400 });
    qp.set("pattern", pattern);
    if (args.path !== undefined) qp.set("path", String(args.path));
    if (args.literal !== undefined) qp.set("literal", String(Boolean(args.literal)));
    if (args.case_sensitive !== undefined) qp.set("case_sensitive", String(Boolean(args.case_sensitive)));
    if (args.max_matches !== undefined) qp.set("max_matches", String(args.max_matches));
  }
  const finalUrl = qp.toString() ? `${url}?${qp.toString()}` : url;

  try {
    const isPost = op === "agent_send";
    const bodyText = isPost
      ? JSON.stringify({
          from_agent: String(args.from_agent ?? "").trim(),
          message: String(args.message ?? "").trim(),
          to_agent: args.to_agent ?? null,
          topic: args.topic ?? null,
          kind: args.kind ?? null,
        })
      : "";
    if (isPost) {
      const parsed = JSON.parse(bodyText);
      if (!parsed.from_agent || !parsed.message) {
        return json({ ok: false, error: "agent_send requires args.from_agent and args.message" }, { status: 400 });
      }
      headers["content-type"] = "application/json; charset=utf-8";
    }

    const sig = await gatewaySignedHeaders(isPost ? "POST" : "GET", finalUrl, bodyText);
    Object.assign(headers, sig);
    const upstream = await fetch(finalUrl, {
      method: isPost ? "POST" : "GET",
      headers,
      body: isPost ? bodyText : undefined,
    });
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

