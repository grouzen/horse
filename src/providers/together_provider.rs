use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::together;
use std::path::Path;

use crate::agent::hooks::ProgressHook;
use crate::agent::tools::{BashCommand, ReadFile, SearchDocs};
use crate::config::Config;
use crate::providers::{AgentWrapper, Provider};

pub struct TogetherProvider;

impl Provider for TogetherProvider {
    fn name(&self) -> &str {
        "together"
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["TOGETHER_API_KEY"]
    }

    fn default_model(&self) -> &str {
        "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo"
    }

    fn create_agent(
        &self,
        config: &Config,
        base_dir: &Path,
        preamble: &str,
    ) -> Result<Box<dyn AgentWrapper>> {
        let client = together::Client::from_env();
        let model = client.completion_model(&config.model);

        let agent = AgentBuilder::new(model)
            .preamble(preamble)
            .default_max_turns(config.max_turns)
            .tool(BashCommand::new(base_dir.to_path_buf()))
            .tool(ReadFile::new(base_dir.to_path_buf()))
            .tool(SearchDocs::new(base_dir.to_path_buf()))
            .build();

        Ok(Box::new(TogetherAgentWrapper { agent }))
    }
}

struct TogetherAgentWrapper {
    agent: Agent<together::CompletionModel>,
}

#[async_trait::async_trait]
impl AgentWrapper for TogetherAgentWrapper {
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
