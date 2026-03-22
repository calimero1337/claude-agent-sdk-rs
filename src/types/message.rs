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
    /// Error field present when the assistant encountered an error.
    #[serde(default)]
    pub error: Option<String>,
    /// Token usage statistics for this message.
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
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
    /// The reason the model stopped generating (e.g. "end_turn", "tool_use").
    #[serde(default)]
    pub stop_reason: Option<String>,
    /// Structured output from the model when a structured output schema was used.
    #[serde(default)]
    pub structured_output: Option<serde_json::Value>,
}

/// A task_started system message from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartedMessage {
    pub task_id: String,
    pub description: String,
    #[serde(default)]
    pub session_id: Option<String>,
    // ── P2 Feature 11: additional fields ─────────────────────────────────────
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub task_type: Option<String>,
}

/// A task_progress system message from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressMessage {
    pub task_id: String,
    pub description: String,
    #[serde(default)]
    pub session_id: Option<String>,
    // ── P2 Feature 11: additional fields ─────────────────────────────────────
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub last_tool_name: Option<String>,
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

/// A task_notification system message from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotificationMessage {
    pub task_id: String,
    pub status: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    // ── P2 Feature 11: additional fields ─────────────────────────────────────
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub output_file: Option<String>,
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

/// Rate limit status from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitStatus {
    Allowed,
    AllowedWarning,
    Rejected,
}

/// Rate limit information from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub status: RateLimitStatus,
    #[serde(default)]
    pub resets_at: Option<String>,
    #[serde(default)]
    pub rate_limit_type: Option<String>,
    #[serde(default)]
    pub utilization: Option<f64>,
}

/// A rate limit event from the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    pub rate_limit_info: RateLimitInfo,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

/// A stream event from the CLI (partial message updates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEventMessage {
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    pub event: serde_json::Value,
    #[serde(default)]
    pub parent_tool_use_id: Option<String>,
}

/// Parsed message from the Claude CLI.
#[derive(Debug, Clone)]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Result(ResultMessage),
    TaskStarted(TaskStartedMessage),
    TaskProgress(TaskProgressMessage),
    TaskNotification(TaskNotificationMessage),
    /// A rate limit event from the CLI.
    RateLimit(RateLimitEvent),
    /// A stream event from the CLI (partial message updates).
    Stream(StreamEventMessage),
}

