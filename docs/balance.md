# Balance: the simulator, the targets, and the measured curve

Reference for the difficulty & economy balance pass (spec 022,
`specs/022-balance-pass/`). It records **how the curve is measured** (a headless
simulator that drives the real engine), **what it is tuned to** (the spec's
targets and economy bounds), and **what it currently measures**. The levers are
data: the roster in [`src/opponent.rs`](../src/opponent.rs), the starter in
[`src/profile.rs`](../src/profile.rs), the constants and prices in
[`src/economy.rs`](../src/economy.rs). The harness is
[`tests/balance.rs`](../tests/balance.rs); the blow-by-blow of the tuning is
[`specs/022-balance-pass/tuning-log.md`](../specs/022-balance-pass/tuning-log.md).

## Running it

```
KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table -- --ignored --nocapture
```

- The table run is `#[ignore]`d, so `cargo test` stays fast; it is a **report,
  never an assertion** — a failing curve prints `FAIL` rather than panicking.
- `KAAZAP_SIM_N` overrides the sample size; unset it falls back to
  `DEFAULT_N = 10_000`. That is 50 pairs × 10 000 = **500 000 matches**, about
  **5 seconds** in `--release` (excluding the one-time test-target compile).
- It plays the real engine — the same match loop, the same opponent AI with its
  misplay seam, the same round and match resolution — with a scripted player on
  the player's side. Nothing about settlement or the profile is simulated: the
  economy bounds are arithmetic on the printed rates and the live constants.

### Why N = 10 000

For a proportion near ½ one run's standard error is `√(0.25/N)`, so the
difference between two independent runs has SE `√(0.5/N)` — 0.71 points at
N = 10 000 — small enough that *all 50 pairs at once* came in within about two
points on the one run that was measured (a per-pair figure has to survive being
raised to the 50th power; at N = 4000 the all-pairs check would usually fail).

**Measured agreement (T003, two independent runs, all 50 pairs): max |Δ| =
1.1 points** (`best_outer` vs the Magistrate, tied with two other pairs), with
**0 of 50 pairs over 2.5 points**. Re-checked on the shipped data: **max |Δ| =
1.5 points over 50 pairs, 2026-09-13** (`starter` vs Nima; again 0 of 50 over
2.5, and both runs `targets 8/8, coupling 1/1, bounds 5/5` — see
`specs/022-balance-pass/tuning-log.md`). `DEFAULT_N` was not doubled.

## The scripted player

A deterministic function of `(gs.player, gs.opponent)` — no randomness, no
memory. With `s = player.score()`, `p = opponent.score()`, `STAND_AT = 17`:

1. **Recover**: if `s > 20` → play the hand card/value with `s + v ≤ 20` that
   leaves the **highest** total (lowest index on ties); none → `Stand` (accept
   the bust).
2. **Reach 20**: if some hand card `can_play_as(20 − s)` → play it (first
   index; for a `±` card the sign that lands 20).
3. **Opponent stood** (`opponent.stood && !opponent.bust`):
   - `s > p` → `Stand`; `s == p` and the player alone has a tiebreaker in play
     → `Stand`;
   - else a play landing `t` with `p < t ≤ 20` → play the **smallest** such `t`;
   - else a `Tiebreaker` that lands exactly `p` while the opponent has none in
     play → play it (the tie-steal);
   - else `s < p` → `Hit` (standing behind is a sure loss); `s == p` →
     `Stand` if `s ≥ STAND_AT` else `Hit` (take the tie late, chase early).
4. **Opponent live** → `Stand` if `s ≥ STAND_AT`, else `Hit`.

Rules 1–3 mirror the AI's own scans, so the proxy is "a player as sharp as the
AI's deterministic core, with no misplays". Two **stated limitations**, both
kept as is:

- **It never plays a flip** (neither does the AI — a flip has no
  `playable_values`), so the two Mid-tier flips are dead cards for it. That is
  why no pool-best candidate carries one.
- **In rule 3 it stands on a tie at `s ≥ 17`** even when the *opponent* alone
  holds a tiebreaker in play — a sure loss a human would chase. The proxy is
  simple and deterministic by design; this is a known, accepted gap.

