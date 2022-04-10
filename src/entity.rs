use crate::{cfg, palette, sprites};
use cmpt::*;
use macroquad::{color, material, math, rand, texture};
use macroquad_particles as particles;
use std::f32::consts::PI;

/// Components for game entities.
pub mod cmpt {
    use macroquad::{color, math};

    pub struct AsteroidStage {
        pub max_speed: f32,
        pub radius: f32,
    }

    #[derive(PartialEq)]
    pub enum SpriteVariant {
        Bullet {
            color: color::Color,
        },
        Vector {
            layers: Vec<(Vec<math::Vec2>, color::Color)>,
        },
    }

    pub struct Sprite {
        pub variant: SpriteVariant,
        pub angle: f32,
    }
    #[derive(Default)]
    pub struct Body {
        pub radius: f32,
        pub angle: f32,
        pub speed: f32,
        pub is_hit: bool,
    }

    #[derive(Copy, Clone)]
    pub enum AlienDirection {
        ToRight,
        ToLeft,
    }

    #[derive(Copy, Clone)]
    pub enum AlienKind {
        Big,
        Small,
    }
}

pub struct Bullet {
    pub position: math::Vec2,
    pub sprite: Sprite,
    pub body: Body,
    pub life_timer: f32,
    pub from_enemy: bool,
}

impl Bullet {
    pub fn new(position: math::Vec2, angle: f32, alien_kind: Option<AlienKind>) -> Self {
        let (color, speed, life_timer) = if let Some(kind) = alien_kind {
            (
                cfg::ALIEN_BULLET_COLOR,
                cfg::ALIEN_BULLET_SPEED,
                cfg::ALIEN_BULLET_TIMER_LIMIT_BY_KIND[kind as usize],
            )
        } else {
            (
                cfg::SHIP_BULLET_COLOR,
                cfg::SHIP_BULLET_SPEED,
                cfg::SHIP_BULLET_TIMER_LIMIT,
            )
        };
        Bullet {
            position,
            sprite: Sprite {
                variant: SpriteVariant::Bullet { color },
                angle,
            },
            body: Body {
                radius: cfg::BULLET_RADIUS,
                angle,
                speed,
                is_hit: false,
            },
            life_timer,
            from_enemy: alien_kind.is_some(),
        }
    }
}

pub struct Ship {
    pub position: math::Vec2,
    pub sprite: Sprite,
    pub body: Body,
    pub has_exhaust: bool,
    pub is_destroyed: bool,
    pub weapon_cooldown_timer: f32,
}

impl Ship {
    pub fn new() -> Self {
        Ship {
            position: math::vec2(cfg::ARENA_WIDTH / 2.0, cfg::ARENA_HEIGHT / 2.0),
            sprite: Sprite {
                variant: SpriteVariant::Vector {
                    layers: create_layers(sprites::SHIP, cfg::SHIP_DRAW_RADIUS),
                },
                angle: 0.0,
            },
            body: Body {
                radius: cfg::SHIP_HIT_RADIUS,
                angle: 0.0,
                speed: 0.0,
                is_hit: false,
            },
            has_exhaust: false,
            is_destroyed: false,
            weapon_cooldown_timer: 0.0,
        }
    }
}

pub struct Explosion {
    pub position: math::Vec2,
    pub body: Body,
    pub emitter: particles::Emitter,
    pub life_timer: f32,
}

pub struct Alien {
    pub position: math::Vec2,
    pub sprite: Sprite,
    pub body: Body,
    pub is_destroyed: bool,
    pub kind: AlienKind,
    pub direction: AlienDirection,
    pub weapon_cooldown_timer: f32,
    pub shift_timer: f32,
}

impl Alien {
    pub fn new() -> Self {
        let y = cfg::ARENA_HEIGHT * rand::gen_range(0.15, 0.85);
        let direction = [AlienDirection::ToRight, AlienDirection::ToLeft][rand::gen_range(0, 2)];
        let x = cfg::ARENA_WIDTH * direction as u32 as f32;
        let angle = PI * direction as u32 as f32;
        let kind = if rand::gen_range(0_u32, 10) < 3 {
            AlienKind::Small
        } else {
            AlienKind::Big
        };
        Alien {
            position: math::vec2(x, y),
            sprite: Sprite {
                variant: SpriteVariant::Vector {
                    layers: create_layers(
                        sprites::ALIEN,
                        cfg::ALIEN_DRAW_RADIUS_BY_KIND[kind as usize],
                    ),
                },
                angle: 0.0,
            },
            body: Body {
                radius: cfg::ALIEN_HIT_RADIUS_BY_KIND[kind as usize],
                angle,
                speed: rand::gen_range(32.0, 40.0),
                is_hit: false,
            },
            is_destroyed: false,
            kind,
            direction,
            weapon_cooldown_timer: cfg::ALIEN_SHOOT_PERIOD,
            shift_timer: 0.0,
        }
    }
}

