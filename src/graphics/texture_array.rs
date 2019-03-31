use crate::graphics::Point2;
use crate::{Context, GameResult};
use std::path::Path;

pub struct TextureArray {}

impl TextureArray {
    pub fn batch(&self) -> Batch {
        Batch {}
    }
}

pub struct Batch {}

impl Batch {
    pub fn add(&mut self, index: &Index, sprite: Sprite, dest: Point2) {}

    pub fn draw(&self, context: &mut Context, dest: Point2) -> GameResult<()> {
        Ok(())
    }
}

pub struct Sprite {
    pub row: u32,
    pub column: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Index {}

pub struct Builder {}

impl Builder {
    pub fn new(width: u32, height: u32) -> Builder {
        Builder {}
    }

    pub fn add<P: AsRef<Path>>(&mut self, context: &mut Context, path: P) -> GameResult<Index> {
        Ok(Index {})
    }

    pub fn build(self, context: &mut Context) -> TextureArray {
        TextureArray {}
    }
}
