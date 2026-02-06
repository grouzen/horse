#![allow(dead_code, unused_imports)]

mod bash;
mod read_file;
mod search_docs;

pub use bash::{BashCommand, BashCommandArgs};
pub use read_file::{ReadFile, ReadFileArgs};
pub use search_docs::{SearchDocs, SearchDocsArgs};

/// Available tool types
#[derive(Debug, Clone, Copy)]
pub enum Tools {
    Bash,
    ReadFile,
    SearchDocs,
}

impl TryFrom<&str> for Tools {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "bash" => Ok(Tools::Bash),
            "read_file" => Ok(Tools::ReadFile),
            "search_docs" => Ok(Tools::SearchDocs),
            _ => Err(()),
        }
    }
}

impl Tools {
    /// Extract the display-friendly argument from the JSON args string
    pub fn extract_display_args(&self, args: &str) -> String {
        match self {
            Tools::Bash => serde_json::from_str::<BashCommandArgs>(args)
                .map(|parsed| parsed.command)
                .unwrap_or_else(|_| args.to_string()),
            Tools::ReadFile => serde_json::from_str::<ReadFileArgs>(args)
                .map(|parsed| parsed.path)
                .unwrap_or_else(|_| args.to_string()),
            Tools::SearchDocs => serde_json::from_str::<SearchDocsArgs>(args)
                .map(|parsed| {
                    let path = parsed.path.as_deref().unwrap_or(".");
                    format!("{} in {}", parsed.query, path)
                })
                .unwrap_or_else(|_| args.to_string()),
        }
    }
}
