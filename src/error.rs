//! Error types for the Claude Agent SDK.

use thiserror::Error;

/// The main error type for the Claude Agent SDK.
#[derive(Debug, Error)]
pub enum ClaudeAgentError {
    /// The Claude CLI binary was not found.
    #[error("Claude CLI not found: {0}")]
    CliNotFound(String),

    /// Connection to the CLI process failed.
    #[error("CLI connection error: {0}")]
    ConnectionError(String),

    /// The CLI process exited with an error.
    #[error("Process error (exit code {exit_code:?}): {stderr}")]
    ProcessError {
        exit_code: Option<i32>,
        stderr: String,
    },

    /// The agent exceeded the maximum number of turns.
    #[error("Max turns ({0}) exceeded")]
    MaxTurnsExceeded(usize),

    /// Error parsing a JSON message from the CLI.
    #[error("JSON decode error on line: {line}")]
    JsonDecodeError {
        line: String,
        #[source]
        source: serde_json::Error,
    },

    /// Error parsing a message type/structure.
    #[error("Message parse error: {0}")]
    MessageParseError(String),

    /// A tool execution error.
    #[error("Tool execution error in '{tool_name}': {message}")]
    ToolError { tool_name: String, message: String },

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// I/O error from the subprocess.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// An unexpected/unknown error.
    #[error("{0}")]
    Other(String),
}
