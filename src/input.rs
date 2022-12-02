use glam::{Vec2, Vec4, Vec4Swizzles};
use winit_input_helper::WinitInputHelper;

use crate::{
    camera::Camera,
    components::{compute_transformation_matrix, Transform},
};

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

    pub fn mouse_in_screen(&self) -> Vec2 {
        let (x, y) = self.winit_helper.mouse().unwrap_or_default();

        Vec2::new(x, y)
    }

    pub fn mouse_in_world(&self, camera: &Camera) -> Vec2 {
        let Vec2 { x, y } = self.mouse_in_screen();
        let mouse_transform = Transform::from_position(x, camera.height() - y);

        // TODO: ndc * inverse view * inverse projection.
        let mouse_position = camera.get_view().inverse()
            * compute_transformation_matrix(&mouse_transform)
            * Vec4::new(0.0, 0.0, 0.0, 1.0);

        mouse_position.xy()
    }
}
