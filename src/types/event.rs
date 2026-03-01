//! Stream event types from the Claude CLI.

use serde::{Deserialize, Serialize};

use crate::types::message::ContentBlock;

/// Events emitted by the agent stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// A new message turn started.
    MessageStart { message_id: String, model: String },
    /// A content block started.
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    /// A content block delta (streaming text/json).
    ContentBlockDelta { index: usize, delta: ContentDelta },
    /// A content block finished.
    ContentBlockStop { index: usize },
    /// A message delta with stop metadata.
    MessageDelta {
        stop_reason: StopReason,
        stop_sequence: Option<String>,
        usage: TokenUsage,
    },
    /// The message stream finished.
    MessageStop,
    /// An error event.
    Error { error_type: String, message: String },
}

/// Why the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

/// A streaming content delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_reason_serialization() {
        let json = serde_json::to_string(&StopReason::ToolUse).unwrap();
        assert_eq!(json, r#""tool_use""#);

        let reason: StopReason = serde_json::from_str(r#""end_turn""#).unwrap();
        assert_eq!(reason, StopReason::EndTurn);
    }

    #[test]
    fn token_usage_defaults() {
        let json = r#"{"input_tokens": 10, "output_tokens": 20}"#;
        let usage: TokenUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 20);
        assert_eq!(usage.cache_creation_input_tokens, 0);
        assert_eq!(usage.cache_read_input_tokens, 0);
    }

    #[test]
    fn content_delta_text() {
        let json = r#"{"type": "text_delta", "text": "hello"}"#;
        let delta: ContentDelta = serde_json::from_str(json).unwrap();
        assert!(matches!(delta, ContentDelta::TextDelta { text } if text == "hello"));
    }
}
