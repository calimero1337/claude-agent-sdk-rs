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
}
