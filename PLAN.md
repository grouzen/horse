# Plan: Port ASCII Art to Horse Banner

**Summary:** Extract the minimal subset of ascii-art-rs needed to render text (starting with "HORSE") using the matrix palette with filled horizontal gradient. Create a new `logo.rs` module in `src/console/` that handles color gradients, block font rendering, and ANSI output in a single file. Display the banner when the REPL starts in `src/main.rs`. The implementation will be simplified but flexible enough to render any text string.

## Steps

1. **Add dependencies** to `horse/Cargo.toml`
   - `lazy_static = "1.4"` (for static palette/font data)
   - `regex = "1.10"` (for stripping color tags from font JSON)
   - Note: `serde`, `serde_json`, `anyhow` already present

2. **Create font data file** at `horse/src/console/block_font.json`
   - Copy entire `ascii-art-rs/src/fonts/block.json` file (483 lines)
   - Contains character definitions: uppercase A-Z, numbers, symbols
   - Each character has 6 lines with `<c1>` and `<c2>` color tags

3. **Create logo module** at `horse/src/console/logo.rs`
   - **RgbColor struct** with `r`, `g`, `b` fields
   - `from_hex()` parser for `#RRGGBB` format
   - `to_ansi_fg()` method for 24-bit ANSI codes `\x1b[38;2;{r};{g};{b}m`
   - `interpolate()` method for smooth color transitions
   - **Gradient functions:**
     - `create_gradient(colors: &[RgbColor], steps: usize) -> Vec<RgbColor>` - interpolates between colors
     - `apply_horizontal_gradient(lines: Vec<String>, colors: &[RgbColor]) -> String` - applies per-character coloring
   - **Font loading:**
     - `CFontData` struct with `lines: u8`, `chars: HashMap<String, Vec<String>>`
     - Use `lazy_static!` for `BLOCK_FONT: CFontData` loaded via `include_str!("block_font.json")`
   - **Matrix palette:**
     - Use `lazy_static!` for `MATRIX_PALETTE: Vec<RgbColor>` with `["#00ff41", "#008f11"]`
   - **Rendering logic:**
     - `render_uncolored(text: &str, letter_spacing: usize) -> Result<Vec<String>>` - builds character array, strips color tags with regex
     - `pub fn render_logo(text: &str) -> Result<String>` - main API that combines rendering + gradient application

4. **Update console module** in `horse/src/console.rs`
   - Add `pub mod logo;` declaration
   - Keep existing: `colors`, `markdown`, `repl`, `spinner`

5. **Integrate banner into startup** in `horse/src/main.rs`
   - After provider/agent initialization (around line 87-93)
   - Call `console::logo::render_logo("HORSE")?`
   - Print to stdout with extra newline: `println!("{}\n", logo);`
   - Place before first REPL prompt

## Verification

- **Build**: `cd horse && cargo build --release`
- **Run**: `./target/release/horse`
- **Expected**: On startup, see "HORSE" in large block letters with horizontal gradient from bright matrix green (left) to dark green (right)
- **Test flexibility**: Modify code to render different text (e.g., "HELLO") and verify it works
- **Color check**: Ensure terminal supports 24-bit color (most modern terminals do)

## Decisions

- **Single file module**: All logo functionality in `src/console/logo.rs` for simplicity
- **Minimal dependencies**: Only added `lazy_static` and `regex`; reused existing Serde
- **Single font**: Block font only to minimize code
- **Horizontal only**: No vertical/diagonal gradients to keep scope tight
- **No shadows**: Omitted shadow effects from filled renderer
- **Static palette**: Matrix hardcoded but structure allows easy extension
- **Flexible text**: Can render any ASCII text via `render_logo(text)`
