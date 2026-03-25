i want to use multiple ai agents at the same time on zed and i want ai agents to talk and help-suggest each other with their knowledge using this program like a messenger for ai agents, i want this program to be written in rust
Completed 2 steps
Here is a complete architecture and implementation guide for building a multi-agent messenger system in Rust that integrates with Zed.

Architecture Overview
The system has three layers: a Zed MCP server (how Zed talks to your program), an Agent Broker (the message bus), and Agent Workers (individual LLM-powered agents). Zed supports MCP (Model Context Protocol) servers natively, making this the cleanest integration path.
​
​

text
Zed Editor
    │  (MCP protocol)
    ▼
┌─────────────────────────────────┐
│   MCP Server (Rust binary)      │
│                                 │
│  ┌──────────────────────────┐   │
│  │     Agent Broker         │   │
│  │  (broadcast channel)     │   │
│  └────┬──────┬──────┬───────┘   │
│       │      │      │           │
│  Agent A  Agent B  Agent C      │
│  (Claude) (GPT-4) (Ollama)      │
└─────────────────────────────────┘
Key Rust Crates
Crate	Purpose
rig (0xPlaygrounds)	LLM agent creation, supports OpenAI, Anthropic, Ollama 
​
tokio	Async runtime, broadcast / mpsc channels for message passing 
​
rmcp	MCP server implementation so Zed can connect 
​
serde / serde_json	Message serialization between agents
AutoAgents	Alternative full multi-agent Rust framework with Ractor 
​
Core Message Bus (Broker)
The heart of the system is a tokio::sync::broadcast channel — every agent sends and receives from it, like a group chat:

rust
// Cargo.toml deps:
// rig-core = "0.9"
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// rmcp = "0.1"

use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: String,       // "agent_rust_expert"
    pub to: Option<String>, // None = broadcast to all
    pub content: String,
    pub msg_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    UserQuery,    // from Zed user
    AgentSuggest, // agent helping another agent
    AgentReply,   // agent's final answer
}

pub struct Broker {
    pub tx: broadcast::Sender<AgentMessage>,
}

impl Broker {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Broker { tx }
    }
    pub fn subscribe(&self) -> broadcast::Receiver<AgentMessage> {
        self.tx.subscribe()
    }
}
Agent Worker Implementation
Each agent runs as an independent tokio::spawn task, listens on the broadcast channel, and responds with its own LLM-powered knowledge using rig:

rust
use rig::providers::openai::Client as OpenAIClient;
use rig::providers::anthropic::Client as AnthropicClient;
use std::sync::Arc;

pub struct AgentWorker {
    pub name: String,
    pub role: String, // system prompt / specialty
    broker: Arc<Broker>,
}

impl AgentWorker {
    pub fn spawn(name: String, role: String, broker: Arc<Broker>) {
        let mut rx = broker.subscribe();
        let tx = broker.tx.clone();

        tokio::spawn(async move {
            // Build rig agent with a specialty role
            let openai = OpenAIClient::from_env();
            let agent = openai
                .agent("gpt-4o")
                .preamble(&role)
                .build();

            while let Ok(msg) = rx.recv().await {
                // Skip own messages to avoid infinite loops
                if msg.from == name { continue; }

                // Only respond to broadcasts or messages directed to this agent
                if msg.to.is_none() || msg.to.as_deref() == Some(&name) {
                    let prompt = format!(
                        "[From {}]: {}\nProvide your expert perspective:",
                        msg.from, msg.content
                    );

                    if let Ok(response) = agent.prompt(&prompt).await {
                        let reply = AgentMessage {
                            from: name.clone(),
                            to: Some(msg.from.clone()),
                            content: response,
                            msg_type: MessageType::AgentSuggest,
                        };
                        let _ = tx.send(reply);
                    }
                }
            }
        });
    }
}
Zed Integration via MCP Server
Zed natively supports MCP servers in its settings.json, making this the recommended integration strategy. Your binary exposes tools that Zed's AI panel can call:
​
​

rust
// In main.rs — expose "ask_agents" as an MCP tool
use rmcp::{ServerHandler, Tool, tool};

struct AgentMcpServer {
    broker: Arc<Broker>,
}

#[tool(description = "Send a question to the multi-agent panel")]
async fn ask_agents(
    &self,
    #[tool(param, description = "Your question")] query: String,
) -> String {
    let msg = AgentMessage {
        from: "zed_user".into(),
        to: None, // broadcast to all agents
        content: query,
        msg_type: MessageType::UserQuery,
    };
    self.broker.tx.send(msg).unwrap();

    // Collect responses from all agents over 5 seconds
    let mut rx = self.broker.subscribe();
    let mut replies = vec![];
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Ok(Ok(msg)) if matches!(msg.msg_type, MessageType::AgentSuggest) => {
                replies.push(format!("**{}**: {}", msg.from, msg.content));
            }
            _ => break,
        }
    }
    replies.join("\n\n---\n\n") // messenger-style thread
}
Then in Zed's settings.json, point to your compiled binary:

json
{
  "context_servers": {
    "agent-messenger": {
      "command": {
        "path": "/path/to/your/agent-messenger",
        "args": []
      }
    }
  }
}
Suggested Agent Roles
A useful starting team of specialized agents for a coding workflow:

rust_expert — focuses on Rust idioms, ownership, lifetimes

architect — suggests system design patterns and tradeoffs

debugger — analyzes errors and proposes fixes

reviewer — critiques code quality, suggests refactors

documenter — writes doc comments, explains what code does

Each agent sees every other agent's messages and can chime in with a follow-up suggestion, creating a real back-and-forth debate thread visible inside Zed.

Running Local Models (Offline)
For privacy, swap the OpenAI backend with Ollama so all agents run on your machine without any API calls:
​
​

rust
use rig::providers::ollama::Client as OllamaClient;

let ollama = OllamaClient::new(); // connects to localhost:11434
let agent = ollama
    .agent("qwen2.5-coder:14b")
    .preamble("You are a Rust expert...")
    .build();
You can mix backends — one agent running Claude via API, others running Ollama locally — since rig normalizes them all under the same Agent trait.
