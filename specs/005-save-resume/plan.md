# Plan: Save & Resume (mid-match)

## Approach

Persist the in-progress match as JSON on disk, mirroring the settings
persistence spec 004 established. The engine's leaf state types gain `serde`
derives; a small **serializable projection** (`SavedGame`) captures a match
in a form that's independent of the runtime types, carries a schema version,
and sidesteps the one field that can't be serialized. `App` writes the save
whenever the match state changes and clears it when the match ends; the
start menu grows a conditional **Continue** item that loads it back.

Two findings from reading the engine shape the whole design:

- **There is no RNG or deck state to persist.** `GameState` holds only
  `{ player, opponent, game_phase, round_outcome }` (game.rs:47) — no `rng`
  field. Hands are dealt once per match (`deal_hand(&mut rand::rng())` in
  `new`/`new_game`) and thereafter live in `PlayerState.hand`; dealer draws
  call the free `draw_dealer_card()` (game.rs:633), which uses a fresh
  thread RNG each time. So everything random that has *already happened* is
  baked into the saved `PlayerState`s (hands, `dealer_row`, `played_row`),
  and future draws are fresh regardless of saving — not observable as
  "unfaithful." Nothing about the RNG needs to be saved or restored.
- **The only non-serde field is `GamePhase::OpponentThinking { until:
  Instant }`** (game.rs:39). `Instant` has no meaningful serialization. Every
  other `GamePhase` variant is trivially serde-able. The projection drops
  the `Instant` on save and re-arms a fresh timer on load.

## Dependencies

None new. `serde` (derive), `serde_json`, and `directories` are already in
`Cargo.toml` from spec 004.

## Core types

### Serde derives on the engine's leaf types

Add `#[derive(Serialize, Deserialize)]` (alongside the existing derives) to
the plain data types that make up a match:

- `card.rs`: `Card`, `FlipKind`, `PlayedCard`
- `player.rs`: `Player`, `PlayerState`
- `game.rs`: `RoundOutcome`

These are pure data (enums of `u8`/`i8`/unit variants, structs of `String`
/ `Vec` / `bool` / `usize`) — the derives are mechanical and change no game
logic. `GamePhase` and `GameState` deliberately do **not** derive serde;
the projection below stands in for them.

### The saved-match projection (`save.rs`, new module)

```rust
const SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct SavedGame {
    version: u32,
    player: PlayerState,
    opponent: PlayerState,
    phase: SavedPhase,
    round_outcome: Option<RoundOutcome>,
}

// Mirrors GamePhase minus the Instant. GameOver is absent on purpose — a
// finished match is never saved (the save is cleared on completion).
#[derive(Serialize, Deserialize)]
enum SavedPhase {
    PlayerTurn,
    AwaitingSignChoice { hand_index: usize },
    OpponentThinking,   // the Instant is dropped here, re-armed on load
    OpponentTurn,
    RoundEnd,
    AwaitingNextRound,
}
```

Conversions live in `save.rs` (keeping game.rs free of persistence
concerns; all the fields it reads/writes are already `pub`):

- **`GameState` → `SavedGame`**, fallible: returns `None` for a `GameOver`
  match (the signal to *clear* rather than write). `OpponentThinking { .. }`
  maps to `SavedPhase::OpponentThinking`.
- **`SavedGame` → `GameState`**: rebuild the struct; `SavedPhase::
  OpponentThinking` becomes `GamePhase::OpponentThinking { until:
  Instant::now() + Duration::from_millis(OPPONENT_THINKING_TIME_MS) }`, so
  the opponent's think-pause resumes fresh rather than skipping. Every other
  variant maps one-to-one.

Resume is therefore exact for everything the player can observe: hands,
both tables, round wins, whose turn, totals, and the pending sign-choice if
any. The only differences are unobservable — future RNG and the re-armed
think timer.

## Persistence (`save.rs`)

Mirrors `settings.rs`, so the failure behavior is identical and already
proven:

- **Location.** `directories::ProjectDirs::from("", "", "kaazap")` →
  `data_dir().join("saves")`, file `savegame.json` — a dedicated **`saves/`**
  subdirectory under the app's per-user data dir (created on first write;
  human-ruled). Settings stay in `config_dir()`; on platforms that don't
  split data from config (macOS, where both resolve to Application Support)
  the savegame still sits apart in its own `saves/` folder.
- **`save(game: &GameState)`** — `to_saved(game)`: `Some(sg)` → write pretty
  JSON (create the dir first); `None` (GameOver) → `clear()`. Best-effort:
  a write failure is swallowed, never fatal (a save you can't write isn't
  worth crashing a game over).
