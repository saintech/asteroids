use std::collections::HashSet;
use std::f32::consts::PI;

use macroquad::{
    color, input, math,
    math::{vec2, Vec2},
    miniquad::date,
    rand, shapes, text, time, window,
};
use macroquad_particles as particles;

const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const SHIP_MAX_SPEED: f32 = 400.0;
const SHIP_ACCEL: f32 = 500.0;
const SHIP_DECEL: f32 = 0.2;
const SHIP_DRAW_RADIUS: f32 = 14.0;
const SHIP_HIT_RADIUS: f32 = 9.0;
const SHIP_TURN_SPEED: f32 = 6.0;
const SHIP_BULLET_COLOR: color::Color = color::Color::new(1.0, 0.0, 0.0, 1.0);
const SHIP_BULLET_TIMER_LIMIT: f32 = 0.8;
const SHIP_BULLET_SPEED: f32 = 600.0;
const BULLET_COOLDOWN: f32 = 0.3;
const BULLET_RADIUS: f32 = 3.0;
const ALIEN_DRAW_RADIUS_BY_KIND: &[f32] = &[14.0, 10.0];
const ALIEN_HIT_RADIUS_BY_KIND: &[f32] = &[10.0, 7.0];
const ALIEN_BULLET_TIMER_LIMIT_BY_KIND: &[f32] = &[0.9, 1.3];
const ALIEN_BULLET_SPEED: f32 = 250.0;
const ALIEN_BULLET_COLOR: color::Color = color::Color::new(1.0, 0.0, 1.0, 1.0);
const ALIEN_SPAWN_PERIOD: f32 = 30.0;
const ALIEN_SHOOT_PERIOD: f32 = 1.3;
const ALIEN_SHIFT_PERIOD: f32 = 1.0;
const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage { max_speed: 180.0, radius: 12.0 },
    AsteroidStage { max_speed: 120.0, radius: 28.0 },
    AsteroidStage { max_speed: 60.0, radius: 40.0 },
];

struct AsteroidStage {
    max_speed: f32,
    radius: f32,
}

#[derive(PartialEq)]
enum SpriteVariant {
    Bullet,
    Ship { has_exhaust: bool },
    Asteroid { draw_points: Vec<(f32, f32)> },
    Alien,
}

struct Sprite {
    variant: SpriteVariant,
    size: f32,
    angle: f32,
    color: color::Color,
}

#[derive(Default)]
struct Body {
    radius: f32,
    angle: f32,
    speed: f32,
    is_hit: bool,
}

struct Bullet {
    position: math::Vec2,
    sprite: Sprite,
    body: Body,
    time_left: f32,
    from_enemy: bool,
}

struct Ship {
    position: math::Vec2,
    sprite: Sprite,
    body: Body,
    is_destroyed: bool,
    shoot_cooldown: f32,
}

struct Explosion {
    position: math::Vec2,
    body: Body,
    emitter: particles::Emitter,
    time_left: f32,
}

#[derive(Copy, Clone)]
enum AlienDirection {
    ToRight,
    ToLeft,
}

#[derive(Copy, Clone)]
enum AlienKind {
    Big,
    Small,
}

struct Alien {
    position: math::Vec2,
    sprite: Sprite,
    body: Body,
    is_destroyed: bool,
    kind: AlienKind,
    direction: AlienDirection,
    shoot_cooldown: f32,
    shift_timer: f32,
}

impl Alien {
    pub fn new() -> Self {
        let y = ARENA_HEIGHT * rand::gen_range(0.15, 0.85);
        let direction = [AlienDirection::ToRight, AlienDirection::ToLeft][rand::gen_range(0, 2)];
        let x = ARENA_WIDTH * direction as u32 as f32;
        let angle = PI * direction as u32 as f32;
        let kind = if rand::gen_range(0_u32, 10) < 3 {
            AlienKind::Small
        } else {
            AlienKind::Big
        };
        Alien {
            position: vec2(x, y),
            sprite: Sprite {
                variant: SpriteVariant::Alien,
                size: ALIEN_DRAW_RADIUS_BY_KIND[kind as usize],
                angle: 0.0,
                color: color::Color::new(0.0, 1.0, 0.0, 1.0),
            },
            body: Body {
                radius: ALIEN_HIT_RADIUS_BY_KIND[kind as usize],
                angle,
                speed: rand::gen_range(80.0, 100.0),
                is_hit: false,
            },
            is_destroyed: false,
            kind,
            direction,
            shoot_cooldown: ALIEN_SHOOT_PERIOD,
            shift_timer: 0.0,
        }
    }
}

