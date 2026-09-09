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

/// Classify a resolved round from the live [`GameState`] at the
/// `round_outcome` `None`→`Some` transition (rows still intact). Precedence
/// (plan §6): both busted → [`Resolution::BothBust`]; exactly one busted →
/// [`Resolution::Bust`] of that side; else a side filled the table
/// (`table_full`, i.e. `table_card_count() >= MAX_TABLE_CARDS`) →
/// [`Resolution::FilledTable`]; else both stood → [`Resolution::Stand`]. Both
/// totals are the raw `score()`s (a busted side's over-20 value is kept —
/// informative). This precedence is a design decision, not spec-settled, so it
/// is pinned by tests. Must be called on a resolved round (`round_outcome`
/// `Some`). Pure.
fn summarize_round(gs: &GameState) -> RoundSummary {
    let player_bust = gs.player.bust;
    let opponent_bust = gs.opponent.bust;
    let resolution = if player_bust && opponent_bust {
        Resolution::BothBust
    } else if player_bust {
        Resolution::Bust(Player::Player)
    } else if opponent_bust {
        Resolution::Bust(Player::Opponent)
    } else if gs.player.table_full() || gs.opponent.table_full() {
        Resolution::FilledTable
    } else {
        Resolution::Stand
    };
    RoundSummary {
        outcome: gs
            .round_outcome
            .expect("summarize_round called on a resolved round"),
        player_total: gs.player.score(),
        opponent_total: gs.opponent.score(),
        resolution,
    }
}

