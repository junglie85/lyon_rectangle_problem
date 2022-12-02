use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use glam::Vec2;
use hecs::{EntityBuilder, With, World};
use papercut::{
    camera::Camera,
    components::{Drawable, Tag, Transform},
    graphics::{Color, Geometry, PolygonShape, Tessellator},
    input::InputHelper,
    Game, RendererConfig, WindowConfig,
};
use rand::{thread_rng, Rng};
use winit::event::VirtualKeyCode;

const PLAYER_TAG: &str = "player";
const ENEMY_TAG: &str = "enemy";
const SMALL_ENEMY_TAG: &str = "small_enemy";
const BULLET_TAG: &str = "bullet";
const SPECIAL_WEAPON_TAG: &str = "special_weapon";

fn main() {
    let wc = WindowConfig {
        title: "Geometry Wars".to_string(),
        size: Vec2::new(1280.0, 720.0),
        _frame_rate: 60,
        fullscreen: None,
    };

    let clear_color = Color::new(0.0, 0.0, 0.0, 1.0);
    let rc = RendererConfig { clear_color };

    papercut::init_logger();
    papercut::start::<GeometryWars>(wc, rc);
}

#[derive(Default)]
struct GeometryWars {
    // TODO: Move these into the world?
    score: u32,
    current_frame: u32,
    last_enemy_spawn_time: u32,
    last_special_weapon_spawn_time: u32,
    paused: bool,
    running: bool,
    font_config: FontConfig,
    player_config: PlayerConfig,
    enemy_config: EnemyConfig,
    bullet_config: BulletConfig,
}

impl Game for GeometryWars {
    fn setup(&mut self, world: &mut World, window_config: &WindowConfig) {
        let font_config = FontConfig {
            file: String::from("fonts/arial.ttf"),
            size: 24,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        };
        self.font_config = font_config;

        let player_config = PlayerConfig {
            shape_radius: 32,
            collision_radius: 32,
            speed: 5.0,
            fill_color: Color::new(0.02, 0.02, 0.02, 1.0),
            outline_color: Color::new(1.0, 0.0, 0.0, 1.0),
            outline_thicknes: 4,
            vertices: 8,
        };
        self.player_config = player_config;

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
        self.enemy_config = enemy_config;

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
        self.bullet_config = bullet_config;

        let mut tessellator = Tessellator::new(0.02);
        let mut eb = EntityBuilder::new();
        self.spawn_player(&mut eb, window_config.size, &mut tessellator);
        world.spawn(eb.build());

        self.running = true;
    }

    fn update(
        &mut self,
        world: &mut World,
        input: &InputHelper,
        window_config: &WindowConfig,
        camera: &Camera,
    ) -> bool {
        let mut tessellator = Tessellator::new(0.02);
        system_user_input(self, world, input, camera);

        if !self.paused {
            system_player_spawner(world, self, window_config.size, &mut tessellator);
            system_enemy_spawner(world, self, window_config.size, &mut tessellator);
            system_bullet_spawner(world, self, &mut tessellator);
            system_special_weapon_spawner(world, self, &mut tessellator);
            system_movement(world, window_config.size);
            system_lifespan(world, &mut tessellator);
            system_collision(world, self);
            system_small_enemy_spawner(world, self, &mut tessellator);

            self.current_frame += 1;
        }

        system_rotate_visible_entities(world);
        system_remove_dead_entities(world);

        println!("Score: {}", self.score);

        self.running
    }
}

