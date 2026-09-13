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

/// A content row's role in the prompt — what it says and how it's emphasized.
/// `Spacer` is an empty row: the breathing room the design brief asks for
/// around the stake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Plain,
    Stake,
    Hint,
    Spacer,
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

    /// What a content row is for — the one place emphasis is decided, so the
    /// spacer rows below can't drift a hardcoded index out from under it.
    fn rows(&self) -> Vec<(Role, String)> {
        let stake = self.stake();
        // Winnings, not the payout: the line stays correct if PAYOUT_RATIO moves.
        let winnings = win_payout(stake).saturating_sub(stake);
        vec![
            (
                Role::Plain,
                format!("Wager — {} · {}", self.opponent.name, self.planet.name),
            ),
            (
                Role::Plain,
                format!("Ante ◈ {}   Balance ◈ {}", self.floor, self.max),
            ),
            (Role::Spacer, String::new()),
            (Role::Stake, format!("◂  Stake ◈ {stake}  ▸")),
            (Role::Spacer, String::new()),
            (Role::Plain, format!("Win +{winnings}   ·   Lose −{stake}")),
            (
                Role::Hint,
                "←/→ stake  ·  Enter play  ·  Esc back".to_string(),
            ),
        ]
    }

    /// The prompt's content rows as plain text, in draw order — pure, so the
    /// tests read the same strings the box renders. Blank entries are the
    /// spacer rows the design brief's breathing-room rule calls for (one above
    /// and one below the stake the player acts on; every other row is packed);
    /// the box grows with them, since [`draw`] sizes the overlay from [`rows`].
    ///
    /// [`draw`]: WagerState::draw
    /// [`rows`]: WagerState::rows
    #[cfg(test)]
    fn lines(&self) -> Vec<String> {
        self.rows().into_iter().map(|(_, text)| text).collect()
    }

    /// Draw the centered, bordered prompt: the content rows from `rows`, each
    /// row's emphasis taken from its [`Role`] (the stake breathing with
    /// `pulse`, the hint muted) — monochrome by construction.
    pub fn draw(&self, frame: &mut Frame, config: &Config, pulse: Emphasis) {
        let rows = self.rows();
        let width = rows.iter().map(|(_, l)| l.chars().count()).max().unwrap_or(0);
        let layout = OverlayLayout::new(*config, width, rows.len());

        clear_rect(frame, layout.outer);
        draw_box(frame, layout.outer, BorderWeight::Single, Emphasis::Normal);
        for (i, (role, line)) in rows.iter().enumerate() {
            let emphasis = match role {
                Role::Stake => pulse,
                Role::Hint => Emphasis::Muted,
                Role::Plain | Role::Spacer => Emphasis::Normal,
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
    fn rows_breathe_around_the_stake() {
        // The design brief's density rule: a blank row above and below the row
        // the player acts on, and nowhere else — the rest of the prompt stays
        // packed. Roles (not indices) carry the emphasis, so the spacers can't
        // shift it.
        let s = state(53);
        let rows = s.rows();
        let roles: Vec<Role> = rows.iter().map(|(role, _)| *role).collect();
        assert_eq!(
            roles,
            vec![
                Role::Plain,  // title
                Role::Plain,  // ante · balance
                Role::Spacer,
                Role::Stake,  // the row the player acts on
                Role::Spacer,
                Role::Plain,  // win · lose
                Role::Hint,
            ]
        );
        for (role, text) in &rows {
            assert_eq!(
                *role == Role::Spacer,
                text.is_empty(),
                "spacers are the only blank rows: {role:?} {text:?}"
            );
        }
        // lines() carries the same rows, blanks included.
        let stake_row = roles.iter().position(|r| *r == Role::Stake).expect("a stake row");
        assert_eq!(s.lines().len(), rows.len());
        assert_eq!(s.lines()[stake_row], "◂  Stake ◈ 10  ▸");
    }

    #[test]
    fn the_prompt_fits_the_minimum_terminal_unclamped() {
        // The box must still fit 139×31 with margin — if it ever outgrows the
        // frame, OverlayLayout clamps and the rows get eaten.
        let (cols, rows) = Config::min_size();
        let config = Config { num_cols: cols, num_rows: rows };
        let s = state(999_999);
        let lines = s.lines();
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let layout = OverlayLayout::new(config, width, lines.len());
        assert_eq!(
            layout.outer.height(),
            lines.len() + crate::V_PAD,
            "box height clamped — the prompt outgrew the minimum terminal"
        );
        assert_eq!(layout.outer.width(), width + 2 * crate::H_PAD, "box width clamped");
        assert!(layout.outer.y1 < rows && layout.outer.x1 < cols, "box off-frame");
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
