use crate::{
    CARD_HEIGHT, CARD_WIDTH,
    app::HandCursor,
    card::{Card, CardView},
    config::Config,
    frame::{Align, BorderWeight, Drawable, Emphasis, Frame, draw_text, draw_text_in},
    game::{GamePhase, GameState, RoundOutcome},
    layout::{BoardLayout, Rect, SideLayout, card_slot, cards_per_row},
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

    /// The two status lines for the current state: an optional over-20
    /// alert (upper row) and the base prompt (lower row).
    fn status_lines(
        &self,
        state: &GameState,
        cursor: &HandCursor,
    ) -> (Option<(String, Emphasis)>, Option<(String, Emphasis)>) {
        let alert = over_twenty_alert(state);
        let selected = state.player.hand.get(cursor.index()).copied().flatten();
        let base = status_message(state, selected, cursor.pending_positive());
        (alert, base)
    }

    /// Draw the status strip in `rect` with `align`: alert on the upper
    /// row (when over 20), base prompt on the lower row.
    fn draw_status(
        &self,
        alert: &Option<(String, Emphasis)>,
        base: &Option<(String, Emphasis)>,
        rect: Rect,
        align: Align,
        frame: &mut Frame,
    ) {
        if let Some((text, emphasis)) = alert {
            draw_text_in(frame, rect, 0, align, text, *emphasis);
        }
        if let Some((text, emphasis)) = base {
            draw_text_in(frame, rect, 1, align, text, *emphasis);
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
        // Decide where the status goes: to the right of the hand if the
        // widest line fits there, otherwise below the board.
        let (alert, base) = self.status_lines(state, cursor);
        let max_len = [&alert, &base]
            .iter()
            .filter_map(|m| m.as_ref().map(|(t, _)| t.chars().count()))
            .max()
            .unwrap_or(0);
        let below = !self.layout.status_fits_right(max_len);

        // Vertical divider. When the status is below, stop it above those
        // two rows so it doesn't run through the status bar.
        let divider_x = self.layout.divider_x;
        let divider_bottom = if below {
            self.config.num_rows.saturating_sub(2)
        } else {
            self.config.num_rows
        };
        for y in 0..divider_bottom {
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

        // Status: right of the hand (right-aligned) or below (left-aligned)
        let (rect, align) = if below {
            (self.layout.status_below, Align::Left)
        } else {
            (self.layout.status_right, Align::Right)
        };
        self.draw_status(&alert, &base, rect, align, frame);

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
        return Some((play_prompt_line(selected, positive), Emphasis::Strong));
    }

    match state.game_phase {
        GamePhase::OpponentThinking { .. } => {
            Some(("Opponent's Turn".to_string(), Emphasis::Muted))
        }
        _ => None,
    }
}

/// The player's turn prompt: navigation is always shown, and the
/// selected card gets a consistent "Play <face>?" with card-appropriate
/// controls (sign-choice cards add the flip hint; others just Enter).
fn play_prompt_line(selected: Option<Card>, positive: bool) -> String {
    let Some(card) = selected else {
        // Empty hand — only hit/stand remain
        return "←/→ card · d: draw · s: stand".to_string();
    };

    match card.sign_choice_magnitude() {
        Some(magnitude) => {
            let value = if positive { magnitude } else { -magnitude };
            format!("←/→ card · Play {value:+}? · ↑/↓ flip · Enter")
        }
        None => format!("←/→ card · Play {}? · Enter", card.label()),
    }
}

/// The over-20 alert, shown on its own status row above the base prompt
/// while the player is over 20 and can still act (so the recovery
/// instructions below it stay visible). Separate from status_message so
/// it never replaces the prompt.
pub fn over_twenty_alert(state: &GameState) -> Option<(String, Emphasis)> {
    if matches!(state.game_phase, GamePhase::PlayerTurn) && state.player.score() > 20 {
        Some(("OVER 20!  (d/s: bust)".to_string(), Emphasis::Alert))
    } else {
        None
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
    fn status_player_turn_shows_nav_and_play_prompt_strong() {
        let gs = GameState::new(); // starts in PlayerTurn
        let (text, emphasis) = msg(&gs).unwrap();
        // Nav is always present; with a fixed card selected there's no
        // flip hint (msg() passes None → empty-hand hint, so check a
        // selected fixed card explicitly below); here just the nav.
        assert!(text.starts_with("←/→ card"));
        assert_eq!(emphasis, Emphasis::Strong);
    }

    #[test]
    fn status_selected_sign_card_shows_pending_sign_and_flip_hint() {
        let gs = GameState::new(); // PlayerTurn
        let card = Some(Card::PlusMinus(3));

        let (pos, _) = status_message(&gs, card, true).unwrap();
        assert_eq!(pos, "←/→ card · Play +3? · ↑/↓ flip · Enter");

        let (neg, _) = status_message(&gs, card, false).unwrap();
        assert_eq!(neg, "←/→ card · Play -3? · ↑/↓ flip · Enter");

        // The tiebreaker (magnitude 1) works the same way
        let (tb, _) = status_message(&gs, Some(Card::Tiebreaker), false).unwrap();
        assert_eq!(tb, "←/→ card · Play -1? · ↑/↓ flip · Enter");
    }

    #[test]
    fn status_selected_fixed_card_shows_play_with_no_flip_hint() {
        let gs = GameState::new();
        let (text, _) = status_message(&gs, Some(Card::Plus(4)), true).unwrap();
        assert_eq!(text, "←/→ card · Play +4? · Enter");
        assert!(!text.contains("flip")); // nothing to flip on a fixed card

        // Flip cards read the same consistent way
        let (flip, _) =
            status_message(&gs, Some(Card::Flip(crate::card::FlipKind::TwoFour)), true).unwrap();
        assert_eq!(flip, "←/→ card · Play 2&4? · Enter");
    }

    fn over_20_game() -> GameState {
        let mut gs = GameState::new();
        // Force a live over-20 total on the player's turn (10+10+5)
        gs.player.dealer_row = vec![
            crate::card::PlayedCard { card: Card::Dealer(10), value: 10 },
            crate::card::PlayedCard { card: Card::Dealer(10), value: 10 },
            crate::card::PlayedCard { card: Card::Dealer(5), value: 5 },
        ];
        gs
    }

    #[test]
    fn status_over_twenty_alert_is_its_own_line() {
        let gs = over_20_game();
        let (text, emphasis) = over_twenty_alert(&gs).unwrap();
        assert!(text.starts_with("OVER 20"));
        assert_eq!(emphasis, Emphasis::Alert);
    }

    #[test]
    fn status_over_twenty_does_not_replace_the_base_prompt() {
        // The whole point of the fix: over 20, the sign/cursor prompt
        // still shows (on the row below the alert)
        let gs = over_20_game();
        let (text, _) = status_message(&gs, Some(Card::PlusMinus(3)), false).unwrap();
        assert_eq!(text, "←/→ card · Play -3? · ↑/↓ flip · Enter");

        // and the base prompt still renders (the alert is a separate line)
        let (hint, _) = status_message(&gs, None, true).unwrap();
        assert!(hint.starts_with("←/→ card"));
    }

    #[test]
    fn status_no_alert_when_not_over_twenty() {
        let gs = GameState::new();
        assert_eq!(over_twenty_alert(&gs), None);
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
