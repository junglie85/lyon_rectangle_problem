use std::time::{Duration, Instant};

use components::{compute_transformation_matrix, Drawable, Transform};
pub use env_logger::init as init_logger;
use futures::executor::block_on;
use glam::{Vec2, Vec4};
use graphics::Color;
use hecs::World;
use renderer::{Globals, GraphicsDevice, Renderer, Vertex};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferAddress, BufferUsages, COPY_BUFFER_ALIGNMENT,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::camera::Camera;

const ASPECT_RATIO: f32 = 16_f32 / 9_f32;
pub const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = DEFAULT_WINDOW_WIDTH as f32 / ASPECT_RATIO;

mod camera;
pub mod components;
pub mod graphics;
mod renderer;

pub struct EngineSettings {
    pub frame_rate: u32,
    pub clear_color: Color,
}

impl Default for EngineSettings {
    fn default() -> Self {
        let frame_rate = 60;
        let clear_color = Color::new(1.0, 0.0, 1.0, 1.0);

        Self {
            frame_rate,
            clear_color,
        }
    }
}

pub trait Game {
    fn pre_init(&self, _settings: &mut EngineSettings) {}
    fn post_init(&self, _world: &mut World, _window_size: Vec2) {}
    fn update(&mut self) {}
}

pub fn start<G>()
where
    G: Game + Default,
{
    let game = G::default();

    let mut engine_settings = EngineSettings::default();
    game.pre_init(&mut engine_settings);

    let mut state = FrameState {
        window_size: PhysicalSize::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
        size_changed: true,
        render: false,
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_inner_size(state.window_size);
    let window = window_builder.build(&event_loop).unwrap();

    let blend_state = wgpu::BlendState {
        color: wgpu::BlendComponent::REPLACE,
        alpha: wgpu::BlendComponent::REPLACE,
    };

    let sample_count = 4; // 1 = disable MSAA.

    let mut device = block_on(GraphicsDevice::new(&window));
    let mut renderer = Renderer::new(
        &device.device,
        device.config.format,
        blend_state,
        sample_count,
        engine_settings.clear_color,
    );

    let mut world = World::new();

    let mut camera = Camera::new(device.size.width as f32, device.size.height as f32);

    let window_size = Vec2::new(
        state.window_size.width as f32,
        state.window_size.height as f32,
    );
    game.post_init(&mut world, window_size);

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        //////////////////// INPUT ////////////////////
        if !process_event(event, &window, control_flow, &mut state) {
            // keep polling inputs.
            return;
        }

        if state.size_changed {
            state.size_changed = false;

            let physical = state.window_size;
            device.resize(physical);
            renderer.resize(&device);
            camera.resize(physical.width as f32, physical.height as f32);
        }

        //////////////////// UPDATE ////////////////////

        //////////////////// RENDER ////////////////////
        if !state.render {
            return;
        }

        state.render = false;

        let mut vertices = Vec::with_capacity(renderer.max_geometry_vertices); // TODO: set a capacity and draw when reached.
        let mut indices = Vec::with_capacity(renderer.max_geometry_indices); // TODO: set a capacity and draw when reached.

        for (_id, (transform, drawable)) in world.query::<(&Transform, &Drawable)>().iter() {
            let t = compute_transformation_matrix(&transform);
            let index_offset = vertices.len() as u16;
            match drawable {
                Drawable::Circle(circle) => {
                    for v in circle.vertices() {
                        let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                        let color = v.color().to_array();
                        let vertex = Vertex { position, color };
                        vertices.push(vertex);
                    }

                    for i in circle.indices() {
                        indices.push(index_offset + i);
                    }
                }
                Drawable::Line(line) => {
                    for v in line.vertices() {
                        let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                        let color = v.color().to_array();
                        let vertex = Vertex { position, color };
                        vertices.push(vertex);
                    }

                    for i in line.indices() {
                        indices.push(index_offset + i);
                    }
                }
                Drawable::Polygon(polygon) => {
                    for v in polygon.vertices() {
                        let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                        let color = v.color().to_array();
                        let vertex = Vertex { position, color };
                        vertices.push(vertex);
                    }

                    for i in polygon.indices() {
                        indices.push(index_offset + i);
                    }
                }
                Drawable::Rect(rect) => {
                    for v in rect.vertices() {
                        let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                        let color = v.color().to_array();
                        let vertex = Vertex { position, color };
                        vertices.push(vertex);
                    }

                    for i in rect.indices() {
                        indices.push(index_offset + i);
                    }
                }
            }
        }

        let unaligned_indices_len = indices.len();
        for _ in 0..(unaligned_indices_len % COPY_BUFFER_ALIGNMENT as usize) {
            indices.push(0);
        }

        let globals = Globals {
            view: camera.get_view().to_cols_array_2d(),
            projection: camera.get_projection().to_cols_array_2d(),
        };

        device
            .queue
            .write_buffer(&renderer.globals_ubo, 0, bytemuck::cast_slice(&[globals]));

        let frame = match device.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("swapchain error: {:?}", e);
                return;
            }
        };

        let mut encoder = device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let vertex_buffer = device.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("frame geometry vbo"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::COPY_SRC,
        });

        let index_buffer = device.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("frame geometry ibo"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::COPY_SRC,
        });

        if vertices.len() > renderer.max_geometry_vertices
            || indices.len() > renderer.max_geometry_indices
        {
            renderer.resize_geometry_buffers(&device.device, vertices.len(), indices.len())
        }

        encoder.copy_buffer_to_buffer(
            &vertex_buffer,
            0,
            &renderer.geometry_vbo,
            0,
            (std::mem::size_of::<Vertex>() * vertices.len()) as BufferAddress,
        );

        encoder.copy_buffer_to_buffer(
            &index_buffer,
            0,
            &renderer.geometry_ibo,
            0,
            (std::mem::size_of::<u16>() * indices.len()) as BufferAddress,
        );

        let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let clear_color = wgpu::Color {
            r: renderer.clear_color.r as f64,
            g: renderer.clear_color.g as f64,
            b: renderer.clear_color.b as f64,
            a: renderer.clear_color.a as f64,
        };

        let color_attachment = if let Some(msaa_target) = &renderer.multisampled_render_target {
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
                    view: renderer.depth_texture_view.as_ref().unwrap(),
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

            pass.set_pipeline(&renderer.geometry_pipeline);
            pass.set_bind_group(0, &renderer.globals_bind_group, &[]);
            pass.set_index_buffer(renderer.geometry_ibo.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, renderer.geometry_vbo.slice(..));

            pass.draw_indexed(0..unaligned_indices_len as u32, 0, 0..1);
        }

        device.queue.submit(Some(encoder.finish()));
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

struct FrameState {
    window_size: PhysicalSize<u32>,
    size_changed: bool,
    render: bool,
}

fn process_event(
    event: Event<()>,
    window: &Window,
    control_flow: &mut ControlFlow,
    state: &mut FrameState,
) -> bool {
    match event {
        Event::RedrawRequested(_) => {
            state.render = true;
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
            state.window_size = size;
            state.size_changed = true
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
