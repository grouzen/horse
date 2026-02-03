use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

const TIMEOUT_SECS: u64 = 30;

const ALLOWED_COMMANDS: &[&str] = &[
    "grep", "find", "cat", "head", "tail", "ls", "tree", "wc", "file", "rg",
];

// Allow pipes but block more dangerous patterns
const FORBIDDEN_PATTERNS: &[&str] = &[";", "&&", "||", "`", "$(", ">", "<", ">>", "<<"];

#[derive(Deserialize)]
pub struct BashCommandArgs {
    /// The command to execute
    command: String,
}

#[derive(Debug, Error)]
pub enum BashCommandError {
    #[error("Command not in whitelist: {0}. Allowed commands: {1}")]
    CommandNotAllowed(String, String),
    #[error("Forbidden pattern in command: {0}")]
    ForbiddenPattern(String),
    #[error("Command timed out after {0} seconds")]
    Timeout(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed with exit code {0}: {1}")]
    CommandFailed(i32, String),
    #[error("Empty command")]
    EmptyCommand,
}

#[derive(Deserialize, Serialize)]
pub struct BashCommand {
    #[serde(skip)]
    base_dir: PathBuf,
}

impl BashCommand {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn validate_command(&self, command: &str) -> Result<(), BashCommandError> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(BashCommandError::EmptyCommand);
        }

        // Check for forbidden patterns
        for pattern in FORBIDDEN_PATTERNS {
            if trimmed.contains(pattern) {
                return Err(BashCommandError::ForbiddenPattern(pattern.to_string()));
            }
        }

        // Split by pipe while respecting quotes
        let commands = self.split_respecting_quotes(trimmed, '|');

        for cmd in commands {
            let cmd = cmd.trim();
            if cmd.is_empty() {
                continue;
            }

            // Extract the first word (command name)
            let first_word = cmd
                .split_whitespace()
                .next()
                .ok_or(BashCommandError::EmptyCommand)?;

            // Check if command is in whitelist
            if !ALLOWED_COMMANDS.contains(&first_word) {
                return Err(BashCommandError::CommandNotAllowed(
                    first_word.to_string(),
                    ALLOWED_COMMANDS.join(", "),
                ));
            }
        }

        Ok(())
    }

    /// Split a string by a delimiter while respecting quoted sections
    fn split_respecting_quotes<'a>(&self, s: &'a str, delimiter: char) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut prev_char = '\0';

        for (i, c) in s.char_indices() {
            // Track quote state
            if c == '\'' && prev_char != '\\' && !in_double_quote {
                in_single_quote = !in_single_quote;
            } else if c == '"' && prev_char != '\\' && !in_single_quote {
                in_double_quote = !in_double_quote;
            }

            // Split on delimiter only if not inside quotes
            if c == delimiter && !in_single_quote && !in_double_quote {
                result.push(&s[start..i]);
                start = i + 1;
            }

            prev_char = c;
        }

        // Add the remaining part
        if start < s.len() {
            result.push(&s[start..]);
        } else if start == s.len() {
            result.push("");
        }

        result
    }
}

impl Tool for BashCommand {
    const NAME: &'static str = "bash";

    type Error = BashCommandError;
    type Args = BashCommandArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: format!(
                "Execute a read-only bash command. Only the following commands are allowed: {}. \
                Pipes (|) are allowed for chaining these commands. Redirects and command chaining with ;, &&, || are not allowed.",
                ALLOWED_COMMANDS.join(", ")
            ),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.validate_command(&args.command)?;

