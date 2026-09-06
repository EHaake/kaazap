//! The deck-builder screen: a two-panel "briefcase" card album for building
//! your side deck. Both panels show **every** card type in fixed canonical
//! slots (no scrolling): the **Collection** panel (left) fills a slot solid for
//! each type you own a spare of (not placed), the **Deck** panel (right) for
//! each type you've placed, with a `Deck: N/10` readout. A type absent from a
//! panel is a faint dashed **placeholder** showing the card's dimmed face.
//! Building a deck is moving a card *copy* across: confirming on a present
//! Collection card places one in the Deck, confirming on a present Deck card
//! returns one; confirming on a placeholder does nothing. A full mode navigated
//! *to* (a [`Screen`](crate::screen), not an overlay), reached from the start
//! menu and the campaign map. Mirrors `opponent_select.rs`'s plumbing —
//! per-panel cursors + an outcome enum + `draw(frame, config, …, pulse)` and
//! one app.rs arm.
//!
//! The screen owns only the per-panel cursors and the active panel; the
//! collection and deck live in the [`Profile`], and every edit is applied
//! through the profile's own methods (the app performs it), so the
//! deck-building rules stay in one place. A card's *panel* is what conveys
//! whether it's in the deck, so the album uses three border weights (Heavy =
//! cursor, Single = present, Dashed-faint = placeholder) — the old "double = in
//! deck" weight is gone. See `specs/008-side-deck-customization` and
//! `specs/015-briefcase-deck-builder`.

use crossterm::event::KeyCode;

use crate::{
    CARD_HEIGHT, CARD_WIDTH, SIDE_DECK_SIZE,
    card::{ALL_SIDE_CARDS, Card, CardView},
    config::Config,
    frame::{BorderWeight, Drawable, Emphasis, Frame, draw_box, draw_text, draw_text_centered},
    layout::{BriefcaseLayout, Panel},
    profile::Profile,
};

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

/// The `(available, in_deck)` copy counts for `card`: *available* = owned −
/// in-deck (what the Collection panel shows) and *in-deck* (what the Deck panel
/// shows). A type the player doesn't own comes back `(0, 0)`. Folded from
/// [`Profile::collection_by_type`], so the counts stay defined in one place with
/// no `profile.rs` change; types absent from that list default to zero.
fn card_counts(profile: &Profile, card: Card) -> (usize, usize) {
    profile
        .collection_by_type()
        .into_iter()
        .find(|e| e.card == card)
        .map(|e| (e.owned.saturating_sub(e.in_deck), e.in_deck))
        .unwrap_or((0, 0))
}

/// How many copies `panel` shows for `card`: the Collection shows *available*
/// copies (owned − in-deck), the Deck shows *in-deck* copies. `0` means the type
/// is absent from that panel — drawn as a placeholder, and a no-op to confirm on.
fn panel_count(panel: Panel, card: Card, profile: &Profile) -> usize {
    let (available, in_deck) = card_counts(profile, card);
    match panel {
        Panel::Collection => available,
        Panel::Deck => in_deck,
    }
}

/// The other panel.
fn other(panel: Panel) -> Panel {
    match panel {
        Panel::Collection => Panel::Deck,
        Panel::Deck => Panel::Collection,
    }
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
    collection_cursor: usize, // slot index into ALL_SIDE_CARDS (0..15)
    deck_cursor: usize,       // slot index into ALL_SIDE_CARDS (0..15)
    origin: BuilderOrigin,    // where Back returns to (menu vs. campaign map)
}

impl Default for DeckBuilderState {
    fn default() -> Self {
        Self::new(BuilderOrigin::Menu)
    }
}

impl DeckBuilderState {
    /// A fresh builder opened from `origin` (the menu or the campaign map),
    /// which `Back` returns to. Both panels always render all 15 album slots, so
    /// focus simply defaults to the Collection.
    pub fn new(origin: BuilderOrigin) -> Self {
        Self {
            active: Panel::Collection,
            collection_cursor: 0,
            deck_cursor: 0,
            origin,
        }
    }

