# Tasks: Stats & records — spec 020

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

**Foundational phase:** Phase 1 (T001–T002 — the stats data model, the persisted
serde shape, and the recording/derivation logic the whole feature and the save
format rest on) is foundational. Under the constitution's review cadence the
default is a **per-phase** `skeptical-reviewer` pass. Two tasks carry
`review: per-task`: **T002**, the `Profile`/`CampaignRun` persistence contract —
a mistake in the serde shape, the no-version-bump additivity, or the
reset-preserves-stats amendment corrupts real `profile.json` saves or silently
wipes lifetime records, and every later file inherits the types; and **T003**,
the once-per-match recording seam, whose exactly-once / no-double-count /
correct-mode behavior a mistake would silently corrupt in every user's stats
(the direct analogue of spec 018's twin-call-capture per-task review). Every
other task — T001, T004, T005, and the close-out T006 — is reviewed at phase end.
Phase 2 has a single task (T003); its per-task review serves as the phase review.

---

## Phase 1 — Stats data model + recording/derivation logic (foundational)

<!-- Foundational: the persisted types + the pure recording/derivation logic the
recording seam (Phase 2) and the Records screen (Phase 3) both depend on. T002
(the Profile/CampaignRun persistence contract) gets a per-task review. -->

- [x] **T001 (foundational)** — Create `src/stats.rs` (add to `src/lib.rs`).
  Define `Mode { QuickPlay, Campaign }`; `OpponentRecord { match_wins,
  match_losses, round_wins, round_losses }` (all `u32`, `Serialize`/`Deserialize`/
  `Default`); `Streak { current, longest }` with `record(&mut self, won: bool)`
  (win → `current += 1`, `longest = longest.max(current)`; loss → `current = 0`);
  `ModeRecord { opponents: BTreeMap<String, OpponentRecord> }` (serde-default map)
  with `get(id) -> OpponentRecord` (default when absent), `entry_mut(id)`, and
  `totals() -> (u32, u32)` (match wins, losses summed); `LifetimeStats {
  quick_play, campaign, overall_streak, campaign_completions }` (all
  serde-default) with `record_match(mode, opponent_id, player_won, player_rounds:
  u32, opp_rounds: u32)` (bump the mode's `OpponentRecord`: match W or L,
  `round_wins += player_rounds`, `round_losses += opp_rounds`; then
  `overall_streak.record(player_won)`), `record_campaign_completion()`, and the
  read-side derivations `quick_play()`/`campaign()`/`overall_streak()`/
  `campaign_completions()`/`combined_opponent(id) -> OpponentRecord` (Quick Play +
  Campaign summed); `RunStats { match_wins, match_losses, round_wins,
  round_losses, streak }` (serde-default) with `record_match(player_won,
  player_rounds, opp_rounds)`; and the free `win_rate(wins: u32, losses: u32) ->
  Option<u32>` (`None` at zero matches, else rounded integer percentage). Plan
  §Design 1 / tensions §2, §5. (Copies the plain-serde-struct + methods shape of
  `campaign.rs`'s `CampaignRun`.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  (reported verbatim) — new tests: `Streak::record` raises/tracks/zeroes
  correctly; `record_match` credits a Quick Play win to Quick Play only (match-win
  + round W/L from the round counts) and a campaign loss to Campaign only;
  `combined_opponent` sums the two modes; `overall_streak` advances/resets across
  an interleaved win/loss sequence; `ModeRecord::totals` sums; `win_rate` is
  `None` at zero matches and genuinely rounds otherwise (3 of 4 → 75; 2 of 3 → 67,
  which truncation would wrongly give as 66 — sign-off note 3).*

- [x] **T002 (foundational, review: per-task)** — Wire the persisted model into
  `src/profile.rs` and `src/campaign.rs`. In `profile.rs`: add
  `#[serde(default)] stats: LifetimeStats` to `Profile` (and to the two test
  constructors `profile_with` / the `tests` `Profile { .. }`), init in `Default`,
  **no `PROFILE_VERSION` bump**; add `stats() -> &LifetimeStats`,
  `record_match(mode, opponent_id, player_won, player_rounds: u32, opp_rounds:
  u32)` (calls `self.stats.record_match(...)`; if `Campaign`, also
  `self.campaign.run_stats_mut().record_match(...)` and, when
  `player_won && self.campaign.run_complete()`,
  `self.stats.record_campaign_completion()`), and `distinct_side_cards_owned() ->
  usize` (`self.collection_by_type().len()`); **amend `reset_to_starter`** to
  `let stats = std::mem::take(&mut self.stats); *self = Profile::default();
  self.stats = stats;`. In `campaign.rs`: add `#[serde(default)] run_stats:
  RunStats` to `CampaignRun` (import `crate::stats::RunStats`) with `run_stats()`
  and `run_stats_mut()`. Plan §Design 2–3 / tensions §1, §3, §4. (Copies the
  additive-serde-field pattern of `Profile.campaign` / `Profile.credits`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests:
  `LifetimeStats`/`RunStats` round-trip through JSON; a pre-020 profile
  (`{"version":1,"collection":[],"deck":[]}`, no `stats`/`run_stats` keys) loads
  with defaults and `PROFILE_VERSION` is unchanged (mirrors
  `credits_persist_and_default_to_zero_for_older_profiles`); `record_match` on a
  campaign win that clears the final node increments `campaign_completions`, a
  non-final win does not, and a post-`reset_to_starter` re-clear increments again;
  the **amended** `reset_to_starter_...` test dirties `stats` too and asserts
  `stats` is preserved while credits/collection/deck/campaign/run tally are
  starter (the old "identical to a fresh profile" assertion is replaced by this
  field-wise contract — a deliberate behavior change, not a weakening).*

## Phase 2 — Recording seam (App::tick)

<!-- Integration: record every completed match exactly once at the GameOver
transition. Single task; its per-task review is the phase review. Phase ends with
the person's persist-across-restart attestation (T003). -->

- [x] **T003 (review: per-task)** — Add the once-per-match recording block to
  `App::tick` in `src/app.rs`, **placed after the existing campaign-win block**
  (`app.rs:1218`, so `mark_beaten` has run and `run_complete()` is accurate) and
  before `emit_audio_cues`. Guard on `phase_changed && matches!(game_phase,
  GameOver { .. })` (fires exactly the entering tick — plan tension §2); read
  `player_won` (`matches!(.., GameOver { winner: Player::Player })`),
  `opponent_id = game_state.opponent_profile.id`, `player_rounds =
  game_state.player.rounds_won as u32`, `opp_rounds =
  game_state.opponent.rounds_won as u32`, and `mode = if
  self.profile.campaign().in_progress().is_some() { Mode::Campaign } else {
  Mode::QuickPlay }`; then `self.profile.record_match(mode, opponent_id,
  player_won, player_rounds, opp_rounds); self.profile.save();`. Mirror the
  existing campaign-win block's field-disjoint borrow structure (`&self.screen`
  reads into locals, then `&mut self.profile`). Note `opponent_profile.id` is
  `&'static str` (`opponent.rs:41`), so `opponent_id` is a plain `Copy` — no
  borrow of `&self.screen` is held across the `&mut self.profile` call
  (sign-off note 6). No new `App` field, no reset hook.
  Plan §Design 4. (Copies the existing campaign-win `tick` block's guard/borrow
  shape.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green (the recording
  logic is covered by T001/T002 unit tests; the seam is driver-verified). Report
  that the block is placed after the campaign-win block and that no
  `game.rs`/`player.rs`/`card.rs`/`save.rs` file was touched. Driver (back up +
  checksum-restore the real profile/saves first): finish a Quick Play match and a
  Campaign match (win and loss), quit, relaunch, and confirm via a temporary
  inspection of `profile.json` (or the Phase 3 screen once it lands) that each
  match's match W/L and round W/L were recorded once — a 3–1 win shows 3 round-wins
  / 1 round-loss — in the right mode; an abandoned (quit mid-match) match records
  nothing; clearing the final campaign node increments campaign completions, and
  New Campaign leaves lifetime records + completions while emptying This Run.
  **PAUSE for the person** (phase attestation): confirm records persist across a
  restart and abandoned matches record nothing.*

## Phase 3 — Records screen (Screen + menu wiring + draw)

<!-- The read-only Records Screen and its wiring. Depends on the data model
(Phase 1); recording (Phase 2) must be in for the attestation to show live data.
Phase ends with the person's read-the-screen attestation (T005). -->

- [x] **T004** — Create `src/records.rs` (add to `src/lib.rs`) — state, input,
  and the pure content builders (no draw yet). Define `RecordsView { Overall,
  QuickPlay, Campaign, ThisRun }` (a `VIEWS` array in display order),
  `RecordsOutcome { Moved, Back }`, `RecordsState { view: usize, scroll: usize }`
  with `new()`; `handle_input(&mut self, key) -> Option<RecordsOutcome>` (Left →
  previous view wrapping + `scroll = 0`; Right → next view wrapping + `scroll = 0`
  — emacs `Ctrl+B/F` arrive as Left/Right; Up → `scroll.saturating_sub(1)`; Down →
  `scroll + 1`; PageUp/PageDown → ± a page constant; all `Some(Moved)`; Esc/`x` →
  `Some(Back)`; else `None`). Add the pure builders:
  `collection_line(owned_types, total_types) -> String` (`"Cards: N of 15 — P%"`);
  `view_title(view) -> &'static str` + a pager label; and `view_body(view,
  stats: &LifetimeStats, run: &RunStats) -> Vec<String>` (summary block —
  matches/won/lost/win-rate; the Overall view adds the overall-streak line, the
  Campaign view adds a completions line (Quick Play adds neither — no streak, no
  completions; sign-off note 2), This Run adds its run-streak line; then a
  `By opponent:` header and one row per `opponent::OPPONENTS` by name with match
  `W–L` + round `W–L`, combined for Overall and the single `ModeRecord` for the
  mode views, a never-faced opponent as `0–0`; ThisRun shows a "No matches this
  run yet." line at zero matches, else the run summary with no breakdown). Plan
  §Design 5 / tension §5, §6. (Copies `opponent_select.rs`'s state +
  `handle_input` shape.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests:
  `handle_input` pages Left/Right through the four views wrapping and resets
  scroll, changes scroll on Up/Down/PgUp/PgDn, backs out on Esc/`x`, ignores
  unknown keys; `view_body` includes every `OPPONENTS` name in each breakdown
  view, the Overall breakdown equals Quick Play + Campaign per opponent, the
  summary carries matches/won/lost/win-rate substrings, the Campaign view shows a
  completions line and the Overall/QuickPlay views do not, the streak line appears
  on the Overall and This Run views but not on the Quick Play or Campaign views
  (sign-off note 2), ThisRun shows the no-matches line at zero and the run summary
  otherwise, and a never-faced opponent reads `0–0`; `collection_line` renders
  "N of 15" and the percentage.*

- [ ] **T005** — Draw the Records screen and wire it in + attest. In
  `src/records.rs` add `draw(&mut self, frame, config: &Config, profile:
  &Profile, pulse)`: a full-screen bordered `Rect` (`clear_rect` + `draw_box`),
  the title/pager row (selected view name may breathe with `pulse`), the
  persistent `collection_line(profile.distinct_side_cards_owned(),
  card::ALL_SIDE_CARDS.len())`, a rule, the body viewport from `view_body(view,
  profile.stats(), profile.campaign().run_stats())` using the spec-019 scroll
  clamp (`vh` = body rows, `max_off = body.len().saturating_sub(vh)`, clamp+store
  `self.scroll`), and a footer hint (`◂/▸ view · ↑/↓ scroll · Esc back`, ▲/▼ on
  overflow). Wire the screen: `screen.rs` `Screen::Records { state: RecordsState }`;
  `menu.rs` `MenuItem::Records` (+ `Display` `"Records"`, list slot after
  `SideDeck`, updated menu tests: 5→6 / 6→7 and order); `app.rs`
  `activate_menu_item` `Records => self.open_records()`, `fn open_records`, the
  `Screen::Records { state }` `handle_key` arm (`Moved` → `Sfx::MenuMove`; `Back`
  → `Sfx::MenuBack` + `self.screen = self.start_menu()`), the `?`-help
  no-overlay arm, and the draw dispatch (`Screen::Records { .. } => {}` in the
  immutable match, then `if let Screen::Records { state } = &mut self.screen {
  state.draw(...) }` after it, mirroring the play-log draw block). Plan §Design
  5–6 / tension §6. (Copies `opponent_select`'s `draw` + the menu/screen wiring
  of the other screens + the spec-019 scroll math in `overlay.rs`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — the updated menu
  tests pass. Driver (back up + checksum-restore the real profile/saves first): a
  **Records** item on the start menu opens the full-screen Records screen; Esc
  returns to the menu with its selection preserved; all four views page with
  Left/Right (and `Ctrl+B/F`); each view shows matches/won/lost/win-rate, the
  Campaign view shows completions, This Run reflects the active run (and its
  no-matches empty state before the first run match); the persistent collection
  line matches the collection; the per-opponent breakdown lists the whole roster
  (faced or not); long views scroll with ↑/↓ · PgUp/PgDn; the screen renders
  cleanly on a brand-new profile (zeroes/dashes) and is correct and monochrome at
  139×31 and wider. Snapshot at 139×31 and wider. **PAUSE for the person** (phase
  attestation): confirm the screen reads cleanly, the numbers are correct after a
  match, and it is monochrome.*

## Final phase — Spec close-out

- [ ] **T006** — Docs, driver, sweep. `DECISIONS.md`: stats are additive
  serde-defaulted fields on `profile.json` (no `PROFILE_VERSION` bump);
  match-end-only recording deriving round W/L from `rounds_won` (the deviation
  from the spec's assumed round-resolution seam, and why — abandoned matches
  record nothing, resume/rematch need no special handling); derived-vs-persisted
  split (only per-opponent + streaks + completions persist; totals/win-rate/
  combined/collection derived); `reset_to_starter` preserves the lifetime `stats`
  field (the one behavior change); no per-mode streaks (Erik-ruled — streaks are
  overall-lifetime + current-run only, and the two mode views omit the streak
  line); campaign-completion detection via `run_complete()` ordered after
  `mark_beaten`. `ROADMAP.md`: mark stats &
  records shipped; drop from future. Check off `spec.md` acceptance criteria with
  evidence. Request the pre-merge whole-spec sweep.
  *Verify: `cargo build --all-targets` / `cargo test -q` green, reported
  verbatim; legible snapshots at 139×31 and wider of the Records screen (all four
  views + the empty first-run state), profile/saves checksum-restored; the sweep
  confirms no `game.rs`/`player.rs`/`card.rs`/`save.rs` change and that
  `PROFILE_VERSION` is unchanged; `ROADMAP.md` no longer lists this as future;
  sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/020-stats-and-records/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product owner**.
Dispatch each routine task to the `sdd-implementer` per the model policy; verify
by running the build and tests yourself, then commit. **The two tasks marked
`review: per-task` (T002, T003):** run the `skeptical-reviewer` after each,
scoped to that task's diff, its plan section, and its acceptance criteria (a
shell-assembled bundle), one review plus at most one re-review, and re-run the
verification command yourself before committing. **Every other task (T001, T004,
T005, T006):** review at phase end. Pause for the person after each phase, and
whenever something unexpected bears on spec adherence. The **T003 phase pause**
is the persistence attestation (records survive a restart; abandoned matches
record nothing); the **T005 phase pause** is the read-the-screen attestation (all
four views, empty first-run state, correct numbers, monochrome at 139×31).

Model & effort: the session runs at the orchestrator tier (`claude-opus-4-8`),
medium effort; the planner and the `skeptical-reviewer` sign-off/reviews run at
the top tier (`fable`) per the per-call override — **except while the top-tier
budget is short**, when they drop to `opus` per the documented fallback (this
plan/tasks pair was drafted under that fallback; per Erik's standing ruling a
successful `fable` dispatch does not by itself mean the budget recovered — keep
the `opus` fallback until Erik says otherwise). Implementers run at `opus`.
Clear at every phase boundary and at spec end. Every session-ending pause ends
with a continuation prompt (spec directory, files to read, where to resume,
involvement level, pause cadence, any model switch) in its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo, and why). Compare the spec total against a previous spec of similar size
before treating the policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| Planning: draft (sdd-planner) | opus (documented fallback — top-tier budget short) | ~168K (measured return) | drafted |
| plan + tasks sign-off (skeptical-reviewer) | opus (documented fallback) | ~56K (measured return) | signed off; 6 non-blocking notes (notes 1–3,6 folded into plan/tasks; 4,5 accepted no-change) |
| T001 impl (sdd-implementer) | opus (fable budget short — documented fallback) | ~24K (measured return) | done; 325 tests green, 6 new |
| T002 impl (sdd-implementer) | opus (fable budget short — documented fallback) | ~52K (measured return) | done; 327 tests green; +Debug on stats types (required mechanical fix) |
| T002 review (skeptical-reviewer, per-task) | opus (fable budget short — documented fallback) | ~36K (measured return) | signed off, no blocking; note: once-only completion leans on app launch guard → verify in T003 seam review |
| Phase 1 review (skeptical-reviewer) | opus (fable budget short — documented fallback) | ~41K (measured return) | signed off, no blocking; note: ensure T004/T005 unit-test the "N of 15" collection-completion derivation (already in T004 verify list) |
| T003 impl (sdd-implementer) | opus (fable budget short — documented fallback) | ~20K (measured return) | done; app.rs only, 327 tests green; block placed after campaign-win block |
| T003 review (skeptical-reviewer, per-task) | opus (fable budget short — documented fallback) | ~47K (measured return) | signed off, no blocking; double-count concern CLOSED (GameOver only entered via tick update; completed run has no launchable match). Two non-blocking notes → sweep: (1) add a comment stating the "GameOver only via update()" invariant the narrower phase_changed guard relies on (± plan §2 note); (2) optional cross-module guard test "run complete ⇒ no launchable match" |
| T004 impl (sdd-implementer) | opus (fable budget short — documented fallback) | ~33K (measured return) | done; records.rs state/input/builders, 340 tests green (13 new); reviewed at phase end |
| T005 impl (sdd-implementer) | | | |
| Phase 3 review (skeptical-reviewer) | | | |
| T006 close-out (orchestrator) | claude-opus-4-8 | — | |
| Pre-merge whole-spec sweep (skeptical-reviewer) | | | |
