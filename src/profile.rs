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
    card::{ALL_SIDE_CARDS, Card, DEFAULT_SIDE_DECK},
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
        Profile { version: PROFILE_VERSION, collection, deck }
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
}
