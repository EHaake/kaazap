# Plan: Bounded misplays (spec 013)

## Root cause

The misplay seam (`src/game.rs`, `misplay` ~597) maps the deterministic best move
to a "human error." Its `Hit => Stand` arm (`game.rs:601`) is **unbounded**. The
best move is `Hit` whenever the opponent is below its stand threshold — including
the **first turn at score 0** — so a slip stands on 0. A fresh roll is drawn every
opponent turn (`play_opponent_turn`, `game.rs:644`), so the chance of *some*
premature stand compounds across the climb (Greeb at misplay 0.25 ≈ 25% to stand on
0 on turn one). The same arm flips the chase `Hit → Stand` against a **stood**
player (`decide_vs_stood_player`, `game.rs:483`) — the "standing when far behind"
symptom. Lowering the rates makes suicide rarer, not gone: the model is wrong.

**Qualified invariant** (the loose "never stands below 15" is false): while the
player is **live** and the opponent's table isn't full, the deterministic policy
never stands below `effective_threshold` (min 15); every catastrophic sub-threshold
stand comes from the misplay `Hit => Stand` arm. Two non-catastrophic low stands
are *correct* and must survive the fix and its tests — against a busted/low **stood**
player (`p > 20` or `score > p` → Stand, `game.rs:458,462`) and a forced full-table
stand.

## The fix — two layers in `src/game.rs`

**Layer 1 — when a slip can happen at all.** Gate in `opponent_action` (~583):
apply misplay only while the **position is open**.

```rust
fn opponent_action(&self, roll: f32) -> OpponentAction {
    let best = self.decide_opponent_move();
    if roll < self.opponent_profile.misplay && self.position_is_open() {
        self.misplay(best)
    } else {
        best
    }
}

/// A misplay is only a believable error while the outcome is still open: the
/// player is live (no fixed target to concede or throw) and the opponent is at or
/// under 20 (not fumbling a bust-saving recovery card). Once resolved, every
/// deviation is pure self-harm, so the AI plays its deterministic best.
fn position_is_open(&self) -> bool {
    !self.player.stood && self.opponent.score() <= 20
}
```

This eliminates "standing when far behind" (stood player) and the over-20 recovery
fumble (`PlayHand → Hit` while busted → certain bust).

**Layer 2 — what a slip looks like.** Bound the timid stand in `misplay` (~597):

```rust
const MISPLAY_TIMID_MARGIN: i32 = 2; // near the AI logic / other AI consts

fn misplay(&self, best: OpponentAction) -> OpponentAction {
    match best {
        OpponentAction::Stand if !self.opponent.table_full() => OpponentAction::Hit,
        OpponentAction::Stand => OpponentAction::Stand,
        // Chicken out — but only a plausible early stand, within MISPLAY_TIMID_MARGIN
        // of the threshold. Below the band, hitting is so clearly right that no one
        // would stand, so a slip just hits (never a suicidal low stand, e.g. 0).
        OpponentAction::Hit
            if self.opponent.score() >= self.effective_threshold() - MISPLAY_TIMID_MARGIN =>
        {
            OpponentAction::Stand
        }
        OpponentAction::Hit => OpponentAction::Hit,
        OpponentAction::PlayHand { .. } => OpponentAction::Hit,
    }
}
```

Both layers are required: at score 0 the position *is* open, so only the timid bound
stops stand-on-0; the position gate stops the stood/recovery catastrophes the bound
never sees. Placement is deliberate — `misplay` owns *what a slip looks like*
(re-authoring `misplay_deviates_each_best_move_legally`), `opponent_action` owns
*when a slip can happen* (the new regression test targets `opponent_action`).

## Kept as intended flavor

- **Greedy over-hit** (`Stand => Hit` while live) — the classic beginner error and
  the visible weakness; self-limiting, and the roster is anti-correlated (kesh/masters
  eff 19 barely misplay; rookies eff 15–16 bust under half the time). Per-round
  greedy-bust ≈ 4–12%.
- **Card fumble** (`PlayHand => Hit` while live) — 20 isn't a locked win, so this is
  believable.
- **Masters** (misplay 0.0) — `roll < 0.0` is never true; untouched.

## Tests (`src/game.rs` test module)

New: regression (live, score 0, misplay 1.0 → Hit); per-profile property (over all
`OPPONENTS`, live + non-full table, no misplay-stand below `t − MISPLAY_TIMID_MARGIN`);
boundary (stand at `t−1`, bounded to Hit at `t − MARGIN − 1`); no-misplay-when-resolved
(stood player, all three shapes; over-20 recovery not fumbled); greed-still-fires
(`score ≥ t`, live → Hit).

Re-authored (encode the OLD contract — surfaced per CLAUDE.md):
`misplay_deviates_each_best_move_legally` (~1455, asserted `misplay(Hit)==Stand` at
score 10), `opponent_action_misplays_below_the_rate_and_plays_best_at_or_above`
(~1442, built on a stood player → move to a live player in the band), and the stale
comment on `full_match_terminates_with_a_maximally_misplaying_opponent` (~1165).

Test helpers already present: `board_at(oppScore, playerStood, playerScore, hand)`
and `opponent_at(score, hand)` (confirm signatures when implementing). An empty hand
makes best `Hit` for all `score < effective_threshold` (no play-to-20), which the
property/boundary tests rely on.

## Files

- `src/game.rs` — `MISPLAY_TIMID_MARGIN`, `position_is_open`, the `opponent_action`
  gate, the `misplay` bound; new + re-authored tests.
- `docs/opponents.md` — misplay description (bounded, never suicidal).
- `DECISIONS.md` — spec-013 entry (to `main` at close-out).
- `specs/010-smarter-opponents/spec.md` — a pointer note (bounded by spec 013;
  two tests re-authored).
- **No change:** engine values, board renderer, `opponent.rs` (rates/thresholds),
  save format.

## Alternative weighed & rejected

Modelling misplay as a perturbed `effective_threshold` re-running the deterministic
policy is conceptually tidier (and gets "no misplay vs stood" for free), but it is
**outcome-equivalent on the catastrophic cases** (a greedy over-hit busts the same
regardless of the perturbed threshold) while being a larger rewrite of an
already-tested seam — it fails the constitution's simplicity mandate. The bounded
action-flip is the smaller, correct change.

## Verification

`cargo build` (no new warnings) + `cargo test` (verbatim). Driver spot-check
(`run-kaazap`, real `profile.json`/`saves/` backed up first): the first campaign
opponent no longer stands on 0 or concedes from behind; masters still play tight.
