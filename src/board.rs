use crate::{CARD_HEIGHT, CARD_WIDTH, card::CardView, config::Config, frame::{Drawable, Frame}, game::GameState};

pub struct BoardView {
    pub config: Config,
}

impl BoardView {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn draw(&self, state: &GameState, frame: &mut Frame) {
        // vertical divider down the middle
        let mid = self.config.num_cols / 2;
        for y in 0..self.config.num_rows {
            if mid < frame.len() && y < frame[0].len() {
                frame[mid][y] = '|';
            }
        }

        // layout constants (simple, tweak later)
        let padding_x = 2usize;
        let dealer_y = 2usize;
        let played_y = dealer_y + CARD_HEIGHT + 1;
        let hand_y = self.config.num_rows.saturating_sub(CARD_HEIGHT + 1);

        let spacing_x = CARD_WIDTH + 1;

        let player_origin_x = padding_x;
        let opp_origin_x = mid + padding_x;

        // --- Player side ---
        for (i, c) in state.player.dealer_row.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView { x, y: dealer_y, text: c.value.to_string() }.draw(frame);
        }

        for (i, c) in state.player.played_row.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView { x, y: played_y, text: c.value.to_string() }.draw(frame);
        }

        for (i, c) in state.player.hand.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView { x, y: hand_y, text: c.value.to_string() }.draw(frame);
        }

        // --- Opponent side ---
        for (i, c) in state.opponent.dealer_row.iter().enumerate() {
            let x = opp_origin_x + i * spacing_x;
            CardView { x, y: dealer_y, text: c.value.to_string() }.draw(frame);
        }

        for (i, c) in state.opponent.played_row.iter().enumerate() {
            let x = opp_origin_x + i * spacing_x;
            CardView { x, y: played_y, text: c.value.to_string() }.draw(frame);
        }

        // Opponent hand (hidden values)
        for i in 0..state.opponent.hand.len() {
            let x = opp_origin_x + i * spacing_x;
            CardView { x, y: hand_y, text: "??".to_string() }.draw(frame);
        }
    }
}
