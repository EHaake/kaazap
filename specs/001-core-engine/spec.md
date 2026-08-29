# Spec: Core Pazaak Card Engine

**Status**: Draft — pending review
**Depends on**: nothing (first feature spec)

## Summary

Replace Kaazap's placeholder card model (`LogicCard { value: i32 }`) with
a real card-type system supporting Pazaak's full card variety — plus,
minus, plus-or-minus (±), flip (2&4, 3&6), and tiebreaker cards — along
with the game-logic changes needed to score and resolve rounds correctly
with them. A single signed integer can represent a fixed-value card, but
not a card whose sign is a *choice made at play time* (± cards, the
tiebreaker) or whose effect goes beyond its own value (flip cards'
board manipulation, the tiebreaker's resolution effect) — hence a real
type model rather than a wider integer. Existing behavior (dealer
draws, fixed-value hand cards) maps into the new model unchanged. This is the foundational spec every later system (opponent
personalities, campaign/progression, persistence) builds on top of.

## Goals

1. Replace `LogicCard { value: i32 }` with a real card type model
   distinguishing main-deck (dealer) cards from side-deck (player/
   opponent) cards, and among side-deck cards: plus, minus,
   plus-or-minus, flip, and tiebreaker variants.
2. Implement plus-or-minus cards (±N): the player chooses the sign when
   playing the card, via an inline sub-prompt.
3. Implement flip cards (2&4, 3&6) with canon behavior: playing one
   inverts the sign of every matching-value card on the table, on both
   sides, and both totals recalculate immediately.
4. Implement the tiebreaker card: plays as +1 or −1 (player's choice);
   if the round would otherwise end tied while it's in play, the
   tiebreaker's owner wins the round instead.
5. Ship one fixed default side deck (not player-customizable) containing
   a mix of all card kinds above; each new game draws a random 4-card
   hand from it, for both player and opponent.
6. Preserve existing, correct behavior: main deck still draws 0–10
   (intentional variant — see `DECISIONS.md`, unchanged by this spec), no
   mid-match hand redraw, best-of-3-round match structure.
7. Unit test coverage for all new/changed game logic (scoring, card
   effects, round resolution), per `CLAUDE.md`.

## Non-goals (explicitly deferred)

- **Full side-deck customization** (building/collecting your own deck) —
  deferred; v1's default deck is fixed content, not player-editable.
  Revisit once the campaign loop exists (see `ROADMAP.md`).
- **Campaign/progression system** (currency, card packs, unlocking
  cards) — this spec makes all card kinds available in the fixed default
  deck; *who* has access to which specific cards over a campaign is the
  next spec's job, not this one's.
- **Opponent-specific decks or personalities** — the opponent in this
  spec draws from the same fixed default deck as the player, with the
  same existing simple AI. Distinct opponent decks/strategies are a
  later spec.
- **Mid-match hand redraw** — not in this spec: v1 keeps real Pazaak's
  no-redraw behavior. Deferred rather than rejected — a redraw mechanic
  is a candidate for the post-v1 rule-enhancement pass (see
  `ROADMAP.md`), once a complete baseline game exists to evaluate
  variants against.
- **"How to Play" menu wiring** — separate, small, unrelated fix. Note
  that the overlay's rules text will need updating once these new card
  types exist, but that's a follow-up, not part of this spec.

## Entities

- **Card** — a single card. Main-deck cards are dealer draws (value
  0–10). Side-deck cards belong to the player or opponent's hand and
  have a **kind**:
  - *Plus* / *minus* — one fixed value (e.g. "+4", "-3").
  - *Plus-or-minus (±N)* — one magnitude, sign chosen by the player when
    played (e.g. "±2").
  - *Flip (2&4, 3&6)* — adds no value of its own; when played, inverts
    the sign of every card on the table (both sides) whose value
    matches, ignoring sign — 2s and 4s, or 3s and 6s respectively.
  - *Tiebreaker* — plays as +1 or −1 (sign chosen at play time, like a
    ± card), plus a tie-breaking effect on round resolution.
- **Default side deck** — the fixed pool side-deck cards are drawn from.
  Not player-visible as a collection to manage in v1 — just the source
  a hand gets dealt from. Ten cards, matching real Pazaak's side-deck
  size: +2, +4, −2, −4, ±1, ±3, ±6, 2&4, 3&6, and the ±1 tiebreaker.
- **Hand** — the 4 side-deck cards a player (or opponent) has for the
  current game (all rounds within it). Cards are removed as played and
  not replenished until the next game deals a new hand.

## Key user flows

### Starting a new game

When a new game begins, both the player and the opponent are each dealt
a fresh, independently-drawn random 4-card hand from the fixed default
side deck. Each side draws 4 distinct cards without replacement from its
own copy of the deck — no duplicates within a hand, and the two sides'
draws don't affect each other. This replaces the current behavior of a
single hardcoded hand reused forever.

### Playing a ± card (sign choice)

When the player selects a plus-or-minus card (±N, including the
tiebreaker) from their hand, an inline sub-prompt appears asking which
sign to apply (e.g. "+3 or −3?"), answered with a single keypress; the
card is then committed with the chosen sign. This keeps the existing
direct-keypress play model (keys 1–4) intact. A unified cursor-selection
interaction model (navigate cards with arrows, toggle value, confirm to
play) is deliberately deferred to a later UI-focused spec — see
Resolved decisions.

### Playing a flip card (2&4 or 3&6)

Flip cards need no play-time choice: playing one immediately inverts the
sign of every card on the table whose value matches (ignoring sign) —
the player's and the opponent's rows alike, dealer-drawn and hand-played
cards alike. A 2&4 turns +2s and +4s into −2s and −4s and vice versa;
3&6 likewise for 3s and 6s. Both totals recalculate immediately. If a
player who is already standing has their recalculated total pushed over
20, that player busts (explicit ruling — see Resolved decisions).

### Playing a tiebreaker card

The tiebreaker plays like a ± card: the player chooses +1 or −1 when
playing it. At round resolution, if the player's and opponent's final
totals are equal (both standing, neither busted) and exactly one side
has a tiebreaker in play from that round, that side wins the round
instead of it resulting in a tie. If both sides have one in play, they
cancel out and the tie stands (explicit ruling — the original game's
sources don't cover this case).

## Design requirements

- The hand display needs to show enough information to distinguish card
  kinds at a glance (e.g. a ± card should look visibly different from a
  fixed plus/minus card, and flip cards different again, before they're
  played) — exact visual treatment is an implementation detail for
  `plan.md`, but the requirement itself belongs here.
- Whatever interaction is chosen for ± sign selection
  must be discoverable without consulting external docs — either
  self-evident from the prompt itself or covered by the (currently dead,
  separately tracked) "How to Play" overlay.

## Acceptance criteria

- [ ] Main deck still draws 0–10 inclusive, unchanged.
- [ ] Side-deck cards exist in kinds: plus, minus, plus-or-minus (±),
      flip (2&4, 3&6), and tiebreaker.
- [ ] Playing a ± card prompts the player to choose + or − before it's
      applied to their total.
- [ ] Playing a 2&4 or 3&6 flip card inverts the sign of every
      matching-value card on the table (both sides), and both totals
      update immediately.
- [ ] Playing a tiebreaker card applies +1 or −1 (player's choice), and
      if the round would otherwise end tied with exactly one side's
      tiebreaker in play, that side wins the round instead.
- [ ] At the start of each new game, both player and opponent draw an
      independent, fresh random 4-card hand from the fixed default side
      deck.
- [ ] A played hand card doesn't return until the next game starts (no
      mid-match redraw) — unchanged behavior, covered by a regression
      test.
- [ ] `cargo test` passes, with new unit tests covering scoring and
      round resolution for every new card kind.
- [ ] `cargo build` succeeds with no new warnings introduced by this work.

## Resolved decisions

- Main deck's 0–10 range: intentional, not a bug (see `DECISIONS.md`) —
  unchanged by this spec.
- No mid-match side-deck redraw in v1 — matches real Pazaak. Kept open
  as a possible post-v1 rule enhancement rather than permanently ruled
  out.
- No full side-deck customization in v1 — confirmed; default deck is
  fixed content, not player-editable.
- Minus cards ship in the v1 default deck rather than being deferred to
  the campaign spec — confirmed.
- Default side deck (10 cards; each side draws 4 without replacement per
  game): +2, +4, −2, −4, ±1, ±3, ±6, 2&4, 3&6, ±1 tiebreaker.
  Composition delegated to Claude — weighted toward minus/± cards since
  the dealer only ever adds, values spread so choices stay meaningful.
  One tunable constant in code; expected to be rebalanced by the
  campaign spec.
- Tiebreaker corrected to match the original game: plays as ±1 (sign
  chosen at play time) and wins otherwise-tied rounds while in play.
  Explicit ruling: if both sides have one in play, the tie stands. (Was
  drafted as fixed +1; corrected against KOTOR sources.)
- Play-time sign choice (± cards, tiebreaker) uses an inline sub-prompt,
  preserving the direct 1–4 keypress play model. The cursor-selection
  interaction model (arrow-key card selection, value toggling,
  confirm-to-play) is deferred to a later UI-focused spec — the campaign
  screens (shop, pack opening, opponent select) will need cursor
  selection anyway, so the model gets designed once with all its use
  cases known. Engine API takes the chosen sign as a parameter, so
  swapping the UI later touches no game logic.
- 2&4 / 3&6 use canon flip mechanics: invert the sign of every
  matching-value card on the entire table (both sides), contributing no
  numeric value of their own. Terminology aligned with KOTOR
  accordingly: 2&4/3&6 are the *flip* cards; ±N cards are
  *plus-or-minus* cards.
- Ruling (canon sources thin): flips recalculate both totals
  immediately, and a standing player whose total is pushed over 20 by a
  flip busts. Revisit in playtesting if it plays badly.
