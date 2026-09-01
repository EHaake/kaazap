use crate::{app::HandCursor, game::GameState, menu::MenuState, settings::SettingsState};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState>, cursor: HandCursor },
    Settings { settings_state: SettingsState },
}
