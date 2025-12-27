use crate::game::GameState;

pub enum Screen {
    StartMenu { selected: MenuState },
    InGame { game_state: GameState },
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

}
