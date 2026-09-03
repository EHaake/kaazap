# Spec: Roster expansion & new campaign worlds

## Summary

The campaign (spec 009) is a **4-world vertical slice** — a Cinder →
{Ashfall, Drift} → The Spindle diamond — that uses **all five** roster opponents
(spec 007), so it can't grow without more opponents. Spec 010 (board-aware AI)
made the roster's five personalities mechanically real and, with four
`AiStrategy` archetypes plus a misplay rate, opened room for **two opponents on
the same difficulty to play genuinely differently**.

This spec roughly **doubles the campaign**: **5 → 10 opponents** and **4 → 8
worlds**, ending in a **new final boss tougher than the Magistrate**. It is pure
content built on the spec 007/009/010 patterns — no engine, AI-logic, save, or
new card-type work.

## Goals

1. **Ten opponents, two per difficulty tier.** Difficulty still rises by stand
   threshold (15–19), but each tier now holds **two contrasting personalities**
   (different `AiStrategy` / deck / misplay), so the roster feels varied rather
   than a single rising line. The new final boss is the flawless top.
2. **Eight worlds.** A larger star map in the same **light-branch-that-rejoins**
   shape players already know (spec 009), Outer Rim → Core, ending at the boss.
   The four existing worlds keep their opponents.
3. **A real difficulty ceiling.** A climactic Core boss beyond the Magistrate:
   same flawless play, the nastiest deck — the campaign's summit.
4. **Legible at a glance.** The bigger map still reads clearly at the minimum
   terminal — no overlapping nodes or labels, the fork and rejoin obvious.
5. **No regression.** Every existing rule, screen, save, and the whole match flow
   are unchanged; old profiles load and simply show the new worlds as locked.

## Key behavior

### The roster (Quick Play + the campaign)

Ten selectable opponents instead of five. Each still has a name, a difficulty
label, a one-line blurb, a stand threshold, a side deck, an `AiStrategy`, and a
misplay rate (all from spec 007/010). The five newcomers pair off with the
existing five by tier:

- **Tier 15** — Greeb (naive, slips) **·** a cocky greenhorn who over-pushes
  (Aggressive).
- **Tier 16** — Vessa (aggressive scrapper) **·** a tight broker who folds early
  (Cautious).
- **Tier 17** — Old Toran (patient veteran, Cautious) **·** a bruiser who swings
  for 20 (Aggressive).
- **Tier 18** — Rix (precise ace, Calculating) **·** an aggressive duelist.
- **Tier 19** — The Magistrate (flawless master) **·** **the new final boss**:
  Calculating, misplay 0, the most flexible deck in the game.

Names, labels, blurbs, and exact decks are original, IP-safe flavor (no Star Wars
trademarks) tuned during implementation.

### The map

Eight worlds, Outer Rim → Core: the start forks into two two-world lanes that
**rejoin** at a Mid-Rim hub, then a linear Core run to The Spindle and finally
the boss's world. Clearing a lane needs both its worlds, and the rejoin needs
both lanes — so the whole roster is played, as today. Unlocking, clearing,
routes, the "next" highlight, the cleared-count header, and save/resume all work
exactly as in spec 009 (they're derived from the graph + the beaten set).

## Non-goals (explicitly deferred)

- **The economy/rewards** (credits, shop, card drops) — spec C. Wins still record
  progress only.
- **The two-panel "briefcase" deck-builder** — a later presentation pass.
- **Any engine, AI-logic, or save-format change**, and **any new card type** —
  the 15-card universe is complete; new opponents get new *decks* from it.
- **Map-layout code changes** — positions are data; only the hand-authored
  `fx`/`fy` values (and a new legibility test) change.

## Acceptance criteria

- [ ] `OPPONENTS` has 10 entries, thresholds non-decreasing, ids unique, every
      deck fillable and drawn from the card universe, misplay in range — the new
      boss is threshold 19 / Calculating / misplay 0 with a deck at least as
      strong as the Magistrate's.
- [ ] `PLANETS` has 8 worlds forming a single connected DAG: exactly one start
      (empty `requires`), the fork + rejoin as designed, all `opponents`/`requires`
      ids valid, and the whole roster reachable.
- [ ] The map is legible at the 89×31 minimum: no two nodes share a cell, and no
      planet's label collides with another or clips off-field — enforced by a
      test, not by eye.
- [ ] Navigating and clearing the campaign unlocks worlds in the intended order;
      a fresh profile unlocks only the start; a full sweep completes the run;
      save/resume and win→map progress are unchanged.
- [ ] Quick Play's 10-opponent select list still renders within the minimum
      terminal.
- [ ] `cargo test` green (updated topology tests + new legibility/graph-integrity
      tests + roster guards), `cargo build` no new warnings, no panics in play.

## Resolved decisions

- **Substantial scale** (human-ruled) — +5 opponents, +4 worlds (10 and 8 total),
  over a modest top-up or a large one, as the sweet spot that meaningfully grows
  the campaign while staying legible on the map and quick to balance.
- **Raise the ceiling** (human-ruled) — a new final boss tougher than the
  Magistrate, rather than only broadening the roster under the existing peak.
  Since thresholds cap at 19 and Calculating/misplay-0 is already optimal, "tougher"
  is delivered by a strictly more flexible **deck** — the one lever that doesn't
  require an engine change.
- **Two personalities per tier, differentiated by strategy** — leans on spec
  010's archetypes so the roster grows in breadth without pushing thresholds past
  the sensible 15–19 band.
- **Keep the light-branch-that-rejoins shape** (spec 009's approved design),
  scaled to two-world lanes, rather than a more branching "pick your route" map —
  familiar and legible.