## The five measured decks

Card labels match the in-game display: `+N`/`−N` are `Plus`/`Minus`, `±N` is
`PlusMinus`, `2&4`/`3&6` are `Flip`s, `±1T` is the `Tiebreaker`.

| Deck | Cards | Source of truth |
|---|---|---|
| `starter` | `+1 +1 +1 +1 +2 +2 −1 −1 −1 −2` | `profile::STARTER_SIDE_DECK` |
| `standard` | `+2 +4 −2 −4 ±1 ±3 ±6 2&4 3&6 ±1T` | `card::DEFAULT_SIDE_DECK` |
| `best_outer` | `+3 +3 +3 −3 −3 −3 +2 −2 ±1 ±1` | `BEST_OUTER` (`tests/balance.rs`) |
| `best_outer_mid` | `±3 ±3 ±3 ±3 ±2 ±2 ±2 ±1 ±1 ±1` | `BEST_OUTER_MID` |
| `best_full` | `±6 ±6 ±3 ±3 ±3 ±2 ±2 ±1 ±1 ±1T` | `BEST_FULL` |

**"Best deck from a pool"** means: *a hand-built 10-card deck (copies allowed)
from that pool, the strongest of at most three candidates measured against the
pool's region, fixed as a constant in `tests/balance.rs`*. It is deliberately
**not a search** — the multisets of the Outer pool alone would need a different
tool, and what the doc is for is a human-legible "this is what a player would
build".

> The **starter spares** (`±1 +2 −2`, `profile::STARTER_SPARES`) are
> **lateral**, not upgrades: only `±1` is a card type the starter deck doesn't
> already hold. So the measured `starter` rates describe a player who does
> **not** rebuild — which is the point, since they are what a fresh profile
> actually fields.

### The alternatives that were tried

Each candidate was measured against **its own region's** opponents at
`N = 2000`; the selection rule is the highest unweighted mean over those rows.

| Pool | Candidate | Cards | Mean win % |
|---|---|---|---|
| Outer | A | `+1 +2 +2 +3 +3 −1 −2 −3 ±1 ±1` | 56.80 |
| Outer | B | `±1 ±1 ±1 ±1 +1 −1 +2 −2 +3 −3` | 55.50 |
| Outer | **C — chosen** | `+3 +3 +3 −3 −3 −3 +2 −2 ±1 ±1` | **60.07** |
| Outer + Mid | A | `+1 +2 +3 +4 −2 −4 ±1 ±2 ±3 ±3` | 60.05 |
| Outer + Mid | **B — chosen** | `±3 ±3 ±3 ±3 ±2 ±2 ±2 ±1 ±1 ±1` | **68.03** |
| Outer + Mid | C | `±1 ±1 ±2 ±2 ±3 ±3 +4 −4 +1 −1` | 62.40 |
| Full | A | `±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 ±1T` | 58.53 |
| Full | B | `±6 ±6 ±3 ±3 ±2 ±2 ±1 ±1 ±1 ±1T` | 58.67 |
| Full | C | `±6 ±6 ±6 ±3 ±3 ±2 ±2 ±1 ±1 ±1T` | 58.20 |
| Full | **none of A/B/C — chosen** | `±6 ±6 ±3 ±3 ±3 ±2 ±2 ±1 ±1 ±1T` | see below |

- `BEST_OUTER` = **C**, 3.3 points clear of the next candidate. The scripted
  player stands at 17, so a deck of 3s covers the common "reach exactly 20 from
  17" and the symmetric −3s cover bust recovery; `±1` is the fine adjustment.
- `BEST_OUTER_MID` = **B**, 8.0 points clear. Every card is dual-sign, so every
  hand covers both reaching 20 and recovering from a bust; A's fixed `+4`/`−4`
  are dead weight half the time.