pub struct Asteroid {
    pub position: math::Vec2,
    pub sprite: Sprite,
    pub body: Body,
    pub is_destroyed: bool,
    pub stage: usize,
}

impl Asteroid {
    pub fn new(position: math::Vec2, stage: usize) -> Self {
        let radius = cfg::ASTEROID_STAGES[stage].radius;
        let max_speed = cfg::ASTEROID_STAGES[stage].max_speed;
        let mut layers = Vec::new();
        layers.push({
            let mut draw_points = Vec::new();
            let mut draw_angle: f32 = 0.0;
            while draw_angle < PI * 2.0 {
                let distance = rand::gen_range(0.95, 1.1) * radius;
                draw_points
                    .push(math::vec2(draw_angle.cos() * distance, draw_angle.sin() * distance));
                draw_angle += rand::gen_range(0.6, 1.2);
            }
            (draw_points, palette::DARKPURPLE)
        });
        layers.push({
            let mut draw_points = Vec::new();
            let mut draw_angle: f32 = 0.0;
            while draw_angle < PI * 2.0 {
                let distance = rand::gen_range(0.5, 0.85) * radius;
                draw_points
                    .push(math::vec2(draw_angle.cos() * distance, draw_angle.sin() * distance));
                draw_angle += rand::gen_range(0.5, 0.7);
            }
            (draw_points, palette::BEIGE)
        });
        let angle = rand::gen_range(0.0, 2.0 * PI);
        let speed = max_speed * rand::gen_range(0.5, 1.0);
        Asteroid {
            position,
            sprite: Sprite {
                variant: SpriteVariant::Vector { layers },
                angle: 0.0,
            },
            body: Body { radius, angle, speed, is_hit: false },
            is_destroyed: false,
            stage,
        }
    }
}

pub struct StarBackground {
    pub static_emitter: particles::Emitter,
    pub side_emitter: particles::Emitter,
    pub side_emitter_pos: math::Vec2,
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
                    height: cfg::ARENA_HEIGHT * 1.2,
                };
                side_emitter_pos =
                    math::vec2(cfg::ARENA_WIDTH * multiplier as f32, cfg::ARENA_HEIGHT / 2.0);
            }
            2 => {
                side_cfg.initial_direction = math::vec2(rand::gen_range(-0.3, 0.3), 1.0);
                side_cfg.emission_shape = particles::EmissionShape::Rect {
                    width: cfg::ARENA_WIDTH * 1.2,
                    height: 0.0,
                };
                side_emitter_pos = math::vec2(cfg::ARENA_WIDTH / 2.0, 0.0);
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

pub struct Renderer {
    pub canvas: macroquad_canvas::Canvas2D,
    pub crt_effect: Option<material::Material>,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut canvas = macroquad_canvas::Canvas2D::new(cfg::ARENA_WIDTH, cfg::ARENA_HEIGHT);
        canvas
            .get_texture_mut()
            .set_filter(texture::FilterMode::Nearest);
        Renderer { canvas, crt_effect: None }
    }
}

fn create_layers(
    layers_ref: &[(&[math::Vec2], color::Color)],
    size: f32,
) -> Vec<(Vec<math::Vec2>, color::Color)> {
    layers_ref
        .iter()
        .map(|(points, color)| (points.iter().map(|&vec| vec * size).collect(), *color))
        .collect()
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
            width: cfg::ARENA_WIDTH,
            height: cfg::ARENA_HEIGHT,
        },
        shape: particles::ParticleShape::Circle { subdivisions: 4 },
        colors_curve: particles::ColorCurve {
            start: palette::WHITE,
            mid: palette::WHITE,
            end: palette::WHITE,
        },
        ..Default::default()
    }
}

#[derive(Debug)]
pub enum GameState {
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
pub enum Action {
    Accelerate,
    TurnLeft,
    TurnRight,
    Shoot,
    TogglePause,
}
