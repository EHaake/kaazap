//! The opponent-select screen: choose who to face before a match. A flat,
//! vertical list over the [`OPPONENTS`](crate::opponent::OPPONENTS) roster,
//! navigated like the start menu (arrows / `w`·`s` / the emacs `Ctrl+P`·`Ctrl+N`
//! chords, which arrive here pre-translated to arrows by `resolve_key`) and
//! sharing the one selection pulse. Reached from Start Game; the campaign map
//! (spec D) will eventually supersede it as the way opponents are chosen. See
//! `specs/007-opponent-roster`.

use crossterm::event::KeyCode;

use crate::{
    config::Config,
    frame::{Emphasis, Frame, draw_text_centered},
    layout::MenuLayout,
    opponent::{OPPONENTS, OpponentProfile},
};

/// The result of a key on the select screen: the cursor moved, an opponent was
/// chosen, or the player backed out. Lets the app play the matching menu SFX
/// and transition. (One enum rather than `menu.rs`'s action/event split — the
/// roster is fixed, so there's no separate "activate the current item" step.)
#[derive(Debug, Copy, Clone)]
pub enum SelectOutcome {
    Moved,
    Picked(OpponentProfile),
    Back,
}

#[derive(Debug)]
pub struct OpponentSelectState {
    selected: usize, // index into OPPONENTS
}

impl Default for OpponentSelectState {
    fn default() -> Self {
        Self::new()
    }
}

impl OpponentSelectState {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// Handle a key: Up/Down (and `w`/`s`) move; Enter/Space pick the
    /// highlighted opponent; Esc/`x` back out. Returns `None` for keys this
    /// screen ignores.
    pub fn handle_input(&mut self, key: KeyCode) -> Option<SelectOutcome> {
        match key {
            KeyCode::Up => {
                self.move_selection(-1);
                Some(SelectOutcome::Moved)
            }
            KeyCode::Down => {
                self.move_selection(1);
                Some(SelectOutcome::Moved)
            }
            KeyCode::Enter => Some(SelectOutcome::Picked(OPPONENTS[self.selected])),
            KeyCode::Esc => Some(SelectOutcome::Back),
            KeyCode::Char(c) => match c {
                'w' => {
                    self.move_selection(-1);
                    Some(SelectOutcome::Moved)
                }
                's' => {
                    self.move_selection(1);
                    Some(SelectOutcome::Moved)
                }
                ' ' => Some(SelectOutcome::Picked(OPPONENTS[self.selected])),
                'x' => Some(SelectOutcome::Back),
                _ => None,
            },
            _ => None,
        }
    }

    /// Move the selection by `delta` over the roster, wrapping at the ends.
    fn move_selection(&mut self, delta: isize) {
        let n = OPPONENTS.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(n) as usize;
    }

    /// Draw the title, one row per opponent (`name — difficulty`; the selected
    /// row marked with `▸` and breathing with the pulse), the selected
    /// opponent's blurb, and a controls hint. Reuses `MenuLayout` for the
    /// vertically-centered block.
    pub fn draw(&self, frame: &mut Frame, config: &Config, pulse: Emphasis) {
        const TITLE: &str = "Choose Your Opponent";
        const HINT: &str = "↑/↓ choose  ·  Enter play  ·  Esc back";

        // Reserve the rows below the list for the blurb (at `y + 2`) and hint
        // (at `y + 4`) so the full 10-opponent roster plus its footer fits the
        // minimum terminal (one item-spacing + those four rows = 6).
        let layout = MenuLayout::new(*config, 1, OPPONENTS.len(), 6);

        draw_text_centered(frame, layout.center_x, layout.title_top, TITLE, Emphasis::Normal);

        let mut y = layout.items_top;
        for (i, o) in OPPONENTS.iter().enumerate() {
            let row = format!("{}  —  {}", o.name, o.difficulty);
            let (text, emphasis) = if self.selected == i {
                (format!("▸ {row}"), pulse)
            } else {
                (row, Emphasis::Normal)
            };
            draw_text_centered(frame, layout.center_x, y, &text, emphasis);
            y += layout.item_spacing;
        }

        // The selected opponent's blurb, then the controls hint, below the
        // list — with a clear gap above the blurb so it reads as its own
        // element rather than crowding the last row (playtest: it sat too
        // close).
        let blurb = OPPONENTS[self.selected].blurb;
        draw_text_centered(frame, layout.center_x, y + 2, blurb, Emphasis::Normal);
        draw_text_centered(frame, layout.center_x, y + 4, HINT, Emphasis::Normal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_wraps_over_the_roster() {
        let mut s = OpponentSelectState::new();
        assert_eq!(s.selected, 0);
        s.handle_input(KeyCode::Up); // up from the top wraps to the bottom
        assert_eq!(s.selected, OPPONENTS.len() - 1);
        s.handle_input(KeyCode::Down); // down from the bottom wraps to the top
        assert_eq!(s.selected, 0);
        s.handle_input(KeyCode::Char('s')); // w/s mirror up/down
        assert_eq!(s.selected, 1);
        s.handle_input(KeyCode::Char('w'));
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn enter_and_space_pick_the_highlighted_opponent() {
        let mut s = OpponentSelectState::new();
        s.handle_input(KeyCode::Down); // now on OPPONENTS[1]
        match s.handle_input(KeyCode::Enter) {
            Some(SelectOutcome::Picked(o)) => assert_eq!(o.id, OPPONENTS[1].id),
            other => panic!("expected Picked, got {other:?}"),
        }
        assert!(matches!(
            s.handle_input(KeyCode::Char(' ')),
            Some(SelectOutcome::Picked(_))
        ));
    }

    #[test]
    fn esc_and_x_back_out_and_unknown_keys_are_ignored() {
        let mut s = OpponentSelectState::new();
        assert!(matches!(s.handle_input(KeyCode::Esc), Some(SelectOutcome::Back)));
        assert!(matches!(
            s.handle_input(KeyCode::Char('x')),
            Some(SelectOutcome::Back)
        ));
        assert!(s.handle_input(KeyCode::Char('z')).is_none());
    }

    #[test]
    fn the_full_roster_and_footer_fit_the_minimum_terminal() {
        // At the 89×31 minimum the title, all opponents, the blurb (drawn at
        // `y + 2`) and the controls hint (`y + 4`) must all land on-frame — the
        // footer reserve passed to MenuLayout is what makes the grown roster fit.
        let config = Config { num_cols: 89, num_rows: 31 };
        let layout = MenuLayout::new(config, 1, OPPONENTS.len(), 6);
        let after_items = layout.items_top + OPPONENTS.len() * layout.item_spacing;
        let hint_y = after_items + 4; // must match `draw`
        assert!(
            hint_y < config.num_rows,
            "controls hint at row {hint_y} clips the {}-row minimum terminal",
            config.num_rows
        );
    }
}
