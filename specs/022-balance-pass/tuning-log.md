# Spec 022 — tuning log

Measurements for the balance pass. T003 records the baseline; T004/T005 append
their iterations. All figures come from the documented command:

```
cargo test --release --test balance balance_table -- --ignored --nocapture
```

Machine: macOS (Darwin 25.6.0), release profile, single-threaded simulator.

## T003 — baseline

### Timing

The release test target was built once first (`cargo test --release --test
balance --no-run`), so both wall times below **exclude the compile** — they are
the measurement only.

| run | wall time (`time`, real) | sample size |
| --- | --- | --- |
| run 1 | **4.921 s** | `DEFAULT_N` = 10 000 (500 000 matches) |
| run 2 | **4.786 s** | `DEFAULT_N` = 10 000 (500 000 matches) |

Well under the 60 s the plan's "finishes in seconds" claim is checked against.
`DEFAULT_N` was **not** doubled: no pair needed it (see below).

### Run 1 (verbatim)

```

running 1 test

deck            opponent          n    win%   ev/match@floor
starter         greeb         10000    62.7             +2.5
starter         dax           10000    55.6             +1.1
starter         vessa         10000    46.3             -1.5
starter         nima          10000    56.5             +2.6
starter         toran         10000    50.1             +0.1
starter         brakka        10000    47.9             -1.2
starter         rix           10000    45.9             -3.3
starter         kesh          10000    46.6             -2.7
starter         magistrate    10000    45.5             -4.5
starter         sovereign     10000    34.6            -15.4
standard        greeb         10000    64.0             +2.8
standard        dax           10000    57.6             +1.5
standard        vessa         10000    48.6             -0.5
standard        nima          10000    57.9             +3.2
standard        toran         10000    53.7             +2.2
standard        brakka        10000    50.3             +0.2
standard        rix           10000    48.7             -1.1
standard        kesh          10000    50.3             +0.2
standard        magistrate    10000    49.0             -1.0
standard        sovereign     10000    37.7            -12.3
best_outer      greeb         10000    65.5             +3.1
best_outer      dax           10000    58.8             +1.8
best_outer      vessa         10000    50.5             +0.2
best_outer      nima          10000    58.7             +3.5
best_outer      toran         10000    52.7             +1.6
best_outer      brakka        10000    50.7             +0.4
best_outer      rix           10000    47.3             -2.2
best_outer      kesh          10000    50.6             +0.5
best_outer      magistrate    10000    48.2             -1.8
best_outer      sovereign     10000    37.5            -12.5
best_outer_mid  greeb         10000    71.7             +4.3
best_outer_mid  dax           10000    66.1             +3.2
best_outer_mid  vessa         10000    56.9             +2.8
best_outer_mid  nima          10000    65.3             +6.1
best_outer_mid  toran         10000    61.2             +6.7
best_outer_mid  brakka        10000    58.1             +4.9
best_outer_mid  rix           10000    55.3             +4.3
best_outer_mid  kesh          10000    58.6             +6.8
best_outer_mid  magistrate    10000    55.9             +5.9
best_outer_mid  sovereign     10000    44.9             -5.1
best_full       greeb         10000    74.6             +4.9
best_full       dax           10000    70.4             +4.1
best_full       vessa         10000    61.6             +4.6
best_full       nima          10000    69.2             +7.7
best_full       toran         10000    65.8             +9.5
best_full       brakka        10000    64.2             +8.5
best_full       rix           10000    61.7             +9.3
best_full       kesh          10000    64.1            +11.3
best_full       magistrate    10000    62.5            +12.5
best_full       sovereign     10000    51.8             +1.8
targets
  T1 starter vs greeb >= 65%                                          62.7  FAIL
  T2 starter vs each Outer Rim opponent >= 50%                        min 46.3 (vessa)  FAIL
  T3 starter vs each Mid Rim opponent < 50%                           max 56.5 (nima)  FAIL
  T4 starter vs each Core opponent < 33%                              max 45.9 (rix)  FAIL
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 50.5 (vessa)  FAIL
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 58.1 (brakka)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.8 (sovereign), sovereign 51.8  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.8 vs next 61.6 (vessa)  PASS
  C  B4 coupling (T004): best EV_m@floor 6.8 (kesh) vs 2·EV_g 5.1  PASS; @2×floor 13.7  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 50, P_core 120)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 60); grind k=0  FAIL
  B3 Core card by Greeb grind >= 20 matches                           k_grind=32  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=12 (toran), @2×floor=6  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 3/8, coupling 1/1, bounds 4/5
test balance_table ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 4.76s

```

```
cargo test --release --test balance balance_table -- --ignored --nocapture >   4.73s user 0.09s system 97% cpu 4.921 total
```

### Run 2 (verbatim)

```

running 1 test

deck            opponent          n    win%   ev/match@floor
starter         greeb         10000    62.0             +2.4
starter         dax           10000    55.5             +1.1
starter         vessa         10000    46.9             -1.2
starter         nima          10000    56.9             +2.8
starter         toran         10000    50.7             +0.4
starter         brakka        10000    48.2             -1.1
starter         rix           10000    45.3             -3.8
starter         kesh          10000    47.2             -2.2
starter         magistrate    10000    45.3             -4.7
starter         sovereign     10000    34.0            -16.0
standard        greeb         10000    63.7             +2.7
standard        dax           10000    58.2             +1.6
standard        vessa         10000    48.5             -0.6
standard        nima          10000    57.2             +2.9
standard        toran         10000    53.1             +1.9
standard        brakka        10000    50.4             +0.3
standard        rix           10000    48.7             -1.0
standard        kesh          10000    49.4             -0.5
standard        magistrate    10000    49.5             -0.5
standard        sovereign     10000    37.6            -12.4
best_outer      greeb         10000    65.2             +3.0
best_outer      dax           10000    58.7             +1.7
best_outer      vessa         10000    50.6             +0.2
best_outer      nima          10000    58.8             +3.5
best_outer      toran         10000    53.5             +2.1
best_outer      brakka        10000    51.8             +1.1
best_outer      rix           10000    48.0             -1.6
best_outer      kesh          10000    50.8             +0.6
best_outer      magistrate    10000    47.1             -2.9
best_outer      sovereign     10000    37.6            -12.3
best_outer_mid  greeb         10000    70.9             +4.2
best_outer_mid  dax           10000    66.1             +3.2
best_outer_mid  vessa         10000    57.3             +2.9
best_outer_mid  nima          10000    65.9             +6.4
best_outer_mid  toran         10000    60.9             +6.5
best_outer_mid  brakka        10000    59.1             +5.5
best_outer_mid  rix           10000    55.2             +4.2
best_outer_mid  kesh          10000    59.0             +7.2
best_outer_mid  magistrate    10000    56.5             +6.6
best_outer_mid  sovereign     10000    44.9             -5.1
best_full       greeb         10000    75.6             +5.1
best_full       dax           10000    70.7             +4.1
best_full       vessa         10000    61.7             +4.7
best_full       nima          10000    69.4             +7.8
best_full       toran         10000    66.2             +9.7
best_full       brakka        10000    63.1             +7.9
best_full       rix           10000    61.1             +8.9
best_full       kesh          10000    64.2            +11.4
best_full       magistrate    10000    61.8            +11.8
best_full       sovereign     10000    51.2             +1.2
targets
  T1 starter vs greeb >= 65%                                          62.0  FAIL
  T2 starter vs each Outer Rim opponent >= 50%                        min 46.9 (vessa)  FAIL
  T3 starter vs each Mid Rim opponent < 50%                           max 56.9 (nima)  FAIL
  T4 starter vs each Core opponent < 33%                              max 45.3 (magistrate)  FAIL
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 50.6 (vessa)  FAIL
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 59.0 (kesh)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.2 (sovereign), sovereign 51.2  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.2 vs next 61.1 (rix)  PASS
  C  B4 coupling (T004): best EV_m@floor 7.2 (kesh) vs 2·EV_g 4.8  PASS; @2×floor 14.4  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 50, P_core 120)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 60); grind k=0  FAIL
  B3 Core card by Greeb grind >= 20 matches                           k_grind=34  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=12 (kesh), @2×floor=6  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 3/8, coupling 1/1, bounds 4/5
test balance_table ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 4.59s

```

