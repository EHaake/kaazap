# Plan: Control & Input Polish

## Approach

Two small, independent, additive changes — no new types, modules, or
dependencies:

1. **Emacs → arrow translation at the input boundary.** A pure
   `resolve_key(code, modifiers) -> KeyCode` maps `Ctrl+P/N/B/F` to
   `Up/Down/Left/Right` and passes everything else through. `main.rs` runs
   every key through it before routing, so *every* arrow-driven surface
   (start menu, settings, in-game hand cursor, discard-confirm) gets the
   emacs keys with zero per-screen code — Option A, human-ruled.
2. **Space becomes phase-contextual** in the engine's key mapping:
   `game_action_from_key(' ')` returns the primary action for the current
   phase — `NextRound` at `AwaitingNextRound`, `NextGame` at `GameOver`,
   else `Hit` (unchanged during play).

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

### Space phase-contextual (`game.rs`)

Split the current `'d' | ' ' => Hit` so Space picks the phase's primary
action:

```rust
'd' => Some(GameAction::Hit),
' ' => Some(match self.game_phase {
    GamePhase::AwaitingNextRound => GameAction::NextRound,
    GamePhase::GameOver { .. } => GameAction::NextGame,
    _ => GameAction::Hit,
}),
```

`apply_game_action` already gates each action by phase, so Space only ever
produces the action that phase accepts. This is an engine input change, so
it ships with tests.

## Architecture / flow changes (file-by-file)

- **`app.rs`**: add `pub fn resolve_key` + its unit tests. `handle_key` is
  unchanged — it still receives a resolved `KeyCode`. (Needs
  `use crossterm::event::KeyModifiers`.)
- **`main.rs`**: run each key through `resolve_key` before the quit-check /
  route. One-line change plus the import.
- **`game.rs`**: `game_action_from_key(' ')` becomes phase-contextual; add
  tests.
- **Optional (discoverability):** the `?` game-help / how-to-play text could
  note that Space advances and that the emacs keys mirror the arrows — a
  low-risk asset-text touch, not required by the spec.
- **Untouched:** `menu.rs`, `settings.rs`, `board.rs`, `render.rs`,
  `save.rs`, `audio.rs`, `overlay.rs` — they already speak arrows and the
  existing keys.

## Testing

- **`resolve_key`** (`app.rs`): `Ctrl+P/N/B/F` → `Up/Down/Left/Right`; a bare
  letter (no `CONTROL`) and an already-arrow key pass through unchanged; a
  non-mapped `CONTROL` chord (e.g. `Ctrl+X`) passes through.
- **Space mapping** (`game.rs`): `game_action_from_key(' ')` is `NextRound`
  at `AwaitingNextRound`, `NextGame` at `GameOver`, `Hit` at `PlayerTurn`;
  `'d'` stays `Hit`; `'n'` / `'g'` unchanged.
- **In-app** (driver): `Ctrl+N`/`Ctrl+P` navigate the menu; `Ctrl+B`/`Ctrl+F`
  adjust a settings volume; Space advances a round and starts a new game;
  existing keys still work.

## Suggested phasing (detailed in `tasks.md`)

Small — two tasks plus close-out, review after each:

1. **Emacs translation** — `resolve_key` + `main.rs` wiring + tests.
2. **Space-to-advance** — `game.rs` mapping + tests (and the optional
   help-text touch).
3. **Verification & close-out** — driver sweep, skeptical-review (light),
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
