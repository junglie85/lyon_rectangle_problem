use glam::Mat4;

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
