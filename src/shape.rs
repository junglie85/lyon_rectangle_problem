use glam::Vec2;

use crate::renderer::GeometryPrimitive;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub trait Shape {
    fn fill_primitive(&self, primitive: &mut GeometryPrimitive);

    fn stroke_primitive(&self, primitive: &mut GeometryPrimitive);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32, // TODO: Quaternion
    pub origin: Vec2,
    pub z_index: i32,
    pub fill_color: Color,
    pub outline_width: f32,
    pub outline_color: Color,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            origin: Vec2::new(0.0, 0.0),
            z_index: 0,
            fill_color: Color::WHITE,
            outline_width: 0.0,
            outline_color: Color::WHITE,
        }
    }
}

impl Shape for Rect {
    fn fill_primitive(&self, primitive: &mut GeometryPrimitive) {
        let rotation = (-self.rotation).to_radians();

        primitive.color = self.fill_color.to_array();
        primitive.translate = self.position.to_array();
        primitive.rotate = rotation;
        primitive.scale = self.size.to_array();
        primitive.origin = self.origin.to_array();
        primitive.z_index = GeometryPrimitive::FILL_Z_INDEX + self.z_index;
        primitive.width = primitive.width;
    }

    fn stroke_primitive(&self, primitive: &mut GeometryPrimitive) {
        let rotation = (-self.rotation).to_radians();

        // The outline_width is applied in the geometry shader, but it scales the vertex positions
        // which results in the outline being twice the desired width, so here we halve the outline_width.
        // The size and position of the outline are adjusted to account for this and ensure the total
        // rendered size is `2*rect.outline_width + rect.size`, with the outline offset at
        // `position - outline_width`.
        let outline_width = self.outline_width * 0.5;
        let outline_size = [
            self.size[0] + self.outline_width,
            self.size[1] + self.outline_width,
        ];
        let outline_position = [
            self.position[0] - outline_width,
            self.position[1] - outline_width,
        ];

        primitive.color = self.outline_color.to_array();
        primitive.translate = outline_position;
        primitive.rotate = rotation;
        primitive.scale = outline_size;
        primitive.origin = self.origin.to_array();
        primitive.z_index = GeometryPrimitive::STROKE_Z_INDEX + self.z_index;
        primitive.width = outline_width;
    }
}
