# Spec: Economy & progression (campaign subsystem C)

## Summary

The campaign map (spec 009, expanded 011) records progress but the **economy is
stubbed** — a win grants no credits or cards. This spec builds it, the last big
campaign subsystem. Wins pay **credits** (scaled by opponent difficulty) and
**drop a random card**; a **shop** on the campaign map spends credits on cards;
and both the drop and the shop draw from a card pool that **unlocks by campaign
depth** — the three map regions (Outer Rim → Mid Rim → Core) each open more of
the 15-card universe. It turns the campaign from a gauntlet into a progression:
you grow your collection as you push core-ward, then rebuild your deck (in the
existing deck-builder) from what you've earned.

Pure content and persistence over shipped patterns — no engine, board-renderer,
or save-format change (credits is an additive profile field).

## Goals

1. **Credits from wins.** Beating a campaign opponent awards credits scaled by its
   difficulty. Persisted in the profile. (Quick Play stays stakes-free.)
2. **A card drop per win.** Each campaign win also drops one random card from the
   depth-gated pool into your collection.
3. **Depth-gated card pool.** Which cards you can win or buy is gated by how far
   core-ward you've reached: the Outer Rim opens the basic adjusters, the Mid Rim
   the bigger swings and flips, the Core the premium cards (±6, the tiebreaker).
4. **A shop.** Reachable from the campaign map, it sells the currently-available
   pool for credits, showing prices, what you own, and your live balance.
5. **It all flows into the deck-builder.** Cards won or bought grow the
   collection, which the existing deck-builder immediately lets you run.
6. **No regression.** Every existing rule, screen, and save loads and behaves as
   before; old profiles gain `credits: 0` with no version bump; Quick Play and the
   match engine are untouched.

## Key behavior

### Earning (campaign wins only)

When you beat a campaign opponent, once, you earn **credits** (more for harder
opponents) and one **random card** from your current depth-gated pool, dropped
into your collection. The reward is shown on the campaign map when you return
from the match. A **loss** costs nothing (unchanged). **Quick Play** wins earn
nothing — it stays a practice sandbox.

### Depth-gated pool

The 15-card universe is split across the three map regions by power. Your
**available pool** is everything up to the deepest region you've **reached**
(unlocked): start in the Outer Rim with the basic cards; reaching the Mid Rim
adds the bigger swings and flips; reaching the Core adds the premium cards. Both
the win-drop and the shop draw from this same growing pool. (You keep your
starter deck regardless — the gate is on *acquiring more*.)

### The shop

From the campaign map, open the shop: a list of the currently-available cards,
each with its price and how many you already own, and your credit balance. Buy a
card you can afford and it's added to your collection (and your balance drops); a
card you can't afford is refused. Back returns to the map. Nothing else about
the card is new — the shop grows the same collection the deck-builder reads.

## Design requirements

- **Campaign-only earning** (human-ruled) — only campaign wins pay out; the
  existing once-per-win campaign seam already excludes Quick Play.
- **The shop lives on the campaign map** (human-ruled) — the between-worlds
  outfitter, beside the campaign depth that gates its stock; the credit balance
  shows in the map header.
- **Additive persistence** — `credits` is a serde-defaulted profile field; no
  `PROFILE_VERSION` bump; grown collection is just a longer `Vec<Card>`.
- **The reward computation is a deterministic, testable core** — credits and the
  drawn card are a pure function of (opponent difficulty, available pool, one
  injected roll); only that roll is random.
- **The shop is a `Screen`** following the established convention, not a game
  phase; the board and engine stay economy-free.

## Acceptance criteria

- [ ] A campaign win awards difficulty-scaled credits and one card from the
      depth-gated pool, exactly once per opponent; a loss and Quick Play award
      nothing.
- [ ] The available pool grows monotonically with campaign depth (Outer ⊆ Mid ⊆
      Core), is always within the 15-card universe, and every card is assigned a
      tier.
- [ ] The shop, reached from the campaign map, sells the available pool: prices
      shown, owned counts shown, a live balance; an affordable buy grants the card
      and deducts credits, an unaffordable buy is refused.
- [ ] Credits and grown collection persist across restart; an older profile
      (no `credits`) loads with `credits: 0` and no version bump.
- [ ] Cards won or bought appear in the deck-builder as owned copies.
- [ ] `cargo test` green (profile/economy/shop unit tests), `cargo build` no new
      warnings, no panics in play, legible at the 89×31 minimum.

## Resolved decisions

- **Progression = credits + campaign depth** (pre-ruled, `DECISIONS.md`) — depth
  gates availability; credits buy from the available pool; each win drops one
  random card from it. Scarcity is distribution, not new card types.
- **Not a gacha / multi-card pack** (pre-ruled) — a single card drop per win, to
  keep the tuning burden bounded.
- **Campaign-only earning** and **the shop on the campaign map** (both human-ruled
  this session).
- **A finite economy per run** — each win pays once, so a full clear yields a
  bounded purse; resetting a cleared campaign's rewards (NG+) is deferred to the
  roguelike mode (spec E).
