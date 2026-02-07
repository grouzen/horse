# Plan: Multi-Provider LLM Support in Horse

## Overview

Horse has been refactored into a modular structure with `agent/` and `console/` modules. The **only** hardcoded Anthropic type is in the `Repl` struct ([console/repl.rs](horse/src/console/repl.rs#L50)). All hooks and tools are already generic over `CompletionModel`, making multi-provider support straightforward. We'll add a config system at `~/.config/horse/config.toml` with CLI overrides to support all 19 rig-core providers.

## Goals

- Support all 19 rig-core LLM providers (OpenAI, Anthropic, Ollama, Gemini, Groq, Cohere, etc.)
- Global configuration file at `~/.config/horse/config.toml`
- CLI arguments override config file values
- Provider-specific features configurable per-provider (e.g., Anthropic prompt caching)
- Clear error messages for missing credentials
- Maintain backward compatibility (default to Anthropic)

## Implementation Steps

### 1. Add provider configuration infrastructure

**Files to modify:**
- [horse/Cargo.toml](horse/Cargo.toml)
- Create [horse/src/config.rs](horse/src/config.rs)
- [horse/src/lib.rs](horse/src/lib.rs)

**Changes:**
- Add `toml` to dependencies in Cargo.toml (serde already present)
- Create `config.rs` module with well-typed provider features:
  - `Config` struct with optional provider-specific feature sections:
    ```rust
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
        
        // ... other providers
    }
    
    #[derive(Debug, Clone, Deserialize)]
    pub struct AnthropicFeatures {
        #[serde(default)]
        pub prompt_caching: bool,
    }
    
    #[derive(Debug, Clone, Deserialize)]
    pub struct OpenAIFeatures {
        #[serde(default)]
        pub response_format: Option<String>,
        // Future: seed, temperature overrides, etc.
    }
    
    #[derive(Debug, Clone, Deserialize)]
    pub struct OllamaFeatures {
        #[serde(default)]
        pub keep_alive: Option<String>,
        #[serde(default)]
        pub num_ctx: Option<u32>,
    }
    ```
  - `load_config()` function: reads from `~/.config/horse/config.toml`, returns defaults if missing
  - `merge_with_args()` method: CLI args override config file values
- Define TOML schema:
  ```toml
  provider = "anthropic"
  model = "claude-sonnet-4-0"
  max_turns = 20
  
  # Provider-specific features (only the active provider's section is used)
  [anthropic]
  prompt_caching = true
  
  [openai]
  # response_format = "json"  # future feature
  
  [ollama]
  keep_alive = "5m"
  num_ctx = 4096
  ```
- Export config module from lib.rs

### 2. Create provider abstraction layer

**Files to create:**
- [horse/src/providers.rs](horse/src/providers.rs)
- [horse/src/providers/](horse/src/providers/) directory with module per provider

**Changes:**
- Create `Provider` trait with required methods:
  ```rust
  pub trait Provider {
      /// Get the name of this provider (e.g., "anthropic", "openai")
      fn name(&self) -> &str;
      
      /// Get required environment variables for this provider
      fn required_env_vars(&self) -> Vec<&str>;
      
      /// Get the default model for this provider
      fn default_model(&self) -> &str;
      
      /// Create an agent for this provider with given config and context
      fn create_agent(
          &self,
          config: &Config,
          base_dir: &Path,
          preamble: &str,
      ) -> Result<Agent<impl CompletionModel>>;
  }
  ```
- Create concrete provider structs (19 total) that implement `Provider` trait:
  - `AnthropicProvider`, `OpenAIProvider`, `GeminiProvider`, `OllamaProvider`
  - `GroqProvider`, `CohereProvider`, `DeepSeekProvider`, `GaladrielProvider`
  - `AzureProvider`, `HuggingFaceProvider`, `HyperbolicProvider`, `MiraProvider`
  - `MistralProvider`, `MoonshotProvider`, `OpenRouterProvider`, `PerplexityProvider`
  - `TogetherProvider`, `VoyageAIProvider`, `xAIProvider`
- Each provider's `create_agent()` implementation:
  - Accesses its specific features directly from config (e.g., `config.anthropic`)
  - Applies provider-specific features type-safely
  - Returns configured agent with tools
- Create `get_provider(name: &str) -> Result<Box<dyn Provider>>` factory function
- Import all rig provider modules (anthropic, openai, gemini, cohere, etc.)
- Add to lib.rs exports

**Example provider implementation:**
```rust
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
    ) -> Result<Agent<impl CompletionModel>> {
        let client = anthropic::Client::from_env();
        let mut builder = client.completion(&config.model);
        
        // Direct access to provider-specific features
        if let Some(features) = &config.anthropic {
            if features.prompt_caching {
                builder = builder.with_prompt_caching();
            }
        }
        
        let agent = builder
            .preamble(preamble)
            .max_tokens(4096)
            .tool(BashCommand::new(base_dir))
            .tool(ReadFile::new(base_dir))
            .tool(SearchDocs::new(base_dir))
            .build();
        
        Ok(agent)
    }
}

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
    ) -> Result<Agent<impl CompletionModel>> {
        let client = ollama::Client::from_env();
        let mut builder = client.completion(&config.model);
        
        // Direct access to Ollama-specific features
        if let Some(features) = &config.ollama {
            if let Some(keep_alive) = &features.keep_alive {
                builder = builder.keep_alive(keep_alive);
            }
            if let Some(num_ctx) = features.num_ctx {
                builder = builder.context_length(num_ctx);
            }
        }
        
        let agent = builder
            .preamble(preamble)
            .tool(BashCommand::new(base_dir))
            .tool(ReadFile::new(base_dir))
            .tool(SearchDocs::new(base_dir))
            .build();
        
        Ok(agent)
    }
}
```

**Key benefits of trait-based design:**
- **Separation of concerns**: Each provider owns its agent creation logic
- **Extensibility**: Adding a new provider = create struct + implement trait
- **Type safety**: Direct access to `config.anthropic`, `config.ollama`, etc.
- **No helper methods needed**: Each provider knows which config field to access
- **Compile-time safety**: Can't call `.with_prompt_caching()` on OpenAI client
- **Testability**: Can mock providers by implementing the trait
- **Clear responsibilities**: Provider trait defines the contract

### 3. Update Repl to use AgentWrapper

**Files to modify:**
- [horse/src/console/repl.rs](horse/src/console/repl.rs)

**Changes:**
- Remove hardcoded `use rig::{agent::Agent, providers::anthropic}` import
- Add `use crate::providers::AgentWrapper` import
- Update struct field (line ~50):
  - Change `agent: Agent<anthropic::completion::CompletionModel>` to `agent: Box<dyn AgentWrapper>`
- Update `new()` signature:
  - Change parameter from `Agent<anthropic::completion::CompletionModel>` to `Box<dyn AgentWrapper>`
- Update `run()` method to use AgentWrapper trait:
  - Replace `.prompt(input).with_history(&mut history).with_hook(hook.clone()).await`
  - With `.prompt(input.to_string(), &mut history, hook.clone()).await`
- No generic type parameters needed—using trait object for dynamic dispatch
- This approach allows any provider's agent to be used without type constraints in main

### 4. Update main.rs for provider selection

**Files to modify:**
- [horse/src/main.rs](horse/src/main.rs)

**Changes:**
- Extend `Args` struct (lines 16-27):
  - Add `--provider` option with default "anthropic"
  - Keep existing `--model` (overrides config)
  - Keep existing `--max-turns` (overrides config)
- Replace client initialization section (lines 147-165):
  - Load config via `config::load_config(&args.config)`
  - Merge CLI args: `config.merge_with_args(&args)`
  - Get provider instance: `let provider = providers::get_provider(&config.provider)?`
  - Validate env vars: Check `provider.required_env_vars()` are set, clear error if missing
  - Create agent: `let agent = provider.create_agent(&config, &base_dir, &preamble)?`
- Initialize Repl with agent: `let mut repl = Repl::new(agent);`

### 5. Create example config file

**Files to create:**
- [horse/config.example.toml](horse/config.example.toml)

**Content:**
- All 19 providers listed with comments
- Required env vars documented per provider
- Example provider-specific features with well-typed sections
- Document different auth patterns (API key, base URL, multi-var for Azure)

**Example structure:**
```toml
# Horse Configuration File
# Location: ~/.config/horse/config.toml

provider = "anthropic"  # Default provider
model = "claude-sonnet-4-0"  # Default model for the provider
max_turns = 20  # Maximum conversation turns

# === Provider-Specific Features ===
# Each provider can have its own features section.
# Only the section matching your active provider is used.

# Anthropic-specific features
[anthropic]
prompt_caching = true  # Enable Anthropic's prompt caching feature

# OpenAI-specific features (for when provider = "openai")
[openai]
# Future: response_format, seed, etc.

# Ollama-specific features (for when provider = "ollama")
[ollama]
keep_alive = "5m"      # How long to keep model loaded in memory
num_ctx = 4096         # Context window size

# Groq-specific features (for when provider = "groq")
[groq]
# Future: temperature overrides, etc.

# === Supported Providers ===
# Uncomment and set the appropriate provider and model

# OpenAI - Requires: OPENAI_API_KEY
# provider = "openai"
# model = "gpt-4o-mini"

# Ollama - Local, requires running Ollama server
# provider = "ollama"
# model = "llama3"

# Google Gemini - Requires: GEMINI_API_KEY
# provider = "gemini"
# model = "gemini-1.5-pro"

# ... etc for all 19 providers
```

**Benefits of this approach:**
- Type-safe: Rust code can't accidentally access wrong features
- IDE autocomplete works for feature fields  
- Validation at deserialization time
- Easy to extend with new provider-specific features
- Clear separation between providers in config file

### 6. Update documentation

**Files to modify:**
- [horse/README.md](horse/README.md)
- [horse/AGENTS.md](horse/AGENTS.md)

**Changes to README.md:**
- Add "Supported Providers" section listing all 19
- Document config file location and format
- Add examples for OpenAI, Anthropic, Ollama, Gemini
- Update env var section to mention multiple providers
- Add migration notes from Anthropic-only

**Example providers section:**
```markdown
## Supported Providers

Horse supports 19 LLM providers through the rig-core library:

### Cloud Providers
- **Anthropic** - Claude models (ANTHROPIC_API_KEY)
- **OpenAI** - GPT models (OPENAI_API_KEY)
- **Google Gemini** - Gemini models (GEMINI_API_KEY)
- **Groq** - Fast inference (GROQ_API_KEY)
- **Cohere** - Command models (COHERE_API_KEY)
... etc

### Local Providers
- **Ollama** - Local models (no API key, requires server)

See config.example.toml for complete list and configuration options.
```

**Changes to AGENTS.md:**
- Update tech stack section to mention multi-provider support
- Add note about Rig library provider abstraction

### 7. Add validation and error handling

**Files to modify:**
- [horse/src/providers.rs](horse/src/providers.rs)
- [horse/src/config.rs](horse/src/config.rs)

**Changes:**
- In `providers.rs`, validate:
  - Provider name is recognized (clear error with list of valid names)
  - Required env vars are present (name them specifically in error)
  - Model name is reasonable (warn if unknown for provider)
- In `config.rs`, validate:
  - Config file is valid TOML (helpful parse errors)
  - Required fields present with defaults
- Use `anyhow::Context` for error chains

**Example error message:**
```
Error: Missing required environment variable for provider 'openai'

Required: OPENAI_API_KEY

Please set the environment variable or choose a different provider.
Available providers: anthropic, openai, gemini, ollama, ...
```

## Verification Testing

### Test provider categories:

1. **Anthropic (default)**
   ```bash
   ANTHROPIC_API_KEY=xxx horse --provider anthropic
   ```

2. **OpenAI**
   ```bash
   OPENAI_API_KEY=xxx horse --provider openai --model gpt-4
   ```

3. **Ollama (local, no key)**
   ```bash
   horse --provider ollama --model llama3
   ```

4. **Config file**
   ```bash
   mkdir -p ~/.config/horse
   cp config.example.toml ~/.config/horse/config.toml
   # Edit config.toml to set provider
   horse  # Should use config defaults
   ```

5. **CLI override**
   ```bash
   # Config has anthropic, override with openai
   horse --provider openai --model gpt-4o-mini
   ```

6. **Missing credentials**
   ```bash
   # No env vars set
   horse --provider openai
   # Should show: Error: Missing required environment variable OPENAI_API_KEY
   ```

7. **Prompt caching**
   ```bash
   # With Anthropic and caching enabled in config
   ANTHROPIC_API_KEY=xxx horse
   # Should show cached token counts in prompt
   ```

8. **Backward compatibility**
   ```bash
   # Old usage without --provider flag
   ANTHROPIC_API_KEY=xxx horse
   # Should default to Anthropic
   ```

### Run tests:
```bash
cargo make test
cargo make check-all
```

## Design Decisions

### 1. AgentWrapper trait object approach
**Decision:** Use `Box<dyn AgentWrapper>` in Repl instead of making it generic

**Rationale:** 
- Simpler main.rs: No need to handle different concrete types
- Dynamic dispatch: All providers return same type from create_agent()
- Clean abstraction: Repl only depends on AgentWrapper trait, not specific providers
- Follows Rust best practices for runtime polymorphism
- Avoids macro complexity or enum dispatch boilerplate

### 2. Provider as trait, not enum
**Decision:** Use trait-based design with concrete provider structs

**Rationale:** 
- More extensible: Adding providers doesn't modify central enum
- Better separation: Each provider owns its creation logic
- Rust idiomatic: Trait objects allow runtime polymorphism
- Cleaner code: No giant match statement in providers.rs
- Each provider can be in its own module for organization

### 3. Default provider
**Decision:** Default to Anthropic

**Rationale:** Maintains backward compatibility with existing Horse usage. Users can override via config or CLI.

### 4. Configuration location
**Decision:** Single global config at `~/.config/horse/config.toml`

**Rationale:** Reduces complexity. Skip per-directory configs initially. Can add later if needed.

### 5. Validation timing
**Decision:** Validate env vars on startup

**Rationale:** Helpful errors immediately rather than waiting for first API call. Better UX.

### 6. Scope limitation
**Decision:** All rig-core only (19 providers)

**Rationale:** Skip rig-integrations (Bedrock, Vertex, etc.) due to complex multi-var auth patterns. Can add later as separate features.

### 7. Config precedence
**Decision:** CLI args > config file > defaults

**Rationale:** Standard precedence pattern. Allows quick overrides without config changes.

### 8. Provider-specific features
**Decision:** Direct field access on Config struct with strongly-typed feature structs

**Rationale:** 
- Simplicity: No helper methods needed, direct access like `config.anthropic`
- Type safety: Compiler catches mistakes like accessing wrong features
- Better IDE support: Autocomplete and documentation for feature fields
- Clear validation: Serde validates feature structure at config load time
- Each provider's `create_agent()` knows exactly which config field to access
- Flexibility for users: Each provider can have unique, well-documented features
- Some features (like Anthropic's prompt caching) only work with specific providers

### 9. Agent creation ownership
**Decision:** Provider trait owns `create_agent()` method

**Rationale:**
- Single responsibility: Provider knows how to create its own agent
- Avoids god function: No giant `create_agent_for_provider()` with match statement
- Easier testing: Mock individual providers
- Better organization: Each provider's logic is self-contained

## Migration Guide (for users)

### From Anthropic-only to multi-provider

**Old usage (still works):**
```bash
export ANTHROPIC_API_KEY=xxx
horse
```

**New usage with explicit provider:**
```bash
export OPENAI_API_KEY=xxx
horse --provider openai --model gpt-4o-mini
```

**New usage with config file:**
```bash
# Create config
mkdir -p ~/.config/horse
cat > ~/.config/horse/config.toml << EOF
provider = "openai"
model = "gpt-4o-mini"
max_turns = 20
EOF

export OPENAI_API_KEY=xxx
horse  # Uses config settings
```

## Dependencies to Add

Current dependencies in [horse/Cargo.toml](horse/Cargo.toml):
- `rig-core = "0.30.0"` (already present)
- `serde` with derive (already present)
- `anyhow` (already present)

**New dependencies needed:**
- `toml = "0.8"` - For config file parsing

All provider modules are included in `rig-core`, no additional integration crates needed for the initial 19 providers.

## Supported Providers List

From rig-core (all include completion support):

1. **Anthropic** - `ANTHROPIC_API_KEY` - Claude models
2. **Azure** - `AZURE_API_KEY`, `AZURE_API_VERSION`, `AZURE_ENDPOINT` - Azure OpenAI
3. **Cohere** - `COHERE_API_KEY` - Command models
4. **DeepSeek** - `DEEPSEEK_API_KEY` - DeepSeek models
5. **Galadriel** - `GALADRIEL_API_KEY` - Galadriel models
6. **Gemini** - `GEMINI_API_KEY` - Google Gemini models
7. **Groq** - `GROQ_API_KEY` - Fast inference
8. **HuggingFace** - `HUGGINGFACE_API_KEY` - HF inference API
9. **Hyperbolic** - `HYPERBOLIC_API_KEY` - Hyperbolic models
10. **Mira** - `MIRA_API_KEY` - Mira models
11. **Mistral** - `MISTRAL_API_KEY` - Mistral models
12. **Moonshot** - `MOONSHOT_API_KEY` - Moonshot models
13. **Ollama** - `OLLAMA_API_BASE_URL` (optional) - Local models
14. **OpenAI** - `OPENAI_API_KEY`, `OPENAI_BASE_URL` (optional) - GPT models
15. **OpenRouter** - `OPENROUTER_API_KEY` - Multi-provider routing
16. **Perplexity** - `PERPLEXITY_API_KEY` - Perplexity models
17. **Together** - `TOGETHER_API_KEY` - Together AI
18. **VoyageAI** - `VOYAGE_API_KEY` - Voyage embeddings/completion
19. **xAI** - `XAI_API_KEY` - Grok models

## Future Enhancements (Not in scope)

- Add rig-integrations providers (AWS Bedrock, Google Vertex, EternalAI)
- Per-directory config files (`./horse.toml`)
- Multi-provider sessions (switch providers mid-conversation)
- Provider-specific model aliasing
- Auto-detect provider from available env vars
- Provider-specific instructions in AGENTS.md (e.g., `AGENTS-anthropic.md`)
- Interactive provider selection on first run
- Cost tracking per provider
- Rate limit handling per provider
