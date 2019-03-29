use image;
use wgpu;

type BgraImage = image::ImageBuffer<image::Bgra<u8>, Vec<u8>>;
pub type Texture = wgpu::Texture;
pub type TextureView = wgpu::TextureView;

pub struct Gpu {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub pipeline: wgpu::RenderPipeline,
}

impl Gpu {
    pub fn new() -> Gpu {
        let instance = wgpu::Instance::new();

        let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
            power_preference: wgpu::PowerPreference::LowPower,
        });

        let device = adapter.create_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
        });

        let vs_bytes = include_bytes!("shader/basic.vert.spv");
        let vs_module = device.create_shader_module(vs_bytes);

        let fs_bytes = include_bytes!("shader/basic.frag.spv");
        let fs_module = device.create_shader_module(fs_bytes);

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::PipelineStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: wgpu::PipelineStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            },
            rasterization_state: wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            },
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color: wgpu::BlendDescriptor::REPLACE,
                alpha: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWriteFlags::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
        });

        Gpu {
            instance,
            device,
            pipeline,
        }
    }

    pub fn upload_image(&mut self, image: &BgraImage) -> (Texture, TextureView) {
        let extent = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            array_size: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::TRANSFER_DST,
        });

        let texture_view = texture.create_default_view();

        let temp_buf = self
            .device
            .create_buffer_mapped(image.len(), wgpu::BufferUsageFlags::TRANSFER_SRC)
            .fill_from_slice(&image);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                row_pitch: 4 * image.width(),
                image_height: image.height(),
            },
            wgpu::TextureCopyView {
                texture: &texture,
                level: 0,
                slice: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            extent,
        );

        self.device.get_queue().submit(&[encoder.finish()]);

        (texture, texture_view)
    }

    pub fn new_frame_buffer(&mut self, window: &winit::Window) -> FrameBuffer {
        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());
        let surface = self.instance.create_surface(&window);

        let swap_chain = self.device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
            },
        );

        FrameBuffer {
            swap_chain: swap_chain,
        }
    }
}

pub struct Frame<'a> {
    frame: wgpu::SwapChainOutput<'a>,
}

pub struct FrameBuffer {
    swap_chain: wgpu::SwapChain,
}

impl FrameBuffer {
    pub fn next_frame(&mut self) -> Frame {
        Frame {
            frame: self.swap_chain.get_next_texture(),
        }
    }
}
