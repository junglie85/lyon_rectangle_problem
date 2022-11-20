use glam::{Mat4, Vec2, Vec3};

use crate::graphics::{CircleShape, PolygonShape, RectangleShape};

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub origin: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.0, 0.0),
        }
    }
}

pub fn compute_transformation_matrix(t: &Transform) -> Mat4 {
    let mut transform = Mat4::from_translation(Vec3::from((t.translation - t.origin, 0.0)));

    transform *= Mat4::from_translation(Vec3::from((t.origin, 0.0)));
    transform *= Mat4::from_rotation_z(-t.rotation.to_radians());
    transform *= Mat4::from_translation(Vec3::from((-t.origin, 0.0)));

    transform *= Mat4::from_scale(Vec3::new(t.scale.x, t.scale.y, 0.0));
    transform
}

#[derive(Debug, Clone)]
pub enum Drawable {
    Circle(CircleShape),
    Polygon(PolygonShape),
    Rect(RectangleShape),
}
