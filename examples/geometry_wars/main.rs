use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
    time::Duration,
};

use glam::Vec2;
use hecs::{EntityBuilder, With, World};
use papercut::{
    camera::Camera,
    components::{Drawable, Tag, Transform},
    graphics::{Color, Geometry, PolygonShape, Tessellator},
    input::{InputHelper, KeyCode, MouseButton},
    Context, Game, RendererConfig, Scene, WindowConfig,
};
use rand::{thread_rng, Rng};

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
    score: u32,
    running_time: Duration,
    last_enemy_spawn_time: Duration,
    last_special_weapon_spawn_time: Duration,
    paused: bool,
    running: bool,
    font_config: FontConfig,
    player_config: PlayerConfig,
    enemy_config: EnemyConfig,
    bullet_config: BulletConfig,
    world: World,
}

impl Game for GeometryWars {
    fn on_create(&mut self) {
        let font_config = FontConfig {
            file: String::from("fonts/arial.ttf"),
            size: 24,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        };
        self.font_config = font_config;

        let player_config = PlayerConfig {
            shape_radius: 32,
            collision_radius: 32,
            speed: 250.0,
            fill_color: Color::new(0.02, 0.02, 0.02, 1.0),
            outline_color: Color::new(1.0, 0.0, 0.0, 1.0),
            outline_thicknes: 4,
            vertices: 8,
        };
        self.player_config = player_config;

        let enemy_config = EnemyConfig {
            shape_radius: 32,
            collision_radius: 32,
            min_speed: 150.0,
            max_speed: 250.0,
            outline_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_thicknes: 2,
            min_vertices: 3,
            max_vertices: 8,
            small_lifespan: Duration::from_secs_f32(1.5),
            spawn_interval: Duration::from_secs_f32(2.0),
        };
        self.enemy_config = enemy_config;

        let bullet_config = BulletConfig {
            shape_radius: 10,
            collision_radius: 10,
            speed: 800.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_thicknes: 2,
            vertices: 20,
            lifespan: Duration::from_secs_f32(1.5),
        };
        self.bullet_config = bullet_config;

        self.world = World::new();

        self.running = true;
    }

    fn on_update(
        &mut self,
        input: &InputHelper,
        ctx: &mut Context,
        camera: &Camera,
        dt: Duration,
    ) -> bool {
        // TODO: Add dirty flag to shape/drawable and move tesselator behind the scenes.
        let mut tessellator = Tessellator::new(0.02);
        self.system_user_input(input, camera);

        if !self.paused {
            self.system_player_spawner(ctx.window_size(), &mut tessellator);
            self.system_enemy_spawner(ctx.window_size(), &mut tessellator);
            self.system_bullet_spawner(&mut tessellator);
            self.system_special_weapon_spawner(&mut tessellator);
            self.system_movement(ctx.window_size(), dt);
            self.system_lifespan(&mut tessellator, dt);
            self.system_collision();
            self.system_small_enemy_spawner(&mut tessellator);

            self.running_time += dt; // TODO: Running time should be provided by the engine.
        }

        self.system_rotate_visible_entities(dt);
        self.system_remove_dead_entities();

        ctx.set_window_title(format!("Geometry Wars - Score: {}", self.score));

        self.running
    }

    fn on_render(&self, scene: &mut papercut::Scene, ctx: &mut Context) {
        self.system_render(ctx, scene);
    }
}

impl GeometryWars {
    fn system_user_input(&mut self, user_input: &InputHelper, camera: &Camera) {
        if user_input.quit() || user_input.key_pressed(KeyCode::Escape) {
            self.running = false;
        }

        if user_input.key_pressed(KeyCode::P) {
            self.paused = !self.paused;
        }

        if !self.paused {
            for (_id, input) in self.world.query_mut::<&mut Input>() {
                if user_input.key_pressed(KeyCode::W) {
                    input.up = true;
                }
                if user_input.key_pressed(KeyCode::A) {
                    input.left = true;
                }
                if user_input.key_pressed(KeyCode::S) {
                    input.down = true;
                }
                if user_input.key_pressed(KeyCode::D) {
                    input.right = true;
                }

                if user_input.key_released(KeyCode::W) {
                    input.up = false;
                }
                if user_input.key_released(KeyCode::A) {
                    input.left = false;
                }
                if user_input.key_released(KeyCode::S) {
                    input.down = false;
                }
                if user_input.key_released(KeyCode::D) {
                    input.right = false;
                }

                input.left_button = false;
                if user_input.mouse_pressed(MouseButton::Left) {
                    input.left_button = true;
                }
                input.right_button = false;
                if user_input.mouse_pressed(MouseButton::Right) {
                    input.right_button = true;
                }

                input.mouse_screen_position = user_input.mouse_in_viewport();
                input.mouse_world_position = user_input.mouse_in_world(camera);
            }
        }
    }

