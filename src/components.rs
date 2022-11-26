use glam::{Mat4, Vec2, Vec3};

use crate::graphics::{CircleShape, LineShape, PolygonShape, RectangleShape};

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
}

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

impl Transform {
    pub fn from_position(x: f32, y: f32) -> Self {
        let mut transform = Self::default();
        transform.translation.x = x;
        transform.translation.y = y;

        transform
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

pub fn compute_inverse_transformation_matrix(t: &Transform) -> Mat4 {
    let mut transform = Mat4::from_scale(Vec3::from((1.0 / t.scale, 0.0)));
    transform *= Mat4::from_rotation_z(t.rotation.to_radians());
    transform *= Mat4::from_translation(Vec3::from((t.origin - t.translation, 0.0)));
    transform
}

#[derive(Debug, Clone)]
pub enum Drawable {
    Circle(CircleShape),
    Line(LineShape),
    Polygon(PolygonShape),
    Rect(RectangleShape),
}
