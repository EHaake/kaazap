//! Play log: an ordered record of every discrete move in the current round and
//! a running list of resolved round outcomes for the match — the data behind
//! the in-game "play log" overlay. Pure logic + formatting, no dependency on
//! rendering internals.
//!
//! Like `banter.rs` and `audio.rs`, the log **observes from the outside** by
//! diffing successive [`GameState`]s rather than hooking the engine — the
//! constitution forbids observing code from mutating state, and this spec
//! forbids engine change. Banter fires *one* event per transition;
//! the play log instead reconstructs *every* discrete move between two
//! snapshots as an ordered list (see [`moves_since`]). It rests on one
//! invariant the tests pin: at most one card is added to one side per capture,
//! so a side's `score()` after the diff is exactly the total right after its
//! move. `App` seeds the diff at match entry with [`PlayLog::reset`] and feeds
//! each tick through [`PlayLog::observe`], mirroring `App`'s banter seeding.
//! Spec 018.

use crate::card::PlayedCard;
use crate::game::{GamePhase, GameState, RoundOutcome};
use crate::player::{Player, PlayerState};

/// One discrete move visible on the board. Each carries the acting `side` and
/// the acting side's resulting total (its `score()` at capture — exact by the
/// one-card-per-diff invariant). A `Play` carries the whole [`PlayedCard`] so
/// rendering can show the resolved sign / flip identity via `display_text()`.
#[derive(Debug, Clone)]
pub enum Move {
    Draw { side: Player, value: i8, total: i32 },
    Play { side: Player, card: PlayedCard, total: i32 },
    Stand { side: Player, total: i32 },
    Bust { side: Player, total: i32 },
}

/// How a resolved round ended, classified by precedence (bust > filled-table >
/// stand). The producer, `summarize_round`, lands in T002; this type is
/// declared here so [`RoundSummary`] compiles.
#[derive(Debug, Clone)]
pub enum Resolution {
    Bust(Player),
    BothBust,
    FilledTable,
    Stand,
}

/// A resolved round: who won (or a tie), both final totals, and how it ended.
/// Accumulated across the match in [`PlayLog::outcomes`]. Its producer
/// (`summarize_round`) is T002; the fields are `pub` because the overlay
/// renderer (T003) reads them.
#[derive(Debug, Clone)]
pub struct RoundSummary {
    pub outcome: RoundOutcome,
    pub player_total: i32,
    pub opponent_total: i32,
    pub resolution: Resolution,
}

/// Minimal diff seed: the eight per-side lengths/flags plus the two
/// match-lifecycle bits. Totals and card identities are read from the live
/// [`GameState`] in [`moves_since`], not stored here.
struct PlayLogSnapshot {
    p_dealer_len: usize,
    p_played_len: usize,
    p_stood: bool,
    p_bust: bool,
    o_dealer_len: usize,
    o_played_len: usize,
    o_stood: bool,
    o_bust: bool,
    game_over: bool,
    outcome_present: bool,
}

impl PlayLogSnapshot {
    fn of(gs: &GameState) -> Self {
        Self {
            p_dealer_len: gs.player.dealer_row.len(),
            p_played_len: gs.player.played_row.len(),
            p_stood: gs.player.stood,
            p_bust: gs.player.bust,
            o_dealer_len: gs.opponent.dealer_row.len(),
            o_played_len: gs.opponent.played_row.len(),
            o_stood: gs.opponent.stood,
            o_bust: gs.opponent.bust,
            game_over: matches!(gs.game_phase, GamePhase::GameOver { .. }),
            outcome_present: gs.round_outcome.is_some(),
        }
    }
}

/// The moves added to `side` between `prev` (its lengths/flags) and `ps` (its
/// current state), in draw/play → stand → bust order. Only the acting side adds
/// a card, and by the one-card invariant there is at most one new draw *or*
/// play, so this yields that side's move(s) for the diff. Each move's total is
/// `ps.score()` — exact because standing/busting don't change a score and at
/// most one card was added.
fn moves_for_side(
    side: Player,
    prev_dealer_len: usize,
    prev_played_len: usize,
    prev_stood: bool,
    prev_bust: bool,
    ps: &PlayerState,
) -> Vec<Move> {
    let total = ps.score();
    let mut moves = Vec::new();
    for pc in ps.dealer_row.iter().skip(prev_dealer_len) {
        moves.push(Move::Draw { side, value: pc.value, total });
    }
    for pc in ps.played_row.iter().skip(prev_played_len) {
        moves.push(Move::Play { side, card: *pc, total });
    }
    if ps.stood && !prev_stood {
        moves.push(Move::Stand { side, total });
    }
    if ps.bust && !prev_bust {
        moves.push(Move::Bust { side, total });
    }
    moves
}

