//! Subprocess transport for communicating with the Claude CLI.
//!
//! Spawns `claude` as a child process with `--output-format stream-json`
//! and communicates via JSON-newline protocol over stdin/stdout.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, warn};

use crate::error::ClaudeAgentError;
use crate::types::options::ClaudeAgentOptions;

/// A subprocess-based transport that communicates with the Claude CLI.
pub struct SubprocessTransport {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout_lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
}

impl SubprocessTransport {
    /// Spawn the Claude CLI with the given prompt and options.
    pub async fn spawn(
        prompt: &str,
        options: &ClaudeAgentOptions,
    ) -> Result<Self, ClaudeAgentError> {
        let cli_path = Self::find_cli(options)?;
        let cli_args = options.to_cli_args();

        debug!(cli = %cli_path.display(), "spawning claude CLI");

        let mut cmd = Command::new(&cli_path);
        cmd.args(&cli_args)
            .arg("--print")
            .arg(prompt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // Set working directory if configured.
        if let Some(ref cwd) = options.cwd {
            cmd.current_dir(cwd);
        }

        // Set extra environment variables.
        for (k, v) in &options.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| {
            ClaudeAgentError::ConnectionError(format!(
                "failed to spawn {}: {e}",
                cli_path.display()
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ClaudeAgentError::ConnectionError("stdin not captured".into()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ClaudeAgentError::ConnectionError("stdout not captured".into()))?;

        let stdout_lines = BufReader::new(stdout).lines();

        Ok(Self {
            child,
            stdin,
            stdout_lines,
        })
    }

    /// Read the next JSON message from stdout.
    ///
    /// Returns `None` when the process has closed stdout.
    pub async fn read_message(
        &mut self,
    ) -> Result<Option<serde_json::Value>, ClaudeAgentError> {
        loop {
            match self.stdout_lines.next_line().await? {
                None => return Ok(None), // EOF
                Some(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<serde_json::Value>(trimmed) {
                        Ok(val) => return Ok(Some(val)),
                        Err(e) => {
                            warn!(line = %trimmed, "skipping non-JSON line from CLI");
                            // Non-JSON lines are logged but skipped (e.g. CLI startup messages).
                            // If it looks like JSON but failed to parse, return the error.
                            if trimmed.starts_with('{') {
                                return Err(ClaudeAgentError::JsonDecodeError {
                                    line: trimmed.to_string(),
                                    source: e,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Write a JSON message to stdin (appends newline).
    pub async fn write_message(
        &mut self,
        value: &serde_json::Value,
    ) -> Result<(), ClaudeAgentError> {
        let mut buf = serde_json::to_string(value)?;
        buf.push('\n');
        self.stdin.write_all(buf.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Close stdin, signalling end of input.
    pub async fn close_stdin(&mut self) -> Result<(), ClaudeAgentError> {
        self.stdin.shutdown().await?;
        Ok(())
    }

    /// Wait for the child process to exit and return its status.
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, ClaudeAgentError> {
        self.child.wait().await.map_err(ClaudeAgentError::IoError)
    }

    /// Find the `claude` CLI binary.
    fn find_cli(options: &ClaudeAgentOptions) -> Result<PathBuf, ClaudeAgentError> {
        // 1. Explicit path from options.
        if let Some(ref path) = options.cli_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(ClaudeAgentError::CliNotFound(format!(
                "configured path does not exist: {}",
                path.display()
            )));
        }

        // 2. Search PATH.
        if let Ok(path) = which::which("claude") {
            return Ok(path);
        }

        Err(ClaudeAgentError::CliNotFound(
            "claude CLI not found in PATH. Install it with: npm install -g @anthropic-ai/claude-code".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_cli_with_explicit_path_missing() {
        let opts = ClaudeAgentOptions {
            cli_path: Some("/nonexistent/claude".into()),
            ..Default::default()
        };
        let result = SubprocessTransport::find_cli(&opts);
        assert!(result.is_err());
    }

    #[test]
    fn find_cli_searches_path() {
        // This test verifies the PATH search logic works (claude may or may not be installed).
        let opts = ClaudeAgentOptions::default();
        let _result = SubprocessTransport::find_cli(&opts);
        // We don't assert success because the CLI may not be installed in CI.
    }
}
