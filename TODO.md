# TODO

## Simple Agentic Search REPL (Anthropic-only)

- [x] **Step 1**: Update dependencies in `Cargo.toml`
  - Add `tokio` (full features), `clap` (derive), `anyhow`, `serde`/`serde_json`, and `color-eyre`

- [x] **Step 2**: Create tool module
  - Files: `src/tools/mod.rs`, `src/tools/read_file.rs`, `src/tools/bash.rs`
  - **ReadFile tool**: accepts `path`, optional `start_line`/`end_line`, resolves paths relative to base dir, rejects `../` traversal, truncates at 50KB or 1000 lines with `[truncated]` note
  - **BashCommand tool**: accepts `command`, validates first word against whitelist (`grep`, `find`, `cat`, `head`, `tail`, `ls`, `tree`, `wc`, `file`, `rg`), rejects pipes `|`, semicolons `;`, `&&`, `||`, backticks, `$()`, redirects `>` `<`, executes with cwd set to base dir, kills after 30 seconds using `tokio::time::timeout`

- [ ] **Step 3**: Implement `ProgressHook` in `src/hooks.rs`
  - Implement rig's `PromptHook` trait to:
    - Print `🔧 Calling: {tool}({args})` on tool call
    - Print truncated result or brief error summary on tool result
    - Skip reasoning tokens

- [ ] **Step 4**: Build REPL loop in `src/main.rs`
  - Parse CLI args with clap: `--dir` (defaults to `.`), `--model` (defaults to `claude-sonnet-4-20250514`)
  - Canonicalize `--dir` to absolute path for consistent path handling
  - Load `AGENTS.md` from target dir into system preamble if present
  - Initialize Anthropic client from `ANTHROPIC_API_KEY` env var
  - Create agent with tools (passing base dir) and preamble describing search assistant role
  - Run input loop: read line from stdin → call `agent.prompt().with_history().with_hook().await` → print response → repeat until EOF/Ctrl+C

- [ ] **Step 5**: Add directory context to preamble
  - On startup, run `find . -maxdepth 3 -type f` in target dir to gather file list, include in system prompt so agent knows available files
