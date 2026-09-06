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
   Collection. Counts on both sides update, and a card whose copies on a side reach
   zero becomes a placeholder there (see goal 6).
3. **Switch focus between panels.** The player can move the cursor within a panel
   and switch which panel is active.
4. **Keep the deck readout.** A "Deck: N/10" readout shows how close the deck is to
   legal, alerting while it's short (an incomplete deck is the only thing that
   blocks starting a match).
5. **Reachable from the campaign map.** Add a key on the map to open the builder
   and return to the map on exit, for between-match retooling — alongside the
   existing menu entry and the incomplete-deck divert.
6. **A card album with placeholders.** Both panels show the full set of card types
   in fixed slots (canonical order): a type present in that panel renders as a solid
   card with its `×count`; a type absent renders as a faint **ghosted slot** — corner
   ticks + the card's dimmed face — which fills in solid once you have one there. The
   old "double = in deck" weight is retired; a card's *panel* conveys deck membership,
   and the vocabulary is heavy (cursor), plain (present), and a faint corner-tick ghost
   (placeholder).
7. **No regression.** Deck legality (exactly 10, never more copies than owned), the
   menu entry, and the save format are all unchanged. The incomplete-deck divert's
   *mechanism* is unchanged; its return path is corrected to route back to the
   launching screen (see Resolved decisions) — today it wrongly returns to the menu.

## Non-goals (explicitly deferred)

- **Buying / acquiring cards in the builder.** The album shows the full card
  universe — types you don't own appear as faint placeholders so you can see what's
  missing — but you cannot *acquire* cards here; that stays the shop's job (spec
  012). The builder shows the gaps; it never sells or grants.
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
- **Card album** — both panels show **all** card types in the universe, in fixed
  canonical-order slots. A slot is *filled* (solid card + count) when its type is
  present in that panel, or a *placeholder* (a faint ghosted slot — corner ticks + the
  dimmed face) when absent.
- **Collection panel** — the left panel: each type filled with its *available*
  count (owned − in-deck) when > 0, else a placeholder.
- **Deck panel** — the right panel: each type filled with its *in-deck* count when
  > 0, else a placeholder; plus the N/10 readout.

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
- **A panel with nothing present** — the panel is never blank: every slot renders,
  as a placeholder where the type is absent. A brand-new deck shows an
  all-placeholder Deck panel; a fully-decked collection shows placeholders on the
  Collection side. The cursor never lands on a placeholder — it rests on a present
  card, and a panel with no present cards isn't focusable (nothing to select or move
  there).

## Design requirements

- **Card-frame vocabulary (spec 002).** Present cards are card frames with their face
  text; the cursored card carries the **heavy** border and the shared selection pulse
  (and nothing else does); every other present card is **plain** (single). A placeholder
  is a **faint ghosted slot** — just corner ticks (dim) plus the card's dimmed face, no
  edges — so it's clearly sparser than an owned card and recedes rather than competing
  with it. All monochrome box-drawing; no emoji/icons.
- **Two clearly labeled panels** that read as *Collection* and *Deck* at a glance,
  with the N/10 readout owned by the Deck side.
- **Content-sized panels, no scrolling.** Each panel's border hugs a fixed grid
  sized to hold the whole bounded card set (every type shown at once, filled or
  placeholder) — not a half-screen rectangle with empty space, and never a scroll or
  pager. This whole album — both panels, labels, readout, hint — fits the **89×31
  minimum terminal**.
- **The most frequent action — moving a card across — is one keypress** on the
  highlighted card, taught in the established `·`-separated controls hint.
- **Deck edits still go through the profile's own add/remove methods**, so
  validation stays in one place; the screen only expresses intent, exactly as
  today.
- **Empty states are explicit** (see the flows), not a blank half-screen with no
  cue.

## Acceptance criteria

- [ ] The Side Deck menu item opens a two-panel builder — **Collection** (left) and
      **Deck** (right), each a fixed album of every card type — with a **Deck: N/10**
      readout. Present types show a solid card + `×count` (available on the left,
      in-deck on the right); absent types show a faint ghosted-slot placeholder (corner
      ticks + the card's dimmed face).
- [ ] Confirming on a Collection card moves one copy into the Deck (when under 10):
      the available count drops, the Deck count rises, a card whose count hits zero
      becomes a placeholder in that panel, and the readout updates. Confirming on a
      placeholder does nothing.
- [ ] Confirming on a Deck card moves one copy back to the Collection (the reverse).
- [ ] The player can move the cursor within a panel and switch the active panel; the
      cursored card is the one pulsing.
- [ ] The cursor only lands on **present (solid) cards** — placeholders are not
      navigable or selectable. Movement skips them, and a panel with no present cards
      can't be focused (nothing to select there).
- [ ] A deck can never exceed 10 or hold more copies of a card than are owned
      (unchanged legality); a full deck rejects further adds.
- [ ] The builder is reachable from the campaign map via a shown key and returns to
      the map on exit; the menu entry and the incomplete-deck divert still work and
      return to their origins.
- [ ] Cards render in two border weights — heavy (cursor) and plain (present) — with
      placeholders as a faint corner-tick ghost, not a full frame; the old "double
      border = in deck" is gone.
- [ ] The panels are content-sized (borders hug the fixed card grid) with no
      scrolling — the full album, both panels, labels, readout, and hint are legible
      at 89×31; no panics; `cargo build` clean and `cargo test` green (the move-across
      intent, the filled/placeholder album split, and the panel/cursor logic covered).

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
- **Full-universe album with placeholders** (human-ruled at the first visual review,
  refining the initial cut) — both panels show every card type in fixed slots; a type
  absent from a panel (unowned, or all-decked / not-decked) is a faint ghosted-slot
  placeholder (corner ticks + the card's dimmed face) that fills in once present. This
  **supersedes the earlier "owned cards only" view** — you now see the whole set and
  your gaps — while acquiring cards stays the shop's job (you still can't buy or grant
  in the builder). It also makes the panels **content-sized** and **removes scrolling
  entirely** (the grid always shows the full bounded set at once).
- **Return-path correction (a bug fix, not a new behavior)** — routing the builder's
  Back to its launching screen corrects a pre-existing bug: the campaign "incomplete
  deck" divert currently returns to the menu instead of the map. Surfaced during
  planning; the divert *mechanism* itself is untouched.
- **Placeholders are not navigable** (human-ruled at a visual review) — the selection
  cursor only lands on present (solid) cards, the ones actionable in that panel;
  movement skips placeholders and a panel with none isn't focusable. Selecting a card
  you can't act on carried no meaning.
- **Placeholder style: a sparse ghosted slot** (human-ruled across the visual reviews)
  — placeholders began as a dimmed dashed frame, then became just faint corner ticks +
  the dimmed face, because a full dashed border read as "almost a real card." Sparser so
  owned cards clearly dominate.
