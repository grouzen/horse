# TODO: Add Terminal Markdown Formatting to REPL

- [ ] Add `termimad` dependency to Cargo.toml without syntax highlighting features
- [ ] Create markdown module at src/markdown.rs with `render_markdown()` function using default `MadSkin`, falling back to plain text on error
- [ ] Update lib.rs to declare the new `markdown` module as public
- [ ] Replace plain text output in main.rs (line 156) to call `markdown::render_markdown(&response)` instead of `println!`
