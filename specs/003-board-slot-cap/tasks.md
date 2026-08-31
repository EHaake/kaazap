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

- [x] **T001 — Card cap constant, `table_full`, auto-stand on fill**
  *Done. `MAX_TABLE_CARDS = 12` in lib.rs; `table_card_count`/`table_full`
  on `PlayerState`; the auto-stand for either full side inserted in
  `resolve_after_action` before the over-20 checks. Six `cap_` tests:
  hold at ≤20, opponent-side branch, over-20 bust, recovery-as-the-12th,
  refuse-a-13th, and both-full-resolves-by-totals. Mutation check run:
  neutralizing the auto-stand turns all six red; restored to green. 121
  tests pass (115 → +6), 0 warnings. Production diff vs main is the
  auto-stand insertion (game.rs) and the two queries (player.rs) only.*
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

- [x] **T002 — AI stands when its table is full**
  *Done. Early-return guard at the top of `decide_opponent_move`: a full
  opponent table stands. Two `ai_` tests — one unit (full table at 14
  with a +6 that would land on 20 returns `Stand`, not the winning play),
  one end-to-end through `play_opponent_turn` (a full opponent below
  threshold stands instead of drawing a 13th). Mutation-checked: both go
  red with the guard neutralized. 123 tests pass (+2), 0 warnings.*
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

- [x] **T003 — Frame: double border weight + ghost slot**
  *Done. `BorderWeight::Double` (`╔═╗ ║ ╚═╝`) added; the perimeter loop
  extracted to a private `draw_box_glyphs` so `draw_box` and the new
  `draw_ghost_slot` share it. `draw_ghost_slot` draws solid corners with
  dashed `╌`/`╎` edges, always `Muted`, clip-safe. Three tests (double
  glyphs + emphasis; ghost dashed/muted/interior-empty; ghost clips off-
  frame). 126 tests pass (+3), 0 warnings. Additive only — nothing draws
  them yet, so the definitive in-terminal font eyeball happens in T005
  where they first render on the board (spec-002 precedent); dim `Single`
  is the ready ghost fallback if the dashes read poorly there.*
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

- [x] **T004 — Board geometry functions + minimum height**
  *Done. `layout.rs` gains `grid_cols` / `board_grid_rows` /
  `board_block_height` (with private `side_card_span` / `board_grid_height`
  helpers and band constants HEADER/HAND/STATUS/GAP as the single source).
  At min width the grid packs HAND_SIZE (4) per row → 3 rows → block 30;
  monotonic non-increasing in width. `config::min_size` height now reads
  `board_block_height(min_cols)` (24 → 30); imports trimmed. Removed the
  now-dead `MIN_CARD_SIZE_HEIGHT` (obsoleted by the new formula) and its
  already-dead sibling `MIN_CARD_SIZE_WIDTH` from lib.rs. Three tests
  (reflow 4/row→3, 6/row→2, 12/row→1; monotonicity of rows and height;
  min-height == block-height == 30). Additive — `SideLayout` and
  `BoardLayout::new` unchanged, board still renders as before until T005.
  129 tests pass (+3), 0 warnings.*
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

- [x] **T005 — Single grid: `SideLayout` collapse, centered block, board render**
  *Done. `SideLayout` collapsed to `{ header, grid, hand }`;
  `BoardLayout::new` centers the fixed block (`top = (rows - block_h)/2`),
  laying every band from `top` off the shared constants; grid Rect sized to
  `grid_cols` slots. `board.rs` fills the one grid: dealer draws front
  (Single, bare value), played cards back (`MAX-1-j`, Double, signed),
  dim ghosts between — verified in-terminal at 180×48 (centered, 12-row
  margins, played `-4` Double at the back) and 89×30 (4/row × 3 rows, no
  dealer overflow — the 002-review artifact gone; status drops below).
  A `n/12` slot counter labels the grid and shows the cap. Divider now
  spans the block (header→hand). `cards_per_row` removed — the grid uses
  `grid_cols`, the same source `board_grid_rows` uses (single per-row
  truth). All four border weights + the dashed ghost render correctly,
  closing T003's deferred font check. 131 tests pass, 0 warnings.*
  *Flagged deviation: the full-table auto-stand feedback went to the
  header ("Stood — table full") rather than the status line the spec
  named — the turn passes too fast after the 12th card for a PlayerTurn
  status message to be seen, and per-side stood/bust state already lives
  in the header. Vetoable.*
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

- [x] **T005a — Fix the grid at 4×3; center the board in both axes**
  *(human-requested after seeing T005: the width-reflowed grid left a
  ragged partial row on wide terminals.)* The grid is now a constant
  `GRID_COLS`(4) × `GRID_ROWS`(3) — always full rows, columns aligned with
  the hand — and the whole board is a fixed `BOARD_WIDTH`×`BOARD_BLOCK_HEIGHT`
  block centered horizontally as well as vertically. This collapsed the
  width-driven machinery to constants: `grid_cols`/`board_grid_rows`/
  `board_grid_height`/`board_block_height` functions → `GRID_COLS`/
  `BOARD_BLOCK_HEIGHT`/`BOARD_WIDTH` consts; `config::min_size` is now just
  the fixed board size (89×30). The status_right/status_below conditional
  (and `status_fits_right`/`hand_cards_right`) is gone — status is always
  the band below the hand, since a fixed narrow board has no room beside
  it (this retires spec 002's T008a conditional, human-approved). Verified
  in-terminal at 89×30, 130×36, and 180×48: identical centered board, full
  4×3 grid, no ragged row. 128 tests pass, 0 warnings.*

- [x] **T005b — Lighter ghost slots; a little grid/hand separation**
  *(human-requested polish after T005a.)* Empty slots now draw four dim
  corner ticks (`┌ ┐ └ ┘`) instead of a full dashed box — much less busy,
  still marks the slot. Added a `HAND_GAP` (one blank row) between the
  grid and the hand so they no longer butt together; this nudges
  `BOARD_BLOCK_HEIGHT` and the minimum terminal height 30 → 31. Verified
  in-terminal. 128 tests pass, 0 warnings.*

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
