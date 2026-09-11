//! The persistent player profile: the cards you own (a collection) and the
//! side deck you've built from them. The first player-owned save document,
//! distinct from the mid-match save (`save.rs`) — and the one spec C (economy)
//! and spec D (campaign) will extend rather than replace.
//!
//! Modeled on `settings.rs` (a self-owned serde struct that loads to a value
//! with defaults and saves best-effort) plus `save.rs`'s version-discard
//! discipline. `Card` already derives serde, so the collection and deck
//! persist as plain `Vec<Card>` — no projection type. Deck-building rules
//! (own-a-copy, the size cap) live here as methods, so the UI just calls them.
//! See `specs/008-side-deck-customization`.

use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    SIDE_DECK_SIZE,
    campaign::{CampaignRun, NodeRef},
    card::{ALL_SIDE_CARDS, Card, DEFAULT_SIDE_DECK},
    economy::{self, StakeOutcome, WinReward},
    stats::{LifetimeStats, Mode},
};

/// Bump when the on-disk shape changes incompatibly; a file whose version
/// doesn't match is then discarded rather than mis-read (as `save.rs` does).
const PROFILE_VERSION: u32 = 1;

/// One distinct owned card as the deck-builder grid sees it: the card, how
/// many copies are owned, and how many are currently in the built deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardEntry {
    pub card: Card,
    pub owned: usize,
    pub in_deck: usize,
}

/// The player's persistent profile. Every field carries a `#[serde(default)]`
/// so a partial or older file still loads (a missing collection/deck fills
/// from the starter), matching `settings.rs`'s additive-field tolerance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default = "starter_collection")]
    collection: Vec<Card>,
    #[serde(default = "starter_deck")]
    deck: Vec<Card>,
    /// Campaign progress (spec 009). Additive and serde-defaulted, so a
    /// pre-campaign profile loads with a fresh (empty) run — no version bump.
    #[serde(default)]
    campaign: CampaignRun,
    /// The player's credit balance (spec 012, economy; spec 021's wagers spend
    /// and grow it). Additive and serde-defaulted, so a pre-economy profile
    /// loads with 0 — no version bump. A *fresh* profile starts with
    /// [`economy::SEED_PURSE`] instead (see `Default`), which is why the field
    /// default and the struct default differ on purpose.
    #[serde(default)]
    credits: u32,
    /// Lifetime statistics (spec 020). Additive and serde-defaulted, so a
    /// pre-stats profile loads all-zero — no version bump.
    #[serde(default)]
    stats: LifetimeStats,
}

fn default_version() -> u32 {
    PROFILE_VERSION
}

/// The side deck a fresh profile starts with — today's default pool, so a
/// player who never opens the builder plays exactly as before this spec.
fn starter_deck() -> Vec<Card> {
    DEFAULT_SIDE_DECK.to_vec()
}

/// The cards a fresh profile owns: the starter deck plus a few spare adjusters,
/// so building is a real choice from the first launch. Tunable balance data —
/// spec C's economy is what actually grows the collection; the deck is always
/// a sub-multiset of this.
fn starter_collection() -> Vec<Card> {
    let mut cards = starter_deck();
    cards.extend([Card::Plus(1), Card::Minus(1), Card::PlusMinus(2)]);
    cards
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            collection: starter_collection(),
            deck: starter_deck(),
            campaign: CampaignRun::default(),
            credits: economy::SEED_PURSE,
            stats: LifetimeStats::default(),
        }
    }
}

