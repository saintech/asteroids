use std::f32::consts::PI;

use macroquad::{color, input, math, miniquad::date, rand, shapes, text, time, window};
use macroquad_particles as particles;

const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const SHIP_MAX_SPEED: f32 = 400.0;
const SHIP_ACCEL: f32 = 500.0;
const SHIP_DECEL: f32 = 0.2;
const SHIP_DRAW_RADIUS: f32 = 14.0;
const SHIP_HIT_RADIUS: f32 = 9.0;
const SHIP_TURN_SPEED: f32 = 6.0;
const SHIP_BULLET_TIMER_LIMIT: f32 = 0.8;
const SHIP_BULLET_SPEED: f32 = 600.0;
const BULLET_COOLDOWN: f32 = 0.3;
const BULLET_RADIUS: f32 = 3.0;
const ALIEN_DRAW_RADIUS_BY_KIND: &[f32] = &[14.0, 10.0];
const ALIEN_HIT_RADIUS_BY_KIND: &[f32] = &[10.0, 7.0];
const ALIEN_BULLET_TIMER_LIMIT_BY_KIND: &[f32] = &[0.9, 1.3];
const ALIEN_BULLET_SPEED: f32 = 250.0;
const ALIEN_SPAWN_PERIOD: f32 = 30.0;
const ALIEN_SHOOT_PERIOD: f32 = 1.3;
const ALIEN_SHIFT_PERIOD: f32 = 1.0;
const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage {
        max_speed: 180.0,
        radius: 12.0,
    },
    AsteroidStage {
        max_speed: 120.0,
        radius: 28.0,
    },
    AsteroidStage {
        max_speed: 60.0,
        radius: 40.0,
    },
];

struct Bullet {
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
    time_left: f32,
    from_enemy: bool,
}

struct Explosion {
    emitter: particles::Emitter,
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
    time_left: f32,
}

struct AsteroidStage {
    max_speed: f32,
    radius: f32,
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
    x: f32,
    y: f32,
    kind: AlienKind,
    direction: AlienDirection,
    angle: f32,
    speed: f32,
    shoot_timer: f32,
    shift_timer: f32,
}

impl Alien {
    pub fn new() -> Self {
        let y = ARENA_HEIGHT * rand::gen_range(0.15, 0.85);
        let direction =
            vec![AlienDirection::ToRight, AlienDirection::ToLeft].remove(rand::gen_range(0, 2));
        let x = ARENA_WIDTH * direction as u32 as f32;
        let angle = PI * direction as u32 as f32;
        let kind = if rand::gen_range(0_u32, 10) < 3 {
            AlienKind::Small
        } else {
            AlienKind::Big
        };
        Alien {
            x,
            y,
            kind,
            direction,
            angle,
            speed: rand::gen_range(80.0, 100.0),
            shoot_timer: ALIEN_SHOOT_PERIOD,
            shift_timer: 0.0,
        }
    }
}

struct Asteroid {
    draw_points: Vec<(f32, f32)>,
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
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
        return Asteroid {
            draw_points,
            x,
            y,
            angle,
            speed,
            stage,
        };
    }
}

#[derive(Default)]
struct GameState {
    pause_timer: f32,
    game_over: bool,
    ship_is_dead: bool,
    ship_x: f32,
    ship_y: f32,
    ship_angle: f32,
    ship_velocity_x: f32,
    ship_velocity_y: f32,
    ship_has_exhaust: bool,
    bullets: Vec<Bullet>,
    bullet_timer: f32,
    asteroids: Vec<Asteroid>,
    explosions: Vec<Explosion>,
    alien_timer: f32,
    aliens: Vec<Alien>,
}

