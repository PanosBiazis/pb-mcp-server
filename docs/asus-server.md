## Run `pb-gateway` on an Asus laptop (systemd + hardened defaults)

### Build on the Asus laptop

```bash
git clone <your repo url>
cd pb-mcp-server
cargo build --release
```

### Recommended environment

- **`MINT_PROJECT_ROOT`**: the project root you want the gateway to represent
- **`PB_GATEWAY_BIND`**: default `127.0.0.1:8080` (recommended)
- **`PB_GATEWAY_TOKEN`**: optional but strongly recommended if anything can reach the port

### systemd unit (example)

Create `/etc/systemd/system/pb-gateway.service`:

```ini
[Unit]
Description=PB Gateway (status + review packet storage)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=panosboy
WorkingDirectory=/home/panosboy/pb-mcp-server
Environment=MINT_PROJECT_ROOT=/home/panosboy/projects/your-project
Environment=PB_GATEWAY_BIND=127.0.0.1:8080
Environment=PB_GATEWAY_TOKEN=change-me
ExecStart=/home/panosboy/pb-mcp-server/target/release/pb-gateway
Restart=always
RestartSec=2

# Hardening (works well for a simple HTTP service)
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectControlGroups=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
LockPersonality=true
RestrictRealtime=true
RestrictSUIDSGID=true
RestrictNamespaces=true
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
SystemCallArchitectures=native
SystemCallFilter=@system-service

# Needed because we write under MINT_PROJECT_ROOT/.mint-mcp/
ReadWritePaths=/home/panosboy/projects/your-project/.mint-mcp

[Install]
WantedBy=multi-user.target
```

Enable it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pb-gateway
sudo systemctl status pb-gateway
```

### Exposing it safely

If you want Netlify (public internet) to reach your Asus laptop, do **not** expose `pb-gateway` directly.

Better options:

- Bind `pb-gateway` to `127.0.0.1` and use a secure tunnel / reverse proxy that terminates TLS and adds auth.
- If you later expose it, keep the surface **tiny** (status + review-packet only) and require `PB_GATEWAY_TOKEN`.

