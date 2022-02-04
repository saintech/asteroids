use std::f32::consts::PI;

use macroquad::{color, input, math, miniquad::date, rand, shapes, text, time, window};

const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const SHIP_MAX_SPEED: f32 = 400.0;
const SHIP_ACCEL: f32 = 500.0;
const SHIP_DECEL: f32 = 0.2;
const SHIP_DRAW_RADIUS: f32 = 16.0;
const SHIP_HIT_RADIUS: f32 = 10.0;
const SHIP_TURN_SPEED: f32 = 6.0;
const BULLET_SPEED: f32 = 600.0;
const BULLET_COOLDOWN: f32 = 0.3;
const BULLET_TIMER_LIMIT: f32 = 0.8;
const BULLET_RADIUS: f32 = 3.0;
const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage {
        max_speed: 180.0,
        radius: 12.0,
    },
    AsteroidStage {
        max_speed: 120.0,
        radius: 33.0,
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
    time_left: f32,
}

struct AsteroidStage {
    max_speed: f32,
    radius: f32,
}

#[derive(Default)]
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
    ship_x: f32,
    ship_y: f32,
    ship_angle: f32,
    ship_velocity_x: f32,
    ship_velocity_y: f32,
    ship_has_exhaust: bool,
    bullets: Vec<Bullet>,
    bullet_timer: f32,
    asteroids: Vec<Asteroid>,
}

fn reset(state: &mut GameState) {
    state.ship_x = ARENA_WIDTH / 2.0;
    state.ship_y = ARENA_HEIGHT / 2.0;
    state.ship_angle = 0.0;
    state.ship_velocity_x = 0.0;
    state.ship_velocity_y = 0.0;
    state.bullets = Default::default();
    state.bullet_timer = BULLET_COOLDOWN;
    let start_stage = ASTEROID_STAGES.len() - 1;
    state.asteroids = vec![];
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
    reset(state);
}

fn key_pressed(_state: &mut GameState, _key: input::KeyCode) {
    // template
}

fn update(state: &mut GameState, dt: f32) {
    if input::is_key_down(input::KeyCode::Right) {
        state.ship_angle = state.ship_angle + SHIP_TURN_SPEED * dt;
    }
    if input::is_key_down(input::KeyCode::Left) {
        state.ship_angle = state.ship_angle - SHIP_TURN_SPEED * dt;
    }
    state.ship_angle = state.ship_angle.rem_euclid(2.0 * PI);
    state.ship_velocity_x = state.ship_velocity_x - (state.ship_velocity_x * SHIP_DECEL * dt);
    state.ship_velocity_y = state.ship_velocity_y - (state.ship_velocity_y * SHIP_DECEL * dt);
    if input::is_key_down(input::KeyCode::Up) {
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
    for bullet_index in (0..state.bullets.len()).rev() {
        let bullet = &mut state.bullets[bullet_index];
        bullet.time_left = bullet.time_left - dt;
        if bullet.time_left <= 0.0 {
            state.bullets.remove(bullet_index);
        } else {
            bullet.x = (bullet.x + bullet.angle.cos() * BULLET_SPEED * dt).rem_euclid(ARENA_WIDTH);
            bullet.y = (bullet.y + bullet.angle.sin() * BULLET_SPEED * dt).rem_euclid(ARENA_HEIGHT);
            for asteroid_index in (0..state.asteroids.len()).rev() {
                let bullet = &state.bullets[bullet_index];
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
                    if asteroid_stage > 0 {
                        let asteroid_1 = Asteroid::new(asteroid_x, asteroid_y, asteroid_stage - 1);
                        let asteroid_2 = Asteroid::new(asteroid_x, asteroid_y, asteroid_stage - 1);
                        state.asteroids.push(asteroid_1);
                        state.asteroids.push(asteroid_2);
                    }
                    state.asteroids.remove(asteroid_index);
                    break;
                }
            }
        }
    }
    state.bullet_timer = state.bullet_timer + dt;
    if input::is_key_down(input::KeyCode::S) {
        if state.bullet_timer >= BULLET_COOLDOWN {
            state.bullet_timer = 0.0;
            state.bullets.push(Bullet {
                x: state.ship_x + state.ship_angle.cos() * SHIP_HIT_RADIUS,
                y: state.ship_y + state.ship_angle.sin() * SHIP_HIT_RADIUS,
                angle: state.ship_angle,
                time_left: BULLET_TIMER_LIMIT,
            })
        }
    }
    for asteroid in &mut state.asteroids {
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
        ) {
            reset(state);
            break;
        }
    }
    if state.asteroids.len() == 0 {
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
            math::Vec2::new(
                x + (angle + PI * 0.85).cos() * radius * 0.55,
                y + (angle + PI * 0.85).sin() * radius * 0.55,
            ),
            math::Vec2::new(
                x + (angle + PI * rand::gen_range(0.98, 1.02)).cos()
                    * radius
                    * rand::gen_range(0.75, 1.25),
                y + (angle + PI * rand::gen_range(0.98, 1.02)).sin()
                    * radius
                    * rand::gen_range(0.75, 1.25),
            ),
            math::Vec2::new(
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
    for y in -1..=1 {
        for x in -1..=1 {
            let offset_x = x as f32 * ARENA_WIDTH;
            let offset_y = y as f32 * ARENA_HEIGHT;
            color = color::Color::new(0.0, 1.0, 1.0, 1.0);
            draw_ship(
                state.ship_x + offset_x,
                state.ship_y + offset_y,
                state.ship_angle,
                SHIP_DRAW_RADIUS,
                state.ship_has_exhaust,
                color,
            );
            for bullet in &state.bullets {
                color = color::Color::new(0.0, 1.0, 0.0, 1.0);
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
