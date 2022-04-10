use macroquad::{material, miniquad::date, rand, time, window};
use std::collections::HashSet;

mod cfg;
mod entity;
mod palette;
mod sprites;
mod systems;

#[derive(Default)]
pub struct Game {
    state: entity::GameState,
    renderer: entity::Renderer,
    player_actions: HashSet<entity::Action>,
    break_timer: f32,
    alien_timer: f32,
    star_bg: entity::StarBackground,
    ship: Option<entity::Ship>,
    bullets: Vec<entity::Bullet>,
    asteroids: Vec<entity::Asteroid>,
    explosions: Vec<entity::Explosion>,
    aliens: Vec<entity::Alien>,
}

fn load(game: &mut Game) {
    game.renderer.crt_effect = Some(
        material::load_material(
            include_str!("crt.vert"),
            include_str!("crt.frag"),
            Default::default(),
        )
        .unwrap(),
    );
    game.alien_timer = cfg::ALIEN_SPAWN_PERIOD;
}

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: String::from("asteroids"),
        window_width: cfg::ARENA_WIDTH as i32 * 3,
        window_height: cfg::ARENA_HEIGHT as i32 * 3,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(date::now() as u64);
    let mut game = Default::default();
    load(&mut game);
    loop {
        let delta_time = time::get_frame_time();
        systems::input::update(&mut game, delta_time);
        systems::ai::update(&mut game, delta_time);
        systems::timers::update(&mut game, delta_time);
        systems::moving::update(&mut game, delta_time);
        systems::collision::update(&mut game, delta_time);
        systems::damage::update(&mut game, delta_time);
        systems::gamestate::update(&mut game, delta_time);
        systems::cleanup::update(&mut game, delta_time);
        systems::spawn::update(&mut game, delta_time);
        systems::draw::update(&mut game, delta_time);
        window::next_frame().await;
    }
}
