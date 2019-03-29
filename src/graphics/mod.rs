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
use crate::conf;
use crate::{Context, GameResult};
pub use canvas::{set_canvas, Canvas};
pub use color::Color;
pub use drawparam::DrawParam;
pub use font::Font;
pub(crate) use gpu::Gpu;
pub use math::{Matrix4, Point2, Rect, Vector2};
pub use mesh::Mesh;
pub use text::Text;

pub struct Encoder {
    encoder: wgpu::CommandEncoder,
}

impl Encoder {
    pub fn finish(self) -> wgpu::CommandBuffer {
        self.encoder.finish()
    }
}

impl Encoder {
    pub fn render_pass(&mut self, frame: &gpu::TextureView, context: &Context) -> RenderPass {
        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&context.gpu.pipeline);

        RenderPass { render_pass }
    }
}

pub struct RenderPass<'a> {
    render_pass: wgpu::RenderPass<'a>,
}

pub fn encoder(context: &mut Context) -> Encoder {
    let encoder = context
        .gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

    Encoder { encoder }
}

pub fn clear(context: &mut Context) {}

pub fn present(context: &mut Context) {
    let mut encoder_1 = encoder(context);
    let mut encoder_2 = encoder(context);

    context
        .gpu
        .device
        .get_queue()
        .submit(&[encoder_1.finish(), encoder_2.finish()]);

    use std::{thread, time};

    let second = time::Duration::from_millis(16);

    thread::sleep(second);
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
