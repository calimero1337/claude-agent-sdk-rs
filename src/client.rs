//! Bidirectional Claude CLI client for interactive sessions.
//!
//! Use [`ClaudeClient`] when you need to send multiple messages in a
//! conversation, handle tool results, or control the session dynamically.
//!
//! For simple one-shot queries, use [`crate::query::query()`] instead.

use std::sync::Arc;

use crate::error::ClaudeAgentError;
use crate::hooks::HookHandler;
use crate::transport::SubprocessTransport;
use crate::types::control::{ControlRequest, ControlRequestBody, ControlResponse};
use crate::types::message::{Message, ResultMessage};
use crate::types::options::ClaudeAgentOptions;

/// A bidirectional client for interactive Claude sessions.
pub struct ClaudeClient {
    transport: Option<SubprocessTransport>,
    options: ClaudeAgentOptions,
    session_id: Option<String>,
    permission_handler: Option<Arc<dyn Fn(&str, &serde_json::Value) -> bool + Send + Sync>>,
    hook_handler: Option<Arc<dyn HookHandler>>,
}

impl ClaudeClient {
    /// Create a new client with the given options.
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self {
            transport: None,
            options,
            session_id: None,
            permission_handler: None,
            hook_handler: None,
        }
    }

    /// Set a permission handler for runtime tool approval.
    ///
    /// The handler receives `(tool_name, input)` and returns `true` to allow
    /// or `false` to deny the tool call.
    pub fn with_permission_handler(
        mut self,
        handler: Arc<dyn Fn(&str, &serde_json::Value) -> bool + Send + Sync>,
    ) -> Self {
        self.permission_handler = Some(handler);
        self
    }

    /// Set a hook handler for pre/post tool use interception.
    pub fn with_hook_handler(mut self, handler: Arc<dyn HookHandler>) -> Self {
        self.hook_handler = Some(handler);
        self
    }

    /// Connect and start a session with an initial prompt.
    pub async fn connect(&mut self, prompt: &str) -> Result<(), ClaudeAgentError> {
        let mut transport = SubprocessTransport::spawn(&self.options).await?;
        transport.send_user_message(prompt).await?;
        self.transport = Some(transport);
        Ok(())
    }

    /// Read the next response, collecting messages until a ResultMessage is received.
    ///
    /// Control requests (permission callbacks, hooks) are handled transparently:
    /// responses are written back to the CLI and the messages are not included
    /// in the returned list.
    pub async fn receive_response(&mut self) -> Result<Vec<Message>, ClaudeAgentError> {
        // Verify we are connected before entering the loop.
        if self.transport.is_none() {
            return Err(ClaudeAgentError::ConnectionError("not connected".into()));
        }

        let mut messages = Vec::new();
        loop {
            // Re-borrow transport each iteration so we can release the mutable
            // borrow before calling `handle_control_request` (which takes &self).
            let raw = match self
                .transport
                .as_mut()
                .expect("checked above")
                .read_message()
                .await?
            {
                Some(v) => v,
                None => break,
            };

            // Check for control requests (permission/hook callbacks from CLI).
            if raw.get("type").and_then(|t| t.as_str()) == Some("control_request") {
                if let Ok(req) = serde_json::from_value::<ControlRequest>(raw.clone()) {
                    // Compute the response while `transport` is NOT borrowed.
                    let response = self.handle_control_request(&req);
                    let response_value = serde_json::to_value(&response)?;
                    // Now reborrow transport to write the response.
                    self.transport
                        .as_mut()
                        .expect("checked above")
                        .write_message(&response_value)
                        .await?;
                    continue; // Don't add to messages list
                }
            }

            if let Some(msg) = Message::parse(&raw) {
                let is_result = matches!(&msg, Message::Result(r) if {
                    self.session_id = Some(r.session_id.clone());
                    true
                });
                messages.push(msg);
                if is_result {
                    break;
                }
            }
        }
        Ok(messages)
    }

    /// Send a follow-up message in an existing session.
    pub async fn send(&mut self, message: &str) -> Result<(), ClaudeAgentError> {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| ClaudeAgentError::ConnectionError("not connected".into()))?;

        transport.send_user_message(message).await
    }

    /// Get the current session ID (available after first response).
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Disconnect and wait for the process to exit.
    pub async fn disconnect(mut self) -> Result<(), ClaudeAgentError> {
        if let Some(mut transport) = self.transport.take() {
            transport.close_stdin().await?;
            transport.wait().await?;
        }
        Ok(())
    }

    /// Handle an inbound control request and produce the appropriate response.
    fn handle_control_request(&self, req: &ControlRequest) -> ControlResponse {
        match &req.request {
            ControlRequestBody::Permission { tool_name, input } => {
                if let Some(handler) = &self.permission_handler {
                    if handler(tool_name, input) {
                        ControlResponse::allow(req.request_id.clone())
                    } else {
                        ControlResponse::deny(
                            req.request_id.clone(),
                            format!("Permission denied for tool: {}", tool_name),
                        )
                    }
                } else {
                    // No handler registered — allow by default.
                    ControlResponse::allow(req.request_id.clone())
                }
            }
            ControlRequestBody::Hook {
                hook_type,
                tool_name,
                input,
                output,
            } => {
                if let Some(handler) = &self.hook_handler {
                    match hook_type.as_str() {
                        "PreToolUse" => {
                            if handler.pre_tool_use(tool_name, input) {
                                ControlResponse::allow(req.request_id.clone())
                            } else {
                                ControlResponse::deny(req.request_id.clone(), "Blocked by hook")
                            }
                        }
                        "PostToolUse" => {
                            handler.post_tool_use(tool_name, input, output);
                            ControlResponse::allow(req.request_id.clone())
                        }
                        _ => ControlResponse::allow(req.request_id.clone()),
                    }
                } else {
                    ControlResponse::allow(req.request_id.clone())
                }
            }
            ControlRequestBody::Initialize => ControlResponse::allow(req.request_id.clone()),
        }
    }
}