struct Asteroid {
    position: math::Vec2,
    sprite: Sprite,
    body: Body,
    is_destroyed: bool,
    stage: usize,
}

impl Asteroid {
    pub fn new(x: f32, y: f32, stage: usize) -> Self {
        let radius = ASTEROID_STAGES[stage].radius + 5.0;
        let max_speed = ASTEROID_STAGES[stage].max_speed;
        let mut draw_points = vec![];
        let mut draw_angle: f32 = 0.0;
        while draw_angle < PI * 2.0 {
            let distance = rand::gen_range(0.75, 1.0) * radius;
            draw_points.push((draw_angle.cos() * distance, draw_angle.sin() * distance));
            draw_angle += rand::gen_range(0.1, 0.7);
        }
        let angle = rand::gen_range(0.0, 2.0 * PI);
        let speed = max_speed * rand::gen_range(0.5, 1.0);
        Asteroid {
            position: vec2(x, y),
            sprite: Sprite {
                variant: SpriteVariant::Asteroid { draw_points },
                size: f32::NAN,
                angle: 0.0,
                color: color::Color::new(1.0, 1.0, 0.0, 1.0),
            },
            body: Body { radius, angle, speed, is_hit: false },
            is_destroyed: false,
            stage,
        }
    }
}

#[derive(Debug)]
enum GameState {
    LevelLoading,
    LevelRunning,
    LevelCompleted,
    Pause,
    GameOver,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::LevelLoading
    }
}

#[derive(Eq, PartialEq, Hash)]
enum Action {
    Accelerate,
    TurnLeft,
    TurnRight,
    Shoot,
}

impl Default for Action {
    fn default() -> Self {
        Action::Accelerate
    }
}

#[derive(Default)]
struct Game {
    state: GameState,
    player_actions: HashSet<Action>,
    pause_timer: f32,
    alien_timer: f32,
    ship: Option<Ship>,
    bullets: Vec<Bullet>,
    asteroids: Vec<Asteroid>,
    explosions: Vec<Explosion>,
    aliens: Vec<Alien>,
}

fn load(game: &mut Game) {
    game.alien_timer = ALIEN_SPAWN_PERIOD;
}

fn input_system_update(game: &mut Game, _dt: f32) {
    match game.state {
        GameState::LevelRunning => {
            use input::KeyCode;
            let keymap = [
                (KeyCode::Up, Action::Accelerate),
                (KeyCode::Left, Action::TurnLeft),
                (KeyCode::Right, Action::TurnRight),
                (KeyCode::S, Action::Shoot),
            ];
            for (key, action) in keymap {
                if input::is_key_down(key) {
                    game.player_actions.insert(action);
                } else {
                    game.player_actions.remove(&action);
                }
            }
        }
        _ => (),
    }
}

