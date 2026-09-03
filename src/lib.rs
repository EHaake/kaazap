pub mod frame;
pub mod player;
pub mod render;
pub mod config;
pub mod game;
pub mod board;
pub mod card;
pub mod profile;
pub mod opponent;
pub mod opponent_select;
pub mod screen;
pub mod menu;
pub mod overlay;
pub mod layout;
pub mod app;
pub mod settings;
pub mod save;
pub mod audio;

// Card size
pub const CARD_WIDTH: usize = 9;
pub const CARD_HEIGHT: usize = 5;

// Padding
pub const H_PAD: usize = 4;
pub const V_PAD: usize = 4;

// Offsets
pub const TITLE_X_OFFSET: usize = 21;

// Side-deck cards dealt to each side per game
pub const HAND_SIZE: usize = 4;

// A legal built side deck is exactly this many cards (classic Pazaak). One
// source of truth for the default pool's length, the deck-builder's cap, and
// the "deck is playable" rule.
pub const SIDE_DECK_SIZE: usize = 10;

// Max cards one side may hold on the table in a round (dealer draws +
// played cards). Reaching it auto-stands that side. One source of truth
// for the rule (game.rs), the board grid (layout.rs), and the minimum
// terminal height (config.rs).
pub const MAX_TABLE_CARDS: usize = 12;

// Opponent Logic
pub const STAND_THRESHOLD: usize = 17;

// Thread sleep time to keep from wasting cycles
pub const GAME_LOOP_SLEEP_MS: u64 = 50;

// Selection pulse cadence (shared by menu and board selection)
pub const SELECTION_PULSE_MS: u64 = 500;

// Opponent thinking time
pub const OPPONENT_THINKING_TIME_MS: u64 = 1000;
