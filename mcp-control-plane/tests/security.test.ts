import assert from "node:assert/strict";
import { webcrypto } from "node:crypto";
import test from "node:test";

if (!globalThis.crypto) {
  Object.defineProperty(globalThis, "crypto", { value: webcrypto });
}

import authHandler from "../netlify/functions/auth";
import statusHandler from "../netlify/functions/status";
import {
  requireControlPlaneAuth,
  softRateLimit,
} from "../netlify/functions/_shared";

const MANAGED_ENV = [
  "CONTROL_PLANE_MODE",
  "CONTROL_PLANE_TOKEN",
  "AGENT_TOKEN_SECRET",
  "AGENT_JOIN_SECRET",
  "AGENT_POW_DIFFICULTY_BITS",
  "RATE_LIMIT_ENABLED",
  "RATE_LIMIT_WINDOW_MS",
  "RATE_LIMIT_MAX_REQUESTS",
  "PB_GATEWAY_URL",
  "PB_GATEWAY_BASE_URL",
] as const;

function cleanEnv() {
  for (const key of MANAGED_ENV) delete process.env[key];
}

test("production control-plane auth fails closed without a token", () => {
  cleanEnv();
  const decision = requireControlPlaneAuth(new Request("https://unit.invalid/status"));
  assert.deepEqual(decision, {
    ok: false,
    status: 503,
    error: "CONTROL_PLANE_TOKEN not configured",
  });
});

test("development mode is an explicit unauthenticated opt-in", () => {
  cleanEnv();
  process.env.CONTROL_PLANE_MODE = "development";
  assert.deepEqual(
    requireControlPlaneAuth(new Request("https://unit.invalid/status")),
    { ok: true },
  );
});

test("production limiter is enabled with bounded defaults", () => {
  cleanEnv();
  const request = new Request("https://unit.invalid/auth", {
    headers: { "x-nf-client-connection-ip": "192.0.2.44" },
  });
  let decision = { ok: true } as ReturnType<typeof softRateLimit>;
  for (let i = 0; i < 121; i++) decision = softRateLimit(request);
  assert.deepEqual(decision, { ok: false, status: 429, error: "rate limited" });
});

test("production cannot disable the limiter", () => {
  cleanEnv();
  process.env.RATE_LIMIT_ENABLED = "0";
  process.env.RATE_LIMIT_MAX_REQUESTS = "1";
  const request = new Request("https://unit.invalid/auth", {
    headers: { "x-nf-client-connection-ip": "192.0.2.48" },
  });
  assert.deepEqual(softRateLimit(request), { ok: true });
  assert.deepEqual(softRateLimit(request), {
    ok: false,
    status: 429,
    error: "rate limited",
  });
});

test("token minting requires an enrollment gate in production", async () => {
  cleanEnv();
  process.env.CONTROL_PLANE_TOKEN = "admin-secret-value";
  process.env.AGENT_TOKEN_SECRET = "agent-signing-secret-value";
  const response = await authHandler(
    new Request("https://unit.invalid/auth", {
      method: "POST",
      headers: {
        authorization: "Bearer admin-secret-value",
        "content-type": "application/json",
        "x-nf-client-connection-ip": "192.0.2.45",
      },
      body: JSON.stringify({ agent_id: "planner" }),
    }),
  );
  assert.equal(response.status, 503);
  assert.match(await response.text(), /requires AGENT_JOIN_SECRET or proof of work/);
});

test("production self-service accepts a configured join secret", async () => {
  cleanEnv();
  process.env.AGENT_TOKEN_SECRET = "agent-signing-secret-value";
  process.env.AGENT_JOIN_SECRET = "join-secret-value";
  const response = await authHandler(
    new Request("https://unit.invalid/auth", {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-nf-client-connection-ip": "192.0.2.47",
      },
      body: JSON.stringify({ agent_id: "planner", join_secret: "join-secret-value" }),
    }),
  );
  assert.equal(response.status, 200);
  const body = await response.json() as { access_token?: string };
  assert.match(body.access_token ?? "", /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/);
});

test("production status refuses mock success without a gateway", async () => {
  cleanEnv();
  process.env.CONTROL_PLANE_TOKEN = "admin-secret-value";
  const response = await statusHandler(
    new Request("https://unit.invalid/status", {
      headers: {
        authorization: "Bearer admin-secret-value",
        "x-nf-client-connection-ip": "192.0.2.46",
      },
    }),
  );
  assert.equal(response.status, 503);
  assert.match(await response.text(), /not configured/);
});

test.after(cleanEnv);
