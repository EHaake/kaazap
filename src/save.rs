//! Mid-match save & resume. Persists the in-progress match as JSON in the
//! platform data dir, mirroring `settings.rs`'s graceful-fallback behavior
//! (any read error → `None`, never a panic or a crash).
//!
//! `GameState` can't derive serde directly — `GamePhase` carries a
//! non-serializable `Instant` — so a small `SavedGame` projection stands in,
//! carrying a schema version and re-arming the opponent's think-timer on
//! load. There is no RNG or deck state to persist: already-drawn cards live
//! in `PlayerState`, and future draws use a fresh thread RNG regardless (see
//! `plan.md`).

use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    OPPONENT_THINKING_TIME_MS,
    card::{Card, DEFAULT_SIDE_DECK},
    game::{GamePhase, GameState, RoundOutcome},
    opponent::{DEFAULT_OPPONENT, opponent_by_id},
    player::PlayerState,
};

/// Bump when the on-disk shape changes incompatibly; a file whose version
/// doesn't match is then discarded rather than mis-read.
const SAVE_VERSION: u32 = 1;

/// The persisted form of an in-progress match — independent of the runtime
/// types, versioned, and free of `GamePhase`'s `Instant`.
#[derive(Serialize, Deserialize)]
struct SavedGame {
    version: u32,
    player: PlayerState,
    opponent: PlayerState,
    phase: SavedPhase,
    round_outcome: Option<RoundOutcome>,
    /// Which opponent is being faced, so resume restores the same profile
    /// (name + AI threshold + deck). Older saves (pre-roster) lack the field
    /// and default to the neutral "Opponent" they were actually played
    /// against — no version bump, no data loss.
    #[serde(default = "default_opponent_id")]
    opponent_id: String,
    /// The player's built side deck for this match, snapshotted so a resume
    /// deals from the deck the match began with — editing your deck from the
    /// menu afterward changes future matches, not an in-progress saved one.
    /// Older saves (pre-deck-builder) lack the field; an empty deck falls back
    /// to the default pool, exactly what those matches were dealt from — no
    /// version bump, no data loss.
    #[serde(default)]
    player_deck: Vec<Card>,
}

/// The opponent id a save without one resumes against: the default opponent,
/// which is exactly who every pre-roster save was played against.
fn default_opponent_id() -> String {
    DEFAULT_OPPONENT.id.to_string()
}

/// `GamePhase` minus the `Instant`, and minus `GameOver` (a finished match is
/// never saved). `OpponentThinking` drops its deadline here and gets a fresh
/// one on load.
#[derive(Serialize, Deserialize)]
enum SavedPhase {
    PlayerTurn,
    AwaitingSignChoice { hand_index: usize },
    OpponentThinking,
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
}

/// Project a live match to its saved form. `None` for a finished match
/// (`GameOver`) — the signal to clear the save rather than write one.
fn to_saved(game: &GameState) -> Option<SavedGame> {
    let phase = match game.game_phase {
        GamePhase::PlayerTurn => SavedPhase::PlayerTurn,
        GamePhase::AwaitingSignChoice { hand_index } => {
            SavedPhase::AwaitingSignChoice { hand_index }
        }
        GamePhase::OpponentThinking { .. } => SavedPhase::OpponentThinking,
        GamePhase::OpponentTurn => SavedPhase::OpponentTurn,
        GamePhase::RoundEnd => SavedPhase::RoundEnd,
        GamePhase::AwaitingNextRound => SavedPhase::AwaitingNextRound,
        GamePhase::GameOver { .. } => return None,
    };
    Some(SavedGame {
        version: SAVE_VERSION,
        player: game.player.clone(),
        opponent: game.opponent.clone(),
        phase,
        round_outcome: game.round_outcome,
        opponent_id: game.opponent_profile.id.to_string(),
        player_deck: game.player_deck.clone(),
    })
}

/// Rebuild a live match from its saved form. `OpponentThinking` gets a fresh
/// think-deadline so the pause resumes rather than being skipped.
fn from_saved(saved: SavedGame) -> GameState {
    let game_phase = match saved.phase {
        SavedPhase::PlayerTurn => GamePhase::PlayerTurn,
        SavedPhase::AwaitingSignChoice { hand_index } => {
            GamePhase::AwaitingSignChoice { hand_index }
        }
        SavedPhase::OpponentThinking => GamePhase::OpponentThinking {
            until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
        },
        SavedPhase::OpponentTurn => GamePhase::OpponentTurn,
        SavedPhase::RoundEnd => GamePhase::RoundEnd,
        SavedPhase::AwaitingNextRound => GamePhase::AwaitingNextRound,
    };
    GameState {
        player: saved.player,
        opponent: saved.opponent,
        game_phase,
        round_outcome: saved.round_outcome,
        // Restore the opponent by id; an unknown id (older or hand-edited
        // save) falls back to the default profile rather than failing.
        opponent_profile: opponent_by_id(&saved.opponent_id).unwrap_or(DEFAULT_OPPONENT),
        // The match's player deck: the saved deck, or the default pool for a
        // pre-deck-builder save (empty field) — the deck those were dealt from.
        player_deck: if saved.player_deck.is_empty() {
            DEFAULT_SIDE_DECK.to_vec()
        } else {
            saved.player_deck
        },
    }
}

