use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::CompletionModel;

/// A hook that displays tool calls and results in real-time during agent execution.
/// Skips reasoning tokens by default.
#[derive(Clone, Debug)]
pub struct ProgressHook;

impl ProgressHook {
    pub fn new() -> Self {
        Self
    }

    /// Truncate long strings with an ellipsis for display
    fn truncate_display(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

impl<M> PromptHook<M> for ProgressHook
where
    M: CompletionModel,
{
    async fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        // Print tool call notification
        let truncated_args = Self::truncate_display(args, 200);
        println!("🔧 Calling: {tool_name}({truncated_args})");

        ToolCallHookAction::cont()
    }

    async fn on_tool_result(
        &self,
        _tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        _args: &str,
        result: &str,
    ) -> HookAction {
        // Print truncated result or error summary
        let display_result = if result.len() > 500 {
            Self::truncate_display(result, 500)
        } else {
            result.to_string()
        };

        // Check if result looks like an error
        if display_result.to_lowercase().contains("error") {
            println!("   ❌ {display_result}");
        } else {
            println!("   ✅ Result: {display_result}");
        }

        HookAction::cont()
    }
}

impl Default for ProgressHook {
    fn default() -> Self {
        Self::new()
    }
}
