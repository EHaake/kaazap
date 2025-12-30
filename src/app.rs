use std::time::Duration;

use crossterm::event::KeyCode;

use crate::{board::BoardView, config::Config, frame::Frame, menu::MenuState, screen::Screen};

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
            screen: Screen::StartMenu {
                menu_state: MenuState::new(),
            },
            board_view: BoardView::new(config),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match &mut self.screen {
            Screen::StartMenu { menu_state } => {
                match key {
                    KeyCode::Char(c) => match c {
                        'w' | 's' => menu_state.toggle_selected(),
                        ' ' => {
                            menu_state.activate_menu_selection();
                        }
                        _ => {}
                    },
                    KeyCode::Enter => {
                        menu_state.activate_menu_selection();
                    }
                    KeyCode::Up | KeyCode::Down => {
                        menu_state.toggle_selected();
                    }
                    _ => {}
                }
            }

            Screen::InGame { game_state } => {
                match key {
                    KeyCode::Char(c) => {
                        if let Some(action) = game_state.handle_game_input(c) {
                            game_state.apply_game_action(action);
                        }
                    }
                    KeyCode::Esc => {
                        // TODO: Go back to main menu
                    }
                    _ => {}
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
            Screen::StartMenu {
                menu_state: _menu_state,
            } => self.screen.draw(frame, &self.config),
            Screen::InGame { game_state } => self.board_view.draw(game_state, frame),
        }
    }
}
