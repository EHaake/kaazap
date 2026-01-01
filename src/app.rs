use std::time::Duration;

use crossterm::event::KeyCode;

use crate::{
    board::BoardView, config::Config, frame::Frame, game::GameState, menu::{MenuEvent, MenuItem, MenuState}, overlay::{Overlay, OverlayKind}, screen::Screen
};

pub struct App {
    pub config: Config,
    screen: Screen,
    board_view: BoardView,
    overlay: Option<Overlay>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            screen: Screen::StartMenu {
                menu_state: MenuState::new(),
            },
            board_view: BoardView::new(config),
            overlay: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        if self.overlay.is_some() {
            // Handle overlay keybinds
            if let KeyCode::Char(c) = key
                && c == '?' {
                    self.overlay = None
                }
        } else {
            // If overlay is not enabled, if ? is pressed, enable overlay
            if let KeyCode::Char(c) = key
                && c == '?' {
                    self.overlay = match &mut self.screen {
                        Screen::StartMenu { menu_state: _ } => {
                            Some(Overlay::new(OverlayKind::MenuHelp, self.config))
                        }
                        Screen::InGame { game_state: _ } => {
                            Some(Overlay::new(OverlayKind::GameHelp, self.config))
                        }
                    };
                }

            match &mut self.screen {
                // Route the Menu inputs only to Menu
                Screen::StartMenu { menu_state } => {
                    if let Some(menu_action) = menu_state.handle_menu_input(key)
                        && let Some(menu_event) = menu_state.apply_menu_action(menu_action)
                    {
                        self.apply_menu_event(menu_event);
                    }
                }

                // Route the game inputs to game_state
                Screen::InGame { game_state } => {
                    if let KeyCode::Char(c) = key {
                        match c {
                            'x' => {
                                self.screen = Screen::StartMenu {
                                    menu_state: MenuState::new(),
                                }
                            }
                            _ => {
                                if let Some(game_action) = game_state.handle_game_input(c) {
                                    game_state.apply_game_action(game_action);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// MenuEvent will contain one of the screens to switch to
    ///
    fn apply_menu_event(&mut self, menu_event: MenuEvent) {
        let MenuEvent::Activate { menu_item } = menu_event;

        match menu_item {
            MenuItem::StartGame => {
                self.screen = Screen::InGame {
                    game_state: Box::new(GameState::new()),
                };
            }
            MenuItem::HowToPlay => {}
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

        if let Some(overlay) = &self.overlay {
            overlay.draw(frame);
        }
    }
}
