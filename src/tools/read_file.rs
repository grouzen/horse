use std::path::PathBuf;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

const MAX_BYTES: usize = 50 * 1024; // 50KB
const MAX_LINES: usize = 1000;

#[derive(Deserialize)]
pub struct ReadFileArgs {
    /// The path to the file to read, relative to the base directory
    path: String,
    /// Optional starting line number (1-indexed)
    start_line: Option<usize>,
    /// Optional ending line number (1-indexed, inclusive)
    end_line: Option<usize>,
}

#[derive(Debug, Error)]
pub enum ReadFileError {
    #[error("Path traversal not allowed: {0}")]
    PathTraversal(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path is outside base directory")]
    OutsideBaseDir,
}

#[derive(Deserialize, Serialize)]
pub struct ReadFile {
    #[serde(skip)]
    base_dir: PathBuf,
}

impl ReadFile {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf, ReadFileError> {
        // Reject paths containing ".."
        if path.contains("..") {
            Err(ReadFileError::PathTraversal(path.to_string()))
        } else {
            let resolved = self.base_dir.join(path);

            // Canonicalize and verify it's within base_dir
            let canonical = resolved.canonicalize()?;
            let base_canonical = self.base_dir.canonicalize()?;

            if canonical.starts_with(&base_canonical) {
                Ok(canonical)
            } else {
                Err(ReadFileError::OutsideBaseDir)
            }
        }
    }
}

impl Tool for ReadFile {
    const NAME: &'static str = "read_file";

    type Error = ReadFileError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Read the contents of a file. Paths are relative to the working directory. \
                Use start_line and end_line to read specific portions of large files."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read, relative to the working directory"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Optional starting line number (1-indexed)"
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "Optional ending line number (1-indexed, inclusive)"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = self.resolve_path(&args.path)?;
        let content = tokio::fs::read_to_string(&path).await?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Apply line range filter if specified
        let start = args.start_line.map(|s| s.saturating_sub(1)).unwrap_or(0);
        let end = args.end_line.unwrap_or(total_lines).min(total_lines);

        let selected_lines: Vec<&str> = lines
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect();

        // Check truncation limits
        let mut result = String::new();
        let mut byte_count = 0;
        let mut truncated = false;

        for (line_count, line) in selected_lines.into_iter().enumerate() {
            if line_count >= MAX_LINES || byte_count + line.len() + 1 > MAX_BYTES {
                truncated = true;
                break;
            }
            if !result.is_empty() {
                result.push('\n');
                byte_count += 1;
            }
            result.push_str(line);
            byte_count += line.len();
        }

        if truncated {
            result.push_str("\n\n[truncated - file exceeds 50KB or 1000 lines limit]");
        }

        Ok(result)
    }
}
