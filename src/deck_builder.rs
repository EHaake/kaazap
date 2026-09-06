//! The deck-builder screen: a two-panel "briefcase" for building your side
//! deck. The **Collection** panel (left) lists the card copies you own that
//! aren't placed; the **Deck** panel (right) lists the copies you've placed,
//! with a `Deck: N/10` readout. Building a deck is moving a card *copy* across:
//! confirming on a Collection card places one in the Deck, confirming on a Deck
//! card returns one. A full mode navigated *to* (a [`Screen`](crate::screen),
//! not an overlay), reached from the start menu. Mirrors `opponent_select.rs`'s
//! plumbing — per-panel cursors + an outcome enum + `draw(frame, config, …,
//! pulse)` and one app.rs arm.
//!
//! The screen owns only the cursors, the active panel, and the collection's
//! scroll offset; the collection and deck live in the [`Profile`], and every
//! edit is applied through the profile's own methods (the app performs it), so
//! the deck-building rules stay in one place. A card's *panel* is what conveys
//! whether it's in the deck, so the cards use just two border weights (Heavy =
//! cursor, Single otherwise) — the old "double = in deck" weight is gone. See
//! `specs/008-side-deck-customization` and `specs/015-briefcase-deck-builder`.

use crossterm::event::KeyCode;

use crate::{
    CARD_HEIGHT, CARD_WIDTH, SIDE_DECK_SIZE,
    card::{Card, CardView},
    config::Config,
    frame::{BorderWeight, Drawable, Emphasis, Frame, draw_box, draw_text, draw_text_centered},
    layout::{BriefcaseLayout, Panel},
    profile::Profile,
};

/// The collection panel's guaranteed-minimum visible card rows — the floor
/// [`BriefcaseLayout`] promises even at the 89×31 minimum terminal (asserted by
/// its `briefcase_fits_the_minimum_terminal` test). [`handle_input`] has no
/// [`Config`] (its signature is fixed by the app), so it can't read the real
/// `visible_rows`; it keeps `collection_scroll` following the cursor against
/// this floor instead. `draw`, which *does* have the real `visible_rows`,
/// clamps the offset to the actual window — and keeping the cursor inside the
/// minimum window keeps it inside any larger one.
///
/// [`handle_input`]: DeckBuilderState::handle_input
const MIN_VISIBLE_ROWS: usize = 3;

/// The result of a key on the deck-builder: the cursor moved (or the active
/// panel switched), a copy of a card should be added to or removed from the
/// deck, or the player is done. The app performs the add/remove through the
/// [`Profile`] and plays the matching SFX. (`Add`/`Remove` carry the *intent*;
/// the profile decides if it's legal.)
#[derive(Debug, Copy, Clone)]
pub enum BuildOutcome {
    Moved,
    Add(Card),
    Remove(Card),
    Back,
}

/// The card rows one panel shows: for the Collection, each owned type with a
/// spare copy (count = *available* = owned − in-deck); for the Deck, each type
/// with a placed copy (count = *in-deck*). Both in canonical `ALL_SIDE_CARDS`
/// order. A duplicate that's partly placed appears in *both* panels — that's
/// the split-by-location model (`spec.md`, "Resolved decisions").
fn collection_rows(profile: &Profile) -> Vec<(Card, usize)> {
    profile
        .collection_by_type()
        .into_iter()
        .filter_map(|e| {
            let available = e.owned.saturating_sub(e.in_deck);
            (available > 0).then_some((e.card, available))
        })
        .collect()
}

fn deck_rows(profile: &Profile) -> Vec<(Card, usize)> {
    profile
        .collection_by_type()
        .into_iter()
        .filter_map(|e| (e.in_deck > 0).then_some((e.card, e.in_deck)))
        .collect()
}

/// A panel's rows, dispatched by which panel it is.
fn panel_rows(panel: Panel, profile: &Profile) -> Vec<(Card, usize)> {
    match panel {
        Panel::Collection => collection_rows(profile),
        Panel::Deck => deck_rows(profile),
    }
}

/// The other panel.
fn other(panel: Panel) -> Panel {
    match panel {
        Panel::Collection => Panel::Deck,
        Panel::Deck => Panel::Collection,
    }
}

/// Where focus should rest, given which panels have cards: stay on `active`
/// unless it's empty and the other panel has rows — so the builder always
/// opens (and stays) on a non-empty side. An all-decked profile with no spares
/// opens on the Deck; a nothing-placed profile opens on the Collection.
fn resolve_focus(active: Panel, collection_empty: bool, deck_empty: bool) -> Panel {
    match active {
        Panel::Collection if collection_empty && !deck_empty => Panel::Deck,
        Panel::Deck if deck_empty && !collection_empty => Panel::Collection,
        _ => active,
    }
}