fn ai_system_update(game: &mut Game, _dt: f32) {
    for alien in &mut game.aliens {
        let time_to_shift = alien.shift_timer == 0.0;
        let time_to_shoot = alien.shoot_cooldown == 0.0;
        if time_to_shift {
            alien.shift_timer = ALIEN_SHIFT_PERIOD;
            let d_angle = PI / 4.0;
            let origin_angle = PI * alien.direction as u32 as f32;
            alien.body.angle += d_angle * rand::gen_range(-2_i32, 2) as f32;
            alien.body.angle = alien
                .body
                .angle
                .clamp(origin_angle - d_angle, origin_angle + d_angle);
        }
        if time_to_shoot && game.ship.is_some() {
            alien.shoot_cooldown = ALIEN_SHOOT_PERIOD;
            let ship = game.ship.as_ref().unwrap();
            let bullet_time_left = ALIEN_BULLET_TIMER_LIMIT_BY_KIND[alien.kind as usize];
            let radius = ALIEN_HIT_RADIUS_BY_KIND[alien.kind as usize] / 2.0;
            let shoot_angle =
                f32::atan2(ship.position.y - alien.position.y, ship.position.x - alien.position.x);
            game.bullets.push(Bullet {
                position: vec2(
                    alien.position.x + shoot_angle.cos() * radius,
                    alien.position.y + shoot_angle.sin() * radius,
                ),
                sprite: Sprite {
                    variant: SpriteVariant::Bullet,
                    size: f32::NAN,
                    angle: shoot_angle,
                    color: ALIEN_BULLET_COLOR,
                },
                body: Body {
                    radius: BULLET_RADIUS,
                    angle: shoot_angle,
                    speed: ALIEN_BULLET_SPEED,
                    is_hit: false,
                },
                time_left: bullet_time_left,
                from_enemy: true,
            })
        }
        if alien.position.x < -10.0 || ARENA_WIDTH + 10.0 < alien.position.x {
            alien.is_destroyed = true;
        };
    }
}

fn timers_system_update(game: &mut Game, dt: f32) {
    game.alien_timer = f32::max(0.0, game.alien_timer - dt);
    game.pause_timer = f32::max(0.0, game.pause_timer - dt);
    if let Some(ship) = &mut game.ship {
        ship.shoot_cooldown = f32::max(0.0, ship.shoot_cooldown - dt);
    }
    for explosion in &mut game.explosions {
        explosion.time_left = f32::max(0.0, explosion.time_left - dt);
    }
    for bullet in &mut game.bullets {
        bullet.time_left = f32::max(0.0, bullet.time_left - dt);
    }
    for alien in &mut game.aliens {
        alien.shift_timer = f32::max(0.0, alien.shift_timer - dt);
        alien.shoot_cooldown = f32::max(0.0, alien.shoot_cooldown - dt);
    }
}

fn move_system_update(game: &mut Game, dt: f32) {
    match game.state {
        GameState::Pause => (),
        GameState::LevelRunning => {
            if let Some(ship) = &mut game.ship {
                if game.player_actions.contains(&Action::TurnRight) {
                    ship.sprite.angle += SHIP_TURN_SPEED * dt;
                    ship.sprite.angle = ship.sprite.angle.rem_euclid(2.0 * PI);
                }
                if game.player_actions.contains(&Action::TurnLeft) {
                    ship.sprite.angle -= SHIP_TURN_SPEED * dt;
                    ship.sprite.angle = ship.sprite.angle.rem_euclid(2.0 * PI);
                }
                if game.player_actions.contains(&Action::Accelerate)
                    && ship.body.speed <= SHIP_MAX_SPEED
                {
                    ship.sprite.variant = SpriteVariant::Ship { has_exhaust: true };
                    let (result_speed, result_angle) = sum_vectors(
                        ship.body.speed,
                        ship.body.angle,
                        SHIP_ACCEL * dt,
                        ship.sprite.angle,
                    );
                    ship.body.speed = result_speed;
                    ship.body.angle = result_angle;
                } else {
                    ship.sprite.variant = SpriteVariant::Ship { has_exhaust: false };
                }
            }
        }
        _ => (),
    }
    match game.state {
        GameState::Pause => (),
        _ => {
            if let Some(ship) = &mut game.ship {
                ship.body.speed -= ship.body.speed * SHIP_DECEL * dt;
                ship.position.x += ship.body.angle.cos() * ship.body.speed * dt;
                ship.position.x = ship.position.x.rem_euclid(ARENA_WIDTH);
                ship.position.y += ship.body.angle.sin() * ship.body.speed * dt;
                ship.position.y = ship.position.y.rem_euclid(ARENA_HEIGHT);
            }
            for alien in &mut game.aliens {
                alien.position.x += alien.body.angle.cos() * alien.body.speed * dt;
                alien.position.y += alien.body.angle.sin() * alien.body.speed * dt;
                alien.position.y = alien.position.y.rem_euclid(ARENA_HEIGHT);
            }
            for bullet in &mut game.bullets {
                bullet.position.x += bullet.body.angle.cos() * bullet.body.speed * dt;
                bullet.position.x = bullet.position.x.rem_euclid(ARENA_WIDTH);
                bullet.position.y += bullet.body.angle.sin() * bullet.body.speed * dt;
                bullet.position.y = bullet.position.y.rem_euclid(ARENA_HEIGHT);
            }
            for asteroid in &mut game.asteroids {
                asteroid.position.x += asteroid.body.angle.cos() * asteroid.body.speed * dt;
                asteroid.position.x = asteroid.position.x.rem_euclid(ARENA_WIDTH);
                asteroid.position.y += asteroid.body.angle.sin() * asteroid.body.speed * dt;
                asteroid.position.y = asteroid.position.y.rem_euclid(ARENA_HEIGHT);
            }
            for explosion in &mut game.explosions {
                explosion.position.x += explosion.body.angle.cos() * explosion.body.speed * dt;
                explosion.position.x = explosion.position.x.rem_euclid(ARENA_WIDTH);
                explosion.position.y += explosion.body.angle.sin() * explosion.body.speed * dt;
                explosion.position.y = explosion.position.y.rem_euclid(ARENA_HEIGHT);
            }
        }
    }
}

