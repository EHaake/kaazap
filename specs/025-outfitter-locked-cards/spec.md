# Spec: Locked cards in the Outfitter — spec 025

**Status**: Approved (2026-09-16). Rulings A2 (revised from A3), B2, C1.
**Depends on**: spec 012 (the depth-gated pool and the shop), spec 021 (the
ante reserve and the shop's dimming), spec 022 (tier prices), spec 024 (New
Campaign keeps your cards while the map resets)

## Summary

The Outfitter lists only the cards the player can buy right now. A new
player sees seven cards (+1 to +3, −1 to −3, ±1) and nothing saying there
are more, so the depth gate looks like missing cards instead of a promise.
The person hit this in play (2026-09-16).

This spec makes the gate visible. The Outfitter always lists **all 15
cards**, grouped by the region that opens them: Outer Rim, Mid Rim, Core.
A group you haven't reached is visibly locked: its rows are dimmed with
their prices showing, its heading names the region that opens it, and the
cursor passes over it. You can see what you're saving toward from the start.

Nothing about what you can buy, what it costs, or when it opens changes.
No engine, AI, economy, balance-data, or save-format change.

## Goals

1. **The whole card range is visible from the first visit.** Every
   collectible card appears in the Outfitter on a fresh profile, with its
   price and how many you own.
2. **Locked reads as locked, not as unaffordable.** A card you can't buy
   because its region isn't reached looks different from a card you can't
   afford yet. Both are dimmed (the game is monochrome), so the difference
   is carried by the group heading, which says the group is locked, and by
   the cursor, which stops on an unaffordable card but never on a locked
   one.
3. **Each locked group says what opens it**, in the words the map uses
   (the region name).
4. **Buying is unchanged for unlocked cards.** Affordability, the ante
   reserve, the price, the sound on a bought or refused card all behave as
   today. The cursor never lands on a locked card, so it can't be bought.
5. **It fits.** The full list, headings, balance and hint fit the 139×31
   minimum terminal with the Outfitter's current look.

## Non-goals (explicitly deferred)

- **New card types** (+5, +6, −5, −6, ±4, ±5). Logged on the roadmap as
  their own spec.
- **Changing tiers, prices, or when a region opens.** Balance data stays
  as spec 022 left it.
- **Showing unlock regions in the deck builder's album.** The album shows
  what you own; buying is the Outfitter's job.
- **Card descriptions or a detail panel** in the shop.
- **Any change to the map, the primer, or How to Play** text, unless a line
  there now reads wrong (none known).

## Entities

- **Region group** — one heading plus the cards whose region it is, in the
  order Outer Rim, Mid Rim, Core. Within a group, cards keep today's order.
- **Locked group** — a region group whose region the current campaign run
  hasn't reached. After New Campaign (spec 024) the map resets, so groups
  can lock again while you still own copies of their cards.

## Key behavior

### The list

Proposed shape at 139×31, fresh profile (dim rows marked `·` here only for
the sketch; on screen they are simply dimmed):

```
                    Outfitter
         Credits: ◈ 50  ·  spendable ◈ 40

   Outer Rim
 ▸  +1      20 cr   owned ×5
    +2      20 cr   owned ×3
    …
    ±1      20 cr   owned ×1

   Mid Rim  ·  reach the Mid Rim to unlock
·   +4     100 cr   owned ×0
    …
·   3&6    100 cr   owned ×0

   Core  ·  reach the Core to unlock
·   ±6     200 cr   owned ×0
·   ±1T    200 cr   owned ×0

      ↑/↓ choose  ·  Enter buy  ·  Esc back
```

- Locked rows look like any other row (card, price, owned count), dimmed.
- Once a region is reached, its heading is just the region name and its
  rows read like any other buyable row (dimmed only if unaffordable).

### Moving and buying

- ↑/↓ move over **unlocked** cards only, skipping locked rows and
  headings, and wrap at the ends as today. The cursor opens on the first
  Outer Rim card.
- Since the cursor can't reach a locked card, Enter and Space can't buy
  one.

## Design requirements

- Monochrome only: normal, bold, dim, and text. No new emphasis levels.
- The *Density and breathing room* rule: the list stays compact; group
  headings get one empty row above them to separate groups, nothing more.
- The cursored row still pulses and the list must not jitter as the cursor
  moves.

## Acceptance criteria

- [ ] On a fresh profile the Outfitter lists all 15 cards in three groups
      (Outer Rim 7, Mid Rim 6, Core 2), each row with its price and owned
      count.
- [ ] On a fresh profile the Mid Rim and Core rows are dimmed, and their
      headings name the region that opens them; the Outer Rim heading is the
      bare region name.
- [ ] Reaching the Mid Rim (a Mid Rim planet unlocked) unlocks the Mid Rim
      group and leaves the Core locked; reaching the Core unlocks all three.
- [ ] ↑/↓ never land the cursor on a locked row or a heading, wrapping over
      the unlocked cards only; so no key buys a locked card.
- [ ] Buying an unlocked card behaves exactly as before (affordability, the
      ante reserve, price, sound, save).
- [ ] After New Campaign on a profile that owns Mid Rim or Core cards, those
      groups show locked again with the owned counts intact.
- [ ] The 15 cards' tier grouping comes from the same source that gates the
      pool, so the list and what's buyable can't disagree.
- [ ] The full list, headings, balance and hint fit 139×31, checked by a
      test and by running the game.
- [ ] No change to `card.rs`, `game.rs`, `player.rs`, `save.rs`, the tier
      table or prices in `economy.rs`, or the profile format.
- [ ] `docs/economy.md`'s shop section describes the new list.

## Resolved decisions (the person, 2026-09-16)

- **Locked rows show their price, dimmed** (A2). First ruled A3 (a `locked`
  word hiding the price), then reversed the same day so a player can see
  what they're saving toward. Locked and unaffordable rows look alike; the
  heading and the cursor tell them apart.
- **B2 — the cursor skips locked rows.** They are there to be seen, not
  selected, so Enter on a locked card can't happen.
- **C1 — the heading names the region** (`reach the Mid Rim to unlock`),
  matching the map's labels, rather than naming planets.
- **Taken ahead of the compact layout**, and **new card types logged as their
  own roadmap item** rather than folded in.
