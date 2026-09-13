# Close-out: drafted `ROADMAP.md` / `DECISIONS.md` text (spec 022)

`ROADMAP.md` and `DECISIONS.md` are repo-wide files: per `CLAUDE.md`'s git
conventions they commit **straight to `main`**, never on a spec branch. This
file holds the text drafted in T007, ready to paste on `main` after the merge
(the same close-out shape specs 020 and 021 used). Nothing here is applied by
the spec branch.

No `CLAUDE.md` amendment is needed this time: spec 022 changed no rule the
constitution states.

---

## 1. `ROADMAP.md` — four edits

### 1a. New entry at the end of the `## Shipped` list

Insert immediately after the **Wager & loss condition (spec 021)** entry (which
ends "…Every number is a tunable constant in `economy.rs`; see
`docs/economy.md`.", just before `## Backlog`):

```markdown
- **Difficulty & economy balance pass** (spec 022) — the difficulty curve is now
  **measured, not guessed**. A headless **balance simulator**
  (`tests/balance.rs`, an ignored integration test run as
  `KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table --
  --ignored --nocapture`) plays a **scripted player** — the AI's own
  deterministic core, no misplays — against all ten opponents with five named
  decks (the starter, the standard deck, and a best deck per card pool), then
  prints the win-rate table plus every spec target and economy bound with
  PASS/FAIL. Tuning moved **data only** (no AI logic, no new mechanics): the
  **starter deck is now Outer-tier only** (`profile::STARTER_SIDE_DECK`, its own
  constant, with lateral spares), the **standard deck** keeps its role as the
  opponent baseline and is what **Quick Play** deals, the roster was retuned
  (the Outer Rim's weakness moved out of `misplay` and into weak all-±1 decks —
  Greeb's slip rate fell 0.44 → 0.18 in the process; Nima 16/Cautious →
  17/Basic; Rix 18 → 19; Kesh 18/Aggressive → 19/Basic), and the **Mid and Core
  card prices rose 50 → 100 and 120 → 200**. Final curve at N = 10 000: all
  eight targets and all five economy bounds pass — the starter wins 69 % against
  Greeb, stays under 41 % across the Mid Rim and under 30 % across the Core,
  while a full-pool deck takes the finale 51 % of the time. Four ordinary tests
  guard a subset of that curve at a small sample, so a future data edit that
  breaks it fails `cargo test`. **No engine, save-format or UI change**, and
  existing profiles keep their cards and credits. Method, measurements and
  re-run instructions in `docs/balance.md`; `docs/opponents.md` and
  `docs/economy.md` re-synced.
```

### 1b. Replace the **Difficulty & economy balance pass** backlog bullet

Under "### Stakes, loss condition & difficulty balance (now being sequenced)",
replace the whole bullet (currently `ROADMAP.md` lines ~395–411, beginning
"- **Difficulty & economy balance pass** — the concrete home for the
cross-cutting balance pass noted above") with:

```markdown
- **Difficulty & economy balance pass** — ✅ **Shipped (spec 022** — see Shipped
  above and `docs/balance.md`). The three coupled levers (opponents, card
  prices, economy constants) were tuned together against measured win rates, and
  the invariant landed as a set of explicit targets: the starter deck clears the
  Outer Rim, **walls at the Mid Rim** (under 50 %) and **cannot credibly take
  the Core** (under 33 %), while funding a Core card by safe floor-stake
  grinding takes **43** expected matches against **11** by betting in the Mid
  Rim with a bought deck — so progress needs better cards or bigger bets. The
  human-flagged caveat stands: "cards required" is probabilistic and the *feel*
  only settles by playtest, so every lever remained a tunable constant and
  `docs/balance.md` records how to re-measure after a change. Distinct from the
  global **Difficulty setting** (easy / normal / hard) in *Other* — that is a
  player-facing selector layered on this curve, which is now the baseline it
  moves relative to.
```

### 1c. The **Difficulty setting** bullet now has a baseline

In *Other (not campaign-dependent)*, in the **Difficulty setting** bullet,
replace the sentence:

```markdown
  **Now unblocked:** the board-aware AI shipped (spec 010), so difficulty
  can scale how well opponents actually *think* — e.g. globally nudging the
  misplay rate and/or the effective threshold, not merely the raw stand
  thresholds.
```

with:

```markdown
  **Now unblocked, and now with a baseline:** the board-aware AI shipped
  (spec 010), so difficulty can scale how well opponents actually *think* —
  e.g. globally nudging the misplay rate and/or the effective threshold, not
  merely the raw stand thresholds — and spec 022 measured the **normal** curve
  those nudges would move relative to (`docs/balance.md`). Any proposed offset
  can be re-measured against the same targets with the balance simulator before
  it ships, rather than being playtested blind.
```

### 1d. The spec 008 starter note is superseded

In the **Side-deck customization (spec 008)** entry under `## Shipped`
(currently `ROADMAP.md` lines 90–91), replace:

```markdown
  whole 15-card side-card universe is `card::ALL_SIDE_CARDS`; the starter is the
  default 10 + a few spares (tunable in C's balance pass).
```

with:

```markdown
  whole 15-card side-card universe is `card::ALL_SIDE_CARDS`; the starter was
  the default 10 + a few spares — **superseded by spec 022**, which gave a fresh
  profile its own Outer-tier `profile::STARTER_SIDE_DECK` plus lateral spares
  and left `card::DEFAULT_SIDE_DECK` as the opponent baseline and Quick Play's
  deal.
```

---

## 2. `DECISIONS.md` — append a new section at the end of the file

Append after the spec 021 section's closing paragraph ("No
`game.rs`/`player.rs`/`card.rs`/`save.rs` change; … Monochrome by
construction."):

```markdown
## Difficulty & economy balance pass (spec 022)

The difficulty curve stops being a guess. A headless simulator measures it, the
tuning moves data only, and `docs/balance.md` records the method, the numbers
and how to re-run them. Ruled with the human on 2026-09-12 — on the
recommendations as proposed except **C**.

- **Measure with a headless simulator, then playtest.** The roadmap assumed
  playtest-only tuning; a simulator makes the invariant a number and the
  playtest attests the feel. Test-side code, no new crates, no seed.
- **The starter deck becomes Outer-tier only.** The discovery that drove the
  spec: the starter *was* the premium standard deck, so buying cards was
  optional. The standard deck keeps its role as the opponent baseline; the
  starter is a new, separate constant whose composition the tuning settled
  (`+1 +1 +1 +1 +2 +2 −1 −1 −1 −2`, spares `±1 +2 −2`).
- **Quick Play deals the standard (premium) deck** [human ruling, against the
  recommendation to deal the built deck]. Accepted downside, on record: the deck
  you build only matters in campaign matches, so Quick Play is no longer a place
  to test a build. Reversible in one line if that proves annoying.
- **Targets, not vibes**: the starter beats Greeb at least two matches in three,
  is under half against the Mid Rim and under a third against the Core; each
  better pool's best deck wins at least as often as the one below it; a finished
  deck has a roughly even finale (45–55 % against the Sovereign).
- **Data levers only.** Opponent data, card tiers and prices, economy constants,
  the starter deck. No AI logic, no new mechanics; an unreachable target would
  have been a question for the human rather than a code change.
- **Existing profiles keep their cards.** Only new and reset profiles get the
  new starter; no migration, no top-up, no version bump.
- **A short balance doc plus re-synced tables** (`docs/balance.md`, and
  `docs/opponents.md` / `docs/economy.md` re-synced), so the next pass starts
  from data.
- **The two spec 021 close-out notes were fixed on `main` before this spec was
  drafted**: the constitution no longer names pack-opening as a future mode, and
  the spec 020 completion bullet points at its spec 021 supersession.

Ruled at the **Phase 2 pause on 2026-09-13**, after the human played the tuned
build:

- **The Outer Rim is retuned to weak decks, not high slip rates.** Greeb, Dax
  and Vessa had been carrying their weakness in `misplay` (0.44 / 0.36 / 0.34),
  which read as *random* rather than *weak* in play. Moving it into the decks —
  all-±1 hands with almost no exact-20 coverage — held the same win-rate curve
  while the slip rates fell to 0.18 / 0.16 / 0.15, so the early opponents now
  play their best line most turns and still lose. The re-measured table is the
  one the docs record.
- **Nima's and Kesh's blurbs were reworded** — a deliberate, **limited
  exception** to the spec's "no roster-text rewrite" non-goal. Their tuned
  `strategy` no longer matched what their old blurbs promised (Nima had been
  described as folding early, Kesh as hair-trigger), so two sentences changed to
  keep the roster text honest. Nothing else in the roster's prose moved.
- **`strategy` is a tuning knob, independent of blurb text.** Nima went
  Cautious → Basic and Kesh Aggressive → Basic for measured reasons (holding an
  effective threshold while the raw threshold moved, and opening the Mid Rim as
  a wall). The archetype is difficulty data first and characterization second;
  where the two diverge, the text follows the data — as it did above — rather
  than the data being held hostage to the text.

Design tensions resolved during planning:

- **The simulator is an integration test, not a `#[cfg(test)]` module in
  `game.rs`.** `tests/balance.rs` uses only the public API, keeps ~300 lines of
  harness out of the engine, is built by `cargo build --all-targets` and run by
  `cargo test`, and is nowhere near the binary. The table run is `#[ignore]`d
  (and `--release`, for 500 000 matches in seconds) so `cargo test` stays fast;
  the guards are ordinary tests in the same file at a small sample size.
- **The scripted player is the AI's own deterministic core, in a fixed order**
  — recover from a bust, land exactly 20, beat a stood opponent (including the
  tiebreaker steal), else play to a stand-at-17 threshold. A deterministic
  function of the board with no randomness and no memory, so a measured
  difference is the *data's*, not the proxy's. Two stated limitations, kept as
  is and documented: it never plays a flip (neither does the AI, which is why no
  candidate deck carries one), and it stands on a tie at ≥ 17 even when the
  opponent alone holds a tiebreaker — a sure loss a human would chase.
- **The simulator does the bound arithmetic and prints PASS/FAIL**, from the
  measured table and the live constants, so a tuning iteration is one command
  and a failing curve prints rather than panics. The ignored run never asserts;
  the separate guards do.
- **"Best deck from a pool" is a hand-built deck, not a search**: the strongest
  of at most three human-legible candidates per pool, measured against that
  pool's region and fixed as a constant. Searching the multisets would need a
  different tool and would answer a question the doc isn't asking ("what would a
  player build?"). The alternatives and their rates are recorded in
  `docs/balance.md`.
- **Quick Play keeps today's deck-validity divert.** `open_opponent_select`
  still sends an under-filled built deck to the builder before Quick Play even
  though Quick Play no longer *uses* that deck; the spec is silent, so the plan
  kept the behavior as a consistency nudge rather than adding a behavior change.
  Dropping it is a two-line change if it ever annoys.

**Supersedes the spec 008 bullet above**, "Modest starter collection, seeded by
spec 008": the starter is no longer the default 10-card deck plus `+1`, `−1`,
`±2` spares. It is its own Outer-tier constant (`profile::STARTER_SIDE_DECK`)
with **lateral** spares (`±1 +2 −2` — only `±1` is a type the deck doesn't
already hold), so the measured starter rates describe the deck a fresh player
actually fields, and `card::DEFAULT_SIDE_DECK` is the opponent baseline and
Quick Play's deal. The "exact list is tunable balance data" part of that bullet
still holds — this pass is what tuned it.

No `game.rs`/`player.rs`/`card.rs`/`save.rs` logic change and no UI change;
`PROFILE_VERSION` and `SAVE_VERSION` both stay 1. Monochrome by construction.
```

---

## 3. Not drafted here (deliberately)

- The spec 008 bullet "**Modest starter collection, seeded by spec 008**"
  (`DECISIONS.md` lines 115–118) is left **as written**: the new section states
  plainly that it supersedes it, and `DECISIONS.md` reads as a dated log —
  the same call spec 021's close-out made for spec 020's completion bullet.
  Editing the older bullet in place is a separate call if the human prefers a
  pointer there.
