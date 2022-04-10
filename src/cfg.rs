use crate::{entity::cmpt::AsteroidStage, entity::Action, palette};
use macroquad::{color, input};

pub const ARENA_WIDTH: f32 = 432.0;
pub const ARENA_HEIGHT: f32 = 240.0; // 600 * 0.4
pub const SHIP_MAX_SPEED: f32 = 160.0;
pub const SHIP_ACCEL: f32 = 200.0;
pub const SHIP_DECEL: f32 = 0.08;
pub const SHIP_DRAW_RADIUS: f32 = 7.0;
pub const SHIP_HIT_RADIUS: f32 = 4.0;
pub const SHIP_TURN_SPEED: f32 = 6.0;
pub const SHIP_BULLET_COLOR: color::Color = palette::RED;
pub const SHIP_BULLET_TIMER_LIMIT: f32 = 0.8;
pub const SHIP_BULLET_SPEED: f32 = 240.0;
pub const SHIP_EXPLOSION_COLOR: color::Color = palette::BLUE;
pub const BULLET_COOLDOWN: f32 = 0.3;
pub const BULLET_RADIUS: f32 = 1.2;
pub const ALIEN_DRAW_RADIUS_BY_KIND: &[f32] = &[9.0, 6.9];
pub const ALIEN_HIT_RADIUS_BY_KIND: &[f32] = &[5.5, 4.4];
pub const ALIEN_BULLET_TIMER_LIMIT_BY_KIND: &[f32] = &[0.9, 1.3];
pub const ALIEN_BULLET_SPEED: f32 = 100.0;
pub const ALIEN_BULLET_COLOR: color::Color = palette::PINK;
pub const ALIEN_SPAWN_PERIOD: f32 = 30.0;
pub const ALIEN_SHOOT_PERIOD: f32 = 1.3;
pub const ALIEN_SHIFT_PERIOD: f32 = 1.0;
pub const ALIEN_EXPLOSION_COLOR: color::Color = palette::GREEN;
pub const ASTEROID_STAGES: &[AsteroidStage] = &[
    AsteroidStage { max_speed: 72.0, radius: 4.8 },
    AsteroidStage { max_speed: 48.0, radius: 11.2 },
    AsteroidStage { max_speed: 24.0, radius: 16.0 },
];
pub const KEYMAP: &[(input::KeyCode, Action)] = &[
    (input::KeyCode::Up, Action::Accelerate),
    (input::KeyCode::Left, Action::TurnLeft),
    (input::KeyCode::Right, Action::TurnRight),
    (input::KeyCode::S, Action::Shoot),
    (input::KeyCode::Escape, Action::TogglePause),
];