/// `<data_dir>/saves/savegame.json`, if a data dir is resolvable.
fn save_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "kaazap")
        .map(|dirs| dirs.data_dir().join("saves").join("savegame.json"))
}

/// Write the in-progress match — or clear the save if the match is over.
/// Best-effort: a missing data dir or an unwritable path is swallowed, never
/// fatal (a save you can't write isn't worth crashing a game over).
pub fn save(game: &GameState) {
    let Some(saved) = to_saved(game) else {
        clear();
        return;
    };
    let Some(path) = save_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(&saved) {
        let _ = fs::write(path, json);
    }
}

/// Load a resumable match, or `None` on any error — missing, unreadable,
/// malformed, or an incompatible version.
pub fn load() -> Option<GameState> {
    let text = fs::read_to_string(save_path()?).ok()?;
    from_json(&text)
}

/// The filesystem-free core of `load`, so the fallback and the version check
/// are testable without touching disk.
fn from_json(text: &str) -> Option<GameState> {
    let saved: SavedGame = serde_json::from_str(text).ok()?;
    (saved.version == SAVE_VERSION).then(|| from_saved(saved))
}

/// Is there a loadable (valid, current-version) save? Drives the menu's
/// Continue item — validity, not mere file presence, gates it.
pub fn exists() -> bool {
    load().is_some()
}