impl Profile {
    /// Load the profile from disk, or the starter profile on any error —
    /// missing dir/file, unreadable, malformed, or an incompatible version.
    /// Never panics (like `Settings::load`).
    pub fn load() -> Self {
        Self::path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| Self::from_json(&text))
            .unwrap_or_default()
    }

    /// Save the profile. Best-effort: a missing data dir or unwritable path is
    /// swallowed — a profile you can't write isn't worth crashing over.
    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Reset to a brand-new starter profile: starter collection + deck, no
    /// campaign progress, the seed purse — a full fresh start (spec 014's New
    /// Campaign). Lifetime stats survive the reset (spec 020) — like Settings,
    /// which live in a separate file and are also untouched — so a New Campaign
    /// doesn't erase the player's cross-run record. The caller persists (`save`)
    /// and clears any in-progress match save.
    pub fn reset_to_starter(&mut self) {
        let stats = std::mem::take(&mut self.stats);
        *self = Profile::default();
        self.stats = stats;
    }

    /// Parse profile JSON, discarding a document whose version doesn't match
    /// rather than mis-reading it. The filesystem-free core of `load`, so the
    /// fallback and version check are testable without disk.
    fn from_json(text: &str) -> Option<Self> {
        let profile: Profile = serde_json::from_str(text).ok()?;
        (profile.version == PROFILE_VERSION).then_some(profile)
    }

    /// `<data_dir>/profile.json`, beside the match save's `saves/` subfolder.
    fn path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "kaazap").map(|dirs| dirs.data_dir().join("profile.json"))
    }

    /// The built side deck, for dealing a match hand.
    pub fn deck(&self) -> &[Card] {
        &self.deck
    }

    /// The campaign run — progress, cursor position, and the in-flight match.
    pub fn campaign(&self) -> &CampaignRun {
        &self.campaign
    }

    /// Mutable campaign run, for recording progress / the in-flight pointer.
    /// Callers pair a mutation with [`Profile::save`], as with deck edits.
    pub fn campaign_mut(&mut self) -> &mut CampaignRun {
        &mut self.campaign
    }

    /// The player's credit balance (spec 012, economy).
    pub fn credits(&self) -> u32 {
        self.credits
    }

    /// Add credits (from a campaign win). Saturating, so a long streak can't
    /// wrap. Callers pair this with [`Profile::save`].
    pub fn earn_credits(&mut self, amount: u32) {
        self.credits = self.credits.saturating_add(amount);
    }

    /// Stake a campaign match and mark it in flight (spec 021). The escrow: the
    /// node's stake leaves the balance now and only comes back through
    /// [`Profile::settle_campaign_match`]. Refuses — returning `false` and
    /// changing nothing — if the balance can't cover the stake; the wager
    /// prompt gates on the same balance, so that's a guard, not a path.
    /// Callers pair this with [`Profile::save`].
    pub fn stake_match(&mut self, node: NodeRef) -> bool {
        if node.stake > self.credits {
            return false;
        }
        self.credits -= node.stake;
        self.campaign.set_in_progress(Some(node));
        true
    }

    /// Settle the campaign match in flight (spec 021): take the stake out of
    /// escrow, pay [`economy::win_payout`] on a win, mark the opponent beaten,
    /// and count a campaign completion on the `!was_complete && run_complete()`
    /// edge — so a rematch of an already-beaten node changes no progress and
    /// never re-counts a completion. Returns `None` when there is no campaign
    /// pointer (a Quick Play match), leaving the balance untouched.
    ///
    /// Paying exactly once is a data property, not an ordering rule:
    /// [`CampaignRun::take_stake`] zeroes the escrow, so a second settlement
    /// pays `win_payout(0) == 0` and `mark_beaten` is idempotent. Callers pair
    /// this with [`Profile::save`].
    pub fn settle_campaign_match(&mut self, player_won: bool) -> Option<StakeOutcome> {
        let node = self.campaign.in_progress()?.clone();
        let stake = self.campaign.take_stake();
        if player_won {
            self.credits = self.credits.saturating_add(economy::win_payout(stake));
            let was_complete = self.campaign.run_complete();
            self.campaign.mark_beaten(&node.planet, &node.opponent);
            if !was_complete && self.campaign.run_complete() {
                self.stats.record_campaign_completion();
            }
            Some(StakeOutcome::Won(stake))
        } else {
            Some(StakeOutcome::Lost(stake))
        }
    }

    /// Whether the player can no longer afford any launchable match — spec
    /// 021's run-over condition. A pure predicate over the balance and the
    /// cheapest ante on the map, evaluated at the app's two spec'd seams.
    pub fn is_broke(&self) -> bool {
        self.credits < economy::cheapest_floor(&self.campaign)
    }

    /// Whether `price` is spendable: it must leave the cheapest launchable ante
    /// behind, so a purchase can never strand the player (spec 021's shop
    /// reserve). One rule in one place — [`Profile::try_purchase`] enforces it
    /// and the shop's dimming reads it.
    pub fn can_afford(&self, price: u32) -> bool {
        self.credits >= price.saturating_add(economy::cheapest_floor(&self.campaign))
    }

    /// The player's lifetime statistics (spec 020).
    pub fn stats(&self) -> &LifetimeStats {
        &self.stats
    }

    /// Record a completed match (spec 020). Always bumps lifetime stats for the
    /// given mode; for a Campaign match it also updates the run tally. It does
    /// *not* count campaign completions — since spec 021 a beaten node can be
    /// replayed, so completion is the `mark_beaten` edge owned by
    /// [`Profile::settle_campaign_match`]. Callers pair this with
    /// [`Profile::save`].
    pub fn record_match(
        &mut self,
        mode: Mode,
        opponent_id: &str,
        player_won: bool,
        player_rounds: u32,
        opp_rounds: u32,
    ) {
        self.stats
            .record_match(mode, opponent_id, player_won, player_rounds, opp_rounds);
        if let Mode::Campaign = mode {
            self.campaign
                .run_stats_mut()
                .record_match(player_won, player_rounds, opp_rounds);
        }
    }

    /// How many distinct side-card types the player owns (spec 020) — one per
    /// grid row, since `collection_by_type` lists each owned type once.
    pub fn distinct_side_cards_owned(&self) -> usize {
        self.collection_by_type().len()
    }

    /// Grant one copy of `card` to the collection (a win drop or a shop buy).
    /// The deck-builder and every count query pick it up automatically, since
    /// the collection is a bag of copies.
    pub fn grant_card(&mut self, card: Card) {
        self.collection.push(card);
    }

    /// Buy `card` for `price`: spend the credits and grant the card if the
    /// player can afford it — [`Profile::can_afford`], so the ante reserve is
    /// held back — otherwise do nothing. Only `price` is deducted, never the
    /// reserve. Returns whether it happened, so the app persists only on `true`
    /// (the deck-edit pattern).
    pub fn try_purchase(&mut self, card: Card, price: u32) -> bool {
        if self.can_afford(price) {
            self.credits -= price;
            self.grant_card(card);
            true
        } else {
            false
        }
    }

    /// Apply a campaign win's reward: earn credits scaled by the beaten
    /// opponent's `threshold` and drop one card from the current depth-gated
    /// pool (chosen by `roll`), returning what was granted (for the map reveal).
    /// Keeps the whole reward application testable off one injected roll, so the
    /// `App::tick` seam that calls it stays a thin wrapper. See `docs/economy.md`.
    pub fn apply_win_reward(&mut self, threshold: usize, roll: usize) -> WinReward {
        let pool = economy::available_pool(&self.campaign);
        let reward = economy::win_reward(threshold, &pool, roll);
        self.earn_credits(reward.credits);
        self.grant_card(reward.card);
        reward
    }

    /// A legal deck is exactly `SIDE_DECK_SIZE` cards, each backed by an owned
    /// copy (a sub-multiset of the collection). The rule a match start checks.
    pub fn deck_is_valid(&self) -> bool {
        self.deck.len() == SIDE_DECK_SIZE
            && self
                .deck
                .iter()
                .all(|&card| self.count_in_deck(card) <= self.count_owned(card))
    }

    /// Add one copy of `card` to the deck. Fails (returns `false`) if the deck
    /// is already full or the player owns no unused copy of that card.
    pub fn try_add_to_deck(&mut self, card: Card) -> bool {
        let has_spare = self.count_in_deck(card) < self.count_owned(card);
        if self.deck.len() < SIDE_DECK_SIZE && has_spare {
            self.deck.push(card);
            true
        } else {
            false
        }
    }

    /// Remove one copy of `card` from the deck. Fails if none is in the deck.
    pub fn remove_from_deck(&mut self, card: Card) -> bool {
        if let Some(i) = self.deck.iter().position(|&c| c == card) {
            self.deck.remove(i);
            true
        } else {
            false
        }
    }

    /// The collection as deck-builder rows: each distinct owned card in the
    /// canonical `ALL_SIDE_CARDS` order, with its owned and in-deck counts.
    /// Unowned cards are omitted (nothing to build with yet in this spec).
    pub fn collection_by_type(&self) -> Vec<CardEntry> {
        ALL_SIDE_CARDS
            .iter()
            .filter_map(|&card| {
                let owned = self.count_owned(card);
                (owned > 0).then_some(CardEntry {
                    card,
                    owned,
                    in_deck: self.count_in_deck(card),
                })
            })
            .collect()
    }

    /// How many copies of `card` the player owns — for the shop to show an
    /// "owned ×N" tally against a card that may not be in the deck-builder grid
    /// yet (the grid omits unowned cards; the shop lists the whole pool).
    pub fn owned_count(&self, card: Card) -> usize {
        self.count_owned(card)
    }

    fn count_owned(&self, card: Card) -> usize {
        self.collection.iter().filter(|&&c| c == card).count()
    }

    fn count_in_deck(&self, card: Card) -> usize {
        self.deck.iter().filter(|&&c| c == card).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A profile with an explicit collection and deck, current version — for
    /// exercising the deck-building rules without the starter's contents.
    fn profile_with(collection: Vec<Card>, deck: Vec<Card>) -> Profile {
        Profile {
            version: PROFILE_VERSION,
            collection,
            deck,
            campaign: CampaignRun::default(),
            credits: 0,
            stats: LifetimeStats::default(),
        }
    }

    /// A starter profile with an explicit balance, so the staking tests read as
    /// their own arithmetic rather than as offsets from the seed purse.
    fn profile_with_credits(credits: u32) -> Profile {
        Profile {
            credits,
            ..Profile::default()
        }
    }

    /// A campaign match pointer with a stake.
    fn node(planet: &str, opponent: &str, stake: u32) -> NodeRef {
        NodeRef {
            planet: planet.to_string(),
            opponent: opponent.to_string(),
            stake,
        }
    }

    #[test]
    fn campaign_state_round_trips_and_defaults_for_older_profiles() {
        use crate::campaign::planet_by_id;
        let mut p = Profile::default();
        p.campaign_mut().mark_beaten("cinder", "greeb");
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert!(p2.campaign().planet_cleared(&planet_by_id("cinder").unwrap()));

        // A pre-009 profile (no `campaign` field) loads with a fresh empty run.
        let older = r#"{"version":1,"collection":[],"deck":[]}"#;
        let p3 = Profile::from_json(older).expect("an older profile still loads");
        assert!(!p3.campaign().run_complete());
        assert!(!p3.campaign().planet_cleared(&planet_by_id("cinder").unwrap()));
    }

    #[test]
    fn reset_to_starter_wipes_the_run_but_preserves_lifetime_stats() {
        let mut p = Profile::default();
        // Dirty every persisted field: campaign progress, an in-flight match,
        // credits, the collection — and lifetime stats plus the run tally.
        p.campaign_mut().mark_beaten("cinder", "greeb");
        p.campaign_mut().set_in_progress(Some(node("scree", "dax", 0)));
        p.earn_credits(250);
        p.grant_card(Card::PlusMinus(6));
        p.record_match(Mode::Campaign, "greeb", true, 3, 1);
        assert!(p.campaign().has_progress() && p.credits() > 0, "sanity: profile is dirtied");
        assert_eq!(p.stats().campaign().get("greeb").match_wins, 1, "sanity: stats recorded");
        assert_eq!(p.campaign().run_stats().match_wins, 1, "sanity: run tally recorded");

        p.reset_to_starter();

        // The run resets to starter state...
        assert_eq!(p.credits(), economy::SEED_PURSE, "credits reset to the seed purse");
        assert!(!p.campaign().has_progress(), "campaign progress cleared");
        assert!(p.campaign().in_progress().is_none(), "in-progress match cleared");
        assert_eq!(p.campaign().run_stats().match_wins, 0, "run tally cleared");
        assert_eq!(p.deck(), starter_deck().as_slice(), "deck back to starter");
        assert_eq!(p.collection, Profile::default().collection, "collection back to starter");
        // ...but lifetime stats survive the reset (like Settings, in a separate file).
        assert_eq!(
            p.stats().campaign().get("greeb").match_wins,
            1,
            "lifetime stats are preserved across a reset",
        );
    }

    #[test]
    fn credits_seed_a_fresh_profile_but_default_to_zero_for_older_profiles() {
        // A brand-new profile opens with the seed purse (spec 021)...
        assert_eq!(Profile::default().credits(), economy::SEED_PURSE);

        // ...while the *serde* field default stays 0, so a pre-economy profile
        // (no `credits` key) still loads with an empty balance — the same
        // additive-field discipline as `campaign`, and no version bump.
        let older = r#"{"version":1,"collection":[],"deck":[]}"#;
        let p = Profile::from_json(older).expect("an older profile still loads");
        assert_eq!(p.credits(), 0);
        assert_eq!(PROFILE_VERSION, 1, "the seed purse is no on-disk shape change");

        // An existing profile keeps whatever balance it had.
        let existing = r#"{"version":1,"collection":[],"deck":[],"credits":75}"#;
        let p = Profile::from_json(existing).expect("an existing profile still loads");
        assert_eq!(p.credits(), 75);

        // Earned credits round-trip through JSON.
        let mut p = profile_with_credits(0);
        p.earn_credits(75);
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(p2.credits(), 75);
    }

    #[test]
    fn staking_escrows_the_credits_and_records_the_match_in_flight() {
        let mut p = profile_with_credits(50);
        assert!(p.stake_match(node("cinder", "greeb", 20)));
        assert_eq!(p.credits(), 30, "the stake leaves the balance immediately");
        assert_eq!(p.campaign().in_progress(), Some(&node("cinder", "greeb", 20)));
        assert_eq!(p.campaign().stake_at_risk(), Some(20));

        // A stake the balance can't cover is refused, changing nothing.
        let mut q = profile_with_credits(50);
        assert!(!q.stake_match(node("cinder", "greeb", 60)));
        assert_eq!(q.credits(), 50);
        assert!(q.campaign().in_progress().is_none());
    }

    #[test]
    fn settling_a_win_pays_double_the_stake_and_marks_the_node_beaten() {
        let mut p = profile_with_credits(50);
        assert!(p.stake_match(node("cinder", "greeb", 20)));
        assert_eq!(p.credits(), 30);

        assert_eq!(p.settle_campaign_match(true), Some(StakeOutcome::Won(20)));
        assert_eq!(p.credits(), 70, "the stake back plus even-money winnings");
        assert!(p.campaign().is_opponent_beaten("cinder", "greeb"));
        assert_eq!(
            p.campaign().in_progress().map(|n| n.stake),
            Some(0),
            "the escrow is emptied by the settlement",
        );
        assert_eq!(p.campaign().stake_at_risk(), None);

        // Settling twice can't pay twice — the escrow is already empty.
        assert_eq!(p.settle_campaign_match(true), Some(StakeOutcome::Won(0)));
        assert_eq!(p.credits(), 70);
    }

    #[test]
    fn settling_a_loss_keeps_the_stake_and_leaves_the_node_unbeaten() {
        let mut p = profile_with_credits(50);
        assert!(p.stake_match(node("cinder", "greeb", 20)));

        assert_eq!(p.settle_campaign_match(false), Some(StakeOutcome::Lost(20)));
        assert_eq!(p.credits(), 30, "a loss pays nothing back");
        assert!(!p.campaign().is_opponent_beaten("cinder", "greeb"));
        assert_eq!(p.campaign().stake_at_risk(), None);

        assert_eq!(p.settle_campaign_match(false), Some(StakeOutcome::Lost(0)));
        assert_eq!(p.credits(), 30);
    }

    #[test]
    fn settling_without_a_pointer_is_a_no_op_and_a_zero_stake_pointer_still_settles() {
        // Quick Play: no campaign pointer, so there is nothing to settle.
        let mut p = profile_with_credits(50);
        assert_eq!(p.settle_campaign_match(true), None);
        assert_eq!(p.settle_campaign_match(false), None);
        assert_eq!(p.credits(), 50);
        assert!(!p.campaign().has_progress());

        // A pre-021 save's pointer carries no stake: the win still marks the
        // node beaten, and pays nothing.
        let mut q = profile_with_credits(50);
        assert!(q.stake_match(node("cinder", "greeb", 0)));
        assert_eq!(q.credits(), 50);
        assert_eq!(q.campaign().stake_at_risk(), None);
        assert_eq!(q.settle_campaign_match(true), Some(StakeOutcome::Won(0)));
        assert_eq!(q.credits(), 50);
        assert!(q.campaign().is_opponent_beaten("cinder", "greeb"));
    }

    #[test]
    fn a_rematch_settles_for_credits_but_changes_no_progress_or_completions() {
        use crate::campaign::PLANETS;
        let mut p = profile_with_credits(100);
        for planet in PLANETS {
            for opp in planet.opponents {
                p.campaign_mut().mark_beaten(planet.id, opp);
            }
        }
        assert!(p.campaign().run_complete(), "sanity: the run is complete");
        assert_eq!(p.stats().campaign_completions(), 0, "sanity: nothing settled yet");

        // A rematch win against an already-beaten final node pays...
        assert!(p.stake_match(node("zenith", "sovereign", 50)));
        assert_eq!(p.settle_campaign_match(true), Some(StakeOutcome::Won(50)));
        assert_eq!(p.credits(), 150);
        // ...but counts no completion and un-beats nothing.
        assert_eq!(p.stats().campaign_completions(), 0, "a rematch never completes the run");
        assert!(p.campaign().run_complete());

        // A rematch loss costs the stake and likewise touches no progress.
        assert!(p.stake_match(node("zenith", "sovereign", 50)));
        assert_eq!(p.settle_campaign_match(false), Some(StakeOutcome::Lost(50)));
        assert_eq!(p.credits(), 100);
        assert_eq!(p.stats().campaign_completions(), 0);
        assert!(p.campaign().run_complete(), "a loss un-beats nothing");
    }

    #[test]
    fn is_broke_reads_the_balance_against_the_cheapest_launchable_ante() {
        use crate::campaign::PLANETS;
        let mut p = profile_with_credits(9);
        assert_eq!(economy::cheapest_floor(p.campaign()), 10, "sanity: Cinder sets the floor");
        assert!(p.is_broke(), "a credit short of the cheapest ante is broke");
        p.earn_credits(1);
        assert!(!p.is_broke(), "exactly the cheapest ante is still playable");

        // A complete run keeps its rematch floor, so the same balance is fine.
        for planet in PLANETS {
            for opp in planet.opponents {
                p.campaign_mut().mark_beaten(planet.id, opp);
            }
        }
        assert!(p.campaign().run_complete());
        assert_eq!(p.credits(), 10);
        assert!(!p.is_broke(), "a cleared run can always be replayed at the floor");

        // A match in flight doesn't change the answer — the predicate reads the
        // balance that's left after the escrow, nothing else.
        let mut in_flight = profile_with_credits(19);
        assert!(in_flight.stake_match(node("cinder", "greeb", 10)));
        assert_eq!(in_flight.credits(), 9);
        assert!(in_flight.is_broke());
        let mut in_flight = profile_with_credits(20);
        assert!(in_flight.stake_match(node("cinder", "greeb", 10)));
        assert!(!in_flight.is_broke());
    }

    #[test]
    fn stats_persist_and_default_to_empty_for_older_profiles() {
        // A pre-020 profile (no `stats`, no `run_stats` keys) loads with empty
        // stats and no version bump — the same additive-field discipline as
        // `credits` / `campaign`.
        let older = r#"{"version":1,"collection":[],"deck":[]}"#;
        let p = Profile::from_json(older).expect("an older profile still loads");
        assert_eq!(p.stats().campaign_completions(), 0);
        assert_eq!(p.stats().overall_streak().current, 0);
        assert_eq!(p.campaign().run_stats().match_wins, 0);
        assert_eq!(PROFILE_VERSION, 1, "no version bump for the additive stats field");

        // Dirtied lifetime + run stats round-trip through a full Profile.
        let mut p = Profile::default();
        p.record_match(Mode::QuickPlay, "yuka", true, 3, 1);
        p.record_match(Mode::Campaign, "greeb", false, 1, 3);
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(p2.stats().quick_play().get("yuka").match_wins, 1);
        assert_eq!(p2.stats().campaign().get("greeb").match_losses, 1);
        assert_eq!(p2.stats().overall_streak().longest, 1);
        // The Campaign match also fed the run tally, which round-trips too.
        assert_eq!(p2.campaign().run_stats().match_losses, 1);
        assert_eq!(p2.campaign().run_stats().round_losses, 3);
    }

    #[test]
    fn campaign_completion_counts_only_a_final_clearing_win_and_recounts_after_reset() {
        use crate::campaign::PLANETS;
        // Clear a node the way the real seam does: stake it, record the match,
        // settle the win. Settlement — not `record_match` — owns `mark_beaten`
        // and the completion edge.
        fn win_node(p: &mut Profile, planet: &str, opponent: &str) {
            assert!(p.stake_match(node(planet, opponent, 10)));
            p.record_match(Mode::Campaign, opponent, true, 3, 0);
            assert_eq!(p.settle_campaign_match(true), Some(StakeOutcome::Won(10)));
        }

        // Beat every campaign opponent except the final boss; none of these
        // completes the run.
        fn clear_all_but_last(p: &mut Profile) {
            for planet in PLANETS {
                for opp in planet.opponents {
                    if planet.id == "zenith" && *opp == "sovereign" {
                        continue;
                    }
                    win_node(p, planet.id, opp);
                }
            }
        }

        let mut p = Profile::default();
        clear_all_but_last(&mut p);
        assert_eq!(p.stats().campaign_completions(), 0, "no completion until the final node clears");

        // Recording the final match on its own never increments — the clause
        // lives in settlement now (spec 021's rematches made it unsound here).
        p.record_match(Mode::Campaign, "sovereign", true, 3, 2);
        assert_eq!(p.stats().campaign_completions(), 0, "record_match alone counts nothing");

        // Settling the final clearing win increments exactly once.
        win_node(&mut p, "zenith", "sovereign");
        assert!(p.campaign().run_complete());
        assert_eq!(p.stats().campaign_completions(), 1, "the final clearing win completes the run");
        // ...and recording another match on the completed run doesn't re-count.
        p.record_match(Mode::Campaign, "sovereign", true, 3, 0);
        assert_eq!(p.stats().campaign_completions(), 1, "record_match never increments");

        // A fresh run (post-reset) can complete again and increment a second time.
        p.reset_to_starter();
        assert_eq!(p.stats().campaign_completions(), 1, "completions survive the reset");
        clear_all_but_last(&mut p);
        win_node(&mut p, "zenith", "sovereign");
        assert_eq!(p.stats().campaign_completions(), 2, "a second full run increments again");
    }

    #[test]
    fn earning_grows_the_balance_and_purchase_holds_back_the_ante_reserve() {
        let owned = |p: &Profile, card: Card| {
            p.collection_by_type()
                .iter()
                .find(|e| e.card == card)
                .map_or(0, |e| e.owned)
        };
        let mut p = profile_with_credits(50);
        let before = owned(&p, Card::PlusMinus(6));
        p.earn_credits(9);
        assert_eq!(p.credits(), 59);
        assert_eq!(economy::cheapest_floor(p.campaign()), 10, "sanity: a fresh run's floor");

        // 59 covers the 50-credit price but would leave 9 — under the cheapest
        // ante, so the shop refuses it and reads it as unaffordable.
        assert!(!p.can_afford(50));
        assert!(!p.try_purchase(Card::PlusMinus(6), 50));
        assert_eq!(p.credits(), 59, "a refused purchase changes nothing");
        assert_eq!(owned(&p, Card::PlusMinus(6)), before);

        // One credit more and the purchase clears the reserve exactly.
        p.earn_credits(1);
        assert!(p.can_afford(50));
        assert!(p.try_purchase(Card::PlusMinus(6), 50));
        assert_eq!(p.credits(), 10, "only the price is deducted — the reserve is never spent");
        assert_eq!(owned(&p, Card::PlusMinus(6)), before + 1);

        // grant_card alone also grows the collection (a win drop).
        let plus2_before = owned(&p, Card::Plus(2));
        p.grant_card(Card::Plus(2));
        assert_eq!(owned(&p, Card::Plus(2)), plus2_before + 1);
    }

    #[test]
    fn a_staked_match_in_flight_round_trips_through_the_profile_json() {
        let mut p = profile_with_credits(50);
        assert!(p.stake_match(node("scree", "dax", 20)));
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(p2.credits(), 30);
        assert_eq!(p2.campaign().in_progress(), Some(&node("scree", "dax", 20)));
        assert_eq!(p2.campaign().stake_at_risk(), Some(20));
    }

    #[test]
    fn applying_a_win_reward_pays_credits_and_drops_one_pool_card() {
        let mut p = Profile::default(); // fresh: Outer depth, the seed purse
        let before_total: usize = p.collection_by_type().iter().map(|e| e.owned).sum();

        // Threshold 15 → 10 credits; roll 0 picks the first card of the pool.
        let reward = p.apply_win_reward(15, 0);

        assert_eq!(reward.credits, 10);
        assert_eq!(p.credits(), economy::SEED_PURSE + 10);
        // The dropped card comes from the current (Outer) depth-gated pool...
        assert!(economy::available_pool(p.campaign()).contains(&reward.card));
        // ...and exactly one card was added to the collection.
        let after_total: usize = p.collection_by_type().iter().map(|e| e.owned).sum();
        assert_eq!(after_total, before_total + 1);
    }

    #[test]
    fn default_profile_has_a_valid_deck_within_the_collection() {
        let p = Profile::default();
        assert_eq!(p.deck().len(), SIDE_DECK_SIZE);
        assert!(p.deck_is_valid());
        // The starter deck is the old default pool, so an untouched profile
        // plays exactly as before this spec.
        assert_eq!(p.deck(), &DEFAULT_SIDE_DECK);
    }

    #[test]
    fn json_round_trip_preserves_collection_and_deck() {
        let p = Profile::default();
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(p2.collection, p.collection);
        assert_eq!(p2.deck, p.deck);
    }

    #[test]
    fn missing_or_garbage_json_is_rejected_but_an_empty_object_is_the_starter() {
        // Unparseable / wrong-shape input → None (load() then uses the
        // starter). A field-less object fills every field from its serde
        // default, yielding a valid current-version starter profile.
        assert!(Profile::from_json("").is_none());
        assert!(Profile::from_json("not json").is_none());
        assert!(Profile::from_json("[1,2,3]").is_none());

        let starter = Profile::from_json("{}").expect("a field-less profile loads as the starter");
        assert!(starter.deck_is_valid());
        assert_eq!(starter.collection, Profile::default().collection);
        assert_eq!(starter.deck, Profile::default().deck);
    }

    #[test]
    fn a_wrong_version_document_is_discarded() {
        let mut val: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&Profile::default()).unwrap()).unwrap();
        val["version"] = serde_json::json!(PROFILE_VERSION + 1);
        assert!(Profile::from_json(&val.to_string()).is_none());
    }

    #[test]
    fn try_add_respects_ownership() {
        // Own two +1 and one -1, deck empty.
        let mut p = profile_with(
            vec![Card::Plus(1), Card::Plus(1), Card::Minus(1)],
            vec![],
        );
        assert!(p.try_add_to_deck(Card::Plus(1)));
        assert!(p.try_add_to_deck(Card::Plus(1)));
        assert!(!p.try_add_to_deck(Card::Plus(1)), "no third +1 is owned");
        assert!(p.try_add_to_deck(Card::Minus(1)));
        assert!(!p.try_add_to_deck(Card::PlusMinus(6)), "±6 isn't owned at all");
        assert_eq!(p.deck().len(), 3);
    }

    #[test]
    fn try_add_blocks_at_the_deck_size_cap() {
        // A big collection so ownership never limits — only the cap should.
        let mut p = profile_with(vec![Card::Plus(1); SIDE_DECK_SIZE + 5], vec![]);
        for _ in 0..SIDE_DECK_SIZE {
            assert!(p.try_add_to_deck(Card::Plus(1)));
        }
        assert_eq!(p.deck().len(), SIDE_DECK_SIZE);
        assert!(p.deck_is_valid());
        assert!(!p.try_add_to_deck(Card::Plus(1)), "the deck is full");
    }

    #[test]
    fn remove_takes_one_copy_and_reports_absence() {
        let mut p = profile_with(
            vec![Card::Plus(1), Card::Plus(1)],
            vec![Card::Plus(1), Card::Plus(1)],
        );
        assert!(p.remove_from_deck(Card::Plus(1)));
        assert_eq!(p.deck().len(), 1);
        assert!(p.remove_from_deck(Card::Plus(1)));
        assert!(!p.remove_from_deck(Card::Plus(1)), "none left to remove");
    }

    #[test]
    fn collection_by_type_groups_counts_in_canonical_order() {
        let p = profile_with(
            vec![Card::Plus(1), Card::Plus(1), Card::Minus(1), Card::PlusMinus(2)],
            vec![Card::Plus(1), Card::PlusMinus(2)],
        );
        let entries = p.collection_by_type();
        // Distinct owned types only, in ALL_SIDE_CARDS order: +1, -1, ±2.
        assert_eq!(
            entries.iter().map(|e| e.card).collect::<Vec<_>>(),
            vec![Card::Plus(1), Card::Minus(1), Card::PlusMinus(2)]
        );
        assert_eq!((entries[0].owned, entries[0].in_deck), (2, 1)); // +1
        assert_eq!((entries[1].owned, entries[1].in_deck), (1, 0)); // -1
        assert_eq!((entries[2].owned, entries[2].in_deck), (1, 1)); // ±2
    }

    #[test]
    fn a_short_deck_is_invalid() {
        let mut p = Profile::default();
        let first = p.deck()[0];
        assert!(p.remove_from_deck(first));
        assert!(!p.deck_is_valid(), "a deck under SIDE_DECK_SIZE isn't playable");
    }

    #[test]
    fn a_full_deck_that_over_owns_a_card_is_invalid() {
        // Isolates the sub-multiset clause from the length one: the deck is a
        // full SIDE_DECK_SIZE (length clause passes) but holds two +1 when only
        // one is owned, so it must still be invalid. Guards against a refactor
        // that dropped the ownership check (length alone would wrongly pass).
        let p = profile_with(
            vec![
                Card::Plus(1), Card::Plus(2), Card::Plus(3), Card::Plus(4), Card::Minus(1),
                Card::Minus(2), Card::Minus(3), Card::Minus(4), Card::PlusMinus(1), Card::PlusMinus(2),
            ],
            vec![
                Card::Plus(1), Card::Plus(1), Card::Plus(3), Card::Plus(4), Card::Minus(1),
                Card::Minus(2), Card::Minus(3), Card::Minus(4), Card::PlusMinus(1), Card::PlusMinus(2),
            ],
        );
        assert_eq!(p.deck().len(), SIDE_DECK_SIZE); // the length clause passes
        assert!(!p.deck_is_valid(), "over-owning a card must fail the sub-multiset check");
    }
}
