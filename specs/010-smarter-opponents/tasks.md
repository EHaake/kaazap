# Tasks: Smarter (board-aware) opponent AI

Each task builds (`cargo build`) and tests (`cargo test`) green before it's
done, with actual output reported. One commit per task referencing its ID; the
draft PR (opened at T001) tracks the diff. Engine work is reviewed **per task**.
Do not weaken a test to make it pass.

Branch `010-smarter-opponents` created with the spec/plan/tasks. Plan:
`~/.claude/plans/iterative-meandering-blum.md` (mirrors this spec's docs).

---

- [x] **T001 — Strategy data model**
  New `AiStrategy { Basic, Aggressive, Cautious, Calculating }` (`Copy`) and two
  fields on `OpponentProfile` (`src/opponent.rs`): `strategy: AiStrategy`,
  `misplay: f32`. Fill the roster (`OPPONENTS`) + `DEFAULT_OPPONENT` per the
  plan's mapping (default = `Basic`, misplay `0.0`). Open the draft PR.
  *Verify: `cargo build` no new warnings; `cargo test` green — a profile-
  integrity test (every roster `misplay` in `0.0..=1.0`; `DEFAULT_OPPONENT` is
  deterministic); existing roster tests still pass. (No AI-behavior change yet —
  the new fields are unread until T002/T003.)*

- [ ] **T002 — Board-aware deterministic core**
  Rewrite `decide_opponent_move` (`src/game.rs`) to branch on `self.player.stood`
  → `decide_vs_stood_player` (stand when ahead; chase / play-to-win when behind;
  tie handling via `has_tiebreaker_in_play`) vs `decide_vs_live_player` (today's
  threshold play, Aggressive/Cautious shifting the effective threshold). Keep it
  deterministic. Add the both-boards test helper `board_at(...)`.
  *Verify: `cargo test` green — new board-aware policy tests (ahead-of-stood →
  Stand; behind + winning card → plays it; behind + none → Hit; tie ±
  tiebreaker; per-archetype differences); **every existing `ai_*` test still
  passing** (player-at-0 reduces to today's behavior).*

- [ ] **T003 — The misplay seam**
  `opponent_action(&self, roll: f32)` (misplay for `roll < profile.misplay`, else
  `decide_opponent_move()`), `misplay(&self, best)` (legal suboptimal deviation),
  and point `play_opponent_turn` at `opponent_action(rand::random_range(..))`.
  *Verify: `cargo test` green — seam tests (rolls below/above the rate; each
  `best` → its deviation; `DEFAULT_OPPONENT` never deviates); the end-to-end AI
  tests that drive `update()` stay deterministic (default misplay 0).*

- [ ] **T004 — Verification & close-out**
  Full driver sweep (back up `profile.json`/`savegame.json`): stand at a good
  total and watch an opponent try to beat you (stand when ahead, chase/play when
  behind) instead of grinding its threshold; sample archetypes (Greeb slips, the
  Magistrate plays tight); confirm resolution + no panics. Rewrite
  `docs/opponents.md` (board-aware order; three levers; strategy column). Run the
  `skeptical-reviewer`. On the human's word: `ROADMAP.md` (board-aware AI →
  Shipped; difficulty setting unblocked) + `DECISIONS.md` on `main`; mark ready
  and merge.
  *Verify: all `spec.md` boxes checked with evidence; build/test reported
  verbatim; reviewer findings resolved or ruled; docs updated.*

---

## Handoff note

Read `CLAUDE.md`, then this spec's `spec.md` / `plan.md` / this file. The seam is
`decide_opponent_move(&self) -> OpponentAction`, which already has the whole
`GameState` — board-awareness reads `self.player.score()` / `self.player.stood`
with no plumbing. Keep `decide_opponent_move` the **deterministic core** (all
policy tests target it); the randomness lives in a thin `opponent_action(roll)`
wrapper that `play_opponent_turn` calls. `DEFAULT_OPPONENT` is `Basic` +
misplay 0, and the player-not-stood branch is today's behavior, so every
existing `ai_*` test stays green unchanged. Strategy + misplay are `Copy` const
fields on `OpponentProfile`; no save change (profiles rebuild from `id`).
