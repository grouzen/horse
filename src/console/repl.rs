use rig::{agent::Agent, providers::anthropic};
use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use rig::completion::{Prompt, Usage};

use crate::{
    agent::hooks::ProgressHook,
    console::{colors, markdown, spinner::create_spinner},
};

/// Format a number with k suffix for values >= 1000
fn format_token_count(count: u64) -> String {
    if count < 1000 {
        count.to_string()
    } else {
        let k_value = count as f64 / 1000.0;
        format!("{:.1}k", k_value)
    }
}

/// Generate the prompt string with token usage information
fn format_prompt(usage: Usage) -> String {
    let input_str = format_token_count(usage.input_tokens);
    let output_str = format_token_count(usage.output_tokens);

    if usage.cached_input_tokens > 0 {
        let cached_str = format_token_count(usage.cached_input_tokens);
        format!(
            "{} {} ({} {}), {} {}> ",
            colors::color_dim("in"),
            colors::color_prompt_number(&input_str),
            colors::color_prompt_number(&cached_str),
            colors::color_dim("cached"),
            colors::color_dim("out"),
            colors::color_prompt_number(&output_str)
        )
    } else {
        format!(
            "{} {}, {} {}> ",
            colors::color_dim("in"),
            colors::color_prompt_number(&input_str),
            colors::color_dim("out"),
            colors::color_prompt_number(&output_str)
        )
    }
}

pub struct Repl {
    agent: Agent<anthropic::completion::CompletionModel>,
}

impl Repl {
    pub fn new(agent: Agent<anthropic::completion::CompletionModel>) -> Self {
        Self { agent }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!(
            "{}",
            colors::color_success(">> Ready! Type your queries (Ctrl+C or Ctrl+D to exit)")
        );
        println!();

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        let mut history = Vec::new();
        let hook = ProgressHook::new();

        loop {
            // Prompt with token usage
            print!("{}", format_prompt(hook.get_total_usage()));
            io::stdout().flush()?;

            // Read line
            buffer.clear();
            let bytes_read = handle
                .read_line(&mut buffer)
                .context("Failed to read line from stdin")?;

            // Check for EOF (Ctrl+D)
            if bytes_read == 0 {
                println!("\n{}", colors::color_status(">> Goodbye!"));
                break;
            }

            let input = buffer.trim();

            // Skip empty lines
            if input.is_empty() {
                continue;
            }

            // Start spinner and give it to the hook for control
            let spinner = create_spinner("Processing");
            hook.set_external_spinner(spinner);

            // Execute query with history and progress hook
            match self
                .agent
                .prompt(input)
                .with_history(&mut history)
                .with_hook(hook.clone())
                .await
            {
                Ok(response) => {
                    // Clear any remaining spinner
                    if let Some(s) = hook.get_external_spinner() {
                        s.finish_and_clear();
                    }

                    markdown::render_markdown(&response);
                }
                Err(e) => {
                    // Clear any remaining spinner
                    if let Some(s) = hook.get_external_spinner() {
                        s.finish_and_clear();
                    }

                    eprintln!("{}", colors::color_error(format!(">> Error: {:#}\n", e)));
                }
            }
        }

        Ok(())
    }
}
