use crate::{CARD_HEIGHT, CARD_WIDTH, frame::{Drawable, Frame}};

#[derive(Debug, Copy, Clone)]
pub enum CardKind {
    Dealer,           // only +1..+10
    PlayerPlus,       // Player +1..+6
    PlayerMinus,      // Player -1..-6
    //PlayerPlusMinus,  // Player +-1..+-6 so we can 'flip' its value
}

#[derive(Clone, Copy, Debug)]
pub enum Owner {
    Player,
    Opponent,
    Dealer,
}

#[derive(Clone, Copy, Debug)]
pub struct LogicCard {
    pub value: i32,
    // pub kind: CardKind,
}

pub struct CardView {
    pub x: usize,
    pub y: usize,
    // pub width: usize,
    // pub height: usize,
    pub text: String,
}

impl CardView {
    // fn display_text(&self) -> String {
    //     if !self.face_up {
    //         return "".to_string();
    //     }
    //
    //     self.value.to_string()
    // } 
}

impl Drawable for CardView {
    fn draw(&self, frame: &mut Frame) {
        let x0 = self.x;
        let y0 = self.y;

        if x0 >= frame.len() || y0 >= frame[0].len() {
            return;
        }

        if x0 + CARD_WIDTH > frame.len() || y0 + CARD_HEIGHT > frame[0].len() {
            return;
        }

        let x1 = x0 + CARD_WIDTH - 1;
        let y1 = y0 + CARD_HEIGHT - 1;

        // borders
        (x0..=x1).for_each(|x| {
            frame[x][y0] = '-';
            frame[x][y1] = '-';
        });

        (y0..=y1).for_each(|y| {
            frame[x0][y] = '|';
            frame[x1][y] = '|';
        });

        // corners
        frame[x0][y0] = '+';
        frame[x1][y0] = '+';
        frame[x0][y1] = '+';
        frame[x1][y1] = '+';

        // interior
        ((x0 + 1)..x1).for_each(|x| {
            ((y0 + 1)..y1).for_each(|y| {
                frame[x][y] = ' ';
            });
        });

        // centered text
        let inner_width = CARD_WIDTH - 2;
        let text_y = y0 + CARD_HEIGHT / 2;

        // clamp to available space
        let text = if self.text.len() > inner_width {
            self.text[..inner_width].to_string()
        } else {
            self.text.clone()
        };

        let start_x = x0 + 1 + (inner_width - text.len()) / 2;

        for (i, ch) in text.chars().enumerate() {
            frame[start_x + i][text_y] = ch;
        }
    }
}
