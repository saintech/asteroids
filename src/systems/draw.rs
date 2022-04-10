use crate::{cfg, entity, entity::cmpt, palette};
use macroquad::{camera, color, material, math, rand, shapes, text, texture, time, window};
use std::f32::consts::PI;

pub fn update(game: &mut crate::Game, _dt: f32) {
    let renderer = &mut game.renderer;
    camera::set_camera(&renderer.canvas.camera);
    window::clear_background(palette::BLACK);
    game.star_bg
        .static_emitter
        .draw(math::vec2(cfg::ARENA_WIDTH / 2.0, cfg::ARENA_HEIGHT / 2.0));
    game.star_bg
        .side_emitter
        .draw(game.star_bg.side_emitter_pos);
    for explosion in &mut game.explosions {
        explosion.emitter.draw(explosion.position);
    }
    for y in -1..=1 {
        for x in -1..=1 {
            let offset = math::vec2(x as f32 * cfg::ARENA_WIDTH, y as f32 * cfg::ARENA_HEIGHT);
            for entity::Ship { position, sprite, has_exhaust, .. } in &game.ship {
                let position = *position + offset;
                draw_ship(position, sprite, *has_exhaust);
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + body.angle.cos() * body.speed * 0.2,
                //     position.y + body.angle.sin() * body.speed * 0.2,
                //     3.0,
                //     color::GREEN,
                // );
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + sprite.angle.cos() * cfg::SHIP_ACCEL * 0.2,
                //     position.y + sprite.angle.sin() * cfg::SHIP_ACCEL * 0.2,
                //     2.0,
                //     color::BLUE,
                // );
                // let (result_speed, result_angle) =
                //     sum_vectors(body.speed, body.angle, cfg::SHIP_ACCEL, sprite.angle);
                // shapes::draw_line(
                //     position.x,
                //     position.y,
                //     position.x + result_angle.cos() * result_speed * 0.2,
                //     position.y + result_angle.sin() * result_speed * 0.2,
                //     1.0,
                //     color::RED,
                // );
            }
            for entity::Asteroid { position, sprite, .. } in &game.asteroids {
                let layers = match &sprite.variant {
                    cmpt::SpriteVariant::Vector { layers } => layers,
                    _ => unreachable!(),
                };
                let mut position = *position + offset;
                position.x = position.x as i32 as f32;
                position.y = position.y as i32 as f32;
                draw_layers(layers, position, 0.0);
            }
            for entity::Alien { position, sprite, .. } in &game.aliens {
                let angle_by_x = f32::min((position.x * 8.0) % 180.0, 90.0);
                let layers = match &sprite.variant {
                    cmpt::SpriteVariant::Vector { layers } => layers,
                    _ => unreachable!(),
                };
                let position = math::vec2(
                    position.x as i32 as f32 + 0.5,
                    (position.y + offset.y) as i32 as f32 + 0.5,
                );
                draw_layers(&layers[..2], position, angle_by_x.to_radians());
                draw_layers(&layers[2..], position, 0.0);
            }
            for entity::Bullet { position, sprite, .. } in &game.bullets {
                let position = *position + offset;
                let color = match sprite.variant {
                    cmpt::SpriteVariant::Bullet { color } => color,
                    _ => unreachable!(),
                };
                shapes::draw_line(
                    position.x,
                    position.y,
                    position.x + sprite.angle.cos() * 4.0,
                    position.y + sprite.angle.sin() * 4.0,
                    2.0,
                    color,
                );
            }
        }
    }
    camera::set_default_camera();
    window::clear_background(palette::BLACK);
    material::gl_use_material(renderer.crt_effect.unwrap());
    let window_size = math::vec2(window::screen_width(), window::screen_height());
    let size_multiplier = f32::min(
        (window_size.x / cfg::ARENA_WIDTH).trunc(),
        (window_size.y / cfg::ARENA_HEIGHT).trunc(),
    );
    let dest_size =
        math::vec2(cfg::ARENA_WIDTH * size_multiplier, cfg::ARENA_HEIGHT * size_multiplier);
    let d_size = window_size - dest_size;
    texture::draw_texture_ex(
        *renderer.canvas.get_texture(),
        (d_size.x as i32 / 2) as f32,
        (d_size.y as i32 / 2) as f32,
        palette::WHITE,
        texture::DrawTextureParams {
            dest_size: Some(dest_size),
            ..Default::default()
        },
    );
    // game.renderer.canvas.draw();
    material::gl_use_default_material();
    if game
        .player_actions
        .contains(&entity::Action::ToggleDebugInfo)
    {
        renderer.show_debug_info = !renderer.show_debug_info;
    }
    if renderer.show_debug_info {
        let color = palette::DARKGRAY;
        let ship = game.ship.as_ref();
        [
            ("fps", time::get_fps() as f32),
            ("ship.sprite.angle", ship.map_or(0.0, |sh| sh.sprite.angle)),
            ("ship.position.x", ship.map_or(0.0, |sh| sh.position.x)),
            ("ship.position.y", ship.map_or(0.0, |sh| sh.position.y)),
            ("ship.body.angle", ship.map_or(0.0, |sh| sh.body.angle)),
            ("ship.body.speed", ship.map_or(0.0, |sh| sh.body.speed)),
            ("bullet_0.position.x", game.bullets.get(0).map_or(0.0, |b| b.position.x)),
            ("bullet_0.position.y", game.bullets.get(0).map_or(0.0, |b| b.position.y)),
            ("expl_0.position.x", game.explosions.get(0).map_or(0.0, |e| e.position.x)),
            ("expl_0.position.y", game.explosions.get(0).map_or(0.0, |e| e.position.y)),
            ("alien_0.position.x", game.aliens.get(0).map_or(0.0, |a| a.position.x)),
            ("alien_0.position.y", game.aliens.get(0).map_or(0.0, |a| a.position.y)),
        ]
        .iter()
        .enumerate()
        .for_each(|(i, (name, val))| {
            text::draw_text(&format!("{}: {}", name, val), 0.0, 16.0 * (i + 1) as f32, 16.0, color)
        });
    }
}

