use crate::graphics::{DrawParam, Drawable, Image};
use crate::Context;

pub struct SpriteBatch {}

impl SpriteBatch {
    pub fn new(image: Image) -> SpriteBatch {
        SpriteBatch {}
    }

    pub fn add(&mut self, param: DrawParam) {}
}

impl Drawable for SpriteBatch {}
