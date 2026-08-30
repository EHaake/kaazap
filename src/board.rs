use crate::{
    CARD_HEIGHT, CARD_WIDTH,
    app::HandCursor,
    card::{Card, CardView},
    config::Config,
    frame::{Align, BorderWeight, Drawable, Emphasis, Frame, draw_text, draw_text_in},
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
                    draw_text(frame, mid_x - 9, mid_y, "YOU WIN THE GAME! :)", Emphasis::Alert);
                }
                Player::Opponent => {
                    draw_text(frame, mid_x - 9, mid_y, "YOU LOST THE GAME! :(", Emphasis::Alert);
                }
            }
            draw_text(frame, mid_x - 11, mid_y + 2, "(g: new game, x: menu)", Emphasis::Muted);

            return;
        }

        // Round outcome only renders during AwaitingNextRound, which is
        // exactly when n is the key that advances
        match state.round_outcome {
            Some(RoundOutcome::PlayerWon) => {
                draw_text(frame, mid_x - 9, mid_y, "You won this round!", Emphasis::Alert);
            }
            Some(RoundOutcome::Tied) => {
                draw_text(frame, mid_x - 4, mid_y, "You Tied!", Emphasis::Alert);
            }
            Some(RoundOutcome::OpponentWon) => {
                draw_text(frame, mid_x - 11, mid_y, "Opponent won the round!", Emphasis::Alert);
            }
            None => return,
        }
        draw_text(frame, mid_x - 7, mid_y + 2, "(n: next round)", Emphasis::Muted);
    }

    /// Draw the single status message (if any) right-aligned in the
    /// player-half status strip.
    fn draw_status(&self, state: &GameState, cursor: &HandCursor, frame: &mut Frame) {
        let selected = state.player.hand.get(cursor.index()).copied().flatten();
        if let Some((msg, emphasis)) = status_message(state, selected, cursor.pending_positive()) {
            draw_text_in(frame, self.layout.status, 0, Align::Right, &msg, emphasis);
        }
    }

    /// Draw one side's header: name (left), score (right, Strong),
    /// rounds won (right), and a bust/stood note (left).
    fn draw_side_header(&self, side: &SideLayout, name: &str, p: &PlayerState, frame: &mut Frame) {
        draw_text_in(frame, side.header, 0, Align::Left, &format!("{name}: {}", p.name), Emphasis::Normal);
        draw_text_in(frame, side.header, 0, Align::Right, &format!("Score: {}", p.score()), Emphasis::Strong);
        draw_text_in(frame, side.header, 1, Align::Right, &format!("Rounds won: {}", p.rounds_won), Emphasis::Normal);

        if p.bust {
            draw_text_in(frame, side.header, 1, Align::Left, "BUSTED!!", Emphasis::Alert);
        } else if p.stood {
            draw_text_in(frame, side.header, 1, Align::Left, "Stood", Emphasis::Muted);
        }
    }

    /// Draw Top info (Player name, score, etc.)
    ///
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        self.draw_side_header(&self.layout.player, "Player", &state.player, frame);
        self.draw_side_header(&self.layout.opponent, "Opponent", &state.opponent, frame);
    }

    /// Draw one side's zones: Muted labels, dealer grid (wraps), played
    /// row, and hand. The player's side passes `selection` (the cursor's
    /// slot, pending sign, and pulse emphasis) and reveals hand faces +
    /// number keys; the opponent's side passes None and hides its hand.
    fn draw_side(
        &self,
        side: &SideLayout,
        ps: &PlayerState,
        selection: Option<Selection>,
        frame: &mut Frame,
    ) {
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

            let view = match selection {
                // Selected player card: heavy breathing border — the
                // face keeps its ± label; the pending sign shows in the
                // status line, not by mutating the card's own text
                Some(sel) if sel.index == i => {
                    let mut v = CardView::new(x, y, card.label());
                    v.weight = BorderWeight::Heavy;
                    v.emphasis = sel.pulse;
                    v
                }
                // Other player cards recede (Muted) so the selection pops
                Some(_) => {
                    let mut v = CardView::new(x, y, card.label());
                    v.emphasis = Emphasis::Muted;
                    v
                }
                // Opponent card: hidden
                None => CardView::new(x, y, "?".to_string()),
            };
            view.draw(frame);

            if selection.is_some() {
                // number key centered just below the card
                draw_text(frame, x + CARD_WIDTH / 2, y + CARD_HEIGHT, &(i + 1).to_string(), Emphasis::Normal);
            }
        }
    }

    /// Draw the current game state
    ///
    pub fn draw(&self, state: &GameState, cursor: &HandCursor, pulse: Emphasis, frame: &mut Frame) {
        // Vertical divider down the middle
        let divider_x = self.layout.divider_x;
        for y in 0..self.config.num_rows {
            if divider_x < frame.len() && y < frame[0].len() {
                frame[divider_x][y].ch = '│';
            }
        }

        self.draw_top_info(state, frame);

        let selection = Selection {
            index: cursor.index(),
            pulse,
        };
        self.draw_side(&self.layout.player, &state.player, Some(selection), frame);
        self.draw_side(&self.layout.opponent, &state.opponent, None, frame);

        // Draw the status line (turn / prompt / alert / cursor hint)
        self.draw_status(state, cursor, frame);

        // Draw Round/Game Outcome if it exists
        self.draw_round_outcome_text(state, frame);
    }
}

/// The player's hand selection state, passed into rendering.
#[derive(Clone, Copy)]
struct Selection {
    index: usize,
    pulse: Emphasis,
}

