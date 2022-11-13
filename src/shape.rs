use crate::renderer::GeometryPrimitive;

pub trait Shape {
    fn fill_primitive(&self, primitive: &mut GeometryPrimitive);

    fn stroke_primitive(&self, primitive: &mut GeometryPrimitive);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub origin: [f32; 2],
    pub z_index: i32,
    pub fill_color: [f32; 4],
    pub outline_width: f32,
    pub outline_color: [f32; 4],
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            position: [0.0; 2],
            size: [1.0; 2],
            rotation: 0.0,
            origin: [0.0; 2],
            z_index: 0,
            fill_color: [1.0; 4],
            outline_width: 0.0,
            outline_color: [1.0; 4],
        }
    }
}

impl Shape for Rect {
    fn fill_primitive(&self, primitive: &mut GeometryPrimitive) {
        let rotation = (-self.rotation).to_radians();

        primitive.color = self.fill_color;
        primitive.translate = self.position;
        primitive.rotate = rotation;
        primitive.scale = self.size;
        primitive.origin = self.origin;
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

        primitive.color = self.outline_color;
        primitive.translate = outline_position;
        primitive.rotate = rotation;
        primitive.scale = outline_size;
        primitive.origin = self.origin;
        primitive.z_index = GeometryPrimitive::STROKE_Z_INDEX + self.z_index;
        primitive.width = outline_width;
    }
}
