# Spec: Smarter (board-aware) opponent AI

## Summary

Today every opponent plays *solitaire* — it draws toward its own private
`stand_threshold` and stands there, **never looking at your board**. So it will
stand at 15 while you've stood at 19, not even trying to beat you. Matches feel
arbitrary rather than adversarial, and the roster's "distinct personalities" are
only a label — mechanically the five opponents differ by one number.

This spec makes opponents **read your visible total and play to win the round**,
gives each opponent a distinct **strategy** so the roster finally plays like its
personalities, and adds **a dash of randomness** — occasional imperfect play,
more from easy opponents and essentially none from the boss — so a learned
opponent isn't perfectly exploitable and matches feel a touch more human.

It's the single highest-leverage improvement to the moment-to-moment
experience, and it makes the difficulty curve real: an easy opponent slips, the
Master plays tight.

## Goals

1. **Board-aware play.** When you've stood, the opponent knows your final total
   and plays to beat it — standing when it's already ahead (instead of over- or
   under-playing to its own number), chasing when it's behind, and using its
   hand cards to land a winning total without busting.
2. **Distinct strategies.** Each opponent has a strategy archetype (naive /
   aggressive / cautious / calculating) that changes *how* it plays — how hard
   it pushes, how much bust risk it accepts, whether it computes the safest
   winning play. The five roster opponents feel individual, difficulty rising
   core-ward.
3. **A dash of randomness.** Opponents occasionally make an imperfect move —
   frequently for the rookie, rarely for the master — over an otherwise
   deterministic policy, so they don't feel like a solved machine.
4. **No regression.** The default opponent and all current match flow behave
   exactly as before when the board doesn't call for board-aware play; every
   existing rule (recovery over 20, land-on-20 auto-stand, full-table
   auto-stand, tie resolution) is unchanged.
5. **Test coverage** per `CLAUDE.md` — the deterministic decision core (board-
   aware branches + per-strategy behavior) is unit-tested against both boards;
   the randomness sits behind a seam that is itself unit-tested with explicit
   rolls.

## Non-goals (explicitly deferred)

- **The global difficulty setting** (easy/normal/hard) — its own backlog item.
  This spec makes it *meaningful* (opponents actually think, so difficulty can
  scale how well) but does not build the toggle.
- **Bluffing / deception** as a distinct system, **multi-turn lookahead**, and a
  full **expected-value solver**. The AI reasons about the current board, one
  action at a time (as the turn machine already drives it).
- **New opponents or decks.** Same five-opponent roster; only how they *think*
  changes (plus the two new profile fields).

## Key behavior

### Reading the board

The opponent decides one action at a time, and by then your latest action is
committed and visible. The decisive new logic fires when **you have stood** (your
total is final at `P`); the opponent, at total `S`:

- **`S > P` (and `S ≤ 20`) → it has already won the round → it stands.** Today
  it wrongly keeps grinding toward its own threshold. This is the headline fix.
- **`S = P` → a tie** (which it loses unless it alone has a tiebreaker on the
  table) → it tries to get above you, or stands if its tiebreaker wins it.
- **`S < P` → it's behind → it plays a hand card that lands a winning total if
  it can, otherwise it hits** (chasing is its only chance; standing behind is a
  certain loss).

While you're still live (haven't stood), the opponent can't know your final
total, so it plays to its threshold as before — flavored by its strategy.

### The strategies

- **Naive** (rookie) — sensible threshold play plus the core board-aware fix, but
  it slips often. The opponent you learn to beat.
- **Aggressive** (scrapper) — pushes for higher totals and chases hard, accepting
  bust risk.
- **Cautious** (veteran) — stands earlier, so it stops building its own hand
  before an avoidable bust; behind a stood player it still chases (hitting is its
  only chance to win).
- **Calculating** (ace / master) — targets the smallest total that beats you
  (minimizing bust risk) and uses the tiebreaker to steal a tie. The master is
  the flawless version of this.

Difficulty rises across the roster (Greeb → The Magistrate) through the strategy
*and* the error rate: the rookie misplays often, the master essentially never.

## Design requirements

- **The decision core stays deterministic and testable.** Given a board and a
  strategy, the "best" move is a pure function; the randomness is a separate,
  seam-tested layer that occasionally substitutes a legal but suboptimal move.
