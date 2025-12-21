use crate::{card::LogicCard, player::PlayerState};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Deal,
    Stand,
    NextRound,
    PlayHand { index: usize },
}

#[derive(Debug, Clone, Copy)]
pub enum RoundOutcome {
    PlayerWon,
    OpponentWon,
    Tied,
}

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    PlayerTurn,
    OpponentThinking { until: Instant },
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
}

pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    // pub dealer_deck: Vec<Card>,  // dealer will just randomly draw a +1..+10 (infinite deck)
    pub game_phase: GamePhase,
    pub round_outcome: Option<RoundOutcome>,
}
impl GameState {
    // pub fn new_demo() -> Self {
    //     Self {
    //         player: PlayerState {
    //             name: "Your name".to_string(),
    //             dealer_row: vec![LogicCard { value: 7 }, LogicCard { value: 4 }],
    //             played_row: vec![LogicCard { value: 3 }],
    //             hand: vec![
    //                 LogicCard { value: 2 },
    //                 LogicCard { value: 6 },
    //                 LogicCard { value: 1 },
    //                 LogicCard { value: 4 },
    //             ],
    //         },
    //         opponent: PlayerState {
    //             name: "Opponent".to_string(),
    //             dealer_row: vec![LogicCard { value: 9 }],
    //             played_row: vec![],
    //             hand: vec![
    //                 LogicCard { value: 5 },
    //                 LogicCard { value: 3 },
    //                 LogicCard { value: 6 },
    //                 LogicCard { value: 2 },
    //             ],
    //         },
    //     }
    // }

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
            },
            game_phase: GamePhase::PlayerTurn,
            round_outcome: None,
        }
    }

    //
    // Take the keys from the game loop and hand them it to action_from_key
    pub fn handle_input(&mut self, key: char) -> Option<Action> {
        self.action_from_key(key)
    }

    //
    // Covert a key pressed into an Action
    pub fn action_from_key(&self, key: char) -> Option<Action> {
        match key {
            '1' | '2' | '3' | '4' => Some(Action::PlayHand {
                index: key.to_digit(10)? as usize - 1,
            }),
            'd' => Some(Action::Deal),
            's' => Some(Action::Stand),
            'n' => Some(Action::NextRound),
            _ => None,
        }
    }

    //
    // Centralize action validation
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Deal => {
                if matches!(self.game_phase, GamePhase::PlayerTurn) && !self.player.stood {
                    self.player_deal();
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

    // This is called whenever we mutate state to check round-end conditions
    fn resolve_after_action(&mut self) {
        // Don't resolve if awaiting next turn
        if matches!(self.game_phase, GamePhase::AwaitingNextRound) {
            return;
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

        // Check for round end conditions
        let player_done = self.player.stood || self.player.bust;
        let opponent_done = self.opponent.stood || self.opponent.bust;

        if player_done && opponent_done {
            self.game_phase = GamePhase::RoundEnd;
        }
    }

    /// Perform end of round tabulations and score updates,
    /// transitioning into AwaitingNextRound phase.
    ///
    fn finalize_round(&mut self) {
        if self.player.bust {
            self.round_outcome = Some(RoundOutcome::OpponentWon);
        } else if self.opponent.bust {
            self.round_outcome = Some(RoundOutcome::PlayerWon);
        } else if self.player.stood && self.opponent.stood {
            // Tie, player wins, opponent wins
            if self.player.score() == self.opponent.score() {
                self.round_outcome = Some(RoundOutcome::Tied);
            } else if self.player.score() > self.opponent.score() {
                self.round_outcome = Some(RoundOutcome::PlayerWon);
            } else if self.player.score() < self.opponent.score() {
                self.round_outcome = Some(RoundOutcome::OpponentWon);
            }
        }

        self.apply_reward();
        self.game_phase = GamePhase::AwaitingNextRound;
    }

    fn apply_reward(&mut self) {
        match self.round_outcome {
            Some(RoundOutcome::OpponentWon) => {
                self.opponent.rounds_won += 1;
            }
            Some(RoundOutcome::PlayerWon) => {
                self.player.rounds_won += 1;
            }
            Some(RoundOutcome::Tied) => {}
            None => {}
        }
    }

    pub fn tick(&mut self) {
        match self.game_phase {
            GamePhase::PlayerTurn => {
                // If player has stood, auto advance to next Opponent's turn
                if self.player.stood {
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

    // Check if opponent's turn
    fn opponent_can_act(&self) -> bool {
        !self.opponent.stood && !self.opponent.bust
    }

    // Deal a card to the player
    fn player_deal(&mut self) {
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

    /// Play the opponent's turn (deal, play card, stand)
    fn play_opponent_turn(&mut self) {
        self.opponent_deal();
        self.game_phase = GamePhase::PlayerTurn;
        self.resolve_after_action();
    }

    /// Opponent hits (gets dealer card)
    fn opponent_deal(&mut self) {
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.opponent.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });
    }

    // Set gamestate to opponent's turn if we are on the player's turn
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

    ///  Remove card from player hand and add it to played_row
    fn play_card(&mut self, digit: usize) {
        // Bounds checking already done before entering this fn
        let Some(Some(LogicCard { value: _ })) = self.player.hand.get(digit) else {
            return;
        };

        if digit < self.player.hand.len() {
            // "Remove" the card from the player's hand by setting value to 0
            let card_to_play = self.player.hand[digit];
            self.player.hand[digit] = None;
            self.player.played_row.push(card_to_play.unwrap());
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
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
