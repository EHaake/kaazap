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
pub const START_PLANET: &str = "cinder";

/// The first campaign map: Cinder (Outer Rim) → the Ashfall/Drift fork (Mid
/// Rim, either order) → The Spindle (Core, two opponents). Uses all five roster
/// opponents. Names are original placeholders, tunable — a vertical slice that
/// grows with the roster and the spec-C economy.
pub const PLANETS: [Planet; 4] = [
    Planet {
        id: "cinder",
        name: "Cinder",
        region: "Outer Rim",
        blurb: "A slag-heap world where every hand is a warm-up.",
        fx: 0.12,
        fy: 0.50,
        opponents: &["greeb"],
        requires: &[],
    },
    Planet {
        id: "ashfall",
        name: "Ashfall",
        region: "Mid Rim",
        blurb: "Dust, debt, and a scrapper who plays like she has both.",
        fx: 0.42,
        fy: 0.28,
        opponents: &["vessa"],
        requires: &["cinder"],
    },
    Planet {
        id: "drift",
        name: "Drift",
        region: "Mid Rim",
        blurb: "A quiet station where an old hand waits out the years.",
        fx: 0.42,
        fy: 0.72,
        opponents: &["toran"],
        requires: &["cinder"],
    },
    Planet {
        id: "the-spindle",
        name: "The Spindle",
        region: "Core",
        blurb: "The core-world tower where the table's sharpest hold court.",
        fx: 0.82,
        fy: 0.50,
        opponents: &["rix", "magistrate"],
        requires: &["ashfall", "drift"],
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
    fn the_fork_unlocks_both_mid_worlds_and_the_core_needs_both() {
        let ashfall = planet_by_id("ashfall").unwrap();
        let drift = planet_by_id("drift").unwrap();
        let spindle = planet_by_id("the-spindle").unwrap();
        let mut run = CampaignRun::default();

        // Beating Cinder's opponent unlocks BOTH mid worlds (the fork).
        run.mark_beaten("cinder", "greeb");
        assert!(run.planet_unlocked(&ashfall));
        assert!(run.planet_unlocked(&drift));
        assert!(!run.planet_unlocked(&spindle), "the Core needs both mid worlds");

        // One mid world is not enough for the Core.
        run.mark_beaten("ashfall", "vessa");
        assert!(!run.planet_unlocked(&spindle));

        // Both mid worlds cleared → the Core unlocks (the rejoin).
        run.mark_beaten("drift", "toran");
        assert!(run.planet_unlocked(&spindle));
    }

    #[test]
    fn mark_beaten_is_idempotent_and_a_full_sweep_completes_the_run() {
        let mut run = CampaignRun::default();
        run.mark_beaten("cinder", "greeb");
        run.mark_beaten("cinder", "greeb"); // no double-count
        assert!(run.planet_cleared(&planet_by_id("cinder").unwrap()));

        run.mark_beaten("ashfall", "vessa");
        run.mark_beaten("drift", "toran");
        run.mark_beaten("the-spindle", "rix");
        assert!(!run.run_complete());
        run.mark_beaten("the-spindle", "magistrate");
        assert!(run.run_complete());
    }
}
