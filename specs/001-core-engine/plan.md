# Plan: Core Pazaak Card Engine

**Status**: Approved
**Implements**: spec.md in this directory

Guiding constraint from `CLAUDE.md`: simplest design that satisfies the
spec. Everything below extends existing patterns (`GamePhase`,
`apply_*_action`, `Vec` rows on `PlayerState`) rather than introducing
new machinery.

## Data model / core types

`card.rs` — `LogicCard { value: i32 }` and the unused `CardKind` enum
are replaced by:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipKind { TwoFour, ThreeSix }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Dealer(u8),     // main-deck draw, 0–10 (intentional variant)
    Plus(u8),       // +N, 1–6
    Minus(u8),      // -N, 1–6
    PlusMinus(u8),  // ±N, sign chosen at play time
    Flip(FlipKind), // 2&4 or 3&6 — board effect, no value of its own
    Tiebreaker,     // ±1, wins otherwise-tied rounds
}
```

`Card` gets a `label() -> String` method for rendering ("7", "+4", "-3",
"±2", "2&4", "3&6", "±1T") — one source of truth for card text, used by
`board.rs` instead of `c.value.to_string()`.

Table entries need a *current* signed value (flips mutate signs after
play) plus the card identity (tiebreaker detection, flip-card labels):

```rust
#[derive(Debug, Clone, Copy)]
pub struct PlayedCard {
    pub card: Card, // identity: tiebreaker checks, "2&4" label
    pub value: i8,  // current signed contribution; 0 for flip cards
}
```

Both `dealer_row` and `played_row` on `PlayerState` become
`Vec<PlayedCard>` (dealer entries: `card: Dealer(n), value: n as i8` —
dealer cards must be flippable too, per spec). `score()` stays a sum of
`value` over both rows — unchanged in spirit. Rendering rule: flip cards
draw their label; everything else draws its signed `value` (a ± card
played as −3 shows "-3").

The default deck is a constant:

```rust
pub const DEFAULT_SIDE_DECK: [Card; 10] = [ /* per spec */ ];
```

with a `deal_hand(rng) -> Vec<Option<Card>>` that samples 4 distinct
cards (`choose_multiple` from `rand`, already a dependency). Called for
both sides in `GameState::new()` and `new_game()` — the latter is the
actual behavior change (hands currently persist forever).

## Architecture / game flow changes

All in `game.rs`, extending the existing state machine — no new modules.

- **New phase**: `GamePhase::AwaitingSignChoice { hand_index: usize }`,
  entered when the player plays a `PlusMinus` or `Tiebreaker` card (card
  stays in hand until committed). `game_action_from_key` is already
  `&self`, so it becomes phase-aware: in this phase, `+`/`-` (also
  `1`/`2`) map to a new `GameAction::ChooseSign { positive: bool }`, and
  a cancel key returns to `PlayerTurn` with the card unspent. All other
  phases keep the existing key mapping untouched.
- **One shared commit path**: `commit_play(side, index, value: i8)`
  removes the card from the hand and pushes `PlayedCard`. Fixed-value
  cards commit immediately with their face value; sign-choice cards
  commit with the chosen value (player: from the prompt; opponent: the
  AI passes it directly — `OpponentAction::PlayHand` gains a `value`
  field, no prompt for the AI). `Flip` cards route to `apply_flip`
  instead of contributing a value.
- **`apply_flip(kind)`**: negate `value` on every `PlayedCard` in all
  four rows where `value.abs()` matches the kind's pair (2/4 or 3/6),
  skipping zeros. Then run the existing resolution logic for *both*
  players — including the spec's ruling that a standing player pushed
  over 20 busts. *(Correction at T005: `resolve_after_action` needed no
  change — its bust checks never consulted `stood`, so a standing player
  pushed over 20 already busts. Proven by test, not assumed:
  `flip_busts_a_standing_player_pushed_over_twenty`.)*
- **Tiebreaker resolution**: in `finalize_round`'s `Tied` branch, check
  each side's `played_row` for a `Tiebreaker` entry. Exactly one side
  has one → that side wins the round; both or neither → tie stands
  (per spec ruling).
- **Opponent AI** (deliberately minimal, per spec non-goal): the
  existing "play a card that hits exactly 20" predicate generalizes to
  "any playable value of this card equals the target" — `Plus(n)` → +n,
  `Minus(n)` → −n, `PlusMinus(n)` → ±n, `Tiebreaker` → ±1. The AI does
  **not** play flip cards in this spec (see Known limitations). Stand
  threshold and hit logic unchanged.

`app.rs` needs no structural change — sign-choice keys arrive through
the existing `handle_game_input` → `apply_game_action` path.

`board.rs`: card text switches to `Card::label()` / `PlayedCard.value`;
the sign-choice prompt renders in the existing turn-text area (e.g.
"Play +3 or -3?  [+/-]  (c to cancel)") when the phase is
`AwaitingSignChoice`. Card-kind distinction is carried by label text
alone — the `Frame` is `Vec<Vec<char>>`, so color is out of scope (see
Known limitations).

## Known limitations

- **AI ignores flip cards.** Playing a flip well is genuine strategy —
  exactly what the opponent-personalities spec exists for. Until then
  the opponent treats a drawn flip card as unplayable. Acceptable: the
  spec pins this opponent to "the same existing simple AI."
- **No color/visual styling for card kinds** — the char-based `Frame`
  has no color support, so kind distinction rides on labels ("±3" vs
  "+3" vs "2&4"). Revisit only if labels prove illegible in play.
- **Sub-prompt is interim UI** — replaced wholesale when the
  cursor-selection spec lands (see `ROADMAP.md`); engine API already
  takes the resolved value as a parameter, so that swap won't touch
  game logic.

## Testing strategy

Unit tests in `#[cfg(test)]` modules in `game.rs`/`card.rs`, per
`CLAUDE.md` (game logic only; rendering verified by running the game).
State-mutation functions are already RNG-free once a hand/row exists, so
tests construct `GameState` directly — no RNG injection machinery.

