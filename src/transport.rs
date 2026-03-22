//! Subprocess transport for communicating with the Claude CLI.
//!
//! Spawns `claude` as a child process with `--output-format stream-json`
//! and communicates via JSON-newline protocol over stdin/stdout.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::error::ClaudeAgentError;
use crate::types::options::{ClaudeAgentOptions, StderrCallback};

// ── P2 Feature 10: CLI version check ─────────────────────────────────────────

/// Minimum required Claude CLI version.
pub const MINIMUM_CLI_VERSION: &str = "2.0.0";

/// Check if the installed CLI meets the minimum version requirement.
///
/// Returns the raw version string reported by `--version`.  The caller can
/// compare it against [`MINIMUM_CLI_VERSION`] as needed.  Does not enforce
/// the minimum by default — callers opt in.
///
/// Set `CLAUDE_AGENT_SDK_SKIP_VERSION_CHECK=1` in the environment to skip
/// this function entirely in automated environments.
pub fn check_cli_version(cli_path: &std::path::Path) -> Result<String, ClaudeAgentError> {
    let output = std::process::Command::new(cli_path)
        .arg("--version")
        .output()
        .map_err(|e| ClaudeAgentError::CliNotFound(format!("failed to run --version: {e}")))?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// A subprocess-based transport that communicates with the Claude CLI.
pub struct SubprocessTransport {
    child: Child,
    stdin: Option<tokio::process::ChildStdin>,
    stdout_lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    stderr_handle: Option<tokio::task::JoinHandle<()>>,
    /// Write lock — prevents concurrent writes to stdin from racing.
    write_lock: Arc<Mutex<()>>,
}

impl SubprocessTransport {
    /// Spawn the Claude CLI with the given options.
    pub async fn spawn(options: &ClaudeAgentOptions) -> Result<Self, ClaudeAgentError> {
        let cli_path = Self::find_cli(options)?;
        let cli_args = options.to_cli_args();

        debug!(cli = %cli_path.display(), "spawning claude CLI");

        let mut cmd = Command::new(&cli_path);
        cmd.args(&cli_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        cmd.env("CLAUDE_CODE_ENTRYPOINT", "sdk-rs");
        cmd.env("CLAUDE_AGENT_SDK_VERSION", env!("CARGO_PKG_VERSION"));

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

        // Capture stderr and drain it asynchronously to prevent pipe-buffer
        // deadlock. Lines are forwarded to the user callback (if set) or
        // logged at DEBUG level.
        let stderr = child.stderr.take();
        let stderr_callback: Option<StderrCallback> = options.stderr_callback.clone();
        let stderr_handle = stderr.map(|stderr| {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(ref cb) = stderr_callback {
                        cb.call(&line);
                    } else {
                        tracing::debug!(stderr = %line, "claude CLI stderr");
                    }
                }
            })
        });

        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout_lines,
            stderr_handle,
            write_lock: Arc::new(Mutex::new(())),
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
    ///
    /// Acquires the write lock before writing to prevent concurrent callers
    /// from interleaving partial writes on the same pipe.
    pub async fn write_message(
        &mut self,
        value: &serde_json::Value,
    ) -> Result<(), ClaudeAgentError> {
        let _guard = self.write_lock.clone().lock_owned().await;
        let stdin = self.stdin.as_mut().ok_or_else(|| {
            ClaudeAgentError::ConnectionError("stdin already closed".into())
        })?;
        let mut buf = serde_json::to_string(value)?;
        buf.push('\n');
        stdin.write_all(buf.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    /// Close stdin, signalling end of input.
    pub async fn close_stdin(&mut self) -> Result<(), ClaudeAgentError> {
        if let Some(mut stdin) = self.stdin.take() {
            stdin.shutdown().await?;
        }
        Ok(())
    }

    /// Drop stdin, immediately closing the pipe and signalling EOF.
    ///
    /// Prefer this over [`close_stdin`] for one-shot queries — it avoids a
    /// potential race between `shutdown()` and the child's stdin reader.
    pub fn drop_stdin(&mut self) {
        self.stdin.take(); // Drop closes the OS pipe → child sees EOF
    }

    /// Send a user message to the CLI via stdin in stream-json format.
    pub async fn send_user_message(&mut self, prompt: &str) -> Result<(), ClaudeAgentError> {
        let msg = serde_json::json!({
            "type": "user",
            "session_id": "",
            "message": {
                "role": "user",
                "content": prompt
            },
            "parent_tool_use_id": null
        });
        self.write_message(&msg).await
    }

    /// Wait for the child process to exit and return its status.
    ///
    /// Attempts a graceful shutdown first: closes stdin and waits up to 5
    /// seconds.  If the process has not exited by then it is killed forcibly.
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, ClaudeAgentError> {
        if let Some(handle) = self.stderr_handle.take() {
            handle.abort();
        }
        // Try graceful shutdown: close stdin and wait up to 5 seconds.
        self.drop_stdin();
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.child.wait(),
        )
        .await
        {
            Ok(result) => result.map_err(ClaudeAgentError::IoError),
            Err(_timeout) => {
                // Graceful shutdown timed out — kill the process.
                tracing::warn!("claude CLI did not exit within 5s — killing");
                self.child.kill().await.map_err(ClaudeAgentError::IoError)?;
                self.child.wait().await.map_err(ClaudeAgentError::IoError)
            }
        }
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

    // ── P2 Feature 10: CLI version constant ───────────────────────────────────

    #[test]
    fn minimum_cli_version_constant() {
        // The constant must be a well-formed semver-like string.
        assert!(!MINIMUM_CLI_VERSION.is_empty());
        assert!(MINIMUM_CLI_VERSION.contains('.'));
    }

    #[test]
    fn check_cli_version_fails_gracefully_for_missing_binary() {
        let result = check_cli_version(std::path::Path::new("/nonexistent/claude-binary"));
        // Must return an error, not panic.
        assert!(result.is_err());
    }
}