```
cargo test --release --test balance balance_table -- --ignored --nocapture >   4.61s user 0.08s system 98% cpu 4.786 total
```

### Agreement between the two runs (per pair)

Win-rate percentage points, all 50 pairs:

```
deck            opponent       run1   run2    |d|
starter         greeb          62.7   62.0    0.7
starter         dax            55.6   55.5    0.1
starter         vessa          46.3   46.9    0.6
starter         nima           56.5   56.9    0.4
starter         toran          50.1   50.7    0.6
starter         brakka         47.9   48.2    0.3
starter         rix            45.9   45.3    0.6
starter         kesh           46.6   47.2    0.6
starter         magistrate     45.5   45.3    0.2
starter         sovereign      34.6   34.0    0.6
standard        greeb          64.0   63.7    0.3
standard        dax            57.6   58.2    0.6
standard        vessa          48.6   48.5    0.1
standard        nima           57.9   57.2    0.7
standard        toran          53.7   53.1    0.6
standard        brakka         50.3   50.4    0.1
standard        rix            48.7   48.7    0.0
standard        kesh           50.3   49.4    0.9
standard        magistrate     49.0   49.5    0.5
standard        sovereign      37.7   37.6    0.1
best_outer      greeb          65.5   65.2    0.3
best_outer      dax            58.8   58.7    0.1
best_outer      vessa          50.5   50.6    0.1
best_outer      nima           58.7   58.8    0.1
best_outer      toran          52.7   53.5    0.8
best_outer      brakka         50.7   51.8    1.1
best_outer      rix            47.3   48.0    0.7
best_outer      kesh           50.6   50.8    0.2
best_outer      magistrate     48.2   47.1    1.1
best_outer      sovereign      37.5   37.6    0.1
best_outer_mid  greeb          71.7   70.9    0.8
best_outer_mid  dax            66.1   66.1    0.0
best_outer_mid  vessa          56.9   57.3    0.4
best_outer_mid  nima           65.3   65.9    0.6
best_outer_mid  toran          61.2   60.9    0.3
best_outer_mid  brakka         58.1   59.1    1.0
best_outer_mid  rix            55.3   55.2    0.1
best_outer_mid  kesh           58.6   59.0    0.4
best_outer_mid  magistrate     55.9   56.5    0.6
best_outer_mid  sovereign      44.9   44.9    0.0
best_full       greeb          74.6   75.6    1.0
best_full       dax            70.4   70.7    0.3
best_full       vessa          61.6   61.7    0.1
best_full       nima           69.2   69.4    0.2
best_full       toran          65.8   66.2    0.4
best_full       brakka         64.2   63.1    1.1
best_full       rix            61.7   61.1    0.6
best_full       kesh           64.1   64.2    0.1
best_full       magistrate     62.5   61.8    0.7
best_full       sovereign      51.8   51.2    0.6

max |d| = 1.1 points  (best_outer vs magistrate)   pairs over 2.5: 0 of 50
```

