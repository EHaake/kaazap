# Spec: Opponent Roster & Personalities

## Summary

Today the game has one opponent — a generic "Opponent" with a single fixed
strategy and the same side deck as the player. This spec makes the opponent
a **choice**: a small roster of named opponents, each with a distinct
**difficulty** (how aggressively it plays) and its own **side deck**, picked
from a new **opponent-select screen** reached when you start a game.

This is the first slice of the campaign (subsystem A). It has no campaign map,
economy, or deck-building yet — it's the roster and the personalities that
those later features will build on, made playable now via the select screen
so the variety is real and testable before the map exists.

## Goals

1. **A roster of distinct opponents.** Several named opponents spanning easy
   to hard, each recognizably different to play against.
2. **Personality via play style + deck.** Each opponent has its own *stand
   threshold* (an aggressive opponent pushes for higher totals; a cautious
   one stands early) and its own *side deck*, so difficulty is felt in how
   they play and what they can do — not just a label.
3. **Choose who you face.** Starting a game opens an opponent-select screen
   listing the roster (name, difficulty, a one-line description); pick one to
   begin the match. You can back out to the menu without starting.
4. **Resume faces the same opponent.** A saved mid-match game (spec 005)
   resumes against the opponent it was played against.
5. **Test coverage** for the new logic per `CLAUDE.md` — the parameterized
   AI, roster integrity, per-opponent dealing, select-screen navigation, and
   the save round-trip.

## Non-goals (explicitly deferred)

- **Player deck customization / a card collection** — spec B; the player
  keeps the current default side deck here.
- **Credits, shop, card rewards, progression** — spec C.
- **The campaign map / planet nodes / opponent ordering into a campaign** —
  spec D. The select screen is a flat quick-play list, not the campaign.
- **Richer AI** — stochastic "misplays," bluffing, or player-aware/positional
  play (standing because it's ahead of *your* board). The AI stays
  deterministic and plays only its own hand, as it does today; variety comes
  from threshold + deck. Bespoke signature-opponent strategies are a later
  spec.
- **Opponent portraits / dialogue / taunts** — names, a difficulty label, and
  a one-line blurb only.

## Key user flows

### Choosing an opponent

From the start menu you activate **Start Game**. Instead of dropping straight
into a match, an **opponent-select screen** appears: a vertical list of the
roster, each row showing the opponent's name, difficulty, and a short
description, navigated with the same keys as the menu (arrows / `w`·`s` /
emacs `Ctrl+P`·`Ctrl+N`, confirm with Enter/Space). You pick one and the
match begins against them. **Esc** (or `x`) returns to the start menu without
starting a game. If a saved match already exists, the existing "discard your
saved match?" confirm still appears first; confirming leads to the select
screen.

### Playing a chosen opponent

The match plays exactly as today, except the board shows the chosen
opponent's **name**, and the opponent **plays to its own difficulty** — a
cautious opponent stands sooner and reaches 20 less often; an aggressive one
pushes higher and busts more — using its own side deck.

### Resuming

**Continue** resumes a saved match against the opponent it was played
against (name and play style intact). A save file from before this spec
resumes against the original generic opponent, unchanged.

## Design requirements

- **Consistent with existing navigation.** The select screen uses the same
  list-navigation and selection feel as the start menu (including the
  selection pulse and the emacs synonyms), and the same "centered bordered"
  visual vocabulary.
- **No regression.** Every existing control, the match flow, and the current
  AI behavior for the default opponent are unchanged; only the opponent you
  face becomes selectable.
- **Difficulty is legible.** The select screen tells the player, at a glance,
  roughly how hard each opponent is.

## Acceptance criteria

- [ ] Activating **Start Game** opens an opponent-select screen listing the
      roster (name, difficulty, description); with a save present, the
      discard-confirm precedes it.
- [ ] Selecting an opponent starts a match against them; the board shows that
      opponent's name.
- [ ] Different opponents play differently — a lower-threshold opponent
      stands at a total where a higher-threshold one would hit (and vice
      versa) — and draw from their own side decks.
- [ ] **Esc**/`x` on the select screen returns to the start menu without
      starting a match.
- [ ] The select screen responds to arrows, `w`/`s`, and the emacs nav keys,
      matching the start menu.
- [ ] **Continue** resumes against the correct opponent; a pre-spec save
      resumes against the default opponent.
- [ ] Every pre-existing control and the default-opponent AI behavior are
      unchanged.
- [ ] Unit tests cover the parameterized AI, roster integrity, per-opponent
      dealing, select-screen navigation, and the save round-trip; `cargo
      test` green, `cargo build` no new warnings.

## Resolved decisions

- **Opponent-select is a `Screen`, not an overlay** (human-ruled). It's a
  full mode you navigate *to* and make a consequential choice on — the
  constitution's `Screen` criterion (like the future campaign map), not the
  "panel over the menu" overlay criterion.
- **AI variety = deterministic stand threshold + per-opponent deck**
  (human-ruled). No randomness or player-awareness this spec; it keeps the AI
  fully unit-testable and reuses the existing decision function, and difficulty
  is still clearly felt.
- **Original opponent names, not Star Wars trademarks** (see `DECISIONS.md`,
  consistent with the music IP stance). Names/blurbs are tunable flavor data.
- **Resume records the opponent's identity** so a mid-match save reloads the
  same opponent; pre-spec saves fall back to the default opponent (matching
  spec 005's tolerance for older/interrupted saves).
