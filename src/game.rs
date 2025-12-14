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
    pub fn new_demo() -> Self {
        Self {
            player: PlayerState {
                name: "Player".to_string(),
                dealer_row: vec![LogicCard { value: 7 }, LogicCard { value: 4 }],
                played_row: vec![LogicCard { value: 3 }],
                hand: vec![
                    LogicCard { value: 2 },
                    LogicCard { value: 6 },
                    LogicCard { value: 1 },
                    LogicCard { value: 4 },
                ],
            },
            opponent: PlayerState {
                name: "Opponent".to_string(),
                dealer_row: vec![LogicCard { value: 9 }],
                played_row: vec![],
                hand: vec![
                    LogicCard { value: 5 },
                    LogicCard { value: 3 },
                    LogicCard { value: 6 },
                    LogicCard { value: 2 },
                ],
            },
        }
    }
}

