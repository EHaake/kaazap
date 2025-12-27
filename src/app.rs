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
            // screen: Screen::StartMenu {
            //     selected: MenuState {
            //         selected: MenuItem::StartGame,
            //     },
            //     },
            screen: Screen::InGame { game_state: GameState::new() },
            
            board_view: BoardView::new(config),
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match &mut self.screen {
            Screen::StartMenu { selected: _ } => {
                todo!()
            }
            Screen::InGame { game_state } => {
                if let Some(action) = game_state.handle_input(key) {
                    game_state.apply_action(action);
                }
            }
        }
    }

    pub fn tick(&mut self) {
        match &mut self.screen {
            Screen::StartMenu { selected: _ } => todo!(),
            Screen::InGame { game_state } => game_state.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match &mut self.screen {
            Screen::StartMenu { selected: _ } => todo!(),
            Screen::InGame { game_state } => self.board_view.draw(game_state, frame),
        }
    }
}
