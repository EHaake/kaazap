# Tasks: Tournament rounds — spec 029

**Status**: Signed off (skeptical-reviewer, 2026-09-22 — Phase 6 / R9 revision: one review, one re-review, B1–B3 resolved and all notes applied)
**Implements**: plan.md in this directory

**The R9 revision (2026-09-22).** At the merge pause the person ruled that the
per-planet art is integrated in this spec (ruling **R9**, acceptance criterion
**21**). **Phase 6** (T012–T014) is added for it, and **T011** is revised to
close again after Phase 6 with a second pre-merge sweep scoped to it. Nothing in
Phases 1–5 or T011a is reopened. The art itself arrives from another session as
sixteen files in `assets/planets/`; **T012 runs now, T013 and T014 wait for the
files** (plan §Design 12 says why: `include_str!` cannot compile without them).

T001–T006 were signed off on 2026-09-20 (one review, one re-review, B1–B6
resolved), implemented, reviewed, committed, and attested by the person at the
Phase 1 and Phase 2 pauses. **T004a, T005a and T005b** were signed off on
2026-09-21 for `spec.md`'s *Amendment, 2026-09-21* and are likewise implemented,
reviewed, committed and attested. Nothing about any of them is reopened here.
What is pending sign-off is **T005c and T005d** in Phase 2, added 2026-09-21 for
`spec.md`'s *Amendment, 2026-09-21 (second)* (R4 the art's proportions, R5 the
text's alignment to the art, R6 the credit balance, and the per-planet art
brief), and the second Phase 2 re-walkthrough in T005d's Verify. T007–T011 are
unchanged and were never started.

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
other's. Phase 6 (R9) depends on Phase 2's venue and on nothing else; it is not
foundational.

**Three tasks carry `review: per-task`** — T001 (the persisted data model and the
exactly-once `take_settlement`, which every later task builds on and which a
save file inherits), T003 (the settlement: the one place credits are paid and
an opponent is beaten), and **T012** (R9's geometry: AC 21's validation test
derives its canvas from this layout, so a slip here would be baked into the very
test meant to catch it — and the art's delivery sits between T012 and the rest
of its phase, so a phase-end review would come too late). Every other task is
covered by its phase review.

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

## Phase 2 — The venue and the lock (walkthrough: Enter on an un-beaten planet now opens the venue at 0–0 with nothing staked; play matches from it, come back to it between them, open the Card Shop and the collection and return to it, quit to the menu and re-enter the campaign to land back at it — and win the series to be handed back to the map; then, after T004a–T005b, walk the amended venue: horizontal bands with a dominant art region at **both** widths, `Best of 3` in the series row, and a **Card Shop** button that opens a screen headed the same; then, after T005c, walk the re-proportioned venue: the art about a sixth smaller by area but still the biggest thing on the screen, **every** row of text centred over the art box rather than over the whole screen, a shorter hint sitting clear of the left edge, and a `Credits: ◈ …` row that moves after a match settles — T005d adds the per-planet art brief, a document, with nothing to look at)

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

- [x] **T004a** — `src/layout.rs` + `src/venue.rs`: the venue becomes horizontal
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
  renamed four symbols to avoid. **Two more items the amendment sign-off added
  (2026-09-21), because each would otherwise make this task stop or fail its own
  gate**: (a) `draw`'s **own method doc** (`src/venue.rs` ~179–181) says "Draw
  the **eight-row text block** and, **at 139 columns and wider**, the **right
  rail**…" — false in three ways after this task, by the same standard as the
  module doc, so correct it too; (b) the test helper `fn rows_for` (~250) has
  **exactly one caller**, the breathing test this task converts to a frame test,
  so it becomes `dead_code` and fails this task's own "no new warnings" bar —
  **delete it in this task** (its sibling `rows_for_opponent` keeps its caller
  in the fit test and stays). Neither is a stop-and-report; both are inside this
  task's footprint.
  One more assertion, named so it is dropped on purpose rather than by
  accident (amendment sign-off, 2026-09-21): the replaced layout test's
  `assert!(l.center_x < cols, …)` is in neither list below. Its truth value does
  not move (44 < 89, 69 < 139), so it is **not** a stop-and-report — it simply
  goes with the test being replaced. What still covers what it was guarding is
  the **fit test's** `x + w <= cols`, which measures text placed from
  `center_x`; `in_bounds` on the two Rects does not, since it says nothing
  about `center_x` (corrected at the re-review).
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

- [x] **T005a** — `src/venue.rs`: the series line reads `Best of 3`. Per plan
  §Design 5, for amendment **R1**. `series_line` formats `"Series  {} – {}   ·
  {}"` with `campaign::series_length_label(&series.opponent)` in place of
  `wins_needed(&series.opponent)` — the same function the map's planet detail
  will use once **T007** lands it, so the player meets one phrase for one idea. Drop the now-unused
  `wins_needed` import and **add `series_length_label` to the
  `crate::campaign::{…}` list** (keep `Series`); `wins_needed` itself stays
  where it is and keeps its other callers. `series_line`'s doc comment contains
  an intra-doc link ``[`wins_needed`]`` which becomes broken — `cargo build`
  will **not** catch that, only `cargo doc` would, so replace the doc with plan
  §Design 5's text rather than leaving the link dangling. (Both added at the
  amendment sign-off, 2026-09-21.)
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

- [x] **T005b** — the Card Shop rename: `src/shop.rs` + `src/venue.rs` +
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
  read as misses: **earlier** specs' directories and `DECISIONS.md` (R2 says
  they keep the word — they record what those specs did). **Corrected at the
  amendment sign-off, 2026-09-21**: the first draft carved out all of `specs/**`,
  which swept in **spec 029's own `spec.md`** and would have left acceptance
  criteria 3 and 10 naming a button this task deletes — the document-level form
  of the defect R2 exists to remove. The orchestrator has since corrected that
  file's live prose (the person had already ruled R2), leaving its dated ruling
  records and its spec-012 reference with the old word on purpose. This task
  still renames nothing under `specs/`;
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

<!-- The two tasks below are the 2026-09-21 **second** amendment (R4 the art's
proportions, R5 the text's alignment, R6 the credit balance, plus the per-planet
art brief). Everything above this line is done, reviewed, committed and
attested, including the first amendment's T004a/T005a/T005b.

Order is fixed in one direction: **T005c first**, because T005d's canvas numbers
are derived from the geometry T005c lands. T005c is one task across two files
for the same reason T004a was — `VenueLayout`'s shape and `venue.rs`, its only
consumer, cannot land in separate builds — and because R4, R5 and R6 are
entangled: R6's extra header row is one of R4's two axes, and R5 re-derives the
column R6's row is drawn from. Split three ways, the pinned Rects would be wrong
in two intermediate commits.

**Neither carries `review: per-task`**, on the same reasoning T004a's comment
records: the criterion is a task whose mistake later files would inherit, and
nothing outside `venue.rs` reads `VenueLayout` while Phases 3–5 touch none of
it. T005d touches no code at all. The phase **re-review** covers both diffs
together, and T005c's enumerated list of changed assertions is what that review
checks against — the same instrument that worked at T004a, where for the first
time in this spec a deliberate assertion change caused no stop.

The walkthrough list near the foot of this file gets a third Phase 2 row when
the second re-walkthrough is attested — the orchestrator writes it. -->

