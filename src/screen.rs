use crate::{config::Config, frame::Frame, game::GameState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
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

    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        // let mid = config.num_cols / 2;

        match self {
            Screen::StartMenu { menu_state } => self.draw_menu(menu_state, frame, config),
            Screen::InGame { game_state } => self.draw_game(game_state),
        }

        
    }

    fn draw_menu(&self, menu_state: &MenuState, frame: &mut Frame, config: &Config) {
        todo!()
    }

    fn draw_game(&self, game_state: &GameState) {
        todo!()
    }
}
