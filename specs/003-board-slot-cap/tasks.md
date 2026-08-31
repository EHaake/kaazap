# Tasks: Board Slot Cap & Vertical Centering

**Status**: Draft (in review)
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task is completable and
testable on its own; resume at the first unchecked task.

<!-- WARNING, worth leaving in: once implementation starts, this file
gets written by more than one party. Never edit it from a stale copy;
prefer small targeted edits over regenerating it wholesale. -->

Per the constitution: every task ends with `cargo build` and `cargo test`,
both green, actual output reported. *Verify:* lines are the task-specific
checks on top of that baseline. One commit per task, referencing the task
ID; push after each commit — the draft PR (open it when T001 lands) tracks
the running diff.

**Review cadence**: Phase 1 (engine rules) and Phase 2 (board geometry,
layout, rendering) are foundational — they change resolution behavior,
reshape the board, and raise the minimum terminal size — so stop for
review after **every task**. Phase 3 (acceptance) is a sweep — stop after
the **phase**.

**Scope guard** (inverse of spec 002's tripwire — this spec *does* edit
the engine): `game.rs`/`player.rs` changes are limited to the card-count
cap, the auto-stand, and the AI full-table guard described in `plan.md`.
A task that finds itself changing scoring, flips, tiebreaker, sign-choice,
or next-round/new-game logic has gone off-plan and stops for review. The
number `12` lives in exactly one place — `MAX_TABLE_CARDS` — and every
other use references it.

---

## Phase 1 — Engine rules (`lib.rs`, `player.rs`, `game.rs`)

Review: after every task.

- [ ] **T001 — Card cap constant, `table_full`, auto-stand on fill**
  `lib.rs`: `pub const MAX_TABLE_CARDS: usize = 12;`. `player.rs`:
  `table_card_count()` (dealer_row + played_row lengths) and
  `table_full()`. `game.rs`: in `resolve_after_action`, after scores are
  computed and *before* the existing over-20 bust checks, set
  `stood = true` for either side whose `table_full()` — so a full side
  ≤ 20 holds and a full side > 20 busts through the paths already there.
  No new phase, no new flag.
  Tests (`cap_`): reaching 12 cards at ≤ 20 auto-stands (stood set, not
  bust, turn passes / both-stood → `RoundEnd`); the 12th card at > 20
  busts (`bust`, `RoundEnd`, that side loses on `update`); an 11-card
  side over 20 plays a recovery card as its 12th and holds (not bust);
  through the production loop a side never exceeds `MAX_TABLE_CARDS`.
  Mutation check (constitution): confirm a `cap_` test goes red if the
  auto-stand line is removed (i.e. a 13th card would otherwise land) —
  note it in the task so the guard is known to be able to fail.
  *Verify: `cargo test cap_` green and full `cargo test` green (report
  count); `git diff main -- src/game.rs` shows only the auto-stand
  insertion, `src/player.rs` only the two new queries.*

- [ ] **T002 — AI stands when its table is full**
  `game.rs`: guard at the top of `decide_opponent_move` —
  `if self.opponent.table_full() { return OpponentAction::Stand; }` — so
  the AI never *chooses* an impossible hit (the T001 auto-stand is the
  real enforcement; this makes the AI rule explicit and unit-testable).
  Tests (`ai_`): `decide_opponent_move` returns `Stand` on a full
  opponent table even sitting below the stand threshold with an otherwise
  playable card; end-to-end through `play_opponent_turn` a full opponent
  stands rather than drawing a 13th.
  *Verify: `cargo test ai_` green; full `cargo test` green; `git diff
  main -- src/game.rs` limited to this guard plus T001's insertion.*

## Phase 2 — Board geometry, layout, rendering (`frame.rs`, `layout.rs`, `config.rs`, `board.rs`)

Review: after every task.

- [ ] **T003 — Frame: double border weight + ghost slot**
  `frame.rs`: add `BorderWeight::Double`; teach `draw_box` the
  `╔ ═ ╗ ║ ╚ ╝` glyph set (dealer stays `Single`, the hand's selected
  card stays `Heavy` — three distinct weights, no collision). Add
  `draw_ghost_slot(frame, rect)` drawing an empty slot as a `Muted`
  dashed outline (`┌╌╌┐ ╎ └╌╌┘`), clip-safe. Additive only — nothing
  consumes them yet, so visuals are unchanged this task.
  Tests (`box_`/`ghost_`): `draw_box` with `Double` writes the double
  corners/edges; `draw_ghost_slot` writes a `Muted` outline within a
  small in-memory frame and clips rather than panicking when the rect
  runs off-frame.
  *Verify: `cargo test box_ ghost_` green; real-terminal eyeball of a
  `╔═╗` box and the dashed ghost, reported — if the dashed glyphs read
  badly, fall back to a dim `Single` outline (one call to change) and
  record it here.*

