use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

const TIMEOUT_SECS: u64 = 30;
const MAX_COUNT: usize = 100;
const CONTEXT_LINES: usize = 2;

#[derive(Deserialize)]
pub struct SearchDocsArgs {
    /// The search query/pattern to find in documents
    pub query: String,
    /// Optional path or glob pattern to search in (defaults to current directory)
    pub path: Option<String>,
}

#[derive(Debug, Error)]
pub enum SearchDocsError {
    #[error(
        "rga command not found. Please install ripgrep-all: https://github.com/phiresky/ripgrep-all"
    )]
    RgaNotInstalled,
    #[error("Search query is empty")]
    EmptyQuery,
    #[error("Search timed out after {0} seconds")]
    Timeout(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Search failed with exit code {0}: {1}")]
    SearchFailed(i32, String),
}

#[derive(Deserialize, Serialize)]
pub struct SearchDocs {
    #[serde(skip)]
    base_dir: PathBuf,
}

impl SearchDocs {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl Tool for SearchDocs {
    const NAME: &'static str = "search_docs";

    type Error = SearchDocsError;
    type Args = SearchDocsArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Search through documents (PDFs, Word docs, Excel, etc.) using ripgrep-all. \
                Automatically handles binary formats and extracts text. \
                Use this when you need to find content in non-text files. \
                Do not use it until other tools have been tried."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query/pattern to find in documents"
                    },
                    "path": {
                        "type": "string",
                        "description": "Optional path or glob pattern to search in (defaults to current directory)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate query is not empty
        if args.query.trim().is_empty() {
            return Err(SearchDocsError::EmptyQuery);
        }

        // Build rga command with flags
        let path = args.path.as_deref().unwrap_or(".");

        let mut cmd = Command::new("rga");
        cmd.arg("-i") // case-insensitive
            .arg("--max-count")
            .arg(MAX_COUNT.to_string())
            .arg("--context")
            .arg(CONTEXT_LINES.to_string())
            .arg("--color")
            .arg("never")
            .arg(&args.query)
            .arg(path)
            .current_dir(&self.base_dir);

        // Execute with timeout
        let result = timeout(Duration::from_secs(TIMEOUT_SECS), cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                match output.status.code() {
                    Some(0) => {
                        // Success - return stdout
                        Ok(String::from_utf8_lossy(&output.stdout).to_string())
                    }
                    Some(1) => {
                        // No matches found (rga returns 1 when no matches)
                        Ok("No matches found".to_string())
                    }
                    Some(127) => {
                        // Command not found
                        Err(SearchDocsError::RgaNotInstalled)
                    }
                    Some(code) => {
                        // Other error
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Err(SearchDocsError::SearchFailed(code, stderr))
                    }
                    None => {
                        // Process was terminated by signal
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Err(SearchDocsError::SearchFailed(-1, stderr))
                    }
                }
            }
            Ok(Err(e)) => {
                // Check if it's a "not found" error (ENOENT)
                if e.kind() == std::io::ErrorKind::NotFound {
                    Err(SearchDocsError::RgaNotInstalled)
                } else {
                    Err(SearchDocsError::Io(e))
                }
            }
            Err(_) => {
                // Timeout
                Err(SearchDocsError::Timeout(TIMEOUT_SECS))
            }
        }
    }
}
