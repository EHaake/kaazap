# Tasks: Compact layout below 139 columns — spec 026

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-17; re-review after B1/B2 and N1–N7; N8 applied by the orchestrator, N9 reported to the person)
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task should be completable (and
testable) on its own. If a session ends mid-list, resume by finding the first
unchecked task — don't re-verify everything above it unless something looks off.

<!-- WARNING: once implementation starts, this file gets written by more than one
party — whoever's steering adds scope and reshuffles tasks; the implementing
session (the orchestrator — never the sdd-implementer subagent) checks boxes and
adds findings. Never edit this file from a stale copy. Prefer small, targeted
edits over regenerating it wholesale — a full replacement silently discards
whatever the other party added since your copy was taken. -->

Per the constitution: every implementation task ends with an actual build and,
where tests exist for what changed, an actual test run — reported, not summarized.
Under the model policy, the orchestrator verifies after an implementer returns,
and only the orchestrator commits. **Experiment 2 is live from this spec**: every
task below is dispatched to `sdd-implementer-fable` (`claude-fable-5-1` at
medium) per `CLAUDE.md`, with the plain `sdd-implementer` (opus, high) as the
fallback dispatch when Fable's allowance runs out — needing the fallback is
itself a result, logged below. The `skeptical-reviewer` runs at opus (high) for
phase reviews and the sweep; the planner and the plan/tasks sign-off ran at the
top tier (fable, per-call override).

**Foundational phase: Phase 1.** T001 changes the constant, the minimum and the
`BoardLayout` shape that every later test and screen inherits, so it carries
`review: per-task`; T002 completes the compact board and ends the phase with
the review, the 89×31 driver walkthrough and a **pause for the person** (the
constitution's default: a pause after every phase unless the person says to
run further). Phase 2 is the fit sweep and the resize proof, reviewed once as a
phase and followed by the second pause. No other task carries its own review.

---

## Phase 1 — The compact board (foundational)

<!-- T001 is the geometry and the minimum; T002 the panel-less draw with the
stake. After T002 the game plays at 89 columns. -->

