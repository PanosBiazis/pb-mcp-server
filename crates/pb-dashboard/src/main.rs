//! pb-mcp-server Admin Dashboard
//!
//! Cross-platform GUI dashboard for monitoring and administering a pb-gateway
//! instance. Connects to the gateway HTTP API and displays server status, agent
//! workers, bus messages, tools, security info, and performance metrics.
//!
//! # Usage
//!
//! ```sh
//! cargo run -p pb-dashboard
//! ```

use egui::{Color32, RichText, Vec2};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Which dashboard tab is currently selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DashboardTab {
    Overview,
    Agents,
    Bus,
    Tools,
    Security,
    Metrics,
}

impl DashboardTab {
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Agents => "Agents",
            Self::Bus => "Bus",
            Self::Tools => "Tools",
            Self::Security => "Security",
            Self::Metrics => "Metrics",
        }
    }

    const ALL: [Self; 6] = [
        Self::Overview,
        Self::Agents,
        Self::Bus,
        Self::Tools,
        Self::Security,
        Self::Metrics,
    ];
}

#[derive(Clone, Debug)]
struct AgentInfo {
    id: String,
    role: String,
    status: String,
    message_count: String,
}

#[derive(Clone, Debug)]
struct BusMessage {
    id: String,
    from: String,
    to: String,
    topic: String,
    kind: String,
    body: String,
    timestamp: String,
}

#[derive(Clone, Debug)]
struct ToolEntry {
    name: String,
    category: String,
    profile: String,
    risk_level: String,
    enabled: bool,
}

#[derive(Clone, Debug)]
struct SecurityInfo {
    token_enabled: String,
    signing_enabled: String,
    notes: String,
}

#[derive(Clone, Debug, Default)]
struct MetricsData {
    requests_per_sec: Vec<f64>,
    response_times_ms: Vec<f64>,
    error_rates: Vec<f64>,
}

