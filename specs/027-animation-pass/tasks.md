# Tasks: Animation pass — spec 027

> **Status**: Draft — pending sign-off
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
and only the orchestrator commits. **Experiment 2 is live**: every task below is
dispatched to `sdd-implementer-fable` (`claude-fable-5-1` at medium) per
`CLAUDE.md`, with the plain `sdd-implementer` (opus, high) as the fallback
dispatch when Fable's allowance runs out — needing the fallback is itself a
result, logged below; never infer the budget from a successful dispatch. The
`skeptical-reviewer` runs at opus (high) for phase reviews and the sweep; the
planner and the plan/tasks sign-off ran at the top tier (fable, per-call
override).

**Foundational phase: Phase 1.** T001 writes `motion.rs` — the struct, the
diff rule and the clocks that the board (T002) and the app (T003) both read, so
it carries `review: per-task`; T003 ends the phase with the review, the driver
walkthrough at both widths and a **pause for the person** (the constitution's
default: a pause after every phase unless the person says to run further).
Phase 2 is the Animations setting and the two documents, reviewed once as a
phase and followed by the second pause. No other task carries its own review.

**Plan §Open questions 1 (the Score's resting weight)** was ruled at planning:
the Score rests `Normal` from this spec on (spec Q7), as the plan reads it.
The ruling is flagged to the person in the spec-conformance summary; if they
overrule it before T001 starts, T001 drops `Elem::Score` and T002 leaves
`draw_side_header`'s Score line untouched — the orchestrator edits both task
lines before dispatching.

---

## Phase 1 — The transitions (foundational)

<!-- T001 is the pure module; T002 the board reading it; T003 the app feeding
it. After T003 the game plays with every transition, always on. -->

- [ ] **T001** — `src/lib.rs` + `src/motion.rs` (new): the beats and
  `BoardMotion`. `review: per-task`. In `lib.rs`, per plan §Design 1: add `pub
  mod motion;` and the three constants `ARRIVAL_BEAT_MS = 600`, `POPUP_BEAT_MS
  = 800`, `THINKING_STEP_MS = 300` with the plan's comment, beside
  `SELECTION_PULSE_MS`. In `motion.rs`, per plan §Design 2: the module doc;
  `pub enum Elem { Dealer(Player, usize), Played(Player, usize), Score(Player)
  }` with `fn side`; private `SideSnapshot` and `MotionSnapshot` (`of` sets
  `resolved = round_outcome.is_some() || GameOver`, `thinking =
  OpponentThinking`); `pub struct BoardMotion` deriving `Debug, Default,
  PartialEq, Eq` with `prev`, `arrivals: Vec<(Elem, Duration)>`, `popup_wait:
  Duration`, `thinking_for: Option<Duration>`; `observe(&mut self, gs, dt)`
  exactly as the plan's listing (clocks first, seed on `prev == None`, else
  `diff_side` per side plus the resolved and thinking edges); `diff_side` (a
  shrunk row discards the side's arrivals and starts nothing; otherwise one
  arrival per new index and a restarted `Score` if the total differs);
  `is_arriving`, `popup_due`, `thinking_suffix`; `pub fn thinking_suffix_at`.
  Tests (plan §Tests), in `motion.rs`'s tests module:
  `beats_are_named_constants_within_bounds`,
  `the_first_observation_starts_nothing`,
  `a_dealt_card_and_its_total_arrive_for_one_beat`,
  `a_play_arrives_with_the_total_and_a_draw_and_play_arrive_together`,
  `transitions_run_on_independent_clocks`,
  `a_cleared_row_discards_its_transitions_and_starts_none`,
  `the_popup_waits_one_beat_and_n_clears_the_wait`,
  `the_thinking_indicator_steps_through_the_pause_and_reverts` — each built
  on `GameState::new()` with rows pushed by hand (`PlayedCard { card, value }`)
  and phases set by hand, never sleeping. Do not run `cargo fmt`. (Copies:
  `banter.rs` for the module shape, `BanterSnapshot::of` and the snapshot
  tests; `app.rs`'s `SelectionPulse::tick` for the `Duration` accumulation
  and its `pulse_*` tests.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green,
  reported verbatim, with the eight named tests passing; `git diff --stat`
  shows only `src/lib.rs` and `src/motion.rs`; `git diff -- src/lib.rs` adds
  only the `mod` line and the three constants with their comment; the
  implementer's report quotes `observe` and `diff_side` verbatim. The
  orchestrator re-runs the verification command itself before committing
  (per-task review).*

- [ ] **T002** — `src/board.rs` + `src/app.rs` (one call site): the board
  reads the motion. Per plan §Design 3: `draw` gains `motion:
  Option<&BoardMotion>` before `frame` (import `crate::motion::{BoardMotion,
  Elem}`), and `App::draw`'s one `board_view.draw(…)` call passes `None` there
  so the crate compiles (T003 replaces it); add the free fn `arriving(motion,
  elem) -> bool`; `draw_side_header` takes `who: Player` and `motion` and
  draws the Score line `Strong` only while `Elem::Score(who)` is arriving,
  `Normal` otherwise (plan tension 1 — see the note above Phase 1);
  `draw_top_info` passes the sides; `draw_side` takes `who` and `motion` and
  sets `v.emphasis = Emphasis::Strong` on a dealer card while
  `Elem::Dealer(who, i)` is arriving and on a played card while
  `Elem::Played(who, j)` is (the `Double` weight stays); `status_lines` takes
  `motion` and appends `thinking_suffix()` to the base line when both are
  `Some`; the popup call is guarded by `motion.is_none_or(|m| m.popup_due())`;
  update `draw`'s doc. `status_message`, `over_twenty_alert`, the hand branch
  and the panel/stake tail are untouched. Tests (plan §Tests): `drawn_board`
  gains a `motion: Option<&BoardMotion>` parameter (existing callers pass
  `None`); add `arrivals_draw_strong_then_settle_on_both_layouts`,
  `the_popup_waits_and_the_deciding_card_shows_through`,
  `the_thinking_line_steps_muted_and_stays_clear_of_the_stake` — the motion
  in each built by `observe`-ing a seed state and then the changed one, cells
  read through `frame[x][y].emphasis` and `row_text`, slot corners from
  `card_slot`. Do not run `cargo fmt`. (Copies: `board.rs`'s own
  `the_compact_board_carries_the_stake_clear_of_the_alert` for reading rows
  and emphasis; `over_20_game` for building rows by hand.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the three new tests passing and `status_opponent_turn_is_muted`
  unchanged; `git diff --stat` shows only `src/board.rs` and `src/app.rs`;
  `git diff -- src/app.rs` is the one added `None` argument; the implementer's
  report quotes the new `draw_side_header` Score line and the popup guard
  verbatim.*

- [ ] **T003** — `src/app.rs`: the app feeds the motion. Per plan §Design 4:
  the `motion: BoardMotion` field with its doc, set in `new`; in `tick`, after
  `update_play_log()`, the `match &self.screen` that observes on `InGame` and
  resets otherwise, with the plan's comment; in `draw`'s `InGame` arm pass
  `self.settings.animations.then_some(&self.motion)` — **`animations` does not
  exist until T004**, so for this task pass `Some(&self.motion)` with a `//
  T004: gated by settings.animations` comment. No other `app.rs` code change.
  Tests (plan §Tests): `the_board_transitions_after_a_hit_and_settles_when_animations_are_off`
  (the Off step is **deferred to T004**; here: settled first frame; a dealer
  card pushed by hand with the phase set to `OpponentThinking`, then
  `tick(ZERO)` → slot 0 `Strong` and `Opponent's Turn .` on the status row;
  `round_outcome` + `AwaitingNextRound` set by hand, `tick(ZERO)` → no popup
  text; the outcome cleared, rows emptied and `PlayerTurn` set by hand,
  `tick(ZERO)` → no popup and no `Strong` in the grid; `handle_key(Esc)` then
  `tick(ZERO)` → `app.motion == BoardMotion::default()`) and
  `a_resumed_popup_draws_on_the_first_frame`. Both use `Profile::default()`,
  `Config { 89, 31 }`, `crate::frame::new_frame`, read the player's grid rect
  from `BoardLayout::new(config).player.grid` and the popup text by row.
  **Press no game key**: every key that reaches the engine ends in
  `save_game()` (the `game_changed` branch of `handle_key`), which writes the
  real save file — rows and phases are set by hand; `Esc` (the `Menu` arm)
  is the only key pressed. Do not run `cargo fmt`. (Copies: `the_first_match_popup_holds_the_match_and_swallows_play_keys`
  for entering a match and ticking;
  `the_game_over_frame_shows_the_settled_stake_on_both_layouts` for drawing an
  `App` into a frame and reading rows.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with both new tests passing; `git diff --stat` shows only
  `src/app.rs`; `git diff -- src/app.rs` outside `mod tests` touches only the
  field, `new`, the `tick` match and the `draw` argument; the implementer's
  report quotes the `tick` match verbatim.
  **PAUSE for the person** (after the Phase 1 review): the orchestrator drives
  the **89×31 and 139×31** Quick Play walkthrough in plan §Verification with
  the `run-kaazap` skill — real profile, settings and save backed up and
  checksummed first, restored and checksum-verified after — and reports in
  plain language what it saw: the new card and the score bold after a hit and
  settled about half a second later while the cursor keeps breathing; the
  opponent's dots stepping through its pause and gone when it acts; its cards
  bold when they land; the deciding card seen before the round popup, then the
  popup; `n` and `g` inside the beat acting at once; a settled board after `g`
  and on Continue; at 89 the stake on the band's upper row with the dots on the
  lower one. Then the person plays it and may ask for a beat tuning within the
  spec's bounds — logged as **T003a**, editing only the three constants in
  `src/lib.rs` (the bounds test guards).*

## Phase 2 — The Animations setting and the documents

<!-- T004 adds the row, gates the board and lands the README and brief lines;
the phase ends with the review and the second walkthrough. -->

- [ ] **T004** — `src/settings.rs` + `src/app.rs` + `src/audio.rs` (tests
  only) + `Readme.md` + `design/brief.md`: the Animations row. In
  `settings.rs`, per plan §Design 5: `animations: bool` with `#[serde(default
  = "default_animations")]` (true) and its doc, in `Default`; move
  `VOLUME_STEP` here from `app.rs`; `pub fn adjust(&mut self, row, right:
  bool)`; `SettingRow::Animations`; `const ROWS: [SettingRow; 3]`;
  `SettingsAction::{Up, Down, Left, Right, Back}` (rename `Quieter`/`Louder`);
  `move_up`/`move_down` by index in `ROWS`, clamped; `draw_overlay` with a
  `row_text(row, settings)` helper, three rows, content height 7, hint `↑/↓
  select  ·  ←/→ change  ·  Esc back`; update the struct doc's "Two for now"
  comment. In `app.rs`, per plan §Design 4: `handle_settings_input`'s
  `Left | Right` arm calls `self.settings.adjust(row, right)` then
  `set_settings`, `save`, `MenuMove` as today; remove `VOLUME_STEP`; `draw`'s
  `InGame` arm becomes `self.settings.animations.then_some(&self.motion)`
  (drop the T003 comment). In `audio.rs`, add `..Settings::default()` to the
  three `Settings { … }` test literals; in `settings.rs`'s existing tests add
  the field. Tests (plan §Tests), `settings.rs`:
  `settings_animations_default_on_missing_key_on_and_off_reads_off`,
  `adjust_toggles_animations_and_steps_volumes`,
  `settings_rows_move_over_three_rows_and_clamp`,
  `the_animations_row_reads_on_or_off_and_fits`; `app.rs`: add the Off step to
  `the_board_transitions_after_a_hit_and_settles_when_animations_are_off`
  (`app.settings.animations = false` → the slot `Normal` and the plain
  `Opponent's Turn`; `true` → `Strong` again — set the field directly, never
  through `handle_settings_input`, which writes the real settings file). In
  `Readme.md` ~line 19 and `design/brief.md`'s Motion section, the exact
  wording in plan §Design 6. Do not run `cargo fmt`. (Copies: `settings.rs`'s
  own `draw_overlay` and tests; `menu.rs`'s cursor index handling for the
  clamped row move.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the four new tests and the extended app test passing; `git
  diff --stat` shows only `src/settings.rs`, `src/app.rs`, `src/audio.rs`,
  `Readme.md`, `design/brief.md`; `git diff -- src/audio.rs` touches only
  `mod tests`; `grep -rn "Louder\|Quieter" src/` is empty; `grep -n
  "Animations" Readme.md` finds the clause; `grep -n "spec 027" design/brief.md`
  finds the sentence in the Motion section.
  **PAUSE for the person** (after the Phase 2 review): the orchestrator drives
  the Phase 2 walkthrough in plan §Verification with the `run-kaazap` skill —
  real profile, settings and save backed up and checksummed first, restored
  and checksum-verified after — and reports in plain language: the third row
  and its `On`/`Off`; the settings file on disk showing `"animations": false`
  after a toggle; a match with it Off showing no bold card or score, the popup
  on the resolving frame and the plain thinking line; back On, the Phase 1
  checks holding; a settings file with the key removed reading On. Then the
  person tries it.*

## Final phase — Spec close-out

- [ ] **T005** — Close-out. Draft
  `specs/027-animation-pass/closeout-main-docs.md` in spec 026's shape:
  **ROADMAP** — mark the animation pass shipped as spec 027 (`grep -n -i
  "animation" ROADMAP.md`, read and judge), and close spec 016's "light
  portrait animation" deferral with the repo's inline "closed by spec 027 (Q5
  A): portraits stay static" convention; **DECISIONS** — the rulings (Q1 a–e in,
  f out; Q2 A emphasis-only arrival; Q3 A the pulse keeps breathing and the
  brief's amendment; Q4 A the Animations row, missing key reads On; Q5 A
  portraits static; Q6 A board only; the person's ruling on plan §Open
  questions 1, the Score's resting weight), plan tension §2 (one struct
  observed in `tick`, `None` is the settled draw), §4 (a shrunk row is a clear),
  §7 (the `Left`/`Right` rename and `Settings::adjust`), and the beat values as
  shipped (after any T003a tuning) — to apply on `main` after the merge, never
  on the branch. Run `cargo test -q` three consecutive times and paste the
  tails. Mechanical checks (three-dot, since `main` may move): `git diff
  main...HEAD --stat` lists none of `src/game.rs`, `src/main.rs`,
  `src/frame.rs`, `src/render.rs`, `src/layout.rs`, `src/portrait.rs`,
  `src/card.rs`, `src/player.rs`, `src/save.rs`, `src/profile.rs`,
  `src/economy.rs`, `src/wager.rs`, `src/campaign.rs`, `src/campaign_map.rs`,
  `src/opponent.rs`, `tests/balance.rs`, `Cargo.toml`, `Cargo.lock`; `git diff
  main...HEAD -- src/audio.rs` touches only `mod tests`; `grep -n "VERSION"
  src/save.rs src/profile.rs` still reads 1 and 1; `grep -rn "Color" src/`
  shows nothing new against `main`; `cargo build --all-targets` warning count
  equals `main`'s; the three constants in `src/lib.rs` satisfy the bounds
  (the T001 test is green). Check off `spec.md`'s acceptance criteria with
  evidence (the T003 and T004 walkthrough reports are the evidence for the
  driver criteria). Request the pre-merge sweep; apply `closeout-main-docs.md`
  on `main` after the merge. Never chain a file edit, a branch switch and a
  commit in one shell command (spec 025's miss).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/027-animation-pass/{spec,plan,tasks}.md`, then
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
`skeptical-reviewer` pass (opus) at the end of each phase — after T003 and
after T004 — on a shell-assembled bundle (the phase diff, the task lines, plan
§Design and §Tests, the acceptance criteria), one review plus at most one
re-review; the Phase 1 review also checks the changed board and the Phase 2
review the changed Settings overlay against the constitution's *acted-on
element stands apart* rule and the modal padding rule (plan tension 8 states
the reading). **Pause cadence**: pause after **every** phase for the person,
per the constitution, unless the person says to run further — after Phase 1
once the review is done and the orchestrator's 89×31 + 139×31 walkthrough in
T003 is reported in plain language (the person sees the transitions before the
setting exists; a beat tuning becomes T003a), and after Phase 2 once the review
and the settings walkthrough in T004 are reported. Back up + checksum-restore
the real profile, settings and save before and after every driver session —
this spec's walkthroughs write the settings file on purpose. Repo-wide docs
(`ROADMAP.md`, `DECISIONS.md`) change only via `closeout-main-docs.md` on
`main` after the merge; `Readme.md` and `design/brief.md` ride in on the branch
(T004). Never run `cargo fmt`.

Plan §Open questions 1 (the Score's resting weight) is a product question: the
dispatcher takes it to the person at sign-off, and the orchestrator edits T001
and T002 before dispatching if the ruling differs from the plan's reading.

Model & effort: the session runs at the session tier (`claude-fable-5-1` at
medium, from `.claude/settings.json`); the planner and the sign-off ran at the
top tier (fable, high, per-call override). Implementation runs on
`sdd-implementer-fable` (experiment 2), the phase reviews and the sweep at
opus. A task that isn't routine goes to a decision review at the top tier,
never resolved by the session; a product question `spec.md` doesn't settle
goes to the person. Every session-ending pause ends with a continuation prompt
(spec directory, files to read, where to resume, involvement level, pause
cadence, any model switch) in its own fenced block.

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
| **Experiment 2 live** — 2026-09-18. Implementer `sdd-implementer-fable` (`claude-fable-5-1`, medium), fallback `sdd-implementer` (opus, high); reviewer opus (high); planner and sign-off at the top tier (fable, override). Compare against spec 026's log (the first full Experiment 2 spec: 6/6 first try, ~280K implementer total). Fable allowance reading at planning: _orchestrator to fill_. | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | fable (override) → fable | ~120K (planner's own estimate: ~85K read — bundle, board.rs, settings.rs, frame.rs, main.rs, banter.rs, lib.rs, the app.rs and game.rs sections, the brief's Motion section — the rest reasoning and the two files) | 1 | yes | — | drafted; 5 tasks in 3 phases, Phase 1 foundational, T001 `review: per-task`; one product question (the Score's resting weight, plan tension 1) for the person at sign-off; 4 design choices flagged for sign-off |
| Planning: draft (sdd-planner) — measured | fable (override) → fable | ~181K (measured return for the draft dispatch) | — | — | — | the measured figure for the draft row above; the planner's own estimate was ~120K |
