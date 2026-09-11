//! The campaign economy (spec 012, subsystem C; wagering, spec 021): the
//! depth-gated card pool, shop pricing, and the ante/payout rules. Pure logic
//! over `campaign`/`card` data — no rendering and no state of its own; the
//! profile holds credits and the grown collection. Everything here is a
//! function of the campaign run (for depth) and the opponent's difficulty, so
//! it is fully unit-testable; nothing here is random any more (spec 021
//! replaced the win reward's card drop with a staked payout, so the injected
//! `roll` seam is gone). See `docs/economy.md`.

use crate::{
    campaign::{CampaignRun, PLANETS},
    card::{ALL_SIDE_CARDS, Card},
    opponent::opponent_by_id,
};

/// Credits a fresh or reset profile starts with (spec 021).
pub const SEED_PURSE: u32 = 50;
/// The threshold an ante floor is measured from: a 14-threshold opponent would
/// cost nothing, so the easiest real opponent (15) sits one step up.
pub const ANTE_BASE_THRESHOLD: usize = 14;
/// Credits the ante floor rises per point of stand threshold above the base.
pub const ANTE_PER_THRESHOLD_STEP: u32 = 10;
/// The wager prompt's increment.
pub const STAKE_STEP: u32 = 5;
/// Winnings per credit staked (1 = even money): a win returns stake × (1 + PAYOUT_RATIO).
pub const PAYOUT_RATIO: u32 = 1;

/// The three campaign regions as an ordered depth tier. Deeper regions unlock
/// strictly more of the card universe, so `Outer < Mid < Core`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegionTier {
    Outer,
    Mid,
    Core,
}

/// Map a planet's `region` string to its depth tier. An unrecognized region
/// falls back to `Outer` (the most conservative — never over-grants); the
/// `every_planet_region_maps_to_a_known_tier` test guards against a typo ever
/// reaching that fallback for a real planet.
pub fn region_tier(region: &str) -> RegionTier {
    match region {
        "Outer Rim" => RegionTier::Outer,
        "Mid Rim" => RegionTier::Mid,
        "Core" => RegionTier::Core,
        _ => RegionTier::Outer,
    }
}

/// The deepest region the player has **reached** — the max tier over every
/// unlocked planet. Reaching the Mid Rim opens the Mid pool; reaching the Core
/// opens the Core pool. Always at least `Outer` (the start planet is unlocked
/// from the beginning).
pub fn deepest_reached(run: &CampaignRun) -> RegionTier {
    PLANETS
        .iter()
        .filter(|p| run.planet_unlocked(p))
        .map(|p| region_tier(p.region))
        .max()
        .unwrap_or(RegionTier::Outer)
}

/// Which tier a collectible card unlocks at. Partitions the 15-card universe by
/// power: basic adjusters in the Outer Rim, bigger swings and flips in the Mid
/// Rim, the premium ±6 and tiebreaker in the Core. A non-collectible card (never
/// in `ALL_SIDE_CARDS`) falls to `Core` so it can't leak into an early pool.
pub fn card_tier(card: Card) -> RegionTier {
    match card {
        Card::Plus(1..=3) | Card::Minus(1..=3) | Card::PlusMinus(1) => RegionTier::Outer,
        Card::Plus(4) | Card::Minus(4) | Card::PlusMinus(2 | 3) | Card::Flip(_) => RegionTier::Mid,
        Card::PlusMinus(6) | Card::Tiebreaker => RegionTier::Core,
        _ => RegionTier::Core,
    }
}

/// The cards available to win or buy at the player's current depth: every
/// universe card whose tier is at or above-shallow-of the deepest region
/// reached. Always non-empty (the Outer tier is always available).
pub fn available_pool(run: &CampaignRun) -> Vec<Card> {
    let depth = deepest_reached(run);
    ALL_SIDE_CARDS
        .iter()
        .copied()
        .filter(|&card| card_tier(card) <= depth)
        .collect()
}

