//! Abstract transport trait for Claude CLI communication.
//!
//! Implement this trait to provide custom transports (e.g., for testing,
//! remote execution, or alternative CLI backends).

use async_trait::async_trait;

use crate::error::ClaudeAgentError;
use crate::transport::SubprocessTransport;

/// Abstract transport for communicating with the Claude CLI.
///
/// Implement this trait to provide custom transports (e.g., for testing,
/// remote execution, or alternative CLI backends).
#[async_trait]
pub trait Transport: Send + Sync {
    /// Read the next JSON message from the CLI.
    async fn read_message(&mut self) -> Result<Option<serde_json::Value>, ClaudeAgentError>;

    /// Write a JSON message to the CLI.
    async fn write_message(&mut self, value: &serde_json::Value) -> Result<(), ClaudeAgentError>;

    /// Send a user message in stream-json format.
    async fn send_user_message(&mut self, prompt: &str) -> Result<(), ClaudeAgentError>;

    /// Signal end of input.
    async fn close_input(&mut self) -> Result<(), ClaudeAgentError>;

    /// Check if the transport is ready (connected and alive).
    fn is_ready(&self) -> bool;

    /// Wait for the underlying process to exit.
    async fn wait(&mut self) -> Result<std::process::ExitStatus, ClaudeAgentError>;
}

#[async_trait]
impl Transport for SubprocessTransport {
    async fn read_message(&mut self) -> Result<Option<serde_json::Value>, ClaudeAgentError> {
        SubprocessTransport::read_message(self).await
    }

    async fn write_message(&mut self, value: &serde_json::Value) -> Result<(), ClaudeAgentError> {
        SubprocessTransport::write_message(self, value).await
    }

    async fn send_user_message(&mut self, prompt: &str) -> Result<(), ClaudeAgentError> {
        SubprocessTransport::send_user_message(self, prompt).await
    }

    async fn close_input(&mut self) -> Result<(), ClaudeAgentError> {
        SubprocessTransport::close_stdin(self).await
    }

    /// Always returns `true` for a spawned subprocess transport — the child
    /// process was alive at construction time. A full liveness check (e.g.,
    /// `try_wait`) would require mutable access and is therefore left for
    /// callers that need it.
    fn is_ready(&self) -> bool {
        true
    }

    async fn wait(&mut self) -> Result<std::process::ExitStatus, ClaudeAgentError> {
        SubprocessTransport::wait(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time check: SubprocessTransport satisfies the Transport trait.
    fn _assert_transport_impl<T: Transport>() {}

    #[allow(dead_code)]
    fn _check() {
        _assert_transport_impl::<SubprocessTransport>();
    }

    /// Verify a mock transport can be implemented and used through the trait.
    struct MockTransport {
        messages: Vec<serde_json::Value>,
        ready: bool,
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn read_message(
            &mut self,
        ) -> Result<Option<serde_json::Value>, ClaudeAgentError> {
            Ok(self.messages.pop())
        }

        async fn write_message(
            &mut self,
            value: &serde_json::Value,
        ) -> Result<(), ClaudeAgentError> {
            self.messages.push(value.clone());
            Ok(())
        }

        async fn send_user_message(&mut self, prompt: &str) -> Result<(), ClaudeAgentError> {
            self.messages.push(serde_json::json!({ "role": "user", "content": prompt }));
            Ok(())
        }

        async fn close_input(&mut self) -> Result<(), ClaudeAgentError> {
            self.ready = false;
            Ok(())
        }

        fn is_ready(&self) -> bool {
            self.ready
        }

        async fn wait(&mut self) -> Result<std::process::ExitStatus, ClaudeAgentError> {
            // In tests we can't easily construct an ExitStatus, so we just
            // signal not-ready and return an error to indicate no real process.
            Err(ClaudeAgentError::ConnectionError("mock transport has no process".into()))
        }
    }

    #[tokio::test]
    async fn mock_transport_write_read_roundtrip() {
        let mut transport = MockTransport { messages: vec![], ready: true };
        assert!(transport.is_ready());

        let msg = serde_json::json!({"type": "ping"});
        transport.write_message(&msg).await.unwrap();

        let received = transport.read_message().await.unwrap();
        assert_eq!(received, Some(serde_json::json!({"type": "ping"})));
    }

    #[tokio::test]
    async fn mock_transport_close_input_marks_not_ready() {
        let mut transport = MockTransport { messages: vec![], ready: true };
        transport.close_input().await.unwrap();
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn mock_transport_send_user_message_enqueues_message() {
        let mut transport = MockTransport { messages: vec![], ready: true };
        transport.send_user_message("hello").await.unwrap();
        let msg = transport.read_message().await.unwrap().unwrap();
        assert_eq!(msg["role"], "user");
        assert_eq!(msg["content"], "hello");
    }

    #[tokio::test]
    async fn mock_transport_wait_returns_error() {
        let mut transport = MockTransport { messages: vec![], ready: true };
        let result = transport.wait().await;
        assert!(result.is_err());
    }
}
