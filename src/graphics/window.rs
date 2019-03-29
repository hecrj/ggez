use crate::conf;
use crate::graphics::gpu;
use crate::Context;
use winit;

pub struct Window {
    window: winit::Window,
    frame_buffer: gpu::FrameBuffer,
}

impl Window {
    pub fn new(
        context: &mut Context,
        events_loop: &winit::EventsLoop,
        window_setup: &conf::WindowSetup,
        window_mode: conf::WindowMode,
    ) -> Window {
        let window_builder = winit::WindowBuilder::new()
            .with_title(window_setup.title.clone())
            .with_transparency(window_setup.transparent)
            .with_resizable(window_mode.resizable);

        let window = window_builder.build(events_loop).unwrap();

        let frame_buffer = context.gpu.new_frame_buffer(&window);

        Window {
            window,
            frame_buffer,
        }
    }

    pub fn next_frame(&mut self) -> Frame {
        self.frame_buffer.next_frame()
    }
}

pub type Frame<'a> = gpu::Frame<'a>;
