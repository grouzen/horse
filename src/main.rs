use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use rig::agent::{Agent, AgentBuilder};
use rig::client::ProviderClient;
use rig::completion::{Prompt, Usage};
use rig::providers::anthropic;

mod colors;
mod hooks;
mod tools;

use hooks::ProgressHook;
use tools::{BashCommand, ReadFile, SearchDocs};

#[derive(Parser, Debug)]
#[command(name = "horse")]
#[command(about = "An agentic search assistant for intelligent directory exploration")]
struct Args {
    /// Target directory to search and execute commands in
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// Claude model to use
    #[arg(short, long, default_value = "claude-sonnet-4-0")]
    model: String,

    /// Maximum number of turns for the agent
    #[arg(short = 't', long, default_value = "20")]
    max_turns: usize,
}

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

/// Gather directory structure by running `find` command
async fn gather_directory_context(base_dir: &Path) -> Result<String> {
    use tokio::process::Command;

    let output = Command::new("find")
        .arg(".")
        .arg("-maxdepth")
        .arg("3")
        .arg("-type")
        .arg("f")
        .current_dir(base_dir)
        .output()
        .await
        .context("Failed to execute find command")?;

    if output.status.success() {
        String::from_utf8(output.stdout).context("Failed to parse find output as UTF-8")
    } else {
        Ok("(Directory listing unavailable)".to_string())
    }
}

/// Load the AGENTS.md file from the target directory if it exists,
/// otherwise return a default preamble.
async fn load_preamble(base_dir: &Path) -> Result<String> {
    let agents_file = base_dir.join("AGENTS.md");
    let mut preamble = if agents_file.exists() {
        println!("{}", colors::color_status(">> Loading AGENTS.md..."));
        tokio::fs::read_to_string(&agents_file)
            .await
            .context("Failed to read AGENTS.md")?
    } else {
        "You are a helpful search assistant. You can read files and execute safe bash commands \
            to help users explore and understand their codebase."
            .to_string()
    };

    // Add directory context
    println!(
        "{}",
        colors::color_status(">> Gathering directory structure...")
    );
    match gather_directory_context(base_dir).await {
        Ok(file_list) => {
            preamble.push_str("\n\n## Available Files\n\n");
            preamble.push_str("The following files are available in the working directory:\n\n");
            preamble.push_str(&file_list);
        }
        Err(e) => {
            eprintln!(
                "{}",
                colors::color_warning(format!(
                    "[!] Warning: Could not gather directory context: {:#}",
                    e
                ))
            );
        }
    }

    Ok(preamble)
}

/// Run the interactive REPL loop for the agent.
async fn run_repl(agent: Agent<anthropic::completion::CompletionModel>) -> Result<()> {
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

        // Execute query with history and progress hook
        match agent
            .prompt(input)
            .with_history(&mut history)
            .with_hook(hook.clone())
            .await
        {
            Ok(response) => {
                horse::markdown::render_markdown(&response);
            }
            Err(e) => {
                eprintln!("{}", colors::color_error(format!(">> Error: {:#}\n", e)));
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install color-eyre without using `?` since it returns ErrReport
    if let Err(e) = color_eyre::install() {
        eprintln!(
            "{}",
            colors::color_warning(format!("Warning: Failed to install color-eyre: {}", e))
        );
    }

    let args = Args::parse();

    // Canonicalize directory to absolute path
    let base_dir = args
        .dir
        .canonicalize()
        .context("Failed to canonicalize target directory")?;

    println!(
        "Horse - {}",
        colors::color_success(
            "An read-only agentic search assistant for intelligent directory exploration"
        )
    );
    println!(
        "Working directory: {}",
        colors::color_status(base_dir.display())
    );
    println!("Model: {}", colors::color_status(&args.model));
    println!("Max turns: {}", colors::color_status(args.max_turns));
    println!();

    // Load preamble from AGENTS.md or use default
    let preamble = load_preamble(&base_dir).await?;

    // Initialize Anthropic client (from_env reads ANTHROPIC_API_KEY automatically)
    let client = anthropic::Client::from_env();

    let model =
        anthropic::completion::CompletionModel::new(client, &args.model).with_prompt_caching();

    // Create agent with tools and preamble

    let agent = AgentBuilder::new(model)
        .preamble(&preamble)
        .default_max_turns(args.max_turns)
        .tool(ReadFile::new(base_dir.clone()))
        .tool(BashCommand::new(base_dir.clone()))
        .tool(SearchDocs::new(base_dir.clone()))
        .build();

    // Run the REPL loop
    run_repl(agent).await
}
