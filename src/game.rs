use crate::{config::Config, player::PlayerState};

#[derive(Debug, Clone, Copy)]
pub enum GamePhase {
    DealOpeningCards,
    PlayerTurn,
    OpponentTurn,
    RoundResolution,
}

pub struct Game {
    pub config: Config,
    pub player: PlayerState,
    pub opponent: PlayerState,
    // pub dealer_deck: Vec<Card>,  // dealer will just randomly draw a +1..+10 (infinite deck)
    pub game_phase: GamePhase,
}
