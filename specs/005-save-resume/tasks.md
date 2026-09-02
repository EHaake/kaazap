# Tasks: Save & Resume (mid-match)

Ordered, independently verifiable steps for spec 005. Each task builds
(`cargo build`) and tests (`cargo test`) green before it's done, with the
actual output reported — per `CLAUDE.md`. One commit per task referencing
its ID; push after each (the draft PR, opened at T001, tracks the diff).

**Review cadence:** the save format and the engine-touching serde work are
foundational — **stop for review after every task in Phases 1–3**; Phase 4
is the close-out. The engine scope guard holds throughout: `game.rs` /
`player.rs` / `card.rs` gain **only** serde derives (no logic change), and
saving observes `GameState` without reaching into rendering or audio.

---

## Phase 1 — Persistence core (`save.rs`, engine derives)

- [ ] **T001 — Serde derives on the leaf match types + branch/PR setup**
  Create the `005-save-resume` branch (already done for the spec/plan) and
  open the draft PR. Add `#[derive(Serialize, Deserialize)]` to the pure
  data types a match is made of: `Card`, `FlipKind`, `PlayedCard`
  (`card.rs`); `Player`, `PlayerState` (`player.rs`); `RoundOutcome`
  (`game.rs`). Import `serde::{Serialize, Deserialize}` where needed. No
  logic changes; `GamePhase` and `GameState` are deliberately left underived
  (the T002 projection stands in for them).
  *Verify: `cargo build` no new warnings; `cargo test` still green; a scratch
  round-trip (`serde_json::to_string` → `from_str`) of a `PlayerState` with a
  populated `dealer_row`/`played_row`/`hand` compiles and reproduces it
  (folded into T002's tests, not left as scratch).*

- [ ] **T002 — `save.rs`: the `SavedGame` projection + load/save/clear**
  New `save.rs` (registered in `lib.rs`). Define `SAVE_VERSION`, `SavedGame`
  (`version`, `player`, `opponent`, `phase: SavedPhase`, `round_outcome`),
  and `SavedPhase` (mirrors `GamePhase` minus the `Instant`, no `GameOver`).
  Conversions: `to_saved(&GameState) -> Option<SavedGame>` (`None` on
  `GameOver` = "clear"; `OpponentThinking { .. }` → `SavedPhase::
  OpponentThinking`), and `SavedGame -> GameState` (re-arm
  `OpponentThinking { until: Instant::now() + OPPONENT_THINKING_TIME_MS }`).
  Persistence mirroring `settings.rs`: path
  `ProjectDirs::from("","","kaazap").data_dir().join("saves").join("savegame.json")`;
  `save(&GameState)`, `load() -> Option<GameState>`, `exists() -> bool`
  (= `load().is_some()`), `clear()`; a filesystem-free `from_json` core for
  the fallback/version tests. All best-effort — never panic.
  *Verify: `cargo test` green with new tests — a round-trip preserving match
  position across representative states (mid–player-turn with a partial table
  + pending `AwaitingSignChoice`; mid–opponent sequence; awaiting-next-round
  with round wins); `OpponentThinking` round-trips to a valid re-armed
  `OpponentThinking` without panicking; malformed/empty/wrong-`version` JSON
  → `from_json` is `None`; `to_saved` of a `GameOver` game is `None`.*

## Phase 2 — Menu (`menu.rs`)

- [ ] **T003 — `Continue` menu item, shown only when a save exists**
  Add `MenuItem::Continue` (+ `Display` "Continue"). Turn `MenuState` into a
  computed list: `items: Vec<MenuItem>` (Continue prepended iff a save
  exists), `selected: usize` indexing it; `new(has_save: bool)` builds the
  list; `move_selection` / `draw` / `selected()` read `self.items`. Update
  the existing `menu_selection_moves_over_all_items_and_wraps` test for the
  index model.
  *Verify: `cargo test` green; new tests — `MenuState::new(true)` includes
  `Continue` at the top and `new(false)` omits it, and navigation wraps over
  4 items vs 3 respectively; driver snapshot of the menu unchanged when no
  save exists (Continue absent).*

## Phase 3 — App wiring (`app.rs`)

- [ ] **T004 — Consolidate the modals into one `Modal` enum**
  Pure refactor (no behavior change): replace `overlay: Option<Overlay>` and
  `settings_panel: Option<SettingsState>` with `modal: Option<Modal>` where
  `Modal { Help(Overlay), Settings(SettingsState) }`. Route input in
  `handle_key` and paint in `draw` off the single field; keep the global `m`
  mute, the `MenuBack` close cue, and the settings behavior identical.
  *Verify: `cargo build` no new warnings; `cargo test` green; driver — How to
  Play and Settings still open/close/adjust exactly as before (selection
  preserved, back cue), `?` help unchanged.*

- [ ] **T005 — Save/clear triggers + `has_save` plumbing**
  `App` gains `has_save: bool` (seeded from `save::exists()` in `new`) and a
  `start_menu()` helper that builds `Screen::StartMenu` with the current
  `has_save`; use it in `new` and the in-game Esc-to-menu path. Save the
  match: on new-match creation (StartGame / in-game `g`), after a player
  action reaches `InGame` in `handle_key`, and after `tick`'s `update()`
  **iff the phase discriminant changed** — with `to_saved` returning `None`
  on `GameOver` clearing the file. Every write sets `has_save = true`, a
  clear sets it `false`.
  *Verify: `cargo build`/`cargo test` green; driver — start a match, quit,
  and confirm `saves/savegame.json` exists; play a full match to a win and
  confirm the file is gone; idle ticks don't rewrite it (mtime steady while
  waiting on input).*

