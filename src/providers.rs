use anyhow::Result;
#[allow(unused_imports)]
use rig::agent::{Agent, AgentBuilder};
#[allow(unused_imports)]
use rig::client::{CompletionClient, ProviderClient};
#[allow(unused_imports)]
use rig::completion::Prompt;
use std::path::Path;

use crate::agent::hooks::ProgressHook;
#[allow(unused_imports)]
use crate::agent::tools::{BashCommand, ReadFile, SearchDocs};
use crate::config::Config;
use crate::console::colors;

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

// ---------------------------------------------------------------------------
// Macro for defining standard (no-custom-logic) providers
// ---------------------------------------------------------------------------

/// Defines a provider struct, its `Provider` impl, a private agent wrapper, and its
/// `AgentWrapper` impl – eliminating ~75 lines of boilerplate per provider.
#[allow(unused_macros)]
macro_rules! define_standard_provider {
    (
        provider_struct: $provider_struct:ident,
        wrapper_struct:  $wrapper_struct:ident,
        name:            $name:expr,
        env_vars:        [$($env:expr),*],
        default_model:   $default_model:expr,
        client_type:     $client_type:ty,
        model_type:      $model_type:ty
    ) => {
        pub struct $provider_struct;

        impl Provider for $provider_struct {
            fn name(&self) -> &str {
                $name
            }

            fn required_env_vars(&self) -> Vec<&str> {
                vec![$($env),*]
            }

            fn default_model(&self) -> &str {
                $default_model
            }

            fn create_agent(
                &self,
                config: &Config,
                base_dir: &Path,
                preamble: &str,
            ) -> Result<Box<dyn AgentWrapper>> {
                let client = <$client_type>::from_env();
                let model = client.completion_model(&config.model);

                let agent = AgentBuilder::new(model)
                    .preamble(preamble)
                    .default_max_turns(config.max_turns)
                    .tool(BashCommand::new(base_dir.to_path_buf()))
                    .tool(ReadFile::new(base_dir.to_path_buf()))
                    .tool(SearchDocs::new(base_dir.to_path_buf()))
                    .build();

                Ok(Box::new($wrapper_struct { agent }))
            }
        }

        struct $wrapper_struct {
            agent: Agent<$model_type>,
        }

        #[async_trait::async_trait]
        impl AgentWrapper for $wrapper_struct {
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
    };
}

// ---------------------------------------------------------------------------
// Anthropic – kept as a separate module (has prompt-caching logic)
// ---------------------------------------------------------------------------

#[cfg(feature = "provider-anthropic")]
mod anthropic_provider;
#[cfg(feature = "provider-anthropic")]
pub use anthropic_provider::AnthropicProvider;

// ---------------------------------------------------------------------------
// Standard providers – generated via macro, gated behind feature flags
// ---------------------------------------------------------------------------

#[cfg(feature = "provider-azure")]
define_standard_provider! {
    provider_struct: AzureProvider,
    wrapper_struct:  AzureAgentWrapper,
    name:            "azure",
    env_vars:        ["AZURE_API_KEY", "AZURE_API_VERSION", "AZURE_ENDPOINT"],
    default_model:   "gpt-4",
    client_type:     rig::providers::azure::Client,
    model_type:      rig::providers::azure::CompletionModel
}

#[cfg(feature = "provider-cohere")]
define_standard_provider! {
    provider_struct: CohereProvider,
    wrapper_struct:  CohereAgentWrapper,
    name:            "cohere",
    env_vars:        ["COHERE_API_KEY"],
    default_model:   "command-r-plus",
    client_type:     rig::providers::cohere::Client,
    model_type:      rig::providers::cohere::CompletionModel
}

#[cfg(feature = "provider-deepseek")]
define_standard_provider! {
    provider_struct: DeepSeekProvider,
    wrapper_struct:  DeepSeekAgentWrapper,
    name:            "deepseek",
    env_vars:        ["DEEPSEEK_API_KEY"],
    default_model:   "deepseek-chat",
    client_type:     rig::providers::deepseek::Client,
    model_type:      rig::providers::deepseek::CompletionModel
}

#[cfg(feature = "provider-galadriel")]
define_standard_provider! {
    provider_struct: GaladrielProvider,
    wrapper_struct:  GaladrielAgentWrapper,
    name:            "galadriel",
    env_vars:        ["GALADRIEL_API_KEY"],
    default_model:   "llama3.1:70b",
    client_type:     rig::providers::galadriel::Client,
    model_type:      rig::providers::galadriel::CompletionModel
}

#[cfg(feature = "provider-gemini")]
define_standard_provider! {
    provider_struct: GeminiProvider,
    wrapper_struct:  GeminiAgentWrapper,
    name:            "gemini",
    env_vars:        ["GEMINI_API_KEY"],
    default_model:   "gemini-1.5-pro",
    client_type:     rig::providers::gemini::Client,
    model_type:      rig::providers::gemini::CompletionModel
}

#[cfg(feature = "provider-groq")]
define_standard_provider! {
    provider_struct: GroqProvider,
    wrapper_struct:  GroqAgentWrapper,
    name:            "groq",
    env_vars:        ["GROQ_API_KEY"],
    default_model:   "llama-3.3-70b-versatile",
    client_type:     rig::providers::groq::Client,
    model_type:      rig::providers::groq::CompletionModel
}

#[cfg(feature = "provider-huggingface")]
define_standard_provider! {
    provider_struct: HuggingFaceProvider,
    wrapper_struct:  HuggingFaceAgentWrapper,
    name:            "huggingface",
    env_vars:        ["HUGGINGFACE_API_KEY"],
    default_model:   "meta-llama/Meta-Llama-3-8B-Instruct",
    client_type:     rig::providers::huggingface::Client,
    model_type:      rig::providers::huggingface::completion::CompletionModel
}

#[cfg(feature = "provider-hyperbolic")]
define_standard_provider! {
    provider_struct: HyperbolicProvider,
    wrapper_struct:  HyperbolicAgentWrapper,
    name:            "hyperbolic",
    env_vars:        ["HYPERBOLIC_API_KEY"],
    default_model:   "meta-llama/Meta-Llama-3-70B-Instruct",
    client_type:     rig::providers::hyperbolic::Client,
    model_type:      rig::providers::hyperbolic::CompletionModel
}

#[cfg(feature = "provider-mira")]
define_standard_provider! {
    provider_struct: MiraProvider,
    wrapper_struct:  MiraAgentWrapper,
    name:            "mira",
    env_vars:        ["MIRA_API_KEY"],
    default_model:   "gpt-4",
    client_type:     rig::providers::mira::Client,
    model_type:      rig::providers::mira::CompletionModel
}

#[cfg(feature = "provider-mistral")]
define_standard_provider! {
    provider_struct: MistralProvider,
    wrapper_struct:  MistralAgentWrapper,
    name:            "mistral",
    env_vars:        ["MISTRAL_API_KEY"],
    default_model:   "mistral-large-latest",
    client_type:     rig::providers::mistral::Client,
    model_type:      rig::providers::mistral::CompletionModel
}

#[cfg(feature = "provider-moonshot")]
define_standard_provider! {
    provider_struct: MoonshotProvider,
    wrapper_struct:  MoonshotAgentWrapper,
    name:            "moonshot",
    env_vars:        ["MOONSHOT_API_KEY"],
    default_model:   "moonshot-v1-8k",
    client_type:     rig::providers::moonshot::Client,
    model_type:      rig::providers::moonshot::CompletionModel
}

#[cfg(feature = "provider-ollama")]
define_standard_provider! {
    provider_struct: OllamaProvider,
    wrapper_struct:  OllamaAgentWrapper,
    name:            "ollama",
    env_vars:        [],
    default_model:   "llama3",
    client_type:     rig::providers::ollama::Client,
    model_type:      rig::providers::ollama::CompletionModel
}

#[cfg(feature = "provider-openai")]
define_standard_provider! {
    provider_struct: OpenAIProvider,
    wrapper_struct:  OpenAIAgentWrapper,
    name:            "openai",
    env_vars:        ["OPENAI_API_KEY"],
    default_model:   "gpt-4o-mini",
    client_type:     rig::providers::openai::CompletionsClient,
    model_type:      rig::providers::openai::CompletionModel
}

#[cfg(feature = "provider-openrouter")]
define_standard_provider! {
    provider_struct: OpenRouterProvider,
    wrapper_struct:  OpenRouterAgentWrapper,
    name:            "openrouter",
    env_vars:        ["OPENROUTER_API_KEY"],
    default_model:   "anthropic/claude-3.5-sonnet",
    client_type:     rig::providers::openrouter::Client,
    model_type:      rig::providers::openrouter::CompletionModel
}

#[cfg(feature = "provider-perplexity")]
define_standard_provider! {
    provider_struct: PerplexityProvider,
    wrapper_struct:  PerplexityAgentWrapper,
    name:            "perplexity",
    env_vars:        ["PERPLEXITY_API_KEY"],
    default_model:   "llama-3.1-sonar-large-128k-online",
    client_type:     rig::providers::perplexity::Client,
    model_type:      rig::providers::perplexity::CompletionModel
}

#[cfg(feature = "provider-together")]
define_standard_provider! {
    provider_struct: TogetherProvider,
    wrapper_struct:  TogetherAgentWrapper,
    name:            "together",
    env_vars:        ["TOGETHER_API_KEY"],
    default_model:   "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
    client_type:     rig::providers::together::Client,
    model_type:      rig::providers::together::CompletionModel
}

#[cfg(feature = "provider-xai")]
define_standard_provider! {
    provider_struct: XAIProvider,
    wrapper_struct:  XAIAgentWrapper,
    name:            "xai",
    env_vars:        ["XAI_API_KEY"],
    default_model:   "grok-beta",
    client_type:     rig::providers::xai::Client,
    model_type:      rig::providers::xai::CompletionModel
}

// ---------------------------------------------------------------------------
// Provider dispatch
// ---------------------------------------------------------------------------

/// Get a provider instance by name
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>> {
    match name.to_lowercase().as_str() {
        #[cfg(feature = "provider-anthropic")]
        "anthropic" => Ok(Box::new(AnthropicProvider)),
        #[cfg(feature = "provider-azure")]
        "azure" => Ok(Box::new(AzureProvider)),
        #[cfg(feature = "provider-cohere")]
        "cohere" => Ok(Box::new(CohereProvider)),
        #[cfg(feature = "provider-deepseek")]
        "deepseek" => Ok(Box::new(DeepSeekProvider)),
        #[cfg(feature = "provider-galadriel")]
        "galadriel" => Ok(Box::new(GaladrielProvider)),
        #[cfg(feature = "provider-gemini")]
        "gemini" => Ok(Box::new(GeminiProvider)),
        #[cfg(feature = "provider-groq")]
        "groq" => Ok(Box::new(GroqProvider)),
        #[cfg(feature = "provider-huggingface")]
        "huggingface" => Ok(Box::new(HuggingFaceProvider)),
        #[cfg(feature = "provider-hyperbolic")]
        "hyperbolic" => Ok(Box::new(HyperbolicProvider)),
        #[cfg(feature = "provider-mira")]
        "mira" => Ok(Box::new(MiraProvider)),
        #[cfg(feature = "provider-mistral")]
        "mistral" => Ok(Box::new(MistralProvider)),
        #[cfg(feature = "provider-moonshot")]
        "moonshot" => Ok(Box::new(MoonshotProvider)),
        #[cfg(feature = "provider-ollama")]
        "ollama" => Ok(Box::new(OllamaProvider)),
        #[cfg(feature = "provider-openai")]
        "openai" => Ok(Box::new(OpenAIProvider)),
        #[cfg(feature = "provider-openrouter")]
        "openrouter" => Ok(Box::new(OpenRouterProvider)),
        #[cfg(feature = "provider-perplexity")]
        "perplexity" => Ok(Box::new(PerplexityProvider)),
        #[cfg(feature = "provider-together")]
        "together" => Ok(Box::new(TogetherProvider)),
        #[cfg(feature = "provider-xai")]
        "xai" => Ok(Box::new(XAIProvider)),
        _ => anyhow::bail!(colors::color_error(format!(
            "Unknown provider '{}'\n\nSupported providers:\n  {}",
            name,
            supported_providers().join(", ")
        ))),
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
            "Missing required environment variables for provider '{}':\n  {}\n\nPlease set the environment variable(s) or choose a different provider.",
            provider.name(),
            missing_vars.join(", ")
        )));
    }

    Ok(())
}

