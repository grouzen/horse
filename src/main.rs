use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

mod agent;
mod config;
mod console;
mod providers;

use crate::console::{colors, repl::Repl};

#[derive(Parser, Debug)]
#[command(name = "horse")]
#[command(about = "An agentic search assistant for intelligent directory exploration")]
struct Args {
    /// Target directory to search and execute commands in
    #[arg(default_value = ".")]
    dir: PathBuf,

    /// LLM provider to use (overrides config file)
    #[arg(short, long)]
    provider: Option<String>,

    /// Model to use (overrides config file)
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Maximum number of turns for the agent (overrides config file)
    #[arg(short = 't', long)]
    max_turns: Option<usize>,
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

    // Load config and merge with CLI arguments
    let mut config = config::Config::load()?;
    config.merge_with_args(args.provider, args.model, args.max_turns);

    // Canonicalize directory to absolute path
    let base_dir = args
        .dir
        .canonicalize()
        .context("Failed to canonicalize target directory")?;

    // Render and display the HORSE logo banner
    let logo = console::logo::render_logo("HORSE")?;
    println!("{}\n", logo);

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
    println!("Provider: {}", colors::color_status(&config.provider));
    println!("Model: {}", colors::color_status(&config.model));
    println!("Max turns: {}", colors::color_status(config.max_turns));
    println!();

    // Load preamble from AGENTS.md or use default
    let preamble = load_preamble(&base_dir).await?;

    // Get provider instance
    let provider = providers::get_provider(&config.provider)?;

    // Validate required environment variables
    providers::validate_env_vars(provider.as_ref())?;

    // Create agent using the provider
    let agent = provider.create_agent(&config, &base_dir, &preamble)?;

    let mut repl = Repl::new(agent);

    // Run the REPL loop
    repl.run().await
}
