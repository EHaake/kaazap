# Tasks: Difficulty & economy balance pass — spec 022

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-13) — ready for implementation
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
implementer returns, and only the orchestrator commits. **Per the person's ruling
of 2026-09-13, every task below — including every simulator run and every tuning
iteration — runs in the `sdd-implementer` at the implementation tier; no task
sends simulator output to the top tier.**

**Foundational phase:** Phase 1 (T001–T002 — the starter constants, the Quick
Play deal, and the simulator that every later number comes from) is
foundational. Under the constitution's review cadence the default is a
**per-phase** `skeptical-reviewer` pass. One task carries `review: per-task`:
**T002**, the simulator — a wrong win attribution, a scripted-player rule that
misreads the board, or a bound formula off by the reserve would silently
invalidate every measurement, every tuning decision, and the doc's whole
table. Every other task — T001, T003–T008 — is reviewed at phase end.

---

## Phase 1 — Foundations: the starter, the Quick Play deal, the simulator (foundational)

<!-- Foundational: the two constants the tuning edits and the instrument that
measures them. T002 gets a per-task review. Phase ends with a short pause: the
person can see the new starter in the builder/shop and play Quick Play with the
premium deck (the starter composition may still move in Phase 2). -->

- [x] **T001 (foundational)** — `src/profile.rs`, `src/app.rs`, `src/card.rs`
  (comments only), `src/deck_builder.rs` (tests only): the Outer-tier starter
  and the Quick Play deal. In `profile.rs` add `pub const STARTER_SIDE_DECK:
  [Card; SIDE_DECK_SIZE]` = `+1 +1 +2 +2 +3 −1 −1 −2 −2 −3` and `pub const
  STARTER_SPARES: [Card; 3]` = `+3 −3 ±1` (plan §Design 1 — first guesses,
  T004 may move them), make `starter_deck()` / `starter_collection()` read
  them, drop the non-test `DEFAULT_SIDE_DECK` import, and update the module /
  fn docs. In `app.rs` add the pure `fn player_deck_for(is_campaign: bool,
  built: &[Card]) -> Vec<Card>` and make `start_match` deal
  `player_deck_for(campaign.is_some(), self.profile.deck())` — read
  `is_campaign` before the existing `match campaign` moves it; update the
  `start_match` doc (plan §Design 2; the deck-validity divert in
  `open_opponent_select` stays — plan tension §7). In `card.rs` edit only the
  two doc comments (`DEFAULT_SIDE_DECK` is the *standard* deck — opponent
  baseline and Quick Play — not the starter; `deal_hand`'s "the player uses
  DEFAULT_SIDE_DECK"). Tests: rewrite `default_profile_has_a_valid_deck_within_the_collection`
  as `default_profile_plays_a_valid_outer_tier_starter` (deck ==
  `STARTER_SIDE_DECK`, != `DEFAULT_SIDE_DECK`, valid, collection = deck +
  spares, every deck and spare card `economy::card_tier(..) == Outer`); add
  `an_existing_profile_keeps_its_premium_deck_collection_and_credits` (JSON
  with `deck = DEFAULT_SIDE_DECK`, `collection = DEFAULT_SIDE_DECK + [+1, −1,
  ±2]`, `credits: 75` loads with exactly those; `PROFILE_VERSION == 1`); add
  `quick_play_deals_the_standard_deck_and_campaign_deals_the_built_one` in
  `app.rs` with a built deck sharing no card with the standard one; in
  `deck_builder.rs` restage every test that reads the default profile's exact
  slots (`enter_moves_a_present_card…`, `a_partly_decked_card…`,
  `a_type_present_in_one_panel…`, `an_unowned_type_is_a_placeholder…`,
  `arrows_skip_placeholders…`) through the existing `profile_from(collection,
  deck)` helper with explicit layouts, so no test depends on the starter's
  composition, and fix the `default_profile` doc comment (or remove the helper
  if nothing uses it). `game.rs`, `player.rs`, `save.rs` untouched. (Copies the
  `DEFAULT_SIDE_DECK` const shape in `card.rs` and the `profile_from` staging
  pattern already in `deck_builder.rs` tests.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  (reported verbatim) — the three new/rewritten tests above pass; the existing
  `reset_to_starter_wipes_the_run_but_preserves_lifetime_stats`,
  `missing_or_garbage_json_is_rejected_but_an_empty_object_is_the_starter`, and
  every `save.rs` / `game.rs` test pass unchanged; `grep -n DEFAULT_SIDE_DECK
  src/profile.rs` shows test-only uses; `git diff --stat` lists only the four
  files named.*

- [x] **T002 (foundational, review: per-task)** — Create `tests/balance.rs`
  exactly as plan §Design 3: constants (`SCRIPTED_STAND_AT = 17`, `STEP_CAP =
  5_000`, `DEFAULT_N = 10_000` overridable by `KAAZAP_SIM_N`, `GUARD_N = 600`,
  `TOL = 0.02`), the three pool-best candidate consts, `Move`,
  `scripted_move` (plan tension §2, rules 1–4 in that order, flips never
  played), `play_match` (the plan's loop verbatim: stale-frame `update()`,
  `PlayHand` then `ChooseSign { positive: value > 0 }` in the same step,
  `OpponentThinking` → `OpponentTurn`, panic past `STEP_CAP`), `win_rate`,
  `decks()` (starter = `kaazap::profile::STARTER_SIDE_DECK`, standard =
  `DEFAULT_SIDE_DECK`, the three candidates), `opponents_in(tier)` from
  `PLANETS` × `region_tier` × `opponent_by_id`, `tier_price` (min `card_price`
  over the tier's cards), `floor_of`, `ev_per_match`, `measure(n)` over the
  5×10 grid, `targets` (T1–T8 plus the coupled `C` line — best `EV_m` at the
  floor and at 2×floor against `2·EV_g`, plan §Design 3 / tension §8) and
  `bounds` (B1–B5) with the plan's exact formulas and PASS/FAIL strings, and
  the `#[ignore]` `balance_table` that prints the table, both blocks, and the
  `summary: targets a/8, coupling c/1, bounds b/5` line — never asserting.
  Ordinary tests:
  `scripted_player_follows_its_rules_on_fixed_boards` (every board in plan
  §Design 3), `named_decks_are_ten_cards_from_their_pools`,
  `a_scripted_match_terminates_against_every_roster_opponent` (one match per
  pair). No new crate; nothing under `src/` changes. (Copies
  `full_match_terminates_within_bounded_updates` and the `opponent_at` board
  helper in `game.rs` tests.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green, reported
  verbatim, with the three new tests listed as passing and `balance_table`
  as ignored; then `KAAZAP_SIM_N=200 cargo test --release --test balance
  balance_table -- --ignored --nocapture` prints 50 rows (5 decks × 10
  opponents in roster order), eight `T` lines plus the `C` line, five `B`
  lines with the constants header `(SEED 50, reserve 10, P_outer 20, P_mid 50,
  P_core 120)`, and the summary line — output pasted verbatim (the PASS/FAIL
  values are not judged here). Report that `git diff --stat` shows only
  `tests/balance.rs`.
  **PAUSE for the person** (phase attestation, profile/saves backed up and
  checksum-restored): Quick Play with an Outer-only built deck deals ±6 /
  flip / tiebreaker cards over a match; a campaign match deals only Outer
  cards; the deck-builder album shows 7 owned types and 8 placeholders and fits
  139×31; the shop reads the new owned counts; New Campaign and a run-over
  reset both hand out the new starter.*

## Phase 2 — Measure, then tune

<!-- Findings carried from the T002 per-task review (2026-09-13), for T005–T007:
(a) the `C` line (EV form, all Mid opponents) and B4 (ceil'd match counts,
filtered to w > 0.5) agree whenever EV_g > 0 but are not identical — the doc
states which form T004's stop condition reads; (b) T8's TOL applies per
adjacent pair (outer ≤ outer_mid ≤ full), not end-to-end, and the same TOL is
applied to the "sovereign hardest" half — the doc states both; (c) B4's
`@2×floor` count is the floor-optimal opponent's, not the min over opponents
(one-match edge); (d) `easiest_other` in T8 is really the hardest other
opponent and prints as "next" — rename when the file is next touched (T006);
(e) T006 must drop GUARD_N's `#[allow(dead_code)]`; (f) `git diff --stat`
can't show an untracked file — T002's scope was confirmed via `git status`.
From the Phase 1 review (2026-09-13): (g) T004 adds
`assert!(ALL_SIDE_CARDS.contains(card))` to the starter test's tier loop so a
retuned starter can never leave the album's universe; (h) T007 fixes the
`start_match` precondition paragraph in app.rs — an undersized deck now only
affects campaign matches, Quick Play always deals 10. -->

<!-- The baseline measurement fixes the pool-best decks and records the
untuned curve; T004 tunes opponents/starter/tiers to the win-rate targets; T005
tunes prices and economy constants to the bounds. Each tuning task is a bounded
loop at the implementation tier; non-convergence goes to the person. Phase
ends with the spec's play attestation. -->

- [x] **T003** — Baseline. Build the release test target first (`cargo test
  --release --test balance --no-run`) so the timed runs exclude the one-time
  compile, then run the documented command twice at `DEFAULT_N` (`cargo test
  --release --test balance balance_table -- --ignored --nocapture`, 500k
  matches), timing each (`time`). Then, for each pool, measure up to
  three hand-built candidate decks (the plan's plus at most two alternatives
  each, no flips, buildable from the pool) against that pool's region
  opponents at `N = 2000` by temporarily swapping the const, and fix the best
  as `BEST_OUTER` / `BEST_OUTER_MID` / `BEST_FULL`. Create
  `specs/022-balance-pass/tuning-log.md` with: both full runs, the per-pair
  max |Δ| between them, the wall times, the candidate tables with the choice
  and reason, and the baseline summary line. If any pair's |Δ| exceeds 2.5
  points, double `DEFAULT_N` **once** (to 20_000), re-run the pair of runs,
  and report the result either way — no further raising; a second miss is
  reported as a finding, not fixed. Plan tensions §3, §5. (Copies nothing — a
  measurement task; the only code edits are the three candidate consts and
  possibly `DEFAULT_N`.)
  *Verify: `tuning-log.md` holds two complete 50-row tables with their wall
  times (stated as excluding the compile) and the per-pair max |Δ| — ≤ 2.5
  points, or one doubling of `DEFAULT_N` recorded with the re-run's max |Δ|
  — plus the candidate tables; the reported wall time of one run is under
  60 s (the plan's "seconds" claim — report the actual figure); `cargo build
  --all-targets` / `cargo test -q` green; `git diff --stat` shows only
  `tests/balance.rs` and the log.*

- [x] **T004** — Tune the win-rate curve (targets T1–T8 **and the B4
  coupling `C`** — `EV_m > 2·EV_g` for some Mid Rim opponent, at its floor or
  at 2×floor; plan tension §8 shows why only T004's levers move it). Levers and
  limits exactly plan §Design 4 (T004 bullet): `OPPONENTS` values inside the
  kept structure, `STARTER_SIDE_DECK` / `STARTER_SPARES` (Outer only),
  `card_tier` (with `card_tier_partitions_the_universe`'s lists updated), the
  three candidate consts. Never the scripted player, the loop, `card_price`,
  or the economy constants. Start from plan tension §8's pressure points.
  Loop: edit → run the command at `DEFAULT_N` → read the `T` lines and the `C`
  line; at most **six** iterations; append each iteration's changed values
  and summary line to `tuning-log.md`. Stop when all eight targets and `C`
  pass, or at six with the stuck line named and what was tried. Re-sync
  nothing in `docs/` yet (T007). (Copies the data-only shape of the
  `OPPONENTS` / `card_tier` consts.)
  *Verify: the final run's eight `T` lines all read PASS, the `C` line reads
  PASS (say whether at the floor or `PASS (above floor)`), and the summary
  says `targets 8/8, coupling 1/1` (pasted verbatim, with `N`); `cargo build
  --all-targets` / `cargo test -q` green including `roster_runs_easy_to_hard_by_threshold`,
  `misplay_rates_are_valid_and_the_default_is_deterministic`,
  `the_final_boss_is_flawless_and_fully_equipped`,
  `card_tier_partitions_the_universe`, `default_profile_plays_a_valid_outer_tier_starter`;
  `tuning-log.md` has one entry per iteration. If not converged: the return
  names the stuck target, the pinned lever, and the six tables — the
  orchestrator re-dispatches once as T004a with the log; a second
  non-convergence goes to the person as a plain-language question (which
  target, which lever is pinned by which other target), not to a decision
  review and not to the orchestrator's escape hatch (CLAUDE.md's
  do-it-yourself rule does not apply to tuning — plan §Design 4).*

- [x] **T005** — Tune the economy bounds B1, B2, B3, B5. Levers exactly plan
  §Design 4 (T005 bullet): `card_price` by tier, `SEED_PURSE`,
  `ANTE_BASE_THRESHOLD`, `ANTE_PER_THRESHOLD_STEP` — **not** `PAYOUT_RATIO`,
  **not** `STAKE_STEP` (it enters no bound; `wager.rs` stays untouched), and
  nothing T004 owns. B4 is not this task's: the final run re-reads it, and if
  it reads FAIL the return says so and the orchestrator re-dispatches
  **T004a** (its levers are T004's — plan tension §8), not T005a — but only
  when B4 fails on T004's data: lowering `ANTE_BASE_THRESHOLD` changes the
  floor *ratio* and can break a passing `C` line, and that is this task's own
  edit to revert, never a T004a request. Amend the
  exact-value tests the new constants break in the same edit
  (`ante_floor_is_the_difficulty_scalar`, `payout_is_even_money` untouched,
  `earning_grows_the_balance_and_purchase_holds_back_the_ante_reserve`'s 59/60
  boundary, `every_card_has_a_positive_price_that_rises_with_tier`,
  `cheapest_floor_is_the_min_over_launchable_nodes`'s `== 10`) — to the new
  values, never loosened. Same bounded loop as T004 (six iterations, log every
  one). A final run after the last edit is the table `docs/balance.md`
  records.
  *Verify: the final run's five `B` lines all read PASS (B4 may read `PASS
  (above floor)` — say which) and the summary says `targets 8/8, coupling
  1/1, bounds 5/5` (pasted verbatim); `cargo build --all-targets` / `cargo
  test -q` green; `PAYOUT_RATIO == 1` and `STAKE_STEP` unchanged; the
  working-tree diff for this task (`git diff --stat` before the commit) lists
  only `src/economy.rs`, `tuning-log.md`, and the test files whose exact
  values the constants pin (`src/profile.rs` tests, `src/economy.rs` tests) —
  no `src/opponent.rs`, no non-test `profile.rs` line (the orchestrator reads
  `git diff -- src/profile.rs` and confirms every hunk sits under
  `#[cfg(test)]`; `--stat` alone can't show it), no `wager.rs`.
  Non-convergence on B1/B2/B3/B5 escalates as in T004 (one T005a, then the
  person; never the escape hatch). **PAUSE for the person** (phase attestation, profile/saves backed up
  and checksum-restored): on a fresh campaign the starter clears the Outer Rim
  over a few attempts; the Mid Rim is a wall until Mid cards are bought; the
  shop and deck-builder read correctly with the new collection; Quick Play
  plays with the premium deck. The orchestrator's pause report states the
  measured table's headline numbers in plain words.*

- [x] **T004a** (person's Phase 2 finding, 2026-09-13) — Retune the Outer
  Rim so its play reads as weak rather than random: give Greeb, Dax and Vessa
  really weak decks (Outer-tier, e.g. mostly 1s) and bring their `misplay`
  rates down as far as the targets allow (aim: Greeb ≤ 0.25, Dax and Vessa
  lower still; the ramp stays non-increasing with Greeb the roster max). Same
  levers and limits as T004 (T004's bullet in plan §Design 4), same bounded
  loop (≤ 6 iterations, each logged in `tuning-log.md` under `## T004a`),
  same stop condition: `targets 8/8, coupling 1/1, bounds 5/5` on the final
  run — the T005 prices stay fixed, so B2/B3 must still hold with the new
  `w_g`. If the misplay aim and the targets cannot both be met, stop and
  report the lowest misplay ramp that passes, for the person.
  *Verify: final run's summary line verbatim (with N); the three Outer Rim
  misplays and decks (old → new); `cargo build --all-targets` / `cargo test
  -q` green; `git diff --stat` shows only `src/opponent.rs`, possibly
  `src/profile.rs` (starter values) / `tests/balance.rs` (candidate consts),
  and the log. The final table replaces T005's as the one `docs/balance.md`
  records.*

- [x] **T004b** (person's ruling, 2026-09-13 — a deliberate exception to the
  spec's "roster changes" non-goal, limited to two blurbs) — Reword Nima's
  and Kesh's in-game blurbs in `src/opponent.rs` so each describes the plain
  (`Basic`) strategy they now run, in the same voice and length as the other
  blurbs; nothing else about the roster text changes. Fix the matching prose
  in `docs/opponents.md` in T007, not here.
  *Verify: `cargo build --all-targets` / `cargo test -q` green; `git diff
  --stat` shows only `src/opponent.rs`; the two new blurbs fit the
  opponent-select line width (the driver snapshot at the Phase 3 pause).*

## Phase 3 — Guards and docs

<!-- Findings carried from the Phase 2 review (2026-09-13):
(i) T006 sizes every guard bar from the SE at GUARD_N (≈1.9 pts at N=600),
not from T003's 1.1-point max |Δ| at N=10 000; (j) T007 corrects the
tuning-log line in T004 iteration 1 claiming "the Sovereign's two dead flips
were worth ~11 points" — the Sovereign never carried flips (Rix and the
Magistrate did); the grounded flip-cost figure is the Magistrate/Sovereign
gap (5.2 pts on the final table); (k) docs/balance.md presents "a dead flip
costs ~4–5 points" and "the AI peaks near effective threshold 18" as tuning
observations, not measured constants; (l) docs/balance.md notes T8 is
tolerance-carried on the Mid Rim rows (smallest outer_mid → full gap +0.9 on
Dax, TOL 2); (m) docs/balance.md states the starter spares are lateral (one
new type, ±1) and the measured starter rates describe a player who does not
rebuild; (n) docs/opponents.md shows Rix/Kesh/Magistrate/Sovereign all at
threshold 19 / floor 50 — say so plainly; (o) closeout DECISIONS text records
that `strategy` is a tuning knob independent of blurb text (Nima Cautious →
Basic, Kesh Aggressive → Basic), pending the person's ruling at the Phase 2
pause on whether the blurbs get reworded. -->

<!-- The unit-test guards sized against the measured gaps, then the balance doc
and the two re-synced docs. Phase ends with a short pause: the docs are
readable and the guards are green. -->

- [x] **T006** — `tests/balance.rs`: the guards. Add
  `starter_deck_beats_greeb_above_the_floor` (rate ≥ 0.50),
  `starter_deck_cannot_credibly_take_the_core` (≤ 0.50 vs each Core opponent),
  `the_full_pool_deck_outperforms_the_starter_against_every_opponent` (strict
  `>` per opponent), all at `GUARD_N`. From T005's final table compute each
  guard's margin in standard errors (single rate: `(w − bound) / √(w(1−w)/N)`;
  the difference: `gap / √((w₁(1−w₁) + w₂(1−w₂))/N)`), raise `GUARD_N` until
  every margin ≥ 5 SE, and write the margins in a comment above the guards.
  Plan tension §6. (Copies `win_rate` + the existing assertion style in the
  same file.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green **three times
  in a row** (all three tails pasted); the smallest margin and `GUARD_N`
  reported with the arithmetic; `time cargo test --test balance` reported
  (debug) — under 30 s; `git diff --stat` shows only `tests/balance.rs`.*

- [x] **T007** — Docs. Create `docs/balance.md` (plan §Design 5: command and
  `N`, the all-pairs agreement figure, the scripted player rules verbatim plus
  its two stated limitations — it never plays a flip, and rule 3 stands on a
  tie at ≥ 17 even when the opponent alone holds a tiebreaker in play (a sure
  loss; the proxy is kept as is) — the five decks with the candidate
  alternatives and rates, the
  targets table with measured values / `N` / date, the bounds with the
  arithmetic shown, the guards and margins, how to re-run and what to update
  after a change). Re-sync `docs/opponents.md` (roster table, the per-opponent
  prose where a value moved, the "first cut" closer → tuned in spec 022) and
  `docs/economy.md` (constants, ante, tier/price tables; starter is Outer-tier;
  "first guesses" → tuned; Quick Play deals the standard deck). `README.md`:
  one clause at the Quick Play sentence. Update the `profile.rs` / `app.rs`
  docs if T004/T005 moved anything they name. Draft
  `specs/022-balance-pass/closeout-main-docs.md` with the `ROADMAP.md`
  (balance pass shipped; Difficulty setting has its baseline; lines 90–91
  starter note) and `DECISIONS.md` (rulings A–H, plan tensions §1, §2, §4, §5,
  §7; lines 115–118 superseded) text to apply on `main` after the merge —
  never on the branch. (Copies `docs/economy.md`'s section shape and 021's
  `closeout-main-docs.md`.)
  *Verify: `cargo build --all-targets` / `cargo test -q` green; every value in
  the three tables equals the const it snapshots — report a `grep` per
  constant and per roster threshold/misplay; `balance.md`'s table is T005's
  final run byte-for-byte in the numbers; `git diff --stat` shows docs, README,
  the spec-dir files, and at most comment lines in `src/`. **PAUSE for the
  person**: the docs read correctly; nothing to try in play.*

- [x] **T006a** (T007 finding, 2026-09-13) — `tests/balance.rs` comments
  only: the guard-margin comment names Kesh (Mid Rim) as the binding Core
  opponent — the Core guard iterates rix/magistrate/sovereign, so the binding
  margin is Rix at 10.8 SE (Kesh's 6.5 SE figure is not a guard); and the
  file's line-2 command says `KAAZAP_SIM_N=4000` while `DEFAULT_N` and the
  documented command are 10 000. Fix both; `docs/balance.md` already states
  the corrected figures. No code change.
  *Verify: `cargo build --all-targets` / `cargo test -q` green; `git diff`
  shows comment lines only in `tests/balance.rs`.*

- [x] **T007a** (T006a finding, 2026-09-13) — `docs/balance.md` only: the
  guards section repeats the Kesh misattribution in two places (the Core
  guard's margin is Rix at 10.8 SE; the smallest guard margin is 8.8 SE, full
  vs starter against Greeb; Kesh's 6.5 SE is a Mid Rim figure no guard
  covers). Match the corrected comment in `tests/balance.rs`.
  *Verify: `grep -n "6.5 SE\|binding Kesh" docs/balance.md` shows only a
  correctly-framed non-guard mention, or nothing; the numbers equal the
  `tests/balance.rs` comment.*

- [x] **T007b** (Phase 3 review notes, 2026-09-13) — text only, three
  files: (1) `tests/balance.rs` margin comment and `docs/balance.md`'s
  matching sentence: the smallest starter-vs-Mid-Rim margin at GUARD_N is
  Toran at 4.9 SE (w = .402 → .098/.02002), below the 5 SE bar — which is why
  no Mid Rim guard exists; Kesh (6.5 SE) is the largest, not the smallest;
  (2) `docs/balance.md` re-run step 3: add this file's own guards table and
  margin paragraph to the list of things to update; (3)
  `specs/022-balance-pass/closeout-main-docs.md` ROADMAP text: the pre-022
  shipped Greeb slip rate was 0.25, not 0.44 (0.44 was an intra-spec peak) —
  say "0.25 → 0.18". Optionally the minor (a)–(c) wording notes from the
  review if trivial.
  *Verify: `cargo build --all-targets` / `cargo test -q` green; `git diff`
  shows comment/doc lines only.*

- [x] **T008a** (sweep findings, 2026-09-13) — text only: (B1) `src/card.rs`
  test comment above the `DEFAULT_SIDE_DECK` universe loop — it is the
  standard deck, not the starter; (B2) "all-±1 decks/hands" → "all-1s decks
  (`+1`/`−1` only, no ± card)" in `docs/balance.md` rule of thumb 3 and both
  places in `closeout-main-docs.md`; (S1) the same paragraph's "(Greeb 0.44 →
  0.18)" gains "0.44 mid-tuning; 0.25 as shipped before this pass"; (S2)
  `Readme.md` line ~71 "dealt from each match" → "each campaign match"; (S3)
  a fourth ROADMAP edit in `closeout-main-docs.md` qualifying line 84's
  "Matches deal the player's hand from the built deck" with "(campaign
  matches, since spec 022)"; (S4) `docs/opponents.md` "the two masters' card"
  → three opponents hold the tiebreaker (Old Toran too); (S5) soften
  `docs/opponents.md`'s "found that this AI plays better at 18" to the
  rule-of-thumb wording balance.md uses; (S9) closeout "Four ordinary tests
  guard … the curve" → three sampled guards plus one exact tier test; (S6)
  run the documented command twice on the shipped data, record the per-pair
  max |Δ| in `tuning-log.md` (`## T008a — shipped-data agreement`) and cite
  it in `docs/balance.md` alongside T003's figure.
  *Verify: `cargo build --all-targets` / `cargo test -q` green; `git diff`
  shows comment/doc lines only under `src/`; the two runs' max |Δ| reported.*

## Final phase — Spec close-out

- [ ] **T008** — Flake check, sweep, AC checkoff. Run `cargo test -q` **ten
  consecutive times** and paste the ten tails. Mechanical checks: `git diff
  main --stat` shows no `game.rs`, `player.rs`, `save.rs`, `campaign.rs`,
  `wager.rs`, `Cargo.toml`, `Cargo.lock`; `git diff main -- src/card.rs`
  changes comment lines only; `PROFILE_VERSION == 1` and `SAVE_VERSION == 1`;
  `PAYOUT_RATIO == 1`; `cargo build --all-targets` warning count equals
  `main`'s; `grep -rn "starter" docs README.md src` finds no sentence calling
  the starter the default/premium deck. Check off `spec.md` acceptance
  criteria with evidence (the verbatim lines). Request the pre-merge
  whole-spec sweep; apply `closeout-main-docs.md` on `main` after the merge.
  *Verify: ten green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/022-balance-pass/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product
owner**. Dispatch each task to the `sdd-implementer` per the model policy;
verify by running the build and tests yourself, then commit. **The one task
marked `review: per-task` (T002):** run the `skeptical-reviewer` after it,
scoped to its diff, plan §Design 3, tensions §2–§4 and §8's B4 bullet (the
derivation the `C` line implements), and the simulator
acceptance criterion (a shell-assembled bundle), one review plus at most one
re-review, and re-run the verification command yourself before committing.
**Every other task (T001, T003–T008):** review at phase end. Pause for the
person after each phase and whenever something unexpected bears on spec
adherence. The **Phase 1 pause** is the starter/Quick-Play look (new
collection in the builder and shop, premium hand in Quick Play, Outer hand in
campaign); the **Phase 2 pause** is the curve attestation (Outer Rim clearable,
Mid Rim a wall, shop/builder legible, Quick Play premium); the **Phase 3
pause** is a read of the docs. Back up + checksum-restore the real
profile/saves before every driver session — this spec resets real profiles by
design.

Model & effort: the session runs at the session tier (`claude-fable-5-1`,
medium) per `.claude/settings.json`; the planner and the sign-off ran at the
top tier (`fable`) by per-call override. **Binding ruling (person,
2026-09-13): every implementation task, every simulator run, and every tuning
iteration runs in the `sdd-implementer` at `opus`; no simulator output is sent
to the top tier.** A tuning task that does not converge after one re-dispatch
(T004a / T005a) goes to the **person** as a plain-language question, not to a
decision review — and **CLAUDE.md's escape hatch (the orchestrator doing a
task itself after two failed verifications) does not apply to T004/T005**;
the session never tunes by hand. A B4 failure surfacing in T005 is a T004a
re-dispatch, not a T005 lever. Per-phase and per-task reviews and the sweep
run at `opus`.
Per Erik's standing ruling, never infer the fable budget from a successful
dispatch — ask if unsure, and log which tier actually ran. Every
session-ending pause ends with a continuation prompt (spec directory, files to
read, where to resume, involvement level, pause cadence, any model switch) in
its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo, and why). Compare the spec total against spec 021 before treating the
policy as settled. Tuning tasks: one row per dispatch (T004, T004a, …), with the
iteration count in the outcome column. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| **Experiment 1, spec 2** — session `claude-fable-5-1` at medium throughout. Fable allowance at start: 68% left (person's reading, 2026-09-13, after Phase 1); at spec end: (read after the merge). Person's ruling 2026-09-13: all implementation, simulator runs, and tuning at `opus`. | — | — | header |
| Planning: draft + sign-off fixes (sdd-planner) | fable | ~230K (budget counter at return; ~205K draft + ~25K fixes) | drafted; three design edges flagged (plan §Open questions); sign-off's 3 blocking findings (B4 ownership → T004; DEFAULT_N 10_000 sized for all 50 pairs; T005 diff check vs working tree) and 5 notes applied |
| plan + tasks sign-off (skeptical-reviewer) | fable | ~115K (measured return) | 3 blocking (B4 owned by a task with no lever over it; DEFAULT_N sized per pair, not per grid; T005 verify uncheckable) + 5 notes — all sent to the planner and applied |
| sign-off re-review (skeptical-reviewer) | fable | ~20K (measured return) | signed off; 3 wording notes applied by the orchestrator (T005: profile.rs hunks read under cfg(test); T005: own ANTE_BASE change is its own revert, not a T004a; T002 review bundle adds tension §8) |
| T001 impl (sdd-implementer) | opus | ~61K (measured return) | done first try; restaged a sixth deck_builder test (`nav_reports_moved_only_when_it_actually_moves`) that also read the old starter slots — inside the task's stated goal |
| T002 impl (sdd-implementer) | opus | ~99K (measured return) | done first try; two plan fixed-boards were arithmetically impossible (23−2≠18; 16+4=20 triggers rule 2) — covered the rules with corrected boards, reviewer confirmed faithful |
| T002 review (skeptical-reviewer, per-task) | opus | ~84K (measured return) | signed off, 0 blocking, 10 notes — carried to T005/T007 docs (C vs B4 not identical; T8 tolerance is per adjacent pair and also applied to the sovereign-hardest half; `easiest_other`/"next" label misnamed) and T006 (drop GUARD_N allow) and the sweep |
| Phase 1 review (skeptical-reviewer) | opus | ~58K (measured return) | signed off, 0 blocking, 7 notes — N2 (no test pins starter ⊆ ALL_SIDE_CARDS) carried to T004; N1 (start_match precondition doc half-stale) to T007; rest informational |
| T003 impl (sdd-implementer) | opus | ~77K (measured return) | done first try; two DEFAULT_N runs ≈ 4.9 s and 4.8 s (compile excluded), max per-pair |Δ| 1.1 pts, no doubling; BEST_OUTER and BEST_OUTER_MID replaced by measured winners, BEST_FULL kept after an N=10k tie-break; finding: T8 sits at 1.9 vs TOL 2 because the plan's full-pool deck barely beats the Mid deck — T004 may replace it |
| T004 impl (sdd-implementer) | opus | ~146K (measured return) | iterations: 5 (converged at 4; 5th fixed the misplay ramp and re-checked) — `targets 8/8, coupling 1/1`, C passes at the floor; levers used: opponent decks/misplays/strategies/two thresholds, starter deck + spares, BEST_FULL; card_tier untouched |
| T004a retune (sdd-implementer) | opus | ~72K (measured return) | iterations: 2 — Outer Rim decks now all ±1s (Dax/Vessa with one or two +2s), misplays 0.44/0.36/0.34 → 0.18/0.16/0.15 (pinned by Nima's 0.15 and the non-increasing ramp); all 8 targets, C, 5 bounds pass; T1 69.0, B3 k_grind 43 |
| T004b blurbs (sdd-implementer) | opus | ~22K (measured return) | done first try; both new blurbs shorter than the Sovereign's 65-char longest |
| T005 impl (sdd-implementer) | opus | ~57K (measured return) | iterations: 1 — Mid price 50→100, Core 120→200; `bounds 5/5`, B4 PASS at the floor; no exact-value test needed amending (floors unchanged); profile.rs untouched |
| Phase 2 review (skeptical-reviewer) | opus | ~91K (measured return) | signed off, 0 blocking, 8 notes — log factual slip (Sovereign never had flips) + T8 tolerance-carried on Dax + observed-not-measured design notes → T007; guard bars sized from GUARD_N SE → T006; Nima/Kesh strategy vs blurb, four opponents at floor 50, Outer Rim misplay ≥ 0.34, lateral spares → person at the Phase 2 pause |
| T006 impl (sdd-implementer) | opus | ~49K (measured return) | done first try; GUARD_N stays 600 — smallest margin 6.5 SE (starter vs Kesh ≤ 50%), Greeb guard 10.1 SE, full-vs-starter binding at Greeb 8.8 SE; balance test binary 1.1 s debug; notes (d) and (e) applied |
| T007 impl (sdd-implementer) | opus | ~163K (measured return) | done first try; balance.md 289 lines, every table value machine-checked against source and the T004a table; two stale comments found in tests/balance.rs → T006a |
| T006a comments (sdd-implementer) | opus | ~34K (measured return) | done; found the same misattribution in docs/balance.md → T007a |
| T007a balance.md fix (sdd-implementer) | opus | ~23K (measured return) | done; guards table and sizing paragraph now match the test comment |
| Phase 3 review (skeptical-reviewer) | opus | ~94K (measured return) | signed off, 0 blocking, 3 notes + minor: "Kesh 6.5 SE is the smallest Mid Rim margin" is wrong (Toran 4.9 SE is, and is why no Mid Rim guard exists); balance.md re-run step omits its own guards table; closeout ROADMAP text says Greeb 0.44 → 0.18 but the pre-022 shipped value was 0.25 — all three → T007b |
| T007b review-note text fixes (sdd-implementer) | opus | ~29K (measured return) | done; left balance.md rule-of-thumb #3's "0.44 → 0.18" (intra-spec narrative) for the sweep to judge |
| T008 close-out (orchestrator) | fable (session, medium) | — | ten consecutive `cargo test -q` green; no forbidden file in `git diff main`; card.rs comment-only; PROFILE_VERSION 1 / SAVE_VERSION 1 / PAYOUT_RATIO 1; warnings 0 on branch and on main (worktree build); spec.md ACs checked off with evidence |
| Pre-merge whole-spec sweep (skeptical-reviewer) | opus | ~146K (measured return) | fix-and-re-review: 2 blocking text errors (card.rs test comment still calls DEFAULT_SIDE_DECK the starter; "all-±1 decks" in balance.md and the closeout text — the Outer Rim decks hold no ± card) + 9 notes → T008a |
| T008a sweep fixes (sdd-implementer) | opus | ~48K (measured return) | B1, B2, S1–S5, S9 fixed; S6: two shipped-data runs both 8/8 · 1/1 · 5/5, max |Δ| 1.5 (starter vs nima), 0/50 over 2.5 |
| Sweep re-review (skeptical-reviewer) | opus | | |
