# Plan: Roster expansion & new campaign worlds

Technical design for `spec.md`. The master plan
(`~/.claude/plans/iterative-meandering-blum.md`) mirrors this. Pure content over
the spec 007/009/010 patterns — no engine, AI, save, or card-type change.

## What the code already gives us

- **A world is one `const` struct** — `Planet { id, name, region, blurb, fx, fy,
  opponents: &'static [&'static str], requires: &'static [&'static str] }` in
  `campaign.rs`'s `PLANETS`. Routes, unlock/clear (`planet_unlocked` /
  `planet_cleared` / `next_opponent`), the cursor, and the cleared-count header
  are **all derived** from `requires` + the `beaten` set — no logic changes.
- **An opponent is one `const` struct** — `OpponentProfile { id, name, difficulty,
  blurb, stand_threshold, side_deck, strategy, misplay }` in `opponent.rs`'s
  `OPPONENTS`; planets reference opponents by `id` (`opponent_by_id`).
- **The only manual burden is map legibility** — `fx`/`fy` (normalized 0..1) are
  hand-authored, `CampaignMapLayout::node_pos` (`layout.rs`) maps them to cells,
  and **nothing checks overlap**. Field is ~76×23 at the 89×31 minimum; a
  cursored label (`▸ NAME ◂`, uppercased, `draw_text_centered`) is ~name+4 cols.

## Roster — `src/opponent.rs`

Grow `OPPONENTS` 5 → 10; insert the five new entries so `stand_threshold` stays
**non-decreasing in array order** (`roster_runs_easy_to_hard_by_threshold`).
Mapping (existing rows unchanged):

| id | tier | strategy | misplay | deck character |
|---|---|---|---|---|
| `dax` | 15 | Aggressive | ~0.22 | small plain ± (like Greeb), no tiebreaker |
| `nima` | 16 | Cautious | ~0.15 | a couple of ±, a recovery minus |
| `brakka` | 17 | Aggressive | ~0.12 | wider ±, some recovery |
| `kesh` | 18 | Aggressive | ~0.06 | strong ± + recovery + a flip |
| `sovereign` | 19 | Calculating | 0.0 | **most flexible**: max ± range + recovery + tiebreaker |

- Decks are 10-card `&'static [Card]` literals from `card::ALL_SIDE_CARDS`
  (repeats fine). Each ≥ `HAND_SIZE`. `every_roster_card_is_in_the_universe` and
  `every_roster_deck_can_fill_a_hand` guard these.
- **`sovereign` (final boss)**: same threshold 19 / `Calculating` / misplay 0 as
  the Magistrate, but a strictly more flexible deck (heavy ±1/±3/±6, `-4`/`-2`
  recovery, the `Tiebreaker`, at most one flip) so it near-always holds the exact
  card to hit, recover, or steal a tie — the one lever for "tougher" without an
  engine change. `difficulty` label above "Master".
- Names/labels/blurbs: original flavor in the existing register, tuned here.

## Worlds — `src/campaign.rs`

Rewrite `PLANETS` to 8 worlds (a stretched diamond). Existing four keep their
opponents; four new worlds join.

| world id | region | opponents | requires |
|---|---|---|---|
| `cinder` | Outer Rim | `greeb` | — |
| *(new, e.g. `scree`)* | Outer Rim | `dax` | `cinder` |
| `ashfall` | Outer Rim | `vessa` | `cinder` |
| *(new)* | Mid Rim | `nima` | *(new W2)* |
| `drift` | Mid Rim | `toran` | `ashfall` |
| *(new)* | Mid Rim | `brakka`, `kesh` | *(new W4)*, `drift` *(rejoin)* |
| `the-spindle` | Core | `rix`, `magistrate` | *(new W6)* |
| *(new, boss)* | Core | `sovereign` | `the-spindle` |

- **Topology:** one fork off `cinder` into two two-world lanes (W2→W4 and
  ashfall→drift) that rejoin at W6, then linear W6→spindle→boss. Only `cinder`
  has empty `requires`. Difficulty-monotonic on every path; both lanes required,
  so all 10 opponents are played.
- **Positions:** `fx` 0.06 → ~0.88 rim→core; lanes at `fy≈0.30`/`0.70`; the Core
  spine (W6/spindle/boss) **staggered in `fy`** so no two share a label row. Boss
  name short + `fx` capped so its cursored label doesn't clip the right edge.
  Final values are tuned to pass the new legibility test.
- The Spindle keeps `[rix, magistrate]`, so
  `clearing_a_planet_needs_all_its_opponents` still passes unchanged.

## Tests

- **Rewrite** `the_fork_unlocks_both_mid_worlds_and_the_core_needs_both` and
  `navigation_stays_on_unlocked_planets` for the new DAG.
- **Generalize** `mark_beaten_is_idempotent_and_a_full_sweep_completes_the_run`
  to derive the sweep from `PLANETS` (beat every opponent of every planet), so it
  survives future content.
- **New — map legibility** (in the module owning `node_pos`, at the 89×31 min):
  node cells pairwise-unique; each planet's cursored-label span within the field;
  no two planets sharing a label row have overlapping label spans.
- **New — graph integrity** (`campaign.rs`): exactly one start node; the DAG is
  acyclic; every planet is reachable from the start by a valid clear order.
- Pass automatically: `planets_are_well_formed`, `planet_ids_are_unique`,
  `a_fresh_run_unlocks_only_the_start` (all new worlds have non-empty `requires`),
  `campaign_map_layout_fits_the_minimum_terminal`, and the opponent guards.

## Files

- `src/opponent.rs` — 5 new `OPPONENTS` entries + decks (threshold-sorted).
- `src/campaign.rs` — rewrite `PLANETS`; updated + new tests.
- `src/layout.rs` **or** `src/campaign_map.rs` — the new legibility test only.
- Close-out docs: `docs/opponents.md`, README, `ROADMAP.md`, `DECISIONS.md`.
- **No change:** `game.rs`/`player.rs`/`card.rs`, `save.rs`/`profile.rs` format,
  `app.rs`, `opponent_select.rs` (lists `OPPONENTS` automatically).

## Verification

- `cargo build` (no new warnings) + `cargo test` (green; report verbatim).
- Driver sweep (`run-kaazap` driver, backing up real `profile.json`/`saves/`):
  the 8-world map renders legibly (nodes/labels, fork + rejoin); navigate the
  unlock order; play new opponents incl. the boss (board-aware AI, win→map
  progress, no panics); snapshot Quick Play at 89×31 to confirm the 10-item list
  fits (tighten spacing only if it overflows).
