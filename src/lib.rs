pub mod frame;
pub mod player;
pub mod render;
pub mod config;
pub mod game;
pub mod board;
pub mod card;
pub mod portrait;
pub mod banter;
pub mod play_log;
pub mod profile;
pub mod opponent;
pub mod opponent_select;
pub mod deck_builder;
pub mod campaign;
pub mod campaign_map;
pub mod economy;
pub mod shop;
pub mod screen;
pub mod menu;
pub mod overlay;
pub mod layout;
pub mod app;
pub mod settings;
pub mod save;
pub mod audio;
pub mod stats;
pub mod records;
pub mod wager;
pub mod motion;

// Card size
pub const CARD_WIDTH: usize = 9;
pub const CARD_HEIGHT: usize = 5;

// Padding
/// Horizontal padding of an overlay box, **per side**: the box is
/// `content_width + 2 * H_PAD` wide, and `inner` is inset `H_PAD / 2` on
/// the left and the right.
pub const H_PAD: usize = 4;
/// Vertical padding of an overlay box, **in total** (not per side): the box
/// is `content_height + V_PAD` tall, and `inner` is inset `V_PAD / 2` at the
/// top and the bottom — so `inner` is exactly as tall as the content.
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

// Spec 027 — one-shot board transitions (drawing only; see motion.rs). Bounds
// from the spec, pinned by motion::tests::beats_are_named_constants_within_bounds:
// SELECTION_PULSE_MS <= ARRIVAL_BEAT_MS <= 1000, ARRIVAL_BEAT_MS <= POPUP_BEAT_MS
// <= 1000, and THINKING_STEP_MS * 2 <= OPPONENT_THINKING_TIME_MS.
pub const ARRIVAL_BEAT_MS: u64 = 600; // a card arriving / a total changing draws Strong this long
pub const POPUP_BEAT_MS: u64 = 800; // the round/game popup waits this long after the round resolves
pub const THINKING_STEP_MS: u64 = 300; // the thinking indicator steps . / .. / ... at this cadence

// Campaign-map starfield twinkle period, per star. Deliberately slower than the
// selection pulse so the ambient backdrop reads as depth, not a synchronized
// blink (see design/brief.md's Motion amendment).
pub const STARFIELD_TWINKLE_MS: u64 = 3000;

// Opponent thinking time
pub const OPPONENT_THINKING_TIME_MS: u64 = 1000;