- [ ] **T006 — Continue → resume, and the discard confirmation**
  `MenuItem::Continue` → `save::load()` → `Screen::InGame` (restoring the
  cursor). `MenuItem::StartGame` → if `has_save`, open `Modal::ConfirmNewGame`
  (the third `Modal` variant), else start a new match. The confirm renders a
  two-choice **Yes / No** box (reusing `OverlayLayout` / `draw_box`),
  defaulting to **No**, navigated with the cursor vocabulary; Enter selects,
  Esc cancels. Yes → discard + start fresh; No/Esc → back to the menu, save
  intact.
  *Verify: `cargo test` green; driver — Continue restores the exact board
  (round, wins, tables, hands, turn); Start Game with a save prompts, No
  keeps the save, Yes replaces it; a hand-corrupted `savegame.json` shows no
  Continue and doesn't crash.*

## Phase 4 — Verification & close-out

- [ ] **T007 — Acceptance sweep, review, and merge**
  *Acceptance sweep done: every `spec.md` box checked with evidence — 154
  tests / 0 warnings; driver runs for quit→resume, no-save menu, discard
  confirm (No keeps / Yes overwrites), a full match to `Opp won: 3` →
  GameOver clearing the save, and a corrupt file → no Continue / no crash;
  version-mismatch covered by the T002 unit test. README status line notes
  mid-match save/resume. Engine diff is serde/`Clone` derives only.
  Remaining: the skeptical-reviewer pass, then ROADMAP Shipped + PR ready +
  merge on the human's word.*
  Walk every `spec.md` acceptance box with evidence (build/test output;
  driver runs for each flow — quit/resume, no-save menu, discard confirm,
  save-cleared-on-finish, corrupt-file fallback, version-mismatch discard).
  Update the README if user-facing setup/behavior warrants (a Continue /
  save-location note). Run the `skeptical-reviewer` over the branch;
  sub-letter any real findings. On the human's word: mark spec 005 Shipped in
  `ROADMAP.md` (drop/annotate the Save/resume backlog line), mark the PR
  ready, and merge.
  *Verify: all `spec.md` boxes checked with evidence; `cargo test` green and
  reported verbatim; no new warnings; engine files show only serde-derive
  changes; reviewer findings resolved or ruled.*

---

## Handoff note

Read `CLAUDE.md` (Settings is an overlay, not a `Screen`, post spec 004),
then this spec's `spec.md`, `plan.md`, and this file. Implement from T001 in
order. The two findings the plan rests on: **no RNG/deck state to persist**
(already-drawn cards live in `PlayerState`; future draws are fresh), and the
**only non-serde field is the `Instant` in `GamePhase::OpponentThinking`**
(dropped on save, re-armed on load). Sub-letter (`T00Xa`) any genuinely new
scope and flag it. Do not weaken a test to make it pass — flag it instead.
