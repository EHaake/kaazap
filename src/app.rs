use crate::{
    board::BoardView, config::Config, frame::Frame, game::GameState, screen::{MenuItem, MenuState, Screen}
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
            screen: Screen::StartMenu {
                selected: MenuState {
                    selected: MenuItem::StartGame,
                },
            },
            board_view: BoardView::new(config),
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match self.screen {
            Screen::StartMenu { selected } => {
                self.handle_menu_input(key);
            }
            Screen::InGame { game_state } => {
                self.handle_game_input(&mut game_state, key);
            }
        }
    }

    pub fn tick(&mut self) {}

    fn handle_menu_input(&self, key: char) -> _ {
        todo!()
    }

    fn handle_game_input(&self, game_state: &mut GameState, key: char) {
        if let Some(action) = game_state.handle_input(key) {
            game_state.apply_action(action);
        }
    }

    fn draw(&self, state: &GameState, frame: &mut Frame) {
        self.board_view.draw(state, frame);
    }
}
