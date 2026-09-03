use crossterm::event::KeyCode;

use std::fmt;

use crate::{TITLE_X_OFFSET, config::Config, frame::{Emphasis, Frame, draw_text}, layout::MenuLayout};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Continue,
    StartGame,
    SideDeck,
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
    // The menu items in display order. `Continue` leads the list only when a
    // resumable save exists, so the length varies (3 or 4).
    items: Vec<MenuItem>,
    selected: usize, // index into `items`
    title_text: Vec<&'static str>,
}

impl MenuState {
    /// Build the menu. `Continue` leads the list only when a resumable save
    /// exists — and is then the default selection, the likely action for a
    /// returning player; otherwise the list starts at Start Game.
    pub fn new(has_save: bool) -> Self {
        let title_art = include_str!("../assets/kaazap_title.txt");
        let title_text = title_art.lines().collect::<Vec<&'static str>>();

        let mut items = Vec::new();
        if has_save {
            items.push(MenuItem::Continue);
        }
        items.extend([
            MenuItem::StartGame,
            MenuItem::SideDeck,
            MenuItem::HowToPlay,
            MenuItem::Settings,
        ]);

        Self {
            items,
            selected: 0,
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
                menu_item: self.items[self.selected],
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

    /// Move the selection by `delta` over the current items, wrapping at the
    /// ends. Works for any number of items.
    pub fn move_selection(&mut self, delta: isize) {
        let n = self.items.len() as isize;
        if n == 0 {
            return;
        }
        self.selected = (self.selected as isize + delta).rem_euclid(n) as usize;
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

        for (i, menu_item) in self.items.iter().enumerate() {
            let (text, emphasis) = if self.selected == i {
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
        let layout = MenuLayout::new(*config, self.title_text.len(), self.items.len());

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

#[cfg(test)]
mod tests {
    use super::*;

    fn selected(m: &MenuState) -> MenuItem {
        m.items[m.selected]
    }

    #[test]
    fn menu_without_a_save_omits_continue_and_wraps_over_four() {
        let mut m = MenuState::new(false);
        assert_eq!(m.items.len(), 4);
        assert_eq!(selected(&m), MenuItem::StartGame); // default at the top
        m.move_selection(1);
        assert_eq!(selected(&m), MenuItem::SideDeck);
        m.move_selection(1);
        assert_eq!(selected(&m), MenuItem::HowToPlay);
        m.move_selection(1);
        assert_eq!(selected(&m), MenuItem::Settings);
        m.move_selection(1); // wraps to the top
        assert_eq!(selected(&m), MenuItem::StartGame);
        m.move_selection(-1); // wraps backward to the bottom
        assert_eq!(selected(&m), MenuItem::Settings);
    }

    #[test]
    fn menu_with_a_save_leads_with_continue_and_wraps_over_five() {
        let mut m = MenuState::new(true);
        assert_eq!(m.items.len(), 5);
        assert_eq!(selected(&m), MenuItem::Continue); // default for a returning player
        m.move_selection(-1); // wraps backward to the bottom
        assert_eq!(selected(&m), MenuItem::Settings);
        m.move_selection(1); // back to the top
        assert_eq!(selected(&m), MenuItem::Continue);
        m.move_selection(1); // Continue → Start Game → Side Deck
        m.move_selection(1);
        assert_eq!(selected(&m), MenuItem::SideDeck);
    }
}

/// Implement display for MenuItem enum to turn variants into strings
///
impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuItem::Continue => write!(f, "Continue"),
            MenuItem::StartGame => write!(f, "Start Game"),
            MenuItem::SideDeck => write!(f, "Side Deck"),
            MenuItem::HowToPlay => write!(f, "How To Play"),
            MenuItem::Settings => write!(f, "Settings"),
        }
    }
}
