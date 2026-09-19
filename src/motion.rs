//! One-shot board transitions (spec 027): which element just changed and how
//! much of its beat is left. `App` feeds it from `tick` by diffing the game
//! state — the banter/audio observer pattern — and `board.rs` reads it when it
//! draws. Drawing state only: never saved, never read by the engine.

use std::time::Duration;

use crate::game::{GamePhase, GameState};
use crate::player::Player;
use crate::{ARRIVAL_BEAT_MS, FLIP_BEAT_MS, HAND_SIZE, POPUP_BEAT_MS, THINKING_STEP_MS};

/// A board element that can be in transition: a card by its index in its
/// side's row (stable until the row clears), a side's Score figure, or — the
/// source ghost, Revision 1 — the hand slot a side card was just played from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elem {
    Dealer(Player, usize),
    Played(Player, usize),
    Score(Player),
    Hand(Player, usize),
}

impl Elem {
    fn side(self) -> Player {
        match self {
            Elem::Dealer(who, _) | Elem::Played(who, _) | Elem::Score(who) | Elem::Hand(who, _) => who,
        }
    }
}

/// `hand[i]`: whether slot i holds a card (Revision 1 — a true→false slot
/// starts a `Hand` ghost).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SideSnapshot {
    dealers: usize,
    played: usize,
    score: i32,
    hand: [bool; HAND_SIZE],
}

/// The facts the diff compares. `resolved` is exactly the condition under which
/// `draw_round_outcome_text` draws a popup today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MotionSnapshot {
    player: SideSnapshot,
    opponent: SideSnapshot,
    resolved: bool,
    thinking: bool,
}

impl MotionSnapshot {
    fn of(gs: &GameState) -> Self {
        let side = |p: &crate::player::PlayerState| SideSnapshot {
            dealers: p.dealer_row.len(),
            played: p.played_row.len(),
            score: p.score(),
            hand: std::array::from_fn(|i| p.hand.get(i).is_some_and(Option::is_some)),
        };
        Self {
            player: side(&gs.player),
            opponent: side(&gs.opponent),
            resolved: gs.round_outcome.is_some()
                || matches!(gs.game_phase, GamePhase::GameOver { .. }),
            thinking: matches!(gs.game_phase, GamePhase::OpponentThinking { .. }),
        }
    }

