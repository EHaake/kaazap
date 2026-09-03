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
    layout::MenuLayout,
    profile::Profile,
};

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

        // One title line + the list; reserve the footer rows for the balance
        // (above the list) and the hint (below), so the whole thing centres and
        // fits the minimum terminal.
        let layout = MenuLayout::new(*config, 1, pool.len().max(1), 4);
        draw_text_centered(frame, layout.center_x, layout.title_top, TITLE, Emphasis::Normal);

        let balance = format!("Credits: ◈ {credits}");
        draw_text_centered(frame, layout.center_x, layout.title_top + 1, &balance, Emphasis::Strong);

        let mut y = layout.items_top;
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
            draw_text_centered(frame, layout.center_x, y, &row, emphasis);
            y += layout.item_spacing;
        }

        draw_text_centered(frame, layout.center_x, y + 2, HINT, Emphasis::Muted);
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
}
