# Tasks: Opponent banter — spec 017

> **Status:** Approved — implementation in progress.
**Implements**: plan.md in this directory
**Person approval:** granted (product-owner spec-conformance + technical-lead sign-off, after skeptical-reviewer sign-off with notes applied).

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

**Foundational phases:** Phase 1 (T001 — the banter event-diff + line-selection engine every
line-set and the wiring depend on) and Phase 3 (T003 — the panel-extras render primitive) are
foundational. Under the constitution's review cadence the default is a **per-phase**
`skeptical-reviewer` pass; only **T001** carries `review: per-task`, because a mistake in the
event precedence, the no-repeat selection, or the `BanterSet`/`banter_event` shape is inherited
by a dozen line-sets and the whole tick wiring. Every other task — T002, T003, T004, T005, and
the close-out — is reviewed at phase end.

---

## Phase 1 — Banter engine (foundational)

<!-- Foundational: the event diff, the line-selection rule, and the BanterSet/BanterEvent shape
everything else rests on. Per-task skeptical-reviewer pass. -->

- [x] **T001 (foundational, review: per-task)** — Create `src/banter.rs` (add to `src/lib.rs`).
  Define `BanterEvent` (MatchStart, RoundWin, RoundLoss, RoundTie, OpponentBust, PlayerBust,
  MatchWin, MatchLoss — opponent's POV); `BanterSnapshot { o_bust, p_bust, outcome, game_over,
  opp_won_game }` + `of(&GameState)` mirroring `AudioSnapshot::of` (`audio.rs:268`);
  `banter_event(prev, curr) -> Option<BanterEvent>` with precedence match-end > bust >
  round-outcome (plan §Design 1 / tension §5); `BanterSet` (eight `&'static [&'static str]`
  fields) + `lines_for`; `pick(lines, last, rng) -> &'static str` (no back-to-back repeat, plan
  §4); `banter_for(id) -> &'static BanterSet` returning the roster set or `GENERIC`. Author the
  **`GENERIC`** neutral set (all eight classes, terse, characterless) plus a placeholder roster
  set for **one** id (e.g. `greeb`) so `banter_for` compiles and tests run — T002 adds the rest.
  Add `pub const BANTER_MAX_WIDTH: usize = PANEL_W - 2;` and `const ROUND_PIPS: usize = 3;` to
  `src/portrait.rs` (geometry the fit test and T003 both use). (Copies the pattern of
  `audio.rs`'s snapshot/diff and the `opponent.rs` const-table style.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green (reported verbatim)
  — new tests: `banter_event` returns the right event and honors precedence for game-over
  (over a co-firing bust/outcome), each bust, each outcome, and `None` on a stale/no-change diff;
  `pick` never returns `last` for a ≥2-line pool and returns the sole line for a 1-line pool;
  every class of `GENERIC` (and of `banter_for("default")` / an unknown id) is non-empty and
  `GENERIC`'s five repeatable classes (`round_win`/`round_loss`/`round_tie`/`opponent_bust`/
  `player_bust`) each have `len() >= 2` (plan §4 — so a repeatable event can't repeat a sole line
  back-to-back); every `GENERIC` line `chars().count() <= BANTER_MAX_WIDTH`.*

## Phase 2 — The voices (content)

- [x] **T002** — Author the 10 roster `BanterSet`s in `src/banter.rs` (`greeb`, `dax`, `vessa`,
  `nima`, `toran`, `brakka`, `rix`, `kesh`, `magistrate`, `sovereign`), each a **distinct** voice
  consistent with the opponent's `difficulty`/`blurb` (`opponent.rs`) — the rookie eager and
  rattled, the veteran dry, the boss cold — covering all eight event classes with a few variants
  each, terse, `<= BANTER_MAX_WIDTH`. Repoint `banter_for` to the per-id sets. (Copies T001's
  `BanterSet` const pattern.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: every line of every class
  of every roster set (and `GENERIC`) `chars().count() <= BANTER_MAX_WIDTH`; every class of every
  set is non-empty and every set's five repeatable classes (`round_win`/`round_loss`/`round_tie`/
  `opponent_bust`/`player_bust`) have `len() >= 2` (plan §4); the 10 roster sets are pairwise
  distinct and each differs from `GENERIC` (concatenated-lines proxy); **within every class of
  every set, the lines are distinct** (no duplicate variants — carried from the T001 review:
  guards `pick` against a pool of all-equal lines, which could otherwise loop). Driver/person:
  read the sets — 10 distinct, in-character voices, none blank, none another opponent's voice.*

