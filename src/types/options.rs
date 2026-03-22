//! Configuration options for agent sessions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

/// Permission mode for the Claude CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    Plan,
    BypassPermissions,
}

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpServerConfig {
    Stdio {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
    },
    Sse {
        url: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
    },
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
    },
    /// In-process SDK-hosted MCP server.
    Sdk {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instance: Option<String>,
    },
    /// Claude AI proxy server.
    #[serde(rename = "claudeai-proxy")]
    ClaudeAiProxy {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

/// Effort level for the Claude CLI session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    #[default]
    Medium,
    High,
    Max,
}

/// Definition of a sub-agent that can be spawned within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Description of what this agent does.
    pub description: String,
    /// System prompt for the agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Tools available to this agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    /// Model to use for this agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// MCP servers available to this agent.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, serde_json::Value>,
}

/// Configuration for extended thinking / chain-of-thought.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThinkingConfig {
    /// Adaptive thinking (model decides).
    Adaptive,
    /// Enabled with a specific token budget.
    Enabled { budget_tokens: u32 },
    /// Thinking disabled.
    Disabled,
}

/// Sandbox network configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxNetworkConfig {
    #[serde(default)]
    pub enabled: bool,
}

/// Controls which sandbox violation types to ignore.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxIgnoreViolations {
    #[serde(default)]
    pub file: bool,
    #[serde(default)]
    pub network: bool,
}

/// Sandbox configuration for isolated execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<SandboxNetworkConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_violations: Option<SandboxIgnoreViolations>,
}

/// Configuration for an SDK plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkPluginConfig {
    #[serde(rename = "type")]
    pub plugin_type: String,
    pub path: String,
}

/// A clonable, debuggable wrapper around a stderr line callback.
///
/// `Arc<dyn Fn(&str) + Send + Sync>` doesn't implement `Debug` on its own,
/// so this newtype provides a no-op `Debug` impl and a `Clone` impl via `Arc`.
#[derive(Clone)]
pub struct StderrCallback(pub Arc<dyn Fn(&str) + Send + Sync>);

impl fmt::Debug for StderrCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StderrCallback(<fn>)")
    }
}

impl StderrCallback {
    /// Invoke the callback with a stderr line.
    pub fn call(&self, line: &str) {
        (self.0)(line);
    }
}

/// Options for a Claude agent session (subset of Python SDK's ClaudeAgentOptions).
#[derive(Debug, Clone, Default)]
pub struct ClaudeAgentOptions {
    /// System prompt.
    pub system_prompt: Option<String>,
    /// Model to use.
    pub model: Option<String>,
    /// Maximum number of turns.
    pub max_turns: Option<u32>,
    /// Permission mode.
    pub permission_mode: Option<PermissionMode>,
    /// MCP servers.
    pub mcp_servers: HashMap<String, McpServerConfig>,
    /// Allowed tools.
    pub allowed_tools: Vec<String>,
    /// Disallowed tools.
    pub disallowed_tools: Vec<String>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Path to the claude CLI binary.
    pub cli_path: Option<PathBuf>,
    /// Extra environment variables.
    pub env: HashMap<String, String>,
    /// Continue a previous conversation.
    pub continue_conversation: bool,
    /// Resume a specific session.
    pub resume: Option<String>,
    /// Additional directories.
    pub add_dirs: Vec<PathBuf>,
    /// Maximum output tokens per turn.
    pub max_tokens: Option<u32>,
    /// JSON schema for structured output validation.
    pub json_schema: Option<serde_json::Value>,
    /// Maximum budget in USD per session.
    pub max_budget_usd: Option<f64>,
    /// Effort level.
    pub effort: Option<Effort>,
    /// Fallback model if primary is unavailable.
    pub fallback_model: Option<String>,
    /// Append to system prompt instead of replacing.
    pub append_system_prompt: Option<String>,
    /// Beta features to enable.
    pub betas: Vec<String>,
    /// Which settings sources to load (e.g. "user", "project", "local", "none").
    pub setting_sources: Vec<String>,
    /// Base tool set override.
    pub tools: Vec<String>,
    /// Sub-agent definitions for multi-agent sessions.
    pub agents: HashMap<String, AgentDefinition>,
    /// Extended thinking configuration.
    pub thinking: Option<ThinkingConfig>,
    /// Optional callback for stderr lines from the CLI process.
    ///
    /// The callback is not serializable; it is skipped during cloning of the
    /// options across process boundaries (the `Arc` still clones cheaply within
    /// a single process). When `None`, stderr lines are logged at `DEBUG` level
    /// via `tracing`.
    pub stderr_callback: Option<StderrCallback>,
    // ── P2 features ──────────────────────────────────────────────────────────
    /// Fork the current session instead of starting a new one (--fork-session).
    pub fork_session: bool,
    /// Include partial messages in the output stream (--include-partial-messages).
    pub include_partial_messages: bool,
    /// Sandbox configuration for isolated execution.
    pub sandbox: Option<SandboxSettings>,
    /// SDK plugin configurations.
    pub plugins: Vec<SdkPluginConfig>,
    /// Maximum buffer size for reading messages (API parity; currently advisory).
    pub max_buffer_size: Option<usize>,
    /// User identifier for SDK metadata (not forwarded as CLI flag).
    pub user: Option<String>,
}

