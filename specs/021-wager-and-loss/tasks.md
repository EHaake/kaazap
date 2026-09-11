# Tasks: Wager & loss condition — spec 021

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-10) — ready for implementation
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

**Foundational phase:** Phase 1 (T001–T002 — the tunable constants, the
`stake` field on the persisted in-flight pointer, and the pure escrow /
settlement / broke / reserve / once-only-completion logic every screen calls)
is foundational. Under the constitution's review cadence the default is a
**per-phase** `skeptical-reviewer` pass. Two tasks carry `review: per-task`:
**T002**, the `Profile`/`CampaignRun` staking contract — a mistake in the
serde shape, the seed-purse default, the exactly-once settlement, or the
completion edge corrupts real `profile.json` balances or double-pays every
user, and every later file inherits the API; and **T004**, the app flow that
escrows at launch and settles at the once-per-match seam — the direct analogue
of spec 020's T003, where a wrong guard silently pays twice or never. Every
other task — T001, T003, T005, T006, and the close-out T007 — is reviewed at
phase end.

---

## Phase 1 — Staking core: constants + profile/campaign logic (foundational)

<!-- Foundational: the constants, the persisted stake, and the pure operations
(escrow, settle, broke, reserve, rematch rule, once-only completion) that the
prompt (Phase 2), the loss flow (Phase 3), and the docs all rest on. T002 gets a
per-task review. -->

