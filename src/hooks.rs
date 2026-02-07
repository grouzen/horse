use crate::colors;
use crate::spinner::create_spinner;
use crate::tools::Tools;
use indicatif::ProgressBar;
use rig::agent::{HookAction, PromptHook, ToolCallHookAction};
use rig::completion::{CompletionModel, CompletionResponse, Usage};
use std::sync::{Arc, Mutex};

/// A hook that displays tool calls and results in real-time during agent execution.
/// Skips reasoning tokens by default. Tracks token usage including cache reads.
#[derive(Clone, Debug)]
pub struct ProgressHook {
    total_usage: Arc<Mutex<Usage>>,
    spinner: Arc<Mutex<Option<ProgressBar>>>,
    external_spinner: Arc<Mutex<Option<ProgressBar>>>,
}

impl ProgressHook {
    pub fn new() -> Self {
        Self {
            total_usage: Arc::new(Mutex::new(Usage::default())),
            spinner: Arc::new(Mutex::new(None)),
            external_spinner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_total_usage(&self) -> Usage {
        *self.total_usage.lock().unwrap()
    }

    /// Set the internal tool calling spinner
    pub fn set_spinner(&self, spinner: ProgressBar) {
        if let Ok(mut s) = self.spinner.lock() {
            *s = Some(spinner);
        }
    }

    /// Set the extenal spinner (typically the main "Processing" spinner)
    pub fn set_external_spinner(&self, spinner: ProgressBar) {
        if let Ok(mut s) = self.external_spinner.lock() {
            *s = Some(spinner);
        }
    }

    /// Take and return the current spinner, if any
    pub fn get_spinner(&self) -> Option<ProgressBar> {
        self.spinner.lock().ok().and_then(|mut s| s.take())
    }

    /// Take and return the current spinner, if any
    pub fn get_external_spinner(&self) -> Option<ProgressBar> {
        self.external_spinner.lock().ok().and_then(|mut s| s.take())
    }

    pub fn set_total_usage(&self, delta: Usage) {
        let mut total = self.total_usage.lock().unwrap();
        *total += delta;
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
        // Stop the external spinner (if any) before printing tool call
        if let Some(s) = self.get_external_spinner() {
            s.finish_and_clear();
        }

        // Extract relevant argument based on tool type
        let display_args = Tools::try_from(tool_name)
            .map(|tool| tool.extract_display_args(args))
            .unwrap_or_else(|_| args.to_string());

        let truncated_args = Self::truncate_display(&display_args, 200);
        println!(
            "{}",
            colors::color_debug(format!("\n>> {tool_name}({truncated_args})"))
        );

        // Start spinner for tool execution
        let spinner = create_spinner("Executing tool");
        self.set_spinner(spinner);

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
        // Check if result contains an ToolCallError and display it
        // TODO: would be nice to have a better way to detect errors (open an issue in rig repo?)
        if result.contains("ToolCallError") {
            let truncated_result = Self::truncate_display(result, 500);
            println!(
                "{}",
                colors::color_error(format!(">> Error: {truncated_result}"))
            );
        }

        HookAction::cont()
    }

    async fn on_completion_response(
        &self,
        _prompt: &rig::completion::Message,
        response: &CompletionResponse<M::Response>,
    ) -> HookAction {
        // Stop tool spinner before printing output
        if let Some(s) = self.get_spinner() {
            s.finish_and_clear();
        }

        // Extract and accumulate token usage
        self.set_total_usage(response.usage);

        HookAction::cont()
    }
}

impl Default for ProgressHook {
    fn default() -> Self {
        Self::new()
    }
}
