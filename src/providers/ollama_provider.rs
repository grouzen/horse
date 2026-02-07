use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::ollama;
use std::path::Path;

use crate::agent::hooks::ProgressHook;
use crate::agent::tools::{BashCommand, ReadFile, SearchDocs};
use crate::config::Config;
use crate::providers::{AgentWrapper, Provider};

pub struct OllamaProvider;

impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec![] // Ollama is local, no API key required
    }

    fn default_model(&self) -> &str {
        "llama3"
    }

    fn create_agent(
        &self,
        config: &Config,
        base_dir: &Path,
        preamble: &str,
    ) -> Result<Box<dyn AgentWrapper>> {
        let client = ollama::Client::from_env();
        let model = client.completion_model(&config.model);

        // Ollama-specific features could be added here if needed
        // when the API supports them

        let agent = AgentBuilder::new(model)
            .preamble(preamble)
            .default_max_turns(config.max_turns)
            .tool(BashCommand::new(base_dir.to_path_buf()))
            .tool(ReadFile::new(base_dir.to_path_buf()))
            .tool(SearchDocs::new(base_dir.to_path_buf()))
            .build();

        Ok(Box::new(OllamaAgentWrapper { agent }))
    }
}

struct OllamaAgentWrapper {
    agent: Agent<ollama::CompletionModel>,
}

#[async_trait::async_trait]
impl AgentWrapper for OllamaAgentWrapper {
    async fn prompt(
        &mut self,
        input: String,
        history: &mut Vec<rig::completion::Message>,
        hook: ProgressHook,
    ) -> Result<String> {
        self.agent
            .prompt(input)
            .with_hook(hook)
            .with_history(history)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}
