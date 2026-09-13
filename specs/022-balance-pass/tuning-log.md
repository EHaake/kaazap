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
