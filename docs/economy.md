# Economy & progression tuning

Reference for the campaign economy — how credits are **staked and won or lost**,
how the card pool unlocks by depth, how the shop prices cards, and what happens
when you run out. Shipped in spec 012 (`specs/012-economy/`) and reworked into a
two-directional loop in spec 021 (`specs/021-wager-and-loss/`). The mechanism
lives in [`src/economy.rs`](../src/economy.rs) (pure logic) and
[`src/profile.rs`](../src/profile.rs) (persistence); the wager prompt is
[`src/wager.rs`](../src/wager.rs) and the shop UI is [`src/shop.rs`](../src/shop.rs).
This file explains the *mechanism* and snapshots the *current values* (tunable
balance data).

## The loop

Stake credits on a campaign match → **win and the stake comes back doubled, lose
and it's gone** → spend winnings at the **shop** on the campaign map (cards come
only from the shop now; there are no free drops) → build a stronger deck in the
deck-builder → push deeper, which **unlocks more of the card pool**. Beaten
opponents stay launchable as **rematches**, so grinding small, safe stakes is a
real (if slow) way to fund a card. Drop below the cheapest ante you could
possibly pay and the **run is over**: a full reset to the starter deck and the
seed purse.

## Tunable constants

All in [`src/economy.rs`](../src/economy.rs), all first guesses for the balance
pass to change:

| Constant | Name | Default | Meaning |
|---|---|---|---|
| Seed purse | `SEED_PURSE` | **50** | Credits a fresh or reset profile starts with (was 0). One Outer card, or a few minimum antes. |
| Ante base | `ANTE_BASE_THRESHOLD` | **14** | The stand threshold an ante is measured from (a 14-threshold opponent would be free). |
| Ante step | `ANTE_PER_THRESHOLD_STEP` | **10** | Credits the floor rises per point of threshold above the base. |
| Stake step | `STAKE_STEP` | **5** | The increment the wager prompt walks in. |
| Payout | `PAYOUT_RATIO` | **1** | Winnings per credit staked — 1 is even money, so a win returns `stake × 2`. |
| Maximum stake | — | the full balance | No cap; the prompt's upper bound is what you have. |

The **ante floor** is `ante_floor(threshold) = (threshold − ANTE_BASE_THRESHOLD)
× ANTE_PER_THRESHOLD_STEP` — the same difficulty scalar the old win reward used,
so difficulty keeps one number:

| Opponent threshold | 15 | 16 | 17 | 18 | 19 |
|---|---|---|---|---|---|
| Ante floor | 10 | 20 | 30 | 40 | 50 |

`ante_floor_for(opponent_id)` looks the threshold up in the roster and falls back
to the baseline `STAND_THRESHOLD` (17 → 30) for an id no roster entry claims —
never to a free match. `win_payout(stake)` is `stake × (1 + PAYOUT_RATIO)`: the
stake back plus its winnings.

## Staking a match

Enter on a launchable node opens the **wager prompt** ([`src/wager.rs`](../src/wager.rs),
`Modal::Wager`) over the map rather than starting the match: it names the
opponent and planet and shows the floor, the balance, and what a win pays. The
stake opens at the floor and ←/→ walk the grid `floor + k · STAKE_STEP`, clamped
to the balance, so all-in is always reachable. Enter commits; Esc backs out with
nothing changed.

If the balance can't cover the node's floor, the launch is refused before any
prompt opens — a soft no-op with a map banner, like an unaffordable shop buy
(`App::launch_campaign_node`, `MapBanner::CantCover`).

On commit the stake is **escrowed**: `Profile::stake_match` deducts it from the
balance immediately and stores the in-flight `NodeRef { planet, opponent, stake }`.
The map header's `◈ N` is honest mid-match, and the stake rides beside the board
in the presence panel ([`src/portrait.rs`](../src/portrait.rs)) for the whole
match. Quick Play never touches any of this: no prompt, no stake, no payout.

## Rematches

`CampaignRun::launchable_opponent` returns the planet's next un-beaten opponent
and, once the planet is cleared, falls back to its **final** opponent. A rematch
is a normal staked match — same floor, same payout — but changes no progress:
`mark_beaten` on an already-beaten node is idempotent, no world re-unlocks, and a
campaign completion is never re-counted (see below). Match and round records
(spec 020) record rematches like any other match.

## Settling — one seam, exactly once

Settlement happens in `App::tick`, in the single resolution block on the
`phase_changed` edge into `GamePhase::GameOver`: `settle_campaign_match` →
`record_match` → `save`. Spec 012's every-tick "not yet beaten" guard is gone —
rematches make it useless as a once-guard — and spec 020's separate record block
folded into the same edge.

