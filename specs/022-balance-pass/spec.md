# Spec: Difficulty & economy balance pass — spec 022

**Status**: Approved (person, 2026-09-12) — planning
**Depends on**: spec 021 (wager & loss, rematches), spec 012 (economy, shop,
depth-gated pool), spec 010/013 (board-aware AI, bounded misplays), spec 008
(profile, starter collection, deck-builder)

## Summary

Every lever of the campaign's difficulty curve exists — opponent thresholds,
decks, strategies and misplay rates; shop tiers and prices; the seed purse, ante
floors, stake step and payout — but the curve itself has never been measured or
tuned, and one structural fact defeats it outright: the **starter deck is a
premium deck**. A fresh profile begins with ±6 and the tiebreaker (the two
Core-tier cards the shop prices at 120), ±3, both flips and ±1 — the same deck
as Old Toran's — so shop cards are optional today. This spec makes the starter a
genuinely basic deck, builds a **headless match simulator** so the curve can be
measured rather than guessed, and tunes the existing constants until the curve
hits stated targets: a basic deck clears the Outer Rim, stalls in the Mid Rim,
and cannot credibly take the Core; the deck each region lets you buy can. Quick
Play keeps the premium deck.

## Goals

1. **Measure the curve.** A developer-run headless simulator plays many matches
   of a given side deck against a given opponent with a scripted player and
   reports the win rate and the expected credits per match at that opponent's
   ante floor, for a named set of deck × opponent pairs.
2. **A basic starter deck.** A fresh or reset profile starts with a deck (and a
   few spare cards) drawn only from the Outer tier of the card universe, so
   every shop tier above it is a real upgrade.
3. **A tuned curve that hits its targets** (the table under *Key behavior*),
   reached by changing balance data only: opponent parameters, card tiers and
   prices, and the economy constants.
4. **Grinding is viable, not dominant.** Floor-stake rematches against the
   easiest opponent earn credits slowly but reliably with the starter deck;
   bigger bets against opponents the current deck reliably beats earn faster.
5. **Quick Play keeps the standard (premium) deck** regardless of the built
   deck; campaign matches deal the built deck, as today.
6. **Documented baseline.** The method, the scripted player, the measured table
   and the targets are written down so the next tuning pass starts from data;
   the opponents and economy docs are re-synced to the new values.
7. **No regression.** No engine rule, AI decision logic, or save-format change;
   an existing profile loads with the cards and credits it has.

## Non-goals (explicitly deferred)

- **The global Difficulty setting (easy / normal / hard).** A player-facing
  selector layered on this baseline curve — its own roadmap item; tuning the
  baseline comes first.
- **New mechanics as levers**: a stake cap, better-than-even payouts for harder
  opponents, a keep-your-cards soft reset, a "runs ended broke" counter, a rare
  card drop, selling cards back. Each was a noted lever in spec 021; none is
  built unless the data levers prove unable to reach the targets — and that
  comes back to the person as a question, not a plan decision.
- **New AI archetypes or changes to the decision core.** The pass moves the
  data each opponent carries (threshold, deck, strategy choice, misplay rate),
  not how an archetype thinks. Smarter or bespoke AI is a separate spec.
- **Roster or map changes.** Same ten opponents, same eight planets, same
  opponent-per-planet assignment, same two-contrasting-personalities-per-tier
  shape. Difficulty is retuned inside that structure.
- **A seeded / reproducible engine.** Threading a seed through `GameState` is
  an engine refactor the simulator doesn't need: sample sizes large enough that
  repeated runs agree within a couple of points serve the purpose.
- **An in-game balance or statistics screen.** The simulator is a developer
  tool; nothing about it ships in the game binary.
- **Telling the player Quick Play ignores the built deck.** No new UI; the
  behavior is documented. Cheap to add a line later if it confuses in play.
- **The endgame / victory award and the roguelike-mode question** — deferred by
  earlier ruling (`ROADMAP.md`).
- **Revisiting the 0–10 main-deck variant** or any rule in `DECISIONS.md`'s
  rule-variants list.

## Entities

- **Starter deck / starter collection** — what a fresh or reset profile owns and
  plays: a 10-card deck plus a few spares, all Outer-tier. Tunable balance data;
  distinct from the standard deck below for the first time.
- **Standard deck** — the fixed premium 10-card deck that already exists as the
  opponent baseline (Old Toran, the default opponent). After this spec it is also
  the deck Quick Play deals for the player.
