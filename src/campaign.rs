//! The campaign map's data model: the graph of planets you travel Outer Rim →
//! Core, and the per-run state tracking which opponents you've beaten. The map
//! graph is `const` data (like `opponent.rs`'s roster); all derived state —
//! cleared, unlocked, next opponent — is computed from the beaten set plus the
//! graph, so there is one source of truth and nothing to keep in sync.
//!
//! Subsystem D of the campaign epic, scoped to navigation + progression
//! structure — the economy (credits/rewards) is spec C and lives elsewhere.
//! See `specs/009-campaign-map`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::stats::RunStats;

/// A node on the campaign map: a planet holding an ordered list of opponents,
/// unlocked once every planet in `requires` is cleared. All fields are `Copy`
/// (`&'static` + `f32`), so `PLANETS` stays a plain `const` roster with no
/// lifetimes to thread.
#[derive(Debug, Clone, Copy)]
pub struct Planet {
    pub id: &'static str,
    pub name: &'static str,
    pub region: &'static str,
    pub blurb: &'static str,
    /// Normalized bird's-eye position, each `0.0..=1.0`; `fx` trends rim→core.
    pub fx: f32,
    pub fy: f32,
    /// Opponent ids (resolve via [`crate::opponent::opponent_by_id`]), in the
    /// order they're played.
    pub opponents: &'static [&'static str],
    /// Planet ids that must all be cleared for this planet to unlock (empty =
    /// a start node, unlocked from the beginning).
    pub requires: &'static [&'static str],
}

/// The planet a fresh run begins on (the only one unlocked at the start).
/// A documentation/test anchor — production derives "unlocked" from an empty
/// `requires`, never from this constant (kept `pub` so it isn't flagged as
/// dead code, since only tests reference it today).
pub const START_PLANET: &str = "cinder";

/// The campaign map, Outer Rim → Core (spec 011 grew it 4 → 8 worlds). Cinder
/// forks into two two-world lanes — Scree → Karrus and Ashfall → Drift — that
/// rejoin at The Anvil, then a linear Core run: The Anvil → The Spindle →
/// Zenith, where the final boss waits. Every roster opponent appears exactly
/// once. `fx` trends rim→core; `fy` separates the lanes and staggers the Core
/// spine so no two nodes share a label row (guarded by a legibility test).
/// Names are original flavor, tunable.
pub const PLANETS: [Planet; 8] = [
    Planet {
        id: "cinder",
        name: "Cinder",
        region: "Outer Rim",
        blurb: "A slag-heap world where every hand is a warm-up.",
        fx: 0.06,
        fy: 0.50,
        opponents: &["greeb"],
        requires: &[],
    },
    // Lane A: Scree → Karrus (the upper fork off Cinder).
    Planet {
        id: "scree",
        name: "Scree",
        region: "Outer Rim",
        blurb: "A rubble moon where the young come to make a name.",
        fx: 0.23,
        fy: 0.28,
        opponents: &["dax"],
        requires: &["cinder"],
    },
    // Lane B: Ashfall → Drift (the lower fork off Cinder).
    Planet {
        id: "ashfall",
        name: "Ashfall",
        region: "Outer Rim",
        blurb: "Dust, debt, and a scrapper who plays like she has both.",
        fx: 0.23,
        fy: 0.72,
        opponents: &["vessa"],
        requires: &["cinder"],
    },
    Planet {
        id: "karrus",
        name: "Karrus",
        region: "Mid Rim",
        blurb: "A way-station of brokers who never bet past their means.",
        fx: 0.44,
        fy: 0.36,
        opponents: &["nima"],
        requires: &["scree"],
    },
    Planet {
        id: "drift",
        name: "Drift",
        region: "Mid Rim",
        blurb: "A quiet station where an old hand waits out the years.",
        fx: 0.44,
        fy: 0.64,
        opponents: &["toran"],
        requires: &["ashfall"],
    },
    // The rejoin: both lanes must be cleared to reach The Anvil.
    Planet {
        id: "the-anvil",
        name: "The Anvil",
        region: "Mid Rim",
        blurb: "A furnace-world where the hard cases hammer it out.",
        fx: 0.62,
        fy: 0.50,
        opponents: &["brakka", "kesh"],
        requires: &["karrus", "drift"],
    },
    Planet {
        id: "the-spindle",
        name: "The Spindle",
        region: "Core",
        blurb: "The core-world tower where the table's sharpest hold court.",
        fx: 0.80,
        fy: 0.32,
        opponents: &["rix", "magistrate"],
        requires: &["the-anvil"],
    },
    Planet {
        id: "zenith",
        name: "Zenith",
        region: "Core",
        blurb: "The summit table, where the house's best has never lost.",
        fx: 0.90,
        fy: 0.66,
        opponents: &["sovereign"],
        requires: &["the-spindle"],
    },
];