impl Message {
    /// Parse a raw JSON value from the CLI into a typed Message.
    pub fn parse(data: &serde_json::Value) -> Option<Self> {
        let msg_type = data.get("type")?.as_str()?;
        match msg_type {
            "user" => {
                // The CLI may wrap user messages: {"type": "user", "message": {...}}
                if let Some(inner) = data.get("message") {
                    serde_json::from_value(inner.clone()).ok().map(Message::User)
                } else {
                    serde_json::from_value(data.clone()).ok().map(Message::User)
                }
            }
            "assistant" => {
                // The CLI wraps assistant messages: {"type": "assistant", "message": {...}}
                // Try the wrapped format first, fall back to flat.
                if let Some(inner) = data.get("message") {
                    serde_json::from_value(inner.clone()).ok().map(Message::Assistant)
                } else {
                    serde_json::from_value(data.clone()).ok().map(Message::Assistant)
                }
            }
            "system" => {
                let subtype = data.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
                let data_field = data.get("data").cloned().unwrap_or_default();
                match subtype {
                    "task_started" => {
                        serde_json::from_value(data_field).ok().map(Message::TaskStarted)
                    }
                    "task_progress" => {
                        serde_json::from_value(data_field).ok().map(Message::TaskProgress)
                    }
                    "task_notification" => {
                        serde_json::from_value(data_field).ok().map(Message::TaskNotification)
                    }
                    _ => serde_json::from_value(data.clone()).ok().map(Message::System),
                }
            }
            "result" => serde_json::from_value(data.clone()).ok().map(Message::Result),
            "rate_limit_event" => serde_json::from_value(data.clone()).ok().map(Message::RateLimit),
            "stream_event" => serde_json::from_value(data.clone()).ok().map(Message::Stream),
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

    #[test]
    fn parse_task_started_message() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_started",
            "data": { "task_id": "t1", "description": "Starting work" }
        });
        let msg = Message::parse(&raw);
        assert!(matches!(msg, Some(Message::TaskStarted(ref m)) if m.task_id == "t1"));
    }

    #[test]
    fn parse_task_notification_message() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_notification",
            "data": { "task_id": "t2", "status": "completed", "summary": "Done" }
        });
        let msg = Message::parse(&raw);
        assert!(matches!(msg, Some(Message::TaskNotification(ref m)) if m.status == "completed"));
    }

    #[test]
    fn parse_unknown_system_falls_back() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "something_new",
            "data": { "info": "test" }
        });
        let msg = Message::parse(&raw);
        assert!(matches!(msg, Some(Message::System(_))));
    }

    // ── Feature 2: RateLimitEvent ────────────────────────────────────────────

    #[test]
    fn parse_rate_limit_event_allowed() {
        let raw = serde_json::json!({
            "type": "rate_limit_event",
            "rate_limit_info": {
                "status": "allowed",
                "resets_at": "2026-03-22T10:00:00Z",
                "rate_limit_type": "requests_per_minute",
                "utilization": 0.42
            },
            "uuid": "rl-uuid-1",
            "session_id": "sess-1"
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::RateLimit(evt) => {
                assert!(matches!(evt.rate_limit_info.status, RateLimitStatus::Allowed));
                assert_eq!(evt.rate_limit_info.resets_at.as_deref(), Some("2026-03-22T10:00:00Z"));
                assert_eq!(
                    evt.rate_limit_info.rate_limit_type.as_deref(),
                    Some("requests_per_minute")
                );
                assert!((evt.rate_limit_info.utilization.unwrap() - 0.42).abs() < f64::EPSILON);
                assert_eq!(evt.uuid.as_deref(), Some("rl-uuid-1"));
                assert_eq!(evt.session_id.as_deref(), Some("sess-1"));
            }
            other => panic!("expected RateLimit, got {:?}", other),
        }
    }

    #[test]
    fn parse_rate_limit_event_rejected() {
        let raw = serde_json::json!({
            "type": "rate_limit_event",
            "rate_limit_info": { "status": "rejected" }
        });
        let msg = Message::parse(&raw).expect("should parse");
        assert!(matches!(
            msg,
            Message::RateLimit(ref evt)
                if matches!(evt.rate_limit_info.status, RateLimitStatus::Rejected)
        ));
    }

    #[test]
    fn parse_rate_limit_event_allowed_warning() {
        let raw = serde_json::json!({
            "type": "rate_limit_event",
            "rate_limit_info": { "status": "allowed_warning" }
        });
        let msg = Message::parse(&raw).expect("should parse");
        assert!(matches!(
            msg,
            Message::RateLimit(ref evt)
                if matches!(evt.rate_limit_info.status, RateLimitStatus::AllowedWarning)
        ));
    }

    // ── Feature 3: ResultMessage stop_reason + structured_output ────────────

    #[test]
    fn parse_result_message_with_stop_reason() {
        let raw = serde_json::json!({
            "type": "result",
            "subtype": "success",
            "session_id": "sess-42",
            "duration_ms": 1500,
            "duration_api_ms": 800,
            "is_error": false,
            "num_turns": 3,
            "stop_reason": "end_turn",
            "structured_output": { "answer": 42, "unit": "items" }
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::Result(r) => {
                assert_eq!(r.stop_reason.as_deref(), Some("end_turn"));
                let structured = r.structured_output.expect("structured_output present");
                assert_eq!(structured["answer"], 42);
            }
            other => panic!("expected Result, got {:?}", other),
        }
    }

    #[test]
    fn parse_result_message_stop_reason_absent_is_none() {
        let raw = serde_json::json!({
            "type": "result",
            "subtype": "success",
            "session_id": "sess-43"
        });
        let msg = Message::parse(&raw).expect("should parse");
        if let Message::Result(r) = msg {
            assert!(r.stop_reason.is_none());
            assert!(r.structured_output.is_none());
        } else {
            panic!("expected Result");
        }
    }

    // ── Feature 4: AssistantMessage error + usage ────────────────────────────

    #[test]
    fn parse_assistant_message_with_error_and_usage() {
        let raw = serde_json::json!({
            "type": "assistant",
            "model": "claude-opus-4",
            "content": [],
            "error": "context_length_exceeded",
            "usage": { "input_tokens": 100, "output_tokens": 0 }
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::Assistant(a) => {
                assert_eq!(a.error.as_deref(), Some("context_length_exceeded"));
                let usage = a.usage.expect("usage present");
                assert_eq!(usage["input_tokens"], 100);
            }
            other => panic!("expected Assistant, got {:?}", other),
        }
    }

    #[test]
    fn parse_assistant_message_without_error_fields() {
        let raw = serde_json::json!({
            "type": "assistant",
            "model": "claude-opus-4",
            "content": [{ "type": "text", "text": "Hello" }]
        });
        let msg = Message::parse(&raw).expect("should parse");
        if let Message::Assistant(a) = msg {
            assert!(a.error.is_none());
            assert!(a.usage.is_none());
            assert_eq!(a.text_content(), "Hello");
        } else {
            panic!("expected Assistant");
        }
    }

    // ── Feature 5: StreamEventMessage ────────────────────────────────────────

    #[test]
    fn parse_stream_event_message() {
        let raw = serde_json::json!({
            "type": "stream_event",
            "uuid": "stream-uuid-1",
            "session_id": "sess-stream-1",
            "event": {
                "type": "content_block_delta",
                "index": 0,
                "delta": { "type": "text_delta", "text": "Hello " }
            },
            "parent_tool_use_id": null
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::Stream(s) => {
                assert_eq!(s.uuid.as_deref(), Some("stream-uuid-1"));
                assert_eq!(s.session_id.as_deref(), Some("sess-stream-1"));
                assert_eq!(s.event["type"], "content_block_delta");
                assert!(s.parent_tool_use_id.is_none());
            }
            other => panic!("expected Stream, got {:?}", other),
        }
    }

    #[test]
    fn parse_stream_event_message_minimal() {
        let raw = serde_json::json!({
            "type": "stream_event",
            "event": { "type": "message_stop" }
        });
        let msg = Message::parse(&raw).expect("should parse");
        assert!(matches!(msg, Message::Stream(ref s) if s.event["type"] == "message_stop"));
    }

    // ── P2 Feature 11: Complete task message fields ───────────────────────────

    #[test]
    fn task_started_message_parses_new_fields() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_started",
            "data": {
                "task_id": "t10",
                "description": "Starting",
                "uuid": "uuid-10",
                "tool_use_id": "tu-10",
                "task_type": "sub_agent"
            }
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::TaskStarted(m) => {
                assert_eq!(m.uuid.as_deref(), Some("uuid-10"));
                assert_eq!(m.tool_use_id.as_deref(), Some("tu-10"));
                assert_eq!(m.task_type.as_deref(), Some("sub_agent"));
            }
            other => panic!("expected TaskStarted, got {:?}", other),
        }
    }

    #[test]
    fn task_progress_message_parses_new_fields() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_progress",
            "data": {
                "task_id": "t11",
                "description": "In progress",
                "uuid": "uuid-11",
                "tool_use_id": "tu-11",
                "last_tool_name": "Bash",
                "usage": { "input_tokens": 50 }
            }
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::TaskProgress(m) => {
                assert_eq!(m.uuid.as_deref(), Some("uuid-11"));
                assert_eq!(m.tool_use_id.as_deref(), Some("tu-11"));
                assert_eq!(m.last_tool_name.as_deref(), Some("Bash"));
                assert_eq!(m.usage.as_ref().unwrap()["input_tokens"], 50);
            }
            other => panic!("expected TaskProgress, got {:?}", other),
        }
    }

    #[test]
    fn task_notification_message_parses_new_fields() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_notification",
            "data": {
                "task_id": "t12",
                "status": "completed",
                "uuid": "uuid-12",
                "tool_use_id": "tu-12",
                "output_file": "/tmp/result.json",
                "usage": { "output_tokens": 200 }
            }
        });
        let msg = Message::parse(&raw).expect("should parse");
        match msg {
            Message::TaskNotification(m) => {
                assert_eq!(m.uuid.as_deref(), Some("uuid-12"));
                assert_eq!(m.tool_use_id.as_deref(), Some("tu-12"));
                assert_eq!(m.output_file.as_deref(), Some("/tmp/result.json"));
                assert_eq!(m.usage.as_ref().unwrap()["output_tokens"], 200);
            }
            other => panic!("expected TaskNotification, got {:?}", other),
        }
    }

    #[test]
    fn task_started_new_fields_default_to_none() {
        let raw = serde_json::json!({
            "type": "system",
            "subtype": "task_started",
            "data": { "task_id": "t20", "description": "plain" }
        });
        let msg = Message::parse(&raw).expect("should parse");
        if let Message::TaskStarted(m) = msg {
            assert!(m.uuid.is_none());
            assert!(m.tool_use_id.is_none());
            assert!(m.task_type.is_none());
        } else {
            panic!("expected TaskStarted");
        }
    }
}
