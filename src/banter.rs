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

/// Greeb (Rookie) — green and eager: jittery, over-excited, apologetic when he
/// loses, and yelps when he busts. Surprised by his own wins.
const GREEB: BanterSet = BanterSet {
    match_start: &["Here goes nothing!", "Okay, okay, ready!"],
    round_win: &["I got one! Whoa!", "Did that just work?"],
    round_loss: &["Aw, shucks.", "Sorry, sorry!"],
    round_tie: &["A tie? Phew.", "Even? Okay!"],
    opponent_bust: &["Yikes, too much!", "No no no, over!"],
    player_bust: &["Oh! You popped!", "Phew, not me!"],
    match_win: &["I actually won!"],
    match_loss: &["Aw, well played."],
};

/// Dax Runo (Greenhorn) — a cocky kid: brash trash-talk, struts on a win, makes
/// excuses when it goes wrong, and busts big.
const DAX: BanterSet = BanterSet {
    match_start: &["Watch and learn.", "This'll be quick."],
    round_win: &["Too easy.", "Boom. Called it."],
    round_loss: &["Lucky. So lucky.", "Bah, whatever."],
    round_tie: &["Tch, a tie.", "Even? Boring."],
    opponent_bust: &["Ah, come on!", "Blew it big."],
    player_bust: &["Ha! Nice one, kid.", "Told you. Bust."],
    match_win: &["Not even close."],
    match_loss: &["Rematch. Now."],
};

/// Vessa Korr (Scrapper) — street-hard and defiant: takes the hit and swings
/// back, never done even when she's down.
const VESSA: BanterSet = BanterSet {
    match_start: &["Come on then.", "Let's scrap."],
    round_win: &["That's how.", "Chalk it up."],
    round_loss: &["Fine. I'm not done.", "You'll pay for that."],
    round_tie: &["Nobody blinks.", "Locked up."],
    opponent_bust: &["Pushed too hard.", "Ah, hell."],
    player_bust: &["Ha, you cracked.", "Down you go."],
    match_win: &["Still standing."],
    match_loss: &["Next time's mine."],
};

/// Nima Sarn (Broker) — cool and mercantile: everything is a transaction, wins
/// are profit, losses a minor cost. Never ruffled.
const NIMA: BanterSet = BanterSet {
    match_start: &["Let's talk terms.", "Credits on the line."],
    round_win: &["Profit.", "Into the ledger."],
    round_loss: &["A minor cost.", "I'll recoup it."],
    round_tie: &["We break even.", "No margin lost."],
    opponent_bust: &["Overspent.", "A bad investment."],
    player_bust: &["That'll cost you.", "Poor accounting."],
    match_win: &["Business is good."],
    match_loss: &["I've paid worse."],
};

/// Old Toran (Veteran) — dry, calm, wry: he's seen it all, understated, with a
/// faint teaching tone.
const TORAN: BanterSet = BanterSet {
    match_start: &["Sit. Let's play.", "Been at this awhile."],
    round_win: &["Patience wins.", "As it goes."],
    round_loss: &["Nicely done.", "You've got an eye."],
    round_tie: &["Even hands.", "Happens."],
    opponent_bust: &["Ah, greedy of me.", "Should've held."],
    player_bust: &["Over you go.", "Reached too far."],
    match_win: &["Age and cunning."],
    match_loss: &["Well earned, that."],
};

/// Brakka (Bruiser) — big, booming brute: blunt bravado, dares and taunts, and
/// laughs off his own busts.
const BRAKKA: BanterSet = BanterSet {
    match_start: &["Try to keep up!", "Sit down, small fry."],
    round_win: &["Crushed it!", "Ha! Feel that?"],
    round_loss: &["Pah, a scratch.", "Enjoy it, runt."],
    round_tie: &["A standoff!", "Nobody flinched!"],
    opponent_bust: &["Bah! Too greedy!", "Ha! Blew past it!"],
    player_bust: &["Ha! Splat!", "Too big for ya!"],
    match_win: &["Smashed you flat!"],
    match_loss: &["Bah! You got lucky!"],
};

