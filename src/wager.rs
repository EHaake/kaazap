//! The wager prompt: a small centered modal shown before a campaign match
//! launches. It names the opponent and planet, shows the ante floor and the
//! balance, and lets the player walk a stake grid with ←/→ before committing.
//! State + input + the pure content builder live here; the app wiring is T004.
//!
//! See `specs/021-wager-and-loss`.

use crossterm::event::KeyCode;

use crate::{
    campaign::Planet,
    config::Config,
    economy::{STAKE_STEP, ante_floor, win_payout},
    frame::{Align, BorderWeight, Emphasis, Frame, clear_rect, draw_box, draw_text_in},
    layout::OverlayLayout,
    opponent::OpponentProfile,
};

/// What a keypress on the wager prompt resolves to.
#[derive(Debug, Clone, Copy)]
pub enum WagerOutcome {
    Moved,
    Commit,
    Cancel,
}

/// The wager prompt's transient state: which match is being staked, the ante
/// floor and the balance it's bounded by, and the index `k` into the stake grid
/// `floor + k·STAKE_STEP` (clamped to the balance).
#[derive(Debug)]
pub struct WagerState {
    planet: Planet,
    opponent: OpponentProfile,
    floor: u32,
    max: u32,
    k: usize,
}

impl WagerState {
    /// Open the prompt at the ante floor.
    ///
    /// Precondition: `balance >= ante_floor(opponent.stand_threshold)` — the
    /// launch gate checks it, and `max` is the full balance (not a post-reserve
    /// amount), so an all-in stake is always reachable.
    pub fn new(planet: Planet, opponent: OpponentProfile, balance: u32) -> Self {
        Self {
            planet,
            opponent,
            floor: ante_floor(opponent.stand_threshold),
            max: balance,
            k: 0,
        }
    }

    /// The currently selected stake: the grid value, clamped to the balance so
    /// the top of the grid is always exactly all-in.
    pub fn stake(&self) -> u32 {
        (self.floor + self.k as u32 * STAKE_STEP).min(self.max)
    }

    /// The highest grid index — `ceil((max − floor) / STAKE_STEP)`, so the last
    /// step lands on the balance even when it sits off the grid.
    fn k_max(&self) -> usize {
        self.max.saturating_sub(self.floor).div_ceil(STAKE_STEP) as usize
    }

