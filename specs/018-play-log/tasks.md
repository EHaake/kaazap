# Tasks: Play log — spec 018

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
Under the model policy, the orchestrator re-runs build + tests itself after an
implementer returns, and only the orchestrator commits.

**Foundational phase:** Phase 1 (T001–T003 — the `play_log.rs` capture/summarize/render engine
the whole feature rests on) is foundational. Under the constitution's review cadence the default
is a **per-phase** `skeptical-reviewer` pass. Two tasks carry `review: per-task`: **T001**,
because a mistake in the delta-diff shape (the one-card invariant, the ordering, the reset
signals) or the `Move`/`PlayLogSnapshot` types is inherited by T002, T003, T005, and T007; and
**T005**, the twin-call-site capture wiring — the exact spec-failure mode (missing the opponent's
timer-driven moves) and the documented subtlety this spec turns on. Every other task — T002, T003,
T004, T006, T007, and the close-out — is reviewed at phase end.

---

## Phase 1 — Play-log engine (foundational)

<!-- Foundational: the capture diff, the round summary, and the render formatting the overlay and
the app wiring both depend on. T001 gets a per-task skeptical-reviewer pass. -->

- [x] **T001 (foundational, review: per-task)** — Create `src/play_log.rs` (add to `src/lib.rs`).
  Define `Move { Draw{side,value,total}, Play{side,card:PlayedCard,total}, Stand{side,total},
  Bust{side,total} }`; `PlayLogSnapshot` (eight per-side lens/flags + `game_over` +
  `outcome_present`) + `PlayLogSnapshot::of(&GameState)`; the pure `moves_since(prev, gs) ->
  Vec<Move>` (delta reconstruction in player-then-opponent, draw→stand→bust order, plan §Design 1
  / tensions §2/§5), `match_restarted(prev, curr)` and `round_reset(prev, gs)` (tension §7); and
  `PlayLog { outcomes, moves, opponent_name, prev }` with `reset(&mut self, opponent_name: &str)`
  and `observe(&mut self, gs)` (seed silently on `prev = None`; on a rematch clear both lists; on a
  round reset clear `moves`; else append `moves_since`). Round-outcome capture is T002 — but so the
  `outcomes: Vec<RoundSummary>` field compiles, **declare the small `Resolution` and `RoundSummary`
  data types here in T001** (their `summarize_round` producer lands in T002); leave the `outcomes`
  push as a TODO stub for now (`observe` compiles, outcomes stays empty). (Copies the
  snapshot/diff pattern of `banter.rs` `BanterSnapshot`/`banter_event` + `App::update_banter`
  seeding.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green (reported verbatim) —
  new tests: `moves_since` yields exactly the one move (right side/value/card/total) for a dealer
  draw, a fixed-value play, a `±` and a tiebreaker committed at each sign, a flip play (identity +
  post-flip total), a stand, and a bust; a one-diff draw-fills-stands-busts yields draw→stand→bust
  in order; a player flip that busts a standing opponent yields player-play then opponent-bust; a
  no-change diff yields nothing; each move's `total` equals the acting side's `score()` (the
  one-card invariant); `observe` from `prev = None` emits nothing; a completed round then an
  empty-rows state clears `moves`; a `game_over` true→false clears both lists.*

- [x] **T002** — Round-outcome summarization in `src/play_log.rs`. The `Resolution { Bust(Player),
  BothBust, FilledTable, Stand }` and `RoundSummary { outcome, player_total, opponent_total,
  resolution }` types were already declared in T001; add the pure `summarize_round(gs) ->
  RoundSummary` with the plan §6 precedence (both-bust → `BothBust`; one bust → `Bust(side)`; else
  full table → `FilledTable`; else `Stand`), totals from `player.score()`/`opponent.score()`. Wire
  the T001 stub: in `observe`'s else-branch, replace the `let _newly_resolved = …;` line with a real
  `if newly_resolved { self.outcomes.push(summarize_round(gs)); }` (and drop the now-unused
  underscore binding). (Copies T001's pure helpers + the `finalize_round` fact-reading in `game.rs`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: `summarize_round`
  classifies a lone player bust, a lone opponent bust, a both-bust tie, a full-table auto-stand
  with no bust, and a plain double-stand, each with the right outcome and both totals; end-to-end
  `observe` over a finished round appends exactly one `RoundSummary` and none while the round is
  live.*
  *T001-review carryover to fold in here (same file): add a direct `reset()` test (clears a
  populated log + stores `opponent_name`); add a test where both `match_restarted` and `round_reset`
  conditions hold at once, pinning that `match_restarted` wins; and replace the artificial flip
  fixture in `play_log.rs` test 9 with one that actually mutates an opponent card so the
  flip-induced opponent total is exercised rather than tautological.*

- [x] **T003** — Render the log to lines in `src/play_log.rs`. Add the pure
  `PlayLog::render_lines(&self, inner_height_budget: usize) -> Vec<String>`: title (line 0), a
  "Round outcomes" section (one line per `RoundSummary`: winner/tie, both totals, resolution), a
  blank, a "This round" section (one line per `Move`: side label, value/`PlayedCard::display_text()`,
  resulting total), with `Player::Player` → "You" and `Player::Opponent` → `opponent_name`; apply
  the §8 most-recent-that-fit trim to the move lines against the budget. (Copies the line-list
  shape `overlay.rs` `read_text_from_file` produces — line 0 = title, rest content.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: the title is first, both
  section headers appear in order, the player is "You" and the opponent is named, each move line
  carries side + value/card + total and each outcome line carries winner/totals/resolution
  (substring asserts); with a budget below the content size, headers + outcomes + the most-recent
  move lines that fit are kept and the oldest moves are dropped.*

## Phase 2 — Overlay plumbing + in-match wiring

<!-- Integration: expose a dynamic-content overlay draw path (first dynamic overlay), then capture
live, toggle, and draw. T005 (twin-call capture) carries a per-task review; the rest at phase end.
Phase ends with the person's play-and-read attestation (T007). -->

- [x] **T004** — Content-driven overlay draw in `src/overlay.rs`. Extract the body of the private
  `draw_overlay` into `pub fn draw_text_overlay(config: Config, content: &[String], frame: &mut
  Frame)` (measure → `OverlayLayout::new` → `clear_rect` → `draw_box` → first-line-centered /
  rest-left `draw_text_in`) and make `Overlay::draw_overlay` a one-line delegation to it. No
  behavior change for the three static overlays. (Copies the existing `draw_overlay` internals.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — the existing `measure` tests still
  pass; report that `Overlay`'s public behavior is unchanged (static overlays still render via the
  extracted fn). Driver (optional): the `?` / How to Play overlays look identical.*

- [x] **T005 (review: per-task)** — Capture wiring on `App` in `src/app.rs`. Add the field
  `play_log: PlayLog` (grouped with `prev_banter`, init `PlayLog::default()`). Add `fn
  update_play_log(&mut self)` mirroring `update_banter` (early-return off `Screen::InGame`, else
  `self.play_log.observe(game_state)`) and call it right after `update_banter` at **both** sites:
  `handle_key` (`app.rs:860`) and `tick` (`app.rs:1136`). Add `self.play_log.reset(<opponent
  name>)` at `start_match` (`app.rs:512`, name from the opponent before it is moved) and at
  `Continue`/resume (`app.rs:1027`, `game.opponent.name` — read the name **before** `game` is moved
  into the `Box` on the next line, `app.rs:1028`). No overlay/toggle yet — capture only.
  (Copies the `prev_banter` seeding + twin-call `update_banter` pattern exactly.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green (capture logic is covered by the
  Phase 1 tests; the wiring is driver-verified at T007). Report that both call sites are present
  (`handle_key` + `tick`) and that no `game.rs`/`player.rs`/`card.rs`/`save.rs` file was touched.*

- [x] **T006** — Toggle + routing in `src/app.rs`. Add the unit-like `Modal::PlayLog` variant to
  the `Modal` enum (`app.rs:253`). In `handle_key`: when `Modal::PlayLog` is open, `L` or `Esc`
  closes it (play `Sfx::MenuBack`, as Help does); when no modal is open and the screen is
  `Screen::InGame`, `L` opens `Modal::PlayLog`. Leave the `m`-mute check ahead of modal routing and
  the lowercase-`l` sign-minus binding untouched. (Copies the `Modal::Help` open/close routing at
  `app.rs:628`/`app.rs:644`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green. Driver: in a match, `L` opens an
  (as-yet-borderless/stub or, if T007 already landed, full) overlay and `L`/`Esc` close it; `l`
  during a sign choice still commits the minus sign; `m` still mutes under the overlay.*

- [x] **T007** — Draw the play log + attest. In `src/app.rs`, add a `Some(Modal::PlayLog)` arm to
  the modal-draw match (`app.rs:1160`) that, on `Screen::InGame`, builds
  `self.play_log.render_lines(<inner-height budget from self.config>)` and calls
  `overlay::draw_text_overlay(self.config, &lines, frame)`. No resize arm is needed (content is
  rebuilt each draw from `self.config`; note this in the diff). **Derive the `inner_height_budget`
  passed to `render_lines` from the same `OverlayLayout` the draw uses** (or otherwise ensure the
  app-side budget agrees with `draw_text_overlay`'s real inner height), so the §8 most-recent-trim
  and the box clamp drop the same lines; confirm agreement in the driver check. (Copies the `Some(Modal::Help(..))
  => overlay.draw(frame)` draw arm + T004's `draw_text_overlay`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green. Driver (back up + checksum-restore
  the real profile/saves first): in Quick Play and a Campaign match — `L` opens a centered,
  bordered, monochrome overlay and `L`/`Esc` close it; it lists this round's moves in order (both
  sides, every kind: dealer draws, fixed/`±`/tiebreaker/flip plays with the resolved sign, stands,
  busts) each with the resulting total; an opponent move made on the think timer while the overlay
  is up appears live without reopening; the round-outcomes section shows each completed round's
  winner/tie, both totals, and how it resolved; a new round empties the moves while outcomes
  persist; a rematch clears both; a resumed mid-match save opens with an empty log and logs from
  resume on; the opponent hand card only appears once played; nothing renders in color; the overlay
  claims no left margin. Snapshot at 139×31 and wider. **PAUSE for the person** (phase attestation):
  confirm the log reads clearly, updates live, and is monochrome.*
  *Phase-1-review carryover for the draw author to decide (empty-log rendering): at match start the
  log has no outcomes and no moves, so `render_lines` currently emits both section headers with
  nothing beneath. Decide whether the overlay should suppress an empty section or show a placeholder
  line ("No moves yet") — a small cosmetic call; confirm it reads acceptably in the driver check.*

## Final phase — Spec close-out

- [ ] **T008** — Docs, driver, sweep. `DECISIONS.md`: the play log is transient (never saved),
  lives on `App` as a `PlayLog`, captured by the twin-call per-tick delta diff (not engine hooks);
  the round-resolution precedence (bust > filled-table > stand); the first *dynamic* overlay and
  the `draw_text_overlay` extraction; `Modal::PlayLog` capturing input while the game keeps ticking
  (no pause). `ROADMAP.md`: mark the play log shipped; drop from future. Check off `spec.md`
  acceptance criteria with evidence. Request the pre-merge whole-spec sweep.
  *T005-review carryover to fold in here (plan.md doc fix): plan §7 and §3-app say the resume site
  reads `game.opponent.name`, but `GameState.opponent` (a `PlayerState`) has no such field usable
  here — the name lives on `game.opponent_profile.name` (a `&'static str`), which is what the code
  correctly uses. Correct `game.opponent.name` → `game.opponent_profile.name` in plan.md §7 and §3
  so the plan doesn't teach the next reader a wrong field.*
  *Phase-1-review carryover cleanups to fold in here (src/play_log.rs, all non-blocking): (a) delete
  the now-stale `#[allow(dead_code)]` on `PlayLog.opponent_name` (`render_lines` reads it, so it no
  longer warns); (b) add `assert!(!log.outcomes.is_empty())` to the round-reset test so "outcomes
  persist across a new round" is pinned, not just inspected; (c) add a `render_lines(0)` (budget
  below `fixed_count`) no-panic assertion to close the zero-budget gap.*
  *Phase-2-review carryover to fold in here (all non-blocking): (d) tighten
  `render_lines_does_not_panic_at_tiny_budgets` to also assert both placeholders survive at budget 4
  (they live in the fixed part, so a regression that moved them out would otherwise pass silently);
  (e) record in DECISIONS.md the one place plan §8's "fixed section is never trimmed" yields to the
  box: on a terminal short enough that `fixed_count > inner_height_budget` (~rows ≤ 10 with several
  outcomes — below what `Config::from_terminal` admits, so unreachable in practice) the fixed lines
  clip off the bottom rather than trim. Known non-issue; no code change. Note T007 already folded in
  the (b)/(c) equivalents via its empty-log test — reconcile rather than duplicate.*
  *Verify: `cargo build --all-targets` / `cargo test -q` green, reported verbatim; legible snapshots
  at 139×31 and wider showing the open log over the board; the sweep confirms no
  `game.rs`/`player.rs`/`card.rs`/`save.rs` change and that the log never enters the save format;
  `ROADMAP.md` no longer lists this as future; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/018-play-log/{spec,plan,tasks}.md`, then implement from the first
unchecked task. Involvement level is **product owner**. Dispatch each routine task to the
`sdd-implementer` per the model policy; verify by running the build and tests yourself, then
commit. **The two tasks marked `review: per-task` (T001, T005):** run the `skeptical-reviewer`
after each, scoped to that task's diff, its plan section, and its acceptance criteria (a
shell-assembled bundle), one review plus at most one re-review, and re-run the verification command
yourself before committing. **Every other task (T002, T003, T004, T006, T007, T008):** review at
phase end. Pause for the person after each phase, and whenever something unexpected bears on spec
adherence. The **T007 phase pause** is the spec's play-and-read attestation — the person confirms
the log opens on `L`, updates live under the overlay, reads clearly for both sides across a round
and across rounds, and is monochrome.

Model & effort: the session runs at the orchestrator tier (`claude-opus-4-8`), medium effort; the
planner and the `skeptical-reviewer` sign-off/reviews run at the top tier (`fable`) per the
per-call override — **except while the top-tier budget is short**, when they drop to `opus` per the
documented fallback (this plan/tasks pair was drafted under that fallback). Implementers run at
`opus`. Clear at every phase boundary and at spec end. Every session-ending pause ends with a
continuation prompt (spec directory, files to read, where to resume, involvement level, pause
cadence, any model switch) in its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs and reviewer
invocations), any escape-hatch miss (a task the orchestrator had to redo, and why). Compare the
spec total against a previous spec of similar size before treating the policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| Planning: draft (sdd-planner) | opus (documented fallback — top-tier budget short) | ~143.5K | drafted |
| plan + tasks sign-off (skeptical-reviewer) | opus (documented fallback — top-tier budget short) | ~62.5K | signed off; 3 non-blocking notes folded into T001/T005/T007 text |
| T001 impl (sdd-implementer) | opus | ~70.9K | done; build clean, 303 tests pass (13 new) |
| T001 review (skeptical-reviewer, per-task) | opus | ~55.1K | signed off; 3 non-blocking test-quality notes → folded into T002 |
| T002 impl (sdd-implementer) | opus | ~43.2K | done; build clean, 312 tests pass; T001-review carryovers folded in |
| T003 impl (sdd-implementer) | opus | ~42.5K | done; build clean, 319 tests pass (7 new); stale #[allow(dead_code)] on opponent_name to clean |
| Phase 1 review (skeptical-reviewer) | opus | ~57.3K | passed; no blocking; 3 non-blocking cleanups → folded into T008 |
| T004 impl (sdd-implementer) | opus | ~19.2K | done; build clean, 319 tests pass; draw_text_overlay extracted, draw_border/add_content inlined+removed |
| T005 impl (sdd-implementer) | opus | ~46.2K | done; build clean, 319 tests pass; twin call sites + both reset sites wired; only src/app.rs touched |
| T005 review (skeptical-reviewer, per-task) | fable (ran; but budget NOT actually recovered — orchestrator misjudged a successful dispatch as recovery; Erik ruled 2026-09-09 the opus fallback stays) | ~30.9K | signed off, no blocking; 2 non-blocking notes (1 → T008 plan-text fix; 1 already in T007 checklist) |
| T006 impl (sdd-implementer) | opus | ~22.6K | done; build clean, 319 tests pass; Modal::PlayLog variant + L/Esc routing; temp `Some(Modal::PlayLog) => {}` draw stub (T007 replaces) |
| T007 impl (sdd-implementer) | opus | ~62.2K | done; build clean, 321 tests pass (2 new); draw arm w/ max-clamped OverlayLayout budget; empty-section placeholders folded into render_lines (Phase-1 carryover). ATTESTATION PENDING (Erik) |
| Phase 2 review (skeptical-reviewer) | fable (ran; budget NOT recovered — same orchestrator misjudgment as the T005 row; opus fallback stays per Erik 2026-09-09; T008 sweep runs on opus fallback) | ~44.1K | signed off, no blocking; 2 actionable non-blocking notes → T008 (tighten tiny-budget test; record the short-terminal clip as a known non-issue), 2 cosmetic → attestation |
| T008 close-out (orchestrator) | | | |
| Pre-merge whole-spec sweep (skeptical-reviewer) | | | |
