use crate::{CARD_HEIGHT, CARD_WIDTH, config::Config};

pub struct Board {
    pub player_origin_x: usize,
    pub opponent_origin_x: usize,
    pub dealer_row_y: usize,
    pub side_row_y: usize,
    pub hand_row_y: usize,
    pub card_spacing_x: usize,
}

impl Board {
    // Constructor
    pub fn from_config(config: &Config) -> Self {
        let middle = config.num_cols / 2;

        Self {
            player_origin_x: 2,
            opponent_origin_x: middle + 2,
            dealer_row_y: 2,
            side_row_y: 10,
            hand_row_y: config.num_rows - CARD_HEIGHT - 1,
            card_spacing_x: CARD_WIDTH + 1,
        }
    }

    pub fn card_position_player_dealer(&self, index: usize) -> (usize, usize) {
        (
            self.player_origin_x + index * self.card_spacing_x,
            self.dealer_row_y,
        )
    }

    pub fn card_position_player_side(&self, index: usize) -> (usize, usize) {
        (
            self.player_origin_x + index * self.card_spacing_x,
            self.side_row_y,
        )
    }

    pub fn card_position_player_hand(&self, index: usize) -> (usize, usize) {
        (
            self.player_origin_x + index * self.card_spacing_x,
            self.hand_row_y,
        )
    }

    pub fn card_position_opponent_dealer(&self, index: usize) -> (usize, usize) {
        (
            self.opponent_origin_x + index * self.card_spacing_x,
            self.dealer_row_y,
        )
    }

    pub fn card_position_opponent_side(&self, index: usize) -> (usize, usize) {
        (
            self.opponent_origin_x + index * self.card_spacing_x,
            self.side_row_y,
        )
    }
}
