use crate::screen::{MenuItem, MenuState, Screen};

pub struct App {
    screen: Screen,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::StartMenu { selected: MenuState { selected: MenuItem::StartGame } }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
