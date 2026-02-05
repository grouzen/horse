use termimad::MadSkin;

/// Renders markdown text to the terminal using termimad's default theme.
pub fn render_markdown(text: &str) {
    let skin = MadSkin::default();
    let rendered = skin.term_text(text);
    println!("\n{}\n", rendered);
}
