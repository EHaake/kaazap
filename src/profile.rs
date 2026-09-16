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
//! The starter a fresh (or reset) profile begins with lives here too, as
//! [`STARTER_SIDE_DECK`] + [`STARTER_SPARES`] — Outer-tier balance data, tuned
//! by spec 022, distinct from `card::DEFAULT_SIDE_DECK` (the standard deck).
//! See `specs/008-side-deck-customization` and `specs/022-balance-pass`.

use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    SIDE_DECK_SIZE,
    campaign::{CampaignRun, NodeRef},
    card::{ALL_SIDE_CARDS, Card},
    economy::{self, StakeOutcome},
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

/// How a finished campaign match settled (spec 024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settlement {
    pub outcome: StakeOutcome,
    /// Whether this settlement was the win that **completed the run** — the
    /// `!was_complete && run_complete()` edge, so a rematch is never one and a
    /// replayed campaign's final win is. The once-per-completion signal the
    /// victory notice rides on.
    pub completed_run: bool,
}

/// The player's persistent profile: the collection and built deck, campaign
/// progress, credits, lifetime stats, and the onboarding seen-marks. Every
/// field carries a `#[serde(default)]` so a partial or older file still loads
/// (a missing collection/deck fills from the starter), matching
/// `settings.rs`'s additive-field tolerance.
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
    /// Onboarding seen-marks (spec 023): set when the primer / first-match
    /// popup is *dismissed*, never when shown. Additive and serde-defaulted
    /// false — no version bump — and, like `stats`, they survive
    /// [`Profile::reset_to_starter`].
    #[serde(default)]
    primer_seen: bool,
    #[serde(default)]
    first_match_seen: bool,
}

fn default_version() -> u32 {
    PROFILE_VERSION
}

/// The side deck a fresh or reset profile plays (spec 022): Outer-tier only,
/// so every shop tier above it is a real upgrade. Tunable balance data; the
/// tiered-ness is pinned by `default_profile_plays_a_valid_outer_tier_starter`.
pub const STARTER_SIDE_DECK: [Card; SIDE_DECK_SIZE] = [
    Card::Plus(1),
    Card::Plus(1),
    Card::Plus(1),
    Card::Plus(1),
    Card::Plus(2),
    Card::Plus(2),
    Card::Minus(1),
    Card::Minus(1),
    Card::Minus(1),
    Card::Minus(2),
];

/// The Outer-tier spares a fresh profile owns beyond its deck, so the builder
/// is a real choice from the first launch (spec 008's intent, kept). Tuned
/// **lateral** rather than as upgrades in spec 022 (`±1` is the only type the
/// starter deck doesn't already hold), so the measured starter win rates in
/// `docs/balance.md` describe the deck a fresh player actually fields.
pub const STARTER_SPARES: [Card; 3] = [Card::PlusMinus(1), Card::Plus(2), Card::Minus(2)];

/// The side deck a fresh profile starts with — the Outer-tier starter.
fn starter_deck() -> Vec<Card> {
    STARTER_SIDE_DECK.to_vec()
}