## Phase 3 — Panel-extras render path (foundational, contained)

<!-- Foundational: the in-match-only drawer for the banter line + pips, kept separate from the
shared draw_presence_panel so the two preview callers are untouched. Reviewed at phase end. -->

- [ ] **T003 (foundational)** — In `src/portrait.rs`, add `draw_presence_extras(frame, panel:
  Rect, banter: Option<&str>, opponent_rounds_won: usize)`: compute the panel interior as
  `draw_presence_panel` does, draw `banter` (if `Some`) centered on interior row 14 via
  `draw_text_in` (clip-safe), and the pip string on interior row 15 — `opponent_rounds_won`
  filled glyphs (`Emphasis::Strong`) + `ROUND_PIPS - rounds_won` empty (`Emphasis::Muted`),
  centered (plan §Design 2, tensions §6/§7). **Leave `draw_presence_panel` unchanged.** (Copies
  the clip-safe drawer style already in `portrait.rs`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: `draw_presence_extras` is
  clip-safe off-frame (no panic, empty frame ok); for `rounds_won` in `0..=3` the pip row carries
  exactly `ROUND_PIPS` markers with `rounds_won` filled; a `BANTER_MAX_WIDTH`-long line lands
  fully inside the interior and one longer is clipped (no cell written on or past the panel
  border). `draw_presence_panel`'s signature/callers are unchanged.*

## Phase 4 — Wire banter onto the in-match board

- [ ] **T004** — Banter state + per-tick update in `src/app.rs`. Add fields `banter:
  Option<&'static str>` and `prev_banter: Option<BanterSnapshot>` (grouped with `prev_audio`).
  Add `fn update_banter(&mut self)` mirroring `emit_audio_cues` (`app.rs:511`): snapshot the
  current `GameState`; on a `banter_event` from `prev_banter`, set `self.banter =
  Some(pick(lines_for(banter_for(opponent id), ev), self.banter, &mut rand::rng()))`; store the
  new snapshot. Call it right after each `emit_audio_cues` (`app.rs:798`, `app.rs:1068`). Seed at
  the two match-start sites beside the existing `prev_audio = None`: fresh (`app.rs:493/502`) →
  `prev_banter = None`, `banter = Some(pick(banter_for(opponent.id).match_start, None, rng))`;
  resume (`app.rs:965/972`) → `prev_banter = None`, `banter = None`. (Copies the `prev_audio` /
  `emit_audio_cues` pattern.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green (the behavior is exercised by the
  T001 pure-logic tests; the wiring itself is driver-verified at T006). Report that no
  `game.rs`/`save.rs`/AI file was touched.*
- [ ] **T005** — Thread the line into the board. In `src/board.rs`, add a `banter: Option<&str>`
  parameter to `BoardView::draw` and, after the existing `draw_presence_panel` call
  (`board.rs:267`), call `draw_presence_extras(frame, self.layout.opponent_panel, banter,
  state.opponent.rounds_won)`. In `src/app.rs`, pass `self.banter` at the `board_view.draw`
  call-site (`app.rs:1081`). Nothing else in the board changes.
  *Verify: `cargo build --all-targets` / `cargo test -q` green. Driver: start a Quick Play match
  (default opponent → generic neutral line) and a campaign match — a greeting shows at match
  start; a voice line replaces it on each round win/loss/tie, each bust, and the final blow, and
  persists between events; pips fill as the opponent wins rounds; the board's own prompts are
  undisturbed; opponent-select and the campaign map still show name + portrait only. Snapshot at
  139×31 and wider. **PAUSE for the person** (phase attestation): confirm the voices read as
  distinct and in character, and that everything is monochrome. **Phase 2 review taste notes to
  weigh here:** (1) Toran's "Over you go." vs Vessa's "Down you go." (both player_bust) are a
  faint rhyme back-to-back; (2) Dax calling the player "kid" when Dax is himself the roster's
  cocky kid; (3) Sovereign's round_loss ("Amusing."/"A trifle.") and match_loss ("Enjoy it.
  Briefly.") sit in one adjacent flat register — by design, but worth an ear.*

## Final phase — Spec close-out

- [ ] **T006** — Docs, driver, sweep. `DECISIONS.md`: banter is transient (never saved), lives on
  `App`; the event precedence (match-end > bust > round-outcome); opponent-only pips (the
  delegated default, revisited at attestation); the generic neutral fallback; voices in `banter.rs`
  rather than on `OpponentProfile`, and why. `ROADMAP.md`: mark opponent banter shipped; drop from
  future. Check off `spec.md` acceptance criteria with evidence. Request the pre-merge whole-spec
  sweep.
  *Verify: `cargo build --all-targets` / `cargo test -q` green, reported verbatim; legible
  snapshots at 139×31 and wider showing the in-match banter line + pips, and the unchanged
  opponent-select / campaign-map previews; the sweep confirms no `game.rs`/`player.rs`/`card.rs`/
  `save.rs`/AI/`audio.rs` change and that banter never enters the save format; `ROADMAP.md` no
  longer lists this as future; sweep clean or findings resolved. **Carried from T001 review:** the
  sweep owns the repo-wide single-source check on `ROUND_PIPS` (`portrait.rs`) vs the first-to-3
  win threshold in `game.rs` — confirm they aren't a drift-prone double encoding, or note the
  accepted duplication (display geometry vs game logic).*

---

## Handoff note

Read `CLAUDE.md` and `specs/017-opponent-banter/{spec,plan,tasks}.md`, then implement from the
first unchecked task. Involvement level is **product owner**. Dispatch each routine task to the
`sdd-implementer` per the model policy; verify by running the build and tests yourself, then
commit. **The one task marked `review: per-task` (T001):** run the `skeptical-reviewer` after it,
scoped to that task's diff, its plan section, and its acceptance criteria (a shell-assembled
bundle), one review plus at most one re-review, and re-run the verification command yourself
before committing. **Every other task (T002–T006):** review at phase end. Pause for the person
after each phase, and whenever something unexpected bears on spec adherence. The **T005 phase
pause** is the spec's play-and-read attestation — the person confirms the ten voices read as
distinct and in character and that everything is monochrome.

Model & effort: the session runs at the step-down tier (`opus`), medium effort; the planner and
the `skeptical-reviewer` sign-off/reviews run at the top tier (`fable`) per the per-call
override; implementers run one tier down. Clear at every phase boundary and at spec end.

Every pause produces a report in this shape, in this order:

1. **Why this pause** — a phase boundary, a spec-adherence question, or an escalation
   trigger. One line.
2. **What you can now do** — behavior that exists and can be tried, as a user would
   experience it, so attestation is possible.
3. **Where execution deviated from the spec, and why** — every place, per "never silently,"
   not just the interesting ones.
4. **What needs your decision** — product questions only; technical detail lives in plan.md
   and the commit log.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs and
reviewer invocations), any escape-hatch miss (a task the orchestrator had to redo at the top
tier, and why). Compare the spec total against a previous spec of similar size before
treating the policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| Planning: draft (sdd-planner) | opus (per-call override, per person's request — not the usual fable) | ~94.4K | drafted |
| plan + tasks sign-off (skeptical-reviewer) | opus (per-call override, per person's request — not the usual fable) | ~59.1K | signed off with notes; Note 1 (repeatable-class ≥2-line floor) applied to plan §4 + T001/T002 |
| T001 impl (sdd-implementer) | opus (one down) | ~42.1K | done; build clean, 276 tests pass (orchestrator re-ran) |
| T001 review (skeptical-reviewer, per-task) | opus (one down, per policy — planner's "fable" row was wrong) | ~33.5K | APPROVE WITH NOTES; 2 non-blocking carried (intra-class distinctness→T002, ROUND_PIPS single-source→sweep) |
| T002 impl (sdd-implementer) | opus (one down) | ~35.2K | done; 282 tests pass (verbatim, not per-task); +1 additive test (no line shared across sets) |
| Phase 2 review (skeptical-reviewer) | opus (default) | ~27.3K | APPROVE WITH NOTES; POV/class correct across all 10, voices distinct; 3 taste notes carried to T005 |
| T003 impl (sdd-implementer) | opus (one down) | _TBD_ | _pending_ |
| Phase 3 review (skeptical-reviewer) | opus (default) | _TBD_ | _pending_ |
| T004 impl (sdd-implementer) | opus (one down) | _TBD_ | _pending_ |
| T005 impl (sdd-implementer) | opus (one down) | _TBD_ | _pending_ |
| Phase 4 review (skeptical-reviewer) | opus (default) | _TBD_ | _pending_ |
| T006 close-out (orchestrator) | opus (top) | _TBD_ | _pending_ |
| Pre-merge whole-spec sweep (skeptical-reviewer) | opus (default) | _TBD_ | _pending_ |
