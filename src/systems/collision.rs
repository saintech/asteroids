use crate::entity::cmpt;
use macroquad::math;

pub fn update(game: &mut crate::Game, _dt: f32) {
    let (mut enemy_bullets, mut ship_bullets): (Vec<_>, _) =
        game.bullets.iter_mut().partition(|b| b.from_enemy);
    for alien in &mut game.aliens {
        for ship in &mut game.ship {
            do_collision(alien.position, &mut alien.body, ship.position, &mut ship.body);
        }
    }
    for enemy_bullet in &mut enemy_bullets {
        for ship in &mut game.ship {
            do_collision(
                enemy_bullet.position,
                &mut enemy_bullet.body,
                ship.position,
                &mut ship.body,
            );
        }
        for asteroid in &mut game.asteroids {
            do_collision(
                enemy_bullet.position,
                &mut enemy_bullet.body,
                asteroid.position,
                &mut asteroid.body,
            );
        }
    }
    for ship_bullet in &mut ship_bullets {
        for enemy_bullet in &mut enemy_bullets {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                enemy_bullet.position,
                &mut enemy_bullet.body,
            );
        }
        for alien in &mut game.aliens {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                alien.position,
                &mut alien.body,
            );
        }
        for asteroid in &mut game.asteroids {
            do_collision(
                ship_bullet.position,
                &mut ship_bullet.body,
                asteroid.position,
                &mut asteroid.body,
            );
        }
    }
    for asteroid in &mut game.asteroids {
        for ship in &mut game.ship {
            do_collision(asteroid.position, &mut asteroid.body, ship.position, &mut ship.body);
        }
        for alien in &mut game.aliens {
            do_collision(asteroid.position, &mut asteroid.body, alien.position, &mut alien.body);
        }
    }
}

fn do_collision(
    a_pos: math::Vec2,
    a_body: &mut cmpt::Body,
    b_pos: math::Vec2,
    b_body: &mut cmpt::Body,
) {
    let d_pos = a_pos - b_pos;
    let is_intersecting =
        d_pos.x.powi(2) + d_pos.y.powi(2) <= (a_body.radius + b_body.radius).powi(2);
    if is_intersecting {
        a_body.is_hit = true;
        b_body.is_hit = true;
    }
}
