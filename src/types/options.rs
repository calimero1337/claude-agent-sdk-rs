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
}
