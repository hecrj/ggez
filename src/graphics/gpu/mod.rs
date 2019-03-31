#![allow(unsafe_code)]
mod backend;
mod context;
mod swapchain;

use core::mem::ManuallyDrop;
use core::ptr::read;
use gfx_hal::{self as hal, Device, Instance, PhysicalDevice, QueueFamily, Surface};
use image;
use std::rc::Rc;
use winit;

use backend::Backend;
use context::Context;
use swapchain::Swapchain;

use crate::graphics::Color;

type BgraImage = image::ImageBuffer<image::Bgra<u8>, Vec<u8>>;
pub type Texture = ();
pub type TextureView = ();

pub struct Gpu {
    context: Rc<Context>,
    window: winit::Window,
    surface: <Backend as hal::Backend>::Surface,
    queue_group: ManuallyDrop<hal::QueueGroup<Backend, hal::Graphics>>,
    swapchain: ManuallyDrop<Swapchain>,
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

        Ok(Gpu {
            context: Rc::new(context),
            window,
            surface,
            queue_group: ManuallyDrop::new(queue_group),
            swapchain: ManuallyDrop::new(swapchain),
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

    pub fn upload_image(&mut self, image: &BgraImage) -> (Texture, TextureView) {
        ((), ())
    }
}

impl core::ops::Drop for Gpu {
    fn drop(&mut self) {
        let _ = self.context.device.wait_idle();

        unsafe {
            ManuallyDrop::into_inner(read(&mut self.swapchain)).destroy(&self.context.device);

            ManuallyDrop::drop(&mut self.queue_group);
        }
    }
}