fn collision_system_update(game: &mut Game, _dt: f32) {
    for alien in &mut game.aliens {
        if let Some(ship) = game.ship.as_mut() {
            if are_circles_intersecting(
                alien.position.x,
                alien.position.y,
                alien.body.radius,
                ship.position.x,
                ship.position.y,
                ship.body.radius,
            ) {
                alien.body.is_hit = true;
                ship.body.is_hit = true;
            }
        }
    }
    for bullet in &mut game.bullets {
        if let Some(ship) = game.ship.as_mut() {
            if are_circles_intersecting(
                bullet.position.x,
                bullet.position.y,
                bullet.body.radius,
                ship.position.x,
                ship.position.y,
                ship.body.radius,
            ) && bullet.from_enemy
            {
                bullet.body.is_hit = true;
                ship.body.is_hit = true;
            }
        }
        for alien in &mut game.aliens {
            if are_circles_intersecting(
                bullet.position.x,
                bullet.position.y,
                bullet.body.radius,
                alien.position.x,
                alien.position.y,
                alien.body.radius,
            ) && !bullet.from_enemy
            {
                bullet.body.is_hit = true;
                alien.body.is_hit = true;
            }
        }
        for asteroid in &mut game.asteroids {
            if are_circles_intersecting(
                bullet.position.x,
                bullet.position.y,
                bullet.body.radius,
                asteroid.position.x,
                asteroid.position.y,
                asteroid.body.radius,
            ) {
                bullet.body.is_hit = true;
                asteroid.body.is_hit = true;
            }
        }
    }
    for asteroid in &mut game.asteroids {
        if let Some(ship) = game.ship.as_mut() {
            if are_circles_intersecting(
                asteroid.position.x,
                asteroid.position.y,
                asteroid.body.radius,
                ship.position.x,
                ship.position.y,
                ship.body.radius,
            ) {
                asteroid.body.is_hit = true;
                ship.body.is_hit = true;
            }
        }
        for alien in &mut game.aliens {
            if are_circles_intersecting(
                asteroid.position.x,
                asteroid.position.y,
                asteroid.body.radius,
                alien.position.x,
                alien.position.y,
                alien.body.radius,
            ) {
                asteroid.body.is_hit = true;
                alien.body.is_hit = true;
            }
        }
    }
}

