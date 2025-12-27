use std::time::Duration;

use crate::{
    board::BoardView, config::Config, frame::Frame, game::GameState, screen::{MenuState, Screen}
};

pub struct App {
    pub config: Config,
    screen: Screen,
    board_view: BoardView,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            // screen: Screen::StartMenu {
            //     selected: MenuState {
            //         selected: MenuItem::StartGame,
            //     },
            //     },

            // This enters a new game automatically since we set Screen to be InGame
            // screen: Screen::InGame { game_state: Box::new(GameState::new()) },

            screen: Screen::StartMenu { menu_state: MenuState::new() },
            board_view: BoardView::new(config),
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match &mut self.screen {
            Screen::StartMenu { menu_state: _ } => {
                todo!()
            }
            Screen::InGame { game_state } => {
                if let Some(action) = game_state.handle_input(key) {
                    game_state.apply_action(action);
                }
            }
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        match &mut self.screen {
            Screen::StartMenu { menu_state } => menu_state.tick(dt),
            Screen::InGame { game_state } => game_state.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match &self.screen {
            Screen::StartMenu { menu_state: _menu_state } => self.screen.draw(frame, &self.config),
            Screen::InGame { game_state } => self.board_view.draw(game_state, frame),
        }
    }
}
