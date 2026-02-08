use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RgbColor {
    /// Create a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create RGB color from hex string (e.g., "#ff0000" or "ff0000")
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(anyhow!("Invalid hex color length: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| anyhow!("Invalid hex color: {}", hex))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| anyhow!("Invalid hex color: {}", hex))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| anyhow!("Invalid hex color: {}", hex))?;

        Ok(Self::new(r, g, b))
    }

    /// Generate ANSI escape sequence for foreground color (24-bit color)
    pub fn to_ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Interpolate between two colors
    pub fn interpolate(&self, other: &RgbColor, t: f32) -> RgbColor {
        let t = t.clamp(0.0, 1.0);
        let r = (self.r as f32 * (1.0 - t) + other.r as f32 * t) as u8;
        let g = (self.g as f32 * (1.0 - t) + other.g as f32 * t) as u8;
        let b = (self.b as f32 * (1.0 - t) + other.b as f32 * t) as u8;
        RgbColor::new(r, g, b)
    }
}

/// Create a gradient with the specified number of steps
fn create_gradient(colors: &[RgbColor], steps: usize) -> Vec<RgbColor> {
    if colors.is_empty() {
        return vec![];
    }

    if colors.len() == 1 {
        return vec![colors[0]; steps];
    }

    let mut gradient = Vec::with_capacity(steps);
    let segments = colors.len() - 1;
    let steps_per_segment = (steps - 1) as f32 / segments as f32;

    for i in 0..steps {
        let position = i as f32;
        let segment = (position / steps_per_segment) as usize;
        let segment = segment.min(segments - 1);

        let local_t = (position - segment as f32 * steps_per_segment) / steps_per_segment;
        let color = colors[segment].interpolate(&colors[segment + 1], local_t);
        gradient.push(color);
    }

    gradient
}

/// Apply horizontal gradient (left to right on each line)
fn apply_horizontal_gradient(lines: Vec<String>, colors: &[RgbColor]) -> String {
    let mut result = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            result.push(line);
        } else {
            let chars: Vec<char> = line.chars().collect();
            let gradient = create_gradient(colors, chars.len());
            let mut colored_line = String::new();

            for (i, ch) in chars.iter().enumerate() {
                if ch.is_whitespace() {
                    colored_line.push(*ch);
                } else {
                    let color = gradient.get(i).unwrap_or(&colors[0]);
                    colored_line.push_str(&format!("{}{}", (*color).to_ansi_fg(), ch));
                }
            }
            colored_line.push_str("\x1b[0m");
            result.push(colored_line);
        }
    }

    result.join("\n")
}

/// CFonts font definition structure
#[derive(Debug, Clone, Deserialize)]
struct CFontData {
    lines: u8,
    letterspace: Vec<String>,
    chars: HashMap<String, Vec<String>>,
}

impl CFontData {
    /// Get character definition for a given character
    pub fn get_char(&self, ch: char) -> Option<&Vec<String>> {
        // Try uppercase first (most common)
        if let Some(lines) = self.chars.get(&ch.to_uppercase().to_string()) {
            return Some(lines);
        }

        // Try the character as-is
        if let Some(lines) = self.chars.get(&ch.to_string()) {
            return Some(lines);
        }

        // Try space for unknown characters
        self.chars.get(" ")
    }
}

// Load the block font data at compile time
lazy_static! {
    static ref BLOCK_FONT: CFontData = {
        let json_data = include_str!("block_font.json");
        serde_json::from_str(json_data).expect("Failed to parse block_font.json")
    };
}

// Define the Matrix palette
lazy_static! {
    static ref MATRIX_PALETTE: Vec<RgbColor> = {
        vec![
            RgbColor::from_hex("#00ff41").expect("Failed to parse matrix green"),
            RgbColor::from_hex("#008f11").expect("Failed to parse matrix dark green"),
        ]
    };
}

/// Render text without colors (but with font structure)
fn render_uncolored(text: &str, letter_spacing: usize) -> Result<Vec<String>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Ok(vec![String::new(); BLOCK_FONT.lines as usize]);
    }

    // Get character data for each character
    let mut char_data = Vec::new();
    for &ch in &chars {
        if let Some(lines) = BLOCK_FONT.get_char(ch) {
            char_data.push(lines.clone());
        } else {
            // Use space for unknown characters
            if let Some(space_lines) = BLOCK_FONT.get_char(' ') {
                char_data.push(space_lines.clone());
            } else {
                // Fallback: create empty lines
                char_data.push(vec![String::new(); BLOCK_FONT.lines as usize]);
            }
        }
    }

    // Combine characters horizontally
    let mut result_lines = vec![String::new(); BLOCK_FONT.lines as usize];

    for (char_idx, char_lines) in char_data.iter().enumerate() {
        // Add letter spacing (except before first character)
        if char_idx > 0 {
            for (line_idx, result_line) in result_lines.iter_mut().enumerate() {
                if line_idx < BLOCK_FONT.letterspace.len() {
                    result_line.push_str(&BLOCK_FONT.letterspace[line_idx].repeat(letter_spacing));
                } else {
                    result_line.push_str(&" ".repeat(letter_spacing));
                }
            }
        }

        // Add character lines
        for (line_idx, char_line) in char_lines.iter().enumerate() {
            if line_idx < result_lines.len() {
                result_lines[line_idx].push_str(char_line);
            }
        }
    }

    // Remove color tags from the result since we'll apply gradient later
    let re = Regex::new(r"<c\d+>(.*?)</c\d+>").unwrap();
    let clean_lines = result_lines
        .iter()
        .map(|line| re.replace_all(line, "$1").to_string())
        .collect();

    Ok(clean_lines)
}

/// Render logo text with horizontal gradient
///
/// # Arguments
/// * `text` - The text to render (e.g., "HORSE")
///
/// # Returns
/// A string containing the rendered text with ANSI color codes
///
/// # Example
/// ```
/// use horse::console::logo::render_logo;
/// # fn main() -> anyhow::Result<()> {
/// let logo = render_logo("HORSE")?;
/// println!("{}", logo);
/// # Ok(())
/// # }
/// ```
pub fn render_logo(text: &str) -> Result<String> {
    if text.trim().is_empty() {
        return Err(anyhow!("Text cannot be empty"));
    }

    // Render the text structure without colors
    let uncolored_lines = render_uncolored(text, 1)?;

    // Apply horizontal gradient using the Matrix palette
    let colored_text = apply_horizontal_gradient(uncolored_lines, &MATRIX_PALETTE);

    Ok(colored_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_color_from_hex() {
        let color = RgbColor::from_hex("#00ff41").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 65);
    }

    #[test]
    fn test_rgb_color_interpolate() {
        let color1 = RgbColor::new(0, 0, 0);
        let color2 = RgbColor::new(255, 255, 255);
        let mid = color1.interpolate(&color2, 0.5);
        assert_eq!(mid.r, 127);
        assert_eq!(mid.g, 127);
        assert_eq!(mid.b, 127);
    }

    #[test]
    fn test_create_gradient() {
        let colors = vec![RgbColor::new(0, 0, 0), RgbColor::new(255, 255, 255)];
        let gradient = create_gradient(&colors, 3);
        assert_eq!(gradient.len(), 3);
    }

    #[test]
    fn test_render_logo() {
        let result = render_logo("TEST");
        assert!(result.is_ok());
        let logo = result.unwrap();
        assert!(!logo.is_empty());
        assert!(logo.contains("\x1b[")); // Should contain ANSI codes
    }

    #[test]
    fn test_render_logo_empty() {
        let result = render_logo("");
        assert!(result.is_err());
    }
}
