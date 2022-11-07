use futures::executor::block_on;
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuVertex {
    position: [f32; 3],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

fn main() {
    env_logger::init();

    let num_instances: u32 = 1;
    let tolerance = 0.00002;

    let mut geometry: VertexBuffers<GpuVertex, u16> = VertexBuffers::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();

    // let rect = Box2D::new(point(0.0, 0.0), point(50.0, 50.0));
    let rect = Box2D::new(point(0.0, 0.0), point(0.125, 0.125));
    let mut builder = Path::builder();
    builder.add_rectangle(&rect, Winding::Negative);
    let path = builder.build();

    dbg!(&path);

    fill_tess
        .tessellate_path(
            &path,
            &FillOptions::tolerance(tolerance).with_fill_rule(tessellation::FillRule::NonZero),
            &mut BuffersBuilder::new(&mut geometry, WithId),
        )
        .unwrap();

    let fill_range = 0..(geometry.indices.len() as u32);

    stroke_tess
        .tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance),
            &mut BuffersBuilder::new(&mut geometry, WithId),
        )
        .unwrap();

    let stroke_range = fill_range.end..(geometry.indices.len() as u32);

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

    dbg!(&geometry);

    let vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let vs_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Geometry vs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
    });
    let fs_module = &device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Geometry fs"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./../shaders/geometry.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[],
        push_constant_ranges: &[],
        label: None,
    });

    let depth_stencil_state = Some(wgpu::DepthStencilState {
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

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
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
            module: &fs_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
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
        depth_stencil: depth_stencil_state.clone(),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let size = window.inner_size();

    let mut surface_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
    };

    surface.configure(&device, &surface_desc);

    let mut depth_texture_view = None;

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        if !update_inputs(event, &window, control_flow, &mut scene) {
            // keep polling inputs.
            return;
        }

        if scene.size_changed {
            scene.size_changed = false;
            let physical = scene.window_size;
            surface_desc.width = physical.width;
            surface_desc.height = physical.height;
            surface.configure(&device, &surface_desc);

            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture"),
                size: wgpu::Extent3d {
                    width: surface_desc.width,
                    height: surface_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            });

            depth_texture_view =
                Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
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

        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &frame_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
                resolve_target: None,
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

            pass.set_pipeline(&render_pipeline);
            pass.set_index_buffer(ibo.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, vbo.slice(..));

            pass.draw_indexed(fill_range.clone(), 0, 0..(num_instances as u32));
            pass.draw_indexed(stroke_range.clone(), 0, 0..1);
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
pub struct WithId;

// TODO: Pass in color and ZIndex
impl FillVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        let p = vertex.position().to_array();
        GpuVertex {
            position: [p[0], -p[1], 0.1],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

// TODO: We want the color, ZIndex and the width passed in.
impl StrokeVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        let stroke_width = 0.00001;
        let p = (vertex.position() + vertex.normal() * stroke_width).to_array();
        let z_index = 0.2;
        GpuVertex {
            position: [p[0], -p[1], z_index],
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

// // var transformed_pos = world_pos * vec3<f32>(globals.zoom / (0.5 * globals.resolution.x), globals.zoom / (0.5 * globals.resolution.y), 1.0);
// // TODO: Pass in color and ZIndex
// impl FillVertexConstructor<GpuVertex> for WithId {
//     fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
//         let global_zoom = 1.0;
//         let global_resolution = [DEFAULT_WINDOW_WIDTH, -DEFAULT_WINDOW_HEIGHT];
//         let p = vertex.position().to_array();
//         let p = [
//             p[0] * (global_zoom / (0.5 * global_resolution[0])),
//             p[1] * (global_zoom / (0.5 * global_resolution[1])),
//         ];
//         GpuVertex {
//             position: [p[0], p[1], 0.1],
//             color: [1.0, 1.0, 1.0, 1.0],
//         }
//     }
// }

// // TODO: We want the color, ZIndex and the width passed in.
// impl StrokeVertexConstructor<GpuVertex> for WithId {
//     fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
//         let global_zoom = 1.0;
//         let global_resolution = [DEFAULT_WINDOW_WIDTH, -DEFAULT_WINDOW_HEIGHT];
//         let stroke_width = 1.0;
//         let p = (vertex.position() + vertex.normal() * stroke_width).to_array();
//         let p = [
//             p[0] * (global_zoom / (0.5 * global_resolution[0])),
//             p[1] * (global_zoom / (0.5 * global_resolution[1])),
//         ];
//         let z_index = 0.2;
//         GpuVertex {
//             position: [p[0], p[1], z_index],
//             color: [0.0, 0.0, 0.0, 1.0],
//         }
//     }
// }

struct SceneParams {
    window_size: PhysicalSize<u32>,
    size_changed: bool,
    render: bool,
}

fn update_inputs(
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
        _evt => {}
    }

    *control_flow = ControlFlow::Poll;

    true
}
