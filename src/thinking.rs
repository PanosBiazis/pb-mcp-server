//! Deterministic planning / reflection scaffolds for agents (no external models).
//! Session log tools write under `.mint-mcp/` and are **local-network only** (same gate as other disk writes).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const LOG_REL: &str = ".mint-mcp/agent-thinking.log";
const MAX_NOTE_CHARS: usize = 4096;
const MAX_LOG_BYTES: u64 = 400 * 1024;
const KEEP_TAIL_BYTES: usize = 200 * 1024;

fn log_path(root: &Path) -> PathBuf {
    root.join(LOG_REL)
}

fn rotate_log(path: &Path) {
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if meta.len() <= MAX_LOG_BYTES {
        return;
    }
    let Ok(raw) = fs::read_to_string(path) else {
        return;
    };
    let bytes = raw.as_bytes();
    if bytes.len() <= KEEP_TAIL_BYTES {
        return;
    }
    let start = bytes.len() - KEEP_TAIL_BYTES;
    let slice = &raw[start..];
    let cut = slice
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap_or(start);
    let _ = fs::write(path, &raw.as_bytes()[cut..]);
}

/// Structured plan template (fill-in-the-blank for agents).
pub fn thinking_plan_skeleton_work(
    objective: String,
    constraints: Vec<String>,
    known_steps: Vec<String>,
    max_chars: usize,
) -> String {
    let max_chars = max_chars.clamp(512, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("THINKING_PLAN_SKELETON\n\n");
    s.push_str("## Objective\n");
    s.push_str(objective.trim());
    s.push_str("\n\n## Constraints / invariants\n");
    let mut n = 0usize;
    for c in constraints.iter().take(24) {
        let t = c.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(none listed — add assumptions you rely on.)\n");
    }
    s.push_str("\n## Known steps / milestones\n");
    n = 0;
    for c in known_steps.iter().take(24) {
        let t = c.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(draft ordered steps; keep each step verifiable.)\n");
    }
    s.push_str("\n## Open questions\n- …\n\n## Risks & mitigations\n- Risk: …  Mitigation: …\n\n## Verification\n- How will you know you are done?\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// “Step back” reframing prompts (deterministic checklist).
pub fn thinking_step_back_work(problem: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let p = problem.trim();
    let mut s = String::from("THINKING_STEP_BACK\n\n");
    s.push_str("Problem as stated:\n");
    s.push_str(if p.is_empty() { "(empty — restate the user goal.)" } else { p });
    s.push_str("\n\nWork through:\n");
    s.push_str("1) What is the smallest outcome that would still help?\n");
    s.push_str("2) What are you assuming? Which assumptions are risky?\n");
    s.push_str("3) What evidence would falsify your approach?\n");
    s.push_str("4) Can you decompose into parallelizable pieces vs one serial chain?\n");
    s.push_str("5) What tools in this MCP give ground truth (read_file, git_*, grep_code, bus)?\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Turn free-form claims into an explicit audit list.
pub fn thinking_assumption_audit_work(claims: Vec<String>, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("THINKING_ASSUMPTION_AUDIT\n\n");
    let mut n = 0usize;
    for c in claims.iter().take(32) {
        let t = c.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!(
            "{n}. Claim: {t}\n   Evidence needed: …\n   If false, plan B: …\n\n"
        ));
    }
    if n == 0 {
        s.push_str("(no claims passed — list explicit assumptions from your plan.)\n");
    }
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Append one timestamped line to `.mint-mcp/agent-thinking.log` (local network only).
pub fn thinking_session_append_work(root: PathBuf, note: String) -> String {
    let note = note.replace(['\r', '\n'], " ").trim().to_string();
    if note.is_empty() {
        return "Error: note must not be empty".to_string();
    }
    let (note, _) = crate::ops::truncate_chars(&note, MAX_NOTE_CHARS);
    let path = log_path(&root);
    if let Some(dir) = path.parent() {
        if let Err(e) = fs::create_dir_all(dir) {
            return format!("Error creating .mint-mcp: {e}");
        }
    }
    rotate_log(&path);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let line = format!("{ts}\t{note}\n");
    let mut f = match fs::OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Error open log: {e}"),
    };
    if let Err(e) = f.write_all(line.as_bytes()) {
        return format!("Error write log: {e}");
    }
    rotate_log(&path);
    format!(
        "THINKING_SESSION_APPEND\nwrote: {}\nbytes_line: {}\n",
        path.display(),
        line.len()
    )
}

/// Read last N lines from the thinking session log (local network only; avoids leaking on public deployments).
pub fn thinking_session_read_work(root: PathBuf, tail_lines: usize) -> String {
    let tail_lines = tail_lines.clamp(1, 500);
    let path = log_path(&root);
    if !path.exists() {
        return format!(
            "THINKING_SESSION_READ\npath: {}\n(empty — no entries yet)\n",
            path.display()
        );
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return format!("Error: {e}"),
    };
    let lines: Vec<&str> = raw.lines().collect();
    let start = lines.len().saturating_sub(tail_lines);
    let mut out = String::from("THINKING_SESSION_READ\n");
    out.push_str(&format!("path: {}\nlines: {} (showing last {})\n\n", path.display(), lines.len(), tail_lines));
    for line in &lines[start..] {
        out.push_str(line);
        out.push('\n');
    }
    let (out, _) = crate::ops::truncate_chars(&out, crate::ops::MAX_RESPONSE_CHARS);
    out
}

/// Checklist template for execution tracking (agents tick items in their own state).
pub fn plan_execution_checklist_work(milestones: Vec<String>, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("PLAN_EXECUTION_CHECKLIST\n\n");
    let mut n = 0usize;
    for m in milestones.iter().take(48) {
        let t = m.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("[ ] {n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(no milestones — pass ordered strings: design, implement, test, ship, …)\n");
    }
    s.push_str("\nAdd dates/owners in your tracker; use agent_subagent_task for delegated items.\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Structured brief to paste into `agent_subagent_task.task_body` or human runbooks.
pub fn plan_handoff_brief_work(
    title: String,
    scope: String,
    deliverables: Vec<String>,
    acceptance: Vec<String>,
    blockers: Vec<String>,
    max_chars: usize,
) -> String {
    let max_chars = max_chars.clamp(512, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("PLAN_HANDOFF_BRIEF\n\n");
    s.push_str("## Title\n");
    s.push_str(title.trim());
    s.push_str("\n\n## Scope\n");
    s.push_str(scope.trim());
    s.push_str("\n\n## Deliverables\n");
    let mut k = 0usize;
    for line in deliverables.iter().take(24) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        k += 1;
        s.push_str(&format!("{k}. {t}\n"));
    }
    if k == 0 {
        s.push_str("(list concrete outputs)\n");
    }
    s.push_str("\n## Acceptance / verification\n");
    k = 0;
    for line in acceptance.iter().take(24) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        k += 1;
        s.push_str(&format!("{k}. {t}\n"));
    }
    if k == 0 {
        s.push_str("(tests, commands, or observable checks)\n");
    }
    s.push_str("\n## Blockers / dependencies\n");
    k = 0;
    for line in blockers.iter().take(16) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        k += 1;
        s.push_str(&format!("{k}. {t}\n"));
    }
    if k == 0 {
        s.push_str("(none listed)\n");
    }
    s.push_str("\n## Ground-truth tools\nread_file, grep_code, git_*, code_index_search, agent_thread.\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Round-robin split of work items into N buckets (for parallel subagents / fan_out planning).
pub fn plan_work_buckets_work(items: Vec<String>, buckets: usize, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let b = buckets.clamp(2, 16);
    let mut slots: Vec<Vec<String>> = (0..b).map(|_| Vec::new()).collect();
    let mut i = 0usize;
    for it in items.iter().take(200) {
        let t = it.trim();
        if t.is_empty() {
            continue;
        }
        slots[i % b].push(t.to_string());
        i += 1;
    }
    let mut s = format!("PLAN_WORK_BUCKETS buckets={b} items_assigned={i}\n\n");
    for (idx, slot) in slots.iter().enumerate() {
        s.push_str(&format!("--- bucket_{idx} ({} items) ---\n", slot.len()));
        for (j, line) in slot.iter().enumerate() {
            s.push_str(&format!("  {}. {}\n", j + 1, line));
        }
        s.push('\n');
    }
    s.push_str("hint: assign one bucket per sub_agent id; use agent_fan_out if the same message goes to all.\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Likelihood / impact / mitigation scaffold for release or architecture reviews.
pub fn plan_risk_register_work(risks: Vec<String>, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("PLAN_RISK_REGISTER\n\n");
    let mut n = 0usize;
    for r in risks.iter().take(32) {
        let t = r.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!(
            "## R{n}: {t}\n- Likelihood (L/M/H): …\n- Impact (L/M/H): …\n- Mitigation: …\n- Owner / review trigger: …\n\n"
        ));
    }
    if n == 0 {
        s.push_str("(no risks — add technical, security, schedule, dependency, and operational items.)\n");
    }
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Acceptance criteria / “done means…” checklist for agents and reviewers.
pub fn plan_definition_of_done_work(outcomes: Vec<String>, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("PLAN_DEFINITION_OF_DONE\n\n");
    let mut n = 0usize;
    for o in outcomes.iter().take(32) {
        let t = o.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("[ ] {n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(add observable outcomes: tests pass, metrics, docs updated, bus handoff closed, …)\n");
    }
    s.push_str("\nSign-off: owner __  date __  evidence links (git SHA, CI run, ticket) __\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// “Assume we failed” pre-mortem prompts (deterministic; you fill causes and guards).
pub fn reasoning_premortem_work(project: String, timeframe: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let p = project.trim();
    let tf = timeframe.trim();
    let mut s = String::from("REASONING_PREMORTEM\n\n");
    s.push_str("Project / decision:\n");
    s.push_str(if p.is_empty() { "(name the initiative.)" } else { p });
    s.push_str("\n\nHorizon:\n");
    s.push_str(if tf.is_empty() { "(e.g. next sprint, release cut, 30 days.)" } else { tf });
    s.push_str("\n\nAssume it failed badly. Answer:\n");
    s.push_str("1) What concrete failure happened (symptoms users saw)?\n");
    s.push_str("2) What early warnings did we ignore?\n");
    s.push_str("3) Root causes (technical, process, people, external)?\n");
    s.push_str("4) What guardrails or tests would have caught this earlier?\n");
    s.push_str("5) What do we do in the next 48h to reduce probability?\n");
    s.push_str("6) Which MCP tools verify mitigations (grep_code, git_diff, project_cargo_test / project_pytest when enabled)?\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Test pyramid + coverage areas scaffold for agents (fill-in; no execution).
pub fn plan_test_strategy_work(
    scope: String,
    risk_hotspots: Vec<String>,
    max_chars: usize,
) -> String {
    let max_chars = max_chars.clamp(512, crate::ops::MAX_RESPONSE_CHARS);
    let sc = scope.trim();
    let mut s = String::from("PLAN_TEST_STRATEGY\n\n");
    s.push_str("## Scope under test\n");
    s.push_str(if sc.is_empty() { "(crate, service, or feature area.)" } else { sc });
    s.push_str("\n\n## Risk / change hotspots\n");
    let mut n = 0usize;
    for r in risk_hotspots.iter().take(20) {
        let t = r.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(list fragile modules, concurrency, parsing, crypto, migrations, …)\n");
    }
    s.push_str("\n## Unit\n- [ ] …\n\n## Integration\n- [ ] …\n\n## E2E / smoke (if any)\n- [ ] …\n\n");
    s.push_str("## Data / fixtures\n- [ ] …\n\n## How to run (local / CI)\n");
    s.push_str("- Rust: project_cargo_test (MINT_PROJECT_TEST_ENABLE=1 + cargo in MINT_TERMINAL_ALLOW)\n");
    s.push_str("- Python: project_pytest (same gate + pytest allowlisted)\n");
    s.push_str("- Or terminal_run with explicit argv on trusted LAN only.\n\n");
    s.push_str("## Exit criteria\n- [ ] All targeted tests green; no new clippy -D warnings (if enforced).\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Five-whys ladder (you answer each “why”; deterministic template).
pub fn reasoning_five_whys_work(seed_problem: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let p = seed_problem.trim();
    let mut s = String::from("REASONING_FIVE_WHYS\n\n");
    s.push_str("Observed problem / symptom:\n");
    s.push_str(if p.is_empty() { "(state the failure or bug.)" } else { p });
    s.push_str("\n\nFill each line with your answer; stop early if root cause is clear.\n");
    s.push_str("1) Why did this happen? → …\n");
    s.push_str("2) Why was that? → …\n");
    s.push_str("3) Why was that? → …\n");
    s.push_str("4) Why was that? → …\n");
    s.push_str("5) Why was that? → …\n\n");
    s.push_str("Root cause hypothesis:\n- …\n\nCountermeasures:\n- …\n\nVerify with: read_file, grep_code, git_diff, project_cargo_test (if enabled).\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Short retrospective scaffold after a milestone or agent session.
pub fn plan_retrospective_work(iteration: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let it = iteration.trim();
    let mut s = String::from("PLAN_RETROSPECTIVE\n\n");
    s.push_str("Iteration / window:\n");
    s.push_str(if it.is_empty() { "(sprint, PR, or agent batch.)" } else { it });
    s.push_str("\n\n## Went well\n- …\n\n## Did not go well\n- …\n\n## Surprises\n- …\n\n## Actions (owners)\n");
    s.push_str("| Action | Owner | Due | Metric |\n|--------|-------|-----|--------|\n| … | … | … | … |\n\n");
    s.push_str("## Bus / multi-agent\n- Topics that worked: …\n- Adjust serial vs parallel next time: …\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Explicit premise → intermediate steps → open conclusion (no automated inference).
pub fn reasoning_chain_work(
    premises: Vec<String>,
    steps: Vec<String>,
    open_question: String,
    max_chars: usize,
) -> String {
    let max_chars = max_chars.clamp(512, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("REASONING_CHAIN\n\n");
    s.push_str("## Premises / facts (cite tools or sources)\n");
    let mut n = 0usize;
    for p in premises.iter().take(20) {
        let t = p.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(list what you treat as given.)\n");
    }
    s.push_str("\n## Deductive / constructive steps\n");
    n = 0;
    for p in steps.iter().take(24) {
        let t = p.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t}\n"));
    }
    if n == 0 {
        s.push_str("(each step should follow from premises or prior steps.)\n");
    }
    s.push_str("\n## Open question / decision\n");
    s.push_str(open_question.trim());
    s.push_str("\n\n## Confidence\n- High / medium / low because …\n\n## What would change your mind?\n- …\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Structured pushback prompts on a claim (deterministic; not an adversarial model).
pub fn reasoning_devils_advocate_work(claim: String, context: String, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let c = claim.trim();
    let ctx = context.trim();
    let mut s = String::from("REASONING_DEVILS_ADVOCATE\n\n");
    s.push_str("Claim:\n");
    s.push_str(if c.is_empty() { "(empty — state the thesis.)" } else { c });
    s.push_str("\n\nContext:\n");
    s.push_str(if ctx.is_empty() { "(optional background)" } else { ctx });
    s.push_str("\n\nStress-test:\n");
    s.push_str("1) Strongest alternative explanation?\n");
    s.push_str("2) What data would disprove the claim?\n");
    s.push_str("3) Where could sampling bias or missing negatives hide?\n");
    s.push_str("4) Operationalize: what exact observation falsifies each sub-claim?\n");
    s.push_str("5) After pushback, what survives? What needs evidence from read_file/grep/git?\n");
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}

/// Evidence table scaffold: claim | needed evidence | possible source tool.
pub fn reasoning_evidence_matrix_work(claims: Vec<String>, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(400, crate::ops::MAX_RESPONSE_CHARS);
    let mut s = String::from("REASONING_EVIDENCE_MATRIX\n\n");
    s.push_str("claim | evidence_needed | source_hint (e.g. read_file, git_log, grep_code)\n");
    s.push_str("------|-------------------|------------------\n");
    let mut n = 0usize;
    for c in claims.iter().take(24) {
        let t = c.trim();
        if t.is_empty() {
            continue;
        }
        n += 1;
        s.push_str(&format!("{n}. {t} | … | …\n"));
    }
    if n == 0 {
        s.push_str("(pass atomic claims, one per row to fill.)\n");
    }
    let (out, _) = crate::ops::truncate_chars(&s, max_chars);
    out
}
