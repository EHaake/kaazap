use crate::{
    OPPONENT_THINKING_TIME_MS, STAND_THRESHOLD, card::{Card, PlayedCard}, player::{Player, PlayerState}
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    Hit,
    Stand,
    NextRound,
    NextGame,
    PlayHand { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpponentAction {
    Hit,
    Stand,
    PlayHand { index: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum RoundOutcome {
    PlayerWon,
    OpponentWon,
    Tied,
}

#[derive(Debug, Clone)]
pub enum GamePhase {
    PlayerTurn,
    OpponentThinking { until: Instant },
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
    GameOver { winner: Player },
}

#[derive(Debug)]
pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    pub game_phase: GamePhase,
    pub round_outcome: Option<RoundOutcome>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player: PlayerState {
                name: "Your Name".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                // Interim fixed hand, same values as before — random
                // dealing from DEFAULT_SIDE_DECK arrives in T007
                hand: vec![
                    Some(Card::Plus(5)),
                    Some(Card::Plus(3)),
                    Some(Card::Plus(6)),
                    Some(Card::Plus(2)),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
                played_card: false,
            },
            opponent: PlayerState {
                name: "Opponent".to_string(),
                dealer_row: vec![],
                played_row: vec![],
                hand: vec![
                    Some(Card::Plus(2)),
                    Some(Card::Plus(6)),
                    Some(Card::Plus(1)),
                    Some(Card::Plus(4)),
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
                played_card: false,
            },
            game_phase: GamePhase::PlayerTurn,
            round_outcome: None,
        }
    }

    /// Take the keys from the game loop and hand them it to action_from_key
    ///
    pub fn handle_game_input(&mut self, key: char) -> Option<GameAction> {
        self.game_action_from_key(key)
    }

    /// Convert a key pressed into an Action
    ///
    pub fn game_action_from_key(&self, key: char) -> Option<GameAction> {
        match key {
            '1' | '2' | '3' | '4' => Some(GameAction::PlayHand {
                index: key.to_digit(10)? as usize - 1,
            }),
            'd' | ' ' => Some(GameAction::Hit),
            's' => Some(GameAction::Stand),
            'n' => Some(GameAction::NextRound),
            'g' => Some(GameAction::NextGame),
            _ => None,
        }
    }

    /// Centralize action validation
    ///
    pub fn apply_game_action(&mut self, action: GameAction) {
        match action {
            GameAction::Hit => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) && !self.player.stood {
                    self.player_hit();
                    self.resolve_after_action();
                }
            }
            GameAction::Stand => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.player_stand();
                    self.resolve_after_action();
                }
            }
            GameAction::PlayHand { index } => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.play_card(index);
                    self.resolve_after_action();
                }
            }
            GameAction::NextRound => {
                if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
                    self.next_round();
                    self.resolve_after_action();
                }
            }
            GameAction::NextGame => {
                if matches!(self.game_phase, GamePhase::GameOver { winner }) {
                    self.new_game();
                }
            }
        }
    }

    /// Take an OpponentAction and perform the action by calling appropriate fn's
    ///
    pub fn apply_opponent_action(&mut self, action: OpponentAction) {
        match action {
            OpponentAction::Hit => {
                self.opponent_hit();
                self.resolve_after_action();
            }
            OpponentAction::Stand => {
                self.opponent_stand();
                self.resolve_after_action();
            }
            OpponentAction::PlayHand { index } => {
                self.opponent_play_card(index);
                self.resolve_after_action();
            }
        }
    }

    /// After each state mutation action, check scores to see if status or
    /// GamePhase updates need to be applied
    ///
    fn resolve_after_action(&mut self) {
        // Don't resolve if awaiting next turn
        if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
            return;
        }

        // If player has played a card, move to Opponent's turn and reset flag
        if self.player.played_card {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
            self.player.played_card = false;
        }

        let player_score = self.player.score();
        let opponent_score = self.opponent.score();

        // Check for bust
        //
        // If player busts, round ends
        if player_score > 20 {
            self.player.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            return;
        }

        // If opponent busts, round ends
        if opponent_score > 20 {
            self.opponent.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            return;
        }

        // If player is at 20, stand
        if player_score == 20 {
            self.player.stood = true
        }

        // If opponent at 20, stand
        if opponent_score == 20 {
            self.opponent.stood = true;
        }

        // Check if both players have stood
        if self.player.stood && self.opponent.stood {
            self.game_phase = GamePhase::RoundEnd;
        }
    }

    /// Perform end of round tabulations and score updates,
    /// transitioning into AwaitingNextRound phase.
    ///
    fn finalize_round(&mut self) {
        let player_score = self.player.score();
        let opponent_score = self.opponent.score();

        // Check scores and decide round outcome
        let outcome = if self.player.bust {
            RoundOutcome::OpponentWon
        } else if self.opponent.bust || player_score > opponent_score {
            RoundOutcome::PlayerWon
        } else if opponent_score > player_score {
            RoundOutcome::OpponentWon
        } else {
            RoundOutcome::Tied
        };

        // Apply reward outcome (increment rounds won or not if tied)
        self.round_outcome = Some(outcome);
        self.apply_reward(outcome);

        // Check for game win else we move into AwaitingNextRound
        if self.player.rounds_won == 3 {
            self.game_phase = GamePhase::GameOver {
                winner: Player::Player,
            }
        } else if self.opponent.rounds_won == 3 {
            self.game_phase = GamePhase::GameOver {
                winner: Player::Opponent,
            }
        } else {
            self.game_phase = GamePhase::AwaitingNextRound;
        }
    }

    /// Apply round reward to the player who won, or nothing if tied
    ///
    fn apply_reward(&mut self, outcome: RoundOutcome) {
        match outcome {
            RoundOutcome::OpponentWon => {
                self.opponent.rounds_won += 1;
            }
            RoundOutcome::PlayerWon => {
                self.player.rounds_won += 1;
            }
            RoundOutcome::Tied => {}
        }
    }

    /// Check the GamePhase each tick of the gameloop and take appropriate actions
    ///
    pub fn update(&mut self) {
        match self.game_phase {
            GamePhase::PlayerTurn => {
                // If player is done for the round, immediately switch back to Opponent
                if !self.player_can_act() {
                    self.game_phase = GamePhase::OpponentThinking {
                        until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
                    };
                }
            }
            GamePhase::OpponentThinking { until } => {
                if Instant::now() >= until {
                    self.game_phase = GamePhase::OpponentTurn;
                }
            }
            GamePhase::OpponentTurn => {
                self.play_opponent_turn();
            }
            GamePhase::RoundEnd => {
                self.finalize_round();
            }
            _ => {}
        }
    }

    /// Check if opponent can still play this round
    ///
    fn opponent_can_act(&self) -> bool {
        !self.opponent.stood && !self.opponent.bust
    }

    /// Check if player can still play this round
    ///
    fn player_can_act(&self) -> bool {
        !self.player.stood && !self.player.bust
    }

    /// Deal a card to the player
    ///
    fn player_hit(&mut self) {
        self.player.dealer_row.push(draw_dealer_card());

        // Set gamephase to opponent's turn
        if self.opponent_can_act() {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
            };
        }
    }

    /// Opponent's play logic:
    /// Return an OpponentAction based on opponent's hand and state
    ///
    fn decide_opponent_move(&self) -> OpponentAction {
        let score = self.opponent.score();
        let target = 20 - score;

        // Interim: only fixed Plus cards are recognized, matching current
        // behavior — the generalized predicate arrives in T008
        let card_hits_twenty =
            |card: &Card| -> bool { matches!(card, Card::Plus(n) if *n as i32 == target) };

        if let Some(index) = self.first_hand_index(card_hits_twenty) {
            return OpponentAction::PlayHand { index };
        }

        // if score is >= 17, stand
        if score >= STAND_THRESHOLD as i32 {
            return OpponentAction::Stand;
        }

        OpponentAction::Hit
    }

    /// Helper to finds first occurrence of card in hand that matches predicate
    ///
    fn first_hand_index<P>(&self, mut pred: P) -> Option<usize>
    where
        P: FnMut(&Card) -> bool,
    {
        self.opponent
            .hand
            .iter()
            .enumerate()
            .find_map(|(i, slot)| slot.as_ref().filter(|card| pred(card)).map(|_| i))
    }

    /// Play the opponent's turn (deal, play card, stand)
    ///
    fn play_opponent_turn(&mut self) {
        match self.decide_opponent_move() {
            OpponentAction::Hit => {
                self.opponent_hit();
            }
            OpponentAction::Stand => {
                self.opponent_stand();
            }
            OpponentAction::PlayHand { index } => {
                self.opponent_play_card(index);
            }
        }

        self.game_phase = GamePhase::PlayerTurn;
        self.resolve_after_action();
    }

    /// Opponent hits (gets dealer card)
    ///
    fn opponent_hit(&mut self) {
        self.opponent.dealer_row.push(draw_dealer_card());
    }

    /// Set gamestate to opponent's turn if we are on the player's turn
    ///
    pub fn player_stand(&mut self) {
        // Only allow if GamePhase is player's turn
        if let GamePhase::PlayerTurn = self.game_phase {
            self.player.stood = true;

            if self.opponent_can_act() {
                self.game_phase = GamePhase::OpponentThinking {
                    until: Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS),
                };
            }
        }
    }

    /// Opponent Stands
    ///
    fn opponent_stand(&mut self) {
        self.opponent.stood = true;
    }

    ///  Remove card from player hand and add it to played_row
    ///
    fn play_card(&mut self, index: usize) {
        let Some(Some(card)) = self.player.hand.get(index).copied() else {
            return;
        };

        // Interim: only fixed-value cards commit; ± / flip / tiebreaker
        // handling lands in T003-T005 (they can't appear in hands yet)
        let Some(value) = fixed_face_value(card) else {
            return;
        };

        self.player.hand[index] = None;
        self.player.played_row.push(PlayedCard { card, value });
    }

    /// Opponent plays card
    ///
    fn opponent_play_card(&mut self, index: usize) {
        let Some(Some(card)) = self.opponent.hand.get(index).copied() else {
            return;
        };

        let Some(value) = fixed_face_value(card) else {
            return;
        };

        self.opponent.hand[index] = None;
        self.opponent.played_row.push(PlayedCard { card, value });
    }

    /// Setup for next round.
    /// Clear the player and opponent's dealer and played rows, and reset flags.
    fn setup_next_round(&mut self) {
        // Clear dealer row for both players
        self.player.dealer_row = vec![];
        self.opponent.dealer_row = vec![];

        // Clear played row for both players
        self.player.played_row = vec![];
        self.opponent.played_row = vec![];

        // Reset stood and busted flags
        self.player.bust = false;
        self.player.stood = false;
        self.opponent.bust = false;
        self.opponent.stood = false;

        // Reset round outcome
        self.round_outcome = None;

        // Set GamePhase to player turn
        self.game_phase = GamePhase::PlayerTurn;
        // Reset round outcome
        self.round_outcome = None;
    }

    /// If in proper game phase, setup next round
    ///
    fn next_round(&mut self) {
        if let GamePhase::AwaitingNextRound = self.game_phase {
            self.setup_next_round();
        }
    }

    /// Reset all game stats
    ///
    fn new_game(&mut self) {
        if let GamePhase::GameOver { winner: _ } = self.game_phase {
            self.player.rounds_won = 0;
            self.opponent.rounds_won = 0;
            self.setup_next_round();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a random dealer card, 0-10 inclusive (intentional variant)
///
fn draw_dealer_card() -> PlayedCard {
    let n: u8 = rand::random_range(0..=10);
    PlayedCard {
        card: Card::Dealer(n),
        value: n as i8,
    }
}

/// The committed value of a card that needs no play-time choice,
/// or None for kinds that do (or that have a board effect instead)
///
fn fixed_face_value(card: Card) -> Option<i8> {
    match card {
        Card::Plus(n) => Some(n as i8),
        Card::Minus(n) => Some(-(n as i8)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dealer_draw_stays_within_bounds_and_hits_both_ends() {
        let mut seen_zero = false;
        let mut seen_ten = false;

        for _ in 0..1000 {
            let pc = draw_dealer_card();
            let Card::Dealer(n) = pc.card else {
                panic!("dealer draw produced a non-dealer card: {:?}", pc.card);
            };

            assert!(n <= 10, "dealer draw out of bounds: {n}");
            assert_eq!(pc.value, n as i8);

            seen_zero |= n == 0;
            seen_ten |= n == 10;
        }

        // Catches the range shrinking (e.g. 0..10 instead of 0..=10);
        // odds of a false failure are (10/11)^1000 — negligible
        assert!(seen_zero, "0 never drawn in 1000 draws");
        assert!(seen_ten, "10 never drawn in 1000 draws");
    }

    #[test]
    fn fixed_face_value_covers_only_choice_free_kinds() {
        assert_eq!(fixed_face_value(Card::Plus(4)), Some(4));
        assert_eq!(fixed_face_value(Card::Minus(3)), Some(-3));
        assert_eq!(fixed_face_value(Card::PlusMinus(2)), None);
        assert_eq!(fixed_face_value(Card::Tiebreaker), None);
        assert_eq!(
            fixed_face_value(Card::Flip(crate::card::FlipKind::TwoFour)),
            None
        );
    }
}
