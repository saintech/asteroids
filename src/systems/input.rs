use crate::{cfg, entity};
use macroquad::input;

pub fn update(game: &mut crate::Game, _dt: f32) {
    game.player_actions.clear();
    match game.state {
        entity::GameState::Pause => {
            use entity::Action::*;
            let pause_key = match cfg::KEYMAP[4] {
                (pause_key, TogglePause) => pause_key,
                _ => unreachable!(),
            };
            if input::is_key_pressed(pause_key) {
                game.player_actions.insert(TogglePause);
            }
        }
        entity::GameState::LevelRunning => {
            use entity::Action::*;
            for &(key, action) in cfg::KEYMAP {
                if action == TogglePause || action == ToggleDebugInfo {
                    if input::is_key_pressed(key) {
                        game.player_actions.insert(action);
                    }
                } else {
                    if input::is_key_down(key) {
                        game.player_actions.insert(action);
                    }
                }
            }
        }
        _ => (),
    }
}
