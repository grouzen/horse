use indicatif::{ProgressBar, ProgressStyle};

/// Creates a braille-pattern spinner with a custom message.
///
/// The spinner uses Unicode braille characters (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) to create a smooth
/// rotating animation effect in the terminal.
///
/// # Arguments
///
/// * `message` - The message to display next to the spinner (e.g., "Processing")
///
/// # Returns
///
/// A `ProgressBar` handle that can be used to control the spinner lifecycle.
/// Call `.finish_and_clear()` on the spinner to cleanly remove it from the terminal.
///
/// # Example
///
/// ```no_run
/// use horse::console::spinner::create_spinner;
///
/// let spinner = create_spinner("Processing");
/// // ... do some work ...
/// spinner.finish_and_clear();
/// ```
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();

    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("Failed to set spinner template"),
    );

    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    spinner
}
