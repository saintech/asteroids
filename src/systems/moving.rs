use crate::{cfg, entity, entity::cmpt};
use macroquad::math;
use std::f32::consts::PI;

pub fn update(game: &mut crate::Game, dt: f32) {
    match game.state {
        cmpt::GameState::Pause => (),
        cmpt::GameState::LevelRunning => {
            if let Some(ship) = &mut game.ship {
                if game.player_actions.contains(&cmpt::Action::TurnRight) {
                    ship.sprite.angle += cfg::SHIP_TURN_SPEED * dt;
                    ship.sprite.angle = ship.sprite.angle.rem_euclid(2.0 * PI);
                }
                if game.player_actions.contains(&cmpt::Action::TurnLeft) {
                    ship.sprite.angle -= cfg::SHIP_TURN_SPEED * dt;
                    ship.sprite.angle = ship.sprite.angle.rem_euclid(2.0 * PI);
                }
                if game.player_actions.contains(&cmpt::Action::Accelerate)
                    && ship.body.speed <= cfg::SHIP_MAX_SPEED
                {
                    ship.has_exhaust = true;
                    let (result_speed, result_angle) = sum_vectors(
                        ship.body.speed,
                        ship.body.angle,
                        cfg::SHIP_ACCEL * dt,
                        ship.sprite.angle,
                    );
                    ship.body.speed = result_speed;
                    ship.body.angle = result_angle;
                } else {
                    ship.has_exhaust = false;
                }
            }
        }
        _ => (),
    }
    match game.state {
        cmpt::GameState::Pause => (),
        _ => {
            if let Some(entity::Ship { position, body, .. }) = &mut game.ship {
                body.speed -= body.speed * cfg::SHIP_DECEL * dt;
                move_position(position, body, dt, true, true);
            }
            for entity::Alien { position, body, .. } in &mut game.aliens {
                move_position(position, body, dt, false, true);
            }
            for entity::Bullet { position, body, .. } in &mut game.bullets {
                move_position(position, body, dt, true, true);
            }
            for entity::Asteroid { position, body, .. } in &mut game.asteroids {
                move_position(position, body, dt, true, true);
            }
            for entity::Explosion { position, body, .. } in &mut game.explosions {
                move_position(position, body, dt, true, true);
            }
        }
    }
}

fn move_position(
    position: &mut math::Vec2,
    body: &cmpt::Body,
    dt: f32,
    wrap_x: bool,
    wrap_y: bool,
) {
    *position += math::vec2(body.angle.cos() * body.speed * dt, body.angle.sin() * body.speed * dt);
    if wrap_x {
        position.x = position.x.rem_euclid(cfg::ARENA_WIDTH);
    }
    if wrap_y {
        position.y = position.y.rem_euclid(cfg::ARENA_HEIGHT);
    }
}

fn sum_vectors(v1_magnitude: f32, v1_angle: f32, v2_magnitude: f32, v2_angle: f32) -> (f32, f32) {
    let d_rotation = v2_angle - v1_angle;
    let d_angle = 2.0 * (d_rotation % PI) - d_rotation;
    // https://www.mathstopia.net/vectors/parallelogram-law-vector-addition
    // = √(A² + B² + 2ABcosα)
    let result_magnitude = f32::sqrt(
        v1_magnitude.powi(2)
            + v2_magnitude.powi(2)
            + 2.0 * v1_magnitude * v2_magnitude * d_angle.cos(),
    );
    // find angle with three sides (the Law of Cosines)
    // https://en.wikipedia.org/wiki/Solution_of_triangles#Three_sides_given_(SSS)
    // = arccos((B² + C² - A²)/(2BC))
    let mut d_angle_r = f32::acos(
        (v1_magnitude.powi(2) + result_magnitude.powi(2) - v2_magnitude.powi(2))
            / (2.0 * v1_magnitude * result_magnitude),
    ) * d_angle.signum();
    if d_angle_r.is_nan() {
        d_angle_r = 0.0;
    }
    let result_angle = (v1_angle + d_angle_r).rem_euclid(2.0 * PI);
    (result_magnitude, result_angle)
}