/// The single status message for the current game state, with its
/// emphasis. `selected` / `positive` describe the cursor's current hand
/// card so a sign-choice card's pending sign shows here (keeping the
/// card's own face as its ± label). Precedence: over-20 alert > awaiting
/// sign prompt (number-key path) > selected-± pending sign > cursor hint
/// > opponent turn. Pure, so precedence is unit-testable.
pub fn status_message(
    state: &GameState,
    selected: Option<Card>,
    positive: bool,
) -> Option<(String, Emphasis)> {
    // Alert: over 20 during the player's turn (drawing is disabled)
    if matches!(state.game_phase, GamePhase::PlayerTurn) && state.player.score() > 20 {
        return Some(("OVER 20! Play a card (d/s: bust)".to_string(), Emphasis::Alert));
    }

    // Sign prompt while a plus-or-minus / tiebreaker card waits to commit
    // (this is the direct number-key play path, answered with h/l)
    if let GamePhase::AwaitingSignChoice { hand_index } = state.game_phase {
        if let Some(Some(card)) = state.player.hand.get(hand_index)
            && let Some(magnitude) = card.sign_choice_magnitude()
        {
            return Some((
                format!("+{magnitude} (h) or -{magnitude} (l)? (c cancels)"),
                Emphasis::Strong,
            ));
        }
        return None;
    }

    if matches!(state.game_phase, GamePhase::PlayerTurn) {
        // A sign-choice card is selected: show the pending sign here so
        // the ↑/↓ toggle is visible without the card face losing its ±
        if let Some(card) = selected
            && let Some(magnitude) = card.sign_choice_magnitude()
        {
            let value = if positive { magnitude } else { -magnitude };
            return Some((
                format!("Play {value:+}?  (↑/↓ flip · Enter play)"),
                Emphasis::Strong,
            ));
        }

        // Otherwise the generic cursor hint (implies it's your move)
        return Some((
            "←/→ card  ↑/↓ sign  Enter play".to_string(),
            Emphasis::Strong,
        ));
    }

    match state.game_phase {
        GamePhase::OpponentThinking { .. } => {
            Some(("Opponent's Turn".to_string(), Emphasis::Muted))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Card;
    use crate::game::GameState;

    // No card selected (or a non-sign card) — the common case
    fn msg(state: &GameState) -> Option<(String, Emphasis)> {
        status_message(state, None, true)
    }

    #[test]
    fn status_player_turn_shows_the_cursor_hint_strong() {
        let gs = GameState::new(); // starts in PlayerTurn
        let (text, emphasis) = msg(&gs).unwrap();
        assert!(text.contains("Enter play"));
        assert_eq!(emphasis, Emphasis::Strong);
    }

    #[test]
    fn status_selected_sign_card_shows_pending_sign_and_flips() {
        let gs = GameState::new(); // PlayerTurn
        let card = Some(Card::PlusMinus(3));

        let (pos, _) = status_message(&gs, card, true).unwrap();
        assert_eq!(pos, "Play +3?  (↑/↓ flip · Enter play)");

        let (neg, _) = status_message(&gs, card, false).unwrap();
        assert_eq!(neg, "Play -3?  (↑/↓ flip · Enter play)");

        // The tiebreaker (magnitude 1) works the same way
        let (tb, _) = status_message(&gs, Some(Card::Tiebreaker), false).unwrap();
        assert_eq!(tb, "Play -1?  (↑/↓ flip · Enter play)");
    }

    #[test]
    fn status_selected_fixed_card_still_shows_the_generic_hint() {
        let gs = GameState::new();
        let (text, _) = status_message(&gs, Some(Card::Plus(4)), true).unwrap();
        assert!(text.contains("Enter play"));
        assert!(!text.contains("Play +"));
    }

    #[test]
    fn status_over_twenty_alert_beats_turn_text() {
        let mut gs = GameState::new();
        // Force a live over-20 total on the player's turn
        gs.player.dealer_row = vec![
            crate::card::PlayedCard { card: Card::Dealer(10), value: 10 },
            crate::card::PlayedCard { card: Card::Dealer(10), value: 10 },
            crate::card::PlayedCard { card: Card::Dealer(5), value: 5 },
        ];
        let (text, emphasis) = msg(&gs).unwrap();
        assert!(text.starts_with("OVER 20"));
        assert_eq!(emphasis, Emphasis::Alert);
    }

    #[test]
    fn status_sign_prompt_shows_the_cards_magnitude() {
        let mut gs = GameState::new();
        gs.player.hand[0] = Some(Card::PlusMinus(3));
        gs.game_phase = GamePhase::AwaitingSignChoice { hand_index: 0 };
        let (text, emphasis) = msg(&gs).unwrap();
        assert_eq!(text, "+3 (h) or -3 (l)? (c cancels)");
        assert_eq!(emphasis, Emphasis::Strong);
    }

    #[test]
    fn status_opponent_turn_is_muted() {
        let mut gs = GameState::new();
        gs.game_phase = GamePhase::OpponentThinking {
            until: std::time::Instant::now(),
        };
        assert_eq!(
            msg(&gs),
            Some(("Opponent's Turn".to_string(), Emphasis::Muted))
        );
    }

    #[test]
    fn status_is_empty_at_round_end() {
        let mut gs = GameState::new();
        gs.game_phase = GamePhase::RoundEnd;
        assert_eq!(msg(&gs), None);
    }
}
