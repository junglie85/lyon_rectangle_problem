use futures::executor::block_on;
use renderer::{Bananas, Globals, Primitive, Renderer, PRIMITIVES_BUFFER_LEN};
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::camera::Camera;

const ASPECT_RATIO: f32 = 16_f32 / 9_f32;
pub const DEFAULT_WINDOW_WIDTH: f32 = 1024.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = DEFAULT_WINDOW_WIDTH as f32 / ASPECT_RATIO;

mod camera;
mod renderer;

fn main() {
    env_logger::init();

    let mut scene = SceneParams {
        window_size: PhysicalSize::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
        size_changed: true,
        render: false,
    };

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_inner_size(scene.window_size);
    let window = window_builder.build(&event_loop).unwrap();

    let blend_state = wgpu::BlendState {
        color: wgpu::BlendComponent::REPLACE,
        alpha: wgpu::BlendComponent::REPLACE,
    };

    let sample_count = 4; // 1 = disable MSAA.

    let mut bananas = block_on(Bananas::new(&window));
    let mut renderer = Renderer::new(
        &bananas.device,
        bananas.config.format,
        blend_state,
        sample_count,
    );

    let mut camera = Camera::new(bananas.size.width as f32, bananas.size.height as f32);

    let start = Instant::now();
    let mut next_report = start + Duration::from_secs(1);
    let mut frame_count: u32 = 0;

    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        //////////////////// INPUT ////////////////////
        if !process_event(event, &window, control_flow, &mut scene) {
            // keep polling inputs.
            return;
        }

        if scene.size_changed {
            scene.size_changed = false;

            let physical = scene.window_size;
            bananas.resize(physical);
            renderer.resize(&bananas);
            camera.resize(physical.width as f32, physical.height as f32);
        }

        //////////////////// UPDATE ////////////////////
        let mut primitives = vec![Primitive::default(); PRIMITIVES_BUFFER_LEN];
        // Bottom left square
        primitives[renderer.stroke_id as usize] = Primitive {
            color: [0.0, 0.0, 0.0, 1.0],
            scale: [200.0, 200.0],
            z_index: 2,
            width: 0.5,
            ..Primitive::default()
        };
        primitives[renderer.fill_id as usize] = Primitive {
            color: [1.0, 1.0, 1.0, 1.0],
            scale: [200.0, 200.0],
            z_index: 1,
            ..Primitive::default()
        };

        // Top right rectangle
        primitives[renderer.stroke_id as usize + 2] = Primitive {
            color: [1.0, 0.0, 0.0, 1.0],
            translate: [400.0, 400.0],
            scale: [300.0, 200.0],
            z_index: 2,
            width: 2.5,
            ..Primitive::default()
        };
        primitives[renderer.fill_id as usize + 2] = Primitive {
            color: [0.0, 1.0, 0.0, 1.0],
            translate: [400.0, 400.0],
            scale: [300.0, 200.0],
            z_index: 1,
            ..Primitive::default()
        };

        // Thingy rectangle
        primitives[renderer.stroke_id as usize + 4] = Primitive {
            color: [1.0, 0.0, 0.0, 1.0],
            translate: [0.0, 0.0],
            scale: [0.0, 0.0],
            z_index: 2,
            width: 0.0,
            ..Primitive::default()
        };
        primitives[renderer.fill_id as usize + 4] = Primitive {
            color: [1.0, 1.0, 1.0, 1.0],
            translate: [397.5, 390.0],
            scale: [5.0, 10.0],
            z_index: 1,
            ..Primitive::default()
        };

        //////////////////// RENDER ////////////////////
        if !scene.render {
            return;
        }

        scene.render = false;

        let frame = match bananas.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                println!("swapchain error: {:?}", e);
                return;
            }
        };

        let render_target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = bananas
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let globals = Globals {
            view: camera.get_view().to_cols_array_2d(),
            projection: camera.get_projection().to_cols_array_2d(),
        };

        bananas
            .queue
            .write_buffer(&renderer.globals_ubo, 0, bytemuck::cast_slice(&[globals]));
        bananas.queue.write_buffer(
            &renderer.primitives_ubo,
            0,
            bytemuck::cast_slice(&primitives),
        );

        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
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

            pass.draw_indexed(
                renderer.geometry_fill_range.clone(),
                0,
                0..(renderer.instance_count as u32),
            );
            pass.draw_indexed(
                renderer.geometry_stroke_range.clone(),
                0,
                0..(renderer.instance_count as u32),
            );
        }

        bananas.queue.submit(Some(encoder.finish()));
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