    fn system_player_spawner(&mut self, window_size: Vec2, tessellator: &mut Tessellator) {
        let alive = !self
            .world
            .query::<&Input>()
            .iter()
            .collect::<Vec<_>>()
            .is_empty();

        if !alive {
            let mut eb = EntityBuilder::new();
            build_player(&mut eb, window_size, tessellator, &self.player_config);
            self.world.spawn(eb.build());
        }
    }

    fn system_enemy_spawner(&mut self, window_size: Vec2, tessellator: &mut Tessellator) {
        if self.last_enemy_spawn_time + self.enemy_config.spawn_interval < self.running_time {
            let mut eb = EntityBuilder::new();
            build_enemy(&mut eb, window_size, tessellator, &self.enemy_config);
            self.world.spawn(eb.build());
            self.last_enemy_spawn_time = self.running_time;
        }
    }

    fn system_small_enemy_spawner(&mut self, tessellator: &mut Tessellator) {
        let mut to_spawn = Vec::new();

        for (_id, (tag, shape, transform, physics, health, score)) in self
            .world
            .query::<(&Tag, &Drawable, &Transform, &Physics, &Health, &Score)>()
            .iter()
        {
            if tag.name == ENEMY_TAG && health.health <= 0 {
                if let Drawable::Polygon(parent_shape) = &shape {
                    build_small_enemies(
                        &mut to_spawn,
                        transform.translation,
                        parent_shape,
                        physics,
                        score,
                        self.enemy_config.small_lifespan,
                        tessellator,
                    );
                }
            }
        }

        for mut eb in to_spawn {
            self.world.spawn(eb.build());
        }
    }

    fn system_bullet_spawner(&mut self, tessellator: &mut Tessellator) {
        let mut to_spawn = Vec::new();

        for (_id, (input, transform)) in self.world.query_mut::<(&Input, &Transform)>() {
            if input.left_button {
                let parent_position = transform.translation;
                let mouse_position = input.mouse_world_position;

                let mut eb = EntityBuilder::new();
                build_bullet(
                    &mut eb,
                    parent_position,
                    mouse_position,
                    tessellator,
                    &self.bullet_config,
                );
                to_spawn.push(eb);
            }
        }

        for mut eb in to_spawn {
            self.world.spawn(eb.build());
        }
    }

    fn system_special_weapon_spawner(&mut self, tessellator: &mut Tessellator) {
        let respawn_interval = Duration::from_secs_f32(10.0);

        let mut to_spawn = Vec::new();

        for (_id, (input, transform, drawable)) in
            self.world.query_mut::<(&Input, &Transform, &Drawable)>()
        {
            let parent_position = transform.translation;
            if input.right_button {
                if self.last_special_weapon_spawn_time + respawn_interval < self.running_time {
                    if let Drawable::Polygon(parent_shape) = drawable {
                        build_special_weapon(
                            &mut to_spawn,
                            parent_position,
                            parent_shape,
                            tessellator,
                        );
                    }
                    self.last_special_weapon_spawn_time = self.running_time;
                }
            }
        }

        for mut eb in to_spawn {
            self.world.spawn(eb.build());
        }
    }

    fn system_movement(&mut self, window_size: Vec2, dt: Duration) {
        for (_id, (transform, physics, drawable, input)) in
            self.world
                .query_mut::<(&mut Transform, &mut Physics, &Drawable, Option<&Input>)>()
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

                future_pos = future_pos + (move_dir * physics.velocity) * dt.as_secs_f32();

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
                future_pos = future_pos + physics.velocity * dt.as_secs_f32();

                if let Drawable::Polygon(shape) = drawable {
                    if future_pos.x - shape.radius < 0.0 {
                        physics.velocity.x = -physics.velocity.x;
                        future_pos.x = shape.radius;
                    }
                    if future_pos.x + shape.radius > window_size.x {
                        physics.velocity.x = -physics.velocity.x;
                        future_pos.x = window_size.x - shape.radius;
                    }
                    if future_pos.y - shape.radius < 0.0 {
                        physics.velocity.y = -physics.velocity.y;
                        future_pos.y = shape.radius;
                    }
                    if future_pos.y + shape.radius > window_size.y {
                        physics.velocity.y = -physics.velocity.y;
                        future_pos.y = window_size.y - shape.radius;
                    }
                }
            }

