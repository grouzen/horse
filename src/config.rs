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
}

// Provider-specific feature structures
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicFeatures {
    #[serde(default)]
    pub prompt_caching: bool,
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
            provider = "anthropic"
            model = "claude-sonnet-4-0"
            max_turns = 10

            [anthropic]
            prompt_caching = true
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-sonnet-4-0");
        assert_eq!(config.max_turns, 10);
        assert!(config.anthropic.is_some());
        assert!(config.anthropic.unwrap().prompt_caching);
    }

    #[test]
    fn test_merge_with_args() {
        let mut config = Config::default();
        config.merge_with_args(
            Some("anthropic".to_string()),
            Some("claude-opus-4-0".to_string()),
            Some(15),
        );

        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-opus-4-0");
        assert_eq!(config.max_turns, 15);
    }
}
