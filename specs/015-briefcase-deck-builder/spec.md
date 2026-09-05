# Spec: Two-panel "briefcase" deck-builder — spec 015

**Status**: Draft — pending review
**Depends on**: spec 008 (side-deck customization — the collection/deck model
and today's builder), spec 009/011 (the campaign map — the new entry point),
spec 002 (the card-frame visual vocabulary).

## Summary

The side-deck builder today is a single grid of every owned card type, each card
carrying an `in-deck/owned` badge and leaning on three border weights (heavy =
cursor, double = in deck, light = owned-but-not-in-deck) to say what's where. It
works, but it reads busy and doesn't match the KOTOR "briefcase" mental model the
game evokes. This spec reshapes it into two side-by-side panels — your
**Collection** on the left, your **built Deck** on the right — where building a
deck means moving card copies across from one panel to the other. It also makes
the builder reachable from the campaign map, so you can retool between matches
without backing out to the main menu.

This is a presentation-and-navigation reshape plus one small new entry point. What
a deck *is* — up to 10 copies drawn from the cards you own, with the same
add/remove legality — does not change; the hand is still dealt as a random draw
from the 10, so deck order stays irrelevant.

## Goals

1. **Two-panel briefcase.** Replace the single grid with a Collection panel (left)
   and a Deck panel (right), each rendering cards in the established card-frame
   vocabulary.
2. **Move copies across.** Confirming on a card in the Collection places one copy
   into the Deck; confirming on a card in the Deck returns one copy to the
   Collection. Counts on both sides update, and a card with no remaining copies on
   a side leaves that side.
3. **Switch focus between panels.** The player can move the cursor within a panel
   and switch which panel is active.
4. **Keep the deck readout.** A "Deck: N/10" readout shows how close the deck is to
   legal, alerting while it's short (an incomplete deck is the only thing that
   blocks starting a match).
5. **Reachable from the campaign map.** Add a key on the map to open the builder
   and return to the map on exit, for between-match retooling — alongside the
   existing menu entry and the incomplete-deck divert.
6. **Cleaner card vocabulary.** Because a card's *panel* now says whether it's in
   the deck, the screen drops back to the brief's two border weights (heavy =
   cursor, plain otherwise); the third "double = in deck" weight is retired.
7. **No regression.** Deck legality (exactly 10, never more copies than owned), the
   menu entry, the incomplete-deck divert, and the save format are all unchanged.

## Non-goals (explicitly deferred)

- **Buying cards / showing unowned cards.** The Collection panel shows only cards
  you own; acquiring new cards is the shop's job (spec 012). Folding "cards you
  could buy" into the builder would blur the two screens — deferred as a shop
  concern, not a builder one.
- **Deck ordering / reordering.** The match deals the hand as a random sample from
  the 10, so order carries no meaning; both panels sort canonically and there is no
  reorder gesture. Adding one would imply a rules meaning that doesn't exist.
- **NG+ / carrying an arsenal across a reset.** Out of scope here; it's owned by the
  roguelike mode (spec E). This spec doesn't touch collection/economy persistence.
- **Reworking the shop or campaign-map *presentation*.** Those belong to the
  separate presentation-polish spec. This spec only *adds a launch key* to the map;
  it does not restyle the map.
- **Changing what a deck is or its size.** `SIDE_DECK_SIZE` (10) and the add/remove
  rules are unchanged; this is a new view over the same model.

## Entities

- **Collection** — the multiset of card copies the player owns. For the builder,
  each distinct card type has an *owned* count and an *in-deck* count (copies of it
  currently placed in the deck). A type's *available* copies = owned − in-deck.
- **Deck** — the multiset of up to 10 copies the player has placed for matches.
  Legal when it holds exactly 10, each backed by an owned copy.
- **Collection panel** — the left panel: every card type with ≥1 *available* copy,
  shown with that available count.
- **Deck panel** — the right panel: every card type with ≥1 *in-deck* copy, shown
  with that in-deck count, plus the N/10 readout.

These are the same underlying collection and deck as today; only the view splits
them by where each copy currently sits.

## Key user flows

### Opening the builder

From the main menu's **Side Deck** item (as today), from the **campaign map** via a
dedicated key (new), or automatically when starting a match finds an incomplete
deck (as today). The builder opens with the Collection panel active.

### Building the deck

