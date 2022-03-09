use std::collections::HashSet;
use std::f32::consts::PI;

use macroquad::{
    camera, color, input, material, math, miniquad::date, rand, shapes, text, texture, time, window,
};
use macroquad_particles as particles;

const ARENA_WIDTH: f32 = 432.0;
const ARENA_HEIGHT: f32 = 240.0; // 600 * 0.4
const SHIP_MAX_SPEED: f32 = 160.0;
const SHIP_ACCEL: f32 = 200.0;
const SHIP_DECEL: f32 = 0.08;
const SHIP_DRAW_RADIUS: f32 = 5.6;
const SHIP_HIT_RADIUS: f32 = 3.6;
const SHIP_TURN_SPEED: f32 = 6.0;
const SHIP_BULLET_COLOR: color::Color = color::Color::new(1.0, 0.0, 0.0, 1.0);
const SHIP_BULLET_TIMER_LIMIT: f32 = 0.8;
const SHIP_BULLET_SPEED: f32 = 240.0;
const BULLET_COOLDOWN: f32 = 0.3;
const BULLET_RADIUS: f32 = 1.2;
const ALIEN_DRAW_RADIUS_BY_KIND: &[f32] = &[5.6, 4.0];
const ALIEN_HIT_RADIUS_BY_KIND: &[f32] = &[4.0, 2.8];
const ALIEN_BULLET_TIMER_LIMIT_BY_KIND: &[f32] = &[0.9, 1.3];
const ALIEN_BULLET_SPEED: f32 = 100.0;
const ALIEN_BULLET_COLOR: color::Color = color::Color::new(1.0, 0.0, 1.0, 1.0);
const ALIEN_SPAWN_PERIOD: f32 = 30.0;
const ALIEN_SHOOT_PERIOD: f32 = 1.3;
const ALIEN_SHIFT_PERIOD: f32 = 1.0;
const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage { max_speed: 72.0, radius: 4.8 },
    AsteroidStage { max_speed: 48.0, radius: 11.2 },
    AsteroidStage { max_speed: 24.0, radius: 16.0 },
];
const KEYMAP: &[(input::KeyCode, Action)] = &[
    (input::KeyCode::Up, Action::Accelerate),
    (input::KeyCode::Left, Action::TurnLeft),
    (input::KeyCode::Right, Action::TurnRight),
    (input::KeyCode::S, Action::Shoot),
    (input::KeyCode::Escape, Action::TogglePause),
];

struct AsteroidStage {
    max_speed: f32,
    radius: f32,
}

#[derive(PartialEq)]
enum SpriteVariant {
    Bullet,
    Ship { has_exhaust: bool },
    Asteroid { draw_points: Vec<math::Vec2> },
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
    life_timer: f32,
    from_enemy: bool,
}

impl Bullet {
    pub fn new(position: math::Vec2, angle: f32, alien_kind: Option<AlienKind>) -> Self {
        let (color, speed, life_timer) = if let Some(kind) = alien_kind {
            (
                ALIEN_BULLET_COLOR,
                ALIEN_BULLET_SPEED,
                ALIEN_BULLET_TIMER_LIMIT_BY_KIND[kind as usize],
            )
        } else {
            (SHIP_BULLET_COLOR, SHIP_BULLET_SPEED, SHIP_BULLET_TIMER_LIMIT)
        };
        Bullet {
            position,
            sprite: Sprite {
                variant: SpriteVariant::Bullet,
                size: f32::NAN,
                angle,
                color,
            },
            body: Body {
                radius: BULLET_RADIUS,
                angle,
                speed,
                is_hit: false,
            },
            life_timer,
            from_enemy: alien_kind.is_some(),
        }
    }
}

struct Ship {
    position: math::Vec2,
    sprite: Sprite,
    body: Body,
    is_destroyed: bool,
    weapon_cooldown_timer: f32,
}

impl Ship {
    pub fn new() -> Self {
        Ship {
            position: math::vec2(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0),
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
            weapon_cooldown_timer: 0.0,
        }
    }
}

