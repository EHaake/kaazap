# Tasks: Tournament rounds — spec 029

> **Status**: Signed off (skeptical-reviewer, 2026-09-20 — one review, one re-review, B1–B6 all resolved; two second-look notes carried to the tier log and the pre-merge sweep). Ready for implementation.
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

- [ ] **T001** — `src/campaign.rs` + `src/profile.rs` + `src/app.rs` +
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

- [ ] **T002** — `src/economy.rs` + `src/profile.rs` + `src/shop.rs` +
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

- [ ] **T003** — `src/profile.rs`: the settlement rule. `review: per-task`. Per
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
  `src/profile.rs`; `git diff -- src/profile.rs` shows `resolve_match`'s
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

## Phase 2 — The venue and the lock (walkthrough: Enter on an un-beaten planet now opens the venue at 0–0 with nothing staked; play matches from it, come back to it between them, open the Outfitter and the collection and return to it, quit to the menu and re-enter the campaign to land back at it — and win the series to be handed back to the map)

<!-- T004 is the geometry, T005 the screen module (pub, so nothing is dead code
before it is wired), T006 the wiring and the one routing rule. The venue is
reachable only after T006. -->

- [ ] **T004** — `src/layout.rs`: the venue's geometry. Per plan §Design 4:
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

- [ ] **T005** — `src/venue.rs` (new) + `src/lib.rs`: the venue screen. Per plan
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

- [ ] **T006** — `src/screen.rs` + `src/app.rs` + `src/deck_builder.rs`: the
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

## Phase 3 — The series where the player already looks (walkthrough: the map's planet detail names what a launch commits you to before you take it, the status band carries the running score through every match of a series and nothing during a rematch, and the map banner after the deciding match names the series result beside the credits)

- [ ] **T007** — `src/campaign_map.rs` + `src/app.rs`: the map's detail line and
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
  Do not run `cargo fmt`. (Copies: `campaign_map.rs`'s own `banner_line` and
  `axis_line` for the pure-wording-function idiom.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/campaign_map.rs` and
  `src/app.rs`; the report quotes `banner_line` and the detail-row draw call
  verbatim.*

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
  and the series, and leave the rematch sentence true. Change no other
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
  the venue and the board show the score; J1/K1/L1; M1 as amended — a plain
  placeholder with the portrait **beside** it; N1 139 columns; O1 the locked
  floor; P1 campaign entry goes to the venue; Q music deferred), and the plan's
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
| | | |

| plan + tasks re-review (skeptical-reviewer) | opus → opus | 92K | 1 | — | **1 blocking (B6)** | B1's reshaped rule verified correct on all four cases (locked node, beaten rematch, un-beaten node with no series, Quick Play — which cannot reach it, since `resolve_match` returns `None` with no in-flight pointer); the claimed property and the Phase 1 marking flip both confirmed. **B6**: the B4 fix widened T002's grep to `src/ tests/ docs/` without re-running it — `src/wager.rs` names `cheapest_floor` twice, including a production doc asserting the pre-O1 rule, while plan §Files listed `wager.rs` under No change. T002's and T011's gates were unsatisfiable. Nine second-look notes |
| B6 + second-look fixes (the orchestrator) | opus → opus (session) | — | — | — | — | The review-loop cap was spent, so the orchestrator applied these rather than opening a third pass, per CLAUDE.md. **B6**: `src/wager.rs` moved out of plan §Files' No change as a doc-only edit owned by T002 (correct the `reserve` doc and the test-helper doc to name `reserve_floor` and the locked-opponent floor); exempting it from the grep was rejected as satisfying the gate while leaving the false claim standing. **Second-look, applied**: T002's caller count seven → eight and the file list gains `wager.rs`; T001's literal-site count twice → three times (~397, ~414, ~421); T001's Verify gains the one named carve-out for `take_stake_empties_the_escrow_exactly_once`, whose assertions change shape by design (same contradictory-gate class as B2); T003's two-kinds bar gains the helper-mutation clause that was its one leak; both `self.screen` greps switched to `grep -nE` for BSD grep on darwin; the tier-log header corrected after the Phase 1 flip; the Phase 1 pause text now warns that the lock does not exist until Phase 2, so switching opponents mid-series silently replaces the series. **Second-look, deliberately left open → pre-merge sweep**: (8) `no_settled_match_beats_an_unbeaten_opponent_outright` lives in `campaign.rs` but the property it names is jointly pinned with T003's test, so the name overclaims for where it sits; (9) the `\|\| (NotInSeries && player_won)` clause in `beats` is documented as provably a no-op, and a branch whose own comment says it does no work is the mild smell CLAUDE.md's Simplicity section names |

---

## Notes for the close-out (T011), gathered during implementation

<!-- Non-blocking observations from per-task and phase reviews. Not code
changes: they belong in `closeout-main-docs.md`'s DECISIONS entry or in a
roadmap follow-up, so the next person to touch this code finds them. -->

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
