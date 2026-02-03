# 🐴 Horse (better than cat?)

Minimalist, read-only CLI search tool: Unix philosophy meets agentic RAG for intelligent directory exploration.

Your trusty stead in the modern Unix world of agentic command line utilities!

## Features

- **Interactive REPL** — Chat with agentic LLMs to explore and understand code and knowledge bases
- **File reading** — Read files with optional line range support
- **Safe bash execution** — Whitelisted read-only commands (`grep`, `find`, `cat`, `head`, `tail`, `ls`, `tree`, `wc`, `file`, `rg`)
- **AGENTS.md support** — Automatically loads project-specific instructions
- **Token tracking** — Displays usage stats including cache reads

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

```bash
# Set your API key
export ANTHROPIC_API_KEY=your_key

# Run in current directory
horse

# Run in a specific directory
horse --dir /path/to/project

# Use a different model
horse --model claude-sonnet-4-0

# Set max conversation turns
horse --max-turns 30
```

### CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `-d, --dir` | `.` | Target directory to search |
| `-m, --model` | `claude-sonnet-4-0` | Claude model to use |
| `-t, --max-turns` | `20` | Max agent turns per query |

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/) (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please run `cargo make check-all` before submitting.