// TODO: These shouldn't take a game if everything is in the world
impl GeometryWars {
    fn spawn_player(
        &self,
        eb: &mut EntityBuilder,
        window_size: Vec2,
        tessellator: &mut Tessellator,
    ) {
        let player_config = &self.player_config;

        let tag = Tag {
            name: PLAYER_TAG.to_string(),
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
        shape.update(tessellator);

        let drawable = Drawable::Polygon(shape);

        let mut collider = CircleCollider::default();
        collider.radius = player_config.collision_radius as f32;

        let mut physics = Physics::default();
        physics.velocity = Vec2::new(player_config.speed, player_config.speed);

        let input = Input::default();

        let health = Health { health: 1 };

        eb.add_bundle((tag, transform, drawable, collider, physics, input, health));
    }

    fn spawn_enemy(
        &mut self,
        eb: &mut EntityBuilder,
        window_size: Vec2,
        tessellator: &mut Tessellator,
    ) {
        let enemy_config = &self.enemy_config;

        let mut rng = thread_rng();

        let tag = Tag {
            name: ENEMY_TAG.to_string(),
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
        shape.update(tessellator);

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

        let health = Health { health: 1 };

        let score = Score {
            score: vertex_count * 100,
        };

        eb.add_bundle((tag, transform, drawable, collider, physics, health, score));

        self.last_enemy_spawn_time = self.current_frame;
    }

    fn spawn_small_enemies(
        &self,
        ebs: &mut Vec<EntityBuilder>,
        parent_position: Vec2,
        parent_shape: &PolygonShape,
        parent_physics: &Physics,
        parent_score: &Score,
        lifespan: u32,
        tessellator: &mut Tessellator,
    ) {
        let position = parent_position;
        let speed = parent_physics.velocity.x.abs();
        let radius = parent_shape.radius / 2.0;
        let fill_color = parent_shape.fill_color;
        let outline_color = parent_shape.outline_color;
        let outline_thickness = parent_shape.outline_thickness;
        let point_count = parent_shape.point_count;
        let offset_angle = 360.0 / point_count as f32;
        let score = parent_score.score * 2;

        for i in 0..point_count {
            let tag = Tag {
                name: SMALL_ENEMY_TAG.to_string(),
            };

            let mut transform = Transform::from_position(position.x, position.y);
            transform.origin = Vec2::new(radius, radius);

            let mut shape = PolygonShape::default();
            shape.radius = radius;
            shape.point_count = point_count;
            shape.fill_color = fill_color;
            shape.outline_color = outline_color;
            shape.outline_thickness = outline_thickness;
            shape.update(tessellator);

            let drawable = Drawable::Polygon(shape);

            let mut collider = CircleCollider::default();
            collider.radius = radius;

            let angle = offset_angle * i as f32 * (PI / 180.0);
            let mut physics = Physics::default();
            physics.velocity = Vec2::new(speed * angle.cos(), speed * angle.sin());

            let lifespan = Lifespan {
                total: lifespan,
                remaining: lifespan,
            };

            let score = Score { score };

            let mut eb = EntityBuilder::new();
            eb.add_bundle((tag, transform, drawable, collider, physics, lifespan, score));
            ebs.push(eb);
        }
    }

    fn spawn_bullet(
        &self,
        eb: &mut EntityBuilder,
        from: Vec2,
        to: Vec2,
        tessellator: &mut Tessellator,
    ) {
        let bullet_config = &self.bullet_config;

        let tag = Tag {
            name: BULLET_TAG.to_string(),
        };

        let mut transform = Transform::from_position(from.x, from.y);
        transform.origin = Vec2::new(
            bullet_config.shape_radius as f32,
            bullet_config.shape_radius as f32,
        );

        let mut shape = PolygonShape::default();
        shape.radius = bullet_config.shape_radius as f32;
        shape.point_count = bullet_config.vertices;
        shape.fill_color = bullet_config.fill_color;
        shape.outline_color = bullet_config.outline_color;
        shape.outline_thickness = bullet_config.outline_thicknes as f32;
        shape.update(tessellator);

        let drawable = Drawable::Polygon(shape);

        let mut collider = CircleCollider::default();
        collider.radius = bullet_config.collision_radius as f32;

        let bullet_direction = (to - from).normalize_or_zero();
        let bullet_speed = Vec2::new(bullet_config.speed, bullet_config.speed);
        let bullet_velocity = bullet_direction * bullet_speed;
        let mut physics = Physics::default();
        physics.velocity = bullet_velocity;

        let lifespan = Lifespan {
            total: bullet_config.lifespan,
            remaining: bullet_config.lifespan,
        };

        eb.add_bundle((tag, transform, drawable, collider, physics, lifespan));
    }

    fn spawn_special_weapon(
        &mut self,
        ebs: &mut Vec<EntityBuilder>,
        parent_position: Vec2,
        parent_shape: &PolygonShape,
        tessellator: &mut Tessellator,
    ) {
        let respawn_interval = 600;
        if self.last_special_weapon_spawn_time + respawn_interval < self.current_frame {
            let position = parent_position;
            let speed = 20.0;
            let radius = parent_shape.radius / 2.0;
            let fill_color = parent_shape.fill_color;
            let outline_color = parent_shape.outline_color;
            let outline_thickness = parent_shape.outline_thickness;
            let entity_count = 18;
            let offset_angle = 360.0 / entity_count as f32;
            let lifespan = 180;

            for i in 0..entity_count {
                let tag = Tag {
                    name: SPECIAL_WEAPON_TAG.to_string(),
                };

                let mut transform = Transform::from_position(position.x, position.y);
                transform.origin = Vec2::new(radius, radius);

                let mut shape = PolygonShape::default();
                shape.radius = radius;
                shape.point_count = entity_count;
                shape.fill_color = fill_color;
                shape.outline_color = outline_color;
                shape.outline_thickness = outline_thickness;
                shape.update(tessellator);

                let drawable = Drawable::Polygon(shape);

                let mut collider = CircleCollider::default();
                collider.radius = radius;

                let angle = offset_angle * i as f32 * (PI / 180.0);
                let mut physics = Physics::default();
                physics.velocity = Vec2::new(speed * angle.cos(), speed * angle.sin());

                let lifespan = Lifespan {
                    total: lifespan,
                    remaining: lifespan,
                };

                let mut eb = EntityBuilder::new();
                eb.add_bundle((tag, transform, drawable, collider, physics, lifespan));
                ebs.push(eb);
            }

            self.last_special_weapon_spawn_time = self.current_frame;
        }
    }
}

#[derive(Debug, Default)]
struct FontConfig {
    file: String,
    size: u32,
    color: Color,
}

#[derive(Debug, Default)]
struct PlayerConfig {
    shape_radius: u32,
    collision_radius: u32,
    speed: f32,
    fill_color: Color,
    outline_color: Color,
    outline_thicknes: u32,
    vertices: u32,
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
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

#[derive(Debug, Default, Clone)]
pub struct CircleCollider {
    radius: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Physics {
    velocity: Vec2,
}

#[derive(Debug, Default, Clone)]
pub struct Input {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    left_button: bool,
    right_button: bool,
    mouse_screen_position: Vec2,
    mouse_world_position: Vec2,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Score {
    score: u32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Health {
    health: u32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Lifespan {
    total: u32,
    remaining: u32,
}

fn system_user_input(
    game: &mut GeometryWars,
    world: &mut World,
    user_input: &InputHelper,
    camera: &Camera,
) {
    if user_input.quit() || user_input.key_pressed(VirtualKeyCode::Escape) {
        game.running = false;
    }

    if user_input.key_pressed(VirtualKeyCode::P) {
        game.paused = !game.paused;
    }

    if !game.paused {
        for (_id, input) in world.query_mut::<&mut Input>() {
            if user_input.key_pressed(VirtualKeyCode::W) {
                input.up = true;
            }
            if user_input.key_pressed(VirtualKeyCode::A) {
                input.left = true;
            }
            if user_input.key_pressed(VirtualKeyCode::S) {
                input.down = true;
            }
            if user_input.key_pressed(VirtualKeyCode::D) {
                input.right = true;
            }

            if user_input.key_released(VirtualKeyCode::W) {
                input.up = false;
            }
            if user_input.key_released(VirtualKeyCode::A) {
                input.left = false;
            }
            if user_input.key_released(VirtualKeyCode::S) {
                input.down = false;
            }
            if user_input.key_released(VirtualKeyCode::D) {
                input.right = false;
            }

            input.left_button = false;
            if user_input.mouse_pressed(0) {
                input.left_button = true;
            }
            input.right_button = false;
            if user_input.mouse_pressed(1) {
                input.right_button = true;
            }

            input.mouse_screen_position = user_input.mouse_in_viewport();
            input.mouse_world_position = user_input.mouse_in_world(camera);
        }
    }
}

fn system_player_spawner(
    world: &mut World,
    game: &mut GeometryWars,
    window_size: Vec2,
    tessellator: &mut Tessellator,
) {
    let alive = !world
        .query::<&Input>()
        .iter()
        .collect::<Vec<_>>()
        .is_empty();

    if !alive {
        let mut eb = EntityBuilder::new();
        game.spawn_player(&mut eb, window_size, tessellator);
        world.spawn(eb.build());
    }
}

fn system_enemy_spawner(
    world: &mut World,
    game: &mut GeometryWars,
    window_size: Vec2,
    tessellator: &mut Tessellator,
) {
    if game.last_enemy_spawn_time + game.enemy_config.spawn_interval < game.current_frame {
        let mut eb = EntityBuilder::new();
        game.spawn_enemy(&mut eb, window_size, tessellator);
        world.spawn(eb.build());
    }
}

fn system_small_enemy_spawner(
    world: &mut World,
    game: &GeometryWars,
    tessellator: &mut Tessellator,
) {
    let mut to_spawn = Vec::new();

    for (_id, (tag, shape, transform, physics, health, score)) in world
        .query::<(&Tag, &Drawable, &Transform, &Physics, &Health, &Score)>()
        .iter()
    {
        if tag.name == ENEMY_TAG && health.health <= 0 {
            if let Drawable::Polygon(parent_shape) = &shape {
                game.spawn_small_enemies(
                    &mut to_spawn,
                    transform.translation,
                    parent_shape,
                    physics,
                    score,
                    game.enemy_config.small_lifespan,
                    tessellator,
                );
            }
        }
    }

    for mut eb in to_spawn {
        world.spawn(eb.build());
    }
}

fn system_bullet_spawner(world: &mut World, game: &GeometryWars, tessellator: &mut Tessellator) {
    let mut to_spawn = Vec::new();

    for (_id, (input, transform)) in world.query_mut::<(&Input, &Transform)>() {
        if input.left_button {
            let parent_position = transform.translation;
            let mouse_position = input.mouse_world_position;

            let mut eb = EntityBuilder::new();
            game.spawn_bullet(&mut eb, parent_position, mouse_position, tessellator);
            to_spawn.push(eb);
        }
    }

    for mut eb in to_spawn {
        world.spawn(eb.build());
    }
}

fn system_special_weapon_spawner(
    world: &mut World,
    game: &mut GeometryWars,
    tessellator: &mut Tessellator,
) {
    let mut to_spawn = Vec::new();

    for (_id, (input, transform, drawable)) in world.query_mut::<(&Input, &Transform, &Drawable)>()
    {
        let parent_position = transform.translation;
        if input.right_button {
            if let Drawable::Polygon(parent_shape) = drawable {
                game.spawn_special_weapon(
                    &mut to_spawn,
                    parent_position,
                    parent_shape,
                    tessellator,
                );
            }
        }
    }

    for mut eb in to_spawn {
        world.spawn(eb.build());
    }
}

fn system_movement(world: &mut World, window_size: Vec2) {
    for (_id, (transform, physics, drawable, input)) in
        world.query_mut::<(&mut Transform, &mut Physics, &Drawable, Option<&Input>)>()
    {
        let mut future_pos = transform.translation;

        if let Some(input) = input {
            let mut move_dir = Vec2::new(0.0, 0.0);
            if input.up {
                move_dir.y += 1.0;
            }
            if input.down {
                move_dir.y += -1.0;
            }
            if input.left {
                move_dir.x += -1.0;
            }
            if input.right {
                move_dir.x += 1.0;
            }

            let move_dir = move_dir.normalize_or_zero();

            future_pos = future_pos + (move_dir * physics.velocity); // TODO: * dt;

            if let Drawable::Polygon(shape) = drawable {
                if future_pos.x - shape.radius < 0.0 {
                    future_pos.x = shape.radius;
                }
                if future_pos.x + shape.radius > window_size.x {
                    future_pos.x = window_size.x - shape.radius;
                }
                if future_pos.y - shape.radius < 0.0 {
                    future_pos.y = shape.radius;
                }
                if future_pos.y + shape.radius > window_size.y {
                    future_pos.y = window_size.y - shape.radius;
                }
            }
        } else {
            future_pos = future_pos + physics.velocity; // TODO: * dt;

            if let Drawable::Polygon(shape) = drawable {
                if future_pos.x - shape.radius < 0.0 {
                    future_pos.x = shape.radius;
                    physics.velocity.x = -physics.velocity.x;
                }
                if future_pos.x + shape.radius > window_size.x {
                    future_pos.x = window_size.x - shape.radius;
                    physics.velocity.x = -physics.velocity.x;
                }
                if future_pos.y - shape.radius < 0.0 {
                    future_pos.y = shape.radius;
                    physics.velocity.y = -physics.velocity.y;
                }
                if future_pos.y + shape.radius > window_size.y {
                    future_pos.y = window_size.y - shape.radius;
                    physics.velocity.y = -physics.velocity.y;
                }
            }
        }

        transform.translation = future_pos;
    }
}

fn system_lifespan(world: &mut World, tessellator: &mut Tessellator) {
    for (_id, (lifespan, drawable, tag)) in
        world.query_mut::<(&mut Lifespan, &mut Drawable, &Tag)>()
    {
        if tag.name == SMALL_ENEMY_TAG || tag.name == BULLET_TAG || tag.name == SPECIAL_WEAPON_TAG {
            lifespan.remaining -= 1;
            if lifespan.remaining > 0 {
                let alpha_ratio = lifespan.remaining as f32 / lifespan.total as f32;

                if let Drawable::Polygon(shape) = drawable {
                    let new_alpha = 1.0 * alpha_ratio;
                    shape.fill_color.a = new_alpha;
                    shape.outline_color.a = new_alpha;
                    shape.update(tessellator);
                }
            }
        }
    }
}

fn system_collision(world: &mut World, game: &mut GeometryWars) {
    let mut colliding = HashSet::new();

    let mut players = HashMap::new();
    let mut enemies = HashMap::new();
    let mut small_enemies = HashMap::new();
    let mut bullets = HashMap::new();
    let mut special_weapons = HashMap::new();

    {
        let mut binding = world.query::<(&Tag, &Transform, &CircleCollider)>();
        for (id, (tag, transform, collider)) in binding.iter() {
            match tag.name.as_str() {
                PLAYER_TAG => players.insert(id, (transform, collider)),
                ENEMY_TAG => enemies.insert(id, (transform, collider)),
                SMALL_ENEMY_TAG => small_enemies.insert(id, (transform, collider)),
                BULLET_TAG => bullets.insert(id, (transform, collider)),
                SPECIAL_WEAPON_TAG => special_weapons.insert(id, (transform, collider)),
                _ => unreachable!("unknown entity"),
            };
        }

        for (player_id, player) in &players {
            for (enemy_id, enemy) in &enemies {
                if check_collision(player, enemy) {
                    colliding.insert(*player_id);
                    colliding.insert(*enemy_id);
                }
            }

            for (enemy_id, enemy) in &small_enemies {
                if check_collision(player, enemy) {
                    colliding.insert(*player_id);
                    colliding.insert(*enemy_id);
                }
            }
        }

        for (weapon_id, weapon) in bullets.into_iter().chain(special_weapons) {
            for (enemy_id, enemy) in &enemies {
                if check_collision(&weapon, enemy) {
                    colliding.insert(weapon_id);
                    colliding.insert(*enemy_id);
                }
            }

            for (enemy_id, enemy) in &small_enemies {
                if check_collision(&weapon, enemy) {
                    colliding.insert(weapon_id);
                    colliding.insert(*enemy_id);
                }
            }
        }
    }

    let mut total_score = game.score;

    for id in colliding {
        if let Ok((lifespan, health, score)) =
            world.query_one_mut::<(Option<&mut Lifespan>, Option<&mut Health>, Option<&Score>)>(id)
        {
            if let Some(lifespan) = lifespan {
                lifespan.remaining = 0;
            }
            if let Some(health) = health {
                health.health = 0;
            }
            if let Some(score) = score {
                total_score += score.score;
            }
        }
    }

    game.score = total_score;
}

fn system_rotate_visible_entities(world: &mut World) {
    for (_id, (transform,)) in world.query_mut::<With<(&mut Transform,), &Drawable>>() {
        transform.rotation += 1.0;
    }
}

fn system_remove_dead_entities(world: &mut World) {
    let mut to_remove = HashSet::new();

    for (id, (lifespan, health)) in world.query::<(Option<&Lifespan>, Option<&Health>)>().iter() {
        if let Some(lifespan) = lifespan {
            if lifespan.remaining <= 0 {
                to_remove.insert(id);
            }
        }
        if let Some(health) = health {
            if health.health <= 0 {
                to_remove.insert(id);
            }
        }
    }

    for entity in to_remove.into_iter() {
        world.despawn(entity).expect("TODO: error handling");
    }
}

fn check_collision(a: &(&Transform, &CircleCollider), b: &(&Transform, &CircleCollider)) -> bool {
    let distance = a.0.translation - b.0.translation;
    let actual_distance = distance.x * distance.x + distance.y * distance.y;
    let min_distance = a.1.radius * a.1.radius + b.1.radius * b.1.radius;

    actual_distance <= min_distance
}