/// The cards a fresh profile owns: the starter deck plus a few spare adjusters,
/// so building is a real choice from the first launch. Tunable balance data —
/// the economy is what actually grows the collection; the deck is always a
/// sub-multiset of this.
fn starter_collection() -> Vec<Card> {
    let mut cards = starter_deck();
    cards.extend(STARTER_SPARES);
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
            primer_seen: false,
            first_match_seen: false,
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
    /// campaign progress, the seed purse — the full wipe. Since spec 024 this
    /// is **Reset Everything**'s operation and the run-over reset's, not New
    /// Campaign's, which resets the map only
    /// ([`Profile::reset_campaign_run`]). Lifetime stats survive the wipe (spec
    /// 020) — like Settings, which live in a separate file and are also
    /// untouched — so it doesn't erase the player's cross-run record. The
    /// onboarding seen-marks (spec 023) survive it for the same reason: a reset
    /// is a fresh run, not a first launch. The caller persists (`save`) and
    /// clears any in-progress match save.
    pub fn reset_to_starter(&mut self) {
        let stats = std::mem::take(&mut self.stats);
        let primer_seen = std::mem::take(&mut self.primer_seen);
        let first_match_seen = std::mem::take(&mut self.first_match_seen);
        *self = Profile::default();
        self.stats = stats;
        self.primer_seen = primer_seen;
        self.first_match_seen = first_match_seen;
    }

    /// Reset the campaign map only — spec 024's New Campaign: the beaten set,
    /// the in-flight pointer (and its escrowed stake) and the run tally all go
    /// with `CampaignRun::default()`; credits, collection, deck, lifetime stats
    /// and the onboarding marks are untouched. Supersedes spec 014's "New
    /// Campaign = full fresh start"; [`Profile::reset_to_starter`] is now
    /// reached only by Reset Everything and the run-over acknowledgement. The
    /// caller persists (`save`) and clears any in-progress match save.
    pub fn reset_campaign_run(&mut self) {
        self.campaign = CampaignRun::default();
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

    /// Resolve a finished match: record it into lifetime + run statistics, then
    /// settle any campaign stake. One method owns the **order** (spec 024): the
    /// first-clear record is captured on the completion edge from the run tally,
    /// so the completing match must already be recorded when settlement runs.
    /// `Mode` is derived here from the in-flight pointer — settlement never
    /// clears it, so reading it first or last is the same answer. Returns `None`
    /// for a Quick Play match (recorded, nothing to settle). Callers pair this
    /// with [`Profile::save`].
    pub fn resolve_match(
        &mut self,
        opponent_id: &str,
        player_won: bool,
        player_rounds: u32,
        opp_rounds: u32,
    ) -> Option<Settlement> {
        let mode = if self.campaign.in_progress().is_some() {
            Mode::Campaign
        } else {
            Mode::QuickPlay
        };
        self.record_match(mode, opponent_id, player_won, player_rounds, opp_rounds);
        let was_complete = self.campaign.run_complete();
        let outcome = self.settle_campaign_match(player_won)?;
        let tally = self.campaign.run_stats_mut();
        match outcome {
            StakeOutcome::Won(stake) => {
                tally.record_credits_won(economy::win_payout(stake).saturating_sub(stake))
            }
            StakeOutcome::Lost(stake) => tally.record_credits_lost(stake),
        }
        Some(Settlement {
            outcome,
            completed_run: !was_complete && self.campaign.run_complete(),
        })
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
    ///
    /// The completion edge hands the run tally's `matches_played()` to
    /// [`LifetimeStats::record_campaign_completion`] as the first-clear record
    /// (spec 024): the run's matches played *including* the completing match,
    /// which holds because [`Profile::resolve_match`] records before it
    /// settles. Private since spec 024 — callers go through `resolve_match`;
    /// settling alone moves no run tally and no credit counter.
    fn settle_campaign_match(&mut self, player_won: bool) -> Option<StakeOutcome> {
        let node = self.campaign.in_progress()?.clone();
        let stake = self.campaign.take_stake();
        if player_won {
            self.credits = self.credits.saturating_add(economy::win_payout(stake));
            let was_complete = self.campaign.run_complete();
            self.campaign.mark_beaten(&node.planet, &node.opponent);
            if !was_complete && self.campaign.run_complete() {
                let matches = self.campaign.run_stats().matches_played();
                self.stats.record_campaign_completion(matches);
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

    /// Whether anything the campaign-entry choices would affect exists (spec
    /// 024): the run has progress, or the pool differs from the starter —
    /// credits, collection or deck. A truly fresh profile opens the map
    /// directly.
    pub fn differs_from_starter(&self) -> bool {
        let starter = Profile::default();
        self.campaign.has_progress()
            || self.credits != starter.credits
            || self.collection != starter.collection
            || self.deck != starter.deck
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

    /// Whether the how-to-play primer has been dismissed (spec 023).
    pub fn primer_seen(&self) -> bool {
        self.primer_seen
    }

    /// Mark the primer dismissed. Callers pair this with [`Profile::save`], as
    /// with deck edits.
    pub fn mark_primer_seen(&mut self) {
        self.primer_seen = true;
    }

    /// Whether the first-match popup has been dismissed (spec 023).
    pub fn first_match_seen(&self) -> bool {
        self.first_match_seen
    }

    /// Mark the first-match popup dismissed. Callers pair this with
    /// [`Profile::save`], as with deck edits.
    pub fn mark_first_match_seen(&mut self) {
        self.first_match_seen = true;
    }

    /// Record a completed match (spec 020). Always bumps lifetime stats for the
    /// given mode; for a Campaign match it also updates the run tally. It does
    /// *not* count campaign completions — since spec 021 a beaten node can be
    /// replayed, so completion is the `mark_beaten` edge owned by
    /// [`Profile::settle_campaign_match`]. Private since spec 024 — callers go
    /// through [`Profile::resolve_match`], which pairs this with [`Profile::save`].
    fn record_match(
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
            primer_seen: false,
            first_match_seen: false,
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

    /// Play a campaign match the way the app's seam does (spec 024): stake it,
    /// then one [`Profile::resolve_match`] records it and settles the stake.
    fn play_node(
        p: &mut Profile,
        planet: &str,
        opponent: &str,
        stake: u32,
        player_won: bool,
    ) -> Settlement {
        assert!(p.stake_match(node(planet, opponent, stake)));
        let (player_rounds, opp_rounds) = if player_won { (3, 1) } else { (1, 3) };
        p.resolve_match(opponent, player_won, player_rounds, opp_rounds)
            .expect("a staked campaign match settles")
    }

    /// Win every campaign node in map order through [`play_node`], returning
    /// each settlement — a whole run driven through `resolve_match` only.
    fn sweep_run(p: &mut Profile, stake: u32) -> Vec<Settlement> {
        use crate::campaign::PLANETS;
        let mut settled = Vec::new();
        for planet in PLANETS {
            for opp in planet.opponents {
                settled.push(play_node(p, planet.id, opp, stake, true));
            }
        }
        settled
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
    fn reset_to_starter_wipes_the_run_but_preserves_lifetime_stats_and_onboarding_marks() {
        let mut p = Profile::default();
        // Dirty every persisted field: campaign progress, an in-flight match,
        // credits, the collection — lifetime stats plus the run tally, and the
        // onboarding seen-marks.
        p.campaign_mut().mark_beaten("cinder", "greeb");
        p.campaign_mut().set_in_progress(Some(node("scree", "dax", 0)));
        p.earn_credits(250);
        p.grant_card(Card::PlusMinus(6));
        p.record_match(Mode::Campaign, "greeb", true, 3, 1);
        p.mark_primer_seen();
        p.mark_first_match_seen();
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
        // ...and so do the onboarding marks: a reset is a fresh run, not a
        // first launch, so neither piece of onboarding comes back.
        assert!(p.primer_seen(), "the primer mark survives a reset");
        assert!(p.first_match_seen(), "the first-match mark survives a reset");
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
    fn onboarding_marks_default_unset_round_trip_and_load_unset_from_older_documents() {
        // A brand-new profile has seen neither piece of onboarding.
        let fresh = Profile::default();
        assert!(!fresh.primer_seen());
        assert!(!fresh.first_match_seen());

        // A pre-023 profile (no marks) loads with both unset — the same
        // additive-field discipline as `credits` / `campaign` / `stats`.
        let older = r#"{"version":1,"collection":[],"deck":[]}"#;
        let p = Profile::from_json(older).expect("an older profile still loads");
        assert!(!p.primer_seen());
        assert!(!p.first_match_seen());
        assert_eq!(PROFILE_VERSION, 1, "the seen-marks are no on-disk shape change");

        // Both marks round-trip through the profile JSON.
        let mut p = Profile::default();
        p.mark_primer_seen();
        p.mark_first_match_seen();
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert!(p2.primer_seen());
        assert!(p2.first_match_seen());

        // The marks are independent: one set doesn't set the other.
        let mut only_primer = Profile::default();
        only_primer.mark_primer_seen();
        let json = serde_json::to_string(&only_primer).unwrap();
        let p3 = Profile::from_json(&json).expect("a valid profile loads");
        assert!(p3.primer_seen());
        assert!(!p3.first_match_seen());
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
        assert_eq!(
            p.resolve_match("sovereign", true, 3, 1),
            Some(Settlement { outcome: StakeOutcome::Won(50), completed_run: false }),
        );
        assert_eq!(p.credits(), 150);
        // ...but counts no completion and un-beats nothing.
        assert_eq!(p.stats().campaign_completions(), 0, "a rematch never completes the run");
        assert!(p.campaign().run_complete());

        // A rematch loss costs the stake and likewise touches no progress.
        assert!(p.stake_match(node("zenith", "sovereign", 50)));
        assert_eq!(
            p.resolve_match("sovereign", false, 1, 3),
            Some(Settlement { outcome: StakeOutcome::Lost(50), completed_run: false }),
        );
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
        // Clear a node the way the real seam does: stake it, then resolve the
        // match — `resolve_match` records it and settles the stake in that
        // order (spec 024). Settlement — not `record_match` — owns
        // `mark_beaten` and the completion edge.
        fn win_node(p: &mut Profile, planet: &str, opponent: &str) {
            assert!(p.stake_match(node(planet, opponent, 10)));
            assert_eq!(
                p.resolve_match(opponent, true, 3, 0).map(|s| s.outcome),
                Some(StakeOutcome::Won(10)),
            );
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
    fn resolve_match_moves_the_run_credit_counters_and_nothing_else_does() {
        let net = economy::win_payout(20) - 20;

        // A staked win adds its net gain — the number the map banner shows.
        let mut p = profile_with_credits(100);
        assert_eq!(
            play_node(&mut p, "cinder", "greeb", 20, true).outcome,
            StakeOutcome::Won(20),
        );
        assert_eq!(p.campaign().run_stats().credits_won, net);
        assert_eq!(p.campaign().run_stats().credits_lost, 0);

        // Settling the same pointer again adds nothing — the escrow is empty.
        assert_eq!(
            p.resolve_match("greeb", true, 3, 1).map(|s| s.outcome),
            Some(StakeOutcome::Won(0)),
        );
        assert_eq!(p.campaign().run_stats().credits_won, net, "an emptied escrow pays nothing");
        assert_eq!(p.campaign().run_stats().credits_lost, 0);

        // A loss adds the forfeited stake, and nothing to the win counter.
        let mut q = profile_with_credits(100);
        assert_eq!(
            play_node(&mut q, "cinder", "greeb", 20, false).outcome,
            StakeOutcome::Lost(20),
        );
        assert_eq!(q.campaign().run_stats().credits_lost, 20);
        assert_eq!(q.campaign().run_stats().credits_won, 0);

        // A stake forfeited by discarding a saved match settles nothing, so it
        // counts toward neither counter.
        let mut r = profile_with_credits(100);
        assert!(r.stake_match(node("cinder", "greeb", 20)));
        r.campaign_mut().set_in_progress(None);
        assert_eq!(r.resolve_match("greeb", true, 3, 1), None, "nothing in flight settles");
        assert_eq!(r.campaign().run_stats().credits_won, 0);
        assert_eq!(r.campaign().run_stats().credits_lost, 0);

        // Both resets zero them — the tally lives on the run.
        p.reset_campaign_run();
        assert_eq!(p.campaign().run_stats().credits_won, 0, "a map reset zeroes the counters");
        assert_eq!(p.campaign().run_stats().credits_lost, 0);
        q.reset_to_starter();
        assert_eq!(q.campaign().run_stats().credits_won, 0, "a full reset zeroes them too");
        assert_eq!(q.campaign().run_stats().credits_lost, 0);
    }

    #[test]
    fn the_run_counters_and_first_clear_round_trip_and_default_for_older_profiles() {
        // Dirty both counters, then round-trip the whole profile.
        let mut p = profile_with_credits(100);
        play_node(&mut p, "cinder", "greeb", 20, true);
        play_node(&mut p, "scree", "dax", 30, false);
        let json = serde_json::to_string(&p).unwrap();
        let p2 = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(p2.campaign().run_stats().credits_won, economy::win_payout(20) - 20);
        assert_eq!(p2.campaign().run_stats().credits_lost, 30);

        // A pre-024 document loads with both counters zero and no record — the
        // same additive-field discipline as `credits` / `campaign` / `stats`.
        let older = r#"{"version":1,"collection":[],"deck":[]}"#;
        let old = Profile::from_json(older).expect("an older profile still loads");
        assert_eq!(old.campaign().run_stats().credits_won, 0);
        assert_eq!(old.campaign().run_stats().credits_lost, 0);
        assert_eq!(old.stats().first_clear_matches(), None);
        assert_eq!(PROFILE_VERSION, 1, "the counters and the record are no shape change");

        // The first-clear record is a lifetime number, so it round-trips on its
        // own — a profile whose recording run is long gone still carries it.
        let doc = r#"{"version":1,"collection":[],"deck":[],"stats":{"campaign_completions":2,"first_clear_matches":14}}"#;
        let recorded = Profile::from_json(doc).expect("a recorded profile loads");
        assert_eq!(recorded.stats().campaign_completions(), 2);
        assert_eq!(recorded.stats().first_clear_matches(), Some(14));
        let json = serde_json::to_string(&recorded).unwrap();
        let again = Profile::from_json(&json).expect("a valid profile loads");
        assert_eq!(again.stats().first_clear_matches(), Some(14));
    }

    #[test]
    fn the_first_clear_counts_the_completing_match_and_survives_a_replay() {
        let mut p = profile_with_credits(1000);
        // A loss along the way still counts toward the run's matches played.
        play_node(&mut p, "cinder", "greeb", 10, false);
        sweep_run(&mut p, 10);
        assert!(p.campaign().run_complete());

        let played = p.campaign().run_stats().matches_played();
        assert_eq!(played, 11, "ten campaign nodes plus the loss on the way");
        assert_eq!(p.stats().campaign_completions(), 1);
        assert_eq!(
            p.stats().first_clear_matches(),
            Some(played),
            "the record includes the completing match",
        );

        // A rematch win afterwards leaves both unchanged.
        play_node(&mut p, "zenith", "sovereign", 10, true);
        assert_eq!(p.stats().campaign_completions(), 1);
        assert_eq!(p.stats().first_clear_matches(), Some(played));

        // A second full run completes again, but never re-sets the record.
        p.reset_campaign_run();
        sweep_run(&mut p, 10);
        assert_eq!(p.stats().campaign_completions(), 2, "a replay completes again");
        assert_eq!(
            p.stats().first_clear_matches(),
            Some(played),
            "the record is the first clear only",
        );
    }

    #[test]
    fn resolve_match_reports_the_completion_edge_and_skips_quick_play() {
        let mut p = profile_with_credits(1000);
        let settled = sweep_run(&mut p, 10);
        let (last, rest) = settled.split_last().expect("a run has nodes");
        assert!(rest.iter().all(|s| !s.completed_run), "no earlier win completes the run");
        assert!(last.completed_run, "the final clearing win does");

        // A rematch win on the complete run is never a completion.
        assert!(!play_node(&mut p, "zenith", "sovereign", 10, true).completed_run);

        // No pointer means Quick Play: nothing settles, the lifetime stats
        // record it, and the run tally doesn't move.
        let played = p.campaign().run_stats().matches_played();
        p.campaign_mut().set_in_progress(None);
        assert_eq!(p.resolve_match("yuka", true, 3, 0), None, "a Quick Play match settles nothing");
        assert_eq!(p.stats().quick_play().get("yuka").match_wins, 1);
        assert_eq!(
            p.campaign().run_stats().matches_played(),
            played,
            "the run tally is campaign-only",
        );
    }

    #[test]
    fn the_completion_edge_and_the_completions_counter_always_agree() {
        use crate::campaign::PLANETS;
        // The edge is evaluated twice — once inside settlement for the
        // completions counter, once in `resolve_match` for the signal (plan
        // tension §1). Across a whole run they must never disagree.
        fn check(p: &mut Profile, planet: &str, opponent: &str, player_won: bool) {
            let before = p.stats().campaign_completions();
            let settled = play_node(p, planet, opponent, 10, player_won);
            let after = p.stats().campaign_completions();
            assert_eq!(
                settled.completed_run,
                after == before + 1,
                "{planet}/{opponent}: the signal and the completions counter disagree",
            );
        }

        let mut p = profile_with_credits(1000);
        for planet in PLANETS {
            for opp in planet.opponents {
                check(&mut p, planet.id, opp, false); // a loss first...
                check(&mut p, planet.id, opp, true); // ...then the win
            }
        }
        assert!(p.campaign().run_complete());
        assert_eq!(p.stats().campaign_completions(), 1);

        // ...and a rematch afterwards: no signal, no count.
        check(&mut p, "zenith", "sovereign", true);
        assert_eq!(p.stats().campaign_completions(), 1);
    }

    #[test]
    fn new_campaign_resets_the_map_and_keeps_the_pool() {
        // Dirty everything a New Campaign has an opinion about: progress, an
        // in-flight staked match, a bought card, an edited deck, credits,
        // lifetime stats and both onboarding marks.
        let mut p = profile_with_credits(200);
        p.campaign_mut().mark_beaten("cinder", "greeb");
        p.grant_card(Card::PlusMinus(6));
        let first = p.deck()[0];
        assert!(p.remove_from_deck(first));
        p.record_match(Mode::Campaign, "greeb", true, 3, 1);
        p.mark_primer_seen();
        p.mark_first_match_seen();
        assert!(p.stake_match(node("scree", "dax", 20)));
        assert_eq!(p.campaign().stake_at_risk(), Some(20), "sanity: a stake is in escrow");
        let credits = p.credits();
        let collection = p.collection.clone();
        let deck = p.deck().to_vec();

        p.reset_campaign_run();

        // The map, the in-flight match and its escrow, and the run tally go...
        assert!(!p.campaign().has_progress(), "beaten opponents cleared");
        assert!(p.campaign().in_progress().is_none(), "the in-flight match dropped");
        assert_eq!(p.campaign().stake_at_risk(), None, "the escrowed stake goes with it");
        assert_eq!(p.campaign().run_stats().matches_played(), 0, "the run tally zeroed");
        // ...and everything the player earned stays.
        assert_eq!(p.credits(), credits, "credits are kept (the escrow is not refunded)");
        assert_eq!(p.collection, collection, "the collection is kept");
        assert_eq!(p.deck(), deck.as_slice(), "the built deck is kept");
        assert_eq!(p.stats().campaign().get("greeb").match_wins, 1, "lifetime stats are kept");
        assert!(p.primer_seen(), "the primer mark is kept");
        assert!(p.first_match_seen(), "the first-match mark is kept");
    }

    #[test]
    fn the_entry_panel_shows_whenever_the_run_or_the_pool_differs_from_the_starter() {
        assert!(
            !Profile::default().differs_from_starter(),
            "a truly fresh profile opens the map directly",
        );

        // ...and so does one loaded back from disk: a fresh profile writes its
        // seed purse, so the round trip is still the starter. (A pre-economy
        // document with no `credits` key loads with 0 instead — a pool that
        // really does differ from the starter, so spec 021's migration path
        // sees the entry panel, which is what the predicate specifies.)
        let fresh = serde_json::to_string(&Profile::default()).unwrap();
        assert!(
            !Profile::from_json(&fresh)
                .expect("a fresh profile reloads")
                .differs_from_starter(),
            "a reloaded fresh profile is still the starter",
        );
        assert!(
            Profile::from_json(r#"{"version":1}"#)
                .expect("a pre-economy document loads")
                .differs_from_starter(),
            "a document with no credits key loads with 0 credits, so its pool \
             differs from the starter and the entry panel shows",
        );

        let mut beaten = Profile::default();
        beaten.campaign_mut().mark_beaten("cinder", "greeb");
        assert!(beaten.differs_from_starter(), "progress on the map");

        let mut earned = Profile::default();
        earned.earn_credits(1);
        assert!(earned.differs_from_starter(), "a credit earned");

        let mut spent = Profile::default();
        spent.credits -= 1; // a shop buy, without the affordability dance
        assert!(spent.differs_from_starter(), "a credit spent");

        let mut granted = Profile::default();
        granted.grant_card(Card::PlusMinus(6));
        assert!(granted.differs_from_starter(), "a card in the collection");

        let mut edited = Profile::default();
        let first = edited.deck()[0];
        assert!(edited.remove_from_deck(first));
        assert!(edited.differs_from_starter(), "a deck the player edited");

        let mut staked = Profile::default();
        assert!(staked.stake_match(node("cinder", "greeb", 10)));
        assert!(staked.differs_from_starter(), "a staked match in flight");
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
    fn default_profile_plays_a_valid_outer_tier_starter() {
        use crate::card::{ALL_SIDE_CARDS, DEFAULT_SIDE_DECK};
        use crate::economy::{RegionTier, card_tier};

        let p = Profile::default();
        assert_eq!(p.deck().len(), SIDE_DECK_SIZE);
        assert!(p.deck_is_valid());
        // The starter is its own deck now (spec 022), no longer the standard
        // pool — that stays the opponent baseline (since spec 024 every match,
        // Quick Play included, deals the player's built deck).
        assert_eq!(p.deck(), &STARTER_SIDE_DECK);
        assert_ne!(p.deck(), &DEFAULT_SIDE_DECK);

        // The collection is exactly the deck plus the spares.
        let mut expected = STARTER_SIDE_DECK.to_vec();
        expected.extend(STARTER_SPARES);
        assert_eq!(p.collection, expected);

        // The point of the starter: every shop tier above Outer is an upgrade.
        for card in STARTER_SIDE_DECK.iter().chain(STARTER_SPARES.iter()) {
            assert_eq!(card_tier(*card), RegionTier::Outer, "{card:?}");
            // ...and every starter card stays inside the album's universe.
            assert!(ALL_SIDE_CARDS.contains(card), "universe missing {card:?}");
        }
    }

    #[test]
    fn an_existing_profile_keeps_its_premium_deck_collection_and_credits() {
        use crate::card::DEFAULT_SIDE_DECK;

        // A profile written before spec 022: the serde defaults only fill
        // *missing* keys, so a saved deck/collection/credits survive a change
        // to the starter untouched (ruling F) — and the format is unchanged,
        // so the version stays 1.
        let mut collection = DEFAULT_SIDE_DECK.to_vec();
        collection.extend([Card::Plus(1), Card::Minus(1), Card::PlusMinus(2)]);
        let doc = serde_json::json!({
            "version": 1,
            "collection": collection,
            "deck": DEFAULT_SIDE_DECK.to_vec(),
            "credits": 75,
        });
        let p = Profile::from_json(&doc.to_string()).expect("an existing profile loads");
        assert_eq!(p.deck(), &DEFAULT_SIDE_DECK);
        assert_eq!(p.collection, collection);
        assert_eq!(p.credits(), 75);
        assert_eq!(PROFILE_VERSION, 1, "no save-format change in this spec");
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