struct DashboardApp {
    gateway_url: String,
    agents: Vec<AgentInfo>,
    bus_messages: Vec<BusMessage>,
    tools: Vec<ToolEntry>,
    security_status: SecurityInfo,
    metrics: MetricsData,
    selected_tab: DashboardTab,
    auto_refresh: bool,
    refresh_interval_secs: f32,
    last_refresh: Instant,
    status_text: String,
    bus_filter: String,
    connected: bool,
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:8123".to_string(),
            agents: Vec::new(),
            bus_messages: Vec::new(),
            tools: Vec::new(),
            security_status: SecurityInfo {
                token_enabled: "—".to_string(),
                signing_enabled: "—".to_string(),
                notes: "Not connected".to_string(),
            },
            metrics: MetricsData::default(),
            selected_tab: DashboardTab::Overview,
            auto_refresh: true,
            refresh_interval_secs: 5.0,
            last_refresh: Instant::now(),
            status_text:
                "Enter gateway URL; live gateway polling is not wired in this dashboard yet"
                    .to_string(),
            bus_filter: String::new(),
            connected: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Demo data helpers
// ---------------------------------------------------------------------------

fn load_demo_data(app: &mut DashboardApp) {
    app.agents = vec![
        AgentInfo {
            id: "agent-coder-01".into(),
            role: "coder".into(),
            status: "running".into(),
            message_count: "142".into(),
        },
        AgentInfo {
            id: "agent-reviewer-01".into(),
            role: "reviewer".into(),
            status: "idle".into(),
            message_count: "87".into(),
        },
        AgentInfo {
            id: "agent-planner-01".into(),
            role: "planner".into(),
            status: "running".into(),
            message_count: "53".into(),
        },
        AgentInfo {
            id: "agent-linter-01".into(),
            role: "linter".into(),
            status: "stopped".into(),
            message_count: "31".into(),
        },
    ];

    app.bus_messages = vec![
        BusMessage {
            id: "msg-001".into(),
            from: "agent-coder-01".into(),
            to: "agent-reviewer-01".into(),
            topic: "code-change".into(),
            kind: "notification".into(),
            body: "Updated src/lib.rs — added parse_config()".into(),
            timestamp: "2025-01-15T10:23:01Z".into(),
        },
        BusMessage {
            id: "msg-002".into(),
            from: "agent-reviewer-01".into(),
            to: "agent-coder-01".into(),
            topic: "review-result".into(),
            kind: "reply".into(),
            body: "LGTM — minor: add doc comment on parse_config".into(),
            timestamp: "2025-01-15T10:24:12Z".into(),
        },
        BusMessage {
            id: "msg-003".into(),
            from: "agent-planner-01".into(),
            to: "".into(),
            topic: "task-assign".into(),
            kind: "broadcast".into(),
            body: "New task: refactor error handling module".into(),
            timestamp: "2025-01-15T10:25:00Z".into(),
        },
        BusMessage {
            id: "msg-004".into(),
            from: "agent-coder-01".into(),
            to: "agent-linter-01".into(),
            topic: "lint-request".into(),
            kind: "request".into(),
            body: "Lint src/error_handlers.rs after changes".into(),
            timestamp: "2025-01-15T10:26:30Z".into(),
        },
        BusMessage {
            id: "msg-005".into(),
            from: "agent-linter-01".into(),
            to: "agent-coder-01".into(),
            topic: "lint-result".into(),
            kind: "reply".into(),
            body: "2 warnings, 0 errors — see attached report".into(),
            timestamp: "2025-01-15T10:27:05Z".into(),
        },
    ];

    app.tools = vec![
        ToolEntry {
            name: "read_file".into(),
            category: "core-code".into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            enabled: true,
        },
        ToolEntry {
            name: "search_code".into(),
            category: "core-code".into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            enabled: true,
        },
        ToolEntry {
            name: "git_status".into(),
            category: "core-code".into(),
            profile: "core-code".into(),
            risk_level: "low".into(),
            enabled: true,
        },
        ToolEntry {
            name: "exec_local".into(),
            category: "terminal-exec".into(),
            profile: "local-power".into(),
            risk_level: "high".into(),
            enabled: false,
        },
        ToolEntry {
            name: "web_fetch".into(),
            category: "web-fetch".into(),
            profile: "web-tools".into(),
            risk_level: "medium".into(),
            enabled: false,
        },
        ToolEntry {
            name: "web_search".into(),
            category: "web-search".into(),
            profile: "web-tools".into(),
            risk_level: "medium".into(),
            enabled: false,
        },
        ToolEntry {
            name: "ocr_extract".into(),
            category: "ocr-tools".into(),
            profile: "local-power".into(),
            risk_level: "medium".into(),
            enabled: false,
        },
        ToolEntry {
            name: "ml_inference".into(),
            category: "ml-tools".into(),
            profile: "experimental-lab".into(),
            risk_level: "high".into(),
            enabled: false,
        },
        ToolEntry {
            name: "review_packet".into(),
            category: "review-refactor".into(),
            profile: "review-refactor".into(),
            risk_level: "low".into(),
            enabled: true,
        },
        ToolEntry {
            name: "agent_send".into(),
            category: "agent-orchestration".into(),
            profile: "agent-orchestration".into(),
            risk_level: "medium".into(),
            enabled: true,
        },
    ];

    app.security_status = SecurityInfo {
        token_enabled: "Yes".into(),
        signing_enabled: "No".into(),
        notes: "Token protection active. Request signing not configured. Project-root confinement enabled.".into(),
    };

    let n = 60;
    let mut rps = Vec::with_capacity(n);
    let mut rt = Vec::with_capacity(n);
    let mut er = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64;
        rps.push(12.0 + 5.0 * (t * 0.15).sin() + 3.0 * (t * 0.07).cos());
        rt.push(45.0 + 15.0 * (t * 0.1).sin() + 8.0 * (t * 0.23).cos());
        er.push(0.02 + 0.03 * (t * 0.05).sin().abs());
    }
    app.metrics = MetricsData {
        requests_per_sec: rps,
        response_times_ms: rt,
        error_rates: er,
    };
}

