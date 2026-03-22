//! Configuration options for agent sessions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
}
