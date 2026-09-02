# Spec: Control & Input Polish

## Summary

Two small, additive control quality-of-life changes surfaced during the
spec 005 review — both about making input feel more natural, neither
changing any existing behavior:

1. **Space to advance** at round end and game end. Today you press `n`
   (next round) or `g` (new game) to move on; Space should work too, so
   "continue" is one consistent key you can lean on without hunting for the
   right letter.
2. **Emacs navigation keys.** `Ctrl+P` / `Ctrl+N` / `Ctrl+B` / `Ctrl+F` as
   synonyms for Up / Down / Left / Right, so the menus respond to the
   home-row emacs bindings a lot of terminal users have in their fingers.

Both are pure additions — every existing control keeps working; these are
extra synonyms layered on top.

The emacs keys need one piece of plumbing: the game loop currently forwards
only a `KeyCode` and drops the modifiers, so `Ctrl+P` arrives as a bare
`p`. This spec threads the modifier through the input boundary (see the
plan for the exact mechanism).

## Goals

1. **Space advances at round/game end.** At the round-end pause
   (`AwaitingNextRound`), Space starts the next round (like `n`); at game
   over (`GameOver`), Space starts a new game (like `g`). During the
   player's turn, Space keeps its current meaning (draw a card).
2. **Emacs nav keys as arrow synonyms.** `Ctrl+P` = Up, `Ctrl+N` = Down,
   `Ctrl+B` = Left, `Ctrl+F` = Right — wherever the arrow keys already
   navigate (start-menu items; settings rows and volume). Inert where the
   matching arrow is (e.g. Left/Right on the vertical start menu do
   nothing, so `Ctrl+B` / `Ctrl+F` there do nothing either).
3. **Strictly additive.** Every existing control still works unchanged —
   `n` / `g`, the arrow keys, `w` / `s`, `a` / `d`, Enter, and Space's
   in-play draw.
4. **Test coverage** for the new input-mapping logic per `CLAUDE.md`
   (Space's phase-contextual action; the emacs→arrow translation as a pure,
   testable mapping).

## Non-goals (explicitly deferred)

- **A configurable / rebindable keymap.** These are fixed additions, not a
  user-editable binding system.
- **Other binding schemes** (vim-style `hjkl` navigation, WASD beyond
  what already exists, etc.). Just the four emacs nav keys and Space.
- **Mouse input.**
- **New on-screen hints for the synonyms.** The existing prompts keep
  showing their current primary keys; the emacs keys and Space-to-advance
  are discoverable extras, not new UI to clutter the hints with. (The
  `?` help overlays may mention them — a plan/implementation detail.)

## Key user flows

### Advancing a round or game with Space

A round ends and the "n: next round" pause shows. You press **Space** and
the next round begins — the same as `n`. When a match ends, **Space**
starts a new game, the same as `g`. Mid-turn, Space still draws a card;
the key just does the sensible "primary action" for whatever phase you're
in.

### Navigating a menu with emacs keys

On the start menu you press **Ctrl+N** / **Ctrl+P** to move the highlight
down / up, exactly like the arrows. In Settings, **Ctrl+P** / **Ctrl+N**
move between the Music and SFX rows and **Ctrl+F** / **Ctrl+B** raise /
lower the selected channel's volume — the same as `←` / `→`.

## Design requirements

- **Additive and unsurprising.** No existing key changes meaning; muscle
  memory is untouched.
- **Consistent direction mapping.** An emacs key means the same direction
  everywhere the arrows do — no per-screen special-casing of what
  `Ctrl+P` "means."
- **No visual clutter.** The primary on-screen hints stay as they are.

## Acceptance criteria

- [ ] At round end (`AwaitingNextRound`), **Space** advances to the next
      round — identical to pressing `n`.
- [ ] At game over (`GameOver`), **Space** starts a new game — identical to
      pressing `g`.
- [ ] During the player's turn, **Space** still draws a card (unchanged).
- [ ] On the start menu, **Ctrl+P** / **Ctrl+N** move the selection up /
      down (like the arrows and `w` / `s`).
- [ ] In Settings, **Ctrl+P** / **Ctrl+N** change the selected row and
      **Ctrl+B** / **Ctrl+F** lower / raise its volume (like the arrows and
      `a` / `d`).
- [ ] Every pre-existing control still works exactly as before.
- [ ] Unit tests cover the Space phase-contextual mapping and the
      emacs→arrow translation; `cargo test` green, `cargo build` no new
      warnings.

## Resolved decisions

- **Emacs keys translate to arrow `KeyCode`s at the input boundary
  (proposed — confirm in review).** The simplest mechanism is to translate
  `Ctrl+P/N/B/F` into `Up/Down/Left/Right` once, where the loop reads the
  key, before routing — so every screen that already handles arrows gets it
  for free with no per-screen code. A **consequence**: the emacs keys then
  also drive the in-game hand cursor and the discard-confirm Yes/No (any
  arrow-driven control), not just the menu and settings. That's a harmless,
  consistent superset of the request. The alternative — scoping strictly to
  menu + settings — needs modifier-aware handling threaded into those
  screens specifically (more code, less consistent). Recommend the global
  translation; flag if you'd rather restrict it.
- **Space is phase-contextual** (not a new global binding): it maps to the
  natural "primary action" of the current phase — draw during play, next
  round at the round pause, new game at game over. This reuses the engine's
  existing phase-aware key handling (as `AwaitingSignChoice` already does)
  and touches `game.rs` input mapping, so it ships with tests.
