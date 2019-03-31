#![allow(unsafe_code)]
use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use core::ptr::read;
use gfx_hal::{self as hal, Device, Surface, Swapchain as _};

use crate::graphics::color::Color;
use crate::graphics::gpu::backend::{self, Backend};

pub struct Swapchain {
    swapchain: ManuallyDrop<<Backend as hal::Backend>::Swapchain>,
    image_views: Vec<<Backend as hal::Backend>::ImageView>,
    framebuffers: Vec<<Backend as hal::Backend>::Framebuffer>,
    render_pass: ManuallyDrop<<Backend as hal::Backend>::RenderPass>,
    render_area: hal::pso::Rect,
    command_pool: ManuallyDrop<hal::pool::CommandPool<Backend, hal::Graphics>>,
    command_buffers: Vec<
        hal::command::CommandBuffer<
            Backend,
            hal::Graphics,
            hal::command::MultiShot,
            hal::command::Primary,
        >,
    >,
    image_available_semaphores: Vec<<Backend as hal::Backend>::Semaphore>,
    render_finished_semaphores: Vec<<Backend as hal::Backend>::Semaphore>,
    in_flight_fences: Vec<<Backend as hal::Backend>::Fence>,
    frames_in_flight: usize,
    current_frame: usize,
}

impl Swapchain {
    pub fn new(
        device: &backend::Device,
        surface: &mut <Backend as hal::Backend>::Surface,
        adapter: &hal::Adapter<Backend>,
        queue_group: &hal::QueueGroup<Backend, hal::Graphics>,
    ) -> Result<Swapchain, &'static str> {
        let (swapchain, extent, backbuffer, format, frames_in_flight) = {
            let (caps, preferred_formats, present_modes, composite_alphas) =
                surface.compatibility(&adapter.physical_device);

            info!("{:?}", caps);
            info!("Preferred Formats: {:?}", preferred_formats);
            info!("Present Modes: {:?}", present_modes);
            info!("Composite Alphas: {:?}", composite_alphas);

            let present_mode = {
                use hal::window::PresentMode::*;
                [Mailbox, Fifo, Relaxed, Immediate]
                    .iter()
                    .cloned()
                    .find(|pm| present_modes.contains(pm))
                    .ok_or("No PresentMode values specified!")?
            };

            let composite_alpha = {
                use hal::window::CompositeAlpha::*;
                [Opaque, Inherit, PreMultiplied, PostMultiplied]
                    .iter()
                    .cloned()
                    .find(|ca| composite_alphas.contains(ca))
                    .ok_or("No CompositeAlpha values specified!")?
            };

            let format = match preferred_formats {
                None => hal::format::Format::Rgba8Srgb,
                Some(formats) => match formats
                    .iter()
                    .find(|format| format.base_format().1 == hal::format::ChannelType::Unorm)
                    .cloned()
                {
                    Some(srgb_format) => srgb_format,
                    None => formats
                        .get(0)
                        .cloned()
                        .ok_or("Preferred format list was empty!")?,
                },
            };

            let extent = caps.extents.end;

            let image_count = if present_mode == hal::window::PresentMode::Mailbox {
                (caps.image_count.end - 1).min(3)
            } else {
                (caps.image_count.end - 1).min(2)
            };

            let image_layers = 1;
            let image_usage = if caps.usage.contains(hal::image::Usage::COLOR_ATTACHMENT) {
                hal::image::Usage::COLOR_ATTACHMENT
            } else {
                Err("The Surface isn't capable of supporting color!")?
            };

            let swapchain_config = hal::window::SwapchainConfig {
                present_mode,
                composite_alpha,
                format,
                extent,
                image_count,
                image_layers,
                image_usage,
            };

            info!("{:?}", swapchain_config);

            let (swapchain, backbuffer) = unsafe {
                device
                    .create_swapchain(surface, swapchain_config, None)
                    .map_err(|_| "Failed to create the swapchain!")?
            };

            (swapchain, extent, backbuffer, format, image_count as usize)
        };

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = {
            let mut image_available_semaphores: Vec<<backend::Backend as hal::Backend>::Semaphore> =
                vec![];
            let mut render_finished_semaphores: Vec<<backend::Backend as hal::Backend>::Semaphore> =
                vec![];
            let mut in_flight_fences: Vec<<backend::Backend as hal::Backend>::Fence> = vec![];

            for _ in 0..frames_in_flight {
                in_flight_fences.push(
                    device
                        .create_fence(true)
                        .map_err(|_| "Could not create a fence!")?,
                );

                image_available_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );

                render_finished_semaphores.push(
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")?,
                );
            }