/// Extract the final text from a list of messages.
pub fn extract_response_text(messages: &[Message]) -> String {
    let mut parts = Vec::new();
    for msg in messages {
        if let Message::Assistant(a) = msg {
            let text = a.text_content();
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }
    parts.join("\n\n")
}

/// Extract the ResultMessage from a list of messages (if any).
pub fn extract_result(messages: &[Message]) -> Option<&ResultMessage> {
    messages.iter().find_map(|m| m.as_result())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::NoopHookHandler;
    use crate::types::control::ControlRequestBody;

    fn make_client() -> ClaudeClient {
        ClaudeClient::new(ClaudeAgentOptions::default())
    }

    #[test]
    fn handle_initialize_allows() {
        let client = make_client();
        let req = ControlRequest {
            request_id: "r1".to_string(),
            request: ControlRequestBody::Initialize,
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
        assert_eq!(resp.request_id, "r1");
        assert_eq!(resp.msg_type, "control_response");
    }

    #[test]
    fn handle_permission_no_handler_allows_by_default() {
        let client = make_client();
        let req = ControlRequest {
            request_id: "r2".to_string(),
            request: ControlRequestBody::Permission {
                tool_name: "Bash".to_string(),
                input: serde_json::json!({"command": "ls"}),
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }

    #[test]
    fn handle_permission_handler_deny() {
        let client = make_client().with_permission_handler(Arc::new(|_tool, _input| false));
        let req = ControlRequest {
            request_id: "r3".to_string(),
            request: ControlRequestBody::Permission {
                tool_name: "Bash".to_string(),
                input: serde_json::Value::Null,
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(!resp.response.allowed);
        assert!(resp.response.reason.is_some());
    }

    #[test]
    fn handle_permission_handler_allow() {
        let client = make_client().with_permission_handler(Arc::new(|_tool, _input| true));
        let req = ControlRequest {
            request_id: "r4".to_string(),
            request: ControlRequestBody::Permission {
                tool_name: "Read".to_string(),
                input: serde_json::Value::Null,
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }

    #[test]
    fn handle_hook_no_handler_allows() {
        let client = make_client();
        let req = ControlRequest {
            request_id: "r5".to_string(),
            request: ControlRequestBody::Hook {
                hook_type: "PreToolUse".to_string(),
                tool_name: "Write".to_string(),
                input: serde_json::Value::Null,
                output: None,
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }

    #[test]
    fn handle_hook_noop_handler_allows() {
        let client = make_client().with_hook_handler(Arc::new(NoopHookHandler));
        let req = ControlRequest {
            request_id: "r6".to_string(),
            request: ControlRequestBody::Hook {
                hook_type: "PreToolUse".to_string(),
                tool_name: "Write".to_string(),
                input: serde_json::Value::Null,
                output: None,
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }

    #[test]
    fn handle_post_tool_use_hook_allows() {
        let client = make_client().with_hook_handler(Arc::new(NoopHookHandler));
        let req = ControlRequest {
            request_id: "r7".to_string(),
            request: ControlRequestBody::Hook {
                hook_type: "PostToolUse".to_string(),
                tool_name: "Write".to_string(),
                input: serde_json::Value::Null,
                output: Some(serde_json::json!({"result": "ok"})),
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }

    #[test]
    fn handle_unknown_hook_type_allows() {
        let client = make_client().with_hook_handler(Arc::new(NoopHookHandler));
        let req = ControlRequest {
            request_id: "r8".to_string(),
            request: ControlRequestBody::Hook {
                hook_type: "UnknownHookType".to_string(),
                tool_name: "Bash".to_string(),
                input: serde_json::Value::Null,
                output: None,
            },
        };
        let resp = client.handle_control_request(&req);
        assert!(resp.response.allowed);
    }
}
