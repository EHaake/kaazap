//! The shop screen: browse the whole card range, grouped by the region that
//! opens it, and buy from the groups you've reached (spec 025).
//! A full mode navigated *to* (a [`Screen`](crate::screen)), reached from the
//! campaign map — the between-worlds outfitter, beside the campaign depth that
//! gates its stock. Mirrors `opponent_select.rs`/`deck_builder.rs`: a cursor + an
//! owned outcome enum + `draw(frame, config, profile, pulse)` + one app arm. The
//! screen owns only the cursor; the tiers and prices come from [`economy`], the
//! balance and collection from the [`Profile`], and a purchase is applied through
//! the profile's own `try_purchase` (the app performs it). See `specs/012-economy`
//! and `specs/025-outfitter-locked-cards`.

use crossterm::event::KeyCode;

use crate::{
    card::{ALL_SIDE_CARDS, Card},
    config::Config,
    economy::{self, RegionTier},
    frame::{Emphasis, Frame, draw_text, draw_text_centered},
    profile::Profile,
};

/// The three groups, in list order.
const TIERS: [RegionTier; 3] = [RegionTier::Outer, RegionTier::Mid, RegionTier::Core];

/// Every collectible card in list order: grouped by `economy::card_tier`
/// (Outer, Mid, Core), `ALL_SIDE_CARDS` order within a group.
fn listing() -> Vec<Card> {
    TIERS
        .iter()
        .flat_map(|&tier| ALL_SIDE_CARDS.iter().copied().filter(move |&c| economy::card_tier(c) == tier))
        .collect()
}

/// How many cards at the head of `listing()` are buyable at `depth`. The
/// unlocked cards are always a prefix, because groups are in tier order
/// (plan tension §1).
fn unlocked_count(depth: RegionTier) -> usize {
    listing().into_iter().filter(|&c| economy::card_tier(c) <= depth).count()
}

/// A group's heading: the bare region name once reached, else
/// `"<Region>  ·  reach the <Region> to unlock"`.
fn heading(tier: RegionTier, depth: RegionTier) -> String {
    let name = tier.region_name();
    if tier <= depth {
        name.to_string()
    } else {
        format!("{name}  ·  reach the {name} to unlock")
    }
}

/// One card row — extracted so the fit test measures the string the draw uses.
/// A fixed-width marker (cursored or blank) keeps a row's length unchanged as
/// the cursor moves.
fn row_text(card: Card, cursored: bool, owned: usize) -> String {
    let marker = if cursored { "▸" } else { " " };
    format!("{marker}  {:<4}   {:>3} cr   owned ×{owned}", card.label(), economy::card_price(card))
}

/// Rows the list occupies: each group is a blank row, its heading, its cards.
const LIST_ROWS: usize = ALL_SIDE_CARDS.len() + 2 * TIERS.len(); // 21

/// `(title_y, list_top, hint_y)`: title, balance, the list (which opens with
/// its first blank row), one blank, the hint — 25 rows, centered vertically.
/// The list is single-spaced within a group so the whole range fits the
/// minimum terminal (see `the_full_list_fits_the_minimum_terminal`).
fn anchors(num_rows: usize) -> (usize, usize, usize) {
    let block = 2 + LIST_ROWS + 2; // title+balance, the list, gap, hint
    let top = num_rows.saturating_sub(block) / 2;
    let list_top = top + 2;
    let hint_y = list_top + LIST_ROWS + 1;
    (top, list_top, hint_y)
}

/// The list's shared left column (plan tension §2): centered on `center_x` by
/// the widest line the list can draw — the locked heading of each tier that
/// can lock (Mid, Core; Outer is always reached, so its locked form is never
/// drawn and is excluded) at its 3-column indent, and every row with a
/// two-digit owned count.
fn list_left(center_x: usize) -> usize {
    let headings = [RegionTier::Mid, RegionTier::Core]
        .into_iter()
        .map(|tier| 3 + heading(tier, RegionTier::Outer).chars().count());
    let rows = ALL_SIDE_CARDS.iter().map(|&c| row_text(c, true, 99).chars().count());
    let widest = headings.chain(rows).max().unwrap_or(0);
    center_x.saturating_sub(widest / 2)
}

/// The result of a key on the shop: the cursor moved, the highlighted card
/// should be bought, or the player is done. The app performs the purchase
/// through the [`Profile`] (which decides affordability) and plays the SFX.
#[derive(Debug, Copy, Clone)]
pub enum ShopOutcome {
    Moved,
    Buy(Card),
    Back,
}