- [x] **T001** — `src/layout.rs` + `src/config.rs` + `src/board.rs` (compile
  fix only) + `src/opponent_select.rs` (tests only): the threshold and the
  optional panel. `review: per-task`. In `layout.rs`, per plan §Design 1:
  rename `IN_MATCH_MIN_WIDTH` → `WIDE_LAYOUT_MIN_WIDTH` (same value and
  derivation, the plan's doc comment); `BoardLayout.opponent_panel` becomes
  `Option<Rect>`, `Some` iff `cols >= WIDE_LAYOUT_MIN_WIDTH`, computed with the
  existing `panel_x0`/`PANEL_H_INMATCH` arithmetic — no other line of `new`
  changes; add `PartialEq, Eq` to `Rect`'s derive; change the "139×31" figures
  in the `CampaignMapLayout` clamp comment, the briefcase `const _` message and
  `CELL_H` doc to "89×31". In `config.rs`, per plan §Design 2: `min_size()`
  returns `(BOARD_WIDTH, BOARD_BLOCK_HEIGHT)` with the plan's doc; add
  `#[cfg(test)] pub fn fit_sizes() -> [Config; 2]` (89×31, 139×31) inside
  `impl Config`; in `from_terminal`'s bail change `Minimum size required:
  {}x{}` to `{} x {}` so the startup error quotes `89 x 31` like the too-small
  screen (AC 1), nothing else in the message. In `board.rs`, only what compiles: the panel tail of `draw` becomes `if let
  Some(panel) = self.layout.opponent_panel { draw_presence_panel(…);
  draw_presence_extras(…) }` (T002 finishes it). Tests (plan §Tests): in
  `config.rs` rename `config_min_size_is_board_plus_panel_margins` →
  `config_min_size_is_the_board_block` (pins `(89, 31)` and `fit_sizes()`); in
  `layout.rs` add `the_wide_threshold_is_the_board_plus_panel_margins`
  (`WIDE_LAYOUT_MIN_WIDTH == 139`), replace
  `board_and_panel_fit_the_minimum_terminal` with
  `the_panel_appears_at_the_threshold_and_not_below` (139 → `Some(Rect::new(117,
  138, 0, 19))`, in bounds, right of `opponent.hand.x1`, `y1 <= 30`; 138 and 89
  → `None`; 180×48 → `Some` in bounds), add
  `board_rects_are_identical_across_the_threshold_apart_from_centering`
  (`[89, 138, 139, 180]` × 31 rows, every board rect and `divider_x` shifted by
  `-left` equals the 89 case), and loop `Config::fit_sizes()` in
  `campaign_map_layout_fits_the_minimum_terminal`,
  `the_campaign_map_is_legible_at_the_minimum_terminal`,
  `briefcase_fits_the_minimum_terminal` and
  `overlay_layout_pads_content_symmetrically` (assertions unchanged); in
  `opponent_select.rs` loop `fit_sizes()` in
  `preview_panel_is_on_frame_and_clear_of_the_list_at_the_minimum` and
  `the_full_roster_and_footer_fit_the_minimum_terminal` (drop the
  `IN_MATCH_MIN_WIDTH` imports and the "139×31" comment figures). Do not run
  `cargo fmt`. (Copies: `layout.rs`'s own `BoardLayout::new` and its tests
  module — `cfg`, `in_bounds`, `vertically_disjoint` helpers; `config.rs`'s
  existing test shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green,
  reported verbatim — in particular `the_campaign_map_is_legible_at_the_minimum_terminal`
  and `campaign_map_layout_fits_the_minimum_terminal` pass at 89 (the plan's
  hand-derived map figures are the expectation; a failure here is a finding for
  the orchestrator, not something to fix by moving a planet);
  `the_prompt_fits_the_minimum_terminal_unclamped` (wager.rs, untouched) and
  `turn_hints_fit_the_status_band` still pass now that they measure 89; `git
  diff --stat` shows only `src/layout.rs`, `src/config.rs`, `src/board.rs`,
  `src/opponent_select.rs`; `grep -rn IN_MATCH_MIN_WIDTH src/` is empty; `grep
  -n "required: {} x {}" src/config.rs` finds the bail; the implementer's
  report quotes the new `opponent_panel` computation and `min_size` verbatim.
  The orchestrator re-runs the verification command itself before committing
  (per-task review).*

- [x] **T002** — `src/portrait.rs` + `src/board.rs`: the compact board's stake
  line. In `portrait.rs`, per plan §Design 3: add `pub fn stake_line(stake:
  u32) -> String` (`Stake ◈ {stake}`), use it in `draw_presence_extras`, and
  reword `PANEL_W`'s "the game's minimum terminal width" to "the wide layout's
  threshold". In `board.rs`, per plan §Design 4: add `pub fn is_wide(&self) ->
  bool`; the tail of `draw` becomes the `match self.layout.opponent_panel`
  with the wide arm as today and the compact arm drawing `stake_line(stake)`
  `Align::Right`, `Emphasis::Strong` on `self.layout.status` row 0 only when
  `stake` is `Some` (import `stake_line`); update `draw`'s doc (banter only on
  the wide layout; the stake moves to the band on the compact one). Tests
  (plan §Tests), in `board.rs`'s tests module using `frame::new_frame`,
  `over_20_game()` and `HandCursor::default()`:
  `the_compact_board_carries_the_stake_clear_of_the_alert` (89×31, `!is_wide()`,
  stake `Some(999_999)`: row `status.y0` starts at `status.x0` with the alert
  text, ends at `status.x1` with `stake_line(999_999)`, the cells between are
  blank, the stake cells are `Emphasis::Strong`, row `status.y0 + 1` holds the
  prompt; then at `GamePhase::GameOver { winner: Player::Player }` the row
  still ends in the stake line), `quick_play_shows_no_stake_line` (stake `None`
  → no `Stake` on either status row at 89), and
  `the_wide_board_keeps_the_stake_in_the_panel` (139×31, `is_wide()`, same
  draw → no `Stake` on the status rows; row `panel.y0 + 18` contains
  `◈ 999999`). Do not run `cargo fmt`. (Copies: `portrait.rs`'s
  `a_staked_match_draws_the_stake_under_the_pips` for reading a frame row as a
  `String`; `board.rs`'s `popup_rect_is_sized_centered_and_in_bounds` for
  building a `BoardView` in a test.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the three named tests passing and every `portrait.rs` test
  unchanged; `git diff --stat` shows only `src/portrait.rs` and `src/board.rs`;
  `git diff -- src/portrait.rs` adds only `stake_line`, its call and the one doc
  line; the implementer's report quotes the compact arm verbatim.
  **PAUSE for the person** (after the Phase 1 review): the orchestrator drives
  the **89×31** walkthrough in plan §Verification with the `run-kaazap` skill —
  real profile, settings and save backed up and checksummed first, restored and
  checksum-verified after — and reports what it saw in plain language: every
  screen listed there on frame with nothing clipped (the deck builder's hint
  line whole and centered; the map's eight planets and labels clear of the
  rail; the select preview beside the list), a staked campaign match with
  `Stake ◈ N` at the right of the status band's upper row from the first frame
  through an over-20 alert, a round popup and the game-over popup (this is the
  App-level check that the stake is still passed at game over), a Quick Play
  match with no stake line, no panel and no banter line; starting in a
  terminal below 89×31 shows the bail's `89 x 31`. **The play log is checked
  at Phase 2, not here** — T003 widens its box, so at Phase 1 it is still the
  40-column box and would clip (sign-off N8). Long play-log round headers
  clipping is reported at Phase 2 as a pre-existing finding (plan tension 4).
  Then the person tries the compact board before Phase 2 starts.*

- [x] **T002a** — finding F1 from the Phase 1 walkthrough (2026-09-17): on a
  staked campaign match at 89×31 the `Stake ◈ 30` line was on the band from
  the first frame, through the over-20 alert and the round-outcome popup, but
  **absent at the game-over popup** (`YOU WIN THE GAME!`, row 29 blank). Spec
  AC 4 and §The compact board say the stake shows "from the first frame to the
  game-over popup"; plan §Tests notes the App half depends on when
  `stake_at_risk()` clears at settlement. Dispatched as a diagnosis bundle to
  the implementer: fix if routine and inside the presentation footprint, else
  return the diagnosis and options for a decision review. **Diagnosis**: the
  match settles on the tick that draws the game-over frame, and the wide panel
  has been blank there since spec 021. **Ruling A (the person, 2026-09-17,
  spec Q6)**: show the settled stake at game over on both layouts. Per plan
  §Design 4a: `App::stake_to_show()` returns `stake_at_risk()` or, at
  `GamePhase::GameOver`, the settled amount from `self.banner`; `App::draw`
  passes it to the board. `src/app.rs` only, plus an App-level test drawing
  a staked game at `GameOver` into 89×31 (band ends in `Stake ◈ N`) and
  139×31 (panel row holds `◈ N`).
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new test; `git diff --stat` shows only `src/app.rs`; the
  Phase 2 walkthrough's game-over snapshots show the stake at 89 and 139.*

## Phase 2 — Every screen at 89, and the resize

<!-- T003 makes every fit test measure both widths; T004 proves the resize
path, updates the README and ends the phase with the review and the two driver
walkthroughs. -->

- [ ] **T003** — `src/overlay.rs` + `src/records.rs` + `src/shop.rs` +
  `src/app.rs` + `src/board.rs` (tests): the fit sweep. In `overlay.rs`, per
  plan §Design 5: `SCROLL_MIN_W` 40 → 52 with the plan's doc; extract `fn
  scroll_box(config: Config) -> Rect` from `draw_scrollable_overlay` (the
  `box_w`/`box_h`/`x0`/`y0`/`outer` lines, unchanged arithmetic) and call it;
  loop `Config::fit_sizes()` in `help_texts_fit_the_minimum_terminal_unclamped`,
  `onboarding_texts_are_the_spec_text_and_fit` and
  `scrollable_overlay_pins_to_bottom_and_clamps_overscroll` (replacing its
  hard-coded 139); add `the_play_log_box_is_the_same_width_compact_and_wide`
  (`scroll_box` at 89×31 and 139×31 both 52 wide and `x1 < cols`; at 200×48
  wider than 52). In `records.rs`, per plan §Design 6: extract `fn
  content_size(bodies: &[Vec<String>], coll: &str) -> (usize, usize)` (move
  `FOOTER_MEASURE` beside it) and call it from `draw`; add
  `the_records_popup_is_never_capped_at_the_minimum_terminal` (a
  `LifetimeStats` with three-digit match counts for one opponent in both
  `Mode`s via `record_match` in a loop, `record_campaign_completion(999)`, a
  `RunStats` with a streak; `content_size` over the four `view_body`s and
  `collection_line(15, 15)`; for each `fit_sizes()` config `content_w + 10 <=
  cols - 8` and `content_h + 8 <= rows - 4`, with a comment naming `draw`'s
  clamps). Loop `Config::fit_sizes()` in `shop.rs`'s
  `the_full_list_fits_the_minimum_terminal`, `app.rs`'s
  `the_campaign_entry_panel_fits_the_minimum_terminal` and
  `both_notices_read_right_breathe_and_fit_the_minimum_terminal`, and
  `board.rs`'s `turn_hints_fit_the_status_band`; drop stale "139x31" comment
  figures in those tests. `wager.rs` is not touched (spec AC 3, plan tension
  3). If `the_full_list_fits_the_minimum_terminal` fails at 89 (the plan's
  Outfitter arithmetic is the expectation), that is a finding for the
  orchestrator to route to the person, not something to fix by re-laying-out
  the shop. Do not run `cargo fmt`. (Copies: `records.rs`'s
  `the_campaign_view_folds_in_the_first_clear_record` for the stats fixture;
  `overlay.rs`'s existing fit tests for the loop shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the two new tests passing; `git diff --stat` shows only
  `src/overlay.rs`, `src/records.rs`, `src/shop.rs`, `src/app.rs`,
  `src/board.rs`; `git diff -- src/wager.rs` is empty; `git diff --
  src/records.rs src/overlay.rs` changes no drawing call beyond the two
  extractions and the constant; `grep -rn "139" src/*.rs`, read line by line,
  shows no `139` where a *minimum* is described — threshold docs (`fit_sizes`,
  `is_wide`, `SCROLL_MIN_W`, `WIDE_LAYOUT_MIN_WIDTH`) and test literals are
  fine.*

- [ ] **T004** — `src/app.rs` (tests) + `Readme.md`: the resize proof and the
  README. In `app.rs`'s tests, per plan §Tests: add
  `a_resize_across_the_threshold_keeps_the_match_and_toggles_the_panel`
  (`App::new(139×31)`; `screen = InGame { Box::new(GameState::new()), a
  HandCursor after one move_right over the hand }`; `modal =
  Some(Modal::Help(Overlay::new(OverlayKind::GameHelp, wide)))`; record the
  cursor index and `player.hand`; `resize(138×31)` → `!is_too_small()`,
  `!app.board_view.is_wide()`, still `InGame` with the same index and hand,
  modal still `Help` of kind `GameHelp` (`matches!`); `app.draw` into a 138×31
  `new_frame` without panic; `resize(139×31)` → `is_wide()` and the same
  checks again) and `the_too_small_screen_quotes_the_new_minimum`
  (`draw_too_small` into a 40×10 frame; some row contains `Need at least 89 x
  31`). No `app.rs` code change. In `Readme.md` replace the **Terminal size**
  paragraph (~line 57) with plan §Design 8's wording. Do not run `cargo fmt`.
  (Copies: `app.rs`'s `the_first_match_popup_holds_the_match_and_swallows_play_keys`
  for entering a match in a test, and
  `resize_too_small_pauses_then_a_valid_resize_resumes`.)
  *Verify: `cargo build --all-targets` + `cargo test -q` green verbatim with
  both new tests passing; `git diff --stat` shows only `src/app.rs` and
  `Readme.md`; `git diff -- src/app.rs` touches only the `mod tests` block;
  `grep -n "139" Readme.md`, read line by line, shows no `139` where a minimum
  is described (the threshold sentence and any history line are fine).
  **PAUSE for the person** (after the Phase 2 review): the orchestrator drives
  the walkthroughs in plan §Verification with the `run-kaazap` skill at
  **89×31 and 139×31** — real profile, settings and save backed up and
  checksummed first, restored and checksum-verified after — and reports what it
  saw: at 89 the T002 walkthrough repeated (every screen on frame with nothing
  clipped, the deck builder's hint line whole and centered, the map clear of
  the rail, the select preview beside the list, the staked match's `Stake ◈ N`
  from the first frame through the game-over popup, Quick Play without it) plus
  the play log at its 52-column box; at 139 spec 016's panel with no band
  stake; a resize across 139 both ways keeping the match, cursor and open help;
  below 89×31 the too-small screen quoting `89 x 31`. Long play-log round
  headers clipping is reported as a pre-existing finding (plan tension 4). Then
  the person tries it.*

## Final phase — Spec close-out

- [ ] **T005** — Close-out. Draft
  `specs/026-compact-layout/closeout-main-docs.md` in spec 025's shape:
  **ROADMAP** — mark "A compact layout below 139 columns" (~line 645) shipped
  as spec 026, and annotate the entries that describe 139×31 as the minimum
  (`grep -n "139" ROADMAP.md`, read and judge — ~210, ~420, ~526) with the
  repo's inline "superseded by spec 026, which …" convention; **DECISIONS** —
  the five rulings (Q1 B drop portrait and banter but keep the stake; Q2 A
  width alone, live, no setting; Q3 B the preview and rail keep portraits; Q4 A
  with C's rule, the stake on the band's upper row, visibility for the whole
  match without growing the block; Q5 A the threshold stays 139), plan tension
  §1 (the panel is an `Option` chosen in `BoardLayout::new`), §3 (the rename
  and `fit_sizes`), §4 (the play-log floor), and a supersession line for the
  "grew the minimum terminal, 89×31 → 139×31" decision (~line 406) — to apply
  on `main` after the merge, never on the branch. Run `cargo test -q` three
  consecutive times and paste the tails. Mechanical checks (three-dot, since
  `main` may move): `git diff main...HEAD --stat` lists none of `src/main.rs`,
  `src/card.rs`, `src/game.rs`, `src/player.rs`, `src/save.rs`,
  `src/profile.rs`, `src/economy.rs`, `src/wager.rs`, `src/campaign.rs`,
  `src/campaign_map.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`; `git
  diff main...HEAD -- src/portrait.rs` adds only `stake_line`, its call and one
  doc line; `grep -n "VERSION" src/save.rs src/profile.rs` still reads 1 and 1;
  `cargo build --all-targets` warning count equals `main`'s. Check off
  `spec.md`'s acceptance criteria with evidence (the T004 walkthrough report is
  the evidence for the driver criteria). Request the pre-merge sweep; apply
  `closeout-main-docs.md` on `main` after the merge. Never chain a file edit, a
  branch switch and a commit in one shell command (spec 025's miss).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/026-compact-layout/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product
owner**. **Experiment 2 is live**: dispatch each task to
`sdd-implementer-fable` (Fable 5.1, medium) on a shell-assembled task bundle
(task line, plan section, acceptance criteria, files, the pattern file to
copy); fall back to `sdd-implementer` (opus, high) only when Fable's allowance
runs out, and log the fallback as a result — never infer the budget from a
successful dispatch. Verify from the implementer's verbatim output, except
**T001 (`review: per-task`)**: the orchestrator re-runs the verification
command itself, then dispatches a `skeptical-reviewer` (opus) on T001's diff
alone before T002 starts. **Foundational phase: Phase 1.** One
`skeptical-reviewer` pass (opus) at the end of each phase — after T002 and
after T004 — on a shell-assembled bundle (the phase diff, the task lines, plan
§Design and §Tests, the acceptance criteria), one review plus at most one
re-review. **Pause cadence**: pause after **every** phase for the person, per
the constitution, unless the person says to run further — after Phase 1 once
the review is done and the orchestrator's 89×31 walkthrough in T002 is reported
in plain language (the person sees a real compact board before Phase 2), and
after Phase 2 once the review and the 89×31 + 139×31 + resize walkthroughs in
T004 are reported. Back up +
checksum-restore the real profile, settings and save before and after every
driver session. Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`) change only via
`closeout-main-docs.md` on `main` after the merge; `Readme.md` rides in on the
branch (T004). Never run `cargo fmt`.

Model & effort: the session runs at the session tier (`claude-fable-5-1` at
medium, from `.claude/settings.json`); the planner and the sign-off ran at the
top tier (fable, high, per-call override). Implementation runs on
`sdd-implementer-fable` (experiment 2), the phase reviews and the sweep at
opus. A task that isn't routine goes to a decision review at the top tier,
never resolved by the session; a product question `spec.md` doesn't settle
goes to the person. If the T001 map tests fail at 89, that is such a question
(planet positions are campaign data), not a fix. Every session-ending pause
ends with a continuation prompt (spec directory, files to read, where to
resume, involvement level, pause cadence, any model switch) in its own fenced
block.

## Tier log (this spec, under the model policy)

<!-- Experiment 2's measures: dispatches per task (1 = done in one dispatch;
count re-dispatches and fallbacks), first-try rate (verification green on the
first return, no re-dispatch), and blocking findings per phase review. One row
per implementer run and per reviewer invocation; a summary row per phase the
orchestrator fills at that phase's review. Record token usage from each
subagent return and any escape-hatch miss (a task the orchestrator had to redo,
and why). -->

| Task / invocation | Tier (dispatched → ran) | Tokens | Dispatches | First try | Blocking findings | Outcome / miss reason |
|---|---|---|---|---|---|---|
| **Experiment 2 live** — 2026-09-17, Fable allowance reading **96 %** at planning. Implementer `sdd-implementer-fable` (`claude-fable-5-1`, medium), fallback `sdd-implementer` (opus, high); reviewer opus (high); planner and sign-off at the top tier (fable, override). Specs 023–025 ran the fallback and are **not** experiment data; compare against specs 021–022. | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | fable (override) → fable | ~150K (planner's own estimate; ~120K read, the rest reasoning and the two files) | 1 | yes | — | drafted; 5 tasks in 3 phases, Phase 1 foundational, T001 `review: per-task`; no product fork; 4 design choices flagged for sign-off |
| plan + tasks sign-off (skeptical-reviewer) | fable (override) → fable | ~87K (measured return; reviewer's own ~70K in / 4K out) | 1 | — | 2 (B1 wager test vs AC 3 — the spec corrected; B2 no Phase 1 pause) + 7 notes N1–N7 | B1 fixed in spec.md by the orchestrator (AC 3 reworded; reported to the person as N9 asks); B2 and N1–N7 sent to the planner |
| Planning: draft (sdd-planner) — measured | fable (override) → fable | ~181K (measured return for the draft dispatch) | — | — | — | the measured figure for the draft row above; the planner's own estimate was ~150K |
| Planning: sign-off notes (sdd-planner, same context) | fable (override) → fable | ~25K (planner's own estimate of the delta) | 1 | yes | — | B1 (tension 3 / open question 3 now cite the corrected AC 3), B2 (Phase 1 ends with a PAUSE and the 89×31 walkthrough in T002; handoff cadence), N1 (bail `{} x {}` in T001), N2 (grep checks reworded), N3 (App-level half of AC 4 → walkthroughs), N4 (Outfitter finding-not-fix in T003), N5 (deck-builder hint line in the walkthroughs), N6 (`impl Config` block), N7 (tension 4 wording) applied; both files still Draft. Measured cumulative return for the planner agent after this pass: ~198K (draft + notes) |
| sign-off re-review (skeptical-reviewer, same context) | fable (override) → fable | ~116K cumulative measured for the reviewer agent (~29K delta this pass; reviewer's own ~25K in / 1.5K out) | 1 | — | 0 (B1, B2 resolved) + 2 notes | **signed off**; N8 (the Phase 1 walkthrough listed the play log, which T003 widens at Phase 2) applied by the orchestrator in T002's PAUSE; N9 (tell the person AC 3 was reworded at sign-off) goes in the session's closing report |
| T001 impl | sdd-implementer-fable → fable | ~67K measured return (implementer's own ~60K in / 12K out) | 1 | yes | — | done; one flagged deviation: `WIDE_LAYOUT_MIN_WIDTH` imported inside `fit_sizes` (a top-level import warned as unused, its only use being `cfg(test)`); the plan's hand-derived map figures held at 89 |
| T001 per-task review (skeptical-reviewer) | opus → opus | ~48K measured return (reviewer's own ~18K in / 2K out) | 1 | — | 0 + 7 notes | signed off; notes carried forward: the stale "always visible" panel comment in `board.rs` (folded into T002's bundle), `cfg` loop variable shadowing the `cfg()` helper in `overlay_layout_pads_content_symmetrically` (to the sweep) |
| T002 impl | sdd-implementer-fable → fable | ~55K measured return (implementer's own ~45K in / 3K out) | 1 | yes | — | done; no behaviour deviation; also replaced the stale "always visible" panel comment carried from the T001 review; the three named tests pass, 419 total |
| Phase 1 review (skeptical-reviewer) | opus → opus | ~60K measured return (reviewer's own ~52K in / 1.5K out) | 1 | — | 0 + 5 notes | signed off; notes: watch the band's upper row at round end / opponent stand in the walkthrough (done — only the alert ever lands there; BUSTED/Stood go in the header); the game-over test proves the outcome, not the popup extent; `cfg` shadowing still to the sweep |
| T002a diagnosis (sdd-implementer-fable) | sdd-implementer-fable → fable | ~33K measured return (implementer's own ~31K) | 1 | — | — | diagnosed, nothing changed: settlement zeroes the escrow on the tick that produces the game-over frame, and the wide panel has been blank at game over since spec 021 too; three options returned; product question to the person at the Phase 1 pause |
| T002a impl (ruling A) | sdd-implementer-fable → fable | ~39K measured return (implementer's own ~37K) | 1 | yes | — | done; `App::stake_to_show()` with one extra guard (campaign pointer set, so a Quick Play game over never shows a stale banner); 420 tests |
| Phase 1 driver walkthrough (orchestrator, 89×31) | — | — | — | — | — | every listed screen on frame, nothing clipped; the bail quotes `89 x 31`; **finding F1**: at the game-over popup the band's `Stake ◈ N` is gone (App-level half of AC 4) → T002a |
| **Phase 1 summary** | — | — | dispatches/task: 1.0 (T001, T002, T002a; plus one diagnosis dispatch) | first-try rate: 3/3 | blocking: 0 (per-task) + 0 (phase) | Phase 1 done on Fable in one dispatch each; walkthrough finding F1 (stake absent on the game-over frame) ruled A by the person and fixed as T002a; its game-over frames are checked in the Phase 2 walkthrough |
| T003 impl | sdd-implementer-fable → | | | | | |
| T004 impl | sdd-implementer-fable → | | | | | |
| Phase 2 review (skeptical-reviewer) | opus → | | | | | |
| Phase 2 driver walkthroughs (orchestrator, 89×31 and 139×31) | — | — | — | — | — | |
| **Phase 2 summary** | — | — | dispatches/task: | first-try rate: | blocking: | |
| T005 close-out | sdd-implementer-fable → | | | | | |
| Pre-merge sweep (skeptical-reviewer) | opus → | | | | | |
| **Final summary** | — | — | dispatches/task: | first-try rate: | blocking: | |
