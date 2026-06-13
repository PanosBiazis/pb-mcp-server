//! App-inspired local tools.
//!
//! These are deterministic local equivalents of common capabilities found in
//! workflow MCP servers and coding-agent apps. They do not call external apps.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

/// Return a static catalog of app-inspired local pb-mcp capabilities.
pub fn app_capability_catalog_work() -> Value {
    json!({
        "format": "pb-mcp-server.app.capability_catalog.v1",
        "boundary": "local deterministic equivalents; no external app control",
        "families": [
            {
                "source_style": "n8n / workflow MCP",
                "tools": [
                    {"name": "workflow.validate", "local_equivalent": "validate workflow graph nodes/connections"},
                    {"name": "workflow.execution_plan", "local_equivalent": "topological execution levels for DAG workflows"},
                    {"name": "workflow.template_generate", "local_equivalent": "generate safe starter workflow graphs"},
                    {"name": "app.node_suggest", "local_equivalent": "suggest workflow/app nodes by intent"},
                    {"name": "tool.recipe", "local_equivalent": "map an intent to pb-mcp tool sequences"}
                ]
            },
            {
                "source_style": "Claude Code / Codex / OpenCode",
                "tools": [
                    {"name": "agent.task_packet", "local_equivalent": "structured coding-agent task brief"},
                    {"name": "agent.prompt_pack", "local_equivalent": "pack prompt/context/constraints into an agent-ready instruction block"},
                    {"name": "agent.session_summary", "local_equivalent": "summarize agent session logs into decisions/actions/risks"},
                    {"name": "agent.patch_plan", "local_equivalent": "validate planned file edits and verification steps"},
                    {"name": "app.command_palette", "local_equivalent": "command-palette suggestions mapped to pb-mcp tools"},
                    {"name": "app.capability_search", "local_equivalent": "search local app-inspired capability catalog"},
                    {"name": "workspace_context", "local_equivalent": "repo orientation before edits"}
                ]
            },
            {
                "source_style": "Skills / plugins / command surfaces",
                "tools": [
                    {"name": "app.skill_manifest_validate", "local_equivalent": "validate a local skill manifest/instruction block"},
                    {"name": "app.skill_template_generate", "local_equivalent": "generate a deterministic skill skeleton"},
                    {"name": "app.plugin_manifest_validate", "local_equivalent": "validate local plugin metadata"},
                    {"name": "app.plugin_route_plan", "local_equivalent": "route an intent through plugin skills/commands/tools"},
                    {"name": "app.slash_command_expand", "local_equivalent": "expand slash-style commands into pb-mcp tool sequences"},
                    {"name": "app.hooks_policy_validate", "local_equivalent": "validate hook definitions before use"},
                    {"name": "app.permission_policy_evaluate", "local_equivalent": "evaluate a proposed action against a local permission profile"},
                    {"name": "app.model_profile_select", "local_equivalent": "choose a model/runtime profile from task constraints"},
                    {"name": "app.context_budget_plan", "local_equivalent": "rank context items into a token budget"},
                    {"name": "app.task_queue_plan", "local_equivalent": "order dependent agent tasks into executable levels"}
                ]
            },
            {
                "source_style": "Shells / terminals",
                "tools": [
                    {"name": "shell.command_catalog", "local_equivalent": "catalog common bash/powershell/batch/macos terminal commands"},
                    {"name": "shell.command_translate", "local_equivalent": "translate common command intent across shell families"},
                    {"name": "shell.script_template_generate", "local_equivalent": "generate safe script skeletons for bash/powershell/batch/zsh"},
                    {"name": "shell.command_safety_check", "local_equivalent": "classify shell command risk before terminal execution"}
                ]
            },
            {
                "source_style": "OpenClaw / Hermes / multi-agent apps",
                "tools": [
                    {"name": "agent_fan_out", "local_equivalent": "parallel agent dispatch"},
                    {"name": "agent_send_pipeline", "local_equivalent": "serial agent handoff"},
                    {"name": "consensus_propose", "local_equivalent": "local consensus proposal/vote/tally"}
                ]
            },
            {
                "source_style": "Generic MCP server builders",
                "tools": [
                    {"name": "mcp.manifest_validate", "local_equivalent": "validate a local MCP-style tool/server manifest"},
                    {"name": "mcp.tool_spec_generate", "local_equivalent": "generate a deterministic MCP-style tool spec skeleton"},
                    {"name": "mcp.server_config_generate", "local_equivalent": "generate stdio/http server config skeletons"},
                    {"name": "mcp.prompt_template_validate", "local_equivalent": "validate prompt-template variables"},
                    {"name": "mcp.resource_template_validate", "local_equivalent": "validate resource-template URI placeholders"},
                    {"name": "mcp.sampling_request_plan", "local_equivalent": "plan a local MCP sampling-style request"}
                ]
            }
        ]
    })
}

/// Search the app capability catalog by source style, tool name, or local equivalent.
pub fn app_capability_search_work(args: Value) -> Value {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if query.is_empty() {
        return json!({"error": "query must be a non-empty string"});
    }
    let catalog = app_capability_catalog_work();
    let mut matches = Vec::new();
    if let Some(families) = catalog.get("families").and_then(|v| v.as_array()) {
        for family in families {
            let source_style = family
                .get("source_style")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(tools) = family.get("tools").and_then(|v| v.as_array()) {
                for tool in tools {
                    let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let local = tool
                        .get("local_equivalent")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let haystack = format!("{source_style} {name} {local}").to_ascii_lowercase();
                    if haystack.contains(&query) {
                        matches.push(json!({
                            "source_style": source_style,
                            "name": name,
                            "local_equivalent": local
                        }));
                    }
                }
            }
        }
    }
    json!({"query": query, "match_count": matches.len(), "matches": matches})
}

/// Validate an n8n-style workflow graph made of nodes and connections.
pub fn workflow_validate_work(args: Value) -> Value {
    let workflow = args.get("workflow").unwrap_or(&args);
    let nodes = match workflow.get("nodes").and_then(|v| v.as_array()) {
        Some(nodes) => nodes,
        None => return json!({"valid": false, "errors": ["workflow.nodes must be an array"]}),
    };
    let connections = workflow
        .get("connections")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut ids = BTreeSet::<String>::new();
    let mut types = BTreeMap::<String, usize>::new();

    for (index, node) in nodes.iter().enumerate() {
        let Some(id) = node_id(node) else {
            errors.push(format!("node[{index}] needs string id or name"));
            continue;
        };
        if !ids.insert(id.clone()) {
            errors.push(format!("duplicate node id/name: {id}"));
        }
        let kind = node
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        *types.entry(kind.clone()).or_default() += 1;
        if kind == "unknown" {
            warnings.push(format!("node {id} has no type"));
        }
    }

    let mut edge_count = 0usize;
    for (index, edge) in connections.iter().enumerate() {
        let Some((from, to)) = connection_edge(edge) else {
            errors.push(format!(
                "connection[{index}] needs from/to or source/target"
            ));
            continue;
        };
        edge_count += 1;
        if !ids.contains(&from) {
            errors.push(format!("connection[{index}] unknown from node: {from}"));
        }
        if !ids.contains(&to) {
            errors.push(format!("connection[{index}] unknown to node: {to}"));
        }
        if from == to {
            warnings.push(format!("connection[{index}] is a self-loop on {from}"));
        }
    }

    let cycle = detect_cycle(&ids, &connections);
    if cycle {
        errors.push("workflow graph contains a cycle".to_string());
    }

    json!({
        "valid": errors.is_empty(),
        "node_count": nodes.len(),
        "edge_count": edge_count,
        "node_types": types,
        "errors": errors,
        "warnings": warnings
    })
}

/// Build a deterministic topological execution plan for a valid workflow DAG.
pub fn workflow_execution_plan_work(args: Value) -> Value {
    let validation = workflow_validate_work(args.clone());
    if !validation
        .get("valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return json!({"error": "workflow is invalid", "validation": validation});
    }
    let workflow = args.get("workflow").unwrap_or(&args);
    let nodes = workflow
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let connections = workflow
        .get("connections")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut indegree = HashMap::<String, usize>::new();
    let mut outgoing = HashMap::<String, Vec<String>>::new();
    for node in &nodes {
        if let Some(id) = node_id(node) {
            indegree.entry(id.clone()).or_insert(0);
            outgoing.entry(id).or_default();
        }
    }
    for edge in &connections {
        if let Some((from, to)) = connection_edge(edge) {
            outgoing.entry(from).or_default().push(to.clone());
            *indegree.entry(to).or_default() += 1;
        }
    }

    let mut queue = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<_>>();
    let mut levels = Vec::<Value>::new();
    let mut ordered = Vec::<String>::new();
    while !queue.is_empty() {
        let mut level = Vec::new();
        for _ in 0..queue.len() {
            if let Some(id) = queue.pop_front() {
                ordered.push(id.clone());
                level.push(id.clone());
                for to in outgoing.get(&id).into_iter().flatten() {
                    if let Some(degree) = indegree.get_mut(to) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push_back(to.clone());
                        }
                    }
                }
            }
        }
        level.sort();
        levels.push(json!({"level": levels.len(), "nodes": level}));
    }
    json!({
        "node_count": nodes.len(),
        "edge_count": connections.len(),
        "ordered_nodes": ordered,
        "levels": levels
    })
}

