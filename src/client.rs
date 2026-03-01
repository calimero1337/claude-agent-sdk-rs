//! Bidirectional Claude CLI client for interactive sessions.
//!
//! Use [`ClaudeClient`] when you need to send multiple messages in a
//! conversation, handle tool results, or control the session dynamically.
//!
//! For simple one-shot queries, use [`crate::query::query()`] instead.

use crate::error::ClaudeAgentError;
use crate::transport::SubprocessTransport;
use crate::types::message::{Message, ResultMessage};
use crate::types::options::ClaudeAgentOptions;

/// A bidirectional client for interactive Claude sessions.
pub struct ClaudeClient {
    transport: Option<SubprocessTransport>,
    options: ClaudeAgentOptions,
    session_id: Option<String>,
}

impl ClaudeClient {
    /// Create a new client with the given options.
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self {
            transport: None,
            options,
            session_id: None,
        }
    }

    /// Connect and start a session with an initial prompt.
    pub async fn connect(&mut self, prompt: &str) -> Result<(), ClaudeAgentError> {
        let transport = SubprocessTransport::spawn(prompt, &self.options).await?;
        self.transport = Some(transport);
        Ok(())
    }

    /// Read the next response, collecting messages until a ResultMessage is received.
    pub async fn receive_response(&mut self) -> Result<Vec<Message>, ClaudeAgentError> {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| ClaudeAgentError::ConnectionError("not connected".into()))?;

        let mut messages = Vec::new();
        while let Some(raw) = transport.read_message().await? {
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

        let msg = serde_json::json!({
            "type": "user",
            "content": message
        });
        transport.write_message(&msg).await
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
