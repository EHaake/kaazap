pub mod frame;
pub mod player;
pub mod render;
pub mod config;
pub mod game;
pub mod board;
pub mod card;
pub mod screen;
pub mod menu;
pub mod overlay;
pub mod app;

// Card size
pub const CARD_WIDTH: usize = 9;
pub const CARD_HEIGHT: usize = 5;

pub const MIN_CARD_SIZE_WIDTH: usize = 7;
pub const MIN_CARD_SIZE_HEIGHT: usize = 4;

// Padding
pub const H_PAD: usize = 4;
pub const V_PAD: usize = 4;

// Opponent Logic
pub const STAND_THRESHOLD: usize = 17;

// Thread sleep time to keep from wasting cycles
pub const GAME_LOOP_SLEEP_MS: u64 = 50;

// Menu animation time
pub const MENU_ANIMATION_TIME_MS: u64 = 500;

// Opponent thinking time
pub const OPPONENT_THINKING_TIME_MS: u64 = 1000;
