use anyhow::Result;
use std::path::Path;

use crate::{agent::hooks::ProgressHook, config::Config, console::colors};

mod anthropic_provider;
mod azure_provider;
mod cohere_provider;
mod deepseek_provider;
mod galadriel_provider;
mod gemini_provider;
mod groq_provider;
mod huggingface_provider;
mod hyperbolic_provider;
mod mira_provider;
mod mistral_provider;
mod moonshot_provider;
mod ollama_provider;
mod openai_provider;
mod openrouter_provider;
mod perplexity_provider;
mod together_provider;
mod xai_provider;

pub use anthropic_provider::AnthropicProvider;
pub use azure_provider::AzureProvider;
pub use cohere_provider::CohereProvider;
pub use deepseek_provider::DeepSeekProvider;
pub use galadriel_provider::GaladrielProvider;
pub use gemini_provider::GeminiProvider;
pub use groq_provider::GroqProvider;
pub use huggingface_provider::HuggingFaceProvider;
pub use hyperbolic_provider::HyperbolicProvider;
pub use mira_provider::MiraProvider;
pub use mistral_provider::MistralProvider;
pub use moonshot_provider::MoonshotProvider;
pub use ollama_provider::OllamaProvider;
pub use openai_provider::OpenAIProvider;
pub use openrouter_provider::OpenRouterProvider;
pub use perplexity_provider::PerplexityProvider;
pub use together_provider::TogetherProvider;
pub use xai_provider::XAIProvider;

/// Provider trait defines the contract for all LLM providers
pub trait Provider {
    /// Get the name of this provider (e.g., "anthropic", "openai")
    fn name(&self) -> &str;

    /// Get required environment variables for this provider
    fn required_env_vars(&self) -> Vec<&str>;

    /// Get the default model for this provider
    #[allow(dead_code)]
    fn default_model(&self) -> &str;

    /// Create an agent for this provider with given config and context
    /// Returns an error if required environment variables are missing
    fn create_agent(
        &self,
        config: &Config,
        base_dir: &Path,
        preamble: &str,
    ) -> Result<Box<dyn AgentWrapper>>;
}

/// Wrapper trait to handle different agent types
#[async_trait::async_trait]
pub trait AgentWrapper: Send + Sync {
    /// Run the agent with the given prompt
    async fn prompt(
        &mut self,
        input: String,
        history: &mut Vec<rig::completion::Message>,
        hook: ProgressHook,
    ) -> Result<String>;
}

/// Get a provider instance by name
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>> {
    match name.to_lowercase().as_str() {
        "anthropic" => Ok(Box::new(AnthropicProvider)),
        "azure" => Ok(Box::new(AzureProvider)),
        "cohere" => Ok(Box::new(CohereProvider)),
        "deepseek" => Ok(Box::new(DeepSeekProvider)),
        "galadriel" => Ok(Box::new(GaladrielProvider)),
        "gemini" => Ok(Box::new(GeminiProvider)),
        "groq" => Ok(Box::new(GroqProvider)),
        "huggingface" => Ok(Box::new(HuggingFaceProvider)),
        "hyperbolic" => Ok(Box::new(HyperbolicProvider)),
        "mira" => Ok(Box::new(MiraProvider)),
        "mistral" => Ok(Box::new(MistralProvider)),
        "moonshot" => Ok(Box::new(MoonshotProvider)),
        "ollama" => Ok(Box::new(OllamaProvider)),
        "openai" => Ok(Box::new(OpenAIProvider)),
        "openrouter" => Ok(Box::new(OpenRouterProvider)),
        "perplexity" => Ok(Box::new(PerplexityProvider)),
        "together" => Ok(Box::new(TogetherProvider)),
        "xai" => Ok(Box::new(XAIProvider)),
        _ => anyhow::bail!(
            "Unknown provider: {}. Supported providers: anthropic, azure, cohere, deepseek, galadriel, gemini, groq, huggingface, hyperbolic, mira, mistral, moonshot, ollama, openai, openrouter, perplexity, together, xai",
            name
        ),
    }
}

/// Validate that all required environment variables are set for a provider
pub fn validate_env_vars(provider: &dyn Provider) -> Result<()> {
    let missing_vars: Vec<&str> = provider
        .required_env_vars()
        .into_iter()
        .filter(|var| std::env::var(var).is_err())
        .collect();

    if !missing_vars.is_empty() {
        anyhow::bail!(colors::color_error(format!(
            "Missing required environment variables for provider '{}': {}",
            provider.name(),
            missing_vars.join(", ")
        )));
    }

    Ok(())
}
