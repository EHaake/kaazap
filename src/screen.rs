use crate::{frame::Frame, game::GameState};

pub enum Screen {
    StartMenu { selected: MenuState },
    InGame { game_state: Box<GameState> },
}

#[derive(Debug, Copy, Clone)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
}

#[derive(Debug)]
pub struct MenuState {
    pub(crate) selected: MenuItem,
}

impl Screen {

    pub fn draw(&self, state: &MenuState, frame: &mut Frame) {

    }
}
