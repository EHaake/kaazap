use crate::{
    CARD_HEIGHT, CARD_WIDTH,
    card::CardView,
    config::Config,
    frame::{Drawable, Frame},
    game::GameState,
};

pub struct BoardView {
    pub config: Config,
}

impl BoardView {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    //
    // Draw Top info (Player name, score, et)
    //
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        let mid = self.config.num_cols / 2;
        let padding_y: usize = 1;
        let padding_x: usize = 4;

        // --- Player Side ---
        let player_name_display = format!("Player: {}", state.player.name);
        for (i, ch) in player_name_display.chars().enumerate() {
            frame[padding_x + i][padding_y] = ch;
        }

        let player_score_display = format!("Score: {}", state.player.score());
        for (i, ch) in player_score_display.chars().enumerate() {
            frame[mid - 12 + i][padding_y] = ch;
        }
        //
        // If Bust, display so!
        if state.player.bust {
            let bust_display = "BUSTED!!".to_string();
            for (i, ch) in bust_display.chars().enumerate() {
                frame[mid - 12 + i][padding_y + 1] = ch;
            }
        }

        // --- Opponent Side ---
        let opponent_name_display = format!("Opponent: {}", state.opponent.name);
        for (i, ch) in opponent_name_display.chars().enumerate() {
            frame[mid + padding_x + i][padding_y] = ch;
        }

        let opponent_score_display = format!("Score: {}", state.opponent.score());
        for (i, ch) in opponent_score_display.chars().enumerate() {
            frame[self.config.num_cols - 12 + i][padding_y] = ch;
        }
        // If Bust, display so!
        if state.opponent.bust {
            let bust_display = "BUSTED!!".to_string();
            for (i, ch) in bust_display.chars().enumerate() {
                frame[self.config.num_cols - 12 + i][padding_y + 1] = ch;
            }
        }
    }

    pub fn draw(&self, state: &GameState, frame: &mut Frame) {
        //
        // draw a vertical divider down the middle
        let mid = self.config.num_cols / 2;
        for y in 0..self.config.num_rows {
            if mid < frame.len() && y < frame[0].len() {
                frame[mid][y] = '|';
            }
        }

        // layout constants (simple, tweak later)
        let padding_x: usize = 4;
        let dealer_y: usize = 4;
        let played_y = dealer_y + CARD_HEIGHT + 1;
        let hand_y = self.config.num_rows.saturating_sub(CARD_HEIGHT + 1);

        let spacing_x = CARD_WIDTH + 1;

        let player_origin_x = padding_x;
        let opp_origin_x = mid + padding_x;

        // Top Info
        self.draw_top_info(state, frame);

        //
        // --- Player side ---
        //
        // Dealer Cards
        for (i, c) in state.player.dealer_row.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView {
                x,
                y: dealer_y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }
        // Played Cards
        for (i, c) in state.player.played_row.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView {
                x,
                y: played_y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }
        // Hand cards
        for (i, c) in state.player.hand.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView {
                x,
                y: hand_y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }

        //
        // --- Opponent side ---
        //
        // Dealer Cards
        for (i, c) in state.opponent.dealer_row.iter().enumerate() {
            let x = opp_origin_x + i * spacing_x;
            CardView {
                x,
                y: dealer_y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }
        // Played Cards
        for (i, c) in state.opponent.played_row.iter().enumerate() {
            let x = opp_origin_x + i * spacing_x;
            CardView {
                x,
                y: played_y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }
        // Opponent hand cards (hidden values)
        for i in 0..state.opponent.hand.len() {
            let x = opp_origin_x + i * spacing_x;
            CardView {
                x,
                y: hand_y,
                text: "?".to_string(),
            }
            .draw(frame);
        }
    }
}
