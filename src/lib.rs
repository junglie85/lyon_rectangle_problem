use std::time::{Duration, Instant};

pub use env_logger::init as init_logger;
use futures::executor::block_on;
use glam::Vec2;
use hecs::World;
use renderer::{Bananas, Renderer};
use shape::{Color, Drawable, Style, Transform};
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
mod renderer;
mod shape;

pub fn start() {
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

    let mut world = World::new();

    let mut camera = Camera::new(bananas.size.width as f32, bananas.size.height as f32);

    let mut transform = Transform::default();
    transform.position = Vec2::new(200.0, 200.0);
    transform.size = Vec2::new(200.0, 200.0);
    transform.rotation = 30.0;
    transform.origin = Vec2::new(100.0, 100.0);
    let mut drawable = Drawable::default();
    drawable.style = Style::Rect;
    drawable.z_index = 1;
    drawable.fill_color = Color::WHITE;
    drawable.outline_width = 1.0;
    drawable.outline_color = Color::BLACK;
    world.spawn((transform, drawable));

    let mut transform = Transform::default();
    transform.position = Vec2::new(400.0, 400.0);
    transform.size = Vec2::new(300.0, 200.0);
    transform.rotation = 0.0;
    transform.origin = Vec2::new(0.0, 0.0);
    let mut drawable = Drawable::default();
    drawable.style = Style::Rect;
    drawable.z_index = 1;
    drawable.fill_color = Color::BLACK;
    drawable.outline_width = 5.0;
    drawable.outline_color = Color::WHITE;
    world.spawn((transform, drawable));

    let mut transform = Transform::default();
    transform.position = Vec2::new(395.0, 389.0);
    transform.size = Vec2::new(5.0, 5.0);
    transform.rotation = 0.0;
    transform.origin = Vec2::new(0.0, 0.0);
    let mut drawable = Drawable::default();
    drawable.style = Style::Rect;
    drawable.z_index = 1;
    drawable.fill_color = Color::WHITE;
    drawable.outline_width = 0.0;
    drawable.outline_color = Color::WHITE;
    world.spawn((transform, drawable));

    let mut transform = Transform::default();
    transform.position = Vec2::new(389.0, 395.0);
    transform.size = Vec2::new(5.0, 5.0);
    transform.rotation = 0.0;
    transform.origin = Vec2::new(0.0, 0.0);
    let mut drawable = Drawable::default();
    drawable.style = Style::Rect;
    drawable.z_index = 1;
    drawable.fill_color = Color::WHITE;
    drawable.outline_width = 0.0;
    drawable.outline_color = Color::WHITE;
    world.spawn((transform, drawable));

    let mut transform = Transform::default();
    transform.position = Vec2::new(401.0, 401.0);
    transform.size = Vec2::new(5.0, 5.0);
    transform.rotation = 0.0;
    transform.origin = Vec2::new(0.0, 0.0);
    let mut drawable = Drawable::default();
    drawable.style = Style::Rect;
    drawable.z_index = 2;
    drawable.fill_color = Color::WHITE;
    drawable.outline_width = 0.0;
    drawable.outline_color = Color::WHITE;
    world.spawn((transform, drawable));

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

        //////////////////// RENDER ////////////////////
        if !state.render {
            return;
        }

        state.render = false;

        let mut scene = renderer.begin_scene(&camera);

        // renderer.draw_shape(&mut scene, &pixel_measure_1);
        // renderer.draw_shape(&mut scene, &pixel_measure_2);
        // renderer.draw_shape(&mut scene, &pixel_measure_3);
        // renderer.draw_shape(&mut scene, &bottom_left);
        // renderer.draw_shape(&mut scene, &top_right);
        for (_id, (transform, drawable)) in world.query::<(&Transform, &Drawable)>().iter() {
            renderer.draw(&mut scene, transform, drawable);
        }

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
