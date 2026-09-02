//! Opponent roster: the named opponents you can face, each with its own
//! difficulty (stand threshold) and side deck. The single pre-roster opponent
//! is preserved as [`DEFAULT_OPPONENT`] for the engine default and older
//! saves; [`OPPONENTS`] is the selectable roster. First slice of the campaign
//! (subsystem A) — see `specs/007-opponent-roster`.

use crate::{
    STAND_THRESHOLD,
    card::{Card, DEFAULT_SIDE_DECK, FlipKind},
};

/// A selectable opponent: identity + flavor, plus the two knobs that make it
/// play distinctly — its stand threshold and its side deck. All fields are
/// `Copy`, so a profile is stored by value on `GameState` with no lifetimes.
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
        blurb: "Cautious and green — folds early.",
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
    },
    OpponentProfile {
        id: "vessa",
        name: "Vessa Korr",
        difficulty: "Scrapper",
        blurb: "Plays the odds, rarely overreaches.",
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
    },
    OpponentProfile {
        id: "toran",
        name: "Old Toran",
        difficulty: "Veteran",
        blurb: "Balanced and patient. Knows the game.",
        stand_threshold: STAND_THRESHOLD, // 17 — the baseline
        side_deck: &DEFAULT_SIDE_DECK,
    },
    OpponentProfile {
        id: "rix",
        name: "Rix Vandal",
        difficulty: "Ace",
        blurb: "Aggressive — squeezes out every point.",
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
}
