# Spec: Bounded misplays — opponent AI competence floor (spec 013)

## Summary

A **spec-010 correction.** Spec 010 gave each opponent a per-turn **misplay rate**
so weaker opponents make human errors, and framed it as "the deliberate, **bounded**
version" (`specs/010-smarter-opponents/spec.md:144`). The shipped code never
bounded it: a misplay can turn the correct **hit** into a **stand** at *any* score,
so opponents routinely **stand on 0** and **concede from far behind** — catastrophic,
suicidal moves no player would make. This spec bounds the misplay model so a slip is
always a *believable* mistake, never a suicidal one, without removing the weakness
that makes early opponents beatable.

No engine, board-renderer, save-format, roster, or rate change — only the misplay
seam's decision logic and its tests.

## Goals

1. **No suicidal stands.** A misplaying opponent never stands on a low total (0,
   3, 6, …) and never concedes a round it could still contest. Weakness shows as
   *plausible* error, not self-destruction.
2. **Opponents still feel weak.** The believable open-position errors stay: an
   eager opponent still over-hits and busts, still fumbles a card, and still stands
   a touch early — so early opponents remain distinctly weaker than the masters.
3. **Masters unchanged.** Opponents with misplay 0.0 (the Magistrate, the
   Sovereign) play exactly as before.
4. **No regression.** Every other rule, screen, and save behaves as before; the
   deterministic decision core is untouched.

## Key behavior

### What a misplay may and may not do

A misplay is a recognizable human error while the **position is still open** — the
player is still live and the opponent is at or under 20, climbing toward its
threshold. In that window an opponent may:

- **stand a little early** — but only within a couple of its own stand threshold
  (e.g. stand on 15 when it would normally push to 17), never on a low total;
- **over-hit and bust** — the classic "one more card" greed;
- **fumble a card** — hit instead of playing the side card it should.

Once the position is **resolved** — the player has **stood** (their total is final)
or the opponent has **busted** (over 20, holding a card that would save it) — the
opponent plays its deterministic best move, with no misplay. Every deviation there
is pure self-harm (concede a chase, throw a lead, or fumble a save into a certain
bust), so a competent floor applies.

### Difficulty is unchanged

The per-opponent misplay **rates** and stand **thresholds** are unchanged. Weaker
opponents still slip more often; the difference is that a slip is now survivable
for them, not a gift to the player. (Whether the rates then *feel* right is the
tracked balance pass, not this spec.)

## Design requirements

- **A misplay is bounded, not suppressed.** Keep spec 010's testable seam
  (`opponent_action(roll)` — a pure function of state + one injected roll); the
  masters' misplay-0 path stays exactly as is.
- **The competence floor is a property, tested against the whole roster** — not a
  hand-checked example. Guard the actual invariant, including the *correct* low
  stands (against a busted/low stood player, and a forced full-table stand) so the
  test does not false-fail on a good move.
- **Surfaced, not silent.** Two existing spec-010 tests encode the old, buggy
  contract; they are re-authored deliberately (per `CLAUDE.md`: "if a test seems
  wrong, flag it and ask"), with the change recorded in `DECISIONS.md` and a
  pointer on spec 010's acceptance evidence.

## Acceptance criteria

- [x] A misplaying opponent never stands below `effective_threshold − 2` while the
      player is live and its table isn't full — verified as a property over every
      roster opponent, and by a regression test pinning the exact reported bug
      (live player, opponent score 0, misplay 1.0 → hits, never stands).
      *(`no_roster_opponent_misplays_into_a_suicidal_stand_while_live` over all
      `OPPONENTS`; `a_misplay_never_stands_on_a_low_total`;
      `the_timid_stand_is_bounded_to_the_band` pins the band edge 14→hit/15→stand.
      Mutation-checked: reverting to the unbounded `Hit => Stand` turns these red.)*
- [x] Against a **stood** player, a misplaying opponent plays its deterministic
      best move for every move shape (ahead → stand, behind → hit/chase, winning
      card → play it); the over-20 recovery play is likewise never fumbled.
      *(`a_resolved_position_is_played_straight_never_misplayed` — all four shapes,
      misplay 1.0. Mutation-checked: removing the `position_is_open` gate turns it
      red.)*
- [x] The believable open-position errors still fire: a greedy over-hit
      (`stand → hit`) still occurs while live, and a timid early stand still occurs
      within the band — so weak opponents remain weaker than the masters.
      *(`a_greedy_over_hit_still_fires_while_the_player_is_live`,
      `a_live_card_fumble_still_fires_through_the_seam`,
      `misplay_deviates_each_best_move_legally`,
      `opponent_action_misplays_below_the_rate_and_plays_best_at_or_above`. Live
      driver: Greeb hit 0→7→11, then played the board-aware +1 to win once the
      player stood.)*
- [x] Opponents with misplay 0.0 are provably unaffected (the existing
      "default opponent never misplays" guard still holds).
      *(`the_default_opponent_never_misplays`,
      `misplay_rates_are_valid_and_the_default_is_deterministic`; `opponent.rs`
      rates untouched.)*
- [x] `cargo test` green (the new regression + property + boundary +
      position-resolved + greed-still-fires tests, plus the re-authored spec-010
      tests); `cargo build` no new warnings; no panics in play. *(245 passed, 0
      failed; build 0 warnings; driver spot-check — full round, no panic, save/resume
      intact.)*

## Resolved decisions

- **Restore spec 010's stated "bounded" intent** — this corrects a divergence from
  spec 010, it is not new scope.
- **Bounded, not zeroed.** Weakness is kept; only the *catastrophic* outcomes are
  removed. `MISPLAY_TIMID_MARGIN = 2` (a rookie's worst timid stand is 13 — weak
  but recoverable); the greedy over-hit bust and the card fumble are kept as
  believable flavor; misplay is gated off once the position is resolved. Rationale
  in `DECISIONS.md` (spec 013).
- **Not a difficulty setting.** Making the misplay model sane is a prerequisite for
  the tracked global difficulty option, not that feature.