/// Resolve a planet id to its data. Unknown ids (older / hand-edited saves) →
/// `None`.
pub fn planet_by_id(id: &str) -> Option<Planet> {
    PLANETS.iter().copied().find(|p| p.id == id)
}

/// A pointer to the campaign match currently in flight — which planet, which
/// opponent. Persisted in the profile so a resumed match (Continue) still knows
/// it belongs to the campaign and routes back to the map at game over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRef {
    pub planet: String,
    pub opponent: String,
    /// Credits staked on this match (spec 021), held in escrow until it
    /// settles. Serde-defaulted, so a pre-021 profile loads with no stake.
    #[serde(default)]
    pub stake: u32,
    /// Whether this match has already been settled (spec 029). Serde-defaults
    /// to `false`, so a node written before this spec settles normally.
    #[serde(default)]
    pub settled: bool,
}

/// The series in progress (spec 029): the run of matches against one opponent
/// on one planet that beats them. At most one exists at a time, and while one
/// does it is the only campaign match the player may play — the **lock**.
/// Cleared by the same reset paths that clear the rest of the run, because it
/// is a plain field on [`CampaignRun`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    pub planet: String,
    pub opponent: String,
    pub player_wins: u32,
    pub opponent_wins: u32,
}

/// What a settled campaign match did to the series (spec 029).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesOutcome {
    /// A **rematch against an already-beaten opponent** — the only match that
    /// belongs to no series, and the one case that settles exactly as it did
    /// before this spec. A match against an un-beaten opponent never lands
    /// here: if it arrives with no series running (a match left in flight
    /// across the upgrade to this spec), one is started for it.
    NotInSeries,
    /// The tally moved; the series is still undecided.
    Continues,
    /// The player took the series: the opponent is beaten, now.
    Won,
    /// The opponent took it: the score is discarded, the opponent stays
    /// un-beaten, nothing further is taken (ruling C1).
    Lost,
}

/// The final opponent — the only best-of-five (ruling G1). One constant rather
/// than a roster field: `opponent.rs` is balance data this spec must not touch,
/// and a second `sovereign` would be a map bug, not a series rule.
pub const FINAL_OPPONENT: &str = "sovereign";

/// Match wins that take a series against `opponent`: 3 for the final opponent,
/// 2 for everyone else (spec 029, ruling G1). Derived, never stored — a stored
/// length would be a second source of truth an older or hand-edited save could
/// contradict.
pub fn wins_needed(opponent: &str) -> u32 {
    if opponent == FINAL_OPPONENT { 3 } else { 2 }
}

/// What a launch from the map commits the player to, for the planet detail
/// (spec 029): "Best of 3", or "Best of 5" for the final opponent.
pub fn series_length_label(opponent: &str) -> &'static str {
    if opponent == FINAL_OPPONENT { "Best of 5" } else { "Best of 3" }
}

/// The player's campaign progress: which opponents are beaten on each planet,
/// and the campaign match in flight (if any). Everything else — cleared /
/// unlocked / next opponent — is derived from `beaten` + [`PLANETS`], never
/// stored, so it can't drift.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CampaignRun {
    #[serde(default)]
    beaten: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    in_progress: Option<NodeRef>,
    /// The series in progress (spec 029), if any. Its own field beside
    /// `in_progress` rather than part of it: the pointer is cleared by a Quick
    /// Play match and by the kill-with-no-save forfeit, and neither of those
    /// ends a series. Serde-defaulted, so a pre-029 profile loads with none
    /// running.
    #[serde(default)]
    series: Option<Series>,
    /// Flat per-run statistics (spec 020). Additive and serde-defaulted, so an
    /// older run loads with an empty tally; cleared for free on reset since it's
    /// a plain field on the run.
    #[serde(default)]
    run_stats: RunStats,
}

impl CampaignRun {
    /// Record that `opponent` was beaten on `planet`. Idempotent — the per-planet
    /// list is a set.
    pub fn mark_beaten(&mut self, planet: &str, opponent: &str) {
        let list = self.beaten.entry(planet.to_string()).or_default();
        if !list.iter().any(|o| o == opponent) {
            list.push(opponent.to_string());
        }
    }

