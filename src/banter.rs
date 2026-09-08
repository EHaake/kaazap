//! Opponent banter: which line an opponent says, and when. Pure logic + static
//! content, no dependency on rendering (the panel drawer lives in `portrait.rs`).
//!
//! `App` diffs successive [`BanterSnapshot`]s through [`banter_event`] to decide
//! which line class fires — mirroring `audio.rs`'s snapshot/diff pattern (the
//! engine itself says nothing). Lines are picked from a per-opponent [`BanterSet`]
//! (or [`GENERIC`] for the default / an unknown id) via [`pick`], which never
//! repeats the currently-shown line back-to-back. Spec 017.

use crate::game::{GamePhase, GameState, RoundOutcome};
use crate::player::Player;

/// A banter-worthy event, from the **opponent's** point of view (RoundWin = the
/// opponent won the round; MatchWin = the opponent won the match). `MatchStart`
/// is set at seeding, never produced by [`banter_event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BanterEvent {
    MatchStart,
    RoundWin,
    RoundLoss,
    RoundTie,
    OpponentBust,
    PlayerBust,
    MatchWin,
    MatchLoss,
}

/// A minimal snapshot of the banter-relevant facts. `App` diffs successive
/// snapshots to decide which line fires. (No `PartialEq` — the diff runs through
/// [`banter_event`], and `RoundOutcome` isn't `Eq`.)
#[derive(Debug, Clone, Copy)]
pub struct BanterSnapshot {
    o_bust: bool,
    p_bust: bool,
    outcome: Option<RoundOutcome>,
    game_over: bool,
    opp_won_game: bool,
}

impl BanterSnapshot {
    pub fn of(gs: &GameState) -> Self {
        let (game_over, opp_won_game) = match gs.game_phase {
            GamePhase::GameOver { winner } => (true, matches!(winner, Player::Opponent)),
            _ => (false, false),
        };
        Self {
            o_bust: gs.opponent.bust,
            p_bust: gs.player.bust,
            outcome: gs.round_outcome,
            game_over,
            opp_won_game,
        }
    }
}

/// The banter event for the transition `prev` → `curr`, or `None` if nothing
/// new fires. Ordered precedence (spec 017 §5): **match end** (the closing line)
/// beats a **bust** (the vivid reaction) beats a **round outcome** (the plain
/// resolution) — a bust and the outcome it causes surface in one tick, and the
/// final blow surfaces the bust and game-over together. Pure — unit-tested with
/// no game loop.
pub fn banter_event(prev: &BanterSnapshot, curr: &BanterSnapshot) -> Option<BanterEvent> {
    if curr.game_over && !prev.game_over {
        return Some(if curr.opp_won_game {
            BanterEvent::MatchWin
        } else {
            BanterEvent::MatchLoss
        });
    }
    if curr.o_bust && !prev.o_bust {
        return Some(BanterEvent::OpponentBust);
    }
    if curr.p_bust && !prev.p_bust {
        return Some(BanterEvent::PlayerBust);
    }
    if curr.outcome.is_some() && prev.outcome.is_none() {
        return Some(match curr.outcome {
            Some(RoundOutcome::OpponentWon) => BanterEvent::RoundWin,
            Some(RoundOutcome::PlayerWon) => BanterEvent::RoundLoss,
            _ => BanterEvent::RoundTie,
        });
    }
    None
}

/// A voice: one line pool per event class, all `&'static`. The five *repeatable*
/// classes (`round_win`/`round_loss`/`round_tie`/`opponent_bust`/`player_bust` —
/// events that can fire in consecutive rounds) carry ≥2 lines each so [`pick`]
/// always has an alternative and can honor the no-back-to-back-repeat rule; the
/// three once-per-match classes may have one.
pub struct BanterSet {
    pub match_start: &'static [&'static str],
    pub round_win: &'static [&'static str],
    pub round_loss: &'static [&'static str],
    pub round_tie: &'static [&'static str],
    pub opponent_bust: &'static [&'static str],
    pub player_bust: &'static [&'static str],
    pub match_win: &'static [&'static str],
    pub match_loss: &'static [&'static str],
}

