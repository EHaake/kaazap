use crate::{CARD_HEIGHT, CARD_WIDTH, frame::Drawable};

#[derive(Debug, Copy, Clone)]
pub enum CardKind {
    Dealer,           // only +1..+10
    PlayerPlus,       // Player +1..+6
    PlayerMinus,      // Player -1..-6
    //PlayerPlusMinus,  // Player +-1..+-6 so we can 'flip' its value
}

pub struct Card {
    x: usize,
    y: usize,
    pub value: i32,
    pub kind: CardKind,
}

impl Card {
    pub fn new(x: usize, y: usize, value: i32, kind: CardKind) -> Self {
        Self { x, y, value, kind }
    }
}

impl Drawable for Card {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        let x0 = self.x;
        let y0 = self.y;

        let x1 = x0 + CARD_WIDTH - 1;
        let y1 = y0 + CARD_HEIGHT - 1;

        // Top & bottom horizontal borders
        let tmp = x0..=x1;
        for x in tmp {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        }

        // Left & right vertical borders
        let tmp1 = y0..=y1;
        for y in tmp1 {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        }

        // Corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';

        // Fill inside with spaces
        let tmp2 = (x0 + 1)..x1;
        for x in tmp2 {
            let tmp3 = (y0 + 1)..y1;
            for y in tmp3 {
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
