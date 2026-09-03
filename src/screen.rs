use crate::{
    app::HandCursor, campaign_map::CampaignMapState, deck_builder::DeckBuilderState,
    game::GameState, menu::MenuState, opponent_select::OpponentSelectState,
};

#[derive(Debug)]
pub enum Screen {
    StartMenu { menu_state: MenuState },
    InGame { game_state: Box<GameState>, cursor: HandCursor },
    OpponentSelect { state: OpponentSelectState },
    DeckBuilder { state: DeckBuilderState },
    CampaignMap { state: CampaignMapState },
}
