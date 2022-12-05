use std::time::{Duration, Instant};

use components::{compute_transformation_matrix, Drawable, Transform};
pub use env_logger::init as init_logger;
use futures::executor::block_on;
use glam::{Vec2, Vec4};
use graphics::Color;
use input::InputHelper;
use renderer::{Globals, GraphicsDevice, Renderer, Vertex};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferAddress, BufferUsages, COPY_BUFFER_ALIGNMENT,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

use crate::camera::Camera;

const ASPECT_RATIO: f32 = 16_f32 / 9_f32;
pub const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = DEFAULT_WINDOW_WIDTH as f32 / ASPECT_RATIO;
pub const DEFAULT_TITLE: &str = "Papercut2D";

pub mod camera;
pub mod components;
pub mod graphics;
pub mod input;
mod renderer;

#[derive(Debug)]
pub enum Fullscreen {
    Exclusive,
    Borderless,
}

impl Default for Fullscreen {
    fn default() -> Self {
        Fullscreen::Borderless
    }
}

#[derive(Debug)]
pub struct WindowConfig {
    pub title: String,
    pub size: Vec2,
    pub fullscreen: Option<Fullscreen>,
    pub _frame_rate: u32, // TODO: Enable desired framerate to be configured
}

impl Default for WindowConfig {
    fn default() -> Self {
        let title = DEFAULT_TITLE.to_string();
        let size = Vec2::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        let fullscreen = None;
        let frame_rate = 60;

        Self {
            title,
            size,
            fullscreen,
            _frame_rate: frame_rate,
        }
    }
}

#[derive(Debug)]
pub struct RendererConfig {
    pub clear_color: Color,
}

impl Default for RendererConfig {
    fn default() -> Self {
        let clear_color = Color::new(1.0, 0.0, 1.0, 1.0);

        Self { clear_color }
    }
}

#[derive(Debug)]
pub struct Context {
    window_title: String,
    window_size: Vec2,
}

impl<'frame> Context {
    pub fn window_size(&self) -> Vec2 {
        self.window_size
    }

    pub fn set_window_title(&mut self, title: impl Into<String>) {
        self.window_title = title.into();
    }

    pub fn draw_shape(&self, transform: &Transform, drawable: &Drawable, scene: &mut Scene) {
        let t = compute_transformation_matrix(&transform);
        let index_offset = scene.vertices.len() as u16;
        match drawable {
            Drawable::Circle(circle) => {
                for v in circle.vertices() {
                    let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                    let color = v.color().to_array();
                    let vertex = Vertex { position, color };
                    scene.vertices.push(vertex);
                }

                for i in circle.indices() {
                    scene.indices.push(index_offset + i);
                }
            }
            Drawable::Line(line) => {
                for v in line.vertices() {
                    let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                    let color = v.color().to_array();
                    let vertex = Vertex { position, color };
                    scene.vertices.push(vertex);
                }

                for i in line.indices() {
                    scene.indices.push(index_offset + i);
                }
            }
            Drawable::Polygon(polygon) => {
                for v in polygon.vertices() {
                    let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                    let color = v.color().to_array();
                    let vertex = Vertex { position, color };
                    scene.vertices.push(vertex);
                }

                for i in polygon.indices() {
                    scene.indices.push(index_offset + i);
                }
            }
            Drawable::Rect(rect) => {
                for v in rect.vertices() {
                    let position = (t * Vec4::from((v.position(), 0.0, 1.0))).to_array();
                    let color = v.color().to_array();
                    let vertex = Vertex { position, color };
                    scene.vertices.push(vertex);
                }

                for i in rect.indices() {
                    scene.indices.push(index_offset + i);
                }
            }
        }
    }
}

pub trait Game {
    fn on_create(&mut self, _ctx: &mut Context) {}
    fn on_update(
        &mut self,
        _scene: &mut Scene,
        input: &InputHelper,
        _ctx: &mut Context,
        _camera: &Camera,
        _dt: Duration,
    ) -> bool {
        !input.quit()
    }
}