    fn side(&self, who: Player) -> SideSnapshot {
        match who {
            Player::Player => self.player,
            Player::Opponent => self.opponent,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BoardMotion {
    prev: Option<MotionSnapshot>,
    /// Arrivals in flight: the element and the beat remaining. Independent
    /// clocks — a new one never shortens or delays another.
    arrivals: Vec<(Elem, Duration)>,
    /// Time left before the round/game popup draws; ZERO whenever the board is
    /// not resolved, so a resumed or already-shown popup is due at once.
    popup_wait: Duration,
    /// Time spent in the current `OpponentThinking` pause; None outside it.
    thinking_for: Option<Duration>,
}

impl BoardMotion {
    /// Advance every clock by `dt`, then diff `gs` against the last observation
    /// and start the transitions it implies. The first observation after a
    /// reset seeds silently: nothing starts (spec: the first frame is settled).
    pub fn observe(&mut self, gs: &GameState, dt: Duration) {
        for (_, left) in &mut self.arrivals {
            *left = left.saturating_sub(dt);
        }
        self.arrivals.retain(|(_, left)| !left.is_zero());
        self.popup_wait = self.popup_wait.saturating_sub(dt);
        if let Some(t) = &mut self.thinking_for {
            *t += dt;
        }

        let curr = MotionSnapshot::of(gs);
        match self.prev {
            None => {
                self.popup_wait = Duration::ZERO;
                self.thinking_for = curr.thinking.then_some(Duration::ZERO);
            }
            Some(prev) => {
                for who in [Player::Player, Player::Opponent] {
                    self.diff_side(who, prev.side(who), curr.side(who));
                }
                if curr.resolved && !prev.resolved {
                    self.popup_wait = ms(POPUP_BEAT_MS);
                }
                if !curr.resolved {
                    self.popup_wait = Duration::ZERO;
                }
                if curr.thinking && !prev.thinking {
                    self.thinking_for = Some(Duration::ZERO);
                }
                if !curr.thinking {
                    self.thinking_for = None;
                }
            }
        }
        self.prev = Some(curr);
    }

    /// A row that shrank is a clear (round end, rematch): drop this side's
    /// transitions and start none — a reset total is not a change to guide the
    /// eye to. Otherwise: one arrival per new card index, the Score if the
    /// total differs (restarting its clock if it is already running), and
    /// (Revision 1) a `Hand(who, i)` ghost for each slot that went filled →
    /// empty — all on the arrival beat.
    fn diff_side(&mut self, who: Player, prev: SideSnapshot, curr: SideSnapshot) {
        if curr.dealers < prev.dealers || curr.played < prev.played {
            self.arrivals.retain(|(e, _)| e.side() != who);
            return;
        }
        for i in prev.dealers..curr.dealers {
            self.arrivals.push((Elem::Dealer(who, i), ms(ARRIVAL_BEAT_MS)));
        }
        for i in prev.played..curr.played {
            self.arrivals.push((Elem::Played(who, i), ms(ARRIVAL_BEAT_MS)));
        }
        for i in 0..HAND_SIZE {
            if prev.hand[i] && !curr.hand[i] {
                self.arrivals.push((Elem::Hand(who, i), ms(ARRIVAL_BEAT_MS)));
            }
        }
        if curr.score != prev.score {
            self.arrivals.retain(|(e, _)| *e != Elem::Score(who));
            self.arrivals.push((Elem::Score(who), ms(ARRIVAL_BEAT_MS)));
        }
    }

    pub fn is_arriving(&self, elem: Elem) -> bool {
        self.arrivals.iter().any(|(e, _)| *e == elem)
    }

    /// Revision 1: whether `elem` is still inside the flip window of its
    /// arrival — the first FLIP_BEAT_MS of the beat. A read of the countdown,
    /// not a clock of its own; true only for a `Dealer` arrival (a played
    /// card shows its value from its first frame, spec Q9).
    pub fn is_face_down(&self, elem: Elem) -> bool {
        self.arrivals.iter().any(|(e, left)| {
            matches!(e, Elem::Dealer(..)) && *e == elem && *left + ms(FLIP_BEAT_MS) > ms(ARRIVAL_BEAT_MS)
        })
    }

    pub fn popup_due(&self) -> bool {
        self.popup_wait.is_zero()
    }

    /// The thinking indicator for this instant, while the opponent is thinking.
    pub fn thinking_suffix(&self) -> Option<&'static str> {
        self.thinking_for.map(thinking_suffix_at)
    }
}

/// ` .`, ` ..`, ` ...`, cycling every THINKING_STEP_MS from the start of the pause. Pure.
pub fn thinking_suffix_at(elapsed: Duration) -> &'static str {
    const DOTS: [&str; 3] = [" .", " ..", " ..."];
    DOTS[((elapsed.as_millis() as u64 / THINKING_STEP_MS) % 3) as usize]
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, FlipKind, PlayedCard};
    use crate::game::RoundOutcome;
    use crate::{FLIP_BEAT_MS, OPPONENT_THINKING_TIME_MS, SELECTION_PULSE_MS};
    use std::time::Instant;

    const ZERO: Duration = Duration::ZERO;

    fn arrival() -> Duration {
        ms(ARRIVAL_BEAT_MS)
    }

    fn flip() -> Duration {
        ms(FLIP_BEAT_MS)
    }

    fn popup() -> Duration {
        ms(POPUP_BEAT_MS)
    }

    fn step() -> Duration {
        ms(THINKING_STEP_MS)
    }

    fn dealer(n: u8) -> PlayedCard {
        PlayedCard { card: Card::Dealer(n), value: n as i8 }
    }

    fn thinking_phase() -> GamePhase {
        GamePhase::OpponentThinking { until: Instant::now() + ms(OPPONENT_THINKING_TIME_MS) }
    }

    /// A fresh game observed once: the seed, nothing in flight.
    fn seeded() -> (GameState, BoardMotion) {
        let gs = GameState::new();
        let mut m = BoardMotion::default();
        m.observe(&gs, ZERO);
        (gs, m)
    }

    fn side_settled(m: &BoardMotion, who: Player, dealers: usize, played: usize) -> bool {
        (0..dealers).all(|i| !m.is_arriving(Elem::Dealer(who, i)))
            && (0..played).all(|i| !m.is_arriving(Elem::Played(who, i)))
            && !m.is_arriving(Elem::Score(who))
    }

    #[test]
    fn beats_are_named_constants_within_bounds() {
        assert!(SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS);
        assert!(ARRIVAL_BEAT_MS <= 1000);
        assert!(ARRIVAL_BEAT_MS <= POPUP_BEAT_MS);
        assert!(POPUP_BEAT_MS <= 1000);
        assert!(THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS);
        assert!(150 <= FLIP_BEAT_MS);
        assert!(FLIP_BEAT_MS * 2 <= ARRIVAL_BEAT_MS);
    }

