# 🐴 Horse (better than cat?)

Minimalist, read-only CLI search tool: Unix philosophy meets agentic RAG for intelligent directory exploration.

Your trusty stead in the modern Unix world of agentic command line utilities!

## Features

- **Interactive REPL** — Chat with agentic LLMs to explore and understand code and knowledge bases
- **File reading** — Read files with optional line range support
- **Safe bash execution** — Whitelisted read-only commands (`grep`, `find`, `cat`, `head`, `tail`, `ls`, `tree`, `wc`, `file`, `rg`)
- **AGENTS.md support** — Automatically loads project-specific instructions
- **Markdown rendering** — Rich formatting of model responses with syntax highlighting
- **Token tracking** — Displays usage stats including cache reads

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Configuration

Horse supports configuration via a TOML file at `~/.config/horse/config.toml`. 

To get started, copy the example configuration:

```bash
mkdir -p ~/.config/horse
cp config.example.toml ~/.config/horse/config.toml
```

Edit the config file to set your preferred provider, model, and provider-specific features. All settings in the config file can be overridden via command-line arguments.

See [config.example.toml](config.example.toml) for detailed documentation of all available options.

## Supported Providers

Horse supports 18 LLM providers through the [rig-core](https://github.com/0xPlaygrounds/rig) library:

### Cloud Providers (Require API Keys)
- **Anthropic** - Claude models (`ANTHROPIC_API_KEY`) - [Get API key](https://console.anthropic.com/)
- **OpenAI** - GPT models (`OPENAI_API_KEY`) - [Get API key](https://platform.openai.com/api-keys)
- **Google Gemini** - Gemini models (`GEMINI_API_KEY`) - [Get API key](https://ai.google.dev/)
- **Azure OpenAI** - Microsoft-hosted OpenAI (`AZURE_API_KEY`, `AZURE_API_VERSION`, `AZURE_ENDPOINT`)
- **Groq** - Fast inference (`GROQ_API_KEY`) - [Get API key](https://console.groq.com/)
- **Cohere** - Command models (`COHERE_API_KEY`) - [Get API key](https://dashboard.cohere.com/)
- **DeepSeek** - DeepSeek models (`DEEPSEEK_API_KEY`) - [Get API key](https://platform.deepseek.com/)
- **Mistral AI** - Mistral models (`MISTRAL_API_KEY`) - [Get API key](https://console.mistral.ai/)
- **Perplexity** - Search-augmented models (`PERPLEXITY_API_KEY`) - [Get API key](https://www.perplexity.ai/settings/api)
- **xAI** - Grok models (`XAI_API_KEY`) - [Get API key](https://x.ai/)
- **OpenRouter** - Multi-provider access (`OPENROUTER_API_KEY`) - [Get API key](https://openrouter.ai/keys)
- **Together AI** - Fast inference (`TOGETHER_API_KEY`) - [Get API key](https://api.together.xyz/settings/api-keys)
- **Hugging Face** - HF Inference API (`HUGGINGFACE_API_KEY`) - [Get API key](https://huggingface.co/settings/tokens)
- **Hyperbolic** - GPU inference (`HYPERBOLIC_API_KEY`) - [Get API key](https://www.hyperbolic.xyz/)
- **Galadriel** - Blockchain-based AI (`GALADRIEL_API_KEY`) - [Get API key](https://galadriel.com/)
- **Mira** - AI infrastructure (`MIRA_API_KEY`)
- **Moonshot** - Chinese LLM provider (`MOONSHOT_API_KEY`) - [Get API key](https://platform.moonshot.cn/)

### Local Providers (No API Key Required)
- **Ollama** - Run models locally (requires [Ollama](https://ollama.ai/) server running)

See [config.example.toml](config.example.toml) for complete list with default models and configuration options.

## Usage

### Using Anthropic (Default)

```bash
# Set your API key
export ANTHROPIC_API_KEY=your_key

# Run in current directory
horse

# Run in a specific directory
horse /path/to/project
```

### Using OpenAI

```bash
export OPENAI_API_KEY=your_key
horse --provider openai --model gpt-4o
```

### Using Ollama (Local)

```bash
# Start Ollama server first: ollama serve
horse --provider ollama --model llama3
```

### Using Google Gemini

```bash
export GEMINI_API_KEY=your_key
horse --provider gemini --model gemini-1.5-pro
```

### Using Azure OpenAI

```bash
export AZURE_API_KEY=your_key
export AZURE_API_VERSION=2024-02-01
export AZURE_ENDPOINT=https://your-resource.openai.azure.com/
horse --provider azure --model gpt-4
```

### CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --provider` | `anthropic` | LLM provider to use (see Supported Providers) |
| `-m, --model` | Provider-specific | Model name (e.g., `claude-sonnet-4-0`, `gpt-4o`, `llama3`) |
| `-t, --max-turns` | `20` | Max agent turns per query |
| `-c, --config` | `~/.config/horse/config.toml` | Path to config file |

### Configuration Priority

Settings are applied in this order (later overrides earlier):
1. Config file defaults
2. Config file settings
3. Command-line arguments

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/) (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please run `cargo make check-all` before submitting.
