# Economy & progression tuning

Reference for the campaign economy — how credits are earned, how the card pool
unlocks by depth, and how the shop prices cards. Shipped in spec 012
(`specs/012-economy/`), the last of the campaign subsystems. The mechanism lives
in [`src/economy.rs`](../src/economy.rs) (pure logic) and [`src/profile.rs`](../src/profile.rs)
(persistence); the shop UI is [`src/shop.rs`](../src/shop.rs). This file explains
the *mechanism* and snapshots the *current values* (tunable balance data).

## The loop

Beat a campaign opponent → earn **credits** (scaled by its difficulty) and a
**random card drop** → spend credits at the **shop** on the campaign map → build
a stronger deck in the deck-builder from your grown collection → push deeper,
which **unlocks more of the card pool**. It's finite per run: each opponent pays
once, so a full clear yields a bounded purse (~300 credits).

## Earning — campaign wins only

A win is recorded (and rewarded) at one seam: `App::tick` sees the match reach
`GameOver` with the player as winner, on a node whose opponent isn't yet beaten
(`app.rs`, the "Economy (spec 012)" block). That guard fires **exactly once per
node**, so the reward is granted once. A **loss** pays nothing (and keeps the node
open to retry). **Quick Play** wins pay nothing — Quick Play has no campaign
`in_progress`, so it never reaches the seam. Earning is campaign-only by
construction.

Credits scale with the beaten opponent's **stand threshold** (15→19), the roster's
difficulty scalar:

| Opponent threshold | 15 | 16 | 17 | 18 | 19 |
|---|---|---|---|---|---|
| Credits (`(threshold−14)×10`) | 10 | 20 | 30 | 40 | 50 |

A full 10-opponent clear therefore pays **10+10+20+20+30+30+40+40+50+50 = 300**
credits, plus ~10 free dropped cards.

## The depth-gated pool

Which cards you can win or buy is gated by how far core-ward you've **reached**.
The three map regions form an ordered `RegionTier` (Outer < Mid < Core); your
available pool is every card whose tier is at or below the **deepest region you've
unlocked** (`economy::deepest_reached` over `planet_unlocked` planets). Both the
win-drop and the shop draw from this same pool, so it grows monotonically as you
progress (Outer ⊆ Mid ⊆ Core).

The 15-card universe is partitioned by power (`economy::card_tier`):

| Tier | Opens at | Cards | Price |
|---|---|---|---|
| **Outer** | the start | `+1 +2 +3  −1 −2 −3  ±1` | 20 |
| **Mid** | reaching the Mid Rim | `+4 −4  ±2 ±3  2&4 3&6` | 50 |
| **Core** | reaching the Core | `±6  ±1T` | 120 |

> You keep your **starter** deck and collection regardless of tier — the gate is
> on *acquiring more* (shop + drops), which is what grows the collection. So the
> Outer pool is what a fresh run can win or buy; the premium ±6 and the
> round-stealing tiebreaker only become acquirable once you reach the Core.

The pool is evaluated **after** a win is recorded, so the win that first unlocks a
region (e.g. clearing The Anvil, which opens the Core) can already draw its drop
from — and shop the — new tier. This is deliberate: the drop and the shop, both
read post-win, always agree.

## The shop

Reached from the campaign map with **`b`** (the "Outfitter"). It lists the
currently-available pool — each card with its price and how many you already own —
and your live credit balance (also shown in the map header, `◈ N`). Enter buys the
highlighted card if you can afford it (credits deducted, a copy granted); a card
you can't afford is dimmed and a buy on it is a soft no-op. Purchases go through
`Profile::try_purchase`, which decides affordability and grows the collection —
the same bag-of-copies the deck-builder reads, so a bought card is immediately
usable.

## Persistence

`credits: u32` is an additive `#[serde(default)]` field on `Profile` (the
`campaign`-field precedent), so `PROFILE_VERSION` stays 1 and a pre-economy
`profile.json` loads with `credits: 0` and no version bump. The grown collection
is just a longer `Vec<Card>`. Both persist through the profile's existing
best-effort `save()`, written after each earn/purchase.

## Tuning & guards

All of the above is **data** — the credit formula, the tier partition, and the
prices in `economy.rs` — changed without touching the shop or the win seam. Guards
that keep it honest:

- `card_tier_partitions_the_universe` — every one of the 15 cards is tiered
  exactly once (no card falls to the `_ => Core` fallback).
- `every_planet_region_maps_to_a_known_tier` — the region→tier mapping covers
  every `PLANETS` region (guards the string coupling).
- `the_pool_grows_monotonically_with_depth` — Outer ⊆ Mid ⊆ Core ⊆ universe, and
  the Core opens the whole pool.
- `win_reward_scales_credits_and_picks_the_card_by_roll` — the reward is a pure
  function of (threshold, pool, roll).
- `earning_grows_the_balance_and_purchase_is_affordability_gated`,
  `credits_persist_and_default_to_zero_for_older_profiles` (in `profile.rs`).
- `the_full_pool_fits_the_minimum_terminal` — the shop's 15-row Core list fits
  89×31.

A dedicated **balance pass** (do the rewards and prices make progression feel
right?) is a tracked cross-cutting item in `ROADMAP.md`.

## See also

- [`src/economy.rs`](../src/economy.rs) — tiers, pool, pricing, `win_reward`.
- [`src/profile.rs`](../src/profile.rs) — credits + collection persistence.
- [`src/shop.rs`](../src/shop.rs) — the shop screen.
- [`docs/opponents.md`](opponents.md) — the difficulty scalar credits ride on.
- [`DECISIONS.md`](../DECISIONS.md) — why the economy is credits + depth (not gacha).
