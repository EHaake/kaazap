use rand::{Rng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};

use crate::{
    CARD_HEIGHT, CARD_WIDTH, HAND_SIZE,
    frame::{Align, BorderWeight, Cell, Drawable, Emphasis, Frame, draw_box, draw_text_in},
    layout::Rect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlipKind {
    TwoFour,
    ThreeSix,
}

impl FlipKind {
    /// Whether a card at this table value gets its sign inverted by
    /// this flip kind — sign ignored, zeros never match
    pub fn flips_value(&self, value: i8) -> bool {
        match self {
            FlipKind::TwoFour => matches!(value.abs(), 2 | 4),
            FlipKind::ThreeSix => matches!(value.abs(), 3 | 6),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Dealer(u8),     // main-deck draw, 0-10 (intentional variant)
    Plus(u8),       // +N, 1-6
    Minus(u8),      // -N, 1-6
    PlusMinus(u8),  // ±N, sign chosen at play time
    Flip(FlipKind), // 2&4 or 3&6 - board effect, no value of its own
    Tiebreaker,     // ±1, wins otherwise-tied rounds
}

impl Card {
    /// One source of truth for card text, used by rendering.
    ///
    pub fn label(&self) -> String {
        match self {
            Card::Dealer(n) => n.to_string(),
            Card::Plus(n) => format!("+{n}"),
            Card::Minus(n) => format!("-{n}"),
            Card::PlusMinus(n) => format!("±{n}"),
            Card::Flip(FlipKind::TwoFour) => "2&4".to_string(),
            Card::Flip(FlipKind::ThreeSix) => "3&6".to_string(),
            Card::Tiebreaker => "±1T".to_string(),
        }
    }

    /// Every signed value this card can be committed at — empty for
    /// kinds that don't play a chosen value (flips, dealer cards)
    pub fn playable_values(&self) -> Vec<i8> {
        match self {
            Card::Plus(n) => vec![*n as i8],
            Card::Minus(n) => vec![-(*n as i8)],
            Card::PlusMinus(n) => vec![*n as i8, -(*n as i8)],
            Card::Tiebreaker => vec![1, -1],
            Card::Flip(_) | Card::Dealer(_) => vec![],
        }
    }

    /// Can this card be played at exactly this signed value?
    pub fn can_play_as(&self, value: i8) -> bool {
        self.playable_values().contains(&value)
    }

    /// The magnitude a sign-choice card plays at (± cards and the
    /// tiebreaker), or None for kinds needing no play-time choice.
    /// One source of truth for both the commit logic and the prompt.
    pub fn sign_choice_magnitude(&self) -> Option<i8> {
        match self {
            Card::PlusMinus(n) => Some(*n as i8),
            Card::Tiebreaker => Some(1),
            _ => None,
        }
    }
}

/// A card on the table: identity plus its current signed contribution.
/// Flips mutate `value` after play; flip cards themselves stay at 0.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayedCard {
    pub card: Card,
    pub value: i8,
}

impl PlayedCard {
    /// Table text: flips show their identity, dealer cards their bare
    /// value, and side cards their current explicitly-signed value —
    /// so a ± played as -3 reads "-3", and a flipped +4 reads "-4"
    pub fn display_text(&self) -> String {
        match self.card {
            Card::Flip(_) => self.card.label(),
            Card::Dealer(_) => self.value.to_string(),
            // The tiebreaker keeps its T marker once played, so a tie it
            // decides is legible on the table — otherwise it's an
            // indistinguishable +1/-1 and losing the tie looks arbitrary
            Card::Tiebreaker => format!("{:+}T", self.value),
            _ => format!("{:+}", self.value),
        }
    }
}

/// The fixed pool side-deck hands are drawn from. One tunable constant,
/// expected to be rebalanced by the campaign spec.
pub const DEFAULT_SIDE_DECK: [Card; 10] = [
    Card::Plus(2),
    Card::Plus(4),
    Card::Minus(2),
    Card::Minus(4),
    Card::PlusMinus(1),
    Card::PlusMinus(3),
    Card::PlusMinus(6),
    Card::Flip(FlipKind::TwoFour),
    Card::Flip(FlipKind::ThreeSix),
    Card::Tiebreaker,
];

/// Draw a fresh hand: HAND_SIZE distinct cards from `deck`. Each side deals
/// its own hand, independently, once per game, from its own side deck (the
/// player uses [`DEFAULT_SIDE_DECK`]; an opponent uses its profile's deck).
pub fn deal_hand<R: Rng + ?Sized>(rng: &mut R, deck: &[Card]) -> Vec<Option<Card>> {
    deck.choose_multiple(rng, HAND_SIZE)
        .copied()
        .map(Some)
        .collect()
}

pub struct CardView {
    pub x: usize,
    pub y: usize,
    pub text: String,
    pub weight: BorderWeight, // Heavy marks cursor selection (T007)
    pub emphasis: Emphasis,   // applied to border and face text
}

impl CardView {
    /// A card at (x, y) with default chrome: single border, no emphasis.
    pub fn new(x: usize, y: usize, text: String) -> Self {
        Self {
            x,
            y,
            text,
            weight: BorderWeight::Single,
            emphasis: Emphasis::Normal,
        }
    }
}

impl Drawable for CardView {
    fn draw(&self, frame: &mut Frame) {
        if frame.is_empty() {
            return;
        }
        let (w, h) = (frame.len(), frame[0].len());

        // All-or-nothing: only draw when the whole card fits (unchanged)
        if self.x + CARD_WIDTH > w || self.y + CARD_HEIGHT > h {
            return;
        }

        let rect = Rect::new(self.x, self.x + CARD_WIDTH - 1, self.y, self.y + CARD_HEIGHT - 1);

        // Blank the interior (defensive: card may overdraw other content)
        for cx in (rect.x0 + 1)..rect.x1 {
            for cy in (rect.y0 + 1)..rect.y1 {
                frame[cx][cy] = Cell::default();
            }
        }

        draw_box(frame, rect, self.weight, self.emphasis);

        // Centered face text on the card's middle interior row
        let interior = Rect::new(rect.x0 + 1, rect.x1 - 1, rect.y0 + 1, rect.y1 - 1);
        draw_text_in(
            frame,
            interior,
            CARD_HEIGHT / 2 - 1,
            Align::Center,
            &self.text,
            self.emphasis,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_dealer_shows_bare_value() {
        assert_eq!(Card::Dealer(0).label(), "0");
        assert_eq!(Card::Dealer(7).label(), "7");
        assert_eq!(Card::Dealer(10).label(), "10");
    }

    #[test]
    fn label_plus_has_leading_plus() {
        assert_eq!(Card::Plus(4).label(), "+4");
    }

    #[test]
    fn label_minus_has_leading_minus() {
        assert_eq!(Card::Minus(3).label(), "-3");
    }

    #[test]
    fn label_plus_minus_has_plus_minus_sign() {
        assert_eq!(Card::PlusMinus(2).label(), "±2");
    }

    #[test]
    fn label_flip_cards_show_their_pair() {
        assert_eq!(Card::Flip(FlipKind::TwoFour).label(), "2&4");
        assert_eq!(Card::Flip(FlipKind::ThreeSix).label(), "3&6");
    }

    #[test]
    fn label_tiebreaker_is_distinct_from_plus_minus_one() {
        assert_eq!(Card::Tiebreaker.label(), "±1T");
        assert_ne!(Card::Tiebreaker.label(), Card::PlusMinus(1).label());
    }

    #[test]
    fn flip_value_matching_ignores_sign_and_skips_zero() {
        assert!(FlipKind::TwoFour.flips_value(2));
        assert!(FlipKind::TwoFour.flips_value(-4));
        assert!(!FlipKind::TwoFour.flips_value(3));
        assert!(!FlipKind::TwoFour.flips_value(0));
        assert!(FlipKind::ThreeSix.flips_value(-3));
        assert!(FlipKind::ThreeSix.flips_value(6));
        assert!(!FlipKind::ThreeSix.flips_value(2));
        assert!(!FlipKind::ThreeSix.flips_value(0));
    }

    #[test]
    fn display_text_shows_signed_side_cards_bare_dealer_and_flip_labels() {
        // Side cards: explicit sign, current value (post-flip aware)
        let plus = PlayedCard { card: Card::Plus(4), value: 4 };
        assert_eq!(plus.display_text(), "+4");
        let flipped_plus = PlayedCard { card: Card::Plus(4), value: -4 };
        assert_eq!(flipped_plus.display_text(), "-4");
        let pm = PlayedCard { card: Card::PlusMinus(3), value: -3 };
        assert_eq!(pm.display_text(), "-3");
        // A played tiebreaker keeps its T marker (both signs) so a tie it
        // decides is legible — not an indistinguishable +1/-1
        let tb = PlayedCard { card: Card::Tiebreaker, value: 1 };
        assert_eq!(tb.display_text(), "+1T");
        let tb_neg = PlayedCard { card: Card::Tiebreaker, value: -1 };
        assert_eq!(tb_neg.display_text(), "-1T");

        // Dealer cards: bare value, negative when flipped
        let dealer = PlayedCard { card: Card::Dealer(7), value: 7 };
        assert_eq!(dealer.display_text(), "7");
        let flipped_dealer = PlayedCard { card: Card::Dealer(4), value: -4 };
        assert_eq!(flipped_dealer.display_text(), "-4");

        // Flips: identity, never their (zero) value
        let flip = PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 };
        assert_eq!(flip.display_text(), "2&4");
    }

    #[test]
    fn can_play_as_matches_fixed_cards_at_their_own_value_only() {
        assert!(Card::Plus(4).can_play_as(4));
        assert!(!Card::Plus(4).can_play_as(-4));
        assert!(!Card::Plus(4).can_play_as(3));

        assert!(Card::Minus(3).can_play_as(-3));
        assert!(!Card::Minus(3).can_play_as(3));
    }

    #[test]
    fn can_play_as_accepts_either_sign_for_plus_minus_and_tiebreaker() {
        assert!(Card::PlusMinus(6).can_play_as(6));
        assert!(Card::PlusMinus(6).can_play_as(-6));
        assert!(!Card::PlusMinus(6).can_play_as(5));
        assert!(!Card::PlusMinus(6).can_play_as(0));

        assert!(Card::Tiebreaker.can_play_as(1));
        assert!(Card::Tiebreaker.can_play_as(-1));
        assert!(!Card::Tiebreaker.can_play_as(2));
        assert!(!Card::Tiebreaker.can_play_as(0));
    }

    #[test]
    fn can_play_as_never_matches_flips_or_dealer_cards() {
        for value in -10..=10 {
            assert!(!Card::Flip(FlipKind::TwoFour).can_play_as(value));
            assert!(!Card::Flip(FlipKind::ThreeSix).can_play_as(value));
            assert!(!Card::Dealer(5).can_play_as(value));
        }
    }

    #[test]
    fn sign_choice_magnitude_covers_plus_minus_and_tiebreaker_only() {
        assert_eq!(Card::PlusMinus(3).sign_choice_magnitude(), Some(3));
        assert_eq!(Card::Tiebreaker.sign_choice_magnitude(), Some(1));
        assert_eq!(Card::Plus(4).sign_choice_magnitude(), None);
        assert_eq!(Card::Minus(2).sign_choice_magnitude(), None);
        assert_eq!(Card::Flip(FlipKind::TwoFour).sign_choice_magnitude(), None);
        assert_eq!(Card::Dealer(5).sign_choice_magnitude(), None);
    }

    #[test]
    fn deal_hand_is_four_distinct_cards_from_the_deck() {
        for _ in 0..200 {
            let hand = deal_hand(&mut rand::rng(), &DEFAULT_SIDE_DECK);
            assert_eq!(hand.len(), HAND_SIZE);

            let cards: Vec<Card> = hand
                .iter()
                .map(|slot| slot.expect("every dealt slot must be filled"))
                .collect();

            for card in &cards {
                assert!(
                    DEFAULT_SIDE_DECK.contains(card),
                    "dealt a card that isn't in the deck: {card:?}"
                );
            }

            for i in 0..cards.len() {
                for j in (i + 1)..cards.len() {
                    assert_ne!(cards[i], cards[j], "hand contains a duplicate card");
                }
            }
        }
    }

    #[test]
    fn deal_hand_draws_only_from_the_given_deck() {
        // A deck disjoint from DEFAULT_SIDE_DECK: every dealt card must come
        // from it, proving deal_hand honors its deck argument (an opponent's
        // deck, not the default pool).
        let deck = [Card::Plus(7), Card::Minus(7), Card::PlusMinus(5), Card::Plus(9)];
        for _ in 0..200 {
            for slot in deal_hand(&mut rand::rng(), &deck) {
                let card = slot.expect("every dealt slot must be filled");
                assert!(deck.contains(&card), "dealt {card:?} not in the given deck");
            }
        }
    }

    #[test]
    fn deal_can_reach_every_card_in_the_deck() {
        // Catches sampling that silently favors part of the deck — every
        // card must be reachable, including the flips and the tiebreaker
        let mut unseen = DEFAULT_SIDE_DECK.to_vec();

        for _ in 0..500 {
            for slot in deal_hand(&mut rand::rng(), &DEFAULT_SIDE_DECK) {
                if let Some(card) = slot {
                    unseen.retain(|c| *c != card);
                }
            }
            if unseen.is_empty() {
                break;
            }
        }

        assert!(unseen.is_empty(), "never dealt: {unseen:?}");
    }

    #[test]
    fn deck_matches_spec_composition() {
        // Spec's resolved 10-card multiset:
        // +2, +4, -2, -4, ±1, ±3, ±6, 2&4, 3&6, ±1T
        let mut deck = DEFAULT_SIDE_DECK.to_vec();
        let mut expected = vec![
            Card::Plus(2),
            Card::Plus(4),
            Card::Minus(2),
            Card::Minus(4),
            Card::PlusMinus(1),
            Card::PlusMinus(3),
            Card::PlusMinus(6),
            Card::Flip(FlipKind::TwoFour),
            Card::Flip(FlipKind::ThreeSix),
            Card::Tiebreaker,
        ];
        deck.sort_by_key(|c| c.label());
        expected.sort_by_key(|c| c.label());
        assert_eq!(deck, expected);
    }
}
