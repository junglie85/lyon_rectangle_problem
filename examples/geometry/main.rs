use glam::Vec2;
use hecs::World;
use papercut::{
    components::{Drawable, Transform},
    graphics::{
        CircleShape, Color, Geometry, LineShape, PolygonShape, RectangleShape, Tessellator,
    },
    EngineSettings,
};

fn main() {
    papercut::init_logger();
    papercut::start::<GeometryExample>();
}

#[derive(Default)]
struct GeometryExample;

impl papercut::Game for GeometryExample {
    fn post_init(&mut self, world: &mut World, _settings: &EngineSettings) {
        let tolerance = 0.02;
        let mut tessellator = Tessellator::new(tolerance);

        let mut transform = Transform::default();
        transform.translation = Vec2::new(200.0, 200.0);
        transform.origin = Vec2::new(100.0, 100.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(200.0, 200.0);
        rect.fill_color = Color::WHITE;
        rect.outline_thickness = 1.0;
        rect.outline_color = Color::BLACK;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(200.0, 200.0);
        transform.rotation = 30.0;
        transform.origin = Vec2::new(100.0, 100.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(200.0, 200.0);
        rect.fill_color = Color::WHITE;
        rect.outline_thickness = 1.0;
        rect.outline_color = Color::BLACK;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(400.0, 400.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(300.0, 150.0);
        rect.fill_color = Color::BLACK;
        rect.outline_thickness = 5.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        // Bottom left
        let mut transform = Transform::default();
        transform.translation = Vec2::new(400.0, 405.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(1.0, 0.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(405.0, 400.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 0.0, 1.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(405.0, 405.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        // Top left
        let mut transform = Transform::default();
        transform.translation = Vec2::new(400.0, 540.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(1.0, 0.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(405.0, 545.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 0.0, 1.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(405.0, 540.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        // Bottom right
        let mut transform = Transform::default();
        transform.translation = Vec2::new(695.0, 405.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(1.0, 0.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(690.0, 400.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 0.0, 1.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(690.0, 405.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        // Top right
        let mut transform = Transform::default();
        transform.translation = Vec2::new(695.0, 540.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(1.0, 0.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(690.0, 545.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 0.0, 1.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(690.0, 540.0);
        let mut rect = RectangleShape::default();
        rect.size = Vec2::new(5.0, 5.0);
        rect.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        rect.outline_thickness = 0.0;
        rect.outline_color = Color::WHITE;
        rect.update(&mut tessellator);
        let drawable = Drawable::Rect(rect);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(400.0, 100.0);
        transform.origin = Vec2::new(100.0, 100.0);
        let mut circle = CircleShape::default();
        circle.radius = 100.0;
        circle.fill_color = Color::new(0.0, 0.0, 1.0, 1.0);
        circle.outline_thickness = 10.0;
        circle.outline_color = Color::new(1.0, 1.0, 0.0, 1.0);
        circle.update(&mut tessellator);
        let drawable = Drawable::Circle(circle);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(120.0, 450.0);
        transform.origin = Vec2::new(100.0, 100.0);
        let mut polygon = PolygonShape::default();
        polygon.radius = 100.0;
        polygon.point_count = 5;
        polygon.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        polygon.outline_thickness = 10.0;
        polygon.outline_color = Color::new(1.0, 0.0, 0.0, 1.0);
        polygon.update(&mut tessellator);
        let drawable = Drawable::Polygon(polygon);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(900.0, 550.0);
        transform.rotation = 90.0;
        let mut polygon = PolygonShape::default();
        polygon.radius = 50.0;
        polygon.point_count = 3;
        polygon.fill_color = Color::new(0.0, 1.0, 0.0, 1.0);
        polygon.outline_thickness = 2.0;
        polygon.outline_color = Color::new(1.0, 0.0, 0.0, 1.0);
        polygon.update(&mut tessellator);
        let drawable = Drawable::Polygon(polygon);
        world.spawn((transform, drawable));

        let mut transform = Transform::default();
        transform.translation = Vec2::new(400.0, 100.0);
        let mut line = LineShape::default();
        line.length = 100.0;
        line.angle = 60.0;
        line.outline_thickness = 10.0;
        line.outline_color = Color::new(1.0, 1.0, 0.0, 1.0);
        line.update(&mut tessellator);
        let drawable = Drawable::Line(line);
        world.spawn((transform, drawable));
    }
}
