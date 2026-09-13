# Opponents & difficulty tuning

Reference for the opponent roster and how each opponent's difficulty is
tuned. The roster shipped in spec 007 (`specs/007-opponent-roster/`); the
**board-aware AI, per-opponent strategies, and the misplay seam** in spec 010
(`specs/010-smarter-opponents/`); and it **grew to ten opponents across an
eight-world campaign** in spec 011 (`specs/011-roster-and-worlds/`). Every value
below was **tuned against measured win rates** in spec 022
(`specs/022-balance-pass/`) — see [`balance.md`](balance.md) for the simulator,
the targets, and the curve. The authoritative data lives in
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
enforces the threshold ordering). Since spec 011 the roster pairs **contrasting
personalities within a threshold tier** — difficulty rises by threshold, while
the `AiStrategy` archetype varies the play within a tier. Since spec 022's
tuning the **top four all sit at threshold 19** (see the note under the table):

| Opponent | `id` | Label | Threshold | Strategy | Misplay | Side deck (10 cards) |
|---|---|---|---|---|---|---|
| **Greeb** | `greeb` | Rookie | **15** | Cautious | **0.18** | +1 +1 +1 +1 +1 −1 −1 −1 −1 −1 |
| **Dax Runo** | `dax` | Greenhorn | **15** | Aggressive | 0.16 | +1 +1 +1 +1 +1 +2 −1 −1 −1 −1 |
| **Vessa Korr** | `vessa` | Scrapper | **16** | Aggressive | 0.15 | +1 +1 +1 +1 +2 +2 −1 −1 −1 −1 |
| **Nima Sarn** | `nima` | Broker | **17** | Basic | 0.15 | +2 +3 −2 −3 −4 ±1 ±2 −1 +1 +2 |
| **Old Toran** | `toran` | Veteran | **17** | Cautious | 0.10 | +2 +4 −2 −4 ±1 ±3 ±6 2&4 3&6 ±1T |
| **Brakka** | `brakka` | Bruiser | **17** | Aggressive | 0.12 | ±6 ±3 +4 +3 +2 −2 −4 ±1 −1 2&4 |
| **Rix Vandal** | `rix` | Ace | **19** | Calculating | 0.03 | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 −1 |
| **Kesh Varn** | `kesh` | Duelist | **19** | Basic | 0.06 | ±6 ±3 ±1 +4 −4 −2 +2 −3 +3 2&4 |
| **The Magistrate** | `magistrate` | Master | **19** | Calculating | **0.0** | ±6 **±6** ±3 ±3 ±2 ±2 ±1 +4 ±1T 2&4 |
| **The Sovereign** | `sovereign` | Kingpin | **19** | Calculating | **0.0** | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 ±1T |

> **The top four share threshold 19.** Rix Vandal, Kesh Varn, The Magistrate and
> The Sovereign all stand at 19, so they also all carry the same **ante floor of
> 50** (`(19 − 14) × 10`, see [`economy.md`](economy.md)). Difficulty among them
> is carried by deck and strategy, not by the threshold: as a rule of thumb the
> balance pass found this AI *seems to peak* near an effective threshold of 18
> rather than 19 — a tuning observation, not a measured constant — so the
> opponents just below the finale must not hold both the better threshold and
> the better deck (`balance.md`, "What the tuning turned on").

What the gradient does (each opponent's blurb reflects its strategy):

- **Greeb** — threshold 15 and **Cautious** (effective 14 — he folds early),
  slipping 0.18 of the time. His weakness is the **deck**, not the slips: five
  `+1`s and five `−1`s and nothing else (no dual-sign ±, no flips, **no
  tiebreaker**), so he can land exactly 20 only from 19 and recover only from
  21. He plays his best line most
  turns and still loses — weak rather than random. The pushover you learn to
  beat.
- **Vessa Korr** — a step up: threshold 16 and **Aggressive** (effective 17;
  when she can beat you with a card she pushes for the *highest* safe total),
  misplay 0.15. Her deck is 1s and 2s only — no 3s, no ±, no flips, no
  tiebreaker — so the extra reach costs her busts she can rarely climb out of.
  A scrapper who pushes hard and busts for it.
- **Old Toran** — the deck **baseline** (the standard side deck, identical to the
  player's), but **Cautious**: he stands a point early (effective 16) and errs
  only 10% of the time. Balanced and patient.
- **Rix Vandal** — threshold 19 and **Calculating** (targets the *minimal* safe
  winning total), with a genuinely strong, fully playable deck: the full ±
  range (±1/±2/±3/±6, doubled at the top), +4/−4 recovery and a −1 — but **no
  tiebreaker** — only three of the roster hold it: Old Toran (it rides in the
  standard deck) and the two masters — and no dead flips. Errs rarely
  (0.03). An ace who counts every point.
- **The Magistrate** — threshold 19, **Calculating**, and **flawless** (misplay
  0.0): targets the minimal safe winning total, steals ties with the tiebreaker,
  and never slips. Its deck carries the full ± range (**doubled ±6, ±3 and ±2**)
  plus +4 and the tiebreaker, so it usually holds both a big swing and the exact
  card it needs. The one **2&4 flip** its guard requires is a dead card for the
  AI, which is why it measures about five points softer than the Sovereign.

The spec-011 additions — each the contrasting tier-mate of one above:

- **Dax Runo** (15, **Aggressive**, 0.16) — a reckless greenhorn beside naive
  Greeb: an almost-all-1s, plus-leaning deck with a single +2, so he pushes for
  big totals on cards that can't reach them and busts for it.
- **Nima Sarn** (17, **Basic**, 0.15) — a tight broker, moved into Toran's
  tier: recovery-leaning cards (−2/−3/−4) and steady threshold play. Spec 022 moved
  her from 16/Cautious to 17/**Basic** so the Mid Rim opens as a wall; the blurb
  was reworded to match ("steady play, with the odd miscount").
- **Brakka** (17, **Aggressive**, 0.12) — a bruiser beside patient Toran: wide ±
  swings toward 20.
- **Kesh Varn** (19, **Basic**, 0.06) — a duelist beside calculating Rix: a
  strong ± + recovery deck, played straight. Spec 022 moved him from
  18/Aggressive to 19/**Basic** — the roster order forced the threshold up with
  Rix, and Basic keeps his *effective* threshold (and so his strength) where it
  was, while his ante floor rose 40 → 50; the blurb was reworded to match
  ("a patient duelist — holds his nerve").
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

The dedicated **balance pass** shipped as **spec 022**: every threshold,
misplay rate, strategy and deck above was tuned against measured win rates from
a headless simulator, and a subset of the curve is now pinned by ordinary tests
(`tests/balance.rs`). See [`balance.md`](balance.md) for the command, the
targets, the measured table and what to re-run after a change.

## See also

- [`src/opponent.rs`](../src/opponent.rs) — the roster + `AiStrategy` (source of
  truth).
- [`src/game.rs`](../src/game.rs) — `decide_opponent_move` + `opponent_action`
  (the AI core and its misplay seam).
- [`balance.md`](balance.md) — the simulator, the targets and the measured
  difficulty curve (spec 022).
- [`ROADMAP.md`](../ROADMAP.md) — the difficulty setting (now unblocked by
  spec 010) and the balance pass.
- [`DECISIONS.md`](../DECISIONS.md) — why a campaign/progression layer exists;
  campaign design decisions.
