//! Hook handler trait for intercepting tool use in Claude sessions.

use serde_json::Value;

/// Trait for handling pre/post tool use hooks from the Claude CLI.
///
/// Implement this trait to intercept tool calls during agent execution.
/// The default implementations allow all operations.
pub trait HookHandler: Send + Sync {
    /// Called before a tool is executed. Return `false` to block the tool call.
    fn pre_tool_use(&self, tool_name: &str, input: &Value) -> bool {
        let _ = (tool_name, input);
        true
    }

    /// Called after a tool completes successfully.
    fn post_tool_use(&self, tool_name: &str, input: &Value, output: &Option<Value>) {
        let _ = (tool_name, input, output);
    }

    // ── P2 Feature 7: 8 additional hook event types ───────────────────────────

    /// Called after a tool fails. The default no-op implementation ignores the event.
    fn post_tool_use_failure(&self, tool_name: &str, input: &Value, error: &str) {
        let _ = (tool_name, input, error);
    }

    /// Called when the agent stops. `reason` is a human-readable explanation.
    fn on_stop(&self, reason: &str) {
        let _ = reason;
    }

    /// Called when a sub-agent stops. Returns immediately by default.
    fn on_subagent_stop(&self, agent_id: &str, reason: &str) {
        let _ = (agent_id, reason);
    }

    /// Called before a user prompt is submitted.
    ///
    /// Return `false` to cancel submission. The default implementation allows
    /// all prompts.
    fn on_user_prompt_submit(&self, prompt: &str) -> bool {
        let _ = prompt;
        true
    }

    /// Called before the context window is compacted.
    fn on_pre_compact(&self) {}

    /// Called when the CLI emits a notification message.
    fn on_notification(&self, message: &str) {
        let _ = message;
    }

    /// Called when a sub-agent starts. `agent_type` identifies the agent kind.
    fn on_subagent_start(&self, agent_id: &str, agent_type: &str) {
        let _ = (agent_id, agent_type);
    }

    /// Called when the CLI requests permission before performing an action.
    ///
    /// Return `false` to deny. The default implementation allows all requests.
    fn on_permission_request(&self, tool_name: &str, input: &Value) -> bool {
        let _ = (tool_name, input);
        true
    }
}

/// A no-op hook handler that allows everything. Used as the default.
pub struct NoopHookHandler;

impl HookHandler for NoopHookHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    struct BlockAllHook;
    impl HookHandler for BlockAllHook {
        fn pre_tool_use(&self, _tool_name: &str, _input: &Value) -> bool {
            false
        }
    }

    #[test]
    fn noop_allows_everything() {
        let h = NoopHookHandler;
        assert!(h.pre_tool_use("Bash", &Value::Null));
        // post_tool_use is infallible — just verify it doesn't panic.
        h.post_tool_use("Bash", &Value::Null, &None);
    }

    #[test]
    fn custom_hook_can_block() {
        let h = BlockAllHook;
        assert!(!h.pre_tool_use("Write", &serde_json::json!({"file_path": "/tmp/x"})));
    }

    // ── P2 Feature 7: new hook methods ────────────────────────────────────────

    #[test]
    fn noop_post_tool_use_failure_does_not_panic() {
        let h = NoopHookHandler;
        h.post_tool_use_failure("Bash", &Value::Null, "exit code 1");
    }

    #[test]
    fn noop_on_stop_does_not_panic() {
        let h = NoopHookHandler;
        h.on_stop("max_turns");
    }

    #[test]
    fn noop_on_subagent_stop_does_not_panic() {
        let h = NoopHookHandler;
        h.on_subagent_stop("agent-1", "completed");
    }

    #[test]
    fn noop_on_user_prompt_submit_allows() {
        let h = NoopHookHandler;
        assert!(h.on_user_prompt_submit("hello"));
    }

    #[test]
    fn noop_on_pre_compact_does_not_panic() {
        let h = NoopHookHandler;
        h.on_pre_compact();
    }

    #[test]
    fn noop_on_notification_does_not_panic() {
        let h = NoopHookHandler;
        h.on_notification("rate limit approaching");
    }

    #[test]
    fn noop_on_subagent_start_does_not_panic() {
        let h = NoopHookHandler;
        h.on_subagent_start("agent-2", "code_review");
    }

    #[test]
    fn noop_on_permission_request_allows() {
        let h = NoopHookHandler;
        assert!(h.on_permission_request("Write", &serde_json::json!({"file_path": "/tmp/x"})));
    }

    struct BlockingHook;
    impl HookHandler for BlockingHook {
        fn on_user_prompt_submit(&self, _prompt: &str) -> bool {
            false
        }
        fn on_permission_request(&self, _tool_name: &str, _input: &Value) -> bool {
            false
        }
    }

    #[test]
    fn custom_hook_can_block_user_prompt() {
        let h = BlockingHook;
        assert!(!h.on_user_prompt_submit("dangerous prompt"));
    }

    #[test]
    fn custom_hook_can_block_permission_request() {
        let h = BlockingHook;
        assert!(!h.on_permission_request("Bash", &Value::Null));
    }
}
