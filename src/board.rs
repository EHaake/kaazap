use crate::{
    CARD_HEIGHT, CARD_WIDTH,
    card::CardView,
    config::Config,
    frame::{Align, Drawable, Emphasis, Frame, draw_text, draw_text_in},
    game::{GamePhase, GameState, RoundOutcome},
    layout::{BoardLayout, SideLayout, card_slot, cards_per_row},
    player::{Player, PlayerState},
};

pub struct BoardView {
    pub config: Config,
    layout: BoardLayout,
}

/// Handles the drawing of the board state
///
impl BoardView {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            layout: BoardLayout::new(config),
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

    /// Draw one side's header: name (left), score (right, Strong),
    /// rounds won (right), and a bust/stood note (left).
    fn draw_side_header(&self, side: &SideLayout, name: &str, p: &PlayerState, frame: &mut Frame) {
        draw_text_in(frame, side.header, 0, Align::Left, &format!("{name}: {}", p.name), Emphasis::Normal);
        draw_text_in(frame, side.header, 0, Align::Right, &format!("Score: {}", p.score()), Emphasis::Strong);
        draw_text_in(frame, side.header, 1, Align::Right, &format!("Rounds won: {}", p.rounds_won), Emphasis::Normal);

        if p.bust {
            draw_text_in(frame, side.header, 1, Align::Left, "BUSTED!!", Emphasis::Normal);
        } else if p.stood {
            draw_text_in(frame, side.header, 1, Align::Left, "Stood", Emphasis::Normal);
        }
    }

    /// Draw Top info (Player name, score, etc.)
    ///
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        self.draw_side_header(&self.layout.player, "Player", &state.player, frame);
        self.draw_side_header(&self.layout.opponent, "Opponent", &state.opponent, frame);
    }

    /// Draw one side's zones: Muted labels, dealer grid (wraps), played
    /// row, and hand. `reveal_hand` shows the player's card faces and
    /// number keys; the opponent's hand shows hidden "?" faces.
    fn draw_side(&self, side: &SideLayout, ps: &PlayerState, reveal_hand: bool, frame: &mut Frame) {
        // Zone labels sit one row above each zone (clip-safe if tight)
        draw_text(frame, side.dealer.x0, side.dealer.y0.saturating_sub(1), "Dealer", Emphasis::Muted);
        draw_text(frame, side.played.x0, side.played.y0.saturating_sub(1), "Played", Emphasis::Muted);
        draw_text(frame, side.hand.x0, side.hand.y0.saturating_sub(1), "Hand", Emphasis::Muted);

        // Dealer draws wrap into rows within the zone
        let per = cards_per_row(side.dealer);
        for (i, c) in ps.dealer_row.iter().enumerate() {
            let (x, y) = card_slot(side.dealer, i % per, i / per);
            CardView::new(x, y, c.display_text()).draw(frame);
        }

        // Played cards — one row; flips sit here at value 0 (no filter)
        for (i, c) in ps.played_row.iter().enumerate() {
            let (x, y) = card_slot(side.played, i, 0);
            CardView::new(x, y, c.display_text()).draw(frame);
        }

        // Hand — faces + number keys for the player, hidden for the opponent
        for (i, c) in ps.hand.iter().enumerate() {
            let Some(card) = c else { continue };
            let (x, y) = card_slot(side.hand, i, 0);
            let text = if reveal_hand {
                card.label()
            } else {
                "?".to_string()
            };
            CardView::new(x, y, text).draw(frame);

            if reveal_hand {
                // number key centered just below the card
                draw_text(frame, x + CARD_WIDTH / 2, y + CARD_HEIGHT, &(i + 1).to_string(), Emphasis::Normal);
            }
        }
    }

    /// Draw the current game state
    ///
    pub fn draw(&self, state: &GameState, frame: &mut Frame) {
        // Vertical divider down the middle
        let divider_x = self.layout.divider_x;
        for y in 0..self.config.num_rows {
            if divider_x < frame.len() && y < frame[0].len() {
                frame[divider_x][y].ch = '│';
            }
        }

        self.draw_top_info(state, frame);
        self.draw_side(&self.layout.player, &state.player, true, frame);
        self.draw_side(&self.layout.opponent, &state.opponent, false, frame);

        // Draw Turn Text
        self.draw_turn_text(state, frame);

        // Draw Round/Game Outcome if it exists
        self.draw_round_outcome_text(state, frame);
    }
}
