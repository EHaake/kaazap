use crossterm::event::KeyCode;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::{fmt, time::Duration};

use crate::{MENU_ANIMATION_TIME_MS, config::Config, frame::Frame};

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
}

#[derive(Debug, Copy, Clone)]
pub enum MenuEvent {
    Activate { menu_item: MenuItem },
}

#[derive(Debug, Copy, Clone)]
pub enum MenuAction {
    Select,
    SelectionDown,
    SelectionUp,
}

#[derive(Debug)]
pub struct MenuState {
    selected: MenuItem,
    time_accumulated: Duration,
    title_text: Vec<&'static str>,
    animation_state: bool,
}

impl MenuState {
    pub fn new() -> Self {
        let title_art = include_str!("../assets/kaazap_title.txt");
        let title_text = title_art.lines().collect::<Vec<&'static str>>();

        Self {
            selected: MenuItem::StartGame,
            time_accumulated: Duration::from_millis(0),
            title_text,
            animation_state: false,
        }
    }

    /// Draw Text Helper
    ///
    /// Takes the text to draw, location coords and frame to draw into
    fn draw_text(&self, text: &str, x: usize, y: usize, frame: &mut Frame) {
        for (i, ch) in text.chars().enumerate() {
            frame[x + i][y] = ch;
        }
    }

    /// Draw the title which is a Vector<&'static str>
    ///
    /// Iterate through each line and send it to draw_text
    fn draw_title(&self, x: usize, y: usize, frame: &mut Frame) {
        for (row, line) in self.title_text.iter().enumerate() {
            self.draw_text(line, x, y + row, frame);
        }
    }

    fn draw_menu_items(&self, x: usize, y: usize, frame: &mut Frame) {
        let mut padding_y = y + 15;

        for menu_item in MenuItem::iter() {
            let menu_item_text = menu_item.to_string();
            padding_y += 2;

            // If this is the selected item, draw an annotation
            if self.selected == menu_item {
                match self.animation_state {
                    true => {
                        let selected_text = format!("-- {} --", menu_item_text);
                        let padding_x = x - 2 - selected_text.len() / 2;
                        self.draw_text(&selected_text, padding_x, padding_y, frame);
                    }
                    false => {
                        let selected_text = format!("++ {} ++", menu_item_text);
                        let padding_x = x - 2 - selected_text.len() / 2;
                        self.draw_text(&selected_text, padding_x, padding_y, frame);
                    }
                }
            } else {
                let padding_x = x - 2 - menu_item_text.len() / 2;
                self.draw_text(&menu_item_text, padding_x, padding_y, frame);
            }
        }
    }

    /// Main draw fn figures out where to render each element, then sends it out
    ///
    pub fn draw(&self, frame: &mut Frame, config: &Config) {
        // TODO: stop using magic numbers for positioning
        let mid = config.num_cols / 2;
        let padding_x = self.title_text[1].len() / 2 - 19;
        let padding_y = 5;

        self.draw_title(mid - padding_x, padding_y, frame);

        self.draw_menu_items(mid, padding_y, frame);
    }

    /// Accumulate time up to duration to drive menu animations
    ///
    pub fn tick(&mut self, dt: Duration) {
        self.time_accumulated += dt;
        if self.time_accumulated >= Duration::from_millis(MENU_ANIMATION_TIME_MS) {
            // toggle anim status
            self.animation_state = !self.animation_state;
            self.time_accumulated -= Duration::from_millis(MENU_ANIMATION_TIME_MS);
        }
    }

    pub fn toggle_selected(&mut self) -> MenuItem {
        match self.selected {
            MenuItem::StartGame => self.selected = MenuItem::HowToPlay,
            MenuItem::HowToPlay => self.selected = MenuItem::StartGame,
        }

        self.selected
    }

    pub fn handle_menu_input(&mut self, key: KeyCode) -> Option<MenuAction> {
        self.menu_action_from_key(key)
    }

    /// Convert a key pressed into an Action
    ///
    pub fn menu_action_from_key(&self, key: KeyCode) -> Option<MenuAction> {
        match key {
            KeyCode::Up => Some(MenuAction::SelectionUp),
            KeyCode::Down => Some(MenuAction::SelectionDown),
            KeyCode::Enter => Some(MenuAction::Select),
            KeyCode::Char(c) => match c {
                'w' => Some(MenuAction::SelectionUp),
                's' => Some(MenuAction::SelectionDown),
                ' ' => Some(MenuAction::Select),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn apply_menu_action(&mut self, action: MenuAction) -> Option<MenuEvent> {
        match action {
            MenuAction::Select => Some(MenuEvent::Activate {
                menu_item: self.selected,
            }),
            MenuAction::SelectionDown => {
                self.toggle_selected();
                None
            }
            MenuAction::SelectionUp => {
                self.toggle_selected();
                None
            }
        }
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement display for MenuItem enum to turn variants into strings
///
impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuItem::StartGame => write!(f, "Start Game"),
            MenuItem::HowToPlay => write!(f, "How To Play"),
        }
    }
}