- The player sees two panels: **Collection** (left) listing the copies they own
  that aren't in the deck, and **Deck** (right) listing the copies they've placed,
  with a **Deck: N/10** readout.
- Moving the cursor highlights a card in the active panel (the highlight is the one
  moving thing, per the selection pulse). The player can switch the active panel.
- **Confirming on a Collection card** moves one copy into the Deck: its available
  count drops by one (and the card leaves the Collection when it reaches zero), the
  Deck gains or increments that card, and the readout updates. If the deck already
  holds 10, the action does nothing (the readout already reads 10/10).
- **Confirming on a Deck card** moves one copy back to the Collection: the reverse.
- **Leaving** (Esc/x) returns to wherever the builder was opened from — the menu or
  the map.

### Edge states

- **Deck short (N < 10)** — the readout alerts ("Deck: N/10 — add M more"), as
  today, since an incomplete deck is the only thing blocking a match.
- **Deck full (10/10)** — adding is a no-op; the player removes a card first.
- **Collection panel empty** — every owned copy is already in the deck (possible
  when the player owns 10 or fewer copies total). The panel shows nothing to add;
  focus rests in the Deck panel.
- **Deck panel empty** — nothing placed yet; the readout reads 0/10 (alert), focus
  rests in the Collection panel.

## Design requirements

- **Card-frame vocabulary (spec 002).** Every card in both panels is a card frame
  with its face text; the cursored card carries the heavy border and the shared
  selection pulse, and nothing else does. No third border weight — the panel a card
  sits in is what conveys "in the deck." Monochrome, no new glyphs beyond
  box-drawing.
- **Two clearly labeled panels** that read as *Collection* and *Deck* at a glance,
  with the N/10 readout owned by the Deck side.
- **Fits the 89×31 minimum terminal** (today's builder constraint): both panels,
  their labels, the readout, and the controls hint. If a panel can hold more cards
  than fit, it degrades gracefully (the plan decides scroll vs. paging).
- **The most frequent action — moving a card across — is one keypress** on the
  highlighted card, taught in the established `·`-separated controls hint.
- **Deck edits still go through the profile's own add/remove methods**, so
  validation stays in one place; the screen only expresses intent, exactly as
  today.
- **Empty states are explicit** (see the flows), not a blank half-screen with no
  cue.

## Acceptance criteria

- [ ] The Side Deck menu item opens a two-panel builder: **Collection** (left, the
      copies you own and haven't placed) and **Deck** (right, the copies you've
      placed) with a **Deck: N/10** readout.
- [ ] Confirming on a Collection card moves one copy into the Deck (when under 10):
      the available count drops, the Deck count rises, a card that hits zero copies
      leaves its panel, and the readout updates.
- [ ] Confirming on a Deck card moves one copy back to the Collection (the reverse).
- [ ] The player can move the cursor within a panel and switch the active panel; the
      cursored card is the one pulsing.
- [ ] A deck can never exceed 10 or hold more copies of a card than are owned
      (unchanged legality); a full deck rejects further adds.
- [ ] The builder is reachable from the campaign map via a shown key and returns to
      the map on exit; the menu entry and the incomplete-deck divert still work and
      return to their origins.
- [ ] Cards render in the card-frame vocabulary with only two border weights (heavy
      = cursor, plain otherwise); the old "double border = in deck" is gone.
- [ ] Legible at 89×31 with both panels, labels, readout, and hint; no panics;
      `cargo build` clean and `cargo test` green (the move-across intent and the
      panel/cursor logic covered).

## Resolved decisions

- **Split copies by location** (human-ruled) — the Collection panel shows copies
  *not* in the deck and the Deck panel shows copies *in* it, so a duplicate can
  appear on both sides and "building" is literally moving a copy across. Chosen over
  a two-synced-views model; it makes the briefcase metaphor real and retires the
  confusing triple border.
- **Also reachable from the campaign map** (human-ruled) — a launch key on the map,
  mirroring the shop's, for between-match retooling. This is a small new capability
  *beyond* the roadmap's "presentation-only" framing, taken deliberately because it
  delivers the "retool between nodes" feel the roadmap itself calls for.
- **Deck order stays irrelevant / no reordering** — the hand is a random draw from
  the 10 (verified in `with_opponent` → `deal_hand`), so panels sort canonically and
  no reorder gesture is offered.
- **Owned cards only** — acquiring cards remains the shop's job; the builder never
  shows unowned/purchasable cards.
