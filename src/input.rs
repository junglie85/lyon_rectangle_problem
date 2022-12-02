use glam::{Vec2, Vec4, Vec4Swizzles};
use winit_input_helper::WinitInputHelper;

use crate::camera::Camera;

pub struct InputHelper<'frame> {
    winit_helper: &'frame WinitInputHelper,
}

impl<'frame> InputHelper<'frame> {
    pub(crate) fn new(winit_helper: &'frame WinitInputHelper) -> Self {
        Self { winit_helper }
    }

    pub fn quit(&self) -> bool {
        self.winit_helper.quit()
    }

    // TODO: Own key codes
    pub fn key_pressed(&self, key: winit::event::VirtualKeyCode) -> bool {
        self.winit_helper.key_pressed(key)
    }

    pub fn key_released(&self, key: winit::event::VirtualKeyCode) -> bool {
        self.winit_helper.key_released(key)
    }

    // TODO: Own mouse codes
    pub fn mouse_pressed(&self, button: usize) -> bool {
        self.winit_helper.mouse_released(button)
    }

    pub fn mouse_in_viewport(&self) -> Vec2 {
        let (x, y) = self.winit_helper.mouse().unwrap_or_default();

        Vec2::new(x, y)
    }

    pub fn mouse_in_world(&self, camera: &Camera) -> Vec2 {
        let viewport_position = self.mouse_in_viewport();
        let viewport_dimensions = Vec2::new(camera.width(), camera.height());
        let mut ndc = viewport_position / viewport_dimensions * 2.0 - 1.0;
        ndc.y *= -1.0;
        let ndc = Vec4::from((ndc, 1.0, 1.0));

        let inverse_projection = camera.get_projection().inverse();
        let inverse_view = camera.get_view().inverse();

        let mouse_position = inverse_view * inverse_projection * ndc;

        mouse_position.xy()
    }
}
