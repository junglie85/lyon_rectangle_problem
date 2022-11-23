use glam::Vec2;
use papercut::{
    components::{CircleCollider, Drawable, Input, Physics, Tag, Transform},
    graphics::{Color, Geometry, PolygonShape, Tesselator},
};
use rand::{thread_rng, Rng};

fn main() {
    papercut::init_logger();
    papercut::start::<GeometryWars>();
}

#[derive(Default)]
struct GeometryWars {
    score: u32,
    current_frame: u32,
    last_enemy_spawn_time: u32,
    last_special_weapon_spawn_time: u32,
    paused: bool,
    running: bool,
}

impl papercut::Game for GeometryWars {
    fn pre_init(&self, settings: &mut papercut::EngineSettings) {
        let window_config = WindowConfig {
            width: 1280,
            height: 720,
            frame_limit: 60,
            fullscreen: false,
        };

        let clear_color = Color::new(0.0, 0.0, 0.0, 1.0);

        settings.frame_rate = window_config.frame_limit;
        settings.clear_color = clear_color;
    }

    fn post_init(&self, world: &mut hecs::World, window_size: Vec2) {
        let font_config = FontConfig {
            file: String::from("fonts/arial.ttf"),
            size: 24,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        };

        let player_config = PlayerConfig {
            shape_radius: 32,
            collision_radius: 32,
            speed: 5.0,
            fill_color: Color::new(0.02, 0.02, 0.02, 1.0),
            outline_color: Color::new(1.0, 0.0, 0.0, 1.0),
            outline_thicknes: 4,
            vertices: 8,
        };

        let enemy_config = EnemyConfig {
            shape_radius: 32,
            collision_radius: 32,
            min_speed: 3.0,
            max_speed: 3.0,
            outline_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_thicknes: 2,
            min_vertices: 3,
            max_vertices: 8,
            small_lifespan: 90,
            spawn_interval: 120,
        };

        let bullet_config = BulletConfig {
            shape_radius: 10,
            collision_radius: 10,
            speed: 20.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_thicknes: 2,
            vertices: 20,
            lifespan: 90,
        };

        let mut tessellator = Tesselator::new(0.02);
        spawn_player(world, &player_config, window_size, &mut tessellator);
        spawn_enemy(world, &enemy_config, window_size, &mut tessellator);
    }
}

fn spawn_player(
    world: &mut hecs::World,
    player_config: &PlayerConfig,
    window_size: Vec2,
    tesselator: &mut Tesselator,
) {
    let tag = Tag {
        name: String::from("player"),
    };

    let mut transform = Transform::default();
    transform.translation = Vec2::new(window_size.x / 2.0, window_size.y / 2.0);
    transform.origin = Vec2::new(
        player_config.shape_radius as f32,
        player_config.shape_radius as f32,
    );

    let mut shape = PolygonShape::default();
    shape.radius = player_config.shape_radius as f32;
    shape.point_count = player_config.vertices;
    shape.fill_color = player_config.fill_color;
    shape.outline_color = player_config.outline_color;
    shape.outline_thickness = player_config.outline_thicknes as f32;
    shape.update(tesselator);

    let drawable = Drawable::Polygon(shape);

    let mut collider = CircleCollider::default();
    collider.radius = player_config.collision_radius as f32;

    let mut physics = Physics::default();
    physics.velocity = Vec2::new(player_config.speed, player_config.speed);

    let input = Input::default();

    world.spawn((tag, transform, drawable, collider, physics, input));
}

fn spawn_enemy(
    world: &mut hecs::World,
    enemy_config: &EnemyConfig,
    window_size: Vec2,
    tesselator: &mut Tesselator,
) {
    let mut rng = thread_rng();

    let tag = Tag {
        name: String::from("enemy"),
    };

    let x_range =
        enemy_config.shape_radius as f32..window_size.x - enemy_config.shape_radius as f32;
    let y_range =
        enemy_config.shape_radius as f32..window_size.y - enemy_config.shape_radius as f32;
    let pos_x = rng.gen_range(x_range);
    let pos_y = rng.gen_range(y_range);
    let enemy_position = Vec2::new(pos_x, pos_y);

    let mut transform = Transform::default();
    transform.translation = enemy_position;
    transform.origin = Vec2::new(
        enemy_config.shape_radius as f32,
        enemy_config.shape_radius as f32,
    );

    let vertex_count = rng.gen_range(enemy_config.min_vertices..=enemy_config.max_vertices);
    let r = rng.gen_range(0.0_f32..=1.0_f32);
    let g = rng.gen_range(0.0_f32..=1.0_f32);
    let b = rng.gen_range(0.0_f32..=1.0_f32);
    let fill_color = Color::new(r, g, b, 1.0);

    let mut shape = PolygonShape::default();
    shape.radius = enemy_config.shape_radius as f32;
    shape.point_count = vertex_count;
    shape.fill_color = fill_color;
    shape.outline_color = enemy_config.outline_color;
    shape.outline_thickness = enemy_config.outline_thicknes as f32;
    shape.update(tesselator);

    let drawable = Drawable::Polygon(shape);

    let mut collider = CircleCollider::default();
    collider.radius = enemy_config.collision_radius as f32;

    let mut speed_x = rng.gen_range(enemy_config.min_speed..=enemy_config.max_speed);
    if rng.gen_bool(0.5) {
        speed_x = -speed_x;
    }
    let mut speed_y = rng.gen_range(enemy_config.min_speed..=enemy_config.max_speed);
    if rng.gen_bool(0.5) {
        speed_y = -speed_y;
    }
    let enemy_speed = Vec2::new(speed_x, speed_y);

    let mut physics = Physics::default();
    physics.velocity = enemy_speed;

    let score = Score {
        score: vertex_count * 100,
    };

    world.spawn((tag, transform, drawable, collider, physics, score));

    // TODO: game_state.last_enemy_spawn_time = game_state.current_frame;
}

struct WindowConfig {
    width: u32,
    height: u32,
    frame_limit: u32,
    fullscreen: bool,
}

struct FontConfig {
    file: String,
    size: u32,
    color: Color,
}

struct PlayerConfig {
    shape_radius: u32,
    collision_radius: u32,
    speed: f32,
    fill_color: Color,
    outline_color: Color,
    outline_thicknes: u32,
    vertices: u32,
}

struct EnemyConfig {
    shape_radius: u32,
    collision_radius: u32,
    min_speed: f32,
    max_speed: f32,
    outline_color: Color,
    outline_thicknes: u32,
    min_vertices: u32,
    max_vertices: u32,
    small_lifespan: u32,
    spawn_interval: u32,
}

struct BulletConfig {
    shape_radius: u32,
    collision_radius: u32,
    speed: f32,
    fill_color: Color,
    outline_color: Color,
    outline_thicknes: u32,
    vertices: u32,
    lifespan: u32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Score {
    score: u32,
}
