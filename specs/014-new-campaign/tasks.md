# Tasks: New Campaign option (spec 014)

Each task builds (`cargo build`) and tests (`cargo test`) green before it's done,
with actual output reported. One commit per task referencing its ID; the draft PR
(opened at T001) tracks the diff. New Campaign is a **destructive** reset, so it
ships behind a default-No confirm and with tests. Do not weaken a test to pass.

Branch `014-new-campaign` off `main` (013 merged). Plan mirrors `plan.md`; approved
plan also at `~/.claude/plans/iterative-meandering-blum.md`.

---

- [ ] **T001 — Setup**
  Branch `014-new-campaign`; `specs/014-new-campaign/{spec,plan,tasks}.md`; open the
  draft PR.
  *Verify: baseline `cargo build` / `cargo test` still green; draft PR open.*

- [x] **T002 — Progress check + reset (pure)**
  `CampaignRun::has_progress()` (`src/campaign.rs`) and `Profile::reset_to_starter()`
  (`src/profile.rs`, `*self = Profile::default()`), with tests.
  *Verify: `cargo test` green — `has_progress` false on default / true after
  `mark_beaten`; `reset_to_starter` turns a dirtied profile (progress + credits +
  edited collection) back into the starter (starter collection/deck, empty campaign,
  0 credits), version unchanged; `cargo build` clean.*

- [x] **T003 — The modal flow (`src/app.rs`)**
  Two `Modal` variants (`CampaignEntry`, `ConfirmNewCampaign`); extract
  `enter_campaign_continue()`; the StartCampaign branch (choice when
  `has_progress`); input handling (toggle/commit/cancel, the reset action on Yes);
  `draw_two_choice` helper + the two draw arms. Board/engine untouched.
  *Verify: `cargo build` clean; `cargo test` green (247 passed) — existing menu/
  confirm/draw tests unaffected.*
  *Conscious accept (mirrors spec 012's App::tick note): the modal-handler glue is
  not unit-tested, following the existing `ConfirmNewGame` handler precedent (also
  untested). Two reasons: `App::new` loads the **real** profile from disk, so
  driving the destructive "Yes" in a test would wipe the player's real
  `profile.json`; and the handlers are thin wiring over the T002-unit-tested
  `reset_to_starter` / `has_progress`. The full flow is verified by the driver
  sweep (T004) and the skeptical-reviewer.*

- [ ] **T004 — Verify & close-out**
  Driver sweep (back up `profile.json`/`saves/`, restore + checksum): with cleared
  progress, Campaign → Continue/New Campaign panel; New → confirm; Yes → fresh map
  (0/8, only Cinder unlocked); a no-progress profile skips the panel; snapshot both
  modals at 89×31; no panics. Run the `skeptical-reviewer`. Update the README
  (`Readme.md`) note on campaign entry. On the human's word: `ROADMAP.md` +
  `DECISIONS.md` entries on `main`; mark ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported verbatim;
  reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The feature is
a menu-flow addition + a profile reset over shipped patterns: `Profile::default()`
(`profile.rs:79`) is the starter, so the reset is `*self = Profile::default()`
(settings live in a separate file and survive). The choice and confirm are `Modal`s
mirroring `ConfirmNewGame` (`app.rs:250`, `handle_confirm_input` at `:745`,
`draw_confirm_new_game` at `:944`); the entry point is
`activate_menu_item(MenuItem::StartCampaign)` (`app.rs:808`). Board/engine/save
format are untouched.
