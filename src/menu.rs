use crossterm::event::KeyCode;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::fmt;

use crate::{TITLE_X_OFFSET, config::Config, frame::{Emphasis, Frame, draw_text}, layout::MenuLayout};

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    HowToPlay,
    Settings,
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
    title_text: Vec<&'static str>,
}

impl MenuState {
    pub fn new() -> Self {
        let title_art = include_str!("../assets/kaazap_title.txt");
        let title_text = title_art.lines().collect::<Vec<&'static str>>();

        Self {
            selected: MenuItem::StartGame,
            title_text,
        }
    }

    /// Convert a KeyCode from the main gameloop and return a MenuAction
    ///
    pub fn handle_menu_input(&mut self, key: KeyCode) -> Option<MenuAction> {
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

    /// Take a MenuAction and return an optional MenuEvent
    ///
    pub fn apply_menu_action(&mut self, action: MenuAction) -> Option<MenuEvent> {
        match action {
            MenuAction::Select => Some(MenuEvent::Activate {
                menu_item: self.selected,
            }),
            MenuAction::SelectionDown => {
                self.move_selection(1);
                None
            }
            MenuAction::SelectionUp => {
                self.move_selection(-1);
                None
            }
        }
    }

    /// Move the selection by `delta` over the ordered menu items, wrapping
    /// at the ends. Works for any number of items.
    pub fn move_selection(&mut self, delta: isize) {
        let items: Vec<MenuItem> = MenuItem::iter().collect();
        let n = items.len() as isize;
        let current = items.iter().position(|&i| i == self.selected).unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(n) as usize;
        self.selected = items[next];
    }

    /// Draw the title which is a Vector<&'static str>
    ///
    /// Iterate through each line and send it to draw_text
    fn draw_title(&self, x: usize, y: usize, frame: &mut Frame) {
        for (row, line) in self.title_text.iter().enumerate() {
            draw_text(frame, x, y + row, line, Emphasis::Normal);
        }
    }

    /// Draw the menu items. The selected item carries a constant "▸ "
    /// marker (its always-on selection anchor) and breathes with the
    /// shared pulse emphasis; unselected items render plain.
    fn draw_menu_items(&self, layout: &MenuLayout, pulse: Emphasis, frame: &mut Frame) {
        let mut y = layout.items_top;

        for menu_item in MenuItem::iter() {
            let (text, emphasis) = if self.selected == menu_item {
                (format!("▸ {menu_item}"), pulse)
            } else {
                (menu_item.to_string(), Emphasis::Normal)
            };

            let x = layout.center_x - text.chars().count() / 2;
            draw_text(frame, x, y, &text, emphasis);

            y += layout.item_spacing;
        }
    }

    /// Main draw fn figures out where to render each element, then sends it out
    ///
    pub fn draw(&self, frame: &mut Frame, config: &Config, pulse: Emphasis) {
        let layout = MenuLayout::new(*config, self.title_text.len(), MenuItem::iter().count());

        // Title art keeps its own centering (leading-whitespace aware),
        // anchored on the layout's center. Saturating so an edited art
        // asset can't underflow-panic.
        let title_x = layout
            .center_x
            .saturating_sub((self.title_text[1].len() / 2).saturating_sub(TITLE_X_OFFSET));
        self.draw_title(title_x, layout.title_top, frame);

        self.draw_menu_items(&layout, pulse, frame);
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_selection_moves_over_all_items_and_wraps() {
        let mut m = MenuState::new();
        assert_eq!(m.selected, MenuItem::StartGame); // starts at the top
        m.move_selection(1);
        assert_eq!(m.selected, MenuItem::HowToPlay);
        m.move_selection(1);
        assert_eq!(m.selected, MenuItem::Settings);
        m.move_selection(1); // wraps to the top
        assert_eq!(m.selected, MenuItem::StartGame);
        m.move_selection(-1); // wraps backward to the bottom
        assert_eq!(m.selected, MenuItem::Settings);
    }
}

/// Implement display for MenuItem enum to turn variants into strings
///
impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuItem::StartGame => write!(f, "Start Game"),
            MenuItem::HowToPlay => write!(f, "How To Play"),
            MenuItem::Settings => write!(f, "Settings"),
        }
    }
}
