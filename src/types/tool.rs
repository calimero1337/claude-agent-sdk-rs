//! Tool definitions and related types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Defines a tool available to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl Tool {
    /// Create a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }

    /// Create a simple tool with no parameters.
    pub fn simple(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(
            name,
            description,
            serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
        )
    }
}

/// Specifies how the model should choose tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    Auto,
    Any,
    Tool { name: String },
}

impl Default for ToolChoice {
    fn default() -> Self {
        Self::Auto
    }
}

/// A tool use invocation from the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: Value,
}

impl ToolUseBlock {
    /// Deserialize the input into a specific type.
    pub fn parse_input<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.input.clone())
    }
}

/// The result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: ToolResultContent,
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful text result.
    pub fn success(tool_use_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Text(text.into()),
            is_error: false,
        }
    }

    /// Create an error result.
    pub fn error(tool_use_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Text(message.into()),
            is_error: true,
        }
    }
}

/// Content returned by a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<HashMap<String, Value>>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_creation() {
        let tool = Tool::simple("echo", "Echoes the input");
        assert_eq!(tool.name, "echo");
        assert_eq!(tool.input_schema["type"], "object");
    }

    #[test]
    fn tool_result_success() {
        let result = ToolResult::success("tu_123", "output text");
        assert!(!result.is_error);
        assert_eq!(result.tool_use_id, "tu_123");
    }

    #[test]
    fn tool_result_error() {
        let result = ToolResult::error("tu_456", "something went wrong");
        assert!(result.is_error);
    }

    #[test]
    fn tool_choice_default() {
        let choice = ToolChoice::default();
        assert!(matches!(choice, ToolChoice::Auto));
    }

    #[test]
    fn tool_use_block_parse_input() {
        let block = ToolUseBlock {
            id: "tu_1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls -la"}),
        };
        let cmd: serde_json::Value = block.parse_input().unwrap();
        assert_eq!(cmd["command"], "ls -la");
    }
}
