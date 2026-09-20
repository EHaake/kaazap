# Tasks: Tournament rounds — spec 029

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

## Phase 1 — The series in the run (foundational; walkthrough: none — nothing creates a series yet, so every campaign match still launches from the map, settles and clears a planet exactly as it does today; the phase's whole claim is that nothing the person can see changed, and that is what the unedited existing tests pin)

<!-- T001 is the persisted model, T002 the one floor, T003 the settlement rule.
After T003 a win still beats an opponent, because with no series every match
settles through the NotInSeries path. -->

- [ ] **T001** — `src/campaign.rs`: the series, and the exactly-once settlement
  take. `review: per-task`. Per plan §Design 1 and §Design tension 3:
  `pub struct Series { planet: String, opponent: String, player_wins: u32,
  opponent_wins: u32 }` deriving `Debug, Clone, PartialEq, Eq, Serialize,
  Deserialize`; `pub enum SeriesOutcome { NotInSeries, Continues, Won, Lost }`
  deriving `Debug, Clone, Copy, PartialEq, Eq`; `pub const FINAL_OPPONENT:
  &str = "sovereign"`; `pub fn wins_needed(opponent: &str) -> u32` (3 for
  `FINAL_OPPONENT`, else 2) and `pub fn series_length_label(opponent: &str) ->
  &'static str` (`"Best of 5"` / `"Best of 3"`). On `CampaignRun`, add
  `#[serde(default)] series: Option<Series>` and `series()`, `begin_series`,
  `record_series_match` exactly as the plan's listing. On `NodeRef`, add
  `#[serde(default)] pub settled: bool` with the plan's doc.
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
  `record_series_match_ignores_a_node_that_is_not_the_series`
  (`NotInSeries` for a different planet/opponent, and the tally does not move);
  `take_settlement_hands_over_the_match_exactly_once` (the shape of the existing
  `take_stake_empties_the_escrow_exactly_once`, which this replaces: a staked
  node yields the node and the stake, then `None`, and `stake_at_risk()` is
  `None` after); `a_pre_029_node_is_unsettled_and_unstaked` (deserialize
  `{"planet":"cinder","opponent":"greeb"}` → `settled: false`, `stake: 0`, and
  `take_settlement` hands it over once); and a round-trip of a `CampaignRun`
  carrying a series. Do not run `cargo fmt`. (Copies: `campaign.rs`'s own
  `NodeRef`/`take_stake` doc voice and its tests module.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new tests passing and **every existing `campaign.rs` and
  `profile.rs` test passing unedited except the one `take_stake` call site**;
  `git diff --stat` shows only `src/campaign.rs` and `src/profile.rs`, with
  `profile.rs` changed on two lines; `grep -n "take_stake" src/ tests/` is
  empty; the implementer's report quotes `take_settlement` and
  `record_series_match` verbatim. The orchestrator re-runs the verification
  command itself before committing (per-task review), then dispatches a
  `skeptical-reviewer` on T001's diff alone before T002 starts.*

- [ ] **T002** — `src/economy.rs` + `src/profile.rs` + `src/shop.rs` +
  `tests/balance.rs`: one floor, and the four callers that read it. Per plan
  §Design tension 2: make `cheapest_floor` **private** (`fn`, not `pub fn`) and
  add `pub fn reserve_floor(run: &CampaignRun) -> u32` exactly as the plan's
  listing, with its doc naming ruling O1 and the reason. Then move **all four
  callers in the same task** — a `pub fn` whose only callers are
  `#[cfg(test)]` is `dead_code` against `cargo build --all-targets`, and a
  private `cheapest_floor` with no in-crate caller is too (spec 028's T003
  lesson): `Profile::is_broke`, `Profile::can_afford`, `shop.rs`'s `spendable`
  line, and `app.rs`'s `WagerState::new` reserve argument in
  `launch_campaign_node`. `tests/balance.rs` imports `cheapest_floor` for one
  report line — move it to `reserve_floor` (same value for a default run, so the
  report is unchanged). Nothing else in any of those functions changes.
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
  `dead_code` on either floor function; `cargo test -q` green verbatim;
  `git diff --stat` shows only those four files; `grep -rn "cheapest_floor"
  src/ tests/` matches **only** `src/economy.rs`; the implementer's report
  quotes `reserve_floor` and the four changed call sites verbatim.*

- [ ] **T003** — `src/profile.rs`: the settlement rule. `review: per-task`. Per
  plan §Design 3: `Settlement` gains `pub series: SeriesOutcome` with the
  plan's doc; `settle_campaign_match` becomes the plan's listing exactly —
  one `take_settlement`, then `record_series_match`, then the payout, then
  `mark_beaten` **only** when the series was `Won` or when there is no series
  at all and the match was won, with the completion edge unmoved inside that
  branch; `resolve_match` carries `series` into the `Settlement` it returns and
  is otherwise unchanged (it still records statistics before settling — spec
  024's ordering, which the first-clear record depends on). Existing
  `Settlement { … }` literals in `profile.rs`'s tests gain
  `series: SeriesOutcome::NotInSeries` — **that is the expected mechanical
  edit, and the only one**: no existing assertion about credits, progress,
  completions or the run tally may change value, because a run with no series
  settles through the `NotInSeries` path exactly as before this spec. If one
  does change, stop and report it rather than adjusting the expectation.
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
  each leave `series()` `None` from 1–0). Do not run `cargo fmt`. (Copies:
  `profile.rs`'s own `node`/`play_node`/`sweep_run` test helpers — use them
  rather than writing new ones.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the new tests passing and every pre-existing `profile.rs`
  assertion **unchanged in value**; `git diff --stat` shows only
  `src/profile.rs`; `git diff -- src/profile.rs` shows `resolve_match`'s
  record-then-settle order intact; the implementer's report quotes
  `settle_campaign_match` verbatim. The orchestrator re-runs the verification
  command itself before committing (per-task review), then dispatches a
  `skeptical-reviewer` on T003's diff alone before T004 starts.*

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
  `Screen::Venue { state: VenueState }`; the pure
  `enum CampaignHome { Map, Venue }` + `fn campaign_home(in_series: bool)` with
  the plan's doc; `fn open_campaign_home(&mut self)` — **the only place
  `Screen::CampaignMap` or `Screen::Venue` is constructed**, which is what keeps
  the return target from drifting (`open_campaign_map` is absorbed into it);
  `enter_campaign_map` renamed to `enter_campaign` and its body's screen line
  replaced by `open_campaign_home()`; `map_entry_modal` renamed to
  `campaign_entry_modal` (and its test's name); `BuilderOrigin::Map` →
  `BuilderOrigin::Campaign` and `BackTo::Map` → `BackTo::Campaign`, with
  `BackTo::Campaign => self.open_campaign_home()`; the Outfitter's
  `ShopOutcome::Back` arm → `self.open_campaign_home()`; all four
  `enter_campaign_map` call sites → `enter_campaign`. Split
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
  Test (plan §Tests), in `app.rs`'s tests module: `campaign_home_follows_the_lock`
  — the pure mapping both ways. **Pure; it must not construct an `App`.**
  Do not run `cargo fmt`. (Copies: `app.rs`'s own `Screen::CampaignMap` input
  arm and `back_destination` for the pure-routing-decision idiom.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/screen.rs`, `src/app.rs` and
  `src/deck_builder.rs`; **`grep -n "Screen::CampaignMap {\|Screen::Venue {"
  src/app.rs` shows construction in `open_campaign_home` only** (the match arms
  and tests may still pattern-match on them — the check is that nothing else
  *assigns* a campaign screen); `grep -rn "enter_campaign_map\|open_campaign_map\|
  launch_campaign_node\|BuilderOrigin::Map\|BackTo::Map" src/` is empty; the
  report quotes `open_campaign_home`, `launch_from_map` and the venue input arm
  verbatim.
  **PAUSE for the person** (after the Phase 2 review): the orchestrator drives
  the Phase 2 walkthrough in plan §Verification with the `run-kaazap` skill —
  `KAAZAP_DATA_DIR` pointed at a scratch directory, confirmed in the report —
  at 89×31 and 139×31, and reports in plain language: that Enter on an un-beaten
  planet opens the venue at 0–0 with the credit balance untouched; what the
  venue shows at each width (and that the art region and the portrait sit beside
  each other, unclipped, at the wider one, and that neither appears at the
  narrower); that `b` and `c` come back to the venue rather than the map; that a
  lost match returns to the venue with the score at 0–1; that quitting to the
  menu and re-entering the campaign lands back at the venue with the same score
  and the map cannot be reached; that winning two hands the player back to the
  map with the planet cleared; and that Enter on a cleared planet goes straight
  to the wager prompt with no venue.*

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
  what the map banner said after the match that decided a series.*

## Phase 4 — What the player is told (walkthrough: start a fresh campaign and read the first-campaign primer, then open How to Play — both now state the two-of-three rule, the three-of-five final, and that a series once started is played out)

- [ ] **T009** — `assets/primer_text.txt` + `assets/how_to_play_text.txt` +
  `src/overlay.rs`: the rule, written down. Per plan §Design 9: add the plan's
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
  each name the two-of-three rule and the three-of-five final. Change no other
  text and no other asset. Do not run `cargo fmt`.
  (Copies: `assets/primer_text.txt`'s own voice — short lines, no more than the
  current widest.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with all three `overlay.rs` fit/breathing tests passing;
  `git diff --stat` shows only the two assets and `src/overlay.rs`;
  `wc -l assets/how_to_play_text.txt` is **≤ 27**; the report pastes both files
  in full.
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
  case. Do not edit any existing row of `docs/balance.md`. Do not run
  `cargo fmt`.
  (Copies: `tests/balance.rs`'s own `ev_per_match`/`pct` for the pure-report-
  helper idiom; `docs/balance.md`'s *The measured curve* table for the new
  table's shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the guard test passing; the simulator run's output pasted in
  full in the report, with the N used; `git diff --stat` shows only
  `tests/balance.rs` and `docs/balance.md`; `git diff -- docs/balance.md` is
  additions only; `git diff main...HEAD -- src/opponent.rs src/economy.rs` shows
  no changed balance value.*

## Final phase — Spec close-out

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
  remembered (and why — spec 015's bug), one `reserve_floor` rather than two
  floor calculations, `take_settlement` extending spec 021's exactly-once
  *data* property to cover the series tally and `mark_beaten`, the series stored
  beside the in-flight pointer rather than inside it (so a Quick Play match
  cannot end a series), the required wins derived from the opponent id, and the
  four renames with their reasons. Record the two items the walkthroughs may
  have ruled on (plan §Open questions 1 and 2). Run `cargo test -q` three
  consecutive times and paste the tails. Mechanical checks (three-dot, since
  `main` may move): `git diff main...HEAD --stat` lists none of `src/game.rs`,
  `src/card.rs`, `src/player.rs`, `src/save.rs`, `src/opponent.rs`,
  `Cargo.toml`, `Cargo.lock`; `grep -n "VERSION" src/save.rs src/profile.rs`
  still reads 1 and 1; `grep -rn "serde(default" src/campaign.rs` shows the two
  new fields among the old; `grep -rn "cheapest_floor" src/ tests/` matches only
  `src/economy.rs`; `grep -rn "Screen::CampaignMap {\|Screen::Venue {"
  src/app.rs` shows one construction site each; `cargo build --all-targets`
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
criteria), one review plus at most one re-review. The Phase 2 review also checks
the venue against the constitution's *acted-on element stands apart* rule and
the modal's even padding, and checks by reading that `open_campaign_home` is the
only place a campaign screen is assigned.

**Pause cadence — when there's something to try.** Pause for the person after
**Phase 2**, **Phase 3** and **Phase 4**, once each phase's review and its
walkthrough are done. **Phases 1 and 5 are marked `walkthrough: none`** — Phase
1 changes nothing the person can observe (with no series yet created, every
campaign match behaves exactly as today, which is the phase's own claim and what
its tests pin), and Phase 5 touches only an `#[ignore]`d simulator and a
document — so they run through without a pause, and each appends its one-line
reason to the walkthrough list below. Their criteria are pinned by tests and by
the close-out's checks.

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
| **Economy profile** — the project's since 2026-09-19; spec 028 was the first spec under it (its log: planning 390K, implementer 891K over 16 dispatches, reviewer 553K over 9, total ≈1.83M, no tier misses). Every role resolves to `opus` / `claude-opus-5`; no per-call override anywhere, including the planner, the sign-off and the close-out; session at `claude-opus-5` medium. **Also the first spec under the `walkthrough:` pause cadence** — Phases 1 and 5 are marked `none` and run unpaused, so this log is the evidence for whether the marking held. | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | opus → opus | 272K | 1 | — | — | drafted; 11 tasks in 6 phases, Phase 1 foundational, T001 and T003 `review: per-task`, Phases 1 and 5 marked `walkthrough: none`; no product question returned; 5 design choices flagged for sign-off |
