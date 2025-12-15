use crate::{card::LogicCard, player::PlayerState};
use std::{
    thread,
    time::{Duration, Instant},
};

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
                    LogicCard { value: 5 },
                    LogicCard { value: 3 },
                    LogicCard { value: 6 },
                    LogicCard { value: 2 },
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
                    LogicCard { value: 2 },
                    LogicCard { value: 6 },
                    LogicCard { value: 1 },
                    LogicCard { value: 4 },
                ],
                bust: false,
                stood: false,
                rounds_won: 0,
            },
            game_phase: GamePhase::PlayerTurn,
            round_outcome: None,
        }
    }

    pub fn handle_input(&mut self, key: char) {
        match key {
            '1' | '2' | '3' | '4' => {
                self.play_card(key);
            },
            'd' => self.player_deal(),
            's' => self.player_stand(),
            'n' => self.next_round(),
            _ => {}
        }
    }

    //
    // Check board state for updates
    pub fn update(&mut self, delta: Duration) {
        // TODO: this is inneficient and a bit of a hack to fix a bug, refactor needed
        if !matches!(self.game_phase, GamePhase::AwaitingNextRound) {
            // Opponent wins
            if self.player.score() > 20 {
                self.player.bust = true;
                self.game_phase = GamePhase::RoundEnd;
            } else if self.opponent.score() > 20 {
                self.opponent.bust = true;
                self.game_phase = GamePhase::RoundEnd;
            } else if self.player.score() == 20 || self.player.stood {
                // If player gets to 20 but opponent hasn't stood or busted, they get more turns
                self.player.stood = true;
                if self.opponent.stood || self.opponent.bust {
                    self.game_phase = GamePhase::RoundEnd;
                }
            } else if self.opponent.score() == 20 {
                // If opponent gets to 20 first (player still < 20 and not stood, need to give
                // player more draws)
                self.opponent.stood = true;
                if self.player.stood || self.player.bust {
                    self.game_phase = GamePhase::RoundEnd;
                } else {
                    self.game_phase = GamePhase::PlayerTurn;
                }
            }
        }

        //
        // Match on game phase to decide what to do
        match self.game_phase {
            GamePhase::OpponentThinking { until } => {
                if Instant::now() >= until {
                    self.game_phase = GamePhase::OpponentTurn;
                }
            }
            GamePhase::OpponentTurn => {
                self.play_opponent_turn();
            }
            GamePhase::RoundEnd => {
                if self.player.bust {
                    self.opponent.rounds_won += 1;
                    self.round_outcome = Some(RoundOutcome::OpponentWon);
                } else if self.opponent.bust {
                    self.player.rounds_won += 1;
                    self.round_outcome = Some(RoundOutcome::PlayerWon);
                } else if self.player.stood && self.opponent.stood {
                    // Tie, player wins, opponent wins
                    if self.player.score() == self.opponent.score() {
                        self.round_outcome = Some(RoundOutcome::Tied);
                    } else if self.player.score() > self.opponent.score() {
                        self.player.rounds_won += 1;
                        self.round_outcome = Some(RoundOutcome::PlayerWon);
                    } else if self.player.score() < self.opponent.score() {
                        self.opponent.rounds_won += 1;
                        self.round_outcome = Some(RoundOutcome::OpponentWon);
                    }
                }

                self.setup_for_next_round();
            }
            GamePhase::PlayerTurn => {
                // If player has stood, need to go to next Opponent's turn
                if self.player.stood {
                    self.game_phase = GamePhase::OpponentThinking {
                        until: Instant::now() + Duration::from_secs(1),
                    };
                }
            }
            // TODO: Handle rest of phases here
            _ => {}
        }
    }

    // Play the opponent's turn (deal, play card, stand)
    fn play_opponent_turn(&mut self) {
        self.opponent_deal();
    }

    // Opponent hits
    fn opponent_deal(&mut self) {
        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.opponent.dealer_row.push(LogicCard {
            value: new_dealer_card_val,
        });

        self.game_phase = GamePhase::PlayerTurn;
    }

    // Deal a card to the player if they are still in the game
    // Check score and toggle bust flag if they are over 20
    pub fn player_deal(&mut self) {
        if let GamePhase::PlayerTurn = self.game_phase
            && !self.player.stood
        {
            let new_dealer_card_val: i32 = rand::random_range(0..=10);
            self.player.dealer_row.push(LogicCard {
                value: new_dealer_card_val,
            });

            // Set gamephase to opponent's turn
            // self.game_phase = GamePhase::OpponentTurn;
            self.game_phase = GamePhase::OpponentThinking {
                until: Instant::now() + Duration::from_secs(1),
            };
        }
    }

    // Set gamestate to opponent's turn if we are on the player's turn
    pub fn player_stand(&mut self) {
        // Only allow if GamePhase is player's turn
        if let GamePhase::PlayerTurn = self.game_phase {
            self.game_phase = GamePhase::OpponentTurn;
            self.player.stood = true;
        }
    }

    fn next_round(&mut self) {
        if let GamePhase::AwaitingNextRound = self.game_phase {
            // Clear dealer row for both players
            self.player.dealer_row = vec![];
            self.opponent.dealer_row = vec![];

            // Reset stood and busted flags
            self.player.bust = false;
            self.opponent.bust = false;
            self.player.stood = false;

            // Reset round outcome
            self.round_outcome = None;

            // Set GamePhase to player turn
            self.game_phase = GamePhase::PlayerTurn;
        }
    }

    fn setup_for_next_round(&mut self) {
        self.game_phase = GamePhase::AwaitingNextRound;
    }

    fn play_card(&mut self, key: char) {
        // remove card from player hand
        // add it to played_row
        let digit = key.to_digit(10).unwrap() as usize;

        if digit <= self.player.hand.len() {
            self.player.played_row.push(self.player.hand.remove(digit - 1));
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
