# Plan: Opponent Roster & Personalities

## Approach

Three moves: a small opponent **data model + roster**, **parameterizing the
existing AI/dealing** to read a per-match opponent profile, and a new
**opponent-select `Screen`**. No new dependencies. The engine's decision
function and phase machine are reused as-is — only their inputs change from
globals to a profile carried on `GameState`.

## Data model — new `src/opponent.rs`

One lean `Copy` struct plus a roster (AI param is just a threshold field — no
nested type; difficulty is a display string — no enum yet, per Simplicity):

```rust
#[derive(Debug, Clone, Copy)]
pub struct OpponentProfile {
    pub id: &'static str,           // stable key, used by the save file
    pub name: &'static str,         // board + select screen
    pub difficulty: &'static str,   // display label
    pub blurb: &'static str,        // one-line flavor for the select row
    pub stand_threshold: usize,     // replaces the global STAND_THRESHOLD
    pub side_deck: &'static [Card], // this opponent's 10-card side deck
}

pub const DEFAULT_OPPONENT: OpponentProfile = /* "Opponent", 17, DEFAULT_SIDE_DECK */;
pub const OPPONENTS: [OpponentProfile; 5] = [ /* easy → hard */ ];
pub fn opponent_by_id(id: &str) -> Option<OpponentProfile>; // searches OPPONENTS
```

- All-`Copy` fields ⇒ `GameState` stores an `OpponentProfile` **by value** (no
  lifetimes on `GameState`; roster stays a `const`).
- `DEFAULT_OPPONENT` is a distinct neutral profile (threshold `STAND_THRESHOLD`
  = 17, `DEFAULT_SIDE_DECK`, name "Opponent") used by `new()` / `Default` /
  tests / old-save fallback — so **existing AI tests pass unchanged**. The 5
  `OPPONENTS` are the selectable roster.
- Roster values are tunable data (balance is a later cross-cutting pass): 5
  opponents, distinct thresholds (~15–19) and differently-weighted 10-card
  decks. Names/blurbs original (IP note in `DECISIONS.md`).

## Engine changes

- **`src/card.rs`** — `deal_hand<R: Rng + ?Sized>(rng, deck: &[Card]) ->
  Vec<Option<Card>>` (`choose_multiple` over the passed slice). Player passes
  `&DEFAULT_SIDE_DECK`, opponent passes `profile.side_deck`. Update the
  `deal_hand` test.
- **`src/game.rs`** —
  - `GameState` gains `pub opponent_profile: OpponentProfile`.
  - `GameState::new()` → `Self::with_opponent(DEFAULT_OPPONENT)`; add
    `pub fn with_opponent(profile: OpponentProfile) -> Self`. Both seed
    `opponent.name = profile.name` and deal from `profile.side_deck`.
  - `decide_opponent_move` reads `self.opponent_profile.stand_threshold`
    (was `STAND_THRESHOLD`, ~line 428).
  - `new_game()` (post-`GameOver` rematch, ~line 621) re-deals the opponent
    hand from `self.opponent_profile.side_deck`.
- **`src/lib.rs`** — `pub mod opponent; pub mod opponent_select;`.
  `STAND_THRESHOLD` stays as `DEFAULT_OPPONENT`'s value.

## Opponent-select Screen

- **`src/screen.rs`** — add `OpponentSelect { state: OpponentSelectState }`.
- **New `src/opponent_select.rs`** — mirrors `menu.rs` (the cleanest reusable
  list: index + `move_selection` wrap via `rem_euclid`, `src/menu.rs:94`).
  `OpponentSelectState { selected: usize }` over `OPPONENTS`; an input handler
  (Up/`w`, Down/`s`, Enter/Space → `Pick(index)`, Esc/`x` → `Back`); a
  `draw(&self, frame, &Config, Emphasis)` — "Choose Opponent" title, one row
  per opponent (name · difficulty · blurb) with the shared `pulse` marker,
  reusing `MenuLayout` (`src/layout.rs:126`) or a light analogue, plus a hint
  line. Emacs nav is free via the existing `resolve_key` boundary (`main.rs`).
- **`src/app.rs`** —
  - `apply_menu_event` `StartGame` arm (~565): save present → `ConfirmNewGame`
    whose **Yes navigates to `OpponentSelect`**; no save → `OpponentSelect`
    directly.
  - Replace `start_new_game()` with `start_match(profile: OpponentProfile)`
    (`Screen::InGame` via `GameState::with_opponent`, reset `prev_audio`,
    `save_game()`); called by the confirm-Yes branch and the select `Pick`.
  - The three `match &self.screen` sites a new variant forces: input route
    (~398), `?`-help (~385; `OpponentSelect` inert), draw (~615). Select
    `Back`/Esc → `start_menu()`. Emit the existing menu nav/select SFX.

## Save / resume (`src/save.rs`)

- `SavedGame` gains `opponent_id: String` with `#[serde(default =
  "default_opponent_id")]` (→ `DEFAULT_OPPONENT.id`). **No `SAVE_VERSION`
  bump:** an old save lacks the field and defaults to the "Opponent" it was
  actually played against — correct, no data loss.
- `to_saved`: `opponent_id = game.opponent_profile.id.to_string()`.
- `from_saved`: `opponent_profile = opponent_by_id(&saved.opponent_id)
  .unwrap_or(DEFAULT_OPPONENT)`.

## Testing

- **Parameterized AI** (`game.rs`): same board, different `stand_threshold` →
  different decision (e.g. score 16 → `Hit` at 17, `Stand` at 15; score 17 →
  `Stand` at 17, `Hit` at 19). Existing AI tests untouched (default 17).
- **`deal_hand`** draws from the passed deck.
- **Roster integrity**: every deck length ≥ `HAND_SIZE`, ids unique, names
  non-empty.
- **`with_opponent`** seeds name/profile; **`new()`** = "Opponent"/17 baseline.
- **`OpponentSelectState`** navigation wraps; `Pick`/`Back` events.
- **Save round-trip** carries `opponent_id`; a field-less save → default.
- `board.rs` already renders `opponent.name` (confirmed in-app) — seeding it
  from the profile is the only display change; no board-draw edit expected.

## Files

New: `src/opponent.rs`, `src/opponent_select.rs`. Modified: `src/screen.rs`,
`src/game.rs`, `src/card.rs`, `src/app.rs`, `src/save.rs`, `src/lib.rs`,
maybe `src/layout.rs`. No change expected: `src/board.rs`, `src/menu.rs`.

## Resolved decisions

- **Profile stored by value on `GameState`** (all-`Copy` fields) — avoids
  lifetimes and lets the roster be a plain `const`.
- **`DEFAULT_OPPONENT` kept distinct** from the roster so `new()`/tests keep
  the exact current behavior (threshold 17, name "Opponent", default deck).
- **`serde(default)` over a `SAVE_VERSION` bump** — old saves resume correctly
  against the default opponent rather than being discarded.
- **Select is a `Screen`; AI stays deterministic (threshold + deck)** — the
  two human rulings from planning.
