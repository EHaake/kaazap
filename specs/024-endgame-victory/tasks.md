# Tasks: Endgame, victory & what you keep — spec 024

> **Status**: Signed off (skeptical-reviewer at opus, 2026-09-15; re-review's B3 — the three spec.md amendments — pending the person's ratification)
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
implementer returns, and only the orchestrator commits. **Experiment 2 is paused
for this spec** (the person's ruling of 2026-09-14 stands until the Fable
allowance resets): every task below is dispatched to the `sdd-implementer` at
`opus`, not to `sdd-implementer-fable`.

**Foundational phase:** Phase 1 (T001–T003 — the statistics fields, the profile's
resolution seam and resets, and the app's match-resolution block) is
foundational: every later task reads the counters, the first-clear record or the
`Settlement` edge that Phase 1 defines, and a mistake in the settlement seam
would be written into real `profile.json` files before anything downstream shows
it. Under the constitution's review cadence the default is a **per-phase**
`skeptical-reviewer` pass; **T002 alone carries `review: per-task`**, because it
owns the record-then-settle order, the once-only completion edge and the two
resets — the three things the notices, the Records line and New Campaign all
inherit. Every other task is reviewed at phase end.

---

## Phase 1 — The data model and the resolution seam (foundational)

<!-- Dependency order inside the phase: the statistics fields (T001) before the
profile methods that write them (T002) before the app block that calls them
(T003). No UI in this phase; it ends with a review, not a person pause — there
is nothing new to see on screen yet. -->

- [x] **T001 (foundational)** — `src/stats.rs`: the run counters, the
  first-clear record, and the shared summary. Add `#[serde(default)] pub
  credits_won: u32` and `#[serde(default)] pub credits_lost: u32` to `RunStats`
  with the doc comments in plan §Design 1, plus `matches_played()`,
  `record_credits_won(amount)` and `record_credits_lost(amount)` (both
  saturating). Add `#[serde(default)] first_clear_matches: Option<u32>` to
  `LifetimeStats` with its accessor, and give `record_campaign_completion` a
  `matches_played: u32` parameter that sets the record **only when
  `campaign_completions == 0`** (plan §Design 1, exact body). Update both
  callers in this task, so the crate builds green at its end (sign-off B1):
  `src/profile.rs:252` becomes
  `let matches = self.campaign.run_stats().matches_played();` then
  `self.stats.record_campaign_completion(matches);` (disjoint field borrows),
  and the `records.rs` test that calls it
  (`campaign_has_completions_line_others_do_not`) passes a number. Note in the
  `settle_campaign_match` doc that the number is the run's matches played
  *including* the completing match once `resolve_match` (T003) owns the order —
  until then the app still settles before recording, so a completion reached
  between T001 and T003 would record one short; nothing reads the record until
  T006, so it is invisible. Add `pub fn run_summary_lines(run, worlds_cleared,
  worlds_total) -> [String; 3]` with the three exact format strings in plan
  §Design 1 (`worlds_total` is `PLANETS.len()` = 8 at the call sites; the
  spec's example figures are illustrative). Tests:
  `run_counters_default_zero_round_trip_and_accumulate`,
  `the_first_completion_sets_the_record_and_later_ones_never_do` (including the
  deserialized-with-completions case — plan §Tests bullet 2), and
  `run_summary_lines_read_the_run_tally` (bullet 3). (Copies the `Streak` /
  `RunStats::record_match` field + method shape and the `win_rate_*` test shape
  in the same file.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  (reported verbatim) with the three tests named passing and every existing
  `profile.rs` test passing unchanged; `git diff --stat` shows only
  `src/stats.rs`, `src/profile.rs` and `src/records.rs`, and the `profile.rs`
  hunk is only the `record_campaign_completion` call site and its doc.*

- [x] **T002 (foundational, `review: per-task`)** — `src/profile.rs` +
  `src/campaign.rs`: the resolution seam, the map-only reset, and the entry
  predicate. In `campaign.rs` add `pub fn worlds_cleared(&self) -> usize` (plan
  §Design 2). In `profile.rs` add `#[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct Settlement { pub outcome: StakeOutcome, pub completed_run: bool }`
  and `pub fn resolve_match(&mut self, opponent_id, player_won, player_rounds,
  opp_rounds) -> Option<Settlement>` exactly as plan §Design 3 (doc and body
  included — it records, captures `was_complete`, settles, bumps the two credit
  counters from the returned `StakeOutcome`, and reports the edge).
  **`settle_campaign_match` is not touched in this task** — its signature and
  body stay as T001 left them, so `app.rs`'s call site and every existing
  settling test still compile (sign-off B2, plan tension §1). Add `pub fn
  reset_campaign_run(&mut self)` (`self.campaign = CampaignRun::default()`, doc
  per plan) and `pub fn differs_from_starter(&self) -> bool` (progress, or
  credits / collection / deck differing from the starter). `record_match` and
  `settle_campaign_match` stay `pub` **in this task** (T003 privatizes them once
  `app.rs` stops calling them); `reset_to_starter` is unchanged. Tests (plan
  §Tests bullets 4–11): re-point
  `campaign_completion_counts_only_a_final_clearing_win_and_recounts_after_reset`'s
  `win_node` helper and
  `a_rematch_settles_for_credits_but_changes_no_progress_or_completions` at
  `resolve_match` (the latter would otherwise be vacuous — it calls settlement
  directly, which no longer carries the new outputs), and add
  `resolve_match_moves_the_run_credit_counters_and_nothing_else_does`,
  `the_run_counters_and_first_clear_round_trip_and_default_for_older_profiles`,
  `the_first_clear_counts_the_completing_match_and_survives_a_replay`,
  `resolve_match_reports_the_completion_edge_and_skips_quick_play`,
  `the_completion_edge_and_the_completions_counter_always_agree` (bullet 8 —
  the two evaluations of `!was_complete && run_complete()`),
  `new_campaign_resets_the_map_and_keeps_the_pool`,
  `the_entry_panel_shows_whenever_the_run_or_the_pool_differs_from_the_starter`,
  and in `campaign.rs` `worlds_cleared_counts_cleared_planets`. The private
  `profile_with` test helper constructs `Profile` literally — it needs no new
  field here, but check it still compiles. (Copies `settle_campaign_match`'s own
  shape, the `reset_to_starter_wipes_the_run_but_preserves_lifetime_stats_and_onboarding_marks`
  test, and `is_broke`'s pure-predicate shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the eight tests named passing and every existing `settling_*` /
  `staking_*` / `is_broke_*` test passing **unchanged** (they assert on
  `Option<StakeOutcome>`, which this task leaves alone);
  `git diff --stat` shows only `src/profile.rs` and `src/campaign.rs`; the
  implementer's report quotes the `resolve_match` body and the completion-edge
  lines verbatim. Orchestrator re-runs the verification command itself before
  committing (per-task review).*

- [x] **T003 (foundational)** — `src/app.rs` (+ two `pub` removals in
  `src/profile.rs`): the match-resolution block. Replace the `settle_campaign_match`
  → `record_match` pair in `tick`'s `GameOver` block with the single
  `resolve_match` call in plan §Design 4 (banner from `settlement.outcome`;
  the `completed_run` flag is consumed in T004, so **ignore it here** — do not
  add an unread field), delete the local `mode` computation and the `stats::Mode`
  import, and rewrite the block's comment to say that one method now owns
  record-then-settle and why the order matters (plan tension §1) — the comment
  must **not** name `settle_campaign_match`, `record_match` or `Mode`, since the
  Verify below greps for those identifiers. Then make
  `Profile::record_match` and `Profile::settle_campaign_match` private (drop
  `pub`); their `profile.rs` tests are in the same module and keep working.
  No behavior change is expected in this task. (Copies nothing new — it is a
  contraction of the existing block.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with no test changed by this task; `grep -n
  "settle_campaign_match\|record_match\|Mode::" src/app.rs` is empty; `git diff
  --stat` shows only `src/app.rs` and `src/profile.rs`.*

## Phase 2 — The ending on the map

<!-- The notices first (T004, which also introduces the victory flag the tick
block set up), then the map marker (T005) and the Records line (T006). Phase
ends with a review and a driver walkthrough of a completed run — reported, not
stopped at (the person's 2026-09-15 run-straight-through ruling). -->

- [x] **T004** — `src/app.rs`: the victory notice and the run summary on both
  notices. Add `Modal::Victory` (unit-like, doc-commented like `RunOver`) and
  the `victory_due: bool` field on `App` (false in `new`), set in `tick`'s
  resolution block from `settlement.completed_run` and **taken** by
  `enter_campaign_map` (`std::mem::take`) — plan §Design 4 wiring. Rename
  `onboarding_dismissed` to `notice_dismissed` (doc: the primer, the first-match
  popup and the victory notice; rename its test to
  `notice_dismissed_on_enter_space_or_esc_only`). Give `map_entry_modal` the
  `victory_due` parameter in the precedence run-over → victory → primer. Add the
  pure `victory_notice_lines` and `run_over_notice_lines` (plan §Design 4, the
  exact line tables, blank rows included), replace `draw_run_over` with
  `draw_notice(&self, frame, lines)` (row 0 Strong, last row Muted, the rest
  Normal, all `Align::Center`, `OverlayLayout::new(config, widest,
  lines.len())`), and add the `Modal::Victory` branch to `handle_key`'s modal
  chain (after `RunOver`: `notice_dismissed` → close + `Sfx::MenuSelect`,
  everything else swallowed) and the two `draw` arms. Both builders read
  `profile.campaign().run_stats()`, `…worlds_cleared()` and `PLANETS.len()`.
  Tests: `map_entry_modal_prefers_run_over_then_victory_then_primer` (plan
  §Tests bullet 12), `notice_dismissed_on_enter_space_or_esc_only` (renamed),
  and `both_notices_read_right_breathe_and_fit_the_minimum_terminal` (bullet
  14 — exact first/last lines, the blank row above the dismiss line, no two
  consecutive blanks, the summary block between the spec'd blocks, the run-over
  reset note after the summary, `OverlayLayout` unclamped at 139×31). (Copies
  `Modal::RunOver` + `handle_run_over_input` + `draw_run_over`, and
  `wager.rs`'s `the_prompt_fits_the_minimum_terminal_unclamped`.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the three tests passing; `git diff --stat` shows only
  `src/app.rs`; the implementer's report quotes the `enter_campaign_map` body
  (showing the `mem::take`) and the `Modal::Victory` chain branch.*

- [x] **T005** — `src/campaign_map.rs`: the completed marker. Add the pure `fn
  axis_line(run_complete: bool) -> (&'static str, Emphasis)` (plan §Design 5 —
  `"★  Campaign complete"` Strong when complete, the existing
  `"Outer Rim  →  The Core"` Muted otherwise) and call it from `draw_header`'s
  `None` banner branch; switch `draw_header`'s inline cleared-planet count to
  `run.worlds_cleared()`. The banner branch, the planet panel and `banner_line`
  are untouched. Test: `the_header_axis_gives_way_to_the_completed_marker`
  (both arms, text and emphasis). (Copies `banner_line`'s pure-fn + test shape
  in the same file.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/campaign_map.rs`; `git diff --
  src/campaign_map.rs` has no hunk outside `draw_header`, the new fn and `mod
  tests`.*

- [x] **T006** — `src/records.rs`: the first-clear line. In `view_body`'s
  `RecordsView::Campaign` arm build the fixed 5th row as `Campaign completions:
  N` plus `  ·  first clear in M matches` when
  `stats.first_clear_matches()` is `Some(M)` (plan §Design 6) — still exactly
  one pushed line. Tests: extend `campaign_has_completions_line_others_do_not`
  to cover both shapes (unset → no `first clear`; set → the exact folded line)
  or add `the_campaign_view_folds_in_the_first_clear_record`; the existing
  `by_opponent_table_is_anchored_across_breakdown_views` must pass unchanged.
  (Copies the arm's existing `lines.push(format!(…))` and the neighbouring
  view-body tests.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the anchored-table test passing unchanged; `git diff --stat`
  shows only `src/records.rs`. **PAUSE for the person — advisory** (the person's
  2026-09-15 ruling: run straight through; drive this and report it rather than
  stopping, unless something is wrong) (profile/saves backed up
  and checksum-restored; driven on a scratch profile): finish a run — the
  acknowledgement of the completing win lands on the map with the victory
  notice, showing this run's matches, credits won and lost, best streak and
  worlds; map keys do nothing under it; Enter/Space/Esc dismiss and the settled
  banner is still on the header; `★ Campaign complete` stays on the header
  afterwards; a rematch win raises no notice; Records → Campaign shows the
  first-clear line; then drive a profile broke and check the run-over notice
  carries the same summary between its title and its reset note, with Esc still
  ignored. Snapshots at 139×31.*

## Phase 3 — What you keep

<!-- The campaign-entry panel (T007), then the deal and its text (T008), then
the docs (T009). Phase ends with a review and a driver walkthrough; the
person's own attestation comes at the end of the spec under the 2026-09-15
run-straight-through ruling. -->

- [x] **T007** — `src/app.rs`: New Campaign, Reset Everything, and the
  three-choice panel. Add `enum CampaignChoice` (with `ALL`, `label`, wrapping
  `step`) and `enum ResetScope`; change `Modal::CampaignEntry { on_new }` to
  `{ choice: CampaignChoice }` and `Modal::ConfirmNewCampaign { on_yes }` to
  `Modal::ConfirmReset { on_yes, scope }` (rename the handler to
  `handle_confirm_reset_input`, keeping the `confirm_choice` guard exactly as
  it is); rewrite `handle_campaign_entry_input` to step the highlight and commit
  per plan §Design 4 (Continue → `enter_campaign_continue`, New Campaign →
  `ConfirmReset { MapOnly }`, Reset Everything → `ConfirmReset { Everything }`,
  Esc closes); gate `activate_menu_item`'s `StartCampaign` on
  `profile.differs_from_starter()`; split `reset_run`'s tail into
  `discard_match_and_banner`, add `reset_map_only`, and replace
  `start_new_campaign` with `start_fresh_campaign(scope)` (reset, `Sfx::MenuSelect`,
  `enter_campaign_map(true)`). Generalize `draw_two_choice` into
  `draw_choice_panel(frame, title, note, labels, selected, hint, pulse)` with
  the pure `choice_row_width(labels)` and the pure
  `choice_rows(note_present) -> (note_row, choice_row, hint_row, height)` in
  plan tension §4 — note absent `(1, 2, 4, 5)`, note present `(1, 3, 5, 6)`, so
  the acted-on choice row always has a blank row above and below it. This also
  changes the spec-021 discard-a-save confirm, which passes the same note;
  `draw_campaign_entry` passes the three labels,
  `draw_confirm_reset` passes the scope's title (`"New campaign? Resets the map;
  you keep your cards and credits."` / `"Reset everything? Erases progress,
  credits & cards."`) with `stake_forfeit_note()` unchanged. Tests:
  `campaign_choice_steps_and_wraps_in_both_directions`,
  `the_campaign_entry_panel_fits_the_minimum_terminal` and
  `a_choice_panel_keeps_a_blank_row_around_the_choice_row` (plan §Tests bullets
  15–17); the existing `confirm_choice_commits_only_on_enter_with_yes` stays
  unchanged. (Copies `draw_two_choice` itself, `handle_confirm_new_campaign_input`,
  and `back_destination`'s pure-enum + test shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the three new tests passing and the spec-014 confirm test
  unchanged; `git diff --stat` shows only `src/app.rs`; the implementer's report
  quotes `handle_campaign_entry_input`'s commit arm and both reset fns.*

- [x] **T008** — `src/app.rs` + `src/opponent_select.rs`: Quick Play deals the
  built deck. Delete `player_deck_for` and its test
  (`quick_play_deals_the_standard_deck_and_campaign_deals_the_built_one`), drop
  the `DEFAULT_SIDE_DECK` import, and make `start_match` deal
  `self.profile.deck().to_vec()` for both modes with a comment citing spec 024
  (plan tension §5); update `start_match`'s doc — the deck-valid precondition
  now binds Quick Play too, upheld by `open_opponent_select`'s existing divert,
  which is no longer "only a consistency nudge". Then, per plan §Design 7, set
  `QUICK_PLAY_NOTE =
  "Quick Play deals your deck. Nothing is staked."` and update its assertion in
  `the_full_roster_and_footer_fit_the_minimum_terminal`; the rows and the
  footer reserve are unchanged. Then the **comment-only** corrections the
  amended acceptance criterion allows (plan §Design 7): `src/card.rs:106-108`
  (`DEFAULT_SIDE_DECK`'s doc), `:147-148` (`deal_hand`'s doc) and `:433` (a test
  comment), plus `src/profile.rs:826-827` ("the standard pool … and Quick
  Play's deck") — each keeps the standard deck's remaining role, the
  opponents' baseline, and drops the claim that Quick Play deals it.
  `src/profile.rs:13` only names `DEFAULT_SIDE_DECK` "the standard deck", which
  stays true — leave it or reword it, either passes. No code line in `card.rs`
  or `profile.rs` changes. (Copies nothing new.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `grep -n "DEFAULT_SIDE_DECK\|player_deck_for" src/app.rs` is empty;
  `git diff -- src/card.rs` and `git diff -- src/profile.rs` each show only
  comment / doc-comment lines; `git diff --stat` shows only `src/app.rs`,
  `src/opponent_select.rs`, `src/card.rs` and `src/profile.rs`.*

- [x] **T009** — `README.md`, `docs/economy.md` + `docs/balance.md`: the texts
  that described the old rules (plan §Design 7). In the README: line ~71 ("the
  10 cards your hand is dealt from each **campaign** match" — now every match);
  lines ~82–83 (Start Campaign asks once there is anything to affect — cleared
  progress *or* a pool that differs from the starter — and offers three
  choices); lines ~85–88 (New Campaign resets the map while your cards and
  credits stay, Reset Everything is the full wipe, and Quick Play deals the deck
  you built with nothing staked — dropping "deals you the **standard** side
  deck — campaign matches deal the one you built", which is line-broken in the
  source). In `docs/economy.md:69-70`, the same Quick Play correction.
  In `docs/balance.md` add the short **### Replays** subsection after *### The
  economy bounds* (plan §Design 7): the measured curve describes a starter-deck
  run, a replay since spec 024 keeps the pool and so starts premium and easier,
  opt-in and deliberately not retuned — no constant moved. (Copies the
  neighbouring prose voice in each file.)
  *Verify: `cargo build --all-targets` + `cargo test -q` green verbatim
  (docs-only, but the constitution's command still runs); `grep -rn "standard
  deck\|standard pool\|\*\*standard\*\*" README.md docs` is read, not counted —
  `docs/opponents.md`'s baseline lines legitimately remain — and no hit claims
  Quick Play deals it; `git diff --stat` shows only `README.md`,
  `docs/economy.md` and `docs/balance.md`. **PAUSE for the person — advisory**
  (the person's 2026-09-15 ruling: run straight through; drive this and report
  it rather than stopping, unless something is wrong) (profile/saves backed up
  and checksum-restored): on a profile with a built pool, Start Campaign shows
  Continue / New Campaign / Reset Everything; New Campaign (confirmed) clears
  the map and keeps cards, credits and records; its No and Esc change nothing;
  the stake note appears when a match is in flight; Reset Everything wipes to
  the starter; a fresh profile still opens the map directly; a New Campaign on a
  balance below the cheapest ante meets the run-over notice; a Quick Play match
  deals a deliberately non-standard built deck; the opponent-select line reads
  right with the full roster at 139×31.*

## Final phase — Spec close-out

- [x] **T010** — Close-out. Draft
  `specs/024-endgame-victory/closeout-main-docs.md` (plan §Design 8, in 021–023's
  shape): **ROADMAP** — the endgame/victory item and the "run summary on the
  run-over notice" backlog item shipped, **plus inline "superseded by spec 024,
  which …" annotations** (the convention at ROADMAP lines 95–98) on lines
  173–175, 284–285 and 489, which still describe the old New Campaign and the
  old Quick Play deck in the present tense; **DECISIONS** — spec 024's six
  resolved decisions, the explicit reversal of spec 022's "Quick Play deals the
  standard (premium) deck" and spec 014's "New Campaign = full fresh start"
  (each quoted as superseded, with the reason), and plan tensions §1 (including
  why settlement's signature stayed put and the edge is evaluated twice), §2,
  §4, §5, §7 — to apply on `main` after the merge, never on the branch. Run
  `cargo test -q` **three consecutive times** and paste the tails. Mechanical
  checks: `git diff main --stat` shows no `src/game.rs`, `src/player.rs`,
  `src/save.rs`, `src/economy.rs`, `src/wager.rs`, `tests/balance.rs`,
  `Cargo.toml`, `Cargo.lock`, and `git diff main -- src/card.rs` contains only
  comment lines (the amended acceptance criterion); `grep -n "PROFILE_VERSION:
  u32" src/profile.rs` and the `SAVE_VERSION` line both read 1; `cargo build
  --all-targets` warning count equals `main`'s; `grep -rn "standard
  deck\|standard pool\|\*\*standard\*\*" src README.md docs` is a read-and-judge
  check (the opponent-baseline hits in `docs/opponents.md` and `src/profile.rs:13`
  stay): no hit says Quick Play is dealt one; `grep -rn "Erases progress" src` appears only
  under the Reset Everything title. Check off `spec.md`'s acceptance criteria
  with evidence (the verbatim lines). Request the pre-merge whole-spec sweep;
  apply `closeout-main-docs.md` on `main` after the merge.
  *Verify: three green tails, zero failures; every mechanical check listed with
  its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/024-endgame-victory/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product
owner**. Dispatch each task to the `sdd-implementer` (opus, high) — **experiment
2 is paused for this spec**, so no dispatch goes to `sdd-implementer-fable`
unless the person says the Fable allowance has reset. Verify by running the
build and tests yourself, then commit. **T002 is marked `review: per-task`**
(the settlement seam everything downstream inherits); everything else is
reviewed at the end of its phase (Phase 1 after T003, Phase 2 after T006,
Phase 3 after T009) with a shell-assembled bundle — the phase diff, the task
lines, the plan sections they cite, the acceptance criteria — one review plus
at most one re-review. **The person ruled on 2026-09-15 that implementation
runs straight through the whole spec in the spec session, without stopping at
the phase pauses**, unless something needs their attention — so the "PAUSE for
the person" lines in T006 and T009 are **advisory**: they say what to drive and
what to show in the phase report, not where to stop. Phase reviews by the
`skeptical-reviewer` still happen at every phase end, and anything unexpected
that bears on spec adherence still goes to the person immediately. The driver
walkthroughs themselves still happen (the orchestrator drives them and reports
what it saw): after Phase 2, the victory notice and the run-over summary on a
scratch profile; after Phase 3, New Campaign / Reset Everything and the Quick
Play deal.
Back up + checksum-restore the real profile/saves before every driver session —
this spec's walkthroughs include a full-run completion, a run-over reset and
two kinds of New Campaign. Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`) change
only via `closeout-main-docs.md` on `main` after the merge; `README.md` and
`docs/balance.md` ride in on the branch (T009).

Model & effort: the session runs at the session tier (`claude-fable-5-1`,
medium) per `.claude/settings.json`; the planner and the sign-off ran at
`opus` with the top-tier override dropped (the Fable allowance is low — the
constitution's Fallback clause). Implementation runs in the `sdd-implementer`
at `opus`, one task per dispatch; the per-task review, the per-phase reviews
and the sweep run at `opus`. A task that isn't routine goes to a decision
review, never resolved by the session; a product question `spec.md` doesn't
settle goes to the person. Per Erik's standing ruling, never infer the Fable
budget from a successful dispatch — ask if unsure, and log which tier actually
ran. Every session-ending pause ends with a continuation prompt (spec
directory, files to read, where to resume, involvement level, pause cadence,
any model switch) in its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo, and why). Compare the spec total against specs 021–023 before treating the
policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| **Experiment 1, spec 4** — session `claude-fable-5-1`, at **high** for the spec conversation (person raised it, 2026-09-15) and — by the person's ruling the same day — implementation runs on in the same session without phase pauses, still at high unless the person lowers it. Fable allowance at spec start: 20% left (person's reading 2026-09-15, after the spec 023 merge). | — | — | header |
| **Experiment 2 still paused** (person's ruling 2026-09-14, carried into this spec): the Fable allowance had not reset when planning began, so every implementer dispatch goes to `sdd-implementer` (opus, high) and the planner / sign-off ran at `opus` with the override dropped. Experiment 2 resumes with the Fable implementer on the first spec that starts after the reset, not mid-spec. | — | — | header |
| Planning: draft (sdd-planner) | opus (override dropped, experiment 2 paused) | ~203K (measured return; planner's own count ~205K in / 13K out) | drafted; 10 tasks in 4 phases, T002 the one per-task review; no product fork |
| Planning: sign-off fixes (sdd-planner, same context) | opus | ~39K (measured delta; planner's own count ~65K in / 9K out) | B1 (T001 updates its own caller) and B2 (option b: resolve_match owns the edge and counters, settle_campaign_match's signature untouched) applied; S2.1–S2.8 applied; pause ruling written into the handoff |
| plan + tasks sign-off (skeptical-reviewer) | opus (override dropped) | ~153K (measured return; reviewer's own estimate ~68K) | 2 blocking (T001 and T002 could not build green as scoped) + 8 notes; S2.1 exposed a spec-internal conflict (goal 5 vs the card.rs freeze) — orchestrator amended spec.md (three hunks) |
| sign-off re-review (skeptical-reviewer) | opus | ~32K (measured delta; reviewer's own estimate ~23K) | B1, B2 fixed; new B3: the spec.md amendments need the person's ratification — amendment note added under the spec's Status, ratification requested, Phase 1 dispatched meanwhile (nothing in it depends on the amended lines); N1 (test rename), N2 (profile.rs comment-only check), N3 (grep is read-and-judge), N5 (settle_campaign_match doc line) applied by the orchestrator |
| T001 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~62K (measured return) | done first try; `run_summary_lines` signature on one line (rustfmt-wrapping avoided); note for T002: the "completions > 0, no record" case needs a `LifetimeStats` JSON round-trip, `first_clear_matches` has no setter |
| T002 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~99K (measured return) | done first try; resolve_match body verbatim from plan §Design 3; settle_campaign_match untouched; orchestrator re-ran verification: 397 unit + 6 integration passing, diff = campaign.rs + profile.rs |
| T002 per-task review (skeptical-reviewer) | opus | ~65K (measured return; reviewer's own count ~46K in / 3K out) | signed off, 0 blocking, 5 notes — N1 run_complete could be `worlds_cleared() == PLANETS.len()` (simplicity; carry to the Phase 1 review or T003), N2 differs_from_starter is order-sensitive on the deck (safe direction; know it before the Phase 3 driver), N3 the default() case of the entry-panel test is tautological (reviewer verified Profile::default() is the starter), N4 the staked case is pinned via credits not the pointer, N5 the Mode derivation is duplicated until T003 — T003's `Mode::` grep must actually run |
| T003 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~46K (measured return) | done first try; grep for the three identifiers in app.rs empty; stale settle_campaign_match doc paragraph corrected (comment-only); `stats::Mode` import already gone (T008 removes only DEFAULT_SIDE_DECK); T002 review N1 (run_complete via worlds_cleared) not applied — outside the diff allowlist, carried to the Phase 1 review |
| Phase 1 review (skeptical-reviewer) | opus | ~78K (measured return; reviewer's own count ~75K) | signed off, 0 blocking, 5 notes — N1 the net-gain formula lives in profile.rs and campaign_map.rs with nothing pinning them equal (economy.rs frozen, so a test not a refactor; fold into T004/T005 if free, else sweep), N2 the fresh-profile entry-panel case is tautological — `Profile::from_json(r#"{\"version\":1}"#).differs_from_starter() == false` is the assertion that would keep it sound (T007), N3 `run_counters_default_zero_round_trip_and_accumulate` never round-trips (name overstates; sweep), N4 run_complete refactor: leave it (no record change), N5 order-sensitive deck comparison stands |
| T004 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~77K (measured return) | done first try; `draw_notice` replaces `draw_run_over` and serves both notices; victory 73×14 and run-over 78×13 boxes, unclamped at 139×31 (fit test); `victory_due` set with `|=`, taken at map entry; Phase 1 N1 (net-gain pin) not applicable in app.rs — `banner_line` is private to campaign_map.rs — carried to T005 |
| T005 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~52K (measured return) | done first try; pure `axis_line(run_complete)`; Phase 1 N1 closed — `the_banner_and_the_run_tally_report_the_same_net_gain` pins the duplicated net-gain formula equal |
| T006 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~51K (measured return) | done first try; one push keeps the 5th-row anchor; note: the widened Campaign line widens the Records box in every view (~55 chars, well inside 139) — eyeball at the driver |
| Phase 2 review (skeptical-reviewer) | opus | ~80K (measured return; reviewer's own count ~52K in / 2K out) | signed off, 0 blocking, 4 notes — N1 victory_due lifecycle verified by reading (acknowledgement is the only exit from a campaign GameOver; `|=` + mem::take), driver-attested, carry to the sweep; N2 every non-dismiss key swallowed under the notice, except the global mute `m` (pre-existing for every overlay); N3 `App::run_tally()`/`worlds_cleared()` accessors are indirection the plan didn't ask for (minor; sweep decides); N4 T006 box-widening note closed (plan §Design 6 accepts it; 131-col clamp) |
| Driver tooling fix (orchestrator) | — | — | play.py matched "You Tied" but the board says "You tied!" — the play loop stalled on a tied round; one-line case fix in `.claude/skills/run-kaazap/play.py` |
| Phase 2 driver walkthrough (orchestrator, scratch profiles, 139×31; real profile/settings/save moved aside, checksummed, restore due at Phase 3 end) | — | — | Eight stand-at-18 attempts against the Sovereign all lost (the driver plays no side cards), so the completing win was driven against Greeb on a profile with the other seven worlds cleared — the same completion edge. Acknowledging the win landed on the map with the victory notice: spec text verbatim, `Matches played 13 · won 10 · lost 3` / `Credits won 430 · lost 150` / `Best streak 5 · Worlds cleared 8/8`, `Enter continue` with a blank row above; ↓ and b swallowed (no Outfitter); Enter dismissed with `★ Won 10 credits` still on the header; after ↑ the header read `★ Campaign complete`, `8/8 cleared ◈ 610`; profile.json: completions 1, first_clear_matches 13, credits_won 430. A rematch (lost) raised no notice: `Lost 10 credits`, marker intact. Broke profile (8 credits, 2/8): the run-over notice carried `Matches played 9 · won 3 · lost 6` / `Credits won 80 · lost 130` / `Best streak 2 · Worlds cleared 2/8` between its title and reset note; Esc ignored; Enter reset. Records Campaign view not reached (menu navigation landed in the deck builder) — checked at the Phase 3 walkthrough on a seeded profile. |
| T007 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~87K (measured return; implementer's own estimate ~60K) | done first try; `draw_choice_panel` takes N labels; `CHOICE_GAP` const named (the literal 6 was inline twice); Phase 1 N2 assertion not added — it belongs in profile.rs, outside T007's allowlist — handed to T008, whose allowlist includes profile.rs |
| T008 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~71K (measured return, two passes; implementer's own count ~75K in / 8K out) | own work done first try; **Verify deviation:** the profile.rs diff carries one test hunk (the ruled assertion) besides the comment-only hunk; stopped on the carried Phase 1 N2 assertion — it is false: a document without a `credits` key loads with 0 (spec 021's deliberate serde default) and so differs from the starter. Orchestrator ruled option 1 (the spec's own predicate: such a document shows the panel; assert that a serialized fresh profile is the starter) — recorded as plan §Open questions 5; second pass applied it, test-only. Plan's profile.rs line refs were stale (:826 → :1176) |
| T009 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~51K (measured return; implementer's own estimate ~34K) | done first try; the README is tracked as `Readme.md` (case-insensitive FS hides it from `git diff -- README.md`) — T010's sweep must spell it so; `docs/economy.md`'s "Settling — one seam, exactly once" section still describes settle-then-record and is outside T009's footprint → T010 |
| Phase 3 driver walkthrough (orchestrator, scratch profiles, 139×31) | — | — | Quick Play on a ten-card +1/−1 deck: opponent select reads `Quick Play deals your deck. Nothing is staked.`, the dealt hand was `-1 +1 +1 -1` (built deck, not the standard). Completed-run profile: Start Campaign showed `▸ Continue  New Campaign  Reset Everything`; New Campaign → `New campaign? Resets the map; you keep your cards and credits.` default No; Esc left both panels with the profile unchanged; Yes → `0/8 cleared ◈ 600`, axis label back, deck builder still holding ±1 ±2 ±3 ±6 2&4 3&6 ±1T, `beaten` empty and run tally zeroed in profile.json; Reset Everything → `Reset everything? Erases progress, credits & cards.` default No; Yes → `◈ 50`, starter deck. Records → Campaign (3/4): `Campaign completions: 1  ·  first clear in 13 matches`. Poor completed profile (8 credits): New Campaign → Yes landed on the run-over notice; Enter → 50 credits, starter deck. Real profile/settings/save restored afterwards, checksums verified (next row). |
| Real data restored (orchestrator) | — | — | profile.json, settings.json and saves/savegame.json copied back from the scratchpad backup; SHA-256 of all three match the pre-driver checksums (81bb34d1…, 6fbd0816…, afe07a8c…) |
| Phase 3 review (skeptical-reviewer) | opus | ~91K (measured return; reviewer's own estimate ~95K in / 4K out) | signed off, 0 blocking, 8 notes → T010: N1 `confirm_choice` doc + test comment still call New Campaign the wipe; N2 `reset_to_starter` doc contradicts `reset_campaign_run`'s; N3 `has_progress` doc names a gate it no longer is; N4 docs/economy.md "Settling — one seam" is wrong on order, caller and API (confirmed, wider than T009 said); N5 the T008 profile.rs hunk is test code not comment-only (authorized by the ruling; note the Verify deviation) and the ruled statement — a document with no credits key differs from the starter — should be asserted, not only commented; N6 the entry-panel fit test re-derives the width max and measures the confirm's hint (fold in if app.rs is touched); N7 balance.md:218 overstates ("already answered for the cards you kept"); N8 Phase 1 N3 / Phase 2 N3 stay in the tier log, not T010 |
| T010 close-out (sdd-implementer) | opus (fallback, experiment 2 paused) | ~115K (measured return) | done first try; six of the seven review notes applied (N7 on balance.md missed — sweep N1; comment/doc/test-only; `choice_panel_width` extracted for the fit test); closeout-main-docs.md drafted; all 13 acceptance criteria checked with evidence; three green test runs; mechanical checks clean (card.rs comment-only; versions 1; warnings 0 = main's 0; standard-deck grep: four legitimate hits); found the spec 023 Quick Play sentence twice in ROADMAP (~329 and 489) — both annotated in the close-out; orchestrator re-ran verification: 403 unit + 6 integration passing |
| Pre-merge sweep (skeptical-reviewer) | opus | ~178K (measured return; reviewer's own estimate ~120K) | 2 blocking, both text-only — B1 the close-out's supersession list misses four present-tense assertions (ROADMAP 87-88, 95-97; DECISIONS 854-858, 865-866) and its §1d anchor for the ~329 occurrence does not exist as quoted; B2 docs/economy.md:143-146 still says the run-over reset runs `start_new_campaign` "exactly as for New Campaign" (now backwards). Notes: N1 Phase 3 N7 not actually applied (T010 row corrected: six of seven); N2 New Campaign on a broke balance shows an all-zero summary on the run-over notice — spec-conformant, shown to the person at attestation; N3/N4/N5 the carried Phase 1 N3, Phase 2 N3 and Phase 2 N1 closed (rename when stats.rs is next open; accessors a wash; lifecycle holds by construction); N6 "±1 deck" → "+1/−1 deck"; N8 three optional stale spots; N9 play.py fix and the T008 test hunk properly accounted for; N10 spec ratification still with the person; N11 §1b line cite and a superseded tail clause. Fixes dispatched to the T010 implementer; re-review next |
| Sweep fixes (sdd-implementer, T010's context) | opus | ~25K (measured delta; implementer's own count ~40K) | B1, B2, N1, N6, N11 and all three optional N8 items applied, text-only; every close-out replace anchor checked programmatically against the live ROADMAP.md / DECISIONS.md — 14 of 14 match verbatim and are unique |
