use std::f32;

use macroquad::{color, input, miniquad::date, rand, shapes, time, window};

const ARENA_WIDTH: f32 = 800.0;
const ARENA_HEIGHT: f32 = 600.0;
const SHIP_RADIUS: f32 = 30.0;
const BULLET_TIMER_LIMIT: f32 = 0.5;
const BULLET_RADIUS: f32 = 5.0;
const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage {
        speed: 120.0,
        radius: 15.0,
    },
    AsteroidStage {
        speed: 70.0,
        radius: 30.0,
    },
    AsteroidStage {
        speed: 50.0,
        radius: 50.0,
    },
    AsteroidStage {
        speed: 20.0,
        radius: 80.0,
    },
];

struct Bullet {
    x: f32,
    y: f32,
    angle: f32,
    time_left: f32,
}

struct AsteroidStage {
    speed: f32,
    radius: f32,
}

#[derive(Default)]
struct Asteroid {
    x: f32,
    y: f32,
    angle: f32,
    stage: usize,
}

#[derive(Default)]
struct GameState {
    ship_x: f32,
    ship_y: f32,
    ship_angle: f32,
    ship_speed_x: f32,
    ship_speed_y: f32,
    bullets: Vec<Bullet>,
    bullet_timer: f32,
    asteroids: Vec<Asteroid>,
}

fn reset(state: &mut GameState) {
    state.ship_x = ARENA_WIDTH / 2.0;
    state.ship_y = ARENA_HEIGHT / 2.0;
    state.ship_angle = 0.0;
    state.ship_speed_x = 0.0;
    state.ship_speed_y = 0.0;
    state.bullets = Default::default();
    state.bullet_timer = BULLET_TIMER_LIMIT;
    state.asteroids = vec![
        Asteroid {
            x: 100.0,
            y: 100.0,
            ..Default::default()
        },
        Asteroid {
            x: ARENA_WIDTH - 100.0,
            y: 100.0,
            ..Default::default()
        },
        Asteroid {
            x: ARENA_WIDTH / 2.0,
            y: ARENA_HEIGHT - 100.0,
            ..Default::default()
        },
    ];
    for asteroid in &mut state.asteroids {
        asteroid.angle = rand::gen_range(0.0, 1.0) * (2.0 * f32::consts::PI);
        asteroid.stage = ASTEROID_STAGES.len() - 1;
    }
}

fn load(state: &mut GameState) {
    reset(state);
}

fn update(state: &mut GameState, dt: f32) {
    let turn_speed = 10.0;
    if input::is_key_down(input::KeyCode::Right) {
        state.ship_angle = state.ship_angle + turn_speed * dt;
    }
    if input::is_key_down(input::KeyCode::Left) {
        state.ship_angle = state.ship_angle - turn_speed * dt;
    }
    state.ship_angle = state.ship_angle.rem_euclid(2.0 * f32::consts::PI);
    if input::is_key_down(input::KeyCode::Up) {
        let ship_speed = 100.0;
        state.ship_speed_x = state.ship_speed_x + state.ship_angle.cos() * ship_speed * dt;
        state.ship_speed_y = state.ship_speed_y + state.ship_angle.sin() * ship_speed * dt;
    }
    state.ship_x = (state.ship_x + state.ship_speed_x * dt).rem_euclid(ARENA_WIDTH);
    state.ship_y = (state.ship_y + state.ship_speed_y * dt).rem_euclid(ARENA_HEIGHT);
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
            let bullet_speed = 500.0;
            bullet.x = (bullet.x + bullet.angle.cos() * bullet_speed * dt).rem_euclid(ARENA_WIDTH);
            bullet.y = (bullet.y + bullet.angle.sin() * bullet_speed * dt).rem_euclid(ARENA_HEIGHT);
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
                        let angle_1 = rand::gen_range(0.0, 1.0) * (2.0 * f32::consts::PI);
                        let angle_2 = (angle_1 - f32::consts::PI).rem_euclid(2.0 * f32::consts::PI);
                        state.asteroids.push(Asteroid {
                            x: asteroid_x,
                            y: asteroid_y,
                            angle: angle_1,
                            stage: asteroid_stage - 1,
                        });
                        state.asteroids.push(Asteroid {
                            x: asteroid_x,
                            y: asteroid_y,
                            angle: angle_2,
                            stage: asteroid_stage - 1,
                        });
                    }
                    state.asteroids.remove(asteroid_index);
                    break;
                }
            }
        }
    }
    state.bullet_timer = state.bullet_timer + dt;
    if input::is_key_down(input::KeyCode::S) {
        if state.bullet_timer >= BULLET_TIMER_LIMIT {
            state.bullet_timer = 0.0;
            state.bullets.push(Bullet {
                x: state.ship_x + state.ship_angle.cos() * SHIP_RADIUS,
                y: state.ship_y + state.ship_angle.sin() * SHIP_RADIUS,
                angle: state.ship_angle,
                time_left: 4.0,
            })
        }
    }
    for asteroid in &mut state.asteroids {
        asteroid.x = (asteroid.x
            + asteroid.angle.cos() * ASTEROID_STAGES[asteroid.stage].speed * dt)
            .rem_euclid(ARENA_WIDTH);
        asteroid.y = (asteroid.y
            + asteroid.angle.sin() * ASTEROID_STAGES[asteroid.stage].speed * dt)
            .rem_euclid(ARENA_HEIGHT);
        if are_circles_intersecting(
            state.ship_x,
            state.ship_y,
            SHIP_RADIUS,
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

fn draw(state: &mut GameState) {
    let mut color;
    for y in -1..=1 {
        for x in -1..=1 {
            let offset_x = x as f32 * ARENA_WIDTH;
            let offset_y = y as f32 * ARENA_HEIGHT;
            color = color::Color::new(0.0, 0.0, 1.0, 1.0);
            shapes::draw_circle(
                state.ship_x + offset_x,
                state.ship_y + offset_y,
                SHIP_RADIUS,
                color,
            );
            let ship_circle_distance = 20.0;
            color = color::Color::new(0.0, 1.0, 1.0, 1.0);
            shapes::draw_circle(
                state.ship_x + offset_x + state.ship_angle.cos() * ship_circle_distance,
                state.ship_y + offset_y + state.ship_angle.sin() * ship_circle_distance,
                5.0,
                color,
            );
            for bullet in &state.bullets {
                color = color::Color::new(0.0, 1.0, 0.0, 1.0);
                shapes::draw_circle(
                    bullet.x + offset_x,
                    bullet.y + offset_y,
                    BULLET_RADIUS,
                    color,
                );
            }
            for asteroid in &state.asteroids {
                color = color::Color::new(1.0, 1.0, 0.0, 1.0);
                shapes::draw_circle(
                    asteroid.x + offset_x,
                    asteroid.y + offset_y,
                    ASTEROID_STAGES[asteroid.stage].radius,
                    color,
                );
            }
        }
    }
}

#[macroquad::main("asteroids")]
async fn main() {
    rand::srand(date::now() as u64);
    let mut state = GameState::default();
    load(&mut state);
    loop {
        update(&mut state, time::get_frame_time());
        draw(&mut state);
        window::next_frame().await;
    }
}