/// Get a list of all supported provider names (only those compiled in)
#[allow(clippy::vec_init_then_push)]
pub fn supported_providers() -> Vec<&'static str> {
    let mut providers = Vec::new();
    #[cfg(feature = "provider-anthropic")]
    providers.push("anthropic");
    #[cfg(feature = "provider-azure")]
    providers.push("azure");
    #[cfg(feature = "provider-cohere")]
    providers.push("cohere");
    #[cfg(feature = "provider-deepseek")]
    providers.push("deepseek");
    #[cfg(feature = "provider-galadriel")]
    providers.push("galadriel");
    #[cfg(feature = "provider-gemini")]
    providers.push("gemini");
    #[cfg(feature = "provider-groq")]
    providers.push("groq");
    #[cfg(feature = "provider-huggingface")]
    providers.push("huggingface");
    #[cfg(feature = "provider-hyperbolic")]
    providers.push("hyperbolic");
    #[cfg(feature = "provider-mira")]
    providers.push("mira");
    #[cfg(feature = "provider-mistral")]
    providers.push("mistral");
    #[cfg(feature = "provider-moonshot")]
    providers.push("moonshot");
    #[cfg(feature = "provider-ollama")]
    providers.push("ollama");
    #[cfg(feature = "provider-openai")]
    providers.push("openai");
    #[cfg(feature = "provider-openrouter")]
    providers.push("openrouter");
    #[cfg(feature = "provider-perplexity")]
    providers.push("perplexity");
    #[cfg(feature = "provider-together")]
    providers.push("together");
    #[cfg(feature = "provider-xai")]
    providers.push("xai");
    providers
}
