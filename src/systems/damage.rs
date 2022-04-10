use crate::{cfg, entity, entity::cmpt, palette};
use macroquad::{color, math};
use macroquad_particles as particles;
use std::f32::consts::PI;

pub fn update(game: &mut crate::Game, _dt: f32) {
    if let Some(ship) = game.ship.as_mut().filter(|sh| sh.body.is_hit) {
        ship.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(ship_explosion(cfg::SHIP_EXPLOSION_COLOR));
        game.explosions.push(entity::Explosion {
            position: ship.position,
            body: Default::default(),
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    for alien in game.aliens.iter_mut().filter(|a| a.body.is_hit) {
        alien.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(ship_explosion(cfg::ALIEN_EXPLOSION_COLOR));
        game.explosions.push(entity::Explosion {
            position: alien.position,
            body: cmpt::Body {
                angle: alien.body.angle,
                speed: alien.body.speed,
                ..Default::default()
            },
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
    }
    game.bullets.retain(|b| !b.body.is_hit);
    let mut new_asteroids: Vec<entity::Asteroid> = Default::default();
    for asteroid in game.asteroids.iter_mut().filter(|a| a.body.is_hit) {
        asteroid.is_destroyed = true;
        let expl_emitter = particles::Emitter::new(asteroid_explosion());
        game.explosions.push(entity::Explosion {
            position: asteroid.position,
            body: cmpt::Body {
                angle: asteroid.body.angle,
                speed: asteroid.body.speed * 1.5,
                ..Default::default()
            },
            life_timer: expl_emitter.config.lifetime,
            emitter: expl_emitter,
        });
        if asteroid.stage > 0 {
            new_asteroids.push(entity::Asteroid::new(asteroid.position, asteroid.stage - 1));
            new_asteroids.push(entity::Asteroid::new(asteroid.position, asteroid.stage - 1));
        }
    }
    game.asteroids.append(&mut new_asteroids);
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
        size: 1.7,
        size_curve: Some(particles::Curve {
            points: vec![(0.0, 1.0), (0.85, 1.0), (1.0, 0.0)],
            ..Default::default()
        }),
        shape: particles::ParticleShape::Circle { subdivisions: 7 },
        colors_curve: particles::ColorCurve {
            start: palette::LIGHTGRAY,
            mid: palette::LIGHTGRAY,
            end: palette::LIGHTGRAY,
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
        size_curve: Some(particles::Curve {
            points: vec![(0.0, 1.0), (0.85, 1.0), (1.0, 0.0)],
            ..Default::default()
        }),
        shape: particles::ParticleShape::Circle { subdivisions: 4 },
        colors_curve: particles::ColorCurve {
            start: color,
            mid: color,
            end: color,
        },
        ..Default::default()
    }
}
