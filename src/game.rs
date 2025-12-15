use crate::{card::LogicCard, player::PlayerState};
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    PlayerTurn,
    OpponentThinking { until: Instant },
    OpponentTurn,
    RoundEnd,
}

pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    // pub dealer_deck: Vec<Card>,  // dealer will just randomly draw a +1..+10 (infinite deck)
    pub game_phase: GamePhase,
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
        }
    }

    pub fn input_handle(&mut self, key: char) {
        if key == 'd' {
            self.player_deal();
        } else if key == 's' {
            self.player_stand();
        } else if key == 'n' {
            self.next_round();
        }
    }

    //
    // Check board state for updates
    pub fn update(&mut self, delta: Duration) {
        // Opponent wins
        if self.player.score() > 20 {
            self.player.bust = true;
            self.opponent.rounds_won += 1;
            self.game_phase = GamePhase::RoundEnd;
        }

        // Player wins
        if self.opponent.score() > 20 {
            self.opponent.bust = true;
            self.player.rounds_won += 1;
            self.game_phase = GamePhase::RoundEnd;
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
        }

        // Set gamephase to opponent's turn
        // self.game_phase = GamePhase::OpponentTurn;
        self.game_phase = GamePhase::OpponentThinking {
            until: Instant::now() + Duration::from_secs(1),
        };
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
        // Clear dealer row for both players
        self.player.dealer_row = vec![];
        self.opponent.dealer_row = vec![];

        // Reset stood and busted flags
        self.player.bust = false;
        self.opponent.bust = false;
        self.player.stood = false;

        // Set GamePhase to player turn
        self.game_phase = GamePhase::PlayerTurn;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
