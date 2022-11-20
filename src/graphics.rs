use glam::Vec2;
use lyon::{
    geom::{point, Box2D},
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertexConstructor, StrokeOptions,
        StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
    },
    path::{Path, Winding},
};

pub struct Tesselator {
    tolerance: f32,
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
}

impl Tesselator {
    pub fn new(tolerance: f32) -> Self {
        let fill_tess = FillTessellator::new();
        let stroke_tess = StrokeTessellator::new();

        Self {
            tolerance,
            fill_tess,
            stroke_tess,
        }
    }

    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    fn tesselate(
        &mut self,
        path: &Path,
        fill_color: Color,
        outline_color: Color,
        outline_thickness: f32,
        geometry: &mut VertexBuffers<GeometryVertex, u16>,
    ) {
        self.fill_tess
            .tessellate_path(
                path,
                &FillOptions::tolerance(self.tolerance)
                    .with_fill_rule(lyon::tessellation::FillRule::NonZero),
                &mut BuffersBuilder::new(geometry, GeometryVertexCtor(fill_color))
                    .with_inverted_winding(),
            )
            .unwrap();

        self.stroke_tess
            .tessellate_path(
                path,
                &StrokeOptions::tolerance(self.tolerance()).with_line_width(outline_thickness),
                &mut BuffersBuilder::new(geometry, GeometryVertexCtor(outline_color))
                    .with_inverted_winding(),
            )
            .unwrap();
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

pub trait Geometry {
    fn update(&mut self, tesselator: &mut Tesselator);
}

#[derive(Copy, Clone, Debug)]
pub struct GeometryVertex {
    position: Vec2,
    color: Color,
}

impl GeometryVertex {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

pub struct GeometryVertexCtor(Color);

impl FillVertexConstructor<GeometryVertex> for GeometryVertexCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> GeometryVertex {
        let pos = vertex.position();
        GeometryVertex {
            position: Vec2::new(pos.x, pos.y),
            color: self.0,
        }
    }
}

impl StrokeVertexConstructor<GeometryVertex> for GeometryVertexCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> GeometryVertex {
        let pos = if vertex.side().is_negative() {
            vertex.position_on_path()
        } else {
            vertex.position_on_path() + vertex.normal() * vertex.line_width()
        };
        GeometryVertex {
            position: Vec2::new(pos.x, pos.y),
            color: self.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub size: Vec2,
    pub fill_color: Color,
    pub outline_thickness: f32,
    pub outline_color: Color,
    geometry: VertexBuffers<GeometryVertex, u16>,
}

impl Default for Rect {
    fn default() -> Self {
        let geometry = VertexBuffers::new();

        Self {
            size: Vec2::new(0.0, 0.0),
            fill_color: Color::WHITE,
            outline_thickness: 0.0,
            outline_color: Color::WHITE,
            geometry,
        }
    }
}

impl Rect {
    pub fn vertices(&self) -> &[GeometryVertex] {
        &self.geometry.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.geometry.indices
    }
}

impl Geometry for Rect {
    fn update(&mut self, tesselator: &mut Tesselator) {
        let rect = Box2D::new(point(0.0, 0.0), point(self.size.x, self.size.y));
        let mut builder = Path::builder();
        builder.add_rectangle(&rect, Winding::Positive);
        let path = builder.build();

        tesselator.tesselate(
            &path,
            self.fill_color,
            self.outline_color,
            self.outline_thickness,
            &mut self.geometry,
        );
    }
}