- [x] **T005c** — `src/layout.rs` + `src/venue.rs` + `src/shop.rs`: the venue's proportions, its
  alignment and its balance. Per plan §Design 4 and §Design 5, for `spec.md`'s
  amendments **R4**, **R5** and **R6**. Two files in one task because
  `VenueLayout`'s shape changes and `venue.rs` is its only consumer; a third,
  `src/shop.rs`, added at the sign-off for the one-line `credits_label` hoist
  R6's row reads.

  **`src/layout.rs`.** `VenueLayout::HEADER_H` becomes **5** (the credit row,
  R6) and its doc names the fifth row and says the row gets **no** air — only
  the acted-on row does. Add `const ART_W_NUM: usize = 7;` and `const
  ART_W_DEN: usize = 8;` with plan §Design 4's doc, including *why a fraction
  and not a fixed inset* (no single column inset lands inside R4's 15–20 % at
  both fit sizes — the ranges are 7–9 columns at 89 and 12–17 at 139, and they
  do not overlap). Rename the field `center_x` to **`text_x`** and derive it
  from the art — `(art.x0 + art.x1) / 2` — with plan's doc: it is the art's
  centre, not the terminal's, and it is named for what it is *for* because
  `center_x` reads as the terminal's centre, which is exactly what R5 stopped
  using. **Do not rename `MenuLayout::center_x` or `BriefcaseLayout::center_x`**
  in the same file: they are other screens' and they really are the terminal's
  centre. `VenueLayout::new` computes plan §Design 4's arithmetic in that order:
  the portrait's column unchanged, then the art's available span
  (`MARGIN_X ..= portrait.x0 - PANEL_GAP - 1`), then `art_w = span_w *
  ART_W_NUM / ART_W_DEN`, then `art.x0 = span_x0 + (span_w - art_w) / 2` — the
  trim **split between the art's two margins**, so the art stays centred in its
  span and `text_x` does not move with the fraction. Keep the `.max()` clamps.
  Three doc comments in this file become false and change with it: `MARGIN_X`'s
  ("the art's left margin and the portrait's right margin — **equal**, so the
  two regions together sit centered under the header rows" — the art's left
  margin is 7 at 89 columns now, the margins are deliberately unequal, and the
  band no longer has to be centred under anything because the text follows the
  art); the `VenueLayout` struct doc's "Four header rows at the top"; and the
  comment inside `new` that says the art "takes everything left of the gap
  before it".

  **`src/venue.rs`.** `HINT` becomes `"←/→ choose · Enter confirm · b shop · c
  deck · Esc menu"` — **55 characters**, the board's own single-space `·`
  separators rather than the venue's `  ·  ` (plan §Design 4's R5 decision: at
  89 columns, centred on the art's centre of 31, the old 63-character form spans
  columns 0..=62, flush against the frame edge, which `spec.md` forbids in as
  many words; the new form spans 4..=58). `header_rows` gains a fifth parameter
  `credits: u32` and a fifth row that **is `shop::credits_label(credits)`'s
  output**, not a second literal of it (added at the sign-off, 2026-09-21):
  hoist `Credits: ◈ {credits}` out of `shop.rs`'s inline balance `format!` into
  `pub fn credits_label(credits: u32) -> String`, have the shop's own row use it
  plus its ` · spendable ◈ …` clause, and have the venue read it. **This is the
  third label this screen borrows and the third time the answer is one source
  rather than two that agree** — R1 took `series_length_label`, R2 took
  `shop::TITLE`, and the sign-off found R6 about to take the option both
  rejected, with nothing able to catch a drift (`shop.rs` builds its string
  inline in `draw`, so no test reaches it, and there is no test for R6). — the Card
  Shop's own opening words (`src/shop.rs`'s balance line), not a second
  phrasing, because the venue's balance row exists to inform the trip to that
  screen. `draw` reads `profile.credits()`, draws the fifth header row at
  `top + 4` with **`Emphasis::Normal`** (not Muted — it is information the
  player acts on, and not Strong, which the planet name owns), and its single
  `let cx = layout.center_x;` becomes `let cx = layout.text_x;`. Nothing else in
  `draw` moves. Correct the two docs that become false: `header_rows`' "The
  **four** header rows above the art" and `draw`'s own "Draw the venue's three
  bands: the **four** header rows" — both say five, and `header_rows`' doc says
  what the fifth is. **The placeholder's contents do not change**: the person
  closed plan §Open questions 2 on 2026-09-21 — the region keeps the planet's
  name — so this task changes the region's *size* and the column its label
  centres on, and nothing about the label.

  **The numbers this task must produce**, from plan §Design 4's table, so the
  report can be checked against them rather than trusted: at **89×31** `art ==
  Rect::new(7, 56, 5, 26)` (50×22 = **1100** cells) and `portrait ==
  Rect::new(64, 85, 5, 19)` (330); at **139×31** `art == Rect::new(10, 103, 5,
  26)` (94×22 = **2068**) and `portrait == Rect::new(114, 135, 5, 19)`.
  `text_x` is **31** at 89 and **56** at 139 (the terminal's centres, 44 and 69,
  are no longer used anywhere). Against R3's 1334 and 2484 that is **17.5 %**
  and **16.8 %** smaller — R4's "about 15–20 % smaller overall", at both sizes
  and within 0.8 points of each other.

  **This task moves numbers two shipped tests pin, so existing assertions change
  value — deliberately, and only in the ways enumerated below. Anything else
  that moves is a stop-and-report, not an expectation to adjust.** The list is
  exhaustive and this spec has paid four times for lists that were not.

  (1) **`src/layout.rs`, `the_venue_bands_stack_and_the_art_takes_the_rest`:**
  • `assert_eq!(l.portrait.x0, l.art.x1 + PANEL_GAP + 1, "art and portrait
  aren't exactly PANEL_GAP apart…")` — **deleted**. Old bar: the gap is
  *exactly* `PANEL_GAP`. New fact: R4's trim adds its right half to the gap, so
  the clear columns are **7 at 89 (57..=63) and 10 at 139 (104..=113)**. Its
  surviving half is the line immediately above it (`art.x1 + PANEL_GAP <
  portrait.x0`, "at least `PANEL_GAP` clear"), which **keeps its value**, and
  the exact counts follow from the two pinned Rects. Dropped on purpose, not by
  accident.
  • `assert_eq!(l.art.x0, 3, "art's left margin at {cols} columns")` —
  **replaced**. Old value: 3 at both widths. New: the art is centred in its
  span, so it is **7 at 89 and 10 at 139**. The new assertion is the rule rather
  than the number — `assert!(l.art.x0 >= VenueLayout::MARGIN_X, …)` (the const
  is visible to the test module; second-look note 28's art half, resolved here
  because this line has to be rewritten anyway) — with the concrete values
  carried by the Rect pins below.
  • the four pinned Rects — **all four change value**: at 89×31 `art`
  `Rect::new(3, 60, 4, 26)` → **`Rect::new(7, 56, 5, 26)`** and `portrait`
  `Rect::new(64, 85, 4, 18)` → **`Rect::new(64, 85, 5, 19)`**; at 139×31 `art`
  `Rect::new(3, 110, 4, 26)` → **`Rect::new(10, 103, 5, 26)`** and `portrait`
  `Rect::new(114, 135, 4, 18)` → **`Rect::new(114, 135, 5, 19)`**. (The
  portrait's *columns* do not move — only its rows, by the one row R6 adds.)
  • **one assertion added**: `text_x` is the art's centre —
  `assert_eq!(l.text_x, (l.art.x0 + l.art.x1) / 2, …)` and pinned
  `assert_eq!(l.text_x, if cols < WIDE_LAYOUT_MIN_WIDTH { 31 } else { 56 }, …)`,
  with the terminal's own centres (44, 69) named in the comment as what these
  are *not*. R5's whole ruling rests on this, so it is asserted rather than left
  to follow from the Rects.
  • **kept, values unchanged, and a stop-and-report if any moves**: the
  `Config::fit_sizes()` loop; both Rects `in_bounds`; `l.header_y == 0`;
  `l.art.x1 + PANEL_GAP < l.portrait.x0`; `l.portrait.x1 < cols`;
  `l.portrait.x1 == cols - 4`; `l.portrait.height() == VENUE_PANEL_H`;
  `l.art.y1 < l.action_y`; `l.art.y1 + 2 == l.action_y`; `l.action_y + 2 ==
  l.hint_y`; `l.hint_y == rows - 1`.
  • **symbolic assertions whose value moves without their text changing**, named
  so the review is not surprised and so nobody "fixes" them:
  `l.header_y + VenueLayout::HEADER_H == l.art.y0` (4 → 5 on both sides) and
  `l.art.y0 == l.portrait.y0` (4 → 5). **No edit.**
  • the test's own comment says "four header rows" — **five**. **And the rest of
  that same comment block** (added at the sign-off, 2026-09-21): its opening
  sentence claims "the two regions' outer margins are equal so the band sits
  centered under the header" — **both clauses are false** after this task (the
  width is seven eighths, and R5 makes the text follow the art rather than the
  band sit under centred text). Rewrite the sentence, not just the one word;
  editing "four" to "five" and leaving the next clause wrong is the defect this
  spec renamed four symbols to avoid.
  • **`src/layout.rs` ~808–809, the inline comment above the deleted
  assertion** (sign-off, 2026-09-21): "Art then panel, exactly `PANEL_GAP` clear
  columns apart, with equal outer margins and the panel clear of the edge."
  After this task the clear columns are **7 at 89 and 10 at 139** and the
  margins are **7 vs 3**. Correct it; it is the statement plan §Design 4
  explicitly contradicts ("the margins are no longer equal — deliberately").
  • `assert!(l.portrait.y1 <= l.art.y1, …)` — **kept, text unchanged**; its
  operands move 18 → 19 while the assertion stays true. Listed (sign-off,
  2026-09-21) because it belongs in the symbolic category above and its absence
  would have an implementer working the list literally hit an unlisted assertion
  whose values moved — and this task's own rule then tells them to stop.

  (2) **`src/layout.rs`, `the_art_region_dominates_at_both_widths`:**
  • `assert!(art_area > portrait_area, …)` and `assert!(wide > narrow, …)` —
  **kept, text unchanged**; their values move (1100 and 2068 against 330, and
  1100 → 2068).
  • **(also `src/venue.rs`, added at the sign-off)** `draw`'s inline comment
  "The header band, compact: place, planet, opponent, series." must name
  **credits** — a third row-list beyond `header_rows`' doc and `draw`'s own doc,
  which the list named as "the two docs that become false".
  • **R4's band added**, because "about 15–20 % smaller" is the claim the person
  will eyeball and a plan sentence must not stand in for it: at each fit size,
  with R3's figure as a literal (`1334` at 89, `2484` at 139), assert
  `80 * old <= 100 * art_area && 100 * art_area <= 85 * old`. Integer
  arithmetic, no floats. This fails if the fraction, `HEADER_H` or `FOOTER_H` is
  ever touched without re-checking R4, which is exactly what it is for.
  • the test's comment currently states 1334/2484 as the *current* areas —
  **false after this task**; it states 1100/2068 and the two percentages.

  (3) **`src/venue.rs`, `the_venue_rows_breathe_only_around_the_action_row`:**
  • the blank-row assertions at `action_y - 1` and `action_y + 1` (rows 27 and
  29) — **unchanged in text and in value**.
  • the filled-rows loop over `layout.header_y .. layout.header_y +
  VenueLayout::HEADER_H` — **unchanged in text**, and this is how R6 gets
  checked: with `HEADER_H` at 5 the loop now requires row 4, the credit row, to
  carry text on the drawn frame. Its coverage grows without an edit; **no new
  test for R6**, and say so in the report so the absence does not read as a gap.
  The `Profile::default()` this test builds has the seed purse, so the row is
  non-blank without any extra setup.

  (4) **`src/venue.rs`, `the_venue_text_fits_the_minimum_terminal`:**
  • `let x = layout.center_x.saturating_sub(w / 2);` → `layout.text_x…` —
  mechanical, with the test's comment ("centered on `center_x` as `draw`
  centers it") moving with it.
  • `assert!(x + w <= cols, …)` — **strengthened to
  `assert!(x >= 1 && x + w <= cols - 1, …)`**. Old bar: the **right** edge only,
  and because `x` comes from a `saturating_sub` a left overflow clamped silently
  to column 0 — so the precise failure `spec.md` forbids ("it must not end up
  flush against column 0, which reads as a rendering fault") would have passed
  this test. This is the single most load-bearing assertion change in the task.
  • **one assertion added, pinning the binding case**: at 89 columns the hint's
  left column is exactly **4** (`layout.text_x - HINT.chars().count() / 2 ==
  4`). Pinning 4 rather than "> 0" is what makes an unsanctioned re-lengthening
  fail — the old 63-character hint would sit at 0 and even a 57-character one at
  2 — instead of quietly closing on the edge.
  • the test helper `rows_for_opponent` (~250) and its `header_rows` call gain
  the `credits` argument. Pass **`u32::MAX`** here, so the widest representable
  balance is the one measured and the credit row can never become the binding
  row unnoticed; the breathing test uses the profile's real balance.

  (5) **Not changing, and a stop-and-report if they do**:
  `the_action_row_keeps_its_width_as_the_cursor_moves` (53 either way — no label
  changes here), `the_series_line_names_the_score_and_the_length` (T005a's two
  strings), `the_venue_keys_move_confirm_and_shortcut`, and **every test in
  every other file** — `layout.rs`'s board, briefcase, map and overlay tests,
  `overlay.rs`'s two asset tests, `board.rs`'s, `campaign_map.rs`'s and
  `profile.rs`'s included. No asset file and no other source file is touched.

  Do not run `cargo fmt`. (Copies: `CampaignMapLayout::new` in the same file for
  the full-screen banding, its `.max()` clamp and its top-anchored
  `portrait_panel`; `src/shop.rs`'s balance line (~173) for the credit row's
  exact wording and glyph; `src/board.rs`'s in-match hint for the ` · `
  separator idiom.)
  **One simplification the sign-off asked for**: `draw`'s
  `let art_cx = (layout.art.x0 + layout.art.x1) / 2;` is, after this task, the
  literal definition of `text_x` — the plan names that identity and then
  recomputes it anyway. Use `cx` for the placeholder label and keep only
  `art_cy`. They cannot drift (identical expression), so this is one fewer
  computation of one fact, not a risk fixed.

  *Verify: `cargo build --all-targets` no new warnings — in particular no
  `dead_code` on `ART_W_NUM`/`ART_W_DEN` and no unused `center_x`; `cargo test
  -q` green verbatim, with the re-pinned and added assertions passing;
  `git diff --stat` shows only `src/layout.rs`, `src/venue.rs` and
  `src/shop.rs` (the last added at the sign-off for `credits_label`, and its
  diff is the hoist and nothing else);
  `grep -rn "Credits: ◈" src/` matches **exactly one line**, `credits_label`'s
  own body — one literal, two readers, which is what makes the plan's
  shared-wording claim a fact instead of an assertion. **Two details the
  re-review pinned (2026-09-21), both of which a literal reading would have got
  wrong**: (i) `src/shop.rs`'s `the_full_list_fits_the_minimum_terminal` (~407)
  **re-types the whole format a third time** for a width fixture — build it from
  `credits_label(99_999)` too, or the grep matches two lines and its stated
  rationale is false while the gate still passes; (ii) the shop's own balance
  row keeps its remainder **byte-for-byte**, `"  ·  spendable ◈ {spendable}"`
  with **double** spaces around the `·` — the plan and this task write it
  single-spaced as shorthand, and taking that literally would silently narrow a
  screen R6 does not touch. The shop screen must render identically;
  `grep -n "center_x" src/venue.rs` is **empty** (the venue reads `text_x`
  only — this grep is scoped to `venue.rs` because `layout.rs` keeps two other
  screens' `center_x` fields on purpose, and a file-wide grep there would be
  unsatisfiable, which is the failure T006's and T005b's gates already taught
  this spec twice); `grep -n "text_x" src/layout.rs src/venue.rs` shows the
  field, its derivation and the venue's uses. (**The `grep -c "·"` check was
  dropped at the sign-off, 2026-09-21**: `-c` counts matching *lines*, not
  occurrences, and with no expected value a shortened hint and an unchanged one
  produce the same number — it confirmed nothing. The verbatim `HINT` paste with
  its character count, below, is the real check, so the report **must** carry
  it.) The report
  states the concrete `art` and `portrait` Rects and `text_x` at **both** fit
  sizes, the two art areas **with their percentage reductions from 1334 and
  2484**, the hint's character count and its left column at 89 columns, and
  **lists every existing assertion it changed with the old value and the new
  one**, mapped to the numbered items above — so the phase re-review checks that
  list rather than re-deriving it.*

