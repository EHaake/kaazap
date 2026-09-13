# Plan: Difficulty & economy balance pass — spec 022

> **Status**: Draft — pending sign-off
**Implements**: `spec.md` in this directory

## Context

Every balance lever exists as data — the roster's thresholds, decks, strategies
and misplay rates (`opponent.rs`), the card tiers and prices (`economy.rs`),
the seed purse / ante / stake / payout constants (`economy.rs`), the starter
deck (`profile.rs`) — but none of it has been measured, and the starter is the
**premium** `DEFAULT_SIDE_DECK`. This spec (1) gives a fresh profile an
**Outer-tier-only starter** as its own constant, (2) makes **Quick Play deal the
standard deck** (ruling C) while campaign matches keep dealing the built deck,
(3) adds a **headless simulator** as an integration test that drives the real
engine with a scripted player and prints win rates, expected credits, and
pass/fail against the spec's targets and economy bounds, (4) **tunes the data**
until the targets pass, (5) **guards** a subset with ordinary tests, and
(6) writes `docs/balance.md` and re-syncs the two existing docs.

It is a **data + measurement spec with two one-line flow changes**. The engine
(`game.rs`, `player.rs`), the AI core and misplay seam, the wager prompt,
settlement, and both save formats are untouched. No new persisted field, no
version bump, no new crate. Nothing about the simulator is in the game binary:
it lives in `tests/balance.rs`, compiled only by `cargo test`/`--all-targets`.

**Binding model constraint (person, 2026-09-13):** every implementation task,
every simulator run, and every tuning iteration runs in the `sdd-implementer`
at the implementation tier. No task below needs the top tier to run or to read
simulator output; a target the data levers cannot reach goes to the **person**
(spec Non-goals / ruling E), not to a decision review.

## What the code already gives us

- **The headless match loop exists and is the pattern.**
  `full_match_terminates_within_bounded_updates` (`game.rs:1159`) drives a
  whole match through the production state machine with no clock: at
  `PlayerTurn` it applies a `GameAction`, at `OpponentThinking` it sets
  `game_phase = OpponentTurn` directly (the field is `pub`), at
  `OpponentTurn | RoundEnd` it calls `update()`, at `AwaitingNextRound` it
  applies `NextRound`, and stops at `GameOver { winner }`. Everything it uses is
  public (`GameState::with_opponent`, `apply_game_action`, `update`,
  `game_phase`, `player`/`opponent` of type `PlayerState` with pub fields and
  `score()` / `has_tiebreaker_in_play()` / `table_full()`), so an
  **integration test in `tests/`** can run it against `kaazap::…` with nothing
  exposed that isn't already.
- **Sign-choice cards are two actions.** `PlayHand { index }` on a `±N` or the
  tiebreaker moves the phase to `AwaitingSignChoice { hand_index }`;
  `ChooseSign { positive }` commits (`game.rs:181-195`, `732-791`). Fixed cards
  commit on `PlayHand` alone; flips commit at 0 and rework the table. The
  scripted player therefore decides an `(index, value)` and the driver issues
  one or two actions.
- **Hands are dealt once per match, not per round.** `with_opponent` deals
  both hands; `setup_next_round` clears rows only; `new_game` re-deals
  (`game.rs:86-112, 795-841`). Four side cards last the whole first-to-3 match
  — card conservation is real, which is why the scripted player spends a card
  only for one of the four spec'd reasons.
- **The AI's own helpers are the model for the scripted player.**
  `best_recovery_play` (highest total ≤ 20), the exact-20 scan in
  `decide_vs_live_player`, and `best_winning_play_vs` / `winning_tie_play_vs`
  in `decide_vs_stood_player` (`game.rs:430-655`) are private, so the test
  re-implements the same three scans over `player.hand` with
  `Card::playable_values()` (public). Same logic, player's side, no engine edit.
- **Randomness is unseeded and that is fine.** `rand::rng()` at deals,
  `rand::random_range` for the misplay roll and the 0–10 dealer draw. The spec
  forbids seed threading; the simulator buys agreement with sample size
  (design §3).
- **Region membership is derivable.** `campaign::PLANETS` (8 planets, `region`
  strings, `opponents` ids) + `economy::region_tier` give the opponent sets
  without a second hand-written list: Outer Rim = Greeb, Dax, Vessa; Mid Rim =
  Nima, Toran, Brakka, Kesh; Core = Rix, Magistrate, Sovereign. Ante floors by
  today's thresholds: 10, 10, 20 / 20, 30, 30, 40 / 40, 50, 50.
- **"Affordable" already has a definition.** `Profile::can_afford(price)` is
  `credits ≥ price + cheapest_floor(run)` (the spec 021 shop reserve), and
  `cheapest_floor` is Greeb's floor in every run state (Cinder's rematch). The
  economy bounds use that definition, via `economy::cheapest_floor(&CampaignRun::default())`.
