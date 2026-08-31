use crate::{
    CARD_HEIGHT, CARD_WIDTH, MAX_TABLE_CARDS,
    app::HandCursor,
    card::{Card, CardView},
    config::Config,
    frame::{Align, BorderWeight, Drawable, Emphasis, Frame, clear_rect, draw_box, draw_ghost_slot, draw_text, draw_text_in},
    game::{GamePhase, GameState, RoundOutcome},
    layout::{BoardLayout, GRID_COLS, Rect, SideLayout, card_slot},
    player::{Player, PlayerState},
};

// Interior padding around a popup's text, inside its border.
const POPUP_PAD_X: usize = 3;
const POPUP_PAD_Y: usize = 1;

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


    /// Draw the round/game outcome as a bordered popup centered on the
    /// board — nothing to draw mid-play.
    fn draw_round_outcome_text(&self, state: &GameState, frame: &mut Frame) {
        let (title, hint) = match state.game_phase {
            GamePhase::GameOver { winner } => match winner {
                Player::Player => ("YOU WIN THE GAME! :)", "g: new game · x: menu"),
                Player::Opponent => ("YOU LOST THE GAME! :(", "g: new game · x: menu"),
            },
            // Round outcome shows during AwaitingNextRound, when n advances
            _ => match state.round_outcome {
                Some(RoundOutcome::PlayerWon) => ("You won this round!", "n: next round"),
                Some(RoundOutcome::Tied) => ("You tied!", "n: next round"),
                Some(RoundOutcome::OpponentWon) => ("Opponent won the round!", "n: next round"),
                None => return,
            },
        };

        self.draw_popup(
            &[(title, Emphasis::Alert), ("", Emphasis::Normal), (hint, Emphasis::Muted)],
            frame,
        );
    }

    /// The outer Rect for a popup of `content_w` × `n_lines`, centered on
    /// the board and clamped to the frame so it never inverts or runs off-
    /// screen. (A popup is only drawn at or above the minimum size, but the
    /// clamp is cheap defense-in-depth, matching OverlayLayout.)
    fn popup_rect(&self, content_w: usize, n_lines: usize) -> Rect {
        let (cols, rows) = (self.config.num_cols, self.config.num_rows);
        let box_w = content_w + 2 * POPUP_PAD_X + 2; // + left/right borders
        let box_h = n_lines + 2 * POPUP_PAD_Y + 2; // + top/bottom borders

        // Center on the board: its divider, and the middle of header..hand.
        let cx = self.layout.divider_x;
        let cy = (self.layout.player.header.y0 + self.layout.player.hand.y1) / 2;
        let x0 = cx.saturating_sub(box_w / 2);
        let y0 = cy.saturating_sub(box_h / 2);
        let x1 = (x0 + box_w.saturating_sub(1)).min(cols.saturating_sub(1));
        let y1 = (y0 + box_h.saturating_sub(1)).min(rows.saturating_sub(1));
        // Clamp the origin so a box larger than the frame never inverts.
        Rect::new(x0.min(x1), x1, y0.min(y1), y1)
    }

    /// Draw a small bordered popup centered on the board: clear the ground
    /// behind it, frame it, and center each line with its own emphasis, so
    /// a message sits above the board instead of overlaying the cards.
    fn draw_popup(&self, lines: &[(&str, Emphasis)], frame: &mut Frame) {
        let content_w = lines.iter().map(|(t, _)| t.chars().count()).max().unwrap_or(0);
        let outer = self.popup_rect(content_w, lines.len());

        clear_rect(frame, outer);
        draw_box(frame, outer, BorderWeight::Single, Emphasis::Normal);

        // Lines start below the top border + vertical pad, centered across
        // the full interior so the horizontal padding stays symmetric.
        let inner = Rect::new(outer.x0 + 1, outer.x1.saturating_sub(1), outer.y0 + 1 + POPUP_PAD_Y, outer.y1);
        for (i, (text, emphasis)) in lines.iter().enumerate() {
            draw_text_in(frame, inner, i, Align::Center, text, *emphasis);
        }
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
            // A full table stood this side involuntarily — say so, since
            // the turn passes too fast for a status-line message to land.
            let note = if p.table_full() { "Stood — table full" } else { "Stood" };
            draw_text_in(frame, side.header, 1, Align::Left, note, Emphasis::Muted);
        }
    }

    /// Draw Top info (Player name, score, etc.)
    ///
    fn draw_top_info(&self, state: &GameState, frame: &mut Frame) {
        self.draw_side_header(&self.layout.player, "Player", &state.player, frame);
        self.draw_side_header(&self.layout.opponent, "Opponent", &state.opponent, frame);
    }

    /// Draw one side's grid and hand. Dealer draws fill the grid from the
    /// front (Single border, bare value), played cards from the back
    /// (Double border, signed value), and dim ghost outlines mark the
    /// unfilled slots between — filling from opposite ends so a new dealer
    /// draw never shifts an already-played card. The player's side passes
    /// `selection` and reveals its hand + number keys; the opponent's side
    /// passes None and hides its hand.
    fn draw_side(
        &self,
        side: &SideLayout,
        ps: &PlayerState,
        selection: Option<Selection>,
        frame: &mut Frame,
    ) {
        // Fixed grid: GRID_COLS per row, the same count the layout reserves.
        let per = GRID_COLS;
        let dealers = ps.dealer_row.len();
        let played = ps.played_row.len();

        // Slot counter above the grid — doubles as its label and shows how
        // close this side is to the cap.
        draw_text(
            frame,
            side.grid.x0,
            side.grid.y0.saturating_sub(1),
            &format!("{}/{}", dealers + played, MAX_TABLE_CARDS),
            Emphasis::Muted,
        );

        // Dealer draws: grid index i, Single border, bare value.
        for (i, c) in ps.dealer_row.iter().enumerate() {
            let (x, y) = card_slot(side.grid, i % per, i / per);
            CardView::new(x, y, c.display_text()).draw(frame);
        }

        // Played cards: filled from the back, Double border, signed value.
        for (j, c) in ps.played_row.iter().enumerate() {
            let idx = MAX_TABLE_CARDS - 1 - j;
            let (x, y) = card_slot(side.grid, idx % per, idx / per);
            let mut v = CardView::new(x, y, c.display_text());
            v.weight = BorderWeight::Double;
            v.draw(frame);
        }

        // Ghost placeholders for the unfilled middle slots.
        for idx in dealers..(MAX_TABLE_CARDS - played) {
            let (x, y) = card_slot(side.grid, idx % per, idx / per);
            draw_ghost_slot(frame, Rect::new(x, x + CARD_WIDTH - 1, y, y + CARD_HEIGHT - 1));
        }

        // Hand — faces + number keys for the player, hidden for the opponent
        draw_text(frame, side.hand.x0, side.hand.y0.saturating_sub(1), "Hand", Emphasis::Muted);
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
        let (alert, base) = self.status_lines(state, cursor);

        // Vertical divider spans the block: from the header down through
        // the hand, stopping above the status band below it.
        let divider_x = self.layout.divider_x;
        let divider_top = self.layout.player.header.y0;
        let divider_bottom = self.layout.player.hand.y1;
        for y in divider_top..=divider_bottom {
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

        // Status: the two-row band below the hand, left-aligned.
        self.draw_status(&alert, &base, self.layout.status, Align::Left, frame);

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

    #[test]
    fn popup_rect_is_sized_centered_and_in_bounds() {
        let config = Config { num_cols: 89, num_rows: 31 };
        let bv = BoardView::new(config);
        let r = bv.popup_rect(23, 3); // widest outcome string, 3 lines
        // 23 content + 2*3 pad + 2 border = 31 wide; 3 + 2*1 + 2 = 7 tall
        assert_eq!((r.x1 - r.x0 + 1, r.y1 - r.y0 + 1), (31, 7));
        assert!(r.x1 < 89 && r.y1 < 31, "in bounds");
        assert!(
            ((r.x0 + r.x1) / 2).abs_diff(bv.layout.divider_x) <= 1,
            "centered on the divider"
        );
    }

    #[test]
    fn popup_rect_clamps_to_a_tiny_frame_without_inverting() {
        // A frame smaller than the box (below the enforced minimum): clamp
        // to the frame, never invert or overrun.
        let bv = BoardView::new(Config { num_cols: 20, num_rows: 8 });
        let r = bv.popup_rect(23, 3); // wants 31x7 in a 20x8 frame
        assert!(r.x0 <= r.x1 && r.y0 <= r.y1, "never inverted");
        assert!(r.x1 < 20 && r.y1 < 8, "clamped in bounds");
    }
}