    #[test]
    fn the_first_observation_starts_nothing() {
        let mut gs = GameState::new();
        gs.player.dealer_row = vec![dealer(7), dealer(5)];
        gs.player.played_row = vec![PlayedCard { card: Card::Plus(3), value: 3 }];
        gs.opponent.dealer_row = vec![dealer(9), dealer(8)];
        gs.opponent.played_row = vec![PlayedCard { card: Card::Minus(2), value: -2 }];
        gs.round_outcome = Some(RoundOutcome::OpponentWon);
        gs.game_phase = GamePhase::AwaitingNextRound;

        let mut m = BoardMotion::default();
        m.observe(&gs, ZERO);
        assert!(side_settled(&m, Player::Player, 2, 1));
        assert!(side_settled(&m, Player::Opponent, 2, 1));
        assert!(m.popup_due());
        assert_eq!(m.thinking_suffix(), None);

        // Resumed mid-pause: the indicator shows from the first frame.
        gs.round_outcome = None;
        gs.game_phase = thinking_phase();
        let mut m = BoardMotion::default();
        m.observe(&gs, ZERO);
        assert!(side_settled(&m, Player::Player, 2, 1));
        assert!(side_settled(&m, Player::Opponent, 2, 1));
        assert_eq!(m.thinking_suffix(), Some(" ."));
    }

    #[test]
    fn a_dealt_card_and_its_total_arrive_for_one_beat() {
        for who in [Player::Player, Player::Opponent] {
            let other = match who {
                Player::Player => Player::Opponent,
                Player::Opponent => Player::Player,
            };
            let (mut gs, mut m) = seeded();
            match who {
                Player::Player => gs.player.dealer_row.push(dealer(7)),
                Player::Opponent => gs.opponent.dealer_row.push(dealer(7)),
            }
            m.observe(&gs, ZERO);
            assert!(m.is_arriving(Elem::Dealer(who, 0)));
            assert!(m.is_arriving(Elem::Score(who)));
            assert!(!m.is_arriving(Elem::Dealer(who, 1)));
            assert!(!m.is_arriving(Elem::Score(other)));

            m.observe(&gs, arrival() - ms(1));
            assert!(m.is_arriving(Elem::Dealer(who, 0)));
            assert!(m.is_arriving(Elem::Score(who)));

            m.observe(&gs, ms(1));
            assert!(!m.is_arriving(Elem::Dealer(who, 0)));
            assert!(!m.is_arriving(Elem::Score(who)));
        }
    }

    #[test]
    fn a_dealt_card_is_face_down_for_the_flip_beat_then_faces_up() {
        for who in [Player::Player, Player::Opponent] {
            let (mut gs, mut m) = seeded();
            match who {
                Player::Player => gs.player.dealer_row.push(dealer(7)),
                Player::Opponent => gs.opponent.dealer_row.push(dealer(7)),
            }
            m.observe(&gs, ZERO);
            assert!(m.is_face_down(Elem::Dealer(who, 0)));
            assert!(m.is_arriving(Elem::Dealer(who, 0)));

            m.observe(&gs, flip() - ms(1));
            assert!(m.is_face_down(Elem::Dealer(who, 0)));

            // The flip ends: the value shows for the rest of the beat.
            m.observe(&gs, ms(1));
            assert!(!m.is_face_down(Elem::Dealer(who, 0)));
            assert!(m.is_arriving(Elem::Dealer(who, 0)));

            m.observe(&gs, arrival() - flip());
            assert!(!m.is_arriving(Elem::Dealer(who, 0)));
            assert!(!m.is_face_down(Elem::Dealer(who, 0)));
        }

        // A played card is never face down, even on its first frame (the
        // countdown alone would say it is; the Dealer guard says otherwise).
        let (mut gs, mut m) = seeded();
        gs.player.played_row.push(PlayedCard { card: Card::Plus(3), value: 3 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Played(Player::Player, 0)));
        assert!(!m.is_face_down(Elem::Played(Player::Player, 0)));
    }

