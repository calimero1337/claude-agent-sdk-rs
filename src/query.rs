//! One-shot query interface — fire-and-forget, returns all messages.
//!
//! This is the simple entry point for interacting with the Claude CLI.
//! For bidirectional/interactive sessions, use [`crate::client::ClaudeClient`].

use crate::error::ClaudeAgentError;
use crate::transport::SubprocessTransport;
use crate::types::message::{Message, ResultMessage};
use crate::types::options::ClaudeAgentOptions;

/// Result of a completed query.
#[derive(Debug)]
pub struct QueryResult {
    /// All messages received during the query.
    pub messages: Vec<Message>,
    /// The final result message (contains cost, usage, session_id).
    pub result: Option<ResultMessage>,
}

impl QueryResult {
    /// Extract the final text response from the assistant.
    pub fn response_text(&self) -> String {
        let mut parts = Vec::new();
        for msg in &self.messages {
            if let Message::Assistant(a) = msg {
                let text = a.text_content();
                if !text.is_empty() {
                    parts.push(text);
                }
            }
        }
        parts.join("\n\n")
    }
}

/// Execute a one-shot query to the Claude CLI.
///
/// Spawns `claude` as a subprocess, sends the prompt, collects all messages,
/// and returns the complete result.
///
/// # Example
///
/// ```rust,no_run
/// use claude_agent_sdk::{query, ClaudeAgentOptions};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = query("What is 2 + 2?", &ClaudeAgentOptions::default()).await?;
/// println!("{}", result.response_text());
/// # Ok(())
/// # }
/// ```
pub async fn query(
    prompt: &str,
    options: &ClaudeAgentOptions,
) -> Result<QueryResult, ClaudeAgentError> {
    let mut transport = SubprocessTransport::spawn(prompt, options).await?;

    let mut messages = Vec::new();
    let mut result_msg = None;

    // Read all messages until EOF.
    while let Some(raw) = transport.read_message().await? {
        if let Some(msg) = Message::parse(&raw) {
            match &msg {
                Message::Result(r) => {
                    result_msg = Some(r.clone());
                }
                _ => {}
            }
            messages.push(msg);
        }
    }

    // Wait for process to complete.
    let status = transport.wait().await?;
    if !status.success() && result_msg.is_none() {
        return Err(ClaudeAgentError::ProcessError {
            exit_code: status.code(),
            stderr: "claude process exited with non-zero status".into(),
        });
    }

    Ok(QueryResult {
        messages,
        result: result_msg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_result_response_text_empty() {
        let result = QueryResult {
            messages: vec![],
            result: None,
        };
        assert_eq!(result.response_text(), "");
    }
}