// ---------------------------------------------------------------------------
// eframe::App implementation
// ---------------------------------------------------------------------------

impl eframe::App for DashboardApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_refresh && self.connected {
            if self.last_refresh.elapsed().as_secs_f32() >= self.refresh_interval_secs {
                self.last_refresh = Instant::now();
                self.status_text =
                    format!("Refreshed at {}", chrono::Local::now().format("%H:%M:%S"));
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("⬡ pb-dashboard").strong().size(16.0));
            ui.separator();
            ui.label("Gateway:");
            ui.add(
                egui::TextEdit::singleline(&mut self.gateway_url)
                    .desired_width(220.0)
                    .hint_text("http://localhost:8123"),
            );
            if ui.button("Load Demo").clicked() {
                self.connected = true;
                load_demo_data(self);
                self.status_text = format!(
                    "Loaded local demo data for {}; live gateway polling/control is not wired yet",
                    self.gateway_url
                );
            }
            ui.separator();
            ui.label("Auto-refresh:");
            ui.toggle_value(&mut self.auto_refresh, "On");
            if self.auto_refresh {
                ui.add(
                    egui::Slider::new(&mut self.refresh_interval_secs, 1.0..=60.0)
                        .text("s")
                        .step_by(1.0),
                );
            }
            ui.separator();
            let dot_color = if self.connected {
                Color32::GREEN
            } else {
                Color32::GRAY
            };
            ui.colored_label(dot_color, "●");
            ui.label(if self.connected {
                "connected"
            } else {
                "disconnected"
            });
        });
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            for tab in DashboardTab::ALL {
                let is_selected = self.selected_tab == tab;
                let label = RichText::new(tab.label()).size(14.0);
                if ui.selectable_label(is_selected, label).clicked() {
                    self.selected_tab = tab;
                }
            }
        });
        ui.separator();

        match self.selected_tab {
            DashboardTab::Overview => self.draw_overview(ui),
            DashboardTab::Agents => self.draw_agents(ui),
            DashboardTab::Bus => self.draw_bus(ui),
            DashboardTab::Tools => self.draw_tools(ui),
            DashboardTab::Security => self.draw_security(ui),
            DashboardTab::Metrics => self.draw_metrics(ui),
        }

        ui.separator();
        ui.label(
            RichText::new(&self.status_text)
                .small()
                .color(Color32::GRAY),
        );
    }
}

// ---------------------------------------------------------------------------
// Tab drawing helpers
// ---------------------------------------------------------------------------