- **No engine plumbing or save change.** The decision function already has the
  whole game state in scope; strategy is `Copy` const data on the opponent
  profile; the save persists only the opponent id and rebuilds the profile from
  code.
- **The stable seam holds.** The decision function still returns the same action
  type, and its single caller is unchanged in shape — only the brain grows.
- **Difficulty stays legible.** An easy opponent visibly makes mistakes; a hard
  one closes out won rounds and doesn't throw them away.

## Acceptance criteria

- [x] When you stand and the opponent is already ahead (and not busting), it
      **stands** rather than continuing to its own threshold.
      *(`ai_stands_once_it_is_already_beating_a_stood_player`; live driver: the
      Magistrate stood at 9 vs a stood 5, far below its threshold of 19.)*
- [x] When you stand and the opponent is behind, it **plays a hand card that
      lands a winning total** if it has one, otherwise **hits** to chase.
      *(`ai_behind_a_stood_player_plays_a_winning_card`,
      `..._with_no_winning_card_hits_to_chase`.)*
- [x] Tie handling is correct: an equal total only wins for the side that alone
      holds a tiebreaker on the table; the AI reasons accordingly.
      *(`ai_stands_on_a_tie_it_wins_with_a_lone_tiebreaker`,
      `ai_on_a_losing_tie_tries_to_pull_ahead`,
      `ai_calculating_steals_a_tie_with_the_tiebreaker`.)*
- [x] The four strategies play observably differently (the calculating one
      targets the minimal safe winning total and steals ties, the aggressive one
      pushes high, the cautious one stands a point earlier).
      *(`ai_archetypes_pick_different_winning_totals`,
      `ai_cautious_stands_one_sooner_and_aggressive_one_later_while_live`,
      `ai_calculating_steals_a_tie_with_the_tiebreaker`.)*
- [x] Opponents occasionally misplay at a per-opponent rate — the rookie visibly,
      the master essentially never — and the default opponent never does.
      *(`opponent_action_misplays_below_the_rate_and_plays_best_at_or_above`,
      `the_default_opponent_never_misplays`,
      `misplay_rates_are_valid_and_the_default_is_deterministic`.)*
      **Amended by spec 013:** the shipped misplay was **unbounded** — contradicting
      the "bounded" intent below, it could stand on 0 or concede from far behind.
      Spec 013 bounds it (a position-open gate + a timid-stand margin); the first two
      tests above were re-authored to the bounded contract. See
      `specs/013-bounded-misplays/`.
- [x] Every pre-existing rule and the whole match flow are unchanged when the
      board doesn't call for board-aware play; no panics.
      *(All prior `ai_*` tests pass unchanged; `full_match_terminates_*`; live
      driver played two rounds through with correct resolution and no panic.)*
- [x] Unit tests cover the deterministic core (board-aware branches, per-strategy
      behavior, against both boards) and the randomness seam (explicit rolls);
      every existing `ai_*` test still passes. `cargo test` green, `cargo build`
      no new warnings. *(221 passed, 0 failed; build 0 warnings.)*

## Resolved decisions

- **Board-aware + per-opponent strategy archetypes** (human-ruled) — not a
  single shared policy (opponents must feel individual) and not a near-optimal
  solver (that erases personality and is hard to tune into a fun curve).
- **A dash of randomness over a deterministic core** (human-ruled) — occasional
  misplays keep opponents from being perfectly solvable and add a human feel,
  while the deterministic core stays fully unit-testable; the random layer sits
  behind a testable roll seam. (Spec 007 deferred stochastic play; this is the
  deliberate, bounded version of it.) *(The shipped code was in fact **un**bounded;
  spec 013 corrected it to match this stated intent — see the amendment note above.)*
- **Strategy is `OpponentProfile` data** (a `Copy` `AiStrategy` enum + a
  `misplay` rate), consistent with spec 007's "personality lives in the profile"
  design — no plumbing, no save change. The default opponent is deterministic
  (misplay 0) so existing tests stay deterministic.
- **React precisely when the player has stood; play to threshold while live.**
  The stood case is the clean, high-value, testable win; probabilistic reasoning
  about a live player's eventual total is deferred as fuzzy and low-value.
