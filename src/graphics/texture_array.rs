//! A texture array.
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use gfx;
use gfx::format::{ChannelTyped, SurfaceTyped};
use gfx::memory::Bind;
use gfx::memory::Usage;
use gfx::traits::Factory;
use gfx_device_gl;
use image;

use context::Context;
use graphics::{self, BackendSpec, GlBackendSpec};
use GameError;
use GameResult;

type SurfaceType = <<GlBackendSpec as BackendSpec>::SurfaceType as gfx::format::Formatted>::Surface;
type ChannelType = <<GlBackendSpec as BackendSpec>::SurfaceType as gfx::format::Formatted>::Channel;

/// A texture array
#[derive(Debug, Clone)]
pub struct TextureArray {
    texture: gfx::handle::RawTexture<gfx_device_gl::Resources>,
    resource: gfx::handle::RawShaderResourceView<gfx_device_gl::Resources>,
    sampler_info: gfx::texture::SamplerInfo,
    x_unit: f32,
    y_unit: f32,
}

impl TextureArray {
    /// Obtain a batch for the texture array
    pub fn batch(&self) -> Batch {
        Batch::new(self.clone())
    }
}

/// Represents a batch of sprites that can be written with a texture array all at once.
#[derive(Debug)]
pub struct Batch {
    texture_array: TextureArray,
    sprites: Vec<graphics::InstanceProperties>,
}

impl Batch {
    fn new(texture_array: TextureArray) -> Batch {
        Batch {
            texture_array,
            sprites: Vec::new(),
        }
    }

    /// Add a sprite to the batch
    pub fn add(&mut self, index: &Index, sprite: Sprite, dest: graphics::Point2) {
        let mut properties = graphics::InstanceProperties::from(graphics::DrawParam {
            src: graphics::Rect {
                x: (index.offset.x + sprite.column * sprite.width) as f32
                    * self.texture_array.x_unit,
                y: (index.offset.y + sprite.row * sprite.height) as f32 * self.texture_array.y_unit,
                w: sprite.width as f32 * self.texture_array.x_unit,
                h: sprite.height as f32 * self.texture_array.y_unit,
            },
            dest,
            color: Some(graphics::types::WHITE),
            scale: graphics::Point2::new(sprite.width as f32, sprite.height as f32),
            ..Default::default()
        });

        properties.layer = index.layer.into();

        self.sprites.push(properties);
    }

    fn flush(&self, context: &mut Context) -> GameResult<()> {
        let gfx = &mut context.gfx_context;
        if gfx.data.rect_instance_properties.len() < self.sprites.len() {
            gfx.data.rect_instance_properties = gfx.factory.create_buffer(
                self.sprites.len(),
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::TRANSFER_DST,
            )?;
        }
        gfx.encoder
            .update_buffer(&gfx.data.rect_instance_properties, &self.sprites[..], 0)?;
        Ok(())
    }

    /// Draw the batch
    pub fn draw(&self, context: &mut Context, dest: graphics::Point2) -> GameResult<()> {
        let cell = Rc::clone(&context.gfx_context.current_shader);
        let previous_shader = *cell.borrow();

        *context.gfx_context.current_shader.borrow_mut() =
            Some(context.gfx_context.texture_array_shader);

        self.flush(context)?;

        {
            let gfx = &mut context.gfx_context;

            let sampler = gfx
                .samplers
                .get_or_insert(self.texture_array.sampler_info, gfx.factory.as_mut());
            gfx.data.vbuf = gfx.quad_vertex_buffer.clone();

            let typed_thingy =
                GlBackendSpec::raw_to_typed_shader_resource(self.texture_array.resource.clone());
            gfx.data.tex_array = (typed_thingy, sampler);
            let mut slice = gfx.quad_slice.clone();
            slice.instances = Some((self.sprites.len() as u32, 0));
            let curr_transform = gfx.get_transform();

            gfx.push_transform(
                graphics::DrawParam {
                    dest,
                    ..Default::default()
                }
                .into_matrix()
                    * curr_transform,
            );
            gfx.calculate_transform_matrix();
            gfx.update_globals()?;
            gfx.draw(Some(&slice))?;
            gfx.pop_transform();
            gfx.calculate_transform_matrix();
            gfx.update_globals()?;
        }

        *cell.borrow_mut() = previous_shader;
        Ok(())
    }
}

/// Represents a sprite
#[derive(Debug)]
pub struct Sprite {
    /// Sprite row
    pub row: u32,
    /// Sprite column
    pub column: u32,
    /// Sprite width
    pub width: u32,
    /// Sprite height
    pub height: u32,
}

/// A texture array builder
#[derive(Debug)]
pub struct Builder {
    width: u32,
    height: u32,
    layers: Vec<Texture>,
    current: Texture,
}

impl Builder {
    /// Create a new texture array builder
    pub fn new(width: u16, height: u16) -> Builder {
        Builder {
            width: width as u32,
            height: height as u32,
            layers: Vec::new(),
            current: Texture::new(width, height),
        }
    }