- [ ] **T004 — Board geometry functions + minimum height**
  `layout.rs`: `board_grid_rows(cols)` = `MAX_TABLE_CARDS.div_ceil(cards
  per row over one side's span at `cols`)`, and `board_block_height(cols)`
  = `grid_rows*CARD_HEIGHT + (grid_rows-1) + 13` (header + gaps + hand +
  status, per plan). `config.rs`: `min_size()`'s height term becomes
  `board_block_height(min_cols)` (≈ 30), replacing `CARD_HEIGHT *
  MIN_CARD_SIZE_HEIGHT + V_PAD`. Pure additions — `SideLayout` is
  untouched here, so `board.rs` still compiles against the old fields.
  Tests (`layout_`/`config_`): `board_grid_rows` is 3 at minimum width,
  non-increasing as width grows, and ≥ 1; `board_block_height` is
  monotonic non-increasing; `min_size()` height equals
  `board_block_height(min_cols)`; `Config::fits` accepts the new minimum
  and rejects one row short.
  *Verify: `cargo test layout_ config_` green; full suite green (the
  existing `config_fits_*` test still passes against the new minimum).*

- [ ] **T005 — Single grid: `SideLayout` collapse, centered block, board render**
  The coupled change (struct + its consumer, together so it compiles).
  `layout.rs`: `SideLayout { header, grid, hand }` (drop `dealer` /
  `played`); `BoardLayout::new` centers the block —
  `top = (rows - board_block_height(cols)) / 2` (saturating) — laying
  header, grid, hand, and the status bands from `top` down, mirrored
  across the divider; `grid` sized to `board_grid_rows(cols)`;
  `status_right` / `status_below` repositioned relative to the block
  (002's conditional placement preserved). `board.rs`: fill the one grid
  from the two vectors — dealer draws front (grid index `i`, `Single`,
  bare value), played cards back (grid index `MAX_TABLE_CARDS-1-j`,
  `Double`, signed value), `draw_ghost_slot` for the middle indices; the
  old "Dealer"/"Played" zone labels retire; the status line announces the
  full-table auto-stand (a brief message in the over-20 alert's style, so
  the player sees why their turn ended). Update the `layout_` tests to
  the new `SideLayout` (header/grid/hand in-bounds and vertically
  disjoint; block vertically centered — top margin ≈ bottom; grid holds
  the `0..MAX_TABLE_CARDS` slot positions with no overlap; filled + ghost
  indices reflow with width: 4/row → 3 rows vs a width giving 6/row →
  2 rows).
  *Verify: `cargo test layout_` green; full `cargo test` green; in-app at
  a wide/tall size (e.g. 180×48) — board is one centered block, dealer
  `Single` / played `Double` / empty dim ghost, no top/bottom spread; and
  at ~89×30 a many-low-draw round fills the grid with no card wrapping
  into a lower band (the 002-review artifact, gone); driver snapshot
  confirms `╔`/`═` on played cards and grid rows never exceeded.*

## Phase 3 — Acceptance

Review: after the phase.

- [ ] **T006 — Acceptance sweep + close-out**
  Walk every `spec.md` acceptance box with evidence: `cargo build` (no
  new warnings vs `main`), full `cargo test` (report count), a live game
  at a tall terminal showing the centered block, the dealer-overflow
  artifact gone near minimum height (driver, many-low-draw round), and a
  regression pass that arrows-only and keys-only play still work.
  Docs / close-out: update `assets/how_to_play_text.txt` to state the
  12-card table cap and that filling it stands you (rides the branch);
  move the "Board slot cap & vertical centering" item from ROADMAP
  Backlog to Shipped as spec 003 (ROADMAP commits straight to `main` per
  convention); refresh the README only if the raised minimum or the rule
  is worth a user-facing line. Offer the `skeptical-reviewer` pass over
  the branch (spec 002's caught a real root cause); sub-letter any real
  findings (`T006a`, …) rather than renumbering. Mark the PR ready only
  on the human's word.
  *Verify: every `spec.md` acceptance box checked with evidence; build /
  test output reported verbatim; reviewer findings (if run) resolved or
  explicitly ruled; ROADMAP/how-to-play updated.*

---

## Handoff note

Read `CLAUDE.md`, `design/brief.md`, then this spec's `spec.md`,
`plan.md`, and this file. Implement in order from T001. **Stop for review
after every task in Phases 1–2; after the phase for Phase 3.** Push the
branch and open a **draft PR** when T001 lands, so the running diff is
reviewable commit-by-commit. One commit per task referencing the task ID.
Sub-letter (`T00Xa`) any genuinely new scope and flag it — never silently
absorb it, never renumber. The engine scope guard above applies to every
task: cap + auto-stand + AI guard only, and `12` lives once as
`MAX_TABLE_CARDS`.
