//! The campaign economy (spec 012, subsystem C): the depth-gated card pool,
//! shop pricing, and the per-win reward. Pure logic over `campaign`/`card` data
//! — no rendering and no state of its own; the profile holds credits and the
//! grown collection. Everything here is a function of the campaign run (for
//! depth) plus one injected `roll` (for the drop), so it is fully unit-testable
//! and the randomness lives in a single caller-supplied index (the spec-010
//! deterministic-core pattern). See `docs/economy.md`.

use crate::{
    campaign::{CampaignRun, PLANETS},
    card::{ALL_SIDE_CARDS, Card},
};

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

/// What a campaign win grants: credits plus one card dropped into the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinReward {
    pub credits: u32,
    pub card: Card,
}

/// Compute a win's reward: credits scaled by the beaten opponent's stand
/// threshold (15→10 … 19→50), and a card chosen from `pool` by `roll`. Pure and
/// deterministic given `roll` — the caller injects the single random index, so
/// the whole thing is unit-testable. `pool` must be non-empty (an
/// `available_pool` always is).
pub fn win_reward(threshold: usize, pool: &[Card], roll: usize) -> WinReward {
    let credits = threshold.saturating_sub(14) as u32 * 10;
    let card = pool[roll % pool.len()];
    WinReward { credits, card }
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
    fn win_reward_scales_credits_and_picks_the_card_by_roll() {
        let pool = [Card::Plus(1), Card::Plus(2), Card::Plus(3)];
        // Credits scale with the beaten opponent's threshold.
        assert_eq!(win_reward(15, &pool, 0).credits, 10);
        assert_eq!(win_reward(17, &pool, 0).credits, 30);
        assert_eq!(win_reward(19, &pool, 0).credits, 50);
        // The card is the roll-indexed pool entry, wrapping.
        assert_eq!(win_reward(15, &pool, 0).card, Card::Plus(1));
        assert_eq!(win_reward(15, &pool, 1).card, Card::Plus(2));
        assert_eq!(win_reward(15, &pool, 5).card, Card::Plus(3)); // 5 % 3 == 2
    }
}
