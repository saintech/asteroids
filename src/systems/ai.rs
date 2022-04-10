use crate::{cfg, entity};
use macroquad::rand;
use std::f32::consts::PI;

pub fn update(game: &mut crate::Game, _dt: f32) {
    for alien in &mut game.aliens {
        let time_to_shift = alien.shift_timer == 0.0;
        let time_to_shoot = alien.weapon_cooldown_timer == 0.0;
        if time_to_shift {
            alien.shift_timer = cfg::ALIEN_SHIFT_PERIOD;
            let d_angle = PI / 4.0;
            let origin_angle = PI * alien.direction as u32 as f32;
            alien.body.angle += d_angle * rand::gen_range(-2_i32, 2) as f32;
            alien.body.angle = alien
                .body
                .angle
                .clamp(origin_angle - d_angle, origin_angle + d_angle);
        }
        if time_to_shoot && game.ship.is_some() {
            alien.weapon_cooldown_timer = cfg::ALIEN_SHOOT_PERIOD;
            let ship = game.ship.as_ref().unwrap();
            let shoot_angle =
                f32::atan2(ship.position.y - alien.position.y, ship.position.x - alien.position.x);
            game.bullets
                .push(entity::Bullet::new(alien.position, shoot_angle, Some(alien.kind)));
        }
        if alien.position.x < -4.0 || cfg::ARENA_WIDTH + 4.0 < alien.position.x {
            alien.is_destroyed = true;
        }
    }
}
