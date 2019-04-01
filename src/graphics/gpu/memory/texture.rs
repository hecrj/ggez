use core::mem::ManuallyDrop;
use core::ops::Drop;
use std::mem::size_of;

use gfx_hal::{self as hal, Device, PhysicalDevice};
use image;

use crate::graphics::gpu::backend::Backend;
use crate::graphics::gpu::memory::buffer::Buffer;
use crate::graphics::gpu::memory::chunk::Chunk;
use crate::graphics::gpu::worker::Worker;

/// A texture loaded on the GPU
pub struct Texture {
    image: ManuallyDrop<<Backend as hal::Backend>::Image>,
    image_view: ManuallyDrop<<Backend as hal::Backend>::ImageView>,
    chunk: ManuallyDrop<Chunk>,
}

impl Drop for Texture {
    fn drop(&mut self) {
        let device = &self.chunk.context().device;
        let _ = device.wait_idle();

        unsafe {
            use core::ptr::read;
            device.destroy_image(ManuallyDrop::into_inner(read(&mut self.image)));
            device.destroy_image_view(ManuallyDrop::into_inner(read(&mut self.image_view)));
            ManuallyDrop::drop(&mut self.chunk);
        }
    }
}

impl Texture {
    fn new(worker: &mut Worker, image: &image::DynamicImage) -> Result<Texture, &'static str> {
        let context = worker.context();
        let device = &context.device;

        // Convert to BGRA
        let image_data = image.to_bgra();

        // Create image
        let image = unsafe {
            context
                .device
                .create_image(
                    hal::image::Kind::D2(image_data.width(), image_data.height(), 1, 1),
                    1,
                    hal::format::Format::Bgra8Srgb,
                    hal::image::Tiling::Optimal,
                    hal::image::Usage::TRANSFER_DST | gfx_hal::image::Usage::SAMPLED,
                    hal::image::ViewCapabilities::empty(),
                )
                .map_err(|_| "Couldn't create the image!")?
        };

        // Create view
        let view = unsafe {
            context
                .device
                .create_image_view(
                    &image,
                    hal::image::ViewKind::D2,
                    hal::format::Format::Bgra8Srgb,
                    hal::format::Swizzle::NO,
                    hal::image::SubresourceRange {
                        aspects: hal::format::Aspects::COLOR,
                        levels: 0..1,
                        layers: 0..1,
                    },
                )
                .map_err(|_| "Couldn't create the image view!")?
        };

        // Calculate alignment
        let pixel_size = size_of::<image::Bgra<u8>>();
        let row_size = pixel_size * (image_data.width() as usize);
        let limits = context.adapter.physical_device.limits();
        let row_alignment_mask = limits.min_buffer_copy_pitch_alignment as u32 - 1;
        let row_pitch = ((row_size as u32 + row_alignment_mask) & !row_alignment_mask) as usize;
        debug_assert!(row_pitch as usize >= row_size);

        info!(
            "Loading image of {}x{} (row_pitch: {})",
            image_data.width(),
            image_data.height(),
            row_pitch
        );

        // Write image data to a staging buffer
        let required_bytes = row_pitch * image_data.height() as usize;
        let mut staging_buffer = Buffer::new(
            worker.context(),
            required_bytes,
            hal::buffer::Usage::TRANSFER_SRC,
        )?;

        staging_buffer.write(|writer| {
            for y in 0..image_data.height() as usize {
                let row = &(*image_data)[y * row_size..(y + 1) * row_size];
                let dest_base = y * row_pitch;
                writer[dest_base..dest_base + row.len()].copy_from_slice(row);
            }
        })?;

        // Prepare image memory
        let requirements = unsafe { device.get_image_requirements(&image) };
        let chunk = Chunk::new(context, requirements, hal::memory::Properties::DEVICE_LOCAL)?;

        worker.perform(|command_buffer| unsafe {
            command_buffer.pipeline_barrier(
                hal::pso::PipelineStage::TOP_OF_PIPE..hal::pso::PipelineStage::TRANSFER,
                hal::memory::Dependencies::empty(),
                &[hal::memory::Barrier::Image {
                    states: (hal::image::Access::empty(), hal::image::Layout::Undefined)
                        ..(
                            hal::image::Access::TRANSFER_WRITE,
                            hal::image::Layout::TransferDstOptimal,
                        ),
                    target: &image,
                    families: None,
                    range: hal::image::SubresourceRange {
                        aspects: hal::format::Aspects::COLOR,
                        levels: 0..1,
                        layers: 0..1,
                    },
                }],
            );

            command_buffer.copy_buffer_to_image(
                staging_buffer.buffer(),
                &image,
                hal::image::Layout::TransferDstOptimal,
                &[gfx_hal::command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: (row_pitch / pixel_size) as u32,
                    buffer_height: image_data.height(),
                    image_layers: gfx_hal::image::SubresourceLayers {
                        aspects: hal::format::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: gfx_hal::image::Offset { x: 0, y: 0, z: 0 },
                    image_extent: gfx_hal::image::Extent {
                        width: image_data.width(),
                        height: image_data.height(),
                        depth: 1,
                    },
                }],
            );

            command_buffer.pipeline_barrier(
                hal::pso::PipelineStage::TRANSFER..hal::pso::PipelineStage::FRAGMENT_SHADER,
                gfx_hal::memory::Dependencies::empty(),
                &[gfx_hal::memory::Barrier::Image {
                    states: (
                        gfx_hal::image::Access::TRANSFER_WRITE,
                        hal::image::Layout::TransferDstOptimal,
                    )
                        ..(
                            gfx_hal::image::Access::SHADER_READ,
                            hal::image::Layout::ShaderReadOnlyOptimal,
                        ),
                    target: &image,
                    families: None,
                    range: hal::image::SubresourceRange {
                        aspects: hal::format::Aspects::COLOR,
                        levels: 0..1,
                        layers: 0..1,
                    },
                }],
            );
        })?;

        Ok(Texture {
            image: ManuallyDrop::new(image),
            image_view: ManuallyDrop::new(view),
            chunk: ManuallyDrop::new(chunk),
        })
    }
}