/// The line pool for `ev` within `set`.
pub fn lines_for(set: &'static BanterSet, ev: BanterEvent) -> &'static [&'static str] {
    match ev {
        BanterEvent::MatchStart => set.match_start,
        BanterEvent::RoundWin => set.round_win,
        BanterEvent::RoundLoss => set.round_loss,
        BanterEvent::RoundTie => set.round_tie,
        BanterEvent::OpponentBust => set.opponent_bust,
        BanterEvent::PlayerBust => set.player_bust,
        BanterEvent::MatchWin => set.match_win,
        BanterEvent::MatchLoss => set.match_loss,
    }
}

/// A random line from `lines`, never equal to `last` when `lines.len() > 1`
/// (the no-back-to-back-repeat rule, spec 017 §4). A single-line pool returns
/// that sole line. Pure — `rng` is injected so the choice is unit-testable.
pub fn pick(lines: &[&'static str], last: Option<&str>, rng: &mut impl rand::Rng) -> &'static str {
    if lines.len() <= 1 {
        return lines[0];
    }
    loop {
        let candidate = lines[rng.random_range(0..lines.len())];
        if Some(candidate) != last {
            return candidate;
        }
    }
}

/// The neutral, characterless voice — used for the default opponent and as the
/// fallback for any unknown id. Never blank, never a roster voice.
const GENERIC: BanterSet = BanterSet {
    match_start: &["Let's play."],
    round_win: &["That one's mine.", "A point to me."],
    round_loss: &["Your round.", "Take it."],
    round_tie: &["A draw.", "Dead even."],
    opponent_bust: &["Too far.", "Over I go."],
    player_bust: &["You went over.", "Busted."],
    match_win: &["I win."],
    match_loss: &["Well played."],
};

/// Placeholder roster voice (T002 authors the real lines and the other nine).
const GREEB: BanterSet = BanterSet {
    match_start: &["Here goes!"],
    round_win: &["Got one!", "Hah, mine!"],
    round_loss: &["Aw, yours.", "Fine, fine."],
    round_tie: &["A tie?", "Even, huh."],
    opponent_bust: &["Too much!", "Oops, over."],
    player_bust: &["You popped!", "Over you go!"],
    match_win: &["I did it!"],
    match_loss: &["Good game."],
};

/// The voice for opponent `id`: the roster set, or [`GENERIC`] for `"default"`
/// and any unknown id.
pub fn banter_for(id: &str) -> &'static BanterSet {
    match id {
        "greeb" => &GREEB,
        _ => &GENERIC,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(
        o_bust: bool,
        p_bust: bool,
        outcome: Option<RoundOutcome>,
        game_over: bool,
        opp_won_game: bool,
    ) -> BanterSnapshot {
        BanterSnapshot { o_bust, p_bust, outcome, game_over, opp_won_game }
    }

    const EMPTY: BanterSnapshot = BanterSnapshot {
        o_bust: false,
        p_bust: false,
        outcome: None,
        game_over: false,
        opp_won_game: false,
    };

    #[test]
    fn game_over_beats_co_firing_bust_and_outcome() {
        // The final blow: opponent-loses-match-by-busting surfaces game_over,
        // the bust, and the (player-won) outcome in one tick. Match end wins.
        let curr = snap(true, false, Some(RoundOutcome::PlayerWon), true, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::MatchLoss));

        // Opponent wins the match on the final round.
        let curr = snap(false, true, Some(RoundOutcome::OpponentWon), true, true);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::MatchWin));
    }

    #[test]
    fn bust_beats_the_outcome_it_causes() {
        // Opponent busts, losing the (non-final) round: reads the bust, not RoundLoss.
        let curr = snap(true, false, Some(RoundOutcome::PlayerWon), false, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::OpponentBust));

        // Player busts, losing the round to the opponent: reads the bust, not RoundWin.
        let curr = snap(false, true, Some(RoundOutcome::OpponentWon), false, false);
        assert_eq!(banter_event(&EMPTY, &curr), Some(BanterEvent::PlayerBust));
    }

    #[test]
    fn outcome_maps_to_the_right_round_event() {
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::OpponentWon), false, false)),
            Some(BanterEvent::RoundWin)
        );
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::PlayerWon), false, false)),
            Some(BanterEvent::RoundLoss)
        );
        assert_eq!(
            banter_event(&EMPTY, &snap(false, false, Some(RoundOutcome::Tied), false, false)),
            Some(BanterEvent::RoundTie)
        );
    }

    #[test]
    fn stale_or_no_change_yields_none() {
        // No change at all.
        assert_eq!(banter_event(&EMPTY, &EMPTY), None);
        // Outcome already set in prev (round already resolved) — not newly set.
        let resolved = snap(false, false, Some(RoundOutcome::OpponentWon), false, false);
        assert_eq!(banter_event(&resolved, &resolved), None);
        // Bust already true in prev — not new.
        let busted = snap(true, false, None, false, false);
        assert_eq!(banter_event(&busted, &busted), None);
        // Game-over already true in prev — not new.
        let over = snap(false, false, None, true, true);
        assert_eq!(banter_event(&over, &over), None);
    }

    #[test]
    fn pick_never_repeats_last_for_multi_line_pool() {
        let pool = GENERIC.round_win;
        assert!(pool.len() >= 2);
        let mut rng = rand::rng();
        for _ in 0..1000 {
            let got = pick(pool, Some(pool[0]), &mut rng);
            assert_ne!(got, pool[0]);
        }
    }

    #[test]
    fn pick_returns_sole_line_for_single_line_pool() {
        let pool = GENERIC.match_start;
        assert_eq!(pool.len(), 1);
        let mut rng = rand::rng();
        // Even asked to avoid the only line, it returns it.
        assert_eq!(pick(pool, Some(pool[0]), &mut rng), pool[0]);
    }

    #[test]
    fn generic_classes_non_empty_and_repeatables_have_floor() {
        let g = &GENERIC;
        for class in [
            g.match_start,
            g.round_win,
            g.round_loss,
            g.round_tie,
            g.opponent_bust,
            g.player_bust,
            g.match_win,
            g.match_loss,
        ] {
            assert!(!class.is_empty(), "GENERIC has a blank class");
        }
        for class in [g.round_win, g.round_loss, g.round_tie, g.opponent_bust, g.player_bust] {
            assert!(class.len() >= 2, "repeatable class must carry >= 2 lines");
        }
    }

    #[test]
    fn every_generic_line_fits_the_panel() {
        use crate::portrait::BANTER_MAX_WIDTH;
        let g = &GENERIC;
        for class in [
            g.match_start,
            g.round_win,
            g.round_loss,
            g.round_tie,
            g.opponent_bust,
            g.player_bust,
            g.match_win,
            g.match_loss,
        ] {
            for line in class {
                assert!(
                    line.chars().count() <= BANTER_MAX_WIDTH,
                    "line {line:?} exceeds BANTER_MAX_WIDTH"
                );
            }
        }
    }

    #[test]
    fn banter_for_default_and_unknown_resolve_to_generic() {
        // `default` and any unknown id both resolve to GENERIC (compared by
        // content — BanterSet isn't PartialEq, and a `const`'s address isn't
        // stable across reference sites). match_start is a sufficient proxy.
        assert_eq!(banter_for("default").match_start, GENERIC.match_start);
        assert_eq!(banter_for("nonexistent-id").match_start, GENERIC.match_start);
        // The placeholder roster id resolves to a different voice.
        assert_ne!(banter_for("greeb").match_start, GENERIC.match_start);
    }
}
