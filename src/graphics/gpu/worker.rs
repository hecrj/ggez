use std::rc::Rc;

use arrayvec::ArrayVec;
use core::mem::ManuallyDrop;
use core::ptr::read;
use gfx_hal::{self as hal, Device, Surface, Swapchain as _};

use crate::graphics::gpu::backend::{self, Backend};
use crate::graphics::gpu::context::Context;
use crate::graphics::gpu::memory::{Buffer, Chunk};

pub struct Worker {
    context: Rc<Context>,
    queue_group: ManuallyDrop<hal::QueueGroup<Backend, hal::Graphics>>,
    command_pool: ManuallyDrop<hal::pool::CommandPool<Backend, hal::Graphics>>,
}

impl Worker {
    pub fn context(&self) -> &Rc<Context> {
        &self.context
    }

    pub fn perform<F>(&mut self, f: F) -> Result<(), &'static str>
    where
        F: FnOnce(
            &mut hal::command::CommandBuffer<
                Backend,
                hal::Graphics,
                hal::command::OneShot,
                hal::command::Primary,
            >,
        ),
    {
        let mut command_buffer = self
            .command_pool
            .acquire_command_buffer::<gfx_hal::command::OneShot>();

        unsafe {
            command_buffer.begin();
        }

        f(&mut command_buffer);

        unsafe {
            command_buffer.finish();
        }

        let device = &self.context.device;

        let upload_fence = device
            .create_fence(false)
            .map_err(|_| "Couldn't create an upload fence!")?;

        let queue = &mut self.queue_group.queues[0];

        unsafe {
            queue.submit_nosemaphores(Some(&command_buffer), Some(&upload_fence));

            let _ = device
                .wait_for_fence(&upload_fence, core::u64::MAX)
                .map_err(|_| "Couldn't wait for the fence!")?;

            device.destroy_fence(upload_fence);
        }

        Ok(())
    }
}
