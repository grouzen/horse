# Plan: Add Color Scheme to REPL

## Overview

Enhance the REPL with a coherent color scheme to improve readability and visual hierarchy. Using `owo-colors` for system messages (lightweight, compile-time) and enabling `termimad`'s built-in colors for markdown rendering. The scheme uses dim colors for debugging/background info, bright colors for highlights and successes, and red for errors. This improves UX by making important information stand out while keeping noise subdued.

## Color Scheme

- **Prompt numbers**: Bright cyan/blue (highlighted)
- **Prompt text**: Gray (slightly dim, not too bright)
- **Status messages** (Loading/Ready): Dim green
- **Tool calls**: Dim yellow or dark gray (debugging level - less important)
- **Errors**: Bright red
- **Warnings**: Dim magenta or orange
- **Success messages**: Bright green
- **AI responses**: Termimad's markdown colors (enabled by removing default-features = false)

## Implementation Steps

### 1. Update Dependencies in `Cargo.toml`

- Add `owo-colors = "4"` to dependencies
- Change `termimad = { version = "0.32", default-features = false }` to `termimad = "0.32"` (enable color support)

### 2. Create Color Utilities Module: `src/colors.rs`

Create a new file with helper functions for the color scheme:
- Import `owo_colors::OwoColorize` trait
- Define helper functions:
  - `format_prompt_number()` - cyan/blue for token counts
  - `format_debug()` - dark gray for tool calls
  - `format_error()` - bright red
  - `format_warning()` - dim magenta or orange
  - `format_success()` - bright green
  - `format_status()` - dim green for loading/ready messages
  - `format_dim()` - gray for normal text in prompt

### 3. Update `src/lib.rs`

Export the new colors module to make it available throughout the codebase.

### 4. Modify `src/main.rs`

Colorize the following output locations:
- Line 87: `">> Loading AGENTS.md..."` - use `format_status()`
- Line 98: `">> Gathering directory structure..."` - use `format_status()`
- Line 106: `"[!] Warning: ..."` - use `format_warning()`
- Line 115: `">> Ready! ..."` - use `format_success()`
- Line 126: `format_prompt()` - numbers in cyan, text in gray using color helpers
- Line 137: `">> Goodbye!"` - use `format_status()`
- Line 159: `">> Error: ..."` - use `format_error()`
- Line 171: `"Warning: ..."` - use `format_warning()`
- Lines 182-186: Startup info - keep default or subtle dim
- Line 188+: Update `format_prompt()` function to apply cyan to numeric values and gray to separators

### 5. Modify `src/hooks.rs`

Colorize the following output locations:
- Line 63: `">> Tool call: ..."` - use `format_debug()` for dim yellow/dark gray
- Line 79: `">> Error: ..."` - use `format_error()` for bright red

### 6. Verify `src/markdown.rs`

Test that markdown rendering uses termimad's color scheme automatically. No code changes needed if termimad handles it (likely the case).

## Verification Steps

1. Run `cargo make test` to ensure tests pass
2. Run `cargo make check-all` for lints and formatting
3. Manual testing:
   - Start the REPL and verify prompt colors (numbers bright cyan, rest gray)
   - Send a query that triggers tool calls - verify they appear in dim color
   - Trigger an error - verify red coloring
   - Check loading/ready messages for appropriate colors
   - Verify markdown responses render with colors
   - Test Ctrl+D exit to see "Goodbye" message

## Design Decisions

- **Chose owo-colors over colored/yansi**: Lightweight, compile-time overhead minimal, modern API
- **Enabled termimad colors**: Better markdown rendering experience, crossterm dependency acceptable
- **Tool calls as debugging**: Dimmed (dark gray or dim yellow) since they're background information
- **Consistent prefix coloring**: Color the entire line including `>>` prefix for visual cohesion

## Technical Considerations

- `owo-colors` is a compile-time library with zero runtime overhead
- Removing `default-features = false` from termimad adds crossterm dependency but enables proper markdown color rendering
- Color scheme maintains visual hierarchy: important info stands out, debugging info recedes
- All colors are ANSI-compatible and will work in most terminal emulators
