use std::cmp::max;

use crate::{
    CARD_HEIGHT, CARD_WIDTH, H_PAD,
    card::CardView,
    config::Config,
    frame::{Drawable, Frame},
    game::{GamePhase, GameState, RoundOutcome}, player::Player,
};

pub struct PlayArea {
    pub left: usize,
    pub right: usize,
}

pub struct BoardView {
    pub config: Config,
    player_area: PlayArea,
    opponent_area: PlayArea,
    cards_per_row: usize,
}

impl BoardView {
    pub fn new(config: Config) -> Self {
        let player_area = PlayArea {
            left: H_PAD,
            right: config.num_cols / 2 - H_PAD,
        };

        let opponent_area = PlayArea {
            left: config.num_cols / 2 + H_PAD,
            right: config.num_cols - H_PAD,
        };

        let available_width = player_area.right - player_area.left;
        let slot_width = CARD_WIDTH + 1;
        let cards_per_row = max(1, available_width / slot_width);

        Self {
            config,
            player_area,
            opponent_area,
            cards_per_row,
        }
    }

    // Draw Text Helper
    //
    fn draw_text(&self, text: &str, x: usize, y: usize, frame: &mut Frame) {
        for (i, ch) in text.chars().enumerate() {
            frame[x + i][y] = ch;
        }
    }

    // Draw round/game outcome text in the middle of screen
    //
    fn draw_round_outcome_text(&self, state: &GameState, frame: &mut Frame) {
        let mid_x = self.config.num_cols / 2;
        let mid_y = self.config.num_rows / 2;

        if let GamePhase::GameOver { winner } = state.game_phase {
            match winner {
                Player::Player => {
                    self.draw_text("YOU WIN THE GAME! :)", mid_x - 9, mid_y, frame);
                }
                Player::Opponent => {
                    self.draw_text("YOU LOST THE GAME! :(", mid_x - 9, mid_y, frame);
                }
            }

            return;
        }

        match state.round_outcome {
            Some(RoundOutcome::PlayerWon) => {
                self.draw_text("You won this round!", mid_x - 9, mid_y, frame);
            }
            Some(RoundOutcome::Tied) => {
                self.draw_text("You Tied!", mid_x - 4, mid_y, frame);
            }
            Some(RoundOutcome::OpponentWon) => {
                self.draw_text("Opponent won the round!", mid_x - 11, mid_y, frame);
            }
            None => {}
        }
    }

    // Draw whose turn it is
    //
    fn draw_turn_text(&self, state: &GameState, frame: &mut Frame) {
        let mid = self.config.num_cols / 2;
        let padding_y: usize = 4;
        let padding_x: usize = 15;

        match state.game_phase {
            GamePhase::PlayerTurn => self.draw_text(
                "Your Turn",
                mid - padding_x,
                self.config.num_rows - padding_y,
                frame,
            ),
            GamePhase::OpponentThinking { until: _until } => self.draw_text(
                "Opponent's Turn",
                self.config.num_cols - padding_x - 4,
                self.config.num_rows - padding_y,
                frame,
            ),
            _ => {}
        }
    }

    // Draw Top info (Player name, score, et)
    //
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        let mid = self.config.num_cols / 2;
        let padding_y: usize = 1;
        let padding_x: usize = 4;

        // --- Player Side ---
        let player_name_display = format!("Player: {}", state.player.name);
        self.draw_text(player_name_display.as_str(), padding_x, padding_y, frame);

        let player_score_display = format!("Score: {}", state.player.score());
        self.draw_text(player_score_display.as_str(), mid - 12, padding_y, frame);

        let player_round_score_display = format!("Rounds won: {}", state.player.rounds_won);
        self.draw_text(
            player_round_score_display.as_str(),
            mid - 17,
            padding_y + 1,
            frame,
        );

        // If Bust or stood, display so!
        //
        if state.player.bust {
            self.draw_text("BUSTED!!", padding_x, padding_y + 1, frame);
        } else if state.player.stood {
            self.draw_text("Stood", padding_x, padding_y + 1, frame);
        }

        // --- Opponent Side ---
        let opponent_name_display = format!("Opponent: {}", state.opponent.name);
        self.draw_text(
            opponent_name_display.as_str(),
            mid + padding_x,
            padding_y,
            frame,
        );

        let opponent_score_display = format!("Score: {}", state.opponent.score());
        self.draw_text(
            opponent_score_display.as_str(),
            self.config.num_cols - 12,
            padding_y,
            frame,
        );

        let opponent_round_score_display = format!("Rounds won: {}", state.opponent.rounds_won);
        self.draw_text(
            opponent_round_score_display.as_str(),
            self.config.num_cols - 17,
            padding_y + 1,
            frame,
        );
        //
        // If Bust or stood, display so!
        if state.opponent.bust {
            self.draw_text("BUSTED!!", mid + padding_x, padding_y + 1, frame);
        } else if state.opponent.stood {
            self.draw_text("Stood", mid + padding_x, padding_y + 1, frame);
        }
    }

    // --- Drawable trait impl ---
    //
    // Draw the current game state
    pub fn draw(&self, state: &GameState, frame: &mut Frame) {
        //
        // draw a vertical divider down the middle
        let mid = self.config.num_cols / 2;
        for y in 0..self.config.num_rows {
            if mid < frame.len() && y < frame[0].len() {
                frame[mid][y] = '|';
            }
        }

        // Top Info
        self.draw_top_info(state, frame);

        // layout constants (simple, tweak later)
        let dealer_y: usize = 4;
        let hand_y = self.config.num_rows.saturating_sub(CARD_HEIGHT + 1);
        let played_y = hand_y - CARD_HEIGHT - 1;

        let spacing_x = CARD_WIDTH + 1;

        let player_origin_x = self.player_area.left;
        let opp_origin_x = self.opponent_area.left;

        // --- Player side ---
        //
        // Dealer Cards
        for (i, c) in state.player.dealer_row.iter().enumerate() {
            let row = i / self.cards_per_row;
            let col = i % self.cards_per_row;

            let x = player_origin_x + col * spacing_x;
            let y = dealer_y + row * (CARD_HEIGHT + 1);

            CardView {
                x,
                y,
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
            if c.is_some() {
                let x = player_origin_x + i * spacing_x;
                CardView {
                    x,
                    y: hand_y,
                    text: c.unwrap().value.to_string(),
                }
                .draw(frame);
            }
        }

        // --- Opponent side ---
        //
        // Dealer Cards
        for (i, c) in state.opponent.dealer_row.iter().enumerate() {
            let row = i / self.cards_per_row;
            let col = i % self.cards_per_row;

            let x = opp_origin_x + col * spacing_x;
            let y = dealer_y + row * (CARD_HEIGHT + 1);
            CardView {
                x,
                y,
                text: c.value.to_string(),
            }
            .draw(frame);
        }
        // Played Cards
        for (i, c) in state.opponent.played_row.iter().enumerate() {
            if c.value != 0 {
                let x = opp_origin_x + i * spacing_x;
                CardView {
                    x,
                    y: played_y,
                    text: c.value.to_string(),
                }
                .draw(frame);
            }
        }
        // Opponent hand cards (hidden values)
        for (i, c) in state.opponent.hand.iter().enumerate() {
            if c.is_some() {
                let x = opp_origin_x + i * spacing_x;
                CardView {
                    x,
                    y: hand_y,
                    text: "?".to_string(),
                }
                .draw(frame);
            }
        }

        // Draw Turn Text
        self.draw_turn_text(state, frame);

        // Draw Round/Game Outcome if it exists
        self.draw_round_outcome_text(state, frame);
    }
}
