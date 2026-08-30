use std::time::Duration;

use crossterm::event::KeyCode;

use crate::{
    board::BoardView,
    card::Card,
    config::Config,
    frame::Frame,
    game::{GameAction, GamePhase, GameState},
    menu::{MenuEvent, MenuItem, MenuState},
    overlay::{Overlay, OverlayKind},
    screen::Screen,
};

/// The player's card-selection cursor: which hand slot is selected, and
/// the pending sign for a plus-or-minus / tiebreaker card. Pure logic
/// over the hand — the arrow-key/Enter interaction model deferred from
/// spec 001. Coexists with the direct number-key + h/l play path.
#[derive(Debug)]
pub struct HandCursor {
    index: usize,
    pending_positive: bool,
}

impl Default for HandCursor {
    fn default() -> Self {
        Self {
            index: 0,
            pending_positive: true,
        }
    }
}

impl HandCursor {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn pending_positive(&self) -> bool {
        self.pending_positive
    }

    /// Move to the next occupied slot to the right, wrapping. Resets the
    /// pending sign to positive.
    pub fn move_right(&mut self, hand: &[Option<Card>]) {
        if let Some(next) = next_occupied(hand, self.index, 1) {
            self.index = next;
        }
        self.pending_positive = true;
    }

    /// Move to the next occupied slot to the left, wrapping.
    pub fn move_left(&mut self, hand: &[Option<Card>]) {
        if let Some(next) = next_occupied(hand, self.index, -1) {
            self.index = next;
        }
        self.pending_positive = true;
    }

    /// Flip the pending sign — only meaningful on a sign-choice card.
    pub fn toggle_sign(&mut self, hand: &[Option<Card>]) {
        if is_sign_choice(hand, self.index) {
            self.pending_positive = !self.pending_positive;
        }
    }

    /// After the hand changes (a card was played), snap the cursor back
    /// onto an occupied slot and reset the pending sign.
    pub fn normalize(&mut self, hand: &[Option<Card>]) {
        if !matches!(hand.get(self.index), Some(Some(_)))
            && let Some(i) = first_occupied(hand)
        {
            self.index = i;
        }
        self.pending_positive = true;
    }
}

fn first_occupied(hand: &[Option<Card>]) -> Option<usize> {
    hand.iter().position(|slot| slot.is_some())
}

fn is_sign_choice(hand: &[Option<Card>], index: usize) -> bool {
    matches!(hand.get(index), Some(Some(c)) if c.sign_choice_magnitude().is_some())
}

/// The next occupied slot from `from`, scanning in `dir` (+1/-1) with
/// wraparound, excluding `from` itself. None if no other occupied slot.
fn next_occupied(hand: &[Option<Card>], from: usize, dir: isize) -> Option<usize> {
    let n = hand.len();
    if n == 0 {
        return None;
    }

    let mut i = from as isize;
    for _ in 0..n {
        i = (i + dir).rem_euclid(n as isize);
        let idx = i as usize;
        if idx == from {
            return None; // wrapped all the way around
        }
        if hand[idx].is_some() {
            return Some(idx);
        }
    }
    None
}

/// Commit the cursor-selected card: emit PlayHand, and for a sign-choice
/// card follow immediately with ChooseSign at the pending sign — the
/// engine passes through AwaitingSignChoice and back within one event.
/// Only acts during the player's turn.
fn cursor_confirm(game_state: &mut GameState, cursor: &mut HandCursor) {
    if !matches!(game_state.game_phase, GamePhase::PlayerTurn) {
        return;
    }

    let index = cursor.index();
    let card = match game_state.player.hand.get(index) {
        Some(Some(c)) => *c,
        _ => return,
    };

    game_state.apply_game_action(GameAction::PlayHand { index });
    if card.sign_choice_magnitude().is_some() {
        game_state.apply_game_action(GameAction::ChooseSign {
            positive: cursor.pending_positive(),
        });
    }

    cursor.normalize(&game_state.player.hand);
}