- `BEST_FULL`: the three Full candidates were a statistical tie at `N = 2000`
  (spread 0.5 points against a difference SE of ≈ 0.9); a re-measurement of A
  and B at `N = 10 000` picked **A** (58.10 vs 57.40). **T004 then replaced it**
  (allowed by the plan's "T004 may replace a candidate if it finds a better
  one") with the all-± shape that won the Outer + Mid pool plus the ±6s and the
  tiebreaker — today's `±6 ±6 ±3 ±3 ±3 ±2 ±2 ±1 ±1 ±1T`, against A's
  `±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 ±1T`. It also fixed the ordering target, whose
  worst drop went from 1.9 points (against a tolerance of 2) to 0.0.

## The measured curve

> **N = 10 000, 2026-09-13** (macOS, release profile). This is the T004a final
> run in `specs/022-balance-pass/tuning-log.md`, byte-for-byte in the numbers.
> Re-sync this section and the date whenever a lever moves.

Win rate (%) for the scripted player, by opponent and deck:

| Opponent | Region | Floor | `starter` | `standard` | `best_outer` | `best_outer_mid` | `best_full` |
|---|---|---|---|---|---|---|---|
| greeb | Outer Rim | 10 | 69.0 | 79.4 | 82.9 | 88.2 | 89.0 |
| dax | Outer Rim | 10 | 60.9 | 72.1 | 75.5 | 82.3 | 83.5 |
| vessa | Outer Rim | 20 | 55.4 | 67.8 | 70.2 | 79.6 | 81.0 |
| nima | Mid Rim | 30 | 38.2 | 49.6 | 53.3 | 64.7 | 66.6 |
| toran | Mid Rim | 30 | 40.2 | 53.2 | 56.4 | 66.8 | 69.2 |
| brakka | Mid Rim | 30 | 37.3 | 50.6 | 53.6 | 66.1 | 67.7 |
| rix | Core | 50 | 29.9 | 41.0 | 43.0 | 55.4 | 58.2 |
| kesh | Mid Rim | 50 | 37.1 | 50.0 | 52.7 | 65.4 | 67.3 |
| magistrate | Core | 50 | 27.4 | 40.1 | 41.2 | 53.5 | 56.1 |
| sovereign | Core | 50 | 23.8 | 35.2 | 37.2 | 48.8 | 51.1 |

### The targets

| # | Target | Measured | |
|---|---|---|---|
| T1 | starter vs Greeb ≥ 65 % | **69.0** | PASS |
| T2 | starter vs each Outer Rim opponent ≥ 50 % | min **55.4** (vessa) | PASS |
| T3 | starter vs each Mid Rim opponent < 50 % | max **40.2** (toran) | PASS |
| T4 | starter vs each Core opponent < 33 % | max **29.9** (rix) | PASS |
| T5 | `best_outer` vs each Outer Rim opponent ≥ 55 % | min **70.2** (vessa) | PASS |
| T6 | `best_outer_mid` vs each Mid Rim opponent ≥ 50 % | min **64.7** (nima) | PASS |
| T7 | `best_full` vs each Core opponent ≥ 45 %; Sovereign 45–55 % | min **51.1** (sovereign), sovereign **51.1** | PASS |
| T8 | ordering `outer ≤ outer_mid ≤ full` (tol 2); Sovereign hardest | worst drop **0.0**; sovereign **51.1** vs hardest other **56.1** (magistrate) | PASS |
| C | B4 coupling: best `EV_m`@floor vs 2·`EV_g` | **15.4** (kesh) vs **7.6**; @2×floor **30.9** | PASS |

Notes on how T8 and C are actually evaluated:

- **T8's `TOL = 0.02` applies per adjacent pair** (`outer → outer_mid` and
  `outer_mid → full`), **not end to end**, and the *same* tolerance is applied
  to the "Sovereign is hardest" half (`sovereign ≤ hardest_other + TOL`). The
  printed `hardest other` is the **lowest** full-pool rate among the other nine
  opponents — the hardest opponent the Sovereign must out-hard.
- **T8 is thin on some rows.** Every adjacent gap on the table above is
  positive (worst drop 0.0), but the smallest are Greeb **+0.8** and Dax
  **+1.2** from `outer_mid` to `full` (on T005's table the thinnest was Dax at
  +0.9). Those rows can invert on a re-run, in which case T8 passes on the
  `TOL` allowance (0.02, i.e. two percentage points — printed as "tol 2")
  rather than on a strict ordering. The cause is
  structural: the full pool is a superset, but `±6`s buy less for this scripted
  player than another `±3` would.
