# Plan: Control & Input Polish

## Approach

Three small changes — no new types, modules, or dependencies:

1. **Emacs → arrow translation at the input boundary.** A pure
   `resolve_key(code, modifiers) -> KeyCode` maps `Ctrl+P/N/B/F` to
   `Up/Down/Left/Right` and passes everything else through. `main.rs` runs
   every key through it before routing, so *every* arrow-driven surface
   (start menu, settings, in-game hand cursor, discard-confirm) gets the
   emacs keys with zero per-screen code — Option A, human-ruled.
2. **Space becomes phase-contextual** across the two input layers.
   Round-end/game-over live in the engine's key mapping:
   `game_action_from_key(' ')` returns `NextRound` at `AwaitingNextRound`,
   `NextGame` at `GameOver`, and `None` otherwise. The in-play case lives in
   `app.rs`: on the player's turn, Space is routed to `cursor_confirm` — the
   same path Enter takes — so it plays the highlighted card.
3. **Space plays the selected card in-play; `D` is the sole draw key**
   (added during implementation, human-ruled). Space is the menu's "select /
   confirm" key, so in-game it confirms the highlighted card like Enter. The
   engine's Space mapping drops its old `_ => Hit` fallback, so Space never
   draws at either layer; `D` keeps drawing.

## Core changes

### Emacs translation (`app.rs` + `main.rs`)

```rust
// app.rs — pure, unit-testable, no App needed
pub fn resolve_key(code: KeyCode, modifiers: KeyModifiers) -> KeyCode {
    if modifiers.contains(KeyModifiers::CONTROL)
        && let KeyCode::Char(c) = code
    {
        match c.to_ascii_lowercase() {
            'p' => return KeyCode::Up,
            'n' => return KeyCode::Down,
            'b' => return KeyCode::Left,
            'f' => return KeyCode::Right,
            _ => {}
        }
    }
    code
}
```

`main.rs` resolves once, then does the existing quit-check / route on the
resolved code:

```rust
Event::Key(key_event) => {
    let code = resolve_key(key_event.code, key_event.modifiers);
    match code {
        KeyCode::Char('q') => break 'gameloop,
        _ => app.handle_key(code),
    }
}
```

Everything downstream is untouched — `menu.rs` / `settings.rs` / `app.rs`
already handle `Up/Down/Left/Right`. Only bare keys with `CONTROL` held are
affected; every existing plain key passes through unchanged.

### Space phase-contextual (`game.rs` + `app.rs`)

Two layers. In the engine, split the old `'d' | ' ' => Hit` so Space maps
only to the between-round advances and never to a draw:

```rust
'd' => Some(GameAction::Hit),
' ' => match self.game_phase {
    GamePhase::AwaitingNextRound => Some(GameAction::NextRound),
    GamePhase::GameOver { .. } => Some(GameAction::NextGame),
    _ => None,
},
```

In `app.rs`, Space mirrors Enter on the player's turn so it plays the
highlighted card (the engine never sees Space during play — this guard
intercepts it first):

```rust
KeyCode::Enter | KeyCode::Char(' ') if player_turn => {
    cursor_confirm(game_state, cursor);
    game_changed = true;
}
```

`apply_game_action` already gates each engine action by phase, and
`cursor_confirm` is already the tested play-the-card path. Both layers are
input changes, so they ship with tests (engine mapping in `game.rs`;
`cursor_confirm` behavior already covered — the one-line Space→cursor route
is verified in-app via the driver, since an `App`-level `handle_key` test
would write the real savegame file).

## Architecture / flow changes (file-by-file)

- **`app.rs`**: add `pub fn resolve_key` + its unit tests (needs
  `use crossterm::event::KeyModifiers`). Also extend the InGame routing so
  `KeyCode::Enter | KeyCode::Char(' ') if player_turn` both call
  `cursor_confirm` — Space plays the highlighted card like Enter.
- **`main.rs`**: run each key through `resolve_key` before the quit-check /
  route. One-line change plus the import.
- **`game.rs`**: `game_action_from_key(' ')` maps to `NextRound`/`NextGame`
  at the pauses and `None` otherwise (no longer `Hit`); add tests.
- **`assets/game_overlay_text.txt`**: the `?` help now reads
  `Enter / Space  Play the selected card`, `D  Draw a card`, and keeps
  `N / Space` / `G / Space` and the `Ctrl-P/N/B/F` line. (Menu overlay text
  gained the emacs line in T002.)
- **Untouched:** `menu.rs`, `settings.rs`, `board.rs`, `render.rs`,
  `save.rs`, `audio.rs`, `overlay.rs` — they already speak arrows and the
  existing keys.

## Testing

- **`resolve_key`** (`app.rs`): `Ctrl+P/N/B/F` → `Up/Down/Left/Right`; a bare
  letter (no `CONTROL`) and an already-arrow key pass through unchanged; a
  non-mapped `CONTROL` chord (e.g. `Ctrl+X`) passes through.
- **Space mapping** (`game.rs`): `game_action_from_key(' ')` is `NextRound`
  at `AwaitingNextRound`, `NextGame` at `GameOver`, and `None` at
  `PlayerTurn` (played via the cursor model in `app.rs`, not the engine);
  `'d'` stays `Hit`; `'n'` / `'g'` unchanged.
- **In-app** (driver): `Ctrl+N`/`Ctrl+P` navigate the menu; `Ctrl+B`/`Ctrl+F`
  adjust a settings volume; on the player's turn Space plays the highlighted
  card (board/score change, no draw) while `D` draws and ends the turn;
  Space advances a round and starts a new game; existing keys still work.

## Suggested phasing (detailed in `tasks.md`)

Small — three tasks plus close-out, review after each:

1. **Emacs translation** — `resolve_key` + `main.rs` wiring + tests.
2. **Space-to-advance** — `game.rs` mapping + tests and the help-text touch.
3. **Space plays the selected card** — `app.rs` Space→`cursor_confirm`,
   engine Space mapping drops its `Hit` fallback, help text updated (added
   during implementation).
4. **Verification & close-out** — driver sweep, skeptical-review (light),
   README/ROADMAP, merge on the human's word.

## Resolved decisions

- **Global translation at the boundary** (human-ruled, Option A) — emacs
  keys work everywhere arrows do; the simplest and most consistent
  mechanism, and it needs no per-screen code.
- **`resolve_key` is a pure lib fn** in `app.rs`, so it's unit-tested
  directly rather than through `main.rs` (the binary, which the lib test
  harness doesn't cover).
- **Ctrl+letter is case-folded** (`to_ascii_lowercase`) so `Ctrl+Shift+P`
  and terminal case quirks still map. Only `CONTROL`-held keys are touched,
  so nothing plain regresses.
- **Space plays the selected card in-play; `D` is the sole draw key**
  (human-ruled, added mid-implementation). Split across the two layers on
  purpose: the engine can't express "play the *highlighted* card" (that's a
  cursor concept in `app.rs`, not an indexed `GameAction`), so `app.rs` owns
  the in-play case via `cursor_confirm` and the engine returns `None` for
  Space during play. Keeping both layers non-contradictory (engine never
  maps Space to `Hit`) means Space can't accidentally draw even if the
  `app.rs` guard were ever bypassed.