    /// Has `opponent` been beaten on `planet`?
    pub fn is_opponent_beaten(&self, planet: &str, opponent: &str) -> bool {
        self.beaten
            .get(planet)
            .is_some_and(|list| list.iter().any(|o| o == opponent))
    }

    /// A planet is cleared once every one of its opponents is beaten.
    pub fn planet_cleared(&self, planet: &Planet) -> bool {
        planet
            .opponents
            .iter()
            .all(|o| self.is_opponent_beaten(planet.id, o))
    }

    /// A planet is unlocked if every planet it requires is cleared (vacuously
    /// true for a start node with no requirements).
    pub fn planet_unlocked(&self, planet: &Planet) -> bool {
        planet
            .requires
            .iter()
            .all(|req| planet_by_id(req).is_some_and(|p| self.planet_cleared(&p)))
    }

    /// The next opponent to play on a planet: the first of its opponents not yet
    /// beaten, or `None` if the planet is cleared.
    pub fn next_opponent(&self, planet: &Planet) -> Option<&'static str> {
        planet
            .opponents
            .iter()
            .copied()
            .find(|o| !self.is_opponent_beaten(planet.id, o))
    }

    /// The opponent a launch from this planet would face: the next un-beaten
    /// one, or — once the planet is cleared — its final opponent, as a rematch.
    /// Callers gate on [`Self::planet_unlocked`].
    pub fn launchable_opponent(&self, planet: &Planet) -> Option<&'static str> {
        self.next_opponent(planet)
            .or_else(|| planet.opponents.last().copied())
    }

    /// The credits riding on the match in flight, if any are staked.
    pub fn stake_at_risk(&self) -> Option<u32> {
        self.in_progress
            .as_ref()
            .map(|n| n.stake)
            .filter(|&stake| stake > 0)
    }

    /// Take this match's settlement — the node and its escrowed stake — **once**.
    /// The first call marks the node settled and empties the escrow; every later
    /// call returns `None`, so a second settlement pays nothing, moves no series
    /// tally and beats nobody. Settling exactly once is therefore a property of
    /// the data, not an ordering rule about who calls what: spec 021 bought that
    /// for the payout by zeroing the escrow, and spec 029 extends it to
    /// cover the series tally and [`Self::mark_beaten`], neither of which is
    /// idempotent on its own. It does **not** cover `Profile::record_match`,
    /// which runs before settlement — a second `resolve_match` would still
    /// double-count statistics, as it would before this spec; that one is still
    /// guarded only by the `GameOver` edge.
    pub fn take_settlement(&mut self) -> Option<(NodeRef, u32)> {
        let node = self.in_progress.as_mut()?;
        if std::mem::replace(&mut node.settled, true) {
            return None;
        }
        let stake = std::mem::take(&mut node.stake);
        Some((node.clone(), stake))
    }

    /// The series in progress, if any.
    pub fn series(&self) -> Option<&Series> {
        self.series.as_ref()
    }

    /// Start a series against `opponent` on `planet`, at 0–0. Overwrites any
    /// existing one; the lock means a second can't be reached (the map is the
    /// only caller and is unreachable while locked).
    pub fn begin_series(&mut self, planet: &str, opponent: &str) {
        self.series = Some(Series {
            planet: planet.to_string(),
            opponent: opponent.to_string(),
            player_wins: 0,
            opponent_wins: 0,
        });
    }

    /// Credit a settled match to the series in progress, in three cases and no
    /// others:
    ///
    /// - **The series being played** — this node is the locked one. The
    ///   winner's tally goes up by one, and if it reaches [`wins_needed`] the
    ///   series ends and the lock is released; the score is discarded either
    ///   way it ends.
    /// - **A rematch** — the opponent is *already beaten*, so this match
    ///   belongs to no series: [`SeriesOutcome::NotInSeries`], and nothing
    ///   moves. This is the **only** case that returns it.
    /// - **Anything else is a match against an un-beaten opponent with no
    ///   series of its own**, which means a match left in flight across the
    ///   upgrade to this spec. `spec.md` §Saving and resuming: it "resolves as
    ///   the first match of a fresh series against that opponent" — so one is
    ///   started here and credited (1–0, `Continues`). Without this the match
    ///   would beat its opponent outright and clear a world in one.
    ///
    /// The test is therefore [`Self::is_opponent_beaten`], not whether a series
    /// happens to exist: a beaten opponent is a rematch and an un-beaten one is
    /// always in a series, so no settled match can beat an opponent who has not
    /// lost one. (A *different* series running when that third case fires is
    /// unreachable — the lock means only the locked node can be played — and it
    /// is replaced rather than special-cased, because the alternative is a
    /// variant that exists only for a state the design forbids.)
    ///
    /// Called once per match: [`Self::take_settlement`] is the guard.
    pub fn record_series_match(
        &mut self,
        planet: &str,
        opponent: &str,
        player_won: bool,
    ) -> SeriesOutcome {
        if self.is_opponent_beaten(planet, opponent) {
            return SeriesOutcome::NotInSeries;
        }
        let running = self
            .series
            .as_ref()
            .is_some_and(|s| s.planet == planet && s.opponent == opponent);
        if !running {
            self.begin_series(planet, opponent);
        }
        let needed = wins_needed(opponent);
        let series = self.series.as_mut().expect("a series was just begun");
        if player_won {
            series.player_wins += 1;
        } else {
            series.opponent_wins += 1;
        }
        let decided = if series.player_wins >= needed {
            Some(SeriesOutcome::Won)
        } else if series.opponent_wins >= needed {
            Some(SeriesOutcome::Lost)
        } else {
            None
        };
        match decided {
            Some(outcome) => {
                self.series = None;
                outcome
            }
            None => SeriesOutcome::Continues,
        }
    }

    /// The whole run is complete once every planet is cleared.
    pub fn run_complete(&self) -> bool {
        PLANETS.iter().all(|p| self.planet_cleared(p))
    }

    /// How many planets are cleared — the map header's progress figure and the
    /// run summary's "worlds cleared" (spec 024). Derived, never stored.
    pub fn worlds_cleared(&self) -> usize {
        PLANETS.iter().filter(|p| self.planet_cleared(p)).count()
    }

    /// Whether the player has cleared any opponent yet — real progress worth
    /// preserving or wiping. False for a fresh run (and one where a first match
    /// was started but never won); true once anything is `mark_beaten`. One
    /// disjunct of `Profile::differs_from_starter`, which drives the Continue /
    /// New Campaign / Reset Everything panel at campaign entry (spec 024,
    /// superseding spec 014's two-choice gate on this alone). Checks for a
    /// non-empty opponent list rather than a non-empty map, so a stray empty entry
    /// never reads as progress.
    pub fn has_progress(&self) -> bool {
        self.beaten.values().any(|list| !list.is_empty())
    }

    /// The campaign match currently in flight, if any.
    pub fn in_progress(&self) -> Option<&NodeRef> {
        self.in_progress.as_ref()
    }

    pub fn set_in_progress(&mut self, node: Option<NodeRef>) {
        self.in_progress = node;
    }

    /// The flat per-run statistics for this campaign run.
    pub fn run_stats(&self) -> &RunStats {
        &self.run_stats
    }

    /// Mutable per-run statistics, for recording a completed campaign match.
    pub fn run_stats_mut(&mut self) -> &mut RunStats {
        &mut self.run_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opponent::opponent_by_id;

    #[test]
    fn planets_are_well_formed() {
        for p in PLANETS {
            assert!(!p.id.is_empty() && !p.name.is_empty(), "{} malformed", p.id);
            assert!((0.0..=1.0).contains(&p.fx), "{} fx out of range", p.id);
            assert!((0.0..=1.0).contains(&p.fy), "{} fy out of range", p.id);
            assert!(!p.opponents.is_empty(), "{} has no opponents", p.id);
            for o in p.opponents {
                assert!(opponent_by_id(o).is_some(), "{} lists unknown opponent {o}", p.id);
            }
            for req in p.requires {
                assert!(planet_by_id(req).is_some(), "{} requires unknown planet {req}", p.id);
            }
        }
    }

    #[test]
    fn planet_ids_are_unique() {
        for (i, a) in PLANETS.iter().enumerate() {
            for b in &PLANETS[i + 1..] {
                assert_ne!(a.id, b.id, "duplicate planet id {}", a.id);
            }
        }
    }

    #[test]
    fn start_planet_exists_and_has_no_requirements() {
        let start = planet_by_id(START_PLANET).expect("START_PLANET must be a real planet");
        assert!(start.requires.is_empty(), "the start planet must be unlocked from the start");
    }

    #[test]
    fn a_fresh_run_unlocks_only_the_start() {
        let run = CampaignRun::default();
        for p in PLANETS {
            let unlocked = run.planet_unlocked(&p);
            if p.id == START_PLANET {
                assert!(unlocked, "the start planet should be unlocked");
            } else {
                assert!(!unlocked, "{} should be locked in a fresh run", p.id);
            }
            assert!(!run.planet_cleared(&p), "{} should not be cleared fresh", p.id);
        }
        assert!(!run.run_complete());
    }

    #[test]
    fn has_progress_is_false_until_an_opponent_is_beaten() {
        let mut run = CampaignRun::default();
        assert!(!run.has_progress(), "a fresh run has no progress");
        run.mark_beaten("cinder", "greeb");
        assert!(run.has_progress(), "clearing an opponent is progress");
    }

    #[test]
    fn has_progress_ignores_a_stray_empty_beaten_entry() {
        // Only reachable via a hand-edited save, but the defensive claim must
        // hold: an empty opponent list is not progress (guards the `values()`
        // check against a refactor back to a bare `!beaten.is_empty()`).
        let run: CampaignRun = serde_json::from_str(r#"{"beaten":{"cinder":[]}}"#).unwrap();
        assert!(!run.has_progress());
    }

    #[test]
    fn clearing_a_planet_needs_all_its_opponents() {
        let spindle = planet_by_id("the-spindle").unwrap();
        let mut run = CampaignRun::default();
        assert_eq!(run.next_opponent(&spindle), Some("rix"));
        run.mark_beaten("the-spindle", "rix");
        assert!(!run.planet_cleared(&spindle), "one of two beaten is not cleared");
        assert_eq!(run.next_opponent(&spindle), Some("magistrate"));
        run.mark_beaten("the-spindle", "magistrate");
        assert!(run.planet_cleared(&spindle));
        assert_eq!(run.next_opponent(&spindle), None);
    }

    #[test]
    fn launchable_opponent_falls_back_to_a_rematch_once_cleared() {
        let cinder = planet_by_id("cinder").unwrap();
        let anvil = planet_by_id("the-anvil").unwrap();
        let mut run = CampaignRun::default();

        // Uncleared: the next un-beaten opponent, in order.
        assert_eq!(run.launchable_opponent(&cinder), Some("greeb"));
        assert_eq!(run.launchable_opponent(&anvil), Some("brakka"));
        run.mark_beaten("the-anvil", "brakka");
        assert_eq!(run.launchable_opponent(&anvil), Some("kesh"));

        // Cleared: the planet's final opponent, as a rematch.
        run.mark_beaten("the-anvil", "kesh");
        assert!(run.planet_cleared(&anvil));
        assert_eq!(run.launchable_opponent(&anvil), Some("kesh"));

        // Same rule on a single-opponent planet.
        run.mark_beaten("cinder", "greeb");
        assert!(run.planet_cleared(&cinder));
        assert_eq!(run.launchable_opponent(&cinder), Some("greeb"));
    }

    #[test]
    fn the_stake_rides_on_the_in_flight_node_and_defaults_to_zero() {
        // A pre-021 node (no `stake` key) loads unstaked...
        let node: NodeRef =
            serde_json::from_str(r#"{"planet":"cinder","opponent":"greeb"}"#).unwrap();
        assert_eq!(node.stake, 0);

        // ...and a staked one round-trips.
        let staked = NodeRef {
            planet: "scree".to_string(),
            opponent: "dax".to_string(),
            stake: 20,
            settled: false,
        };
        let json = serde_json::to_string(&staked).unwrap();
        assert_eq!(serde_json::from_str::<NodeRef>(&json).unwrap(), staked);
    }

    #[test]
    fn take_settlement_hands_over_the_match_exactly_once() {
        let mut run = CampaignRun::default();
        // Nothing in flight: nothing at risk, nothing to hand over.
        assert_eq!(run.stake_at_risk(), None);
        assert_eq!(run.take_settlement(), None);

        // An unstaked match in flight is still nothing at risk.
        run.set_in_progress(Some(NodeRef {
            planet: "cinder".to_string(),
            opponent: "greeb".to_string(),
            stake: 0,
            settled: false,
        }));
        assert_eq!(run.stake_at_risk(), None);

        run.set_in_progress(Some(NodeRef {
            planet: "scree".to_string(),
            opponent: "dax".to_string(),
            stake: 20,
            settled: false,
        }));
        assert_eq!(run.stake_at_risk(), Some(20));
        let (node, stake) = run.take_settlement().expect("a match in flight hands itself over");
        assert_eq!((node.planet.as_str(), node.opponent.as_str()), ("scree", "dax"));
        assert_eq!(stake, 20);
        assert_eq!(run.take_settlement(), None, "the match settles only once");
        assert_eq!(run.stake_at_risk(), None);
    }

    #[test]
    fn a_pre_029_node_is_unsettled_and_unstaked() {
        // A node written before this spec carries neither key...
        let node: NodeRef =
            serde_json::from_str(r#"{"planet":"cinder","opponent":"greeb"}"#).unwrap();
        assert_eq!(node.stake, 0);
        assert!(!node.settled, "a pre-029 node has not been settled");

        // ...and still settles, exactly once.
        let mut run = CampaignRun::default();
        run.set_in_progress(Some(node));
        let (handed, stake) = run.take_settlement().expect("a pre-029 node settles");
        assert_eq!(handed.opponent, "greeb");
        assert_eq!(stake, 0);
        assert_eq!(run.take_settlement(), None, "the match settles only once");
    }

    #[test]
    fn wins_needed_is_two_except_for_the_final_opponent() {
        use crate::opponent::OPPONENTS;
        for o in OPPONENTS {
            let needed = if o.id == FINAL_OPPONENT { 3 } else { 2 };
            assert_eq!(wins_needed(o.id), needed, "{} should need {needed} wins", o.id);
        }
        assert_eq!(series_length_label(FINAL_OPPONENT), "Best of 5");
        assert_eq!(series_length_label("greeb"), "Best of 3");

        // The final opponent is the last opponent of the last planet, so a map
        // edit that retires the boss fails here rather than silently shortening
        // the final series.
        let last = PLANETS.last().expect("the map has planets");
        assert_eq!(last.opponents.last().copied(), Some(FINAL_OPPONENT));
    }

    #[test]
    fn a_series_resolves_only_at_the_required_wins() {
        // Best of three, the player's way: 1–0, 1–1, and the second win takes it.
        let mut run = CampaignRun::default();
        run.begin_series("cinder", "greeb");
        assert_eq!(run.record_series_match("cinder", "greeb", true), SeriesOutcome::Continues);
        assert_eq!(run.record_series_match("cinder", "greeb", false), SeriesOutcome::Continues);
        assert_eq!(
            run.series(),
            Some(&Series {
                planet: "cinder".to_string(),
                opponent: "greeb".to_string(),
                player_wins: 1,
                opponent_wins: 1,
            }),
        );
        assert_eq!(run.record_series_match("cinder", "greeb", true), SeriesOutcome::Won);
        assert_eq!(run.series(), None, "a decided series is cleared");

        // Best of three, the opponent's way: 0–1, then 0–2 takes it.
        let mut run = CampaignRun::default();
        run.begin_series("scree", "dax");
        assert_eq!(run.record_series_match("scree", "dax", false), SeriesOutcome::Continues);
        assert_eq!(run.record_series_match("scree", "dax", false), SeriesOutcome::Lost);
        assert_eq!(run.series(), None, "a lost series is discarded too");

        // Best of five: two wins are not enough for the final opponent.
        let mut run = CampaignRun::default();
        run.begin_series("zenith", FINAL_OPPONENT);
        assert_eq!(
            run.record_series_match("zenith", FINAL_OPPONENT, true),
            SeriesOutcome::Continues,
        );
        assert_eq!(
            run.record_series_match("zenith", FINAL_OPPONENT, true),
            SeriesOutcome::Continues,
            "two wins take nobody on the final table",
        );
        assert_eq!(run.record_series_match("zenith", FINAL_OPPONENT, false), SeriesOutcome::Continues);
        assert_eq!(run.record_series_match("zenith", FINAL_OPPONENT, true), SeriesOutcome::Won);
        assert_eq!(run.series(), None);

        // ...and the same length the other way.
        let mut run = CampaignRun::default();
        run.begin_series("zenith", FINAL_OPPONENT);
        for _ in 0..2 {
            assert_eq!(
                run.record_series_match("zenith", FINAL_OPPONENT, false),
                SeriesOutcome::Continues,
            );
        }
        assert_eq!(run.record_series_match("zenith", FINAL_OPPONENT, false), SeriesOutcome::Lost);
        assert_eq!(run.series(), None);
    }

    #[test]
    fn a_match_in_flight_with_no_series_starts_one() {
        // A match left in flight across the upgrade to this spec: no series, an
        // un-beaten opponent, so it is the first match of a fresh series.
        let mut run = CampaignRun::default();
        assert_eq!(run.series(), None, "sanity: nothing running");
        assert_eq!(run.record_series_match("cinder", "greeb", true), SeriesOutcome::Continues);
        assert_eq!(
            run.series(),
            Some(&Series {
                planet: "cinder".to_string(),
                opponent: "greeb".to_string(),
                player_wins: 1,
                opponent_wins: 0,
            }),
        );

        // The same call against an already-beaten opponent is a rematch.
        let mut rematch = CampaignRun::default();
        rematch.mark_beaten("cinder", "greeb");
        assert_eq!(rematch.record_series_match("cinder", "greeb", true), SeriesOutcome::NotInSeries);
        assert_eq!(rematch.series(), None, "a rematch starts no series");
    }

    #[test]
    fn no_settled_match_beats_an_unbeaten_opponent_outright() {
        // `NotInSeries` is the one outcome that settles as it did before this
        // spec — the arm that could beat an opponent on a single win. Over the
        // series being played, a settled match against a different un-beaten
        // node, a match with no series at all, and a rematch, it is returned
        // only when the opponent is already beaten.
        let mut run = CampaignRun::default();
        run.mark_beaten("cinder", "greeb");
        run.begin_series("scree", "dax");
        for (planet, opponent, player_won) in [
            ("scree", "dax", true),      // the series being played
            ("ashfall", "vessa", true),  // a different un-beaten node
            ("karrus", "nima", false),   // no series of its own
            ("cinder", "greeb", true),   // the rematch
        ] {
            let outcome = run.record_series_match(planet, opponent, player_won);
            assert_eq!(
                outcome == SeriesOutcome::NotInSeries,
                run.is_opponent_beaten(planet, opponent),
                "{opponent} on {planet}: NotInSeries means an already-beaten opponent, nothing else",
            );
        }
    }

    #[test]
    fn a_campaign_run_round_trips_its_series() {
        let mut run = CampaignRun::default();
        run.mark_beaten("cinder", "greeb");
        run.begin_series("scree", "dax");
        assert_eq!(run.record_series_match("scree", "dax", false), SeriesOutcome::Continues);

        let json = serde_json::to_string(&run).unwrap();
        let loaded: CampaignRun = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.series(), run.series(), "the series survives a save/load");
        assert!(loaded.is_opponent_beaten("cinder", "greeb"));

        // A pre-029 run has no `series` key and loads with none running.
        let older: CampaignRun = serde_json::from_str(r#"{"beaten":{"cinder":["greeb"]}}"#).unwrap();
        assert_eq!(older.series(), None);
    }

    #[test]
    fn the_fork_opens_both_lanes_and_the_rejoin_needs_both() {
        let scree = planet_by_id("scree").unwrap();
        let ashfall = planet_by_id("ashfall").unwrap();
        let anvil = planet_by_id("the-anvil").unwrap();
        let mut run = CampaignRun::default();

        // Beating Cinder's opponent opens BOTH lanes' first worlds (the fork).
        run.mark_beaten("cinder", "greeb");
        assert!(run.planet_unlocked(&scree));
        assert!(run.planet_unlocked(&ashfall));
        assert!(!run.planet_unlocked(&anvil), "the rejoin needs both lanes cleared");

        // Clearing only lane A (scree → karrus) is not enough for the rejoin.
        run.mark_beaten("scree", "dax");
        run.mark_beaten("karrus", "nima");
        assert!(!run.planet_unlocked(&anvil));

        // Clearing lane B too (ashfall → drift) → The Anvil unlocks (the rejoin).
        run.mark_beaten("ashfall", "vessa");
        run.mark_beaten("drift", "toran");
        assert!(run.planet_unlocked(&anvil));
    }

    #[test]
    fn mark_beaten_is_idempotent_and_a_full_sweep_completes_the_run() {
        // Beat every opponent on every planet, optionally skipping one — derived
        // from PLANETS so it stays correct as the map grows.
        let sweep = |run: &mut CampaignRun, skip: Option<(&str, &str)>| {
            for p in PLANETS {
                for o in p.opponents {
                    if skip == Some((p.id, *o)) {
                        continue;
                    }
                    run.mark_beaten(p.id, o);
                }
            }
        };

        let mut run = CampaignRun::default();
        run.mark_beaten("cinder", "greeb");
        run.mark_beaten("cinder", "greeb"); // idempotent — no double-count
        assert!(run.planet_cleared(&planet_by_id("cinder").unwrap()));

        // A sweep missing just the final boss leaves the run incomplete (so the
        // completeness below isn't vacuous)...
        let mut partial = CampaignRun::default();
        sweep(&mut partial, Some(("zenith", "sovereign")));
        assert!(!partial.run_complete());

        // ...and the full sweep completes it.
        sweep(&mut run, None);
        assert!(run.run_complete());
    }

    #[test]
    fn worlds_cleared_counts_cleared_planets() {
        let mut run = CampaignRun::default();
        assert_eq!(run.worlds_cleared(), 0, "a fresh run has cleared nothing");

        // Cinder's only opponent clears it.
        run.mark_beaten("cinder", "greeb");
        assert_eq!(run.worlds_cleared(), 1);

        // A part-beaten planet doesn't count until every opponent is beaten.
        run.mark_beaten("the-spindle", "rix");
        assert_eq!(run.worlds_cleared(), 1, "one of two beaten is not a cleared world");
        run.mark_beaten("the-spindle", "magistrate");
        assert_eq!(run.worlds_cleared(), 2);

        // A full sweep clears every world.
        for p in PLANETS {
            for o in p.opponents {
                run.mark_beaten(p.id, o);
            }
        }
        assert_eq!(run.worlds_cleared(), PLANETS.len());
        assert!(run.run_complete());
    }

    #[test]
    fn the_graph_has_one_start_is_acyclic_and_fully_reachable() {
        let starts: Vec<&str> = PLANETS
            .iter()
            .filter(|p| p.requires.is_empty())
            .map(|p| p.id)
            .collect();
        assert_eq!(starts, vec![START_PLANET], "exactly one start planet");

        // Fixpoint clear: repeatedly clear any planet whose requires are all
        // cleared. Terminating with every planet cleared proves the graph is
        // acyclic AND fully reachable from the start (a cycle or an orphan would
        // leave some planet permanently un-clearable).
        let mut cleared: Vec<&str> = Vec::new();
        let mut changed = true;
        while changed {
            changed = false;
            for p in &PLANETS {
                if !cleared.contains(&p.id) && p.requires.iter().all(|r| cleared.contains(r)) {
                    cleared.push(p.id);
                    changed = true;
                }
            }
        }
        assert_eq!(
            cleared.len(),
            PLANETS.len(),
            "every planet must be reachable via a valid clear order"
        );
    }

    #[test]
    fn every_roster_opponent_appears_on_exactly_one_planet() {
        use crate::opponent::OPPONENTS;
        let appearances: Vec<&str> = PLANETS
            .iter()
            .flat_map(|p| p.opponents.iter().copied())
            .collect();
        // No opponent is stranded (on no planet, unplayable) or double-booked...
        for o in OPPONENTS {
            let count = appearances.iter().filter(|a| **a == o.id).count();
            assert_eq!(count, 1, "{} should appear on exactly one planet, found {count}", o.id);
        }
        // ...and the map references nothing outside the roster.
        assert_eq!(
            appearances.len(),
            OPPONENTS.len(),
            "the map references a non-roster or duplicate opponent"
        );
    }

    #[test]
    fn difficulty_is_monotonic_along_every_edge() {
        use crate::opponent::opponent_by_id;
        // (min, max) stand threshold among a planet's opponents.
        let bounds = |p: &Planet| -> (usize, usize) {
            let ts: Vec<usize> = p
                .opponents
                .iter()
                .map(|o| opponent_by_id(o).unwrap().stand_threshold)
                .collect();
            (*ts.iter().min().unwrap(), *ts.iter().max().unwrap())
        };
        // For every requires-edge (predecessor → successor), the predecessor's
        // hardest opponent is no harder than the successor's easiest — so
        // difficulty never drops as you travel rim → core along any path.
        for succ in PLANETS {
            let (succ_min, _) = bounds(&succ);
            for req in succ.requires {
                let pred = planet_by_id(req).unwrap();
                let (_, pred_max) = bounds(&pred);
                assert!(
                    pred_max <= succ_min,
                    "{} (max threshold {pred_max}) is harder than its successor {} (min {succ_min})",
                    pred.id,
                    succ.id
                );
            }
        }
    }
}
