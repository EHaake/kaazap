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

    /// The whole run is complete once every planet is cleared.
    pub fn run_complete(&self) -> bool {
        PLANETS.iter().all(|p| self.planet_cleared(p))
    }

    /// Whether the player has cleared any opponent yet — real progress worth
    /// preserving or wiping. False for a fresh run (and one where a first match
    /// was started but never won); true once anything is `mark_beaten`. Drives the
    /// Continue / New Campaign choice at campaign entry (spec 014). Checks for a
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
