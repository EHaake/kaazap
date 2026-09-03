//! Opponent roster: the named opponents you can face, each with its own
//! difficulty (stand threshold) and side deck. The single pre-roster opponent
//! is preserved as [`DEFAULT_OPPONENT`] for the engine default and older
//! saves; [`OPPONENTS`] is the selectable roster. First slice of the campaign
//! (subsystem A) — see `specs/007-opponent-roster`. This const is the source
//! of truth for the roster; `docs/opponents.md` explains how difficulty is
//! tuned and snapshots the current values.

use crate::{
    STAND_THRESHOLD,
    card::{Card, DEFAULT_SIDE_DECK, FlipKind},
};

/// How an opponent's AI plays — the deterministic policy archetype it uses to
/// decide Hit / Stand / play-a-card, on top of its stand threshold and deck.
/// Board-aware (reads the player's total) as of spec 010; see
/// `decide_opponent_move` in `game.rs` and `docs/opponents.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiStrategy {
    /// Sensible threshold play plus the core board-aware fix — stand once it's
    /// already beating a stood player. The baseline.
    Basic,
    /// Pushes for higher totals and chases a stood player hard, accepting bust
    /// risk.
    Aggressive,
    /// Stands earlier; only chases a stood player when it can win *safely*,
    /// never over-hitting into an avoidable bust.
    Cautious,
    /// Targets the smallest total that beats the player (minimizing bust risk)
    /// and plays the tiebreaker to steal a tie.
    Calculating,
}

/// A selectable opponent: identity + flavor, plus the knobs that make it play
/// distinctly — its stand threshold, side deck, AI strategy, and error rate.
/// All fields are `Copy`, so a profile is stored by value on `GameState` with
/// no lifetimes.
#[derive(Debug, Clone, Copy)]
pub struct OpponentProfile {
    /// Stable key persisted in the save file (resolve with [`opponent_by_id`]).
    pub id: &'static str,
    /// Shown on the board and the select screen.
    pub name: &'static str,
    /// Difficulty label for the select screen.
    pub difficulty: &'static str,
    /// One-line flavor for the select row.
    pub blurb: &'static str,
    /// The AI stands once its (non-winning) score reaches this — higher is
    /// more aggressive (pushes for bigger totals, busts more).
    pub stand_threshold: usize,
    /// The side deck this opponent draws its hand from.
    pub side_deck: &'static [Card],
    /// How its AI plays — the board-aware policy archetype (spec 010).
    pub strategy: AiStrategy,
    /// Chance (`0.0..=1.0`) of an imperfect move on any decision — the "human
    /// error" dash. Higher for easy opponents, `0.0` for the master and the
    /// default (so the default stays deterministic for tests).
    pub misplay: f32,
}

/// The neutral default opponent — the pre-roster "Opponent" behavior
/// (threshold [`STAND_THRESHOLD`], the standard [`DEFAULT_SIDE_DECK`]). Used by
/// `GameState::new()`/`Default`, the tests, and as the fallback when a save
/// names an unknown opponent or predates the roster. Deliberately **not** in
/// [`OPPONENTS`], so it is never itself a campaign choice.
pub const DEFAULT_OPPONENT: OpponentProfile = OpponentProfile {
    id: "default",
    name: "Opponent",
    difficulty: "—",
    blurb: "",
    stand_threshold: STAND_THRESHOLD,
    side_deck: &DEFAULT_SIDE_DECK,
    strategy: AiStrategy::Basic,
    misplay: 0.0, // deterministic baseline — keeps the AI tests deterministic
};

