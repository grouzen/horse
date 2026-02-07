use owo_colors::OwoColorize;

/// Format prompt numbers and token counts in cyan/blue
pub fn color_prompt_number(text: impl std::fmt::Display) -> String {
    format!("{}", text.cyan())
}

/// Format debug messages (e.g., tool calls) in dark gray
pub fn color_debug(text: impl std::fmt::Display) -> String {
    format!("{}", text.bright_black())
}

/// Format error messages in bright red
pub fn color_error(text: impl std::fmt::Display) -> String {
    format!("{}", text.bright_red())
}

/// Format warning messages in dim magenta
pub fn color_warning(text: impl std::fmt::Display) -> String {
    format!("{}", text.magenta().dimmed())
}

/// Format success messages in bright green
pub fn color_success(text: impl std::fmt::Display) -> String {
    format!("{}", text.bright_green())
}

/// Format status messages (loading/ready) in dim green
pub fn color_status(text: impl std::fmt::Display) -> String {
    format!("{}", text.green().dimmed())
}

/// Format dim text (e.g., normal text in prompt) in gray
pub fn color_dim(text: impl std::fmt::Display) -> String {
    format!("{}", text.bright_black())
}