- [x] **T005d** — `specs/029-tournament-rounds/planet-art-brief.md` (new): the
  per-planet art brief. Per plan §Design 11, for `spec.md`'s *Amendment,
  2026-09-21 (second)* ("A new deliverable: the per-planet art brief"). **A
  document only — no code, no asset, no test.** Authoring the art stays a
  non-goal of this spec; this is the brief a design agent is handed so the work
  can be done elsewhere and folded back in.
  **Written the way `specs/016-opponent-portraits/portrait-art-brief.md` was**,
  which is this task's pattern file in every sense: the same
  directory-of-the-spec location, the same `-art-brief.md` name, the same
  opening *Purpose* / *hard format constraint* / *palette* / *canvas* /
  *aesthetic* / *per-subject direction table* / *validation checklist* spine,
  and the same framing sentence — hand this whole file over as the prompt, and
  Claude Code validates and integrates whatever comes back. Read it before
  writing; do not invent a new shape.
  It must pin, at minimum:
  (a) **The hard format constraint first**, as spec 016 does: kaazap is a
  monochrome character-grid renderer with no image support and no colour, so the
  deliverable is plain text, one glyph per cell, UTF-8, LF newlines, one
  trailing newline, drawn at a single uniform emphasis so depth comes only from
  glyph density.
  (b) **The canvas, at both fit sizes, and that it is the art region's
  *interior***: `draw_box` keeps a single-weight border around the art Rect, so
  the drawable grid is **48 × 20 columns at 89-column terminals** (art Rect
  50×22) and **92 × 20 at 139** (art Rect 94×22). State the derivation, not just
  the numbers, and name
  `the_venue_bands_stack_and_the_art_takes_the_rest` in `src/layout.rs` as where
  the Rects are pinned — with the instruction that if the venue's geometry ever
  moves again the canvas must be **re-derived from `VenueLayout`, not copied
  from this brief** (nothing in the build reads the document, so nothing will
  catch it drifting).
  (c) **Two assets per planet — sixteen files — and the reason.** The two
  regions are the same 20 rows but 48 and 92 columns wide, so one grid cannot
  serve both: the wide one centre-cropped at 89 throws away 48 % of the
  composition on the size ruling R3 said the person least wants weak, and the
  narrow one centred in the wide region leaves 22 blank columns each side and
  undoes the dominance R3 and R4 are about. So: `assets/planets/<id>-narrow.txt`
  (48×20) and `assets/planets/<id>-wide.txt` (92×20) for each of the eight
  planet ids, and **the narrow grid is its own composition of the same subject,
  not a crop of the wide one**. Say that this spends the cost `spec.md`'s R3
  already named and the person accepted, and record the rejected single-asset
  alternative in one line — a 92-wide grid composed so its central 48 columns
  stand alone — so the later art spec can take it knowingly rather than blind.
  (d) **The character repertoire**, carried over from spec 016's brief rather
  than re-invented: the block/shade/half/quadrant set plus space, its
  East-Asian-ambiguous-width caveat, and that box-drawing glyphs are **reserved
  for the border the game draws** and must not appear inside the art. Plus one
  constraint the portraits did not need: **every line exactly the canvas width**,
  space-padded, because a short line leaves a ragged hole inside a bordered box
  rather than a left-aligned face.
  (e) **The eight planets**, by `id`, `name`, `region` and `blurb`, taken from
  `src/campaign.rs`'s `PLANETS` (ids: `cinder`, `scree`, `ashfall`, `karrus`,
  `drift`, `the-anvil`, `the-spindle`, `zenith`), in a table with a one-line art
  direction per planet drafted **from its own blurb and region** — the way spec
  016's table drafted a vibe per opponent from the roster's blurbs. The
  Outer Rim → Mid Rim → Core progression is the set's arc, so the direction
  should harden along it, as spec 016's does down the difficulty ladder. Say
  plainly that the subject is **the venue on that planet** — the tournament hall
  and what is out its window — not a map or a planet seen from space, because
  the person's reason for the large region was "it feels like you are actually
  in the venue, on the planet".
  (f) **How it loads, and that it is not this spec's work.** The portraits'
  mechanism is the model: authored text under `assets/`, `include_str!`-embedded
  into a `&'static str` field (`OpponentProfile.portrait`), drawn by a clip-safe
  drawer. Planet art would hang off a field on `Planet` in `src/campaign.rs`,
  and the venue would pick the widest asset that fits the region and centre it —
  one rule, no width threshold, so arbitrary terminal widths work and not only
  the two fit sizes. **State explicitly that none of this is built in spec 029
  and that wiring it up is the deferred art spec's job**, so nobody reads the
  brief as a work order against this branch.
  (g) **The validation checklist**, in spec 016's shape and enforceable without
  judgement: sixteen files present and correctly named; each exactly 20 lines;
  each line exactly 48 or 92 displayed columns; palette glyphs and space only,
  no ANSI or escape codes, no wide/zero-width/combining characters; the eight
  narrow grids pairwise distinct and the eight wide ones likewise; and last, the
  person's own go/no-go look at the running venue at 89×31 and 139×31.
  **Change nothing else.** No `src/`, no `assets/`, no `docs/`, no other file
  under `specs/`. Do not run `cargo fmt`.
  (Copies: `specs/016-opponent-portraits/portrait-art-brief.md` — structure,
  voice, palette block and validation-checklist shape lift almost verbatim;
  `src/campaign.rs`'s `PLANETS` for the ids, names, regions and blurbs.)
  **Three additions from the sign-off (2026-09-21), each a gap against the
  pattern file rather than a preference**: (h) **the originality constraint** —
  spec 016's brief spends a bullet on "Original designs, no trademarked
  species… no recognizable franchise creatures", and the risk is *higher* for
  planet halls than for faces, since the subject is a tournament hall in a KOTOR
  homage the constitution says is meant for itch.io. An implied aesthetic
  section is not the same as a pinned constraint, which is what this list is
  for. (i) **a format exemplar** — spec 016 shipped two baseline grids and
  called them "the two to beat", which is how the tool learned the format. This
  brief adds a constraint the portraits did not have (every line exactly the
  canvas width, space-padded); show at least one illustrated row so
  "space-padded" is not left to interpretation. (j) **one line in the
  rejected-alternatives note** saying that making the art region the same width
  at both sizes — the obvious way to need only one asset per planet — is
  **foreclosed by acceptance criterion 16**, which requires the region to be
  strictly larger at 139 than at 89. That is what makes the two-asset cost
  unavoidable rather than chosen, and it is the first thing a reader will ask.
  *Verify: **`git status --short` shows exactly one line**, `?? specs/029-tournament-rounds/planet-art-brief.md`, and nothing else —
  **not `git diff --stat`, which cannot list an untracked file and would print
  nothing at all** for a task whose entire output is one new file. Corrected at
  the second amendment's sign-off (2026-09-21), which found the original wording
  unsatisfiable and noted this repo has already written the same lesson down
  twice: `specs/022-balance-pass/tasks.md` ("`git diff --stat` can't show an
  untracked file") and `specs/023-first-run-onboarding/tasks.md`. Sixth
  instance on this spec of a gate that could not produce the output it asked
  for;
  `cargo build --all-targets` and `cargo test -q` green verbatim and **unchanged
  from T005c's run** (this task compiles nothing — reporting them is the
  constitution's bar, not evidence of a change); the report **re-derives the
  canvas from the layout test rather than from the brief** — quoting
  `the_venue_bands_stack_and_the_art_takes_the_rest`'s pinned Rects and showing
  `50 - 2 = 48`, `94 - 2 = 92`, `22 - 2 = 20` — and confirms the brief states
  that same pair; `for id in cinder scree ashfall karrus drift the-anvil
  the-spindle zenith; do grep -c "$id" specs/029-tournament-rounds/planet-art-
  brief.md; done` returns eight non-zero counts, pasted; the report quotes the
  brief's canvas section and its one-asset-or-two paragraph verbatim, and states
  the file count it asks for (16) and the directory it asks for them in.
  **PAUSE for the person** (after the Phase 2 re-review, which covers T005c and
  T005d together): the orchestrator drives the second amendment re-walkthrough
  in plan §Verification with the `run-kaazap` skill — `KAAZAP_DATA_DIR` pointed
  at a scratch directory, confirmed in the report — at 89×31 and again at
  139×31, and reports in plain language, in three parts.
  (a) **The proportions**: the art region is smaller than it was — about a sixth
  smaller by area at both sizes — while still clearly the biggest thing on the
  screen and still visibly bigger at the wider size. This is theirs to eyeball
  and it is the one thing in this pause only they can settle; give them the two
  before-and-after sizes in plain numbers and ask whether it is where they want
  it.
  (b) **The alignment**: every row of text now sits centred over the art box
  rather than over the whole screen, so the text reads as belonging to the
  picture under it instead of drifting right of it; the hint line is shorter
  (single spaces around its dots) and at the narrow size sits four columns clear
  of the left edge rather than against it.
  (c) **The balance**: the fifth row reads `Credits: ◈ …` in the Card Shop's own
  words, and it moves after a match settles — play one and come back to the
  venue to show it.
  Also confirm the action row still has an empty row above and below it with the
  header rows tight together, and that nothing is clipped at either size.
  **One line about T005d**, which has nothing to look at: the per-planet art
  brief now rides this branch, and it asks a design agent for **two** grids per
  planet — one for each layout width, sixteen in all — which is the cost their
  R3 ruling named. If they would rather spend eight drawings than sixteen, the
  brief records the single-asset alternative and the later art spec can take it;
  ask, rather than assume.
  **Say plainly what is not being re-walked**: the whole Phase 2 flow and the
  first amendment's wording — the lock, the routing, the wager, the settlement,
  quitting and resuming, `Best of 3`, the Card Shop rename — is untouched by
  these two tasks and is not re-run.*

- [x] **T005e** — `specs/029-tournament-rounds/planet-art-brief.md` +
  `specs/029-tournament-rounds/plan.md` (§Design 11): tighten the brief before it
  is handed off. From the second Phase 2 amendment review's notes (a), (b) and
  (c), recorded as close-out note 37. This is documentation only; no code, no
  asset, no test.
  **(a) The intermediate-width hole.** §Design 11 and the brief's *How it loads*
  both say "pick the widest asset that fits the region and centre it". The art
  interior is `span_w * 7/8 - 2` of a **continuously varying** span, so between
  the two fit sizes it takes intermediate values — at 120×31 the interior is 75
  columns, where that rule centres the 48-wide grid and leaves ~13 blank columns
  each side. That is the precise failure the brief rejects the single-asset
  option for ("leaves 22 blank columns on each side, which undoes exactly the
  dominance rulings R3 and R4 are about"), so the brief currently argues against
  its own loading rule. Say what happens between the fit sizes — and if the
  honest answer is that the later art spec must decide (stretch, tile, letterbox,
  or author a third size), **say that**, in the brief and in §Design 11, rather
  than leaving a rule that reads as settled.
  **(b) The checklist must be executable without judgement.** Item 4 says "only
  glyphs from the **allowed palette** (+ space)" while the palette section
  permits "a few plain ASCII marks … **only** if strictly single-width and used
  sparingly" — so a mechanical validator implementing item 4 rejects a file the
  palette allows, and "sparingly" is a judgement call. This tension is inherited
  faithfully from spec 016's brief, which is why it did not block; fix it here
  anyway, by giving item 4 an **explicit codepoint whitelist** (the block/shade
  glyphs plus the exact ASCII marks allowed) so one command can decide. Add the
  two checks the format section requires but the checklist omits: **LF line
  endings, not CRLF**, and **valid UTF-8**.
  **(c) One reconciling clause, moved.** *The aesthetic*'s "No figures at the
  table" is two sections from Scree's "a loose crowd pressed close around a small
  table" and The Spindle's "tiered seating looking down on a single table". The
  next sentence already reconciles them ("distant silhouettes … as texture — a
  discernible character is not"); put that distinction where the per-planet
  directions are read too, so a skimming agent cannot take the prohibition
  literally and drop the crowds those two planets are built around.
  Change nothing else in the brief — its canvas derivation, its exemplar (verified
  correct to the character at review), its planet table (verified canon against
  `PLANETS`) and its two-asset decision all stand. Do not run `cargo fmt`.
  (Copies: `specs/016-opponent-portraits/portrait-art-brief.md` for voice.)
  *Verify: **`git status --short` shows exactly one line**, ` M specs/029-tournament-rounds/plan.md`,
  plus the brief as modified — **not** `git diff --stat` for the brief, which is
  now tracked, so `git diff --stat` **is** right for both files here; state both.
  `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  green verbatim and unchanged from T005d (this task compiles nothing — reporting
  it is the check that it truly touched no code); the report pastes the revised
  checklist section and the revised *How it loads* section in full, and states
  the codepoint whitelist explicitly.
  **PAUSE for the person** (after this phase's review): the orchestrator reports
  the venue's new proportions and alignment in plain language at both widths,
  that the credit balance is on it, and hands over the art brief. **Two things to
  put to them**: the brief asks for **sixteen** drawings (two per planet, because
  the region is 48 and 92 columns wide at the two terminal sizes) — eight is
  possible if they accept one of the compromises the brief records; and whether
  the art region's proportions now look right, since R4's exact numbers were the
  plan's to pick and theirs to eyeball.*

- [x] **T005f** — `src/layout.rs` + `src/venue.rs` +
  `specs/029-tournament-rounds/planet-art-brief.md`: a consistent art–portrait
  gap (ruling **R7**, the person's finding at the second amendment's
  walkthrough, 2026-09-22). Per plan §Design 4's R7 note: keep `art_w` exactly as
  T005c computes it; set `portrait.x0 = art.x1 + PANEL_GAP + 1`; centre the
  group (`art.x0 = (cols − (art_w + PANEL_GAP + PANEL_W)) / 2`). Pins: 89×31 art
  `(7, 56, 5, 26)` **unchanged**, portrait `(64, 85, 5, 19)` → `(60, 81, 5, 19)`;
  139×31 art `(10, 103, 5, 26)` **unchanged**, portrait `(114, 135, 5, 19)` →
  `(107, 128, 5, 19)`.
  **Sanctioned assertion changes — a class, stated so this cannot under-enumerate
  a fifth time**: any assertion whose subject is the portrait's *x* position, the
  art–portrait gap, or the right-hand margin. Specifically expect the two
  portrait pins above, the "at least `PANEL_GAP`" gap assertion becoming
  **exactly** `PANEL_GAP` again (`portrait.x0 == art.x1 + PANEL_GAP + 1`), and any
  portrait-anchored right-margin assertion becoming **equal outer margins**
  (`art.x0 == cols − 1 − portrait.x1`). **Everything else must not move**: the art
  Rects, `text_x`, every *y* value and height, the R4 band, the hint's left
  column, the fit and breathing tests. Anything outside the class that moves is
  a stop-and-report. Correct every doc and comment that describes the portrait
  as right-anchored or the margins as unequal (`MARGIN_X`'s doc, `new`'s comment,
  the bands test's comment block).
  **The brief**: its canvas sizes and intermediate-width table are unchanged
  (both depend only on `art_w`), but its quoted excerpt of the pinned Rects
  includes the portrait Rects — update those two to match, and any prose that
  locates the portrait. Nothing else in the brief changes.
  Do not run `cargo fmt`.
  *Verify: the constitution's full command, verbatim; `git diff --stat` shows only
  the three files; the report lists every changed assertion and states it is in
  the sanctioned class; the report gives both portrait Rects and both outer
  margins as numbers. No per-task review; covered by a short review of this
  task's diff. **No pause** — the person said to continue straight to Phase 3
  once this is fixed; its attestation goes in the Phase 3 report.*

## Phase 3 — The series where the player already looks (walkthrough: the map's planet detail names what a launch commits you to before you take it, the status band carries the running score through every match of a series and nothing during a rematch, and the map banner after the deciding match names the series result beside the credits)

- [x] **T007** — `src/campaign_map.rs` + `src/app.rs` + `docs/economy.md`: the map's detail line and
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

- [x] **T008** — `src/board.rs` + `src/app.rs`: the series score during a match.
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

- [x] **T009** — `assets/primer_text.txt` + `assets/how_to_play_text.txt` +
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
  each name the two-of-three rule and the three-of-five final. In **`Readme.md`** — note the casing: the tracked file is
  `Readme.md`, not `README.md`, which T005b's gate spelled wrong and got away
  with only because darwin's filesystem is case-insensitive (on a case-sensitive
  one the grep would have printed "No such file" and looked satisfied). Folded
  in by the orchestrator at T005b, 2026-09-21. At
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

- [x] **T009a** — `assets/how_to_play_text.txt`: ruling **R8** (the person, at
  the Phase 4 pause, 2026-09-22). Replace the two campaign lines "Opponents are
  Best of 3, the last Best of 5, / and a series, once started, is played out."
  with "Each opponent is Best of 3 matches, the last / Best of 5. A started series
  is played out." (44 and 42 columns — no wider than the panel's widest, line
  count unchanged at 26). Change nothing else. **No assertion changes value**:
  the overlay tests check "Best of 3", "Best of 5" and "played out", all still
  present. *Verify: the full command verbatim; `wc -l` still 26;
  `git diff --stat` shows only the asset.*

## Phase 5 — Balance measured, not changed (walkthrough: none — the simulator is an `#[ignore]`d report and `docs/balance.md` is a document; no code the game runs changes, no constant moves, and nothing on screen differs)

- [x] **T010** — `tests/balance.rs` + `docs/balance.md`: the series rates,
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

## Phase 6 — The planet's art at the venue (ruling R9; walkthrough: every planet's venue shows its own picture filling its box edge to edge — the narrow picture at 89×31, the wide one at 139×31 — with nothing clipped, no planet name in the box, and the opponent's portrait beside it; at a width in between (120 columns) the box keeps the narrow picture's size and the spare columns sit either side of the picture-and-portrait pair; on a taller terminal (139×40) the whole screen sits centred with the spare rows above and below it. The person's look at both fit sizes is the go/no-go on the art)

<!-- Added by the R9 revision, 2026-09-22, after T011a; the close-out below
closes again after this phase. Sequencing, because the art arrives from another
session at an unknown time:

- T012 runs NOW. It is geometry only and names no asset — the drawing sizes are
  known (48×20 and 92×20, the brief's canvas).
- Then the phase WAITS for the sixteen files in `assets/planets/`. The wait is
  not a phase pause and not a design question; the orchestrator says in one
  plain sentence that the venue is ready for the art and waits.
- T013 is the checkpoint: the first thing done when the files arrive, and a
  failing delivery is bounced to the art session with its failures listed — a
  bounce, not a fix, and not a question for the planner.
- T014 needs T013 committed: its `include_str!` lines do not compile without
  the files.

T012 is observable on its own (at non-fit sizes the placeholder box shrinks to
the drawing's size; at taller terminals the screen centres), but it is walked
with the rest of the phase rather than paused on: a pause after T012 would show
the person an empty box they are about to see filled. -->

- [x] **T012** — `src/layout.rs` + `src/config.rs` + `src/venue.rs` +
  `specs/029-tournament-rounds/planet-art-brief.md`: the art box fits the
  drawing (ruling **R9**'s geometry). `review: per-task`. **Runs before the art
  arrives** — it names no asset. Per plan §Design 4's R9 section, which gives
  the code and the arithmetic:
  **(a) `src/layout.rs`, `VenueLayout`.** Delete `MARGIN_X`, `ART_W_NUM`,
  `ART_W_DEN` and `span_w`. Add the three private constants
  `ART_CANVAS_W_NARROW = 48`, `ART_CANVAS_W_WIDE = 92`, `ART_CANVAS_H = 20` with
  the plan's doc, and the field `pub wide_art: bool` with its doc. Rewrite `new`
  to the plan's arithmetic, line for line (horizontal: the box is the drawing
  plus its border, the group centred as T005f already does; vertical: the
  31-row composition centred, odd spare row below). Correct every doc the plan's
  R9 section lists as falsified: `VENUE_PANEL_H`'s, the struct doc, `art`'s,
  `hint_y`'s, `new`'s (the `.max()` paragraph and the "seven eighths" comment).
  **(b) `src/config.rs`.** Beside `fit_sizes()`, `#[cfg(test)] pub fn
  sizes_from_minimum() -> Vec<Config>`: every width from `min_size().0` to 220
  inclusive, at `min_size().1`, `+ 1`, `+ 2`, 40 and 60 rows (660 sizes), doc'd
  as "the sizes the venue's every-size tests measure". Pattern: `fit_sizes()`.
  **(c) `src/layout.rs` tests.** Rename `the_venue_bands_stack_and_the_art_takes
  _the_rest` → `the_venue_bands_stack_around_the_art` and rewrite its comment
  block (the art no longer takes the rest). Add
  `the_art_box_is_the_drawing_plus_its_border_at_every_size` exactly as plan
  §Tests' R9 bullet lists it, including the four pinned non-fit rows of §Design
  4's R9 table and the opening assertion that `sizes_from_minimum()` contains
  both `fit_sizes()`. The expected box at each size is taken from
  `VenueLayout::new` at the fit size with the same `wide_art` — **no literal 50,
  94 or 22** in that relation. In `the_art_region_dominates_at_both_widths`,
  comment only ("the fraction" → "the canvas sizes").
  **(d) `src/venue.rs`.** In `the_venue_text_fits_the_minimum_terminal` only: the
  outer loop runs over `Config::sizes_from_minimum()` instead of
  `Config::fit_sizes()`, and the comment's "at both layout widths" becomes "at
  every size from the minimum". `draw` is **not** touched — the placeholder
  stays until T014.
  **(e) The brief.** The three statements plan §Design 11's R9 note names, and
  the old test name wherever the brief uses it. The canvas, the palette, the
  checklist, the file names and the per-planet table do not change.
  **Sanctioned assertion changes — the complete list, and the class behind it.**
  (1) the rename above; (2) in it, `l.art.x0 >= VenueLayout::MARGIN_X` →
  `l.art.x0 >= 3` (same bound; message unchanged); (3) the fit test's loop
  source, `fit_sizes()` → `sizes_from_minimum()`. **The class**: nothing whose
  value is observed at 89×31 or 139×31 may move — every pinned Rect, `text_x`
  31 / 56, `header_y` 0, `action_y` 28, `hint_y` 30 and `hint_y == rows − 1`,
  the equal outer margins, the R4 band and its 1334 / 2484, the hint's left
  column 4, the breathing test. An existing assertion that fails, or any change
  outside the three items, is a **stop-and-report**: it means the fit-size
  layout moved, and R9 says it must not.
  Do not run `cargo fmt`.
  **Sign-off additions (2026-09-22)**: (a) `Config` does not derive
  `PartialEq`, so where a test checks that `sizes_from_minimum()` contains both
  `fit_sizes()`, compare the fields rather than using `.contains` (or add the
  derive and say so). (b) Two more docs become false and are yours: the
  dominates test's comment (`layout.rs` ~918) "fails if the fraction, HEADER_H
  or FOOTER_H is ever touched" — after R9 neither constant affects the art's
  area — and `venue.rs`'s `HINT` doc "on the last row", false above 31 rows.
  (c) The brief edits in (e) include at least these three, named so none is
  missed: the heading "…and that it is **not** spec 029's work" (~278); "pick
  the widest asset… and centre it" (~285), false under box-fits-art; and "None
  of that is built in spec 029… deferred art spec's job" (~322–325).
  *Verify: the constitution's full command, verbatim; `git diff --stat` lists
  exactly the four files; `grep -nwE "MARGIN_X|ART_W_NUM|ART_W_DEN|span_w"
  src/layout.rs` empty — **word-match (`-w`)**, because the substring form also
  matches `CampaignMapLayout`'s `FIELD_MARGIN_X`, which this task rightly leaves
  alone, and so could never come back empty (sign-off B1, 2026-09-22); `grep -rn "the_venue_bands_stack_and_the_art_takes_the_rest"
  src/ specs/029-tournament-rounds/planet-art-brief.md` empty; the report gives
  the computed `art`, `portrait`, `header_y`, `action_y`, `hint_y` and `text_x`
  at all six sizes of §Design 4's R9 table, and maps every changed assertion to
  items (1)–(3). Then the orchestrator: re-runs the command itself; captures the
  venue on one scratch profile (`KAAZAP_DATA_DIR`) at 89×31 and 139×31 **before
  dispatching T012 and again after**, and confirms the two captures at each size
  are identical — the strongest evidence that nothing moved at the fit sizes;
  and dispatches the per-task review on T012's diff alone. Then the phase waits
  for the art (see the comment above).*

> **Orchestrator miss, 2026-09-22.** The sixteen art files were committed
> **before** T013, and by accident: the art session wrote them into
> `assets/planets/` while the orchestrator was committing the Phase 6 plan with
> `git add -A`, which swept them into `7445c68` ("plan Phase 6") unvalidated and
> unmentioned in its message. The working tree matches that commit, so what is
> committed is the art session's final output. History is not rewritten (never
> force-push). T013 therefore **validates the files already committed** instead
> of committing them; a failing delivery is still bounced to the art session and
> its fix lands as a new commit. From here on the orchestrator stages explicit
> paths, never `git add -A`.

- [x] **T013** — `assets/planets/` (sixteen delivered files): **the delivery
  checkpoint — starts only when the art session's files are in
  `assets/planets/`.** Validate them against `planet-art-brief.md`'s checklist,
  items 1–6, by shell, using the brief's own numbers and commands. **Writes no
  code and edits no file.**
  Item 1: `ls -1 assets/planets/` lists exactly `{id}-narrow.txt` and
  `{id}-wide.txt` for the eight ids of `PLANETS` (`cinder`, `scree`, `ashfall`,
  `karrus`, `drift`, `the-anvil`, `the-spindle`, `zenith`) and nothing else.
  Items 2–3: one `python3` check that prints, per file, its line count, whether
  it ends in exactly one LF, and the set of its line lengths **in characters, not
  bytes** — every file must read 20, yes, `{48}` for a `-narrow` file and `{92}`
  for a `-wide` one. Items 4–5: the brief's own command, verbatim — every file
  `lf-ok glyphs-ok`. Item 6: `python3` prints the count of distinct contents
  among the eight `-narrow` files and among the eight `-wide` files — 8 and 8.
  **A failing delivery is a bounce, not a fix**: the report lists every failing
  file with its item and what was found; the orchestrator sends that list back
  to the art session through the person; no file is edited, re-padded or
  converted here — a repaired file is no longer the delivered artwork, and the
  checklist exists to be enforceable without judgement. T013 stays unchecked
  until a delivery passes, and each attempt is a tier-log row.
  *Verify: every command and its full output in the report;
  `git status --short -uall` shows exactly sixteen `?? assets/planets/…` lines
  and nothing else (`-uall`, because without it git collapses an untracked
  directory to one line). On a pass the orchestrator stages `assets/planets/`,
  confirms `git diff --cached --stat` lists exactly the sixteen files, and
  commits them unmodified as T013.*

- [x] **T014** — `src/campaign.rs` + `src/venue.rs` + `assets/CREDITS.md` +
  `.gitattributes` (new): the
  art at the venue, and AC 21's tests. **Needs T013 committed.** Per plan
  §Design 12, which gives the code:
  **(a) `src/campaign.rs`.** `Planet` gains `art_narrow` and `art_wide`
  (`&'static str`) with the plan's doc; each of the eight `PLANETS` entries gains
  `art_narrow: include_str!("../assets/planets/{id}-narrow.txt")` and
  `art_wide: include_str!("../assets/planets/{id}-wide.txt")`. Pattern:
  `src/opponent.rs`'s `portrait: include_str!(…)` fields.
  **(b) `src/venue.rs`.** `planet_art` and `draw_art` exactly as the plan writes
  them; in `draw`, the three placeholder lines (`draw_box`, `art_cy`, the
  centred `planet.name`) and their comment become one `draw_art(frame,
  layout.art, planet_art(&planet, &layout), planet.name)` with a comment saying
  the box is the drawing's size (R9), the portraits' drawer draws it, and the
  name is only the fallback. Imports gain `campaign::Planet`, `layout::Rect` and
  `portrait::draw_portrait`. Any doc in the file that still calls the region a
  placeholder is corrected; nothing else in `draw` moves.
  **(c) Four tests in `src/venue.rs`**, each as plan §Design 12 specifies:
  `every_planets_art_passes_the_briefs_checklist` (items 1–6, the canvas
  derived from `VenueLayout::new(c).art` over `Config::fit_sizes()`, the
  23-character palette with its count asserted, the `\r` check explicit),
  `every_planets_art_fills_its_box_at_every_size` (over
  `Config::sizes_from_minimum()`), `the_venue_draws_the_planets_art_inside_its_box`
  (every planet, both fit sizes, the drawn frame's interior equals the drawing),
  and `a_planet_without_art_shows_its_name`. No new test constructs an `App`.
  **(d) `assets/CREDITS.md`.** A *Venue art* section after *Portraits*, in its
  shape; the orchestrator's bundle names the tool the art was authored with.
  **Existing assertions: none changes** — T014 only adds. An existing test that
  fails is a stop-and-report. Do not run `cargo fmt`.
  **Sign-off additions (2026-09-22)**: (a) the `\r` assertion must run
  **before** the item-4 palette check, or item 4 must iterate over `lines()` —
  otherwise a CRLF file fails item 4 first and mutation check (ii) fails on the
  wrong assertion, making the "real hole" claim untested. (b) Add a
  **`.gitattributes`** line `assets/planets/*.txt -text`: Windows is a target
  platform, Git for Windows defaults to `core.autocrlf=true`, and a CRLF checkout
  would put `\r` in every embedded line — failing the new assertion and
  mis-sizing the art. `.gitattributes` joins this task's file list.
  *Verify: the full command, verbatim; `cargo test --lib venue -- --list`
  shows the four new names; `git diff --stat` lists exactly the three tracked
  files **and `git status --short` shows `?? .gitattributes`** — it is new and
  untracked, so `git diff --stat` alone would pass whether or not it exists
  (re-review, 2026-09-22); the orchestrator stages it with the commit and
  includes it in the Phase 6 review diff;
  `grep -c 'include_str!("../assets/planets/' src/campaign.rs` → 16;
  `grep -nwE '48|92|20|50|94|22' src/venue.rs` → nothing (it returns nothing
  today; widened at sign-off so the box sizes can't be restated either) (AC 21: the canvas is derived,
  not restated; `-w` rather than `\b`, for darwin's BSD grep). **Two mutation checks, each reported with the failing test's
  output**: (i) append one space to one line of one `-narrow` file → the
  checklist test fails, naming that file and the width; (ii) convert one file to
  CRLF (`perl -pi -e 's/\n/\r\n/'`) → the checklist test fails on the `\r`
  assertion, which is the hole `str::lines` would otherwise leave. Restore each
  with `git checkout -- <file>` and show **`git status --short assets/planets`
  empty** and `git diff --stat` still listing exactly this task's files before
  returning. (Not the whole tree clean: the implementer never commits, so its
  own edits always show — sign-off B2, 2026-09-22.) Then the orchestrator: the Phase 6 review, then the walkthrough in
  plan §Verification ("After the Phase 6 review"), then the pause for the
  person's go/no-go. **Three things to put to them at that pause** (sign-off,
  2026-09-22): plan §Open questions 7 (the art is drawn at the portraits' full
  plain weight — dim it?), §Open questions 8 (on terminals taller than 31 rows
  the whole venue is centred vertically, so the hint is no longer on the bottom
  row), and the text of the new `design/brief.md` amendment recording the art as
  the design record's second bounded exception.*

- [x] **T014a** — `src/venue.rs` + `src/layout.rs`: two fixes from the **second
  pre-merge sweep** (2026-09-22). (a) AC 21's checklist test skips hidden files
  when listing `assets/planets/` — Finder's `.DS_Store` is hidden by `.gitignore`
  and would otherwise fail `cargo test` while `git status` shows nothing; only
  dotfiles are skipped, so a misnamed art file still fails. (b) The dominates
  test's comment says "strictly larger at 139 columns than at 89", not "when the
  terminal is wider", which the box-fits-art rule made false. *Verified: 488 lib
  tests green; a `.DS_Store` present → the test passes; a `cinder-narrow.text`
  present → it fails naming the extra file; both removed and
  `git status --short assets/planets` empty. Re-reviewed clean.*

## Final phase — Spec close-out (walkthrough: none — documentation, mechanical checks and the pre-merge sweep; the person's walkthrough list above is what they walk at this phase)

- [x] **T011a** — `src/app.rs` + `src/economy.rs` + `src/wager.rs` + `src/shop.rs` +
  `docs/economy.md`: two defects the **pre-merge sweep** found (2026-09-22), both
  ruled fix-now and neither a question for the person.
  **(B1, blocking) A series can start against an opponent the player cannot
  cover.** `launch_from_map`'s un-beaten branch runs `begin_series` → `save` →
  `open_campaign_home` with no affordability check; only `open_wager` checks
  `credits < floor`. With Cinder cleared and 15 credits, Enter on Ashfall (ante
  20) locks the player into a series whose Play silently refuses, the map is
  unreachable, and the next campaign entry ends the run — without a credit
  staked. It contradicts plan §Design 2 and §Open questions 5 ("CantCover is
  unreachable from the venue"), ruling C1, AC 4, and `docs/economy.md`'s "the
  launch is refused before any prompt opens". **Fix**: one private helper,
  `fn refuse_uncovered(&mut self, floor: u32) -> bool` — "Refuse a launch the
  balance can't cover (spec 021): the map's banner and the back cue — nothing
  staked, nothing locked. True when it refused." — which `open_wager` uses in
  place of its inline check, and which `launch_from_map`'s series branch calls
  with `economy::ante_floor_for(opponent)` **before** `begin_series`. One copy of
  the check, two callers. **No `App` unit test**: a regression in that branch
  calls `save()`, which would write the developer's real profile (close-out note
  21). Verified by driver walkthrough instead (below).
  **(N1) The wager prompt's run-over warning can over-warn on a deciding match.**
  While locked, the warning reads `reserve_floor` = the locked opponent's ante,
  but a loss that decides the series releases the lock and the broke check then
  reads the map's cheapest ante. It only over-warns, never under-warns, but it
  breaks the 2026-09-17 chore's rule that the warning predicts exactly the check
  that ends the run. **Fix**: in `src/economy.rs`, beside `reserve_floor`,
  `pub fn reserve_after_a_loss(run: &CampaignRun, planet: &str, opponent: &str)
  -> u32` — clone the run, `record_series_match(planet, opponent, false)` on the
  clone, return `reserve_floor(&after)` — reusing the series' own decision rule
  rather than re-deriving it; `open_wager` passes it as `WagerState::new`'s
  reserve. Test in `economy.rs`: a fresh run → 10; the half-cleared run of
  `reserve_floor_follows_the_lock` locked on Rix at 0–0 → 50, at 0–1 → 10; the
  final opponent at 0–1 → 50, at 0–2 → 10; a rematch of already-beaten Greeb →
  10. This changes only *when* the warning row shows — no string, no floor, no
  stake range, no escrow — so it is inside the spec's "no change to the wager
  arithmetic" non-goal.
  **Docs, same change** (the sweep's N2 and N8 ride along): `src/wager.rs`'s
  `reserve` field doc, `Role::Warning` doc, `new`'s doc and `loss_ends_the_run`'s
  doc — none should say "the run's cheapest ante", and they should say the
  reserve is what the broke check reads *after a loss*; `economy::reserve_floor`'s
  doc drops "the wager prompt's warning" from its readers; `src/app.rs` ~759–760
  ("the campaign-map Card Shop… Back returns to the map") and ~997 ("Esc backs
  out to the map") and `src/shop.rs` ~3–5 ("reached from the campaign map") —
  each now also the venue; `docs/economy.md`'s refusal paragraph names both
  callers, and its *One visible consequence* paragraph gains the deciding-match
  exception.
  **Existing assertions**: none should change value. Anything that does is a
  stop-and-report. Do not run `cargo fmt`.
  *Verify: the full command verbatim; `git diff --stat` shows only the five
  files; `grep -n "cheapest ante" src/wager.rs` empty; the report quotes
  `refuse_uncovered`, `reserve_after_a_loss`, `launch_from_map` and `open_wager`
  verbatim. Then the orchestrator: the sweep's **one permitted re-review** on this
  diff, and two driver checks on scratch profiles — (a) Cinder cleared, 15
  credits, Enter on Ashfall → "Can't cover the 20-credit ante" on the map, no
  venue, and no `series` key in the profile JSON; (b) a hand-edited profile locked
  on Rix with 80 credits — at 0–1 a 40 stake shows no warning, at 0–0 it does.*

> **Held open by ruling R9 (2026-09-22).** At the merge pause the person
> added integrating the per-planet art to this spec. T011 was complete for the
> spec as it then stood; it is **re-opened**, and closes again after the art
> phase lands — its close-out doc, AC checklist and mechanical checks must be
> refreshed for acceptance criterion 21 and a second pre-merge sweep run on the
> art phase. The art phase is **Phase 6** (T012–T014, above); T011 runs after
> T014 and the Phase 6 pause. **Revised by the R9 revision**: the close-out doc
> the first T011 run drafted is **refreshed, not redrafted** — the edits below
> marked *(R9)* are the whole of what changes in it; everything it already
> records stays. The first sweep's findings are closed (T011a and its
> re-review); only the second sweep is owed.

- [x] **T011** — Close-out. Draft
  `specs/029-tournament-rounds/closeout-main-docs.md` in spec 028's shape:
  **ROADMAP** — mark tournament rounds shipped as spec 029 (`grep -n -i
  "series\|tournament\|venue\|best of" ROADMAP.md`, read and judge), and add the
  follow-ups this spec deliberately deferred and named. *(R9)* **Per-planet venue
  art is no longer one of them** — it ships in this spec: remove it from the
  draft's follow-ups and record it with the shipped spec instead (each planet's
  venue shows its own art, sixteen drawings, the box sized to the drawing). **The refresh
  must also correct**, since "everything it already records stays" would
  otherwise carry R9-false text to `main` (sign-off, 2026-09-22): §1a's "holds a
  plain placeholder" and "brief for the deferred art spec"; §1b's "per-planet
  venue art… first concrete pieces"; §1c becomes **mark the existing `main`
  bullet shipped** rather than removing or rewriting it (removal would leave
  `main`'s stale deferred bullet standing); and §3's open question about a
  bounded-exception amendment is **closed** — the orchestrator made it in
  `design/brief.md` under R9.
  The follow-ups that remain: **per-planet music**
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
  own**: R1 the series length reads `Best of 3` rather than a second phrase —
  the same words the map's planet detail will use once **T007** lands them, which
  it has not yet, so a pause report must not tell the person the map already says
  it (amendment sign-off, 2026-09-21), R2 the shop is the **Card Shop** everywhere the player reads
  it — with the note that earlier specs' documents, `DECISIONS.md` and
  `ROADMAP.md` keep the old word on purpose, since rewriting them would falsify
  history — and R3 the art dominates the screen at every width, with the
  portrait keeping its own column and the cost named (each planet's art must
  work at two quite different sizes, which lands on the deferred art spec). Note
  that R3 cost one struct and one constant (`VenueRail`, `VENUE_ART_W`), deleted
  rather than kept as unconditional wrappers.
  **And `spec.md`'s *Amendment, 2026-09-21 (second)*, which is three more
  rulings of the person's own plus a deliverable**: **R4** the art about 15–20 %
  smaller — landed at 17.5 % (89 columns) and 16.8 % (139) by taking seven
  eighths of the art's available columns, the **one scale factor** this plan
  takes, because no fixed column inset lands inside R4's band at both fit sizes
  (a divergence from §Design tension 7's "a subtraction, not a ratio", recorded
  there rather than slipped in); **R5** every text row centres on the **art's**
  centre rather than the terminal's, with the consequence the person was warned
  of resolved by shortening the controls hint from 63 characters to 55 using the
  board's own ` · ` separators — the alternatives (clamping the row, aligning
  only the header, splitting the hint, widening the art) each rejected for a
  stated reason, and `spec.md`'s "not flush against column 0" rule turned into
  an assertion the fit test had been silently satisfying through
  `saturating_sub`; **R6** the venue shows the credit balance, in the Card
  Shop's own words, checked by the existing drawn-frame breathing test rather
  than by a new one; and **plan §Open questions 2 closed by the person** — the
  placeholder keeps the planet's name. Record that the **per-planet art brief**
  ships on this branch as `specs/029-tournament-rounds/planet-art-brief.md`, in
  spec 016's shape, and that it asks for **two grids per planet** (48×20 and
  92×20, sixteen files) because the art region is a different width at the two
  layout sizes — the cost R3 named — with the single-asset alternative recorded
  in the brief for the deferred art spec to take if the person prefers it.
  *(R9)* Correct that sentence's ending — there is no deferred art spec now —
  and add **R9**, the person's ruling at the merge pause: the art is integrated
  in this spec (authoring it stays outside, by the tool the person used, from the
  brief); **between the fit sizes the box fits the art** — the person's choice
  over the four options the brief priced (letterbox, stretch, tile/extend, a
  third size); the planet's name survives only as the fallback for a planet with
  no art. And R9's three plan calls with their reasons: **a taller terminal
  centres the whole venue vertically** (plan §Open questions 8 — the text keeps
  its relationship to the picture, as the board's block does; the hint leaves the
  last row above 31 rows); **the drawing at full strength**, like the portraits
  (§Open questions 7, and whatever the person said about it at the Phase 6
  pause); **R4's fraction deleted** — the box follows the drawings now, R4's
  result survives as their size and its band test still pins it. And that
  **AC 21's checklist test derives its canvas from the venue's layout**, which
  closes note 37's concern that nothing tied the brief to the code.
  And the plan's
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
  *(R9)* Plus: `git ls-files assets/planets | wc -l` → 16;
  `grep -c 'include_str!("../assets/planets/' src/campaign.rs` → 16; and the
  AC 20 checks above re-run after Phase 6 (the art adds no crate, no colour path
  — the palette admits no escape character — and touches no engine or save file).
  **Re-run every mechanical check and the three test runs fresh**; the first
  run's outputs predate Phase 6 and are not evidence for it.
  Check off `spec.md`'s **21** acceptance criteria with evidence (the Phase 2, 3
  and 4 walkthrough reports are the evidence for the venue, the routing, the
  board and the texts; T010's simulator output for criterion 19; *(R9)* for
  criterion 21, T013's checklist output, T014's four tests by name with their
  green output, and the person's go/no-go at the Phase 6 pause — item 7 — quoted).
  *(R9)* The first pre-merge sweep ran on 2026-09-22 and its findings are closed
  (T011a, re-reviewed). **Request the second pre-merge sweep, scoped to
  Phase 6**: its bundle is `git diff <the R9 plan commit>..HEAD` (the commit that
  lands this revision of `plan.md` and `tasks.md`; the orchestrator names its
  hash), `spec.md`'s R9 and AC 21, plan §Design 4's R9 section, §Design 11's R9
  note, §Design 12, the R9 bullets of §Tests, and `closeout-main-docs.md`'s diff
  since the first sweep. Beyond the diff it checks the one seam Phase 6 opens
  across phases: nothing from Phases 1–5 still assumes the art box grows with
  the terminal — the brief and the tests' comments. (`spec.md`'s venue
  paragraph and AC 16 were reconciled by the orchestrator before sign-off.)
  The sweep bundle **also carries `design/brief.md`**, whose *Skeuomorphism
  boundary* gained a second bounded exception for this art (sign-off B3).
  Apply `closeout-main-docs.md` on `main` after the merge. Never chain a
  file edit, a branch switch and a commit in one shell command (spec 025's miss).
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and fresh output; AC 21 ticked with the evidence above; the second
  sweep clean or its findings resolved.*

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
Verify from the implementer's verbatim output, except **T001, T003 and T012
(`review: per-task`)**: the orchestrator re-runs the verification command itself
and dispatches a `skeptical-reviewer` on that task's diff alone before the next
task starts.

**Foundational phase: Phase 1** (Phase 6 is not foundational). One
`skeptical-reviewer` pass at the end of each phase — after T003, T006, T008,
T009, T010 and **T014** — on a shell-assembled bundle
(the phase diff, the task lines, plan §Design and §Tests, the acceptance
criteria), one review plus at most one re-review. **Phase 2 has three reviews
rather than one**, because the person amended `spec.md` twice mid-phase: the
original pass after T006, a re-review after T005b (the first amendment), and a
second re-review after **T005d** covering T005c and T005d together. Each is one
review plus at most one re-review of its own, and each checks its tasks'
enumerated assertion lists rather than re-deriving them. The Phase 1 review also
checks
that the only existing tests T003 changed are the two sanctioned kinds. The
Phase 2 review also checks the venue against the constitution's *acted-on
element stands apart* rule and the modal's even padding, and runs the
single-assignment grep itself rather than taking T006's report for it.

**Pause cadence — when there's something to try.** Pause for the person after
**Phase 1**, **Phase 2**, **Phase 3**, **Phase 4** and **Phase 6**, once each phase's review
and its walkthrough are done — and Phase 2 pauses **three** times, once per
amendment, because each amendment changed what the venue shows and the person is
the only one who can attest to that. The third pause is T005d's, after the
second re-review; T005c and T005d are walked together and the brief itself has
nothing to look at. **Only Phase 5 and the close-out are marked
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

**Phase 6 (the R9 revision) pauses once, and waits once.** It is marked with a
walkthrough and pauses after T014's phase review and the orchestrator's driven
renders: the person's look at every planet's art at both fit sizes is AC 21's
go/no-go, and nobody else can give it. Before that, between **T012** (runs now,
per-task review) and **T013** (the delivery checkpoint), the phase **waits** for
the sixteen files. The wait is not a phase pause and not a blocker: the
orchestrator reports in plain words that the venue is ready for the art and that
it will continue when the files are in `assets/planets/`. No continuation prompt
unless the person asks for one or says they are stopping — the first unchecked
task, T013, is the resume point. A delivery that fails T013 goes back to the art
session with its failures listed; that is a bounce, reported plainly, not a
question for the person to decide. After Phase 6, **T011** runs without a pause
(the close-out is `walkthrough: none`) and ends at the second pre-merge sweep and
the merge pause.

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
| 2 (amendment) — the venue's wording and its art | **yes** | The series line reads **Best of 3**; the second action and the screen it opens both read **Card Shop**; the art region now dominates the venue at **both** widths, with the portrait in its own column beside it. Driven and attested by the orchestrator at both widths; **one item still not hand-attested** — the deck-guard divert (see the tier row). |
| 2 (second amendment) — the venue's proportions, alignment and balance | **yes** | The art is ~17% smaller at both widths; every text row now centres on the **art** rather than the terminal, which is what the person's eye caught; the credit balance is on the screen. Plus the **art brief** handed over for a design agent. Driven and attested by the orchestrator at both widths; two questions put to the person (sixteen drawings vs eight, and whether the proportions look right now). |
| 3 — The series where the player already looks | **yes** | The map's planet detail says `Best of 3` before a launch and nothing on a cleared planet; the status band carries `Series 0 – 0` beside the turn prompt at both widths; a rematch shows no score; the deciding loss's map banner reads `Series lost · Lost 10 credits`. **Not seen on screen**: the winning banner (`★  Series won · …`) — the auto-player lost six straight tries; it is covered by an exact-string test. One question put to the person (the deciding game-over frame). |
| 4 — What the player is told | **yes** | Start a fresh campaign: the first-campaign primer names Best of 3, Best of 5 and the commitment. Open How to Play from the menu: its campaign section says the same. Both fit 89×31, nothing clipped. One wording question put to the person. |
| 5 — Balance measured, not changed | no | Nothing to see: the simulator is an ignored test and `docs/balance.md` is a document; no constant, deck or roster value moved. The recorded result worth knowing: as a series, a deck the curve calls ready wins more reliably and one it calls unready loses more reliably — the starter's chance against the final opponent falls to about 9 %. |
| 6 — The planet's art at the venue | **yes** | Every planet's venue shows its own picture filling its box — narrow at 89 columns, wide at 139 — with the portrait beside it; at in-between widths the box keeps the narrow picture's size; on a taller terminal the venue sits centred. The person's look at the art is the go/no-go. |
| Close-out | no | Documentation, checks and the sweep. **Two items worth walking before merge, from the sweep**: with too few credits for an un-beaten planet's ante, Enter on it should refuse with "Can't cover the N-credit ante" and stay on the map; and on the match that could lose a series, the wager prompt should not warn that the run ends unless it actually would. Both driven by the orchestrator. |


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

**From the Phase 2 amendment review (2026-09-21, no blocking findings):**

26. **An enumeration miss in T004a, recorded rather than fixed.** The task line
    said the fit test "loses one clause"; the diff also had to drop
    `assert!(layout.top + BLOCK_H <= rows_avail, …)` and its binding, because
    `layout.top` and `BLOCK_H` no longer exist — it could not compile. Coverage
    is replaced and improved by the bands test, so nothing is lost, but the
    "anything else that moves is a stop-and-report" bar was under-enumerated and
    the implementer neither stopped nor listed it. **Third instance in this spec
    of an enumeration that was not quite complete** (T001, T006, this) — the
    pattern is worth a line in the close-out's DECISIONS entry.
27. `src/layout.rs` — `portrait_y1`'s `.max(band_y0)` is a no-op clamp
    (`VENUE_PANEL_H` is 15, so the sum always exceeds `band_y0`), and it is the
    one derived edge with no clamp against `rows`: below the enforced minimum,
    `art_y1` degrades while `portrait_y1` would sit off-frame. Unreachable —
    `Config::from_terminal` errors out below 89×31 — and the plan's stated bar
    ("degrades instead of inverting a `Rect`") is met. **Sweep.**
28. The bands test hardcodes the margin rule as `3` and `cols - 4` though
    `VenueLayout::MARGIN_X` is visible to the test module. The concrete `Rect`
    pins beside them are deliberately literal (to catch a uniform arithmetic
    slip) and that stands; these two assert the *rule* rather than the table, so
    they are the two worth reading from the const. **Sweep.**
29. `the_art_region_dominates_at_both_widths` asserts only
    `art_area > portrait_area`, which is narrower than AC 16's "the largest
    element on the screen". Sound today — every other element is a single row of
    text — but a later spec adding a second panel would not be caught.
30. `row_text` in `venue.rs`'s tests is a second copy of `board.rs`'s helper.
    The task line instructed the copy, so authorized, not drift. **Sweep** may
    decide whether a shared test helper earns its place.
31. `Readme.md`'s edited line runs to ~86 columns against the file's ~78-column
    wrap: `a shop` → `a **Card Shop**` added 11 characters without reflowing,
    and nothing tests it. **T009 owns the README's other line** — reflow there.
32. T005a's `grep -rn "first to" src/` gate output was not in the review bundle;
    the reviewer settled it from the diff and a repo-wide grep instead, and the
    claim holds. Same point as note 25: **echo the gate's command and its
    output, don't retype the label.**

**From the second Phase 2 amendment review (2026-09-21, no blocking findings):**

33. **An orchestrator process miss, three instances now.** The review bundles
    have shown `cargo test -q --lib` plus `cargo build --all-targets | grep -c
    "^warning"` instead of the constitution's
    `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`.
    The warning-count grep **returns 0 for a failed build as readily as a clean
    one**, so it cannot distinguish them, and `--lib` compiles no integration
    target. The reviewer checked all 8 files in `tests/` itself and confirmed
    none references a symbol this spec renames, so nothing was missed — but that
    is inspection standing in for a command. **Notes 3, 25 and 32 asked for the
    verbatim full command; this is the third recurrence. The orchestrator pastes
    the full command's output from here on.** Belongs in the close-out's
    DECISIONS entry beside note 26's pattern.
34. **A fourth under-enumeration, same pattern as note 26.** T005c enumerated
    "three doc comments become false"; the diff also rewrites `header_y`'s field
    doc. Authorized — plan §Design 4's code block shows that exact doc — but the
    task's own stop-and-report bar was short again, and the implementer neither
    stopped nor listed it. Fourth instance in this spec; one close-out entry
    should carry all four.
35. **The credit row's *content* has no test**, which both the task line and plan
    §Design 5 sanction. The breathing test requires row 4 to be non-blank, so
    deleting it fails, and `credits_label` makes a wording drift
    unrepresentable — but replacing the row with any other non-empty string
    keeps the suite green. AC 3's credit clause rests on the rendered frame and
    the shared function, not on an assertion. Coverage honesty, same spirit as
    note 17.
36. The binding-case fit assertion is pinned to a literal `Config { 89, 31 }`
    outside the `fit_sizes()` loop. If the enforced minimum ever moves, the loop
    follows it and this assertion silently stops being about the binding case
    rather than failing. `Config::fit_sizes()`'s first element would be one
    fewer place holding one fact. **Sweep.**
37. **Three soft spots in the art brief, being fixed in T005e rather than
    recorded**, because the brief's whole purpose is to be handed to a design
    agent unaided and these would degrade what comes back: (a) plan §Design 11's
    "widest asset that fits, centred" rule leaves ~13 blank columns each side at
    an *intermediate* width like 120 — the precise failure the brief rejects the
    single-asset option for; (b) validation checklist item 4 is not mechanically
    executable, since the palette section permits "a few plain ASCII marks …
    sparingly" while item 4 says "only glyphs from the allowed palette" — a
    tension inherited faithfully from spec 016's brief — and the checklist
    checks columns and glyphs but not LF-vs-CRLF or UTF-8, both of which the
    format section requires; (c) *The aesthetic*'s "No figures at the table"
    sits two sections from Scree's "a loose crowd pressed close around a small
    table" and The Spindle's "tiered seating looking down on a single table" —
    reconciled by the next sentence, but a skimming agent may not get there.

**From T005f's review (2026-09-22, no blocking findings):**

38. "Equal outer margins" holds only when `cols − group_w` is even; at odd
    leftovers the right margin is one wider (160 columns: 11 and 12). Unavoidable
    with a fixed gap and fixed art width. The plan note, `MARGIN_X`'s doc and the
    bands-test comment state equality unqualified — add "to within one column".
    **Sweep.**
39. Two doc sentences T005f rewrote read badly: `text_x`'s ("…sit to its right,
    25 columns of the centred group") and `ART_W_NUM`'s ("The share of that
    span", which points at another item's doc). Both also overrun the wrap.
    **Sweep.**
40. `MARGIN_X` is now only an input to the sizing formula, not an on-screen
    margin. Its doc says so; the name no longer describes anything visible.
    **Sweep may rename.**

**From the Phase 3 review (2026-09-22, no blocking findings):**

41. A match left in flight across the upgrade shows **no** score while played:
    `board_series_line` needs a series, and `record_series_match` only creates
    one when that match settles. It would have read `Series 0 – 0`. One-off, for
    a pre-029 save, and not an AC 11 failure; `board_series_line`'s doc lists
    three no-score cases and misses this fourth. **Sweep.**
42. `launchable_opponent`'s doc says "callers gate on `planet_unlocked`", and the
    new caller `series_length_detail` does not — so locked planets show their
    series length too. Authorized (T007's own test needs `Best of 5` on a locked
    Zenith) and the value is correct, but the doc's contract is now false for one
    caller. **Sweep.**
43. A contradictory banner ("Series won · Lost N credits") is unreachable in
    play — only a hand-edited save already at or past `wins_needed` could produce
    it. Recorded for completeness.
44. `the_banner_and_the_run_tally_report_the_same_net_gain` passes
    `SeriesOutcome::NotInSeries` rather than `settled.series`, per the task line;
    it no longer checks the banner against the real settlement pairing. Minor.

45. **plan §Open questions 1 — put to the person at the Phase 3 pause
    (2026-09-22); they continued to Phase 4 without ruling.** Behaviour stands
    as built: the deciding match's game-over frame shows no series score, and
    the map banner that follows names the result. Not treated as a ruling
    either way — the close-out records it as *asked, unanswered, left as
    built*, and holding the final score remains a one-field sub-lettered task
    if they later want it.

**From the Phase 4 review (2026-09-22, no blocking findings):**

46. How to Play's "Opponents are Best of 3" names no unit, in a panel that opens
    with "First to 3 round wins takes the match" — a new player could read it as
    rounds. Meets AC 18 (R1 made "Best of 3" the one phrase for the idea), so
    not blocking; **put to the person at the Phase 4 pause** with a two-line
    fix that fits the 44-column width ("Each opponent is Best of 3 matches, the
    last / Best of 5. A started series is played out.").
47. `Readme.md`: the rematch sentence does not say rematches are single matches
    launched from the map with no venue, right after a sentence saying each
    opponent is a series played at the venue; "the map stays closed until you
    win it or lose it" omits New Campaign and Reset Everything as releases; the
    locked-floor clause reads as a second check rather than a replacement
    (accurate either way). **Sweep.**
48. `help_texts_name_the_new_keys_and_nothing_old` now also carries the series
    assertions, so its name undersells it. Cosmetic; **sweep.**

**From the Phase 5 review (2026-09-22, no blocking findings):**

49. Six series cells differ by 0.1 when recomputed from the one-decimal win%
    beside them (e.g. starter vs sovereign: 23.2 → 8.5 by hand, 8.6 in the
    table), because the report converts the unrounded rate. And the new run's
    per-match rates are not written into the doc, so applying the formula to the
    older measured-curve table gives different figures (starter vs Greeb 77.2,
    not 78.8). The section's note says the runs differ; one clause ("computed
    from the unrounded rates of the 2026-09-22 run") would pre-empt a false
    discrepancy report. **Sweep.**
50. The bundle omitted half of T010's balance-value gate. **Closed by the
    orchestrator** the same day: across the whole spec
    (`git diff main...HEAD`), `src/opponent.rs` is untouched and `src/economy.rs`
    removes only two doc lines and the `pub` on `cheapest_floor` — no constant
    changed. AC 20's balance-data half holds.
51. The largest per-match drift between runs, starter vs Rix 29.9 → 28.1, is
    ~2.8 standard errors — plausible as the largest of 50 cells with no fixed
    seed. If a later run drifts the same way again, look at the engine before
    blaming chance.

**From the Phase 6 review (2026-09-22, no blocking findings):**

52. **`assets/CREDITS.md` names no art tool** — a visible marker stands where the
    name goes. The person's to supply; **a merge gate for the second sweep**.
    **Closed 2026-09-22**: the person named Opus 5.5, now in `CREDITS.md`.
53. **A `.DS_Store` trap.** AC 21's checklist test lists `assets/planets/` with
    `read_dir` and demands an exact match, and `.gitignore` hides `.DS_Store` —
    so if Finder ever opens that folder, `cargo test` fails on this machine
    while `git status` shows nothing. Skipping dotfiles would remove the trap.
    **Second sweep to rule.**
    **Fixed 2026-09-22 (T014a)**: hidden files are skipped; a misnamed file
    still fails.
54. `spec.md`'s brief-as-deliverable paragraph still said the art non-goal
    "stands"; R9 superseded it. **Corrected by the orchestrator** with a pointer.
55. **The `.gitattributes` rationale was half wrong in the record.** A CRLF
    checkout would *not* mis-size the art — `str::lines` strips `\r\n` and the
    drawer iterates `lines()` — but it *would* fail the new `\r` assertion, so
    `cargo test` would break on Windows with `autocrlf=true`. That alone
    justifies the rule, and `-text` is right. The portraits have no such
    protection, which is fine only because they have no `\r` test.
56. Nothing can test `CREDITS.md`'s originality claims; they rest on the
    person's look at the art.

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
| plan + tasks re-review (skeptical-reviewer) | opus → opus | 92K | 1 | — | **1 blocking (B6)** | B1's reshaped rule verified correct on all four cases (locked node, beaten rematch, un-beaten node with no series, Quick Play — which cannot reach it, since `resolve_match` returns `None` with no in-flight pointer); the claimed property and the Phase 1 marking flip both confirmed. **B6**: the B4 fix widened T002's grep to `src/ tests/ docs/` without re-running it — `src/wager.rs` names `cheapest_floor` twice, including a production doc asserting the pre-O1 rule, while plan §Files listed `wager.rs` under No change. T002's and T011's gates were unsatisfiable. Nine second-look notes |
| B6 + second-look fixes (the orchestrator) | opus → opus (session) | — | — | — | — | The review-loop cap was spent, so the orchestrator applied these rather than opening a third pass, per CLAUDE.md. **B6**: `src/wager.rs` moved out of plan §Files' No change as a doc-only edit owned by T002 (correct the `reserve` doc and the test-helper doc to name `reserve_floor` and the locked-opponent floor); exempting it from the grep was rejected as satisfying the gate while leaving the false claim standing. **Second-look, applied**: T002's caller count seven → eight and the file list gains `wager.rs`; T001's literal-site count twice → three times (~397, ~414, ~421); T001's Verify gains the one named carve-out for `take_stake_empties_the_escrow_exactly_once`, whose assertions change shape by design (same contradictory-gate class as B2); T003's two-kinds bar gains the helper-mutation clause that was its one leak; both `self.screen` greps switched to `grep -nE` for BSD grep on darwin; the tier-log header corrected after the Phase 1 flip; the Phase 1 pause text now warns that the lock does not exist until Phase 2, so switching opponents mid-series silently replaces the series. **Second-look, deliberately left open → pre-merge sweep**: (8) `no_settled_match_beats_an_unbeaten_opponent_outright` lives in `campaign.rs` but the property it names is jointly pinned with T003's test, so the name overclaims for where it sits; (9) the `\|\| (NotInSeries && player_won)` clause in `beats` is documented as provably a no-op, and a branch whose own comment says it does no work is the mild smell CLAUDE.md's Simplicity section names |
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
| **Amendment replan** (sdd-planner) | opus → opus | 186K | 1 | — | — | Mid-implementation revision for `spec.md`'s *Amendment, 2026-09-21* (the person's rulings R1/R2/R3 after walking the venue). Rewrote plan §Design tension 7, §Design 4 and §Design 5 for horizontal bands, pinned the arithmetic at both fit sizes, added T004a/T005a/T005b, and **enumerated seven existing venue assertions with their old and new values** rather than leaving them to be discovered — the failure this spec hit twice. **Deleted** `VenueRail` and `VENUE_ART_W` rather than making them unconditional, on the grounds that a one-variant wrapper whose doc describes a condition that no longer exists is the false-name defect this spec renamed four symbols to avoid. Left the placeholder question open, as instructed |
| Amendment sign-off (skeptical-reviewer) | opus → opus | 131K | 1 | — | **1 blocking (B1)** | Recomputed all ten lines of the arithmetic independently at both fit sizes and confirmed every cell, area and margin; checked the rename gate against the actual tree (every enumerated site real, nothing un-enumerated inside scope, the one carve-out genuine); counted the hint row and the action row by hand. **B1**: the revision generalized R2's carve-out from *earlier* specs' documents to all of `specs/**`, sweeping in **spec 029's own `spec.md`** — leaving AC 3, AC 10, the venue entity and the venue's four actions naming a button T005b deletes, with no gate covering `specs/` and nobody assigned to reconcile it. The document-level form of the defect R2 exists to remove, and criteria the person ticks by hand. 8 second-look notes |
| Amendment B1 + note fixes (the orchestrator) | opus → opus (session) | — | — | — | — | **B1 resolved by correcting `spec.md`, not by widening the gate**: its live prose now says Card Shop throughout, while the dated ruling records and the spec-012 reference keep the old word deliberately, with the distinction stated in the status block. The carve-out in plan §Files and T005b narrowed to *earlier* specs. **Four notes folded into task lines**, each because it would otherwise make a task stop or fail its own gate: `rows_for` becomes dead code against T004a's no-warnings bar (delete it there); `draw`'s own method doc is false in three ways, not just the module doc; the replaced test's `center_x < cols` assertion named as dropped on purpose; T005a must add the `series_length_label` import and replace a doc whose intra-doc link `cargo build` cannot catch. One wording fix in both files: the re-walkthrough claimed the venue uses "the map's words", but the map has no such words until T007 |
| Amendment re-review (skeptical-reviewer) | opus → opus | 55K | 1 | yes | **0 blocking** | **Signed off.** B1 resolved and the correction verified complete — ten remaining "Outfitter" occurrences in `spec.md`, every one a dated record or a pointer to one — and the fix found to have gone **wider** than the finding, correcting three passages B1 had not named. All five applied notes verified against the source. Three residual prose errors returned and fixed: R2's paragraph undercounted its own correction as "four mentions", the status block's "above" pointed the wrong way, and T004a's justification named `in_bounds` where the fit test is what actually covers the dropped assertion. **Carried to the sweep**: S4 (`FOOTER_H`/`action_y` double encoding), S5 (nothing asserts the art draws at 89 columns — R3's most load-bearing change), S7 (`shop.rs` and `app.rs` doc staleness predating this amendment) |
| T004a (sdd-implementer) | opus → opus | 83K | 1 | yes | — | The bands, across `layout.rs` and `venue.rs`. **Its own arithmetic agreed with the plan's table at both sizes** — art `Rect::new(3, 60, 4, 26)` = 58×23 = 1334 cells at 89, `Rect::new(3, 110, 4, 26)` = 108×23 = 2484 at 139, portrait 330 at both, gap columns exactly `PANEL_GAP`. `VENUE_ART_W` and `VenueRail` deleted; both greps empty. **Every changed assertion mapped to its enumerated item in a table**, and nothing off the list moved — the enumeration did the job it was written for, and this is the first task in the spec where a deliberate assertion change caused no stop. Orchestrator confirmed by rendering the venue at both widths: the art is unmistakably the dominant element. **Three findings**: `FOOTER_H` is `pub` per the plan with no reader outside `new` (glance at the phase review); the frame test needed `row_text`, copied from `board.rs`'s test module, now in three modules and a candidate for a shared helper; `the_art_region_dominates_at_both_widths` destructures `fit_sizes()` as `[narrow, wide]` and would silently invert if that order ever changed |
| T005a + T005b (sdd-implementer, one dispatch) | opus → opus | 57K | 1 | yes | — | R1 and R2, done in order in one dispatch and reported separately. **T005a**: exactly the two named assertion values changed, plus one *added* assertion that `series_line` contains `series_length_label`'s output — so if either screen's phrasing later drifts from the map's, the test fails rather than two literals quietly disagreeing. **T005b**: nine files, **no assertion changed value** — the only test-file touch is an assertion message, `overlay.rs` is untouched and its two asset tests stayed green unedited, and the implementer verified the width claim directly (both asset lines 46→46 and 45→45 characters, both files' line counts unchanged). `shop::TITLE` hoisted to a `pub const` read by the venue's `ACTIONS`, so the button and the header cannot disagree. Grep gate returns exactly the one permitted survivor. **Committed together**, not separately: both touch `src/venue.rs` and the tree could not be split without interactive staging. **One finding folded into T009**: the readme is tracked as `Readme.md`, and T005b's gate spelled it `README.md` — harmless on darwin, silently unsatisfiable on a case-sensitive filesystem |
| **Phase 2 amendment review** (skeptical-reviewer) | opus → opus | 95K | 1 | — | **0 blocking** | **Signed off.** Checked the density rule against the **rendered frames** rather than the arithmetic — both snapshots omit exactly rows 27 and 29, so the action row has real air and the header and footer stay compact — and confirmed the frame test has a working non-vacuity guard (a no-series early return would still pass the blank assertions but fail the filled ones). Recomputed AC 16 independently and matched every box edge to both frames: art 1334 cells at 89 and 2484 at 139 against the portrait's 330, largest at both widths and strictly larger at 139, with the band's centre landing exactly on `center_x` so the header and footer text centre over it. Verified `shop::TITLE` has **one definition and exactly two readers**, so the button and the header are unable to disagree rather than tested to agree. **Checked `spec.md` itself** and confirmed the sign-off's document-level catch is genuinely closed: every surviving "Outfitter" is in the set the status block declares keeps the word. Noted the change runs *against* over-engineering — a wrapper type deleted, a parameter dropped. 7 second-look notes, recorded above |
| **Second amendment replan** (sdd-planner) | opus → opus | 246K | 1 | — | — | R4/R5/R6 plus the art brief. Re-pinned the arithmetic: art **1100** cells at 89 and **2068** at 139, **17.5%** and **16.8%** down from 1334/2484 — both inside the person's 15–20%. **Solved the R5 problem rather than deferring it**: the art's centre is `(cols−26)/2` regardless of margin, so the 63-character hint centred there would sit flush against column 0, which `spec.md` forbids; shortened to 55 using `board.rs`'s own single-space `·` idiom, with four alternatives rejected in writing. Width became a **fraction** rather than a fixed inset, diverging from the plan's own "a subtraction, not a ratio" rule, because the workable inset ranges at the two widths (7–9 vs 12–17) do not overlap — recorded as the divergence it is. `center_x` → `text_x` because the old name asserted the terminal's centre. Decided **two art assets per planet**, and routed the doubled drawing cost to the person. **Flagged rather than edited** that R6 changed behavior with no criterion naming it; the orchestrator amended AC 3 |
| Second amendment sign-off (skeptical-reviewer) | opus → opus | 142K | 1 | — | **2 blocking** | Recomputed the whole geometry by hand and confirmed every cell, both percentages, and the hint at **55 characters counted one by one**. **B1**: T005d's Verify gated on `git diff --stat`, which **cannot list an untracked file** — the task's entire product is one new file, so the command prints nothing. **Sixth instance on this spec of a gate that could not produce its own output**, and the reviewer found this repo had already written the lesson down **twice**, in specs 022 and 023. **B2**: T005c's change list declared itself exhaustive and omitted four items, three of which would have shipped comments asserting the opposite of plan §Design 4. Also confirmed the strengthened fit-test bar is a real catch: the shipped assertion checks only the right edge while `saturating_sub` clamps a left overflow to column 0, so R5's forbidden failure would have passed. 6 second-look notes |
| Second amendment fixes (the orchestrator) | opus → opus (session) | — | — | — | — | B1 → `git status --short`, with both precedents named. B2 → all four items added. **S1 applied the stronger way**: R6's credit row was about to be a second literal of `shop.rs`'s wording with nothing able to catch a drift — the option R1 and R2 both rejected on this same screen — so `shop.rs` gains `credits_label`, the venue reads it, and a grep pins it. Plus spec.md's three stale "all the width left over" sentences, three arithmetic slips, and a vacuous `grep -c` that counted lines rather than occurrences |
| Second amendment re-review (skeptical-reviewer) | opus → opus | 61K | 1 | yes | **0 blocking** | **Signed off.** Both blockers resolved, all five notes correct, and the S5 clause confirmed moved out of T005d into T005c. Returned **four residual paper-trail items, two of which would have produced a wrong implementation**: `shop.rs`'s width fixture re-types the balance format a **third** time, so the new grep would match two lines while its stated rationale stayed false; and the shop's remainder is **double**-spaced in code while the plan wrote it single-spaced as shorthand, which taken literally would silently narrow a screen R6 does not touch. Both pinned in T005c. Also: the plan's files inventory and its "nothing else about the drawing loop moves" sentence, now corrected |
| T005c (sdd-implementer) | opus → opus | 92K | 1 | yes | — | R4, R5 and R6. **Its arithmetic agreed with the plan's table at every cell**, derived independently before editing: art `Rect::new(7, 56, 5, 26)` = 50×22 = **1100** at 89 (**17.54%** down from 1334) and `Rect::new(10, 103, 5, 26)` = 94×22 = **2068** at 139 (**16.75%** down from 2484), `text_x` 31 and 56 against the terminal centres of 44 and 69, clear columns 7 and 10. `HINT` at **55** characters with its left column at **4**. All five gates satisfied: `grep -n "center_x" src/venue.rs` empty, `grep -rn "Credits: ◈" src/` **one line** — `credits_label`'s own body, so the shared-wording claim is now a fact. **Every changed assertion mapped to its enumerated item and nothing unlisted moved** — the second time in a row the enumeration did its job, after being extended twice under review. Orchestrator confirmed by rendering both widths and the shop screen: the text sits over the art, and the shop's balance row is byte-identical. **Three findings**: the plan's dictated doc text uses British spellings where the surrounding code uses "center", so `layout.rs` now carries both within a few lines (a chore, not this task); `credits_label` returns `String` so it is not const-usable, which no caller wanted; and the strengthened fit assertion makes **61 characters** the hard ceiling on any future venue row at 89 columns, six over the current hint |
| T005d (sdd-implementer) | opus → opus | 62K | 1 | yes | — | The art brief, 307 lines, one untracked file — `git status --short` shows exactly the one line the sign-off's B1 fix asked for. Canvas **48×20** and **92×20**, re-derived from the layout test's pinned Rects with the derivation shown as a table and the test named, so a later reader can re-check it when the geometry moves. (h), (i) and (j) all present. **Four gaps it found in the task's own list and closed**: its first exemplar was hand-typed and wrong, so it rebuilt the block with a script that asserts each row's length — a brief whose one worked example teaches the wrong width is worse than none; the self-check command it first offered used `awk`, which counts **bytes**, and every palette glyph is multi-byte, so a correct file would have looked 3× too wide — replaced with a character-counting `python3` one-liner and an explicit warning; trailing spaces are load-bearing under the exact-width rule and do not survive a chat paste, so it added a "deliver files, not pasted grids" paragraph **without** adding a re-pad escape to the checklist, since the checklist must be enforceable without judgement; and it ruled figures out of the art, reading off AC 16's "as a separate element" so the art cannot compete with the portrait panel. **One bundle-quality finding for the orchestrator**: the bundle's excerpt of the layout test stopped before the pinned-Rect block the canvas derives from, and omitted `Rect::new`'s inclusivity — which is what makes the Rect 50×22 rather than 49×21 |
| **Second Phase 2 amendment review** (skeptical-reviewer) | opus → opus | 114K | 1 | — | **0 blocking** | **Signed off.** Re-derived every number independently and matched all of them; **counted the rendered box borders** to confirm AC 16 (50×22 and 94×22 art against 22×15 portrait) and read AC 3, AC 17 and R5 off the frames rather than the arithmetic — R5 now reads as deliberate, with a 15-character header leaving 16 interior columns left and 17 right instead of the 13-column drift the person saw. Confirmed the strengthened fit assertion closes a real hole and that the Card Shop renders byte-identically. **Verified the brief's illustrated exact-width exemplar by regex rather than by eye — correct to the character** — and its planet table canon against `PLANETS`. 7 second-look notes: three about the brief went into **T005e** rather than the close-out, because the brief's purpose is to be handed off unaided; one is an **orchestrator process miss on its third recurrence** (the review bundles have echoed `--lib` plus a warning-count grep that returns 0 for a failed build as readily as a clean one — the full command from here on); one is the **fourth** under-enumeration in this spec |
| T005e (sdd-implementer) | opus → opus | 67K | 1 | yes | — | The brief's three soft spots, fixed before hand-off. **(a)** Worked the intermediate-width arithmetic from the code rather than trusting the review's numbers, confirmed them, and **found more than the review did**: the gap is not confined to widths *between* the fit sizes — above 139 it reopens and grows without bound (160 columns → 9 blank each side), and the worst case is **138 columns at 21 and 22 blank columns**, numerically the same emptiness the brief cites for rejecting the single-asset option. Resolved honestly: the rule is now stated as exact at 89 and 139 and **unsettled in between**, with a table, four options priced, none chosen, and no mechanism invented. Closes by telling the design agent the decision changes nothing about the drawing — 48×20 and 92×20 are needed under all four. **(b)** Item 4 is now a closed **23-codepoint whitelist**, with the four ASCII marks chosen from the palette's own examples and line-alikes excluded because the brief already bars box-drawing glyphs as reading like a seam; the "sparingly" judgement is explicitly reassigned to the human look rather than pretended mechanical. New item for UTF-8 and LF-not-CRLF, and a **single command for both**, which the implementer ran against two scratch fixtures — clean file `lf-ok glyphs-ok`, a CRLF file with `│A` reported `U+000D,U+0041,U+2502`. **(c)** The crowd distinction restated above the per-planet table. **Three findings**, all for the deferred art spec: the above-139 case must be covered too; the 138-column worst case is worth knowing if anyone revisits the two-asset decision; and both canvas sizes are exact by one column, so any change to `MARGIN_X`, `PANEL_GAP`, `PANEL_W` or the 7/8 fraction invalidates them |
| **Session model change** (the person, `/model`) | opus → **claude-opus-5-5** (session) | — | — | — | — | 2026-09-22: the person switched **this session** to `claude-opus-5-5` with `/model`. Session-only by CLAUDE.md's own rule (project settings outrank a picker choice, and new sessions open from `.claude/settings.json`), so **no role-table row was edited** — this is not a request to move a role. Subagent dispatches carry no override and run at their definitions' `opus`. Recorded here so the tier log shows which model orchestrated from T005f on |
| T005f (sdd-implementer) | opus → opus | 44K | 1 | yes | — | Ruling R7. Art unchanged, gap exactly `PANEL_GAP`, group centred: portrait `(60, 81)` at 89 and `(107, 128)` at 139, outer margins 7/7 and 10/10. Four assertions changed, **all inside the sanctioned class** — stating the class rather than a list worked first time, after four under-enumerations. `venue.rs` correctly left untouched (its one centre comment involves no portrait position). Orchestrator rendered both widths: three clear columns at each |
| T005f review (skeptical-reviewer) | opus → opus | 38K | 1 | — | **0 blocking** | **Signed off.** Proved `span_w`, `art_w` and `art.x0` are identical to T005c's at **every** width, not just the two fit sizes, so the art Rects, `text_x` and both of the brief's derived tables are genuinely unchanged. 3 doc-wording notes to the sweep |
| T007 (sdd-implementer) | opus → opus | 46K | 1 | yes | — | Map detail row names `Best of 3` / `Best of 5` from `series_length_label` (the same source as the venue); `banner_line` prefixes the series result and prints today's strings byte-for-byte for the two series-free cases; `docs/economy.md`'s stale rename paragraph fixed. No assertion changed value. **One unsanctioned literal changed shape to compile** — a test in `app.rs` near line 2971 built the old tuple variant; the bundle did not name it (**fifth enumeration gap, a construction site this time**). Rule extracted to a pure `series_length_detail` helper so it tests without a terminal. The label shows on locked planets as well as launchable ones, per the task's "not cleared" — for the phase review |
| T008 (sdd-implementer) | opus → opus | 71K | 1 | yes | — | Series score on the status band's lower row, right-aligned, at both widths; `board_series_line` pure and tested without an `App`, including a stale pointer on a different planet. **Enumerated every `BoardView::draw` call site by grep before editing** (two, plus its own new test) — the practice this bundle asked for after T007's miss, and there was no gap this time. Longest turn prompt 64 + one blank + widest series line 12 = 77 inside the 81-cell band. No existing assertion changed. Deciding-match game-over frame left unchanged for the person |
| **Phase 3 review** (skeptical-reviewer) | opus → opus | 98K | 1 | — | **0 blocking** | **Signed off.** AC 2, 6, 11 and 12 met by the code; the series-free banners byte-for-byte the old strings; confirmed from `record_series_match` that a contradictory banner cannot happen in play; confirmed Quick Play clears the pointer, so "no campaign pointer" is a fact rather than a claim; and confirmed plan §Open questions 1 was left alone. 4 second-look notes |
| Phase 3 walkthrough (orchestrator, `run-kaazap`) | opus-5.5 (session) | — | — | — | — | Scratch `KAAZAP_DATA_DIR` throughout. **Attested**: fresh map shows `Cinder · Outer Rim   Best of 3` (AC 2); cleared Cinder shows no length and "Enter to rematch" (AC 12); `Series 0 – 0` and `Series 0 – 1` on the status band beside the prompt at **89** and **139** (AC 11); the non-deciding game-over frame shows the freshly updated `Series 0 – 1`; the **deciding** game-over frame shows no score (plan §Open questions 1, captured for the person); the deciding loss's banner `Series lost · Lost 10 credits` (AC 6); a rematch shows no score line and its banner is the plain `Lost 10 credits` (AC 11, 12). **Not attested on screen**: the `★  Series won` banner — six auto-played matches produced no series win; the string is pinned by `the_banner_names_the_series_beside_the_settlement` |
| T009 (sdd-implementer) | opus → opus | 55K | 1 | yes | — | The rule written into the primer (10 → 14 lines, the one sanctioned assertion change), How to Play (24 → 26 of its 27-line ceiling) and the Readme. **Used R1's "Best of 3"/"Best of 5" over the plan's pre-R1 wording** — plan §Design 9 predates the amendment — and the plan's own How to Play line was 53 columns against the file's widest 45, so How to Play carries the phrase without the counts; the primer carries both. Readme: the campaign paragraph now names the series, the venue, the lock and the locked-opponent floor; the status blockquote T005b broke was reflowed whole, text unchanged. **Findings**: How to Play has one line of headroom left; plan §Design 9's strings are stale against R1 |
| Phase 4 walkthrough (orchestrator, `run-kaazap`) | opus-5.5 (session) | — | — | — | — | Fresh scratch profile (`…/scratchpad/kz4`) at **89×31**. **Attested**: the first-campaign primer opens on first campaign entry, reads "Each opponent is Best of 3: win 2 matches of 3. / The last is Best of 5: win 3 of 5. A series, / once started, is played out.", fits with one blank row above and below its content and one above the dismiss line, nothing clipped; How to Play from the menu fits rows 0–29, carries "Opponents are Best of 3, the last Best of 5, / and a series, once started, is played out.", nothing clipped. The map's detail row behind the primer showed `Best of 3` for Cinder |
| **Phase 4 review** (skeptical-reviewer) | opus → opus | 41K | 1 | — | **0 blocking** | **Signed off.** Ruled that How to Play's "Best of 3" satisfies AC 18 — R1 made it the single phrase for the idea — and that restoring "2 match wins of 3" would reintroduce the second phrasing R1 removed; the real weakness is the missing unit ("matches"), routed to the person. Compared the reflowed Readme blockquote word by word: identical. 5 second-look notes; plan §Design 9's pre-R1 strings updated by the orchestrator |
| T009a (sdd-implementer) | opus → opus | 23K | 1 | yes | — | Ruling R8: How to Play's two campaign lines now say "Best of 3 matches". 26 lines, 44/42 columns, no assertion moved. Covered by the Phase 4 review's own suggested text; no separate review |
| T010 (sdd-implementer) | opus → opus | 49K | 1 | yes | — | `series_rate` and its guard test with the spec's flagged vectors; a `series%` column in the report; simulator re-run at **N = 10,000**, release profile, **no lever moved**, still `targets 8/8, coupling 1/1, bounds 5/5`, per-match rates within 1.8 points of the recorded table. `docs/balance.md` gains *Series rates*, additions only. Headline: starter vs Greeb 70.3 → **78.8 %** as a series, starter vs Rix 28.1 → **19.3 %**, and the Sovereign's best of five turns the starter's 23.2 % into **8.6 %** while the full deck's 51.3 % becomes 52.4 %. **Findings**: the run takes ~9 s in release, not minutes; the per-match table above it is still the 2026-09-13 run, and the new section says so |
| **Phase 5 review** (skeptical-reviewer) | opus → opus | 43K | 1 | — | **0 blocking** | **Signed off.** Recomputed the closed forms and eight table cells by hand; checked all 50 doc cells against the run; confirmed every region against `PLANETS`; confirmed best of five is chosen from the game's own `wins_needed`, so the report cannot drift from the game. 3 second-look notes; the one gap in the bundle (the `economy.rs` half of the balance gate) closed by the orchestrator |
| T011 close-out (sdd-implementer) | opus → opus | 206K | 1 | yes | — | Drafted `closeout-main-docs.md` (ROADMAP edits anchored to `main` at `a5a854c`, the DECISIONS entry with every ruling A1–Q and R1–R8, the design calls, the O1 side effect, the unanswered Open question 1, and the gate pattern), triaged all 51 notes (10 resolved, 17 DECISIONS, 1 ROADMAP, 23 sweep) plus 10 new sweep items, ran `cargo test -q` three times green (496 passed, 1 ignored), and ran every mechanical check: no forbidden file touched, both versions still 1, two new `serde(default)` fields, `cheapest_floor` only in `economy.rs`, the single-assignment grep exactly two lines, and **0 warnings on `main` and 0 on the branch** via a throwaway worktree, since removed. Checked all 20 ACs with evidence, four with recorded caveats. **Found N1** — the wager warning over-fires on a series-deciding match |
| **Pre-merge sweep** (skeptical-reviewer) | opus → opus | 204K | 1 | — | **1 blocking (B1)** | **Found the cross-phase defect no phase review could see**: the floor rises the moment a series starts, and the map's launch never checked the balance against it — a player with 15 credits could press Enter on a 20-ante planet, be locked into a series with no playable match, and lose the run at the next entry without staking a credit. It falsified plan §Design 2 and §Open questions 5, which sign-off had verified by reasoning about two seams and missing the third door. Ruled **N1 fix-now** (inside the footprint: it changes only when the warning shows, and repairs this spec's own change to the warning's input), N2 and N8 as ride-alongs, everything else leave-and-record with reasons, and **nothing for the person**. AC 20 confirmed across the whole diff |
| T011a (sdd-implementer) | opus → opus | 57K | 1 | yes | — | B1: one `refuse_uncovered` helper for both the map's series launch and every wager, checked before `begin_series`. N1: `economy::reserve_after_a_loss` (records the loss on a copy of the run so the series' own rule decides) with a six-value test whose two deciding cases would fail under the old behaviour. Doc ride-alongs in five files. No assertion value moved |
| Sweep re-review (skeptical-reviewer) | opus → opus | 42K | 1 | yes | **0 blocking** | **Signed off; clear to merge from the code side.** Checked the six test values against the series rule by hand. Its one open item — a driver check of B1 — was already done by the orchestrator |
| Sweep fixes, driven (orchestrator, `run-kaazap`) | opus-5.5 (session) | — | — | — | — | Scratch `…/scratchpad/kz5`. **B1**: 15 credits, Cinder cleared, Enter on Ashfall (Vessa, ante 20) → the map banner "Can't cover the 20-credit ante", no venue, profile `series: None`, credits unchanged; Enter on Scree (Dax, ante 10) still opens the venue, as it should. **N1**: locked on Rix (ante 50) with 80 credits — at **0–1** a 50 stake shows **no** warning (a loss decides the series and releases the lock); at **0–0** it shows "Lose this and the run is over." The sweep's example stake of 40 is below Rix's 50 ante, so 50 was used |
| **Ruling R9** (the person, at the merge pause) | — | — | — | — | — | Integrate the per-planet art in this spec rather than merge on the placeholder; asked the same day, chose **the box fits the art** over centring it in a larger box or commissioning more sizes. `spec.md` amended (R9, AC 21, the non-goal narrowed to authoring); T011 re-opened |
| Phase 6 replan (sdd-planner) | opus → opus | 246K | 1 | — | — | Phase 6: T012 geometry (runs now, `review: per-task`), T013 the delivery checkpoint, T014 load and draw. Pinned six sizes, the fit sizes identical to today. Flagged three `spec.md` sentences R9 made false — corrected by the orchestrator before sign-off — and two choices for the person (the art's weight, vertical centring on tall terminals) |
| Phase 6 sign-off (skeptical-reviewer) | opus → opus | 145K | 1 | — | **3 blocking** | Recomputed all six sizes; confirmed T012 genuinely needs no art files and T014 cannot compile without them. **B1**: a grep gate that matched `FIELD_MARGIN_X` and so could never pass. **B2**: a "clean working tree" check impossible for an uncommitting implementer. **B3**: `design/brief.md` makes opponent portraits the *single* exception to "no block-art", amended before spec 016 shipped them — R9 needs the same amendment and none was scheduled. 8 notes, including a real Windows defect: a CRLF checkout would mis-size every art line |
| Phase 6 fixes + re-review | opus → opus | 41K | 1 | yes | **0 blocking** | B1 word-matched; B2 scoped to `assets/planets`; **B3: the orchestrator wrote the `design/brief.md` amendment**, bounded like the portraits', its text going to the person at the Phase 6 pause; all notes applied, `.gitattributes` added to T014. Re-review signed off; its one note (`.gitattributes` missing from T014's file list, so the rule could silently not ship) fixed before dispatch |
| T013 (sdd-implementer) | opus → opus | 29K | 1 | yes | — | **The art delivery passes the brief's checklist, items 1–6, first time**: exactly the sixteen expected names; 20 lines each with one trailing newline; every line exactly 48 or 92 characters (counted as characters, not bytes); only the 23 whitelisted codepoints, valid UTF-8, LF only (the brief's own one-command check, verbatim); eight distinct narrow grids and eight distinct wide. Nothing bounced. Validated where the files already were — committed early in `7445c68`, see the note above T013 — and `git status --short assets/planets` empty. Item 7, the person's look, is the Phase 6 pause |
| T012 (sdd-implementer) | opus → opus | 81K | 1 | yes | — | The art box is exactly the drawing plus its border at every size (R9); the whole venue centres vertically on taller terminals. Arithmetic agreed with all six rows of the plan's table. Only the three sanctioned assertion changes; a new test pins every size from the minimum (660 of them). **Orchestrator captured the running venue before and after at 89×31 and 139×31 — identical, character for character** |
| T012 per-task review (skeptical-reviewer) | opus → opus | 62K | 1 | — | **0 blocking** | **Signed off.** Recomputed all six sizes from the code; confirmed nothing references `assets/planets`. Notes: two docs describe AC 21's test in the present tense before T014 adds it — **T014's review must confirm they are true**; the brief says "a field on `Planet`" where T014 adds two; the 31-row block and `Config::min_size()` agree by arithmetic, guarded by a test |
| T014 (sdd-implementer) | opus → opus | 66K | 1 | yes | — | Each planet's venue draws its own art: two `include_str!` fields on `Planet`, picked by the layout's `wide_art`, drawn through the portraits' clip-safe drawer; the planet name kept only as a fallback. Four tests, including AC 21's checklist test, which derives the canvas from `VenueLayout` (both sentences that already claimed it are now true) and checks CR **before** the palette. **Both mutation checks failed as they should** — an extra space failed on width, a CRLF file on the `\r` assertion — and both files were restored. `.gitattributes` added. No existing assertion changed. **One gap only the person can fill**: `assets/CREDITS.md` needs the name of the tool that drew the art, which is recorded nowhere; left as a visible marker and put to the person at the Phase 6 pause. Orchestrator rendered Scree's venue at 89×31: the drawing fills its box exactly |
| **Phase 6 review** (skeptical-reviewer) | opus → opus | 85K | 1 | — | **0 blocking** | **Signed off.** Confirmed AC 21 end to end — the drawing and its box come from one decision, the canvas is derived, all 660 sizes fill exactly, and the drawn-frame test catches a fallback to one planet — and **confirmed the three present-tense sentences T012's reviewer handed over are now true**. Checked the accidental commit `7445c68`: exactly the 16 art files plus the three spec documents, nothing else swept in. The art stays inside the design record's new exception. 5 notes: the CREDITS tool name is a merge gate for the person; a `.DS_Store` trap in the file-list check; one stale spec sentence (fixed); the `.gitattributes` rationale corrected in the record |
| Phase 6 walkthrough (orchestrator, `run-kaazap`) | opus-5.5 (session) | — | — | — | — | Scratch `…/scratchpad/kz6`, a series hand-set on each planet in turn. **Rendered all eight planets' venues at 89×31 and 139×31** (sixteen frames): every one shows its own drawing filling its box, the portrait beside it, nothing clipped, no planet name in the box. Plus Cinder at **120×31** (the narrow box, spare columns either side of the group) and **139×40** (the whole venue centred vertically, header from row 4, hint on row 34). Frames sent to the person for the item-7 go/no-go |
| **Phase 6 pause — the person's answers** (2026-09-22) | — | — | — | — | — | The art tool was **Opus 5.5** — written into `assets/CREDITS.md` in place of the marker (close-out note 52 closed). The person had **Cinder's and Scree's art redrawn** "with right angles only" and committed it locally (`bc2598c`, unpushed), then said to continue — read as the item-7 go on the art. The redraw was validated like the first delivery: the brief's items 2–5 by shell on the four files, and AC 21's test over all sixteen — all green, 488 lib tests. **Not ruled on, left as built**: the art's weight (plan §Open questions 7 — full plain weight), vertical centring on tall terminals (§Open questions 8), and the `design/brief.md` amendment text, which was shown and not edited |
| Close-out refresh (sdd-implementer) | opus → opus | 151K | 1 | yes | — | T011 re-run as a refresh for R9: §1a–§1c and §3 corrected (main's deferred art bullet marked shipped, not removed), R9 / AC 21 / the design-record amendment / the Phase 6 answers / the early-commit miss added to the DECISIONS draft, notes 49–56 triaged, AC 21 ticked with evidence, every check re-run fresh — 502 tests, 0 warnings on main and on the branch |
| **Second pre-merge sweep** (skeptical-reviewer) | opus → opus | 197K | 1 | — | **1 blocking (B1)** | Scoped to everything since R9. **B1**: the close-out's DECISIONS text said the person "chose the loading rule from the four the brief priced" — the opposite of every other record: they chose a fifth, **the box fits the art**, over all four. Would have shipped to `main` as the permanent record. Ruled the `.DS_Store` trap **fix-now** (one-line filter). Found the cross-phase seam clean in code — nothing from Phases 1–5 still makes the box grow — with three stale present-tense sentences in documents. AC 20 and AC 21 hold |
| Second sweep fixes | opus → opus + session | 29K | 1 | yes | — | **T014a (sdd-implementer)**: hidden files skipped in AC 21's file-list check, proven both ways — a `.DS_Store` now passes, a misnamed `cinder-narrow.text` still fails — plus one stale test comment. **Orchestrator**: B1 corrected in the close-out; the three stale sentences in `spec.md`, the brief and the plan given pointers to R9; note 52 marked closed; note 53 marked fixed |
| Phase 2 amendment re-walkthrough (the orchestrator, `run-kaazap`) | opus → opus (session) | — | — | — | — | Driven at **89×31 and 139×31** with `KAAZAP_DATA_DIR` at a scratch directory. **Attested**: the series row reads `Series  0 – 0   ·   Best of 3` (R1); the second action reads **Card Shop** and taking it from the action row — not just `b` — opens a screen headed **Card Shop**, with Esc returning to the venue (R2, AC 3); the art region draws at **both** widths as the dominant element with the portrait in its own column beside it, sharing a top edge, nothing clipped, the header text centred over the band and the footer below it (R3, AC 16); rows 27 and 29 blank around the action row at both widths (AC 17); Play still opens the wager over the venue and Esc still returns to it. **Still not hand-attested**: the deck-guard divert (second-look note 23). The driver cannot switch the briefcase's focus — `key:\t` reaches the pty but the cursor stays in the Collection pane — so the deck could not be made invalid. Verified by inspection **twice** (the Phase 2 review and the amendment review); left for the person, who can do it in two keystrokes |
| Deck-guard divert — **attested by the person** | — | — | — | — | — | Second-look note 23, the one item two driver attempts could not reach: the person made their deck invalid and pressed Play at the venue, and it took them to the collection as expected. **Closed by attestation**, after being verified by inspection in two reviews. Nothing further is owed on it |
| **Phase 2 review** (skeptical-reviewer) | opus → opus | 161K | 1 | — | **0 blocking** | **Signed off.** Did all three checks the handoff note requires. (1) Ran the single-assignment grep itself and **widened it three ways** — `self\.screen\s*=` across `src/` (13 hits, only two set a campaign screen), the bare variant names, and `mem::replace`/`&mut self.screen` — confirming the gate is load-bearing rather than narrow, and **upheld T006's deviation** from the plan's listing: in the expression form neither line contains the variant name, so the gate would return zero and pass while checking nothing. (2) Density verified as instrumented rather than asserted, against the constitution's *corrected* form (only the acted-on line gets air), and both modals the venue can raise already pad evenly through `OverlayLayout`. (3) **Closed the Phase 1 caveat** by walking every door, including a quit and re-entry, a Quick Play match started mid-series, and the deck-builder divert — `launch_from_map` is the only production caller of `begin_series` and is unreachable while a series runs. Also **verified plan §Open question 5 independently**: `CantCover` is unreachable from the venue because the venue's floor *is* `reserve_floor` while locked, and the missing banner-clearing line makes nothing worse, because the map's arm clears the banner before `launch_from_map` runs. 7 second-look notes; one gave `docs/economy.md`'s rename fallout to T007 |
| Phase 2 walkthrough (the orchestrator, `run-kaazap`) | opus → opus (session) | — | — | — | — | Driven at **89×31 and 139×31** with `KAAZAP_DATA_DIR` at a scratch directory (`…/scratchpad/kz2`); the real profile was never in play. **Attested**: AC 2 (venue at 0–0, credits still 50 and `in_progress: None` — arriving stakes nothing); AC 3 (the four rows, the four actions, `b` → Outfitter and `c` → collection both returning to the venue, and the Outfitter reading `spendable ◈ 40` = 50 − Greeb's ante, the locked floor); AC 4 (Play → Esc → venue, balance untouched); AC 5 (0–1 and 1–1 both returned to the venue); AC 6 and AC 7 (the second win landed on the **map** at `1/8 cleared` with `beaten: {cinder: [greeb]}`); AC 8 (a deciding loss landed on the map, opponent un-beaten, only the stakes gone); AC 9 (Start Campaign → Continue went **straight to the venue**, map never drawn, twice); AC 12 (cleared Cinder → wager directly, "Tournament Hall" absent); AC 13 (quit at the venue and return → same score; a match killed mid-play resumed at Score 3–1 with the same hand, then its finish decided the series and routed to the map); AC 16 (89 text-only and nothing clipped; 139 drew the bordered art region holding "Cinder" with the portrait beside it, sharing rows 8–22, a clear gap, both on-frame); AC 17 (blanks at rows 15 and 17 only, around the action row). **One item left uncovered**: the deck-guard divert of second-look note 23 — the driver cannot deliver Tab to the app, so the deck could not be made invalid from the venue. The Phase 2 review verified that routing by inspection; the person can try it by hand. **Driver notes**: the scratch purse was raised to 2000 twice so a run-over reset could not destroy the walkthrough, and each match ran after clearing the scratch `saves/`, since the driver's kill leaves one |
| Phase 2 walkthrough — an incidental finding | — | — | — | — | — | With a live series at 0–0 and otherwise starter state, **Start Campaign skips the Continue / New Campaign / Reset Everything panel** and goes straight to the venue, because `Profile::differs_from_starter` reads `campaign.has_progress()` and a series is not progress by that definition. Not a spec violation — AC 9 says entering the campaign goes to the venue, which it does, and AC 14's resets still clear the series — but a player who starts a series and immediately quits has no visible door to New Campaign until they play a match. **For the sweep to rule on** |