/// Rix Vandal (Ace) — precise and clinical: talks in odds and math, arrogant,
/// with open disdain for sloppy play.
const RIX: BanterSet = BanterSet {
    match_start: &["The odds favor me.", "Precision wins."],
    round_win: &["Calculated.", "As predicted."],
    round_loss: &["A rounding error.", "Statistically rare."],
    round_tie: &["A null result.", "Perfectly balanced."],
    opponent_bust: &["Miscalculated.", "An error. Rare."],
    player_bust: &["Predictable.", "Sloppy math."],
    match_win: &["The math held."],
    match_loss: &["A variance. Once."],
};

/// Kesh Varn (Duelist) — sharp and dangerous: a duelist's menace and honor,
/// clipped threats, blade imagery.
const KESH: BanterSet = BanterSet {
    match_start: &["Blades out.", "Guard yourself."],
    round_win: &["First blood.", "A clean cut."],
    round_loss: &["A fair touch.", "Well struck."],
    round_tie: &["Blade to blade.", "We cross even."],
    opponent_bust: &["My edge slipped.", "Cut too deep."],
    player_bust: &["You overreached.", "Your guard broke."],
    match_win: &["The edge was mine."],
    match_loss: &["A worthy blade."],
};

/// The Magistrate (Master) — imperious cold authority: pronounces rather than
/// talks, in the language of law, judgment, and sentence.
const MAGISTRATE: BanterSet = BanterSet {
    match_start: &["Court is in session.", "State your case."],
    round_win: &["So ruled.", "The verdict stands."],
    round_loss: &["Noted for appeal.", "A minor objection."],
    round_tie: &["Case adjourned.", "No ruling yet."],
    opponent_bust: &["I overstepped.", "A misjudgment."],
    player_bust: &["Guilty. Overreach.", "Sentence: bust."],
    match_win: &["The law prevails."],
    match_loss: &["An unusual verdict."],
};

/// The Sovereign (Kingpin) — regal and glacial: minimal words, utterly
/// untouchable, the house always wins; even the rare loss is dismissed.
const SOVEREIGN: BanterSet = BanterSet {
    match_start: &["Begin.", "The house awaits."],
    round_win: &["Naturally.", "As it must be."],
    round_loss: &["Amusing.", "A trifle."],
    round_tie: &["Inconsequential.", "It matters not."],
    opponent_bust: &["A rare indulgence.", "How careless."],
    player_bust: &["Beneath me.", "Of course."],
    match_win: &["The house wins."],
    match_loss: &["Enjoy it. Briefly."],
};