- **`load() -> Option<GameState>`** — read + `serde_json::from_str`; on any
  error (missing, unreadable, malformed, or `version != SAVE_VERSION`)
  return `None`. A filesystem-free `from_json` core makes the fallback and
  the version check unit-testable.
- **`exists() -> bool`** — is there a *loadable* save? Implemented as
  `load().is_some()` (validity, not mere file presence, gates Continue).
- **`clear()`** — remove the file if present; absence is success.

## Save & clear triggers (`app.rs`)

`GameState` only mutates in two places: `apply_game_action` (player input,
routed from `handle_key`) and `update()` (the tick state machine — opponent
moves, round resolution). So `App` saves at exactly those points, and only
when something changed:

- **On new match** (StartGame / in-game "play again") — save the fresh
  position immediately, so quitting right away still leaves a resumable
  game.
- **After a player action** that reached `InGame` in `handle_key` — save.
- **After `update()` in `tick`** — save **iff** the game phase changed this
  tick (the discriminant differs from before the call). Idle ticks (waiting
  on input in `PlayerTurn` / `AwaitingNextRound`) change nothing and don't
  rewrite the file; every opponent move / round transition does change the
  phase and is captured. This keeps writes to a handful per match, not one
  per frame.
- **On `GameOver`** — `save()` sees `to_saved` return `None` and clears the
  file. So finishing a match removes the save as a side effect of the same
  code path.

