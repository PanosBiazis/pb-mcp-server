# Security Policy

## Supported Versions

Only the latest release in the `v0.2.x` series receives security patches.
Older release lines are end-of-life and will not receive back-ports.

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | ✅ Yes             |
| 0.1.x   | ❌ No              |

---

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Send a report by email to **panosbiazis2003@gmail.com** with the subject line:

```
[SECURITY] pb-mcp-server
```

Include as much of the following as you can:

- A clear description of the vulnerability and its potential impact
- Steps to reproduce (proof-of-concept code, environment details, etc.)
- The version(s) of `pb-mcp-server` you tested against
- Any mitigations or workarounds you are aware of

### Response timeline

| Milestone                              | Target            |
| -------------------------------------- | ----------------- |
| Initial acknowledgement                | Within **48 hours** |
| Confirmed / triaged status update      | Within **5 days** |
| Patch released (critical / high)       | Within **14 days** |
| Patch released (medium / low)          | Next scheduled release |

If you have not received an acknowledgement within 48 hours, please follow up — emails occasionally land in spam.

---

## Disclosure Policy

`pb-mcp-server` follows **coordinated (responsible) disclosure**:

1. Reporter submits a private report.
2. Maintainer confirms the issue and develops a fix.
3. A patched release is published.
4. A public advisory (GitHub Security Advisory) is posted **after** the patch is available, crediting the reporter (unless they prefer to remain anonymous).

We ask that reporters refrain from publicly disclosing the vulnerability until a patch has been released or 90 days have elapsed, whichever comes first.

---

## Security Considerations

### Runtime permissions

`pb-mcp-server` runs inside your IDE (Zed, Cursor) and inherits the **same filesystem and process permissions as your IDE process**. It has read/write access to your project directory by default. Treat it with the same level of trust you would give any IDE extension.

### Network profile

Setting the environment variable:

```
MINT_NETWORK_PROFILE=public
```

switches the server into a **read-only mode** that disables all write and mutating operations. Use this when running the server in CI pipelines or other environments where accidental writes would be harmful.

### Terminal execution

Terminal execution is **disabled by default**. It must be explicitly opted in by setting:

```
MINT_TERMINAL_ENABLE=1
```

Only set this flag if you understand that it allows the MCP agent to execute shell commands in your environment. Never enable it in shared or multi-tenant environments.

### HTTP gateway authentication

When the HTTP gateway is active, every request must carry a valid bearer token:

```
PB_GATEWAY_TOKEN=<your-secret-token>
```

The server will reject requests that do not present this token. Never expose the gateway port on a public network without a strong, randomly generated token.

### Message integrity

All inter-node messages exchanged by `pb-mcp-server` are signed with **HMAC-SHA256**. Messages with an invalid or missing signature are rejected before processing.

### Memory safety

The codebase enforces `#![deny(unsafe_code)]` at compile time. No `unsafe` Rust blocks are permitted in the project source, reducing the attack surface for memory-safety vulnerabilities.

---

## Known Security Boundaries

The following behaviors are **intentional by design** and are not considered security vulnerabilities:

| Scenario | Rationale |
| --- | --- |
| A user sets `MINT_TERMINAL_ENABLE=1` and the agent runs shell commands | The user explicitly opted in to terminal execution |
| The agent reads files inside the open project directory | The server is designed to assist with the project; file access is expected |
| A user exposes the HTTP gateway on a local port | The gateway is localhost-only by default; exposure is a conscious user choice |
| `MINT_NETWORK_PROFILE=public` does not prevent *reading* project files | Read-only mode restricts writes, not reads, by design |

If you are unsure whether something qualifies as a security issue, **err on the side of reporting it** — we would rather receive a false positive than miss a real vulnerability.

---

## Contact

**Maintainer:** PanosBiazis  
**Email:** panosbiazis2003@gmail.com  
**Repository:** https://github.com/PanosBiazis/pb-mcp-server  
**License:** MIT
