# Opponents & difficulty tuning

Reference for the opponent roster and how each opponent's difficulty is
tuned. The roster shipped in spec 007 (`specs/007-opponent-roster/`); the
**board-aware AI, per-opponent strategies, and the misplay seam** in spec 010
(`specs/010-smarter-opponents/`); and it **grew to ten opponents across an
eight-world campaign** in spec 011 (`specs/011-roster-and-worlds/`). The
authoritative data lives in
[`src/opponent.rs`](../src/opponent.rs) (the `OPPONENTS` const) and the AI logic
in [`src/game.rs`](../src/game.rs) (`decide_opponent_move` + `opponent_action`);
this file explains the *mechanism* and snapshots the *current values*.

## How the opponent plays

The opponent's brain is a **deterministic core**, `decide_opponent_move`, with a
thin **randomness seam**, `opponent_action`, around it (board-aware AI shipped in
spec 010). On each of its turns it decides, **in this order**:

1. **Table full** (12 cards) → **Stand**.
2. **Over 20** → play the **best recovery card** (the hand card that lands it
   back on the highest total ≤ 20), or **Stand** into the bust if none fits.
3. **The player has stood** — their total `P` is final, so *play to beat it*:
   - already ahead and not busting (`S > P`) → **Stand**, locking in the win
     (the spec-010 fix — it used to grind its own threshold and could bust a
     round it had already won);
   - a tie (`S == P`) → **Stand** only if it *alone* holds a tiebreaker in play,
     otherwise try to pull ahead;
   - behind (`S < P`) → play a hand card that lands a **winning** total
     (`> P`, ≤ 20) if it has one, else **Hit** and chase (standing behind is a
     certain loss).
4. **The player is still live** (no final target yet) → play to its own
   **stand threshold**: land exactly on 20 with a hand card if it can; else
   **Stand** at/above the (strategy-adjusted) threshold; else **Hit**.

Then the **misplay seam**: with probability `misplay` (per-opponent, below) the
chosen move is swapped for a legal-but-worse one — over-reaching (`Stand → Hit`),
chickening out (`Hit → Stand`), or fumbling a good card (`PlayHand → Hit`). A
misplay is **bounded so it's a believable error, never a suicidal one** (spec 013):
it only fires while the position is still *open* — the player is live and the
opponent is at or under 20 — so a resolved position (player stood, or a bust to
recover) is always played straight; and a "chicken out" is capped to within
`MISPLAY_TIMID_MARGIN` (2) of the threshold, so a timid stand is ~15, never a stand
on 0. The core stays a pure function of the board (and so is fully unit-tested);
only this outer roll is random, and the default opponent's rate is `0.0`.

Round resolution the AI reasons against: closest to 20 without busting wins;
equal totals **tie unless exactly one side has a tiebreaker in play**; over 20
loses.

## The difficulty levers

Difficulty is **not** a single rating — it emerges from several independent
knobs on each opponent, which interact:

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

**Threshold and deck combine.** A high threshold is only survivable if the deck
has recovery minuses to bail out of the busts that aggression causes — which is
why the hard opponents pair a high threshold *with* a strong recovery deck.
And the **tiebreaker is itself a difficulty lever**: an opponent without one
simply cannot win a tied round.

### 3. Strategy & error rate (`strategy`, `misplay`)

New in spec 010. **Strategy** is the policy archetype that colors *how* the
opponent plays the decisions above:

- **Basic** — sensible threshold play plus the board-aware fix (stands once it's
  already beating a stood player). The baseline.
- **Aggressive** — pushes one higher (effective threshold **+1**) and, when it
  can beat a stood player with a card, takes the **highest** safe total.
- **Cautious** — stands one earlier (effective threshold **−1**), so it stops
  building its own hand before an avoidable bust; behind a *stood* player it
  still chases (hitting is its only chance to win).
- **Calculating** — targets the **minimal** safe winning total against a stood
  player (least bust-adjacent) and, uniquely, **plays the tiebreaker to steal a
  tie** — landing exactly on your total while it alone holds a tiebreaker, a
  guaranteed win the other archetypes leave on the table.