/// The minimal scroll (first-visible-row offset) that keeps `cursor_row` inside
/// a `visible_rows`-tall window over `total_rows`, moving the previous `scroll`
/// as little as possible and never past the last full window: scroll up to
/// reveal a cursor above the window, down to reveal one below, clamped into
/// `0..=max_scroll`.
fn scroll_to_reveal(scroll: usize, cursor_row: usize, visible_rows: usize, total_rows: usize) -> usize {
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let mut s = scroll.min(max_scroll);
    if cursor_row < s {
        s = cursor_row;
    } else if cursor_row >= s + visible_rows {
        s = cursor_row + 1 - visible_rows;
    }
    s.min(max_scroll)
}

/// Where the deck-builder was opened from, so `Back` returns there. The three
/// entry points funnel through the app's `open_deck_builder(origin)`, and the
/// `BuildOutcome::Back` arm routes on this — the menu vs. the campaign map. It
/// also fixes a pre-existing bug where the campaign-launch "incomplete deck"
/// divert dropped the player on the *menu* instead of the map (`spec.md`,
/// "Return-path correction").
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuilderOrigin {
    Menu,
    Map,
}

#[derive(Debug)]
pub struct DeckBuilderState {
    active: Panel,
    collection_cursor: usize, // index into collection_rows()
    deck_cursor: usize,       // index into deck_rows()
    collection_scroll: usize, // first visible row of the collection grid
    origin: BuilderOrigin,    // where Back returns to (menu vs. campaign map)
}

impl Default for DeckBuilderState {
    fn default() -> Self {
        Self::new(BuilderOrigin::Menu)
    }
}

impl DeckBuilderState {
    /// A fresh builder opened from `origin` (the menu or the campaign map),
    /// which `Back` returns to. Focus defaults to the Collection and is
    /// normalized onto a non-empty panel at the top of `handle_input`
    /// (`focus_nonempty`) — so an all-decked profile with no spares lands on the
    /// Deck once a key is pressed. (The starter collection can't be emptied, so
    /// Collection is always a valid default at open.)
    pub fn new(origin: BuilderOrigin) -> Self {
        Self {
            active: Panel::Collection,
            collection_cursor: 0,
            deck_cursor: 0,
            collection_scroll: 0,
            origin,
        }
    }

    /// Where this builder was opened from — the app's `Back` arm routes on it
    /// (the menu vs. the campaign map).
    pub fn origin(&self) -> BuilderOrigin {
        self.origin
    }

    /// Handle a key against the current `profile`: `Tab`/`BackTab` switch the
    /// active panel; arrows / `wasd` move that panel's cursor over its grid;
    /// `Enter`/`Space` move a copy across (add from the Collection, remove from
    /// the Deck); `Esc`/`x` leave. `None` for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<BuildOutcome> {
        // Backing out is always meaningful, even with nothing owned.
        if matches!(key, KeyCode::Esc | KeyCode::Char('x')) {
            return Some(BuildOutcome::Back);
        }

        // The previous add/remove (applied by the app after the last key) may
        // have emptied a side or shrunk a list under a cursor — normalize
        // before acting so focus rests on a non-empty panel and every cursor
        // is in range.
        self.focus_nonempty(profile);
        self.clamp_cursors(profile);

