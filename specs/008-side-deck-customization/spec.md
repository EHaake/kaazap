# Spec: Collection & Side-Deck Customization

## Summary

Today the player always plays with the same fixed side deck (a hardcoded
`DEFAULT_SIDE_DECK`). This spec gives the player a **collection of cards they
own** and lets them **build their own 10-card side deck** from it on a new
**deck-builder screen** reached from the start menu. Matches then deal the
player's hand from the deck they built.

This is the second slice of the campaign (subsystem B). It introduces the
**player profile** — a persistent, player-owned document (`profile.json`)
separate from the mid-match save (spec 005) — that the economy (spec C) and
campaign map (spec D) will later *extend* rather than replace. There is no
way to *earn* new cards yet (that's the economy, spec C); the collection is a
fixed starter set here, and the point of this spec is the ownership model and
the building itself.

## Goals

1. **A player card collection.** The player owns a bag of cards — possibly
   several copies of the same card — persisted across sessions in a profile
   file.
2. **Build your own side deck.** A deck-builder screen shows the collection
   and lets the player add and remove cards to assemble a **10-card** side
   deck, classic-Pazaak style (duplicates allowed, up to the number owned).
3. **Matches use the built deck.** Starting a match deals the player's hand
   from their built deck instead of the fixed default.
4. **It persists.** The collection and the built deck survive quitting and
   relaunching; a resumed mid-match game keeps dealing from the deck it began
   with.
5. **Test coverage** per `CLAUDE.md` — the profile (load/save, versioning,
   deck validity, add/remove rules, grouped counts), the engine seam (matches
   deal from the built deck), the builder navigation, and the save round-trip.

## Non-goals (explicitly deferred)

- **Earning cards** — credits, a shop, per-win card drops, unlock-by-depth.
  All spec C. The collection is a fixed starter set in this spec.
- **New card *types*.** The side-card universe is already complete (15
  distinct cards); this spec adds no new ones. Campaign scarcity is about
  *distribution* (copies + credits), not new types — see `DECISIONS.md`.
- **Editing opponents' decks.** Opponents keep their own decks (spec 007);
  this spec only touches the *player's* deck.
- **Mid-match deck changes.** You build your deck from the menu, not during a
  match; the deck is fixed for the duration of a match (and its rematches).
- **A card-detail / stats view, drag-and-drop, or multi-deck loadouts.** One
  deck, a compact grid, add/remove.

## Key user flows

### Building a deck

From the start menu you activate **Side Deck**. A deck-builder screen appears:
a compact grid of the cards you own, each cell showing the card and a small
**in-deck / owned** count (e.g. `2/3`), with a running **"Deck: N/10"**
readout and a controls hint. Navigate the grid with arrows / `w`·`a`·`s`·`d` /
the emacs nav keys. **Enter/Space** adds one copy of the highlighted card to
your deck (if you own a spare and the deck isn't full); **Backspace** (or `-`)
removes one copy. Your changes are saved as you make them. **Esc** (or `x`)
returns to the start menu.

### Playing with your deck

When you start a match, your hand is dealt from the deck you built. If your
deck isn't a full 10 cards, starting a game takes you to the deck-builder
first (the "Deck: N/10" readout, shown in an alert style while incomplete,
tells you what's missing) rather than starting an under-strength match.

### Persistence & resuming

Your collection and built deck are saved to a profile file and restored on
next launch. **Continue** resumes a saved match dealing from the deck that
match started with — editing your deck from the menu afterward changes future
matches, not the one already in progress. A profile file from before this
spec (or none at all) yields the starter collection and deck.

## Design requirements

- **Consistent with existing navigation and look.** The builder uses the same
  centered, monochrome, bordered vocabulary as the rest of the game, the same
  selection pulse, and the same nav synonyms (arrows / `wasd` / emacs). Cards
  are drawn with the existing card visual.
- **The deck rule is legible.** The "Deck: N/10" readout always shows how
  close the deck is to legal, and signals clearly when it isn't yet playable.
- **No regression.** With an untouched (starter) profile, play is exactly as
  before — the starter deck equals today's default deck, so a player who never
  opens the builder sees no change.
- **Centralized validation.** The rules (deck ≤ 10, only cards you own, a copy
  count you can't exceed) live in the profile's own mutation methods, not
  scattered in the UI — matching the engine's `apply_*_action` discipline.

## Acceptance criteria

- [ ] The start menu has a **Side Deck** item that opens a deck-builder screen
      showing the player's owned cards in a grid with per-card in-deck/owned
      counts and a **Deck: N/10** readout.
- [ ] **Enter/Space** adds a copy to the deck (blocked at 10 cards, or when no
      unused copy of that card is owned); **Backspace**/`-` removes a copy; the
      counts and readout update live.
- [ ] The builder responds to arrows, `w`/`a`/`s`/`d`, and the emacs nav keys;
      **Esc**/`x` returns to the start menu.
- [ ] Changes persist: rebuild a deck, quit, relaunch — the collection and
      deck are as left.
- [ ] Starting a match deals the player's hand from the built deck (a card put
      *only* in the built deck can appear in hand; a default-only card can't).
- [ ] Starting a game with an incomplete (≠10) deck routes to the builder
      rather than starting the match.
- [ ] **Continue** resumes dealing from the deck the match began with; a
      pre-spec / missing profile yields the starter collection and deck, and a
      pre-spec match save resumes with the default deck.
- [ ] With a starter profile and the builder never opened, play is identical to
      before this spec (starter deck == old default deck).
- [ ] Unit tests cover the profile (load/save, version discard, deck validity,
      add/remove, grouped counts), the engine seam, builder navigation, and the
      save round-trip; `cargo test` green, `cargo build` no new warnings.

## Resolved decisions

- **Collection is a bag of copies, not a set of types** (human-ruled). You own
  N copies of a card and may run up to N in your deck (duplicates allowed,
  classic Pazaak). This is the model spec C's per-win drops and shop purchases
  grow with no rework.
- **The starter collection is modest, seeded by this spec** (human-ruled).
  Since the economy (spec C) doesn't exist yet to grow it, B seeds a starting
  collection: today's default 10-card deck plus a small handful of spare
  adjusters. Because the 15-card universe is already 10 types deep in the
  default deck, "room to grow" is necessarily small at the type level — the
  real growth lever is copies + credits in spec C. The exact starter list is
  tunable flavor/balance data, revisited in C's balance pass.
- **Decks are exactly 10 cards to play** (human-ruled). The classic size; the
  builder lets the deck sit below 10 while editing but a match requires 10, so
  an incomplete deck routes back to the builder.
- **The deck-builder is a `Screen`, not an overlay** (per the constitution) —
  a full mode you navigate *to* and make consequential edits on, like the
  opponent-select screen and the future campaign map.
- **The profile is a new persistent document** modeled on the settings file
  (self-owned serde struct, load-with-defaults, best-effort save) with the
  match save's version-discard discipline. Distinct from the match save; later
  specs extend it.
- **A match snapshots the deck it started with** (in the match save), so
  editing your deck from the menu never retroactively rewrites an in-progress
  saved match; rematches within that match use the snapshot.
