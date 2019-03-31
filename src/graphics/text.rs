//! Text
use crate::graphics::{Drawable, Font};
use crate::{Context, GameResult};

/// Font
#[derive(Debug, Clone, Copy)]
pub struct Text {}

impl Text {
    /// Gets the default font
    pub fn new(context: &mut Context, text: &str, font: &Font) -> GameResult<Text> {
        Ok(Text {})
    }
}

impl Drawable for Text {}
