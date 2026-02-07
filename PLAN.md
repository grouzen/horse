# Plan: Add Braille Spinner for Provider Responses

## Overview

Add a modern braille-pattern loader using `indicatif` that displays "Processing" during LLM response generation and tool execution. The spinner will run as a concurrent task while the main thread awaits responses.

## Implementation Steps

### 1. Add `indicatif` dependency

**File**: `Cargo.toml`

Add `indicatif = "0.17"` to the `[dependencies]` section to enable braille spinner functionality.

### 2. Create spinner helper module

**File**: `src/spinner.rs` (new file)

Create a new module with:
- `create_braille_spinner(message: &str) -> ProgressBar` function
  - Configures spinner with braille pattern characters: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
  - Sets appropriate tick rate (~80ms for smooth animation)
  - Uses style template with custom message
  - Returns `ProgressBar` handle for lifecycle control

**File**: `src/lib.rs`

Add `pub mod spinner;` declaration to expose the new module.

### 3. Add spinner to main prompt loop

**File**: `src/main.rs` (lines ~172-183)

Modify the `run_repl()` function:
- Import `horse::spinner::create_braille_spinner`
- Before `agent.prompt().await`: start spinner with "Processing" message
- After response received (both `Ok` and `Err` branches): call `spinner.finish_and_clear()` to cleanly remove from terminal
- Ensure spinner cleanup happens in all code paths

### 4. Integrate spinner in hooks

**File**: `src/hooks.rs`

Update `ProgressHook` to show spinner during tool execution:
- Add `Option<ProgressBar>` field to `ProgressHook` struct
- In `on_tool_call` (line ~51): start new spinner before tool execution
- In `on_tool_result` (line ~72): stop spinner after result
- Ensure spinner is cleared before printing tool outputs to avoid visual conflicts

### 5. Handle edge cases

- Ensure spinner cleanup in panic/error paths (RAII pattern or explicit cleanup)
- Verify spinner doesn't interfere with `termimad` markdown rendering
- Check that spinner output doesn't corrupt terminal state or command history

## Verification Steps

1. Run `cargo make test` to ensure no regressions
2. Run `cargo make check-all` for lints and clippy warnings
3. Manual testing:
   - Launch REPL
   - Send a prompt
   - Verify braille spinner appears with "Processing" message
   - Verify spinner disappears cleanly when response renders
   - Test with tool-calling prompts to confirm spinner during tool execution
   - Verify terminal state remains clean after spinner removal

## Technical Decisions

- **Library choice**: Using `indicatif` over custom implementation for reliability and maintained braille patterns
- **Scope**: Spinner shows during both LLM generation AND tool execution for consistent user feedback
- **Message**: Using "Processing" as requested by user
- **Pattern**: Braille characters provide modern, minimal visual feedback without being distracting

## Braille Pattern

The spinner will cycle through these Unicode braille characters:
```
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

These create a smooth rotating animation effect in the terminal.
