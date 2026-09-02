# Tasks: Control & Input Polish

Small, additive spec. Each task builds (`cargo build`) and tests
(`cargo test`) green before it's done, with actual output reported. One
commit per task referencing its ID; the draft PR (opened at T001) tracks
the diff. Human-approved to roll through T001–T002 without pausing between.

---

- [ ] **T001 — Emacs → arrow translation at the input boundary**
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

- [ ] **T002 — Space-to-advance + help text**
  Split `game_action_from_key`'s `'d' | ' '` so Space is phase-contextual:
  `NextRound` at `AwaitingNextRound`, `NextGame` at `GameOver`, else `Hit`
  (`'d'` stays `Hit`). Update the `?` game-help / how-to-play text to note
  Space advances and that `Ctrl+P/N/B/F` mirror the arrows.
  *Verify: `cargo test` green with new `game_action_from_key` tests (Space →
  NextRound / NextGame / Hit by phase; `'d'`, `'n'`, `'g'` unchanged);
  driver — Space advances a round and starts a new game, and still draws
  mid-turn.*

- [ ] **T003 — Verification & close-out**
  Full driver sweep of every acceptance box; confirm existing controls
  unchanged. Run the `skeptical-reviewer` (light — small surface). On the
  human's word: mark spec 006 Shipped in `ROADMAP.md`, mark the PR ready,
  and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file.
Both changes are additive — no existing key changes meaning. `resolve_key`
is a pure lib fn (tested directly); the Space change reuses the engine's
existing phase-aware key handling. Do not weaken a test to make it pass.
