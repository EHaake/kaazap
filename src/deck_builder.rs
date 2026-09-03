//! The deck-builder screen: browse the cards you own and swap them in and out
//! of your side deck. A full mode navigated *to* (a [`Screen`](crate::screen),
//! not an overlay), reached from the start menu. Mirrors `opponent_select.rs`'s
//! plumbing — a cursor + an outcome enum + `draw(frame, config, …, pulse)` and
//! one app.rs arm — but over a 2D grid of cards instead of a vertical list.
//!
//! The screen owns only the cursor; the collection and deck live in the
//! [`Profile`], and every edit is applied through the profile's own methods
//! (the app performs it), so the deck-building rules stay in one place. See
//! `specs/008-side-deck-customization`.

use crossterm::event::KeyCode;

use crate::{
    CARD_HEIGHT, CARD_WIDTH, SIDE_DECK_SIZE,
    card::{Card, CardView},
    config::Config,
    frame::{BorderWeight, Drawable, Emphasis, Frame, draw_text},
    layout::GridLayout,
    profile::Profile,
};

/// How many columns the collection grid uses. Five holds the whole 15-card
/// universe in three rows, which fits the minimum terminal (see the
/// `GridLayout` test); clamped down for smaller collections.
const COLS: usize = 5;

/// The columns actually used for a collection of `count` cards — never more
/// than there are cards (so a tiny collection doesn't leave phantom columns).
fn grid_cols(count: usize) -> usize {
    COLS.min(count.max(1))
}

/// The result of a key on the deck-builder: the cursor moved, a copy of a card
/// should be added to or removed from the deck, or the player is done. The app
/// performs the add/remove through the [`Profile`] and plays the matching SFX.
/// (`Add`/`Remove` carry the *intent*; the profile decides if it's legal.)
#[derive(Debug, Copy, Clone)]
pub enum BuildOutcome {
    Moved,
    Add(Card),
    Remove(Card),
    Back,
}

#[derive(Debug)]
pub struct DeckBuilderState {
    cursor: usize, // index into profile.collection_by_type()
}

impl Default for DeckBuilderState {
    fn default() -> Self {
        Self::new()
    }
}

impl DeckBuilderState {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    /// Handle a key against the current `profile`: arrows / `wasd` move the 2D
    /// cursor; Enter/Space add a copy of the highlighted card; Backspace/`-`
    /// remove one; Esc/`x` leave. `None` for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<BuildOutcome> {
        let entries = profile.collection_by_type();
        let n = entries.len();
        // With nothing owned there's nothing to move over or build — only
        // backing out is meaningful. (Can't happen with the starter, but the
        // grid is data-driven, so guard rather than index into an empty list.)
        if n == 0 {
            return match key {
                KeyCode::Esc | KeyCode::Char('x') => Some(BuildOutcome::Back),
                _ => None,
            };
        }
        // Keep the cursor in range if the collection ever shrinks under it.
        self.cursor = self.cursor.min(n - 1);
        let cols = grid_cols(n);

        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                self.move_vertical(-1, n, cols);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_vertical(1, n, cols);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Left | KeyCode::Char('a') => {
                self.move_horizontal(-1, n);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_horizontal(1, n);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Enter | KeyCode::Char(' ') => Some(BuildOutcome::Add(entries[self.cursor].card)),
            KeyCode::Backspace | KeyCode::Char('-') => {
                Some(BuildOutcome::Remove(entries[self.cursor].card))
            }
            KeyCode::Esc | KeyCode::Char('x') => Some(BuildOutcome::Back),
            _ => None,
        }
    }

    /// Move left/right over the flat list, wrapping at the ends (reading
    /// order — off the end of a row continues onto the next).
    fn move_horizontal(&mut self, delta: isize, n: usize) {
        self.cursor = (self.cursor as isize + delta).rem_euclid(n as isize) as usize;
    }

    /// Move up/down within the cursor's column, wrapping top-to-bottom and
    /// skipping the ragged empty cell a short last row leaves.
    fn move_vertical(&mut self, dir: isize, n: usize, cols: usize) {
        let col = self.cursor % cols;
        let rows = n.div_ceil(cols) as isize;
        let mut row = (self.cursor / cols) as isize;
        for _ in 0..rows {
            row = (row + dir).rem_euclid(rows);
            let idx = row as usize * cols + col;
            if idx < n {
                self.cursor = idx;
                return;
            }
        }
    }

    /// Draw the title, the "Deck: N/10" readout, the grid of owned cards (each
    /// a card box with an `in-deck/owned` caption; the cursored one heavy +
    /// pulsing, in-deck ones emphasized), and the controls hint.
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        const TITLE: &str = "Side Deck";
        const HINT: &str = "↑/↓/←/→ move  ·  Enter add  ·  Backspace remove  ·  Esc done";

        let entries = profile.collection_by_type();
        let layout = GridLayout::new(*config, entries.len(), grid_cols(entries.len()));
        let center = |text: &str| layout.center_x.saturating_sub(text.chars().count() / 2);

        draw_text(frame, center(TITLE), layout.title_y, TITLE, Emphasis::Normal);

        // The deck-size readout: how close the built deck is to legal. Alert
        // while short, so an incomplete deck (the only thing blocking a match)
        // is obvious.
        let n = profile.deck().len();
        let (readout, readout_emphasis) = if profile.deck_is_valid() {
            (format!("Deck: {n}/{SIDE_DECK_SIZE}"), Emphasis::Normal)
        } else {
            let short = SIDE_DECK_SIZE.saturating_sub(n);
            (format!("Deck: {n}/{SIDE_DECK_SIZE} — add {short} more"), Emphasis::Alert)
        };
        draw_text(frame, center(&readout), layout.readout_y, &readout, readout_emphasis);

        for (i, entry) in entries.iter().enumerate() {
            let (x, y) = layout.card_origin(i);
            let cursored = i == self.cursor;

            let mut view = CardView::new(x, y, entry.card.label());
            view.weight = if cursored { BorderWeight::Heavy } else { BorderWeight::Single };
            view.emphasis = card_emphasis(cursored, entry.in_deck > 0, pulse);
            view.draw(frame);

            // Caption row beneath the card: how many of this card are in the
            // deck out of how many are owned.
            let badge = format!("{}/{}", entry.in_deck, entry.owned);
            let badge_x = x + CARD_WIDTH.saturating_sub(badge.chars().count()) / 2;
            draw_text(frame, badge_x, y + CARD_HEIGHT, &badge, card_emphasis(cursored, entry.in_deck > 0, pulse));
        }

        draw_text(frame, center(HINT), layout.hint_y, HINT, Emphasis::Muted);
    }
}