fn reset(state: &mut GameState) {
    state.pause_timer = 0.0;
    state.game_over = false;
    state.ship_is_dead = false;
    state.ship_x = ARENA_WIDTH / 2.0;
    state.ship_y = ARENA_HEIGHT / 2.0;
    state.ship_angle = 0.0;
    state.ship_velocity_x = 0.0;
    state.ship_velocity_y = 0.0;
    state.bullets = Default::default();
    state.bullet_timer = BULLET_COOLDOWN;
    state.asteroids = Default::default();
    state.explosions = Default::default();
    state.aliens = Default::default();
    let start_stage = ASTEROID_STAGES.len() - 1;
    while state.asteroids.len() < 5 {
        let x = rand::gen_range(0.0, ARENA_WIDTH);
        let y = rand::gen_range(0.0, ARENA_HEIGHT);
        const RADIUS: i32 = (ARENA_HEIGHT * 0.3) as i32;
        let is_too_close =
            (x - state.ship_x) as i32 ^ 2 + (y - state.ship_y) as i32 ^ 2 <= RADIUS ^ 2;
        if !is_too_close {
            state.asteroids.push(Asteroid::new(x, y, start_stage));
        }
    }
}

fn load(state: &mut GameState) {
    state.alien_timer = ALIEN_SPAWN_PERIOD;
    reset(state);
}

fn key_pressed(_state: &mut GameState, _key: input::KeyCode) {
    // template
}

