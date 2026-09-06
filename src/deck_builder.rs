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
//! deck-building rules stay in one place. The selection cursor only ever rests
//! on a *present* card: movement skips placeholders, and a panel with no present
//! cards isn't focusable (`spec.md`, "The cursor only lands on present cards").
//! A card's *panel* is what conveys whether it's in the deck, so the album uses
//! three border weights (Heavy = cursor, Single = present, Dashed-faint =
//! placeholder) — the old "double = in deck" weight is gone. See
//! `specs/008-side-deck-customization` and `specs/015-briefcase-deck-builder`.

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

/// Whether `slot` (an index into [`ALL_SIDE_CARDS`]) is a *present* card in
/// `panel` — Collection: available > 0; Deck: in-deck > 0. Placeholders
/// (count 0) aren't present, so the cursor never rests on one and movement
/// skips them.
fn is_present(panel: Panel, slot: usize, profile: &Profile) -> bool {
    panel_count(panel, ALL_SIDE_CARDS[slot], profile) > 0
}

/// The first present slot in `panel` (canonical order), or `None` when the panel
/// is all placeholders — nothing to select there, so it isn't focusable. The
/// focus seam the always-15-slots album draw would otherwise lack: initial
/// focus, `Tab`, the cursor re-validation, and the draw display-cursor all pin
/// onto a present slot through this.
fn first_present(panel: Panel, profile: &Profile) -> Option<usize> {
    (0..ALL_SIDE_CARDS.len()).find(|&slot| is_present(panel, slot, profile))
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
    /// which `Back` returns to. Focus defaults to the Collection with the cursor
    /// at slot 0; since `new` has no profile, the actual *present* slot is
    /// resolved lazily — the draw display-cursor and the `handle_input`
    /// re-validation both pin the cursor onto a present card (or hand focus to
    /// the other panel when the Collection has none).
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

    /// Handle a key against the current `profile`: `Tab`/`BackTab` switch to the
    /// other panel *if it has present cards* (an all-placeholder panel isn't
    /// focusable); arrows / `wasd` move the active cursor to the next *present*
    /// slot in that direction, skipping placeholders and wrapping; `Enter`/`Space`
    /// move a copy across (add from the Collection, remove from the Deck);
    /// `Esc`/`x` leave. `None` for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<BuildOutcome> {
        // Backing out is always meaningful — even when nothing is selectable.
        if matches!(key, KeyCode::Esc | KeyCode::Char('x')) {
            return Some(BuildOutcome::Back);
        }

        // Start from a valid selection. The app may have applied an add/remove
        // since the last key, emptying the cursored slot — or the whole active
        // panel — so pin focus and the cursor back onto present slots first.
        self.revalidate(profile);

        match key {
            KeyCode::Tab | KeyCode::BackTab => {
                // Switch only to a panel that has present cards; an
                // all-placeholder panel has nothing to select or move.
                if first_present(other(self.active), profile).is_some() {
                    self.active = other(self.active);
                }
                Some(BuildOutcome::Moved)
            }
            KeyCode::Up | KeyCode::Char('w') => {
                self.move_vertical(-1, profile);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_vertical(1, profile);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Left | KeyCode::Char('a') => {
                self.move_horizontal(-1, profile);
                Some(BuildOutcome::Moved)
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.move_horizontal(1, profile);
                Some(BuildOutcome::Moved)
            }
            // Re-validation keeps the cursor on a present slot, so this always
            // moves a real copy across. The count==0 guard is defensive (both
            // panels empty — unreachable in normal play, the starter always owns
            // cards) and never fires via the cursor.
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

    /// Move the active cursor to the previous/next *present* slot in reading
    /// order, skipping placeholders and wrapping at the ends. Stays put when the
    /// panel has no other present slot (a single present card, or none).
    fn move_horizontal(&mut self, delta: isize, profile: &Profile) {
        let panel = self.active;
        let n = ALL_SIDE_CARDS.len();
        let mut slot = self.cursor(panel) as isize;
        for _ in 0..n {
            slot = (slot + delta).rem_euclid(n as isize);
            if is_present(panel, slot as usize, profile) {
                *self.cursor_mut(panel) = slot as usize;
                return;
            }
        }
    }

    /// Move the active cursor up/down within its column to the next *present*
    /// slot, wrapping top-to-bottom and skipping both placeholders and the ragged
    /// empty 16th cell the short last row leaves. Stays put when the column has
    /// no other present slot.
    fn move_vertical(&mut self, dir: isize, profile: &Profile) {
        let panel = self.active;
        let n = ALL_SIDE_CARDS.len();
        let cols = BriefcaseLayout::COLS;
        let cur = self.cursor(panel);
        let col = cur % cols;
        let rows = n.div_ceil(cols) as isize;
        let mut row = (cur / cols) as isize;
        for _ in 0..rows {
            row = (row + dir).rem_euclid(rows);
            let idx = row as usize * cols + col;
            if idx < n && is_present(panel, idx, profile) {
                *self.cursor_mut(panel) = idx;
                return;
            }
        }
    }

    /// The slot `draw` should highlight for `panel`, and the slot re-validation
    /// pins the cursor onto: the stored cursor when it's on a present slot, else
    /// the first present slot, else `None` (an all-placeholder panel shows no
    /// cursor). This is what keeps a placeholder from ever being highlighted —
    /// even the transient frame right after an add empties the cursored slot,
    /// before the next input re-validates.
    fn display_cursor(&self, panel: Panel, profile: &Profile) -> Option<usize> {
        let stored = self.cursor(panel);
        if is_present(panel, stored, profile) {
            Some(stored)
        } else {
            first_present(panel, profile)
        }
    }

    /// Pin focus and the active cursor onto present slots before handling a key.
    /// If the active panel has no present cards but the other does, hand focus
    /// over (an add/remove can empty the active panel); then snap the active
    /// cursor onto a present slot. A panel with nothing present isn't focusable
    /// and keeps no cursor.
    fn revalidate(&mut self, profile: &Profile) {
        if first_present(self.active, profile).is_none()
            && first_present(other(self.active), profile).is_some()
        {
            self.active = other(self.active);
        }
        if let Some(slot) = self.display_cursor(self.active, profile) {
            *self.cursor_mut(self.active) = slot;
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
    /// with no count. Only the active panel shows a cursor, and only ever on a
    /// present slot (the display-cursor fallback), so a placeholder is never
    /// Heavy/pulse.
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

        // Only the active panel shows a cursor, and the display-cursor keeps it
        // on a present slot even when the stored cursor drifted onto a
        // placeholder (the transient frame after an add empties a slot).
        let display_cursor = if panel == self.active {
            self.display_cursor(panel, profile)
        } else {
            None
        };
        for (i, &card) in ALL_SIDE_CARDS.iter().enumerate() {
            let (x, y) = layout.card_origin(panel, i);
            let count = panel_count(panel, card, profile);

            // The cursored slot pops (bright pulse); every other slot recedes to
            // dim (Muted). A placeholder is never the cursor, so it never pops.
            let cursored = display_cursor == Some(i);
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
                // Never cursored, so always Muted — never Heavy/pulse.
                view.weight = BorderWeight::Dashed;
                view.emphasis = Emphasis::Muted;
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

    /// Build a profile with an explicit collection and deck. `Profile`'s fields
    /// are private and its `profile_with` test helper isn't reachable here, so
    /// go through the public serde derive (as `profile.rs`'s own tests do) to
    /// stage exact placeholder/present layouts the starter can't produce.
    fn profile_from(collection: Vec<Card>, deck: Vec<Card>) -> Profile {
        let doc = serde_json::json!({
            "version": 1,
            "collection": collection,
            "deck": deck,
        });
        serde_json::from_value(doc).expect("a valid test profile")
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

        // Switch to the Deck. Its remembered cursor (slot 0 = Plus(1)) is a
        // placeholder there, so the next key re-validates onto the first present
        // Deck slot — slot 1 = Plus(2), a decked starter — and Enter removes it.
        // The cursor can never be steered onto a placeholder to begin with.
        s.handle_input(KeyCode::Tab, &p);
        assert_eq!(s.active, Panel::Deck);
        assert_eq!(ALL_SIDE_CARDS[1], Card::Plus(2));
        assert!(matches!(
            s.handle_input(KeyCode::Enter, &p),
            Some(BuildOutcome::Remove(Card::Plus(2)))
        ));
        assert_eq!(s.deck_cursor, 1, "re-validation snapped onto the first present Deck slot");
        assert!(matches!(
            s.handle_input(KeyCode::Char(' '), &p),
            Some(BuildOutcome::Remove(Card::Plus(2)))
        ));
    }

    #[test]
    fn both_panels_all_placeholders_is_inert_and_never_panics() {
        // Unreachable in normal play (the starter always owns cards), but the
        // code must never index an empty set or panic: with nothing owned, both
        // panels are all placeholders — no cursor shows, the defensive Enter
        // guard makes confirming a no-op, and movement/Tab change nothing.
        let p = profile_from(vec![], vec![]);
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        assert!(first_present(Panel::Collection, &p).is_none());
        assert!(first_present(Panel::Deck, &p).is_none());
        assert_eq!(s.display_cursor(Panel::Collection, &p), None, "no cursor when nothing is present");
        assert_eq!(s.display_cursor(Panel::Deck, &p), None);

        // Enter/Space hit the defensive placeholder guard → no-op.
        assert!(s.handle_input(KeyCode::Enter, &p).is_none());
        assert!(s.handle_input(KeyCode::Char(' '), &p).is_none());
        // Movement and Tab don't panic and change nothing.
        assert!(matches!(s.handle_input(KeyCode::Right, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 0);
        s.handle_input(KeyCode::Tab, &p);
        assert_eq!(s.active, Panel::Collection, "no focusable panel to switch to");
        // Backing out still works.
        assert!(matches!(s.handle_input(KeyCode::Esc, &p), Some(BuildOutcome::Back)));
    }

    #[test]
    fn initial_focus_lands_on_a_present_card() {
        // A Collection whose slot 0 (Plus1) is absent but slot 4 (Minus1) is
        // present: `new` defaults the stored cursor to slot 0, yet the drawn
        // cursor (display-cursor) and the first re-validation both resolve onto
        // the first present slot — never a placeholder.
        let p = profile_from(vec![Card::Minus(1)], vec![]);
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        assert_eq!(s.active, Panel::Collection, "opens on the collection");
        assert_eq!(panel_count(Panel::Collection, ALL_SIDE_CARDS[0], &p), 0, "slot 0 is a placeholder");

        // The frame highlights the first present slot even before any key.
        assert_eq!(s.display_cursor(Panel::Collection, &p), Some(4));
        // And the stored cursor snaps there as soon as input is handled.
        s.revalidate(&p);
        assert_eq!(s.collection_cursor, 4);
        assert!(is_present(Panel::Collection, s.collection_cursor, &p));
    }

    #[test]
    fn tab_will_not_focus_an_all_placeholder_panel() {
        // An empty deck: the Deck panel is all placeholders (not focusable),
        // while the Collection has present cards. Tab/BackTab keep focus on the
        // Collection — there is nothing to select in the Deck.
        let p = profile_from(vec![Card::Plus(1), Card::Minus(1)], vec![]);
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        assert_eq!(s.active, Panel::Collection);
        assert!(first_present(Panel::Deck, &p).is_none(), "empty deck → all placeholders");
        assert!(first_present(Panel::Collection, &p).is_some());

        s.handle_input(KeyCode::Tab, &p);
        assert_eq!(s.active, Panel::Collection, "Tab does not switch to an all-placeholder Deck");
        s.handle_input(KeyCode::BackTab, &p);
        assert_eq!(s.active, Panel::Collection, "BackTab does not switch either");
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
    fn arrows_skip_placeholders_and_wrap_to_present_slots() {
        // The default Collection is present only at slots 0 (Plus1), 4 (Minus1),
        // and 9 (PlusMinus2) — every slot between them is a placeholder.
        // Left/Right visit present slots in reading order, skipping placeholders
        // and wrapping; the cursor never lands on a placeholder.
        let p = default_profile();
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        for slot in [0usize, 4, 9] {
            assert_eq!(panel_count(Panel::Collection, ALL_SIDE_CARDS[slot], &p), 1);
        }
        assert_eq!(s.collection_cursor, 0);

        assert!(matches!(s.handle_input(KeyCode::Right, &p), Some(BuildOutcome::Moved)));
        assert_eq!(s.collection_cursor, 4, "Right skips placeholders 1..3 to slot 4");
        s.handle_input(KeyCode::Right, &p);
        assert_eq!(s.collection_cursor, 9, "Right skips placeholders 5..8 to slot 9");
        s.handle_input(KeyCode::Right, &p);
        assert_eq!(s.collection_cursor, 0, "Right wraps past 10..14 to the first present slot");
        s.handle_input(KeyCode::Left, &p);
        assert_eq!(s.collection_cursor, 9, "Left wraps backward to the last present slot");

        // wasd mirror the arrows: 'd' Right, 'a' Left.
        s.handle_input(KeyCode::Char('d'), &p);
        assert_eq!(s.collection_cursor, 0, "d (Right) wraps 9 → 0");
        s.handle_input(KeyCode::Char('a'), &p);
        assert_eq!(s.collection_cursor, 9, "a (Left) wraps 0 → 9");
    }

    #[test]
    fn a_one_present_card_panel_does_not_move_on_any_arrow() {
        // The Collection owns exactly one type (Plus1 → slot 0 present); every
        // other slot is a placeholder. No arrow (or wasd) can leave the single
        // present slot.
        let p = profile_from(vec![Card::Plus(1)], vec![]);
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        s.revalidate(&p);
        assert_eq!(s.collection_cursor, 0);
        assert_eq!(first_present(Panel::Collection, &p), Some(0), "the sole present card");

        for key in [
            KeyCode::Right,
            KeyCode::Left,
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Char('w'),
            KeyCode::Char('a'),
            KeyCode::Char('s'),
            KeyCode::Char('d'),
        ] {
            assert!(matches!(s.handle_input(key, &p), Some(BuildOutcome::Moved)));
            assert_eq!(s.collection_cursor, 0, "the sole present card is a fixed point under {key:?}");
        }
    }

    #[test]
    fn vertical_move_skips_placeholders_and_the_empty_sixteenth_slot() {
        // A Collection present only at slots 3 (Plus4) and 11 (PlusMinus6), both
        // in the last column (col 3: slots 3, 7, 11, and the empty 16th slot 15).
        // Slot 7 (Minus4) is a placeholder. Vertical movement in that column
        // skips the placeholder and the empty 16th slot, cycling 3 ↔ 11 — never
        // landing on slot 15 (out of ALL_SIDE_CARDS range).
        let p = profile_from(vec![Card::Plus(4), Card::PlusMinus(6)], vec![]);
        let mut s = DeckBuilderState::new(BuilderOrigin::Menu);
        let cols = BriefcaseLayout::COLS; // 4
        assert_eq!(3 % cols, cols - 1, "slot 3 is in the last column");
        assert_eq!(panel_count(Panel::Collection, ALL_SIDE_CARDS[7], &p), 0, "slot 7 is a placeholder");

        // Focus resolves onto the first present slot (3) before any move.
        s.revalidate(&p);
        assert_eq!(s.collection_cursor, 3);

        s.handle_input(KeyCode::Down, &p);
        assert_eq!(s.collection_cursor, 11, "Down skips placeholder slot 7 to slot 11");
        s.handle_input(KeyCode::Down, &p);
        assert_eq!(s.collection_cursor, 3, "Down wraps past the empty 16th slot back to 3");
        assert!(s.collection_cursor < ALL_SIDE_CARDS.len(), "never lands on the empty slot 15");

        // Up mirrors: 3 → 11 (wrapping up past the empty slot), 11 → 3.
        s.handle_input(KeyCode::Up, &p);
        assert_eq!(s.collection_cursor, 11);
        s.handle_input(KeyCode::Char('w'), &p); // 'w' = Up
        assert_eq!(s.collection_cursor, 3);
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
