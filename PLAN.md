## Plan: Add Terminal Markdown Formatting to REPL

Add `termimad` to render all agent responses as markdown with default theme, no syntax highlighting. Falls back to plain text on rendering errors. This covers headings, code blocks, tables, bold/italic, lists, and other markdown elements.

### Steps

1. **Add `termimad` dependency** to [Cargo.toml](Cargo.toml) without syntax highlighting features (keeps binary lightweight)

2. **Create markdown module** at [src/markdown.rs](src/markdown.rs) with `render_markdown()` function using default `MadSkin`, falling back to plain text on error

3. **Update [lib.rs](src/lib.rs)** to declare the new `markdown` module as public

4. **Replace plain text output** in [main.rs](main.rs#L156) to call `markdown::render_markdown(&response)` instead of `println!`
