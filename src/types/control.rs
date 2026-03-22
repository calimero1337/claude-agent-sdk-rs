//! Control protocol types for bidirectional SDK ↔ CLI communication.
//!
//! The control protocol enables permission callbacks, hooks, and session
//! management. Messages flow on stdin/stdout alongside regular messages.

use serde::{Deserialize, Serialize};

/// A control request from the CLI to the SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    /// Unique request ID for correlation.
    pub request_id: String,
    /// The request body.
    pub request: ControlRequestBody,
}

/// Body of a control request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype")]
pub enum ControlRequestBody {
    /// CLI asks SDK for permission to use a tool.
    #[serde(rename = "permission")]
    Permission {
        tool_name: String,
        input: serde_json::Value,
    },
    /// Session initialization handshake.
    #[serde(rename = "initialize")]
    Initialize,
    /// Pre/post tool use hook callback.
    #[serde(rename = "hook")]
    Hook {
        hook_type: String,
        tool_name: String,
        #[serde(default)]
        input: serde_json::Value,
        #[serde(default)]
        output: Option<serde_json::Value>,
    },
}

/// A control response from the SDK to the CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    /// Message type discriminator — always `"control_response"`.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Must match the request's `request_id`.
    pub request_id: String,
    /// The response body.
    pub response: ControlResponseBody,
}

/// Body of a control response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponseBody {
    /// Whether the action is allowed.
    pub allowed: bool,
    /// Optional reason for denial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ControlResponse {
    /// Create an "allowed" response.
    pub fn allow(request_id: String) -> Self {
        Self {
            msg_type: "control_response".to_string(),
            request_id,
            response: ControlResponseBody {
                allowed: true,
                reason: None,
            },
        }
    }

    /// Create a "denied" response with a reason.
    pub fn deny(request_id: String, reason: impl Into<String>) -> Self {
        Self {
            msg_type: "control_response".to_string(),
            request_id,
            response: ControlResponseBody {
                allowed: false,
                reason: Some(reason.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_response_allow_round_trip() {
        let resp = ControlResponse::allow("req-001".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: ControlResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.request_id, "req-001");
        assert_eq!(decoded.msg_type, "control_response");
        assert!(decoded.response.allowed);
        assert!(decoded.response.reason.is_none());
    }

    #[test]
    fn control_response_deny_round_trip() {
        let resp = ControlResponse::deny("req-002".to_string(), "tool not permitted");
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: ControlResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.request_id, "req-002");
        assert!(!decoded.response.allowed);
        assert_eq!(
            decoded.response.reason.as_deref(),
            Some("tool not permitted")
        );
    }

    #[test]
    fn control_request_permission_round_trip() {
        let req = ControlRequest {
            request_id: "req-003".to_string(),
            request: ControlRequestBody::Permission {
                tool_name: "Bash".to_string(),
                input: serde_json::json!({"command": "ls"}),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: ControlRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.request_id, "req-003");
        match decoded.request {
            ControlRequestBody::Permission { tool_name, input } => {
                assert_eq!(tool_name, "Bash");
                assert_eq!(input["command"], "ls");
            }
            _ => panic!("unexpected variant"),
        }
    }

    #[test]
    fn control_request_hook_round_trip() {
        let req = ControlRequest {
            request_id: "req-004".to_string(),
            request: ControlRequestBody::Hook {
                hook_type: "PreToolUse".to_string(),
                tool_name: "Write".to_string(),
                input: serde_json::json!({"file_path": "/tmp/x.rs"}),
                output: None,
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: ControlRequest = serde_json::from_str(&json).unwrap();
        match decoded.request {
            ControlRequestBody::Hook {
                hook_type,
                tool_name,
                ..
            } => {
                assert_eq!(hook_type, "PreToolUse");
                assert_eq!(tool_name, "Write");
            }
            _ => panic!("unexpected variant"),
        }
    }

    #[test]
    fn control_request_initialize_round_trip() {
        let req = ControlRequest {
            request_id: "req-005".to_string(),
            request: ControlRequestBody::Initialize,
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: ControlRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.request_id, "req-005");
        assert!(matches!(decoded.request, ControlRequestBody::Initialize));
    }

    #[test]
    fn deny_omits_reason_when_none_on_allow() {
        let resp = ControlResponse::allow("r".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        // `reason` field must be absent (skip_serializing_if = None).
        assert!(!json.contains("reason"));
    }
}