        // If command contains pipe, use shell; otherwise execute directly
        let mut child = if args.command.contains('|') {
            Command::new("sh")
                .arg("-c")
                .arg(&args.command)
                .current_dir(&self.base_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        } else {
            // Parse command into parts for direct execution
            let parts: Vec<&str> = args.command.split_whitespace().collect();
            let (cmd, cmd_args) = parts.split_first().ok_or(BashCommandError::EmptyCommand)?;

            Command::new(cmd)
                .args(cmd_args)
                .current_dir(&self.base_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        let result = timeout(Duration::from_secs(TIMEOUT_SECS), async {
            let mut stdout = String::new();
            let mut stderr = String::new();

            if let Some(ref mut stdout_pipe) = child.stdout {
                stdout_pipe.read_to_string(&mut stdout).await?;
            }
            if let Some(ref mut stderr_pipe) = child.stderr {
                stderr_pipe.read_to_string(&mut stderr).await?;
            }

            let status = child.wait().await?;

            Ok::<_, std::io::Error>((status, stdout, stderr))
        })
        .await;

        match result {
            Ok(Ok((status, stdout, stderr))) => {
                if status.success() {
                    let mut output = stdout;
                    if !stderr.is_empty() {
                        if !output.is_empty() {
                            output.push_str("\n--- stderr ---\n");
                        }
                        output.push_str(&stderr);
                    }
                    Ok(output)
                } else {
                    let exit_code = status.code().unwrap_or(-1);
                    let error_output = if stderr.is_empty() { stdout } else { stderr };
                    Err(BashCommandError::CommandFailed(exit_code, error_output))
                }
            }
            Ok(Err(e)) => Err(BashCommandError::Io(e)),
            Err(_) => {
                // Timeout - kill the process
                let _ = child.kill().await;
                Err(BashCommandError::Timeout(TIMEOUT_SECS))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_respecting_quotes() {
        let bash = BashCommand::new(PathBuf::from("."));

        // Test simple pipe without quotes
        let result = bash.split_respecting_quotes("ls | grep test", '|');
        assert_eq!(result, vec!["ls ", " grep test"]);

        // Test pipe inside double quotes (should not split)
        let result = bash.split_respecting_quotes(r#"grep -E "README|readme" | cat"#, '|');
        assert_eq!(result, vec![r#"grep -E "README|readme" "#, " cat"]);

        // Test pipe inside single quotes (should not split)
        let result = bash.split_respecting_quotes(r#"grep -E 'README|readme' | cat"#, '|');
        assert_eq!(result, vec![r#"grep -E 'README|readme' "#, " cat"]);

        // Test multiple pipes in quotes
        let result = bash.split_respecting_quotes(
            r#"find . -type f | grep -E "README|readme|project|overview" | head"#,
            '|',
        );
        assert_eq!(
            result,
            vec![
                "find . -type f ",
                r#" grep -E "README|readme|project|overview" "#,
                " head"
            ]
        );

        // Test no pipes
        let result = bash.split_respecting_quotes("ls -la", '|');
        assert_eq!(result, vec!["ls -la"]);
    }

    #[test]
    fn test_validate_command_with_quoted_pipes() {
        let bash = BashCommand::new(PathBuf::from("."));

        // This should pass - pipes inside quotes should not cause validation issues
        let result = bash.validate_command(
            r#"find . -type f | grep -E "README|readme|project|overview|description" | grep -i "ollana""#,
        );
        assert!(result.is_ok(), "Expected Ok, got: {result:?}");

        // This should also pass
        let result = bash.validate_command(r#"grep -E "foo|bar|baz" file.txt"#);
        assert!(result.is_ok(), "Expected Ok, got: {result:?}");

        // This should fail - "notallowed" is not in whitelist
        let result = bash.validate_command("notallowed arg1 arg2");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BashCommandError::CommandNotAllowed(_, _))
        ));
    }

    #[test]
    fn test_validate_allowed_commands() {
        let bash = BashCommand::new(PathBuf::from("."));

        // Test all allowed commands
        for cmd in ALLOWED_COMMANDS {
            let result = bash.validate_command(&format!("{cmd} arg1"));
            assert!(result.is_ok(), "Command {cmd} should be allowed");
        }

        // Test piped allowed commands
        let result = bash.validate_command("find . -name test | grep foo");
        assert!(result.is_ok());

        let result = bash.validate_command("cat file.txt | head -n 10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_forbidden_patterns() {
        let bash = BashCommand::new(PathBuf::from("."));

        // Test forbidden patterns
        let result = bash.validate_command("ls; rm -rf /");
        assert!(matches!(result, Err(BashCommandError::ForbiddenPattern(_))));

        let result = bash.validate_command("ls && echo test");
        assert!(matches!(result, Err(BashCommandError::ForbiddenPattern(_))));

        let result = bash.validate_command("ls > output.txt");
        assert!(matches!(result, Err(BashCommandError::ForbiddenPattern(_))));
    }
}
