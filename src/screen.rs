use crate::{app::HandCursor, game::GameState, menu::MenuState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState>, cursor: HandCursor },
}