**Max |Δ| = 1.1 points**, tied across three pairs — `best_outer` vs
`magistrate` (48.2 / 47.1), `best_outer` vs `brakka` (50.7 / 51.8) and
`best_full` vs `brakka` (64.2 / 63.1). Zero pairs exceed 2.5 points, so
`DEFAULT_N` stays at 10 000 (no doubling; plan tension §3 satisfied, and the
spec's "agree within about two points on every pair" holds with room).

### Baseline summary line

Both runs, with the plan's candidate decks still in place:

```
summary: targets 3/8, coupling 1/1, bounds 4/5
```

Failing at baseline: T1, T2, T3, T4, T5 and B2 — T004/T005 territory. B2's
failure is the arithmetic the plan predicted in tension §8 (`90 >= 60`).

### Pool-best candidates

Three hand-built 10-card candidates per pool (copies allowed, no flips — the
scripted player never plays one), each measured against **its own region's
opponents** at `N = 2000` (`KAAZAP_SIM_N=2000`), by swapping the consts in
`tests/balance.rs`. Regions come from `PLANETS`: Outer Rim = greeb, dax, vessa;
Mid Rim = nima, toran, brakka, kesh; Core = rix, magistrate, sovereign.

**Selection rule: the highest mean win rate over the region's opponents**
(unweighted mean of the region rows).

Candidates:

| pool | cand | deck |
| --- | --- | --- |
| Outer | A (the plan's) | +1 +2 +2 +3 +3 −1 −2 −3 ±1 ±1 |
| Outer | B (symmetric, ±1-heavy) | ±1 ±1 ±1 ±1 +1 −1 +2 −2 +3 −3 |
| Outer | C (big-swing 3s) | +3 +3 +3 −3 −3 −3 +2 −2 ±1 ±1 |
| Outer+Mid | A (the plan's) | +1 +2 +3 +4 −2 −4 ±1 ±2 ±3 ±3 |
| Outer+Mid | B (all-±, full 1–3 coverage) | ±3 ±3 ±3 ±3 ±2 ±2 ±2 ±1 ±1 ±1 |
| Outer+Mid | C (± core plus fixed 4s) | ±1 ±1 ±2 ±2 ±3 ±3 +4 −4 +1 −1 |
| Full | A (the plan's) | ±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 ±1T |
| Full | B (all-±, wider small coverage) | ±6 ±6 ±3 ±3 ±2 ±2 ±1 ±1 ±1 ±1T |
| Full | C (±6-heavy) | ±6 ±6 ±6 ±3 ±3 ±2 ±2 ±1 ±1 ±1T |

Measured (win %, N = 2000):

```

best_outer  (vs greeb, dax, vessa, N=2000)
cand         greeb         dax       vessa     mean
A             63.9        58.3        48.2    56.80
B             62.3        57.0        47.2    55.50
C             67.0        61.0        52.2    60.07

best_outer_mid  (vs nima, toran, brakka, kesh, N=2000)
cand          nima       toran      brakka        kesh     mean
A             63.7        59.5        57.8        59.2    60.05
B             69.7        68.2        67.0        67.2    68.03
C             66.1        62.9        59.0        61.6    62.40

best_full  (vs rix, magistrate, sovereign, N=2000)
cand           rix  magistrate   sovereign     mean
A             61.0        62.4        52.2    58.53
B             61.3        63.2        51.5    58.67
C             62.1        61.7        50.8    58.20
```

**Chosen:**

- `BEST_OUTER` = **C** (`+3 +3 +3 −3 −3 −3 +2 −2 ±1 ±1`) — 60.07 mean, 3.3
  points clear of the plan's A. The scripted player stands at 17, so a deck
  stocked with 3s covers the common "reach exactly 20 from 17" and the
  symmetric −3s cover bust recovery; ±1 supplies the fine adjustment. Clearly
  outside the noise band (difference SE over three pooled rows ≈ 0.9 points).
- `BEST_OUTER_MID` = **B** (`±3 ±3 ±3 ±3 ±2 ±2 ±2 ±1 ±1 ±1`) — 68.03 mean, 8.0
  points clear of the plan's A. Every card is dual-sign, so every hand covers
  both reaching 20 and recovering from a bust; the fixed +4/−4 in A are dead
  weight half the time.
- `BEST_FULL` = **A**, the plan's deck (`±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 ±1T`) —
  58.53 vs B's 58.67 and C's 58.20 at N = 2000, i.e. a statistical tie (the
  spread, 0.5 points, is smaller than the difference SE of ≈ 0.9). A tie-break
  re-measurement of the two leaders at N = 10 000 resolved it the other way and
  with a larger margin: over the Core opponents A scored 61.1 / 62.8 / 50.4 =
  **58.10** against B's 60.9 / 61.5 / 49.8 = **57.40**. Under the same rule, at
  the more precise sample, A wins — and A is the plan's deck, so nothing moves.
  (This extra 10 000-match confirmation is the only step beyond the task's
  `N = 2000` protocol; it was measurement only, and it is recorded here because
  the N = 2000 ordering of A and B is not meaningful on its own.)

### Post-fix baseline (the numbers T004 starts from)

`DEFAULT_N` = 10 000, with `BEST_OUTER` = C and `BEST_OUTER_MID` = B in place
(`BEST_FULL` unchanged). Rows for the two changed decks, then the verdicts:

```
best_outer      greeb         10000    67.5             +3.5
best_outer      dax           10000    60.9             +2.2
best_outer      vessa         10000    51.9             +0.7
best_outer      nima          10000    61.6             +4.7
best_outer      toran         10000    56.5             +3.9
best_outer      brakka        10000    52.7             +1.6
best_outer      rix           10000    49.8             -0.1
best_outer      kesh          10000    53.0             +2.4
best_outer      magistrate    10000    51.1             +1.1
best_outer      sovereign     10000    39.1            -10.9
best_outer_mid  greeb         10000    75.9             +5.2
best_outer_mid  dax           10000    71.7             +4.3
best_outer_mid  vessa         10000    64.5             +5.8
best_outer_mid  nima          10000    71.1             +8.4
best_outer_mid  toran         10000    66.5             +9.9
best_outer_mid  brakka        10000    66.0             +9.6
best_outer_mid  rix           10000    61.5             +9.2
best_outer_mid  kesh          10000    65.8            +12.6
best_outer_mid  magistrate    10000    62.5            +12.5
best_outer_mid  sovereign     10000    51.4             +1.4
targets
  T1 starter vs greeb >= 65%                                          62.5  FAIL
  T2 starter vs each Outer Rim opponent >= 50%                        min 46.3 (vessa)  FAIL
  T3 starter vs each Mid Rim opponent < 50%                           max 56.7 (nima)  FAIL
  T4 starter vs each Core opponent < 33%                              max 45.4 (magistrate)  FAIL
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 51.9 (vessa)  FAIL
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.8 (kesh)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 50.6 (sovereign), sovereign 50.6  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 1.9 (vessa); sovereign 50.6 vs next 61.8 (rix)  PASS
  C  B4 coupling (T004): best EV_m@floor 12.6 (kesh) vs 2·EV_g 5.0  PASS; @2×floor 25.2  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 50, P_core 120)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 60); grind k=0  FAIL
  B3 Core card by Greeb grind >= 20 matches                           k_grind=32  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=7 (kesh), @2×floor=4  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 3/8, coupling 1/1, bounds 4/5
```

Note for T004: with the stronger Outer+Mid deck, **T8's ordering check now sits
at the edge of its tolerance** — worst drop 1.9 points (vessa) against
`TOL = 2`, and the Mid Rim rows are within 1.7 points of the full-pool deck
(nima 71.1 vs 70.2, brakka 66.0 vs 64.3, kesh 65.8 vs 64.1). It passes here,
but it can flip on a re-run. The cause is structural rather than a tuning
knob: the full pool is a superset of the Outer+Mid pool, yet `BEST_FULL`'s ±6s
and fixed +4/−4 are worse for this scripted player than a fourth ±3. A
full-pool candidate shaped like the Outer+Mid winner (all-±, ±6 and the
tiebreaker substituted in sparingly) would restore the margin — T003 is capped
at three candidates per pool (plan tension §5), so that is T004's call under
"T004 may replace a candidate if it finds a better one".

Also unchanged from the plan's prediction (tension §8): B2 still fails by
arithmetic (`90 >= 60`), and T1–T5 fail — the tuning tasks own those.

## T004 — iteration 1

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| `STARTER_SIDE_DECK` | +1 +1 +2 +2 +3 −1 −1 −2 −2 −3 | +1 +1 +1 +2 +2 +2 −1 −1 −2 −2 (no 3s) |
| `STARTER_SPARES` | +3 −3 ±1 | ±1 +2 −2 (lateral, so the starter deck is what a fresh player fields) |
| greeb `misplay` | 0.25 | 0.40 |
| greeb `side_deck` | +1 +2 +3 −1 −2 −3 +1 −1 +2 −2 | +1 +1 +1 +2 +2 −1 −1 −1 −2 −2 |
| dax `misplay` | 0.22 | 0.32 |
| vessa `misplay` | 0.15 | 0.30 |
| vessa `side_deck` | +2 +4 −2 −4 ±1 ±2 +1 −1 +3 −3 | +3 +2 +2 +1 +1 −1 −1 −2 −2 −3 |
| nima `stand_threshold` | 16 | 17 |
| nima `strategy` | Cautious | Basic |
| rix `misplay` | 0.05 | 0.03 |
| rix `side_deck` | ±6 ±3 ±1 −4 −2 +4 +2 F24 F36 T | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 −1 |
| magistrate `side_deck` | ±6 ±6 ±3 ±1 −4 −4 −2 F24 F36 T | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 T F24 |
| sovereign `side_deck` | ±6 ±6 ±3 ±3 ±1 −4 −4 −2 −1 T | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 T |
| `BEST_FULL` | ±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 T | ±6 ±6 ±3 ±3 ±3 ±2 ±2 ±1 ±1 T |

Reasoning: the AI's strongest branch is "play the card that lands exactly 20",
so opponent strength tracks *playable* hand coverage — which is why the
Sovereign's two dead flips were worth ~11 points. The Core masters get
full ±1/±2/±3/±6 + 4 coverage (the Magistrate keeps one flip, as its guard
requires); the starter loses its 3s (the 17 → 20 card) and the Outer Rim is
softened to compensate. `BEST_FULL` is re-shaped to the all-± form that won
the Outer+Mid pool, which also fixes T8's 1.9-point ordering drop.

```
targets
  T1 starter vs greeb >= 65%                                          63.5  FAIL
  T2 starter vs each Outer Rim opponent >= 50%                        min 48.9 (vessa)  FAIL
  T3 starter vs each Mid Rim opponent < 50%                           max 44.5 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 31.9 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 61.5 (vessa)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.0 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.7 (sovereign), sovereign 51.7  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.7 vs next 56.7 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 12.6 (kesh) vs 2·EV_g 5.4  PASS; @2×floor 25.1  PASS
summary: targets 6/8, coupling 1/1, bounds 4/5   (N = 10 000)
```

Only T1 (+1.5 needed) and T2 (+1.1 needed on vessa) are left; T4 passes with
1.1 points of margin, so the starter deck must not get stronger — the next
iteration weakens Greeb and Vessa instead.

## T004 — iteration 2

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| greeb `strategy` | Basic | Cautious (effective threshold 15 → 14 — "folds early") |
| greeb `misplay` | 0.40 | 0.38 |
| dax `side_deck` | +4 +3 +3 +2 +2 +1 −1 −2 −3 −2 | +3 +2 +2 +2 +1 +1 −1 −1 −2 −2 (no +4/−3: narrower reach-20 coverage) |
| vessa `misplay` | 0.30 | 0.34 |
| vessa `side_deck` | +3 +2 +2 +1 +1 −1 −1 −2 −2 −3 | +1 +1 +1 +2 +2 +2 +2 −1 −1 −2 (no 3s, thin recovery) |
| rix `side_deck` | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 −1 | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 T |

```
targets
  T1 starter vs greeb >= 65%                                          67.9  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 57.2 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 44.5 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 31.7 (magistrate)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 66.3 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 64.7 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 49.2 (rix), sovereign 52.0  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (greeb); sovereign 52.0 vs next 49.2 (rix)  FAIL
  C  B4 coupling (T004): best EV_m@floor 13.1 (kesh) vs 2·EV_g 7.2  PASS; @2×floor 26.2  PASS
summary: targets 7/8, coupling 1/1, bounds 4/5   (N = 10 000)
```

T1 and T2 cleared with room (Cautious Greeb is worth ~4 points, the narrowed
Outer decks ~3). T8 broke instead: with the Sovereign's exact deck and
threshold 18, **Rix became the hardest opponent for the full-pool deck**
(49.2 vs the Sovereign's 52.0). The finding behind it: this AI plays *better*
at an effective threshold of 18 than at 19 — standing at 19 costs more busts
than the higher total wins back — so the Sovereign's mandatory max threshold is
a handicap, and the opponents below it must not be given both the best deck and
the better threshold.

## T004 — iteration 3

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| greeb `misplay` | 0.38 | 0.44 (still the roster's highest — the rookie slips most) |
| rix `stand_threshold` | 18 | 19 |
| rix `side_deck` | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 T | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 −1 (the tiebreaker is the two masters' card) |
| kesh `stand_threshold` | 18 | 19 (forced: the roster order puts Rix before Kesh) |
| kesh `strategy` | Aggressive | Basic (keeps its effective threshold at 19, unchanged strength; the floor rises 40 → 50) |
| `STARTER_SIDE_DECK` | +1 +1 +1 +2 +2 +2 −1 −1 −2 −2 | +1 +1 +1 +1 +2 +2 −1 −1 −1 −2 |

```
targets
  T1 starter vs greeb >= 65%                                          65.3  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 52.9 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 40.0 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 29.2 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 67.7 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 64.5 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.2 (sovereign), sovereign 51.2  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.2 vs next 56.0 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 16.7 (kesh) vs 2·EV_g 6.1  PASS; @2×floor 33.5  PASS
summary: targets 8/8, coupling 1/1, bounds 4/5   (N = 10 000)
```

The stop condition is met — but **T1 passes by 0.3 points** (65.3 vs 65), well
inside the ±1 point of run-to-run noise T003 measured, so it would flip on a
re-run and leave T006 no margin to write a guard against. Iteration 4 buys
headroom there and changes nothing else.

## T004 — iteration 4 (targets converge)

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| greeb `side_deck` | +1 +1 +1 +2 +2 −1 −1 −1 −2 −2 | +1 +1 +1 +1 +2 −1 −1 −1 −1 −2 (almost all 1s — the rookie can rarely land 20) |

```
targets
  T1 starter vs greeb >= 65%                                          68.7  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 54.2 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 40.2 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 28.7 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 66.8 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.3 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.7 (sovereign), sovereign 51.7  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.7 vs next 56.8 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 15.9 (kesh) vs 2·EV_g 7.5  PASS; @2×floor 31.7  PASS
summary: targets 8/8, coupling 1/1, bounds 4/5   (N = 10 000)
```

**Converged: `targets 8/8, coupling 1/1` at iteration 4**, with `C` passing
at the floor (not only at 2× floor). Margins on the final run, in percentage
points of win rate: T1 +3.7, T2 +4.2, T3 +9.8, T4 +4.3, T5 +11.8, T6 +15.3,
T7 +6.7 above the 45 bar and 3.3 below the Sovereign's 55 ceiling, T8 worst
drop 0.0 against a tolerance of 2 with the Sovereign 5.1 points clear of the
next-hardest, `C` 15.9 vs 7.5. Every margin is wider than the ±1.1 points of
run-to-run spread T003 measured, so T006 has room to write guards.

### What the tuning turned on (for `docs/balance.md` and T005/T006)

1. **Opponent strength is mostly *playable hand coverage*.** The AI's strongest
   branch plays the card that lands exactly 20, so a deck's value is how often
   a 4-card hand holds the exact value it needs. Dead cards (flips) cost about
   4–5 win-rate points each, and narrowing a deck to ±1/±2 is worth as much as
   a threshold step. This is why the two Core masters now carry full
   ±1/±2/±3/±6 + 4 coverage and the Outer Rim carries almost none.
2. **The AI peaks at an effective threshold of about 18.** At 19 it busts more
   than the higher total wins back (iteration 2: Rix, at threshold 18 with the
   Sovereign's exact deck, was *harder* than the Sovereign). Since the Sovereign
   must hold the roster's maximum threshold, the finale's edge has to come from
   its deck and its flawless play, and the opponents just below it must not get
   both the best deck *and* the better threshold — Rix moved to 19 and gave up
   the tiebreaker for that reason (which forced Kesh to 19 too, as the roster
   order puts Rix first).
3. **The Magistrate is structurally ~5 points softer than the Sovereign** — its
   guard requires it to keep a dead flip — so it, not the Sovereign, is what T4
   binds on. The starter deck had to come down far enough that a nine-playable
   master holds it under 33 %.
4. **`BEST_FULL` was replaced** (allowed by plan tension §5) with the all-±
   shape that won the Outer+Mid pool, plus the ±6s and the tiebreaker:
   `±6 ±6 ±3 ±3 ±3 ±2 ±2 ±1 ±1 T`, against T003's `±6 ±6 ±3 ±3 ±2 ±1 ±1 +4 −4 T`.
   Measured against the Core opponents on the final roster it reads
   49.2 / 51.7 / 56.8 (rix / sovereign / magistrate) where T003's shape trailed
   `BEST_OUTER_MID` by 1.9 points on some rows; the fixed +4/−4 are dead weight
   half the time for a player that needs both signs. T8's worst ordering drop
   went from 1.9 (against a tolerance of 2) to 0.0.
5. **`STARTER_SPARES` are now lateral, not upgrades** (±1, +2, −2 instead of
   +3, −3, ±1): with 3s out of the starter deck, keeping 3s as spares would have
   meant the measured "starter" is not what a fresh player would field.
6. **For T005**: the Greeb grind is `w_g = 68.7 %`, `EV_g = 3.7` at floor 10, so
   B3 sits at `k_grind = 22` — a Core price rise widens it, a further Greeb
   softening would narrow it. Kesh's floor is now 50 and Nima's 30, which is
   where B4's headroom comes from.

## T004 — iteration 5 (final run)

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| dax `misplay` | 0.32 | 0.36 |

Only reason: with Vessa at 0.34 the Scrapper was slipping *more* than the
Greenhorn. The roster now reads greeb 0.44 > dax 0.36 > vessa 0.34 > nima 0.15 >
brakka 0.12 > toran 0.10 > kesh 0.06 > rix 0.03 > magistrate/sovereign 0.00.
Dax appears in T2 and T5 only, and both move the safe way. The run doubles as a
**reproducibility check** of iteration 4's table.

```
targets
  T1 starter vs greeb >= 65%                                          67.9  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 54.5 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 40.5 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 29.0 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 67.8 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 64.4 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.8 (sovereign), sovereign 51.8  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.8 vs next 56.7 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 16.2 (kesh) vs 2·EV_g 7.2  PASS; @2×floor 32.4  PASS
summary: targets 8/8, coupling 1/1, bounds 4/5   (N = 10 000)
```

Iteration 4 → iteration 5, per target (Dax excluded — deliberately changed):
T1 68.7 → 67.9, T3 40.2 → 40.5, T4 28.7 → 29.0, T6 65.3 → 64.4,
T7 51.7 → 51.8, T8 51.7 vs 56.8 → 51.8 vs 56.7, `C` 15.9 vs 7.5 → 16.2 vs 7.2.
Every line moves by less than a point, so the converged table reproduces.

### Final measured grid (N = 10 000, the table `docs/balance.md` records in T007)

```
deck            opponent          n    win%   ev/match@floor
starter         greeb         10000    67.9             +3.6
starter         dax           10000    54.5             +0.9
starter         vessa         10000    55.4             +2.1
starter         nima          10000    37.6             -7.4
starter         toran         10000    40.5             -5.7
starter         brakka        10000    36.0             -8.4
starter         rix           10000    29.0            -21.0
starter         kesh          10000    36.3            -13.7
starter         magistrate    10000    27.9            -22.1
starter         sovereign     10000    23.8            -26.2
standard        greeb         10000    77.9             +5.6
standard        dax           10000    64.7             +2.9
standard        vessa         10000    68.3             +7.3
standard        nima          10000    49.6             -0.2
standard        toran         10000    53.1             +1.9
standard        brakka        10000    50.1             +0.0
standard        rix           10000    41.8             -8.2
standard        kesh          10000    50.4             +0.4
standard        magistrate    10000    40.7             -9.3
standard        sovereign     10000    34.4            -15.6
best_outer      greeb         10000    80.5             +6.1
best_outer      dax           10000    67.8             +3.6
best_outer      vessa         10000    69.7             +7.9
best_outer      nima          10000    52.6             +1.5
best_outer      toran         10000    56.0             +3.6
best_outer      brakka        10000    53.4             +2.0
best_outer      rix           10000    42.9             -7.2
best_outer      kesh          10000    52.8             +2.8
best_outer      magistrate    10000    41.6             -8.4
best_outer      sovereign     10000    36.9            -13.1
best_outer_mid  greeb         10000    86.7             +7.3
best_outer_mid  dax           10000    78.0             +5.6
best_outer_mid  vessa         10000    79.8            +11.9
best_outer_mid  nima          10000    64.4             +8.6
best_outer_mid  toran         10000    66.2             +9.7
best_outer_mid  brakka        10000    65.3             +9.2
best_outer_mid  rix           10000    56.6             +6.6
best_outer_mid  kesh          10000    66.2            +16.2
best_outer_mid  magistrate    10000    53.8             +3.8
best_outer_mid  sovereign     10000    49.9             -0.1
best_full       greeb         10000    87.6             +7.5
best_full       dax           10000    79.2             +5.8
best_full       vessa         10000    81.2            +12.5
best_full       nima          10000    65.9             +9.6
best_full       toran         10000    68.1            +10.9
best_full       brakka        10000    66.8            +10.1
best_full       rix           10000    58.7             +8.7
best_full       kesh          10000    67.7            +17.7
best_full       magistrate    10000    56.7             +6.7
best_full       sovereign     10000    51.8             +1.8
```

### Final roster values

| opponent | threshold | strategy | misplay | side deck |
| --- | --- | --- | --- | --- |
| greeb | 15 | Cautious | 0.44 | +1 +1 +1 +1 +2 −1 −1 −1 −1 −2 |
| dax | 15 | Aggressive | 0.36 | +3 +2 +2 +2 +1 +1 −1 −1 −2 −2 |
| vessa | 16 | Aggressive | 0.34 | +1 +1 +1 +2 +2 +2 +2 −1 −1 −2 |
| nima | 17 | Basic | 0.15 | *(unchanged)* |
| toran | 17 | Cautious | 0.10 | *(unchanged — the standard deck)* |
| brakka | 17 | Aggressive | 0.12 | *(unchanged)* |
| rix | 19 | Calculating | 0.03 | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 −1 |
| kesh | 19 | Basic | 0.06 | *(unchanged)* |
| magistrate | 19 | Calculating | 0.00 | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 T F24 |
| sovereign | 19 | Calculating | 0.00 | ±6 ±6 ±3 ±3 ±2 ±2 ±1 +4 −4 T |

Starter deck `+1 +1 +1 +1 +2 +2 −1 −1 −1 −2`, spares `±1 +2 −2`;
`card_tier` untouched (no re-tiering was needed, so
`card_tier_partitions_the_universe`'s lists are unchanged).

## T005 — iteration 1

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| `card_price` Mid tier | 50 | 100 |
| `card_price` Core tier | 120 | 200 |

`SEED_PURSE` (50), `ANTE_BASE_THRESHOLD` (14) and `ANTE_PER_THRESHOLD_STEP`
(10) were left alone, so every floor — and therefore the reserve, the EVs, the
`C` line and B4 — is exactly as T004 measured it; only the two prices move.

Reasoning before the run (plan tension §8): B2 was the one failing bound, and
it fails by arithmetic — a clean floor-stake Outer Rim clear leaves
`50 + (10 + 10 + 20) = 90`, against `P_mid + reserve = 60`. Raising the Mid
price is the lever that touches nothing else: `P_mid = 100` puts the first Mid
card at `110`, i.e. 20 credits and about six Greeb grinds (or two wins above the
floor) past a perfect Outer clear. The Core price moved with it for two
reasons: it keeps the tier ladder meaningful (a premium Core card at 120 would
have been only 20 credits above a Mid card), and it widens B3's window, which
read `k_grind = 23` against a bar of 20 with `w_g` sitting five points under the
0.70 edge §8 names. `PAYOUT_RATIO` and `STAKE_STEP` untouched.

No exact-value test needed amending: the floors did not move, so
`ante_floor_is_the_difficulty_scalar`, `cheapest_floor_…`'s `== 10` and
`profile.rs`'s 59/60 reserve boundary all still hold as written, and
`every_card_has_a_positive_price_that_rises_with_tier` asserts the tier ladder
as inequalities (20 < 100 < 200), not values.

```
targets
  T1 starter vs greeb >= 65%                                          67.6  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 54.8 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 39.6 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 28.8 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 67.9 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.0 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.0 (sovereign), sovereign 51.0  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.0 vs next 56.2 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 15.8 (kesh) vs 2·EV_g 7.0  PASS; @2×floor 31.6  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 100, P_core 200)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 110); grind k=6  PASS
  B3 Core card by Greeb grind >= 20 matches                           k_grind=46  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=11 (kesh), @2×floor=6  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 8/8, coupling 1/1, bounds 5/5
```

Stop condition met on the first iteration. B4 reads PASS **at the floor**
(`k_bet = 11` against `k_grind/2 = 23`), not "above floor", so nothing routes
back to T004a.

## T005 — final table

The run above is the final run (one iteration, one edit). Verbatim and in full
— this is the table `docs/balance.md` records in T007 (N = `DEFAULT_N` = 10 000,
2026-09-13, macOS, release profile).

```

running 1 test

deck            opponent          n    win%   ev/match@floor
starter         greeb         10000    67.6             +3.5
starter         dax           10000    54.8             +1.0
starter         vessa         10000    55.9             +2.4
starter         nima          10000    38.2             -7.0
starter         toran         10000    39.6             -6.2
starter         brakka        10000    37.2             -7.7
starter         rix           10000    28.8            -21.2
starter         kesh          10000    37.2            -12.8
starter         magistrate    10000    28.4            -21.6
starter         sovereign     10000    23.2            -26.8
standard        greeb         10000    77.1             +5.4
standard        dax           10000    65.0             +3.0
standard        vessa         10000    67.5             +7.0
standard        nima          10000    50.3             +0.2
standard        toran         10000    52.8             +1.7
standard        brakka        10000    49.8             -0.1
standard        rix           10000    40.2             -9.8
standard        kesh          10000    50.0             -0.0
standard        magistrate    10000    39.5            -10.5
standard        sovereign     10000    35.3            -14.7
best_outer      greeb         10000    80.0             +6.0
best_outer      dax           10000    67.9             +3.6
best_outer      vessa         10000    69.3             +7.7
best_outer      nima          10000    52.2             +1.3
best_outer      toran         10000    56.6             +4.0
best_outer      brakka        10000    52.8             +1.7
best_outer      rix           10000    42.6             -7.4
best_outer      kesh          10000    52.5             +2.5
best_outer      magistrate    10000    42.2             -7.8
best_outer      sovereign     10000    36.1            -13.9
best_outer_mid  greeb         10000    86.4             +7.3
best_outer_mid  dax           10000    77.5             +5.5
best_outer_mid  vessa         10000    79.5            +11.8
best_outer_mid  nima          10000    65.0             +9.0
best_outer_mid  toran         10000    66.4             +9.8
best_outer_mid  brakka        10000    66.2             +9.7
best_outer_mid  rix           10000    57.0             +7.0
best_outer_mid  kesh          10000    65.8            +15.8
best_outer_mid  magistrate    10000    54.1             +4.1
best_outer_mid  sovereign     10000    48.0             -2.0
best_full       greeb         10000    87.7             +7.5
best_full       dax           10000    78.4             +5.7
best_full       vessa         10000    81.0            +12.4
best_full       nima          10000    66.7            +10.0
best_full       toran         10000    67.8            +10.7
best_full       brakka        10000    67.5            +10.5
best_full       rix           10000    59.3             +9.3
best_full       kesh          10000    67.6            +17.6
best_full       magistrate    10000    56.2             +6.2
best_full       sovereign     10000    51.0             +1.0
targets
  T1 starter vs greeb >= 65%                                          67.6  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 54.8 (dax)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 39.6 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 28.8 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 67.9 (dax)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.0 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.0 (sovereign), sovereign 51.0  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.0 vs next 56.2 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 15.8 (kesh) vs 2·EV_g 7.0  PASS; @2×floor 31.6  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 100, P_core 200)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 110); grind k=6  PASS
  B3 Core card by Greeb grind >= 20 matches                           k_grind=46  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=11 (kesh), @2×floor=6  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 8/8, coupling 1/1, bounds 5/5
test balance_table ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 4.69s
```

### Final economy constants

| constant | value | changed by T005? |
| --- | --- | --- |
| `SEED_PURSE` | 50 | no |
| `ANTE_BASE_THRESHOLD` | 14 | no |
| `ANTE_PER_THRESHOLD_STEP` | 10 | no |
| `card_price` Outer | 20 | no |
| `card_price` Mid | 100 | **yes** (50) |
| `card_price` Core | 200 | **yes** (120) |
| `PAYOUT_RATIO` | 1 | no (out of scope) |
| `STAKE_STEP` | 5 | no (out of scope) |

The win-rate grid reproduces T004's converged table within sampling noise
(T004's final run: T1 67.9 → 67.6, T3 40.5 → 39.6, T4 29.0 → 28.8, T6 64.4 →
65.0, T7 51.8 → 51.0, `C` 16.2 vs 7.2 → 15.8 vs 7.0) — nothing T004 owns was
touched, so the differences are the simulator's unseeded sampling only.

## T004a — iteration 1

The person's Phase 2 finding: the Outer Rim's play reads *random*, not weak.
The fix direction is to move the weakness out of `misplay` and into the decks —
a deck of only ±1s gives the AI almost no exact-20 coverage (the finding from
T004: opponent strength is mostly playable hand coverage), so the same win-rate
curve can be held with far fewer visible slips.

Changed (old → new):

| lever | old | new |
| --- | --- | --- |
| greeb `side_deck` | +1 +1 +1 +1 +2 −1 −1 −1 −1 −2 | +1 +1 +1 +1 +1 −1 −1 −1 −1 −1 (all 1s — no exact-20 coverage beyond 19/21) |
| greeb `misplay` | 0.44 | 0.22 |
| dax `side_deck` | +3 +2 +2 +2 +1 +1 −1 −1 −2 −2 | +1 +1 +1 +1 +1 +2 −1 −1 −1 −1 (one +2 left, plus-leaning for the pusher) |
| dax `misplay` | 0.36 | 0.18 |
| vessa `side_deck` | +1 +1 +1 +2 +2 +2 +2 −1 −1 −2 | +1 +1 +1 +1 +2 +2 −1 −1 −1 −1 |
| vessa `misplay` | 0.34 | 0.16 |

Thresholds untouched (greeb 15, dax 15, vessa 16), so every ante floor — and
therefore B2's `after_outer` sum and the cheapest floor — is exactly as T005
measured it. No Mid/Core opponent, no starter value, no candidate deck moved.

```
targets
  T1 starter vs greeb >= 65%                                          69.6  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 55.4 (vessa)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 39.9 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 29.6 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 71.1 (vessa)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.2 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.1 (sovereign), sovereign 51.1  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.1 vs next 57.6 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 17.5 (kesh) vs 2·EV_g 7.8  PASS; @2×floor 35.0  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 100, P_core 200)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 110); grind k=6  PASS
  B3 Core card by Greeb grind >= 20 matches                           k_grind=41  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=10 (kesh), @2×floor=5  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 8/8, coupling 1/1, bounds 5/5   (N = 10 000)
```

The stop condition is met on the first iteration and the misplay aim
(greeb ≤ 0.25, dax and vessa lower) with it: the ±1 decks gave back more than
the halved misplay rates took — T1 rose 67.6 → 69.6 and `w_g = 0.696` is still
far under B3's 0.90 edge (`k_grind = 41` against a bar of 20). Iteration 2
probes how much further the misplay ramp can come down, since the roster floor
is nima's 0.15, not a target.

## T004a — iteration 2 (the lowest ramp the roster allows)

Changed (old → new) — misplay only; the three ±1 decks from iteration 1 stand:

| lever | old | new |
| --- | --- | --- |
| greeb `misplay` | 0.22 | 0.18 |
| dax `misplay` | 0.18 | 0.16 |
| vessa `misplay` | 0.16 | 0.15 |

This is the floor: the ramp must be non-increasing along the roster order and
nima (Mid Rim, out of this task's footprint) sits at 0.15, so vessa cannot go
below 0.15, dax cannot go below vessa, and greeb must stay strictly the maximum.
No target pins the ramp — only that structure does.

```
targets
  T1 starter vs greeb >= 65%                                          70.0  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 56.0 (vessa)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 40.8 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 29.6 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 70.9 (vessa)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 65.1 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.2 (sovereign), sovereign 51.2  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.2 vs next 57.1 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 16.4 (kesh) vs 2·EV_g 8.0  PASS; @2×floor 32.9  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 100, P_core 200)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 110); grind k=6  PASS
  B3 Core card by Greeb grind >= 20 matches                           k_grind=41  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=10 (kesh), @2×floor=5  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 8/8, coupling 1/1, bounds 5/5   (N = 10 000)
```

Halving the slip rates again cost nothing measurable (T1 69.6 → 70.0, T2 min
55.4 → 56.0, both inside the ±1.1 points of run-to-run spread): once an Outer
Rim deck is all ±1s the AI has almost no play to slip *on*, so `misplay` is no
longer where their weakness lives. Converged here — the misplay aim is beaten
(greeb 0.18 against an aim of ≤ 0.25) and the ramp is at its structural floor.

## T004a — final table

A no-change reproducibility run of iteration 2's values, verbatim and in full —
this is the table `docs/balance.md` records in T007, replacing T005's
(N = `DEFAULT_N` = 10 000, 2026-09-13, macOS, release profile). It reproduces
iteration 2 within noise on every line (T1 70.0 → 69.0, T2 min 56.0 → 55.4,
T3 40.8 → 40.2, T4 29.6 → 29.9, T7 51.2 → 51.1, `C` 16.4 vs 8.0 → 15.4 vs 7.6,
`k_grind` 41 → 43).

```

running 1 test

deck            opponent          n    win%   ev/match@floor
starter         greeb         10000    69.0             +3.8
starter         dax           10000    60.9             +2.2
starter         vessa         10000    55.4             +2.2
starter         nima          10000    38.2             -7.1
starter         toran         10000    40.2             -5.9
starter         brakka        10000    37.3             -7.6
starter         rix           10000    29.9            -20.1
starter         kesh          10000    37.1            -12.8
starter         magistrate    10000    27.4            -22.6
starter         sovereign     10000    23.8            -26.2
standard        greeb         10000    79.4             +5.9
standard        dax           10000    72.1             +4.4
standard        vessa         10000    67.8             +7.1
standard        nima          10000    49.6             -0.3
standard        toran         10000    53.2             +1.9
standard        brakka        10000    50.6             +0.4
standard        rix           10000    41.0             -9.0
standard        kesh          10000    50.0             +0.0
standard        magistrate    10000    40.1             -9.9
standard        sovereign     10000    35.2            -14.8
best_outer      greeb         10000    82.9             +6.6
best_outer      dax           10000    75.5             +5.1
best_outer      vessa         10000    70.2             +8.1
best_outer      nima          10000    53.3             +2.0
best_outer      toran         10000    56.4             +3.8
best_outer      brakka        10000    53.6             +2.2
best_outer      rix           10000    43.0             -7.0
best_outer      kesh          10000    52.7             +2.7
best_outer      magistrate    10000    41.2             -8.8
best_outer      sovereign     10000    37.2            -12.8
best_outer_mid  greeb         10000    88.2             +7.6
best_outer_mid  dax           10000    82.3             +6.5
best_outer_mid  vessa         10000    79.6            +11.8
best_outer_mid  nima          10000    64.7             +8.8
best_outer_mid  toran         10000    66.8            +10.1
best_outer_mid  brakka        10000    66.1             +9.7
best_outer_mid  rix           10000    55.4             +5.4
best_outer_mid  kesh          10000    65.4            +15.4
best_outer_mid  magistrate    10000    53.5             +3.5
best_outer_mid  sovereign     10000    48.8             -1.3
best_full       greeb         10000    89.0             +7.8
best_full       dax           10000    83.5             +6.7
best_full       vessa         10000    81.0            +12.4
best_full       nima          10000    66.6            +10.0
best_full       toran         10000    69.2            +11.5
best_full       brakka        10000    67.7            +10.6
best_full       rix           10000    58.2             +8.2
best_full       kesh          10000    67.3            +17.3
best_full       magistrate    10000    56.1             +6.1
best_full       sovereign     10000    51.1             +1.1
targets
  T1 starter vs greeb >= 65%                                          69.0  PASS
  T2 starter vs each Outer Rim opponent >= 50%                        min 55.4 (vessa)  PASS
  T3 starter vs each Mid Rim opponent < 50%                           max 40.2 (toran)  PASS
  T4 starter vs each Core opponent < 33%                              max 29.9 (rix)  PASS
  T5 best_outer vs each Outer Rim opponent >= 55%                     min 70.2 (vessa)  PASS
  T6 best_outer_mid vs each Mid Rim opponent >= 50%                   min 64.7 (nima)  PASS
  T7 best_full vs each Core opponent >= 45%; sovereign 45-55%         min 51.1 (sovereign), sovereign 51.1  PASS
  T8 ordering: outer <= outer_mid <= full (tol 2); sovereign hardest  worst drop 0.0 (-); sovereign 51.1 vs next 56.1 (magistrate)  PASS
  C  B4 coupling (T004): best EV_m@floor 15.4 (kesh) vs 2·EV_g 7.6  PASS; @2×floor 30.9  PASS
bounds (SEED 50, reserve 10, P_outer 20, P_mid 100, P_core 200)
  B1 first Outer card within 5 Greeb floor matches                    k=0  PASS
  B2 first Mid card not affordable after one Outer Rim clear          (90 vs 110); grind k=6  PASS
  B3 Core card by Greeb grind >= 20 matches                           k_grind=43  PASS
  B4 Core card by Mid Rim bets < k_grind/2                            best k_bet@floor=11 (kesh), @2×floor=6  PASS
  B5 ruin within 8 floor losses; staked win never broke (021)         k_ruin=5  PASS
summary: targets 8/8, coupling 1/1, bounds 5/5
test balance_table ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 4.50s

```

### Final Outer Rim values (T004a)

| opponent | threshold | strategy | misplay (was) | side deck (was) |
| --- | --- | --- | --- | --- |
| greeb | 15 *(unchanged)* | Cautious *(unchanged)* | 0.18 (0.44) | +1 +1 +1 +1 +1 −1 −1 −1 −1 −1 (+1 +1 +1 +1 +2 −1 −1 −1 −1 −2) |
| dax | 15 *(unchanged)* | Aggressive *(unchanged)* | 0.16 (0.36) | +1 +1 +1 +1 +1 +2 −1 −1 −1 −1 (+3 +2 +2 +2 +1 +1 −1 −1 −2 −2) |
| vessa | 16 *(unchanged)* | Aggressive *(unchanged)* | 0.15 (0.34) | +1 +1 +1 +1 +2 +2 −1 −1 −1 −1 (+1 +1 +1 +2 +2 +2 +2 −1 −1 −2) |

No threshold moved, so every ante floor, the cheapest floor (greeb, 10), B2's
`after_outer` sum (50 + 10 + 10 + 20 = 90 < 110) and the T005 prices are exactly
as T005 left them. Nothing outside the three Outer Rim entries changed: no Mid
or Core opponent, no starter deck or spares, no candidate deck, no `card_tier`,
no economy constant.

### What T004a turned on

The Outer Rim's weakness had been carried by `misplay` (0.44 / 0.36 / 0.34 —
the AI throwing away a won position roughly every third turn), which is what
read as *random* to the player. Moving it into the decks instead trades that
for a legible weakness: with only ±1-magnitude cards the AI can reach exactly 20
from 19 (or recover from 21) and from nowhere else, so it plays its best line
almost every turn and still loses — Greeb's win rate against the starter barely
moved (T005's 67.6 → 69.0 for the player) while his slip rate fell by more than
half. The lever is spent, though: at the structural floor the ramp is
0.18 / 0.16 / 0.15 against nima's 0.15, so any further reduction would need a
Mid Rim opponent to move, which this task does not own.
