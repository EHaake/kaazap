# Tasks: Series-aware banter, spoken word by word — spec 030

**Status**: Draft — pending sign-off
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

**Every Verify below runs the constitution's verification command, verbatim:**

    cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25

"The full command" in a Verify line means exactly that. Never a narrower
`cargo test --lib`, and never a warning-count grep in its place. Bundles and
reports echo the command that was run, not a label for it.

**Foundational phase: Phase 1.** It holds the `Speech` seam (`say`,
`advance_speech`) and the burble, and every line Phases 2 and 3 add is said
through them. Phase 2 depends on Phase 1. Phase 3 depends on Phase 1 and on
T004's pools, not on T005/T006.

**No task carries `review: per-task`.** The one task whose mistake later work
would inherit is T003 (the `App` speech seam). It is Phase 1's last task, so
the phase review is the next thing that runs anyway, and a per-task review
before it would buy nothing. Every task is covered by its phase review.

**Every phase header carries a `walkthrough:` marking** (the constitution's
*Pause cadence*). Phases 1–3 each change what the player sees or hears; the
close-out does not. The running **walkthrough list** near the foot of this file
is what unpaused phases append to and what the person walks at the close-out.

**No new test may construct an `App`**: `App::new` reads the real settings and
profile, and `Profile::save()` has no `cfg(test)` guard. Every decision here
sits behind a pure function and is tested there. The `App` wiring is checked
by the phase walkthroughs.

**Never run `cargo fmt`** (the repo is not rustfmt-clean; it would rewrite ~28
files). **Stage explicit paths, never `git add -A`**. T002 creates an untracked
file (`assets/sfx/burble.wav`) that must be added by name.

---

## Phase 1 — The line, spoken (foundational; walkthrough: at 139 columns or wider, start a Quick Play match — the opponent's greeting appears beside the portrait one word at a time, each word where it will sit in the finished line, each with a soft murmur; round, bust and match-end lines do the same. With Sound Effects at 0, or after `m`, the words still appear but make no sound. With Animations Off each line appears whole, with one murmur. Continue on a saved match shows no line until something happens. At 89 columns the board shows no line and makes no murmur, as before. **The murmur itself is the person's to approve by ear** (AC 9) — they may ask for it higher or lower, shorter, softer, rounder or buzzier, and each ask is a quick regenerate, not a redesign)