- [x] **T001 (foundational)** — `src/economy.rs` + `src/campaign.rs`: the
  constants and pure rules. In `economy.rs` add `SEED_PURSE = 50`,
  `ANTE_BASE_THRESHOLD = 14`, `ANTE_PER_THRESHOLD_STEP = 10`, `STAKE_STEP = 5`,
  `PAYOUT_RATIO = 1` (each with a one-line doc naming what it tunes);
  `ante_floor(threshold: usize) -> u32`, `ante_floor_for(opponent_id) -> u32`
  (unknown id → `crate::STAND_THRESHOLD`), `win_payout(stake) -> u32`
  (`stake × (1 + PAYOUT_RATIO)`, saturating), `cheapest_floor(run) -> u32` (min
  `ante_floor_for` over unlocked planets' `launchable_opponent`, 0 if none),
  and `StakeOutcome { Won(u32), Lost(u32) }`. **Leave** `WinReward` /
  `win_reward` in place for now — T004 removes them together with the tick
  block that calls them, so every task compiles on its own. In `campaign.rs` add
  `#[serde(default)] pub stake: u32` to `NodeRef`, and on `CampaignRun`:
  `launchable_opponent(&self, planet) -> Option<&'static str>`
  (`next_opponent` or the cleared planet's last opponent), `stake_at_risk(&self)
  -> Option<u32>` (in-flight stake when > 0), `take_stake(&mut self) -> u32`
  (return + zero). Plan §Design 1–2 / tensions §1, §7. (Copies the
  constants-as-data + pure-fn shape of `economy.rs`'s `card_price`/`card_tier`
  and `CampaignRun::next_opponent`.) `#[serde(default)]` covers JSON only —
  a Rust struct literal without the field is a compile error — so this task
  also adds `stake: 0` to **every existing `NodeRef` literal** (the `app.rs`
  launch site if it is a literal, the `profile.rs` reset test, and any other
  test literal `grep -n "NodeRef {"` finds), so the task builds and tests
  green on its own. Nothing else outside `economy.rs`/`campaign.rs` changes.
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  (reported verbatim) — new tests: `ante_floor` 15..=19 → 10/20/30/40/50 and
  `ante_floor_for` of an unknown id → 30; `win_payout(20) == 40`,
  `win_payout(0) == 0`; `cheapest_floor` equals an independently computed min
  over unlocked planets × `launchable_opponent` for a fresh, a half-cleared,
  and a complete run, and is 10 in all three; `launchable_opponent` — uncleared
  → next un-beaten, The Anvil with only Brakka beaten → Kesh, The Anvil cleared
  → Kesh, cleared Cinder → Greeb; a `NodeRef` JSON without `stake` loads as 0
  and one with `stake: 20` round-trips.*

- [x] **T002 (foundational, review: per-task)** — `src/profile.rs`: the
  staking contract. `Default` credits → `economy::SEED_PURSE` (the serde field
  default stays 0 — an existing file keeps its balance; **no `PROFILE_VERSION`
  bump**). Add `stake_match(node: NodeRef) -> bool` (escrow: refuse if
  `stake > credits`, else deduct + `set_in_progress(Some(node))`),
  `settle_campaign_match(player_won) -> Option<StakeOutcome>` (exactly plan
  §Design 3: `take_stake`, pay `win_payout` on a win, `mark_beaten`, count a
  completion only on the `!was_complete && run_complete()` edge; `None` when no
  campaign pointer), `is_broke()`, `can_afford(price)` (`credits >= price +
  cheapest_floor`) with `try_purchase` using it. **Remove** the
  `player_won && run_complete()` completion clause from `record_match`
  (tension §2). Make `applying_a_win_reward_pays_credits_and_drops_one_pool_card`
  seed-relative (`SEED_PURSE + 10`) — `apply_win_reward` itself stays until
  T004 deletes it with its caller. Plan §Design 3 / tensions §2, §3, §5, §6. (Copies the additive-serde-field +
  tested-profile-op pattern of `try_purchase` / `record_match`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests:
  `Profile::default().credits() == SEED_PURSE`; older JSON without `credits` →
  0 and with `credits: 75` → 75 (amend `credits_persist_and_default_to_zero…`);
  `reset_to_starter` → `SEED_PURSE` (amend the reset test) and `PROFILE_VERSION
  == 1`; `stake_match` 50/20 → 30 credits + pointer with stake 20 +
  `stake_at_risk() == Some(20)`, 50/60 → false and unchanged; settle from
  30/stake 20: win → 70, `Won(20)`, beaten, stake 0, `stake_at_risk()` None;
  loss → 30, `Lost(20)`, not beaten; a second settle → `Won(0)`/`Lost(0)`, no
  change; no pointer → `None`, credits untouched; a zero-stake pointer settles
  (win marks beaten, pays 0); a rematch win/loss on a complete run changes
  neither `beaten` nor completions but pays; the rewritten
  `campaign_completion_counts_only_a_final_clearing_win_and_recounts_after_reset`
  drives clears through `stake_match` + `settle_campaign_match` (0 → 1 → reset
  → 2) and asserts `record_match` alone never increments; `is_broke` — fresh
  run 9 → true, 10 → false, complete run 10 → false, and a match in flight
  (staked pointer set) does not change the answer; the rewritten
  affordability test — 60 credits / price 50 buys leaving 10, 59 refuses and
  `can_afford` is false, the reserve is never deducted; a staked pointer
  round-trips through a full `Profile` JSON.*

## Phase 2 — The staked launch and resolution loop

<!-- The wager prompt module, then the app flow that gates a launch on the
floor, escrows on commit, settles at the once-per-match seam, and banners the
result — plus rematches on the map. Phase ends with the person's staked-match
attestation (T004). -->

- [x] **T003** — Create `src/wager.rs` (add to `src/lib.rs`): `WagerOutcome {
  Moved, Commit, Cancel }`, `WagerState { planet: Planet, opponent:
  OpponentProfile, floor, max, k }` with `new(planet, opponent, balance)`
  (`floor = ante_floor(opponent.stand_threshold)`, `max = balance`, `k = 0`),
  `stake()` = `min(floor + k·STAKE_STEP, max)`, `k_max = ceil((max − floor) /
  STAKE_STEP)`, `planet_id()`, `opponent()`, `handle_input(key)` (Left/`a` →
  `k − 1` or `None` at 0; Right/`d` → `k + 1` or `None` at `k_max`; Enter/Space
  → `Commit`; Esc/`x` → `Cancel`; else `None`), `lines() -> Vec<String>` (title
  with opponent + planet names; `Ante ◈ {floor}   Balance ◈ {max}`; `◂  Stake ◈
  {stake}  ▸`; `Win +{payout − stake}   ·   Lose −{stake}`; the key hint), and
  `draw(frame, config, pulse)` via `OverlayLayout` + `clear_rect` + `draw_box` +
  centered `draw_text_in`, the stake row on `pulse`, all else Normal/Muted.
  Plan §Design 5 / tension §7. (Copies `records.rs`'s modal state +
  `handle_input` + `draw` shape and `draw_two_choice`'s box construction.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: opens
  at the floor; floor 10 / balance 53 walks Right 10,15,…,50,53 then `None`,
  Left 53 → 50 … → 10 then `None`; balance == floor → Left/Right `None`; Enter
  and Space → `Commit` at the current stake; Esc/`x` → `Cancel`; unknown →
  `None`; `lines()` contains the opponent name, `Ante ◈ 10`, `Balance ◈ 53`,
  `+15`/`−15` at stake 15, and every line is ≤ 70 columns.*

- [x] **T004 (review: per-task)** — Wire the loop in `src/app.rs` and
  `src/campaign_map.rs`. `campaign_map.rs`: `MapBanner { Settled(StakeOutcome),
  CantCover { floor } }` replaces the reward banner; `draw(…, banner:
  Option<&MapBanner>, …)`; header text `★  Won {s} credits` (Strong) / `Lost {s}
  credits` (Normal) / `Can't cover the {floor}-credit ante` (Normal); Enter uses
  `launchable_opponent` (a cleared planet launches its final opponent); the
  cleared / complete status lines mention the rematch; the rail preview uses
  `launchable_opponent`. `app.rs`: `Modal::Wager(WagerState)`; `last_reward` →
  `banner: Option<MapBanner>`; `launch_campaign_node` becomes the gate (deck
  divert → `credits < floor` ⇒ `CantCover` banner + `MenuBack` ⇒ else open the
  wager modal); **delete** `WinReward`, `win_reward`, and
  `win_reward_scales_credits_and_picks_the_card_by_roll` from `economy.rs`
  (update its module doc — no roll seam remains) and `apply_win_reward` + its
  test from `profile.rs`; `handle_wager_input` (`Moved` → `MenuMove`, `Cancel` → close +
  `MenuBack`, `Commit` → close + `MenuSelect` + `start_match(opp, Some(NodeRef
  { planet, opponent, stake }))`) dispatched before the `CampaignEntry` branch;
  `start_match` escrows via `stake_match` for a campaign node (return without
  launching if it refuses) and `set_in_progress(None)` for Quick Play; **delete
  the every-tick campaign-win block** and extend the `phase_changed` GameOver
  block to `settle_campaign_match` → `banner = Settled(outcome)` →
  `record_match` → `save` (plan §Design 4 code); `draw` arm for `Modal::Wager`;
  `Screen::InGame` draw passes `self.profile.campaign().stake_at_risk()` — add
  the `stake: Option<u32>` parameter to `BoardView::draw` now and forward it to
  `draw_presence_extras` as an ignored `_stake` (T006 draws it). Plan §Design
  4, 6 / tensions §3, §4. (Copies the `Modal::Records` dispatch/draw wiring and
  the existing tick block's field-disjoint borrow shape.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests: Enter
  on cleared Cinder → `Launch { cinder, greeb }` (replaces
  `enter_on_a_cleared_planet_is_a_no_op`); the existing map/shop/menu tests
  pass. Report that the every-tick win block is gone, that the GameOver block
  calls `settle_campaign_match` before `record_match`, and that no
  `game.rs`/`player.rs`/`card.rs`/`save.rs` file was touched. Driver (back up
  + checksum-restore the real profile/saves first): with `◈ 15` and a floor-20 node unlocked, Enter on it refuses with the can't-cover message and opens no prompt (AC2); Enter on Cinder opens the
  prompt at `◈ 10` showing Greeb, ante, balance, and the payout; ←/→ step by 5
  and clamp at the balance; Esc returns with `◈` unchanged; Enter drops the
  header `◈` by the stake and starts the match; a win banners `Won N credits`
  with `◈` up by 2N and the node marked; a loss banners `Lost N credits` with
  `◈` unchanged from the escrowed value; Enter on the cleared Cinder stakes a
  rematch vs Greeb whose win/loss changes no progress and (via Records) no
  completions; a Quick Play match shows no prompt and pays nothing; mid-match
  Esc → Continue resumes with the stake still escrowed and settles normally.
  **PAUSE for the person** (phase attestation): a staked win and loss, a
  rematch, and a resumed staked match behave as the spec says.*

## Phase 3 — Loss condition, escape hatches, and the visible stake

<!-- The run-over modal at the two broke seams, the discard confirms that name
the stake at risk, the reserve-aware shop, and the in-match stake row. Phase
ends with the person's broke-and-reset attestation (T005/T006). -->

- [ ] **T005** — `src/app.rs`: the loss condition. `Modal::RunOver`; pure `fn
  run_over_acknowledged(key) -> bool` (Enter/Space only — Esc does not
  dismiss); `handle_run_over_input` (acknowledge → `modal = None` +
  `start_new_campaign()`); `fn enter_campaign_map()` (`open_campaign_map` then
  `Modal::RunOver` if `is_broke()`), used by the game-over acknowledgement,
  `enter_campaign_continue`'s no-save branch, and the `PendingStart::Campaign`
  Yes arm — which also forfeits explicitly (`set_in_progress(None)` +
  `profile.save()` beside `save::clear()`). **Ordering:** in that Yes arm (and
  any other modal-hosted caller) close the confirm (`modal = None`) *before*
  calling `enter_campaign_map()`, so a raised `Modal::RunOver` is never
  overwritten by the confirm's own close. **Stale pointer:** in
  `enter_campaign_continue`'s no-save branch, a lingering `in_progress` with a
  nonzero stake (a kill mid-match before any save) is cleared as a forfeit
  (`set_in_progress(None)` + save) before the broke check, so the confirm notes
  never name a stake with no match behind it. `draw_run_over` (a bordered box:
  title `You're broke — the run is over.` Strong, note `Deck, collection, and
  progress reset to the starter; your records stay.`, hint `Enter  continue`
  Muted); `draw_two_choice` gains `note: Option<&str>` on row 1 (Muted) and the
  two discard confirms pass `stake_at_risk().map(|s| format!("…and forfeit your
  {s}-credit stake."))` (`draw_campaign_entry` passes `None`). Plan §Design 4 /
  tensions §5, §9. (Copies `handle_confirm_new_campaign_input` +
  `draw_confirm_new_campaign` and `start_new_campaign`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — test:
  `run_over_acknowledged` is true for Enter/Space and false for Esc/`x`/others;
  `confirm_choice_commits_only_on_enter_with_yes` still passes. Driver (back up
  + checksum-restore first): with a small balance, stake all-in and lose →
  after Enter on the outcome the map opens with the run-over notice; Esc does
  nothing; Enter resets to a fresh first-world map at `◈ 50`, starter deck and
  collection, Records lifetime rows intact and This Run empty; a profile
  hand-edited to 0 credits with no save meets the notice on Continue; a staked
  mid-match save shows `…and forfeit your N-credit stake.` on both the
  Start-Campaign discard confirm and the New Campaign confirm (and not for a
  Quick Play save); confirming the campaign discard drops the escrow and
  raises the notice when that leaves the balance under the floor.*

- [ ] **T006** — `src/shop.rs`, `src/portrait.rs`, `src/board.rs`: the
  reserve-aware shop and the stake row. Shop: `affordable =
  profile.can_afford(price)`; balance row `Credits: ◈ {credits}  ·  spendable
  ◈ {credits − cheapest_floor}`. Portrait: `PANEL_H_INMATCH` → `2 + 1 +
  PORTRAIT_HEIGHT + 1 + 2 + 2` (20); `draw_presence_extras(…, stake:
  Option<u32>)` draws `Stake ◈ {n}` centered, Strong, on interior row 17 when
  `Some`. Board: forward the `stake` parameter T004 added. Plan §Design 7–8 /
  tensions §6, §8. (Copies `draw_presence_extras`'s banter-row drawing and the
  shop's `affordable` dimming.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green — tests:
  `draw_presence_extras(.., Some(40))` puts `◈ 40` on frame row 18 and `None`
  leaves rows 17–18 blank; the banter (row 15) / pip (row 16) tests and
  `the_full_pool_fits_the_minimum_terminal` still pass; the board layout test's
  `opponent_panel.y1 <= 30` still holds at 139×31. Driver (back up +
  checksum-restore first): at 139×31 a staked match shows `Stake ◈ N` under the
  pips inside the panel border, Quick Play shows nothing there; in the shop with
  `◈ 25` on a fresh run a 20-credit card is dimmed and Enter refuses it, with
  `◈ 30` it buys leaving `◈ 10`; the spendable readout matches. **PAUSE for the
  person** (phase attestation): going broke resets the run and preserves
  records; the shop can't strand you; the stake is visible in-match at 139×31.*

## Final phase — Spec close-out

- [ ] **T007** — Docs, driver, sweep. Rewrite `docs/economy.md` around the
  two-directional loop: the constants table (seed purse 50, ante floor formula +
  the 15–19 → 10–50 table, stake step 5, payout 1:1, no cap) and where they
  live; escrow at launch and settlement at the seam; rematches (final
  opponent, no progress, completion counted once by the edge); broke → run-over
  → `reset_to_starter`; the shop reserve; persistence (`stake` on `NodeRef`,
  seed purse for new/reset profiles only, no version bump); the guard tests.
  `README.md`: drop "card packs" / "drop cards" from the blurb (lines 13–14,
  24–25) and mention stakes, rematches, and going broke (line 74–75 area).
  `ROADMAP.md`: Wager & loss shipped; the balance pass unblocked.
  (`ROADMAP.md` and `DECISIONS.md` are repo-wide files per `CLAUDE.md`'s git
  conventions: draft their text in this task, but apply and commit it on
  `main` after the merge, as spec 020's close-out did — never edit those two
  files on the spec branch.)
  `DECISIONS.md`: the spec's resolved decisions plus plan tensions §1–§5 (stake
  on `NodeRef`; settlement + completion edge in one profile op, superseding
  spec 020's `record_match` clause; `take_stake` exactly-once; one edge-based
  resolution block; broke checked at two seams). Check off `spec.md`
  acceptance criteria with evidence. Request the pre-merge whole-spec sweep.
  *Verify: `cargo build --all-targets` / `cargo test -q` green, reported
  verbatim; legible snapshots at 139×31 and wider of the wager prompt, the
  won/lost/can't-cover banners, the run-over notice, a staked board, and the
  shop with a reserve-dimmed card, profile/saves checksum-restored; the sweep
  confirms no `game.rs`/`player.rs`/`card.rs`/`save.rs` change,
  `PROFILE_VERSION == 1` and `SAVE_VERSION == 1`, no remaining `WinReward` /
  `win_reward` / `apply_win_reward` references, and every constant in the
  spec's table present in `economy.rs` with its default; `ROADMAP.md` no longer
  lists this as future; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/021-wager-and-loss/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product
owner**. Dispatch each routine task to the `sdd-implementer` per the model
policy; verify by running the build and tests yourself, then commit. **The two
tasks marked `review: per-task` (T002, T004):** run the `skeptical-reviewer`
after each, scoped to that task's diff, its plan section, and its acceptance
criteria (a shell-assembled bundle), one review plus at most one re-review, and
re-run the verification command yourself before committing. **Every other task
(T001, T003, T005, T006, T007):** review at phase end. Pause for the person
after each phase, and whenever something unexpected bears on spec adherence.
The **T004 phase pause** is the staked-loop attestation (win, loss, rematch,
resumed staked match); the **T006 phase pause** is the loss-condition
attestation (broke → reset with records intact, the shop reserve, the in-match
stake line at 139×31). Back up + checksum-restore the real profile/saves
before every driver session — this spec resets real profiles by design.

Model & effort: the session runs at the orchestrator tier (`claude-opus-4-8`),
medium effort; the planner and the `skeptical-reviewer` sign-off run at the top
tier (`fable`) per the per-call override — Erik confirmed on 2026-09-10 that
the fable budget is back, and this plan/tasks pair and its sign-off ran there.
Decision reviews (a non-routine task) also go to `fable`; per Erik's standing
ruling, never infer the budget from a successful dispatch — ask if unsure, and
log which tier actually ran. Per-phase and per-task reviews and the sweep run
at `opus`; implementers run at `opus`.
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
| Planning: draft (sdd-planner) | fable | ~210K (measured return) | drafted; no product questions |
| plan + tasks sign-off (skeptical-reviewer) | fable | ~69K (measured return) | 1 blocking (T001 NodeRef literals) + 11 notes; B1 and notes 1,2,3,4,6,11 fixed; 5,7,8,9,10 accepted no-change |
| sign-off re-review (skeptical-reviewer) | fable | ~21K (measured return) | signed off; T007 wording tightened; open for sweep: T001 no longer strictly "additive only" (cosmetic) |
| T001 impl (sdd-implementer) | opus | ~46K (measured return) | done; module-doc rewrite deferred to T004 (win_reward still present); bundle wrongly said economy.rs already imported opponent_by_id |
| T002 impl (sdd-implementer) | opus | ~65K (measured return) | done; two amended tests renamed to match new contracts; NodeRef literal in reset test → node() helper |
| T002 review (skeptical-reviewer, per-task) | opus | ~46K (measured return) | signed off, no blocking; open for sweep: rematch record_match coverage (AC5 second half) unpinned in T002; pre-021 in_progress-without-stake not pinned end-to-end via Profile::from_json; stake_match doc claims the prompt gates on balance (verify at T003/T004); abandoned escrow on discard is the plan §5/T005 explicit forfeit |
| Phase 1 review (skeptical-reviewer) | opus | ~57K (measured return) | signed off, no blocking; handoffs: T004 must wire settle_campaign_match + launchable_opponent (completions uncounted at Phase 1 HEAD by design); banner "Won N" should show win_payout(stake) − stake, not the raw Won payload, so it stays right if PAYOUT_RATIO changes; cheapest_floor is 10 in every run state (Cinder rematch) so the unlocked-planet filter is inert — docs must not claim it was exercised |
| T003 impl (sdd-implementer) | opus | ~52K (measured return) | done; cosmetic arrow-dimming skipped (one stake string shared by lines()/draw); k_max private |
| T004 impl (sdd-implementer) | opus | ~89K (measured return) | done; banner_line extracted as a pure helper; refused can't-cover launch sounds MenuSelect then MenuBack (pre-existing divert shape) |
| T004 review (skeptical-reviewer, per-task) | opus | ~70K (measured return) | signed off, no blocking; open for sweep: stale ordering comment at the GameOver block (attributes completion to record_match); banner_line has no test pinning the net-gain rule; net-gain expression duplicated in wager.rs and campaign_map.rs; start_match doc names the old campaign caller |
| Phase 2 review (skeptical-reviewer) | opus | ~58K (measured return) | signed off, no blocking; open for sweep: wager width test never walks to k_max (comment overstates); new() precondition prose-only (single caller upholds it); draw picks emphasis by row index; AC2 rests on the driver/attestation only |
| T005 impl (sdd-implementer) | | | |
| T006 impl (sdd-implementer) | | | |
| Phase 3 review (skeptical-reviewer) | | | |
| T007 close-out (orchestrator) | claude-opus-4-8 | — | |
| Pre-merge whole-spec sweep (skeptical-reviewer) | | | |
