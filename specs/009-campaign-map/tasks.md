# Tasks: Campaign Map

Subsystem D of the campaign epic (navigation + progression layer; economy
stubbed). Each task builds (`cargo build`) and tests (`cargo test`) green before
it's done, with actual output reported. One commit per task referencing its ID;
the draft PR (opened at T001) tracks the diff. Data/logic work is reviewed **per
task**; the map UI is reviewed **per phase**. Do not weaken a test to pass.

Branch `009-campaign-map` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [x] **T001 — Campaign data model & run state**
  New `src/campaign.rs`: `Planet` (`id`, `name`, `region`, `blurb`, `fx`, `fy`,
  `opponents`, `requires`; `Copy`), `const PLANETS: [Planet; 4]` (Cinder →
  {Ashfall, Drift} → The Spindle, using all 5 roster opponents), `START_PLANET`,
  `planet_by_id`; `CampaignRun` (serde, `Default`: `beaten` set-map, `current`,
  `in_progress: Option<NodeRef>`) with the derived-state logic
  (`mark_beaten`, `planet_cleared`, `planet_unlocked`, `next_opponent`,
  `run_complete`). Embed `#[serde(default)] campaign: CampaignRun` in `Profile`
  (`profile.rs`) + accessors; update `Default`/`profile_with`. `pub mod
  campaign;` in `src/lib.rs`. Open the draft PR.
  *Verify: `cargo build` no new warnings; `cargo test` green — PLANETS
  integrity (unique ids; opponents resolve; `requires` real; `fx/fy` in 0..=1;
  start has no requires); clear/unlock/next-opponent + the fork (Ashfall/Drift
  after Cinder, Spindle after both); `run_complete`; profile campaign
  round-trip; a pre-009 profile loads a default run.*

- [x] **T002 — Full-screen map screen + menu front door**
  New `src/campaign_map.rs` (`CampaignMapState`, `MapOutcome`, `handle_input`
  over unlocked planets, `tick(dt)` starfield, `draw` — header, node scatter +
  routes, twinkling starfield, bottom info panel); `CampaignMapLayout` in
  `src/layout.rs` (per-draw, full-screen); `Screen::CampaignMap` in
  `src/screen.rs`; `pub mod campaign_map;` + `STARFIELD_TWINKLE_MS` in
  `src/lib.rs`. `src/menu.rs`: `MenuItem::StartCampaign` + rename `StartGame` →
  `QuickPlay` (labels + tests). `src/app.rs`: `open_campaign_map`, the three
  Screen arms (input/`?`/draw), the menu arms, and the `App::tick` twinkle call.
  *Verify: `cargo test` green with map-navigation + layout-fit tests; driver —
  Start Campaign opens a full-screen map (header, nodes, routes, starfield,
  panel), cursor moves over unlocked planets, Quick Play still runs
  opponent-select, Esc returns to the menu.*

- [ ] **T003 — Launch + win→progress spine**
  `start_match` gains `campaign: Option<NodeRef>` (sets/clears
  `profile.campaign.in_progress` + `save()`); the map `Launch` arm guards
  `deck_is_valid()` then launches with `Some(node)`; the `App::tick` GameOver
  seam records a campaign win (`mark_beaten` + `save`); the InGame-GameOver
  campaign path routes the acknowledge key back to the map (suppressing the
  quick-play rematch). `Continue` unchanged.
  *Verify: `cargo test` green (any pure logic added); driver (back up
  profile.json + savegame.json) — launch Cinder→Greeb, win → node cleared +
  fork unlocked; clear both mid worlds → Spindle unlocks; loss → planet still
  open; quit mid-campaign-match → Continue resumes → win still records; Quick
  Play unaffected.*

- [ ] **T004 — Verification & close-out**
  Full driver sweep of every acceptance box. Run the `skeptical-reviewer`.
  Update `Readme.md` (campaign is new user-facing behavior), `ROADMAP.md`
  (Campaign epic — D shipped / the front-door rework), `DECISIONS.md` (the
  rulings: front door, first-map shape, loss-has-no-penalty, campaign-context-
  in-profile). On the human's word: mark the PR ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The map is
a full-screen `Screen` (`campaign_map.rs`, mirroring `opponent_select.rs` but
spanning the whole terminal via a per-draw `CampaignMapLayout`). The graph is
`const` data (`campaign.rs`) with all derived state computed from a `beaten` set
+ `PLANETS` — no stored redundancy. The engine stays campaign-agnostic: campaign
context lives in the profile as an `in_progress` node pointer (persisted, so a
resumed campaign match still routes to the map at `GameOver`). Matches launch
through the existing `start_match`; wins are recorded at the existing `App::tick`
GameOver seam. The twinkle is the map's own slow `dt` accumulator, separate from
the selection pulse (per the amended Motion rule). Quick Play == today's Start
Game; Continue is unchanged.
