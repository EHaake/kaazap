use crate::game::GameState;

pub enum Screen {
    StartMenu { selected: MenuItem },
    InGame { game_state: GameState },
}

#[derive(Copy, Clone)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
}