Because the file always reflects the current position, **quitting needs no
special handling** — `main.rs` is untouched; the last save is already
current, and even an unclean terminal close resumes to the last change
(≤ one action old, honoring the spec's "lose at most the current turn").

`App` caches `has_save: bool` (seeded at startup from `save::exists()`, set
`true` on any write, `false` on clear) so the menu never does file I/O per
frame.

## The Continue menu item (`menu.rs`)

`MenuItem` gains a `Continue` variant. The menu becomes a computed list
rather than a fixed `MenuItem::iter()`, so Continue can be present only when
a save exists:

```rust
pub struct MenuState {
    items: Vec<MenuItem>,   // [Continue?] Start Game, How To Play, Settings
    selected: usize,        // index into items (was: selected: MenuItem)
    title_text: Vec<&'static str>,
}
impl MenuState { pub fn new(has_save: bool) -> Self { /* prepend Continue iff has_save */ } }
```

`move_selection` already collects into a `Vec` and indexes — it just reads
`self.items` now. `Display` gets a `Continue => "Continue"` arm. `App`
builds every `StartMenu` through a helper that passes `self.has_save`, so
the item is correct on first launch and every return to the menu.

## Discard confirmation, and consolidating the modals (`app.rs`)

Choosing **Start Game** while a save exists must confirm before discarding
it. That's a third modal over the menu, alongside spec 004's `overlay` (How
to Play / `?` help) and `settings_panel`. The T010 review anticipated this
exact moment ("worth a single modal enum if a third modal appears"), so the
plan **collapses the parallel `Option` fields into one**:

```rust
enum Modal {
    Help(Overlay),           // How to Play, ? help (static text)
    Settings(SettingsState), // the settings panel
    ConfirmNewGame,          // the discard-save confirmation
}
// App: modal: Option<Modal>   (replaces `overlay` + `settings_panel`)
```

`handle_key` matches `&mut self.modal` first (one place, mutual exclusion
enforced by the type), routing to the active modal; `draw` matches
`&self.modal` to paint it. The confirm modal renders a two-choice box
(reusing `OverlayLayout` / `draw_box`, like the settings panel) —
**Yes / No**, navigated with the established cursor vocabulary and
**defaulting to No** (the safe choice); Enter selects, Esc cancels. Yes →
discard the save and start a fresh match; No / Esc → back to the menu, save
intact.

*(Human-ruled: the enum. The considered alternative — a third parallel
`confirm: Option<ConfirmState>` field — is less refactoring but keeps the
implicit "only one is ever Some" invariant the T010 review flagged, so it's
not taken.)*

## Architecture / flow changes (file-by-file)

- **`card.rs` / `player.rs` / `game.rs`** — add serde derives to the leaf
  types listed above. No logic change. (game.rs also exposes nothing new;
  `RoundOutcome` gains derives and everything `save.rs` needs is already
  `pub`.)
- **`save.rs`** (new) — `SavedGame` / `SavedPhase`, the conversions,
  `save` / `load` / `exists` / `clear`, `SAVE_VERSION`, and the unit tests.
- **`menu.rs`** — `MenuItem::Continue`; `MenuState` as a computed item list
  keyed on `has_save`; `Display` arm; update the existing nav test.
- **`app.rs`** — replace `overlay` + `settings_panel` with `modal:
  Option<Modal>`; add `has_save`; a `start_menu()` helper; route
  `MenuItem::Continue` → `save::load()` → `Screen::InGame`;
  `MenuItem::StartGame` → confirm modal when `has_save` else new match; the
  confirm modal's input + draw; and the save/clear calls after actions,
  after `update()` on a phase change, and on match creation.
- **`lib.rs`** — register `mod save;`.
- **Untouched:** `main.rs` (no quit hook needed), `board.rs` / `frame.rs` /
  `layout.rs` / `render.rs` / `audio.rs` / `settings.rs`. The game/render/
  audio boundaries stay intact — saving observes `GameState`, it doesn't
  reach into rendering, and the engine gains only derives.

## Known limitations

- **Fresh RNG after resume.** Future dealer draws differ from an unsaved
  run. Unobservable (the player never saw the un-drawn cards), and there's
  no RNG state to save; documented so it isn't mistaken for a bug.
- **Re-armed think timer.** Resuming mid–opponent-think restarts the short
  think pause rather than continuing it. Imperceptible.
- **Single autosave, no slots or manual save** — per the spec's non-goals.
- **Version bump discards, doesn't migrate.** For v1 an incompatible
  `version` is dropped (no Continue). Real migration arrives if/when a
  campaign save format needs to preserve old files.

## Testing strategy

Per `CLAUDE.md`, the logic gets unit tests; disk I/O is verified by running.

- **Round-trip** (`save.rs`): build representative `GameState`s (mid–player
  turn with a partial table and a pending sign choice; mid–opponent
  sequence; awaiting-next-round with round wins on the board), convert to
  `SavedGame` and back through JSON, and assert the match position is
  preserved (hands, both rows, `rounds_won`, `stood`/`bust`, phase
  discriminant, `round_outcome`).
- **Instant handling**: an `OpponentThinking` game round-trips to a valid
  `OpponentThinking` (timer re-armed), never panicking on the `Instant`.
- **Fallback**: malformed / empty / wrong-`version` JSON → `from_json`
  returns `None` (mirrors the settings fallback tests).
- **Clear-on-completion**: `to_saved` of a `GameOver` game returns `None`
  (the signal to clear); a save→finish→`exists()` sequence ends `false`
  (exercised where the filesystem is available).
- **Menu**: `MenuState::new(true)` includes `Continue` and `new(false)`
  omits it; navigation wraps over the right item count in both.
- **In-app** (driver, by running): quit mid-match → relaunch → Continue
  restores the board; finish a match → Continue gone; Start Game with a
  save → confirm → Yes/No behave; a hand-corrupted `savegame.json` → no
  Continue, no crash.

## Suggested phasing (detailed in `tasks.md`)

1. **Serde derives + `save.rs` core** — derives on the leaf types,
   `SavedGame`/`SavedPhase` + conversions + `save`/`load`/`exists`/`clear`,
   with the round-trip / fallback / clear unit tests. Review after.
2. **Menu Continue item** — `MenuItem::Continue`, the computed `MenuState`
   list, `has_save` plumbing, nav test. Review after.
3. **App integration** — the `Modal` consolidation, save/clear triggers,
   Continue → load, StartGame → confirm, the confirm modal. Review after.
4. **Verification & close-out** — driver walk-through of every acceptance
   flow, README note if warranted, skeptical-review, ROADMAP + merge.

## Resolved / open decisions

- **Serializable projection over deriving on `GameState`** — isolates the
  wire format, hosts the version field, and is the natural home for the
  `Instant` fix. Not speculative: it exists because `GamePhase` genuinely
  can't derive serde.
- **No RNG persistence** (finding above) — there is no deck/RNG state; not a
  simplification we chose, a fact of the engine.
- **Save on state-change, clear on `GameOver`, no `main.rs` quit hook** —
  the file is always current, so quit/close need no special path.
- **Savegame in a dedicated `saves/` subdir of `data_dir`** (human-ruled),
  `settings.json` stays in `config_dir` — a save is per-user data, kept in
  its own folder rather than loose alongside config.
- **Consolidate the three modals into one `Modal` enum** (human-ruled; the
  T010 review anticipated it) — replaces spec 004's parallel `overlay` +
  `settings_panel` fields.
