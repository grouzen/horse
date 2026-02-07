use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::perplexity;
use std::path::Path;

use crate::agent::hooks::ProgressHook;
use crate::agent::tools::{BashCommand, ReadFile, SearchDocs};
use crate::config::Config;
use crate::providers::{AgentWrapper, Provider};

pub struct PerplexityProvider;

impl Provider for PerplexityProvider {
    fn name(&self) -> &str {
        "perplexity"
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["PERPLEXITY_API_KEY"]
    }

    fn default_model(&self) -> &str {
        "llama-3.1-sonar-large-128k-online"
    }

    fn create_agent(
        &self,
        config: &Config,
        base_dir: &Path,
        preamble: &str,
    ) -> Result<Box<dyn AgentWrapper>> {
        let client = perplexity::Client::from_env();
        let model = client.completion_model(&config.model);

        let agent = AgentBuilder::new(model)
            .preamble(preamble)
            .default_max_turns(config.max_turns)
            .tool(BashCommand::new(base_dir.to_path_buf()))
            .tool(ReadFile::new(base_dir.to_path_buf()))
            .tool(SearchDocs::new(base_dir.to_path_buf()))
            .build();

        Ok(Box::new(PerplexityAgentWrapper { agent }))
    }
}

struct PerplexityAgentWrapper {
    agent: Agent<perplexity::CompletionModel>,
}

#[async_trait::async_trait]
impl AgentWrapper for PerplexityAgentWrapper {
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
