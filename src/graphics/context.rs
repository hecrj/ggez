use crate::conf;
use image;
use wgpu;
use winit;

type BgraImage = image::ImageBuffer<image::Bgra<u8>, Vec<u8>>;
pub(crate) type Texture = wgpu::Texture;
pub(crate) type TextureView = wgpu::TextureView;

pub struct Context {
    window: winit::Window,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub pipeline: wgpu::RenderPipeline,
    pub swap_chain: wgpu::SwapChain,
}

impl Context {
    pub fn new(
        events_loop: &winit::EventsLoop,
        window_setup: &conf::WindowSetup,
        window_mode: conf::WindowMode,
    ) -> Context {
        let window_builder = winit::WindowBuilder::new()
            .with_title(window_setup.title.clone())
            .with_transparency(window_setup.transparent)
            .with_resizable(window_mode.resizable);

        let window = window_builder.build(events_loop).unwrap();

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

        let size = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor());

        let surface = instance.create_surface(&window);

        let swap_chain = device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: size.width.round() as u32,
                height: size.height.round() as u32,
            },
        );

        Context {
            window,
            device,
            surface,
            pipeline,
            swap_chain,
        }
    }

    pub(crate) fn upload_image(&mut self, image: &BgraImage) -> (Texture, TextureView) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth: 1,
            },
            array_size: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsageFlags::SAMPLED | wgpu::TextureUsageFlags::TRANSFER_DST,
        });

        let texture_view = texture.create_default_view();

        (texture, texture_view)
    }
}