fn draw_ship(smooth_pos: math::Vec2, sprite: &cmpt::Sprite, has_exhaust: bool) {
    let position = math::vec2(smooth_pos.x as i32 as f32 + 0.5, smooth_pos.y as i32 as f32 + 0.5);
    let &cmpt::Sprite { ref variant, angle, .. } = sprite;
    let radius = cfg::SHIP_DRAW_RADIUS;
    let layers = match variant {
        cmpt::SpriteVariant::Vector { layers } => layers,
        _ => unreachable!(),
    };
    // let draw_points = vec![
    //     math::vec2(1.0, 0.0) * radius,
    //     math::vec2(-0.70710677, 0.70710677) * radius,
    //     math::vec2(-0.31606203, 0.2451628) * radius,
    //     math::vec2(-0.31606197, -0.24516287) * radius,
    //     math::vec2(-0.70710665, -0.7071069) * radius,
    // ];
    // draw_polygon(&layers[0].0, position, 0.0, layers[0].1);
    draw_layers(layers, position, angle);
    if has_exhaust {
        let v1_offset = math::vec2(
            (angle + PI * 0.85).cos() * radius * 0.55,
            (angle + PI * 0.85).sin() * radius * 0.55,
        );
        let v2_offset = math::vec2(
            (angle + PI * rand::gen_range(0.98, 1.02)).cos() * radius * rand::gen_range(0.75, 1.25),
            (angle + PI * rand::gen_range(0.98, 1.02)).sin() * radius * rand::gen_range(0.75, 1.25),
        );
        let v3_offset = math::vec2(
            (angle + PI * 1.15).cos() * radius * 0.55,
            (angle + PI * 1.15).sin() * radius * 0.55,
        );
        shapes::draw_triangle(
            position + v1_offset,
            position + v2_offset,
            position + v3_offset,
            palette::WHITE,
        );
    }
}

fn draw_polygon(
    draw_points: &[math::Vec2],
    offset: math::Vec2,
    rotation: f32,
    color: color::Color,
) {
    use macroquad::prelude::{DrawMode, Vertex};
    let gl = unsafe { window::get_internal_gl().quad_gl };
    let vertices: Vec<_> = draw_points
        .iter()
        .map(|&p| math::Mat2::from_angle(rotation).mul_vec2(p))
        .map(|p| p + offset)
        .map(|p| Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, color))
        .collect();
    let indices: Vec<_> = (1..(draw_points.len() as u16 - 1))
        .flat_map(|i| [0, i, i + 1])
        .collect();
    gl.texture(None);
    gl.draw_mode(DrawMode::Triangles);
    gl.geometry(&vertices, &indices);
}

fn draw_layers(layers: &[(Vec<math::Vec2>, color::Color)], offset: math::Vec2, rotation: f32) {
    for (draw_points, color) in layers {
        draw_polygon(draw_points, offset, rotation, *color);
    }
}
