# TODO: Add Color Scheme to REPL

## Steps

- [x] **Step 1: Update Dependencies in `Cargo.toml`**
  - Add `owo-colors = "4"` to dependencies
  - Change `termimad = { version = "0.32", default-features = false }` to `termimad = "0.32"` (enable color support)

- [x] **Step 2: Create Color Utilities Module: `src/colors.rs`**
  - Create a new file with helper functions for the color scheme
  - Import `owo_colors::OwoColorize` trait
  - Define helper functions:
    - `color_prompt_number()` - cyan/blue for token counts
    - `color_debug()` - dark gray for tool calls
    - `color_error()` - bright red
    - `color_warning()` - dim magenta or orange
    - `color_success()` - bright green
    - `color_status()` - dim green for loading/ready messages
    - `color_dim()` - gray for normal text in prompt

- [x] **Step 3: Update `src/lib.rs`**
  - Export the new colors module to make it available throughout the codebase

- [x] **Step 4: Modify `src/main.rs`**
  - Colorize the following output locations:
    - Line 87: `">> Loading AGENTS.md..."` - use `color_status()`
    - Line 98: `">> Gathering directory structure..."` - use `color_status()`
    - Line 106: `"[!] Warning: ..."` - use `color_warning()`
    - Line 115: `">> Ready! ..."` - use `color_success()`
    - Line 126: `format_prompt()` - numbers in cyan, text in gray using color helpers
    - Line 137: `">> Goodbye!"` - use `color_status()`
    - Line 159: `">> Error: ..."` - use `color_error()`
    - Line 171: `"Warning: ..."` - use `color_warning()`
    - Lines 182-186: Startup info - keep default or subtle dim
    - Line 188+: Update `format_prompt()` function to apply cyan to numeric values and gray to separators

- [x] **Step 5: Modify `src/hooks.rs`**
  - Colorize the following output locations:
    - Line 63: `">> Tool call: ..."` - use `color_debug()` for dim yellow/dark gray
    - Line 79: `">> Error: ..."` - use `color_error()` for bright red

- [ ] **Step 6: Verify `src/markdown.rs`**
  - Test that markdown rendering uses termimad's color scheme automatically
  - No code changes needed if termimad handles it

## Verification

- [x] Run `cargo make test` to ensure tests pass
- [x] Run `cargo make check-all` for lints and formatting
- [ ] Manual testing:
  - Start the REPL and verify prompt colors (numbers bright cyan, rest gray)
  - Send a query that triggers tool calls - verify they appear in dim color
  - Trigger an error - verify red coloring
  - Check loading/ready messages for appropriate colors
  - Verify markdown responses render with colors
  - Test Ctrl+D exit to see "Goodbye" message