/// The voice for opponent `id`: the roster set, or [`GENERIC`] for `"default"`
/// and any unknown id.
pub fn banter_for(id: &str) -> &'static BanterSet {
    match id {
        "greeb" => &GREEB,
        "dax" => &DAX,
        "vessa" => &VESSA,
        "nima" => &NIMA,
        "toran" => &TORAN,
        "brakka" => &BRAKKA,
        "rix" => &RIX,
        "kesh" => &KESH,
        "magistrate" => &MAGISTRATE,
        "sovereign" => &SOVEREIGN,
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

    // ---- T002: the ten roster voices --------------------------------------

    /// The ten roster ids, in tasks.md order.
    const ROSTER_IDS: [&str; 10] = [
        "greeb", "dax", "vessa", "nima", "toran", "brakka", "rix", "kesh",
        "magistrate", "sovereign",
    ];

    /// All eight classes of a set, in a fixed order.
    fn classes(set: &'static BanterSet) -> [&'static [&'static str]; 8] {
        [
            set.match_start,
            set.round_win,
            set.round_loss,
            set.round_tie,
            set.opponent_bust,
            set.player_bust,
            set.match_win,
            set.match_loss,
        ]
    }

    /// The five *repeatable* classes (must carry >= 2 distinct lines).
    fn repeatable(set: &'static BanterSet) -> [&'static [&'static str]; 5] {
        [set.round_win, set.round_loss, set.round_tie, set.opponent_bust, set.player_bust]
    }

    /// GENERIC plus the ten roster sets — every set the game can voice.
    fn all_sets() -> Vec<(&'static str, &'static BanterSet)> {
        let mut v = vec![("default", &GENERIC as &'static BanterSet)];
        for id in ROSTER_IDS {
            v.push((id, banter_for(id)));
        }
        v
    }

    /// A concatenation proxy for whole-set equality (BanterSet isn't PartialEq).
    fn set_fingerprint(set: &'static BanterSet) -> String {
        classes(set).iter().flat_map(|c| c.iter().copied()).collect::<Vec<_>>().join("|")
    }

    #[test]
    fn every_line_of_every_set_fits_the_panel() {
        use crate::portrait::BANTER_MAX_WIDTH;
        for (id, set) in all_sets() {
            for class in classes(set) {
                for line in class {
                    assert!(
                        line.chars().count() <= BANTER_MAX_WIDTH,
                        "{id}: line {line:?} exceeds BANTER_MAX_WIDTH"
                    );
                }
            }
        }
    }

    #[test]
    fn every_set_non_empty_with_repeatable_floor() {
        for (id, set) in all_sets() {
            for class in classes(set) {
                assert!(!class.is_empty(), "{id} has a blank class");
            }
            for class in repeatable(set) {
                assert!(class.len() >= 2, "{id}: repeatable class must carry >= 2 lines");
            }
        }
    }

    #[test]
    fn every_class_of_every_set_has_distinct_lines() {
        // Carried from the T001 review: `pick` loops forever on an all-equal
        // pool, so no class may hold a duplicate variant.
        for (id, set) in all_sets() {
            for class in classes(set) {
                for i in 0..class.len() {
                    for j in (i + 1)..class.len() {
                        assert_ne!(
                            class[i], class[j],
                            "{id}: duplicate line {:?} within a class",
                            class[i]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn roster_sets_are_pairwise_distinct_and_differ_from_generic() {
        let generic = set_fingerprint(&GENERIC);
        let roster: Vec<(&str, String)> =
            ROSTER_IDS.iter().map(|&id| (id, set_fingerprint(banter_for(id)))).collect();
        for (id, fp) in &roster {
            assert_ne!(*fp, generic, "{id} matches GENERIC");
        }
        for i in 0..roster.len() {
            for j in (i + 1)..roster.len() {
                assert_ne!(
                    roster[i].1, roster[j].1,
                    "{} and {} share a voice",
                    roster[i].0, roster[j].0
                );
            }
        }
    }

    #[test]
    fn no_line_is_shared_across_any_two_sets() {
        // Stronger than fingerprint distinctness: no individual line may recur
        // in another set (including GENERIC).
        let sets = all_sets();
        for a in 0..sets.len() {
            for b in (a + 1)..sets.len() {
                let (id_a, set_a) = sets[a];
                let (id_b, set_b) = sets[b];
                for class_a in classes(set_a) {
                    for line in class_a {
                        for class_b in classes(set_b) {
                            assert!(
                                !class_b.contains(line),
                                "{id_a} and {id_b} share the line {line:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn banter_for_maps_each_roster_id_to_a_non_generic_set() {
        let generic = set_fingerprint(&GENERIC);
        for id in ROSTER_IDS {
            assert_ne!(
                set_fingerprint(banter_for(id)),
                generic,
                "{id} resolves to GENERIC"
            );
        }
        assert_eq!(set_fingerprint(banter_for("default")), generic);
        assert_eq!(set_fingerprint(banter_for("nonexistent-id")), generic);
    }
}
