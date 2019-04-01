#![allow(unsafe_code)]
mod backend;
mod context;
pub mod memory;
mod swapchain;
pub mod worker;

use core::mem::ManuallyDrop;
use core::ptr::read;
use gfx_hal::{self as hal, Device};
use std::rc::Rc;
use winit;

use backend::Backend;
use context::Context;
use swapchain::Swapchain;

use crate::graphics::Color;

pub struct Gpu {
    context: Rc<Context>,
    window: winit::Window,
    surface: <Backend as hal::Backend>::Surface,
    queue_group: ManuallyDrop<hal::QueueGroup<Backend, hal::Graphics>>,
    swapchain: ManuallyDrop<Swapchain>,
    sampler: ManuallyDrop<<Backend as hal::Backend>::Sampler>,
}

impl core::ops::Drop for Gpu {
    fn drop(&mut self) {
        let _ = self.context.device.wait_idle();

        unsafe {
            ManuallyDrop::into_inner(read(&mut self.swapchain)).destroy(&self.context.device);

            self.context
                .device
                .destroy_sampler(ManuallyDrop::into_inner(read(&mut self.sampler)));

            ManuallyDrop::drop(&mut self.queue_group);
        }
    }
}

impl Gpu {
    pub fn new(window: winit::Window) -> Result<Gpu, &'static str> {
        let (context, mut surface, queue_group) = Context::new(&window)?;

        let swapchain = Swapchain::new(
            &context.device,
            &mut surface,
            &context.adapter,
            &queue_group,
        )?;

        let sampler = unsafe {
            context
                .device
                .create_sampler(gfx_hal::image::SamplerInfo::new(
                    gfx_hal::image::Filter::Nearest,
                    gfx_hal::image::WrapMode::Tile,
                ))
                .map_err(|_| "Couldn't create the sampler!")?
        };

        Ok(Gpu {
            context: Rc::new(context),
            window,
            surface,
            queue_group: ManuallyDrop::new(queue_group),
            swapchain: ManuallyDrop::new(swapchain),
            sampler: ManuallyDrop::new(sampler),
        })
    }

    pub fn clear(&mut self, color: &Color) {
        self.swapchain.clear(color);
    }

    pub fn present(&mut self) {
        let result = self
            .swapchain
            .present(&self.context.device, &mut self.queue_group);

        match result {
            Ok(_) => {}
            Err(_) => {
                warn!("Recreating swapchain");
                self.context.device.wait_idle().unwrap();

                unsafe {
                    ManuallyDrop::into_inner(read(&mut self.swapchain))
                        .destroy(&self.context.device);
                }

                self.swapchain = ManuallyDrop::new(
                    Swapchain::new(
                        &self.context.device,
                        &mut self.surface,
                        &self.context.adapter,
                        &self.queue_group,
                    )
                    .unwrap(),
                );
            }
        }
    }
}
