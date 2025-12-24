use crate::{STAND_THRESHOLD, card::LogicCard, player::{Player, PlayerState}};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Hit,
    Stand,
    NextRound,
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
                hand: vec![
                    Some(LogicCard { value: 5 }),
                    Some(LogicCard { value: 3 }),
                    Some(LogicCard { value: 6 }),
                    Some(LogicCard { value: 2 }),
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
                    Some(LogicCard { value: 2 }),
                    Some(LogicCard { value: 6 }),
                    Some(LogicCard { value: 1 }),
                    Some(LogicCard { value: 4 }),
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
    pub fn handle_input(&mut self, key: char) -> Option<Action> {
        self.action_from_key(key)
    }

    /// Convert a key pressed into an Action
    ///
    pub fn action_from_key(&self, key: char) -> Option<Action> {
        match key {
            '1' | '2' | '3' | '4' => Some(Action::PlayHand {
                index: key.to_digit(10)? as usize - 1,
            }),
            'd' => Some(Action::Hit),
            's' => Some(Action::Stand),
            'n' => Some(Action::NextRound),
            _ => None,
        }
    }

    /// Centralize action validation
    ///
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Hit => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) && !self.player.stood {
                    self.player_hit();
                    self.resolve_after_action();
                }
            }
            Action::Stand => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.player_stand();
                    self.resolve_after_action();
                }
            }
            Action::NextRound => {
                if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
                    self.next_round();
                    self.resolve_after_action();
                }
            }
            Action::PlayHand { index } => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) {
                    self.play_card(index);
                    self.resolve_after_action();
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
                until: Instant::now() + Duration::from_secs(1),
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
            self.game_phase = GamePhase::GameOver { winner: Player::Player }
        } else if self.opponent.rounds_won == 3 {
            self.game_phase = GamePhase::GameOver { winner: Player::Opponent }
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
                        until: Instant::now() + Duration::from_secs(1),
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
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.player.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });

        // Set gamephase to opponent's turn
        if self.opponent_can_act() {
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_secs(1),
            };
        }
    }

    /// Opponent's play logic:
    /// Return an OpponentAction based on opponent's hand and state
    ///
    fn decide_opponent_move(&self) -> OpponentAction {
        let score = self.opponent.score();
        let target = 20 - score;

        let card_hits_twenty = |card: &LogicCard| -> bool {
            card.value == target
        };

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
        P: FnMut(&LogicCard) -> bool,
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
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.opponent.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });
    }

    /// Set gamestate to opponent's turn if we are on the player's turn
    ///
    pub fn player_stand(&mut self) {
        // Only allow if GamePhase is player's turn
        if let GamePhase::PlayerTurn = self.game_phase {
            self.player.stood = true;

            if self.opponent_can_act() {
                self.game_phase = GamePhase::OpponentThinking {
                    until: Instant::now() + Duration::from_secs(1),
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
        // Bounds checking already done before entering this fn
        let Some(Some(LogicCard { value: _ })) = self.player.hand.get(index) else {
            return;
        };

        if index < self.player.hand.len() {
            // "Remove" the card from the player's hand by setting value to 0
            let card_to_play = self.player.hand[index];
            self.player.hand[index] = None;
            self.player.played_row.push(card_to_play.unwrap());
        }
    }

    /// Opponent plays card
    ///
    fn opponent_play_card(&mut self, index: usize) {
        // Bounds checking already done before entering this fn
        let Some(Some(LogicCard { value: _ })) = self.opponent.hand.get(index) else {
            return;
        };

        if index < self.opponent.hand.len() {
            // "Remove" the card from the opponent's hand by setting value to 0
            let card_to_play = self.opponent.hand[index];
            self.opponent.hand[index] = None;
            self.opponent.played_row.push(card_to_play.unwrap());
        }
    }

    /// Setup for next round.
    /// Clear the player and opponent's dealer and played rows, and reset flags.
    fn next_round(&mut self) {
        if let GamePhase::AwaitingNextRound = self.game_phase {
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
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
