use glam::Vec2;

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub origin: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            size: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            origin: Vec2::new(0.0, 0.0),
        }
    }
}

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

#[derive(Debug, Copy, Clone)]
pub enum Style {
    Rect,
    // Circle(radius_on_width_or_height),
    // Triangle,
    // Polygon(point_count),
    // Sprite(metadata),
}

#[derive(Debug, Copy, Clone)]
pub struct Drawable {
    pub style: Style,
    pub z_index: i32,
    pub fill_color: Color,
    pub outline_width: f32,
    pub outline_color: Color,
}

impl Default for Drawable {
    fn default() -> Self {
        Self {
            z_index: 0,
            fill_color: Color::WHITE,
            outline_width: 0.0,
            outline_color: Color::WHITE,
            style: Style::Rect,
        }
    }
}
