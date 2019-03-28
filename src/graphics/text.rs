//! Text

/// Font
#[derive(Debug, Clone, Copy)]
pub struct Font {}

impl Font {
    /// Gets the default font
    pub fn default_font() -> Option<Font> {
        Some(Font {})
    }
}
