use futures::executor::block_on;
use renderer::{Bananas, Renderer};
use shape::Rect;
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
mod shape;

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
        let mut bottom_left = Rect::default();
        bottom_left.position = [200.0, 200.0];
        bottom_left.size = [200.0, 200.0];
        bottom_left.rotation = 30.0;
        bottom_left.origin = [100.0, 100.0];
        bottom_left.z_index = 1;
        bottom_left.fill_color = [1.0, 1.0, 1.0, 1.0];
        bottom_left.outline_width = 1.0;
        bottom_left.outline_color = [0.0, 0.0, 0.0, 1.0];

        let mut top_right = Rect::default();
        top_right.position = [400.0, 400.0];
        top_right.size = [300.0, 200.0];
        top_right.rotation = 0.0;
        top_right.origin = [0.0, 0.0];
        top_right.z_index = 1;
        top_right.fill_color = [0.0, 0.0, 0.0, 1.0];
        top_right.outline_width = 5.0;
        top_right.outline_color = [1.0, 1.0, 1.0, 1.0];

        let mut pixel_measure_1 = Rect::default();
        pixel_measure_1.position = [395.0, 389.0];
        pixel_measure_1.size = [5.0, 5.0];
        pixel_measure_1.rotation = 0.0;
        pixel_measure_1.origin = [0.0, 0.0];
        pixel_measure_1.z_index = 1;
        pixel_measure_1.fill_color = [1.0, 1.0, 1.0, 1.0];
        pixel_measure_1.outline_width = 0.0;
        pixel_measure_1.outline_color = [1.0, 1.0, 1.0, 1.0];

        let mut pixel_measure_2 = Rect::default();
        pixel_measure_2.position = [389.0, 395.0];
        pixel_measure_2.size = [5.0, 5.0];
        pixel_measure_2.rotation = 0.0;
        pixel_measure_2.origin = [0.0, 0.0];
        pixel_measure_2.z_index = 1;
        pixel_measure_2.fill_color = [1.0, 1.0, 1.0, 1.0];
        pixel_measure_2.outline_width = 0.0;
        pixel_measure_2.outline_color = [1.0, 1.0, 1.0, 1.0];

        let mut pixel_measure_3 = Rect::default();
        pixel_measure_3.position = [401.0, 401.0];
        pixel_measure_3.size = [5.0, 5.0];
        pixel_measure_3.rotation = 0.0;
        pixel_measure_3.origin = [0.0, 0.0];
        pixel_measure_3.z_index = 1;
        pixel_measure_3.fill_color = [1.0, 1.0, 1.0, 1.0];
        pixel_measure_3.outline_width = 0.0;
        pixel_measure_3.outline_color = [1.0, 1.0, 1.0, 1.0];

        //////////////////// RENDER ////////////////////
        if !scene.render {
            return;
        }

        scene.render = false;

        let clear_color = wgpu::Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };

        renderer.begin_scene();

        renderer.draw_shape(&pixel_measure_1);
        renderer.draw_shape(&pixel_measure_2);
        renderer.draw_shape(&pixel_measure_3);
        renderer.draw_shape(&bottom_left);
        renderer.draw_shape(&top_right);

        let primitives = renderer.get_primitives();

        renderer.render(&bananas, &camera, clear_color, &primitives);

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