    /// The planet this wager is for (the caller re-enters the map with it).
    pub fn planet_id(&self) -> &'static str {
        self.planet.id
    }

    /// The opponent this wager is for.
    pub fn opponent(&self) -> OpponentProfile {
        self.opponent
    }

    /// Left/Right walk the stake grid — a move that can't happen (Left at the
    /// floor, Right at the balance) returns `None`, like the map's `step`, so
    /// no move cue fires. (Ctrl+B/F arrive as Left/Right via `resolve_key`.)
    pub fn handle_input(&mut self, key: KeyCode) -> Option<WagerOutcome> {
        match key {
            KeyCode::Left | KeyCode::Char('a') => {
                if self.k == 0 {
                    None
                } else {
                    self.k -= 1;
                    Some(WagerOutcome::Moved)
                }
            }
            KeyCode::Right | KeyCode::Char('d') => {
                if self.k >= self.k_max() {
                    None
                } else {
                    self.k += 1;
                    Some(WagerOutcome::Moved)
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => Some(WagerOutcome::Commit),
            KeyCode::Esc | KeyCode::Char('x') => Some(WagerOutcome::Cancel),
            _ => None,
        }
    }

    /// The prompt's content rows, in draw order — pure, so the tests read the
    /// same strings the box renders.
    pub fn lines(&self) -> Vec<String> {
        let stake = self.stake();
        // Winnings, not the payout: the line stays correct if PAYOUT_RATIO moves.
        let winnings = win_payout(stake).saturating_sub(stake);
        vec![
            format!("Wager — {} · {}", self.opponent.name, self.planet.name),
            format!("Ante ◈ {}   Balance ◈ {}", self.floor, self.max),
            format!("◂  Stake ◈ {stake}  ▸"),
            format!("Win +{winnings}   ·   Lose −{stake}"),
            "←/→ stake  ·  Enter play  ·  Esc back".to_string(),
        ]
    }

    /// Draw the centered, bordered prompt: the content rows from [`lines`], the
    /// stake row breathing with `pulse` and the hint muted — monochrome by
    /// construction.
    ///
    /// [`lines`]: WagerState::lines
    pub fn draw(&self, frame: &mut Frame, config: &Config, pulse: Emphasis) {
        let lines = self.lines();
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let layout = OverlayLayout::new(*config, width, lines.len());

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
        for (i, line) in lines.iter().enumerate() {
            let emphasis = match i {
                2 => pulse,                        // the stake row
                4 => Emphasis::Muted,              // the key hint
                _ => Emphasis::Normal,
            };
            draw_text_in(frame, layout.inner, i, Align::Center, line, emphasis);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::planet_by_id;
    use crate::opponent::opponent_by_id;

    /// Greeb: stand threshold 15 → ante floor 10.
    fn state(balance: u32) -> WagerState {
        let planet = planet_by_id("cinder").expect("cinder is a real planet");
        let opponent = opponent_by_id("greeb").expect("greeb is in the roster");
        assert_eq!(ante_floor(opponent.stand_threshold), 10, "fixture floor");
        WagerState::new(planet, opponent, balance)
    }

    #[test]
    fn opens_at_the_floor() {
        let s = state(53);
        assert_eq!(s.stake(), 10);
        assert_eq!(s.planet_id(), "cinder");
        assert_eq!(s.opponent().id, "greeb");
    }

    #[test]
    fn right_walks_the_grid_to_all_in_then_stops() {
        let mut s = state(53);
        let mut walked = vec![s.stake()];
        while let Some(WagerOutcome::Moved) = s.handle_input(KeyCode::Right) {
            walked.push(s.stake());
        }
        assert_eq!(walked, vec![10, 15, 20, 25, 30, 35, 40, 45, 50, 53]);
        // At the top, Right is a no-op with no move cue.
        assert!(s.handle_input(KeyCode::Right).is_none());
        assert!(s.handle_input(KeyCode::Char('d')).is_none());
        assert_eq!(s.stake(), 53);
    }

    #[test]
    fn left_walks_back_down_to_the_floor_then_stops() {
        let mut s = state(53);
        while let Some(WagerOutcome::Moved) = s.handle_input(KeyCode::Right) {}
        assert_eq!(s.stake(), 53);

        let mut walked = vec![];
        while let Some(WagerOutcome::Moved) = s.handle_input(KeyCode::Left) {
            walked.push(s.stake());
        }
        assert_eq!(walked, vec![50, 45, 40, 35, 30, 25, 20, 15, 10]);
        assert!(s.handle_input(KeyCode::Left).is_none());
        assert!(s.handle_input(KeyCode::Char('a')).is_none());
        assert_eq!(s.stake(), 10);
    }

    #[test]
    fn balance_at_the_floor_has_nowhere_to_move() {
        let mut s = state(10);
        assert_eq!(s.stake(), 10);
        assert!(s.handle_input(KeyCode::Right).is_none());
        assert!(s.handle_input(KeyCode::Left).is_none());
        assert_eq!(s.stake(), 10);
    }

    #[test]
    fn enter_and_space_commit_at_the_current_stake() {
        let mut s = state(53);
        assert!(matches!(
            s.handle_input(KeyCode::Enter),
            Some(WagerOutcome::Commit)
        ));
        assert_eq!(s.stake(), 10, "committing doesn't move the stake");

        s.handle_input(KeyCode::Right);
        assert!(matches!(
            s.handle_input(KeyCode::Char(' ')),
            Some(WagerOutcome::Commit)
        ));
        assert_eq!(s.stake(), 15);
    }

    #[test]
    fn esc_and_x_cancel_unknown_none() {
        let mut s = state(53);
        assert!(matches!(
            s.handle_input(KeyCode::Esc),
            Some(WagerOutcome::Cancel)
        ));
        assert!(matches!(
            s.handle_input(KeyCode::Char('x')),
            Some(WagerOutcome::Cancel)
        ));
        assert!(s.handle_input(KeyCode::Char('q')).is_none());
        assert!(s.handle_input(KeyCode::Up).is_none());
    }

    #[test]
    fn lines_carry_the_opponent_ante_balance_and_payout() {
        let mut s = state(53);
        s.handle_input(KeyCode::Right); // stake 15
        assert_eq!(s.stake(), 15);
        let text = s.lines().join("\n");
        assert!(text.contains("Greeb"), "opponent name missing: {text}");
        assert!(text.contains("Ante ◈ 10"), "ante missing: {text}");
        assert!(text.contains("Balance ◈ 53"), "balance missing: {text}");
        assert!(text.contains("Stake ◈ 15"), "stake missing: {text}");
        assert!(text.contains("+15"), "winnings missing: {text}");
        assert!(text.contains("−15"), "loss missing: {text}");
    }

    #[test]
    fn every_line_fits_seventy_columns() {
        // Widest case: the deepest opponent (biggest floor) and a fat balance.
        for opponent in crate::opponent::OPPONENTS {
            for planet in crate::campaign::PLANETS {
                let mut s = WagerState::new(planet, opponent, 999_999);
                for _ in 0..3 {
                    s.handle_input(KeyCode::Right);
                }
                for line in s.lines() {
                    assert!(
                        line.chars().count() <= 70,
                        "line over 70 columns ({}): {line}",
                        line.chars().count()
                    );
                }
            }
        }
    }
}
