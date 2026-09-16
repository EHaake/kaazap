# Tasks: First-run onboarding & controls refinement — spec 023

> **Status**: Signed off (skeptical-reviewer at fable, 2026-09-13)
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
implementer returns, and only the orchestrator commits. Every task below runs in
the `sdd-implementer` at the implementation tier.

**Foundational phase:** Phase 1 (T001–T005 — the profile marks, the engine key
map, the in-match key table, and every hint that describes it) is foundational:
Phase 2's popup text and How to Play describe the controls Phase 1 ships, and
its dismissal writes the marks T001 adds. Under the constitution's review
cadence the default is a **per-phase** `skeptical-reviewer` pass, and **no task
carries `review: per-task`**: nothing here is copied by a dozen later files —
T007, the largest task, is the last code task of its phase, so the phase review
is its review. Every task is reviewed at phase end.

---

## Phase 1 — Controls refinement and the profile marks (foundational)

<!-- Foundational: the data model (two marks) and the controls every later text
describes. Order inside the phase follows dependency: engine key map before the
app's key table before the hints. Phase ends with a pause: the person plays a
match with the new keys (no onboarding pieces yet). -->

- [x] **T001 (foundational)** — `src/profile.rs`: the seen marks. Add
  `#[serde(default)] primer_seen: bool` and `#[serde(default)]
  first_match_seen: bool` to `Profile` with the doc comment in plan §Design 1;
  accessors `primer_seen()`, `mark_primer_seen()`, `first_match_seen()`,
  `mark_first_match_seen()` (doc: callers pair a mark with `save`, like deck
  edits); `Default` leaves both false; `reset_to_starter` takes both alongside
  `stats` and restores them after the `Profile::default()` swap; the struct
  doc's field list mentions them. `PROFILE_VERSION` stays 1. Tests: add
  `onboarding_marks_default_unset_round_trip_and_load_unset_from_older_documents`
  (plan §Tests bullet 1 — `Default`, the older JSON, mark → JSON →
  `from_json`, `PROFILE_VERSION == 1`) and extend
  `reset_to_starter_wipes_the_run_but_preserves_lifetime_stats` to mark both
  before the reset and assert both survive (rename to `…preserves_lifetime_stats_and_onboarding_marks`).
  (Copies the `stats` field + `reset_to_starter`'s take/restore shape and the
  `credits_seed_a_fresh_profile_but_default_to_zero_for_older_profiles` test
  shape in the same file.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  (reported verbatim) with the two tests named passing; `git diff --stat`
  shows only `src/profile.rs`.*

- [x] **T002 (foundational)** — `src/game.rs`: the key map and the pause
  helper. In `game_action_from_key` delete the `AwaitingSignChoice` branch
  (h/l/+/−/1/2/c) and the `'1' | '2' | '3' | '4' => PlayHand` arm; make `' '`
  map to `Hit` in `PlayerTurn` (plan §Design 2, exact match block) and keep
  `NextRound` / `NextGame` at the pauses; update the arm's comment (Space
  draws; the cursor model in `app.rs` owns select/play). Add `pub fn
  restart_opponent_pause(&mut self)` (plan §Design 2 doc; re-arms `until =
  Instant::now() + OPPONENT_THINKING_TIME_MS` iff the phase is
  `OpponentThinking`). `GameAction` (incl. `CancelSignChoice`),
  `apply_game_action`, `play_card`, `commit_sign_choice`, the AI, and every
  `sign_*` action test stay untouched. Tests: rewrite
  `space_advances_at_pauses_and_never_draws` as
  `space_draws_on_the_players_turn_and_advances_at_the_pauses` (plan §Tests
  bullet 2, including `None` in `OpponentThinking` and `AwaitingSignChoice`);
  add `space_over_twenty_accepts_the_bust_like_d` (the `over_20`-style board:
  apply the action from `' '`, assert the same `stood`/`bust` outcome as from
  `'d'`); rewrite `sign_phase_maps_only_choice_and_cancel_keys` as
  `sign_phase_maps_no_keys` (every key in `h l + - 1 2 c d s ' '` → `None`);
  rewrite `sign_normal_phase_key_mapping_unchanged` as
  `number_and_sign_keys_map_to_nothing_on_the_players_turn` (`'1'..'4'`,
  `h`, `l`, `+`, `-`, `c` → `None`; `d` → `Hit`, `s` → `Stand`); add
  `cancel_sign_choice_outside_the_phase_is_a_noop` and
  `restart_opponent_pause_re_arms_only_a_running_pause` (plan §Tests bullets 3–4).
  `tests/balance.rs` and the two headless loops are not edited. (Copies the
  existing `' '` match arm and the `sign_cancel_restores_turn_with_card_unspent`
  test shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim — the six tests named pass, every `sign_choose_*` /
  `sign_play_*` / `sign_cancel_*` / `sign_update_*` test and every
  `tests/balance.rs` test pass unchanged; `git diff --stat` shows only
  `src/game.rs`; `git diff -- src/game.rs` has no hunk outside
  `game_action_from_key`, the new fn, and `mod tests`.*

- [x] **T003 (foundational)** — `src/app.rs`: the in-match key table, cursor
  select, and the Continue guard. Add `enum TurnKey` + `fn turn_key(key,
  player_turn)` exactly as plan §Design 3 (doc included); add
  `HandCursor::select(index, hand)` (occupied slot → index set and the
  pending sign reset, exactly what `move_left` / `move_right` do after
  landing, the current slot included; empty / out-of-range → nothing, sign
  included — plan §Design 3); rewrite the
  `Screen::InGame` arm of `handle_key` as `match turn_key(key, player_turn)`
  (plan §Design 3 wiring: `Menu` → `start_menu()`, cursor variants → the
  cursor with no save, `Play` → `cursor_confirm` + `game_changed`, `Engine(c)`
  → the existing `handle_game_input` block, `Ignore` → nothing); drop the
  Space-mirrors-Enter comment and update the arm's and `HandCursor`'s docs
  (no "direct number-key + h/l play path"); in `activate_menu_item`'s
  `Continue` arm apply `GameAction::CancelSignChoice` to the loaded game
  before the cursor normalize, with the plan tension §5 comment. Tests: add
  `turn_key_binds_the_spec_023_keys` (plan §Tests bullet 5),
  `cursor_select_lands_on_an_occupied_slot_and_resets_the_sign_like_a_move`
  and `cursor_select_ignores_an_empty_or_out_of_range_slot` (bullet 6 —
  including the same-slot case and the equality with `move_right`'s state); the
  existing `cursor_confirm_*` tests stand unchanged. (Copies `confirm_choice`
  / `ConfirmChoice` and its test, and `HandCursor::move_right` + its tests.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the three new tests passing and all `cursor_confirm_*` tests
  unchanged; `git diff --stat` shows only `src/app.rs`.*

- [x] **T004** — `src/board.rs`: the turn hints. Delete the
  `AwaitingSignChoice` branch of `status_message` (the phase now falls to
  `_ => None`) and its "number-key path" doc; rewrite `play_prompt_line` and
  `over_twenty_alert` to the exact strings in plan tension §7. Tests: rewrite
  `status_player_turn_shows_nav_and_play_prompt_strong`,
  `status_selected_sign_card_shows_pending_sign_and_flip_hint`,
  `status_selected_fixed_card_shows_play_with_no_flip_hint`, and
  `status_over_twenty_does_not_replace_the_base_prompt` to the new strings
  (exact `assert_eq!`, ± and tiebreaker and flip and fixed covered); replace
  `status_sign_prompt_shows_the_cards_magnitude` with
  `status_never_shows_a_sign_prompt` (`AwaitingSignChoice` → `None`); assert
  the alert text in `status_over_twenty_alert_is_its_own_line`; add
  `turn_hints_fit_the_status_band` (each of the four prompt shapes, widest
  label, `chars().count() <= BoardLayout::new(min config).status.width()`).
  (Copies the existing `status_*` test shape and `popup_rect_*`'s use of
  `bv.layout`.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `grep -n "(h)\|h/l\|number-key" src/board.rs` is empty; `git
  diff --stat` shows only `src/board.rs`.*

- [x] **T005** — `assets/game_overlay_text.txt`, `assets/how_to_play_text.txt`,
  `src/overlay.rs` (tests only): the help texts. Edit the five control rows
  and the last line of the game overlay exactly as plan §Design 4; in How to
  Play replace the last line with the spec's campaign section and controls
  line (plan §Design 4 order — campaign block, blank, the two controls lines
  ending `? closes.`); leave "press d/s to bust" (plan §Open questions 3).
  Tests in `overlay.rs`: `help_texts_name_the_new_keys_and_nothing_old` (plan
  §Tests bullet 9 — every positive and negative line check) and
  `help_texts_fit_the_minimum_terminal_unclamped` (both texts via
  `Overlay::new(kind, min).read_text_from_file()` — or `overlay_text` once T006
  lands; `OverlayLayout` height `== lines + V_PAD`, width `== w + 2·H_PAD`).
  (Copies `wager.rs`'s `the_prompt_fits_the_minimum_terminal_unclamped`.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with both tests passing; `grep -n "Space" assets/*.txt` shows no
  line pairing Space with play; `git diff --stat` shows only the two asset
  files and `src/overlay.rs`. **PAUSE for the person** (profile/saves backed
  up and checksum-restored): a Quick Play match with the new keys — 1–4
  select and never play, Enter and P play at the shown sign (flip with ↑/↓),
  Space draws and over 20 accepts the bust, no + or − prompt ever appears, `?`
  and the turn hint read right; Space still advances at round end / game
  over and confirms in menus.*

## Phase 2 — The onboarding pieces and the Quick Play line

<!-- The overlay texts first (T006), then the two modals and their wiring
(T007), then the opponent-select line (T008). Phase ends with the spec's play
attestation on a fresh profile and an existing one. -->

- [x] **T006** — `src/overlay.rs`, `assets/primer_text.txt` (new),
  `assets/first_match_text.txt` (new): the texts. Add `OverlayKind::Primer`
  and `OverlayKind::FirstMatch`; turn `read_text_from_file` into `pub fn
  overlay_text(kind: OverlayKind) -> Vec<String>` (five `include_str!` arms;
  `Overlay::draw` calls it; the T005 test switches to it). Write both asset
  files with the spec's text **verbatim** (§The primer / §The first-match
  popup code blocks, including the leading spaces on the `Enter to …` lines
  and the single blank line above them; no trailing blank line). Tests:
  `onboarding_texts_are_the_spec_text_and_fit` (10 / 12 lines; line 0 is the
  spec'd title; last line trimmed is `Enter to continue` / `Enter to begin`;
  `OverlayLayout` at 139×31 unclamped for both) and
  `onboarding_texts_breathe_only_around_the_dismiss_line` (the line above the
  last is empty; no two consecutive empty lines; the last line is not empty —
  the box's own padding row is the row below). (Copies the `GameHelp`
  `include_str!` arm and `overlay_measure_*` test shape.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with both tests passing; the implementer pastes `cat -A` of both
  asset files and the orchestrator compares them line by line against the
  spec's two code blocks (same lines, same leading spaces, no trailing blank
  line); `git status` shows only `src/overlay.rs` and the two new assets
  (`--stat` can't list untracked files).*

- [x] **T007** — `src/app.rs`: the two modals. Add `Modal::Primer` and
  `Modal::FirstMatch` (unit-like, doc-commented like `RunOver`),
  `onboarding_dismissed(key)`, `map_entry_modal(broke, primer_due)`, and
  `handle_onboarding_input` exactly as plan §Design 3; give
  `enter_campaign_map` the `from_menu: bool` with four call sites —
  `enter_campaign_continue` → `true`, the `PendingStart::Campaign` confirm arm
  → `true`, `start_new_campaign` → `enter_campaign_map(true)` (replacing its
  `open_campaign_map()`; its doc comment drops "the map's own New Campaign
  panel" — New Campaign is menu-only), the game-over acknowledgement →
  `false`; `enter_campaign_map`'s doc updated (plan tension §3); the shop /
  builder Backs keep `open_campaign_map`; hook `start_match` (raise `FirstMatch` when the mark is unset, after
  the screen is set, before `save_game`); add the `held` guard around
  `update()` in `tick` (plan tension §2); add the two `draw` arms via
  `draw_text_overlay` + `overlay_text`; add the new branch to `handle_key`'s
  modal chain. Tests: `onboarding_dismissed_on_enter_space_or_esc_only`,
  `map_entry_modal_prefers_run_over_then_primer` (plan §Tests bullet 11),
  `the_first_match_popup_holds_the_match_and_swallows_play_keys` and
  `the_primer_swallows_map_keys` (bullets 12–13 — **non-dismiss keys only**,
  no phase change, so no disk write; plan tension §4). (Copies
  `run_over_acknowledged` + `handle_run_over_input` + the `RunOver` draw arm,
  the `PlayLog` no-resize draw pattern, and the `resize_too_small_*` App
  test's construction.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim with the four tests passing; `git diff --stat` shows only
  `src/app.rs`; the implementer's report quotes the `handle_onboarding_input`
  body and the four `enter_campaign_map` call sites with their `from_menu`
  values, and confirms `grep -n "open_campaign_map()" src/app.rs` shows no
  call site outside the shop Back, the builder Back, and
  `enter_campaign_map`'s own body (a doc-comment mention is fine).*

- [x] **T008** — `src/opponent_select.rs`: the Quick Play line. Add `const
  QUICK_PLAY_NOTE: &str = "Quick Play deals the standard deck.";`, draw it
  `Muted` at `y + 4`, move `HINT` to `y + 5`, footer reserve 6 → 7 in both
  `MenuLayout::new` calls and the `draw` comment. Tests: update
  `the_full_roster_and_footer_fit_the_minimum_terminal` (`hint_y = after_items
  + 5`, reserve 7, also assert the note row `after_items + 4 < rows`) and
  assert `QUICK_PLAY_NOTE`'s exact text there. (Copies the `HINT` const and
  the blurb/hint draw lines.)
  *Verify: `cargo build --all-targets` no new warnings; `cargo test -q` green
  verbatim; `git diff --stat` shows only `src/opponent_select.rs`. **PAUSE for
  the person** (profile/saves backed up and checksum-restored, then a fresh
  profile): Start Campaign → the primer once (Enter/Space/Esc dismiss; b, c,
  Enter-on-a-node do nothing while it is up); launch a match → the popup
  over the dealt board, the board frozen under it — then **quit with the
  popup up** (mark unset), relaunch, Continue → the match resumes with no
  popup; finish or abandon it, start the next new match **from opponent
  select or the map, not G at game over** (G restarts inside the engine and
  shows nothing) → the popup shows,
  dismiss it; after that a second match, a resumed match, a run-over reset,
  and New Campaign show neither piece; a pre-023-shaped profile with progress
  (primer mark unset) whose first act is New Campaign sees the primer on that
  map; the opponent-select line is on-frame with the full roster. Then the
  person's real profile: each piece once, cards/credits/records intact.*

- [x] **T006a** — Finding from the person's Phase 2 walkthrough (2026-09-14):
  "the bottom `Enter to continue` is over to the left side slightly, which
  makes it look off for both prompts." Restated: the dismiss line is the
  acted-on element (spec §Design requirements); the spec's code blocks give
  it 13 leading spaces, but `draw_text_overlay` left-aligns every line after
  the title inside a box sized to the widest line (52 columns for the primer,
  53 for the popup), so the line lands left of center. Fix: center the
  dismiss line within each text by its leading spaces in the two asset files
  — `(widest − len) / 2` — so `Enter to continue` (17 chars) gets 17 spaces
  and `Enter to begin` (14 chars) gets 19 (20 also acceptable; pick the one
  that centers visually — round down). Update the spec's two code blocks
  (`spec.md` §The primer / §The first-match popup) to the same leading
  spaces, since the person changed the text. No code change; the T006 tests
  compare the trimmed last line, so they stand. *Verify: constitution command
  green; `diff` of each asset against the spec's code block is empty; the
  dismiss line is centered within 1 column of the widest line's center.*

## Final phase — Spec close-out

- [x] **T009** — Close-out. Draft
  `specs/023-first-run-onboarding/closeout-main-docs.md` (plan §Design 7:
  ROADMAP — onboarding item shipped, lines 56–59 superseded; DECISIONS —
  rulings A–H; tension §1 (pass-through kept); tension §3 as amended — the
  primer is raised at every menu entry to the map, Start Campaign / Continue
  / discard-and-enter / a confirmed New Campaign, which is menu-only (no
  `MapOutcome` variant, `ConfirmNewCampaign` raised only from
  `CampaignEntry`), and never from the game-over acknowledgement or a shop /
  builder Back; tension §5; lines 40–41 superseded) in 021/022's
  shape, to apply on `main` after the merge — never on the branch. Run
  `cargo test -q` **three consecutive times** and paste the tails. Mechanical
  checks: `git diff main --stat` shows no `save.rs`, `player.rs`, `card.rs`,
  `economy.rs`, `wager.rs`, `campaign_map.rs`, `tests/balance.rs`,
  `Cargo.toml`, `Cargo.lock`; `git diff main -- src/game.rs` has hunks only in
  `game_action_from_key`, `restart_opponent_pause`, and `mod tests`;
  `PROFILE_VERSION == 1` and `SAVE_VERSION == 1`; `cargo build --all-targets`
  warning count equals `main`'s; `grep -rn "Space" assets src/board.rs
  src/app.rs` shows no line saying Space plays a card and `grep -rn "1-4\|1 2
  3 4" assets src/board.rs` none saying they play one. Check off `spec.md`
  acceptance criteria with evidence (the verbatim lines). Request the
  pre-merge whole-spec sweep; apply `closeout-main-docs.md` on `main` after
  the merge.
  *Verify: three green tails, zero failures; every mechanical check listed
  with its command and output; sweep clean or findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/023-first-run-onboarding/{spec,plan,tasks}.md`,
then implement from the first unchecked task. Involvement level is **product
owner**. Dispatch each task to the `sdd-implementer` per the model policy;
verify by running the build and tests yourself, then commit. **No task is
marked `review: per-task`**: review at the end of each phase (Phase 1 after
T005, Phase 2 after T008) with a shell-assembled bundle — the phase diff, the
task lines, the plan sections they cite, the acceptance criteria — one review
plus at most one re-review. Pause for the person after each phase and whenever
something unexpected bears on spec adherence. The **Phase 1 pause** is the
controls check (a Quick Play match with the new keys); the **Phase 2 pause**
is the onboarding attestation (a fresh profile, then the person's own). Back
up + checksum-restore the real profile/saves before every driver session —
this spec writes the profile on every dismissal and the person's walkthrough
includes a run-over reset and a New Campaign. Repo-wide docs (`ROADMAP.md`,
`DECISIONS.md`) change only via `closeout-main-docs.md` on `main` after the
merge.

Model & effort: the session runs at the session tier (`claude-fable-5-1`,
medium) per `.claude/settings.json`; the planner and the sign-off ran at the
top tier (`fable`) by per-call override. Implementation runs in the
`sdd-implementer` at `opus`, one task per dispatch; per-phase reviews and the
sweep run at `opus`. A task that isn't routine goes to a decision review at
the top tier, never resolved by the session; a product question `spec.md`
doesn't settle goes to the person. Per Erik's standing ruling, never infer the
fable budget from a successful dispatch — ask if unsure, and log which tier
actually ran. Every session-ending pause ends with a continuation prompt (spec
directory, files to read, where to resume, involvement level, pause cadence,
any model switch) in its own fenced block.

## Tier log (this spec, under the model policy)

<!-- Record the evidence: token usage from each subagent return (implementer runs
and reviewer invocations), any escape-hatch miss (a task the orchestrator had to
redo, and why). Compare the spec total against specs 021–022 before treating the
policy as settled. -->

| Task / invocation | Tier | Tokens | Outcome / miss reason |
|---|---|---|---|
| **Experiment 1, spec 3** — session `claude-fable-5-1` at medium throughout. Fable allowance at start: not recorded (the opening prompt's placeholder was left unfilled); after Phase 1 and the experiment 2 setup: 26% left (person's reading, 2026-09-14); at spec end: recorded on `main` after the merge, as spec 022 did (person's reading to be asked for, never inferred). | — | — | header |
| **Experiment 2** — declared 2026-09-13, `CLAUDE.md` amended 2026-09-14 (implementer `sdd-implementer-fable`, `claude-fable-5-1` at medium; `sdd-implementer` at opus the fallback). **No dispatch ever ran under it in this spec** — paused before T006 (next row). Fable allowance at the amendment: 26% left (person's reading, 2026-09-14). | — | — | header |
| **Experiment 2 paused** from T006 onward (person's ruling 2026-09-14): Fable allowance 74% used with three days to the reset, so the constitution's Fallback clause applies for the rest of the window — every task dispatched to `sdd-implementer` (opus, high) instead of `sdd-implementer-fable`; any planner or sign-off dispatch runs at the implementation tier with the override dropped; the session stays as it is. Each dispatch's resolved model is still logged per row. Experiment 2 resumes with the Fable implementer on the first spec that starts after the reset, not mid-spec. | — | — | header |
| Planning: draft + sign-off fixes (sdd-planner) | fable | ~205K draft + ~20K fixes (budget counter at return) | drafted; sign-off's 1 blocking (New Campaign is menu-only → `start_new_campaign` raises the primer via `enter_campaign_map(true)`) and 5 notes applied; two design edges flagged (plan §Open questions) |
| plan + tasks sign-off (skeptical-reviewer) | fable | ~70K (measured return) | 1 blocking (start_new_campaign never raised the primer after the spec's New Campaign amendment; reviewer verified New Campaign is menu-only) + 6 notes — B1, S1–S5 sent to the planner and applied; S6 (phantom map-side panel wording) fixed in spec.md by the orchestrator |
| sign-off re-review (skeptical-reviewer) | fable | ~17K (measured return) | signed off; 2 wording notes applied by the orchestrator (T007: the open_campaign_map grep also hits enter_campaign_map's body and a doc comment; T008: the post-Continue new match must be launched from opponent select or the map, not G) |
| T001 impl (sdd-implementer) | opus | ~41K (measured return) | done first try; note: the private `profile_with` test helper constructs `Profile` literally, so every additive field must be added there too |
| T002 impl (sdd-implementer) | opus | ~41K (measured return) | done first try; kept a two-line early return so `AwaitingSignChoice` answers `None` for every key (plan §Design 2's "d, s unchanged" would have left `d` live there, contradicting §Tests bullet 3) — plan snippet reconciled by the orchestrator; for the Phase 1 review |
| T003 impl (sdd-implementer) | opus | ~43K (measured return) | done first try; `Play` binds lowercase `p` only (matches the existing lowercase x/d/s convention); Continue arm binds `mut game` to apply the cancel |
| T004 impl (sdd-implementer) | opus | ~37K (measured return) | done first try; the ± hint line measures 64 chars against the 81-col band (at the plan's stated ceiling — re-measure if a key is ever added) |
| T005 impl (sdd-implementer) | opus | ~39K (measured return) | done first try; fit test also covers MenuHelp; How to Play is now 24 lines (28-row box) |
| Phase 1 review (skeptical-reviewer) | opus | ~65K (measured return) | signed off, 0 blocking, 5 notes — T002 early return confirmed correct (required by §Tests bullet 3; Esc/x still exits in every phase); Continue's cancel→normalize has no wiring test (plan-sanctioned, tension §4) → sweep; stale test name `status_player_turn_shows_nav_and_play_prompt_strong` (asserts the empty-hand shape) → T009 sweep; uppercase P inert like D/S/N/G (pre-existing convention); Ctrl-P safe (`resolve_key` maps it to Up first). Orchestrator verified the How to Play tail is the spec's text verbatim. |
| T006 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~42K (measured return) | done first try; asset files extracted from the bundle with sed, orchestrator diffed both against spec.md — verbatim; the T005 `text(kind)` test helper folded into direct `overlay_text` calls |
| T007 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~59K (measured return) | done first try; `enter_campaign_map` now assigns the modal unconditionally (all four callers have no modal open at the call); the onboarding branch sits after `RunOver` in the modal chain; open_campaign_map grep: doc mention, own body, builder Back, shop Back, one test |
| T008 impl (sdd-implementer) | opus (fallback, experiment 2 paused) | ~29K (measured return) | done first try; `QUICK_PLAY_NOTE` module-level so the test can assert its text; footer reserve 7 in both `MenuLayout::new` calls — headroom at 31 rows is now the binding constraint for this screen's footer |
| Phase 2 review (skeptical-reviewer) | opus | ~75K (measured return) | signed off, 0 blocking, 6 notes — modal assignment in `start_match`/`enter_campaign_map` is order-sensitive and untested (driver confirmed both the wager-commit and New Campaign paths raise their piece; carry to the sweep as "verified by driver + person, not by a test"); two inaccurate doc comments ("asset re-read on each draw" — it is `include_str!`) → T009; `the_primer_swallows_map_keys` asserts less than its comment claims → T009; opponent-select title/preview not pinned against row 0 (green at 31 rows) → sweep |
| Phase 2 driver walkthrough (orchestrator, scratch profile, 139×31; real profile/saves backed up and checksum-restored) | — | — | fresh profile: primer over the map (b swallowed; Enter dismissed; `primer_seen` true); popup over the dealt board (d and 1 swallowed); quit with the popup up → `first_match_seen` false → Continue resumed with no popup; next new match from the map → popup → dismissed → board live, mark set; Quick Play: no popup, "Quick Play deals the standard deck." on-frame with the full roster; New Campaign: marks kept, no primer; pre-023-shaped profile with progress whose first act is New Campaign → primer; broke pre-023 profile → run-over notice without the primer, then the primer on the next map open |
| T006a walkthrough fix (sdd-implementer) | opus (fallback, experiment 2 paused) | ~24K (measured return) | person's Phase 2 finding: the dismiss line sat left of center; leading spaces 13 → 17 (primer) and 13 → 19 (popup), spec code blocks updated to match; driver snapshots at 139×31 confirm both centered; no test pins the centering (a future text edit could un-center it) → sweep |
| T009 close-out (sdd-implementer) | opus (fallback, experiment 2 paused) | ~81K (measured return) | done first try; three green test runs, every mechanical check clean (warning count 0 on both main and the branch; game.rs hunks only in `game_action_from_key`, `restart_opponent_pause`, `mod tests`); acceptance criteria checked off with evidence; carried notes applied (two doc comments, one test comment, one test rename, driver skill key reference); flagged that `CLAUDE.md` rides in on the branch (person's one-commit ruling of 2026-09-14) |
| Pre-merge sweep (skeptical-reviewer) | opus | ~125K (measured return) | signed off, 0 blocking, 9 notes — fixed before the merge by the orchestrator: tier-log row 2 (experiment 2 never ran a dispatch here; date), the unfilled end-reading placeholder, close-out §2a misattributed the bust line to the board (it is the `?` overlay), and close-out §2c/§3 additions (modal-clear invariant; CLAUDE.md-on-branch record; SKILL.md correction). Carried to a future chore, not done by hand: pin the dismiss line's centering in the T006 test; a `FOOTER_RESERVE` const in opponent_select and a title-vs-row-0 pin; uppercase P inert (pre-existing convention). Reviewer verified all four modal-raising callers clear their modal first. |
