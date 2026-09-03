# Plan: Smarter (board-aware) opponent AI

Technical design for `spec.md`. The master plan
(`~/.claude/plans/iterative-meandering-blum.md`) mirrors this.

## Shape of the change

`decide_opponent_move` (`game.rs:422`) gains a **board-aware branch** and
**per-strategy behavior**, staying a deterministic pure function of the game
state. A thin **randomness seam** wraps it so an opponent occasionally makes a
legal-but-suboptimal move. Strategy + a misplay rate are two new `Copy` fields
on `OpponentProfile`. No plumbing (the decision fn already has the whole
`GameState`), no save change (the save persists only the opponent `id`).

## Data model — `src/opponent.rs`

`OpponentProfile` (`opponent.rs:17-32`, all-`Copy`) gains:
```rust
pub strategy: AiStrategy,   // deterministic policy archetype
pub misplay: f32,           // 0.0..=1.0 — chance of an imperfect move

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiStrategy { Basic, Aggressive, Cautious, Calculating }
```
Unit-variant enum → `Copy`, const-constructible; **no serde** (profiles rebuild
from `id`). Roster (`OPPONENTS`, easy→hard) + `DEFAULT_OPPONENT` each gain the
two fields:

| opponent | strategy | misplay |
|---|---|---|
| Greeb (Rookie) | Basic | 0.25 |
| Vessa Korr (Scrapper) | Aggressive | 0.15 |
| Old Toran (Veteran) | Cautious | 0.10 |
| Rix Vandal (Ace) | Calculating | 0.05 |
| The Magistrate (Master) | Calculating | 0.0 |
| DEFAULT_OPPONENT | Basic | **0.0** (deterministic baseline for tests) |

Archetypes: **Basic** = today's threshold play + the board-aware fix;
**Aggressive** = higher push target, chases hard (bust risk); **Cautious** =
stands earlier, only chases a stood player when it can win safely;
**Calculating** = minimal safe winning total + tiebreaker to steal a tie.
Misplay rates are tunable balance data.

## Decision core — `src/game.rs`

Round-resolution facts the AI reasons against (`finalize_round`, `game.rs:294`):
closest to 20 without busting wins; equal totals tie **unless one side alone has
a tiebreaker in play** (`has_tiebreaker_in_play`, `player.rs:37`); >20 loses.
`player.bust` is resolution-time only, so mid-turn read `self.player.score()` +
`self.player.stood`, not `bust`.

`decide_opponent_move(&self) -> OpponentAction` **stays the deterministic core**
(existing tests keep targeting it). New order:
1. `opponent.table_full()` → `Stand` (unchanged).
2. `opponent.score() > 20` → `best_recovery_play` else `Stand` (unchanged —
   self-preservation first).
3. **NEW — if `self.player.stood`:** `decide_vs_stood_player(s, p)` where
   `p = self.player.score()`:
   - `s > p && s <= 20` → `Stand` (already won — the headline fix).
   - `s == p` → try to exceed `p` (a card that lands `> p, <= 20`, else `Hit`),
     unless a lone opponent tiebreaker already wins the tie → `Stand`.
   - `s < p` → play a card landing a winning total (`> p, <= 20`) if any, else
     `Hit`. Target selection is strategy-flavored (Calculating → minimal `> p`;
     Aggressive → highest safe; Cautious → only if a safe win exists, else `Hit`
     but never an avoidable-bust hit).
4. **else (player live):** `decide_vs_live_player(s)` = today's play (reach-20
   via `first_hand_index`+`can_play_as`; `s >= effective_threshold` → `Stand`;
   else `Hit`), where Aggressive/Cautious shift the effective threshold off
   `stand_threshold`.

Reuses `best_recovery_play` (`game.rs:465`), `first_hand_index` (`game.rs:483`),
`Card::playable_values`/`can_play_as` (`card.rs:54,65`). New private helpers
`decide_vs_stood_player`/`decide_vs_live_player` read
`self.opponent_profile.strategy`. Because the not-stood branch is today's
behavior and `DEFAULT_OPPONENT` is `Basic`, the player-at-0 tests reduce to
current behavior.

## Randomness seam — `src/game.rs`

- `opponent_action(&self, roll: f32) -> OpponentAction`:
  `if roll < self.opponent_profile.misplay { self.misplay(best) } else { best }`,
  `best = self.decide_opponent_move()`. Deterministic given `roll` → testable.
- `misplay(&self, best) -> OpponentAction` — a legal, suboptimal deviation:
  `Stand → Hit` (only if it can still draw — else keep `Stand`), `Hit → Stand`,
  `PlayHand{..} → Hit`. Always legal.
- `play_opponent_turn` (`game.rs:496`) calls
  `self.opponent_action(rand::random_range(0.0f32..1.0))` instead of
  `decide_opponent_move()` (the one production line that gains randomness;
  `rand::random_range` matches `draw_dealer_card`, `game.rs:664`). With
  `DEFAULT_OPPONENT.misplay == 0.0`, `roll < 0.0` is never true, so the
  end-to-end AI tests driving `update()` stay deterministic.

## Testing — `src/game.rs` test module

- **Existing `ai_*` tests unchanged** — they call `decide_opponent_move` (still
  the deterministic core) with the player at 0 / not stood (`opponent_at`,
  `game.rs:1013`), exercising the live branch = today's behavior. All stay green.
- **New both-boards helper** `board_at(player_total, player_stood, opp_total,
  opp_hand)` — seeds `gs.player.dealer_row` (like `player_over_at_23`,
  `game.rs:1205`) so `player.score()` = target and sets `player.stood`, plus the
  opponent side (like `opponent_at`). Pure `pub`-field assignment.
- **Board-aware policy tests** (deterministic): ahead of a stood player → Stand;
  behind with a winning card → plays it; behind without → Hit; tie with/without
  the tiebreaker; per-archetype differences (Calculating minimal-safe vs
  Aggressive high; Cautious won't over-hit).
- **Seam tests:** `opponent_action(roll)` misplays for `roll < misplay`, best
  for `roll >= misplay`; `misplay(best)` returns the expected legal deviation;
  `DEFAULT_OPPONENT` never deviates.

## Docs & close-out

- **`docs/opponents.md`** — rewrite "How the opponent plays" (remove the "never
  looks at the player's board / deferred" claim at `:22-24`; document the
  board-aware order + strategies); "two levers" → **three** (threshold, deck,
  strategy); add strategy + misplay to the roster table + per-opponent notes.
- Post-merge on `main`: **`ROADMAP.md`** — "Smarter / board-aware opponent AI"
  → Shipped (spec 010); note the difficulty setting is now unblocked.
  **`DECISIONS.md`** — a spec-010 entry (board-aware + archetypes +
  deterministic-core-with-a-misplay-seam).

## Files

Modified: `src/game.rs` (decision core + `opponent_action`/`misplay` seam +
`play_opponent_turn` + tests), `src/opponent.rs` (`AiStrategy` + `misplay` +
roster/`DEFAULT_OPPONENT`), `docs/opponents.md`.
No change expected: `src/card.rs`, `src/player.rs`, `src/save.rs`, the screens.