- [ ] **T001** — `src/lib.rs` + `src/banter.rs` + `src/portrait.rs` (tests
  only): the reveal, as pure logic. Per plan §Design 1–2 and §Design tensions
  1–4: `pub const WORD_STEP_MS: u64 = 200;` in `lib.rs` beside the spec 027
  beats, with the plan's comment. In `banter.rs`, beside `pick`:
  `word_count`, `revealed`, `words_due` and `pub struct Speech` with `new(line,
  animated)`, `advance(dt) -> bool`, `settle`, `words_shown`, `text`, each
  with the plan's doc comment (including `advance`'s "a step across more than
  one boundary shows them all and still returns true once", and "shown never
  decreases"). A word is a maximal run of non-space characters. The module doc
  gains one sentence naming `Speech` (spec 030). Tests in `banter.rs`, per plan
  §Tests: `the_word_step_is_about_a_fifth_of_a_second`,
  `every_line_finishes_inside_a_second` (over the eight existing pools of every
  voice; T004 widens it), `a_line_is_revealed_in_place_word_by_word` (every
  line of every pool),
  `a_spoken_line_shows_a_word_per_step_from_the_first_frame`,
  `animations_off_speaks_the_whole_line_and_owes_no_more_burbles`,
  `a_settled_line_is_whole_and_silent`,
  `a_stalled_step_shows_every_due_word_and_owes_one_burble`. In `portrait.rs`'s
  tests module only: `a_revealed_line_draws_each_word_where_the_finished_line_has_it`
  (through the existing `draw_presence_extras` on `inmatch_panel()`; the line
  row is `y = 15`, per the module's own comment). Nothing calls `Speech`
  outside tests yet. It is `pub` in a library crate, so no `dead_code` warning.
  (Copies: `motion.rs`'s `thinking_suffix_at` and
  `beats_are_named_constants_within_bounds`; `banter.rs`'s `all_sets()` /
  `classes()` test helpers; `portrait.rs`'s
  `banter_fits_inside_interior_and_clips_at_the_border`.)
  *Verify: the full command, verbatim, green, no new warnings, the seven new
  `banter.rs` tests and the one `portrait.rs` test passing by name. No existing
  assertion in any file changes, and any that would is stop-and-report. In
  `portrait.rs` the diff is inside `mod tests` only. `git diff --stat` shows
  exactly `src/lib.rs`, `src/banter.rs`, `src/portrait.rs`. The report quotes
  `Speech::advance` and `revealed` verbatim.*

- [ ] **T002** — `scripts/gen_sfx.py` + `assets/sfx/burble.wav` (new) +
  `src/audio.rs` + `assets/CREDITS.md`: the burble. Per plan §Design 3–4 and
  §Design tension 9. In the script: the `BURBLE` parameter block with the
  plan's comment, and a `burble()` synth. It is additive: `harmonics` harmonics
  of a gliding, vibrato'd fundamental, each weighted `1/k` times the resonance
  of two gliding formants, with raised-cosine attack/release and tremolo,
  normalised to `peak`. It uses **no `random`**, so `bust`'s seeded noise is
  undisturbed. `main()` accepts optional sound names: `python3
  scripts/gen_sfx.py burble` writes only `burble.wav`, and no arguments writes
  all fourteen. Run `python3 scripts/gen_sfx.py burble` — **never the no-argument
  form** in this task. In `audio.rs`: `Sfx::Burble` and its `include_bytes!`
  arm; `pub const BURBLE_PITCHES: [f32; 5]`, `pub fn burble_cue(word: usize)
  -> Cue`, `pub const BURBLE_GAP_MS: u64 = 150` and `pub fn
  burble_clear(since_last: Duration) -> bool`, each with the plan's doc (plan
  §Design 4, §Design tension 3). Tests: `the_burble_is_softer_than_the_music`
  (decode `Sfx::Burble.bytes()` and `MUSIC_BYTES` with `rodio::Decoder` over a
  `Cursor`, opening no device; take the music's RMS over its first 60 s of
  samples, by the decoder's own sample rate and channel count; assert `burble
  peak <= music_rms * d.music_volume / d.sfx_volume` with `d =
  Settings::default()`; `println!` both figures with the words `peak` and
  `rms` in the lines), `a_burble_ends_before_the_next_can_start`,
  `burbles_are_spaced_by_the_gap`, `each_word_has_its_own_burble_pitch`.
  **Set `peak`** from the measured music RMS, comfortably under the ceiling,
  and say what you chose and why. **Report the softer-than-the-music test's
  runtime in the debug build** (the 60 s MP3 decode). If it is more than a
  few seconds, shorten the window to 30 s and say so. In `CREDITS.md`, the *Sound effects* sentence names the burble as
  generated by the same script (plan §Design 10). Do not run `cargo fmt`.
  (Copies: the script's `tone`/`sweep`/`write_wav` and its `sounds` dict;
  `audio.rs`'s `Sfx::bytes` arms and its tests module.)
  *Verify: the full command, verbatim, green, no new warnings, the four new
  tests passing by name. **In addition to** (never instead of) the full
  command: `cargo test -q --lib the_burble_is_softer_than_the_music --
  --nocapture 2>&1 | grep -iE 'peak|rms|test result'`, with its printed peak
  and music RMS quoted in the report along with the test's runtime. The script's own output
  line (`wrote burble.wav (…s)`) is quoted. `git status --porcelain assets/sfx`
  prints **exactly** `?? assets/sfx/burble.wav` — the thirteen existing
  sounds untouched (the file is untracked, so `git diff --stat` cannot list it
  and is not the gate). `git diff --stat` shows exactly `scripts/gen_sfx.py`,
  `src/audio.rs`, `assets/CREDITS.md`. No existing assertion changes. The
  report quotes the `BURBLE` block, `burble_cue` and `burble_clear` verbatim. The orchestrator
  stages `assets/sfx/burble.wav` by path.*

- [ ] **T003** — `src/app.rs` + `src/settings.rs` (doc) + `design/brief.md` +
  `Readme.md`: the speech seam. Per plan §Design 5 (*Phase 1*), §Design 10 and
  §Design tensions 1, 3–5. In `app.rs`: field `banter` → `speech:
  Option<Speech>` and a new field `since_burble: Duration` (`Duration::MAX` in
  `App::new`), each with the plan's doc; `fn say`, `fn line_visible` (on the
  board it reads the existing `self.board_view.is_wide()`, and `board.rs` does
  not change), `fn advance_speech` and `fn burble` with the plan's docs.
  `burble` is the only caller of `burble_cue`, and gates on `burble_clear`.
  `advance_speech(dt)` is called in `tick` immediately before
  `self.emit_audio_cues()`. **Enumerate every site first** with `grep -nw
  'self\.banter' src/app.rs` (the `-w` keeps `self.banter_last` out). The
  planner found seven: `start_match`'s seed (~1088); `update_banter`'s rematch
  (~1139), event (~1145) and phase-clear (~1151) branches; the Continue arm
  (~1764); the `draw` `InGame` arm (~1939); and the field declaration and
  `App::new` initialiser, found with `grep -nE '^\s*banter:' src/app.rs`.
  Each `Some(line)` pair becomes `self.say(line)`, each `= None` becomes
  `self.speech = None`, and `draw` computes `let line =
  self.speech.as_ref().map(Speech::text);` once and passes `line.as_deref()`.
  `banter_last` is untouched. **Pools are unchanged in this task**:
  `start_match` still draws `banter_for(opp_id).match_start` with `None` as
  `last`, and `update_banter` still uses `lines_for` (T006 changes both).
  `settings.rs`: the `animations` field doc only (plan §Design 10).
  `design/brief.md` *Motion*: the spec 030 amendment sentence, appended after
  the spec 027 amendment, verbatim from plan §Design 10. `Readme.md`: the one
  status-paragraph phrase, verbatim from plan §Design 10. **It spans lines
  20–21 inside a `> ` block quote**: keep the `> ` prefix on every touched
  line, and re-wrap only those lines. Do not run `cargo fmt`.
  (Copies: `tick`'s `BoardMotion` observe/reset `match` for the
  settle-off-screen rule; `emit_audio_cues` for the observer shape.)
  *Verify: the full command, verbatim, green, no new warnings. `grep -nw
  'self\.banter' src/app.rs` and `grep -nE '^\s*banter:' src/app.rs` both
  empty. `grep -n 'burble_cue(' src/app.rs` returns **exactly one** line, in
  `burble` (the import line has no `(`). `grep -n 'self\.burble(' src/app.rs`
  returns **exactly two**: one in `say`, one in `advance_speech`.
  `grep -n 'advance_speech(' src/app.rs` returns the definition and the one
  call in `tick`. **No line inside any `#[cfg(test)]` module changes, and no
  existing assertion changes**: stop and report if one would. `git diff --stat`
  shows exactly `src/app.rs`, `src/settings.rs`, `design/brief.md`,
  `Readme.md`. `Readme.md`'s diff keeps the `> ` prefix on every changed line.
  The report quotes `say`, `line_visible`, `advance_speech`, `burble` and the
  changed lines of `tick` verbatim. Then the
  orchestrator drives the Phase 1 walkthrough's items 1–4 (plan §Verification)
  on a scratch `KAAZAP_DATA_DIR` before the phase review.*

---

## Phase 2 — Lines that follow the series (walkthrough: play a campaign series against Greeb — the first match opens with a greeting; the second opens with a line about the score, and Greeb behind sounds different from Greeb ahead; at 1–1 the opener treats the match as the decider; the deciding match ends on a line about the whole series, won or lost, and its game-over frame keeps the final score, e.g. `Series 2 – 1`; the map afterwards has its usual banner and no score. Quick Play and a rematch on a cleared planet sound as they did. The Sovereign at 1–1 has lines of its own. **And read every new line, voice by voice, and change whatever doesn't land**)

- [ ] **T004** — `src/banter.rs`: the series pools and their lines. Per plan
  §Design 2 and §Design tension 7. `pub enum SeriesState` and `pub fn
  series_state(&Series) -> SeriesState` (importing `campaign::{Series,
  wins_needed}`); the six `BanterSet` fields with the plan's docs; `pub fn
  start_lines` and `pub fn lines_in_series` with the plan's docs. **Write the
  lines** for all eleven voices, 146 in all: 3 Leading, 3 Trailing, 3 Decider,
  2 series won and 2 series lost per voice, plus 3 All square for The
  Sovereign. Every other voice's `all_square` is `&[]`. Each voice follows its
  const's doc comment. Follow the plan's authoring rules: ≤ 20 characters, ≤ 5
  words, no duplicate within a pool, no line shared with any other voice
  (including every existing line), prefer no repeat of the same voice's other
  pools. **No existing line or pool changes.** Tests, per plan §Tests:
  `the_series_state_for_every_score` (literal tables),
  `every_reachable_state_has_lines_in_every_voice`,
  `every_voice_is_complete_for_series_play`,
  `the_start_pool_follows_the_series_state`,
  `a_deciding_match_end_draws_the_series_result`,
  `outside_a_decided_series_every_event_draws_todays_pool`. Add the test helper
  `series_classes(set)` (the six new pools) and `all_pools(set)` (the eight old
  pools followed by the six new). **Sanctioned existing-test change, as a
  class** (plan §Tests): a test whose loop exists to check a property of *every
  line* may widen its iterator from `classes(set)` to `all_pools(set)`. The
  planner found three: `every_line_of_every_set_fits_the_panel`,
  `every_class_of_every_set_has_distinct_lines`,
  `no_line_is_shared_across_any_two_sets`. T001's
  `every_line_finishes_inside_a_second` makes four. No assertion's condition,
  message meaning or threshold changes. `every_set_non_empty_with_repeatable_floor`,
  `generic_classes_non_empty_and_repeatables_have_floor`,
  `every_generic_line_fits_the_panel` and the fingerprint tests keep
  `classes(set)` / their own lists. Do not run `cargo fmt`.
  (Copies: the existing `BanterSet` consts and their doc voice; the tests
  module's `all_sets()`, `classes()`, `ROSTER_IDS`.)
  *Verify: the full command, verbatim, green, no new warnings, the six new
  tests passing by name. `git diff --stat` shows exactly `src/banter.rs`. No
  existing line was edited or removed: `git diff src/banter.rs | grep -E
  '^-\s+(match_start|round_win|round_loss|round_tie|opponent_bust|player_bust|match_win|match_loss):'`
  is empty. Any existing assertion that moves outside the sanctioned iterator
  class is stop-and-report. The report states the new-line count per voice and
  quotes `series_state`, `start_lines` and `lines_in_series` verbatim.*

- [ ] **T005** — `src/app.rs`: the deciding match's final score. Per plan
  §Design 5 (*Phase 2*, first four bullets) and §Design tension 8. `fn
  match_series` (with `board_series_line` rewritten over it, same format, same
  tests), `fn decided_series` (pure, the plan's doc), field `final_series:
  Option<Series>` (the plan's doc; `None` in `App::new`), set in `tick`'s
  resolution block from `before` taken **before** `resolve_match`, cleared in
  `tick`'s final `match &self.screen` `_` arm beside the `BoardMotion` reset.
  The `InGame` draw arm passes `campaign.series().or(self.final_series.as_ref())`.
  Test, in `app.rs`'s tests module, with **no `App`**:
  `decided_series_agrees_with_the_series_rule` (plan §Tests). Do not run
  `cargo fmt`. (Copies: `victory_due`'s field doc and its set-in-`tick` line;
  the `motion` reset arm; the existing `board_series_line` tests' `node`
  helper.)
  *Verify: the full command, verbatim, green, no new warnings, the new test
  passing by name, and the existing `board_series_line` test unchanged and
  passing. Every line `grep -n 'final_series' src/app.rs` returns is in the
  field's declaration or doc, its initialiser, the `tick` set and clear, or the
  `InGame` draw arm. Nothing on the map path reads it. `git diff --stat` shows
  exactly `src/app.rs`. No existing assertion changes. The report
  quotes `decided_series`, `match_series` and the changed `tick` lines
  verbatim.*

- [ ] **T006** — `src/app.rs`: lines that follow the series. Per plan §Design 5
  (*Phase 2*, last three bullets) and §Design tension 7. `fn series_state_now`
  with the plan's doc. `start_match`: `let state = self.series_state_now();
  let last = state.and(self.banter_last);` then `pick(start_lines(banter_for(opp_id),
  state), last, …)`. Inside a series the start avoids the venue's line (AC 6);
  outside one `last` is `None`, exactly as today (AC 4, sign-off B1). Write no
  `None` literal in the `pick(` call. `update_banter`'s rematch branch draws `start_lines(…,
  self.series_state_now())`, and its event branch draws `lines_in_series(…, ev,
  self.final_series.is_some())`. Update the imports. Do not run `cargo fmt`.
  (Copies: `update_banter` itself.)
  *Verify: the full command, verbatim, green, no new warnings. `grep -n
  'pick(.*None' src/app.rs` is empty. `grep -nw 'lines_for' src/app.rs` and
  `grep -nw 'match_start' src/app.rs` are both empty: every pool now comes
  from `start_lines` / `lines_in_series`, whose Quick Play equivalence T004
  pins. `grep -n 'state.and(self.banter_last)' src/app.rs` returns exactly one
  line, in `start_match`. `git diff --stat` shows exactly `src/app.rs`. No
  existing assertion changes. The report quotes `series_state_now` and the
  changed lines of
  `start_match` and `update_banter` verbatim. Then the orchestrator drives the
  Phase 2 walkthrough (plan §Verification) on a scratch `KAAZAP_DATA_DIR` and
  assembles the new-line list for the person's pause.*

---

## Phase 3 — The opponent at the venue (walkthrough: start a series from the map — at the venue the opponent says a line in the portrait panel, word by word, with the murmur; open the Card Shop or the collection and come back — the same line, whole and silent; Play and commit a stake — the match opens on a different line; after a match that doesn't decide the series, the venue has a new line for the new score; quit to the menu and come back in — a fresh line; quit the game mid-series, relaunch, and Start Campaign — a fresh line. At both 89 and 139 columns, plus, at 89, whether the run-over notice covers the opponent's panel)

- [ ] **T007** — `src/portrait.rs` + `src/layout.rs` + `src/venue.rs` +
  `src/app.rs` (one argument): room for the venue line. Per plan §Design 7–9
  and §Design tension 10. `portrait.rs`: `pub fn draw_banter_line(frame, panel,
  line)` with the plan's doc; `draw_presence_extras` calls it in place of its
  inline `draw_text_in` for the line. `layout.rs`: `VENUE_PANEL_H = 2 + 1 +
  PORTRAIT_HEIGHT + 2` with the plan's rewritten doc. `venue.rs`: `draw` gains
  `line: Option<&str>` before `pulse` and draws it with `draw_banter_line` on
  `layout.portrait`; the `draw` and module docs each gain their clause.
  `app.rs`: the `Screen::Venue` draw arm passes `None` (T008 wires the line).
  New test `the_venue_line_sits_inside_the_portrait_panel` (plan §Tests).
  **Sanctioned existing-test change, as a class** (plan §Tests): an assertion
  whose subject is the venue presence panel's bottom edge, height or area, and
  a `VenueState::draw` call gaining its `line` argument. **Enumerate before
  editing**: `grep -n 'VenueState::new().draw(' src/venue.rs` (the planner
  found two) and `grep -n 'portrait' src/layout.rs` read within the two venue
  tests (the planner found six pinned `Rect`s at ~890, ~892, ~1002–1005, each
  `y1` + 2, and the area comment at ~907, "330" → "374"). **Anything outside
  that class that moves is stop-and-report.** Do not run `cargo fmt`.
  (Copies: `draw_presence_extras`; `venue.rs`'s
  `the_venue_rows_breathe_only_around_the_action_row` for a drawn-frame test
  with `Profile::default()` and no `App`.)
  *Verify: the full command, verbatim, green, no new warnings, the new test
  passing by name, and every existing `portrait.rs` test unchanged and
  passing. The report lists every changed assertion with its old and new
  value, and each is inside the class. `grep -nE 'self\.screen =
  Screen::(CampaignMap|Venue)' src/app.rs` still returns exactly two lines,
  both in `open_campaign_home`. `git diff --stat` shows exactly
  `src/portrait.rs`, `src/layout.rs`, `src/venue.rs`, `src/app.rs`, with
  `app.rs` changed on the one draw-arm line.*

- [ ] **T008** — `src/app.rs`: the venue line on arrival. Per plan §Design 5
  (*Phase 3*) and §Design tension 6. `fn arrive_at_campaign` with the plan's
  doc. `enter_campaign`'s and `launch_from_map`'s `self.open_campaign_home()`
  become `self.arrive_at_campaign()`. The two Back arms (Card Shop,
  deck builder `BackTo::Campaign`) keep `self.open_campaign_home()`. Update
  `enter_campaign`'s doc sentence about Back paths so it names
  `arrive_at_campaign`. The `Screen::Venue` draw arm passes `line.as_deref()`.
  Do not run `cargo fmt`. (Copies: `launch_from_map`'s derived-screen comment;
  `start_match`'s seed as rewritten by T006.)
  *Verify: the full command, verbatim, green, no new warnings. `grep -n
  'self\.arrive_at_campaign()' src/app.rs` returns exactly two lines, in
  `enter_campaign` and `launch_from_map`. `grep -n 'self\.open_campaign_home()'
  src/app.rs` returns exactly three: in `arrive_at_campaign`, the Card Shop
  Back arm and the deck builder Back arm (the test's `app.open_campaign_home()`
  does not match). `grep -nE 'self\.screen = Screen::(CampaignMap|Venue)'
  src/app.rs` still returns exactly two lines, both in `open_campaign_home`.
  `git diff --stat` shows exactly `src/app.rs`. No existing assertion changes.
  The report quotes `arrive_at_campaign` verbatim. Then the orchestrator
  drives the Phase 3 walkthrough (plan §Verification, items 1–7, including the
  relaunch and the 89-column run-over capture) at both widths on a scratch
  `KAAZAP_DATA_DIR`.*

---

## Final phase — Spec close-out (walkthrough: none — documentation, mechanical checks and the pre-merge sweep; the person's walkthrough list below is what they walk at this phase)

- [ ] **T009** — Close-out. Draft `specs/030-series-banter/closeout-main-docs.md`
  in spec 029's shape.
  **ROADMAP**: mark *Series-aware banter* shipped as spec 030 (`grep -n -i
  "banter" ROADMAP.md`, read and judge each hit, including the *What's left
  for v1* paragraph, which now leaves only music). Add the follow-ups this spec
  deliberately deferred: per-opponent burble voices; word-by-word text
  elsewhere; a skip key; and a burble on the compact board, which ruling 7A
  left silent, as a possible later change.
  **DECISIONS**: the rulings 1B, 2A, 3B, 4A, 5B, 6A, **7A** (no line on the
  compact board, so no burble) and **8A** (a resumed match stays blank), and
  the spec's unruled defaults; the person's by-ear approval of the burble
  (AC 9) and any tweaks (the sub-lettered T002 tasks, with the final `BURBLE`
  numbers). Then the plan's design calls with their reasons: one
  `Speech` replacing `banter`; reveal by blanking rather than slicing; burbles
  only from advancing the one `Speech`, at most one per step, and never two
  within `BURBLE_GAP_MS` (the two cases the gap drops a burble); the Animations
  setting read at say-time; arrival as the caller's word, with returns
  settling in `tick`; the series state in `banter.rs`, read through
  `match_series`; `final_series` in the motion idiom rather than
  `victory_due`'s take-on-entry, and `decided_series` repeating the tally rule
  under a test rather than changing `SeriesOutcome`; the loudness ceiling as a
  number and why it covers the defaults; the venue panel's two extra rows and
  the class of layout assertions that moved; `banter_last` fed to the match
  start **only inside a series** (`state.and(self.banter_last)`), so Quick Play
  and rematches pick exactly as before (sign-off B1).
  **Docs on the branch**: confirm `design/brief.md`'s amendment, `Readme.md`'s
  phrase and `assets/CREDITS.md`'s sentence landed (T002/T003).
  `grep -n -i "banter\|sound\|animation\|line" assets/how_to_play_text.txt`,
  read and judge. No change is expected; if one is needed, it is a finding,
  not an edit.
  Run the full command three consecutive times and paste the tails.
  **Mechanical checks** (three-dot, since `main` may move): `git diff
  main...HEAD --stat` lists none of `src/game.rs`, `src/card.rs`,
  `src/player.rs`, `src/campaign.rs`, `src/profile.rs`, `src/save.rs`,
  `src/economy.rs`, `src/opponent.rs`, `Cargo.toml`, `Cargo.lock` (AC 15).
  `grep -n "VERSION" src/save.rs src/profile.rs` still reads 1 and 1.
  `git ls-files assets/sfx | wc -l` → 14. `grep -c 'include_bytes!("../assets/sfx/'
  src/audio.rs` → 14. `git diff main...HEAD -- assets/sfx` names only
  `burble.wav`. `grep -nE 'self\.screen = Screen::(CampaignMap|Venue)'
  src/app.rs` → exactly two lines, both in `open_campaign_home`. `grep -n
  'burble_cue(' src/app.rs` → exactly one line; `grep -n 'self\.burble('
  src/app.rs` → exactly two. `cargo build --all-targets`
  warning count equals `main`'s (compare via a throwaway `git worktree` with
  its own `CARGO_TARGET_DIR`, as spec 029's close-out did — never switch the
  branch). Check off `spec.md`'s **16** acceptance criteria with evidence: the
  tests by name with their green output, the Phase 1–3 walkthrough reports,
  and for AC 9 the person's approval quoted. Request the **pre-merge sweep**:
  its bundle is `git diff main...HEAD`, `spec.md`, `plan.md`, this file, and
  `closeout-main-docs.md`. Apply `closeout-main-docs.md` on `main` after the
  merge. Never chain a file edit, a branch switch and a commit in one shell
  command (spec 025's miss).
  *Verify (the implementer's): three green tails, zero failures; every
  mechanical check listed with its command and fresh output; every criterion
  but AC 9 ticked with evidence. `closeout-main-docs.md` is a new,
  untracked file, so it is checked with `git status --porcelain
  specs/030-series-banter` (it shows as `??`), not `git diff --stat`.
  **The orchestrator's clauses** (not the implementer's, who can do neither):
  AC 9 ticked with the person's approval quoted, and the pre-merge sweep clean
  or its findings resolved.*

---

## Handoff note

Read `CLAUDE.md` and `specs/030-series-banter/{spec,plan,tasks}.md`, then
implement from the first unchecked task. Involvement level is **product owner**:
the person owns `spec.md`, attests by using the app at phase pauses, and
receives spec-conformance summaries — not architecture reviews.

**Model policy — the economy profile** (this project's since 2026-09-19): every
row of the role table resolves to `opus` / `claude-opus-5`, **no dispatch
carries a per-call model override**, the session runs at `claude-opus-5` medium
from `.claude/settings.json`, and the close-out (T009) goes to `sdd-implementer`
like every other task. Nothing runs on Fable; do not move a role back to it
unless the person says so. (This spec's *spec session* ran on
`claude-opus-5-5` at high effort; see the tier log's header row.)

Dispatch each task to `sdd-implementer` on a shell-assembled task bundle (task
line, plan section, acceptance criteria, files, the pattern file to copy).
Verify from the implementer's verbatim output. **No task is `review:
per-task`**, so the orchestrator never re-runs the command itself before
committing. The game drives named in T003, T006 and T008 are walkthroughs,
not re-verifications.

**Foundational phase: Phase 1.** One `skeptical-reviewer` pass at the end of
each phase — after T003, T006, T008 and T009 (the sweep) — on a shell-assembled
bundle (the phase diff, the task lines, plan §Design and §Tests, the acceptance
criteria), one review plus at most one re-review. The Phase 1 review also
checks that no input path in `handle_key` reads `speech` (AC 11), that
`burble` is the only burble emitter (one `burble_cue(` call, two
`self.burble(` calls), and that its gap guard matches plan §Design tension 3. The Phase 3 review also
checks the venue against the constitution's *acted-on element stands apart*
rule. The line is not acted on and sits compact inside the panel, and the
action row's air is unchanged.

**Pause cadence — when there's something to try.** Pause for the person after
**Phase 1**, **Phase 2** and **Phase 3**, once each phase's review and the
orchestrator's driven walkthrough are done. Only the close-out is marked
`walkthrough: none`; it runs without a pause and appends its reason to the
walkthrough list.

**The Phase 1 pause is where the burble is approved, and it may loop.** The
person listens and either approves (AC 9) or asks for a change. Each change is
a sub-lettered task on T002 (`T002a`, `T002b`, …), per plan §Verification's
*Tweak loop*: edit the `BURBLE` numbers and/or `BURBLE_PITCHES`, run `python3
scripts/gen_sfx.py burble`, and run the full command green. Then, **in
addition**, run `cargo test -q --lib the_burble_is_softer_than_the_music --
--nocapture 2>&1 | grep -iE 'peak|rms|test result'` and quote its figures.
`git status --porcelain assets/sfx` shows only ` M assets/sfx/burble.wav`. The
person listens again. A tweak is routine, not a decision review. **A
numbers-only tweak needs no review** (the ceiling test is its check). Anything
more (the synth's code, a new parameter) rides the next phase review's bundle,
or the pre-merge sweep if none remains. It never gets a review of its own,
because the review cap does not allow one. **If they want it louder than the
test's ceiling allows**, that is plan §Open questions 3: a product question,
not a weaker test. Phase 2 may start once the person says continue, even if
the burble is still being tuned. Tweaks are independent of Phases 2–3, and
AC 9 stays unticked until approval.

**Rulings 7A and 8A (the person, 2026-09-23) are transcribed**: no line on the
compact board, so no burble there (plan §Design tension 5); a resumed match
stays blank (plan §Design tension 11). Plan §Open questions 1 and 2 are
closed.

**Every driver session runs with `KAAZAP_DATA_DIR` pointed at a scratch
directory**, and the report says which. The real profile and saves must never
be in play. A driver cannot hear. The orchestrator attests the text and the
timing from captured frames, and only the person attests the sound.

Repo-wide docs (`ROADMAP.md`, `DECISIONS.md`) change only via
`closeout-main-docs.md` on `main` after the merge. `design/brief.md`,
`Readme.md` and `assets/CREDITS.md` ride in on the branch. Never run `cargo
fmt`. Stage explicit paths. A task that isn't routine goes to a decision review
(`skeptical-reviewer`, top tier — on this profile the same model, no override).
A product question `spec.md` doesn't settle goes to the person. A phase pause
ends with what to check and how to say continue, with no continuation prompt
unless the person asks or says they're stopping.

---

## Walkthrough list (what the person walks at the close-out)

<!-- Every phase appends here as it completes: a paused phase appends what the
person already tried and attested, an unpaused phase appends either its
walkthrough items (for the person to try at the close-out) or its one-line
reason for having none. Skipping a pause defers the person's check; it does not
remove it. The orchestrator writes these rows; nobody else. -->

| Phase | Paused? | What the person walks — or why there is nothing |
|---|---|---|

---

## Notes for the close-out (T009), gathered during implementation

<!-- Non-blocking observations from phase reviews. Not code changes: they belong
in `closeout-main-docs.md`'s DECISIONS entry or in a roadmap follow-up, so the
next person to touch this code finds them. -->

---

## Tier log (this spec, under the model policy)

<!-- One row per implementer run and per reviewer invocation; a summary row per
phase the orchestrator fills at that phase's review. Record token usage from
each subagent return and any escape-hatch miss (a task the orchestrator had to
redo, and why). -->

| Task / invocation | Tier (dispatched → ran) | Tokens | Dispatches | First try | Blocking findings | Outcome / miss reason |
|---|---|---|---|---|---|---|
| **Economy profile** — the project's since 2026-09-19 (specs 028 and 029 ran under it). Every role resolves to `opus` / `claude-opus-5`; no per-call override anywhere, including the planner, the sign-off and the close-out; the implementation session at `claude-opus-5` medium from `.claude/settings.json`. **This spec's spec session — the conversation, the planner dispatch and the sign-off — ran on `claude-opus-5-5` at high effort, while `.claude/settings.json` names `claude-opus-5`**; recorded so a comparison against specs 028–029 reads the model that actually ran. Second spec under the `walkthrough:` marking: Phases 1–3 marked yes, the close-out none. | — | — | — | — | — | header |
| Planning: draft (sdd-planner) | opus → claude-opus-5-5 | (from the return) | 1 | — | — | drafted; 9 tasks in 4 phases, Phase 1 foundational, no `review: per-task`, Phases 1–3 `walkthrough: yes`, close-out `none`; one product question returned (the compact board's silence — ruled 7A); one interpretation flagged (the resumed match — ruled 8A); two stated deviations (§Design tensions 7 and 10) |
| Planning: sign-off revision (sdd-planner, same context) | opus → claude-opus-5-5 | (from the return) | 1 | yes | — | B1: `start_match` feeds `banter_last` only inside a series (`state.and(self.banter_last)`), so AC 4 holds literally; the §Design tension 7 deviation is withdrawn. B2: the burble figures are read with `cargo test -q --lib … -- --nocapture \| grep -iE 'peak\|rms\|test result'`, run in addition to the full command. Rulings 7A and 8A transcribed and the open questions on them closed. Second-look: `line_visible` reads the existing `BoardView::is_wide()`, so `board.rs` is out of the footprint; a `BURBLE_GAP_MS` guard (one `burble` emitter, pure `burble_clear`, bounds pinned) keeps an interrupting line's first burble from overlapping the old one; Phase 3 walkthrough gains the relaunch and an 89-column run-over capture; tweak-loop review rule; T009's orchestrator-only clauses; the Readme block-quote note; the burble test's runtime reported |