impl DashboardApp {
    fn draw_overview(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("pb-mcp-server Overview").size(20.0).strong());
            ui.add_space(8.0);
        });

        egui::Grid::new("overview_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Server Status");
                let status_color = if self.connected {
                    Color32::GREEN
                } else {
                    Color32::YELLOW
                };
                ui.colored_label(
                    status_color,
                    if self.connected {
                        "Running"
                    } else {
                        "Not Connected"
                    },
                );
                ui.end_row();

                ui.label("Gateway URL");
                ui.label(&self.gateway_url);
                ui.end_row();

                ui.label("Service");
                ui.label("pb-mcp-server v0.2.0");
                ui.end_row();

                ui.label("Uptime");
                ui.label(if self.connected { "Active" } else { "—" });
                ui.end_row();

                ui.label("Project Root");
                ui.label("/path/to/project (from gateway)");
                ui.end_row();

                ui.label("Active Profile");
                ui.label("default (core-code + agent-orchestration + review-refactor)");
                ui.end_row();

                ui.label("Agents Online");
                ui.label(format!(
                    "{} / {}",
                    self.agents.iter().filter(|a| a.status == "running").count(),
                    self.agents.len()
                ));
                ui.end_row();

                ui.label("Bus Messages");
                ui.label(format!("{}", self.bus_messages.len()));
                ui.end_row();

                ui.label("Tools Enabled");
                ui.label(format!(
                    "{} / {}",
                    self.tools.iter().filter(|t| t.enabled).count(),
                    self.tools.len()
                ));
                ui.end_row();
            });

        ui.add_space(12.0);

        egui::CollapsingHeader::new("Quick Stats")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(150.0, 60.0));
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Agents").strong());
                            ui.label(RichText::new(format!("{}", self.agents.len())).size(24.0));
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(150.0, 60.0));
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Tools").strong());
                            ui.label(
                                RichText::new(format!(
                                    "{}",
                                    self.tools.iter().filter(|t| t.enabled).count()
                                ))
                                .size(24.0),
                            );
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(150.0, 60.0));
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Msgs").strong());
                            ui.label(
                                RichText::new(format!("{}", self.bus_messages.len())).size(24.0),
                            );
                        });
                    });
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(150.0, 60.0));
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Req/s").strong());
                            let last_rps =
                                self.metrics.requests_per_sec.last().copied().unwrap_or(0.0);
                            ui.label(RichText::new(format!("{last_rps:.1}")).size(24.0));
                        });
                    });
                });
            });
    }

    fn draw_agents(&mut self, ui: &mut egui::Ui) {
        ui.heading("Agent Workers");
        ui.add_space(4.0);

        if self.agents.is_empty() {
            ui.label("No agents loaded. Use Load Demo to inspect the dashboard layout.");
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                egui::Grid::new("agents_grid")
                    .num_columns(4)
                    .spacing([16.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("ID").strong());
                        ui.label(RichText::new("Role").strong());
                        ui.label(RichText::new("Status").strong());
                        ui.label(RichText::new("Messages").strong());
                        ui.end_row();

                        for agent in &self.agents {
                            ui.label(&agent.id);
                            ui.label(&agent.role);
                            let color = match agent.status.as_str() {
                                "running" => Color32::GREEN,
                                "idle" => Color32::YELLOW,
                                "stopped" => Color32::RED,
                                _ => Color32::GRAY,
                            };
                            ui.colored_label(color, &agent.status);
                            ui.label(&agent.message_count);
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(8.0);
        ui.separator();
        ui.label("Agent Controls");
        ui.add_space(4.0);

        egui::Grid::new("agent_controls")
            .num_columns(4)
            .spacing([10.0, 6.0])
            .show(ui, |ui| {
                for agent in &self.agents {
                    ui.label(&agent.id);
                    if agent.status == "running" {
                        if ui.button("⏹ Stop").clicked() {
                            self.status_text = format!(
                                "Agent control endpoint is not wired yet; cannot stop {}",
                                agent.id
                            );
                        }
                    } else if ui.button("▶ Start").clicked() {
                        self.status_text = format!(
                            "Agent control endpoint is not wired yet; cannot start {}",
                            agent.id
                        );
                    }
                    if ui.button("🔄 Restart").clicked() {
                        self.status_text = format!(
                            "Agent control endpoint is not wired yet; cannot restart {}",
                            agent.id
                        );
                    }
                    ui.label(format!("[{}]", agent.status));
                    ui.end_row();
                }
            });
    }

    fn draw_bus(&mut self, ui: &mut egui::Ui) {
        ui.heading("Agent Bus Messages");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(
                egui::TextEdit::singleline(&mut self.bus_filter)
                    .desired_width(200.0)
                    .hint_text("topic, agent, kind…"),
            );
            if ui.button("Clear").clicked() {
                self.bus_filter.clear();
            }
        });

        ui.add_space(4.0);

        if self.bus_messages.is_empty() {
            ui.label("No bus messages. Connect to a gateway to fetch recent messages.");
            return;
        }

        let filter = self.bus_filter.to_lowercase();
        let filtered: Vec<&BusMessage> = self
            .bus_messages
            .iter()
            .filter(|m| {
                filter.is_empty()
                    || m.topic.to_lowercase().contains(&filter)
                    || m.from.to_lowercase().contains(&filter)
                    || m.to.to_lowercase().contains(&filter)
                    || m.kind.to_lowercase().contains(&filter)
                    || m.body.to_lowercase().contains(&filter)
            })
            .collect();

        ui.label(format!(
            "{} messages ({} shown)",
            self.bus_messages.len(),
            filtered.len()
        ));
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                for msg in filtered {
                    egui::CollapsingHeader::new(format!(
                        "{} {} → {}  [{}]  {}",
                        msg.timestamp,
                        msg.from,
                        if msg.to.is_empty() {
                            "broadcast".to_string()
                        } else {
                            msg.to.clone()
                        },
                        msg.topic,
                        msg.kind
                    ))
                    .id_salt(&msg.id)
                    .show(ui, |ui| {
                        egui::Grid::new(("bus_msg", &msg.id))
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("ID:");
                                ui.label(&msg.id);
                                ui.end_row();
                                ui.label("From:");
                                ui.label(&msg.from);
                                ui.end_row();
                                ui.label("To:");
                                ui.label(if msg.to.is_empty() {
                                    "broadcast"
                                } else {
                                    &msg.to
                                });
                                ui.end_row();
                                ui.label("Topic:");
                                ui.label(&msg.topic);
                                ui.end_row();
                                ui.label("Kind:");
                                ui.label(&msg.kind);
                                ui.end_row();
                                ui.label("Body:");
                                ui.label(&msg.body);
                                ui.end_row();
                                ui.label("Time:");
                                ui.label(&msg.timestamp);
                                ui.end_row();
                            });
                    });
                }
            });
    }

    fn draw_tools(&mut self, ui: &mut egui::Ui) {
        ui.heading("Registered Tools");
        ui.add_space(4.0);

        if self.tools.is_empty() {
            ui.label("No tool data. Connect to a gateway to fetch tool registry.");
            return;
        }

        let mut categories: std::collections::BTreeMap<String, Vec<&ToolEntry>> =
            std::collections::BTreeMap::new();
        for tool in &self.tools {
            categories
                .entry(tool.category.clone())
                .or_default()
                .push(tool);
        }

        egui::ScrollArea::vertical()
            .max_height(600.0)
            .show(ui, |ui| {
                for (cat, entries) in &categories {
                    egui::CollapsingHeader::new(format!("{} ({} tools)", cat, entries.len()))
                        .default_open(true)
                        .id_salt(cat)
                        .show(ui, |ui| {
                            egui::Grid::new(("tools_cat", cat))
                                .num_columns(4)
                                .spacing([16.0, 6.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Name").strong());
                                    ui.label(RichText::new("Profile").strong());
                                    ui.label(RichText::new("Risk").strong());
                                    ui.label(RichText::new("Enabled").strong());
                                    ui.end_row();

                                    for entry in entries {
                                        ui.label(&entry.name);
                                        ui.label(&entry.profile);
                                        let risk_color = match entry.risk_level.as_str() {
                                            "low" => Color32::from_rgb(80, 180, 80),
                                            "medium" => Color32::from_rgb(220, 180, 40),
                                            "high" => Color32::from_rgb(220, 60, 60),
                                            _ => Color32::GRAY,
                                        };
                                        ui.colored_label(risk_color, &entry.risk_level);
                                        if entry.enabled {
                                            ui.colored_label(Color32::GREEN, "✔ on");
                                        } else {
                                            ui.colored_label(Color32::GRAY, "✘ off");
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                }
            });
    }

    fn draw_security(&mut self, ui: &mut egui::Ui) {
        ui.heading("Security Status");
        ui.add_space(8.0);

        egui::Grid::new("security_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Token Protection");
                let token_color = if self.security_status.token_enabled == "Yes" {
                    Color32::GREEN
                } else {
                    Color32::GRAY
                };
                ui.colored_label(token_color, &self.security_status.token_enabled);
                ui.end_row();

                ui.label("Request Signing");
                let sign_color = if self.security_status.signing_enabled == "Yes" {
                    Color32::GREEN
                } else {
                    Color32::GRAY
                };
                ui.colored_label(sign_color, &self.security_status.signing_enabled);
                ui.end_row();

                ui.label("Notes");
                ui.label(&self.security_status.notes);
                ui.end_row();
            });

        ui.add_space(12.0);

        egui::CollapsingHeader::new("Rate Limits")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("rate_limit_grid")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Global Requests/min");
                        ui.label("120");
                        ui.end_row();
                        ui.label("Per-Agent Requests/min");
                        ui.label("30");
                        ui.end_row();
                        ui.label("Burst Allowance");
                        ui.label("10");
                        ui.end_row();
                    });
            });

        ui.add_space(8.0);

        egui::CollapsingHeader::new("Audit Log (recent)")
            .default_open(false)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        let audit_entries = [
                            ("2025-01-15T10:00:01Z", "INFO", "Gateway started"),
                            ("2025-01-15T10:00:02Z", "INFO", "Token protection enabled"),
                            (
                                "2025-01-15T10:01:15Z",
                                "AUTH",
                                "Agent agent-coder-01 authenticated",
                            ),
                            (
                                "2025-01-15T10:02:30Z",
                                "AUTH",
                                "Agent agent-reviewer-01 authenticated",
                            ),
                            (
                                "2025-01-15T10:05:00Z",
                                "RATE",
                                "Rate limit reached for 192.168.1.50",
                            ),
                            (
                                "2025-01-15T10:10:00Z",
                                "SIGN",
                                "Unsigned request rejected from 10.0.0.12",
                            ),
                            (
                                "2025-01-15T10:15:22Z",
                                "INFO",
                                "Agent agent-linter-01 stopped",
                            ),
                            (
                                "2025-01-15T10:20:00Z",
                                "AUTH",
                                "Agent agent-planner-01 authenticated",
                            ),
                        ];
                        egui::Grid::new("audit_grid")
                            .num_columns(3)
                            .spacing([12.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Time").strong());
                                ui.label(RichText::new("Kind").strong());
                                ui.label(RichText::new("Message").strong());
                                ui.end_row();
                                for (ts, kind, msg) in &audit_entries {
                                    ui.label(*ts);
                                    let kind_color = match *kind {
                                        "AUTH" => Color32::from_rgb(100, 160, 220),
                                        "RATE" => Color32::from_rgb(220, 180, 40),
                                        "SIGN" => Color32::from_rgb(220, 100, 60),
                                        _ => Color32::GRAY,
                                    };
                                    ui.colored_label(kind_color, *kind);
                                    ui.label(*msg);
                                    ui.end_row();
                                }
                            });
                    });
            });
    }

    fn draw_metrics(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance Metrics");
        ui.add_space(8.0);

        if self.metrics.requests_per_sec.is_empty() {
            ui.label("No metrics data. Connect to a gateway to collect metrics.");
            return;
        }

        let x_values: Vec<f64> = (0..self.metrics.requests_per_sec.len())
            .map(|i| i as f64)
            .collect();

        let draw_chart = |ui: &mut egui::Ui, title: &str, y_values: &[f64], line_color: Color32| {
            ui.group(|ui| {
                ui.label(RichText::new(title).strong());
                let line = egui_plot::Line::new(
                    title,
                    egui_plot::PlotPoints::from_iter(
                        x_values.iter().zip(y_values.iter()).map(|(&x, &y)| [x, y]),
                    ),
                )
                .color(line_color)
                .width(2.0);

                egui_plot::Plot::new(title)
                    .view_aspect(3.0)
                    .height(180.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                    });
            });
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            draw_chart(
                ui,
                "Requests / sec",
                &self.metrics.requests_per_sec,
                Color32::from_rgb(100, 200, 120),
            );
            ui.add_space(8.0);
            draw_chart(
                ui,
                "Response Time (ms)",
                &self.metrics.response_times_ms,
                Color32::from_rgb(100, 160, 255),
            );
            ui.add_space(8.0);
            draw_chart(
                ui,
                "Error Rate",
                &self.metrics.error_rates,
                Color32::from_rgb(255, 100, 100),
            );
        });
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("pb-mcp-server Dashboard"),
        ..Default::default()
    };

    eframe::run_native(
        "pb-mcp-server Dashboard",
        options,
        Box::new(|_cc| Ok(Box::new(DashboardApp::default()))),
    )?;

    Ok(())
}