/// The emphasis for a card and its caption: the cursored one pulses; cards
/// with a copy in the deck stand out; the rest recede.
fn card_emphasis(cursored: bool, in_deck: bool, pulse: Emphasis) -> Emphasis {
    if cursored {
        pulse
    } else if in_deck {
        Emphasis::Strong
    } else {
        Emphasis::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_move_the_cursor_over_the_grid() {
        let p = Profile::default();
        let entries = p.collection_by_type();
        let cols = grid_cols(entries.len());
        assert!(entries.len() > cols + 1, "test needs a multi-row starter grid");

        let mut s = DeckBuilderState::new();
        assert_eq!(s.cursor, 0);

        assert!(matches!(s.handle_input(KeyCode::Right, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.cursor, 1);
        assert!(matches!(s.handle_input(KeyCode::Down, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.cursor, 1 + cols); // one row down, same column
        assert!(matches!(s.handle_input(KeyCode::Up, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.cursor, 1); // back up

        // wasd mirror the arrows.
        assert!(matches!(s.handle_input(KeyCode::Char('a'), &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn left_from_the_start_wraps_to_the_end() {
        let p = Profile::default();
        let last = p.collection_by_type().len() - 1;
        let mut s = DeckBuilderState::new();
        assert!(matches!(s.handle_input(KeyCode::Left, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.cursor, last);
    }

    #[test]
    fn enter_and_space_add_the_highlighted_card_backspace_removes_it() {
        let p = Profile::default();
        let entries = p.collection_by_type();
        let mut s = DeckBuilderState::new();
        let under = entries[s.cursor].card;

        assert!(matches!(s.handle_input(KeyCode::Enter, &p), Some(BuildOutcome::Add(c)) if c == under));
        assert!(matches!(s.handle_input(KeyCode::Char(' '), &p), Some(BuildOutcome::Add(c)) if c == under));
        assert!(matches!(s.handle_input(KeyCode::Backspace, &p), Some(BuildOutcome::Remove(c)) if c == under));
        assert!(matches!(s.handle_input(KeyCode::Char('-'), &p), Some(BuildOutcome::Remove(c)) if c == under));
    }

    #[test]
    fn esc_and_x_back_out_and_unknown_keys_are_ignored() {
        let p = Profile::default();
        let mut s = DeckBuilderState::new();
        assert!(matches!(s.handle_input(KeyCode::Esc, &p), Some(BuildOutcome::Back)));
        assert!(matches!(s.handle_input(KeyCode::Char('x'), &p), Some(BuildOutcome::Back)));
        assert!(s.handle_input(KeyCode::Char('z'), &p).is_none());
    }
}
