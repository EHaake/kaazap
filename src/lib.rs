pub mod frame;
pub mod player;
pub mod render;
pub mod config;
pub mod game;
pub mod board;
pub mod card;

// Card size
pub const CARD_WIDTH: usize = 9;
pub const CARD_HEIGHT: usize = 5;

// Thread sleep time to keep from wasting cycles
pub const GAME_LOOP_SLEEP_MS: u64 = 50;
