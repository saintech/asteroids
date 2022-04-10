use crate::{cfg, entity};
use macroquad::{math, rand};

pub fn update(game: &mut crate::Game, _dt: f32) {
    match game.state {
        entity::GameState::LevelLoading => {
            if game.ship.is_none() {
                game.ship = Some(entity::Ship::new());
            }
            if game.asteroids.is_empty() {
                let ship = game.ship.as_ref().unwrap();
                let start_stage = cfg::ASTEROID_STAGES.len() - 1;
                while game.asteroids.len() < 5 {
                    let rand_pos = math::vec2(
                        rand::gen_range(0.0, cfg::ARENA_WIDTH),
                        rand::gen_range(0.0, cfg::ARENA_HEIGHT),
                    );
                    let delta_pos = rand_pos - ship.position;
                    const RADIUS: f32 = cfg::ARENA_HEIGHT * 0.3;
                    let is_too_close = delta_pos.x.powi(2) + delta_pos.y.powi(2) <= RADIUS.powi(2);
                    if !is_too_close {
                        game.asteroids
                            .push(entity::Asteroid::new(rand_pos, start_stage));
                    }
                }
            }
        }
        entity::GameState::LevelRunning => {
            if let Some(ship) = &mut game.ship {
                let shoot_is_ready = ship.weapon_cooldown_timer == 0.0;
                if game.player_actions.contains(&entity::Action::Shoot) && shoot_is_ready {
                    ship.weapon_cooldown_timer = cfg::BULLET_COOLDOWN;
                    let bullet_offset = math::vec2(
                        ship.sprite.angle.cos() * cfg::SHIP_HIT_RADIUS,
                        ship.sprite.angle.sin() * cfg::SHIP_HIT_RADIUS,
                    );
                    game.bullets.push(entity::Bullet::new(
                        ship.position + bullet_offset,
                        ship.sprite.angle,
                        None,
                    ));
                }
            }
            let time_to_spawn_alien = game.alien_timer == 0.0;
            if time_to_spawn_alien {
                game.alien_timer = cfg::ALIEN_SPAWN_PERIOD;
                game.aliens.push(entity::Alien::new());
            }
        }
        _ => (),
    }
}
