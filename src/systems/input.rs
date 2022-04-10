use crate::{cfg, entity::cmpt};
use macroquad::input;

pub fn update(game: &mut crate::Game, _dt: f32) {
    game.player_actions.clear();
    match game.state {
        cmpt::GameState::Pause => {
            use cmpt::Action::*;
            let pause_key = match cfg::KEYMAP[4] {
                (pause_key, TogglePause) => pause_key,
                _ => unreachable!(),
            };
            if input::is_key_pressed(pause_key) {
                game.player_actions.insert(TogglePause);
            }
        }
        cmpt::GameState::LevelRunning => {
            use cmpt::Action::*;
            for &(key, action) in cfg::KEYMAP {
                if action == TogglePause {
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