/// Suggest app/workflow nodes by intent, similar to workflow-builder MCP servers.
pub fn app_node_suggest_work(args: Value) -> Value {
    let query = text_arg(&args, "query")
        .or_else(|| text_arg(&args, "intent"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if query.trim().is_empty() {
        return json!({"error": "query or intent must be a non-empty string"});
    }
    let suggestions = node_suggestion_catalog()
        .into_iter()
        .filter(|item| {
            let haystack = format!("{} {} {} {}", item.0, item.1, item.2, item.3.join(" "))
                .to_ascii_lowercase();
            query.split_whitespace().any(|term| haystack.contains(term))
        })
        .take(25)
        .map(|(style, name, purpose, keywords)| {
            json!({
                "source_style": style,
                "name": name,
                "purpose": purpose,
                "keywords": keywords
            })
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.app.node_suggest.v1",
        "query": query,
        "suggestions": suggestions,
        "note": "local suggestions only; does not query or control external apps"
    })
}

/// Generate starter workflow graphs for common n8n-style patterns.
pub fn workflow_template_generate_work(args: Value) -> Value {
    let kind = text_arg(&args, "kind")
        .or_else(|| text_arg(&args, "template"))
        .unwrap_or_else(|| "manual-code-review".to_string())
        .to_ascii_lowercase();
    let workflow = match kind.as_str() {
        "webhook-approval" | "approval" => json!({
            "nodes": [
                {"id": "webhook", "type": "webhook", "purpose": "receive proposed action"},
                {"id": "validate", "type": "function", "purpose": "check payload and safety flags"},
                {"id": "approval_gate", "type": "manual_approval", "purpose": "require explicit human approval"},
                {"id": "respond", "type": "respond_to_webhook", "purpose": "return approved/rejected result"}
            ],
            "connections": [
                {"from": "webhook", "to": "validate"},
                {"from": "validate", "to": "approval_gate"},
                {"from": "approval_gate", "to": "respond"}
            ]
        }),
        "schedule-report" | "report" => json!({
            "nodes": [
                {"id": "schedule", "type": "schedule_trigger", "purpose": "run periodically"},
                {"id": "collect", "type": "data_collect", "purpose": "read local status/data"},
                {"id": "summarize", "type": "summarize", "purpose": "create compact report"},
                {"id": "notify", "type": "notify", "purpose": "send local notification"}
            ],
            "connections": [
                {"from": "schedule", "to": "collect"},
                {"from": "collect", "to": "summarize"},
                {"from": "summarize", "to": "notify"}
            ]
        }),
        "coding-ci" | "ci" => json!({
            "nodes": [
                {"id": "trigger", "type": "manual_or_git_trigger", "purpose": "start verification"},
                {"id": "status", "type": "git_status", "purpose": "inspect changed files"},
                {"id": "test", "type": "test_runner", "purpose": "run focused tests"},
                {"id": "review", "type": "code_review", "purpose": "summarize risks and failures"}
            ],
            "connections": [
                {"from": "trigger", "to": "status"},
                {"from": "status", "to": "test"},
                {"from": "test", "to": "review"}
            ]
        }),
        "data-pipeline" | "pipeline" => json!({
            "nodes": [
                {"id": "input", "type": "file_or_http_input", "purpose": "load records"},
                {"id": "filter", "type": "filter", "purpose": "drop noise/secrets"},
                {"id": "transform", "type": "transform", "purpose": "normalize schema"},
                {"id": "validate", "type": "validate", "purpose": "quality checks"},
                {"id": "output", "type": "write_or_report", "purpose": "produce result"}
            ],
            "connections": [
                {"from": "input", "to": "filter"},
                {"from": "filter", "to": "transform"},
                {"from": "transform", "to": "validate"},
                {"from": "validate", "to": "output"}
            ]
        }),
        _ => json!({
            "nodes": [
                {"id": "context", "type": "workspace_context", "purpose": "understand repo"},
                {"id": "search", "type": "grep_code", "purpose": "find relevant code"},
                {"id": "change_plan", "type": "agent_patch_plan", "purpose": "plan safe edits"},
                {"id": "verify", "type": "test_runner", "purpose": "run focused verification"}
            ],
            "connections": [
                {"from": "context", "to": "search"},
                {"from": "search", "to": "change_plan"},
                {"from": "change_plan", "to": "verify"}
            ]
        }),
    };
    let validation = workflow_validate_work(json!({"workflow": workflow.clone()}));
    let plan = workflow_execution_plan_work(json!({"workflow": workflow.clone()}));
    json!({
        "format": "pb-mcp-server.workflow.template_generate.v1",
        "kind": kind,
        "workflow": workflow,
        "validation": validation,
        "execution_plan": plan
    })
}

/// Build a structured task packet for coding-agent style applications.
pub fn agent_task_packet_work(args: Value) -> Value {
    let goal = args
        .get("goal")
        .and_then(|v| v.as_str())
        .unwrap_or("unspecified task")
        .trim();
    let app_style = args
        .get("app_style")
        .and_then(|v| v.as_str())
        .unwrap_or("codex")
        .trim();
    let context = string_array_or_empty(&args, "context");
    let files = string_array_or_empty(&args, "files");
    let constraints = string_array_or_empty(&args, "constraints");
    let verification = string_array_or_empty(&args, "verification");
    json!({
        "format": "pb-mcp-server.agent.task_packet.v1",
        "app_style": app_style,
        "goal": goal,
        "context": context,
        "files": files,
        "constraints": constraints,
        "suggested_steps": [
            "inspect relevant files and current state",
            "make the smallest coherent change",
            "run focused verification",
            "summarize changed files and residual risk"
        ],
        "verification": verification,
        "ready_for": ["codex", "claude-code", "opencode", "openclaw", "hermes"]
    })
}

/// Pack an agent prompt from task, context, constraints, and verification.
pub fn agent_prompt_pack_work(args: Value) -> Value {
    let goal = text_arg(&args, "goal")
        .or_else(|| text_arg(&args, "prompt"))
        .unwrap_or_else(|| "unspecified task".to_string());
    let mode = text_arg(&args, "mode").unwrap_or_else(|| "coding-agent".to_string());
    let context = string_array_or_empty(&args, "context");
    let files = string_array_or_empty(&args, "files");
    let constraints = string_array_or_empty(&args, "constraints");
    let verification = string_array_or_empty(&args, "verification");
    json!({
        "format": "pb-mcp-server.agent.prompt_pack.v1",
        "mode": mode,
        "system": "Act as a local coding agent. Inspect first, make scoped changes, preserve user work, and verify before final response.",
        "user": goal,
        "context": context,
        "files": files,
        "constraints": constraints,
        "verification": verification,
        "checklist": [
            "read relevant files",
            "identify minimal implementation path",
            "edit only scoped files",
            "run focused verification",
            "report changed files and residual risk"
        ]
    })
}

/// Summarize session-style logs from coding agent apps.
pub fn agent_session_summary_work(args: Value) -> Value {
    let messages = value_array_or_empty(&args, "messages")
        .into_iter()
        .chain(value_array_or_empty(&args, "events"))
        .take(2000)
        .collect::<Vec<_>>();
    if messages.is_empty() {
        return json!({"error": "messages or events must be a non-empty array"});
    }
    let mut decisions = Vec::new();
    let mut actions = Vec::new();
    let mut risks = Vec::new();
    let mut verification = Vec::new();
    for message in &messages {
        let text = message_text(message).to_ascii_lowercase();
        if text.contains("decid") || text.contains("choose") || text.contains("use ") {
            decisions.push(message_text(message));
        }
        if text.contains("todo") || text.contains("next") || text.contains("fix") {
            actions.push(message_text(message));
        }
        if text.contains("risk") || text.contains("fail") || text.contains("error") {
            risks.push(message_text(message));
        }
        if text.contains("test") || text.contains("check") || text.contains("verify") {
            verification.push(message_text(message));
        }
    }
    json!({
        "format": "pb-mcp-server.agent.session_summary.v1",
        "message_count": messages.len(),
        "decisions": limit_strings(decisions, 25),
        "actions": limit_strings(actions, 25),
        "risks": limit_strings(risks, 25),
        "verification": limit_strings(verification, 25),
        "summary": "deterministic keyword summary for local agent-session logs"
    })
}

/// Validate a planned patch/change set before running an agent.
pub fn agent_patch_plan_work(args: Value) -> Value {
    let goal = text_arg(&args, "goal").unwrap_or_else(|| "unspecified patch".to_string());
    let files = string_array_or_empty(&args, "files");
    let changes = string_array_or_empty(&args, "changes");
    let verification = string_array_or_empty(&args, "verification");
    let mut warnings = Vec::new();
    if files.is_empty() {
        warnings.push("no files listed; patch plan may be too vague".to_string());
    }
    if changes.is_empty() {
        warnings.push("no changes listed; add concrete edit intent".to_string());
    }
    if verification.is_empty() {
        warnings.push("no verification listed; add focused test/check command".to_string());
    }
    let risk_level = if files.len() > 20 || changes.len() > 20 {
        "high"
    } else if files.len() > 5 || verification.is_empty() {
        "medium"
    } else {
        "low"
    };
    json!({
        "format": "pb-mcp-server.agent.patch_plan.v1",
        "goal": goal,
        "files": files,
        "changes": changes,
        "verification": verification,
        "risk_level": risk_level,
        "warnings": warnings,
        "ready": warnings.is_empty()
    })
}

/// Command-palette suggestions inspired by agent app UIs.
pub fn app_command_palette_work(args: Value) -> Value {
    let query = text_arg(&args, "query")
        .or_else(|| text_arg(&args, "intent"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let commands = command_palette_catalog()
        .into_iter()
        .filter(|command| {
            if query.trim().is_empty() {
                return true;
            }
            let haystack =
                format!("{} {} {}", command.0, command.1, command.2.join(" ")).to_ascii_lowercase();
            query.split_whitespace().any(|term| haystack.contains(term))
        })
        .take(30)
        .map(|(name, description, tools)| {
            json!({
                "command": name,
                "description": description,
                "tool_sequence": tools
            })
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.app.command_palette.v1",
        "query": query,
        "commands": commands
    })
}

/// Validate an MCP-style manifest without running external servers.
pub fn mcp_manifest_validate_work(args: Value) -> Value {
    let manifest = args.get("manifest").unwrap_or(&args);
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let name = manifest
        .get("name")
        .or_else(|| manifest.get("server_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if name.trim().is_empty() {
        errors.push("manifest needs name or server_name".to_string());
    }
    let tools = manifest
        .get("tools")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if tools.is_empty() {
        errors.push("manifest.tools must be a non-empty array".to_string());
    }
    let mut seen = BTreeSet::<String>::new();
    for (index, tool) in tools.iter().enumerate() {
        let tool_name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if tool_name.trim().is_empty() {
            errors.push(format!("tools[{index}] needs name"));
        } else if !seen.insert(tool_name.to_string()) {
            errors.push(format!("duplicate tool name: {tool_name}"));
        }
        if tool
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            warnings.push(format!("tool {tool_name} has no description"));
        }
        if tool.get("input_schema").is_none() && tool.get("inputSchema").is_none() {
            warnings.push(format!("tool {tool_name} has no input schema"));
        }
        if tool.get("annotations").is_none() {
            warnings.push(format!("tool {tool_name} has no annotations"));
        }
    }
    json!({
        "format": "pb-mcp-server.mcp.manifest_validate.v1",
        "valid": errors.is_empty(),
        "server_name": name,
        "tool_count": tools.len(),
        "errors": errors,
        "warnings": warnings
    })
}

/// Generate a deterministic MCP-style tool spec skeleton.
pub fn mcp_tool_spec_generate_work(args: Value) -> Value {
    let name = text_arg(&args, "name").unwrap_or_else(|| "local.tool".to_string());
    let description = text_arg(&args, "description")
        .unwrap_or_else(|| "Describe what this tool does.".to_string());
    let read_only = args
        .get("read_only")
        .or_else(|| args.get("readOnly"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let properties = args
        .get("properties")
        .cloned()
        .unwrap_or_else(|| json!({"input": {"type": "object", "description": "Tool input"}}));
    let required = args
        .get("required")
        .cloned()
        .unwrap_or_else(|| json!(["input"]));
    json!({
        "format": "pb-mcp-server.mcp.tool_spec_generate.v1",
        "tool": {
            "name": name,
            "description": description,
            "input_schema": {
                "type": "object",
                "properties": properties,
                "required": required
            },
            "annotations": {
                "title": args.get("title").and_then(|v| v.as_str()).unwrap_or("Generated local tool"),
                "read_only_hint": read_only
            }
        }
    })
}

/// Validate a local skill manifest or instruction block.
pub fn app_skill_manifest_validate_work(args: Value) -> Value {
    let manifest = args.get("manifest").unwrap_or(&args);
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let name = manifest.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let description = manifest
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let instructions_text = manifest
        .get("instructions")
        .or_else(|| manifest.get("body"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let instructions_array_len = manifest
        .get("instructions")
        .and_then(|v| v.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    if name.trim().is_empty() {
        errors.push("skill needs name".to_string());
    }
    if description.trim().is_empty() {
        warnings.push("skill has no description".to_string());
    }
    if instructions_text.trim().is_empty() && instructions_array_len == 0 {
        errors.push("skill needs instructions or body".to_string());
    }
    if instructions_text.len() > 50_000 || instructions_array_len > 200 {
        warnings.push("skill instructions are large; consider splitting".to_string());
    }
    let tools = string_array_or_empty(manifest, "tools");
    json!({
        "format": "pb-mcp-server.app.skill_manifest_validate.v1",
        "valid": errors.is_empty(),
        "name": name,
        "tool_count": tools.len(),
        "errors": errors,
        "warnings": warnings
    })
}

/// Generate a deterministic local skill skeleton.
pub fn app_skill_template_generate_work(args: Value) -> Value {
    let name = text_arg(&args, "name").unwrap_or_else(|| "local-skill".to_string());
    let kind = text_arg(&args, "kind")
        .unwrap_or_else(|| "coding".to_string())
        .to_ascii_lowercase();
    let tools = if kind.contains("security") {
        vec!["workspace_context", "grep_code", "review_focused_review"]
    } else if kind.contains("workflow") || kind.contains("n8n") {
        vec![
            "app_node_suggest",
            "workflow_template_generate",
            "workflow_validate",
        ]
    } else if kind.contains("plugin") {
        vec![
            "app_plugin_manifest_validate",
            "app_plugin_route_plan",
            "app_command_palette",
        ]
    } else {
        vec!["workspace_context", "find_files", "grep_code"]
    };
    json!({
        "format": "pb-mcp-server.app.skill_template_generate.v1",
        "skill": {
            "name": name,
            "description": format!("{kind} helper skill"),
            "triggers": [kind],
            "tools": tools,
            "instructions": [
                "Read relevant context first.",
                "Keep changes scoped to the requested goal.",
                "Run focused verification before reporting completion."
            ]
        }
    })
}

/// Validate local plugin metadata without installing or loading a plugin.
pub fn app_plugin_manifest_validate_work(args: Value) -> Value {
    let manifest = args.get("manifest").unwrap_or(&args);
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let name = manifest.get("name").and_then(|v| v.as_str()).unwrap_or("");
    if name.trim().is_empty() {
        errors.push("plugin needs name".to_string());
    }
    if manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        warnings.push("plugin has no version".to_string());
    }
    let mut seen = BTreeSet::<String>::new();
    for field in ["tools", "skills", "commands"] {
        for (index, item) in value_array_or_empty(manifest, field).iter().enumerate() {
            let id = item
                .get("name")
                .or_else(|| item.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if id.trim().is_empty() {
                errors.push(format!("{field}[{index}] needs name or id"));
            } else if !seen.insert(format!("{field}:{id}")) {
                errors.push(format!("duplicate {field} entry: {id}"));
            }
        }
    }
    json!({
        "format": "pb-mcp-server.app.plugin_manifest_validate.v1",
        "valid": errors.is_empty(),
        "name": name,
        "tool_count": value_array_or_empty(manifest, "tools").len(),
        "skill_count": value_array_or_empty(manifest, "skills").len(),
        "command_count": value_array_or_empty(manifest, "commands").len(),
        "errors": errors,
        "warnings": warnings
    })
}

/// Route a user intent through available plugin-like surfaces.
pub fn app_plugin_route_plan_work(args: Value) -> Value {
    let intent = text_arg(&args, "intent")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if intent.trim().is_empty() {
        return json!({"error": "intent must be a non-empty string"});
    }
    let mut route = Vec::<Value>::new();
    if intent.contains("workflow") || intent.contains("n8n") {
        route.push(json!({"surface": "skill", "name": "workflow-builder"}));
        route.push(json!({"surface": "tool", "name": "workflow_template_generate"}));
        route.push(json!({"surface": "tool", "name": "workflow_validate"}));
    } else if intent.contains("plugin") {
        route.push(json!({"surface": "tool", "name": "app_plugin_manifest_validate"}));
        route.push(json!({"surface": "tool", "name": "app_command_palette"}));
    } else if intent.contains("skill") {
        route.push(json!({"surface": "tool", "name": "app_skill_template_generate"}));
        route.push(json!({"surface": "tool", "name": "app_skill_manifest_validate"}));
    } else if intent.contains("review") || intent.contains("bug") {
        route.push(json!({"surface": "command", "name": "/review"}));
        route.push(json!({"surface": "tool", "name": "review_focused_review"}));
    } else {
        route.push(json!({"surface": "command", "name": "/plan"}));
        route.push(json!({"surface": "tool", "name": "app_tool_recipe"}));
    }
    json!({
        "format": "pb-mcp-server.app.plugin_route_plan.v1",
        "intent": intent,
        "route": route,
        "note": "local routing plan only; no plugin is loaded or executed"
    })
}

/// Expand slash-style commands into local pb-mcp tool sequences.
pub fn app_slash_command_expand_work(args: Value) -> Value {
    let command = text_arg(&args, "command")
        .or_else(|| text_arg(&args, "input"))
        .unwrap_or_default();
    if command.trim().is_empty() {
        return json!({"error": "command or input must be a non-empty string"});
    }
    let head = command
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_start_matches('/')
        .to_ascii_lowercase();
    let tools = match head.as_str() {
        "review" => vec!["workspace_context", "git_diff", "review_focused_review"],
        "fix" => vec!["workspace_context", "grep_code", "agent_patch_plan"],
        "test" => vec!["workspace_context", "project_cargo_test", "project_pytest"],
        "plan" => vec!["agent_task_packet", "app_tool_recipe"],
        "workflow" => vec![
            "app_node_suggest",
            "workflow_template_generate",
            "workflow_validate",
        ],
        "mcp" => vec!["mcp_manifest_validate", "mcp_tool_spec_generate"],
        "skill" => vec!["app_skill_template_generate", "app_skill_manifest_validate"],
        "plugin" => vec!["app_plugin_manifest_validate", "app_plugin_route_plan"],
        _ => vec!["app_command_palette", "app_tool_recipe"],
    };
    json!({
        "format": "pb-mcp-server.app.slash_command_expand.v1",
        "command": command,
        "normalized": head,
        "tool_sequence": tools
    })
}

/// Validate hook definitions before they are wired into an agent runtime.
pub fn app_hooks_policy_validate_work(args: Value) -> Value {
    let hooks = value_array_or_empty(&args, "hooks");
    if hooks.is_empty() {
        return json!({"error": "hooks must be a non-empty array"});
    }
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    for (index, hook) in hooks.iter().enumerate() {
        let event = hook.get("event").and_then(|v| v.as_str()).unwrap_or("");
        if event.trim().is_empty() {
            errors.push(format!("hooks[{index}] needs event"));
        }
        let action = hook
            .get("action")
            .or_else(|| hook.get("command"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if action.trim().is_empty() {
            errors.push(format!("hooks[{index}] needs action or command"));
        }
        let action_lower = action.to_ascii_lowercase();
        if action_lower.contains("rm ")
            || action_lower.contains("curl ")
            || action_lower.contains("wget ")
            || action_lower.contains("sudo ")
        {
            warnings.push(format!(
                "hooks[{index}] action appears risky and should require explicit approval"
            ));
        }
    }
    json!({
        "format": "pb-mcp-server.app.hooks_policy_validate.v1",
        "valid": errors.is_empty(),
        "hook_count": hooks.len(),
        "errors": errors,
        "warnings": warnings
    })
}

/// Evaluate a proposed action against a local permission profile.
pub fn app_permission_policy_evaluate_work(args: Value) -> Value {
    let profile = text_arg(&args, "profile")
        .unwrap_or_else(|| "core-code".to_string())
        .to_ascii_lowercase();
    let action = text_arg(&args, "action")
        .or_else(|| text_arg(&args, "tool"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    let writes = args
        .get("writes")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || action.contains("write")
        || action.contains("edit")
        || action.contains("commit");
    let executes = args
        .get("executes")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || action.contains("run")
        || action.contains("exec")
        || action.contains("terminal");
    let network = args
        .get("network")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || action.contains("web")
        || action.contains("http");
    let allowed = match profile.as_str() {
        "public-safe" => !writes && !executes && !network,
        "core-code" | "review-refactor" => !executes && !network,
        "agent-orchestration" => !executes && !network,
        "local-power" => true,
        "experimental-lab" => !writes,
        _ => !writes && !executes && !network,
    };
    json!({
        "format": "pb-mcp-server.app.permission_policy_evaluate.v1",
        "profile": profile,
        "action": action,
        "signals": {"writes": writes, "executes": executes, "network": network},
        "allowed": allowed,
        "decision": if allowed { "allow" } else { "needs_explicit_approval_or_stronger_profile" }
    })
}

/// Select a local model/runtime profile from task constraints.
pub fn app_model_profile_select_work(args: Value) -> Value {
    let task = text_arg(&args, "task").unwrap_or_else(|| "general".to_string());
    let privacy = text_arg(&args, "privacy").unwrap_or_else(|| "local".to_string());
    let latency = text_arg(&args, "latency").unwrap_or_else(|| "balanced".to_string());
    let coding = task.to_ascii_lowercase().contains("code")
        || task.to_ascii_lowercase().contains("bug")
        || task.to_ascii_lowercase().contains("repo");
    let profile = if privacy == "local" && latency == "fast" {
        "local-small-fast"
    } else if coding {
        "coding-strong"
    } else if task.to_ascii_lowercase().contains("reason") {
        "reasoning-strong"
    } else {
        "balanced-general"
    };
    json!({
        "format": "pb-mcp-server.app.model_profile_select.v1",
        "task": task,
        "privacy": privacy,
        "latency": latency,
        "selected_profile": profile,
        "runtime_notes": [
            "selection is advisory only",
            "caller still chooses the actual model/runtime"
        ]
    })
}

/// Rank and pack context items into a token budget.
pub fn app_context_budget_plan_work(args: Value) -> Value {
    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(8_000)
        .min(200_000) as usize;
    let mut items = value_array_or_empty(&args, "items")
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let name = item
                .get("name")
                .or_else(|| item.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("item")
                .to_string();
            let priority = item.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            let tokens = item
                .get("tokens")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or_else(|| {
                    item.get("text")
                        .and_then(|v| v.as_str())
                        .map(rough_token_count)
                        .unwrap_or(256)
                });
            (index, name, priority, tokens)
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
    let mut used = 0usize;
    let mut selected = Vec::new();
    let mut omitted = Vec::new();
    for (_, name, priority, tokens) in items {
        if used + tokens <= max_tokens {
            used += tokens;
            selected.push(json!({"name": name, "priority": priority, "tokens": tokens}));
        } else {
            omitted.push(json!({"name": name, "priority": priority, "tokens": tokens}));
        }
    }
    json!({
        "format": "pb-mcp-server.app.context_budget_plan.v1",
        "max_tokens": max_tokens,
        "used_tokens": used,
        "selected": selected,
        "omitted": omitted
    })
}

/// Plan dependent tasks into executable levels.
pub fn app_task_queue_plan_work(args: Value) -> Value {
    let tasks = value_array_or_empty(&args, "tasks");
    if tasks.is_empty() {
        return json!({"error": "tasks must be a non-empty array"});
    }
    let mut nodes = Vec::new();
    let mut connections = Vec::new();
    for task in &tasks {
        let id = task
            .get("id")
            .or_else(|| task.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if id.trim().is_empty() {
            continue;
        }
        nodes.push(json!({"id": id, "type": "task"}));
        for dep in string_array_or_empty(task, "deps")
            .into_iter()
            .chain(string_array_or_empty(task, "depends_on"))
        {
            connections.push(json!({"from": dep, "to": id}));
        }
    }
    let workflow = json!({"nodes": nodes, "connections": connections});
    let validation = workflow_validate_work(json!({"workflow": workflow.clone()}));
    if !validation["valid"].as_bool().unwrap_or(false) {
        return json!({"error": "task queue is invalid", "validation": validation});
    }
    let plan = workflow_execution_plan_work(json!({"workflow": workflow}));
    json!({
        "format": "pb-mcp-server.app.task_queue_plan.v1",
        "task_count": tasks.len(),
        "execution_plan": plan
    })
}

/// Generate a deterministic MCP server config skeleton.
pub fn mcp_server_config_generate_work(args: Value) -> Value {
    let name = text_arg(&args, "name").unwrap_or_else(|| "local-pb-mcp".to_string());
    let transport = text_arg(&args, "transport")
        .unwrap_or_else(|| "stdio".to_string())
        .to_ascii_lowercase();
    let config = if transport == "http" || transport == "sse" {
        json!({
            "name": name,
            "transport": transport,
            "url": text_arg(&args, "url").unwrap_or_else(|| "http://localhost:3000/mcp".to_string()),
            "headers": args.get("headers").cloned().unwrap_or_else(|| json!({}))
        })
    } else {
        json!({
            "name": name,
            "transport": "stdio",
            "command": text_arg(&args, "command").unwrap_or_else(|| "pb-mcp-server".to_string()),
            "args": string_array_or_empty(&args, "args"),
            "env": args.get("env").cloned().unwrap_or_else(|| json!({}))
        })
    };
    json!({
        "format": "pb-mcp-server.mcp.server_config_generate.v1",
        "config": config,
        "note": "config skeleton only; does not write files or start a server"
    })
}

/// Validate an MCP-style prompt template with `{variable}` placeholders.
pub fn mcp_prompt_template_validate_work(args: Value) -> Value {
    let template = text_arg(&args, "template")
        .or_else(|| text_arg(&args, "prompt"))
        .unwrap_or_default();
    if template.trim().is_empty() {
        return json!({"valid": false, "errors": ["template or prompt must be non-empty"]});
    }
    let declared = string_array_or_empty(&args, "variables")
        .into_iter()
        .collect::<BTreeSet<_>>();
    let used = extract_braced_names(&template);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for name in &declared {
        if !is_identifier_like(name) {
            errors.push(format!("invalid variable name: {name}"));
        }
        if !used.contains(name) {
            warnings.push(format!("declared variable not used: {name}"));
        }
    }
    for name in &used {
        if !declared.is_empty() && !declared.contains(name) {
            warnings.push(format!("template uses undeclared variable: {name}"));
        }
    }
    json!({
        "format": "pb-mcp-server.mcp.prompt_template_validate.v1",
        "valid": errors.is_empty(),
        "used_variables": used,
        "declared_variables": declared,
        "errors": errors,
        "warnings": warnings
    })
}

/// Validate an MCP-style resource template URI.
pub fn mcp_resource_template_validate_work(args: Value) -> Value {
    let uri_template = text_arg(&args, "uri_template")
        .or_else(|| text_arg(&args, "uriTemplate"))
        .or_else(|| text_arg(&args, "template"))
        .unwrap_or_default();
    if uri_template.trim().is_empty() {
        return json!({"valid": false, "errors": ["uri_template must be non-empty"]});
    }
    let variables = extract_braced_names(&uri_template);
    let mut errors = Vec::new();
    if !uri_template.contains("://") && !uri_template.starts_with("file:") {
        errors.push("resource template should include a URI scheme".to_string());
    }
    for name in &variables {
        if !is_identifier_like(name) {
            errors.push(format!("invalid placeholder name: {name}"));
        }
    }
    json!({
        "format": "pb-mcp-server.mcp.resource_template_validate.v1",
        "valid": errors.is_empty(),
        "uri_template": uri_template,
        "variables": variables,
        "errors": errors
    })
}

/// Plan an MCP sampling-style request without calling a model.
pub fn mcp_sampling_request_plan_work(args: Value) -> Value {
    let prompt = text_arg(&args, "prompt").unwrap_or_default();
    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(1024)
        .min(64_000);
    let temperature = args
        .get("temperature")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.2)
        .clamp(0.0, 2.0);
    let lower = prompt.to_ascii_lowercase();
    let model_profile = if lower.contains("code") || lower.contains("bug") || lower.contains("test")
    {
        "coding-strong"
    } else {
        "balanced-general"
    };
    json!({
        "format": "pb-mcp-server.mcp.sampling_request_plan.v1",
        "request": {
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": max_tokens,
            "temperature": temperature,
            "model_profile": model_profile
        },
        "note": "sampling request plan only; no model/provider call is made"
    })
}

/// Generate a command registry for app-style slash commands.
pub fn app_command_registry_generate_work(args: Value) -> Value {
    let include_experimental = args
        .get("include_experimental")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let mut commands = command_palette_catalog()
        .into_iter()
        .map(|(name, description, tools)| {
            let slug = name.to_ascii_lowercase().replace(' ', "_").replace('/', "");
            json!({
                "name": format!("/{slug}"),
                "title": name,
                "description": description,
                "tool_sequence": tools
            })
        })
        .collect::<Vec<_>>();
    if include_experimental {
        commands.push(json!({
            "name": "/lab",
            "title": "Experimental lab",
            "description": "Explore experimental local tools",
            "tool_sequence": ["tools_runtime_status", "app_capability_search"]
        }));
    }
    json!({
        "format": "pb-mcp-server.app.command_registry_generate.v1",
        "commands": commands
    })
}

/// Match a request to the most relevant local skills.
pub fn app_skill_route_match_work(args: Value) -> Value {
    let query = text_arg(&args, "query")
        .or_else(|| text_arg(&args, "intent"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if query.trim().is_empty() {
        return json!({"error": "query or intent must be non-empty"});
    }
    let matches = value_array_or_empty(&args, "skills")
        .iter()
        .map(|skill| {
            let name = skill.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let description = skill
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let triggers = string_array_or_empty(skill, "triggers");
            let haystack =
                format!("{name} {description} {}", triggers.join(" ")).to_ascii_lowercase();
            let score = query
                .split_whitespace()
                .filter(|term| haystack.contains(term))
                .count();
            json!({"name": name, "score": score, "description": description})
        })
        .filter(|item| item["score"].as_u64().unwrap_or(0) > 0)
        .take(25)
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.app.skill_route_match.v1",
        "query": query,
        "matches": matches
    })
}

/// Build a capability matrix for plugin-like bundles.
pub fn app_plugin_capability_matrix_work(args: Value) -> Value {
    let plugins = value_array_or_empty(&args, "plugins");
    if plugins.is_empty() {
        return json!({"error": "plugins must be a non-empty array"});
    }
    let matrix = plugins
        .iter()
        .map(|plugin| {
            let name = plugin
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("plugin");
            json!({
                "name": name,
                "tools": value_array_or_empty(plugin, "tools").len(),
                "skills": value_array_or_empty(plugin, "skills").len(),
                "commands": value_array_or_empty(plugin, "commands").len(),
                "hooks": value_array_or_empty(plugin, "hooks").len(),
                "has_permissions": plugin.get("permissions").is_some()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.app.plugin_capability_matrix.v1",
        "plugins": matrix
    })
}

/// Plan an app agent mode from common coding-agent modes.
pub fn app_agent_mode_plan_work(args: Value) -> Value {
    let mode = text_arg(&args, "mode")
        .unwrap_or_else(|| "ask".to_string())
        .to_ascii_lowercase();
    let plan = match mode.as_str() {
        "ask" | "chat" => json!({
            "permissions": "read-only",
            "tools": ["workspace_context", "grep_code", "app_tool_recipe"],
            "verification": "explain only unless user asks for edits"
        }),
        "edit" | "agent" => json!({
            "permissions": "workspace-write",
            "tools": ["workspace_context", "grep_code", "agent_patch_plan"],
            "verification": "run focused tests after edits"
        }),
        "review" => json!({
            "permissions": "read-only",
            "tools": ["git_diff", "review_focused_review", "review_missing_tests"],
            "verification": "report findings before summary"
        }),
        "workflow" => json!({
            "permissions": "read-only",
            "tools": ["app_node_suggest", "workflow_template_generate", "workflow_validate"],
            "verification": "validate generated graph"
        }),
        _ => json!({
            "permissions": "least-privilege",
            "tools": ["tools_runtime_status", "app_command_palette"],
            "verification": "choose explicit mode before risky actions"
        }),
    };
    json!({
        "format": "pb-mcp-server.app.agent_mode_plan.v1",
        "mode": mode,
        "plan": plan
    })
}

/// Plan an isolated worktree/session style coding task.
pub fn app_worktree_session_plan_work(args: Value) -> Value {
    let goal = text_arg(&args, "goal").unwrap_or_else(|| "unspecified task".to_string());
    let branch = text_arg(&args, "branch").unwrap_or_else(|| slugify_like(&goal));
    json!({
        "format": "pb-mcp-server.app.worktree_session_plan.v1",
        "goal": goal,
        "branch": branch,
        "files": string_array_or_empty(&args, "files"),
        "steps": [
            "check current git status",
            "create or select isolated branch/worktree if requested by user",
            "make scoped edits",
            "run focused verification",
            "summarize changes and remaining risk"
        ],
        "note": "planning only; does not create a branch or worktree"
    })
}

/// Pack regression/evaluation prompts for app or agent testing.
pub fn app_eval_prompt_pack_work(args: Value) -> Value {
    let prompts = value_array_or_empty(&args, "prompts");
    if prompts.is_empty() {
        return json!({"error": "prompts must be a non-empty array"});
    }
    let packed = prompts
        .iter()
        .take(200)
        .enumerate()
        .map(|(index, prompt)| {
            let text = prompt
                .get("text")
                .or_else(|| prompt.get("prompt"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| prompt.as_str().unwrap_or(""));
            json!({
                "id": prompt.get("id").and_then(|v| v.as_str()).map(str::to_string).unwrap_or_else(|| format!("eval-{index}")),
                "prompt": text,
                "expected_signals": string_array_or_empty(prompt, "expected_signals"),
                "tokens_estimate": rough_token_count(text)
            })
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.app.eval_prompt_pack.v1",
        "count": packed.len(),
        "prompts": packed
    })
}

/// Catalog common shell commands across bash, PowerShell, batch, and zsh/macOS Terminal.
pub fn shell_command_catalog_work(args: Value) -> Value {
    let family = text_arg(&args, "family")
        .unwrap_or_else(|| "all".to_string())
        .to_ascii_lowercase();
    let commands = shell_catalog()
        .into_iter()
        .filter(|entry| family == "all" || entry.0 == family)
        .map(|(shell, command, purpose, risk)| {
            json!({"shell": shell, "command": command, "purpose": purpose, "risk": risk})
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.shell.command_catalog.v1",
        "family": family,
        "commands": commands,
        "note": "catalog only; does not execute commands"
    })
}

/// Translate common command intent between shell families.
pub fn shell_command_translate_work(args: Value) -> Value {
    let intent = text_arg(&args, "intent")
        .or_else(|| text_arg(&args, "command"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if intent.trim().is_empty() {
        return json!({"error": "intent or command must be non-empty"});
    }
    let matches = shell_translation_table()
        .into_iter()
        .filter(|row| row.0.iter().any(|term| intent.contains(term)))
        .take(10)
        .map(|row| {
            json!({
                "intent": row.1,
                "bash": row.2,
                "powershell": row.3,
                "batch": row.4,
                "macos_zsh": row.5
            })
        })
        .collect::<Vec<_>>();
    json!({
        "format": "pb-mcp-server.shell.command_translate.v1",
        "query": intent,
        "translations": matches
    })
}

/// Generate safe starter script templates for common shell families.
pub fn shell_script_template_generate_work(args: Value) -> Value {
    let shell = text_arg(&args, "shell")
        .unwrap_or_else(|| "bash".to_string())
        .to_ascii_lowercase();
    let name = text_arg(&args, "name").unwrap_or_else(|| "task".to_string());
    let body = match shell.as_str() {
        "powershell" | "pwsh" => format!(
            "param()\nSet-StrictMode -Version Latest\n$ErrorActionPreference = 'Stop'\nWrite-Host \"Running {name}\"\n"
        ),
        "batch" | "cmd" => format!("@echo off\r\nsetlocal enabledelayedexpansion\r\necho Running {name}\r\n"),
        "zsh" | "macos" => format!("#!/usr/bin/env zsh\nset -euo pipefail\nprint -r -- \"Running {name}\"\n"),
        _ => format!("#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' 'Running {name}'\n"),
    };
    json!({
        "format": "pb-mcp-server.shell.script_template_generate.v1",
        "shell": shell,
        "script": body,
        "note": "template only; does not write or execute scripts"
    })
}

/// Classify shell command risk before using terminal execution tools.
pub fn shell_command_safety_check_work(args: Value) -> Value {
    let command = text_arg(&args, "command").unwrap_or_default();
    if command.trim().is_empty() {
        return json!({"error": "command must be non-empty"});
    }
    let lower = command.to_ascii_lowercase();
    let mut warnings = Vec::new();
    for pattern in [
        "rm -rf",
        "del /",
        "format ",
        "sudo ",
        "curl ",
        "wget ",
        "invoke-webrequest",
        "iex",
        "eval ",
        "chmod 777",
        "> /dev/",
    ] {
        if lower.contains(pattern) {
            warnings.push(format!("contains risky pattern: {pattern}"));
        }
    }
    let risk = if warnings
        .iter()
        .any(|w| w.contains("rm -rf") || w.contains("format "))
    {
        "high"
    } else if warnings.is_empty() {
        "low"
    } else {
        "medium"
    };
    json!({
        "format": "pb-mcp-server.shell.command_safety_check.v1",
        "command": command,
        "risk": risk,
        "warnings": warnings,
        "recommendation": if risk == "low" { "eligible for normal terminal policy checks" } else { "requires explicit review before execution" }
    })
}

/// Map a high-level intent to a practical local pb-mcp tool sequence.
pub fn app_tool_recipe_work(args: Value) -> Value {
    let intent = args
        .get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if intent.trim().is_empty() {
        return json!({"error": "intent must be a non-empty string"});
    }
    let recipe = if intent.contains("workflow") || intent.contains("n8n") {
        vec![
            "app_capability_search",
            "workflow_validate",
            "workflow_execution_plan",
            "tools_runtime_status",
        ]
    } else if intent.contains("review") || intent.contains("bug") {
        vec![
            "workspace_context",
            "git_diff",
            "review_focused_review",
            "review_missing_tests",
        ]
    } else if intent.contains("plugin") || intent.contains("skill") || intent.contains("command") {
        vec![
            "app_command_palette",
            "app_skill_manifest_validate",
            "app_plugin_manifest_validate",
        ]
    } else if intent.contains("agent") || intent.contains("task") {
        vec![
            "agent_task_packet",
            "agent_coordination_playbook",
            "agent_subagent_task",
            "agent_inbox_wait",
        ]
    } else if intent.contains("code") || intent.contains("repo") {
        vec![
            "workspace_context",
            "find_files",
            "grep_code",
            "code_index_search",
        ]
    } else {
        vec![
            "workspace_context",
            "tools_runtime_status",
            "app_capability_search",
        ]
    };
    json!({
        "intent": intent,
        "recipe": recipe,
        "notes": "Use as a local pb-mcp tool sequence; this does not call external apps."
    })
}

fn text_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn node_id(node: &Value) -> Option<String> {
    node.get("id")
        .or_else(|| node.get("name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

fn connection_edge(edge: &Value) -> Option<(String, String)> {
    let from = edge
        .get("from")
        .or_else(|| edge.get("source"))
        .and_then(|v| v.as_str())?;
    let to = edge
        .get("to")
        .or_else(|| edge.get("target"))
        .and_then(|v| v.as_str())?;
    Some((from.to_string(), to.to_string()))
}

fn detect_cycle(ids: &BTreeSet<String>, connections: &[Value]) -> bool {
    let mut indegree = ids
        .iter()
        .map(|id| (id.clone(), 0usize))
        .collect::<HashMap<_, _>>();
    let mut outgoing = ids
        .iter()
        .map(|id| (id.clone(), Vec::<String>::new()))
        .collect::<HashMap<_, _>>();
    for edge in connections {
        if let Some((from, to)) = connection_edge(edge) {
            if ids.contains(&from) && ids.contains(&to) {
                outgoing.entry(from).or_default().push(to.clone());
                *indegree.entry(to).or_default() += 1;
            }
        }
    }
    let mut queue = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<_>>();
    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        for to in outgoing.get(&id).into_iter().flatten() {
            if let Some(degree) = indegree.get_mut(to) {
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    queue.push_back(to.clone());
                }
            }
        }
    }
    visited != ids.len()
}

fn value_array_or_empty(args: &Value, key: &str) -> Vec<Value> {
    args.get(key)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn message_text(message: &Value) -> String {
    message
        .get("text")
        .or_else(|| message.get("content"))
        .or_else(|| message.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| message.as_str().unwrap_or(""))
        .trim()
        .to_string()
}

fn limit_strings(items: Vec<String>, limit: usize) -> Vec<String> {
    items
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .take(limit)
        .collect()
}

fn rough_token_count(text: &str) -> usize {
    text.split_whitespace().count().max(text.len() / 4).max(1)
}

fn extract_braced_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('}') else {
            break;
        };
        let name = rest[..end].trim();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
        rest = &rest[end + 1..];
    }
    names
}

fn is_identifier_like(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn slugify_like(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').chars().take(60).collect()
}

fn node_suggestion_catalog() -> Vec<(&'static str, &'static str, &'static str, Vec<&'static str>)> {
    vec![
        (
            "n8n",
            "manual_trigger",
            "start a workflow manually",
            vec!["manual", "start", "trigger"],
        ),
        (
            "n8n",
            "schedule_trigger",
            "run on a schedule",
            vec!["cron", "schedule", "timer"],
        ),
        (
            "n8n",
            "webhook",
            "receive HTTP events",
            vec!["webhook", "http", "api"],
        ),
        (
            "n8n",
            "respond_to_webhook",
            "send an HTTP response",
            vec!["respond", "webhook", "http"],
        ),
        (
            "n8n",
            "function",
            "transform data with code",
            vec!["code", "transform", "javascript"],
        ),
        (
            "n8n",
            "if",
            "branch on conditions",
            vec!["condition", "if", "branch"],
        ),
        (
            "n8n",
            "merge",
            "merge multiple branches",
            vec!["merge", "join", "combine"],
        ),
        (
            "n8n",
            "http_request",
            "call an HTTP API",
            vec!["http", "api", "request"],
        ),
        (
            "n8n",
            "ssh",
            "run command through SSH",
            vec!["ssh", "remote", "command"],
        ),
        (
            "mcp",
            "tool_manifest",
            "describe MCP server tools",
            vec!["manifest", "mcp", "schema"],
        ),
        (
            "mcp",
            "resource_template",
            "describe parameterized resources",
            vec!["resource", "template", "mcp"],
        ),
        (
            "codex",
            "workspace_context",
            "inspect repo state",
            vec!["repo", "context", "workspace"],
        ),
        (
            "codex",
            "apply_patch",
            "apply scoped patch",
            vec!["patch", "edit", "diff"],
        ),
        (
            "claude-code",
            "task_plan",
            "plan coding steps",
            vec!["plan", "task", "agent"],
        ),
        (
            "opencode",
            "command_palette",
            "map commands to actions",
            vec!["command", "palette", "action"],
        ),
        (
            "openclaw",
            "multi_agent_fanout",
            "parallelize agent work",
            vec!["agent", "fanout", "parallel"],
        ),
        (
            "hermes",
            "message_route",
            "route agent messages",
            vec!["message", "route", "agent"],
        ),
        (
            "codex",
            "skill",
            "load task-specific instructions",
            vec!["skill", "instructions", "guide"],
        ),
        (
            "codex",
            "plugin",
            "bundle tools, skills, and commands",
            vec!["plugin", "bundle", "extension"],
        ),
        (
            "claude-code",
            "slash_command",
            "expand command into agent action",
            vec!["slash", "command", "prompt"],
        ),
        (
            "opencode",
            "permission_policy",
            "decide whether an action needs approval",
            vec!["permission", "approval", "policy"],
        ),
    ]
}

fn command_palette_catalog() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    vec![
        (
            "Review current diff",
            "Inspect code changes and find risks",
            vec!["workspace_context", "git_diff", "review_focused_review"],
        ),
        (
            "Find relevant code",
            "Search files and symbols before editing",
            vec!["find_files", "grep_code", "code_index_search"],
        ),
        (
            "Build workflow",
            "Validate and plan an n8n-style workflow",
            vec![
                "app_node_suggest",
                "workflow_template_generate",
                "workflow_validate",
            ],
        ),
        (
            "Prepare agent task",
            "Create a task packet for a coding-agent app",
            vec!["agent_task_packet", "agent_prompt_pack"],
        ),
        (
            "Validate MCP manifest",
            "Check a server/tool manifest for missing metadata",
            vec!["mcp_manifest_validate", "mcp_tool_spec_generate"],
        ),
        (
            "Create skill",
            "Generate and validate a local skill skeleton",
            vec!["app_skill_template_generate", "app_skill_manifest_validate"],
        ),
        (
            "Check plugin",
            "Validate plugin metadata and route an intent",
            vec!["app_plugin_manifest_validate", "app_plugin_route_plan"],
        ),
        (
            "Expand slash command",
            "Map slash commands to pb-mcp tool sequences",
            vec!["app_slash_command_expand", "app_tool_recipe"],
        ),
        (
            "Evaluate permission",
            "Check whether an action fits a profile",
            vec!["app_permission_policy_evaluate"],
        ),
        (
            "Plan context",
            "Fit selected context into a token budget",
            vec!["app_context_budget_plan"],
        ),
        (
            "Plan task queue",
            "Order dependent tasks into execution levels",
            vec!["app_task_queue_plan"],
        ),
        (
            "Generate MCP config",
            "Create stdio/http MCP config skeletons",
            vec!["mcp_server_config_generate"],
        ),
        (
            "Validate prompt template",
            "Check MCP-style prompt variables",
            vec!["mcp_prompt_template_validate"],
        ),
        (
            "Plan worktree session",
            "Prepare isolated branch/worktree style work",
            vec!["app_worktree_session_plan"],
        ),
        (
            "Pack eval prompts",
            "Normalize regression prompts for app/agent checks",
            vec!["app_eval_prompt_pack"],
        ),
        (
            "Shell command helper",
            "Catalog, translate, and safety-check shell commands",
            vec![
                "shell_command_catalog",
                "shell_command_translate",
                "shell_command_safety_check",
            ],
        ),
        (
            "Run focused tests",
            "Select and run project verification",
            vec!["workspace_context", "project_cargo_test", "project_pytest"],
        ),
        (
            "Summarize session",
            "Summarize agent logs into decisions and next actions",
            vec!["agent_session_summary", "plan_handoff_brief"],
        ),
        (
            "Coordinate agents",
            "Prepare fan-out or pipeline coordination",
            vec![
                "agent_coordination_playbook",
                "agent_fan_out",
                "agent_send_pipeline",
            ],
        ),
    ]
}

fn shell_catalog() -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    vec![
        ("bash", "ls -la", "list files with details", "low"),
        (
            "bash",
            "find . -name '*.rs'",
            "find files by pattern",
            "low",
        ),
        ("bash", "grep -R 'text' .", "search text recursively", "low"),
        ("bash", "cp src dst", "copy files", "medium"),
        ("bash", "rm path", "remove files", "high"),
        ("powershell", "Get-ChildItem", "list files", "low"),
        (
            "powershell",
            "Select-String -Pattern text -Path *",
            "search text",
            "low",
        ),
        ("powershell", "Copy-Item src dst", "copy files", "medium"),
        ("powershell", "Remove-Item path", "remove files", "high"),
        ("batch", "dir", "list files", "low"),
        ("batch", "findstr /s text *", "search text", "low"),
        ("batch", "copy src dst", "copy files", "medium"),
        ("batch", "del path", "delete files", "high"),
        ("macos", "ls -la", "list files with details", "low"),
        ("macos", "mdfind query", "Spotlight search", "low"),
        ("macos", "open path", "open file or app", "medium"),
        ("macos", "rm path", "remove files", "high"),
    ]
}

type ShellTranslationRow = (
    Vec<&'static str>,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

fn shell_translation_table() -> Vec<ShellTranslationRow> {
    vec![
        (
            vec!["list", "dir", "files"],
            "list files",
            "ls -la",
            "Get-ChildItem -Force",
            "dir",
            "ls -la",
        ),
        (
            vec!["search", "grep", "find text"],
            "search text",
            "grep -R 'text' .",
            "Select-String -Pattern 'text' -Path *",
            "findstr /s /i text *",
            "grep -R 'text' .",
        ),
        (
            vec!["copy", "cp"],
            "copy file",
            "cp source target",
            "Copy-Item source target",
            "copy source target",
            "cp source target",
        ),
        (
            vec!["move", "rename", "mv"],
            "move or rename",
            "mv source target",
            "Move-Item source target",
            "move source target",
            "mv source target",
        ),
        (
            vec!["delete", "remove", "rm"],
            "delete file",
            "rm path",
            "Remove-Item path",
            "del path",
            "rm path",
        ),
        (
            vec!["env", "environment"],
            "print environment",
            "env",
            "Get-ChildItem Env:",
            "set",
            "env",
        ),
    ]
}

fn string_array_or_empty(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .take(1000)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_validation_and_plan_work() {
        let workflow = json!({
            "nodes": [
                {"id": "trigger", "type": "manual"},
                {"id": "code", "type": "function"},
                {"id": "notify", "type": "webhook"}
            ],
            "connections": [
                {"from": "trigger", "to": "code"},
                {"from": "code", "to": "notify"}
            ]
        });
        let validation = workflow_validate_work(json!({"workflow": workflow.clone()}));
        assert_eq!(validation["valid"], true);
        assert_eq!(validation["node_count"], 3);

        let plan = workflow_execution_plan_work(json!({"workflow": workflow}));
        assert_eq!(plan["ordered_nodes"], json!(["trigger", "code", "notify"]));
        assert_eq!(plan["levels"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn workflow_cycle_is_rejected() {
        let validation = workflow_validate_work(json!({
            "nodes": [{"id": "a"}, {"id": "b"}],
            "connections": [{"from": "a", "to": "b"}, {"from": "b", "to": "a"}]
        }));
        assert_eq!(validation["valid"], false);
        assert!(validation["errors"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .contains("cycle"));
    }

    #[test]
    fn app_catalog_search_and_task_packet_work() {
        let search = app_capability_search_work(json!({"query": "n8n"}));
        assert!(search["match_count"].as_u64().unwrap() > 0);

        let packet = agent_task_packet_work(json!({
            "goal": "fix bug",
            "files": ["src/main.rs"],
            "verification": ["cargo test"]
        }));
        assert_eq!(packet["goal"], "fix bug");
        assert_eq!(packet["ready_for"].as_array().unwrap().len(), 5);

        let recipe = app_tool_recipe_work(json!({"intent": "n8n workflow"}));
        assert_eq!(recipe["recipe"][1], "workflow_validate");
    }

    #[test]
    fn app_expansion_tools_work() {
        let node_suggestions = app_node_suggest_work(json!({"query": "webhook approval"}));
        assert!(node_suggestions["suggestions"].as_array().unwrap().len() >= 2);

        let template = workflow_template_generate_work(json!({"kind": "coding-ci"}));
        assert_eq!(template["validation"]["valid"], true);

        let prompt = agent_prompt_pack_work(json!({
            "goal": "fix tests",
            "files": ["src/main.rs"],
            "verification": ["cargo test"]
        }));
        assert_eq!(prompt["user"], "fix tests");

        let summary = agent_session_summary_work(json!({
            "messages": [
                {"text": "Decided to use catalog tests."},
                {"text": "Next fix missing verification risk."}
            ]
        }));
        assert_eq!(summary["message_count"], 2);

        let patch = agent_patch_plan_work(json!({
            "goal": "add tool",
            "files": ["src/ops/app_tools.rs"],
            "changes": ["add deterministic function"],
            "verification": ["cargo test"]
        }));
        assert_eq!(patch["ready"], true);

        let palette = app_command_palette_work(json!({"query": "workflow"}));
        assert!(!palette["commands"].as_array().unwrap().is_empty());

        let manifest = mcp_manifest_validate_work(json!({
            "name": "demo",
            "tools": [{"name": "demo.tool", "description": "demo", "input_schema": {}, "annotations": {}}]
        }));
        assert_eq!(manifest["valid"], true);

        let spec = mcp_tool_spec_generate_work(json!({"name": "demo.tool"}));
        assert_eq!(spec["tool"]["name"], "demo.tool");
    }

    #[test]
    fn app_plugin_skill_command_policy_tools_work() {
        let skill =
            app_skill_template_generate_work(json!({"name": "reviewer", "kind": "security"}));
        assert_eq!(skill["skill"]["name"], "reviewer");
        let skill_validation =
            app_skill_manifest_validate_work(json!({"manifest": skill["skill"].clone()}));
        assert_eq!(skill_validation["valid"], true);

        let plugin_validation = app_plugin_manifest_validate_work(json!({
            "name": "local-plugin",
            "version": "0.1.0",
            "tools": [{"name": "local.tool"}],
            "skills": [{"name": "reviewer"}],
            "commands": [{"name": "/review"}]
        }));
        assert_eq!(plugin_validation["valid"], true);

        let route = app_plugin_route_plan_work(json!({"intent": "build workflow plugin"}));
        assert!(!route["route"].as_array().unwrap().is_empty());

        let slash = app_slash_command_expand_work(json!({"command": "/skill security"}));
        assert_eq!(slash["normalized"], "skill");

        let hooks = app_hooks_policy_validate_work(json!({
            "hooks": [{"event": "before_edit", "action": "review plan"}]
        }));
        assert_eq!(hooks["valid"], true);

        let permission = app_permission_policy_evaluate_work(json!({
            "profile": "core-code",
            "action": "terminal_run",
            "executes": true
        }));
        assert_eq!(permission["allowed"], false);

        let model =
            app_model_profile_select_work(json!({"task": "fix code bug", "privacy": "local"}));
        assert_eq!(model["selected_profile"], "coding-strong");

        let context = app_context_budget_plan_work(json!({
            "max_tokens": 100,
            "items": [
                {"name": "a", "tokens": 60, "priority": 2},
                {"name": "b", "tokens": 60, "priority": 1}
            ]
        }));
        assert_eq!(context["selected"].as_array().unwrap().len(), 1);

        let queue = app_task_queue_plan_work(json!({
            "tasks": [
                {"id": "inspect"},
                {"id": "edit", "deps": ["inspect"]},
                {"id": "test", "deps": ["edit"]}
            ]
        }));
        assert_eq!(
            queue["execution_plan"]["ordered_nodes"],
            json!(["inspect", "edit", "test"])
        );
    }

    #[test]
    fn app_mcp_builder_and_agent_surface_tools_work() {
        let server = mcp_server_config_generate_work(json!({"name": "demo", "transport": "stdio"}));
        assert_eq!(server["config"]["transport"], "stdio");

        let prompt = mcp_prompt_template_validate_work(json!({
            "template": "Fix {file} with {goal}",
            "variables": ["file", "goal"]
        }));
        assert_eq!(prompt["valid"], true);

        let resource = mcp_resource_template_validate_work(json!({
            "uri_template": "file:///{path}"
        }));
        assert_eq!(resource["valid"], true);

        let sampling = mcp_sampling_request_plan_work(json!({"prompt": "fix code bug"}));
        assert_eq!(sampling["request"]["model_profile"], "coding-strong");

        let registry = app_command_registry_generate_work(json!({"include_experimental": true}));
        assert!(!registry["commands"].as_array().unwrap().is_empty());

        let route = app_skill_route_match_work(json!({
            "query": "security review",
            "skills": [{"name": "security-review", "description": "security review", "triggers": ["security"]}]
        }));
        assert_eq!(route["matches"].as_array().unwrap().len(), 1);

        let matrix = app_plugin_capability_matrix_work(json!({
            "plugins": [{"name": "p", "tools": [{"name": "t"}], "skills": [], "commands": [], "hooks": []}]
        }));
        assert_eq!(matrix["plugins"][0]["tools"], 1);

        let mode = app_agent_mode_plan_work(json!({"mode": "review"}));
        assert_eq!(mode["plan"]["permissions"], "read-only");

        let session = app_worktree_session_plan_work(json!({"goal": "Fix parser bug"}));
        assert_eq!(session["branch"], "fix-parser-bug");

        let evals = app_eval_prompt_pack_work(json!({"prompts": [{"prompt": "review this"}]}));
        assert_eq!(evals["count"], 1);
    }

    #[test]
    fn shell_command_tools_work_without_execution() {
        let catalog = shell_command_catalog_work(json!({"family": "powershell"}));
        assert!(!catalog["commands"].as_array().unwrap().is_empty());

        let translated = shell_command_translate_work(json!({"intent": "list files"}));
        assert!(!translated["translations"].as_array().unwrap().is_empty());

        let script = shell_script_template_generate_work(json!({"shell": "bash", "name": "demo"}));
        assert!(script["script"]
            .as_str()
            .unwrap()
            .contains("set -euo pipefail"));

        let safety = shell_command_safety_check_work(json!({"command": "rm -rf /tmp/demo"}));
        assert_eq!(safety["risk"], "high");
    }
}