- Scoring across mixed card kinds, both rows.
- Flip: inverts matching values on *both* sides including dealer rows;
  ignores non-matching and zero values; busts a standing player pushed
  over 20 (round ends, correct outcome).
- Tiebreaker: one side in play wins the tie; both sides → tie; neither
  → tie. (Mutation-check: assert the tie case actually goes red if the
  rule is inverted.)
- Sign commit: chosen sign is what lands in `played_row`.
- Dealing: hand is 4 distinct `Some` cards from the deck; both sides
  dealt independently; `new_game()` re-deals, `setup_next_round()`
  leaves hands untouched (regression for no-mid-match-redraw).
- Dealer draw bounds: helper's output stays within 0–10 across many
  draws.

## File structure

No new files, no new dependencies. Changes confined to: `card.rs`
(types, deck, labels), `player.rs` (row/hand types), `game.rs` (phase,
commit path, flip, tiebreaker, AI predicate, dealing), `board.rs`
(labels + prompt), `app.rs` (none expected). Housekeeping riding along
on this branch: drop the unused `rusty_time` dependency from
`Cargo.toml`, per `CLAUDE.md`.

## Resolved decisions

- One `Card` enum with `Dealer` as a variant, rather than separate
  main-deck/side-deck types — the spec's "distinguish main-deck from
  side-deck" is satisfied by the variant, and one enum keeps
  `PlayedCard`, labels, and tests simple.
- `PlayedCard { card, value }` rather than bare `i8` rows — flips and
  tiebreaker detection need identity; anything less loses it, anything
  more (per-card state objects) is unneeded.
- Opponent sign choice passes through the action
  (`PlayHand { index, value }`) instead of the AI going through the
  prompt phase — the prompt is player UI, not game logic.
- `rand`'s `choose_multiple` for dealing rather than hand-rolled
  index sampling.
- `PlayerState.played_card` deleted (T002 finding, ruled at T003): the
  flag was never set anywhere, so playing a side card has always kept
  the turn with the player. That matches real Pazaak (play a card, then
  still hit or stand), so the behavior stays and the dead flag and its
  never-taken pass-the-turn branch in `resolve_after_action` are gone.
