// Player's cards and interaction

use serde::{Deserialize, Serialize};

use crate::card::{Card, PlayedCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Player,
    Opponent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub dealer_row: Vec<PlayedCard>,      // dealer cards played
    pub played_row: Vec<PlayedCard>,      // Side hand cards played
    pub hand: Vec<Option<Card>>,          // cards in hand
    pub stood: bool,                      // do they get a dealer card next turn?
    pub bust: bool,                       // is score > 20?
    pub rounds_won: usize,                // rounds won
}

impl PlayerState {

    /// Calculate player's score which includes the dealer row and side cards played
    ///
    pub fn score(&self) -> i32 {
        let total_dealer: i32 = self.dealer_row.iter().map(|c| c.value as i32).sum();
        let total_side: i32 = self.played_row.iter().map(|c| c.value as i32).sum();

        total_side + total_dealer
    }

    /// Does this player have a tiebreaker on the table this round?
    /// (played_row is cleared each round, so this is round-scoped.)
    pub fn has_tiebreaker_in_play(&self) -> bool {
        self.played_row
            .iter()
            .any(|pc| pc.card == Card::Tiebreaker)
    }

    /// Cards this side holds on the table this round: dealer draws plus
    /// played side cards. Bounded by MAX_TABLE_CARDS.
    pub fn table_card_count(&self) -> usize {
        self.dealer_row.len() + self.played_row.len()
    }

    /// Has this side filled every table slot? A full side can hold no
    /// more cards and auto-stands (game.rs `resolve_after_action`).
    pub fn table_full(&self) -> bool {
        self.table_card_count() >= crate::MAX_TABLE_CARDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::FlipKind;

    fn empty_player() -> PlayerState {
        PlayerState {
            name: "Test".to_string(),
            dealer_row: vec![],
            played_row: vec![],
            hand: vec![None, None, None, None],
            stood: false,
            bust: false,
            rounds_won: 0,
        }
    }

    #[test]
    fn score_empty_rows_is_zero() {
        assert_eq!(empty_player().score(), 0);
    }

    #[test]
    fn score_sums_both_rows_across_mixed_kinds() {
        let mut p = empty_player();
        p.dealer_row = vec![
            PlayedCard { card: Card::Dealer(7), value: 7 },
            PlayedCard { card: Card::Dealer(0), value: 0 },
        ];
        p.played_row = vec![
            PlayedCard { card: Card::Plus(4), value: 4 },
            PlayedCard { card: Card::Minus(3), value: -3 },
            PlayedCard { card: Card::PlusMinus(2), value: -2 },
            PlayedCard { card: Card::Flip(FlipKind::TwoFour), value: 0 },
            PlayedCard { card: Card::Tiebreaker, value: 1 },
        ];

        assert_eq!(p.score(), 7 + 0 + 4 - 3 - 2 + 0 + 1);
    }

    #[test]
    fn score_dealer_row_alone_counts() {
        let mut p = empty_player();
        p.dealer_row = vec![
            PlayedCard { card: Card::Dealer(10), value: 10 },
            PlayedCard { card: Card::Dealer(5), value: 5 },
        ];

        assert_eq!(p.score(), 15);
    }
}
