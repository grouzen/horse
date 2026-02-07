#[allow(dead_code)]
use anyhow::Result;
use rig::agent::{Agent, AgentBuilder};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::anthropic;
use std::path::Path;

use crate::agent::hooks::ProgressHook;
use crate::agent::tools::{BashCommand, ReadFile, SearchDocs};
use crate::config::Config;
use crate::providers::{AgentWrapper, Provider};

pub struct AnthropicProvider;

impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["ANTHROPIC_API_KEY"]
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-0"
    }

    fn create_agent(
        &self,
        config: &Config,
        base_dir: &Path,
        preamble: &str,
    ) -> Result<Box<dyn AgentWrapper>> {
        let client = anthropic::Client::from_env();
        let mut model = client.completion_model(&config.model);

        // Apply Anthropic-specific features
        if let Some(features) = &config.anthropic
            && features.prompt_caching
        {
            model = model.with_prompt_caching();
        }

        let agent = AgentBuilder::new(model)
            .preamble(preamble)
            .tool(BashCommand::new(base_dir.to_path_buf()))
            .tool(ReadFile::new(base_dir.to_path_buf()))
            .tool(SearchDocs::new(base_dir.to_path_buf()))
            .build();

        Ok(Box::new(AnthropicAgentWrapper { agent }))
    }
}

#[allow(dead_code)]
struct AnthropicAgentWrapper {
    agent: Agent<anthropic::completion::CompletionModel>,
}

#[async_trait::async_trait]
impl AgentWrapper for AnthropicAgentWrapper {
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
