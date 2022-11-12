#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub origin: [f32; 2],
    pub z_index: i32,
    pub fill_color: [f32; 4],
    pub stroke_width: f32,
    pub stroke_color: [f32; 4],
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
            stroke_width: 0.0,
            stroke_color: [1.0; 4],
        }
    }
}
