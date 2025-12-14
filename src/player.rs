// Player's cards and interaction

use crate::card::LogicCard;

pub struct PlayerState {
    pub name: String,
    pub dealer_row: Vec<LogicCard>,   // dealer cards played
    pub played_row: Vec<LogicCard>,   // Side hand cards played
    pub hand: Vec<LogicCard>,         // cards in hand
    // pub stood: bool,             // do they get a dealer card next turn?
    // pub bust: bool,              // is score > 20?
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: String::from("Player 1"),
            dealer_row: vec![],
            played_row: vec![],
            hand: vec![],
            // stood: false,
            // bust: false
        }
    }

    pub fn score(&self) -> i32 {
        let total_dealer: i32 = self.dealer_row.iter().map(|c| c.value).sum();
        let total_side: i32 = self.played_row.iter().map(|c| c.value).sum();

        total_side + total_dealer
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