impl ClaudeAgentOptions {
    /// Build CLI arguments from these options.
    pub(crate) fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--input-format".to_string(),
            "stream-json".to_string(),
        ];

        if let Some(ref system) = self.system_prompt {
            args.push("--system-prompt".to_string());
            args.push(system.clone());
        }

        if let Some(ref model) = self.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(turns) = self.max_turns {
            args.push("--max-turns".to_string());
            args.push(turns.to_string());
        }

        if let Some(ref mode) = self.permission_mode {
            let mode_str = match mode {
                PermissionMode::Default => "default",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Plan => "plan",
                PermissionMode::BypassPermissions => "bypassPermissions",
            };
            args.push("--permission-mode".to_string());
            args.push(mode_str.to_string());
        }

        if !self.mcp_servers.is_empty() {
            let mcp_config = serde_json::json!({
                "mcpServers": &self.mcp_servers
            });
            args.push("--mcp-config".to_string());
            args.push(mcp_config.to_string());
        }

        if !self.allowed_tools.is_empty() {
            args.push("--allowedTools".to_string());
            args.push(self.allowed_tools.join(","));
        }

        if !self.disallowed_tools.is_empty() {
            args.push("--disallowedTools".to_string());
            args.push(self.disallowed_tools.join(","));
        }

        if self.continue_conversation {
            args.push("--continue".to_string());
        }

        if let Some(ref session) = self.resume {
            args.push("--resume".to_string());
            args.push(session.clone());
        }

        for dir in &self.add_dirs {
            args.push("--add-dir".to_string());
            args.push(dir.display().to_string());
        }

        if let Some(tokens) = self.max_tokens {
            args.push("--max-tokens".to_string());
            args.push(tokens.to_string());
        }

        if let Some(ref schema) = self.json_schema {
            args.push("--json-schema".to_string());
            args.push(schema.to_string());
        }

        if let Some(budget) = self.max_budget_usd {
            args.push("--max-budget-usd".to_string());
            args.push(format!("{:.2}", budget));
        }

        if let Some(ref effort) = self.effort {
            let effort_str = match effort {
                Effort::Low => "low",
                Effort::Medium => "medium",
                Effort::High => "high",
                Effort::Max => "max",
            };
            args.push("--effort".to_string());
            args.push(effort_str.to_string());
        }

        if let Some(ref model) = self.fallback_model {
            args.push("--fallback-model".to_string());
            args.push(model.clone());
        }

        if let Some(ref prompt) = self.append_system_prompt {
            args.push("--append-system-prompt".to_string());
            args.push(prompt.clone());
        }

        if !self.betas.is_empty() {
            args.push("--betas".to_string());
            args.push(self.betas.join(","));
        }

        if !self.setting_sources.is_empty() {
            args.push("--setting-sources".to_string());
            args.push(self.setting_sources.join(","));
        }

        if !self.tools.is_empty() {
            args.push("--tools".to_string());
            args.push(self.tools.join(","));
        }

        if let Some(ref thinking) = self.thinking {
            match thinking {
                ThinkingConfig::Enabled { budget_tokens } => {
                    args.push("--max-thinking-tokens".to_string());
                    args.push(budget_tokens.to_string());
                }
                ThinkingConfig::Disabled => {
                    // No flag needed — disabled is the default
                }
                ThinkingConfig::Adaptive => {
                    // Adaptive is signaled by not setting --max-thinking-tokens
                    // but the CLI may support an explicit flag in the future
                }
            }
        }

        // ── P2 features ──────────────────────────────────────────────────────

        if self.fork_session {
            args.push("--fork-session".to_string());
        }

        if self.include_partial_messages {
            args.push("--include-partial-messages".to_string());
        }

        // Merge sandbox settings into --settings JSON if provided.
        if let Some(ref sandbox) = self.sandbox {
            if let Ok(sandbox_json) = serde_json::to_value(sandbox) {
                let settings = serde_json::json!({ "sandbox": sandbox_json });
                args.push("--settings".to_string());
                args.push(settings.to_string());
            }
        }

        // Pass plugin directories via --plugin-dir.
        for plugin in &self.plugins {
            args.push("--plugin-dir".to_string());
            args.push(plugin.path.clone());
        }

        // max_buffer_size and user are not forwarded as CLI flags.

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_produce_minimal_args() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(args.contains(&"--output-format".to_string()));
        assert!(args.contains(&"stream-json".to_string()));
        assert!(!args.contains(&"--model".to_string()));
    }

    #[test]
    fn model_and_system_prompt_added() {
        let opts = ClaudeAgentOptions {
            model: Some("claude-sonnet-4-6".into()),
            system_prompt: Some("You are a test agent".into()),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"claude-sonnet-4-6".to_string()));
        assert!(args.contains(&"--system-prompt".to_string()));
    }

    #[test]
    fn permission_mode_serialization() {
        let json = serde_json::to_string(&PermissionMode::BypassPermissions).unwrap();
        assert_eq!(json, r#""bypassPermissions""#);
    }

    #[test]
    fn max_tokens_added_to_args() {
        let opts = ClaudeAgentOptions {
            max_tokens: Some(8192),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--max-tokens").expect("--max-tokens missing");
        assert_eq!(args[idx + 1], "8192");
    }

    #[test]
    fn json_schema_added_to_args() {
        let opts = ClaudeAgentOptions {
            json_schema: Some(serde_json::json!({"type": "object"})),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(args.contains(&"--json-schema".to_string()));
    }

    #[test]
    fn max_budget_usd_added_to_args() {
        let opts = ClaudeAgentOptions {
            max_budget_usd: Some(0.50),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--max-budget-usd").unwrap();
        assert_eq!(args[idx + 1], "0.50");
    }

    #[test]
    fn effort_added_to_args() {
        let opts = ClaudeAgentOptions {
            effort: Some(Effort::High),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--effort").unwrap();
        assert_eq!(args[idx + 1], "high");
    }

    #[test]
    fn fallback_model_added_to_args() {
        let opts = ClaudeAgentOptions {
            fallback_model: Some("claude-haiku-4-5".to_string()),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--fallback-model").unwrap();
        assert_eq!(args[idx + 1], "claude-haiku-4-5");
    }

    #[test]
    fn append_system_prompt_added_to_args() {
        let opts = ClaudeAgentOptions {
            append_system_prompt: Some("Extra context".to_string()),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--append-system-prompt").unwrap();
        assert_eq!(args[idx + 1], "Extra context");
    }

    // ── Feature 1: --betas ───────────────────────────────────────────────────

    #[test]
    fn betas_added_to_args() {
        let opts = ClaudeAgentOptions {
            betas: vec![
                "interleaved-thinking-2025-05-14".to_string(),
                "another-beta".to_string(),
            ],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--betas").expect("--betas missing");
        assert_eq!(args[idx + 1], "interleaved-thinking-2025-05-14,another-beta");
    }

    #[test]
    fn betas_empty_not_added() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--betas".to_string()));
    }

    // ── Feature 2: --setting-sources ─────────────────────────────────────────

    #[test]
    fn setting_sources_added_to_args() {
        let opts = ClaudeAgentOptions {
            setting_sources: vec!["user".to_string(), "project".to_string()],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args
            .iter()
            .position(|a| a == "--setting-sources")
            .expect("--setting-sources missing");
        assert_eq!(args[idx + 1], "user,project");
    }

    #[test]
    fn setting_sources_none_value_works() {
        let opts = ClaudeAgentOptions {
            setting_sources: vec!["none".to_string()],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--setting-sources").unwrap();
        assert_eq!(args[idx + 1], "none");
    }

    // ── Feature 3: --tools ───────────────────────────────────────────────────

    #[test]
    fn tools_added_to_args() {
        let opts = ClaudeAgentOptions {
            tools: vec!["Bash".to_string(), "Read".to_string(), "Write".to_string()],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--tools").expect("--tools missing");
        assert_eq!(args[idx + 1], "Bash,Read,Write");
    }

    #[test]
    fn tools_empty_not_added() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--tools".to_string()));
    }

    // ── Feature 4: AgentDefinition serialization ─────────────────────────────

    #[test]
    fn agent_definition_serializes_correctly() {
        let agent = AgentDefinition {
            description: "A helpful sub-agent".to_string(),
            prompt: Some("You are a sub-agent".to_string()),
            tools: vec!["Read".to_string()],
            model: Some("claude-haiku-4-5".to_string()),
            mcp_servers: HashMap::new(),
        };
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["description"], "A helpful sub-agent");
        assert_eq!(json["prompt"], "You are a sub-agent");
        assert_eq!(json["model"], "claude-haiku-4-5");
        assert!(json.get("mcp_servers").is_none(), "empty mcp_servers should be omitted");
    }

    #[test]
    fn agent_definition_minimal_serialization() {
        let agent = AgentDefinition {
            description: "Minimal agent".to_string(),
            prompt: None,
            tools: vec![],
            model: None,
            mcp_servers: HashMap::new(),
        };
        let json = serde_json::to_value(&agent).unwrap();
        assert_eq!(json["description"], "Minimal agent");
        assert!(json.get("prompt").is_none(), "None prompt should be omitted");
        assert!(json.get("tools").is_none(), "empty tools should be omitted");
        assert!(json.get("model").is_none(), "None model should be omitted");
    }

    #[test]
    fn agents_not_added_to_cli_args() {
        let mut agents = HashMap::new();
        agents.insert(
            "helper".to_string(),
            AgentDefinition {
                description: "Helper agent".to_string(),
                prompt: None,
                tools: vec![],
                model: None,
                mcp_servers: HashMap::new(),
            },
        );
        let opts = ClaudeAgentOptions { agents, ..Default::default() };
        let args = opts.to_cli_args();
        // agents are sent via initialize control request, not CLI flags
        assert!(!args.contains(&"--agents".to_string()));
    }

    // ── Feature 5: ThinkingConfig / --max-thinking-tokens ────────────────────

    #[test]
    fn thinking_enabled_adds_max_thinking_tokens() {
        let opts = ClaudeAgentOptions {
            thinking: Some(ThinkingConfig::Enabled { budget_tokens: 8000 }),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args
            .iter()
            .position(|a| a == "--max-thinking-tokens")
            .expect("--max-thinking-tokens missing");
        assert_eq!(args[idx + 1], "8000");
    }

    #[test]
    fn thinking_disabled_adds_no_flag() {
        let opts = ClaudeAgentOptions {
            thinking: Some(ThinkingConfig::Disabled),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--max-thinking-tokens".to_string()));
    }

    #[test]
    fn thinking_adaptive_adds_no_flag() {
        let opts = ClaudeAgentOptions {
            thinking: Some(ThinkingConfig::Adaptive),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--max-thinking-tokens".to_string()));
    }

    #[test]
    fn thinking_none_adds_no_flag() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--max-thinking-tokens".to_string()));
    }

    #[test]
    fn thinking_config_serializes_with_type_tag() {
        let cfg = ThinkingConfig::Enabled { budget_tokens: 5000 };
        let json = serde_json::to_value(&cfg).unwrap();
        assert_eq!(json["type"], "enabled");
        assert_eq!(json["budget_tokens"], 5000);
    }

    // ── Feature 6: stderr_callback field ─────────────────────────────────────

    #[test]
    fn stderr_callback_defaults_to_none() {
        let opts = ClaudeAgentOptions::default();
        assert!(opts.stderr_callback.is_none());
    }

    #[test]
    fn stderr_callback_can_be_set_and_invoked() {
        let captured = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let captured_clone = captured.clone();
        let opts = ClaudeAgentOptions {
            stderr_callback: Some(StderrCallback(Arc::new(move |line: &str| {
                captured_clone.lock().unwrap().push(line.to_string());
            }))),
            ..Default::default()
        };
        // Invoke the callback directly to verify it works.
        if let Some(cb) = &opts.stderr_callback {
            cb.call("test stderr line");
        }
        let lines = captured.lock().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "test stderr line");
    }

    #[test]
    fn stderr_callback_debug_output() {
        let cb = StderrCallback(Arc::new(|_line: &str| {}));
        let debug_str = format!("{:?}", cb);
        assert_eq!(debug_str, "StderrCallback(<fn>)");
    }

    // ── P2 Feature 1: --fork-session ─────────────────────────────────────────

    #[test]
    fn fork_session_adds_flag() {
        let opts = ClaudeAgentOptions {
            fork_session: true,
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(args.contains(&"--fork-session".to_string()));
    }

    #[test]
    fn fork_session_false_does_not_add_flag() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--fork-session".to_string()));
    }

    // ── P2 Feature 2: --include-partial-messages ──────────────────────────────

    #[test]
    fn include_partial_messages_adds_flag() {
        let opts = ClaudeAgentOptions {
            include_partial_messages: true,
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(args.contains(&"--include-partial-messages".to_string()));
    }

    #[test]
    fn include_partial_messages_false_does_not_add_flag() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--include-partial-messages".to_string()));
    }

    // ── P2 Feature 4: McpServerConfig Sdk + ClaudeAiProxy variants ───────────

    #[test]
    fn mcp_sdk_variant_serializes() {
        let cfg = McpServerConfig::Sdk {
            name: "my-server".to_string(),
            instance: Some("inst-1".to_string()),
        };
        let json = serde_json::to_value(&cfg).unwrap();
        assert_eq!(json["type"], "sdk");
        assert_eq!(json["name"], "my-server");
        assert_eq!(json["instance"], "inst-1");
    }

    #[test]
    fn mcp_sdk_variant_no_instance_omits_field() {
        let cfg = McpServerConfig::Sdk {
            name: "my-server".to_string(),
            instance: None,
        };
        let json = serde_json::to_value(&cfg).unwrap();
        assert!(json.get("instance").is_none());
    }

    #[test]
    fn mcp_claude_ai_proxy_variant_serializes() {
        let cfg = McpServerConfig::ClaudeAiProxy {
            url: "https://proxy.example.com".to_string(),
            id: Some("proxy-1".to_string()),
        };
        let json = serde_json::to_value(&cfg).unwrap();
        assert_eq!(json["type"], "claudeai-proxy");
        assert_eq!(json["url"], "https://proxy.example.com");
        assert_eq!(json["id"], "proxy-1");
    }

    #[test]
    fn mcp_claude_ai_proxy_no_id_omits_field() {
        let cfg = McpServerConfig::ClaudeAiProxy {
            url: "https://proxy.example.com".to_string(),
            id: None,
        };
        let json = serde_json::to_value(&cfg).unwrap();
        assert!(json.get("id").is_none());
    }

    // ── P2 Feature 5: SandboxSettings ────────────────────────────────────────

    #[test]
    fn sandbox_settings_added_to_args() {
        let opts = ClaudeAgentOptions {
            sandbox: Some(SandboxSettings {
                enabled: true,
                sandbox_type: Some("docker".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let args = opts.to_cli_args();
        assert!(args.contains(&"--settings".to_string()));
        let idx = args.iter().position(|a| a == "--settings").unwrap();
        let settings_json: serde_json::Value =
            serde_json::from_str(&args[idx + 1]).expect("valid JSON");
        assert_eq!(settings_json["sandbox"]["enabled"], true);
        assert_eq!(settings_json["sandbox"]["sandbox_type"], "docker");
    }

    #[test]
    fn sandbox_none_does_not_add_settings_flag() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--settings".to_string()));
    }

    #[test]
    fn sandbox_settings_serialization_round_trip() {
        let settings = SandboxSettings {
            enabled: true,
            sandbox_type: Some("docker".to_string()),
            image: Some("ubuntu:22.04".to_string()),
            network: Some(SandboxNetworkConfig { enabled: false }),
            ignore_violations: Some(SandboxIgnoreViolations { file: true, network: false }),
        };
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["enabled"], true);
        assert_eq!(json["sandbox_type"], "docker");
        assert_eq!(json["image"], "ubuntu:22.04");
        assert_eq!(json["network"]["enabled"], false);
        assert_eq!(json["ignore_violations"]["file"], true);
    }

    // ── P2 Feature 6: SdkPluginConfig / --plugin-dir ─────────────────────────

    #[test]
    fn plugin_dir_added_to_args() {
        let opts = ClaudeAgentOptions {
            plugins: vec![
                SdkPluginConfig {
                    plugin_type: "local".to_string(),
                    path: "/opt/plugins/my-plugin".to_string(),
                },
            ],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let idx = args.iter().position(|a| a == "--plugin-dir").expect("--plugin-dir missing");
        assert_eq!(args[idx + 1], "/opt/plugins/my-plugin");
    }

    #[test]
    fn multiple_plugins_each_get_flag() {
        let opts = ClaudeAgentOptions {
            plugins: vec![
                SdkPluginConfig { plugin_type: "local".to_string(), path: "/p1".to_string() },
                SdkPluginConfig { plugin_type: "local".to_string(), path: "/p2".to_string() },
            ],
            ..Default::default()
        };
        let args = opts.to_cli_args();
        let count = args.iter().filter(|a| a.as_str() == "--plugin-dir").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn no_plugins_means_no_plugin_dir_flag() {
        let opts = ClaudeAgentOptions::default();
        let args = opts.to_cli_args();
        assert!(!args.contains(&"--plugin-dir".to_string()));
    }

    // ── P2 Feature 14: max_buffer_size ────────────────────────────────────────

    #[test]
    fn max_buffer_size_stored_not_in_args() {
        let opts = ClaudeAgentOptions {
            max_buffer_size: Some(65536),
            ..Default::default()
        };
        assert_eq!(opts.max_buffer_size, Some(65536));
        // It must not appear in CLI args.
        let args = opts.to_cli_args();
        assert!(!args.iter().any(|a| a.contains("buffer")));
    }

    // ── P2 Feature 15: user field ─────────────────────────────────────────────

    #[test]
    fn user_field_stored_not_in_args() {
        let opts = ClaudeAgentOptions {
            user: Some("user-42".to_string()),
            ..Default::default()
        };
        assert_eq!(opts.user.as_deref(), Some("user-42"));
        // It must not appear in CLI args.
        let args = opts.to_cli_args();
        assert!(!args.iter().any(|a| a.contains("user-42")));
    }
}
