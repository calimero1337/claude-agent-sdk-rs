//! # Claude Agent SDK
//!
//! A Rust port of the [claude-agent-sdk-python](https://github.com/anthropics/claude-agent-sdk-python).
//!
//! This SDK provides a high-level interface for running Claude agents by
//! spawning the Claude CLI as a subprocess and communicating via JSON-newline
//! protocol over stdin/stdout.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use claude_agent_sdk::{query, ClaudeAgentOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let result = query("What is 2 + 2?", &ClaudeAgentOptions::default()).await?;
//!     println!("{}", result.response_text());
//!     Ok(())
//! }
//! ```
//!
//! ## Interactive Sessions
//!
//! ```rust,no_run
//! use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
//!     client.connect("Hello!").await?;
//!     let messages = client.receive_response().await?;
//!     // Send follow-up...
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod hooks;
pub mod query;
pub mod session_management;
pub mod transport;
pub mod transport_trait;
pub mod types;

// ── Public API re-exports ────────────────────────────────────────────────────

pub use client::ClaudeClient;
pub use error::ClaudeAgentError;
pub use hooks::{HookHandler, NoopHookHandler};
pub use query::{query, QueryResult};
pub use session_management::{sanitize_tag, SessionInfo, SessionMessage};
pub use transport::MINIMUM_CLI_VERSION;
pub use transport_trait::Transport;
pub use types::control::{
    ControlRequest, ControlRequestBody, ControlResponse, ControlResponseBody,
    PermissionResultAllow, PermissionRuleValue, PermissionUpdate,
};
pub use types::event::{ContentDelta, StopReason, StreamEvent, TokenUsage};
pub use types::message::{
    AssistantMessage, ContentBlock, Message, RateLimitEvent, RateLimitInfo, RateLimitStatus,
    ResultMessage, Role, StreamEventMessage, SystemMessage, TaskNotificationMessage,
    TaskProgressMessage, TaskStartedMessage, UserMessage,
};
pub use types::options::{
    AgentDefinition, ClaudeAgentOptions, Effort, McpServerConfig, PermissionMode,
    SandboxIgnoreViolations, SandboxNetworkConfig, SandboxSettings, SdkPluginConfig,
    StderrCallback, ThinkingConfig,
};
pub use types::params::{AgentParams, AgentParamsBuilder, MaxTurns};
pub use types::tool::{Tool, ToolChoice, ToolResult, ToolResultContent, ToolUseBlock};

/// Re-export of the Result type with [`ClaudeAgentError`] as the error.
pub type Result<T> = std::result::Result<T, ClaudeAgentError>;