pub fn start<G>(mut window_config: WindowConfig, renderer_config: RendererConfig)
where
    G: Game + Default + 'static,
{
    let mut game = G::default();

    let event_loop = EventLoop::new();

    let monitor = event_loop
        .available_monitors()
        .next()
        .expect("no monitors found");

    let fullscreen = match window_config.fullscreen {
        Some(Fullscreen::Borderless) => {
            Some(winit::window::Fullscreen::Borderless(Some(monitor.clone())))
        }
        Some(Fullscreen::Exclusive) => {
            let mode = monitor.video_modes().next().expect("no modes found");
            Some(winit::window::Fullscreen::Exclusive(mode.clone()))
        }
        _ => None,
    };

    let window_builder = WindowBuilder::new()
        .with_title(&window_config.title)
        .with_inner_size(PhysicalSize::new(
            window_config.size.x as u32,
            window_config.size.y as u32,
        ))
        .with_position(monitor.position())
        .with_visible(false);
    let window = window_builder.build(&event_loop).unwrap();
    window.set_fullscreen(fullscreen);

    {
        let size = window.inner_size();
        window_config.size = Vec2::new(size.width as f32, size.height as f32);
    }

    let blend_state = wgpu::BlendState::ALPHA_BLENDING;

    let sample_count = 4; // 1 = disable MSAA.

    let mut device = block_on(GraphicsDevice::new(&window));
    let mut renderer = Renderer::new(
        &device.device,
        device.config.format,
        blend_state,
        sample_count,
        renderer_config.clear_color,
    );
    let mut camera = Camera::new(device.size.width as f32, device.size.height as f32);

    let mut input_helper = WinitInputHelper::new();

    let mut ctx = Context {
        window_title: window_config.title,
        window_size: window_config.size,
    };

    game.on_create(&mut ctx);

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;
    let mut fps = 0_u32;
    let mut update_count: u32 = 0;
    let mut ups = 0_u32;

    window.request_redraw();
    window.set_visible(true);

    let mut t = Instant::now();
    let dt = Duration::from_secs_f32(1.0 / 100.0);
    let mut current_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let mut new_frame = true;

    event_loop.run(move |event, _, control_flow| {
        if new_frame {
            let new_time = Instant::now();
            let mut frame_time = new_time.saturating_duration_since(current_time);
            if frame_time > Duration::from_secs_f32(1.0 / 40.0) {
                // If the frame rate dropped below 40 FPS, cap duration at 40 FPS.
                frame_time = Duration::from_secs_f32(1.0 / 40.0);
            }
            println!(
                "Frame time: {} (dt: {})",
                frame_time.as_secs_f32(),
                dt.as_secs_f32()
            );
            current_time = new_time;

            accumulator += frame_time;

            new_frame = false;
        }

        control_flow.set_poll();

        //////////////////// INPUT ////////////////////
        if !input_helper.update(&event) {
            // MainEventsCleared has not been emitted
            return;
        }

        if let Some(physical) = input_helper.window_resized() {
            ctx.window_size.x = physical.width as f32;
            ctx.window_size.y = physical.height as f32;

            device.resize(physical);
            renderer.resize(&device);
            camera.resize(physical.width as f32, physical.height as f32);
        }

        //////////////////// UPDATE ////////////////////
        let mut scene = Scene::default();
        let input = InputHelper::new(&input_helper);
        while accumulator >= dt {
            if !game.on_update(&mut scene, &input, &mut ctx, &camera, dt) {
                control_flow.set_exit();
                return;
            }
            t += dt;
            accumulator = accumulator.saturating_sub(dt);
            update_count += 1;
        }

        //////////////////// RENDER ////////////////////
        // TODO: Timing if not using vSync (which we are currently).
        let unaligned_indices_len = scene.indices.len();
        for _ in 0..(unaligned_indices_len % COPY_BUFFER_ALIGNMENT as usize) {
            scene.indices.push(0);
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
            contents: bytemuck::cast_slice(&scene.vertices),
            usage: BufferUsages::COPY_SRC,
        });

        let index_buffer = device.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("frame geometry ibo"),
            contents: bytemuck::cast_slice(&scene.indices),
            usage: BufferUsages::COPY_SRC,
        });

        if scene.vertices.len() > renderer.max_geometry_vertices
            || scene.indices.len() > renderer.max_geometry_indices
        {
            renderer.resize_geometry_buffers(
                &device.device,
                scene.vertices.len(),
                scene.indices.len(),
            )
        }

        encoder.copy_buffer_to_buffer(
            &vertex_buffer,
            0,
            &renderer.geometry_vbo,
            0,
            (std::mem::size_of::<Vertex>() * scene.vertices.len()) as BufferAddress,
        );

        encoder.copy_buffer_to_buffer(
            &index_buffer,
            0,
            &renderer.geometry_ibo,
            0,
            (std::mem::size_of::<u16>() * scene.indices.len()) as BufferAddress,
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
        window.request_redraw();

        frame_count += 1;
        let now = Instant::now();
        if now >= next_report {
            fps = frame_count;
            ups = update_count;
            frame_count = 0;
            update_count = 0;
            next_report = now + Duration::from_secs(1);
        }
        window.set_title(&format!(
            "{} | FPS: {}, UPS: {}",
            ctx.window_title, fps, ups
        ));

        new_frame = true;
    });
}

#[derive(Debug)]
pub struct Scene {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Default for Scene {
    fn default() -> Self {
        let vertices = Vec::new();
        let indices = Vec::new();

        Self { vertices, indices }
    }
}
