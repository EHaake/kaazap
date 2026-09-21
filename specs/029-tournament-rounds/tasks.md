# Tasks: Tournament rounds — spec 029

> **Status**: Draft — pending sign-off (amendment revision, 2026-09-21).
**Implements**: plan.md in this directory

T001–T006 were signed off on 2026-09-20 (one review, one re-review, B1–B6
resolved), implemented, reviewed, committed, and attested by the person at the
Phase 1 and Phase 2 pauses; nothing about them is reopened here. What is pending
sign-off is **T004a, T005a and T005b** in Phase 2, added 2026-09-21 for
`spec.md`'s *Amendment, 2026-09-21*, and the Phase 2 re-walkthrough in T005b's
Verify. T007–T011 are unchanged and were never started.

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
where tests exist for what changed, an actual test run — reported, not
summarized. Under the model policy, the orchestrator verifies after an
implementer returns, and only the orchestrator commits. This spec runs on the
**economy profile** (the project's since 2026-09-19): every role resolves to
`opus` / `claude-opus-5`, no dispatch carries a top-tier override, and the
close-out is dispatched to `sdd-implementer`.

**Foundational phase: Phase 1.** It is the data model and the settlement rule
every later phase reads — the series on the run, the one floor, the one
exactly-once take. Phases 2–5 each depend on it and on nothing else of each
other's.

**Two tasks carry `review: per-task`** — T001 (the persisted data model and the
exactly-once `take_settlement`, which every later task builds on and which a
save file inherits) and T003 (the settlement: the one place credits are paid and
an opponent is beaten). Every other task is covered by its phase review.

**This is the first spec under the `walkthrough:` marking rule.** Every phase
header below carries `walkthrough: <what the person can try>` or `walkthrough:
none — <why>`, and the running **walkthrough list** near the foot of this file is
what unpaused phases append to and what the person walks at the close-out.

**No new unit test may construct an `App`** (plan §Tests; spec 028 §Design
tension 9): `App::new` reads and repairs the real data directory. Every decision
this spec makes sits behind a pure function or a method on
`CampaignRun`/`Profile`, and is tested there.

**Never run `cargo fmt`** (the repo is not rustfmt-clean; it would rewrite ~28
files).

---

## Phase 1 — The series in the run (foundational; walkthrough: beat Cinder's opponent once — the planet does **not** clear and the map relaunches the same opponent; beat him a second time and it clears and the next worlds unlock. Two wins now take an opponent. There is no venue yet, so the map is still where matches start)

<!-- T001 is the persisted model, T002 the one floor, T003 the settlement rule.

This phase was marked `walkthrough: none` in the first draft, on the strength of
a settlement rule that sign-off finding B1 replaced. Under B1's fix a match
against an un-beaten opponent with no series running *starts* one, so from T003
onward a single campaign win no longer beats anyone — the spec's central rule
becomes true here, three tasks before the venue exists, and it is very much
something the person can see. Marked honestly, not generously; the reviewer's
"these markings are honest" note was written against the pre-B1 draft. -->

- [x] **T001** — `src/campaign.rs` + `src/profile.rs` + `src/app.rs` +
  `src/campaign_map.rs`: the series, and the exactly-once settlement take.
  `review: per-task`. Per plan §Design 1 and §Design tension 3:
  `pub struct Series { planet: String, opponent: String, player_wins: u32,
  opponent_wins: u32 }` deriving `Debug, Clone, PartialEq, Eq, Serialize,
  Deserialize`; `pub enum SeriesOutcome { NotInSeries, Continues, Won, Lost }`
  deriving `Debug, Clone, Copy, PartialEq, Eq`; `pub const FINAL_OPPONENT:
  &str = "sovereign"`; `pub fn wins_needed(opponent: &str) -> u32` (3 for
  `FINAL_OPPONENT`, else 2) and `pub fn series_length_label(opponent: &str) ->
  &'static str` (`"Best of 5"` / `"Best of 3"`). On `CampaignRun`, add
  `#[serde(default)] series: Option<Series>` and `series()`, `begin_series`,
  `record_series_match` exactly as the plan's listing — **three cases, and the
  discriminator is `is_opponent_beaten`, not whether a series exists (sign-off
  B1)**: the locked node's series is tallied; an **already-beaten** opponent is
  a rematch and returns `NotInSeries`; anything else is a match against an
  un-beaten opponent with no series of its own — a match left in flight across
  the upgrade — so `begin_series` for it and credit the match (1–0,
  `Continues`), which is `spec.md` §Saving and resuming's "resolves as the first
  match of a fresh series against that opponent". Without that branch such a
  match beats its opponent outright and clears a world in one. Carry the plan's
  doc comment in full, including why a *different* series running in that third
  case is replaced rather than special-cased.
  On `NodeRef`, add `#[serde(default)] pub settled: bool` with the plan's doc —
  `NodeRef` derives no `Default` and is built as a struct literal in
  **`src/app.rs`** (the wager `Commit` arm, ~941, production), **`src/app.rs`**
  again and **`src/campaign_map.rs`** in tests, `src/profile.rs`'s `node` test
  helper, and **three times** in `campaign.rs`'s own tests (~397, ~414, ~421). **Every one of those gains
  `settled: false`, in this task** — the build breaks otherwise, and
  `cargo build --all-targets` is this task's bar.
  **Replace `take_stake` with `take_settlement`** per the plan's listing —
  `std::mem::replace(&mut node.settled, true)` guards, then `mem::take` empties
  the escrow, returning `(NodeRef, u32)`. `take_stake` is **deleted**, and its
  one non-test caller is `Profile::settle_campaign_match`, so **that call site
  changes in this task too** — a deleted method with a live caller is a hard
  error, not a warning, and `cargo build --all-targets` is this task's bar. The
  edit there is exactly two lines becoming one:
  `let (node, stake) = self.campaign.take_settlement()?;` in place of the
  `in_progress()?.clone()` + `take_stake()` pair. Do **not** change the
  settlement *rule* — that is T003; after this task `settle_campaign_match`
  still pays, marks beaten and counts completions exactly as it does today.
  Leave `stake_at_risk` exactly as it is.
  Tests, in `campaign.rs`'s tests module (plan §Tests):
  `wins_needed_is_two_except_for_the_final_opponent` (every roster opponent via
  `OPPONENTS`; plus `FINAL_OPPONENT` is the last opponent of the last planet in
  `PLANETS`, so a map edit that retires the boss fails here);
  `a_series_resolves_only_at_the_required_wins` (drive `record_series_match`
  through best-of-three both ways and best-of-five, asserting `Continues` until
  the required win, then `Won`/`Lost`, and `series()` is `None` after);
  `a_match_in_flight_with_no_series_starts_one` (**sign-off B1**: no series, an
  **un-beaten** opponent → `Continues` and a series at 1–0 against that node;
  the same call with an **already-beaten** opponent → `NotInSeries` and still no
  series);
  `no_settled_match_beats_an_unbeaten_opponent_outright` (the property B1
  exists for, stated directly: over a series against one node, a settled match
  against a *different* un-beaten node, and a match with no series at all, the
  result is never one that would beat an un-beaten opponent — i.e.
  `NotInSeries` is returned **only** when `is_opponent_beaten` is true);
  `take_settlement_hands_over_the_match_exactly_once` (the shape of the existing
  `take_stake_empties_the_escrow_exactly_once`, which this replaces: a staked
  node yields the node and the stake, then `None`, and `stake_at_risk()` is
  `None` after); `a_pre_029_node_is_unsettled_and_unstaked` (deserialize
  `{"planet":"cinder","opponent":"greeb"}` → `settled: false`, `stake: 0`, and
  `take_settlement` hands it over once); and a round-trip of a `CampaignRun`
  carrying a series. Do not run `cargo fmt`. (Copies: `campaign.rs`'s own
  `NodeRef`/`take_stake` doc voice and its tests module.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new tests passing and **every existing assertion in
  `campaign.rs`, `profile.rs`, `app.rs` and `campaign_map.rs` unchanged in
  value, with exactly four named exceptions** (amended 2026-09-20 by a decision
  review, which found the original one-exception list an enumeration error that
  made this gate unsatisfiable) — (1)
  `take_stake_empties_the_escrow_exactly_once` in `campaign.rs`, which this
  task replaces with `take_settlement_hands_over_the_match_exactly_once`, so
  its assertions change shape by design; and the **second-settlement line** in
  each of three `profile.rs` tests, which pinned the *old* mechanism (an
  emptied escrow paying `win_payout(0)`) and must now pin the new one (a
  consumed `settled` flag returning `None`, plan §Design tension 3): (2)
  `settling_a_win_pays_double_the_stake_and_marks_the_node_beaten` — the second
  `settle_campaign_match(true)` is `None`, not `Some(StakeOutcome::Won(0))`;
  (3) `settling_a_loss_keeps_the_stake_and_leaves_the_node_unbeaten` — the
  second `settle_campaign_match(false)` is `None`, not
  `Some(StakeOutcome::Lost(0))`; (4)
  `resolve_match_moves_the_run_credit_counters_and_nothing_else_does` — the
  second `resolve_match("greeb", true, 3, 1)` is `None`, asserted whole rather
  than through `.map(|s| s.outcome)`. In all three the surrounding credit and
  counter assertions — `credits() == 70`, `credits() == 30`, `credits_won ==
  net`, `credits_lost == 0`, and `in_progress().map(|n| n.stake) == Some(0)` —
  are **unchanged in value**; the property (settling twice settles once) is
  what still holds, only the evidence moves. Reword the three comments that
  call the escrow "already empty" to name the `settled` flag instead. **Any
  assertion outside those four tests that moves, and any *value* change to the
  credit or counter assertions inside them, means stop and report.** Otherwise
  the only edits outside `campaign.rs` are the `settled: false` additions to
  `NodeRef` literals, the one `take_settlement` call site and those three
  assertions; `git diff --stat` shows exactly
  `src/campaign.rs`, `src/profile.rs`, `src/app.rs` and `src/campaign_map.rs`,
  with `app.rs` and `campaign_map.rs` changed on literal lines only;
  `grep -rn "take_stake" src/ tests/` is empty — the doc comment carried from
  the plan names the mechanism ("by zeroing the escrow"), not the deleted
  method, so this gate is satisfiable without weakening it; the implementer's report quotes
  `take_settlement` and `record_series_match` verbatim. The orchestrator re-runs the verification
  command itself before committing (per-task review), then dispatches a
  `skeptical-reviewer` on T001's diff alone before T002 starts.*

- [x] **T002** — `src/economy.rs` + `src/profile.rs` + `src/shop.rs` +
  `src/app.rs` + `tests/balance.rs` + `docs/economy.md` + `docs/balance.md`:
  one floor, every caller that reads it, and the document that explains it. Per
  plan §Design tension 2: make `cheapest_floor` **private** (`fn`, not `pub fn`)
  and add `pub fn reserve_floor(run: &CampaignRun) -> u32` exactly as the plan's
  listing, with its doc naming ruling O1 and the reason. Then move **every
  caller in the same task** — a `pub fn` whose only callers are `#[cfg(test)]`
  is `dead_code` against `cargo build --all-targets`, and a private
  `cheapest_floor` with no in-crate caller is too (spec 028's T003 lesson).
  The callers, all of them, because the previous draft of this task named four
  and there are eight: `Profile::is_broke`, `Profile::can_afford`, **two
  `economy::cheapest_floor` calls in `profile.rs`'s own tests (~901, ~1274)**,
  `shop.rs`'s `spendable` line, **`src/app.rs`'s `WagerState::new` reserve
  argument in `launch_campaign_node` (~920)**, and **two sites in
  `tests/balance.rs` (~455 and ~596)** — not one. `reserve_floor` returns the
  same value for a default run, so the simulator's report is unchanged. Nothing
  else in any of those functions changes.
  Then **`src/wager.rs`, doc only** (re-review finding B6): `WagerState::new`'s
  `reserve` doc (~62–64) calls it "the run's cheapest ante
  (`economy::cheapest_floor`)", which ruling O1 makes false — correct it to name
  `reserve_floor` and say that while a series is locked the reserve is that
  opponent's own ante, so the run-over warning row fires far more often deep in
  the map. The test-helper doc (~211) names the same function; correct it too.
  **No code in `wager.rs` changes** — these two comments are the whole edit, and
  they are what lets this task's grep gate below be satisfiable at all.
  Then the documentation, which rides the branch (CLAUDE.md git conventions),
  not the close-out — plan §Files: **`docs/economy.md`** asserts the old rule in
  seven places and must be corrected to the new one: `take_stake`'s entry
  (~107), the exactly-once paragraph (~113–118, which this spec *strengthens* —
  say so, and say what it still does not cover: `record_match` runs before
  settlement), the `is_broke` sentence (~125–128 — the floor, the function name
  and the `enter_campaign_map` seam all move; this is the most load-bearing line
  in the document), the shop reserve and the "cheapest_floor is 10 in every run
  state" tuning note (~134, ~151–153, ~157, ~165–167 — no longer true while
  locked), "reached from the campaign map with `b`" (~239, ~262 — the Outfitter
  is now also reached from the venue), and the named test (~287). **The one
  prose line in `docs/balance.md` (~195) that names `cheapest_floor` belongs to
  this task too**, not to T010 — which is why T010's "additions only" bar is not
  weakened by it. Describe `is_broke`'s seams as they will be *after* T006
  (`App::enter_campaign`), and say so in a parenthetical if that function does
  not exist yet at this task.
  **Name the visible consequence in the `docs/economy.md` edit** (plan §Design
  tension 2): the wager prompt's "lose this and the run is over" warning is
  driven by this floor, so while locked against a deep opponent it fires at far
  lower stakes than it does today. Correct under O1, unreachable in any of this
  spec's walkthroughs, and therefore written down rather than attested.
  Tests: in `economy.rs`, `reserve_floor_follows_the_lock` — with no series it
  equals `cheapest_floor` on a fresh, a half-cleared and a complete run; with a
  series against `rix` it is 50 while the cheapest is 10; **and for every roster
  opponent, a run locked against them has `reserve_floor == ante_floor_for(id)`**
  (the claim that makes the venue's launch always affordable, plan §Design
  tension 2). Keep `cheapest_floor_is_the_min_over_launchable_nodes` unedited.
  In `profile.rs`, `broke_and_affordable_follow_the_locked_floor` — credits 20
  with a series against `rix` is broke and cannot afford a 20-credit card; the
  same profile with no series is neither. Do not run `cargo fmt`. (Copies:
  `economy.rs`'s own doc voice and its `cleared()` test helper.)
  *Verify: `cargo build --all-targets` no new warnings — in particular no
  `dead_code` on either floor function; `cargo test -q` green verbatim with
  every existing assertion unchanged in value; `git diff --stat` shows exactly
  `src/economy.rs`, `src/profile.rs`, `src/shop.rs`, `src/app.rs`,
  `src/wager.rs`, `tests/balance.rs`, `docs/economy.md` and `docs/balance.md`;
  `grep -rn "cheapest_floor" src/ tests/ docs/` matches **only**
  `src/economy.rs`; the implementer's report quotes `reserve_floor` and all
  eight changed call sites verbatim, and pastes the `docs/economy.md` diff and
  the two corrected `src/wager.rs` comments.*

- [x] **T003** — `src/profile.rs` + `docs/economy.md`: the settlement rule. `review: per-task`. Per
  plan §Design 3: `Settlement` gains `pub series: SeriesOutcome` with the
  plan's doc; `settle_campaign_match` becomes the plan's listing exactly —
  one `take_settlement`, then `record_series_match`, then the payout, then
  `mark_beaten` **only** when the series was `Won` or when there is no series
  at all and the match was won, with the completion edge unmoved inside that
  branch; `resolve_match` carries `series` into the `Settlement` it returns and
  is otherwise unchanged (it still records statistics before settling — spec
  024's ordering, which the first-clear record depends on).
  **This is the task where the campaign's central rule changes, so existing
  tests change with it — deliberately, and in exactly two ways.** (1) Every
  `Settlement { … }` literal gains `series: …` — mechanical. (2) Every test that
  plays **one** campaign match against an **un-beaten** opponent and then
  asserts the opponent is beaten or the planet cleared must now play the
  **series**: the helpers `play_node` and `sweep_run` **gain** a
  play-a-whole-series form (win `wins_needed(opponent)` matches) — *gain*, not
  change: the existing forms keep their behavior. If a helper's own behavior is
  mutated instead, many tests move value without any of them appearing in the
  list this task requires, which is the one way the two-kinds bar leaks. The
  report must therefore also name any helper whose behavior changed and which
  tests that moves. The assertions move from "one
  win clears" to "two wins clear, one does not". That is the spec, not a
  weakened test, and the intermediate assertion (after one win the opponent is
  still un-beaten) is worth adding where it is cheap. **Rematch tests must not
  change value** — an already-beaten opponent still settles through
  `NotInSeries` exactly as before. If any *other* assertion changes value, or
  it is unclear which of the two cases a test is, **stop and report rather than
  adjusting the expectation**.
  **`docs/economy.md` rides with it** (added by the orchestrator at T002,
  2026-09-20, from a finding the T002 implementer returned). Step 3 of that
  document's `settle_campaign_match` walkthrough still reads "On a win, adds
  `win_payout(stake)`, calls `mark_beaten`, and counts a campaign completion
  only on the **edge**" — which is exactly the rule this task replaces. It was
  not among the seven places T002 enumerated, because T002 did not falsify it;
  this task does. Correct step 3 to name the series: the payout is unchanged, but
  `mark_beaten` now fires when the **series** is won (or on a rematch win against
  an already-beaten opponent, which re-marks someone already beaten — a no-op),
  and the completion edge is unmoved inside that branch. Also correct the two
  `profile.rs` doc comments that say `None` means "a Quick Play match" — since
  T001 it also means an already-settled match (close-out note 1). Same principle
  as sign-off finding B4: a spec that renames symbols because a false name is a
  defect cannot leave the document that explains them asserting the old rule, and
  `docs/economy.md` rides the branch rather than the close-out.
  Tests (plan §Tests), in `profile.rs`'s tests module:
  `a_series_beats_the_opponent_only_at_the_deciding_win` (best of three on
  `cinder`/`greeb`: after one win the opponent is **not** beaten and the planet
  **not** cleared; after the second it is, `scree` and `ashfall` unlock, credits
  are the two payouts);
  `a_lost_series_takes_nothing_beyond_the_stakes` (0–2: opponent un-beaten,
  planet uncleared, credits down by exactly the two stakes, `series()` back to
  `None`, no completion counted);
  `resolving_a_match_twice_pays_beats_and_counts_once` (the bundle's exactly-once
  point: `resolve_match` twice on one match — credits move once, the tally
  reaches its value once, the opponent is beaten once, completions move at most
  once);
  `the_deciding_match_and_the_series_agree` (over both lengths and both winners,
  `SeriesOutcome::Won` ⟺ the settled match was won — the claim that keeps the
  map banner from a mixed case);
  `the_final_opponent_needs_three` (a best-of-five on `zenith`/`sovereign`: two
  wins do not complete the run, the third does and counts the completion once);
  `every_reset_clears_the_lock` (`reset_campaign_run` and `reset_to_starter`
  each leave `series()` `None` from 1–0); and — **sign-off B1, the one this
  task exists to get right** —
  `a_pre_029_match_in_flight_does_not_beat_its_opponent`: deserialize a
  `CampaignRun` from JSON with an `in_progress` node and **no `series` key**
  (the shape a profile written before this spec has), `resolve_match` with a
  win, and assert the opponent is **un-beaten**, the planet **uncleared**, the
  stake paid, and the series at 1–0 against that node. Do not run `cargo fmt`. (Copies:
  `profile.rs`'s own `node`/`play_node`/`sweep_run` test helpers — use them
  rather than writing new ones.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new tests passing; `git diff --stat` shows only
  `src/profile.rs` and `docs/economy.md` (the latter added at T002 — see the
  paragraph above); `git diff -- src/profile.rs` shows `resolve_match`'s
  record-then-settle order intact; **the implementer's report lists every
  existing test it changed and which of the two sanctioned reasons each change
  was** (a `Settlement` literal, or one-win-clears becoming two-wins-clear), so
  the per-task review can check that list rather than re-deriving it; the report
  quotes `settle_campaign_match` verbatim. The orchestrator re-runs the
  verification command itself before committing (per-task review), then
  dispatches a `skeptical-reviewer` on T003's diff alone before T004 starts.
  **PAUSE for the person** (after the Phase 1 review): the orchestrator drives
  the Phase 1 walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` at a scratch directory, confirmed in the report — and
  reports in plain language that one win against Cinder's opponent no longer
  clears the planet, that the map offers him again, that the second win clears
  it and unlocks the next two worlds, and that a loss along the way costs only
  its stake. There is no venue yet and the map is still where matches start;
  say so, so the person is not looking for one. One more thing to tell them:
  the lock arrives in Phase 2, so until then starting a series against a
  different opponent silently replaces the one in progress. Ask them to finish a
  series before switching opponents, so a replaced series is not reported back
  as lost progress.*

## Phase 2 — The venue and the lock (walkthrough: Enter on an un-beaten planet now opens the venue at 0–0 with nothing staked; play matches from it, come back to it between them, open the Card Shop and the collection and return to it, quit to the menu and re-enter the campaign to land back at it — and win the series to be handed back to the map; then, after T004a–T005b, walk the amended venue: horizontal bands with a dominant art region at **both** widths, `Best of 3` in the series row, and a **Card Shop** button that opens a screen headed the same)

<!-- T004 is the geometry, T005 the screen module (pub, so nothing is dead code
before it is wired), T006 the wiring and the one routing rule. The venue is
reachable only after T006.

T004a, T005a and T005b were added 2026-09-21, after the person walked the venue
at the Phase 2 pause and ruled the three changes now in `spec.md` as
*Amendment, 2026-09-21*. They land in this phase rather than a new one because
this is the venue's phase and T007 onward had not started: the amendment changes
what an already-built screen shows, not what any later phase depends on.

Order matters a little and only in one direction: **T004a first**, because it
rewrites `VenueLayout` and the venue's `draw` together (the geometry and its
only consumer cannot land in separate builds), and T005a/T005b are one-line
wording changes inside the file it leaves behind. T005a and T005b are
independent of each other.

**None of the three carries `review: per-task`**, and that is deliberate rather
than an oversight. The criterion is a task whose mistake later files would
inherit; nothing outside `venue.rs` reads `VenueLayout`, and Phases 3–5 touch
none of it. The phase **re-review** covers all three diffs together, and T004a's
enumerated list of changed assertions is what that review checks against.

The walkthrough list near the foot of this file gets a second Phase 2 row when
the re-walkthrough is attested — the orchestrator writes it, as with every
other row. -->


- [x] **T004** — `src/layout.rs`: the venue's geometry. Per plan §Design 4:
  `pub const VENUE_ART_W: usize = 30`, `pub const VENUE_PANEL_H: usize = 2 + 1 +
  PORTRAIT_HEIGHT`, `pub struct VenueRail { pub art: Rect, pub portrait: Rect }`
  and `pub struct VenueLayout { pub center_x, pub top, pub rail: Option<VenueRail> }`
  with `VenueLayout::new(config: Config, block_h: usize)`. The rail is `Some`
  iff `cols >= WIDE_LAYOUT_MIN_WIDTH` (the `BoardLayout::opponent_panel`
  idiom, ruling N1); rail width is `VENUE_ART_W + PANEL_GAP + PANEL_W` with a
  3-column right margin; `art` and `portrait` share a top and a bottom
  (`VENUE_PANEL_H` rows each), vertically centered; `center_x` is the middle of
  the area **left of the rail** when there is one, and of the whole terminal
  when there is not; `top = (rows - block_h) / 2`. Carry the plan's doc
  comments, including why the two Rects are one `Option` of a struct rather than
  two `Option`s.
  Tests, in `layout.rs`'s tests module: `the_venue_rail_is_wide_only_and_clear`
  — at 89×31 `rail` is `None`; at 139×31 it is `Some`, `art.x1 + PANEL_GAP <
  portrait.x0`, `portrait.x1 < cols`, `art.y0 == portrait.y0 && art.y1 ==
  portrait.y1`, both on-frame; and at both sizes `top + block_h <= rows` for
  `block_h = 8`. Use `Config::fit_sizes()`. Do not run `cargo fmt`.
  (Copies: `layout.rs`'s `BoardLayout::new` for the wide-layout test and
  `CampaignMapLayout` for a full-screen layout struct's doc voice.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new test passing; `git diff --stat` shows only
  `src/layout.rs`; the report quotes `VenueLayout::new` verbatim and states the
  concrete Rects it computed at 139×31.*

- [x] **T005** — `src/venue.rs` (new) + `src/lib.rs`: the venue screen. Per plan
  §Design 5: the module doc (what the venue is, and that it is a `Screen`
  because it is a mode the player navigates *to*); `pub enum VenueOutcome {
  Moved, Play, OpenShop, OpenCollection, QuitToMenu }` — **one owned outcome
  enum**, per the constitution; `const ACTIONS: [&str; 4] = ["Play",
  "Outfitter", "Collection", "Quit"]`; `pub struct VenueState { selected:
  usize }` with `new`, `handle_input(key) -> Option<VenueOutcome>` (←/→ and
  `a`/`d` wrap; Enter/Space take the highlighted action; `b` → `OpenShop` and
  `c` → `OpenCollection` from any position, as on the map; Esc/`x` →
  `QuitToMenu`; everything else `None`) and `draw(frame, config, profile,
  pulse)` drawing the plan's eight rows through `VenueLayout`. `pub const
  BLOCK_H: usize = 8`. Pure helpers so the fit and the wording are testable
  without a terminal: `series_line(series: &Series) -> String` (`Series  1 – 0
  ·   first to 2`, the count from `wins_needed`) and `action_row_width() ->
  usize`. The action row uses `app.rs`'s `draw_choice_panel` idiom — each label
  drawn as `"{▸ or space} {label}"` at a fixed stride, so the row's width does
  not change as the cursor moves and only the cursored label takes `pulse`.
  The rail, when `Some`: `draw_box` around `rail.art` (Single, Muted) with the
  planet's name centered inside it (Muted — the plain placeholder of ruling M1),
  then `draw_presence_panel(frame, rail.portrait, opponent.name,
  opponent.portrait)`. `draw` returns early when
  `profile.campaign().series()` is `None`, with a comment saying `App` only
  shows this screen while a series is in progress. `lib.rs` gains `pub mod
  venue;` beside `pub mod shop;`. **No `app.rs` or `screen.rs` change in this
  task** — the module is `pub`, so nothing is dead code yet.
  Tests, in `venue.rs`'s tests module (plan §Tests):
  `the_action_row_keeps_its_width_as_the_cursor_moves`;
  `the_venue_block_breathes_only_around_the_action_row` (rows 4 and 6 blank,
  rows 0–3 and 5 and 7 non-blank — build the rows through the same helper
  `draw` uses, don't re-type them);
  `the_venue_text_fits_the_minimum_terminal` (every row, at both
  `Config::fit_sizes()`, centered on `VenueLayout::center_x`, inside the frame
  and — at 139 — ending left of `rail.art.x0`);
  `the_series_line_names_the_score_and_the_length` (`first to 2`, and `first to
  3` against `sovereign`); and the key tests
  (`the_venue_keys_move_confirm_and_shortcut`). Do not run `cargo fmt`.
  (Copies: `src/opponent_select.rs` — the constitution's reference screen shape,
  its `preview_rect` and its tests module; `app.rs`'s `draw_choice_panel` for
  the action-row idiom.)
  *Verify: `cargo build --all-targets` no new warnings — in particular no
  `dead_code`; `cargo test -q` green verbatim with the new tests passing;
  `git diff --stat` shows only `src/venue.rs` and `src/lib.rs`, with `lib.rs`
  the one `pub mod` line; the report quotes `handle_input` and the eight drawn
  rows verbatim, and states the widest row's character count at both widths.*

- [x] **T006** — `src/screen.rs` + `src/app.rs` + `src/deck_builder.rs`: the
  wiring, and the one routing rule. Per plan §Design tension 1 and §Design 6:
  `Screen::Venue { state: VenueState }`; `fn open_campaign_home(&mut self)` —
  an inline `if` over `series().is_some()`, and **the only place
  `self.screen` is assigned a campaign screen**, which is what keeps the return
  target from drifting (`open_campaign_map` is absorbed into it). **No
  `CampaignHome` enum and no `campaign_home` mapping function** — an earlier
  draft had them for testability and sign-off was right that the test would be a
  tautology; the invariant that matters is "no second assignment site", which is
  a grep, below.
  `enter_campaign_map` renamed to `enter_campaign` and its body's screen line
  replaced by `open_campaign_home()`; `map_entry_modal` renamed to
  `campaign_entry_modal` (and its test's name); `BuilderOrigin::Map` →
  `BuilderOrigin::Campaign` and `BackTo::Map` → `BackTo::Campaign`, with
  `BackTo::Campaign => self.open_campaign_home()`; the Outfitter's
  `ShopOutcome::Back` arm → `self.open_campaign_home()`; all four
  `enter_campaign_map` call sites → `enter_campaign`, **and the
  `open_campaign_map()` call in `app.rs`'s own test
  `the_primer_swallows_map_keys` (~3092), which the previous draft of this task
  missed**. Split
  `launch_campaign_node` into `launch_from_map(planet, opponent)` (the deck
  guard, then `is_opponent_beaten` → `open_wager` for a rematch, else
  `begin_series` + `save` + `open_campaign_home`) and `open_wager(planet,
  opponent)` (today's body minus the deck guard), per the plan's listing; add
  `play_series_match` for the venue's `Play` (deck guard, then `open_wager` on
  the locked series' node). Add the three `Screen::Venue` arms: input (the
  plan's listing, with `MenuMove`/`MenuSelect`/`MenuBack` matching the map's
  arm), `?`-help (`None`, beside the map and shop), and draw
  (`state.draw(frame, &self.config, &self.profile, pulse)`). **No `tick`,
  `resize` or `save_game` arm** — the venue has no animation, rebuilds from
  `self.config` each draw, and holds no match.
  **No new unit test** (plan §Tests): the routing rule is an `if` over a
  boolean, and what needs checking is that no *other* line assigns a campaign
  screen — a grep, not a test. Do not run `cargo fmt`. (Copies: `app.rs`'s own
  `Screen::CampaignMap` input arm for the screen-arm shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/screen.rs`, `src/app.rs` and
  `src/deck_builder.rs`;
  **`grep -nE "self\.screen = Screen::(CampaignMap|Venue)" src/app.rs` returns
  exactly two lines, both inside `open_campaign_home`** — this is the invariant
  the whole routing design rests on, and the grep is written this way because
  the obvious one (`"Screen::CampaignMap {"`) also matches every pattern-match
  arm and would pass while saying nothing; `grep -rn
  "enter_campaign_map\|open_campaign_map\|launch_campaign_node\|BuilderOrigin::Map\|BackTo::Map"
  src/` is empty; the report quotes `open_campaign_home`, `launch_from_map` and
  the venue input arm verbatim, and pastes the two-line grep output.
  **PAUSE for the person** (after the Phase 2 review): the orchestrator drives
  the Phase 2 walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` pointed at a scratch directory, confirmed in the report —
  at 89×31 and 139×31, and reports in plain language: that Enter on an un-beaten
  planet opens the venue at 0–0 with the credit balance untouched; what the
  venue shows at each width (and that the art region and the portrait sit beside
  each other, unclipped, at the wider one, and that neither appears at the
  narrower); that `b` and `c` come back to the venue rather than the map;
  **that choosing Play and then pressing Esc at the wager prompt comes back to
  the venue with nothing staked** (AC 4, which no test covers); that a
  lost match returns to the venue with the score at 0–1; that quitting to the
  menu and re-entering the campaign lands back at the venue with the same score
  and the map cannot be reached; **that a match quit mid-play resumes from
  Continue and then lands on the venue or the map by the same rules** (AC 13,
  likewise uncovered); that winning two hands the player back to the
  map with the planet cleared; and that Enter on a cleared planet goes straight
  to the wager prompt with no venue.
  **The report ends with two questions for the person** — questions, not
  findings, because both are about what the venue shows and that is theirs to
  decide: (a) the art region's placeholder is the **planet's name**, which the
  venue already prints six rows above it — do they want it emptier? (b) the
  venue shows **no credit balance**, on the screen where the player chooses
  between playing and shopping. Neither is in this design; both are one small
  task if they say yes.*

<!-- The three tasks below are the 2026-09-21 amendment. Everything above this
line is done, reviewed, committed and attested. -->

- [ ] **T004a** — `src/layout.rs` + `src/venue.rs`: the venue becomes horizontal
  bands, at every width. Per plan §Design tension 7 and §Design 4 and 5, for
  `spec.md`'s amendment **R3**. Two files in one task because they cannot build
  apart: `VenueLayout`'s shape changes and `venue.rs` is its only consumer.
  `src/layout.rs`: delete `VENUE_ART_W` and `VenueRail` (the `Option` they
  existed for is gone — the art and the portrait draw at 89 columns too, R3
  superseding ruling N1); keep `VENUE_PANEL_H` as the portrait panel's height;
  `VenueLayout` becomes the plan's six public fields (`center_x`, `header_y`,
  `art`, `portrait`, `action_y`, `hint_y`) with `pub const HEADER_H: usize = 4`,
  `pub const FOOTER_H: usize = 4` and `const MARGIN_X: usize = 3`;
  `VenueLayout::new(config: Config)` **loses its `block_h` parameter** and
  computes the plan's ten lines of arithmetic in that order, clamping with the
  `.max()` idiom `CampaignMapLayout::new` uses for `field_bottom`. Carry the
  plan's doc comments, including why the two Rects are no longer wrapped and why
  the rows under the portrait stay blank.
  `src/venue.rs`: delete `pub const BLOCK_H`; replace `text_rows` with
  `header_rows(planet_name, planet_region, opponent, series) -> [String;
  VenueLayout::HEADER_H]` (drop the `selected` parameter — the action row is no
  longer one of the returned rows); `draw` draws the four header rows at
  `layout.header_y + 0..4` with today's emphases, then `draw_box` around
  `layout.art` (Single, Muted) with the planet's name centered in it — **the
  placeholder's contents do not change in this task** (plan §Open questions 2 is
  the person's, still open) — then `draw_presence_panel(frame, layout.portrait,
  …)` with **no `if let Some(rail)`**, then the action row at `layout.action_y`
  by today's label-stride loop, then the hint at `layout.hint_y`. Correct the
  module doc's sentence about the right rail "from 139 columns up (ruling N1)":
  it is false after this task, and a false doc comment is the defect this spec
  renamed four symbols to avoid.
  **This task changes the layout two shipped tests pin, so existing assertions
  change value — deliberately, and in exactly two kinds. Anything else that
  moves is a stop-and-report, not an expectation to adjust.**
  (1) **`layout.rs`'s `the_venue_rail_is_wide_only_and_clear` is replaced** by
  `the_venue_bands_stack_and_the_art_takes_the_rest` (both "rail" and
  "wide-only" are false now). Four of its lines change value or go, named here
  so nobody has to decide:
  • `assert!(l.rail.is_none(), "rail drawn at {cols} columns")` at 89 columns —
  **deleted**; there is no `rail` field, and both regions now draw at every
  width.
  • `assert_eq!(rail.art.y1, rail.portrait.y1, "rail bottoms differ")` — **old
  value: equal. New bar: `portrait.y1 <= art.y1`, with `portrait.height() ==
  VENUE_PANEL_H`** (15 rows against the art's 23). Only their **tops** still
  match, and `assert_eq!(art.y0, portrait.y0)` is kept unchanged.
  • `assert!(l.center_x < rail.art.x0, "text block centers on the rail…")` —
  **deleted**; `center_x` is the terminal's center now (69 at 139 columns, which
  is inside the art's columns), and the text clears the art **vertically**
  instead. Replaced by `header_y + HEADER_H == art.y0` and `art.y1 < action_y`.
  • `const BLOCK_H: usize = 8` and `assert!(l.top + BLOCK_H <= rows, …)` —
  **deleted** with the `top`/`block_h` pair they measured.
  Kept with their values unchanged: the `Config::fit_sizes()` loop, both Rects
  `in_bounds`, `art.x1 + PANEL_GAP < portrait.x0`, `portrait.x1 < cols`,
  `art.y0 == portrait.y0`.
  (2) **`venue.rs`'s `the_venue_block_breathes_only_around_the_action_row` and
  `the_venue_text_fits_the_minimum_terminal`** change:
  • the breathing test becomes `the_venue_rows_breathe_only_around_the_action
  _row`, **a frame test** — its `assert_eq!(BLOCK_H, 8, …)` and its rows-4-and-6
  indices go with `BLOCK_H`. Draw the venue at 89×31 into `frame::new_frame`
  over a `Profile::default()` with `campaign_mut().begin_series("cinder",
  "greeb")` — **no `App`** (file header rule; `Profile::default()` touches no
  disk, and nothing in this test calls `save()`) — then assert rows `action_y -
  1` and `action_y + 1` are entirely blank while the four header rows,
  `action_y` and `hint_y` carry text. This is AC 17 on the drawn frame, which is
  strictly stronger than the array version it replaces.
  • the fit test **loses one clause**: `assert!(x + w <= rail.art.x0, …)` is
  **deleted** — the text is above and below the art now, not beside it.
  Everything else about it (every planet × opponent × selection, both fit sizes,
  `x + w <= cols`) keeps its value, with the rows now coming from `header_rows`,
  the hint, **and the action row measured explicitly** as
  `action_labels(selected).join(&" ".repeat(ACTION_GAP))` — the same string the
  stride loop draws. It was row 5 of the old array; without naming it here the
  widest cursored row would quietly stop being measured.
  **Not changing, and a stop-and-report if they do**:
  `the_action_row_keeps_its_width_as_the_cursor_moves`,
  `the_venue_keys_move_confirm_and_shortcut`,
  `the_series_line_names_the_score_and_the_length` (T005a's, not this task's),
  and every test in every other file — `layout.rs`'s board, briefcase, map and
  overlay tests included.
  One test is new: `the_art_region_dominates_at_both_widths` — the art's area
  exceeds the portrait's at each `Config::fit_sizes()`, and the art's area at
  139 columns strictly exceeds its area at 89 (AC 16's amended sentence). The
  bands test also pins the concrete Rects of plan §Design 4's table: at 89×31
  `art == Rect::new(3, 60, 4, 26)` and `portrait == Rect::new(64, 85, 4, 18)`;
  at 139×31 `art == Rect::new(3, 110, 4, 26)` and `portrait == Rect::new(114,
  135, 4, 18)`. Do not run `cargo fmt`.
  (Copies: `CampaignMapLayout::new` in the same file — the full-screen
  header/field/panel banding, its `.max()` clamp and its top-anchored
  `portrait_panel` are exactly this layout's shape; `src/portrait.rs`'s tests
  and `board.rs`'s `the_compact_board_carries_the_stake_clear_of_the_alert` for
  a blank-frame drawing test.)
  *Verify: `cargo build --all-targets` no new warnings — in particular no
  `dead_code` from the deleted constants; `cargo test -q` green verbatim, with
  the replaced and new tests passing; `git diff --stat` shows only
  `src/layout.rs` and `src/venue.rs`; `grep -rnE "VENUE_ART_W|VenueRail" src/`
  is empty and `grep -n "BLOCK_H" src/venue.rs` is empty (do **not** grep
  `layout.rs` for `BLOCK_H` — `BriefcaseLayout::BLOCK_H` is an unrelated private
  constant that stays); the report pastes `VenueLayout::new` verbatim, states
  the concrete `art` and `portrait` Rects it computed at 89×31 and at 139×31,
  and **lists every existing assertion it changed with the old value and the new
  one**, so the phase re-review checks that list rather than re-deriving it.*

- [ ] **T005a** — `src/venue.rs`: the series line reads `Best of 3`. Per plan
  §Design 5, for amendment **R1**. `series_line` formats `"Series  {} – {}   ·
  {}"` with `campaign::series_length_label(&series.opponent)` in place of
  `wins_needed(&series.opponent)` — the same function the map's planet detail
  uses, so the player meets one phrase for one idea. Drop the now-unused
  `wins_needed` import (keep `Series`); `wins_needed` itself stays where it is
  and keeps its other callers.
  **Exactly two existing assertion values change, both in
  `the_series_line_names_the_score_and_the_length`, and they are these**:
  `"Series  1 – 0   ·   first to 2"` → `"Series  1 – 0   ·   Best of 3"`, and
  `"Series  2 – 1   ·   first to 3"` → `"Series  2 – 1   ·   Best of 5"`. **No
  other assertion anywhere changes value** — the new line is 29 characters
  against the old 30, and neither was ever the widest row (the hint is, at 63),
  so `the_venue_text_fits_the_minimum_terminal` and every layout test stay green
  **unedited**. If any of them moves, stop and report: it means a width
  assumption in the plan is wrong.
  Add one assertion the old test could not make: `series_line` **contains**
  `series_length_label(&series.opponent)` for both lengths, so R1's actual point
  survives a later edit to either screen instead of resting on two literals that
  happen to agree today. Do not run `cargo fmt`. (Copies: the test's own
  existing body — this is an edit of four lines, not a new test.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/venue.rs`; `grep -rn "first to"
  src/` is empty (today it returns exactly the three lines this task
  rewrites — the phrase exists nowhere else in `src/`); the report quotes
  `series_line` and both new expected strings verbatim.*

- [ ] **T005b** — the Card Shop rename: `src/shop.rs` + `src/venue.rs` +
  `src/economy.rs` + `src/app.rs` + `src/profile.rs` +
  `assets/primer_text.txt` + `assets/how_to_play_text.txt` + `docs/economy.md` +
  `README.md`. Per plan §Design 5 and §Files, for amendment **R2**. Nine files,
  no behavior change, one new `const`.
  `src/shop.rs`: hoist `TITLE` out of `draw` to module scope as `pub const
  TITLE: &str = "Card Shop";` with the plan's doc comment (one const, two
  readers — a button that says one thing and opens a screen headed another is
  the defect R2 exists to fix, and a shared const makes it unrepresentable
  rather than merely tested); rename the module doc's "the between-worlds
  outfitter". **Leave the `specs/025-outfitter-locked-cards` reference in that
  same module doc alone** — it is a directory path, and it is the one permitted
  survivor of this task's grep gate.
  `src/venue.rs`: `const ACTIONS: [&str; 4] = ["Play", crate::shop::TITLE,
  "Collection", "Quit"];` and the three doc comments that name the Outfitter
  (module doc lines ~5 and ~9, `handle_input`'s doc). `src/economy.rs`: two doc
  comments (~39, ~143). `src/app.rs`: four comments/docs (~320, ~749, ~1509,
  ~3178). `src/profile.rs`: the assertion **message** at ~1244 and the comment
  at ~1247. `assets/primer_text.txt:7` and `assets/how_to_play_text.txt:20`: one
  line each. `docs/economy.md`: ~192 and ~311. `README.md`: line ~29, "a shop on
  the map sells" → "a **Card Shop** on the map sells" (the README's only mention;
  T009 owns its other line and the two do not overlap).
  **Deliberately not renamed** — say so in the report so the absences do not
  read as misses: `specs/**` and `DECISIONS.md` (R2 says they keep the word),
  `ROADMAP.md` (not among R2's enumerated sites, mostly the shipped-spec record,
  and roadmap grooming commits to `main` rather than a spec branch — a one-line
  chore if the person wants it), `CLAUDE.md`'s spec-directory reference, and the
  spec-025 path above. **No symbol is renamed**: `ShopState`, `ShopOutcome`,
  `open_shop` and `shop.rs` already say "shop". **No hint line changes**: the
  map's and the venue's `HINT` both say `b shop`, which is the new name's own
  word, so R2's "the key hints where they name it" costs zero lines here.
  **No existing assertion changes value, and here is why that bar is
  satisfiable** (unlike T001's and T006's first drafts): the only test-file touch
  is an assertion *message* in `src/profile.rs`, which is not a value; and
  `"Card Shop"` is the same **nine characters** as `"Outfitter"`, so
  `action_row_width()` is 53 either way and both asset lines keep their exact
  width and count. `overlay.rs`'s `onboarding_texts_are_the_spec_text_and_fit`
  (line count 10, title row, dismiss row) and
  `help_texts_fit_the_minimum_terminal_unclamped` (box unclamped at both fit
  sizes), and `venue.rs`'s action-row and fit tests, therefore all stay green
  **unedited**. If any assertion value does move, stop and report — it means one
  of those width claims is wrong. **No new test**: the claim R2 makes is about
  strings the player reads, and its check is the grep gate plus the shared
  `const`; a test asserting a UI string equals itself is the tautology sign-off
  rejected once already in this spec. Do not run `cargo fmt`.
  (Copies: nothing structural — `src/shop.rs`'s own `pub const`-at-module-scope
  style for `TITLE`'s placement.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim, **with no test file edited except `src/profile.rs`'s one message**;
  `git diff --stat` lists exactly the nine files above and no others;
  **`grep -rniE "outfitter" src/ assets/ docs/ README.md` returns exactly one
  line — `src/shop.rs`'s `specs/025-outfitter-locked-cards` path** (the gate is
  case-insensitive because five of the sites are lower-case prose, `-E` because
  BSD grep on darwin, and scoped to those four paths because `specs/`,
  `DECISIONS.md`, `ROADMAP.md` and `CLAUDE.md` keep the word by decision — a
  repo-wide gate here would be unsatisfiable, which is the failure T006's grep
  gate already taught this spec once); `grep -rn "TITLE" src/shop.rs src/venue.rs`
  shows one definition and one use; the report pastes the gate's output verbatim.
  **PAUSE for the person** (after the Phase 2 re-review, which covers T004a,
  T005a and T005b together): the orchestrator drives the amendment
  re-walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` pointed at a scratch directory, confirmed in the report — at
  89×31 and again at 139×31, and reports in plain language: that the venue now
  reads as bands, four rows of text at the top, a large bordered art region with
  the opponent's portrait in its own column beside it, and the action row and
  hint at the foot; that the art region is the biggest thing on the screen at
  **both** sizes and visibly bigger at the wider one, with nothing clipped and
  nothing overlapping, and the action row still with an empty row above and
  below it; that the series row reads `Series 0 – 0 · Best of 3`; that the
  middle action says **Card Shop** and opens a screen headed **Card Shop**, with
  Esc returning to the venue; and **the one thing Phase 2 never attested** (the
  driver could not deliver Tab then): at the venue press `c`, remove a card so
  the deck is invalid, Back, then Play — the deck builder opens and Back from it
  returns to the venue.
  **Say plainly what is not being re-walked**: the whole Phase 2 flow the person
  already attested — the lock, the routing, the wager, the settlement, quitting
  and resuming — is untouched by these three tasks and is not re-run.
  **One question, re-put rather than asked fresh**: the art region's placeholder
  is still the planet's name, now in a much larger box — emptier, or as it is?
  It was asked at the Phase 2 pause, the person's amendment records it as
  unanswered, and it is one line either way (plan §Open questions 2).*

## Phase 3 — The series where the player already looks (walkthrough: the map's planet detail names what a launch commits you to before you take it, the status band carries the running score through every match of a series and nothing during a rematch, and the map banner after the deciding match names the series result beside the credits)

- [ ] **T007** — `src/campaign_map.rs` + `src/app.rs` + `docs/economy.md`: the map's detail line and
  the banner. Per plan §Design 7: `MapBanner::Settled(StakeOutcome)` becomes
  `Settled { outcome: StakeOutcome, series: SeriesOutcome }`; `banner_line`
  prefixes `★  Series won  ·  ` / `Series lost  ·  ` for `Won`/`Lost` and
  prints **exactly today's string** for `NotInSeries` and `Continues` (the
  plan's table); the emphasis rule is unchanged (a won stake is Strong). In
  `draw_panel`, draw `series_length_label(opponent)` right-aligned on the
  `{name} · {region}` row (`panel.x1 - 2`, Muted) **only when the planet is not
  cleared** — a rematch commits the player to nothing; the opponent is the
  planet's `launchable_opponent`. In `app.rs`, the one `MapBanner::Settled`
  construction carries `settlement.series`, and `stake_to_show`'s pattern moves
  to the struct variant. Nothing else changes.
  Tests, in `campaign_map.rs`'s tests module:
  `the_banner_names_the_series_beside_the_settlement` (the four cases of the
  plan's table, asserting the two series-free cases are byte-for-byte today's
  strings) and `the_detail_row_names_the_series_length_only_before_a_clear`
  (`Best of 3` for an un-cleared planet, `Best of 5` for Zenith, nothing for a
  cleared one). Keep `the_banner_and_the_run_tally_report_the_same_net_gain`
  passing (it constructs a `Settled` — update its literal only).
  **`docs/economy.md` rides with it, doc only** (added by the orchestrator at the
  Phase 2 review, 2026-09-21, from a finding the review returned). T006 removed
  `launch_campaign_node` and landed the renames, which falsified two places that
  task could not fix without breaking its own diff-stat gate: ~63 still names
  `App::launch_campaign_node`, which is `open_wager` now, and ~154–157's
  parenthetical still says "the renames land with the venue screen later in that
  spec, and until then the code reads `enter_campaign_map` and
  `open_campaign_map`", which is false as of T006. Correct both; the names
  themselves already match, so this is the temporal sentence and one dangling
  identifier, not a rewrite.
  Do not run `cargo fmt`. (Copies: `campaign_map.rs`'s own `banner_line` and
  `axis_line` for the pure-wording-function idiom.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/campaign_map.rs`,
  `src/app.rs` and `docs/economy.md` (the last added at the Phase 2 review — see
  the paragraph above); the report quotes `banner_line` and the detail-row draw
  call verbatim, and pastes the `docs/economy.md` diff.*

- [ ] **T008** — `src/board.rs` + `src/app.rs`: the series score during a match.
  Per plan §Design tension 6 and §Design 8: in `app.rs`, the pure
  `fn board_series_line(node: Option<&NodeRef>, series: Option<&Series>) ->
  Option<String>` with the plan's doc — `Some` only when the in-flight node is
  the series' node, so a rematch, a Quick Play match started mid-series, and a
  stale pointer all give `None`; `App::draw` computes it and passes
  `Option<&str>` into `BoardView::draw`. In `board.rs`, `draw` takes a `series:
  Option<&str>` parameter beside `stake` and draws it **right-aligned on
  `self.layout.status` row 1** (`Emphasis::Strong`) at **both** widths — outside
  the `opponent_panel` match, since the status band exists on both. Nothing else
  in `board.rs` changes; every existing `BoardView::draw` call site (including
  the tests' `drawn_board` helper) passes `None` unless it is testing this.
  Tests (plan §Tests): in `app.rs`, `the_board_shows_a_score_only_for_a_series_
  match` (the four cases above — **pure, no `App`**); in `board.rs`,
  `the_series_score_fits_beside_the_longest_turn_prompt` (at both
  `Config::fit_sizes()`, the longest `status_message` plus the widest drawn
  series line leave at least one blank cell inside `layout.status.width()`), and
  a drawn-frame assertion at 89×31 in the shape of
  `the_compact_board_carries_the_stake_clear_of_the_alert`, one row down: the
  line ends at `status.x1` on row 1, the prompt ends left of it, the cells
  between are blank. Do not run `cargo fmt`. (Copies: `board.rs`'s
  `the_compact_board_carries_the_stake_clear_of_the_alert` — the assertions lift
  almost verbatim; `app.rs`'s `stake_to_show` for the derive-from-state idiom.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with both new tests passing; `git diff --stat` shows only
  `src/board.rs` and `src/app.rs`; the report quotes `board_series_line` and the
  new draw call verbatim, and states the two character counts (longest prompt,
  widest series line) against the 81-cell band.
  **PAUSE for the person** (after the Phase 3 review): the orchestrator drives
  the Phase 3 walkthrough in plan §Verification — scratch `KAAZAP_DATA_DIR`,
  both widths — and reports in plain language: that the map's planet detail says
  what a launch commits you to before you take it; that the running score is
  readable during every match of a series at 89 columns with nothing
  overlapping the turn prompt, and at 139; that a rematch shows no score; and
  what the map banner said after the match that decided a series.
  **One question for the person in this report** (plan §Open questions 1,
  which sign-off confirmed is not an AC 11 failure but is an inconsistency):
  after a match that does **not** decide the series, the band shows the
  freshly-updated score while the game-over popup is up; after the match that
  **does** decide it — won or lost — the band shows nothing, because the series
  is over by then. Show them both frames and ask what the last one should say.
  Holding the final score there is a sub-lettered task, not a redesign.*

## Phase 4 — What the player is told (walkthrough: start a fresh campaign and read the first-campaign primer, then open How to Play — both now state the two-of-three rule, the three-of-five final, and that a series once started is played out)

- [ ] **T009** — `assets/primer_text.txt` + `assets/how_to_play_text.txt` +
  `src/overlay.rs` + `README.md`: the rule, written down. Per plan §Design 9
  and §Files: add the plan's
  three lines (preceded by a blank row) to the primer after the Outfitter
  paragraph, and the plan's **two lines only** to How to Play's campaign
  paragraph. **How to Play has a hard ceiling of 27 lines** — `height + V_PAD`
  must stay ≤ 31 or `help_texts_fit_the_minimum_terminal_unclamped` fails — and
  it is at 24 now; the primer has room but must keep its blank row above the
  dismiss line, must not end on a blank line, and must have no two consecutive
  blanks (`onboarding_texts_breathe_only_around_the_dismiss_line`). In
  `overlay.rs`, `onboarding_texts_are_the_spec_text_and_fit`'s expected Primer
  line count moves **10 → 14** — that is this task's deliberate edit, not a
  weakened test — and add two assertions to
  `help_texts_name_the_new_keys_and_nothing_old`: the Primer and How to Play
  each name the two-of-three rule and the three-of-five final. In **`README.md`**
  (~80–81), "Launching a match opens a **wager prompt**" is no longer what
  launching an un-beaten planet does — correct that paragraph to name the venue
  and the series, and leave the rematch sentence true. **Two more lines in the
  same paragraph, added by the orchestrator at the Phase 1 review (2026-09-20)
  from a finding the phase review returned**: (a) ~79–80, "at each you play its
  opponents to clear it" now needs series language — an opponent is taken by two
  matches of three, the final one by three of five; (b) ~84–86, "If your credits
  ever drop below the cheapest ante on the map, the **run is over**" is a rule
  **T002 falsified** — while a series is locked it is the locked opponent's own
  ante, which can be five times that. Neither had an owner: T009 is the only task
  that touches `README.md`, and CLAUDE.md's git conventions put a README change
  describing a spec's behavior on that spec's branch, so both ride here rather
  than reaching the sweep undiscovered. Change no other
  text and no other asset. Do not run `cargo fmt`.
  (Copies: `assets/primer_text.txt`'s own voice — short lines, no more than the
  current widest; `README.md`'s own campaign paragraph for register.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with all three `overlay.rs` fit/breathing tests passing;
  `git diff --stat` shows only the two assets, `src/overlay.rs` and
  `README.md`; `wc -l assets/how_to_play_text.txt` is **≤ 27**; the report
  pastes both assets in full and the `README.md` diff.
  **PAUSE for the person** (after the Phase 4 review): the orchestrator drives
  the Phase 4 walkthrough in plan §Verification — a fresh scratch profile, the
  primer on first campaign entry, then How to Play from the menu — and reports
  the wording of both in plain language, with whether each fit the 89-column
  terminal with nothing clipped and an empty row above the primer's dismiss
  line.*

## Phase 5 — Balance measured, not changed (walkthrough: none — the simulator is an `#[ignore]`d report and `docs/balance.md` is a document; no code the game runs changes, no constant moves, and nothing on screen differs)

- [ ] **T010** — `tests/balance.rs` + `docs/balance.md`: the series rates,
  measured and recorded. Per plan §Design 10: add `fn series_rate(w: f64,
  needed: u32) -> f64` with the plan's doc and closed forms (best of three
  `w²(3 − 2w)`; best of five `w³(6w² − 15w + 10)`), a `series%` column in
  `balance_table`'s report (best of five for `sovereign`, best of three for
  everyone else), and the guard test
  `series_rate_matches_the_closed_form` with the spec's own flagged vectors —
  0.45 → 0.425 and 0.60 → 0.648 at best of three, 0.33 → 0.205 at best of five,
  0.50 → 0.50 at both (tolerance 1e-3). **Change no constant, no deck, no
  roster value and no target**; this task adds a column and a document section,
  nothing else. Then **run the simulator** (`cargo test --test balance --
  --ignored --nocapture balance_table`, which plays 50 × 10 000 matches — expect
  minutes; `KAAZAP_SIM_N` is available if it is unreasonably slow, and the
  report must say which N produced the recorded numbers) and add a **Series
  rates** section to `docs/balance.md` under *The measured curve*: the two
  closed forms, a table of the measured per-match rates converted to series
  rates for the five decks × ten opponents, and two sentences naming what it
  shows — the curve sharpens in both directions, which is what ruling G1 and
  spec 022's gates wanted, and the sovereign's best-of-five is the sharpest
  case. **Do not edit any existing row of `docs/balance.md`** — the one prose
  line that names `cheapest_floor` (~195) was already corrected in T002, so this
  task is purely additive to that file and the bar below is unambiguous. Do not
  run `cargo fmt`.
  (Copies: `tests/balance.rs`'s own `ev_per_match`/`pct` for the pure-report-
  helper idiom; `docs/balance.md`'s *The measured curve* table for the new
  table's shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the guard test passing; the simulator run's output pasted in
  full in the report, with the N used; `git diff --stat` shows only
  `tests/balance.rs` and `docs/balance.md`; `git diff -- docs/balance.md` is
  additions only; `git diff main...HEAD -- src/opponent.rs src/economy.rs` shows
  no changed balance value.*

## Final phase — Spec close-out (walkthrough: none — documentation, mechanical checks and the pre-merge sweep; the person's walkthrough list above is what they walk at this phase)

- [ ] **T011** — Close-out. Draft
  `specs/029-tournament-rounds/closeout-main-docs.md` in spec 028's shape:
  **ROADMAP** — mark tournament rounds shipped as spec 029 (`grep -n -i
  "series\|tournament\|venue\|best of" ROADMAP.md`, read and judge), and add the
  follow-ups this spec deliberately deferred and named: **per-planet venue art**
  (the region is reserved and holds a plain placeholder; author it the way spec
  016's portraits were run — a brief in the repo, the art drawn by a more
  capable tool, Claude Code validating and integrating), **per-planet music**
  (ruling Q, its own spec), **series-aware banter** (an opponent's match-start
  line now fires two or three times in a row), and **re-tuning the curve if the
  measured series rates play badly** (spec 029 measures, it does not change).
  **DECISIONS** — the spec's rulings (A1 "series"; B1 each match staked
  separately; C1 a lost series costs only the stakes; D2 a venue screen; E2 the
  lock; F1 rematches stay single matches; G1 best of five for the final
  opponent only; H1 no new records; I1 as resolved — the map shows the length,
  the venue and the board show the score; J1/K1/L1; M1 as amended **twice** — a
  plain placeholder with the portrait **beside** it, and then the art region
  enlarged to dominate the screen with the text above and below it; **N1
  superseded** — the art draws at every width, not from 139 columns up; O1 the
  locked floor; P1 campaign entry goes to the venue; Q music deferred), **and
  `spec.md`'s *Amendment, 2026-09-21*, which is three rulings of the person's
  own**: R1 the series length reads the map's words (`Best of 3`) rather than a
  second phrase, R2 the shop is the **Card Shop** everywhere the player reads
  it — with the note that earlier specs' documents, `DECISIONS.md` and
  `ROADMAP.md` keep the old word on purpose, since rewriting them would falsify
  history — and R3 the art dominates the screen at every width, with the
  portrait keeping its own column and the cost named (each planet's art must
  work at two quite different sizes, which lands on the deferred art spec). Note
  that R3 cost one struct and one constant (`VenueRail`, `VENUE_ART_W`), deleted
  rather than kept as unconditional wrappers. And the plan's
  design calls: the return target **derived from the lock** rather than
  remembered (and why — spec 015's bug, and that the invariant is held by a
  reviewed grep rather than by the type system), one `reserve_floor` rather than
  two floor calculations, `take_settlement` extending spec 021's exactly-once
  *data* property to cover the series tally and `mark_beaten` — **and what it
  still does not cover** (`record_match`, which runs before settlement), the
  series stored beside the in-flight pointer rather than inside it (so a Quick
  Play match cannot end a series), the required wins derived from the opponent
  id, **the migration rule that `is_opponent_beaten` — not the presence of a
  series — decides whether a settled match can beat its opponent** (sign-off
  B1), and the four renames with their reasons. Record the **O1 side effect no
  walkthrough could reach**: the wager prompt's "run is over" warning now fires
  at much lower stakes while locked against a deep opponent, because the reserve
  is that opponent's ante rather than the map's cheapest. Record whatever the
  person ruled at the pauses on plan §Open questions 1 and 2, and on the two
  venue questions the Phase 2 report asked (the art label, the credit balance). Run `cargo test -q` three
  consecutive times and paste the tails. Mechanical checks (three-dot, since
  `main` may move): `git diff main...HEAD --stat` lists none of `src/game.rs`,
  `src/card.rs`, `src/player.rs`, `src/save.rs`, `src/opponent.rs`,
  `Cargo.toml`, `Cargo.lock`; `grep -n "VERSION" src/save.rs src/profile.rs`
  still reads 1 and 1; `grep -rn "serde(default" src/campaign.rs` shows the two
  new fields among the old; `grep -rn "cheapest_floor" src/ tests/ docs/`
  matches only `src/economy.rs`;
  `grep -nE "self\.screen = Screen::(CampaignMap|Venue)" src/app.rs` returns
  exactly two lines, both in `open_campaign_home`; `grep -rn "wager prompt"
  README.md` no longer claims a launch opens one; `cargo build --all-targets`
  warning count equals `main`'s (compare via a throwaway `git worktree` with its
  own `CARGO_TARGET_DIR`, as spec 028's close-out did — never switch the branch).
  Check off `spec.md`'s 20 acceptance criteria with evidence (the Phase 2, 3 and
  4 walkthrough reports are the evidence for the venue, the routing, the board
  and the texts; T010's simulator output for criterion 19). Request the pre-merge
  sweep; apply `closeout-main-docs.md` on `main` after the merge. Never chain a
  file edit, a branch switch and a commit in one shell command (spec 025's miss).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/029-tournament-rounds/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product owner**:
the person owns `spec.md`, attests by using the app at phase pauses, and
receives spec-conformance summaries — not architecture reviews.

**Model policy — the economy profile** (this project's since 2026-09-19): every
row of the role table resolves to `opus` / `claude-opus-5`, **no dispatch
carries a per-call model override**, the session runs at `claude-opus-5` medium
from `.claude/settings.json`, and the close-out (T011) goes to
`sdd-implementer` like every other task. Nothing runs on Fable; do not move a
role back to it unless the person says so.

Dispatch each task to `sdd-implementer` on a shell-assembled task bundle (task
line, plan section, acceptance criteria, files, the pattern file to copy).
Verify from the implementer's verbatim output, except **T001 and T003
(`review: per-task`)**: the orchestrator re-runs the verification command itself
and dispatches a `skeptical-reviewer` on that task's diff alone before the next
task starts.

**Foundational phase: Phase 1.** One `skeptical-reviewer` pass at the end of
each phase — after T003, T006, T008, T009 and T010 — on a shell-assembled bundle
(the phase diff, the task lines, plan §Design and §Tests, the acceptance
criteria), one review plus at most one re-review. The Phase 1 review also checks
that the only existing tests T003 changed are the two sanctioned kinds. The
Phase 2 review also checks the venue against the constitution's *acted-on
element stands apart* rule and the modal's even padding, and runs the
single-assignment grep itself rather than taking T006's report for it.

**Pause cadence — when there's something to try.** Pause for the person after
**Phase 1**, **Phase 2**, **Phase 3** and **Phase 4**, once each phase's review
and its walkthrough are done. **Only Phase 5 and the close-out are marked
`walkthrough: none`** — the simulator is `#[ignore]`d and `docs/balance.md` is a
document, and the close-out is documentation, checks and the sweep. They run
through without a pause and append their one-line reason to the walkthrough list
below; their criteria are pinned by tests and by the close-out's own checks.

**Phase 1's marking changed at sign-off, and the change matters.** The first
draft marked it `walkthrough: none` on the strength of a settlement rule that
blocking finding B1 replaced. Under the fix, a match against an un-beaten
opponent with no series running *starts* one — so from T003 onward a single
campaign win no longer beats anyone, which is the spec's central rule becoming
true three tasks before the venue exists. That is observable, so it is marked
`yes` and paused. Four pauses, not three.

**Every driver session runs with `KAAZAP_DATA_DIR` pointed at a scratch
directory**, and the report says which. These walkthroughs play campaign
matches, lose stakes and reset runs; the real profile and saves must never be in
play.

Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`) change only via
`closeout-main-docs.md` on `main` after the merge; `docs/balance.md` and the
assets ride in on the branch. Never run `cargo fmt`. A task that isn't routine
goes to a decision review (`skeptical-reviewer`, top tier — which on this
profile is the same model, no override); a product question `spec.md` doesn't
settle goes to the person. A phase pause ends with what to check and how to say
continue — no continuation prompt unless the person asks or says they're
stopping.

---

## Walkthrough list (what the person walks at the close-out)

<!-- Every phase appends here as it completes: a paused phase appends what the
person already tried and attested, an unpaused phase appends either its
walkthrough items (for the person to try at the close-out) or its one-line
reason for having none. Skipping a pause defers the person's check; it does not
remove it. The orchestrator writes these rows; nobody else. -->

| Phase | Paused? | What the person walks — or why there is nothing |
|---|---|---|
| 1 — The series in the run | **yes** | Beat Cinder's opponent once — the planet does **not** clear and the map offers him again; beat him a second time and it clears and Scree and Ashfall unlock. A lost series costs only its stakes. Driven and attested by the orchestrator before the pause (see the tier log row), then re-attested by the person. |
| 2 — The venue and the lock | **yes** | Enter on an un-beaten planet opens the venue at 0–0 with nothing staked; matches play from it and return to it; the Outfitter and the collection come back to it; quitting to the menu and re-entering lands back on it and the map is never shown; the deciding match hands you back to the map. Driven and attested by the orchestrator at **both** widths before the pause; two questions put to the person. |

| plan + tasks re-review (skeptical-reviewer) | opus → opus | 92K | 1 | — | **1 blocking (B6)** | B1's reshaped rule verified correct on all four cases (locked node, beaten rematch, un-beaten node with no series, Quick Play — which cannot reach it, since `resolve_match` returns `None` with no in-flight pointer); the claimed property and the Phase 1 marking flip both confirmed. **B6**: the B4 fix widened T002's grep to `src/ tests/ docs/` without re-running it — `src/wager.rs` names `cheapest_floor` twice, including a production doc asserting the pre-O1 rule, while plan §Files listed `wager.rs` under No change. T002's and T011's gates were unsatisfiable. Nine second-look notes |
| B6 + second-look fixes (the orchestrator) | opus → opus (session) | — | — | — | — | The review-loop cap was spent, so the orchestrator applied these rather than opening a third pass, per CLAUDE.md. **B6**: `src/wager.rs` moved out of plan §Files' No change as a doc-only edit owned by T002 (correct the `reserve` doc and the test-helper doc to name `reserve_floor` and the locked-opponent floor); exempting it from the grep was rejected as satisfying the gate while leaving the false claim standing. **Second-look, applied**: T002's caller count seven → eight and the file list gains `wager.rs`; T001's literal-site count twice → three times (~397, ~414, ~421); T001's Verify gains the one named carve-out for `take_stake_empties_the_escrow_exactly_once`, whose assertions change shape by design (same contradictory-gate class as B2); T003's two-kinds bar gains the helper-mutation clause that was its one leak; both `self.screen` greps switched to `grep -nE` for BSD grep on darwin; the tier-log header corrected after the Phase 1 flip; the Phase 1 pause text now warns that the lock does not exist until Phase 2, so switching opponents mid-series silently replaces the series. **Second-look, deliberately left open → pre-merge sweep**: (8) `no_settled_match_beats_an_unbeaten_opponent_outright` lives in `campaign.rs` but the property it names is jointly pinned with T003's test, so the name overclaims for where it sits; (9) the `\|\| (NotInSeries && player_won)` clause in `beats` is documented as provably a no-op, and a branch whose own comment says it does no work is the mild smell CLAUDE.md's Simplicity section names |

---

## Notes for the close-out (T011), gathered during implementation

<!-- Non-blocking observations from per-task and phase reviews. Not code
changes: they belong in `closeout-main-docs.md`'s DECISIONS entry or in a
roadmap follow-up, so the next person to touch this code finds them. -->

**From T001's per-task review (2026-09-20, no blocking findings):**

1. `src/profile.rs` ~328 and ~363 — `resolve_match`'s and
   `settle_campaign_match`'s docs still say `None` means "a Quick Play match".
   After T001 `None` has a second meaning, an already-settled match, which is
   what the three amended tests now pin. Outside T001's enumerated footprint,
   so: correct in **T003**, which reworks this seam anyway, or at the sweep.
2. `src/app.rs` ~2877 — the test `NodeRef` carries `settled: false` while the
   test's own header says the match has already settled, so it models a state
   production cannot produce. Harmless (nothing on that path reads `settled`,
   and `stake_at_risk` filters on `stake > 0`), and it is an artifact of T001's
   "every literal gains `settled: false`" instruction rather than drift. Sweep.
3. The orchestrator's T001 re-run evidence was shown as `--lib` plus a warning
   count rather than the constitution's full command. The reviewer checked the
   8 integration targets itself — none constructs a `NodeRef` or asserts on
   profile JSON text, so nothing was missed — but the **T003 per-task review
   bundle must carry the full command's verbatim output**.
4. `wins_needed_is_two_except_for_the_final_opponent`'s loop computes its
   expectation with the function's own expression, so the loop body alone would
   survive a wrong `FINAL_OPPONENT`. The test is rescued by its other two
   assertions (`series_length_label("greeb")` and the `PLANETS` tail check).
   Non-vacuous as a whole; noted so nobody trims it to the loop.
5. `no_settled_match_beats_an_unbeaten_opponent_outright` pins only the
   biconditional its task line asked for; the other half of the property it is
   named for lives in two sibling tests. **Carries a warning for T003**: it
   reads `is_opponent_beaten` *after* the call, which is sound only because
   `record_series_match` marks nobody beaten. If `mark_beaten` ever moves into
   it, that assertion silently changes meaning. (This is second-look note (8)
   from sign-off, now with a concrete reason it matters.)
6. plan §Design tension 3's "nowhere in production" claim for the
   `Some(Won(0))` → `None` change rests on inspection of the
   `phase_changed && GameOver` discriminant edge, not on a test. Pre-existing
   and recorded in the plan; **one line on the close-out walkthrough list**
   rather than a new task.

**From T002 (returned by the implementer, resolved by the orchestrator):**

7. plan §Tests' `can_afford` illustrative numbers were arithmetically impossible
   (it asked for `can_afford(20)` to be true with no series, but the free floor
   is 10, so `20 >= 20 + 10` is false). The plan bullet is corrected in place
   with a dated note; the test asserts the same claim at `can_afford(10)`, where
   the locked and free floors actually differ.
8. The grep gate forced `docs/economy.md` to stop naming the still-existing test
   `cheapest_floor_is_the_min_over_launchable_nodes` (the name contains the
   string). The guards list describes it in prose instead. If a later spec wants
   the exact name back in the document, the gate has to be relaxed or the test
   renamed. **Sweep should decide** whether that is worth doing.
9. `docs/economy.md` now names `enter_campaign` / `open_campaign_home` ahead of
   T006, with a parenthetical saying the renames land later in this spec. **If
   T006 lands different names**, that paragraph and the venue sentence under
   "The shop and its reserve" are the two places to recheck.

**From T003's per-task review (2026-09-20, no blocking findings):**

10. `the_deciding_match_and_the_series_agree` never plays a **mixed** series —
    each series is all wins or all losses, where "the deciding match's result
    equals the series' result" is structurally unavoidable. It is what plan
    §Tests asked for ("over both lengths and both winners") and it is not
    vacuous (the `Continues` arm is real), but it cannot fail for the reason the
    claim exists: §Design 7's mixed case, won the match and lost the series.
    **T007's `banner_line` leans on this claim**, so the sweep should weigh
    adding the two-line win-loss-win case.
11. `resolving_a_match_twice_pays_beats_and_counts_once` infers "beats once"
    from the series tally rather than asserting it. One line
    (`assert!(!q.campaign().is_opponent_beaten("cinder", "greeb"))`) would state
    the claim the test is named for.
12. `docs/economy.md` step 1 attributes the `settled`-flag check to step 1 while
    step 2 is the `take_settlement()` that owns it. Accurate in substance,
    muddled in the step split.
13. `docs/economy.md`'s test list was not extended with the new series tests,
    though T002 added `reserve_floor_follows_the_lock` to the `economy.rs` list.
    Outside T003's scoped doc work; **sweep to decide**.
14. The double-resolve test is deliberately silent on `matches_played`: the
    second `resolve_match` still runs `record_match` before settlement returns
    `None`, so match counters **do** double-count. Plan §Design tension 3 says
    so explicitly and says `take_settlement` does not cover it. Noted because a
    reader could mistake the test's name ("counts once") for a claim about
    match counters.

**From the Phase 1 review (2026-09-20, no blocking findings):**

15. **The Phase 1 walkthrough attests a path Phase 2 stops using.** `begin_series`
    has no production caller until T006, so in Phase 1 *every* campaign match
    against an un-beaten opponent goes through the "match left in flight across
    the upgrade" branch. The player-visible result is identical, but **Phase 2's
    walkthrough must re-attest the two-wins rule** rather than treat Phase 1's
    attestation as covering the launch-time `begin_series` path.
16. `is_broke_reads_the_balance_against_the_cheapest_launchable_ante`
    (`src/profile.rs`) now reads `reserve_floor` and its name is accurate only
    for the no-series half it tests. Cosmetic; **sweep to decide**.
17. AC 14 is two-thirds instrumented: `every_reset_clears_the_lock` covers
    `reset_campaign_run` and `reset_to_starter`, while the run-over reset is
    covered by a comment asserting it is the same call. plan §Tests sanctions
    that ("`run_over` reaches the same code"), and `series` is a plain field
    cleared by both resets. Recorded so the coverage statement is honest rather
    than implied.
18. T003 renamed `sweep_run` to `sweep_run_in_series` and changed its behavior,
    where the task line said helpers should *gain* a series form. The rename is
    total, so every call site appears in the diff and the 11 → 22 move is
    visible — which is the property the "gain, not change" bar existed to
    protect. For the record, not as a defect.
19. **The phase review's confinement caveat, for the Phase 2 review to close**:
    the mid-series replacement is unreachable from Phase 2 onward only because
    T006's lock makes the map unreachable while a series runs, and plan §Tests
    deliberately gives that mapping no unit test — it is a grep plus the
    walkthrough. The **Phase 2 review must run the grep itself** and confirm the
    claim, rather than take T006's report for it.
    **CLOSED at the Phase 2 review (2026-09-21)**: the reviewer ran the grep
    itself, widened it three ways to check the gate was not merely narrow, and
    walked every door — the seven `open_campaign_home` call sites, a quit and
    re-entry, a Quick Play match started mid-series, and the deck-builder divert.
    `launch_from_map` is the only production caller of `begin_series` and is
    reachable only from the map arm, so no reachable state feeds the replacement
    branch. Confirmed, not asserted.

**From the Phase 2 review (2026-09-21, no blocking findings):**

20. **A non-deciding match's `Settled` banner is set and never shown.** `app.rs`
    sets it on every settlement, but the venue draws no banner and clears none,
    so a non-deciding match's message is overwritten by the next settlement
    without ever reaching a screen. Nothing is lost to the player —
    `stake_to_show` puts the settled amount on the game-over frame itself — and
    plan §Open question 5 reasons only about `CantCover`, so this case is
    un-discussed rather than decided. **Recorded for T007's banner work.**
21. **Test isolation, two parts.** (a) `App::new` reads the real on-disk
    profile, so suite greenness is machine-dependent and spec 029 *widened* that
    surface: any future `App` test asserting a campaign screen now needs
    `app.profile = Profile::default()` or it passes on whoever's save is on
    disk. The reviewer checked the other `App::new` sites; none is exposed
    today. (b) `Profile::save()` has **no `cfg(test)` guard**, so "replace the
    profile with a default, then drive input" is one `save()` away from
    overwriting the developer's real profile. The one test doing this is safe
    because its keys are all non-dismiss under `Modal::Primer` — but that safety
    rests on a comment and a key list, not a mechanism. **Sweep should decide on
    a scratch data dir for tests.**
22. `ACTION_GAP = 6` in `venue.rs` is a second copy of `app.rs`'s private
    `CHOICE_GAP = 6`, and `action_row_width()` copies `choice_row_width`.
    Plan-sanctioned as "the same idiom", drift purely cosmetic, but nothing
    tests that the two stay equal.
23. **The venue's deck guard is reachable in play and uncovered**: venue → `c` →
    remove a card → Back → venue → Play diverts to the builder. It routes
    correctly, but there is no test and it was not in the Phase 2 script. Added
    to the walkthrough.
24. `open_wager` silently does nothing when `opponent_by_id`/`planet_by_id` miss
    (an `if let` with no `else`), while the venue's `draw` is deliberately more
    tolerant and falls back to `DEFAULT_OPPONENT` — so for the same corrupt
    series the screen renders while Play does nothing. Unreachable via any
    supported profile; the shape is inherited from `launch_campaign_node`. Noted
    only because the two sites now disagree about tolerance.
25. **An evidence-quality note on the orchestrator, not the code.** The Phase 2
    bundle's echoed *label* for the old-names gate omitted `-E` while the command
    actually run used `grep -rnE`, so the label read as a pattern that could
    never match. The evidence was valid and the reviewer re-ran the alternation
    correctly (clean), but a label that misstates its command is worth not
    repeating: echo the command, don't retype it.

---

## Tier log (this spec, under the model policy)

<!-- One row per implementer run and per reviewer invocation; a summary row per
phase the orchestrator fills at that phase's review. Record token usage from
each subagent return and any escape-hatch miss (a task the orchestrator had to
redo, and why). -->

| Task / invocation | Tier (dispatched → ran) | Tokens | Dispatches | First try | Blocking findings | Outcome / miss reason |
|---|---|---|---|---|---|---|
| **Economy profile** — the project's since 2026-09-19; spec 028 was the first spec under it (its log: planning 390K, implementer 891K over 16 dispatches, reviewer 553K over 9, total ≈1.83M, no tier misses). Every role resolves to `opus` / `claude-opus-5`; no per-call override anywhere, including the planner, the sign-off and the close-out; session at `claude-opus-5` medium. **Also the first spec under the `walkthrough:` pause cadence** — the draft marked Phases 1 and 5 `none`; sign-off's B1 fix made Phase 1 observable and its marking flipped to `yes`, leaving Phase 5 and the close-out unpaused. That flip — a marking corrected the moment the thing underneath it changed — is the evidence this log exists to carry. | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | opus → opus | 272K | 1 | — | — | drafted; 11 tasks in 6 phases, Phase 1 foundational, T001 and T003 `review: per-task`, Phases 1 and 5 marked `walkthrough: none`; no product question returned; 5 design choices flagged for sign-off |
| plan + tasks sign-off (skeptical-reviewer) | opus → opus | 127K | 1 | — | **5 blocking** | B1: a match left in flight across the upgrade would beat its opponent outright and clear a world in one match — contradicts an approved `spec.md` clause. B2, B3: two Verify gates mutually unsatisfiable (unnamed `NodeRef` literal sites; three unnamed `cheapest_floor` call sites and a missing `app.rs`). B4: `docs/economy.md` and `README.md` left asserting the old rule, by a spec that renames four symbols *because* false names are defects. B5: the close-out phase header carried no `walkthrough:` marking. All five flagged design choices upheld; 8 second-look notes |
| Planning: sign-off revision (sdd-planner, fresh context) | opus → opus | 64K | 1 | yes | — | All five applied. **B1 carried a knock-on the finding did not state**: once a match with no series starts one, a single campaign win stops beating anyone from T003 onward — so Phase 1 is no longer invisible, its marking flipped to `walkthrough: yes` (**four pauses, not three**), and T003's "no existing assertion changes value" bar became "two sanctioned kinds of change; anything else stops and reports". Also tightened B1's rule from *does a series exist* to `is_opponent_beaten`, closing a mismatched-node hole the literal fix would have left. Second-look: `CampaignHome` **dropped** (its test would be a tautology aimed at the wrong risk — replaced by a `self\.screen = Screen::` grep that catches a second assignment site); the venue credit balance **not** added to the design but routed to the person at the Phase 2 pause; AC 4 and AC 13 given keystrokes in the Phase 2 script, since both rested on assertion. Both files still Draft |
| T001 (sdd-implementer) | opus → opus | 87K | 1 | no — **stopped on a judgment call** | — | Wrote T001's code in full, then stopped rather than adjust three `profile.rs` assertions T001's Verify bar forbade it to touch. **Correct behavior**: the `take_settlement` swap makes a second settlement return `None` where it returned `Some(StakeOutcome::Won(0))`, which those three pin, so the Verify bar and plan §Design tension 3 could not both hold. Not an escape-hatch case — the orchestrator did **not** consider it well-specified, because resolving it meant widening the one gate this spec's sign-off had already had to fix twice (B2, B3, both unsatisfiable Verify gates from under-enumerated call sites) |
| T001 decision review (skeptical-reviewer) | opus → opus | 95K | 1 | — | — | Ruled option (a): the one-exception list is a planner **enumeration error**, not a design constraint — the three second-settlement lines move to `None`, the property they assert (settling twice settles once) is unchanged, only the evidence moves from an emptied escrow to a consumed flag. Gave the exact amended Verify bar (four named exceptions, each with its new value, surrounding credit assertions held to their current values). On the doc/grep conflict: reword the clause to "by zeroing the escrow" and **keep** the grep, rather than weaken a mechanical check into a judgment call. Also **corrected the orchestrator**: T011 does *not* carry the `take_stake` grep, so the fix had one site, not two. Transcribed to `plan.md` + `tasks.md` in `fda7eaf` **before** the next dispatch |
| T001 completion (sdd-implementer, fresh context) | opus → opus | 53K | 1 | yes | — | Applied the ruling's five edits and nothing else. Green: 461 lib tests, 0 warnings, `grep -rn "take_stake" src/ tests/` empty |
| T001 per-task review (skeptical-reviewer) | opus → opus | 86K | 1 | — | **0 blocking** | **Signed off.** Verified the B1 shape (`is_opponent_beaten` is the first test and the only path to `NotInSeries`), that `take_settlement`'s doc does not overclaim (the `record_match` carve-out is present), that `node.stake` — now `0` in the returned clone — is never read at the call site, that the migration tests are non-vacuous at both levels, and that the four sanctioned exceptions are the only assertions that moved in value. 6 second-look notes, recorded above |
| T002 (sdd-implementer) | opus → opus | 108K | 1 | yes | — | All eight call sites moved in one task, `cheapest_floor` private, the grep gate satisfied across `src/ tests/ docs/`, `docs/economy.md` corrected in all seven places plus the O1 consequence paragraph. **Returned three deviations, all sound**: (1) plan §Tests' `can_afford` numbers were **arithmetically impossible** — it asked for `can_afford(20)` to be true with no series, but the free floor is 10, so `20 >= 20 + 10` is false; the implementer asserted the true values and flagged it rather than bending the test, and the orchestrator corrected the plan bullet. (2) `is_broke`/`can_afford`'s own doc comments still claimed "the cheapest ante on the map" — the identical defect B6 raised against `wager.rs`, so corrected. (3) `tests/balance.rs`'s import moved with its two call sites. **Also returned a finding the task did not cover**: `docs/economy.md`'s step 3 still describes pre-029 settlement, which **T003** falsifies — folded into T003's scope with its diff-stat gate widened, rather than deferred to a close-out that lands on `main` |
| T003 (sdd-implementer) | opus → opus | 131K | 1 | yes | — | The task where the campaign's central rule changes. Hit no judgment call; every existing-test change fell inside one of the two sanctioned kinds, and the implementer returned the required per-test list. **Three declared deviations**: `sweep_run` **removed** rather than kept (all three callers moved to `sweep_run_in_series`, so keeping it would be `dead_code` against the no-new-warnings bar — the review confirmed this does not leak the two-kinds bar, because all three call sites appear in the diff as explicit edits); `matches_played` **11 → 22**, the one numeric expectation that moved; two `docs/economy.md` edits slightly beyond the literal "step 3" instruction, both of statements this task falsified |
| T003 per-task review (skeptical-reviewer) | opus → opus | 75K | 1 | — | **0 blocking** | **Signed off.** Enumerated every hunk against the implementer's list and found no unlisted value move. Verified the 11 → 22 arithmetic **against the roster rather than the formula** (8 planets, 10 nodes, 9 × 2 + 3 + 1 = 22), that `mark_beaten` did not move into `record_series_match`, that the call site never reads the now-always-zero `node.stake`, that `record_match` still runs before settlement, and that the rematch path moved no value. 5 second-look notes, recorded above |
| **Phase 1 review** (skeptical-reviewer) | opus → opus | 110K | 1 | — | **0 blocking** | **Signed off.** Walked all eight ACs Phase 1 owns (1, 7, 8, 10, 13-in-part, 14, 15, 20) and found each met by code *and* a test. Verified the seam the per-task reviews could not see — the `settled` flag across a save/quit/resume, which holds because `stake_match` always receives a freshly-built `NodeRef` rather than mutating the old one — and that `reserve_floor` and the wager prompt's floor are one expression rather than two that agree today. On the no-lock question: the replacement is confined to Phase 1 only by a **Phase 2** check that plan §Tests deliberately leaves to a grep and a walkthrough, so **the confinement claim is scheduled, not yet demonstrated, and Phase 2's review must make it good**; no persisted shape Phase 1 can write is one a Phase 2 build misreads. 7 second-look notes, two of which changed the Phase 1 pause report |
| Phase 1 walkthrough (the orchestrator, `run-kaazap`) | opus → opus (session) | — | — | — | — | Driven at **89×31** with `KAAZAP_DATA_DIR` at a scratch directory (`…/scratchpad/kaazap-data`); the real profile and saves were never in play. **Attested**: a win against Greeb left `0/8 cleared` with the opponent un-beaten and the series at 1–0; the series then went 1–1 and was **lost**, clearing the score to nothing, leaving Greeb un-beaten and the planet uncleared, and taking only the two stakes (AC 8); a later series won 2–0 flipped the map to `1/8 cleared` with `beaten: {cinder: [greeb]}`, Cinder drawn as cleared with "Cleared — Enter to rematch Greeb", and Scree and Ashfall unlocked (AC 1, 7). **Two driver notes, for honesty about the setup**: the scratch profile's purse was raised to 2000 credits so a run-over reset could not destroy the walkthrough mid-way (the auto-player loses often against even the rookie), and the driver's kill leaves a save behind, so each match ran after clearing the scratch `saves/` directory. Neither changes what was attested |
| T004 (sdd-implementer) | opus → opus | 55K | 1 | yes | — | `src/layout.rs` only, 117 insertions, no existing line touched. Computed Rects at 139×31: text area 0..=80 with `center_x` 40, `art` 81..=110, `portrait` 114..=135, both rows 8..=22 — matching plan §Design 4's pinned arithmetic exactly. **Two deviations, both sound**: the plan's test claim included "the widest venue text row ends left of `art.x0`", which cannot be asserted before `venue.rs`'s strings exist (T005), so the implementer asserted `center_x < art.x0` and flagged the real version for T005; and `VenueRail`/`VenueLayout` derive no `PartialEq`, since the task's assertion list is field comparisons. **Returned a finding folded into T005**: `block_h` is a parameter, so T004's test hardcodes 8 and would not notice T005 defining a different `venue::BLOCK_H` |
| T005 (sdd-implementer) | opus → opus | 86K | 1 | yes | — | `src/venue.rs` (new, 358 lines) + the one `pub mod` line. One owned `VenueOutcome`; the eight rows built by a single `text_rows` helper that both `draw` and the tests read, so the breathing test cannot drift from what is drawn. Rows 4 and 6 blank around the action row, which is drawn label-by-label at a fixed stride so its width is 53 for every cursor position. **Widest row is the controls hint at 63 characters** — at 139 columns it ends at 71 against `art.x0` of 81, so T004's un-assertable fit claim is now pinned against the real strings with 9 columns of slack. **T004's finding 1 discharged two ways**: `BLOCK_H = 8` plus an assertion that it is 8, naming why. Three deviations, each resolved to an existing convention rather than a new one: `screen.rs` left to T006 per the task line over the plan's heading; `draw` also returns early on an unknown planet id; `ACTION_GAP` is a local const because `app.rs`'s `CHOICE_GAP` is private and this task may not touch `app.rs` |
| T006 (sdd-implementer) | opus → opus | 103K | 1 | yes | — | Both gates pass: the single-assignment grep returns exactly two lines (771, 775), both inside `open_campaign_home`, and the old-names grep is empty. **One deviation that matters, and the implementer was right**: plan §Design 6's listing writes `open_campaign_home` as `self.screen = if … {…} else {…};`, which makes the invariant's own grep return **zero** lines — the listing and the gate contradicted each other. It wrote two assignment statements instead (same behavior, same single function, same inline `if`) and documented in the function why, so the gate reads two. Verified empirically, not argued. **Two findings**: (a) tests that call `App::new` read the **real on-disk profile**, so `the_primer_swallows_map_keys` failed against the person's own Phase 1 play state (a series at 1–1) — fixed with the `app.profile = Profile::default()` pattern already used three times in that module, no assertion weakened, but a latent problem is now live; (b) the wager's `CantCover` refusal is **silent at the venue** and the banner survives into the next map visit — plan §Open question 5 says it is unreachable there, so routed to the Phase 2 review rather than changed |
| **Phase 2 review** (skeptical-reviewer) | opus → opus | 161K | 1 | — | **0 blocking** | **Signed off.** Did all three checks the handoff note requires. (1) Ran the single-assignment grep itself and **widened it three ways** — `self\.screen\s*=` across `src/` (13 hits, only two set a campaign screen), the bare variant names, and `mem::replace`/`&mut self.screen` — confirming the gate is load-bearing rather than narrow, and **upheld T006's deviation** from the plan's listing: in the expression form neither line contains the variant name, so the gate would return zero and pass while checking nothing. (2) Density verified as instrumented rather than asserted, against the constitution's *corrected* form (only the acted-on line gets air), and both modals the venue can raise already pad evenly through `OverlayLayout`. (3) **Closed the Phase 1 caveat** by walking every door, including a quit and re-entry, a Quick Play match started mid-series, and the deck-builder divert — `launch_from_map` is the only production caller of `begin_series` and is unreachable while a series runs. Also **verified plan §Open question 5 independently**: `CantCover` is unreachable from the venue because the venue's floor *is* `reserve_floor` while locked, and the missing banner-clearing line makes nothing worse, because the map's arm clears the banner before `launch_from_map` runs. 7 second-look notes; one gave `docs/economy.md`'s rename fallout to T007 |
| Phase 2 walkthrough (the orchestrator, `run-kaazap`) | opus → opus (session) | — | — | — | — | Driven at **89×31 and 139×31** with `KAAZAP_DATA_DIR` at a scratch directory (`…/scratchpad/kz2`); the real profile was never in play. **Attested**: AC 2 (venue at 0–0, credits still 50 and `in_progress: None` — arriving stakes nothing); AC 3 (the four rows, the four actions, `b` → Outfitter and `c` → collection both returning to the venue, and the Outfitter reading `spendable ◈ 40` = 50 − Greeb's ante, the locked floor); AC 4 (Play → Esc → venue, balance untouched); AC 5 (0–1 and 1–1 both returned to the venue); AC 6 and AC 7 (the second win landed on the **map** at `1/8 cleared` with `beaten: {cinder: [greeb]}`); AC 8 (a deciding loss landed on the map, opponent un-beaten, only the stakes gone); AC 9 (Start Campaign → Continue went **straight to the venue**, map never drawn, twice); AC 12 (cleared Cinder → wager directly, "Tournament Hall" absent); AC 13 (quit at the venue and return → same score; a match killed mid-play resumed at Score 3–1 with the same hand, then its finish decided the series and routed to the map); AC 16 (89 text-only and nothing clipped; 139 drew the bordered art region holding "Cinder" with the portrait beside it, sharing rows 8–22, a clear gap, both on-frame); AC 17 (blanks at rows 15 and 17 only, around the action row). **One item left uncovered**: the deck-guard divert of second-look note 23 — the driver cannot deliver Tab to the app, so the deck could not be made invalid from the venue. The Phase 2 review verified that routing by inspection; the person can try it by hand. **Driver notes**: the scratch purse was raised to 2000 twice so a run-over reset could not destroy the walkthrough, and each match ran after clearing the scratch `saves/`, since the driver's kill leaves one |
| Phase 2 walkthrough — an incidental finding | — | — | — | — | — | With a live series at 0–0 and otherwise starter state, **Start Campaign skips the Continue / New Campaign / Reset Everything panel** and goes straight to the venue, because `Profile::differs_from_starter` reads `campaign.has_progress()` and a series is not progress by that definition. Not a spec violation — AC 9 says entering the campaign goes to the venue, which it does, and AC 14's resets still clear the series — but a player who starts a series and immediately quits has no visible door to New Campaign until they play a match. **For the sweep to rule on** |