/// The shop price of a card, by tier (tunable balance data).
pub fn card_price(card: Card) -> u32 {
    match card_tier(card) {
        RegionTier::Outer => 20,
        RegionTier::Mid => 50,
        RegionTier::Core => 120,
    }
}

/// The minimum stake for an opponent of this stand threshold — the difficulty
/// scalar, in one tunable formula: 15 → 10, 16 → 20, … 19 → 50.
pub fn ante_floor(threshold: usize) -> u32 {
    threshold.saturating_sub(ANTE_BASE_THRESHOLD) as u32 * ANTE_PER_THRESHOLD_STEP
}

/// The ante floor for an opponent id. An unknown id (older / hand-edited save)
/// falls back to the baseline stand threshold, never to a free match.
pub fn ante_floor_for(opponent_id: &str) -> u32 {
    ante_floor(opponent_by_id(opponent_id).map_or(crate::STAND_THRESHOLD, |o| o.stand_threshold))
}

/// What returns to the balance on a win: the stake back plus its winnings.
pub fn win_payout(stake: u32) -> u32 {
    stake.saturating_mul(1 + PAYOUT_RATIO)
}

/// The lowest ante over every node the player could launch right now (unlocked
/// planets × their launchable opponent); 0 if there are none — unreachable,
/// since the start planet is always unlocked and always has a rematch.
pub fn cheapest_floor(run: &CampaignRun) -> u32 {
    PLANETS
        .iter()
        .filter(|p| run.planet_unlocked(p))
        .filter_map(|p| run.launchable_opponent(p))
        .map(ante_floor_for)
        .min()
        .unwrap_or(0)
}