- **The `C` line and bound B4 are related but not identical.** `C` is the EV
  form — the **maximum** `EV_m` at the floor over *all* Mid Rim opponents —
  while **B4** is the ceil'd match-count form, taken over Mid Rim opponents
  **filtered to `w > 0.5`** and reported as the *minimum* `k_bet`. They agree
  whenever `EV_g > 0`, but B4's `@2×floor` count is the **floor-optimal
  opponent's**, not the minimum over opponents (a one-match edge). T004's stop
  condition reads the `C` line; T005's reads B4.

### The economy bounds

Live constants the bounds are computed from: `SEED_PURSE` **50**, reserve
(`cheapest_floor` of a fresh run) **10**, `P_outer` **20**, `P_mid` **100**,
`P_core` **200**.

| # | Bound | Arithmetic | Measured | |
|---|---|---|---|---|
| B1 | first Outer card within 5 Greeb floor matches | `P_outer + reserve = 30 ≤ SEED_PURSE = 50` | **k = 0** (already affordable) | PASS |
| B2 | first Mid card **not** affordable after one Outer Rim clear, but grindable | `50 + (10 + 10 + 20) = 90` vs `P_mid + reserve = 110`; the 20-credit gap at `EV_g = 3.8`/match | **90 vs 110**, grind **k = 6** | PASS |
| B3 | Core card by Greeb grinding ≥ 20 matches | `need = P_core + reserve − SEED = 200 + 10 − 50 = 160`; `⌈160 / 3.8⌉` | **k_grind = 43** | PASS |
| B4 | Core card by Mid Rim betting in < half as many | `⌈160 / EV_m⌉` at Kesh's floor 50, against `k_grind / 2 = 21.5` | **k_bet = 11** (kesh), **6** at 2×floor | PASS |
| B5 | ruin in a small number of floor losses; a staked win never leaves you broke | `⌈SEED_PURSE / floor_greeb⌉ = ⌈50 / 10⌉` | **k_ruin = 5** | PASS |

`EV_g` is the starter's expected credits per Greeb match at the floor:
`(2w − 1) × floor = (2 × 0.690 − 1) × 10 = 3.8`. B5's second half is spec 021's
pinned property (`credits ≥ win_payout(stake) ≥ 2 × floor`), cited rather than
re-derived. **Summary line of the recorded run: `targets 8/8, coupling 1/1,
bounds 5/5`.**

## The guards

The ignored table run is a report, so a subset of the curve is also pinned by
**ordinary `cargo test` tests** at `GUARD_N = 600` (about 15 000 matches all
told, seconds in debug), with generous margins so they can't be flaky:

| Guard | Bar | Target it protects | Margin |
|---|---|---|---|
| `starter_deck_beats_greeb_above_the_floor` | starter vs Greeb ≥ **50 %** | T1 (65 %) | **10.1 SE** (`w = .690`) |
| `starter_deck_cannot_credibly_take_the_core` | starter vs each Core opponent ≤ **50 %** | T4 (< 33 %) | **10.8 SE** (binding Rix, `w = .299`; magistrate 12.4, sovereign 15.1) |
| `the_full_pool_deck_outperforms_the_starter_against_every_opponent` | `best_full` > `starter`, every opponent | T8's ordering | **8.8 SE** (binding Greeb, gap `.890 − .690 = .200`; every other opponent ≥ 9.0 SE) |
| `default_profile_plays_a_valid_outer_tier_starter` (`profile.rs`) | every starter card is Outer tier | the starter's tiered-ness | exact — no sampling |

