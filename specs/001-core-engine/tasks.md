# Tasks: Core Pazaak Card Engine

**Status**: Approved
**Implements**: plan.md in this directory

Ordered, small, independently verifiable. Each task is completable (and
testable) on its own. If a session ends mid-list, resume by finding the
first unchecked task — don't re-verify everything above it unless
something looks off.

<!-- WARNING, worth leaving in: once implementation starts, this file
gets written by more than one party — whoever's steering adds scope and
reshuffles tasks; the implementer checks boxes and adds findings. Never
edit this file from a stale copy. Prefer small, targeted edits over
regenerating it wholesale — a full replacement silently discards
whatever the other party added since your copy was taken. -->

Per the constitution: every task ends with `cargo build` and `cargo test`,
both green, actual output reported — not paraphrased. The *Verify:* lines
below are the task-specific checks on top of that baseline. One commit
per task, referencing the task ID. Draft PR #1 already tracks this
branch — push after each commit so its diff stays current.

**Review cadence**: Phases 1–2 are foundational — stop for review after
**every task**. Phases 3–4 are mechanical — stop for review after
**each phase**.

Interim-state strategy (why Phase 1 has this shape): every task must
leave the build green, but swapping out `LogicCard` breaks `player.rs`,
`game.rs`, and `board.rs` at once. So T001 adds the new types *alongside*
the old (a lib crate, so unwired items don't even warn), and T002 is the
one atomic migration. Between T002 and T007, hands stay fixed (today's
values as `Plus` cards) so behavior is unchanged until dealing lands.

---

## Phase 1 — Card model (`card.rs`, `player.rs`)

Review: after every task.

- [x] **T001 — New card types, labels, and the default deck (`card.rs`)**
  Add alongside the existing `LogicCard` (untouched this task; deleted in
  T002):
  - `FlipKind { TwoFour, ThreeSix }` and
    `Card { Dealer(u8), Plus(u8), Minus(u8), PlusMinus(u8), Flip(FlipKind), Tiebreaker }`,
    per plan.
  - `Card::label() -> String` — the one source of truth for card text:
    `Dealer(7)` → `"7"`, `Plus(4)` → `"+4"`, `Minus(3)` → `"-3"`,
    `PlusMinus(2)` → `"±2"`, flips → `"2&4"` / `"3&6"`,
    `Tiebreaker` → `"±1T"`.
  - `PlayedCard { card: Card, value: i8 }`.
  - `pub const DEFAULT_SIDE_DECK: [Card; 10]` — +2, +4, −2, −4, ±1, ±3,
    ±6, 2&4, 3&6, tiebreaker (spec's resolved composition; the one
    tunable constant).

  Tests (`#[cfg(test)]` in `card.rs`): `label_*` covering every variant;
  `deck_*` asserting the deck is exactly the spec's 10-card multiset.
  *Verify: `cargo test card::` — label tests for all six variants and
  the deck-composition test pass.*

- [x] **T002 — Migrate rows/hands to the new types; delete `LogicCard` (`player.rs`, `game.rs`, `board.rs`)**
  *Finding (T002): `PlayerState.played_card` is never set `true` anywhere —
  the "card play passes the turn" branch in `resolve_after_action` is dead
  code; playing a card currently leaves the turn with the player. Resolved
  at T003 (human ruling): behavior kept — it matches real Pazaak — and the
  dead flag and branch deleted. Recorded in plan.md.*
  The pivot task — mechanical migration, no new behavior:
  - `PlayerState`: `dealer_row`/`played_row` → `Vec<PlayedCard>`,
    `hand` → `Vec<Option<Card>>`; `score()` sums `value` over both rows.
  - Dealer hits push `PlayedCard { card: Dealer(n), value: n as i8 }`;
    extract the duplicated `rand::random_range(0..=10)` in
    `player_hit`/`opponent_hit` into one shared draw helper (one source
    of truth; the plan's test list presumes it exists).
  - Interim fixed hands in `GameState::new()` keep today's values as
    `Plus` cards (5/3/6/2 and 2/6/1/4) — random dealing arrives in T007.
  - `play_card`/`opponent_play_card`: commit `Plus`/`Minus` at face
    value; other kinds are inert no-ops for now (they can't occur in
    hands until T007; real handling lands in T003–T005).
  - AI predicate interim: `Plus(n)` hitting exactly 20, matching current
    behavior; generalized in T008.
  - `board.rs`: minimal mechanical fix-ups only — dealer/played rows
    render `value`, hand renders `Card::label()`. Full display rules in
    T009.
  - Delete `LogicCard`, plus the dead `CardKind` and `Owner` enums
    (unused anywhere — riding along; flagged at review).

  Tests: `score_*` across mixed card kinds on both rows;
  `dealer_draw_*` bounds test (output stays 0–10 across many draws).
  *Verify: `cargo test` fully green including the new `score_`/
  `dealer_draw_` tests; `grep -rn LogicCard src/` returns nothing;
  manual smoke run — a round plays exactly as before.*

## Phase 2 — Game logic (`game.rs`)

Review: after every task.

- [x] **T003 — Shared commit path**
  `commit_play(side, index, value: i8)` removes the card from the hand
  and pushes `PlayedCard` — replacing the duplicated
  `play_card`/`opponent_play_card` bodies. `OpponentAction::PlayHand`
  gains a `value: i8` field (the AI resolves sign itself — no prompt
  phase for it); `GameAction::PlayHand` unchanged. Fixed-value cards
  commit at face value through this path.
  Tests: `commit_*` — commit empties the hand slot, pushes the right
  `card` identity and the exact passed `value` (a negative value lands
  negative); the slot stays empty afterward.
  *Verify: `cargo test commit_` green; full `cargo test` green.*

- [x] **T004 — Sign-choice phase (± and tiebreaker)**
  - `GamePhase::AwaitingSignChoice { hand_index }`, entered from
    `apply_game_action` when the played card is `PlusMinus` or
    `Tiebreaker` (card stays in hand until committed).
  - `GameAction::ChooseSign { positive: bool }`; `game_action_from_key`
    becomes phase-aware: in this phase `+`/`1` → positive, `-`/`2` →
    negative, `c` cancels back to `PlayerTurn` with the card unspent;
    all other keys ignored in this phase; every other phase keeps
    today's key mapping untouched.
  - `ChooseSign` commits via `commit_play` with the signed magnitude
    (`Tiebreaker` → ±1).
  - `update()` must leave `AwaitingSignChoice` alone (it falls into the
    existing `_ => {}` — keep it that way).

  Tests: `sign_*` — playing a ± card enters the phase with the hand
  unchanged; choosing + and − each commit the correctly-signed value
  and empty the slot; cancel restores `PlayerTurn` with the card still
  in hand; `d`/`s` in this phase produce no action.
  *Verify: `cargo test sign_` green; full `cargo test` green.*

- [x] **T004a — Surface new-game/menu controls in the help overlay**
  *Discovered scope (human observation): starting a new game after game
  over (`g`) and returning to the menu (`x`) already work in code but were
  absent from the `?` controls overlay — a discoverability gap, not a
  missing mechanic. Added `G` and `X` lines to
  `assets/game_overlay_text.txt` and bumped the overlay's `content_height`
  to fit.*
  *Verify: manual — `?` in game shows G and X lines inside an intact
  border. Done via the run-kaazap driver.*

- [x] **T005 — Flip application (2&4, 3&6)**
  - Flips route through the commit path as
    `PlayedCard { card: Flip(k), value: 0 }`, then `apply_flip(kind)`:
    negate `value` on every `PlayedCard` in all four rows where
    `value.abs()` matches the kind's pair (2/4 or 3/6), skipping zeros.
    No sub-prompt — flips play immediately.
  - `resolve_after_action` gains the spec ruling: a standing player
    whose recalculated total goes over 20 busts (round ends with the
    correct outcome). Both sides checked after a flip.

  Tests: `flip_*` — inverts matching values on both sides including
  dealer rows; leaves non-matching and zero values alone; the flip card
  itself contributes 0 to score; a standing player pushed over 20 busts
  and the round resolves against them; totals reflect the inversion
  immediately.
  *Verify: `cargo test flip_` green; full `cargo test` green.*

- [x] **T006 — Tiebreaker resolution**
  *Mutation check performed (two mutations, both confirmed red, rule
  restored): inverting the winner mapping turned the three one-sided
  tests red; making two tiebreakers award a win turned the both-sides
  test red.*
  In `finalize_round`'s `Tied` branch: exactly one side with a
  `Tiebreaker` in its `played_row` wins the round; both or neither →
  the tie stands. (`setup_next_round` clearing `played_row` already
  scopes "in play" to the current round.)
  Tests: `tiebreaker_*` — one side in play wins; both sides → tie;
  neither → tie. Mutation-check per plan: temporarily invert the rule,
  confirm the one-sided test goes red, restore, and report that this
  was done.
  *Verify: `cargo test tiebreaker_` green, and the mutation check
  reported (test observed red under the inverted rule).*

- [ ] **T007 — Random hand dealing**
  - `deal_hand(rng) -> Vec<Option<Card>>` in `card.rs` using `rand`'s
    `choose_multiple`: 4 distinct cards from `DEFAULT_SIDE_DECK`.
  - Called for both sides, independently, in `GameState::new()` and
    `new_game()` — replacing the interim fixed hands.
    `setup_next_round()` keeps hands untouched.

  Tests: `deal_*` — a hand is 4 `Some` cards, all distinct, all members
  of the deck; `new_game()` re-deals a full 4-card hand after cards
  were played; `setup_next_round()` leaves a partially-spent hand
  exactly as it was (the no-mid-match-redraw regression test).
  *Verify: `cargo test deal_` green; full `cargo test` green; manual:
  new games show different hands across runs.*

- [ ] **T008 — Opponent AI predicate generalization**
  The "hits exactly 20" predicate becomes "any playable value of this
  card equals the target": `Plus(n)` → +n, `Minus(n)` → −n,
  `PlusMinus(n)` → ±n, `Tiebreaker` → ±1, `Flip` → never playable
  (known limitation, per plan). The chosen value rides in
  `OpponentAction::PlayHand { index, value }`. Stand threshold and hit
  logic unchanged.
  Tests: `ai_*` — at 18 with ±2 in hand the AI plays it as +2; at 19
  with a tiebreaker it plays +1; a flip card is never selected even
  when nothing else is playable.
  *Verify: `cargo test ai_` green; full `cargo test` green.*

## Phase 3 — Rendering (`board.rs`)

Review: after the phase (both tasks done).

- [ ] **T009 — Card display rules**
  - Hand cards render `Card::label()` — kinds distinguishable before
    play ("±3" vs "+3" vs "2&4"), the spec's design requirement,
    carried by text since `Frame` has no color.
  - Played/dealer rows: flip cards render their label; everything else
    renders its signed `value` (a ± played as −3 shows "-3").
  - Remove the stale `c.value != 0` filter on the opponent's played
    row — under the new model a 0-value played card is precisely a flip
    card, which must render, not hide.

  *Verify: manual — `cargo run`; across a few games confirm: a ± card
  in hand shows "±N" and, played as minus, shows "-N"; a played flip
  shows "2&4"/"3&6" and both totals visibly change; dealer cards
  unchanged; opponent hand still shows "?".*

- [ ] **T010 — Sign-choice sub-prompt**
  During `AwaitingSignChoice`, the existing turn-text area renders e.g.
  `Play +3 or -3?  [+/-]  (c to cancel)` with the actual magnitude
  (±1 for the tiebreaker). The discoverability requirement is satisfied
  by the prompt naming its own keys.
  *Verify: manual — select a ± card: prompt appears with the right
  magnitude; `+`, `-`, and `c` each behave as labeled; prompt clears
  after commit or cancel and normal turn text returns.*

## Phase 4 — Housekeeping

Review: after the phase.

- [ ] **T011 — Drop `rusty_time`**
  Remove it from `Cargo.toml` (unused in `src/`, per constitution);
  regenerate `Cargo.lock` via `cargo build` only — never hand-edited.
  *Verify: `rusty_time` absent from `Cargo.toml` and `Cargo.lock`;
  `cargo build` and `cargo test` green.*

- [ ] **T012 — Acceptance sweep**
  Walk spec.md's acceptance criteria and check every box: full
  `cargo build` (confirming no new warnings vs `main`), full
  `cargo test`, and a manual best-of-3 playthrough exercising every
  card kind (±, flip, and the tiebreaker's tie-win if it can be set up
  in play).
  *Verify: every acceptance checkbox in spec.md checked with evidence;
  build/test output reported verbatim; PR ready to mark non-draft —
  pending the human's call, per the constitution.*

---

## Handoff note

Read `CLAUDE.md`, then `specs/001-core-engine/spec.md`, `plan.md`, and
this file. Implement in order starting at T001. **Stop for review after
every task in Phases 1–2; after each full phase for Phases 3–4.** One
commit per task, referencing the task ID; push after each commit —
draft PR #1 tracks the running diff. Sub-letter (`T00Xa`) any genuinely
new scope discovered mid-build and flag it — never silently absorb it,
never renumber existing tasks.
