use futures::executor::block_on;
use glam::Vec2;
use renderer::{Bananas, Renderer};
use shape::{Color, Rect};
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

pub fn start() {
    env_logger::init();

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

    let clear_color = Color::new(1.0, 0.0, 1.0, 1.0);
    let sample_count = 4; // 1 = disable MSAA.

    let mut bananas = block_on(Bananas::new(&window));
    let mut renderer = Renderer::new(
        &bananas.device,
        bananas.config.format,
        blend_state,
        sample_count,
        clear_color,
    );

    let mut camera = Camera::new(bananas.size.width as f32, bananas.size.height as f32);

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
            bananas.resize(physical);
            renderer.resize(&bananas);
            camera.resize(physical.width as f32, physical.height as f32);
        }

        //////////////////// UPDATE ////////////////////
        let mut bottom_left = Rect::default();
        bottom_left.position = Vec2::new(200.0, 200.0);
        bottom_left.size = Vec2::new(200.0, 200.0);
        bottom_left.rotation = 30.0;
        bottom_left.origin = Vec2::new(100.0, 100.0);
        bottom_left.z_index = 1;
        bottom_left.fill_color = Color::WHITE;
        bottom_left.outline_width = 1.0;
        bottom_left.outline_color = Color::BLACK;

        let mut top_right = Rect::default();
        top_right.position = Vec2::new(400.0, 400.0);
        top_right.size = Vec2::new(300.0, 200.0);
        top_right.rotation = 0.0;
        top_right.origin = Vec2::new(0.0, 0.0);
        top_right.z_index = 1;
        top_right.fill_color = Color::BLACK;
        top_right.outline_width = 5.0;
        top_right.outline_color = Color::WHITE;

        let mut pixel_measure_1 = Rect::default();
        pixel_measure_1.position = Vec2::new(395.0, 389.0);
        pixel_measure_1.size = Vec2::new(5.0, 5.0);
        pixel_measure_1.rotation = 0.0;
        pixel_measure_1.origin = Vec2::new(0.0, 0.0);
        pixel_measure_1.z_index = 1;
        pixel_measure_1.fill_color = Color::WHITE;
        pixel_measure_1.outline_width = 0.0;
        pixel_measure_1.outline_color = Color::WHITE;

        let mut pixel_measure_2 = Rect::default();
        pixel_measure_2.position = Vec2::new(389.0, 395.0);
        pixel_measure_2.size = Vec2::new(5.0, 5.0);
        pixel_measure_2.rotation = 0.0;
        pixel_measure_2.origin = Vec2::new(0.0, 0.0);
        pixel_measure_2.z_index = 1;
        pixel_measure_2.fill_color = Color::WHITE;
        pixel_measure_2.outline_width = 0.0;
        pixel_measure_2.outline_color = Color::WHITE;

        let mut pixel_measure_3 = Rect::default();
        pixel_measure_3.position = Vec2::new(401.0, 401.0);
        pixel_measure_3.size = Vec2::new(5.0, 5.0);
        pixel_measure_3.rotation = 0.0;
        pixel_measure_3.origin = Vec2::new(0.0, 0.0);
        pixel_measure_3.z_index = 1;
        pixel_measure_3.fill_color = Color::WHITE;
        pixel_measure_3.outline_width = 0.0;
        pixel_measure_3.outline_color = Color::WHITE;

        //////////////////// RENDER ////////////////////
        if !state.render {
            return;
        }

        state.render = false;

        let mut scene = renderer.begin_scene(&camera);

        renderer.draw_shape(&mut scene, &pixel_measure_1);
        renderer.draw_shape(&mut scene, &pixel_measure_2);
        renderer.draw_shape(&mut scene, &pixel_measure_3);
        renderer.draw_shape(&mut scene, &bottom_left);
        renderer.draw_shape(&mut scene, &top_right);

        renderer.end_scene(scene, &bananas);

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