/// The moves between `prev` and the live `gs`, emitted **player side, then
/// opponent side**, and within a side **draw/play → stand → bust** (the causal
/// order: a draw fills the table → auto-stand → bust; a player flip → the
/// standing opponent busts). Only the acting side adds a card, and only a
/// player flip can bust the other side, so player-then-opponent is always
/// chronologically correct. Pure — unit-tested with hand-built states.
fn moves_since(prev: &PlayLogSnapshot, gs: &GameState) -> Vec<Move> {
    let mut moves = moves_for_side(
        Player::Player,
        prev.p_dealer_len,
        prev.p_played_len,
        prev.p_stood,
        prev.p_bust,
        &gs.player,
    );
    moves.extend(moves_for_side(
        Player::Opponent,
        prev.o_dealer_len,
        prev.o_played_len,
        prev.o_stood,
        prev.o_bust,
        &gs.opponent,
    ));
    moves
}

/// Whether a match has just *restarted* across `prev` → `curr`: the true→false
/// transition of `game_over`. A rematch (`new_game` in place, after game over)
/// is the only in-game `game_over` true→false transition — the same signal
/// banter uses. On it, both the move list and the outcome list clear. Pure.
fn match_restarted(prev: &PlayLogSnapshot, curr: &PlayLogSnapshot) -> bool {
    prev.game_over && !curr.game_over
}

/// Whether a new round has just begun across `prev` → `gs`: all four `gs` rows
/// empty while `prev` had a non-empty row. `setup_next_round` empties all four
/// rows and nothing else does (draws/plays only grow them), so this is a clean
/// signal to clear the move list. The outcome list is untouched. Pure.
fn round_reset(prev: &PlayLogSnapshot, gs: &GameState) -> bool {
    let all_empty = gs.player.dealer_row.is_empty()
        && gs.player.played_row.is_empty()
        && gs.opponent.dealer_row.is_empty()
        && gs.opponent.played_row.is_empty();
    let prev_had_cards = prev.p_dealer_len > 0
        || prev.p_played_len > 0
        || prev.o_dealer_len > 0
        || prev.o_played_len > 0;
    all_empty && prev_had_cards
}

/// The play log for a match: the running round outcomes, the current round's
/// moves, the opponent's name (a rendering label), and the diff seed.
#[derive(Default)]
pub struct PlayLog {
    /// Resolved round outcomes, accumulated across the match.
    outcomes: Vec<RoundSummary>,
    /// The current round's moves; cleared at each new round.
    moves: Vec<Move>,
    /// The opponent's name, used as a side label by the overlay renderer (T003).
    #[allow(dead_code)]
    opponent_name: String,
    /// The diff seed; `None` makes the next `observe` seed silently.
    prev: Option<PlayLogSnapshot>,
}

impl PlayLog {
    /// Reset for a fresh match entry (mirrors `App`'s `prev_banter = None`
    /// seeding): clear both lists, drop the diff seed so the first `observe`
    /// seeds silently, and store the opponent's name. Called at `start_match`
    /// and at `Continue`/resume.
    pub fn reset(&mut self, opponent_name: &str) {
        self.outcomes.clear();
        self.moves.clear();
        self.opponent_name = opponent_name.to_string();
        self.prev = None;
    }

    /// Observe one game tick, updating the log by diffing against the stored
    /// seed. On the first call after a `reset` (seed is `None`) it only seeds,
    /// emitting nothing — a resumed match is never back-logged. Otherwise: a
    /// rematch clears both lists; a new round clears the move list; else the
    /// diff's moves are appended. Always re-stores the seed.
    pub fn observe(&mut self, gs: &GameState) {
        let curr = PlayLogSnapshot::of(gs);
        if let Some(prev) = &self.prev {
            if match_restarted(prev, &curr) {
                self.moves.clear();
                self.outcomes.clear();
            } else if round_reset(prev, gs) {
                self.moves.clear();
            } else {
                self.moves.append(&mut moves_since(prev, gs));
                // T002 stub: when the round outcome first resolves, capture it
                // with `self.outcomes.push(summarize_round(gs))`. Until then the
                // outcome list stays empty; the transition is computed here so
                // T002 is a one-line fill-in.
                let _newly_resolved = !prev.outcome_present && curr.outcome_present;
            }
        }
        self.prev = Some(curr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, FlipKind, PlayedCard};

    /// An empty, pristine game state: no cards on either side, neither stood or
    /// bust, no outcome, live phase.
    fn empty_gs() -> GameState {
        GameState::new()
    }

    /// The snapshot of an empty, pristine state — the usual `prev` for a
    /// single-move diff.
    fn empty_snap() -> PlayLogSnapshot {
        PlayLogSnapshot::of(&empty_gs())
    }

    #[test]
    fn dealer_draw_yields_one_draw_move_with_side_value_and_total() {
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(5), value: 5 });