        if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
            // Switch panels, but never onto an empty one (there'd be nothing to
            // point at); if the other side is empty, focus stays put.
            let target = other(self.active);
            if !panel_rows(target, profile).is_empty() {
                self.active = target;
            }
            return Some(BuildOutcome::Moved);
        }

        let rows = panel_rows(self.active, profile);
        let n = rows.len();
        // The active side is empty only when *both* sides are (else
        // `focus_nonempty` moved us): nothing to move over or across.
        if n == 0 {
            return None;
        }
        let cols = BriefcaseLayout::COLS;

        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                self.move_vertical(-1, n, cols);
                self.follow_scroll_if_collection(n);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_vertical(1, n, cols);
                self.follow_scroll_if_collection(n);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Left | KeyCode::Char('a') => {
                self.move_horizontal(-1, n);
                self.follow_scroll_if_collection(n);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_horizontal(1, n);
                self.follow_scroll_if_collection(n);
                Some(BuildOutcome::Moved)
            }
            // Move a copy across: the direction is the active panel now.
            KeyCode::Enter | KeyCode::Char(' ') => {
                let card = rows[self.cursor(self.active)].0;
                Some(match self.active {
                    Panel::Collection => BuildOutcome::Add(card),
                    Panel::Deck => BuildOutcome::Remove(card),
                })
            }
            _ => None,
        }
    }

    /// The active panel's cursor.
    fn cursor(&self, panel: Panel) -> usize {
        match panel {
            Panel::Collection => self.collection_cursor,
            Panel::Deck => self.deck_cursor,
        }
    }

    fn cursor_mut(&mut self, panel: Panel) -> &mut usize {
        match panel {
            Panel::Collection => &mut self.collection_cursor,
            Panel::Deck => &mut self.deck_cursor,
        }
    }

    /// Move the active cursor left/right over its flat list, wrapping at the
    /// ends (reading order — off the end of a row continues onto the next).
    fn move_horizontal(&mut self, delta: isize, n: usize) {
        let panel = self.active;
        let c = self.cursor_mut(panel);
        *c = (*c as isize + delta).rem_euclid(n as isize) as usize;
    }

    /// Move the active cursor up/down within its column, wrapping top-to-bottom
    /// and skipping the ragged empty cell a short last row leaves.
    fn move_vertical(&mut self, dir: isize, n: usize, cols: usize) {
        let panel = self.active;
        let cur = self.cursor(panel);
        let col = cur % cols;
        let rows = n.div_ceil(cols) as isize;
        let mut row = (cur / cols) as isize;
        for _ in 0..rows {
            row = (row + dir).rem_euclid(rows);
            let idx = row as usize * cols + col;
            if idx < n {
                *self.cursor_mut(panel) = idx;
                return;
            }
        }
    }

    /// Keep the collection's scroll offset following its cursor after a move —
    /// but only when the collection is the active panel (the deck never
    /// scrolls). `n` is the collection's row count.
    fn follow_scroll_if_collection(&mut self, n: usize) {
        if self.active == Panel::Collection {
            self.follow_collection_scroll(n);
        }
    }

    /// Update `collection_scroll` so the collection cursor stays visible, using
    /// the guaranteed-minimum window ([`MIN_VISIBLE_ROWS`]) since `handle_input`
    /// has no [`Config`]; `draw` clamps this to the actual window.
    fn follow_collection_scroll(&mut self, collection_len: usize) {
        let cols = BriefcaseLayout::COLS;
        let cursor_row = self.collection_cursor / cols;
        let total_rows = collection_len.div_ceil(cols);
        self.collection_scroll =
            scroll_to_reveal(self.collection_scroll, cursor_row, MIN_VISIBLE_ROWS, total_rows);
    }

    /// Move focus onto a non-empty panel if the active one just emptied.
    fn focus_nonempty(&mut self, profile: &Profile) {
        let collection_empty = collection_rows(profile).is_empty();
        let deck_empty = deck_rows(profile).is_empty();
        self.active = resolve_focus(self.active, collection_empty, deck_empty);
    }

    /// Keep both cursors (and the collection scroll) in range as lists shrink
    /// under them — a card leaving a panel shortens that panel's list.
    fn clamp_cursors(&mut self, profile: &Profile) {
        let coll = collection_rows(profile).len();
        let deck = deck_rows(profile).len();
        self.collection_cursor = self.collection_cursor.min(coll.saturating_sub(1));
        self.deck_cursor = self.deck_cursor.min(deck.saturating_sub(1));
        self.follow_collection_scroll(coll);
    }

    /// Draw the title, the "Deck: N/10" readout over the deck panel, the two
    /// bordered/labeled panels (each visible card a card box with a `×N` count
    /// caption; the active panel's cursored card Heavy + pulsing, every other
    /// card Single + dimmed), and the controls hint.
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        const TITLE: &str = "Side Deck";
        const HINT: &str = "Tab switch  ·  Enter move  ·  Esc done";

        let layout = BriefcaseLayout::new(*config);
        draw_text_centered(frame, layout.center_x, layout.title_y, TITLE, Emphasis::Normal);

        // The deck-size readout: how close the built deck is to legal. Alert
        // while short, so an incomplete deck (the only thing blocking a match)
        // is obvious. Owned by the deck side (centered over the deck panel).
        let n = profile.deck().len();
        let (readout, readout_emphasis) = if profile.deck_is_valid() {
            (format!("Deck: {n}/{SIDE_DECK_SIZE}"), Emphasis::Normal)
        } else {
            let short = SIDE_DECK_SIZE.saturating_sub(n);
            (format!("Deck: {n}/{SIDE_DECK_SIZE} — add {short} more"), Emphasis::Alert)
        };
        draw_text_centered(frame, layout.readout_x, layout.readout_y, &readout, readout_emphasis);

        self.draw_panel(frame, &layout, Panel::Collection, "Collection", &collection_rows(profile), pulse);
        self.draw_panel(frame, &layout, Panel::Deck, "Deck", &deck_rows(profile), pulse);

        draw_text_centered(frame, layout.center_x, layout.hint_y, HINT, Emphasis::Muted);
    }

    /// Draw one bordered, labeled panel and its visible window of cards.
    fn draw_panel(
        &self,
        frame: &mut Frame,
        layout: &BriefcaseLayout,
        panel: Panel,
        label: &str,
        rows: &[(Card, usize)],
        pulse: Emphasis,
    ) {
        let border = match panel {
            Panel::Collection => layout.collection,
            Panel::Deck => layout.deck,
        };
        draw_box(frame, border, BorderWeight::Single, Emphasis::Normal);
        let (lx, ly) = match panel {
            Panel::Collection => layout.collection_label,
            Panel::Deck => layout.deck_label,
        };
        draw_text(frame, lx, ly, label, Emphasis::Normal);

        // An empty side gets an explicit cue, not a blank half-panel.
        if rows.is_empty() {
            let cx = (border.x0 + border.x1) / 2;
            draw_text_centered(frame, cx, border.y0 + 2, "(empty)", Emphasis::Muted);
            return;
        }

        let cols = layout.cols;
        let visible = layout.visible_rows;
        let total_rows = rows.len().div_ceil(cols);
        // The deck never scrolls (≤10 types fit `cols*visible`); the collection
        // window is offset by `collection_scroll`, clamped to the real window
        // so a taller terminal never leaves blank rows below.
        let offset_rows = match panel {
            Panel::Collection => self.collection_scroll.min(total_rows.saturating_sub(visible)),
            Panel::Deck => 0,
        };
        let start = offset_rows * cols;
        let cursor = self.cursor(panel);

        for vis in 0..(cols * visible) {
            let flat = start + vis;
            if flat >= rows.len() {
                break;
            }
            let (card, count) = rows[flat];
            let (x, y) = layout.card_origin(panel, vis);

            // Only the *active* panel's cursored card pops (heavy border, bright
            // pulse); every other card — including the inactive panel's
            // remembered cursor — recedes to a Single, dim (Muted) border.
            let cursored = panel == self.active && flat == cursor;
            let emphasis = if cursored { pulse } else { Emphasis::Muted };
            let mut view = CardView::new(x, y, card.label());
            view.weight = if cursored { BorderWeight::Heavy } else { BorderWeight::Single };
            view.emphasis = emphasis;
            view.draw(frame);

            // Caption row beneath the card: how many copies this row holds
            // (available in the Collection, in-deck in the Deck).
            let caption = format!("×{count}");
            draw_text_centered(frame, x + CARD_WIDTH / 2, y + CARD_HEIGHT, &caption, emphasis);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default profile's collection panel (spares: +1, -1, ±2) and deck
    /// panel (the 10 starter cards) are both non-empty — the common case.
    fn default_profile() -> Profile {
        Profile::default()
    }

    #[test]
    fn tab_switches_the_active_panel() {
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.focus_nonempty(&p);
        assert_eq!(s.active, Panel::Collection, "opens on the collection");

        assert!(matches!(s.handle_input(KeyCode::Tab, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.active, Panel::Deck);

        assert!(matches!(s.handle_input(KeyCode::BackTab, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.active, Panel::Collection);
    }

    #[test]
    fn enter_adds_in_collection_and_removes_in_deck() {
        // Re-authored from the old Backspace/`-` removal test: the direction is
        // the active panel now, so Enter in the Collection adds and Enter in the
        // Deck removes.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.focus_nonempty(&p);

        let coll_under = collection_rows(&p)[s.collection_cursor].0;
        assert!(matches!(
            s.handle_input(KeyCode::Enter, &p),
            Some(BuildOutcome::Add(c)) if c == coll_under
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char(' '), &p),
            Some(BuildOutcome::Add(c)) if c == coll_under
        ));

        s.handle_input(KeyCode::Tab, &p); // → Deck
        assert_eq!(s.active, Panel::Deck);
        let deck_under = deck_rows(&p)[s.deck_cursor].0;
        assert!(matches!(
            s.handle_input(KeyCode::Enter, &p),
            Some(BuildOutcome::Remove(c)) if c == deck_under
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char(' '), &p),
            Some(BuildOutcome::Remove(c)) if c == deck_under
        ));
    }

    #[test]
    fn a_partly_decked_card_appears_in_both_panels_with_split_counts() {
        // Start from the default (which decks one +2, owning one), then grant a
        // second +2: now owned 2, in-deck 1 → available 1. That single type must
        // show in *both* panels — Collection (available 1) and Deck (in-deck 1).
        let mut p = default_profile();
        p.grant_card(Card::Plus(2));

        let coll = collection_rows(&p);
        let deck = deck_rows(&p);
        assert_eq!(
            coll.iter().find(|(c, _)| *c == Card::Plus(2)),
            Some(&(Card::Plus(2), 1)),
            "collection shows the one available +2"
        );
        assert_eq!(
            deck.iter().find(|(c, _)| *c == Card::Plus(2)),
            Some(&(Card::Plus(2), 1)),
            "deck shows the one placed +2"
        );
    }

    #[test]
    fn initial_focus_rests_on_a_nonempty_panel() {
        // Default: collection has spares → opens on the Collection.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.focus_nonempty(&p);
        assert_eq!(s.active, Panel::Collection);

        // Empty the deck (every copy back in the collection) → focus a Deck-
        // opened builder onto the Collection.
        let mut empty_deck = default_profile();
        for c in empty_deck.deck().to_vec() {
            empty_deck.remove_from_deck(c);
        }
        assert!(deck_rows(&empty_deck).is_empty(), "sanity: the deck side is empty");
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.active = Panel::Deck;
        s.focus_nonempty(&empty_deck);
        assert_eq!(s.active, Panel::Collection);

        // The all-decked-no-spares case (collection empty, deck not) opens on
        // the Deck. The Profile API can't shrink a collection below the starter,
        // so this is covered at the decision seam directly.
        assert_eq!(resolve_focus(Panel::Collection, true, false), Panel::Deck);
        // And a fully-symmetric guard: neither side moves when both have rows.
        assert_eq!(resolve_focus(Panel::Collection, false, false), Panel::Collection);
        assert_eq!(resolve_focus(Panel::Deck, false, false), Panel::Deck);
    }

    #[test]
    fn arrows_move_and_wrap_over_the_active_grid() {
        // The 10-type deck panel at 4 columns is a multi-row grid to move over.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.handle_input(KeyCode::Tab, &p); // → Deck
        assert_eq!(s.active, Panel::Deck);
        assert_eq!(s.deck_cursor, 0);

        let cols = BriefcaseLayout::COLS;
        assert!(matches!(s.handle_input(KeyCode::Right, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.deck_cursor, 1);
        assert!(matches!(s.handle_input(KeyCode::Down, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.deck_cursor, 1 + cols); // one row down, same column
        assert!(matches!(s.handle_input(KeyCode::Up, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.deck_cursor, 1); // back up

        // wasd mirror the arrows.
        assert!(matches!(s.handle_input(KeyCode::Char('a'), &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.deck_cursor, 0);
        // Left from the start wraps to the last row (reading order).
        assert!(matches!(s.handle_input(KeyCode::Left, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.deck_cursor, deck_rows(&p).len() - 1);
    }

    #[test]
    fn vertical_move_skips_the_ragged_last_row() {
        // The 10-type deck at 4 columns has a short last row (10 = 4+4+2), so
        // the rightmost columns have no bottom-row cell. Moving down such a
        // column must skip the empty slot and wrap to the top — never land out
        // of range (which would panic the next index).
        let p = default_profile();
        let deck = deck_rows(&p);
        let cols = BriefcaseLayout::COLS;
        assert!(deck.len() % cols != 0, "test needs a ragged last row");

        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.handle_input(KeyCode::Tab, &p); // → Deck
        let ragged_col = cols - 1; // a column whose bottom-row cell is absent
        for _ in 0..ragged_col {
            s.handle_input(KeyCode::Right, &p);
        }
        assert_eq!(s.deck_cursor, ragged_col); // row 0 of the ragged column
        s.handle_input(KeyCode::Down, &p);
        assert_eq!(s.deck_cursor, cols + ragged_col); // row 1
        s.handle_input(KeyCode::Down, &p); // row-2 cell is absent → wrap to row 0
        assert_eq!(s.deck_cursor, ragged_col);
        assert!(s.deck_cursor < deck.len(), "cursor stayed in range");
    }

    #[test]
    fn scroll_reveals_the_cursor_and_clamps() {
        // A 4-row collection in a 3-row window (the 89×31 minimum): the cursor
        // on the bottom row scrolls the window down by one, and never further.
        assert_eq!(scroll_to_reveal(0, 0, 3, 4), 0, "top row needs no scroll");
        assert_eq!(scroll_to_reveal(0, 3, 3, 4), 1, "bottom row scrolls down one");
        assert_eq!(scroll_to_reveal(5, 3, 3, 4), 1, "never past the last window");
        assert_eq!(scroll_to_reveal(1, 0, 3, 4), 0, "moving up scrolls back to reveal it");
        // A window at least as tall as the grid never scrolls.
        assert_eq!(scroll_to_reveal(2, 3, 4, 4), 0);
    }

    #[test]
    fn moving_to_the_bottom_row_scrolls_the_collection_to_keep_it_visible() {
        // Exercise the real `handle_input` scroll seam: give the collection
        // enough rows to exceed the minimum window, walk the cursor down, and
        // confirm the offset tracks it while keeping the cursor within
        // MIN_VISIBLE_ROWS of the top.
        let mut p = default_profile();
        // Grant spare copies of several decked types so the collection panel
        // grows past 3 rows (> MIN_VISIBLE_ROWS * cols slots).
        for c in [
            Card::Plus(2), Card::Plus(4), Card::Minus(2), Card::Minus(4),
            Card::PlusMinus(1), Card::PlusMinus(3), Card::PlusMinus(6),
            Card::Flip(crate::card::FlipKind::TwoFour),
            Card::Flip(crate::card::FlipKind::ThreeSix),
            Card::Tiebreaker,
        ] {
            p.grant_card(c);
        }
        let coll = collection_rows(&p);
        let cols = BriefcaseLayout::COLS;
        let total_rows = coll.len().div_ceil(cols);
        assert!(total_rows > MIN_VISIBLE_ROWS, "test needs a scrolling collection");

        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.focus_nonempty(&p);
        assert_eq!(s.active, Panel::Collection);
        assert_eq!(s.collection_scroll, 0);

        // Walk down to the last row.
        while s.collection_cursor / cols < total_rows - 1 {
            s.handle_input(KeyCode::Down, &p);
        }
        let cursor_row = s.collection_cursor / cols;
        assert!(s.collection_scroll <= cursor_row, "window top not below the cursor");
        assert!(
            cursor_row < s.collection_scroll + MIN_VISIBLE_ROWS,
            "cursor stayed within the visible window"
        );
        assert!(
            s.collection_scroll <= total_rows - MIN_VISIBLE_ROWS,
            "scroll never runs past the last full window"
        );
    }

    #[test]
    fn esc_and_x_back_out_and_unknown_keys_are_ignored() {
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        assert!(matches!(s.handle_input(KeyCode::Esc, &p), Some(BuildOutcome::Back)));
        assert!(matches!(s.handle_input(KeyCode::Char('x'), &p), Some(BuildOutcome::Back)));
        assert!(s.handle_input(KeyCode::Char('z'), &p).is_none());
    }

    #[test]
    fn new_records_the_origin_for_back_routing() {
        // The origin is what the app's `Back` arm routes on: a builder opened
        // from the campaign map must return there, not to the menu (the
        // pre-existing divert bug spec 015 fixes). Both variants, so flipping
        // the field's initialization is caught.
        assert_eq!(DeckBuilderState::new(BuilderOrigin::Map).origin(), BuilderOrigin::Map);
        assert_eq!(DeckBuilderState::new(BuilderOrigin::Menu).origin(), BuilderOrigin::Menu);
    }

    #[test]
    fn min_visible_rows_never_exceeds_the_layout_floor() {
        // `MIN_VISIBLE_ROWS` is a second copy of the row floor `BriefcaseLayout`
        // guarantees at the 89×31 minimum; the collection scroll stays correct
        // only while it does not exceed the real window. This guards the one drift
        // direction the layout's own fit test can't — a floor *decrease* below 3.
        let floor = BriefcaseLayout::new(Config { num_cols: 89, num_rows: 31 }).visible_rows;
        assert!(
            MIN_VISIBLE_ROWS <= floor,
            "MIN_VISIBLE_ROWS {MIN_VISIBLE_ROWS} exceeds the layout floor {floor}"
        );
    }
}
