//! Message types mirroring the Claude CLI JSON protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::tool::{ToolResult, ToolUseBlock};

/// The role of a message participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// A content block within a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content.
    Text { text: String },
    /// A tool use invocation from the assistant.
    ToolUse(ToolUseBlock),
    /// A tool result provided by the user.
    ToolResult(ToolResult),
    /// Thinking content (extended thinking feature).
    Thinking { thinking: String, signature: String },
}

impl ContentBlock {
    /// Returns the text content if this is a Text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        }
    }

    /// Returns true if this is a ToolUse block.
    pub fn is_tool_use(&self) -> bool {
        matches!(self, ContentBlock::ToolUse(_))
    }
}

/// A user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// An assistant message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

impl AssistantMessage {
    /// Extract all text content from this message.
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| c.as_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A system message (metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub subtype: String,
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
}

/// A result message (response summary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMessage {
    pub subtype: String,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub duration_api_ms: u64,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub num_turns: u32,
    #[serde(default)]
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// Parsed message from the Claude CLI.
#[derive(Debug, Clone)]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Result(ResultMessage),
}

impl Message {
    /// Parse a raw JSON value from the CLI into a typed Message.
    pub fn parse(data: &serde_json::Value) -> Option<Self> {
        let msg_type = data.get("type")?.as_str()?;
        match msg_type {
            "user" => serde_json::from_value(data.clone()).ok().map(Message::User),
            "assistant" => serde_json::from_value(data.clone()).ok().map(Message::Assistant),
            "system" => serde_json::from_value(data.clone()).ok().map(Message::System),
            "result" => serde_json::from_value(data.clone()).ok().map(Message::Result),
            _ => None, // Forward-compatible: skip unknown types
        }
    }

    /// Returns the assistant message if this is one, else None.
    pub fn as_assistant(&self) -> Option<&AssistantMessage> {
        match self {
            Message::Assistant(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the result message if this is one, else None.
    pub fn as_result(&self) -> Option<&ResultMessage> {
        match self {
            Message::Result(m) => Some(m),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_block_as_text() {
        let block = ContentBlock::Text {
            text: "hi".into(),
        };
        assert_eq!(block.as_text(), Some("hi"));
    }

    #[test]
    fn content_block_is_tool_use() {
        let block = ContentBlock::ToolUse(ToolUseBlock {
            id: "tu_1".into(),
            name: "bash".into(),
            input: serde_json::json!({}),
        });
        assert!(block.is_tool_use());
        assert_eq!(block.as_text(), None);
    }

    #[test]
    fn role_serialization() {
        let json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(json, r#""user""#);
        let role: Role = serde_json::from_str(r#""assistant""#).unwrap();
        assert_eq!(role, Role::Assistant);
    }
}
