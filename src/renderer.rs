use wgpu::{
    BindGroup, Buffer, BufferAddress, BufferUsages, Device, RenderPipeline, TextureView,
    VertexBufferLayout,
};
use winit::window::Window;

use crate::graphics::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Globals {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

pub struct Bananas {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
}

impl Bananas {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
            size,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

pub struct Renderer {
    pub max_geometry_vertices: usize,
    pub max_geometry_indices: usize,
    pub clear_color: Color,
    pub globals_ubo: Buffer,
    pub geometry_pipeline: RenderPipeline,
    pub globals_bind_group: BindGroup,
    pub geometry_ibo: Buffer,
    pub geometry_vbo: Buffer,
    pub multisampled_render_target: Option<TextureView>,
    pub depth_texture_view: Option<TextureView>,
    pub msaa_sample_count: u32,
}

impl Renderer {
    const INITIAL_GEOMETRY_COUNT: usize = 1; // TODO: Figure out better initial buffer capacity.

    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        blend_state: wgpu::BlendState,
        msaa_sample_count: u32,
        clear_color: Color,
    ) -> Self {
        let max_geometry_vertices = 4 * Self::INITIAL_GEOMETRY_COUNT;
        let max_geometry_indices = 6 * Self::INITIAL_GEOMETRY_COUNT;

        let globals_byte_buffer_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;

        let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals ubo"),
            size: globals_byte_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let geometry_vbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("geometry vbo"),
            size: (std::mem::size_of::<Vertex>() * max_geometry_vertices) as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let geometry_ibo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("geometry ibo"),
            size: (std::mem::size_of::<u16>() * max_geometry_indices) as BufferAddress,
            usage: wgpu::BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let geometry_vs_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("geometry vs"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
        });

        let geometry_fs_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("geometry fs"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
        });

        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_byte_buffer_size),
                    },
                    count: None,
                }],
            });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(globals_ubo.as_entire_buffer_binding()),
            }],
        });

        let geometry_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&globals_bind_group_layout],
                push_constant_ranges: &[],
                label: Some("geometry pipeline layout"),
            });

        let geometry_depth_stencil_state = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::GreaterEqual,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState::IGNORE,
                back: wgpu::StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: wgpu::DepthBiasState::default(),
        });

        let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("geometry pipeline"),
            layout: Some(&geometry_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &geometry_vs_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &geometry_fs_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                polygon_mode: wgpu::PolygonMode::Fill,
                front_face: wgpu::FrontFace::Ccw,
                strip_index_format: None,
                cull_mode: Some(wgpu::Face::Back),
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: geometry_depth_stencil_state.clone(),
            multisample: wgpu::MultisampleState {
                count: msaa_sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let multisampled_render_target = None;

        let depth_texture_view = None;

        Self {
            max_geometry_vertices,
            max_geometry_indices,
            clear_color,
            globals_ubo,
            geometry_pipeline,
            globals_bind_group,
            geometry_ibo,
            geometry_vbo,
            multisampled_render_target,
            depth_texture_view,
            msaa_sample_count,
        }
    }

    pub fn resize(&mut self, bananas: &Bananas) {
        let depth_texture = bananas.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: bananas.config.width,
                height: bananas.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: self.msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        self.depth_texture_view =
            Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

        self.multisampled_render_target = if self.msaa_sample_count > 1 {
            Some(Self::create_multisampled_framebuffer(
                &bananas.device,
                &bananas.config,
                self.msaa_sample_count,
            ))
        } else {
            None
        };
    }

    pub fn resize_geometry_buffers(
        &mut self,
        device: &Device,
        max_geometry_vertices: usize,
        max_geometry_indices: usize,
    ) {
        self.geometry_vbo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("geometry vbo"),
            size: (std::mem::size_of::<Vertex>() * max_geometry_vertices) as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.geometry_ibo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("geometry ibo"),
            size: (std::mem::size_of::<u16>() * max_geometry_indices) as BufferAddress,
            usage: wgpu::BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.max_geometry_vertices = max_geometry_vertices;
        self.max_geometry_indices = max_geometry_indices;
    }

    pub fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            label: Some("multisampled frame descriptor"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