**Misplay** is the per-turn chance (`0.0`–`1.0`) that the opponent makes a legal
but suboptimal move instead of its best one — high for the rookie, ~0 for the
master. It's what keeps a learned opponent from being perfectly exploitable, and
what makes the difficulty curve *feel* human as much as it is mechanically hard.
A misplay is **bounded** (spec 013): a slip is a believable error — an early stand
within a couple of the threshold, a greedy over-hit that busts, a fumbled card —
never a *catastrophic* one, so the opponent never stands on a low total and never
concedes a round it could still contest. The rates are the same difficulty scalar
as before; only the *outcome* is floored to competent. The default opponent's rate
is `0.0`, so it (and the test harness) stays deterministic.

## Current roster

> **Snapshot of `OPPONENTS` in [`src/opponent.rs`](../src/opponent.rs) — that
> const is the source of truth.** Re-sync this table when tuning. Card labels
> below match the in-game display: `+N`/`−N` are `Plus`/`Minus`, `±N` is
> `PlusMinus`, `2&4`/`3&6` are `Flip`s, `±1T` is the `Tiebreaker`.

Ordered easiest → hardest (a test, `roster_runs_easy_to_hard_by_threshold`,
enforces the threshold ordering). Since spec 011 the roster is **two contrasting
personalities per threshold tier** — difficulty rises by threshold, while the
`AiStrategy` archetype varies the play within a tier:

| Opponent | `id` | Label | Threshold | Strategy | Misplay | Side deck (10 cards) |
|---|---|---|---|---|---|---|
| **Greeb** | `greeb` | Rookie | **15** | Basic | **0.25** | +1 +2 +3 −1 −2 −3 +1 −1 +2 −2 |
| **Dax Runo** | `dax` | Greenhorn | **15** | Aggressive | 0.22 | +4 +3 +3 +2 +2 +1 −1 −2 −3 −2 |
| **Vessa Korr** | `vessa` | Scrapper | **16** | Aggressive | 0.15 | +2 +4 −2 −4 ±1 ±2 +1 −1 +3 −3 |
| **Nima Sarn** | `nima` | Broker | **16** | Cautious | 0.15 | +2 +3 −2 −3 −4 ±1 ±2 −1 +1 +2 |
| **Old Toran** | `toran` | Veteran | **17** | Cautious | 0.10 | +2 +4 −2 −4 ±1 ±3 ±6 2&4 3&6 ±1T |
| **Brakka** | `brakka` | Bruiser | **17** | Aggressive | 0.12 | ±6 ±3 +4 +3 +2 −2 −4 ±1 −1 2&4 |
| **Rix Vandal** | `rix` | Ace | **18** | Calculating | 0.05 | ±6 ±3 ±1 −4 −2 +4 +2 2&4 3&6 ±1T |
| **Kesh Varn** | `kesh` | Duelist | **18** | Aggressive | 0.06 | ±6 ±3 ±1 +4 −4 −2 +2 −3 +3 2&4 |
| **The Magistrate** | `magistrate` | Master | **19** | Calculating | **0.0** | ±6 **±6** ±3 ±1 −4 **−4** −2 2&4 3&6 ±1T |
| **The Sovereign** | `sovereign` | Kingpin | **19** | Calculating | **0.0** | ±6 ±6 ±3 ±3 ±1 −4 −4 −2 −1 ±1T |