            (
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
            )
        };

        let image_views: Vec<_> = match backbuffer {
            hal::Backbuffer::Images(images) => images
                .into_iter()
                .map(|image| unsafe {
                    device
                        .create_image_view(
                            &image,
                            hal::image::ViewKind::D2,
                            format,
                            hal::format::Swizzle::NO,
                            hal::image::SubresourceRange {
                                aspects: hal::format::Aspects::COLOR,
                                levels: 0..1,
                                layers: 0..1,
                            },
                        )
                        .map_err(|_| "Couldn't create the image_view for the image!")
                })
                .collect::<Result<Vec<_>, &str>>()?,
            hal::Backbuffer::Framebuffer(_) => {
                unimplemented!("Can't handle framebuffer backbuffer!")
            }
        };

        let render_pass = {
            let color_attachment = hal::pass::Attachment {
                format: Some(format),
                samples: 1,
                ops: hal::pass::AttachmentOps {
                    load: hal::pass::AttachmentLoadOp::Clear,
                    store: hal::pass::AttachmentStoreOp::Store,
                },
                stencil_ops: hal::pass::AttachmentOps::DONT_CARE,
                layouts: hal::image::Layout::Undefined..hal::image::Layout::Present,
            };

            let subpass = hal::pass::SubpassDesc {
                colors: &[(0, hal::image::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            unsafe {
                device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .map_err(|_| "Couldn't create a render pass!")?
            }
        };

        let framebuffers: Vec<<backend::Backend as hal::Backend>::Framebuffer> = {
            image_views
                .iter()
                .map(|image_view| unsafe {
                    device
                        .create_framebuffer(
                            &render_pass,
                            vec![image_view],
                            hal::image::Extent {
                                width: extent.width as u32,
                                height: extent.height as u32,
                                depth: 1,
                            },
                        )
                        .map_err(|_| "Failed to create a framebuffer!")
                })
                .collect::<Result<Vec<_>, &str>>()?
        };

        let mut command_pool = unsafe {
            device
                .create_command_pool_typed(
                    &queue_group,
                    hal::pool::CommandPoolCreateFlags::RESET_INDIVIDUAL,
                )
                .map_err(|_| "Could not create the raw command pool!")?
        };

        let command_buffers: Vec<_> = framebuffers
            .iter()
            .map(|_| command_pool.acquire_command_buffer())
            .collect();

        let mut swapchain = Swapchain {
            swapchain: ManuallyDrop::new(swapchain),
            render_pass: ManuallyDrop::new(render_pass),
            render_area: extent.to_extent().rect(),
            image_views,
            framebuffers,
            command_pool: ManuallyDrop::new(command_pool),
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            frames_in_flight,
            current_frame: 0,
        };

        swapchain.prepare_current_frame(device)?;

        Ok(swapchain)
    }

    fn prepare_current_frame(&mut self, device: &backend::Device) -> Result<(), &'static str> {
        let command_buffer = &mut self.command_buffers[self.current_frame];
        let flight_fence = &self.in_flight_fences[self.current_frame];

        unsafe {
            let _ = device
                .wait_for_fence(flight_fence, core::u64::MAX)
                .map_err(|_| "Failed to wait on the fence!")?;

            device
                .reset_fence(flight_fence)
                .map_err(|_| "Couldn't reset the fence!")?;

            command_buffer.begin(false);
        }

        Ok(())
    }

    pub fn clear(&mut self, color: &Color) {
        let command_buffer = &mut self.command_buffers[self.current_frame];
        let framebuffer = &self.framebuffers[self.current_frame];

        let clear_values = [hal::command::ClearValue::Color(
            hal::command::ClearColor::Float([color.r, color.g, color.b, color.a]),
        )];

        unsafe {
            let _ = command_buffer.begin_render_pass_inline(
                &self.render_pass,
                framebuffer,
                self.render_area,
                clear_values.iter(),
            );
        }
    }

    pub fn present(
        &mut self,
        device: &backend::Device,
        queue_group: &mut hal::QueueGroup<Backend, hal::Graphics>,
    ) -> Result<(), &'static str> {
        unsafe {
            self.command_buffers[self.current_frame].finish();
        }

        let image_available = &self.image_available_semaphores[self.current_frame];
        let render_finished = &self.render_finished_semaphores[self.current_frame];
        let flight_fence = &self.in_flight_fences[self.current_frame];
        let command_buffers = &self.command_buffers[self.current_frame..=self.current_frame];

        let index = unsafe {
            self.swapchain
                .acquire_image(
                    core::u64::MAX,
                    hal::window::FrameSync::Semaphore(image_available),
                )
                .unwrap()
        };

        let wait_semaphores: ArrayVec<[_; 1]> = [(
            image_available,
            hal::pso::PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        )]
        .into();
        let signal_semaphores: ArrayVec<[_; 1]> = [render_finished].into();
        let present_wait_semaphores: ArrayVec<[_; 1]> = [render_finished].into();

        let submission = hal::queue::Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };

        let the_command_queue = &mut queue_group.queues[0];

        unsafe {
            the_command_queue.submit(submission, Some(flight_fence));
            self.swapchain
                .present(the_command_queue, index, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")?;
        }

        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;

        self.prepare_current_frame(device)
    }

    pub unsafe fn destroy(mut self, device: &backend::Device) {
        device.destroy_command_pool(
            ManuallyDrop::into_inner(read(&mut self.command_pool)).into_raw(),
        );

        device.destroy_render_pass(ManuallyDrop::into_inner(read(&mut self.render_pass)));

        for fence in self.in_flight_fences.drain(..) {
            device.destroy_fence(fence)
        }

        for semaphore in self.render_finished_semaphores.drain(..) {
            device.destroy_semaphore(semaphore)
        }

        for semaphore in self.image_available_semaphores.drain(..) {
            device.destroy_semaphore(semaphore)
        }

        for framebuffer in self.framebuffers.drain(..) {
            device.destroy_framebuffer(framebuffer)
        }

        for image_view in self.image_views.drain(..) {
            device.destroy_image_view(image_view)
        }

        device.destroy_swapchain(ManuallyDrop::into_inner(read(&mut self.swapchain)));
    }
}
