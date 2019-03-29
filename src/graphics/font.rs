use crate::GameResult;
pub struct Font {}

impl Font {
    pub fn default_font() -> GameResult<Font> {
        Ok(Font {})
    }
}
