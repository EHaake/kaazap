# Tasks: Control & Input Polish

Small controls spec. Each task builds (`cargo build`) and tests
(`cargo test`) green before it's done, with actual output reported. One
commit per task referencing its ID; the draft PR (opened at T001) tracks
the diff. Human-approved to roll through T001–T002 without pausing between;
T004 (Space plays the selected card) was added mid-implementation.

---

- [x] **T001 — Emacs → arrow translation at the input boundary**
  Create the `006-controls-polish` branch (done for the spec/plan) and open
  the draft PR. Add `pub fn resolve_key(code: KeyCode, modifiers:
  KeyModifiers) -> KeyCode` to `app.rs` (`Ctrl+P/N/B/F` → `Up/Down/Left/
  Right`, case-folded; everything else passes through), with
  `use crossterm::event::KeyModifiers`. Wire `main.rs` to resolve each key
  once before the quit-check / `handle_key` route. No downstream changes.
  *Verify: `cargo build` no new warnings; `cargo test` green with new
  `resolve_key` tests (each emacs chord → its arrow; a bare letter and an
  arrow pass through; a non-mapped Ctrl chord passes through); driver —
  `Ctrl+N`/`Ctrl+P` navigate the menu, `Ctrl+B`/`Ctrl+F` adjust a settings
  volume.*

- [x] **T002 — Space-to-advance + help text**
  Split `game_action_from_key`'s `'d' | ' '` so Space is phase-contextual:
  `NextRound` at `AwaitingNextRound`, `NextGame` at `GameOver`, else `Hit`
  (`'d'` stays `Hit`). Update the `?` game-help / how-to-play text to note
  Space advances and that `Ctrl+P/N/B/F` mirror the arrows. *(The in-play
  `Hit` fallback was superseded by T004 — see below.)*
  *Verify: `cargo test` green with new `game_action_from_key` tests; driver —
  Space advances a round and starts a new game.*

- [x] **T004 — Space plays the selected card; `D` is the sole draw key**
  Added mid-implementation at the human's request. In `app.rs`, extend the
  InGame routing so `KeyCode::Enter | KeyCode::Char(' ') if player_turn` both
  call `cursor_confirm` — Space plays the highlighted card like Enter. In
  `game.rs`, drop the Space `_ => Hit` fallback (now `_ => None`) so Space
  never draws at the engine layer; `'d'` stays the draw key. Update the `?`
  game-help text: `Enter / Space  Play the selected card`, `D  Draw a card`
  (Space removed from the draw line), realigned columns.
  *Verify: `cargo build` no new warnings; `cargo test` green with the revised
  `game.rs` Space test (`None` at `PlayerTurn`, `NextRound`/`NextGame` at the
  pauses, `'d'` still `Hit`); driver — on the player's turn Space plays the
  highlighted card (card onto the board, score changes, no dealer draw, turn
  continues) while `D` draws and ends the turn; `?` overlay shows the new
  text. Done — reported verbatim in the session.*

- [x] **T003 — Verification & close-out**
  Driver sweep of every acceptance box (emacs nav on menu + settings; Space
  plays the selected card and `D` draws; Space advances round/game); confirm
  existing controls unchanged. On the human's word: mark spec 006 Shipped in
  `ROADMAP.md`, mark the PR ready, and merge. *(Merged as #7 / 1c7ebfa. Given
  the tiny, driver-verified surface and the human's direct merge directive,
  the formal `skeptical-reviewer` was skipped in favor of a self-run
  adversarial edge check — Space on an empty hand is a safe `cursor_confirm`
  no-op — offered on request.)*
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim (157 passed, 0 warnings).*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The
emacs and Space-to-advance changes are additive; the Space-plays-card change
(T004) reassigns Space's in-play role (draw → play the selected card) with
`D` taking over as the sole draw key. `resolve_key` is a pure lib fn (tested
directly); the engine Space mapping is unit-tested; Space→`cursor_confirm`
is verified in-app via the driver. Do not weaken a test to make it pass.
