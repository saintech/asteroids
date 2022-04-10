use crate::entity::cmpt;

pub fn update(game: &mut crate::Game, _dt: f32) {
    // dbg!(&game.state);
    match game.state {
        cmpt::GameState::Pause => {
            if game.player_actions.contains(&cmpt::Action::TogglePause) {
                game.state = cmpt::GameState::LevelRunning;
            }
        }
        cmpt::GameState::LevelLoading => {
            if game.ship.is_some() && !game.asteroids.is_empty() {
                game.state = cmpt::GameState::LevelRunning;
            }
        }
        cmpt::GameState::LevelRunning => {
            if game.player_actions.contains(&cmpt::Action::TogglePause) {
                game.state = cmpt::GameState::Pause;
            }
            if game.ship.as_ref().map_or(false, |sh| sh.is_destroyed) {
                game.break_timer = 2.0;
                game.state = cmpt::GameState::GameOver;
            }
            if game.asteroids.is_empty() && game.aliens.is_empty() {
                game.break_timer = 2.0;
                game.state = cmpt::GameState::LevelCompleted;
            }
        }
        cmpt::GameState::LevelCompleted => {
            if game.break_timer == 0.0 {
                let old_game = std::mem::replace(game, Default::default());
                game.renderer = old_game.renderer;
                game.alien_timer = old_game.alien_timer;
                game.star_bg = old_game.star_bg;
            }
        }
        cmpt::GameState::GameOver => {
            if game.break_timer == 0.0 {
                let old_game = std::mem::replace(game, Default::default());
                game.renderer = old_game.renderer;
                game.alien_timer = old_game.alien_timer;
                game.star_bg = old_game.star_bg;
            }
        }
    }
}
