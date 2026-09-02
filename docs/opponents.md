# Opponents & difficulty tuning

Reference for the opponent roster and how each opponent's difficulty is
tuned. Shipped in spec 007 (`specs/007-opponent-roster/`). The authoritative
data lives in [`src/opponent.rs`](../src/opponent.rs) (the `OPPONENTS` const)
and the AI logic in [`src/game.rs`](../src/game.rs) (`decide_opponent_move`);
this file explains the *mechanism* and snapshots the *current values*.

## How the opponent plays

The opponent's whole brain is one deterministic function,
`decide_opponent_move`. On each of its turns it decides, **in this order**:

1. **Table full** (12 cards) → **Stand**.
2. **Over 20** → play the **best recovery card** (the hand card that lands it
   back on the highest total ≤ 20), or **Stand** into the bust if none fits.
3. **Can land exactly on 20** with a hand card → play it.
4. **Score ≥ its stand threshold** → **Stand**.
5. Otherwise → **Hit** (draw a dealer card).

Note what it does *not* do: it never looks at the player's board. It plays its
own hand in isolation — no "stand because I'm already ahead of you." Making it
board-aware is a deliberately deferred upgrade (see
[`ROADMAP.md`](../ROADMAP.md) → "Smarter / board-aware opponent AI").

## The two difficulty levers

Difficulty is **not** a single rating — it emerges from two independent knobs
on each opponent, which interact:

### 1. Stand threshold (`stand_threshold`)

Governs step 4 above — the "settle vs. gamble" decision in the middle zone.

- **Lower** → stands earlier → leaves points on the table → **easier** (rarely
  reaches 20).
- **Higher** → keeps hitting toward bigger totals → **harder**, but **busts
  more often** (each hit draws a 0–10 dealer card).

### 2. Side deck (`side_deck`)

The 10-card pool the opponent draws its 4-card hand from (steps 2–3 above are
entirely deck-dependent — the deck decides what it *can* play to reach 20 or
recover). Stronger decks carry:

- a wide **± range** (±1/±3/±6) for precision hitting toward 20,
- **recovery minuses** (−2/−4) to climb back down from an over-20,
- **flips** (2&4, 3&6) for board effects,
- the **tiebreaker** (±1T), which **wins otherwise-tied rounds**.

**The two combine.** A high threshold is only survivable if the deck has
recovery minuses to bail out of the busts that aggression causes — which is
why the hard opponents pair a high threshold *with* a strong recovery deck.
And the **tiebreaker is itself a difficulty lever**: an opponent without one
simply cannot win a tied round.

## Current roster

> **Snapshot of `OPPONENTS` in [`src/opponent.rs`](../src/opponent.rs) — that
> const is the source of truth.** Re-sync this table when tuning. Card labels
> below match the in-game display: `+N`/`−N` are `Plus`/`Minus`, `±N` is
> `PlusMinus`, `2&4`/`3&6` are `Flip`s, `±1T` is the `Tiebreaker`.

Ordered easiest → hardest (a test, `roster_runs_easy_to_hard_by_threshold`,
enforces the threshold ordering):

| Opponent | `id` | Label | Threshold | Side deck (10 cards) |
|---|---|---|---|---|
| **Greeb** | `greeb` | Rookie | **15** | +1 +2 +3 −1 −2 −3 +1 −1 +2 −2 |
| **Vessa Korr** | `vessa` | Scrapper | **16** | +2 +4 −2 −4 ±1 ±2 +1 −1 +3 −3 |
| **Old Toran** | `toran` | Veteran | **17** | +2 +4 −2 −4 ±1 ±3 ±6 2&4 3&6 ±1T |
| **Rix Vandal** | `rix` | Ace | **18** | ±6 ±3 ±1 −4 −2 +4 +2 2&4 3&6 ±1T |
| **The Magistrate** | `magistrate` | Master | **19** | ±6 **±6** ±3 ±1 −4 **−4** −2 2&4 3&6 ±1T |

What the gradient does:

- **Greeb** — threshold 15 and a deck of only small plain +/− (no ±, no flips,
  **no tiebreaker**). Stands early, can't reach 20 easily, can't win ties. The
  pushover.
- **Vessa Korr** — a small step up: bigger values (up to 4), two ± cards, but
  still no tiebreaker or flips.
- **Old Toran** — the **baseline**: threshold 17 and the standard side deck
  (identical to the player's deck and to the pre-roster "Opponent"). The "par"
  reference point.
- **Rix Vandal** — threshold 18 (hits at 17) with a genuinely strong deck: full
  ± range, recovery minuses, both flips, and the tiebreaker.
- **The Magistrate** — threshold 19 (hits right at 18) with the strongest deck:
  **doubled ±6 and −4**, so it very often holds both a big swing and a recovery
  card, which is what makes hitting to the edge survivable for it.

### The default opponent

`DEFAULT_OPPONENT` (`id: "default"`) is a neutral profile — threshold 17, the
standard deck, name "Opponent" — used when no roster opponent applies:
`GameState::new()`/`Default`, tests, and the fallback for a save whose
opponent id is unknown or predates the roster. It is **deliberately not in
`OPPONENTS`**, so it's never itself a selectable choice; behaviorally it equals
Old Toran.

## Tuning

All of the above is **data**, changed by editing the `OPPONENTS` const in
[`src/opponent.rs`](../src/opponent.rs) — no logic changes needed to rebalance,
add, or remove an opponent. Guards that keep the roster honest:

- `every_roster_deck_can_fill_a_hand` — each deck has ≥ `HAND_SIZE` (4) cards.
- `roster_ids_are_unique_and_names_nonempty` — ids are unique, names non-empty.
- `roster_runs_easy_to_hard_by_threshold` — thresholds are non-decreasing in
  roster order.
- `opponents_deal_from_their_own_deck_at_start_and_rematch` (in `game.rs`) —
  an opponent actually deals *its* deck, not the default pool.

A dedicated **balance/tuning pass** — playtesting the spread and adjusting
these numbers — is a tracked cross-cutting item in the campaign epic (see
[`ROADMAP.md`](../ROADMAP.md)). The values here are a reasonable first cut, not
a finished curve.

## See also

- [`src/opponent.rs`](../src/opponent.rs) — the roster (source of truth).
- [`src/game.rs`](../src/game.rs) — `decide_opponent_move` (the AI).
- [`ROADMAP.md`](../ROADMAP.md) — the board-aware-AI upgrade path and the
  balance pass.
- [`DECISIONS.md`](../DECISIONS.md) — why a campaign/progression layer exists;
  campaign design decisions.