        let moves = moves_since(&empty_snap(), &gs);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Draw { side, value, total } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(*value, 5);
                assert_eq!(*total, gs.player.score());
                assert_eq!(*total, 5);
            }
            other => panic!("expected a Draw, got {other:?}"),
        }
    }

    #[test]
    fn fixed_value_play_yields_one_play_move_carrying_the_card() {
        let mut gs = empty_gs();
        let card = PlayedCard { card: Card::Plus(4), value: 4 };
        gs.player.played_row.push(card);

        let moves = moves_since(&empty_snap(), &gs);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { side, card: played, total } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(played.card, Card::Plus(4));
                assert_eq!(played.value, 4);
                assert_eq!(*total, gs.player.score());
                assert_eq!(*total, 4);
            }
            other => panic!("expected a Play, got {other:?}"),
        }
    }

    #[test]
    fn plus_minus_committed_at_each_sign_records_the_committed_value() {
        // +3
        let mut pos = empty_gs();
        pos.player.played_row.push(PlayedCard { card: Card::PlusMinus(3), value: 3 });
        let moves = moves_since(&empty_snap(), &pos);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { card, total, .. } => {
                assert_eq!(card.card, Card::PlusMinus(3));
                assert_eq!(card.value, 3);
                assert_eq!(*total, pos.player.score());
                assert_eq!(*total, 3);
            }
            other => panic!("expected a Play, got {other:?}"),
        }

        // -3
        let mut neg = empty_gs();
        neg.player.played_row.push(PlayedCard { card: Card::PlusMinus(3), value: -3 });
        let moves = moves_since(&empty_snap(), &neg);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { card, total, .. } => {
                assert_eq!(card.value, -3);
                assert_eq!(*total, neg.player.score());
                assert_eq!(*total, -3);
            }
            other => panic!("expected a Play, got {other:?}"),
        }
    }

    #[test]
    fn tiebreaker_committed_at_each_sign_records_the_committed_value() {
        // +1T
        let mut pos = empty_gs();
        pos.player.played_row.push(PlayedCard { card: Card::Tiebreaker, value: 1 });
        let moves = moves_since(&empty_snap(), &pos);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { card, total, .. } => {
                assert_eq!(card.card, Card::Tiebreaker);
                assert_eq!(card.value, 1);
                assert_eq!(*total, pos.player.score());
            }
            other => panic!("expected a Play, got {other:?}"),
        }

        // -1T
        let mut neg = empty_gs();
        neg.player.played_row.push(PlayedCard { card: Card::Tiebreaker, value: -1 });
        let moves = moves_since(&empty_snap(), &neg);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { card, total, .. } => {
                assert_eq!(card.value, -1);
                assert_eq!(*total, neg.player.score());
            }
            other => panic!("expected a Play, got {other:?}"),
        }
    }

    #[test]
    fn flip_play_yields_one_play_with_flip_identity_and_post_flip_total() {
        // The player already holds a Dealer(4); prev captured it. Now a
        // TwoFour flip lands, flipping the 4 to -4 on the board.
        let mut prev_gs = empty_gs();
        prev_gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(4), value: 4 });
        let prev = PlayLogSnapshot::of(&prev_gs);

        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(4), value: -4 });
        gs.player.played_row.push(PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 });

        let moves = moves_since(&prev, &gs);
        // Only the new played flip is a move; the already-present dealer card
        // (now flipped) is not re-drawn.
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Play { side, card, total } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(card.card, Card::Flip(FlipKind::TwoFour));
                assert_eq!(*total, gs.player.score());
                assert_eq!(*total, -4); // post-flip total
            }
            other => panic!("expected a Play, got {other:?}"),
        }
    }

    #[test]
    fn stand_yields_one_stand_move() {
        let mut gs = empty_gs();
        gs.player.stood = true;

        let moves = moves_since(&empty_snap(), &gs);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Stand { side, total } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(*total, gs.player.score());
            }
            other => panic!("expected a Stand, got {other:?}"),
        }
    }

    #[test]
    fn bust_yields_one_bust_move() {
        let mut gs = empty_gs();
        gs.player.bust = true;

        let moves = moves_since(&empty_snap(), &gs);
        assert_eq!(moves.len(), 1);
        match &moves[0] {
            Move::Bust { side, total } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(*total, gs.player.score());
            }
            other => panic!("expected a Bust, got {other:?}"),
        }
    }

    #[test]
    fn one_diff_draw_fills_stands_busts_yields_draw_then_stand_then_bust() {
        // prev: two dealer cards (19), live, not stood/bust.
        let mut prev_gs = empty_gs();
        prev_gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        prev_gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(9), value: 9 });
        let prev = PlayLogSnapshot::of(&prev_gs);

        // curr: a third dealer card lands (24), filling the table -> auto-stand
        // -> bust, all in one diff.
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(9), value: 9 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(5), value: 5 });
        gs.player.stood = true;
        gs.player.bust = true;

        let moves = moves_since(&prev, &gs);
        assert_eq!(moves.len(), 3);
        assert!(matches!(moves[0], Move::Draw { value: 5, total: 24, .. }));
        assert!(matches!(moves[1], Move::Stand { total: 24, .. }));
        assert!(matches!(moves[2], Move::Bust { total: 24, .. }));
    }

    #[test]
    fn player_flip_busting_a_standing_opponent_yields_player_play_then_opponent_bust() {
        // prev: opponent has stood (24 on the board), not yet bust; player has
        // played nothing.
        let mut prev_gs = empty_gs();
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(4), value: 4 });
        prev_gs.opponent.stood = true;
        let prev = PlayLogSnapshot::of(&prev_gs);

        // curr: the player plays a flip; the standing opponent is now bust.
        let mut gs = empty_gs();
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(4), value: 4 });
        gs.opponent.stood = true;
        gs.opponent.bust = true;
        gs.player.played_row.push(PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 });

        let moves = moves_since(&prev, &gs);
        assert_eq!(moves.len(), 2);
        match &moves[0] {
            Move::Play { side, card, .. } => {
                assert_eq!(*side, Player::Player);
                assert_eq!(card.card, Card::Flip(FlipKind::TwoFour));
            }
            other => panic!("expected a player Play first, got {other:?}"),
        }
        match &moves[1] {
            Move::Bust { side, total } => {
                assert_eq!(*side, Player::Opponent);
                assert_eq!(*total, gs.opponent.score());
            }
            other => panic!("expected an opponent Bust second, got {other:?}"),
        }
    }

    #[test]
    fn no_change_diff_yields_nothing() {
        let gs = empty_gs();
        let prev = PlayLogSnapshot::of(&gs);
        assert!(moves_since(&prev, &gs).is_empty());
    }

    #[test]
    fn observe_from_none_seeds_silently_and_emits_nothing() {
        let mut log = PlayLog::default();
        let mut gs = empty_gs();
        // Even with cards already on the board, the first observe only seeds.
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(7), value: 7 });
        gs.player.stood = true;
        log.observe(&gs);
        assert!(log.moves.is_empty());
        assert!(log.outcomes.is_empty());
        assert!(log.prev.is_some());
    }

    #[test]
    fn completed_round_then_empty_rows_clears_moves() {
        let mut log = PlayLog::default();

        // Seed on an empty pristine state.
        log.observe(&empty_gs());

        // A move lands and is recorded.
        let mut mid = empty_gs();
        mid.player.dealer_row.push(PlayedCard { card: Card::Dealer(8), value: 8 });
        mid.round_outcome = Some(RoundOutcome::PlayerWon);
        log.observe(&mid);
        assert_eq!(log.moves.len(), 1);

        // The next round empties all four rows -> the move list clears.
        let next = empty_gs();
        log.observe(&next);
        assert!(log.moves.is_empty());
    }

    #[test]
    fn game_over_true_to_false_clears_both_lists() {
        let mut log = PlayLog::default();

        // Seed on a game-over state.
        let mut over = empty_gs();
        over.game_phase = GamePhase::GameOver { winner: Player::Player };
        log.observe(&over);

        // Stuff both lists with prior-match content.
        log.moves.push(Move::Stand { side: Player::Player, total: 12 });
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::PlayerWon,
            player_total: 20,
            opponent_total: 18,
            resolution: Resolution::Stand,
        });

        // The rematch begins in place (game_over true -> false): both clear.
        let live = empty_gs();
        log.observe(&live);
        assert!(log.moves.is_empty());
        assert!(log.outcomes.is_empty());
    }
}
