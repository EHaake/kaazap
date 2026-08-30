use std::time::Duration;

use crate::{app::HandCursor, config::Config, frame::Frame, game::GameState, menu::MenuState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState>, cursor: HandCursor },
}

impl Screen {
    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        match self {
            Screen::StartMenu { menu_state } => menu_state.draw(frame, config),
            // app calls board.draw() so do nothing if InGame
            Screen::InGame { .. } => {}
        }
    }
}
