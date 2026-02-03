# Plan: Simple Agentic Search REPL (Anthropic-only)

Build a minimal REPL-based search assistant using Anthropic Claude with read file and read-only bash tools. The REPL displays tool calls and outputs in real-time, hides reasoning tokens by default, and uses a strict whitelist approach for bash command safety.

## Steps

### 1. Update dependencies in `Cargo.toml`

Add `tokio` (full features), `clap` (derive), `anyhow`, `serde`/`serde_json`, and `color-eyre`.

### 2. Create tool module

Files: `src/tools/mod.rs`, `src/tools/read_file.rs`, `src/tools/bash.rs`

- **ReadFile tool**: accepts `path`, optional `start_line`/`end_line`, resolves paths relative to base dir, rejects `../` traversal, truncates at 50KB or 1000 lines with `[truncated]` note
- **BashCommand tool**: accepts `command`, validates first word against whitelist (`grep`, `find`, `cat`, `head`, `tail`, `ls`, `tree`, `wc`, `file`, `rg`), rejects pipes `|`, semicolons `;`, `&&`, `||`, backticks, `$()`, redirects `>` `<`, executes with cwd set to base dir, kills after 30 seconds using `tokio::time::timeout`

### 3. Implement `ProgressHook` in `src/hooks.rs`

Implement rig's `PromptHook` trait to:
- Print `🔧 Calling: {tool}({args})` on tool call
- Print truncated result or brief error summary on tool result
- Skip reasoning tokens
- Track token usage including Anthropic cache reads:
  - Use `Arc<Mutex<Usage>>` to accumulate tokens across requests
  - Display per-request token counts (input/output/total)
  - Highlight cache reads with "💾 Cache read: X tokens" when `cached_input_tokens > 0`
  - Show session totals on exit including cumulative cache reads

### 4. Build REPL loop in `src/main.rs`

- Parse CLI args with clap: `--dir` (defaults to `.`), `--model` (defaults to `claude-sonnet-4-0`), `--max-turns` (defaults to `20`)
- Canonicalize `--dir` to absolute path for consistent path handling
- Load `AGENTS.md` from target dir into system preamble if present
- Initialize Anthropic client from `ANTHROPIC_API_KEY` env var
- Create agent with tools (passing base dir), preamble describing search assistant role, and configurable max turns
- Run input loop: read line from stdin → call `agent.prompt().with_history().with_hook().await` → print response → repeat until EOF/Ctrl+C

### 5. Add directory context to preamble

On startup, run `find . -maxdepth 3 -type f` in target dir to gather file list, include in system prompt so agent knows available files.
