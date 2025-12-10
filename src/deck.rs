pub struct Card {
    value: i32,
}

impl Card {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

pub struct Deck {
    deck: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck = Vec::new();

        for value in 0..4 {
            deck.push(Card::new(value));
        }

        Self { deck }
    }
}
