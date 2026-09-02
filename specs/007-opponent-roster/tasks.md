# Tasks: Opponent Roster & Personalities

First spec of the campaign epic (subsystem A). Each task builds (`cargo
build`) and tests (`cargo test`) green before it's done, with actual output
reported. One commit per task referencing its ID; the draft PR (opened at
T001) tracks the diff. Foundational engine work is reviewed **per task**; the
UI is reviewed **per phase**. Do not weaken a test to make it pass.

Branch `007-opponent-roster` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [ ] **T001 — Opponent data model, roster & parameterized engine**
  New `src/opponent.rs`: `OpponentProfile` (`id`, `name`, `difficulty`,
  `blurb`, `stand_threshold`, `side_deck`; `Copy`), a `DEFAULT_OPPONENT`
  (threshold `STAND_THRESHOLD`, `DEFAULT_SIDE_DECK`, name "Opponent"), a
  5-opponent `OPPONENTS` roster (easy→hard, distinct thresholds + 10-card
  decks, original names/blurbs), and `opponent_by_id`. Wire it into the
  engine: `deal_hand(rng, deck: &[Card])` (`src/card.rs`); `GameState` gains
  `opponent_profile: OpponentProfile`; `new()` → `with_opponent(DEFAULT_
  OPPONENT)` + new `with_opponent`; `decide_opponent_move` reads
  `profile.stand_threshold`; `new_game` re-deals from the profile deck;
  `pub mod opponent;` in `src/lib.rs`. Open the draft PR.
  *Verify: `cargo build` no new warnings; `cargo test` green — new tests
  (parameterized AI: same board decides differently by threshold; `deal_hand`
  draws from the passed deck; roster integrity — deck len ≥ HAND_SIZE, unique
  ids, non-empty names; `with_opponent` seeds name/profile; `new()` unchanged
  baseline) plus every existing AI test still passing.*

- [ ] **T002 — Opponent-select screen + Start Game flow**
  New `src/opponent_select.rs` (`OpponentSelectState` over `OPPONENTS`,
  input handler → `Pick`/`Back`, `draw` mirroring `menu.rs` with the pulse
  marker + a hint line); `Screen::OpponentSelect` in `src/screen.rs`;
  `src/lib.rs` `pub mod opponent_select;`. In `src/app.rs`: `Start Game` →
  `OpponentSelect` (via `ConfirmNewGame`-Yes when a save exists, else
  directly); replace `start_new_game` with `start_match(profile)`; add the
  input/`?`/draw match arms; `Back`/Esc → menu; reuse the menu nav/select SFX.
  *Verify: `cargo test` green with `OpponentSelectState` navigation/event
  tests; driver — Start Game shows the roster list, arrows/`w`·`s`/`Ctrl+P`·
  `Ctrl+N` navigate, Enter starts a match against the chosen (named) opponent,
  Esc returns to the menu.*

- [ ] **T003 — Save/resume carries the opponent**
  `SavedGame` gains `opponent_id: String` (`#[serde(default =
  "default_opponent_id")]`); `to_saved` writes `profile.id`; `from_saved`
  resolves via `opponent_by_id(..).unwrap_or(DEFAULT_OPPONENT)`. No
  `SAVE_VERSION` bump.
  *Verify: `cargo test` green — save round-trip preserves `opponent_id`, and a
  field-less (pre-spec) JSON loads as the default opponent; driver — start a
  match vs a specific opponent, quit, Continue resumes against that same
  named opponent.*

- [ ] **T004 — Verification & close-out**
  Full driver sweep of every acceptance box (select flow, distinct play by
  threshold, per-opponent deck, Esc-back, emacs nav, resume). Run the
  `skeptical-reviewer`. Update the README (opponent selection is new
  user-facing behavior) and `ROADMAP.md` (annotate the Campaign epic — A
  shipped). On the human's word: mark the PR ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled; README/ROADMAP updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The
engine decision function and phase machine are reused unchanged — only their
inputs move from globals to an `OpponentProfile` carried on `GameState`.
`DEFAULT_OPPONENT` preserves the exact current behavior so existing AI tests
stay green. The select screen mirrors `menu.rs`; emacs nav comes free via
`resolve_key`. Save adds one `serde(default)` field — no version bump, old
saves resume against the default opponent.