    #[test]
    fn a_play_arrives_with_the_total_and_a_draw_and_play_arrive_together() {
        let (mut gs, mut m) = seeded();

        gs.player.played_row.push(PlayedCard { card: Card::Plus(3), value: 3 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Played(Player::Player, 0)));
        assert!(m.is_arriving(Elem::Score(Player::Player)));
        assert!(!m.is_arriving(Elem::Score(Player::Opponent)));

        // The opponent draws and plays in one move.
        gs.opponent.dealer_row.push(dealer(6));
        gs.opponent.played_row.push(PlayedCard { card: Card::Plus(2), value: 2 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Dealer(Player::Opponent, 0)));
        assert!(m.is_arriving(Elem::Played(Player::Opponent, 0)));
        assert!(m.is_arriving(Elem::Score(Player::Opponent)));

        // A zero-value play: the card arrives, the total did not change.
        m.observe(&gs, arrival());
        assert!(!m.is_arriving(Elem::Score(Player::Player)));
        gs.player.played_row.push(PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Played(Player::Player, 1)));
        assert!(!m.is_arriving(Elem::Score(Player::Player)));
    }

    #[test]
    fn a_play_ghosts_its_hand_slot_for_one_beat() {
        let mut gs = GameState::new();
        gs.player.hand = vec![Some(Card::Plus(3)), Some(Card::Minus(2)), None, None];
        gs.opponent.hand = vec![Some(Card::Plus(2)), None, None, None];
        let mut m = BoardMotion::default();
        m.observe(&gs, ZERO);
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 0)));
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 1)));

        // The player plays from slot 1: the ghost and the card arrive together.
        gs.player.hand[1] = None;
        gs.player.played_row.push(PlayedCard { card: Card::Minus(2), value: -2 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Hand(Player::Player, 1)));
        assert!(m.is_arriving(Elem::Played(Player::Player, 0)));
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 0)));
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 2)));

        m.observe(&gs, arrival() - ms(1));
        assert!(m.is_arriving(Elem::Hand(Player::Player, 1)));
        m.observe(&gs, ms(1));
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 1)));
        assert!(!m.is_arriving(Elem::Played(Player::Player, 0)));

        // The opponent draws and plays in one move: all three transition together.
        gs.opponent.hand[0] = None;
        gs.opponent.dealer_row.push(dealer(6));
        gs.opponent.played_row.push(PlayedCard { card: Card::Plus(2), value: 2 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Hand(Player::Opponent, 0)));
        assert!(m.is_arriving(Elem::Dealer(Player::Opponent, 0)));
        assert!(m.is_arriving(Elem::Played(Player::Opponent, 0)));

        // A slot filling (a rematch's deal) starts nothing.
        m.observe(&gs, arrival());
        gs.player.hand[1] = Some(Card::Plus(1));
        m.observe(&gs, ZERO);
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 1)));

        // A ghost in flight is dropped when the side's rows clear.
        gs.player.hand[0] = None;
        gs.player.played_row.push(PlayedCard { card: Card::Plus(3), value: 3 });
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Hand(Player::Player, 0)));
        gs.player.dealer_row = vec![];
        gs.player.played_row = vec![];
        m.observe(&gs, ZERO);
        assert!(!m.is_arriving(Elem::Hand(Player::Player, 0)));
        assert!(side_settled(&m, Player::Player, 0, 2));
    }

    #[test]
    fn transitions_run_on_independent_clocks() {
        let (mut gs, mut m) = seeded();
        gs.player.dealer_row.push(dealer(4));
        m.observe(&gs, ZERO);

        gs.player.dealer_row.push(dealer(5));
        m.observe(&gs, ms(200));
        assert!(m.is_arriving(Elem::Dealer(Player::Player, 0)));
        assert!(m.is_arriving(Elem::Dealer(Player::Player, 1)));
        assert!(m.is_arriving(Elem::Score(Player::Player)));

        // The first settles at ARRIVAL from its own start; the second lives on.
        m.observe(&gs, arrival() - ms(200));
        assert!(!m.is_arriving(Elem::Dealer(Player::Player, 0)));
        assert!(m.is_arriving(Elem::Dealer(Player::Player, 1)));
        assert!(m.is_arriving(Elem::Score(Player::Player))); // restarted by the second hit

        m.observe(&gs, ms(200));
        assert!(!m.is_arriving(Elem::Dealer(Player::Player, 1)));
        assert!(!m.is_arriving(Elem::Score(Player::Player)));
    }

    #[test]
    fn a_cleared_row_discards_its_transitions_and_starts_none() {
        let (mut gs, mut m) = seeded();
        gs.player.dealer_row = vec![dealer(10), dealer(10)];
        gs.player.played_row = vec![PlayedCard { card: Card::Plus(1), value: 1 }];
        gs.opponent.dealer_row = vec![dealer(9)];
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Dealer(Player::Player, 1)));
        assert!(m.is_arriving(Elem::Dealer(Player::Opponent, 0)));

        // Round end: both rows empty, 20 -> 0, as setup_next_round does.
        gs.player.dealer_row = vec![];
        gs.player.played_row = vec![];
        gs.opponent.dealer_row = vec![];
        gs.opponent.played_row = vec![];
        gs.round_outcome = None;
        gs.game_phase = GamePhase::PlayerTurn;
        m.observe(&gs, ZERO);
        assert!(side_settled(&m, Player::Player, 2, 1));
        assert!(side_settled(&m, Player::Opponent, 1, 0));
        assert_eq!(m, BoardMotion { prev: m.prev, ..BoardMotion::default() });

        // A rematch: GameOver -> PlayerTurn with rounds reset.
        gs.player.dealer_row = vec![dealer(10), dealer(10)];
        gs.player.rounds_won = 3;
        gs.game_phase = GamePhase::GameOver { winner: Player::Player };
        m.observe(&gs, ZERO);
        assert!(m.is_arriving(Elem::Dealer(Player::Player, 0)));
        gs.player.dealer_row = vec![];
        gs.player.rounds_won = 0;
        gs.game_phase = GamePhase::PlayerTurn;
        m.observe(&gs, ZERO);
        assert!(side_settled(&m, Player::Player, 2, 0));
        assert!(m.popup_due());
    }

    #[test]
    fn the_popup_waits_one_beat_and_n_clears_the_wait() {
        let (mut gs, mut m) = seeded();
        assert!(m.popup_due());
        gs.round_outcome = Some(RoundOutcome::PlayerWon);
        gs.game_phase = GamePhase::AwaitingNextRound;
        m.observe(&gs, ZERO);
        assert!(!m.popup_due());
        m.observe(&gs, popup() - ms(1));
        assert!(!m.popup_due());
        m.observe(&gs, ms(1));
        assert!(m.popup_due());

        // A fresh edge, then `n` mid-beat: the outcome is gone, the popup is due.
        let (mut gs, mut m) = seeded();
        gs.player.dealer_row.push(dealer(9));
        gs.round_outcome = Some(RoundOutcome::PlayerWon);
        gs.game_phase = GamePhase::AwaitingNextRound;
        m.observe(&gs, ZERO);
        assert!(!m.popup_due());
        gs.player.dealer_row = vec![];
        gs.round_outcome = None;
        gs.game_phase = GamePhase::PlayerTurn;
        m.observe(&gs, ms(100));
        assert!(m.popup_due());
        assert!(side_settled(&m, Player::Player, 1, 0));

        // The same edge into GameOver.
        let (mut gs, mut m) = seeded();
        gs.round_outcome = Some(RoundOutcome::OpponentWon);
        gs.game_phase = GamePhase::GameOver { winner: Player::Opponent };
        m.observe(&gs, ZERO);
        assert!(!m.popup_due());
        m.observe(&gs, popup() - ms(1));
        assert!(!m.popup_due());
        m.observe(&gs, ms(1));
        assert!(m.popup_due());
    }

    #[test]
    fn the_thinking_indicator_steps_through_the_pause_and_reverts() {
        assert_eq!(thinking_suffix_at(ZERO), " .");
        assert_eq!(thinking_suffix_at(step()), " ..");
        assert_eq!(thinking_suffix_at(step() * 2), " ...");
        assert_eq!(thinking_suffix_at(step() * 3), " .");

        let (mut gs, mut m) = seeded();
        assert_eq!(m.thinking_suffix(), None);
        gs.game_phase = thinking_phase();
        m.observe(&gs, ZERO);
        assert_eq!(m.thinking_suffix(), Some(" ."));
        m.observe(&gs, step());
        assert_eq!(m.thinking_suffix(), Some(" .."));
        gs.game_phase = GamePhase::PlayerTurn;
        m.observe(&gs, ZERO);
        assert_eq!(m.thinking_suffix(), None);

        // At least two distinct states across the pause, sampled every 50 ms.
        gs.game_phase = thinking_phase();
        m.observe(&gs, ZERO);
        let mut seen = Vec::new();
        let mut elapsed = 0;
        while elapsed <= OPPONENT_THINKING_TIME_MS {
            let s = m.thinking_suffix().unwrap();
            if !seen.contains(&s) {
                seen.push(s);
            }
            m.observe(&gs, ms(50));
            elapsed += 50;
        }
        assert!(seen.len() >= 2, "saw {seen:?}");
    }
}
