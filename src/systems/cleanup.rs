pub fn update(game: &mut crate::Game, _dt: f32) {
    game.ship = game.ship.take().filter(|sh| !sh.is_destroyed);
    game.aliens.retain(|a| !a.is_destroyed);
    game.bullets.retain(|b| b.life_timer > 0.0);
    game.asteroids.retain(|a| !a.is_destroyed);
    game.explosions.retain(|e| e.life_timer > 0.0);
}