/// The play log for a match: the running round outcomes, the current round's
/// moves, the opponent's name (a rendering label), and the diff seed.
#[derive(Default)]
pub struct PlayLog {
    /// Resolved round outcomes, accumulated across the match.
    outcomes: Vec<RoundSummary>,
    /// The current round's moves; cleared at each new round.
    moves: Vec<Move>,
    /// The opponent's name, used as a side label by the overlay renderer.
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
                // When the round outcome first resolves (outcome_present
                // false→true), capture the round summary. Rows are still
                // intact at this transition; they clear on the next NextRound.
                let newly_resolved = !prev.outcome_present && curr.outcome_present;
                if newly_resolved {
                    self.outcomes.push(summarize_round(gs));
                }
            }
        }
        self.prev = Some(curr);
    }

    /// The side label used in rendered lines: the player is always "You", the
    /// opponent is named (the label stored at `reset`).
    fn side_label(&self, side: Player) -> &str {
        match side {
            Player::Player => "You",
            Player::Opponent => &self.opponent_name,
        }
    }

    /// One rendered line for a resolved round: the winner (or a tie), both
    /// final totals, and how the round resolved. Punctuation is cosmetic; the
    /// substrings (winner/name, both totals, resolution) are what matters.
    fn outcome_line(&self, s: &RoundSummary) -> String {
        let winner = match s.outcome {
            RoundOutcome::PlayerWon => "You win".to_string(),
            RoundOutcome::OpponentWon => format!("{} wins", self.opponent_name),
            RoundOutcome::Tied => "Tie".to_string(),
        };
        let resolution = match &s.resolution {
            Resolution::Bust(Player::Player) => "You bust".to_string(),
            Resolution::Bust(Player::Opponent) => format!("{} busts", self.opponent_name),
            Resolution::BothBust => "both bust".to_string(),
            Resolution::FilledTable => "filled table".to_string(),
            Resolution::Stand => "stand".to_string(),
        };
        format!(
            "{winner} — You {} / {} {} ({resolution})",
            s.player_total, self.opponent_name, s.opponent_total,
        )
    }

    /// One rendered line for a visible move: the acting side, the value or card
    /// text (a play uses `PlayedCard::display_text()`), and the resulting total.
    fn move_line(&self, m: &Move) -> String {
        match m {
            Move::Draw { side, value, total } => {
                format!("{} drew {value} → {total}", self.side_label(*side))
            }
            Move::Play { side, card, total } => {
                format!(
                    "{} played {} → {total}",
                    self.side_label(*side),
                    card.display_text(),
                )
            }
            Move::Stand { side, total } => {
                format!("{} stood at {total}", self.side_label(*side))
            }
            Move::Bust { side, total } => {
                format!("{} bust at {total}", self.side_label(*side))
            }
        }
    }

    /// Build the play-log overlay's content as a line list (same shape as
    /// `overlay.rs`'s `read_text_from_file`: line 0 = title, centered
    /// downstream; the rest left-aligned content). A "Round outcomes" section
    /// (one line per resolved round) then a blank separator then a "This round"
    /// section (one line per move, oldest → newest).
    ///
    /// Overflow (plan §8): if the whole thing would exceed
    /// `inner_height_budget`, the title, the entire round-outcomes section, the
    /// two section headers and the blank are always kept; only the move lines
    /// are trimmed, keeping the **most recent** that fit and dropping the
    /// oldest. A round is bounded (~26 move lines), so this stays a small pure
    /// trim — no pagination.
    pub fn render_lines(&self, inner_height_budget: usize) -> Vec<String> {
        // Each section shows an indented placeholder when it is otherwise empty
        // — at match start (and right after a new round / rematch) the log has
        // no outcomes and/or no moves, and a bare header reads unfinished. The
        // placeholders are part of the fixed (never-trimmed) section, since they
        // only appear when the section has nothing else to show.
        let outcome_lines: Vec<String> = if self.outcomes.is_empty() {
            vec!["  (none yet)".to_string()]
        } else {
            self.outcomes.iter().map(|s| self.outcome_line(s)).collect()
        };
        let move_lines: Vec<String> = self.moves.iter().map(|m| self.move_line(m)).collect();
        let move_placeholder = move_lines.is_empty();

        // The fixed part that is never trimmed: title, outcomes header, every
        // outcome line (or the empty placeholder), the blank separator, the
        // moves header, and — when there are no moves — the moves placeholder.
        let fixed_count = 4 + outcome_lines.len() + usize::from(move_placeholder);
        let move_budget = inner_height_budget.saturating_sub(fixed_count);
        let dropped = move_lines.len().saturating_sub(move_budget);

        let mut lines = Vec::with_capacity(fixed_count + move_lines.len());
        lines.push("Play Log".to_string());
        lines.push("Round outcomes".to_string());
        lines.extend(outcome_lines);
        lines.push(String::new());
        lines.push("This round".to_string());
        if move_placeholder {
            lines.push("  (no moves yet)".to_string());
        } else {
            lines.extend(move_lines.into_iter().skip(dropped));
        }
        lines
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
        // prev: opponent stood at 16 — a 4-card sits flipped to -4 on the
        // board (20 + 10 + (-4) → 16), not yet bust; player has played nothing.
        let mut prev_gs = empty_gs();
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        prev_gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(4), value: -4 });
        prev_gs.opponent.stood = true;
        let prev = PlayLogSnapshot::of(&prev_gs);

        // curr: the player plays a TwoFour flip, flipping the opponent's -4 back
        // to +4 (16 → 24) — the standing opponent's own card is mutated, so the
        // recorded bust total is genuinely the post-flip 24, not a tautology.
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
                assert_eq!(*total, 24); // the flip-induced post-flip total
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

        // The next round empties all four rows -> the move list clears, but the
        // round-outcomes list persists across the new round.
        let next = empty_gs();
        log.observe(&next);
        assert!(log.moves.is_empty());
        assert!(!log.outcomes.is_empty());
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

    // --- summarize_round classification (plan §6 precedence) ---

    #[test]
    fn summarize_round_classifies_a_lone_player_bust() {
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(5), value: 5 }); // 25
        gs.player.bust = true;
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(8), value: 8 }); // 18
        gs.opponent.stood = true;
        gs.round_outcome = Some(RoundOutcome::OpponentWon);

        let s = summarize_round(&gs);
        assert!(matches!(s.resolution, Resolution::Bust(Player::Player)));
        assert!(matches!(s.outcome, RoundOutcome::OpponentWon));
        assert_eq!(s.player_total, 25); // raw over-20 total kept
        assert_eq!(s.opponent_total, 18);
    }

    #[test]
    fn summarize_round_classifies_a_lone_opponent_bust() {
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(9), value: 9 }); // 19
        gs.player.stood = true;
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(3), value: 3 }); // 23
        gs.opponent.bust = true;
        gs.round_outcome = Some(RoundOutcome::PlayerWon);

        let s = summarize_round(&gs);
        assert!(matches!(s.resolution, Resolution::Bust(Player::Opponent)));
        assert!(matches!(s.outcome, RoundOutcome::PlayerWon));
        assert_eq!(s.player_total, 19);
        assert_eq!(s.opponent_total, 23);
    }

    #[test]
    fn summarize_round_classifies_a_both_bust_tie() {
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(5), value: 5 }); // 25
        gs.player.bust = true;
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(2), value: 2 }); // 22
        gs.opponent.bust = true;
        gs.round_outcome = Some(RoundOutcome::Tied);

        let s = summarize_round(&gs);
        // BothBust takes precedence over either lone Bust.
        assert!(matches!(s.resolution, Resolution::BothBust));
        assert!(matches!(s.outcome, RoundOutcome::Tied));
        assert_eq!(s.player_total, 25);
        assert_eq!(s.opponent_total, 22);
    }

    #[test]
    fn summarize_round_classifies_a_full_table_auto_stand_with_no_bust() {
        let mut gs = empty_gs();
        // Twelve dealer 1s fill the table (MAX_TABLE_CARDS) at 12 — auto-stand,
        // no bust.
        for _ in 0..crate::MAX_TABLE_CARDS {
            gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(1), value: 1 });
        }
        gs.player.stood = true;
        assert!(gs.player.table_full());
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 }); // 10
        gs.opponent.stood = true;
        gs.round_outcome = Some(RoundOutcome::PlayerWon);

        let s = summarize_round(&gs);
        // No bust on either side, but a side filled the table.
        assert!(matches!(s.resolution, Resolution::FilledTable));
        assert!(matches!(s.outcome, RoundOutcome::PlayerWon));
        assert_eq!(s.player_total, 12);
        assert_eq!(s.opponent_total, 10);
    }

    #[test]
    fn summarize_round_classifies_a_plain_double_stand() {
        let mut gs = empty_gs();
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.player.dealer_row.push(PlayedCard { card: Card::Dealer(8), value: 8 }); // 18
        gs.player.stood = true;
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        gs.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 }); // 20
        gs.opponent.stood = true;
        gs.round_outcome = Some(RoundOutcome::OpponentWon);

        let s = summarize_round(&gs);
        // No bust, no full table -> a plain stand.
        assert!(matches!(s.resolution, Resolution::Stand));
        assert!(matches!(s.outcome, RoundOutcome::OpponentWon));
        assert_eq!(s.player_total, 18);
        assert_eq!(s.opponent_total, 20);
    }

    // --- observe: outcome accumulation end-to-end ---

    #[test]
    fn observe_over_a_finished_round_appends_exactly_one_summary() {
        let mut log = PlayLog::default();
        log.observe(&empty_gs()); // seed silently

        // A round resolves: player stands at 20, opponent stands at 18.
        let mut done = empty_gs();
        done.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        done.player.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        done.player.stood = true;
        done.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(10), value: 10 });
        done.opponent.dealer_row.push(PlayedCard { card: Card::Dealer(8), value: 8 });
        done.opponent.stood = true;
        done.round_outcome = Some(RoundOutcome::PlayerWon);
        log.observe(&done);

        assert_eq!(log.outcomes.len(), 1);
        assert!(matches!(log.outcomes[0].resolution, Resolution::Stand));
        assert_eq!(log.outcomes[0].player_total, 20);
        assert_eq!(log.outcomes[0].opponent_total, 18);

        // Observing again with the outcome still present is no transition —
        // nothing further is appended.
        log.observe(&done);
        assert_eq!(log.outcomes.len(), 1);
    }

    #[test]
    fn observe_appends_no_summary_while_the_round_is_live() {
        let mut log = PlayLog::default();
        log.observe(&empty_gs()); // seed silently

        // A move lands but the round has not resolved (round_outcome None).
        let mut live = empty_gs();
        live.player.dealer_row.push(PlayedCard { card: Card::Dealer(9), value: 9 });
        log.observe(&live);

        assert_eq!(log.moves.len(), 1);
        assert!(log.outcomes.is_empty());
    }

    // --- reset lifecycle (carryover from the T001 review) ---

    #[test]
    fn reset_clears_a_populated_log_and_stores_the_opponent_name() {
        let mut log = PlayLog::default();
        log.moves.push(Move::Stand { side: Player::Player, total: 15 });
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::PlayerWon,
            player_total: 20,
            opponent_total: 17,
            resolution: Resolution::Stand,
        });
        log.prev = Some(empty_snap());

        log.reset("Jarael");

        assert!(log.moves.is_empty());
        assert!(log.outcomes.is_empty());
        assert!(log.prev.is_none()); // next observe seeds silently
        assert_eq!(log.opponent_name, "Jarael");
    }

    #[test]
    fn match_restart_wins_when_both_restart_and_reset_conditions_hold() {
        let mut log = PlayLog::default();

        // Seed on a game-over state that had cards on the board.
        let mut over = empty_gs();
        over.game_phase = GamePhase::GameOver { winner: Player::Player };
        over.player.dealer_row.push(PlayedCard { card: Card::Dealer(8), value: 8 });
        log.observe(&over);

        // Stuff both lists with prior-match content.
        log.moves.push(Move::Stand { side: Player::Player, total: 12 });
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::PlayerWon,
            player_total: 20,
            opponent_total: 18,
            resolution: Resolution::Stand,
        });

        // curr: game_over true->false (match_restarted) AND all rows empty while
        // prev had cards (round_reset) — both conditions hold at once.
        let live = empty_gs();
        log.observe(&live);

        // match_restarted wins: BOTH lists clear. Had round_reset won, the
        // outcome list would have survived.
        assert!(log.moves.is_empty());
        assert!(log.outcomes.is_empty());
    }

    // --- render_lines (T003) ---

    /// A populated log: one resolved round plus one move of each kind, with a
    /// named opponent.
    fn populated_log() -> PlayLog {
        let mut log = PlayLog::default();
        log.opponent_name = "Jarael".to_string();
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::PlayerWon,
            player_total: 20,
            opponent_total: 18,
            resolution: Resolution::Stand,
        });
        log.moves.push(Move::Draw { side: Player::Player, value: 7, total: 7 });
        log.moves.push(Move::Play {
            side: Player::Opponent,
            card: PlayedCard { card: Card::PlusMinus(3), value: -3 },
            total: 15,
        });
        log.moves.push(Move::Stand { side: Player::Player, total: 20 });
        log.moves.push(Move::Bust { side: Player::Opponent, total: 24 });
        log
    }

    #[test]
    fn render_lines_puts_title_first_then_both_headers_in_order() {
        let log = populated_log();
        let lines = log.render_lines(100);

        assert_eq!(lines[0], "Play Log");
        let outcomes_at = lines.iter().position(|l| l == "Round outcomes").unwrap();
        let moves_at = lines.iter().position(|l| l == "This round").unwrap();
        assert_eq!(outcomes_at, 1); // header immediately after the title
        assert!(outcomes_at < moves_at); // "Round outcomes" precedes "This round"
    }

    #[test]
    fn render_lines_labels_the_player_you_and_names_the_opponent() {
        let log = populated_log();
        let lines = log.render_lines(100);
        let joined = lines.join("\n");

        assert!(joined.contains("You"));
        assert!(joined.contains("Jarael"));
    }

    #[test]
    fn render_lines_outcome_line_carries_winner_both_totals_and_resolution() {
        let log = populated_log();
        let lines = log.render_lines(100);
        // The outcome line sits just after the "Round outcomes" header.
        let idx = lines.iter().position(|l| l == "Round outcomes").unwrap() + 1;
        let line = &lines[idx];

        assert!(line.contains("You win")); // winner
        assert!(line.contains("20")); // player total
        assert!(line.contains("18")); // opponent total
        assert!(line.contains("stand")); // resolution
    }

    #[test]
    fn render_lines_tie_and_bust_resolution_render_the_expected_substrings() {
        let mut log = PlayLog::default();
        log.opponent_name = "Jarael".to_string();
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::Tied,
            player_total: 25,
            opponent_total: 22,
            resolution: Resolution::Bust(Player::Opponent),
        });
        let lines = log.render_lines(100);
        let idx = lines.iter().position(|l| l == "Round outcomes").unwrap() + 1;
        let line = &lines[idx];

        assert!(line.contains("Tie")); // tie, not a winner
        assert!(line.contains("25"));
        assert!(line.contains("22"));
        assert!(line.contains("Jarael busts")); // which side busted
    }

    #[test]
    fn render_lines_move_lines_carry_side_value_or_card_and_total() {
        let log = populated_log();
        let lines = log.render_lines(100);
        let start = lines.iter().position(|l| l == "This round").unwrap() + 1;
        let move_lines = &lines[start..];
        assert_eq!(move_lines.len(), 4);

        // Draw: side + drawn value + total.
        assert!(move_lines[0].contains("You"));
        assert!(move_lines[0].contains("7"));

        // Play: side + display_text (a committed -3) + total.
        assert!(move_lines[1].contains("Jarael"));
        assert!(move_lines[1].contains("-3")); // display_text sign resolution
        assert!(move_lines[1].contains("15"));

        // Stand: side + total.
        assert!(move_lines[2].contains("You"));
        assert!(move_lines[2].contains("20"));

        // Bust: side + total.
        assert!(move_lines[3].contains("Jarael"));
        assert!(move_lines[3].contains("24"));
    }

    #[test]
    fn render_lines_play_flip_move_shows_the_flip_identity() {
        let mut log = PlayLog::default();
        log.opponent_name = "Jarael".to_string();
        log.moves.push(Move::Play {
            side: Player::Player,
            card: PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 },
            total: -4,
        });
        let lines = log.render_lines(100);
        let start = lines.iter().position(|l| l == "This round").unwrap() + 1;

        // display_text() renders a flip as its identity label, not a signed value.
        let expected = PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 }.display_text();
        assert!(lines[start].contains(&expected));
        assert!(lines[start].contains("-4")); // resulting total
    }

    #[test]
    fn render_lines_over_budget_keeps_headers_outcomes_and_most_recent_moves() {
        let mut log = PlayLog::default();
        log.opponent_name = "Jarael".to_string();
        log.outcomes.push(RoundSummary {
            outcome: RoundOutcome::PlayerWon,
            player_total: 20,
            opponent_total: 18,
            resolution: Resolution::Stand,
        });
        // Five distinct draws: oldest value 1 → newest value 5.
        for v in 1..=5 {
            log.moves.push(Move::Draw { side: Player::Player, value: v, total: v as i32 });
        }

        // Fixed part is 4 lines (title + outcomes header + 1 outcome + blank +
        // moves header = 5, actually): title, "Round outcomes", 1 outcome line,
        // blank, "This round" = 5. Budget 7 leaves room for 2 move lines.
        let lines = log.render_lines(7);

        // Title, both headers and the outcome line all survive.
        assert_eq!(lines[0], "Play Log");
        assert!(lines.iter().any(|l| l == "Round outcomes"));
        assert!(lines.iter().any(|l| l == "This round"));
        assert!(lines.iter().any(|l| l.contains("You win")));

        let start = lines.iter().position(|l| l == "This round").unwrap() + 1;
        let move_lines = &lines[start..];
        // Only the two most recent moves fit.
        assert_eq!(move_lines.len(), 2);
        // The oldest moves (drew 1, 2, 3) are dropped.
        assert!(!move_lines.iter().any(|l| l.contains("drew 1")));
        assert!(!move_lines.iter().any(|l| l.contains("drew 2")));
        assert!(!move_lines.iter().any(|l| l.contains("drew 3")));
        // The most recent two (drew 4, drew 5) are kept, oldest-first.
        assert!(move_lines[0].contains("drew 4"));
        assert!(move_lines[1].contains("drew 5"));
    }

    #[test]
    fn render_lines_empty_log_shows_a_placeholder_under_each_header() {
        // At match start both lists are empty — each header is followed by its
        // indented placeholder rather than nothing.
        let log = PlayLog::default();
        let lines = log.render_lines(100);

        assert_eq!(lines[0], "Play Log");
        let outcomes_at = lines.iter().position(|l| l == "Round outcomes").unwrap();
        let moves_at = lines.iter().position(|l| l == "This round").unwrap();
        assert_eq!(lines[outcomes_at + 1], "  (none yet)");
        assert_eq!(lines[moves_at + 1], "  (no moves yet)");
    }

    #[test]
    fn render_lines_does_not_panic_at_tiny_budgets() {
        // Both placeholders are part of the fixed section, so a budget below the
        // fixed count must saturate rather than panic.
        let log = PlayLog::default();
        let _ = log.render_lines(0);
        // At a budget below the fixed count the placeholders still survive —
        // they live in the never-trimmed fixed part, so a regression that moved
        // them into the trimmable move region would otherwise pass silently.
        let lines = log.render_lines(4);
        assert!(lines.iter().any(|l| l == "  (none yet)"));
        assert!(lines.iter().any(|l| l == "  (no moves yet)"));
    }
}
