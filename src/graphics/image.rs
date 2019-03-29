use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use crate::graphics::gpu;
use crate::graphics::{Color, Drawable};
use crate::{Context, GameResult};

#[derive(Clone)]
pub struct Image {
    width: u16,
    height: u16,
    texture: Arc<gpu::Texture>,
    texture_view: Arc<gpu::TextureView>,
}

impl Image {
    pub fn new<P: AsRef<Path>>(context: &mut Context, path: P) -> GameResult<Image> {
        let image = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            let _ = reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_bgra()
        };

        let (texture, texture_view) = context.gpu.upload_image(&image);

        Ok(Image {
            width: image.width() as u16,
            height: image.height() as u16,
            texture: Arc::new(texture),
            texture_view: Arc::new(texture_view),
        })
    }

    pub fn solid(context: &mut Context, size: u16, color: Color) -> GameResult<Image> {
        let mut values = Vec::new();
        let (r, g, b, a) = color.to_rgba();

        for _ in 0..size {
            for _ in 0..size {
                values.push(b);
                values.push(g);
                values.push(r);
                values.push(a);
            }
        }

        let image = image::ImageBuffer::from_raw(size as u32, size as u32, values).unwrap();
        let (texture, texture_view) = context.gpu.upload_image(&image);

        Ok(Image {
            width: size,
            height: size,
            texture: Arc::new(texture),
            texture_view: Arc::new(texture_view),
        })
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

impl Drawable for Image {}
