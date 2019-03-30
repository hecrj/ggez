use crate::graphics::gpu;
use crate::graphics::Frame;
use crate::Context;
use winit;

pub struct Window {
    window: winit::Window,
    frame_buffer: gpu::FrameBuffer,
}

impl Window {
    pub fn new(context: &mut Context) -> (Window, winit::EventsLoop) {
        let window_setup = &context.conf.window_setup;
        let window_mode = &context.conf.window_mode;
        let events_loop = winit::EventsLoop::new();

        let window_builder = winit::WindowBuilder::new()
            .with_title(window_setup.title.clone())
            .with_transparency(window_setup.transparent)
            .with_resizable(window_mode.resizable);

        let window = window_builder.build(&events_loop).unwrap();

        let frame_buffer = context.gpu.new_frame_buffer(&window);

        (
            Window {
                window,
                frame_buffer,
            },
            events_loop,
        )
    }

    pub fn next_frame(&mut self) -> Frame {
        self.frame_buffer.next_frame()
    }
}