- **Quick Play and campaign launches share one deal site.** `start_match`
  (`app.rs:627-656`) builds `GameState::with_opponent(opponent,
  self.profile.deck().to_vec())` for both callers; `campaign: Option<NodeRef>`
  already says which mode it is. Ruling C is one expression there.
- **The mid-match save snapshots the deck.** `SavedGame.player_deck`
  (`save.rs:55, 98, 131`) restores the deck a match began with and falls back
  to `DEFAULT_SIDE_DECK` when it is missing or short — so a pre-022 Quick Play
  save resumes as it was, and `save.rs` needs no change.
- **The starter is two private fns with serde-default duty.**
  `profile::starter_deck()` / `starter_collection()` (`profile.rs:73-85`) back
  `Default` *and* `#[serde(default = …)]` on `collection`/`deck`, so an
  existing file with those keys is never touched by a change to them (ruling
  F), and `reset_to_starter` (`Profile::default()` swap) picks the new starter
  up for free. Today they return `DEFAULT_SIDE_DECK` + `+1 −1 ±2`.
- **Tests that encode today's starter.** `profile.rs`
  `default_profile_has_a_valid_deck_within_the_collection` asserts
  `deck == DEFAULT_SIDE_DECK`; `deck_builder.rs` tests (`enter_moves_a_present_card…`,
  `a_partly_decked_card…`, `a_type_present_in_one_panel…`,
  `an_unowned_type_is_a_placeholder…`, `arrows_skip_placeholders…`) read the
  default profile's exact slots (`+2` decked, spares `+1 −1 ±2`, `+3` unowned).
  They move onto explicit layouts via the existing `profile_from` helper.
  `save.rs` and `game.rs` tests use `DEFAULT_SIDE_DECK` as the *standard* deck
  and are unaffected.
