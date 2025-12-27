use std::time::Duration;

use crate::{config::Config, frame::Frame, game::GameState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
}

#[derive(Debug)]
pub struct MenuState {
    selected: MenuItem,
    time_accumulated: Duration,
}

impl Screen {
    pub fn draw(&self, frame: &mut Frame, config: &Config) {

        // app call board.draw() so do nothing if InGame
        match self {
            Screen::StartMenu { menu_state } => menu_state.draw(frame, config),
            Screen::InGame { game_state: _game_state } => {}
        }
    }
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            selected: MenuItem::StartGame,
            time_accumulated: Duration::from_millis(0),
        }
    }

    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        // let mid = config.num_cols / 2;
        
    }

    /// 
    pub fn tick(&mut self, dt: Duration) {
        self.time_accumulated += dt;
        if self.time_accumulated >= Duration::from_millis(350) {
            // toggle anim status
            // anim_state.toggle();
            self.time_accumulated -= Duration::from_millis(350);
        }

    }
}
