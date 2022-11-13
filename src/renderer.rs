use std::ops::Range;

use lyon::{
    geom::{point, Box2D},
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertexConstructor, StrokeOptions,
        StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
    },
    path::{Path, Winding},
};
use wgpu::{util::DeviceExt, BindGroup, Buffer, RenderPipeline, TextureView, VertexBufferLayout};
use winit::window::Window;

use crate::{
    camera::Camera,
    shape::{Color, Shape},
};

const GEOMETRY_PRIMITIVES_BUFFER_LEN: usize = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    normal: [f32; 2],
    primitive_id: u32,
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
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Uint32,
                    shader_location: 2,
                },
            ],
        }
    }
}

struct VertexCtor(u32);

impl FillVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            primitive_id: self.0,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> Vertex {
        Vertex {
            position: vertex.position_on_path().to_array(),
            normal: vertex.normal().to_array(),
            primitive_id: self.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Globals {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GeometryPrimitive {
    pub color: [f32; 4],
    pub translate: [f32; 2],
    pub scale: [f32; 2],
    pub origin: [f32; 2],
    pub rotate: f32,
    pub z_index: i32,
    pub width: f32,
    pub _pad_1: i32,
    pub _pad_2: i32,
    pub _pad_3: i32,
}

impl Default for GeometryPrimitive {
    fn default() -> Self {
        Self {
            color: [1.0; 4],
            translate: [0.0; 2],
            scale: [1.0; 2],
            origin: [0.0; 2],
            rotate: 0.0,
            z_index: 0,
            width: 0.0,
            _pad_1: 0,
            _pad_2: 0,
            _pad_3: 0,
        }
    }
}

impl GeometryPrimitive {
    pub(crate) const FILL_Z_INDEX: i32 = 0;
    pub(crate) const STROKE_Z_INDEX: i32 = 0;
}

unsafe impl bytemuck::Pod for GeometryPrimitive {}
unsafe impl bytemuck::Zeroable for GeometryPrimitive {}

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

pub struct Scene<'scene> {
    camera: &'scene Camera,
    geometry_primitives: Vec<GeometryPrimitive>,
    geometry_rect_instances: u32,
}

impl<'scene> Scene<'scene> {
    fn new(camera: &'scene Camera) -> Self {
        let geometry_primitives =
            vec![GeometryPrimitive::default(); GEOMETRY_PRIMITIVES_BUFFER_LEN];

        let geometry_rect_instances = 0;

        Self {
            camera,
            geometry_primitives,
            geometry_rect_instances,
        }
    }
}

pub struct Renderer {
    pub clear_color: Color,
    pub fill_id: u32,
    pub stroke_id: u32,
    pub globals_ubo: Buffer,
    pub primitives_ubo: Buffer,
    pub geometry_pipeline: RenderPipeline,
    pub globals_bind_group: BindGroup,
    pub geometry_ibo: Buffer,
    pub geometry_vbo: Buffer,
    pub geometry_fill_range: Range<u32>,
    pub geometry_stroke_range: Range<u32>,
    pub multisampled_render_target: Option<TextureView>,
    pub depth_texture_view: Option<TextureView>,
    pub msaa_sample_count: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        blend_state: wgpu::BlendState,
        msaa_sample_count: u32,
        clear_color: Color,
    ) -> Self {
        let tolerance = 0.02;
        let stroke_id = 0;
        let fill_id = 1;

        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();

        let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();

        let rect = Box2D::new(point(0.0, 0.0), point(1.0, 1.0));
        let mut builder = Path::builder();
        builder.add_rectangle(&rect, Winding::Positive);
        let path = builder.build();

        fill_tess
            .tessellate_path(
                &path,
                &FillOptions::tolerance(tolerance)
                    .with_fill_rule(lyon::tessellation::FillRule::NonZero),
                &mut BuffersBuilder::new(&mut geometry, VertexCtor(fill_id))
                    .with_inverted_winding(),
            )
            .unwrap();

        let geometry_fill_range = 0..(geometry.indices.len() as u32);

        stroke_tess
            .tessellate_path(
                &path,
                &StrokeOptions::tolerance(tolerance).with_line_width(50.0),
                &mut BuffersBuilder::new(&mut geometry, VertexCtor(stroke_id))
                    .with_inverted_winding(),
            )
            .unwrap();

        let geometry_stroke_range = geometry_fill_range.end..(geometry.indices.len() as u32);

        let globals_byte_buffer_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;
        let primitive_buffer_byte_size = (GEOMETRY_PRIMITIVES_BUFFER_LEN
            * std::mem::size_of::<GeometryPrimitive>())
            as wgpu::BufferAddress;

        let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals ubo"),
            size: globals_byte_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let primitives_ubo = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("primitives ubo"),
            size: primitive_buffer_byte_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let geometry_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let geometry_ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.indices),
            usage: wgpu::BufferUsages::INDEX,
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
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(globals_byte_buffer_size),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(primitive_buffer_byte_size),
                        },
                        count: None,
                    },
                ],
            });

        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &globals_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(globals_ubo.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        primitives_ubo.as_entire_buffer_binding(),
                    ),
                },
            ],
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
            depth_compare: wgpu::CompareFunction::Greater,
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
            clear_color,
            fill_id,
            stroke_id,
            globals_ubo,
            primitives_ubo,
            geometry_pipeline,
            globals_bind_group,
            geometry_ibo,
            geometry_vbo,
            geometry_fill_range,
            geometry_stroke_range,
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

    pub fn begin_scene<'scene>(&'scene self, camera: &'scene Camera) -> Scene {
        Scene::new(camera)
    }

    pub fn end_scene(&self, scene: Scene, bananas: &Bananas) {
        let globals = Globals {
            view: scene.camera.get_view().to_cols_array_2d(),
            projection: scene.camera.get_projection().to_cols_array_2d(),
        };

        bananas
            .queue
            .write_buffer(&self.globals_ubo, 0, bytemuck::cast_slice(&[globals]));
        bananas.queue.write_buffer(
            &self.primitives_ubo,
            0,
            bytemuck::cast_slice(&scene.geometry_primitives),
        );

        let frame = match bananas.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("swapchain error: {:?}", e);
                return;
            }
        };

        let mut encoder = bananas
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clear_color = wgpu::Color {
            r: self.clear_color.r as f64,
            g: self.clear_color.g as f64,
            b: self.clear_color.b as f64,
            a: self.clear_color.a as f64,
        };

        let color_attachment = if let Some(msaa_target) = &self.multisampled_render_target {
            wgpu::RenderPassColorAttachment {
                view: msaa_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
                resolve_target: Some(&render_target),
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: &render_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
                resolve_target: None,
            }
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture_view.as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            pass.set_pipeline(&self.geometry_pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_index_buffer(self.geometry_ibo.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, self.geometry_vbo.slice(..));

            pass.draw_indexed(
                self.geometry_stroke_range.clone(),
                0,
                0..(scene.geometry_rect_instances as u32),
            );
            pass.draw_indexed(
                self.geometry_fill_range.clone(),
                0,
                0..(scene.geometry_rect_instances as u32),
            );
        }

        bananas.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn draw_shape(&self, scene: &mut Scene, shape: &impl Shape) {
        let primitive_id = scene.geometry_rect_instances as usize * 2;
        assert!(primitive_id <= GEOMETRY_PRIMITIVES_BUFFER_LEN - 3);

        let stroke_primitive = &mut scene.geometry_primitives[primitive_id];
        shape.stroke_primitive(stroke_primitive);

        let fill_primitive = &mut scene.geometry_primitives[primitive_id + 1];
        shape.fill_primitive(fill_primitive);

        // TODO: Show origin - debug.

        scene.geometry_rect_instances += 1;
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