Margins are sized from the **standard error at `GUARD_N`** — single rate
`(w − bound) / √(w(1−w)/N)`, difference `gap / √((w₁(1−w₁) + w₂(1−w₂))/N)` —
not from the run-to-run spread of the N = 10 000 table. The smallest quoted
margin is 8.8 SE — the full-vs-starter guard against Greeb — which puts a false
failure well under 1e-6 per guard, so `GUARD_N` stays at 600. No guard covers
the **Mid Rim**, because the thinnest margin there is too thin to be safe:
against a 50 % bar, Toran (`w = .402`) is only **4.9 SE**, under the 5 SE bar
(Nima 6.0, Brakka 6.4, Kesh 6.5). The Core guard iterates only rix, magistrate
and sovereign.

## What the tuning turned on

Observations from the tuning iterations — **rules of thumb, not measured
constants**; they are what to reach for first when re-tuning, not numbers to
quote:

1. **Opponent strength is mostly *playable hand coverage*.** The AI's strongest
   branch plays the card that lands exactly 20, so a deck's value is how often a
   4-card hand holds the value it needs. A dead card (a flip, which the AI never
   plays) seems to be worth roughly **4–5 win-rate points**; the grounded figure
   behind that is the Magistrate/Sovereign gap — same threshold, same
   flawless play, one dead flip — which reads **56.1 vs 51.1** for `best_full`
   on the table above. Narrowing a deck to ±1/±2 buys about as much as a
   threshold step.
2. **The AI seems to peak near an effective threshold of 18.** At 19 it busts
   more than the higher total wins back, so the Sovereign's mandatory maximum
   threshold is a handicap and its edge has to come from its deck and its
   flawless play. Consequence for the roster: opponents just below the finale
   must not get **both** the best deck and the better threshold — Rix moved to
   19 and gave up the tiebreaker for that reason, which forced Kesh to 19 too
   (the roster order puts Rix first).
3. **The Outer Rim's weakness lives in its decks, not in `misplay`.** All-1s
   decks (mostly `+1`/`−1`, and no `±` card) give the AI almost no exact-20
   coverage, so it plays its best line nearly every turn and still loses —
   which reads as *weak* rather than *random*. That let the slip rates drop
   by more than half (Greeb 0.44 → 0.18, where 0.44 was the mid-tuning peak
   and 0.25 the value shipped before this pass) for no measurable win-rate
   change.
4. **The Mid/Core prices are the economy's real lever.** B2 fails by arithmetic
   at `P_mid = 50` (a clean Outer clear leaves 90 against 60); `P_mid = 100` and
   `P_core = 200` fixed B2 and widened B3 without touching a single ante floor,
   an EV, or the `C` line.

## Re-running after a change

1. Edit the data — `OPPONENTS` (`opponent.rs`), `STARTER_SIDE_DECK` /
   `STARTER_SPARES` (`profile.rs`), the constants or `card_price`
   (`economy.rs`), or a pool-best candidate (`tests/balance.rs`).
2. Run the command at the top and read the `targets` / `bounds` block. The stop
   condition is `targets 8/8, coupling 1/1, bounds 5/5`.
3. Then update, in this order:
   - **this file** — the measured-curve table, the targets and bounds tables,
     the guards table and the margin paragraph under it, the date and `N`, and
     any rule of thumb the run overturns;
   - **`specs/022-balance-pass/tuning-log.md`** — append the iteration (levers
     old → new, the verdict block) so the next pass starts from data;
   - **[`docs/opponents.md`](opponents.md)** — the roster table and the prose
     for any opponent whose values moved;
   - **[`docs/economy.md`](economy.md)** — the constants, ante and tier/price
     tables;
   - **the guard margin block in [`tests/balance.rs`](../tests/balance.rs)** —
     recompute the SEs from the new table; if any margin falls under 5 SE,
     raise `GUARD_N` rather than loosening a bar.
4. Finally `cargo test` — the guards are what catch a curve that quietly broke.

## See also

- [`tests/balance.rs`](../tests/balance.rs) — the simulator, the scripted
  player, the targets/bounds arithmetic, and the guards.
- [`specs/022-balance-pass/tuning-log.md`](../specs/022-balance-pass/tuning-log.md)
  — every iteration and every verbatim run.
- [`docs/opponents.md`](opponents.md) — the roster and the difficulty levers.
- [`docs/economy.md`](economy.md) — the constants the bounds are computed from.
