# Spec: Core Pazaak Card Engine

**Status**: Draft — pending review
**Depends on**: nothing (first feature spec)

## Summary

Replace Kaazap's placeholder card model (`LogicCard { value: i32 }`) with
a real card-type system supporting Pazaak's full card variety — plus,
minus, flip (±), and special cards (2&4, 3&6, tiebreaker) — along with
the game-logic changes needed to score and resolve rounds correctly with
them. A single signed integer can represent a fixed-value card, but not
a card whose value is a *choice made at play time* (flip, 2&4, 3&6) or
one with behavior beyond its value (tiebreaker) — hence a real type
model rather than a wider integer. Existing behavior (dealer draws,
fixed-value hand cards) maps into the new model unchanged. This is the foundational spec every later system (opponent
personalities, campaign/progression, persistence) builds on top of.

## Goals

1. Replace `LogicCard { value: i32 }` with a real card type model
   distinguishing main-deck (dealer) cards from side-deck (player/
   opponent) cards, and among side-deck cards: plus, minus, flip, and
   special variants.
2. Implement flip cards (±N): the player chooses the sign when playing
   the card.
3. Implement dual-value special cards (2&4, 3&6): the player chooses
   which of the two listed values applies when playing the card.
4. Implement the tiebreaker card: adds +1 to the player's total; if the
   round would otherwise end tied, the tiebreaker's owner wins instead.
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
  - *Flip* — one magnitude, sign chosen by the player when played (e.g.
    "±2").
  - *Special (2&4, 3&6)* — two fixed possible values; the player chooses
    one when played.
  - *Tiebreaker* — fixed +1 value, plus a tie-breaking effect on round
    resolution.
- **Default side deck** — the fixed pool side-deck cards are drawn from.
  Not player-visible as a collection to manage in v1 — just the source
  a hand gets dealt from. Size and exact composition: open, see below.
- **Hand** — the 4 side-deck cards a player (or opponent) has for the
  current game (all rounds within it). Cards are removed as played and
  not replenished until the next game deals a new hand.

## Key user flows

### Starting a new game

When a new game begins, both the player and the opponent are each dealt
a fresh, independently-drawn random 4-card hand from the fixed default
side deck. This replaces the current behavior of a single hardcoded hand
reused forever.

### Playing a flip or special card

**[OPEN — needs a decision, not just card logic]** When the player
selects a flip card or a dual-value special card (2&4, 3&6) from their
hand, they need some way to indicate which value/sign to apply before
it's committed. Options worth considering: a small inline sub-prompt
("+2 or -2?"), cycling the displayed value with a keypress before
confirming, or a dedicated selection overlay. Whichever is chosen should
fit the existing input-handling pattern in `app.rs` rather than
introducing a new, one-off interaction model.

### Playing a tiebreaker card

Playing the tiebreaker card adds +1 to the player's total as normal. At
round resolution, if the player's and opponent's final totals are equal
(both standing, neither busted) and either side played a tiebreaker card
that game, that side wins the round instead of it resulting in a tie.

## Design requirements

- The hand display needs to show enough information to distinguish card
  kinds at a glance (e.g. a flip card should look visibly different from
  a fixed plus/minus card before it's played) — exact visual treatment
  is an implementation detail for `plan.md`, but the requirement itself
  belongs here.
- Whatever interaction is chosen for flip/special card value selection
  must be discoverable without consulting external docs — either
  self-evident from the prompt itself or covered by the (currently dead,
  separately tracked) "How to Play" overlay.

## Acceptance criteria

- [ ] Main deck still draws 0–10 inclusive, unchanged.
- [ ] Side-deck cards exist in kinds: plus, minus, flip, special (2&4,
      3&6), and tiebreaker.
- [ ] Playing a flip card lets the player choose + or - before it's
      applied to their total.
- [ ] Playing a 2&4 or 3&6 card lets the player choose between the two
      listed values before it's applied.
- [ ] Playing a tiebreaker card adds 1 to the total, and if the round
      would otherwise end tied, the tiebreaker's owner wins instead.
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
- **[OPEN]** Exact size/composition of the fixed default side deck (how
  many of each card kind, what specific values).
- **[OPEN]** UI interaction for choosing a flip card's sign or a special
  card's value at play time.
- **[OPEN]** Confirm the special-card mechanics as drafted above (flip =
  choose sign; 2&4/3&6 = choose one of two values; tiebreaker = +1 and
  wins ties) match your memory of the real rules.