- **The roster guards are the kept structure.** `roster_runs_easy_to_hard_by_threshold`,
  `misplay_rates_are_valid_and_the_default_is_deterministic` (rookie > master
  = 0), `the_final_boss_is_flawless_and_fully_equipped` (Sovereign = max
  threshold, Calculating, 0.0, playable multiset dominates the Magistrate's),
  `every_roster_deck_can_fill_a_hand`, `every_roster_card_is_in_the_universe`;
  in `economy.rs`: `card_tier_partitions_the_universe` (exact partition — a
  re-tiering must update its three lists), `every_card_has_a_positive_price_that_rises_with_tier`,
  `ante_floor_is_the_difficulty_scalar`, `cheapest_floor_…`. Tuning moves
  values inside what these allow.

## Design tensions resolved

### 1. The simulator is an integration test, not a `#[cfg(test)]` module in `game.rs`

`tests/balance.rs` uses only the public API, keeps ~300 lines of harness out of
`game.rs`, is built by `cargo build --all-targets` and run by `cargo test`, and
is nowhere near the binary. The table-printing run is `#[ignore]` so `cargo
test` stays fast; the guards (design §6) are ordinary tests in the same file at
a small sample size. Developer command (documented in `docs/balance.md`):

```
KAAZAP_SIM_N=10000 cargo test --release --test balance balance_table -- --ignored --nocapture
```

`--release` because 50 pairs × 10 000 matches = 500k matches; the spec's
"finishes in seconds" is measured on this command (T003), excluding the
one-time `--release` compile of the test target. `KAAZAP_SIM_N` defaults to
`DEFAULT_N = 10_000` when unset.

### 2. The scripted player does exactly the four things the spec names, in a fixed order

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

It never plays a flip (the AI doesn't either; a flip has no `playable_values`).
That makes the two Mid-tier flips dead cards for the scripted player — a stated
proxy limitation recorded in `docs/balance.md`, and why no pool-best candidate
carries one. Rules 1–3 mirror the AI's own scans, so the proxy is "a player
as sharp as the AI's deterministic core, with no misplays".

### 3. Agreement by sample size, not seeds — sized for all 50 pairs at once

For a proportion near ½, one run's standard error is `√(0.25/N)`; the
difference between two independent runs has SE `√(0.5/N)`. The acceptance
criterion is agreement "on every pair", so the per-pair figure has to survive
being raised to the 50th power. At `N = 10 000` the difference SE is 0.71
points: a single pair is within 2.5 points with probability ≈ 0.9996 (worst
case, p = ½), so all 50 pairs are within 2.5 points ≈ 0.98 of the time and
within 2 points most of the time — "about two points" in the spec's words.
(At `N = 4000` the all-pairs figure would be roughly 0.3–0.5, i.e. the check
would usually fail.) `DEFAULT_N = 10_000`; if T003's two-run comparison still
shows a pair off by more than 2.5 points, T003 doubles `DEFAULT_N` **once**,
re-runs, and reports either way — no open-ended raising. The ordering target
("at least as often") is evaluated with a `TOL = 0.02` allowance for the same
reason; the doc says so.

### 4. The simulator does the bound arithmetic and prints PASS/FAIL — so a tuning iteration is one command

The spec says the bounds are "arithmetic on the printed rates and the
constants". The test performs that arithmetic from the measured table and the
live constants (`SEED_PURSE`, `ante_floor`, `card_price`, `cheapest_floor`)
and prints each target and bound with its value and `PASS`/`FAIL`, ending in
one summary line. The ignored run **never asserts** — it is a report — so a
failing curve prints rather than panics. This is what lets T004/T005 be
bounded edit → run → read loops at the implementation tier with a `Verify:`
the orchestrator checks from the verbatim output. Tier prices are read as the
**minimum `card_price` over the cards of that tier** (robust to re-tiering),
region opponent sets from `PLANETS` (robust to nothing — roster/map changes
are non-goals — but it avoids a second list).

### 5. Full 5×10 grid, one "best" deck per pool, chosen from at most three candidates

The named set is every deck in `{starter, standard, best_outer,
best_outer_mid, best_full}` against every roster opponent (50 pairs). It is a
superset of the spec's minimum and the ordering target needs the whole grid
anyway; a grid is also the simplest code. "Best deck from a pool" is defined
in `docs/balance.md` as *a hand-built 10-card deck (copies allowed) from that
pool, the strongest of at most three candidates measured against the pool's
region in the baseline task (T003), fixed as a constant in `tests/balance.rs`*.
Not a search: C(16,10) multisets of the Outer pool alone would need a
different tool, and a human-legible "this is what a player would build" is
what the doc is for. T004 may replace a candidate if it finds a better one; the
doc records the alternatives and their rates.

### 6. Guards are a subset at generous margins, sized from the measured gaps

Non-ignored tests at `GUARD_N` (start 600; T006 sets it) so `cargo test` stays
seconds long in debug: (a) starter vs Greeb ≥ **50%** (target 65), (b) starter
vs each Core opponent ≤ **50%** (target < 33), (c) full-pool deck rate >
starter rate against **every** opponent, plus (d) every starter card is Outer
tier (`profile.rs`, exact, no sampling). T006 computes each guard's margin in
standard errors from the measured table and raises `GUARD_N` until every
margin is ≥ **5 SE** (false-failure rate below 1e-6 per guard), then runs
`cargo test` three times; T008 runs it ten times. For (c), the binding margin is
the smallest measured full-vs-starter gap (likely vs Greeb); at a 15-point gap
`N = 600` gives ~5.2 SE, at 12 points it needs ~1000.

### 7. Quick Play keeps today's deck-validity divert

`open_opponent_select` sends an under-filled built deck to the builder before
Quick Play (`app.rs:455-459`). Quick Play no longer *uses* that deck, so the
divert is now only a consistency nudge. The spec is silent; the plan keeps it
(no behavior change beyond what the spec asks, "no new UI") and flags it at
sign-off — dropping it is a two-line change if the person prefers.

### 8. Where the expected pressure lands — so tuning doesn't rediscover it

By today's constants, before any measurement:

- **B2 (first Mid card not affordable after one Outer Rim clear)** fails by
  arithmetic: `50 + (10 + 10 + 20) = 90 ≥ 50 + 10`. The Mid price (or the seed
  purse, or the Outer floors via thresholds) has to move. Raising the Mid price
  is the lever that touches nothing else.
- **B3 (Core by grinding ≥ 20 matches)** with `P_core = 120`: `(120 + 10 −
  50) / EV_g ≥ 20 ⇔ EV_g ≤ 4 ⇔ w_g ≤ 0.70` at floor 10 — a five-point window
  against T1's `w_g ≥ 0.65`. Raising the Core price widens it (200 → `w_g ≤ 0.90`).
- **T3/T4 vs T2**: the starter must beat Vessa (16, Aggressive, floor 20) at
  ≥ 50% yet lose to Nima (16, Cautious) under 50%. Same threshold, so the
  separation has to come from decks/misplay — or Nima's threshold moves to 17
  (still non-decreasing). T004 has these levers; the plan names the pressure
  so the first iteration starts there.
- **B4 is a win-rate constraint, not an economy one — it belongs to T004.**
  `k_bet < k_grind / 2` with the same numerator on both sides reduces to
  `EV_m > 2·EV_g`, i.e. `floor_m·(2w_m − 1) > 2·floor_g·(2w_g − 1)` for some
  Mid Rim opponent `m`. Prices and the seed purse cancel out;
  `ANTE_PER_THRESHOLD_STEP` scales both floors equally; `STAKE_STEP` enters no
  bound. Only T004's outputs move it: `w_g`, each `w_m`, and the thresholds
  (through the floors). At `w_g = 0.68`, `EV_g = 3.6`, so some Mid opponent
  needs `EV_m > 7.2` at its floor: Nima (20) `w > 0.68`, Toran/Brakka (30)
  `w > 0.62`, Kesh (40) `w > 0.59` — all above T6's 50% bar, so T6 passing
  does not imply B4. The simulator prints the inequality in the targets block
  as a coupled constraint (`C`, design §3) and T004's stop condition includes
  it; T005 cannot fix a B4 failure and routes one back to T004a.

None of this is decided here — T003 measures, T004/T005 tune.

## Design

### 1. `src/profile.rs` — the starter as constants

```rust
/// The side deck a fresh or reset profile plays (spec 022): Outer-tier only,
/// so every shop tier above it is a real upgrade. Tunable balance data; the
/// tiered-ness is pinned by `default_profile_plays_a_valid_outer_tier_starter`.
pub const STARTER_SIDE_DECK: [Card; SIDE_DECK_SIZE] = [
    Card::Plus(1), Card::Plus(1), Card::Plus(2), Card::Plus(2), Card::Plus(3),
    Card::Minus(1), Card::Minus(1), Card::Minus(2), Card::Minus(2), Card::Minus(3),
];
/// The Outer-tier spares a fresh profile owns beyond its deck, so the builder
/// is a real choice from the first launch (spec 008's intent, kept).
pub const STARTER_SPARES: [Card; 3] = [Card::Plus(3), Card::Minus(3), Card::PlusMinus(1)];

fn starter_deck() -> Vec<Card> { STARTER_SIDE_DECK.to_vec() }
fn starter_collection() -> Vec<Card> { starter_deck().chain(STARTER_SPARES) }   // deck + spares
```

Initial values are first guesses; T004 may change them. `Default`,
`reset_to_starter`, and the two `#[serde(default = …)]` attributes are
unchanged and pick the new fns up. `DEFAULT_SIDE_DECK` is no longer imported
by the non-test code; the tests keep it for the existing-profile check.

### 2. `src/app.rs` — Quick Play deals the standard deck

```rust
/// The deck the player is dealt (spec 022, ruling C): the built deck for a
/// campaign match, the standard deck for Quick Play — whatever the builder holds.
fn player_deck_for(is_campaign: bool, built: &[Card]) -> Vec<Card> {
    if is_campaign { built.to_vec() } else { DEFAULT_SIDE_DECK.to_vec() }
}
```

`start_match` reads `let is_campaign = campaign.is_some();` before the
existing `match campaign`, and passes `player_deck_for(is_campaign,
self.profile.deck())` to `with_opponent`. Its doc comment gains the ruling (and
drops "the profile's built deck" as the only case). Nothing else in the flow
changes; the divert stays (tension §7).

### 3. `tests/balance.rs` — the simulator (new file)

```rust
//! Headless balance simulator (spec 022). Run the table with
//!   KAAZAP_SIM_N=4000 cargo test --release --test balance balance_table -- --ignored --nocapture
//! Everything else in this file is an ordinary test (scripted-player rules,
//! termination, the balance guards). See docs/balance.md.

const SCRIPTED_STAND_AT: i32 = 17;
const STEP_CAP: usize = 5_000;          // a match that doesn't end in this many steps is a bug
const DEFAULT_N: usize = 10_000;        // KAAZAP_SIM_N overrides (tension §3)
const GUARD_N: usize = 600;             // T006 sets the final value
const TOL: f64 = 0.02;                  // ordering allowance (tension §3)

// Pool-best candidates (tension §5); T003 fixes, T004 may replace. All 10 cards, no flips.
const BEST_OUTER: [Card; 10]     = [+1, +2, +2, +3, +3, −1, −2, −3, ±1, ±1];          // Outer tier only
const BEST_OUTER_MID: [Card; 10] = [+1, +2, +3, +4, −2, −4, ±1, ±2, ±3, ±3];          // ≤ Mid tier
const BEST_FULL: [Card; 10]      = [±6, ±6, ±3, ±3, ±2, ±1, ±1, +4, −4, Tiebreaker];

enum Move { Hit, Stand, Play { index: usize, value: i8 } }

fn scripted_move(gs: &GameState) -> Move                        // tension §2, pure
fn play_match(opponent: OpponentProfile, deck: &[Card]) -> bool  // true = player won
fn win_rate(opponent: OpponentProfile, deck: &[Card], n: usize) -> f64

struct Deck { name: &'static str, cards: Vec<Card> }
fn decks() -> [Deck; 5]         // starter (STARTER_SIDE_DECK), standard (DEFAULT_SIDE_DECK), outer, outer+mid, full
fn opponents_in(tier: RegionTier) -> Vec<OpponentProfile>   // via PLANETS × region_tier × opponent_by_id
fn tier_price(tier: RegionTier) -> u32                      // min card_price over ALL_SIDE_CARDS of that tier
fn floor_of(o: &OpponentProfile) -> u32                     // economy::ante_floor(o.stand_threshold)
fn ev_per_match(floor: u32, w: f64) -> f64                  // floor × (2w − 1)

struct Row { deck: &'static str, opponent: &'static str, n: usize, win: f64 }
fn measure(n: usize) -> Vec<Row>                            // the 5×10 grid
fn rate(rows: &[Row], deck: &str, opponent: &str) -> f64
fn targets(rows: &[Row]) -> Vec<(String, bool)>             // T1–T8 below, plus the coupled C line
fn bounds(rows: &[Row]) -> Vec<(String, bool)>              // B1–B5 below

#[test] #[ignore] fn balance_table()                         // prints rows, targets, bounds, summary
#[test] fn scripted_player_follows_its_rules_on_fixed_boards()
#[test] fn named_decks_are_ten_cards_from_their_pools()
#[test] fn a_scripted_match_terminates_against_every_roster_opponent()
// T006 adds: starter_deck_beats_greeb_above_the_floor,
//            starter_deck_cannot_credibly_take_the_core,
//            the_full_pool_deck_outperforms_the_starter_against_every_opponent
```

**`play_match`** is `full_match_terminates_within_bounded_updates` with the
scripted player in the `PlayerTurn` arm:

```rust
let mut gs = GameState::with_opponent(opponent, deck.to_vec());
for _ in 0..STEP_CAP {
    match gs.game_phase {
        GamePhase::PlayerTurn => {
            if gs.player.stood || gs.player.bust { gs.update(); continue; }   // stale-frame: let the engine advance
            match scripted_move(&gs) {
                Move::Hit => gs.apply_game_action(GameAction::Hit),
                Move::Stand => gs.apply_game_action(GameAction::Stand),
                Move::Play { index, value } => {
                    gs.apply_game_action(GameAction::PlayHand { index });
                    if matches!(gs.game_phase, GamePhase::AwaitingSignChoice { .. }) {
                        gs.apply_game_action(GameAction::ChooseSign { positive: value > 0 });
                    }
                }
            }
        }
        GamePhase::AwaitingSignChoice { .. } => unreachable!("answered in the same step"),
        GamePhase::OpponentThinking { .. } => gs.game_phase = GamePhase::OpponentTurn,
        GamePhase::OpponentTurn | GamePhase::RoundEnd => gs.update(),
        GamePhase::AwaitingNextRound => gs.apply_game_action(GameAction::NextRound),
        GamePhase::GameOver { winner } => return winner == Player::Player,
    }
}
panic!("{} vs {:?} did not finish in {STEP_CAP} steps", opponent.id, deck)
```

**Table** (one row per pair, decks in the order above, opponents in roster
order), then the two check blocks, then a summary:

```
deck        opponent         n   win%   ev/match@floor
starter     greeb        10000   68.2     +3.6
…
targets
  T1 starter vs greeb >= 65%                         68.2  PASS
  T2 starter vs each Outer Rim opponent >= 50%        min 54.0 (vessa)  PASS
  T3 starter vs each Mid Rim opponent < 50%           max 47.1 (nima)   PASS
  T4 starter vs each Core opponent < 33%              max 29.8 (rix)    PASS
  T5 best_outer vs each Outer Rim opponent >= 55%     …
  T6 best_outer_mid vs each Mid Rim opponent >= 50%   …
  T7 best_full vs each Core opponent >= 45%; vs sovereign in 45–55%   …
  T8 ordering: outer <= outer_mid <= full per opponent (tol 2); sovereign hardest for full   …
  C  B4 coupling (T004): best EV_m@floor 6.9 (toran) vs 2·EV_g 7.2  FAIL; @2×floor 13.8  PASS (above floor)
bounds (SEED 50, reserve 10, P_outer 20, P_mid 50, P_core 120)
  B1 first Outer card within 5 Greeb floor matches     k=0   PASS
  B2 first Mid card not affordable after one Outer Rim clear (90 vs 60); grind from there k=…   FAIL
  B3 Core card by Greeb grind >= 20 matches            k_grind=…   PASS/FAIL
  B4 Core card by Mid Rim bets < k_grind/2             best k_bet@floor=… (toran), @2×floor=…   PASS/FAIL
  B5 ruin within 8 floor losses; staked win never broke (spec 021)   k_ruin=5   PASS
summary: targets 7/8, coupling 1/1, bounds 4/5
```

**Bound arithmetic** (all from live constants; `reserve =
economy::cheapest_floor(&CampaignRun::default())`, `EV_g =
ev_per_match(floor_greeb, w(starter, greeb))`, `need_core = P_core + reserve −
SEED_PURSE`):

- **B1**: smallest `k ≥ 0` with `SEED_PURSE + k·EV_g ≥ P_outer + reserve`;
  PASS iff `k ≤ 5` (FAIL if `EV_g ≤ 0` and not affordable at `k = 0`).
- **B2**: `after_outer = SEED_PURSE + Σ floor_o` over Outer Rim opponents (a
  clean clear at the floor — the upper bound on credits); PASS iff
  `after_outer < P_mid + reserve` **and** `EV_g > 0` (so grinding gets there);
  prints the grind count `ceil((P_mid + reserve − after_outer)/EV_g)`.
- **B3**: `k_grind = ceil(need_core / EV_g)` (`∞` if `EV_g ≤ 0`); PASS iff
  `k_grind ≥ 20`.
- **B4**: for each Mid Rim opponent `m` with `w_m = w(best_outer_mid, m) > 0.5`:
  `k_bet(S) = ceil(need_core / ev_per_match(S, w_m))` at `S = floor_m` and
  `S = 2·floor_m`. PASS iff `min_m k_bet(floor_m) < k_grind/2`; if only
  `min_m k_bet(2·floor_m) < k_grind/2` it is also PASS (the spec allows "or
  above") but printed as `PASS (above floor)` and recorded as such in the doc.
  Same numerator as B3 so the two counts compare. Because that numerator
  cancels, B4 is the inequality `EV_m > 2·EV_g` (tension §8) — the **`C`
  line** in the targets block prints exactly that comparison (best `EV_m` at
  the floor and at 2×floor against `2·EV_g`) so T004 can read its own
  constraint without the bounds block; the B4 line in the bounds block is the
  same fact in match counts, for the doc.
- **B5**: `k_ruin = ceil(SEED_PURSE / floor_greeb)`; PASS iff `k_ruin ≤ 8`
  ("a small number" pinned here). The "staked win never leaves you broke" half
  is spec 021's pinned property (`is_broke_…` tests, plan 021 tension §5) —
  cited, not re-derived.

**Fixed-board tests** build `GameState::new()` and set `player.dealer_row`,
`player.hand`, `opponent.dealer_row`, `opponent.stood`, `opponent.played_row`
directly (the `opponent_at` pattern in `game.rs` tests, mirrored to the
player's side): 23 with `−2` in hand → recover to 18; 17 with `+3` → reach 20
(and `±3` → `Play { value: 3 }`); opponent stood at 18, player 19 → `Stand`;
player 16 with `+3`/`+4` → the smaller winning total (`+3` → 19); player 17 with
only the tiebreaker, opponent stood at 18 without one → `Play { value: 1 }`;
player 12, opponent stood 18, nothing useful → `Hit`; tied at 18 → `Stand`,
tied at 12 → `Hit`; opponent live, 17 → `Stand`, 16 → `Hit`; a hand of two
flips and nothing else at 16 → `Hit` (flips never play).

### 4. Tuning levers and the order they move in

- **T004 (win-rate curve: targets T1–T8 plus the B4 coupling `C`)** may change: `OPPONENTS` entries
  (`stand_threshold`, `side_deck`, `strategy`, `misplay`) inside the kept
  structure (thresholds non-decreasing in roster order; Greeb slips most;
  Magistrate and Sovereign 0.0; Sovereign the max threshold with a playable
  multiset dominating the Magistrate's; two personalities per tier);
  `STARTER_SIDE_DECK` / `STARTER_SPARES` (Outer tier only, deck 10, a real
  builder choice); `card_tier` (three tiers, Outer buyable from the start,
  every starter card Outer — and `card_tier_partitions_the_universe`'s lists
  updated to match); the three pool-best candidates (each buildable from its
  pool). It **may not** touch the scripted player, the match loop, the
  economy constants, or `card_price`. Its stop condition is `targets 8/8,
  coupling 1/1` — the `C` line is T004's because nothing T005 owns can move
  it (tension §8).
- **T005 (economy bounds B1, B2, B3, B5)** may change: `card_price` by tier,
  `SEED_PURSE`, `ANTE_BASE_THRESHOLD`, `ANTE_PER_THRESHOLD_STEP`. **Not**
  `PAYOUT_RATIO` (even money unless the person rules otherwise — a question,
  not a tuning move), **not** `STAKE_STEP` (it enters no bound; `wager.rs` and
  its tests stay untouched), and not anything T004 owns, so T1–T8 stay as
  measured; floors only rescale EVs. B4 is reported by T005's final run but
  not owned by it: if B4 reads FAIL there, the fix is a T004a re-dispatch,
  never a T005 lever. The tests that pin exact values
  (`ante_floor_is_the_difficulty_scalar`: 15→10 … 19→50 and the unknown-id →
  30; `cheapest_floor_…`'s `== 10`; the `profile.rs` reserve test's 59/60
  boundary; `every_card_has_a_positive_price_that_rises_with_tier`) are
  amended to the new constants **in the same task**, never loosened.
- Both tasks are bounded: at most six edit → run → read iterations per
  dispatch; each iteration appends the changed values and the summary line to
  `specs/022-balance-pass/tuning-log.md`; the dispatch returns converged or
  not. One unconverged dispatch → one re-dispatch (T004a / T005a) with the
  log. A second non-convergence → the orchestrator reports the table and the
  stuck target to the **person** in plain language (which lever is pinned by
  which target) — per spec Non-goals, never a new mechanic, never a top-tier
  read of the output. **CLAUDE.md's escape hatch (the orchestrator does the
  task itself after two failed verifications) does not apply to T004/T005**:
  a tuning task that cannot converge is a product question, not a task the
  session takes over.

### 5. Docs

- **`docs/balance.md`** (new, short): the command and `N`; the agreement
  figure (tension §3, the all-pairs number); the scripted player (tension §2
  verbatim, plus two stated limitations: it never plays a flip, and in rule 3
  it stands on a tie at ≥ 17 even when the opponent alone holds a tiebreaker
  in play — a sure loss a human would chase; the proxy is kept as is);
  the five named decks with their cards and, for the pool-bests, the
  alternatives tried and their rates; the targets table with measured values,
  `N`, and date; the bounds with the arithmetic shown; the guards and their
  margins; how to re-run and what to update after a tuning change.
- **`docs/opponents.md`**: the roster table re-synced; the prose notes
  (Greeb/Vessa/Toran/Rix/Magistrate and the spec-011 additions) adjusted where
  a value moved; "a reasonable first cut, not a finished curve" → tuned in
  spec 022, pointer to `balance.md`.
- **`docs/economy.md`**: constants table, ante table, tier/price table
  re-synced; the "You keep your starter deck" note says the starter is
  Outer-tier; "all first guesses for the balance pass" → tuned, pointer to
  `balance.md`; a line that Quick Play deals the standard deck.
- **`README.md`**: one clause at the Quick Play sentence (line 85–87): Quick
  Play plays the standard side deck, campaign matches your built one.
- **Code comments**: `card.rs:106-108` (`DEFAULT_SIDE_DECK` no longer "doubles
  as the starter"; it is the standard deck — opponent baseline and Quick Play)
  and `card.rs:144-146` (`deal_hand` doc); `profile.rs` module/fn docs;
  `app.rs` `start_match` doc (design §2). `game.rs`, `player.rs`, `save.rs`
  are not edited at all, comments included.
- **`ROADMAP.md` / `DECISIONS.md`** are repo-wide: T007 drafts their text in
  `specs/022-balance-pass/closeout-main-docs.md` (021's pattern) — ROADMAP:
  balance pass shipped, the Difficulty setting now has a baseline; ROADMAP:90-91
  and DECISIONS:115-118 starter notes superseded; DECISIONS: rulings A–H plus
  tensions §1, §2, §4, §5, §7 — applied on `main` after the merge.

## Files

- `src/profile.rs` — `STARTER_SIDE_DECK`, `STARTER_SPARES`; `starter_deck` /
  `starter_collection` read them; docs; tests (rewritten default-profile test,
  new Outer-tier and existing-profile tests).
- `src/app.rs` — `player_deck_for` + test; `start_match` uses it; doc.
- `src/card.rs` — **doc comments only** (`DEFAULT_SIDE_DECK`, `deal_hand`).
- `src/deck_builder.rs` — tests only: starter-dependent tests staged via
  `profile_from`.
- `tests/balance.rs` — **new**: scripted player, match loop, grid, targets,
  bounds, table run, rule/termination tests; later the three guards.
- `src/opponent.rs`, `src/economy.rs` — **data only** in T004/T005 (`OPPONENTS`
  values; `card_tier`, `card_price`, the five constants) plus the exact-value
  tests that pin them.
- `docs/balance.md` (new), `docs/opponents.md`, `docs/economy.md`, `README.md`;
  `specs/022-balance-pass/tuning-log.md` (T003–T005 working record),
  `specs/022-balance-pass/closeout-main-docs.md` (T007).
- **No change**: `game.rs`, `player.rs`, `save.rs`, `campaign.rs`, `wager.rs`,
  `shop.rs`, `campaign_map.rs`, `board.rs`, `portrait.rs`, `stats.rs`,
  `records.rs`, `menu.rs`, `layout.rs`, `frame.rs`, `render.rs`, `overlay.rs`,
  `audio.rs`, `banter.rs`, `play_log.rs`, `Cargo.toml`, `Cargo.lock`.

## Tests

Each claim names the task that owns its check. Driver items are marked.

- **The starter is Outer-tier only and distinct from the standard deck**
  (T001): every card of `STARTER_SIDE_DECK` and `STARTER_SPARES` has
  `card_tier == Outer`; `Profile::default().deck() == STARTER_SIDE_DECK`,
  `!= DEFAULT_SIDE_DECK`, `deck_is_valid()`, collection len = 10 + spares;
  `reset_to_starter` yields the same (existing test, unchanged assertions).
- **An existing profile keeps its cards and credits** (T001): a JSON with
  `deck = DEFAULT_SIDE_DECK`, `collection = DEFAULT_SIDE_DECK + [+1, −1, ±2]`,
  `credits: 75` loads with exactly those; `PROFILE_VERSION == 1`.
- **Quick Play deals the standard deck, campaign the built one** (T001):
  `player_deck_for(false, &disjoint) == DEFAULT_SIDE_DECK`,
  `player_deck_for(true, &disjoint) == disjoint` where `disjoint` shares no
  card with the standard deck. End-to-end — *driver* (T002 phase pause): build
  an Outer-only deck, Quick Play hand shows ±6/flips/tiebreaker cards over a
  match; a campaign hand shows only Outer cards.
- **The deck-builder and shop fit an Outer-only collection** — *driver* (T002
  phase pause): album at 139×31 with 7 owned types / 8 placeholders; shop
  readout with owned counts.
- **A pre-022 Quick Play save resumes as it was** — `save.rs` untouched
  (sweep); existing `round_trip_preserves_the_player_deck` and
  `a_save_without_a_player_deck_resumes_with_the_default` stand.
- **The scripted player is deterministic and follows its rules** (T002):
  `scripted_player_follows_its_rules_on_fixed_boards` — the boards in design §3.
- **Every pair terminates** (T002): one `play_match` per pair (50) returns
  within `STEP_CAP`.
- **Named decks are legal** (T002): each is 10 cards; `BEST_OUTER` all Outer,
  `BEST_OUTER_MID` all ≤ Mid, none contains a flip; `starter ==
  STARTER_SIDE_DECK`, `standard == DEFAULT_SIDE_DECK`.
- **The command finishes in seconds and two runs agree within ~2 points on
  every pair** (T003): wall time of the documented command (excluding the
  first `--release` compile) and the per-pair max |Δ| across two consecutive
  runs, both reported verbatim; one doubling of `DEFAULT_N` allowed if any
  pair exceeds 2.5 points, then report. *Needs verification — the speed is an
  estimate.*
- **Every target and bound passes** (T004: T1–T8 and `C`; T005: B1–B3, B5,
  with B4 re-read): the summary line reads `targets 8/8, coupling 1/1, bounds
  5/5` on a fresh run after the last edit, and
  the kept-structure guards (`roster_runs_easy_to_hard_by_threshold`,
  `misplay_rates_…`, `the_final_boss_…`, `card_tier_partitions_the_universe`,
  `every_card_has_a_positive_price_that_rises_with_tier`) still pass.
- **The guards are not flaky** (T006, T008): each guard's margin ≥ 5 SE at
  `GUARD_N`, computed from the measured table and reported; `cargo test` green
  three times (T006) and ten times (T008) with the balance tests' wall time.
- **Docs match the constants** (T007, sweep): every value in the three tables
  equals the const; the measured table in `balance.md` is the T005 final run.
- **No engine / AI / save change** (T008 sweep): `git diff main --stat` shows
  no `game.rs`, `player.rs`, `save.rs`, `campaign.rs`, `wager.rs`; `card.rs`
  diff is comment-only; `PROFILE_VERSION == 1`, `SAVE_VERSION == 1`; no new
  build warnings.

## Verification

- `cargo build --all-targets 2>&1 | tail -n 20 && cargo test -q 2>&1 | tail -n 25`
  — no new warnings, reported verbatim per the constitution. The balance guards
  run inside it; the table run does not (`#[ignore]`).
- The simulator command (tension §1) with its full output, for T003–T005.
- **Driver / person attestation** (back up + checksum-restore the real
  profile/saves first): Phase 1 pause — Quick Play premium hand, campaign
  Outer hand, album and shop with the new collection; Phase 2 pause — a fresh
  campaign clears the Outer Rim with the starter, the Mid Rim walls until Mid
  cards are bought; snapshots at 139×31.

## Non-goals (from spec)

No Difficulty setting; no new mechanics (stake cap, odds by opponent, soft
reset, runs-ended counter, card drop, selling); no AI logic change or new
archetype; no roster/map change; no seed threading; no in-game balance screen;
no Quick-Play-ignores-your-deck notice in the UI; no endgame award; no
rule-variant revisit.

## Open questions

None product-level that blocks drafting. Three edges are settled here as
design, flagged for the sign-off and the person:
1. **Quick Play's deck-validity divert stays** (tension §7) although Quick Play
   no longer reads the built deck — a two-line drop if preferred.
2. **"At least as often" and "a small number" are pinned** at a 2-point
   sampling allowance (tension §3) and `k_ruin ≤ 8` (design §3, B5).
3. **B4 accepts a pass at twice the Mid floor** as the spec's "or above",
   printed and documented as such; a pass at the floor is preferred.
