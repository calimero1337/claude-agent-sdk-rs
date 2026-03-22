//! Session management APIs for listing, inspecting, and managing Claude sessions.

use serde::{Deserialize, Serialize};

/// Information about a Claude session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub num_turns: Option<u32>,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A message from a session transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub content: serde_json::Value,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Sanitize a string for use as a session tag.
///
/// Removes control characters and normalizes whitespace.
pub fn sanitize_tag(tag: &str) -> String {
    tag.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_info_round_trip() {
        let info = SessionInfo {
            session_id: "sess-abc".to_string(),
            title: Some("My Session".to_string()),
            model: Some("claude-opus-4".to_string()),
            created_at: Some("2026-03-22T10:00:00Z".to_string()),
            updated_at: Some("2026-03-22T11:00:00Z".to_string()),
            num_turns: Some(5),
            total_cost_usd: Some(0.03),
            tags: vec!["work".to_string(), "research".to_string()],
        };
        let json = serde_json::to_string(&info).unwrap();
        let decoded: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.session_id, "sess-abc");
        assert_eq!(decoded.title.as_deref(), Some("My Session"));
        assert_eq!(decoded.num_turns, Some(5));
        assert_eq!(decoded.tags.len(), 2);
    }

    #[test]
    fn session_info_minimal_defaults() {
        let json = r#"{"session_id": "sess-min"}"#;
        let info: SessionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.session_id, "sess-min");
        assert!(info.title.is_none());
        assert!(info.model.is_none());
        assert!(info.tags.is_empty());
    }

    #[test]
    fn session_message_round_trip() {
        let msg = SessionMessage {
            msg_type: "assistant".to_string(),
            content: serde_json::json!([{"type": "text", "text": "Hello"}]),
            timestamp: Some("2026-03-22T10:01:00Z".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: SessionMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.msg_type, "assistant");
        assert_eq!(decoded.timestamp.as_deref(), Some("2026-03-22T10:01:00Z"));
    }

    // ── P2 Feature 16: sanitize_tag ───────────────────────────────────────────

    #[test]
    fn sanitize_tag_removes_control_chars() {
        let input = "hello\x00world\x1b[31m";
        let result = sanitize_tag(input);
        assert_eq!(result, "helloworld[31m");
    }

    #[test]
    fn sanitize_tag_trims_whitespace() {
        let result = sanitize_tag("  my tag  ");
        assert_eq!(result, "my tag");
    }

    #[test]
    fn sanitize_tag_newline_removed() {
        let result = sanitize_tag("tag\nwith\nnewlines");
        assert_eq!(result, "tagwithnewlines");
    }

    #[test]
    fn sanitize_tag_tab_removed() {
        let result = sanitize_tag("tag\twith\ttabs");
        assert_eq!(result, "tagwithtabs");
    }

    #[test]
    fn sanitize_tag_plain_ascii_unchanged() {
        let result = sanitize_tag("work-project-2026");
        assert_eq!(result, "work-project-2026");
    }

    #[test]
    fn sanitize_tag_unicode_preserved() {
        // Non-control Unicode should pass through unchanged.
        let result = sanitize_tag("  tâche  ");
        assert_eq!(result, "tâche");
    }

    #[test]
    fn sanitize_tag_empty_string() {
        let result = sanitize_tag("");
        assert_eq!(result, "");
    }

    #[test]
    fn sanitize_tag_only_whitespace() {
        let result = sanitize_tag("   ");
        assert_eq!(result, "");
    }
}
