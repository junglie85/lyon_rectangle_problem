use glam::Vec2;
use lyon::{
    geom::{point, Box2D, LineSegment},
    lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertexConstructor, StrokeOptions,
        StrokeTessellator, StrokeVertexConstructor, VertexBuffers,
    },
    path::{Path, Polygon, Winding},
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

        if outline_thickness > 0.0 {
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

    fn tesselate_line(
        &mut self,
        path: &Path,
        outline_color: Color,
        outline_thickness: f32,
        geometry: &mut VertexBuffers<GeometryVertex, u16>,
    ) {
        if outline_thickness > 0.0 {
            self.stroke_tess
                .tessellate_path(
                    path,
                    &StrokeOptions::tolerance(self.tolerance()).with_line_width(outline_thickness),
                    &mut BuffersBuilder::new(geometry, GeometryVertexLineCtor(outline_color))
                        .with_inverted_winding(),
                )
                .unwrap();
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

pub struct GeometryVertexLineCtor(Color);

impl StrokeVertexConstructor<GeometryVertex> for GeometryVertexLineCtor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> GeometryVertex {
        let pos = vertex.position();
        GeometryVertex {
            position: Vec2::new(pos.x, pos.y),
            color: self.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircleShape {
    pub radius: f32,
    pub fill_color: Color,
    pub outline_thickness: f32,
    pub outline_color: Color,
    geometry: VertexBuffers<GeometryVertex, u16>,
}

impl Default for CircleShape {
    fn default() -> Self {
        let geometry = VertexBuffers::new();

        Self {
            radius: 0.0,
            fill_color: Color::WHITE,
            outline_thickness: 0.0,
            outline_color: Color::WHITE,
            geometry,
        }
    }
}

impl CircleShape {
    pub fn vertices(&self) -> &[GeometryVertex] {
        &self.geometry.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.geometry.indices
    }
}

impl Geometry for CircleShape {
    fn update(&mut self, tesselator: &mut Tesselator) {
        let mut builder = Path::builder();
        builder.add_circle(
            point(self.radius, self.radius),
            self.radius,
            Winding::Positive,
        );
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

#[derive(Debug, Clone)]
pub struct LineShape {
    pub length: f32,
    pub angle: f32,
    pub outline_thickness: f32,
    pub outline_color: Color,
    geometry: VertexBuffers<GeometryVertex, u16>,
}

impl Default for LineShape {
    fn default() -> Self {
        let geometry = VertexBuffers::new();

        Self {
            length: 0.0,
            angle: 0.0,
            outline_thickness: 0.0,
            outline_color: Color::WHITE,
            geometry,
        }
    }
}

impl LineShape {
    pub fn vertices(&self) -> &[GeometryVertex] {
        &self.geometry.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.geometry.indices
    }
}

impl Geometry for LineShape {
    fn update(&mut self, tesselator: &mut Tesselator) {
        let from = point(0.0, 0.0);

        // Position on circumference = (x + r*cos(a), y + r*sin(a))
        // Where (x, y) is the center of the circle.
        let a = (-self.angle).to_radians() + 90.0_f32.to_radians();
        let to = point(
            from.x + self.length * a.cos(),
            from.y + self.length * a.sin(),
        );

        let mut builder = Path::builder();
        let line = LineSegment { from, to };
        builder.add_line_segment(&line);
        let path = builder.build();

        tesselator.tesselate_line(
            &path,
            self.outline_color,
            self.outline_thickness,
            &mut self.geometry,
        );
    }
}

#[derive(Debug, Clone)]
pub struct PolygonShape {
    pub radius: f32,
    pub point_count: u32,
    pub fill_color: Color,
    pub outline_thickness: f32,
    pub outline_color: Color,
    geometry: VertexBuffers<GeometryVertex, u16>,
}

impl Default for PolygonShape {
    fn default() -> Self {
        let geometry = VertexBuffers::new();

        Self {
            radius: 0.0,
            point_count: 3,
            fill_color: Color::WHITE,
            outline_thickness: 0.0,
            outline_color: Color::WHITE,
            geometry,
        }
    }
}

impl PolygonShape {
    pub fn vertices(&self) -> &[GeometryVertex] {
        &self.geometry.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.geometry.indices
    }
}

impl Geometry for PolygonShape {
    fn update(&mut self, tesselator: &mut Tesselator) {
        if self.point_count >= 3 {
            let points = (0..self.point_count)
                .map(|i| {
                    // Position on circumference = (x + r*cos(a), y + r*sin(a))
                    // Where (x, y) is the center of the circle.
                    let r = self.radius;
                    let a = i as f32 / self.point_count as f32 * 360.0_f32.to_radians()
                        + 90.0_f32.to_radians();
                    point(r + r * a.cos(), r + r * a.sin())
                })
                .collect::<Vec<_>>();

            let mut builder = Path::builder();
            let polygon = Polygon {
                points: &points,
                closed: true,
            };
            builder.add_polygon(polygon);
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
}

#[derive(Debug, Clone)]
pub struct RectangleShape {
    pub size: Vec2,
    pub fill_color: Color,
    pub outline_thickness: f32,
    pub outline_color: Color,
    geometry: VertexBuffers<GeometryVertex, u16>,
}

impl Default for RectangleShape {
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

impl RectangleShape {
    pub fn vertices(&self) -> &[GeometryVertex] {
        &self.geometry.vertices
    }

    pub fn indices(&self) -> &[u16] {
        &self.geometry.indices
    }
}

impl Geometry for RectangleShape {
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