fn damage_system_update(game: &mut Game, _dt: f32) {
    if let Some(ship) = game.ship.as_mut().filter(|sh| sh.body.is_hit) {
        ship.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(ship_explosion(ship.sprite.color));
        game.explosions.push(Explosion {
            position: ship.position,
            body: Default::default(),
            time_left: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    for alien in &mut game.aliens.iter_mut().filter(|a| a.body.is_hit) {
        alien.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(ship_explosion(alien.sprite.color));
        game.explosions.push(Explosion {
            position: alien.position,
            body: Body {
                angle: alien.body.angle,
                speed: alien.body.speed,
                ..Default::default()
            },
            time_left: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    game.bullets.retain(|b| !b.body.is_hit);
    let mut new_asteroids: Vec<Asteroid> = Default::default();
    for asteroid in &mut game.asteroids.iter_mut().filter(|a| a.body.is_hit) {
        asteroid.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(asteroid_explosion());
        game.explosions.push(Explosion {
            position: asteroid.position,
            body: Body {
                angle: asteroid.body.angle,
                speed: asteroid.body.speed * 1.5,
                ..Default::default()
            },
            time_left: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
        if asteroid.stage > 0 {
            new_asteroids.push(Asteroid::new(
                asteroid.position.x,
                asteroid.position.y,
                asteroid.stage - 1,
            ));
            new_asteroids.push(Asteroid::new(
                asteroid.position.x,
                asteroid.position.y,
                asteroid.stage - 1,
            ));
        }
    }
    game.asteroids.append(&mut new_asteroids);
}

fn gamestate_system_update(game: &mut Game, _dt: f32) {
    // dbg!(&game.state);
    match game.state {
        GameState::Pause => (),
        GameState::LevelLoading => {
            if game.ship.is_some() && !game.asteroids.is_empty() {
                game.state = GameState::LevelRunning;
            }
        }
        GameState::LevelRunning => {
            if game.ship.as_ref().map_or(false, |sh| sh.is_destroyed) {
                game.pause_timer = 2.0;
                game.state = GameState::GameOver;
            }
            if game.asteroids.is_empty() && game.aliens.is_empty() {
                game.pause_timer = 2.0;
                game.state = GameState::LevelCompleted;
            }
        }
        GameState::LevelCompleted => {
            if game.pause_timer == 0.0 {
                let alien_timer = game.alien_timer;
                *game = Default::default();
                game.alien_timer = alien_timer;
            }
        }
        GameState::GameOver => {
            if game.pause_timer == 0.0 {
                let alien_timer = game.alien_timer;
                *game = Default::default();
                game.alien_timer = alien_timer;
            }
        }
    }
}

fn cleanup_system_update(game: &mut Game, _dt: f32) {
    game.ship = game.ship.take().filter(|sh| !sh.is_destroyed);
    game.aliens.retain(|a| !a.is_destroyed);
    game.bullets.retain(|b| b.time_left > 0.0);
    game.asteroids.retain(|a| !a.is_destroyed);
    game.explosions.retain(|e| e.time_left > 0.0);
}

fn spawn_system_update(game: &mut Game, _dt: f32) {
    match game.state {
        GameState::LevelLoading => {
            if game.ship.is_none() {
                game.ship = Some(Ship {
                    position: vec2(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0),
                    sprite: Sprite {
                        variant: SpriteVariant::Ship { has_exhaust: false },
                        size: SHIP_DRAW_RADIUS,
                        angle: 0.0,
                        color: color::Color::new(0.0, 1.0, 1.0, 1.0),
                    },
                    body: Body {
                        radius: SHIP_HIT_RADIUS,
                        angle: 0.0,
                        speed: 0.0,
                        is_hit: false,
                    },
                    is_destroyed: false,
                    shoot_cooldown: 0.0,
                });
            }
            if game.asteroids.is_empty() {
                let ship = game.ship.as_ref().unwrap();
                let start_stage = ASTEROID_STAGES.len() - 1;
                while game.asteroids.len() < 5 {
                    let x = rand::gen_range(0.0, ARENA_WIDTH);
                    let y = rand::gen_range(0.0, ARENA_HEIGHT);
                    const RADIUS: i32 = (ARENA_HEIGHT * 0.3) as i32;
                    let is_too_close =
                        (x - ship.position.x) as i32 ^ 2 + (y - ship.position.y) as i32 ^ 2
                            <= RADIUS ^ 2;
                    if !is_too_close {
                        game.asteroids.push(Asteroid::new(x, y, start_stage));
                    }
                }
            }
        }
        GameState::LevelRunning => {
            if let Some(ship) = &mut game.ship {
                let shoot_is_ready = ship.shoot_cooldown == 0.0;
                if game.player_actions.contains(&Action::Shoot) && shoot_is_ready {
                    ship.shoot_cooldown = BULLET_COOLDOWN;
                    game.bullets.push(Bullet {
                        position: vec2(
                            ship.position.x + ship.sprite.angle.cos() * SHIP_DRAW_RADIUS,
                            ship.position.y + ship.sprite.angle.sin() * SHIP_DRAW_RADIUS,
                        ),
                        sprite: Sprite {
                            variant: SpriteVariant::Bullet,
                            size: f32::NAN,
                            angle: ship.sprite.angle,
                            color: SHIP_BULLET_COLOR,
                        },
                        body: Body {
                            radius: BULLET_RADIUS,
                            angle: ship.sprite.angle,
                            speed: SHIP_BULLET_SPEED,
                            is_hit: false,
                        },
                        time_left: SHIP_BULLET_TIMER_LIMIT,
                        from_enemy: false,
                    });
                }
            }
            let time_to_spawn_alien = game.alien_timer == 0.0;
            if time_to_spawn_alien {
                game.alien_timer = ALIEN_SPAWN_PERIOD;
                game.aliens.push(Alien::new());
            }
        }
        _ => (),
    }
}

fn draw_ship(position: &Vec2, sprite: &Sprite) {
    let (x, y) = (position.x, position.y);
    let &Sprite {
        ref variant,
        size: radius,
        angle,
        color,
    } = sprite;
    let has_exhaust = *variant == SpriteVariant::Ship { has_exhaust: true };
    use macroquad::prelude::{DrawMode, Vertex};
    let gl = unsafe { window::get_internal_gl().quad_gl };
    let vertices = [
        (
            x + (angle + PI * 0.0).cos() * radius,
            y + (angle + PI * 0.0).sin() * radius,
        ),
        (
            x + (angle + PI * 0.75).cos() * radius,
            y + (angle + PI * 0.75).sin() * radius,
        ),
        (
            x + (angle + PI * 0.79).cos() * radius * 0.4,
            y + (angle + PI * 0.79).sin() * radius * 0.4,
        ),
        (
            x + (angle + PI * 1.21).cos() * radius * 0.4,
            y + (angle + PI * 1.21).sin() * radius * 0.4,
        ),
        (
            x + (angle + PI * 1.25).cos() * radius,
            y + (angle + PI * 1.25).sin() * radius,
        ),
    ]
    .map(|(x, y)| Vertex::new(x, y, 0.0, 0.0, 0.0, color));
    let indices = [0, 1, 2, 0, 2, 3, 0, 3, 4];
    gl.texture(None);
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
    if has_exhaust {
        shapes::draw_triangle(
            math::vec2(
                x + (angle + PI * 0.85).cos() * radius * 0.55,
                y + (angle + PI * 0.85).sin() * radius * 0.55,
            ),
            math::vec2(
                x + (angle + PI * rand::gen_range(0.98, 1.02)).cos()
                    * radius
                    * rand::gen_range(0.75, 1.25),
                y + (angle + PI * rand::gen_range(0.98, 1.02)).sin()
                    * radius
                    * rand::gen_range(0.75, 1.25),
            ),
            math::vec2(
                x + (angle + PI * 1.15).cos() * radius * 0.55,
                y + (angle + PI * 1.15).sin() * radius * 0.55,
            ),
            color::Color::new(1.0, 1.0, 1.0, 1.0),
        );
    }
}

fn draw_polygon(draw_points: &[(f32, f32)], offset_x: f32, offset_y: f32, color: color::Color) {
    use macroquad::prelude::{DrawMode, Vertex};
    let gl = unsafe { window::get_internal_gl().quad_gl };
    let vertices: Vec<_> = draw_points
        .iter()
        .map(|&(x, y)| Vertex::new(x + offset_x, y + offset_y, 0.0, 0.0, 0.0, color))
        .collect();
    let indices: Vec<_> = (1..(draw_points.len() as u16 - 1))
        .flat_map(|i| [0, i, i + 1])
        .collect();
    gl.texture(None);
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
}

fn draw(game: &mut Game) {
    for explosion in &mut game.explosions {
        explosion.emitter.draw(explosion.position);
    }
    for y in -1..=1 {
        for x in -1..=1 {
            let offset_x = x as f32 * ARENA_WIDTH;
            let offset_y = y as f32 * ARENA_HEIGHT;
            if let Some(Ship { position, sprite, .. }) = &game.ship {
                let position = *position + vec2(offset_x, offset_y);
                draw_ship(&position, sprite);
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + body.angle.cos() * body.speed * 0.2,
                //     position.y + body.angle.sin() * body.speed * 0.2,
                //     3.0,
                //     color::GREEN,
                // );
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + sprite.angle.cos() * SHIP_ACCEL * 0.2,
                //     position.y + sprite.angle.sin() * SHIP_ACCEL * 0.2,
                //     2.0,
                //     color::BLUE,
                // );
                // let (result_speed, result_angle) =
                //     sum_vectors(body.speed, body.angle, SHIP_ACCEL, sprite.angle);
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + result_angle.cos() * result_speed * 0.2,
                //     position.y + result_angle.sin() * result_speed * 0.2,
                //     1.0,
                //     color::RED,
                // );
            }
            for Bullet { position, sprite, .. } in &game.bullets {
                shapes::draw_line(
                    position.x + offset_x,
                    position.y + offset_y,
                    position.x + offset_x + sprite.angle.cos() * 10.0,
                    position.y + offset_y + sprite.angle.sin() * 10.0,
                    2.0,
                    sprite.color,
                );
            }
            for Asteroid { position, sprite, .. } in &game.asteroids {
                if let SpriteVariant::Asteroid { ref draw_points } = &sprite.variant {
                    draw_polygon(
                        &draw_points,
                        position.x + offset_x,
                        position.y + offset_y,
                        sprite.color,
                    );
                } else {
                    unreachable!();
                }
            }
            for Alien { position, sprite, .. } in &game.aliens {
                let angle_by_x = (position.x * 2.0) % 360.0;
                shapes::draw_poly(
                    position.x,
                    position.y + offset_y,
                    6,
                    sprite.size,
                    angle_by_x,
                    sprite.color,
                );
            }
        }
    }

    // Debug info
    let color = color::Color::new(0.5, 0.5, 0.5, 1.0);
    let ship = game.ship.as_ref();
    [
        ("ship.sprite.angle", ship.map_or(0.0, |sh| sh.sprite.angle)),
        ("ship.position.x", ship.map_or(0.0, |sh| sh.position.x)),
        ("ship.position.y", ship.map_or(0.0, |sh| sh.position.y)),
        ("ship.body.angle", ship.map_or(0.0, |sh| sh.body.angle)),
        ("ship.body.speed", ship.map_or(0.0, |sh| sh.body.speed)),
        ("bullet_0.position.x", game.bullets.get(0).map_or(0.0, |b| b.position.x)),
        ("bullet_0.position.y", game.bullets.get(0).map_or(0.0, |b| b.position.y)),
        ("expl_0.position.x", game.explosions.get(0).map_or(0.0, |e| e.position.x)),
        ("expl_0.position.y", game.explosions.get(0).map_or(0.0, |e| e.position.y)),
        ("alien_0.position.x", game.aliens.get(0).map_or(0.0, |a| a.position.x)),
        ("alien_0.position.y", game.aliens.get(0).map_or(0.0, |a| a.position.y)),
    ]
    .iter()
    .enumerate()
    .for_each(|(i, (name, val))| {
        text::draw_text(&format!("{}: {}", name, val), 0.0, 16.0 * (i + 1) as f32, 16.0, color)
    });
}

fn are_circles_intersecting(
    a_x: f32,
    a_y: f32,
    a_radius: f32,
    b_x: f32,
    b_y: f32,
    b_radius: f32,
) -> bool {
    return (a_x - b_x).powf(2.0) + (a_y - b_y).powf(2.0) <= (a_radius + b_radius).powf(2.0);
}

fn sum_vectors(v1_magnitude: f32, v1_angle: f32, v2_magnitude: f32, v2_angle: f32) -> (f32, f32) {
    let d_rotation = v2_angle - v1_angle;
    let d_angle = 2.0 * (d_rotation % PI) - d_rotation;
    // https://www.mathstopia.net/vectors/parallelogram-law-vector-addition
    // = √(A² + B² + 2ABcosα)
    let result_magnitude = f32::sqrt(
        v1_magnitude.powi(2)
            + v2_magnitude.powi(2)
            + 2.0 * v1_magnitude * v2_magnitude * d_angle.cos(),
    );
    // find angle with three sides (the Law of Cosines)
    // https://en.wikipedia.org/wiki/Solution_of_triangles#Three_sides_given_(SSS)
    // = arccos((B² + C² - A²)/(2BC))
    let mut d_angle_r = f32::acos(
        (v1_magnitude.powi(2) + result_magnitude.powi(2) - v2_magnitude.powi(2))
            / (2.0 * v1_magnitude * result_magnitude),
    ) * d_angle.signum();
    if d_angle_r.is_nan() {
        d_angle_r = 0.0;
    }
    let result_angle = (v1_angle + d_angle_r).rem_euclid(2.0 * PI);
    (result_magnitude, result_angle)
}

fn asteroid_explosion() -> particles::EmitterConfig {
    particles::EmitterConfig {
        one_shot: true,
        lifetime: 0.65,
        explosiveness: 1.0,
        amount: 8,
        local_coords: true,
        initial_direction: math::vec2(0.0, 1.0),
        initial_direction_spread: 2.0 * PI,
        initial_velocity: 150.0,
        initial_velocity_randomness: 0.4,
        size: 3.0,
        shape: particles::ParticleShape::Circle { subdivisions: 7 },
        colors_curve: particles::ColorCurve {
            start: color::Color::new(0.6, 0.6, 0.0, 1.0),
            mid: color::Color::new(0.6, 0.6, 0.0, 1.0),
            end: color::Color::new(0.0, 0.0, 0.0, 1.0),
        },
        ..Default::default()
    }
}

fn ship_explosion(color: color::Color) -> particles::EmitterConfig {
    particles::EmitterConfig {
        one_shot: true,
        lifetime: 1.5,
        explosiveness: 1.0,
        amount: 8,
        local_coords: true,
        initial_direction: math::vec2(0.0, 1.0),
        initial_direction_spread: 2.0 * PI,
        initial_velocity: 50.0,
        initial_velocity_randomness: 0.4,
        size: 5.0,
        shape: particles::ParticleShape::Circle { subdivisions: 6 },
        colors_curve: particles::ColorCurve {
            start: color,
            mid: color,
            end: color::Color::new(0.0, 0.0, 0.0, 1.0),
        },
        ..Default::default()
    }
}

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: String::from("asteroids"),
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(date::now() as u64);
    let mut game = Game::default();
    load(&mut game);
    loop {
        let delta_time = time::get_frame_time();
        input_system_update(&mut game, delta_time);
        ai_system_update(&mut game, delta_time);
        timers_system_update(&mut game, delta_time);
        move_system_update(&mut game, delta_time);
        collision_system_update(&mut game, delta_time);
        damage_system_update(&mut game, delta_time);
        gamestate_system_update(&mut game, delta_time);
        cleanup_system_update(&mut game, delta_time);
        spawn_system_update(&mut game, delta_time);
        draw(&mut game);
        window::next_frame().await;
    }
}
