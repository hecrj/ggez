use core::mem::ManuallyDrop;
use gfx_hal::{self as hal, Device, Instance, PhysicalDevice, QueueFamily, Surface};
use winit;

use crate::graphics::gpu::backend::{self, Backend};

pub struct Context {
    instance: ManuallyDrop<backend::Instance>,
    pub adapter: hal::Adapter<Backend>,
    pub device: ManuallyDrop<backend::Device>,
}

impl core::ops::Drop for Context {
    fn drop(&mut self) {
        let _ = self.device.wait_idle();

        unsafe {
            ManuallyDrop::drop(&mut self.device);
            ManuallyDrop::drop(&mut self.instance);
        }
    }
}

impl Context {
    pub fn new(
        window: &winit::Window,
    ) -> Result<
        (
            Context,
            <Backend as hal::Backend>::Surface,
            hal::QueueGroup<Backend, hal::Graphics>,
        ),
        &'static str,
    > {
        let instance = backend::Instance::create("window", 1);
        let surface = instance.create_surface(window);

        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families
                    .iter()
                    .any(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
            })
            .ok_or("Couldn't find a graphical Adapter!")?;

        let (device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
                .ok_or("Couldn't find a QueueFamily with graphics!")?;

            let hal::Gpu { device, mut queues } = unsafe {
                adapter
                    .physical_device
                    .open(&[(&queue_family, &[1.0; 1])])
                    .map_err(|_| "Couldn't open the PhysicalDevice!")?
            };

            let queue_group = queues
                .take::<hal::Graphics>(queue_family.id())
                .ok_or("Couldn't take ownership of the QueueGroup!")?;

            let _ = if queue_group.queues.len() > 0 {
                Ok(())
            } else {
                Err("The QueueGroup did not have any CommandQueues available!")
            }?;

            (device, queue_group)
        };

        info!(
            "Physical device limits: {:?}",
            adapter.physical_device.limits()
        );

        info!(
            "Physical device memory properties: {:?}",
            adapter.physical_device.memory_properties()
        );

        info!(
            "Physical device features: {:?}",
            adapter.physical_device.features()
        );

        Ok((
            Context {
                instance: ManuallyDrop::new(instance),
                adapter,
                device: ManuallyDrop::new(device),
            },
            surface,
            queue_group,
        ))
    }
}
