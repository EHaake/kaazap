use crate::{card::LogicCard, player::PlayerState};

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

    // Deal a card to the player if they are still in the game
    // Check score and toggle bust flag if they are over 20
    pub fn player_deal(&mut self) {
        if let GamePhase::PlayerTurn = self.game_phase {

            let new_dealer_card_val: i32 = rand::random_range(0..=10);
            self.player.dealer_row.push(LogicCard {
                value: new_dealer_card_val,
            });

            if self.player.score() > 20 {
                self.player.bust = true;
                self.game_phase = GamePhase::RoundEnd;
            }
        }
    }

    // Set gamestate to opponent's turn if we are on the player's turn
    pub fn player_stand(&mut self) {
        if let GamePhase::PlayerTurn = self.game_phase {
            self.game_phase = GamePhase::PlayerStood;
        } 
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
