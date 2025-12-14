use crate::{card::LogicCard, player::PlayerState};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    PlayerTurn,
    PlayerStood,
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
            },
            game_phase: GamePhase::PlayerTurn,
        }
    }

    // Check board state for updates
    pub fn update(&mut self, delta: Duration) {
        if self.player.score() > 20 {
            self.player.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            // TODO: Opponent wins
        }

        if self.opponent.score() > 20 {
            self.opponent.bust = true;
            self.game_phase = GamePhase::RoundEnd;
            // TODO: Player wins
        }

        // Check if it's the opponent's turn and if so, play their turn
        if let GamePhase::OpponentTurn = self.game_phase {
            self.play_opponent_turn();
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
        if let GamePhase::PlayerTurn = self.game_phase {
            let new_dealer_card_val: i32 = rand::random_range(0..=10);
            self.player.dealer_row.push(LogicCard {
                value: new_dealer_card_val,
            });

            // Set gamephase to opponent's turn
            self.game_phase = GamePhase::OpponentTurn;
        }
    }

    // Set gamestate to opponent's turn if we are on the player's turn
    pub fn player_stand(&mut self) {
        self.game_phase = GamePhase::OpponentTurn;
        self.game_phase = GamePhase::PlayerStood;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
