// Player's cards and interaction

use crate::deck::Deck;

pub struct Player {
    deck: Deck,
}

impl Player {
    pub fn new() -> Self {
        let mut deck = Deck::new();
    }
}
