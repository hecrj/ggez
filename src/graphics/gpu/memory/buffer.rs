use core::mem::ManuallyDrop;
use core::ops::Drop;
use std::rc::Rc;

use gfx_hal::{self as hal, Device};

use crate::graphics::gpu::backend::Backend;
use crate::graphics::gpu::context::Context;
use crate::graphics::gpu::memory::chunk::Chunk;

/// A cpu visible buffer
pub struct Buffer {
    buffer: ManuallyDrop<<Backend as hal::Backend>::Buffer>,
    chunk: ManuallyDrop<Chunk>,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let device = &self.chunk.context().device;
        let _ = device.wait_idle();

        unsafe {
            use core::ptr::read;
            device.destroy_buffer(ManuallyDrop::into_inner(read(&self.buffer)));
            ManuallyDrop::drop(&mut self.chunk);
        }
    }
}

impl Buffer {
    pub fn new(
        context: &Rc<Context>,
        size: usize,
        usage: hal::buffer::Usage,
    ) -> Result<Buffer, &'static str> {
        let device = &context.device;

        unsafe {
            let mut buffer = device
                .create_buffer(size as u64, usage)
                .map_err(|_| "Couldn't create a buffer!")?;

            let requirements = device.get_buffer_requirements(&buffer);
            let chunk = Chunk::new(context, requirements, hal::memory::Properties::CPU_VISIBLE)?;

            device
                .bind_buffer_memory(chunk.memory(), 0, &mut buffer)
                .map_err(|_| "Couldn't bind the buffer memory!")?;

            Ok(Buffer {
                buffer: ManuallyDrop::new(buffer),
                chunk: ManuallyDrop::new(chunk),
            })
        }
    }

    pub fn buffer(&self) -> &<Backend as hal::Backend>::Buffer {
        &self.buffer
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn write<F>(&mut self, f: F) -> Result<(), &'static str>
    where
        F: FnOnce(&mut hal::mapping::Writer<Backend, u8>),
    {
        let device = &self.chunk.context().device;

        let mut writer = unsafe {
            device
                .acquire_mapping_writer::<u8>(
                    self.chunk.memory(),
                    0..self.chunk.requirements().size,
                )
                .map_err(|_| "Couldn't acquire a mapping writer to the staging buffer!")?
        };

        f(&mut writer);

        unsafe {
            device
                .release_mapping_writer(writer)
                .map_err(|_| "Couldn't release the mapping writer to the staging buffer!")?;
        }

        Ok(())
    }
}
