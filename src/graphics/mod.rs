pub mod canvas;
mod color;
mod drawparam;
pub mod font;
mod gpu;
pub mod image;
mod math;
pub mod mesh;
pub mod spritebatch;
pub mod text;
pub mod texture_array;
pub mod window;

pub use self::image::Image;
use crate::{Context, GameResult};
pub use canvas::{set_canvas, Canvas};
pub use color::Color;
pub use drawparam::DrawParam;
pub use font::Font;
pub(crate) use gpu::Gpu;
pub use math::{Matrix4, Point2, Rect, Vector2};
pub use mesh::Mesh;
pub use text::Text;
pub use window::Window;

pub type Frame<'a> = gpu::Frame<'a>;
pub type Target<'a> = gpu::Target<'a>;
pub type Painter<'a> = gpu::Painter<'a>;

pub fn painter(context: &Context) -> Painter {
    context.gpu.new_painter()
}

pub fn clear(painter: &mut Painter, target: &Target, color: &Color) {
    painter.clear(target, color);
}

pub fn submit(context: &mut Context, commands: &[gpu::Commands]) {
    context.gpu.submit(commands);
}

pub enum FilterMode {
    Nearest,
}

pub fn set_default_filter(context: &mut Context, filter: FilterMode) {}

/// Screen stuff
pub fn get_size(context: &Context) -> (u32, u32) {
    (1280, 1024)
}

pub fn get_screen_coordinates(context: &Context) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: 1280.0,
        h: 1024.0,
    }
}

pub fn set_screen_coordinates(context: &mut Context, rect: Rect) -> GameResult<()> {
    Ok(())
}

/// Transform stack
pub fn push_transform(context: &mut Context, matrix: Option<Matrix4>) {}

pub fn pop_transform(context: &mut Context) {}

pub fn apply_transformations(context: &mut Context) -> GameResult<()> {
    Ok(())
}

/// Drawable stuff
pub enum DrawMode {
    Fill,
}

pub trait Drawable {}

pub fn draw(
    context: &mut Context,
    drawable: &Drawable,
    dest: Point2,
    rotation: f32,
) -> GameResult<()> {
    draw_ex(
        context,
        drawable,
        DrawParam {
            dest,
            rotation,
            ..Default::default()
        },
    )
}

pub fn draw_ex(context: &mut Context, drawable: &Drawable, param: DrawParam) -> GameResult<()> {
    Ok(())
}
