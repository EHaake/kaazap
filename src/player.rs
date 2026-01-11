// Player's cards and interaction

use crate::card::LogicCard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Player,
    Opponent,
}

#[derive(Debug)]
pub struct PlayerState {
    pub name: String,
    pub dealer_row: Vec<LogicCard>,       // dealer cards played
    pub played_row: Vec<LogicCard>,       // Side hand cards played
    pub hand: Vec<Option<LogicCard>>,     // cards in hand
    pub stood: bool,                      // do they get a dealer card next turn?
    pub bust: bool,                       // is score > 20?
    pub rounds_won: usize,                // rounds won
    pub played_card: bool,                // did player play a card this turn?
}

impl PlayerState {

    /// Calculate player's score which includes the dealer row and side cards played
    ///
    pub fn score(&self) -> i32 {
        let total_dealer: i32 = self.dealer_row.iter().map(|c| c.value).sum();
        let total_side: i32 = self.played_row.iter().map(|c| c.value).sum();

        total_side + total_dealer
    }
}
