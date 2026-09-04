# Tasks: Bounded misplays (spec 013)

Each task builds (`cargo build`) and tests (`cargo test`) green before it's done,
with actual output reported. One commit per task referencing its ID; the draft PR
(opened at T001) tracks the diff. This is a focused correction to shipped-AI
behavior — **reviewed at T002** (the behavior change) and again at close-out. Do
not weaken a test to make it pass; the two old tests that encode the buggy contract
are re-authored deliberately, not deleted to go green.

Branch `013-bounded-misplays` off `main` (012-economy is merged). Plan mirrors
`plan.md`; approved plan also at `~/.claude/plans/iterative-meandering-blum.md`.

---

- [ ] **T001 — Setup**
  Branch `013-bounded-misplays`; `specs/013-bounded-misplays/{spec,plan,tasks}.md`;
  open the draft PR.
  *Verify: `cargo build` / `cargo test` still green on the untouched tree (baseline);
  draft PR open.*

- [x] **T002 — Bound the misplay + tests**
  In `src/game.rs`: add `MISPLAY_TIMID_MARGIN: i32 = 2`; add `position_is_open`
  (`!player.stood && opponent.score() <= 20`) and gate `opponent_action` on it;
  bound the `misplay` `Hit => Stand` arm to `score >= effective_threshold −
  MISPLAY_TIMID_MARGIN`. Re-author the two spec-010 tests that lock in the old
  contract (`misplay_deviates_each_best_move_legally`,
  `opponent_action_misplays_below_the_rate_and_plays_best_at_or_above`) and fix the
  stale comment on `full_match_terminates_with_a_maximally_misplaying_opponent`. Add
  the new tests: regression (live, score 0, misplay 1.0 → Hit); per-profile property
  over `OPPONENTS` (live + non-full table, never a misplay-stand below `t − MARGIN`);
  boundary (`t−1` stands, `t − MARGIN − 1` → Hit); no-misplay-when-resolved (stood
  player, all three shapes; over-20 recovery not fumbled); greed-still-fires
  (`score ≥ t`, live → Hit).
  *Verify: `cargo test` green — the new + re-authored tests pass, masters-never-misplay
  still passes; `cargo build` no new warnings. Mutation check: temporarily widen the
  bound and confirm the regression/property tests go red. **(review)***

- [x] **T003 — Docs + close-out**
  Update `docs/opponents.md` (misplay is bounded, never suicidal). Add a pointer note
  to `specs/010-smarter-opponents/spec.md` (misplay model bounded by spec 013; the
  two acceptance-cited tests re-authored). Driver spot-check (back up real
  `profile.json`/`saves/`): the first campaign opponent no longer stands on 0 or
  concedes from behind over several rounds; a master still plays tight. Re-run the
  `skeptical-reviewer`. On the human's word: a `DECISIONS.md` spec-013 entry on
  `main`; mark the PR ready and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported verbatim;
  reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. This is a
**spec-010 correction**: spec 010 promised a "bounded" misplay
(`specs/010-smarter-opponents/spec.md:144`) but shipped an unbounded `Hit => Stand`
(`src/game.rs:601`) that lets opponents stand on 0 and concede from behind. The fix
is two layers in `src/game.rs`: a `position_is_open` gate on `opponent_action` (no
misplay once the player has stood or the opponent is over 20) and a
`MISPLAY_TIMID_MARGIN` bound on the `misplay` `Hit => Stand` arm (no suicidal low
stands). Keep the believable open-position errors (greedy over-hit bust, card
fumble). Masters (misplay 0.0) are untouched. No engine/board/save/roster change.
