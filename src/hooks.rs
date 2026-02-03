use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Usage};
use std::sync::{Arc, Mutex};

/// A hook that displays tool calls and results in real-time during agent execution.
/// Skips reasoning tokens by default. Tracks token usage including cache reads.
#[derive(Clone, Debug)]
pub struct ProgressHook {
    total_usage: Arc<Mutex<Usage>>,
}

impl ProgressHook {
    pub fn new() -> Self {
        Self {
            total_usage: Arc::new(Mutex::new(Usage::default())),
        }
    }

    pub fn total_usage(&self) -> Usage {
        *self.total_usage.lock().unwrap()
    }

    /// Truncate long strings with an ellipsis for display
    fn truncate_display(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            // Find a valid UTF-8 character boundary at or before max_len
            let truncate_at = s
                .char_indices()
                .take_while(|(idx, _)| *idx < max_len)
                .last()
                .map(|(idx, ch)| idx + ch.len_utf8())
                .unwrap_or(0);
            format!("{}...", &s[..truncate_at])
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

    async fn on_completion_response(
        &self,
        _prompt: &rig::completion::Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        // Extract and accumulate token usage
        let usage = response.usage;
        let mut total = self.total_usage.lock().unwrap();
        *total += usage;

        // Display token usage including cache reads
        println!(
            "\n📊 Tokens: in={} out={} total={}",
            usage.input_tokens, usage.output_tokens, usage.total_tokens
        );

        if usage.cached_input_tokens > 0 {
            println!("   💾 Cache read: {} tokens", usage.cached_input_tokens);
        }

        HookAction::cont()
    }
}

impl Default for ProgressHook {
    fn default() -> Self {
        Self::new()
    }
}
