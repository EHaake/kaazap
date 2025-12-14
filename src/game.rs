use crate::{card::LogicCard, player::PlayerState};

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    DealOpeningCards,
    PlayerTurn,
    OpponentTurn,
    RoundResolution,
}

pub struct GameState {
    pub player: PlayerState,
    pub opponent: PlayerState,
    // pub dealer_deck: Vec<Card>,  // dealer will just randomly draw a +1..+10 (infinite deck)
    // pub game_phase: GamePhase,
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
        }
    }
    
    // Deal a card to the player if they are still in the game
    // Check score and toggle bust flag if they are over 20
    pub fn deal_player(&mut self) {
        if self.player.bust {
            return;
        }

        let new_dealer_card_val: i32 = rand::random_range(0..=10);
        self.player.dealer_row.push(LogicCard { value: new_dealer_card_val });

        if self.player.score() > 20 {
            self.player.bust = true;
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

