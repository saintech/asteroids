use crate::entity::cmpt;

pub fn update(game: &mut crate::Game, dt: f32) {
    match game.state {
        cmpt::GameState::Pause => (),
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
