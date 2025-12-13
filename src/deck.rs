use crate::{CARD_HEIGHT, CARD_WIDTH, NUM_COLS, NUM_ROWS, frame::Drawable};

pub struct Card {
    x: usize,
    y: usize,
    value: i32,
}

impl Card {
    pub fn new(x: usize, y: usize, value: i32) -> Self {
        Self { x, y, value }
    }
}

impl Drawable for Card {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        let x0 = self.x;
        let y0 = self.y;

        let x1 = x0 + CARD_WIDTH - 1;
        let y1 = y0 + CARD_HEIGHT - 1;

        // Top & bottom horizontal borders
        for x in x0..=x1 {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        }

        // Left & right vertical borders
        for y in y0..=y1 {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        }

        // Corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';

        // Fill inside with spaces
        for x in (x0 + 1)..x1 {
            for y in (y0 + 1)..y1 {
                frame[x][y] = ' ';
            }
        }

        let value_str = self.value.to_string();
        let inner_width = CARD_WIDTH - 2;
        let start_x = x0 + 1 + (inner_width.saturating_sub(value_str.len())) / 2;
        let text_y = y0 + CARD_HEIGHT / 2;

        for (i, ch) in value_str.chars().enumerate() {
            if i >= inner_width {
                break;
            }

            frame[start_x + i][text_y] = ch;
        }
    }
}

pub struct Deck {
    deck: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut deck = Vec::new();
        let x = NUM_COLS / 2 - (CARD_WIDTH / 2);
        let y = NUM_ROWS - CARD_HEIGHT;
        deck.push(Card::new(x, y, 1));

        Self { deck }
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawable for Deck {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        for card in &self.deck {
            card.draw(frame);
        } 
    }
}