What the gradient does (each opponent's blurb reflects its strategy):

- **Greeb** — threshold 15, **Basic**, and it slips a quarter of the time
  (misplay 0.25). A deck of only small plain +/− (no ±, no flips, **no
  tiebreaker**): stands early, can't reach 20 easily, can't win ties. The
  pushover you learn to beat.
- **Vessa Korr** — a step up: **Aggressive** (effective threshold 17; when she
  can beat you with a card she pushes for the *highest* safe total), misplay
  0.15, with bigger values (up to 4) and two ± cards — still no tiebreaker or
  flips. A scrapper who pushes hard and busts for it.
- **Old Toran** — the deck **baseline** (the standard side deck, identical to the
  player's), but **Cautious**: he stands a point early (effective 16) and errs
  only 10% of the time. Balanced and patient.
- **Rix Vandal** — threshold 18 and **Calculating** (targets the *minimal* safe
  winning total and steals ties with the tiebreaker), with a genuinely strong
  deck: full ± range, recovery minuses, both flips, the tiebreaker. Errs rarely
  (0.05). An ace who counts every point.
- **The Magistrate** — threshold 19, **Calculating**, and **flawless** (misplay
  0.0): targets the minimal safe winning total, steals ties with the tiebreaker,
  and never slips. Its strong deck — **doubled ±6 and −4** — means it usually
  holds both a big swing and a recovery card, which is what makes hitting to the
  edge survivable for it.

The spec-011 additions — each the contrasting tier-mate of one above:

- **Dax Runo** (15, **Aggressive**, 0.22) — a reckless greenhorn beside naive
  Greeb: a plus-heavy deck and a high error rate, so he pushes for big totals and
  busts for them.
- **Nima Sarn** (16, **Cautious**, 0.15) — a tight broker beside aggressive
  Vessa: recovery-leaning cards and an early stand; folds the moment she's ahead.
- **Brakka** (17, **Aggressive**, 0.12) — a bruiser beside patient Toran: wide ±
  swings toward 20.
- **Kesh Varn** (18, **Aggressive**, 0.06) — a hair-trigger duelist beside
  calculating Rix: a strong ± + recovery deck, and he pushes the highest safe
  total.
- **The Sovereign** (19, **Calculating**, 0.0) — the **final boss**, the flawless
  Magistrate's deadlier twin. Same perfect play, but a **fully playable deck**:
  no flips (the AI never plays one), just maximal ± range, recovery, and the
  tiebreaker — so it almost always holds the exact card to hit, recover, or steal
  the tie. Guarded by `the_final_boss_is_flawless_and_fully_equipped`.

### The default opponent

`DEFAULT_OPPONENT` (`id: "default"`) is a neutral profile — threshold 17, the
standard deck, **Basic strategy, misplay `0.0`**, name "Opponent" — used when no
roster opponent applies: `GameState::new()`/`Default`, tests, and the fallback
for a save whose opponent id is unknown or predates the roster. It is
**deliberately not in `OPPONENTS`**, so it's never itself a selectable choice.
It shares Old Toran's threshold and deck, but plays **Basic** (Toran is
**Cautious**, standing a point earlier) and **never misplays** — the
deterministic baseline the AI tests build on.

## Tuning

All of the above is **data**, changed by editing the `OPPONENTS` const in
[`src/opponent.rs`](../src/opponent.rs) — no logic changes needed to rebalance,
add, or remove an opponent. Guards that keep the roster honest:

- `every_roster_deck_can_fill_a_hand` — each deck has ≥ `HAND_SIZE` (4) cards.
- `roster_ids_are_unique_and_names_nonempty` — ids are unique, names non-empty.
- `roster_runs_easy_to_hard_by_threshold` — thresholds are non-decreasing in
  roster order.
- `misplay_rates_are_valid_and_the_default_is_deterministic` — every rate is in
  `0.0..=1.0`, the master never slips, and `DEFAULT_OPPONENT` is Basic + `0.0`.
- `opponents_deal_from_their_own_deck_at_start_and_rematch` (in `game.rs`) —
  an opponent actually deals *its* deck, not the default pool.

The board-aware decision logic itself is covered by the `ai_*` tests in
[`src/game.rs`](../src/game.rs) (stand-when-ahead, chase/play-when-behind, tie
handling, per-archetype differences, and the misplay seam).

A dedicated **balance/tuning pass** — playtesting the spread and adjusting
these numbers — is a tracked cross-cutting item in the campaign epic (see
[`ROADMAP.md`](../ROADMAP.md)). The values here are a reasonable first cut, not
a finished curve.

## See also

- [`src/opponent.rs`](../src/opponent.rs) — the roster + `AiStrategy` (source of
  truth).
- [`src/game.rs`](../src/game.rs) — `decide_opponent_move` + `opponent_action`
  (the AI core and its misplay seam).
- [`ROADMAP.md`](../ROADMAP.md) — the difficulty setting (now unblocked by
  spec 010) and the balance pass.
- [`DECISIONS.md`](../DECISIONS.md) — why a campaign/progression layer exists;
  campaign design decisions.
