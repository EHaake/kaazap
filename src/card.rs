use crate::{CARD_HEIGHT, CARD_WIDTH, frame::{Drawable, Frame}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipKind {
    TwoFour,
    ThreeSix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// A card on the table: identity plus its current signed contribution.
/// Flips mutate `value` after play; flip cards themselves stay at 0.
#[derive(Debug, Clone, Copy)]
pub struct PlayedCard {
    pub card: Card,
    pub value: i8,
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

pub struct CardView {
    pub x: usize,
    pub y: usize,
    // pub width: usize,
    // pub height: usize,
    pub text: String,
}

impl Drawable for CardView {
    fn draw(&self, frame: &mut Frame) {
        let x0 = self.x;
        let y0 = self.y;

        if x0 >= frame.len() || y0 >= frame[0].len() {
            return;
        }

        if x0 + CARD_WIDTH > frame.len() || y0 + CARD_HEIGHT > frame[0].len() {
            return;
        }

        let x1 = x0 + CARD_WIDTH - 1;
        let y1 = y0 + CARD_HEIGHT - 1;

        // borders
        (x0..=x1).for_each(|x| {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        });

        (y0..=y1).for_each(|y| {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        });

        // corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';

        // interior
        ((x0 + 1)..x1).for_each(|x| {
            ((y0 + 1)..y1).for_each(|y| {
                frame[x][y] = ' ';
            });
        });

        // centered text
        let inner_width = CARD_WIDTH - 2;
        let text_y = y0 + CARD_HEIGHT / 2;

        // clamp to available space
        let text = if self.text.len() > inner_width {
            self.text[..inner_width].to_string()
        } else {
            self.text.clone()
        };

        let start_x = x0 + 1 + (inner_width - text.len()) / 2;

        for (i, ch) in text.chars().enumerate() {
            frame[start_x + i][text_y] = ch;
        }
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