`Profile::settle_campaign_match(player_won)`:

1. Reads the in-flight pointer; `None` (Quick Play) settles nothing.
2. `CampaignRun::take_stake()` — returns the stake **and zeroes the escrow**.
3. On a win, adds `win_payout(stake)`, calls `mark_beaten`, and counts a campaign
   completion only on the **edge** `!was_complete && run_complete()`.
4. Returns `StakeOutcome::Won(stake)` / `Lost(stake)` for the map banner ("★ Won
   N credits" — the winnings — or "Lost N credits").

Paying exactly once is therefore a *data* property rather than an ordering rule:
a second settlement would pay `win_payout(0) = 0` and `mark_beaten` is
idempotent. One visible consequence: the escrow reads 0 from the game-over tick
on, so the in-match stake line disappears there while the popup and the map
banner carry the result. The in-progress pointer is cleared by the player's
acknowledgement, not by settlement.

A loss simply keeps the escrow: the credits left the balance at launch, the node
stays open to retry.

## Going broke

`Profile::is_broke()` is a pure predicate: `credits < economy::cheapest_floor(run)`,
where `cheapest_floor` is the minimum ante over every unlocked planet's
launchable opponent. It is evaluated at exactly two seams, both through
`App::enter_campaign_map()` (open the map, then raise `Modal::RunOver` if broke):

- the **game-over acknowledgement** of a campaign match, and
- **campaign entry** — Continue's no-save branch and the confirmed
  discard-and-enter.

Back from the shop or the deck-builder keeps the plain `open_campaign_map`: the
shop reserve makes those paths unable to create a broke state, and keeping the
check at the two spec'd seams keeps the intent legible. A *staked* win can never
leave the player broke (`credits ≥ win_payout(stake) ≥ 2 × floor`), so the
ack-time check needs no "only after a loss" guard. The exception is a stake-0
pointer — a pre-021 campaign save resumed on a 0-credit profile — where the ack
can meet the notice after a win; that is the accepted migration path arriving one
match later, not a separate rule.

The run-over notice is modal and has no decline: Enter/Space acknowledge and run
`start_new_campaign`, i.e. spec 014's `reset_to_starter` — starter deck and
collection, no progress, the seed purse — then a fresh map. Settings (their own
file) and lifetime records survive, exactly as for New Campaign.

Note for tuning: with rematches, Cinder's final opponent is always launchable, so
`cheapest_floor` is **10 in every run state today**. The unlocked-planet filter is
written for correctness, not because any current run exercises it.

## The shop and its reserve

Reached from the campaign map with **`b`** (the "Outfitter"). It lists the
currently-available pool — each card with its price and how many you own — plus
the balance *and* the **spendable** amount, `credits − cheapest_floor`. A
purchase must leave that cheapest ante behind, so shopping can never end a run:
`Profile::can_afford(price)` is `credits ≥ price + cheapest_floor`, and
`try_purchase` deducts only `price`, never the reserve. The shop's dimming reads
the same predicate `try_purchase` enforces, so the readout and the refusal can't
disagree; a reserved-out card is dimmed and a buy on it is a soft no-op.

## The depth-gated pool

Which cards you can buy is gated by how far core-ward you've **reached**. The
three map regions form an ordered `RegionTier` (Outer < Mid < Core); your
available pool is every card whose tier is at or below the **deepest region
you've unlocked** (`economy::deepest_reached` over `planet_unlocked` planets), so
it grows monotonically (Outer ⊆ Mid ⊆ Core).

The 15-card universe is partitioned by power (`economy::card_tier`):

| Tier | Opens at | Cards | Price |
|---|---|---|---|
| **Outer** | the start | `+1 +2 +3  −1 −2 −3  ±1` | 20 |
| **Mid** | reaching the Mid Rim | `+4 −4  ±2 ±3  2&4 3&6` | 50 |
| **Core** | reaching the Core | `±6  ±1T` | 120 |

> You keep your **starter** deck and collection regardless of tier — the gate is
> on *acquiring more*, which since spec 021 means the shop alone. So the Outer
> pool is what a fresh run can buy; the premium ±6 and the round-stealing
> tiebreaker only become buyable once you reach the Core.

The pool is read after settlement, so the win that first unlocks a region (e.g.
clearing The Anvil, which opens the Core) can already shop the new tier.

## Stakes and the mid-match save

A staked match saved mid-match resumes with its stake intact — the balance
already reflects the escrow, and it resolves on resume as if never interrupted.
**Discarding** a saved staked match forfeits the stake (it was already deducted
and the pointer goes with the save): Quick Play over a save, Start Campaign's
discard confirm, and New Campaign each append "…and forfeit your N-credit stake"
to their confirmation when `CampaignRun::stake_at_risk()` is `Some`.

## Persistence

- `credits: u32` remains an additive `#[serde(default)]` field on `Profile`. A
  fresh or reset profile gets `SEED_PURSE` via `Default`; a pre-existing
  `profile.json` still loads with the credits it has (and `0` if the field
  predates the economy) — the seed purse is a *new/reset* value, not a top-up.
- `stake: u32` is an additive `#[serde(default)]` field on `NodeRef`, so the
  stake lives with the in-flight pointer in `profile.json` and is discarded with
  it. A pre-021 in-flight match loads as a stake-0 match.
- **No version bump**: `PROFILE_VERSION` stays 1 and `SAVE_VERSION` stays 1 —
  nothing here touches the mid-match save format (`save.rs`) or the engine.

## Tuning & guards

All of the above is **data** — the ante formula, the payout ratio, the seed
purse, the tier partition, and the prices in `economy.rs` — changed without
touching the prompt, the shop, or the settlement seam. Guards that keep it
honest:

In `economy.rs`:

- `ante_floor_is_the_difficulty_scalar` — 15→10 … 19→50, and an unknown
  opponent id falls back to the baseline rather than to free.
- `payout_is_even_money` — `win_payout(20) == 40`, `win_payout(0) == 0`.
- `cheapest_floor_is_the_min_over_launchable_nodes` — checked against an
  independently spelled-out minimum across fresh / half-cleared / complete runs.
- `card_tier_partitions_the_universe`, `every_planet_region_maps_to_a_known_tier`,
  `the_pool_grows_monotonically_with_depth`,
  `every_card_has_a_positive_price_that_rises_with_tier` — the pool and pricing
  rules from spec 012.

In `profile.rs`:

- `staking_escrows_the_credits_and_records_the_match_in_flight`.
- `settling_a_win_pays_double_the_stake_and_marks_the_node_beaten`,
  `settling_a_loss_keeps_the_stake_and_leaves_the_node_unbeaten`,
  `settling_without_a_pointer_is_a_no_op_and_a_zero_stake_pointer_still_settles`.
- `a_rematch_settles_for_credits_but_changes_no_progress_or_completions` and
  `campaign_completion_counts_only_a_final_clearing_win_and_recounts_after_reset`
  — the completion edge.
- `is_broke_reads_the_balance_against_the_cheapest_launchable_ante`.
- `earning_grows_the_balance_and_purchase_holds_back_the_ante_reserve`.
- `credits_seed_a_fresh_profile_but_default_to_zero_for_older_profiles` and
  `a_staked_match_in_flight_round_trips_through_the_profile_json` — both assert
  `PROFILE_VERSION == 1`.

Elsewhere: `the_stake_rides_on_the_in_flight_node_and_defaults_to_zero` and
`take_stake_empties_the_escrow_exactly_once` (`campaign.rs`),
`launchable_opponent_falls_back_to_a_rematch_once_cleared` (`campaign.rs`), the
wager prompt's grid/commit/fit tests (`wager.rs`, including
`every_line_fits_seventy_columns`), `run_over_acknowledged_only_on_enter_or_space`
(`app.rs`), `a_staked_match_draws_the_stake_under_the_pips` and
`quick_play_leaves_the_stake_rows_blank` (`portrait.rs`), and
`the_full_pool_fits_the_minimum_terminal` (`shop.rs`).

A dedicated **balance pass** (do the floors, payouts, and prices make progression
feel right?) is a tracked cross-cutting item in `ROADMAP.md`; spec 021 exists to
give it the levers.

## See also

- [`src/economy.rs`](../src/economy.rs) — constants, antes, payout, tiers, pricing.
- [`src/profile.rs`](../src/profile.rs) — staking, settlement, broke, purchases.
- [`src/campaign.rs`](../src/campaign.rs) — `NodeRef.stake`, rematches, `take_stake`.
- [`src/wager.rs`](../src/wager.rs) — the wager prompt.
- [`src/shop.rs`](../src/shop.rs) — the shop screen and its reserve readout.
- [`docs/opponents.md`](opponents.md) — the difficulty scalar antes ride on.
- [`DECISIONS.md`](../DECISIONS.md) — why the economy is credits + depth (not gacha), and why it's now two-directional.
