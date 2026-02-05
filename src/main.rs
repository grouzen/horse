use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use rig::agent::{Agent, AgentBuilder};
use rig::client::ProviderClient;
use rig::completion::Prompt;
use rig::providers::anthropic;

mod hooks;
mod tools;

use hooks::ProgressHook;
use tools::{BashCommand, ReadFile};

#[derive(Parser, Debug)]
#[command(name = "horse")]
#[command(about = "An agentic RAG for intelligent directory exploration")]
struct Args {
    /// Target directory to search and execute commands in
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,

    /// Claude model to use
    #[arg(short, long, default_value = "claude-sonnet-4-0")]
    model: String,

    /// Maximum number of turns for the agent
    #[arg(short = 't', long, default_value = "20")]
    max_turns: usize,
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
        println!(">> Loading AGENTS.md...");
        tokio::fs::read_to_string(&agents_file)
            .await
            .context("Failed to read AGENTS.md")?
    } else {
        "You are a helpful search assistant. You can read files and execute safe bash commands \
            to help users explore and understand their codebase."
            .to_string()
    };

    // Add directory context
    println!(">> Gathering directory structure...");
    match gather_directory_context(base_dir).await {
        Ok(file_list) => {
            preamble.push_str("\n\n## Available Files\n\n");
            preamble.push_str("The following files are available in the working directory:\n\n");
            preamble.push_str(&file_list);
        }
        Err(e) => {
            eprintln!("[!] Warning: Could not gather directory context: {:#}", e);
        }
    }

    Ok(preamble)
}

/// Run the interactive REPL loop for the agent.
async fn run_repl(agent: Agent<anthropic::completion::CompletionModel>) -> Result<()> {
    println!(">> Ready! Type your queries (Ctrl+C or Ctrl+D to exit)");
    println!();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();
    let mut history = Vec::new();
    let hook = ProgressHook::new();

    loop {
        // Prompt with token usage
        print!("{}", hook.format_prompt());
        io::stdout().flush()?;

        // Read line
        buffer.clear();
        let bytes_read = handle
            .read_line(&mut buffer)
            .context("Failed to read line from stdin")?;

        // Check for EOF (Ctrl+D)
        if bytes_read == 0 {
            println!("\n>> Goodbye!");
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
                println!("\n{}\n", response);
            }
            Err(e) => {
                eprintln!(">> Error: {:#}\n", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install color-eyre without using `?` since it returns ErrReport
    if let Err(e) = color_eyre::install() {
        eprintln!("Warning: Failed to install color-eyre: {}", e);
    }

    let args = Args::parse();

    // Canonicalize directory to absolute path
    let base_dir = args
        .dir
        .canonicalize()
        .context("Failed to canonicalize target directory")?;

    println!("Horse - Agentic Search REPL");
    println!("Working directory: {}", base_dir.display());
    println!("Model: {}", args.model);
    println!("Max turns: {}", args.max_turns);
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
        .build();

    // Run the REPL loop
    run_repl(agent).await
}