/// The selectable roster, ordered easiest → hardest. Names and blurbs are
/// original flavor (no Star Wars trademarks — see `DECISIONS.md`) and, with
/// the thresholds and decks, are tunable balance data (a balance pass is a
/// later cross-cutting item in the campaign epic).
pub const OPPONENTS: [OpponentProfile; 5] = [
    OpponentProfile {
        id: "greeb",
        name: "Greeb",
        difficulty: "Rookie",
        blurb: "Green and eager — folds early, and slips.",
        stand_threshold: 15,
        side_deck: &[
            Card::Plus(1),
            Card::Plus(2),
            Card::Plus(3),
            Card::Minus(1),
            Card::Minus(2),
            Card::Minus(3),
            Card::Plus(1),
            Card::Minus(1),
            Card::Plus(2),
            Card::Minus(2),
        ],
        strategy: AiStrategy::Basic,
        misplay: 0.25, // a rookie — makes real mistakes
    },
    OpponentProfile {
        id: "vessa",
        name: "Vessa Korr",
        difficulty: "Scrapper",
        blurb: "A scrapper who pushes hard — chases the win, risks the bust.",
        stand_threshold: 16,
        side_deck: &[
            Card::Plus(2),
            Card::Plus(4),
            Card::Minus(2),
            Card::Minus(4),
            Card::PlusMinus(1),
            Card::PlusMinus(2),
            Card::Plus(1),
            Card::Minus(1),
            Card::Plus(3),
            Card::Minus(3),
        ],
        strategy: AiStrategy::Aggressive,
        misplay: 0.15,
    },
    OpponentProfile {
        id: "toran",
        name: "Old Toran",
        difficulty: "Veteran",
        blurb: "Balanced and patient. Knows the game.",
        stand_threshold: STAND_THRESHOLD, // 17 — the baseline
        side_deck: &DEFAULT_SIDE_DECK,
        strategy: AiStrategy::Cautious,
        misplay: 0.10,
    },
    OpponentProfile {
        id: "rix",
        name: "Rix Vandal",
        difficulty: "Ace",
        blurb: "An ace who counts every point — takes the exact play to win.",
        stand_threshold: 18,
        side_deck: &[
            Card::PlusMinus(6),
            Card::PlusMinus(3),
            Card::PlusMinus(1),
            Card::Minus(4),
            Card::Minus(2),
            Card::Plus(4),
            Card::Plus(2),
            Card::Flip(FlipKind::TwoFour),
            Card::Flip(FlipKind::ThreeSix),
            Card::Tiebreaker,
        ],
        strategy: AiStrategy::Calculating,
        misplay: 0.05,
    },
    OpponentProfile {
        id: "magistrate",
        name: "The Magistrate",
        difficulty: "Master",
        blurb: "Relentless. Pushes to the edge and seldom slips.",
        stand_threshold: 19,
        side_deck: &[
            Card::PlusMinus(6),
            Card::PlusMinus(6),
            Card::PlusMinus(3),
            Card::PlusMinus(1),
            Card::Minus(4),
            Card::Minus(4),
            Card::Minus(2),
            Card::Flip(FlipKind::TwoFour),
            Card::Flip(FlipKind::ThreeSix),
            Card::Tiebreaker,
        ],
        strategy: AiStrategy::Calculating,
        misplay: 0.0, // the master — essentially never slips
    },
];

/// Resolve a saved opponent `id` back to its profile. Unknown ids (older or
/// hand-edited saves) return `None`; callers fall back to [`DEFAULT_OPPONENT`].
pub fn opponent_by_id(id: &str) -> Option<OpponentProfile> {
    OPPONENTS.iter().copied().find(|o| o.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HAND_SIZE;

    #[test]
    fn every_roster_deck_can_fill_a_hand() {
        // `deal_hand` draws HAND_SIZE distinct cards, so a deck shorter than
        // that would silently deal a short hand.
        for o in OPPONENTS {
            assert!(
                o.side_deck.len() >= HAND_SIZE,
                "{} has a deck of {} (< {HAND_SIZE})",
                o.id,
                o.side_deck.len()
            );
        }
    }

    #[test]
    fn every_roster_card_is_in_the_universe() {
        // ALL_SIDE_CARDS is documented as the complete side-card universe (the
        // deck-builder grid and spec C's pool draw from it). Guard that claim
        // against a roster deck — or a future opponent — playing a card the
        // collection couldn't represent.
        use crate::card::ALL_SIDE_CARDS;
        for o in OPPONENTS {
            for card in o.side_deck {
                assert!(
                    ALL_SIDE_CARDS.contains(card),
                    "{} plays {card:?}, which isn't in ALL_SIDE_CARDS",
                    o.id
                );
            }
        }
    }

    #[test]
    fn roster_ids_are_unique_and_names_nonempty() {
        for (i, a) in OPPONENTS.iter().enumerate() {
            assert!(!a.name.is_empty(), "{} has an empty name", a.id);
            for b in &OPPONENTS[i + 1..] {
                assert_ne!(a.id, b.id, "duplicate opponent id: {}", a.id);
            }
        }
    }

    #[test]
    fn opponent_by_id_resolves_known_and_rejects_unknown() {
        assert_eq!(opponent_by_id("greeb").map(|o| o.name), Some("Greeb"));
        assert!(opponent_by_id("nope").is_none());
        // The default opponent is intentionally not in the roster.
        assert!(opponent_by_id(DEFAULT_OPPONENT.id).is_none());
    }

    #[test]
    fn roster_runs_easy_to_hard_by_threshold() {
        let thresholds: Vec<usize> = OPPONENTS.iter().map(|o| o.stand_threshold).collect();
        let mut sorted = thresholds.clone();
        sorted.sort_unstable();
        assert_eq!(thresholds, sorted, "roster should be ordered easiest→hardest");
    }

    #[test]
    fn misplay_rates_are_valid_and_the_default_is_deterministic() {
        for o in OPPONENTS {
            assert!(
                (0.0..=1.0).contains(&o.misplay),
                "{} misplay {} out of 0.0..=1.0",
                o.id,
                o.misplay
            );
        }
        // The default opponent must never misplay, so the AI tests (which build
        // against it) stay deterministic.
        assert_eq!(DEFAULT_OPPONENT.misplay, 0.0);
        assert_eq!(DEFAULT_OPPONENT.strategy, AiStrategy::Basic);
        // The master is flawless; the rookie slips the most.
        let master = OPPONENTS.iter().find(|o| o.id == "magistrate").unwrap();
        let rookie = OPPONENTS.iter().find(|o| o.id == "greeb").unwrap();
        assert_eq!(master.misplay, 0.0);
        assert!(rookie.misplay > master.misplay);
    }
}
