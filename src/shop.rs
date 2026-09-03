//! The shop screen: spend credits on cards from the current depth-gated pool.
//! A full mode navigated *to* (a [`Screen`](crate::screen)), reached from the
//! campaign map — the between-worlds outfitter, beside the campaign depth that
//! gates its stock. Mirrors `opponent_select.rs`/`deck_builder.rs`: a cursor + an
//! owned outcome enum + `draw(frame, config, profile, pulse)` + one app arm. The
//! screen owns only the cursor; the pool comes from [`economy`], the balance and
//! collection from the [`Profile`], and a purchase is applied through the
//! profile's own `try_purchase` (the app performs it). See `specs/012-economy`.

use crossterm::event::KeyCode;

use crate::{
    card::Card,
    config::Config,
    economy,
    frame::{Emphasis, Frame, draw_text_centered},
    profile::Profile,
};

/// Vertical anchors for the shop at a given terminal height and pool size:
/// `(title_y, items_top, hint_y)`. The list is **single-spaced** so the full
/// 15-card Core pool plus the balance and hint fit the 89×31 minimum (a
/// double-spaced list would clip — see `the_full_pool_fits_the_minimum_terminal`).
fn anchors(num_rows: usize, n: usize) -> (usize, usize, usize) {
    let block = 2 + 1 + n + 1 + 1; // title+balance, gap, n items, gap, hint
    let top = num_rows.saturating_sub(block) / 2;
    let items_top = top + 3; // title at `top`, balance at `top+1`, then a gap
    let hint_y = items_top + n + 1;
    (top, items_top, hint_y)
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
    cursor: usize, // index into economy::available_pool(run)
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
    /// available pool, wrapping; Enter/Space buy the highlighted card; Esc/`x`
    /// leave. `None` for keys this screen ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<ShopOutcome> {
        let pool = economy::available_pool(profile.campaign());
        let n = pool.len();
        // The pool is never empty in practice (the Outer tier is always
        // unlocked), but the list is data-driven, so guard rather than index in.
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
            KeyCode::Enter | KeyCode::Char(' ') => Some(ShopOutcome::Buy(pool[self.cursor])),
            KeyCode::Esc | KeyCode::Char('x') => Some(ShopOutcome::Back),
            _ => None,
        }
    }

    /// Move the cursor by `delta` over the pool, wrapping at the ends.
    fn move_by(&mut self, delta: isize, n: usize) {
        self.cursor = (self.cursor as isize + delta).rem_euclid(n as isize) as usize;
    }

    /// Draw the title, the credit balance, one row per available card
    /// (`label · price · owned`, the cursored row pulsing, cards you can't
    /// afford dimmed), and the controls hint.
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        const TITLE: &str = "Outfitter";
        const HINT: &str = "↑/↓ choose  ·  Enter buy  ·  Esc back";

        let pool = economy::available_pool(profile.campaign());
        let credits = profile.credits();

        let center_x = config.num_cols / 2;
        let (title_y, items_top, hint_y) = anchors(config.num_rows, pool.len());
        draw_text_centered(frame, center_x, title_y, TITLE, Emphasis::Normal);

        let balance = format!("Credits: ◈ {credits}");
        draw_text_centered(frame, center_x, title_y + 1, &balance, Emphasis::Strong);

        for (i, &card) in pool.iter().enumerate() {
            let price = economy::card_price(card);
            let owned = profile.owned_count(card);
            let affordable = credits >= price;
            let cursored = i == self.cursor;

            // Cursored row pulses; other affordable rows are Normal, ones you
            // can't afford recede to Muted so the selection and your options
            // both read at a glance.
            let emphasis = if cursored {
                pulse
            } else if affordable {
                Emphasis::Normal
            } else {
                Emphasis::Muted
            };
            // A fixed-width marker (cursored or blank) keeps the centered rows
            // from jittering as the cursor moves.
            let marker = if cursored { "▸" } else { " " };
            let row = format!("{marker}  {:<4}   {:>3} cr   owned ×{owned}", card.label(), price);
            draw_text_centered(frame, center_x, items_top + i, &row, emphasis);
        }

        draw_text_centered(frame, center_x, hint_y, HINT, Emphasis::Muted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn arrows_move_over_the_pool_and_wrap() {
        let p = core_profile();
        let n = economy::available_pool(p.campaign()).len();
        let mut s = ShopState::new();
        assert_eq!(s.cursor, 0);

        assert!(matches!(s.handle_input(KeyCode::Up, &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, n - 1); // up from the top wraps to the bottom
        assert!(matches!(s.handle_input(KeyCode::Down, &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, 0);
        // w/s mirror up/down.
        assert!(matches!(s.handle_input(KeyCode::Char('s'), &p), Some(ShopOutcome::Moved)));
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn enter_and_space_buy_the_highlighted_card() {
        let p = core_profile();
        let pool = economy::available_pool(p.campaign());
        let mut s = ShopState::new();
        s.handle_input(KeyCode::Down, &p); // now on pool[1]
        let under = pool[1];

        assert!(matches!(s.handle_input(KeyCode::Enter, &p), Some(ShopOutcome::Buy(c)) if c == under));
        assert!(matches!(s.handle_input(KeyCode::Char(' '), &p), Some(ShopOutcome::Buy(c)) if c == under));
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
    fn the_full_pool_fits_the_minimum_terminal() {
        // The whole 15-card Core pool plus the title, balance, and hint must
        // land within the 89×31 minimum — the single-spaced list is what makes
        // it fit (a double-spaced one clips the last card and the hint).
        let (title_y, items_top, hint_y) = anchors(31, crate::card::ALL_SIDE_CARDS.len());
        assert!(items_top > title_y + 1, "items must clear the title and balance rows");
        assert!(hint_y < 31, "the hint (row {hint_y}) clips the 31-row minimum");
    }
}
