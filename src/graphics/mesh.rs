use crate::graphics::{DrawMode, Drawable, Point2};
use crate::{Context, GameResult};

pub struct Mesh {}

impl Mesh {
    pub fn new_polygon(
        context: &mut Context,
        mode: DrawMode,
        vertices: &[Point2],
    ) -> GameResult<Mesh> {
        Ok(Mesh {})
    }
}

impl Drawable for Mesh {}