fn update(state: &mut GameState, dt: f32) {
    state.pause_timer = f32::max(0.0, state.pause_timer - dt);
    state.alien_timer = f32::max(0.0, state.alien_timer - dt);
    if input::is_key_down(input::KeyCode::Right) && state.pause_timer == 0.0 {
        state.ship_angle = state.ship_angle + SHIP_TURN_SPEED * dt;
    }
    if input::is_key_down(input::KeyCode::Left) && state.pause_timer == 0.0 {
        state.ship_angle = state.ship_angle - SHIP_TURN_SPEED * dt;
    }
    state.ship_angle = state.ship_angle.rem_euclid(2.0 * PI);
    state.ship_velocity_x = state.ship_velocity_x - (state.ship_velocity_x * SHIP_DECEL * dt);
    state.ship_velocity_y = state.ship_velocity_y - (state.ship_velocity_y * SHIP_DECEL * dt);
    if input::is_key_down(input::KeyCode::Up) && state.pause_timer == 0.0 {
        state.ship_has_exhaust = true;
        if f32::hypot(state.ship_velocity_x, state.ship_velocity_y) <= SHIP_MAX_SPEED {
            state.ship_velocity_x += state.ship_angle.cos() * SHIP_ACCEL * dt;
            state.ship_velocity_y += state.ship_angle.sin() * SHIP_ACCEL * dt;
        }
    } else {
        state.ship_has_exhaust = false;
    }
    state.ship_x = (state.ship_x + state.ship_velocity_x * dt).rem_euclid(ARENA_WIDTH);
    state.ship_y = (state.ship_y + state.ship_velocity_y * dt).rem_euclid(ARENA_HEIGHT);
    state.aliens.retain(|a| match a.direction {
        AlienDirection::ToRight => a.x - 10.0 < ARENA_WIDTH,
        AlienDirection::ToLeft => a.x + 10.0 > 0.0,
    });
    for alien in &mut state.aliens {
        alien.x = alien.x + alien.angle.cos() * alien.speed * dt;
        alien.y = (alien.y + alien.angle.sin() * alien.speed * dt).rem_euclid(ARENA_HEIGHT);
        alien.shift_timer -= dt;
        if alien.shift_timer <= 0.0 {
            alien.shift_timer = ALIEN_SHIFT_PERIOD;
            let d_angle = PI / 4.0;
            let origin_angle = PI * alien.direction as u32 as f32;
            alien.angle += d_angle * rand::gen_range(-2_i32, 2) as f32;
            alien.angle = alien
                .angle
                .clamp(origin_angle - d_angle, origin_angle + d_angle);
        }
        alien.shoot_timer -= dt;
        if alien.shoot_timer <= 0.0 {
            alien.shoot_timer = ALIEN_SHOOT_PERIOD;
            let bullet_time_left = ALIEN_BULLET_TIMER_LIMIT_BY_KIND[alien.kind as usize];
            let radius = ALIEN_HIT_RADIUS_BY_KIND[alien.kind as usize] / 2.0;
            let shoot_angle = f32::atan2(state.ship_y - alien.y, state.ship_x - alien.x);
            state.bullets.push(Bullet {
                x: alien.x + shoot_angle.cos() * radius,
                y: alien.y + shoot_angle.sin() * radius,
                angle: shoot_angle,
                speed: ALIEN_BULLET_SPEED,
                time_left: bullet_time_left,
                from_enemy: true,
            })
        }
    }
    'bullets: for bullet_index in (0..state.bullets.len()).rev() {
        let bullet = &mut state.bullets[bullet_index];
        bullet.time_left = bullet.time_left - dt;
        if bullet.time_left <= 0.0 {
            state.bullets.remove(bullet_index);
        } else {
            bullet.x = (bullet.x + bullet.angle.cos() * bullet.speed * dt).rem_euclid(ARENA_WIDTH);
            bullet.y = (bullet.y + bullet.angle.sin() * bullet.speed * dt).rem_euclid(ARENA_HEIGHT);
            let bullet = &state.bullets[bullet_index];
            for asteroid_index in (0..state.asteroids.len()).rev() {
                let asteroid_x = state.asteroids[asteroid_index].x;
                let asteroid_y = state.asteroids[asteroid_index].y;
                let asteroid_stage = state.asteroids[asteroid_index].stage;
                if are_circles_intersecting(
                    bullet.x,
                    bullet.y,
                    BULLET_RADIUS,
                    asteroid_x,
                    asteroid_y,
                    ASTEROID_STAGES[asteroid_stage].radius,
                ) {
                    state.bullets.remove(bullet_index);
                    explode_asteroid(&mut state.asteroids, asteroid_index, &mut state.explosions);
                    break 'bullets;
                }
            }
            if bullet.from_enemy
                && are_circles_intersecting(
                    bullet.x,
                    bullet.y,
                    BULLET_RADIUS,
                    state.ship_x,
                    state.ship_y,
                    SHIP_HIT_RADIUS,
                )
                && !state.ship_is_dead
            {
                state.bullets.remove(bullet_index);
                let color = color::Color::new(0.0, 1.0, 1.0, 1.0);
                let expl_emitter = particles::Emitter::new(ship_explosion(color));
                state.explosions.push(Explosion {
                    time_left: expl_emitter.config.lifetime,
                    emitter: expl_emitter,
                    x: state.ship_x,
                    y: state.ship_y,
                    angle: state.ship_angle,
                    speed: 0.0,
                });
                state.ship_is_dead = true;
                state.pause_timer = 2.0;
                break;
            }
            let bullets = &mut state.bullets;
            let explosions = &mut state.explosions;
            state.aliens.retain(|alien| {
                let bullet = &bullets[bullet_index];
                let is_hit = !bullet.from_enemy
                    && are_circles_intersecting(
                        bullet.x,
                        bullet.y,
                        BULLET_RADIUS,
                        alien.x,
                        alien.y,
                        ALIEN_HIT_RADIUS_BY_KIND[alien.kind as usize],
                    );
                if is_hit {
                    bullets.remove(bullet_index);
                    let color = color::Color::new(0.0, 1.0, 0.0, 1.0);
                    let expl_emitter = particles::Emitter::new(ship_explosion(color));
                    explosions.push(Explosion {
                        time_left: expl_emitter.config.lifetime,
                        emitter: expl_emitter,
                        x: alien.x,
                        y: alien.y,
                        angle: alien.angle,
                        speed: alien.speed,
                    });
                }
                !is_hit
            });
        }
    }
    state.bullet_timer = state.bullet_timer + dt;
    if input::is_key_down(input::KeyCode::S) && state.pause_timer == 0.0 {
        if state.bullet_timer >= BULLET_COOLDOWN {
            state.bullet_timer = 0.0;
            state.bullets.push(Bullet {
                x: state.ship_x + state.ship_angle.cos() * SHIP_DRAW_RADIUS,
                y: state.ship_y + state.ship_angle.sin() * SHIP_DRAW_RADIUS,
                angle: state.ship_angle,
                speed: SHIP_BULLET_SPEED,
                time_left: SHIP_BULLET_TIMER_LIMIT,
                from_enemy: false,
            });
        }
    }
    for asteroid_index in (0..state.asteroids.len()).rev() {
        let asteroid = &mut state.asteroids[asteroid_index];
        asteroid.x =
            (asteroid.x + asteroid.angle.cos() * asteroid.speed * dt).rem_euclid(ARENA_WIDTH);
        asteroid.y =
            (asteroid.y + asteroid.angle.sin() * asteroid.speed * dt).rem_euclid(ARENA_HEIGHT);
        if are_circles_intersecting(
            state.ship_x,
            state.ship_y,
            SHIP_HIT_RADIUS,
            asteroid.x,
            asteroid.y,
            ASTEROID_STAGES[asteroid.stage].radius,
        ) && !state.ship_is_dead
        {
            explode_asteroid(&mut state.asteroids, asteroid_index, &mut state.explosions);
            let color = color::Color::new(0.0, 1.0, 1.0, 1.0);
            let expl_emitter = particles::Emitter::new(ship_explosion(color));
            state.explosions.push(Explosion {
                time_left: expl_emitter.config.lifetime,
                emitter: expl_emitter,
                x: state.ship_x,
                y: state.ship_y,
                angle: state.ship_angle,
                speed: 0.0,
            });
            state.ship_is_dead = true;
            state.pause_timer = 2.0;
            break;
        }
        let asteroid = &state.asteroids[asteroid_index];
        let (asteroid_x, asteroid_y) = (asteroid.x, asteroid.y);
        let asteroid_radius = ASTEROID_STAGES[asteroid.stage].radius;
        let explosions = &mut state.explosions;
        let asteroids = &mut state.asteroids;
        state.aliens.retain(|alien| {
            let is_hit = are_circles_intersecting(
                alien.x,
                alien.y,
                ALIEN_HIT_RADIUS_BY_KIND[alien.kind as usize],
                asteroid_x,
                asteroid_y,
                asteroid_radius,
            );
            if is_hit {
                explode_asteroid(asteroids, asteroid_index, explosions);
                let color = color::Color::new(0.0, 1.0, 0.0, 1.0);
                let expl_emitter = particles::Emitter::new(ship_explosion(color));
                explosions.push(Explosion {
                    time_left: expl_emitter.config.lifetime,
                    emitter: expl_emitter,
                    x: alien.x,
                    y: alien.y,
                    angle: alien.angle,
                    speed: alien.speed * 0.3,
                });
            }
            !is_hit
        });
    }
    state.explosions.retain(|e| e.time_left - dt > 0.0);
    for expl in &mut state.explosions {
        expl.time_left -= dt;
        expl.x = (expl.x + expl.angle.cos() * expl.speed * dt).rem_euclid(ARENA_WIDTH);
        expl.y = (expl.y + expl.angle.sin() * expl.speed * dt).rem_euclid(ARENA_HEIGHT);
    }
    if state.alien_timer == 0.0 && !state.game_over {
        state.alien_timer = ALIEN_SPAWN_PERIOD;
        state.aliens.push(Alien::new());
    }
    if state.game_over && state.pause_timer == 0.0 {
        reset(state);
    }
    if state.asteroids.len() == 0 && state.aliens.len() == 0 && state.pause_timer == 0.0 {
        state.pause_timer = 2.0;
        state.game_over = true;
    }
    if state.ship_is_dead && state.pause_timer == 0.0 {
        reset(state);
    }
}

