use crate::{CARD_HEIGHT, CARD_WIDTH, card::Card, card::CardKind, config::Config, frame::Drawable};


pub struct Deck {
    deck: Vec<Card>,
}

impl Deck {
    pub fn new(config: &Config) -> Self {
        let mut deck = Vec::new();
        let x = config.num_cols / 2 - (CARD_WIDTH / 2);
        let y = config.num_rows - CARD_HEIGHT;

        deck.push(Card::new(x, y, 1, CardKind::PlayerPlus));

        Self { deck }
    }
}

impl Drawable for Deck {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        for card in &self.deck {
            card.draw(frame);
        } 
    }
}
