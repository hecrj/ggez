use core::mem::ManuallyDrop;
use core::ops::Drop;
use std::rc::Rc;

use gfx_hal::{self as hal, Device, PhysicalDevice};

use crate::graphics::gpu::backend::Backend;
use crate::graphics::gpu::context::Context;

/// A chunk of memory with some requirements and properties
pub struct Chunk {
    context: Rc<Context>,
    requirements: hal::memory::Requirements,
    properties: hal::memory::Properties,
    memory: ManuallyDrop<<Backend as hal::Backend>::Memory>,
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let device = &self.context.device;
        let _ = device.wait_idle();

        unsafe {
            use core::ptr::read;
            device.free_memory(ManuallyDrop::into_inner(read(&mut self.memory)));
        }
    }
}

impl Chunk {
    pub fn new(
        context: &Rc<Context>,
        requirements: hal::memory::Requirements,
        properties: hal::memory::Properties,
    ) -> Result<Chunk, &'static str> {
        let device = &context.device;
        let adapter = &context.adapter;

        let memory_type_id = adapter
            .physical_device
            .memory_properties()
            .memory_types
            .iter()
            .enumerate()
            .find(|&(id, memory_type)| {
                requirements.type_mask & (1 << id) != 0
                    && memory_type.properties.contains(properties)
            })
            .map(|(id, _)| hal::adapter::MemoryTypeId(id))
            .ok_or("Couldn't find a memory type to support the buffer!")?;

        let memory = unsafe {
            device
                .allocate_memory(memory_type_id, requirements.size)
                .map_err(|_| "Couldn't allocate buffer memory!")?
        };

        Ok(Chunk {
            context: context.clone(),
            requirements,
            properties,
            memory: ManuallyDrop::new(memory),
        })
    }

    pub fn context(&self) -> &Rc<Context> {
        &self.context
    }

    pub fn requirements(&self) -> &hal::memory::Requirements {
        &self.requirements
    }

    pub fn memory(&self) -> &<Backend as hal::Backend>::Memory {
        &self.memory
    }
}