fn draw_ship(x: f32, y: f32, angle: f32, radius: f32, has_exhaust: bool, color: color::Color) {
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

fn draw(state: &mut GameState) {
    let mut color;
    for explosion in &mut state.explosions {
        explosion.emitter.draw(math::vec2(explosion.x, explosion.y));
    }
    for y in -1..=1 {
        for x in -1..=1 {
            let offset_x = x as f32 * ARENA_WIDTH;
            let offset_y = y as f32 * ARENA_HEIGHT;
            if !state.ship_is_dead {
                color = color::Color::new(0.0, 1.0, 1.0, 1.0);
                draw_ship(
                    state.ship_x + offset_x,
                    state.ship_y + offset_y,
                    state.ship_angle,
                    SHIP_DRAW_RADIUS,
                    state.ship_has_exhaust,
                    color,
                );
            }
            for bullet in &state.bullets {
                color = if bullet.from_enemy {
                    color::Color::new(1.0, 0.0, 1.0, 1.0)
                } else {
                    color::Color::new(1.0, 0.0, 0.0, 1.0)
                };
                shapes::draw_line(
                    bullet.x + offset_x,
                    bullet.y + offset_y,
                    bullet.x + offset_x + bullet.angle.cos() * 10.0,
                    bullet.y + offset_y + bullet.angle.sin() * 10.0,
                    2.0,
                    color,
                );
            }
            for asteroid in &state.asteroids {
                color = color::Color::new(1.0, 1.0, 0.0, 1.0);
                draw_polygon(
                    &asteroid.draw_points,
                    asteroid.x + offset_x,
                    asteroid.y + offset_y,
                    color,
                );
            }
            for alien in &state.aliens {
                let radius = ALIEN_DRAW_RADIUS_BY_KIND[alien.kind as usize];
                color = color::Color::new(0.0, 1.0, 0.0, 1.0);
                shapes::draw_poly(
                    alien.x,
                    alien.y + offset_y,
                    6,
                    radius,
                    (alien.x * 2.0) % 360.0,
                    color,
                );
            }
        }
    }

    // Debug info
    color = color::Color::new(0.5, 0.5, 0.5, 1.0);
    [
        ("ship_angle", state.ship_angle),
        ("ship_x", state.ship_x),
        ("ship_y", state.ship_y),
        ("ship_velocity_x", state.ship_velocity_x),
        ("ship_velocity_y", state.ship_velocity_y),
        ("bullet_0_x", state.bullets.get(0).map_or(0.0, |b| b.x)),
        ("bullet_0_y", state.bullets.get(0).map_or(0.0, |b| b.y)),
        ("expl_0_x", state.explosions.get(0).map_or(0.0, |e| e.x)),
        ("expl_0_y", state.explosions.get(0).map_or(0.0, |e| e.y)),
        ("alien_0_x", state.aliens.get(0).map_or(0.0, |a| a.x)),
        ("alien_0_y", state.aliens.get(0).map_or(0.0, |a| a.y)),
    ]
    .iter()
    .enumerate()
    .for_each(|(i, (name, val))| {
        text::draw_text(
            &format!("{}: {}", name, val),
            0.0,
            16.0 * (i + 1) as f32,
            16.0,
            color,
        )
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

fn explode_asteroid(
    asteroids: &mut Vec<Asteroid>,
    asteroid_index: usize,
    explosions: &mut Vec<Explosion>,
) {
    let asteroid = asteroids.remove(asteroid_index);
    let expl_emitter = particles::Emitter::new(asteroid_explosion());
    explosions.push(Explosion {
        time_left: expl_emitter.config.lifetime,
        emitter: expl_emitter,
        x: asteroid.x,
        y: asteroid.y,
        angle: asteroid.angle,
        speed: asteroid.speed * 1.5,
    });
    if asteroid.stage > 0 {
        asteroids.push(Asteroid::new(asteroid.x, asteroid.y, asteroid.stage - 1));
        asteroids.push(Asteroid::new(asteroid.x, asteroid.y, asteroid.stage - 1));
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
    let mut state = GameState::default();
    load(&mut state);
    loop {
        if let Some(key) = input::get_last_key_pressed() {
            key_pressed(&mut state, key);
        }
        update(&mut state, time::get_frame_time());
        draw(&mut state);
        window::next_frame().await;
    }
}
