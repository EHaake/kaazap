use std::cmp::max;

use crate::{
    CARD_HEIGHT, CARD_WIDTH, H_PAD,
    card::CardView,
    config::Config,
    frame::{Drawable, Emphasis, Frame, draw_text},
    game::{GamePhase, GameState, RoundOutcome},
    player::Player,
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

/// Handles the drawing of the board state
///
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


    /// Draw round/game outcome text in the middle of screen
    ///
    fn draw_round_outcome_text(&self, state: &GameState, frame: &mut Frame) {
        let mid_x = self.config.num_cols / 2;
        let mid_y = self.config.num_rows / 2;

        if let GamePhase::GameOver { winner } = state.game_phase {
            match winner {
                Player::Player => {
                    draw_text(frame, mid_x - 9, mid_y, "YOU WIN THE GAME! :)", Emphasis::Normal);
                }
                Player::Opponent => {
                    draw_text(frame, mid_x - 9, mid_y, "YOU LOST THE GAME! :(", Emphasis::Normal);
                }
            }
            draw_text(frame, mid_x - 11, mid_y + 2, "(g: new game, x: menu)", Emphasis::Normal);

            return;
        }

        // Round outcome only renders during AwaitingNextRound, which is
        // exactly when n is the key that advances
        match state.round_outcome {
            Some(RoundOutcome::PlayerWon) => {
                draw_text(frame, mid_x - 9, mid_y, "You won this round!", Emphasis::Normal);
            }
            Some(RoundOutcome::Tied) => {
                draw_text(frame, mid_x - 4, mid_y, "You Tied!", Emphasis::Normal);
            }
            Some(RoundOutcome::OpponentWon) => {
                draw_text(frame, mid_x - 11, mid_y, "Opponent won the round!", Emphasis::Normal);
            }
            None => return,
        }
        draw_text(frame, mid_x - 7, mid_y + 2, "(n: next round)", Emphasis::Normal);
    }

    /// Draw whose turn it is
    ///
    fn draw_turn_text(&self, state: &GameState, frame: &mut Frame) {
        let mid = self.config.num_cols / 2;
        let padding_y: usize = 5;
        let padding_x: usize = 15;

        match state.game_phase {
            GamePhase::PlayerTurn => {
                // Over 20 the turn continues but drawing won't: say so.
                // Long texts right-align to the divider so they stay on
                // the player's half.
                if state.player.score() > 20 {
                    let text = "OVER 20! Play a card (d/s: bust)";
                    draw_text(
                        frame,
                        mid.saturating_sub(text.chars().count() + 2),
                        self.config.num_rows - padding_y,
                        text,
                        Emphasis::Normal,
                    );
                } else {
                    draw_text(
                        frame,
                        mid - padding_x,
                        self.config.num_rows - padding_y,
                        "Your Turn",
                        Emphasis::Normal,
                    );
                }
            }
            GamePhase::AwaitingSignChoice { hand_index } => {
                if let Some(Some(card)) = state.player.hand.get(hand_index)
                    && let Some(magnitude) = card.sign_choice_magnitude()
                {
                    let prompt = format!("+{magnitude} (h) or -{magnitude} (l)? (c cancels)");
                    draw_text(
                        frame,
                        mid.saturating_sub(prompt.chars().count() + 2),
                        self.config.num_rows - padding_y,
                        &prompt,
                        Emphasis::Normal,
                    );
                }
            }
            GamePhase::OpponentThinking { until: _until } => draw_text(
                frame,
                self.config.num_cols - padding_x - 4,
                self.config.num_rows - padding_y,
                "Opponent's Turn",
                Emphasis::Normal,
            ),
            _ => {}
        }
    }

    /// Draw Top info (Player name, score, et)
    ///
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        let mid = self.config.num_cols / 2;
        let padding_y: usize = 1;
        let padding_x: usize = 4;

        // --- Player Side ---
        let player_name_display = format!("Player: {}", state.player.name);
        draw_text(frame, padding_x, padding_y, &player_name_display, Emphasis::Normal);

        let player_score_display = format!("Score: {}", state.player.score());
        draw_text(frame, mid - 12, padding_y, &player_score_display, Emphasis::Normal);

        let player_round_score_display = format!("Rounds won: {}", state.player.rounds_won);
        draw_text(frame, mid - 17, padding_y + 1, &player_round_score_display, Emphasis::Normal);

        // If Bust or stood, display so!
        if state.player.bust {
            draw_text(frame, padding_x, padding_y + 1, "BUSTED!!", Emphasis::Normal);
        } else if state.player.stood {
            draw_text(frame, padding_x, padding_y + 1, "Stood", Emphasis::Normal);
        }

        // --- Opponent Side ---
        let opponent_name_display = format!("Opponent: {}", state.opponent.name);
        draw_text(frame, mid + padding_x, padding_y, &opponent_name_display, Emphasis::Normal);

        let opponent_score_display = format!("Score: {}", state.opponent.score());
        draw_text(frame, self.config.num_cols - 12, padding_y, &opponent_score_display, Emphasis::Normal);

        let opponent_round_score_display = format!("Rounds won: {}", state.opponent.rounds_won);
        draw_text(frame, self.config.num_cols - 17, padding_y + 1, &opponent_round_score_display, Emphasis::Normal);

        // If Bust or stood, display so!
        if state.opponent.bust {
            draw_text(frame, mid + padding_x, padding_y + 1, "BUSTED!!", Emphasis::Normal);
        } else if state.opponent.stood {
            draw_text(frame, mid + padding_x, padding_y + 1, "Stood", Emphasis::Normal);
        }
    }

    /// Draw the current game state
    ///
    pub fn draw(&self, state: &GameState, frame: &mut Frame) {
        //
        // draw a vertical divider down the middle
        let mid = self.config.num_cols / 2;
        for y in 0..self.config.num_rows {
            if mid < frame.len() && y < frame[0].len() {
                frame[mid][y].ch = '|';
            }
        }

        // Top Info
        self.draw_top_info(state, frame);

        // layout constants (simple, tweak later)
        let dealer_y: usize = 4;
        let hand_y = self.config.num_rows.saturating_sub(CARD_HEIGHT + 2);
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
                text: c.display_text(),
            }
            .draw(frame);
        }
        // Played Cards
        for (i, c) in state.player.played_row.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            CardView {
                x,
                y: played_y,
                text: c.display_text(),
            }
            .draw(frame);
        }
        // Hand cards
        for (i, c) in state.player.hand.iter().enumerate() {
            let x = player_origin_x + i * spacing_x;
            if c.is_some() {
                CardView {
                    x,
                    y: hand_y,
                    text: c.unwrap().label(),
                }
                .draw(frame);

                // Draw card number underneath
                let num_x = player_origin_x + i * spacing_x + (CARD_WIDTH / 2);
                let num_y = hand_y + CARD_HEIGHT;
                frame[num_x][num_y].ch = char::from_digit((i + 1) as u32, 10).unwrap();
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
                text: c.display_text(),
            }
            .draw(frame);
        }
        // Played Cards — flips sit here at value 0, so no zero-filter
        for (i, c) in state.opponent.played_row.iter().enumerate() {
            let x = opp_origin_x + i * spacing_x;
            CardView {
                x,
                y: played_y,
                text: c.display_text(),
            }
            .draw(frame);
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
