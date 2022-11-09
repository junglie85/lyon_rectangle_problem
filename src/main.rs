use futures::executor::block_on;
use glam::Mat4;
use lyon::math::*;
use lyon::path::{Path, Winding};
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator};
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

#[allow(dead_code)]
pub struct Camera {
    width: f32,
    height: f32,
    view: Mat4,
    projection: Mat4,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let projection = glam::Mat4::orthographic_lh(0.0, width, 0.0, height, -1.0, 10.0);

        Self {
            width,
            height,
            view: Mat4::IDENTITY,
            projection,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let projection = glam::Mat4::orthographic_lh(0.0, width, 0.0, height, -1.0, 10.0);

        self.width = width;
        self.height = height;
        self.projection = projection;
    }

    pub fn get_view(&self) -> Mat4 {
        // Just use some jankey values for look at for now.
        // let view = glam::Mat4::look_at_lh(
        //     glam::Vec3::new(-200.0, -200.0, -1.0),
        //     glam::Vec3::new(-200.0, -200.0, 0.0),
        //     glam::Vec3::Y,
        // );

        let view = glam::Mat4::look_at_lh(
            glam::Vec3::new(0.0, 0.0, -1.0),
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::Y,
        );

        view
    }

    pub fn get_projection(&self) -> Mat4 {
        self.projection
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuVertex {
    position: [f32; 3],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Globals {
    pub(crate) view: [[f32; 4]; 4],
    pub(crate) projection: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

const ASPECT_RATIO: f32 = 16_f32 / 9_f32;
pub const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = DEFAULT_WINDOW_WIDTH as f32 / ASPECT_RATIO;

fn create_multisampled_framebuffer(
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

fn main() {
    env_logger::init();

    let sample_count = 4; // 1 = disable MSAA.

    let tolerance = 0.02;

    let mut geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();

    let rect = Box2D::new(point(0.0, 0.0), point(200.0, 200.0));
    let mut builder = Path::builder();
    builder.add_rectangle(&rect, Winding::Positive);
    let path = builder.build();

    fill_tess
        .tessellate_path(
            &path,
            &FillOptions::tolerance(tolerance).with_fill_rule(tessellation::FillRule::NonZero),
            &mut BuffersBuilder::new(&mut geometry, VertexCtor).with_inverted_winding(),
        )
        .unwrap();

    let geometry_fill_range = 0..(geometry.indices.len() as u32);

    stroke_tess
        .tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance),
            &mut BuffersBuilder::new(&mut geometry, VertexCtor).with_inverted_winding(),
        )
        .unwrap();

    let geometry_stroke_range = geometry_fill_range.end..(geometry.indices.len() as u32);

    let mut scene = SceneParams {
        window_size: PhysicalSize::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
        size_changed: true,
        render: false,
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_inner_size(scene.window_size);
    let window = window_builder.build(&event_loop).unwrap();

    // create an instance
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    // create an surface
    let surface = unsafe { instance.create_surface(&window) };

    // create an adapter
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();
    // create a device and a queue
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))
    .unwrap();

    let globals_byte_buffer_size = std::mem::size_of::<Globals>() as wgpu::BufferAddress;

    let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Globals UBO"),
        size: globals_byte_buffer_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let globals_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
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
        label: Some("Globals Bind Group"),
        layout: &globals_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: globals_ubo.as_entire_binding(),
        }],
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
        label: Some("Geometry vs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
    });

    let geometry_fs_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Geometry fs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
    });

    let geometry_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&globals_bind_group_layout],
        push_constant_ranges: &[],
        label: Some("Geometry Pipeline Layout"),
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
        label: Some("Geometry Pipeline"),
        layout: Some(&geometry_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &geometry_vs_module,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x3,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        offset: 12,
                        format: wgpu::VertexFormat::Float32x4,
                        shader_location: 1,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &geometry_fs_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: None,
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
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let size = window.inner_size();

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
    };

    surface.configure(&device, &surface_config);

    let mut multisampled_render_target = None;

    let mut depth_texture_view = None;

    let mut camera = Camera::new(size.width as f32, size.height as f32);

    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        if !process_event(event, &window, control_flow, &mut scene) {
            // keep polling inputs.
            return;
        }

        if scene.size_changed {
            scene.size_changed = false;
            let physical = scene.window_size;
            surface_config.width = physical.width;
            surface_config.height = physical.height;
            surface.configure(&device, &surface_config);

            camera.resize(physical.width as f32, physical.height as f32);

            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture"),
                size: wgpu::Extent3d {
                    width: surface_config.width,
                    height: surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            depth_texture_view =
                Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

            multisampled_render_target = if sample_count > 1 {
                Some(create_multisampled_framebuffer(
                    &device,
                    &surface_config,
                    sample_count,
                ))
            } else {
                None
            };
        }

        if !scene.render {
            return;
        }

        scene.render = false;

        let frame = match surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("Swap-chain error: {:?}", e);
                return;
            }
        };

        let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        let frame_globals = Globals {
            view: camera.get_view().to_cols_array_2d(),
            projection: camera.get_projection().to_cols_array_2d(),
        };

        queue.write_buffer(&globals_ubo, 0, bytemuck::cast_slice(&[frame_globals]));

        {
            let color_attachment = if let Some(msaa_target) = &multisampled_render_target {
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

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view.as_ref().unwrap(),
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

            pass.set_pipeline(&geometry_pipeline);
            pass.set_bind_group(0, &globals_bind_group, &[]);
            pass.set_index_buffer(geometry_ibo.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, geometry_vbo.slice(..));

            pass.draw_indexed(geometry_fill_range.clone(), 0, 0..1);
            pass.draw_indexed(geometry_stroke_range.clone(), 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();

        frame_count += 1;
        let now = Instant::now();
        if now >= next_report {
            println!("{} FPS", frame_count);
            frame_count = 0;
            next_report = now + Duration::from_secs(1);
        }
    });
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct VertexCtor;

// var transformed_pos = world_pos * vec3<f32>(globals.zoom / (0.5 * globals.resolution.x), globals.zoom / (0.5 * globals.resolution.y), 1.0);
// TODO: Pass in color and ZIndex
impl FillVertexConstructor<GpuVertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        let z_index = 0.1;
        let p = vertex.position().to_array();

        GpuVertex {
            position: [p[0], p[1], z_index],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

// TODO: We want the color, ZIndex and the width passed in.
impl StrokeVertexConstructor<GpuVertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        let stroke_width = 1.0;
        let z_index = 0.2;
        let p = vertex.position() + vertex.normal() * stroke_width;

        GpuVertex {
            position: [p.x, p.y, z_index],
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

struct SceneParams {
    window_size: PhysicalSize<u32>,
    size_changed: bool,
    render: bool,
}

fn process_event(
    event: Event<()>,
    window: &Window,
    control_flow: &mut ControlFlow,
    scene: &mut SceneParams,
) -> bool {
    match event {
        Event::RedrawRequested(_) => {
            scene.render = true;
        }
        Event::RedrawEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::Destroyed,
            ..
        }
        | Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
            return false;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            scene.window_size = size;
            scene.size_changed = true
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                },
            ..
        } => match key {
            VirtualKeyCode::Escape => {
                *control_flow = ControlFlow::Exit;
                return false;
            }
            _key => {}
        },
        _event => {}
    }

    *control_flow = ControlFlow::Poll;

    true
}