struct Explosion {
    position: math::Vec2,
    body: Body,
    emitter: particles::Emitter,
    life_timer: f32,
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
    weapon_cooldown_timer: f32,
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
            position: math::vec2(x, y),
            sprite: Sprite {
                variant: SpriteVariant::Alien,
                size: ALIEN_DRAW_RADIUS_BY_KIND[kind as usize],
                angle: 0.0,
                color: color::Color::new(0.0, 1.0, 0.0, 1.0),
            },
            body: Body {
                radius: ALIEN_HIT_RADIUS_BY_KIND[kind as usize],
                angle,
                speed: rand::gen_range(32.0, 40.0),
                is_hit: false,
            },
            is_destroyed: false,
            kind,
            direction,
            weapon_cooldown_timer: ALIEN_SHOOT_PERIOD,
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
    pub fn new(position: math::Vec2, stage: usize) -> Self {
        let radius = ASTEROID_STAGES[stage].radius + 2.0;
        let max_speed = ASTEROID_STAGES[stage].max_speed;
        let mut draw_points = vec![];
        let mut draw_angle: f32 = 0.0;
        while draw_angle < PI * 2.0 {
            let distance = rand::gen_range(0.75, 1.0) * radius;
            draw_points.push(math::vec2(draw_angle.cos() * distance, draw_angle.sin() * distance));
            draw_angle += rand::gen_range(0.1, 0.7);
        }
        let angle = rand::gen_range(0.0, 2.0 * PI);
        let speed = max_speed * rand::gen_range(0.5, 1.0);
        Asteroid {
            position,
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

struct StarBackground {
    static_emitter: particles::Emitter,
    side_emitter: particles::Emitter,
    side_emitter_pos: math::Vec2,
}

impl Default for StarBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl StarBackground {
    pub fn new() -> Self {
        let static_cfg = stars();
        let mut side_cfg = stars();
        let side_emitter_pos;
        side_cfg.lifetime_randomness = 0.0;
        side_cfg.explosiveness = 0.0;
        side_cfg.size_curve = None;
        side_cfg.amount = 100;
        // from left, from right, from top:
        match rand::gen_range::<i32>(0, 3) {
            multiplier @ (0 | 1) => {
                side_cfg.initial_direction =
                    math::vec2(1.0 - 2.0 * multiplier as f32, rand::gen_range(-0.3, 0.3));
                side_cfg.emission_shape = particles::EmissionShape::Rect {
                    width: 0.0,
                    height: ARENA_HEIGHT * 1.2,
                };
                side_emitter_pos = math::vec2(ARENA_WIDTH * multiplier as f32, ARENA_HEIGHT / 2.0);
            }
            2 => {
                side_cfg.initial_direction = math::vec2(rand::gen_range(-0.3, 0.3), 1.0);
                side_cfg.emission_shape = particles::EmissionShape::Rect {
                    width: ARENA_WIDTH * 1.2,
                    height: 0.0,
                };
                side_emitter_pos = math::vec2(ARENA_WIDTH / 2.0, 0.0);
            }
            _ => unreachable!(),
        }
        StarBackground {
            static_emitter: particles::Emitter::new(static_cfg),
            side_emitter: particles::Emitter::new(side_cfg),
            side_emitter_pos,
        }
    }
}

struct Renderer {
    canvas: macroquad_canvas::Canvas2D,
    crt_effect: Option<material::Material>,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut canvas = macroquad_canvas::Canvas2D::new(ARENA_WIDTH, ARENA_HEIGHT);
        canvas
            .get_texture_mut()
            .set_filter(texture::FilterMode::Nearest);
        Renderer { canvas, crt_effect: None }
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Action {
    Accelerate,
    TurnLeft,
    TurnRight,
    Shoot,
    TogglePause,
}

#[derive(Default)]
struct Game {
    state: GameState,
    renderer: Renderer,
    player_actions: HashSet<Action>,
    break_timer: f32,
    alien_timer: f32,
    star_bg: StarBackground,
    ship: Option<Ship>,
    bullets: Vec<Bullet>,
    asteroids: Vec<Asteroid>,
    explosions: Vec<Explosion>,
    aliens: Vec<Alien>,
}

fn load(game: &mut Game) {
    game.renderer.crt_effect = Some(
        material::load_material(CRT_VERTEX_SHADER, CRT_FRAGMENT_SHADER, Default::default())
            .unwrap(),
    );
    game.alien_timer = ALIEN_SPAWN_PERIOD;
}

fn input_system_update(game: &mut Game, _dt: f32) {
    game.player_actions.clear();
    match game.state {
        GameState::Pause => {
            use Action::*;
            let pause_key = match KEYMAP[4] {
                (pause_key, TogglePause) => pause_key,
                _ => unreachable!(),
            };
            if input::is_key_pressed(pause_key) {
                game.player_actions.insert(TogglePause);
            }
        }
        GameState::LevelRunning => {
            use Action::*;
            for &(key, action) in KEYMAP {
                if action == TogglePause {
                    if input::is_key_pressed(key) {
                        game.player_actions.insert(action);
                    }
                } else {
                    if input::is_key_down(key) {
                        game.player_actions.insert(action);
                    }
                }
            }
        }
        _ => (),
    }
}

fn ai_system_update(game: &mut Game, _dt: f32) {
    for alien in &mut game.aliens {
        let time_to_shift = alien.shift_timer == 0.0;
        let time_to_shoot = alien.weapon_cooldown_timer == 0.0;
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
            alien.weapon_cooldown_timer = ALIEN_SHOOT_PERIOD;
            let ship = game.ship.as_ref().unwrap();
            let radius = ALIEN_HIT_RADIUS_BY_KIND[alien.kind as usize] / 2.0;
            let shoot_angle =
                f32::atan2(ship.position.y - alien.position.y, ship.position.x - alien.position.x);
            let weapon_position =
                alien.position + math::vec2(shoot_angle.cos() * radius, shoot_angle.sin() * radius);
            game.bullets
                .push(Bullet::new(weapon_position, shoot_angle, Some(alien.kind)));
        }
        if alien.position.x < -4.0 || ARENA_WIDTH + 4.0 < alien.position.x {
            alien.is_destroyed = true;
        }
    }
}

fn timers_system_update(game: &mut Game, dt: f32) {
    match game.state {
        GameState::Pause => (),
        _ => {
            game.alien_timer = f32::max(0.0, game.alien_timer - dt);
            game.break_timer = f32::max(0.0, game.break_timer - dt);
            if let Some(ship) = &mut game.ship {
                ship.weapon_cooldown_timer = f32::max(0.0, ship.weapon_cooldown_timer - dt);
            }
            for explosion in &mut game.explosions {
                explosion.life_timer = f32::max(0.0, explosion.life_timer - dt);
            }
            for bullet in &mut game.bullets {
                bullet.life_timer = f32::max(0.0, bullet.life_timer - dt);
            }
            for alien in &mut game.aliens {
                alien.shift_timer = f32::max(0.0, alien.shift_timer - dt);
                alien.weapon_cooldown_timer = f32::max(0.0, alien.weapon_cooldown_timer - dt);
            }
        }
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
            if let Some(Ship { position, body, .. }) = &mut game.ship {
                body.speed -= body.speed * SHIP_DECEL * dt;
                move_position(position, body, dt, true, true);
            }
            for Alien { position, body, .. } in &mut game.aliens {
                move_position(position, body, dt, false, true);
            }
            for Bullet { position, body, .. } in &mut game.bullets {
                move_position(position, body, dt, true, true);
            }
            for Asteroid { position, body, .. } in &mut game.asteroids {
                move_position(position, body, dt, true, true);
            }
            for Explosion { position, body, .. } in &mut game.explosions {
                move_position(position, body, dt, true, true);
            }
        }
    }
}

fn collision_system_update(game: &mut Game, _dt: f32) {
    fn do_collision(a_pos: math::Vec2, a_body: &mut Body, b_pos: math::Vec2, b_body: &mut Body) {
        let d_pos = a_pos - b_pos;
        let is_intersecting =
            d_pos.x.powi(2) + d_pos.y.powi(2) <= (a_body.radius + b_body.radius).powi(2);
        if is_intersecting {
            a_body.is_hit = true;
            b_body.is_hit = true;
        }
    }
    let (mut enemy_bullets, mut ship_bullets): (Vec<_>, _) =
        game.bullets.iter_mut().partition(|b| b.from_enemy);
    for alien in &mut game.aliens {
        for ship in &mut game.ship {
            do_collision(alien.position, &mut alien.body, ship.position, &mut ship.body);
        }
    }
    for enemy_bullet in &mut enemy_bullets {
        for ship in &mut game.ship {
            do_collision(
                enemy_bullet.position,
                &mut enemy_bullet.body,
                ship.position,
                &mut ship.body,
            );
        }
        for asteroid in &mut game.asteroids {
            do_collision(
                enemy_bullet.position,
                &mut enemy_bullet.body,
                asteroid.position,
                &mut asteroid.body,
            );
        }
    }
    for ship_bullet in &mut ship_bullets {
        for enemy_bullet in &mut enemy_bullets {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                enemy_bullet.position,
                &mut enemy_bullet.body,
            );
        }
        for alien in &mut game.aliens {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                alien.position,
                &mut alien.body,
            );
        }
        for asteroid in &mut game.asteroids {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                asteroid.position,
                &mut asteroid.body,
            );
        }
    }
    for asteroid in &mut game.asteroids {
        for ship in &mut game.ship {
            do_collision(asteroid.position, &mut asteroid.body, ship.position, &mut ship.body);
        }
        for alien in &mut game.aliens {
            do_collision(asteroid.position, &mut asteroid.body, alien.position, &mut alien.body);
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
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    for alien in game.aliens.iter_mut().filter(|a| a.body.is_hit) {
        alien.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(ship_explosion(alien.sprite.color));
        game.explosions.push(Explosion {
            position: alien.position,
            body: Body {
                angle: alien.body.angle,
                speed: alien.body.speed,
                ..Default::default()
            },
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    game.bullets.retain(|b| !b.body.is_hit);
    let mut new_asteroids: Vec<Asteroid> = Default::default();
    for asteroid in game.asteroids.iter_mut().filter(|a| a.body.is_hit) {
        asteroid.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(asteroid_explosion());
        game.explosions.push(Explosion {
            position: asteroid.position,
            body: Body {
                angle: asteroid.body.angle,
                speed: asteroid.body.speed * 1.5,
                ..Default::default()
            },
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
        if asteroid.stage > 0 {
            new_asteroids.push(Asteroid::new(asteroid.position, asteroid.stage - 1));
            new_asteroids.push(Asteroid::new(asteroid.position, asteroid.stage - 1));
        }
    }
    game.asteroids.append(&mut new_asteroids);
}

fn gamestate_system_update(game: &mut Game, _dt: f32) {
    // dbg!(&game.state);
    match game.state {
        GameState::Pause => {
            if game.player_actions.contains(&Action::TogglePause) {
                game.state = GameState::LevelRunning;
            }
        }
        GameState::LevelLoading => {
            if game.ship.is_some() && !game.asteroids.is_empty() {
                game.state = GameState::LevelRunning;
            }
        }
        GameState::LevelRunning => {
            if game.player_actions.contains(&Action::TogglePause) {
                game.state = GameState::Pause;
            }
            if game.ship.as_ref().map_or(false, |sh| sh.is_destroyed) {
                game.break_timer = 2.0;
                game.state = GameState::GameOver;
            }
            if game.asteroids.is_empty() && game.aliens.is_empty() {
                game.break_timer = 2.0;
                game.state = GameState::LevelCompleted;
            }
        }
        GameState::LevelCompleted => {
            if game.break_timer == 0.0 {
                let old_game = std::mem::replace(game, Default::default());
                game.renderer = old_game.renderer;
                game.alien_timer = old_game.alien_timer;
                game.star_bg = old_game.star_bg;
            }
        }
        GameState::GameOver => {
            if game.break_timer == 0.0 {
                let old_game = std::mem::replace(game, Default::default());
                game.renderer = old_game.renderer;
                game.alien_timer = old_game.alien_timer;
                game.star_bg = old_game.star_bg;
            }
        }
    }
}

fn cleanup_system_update(game: &mut Game, _dt: f32) {
    game.ship = game.ship.take().filter(|sh| !sh.is_destroyed);
    game.aliens.retain(|a| !a.is_destroyed);
    game.bullets.retain(|b| b.life_timer > 0.0);
    game.asteroids.retain(|a| !a.is_destroyed);
    game.explosions.retain(|e| e.life_timer > 0.0);
}

fn spawn_system_update(game: &mut Game, _dt: f32) {
    match game.state {
        GameState::LevelLoading => {
            if game.ship.is_none() {
                game.ship = Some(Ship::new());
            }
            if game.asteroids.is_empty() {
                let ship = game.ship.as_ref().unwrap();
                let start_stage = ASTEROID_STAGES.len() - 1;
                while game.asteroids.len() < 5 {
                    let rand_pos = math::vec2(
                        rand::gen_range(0.0, ARENA_WIDTH),
                        rand::gen_range(0.0, ARENA_HEIGHT),
                    );
                    let delta_pos = rand_pos - ship.position;
                    const RADIUS: f32 = ARENA_HEIGHT * 0.3;
                    let is_too_close = delta_pos.x.powi(2) + delta_pos.y.powi(2) <= RADIUS.powi(2);
                    if !is_too_close {
                        game.asteroids.push(Asteroid::new(rand_pos, start_stage));
                    }
                }
            }
        }
        GameState::LevelRunning => {
            if let Some(ship) = &mut game.ship {
                let shoot_is_ready = ship.weapon_cooldown_timer == 0.0;
                if game.player_actions.contains(&Action::Shoot) && shoot_is_ready {
                    ship.weapon_cooldown_timer = BULLET_COOLDOWN;
                    let bullet_offset = math::vec2(
                        ship.sprite.angle.cos() * SHIP_DRAW_RADIUS,
                        ship.sprite.angle.sin() * SHIP_DRAW_RADIUS,
                    );
                    game.bullets.push(Bullet::new(
                        ship.position + bullet_offset,
                        ship.sprite.angle,
                        None,
                    ));
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

fn draw_ship(smooth_pos: math::Vec2, sprite: &Sprite) {
    // let (x, y) = (position.x, position.y);
    let position = math::vec2(smooth_pos.x as i32 as f32, smooth_pos.y as i32 as f32);
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
        ((angle + PI * 0.0).cos() * radius, (angle + PI * 0.0).sin() * radius),
        ((angle + PI * 0.75).cos() * radius, (angle + PI * 0.75).sin() * radius),
        (
            (angle + PI * 0.79).cos() * radius * 0.4,
            (angle + PI * 0.79).sin() * radius * 0.4,
        ),
        (
            (angle + PI * 1.21).cos() * radius * 0.4,
            (angle + PI * 1.21).sin() * radius * 0.4,
        ),
        ((angle + PI * 1.25).cos() * radius, (angle + PI * 1.25).sin() * radius),
    ]
    .map(|(x, y)| Vertex::new(position.x + x, position.y + y, 0.0, 0.0, 0.0, color));
    let indices = [0, 1, 2, 0, 2, 3, 0, 3, 4];
    gl.texture(None);
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
    if has_exhaust {
        let v1_offset = math::vec2(
            (angle + PI * 0.85).cos() * radius * 0.55,
            (angle + PI * 0.85).sin() * radius * 0.55,
        );
        let v2_offset = math::vec2(
            (angle + PI * rand::gen_range(0.98, 1.02)).cos() * radius * rand::gen_range(0.75, 1.25),
            (angle + PI * rand::gen_range(0.98, 1.02)).sin() * radius * rand::gen_range(0.75, 1.25),
        );
        let v3_offset = math::vec2(
            (angle + PI * 1.15).cos() * radius * 0.55,
            (angle + PI * 1.15).sin() * radius * 0.55,
        );
        shapes::draw_triangle(
            position + v1_offset,
            position + v2_offset,
            position + v3_offset,
            color::Color::new(1.0, 1.0, 1.0, 1.0),
        );
    }
}

fn draw_polygon(draw_points: &[math::Vec2], smooth_offset: math::Vec2, color: color::Color) {
    use macroquad::prelude::{DrawMode, Vertex};
    let offset = math::vec2(smooth_offset.x as i32 as f32, smooth_offset.y as i32 as f32);
    let gl = unsafe { window::get_internal_gl().quad_gl };
    let vertices: Vec<_> = draw_points
        .iter()
        .map(|&p| p + offset)
        .map(|p| Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, color))
        .collect();
    let indices: Vec<_> = (1..(draw_points.len() as u16 - 1))
        .flat_map(|i| [0, i, i + 1])
        .collect();
    gl.texture(None);
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
}

fn draw(game: &mut Game) {
    camera::set_camera(&game.renderer.canvas.camera);
    window::clear_background(color::Color::new(0.1, 0.1, 0.1, 1.0));
    game.star_bg
        .static_emitter
        .draw(math::vec2(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0));
    game.star_bg
        .side_emitter
        .draw(game.star_bg.side_emitter_pos);
    for explosion in &mut game.explosions {
        explosion.emitter.draw(explosion.position);
    }
    for y in -1..=1 {
        for x in -1..=1 {
            let offset = math::vec2(x as f32 * ARENA_WIDTH, y as f32 * ARENA_HEIGHT);
            for Ship { position, sprite, .. } in &game.ship {
                let position = *position + offset;
                draw_ship(position, sprite);
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
                let position = *position + offset;
                shapes::draw_line(
                    position.x,
                    position.y,
                    position.x + sprite.angle.cos() * 4.0,
                    position.y + sprite.angle.sin() * 4.0,
                    2.0,
                    sprite.color,
                );
            }
            for Asteroid { position, sprite, .. } in &game.asteroids {
                let draw_points = match &sprite.variant {
                    SpriteVariant::Asteroid { draw_points } => draw_points,
                    _ => unreachable!(),
                };
                let position = *position + offset;
                draw_polygon(draw_points, position, sprite.color);
            }
            for Alien { position, sprite, .. } in &game.aliens {
                let angle_by_x = (position.x * 2.0) % 144.0;
                shapes::draw_poly(
                    position.x,
                    position.y + offset.y,
                    6,
                    sprite.size,
                    angle_by_x,
                    sprite.color,
                );
            }
        }
    }
    camera::set_default_camera();
    material::gl_use_material(game.renderer.crt_effect.unwrap());
    game.renderer.canvas.draw();
    material::gl_use_default_material();

    // Debug info
    let color = color::Color::new(0.5, 0.5, 0.5, 1.0);
    let ship = game.ship.as_ref();
    [
        ("fps", time::get_fps() as f32),
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

fn move_position(position: &mut math::Vec2, body: &Body, dt: f32, wrap_x: bool, wrap_y: bool) {
    *position += math::vec2(body.angle.cos() * body.speed * dt, body.angle.sin() * body.speed * dt);
    if wrap_x {
        position.x = position.x.rem_euclid(ARENA_WIDTH);
    }
    if wrap_y {
        position.y = position.y.rem_euclid(ARENA_HEIGHT);
    }
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
        initial_velocity: 60.0,
        initial_velocity_randomness: 0.4,
        size: 1.2,
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
        initial_velocity: 20.0,
        initial_velocity_randomness: 0.4,
        size: 2.0,
        shape: particles::ParticleShape::Circle { subdivisions: 6 },
        colors_curve: particles::ColorCurve {
            start: color,
            mid: color,
            end: color::Color::new(0.0, 0.0, 0.0, 1.0),
        },
        ..Default::default()
    }
}

fn stars() -> particles::EmitterConfig {
    particles::EmitterConfig {
        lifetime: 70.0,
        lifetime_randomness: 0.4,
        amount: 50,
        explosiveness: 0.2,
        local_coords: true,
        initial_direction: math::vec2(0.001, 0.0),
        initial_velocity: 28.0,
        initial_velocity_randomness: 0.8,
        size: 1.2,
        size_randomness: 0.6,
        size_curve: Some(particles::Curve {
            points: vec![(0.0, 0.0), (0.05, 1.0), (0.85, 0.8), (1.0, 0.0)],
            ..Default::default()
        }),
        emission_shape: particles::EmissionShape::Rect {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
        },
        shape: particles::ParticleShape::Circle { subdivisions: 4 },
        ..Default::default()
    }
}

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: String::from("asteroids"),
        // window_resizable: false,
        window_width: ARENA_WIDTH as i32 * 3,
        window_height: ARENA_HEIGHT as i32 * 3,
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

const CRT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;

varying vec4 color;
varying vec2 uv;
    
uniform sampler2D Texture;

// https://www.shadertoy.com/view/XtlSD7

void DrawScanline( inout vec3 color, vec2 uv )
{
    // float iTime = 2.0;
    // float scanline 	= clamp( 0.85 + 0.15 * cos( 3.14 * ( uv.y + 0.008 * iTime ) * 240.0 * 1.0 ), 0.0, 1.0 );
    // float grille 	= 0.85 + 0.15 * clamp( 1.5 * cos( 3.14 * uv.x * 432.0 * 1.0 ), 0.0, 1.0 );
    float scanline 	= clamp(0.85 + 0.15 * cos(3.14 * (uv.y + 0.0014) * 240.0), 0.0, 1.0);
    float grille 	= 0.94 + 0.06 * clamp(mod(uv.x * 432.0, 1.3) * 2.0, 0.0, 1.0);
    color *= scanline * grille * 1.2;
}

void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    DrawScanline(res, uv);
    gl_FragColor = vec4(res, 1.0);
}
"#;

const CRT_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";