/// Remove the save file. Absence is success.
pub fn clear() {
    if let Some(path) = save_path() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use std::mem::discriminant;

    use super::*;
    use crate::{
        card::{Card, FlipKind, PlayedCard},
        player::Player,
    };

    // A player mid-round: a dealer draw, a played side card, a partial hand
    // (with a hole), and some round wins — the fields a save must preserve.
    fn a_player(name: &str, rounds_won: usize) -> PlayerState {
        PlayerState {
            name: name.to_string(),
            dealer_row: vec![PlayedCard { card: Card::Dealer(7), value: 7 }],
            played_row: vec![PlayedCard { card: Card::Plus(4), value: 4 }],
            hand: vec![
                Some(Card::Minus(2)),
                None,
                Some(Card::PlusMinus(3)),
                Some(Card::Flip(FlipKind::TwoFour)),
            ],
            stood: true,
            bust: false,
            rounds_won,
        }
    }

    fn a_game(phase: GamePhase) -> GameState {
        GameState {
            player: a_player("You", 1),
            opponent: a_player("Foe", 2),
            game_phase: phase,
            round_outcome: Some(RoundOutcome::PlayerWon),
            opponent_profile: DEFAULT_OPPONENT,
            player_deck: DEFAULT_SIDE_DECK.to_vec(),
        }
    }

    // Compare the saveable content of two players. `PlayedCard` isn't `Eq`,
    // so the rows are mapped to (card, value) tuples (`Card` is `Eq`).
    fn assert_player_eq(a: &PlayerState, b: &PlayerState) {
        let rows = |p: &PlayerState| {
            let map = |r: &[PlayedCard]| r.iter().map(|pc| (pc.card, pc.value)).collect::<Vec<_>>();
            (map(&p.dealer_row), map(&p.played_row))
        };
        assert_eq!(a.name, b.name);
        assert_eq!(a.hand, b.hand); // Card: PartialEq
        assert_eq!(rows(a), rows(b));
        assert_eq!(
            (a.stood, a.bust, a.rounds_won),
            (b.stood, b.bust, b.rounds_won)
        );
    }

    #[test]
    fn round_trip_preserves_the_match_position() {
        for phase in [
            GamePhase::PlayerTurn,
            GamePhase::AwaitingSignChoice { hand_index: 2 },
            GamePhase::OpponentTurn,
            GamePhase::RoundEnd,
            GamePhase::AwaitingNextRound,
        ] {
            let g = a_game(phase);
            let json = serde_json::to_string(&to_saved(&g).unwrap()).unwrap();
            let g2 = from_json(&json).expect("valid save loads");

            assert_player_eq(&g.player, &g2.player);
            assert_player_eq(&g.opponent, &g2.opponent);
            assert_eq!(discriminant(&g.game_phase), discriminant(&g2.game_phase));
            // The pending sign-choice index must survive *exactly*, not just
            // the variant — a wrong index would commit the wrong card (or a
            // no-op) when the player answers the prompt on resume.
            if let GamePhase::AwaitingSignChoice { hand_index } = g.game_phase {
                assert!(
                    matches!(g2.game_phase, GamePhase::AwaitingSignChoice { hand_index: h } if h == hand_index)
                );
            }
            assert!(matches!(g2.round_outcome, Some(RoundOutcome::PlayerWon)));
        }
    }

    #[test]
    fn round_trip_preserves_the_opponent() {
        use crate::opponent::OPPONENTS;
        // A save against a specific roster opponent resumes against the same
        // one — id and AI threshold intact, not the default.
        let mut g = a_game(GamePhase::PlayerTurn);
        g.opponent_profile = OPPONENTS[3];
        let json = serde_json::to_string(&to_saved(&g).unwrap()).unwrap();
        let g2 = from_json(&json).expect("valid save loads");
        assert_eq!(g2.opponent_profile.id, OPPONENTS[3].id);
        assert_eq!(
            g2.opponent_profile.stand_threshold,
            OPPONENTS[3].stand_threshold
        );
    }

    #[test]
    fn a_save_without_an_opponent_id_resumes_against_the_default() {
        use crate::opponent::OPPONENTS;
        // Start from a save naming a roster opponent, then strip the field to
        // mimic a pre-roster save. serde(default) fills it, resolving to the
        // default "Opponent" — exactly who such saves were played against.
        let mut g = a_game(GamePhase::PlayerTurn);
        g.opponent_profile = OPPONENTS[3];
        let mut val: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&to_saved(&g).unwrap()).unwrap()).unwrap();
        assert_eq!(val["opponent_id"], serde_json::json!(OPPONENTS[3].id)); // it was written
        val.as_object_mut().unwrap().remove("opponent_id");
        let g2 = from_json(&val.to_string()).expect("a field-less save still loads");
        assert_eq!(g2.opponent_profile.id, DEFAULT_OPPONENT.id);
    }

    #[test]
    fn round_trip_preserves_the_player_deck() {
        // A distinctive built deck (disjoint from the default pool) must
        // survive the trip, so a resumed match deals from the same deck.
        let mut g = a_game(GamePhase::PlayerTurn);
        g.player_deck = vec![Card::Plus(1), Card::Minus(1), Card::PlusMinus(2), Card::Plus(3)];
        let json = serde_json::to_string(&to_saved(&g).unwrap()).unwrap();
        let g2 = from_json(&json).expect("valid save loads");
        assert_eq!(g2.player_deck, g.player_deck);
    }

    #[test]
    fn a_save_without_a_player_deck_resumes_with_the_default() {
        // A pre-deck-builder save lacks the field; serde(default) leaves it
        // empty, which resolves to the default pool those matches were dealt
        // from — no version bump, no data loss.
        let g = a_game(GamePhase::PlayerTurn);
        let mut val: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&to_saved(&g).unwrap()).unwrap()).unwrap();
        val.as_object_mut().unwrap().remove("player_deck");
        let g2 = from_json(&val.to_string()).expect("a field-less save still loads");
        assert_eq!(g2.player_deck, DEFAULT_SIDE_DECK.to_vec());
    }

    #[test]
    fn opponent_thinking_round_trips_with_a_rearmed_timer() {
        // The Instant can't serialize; it must survive the trip as a fresh
        // OpponentThinking, never a panic.
        let g = a_game(GamePhase::OpponentThinking { until: Instant::now() });
        let json = serde_json::to_string(&to_saved(&g).unwrap()).unwrap();
        let g2 = from_json(&json).expect("valid save loads");
        assert!(matches!(g2.game_phase, GamePhase::OpponentThinking { .. }));
    }

    #[test]
    fn a_finished_match_is_not_saved() {
        // GameOver → None is the "clear the save" signal.
        let g = a_game(GamePhase::GameOver { winner: Player::Player });
        assert!(to_saved(&g).is_none());
    }

    #[test]
    fn corrupt_or_incompatible_json_loads_as_none() {
        assert!(from_json("").is_none());
        assert!(from_json("not json").is_none());
        assert!(from_json("{}").is_none()); // missing fields
        assert!(from_json("[1,2,3]").is_none());

        // A structurally valid save whose version doesn't match is discarded,
        // not mis-parsed.
        let g = a_game(GamePhase::PlayerTurn);
        let mut val: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&to_saved(&g).unwrap()).unwrap()).unwrap();
        val["version"] = serde_json::json!(SAVE_VERSION + 1);
        assert!(from_json(&val.to_string()).is_none());
    }
}
