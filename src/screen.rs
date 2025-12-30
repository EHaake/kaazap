use std::time::Duration;

use crate::{config::Config, frame::Frame, game::GameState, menu::MenuState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState> },
}


impl Screen {
    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        // app calls board.draw() so do nothing if InGame
        match self {
            Screen::StartMenu { menu_state } => menu_state.draw(frame, config),
            Screen::InGame {
                game_state: _game_state,
            } => {}
        }
    }
}

