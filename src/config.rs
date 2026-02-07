use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for Horse
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_provider")]
    pub provider: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_max_turns")]
    pub max_turns: usize,

    // Optional provider-specific feature sections
    #[serde(default)]
    pub anthropic: Option<AnthropicFeatures>,

    #[serde(default)]
    pub openai: Option<OpenAIFeatures>,

    #[serde(default)]
    pub ollama: Option<OllamaFeatures>,

    #[serde(default)]
    pub gemini: Option<GeminiFeatures>,

    #[serde(default)]
    pub groq: Option<GroqFeatures>,

    #[serde(default)]
    pub cohere: Option<CohereFeatures>,

    #[serde(default)]
    pub deepseek: Option<DeepSeekFeatures>,

    #[serde(default)]
    pub galadriel: Option<GaladrielFeatures>,

    #[serde(default)]
    pub azure: Option<AzureFeatures>,

    #[serde(default)]
    pub huggingface: Option<HuggingFaceFeatures>,

    #[serde(default)]
    pub hyperbolic: Option<HyperbolicFeatures>,

    #[serde(default)]
    pub mira: Option<MiraFeatures>,

    #[serde(default)]
    pub mistral: Option<MistralFeatures>,

    #[serde(default)]
    pub moonshot: Option<MoonshotFeatures>,

    #[serde(default)]
    pub openrouter: Option<OpenRouterFeatures>,

    #[serde(default)]
    pub perplexity: Option<PerplexityFeatures>,

    #[serde(default)]
    pub together: Option<TogetherFeatures>,

    #[serde(default)]
    pub voyageai: Option<VoyageAIFeatures>,

    #[serde(default)]
    pub xai: Option<XAIFeatures>,
}

// Provider-specific feature structures
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicFeatures {
    #[serde(default)]
    pub prompt_caching: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIFeatures {
    #[serde(default)]
    pub response_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaFeatures {
    #[serde(default)]
    pub keep_alive: Option<String>,

    #[serde(default)]
    pub num_ctx: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiFeatures {
    // Future: safety settings, etc.
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroqFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct CohereFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeepSeekFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct GaladrielFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureFeatures {
    // Future: deployment name, API version, etc.
}

#[derive(Debug, Clone, Deserialize)]
pub struct HuggingFaceFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyperbolicFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiraFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct MistralFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoonshotFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct PerplexityFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct TogetherFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoyageAIFeatures {
    // Future: specific features
}

#[derive(Debug, Clone, Deserialize)]
pub struct XAIFeatures {
    // Future: specific features
}

// Default value functions
fn default_provider() -> String {
    "anthropic".to_string()
}

fn default_model() -> String {
    "claude-sonnet-4-0".to_string()
}

fn default_max_turns() -> usize {
    20
}

impl Default for Config {
    fn default() -> Self {
        Config {
            provider: default_provider(),
            model: default_model(),
            max_turns: default_max_turns(),
            anthropic: None,
            openai: None,
            ollama: None,
            gemini: None,
            groq: None,
            cohere: None,
            deepseek: None,
            galadriel: None,
            azure: None,
            huggingface: None,
            hyperbolic: None,
            mira: None,
            mistral: None,
            moonshot: None,
            openrouter: None,
            perplexity: None,
            together: None,
            voyageai: None,
            xai: None,
        }
    }
}

impl Config {
    /// Load configuration from the default location (~/.config/horse/config.toml)
    /// Returns default config if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Get the default config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Unable to determine user config directory")?;

        Ok(config_dir.join("horse").join("config.toml"))
    }

    /// Merge CLI arguments into this config (CLI args take precedence)
    pub fn merge_with_args(
        &mut self,
        provider: Option<String>,
        model: Option<String>,
        max_turns: Option<usize>,
    ) {
        if let Some(p) = provider {
            self.provider = p;
        }
        if let Some(m) = model {
            self.model = m;
        }
        if let Some(t) = max_turns {
            self.max_turns = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-sonnet-4-0");
        assert_eq!(config.max_turns, 20);
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
            provider = "openai"
            model = "gpt-4"
            max_turns = 10

            [anthropic]
            prompt_caching = true

            [ollama]
            keep_alive = "5m"
            num_ctx = 4096
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.max_turns, 10);
        assert!(config.anthropic.is_some());
        assert!(config.anthropic.unwrap().prompt_caching);
        assert!(config.ollama.is_some());
    }

    #[test]
    fn test_merge_with_args() {
        let mut config = Config::default();
        config.merge_with_args(
            Some("gemini".to_string()),
            Some("gemini-pro".to_string()),
            Some(15),
        );

        assert_eq!(config.provider, "gemini");
        assert_eq!(config.model, "gemini-pro");
        assert_eq!(config.max_turns, 15);
    }
}