    /// Add a new texture.
    pub fn add<P: AsRef<Path>>(&mut self, context: &mut Context, path: P) -> GameResult<Index> {
        let img = {
            let mut buf = Vec::new();
            let mut reader = context.filesystem.open(path)?;
            reader.read_to_end(&mut buf)?;
            let rgba = image::load_from_memory(&buf)?.to_rgba();
            Arc::new(rgba)
        };

        if img.width() > self.width || img.height() > self.height {
            Err(GameError::ResourceLoadError(String::from(
                "Image is too big",
            )))
        } else {
            let offset = self.current.add(img.clone());

            match offset {
                Some(offset) => Ok(Index {
                    layer: self.layers.len() as u16,
                    offset,
                }),
                None => {
                    self.layers.push(self.current.clone());
                    self.current = Texture::new(self.width as u16, self.height as u16);

                    Ok(Index {
                        layer: self.layers.len() as u16,
                        offset: self.current.add(img).unwrap(),
                    })
                }
            }
        }
    }

    /// Build the texture array
    pub fn build(mut self, context: &mut Context) -> TextureArray {
        if !self.current.is_empty() {
            self.layers.push(self.current.clone());
            self.current = Texture::new(0, 0);
        }

        let images: Vec<Vec<u8>> = self
            .layers
            .iter()
            .map(|layer| layer.to_owned().to_rgba().into_raw())
            .collect();

        let raw_layers: Vec<&[u8]> = images.iter().map(|image| &image[..]).collect();

        let factory = &mut context.gfx_context.factory;
        let channel_type = ChannelType::get_channel_type();
        let surface_format = SurfaceType::get_surface_type();

        let texture = factory
            .create_texture_raw(
                gfx::texture::Info {
                    kind: gfx::texture::Kind::D2Array(
                        self.width as u16,
                        self.height as u16,
                        images.len() as u16,
                        gfx::texture::AaMode::Single,
                    ),
                    levels: 1,
                    format: surface_format,
                    bind: Bind::SHADER_RESOURCE | Bind::RENDER_TARGET | Bind::TRANSFER_SRC,
                    usage: Usage::Data,
                },
                Some(channel_type),
                Some((&raw_layers[..], gfx::texture::Mipmap::Provided)),
            )
            .unwrap();

        let resource_desc = gfx::texture::ResourceDesc {
            channel: channel_type,
            layer: None,
            min: 0,
            max: texture.get_info().levels - 1,
            swizzle: gfx::format::Swizzle::new(),
        };

        let resource = factory
            .view_texture_as_shader_resource_raw(&texture, resource_desc)
            .unwrap();

        TextureArray {
            texture,
            resource,
            sampler_info: context.gfx_context.default_sampler_info,
            x_unit: 1.0 / self.width as f32,
            y_unit: 1.0 / self.height as f32,
        }
    }
}

/// An index that identifies a texture in a texture array.
#[derive(Debug)]
pub struct Index {
    layer: u16,
    offset: Offset,
}

#[derive(Debug, Clone)]
struct Texture {
    images: Vec<Vec<Arc<image::RgbaImage>>>,
    current_row: Vec<Arc<image::RgbaImage>>,
    max_width: u32,
    max_height: u32,
}

impl Texture {
    fn new(max_width: u16, max_height: u16) -> Texture {
        Texture {
            images: Vec::new(),
            current_row: Vec::new(),
            max_width: max_width as u32,
            max_height: max_height as u32,
        }
    }

    fn current_height(&self) -> u32 {
        self.images
            .iter()
            .map(|row| row.iter().map(|i| i.height()).max().unwrap_or(0))
            .sum()
    }

    fn is_empty(&self) -> bool {
        self.images.is_empty() && self.current_row.is_empty()
    }

    fn add(&mut self, image: Arc<image::RgbaImage>) -> Option<Offset> {
        let current_row_width = self.current_row.iter().map(|i| i.width()).sum();

        if current_row_width + image.width() <= self.max_width {
            if self.current_height() + image.height() <= self.max_height {
                self.current_row.push(image);

                Some(Offset {
                    x: current_row_width,
                    y: self.current_height(),
                })
            } else {
                None
            }
        } else {
            let current_row_height = self
                .current_row
                .iter()
                .map(|i| i.height())
                .max()
                .unwrap_or(0);

            if self.current_height() + current_row_height + image.height() <= self.max_height {
                self.images.push(self.current_row.clone());
                self.current_row = vec![image];

                Some(Offset {
                    x: 0,
                    y: self.current_height(),
                })
            } else {
                None
            }
        }
    }

    fn to_rgba(mut self) -> image::RgbaImage {
        let mut values = Vec::new();
        values.resize((self.max_width * self.max_height * 4) as usize, 0 as u8);

        let mut texture =
            image::ImageBuffer::from_raw(self.max_width, self.max_height, values).unwrap();

        if !self.current_row.is_empty() {
            self.images.push(self.current_row.clone());
            self.current_row = Vec::new();
        }

        let mut y = 0;

        for row in self.images {
            let mut x = 0;
            let mut row_height = 0;

            for image in row {
                image::imageops::overlay(&mut texture, &image, x, y);

                x += image.width();
                row_height = row_height.max(image.height());
            }

            y += row_height;
        }

        texture
    }
}

#[derive(Debug)]
struct Offset {
    x: u32,
    y: u32,
}
