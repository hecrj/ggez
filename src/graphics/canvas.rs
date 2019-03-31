use crate::conf;
use crate::graphics::Drawable;
use crate::{Context, GameResult};

pub struct Canvas {}

impl Canvas {
    pub fn new(
        context: &mut Context,
        width: u32,
        height: u32,
        samples: conf::NumSamples,
    ) -> GameResult<Canvas> {
        Ok(Canvas {})
    }
}

impl Drawable for Canvas {}

pub fn set_canvas(context: &mut Context, canvas: Option<&Canvas>) {}