    /// Where this builder was opened from — the app's `Back` arm routes on it
    /// (the menu vs. the campaign map).
    pub fn origin(&self) -> BuilderOrigin {
        self.origin
    }

    /// Handle a key against the current `profile`: `Tab`/`BackTab` switch the
    /// active panel; arrows / `wasd` move that panel's cursor over the fixed
    /// album grid; `Enter`/`Space` move a copy across from a *present* slot (add
    /// from the Collection, remove from the Deck) and do nothing on a
    /// placeholder; `Esc`/`x` leave. `None` for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<BuildOutcome> {
        // Backing out is always meaningful.
        if matches!(key, KeyCode::Esc | KeyCode::Char('x')) {
            return Some(BuildOutcome::Back);
        }

        if matches!(key, KeyCode::Tab | KeyCode::BackTab) {
            // Both panels always render all 15 slots, so switching is always
            // valid — there's no empty side to avoid.
            self.active = other(self.active);
            return Some(BuildOutcome::Moved);
        }

        // The album is a fixed 15-slot grid (every ALL_SIDE_CARDS type), 4 wide.
        let n = ALL_SIDE_CARDS.len();
        let cols = BriefcaseLayout::COLS;

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
            // Move a copy across — but only from a *present* slot; a placeholder
            // (the type is absent from this panel) has nothing to move.
            KeyCode::Enter | KeyCode::Char(' ') => {
                let card = ALL_SIDE_CARDS[self.cursor(self.active)];
                if panel_count(self.active, card, profile) == 0 {
                    return None;
                }
                Some(match self.active {
                    Panel::Collection => BuildOutcome::Add(card),
                    Panel::Deck => BuildOutcome::Remove(card),
                })
            }
            _ => None,
        }
    }

    /// The given panel's cursor (a slot index into `ALL_SIDE_CARDS`).
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

    /// Move the active cursor left/right over the flat slot list, wrapping at the
    /// ends (reading order — off the end of a row continues onto the next).
    fn move_horizontal(&mut self, delta: isize, n: usize) {
        let panel = self.active;
        let c = self.cursor_mut(panel);
        *c = (*c as isize + delta).rem_euclid(n as isize) as usize;
    }

    /// Move the active cursor up/down within its column, wrapping top-to-bottom
    /// and skipping the ragged empty cell the short last row leaves (the empty
    /// 16th slot).
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

    /// Draw the title, the "Deck: N/10" readout over the deck panel, the two
    /// bordered/labeled album panels, and the controls hint.
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

        self.draw_panel(frame, &layout, Panel::Collection, "Collection", profile, pulse);
        self.draw_panel(frame, &layout, Panel::Deck, "Deck", profile, pulse);

        draw_text_centered(frame, layout.center_x, layout.hint_y, HINT, Emphasis::Muted);
    }

    /// Draw one bordered, labeled panel as a fixed album: every `ALL_SIDE_CARDS`
    /// type in its canonical slot. A type present in this panel (Collection:
    /// available > 0; Deck: in-deck > 0) is a solid card with a `×count` caption
    /// — Heavy + pulsing under the active panel's cursor, else Single + dim. A
    /// type absent is a faint dashed placeholder showing the card's dimmed face
    /// with no count (still dashed but brightened when it's the cursored slot, so
    /// the cursor shows without the slot looking owned).
    fn draw_panel(
        &self,
        frame: &mut Frame,
        layout: &BriefcaseLayout,
        panel: Panel,
        label: &str,
        profile: &Profile,
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

        let cursor = self.cursor(panel);
        for (i, &card) in ALL_SIDE_CARDS.iter().enumerate() {
            let (x, y) = layout.card_origin(panel, i);
            let count = panel_count(panel, card, profile);

            // Only the *active* panel's cursored slot pops (bright pulse); every
            // other slot recedes to dim (Muted).
            let cursored = panel == self.active && i == cursor;
            let emphasis = if cursored { pulse } else { Emphasis::Muted };

            let mut view = CardView::new(x, y, card.label());
            if count > 0 {
                // Present: a solid card — Heavy under the cursor, else Single.
                view.weight = if cursored { BorderWeight::Heavy } else { BorderWeight::Single };
                view.emphasis = emphasis;
                view.draw(frame);

                // Caption row beneath: how many copies this panel holds
                // (available in the Collection, in-deck in the Deck).
                let caption = format!("×{count}");
                draw_text_centered(frame, x + CARD_WIDTH / 2, y + CARD_HEIGHT, &caption, emphasis);
            } else {
                // Absent: a dashed-faint placeholder (the card's dimmed face, no
                // count) so you see the gap without it competing with owned cards.
                view.weight = BorderWeight::Dashed;
                view.emphasis = emphasis;
                view.draw(frame);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default profile owns the 10 starter cards (all placed in the deck)
    /// plus three spares not in it (+1, -1, ±2). So in the album: the starter
    /// types are present in the Deck / placeholders in the Collection, the
    /// spares the reverse, and unowned types (e.g. +3) are placeholders in both.
    fn default_profile() -> Profile {
        Profile::default()
    }

    #[test]
    fn tab_switches_the_active_panel() {
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        assert_eq!(s.active, Panel::Collection, "opens on the collection");

        assert!(matches!(s.handle_input(KeyCode::Tab, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.active, Panel::Deck);

        assert!(matches!(s.handle_input(KeyCode::BackTab, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.active, Panel::Collection);
    }

    #[test]
    fn enter_moves_a_present_card_across_add_in_collection_remove_in_deck() {
        // The direction is the active panel: Enter/Space on a *present*
        // Collection card adds, on a present Deck card removes.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);

        // Collection opens active; slot 0 = Plus(1), a spare the default owns
        // (available 1) → present.
        assert_eq!(ALL_SIDE_CARDS[0], Card::Plus(1));
        assert_eq!(panel_count(Panel::Collection, Card::Plus(1), &p), 1);
        assert!(matches!(
            s.handle_input(KeyCode::Enter, &p),
            Some(BuildOutcome::Add(Card::Plus(1)))
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char(' '), &p),
            Some(BuildOutcome::Add(Card::Plus(1)))
        ));

        // Switch to the Deck and land on slot 1 = Plus(2), a starter card in the
        // deck (in-deck 1) → present.
        s.handle_input(KeyCode::Tab, &p);
        assert_eq!(s.active, Panel::Deck);
        s.handle_input(KeyCode::Right, &p);
        assert_eq!(s.deck_cursor, 1);
        assert_eq!(ALL_SIDE_CARDS[1], Card::Plus(2));
        assert!(matches!(
            s.handle_input(KeyCode::Enter, &p),
            Some(BuildOutcome::Remove(Card::Plus(2)))
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char(' '), &p),
            Some(BuildOutcome::Remove(Card::Plus(2)))
        ));
    }

    #[test]
    fn enter_on_a_placeholder_is_a_noop() {
        // In the Collection, slot 1 = Plus(2) is fully in the deck (available 0),
        // so it's a placeholder — confirming does nothing.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.handle_input(KeyCode::Right, &p); // collection_cursor 0 → 1
        assert_eq!(s.collection_cursor, 1);
        assert_eq!(ALL_SIDE_CARDS[1], Card::Plus(2));
        assert_eq!(panel_count(Panel::Collection, Card::Plus(2), &p), 0, "sanity: a placeholder");
        assert!(s.handle_input(KeyCode::Enter, &p).is_none());
        assert!(s.handle_input(KeyCode::Char(' '), &p).is_none());
    }

    #[test]
    fn a_partly_decked_card_shows_in_both_panels_with_split_counts() {
        // Grant a second +2 on top of the default's one (already decked): owned
        // 2, in-deck 1 → available 1. That single type is present in *both*
        // panels — the split-by-location model (`spec.md`, "Resolved decisions").
        let mut p = default_profile();
        p.grant_card(Card::Plus(2));
        assert_eq!(panel_count(Panel::Collection, Card::Plus(2), &p), 1, "one available");
        assert_eq!(panel_count(Panel::Deck, Card::Plus(2), &p), 1, "one placed");
    }

    #[test]
    fn a_type_present_in_one_panel_is_a_placeholder_in_the_other() {
        // A fully-decked starter type (Plus(2), no spare) is present in the Deck
        // and a placeholder in the Collection; a spare-only type (Plus(1)) is the
        // reverse.
        let p = default_profile();
        assert_eq!(panel_count(Panel::Deck, Card::Plus(2), &p), 1);
        assert_eq!(panel_count(Panel::Collection, Card::Plus(2), &p), 0);
        assert_eq!(panel_count(Panel::Collection, Card::Plus(1), &p), 1);
        assert_eq!(panel_count(Panel::Deck, Card::Plus(1), &p), 0);
    }

    #[test]
    fn an_unowned_type_is_a_placeholder_in_both_panels() {
        // Plus(3) isn't in the default collection at all (owned 0), so it's a
        // placeholder on both sides — the "see your gaps" album behavior.
        let p = default_profile();
        assert_eq!(p.owned_count(Card::Plus(3)), 0, "sanity: unowned");
        assert_eq!(panel_count(Panel::Collection, Card::Plus(3), &p), 0);
        assert_eq!(panel_count(Panel::Deck, Card::Plus(3), &p), 0);
    }

    #[test]
    fn arrows_move_and_wrap_over_the_fixed_grid() {
        // The album is a fixed 15-slot grid, 4 wide, regardless of panel or
        // profile — every ALL_SIDE_CARDS type has a slot.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        let cols = BriefcaseLayout::COLS;
        assert_eq!(s.collection_cursor, 0);

        assert!(matches!(s.handle_input(KeyCode::Right, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 1);
        assert!(matches!(s.handle_input(KeyCode::Down, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 1 + cols); // one row down, same column
        assert!(matches!(s.handle_input(KeyCode::Up, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 1); // back up

        // wasd mirror the arrows.
        assert!(matches!(s.handle_input(KeyCode::Char('a'), &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 0);
        // Left from the first slot wraps to the last card (reading order).
        assert!(matches!(s.handle_input(KeyCode::Left, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, ALL_SIDE_CARDS.len() - 1);
    }

    #[test]
    fn vertical_move_skips_the_empty_sixteenth_slot() {
        // 15 cards at 4 columns leave a ragged last row (slots 12,13,14; the
        // 16th, slot 15, is empty). Moving down column 3 must skip that empty
        // slot and wrap to the top — never land on slot 15 (which has no card,
        // and would be out of ALL_SIDE_CARDS range).
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        let cols = BriefcaseLayout::COLS; // 4
        let last_col = cols - 1;
        for _ in 0..last_col {
            s.handle_input(KeyCode::Right, &p);
        }
        assert_eq!(s.collection_cursor, last_col); // row 0, col 3
        s.handle_input(KeyCode::Down, &p);
        assert_eq!(s.collection_cursor, cols + last_col); // row 1, col 3 = 7
        s.handle_input(KeyCode::Down, &p);
        assert_eq!(s.collection_cursor, 2 * cols + last_col); // row 2, col 3 = 11
        s.handle_input(KeyCode::Down, &p); // row 3 col 3 = slot 15 is empty → wrap
        assert_eq!(s.collection_cursor, last_col); // back to row 0, col 3
        assert!(s.collection_cursor < ALL_SIDE_CARDS.len(), "cursor stays on a real card");
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
}