            transform.translation = future_pos;
        }
    }

    fn system_lifespan(&mut self, tessellator: &mut Tessellator, dt: Duration) {
        for (_id, (lifespan, drawable, tag)) in self
            .world
            .query_mut::<(&mut Lifespan, &mut Drawable, &Tag)>()
        {
            if tag.name == SMALL_ENEMY_TAG
                || tag.name == BULLET_TAG
                || tag.name == SPECIAL_WEAPON_TAG
            {
                lifespan.remaining = lifespan.remaining.saturating_sub(dt);
                if lifespan.remaining > Duration::ZERO {
                    let alpha_ratio =
                        lifespan.remaining.as_secs_f32() / lifespan.total.as_secs_f32();

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

    fn system_collision(&mut self) {
        // TODO: Enemies should only spawn small enemies if they have been shot, not on a collision.
        // TODO: Score should only increase if enemies have been shot, not on a collision.
        let mut colliding = HashSet::new();

        let mut players = HashMap::new();
        let mut enemies = HashMap::new();
        let mut small_enemies = HashMap::new();
        let mut bullets = HashMap::new();
        let mut special_weapons = HashMap::new();

        {
            let mut binding = self.world.query::<(&Tag, &Transform, &CircleCollider)>();
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

        let mut total_score = self.score;

        for id in colliding {
            if let Ok((lifespan, health, score)) =
                self.world
                    .query_one_mut::<(Option<&mut Lifespan>, Option<&mut Health>, Option<&Score>)>(
                        id,
                    )
            {
                if let Some(lifespan) = lifespan {
                    lifespan.remaining = Duration::ZERO;
                }
                if let Some(health) = health {
                    health.health = 0;
                }
                if let Some(score) = score {
                    total_score += score.score;
                }
            }
        }

        self.score = total_score;
    }

    fn system_rotate_visible_entities(&mut self, dt: Duration) {
        for (_id, (transform,)) in self.world.query_mut::<With<(&mut Transform,), &Drawable>>() {
            transform.rotation += 60.0 * dt.as_secs_f32();
        }
    }

    fn system_remove_dead_entities(&mut self) {
        let mut to_remove = HashSet::new();

        for (id, (lifespan, health)) in self
            .world
            .query::<(Option<&Lifespan>, Option<&Health>)>()
            .iter()
        {
            if let Some(lifespan) = lifespan {
                if lifespan.remaining <= Duration::ZERO {
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
            self.world.despawn(entity).expect("TODO: error handling");
        }
    }

    fn system_render(&self, ctx: &mut Context, scene: &mut Scene) {
        for (_id, (transform, drawable)) in self.world.query::<(&Transform, &Drawable)>().iter() {
            ctx.draw_shape(transform, drawable, scene);
        }
    }
}

fn check_collision(a: &(&Transform, &CircleCollider), b: &(&Transform, &CircleCollider)) -> bool {
    let distance = a.0.translation - b.0.translation;
    let actual_distance = distance.x * distance.x + distance.y * distance.y;
    let min_distance = a.1.radius * a.1.radius + b.1.radius * b.1.radius;

    actual_distance <= min_distance
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
    small_lifespan: Duration,
    spawn_interval: Duration,
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
    lifespan: Duration,
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
    total: Duration,
    remaining: Duration,
}

fn build_player(
    eb: &mut EntityBuilder,
    window_size: Vec2,
    tessellator: &mut Tessellator,
    player_config: &PlayerConfig,
) {
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

fn build_enemy(
    eb: &mut EntityBuilder,
    window_size: Vec2,
    tessellator: &mut Tessellator,
    enemy_config: &EnemyConfig,
) {
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
}

fn build_small_enemies(
    ebs: &mut Vec<EntityBuilder>,
    parent_position: Vec2,
    parent_shape: &PolygonShape,
    parent_physics: &Physics,
    parent_score: &Score,
    lifespan: Duration,
    tessellator: &mut Tessellator,
) {
    let position = parent_position;
    let speed = parent_physics.velocity;
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
        physics.velocity = Vec2::new(speed.x.abs() * angle.cos(), speed.y.abs() * angle.sin());

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

fn build_bullet(
    eb: &mut EntityBuilder,
    from: Vec2,
    to: Vec2,
    tessellator: &mut Tessellator,
    bullet_config: &BulletConfig,
) {
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
    let bullet_speed = Vec2::new(bullet_config.speed, bullet_config.speed) * bullet_direction;
    let mut physics = Physics::default();
    physics.velocity = bullet_speed;

    let lifespan = Lifespan {
        total: bullet_config.lifespan,
        remaining: bullet_config.lifespan,
    };

    eb.add_bundle((tag, transform, drawable, collider, physics, lifespan));
}

fn build_special_weapon(
    ebs: &mut Vec<EntityBuilder>,
    parent_position: Vec2,
    parent_shape: &PolygonShape,
    tessellator: &mut Tessellator,
) {
    let position = parent_position;
    let speed = 2500.0;
    let radius = parent_shape.radius / 2.0;
    let fill_color = parent_shape.fill_color;
    let outline_color = parent_shape.outline_color;
    let outline_thickness = parent_shape.outline_thickness;
    let entity_count = 18;
    let offset_angle = 360.0 / entity_count as f32;
    let lifespan = Duration::from_secs_f32(1.5);

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
}
