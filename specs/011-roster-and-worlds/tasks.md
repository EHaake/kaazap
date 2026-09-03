# Tasks: Roster expansion & new campaign worlds

Each task builds (`cargo build`) and tests (`cargo test`) green before it's done,
with actual output reported. One commit per task referencing its ID; the draft PR
(opened at T001) tracks the diff. This is foundational content/data — reviewed
**per task**. Do not weaken a test to make it pass.

Branch `011-roster-and-worlds` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [x] **T001 — Roster expansion (5 → 10 opponents)**
  Add five `OPPONENTS` entries in `src/opponent.rs` — `dax` (15, Aggressive),
  `nima` (16, Cautious), `brakka` (17, Aggressive), `kesh` (18, Aggressive),
  `sovereign` (19, Calculating, misplay 0, the final boss) — each with an
  original name/label/blurb and a 10-card deck from `card::ALL_SIDE_CARDS`,
  inserted so `stand_threshold` stays non-decreasing. The boss deck is at least
  as flexible as the Magistrate's.
  *Verify: `cargo build` no new warnings; `cargo test` green — the roster guards
  (`roster_runs_easy_to_hard_by_threshold`, `roster_ids_are_unique_and_names_
  nonempty`, `every_roster_deck_can_fill_a_hand`, `every_roster_card_is_in_the_
  universe`, `misplay_rates_are_valid_and_the_default_is_deterministic`) all pass
  for 10 opponents. Open the draft PR.*

- [ ] **T002 — Worlds & map (4 → 8 worlds)**
  Rewrite `PLANETS` in `src/campaign.rs` to the 8-world DAG (fork into two
  two-world lanes → rejoin → linear Core run → boss), existing four keeping their
  opponents; place `fx`/`fy` legibly. Rewrite the topology-hardcoded tests
  (`the_fork_...`, `navigation_stays_on_unlocked_planets`), generalize the
  full-sweep completion test to derive from `PLANETS`, and add the new **map
  legibility** test (nodes/labels don't collide or clip at 89×31) and **graph
  integrity** test (one start, acyclic, all reachable).
  *Verify: `cargo test` green — the new/updated campaign + layout tests and the
  auto-passing well-formedness/bounds guards; `cargo build` clean.*

- [ ] **T003 — Verification & close-out**
  Full driver sweep (back up `profile.json`/`saves/`): the 8-world map renders
  legibly (nodes, labels, fork + rejoin); navigate the unlock order; play a
  couple of new opponents including the **boss** and confirm board-aware AI,
  win→map progress, and no panics; snapshot Quick Play at 89×31 to confirm the
  10-opponent list fits. Run the `skeptical-reviewer`. Rewrite `docs/opponents.md`
  (10-row roster table + the campaign map/world notes) and the README world/
  opponent counts. On the human's word: `ROADMAP.md` ("More campaign worlds /
  roster expansion" → Shipped) + a `DECISIONS.md` spec-011 entry on `main`; mark
  ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. Everything
here is `const` data over shipped patterns: `PLANETS` (`campaign.rs`) and
`OPPONENTS` (`opponent.rs`). Rendering, unlock/clear, routes, the cursor, the
header counter, and save/resume are all **derived** from `requires` + the
`beaten` set — so adding worlds/opponents needs no logic changes. The one thing
the code does *not* check is node/label overlap on the star map; the new
legibility test is the guard. Keep the Spindle at `[rix, magistrate]` so
`clearing_a_planet_needs_all_its_opponents` stays valid, and give every new world
a non-empty `requires` so `a_fresh_run_unlocks_only_the_start` holds.