- **Scripted player** — the simulator's stand-in for a competent human: a fixed,
  deterministic policy that uses its side cards (reach 20 when a card allows,
  recover from over 20, stand at a sensible total against a live opponent, beat
  a stood opponent's total when it can) and otherwise hits or stands by a
  threshold. It is a proxy, not an optimal player; the doc says what it does.
- **Pair** — one (deck, opponent) combination the simulator measures. The named
  set covers at least: the starter deck against every roster opponent; the best
  deck buildable from the Outer pool, from the Outer + Mid pool, and from the
  full pool, each against the opponents of the regions that pool should beat;
  and the standard deck against every roster opponent (the Quick Play view).
- **Target** — a win-rate band or economy bound a pair must land in, listed
  below. Targets are acceptance criteria; the measured values are recorded.
- **Tunable constants** — the balance data this pass may change: each
  opponent's `stand_threshold`, `side_deck`, `strategy`, `misplay`; each card's
  tier and each tier's price; `SEED_PURSE`, `ANTE_BASE_THRESHOLD`,
  `ANTE_PER_THRESHOLD_STEP`, `STAKE_STEP`, `PAYOUT_RATIO`; the starter deck and
  collection.

## Key behavior

### Targets the pass tunes to

Win rates are the scripted player's, measured by the simulator over a sample
large enough that repeated runs agree within about two percentage points.
"Best deck from a pool" means the strongest 10-card deck buildable from that
pool, with copies, as the doc defines it.

| Pair | Target |
|---|---|
| Starter deck vs Greeb | at least **65 %** — floor-stake grinding is profitable |
| Starter deck vs each Outer Rim opponent | at least **50 %** — the Outer Rim is clearable with the starter |
| Starter deck vs each Mid Rim opponent | **under 50 %** — the Mid Rim is a wall for the starter |
| Starter deck vs each Core opponent | **under 33 %** — the Core is not credibly takeable with the starter |
| Best Outer-pool deck vs each Outer Rim opponent | at least **55 %** |
| Best Outer + Mid-pool deck vs each Mid Rim opponent | at least **50 %** — the tier you can buy beats the region you're in |
| Best full-pool deck vs each Core opponent | at least **45 %**; vs the Sovereign **45–55 %** — the finale is a fair fight for a finished deck |
| Ordering | for every opponent, each better pool's best deck wins at least as often as the one below it; the Sovereign is the hardest opponent for the full-pool deck |

Economy bounds, computed from the measured win rates and the constants:

| Bound | Target |
|---|---|
| First card | a fresh profile can afford its first Outer card within **five** floor-stake matches against Greeb at the measured rate (it may already be affordable at the seed purse) |
| First Mid card | **not** affordable from the seed purse plus one Outer Rim clear at the floor; affordable after grinding or after betting above the floor on matches the deck reliably wins |
| Core card by grinding | funding a Core card purely by floor-stake Greeb rematches with the starter deck takes at least **20** expected matches — a long grind, never the fast path |
| Core card by betting | betting at a Mid Rim floor (or above) against a Mid Rim opponent the Outer + Mid deck beats at the measured rate gets there in **fewer than half** as many expected matches |
| Ruin | a fresh profile that loses every floor-stake match goes broke in a small number of matches (the run-over flow is reachable in play), and a staked win can never leave it broke (unchanged from spec 021) |

### The simulator

A developer command (not part of the game binary; run through `cargo`, exact
form in the plan) plays N matches for every pair in the named set and prints a
table: deck, opponent, matches played, win rate, expected credits per match at
that opponent's ante floor. N is configurable with a default large enough for
the two-point agreement above. It drives the real engine — the same match
loop, the same opponent AI with its misplay seam, the same round and match
resolution — with the scripted player on the player's side. Nothing about
settlement or the profile is simulated; the economy bounds are arithmetic on
the printed rates and the constants.

A small subset of the targets is also guarded by ordinary unit tests with
generous margins (the plan picks which and how wide), so a future edit to the
balance data that breaks the curve fails `cargo test` rather than going
unnoticed. The guards must not be flaky at the chosen sample size.

### The starter deck and Quick Play

- A **fresh or reset** profile's deck and collection contain only Outer-tier
  cards: the deck is 10 cards with duplicates as needed, the collection is the
  deck plus a few Outer-tier spares so the builder is a real choice from the
  first launch (spec 008's intent, kept). The exact composition is tuned
  against the targets and recorded in the doc.
- The **standard deck** is unchanged and remains the opponent baseline.
- **Quick Play** deals the standard deck to the player, whatever the built deck
  holds. Campaign matches deal the built deck, exactly as today. A Quick Play
  match saved mid-match resumes with the deck it began with (spec 008's snapshot
  rule), so a save from before this spec is unaffected.
- An **existing profile** keeps its deck, collection and credits; only New
  Campaign and the run-over reset hand out the new starter.

### Opponents, cards and prices

- Opponent values may move freely within the kept structure: thresholds stay
  non-decreasing along the roster order (the existing guard), the rookie slips
  most and the two Core masters never slip, the Sovereign stays the hardest.
- A card may change tier and a tier may change price, but the three tiers and
  the depth gate stay; the Outer tier is buyable from the start and every
  starter card is Outer-tier, so a fresh player can always buy copies of what
  they hold.
- The economy constants may change; the ante floor stays one formula in the
  threshold, and payout stays even money unless the targets prove unreachable
  (a question for the person, per Non-goals).

## Design requirements

- **Data-only tuning.** Every value the pass changes is an existing named
  constant or roster entry, or the new starter-deck constant. No conditional
  logic is added to the engine, the AI core, the shop, the wager prompt or
  settlement to make a number come out right.
- **The simulator is test-side code with no new crates**, reusing the engine's
  public actions and the existing headless-match pattern in `game.rs`'s tests.
  It must finish in seconds at the default sample size.
- **The scripted player is deterministic given the board** and documented in
  the balance doc, so a measured rate means the same thing next time.
- **Guards tolerate sampling noise** — margins, not exact values — and the
  balance doc records the measured table and the date, so the two can be
  compared later.
- **The starter deck is a separate constant from the standard deck.** The
  standard deck keeps its name and role (opponent baseline, Quick Play); a
  test pins that every starter card is Outer-tier.
- **Additive persistence, no version bump.** No new persisted field is
  expected; if the plan needs one it is serde-defaulted. `PROFILE_VERSION` and
  `SAVE_VERSION` stay 1; `save.rs` and the match save format are untouched.
- **Docs re-synced in the same spec**: the roster table in `docs/opponents.md`,
  the constants and tier tables in `docs/economy.md`, the starter deck wherever
  it is described, and a new short `docs/balance.md` (method, scripted player,
  targets, measured table, how to re-run).
- **Monochrome, legible at the minimums** — no new screens; the deck-builder
  album and the shop already handle an Outer-only collection (placeholders) and
  must still fit.

## Acceptance criteria

- [ ] A documented developer command runs the simulator and prints, for every
      pair in the named set, the deck, the opponent, the matches played, the
      win rate, and the expected credits per match at the floor; the default
      sample size finishes in seconds, and two consecutive runs agree within
      about two points on every pair.
- [ ] A fresh profile's deck is 10 Outer-tier cards and its collection is that
      deck plus a few Outer-tier spares; New Campaign and the run-over reset
      produce the same; the deck-builder shows the new collection; an existing
      profile loads with the deck, collection and credits it had.
- [ ] Quick Play deals the player the standard deck regardless of the built
      deck (verified with a built deck that shares no card with it); a
      campaign match deals the built deck; a mid-match Quick Play save from
      before this spec resumes as it was.
- [ ] Every win-rate target in the table is met by the measured rates, and the
      measured table is recorded with its sample size in `docs/balance.md`.
- [ ] Every economy bound in the table holds by arithmetic on the measured rates
      and the shipped constants, shown in `docs/balance.md`.
- [ ] Unit-test guards cover at least: every starter card is Outer-tier; the
      starter deck wins against Greeb above a generous floor; the starter deck
      wins against each Core opponent below a generous ceiling; the full-pool
      deck outperforms the starter against every opponent; the roster and
      economy guards that already exist still pass with the new data. Ten
      consecutive `cargo test` runs show no flake.
- [ ] `docs/opponents.md` and `docs/economy.md` tables match the shipped
      constants; the starter deck is described as Outer-tier wherever it is
      described; `docs/balance.md` exists with method, scripted player,
      targets, measured table, and the re-run command.
- [ ] No change to engine rules, the AI decision core, the misplay seam, the
      wager prompt, settlement, or any save format; `PROFILE_VERSION` and
      `SAVE_VERSION` are 1; `cargo build` has no new warnings; `cargo test` is
      green.
- [ ] Attested in play by the person: a fresh campaign with the starter deck
      can clear the Outer Rim; the Mid Rim is a wall until Mid cards are bought;
      the shop and deck-builder read correctly with the new collection; Quick
      Play plays with the premium deck.

## Resolved decisions

All ruled with the person on 2026-09-12, on the recommendations as proposed
except **C**:

- **A — Measure with a headless simulator, then playtest.** The roadmap assumed
  playtest-only tuning; a simulator makes the invariant a number and the
  person's playtest attests the feel. Test-side code, no new crates, no seed.
- **B — The starter deck becomes Outer-tier only.** The discovery that drove the
  spec: the starter was the premium standard deck, so cards were optional. The
  standard deck keeps its role as the opponent baseline; the starter is a new,
  separate constant whose exact composition the tuning settles.
- **C — Quick Play deals the standard (premium) deck** [person's ruling,
  against the recommendation to deal the built deck]. The person wants Quick
  Play to keep the premium deck for now. Accepted downside, on record: the deck
  you build only matters in campaign matches, so Quick Play is no longer a
  place to test a build. Reversible in one line if that proves annoying.
- **D — Targets as tabled above**, with the starter beating Greeb at least two
  matches in three, under half against the Mid Rim, under a third against the
  Core, and a finished deck having a roughly even finale.
- **E — Data levers only.** Opponent data, card tiers and prices, economy
  constants, the starter deck. No AI logic, no new mechanics; an unreachable
  target is a question for the person.
- **F — Existing profiles keep their cards.** Only new and reset profiles get
  the new starter; no migration, no top-up, no version bump.
- **G — A short balance doc plus re-synced tables**, so the next pass starts
  from data.
- **H — The two spec 021 close-out notes were fixed on main before this draft**:
  the constitution no longer names pack-opening as a future mode (its own
  commit), and the decisions log's spec 020 completion bullet now points at its
  spec 021 supersession.