/// How a staked campaign match settled (for the map banner) — the stake that
/// changed hands, not the payout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakeOutcome {
    Won(u32),
    Lost(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::FlipKind;

    fn cleared(pairs: &[(&str, &str)]) -> CampaignRun {
        let mut run = CampaignRun::default();
        for (planet, opponent) in pairs {
            run.mark_beaten(planet, opponent);
        }
        run
    }

    #[test]
    fn card_tier_partitions_the_universe() {
        let outer = [
            Card::Plus(1),
            Card::Plus(2),
            Card::Plus(3),
            Card::Minus(1),
            Card::Minus(2),
            Card::Minus(3),
            Card::PlusMinus(1),
        ];
        let mid = [
            Card::Plus(4),
            Card::Minus(4),
            Card::PlusMinus(2),
            Card::PlusMinus(3),
            Card::Flip(FlipKind::TwoFour),
            Card::Flip(FlipKind::ThreeSix),
        ];
        let core = [Card::PlusMinus(6), Card::Tiebreaker];

        for &c in &outer {
            assert_eq!(card_tier(c), RegionTier::Outer, "{c:?}");
        }
        for &c in &mid {
            assert_eq!(card_tier(c), RegionTier::Mid, "{c:?}");
        }
        for &c in &core {
            assert_eq!(card_tier(c), RegionTier::Core, "{c:?}");
        }
        // The three tiers exactly cover the 15-card universe — no gaps, no
        // overlaps, nothing falling to the Core fallback by accident.
        assert_eq!(outer.len() + mid.len() + core.len(), ALL_SIDE_CARDS.len());
        for &c in &ALL_SIDE_CARDS {
            assert!(
                outer.contains(&c) || mid.contains(&c) || core.contains(&c),
                "{c:?} is untiered"
            );
        }
    }

    #[test]
    fn every_planet_region_maps_to_a_known_tier() {
        for p in PLANETS {
            assert!(
                matches!(p.region, "Outer Rim" | "Mid Rim" | "Core"),
                "{} has an unrecognized region {:?}",
                p.id,
                p.region
            );
        }
    }

    #[test]
    fn the_pool_grows_monotonically_with_depth() {
        // Fresh: only Cinder (Outer) unlocked.
        let outer = CampaignRun::default();
        assert_eq!(deepest_reached(&outer), RegionTier::Outer);

        // Clear Cinder → Scree so Karrus (Mid Rim) unlocks.
        let mid = cleared(&[("cinder", "greeb"), ("scree", "dax")]);
        assert_eq!(deepest_reached(&mid), RegionTier::Mid);

        // Clear through The Anvil so The Spindle (Core) unlocks.
        let core = cleared(&[
            ("cinder", "greeb"),
            ("scree", "dax"),
            ("ashfall", "vessa"),
            ("karrus", "nima"),
            ("drift", "toran"),
            ("the-anvil", "brakka"),
            ("the-anvil", "kesh"),
        ]);
        assert_eq!(deepest_reached(&core), RegionTier::Core);

        let (po, pm, pc) = (
            available_pool(&outer),
            available_pool(&mid),
            available_pool(&core),
        );
        // Nested: Outer ⊆ Mid ⊆ Core ⊆ universe, and each region adds cards.
        assert!(po.iter().all(|c| pm.contains(c)));
        assert!(pm.iter().all(|c| pc.contains(c)));
        assert!(pc.iter().all(|c| ALL_SIDE_CARDS.contains(c)));
        assert!(po.len() < pm.len() && pm.len() < pc.len());
        assert_eq!(pc.len(), ALL_SIDE_CARDS.len(), "the Core opens the whole pool");
    }

    #[test]
    fn every_card_has_a_positive_price_that_rises_with_tier() {
        for &c in &ALL_SIDE_CARDS {
            assert!(card_price(c) > 0, "{c:?} has no price");
        }
        assert!(card_price(Card::Plus(1)) < card_price(Card::PlusMinus(3)));
        assert!(card_price(Card::PlusMinus(3)) < card_price(Card::PlusMinus(6)));
    }

    #[test]
    fn ante_floor_is_the_difficulty_scalar() {
        assert_eq!(ante_floor(15), 10);
        assert_eq!(ante_floor(16), 20);
        assert_eq!(ante_floor(17), 30);
        assert_eq!(ante_floor(18), 40);
        assert_eq!(ante_floor(19), 50);
        // The easiest real opponent sits at the bottom of the scale...
        assert_eq!(ante_floor_for("greeb"), 10);
        // ...and an id no roster entry claims falls back to the baseline (17).
        assert_eq!(ante_floor_for("nobody-by-that-name"), 30);
    }

    #[test]
    fn payout_is_even_money() {
        assert_eq!(win_payout(20), 40);
        assert_eq!(win_payout(0), 0);
    }

    #[test]
    fn cheapest_floor_is_the_min_over_launchable_nodes() {
        // Independently computed: the min ante over every unlocked planet's
        // launchable opponent, spelled out rather than reusing the function.
        let expected = |run: &CampaignRun| -> u32 {
            let mut best = u32::MAX;
            for p in PLANETS {
                if !run.planet_unlocked(&p) {
                    continue;
                }
                let opponent = match run.next_opponent(&p) {
                    Some(o) => o,
                    None => *p.opponents.last().unwrap(),
                };
                let threshold = opponent_by_id(opponent).unwrap().stand_threshold;
                best = best.min(ante_floor(threshold));
            }
            if best == u32::MAX { 0 } else { best }
        };

        let fresh = CampaignRun::default();
        let half = cleared(&[("cinder", "greeb"), ("scree", "dax"), ("ashfall", "vessa")]);
        let mut complete = CampaignRun::default();
        for p in PLANETS {
            for o in p.opponents {
                complete.mark_beaten(p.id, o);
            }
        }
        assert!(complete.run_complete(), "sanity: the sweep completes the run");

        for (label, run) in [("fresh", &fresh), ("half-cleared", &half), ("complete", &complete)] {
            assert_eq!(cheapest_floor(run), expected(run), "{label}");
            // Cinder's rematch keeps the cheapest match at the floor forever.
            assert_eq!(cheapest_floor(run), 10, "{label}");
        }
    }
}