#[derive(Debug)]
pub struct ShopState {
    cursor: usize, // index into `listing()`, always `< unlocked_count`
}

impl Default for ShopState {
    fn default() -> Self {
        Self::new()
    }
}

impl ShopState {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    /// Handle a key against `profile`: Up/Down (and `w`/`s`) move over the
    /// unlocked cards only, wrapping; locked rows and headings are never
    /// selectable. Enter/Space buy the highlighted card; Esc/`x` leave. `None`
    /// for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<ShopOutcome> {
        let n = unlocked_count(economy::deepest_reached(profile.campaign()));
        // The unlocked prefix is never empty in practice (the Outer tier is
        // always reached), but the list is data-driven, so guard rather than
        // index in.
        if n == 0 {
            return matches!(key, KeyCode::Esc | KeyCode::Char('x')).then_some(ShopOutcome::Back);
        }
        self.cursor = self.cursor.min(n - 1);

        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                self.move_by(-1, n);
                Some(ShopOutcome::Moved)
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.move_by(1, n);
                Some(ShopOutcome::Moved)
            }
            KeyCode::Enter | KeyCode::Char(' ') => Some(ShopOutcome::Buy(listing()[self.cursor])),
            KeyCode::Esc | KeyCode::Char('x') => Some(ShopOutcome::Back),
            _ => None,
        }
    }

    /// Move the cursor by `delta` over the unlocked cards, wrapping at the ends.
    fn move_by(&mut self, delta: isize, n: usize) {
        self.cursor = (self.cursor as isize + delta).rem_euclid(n as isize) as usize;
    }

    /// Draw the title, the credit balance (and the spendable amount left after
    /// the ante reserve), the whole card range in three region groups — each a
    /// blank row, its heading (bare once reached, else naming the region that
    /// unlocks it), and one row per card (`label · price · owned`; the cursored
    /// row pulsing, locked and unaffordable rows dimmed) — and the controls hint.
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        const TITLE: &str = "Outfitter";
        const HINT: &str = "↑/↓ choose  ·  Enter buy  ·  Esc back";

        let credits = profile.credits();

        let center_x = config.num_cols / 2;
        let (title_y, list_top, hint_y) = anchors(config.num_rows);
        draw_text_centered(frame, center_x, title_y, TITLE, Emphasis::Normal);

        // The balance row also shows the *spendable* amount — credits minus the
        // ante reserve `Profile::can_afford` holds back — so a card dimmed while
        // `credits ≥ price` is explicable rather than mysterious (spec 021).
        let spendable = credits.saturating_sub(economy::cheapest_floor(profile.campaign()));
        let balance = format!("Credits: ◈ {credits}  ·  spendable ◈ {spendable}");
        draw_text_centered(frame, center_x, title_y + 1, &balance, Emphasis::Strong);

        // Headings and rows share one left column so the list reads as a block
        // and a row never shifts as its owned count gains a digit.
        let depth = economy::deepest_reached(profile.campaign());
        let left = list_left(center_x);
        let cards = listing();
        let mut y = list_top;
        let mut i = 0; // index into `cards` — counts card rows only
        for tier in TIERS {
            y += 1; // the blank row above each heading
            // Headings stay Normal even when locked, so the unlock sentence
            // reads at full weight while the rows under it recede.
            draw_text(frame, left + 3, y, &heading(tier, depth), Emphasis::Normal);
            y += 1;
            for &card in cards.iter().filter(|&&c| economy::card_tier(c) == tier) {
                let locked = economy::card_tier(card) > depth;
                let cursored = !locked && i == self.cursor;
                // Affordability is the profile's rule (price + the ante reserve),
                // so the dimming and `try_purchase`'s refusal never disagree.
                let emphasis = if cursored {
                    pulse
                } else if locked || !profile.can_afford(economy::card_price(card)) {
                    Emphasis::Muted
                } else {
                    Emphasis::Normal
                };
                draw_text(frame, left, y, &row_text(card, cursored, profile.owned_count(card)), emphasis);
                y += 1;
                i += 1;
            }
        }

        draw_text_centered(frame, center_x, hint_y, HINT, Emphasis::Muted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A profile whose campaign has reached the Mid Rim (Karrus unlocked), so
    // the Outer and Mid groups are open and the Core is locked.
    fn mid_profile() -> Profile {
        let mut p = Profile::default();
        for (planet, opponent) in [("cinder", "greeb"), ("scree", "dax")] {
            p.campaign_mut().mark_beaten(planet, opponent);
        }
        p
    }

    // A profile whose campaign has reached the Core, so the whole 15-card pool
    // is available and the shop has a full list to navigate.
    fn core_profile() -> Profile {
        let mut p = Profile::default();
        for (planet, opponent) in [
            ("cinder", "greeb"),
            ("scree", "dax"),
            ("ashfall", "vessa"),
            ("karrus", "nima"),
            ("drift", "toran"),
            ("the-anvil", "brakka"),
            ("the-anvil", "kesh"),
        ] {
            p.campaign_mut().mark_beaten(planet, opponent);
        }
        p
    }

    fn depth_of(p: &Profile) -> RegionTier {
        economy::deepest_reached(p.campaign())
    }

    #[test]
    fn the_listing_groups_every_card_by_tier() {
        let l = listing();
        assert_eq!(l.len(), 15);
        assert!(l.windows(2).all(|w| economy::card_tier(w[0]) <= economy::card_tier(w[1])));
        let size = |t| l.iter().filter(|&&c| economy::card_tier(c) == t).count();
        assert_eq!((size(RegionTier::Outer), size(RegionTier::Mid), size(RegionTier::Core)), (7, 6, 2));
        assert_eq!(
            l[..7],
            [
                Card::Plus(1),
                Card::Plus(2),
                Card::Plus(3),
                Card::Minus(1),
                Card::Minus(2),
                Card::Minus(3),
                Card::PlusMinus(1),
            ]
        );
        assert_eq!(l[13..], [Card::PlusMinus(6), Card::Tiebreaker]);
        // A permutation of the universe: same length, each card exactly once.
        for c in ALL_SIDE_CARDS {
            assert_eq!(l.iter().filter(|&&x| x == c).count(), 1, "{c:?}");
        }
    }

    #[test]
    fn the_unlocked_prefix_is_the_available_pool_at_every_depth() {
        for (p, expected) in [(Profile::default(), 7), (mid_profile(), 13), (core_profile(), 15)] {
            let depth = depth_of(&p);
            let n = unlocked_count(depth);
            assert_eq!(n, expected, "{depth:?}");

            let sorted = |cards: &[Card]| {
                let mut v: Vec<String> = cards.iter().map(|c| format!("{c:?}")).collect();
                v.sort();
                v
            };
            let l = listing();
            assert_eq!(sorted(&l[..n]), sorted(&economy::available_pool(p.campaign())), "{depth:?}");
            assert!(l[n..].iter().all(|&c| economy::card_tier(c) > depth), "{depth:?}");
        }
    }

    #[test]
    fn headings_name_the_region_and_lock_until_reached() {
        use RegionTier::*;
        assert_eq!(heading(Outer, Outer), "Outer Rim");
        assert_eq!(heading(Mid, Outer), "Mid Rim  ·  reach the Mid Rim to unlock");
        assert_eq!(heading(Core, Outer), "Core  ·  reach the Core to unlock");

        assert_eq!(heading(Outer, Mid), "Outer Rim");
        assert_eq!(heading(Mid, Mid), "Mid Rim");
        assert_eq!(heading(Core, Mid), "Core  ·  reach the Core to unlock");

        assert_eq!(heading(Outer, Core), "Outer Rim");
        assert_eq!(heading(Mid, Core), "Mid Rim");
        assert_eq!(heading(Core, Core), "Core");
    }

    #[test]
    fn arrows_wrap_over_the_unlocked_cards_only() {
        let p = Profile::default();
        let mut s = ShopState::new();
        assert_eq!(s.cursor, 0);
        assert_eq!(listing()[0], Card::Plus(1));

        assert!(matches!(s.handle_input(KeyCode::Up, &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, 6); // up from the top wraps to the last Outer card
        assert!(matches!(s.handle_input(KeyCode::Down, &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, 0);
        // w/s mirror up/down.
        assert!(matches!(s.handle_input(KeyCode::Char('s'), &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, 1);
        for _ in 0..20 {
            s.handle_input(KeyCode::Down, &p);
            assert!(s.cursor < 7, "cursor {} reached a locked row", s.cursor);
        }

        // With everything unlocked the wrap spans the whole list, as before.
        let core = core_profile();
        let mut s = ShopState::new();
        s.handle_input(KeyCode::Up, &core);
        assert_eq!(s.cursor, 14);
    }

    #[test]
    fn enter_and_space_buy_the_highlighted_card() {
        // Fresh profile: Up wraps to the last Outer card, not into the Mid group.
        let p = Profile::default();
        let mut s = ShopState::new();
        s.handle_input(KeyCode::Up, &p);
        assert!(matches!(s.handle_input(KeyCode::Enter, &p), Some(ShopOutcome::Buy(Card::PlusMinus(1)))));

        // Seven Downs walk the Outer group once: the 1st wraps to `+1`, the 7th
        // lands back where it started, and no buy is ever a deeper card.
        let mut bought = Vec::new();
        for _ in 0..7 {
            s.handle_input(KeyCode::Down, &p);
            match s.handle_input(KeyCode::Enter, &p) {
                Some(ShopOutcome::Buy(c)) => {
                    assert_eq!(economy::card_tier(c), RegionTier::Outer, "{c:?}");
                    bought.push(c);
                }
                other => panic!("expected a buy, got {other:?}"),
            }
        }
        assert_eq!(bought[0], Card::Plus(1));
        assert_eq!(bought[6], Card::PlusMinus(1));
        assert_eq!(bought, listing()[..7]);

        let core = core_profile();
        let mut s = ShopState::new();
        s.handle_input(KeyCode::Down, &core); // now on listing()[1]
        let under = listing()[1];
        assert!(matches!(s.handle_input(KeyCode::Enter, &core), Some(ShopOutcome::Buy(c)) if c == under));
        assert!(matches!(s.handle_input(KeyCode::Char(' '), &core), Some(ShopOutcome::Buy(c)) if c == under));
    }

    #[test]
    fn esc_and_x_back_out_and_unknown_keys_are_ignored() {
        let p = core_profile();
        let mut s = ShopState::new();
        assert!(matches!(s.handle_input(KeyCode::Esc, &p), Some(ShopOutcome::Back)));
        assert!(matches!(s.handle_input(KeyCode::Char('x'), &p), Some(ShopOutcome::Back)));
        assert!(s.handle_input(KeyCode::Char('z'), &p).is_none());
    }

    #[test]
    fn a_reset_map_relocks_groups_but_keeps_owned_counts() {
        let mut p = core_profile();
        p.grant_card(Card::Plus(4));
        p.grant_card(Card::Tiebreaker);
        p.reset_campaign_run();

        let depth = depth_of(&p);
        assert_eq!(unlocked_count(depth), 7);
        assert_eq!(p.owned_count(Card::Plus(4)), 1);
        assert_eq!(p.owned_count(Card::Tiebreaker), 1);
        assert_eq!(heading(RegionTier::Mid, depth), "Mid Rim  ·  reach the Mid Rim to unlock");
        assert_eq!(heading(RegionTier::Core, depth), "Core  ·  reach the Core to unlock");
    }

    #[test]
    fn the_full_list_fits_the_minimum_terminal() {
        // A new card type grows the list — trip here so the fit is re-checked.
        assert_eq!(LIST_ROWS, 21);
        let (cols, rows) = Config::min_size();

        // Vertically: title, balance, three groups (blank, heading, cards), a
        // gap and the hint all land within the minimum height.
        let (title_y, list_top, hint_y) = anchors(rows);
        assert!(list_top > title_y + 1, "the list must clear the title and balance rows");
        assert!(hint_y < rows, "the hint (row {hint_y}) clips the {rows}-row minimum");
        assert!(list_top + LIST_ROWS < hint_y, "the list runs into the hint");

        // The balance row at an implausibly large balance fits centered.
        let balance = format!("Credits: ◈ {}  ·  spendable ◈ {}", 99_999u32, 99_989u32);
        let len = balance.chars().count();
        assert!(
            (cols / 2).saturating_sub(len / 2) + len <= cols,
            "balance row {balance:?} ({len} cols) overflows {cols} columns"
        );

        // Horizontally: every row (a 2-digit owned count) and the locked heading
        // of each tier that can lock fit from the shared left column.
        let left = list_left(cols / 2);
        for &card in &ALL_SIDE_CARDS {
            let row = row_text(card, true, 99);
            let len = row.chars().count();
            assert!(left + len <= cols, "shop row {row:?} ({len} cols) overflows {cols} columns");
        }
        for tier in [RegionTier::Mid, RegionTier::Core] {
            let h = heading(tier, RegionTier::Outer);
            let len = h.chars().count();
            assert!(left + 3 + len <= cols, "heading {h:?} ({len} cols) overflows {cols} columns");
        }
    }
}
