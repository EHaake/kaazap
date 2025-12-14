// Player's cards and interaction

use crate::card::Card;

pub struct PlayerState {
    pub name: String,
    pub dealer_slots: Vec<Card>, // dealer cards played
    pub side_played: Vec<Card>,  // Side hand cards played
    pub stood: bool,             // do they get a dealer card next turn?
    pub bust: bool,              // is score > 20?
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            name: String::from("Player 1"),
            dealer_slots: vec![],
            side_played: vec![],
            stood: false,
            bust: false
        }
    }

    pub fn score(&self) -> i32 {
        let total_dealer: i32 = self.dealer_slots.iter().map(|c| c.value).sum();
        let total_side: i32 = self.side_played.iter().map(|c| c.value).sum();

        total_side + total_dealer
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}
