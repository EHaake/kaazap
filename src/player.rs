// Player's cards and interaction

use crate::{config::Config, deck::Deck};

pub struct Player {
    deck: Deck,
}

impl Player {
    pub fn new(config: &Config) -> Self {
        let deck = Deck::new(config);

        Self { deck }
    }
}