pub struct App {
    pub config: Config,
    screen: Screen,
    board_view: BoardView,
    overlay: Option<Overlay>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            screen: Screen::StartMenu {
                menu_state: MenuState::new(),
            },
            board_view: BoardView::new(config),
            overlay: None,
        }
    }

    /// Route the input key to the appropriate handler
    ///
    pub fn handle_key(&mut self, key: KeyCode) {
        if self.overlay.is_some() {
            // Handle overlay keybinds
            if let KeyCode::Char(c) = key
                && c == '?'
            {
                self.overlay = None
            }
        } else {
            // If overlay is not enabled, if ? is pressed, enable overlay
            if let KeyCode::Char(c) = key
                && c == '?'
            {
                self.overlay = match &mut self.screen {
                    Screen::StartMenu { .. } => {
                        Some(Overlay::new(OverlayKind::MenuHelp, self.config))
                    }
                    Screen::InGame { .. } => {
                        Some(Overlay::new(OverlayKind::GameHelp, self.config))
                    }
                };
            }

            match &mut self.screen {
                // Route the Menu inputs only to Menu
                Screen::StartMenu { menu_state } => {
                    if let Some(menu_action) = menu_state.handle_menu_input(key)
                        && let Some(menu_event) = menu_state.apply_menu_action(menu_action)
                    {
                        self.apply_menu_event(menu_event);
                    }
                }

                // Route the game inputs to game_state. The cursor model
                // (arrows + Enter) and the direct keys (1-4, d/s, h/l)
                // coexist — cursor keys act only on the player's turn.
                Screen::InGame { game_state, cursor } => {
                    let player_turn = matches!(game_state.game_phase, GamePhase::PlayerTurn);
                    match key {
                        KeyCode::Char('x') => {
                            self.screen = Screen::StartMenu {
                                menu_state: MenuState::new(),
                            }
                        }
                        KeyCode::Left if player_turn => cursor.move_left(&game_state.player.hand),
                        KeyCode::Right if player_turn => cursor.move_right(&game_state.player.hand),
                        KeyCode::Up | KeyCode::Down if player_turn => {
                            cursor.toggle_sign(&game_state.player.hand)
                        }
                        KeyCode::Enter if player_turn => cursor_confirm(game_state, cursor),
                        KeyCode::Char(c) => {
                            if let Some(game_action) = game_state.handle_game_input(c) {
                                game_state.apply_game_action(game_action);
                                cursor.normalize(&game_state.player.hand);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// MenuEvent will contain one of the screens to switch to
    ///
    fn apply_menu_event(&mut self, menu_event: MenuEvent) {
        let MenuEvent::Activate { menu_item } = menu_event;

        match menu_item {
            MenuItem::StartGame => {
                self.screen = Screen::InGame {
                    game_state: Box::new(GameState::new()),
                    cursor: HandCursor::default(),
                };
            }
            MenuItem::HowToPlay => {}
        }
    }

    /// Call tick on each sub-screen
    ///
    pub fn tick(&mut self, dt: Duration) {
        match &mut self.screen {
            Screen::StartMenu { menu_state } => menu_state.tick(dt),
            Screen::InGame { game_state, .. } => game_state.update(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match &self.screen {
            Screen::StartMenu {
                menu_state: _menu_state,
            } => self.screen.draw(frame, &self.config),
            Screen::InGame { game_state, .. } => self.board_view.draw(game_state, frame),
        }

        if let Some(overlay) = &self.overlay {
            overlay.draw(frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, FlipKind};
    use crate::game::GamePhase;

    fn hand(cards: &[Option<Card>]) -> Vec<Option<Card>> {
        cards.to_vec()
    }

    #[test]
    fn cursor_move_right_skips_empty_slots_and_wraps() {
        // occupied at 0 and 2; 1 and 3 empty
        let h = hand(&[Some(Card::Plus(2)), None, Some(Card::Minus(4)), None]);
        let mut c = HandCursor::default(); // index 0

        c.move_right(&h);
        assert_eq!(c.index(), 2);
        c.move_right(&h); // wraps past 3 back to 0
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn cursor_move_left_skips_empty_and_wraps() {
        let h = hand(&[Some(Card::Plus(2)), None, Some(Card::Minus(4)), None]);
        let mut c = HandCursor::default();

        c.move_left(&h); // from 0 wraps left to 2
        assert_eq!(c.index(), 2);
        c.move_left(&h);
        assert_eq!(c.index(), 0);
    }

    #[test]
    fn cursor_single_occupied_slot_does_not_move() {
        let h = hand(&[None, Some(Card::Plus(2)), None, None]);
        let mut c = HandCursor::default();
        c.normalize(&h); // snaps to index 1
        assert_eq!(c.index(), 1);

        c.move_right(&h);
        assert_eq!(c.index(), 1);
        c.move_left(&h);
        assert_eq!(c.index(), 1);
    }

    #[test]
    fn cursor_empty_hand_moves_are_noops() {
        let h = hand(&[None, None, None, None]);
        let mut c = HandCursor::default();
        c.move_right(&h);
        c.move_left(&h);
        assert_eq!(c.index(), 0); // unchanged, no panic
    }

    #[test]
    fn cursor_toggle_sign_only_on_sign_choice_cards() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Plus(2)), None, None]);
        let mut c = HandCursor::default(); // index 0, PlusMinus

        assert!(c.pending_positive());
        c.toggle_sign(&h);
        assert!(!c.pending_positive());

        // move to a fixed card — toggling does nothing
        c.move_right(&h); // to index 1 (Plus), resets sign to positive
        assert!(c.pending_positive());
        c.toggle_sign(&h);
        assert!(c.pending_positive());
    }

    #[test]
    fn cursor_moving_resets_pending_sign() {
        let h = hand(&[Some(Card::PlusMinus(3)), Some(Card::Tiebreaker), None, None]);
        let mut c = HandCursor::default();
        c.toggle_sign(&h); // negative
        assert!(!c.pending_positive());
        c.move_right(&h);
        assert!(c.pending_positive()); // reset on move
    }

    #[test]
    fn cursor_normalize_snaps_off_an_emptied_slot() {
        let mut h = hand(&[Some(Card::Plus(2)), Some(Card::Minus(4)), None, None]);
        let mut c = HandCursor::default();
        c.move_right(&h); // index 1
        assert_eq!(c.index(), 1);

        h[1] = None; // that card was played
        c.normalize(&h);
        assert_eq!(c.index(), 0); // snapped back to the remaining card
    }

    // --- confirm emits the right actions through the engine ---

    fn game_with_hand(cards: &[Option<Card>]) -> GameState {
        let mut gs = GameState::new();
        gs.player.hand = cards.to_vec();
        gs
    }

    #[test]
    fn cursor_confirm_plays_a_fixed_card_immediately() {
        let mut gs = game_with_hand(&[Some(Card::Plus(5)), None, None, None]);
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert_eq!(gs.player.played_row[0].value, 5);
        // fixed card commits without lingering in a sign phase
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
    }

    #[test]
    fn cursor_confirm_plays_sign_card_at_the_pending_sign() {
        let mut gs = game_with_hand(&[Some(Card::PlusMinus(3)), None, None, None]);
        let mut c = HandCursor::default();
        c.toggle_sign(&gs.player.hand); // choose negative

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert_eq!(gs.player.played_row[0].value, -3);
        // committed in one event — not left waiting on h/l, and playing a
        // card keeps the player's turn (spec-001 ruling)
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
        assert!(matches!(gs.game_phase, GamePhase::PlayerTurn));
    }

    #[test]
    fn cursor_confirm_tiebreaker_commits_as_pending_sign() {
        let mut gs = game_with_hand(&[Some(Card::Tiebreaker), None, None, None]);
        let mut c = HandCursor::default(); // positive

        cursor_confirm(&mut gs, &mut c);

        assert_eq!(gs.player.played_row[0].card, Card::Tiebreaker);
        assert_eq!(gs.player.played_row[0].value, 1);
    }

    #[test]
    fn cursor_confirm_flip_card_applies_and_does_not_prompt() {
        let mut gs = game_with_hand(&[Some(Card::Flip(FlipKind::TwoFour)), None, None, None]);
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_none());
        assert!(!matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }));
    }

    #[test]
    fn cursor_confirm_does_nothing_off_the_players_turn() {
        let mut gs = game_with_hand(&[Some(Card::Plus(5)), None, None, None]);
        gs.game_phase = GamePhase::OpponentThinking {
            until: std::time::Instant::now(),
        };
        let mut c = HandCursor::default();

        cursor_confirm(&mut gs, &mut c);

        assert!(gs.player.hand[0].is_some()); // untouched
        assert!(gs.player.played_row.is_empty());
    }
}
